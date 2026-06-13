#!/usr/bin/env bash
set -euo pipefail

SOURCE_PATH="${BASH_SOURCE[0]}"
while [[ -L "${SOURCE_PATH}" ]]; do
    SOURCE_DIR="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"
    SOURCE_PATH="$(readlink "${SOURCE_PATH}")"
    [[ "${SOURCE_PATH}" != /* ]] && SOURCE_PATH="${SOURCE_DIR}/${SOURCE_PATH}"
done
SCRIPT_DIR="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"
CANONICAL_UNIT_SOURCE_DIR=""
VM_NAME="${LIMA_VM_NAME:-substrate}"
PROFILE="${LIMA_PROFILE_PATH:-${SCRIPT_DIR}/lima/substrate.yaml}"
PROJECT_PATH=""
PROJECT_PATH_EXPLICIT=0
CHECK_ONLY=0
BUILD_PROFILE="${LIMA_BUILD_PROFILE:-release}"
LAYOUT_SENTINEL="/etc/substrate-lima-layout"
LAYOUT_VERSION="socket-parity-v2-staged-workspace-v1"
STAGED_WORKSPACE_ROOT="/var/lib/substrate/staged-workspace"
STAGED_WORKSPACE_CURRENT="${STAGED_WORKSPACE_ROOT}/current"
STAGED_WORKSPACE_MANIFEST_NAME=".substrate-lima-stage-manifest"
WAIT_TIMEOUT=120
SKIP_GUEST_BUILD="${SUBSTRATE_LIMA_SKIP_GUEST_BUILD:-0}"

log() {
    printf '==> %s\n' "$1"
}

warn() {
    printf 'WARN | %s\n' "$1" >&2
}

fatal() {
    printf 'ERROR | %s\n' "$1" >&2
    exit 1
}

usage() {
    cat <<'USAGE'
Usage: scripts/mac/lima-warm.sh [options] [<project-path>]

Options:
  --check-only      Report the current Lima VM status without creating or provisioning it
  -h, --help        Show this help text

Arguments:
  <project-path>    Repository or release path to stage for guest provisioning (default: current directory)
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --check-only)
            CHECK_ONLY=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            if [[ -z "${PROJECT_PATH}" ]]; then
                PROJECT_PATH="$1"
                PROJECT_PATH_EXPLICIT=1
            else
                fatal "Unexpected argument: $1"
            fi
            shift
            ;;
    esac
done

if [[ -z "${PROJECT_PATH}" ]]; then
    PROJECT_PATH="$(pwd)"
fi
PROJECT_PATH="$(cd "${PROJECT_PATH}" && pwd)"

project_unit_source_dir="${PROJECT_PATH}/scripts/mac/lima/units"
script_unit_source_dir="${SCRIPT_DIR}/lima/units"
if [[ -d "${project_unit_source_dir}" ]]; then
    CANONICAL_UNIT_SOURCE_DIR="${project_unit_source_dir}"
elif [[ -d "${script_unit_source_dir}" ]]; then
    CANONICAL_UNIT_SOURCE_DIR="${script_unit_source_dir}"
else
    fatal "Canonical guest unit directory not found. Expected ${project_unit_source_dir} or ${script_unit_source_dir}."
fi

require_cmd() {
    local name="$1"
    if ! command -v "${name}" >/dev/null 2>&1; then
        fatal "Required command '${name}' not found. Install it and rerun."
    fi
}

check_only_status() {
    local host_os
    host_os="$(uname -s 2>/dev/null || echo "unknown")"
    if [[ "${host_os}" != "Darwin" ]]; then
        echo "[check-only] Host ${host_os} does not support Lima provisioning; skipping warm check."
        exit 0
    fi

    local virtualization=1
    if command -v sysctl >/dev/null 2>&1; then
        virtualization="$(sysctl -n kern.hv_support 2>/dev/null || echo "0")"
    fi
    if [[ "${virtualization}" != "1" ]]; then
        echo "[check-only] Virtualization.framework unavailable (sysctl kern.hv_support != 1)."
    else
        echo "[check-only] Virtualization.framework detected."
    fi

    for binary in limactl jq envsubst file; do
        if command -v "${binary}" >/dev/null 2>&1; then
            echo "[check-only] ${binary} found."
        else
            echo "[check-only] ${binary} missing."
        fi
    done

    if limactl list "${VM_NAME}" >/dev/null 2>&1; then
        local status
        status="$(limactl list "${VM_NAME}" --json | jq -r '.status // "unknown"')"
        echo "[check-only] Lima VM '${VM_NAME}' status: ${status}"
        if [[ "${status}" == "Running" ]]; then
            if limactl shell "${VM_NAME}" sudo -n test -S /run/substrate.sock >/dev/null 2>&1; then
                local ls_output
                ls_output="$(limactl shell "${VM_NAME}" sudo ls -l /run/substrate.sock 2>/dev/null || true)"
                echo "[check-only] Agent socket metadata:"
                [[ -n "${ls_output}" ]] && echo "    ${ls_output}"
            else
                echo "[check-only] Agent socket missing inside guest."
            fi
            if limactl shell "${VM_NAME}" sudo -n test -f /etc/systemd/system/substrate-world-service.service >/dev/null 2>&1; then
                if limactl shell "${VM_NAME}" sudo -n grep -q '^Environment=WORLD_NETFILTER_ENABLE=1$' /etc/systemd/system/substrate-world-service.service >/dev/null 2>&1; then
                    echo "[check-only] Guest systemd env includes WORLD_NETFILTER_ENABLE=1."
                else
                    echo "[check-only] Guest systemd env does not include WORLD_NETFILTER_ENABLE=1."
                fi
            else
                echo "[check-only] Guest systemd service file for substrate-world-service is missing."
            fi
            if limactl shell "${VM_NAME}" sudo -n test -x /usr/local/bin/substrate-gateway >/dev/null 2>&1; then
                echo "[check-only] Guest gateway binary present at /usr/local/bin/substrate-gateway."
            else
                echo "[check-only] Guest gateway binary missing at /usr/local/bin/substrate-gateway."
            fi
        fi
    else
        echo "[check-only] Lima VM '${VM_NAME}' not found."
    fi

    exit 0
}

check_host_prereqs() {
    local host_os
    host_os="$(uname -s 2>/dev/null || echo "unknown")"
    if [[ "${host_os}" != "Darwin" ]]; then
        fatal "This helper only supports macOS hosts (detected ${host_os})."
    fi
    require_cmd limactl
    require_cmd jq
    require_cmd envsubst
    require_cmd file
    require_cmd sysctl

    local hv
    hv="$(sysctl -n kern.hv_support 2>/dev/null || echo "0")"
    if [[ "${hv}" != "1" ]]; then
        fatal "Virtualization.framework unavailable (sysctl kern.hv_support != 1). Enable it in System Settings > Privacy & Security."
    fi
}

render_profile() {
    TMP_PROFILE="$(mktemp)"
    trap 'rm -f "$TMP_PROFILE"' EXIT
    PROJECT="${PROJECT_PATH}" envsubst < "${PROFILE}" > "${TMP_PROFILE}"
}

vm_exists() {
    limactl list "${VM_NAME}" >/dev/null 2>&1
}

vm_status() {
    limactl list "${VM_NAME}" --json | jq -r '.status // "unknown"'
}

create_vm() {
    log "Creating Lima VM '${VM_NAME}' from ${PROFILE} ..."
    limactl start --tty=false --name "${VM_NAME}" "${TMP_PROFILE}"
}

start_vm() {
    log "Starting existing Lima VM '${VM_NAME}' ..."
    limactl start "${VM_NAME}"
}

wait_for_running() {
    local remaining=${WAIT_TIMEOUT}
    while (( remaining > 0 )); do
        local status
        status="$(vm_status)"
        if [[ "${status}" == "Running" ]]; then
            log "Lima VM '${VM_NAME}' is running."
            return
        fi
        sleep 2
        remaining=$((remaining - 2))
    done
    fatal "Lima VM '${VM_NAME}' did not reach Running state within ${WAIT_TIMEOUT} seconds."
}

destroy_vm() {
    warn "Destroying Lima VM '${VM_NAME}' to apply socket parity layout..."
    limactl stop "${VM_NAME}" >/dev/null 2>&1 || true
    limactl delete "${VM_NAME}" >/dev/null 2>&1 || true
}

current_layout_version() {
    limactl shell "${VM_NAME}" sudo -n cat "${LAYOUT_SENTINEL}" 2>/dev/null || true
}

ensure_vm_ready() {
    if vm_exists; then
        local status
        status="$(vm_status)"
        case "${status}" in
            Running)
                log "Lima VM '${VM_NAME}' already running."
                ;;
            *)
                warn "Lima VM '${VM_NAME}' status: ${status}; attempting to start."
                start_vm
                ;;
        esac
    else
        create_vm
    fi

    wait_for_running

    local layout
    layout="$(current_layout_version)"
    if [[ "${layout}" != "${LAYOUT_VERSION}" ]]; then
        warn "Lima VM layout (${layout:-missing}) does not match ${LAYOUT_VERSION}; rebuilding."
        destroy_vm
        create_vm
        wait_for_running
    fi
}

host_git_head() {
    if ! command -v git >/dev/null 2>&1; then
        return
    fi
    git -C "${PROJECT_PATH}" rev-parse HEAD 2>/dev/null || true
}

create_stage_manifest() {
    local manifest_path git_head
    manifest_path="$(mktemp)"
    git_head="$(host_git_head)"
    {
        printf 'project_path=%s\n' "${PROJECT_PATH}"
        printf 'project_basename=%s\n' "$(basename "${PROJECT_PATH}")"
        if [[ -n "${git_head}" ]]; then
            printf 'git_head=%s\n' "${git_head}"
        fi
    } > "${manifest_path}"
    printf '%s\n' "${manifest_path}"
}

stage_workspace() {
    local vm_user="$1"
    local workspace_name stage_parent manifest_path host_stage_root host_stage_workspace
    workspace_name="$(basename "${PROJECT_PATH}")"
    stage_parent="/tmp/substrate-stage-workspace"
    manifest_path="$(create_stage_manifest)"
    host_stage_root="$(mktemp -d)"
    host_stage_workspace="${host_stage_root}/${workspace_name}"

    log "Staging workspace input into guest-local path ${STAGED_WORKSPACE_CURRENT}"
    rsync -a \
        --exclude '.git' \
        --exclude 'target' \
        --exclude '.codex' \
        --exclude '.DS_Store' \
        "${PROJECT_PATH}/" "${host_stage_workspace}/"
    limactl shell "${VM_NAME}" env STAGE_PARENT="${stage_parent}" bash <<'EOF'
set -euo pipefail
rm -rf "${STAGE_PARENT}"
mkdir -p "${STAGE_PARENT}"
EOF
    limactl copy --recursive "${host_stage_workspace}" "${VM_NAME}:${stage_parent}/"
    limactl copy "${manifest_path}" "${VM_NAME}:${stage_parent}/${STAGED_WORKSPACE_MANIFEST_NAME}"
    limactl shell "${VM_NAME}" \
        env STAGE_PARENT="${stage_parent}" \
            WORKSPACE_NAME="${workspace_name}" \
            STAGED_WORKSPACE_ROOT="${STAGED_WORKSPACE_ROOT}" \
            STAGED_WORKSPACE_CURRENT="${STAGED_WORKSPACE_CURRENT}" \
            STAGED_WORKSPACE_MANIFEST_NAME="${STAGED_WORKSPACE_MANIFEST_NAME}" \
            VM_USER="${vm_user}" \
        bash <<'EOF'
set -euo pipefail
test -d "${STAGE_PARENT}/${WORKSPACE_NAME}"
test -f "${STAGE_PARENT}/${STAGED_WORKSPACE_MANIFEST_NAME}"
sudo install -d -o root -g substrate -m0750 "${STAGED_WORKSPACE_ROOT}"
sudo rm -rf "${STAGED_WORKSPACE_CURRENT}"
sudo mv "${STAGE_PARENT}/${WORKSPACE_NAME}" "${STAGED_WORKSPACE_CURRENT}"
sudo chown -R "${VM_USER}:substrate" "${STAGED_WORKSPACE_CURRENT}"
sudo chmod 0750 "${STAGED_WORKSPACE_CURRENT}"
sudo install -o "${VM_USER}" -g substrate -m0640 \
    "${STAGE_PARENT}/${STAGED_WORKSPACE_MANIFEST_NAME}" \
    "${STAGED_WORKSPACE_CURRENT}/${STAGED_WORKSPACE_MANIFEST_NAME}"
rm -rf "${STAGE_PARENT}"
EOF
    rm -rf "${host_stage_root}"
    rm -f "${manifest_path}"
}

verify_staged_workspace() {
    local expected_git_head expect_cargo_sources
    expected_git_head="$(host_git_head)"
    expect_cargo_sources=0
    if [[ -f "${PROJECT_PATH}/Cargo.toml" ]]; then
        expect_cargo_sources=1
    fi
    limactl shell "${VM_NAME}" sudo -n env \
        STAGED_WORKSPACE_CURRENT="${STAGED_WORKSPACE_CURRENT}" \
        STAGED_WORKSPACE_MANIFEST_NAME="${STAGED_WORKSPACE_MANIFEST_NAME}" \
        EXPECTED_PROJECT_PATH="${PROJECT_PATH}" \
        EXPECTED_GIT_HEAD="${expected_git_head}" \
        EXPECT_CARGO_SOURCES="${expect_cargo_sources}" \
        bash <<'EOF'
set -euo pipefail
manifest="${STAGED_WORKSPACE_CURRENT}/${STAGED_WORKSPACE_MANIFEST_NAME}"
test -d "${STAGED_WORKSPACE_CURRENT}"
test -f "${manifest}"
grep -Fqx "project_path=${EXPECTED_PROJECT_PATH}" "${manifest}"
if [[ -n "${EXPECTED_GIT_HEAD}" ]]; then
    grep -Fqx "git_head=${EXPECTED_GIT_HEAD}" "${manifest}"
fi
if [[ "${EXPECT_CARGO_SOURCES}" == "1" ]]; then
    test -f "${STAGED_WORKSPACE_CURRENT}/Cargo.toml"
fi
EOF
}

ensure_substrate_group() {
    local vm_user="$1"
    limactl shell "${VM_NAME}" bash <<EOF
set -euo pipefail
if ! getent group substrate >/dev/null 2>&1; then
    sudo groupadd --system substrate
fi
if id -nG "${vm_user}" | tr ' ' '\n' | grep -qx substrate; then
    exit 0
fi
sudo usermod -aG substrate "${vm_user}"
EOF
}

host_agent_candidate() {
    local base="${PROJECT_PATH}"
    local candidates=(
        "${base}/bin/linux/world-service"
        "${base}/bin/world-service-linux"
        "${base}/bin/world-service"
        "${base}/target/release/world-service"
        "${base}/target/debug/world-service"
    )
    local path
    for path in "${candidates[@]}"; do
        if [[ -f "${path}" ]]; then
            local file_type
            file_type="$(file -b "${path}" 2>/dev/null || true)"
            if echo "${file_type}" | grep -qi "ELF"; then
                printf '%s\n' "${path}"
                return 0
            fi
        fi
    done
    return 1
}

install_agent_from_host() {
    local agent_path="$1"
    log "Installing Linux world-service from ${agent_path}"
    limactl copy "${agent_path}" "${VM_NAME}:/tmp/world-service"
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0755 /tmp/world-service /usr/local/bin/substrate-world-service
sudo rm -f /tmp/world-service
EOF
}

host_cli_candidate() {
    local base="${PROJECT_PATH}"
    local candidates=(
        "${base}/bin/linux/substrate"
        "${base}/bin/substrate-linux"
        "${base}/bin/substrate"
        "${base}/target/aarch64-unknown-linux-gnu/${BUILD_PROFILE}/substrate"
        "${base}/target/x86_64-unknown-linux-gnu/${BUILD_PROFILE}/substrate"
        "${base}/target/${BUILD_PROFILE}/substrate"
    )
    local path
    for path in "${candidates[@]}"; do
        if [[ -f "${path}" ]]; then
            local file_type
            file_type="$(file -b "${path}" 2>/dev/null || true)"
            if echo "${file_type}" | grep -qi "ELF"; then
                printf '%s\n' "${path}"
                return 0
            fi
        fi
    done
    return 1
}

install_cli_from_host() {
    local cli_path="$1"
    log "Installing Linux substrate CLI from ${cli_path}"
    limactl copy "${cli_path}" "${VM_NAME}:/tmp/substrate-cli"
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0755 /tmp/substrate-cli /usr/local/bin/substrate
sudo tee /usr/local/bin/world >/dev/null <<'WORLD'
#!/usr/bin/env bash
exec substrate world "$@"
WORLD
sudo chmod 0755 /usr/local/bin/world
sudo rm -f /tmp/substrate-cli
EOF
}

host_gateway_candidate() {
    local base="${PROJECT_PATH}"
    local candidates=(
        "${base}/bin/linux/substrate-gateway"
        "${base}/bin/substrate-gateway-linux"
        "${base}/bin/substrate-gateway"
        "${base}/target/release/substrate-gateway"
        "${base}/target/debug/substrate-gateway"
    )
    local path
    for path in "${candidates[@]}"; do
        if [[ -f "${path}" ]]; then
            local file_type
            file_type="$(file -b "${path}" 2>/dev/null || true)"
            if echo "${file_type}" | grep -qi "ELF"; then
                printf '%s\n' "${path}"
                return 0
            fi
        fi
    done
    return 1
}

install_gateway_from_host() {
    local gateway_path="$1"
    log "Installing Linux substrate-gateway from ${gateway_path}"
    limactl copy "${gateway_path}" "${VM_NAME}:/tmp/substrate-gateway"
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0755 /tmp/substrate-gateway /usr/local/bin/substrate-gateway
sudo rm -f /tmp/substrate-gateway
EOF
}

build_missing_components_inside_vm() {
    local build_cli="${1:-0}"
    local build_agent="${2:-0}"
    local build_gateway="${3:-0}"

    if [[ "${build_cli}" -ne 1 && "${build_agent}" -ne 1 && "${build_gateway}" -ne 1 ]]; then
        return 0
    fi

    if [[ "${build_agent}" -eq 1 ]]; then
        log "Building Linux world-service inside Lima (profile: ${BUILD_PROFILE})"
    fi
    if [[ "${build_cli}" -eq 1 ]]; then
        log "Building Linux substrate CLI inside Lima for diagnostics (profile: ${BUILD_PROFILE})"
    fi
    if [[ "${build_gateway}" -eq 1 ]]; then
        log "Building Linux substrate-gateway inside Lima (profile: ${BUILD_PROFILE})"
    fi

    if [[ ! -f "${PROJECT_PATH}/Cargo.toml" ]]; then
        if [[ "${build_agent}" -eq 1 ]]; then
            fatal "Linux world-service missing and ${PROJECT_PATH} does not contain Cargo sources. Provide bin/linux/world-service or rerun from a source checkout."
        fi
        if [[ "${build_gateway}" -eq 1 ]]; then
            fatal "Linux substrate-gateway missing and ${PROJECT_PATH} does not contain Cargo sources. Provide bin/linux/substrate-gateway or rerun from a source checkout."
        fi
        warn "Skipping guest CLI build; ${PROJECT_PATH} lacks Cargo sources."
        # The guest CLI is optional (diagnostics only) and release bundles don't ship Cargo sources.
        # Do not abort provisioning when only the guest CLI is missing.
        return 0
    fi

    local status=0
    if limactl shell "${VM_NAME}" env BUILD_PROFILE="${BUILD_PROFILE}" BUILD_GUEST_CLI="${build_cli}" BUILD_GUEST_AGENT="${build_agent}" BUILD_GUEST_GATEWAY="${build_gateway}" STAGED_WORKSPACE_PATH="${STAGED_WORKSPACE_CURRENT}" bash <<'EOF'
set -euo pipefail
build_cli="${BUILD_GUEST_CLI:-0}"
build_agent="${BUILD_GUEST_AGENT:-0}"
build_gateway="${BUILD_GUEST_GATEWAY:-0}"

fix_dns() {
    local probe_host="${1:-ports.ubuntu.com}"
    if getent hosts "${probe_host}" >/dev/null 2>&1; then
        return 0
    fi
    echo "[lima-warm] DNS resolution failed inside Lima for ${probe_host}; applying fallback resolv.conf (1.1.1.1 / 8.8.8.8)..." >&2
    local SUDO_CMD="sudo"
    if sudo -n true 2>/dev/null; then
        SUDO_CMD="sudo -n"
    fi
    $SUDO_CMD sh -c "printf 'nameserver 1.1.1.1\nnameserver 8.8.8.8\n' > /etc/resolv.conf" || true
    $SUDO_CMD systemctl restart dnsmasq 2>/dev/null || true
    $SUDO_CMD systemctl restart systemd-resolved 2>/dev/null || true
    getent hosts "${probe_host}" >/dev/null 2>&1
}

ensure_cargo() {
    # Cargo.lock v4 requires a newer cargo than Ubuntu 24.04's apt cargo on some images.
    # Prefer rustup when we detect a v4 lockfile so we don't fail during `cargo build --locked`.
    local needs_lockfile_v4=0
    if sudo -u "$(id -un)" -g substrate test -f "${STAGED_WORKSPACE_PATH}/Cargo.lock" \
        && sudo -u "$(id -un)" -g substrate grep -qx 'version = 4' "${STAGED_WORKSPACE_PATH}/Cargo.lock" 2>/dev/null; then
        needs_lockfile_v4=1
    fi

    if command -v rustup >/dev/null 2>&1; then
        if [ -f "$HOME/.cargo/env" ]; then
            # shellcheck disable=SC1090
            source "$HOME/.cargo/env"
        fi
        export PATH="$HOME/.cargo/bin:$PATH"
        rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
        rustup default stable >/dev/null 2>&1 || true
        if command -v cargo >/dev/null 2>&1; then
            return 0
        fi
    fi

    if [[ "${needs_lockfile_v4}" -eq 1 ]]; then
        echo "[lima-warm] Cargo.lock v4 detected; installing rustup toolchain (stable)..." >&2
        fix_dns ports.ubuntu.com || true
        if curl -4 --connect-timeout 10 --retry 3 --retry-delay 1 --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal; then
            # shellcheck disable=SC1090
            source "$HOME/.cargo/env"
            export PATH="$HOME/.cargo/bin:$PATH"
            rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
            rustup default stable >/dev/null 2>&1 || true
            command -v cargo >/dev/null 2>&1
            return $?
        fi
        return 1
    fi

    if command -v cargo >/dev/null 2>&1; then
        return 0
    fi
    echo "[lima-warm] cargo not found inside Lima VM; attempting apt install (rustc cargo)..." >&2
    local SUDO="sudo"
    if sudo -n true 2>/dev/null; then
        SUDO="sudo -n"
    fi
    fix_dns ports.ubuntu.com || true
    if $SUDO apt-get update && $SUDO apt-get install -y rustc cargo; then
        return 0
    fi
    echo "[lima-warm] apt install failed; trying rustup via curl (IPv4, retries)..." >&2
    fix_dns ports.ubuntu.com || true
    if curl -4 --connect-timeout 10 --retry 3 --retry-delay 1 --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal; then
        # shellcheck disable=SC1090
        source "$HOME/.cargo/env"
        return 0
    fi
    return 1
}

if [[ "${build_cli}" != "1" && "${build_agent}" != "1" && "${build_gateway}" != "1" ]]; then
    exit 0
fi

if ! ensure_cargo; then
    echo "[lima-warm][ERROR] unable to install cargo inside Lima VM; install Rust manually or provide Linux binaries." >&2
    exit 1
fi
if command -v rustup >/dev/null 2>&1; then
    rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
    rustup default stable >/dev/null 2>&1 || true
fi
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
fi
cargo_bin="$(command -v cargo || true)"
if [[ -z "${cargo_bin}" ]]; then
    echo "[lima-warm][ERROR] cargo still missing after toolchain installation." >&2
    exit 1
fi
BUILD_DIR="/tmp/substrate-lima-build"
BUILD_OUTPUT_DIR="${BUILD_PROFILE}"
BUILD_PROFILE_FLAG=()
case "${BUILD_PROFILE}" in
debug)
    BUILD_OUTPUT_DIR="debug"
    ;;
release)
    BUILD_OUTPUT_DIR="release"
    BUILD_PROFILE_FLAG=(--profile release)
    ;;
*)
    BUILD_PROFILE_FLAG=(--profile "${BUILD_PROFILE}")
    ;;
esac
mkdir -p "${BUILD_DIR}"
workspace_dir="${STAGED_WORKSPACE_PATH}"
if ! sudo -u "$(id -un)" -g substrate test -r "${workspace_dir}/Cargo.toml"; then
    echo "[lima-warm][ERROR] staged workspace is not readable under the substrate group: ${workspace_dir}" >&2
    exit 1
fi
run_guest_cargo_build() {
    fix_dns static.crates.io || true
    if sudo -u "$(id -un)" -g substrate env \
        HOME="$HOME" \
        PATH="$PATH" \
        CARGO_TARGET_DIR="${BUILD_DIR}" \
        bash -lc 'cd "$1" && shift && exec "$@"' bash \
        "${workspace_dir}" "${cargo_bin}" build "$@"; then
        return 0
    fi
    fix_dns static.crates.io || true
    sudo -u "$(id -un)" -g substrate env \
        HOME="$HOME" \
        PATH="$PATH" \
        CARGO_TARGET_DIR="${BUILD_DIR}" \
        bash -lc 'cd "$1" && shift && exec "$@"' bash \
        "${workspace_dir}" "${cargo_bin}" build "$@"
}
mandatory_build_failed=0
cli_build_failed=0
if [[ "${build_agent}" == "1" ]]; then
    if ! run_guest_cargo_build -p world-service "${BUILD_PROFILE_FLAG[@]}" --locked; then
        echo "[lima-warm][ERROR] failed to build Linux world-service inside Lima." >&2
        mandatory_build_failed=1
    else
        sudo install -Dm0755 "${BUILD_DIR}/${BUILD_OUTPUT_DIR}/world-service" /usr/local/bin/substrate-world-service
    fi
fi
if [[ "${build_gateway}" == "1" ]]; then
    if ! run_guest_cargo_build -p substrate-gateway "${BUILD_PROFILE_FLAG[@]}" --locked; then
        echo "[lima-warm][ERROR] failed to build Linux substrate-gateway inside Lima." >&2
        mandatory_build_failed=1
    else
        sudo install -Dm0755 "${BUILD_DIR}/${BUILD_OUTPUT_DIR}/substrate-gateway" /usr/local/bin/substrate-gateway
    fi
fi
if [[ "${build_cli}" == "1" ]]; then
    if ! run_guest_cargo_build --bin substrate "${BUILD_PROFILE_FLAG[@]}" --locked; then
        echo "[lima-warm][WARN] failed to build the optional Linux substrate CLI inside Lima; continuing because diagnostics can fall back to the host CLI." >&2
        cli_build_failed=1
    else
        sudo install -Dm0755 "${BUILD_DIR}/${BUILD_OUTPUT_DIR}/substrate" /usr/local/bin/substrate
        sudo tee /usr/local/bin/world >/dev/null <<'WORLD'
#!/usr/bin/env bash
exec substrate world "$@"
WORLD
        sudo chmod 0755 /usr/local/bin/world
    fi
fi
rm -rf "${BUILD_DIR}" || true
if [[ "${mandatory_build_failed}" -ne 0 ]]; then
    exit 1
fi
if [[ "${cli_build_failed}" -ne 0 ]]; then
    echo "[lima-warm][WARN] Guest provisioning completed without a Linux substrate CLI binary." >&2
fi
EOF
    then
        status=0
    else
        status=$?
    fi
    if [[ "${status}" -ne 0 ]]; then
        if [[ "${build_agent}" -eq 1 && "${build_gateway}" -eq 1 ]]; then
            fatal "Failed to build mandatory Linux guest binaries inside Lima (world-service and/or substrate-gateway) (exit ${status}). Provide prebuilt binaries under bin/linux/ or rerun from a source checkout."
        fi
        if [[ "${build_agent}" -eq 1 ]]; then
            fatal "Failed to build Linux world-service inside Lima (exit ${status}). Provide a prebuilt agent under bin/linux/world-service or rerun from a source checkout."
        fi
        if [[ "${build_gateway}" -eq 1 ]]; then
            fatal "Failed to build Linux substrate-gateway inside Lima (exit ${status}). Provide a prebuilt gateway under bin/linux/substrate-gateway or rerun from a source checkout."
        fi
        warn "Failed to build Linux CLI inside Lima; diagnostics requiring a guest CLI will need to run on the host."
        return 1
    fi
}

install_guest_binaries() {
    local cli_candidate agent_candidate gateway_candidate
    local need_cli_build=0
    local need_agent_build=0
    local need_gateway_build=0

    cli_candidate="$(host_cli_candidate)" || true
    if [[ -n "${cli_candidate:-}" ]]; then
        install_cli_from_host "${cli_candidate}"
    else
        if [[ -f "${PROJECT_PATH}/Cargo.toml" ]]; then
            log "Linux substrate CLI not found in ${PROJECT_PATH}; attempting in-guest build for diagnostics."
            need_cli_build=1
        else
            warn "Linux substrate CLI not found in ${PROJECT_PATH}; skipping guest CLI install (no Cargo sources in bundle). Diagnostics will fall back to host CLI."
        fi
    fi

    agent_candidate="$(host_agent_candidate)" || true
    if [[ -n "${agent_candidate:-}" ]]; then
        install_agent_from_host "${agent_candidate}"
    else
        log "Linux world-service binary not found or invalid in ${PROJECT_PATH}; falling back to an in-guest build."
        need_agent_build=1
    fi

    gateway_candidate="$(host_gateway_candidate)" || true
    if [[ -n "${gateway_candidate:-}" ]]; then
        install_gateway_from_host "${gateway_candidate}"
    else
        log "Linux substrate-gateway binary not found or invalid in ${PROJECT_PATH}; falling back to an in-guest build."
        need_gateway_build=1
    fi

    if [[ "${need_cli_build}" -eq 1 || "${need_agent_build}" -eq 1 || "${need_gateway_build}" -eq 1 ]]; then
        if [[ "${SKIP_GUEST_BUILD}" -eq 1 ]]; then
            if [[ "${need_agent_build}" -eq 1 ]]; then
                warn "Linux world-service missing but SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1; skipping guest build. Ensure another step installs /usr/local/bin/substrate-world-service."
            fi
            if [[ "${need_gateway_build}" -eq 1 ]]; then
                warn "Linux substrate-gateway missing but SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1; skipping guest build. Ensure another step installs /usr/local/bin/substrate-gateway."
            fi
            if [[ "${need_cli_build}" -eq 1 ]]; then
                warn "Linux CLI missing but SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1; skipping guest build. Diagnostics will fall back to host CLI."
            fi
            return 0
        fi
        build_missing_components_inside_vm "${need_cli_build}" "${need_agent_build}" "${need_gateway_build}"
    fi
}

verify_guest_binaries() {
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
test -x /usr/local/bin/substrate-world-service
test -x /usr/local/bin/substrate-gateway
EOF
}

write_systemd_units() {
    local guest_substrate_home="$1"
    local enable_netfilter="${SUBSTRATE_WORLD_NETFILTER_ENABLE:-0}"
    local service_template="${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl"
    local socket_template="${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket"
    local rendered_units_dir
    local netfilter_env=""

    [[ -f "${service_template}" ]] || fatal "Missing canonical service unit source: ${service_template}"
    [[ -f "${socket_template}" ]] || fatal "Missing canonical socket unit source: ${socket_template}"

    case "${enable_netfilter}" in
        1|true|yes|TRUE|YES)
            log "Installing canonical guest systemd units with WORLD_NETFILTER_ENABLE=1"
            netfilter_env="Environment=WORLD_NETFILTER_ENABLE=1"
            ;;
        *)
            log "Installing canonical guest systemd units without WORLD_NETFILTER_ENABLE=1"
            ;;
    esac
    rendered_units_dir="$(mktemp -d)"
    SUBSTRATE_GUEST_HOME="${guest_substrate_home}" WORLD_NETFILTER_ENV="${netfilter_env}" \
        envsubst < "${service_template}" > "${rendered_units_dir}/substrate-world-service.service"
    envsubst < "${socket_template}" > "${rendered_units_dir}/substrate-world-service.socket"

    limactl copy "${rendered_units_dir}/substrate-world-service.service" \
        "${VM_NAME}:/tmp/substrate-world-service.service"
    limactl copy "${rendered_units_dir}/substrate-world-service.socket" \
        "${VM_NAME}:/tmp/substrate-world-service.socket"
    rm -rf "${rendered_units_dir}"

    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0644 /tmp/substrate-world-service.service /etc/systemd/system/substrate-world-service.service
sudo install -Dm0644 /tmp/substrate-world-service.socket /etc/systemd/system/substrate-world-service.socket
sudo rm -f /tmp/substrate-world-service.service /tmp/substrate-world-service.socket
EOF
}

enable_socket_activation() {
    local guest_substrate_home="$1"
    limactl shell "${VM_NAME}" env SUBSTRATE_GUEST_HOME="${guest_substrate_home}" bash <<'EOF'
set -euo pipefail
legacy_unit_prefix="substrate-world"
legacy_service="${legacy_unit_prefix}-agent.service"
legacy_socket="${legacy_unit_prefix}-agent.socket"
sudo install -d -m0755 "${SUBSTRATE_GUEST_HOME}"
sudo install -d -m0750 -o root -g substrate /var/lib/substrate
sudo install -d -m0750 -o root -g substrate /run/substrate
sudo install -d -m0750 -o root -g substrate /run/substrate/substrate-gateway-runtime
sudo systemctl stop "${legacy_service}" "${legacy_socket}" >/dev/null 2>&1 || true
sudo systemctl disable "${legacy_service}" "${legacy_socket}" >/dev/null 2>&1 || true
sudo rm -f "/etc/systemd/system/${legacy_service}" "/etc/systemd/system/${legacy_socket}"
sudo systemctl daemon-reload
sudo systemctl enable substrate-world-service.service >/dev/null
sudo systemctl enable substrate-world-service.socket >/dev/null
sudo systemctl stop substrate-world-service.service >/dev/null 2>&1 || true
sudo systemctl stop substrate-world-service.socket >/dev/null 2>&1 || true
sudo install -d -m0750 -o root -g substrate /run/substrate
sudo install -d -m0750 -o root -g substrate /run/substrate/substrate-gateway-runtime
sudo rm -f /run/substrate.sock
sudo systemctl start substrate-world-service.socket
sudo systemctl start substrate-world-service.service
EOF
}

socket_summary() {
    local meta
    meta="$(limactl shell "${VM_NAME}" sudo stat -c '%U:%G %a' /run/substrate.sock 2>/dev/null || true)"
    if [[ -n "${meta}" ]]; then
        log "Agent socket perms: ${meta} (expected root:substrate 660)"
        if [[ "${meta}" != "root:substrate 660" ]]; then
            warn "Socket permissions differ from expected root:substrate 660."
        fi
    else
        warn "Unable to read /run/substrate.sock metadata."
    fi
}

write_layout_sentinel() {
    limactl shell "${VM_NAME}" bash <<EOF
set -euo pipefail
echo "${LAYOUT_VERSION}" | sudo tee "${LAYOUT_SENTINEL}" >/dev/null
EOF
}

linger_guidance() {
    local vm_user="$1"
    local linger
    linger="$(limactl shell "${VM_NAME}" sudo -n loginctl show-user "${vm_user}" -p Linger 2>/dev/null | cut -d= -f2 || true)"
    if [[ "${linger}" != "yes" ]]; then
        warn "loginctl lingering for ${vm_user} is ${linger:-unknown}. Run 'limactl shell ${VM_NAME} sudo loginctl enable-linger ${vm_user}' so socket activation survives logout."
    else
        log "loginctl lingering already enabled for ${vm_user}."
    fi
}

configure_guest() {
    local vm_user
    local vm_home
    local guest_substrate_home
    vm_user="$(limactl shell "${VM_NAME}" id -un | tr -d '\r')"
    if [[ -z "${vm_user}" ]]; then
        fatal "Unable to determine Lima guest user."
    fi
    vm_home="$(limactl shell "${VM_NAME}" getent passwd "${vm_user}" | cut -d: -f6 | tr -d '\r')"
    if [[ -z "${vm_home}" ]]; then
        vm_home="/home/${vm_user}"
    fi
    guest_substrate_home="${vm_home}/.substrate"
    ensure_substrate_group "${vm_user}"
    stage_workspace "${vm_user}"
    verify_staged_workspace
    install_guest_binaries
    verify_guest_binaries
    write_systemd_units "${guest_substrate_home}"
    enable_socket_activation "${guest_substrate_home}"
    socket_summary
    write_layout_sentinel
    linger_guidance "${vm_user}"
}

if [[ ${CHECK_ONLY} -eq 1 ]]; then
    check_only_status
fi

check_host_prereqs
render_profile
ensure_vm_ready
configure_guest

log "Lima world backend '${VM_NAME}' is ready. Verify with: limactl shell ${VM_NAME} sudo systemctl status substrate-world-service.socket"
log "Optional orchestration parity proof: scripts/mac/orchestration-smoke.sh"
