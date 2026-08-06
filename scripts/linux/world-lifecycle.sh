#!/usr/bin/env bash
set -Eeuo pipefail

LINUX_MANAGED_STATE_SNAPSHOT_ROOT=""
: "${DRY_RUN:=0}"

linux_snapshot_manifest_path() {
    printf '%s/manifest.tsv\n' "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}"
}

linux_snapshot_service_path() {
    printf '%s/services.tsv\n' "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}"
}

linux_snapshot_account_state_path() {
    printf '%s/account.tsv\n' "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}"
}

linux_snapshot_acl_state_path() {
    printf '%s/acl.tsv\n' "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}"
}

linux_manifest_field_v1() {
    local value="${1:-}"
    if [[ -n "${value}" ]]; then
        printf '%s' "${value}"
    else
        printf '%s' "-"
    fi
}

linux_require_world_context() {
    local missing=0
    local var
    for var in \
        WORLD_AGENT_BIN_PATH \
        GATEWAY_BIN_PATH \
        ACL_HELPER_SOURCE_PATH \
        LIFECYCLE_EXECUTOR_BIN_PATH \
        SCRIPT_DIR \
        SERVICE_PATH \
        SOCKET_PATH \
        ACL_DROPIN_PATH \
        SERVICE_UNIT_CONTENT \
        SOCKET_UNIT_CONTENT \
        SOCKET_DROPIN_CONTENT \
        INVOKING_USER \
        SUBSTRATE_CLI_BIN_PATH \
        SUBSTRATE_STATE_PATH \
        WORLD_DEPS_ROOT_PATH \
        WORLD_DEPS_BIN_PATH \
        SOCKET_FS_PATH; do
        if [[ -z "${!var:-}" ]]; then
            printf 'world-lifecycle missing required context: %s\n' "${var}" >&2
            missing=1
        fi
    done
    [[ "${missing}" -eq 0 ]]
}

linux_snapshot_path() {
    local label="$1"
    local target="$2"
    local type="absent"
    local mode=""
    local backup=""
    local owner=""
    local group=""

    if sudo_cmd test -e "${target}" >/dev/null 2>&1 || sudo_cmd test -L "${target}" >/dev/null 2>&1; then
        if sudo_cmd test -L "${target}" >/dev/null 2>&1; then
            type="symlink"
            backup="${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}/${label}.backup"
            sudo_cmd cp -a --no-dereference "${target}" "${backup}"
        elif sudo_cmd test -d "${target}" >/dev/null 2>&1; then
            type="dir"
        elif sudo_cmd test -S "${target}" >/dev/null 2>&1; then
            type="socket"
        else
            type="file"
            backup="${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}/${label}.backup"
            sudo_cmd cp -a --no-dereference "${target}" "${backup}"
        fi
        if [[ "${type}" != "symlink" ]]; then
            mode="$(sudo_cmd stat -c '%a' "${target}" 2>/dev/null || true)"
        fi
        owner="$(sudo_cmd stat -c '%u' "${target}" 2>/dev/null || true)"
        group="$(sudo_cmd stat -c '%g' "${target}" 2>/dev/null || true)"
        if [[ -n "${backup}" && -z "${mode}" ]]; then
            mode="-"
        fi
    fi

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "${label}" \
        "${target}" \
        "${type}" \
        "$(linux_manifest_field_v1 "${mode}")" \
        "$(linux_manifest_field_v1 "${backup}")" \
        "$(linux_manifest_field_v1 "${owner}")" \
        "$(linux_manifest_field_v1 "${group}")" >>"$(linux_snapshot_manifest_path)"
}

linux_snapshot_service_state() {
    local unit="$1"
    local enabled="unknown"
    local active="unknown"
    if [[ ${DRY_RUN} -eq 0 ]]; then
        enabled="$(systemctl is-enabled "${unit}" 2>/dev/null || true)"
        active="$(systemctl is-active "${unit}" 2>/dev/null || true)"
    fi
    case "${enabled}" in
        enabled|enabled-runtime|disabled|masked|masked-runtime|static|unknown) ;;
        *)
            printf 'world-lifecycle unsupported pre-state enabled value %s for %s\n' "${enabled}" "${unit}" >&2
            return 1
            ;;
    esac
    case "${active}" in
        active|inactive|unknown) ;;
        *)
            printf 'world-lifecycle unsupported pre-state active value %s for %s\n' "${active}" "${unit}" >&2
            return 1
            ;;
    esac
    printf '%s\t%s\t%s\n' "${unit}" "${enabled}" "${active}" >>"$(linux_snapshot_service_path)"
}

linux_snapshot_account_state() {
    local group_exists="0"
    local user_member="0"

    if getent group "${SUBSTRATE_GROUP}" >/dev/null 2>&1; then
        group_exists="1"
    fi

    if [[ -n "${INVOKING_USER}" && "${INVOKING_USER}" != "root" ]] \
        && id "${INVOKING_USER}" >/dev/null 2>&1 \
        && user_in_group "${INVOKING_USER}" "${SUBSTRATE_GROUP}"; then
        user_member="1"
    fi

    printf '%s\t%s\t%s\t%s\n' \
        "${SUBSTRATE_GROUP}" \
        "${INVOKING_USER}" \
        "${group_exists}" \
        "${user_member}" >>"$(linux_snapshot_account_state_path)"
}

linux_snapshot_acl_state() {
    local label="$1"
    local target="$2"
    local recursive="${3:-0}"
    local backup=""
    local -a getfacl_args=(getfacl -p)

    if ! sudo_cmd test -e "${target}" >/dev/null 2>&1; then
        printf '%s\t%s\t%s\t%s\n' \
            "${label}" \
            "${target}" \
            "${recursive}" \
            "${backup}" >>"$(linux_snapshot_acl_state_path)"
        return 0
    fi

    if ! command -v getfacl >/dev/null 2>&1; then
        printf 'world-lifecycle cannot snapshot ACL for pre-existing %s without getfacl\n' "${target}" >&2
        return 1
    fi
    if ! command -v setfacl >/dev/null 2>&1; then
        printf 'world-lifecycle cannot restore ACL for pre-existing %s without setfacl\n' "${target}" >&2
        return 1
    fi

    if [[ "${recursive}" == "1" ]]; then
        getfacl_args+=(-R)
    fi
    backup="${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}/${label}.acl"
    sudo_cmd "${getfacl_args[@]}" "${target}" >"${backup}"
    printf '%s\t%s\t%s\t%s\n' \
        "${label}" \
        "${target}" \
        "${recursive}" \
        "${backup}" >>"$(linux_snapshot_acl_state_path)"
}

linux_restore_path() {
    local label="$1"
    local target="$2"
    local type="$3"
    local mode="$4"
    local backup="$5"
    local owner="$6"
    local group="$7"
    local install_mode=""
    local target_parent=""

    [[ "${mode}" == "-" ]] && mode=""
    [[ "${backup}" == "-" ]] && backup=""
    [[ "${owner}" == "-" ]] && owner=""
    [[ "${group}" == "-" ]] && group=""
    install_mode="${mode:-0755}"

    case "${type}" in
        absent)
            sudo_cmd rm -f "${target}" || true
            sudo_cmd rm -rf "${target}" || true
            ;;
        dir)
            sudo_cmd install -d -m"${install_mode}" "${target}"
            if [[ -n "${owner}" && -n "${group}" ]]; then
                sudo_cmd chown "${owner}:${group}" "${target}"
            fi
            ;;
        file|symlink)
            if [[ -n "${backup}" && ( -e "${backup}" || -L "${backup}" ) ]]; then
                target_parent="$(dirname -- "${target}")"
                sudo_cmd mkdir -p "${target_parent}"
                sudo_cmd rm -rf "${target}" || true
                sudo_cmd cp -a "${backup}" "${target}"
                if [[ -n "${owner}" && -n "${group}" ]]; then
                    if [[ "${type}" == "symlink" ]]; then
                        sudo_cmd chown -h "${owner}:${group}" "${target}"
                    else
                        sudo_cmd chown "${owner}:${group}" "${target}"
                    fi
                fi
            fi
            ;;
        socket)
            sudo_cmd rm -f "${target}" || true
            ;;
        *)
            printf 'world-lifecycle restore skipped unknown snapshot type %s for %s (%s)\n' \
                "${type}" "${target}" "${label}" >&2
            ;;
    esac
}

linux_restore_service_state() {
    local unit="$1"
    local enabled="$2"
    local active="$3"

    if [[ "${active}" == "active" && ( "${enabled}" == "masked" || "${enabled}" == "masked-runtime" ) ]]; then
        sudo_cmd systemctl unmask "${unit}"
        linux_start_service_unit "${unit}"
        case "${enabled}" in
            masked)
                sudo_cmd systemctl mask "${unit}"
                ;;
            masked-runtime)
                sudo_cmd systemctl mask --runtime "${unit}"
                ;;
        esac
        return 0
    fi

    case "${enabled}" in
        enabled)
            sudo_cmd systemctl unmask "${unit}"
            sudo_cmd systemctl enable "${unit}"
            ;;
        enabled-runtime)
            sudo_cmd systemctl unmask "${unit}"
            sudo_cmd systemctl enable --runtime "${unit}"
            ;;
        static|unknown)
            ;;
        disabled)
            sudo_cmd systemctl unmask "${unit}"
            sudo_cmd systemctl disable "${unit}"
            ;;
        masked)
            sudo_cmd systemctl mask "${unit}"
            ;;
        masked-runtime)
            sudo_cmd systemctl mask --runtime "${unit}"
            ;;
        *)
            printf 'world-lifecycle unsupported restore enabled value %s for %s\n' "${enabled}" "${unit}" >&2
            return 1
            ;;
    esac

    case "${active}" in
        active)
            linux_start_service_unit "${unit}"
            ;;
        inactive)
            sudo_cmd systemctl stop "${unit}"
            ;;
        unknown)
            ;;
        *)
            printf 'world-lifecycle unsupported restore active value %s for %s\n' "${active}" "${unit}" >&2
            return 1
            ;;
    esac
}

linux_start_service_unit() {
    local unit="$1"
    if [[ "${unit}" == "substrate-lifecycle-publisher-v1.service" ]]; then
        invoke_linux_lifecycle_executor service-state --service-unit "${unit}" --action start >/dev/null
        return 0
    fi
    sudo_cmd systemctl start "${unit}"
}

linux_restore_account_state() {
    local group="$1"
    local user="$2"
    local group_exists_before="$3"
    local user_member_before="$4"

    if [[ "${group_exists_before}" == "0" ]]; then
        if [[ -n "${user}" && "${user}" != "root" ]] \
            && id "${user}" >/dev/null 2>&1 \
            && user_in_group "${user}" "${group}"; then
            sudo_cmd gpasswd -d "${user}" "${group}" || true
        fi
        if getent group "${group}" >/dev/null 2>&1; then
            sudo_cmd groupdel "${group}" || true
        fi
        return 0
    fi

    if [[ -n "${user}" && "${user}" != "root" ]] && id "${user}" >/dev/null 2>&1; then
        if [[ "${user_member_before}" == "1" ]]; then
            if ! user_in_group "${user}" "${group}"; then
                sudo_cmd usermod -aG "${group}" "${user}" || true
            fi
        else
            if user_in_group "${user}" "${group}"; then
                sudo_cmd gpasswd -d "${user}" "${group}" || true
            fi
        fi
    fi
}

linux_restore_acl_state() {
    local label="$1"
    local target="$2"
    local recursive="$3"
    local backup="$4"

    [[ -n "${recursive}" ]] || true
    if [[ -z "${backup}" ]]; then
        return 0
    fi
    if [[ ! -f "${backup}" ]]; then
        printf 'world-lifecycle missing ACL snapshot %s for %s (%s)\n' "${backup}" "${target}" "${label}" >&2
        return 1
    fi
    if ! command -v setfacl >/dev/null 2>&1; then
        printf 'world-lifecycle cannot restore ACL snapshot for %s without setfacl\n' "${target}" >&2
        return 1
    fi
    if ! sudo_cmd test -e "${target}" >/dev/null 2>&1; then
        printf 'world-lifecycle expected ACL restore target missing: %s (%s)\n' "${target}" "${label}" >&2
        return 1
    fi

    sudo_cmd setfacl "--restore=${backup}"
}

record_linux_managed_state() {
    linux_require_world_context || return 2
    if [[ ${DRY_RUN} -eq 1 ]]; then
        return 0
    fi
    LINUX_MANAGED_STATE_SNAPSHOT_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-world-lifecycle.XXXXXX")"
    : >"$(linux_snapshot_manifest_path)"
    : >"$(linux_snapshot_service_path)"
    : >"$(linux_snapshot_account_state_path)"
    : >"$(linux_snapshot_acl_state_path)"

    linux_snapshot_path "world-service-binary" "/usr/local/bin/substrate-world-service"
    linux_snapshot_path "gateway-binary" "/usr/local/bin/substrate-gateway"
    linux_snapshot_path "acl-helper" "${ACL_HELPER_INSTALL_PATH}"
    linux_snapshot_path "world-service-unit" "${SERVICE_PATH}"
    linux_snapshot_path "world-socket-unit" "${SOCKET_PATH}"
    linux_snapshot_path "world-socket-dropin" "${ACL_DROPIN_PATH}"
    linux_snapshot_path "lifecycle-executor" "/usr/libexec/substrate/substrate-lifecycle-linux"
    linux_snapshot_path "lifecycle-service-unit" "/etc/systemd/system/substrate-lifecycle-publisher-v1.service"
    linux_snapshot_path "lifecycle-socket-unit" "/etc/systemd/system/substrate-lifecycle-publisher-v1.socket"
    linux_snapshot_path "run-substrate-dir" "/run/substrate"
    linux_snapshot_path "world-socket-endpoint" "${SOCKET_FS_PATH}"
    linux_snapshot_path "lifecycle-socket-endpoint" "/run/substrate-lifecycle-publisher-v1.sock"
    linux_snapshot_path "state-root" "${SUBSTRATE_STATE_PATH}"
    linux_snapshot_path "world-deps-root" "${WORLD_DEPS_ROOT_PATH}"
    linux_snapshot_path "world-deps-bin" "${WORLD_DEPS_BIN_PATH}"
    linux_snapshot_path "lifecycle-container" "${SUBSTRATE_STATE_PATH}/.substrate-lifecycle-v1"

    linux_snapshot_service_state "substrate-world-service.service"
    linux_snapshot_service_state "substrate-world-service.socket"
    linux_snapshot_service_state "substrate-lifecycle-publisher-v1.service"
    linux_snapshot_service_state "substrate-lifecycle-publisher-v1.socket"
    linux_snapshot_account_state
    linux_snapshot_acl_state "world-socket-acl" "${SOCKET_FS_PATH}" "0"
    linux_snapshot_acl_state "state-root-acl" "${SUBSTRATE_STATE_PATH}" "0"
    linux_snapshot_acl_state "world-deps-acl" "${WORLD_DEPS_ROOT_PATH}" "1"
}

invoke_linux_lifecycle_executor() {
    sudo_cmd "${LIFECYCLE_EXECUTOR_INSTALL_PATH:-/usr/libexec/substrate/substrate-lifecycle-linux}" "$@"
}

install_linux_managed_state() {
    echo "==> Ensuring ${SUBSTRATE_GROUP} group and membership"
    ensure_substrate_group_exists
    ensure_user_in_group "${INVOKING_USER}"

    echo "==> Installing world-service to /usr/local/bin (sudo will prompt if needed)"
    sudo_cmd install -Dm0755 "${WORLD_AGENT_BIN_PATH}" /usr/local/bin/substrate-world-service
    echo "==> Installing substrate-gateway to /usr/local/bin (no dedicated service)"
    sudo_cmd install -Dm0755 "${GATEWAY_BIN_PATH}" /usr/local/bin/substrate-gateway
    echo "==> Installing ACL bridge helper to ${ACL_HELPER_INSTALL_PATH}"
    sudo_cmd install -Dm0755 "${ACL_HELPER_SOURCE_PATH}" "${ACL_HELPER_INSTALL_PATH}"
    echo "==> Installing Linux lifecycle executor to /usr/libexec/substrate/substrate-lifecycle-linux"
    sudo_cmd install -Dm0755 "${LIFECYCLE_EXECUTOR_BIN_PATH}" /usr/libexec/substrate/substrate-lifecycle-linux

    echo "==> Ensuring runtime directories exist"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" /run/substrate
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${SUBSTRATE_STATE_PATH}"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${WORLD_DEPS_ROOT_PATH}"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${WORLD_DEPS_BIN_PATH}"

    echo "==> Writing world systemd units to ${SERVICE_PATH} and ${SOCKET_PATH}"
    install_unit "${SERVICE_PATH}" "${SERVICE_UNIT_CONTENT}"
    install_unit "${SOCKET_PATH}" "${SOCKET_UNIT_CONTENT}"
    install_unit "${ACL_DROPIN_PATH}" "${SOCKET_DROPIN_CONTENT}"

    echo "==> Installing lifecycle publisher units"
    sudo_cmd install -Dm0644 \
        "${SCRIPT_DIR}/substrate-lifecycle-publisher-v1.service" \
        /etc/systemd/system/substrate-lifecycle-publisher-v1.service
    sudo_cmd install -Dm0644 \
        "${SCRIPT_DIR}/substrate-lifecycle-publisher-v1.socket" \
        /etc/systemd/system/substrate-lifecycle-publisher-v1.socket

    echo "==> Reloading systemd and enabling socket activation"
    LEGACY_WORLD_UNIT_PREFIX="substrate-world"
    LEGACY_SERVICE="${LEGACY_WORLD_UNIT_PREFIX}-agent.service"
    LEGACY_SOCKET="${LEGACY_WORLD_UNIT_PREFIX}-agent.socket"
    if sudo_cmd systemctl cat "${LEGACY_SERVICE}" >/dev/null 2>&1; then
        sudo_cmd systemctl stop "${LEGACY_SERVICE}" || true
        sudo_cmd systemctl disable "${LEGACY_SERVICE}" || true
    fi
    if sudo_cmd systemctl cat "${LEGACY_SOCKET}" >/dev/null 2>&1; then
        sudo_cmd systemctl stop "${LEGACY_SOCKET}" || true
        sudo_cmd systemctl disable "${LEGACY_SOCKET}" || true
    fi
    sudo_cmd rm -f "/etc/systemd/system/${LEGACY_SERVICE}" "/etc/systemd/system/${LEGACY_SOCKET}" || true
    sudo_cmd systemctl daemon-reload
    sudo_cmd systemctl enable substrate-world-service.service
    sudo_cmd systemctl enable substrate-world-service.socket
    sudo_cmd systemctl enable substrate-lifecycle-publisher-v1.socket

    echo "==> Restarting socket/service to enforce ${SOCKET_FS_PATH} ownership"
    sudo_cmd systemctl stop substrate-world-service.service
    sudo_cmd systemctl stop substrate-world-service.socket
    sudo_cmd systemctl stop substrate-lifecycle-publisher-v1.service || true
    sudo_cmd systemctl stop substrate-lifecycle-publisher-v1.socket || true
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" /run/substrate
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${SUBSTRATE_STATE_PATH}"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${WORLD_DEPS_ROOT_PATH}"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${WORLD_DEPS_BIN_PATH}"
    sudo_cmd rm -f "${SOCKET_FS_PATH}"
    sudo_cmd rm -f /run/substrate-lifecycle-publisher-v1.sock
    sudo_cmd systemctl start substrate-world-service.socket
    sudo_cmd systemctl start substrate-lifecycle-publisher-v1.socket
    sudo_cmd "${ACL_HELPER_INSTALL_PATH}" --socket "${SOCKET_FS_PATH}" "${SUBSTRATE_GROUP}" || true
    sudo_cmd "${ACL_HELPER_INSTALL_PATH}" --directory-traverse "${SUBSTRATE_STATE_PATH}" "${SUBSTRATE_GROUP}" || true
    sudo_cmd "${ACL_HELPER_INSTALL_PATH}" --tree-readonly "${WORLD_DEPS_ROOT_PATH}" "${SUBSTRATE_GROUP}" || true
    sudo_cmd systemctl start substrate-world-service.service

    echo "==> ${SOCKET_FS_PATH} listing (should be root:${SUBSTRATE_GROUP} 0660)"
    sudo_cmd ls -l "${SOCKET_FS_PATH}"
    echo "==> /run/substrate-lifecycle-publisher-v1.sock listing (should be root:root 0600)"
    sudo_cmd ls -l /run/substrate-lifecycle-publisher-v1.sock || true
    if command -v getfacl >/dev/null 2>&1; then
        echo "==> ${SOCKET_FS_PATH} ACL"
        sudo_cmd getfacl -cp "${SOCKET_FS_PATH}" || true
    else
        print_acl_bridge_warning
    fi
    verify_socket_acl_bridge "${INVOKING_USER}"
    echo "==> ${WORLD_DEPS_ROOT_PATH} listing (should be root:${SUBSTRATE_GROUP} 0750 with named-user ACL bridge when needed)"
    WORLD_DEPS_PROBE_PATH="$(world_deps_probe_path)"
    if sudo_cmd test -e "${WORLD_DEPS_PROBE_PATH}" >/dev/null 2>&1; then
        sudo_cmd ls -ld "${SUBSTRATE_STATE_PATH}" "${WORLD_DEPS_ROOT_PATH}" "${WORLD_DEPS_BIN_PATH}" "${WORLD_DEPS_PROBE_PATH}"
    else
        sudo_cmd ls -ld "${SUBSTRATE_STATE_PATH}" "${WORLD_DEPS_ROOT_PATH}" "${WORLD_DEPS_BIN_PATH}"
        echo "==> World-deps runtime probe path not present yet: ${WORLD_DEPS_PROBE_PATH}"
    fi
    if command -v getfacl >/dev/null 2>&1; then
        echo "==> ${WORLD_DEPS_ROOT_PATH} ACL"
        if sudo_cmd test -e "${WORLD_DEPS_PROBE_PATH}" >/dev/null 2>&1; then
            sudo_cmd getfacl -cp "${SUBSTRATE_STATE_PATH}" "${WORLD_DEPS_ROOT_PATH}" "${WORLD_DEPS_BIN_PATH}" "${WORLD_DEPS_PROBE_PATH}" || true
        else
            sudo_cmd getfacl -cp "${SUBSTRATE_STATE_PATH}" "${WORLD_DEPS_ROOT_PATH}" "${WORLD_DEPS_BIN_PATH}" || true
        fi
    fi
    verify_world_deps_acl_bridge "${INVOKING_USER}"
    echo "==> Installed gateway binary"
    sudo_cmd ls -l /usr/local/bin/substrate-gateway
    echo "==> Installed lifecycle executor"
    sudo_cmd ls -l /usr/libexec/substrate/substrate-lifecycle-linux

    echo "==> substrate-world-service.socket status (last 10 log lines)"
    sudo_cmd systemctl status substrate-world-service.socket --no-pager --lines=10 || true
    echo "==> substrate-world-service.service status (last 10 log lines)"
    sudo_cmd systemctl status substrate-world-service.service --no-pager --lines=10 || true
    echo "==> substrate-lifecycle-publisher-v1.socket status (last 10 log lines)"
    sudo_cmd systemctl status substrate-lifecycle-publisher-v1.socket --no-pager --lines=10 || true
}

restore_linux_managed_state() {
    if [[ -z "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}" || ! -d "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}" ]]; then
        return 0
    fi

    if [[ -f "$(linux_snapshot_manifest_path)" ]]; then
        tac "$(linux_snapshot_manifest_path)" | while IFS=$'\t' read -r label target type mode backup owner group; do
            [[ -n "${label}" ]] || continue
            linux_restore_path "${label}" "${target}" "${type}" "${mode}" "${backup}" "${owner}" "${group}"
        done
    fi

    sudo_cmd systemctl daemon-reload || true

    if [[ -f "$(linux_snapshot_service_path)" ]]; then
        while IFS=$'\t' read -r unit enabled active; do
            [[ -n "${unit}" ]] || continue
            linux_restore_service_state "${unit}" "${enabled}" "${active}"
        done <"$(linux_snapshot_service_path)"
    fi

    if [[ -f "$(linux_snapshot_acl_state_path)" ]]; then
        while IFS=$'\t' read -r label target recursive backup; do
            [[ -n "${label}" ]] || continue
            linux_restore_acl_state "${label}" "${target}" "${recursive}" "${backup}"
        done <"$(linux_snapshot_acl_state_path)"
    fi

    if [[ -f "$(linux_snapshot_account_state_path)" ]]; then
        while IFS=$'\t' read -r group user group_exists_before user_member_before; do
            [[ -n "${group}" ]] || continue
            linux_restore_account_state "${group}" "${user}" "${group_exists_before}" "${user_member_before}"
        done <"$(linux_snapshot_account_state_path)"
    fi

    rm -rf -- "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}"
    LINUX_MANAGED_STATE_SNAPSHOT_ROOT=""
}

main() {
    linux_require_world_context || return 2
    if [[ ${DRY_RUN} -eq 0 ]]; then
        trap 'restore_linux_managed_state' ERR
        record_linux_managed_state
    fi
    install_linux_managed_state

    if [[ ${DRY_RUN} -eq 1 ]]; then
        substrate_cli="$(resolve_substrate_cli 2>/dev/null || true)"
        if [[ -z "${substrate_cli}" ]]; then
            substrate_cli="${SUBSTRATE_CLI_BIN_PATH}"
        fi
    else
        substrate_cli="$(resolve_substrate_cli)"
        maybe_run_gateway_lifecycle_proof "${substrate_cli}"
    fi
    print_linger_guidance "${INVOKING_USER}"

    echo "==> Provisioning complete"
    echo "    Verify socket with: sudo ls -l ${SOCKET_FS_PATH}"
    echo "    Verify socket ACL: sudo getfacl -cp ${SOCKET_FS_PATH}"
    echo "    Verify world-deps ACL: sudo getfacl -cp ${SUBSTRATE_STATE_PATH} ${WORLD_DEPS_ROOT_PATH} ${WORLD_DEPS_BIN_PATH} $(world_deps_probe_path)"
    echo "    Probe capabilities: sudo curl --unix-socket ${SOCKET_FS_PATH} http://localhost/v1/capabilities"
    echo "    Verify gateway lifecycle: $(basename "${substrate_cli:-substrate}") world gateway status --json"
    echo "    Verify lifecycle socket: sudo ls -l /run/substrate-lifecycle-publisher-v1.sock"
    echo "    Doctor socket block: substrate host doctor --json | jq '.host.world_socket'"
    echo "    Doctor member-selection block: substrate agent doctor --json | jq '.checks[] | select(.check==\"member_selection\")'"
    echo "    Shim summary: substrate --shim-status | grep 'World socket'"

    trap - ERR
    rm -rf -- "${LINUX_MANAGED_STATE_SNAPSHOT_ROOT}"
    LINUX_MANAGED_STATE_SNAPSHOT_ROOT=""
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    printf 'scripts/linux/world-lifecycle.sh is a sourced helper for world-provision.sh\n' >&2
    exit 2
fi
