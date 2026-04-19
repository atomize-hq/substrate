#!/usr/bin/env bash
# Provision the Substrate world-agent systemd service + socket on a Linux host.
#
# This script builds the world-agent and substrate-gateway binaries (unless
# --skip-build is set), installs them under /usr/local/bin, writes both the
# .service and .socket unit files, and enables/starts the listener so
# /run/substrate.sock is owned by root. Pass --dry-run to print the actions
# without invoking sudo.
#
# Usage: scripts/linux/world-provision.sh [--profile <name>] [--skip-build] [--dry-run]

set -euo pipefail

PROFILE=release
SKIP_BUILD=0
DRY_RUN=0
SUDO_NONINTERACTIVE=0
ENABLE_WORLD_NETFILTER=0
SUBSTRATE_GROUP="substrate"
SOCKET_FS_PATH="/run/substrate.sock"
INVOKING_USER=""
INVOKING_HOME=""
SUBSTRATE_CLI_BIN_PATH=""

show_cmd() {
    printf '[dry-run]'
    for arg in "$@"; do
        printf ' %q' "$arg"
    done
    printf '\n'
}

run_cmd() {
    if [[ ${DRY_RUN} -eq 1 ]]; then
        show_cmd "$@"
    else
        "$@"
    fi
}

sudo_cmd() {
    if [[ ${SUDO_NONINTERACTIVE} -eq 1 ]]; then
        run_cmd sudo -n "$@"
    else
        run_cmd sudo "$@"
    fi
}

detect_invoking_user() {
    if [[ -n "${SUDO_USER:-}" && "${SUDO_USER}" != "root" ]]; then
        printf '%s\n' "${SUDO_USER}"
        return
    fi
    local current
    current=$(id -un 2>/dev/null || true)
    if [[ -n "${current}" ]]; then
        printf '%s\n' "${current}"
    fi
}

resolve_substrate_cli() {
    local candidate=""
    if [[ -n "${SUBSTRATE_CLI_BIN_PATH}" && -x "${SUBSTRATE_CLI_BIN_PATH}" ]]; then
        printf '%s\n' "${SUBSTRATE_CLI_BIN_PATH}"
        return 0
    fi
    candidate="$(command -v substrate 2>/dev/null || true)"
    if [[ -n "${candidate}" ]]; then
        printf '%s\n' "${candidate}"
        return 0
    fi
    return 1
}

prepare_gateway_smoke_auth() {
    local auth_path="${INVOKING_HOME}/.codex/auth.json"
    if [[ -f "${auth_path}" ]]; then
        printf '%s\n' "present"
        return 0
    fi

    local tmp
    tmp="$(mktemp)"
    cat >"${tmp}" <<'JSON'
{
  "account_id": "acct_smoke",
  "access_token": "header.payload.signature"
}
JSON
    install -d -m0700 "${INVOKING_HOME}/.codex"
    install -m0600 "${tmp}" "${auth_path}"
    rm -f "${tmp}"
    printf '%s\n' "created"
}

cleanup_gateway_smoke_auth() {
    rm -f "${INVOKING_HOME}/.codex/auth.json"
}

run_gateway_lifecycle_proof() {
    local substrate_cli="$1"
    local auth_state=""
    local cleanup_auth=0
    local status_json=""
    local base_url=""
    local port=""
    local health_json=""

    echo "==> Running gateway lifecycle proof"
    auth_state="$(prepare_gateway_smoke_auth)"
    if [[ "${auth_state}" == "created" ]]; then
        cleanup_auth=1
    fi
    trap 'if [[ ${cleanup_auth} -eq 1 ]]; then cleanup_gateway_smoke_auth; fi' RETURN

    (
        cd "${REPO_ROOT}"
        "${substrate_cli}" world gateway sync
        status_json="$("${substrate_cli}" world gateway status --json)"
        if [[ "${status_json}" != *'"status":"available"'* ]]; then
            echo "gateway status did not report available: ${status_json}" >&2
            exit 1
        fi

        "${substrate_cli}" world gateway restart
        status_json="$("${substrate_cli}" world gateway status --json)"
        if [[ "${status_json}" != *'"status":"available"'* ]]; then
            echo "gateway status after restart did not report available: ${status_json}" >&2
            exit 1
        fi
        base_url="$(printf '%s\n' "${status_json}" | sed -n 's/.*"openai_base_url":"\([^"]*\)".*/\1/p')"
        port="$(printf '%s\n' "${base_url}" | sed -n 's#http://127\.0\.0\.1:\([0-9][0-9]*\)$#\1#p')"
        if [[ -z "${port}" ]]; then
            echo "Unable to derive gateway port from ${base_url}" >&2
            exit 1
        fi
        health_json="$(curl --fail --silent "http://127.0.0.1:${port}/health")"
        if [[ "${health_json}" != *'"status":"ok"'* || "${health_json}" != *'"service":"substrate-gateway"'* ]]; then
            echo "gateway health probe returned unexpected payload: ${health_json}" >&2
            exit 1
        fi
    )

}

user_in_group() {
    local user="$1"
    local group="$2"
    local groups
    groups="$(id -nG "${user}" 2>/dev/null || true)"
    [[ " ${groups} " == *" ${group} "* ]]
}

ensure_substrate_group_exists() {
    if getent group "${SUBSTRATE_GROUP}" >/dev/null 2>&1; then
        echo "==> ${SUBSTRATE_GROUP} group already exists."
        return
    fi
    echo "==> Creating ${SUBSTRATE_GROUP} group (sudo may prompt)"
    if sudo_cmd groupadd --system "${SUBSTRATE_GROUP}"; then
        echo "    Created ${SUBSTRATE_GROUP} group."
    else
        echo "ERROR: Unable to create the ${SUBSTRATE_GROUP} group. Run 'sudo groupadd --system ${SUBSTRATE_GROUP}' manually and rerun this script." >&2
        exit 1
    fi
}

ensure_user_in_group() {
    local user="$1"
    if [[ -z "${user}" || "${user}" == "root" ]]; then
        cat <<'MSG'
WARNING: Unable to detect a non-root user to add to the substrate group.
Run 'sudo usermod -aG substrate <user>' manually so shells can access /run/substrate.sock.
MSG
        return
    fi
    if ! id "${user}" >/dev/null 2>&1; then
        cat <<MSG
WARNING: Unable to look up user '${user}'. Ensure the intended operator belongs to the '${SUBSTRATE_GROUP}' group before relying on socket activation:
  sudo usermod -aG ${SUBSTRATE_GROUP} <user>
MSG
        return
    fi
    if user_in_group "${user}" "${SUBSTRATE_GROUP}"; then
        echo "==> ${user} already belongs to ${SUBSTRATE_GROUP}."
        return
    fi
    echo "==> Adding ${user} to ${SUBSTRATE_GROUP} (sudo may prompt)"
    if sudo_cmd usermod -aG "${SUBSTRATE_GROUP}" "${user}"; then
        echo "    Added ${user} to ${SUBSTRATE_GROUP}. Log out/in or run 'newgrp ${SUBSTRATE_GROUP}' to refresh group membership."
    else
        cat <<MSG
WARNING: Failed to add ${user} to ${SUBSTRATE_GROUP}.
Run 'sudo usermod -aG ${SUBSTRATE_GROUP} ${user}' manually and re-login so /run/substrate.sock is accessible without sudo.
MSG
    fi
}

print_linger_guidance() {
    local user="$1"
    echo "==> Lingering guidance"
    if [[ -z "${user}" || "${user}" == "root" ]]; then
        cat <<'MSG'
loginctl enable-linger <user> ensures socket-activated services remain available after logout or reboot.
Run 'sudo loginctl enable-linger <user>' for the operator account once you know which user should keep the socket alive.
MSG
        return
    fi
    if ! command -v loginctl >/dev/null 2>&1; then
        cat <<MSG
loginctl not found. On systemd hosts run the following once so socket activation survives logout:
  sudo loginctl enable-linger ${user}
MSG
        return
    fi
    local linger_state
    linger_state="$(loginctl show-user "${user}" -p Linger 2>/dev/null | cut -d= -f2 || true)"
    if [[ "${linger_state}" == "yes" ]]; then
        echo "    loginctl reports lingering already enabled for ${user}."
    else
        cat <<MSG
    loginctl reports lingering=${linger_state:-unknown} for ${user}.
    Run 'sudo loginctl enable-linger ${user}' so the socket stays available after logout/reboot.
MSG
    fi
}

install_unit() {
    local destination="$1"
    local content="$2"
    if [[ ${DRY_RUN} -eq 1 ]]; then
        printf '[dry-run] install unit %s\n' "${destination}"
        return
    fi
    local tmp
    tmp=$(mktemp)
    printf '%s\n' "${content}" >"${tmp}"
    sudo_cmd install -Dm0644 "${tmp}" "${destination}"
    rm -f "${tmp}"
}

usage() {
    cat <<'USAGE'
Usage: scripts/linux/world-provision.sh [options]

Options:
  --profile <name>   Cargo profile to build (default: release)
  --skip-build       Assume target/<profile>/world-agent already exists
  --dry-run          Print the provisioning steps without executing them
  --world-netfilter  Enable Linux nftables egress scoping (sets WORLD_NETFILTER_ENABLE=1 for substrate-world-agent.service)
  --sudo-noninteractive  Use sudo -n (fail fast if password required)
  -h, --help         Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --world-netfilter)
            ENABLE_WORLD_NETFILTER=1
            shift
            ;;
        --sudo-noninteractive)
            SUDO_NONINTERACTIVE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
 done

if [[ ${EUID} -eq 0 ]]; then
    echo "Please run this script without sudo; it will invoke sudo when needed." >&2
    exit 1
fi

INVOKING_USER="$(detect_invoking_user)"

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)
WORLD_AGENT_BIN_PATH="${REPO_ROOT}/target/${PROFILE}/world-agent"
GATEWAY_BIN_PATH="${REPO_ROOT}/target/${PROFILE}/substrate-gateway"
SUBSTRATE_CLI_BIN_PATH="${REPO_ROOT}/target/${PROFILE}/substrate"

if [[ ${SKIP_BUILD} -eq 0 ]]; then
    echo "==> Building substrate + world-agent + substrate-gateway (profile: ${PROFILE})"
    if [[ "${PROFILE}" == "release" ]]; then
        WORLD_AGENT_BIN_PATH="${REPO_ROOT}/target/release/world-agent"
        GATEWAY_BIN_PATH="${REPO_ROOT}/target/release/substrate-gateway"
        SUBSTRATE_CLI_BIN_PATH="${REPO_ROOT}/target/release/substrate"
        if [[ ${DRY_RUN} -eq 1 ]]; then
            show_cmd cargo build -p substrate --bin substrate -p world-agent -p substrate-gateway --release --manifest-path "${REPO_ROOT}/Cargo.toml"
        else
            cargo build -p substrate --bin substrate -p world-agent -p substrate-gateway --release --manifest-path "${REPO_ROOT}/Cargo.toml"
        fi
    else
        if [[ ${DRY_RUN} -eq 1 ]]; then
            show_cmd cargo build -p substrate --bin substrate -p world-agent -p substrate-gateway --profile "${PROFILE}" --manifest-path "${REPO_ROOT}/Cargo.toml"
        else
            cargo build -p substrate --bin substrate -p world-agent -p substrate-gateway --profile "${PROFILE}" --manifest-path "${REPO_ROOT}/Cargo.toml"
        fi
    fi
else
    echo "==> Skipping build as requested"
fi

if [[ ${DRY_RUN} -eq 0 && ! -x "${WORLD_AGENT_BIN_PATH}" ]]; then
    echo "world-agent binary not found at ${WORLD_AGENT_BIN_PATH}. Did the build succeed?" >&2
    exit 1
fi

if [[ ${DRY_RUN} -eq 0 && ! -x "${GATEWAY_BIN_PATH}" ]]; then
    echo "substrate-gateway binary not found at ${GATEWAY_BIN_PATH}. Did the build succeed?" >&2
    exit 1
fi

echo "==> Ensuring ${SUBSTRATE_GROUP} group and membership"
ensure_substrate_group_exists
ensure_user_in_group "${INVOKING_USER}"

SERVICE_PATH="/etc/systemd/system/substrate-world-agent.service"
SOCKET_PATH="/etc/systemd/system/substrate-world-agent.socket"

SUBSTRATE_HOME_FOR_AGENT="${SUBSTRATE_HOME:-}"
if [[ -z "${SUBSTRATE_HOME_FOR_AGENT}" ]]; then
    INVOKING_HOME="$(getent passwd "${INVOKING_USER}" 2>/dev/null | cut -d: -f6 || true)"
    if [[ -z "${INVOKING_HOME}" ]]; then
        INVOKING_HOME="/home/${INVOKING_USER}"
    fi
    SUBSTRATE_HOME_FOR_AGENT="${INVOKING_HOME}/.substrate"
fi
if [[ -z "${INVOKING_HOME}" ]]; then
    INVOKING_HOME="$(dirname "${SUBSTRATE_HOME_FOR_AGENT}")"
fi

NETFILTER_ENV_LINE=""
if [[ "${ENABLE_WORLD_NETFILTER}" -eq 1 ]]; then
    NETFILTER_ENV_LINE="Environment=WORLD_NETFILTER_ENABLE=1"
fi

read -r -d '' SERVICE_UNIT_CONTENT <<UNIT || true
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
Environment=SUBSTRATE_HOME=${SUBSTRATE_HOME_FOR_AGENT}
${NETFILTER_ENV_LINE}
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
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE

[Install]
WantedBy=multi-user.target
UNIT

read -r -d '' SOCKET_UNIT_CONTENT <<'UNIT' || true
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

echo "==> Installing world-agent to /usr/local/bin (sudo will prompt if needed)"
sudo_cmd install -Dm0755 "${WORLD_AGENT_BIN_PATH}" /usr/local/bin/substrate-world-agent
echo "==> Installing substrate-gateway to /usr/local/bin (no dedicated service)"
sudo_cmd install -Dm0755 "${GATEWAY_BIN_PATH}" /usr/local/bin/substrate-gateway

echo "==> Ensuring runtime directories exist"
sudo_cmd install -d -m0750 /run/substrate
sudo_cmd install -d -m0750 /var/lib/substrate

echo "==> Writing systemd units to ${SERVICE_PATH} and ${SOCKET_PATH}"
install_unit "${SERVICE_PATH}" "${SERVICE_UNIT_CONTENT}"
install_unit "${SOCKET_PATH}" "${SOCKET_UNIT_CONTENT}"

echo "==> Reloading systemd and enabling socket activation"
sudo_cmd systemctl daemon-reload
sudo_cmd systemctl enable substrate-world-agent.service
sudo_cmd systemctl enable substrate-world-agent.socket

echo "==> Restarting socket/service to enforce ${SOCKET_FS_PATH} ownership"
sudo_cmd systemctl stop substrate-world-agent.service
sudo_cmd systemctl stop substrate-world-agent.socket
sudo_cmd rm -f "${SOCKET_FS_PATH}"
sudo_cmd systemctl start substrate-world-agent.socket
sudo_cmd systemctl start substrate-world-agent.service

echo "==> ${SOCKET_FS_PATH} listing (should be root:${SUBSTRATE_GROUP} 0660)"
sudo_cmd ls -l "${SOCKET_FS_PATH}"
echo "==> Installed gateway binary"
sudo_cmd ls -l /usr/local/bin/substrate-gateway

echo "==> substrate-world-agent.socket status (last 10 log lines)"
sudo_cmd systemctl status substrate-world-agent.socket --no-pager --lines=10 || true
echo "==> substrate-world-agent.service status (last 10 log lines)"
sudo_cmd systemctl status substrate-world-agent.service --no-pager --lines=10 || true

if [[ ${DRY_RUN} -eq 1 ]]; then
    substrate_cli="$(resolve_substrate_cli 2>/dev/null || true)"
    if [[ -z "${substrate_cli}" ]]; then
        substrate_cli="${SUBSTRATE_CLI_BIN_PATH}"
    fi
    echo "==> Gateway lifecycle proof"
    show_cmd "${substrate_cli}" world gateway sync
    show_cmd "${substrate_cli}" world gateway status --json
    show_cmd "${substrate_cli}" world gateway restart
    echo "[dry-run] curl --fail --silent http://127.0.0.1:<gateway-port>/health"
else
    substrate_cli="$(resolve_substrate_cli)"
    run_gateway_lifecycle_proof "${substrate_cli}"
fi

print_linger_guidance "${INVOKING_USER}"

echo "==> Provisioning complete"
echo "    Verify socket with: sudo ls -l ${SOCKET_FS_PATH}"
echo "    Probe capabilities: sudo curl --unix-socket ${SOCKET_FS_PATH} http://localhost/v1/capabilities"
echo "    Verify gateway lifecycle: $(basename "${substrate_cli:-substrate}") world gateway status --json"
echo "    Doctor socket block: substrate host doctor --json | jq '.host.world_socket'"
echo "    Shim summary: substrate --shim-status | grep 'World socket'"
