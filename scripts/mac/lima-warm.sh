#!/usr/bin/env bash
set -euo pipefail

VM_NAME="${LIMA_VM_NAME:-substrate}"
PROFILE="${LIMA_PROFILE_PATH:-scripts/mac/lima/substrate.yaml}"
PROJECT_PATH=""
CHECK_ONLY=0
BUILD_PROFILE="${LIMA_BUILD_PROFILE:-release}"
LAYOUT_SENTINEL="/etc/substrate-lima-layout"
LAYOUT_VERSION="socket-parity-v1"
WAIT_TIMEOUT=120

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
  <project-path>    Repository or release path to mount inside the Lima VM (default: current directory)
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

ensure_repo_mount() {
    if ! limactl shell "${VM_NAME}" test -d /src >/dev/null 2>&1; then
        fatal "Repository path ${PROJECT_PATH} is not mounted inside the VM. Ensure it exists and rerun."
    fi
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
        "${base}/bin/linux/world-agent"
        "${base}/bin/world-agent-linux"
        "${base}/bin/world-agent"
        "${base}/target/release/world-agent"
        "${base}/target/debug/world-agent"
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
    log "Installing Linux world-agent from ${agent_path}"
    limactl copy "${agent_path}" "${VM_NAME}:/tmp/world-agent"
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0755 /tmp/world-agent /usr/local/bin/substrate-world-agent
sudo rm -f /tmp/world-agent
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

build_missing_components_inside_vm() {
    local build_cli="${1:-0}"
    local build_agent="${2:-0}"

    if [[ "${build_cli}" -ne 1 && "${build_agent}" -ne 1 ]]; then
        return 0
    fi

    if [[ "${build_agent}" -eq 1 ]]; then
        log "Building Linux world-agent inside Lima (profile: ${BUILD_PROFILE})"
    fi
    if [[ "${build_cli}" -eq 1 ]]; then
        log "Building Linux substrate CLI inside Lima for diagnostics (profile: ${BUILD_PROFILE})"
    fi

    if [[ ! -f "${PROJECT_PATH}/Cargo.toml" ]]; then
        if [[ "${build_agent}" -eq 1 ]]; then
            fatal "Linux world-agent missing and ${PROJECT_PATH} does not contain Cargo sources. Provide bin/linux/world-agent or rerun from a source checkout."
        fi
        warn "Skipping guest CLI build; ${PROJECT_PATH} lacks Cargo sources."
        return 1
    fi

    if ! limactl shell "${VM_NAME}" env BUILD_PROFILE="${BUILD_PROFILE}" BUILD_GUEST_CLI="${build_cli}" BUILD_GUEST_AGENT="${build_agent}" bash <<'EOF'; then
set -euo pipefail
build_cli="${BUILD_GUEST_CLI:-0}"
build_agent="${BUILD_GUEST_AGENT:-0}"

ensure_cargo() {
    if command -v cargo >/dev/null 2>&1; then
        return 0
    fi
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get update
        sudo DEBIAN_FRONTEND=noninteractive apt-get install -y rustc cargo >/dev/null
    fi
    if command -v cargo >/dev/null 2>&1; then
        return 0
    fi
    if curl -4 --connect-timeout 10 --retry 3 --retry-delay 1 --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal >/dev/null 2>&1; then
        :
    else
        return 1
    fi
    return 0
}

if [[ "${build_cli}" != "1" && "${build_agent}" != "1" ]]; then
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
mkdir -p "${BUILD_DIR}"
cd /src
if [[ "${build_cli}" == "1" ]]; then
    CARGO_TARGET_DIR="${BUILD_DIR}" "${cargo_bin}" build --bin substrate --profile "${BUILD_PROFILE}" --locked
    sudo install -Dm0755 "${BUILD_DIR}/${BUILD_PROFILE}/substrate" /usr/local/bin/substrate
    sudo tee /usr/local/bin/world >/dev/null <<'WORLD'
#!/usr/bin/env bash
exec substrate world "$@"
WORLD
    sudo chmod 0755 /usr/local/bin/world
fi
if [[ "${build_agent}" == "1" ]]; then
    CARGO_TARGET_DIR="${BUILD_DIR}" "${cargo_bin}" build -p world-agent --profile "${BUILD_PROFILE}" --locked
    sudo install -Dm0755 "${BUILD_DIR}/${BUILD_PROFILE}/world-agent" /usr/local/bin/substrate-world-agent
fi
rm -rf "${BUILD_DIR}"
EOF
        local status=$?
        if [[ "${build_agent}" -eq 1 ]]; then
            fatal "Failed to build Linux world-agent inside Lima (exit ${status}). Provide a prebuilt agent under bin/linux/world-agent or rerun from a source checkout."
        fi
        warn "Failed to build Linux CLI inside Lima; diagnostics requiring a guest CLI will need to run on the host."
        return 1
    fi
}

install_guest_binaries() {
    local cli_candidate agent_candidate
    local need_cli_build=0
    local need_agent_build=0

    cli_candidate="$(host_cli_candidate)" || true
    if [[ -n "${cli_candidate:-}" ]]; then
        install_cli_from_host "${cli_candidate}"
    else
        log "Linux substrate CLI not found in ${PROJECT_PATH}; attempting in-guest build for diagnostics."
        need_cli_build=1
    fi

    agent_candidate="$(host_agent_candidate)" || true
    if [[ -n "${agent_candidate:-}" ]]; then
        install_agent_from_host "${agent_candidate}"
    else
        log "Linux world-agent binary not found or invalid in ${PROJECT_PATH}; falling back to an in-guest build."
        need_agent_build=1
    fi

    if [[ "${need_cli_build}" -eq 1 || "${need_agent_build}" -eq 1 ]]; then
        build_missing_components_inside_vm "${need_cli_build}" "${need_agent_build}"
    fi
}

write_systemd_units() {
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
cat <<'UNIT' | sudo tee /etc/systemd/system/substrate-world-agent.service >/dev/null
[Unit]
Description=Substrate World Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/substrate-world-agent
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=SUBSTRATE_AGENT_TCP_PORT=61337
Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock
RuntimeDirectory=substrate
RuntimeDirectoryMode=0750
StateDirectory=substrate
StateDirectoryMode=0750
WorkingDirectory=/var/lib/substrate
StandardOutput=journal
StandardError=journal
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE

[Install]
WantedBy=multi-user.target
UNIT

cat <<'UNIT' | sudo tee /etc/systemd/system/substrate-world-agent.socket >/dev/null
[Unit]
Description=Substrate World Agent Socket
PartOf=substrate-world-agent.service

[Socket]
ListenStream=/run/substrate.sock
SocketMode=0660
SocketUser=root
SocketGroup=substrate
DirectoryMode=0750
RemoveOnStop=yes
Service=substrate-world-agent.service

[Install]
WantedBy=sockets.target
UNIT
EOF
}

enable_socket_activation() {
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -d -m0750 /var/lib/substrate
sudo install -d -m0750 /run/substrate
sudo systemctl daemon-reload
sudo systemctl enable substrate-world-agent.service >/dev/null
sudo systemctl enable substrate-world-agent.socket >/dev/null
sudo systemctl stop substrate-world-agent.service >/dev/null 2>&1 || true
sudo systemctl stop substrate-world-agent.socket >/dev/null 2>&1 || true
sudo rm -f /run/substrate.sock
sudo systemctl start substrate-world-agent.socket
sudo systemctl start substrate-world-agent.service
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
    ensure_repo_mount
    local vm_user
    vm_user="$(limactl shell "${VM_NAME}" id -un | tr -d '\r')"
    if [[ -z "${vm_user}" ]]; then
        fatal "Unable to determine Lima guest user."
    fi
    ensure_substrate_group "${vm_user}"
    install_guest_binaries
    write_systemd_units
    enable_socket_activation
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

log "Lima world backend '${VM_NAME}' is ready. Verify with: limactl shell ${VM_NAME} sudo systemctl status substrate-world-agent.socket"
