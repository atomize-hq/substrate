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
            if sudo_cmd test -d "${target}" >/dev/null 2>&1 \
                && ! sudo_cmd test -L "${target}" >/dev/null 2>&1; then
                sudo_cmd rm -rf -- "${target}"
            else
                sudo_cmd rm -f -- "${target}"
            fi
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
        linux_restart_service_unit "${unit}"
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
            linux_restart_service_unit "${unit}"
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

linux_stop_service_unit() {
    local unit="$1"
    if [[ "${unit}" == "substrate-lifecycle-publisher-v1.service" ]]; then
        invoke_linux_lifecycle_executor service-state --service-unit "${unit}" --action stop >/dev/null
        return
    fi
    sudo_cmd systemctl stop "${unit}"
}

linux_restart_service_unit() {
    local unit="$1"
    if [[ "${unit}" == "substrate-lifecycle-publisher-v1.service" ]]; then
        linux_stop_service_unit "${unit}" || return $?
        invoke_linux_lifecycle_executor service-state --service-unit "${unit}" --action start >/dev/null
        return
    fi
    sudo_cmd systemctl restart "${unit}"
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

provision_e3_system_config_mount_target_v1() {
    local target_prefix="${FAKE_ROOT:-}"
    local etc_path="${target_prefix}/etc"
    if [[ -n "${target_prefix}" ]]; then
        sudo_cmd install -d -m0755 "${etc_path}"
    fi
    sudo_cmd python3 - "${etc_path}" "${target_prefix:+test}" <<'PY'
import os
import stat
import sys

etc_path, test_mode = sys.argv[1:]
etc_fd = os.open(etc_path, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
try:
    etc_stat = os.fstat(etc_fd)
    if not stat.S_ISDIR(etc_stat.st_mode) or stat.S_IMODE(etc_stat.st_mode) & 0o022:
        raise RuntimeError("/etc is not a trusted directory")
    try:
        os.mkdir("codex", 0o755, dir_fd=etc_fd)
        created = True
    except FileExistsError:
        created = False
    codex_fd = os.open("codex", os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW, dir_fd=etc_fd)
    try:
        codex_stat = os.fstat(codex_fd)
        if (
            not stat.S_ISDIR(codex_stat.st_mode)
            or stat.S_IMODE(codex_stat.st_mode) != 0o755
            or (not test_mode and (codex_stat.st_uid != 0 or codex_stat.st_gid != 0))
            or os.listdir(codex_fd)
        ):
            raise RuntimeError("/etc/codex is not the exact empty E3 mount target")
        if created:
            os.fsync(codex_fd)
            os.fsync(etc_fd)
    finally:
        os.close(codex_fd)
finally:
    os.close(etc_fd)
PY
}

publish_installed_home_bootstrap_v1() {
    local authority_root="${FAKE_ROOT:-}/var/lib/substrate/install-bootstrap-authority-v1"
    sudo_cmd python3 - \
        "${authority_root}" \
        "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        "${INSTALL_BOOTSTRAP_COMMITMENT}" \
        "${INSTALL_BOOTSTRAP_ACCOUNT}" \
        "${INSTALL_BOOTSTRAP_UID}" \
        "${INSTALL_BOOTSTRAP_PRIMARY_GID}" \
        "${INSTALL_PREFIX}" \
        "${FAKE_ROOT:+test}" <<'PY'
import base64
import datetime
import fcntl
import grp
import hashlib
import json
import os
import pwd
import re
import secrets
import stat
import sys
import time
import uuid

root, carrier, commitment, account, uid_raw, gid_raw, accepted_home, test_mode = sys.argv[1:]
uid = int(uid_raw)
gid = int(gid_raw)

def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()

def domain_hash(domain, member, value, omitted):
    copied = dict(value)
    copied.pop(omitted)
    return hashlib.sha256(canonical({"domain": domain, member: copied})).hexdigest()

def timestamp():
    now = datetime.datetime.now(datetime.timezone.utc)
    return now.strftime("%Y-%m-%dT%H:%M:%S.%fZ")

def uuid7():
    milliseconds = time.time_ns() // 1_000_000
    random_bits = secrets.randbits(74)
    value = ((milliseconds & ((1 << 48) - 1)) << 80) | (7 << 76)
    value |= ((random_bits >> 62) & 0xFFF) << 64
    value |= 2 << 62
    value |= random_bits & ((1 << 62) - 1)
    return str(uuid.UUID(int=value))

def valid_digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None

def valid_u32(value):
    return type(value) is int and 0 <= value <= 0xffffffff

def valid_u64(value):
    return type(value) is int and 0 <= value <= 0xffffffffffffffff

def valid_timestamp(value):
    if not isinstance(value, str) or re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{6}Z", value) is None:
        return False
    try:
        datetime.datetime.strptime(value, "%Y-%m-%dT%H:%M:%S.%fZ")
    except ValueError:
        return False
    return True

def validate_bootstrap_record(value, current_binding=False):
    accepted = value.get("accepted_home", {}) if isinstance(value, dict) else {}
    identity = accepted.get("physical_identity", {}).get("Linux", {}) if isinstance(accepted, dict) else {}
    complete = isinstance(value, dict) and set(value) == {"schema_version", "install_bootstrap_carrier", "host_context_commitment", "intended_account", "intended_uid", "intended_gid", "accepted_home", "installed_at", "record_hash"} and valid_u32(value.get("schema_version")) and value["schema_version"] == 1 and isinstance(value.get("install_bootstrap_carrier"), str) and bool(value["install_bootstrap_carrier"]) and valid_digest(value.get("host_context_commitment")) and isinstance(value.get("intended_account"), str) and bool(value["intended_account"]) and valid_u64(value.get("intended_uid")) and valid_u64(value.get("intended_gid")) and set(accepted) == {"physical_path", "physical_identity"} and isinstance(accepted.get("physical_path"), str) and accepted["physical_path"].startswith("/") and set(accepted.get("physical_identity", {})) == {"Linux"} and set(identity) == {"device_id", "inode"} and valid_u64(identity.get("device_id")) and identity["device_id"] > 0 and valid_u64(identity.get("inode")) and identity["inode"] > 0 and valid_timestamp(value.get("installed_at")) and value.get("record_hash") == domain_hash("substrate.e3.installed-accepted-home-bootstrap.v1", "record", value, "record_hash")
    if not complete or not current_binding:
        return complete
    return value["install_bootstrap_carrier"] == carrier and value["host_context_commitment"] == commitment and value["intended_account"] == account and value["intended_uid"] == uid and value["intended_gid"] == gid and accepted["physical_path"] == accepted_home and identity["device_id"] == home_stat.st_dev and identity["inode"] == home_stat.st_ino

def validate_bootstrap_head(value):
    return isinstance(value, dict) and set(value) == {"schema_version", "head_record_hash", "predecessor_head_hash", "updated_at", "head_hash"} and valid_u32(value.get("schema_version")) and value["schema_version"] == 1 and valid_digest(value.get("head_record_hash")) and (value.get("predecessor_head_hash") is None or valid_digest(value["predecessor_head_hash"])) and valid_timestamp(value.get("updated_at")) and value.get("head_hash") == domain_hash("substrate.e3.installed-accepted-home-bootstrap-head.v1", "head", value, "head_hash")

def validate_bootstrap_head_record(value, current_binding=False):
    if not validate_bootstrap_head(value):
        return False
    path = os.path.join(root, "records", value["head_record_hash"] + ".json")
    if not os.path.exists(path):
        return False
    with open(path, "rb") as source:
        body = source.read()
    record = json.loads(body)
    return canonical(record) == body and validate_bootstrap_record(record, current_binding) and record["record_hash"] == value["head_record_hash"]

def exact_write(path, body, immutable):
    if os.path.exists(path):
        with open(path, "rb") as source:
            existing = source.read()
        if existing == body:
            return
        if immutable:
            raise RuntimeError("installed-home bootstrap publication conflict")
    suffix = "record" if immutable else "head"
    temp = os.path.join(os.path.dirname(path), f".e3-bootstrap-tmp.{uuid7()}.{suffix}")
    descriptor = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
    try:
        os.write(descriptor, body)
        os.fchmod(descriptor, 0o640)
        if not test_mode:
            os.fchown(descriptor, 0, substrate_gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    if immutable:
        try:
            os.link(temp, path, follow_symlinks=False)
        except FileExistsError:
            with open(path, "rb") as source:
                if source.read() != body:
                    raise
        os.unlink(temp)
    else:
        os.replace(temp, path)
    os.chmod(path, 0o640, follow_symlinks=False)
    if not test_mode:
        os.chown(path, 0, substrate_gid, follow_symlinks=False)
    parent_fd = os.open(os.path.dirname(path), os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        os.fsync(parent_fd)
    finally:
        os.close(parent_fd)
    with open(path, "rb") as source:
        if source.read() != body:
            raise RuntimeError("installed-home bootstrap readback mismatch")

def stage_publication(path, body, kind):
    temporary = os.path.join(os.path.dirname(path), f".e3-bootstrap-tmp.{uuid7()}.{kind}")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
    try:
        os.write(descriptor, body)
        os.fchmod(descriptor, 0o640)
        if not test_mode:
            os.fchown(descriptor, 0, substrate_gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    return temporary

def commit_staged_publication(temporary, path, body, immutable):
    if immutable:
        try:
            os.link(temporary, path, follow_symlinks=False)
        except FileExistsError:
            with open(path, "rb") as source:
                if source.read() != body:
                    raise RuntimeError("installed-home bootstrap publication conflict")
        os.unlink(temporary)
    else:
        os.replace(temporary, path)
    os.chmod(path, 0o640, follow_symlinks=False)
    if not test_mode:
        os.chown(path, 0, substrate_gid, follow_symlinks=False)
    parent_fd = os.open(os.path.dirname(path), os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        os.fsync(parent_fd)
    finally:
        os.close(parent_fd)
    with open(path, "rb") as source:
        if source.read() != body:
            raise RuntimeError("installed-home staged publication readback mismatch")

def recover_publication():
    locations = [(root, "head"), (os.path.join(root, "records"), "record")]
    pattern = re.compile(r"^\.e3-bootstrap-tmp\.([0-9a-f-]{36})\.(record|head)$")
    contenders = {}
    for directory, allowed_kind in locations:
        for name in os.listdir(directory):
            if not name.startswith(".e3-bootstrap-tmp."):
                continue
            match = pattern.fullmatch(name)
            if match is None or match.group(2) != allowed_kind:
                raise RuntimeError("unrecognized installed-home bootstrap temporary")
            parsed_uuid = uuid.UUID(match.group(1))
            if parsed_uuid.version != 7 or str(parsed_uuid) != match.group(1):
                raise RuntimeError("invalid installed-home bootstrap temporary identity")
            temporary = os.path.join(directory, name)
            info = os.stat(temporary, follow_symlinks=False)
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or stat.S_IMODE(info.st_mode) != 0o640:
                raise RuntimeError("unsafe installed-home bootstrap temporary")
            with open(temporary, "rb") as source:
                body = source.read()
            value = json.loads(body)
            if canonical(value) != body:
                raise RuntimeError("noncanonical installed-home bootstrap temporary")
            kind = match.group(2)
            if kind == "record":
                valid = validate_bootstrap_record(value, current_binding=True)
                final = os.path.join(root, "records", value.get("record_hash", "") + ".json")
            else:
                valid = validate_bootstrap_head(value)
                final = os.path.join(root, "active.json")
            if not valid:
                raise RuntimeError("invalid installed-home bootstrap temporary")
            if final in contenders:
                raise RuntimeError("ambiguous installed-home bootstrap temporaries")
            contenders[final] = (temporary, body, value, kind)
    for final, (temporary, body, value, kind) in sorted(contenders.items(), key=lambda item: 0 if item[1][3] == "record" else 1):
        if os.path.exists(final):
            with open(final, "rb") as source:
                current_body = source.read()
            current_value = json.loads(current_body)
            current_valid = canonical(current_value) == current_body and (validate_bootstrap_record(current_value) if kind == "record" else validate_bootstrap_head_record(current_value))
            if not current_valid:
                raise RuntimeError("invalid installed-home bootstrap final")
            if current_body == body:
                os.unlink(temporary)
                parent_fd = os.open(os.path.dirname(temporary), os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
                try:
                    os.fsync(parent_fd)
                finally:
                    os.close(parent_fd)
                continue
            if kind != "head":
                raise RuntimeError("installed-home bootstrap temporary conflicts with final")
        if kind == "head":
            record_path = os.path.join(root, "records", value["head_record_hash"] + ".json")
            if not os.path.exists(record_path):
                raise RuntimeError("installed-home bootstrap head lacks its immutable record")
            if not validate_bootstrap_head_record(value, current_binding=True):
                raise RuntimeError("installed-home bootstrap head names an invalid immutable record")
            predecessor = value["predecessor_head_hash"]
            if os.path.exists(final):
                with open(final, "rb") as source:
                    current = json.loads(source.read())
                if predecessor != current.get("head_hash"):
                    raise RuntimeError("stale installed-home bootstrap head temporary")
            elif predecessor is not None:
                raise RuntimeError("invalid initial installed-home bootstrap head temporary")
            os.replace(temporary, final)
        else:
            os.link(temporary, final, follow_symlinks=False)
            os.unlink(temporary)
        parent_fd = os.open(os.path.dirname(final), os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
        try:
            os.fsync(parent_fd)
        finally:
            os.close(parent_fd)
        with open(final, "rb") as source:
            published_body = source.read()
            if published_body != body:
                raise RuntimeError("recovered installed-home bootstrap readback mismatch")
        published = json.loads(published_body)
        if (kind == "record" and not validate_bootstrap_record(published, current_binding=True)) or (kind == "head" and not validate_bootstrap_head_record(published, current_binding=True)):
            raise RuntimeError("recovered installed-home bootstrap is incomplete")

raw_encoded = carrier.encode("ascii")
if not raw_encoded or not re.fullmatch(rb"[A-Za-z0-9_-]+", raw_encoded):
    raise RuntimeError("invalid install bootstrap carrier")
raw = base64.urlsafe_b64decode(raw_encoded + b"=" * ((-len(raw_encoded)) % 4))
if base64.urlsafe_b64encode(raw).rstrip(b"=") != raw_encoded or not raw.endswith(b"\n"):
    raise RuntimeError("noncanonical install bootstrap carrier")
lines = raw[:-1].split(b"\n")
keys = [b"domain", b"version", b"selected_host_prefix", b"host_substrate_home", b"host_substrate_root", b"principal_kind", b"principal_account", b"principal_uid", b"host_context_commitment"]
if len(lines) != len(keys):
    raise RuntimeError("invalid install bootstrap carrier field count")
values = {}
for expected, line in zip(keys, lines):
    key, separator, value = line.partition(b"=")
    if key != expected or separator != b"=" or b"=" in value:
        raise RuntimeError("invalid install bootstrap carrier order")
    values[key] = value
if values[b"domain"] != b"substrate.install_bootstrap_context" or values[b"version"] != b"1" or values[b"principal_kind"] != b"unix":
    raise RuntimeError("invalid install bootstrap carrier identity")
prefix = base64.urlsafe_b64decode(values[b"selected_host_prefix"] + b"=" * ((-len(values[b"selected_host_prefix"])) % 4)).decode()
decoded_account = base64.urlsafe_b64decode(values[b"principal_account"] + b"=" * ((-len(values[b"principal_account"])) % 4)).decode()
if prefix != accepted_home or decoded_account != account or int(values[b"principal_uid"]) != uid or values[b"host_context_commitment"].decode() != commitment:
    raise RuntimeError("install bootstrap carrier binding mismatch")
commitment_frame = raw[: raw.rfind(b"host_context_commitment=")]
if hashlib.sha256(commitment_frame).hexdigest() != commitment:
    raise RuntimeError("install bootstrap carrier commitment mismatch")
if not test_mode:
    entry = pwd.getpwnam(account)
    if entry.pw_uid != uid or entry.pw_gid != gid or pwd.getpwuid(uid).pw_name != account:
        raise RuntimeError("install bootstrap account identity changed")

components = [component for component in accepted_home.split("/") if component]
home_fd = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
try:
    for component in components:
        next_fd = os.open(component, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW, dir_fd=home_fd)
        os.close(home_fd)
        home_fd = next_fd
    home_stat = os.fstat(home_fd)
    if not stat.S_ISDIR(home_stat.st_mode) or (not test_mode and (home_stat.st_uid != uid or stat.S_IMODE(home_stat.st_mode) != 0o700)):
        raise RuntimeError("accepted home is not the validated private root")
finally:
    os.close(home_fd)

try:
    substrate_gid = grp.getgrnam("substrate").gr_gid
except KeyError:
    if not test_mode:
        raise RuntimeError("missing substrate group for E3 bootstrap authority")
    substrate_gid = os.getgid()
os.makedirs(os.path.join(root, "records"), mode=0o750, exist_ok=True)
os.chmod(root, 0o750)
os.chmod(os.path.join(root, "records"), 0o750)
if not test_mode:
    os.chown(root, 0, substrate_gid)
    os.chown(os.path.join(root, "records"), 0, substrate_gid)
lock_path = os.path.join(root, "lock")
lock_fd = os.open(lock_path, os.O_RDWR | os.O_CREAT | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600)
try:
    os.chmod(lock_path, 0o600, follow_symlinks=False)
    fcntl.flock(lock_fd, fcntl.LOCK_EX)
    recover_publication()
    active_path = os.path.join(root, "active.json")
    if os.path.exists(active_path):
        with open(active_path, "rb") as source:
            active_bytes = source.read()
        active = json.loads(active_bytes)
        if canonical(active) != active_bytes or not validate_bootstrap_head_record(active):
            raise RuntimeError("installed-home bootstrap head is not canonical")
        active_record_path = os.path.join(root, "records", active["head_record_hash"] + ".json")
        with open(active_record_path, "rb") as source:
            active_record_bytes = source.read()
        active_record = json.loads(active_record_bytes)
        if canonical(active_record) != active_record_bytes or not validate_bootstrap_record(active_record):
            raise RuntimeError("installed-home bootstrap record is not canonical")
        expected_binding = {
            "schema_version": 1,
            "install_bootstrap_carrier": carrier,
            "host_context_commitment": commitment,
            "intended_account": account,
            "intended_uid": uid,
            "intended_gid": gid,
            "accepted_home": {"physical_path": accepted_home, "physical_identity": {"Linux": {"device_id": home_stat.st_dev, "inode": home_stat.st_ino}}},
        }
        if all(active_record.get(key) == value for key, value in expected_binding.items()):
            raise SystemExit(0)
    record = {
        "schema_version": 1,
        "install_bootstrap_carrier": carrier,
        "host_context_commitment": commitment,
        "intended_account": account,
        "intended_uid": uid,
        "intended_gid": gid,
        "accepted_home": {"physical_path": accepted_home, "physical_identity": {"Linux": {"device_id": home_stat.st_dev, "inode": home_stat.st_ino}}},
        "installed_at": timestamp(),
        "record_hash": "",
    }
    record["record_hash"] = domain_hash("substrate.e3.installed-accepted-home-bootstrap.v1", "record", record, "record_hash")
    record_path = os.path.join(root, "records", record["record_hash"] + ".json")
    if os.path.exists(record_path):
        with open(record_path, "rb") as source:
            existing_body = source.read()
        existing = json.loads(existing_body)
        if canonical(existing) != existing_body or not validate_bootstrap_record(existing, current_binding=True):
            raise RuntimeError("installed-home bootstrap immutable record is invalid")
        record = existing
    record_body = canonical(record)
    prior = None
    prior_bytes = None
    if os.path.exists(active_path):
        with open(active_path, "rb") as source:
            prior_bytes = source.read()
        prior = json.loads(prior_bytes)
        if canonical(prior) != prior_bytes or not validate_bootstrap_head_record(prior):
            raise RuntimeError("installed-home bootstrap head is not canonical")
        if prior["head_record_hash"] == record["record_hash"]:
            raise SystemExit(0)
    head = {
        "schema_version": 1,
        "head_record_hash": record["record_hash"],
        "predecessor_head_hash": None if prior is None else prior["head_hash"],
        "updated_at": timestamp(),
        "head_hash": "",
    }
    head["head_hash"] = domain_hash("substrate.e3.installed-accepted-home-bootstrap-head.v1", "head", head, "head_hash")
    head_body = canonical(head)
    record_temporary = stage_publication(record_path, record_body, "record")
    head_temporary = stage_publication(active_path, head_body, "head")
    if prior_bytes is not None:
        with open(active_path, "rb") as source:
            if source.read() != prior_bytes:
                raise RuntimeError("installed-home bootstrap head CAS conflict")
    commit_staged_publication(record_temporary, record_path, record_body, True)
    commit_staged_publication(head_temporary, active_path, head_body, False)
finally:
    os.close(lock_fd)
PY
}

publish_substrate_artifact_source_v1() {
    local gateway_path="${FAKE_ROOT:-}/usr/local/lib/substrate/e3/substrate-gateway"
    local wrapper_path="${FAKE_ROOT:-}/usr/local/lib/substrate/e3/substrate-world-entry"
    if ! sudo_cmd test -f "${gateway_path}" >/dev/null 2>&1 \
        || ! sudo_cmd test -f "${wrapper_path}" >/dev/null 2>&1; then
        echo "E3 Substrate artifact publication is unavailable until both static E3 artifacts are installed" >&2
        return 1
    fi
    sudo_cmd python3 - \
        "${FAKE_ROOT:-}/var/lib/substrate/runtime-artifacts-v1" \
        "${gateway_path}" \
        "${wrapper_path}" \
        "${SUBSTRATE_SOURCE_COMMIT}" \
        "${SUBSTRATE_SOURCE_TREE}" \
        "${SUBSTRATE_CARGO_LOCK_SHA256}" \
        "${SUBSTRATE_RUSTC_VERSION}" \
        "${SUBSTRATE_SOURCE_TARGET}" \
        "${SUBSTRATE_SOURCE_PROFILE}" \
        "${SUBSTRATE_SOURCE_BUILD_PERFORMED}" <<'PY'
import datetime
import fcntl
import grp
import hashlib
import json
import os
import re
import secrets
import stat
import struct
import sys
import time
import uuid

root, gateway, wrapper, source_commit, source_tree, lock_hash, rustc_version, target, profile, build_performed = sys.argv[1:]
if target != "x86_64-unknown-linux-musl" or profile != "release":
    raise RuntimeError("Substrate E3 artifacts require the exact release musl build")
if build_performed != "1":
    raise RuntimeError("Substrate E3 artifact publication requires an exact current source build")

def canonical(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode()

def domain_hash(domain, member, value, omitted):
    copied = dict(value)
    copied.pop(omitted)
    return hashlib.sha256(canonical({"domain": domain, member: copied})).hexdigest()

def timestamp():
    now = datetime.datetime.now(datetime.timezone.utc)
    return now.strftime("%Y-%m-%dT%H:%M:%S.%fZ")

def uuid7():
    milliseconds = time.time_ns() // 1_000_000
    random_bits = secrets.randbits(74)
    value = ((milliseconds & ((1 << 48) - 1)) << 80) | (7 << 76)
    value |= ((random_bits >> 62) & 0xFFF) << 64
    value |= 2 << 62
    value |= random_bits & ((1 << 62) - 1)
    return str(uuid.UUID(int=value))

def valid_id(value, prefix):
    try:
        raw = value.removeprefix(prefix)
        parsed = uuid.UUID(raw)
    except (ValueError, AttributeError):
        return False
    return value == prefix + raw and parsed.version == 7 and str(parsed) == raw

def valid_digest(value):
    return isinstance(value, str) and re.fullmatch(r"[0-9a-f]{64}", value) is not None

def valid_u32(value):
    return type(value) is int and 0 <= value <= 0xffffffff

def valid_u64(value):
    return type(value) is int and 0 <= value <= 0xffffffffffffffff

def valid_timestamp(value):
    if not isinstance(value, str) or re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{6}Z", value) is None:
        return False
    try:
        datetime.datetime.strptime(value, "%Y-%m-%dT%H:%M:%S.%fZ")
    except ValueError:
        return False
    return True

def validate_store_value(value):
    info = os.stat(root, follow_symlinks=False)
    root_value = value.get("root", {}) if isinstance(value, dict) else {}
    identity = root_value.get("physical_identity", {}).get("Linux", {}) if isinstance(root_value, dict) else {}
    return isinstance(value, dict) and set(value) == {"schema_version", "source_store_id", "root", "created_at", "store_hash"} and valid_u32(value.get("schema_version")) and value["schema_version"] == 1 and valid_id(value.get("source_store_id"), "ias_") and set(root_value) == {"physical_path", "physical_identity"} and root_value.get("physical_path") == root and set(root_value.get("physical_identity", {})) == {"Linux"} and set(identity) == {"device_id", "inode"} and valid_u64(identity.get("device_id")) and identity["device_id"] == info.st_dev and valid_u64(identity.get("inode")) and identity["inode"] == info.st_ino and valid_timestamp(value.get("created_at")) and value.get("store_hash") == domain_hash("substrate.e3.installer-artifact-source-store.v1", "store", value, "store_hash")

def validate_support_value(value):
    if not isinstance(value, dict) or set(value) != {"schema_version", "support_policy_version", "elf_execution_model", "elf_interpreter", "dynamic_loader_cache", "ordered_elf_dependencies", "ordered_present_common_files", "system_config_mount_target", "manifest_hash"}:
        return False
    model = value.get("elf_execution_model")
    if model != "StaticExec":
        pie = model.get("StaticPie", {}) if isinstance(model, dict) and set(model) == {"StaticPie"} else {}
        if set(pie) != {"dynamic_segment_file_offset", "dynamic_segment_byte_length", "rela_virtual_address", "rela_byte_length", "rela_entry_byte_length", "relative_relocation_count", "ordered_dynamic_entries_sha256"} or not valid_u64(pie.get("dynamic_segment_file_offset")) or not all(valid_u64(pie.get(name)) and pie[name] > 0 for name in ("dynamic_segment_byte_length", "rela_virtual_address", "rela_byte_length", "relative_relocation_count")) or not valid_u64(pie.get("rela_entry_byte_length")) or pie["rela_entry_byte_length"] != 24 or pie["rela_byte_length"] // 24 != pie["relative_relocation_count"] or not valid_digest(pie.get("ordered_dynamic_entries_sha256")):
            return False
    expected_paths = ["/etc/hosts", "/etc/nsswitch.conf", "/etc/passwd", "/etc/group", "/etc/resolv.conf", "/etc/ssl/certs/ca-certificates.crt"]
    common = value.get("ordered_present_common_files")
    if not isinstance(common, list) or [item.get("absolute_path") if isinstance(item, dict) else None for item in common] != expected_paths:
        return False
    for item in common:
        if set(item) != {"absolute_path", "device_id", "inode", "mode", "byte_length", "sha256"} or not valid_u64(item["device_id"]) or item["device_id"] <= 0 or not valid_u64(item["inode"]) or item["inode"] <= 0 or not valid_u32(item["mode"]) or item["mode"] & 0o022 or not valid_u64(item["byte_length"]) or not valid_digest(item["sha256"]):
            return False
    mount = value.get("system_config_mount_target")
    if not isinstance(mount, dict) or set(mount) != {"absolute_path", "device_id", "inode", "mode", "owner_uid", "owner_gid", "ordered_entry_names"} or mount.get("absolute_path") != "/etc/codex" or not valid_u64(mount.get("device_id")) or mount["device_id"] <= 0 or not valid_u64(mount.get("inode")) or mount["inode"] <= 0 or not valid_u32(mount.get("mode")) or mount["mode"] != 0o755 or not valid_u64(mount.get("owner_uid")) or mount["owner_uid"] != 0 or not valid_u64(mount.get("owner_gid")) or mount["owner_gid"] != 0 or mount.get("ordered_entry_names") != []:
        return False
    return valid_u32(value.get("schema_version")) and value["schema_version"] == 1 and valid_u32(value.get("support_policy_version")) and value["support_policy_version"] == 1 and value.get("elf_interpreter") is None and value.get("dynamic_loader_cache") is None and value.get("ordered_elf_dependencies") == [] and value.get("manifest_hash") == domain_hash("substrate.e3.runtime-support-manifest.v1", "manifest", value, "manifest_hash")

def validate_entry_value(value, component, installed_path):
    return isinstance(value, dict) and set(value) == {"component", "installed_absolute_path", "device_id", "inode", "file_type", "mode", "owner_uid", "byte_length", "sha256", "runtime_support", "entry_hash"} and value.get("component") == component and value.get("installed_absolute_path") == installed_path and valid_u64(value.get("device_id")) and value["device_id"] > 0 and valid_u64(value.get("inode")) and value["inode"] > 0 and value.get("file_type") == "regular" and valid_u32(value.get("mode")) and value["mode"] == 0o755 and valid_u64(value.get("owner_uid")) and value["owner_uid"] == 0 and valid_u64(value.get("byte_length")) and value["byte_length"] > 0 and valid_digest(value.get("sha256")) and validate_support_value(value.get("runtime_support")) and value.get("entry_hash") == domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", value, "entry_hash")

def validate_record_value(value, source_store_id, expected_reference=None, current_build=False):
    build = value.get("build_input", {}).get("SubstrateSourceBuild", {}) if isinstance(value, dict) else {}
    expected_build = {"source_commit": source_commit, "source_tree": source_tree, "cargo_lock_sha256": lock_hash, "rustc_version": rustc_version, "target_triple": target, "profile": profile}
    reference_ok = expected_reference is None or all(value.get(name) == expected_reference.get(mapped) for name, mapped in (("source_store_id", "source_store_id"), ("source_record_id", "source_record_id"), ("revision", "revision"), ("record_hash", "record_hash")))
    predecessor = value.get("predecessor_ref") if isinstance(value, dict) else None
    predecessor_ok = predecessor is None if value.get("revision") == 1 else isinstance(predecessor, dict) and set(predecessor) == {"source_store_id", "source_record_id", "revision", "record_hash"} and predecessor.get("source_store_id") == source_store_id and valid_id(predecessor.get("source_record_id"), "iar_") and valid_u64(predecessor.get("revision")) and predecessor["revision"] == value.get("revision") - 1 and valid_digest(predecessor.get("record_hash"))
    entries = value.get("entries") if isinstance(value, dict) else None
    build_ok = set(value.get("build_input", {})) == {"SubstrateSourceBuild"} and set(build) == {"source_commit", "source_tree", "cargo_lock_sha256", "rustc_version", "target_triple", "profile"} and isinstance(build.get("source_commit"), str) and re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", build["source_commit"]) and isinstance(build.get("source_tree"), str) and re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", build["source_tree"]) and valid_digest(build.get("cargo_lock_sha256")) and isinstance(build.get("rustc_version"), str) and bool(build["rustc_version"]) and build.get("target_triple") == "x86_64-unknown-linux-musl" and build.get("profile") == "release" and (not current_build or build == expected_build)
    return isinstance(value, dict) and set(value) == {"schema_version", "source_store_id", "source_record_id", "source_stream", "revision", "predecessor_ref", "build_input", "entries", "created_at", "record_hash"} and valid_u32(value.get("schema_version")) and value["schema_version"] == 1 and value.get("source_store_id") == source_store_id and valid_id(value.get("source_record_id"), "iar_") and value.get("source_stream") == "SubstrateSourceBuild" and valid_u64(value.get("revision")) and value["revision"] > 0 and predecessor_ok and build_ok and isinstance(entries, list) and len(entries) == 2 and validate_entry_value(entries[0], "substrate-gateway", "/usr/local/lib/substrate/e3/substrate-gateway") and validate_entry_value(entries[1], "substrate-world-entry", "/usr/local/lib/substrate/e3/substrate-world-entry") and valid_timestamp(value.get("created_at")) and value.get("record_hash") == domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", value, "record_hash") and reference_ok

def validate_head_shape(value, source_store_id):
    reference = value.get("head_ref", {}) if isinstance(value, dict) else {}
    revision = value.get("head_revision") if isinstance(value, dict) else None
    return isinstance(value, dict) and set(value) == {"schema_version", "source_store_id", "source_stream", "head_ref", "head_revision", "predecessor_head_hash", "updated_at", "head_hash"} and valid_u32(value.get("schema_version")) and value["schema_version"] == 1 and value.get("source_store_id") == source_store_id and value.get("source_stream") == "SubstrateSourceBuild" and isinstance(reference, dict) and set(reference) == {"source_store_id", "source_record_id", "revision", "record_hash"} and reference.get("source_store_id") == source_store_id and valid_id(reference.get("source_record_id"), "iar_") and valid_u64(revision) and revision > 0 and valid_u64(reference.get("revision")) and reference["revision"] == revision and valid_digest(reference.get("record_hash")) and ((revision == 1 and value.get("predecessor_head_hash") is None) or (revision > 1 and valid_digest(value.get("predecessor_head_hash")))) and valid_timestamp(value.get("updated_at")) and value.get("head_hash") == domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", value, "head_hash")

def load_valid_store():
    with open(os.path.join(root, "source-store.json"), "rb") as source:
        body = source.read()
    value = json.loads(body)
    if canonical(value) != body or not validate_store_value(value):
        raise RuntimeError("invalid Substrate artifact source store")
    return value

def validate_head_record(value, current_build=False):
    store = load_valid_store()
    if not validate_head_shape(value, store["source_store_id"]):
        return False
    reference = value["head_ref"]
    path = os.path.join(root, "records", "substrate-source-build", f"{reference['revision']:020d}-{reference['source_record_id']}.json")
    if not os.path.exists(path):
        return False
    with open(path, "rb") as source:
        body = source.read()
    record = json.loads(body)
    return canonical(record) == body and validate_record_value(record, store["source_store_id"], reference, current_build)

def fsync_dir(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)

def exact_write(path, value, kind, immutable):
    body = canonical(value)
    if os.path.exists(path):
        with open(path, "rb") as existing:
            existing_body = existing.read()
        if existing_body == body:
            return
        if immutable:
            raise RuntimeError("Substrate artifact source publication conflict")
    temp = os.path.join(os.path.dirname(path), f".e3-artifact-tmp.{uuid7()}.{kind}")
    descriptor = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
    try:
        os.write(descriptor, body)
        os.fchmod(descriptor, 0o640)
        if os.geteuid() == 0:
            os.fchown(descriptor, 0, substrate_gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    if immutable:
        try:
            os.link(temp, path, follow_symlinks=False)
        except FileExistsError:
            with open(path, "rb") as existing:
                if existing.read() != body:
                    raise
        os.unlink(temp)
    else:
        os.replace(temp, path)
    fsync_dir(os.path.dirname(path))
    with open(path, "rb") as published:
        if published.read() != body:
            raise RuntimeError("Substrate artifact source readback mismatch")

def read_canonical_value(path, error):
    with open(path, "rb") as source:
        body = source.read()
    value = json.loads(body)
    if canonical(value) != body:
        raise RuntimeError(error)
    return body, value

def validate_recovery_transaction(contenders):
    if not any(item[3] in {"record", "head"} for item in contenders.values()):
        return
    store = load_valid_store()
    source_store_id = store["source_store_id"]
    head_path = os.path.join(root, "heads", "substrate-source-build.json")
    current_head = None
    current_head_body = None
    if os.path.exists(head_path):
        current_head_body, current_head = read_canonical_value(head_path, "invalid Substrate artifact publication final")
        if not validate_head_record(current_head):
            raise RuntimeError("invalid Substrate artifact publication final")

    pending_records = {}
    pending_head = None
    for final, (_, body, value, kind) in contenders.items():
        if kind == "record":
            if os.path.exists(final):
                current_body, current = read_canonical_value(final, "invalid Substrate artifact publication final")
                if not validate_record_value(current, source_store_id) or current_body != body:
                    raise RuntimeError("Substrate artifact publication temporary conflicts with final")
            else:
                pending_records[os.path.basename(final)] = (body, value)
        elif kind == "head":
            if current_head_body == body:
                continue
            pending_head = value

    if pending_head is None:
        validate_complete_record_chain(current_head)
        if pending_records:
            raise RuntimeError("incomplete Substrate artifact recovery transaction")
        return

    if not validate_head_shape(pending_head, source_store_id):
        raise RuntimeError("invalid Substrate artifact publication temporary")
    if current_head is None:
        if pending_head["predecessor_head_hash"] is not None or pending_head["head_revision"] != 1:
            raise RuntimeError("invalid initial Substrate artifact head temporary")
        expected_predecessor_ref = None
    else:
        if pending_head["predecessor_head_hash"] != current_head["head_hash"] or pending_head["head_revision"] != current_head["head_revision"] + 1:
            raise RuntimeError("stale Substrate artifact head temporary")
        expected_predecessor_ref = current_head["head_ref"]

    reference = pending_head["head_ref"]
    record_name = f"{reference['revision']:020d}-{reference['source_record_id']}.json"
    record_path = os.path.join(root, "records", "substrate-source-build", record_name)
    if record_name in pending_records:
        _, prospective_record = pending_records[record_name]
    elif os.path.exists(record_path):
        _, prospective_record = read_canonical_value(record_path, "invalid Substrate artifact publication final")
    else:
        raise RuntimeError("Substrate artifact head temporary lacks its immutable record")
    if not validate_record_value(prospective_record, source_store_id, reference, current_build=True):
        raise RuntimeError("Substrate artifact head temporary names an invalid immutable record")
    if prospective_record.get("predecessor_ref") != expected_predecessor_ref:
        raise RuntimeError("Substrate artifact record predecessor does not match the current head")
    validate_complete_record_chain(pending_head, pending_records)

def recover_publication():
    locations = [
        (root, "store"),
        (os.path.join(root, "records"), None),
        (os.path.join(root, "records", "substrate-source-build"), "record"),
        (os.path.join(root, "heads"), "head"),
    ]
    contenders = {}
    pattern = re.compile(r"^\.e3-artifact-tmp\.([0-9a-f-]{36})\.(store|record|head)$")
    for directory, allowed_kind in locations:
        for name in os.listdir(directory):
            if not name.startswith(".e3-artifact-tmp."):
                continue
            match = pattern.fullmatch(name)
            if match is None or allowed_kind is None or match.group(2) != allowed_kind:
                raise RuntimeError("unrecognized Substrate artifact publication temporary")
            parsed_uuid = uuid.UUID(match.group(1))
            if parsed_uuid.version != 7 or str(parsed_uuid) != match.group(1):
                raise RuntimeError("invalid Substrate artifact publication temporary identity")
            path = os.path.join(directory, name)
            info = os.stat(path, follow_symlinks=False)
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1 or stat.S_IMODE(info.st_mode) != 0o640:
                raise RuntimeError("unsafe Substrate artifact publication temporary")
            with open(path, "rb") as source:
                body = source.read()
            value = json.loads(body)
            if canonical(value) != body:
                raise RuntimeError("noncanonical Substrate artifact publication temporary")
            kind = match.group(2)
            if kind == "store":
                final = os.path.join(root, "source-store.json")
                valid = validate_store_value(value)
            elif kind == "record":
                final = os.path.join(directory, f"{value.get('revision', 0):020d}-{value.get('source_record_id', '')}.json")
                valid = validate_record_value(value, load_valid_store()["source_store_id"], current_build=True)
            else:
                final = os.path.join(root, "heads", "substrate-source-build.json")
                valid = validate_head_shape(value, load_valid_store()["source_store_id"])
            if not valid:
                raise RuntimeError("invalid Substrate artifact publication temporary")
            if final in contenders:
                raise RuntimeError("ambiguous Substrate artifact publication temporaries")
            contenders[final] = (path, body, value, kind)
    validate_recovery_transaction(contenders)
    for final, (temporary, body, value, kind) in sorted(contenders.items(), key=lambda item: {"store": 0, "record": 1, "head": 2}[item[1][3]]):
        if os.path.exists(final):
            with open(final, "rb") as source:
                current_body = source.read()
            current_value = json.loads(current_body)
            current_valid = canonical(current_value) == current_body
            if kind == "store":
                current_valid = current_valid and validate_store_value(current_value)
            elif kind == "record":
                current_valid = current_valid and validate_record_value(current_value, load_valid_store()["source_store_id"])
            else:
                current_valid = current_valid and validate_head_record(current_value)
            if not current_valid:
                raise RuntimeError("invalid Substrate artifact publication final")
            if current_body == body:
                os.unlink(temporary)
                fsync_dir(os.path.dirname(temporary))
                continue
            if kind != "head":
                raise RuntimeError("Substrate artifact publication temporary conflicts with final")
        if kind == "head":
            reference = value["head_ref"]
            record_path = os.path.join(root, "records", "substrate-source-build", f"{reference['revision']:020d}-{reference['source_record_id']}.json")
            if not os.path.exists(record_path):
                raise RuntimeError("Substrate artifact head temporary lacks its immutable record")
            if not validate_head_record(value, current_build=True):
                raise RuntimeError("Substrate artifact head temporary names an invalid immutable record")
            predecessor = value["predecessor_head_hash"]
            if os.path.exists(final):
                with open(final, "rb") as source:
                    current_value = json.loads(source.read())
                if predecessor != current_value.get("head_hash") or value["head_revision"] != current_value.get("head_revision", 0) + 1:
                    raise RuntimeError("stale Substrate artifact head temporary")
            elif predecessor is not None or value["head_revision"] != 1:
                raise RuntimeError("invalid initial Substrate artifact head temporary")
            os.replace(temporary, final)
        else:
            os.link(temporary, final, follow_symlinks=False)
            os.unlink(temporary)
        fsync_dir(os.path.dirname(final))
        with open(final, "rb") as source:
            published_body = source.read()
            if published_body != body:
                raise RuntimeError("recovered Substrate artifact publication readback mismatch")
        published = json.loads(published_body)
        if (kind == "store" and not validate_store_value(published)) or (kind == "record" and not validate_record_value(published, load_valid_store()["source_store_id"], current_build=True)) or (kind == "head" and not validate_head_record(published, current_build=True)):
            raise RuntimeError("recovered Substrate artifact publication is incomplete")

def stage_publication(path, value, kind):
    body = canonical(value)
    temporary = os.path.join(os.path.dirname(path), f".e3-artifact-tmp.{uuid7()}.{kind}")
    descriptor = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
    try:
        os.write(descriptor, body)
        os.fchmod(descriptor, 0o640)
        if os.geteuid() == 0:
            os.fchown(descriptor, 0, substrate_gid)
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    return temporary, body

def commit_staged_publication(temporary, path, body, immutable):
    if immutable:
        try:
            os.link(temporary, path, follow_symlinks=False)
        except FileExistsError:
            with open(path, "rb") as existing:
                if existing.read() != body:
                    raise RuntimeError("Substrate artifact immutable publication conflict")
        os.unlink(temporary)
    else:
        os.replace(temporary, path)
    fsync_dir(os.path.dirname(path))
    with open(path, "rb") as published:
        if published.read() != body:
            raise RuntimeError("Substrate staged artifact publication readback mismatch")

def validate_complete_record_chain(head, pending_records=None):
    directory = os.path.join(root, "records", "substrate-source-build")
    records = {name for name in os.listdir(directory) if not name.startswith(".")}
    pending_records = {} if pending_records is None else pending_records
    if records.intersection(pending_records):
        raise RuntimeError("conflicting Substrate artifact source record")
    records.update(pending_records)
    if any(re.fullmatch(r"[0-9]{20}-iar_[0-9a-f-]{36}\.json", name) is None for name in records):
        raise RuntimeError("unknown Substrate artifact source record")
    if head is None:
        if records:
            raise RuntimeError("orphan Substrate artifact source record")
        return
    store = load_valid_store()
    if not validate_head_shape(head, store["source_store_id"]):
        raise RuntimeError("invalid Substrate artifact source head")
    reachable = set()
    reference = head["head_ref"]
    while reference is not None:
        name = f"{reference['revision']:020d}-{reference['source_record_id']}.json"
        if name in reachable or name not in records:
            raise RuntimeError("broken Substrate artifact source predecessor chain")
        reachable.add(name)
        if name in pending_records:
            body, value = pending_records[name]
        else:
            with open(os.path.join(directory, name), "rb") as source:
                body = source.read()
            value = json.loads(body)
        if canonical(value) != body or not validate_record_value(value, store["source_store_id"], reference):
            raise RuntimeError("invalid Substrate artifact source predecessor record")
        reference = value.get("predecessor_ref")
    if reachable != records:
        raise RuntimeError("orphan Substrate artifact source record")

def file_support(path, recorded_path=None):
    logical_path = path if recorded_path is None else recorded_path
    ca_path = "/etc/ssl/certs/ca-certificates.crt"
    if logical_path == ca_path:
        if not path.endswith(ca_path):
            raise RuntimeError("logical E3 CA path does not match its installed root")
        root_path = path[:-len(ca_path)] or "/"
        directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC | os.O_NOFOLLOW

        def mount_id(descriptor):
            fdinfo = os.open(f"/proc/self/fdinfo/{descriptor}", os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
            try:
                payload = os.read(fdinfo, 4097)
            finally:
                os.close(fdinfo)
            if len(payload) == 4097:
                raise RuntimeError("required E3 CA mount identity is oversized")
            values = [line[len(b"mnt_id:\t"):] for line in payload.splitlines() if line.startswith(b"mnt_id:\t")]
            if len(values) != 1 or not values[0].isdigit() or int(values[0]) == 0:
                raise RuntimeError("required E3 CA mount identity is malformed")
            return int(values[0])

        def trusted_directory(parent, name, boundary_device=None, boundary_mount_id=None):
            try:
                descriptor = os.open(os.fsencode(name), directory_flags, dir_fd=parent)
            except OSError as error:
                raise RuntimeError("required E3 CA directory is not trusted") from error
            info = os.fstat(descriptor)
            descriptor_mount_id = mount_id(descriptor)
            if not stat.S_ISDIR(info.st_mode) or info.st_uid != 0 or stat.S_IMODE(info.st_mode) & 0o022 or (boundary_device is not None and info.st_dev != boundary_device) or (boundary_mount_id is not None and descriptor_mount_id != boundary_mount_id):
                os.close(descriptor)
                raise RuntimeError("required E3 CA directory is not trusted")
            return descriptor, info, descriptor_mount_id

        def endpoint(parent, name, boundary_device, boundary_mount_id):
            try:
                descriptor = os.open(os.fsencode(name), os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW, dir_fd=parent)
            except OSError as error:
                raise RuntimeError("required E3 CA endpoint is not trusted") from error
            before = os.fstat(descriptor)
            if not stat.S_ISREG(before.st_mode) or before.st_dev != boundary_device or mount_id(descriptor) != boundary_mount_id or before.st_uid != 0 or before.st_nlink != 1 or stat.S_IMODE(before.st_mode) & 0o022 or before.st_size < 0 or before.st_size > 64 * 1024 * 1024:
                os.close(descriptor)
                raise RuntimeError("required E3 CA endpoint is not trusted")
            digest = hashlib.sha256()
            offset = 0
            while True:
                data = os.pread(descriptor, 64 * 1024, offset)
                if not data:
                    break
                digest.update(data)
                offset += len(data)
            after = os.fstat(descriptor)
            fields = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid, value.st_nlink, value.st_size)
            if offset != before.st_size or fields(before) != fields(after):
                os.close(descriptor)
                raise RuntimeError("required E3 CA endpoint changed while hashing")
            return descriptor, before, digest.hexdigest()

        def resolve():
            opened = []
            try:
                root_descriptor = os.open(root_path, directory_flags)
                opened.append(root_descriptor)
                etc, etc_info, boundary_mount_id = trusted_directory(root_descriptor, "etc")
                opened.append(etc)
                boundary_device = etc_info.st_dev
                ssl, _, _ = trusted_directory(etc, "ssl", boundary_device, boundary_mount_id)
                opened.append(ssl)
                certs, _, _ = trusted_directory(ssl, "certs", boundary_device, boundary_mount_id)
                opened.append(certs)
                leaf = os.stat(os.fsencode("ca-certificates.crt"), dir_fd=certs, follow_symlinks=False)
                if stat.S_ISREG(leaf.st_mode):
                    return endpoint(certs, "ca-certificates.crt", boundary_device, boundary_mount_id)
                if not stat.S_ISLNK(leaf.st_mode) or leaf.st_uid != 0 or leaf.st_dev != boundary_device:
                    raise RuntimeError("logical E3 CA leaf is neither the direct file nor exact link")
                raw_target = os.readlink(os.fsencode("ca-certificates.crt"), dir_fd=certs)
                if raw_target != b"../../ca-certificates/extracted/tls-ca-bundle.pem":
                    raise RuntimeError("logical E3 CA link target is not the sole admitted relative target")
                stack = [etc, ssl, certs]
                for _ in range(2):
                    if len(stack) <= 1:
                        raise RuntimeError("logical E3 CA link escapes the held /etc boundary")
                    stack.pop()
                ca_certificates, _, _ = trusted_directory(stack[-1], "ca-certificates", boundary_device, boundary_mount_id)
                opened.append(ca_certificates)
                extracted, _, _ = trusted_directory(ca_certificates, "extracted", boundary_device, boundary_mount_id)
                opened.append(extracted)
                return endpoint(extracted, "tls-ca-bundle.pem", boundary_device, boundary_mount_id)
            finally:
                for descriptor in reversed(opened):
                    os.close(descriptor)

        first_descriptor, first_info, first_digest = resolve()
        try:
            second_descriptor, second_info, second_digest = resolve()
            try:
                fields = lambda value: (value.st_dev, value.st_ino, value.st_mode, value.st_uid, value.st_gid, value.st_nlink, value.st_size)
                if fields(first_info) != fields(second_info) or first_digest != second_digest:
                    raise RuntimeError("logical E3 CA link or endpoint was substituted")
            finally:
                os.close(second_descriptor)
            return {"absolute_path": ca_path, "device_id": first_info.st_dev, "inode": first_info.st_ino, "mode": stat.S_IMODE(first_info.st_mode), "byte_length": first_info.st_size, "sha256": first_digest}
        finally:
            os.close(first_descriptor)

    info = os.stat(path, follow_symlinks=False)
    if not stat.S_ISREG(info.st_mode) or stat.S_IMODE(info.st_mode) & 0o022:
        raise RuntimeError("required E3 support object is not regular")
    with open(path, "rb") as source:
        digest = hashlib.sha256(source.read()).hexdigest()
    return {"absolute_path": logical_path, "device_id": info.st_dev, "inode": info.st_ino, "mode": stat.S_IMODE(info.st_mode), "byte_length": info.st_size, "sha256": digest}

def file_record(path):
    installed_suffixes = ["/usr/local/lib/substrate/e3/substrate-gateway", "/usr/local/lib/substrate/e3/substrate-world-entry"]
    test_prefix = next((path[:-len(suffix)] for suffix in installed_suffixes if path.endswith(suffix)), "")
    info = os.stat(path, follow_symlinks=False)
    if not stat.S_ISREG(info.st_mode) or stat.S_IMODE(info.st_mode) != 0o755 or info.st_uid != 0 or os.listxattr(path, follow_symlinks=False):
        raise RuntimeError("installed E3 artifact metadata mismatch")
    with open(path, "rb") as source:
        data = source.read()
    if len(data) < 64 or data[:16] != b"\x7fELF\x02\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00" or struct.unpack_from("<H", data, 18)[0] != 62 or struct.unpack_from("<I", data, 20)[0] != 1 or struct.unpack_from("<H", data, 52)[0] != 64:
        raise RuntimeError("installed E3 artifact is not x86-64 ELF")
    elf_type = struct.unpack_from("<H", data, 16)[0]
    entrypoint = struct.unpack_from("<Q", data, 24)[0]
    phoff = struct.unpack_from("<Q", data, 32)[0]
    phentsize, phnum = struct.unpack_from("<HH", data, 54)
    if phentsize != 56 or phnum == 0 or phoff + phentsize * phnum > len(data):
        raise RuntimeError("malformed installed E3 ELF program table")
    dynamic = None
    loads = []
    for index in range(phnum):
        offset = phoff + index * phentsize
        kind, flags = struct.unpack_from("<II", data, offset)
        file_offset, virtual_address = struct.unpack_from("<QQ", data, offset + 8)
        file_length, memory_length, alignment = struct.unpack_from("<QQQ", data, offset + 32)
        if file_length > memory_length or file_offset + file_length > len(data) or virtual_address + memory_length >= 1 << 64 or (alignment > 1 and ((alignment & (alignment - 1)) or file_offset % alignment != virtual_address % alignment)):
            raise RuntimeError("malformed installed E3 ELF segment")
        if kind == 1:
            loads.append((file_offset, virtual_address, file_length, memory_length, flags))
        if kind == 3:
            raise RuntimeError("installed E3 artifact has an interpreter")
        if kind == 2:
            if dynamic is not None:
                raise RuntimeError("installed E3 artifact has multiple dynamic segments")
            if file_length != memory_length:
                raise RuntimeError("malformed installed E3 dynamic segment")
            dynamic = (file_offset, virtual_address, file_length)
    if not loads or not any(flags & 1 and entrypoint >= virtual_address and entrypoint < virtual_address + file_length for _, virtual_address, file_length, _, flags in loads):
        raise RuntimeError("installed E3 artifact has no usable executable entrypoint")
    if elf_type == 2 and dynamic is None:
        model = "StaticExec"
    elif elf_type == 3 and dynamic is not None:
        dynamic_offset, dynamic_address, dynamic_length = dynamic
        if not dynamic_length or dynamic_length % 16 or dynamic_offset + dynamic_length > len(data):
            raise RuntimeError("malformed installed E3 dynamic segment")
        allowed = {0, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 21, 25, 26, 27, 28, 30, 0x6FFFFFFB, 0x6FFFFFF9, 0x6FFFFEF5}
        values = {}
        raw_entries = []
        terminal_end = None
        for position in range(dynamic_offset, dynamic_offset + dynamic_length, 16):
            tag, value = struct.unpack_from("<QQ", data, position)
            raw_entries.append(data[position:position + 16])
            if tag not in allowed:
                raise RuntimeError("unsupported installed E3 dynamic tag")
            if tag == 0:
                terminal_end = position + 16
                break
            if tag in values:
                raise RuntimeError("duplicate static-PIE dynamic tag")
            values[tag] = value
        if terminal_end is None or any(data[terminal_end:dynamic_offset + dynamic_length]):
            raise RuntimeError("invalid installed E3 dynamic terminator")
        if values.get(9) != 24 or not values.get(8) or values[8] % 24 or values.get(0x6FFFFFF9) != values[8] // 24 or values.get(10) != 1 or values.get(11) != 24 or 5 not in values or 6 not in values or not ({4, 0x6FFFFEF5} & values.keys()) or values.get(30) != 8 or values.get(0x6FFFFFFB) != 0x08000001 or values.get(21) != 0 or ((25 in values) != (27 in values)) or ((26 in values) != (28 in values)):
            raise RuntimeError("unsupported static-PIE relocation closure")
        def extent(address, length):
            if address + length >= 1 << 64:
                raise RuntimeError("installed E3 dynamic extent overflows")
            for file_offset, virtual_address, file_length, _, _ in loads:
                if address >= virtual_address and address + length <= virtual_address + file_length:
                    start = file_offset + address - virtual_address
                    if start + length <= len(data):
                        return data[start:start + length]
            raise RuntimeError("installed E3 dynamic address escapes load segments")
        if extent(dynamic_address, dynamic_length) != data[dynamic_offset:dynamic_offset + dynamic_length] or extent(values[5], 1) != b"\x00" or any(extent(values[6], 24)):
            raise RuntimeError("installed E3 dynamic metadata extent mismatch")
        symbol_counts = []
        if 4 in values:
            buckets, chains = struct.unpack("<II", extent(values[4], 8))
            table = extent(values[4], 8 + 4 * (buckets + chains))
            if chains != 1 or any(struct.unpack_from("<I", table, position)[0] for position in range(8, len(table), 4)):
                raise RuntimeError("installed E3 SysV hash exposes extra symbols")
            symbol_counts.append(chains)
        if 0x6FFFFEF5 in values:
            buckets, symbol_offset, bloom_words, _ = struct.unpack("<IIII", extent(values[0x6FFFFEF5], 16))
            prefix = extent(values[0x6FFFFEF5], 16 + 8 * bloom_words + 4 * buckets)
            bloom_end = 16 + 8 * bloom_words
            if not buckets or not bloom_words or bloom_words & (bloom_words - 1) or symbol_offset != 1 or any(prefix[16:bloom_end]) or any(struct.unpack_from("<I", prefix, position)[0] for position in range(bloom_end, len(prefix), 4)):
                raise RuntimeError("installed E3 GNU hash exposes extra symbols")
            symbol_counts.append(symbol_offset)
        if not symbol_counts or any(count != 1 for count in symbol_counts):
            raise RuntimeError("installed E3 dynamic symbol count is not exact")
        for tag in (3, 12, 13):
            if tag in values:
                extent(values[tag], 1)
        for address_tag, size_tag in ((25, 27), (26, 28)):
            if address_tag in values:
                if not values[size_tag]:
                    raise RuntimeError("empty installed E3 dynamic array")
                extent(values[address_tag], values[size_tag])
        relocations = extent(values[7], values[8])
        if any(struct.unpack_from("<Q", relocations, position + 8)[0] != 8 for position in range(0, len(relocations), 24)):
            raise RuntimeError("installed E3 artifact has a non-relative relocation")
        model = {"StaticPie": {"dynamic_segment_file_offset": dynamic_offset, "dynamic_segment_byte_length": dynamic_length, "rela_virtual_address": values[7], "rela_byte_length": values[8], "rela_entry_byte_length": values[9], "relative_relocation_count": values[0x6FFFFFF9], "ordered_dynamic_entries_sha256": hashlib.sha256(b"".join(raw_entries)).hexdigest()}}
    else:
        raise RuntimeError("installed E3 artifact is dynamically linked")
    system_path = test_prefix + "/etc/codex"
    system = os.stat(system_path, follow_symlinks=False)
    if not stat.S_ISDIR(system.st_mode) or stat.S_IMODE(system.st_mode) != 0o755 or system.st_uid != 0 or system.st_gid != 0 or os.listdir(system_path):
        raise RuntimeError("E3 system configuration mount target is not exact")
    common_paths = ["/etc/hosts", "/etc/nsswitch.conf", "/etc/passwd", "/etc/group", "/etc/resolv.conf", "/etc/ssl/certs/ca-certificates.crt"]
    support = {"schema_version": 1, "support_policy_version": 1, "elf_execution_model": model, "elf_interpreter": None, "dynamic_loader_cache": None, "ordered_elf_dependencies": [], "ordered_present_common_files": [file_support(test_prefix + support_path, support_path) for support_path in common_paths], "system_config_mount_target": {"absolute_path": "/etc/codex", "device_id": system.st_dev, "inode": system.st_ino, "mode": stat.S_IMODE(system.st_mode), "owner_uid": system.st_uid, "owner_gid": system.st_gid, "ordered_entry_names": []}, "manifest_hash": ""}
    support["manifest_hash"] = domain_hash("substrate.e3.runtime-support-manifest.v1", "manifest", support, "manifest_hash")
    return info, hashlib.sha256(data).hexdigest(), support

os.makedirs(os.path.join(root, "records", "substrate-source-build"), mode=0o750, exist_ok=True)
os.makedirs(os.path.join(root, "heads"), mode=0o750, exist_ok=True)
substrate_gid = grp.getgrnam("substrate").gr_gid
for directory in [root, os.path.join(root, "records"), os.path.join(root, "records", "substrate-source-build"), os.path.join(root, "heads")]:
    os.chmod(directory, 0o750)
    if os.geteuid() == 0:
        os.chown(directory, 0, substrate_gid)
lock_path = os.path.join(root, "lock")
lock_fd = os.open(lock_path, os.O_RDWR | os.O_CREAT | os.O_CLOEXEC | os.O_NOFOLLOW, 0o640)
os.fchmod(lock_fd, 0o640)
if os.geteuid() == 0:
    os.fchown(lock_fd, 0, substrate_gid)
fcntl.flock(lock_fd, fcntl.LOCK_EX)
try:
    recover_publication()
    root_info = os.stat(root, follow_symlinks=False)
    store_path = os.path.join(root, "source-store.json")
    if os.path.exists(store_path):
        with open(store_path, "rb") as source:
            store_body = source.read()
        store = json.loads(store_body)
        if canonical(store) != store_body or not validate_store_value(store):
            raise RuntimeError("invalid Substrate artifact source store")
    else:
        store = {"schema_version": 1, "source_store_id": "ias_" + uuid7(), "root": {"physical_path": root, "physical_identity": {"Linux": {"device_id": root_info.st_dev, "inode": root_info.st_ino}}}, "created_at": timestamp(), "store_hash": ""}
        store["store_hash"] = domain_hash("substrate.e3.installer-artifact-source-store.v1", "store", store, "store_hash")
        exact_write(store_path, store, "store", True)
    entries = []
    for component, path in [("substrate-gateway", gateway), ("substrate-world-entry", wrapper)]:
        info, digest, support = file_record(path)
        installed_path = "/usr/local/lib/substrate/e3/substrate-gateway" if component == "substrate-gateway" else "/usr/local/lib/substrate/e3/substrate-world-entry"
        entry = {"component": component, "installed_absolute_path": installed_path, "device_id": info.st_dev, "inode": info.st_ino, "file_type": "regular", "mode": stat.S_IMODE(info.st_mode), "owner_uid": info.st_uid, "byte_length": info.st_size, "sha256": digest, "runtime_support": support, "entry_hash": ""}
        entry["entry_hash"] = domain_hash("substrate.e3.installer-artifact-source-entry.v1", "entry", entry, "entry_hash")
        entries.append(entry)
    build = {"SubstrateSourceBuild": {"source_commit": source_commit, "source_tree": source_tree, "cargo_lock_sha256": lock_hash, "rustc_version": rustc_version, "target_triple": target, "profile": profile}}
    head_path = os.path.join(root, "heads", "substrate-source-build.json")
    prior = None
    if os.path.exists(head_path):
        with open(head_path, "rb") as source:
            prior = json.loads(source.read())
        ref = prior["head_ref"]
        prior_path = os.path.join(root, "records", "substrate-source-build", f"{ref['revision']:020d}-{ref['source_record_id']}.json")
        with open(prior_path, "rb") as source:
            prior_record = json.loads(source.read())
        validate_complete_record_chain(prior)
        if prior_record["build_input"] == build and prior_record["entries"] == entries:
            raise SystemExit(0)
    else:
        validate_complete_record_chain(None)
    revision = 1 if prior is None else prior["head_revision"] + 1
    record = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_record_id": "iar_" + uuid7(), "source_stream": "SubstrateSourceBuild", "revision": revision, "predecessor_ref": None if prior is None else prior["head_ref"], "build_input": build, "entries": entries, "created_at": timestamp(), "record_hash": ""}
    record["record_hash"] = domain_hash("substrate.e3.installer-artifact-source-record.v1", "record", record, "record_hash")
    record_path = os.path.join(root, "records", "substrate-source-build", f"{revision:020d}-{record['source_record_id']}.json")
    reference = {"source_store_id": store["source_store_id"], "source_record_id": record["source_record_id"], "revision": revision, "record_hash": record["record_hash"]}
    head = {"schema_version": 1, "source_store_id": store["source_store_id"], "source_stream": "SubstrateSourceBuild", "head_ref": reference, "head_revision": revision, "predecessor_head_hash": None if prior is None else prior["head_hash"], "updated_at": timestamp(), "head_hash": ""}
    head["head_hash"] = domain_hash("substrate.e3.installer-artifact-source-head.v1", "head", head, "head_hash")
    record_temporary, record_body = stage_publication(record_path, record, "record")
    head_temporary, head_body = stage_publication(head_path, head, "head")
    if prior is not None:
        with open(head_path, "rb") as current:
            if json.loads(current.read()) != prior:
                raise RuntimeError("Substrate artifact source head CAS conflict")
    commit_staged_publication(record_temporary, record_path, record_body, True)
    commit_staged_publication(head_temporary, head_path, head_body, False)
    fsync_dir(os.path.join(root, "records", "substrate-source-build"))
    fsync_dir(os.path.join(root, "records"))
    fsync_dir(os.path.join(root, "heads"))
    fsync_dir(root)
finally:
    os.close(lock_fd)
PY
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
    linux_snapshot_path "e3-artifact-directory" "/usr/local/lib/substrate/e3"
    linux_snapshot_path "e3-gateway-binary" "/usr/local/lib/substrate/e3/substrate-gateway"
    linux_snapshot_path "e3-world-entry-binary" "/usr/local/lib/substrate/e3/substrate-world-entry"
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
    linux_snapshot_path "e3-system-config-target" "/etc/codex"
    linux_snapshot_path "e3-install-bootstrap-authority" "${SUBSTRATE_STATE_PATH}/install-bootstrap-authority-v1"
    linux_snapshot_path "e3-substrate-artifact-source" "${SUBSTRATE_STATE_PATH}/runtime-artifacts-v1"
    linux_snapshot_path "e3-codex-artifact-source" "${WORLD_DEPS_ROOT_PATH}/runtime-artifacts-v1"
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
    local required_e3_value
    for required_e3_value in \
        E3_GATEWAY_BIN_PATH \
        E3_WORLD_ENTRY_BIN_PATH \
        SUBSTRATE_SOURCE_COMMIT \
        SUBSTRATE_SOURCE_TREE \
        SUBSTRATE_CARGO_LOCK_SHA256 \
        SUBSTRATE_RUSTC_VERSION \
        SUBSTRATE_SOURCE_TARGET \
        SUBSTRATE_SOURCE_PROFILE \
        SUBSTRATE_SOURCE_BUILD_PERFORMED; do
        if [[ -z "${!required_e3_value:-}" ]]; then
            printf 'world-lifecycle missing required E3-D context: %s\n' "${required_e3_value}" >&2
            return 2
        fi
    done

    echo "==> Ensuring ${SUBSTRATE_GROUP} group and membership"
    ensure_substrate_group_exists
    ensure_user_in_group "${INVOKING_USER}"

    echo "==> Installing world-service to /usr/local/bin (sudo will prompt if needed)"
    sudo_cmd install -Dm0755 "${WORLD_AGENT_BIN_PATH}" /usr/local/bin/substrate-world-service
    echo "==> Installing substrate-gateway to /usr/local/bin (no dedicated service)"
    sudo_cmd install -Dm0755 "${GATEWAY_BIN_PATH}" /usr/local/bin/substrate-gateway
    echo "==> Installing descriptor-pinned E3 static artifacts"
    sudo_cmd install -d -m0755 -o root -g root /usr/local/lib/substrate/e3
    sudo_cmd install -Dm0755 -o root -g root \
        "${E3_GATEWAY_BIN_PATH}" \
        /usr/local/lib/substrate/e3/substrate-gateway
    sudo_cmd install -Dm0755 -o root -g root \
        "${E3_WORLD_ENTRY_BIN_PATH}" \
        /usr/local/lib/substrate/e3/substrate-world-entry
    echo "==> Installing ACL bridge helper to ${ACL_HELPER_INSTALL_PATH}"
    sudo_cmd install -Dm0755 "${ACL_HELPER_SOURCE_PATH}" "${ACL_HELPER_INSTALL_PATH}"
    echo "==> Installing Linux lifecycle executor to /usr/libexec/substrate/substrate-lifecycle-linux"
    sudo_cmd install -Dm0755 "${LIFECYCLE_EXECUTOR_BIN_PATH}" /usr/libexec/substrate/substrate-lifecycle-linux

    echo "==> Ensuring runtime directories exist"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" /run/substrate
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${SUBSTRATE_STATE_PATH}"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${WORLD_DEPS_ROOT_PATH}"
    sudo_cmd install -d -m0750 -o root -g "${SUBSTRATE_GROUP}" "${WORLD_DEPS_BIN_PATH}"

    echo "==> Provisioning the E3 system configuration mount target"
    provision_e3_system_config_mount_target_v1
    echo "==> Publishing the authenticated installed-home bootstrap"
    publish_installed_home_bootstrap_v1
    echo "==> Publishing the readback-validated E3 Substrate artifact source"
    publish_substrate_artifact_source_v1

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
    echo "==> Installed E3 static artifacts"
    sudo_cmd ls -l \
        /usr/local/lib/substrate/e3/substrate-gateway \
        /usr/local/lib/substrate/e3/substrate-world-entry
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

    if [[ -f "$(linux_snapshot_service_path)" ]]; then
        while IFS=$'\t' read -r unit _enabled _active; do
            [[ "${unit}" == *.service ]] || continue
            if sudo_cmd systemctl is-active --quiet "${unit}"; then
                linux_stop_service_unit "${unit}"
            fi
        done <"$(linux_snapshot_service_path)"
    fi

    if [[ -f "$(linux_snapshot_manifest_path)" ]]; then
        tac "$(linux_snapshot_manifest_path)" | while IFS=$'\t' read -r label target type mode backup owner group; do
            [[ -n "${label}" ]] || continue
            linux_restore_path "${label}" "${target}" "${type}" "${mode}" "${backup}" "${owner}" "${group}"
        done
    fi

    sudo_cmd systemctl daemon-reload || true

    if [[ -f "$(linux_snapshot_service_path)" ]]; then
        local unit_kind
        for unit_kind in socket service; do
            while IFS=$'\t' read -r unit enabled active; do
                [[ "${unit}" == *."${unit_kind}" ]] || continue
                linux_restore_service_state "${unit}" "${enabled}" "${active}"
            done <"$(linux_snapshot_service_path)"
        done
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
