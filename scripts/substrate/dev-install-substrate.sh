#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-install-substrate"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Substrate Dev Installer

Build Substrate from the current repository and wire development shims to the
freshly built binaries. This is intended for local iteration after removing any
production installation.

Usage:
  dev-install-substrate.sh [--prefix <path>] [--profile <debug|release>] [--version-label <name>] [--no-world] [--anchor-mode <mode>] [--anchor-path <path>] [--caged|--uncaged] [--no-shims]
  dev-install-substrate.sh --help

Options:
  --prefix <path>           Installation prefix for shims/env helper (default: ~/.substrate)
  --profile <name>          Cargo profile to build (debug or release; default: debug)
  --version-label <name>    Version directory label under <prefix>/versions (default: dev)
  --no-world                Mark install metadata as world_disabled (skips provisioning entirely)
  --anchor-mode <mode>      Default anchor mode (project|follow-cwd|custom; default: project) [alias: --world-root-mode]
  --anchor-path <path>      Default anchor path (for custom mode; alias: --world-root-path)
  --caged                   Write caged=true to install metadata (default)
  --uncaged                 Write caged=false to install metadata
  --no-shims                Skip shim deployment (only run cargo build)
  --help                    Show this message
USAGE
}

run_privileged() {
  if [[ ${EUID} -eq 0 ]]; then
    "$@"
    return $?
  fi
  if command -v sudo >/dev/null 2>&1; then
    sudo "$@"
    return $?
  fi
  warn "Command requires elevated privileges but sudo is unavailable: $*"
  return 1
}

detect_invoking_user() {
  if [[ -n "${SUDO_USER:-}" ]]; then
    printf '%s\n' "${SUDO_USER}"
    return
  fi
  if [[ -n "${USER:-}" ]]; then
    printf '%s\n' "${USER}"
    return
  fi
  if command -v id >/dev/null 2>&1; then
    id -un 2>/dev/null || true
    return
  fi
  printf ''
}

user_in_group() {
  local target_user="$1"
  local target_group="$2"
  if [[ -z "${target_user}" || -z "${target_group}" ]]; then
    return 1
  fi
  if id -nG "${target_user}" 2>/dev/null | tr ' ' '\n' | grep -qx "${target_group}"; then
    return 0
  fi
  return 1
}

record_group_existence() {
  if [[ "${IS_LINUX}" -ne 1 || "${WORLD_ENABLED}" -ne 1 ]]; then
    return
  fi
  if [[ -n "${HOST_STATE_GROUP_EXISTED}" ]]; then
    return
  fi
  if command -v getent >/dev/null 2>&1; then
    if getent group substrate >/dev/null 2>&1; then
      HOST_STATE_GROUP_EXISTED="true"
    else
      HOST_STATE_GROUP_EXISTED="false"
    fi
  else
    HOST_STATE_GROUP_EXISTED="unknown"
  fi
}

record_group_created() {
  HOST_STATE_GROUP_CREATED=1
}

record_user_added() {
  local user="$1"
  if [[ -z "${user}" ]]; then
    return
  fi
  for existing in "${HOST_STATE_ADDED_USERS[@]:-}"; do
    if [[ "${existing}" == "${user}" ]]; then
      return
    fi
  done
  HOST_STATE_ADDED_USERS+=("${user}")
}

record_linger_state() {
  local user="$1"
  local state="$2"
  local enabled="${3:-0}"
  if [[ -z "${user}" ]]; then
    return
  fi
  local updated=0
  for idx in "${!HOST_STATE_LINGER_ENTRIES[@]}"; do
    IFS=':' read -r existing_user existing_state existing_enabled <<<"${HOST_STATE_LINGER_ENTRIES[$idx]}"
    if [[ "${existing_user}" == "${user}" ]]; then
      local new_state="${existing_state:-unknown}"
      if [[ -n "${state}" ]]; then
        new_state="${state}"
      fi
      local new_enabled="${existing_enabled:-0}"
      if [[ "${enabled}" -eq 1 ]]; then
        new_enabled="1"
      fi
      HOST_STATE_LINGER_ENTRIES[idx]="${user}:${new_state}:${new_enabled}"
      updated=1
      break
    fi
  done
  if [[ "${updated}" -eq 0 ]]; then
    local normalized_state="${state:-unknown}"
    local normalized_enabled=0
    if [[ "${enabled}" -eq 1 ]]; then
      normalized_enabled=1
    fi
    HOST_STATE_LINGER_ENTRIES+=("${user}:${normalized_state}:${normalized_enabled}")
  fi
}

find_linux_world_agent() {
  local root="$1"
  local target_dir="$2"
  local candidates=(
    "${root}/bin/linux/world-agent"
    "${root}/bin/world-agent-linux"
    "${root}/bin/world-agent"
    "${root}/target/x86_64-unknown-linux-gnu/${target_dir}/world-agent"
    "${root}/target/aarch64-unknown-linux-gnu/${target_dir}/world-agent"
    "${root}/target/${target_dir}/world-agent"
  )
  for candidate in "${candidates[@]}"; do
    if [[ -x "${candidate}" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done
  return 1
}

write_host_state_metadata() {
  if [[ "${IS_LINUX}" -ne 1 || "${WORLD_ENABLED}" -ne 1 ]]; then
    return
  fi
  if [[ -z "${HOST_STATE_PATH}" ]]; then
    return
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    warn "python3 not found; skipping host state metadata recording (${HOST_STATE_PATH})."
    return
  fi

  local events=()
  if [[ -n "${HOST_STATE_GROUP_EXISTED}" ]]; then
    events+=("group_preexisting:${HOST_STATE_GROUP_EXISTED}")
  fi
  if [[ "${HOST_STATE_GROUP_CREATED}" -eq 1 ]]; then
    events+=("group_created:true")
  fi
  for user in "${HOST_STATE_ADDED_USERS[@]:-}"; do
    events+=("user_added:${user}")
  done
  for entry in "${HOST_STATE_LINGER_ENTRIES[@]:-}"; do
    events+=("linger:${entry}")
  done

  if [[ ${#events[@]} -eq 0 ]]; then
    log "No host state changes detected; skipping host metadata write."
    return
  fi

  local event_payload
  event_payload="$(printf '%s\n' "${events[@]}")"
  mkdir -p "$(dirname "${HOST_STATE_PATH}")" || true
  local tmp="${HOST_STATE_PATH}.tmp"
  if ! STATE_EVENTS="${event_payload}" python3 - "${HOST_STATE_PATH}" > "${tmp}" <<'PY'
import datetime
import json
import os
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
events = [line.strip() for line in os.environ.get("STATE_EVENTS", "").splitlines() if line.strip()]
schema_version = 1
timestamp = datetime.datetime.utcnow().isoformat() + "Z"

base = {}
if path.exists():
    try:
        with path.open() as f:
            base = json.load(f)
    except Exception as exc:  # noqa: BLE001
        sys.stderr.write(f"[dev-install-substrate] warning: unable to parse {path}: {exc}\n")
        base = {}

if base.get("schema_version") != schema_version:
    base = {}

base["schema_version"] = schema_version
base.setdefault("created_at", timestamp)
base["updated_at"] = timestamp

host = base.setdefault("host_state", {})
group = host.setdefault("group", {"name": "substrate", "members_added": []})
group.setdefault("name", "substrate")
members = {m for m in group.get("members_added", []) if isinstance(m, str)}
linger = host.setdefault("linger", {})
linger_users = linger.setdefault("users", {})


def parse_bool(raw: str):
    lowered = raw.lower()
    if lowered in ("true", "1", "yes"):
        return True
    if lowered in ("false", "0", "no"):
        return False
    return None


for raw_event in events:
    parts = raw_event.split(":", 3)
    if not parts:
        continue
    kind = parts[0]
    if kind == "group_preexisting" and len(parts) >= 2:
        val = parse_bool(parts[1])
        if val is not None:
            group["existed_before"] = val
        elif "existed_before" not in group:
            group["existed_before"] = None
    elif kind == "group_created" and len(parts) >= 2:
        val = parse_bool(parts[1])
        if val is not None:
            group["created_by_installer"] = val
    elif kind == "user_added" and len(parts) >= 2:
        user = parts[1]
        if user:
            members.add(user)
    elif kind == "linger" and len(parts) >= 4:
        user, state, enabled_flag = parts[1], parts[2], parts[3]
        if not user:
            continue
        entry = linger_users.setdefault(user, {})
        if state:
            entry.setdefault("state_at_install", state)
            entry["state_at_install"] = state
        enabled_val = parse_bool(enabled_flag)
        if enabled_val is not None:
            entry["enabled_by_substrate"] = enabled_val
        elif "enabled_by_substrate" not in entry:
            entry["enabled_by_substrate"] = False

group["members_added"] = sorted(members)
json.dump(base, sys.stdout, indent=2, sort_keys=True)
PY
  then
    warn "Failed to write host state metadata to ${HOST_STATE_PATH}; continuing without blocking install."
    rm -f "${tmp}" || true
    return
  fi

  mv "${tmp}" "${HOST_STATE_PATH}"
  chmod 0644 "${HOST_STATE_PATH}" || true
  log "Host state metadata recorded at ${HOST_STATE_PATH}"
}

ensure_substrate_group_membership() {
  if [[ "${IS_LINUX}" -ne 1 || "${WORLD_ENABLED}" -ne 1 ]]; then
    return
  fi
  record_group_existence
  local target_group="substrate"
  if ! getent group "${target_group}" >/dev/null 2>&1; then
    log "Creating '${target_group}' group (sudo may prompt)..."
    if run_privileged groupadd --system "${target_group}"; then
      log "Created ${target_group} group."
      record_group_created
    else
      warn "Unable to create ${target_group} group automatically. Run 'sudo groupadd --system ${target_group}' and re-run the installer."
      return
    fi
  fi

  local invoking_user
  invoking_user="$(detect_invoking_user)"
  if [[ -z "${invoking_user}" || "${invoking_user}" == "root" ]]; then
    warn "Could not determine the non-root user that should join the '${target_group}' group. Run 'sudo usermod -aG ${target_group} <user>' before retrying if socket access is required."
    return
  fi

  if user_in_group "${invoking_user}" "${target_group}"; then
    log "${invoking_user} already belongs to ${target_group}."
    return
  fi

  log "Adding ${invoking_user} to ${target_group} (sudo may prompt)..."
  if run_privileged usermod -aG "${target_group}" "${invoking_user}"; then
    warn "${invoking_user} added to ${target_group}. Log out/in or run 'newgrp ${target_group}' so shells notice the new membership."
    record_user_added "${invoking_user}"
  else
    warn "Failed to add ${invoking_user} to ${target_group}; run 'sudo usermod -aG ${target_group} ${invoking_user}' manually."
  fi
}

ensure_socket_group_alignment() {
  if [[ "${IS_LINUX}" -ne 1 || "${WORLD_ENABLED}" -ne 1 ]]; then
    return
  fi
  if ! command -v systemctl >/dev/null 2>&1; then
    warn "systemctl not found; verify /run/substrate.sock is root:substrate 0660 after provisioning."
    return
  fi
  local socket_unit="/etc/systemd/system/substrate-world-agent.socket"
  if [[ ! -f "${socket_unit}" ]]; then
    warn "Socket unit missing at ${socket_unit}; rerun scripts/linux/world-provision.sh to install it."
    return
  fi
  if grep -q '^SocketGroup=substrate' "${socket_unit}"; then
    log "substrate-world-agent.socket already sets SocketGroup=substrate."
  else
    log "Updating ${socket_unit} to enforce SocketGroup=substrate (sudo may prompt)..."
    if ! run_privileged sed -i 's/^SocketGroup=.*/SocketGroup=substrate/' "${socket_unit}"; then
      warn "Failed to update ${socket_unit}; edit it manually so SocketGroup=substrate and rerun 'sudo systemctl daemon-reload'."
      return
    fi
  fi

  log "Restarting world-agent units to apply socket ownership (sudo may prompt)..."
  run_privileged systemctl stop substrate-world-agent.service substrate-world-agent.socket || true
  run_privileged rm -f /run/substrate.sock || true
  run_privileged systemctl daemon-reload || true
  run_privileged systemctl start substrate-world-agent.socket || true
  run_privileged systemctl start substrate-world-agent.service || true
  log "Reloaded socket/service units so /run/substrate.sock is recreated as root:substrate 0660."
}

ensure_world_enable_helper_bridge() {
  local target_root="$1"
  local scripts_root="$2"
  local dest_dir="${target_root%/}/scripts/substrate"
  local -a helper_files=("world-enable.sh" "install-substrate.sh")
  mkdir -p "${dest_dir}"
  for helper in "${helper_files[@]}"; do
    local src="${scripts_root%/}/${helper}"
    local dest="${dest_dir}/${helper}"
    if [[ -f "${src}" ]]; then
      ln -sfn "${src}" "${dest}"
      log "Linked ${helper} helper into ${dest}"
    else
      warn "${helper} helper missing at ${src}; CLI world enable path may fail."
    fi
  done
}

ensure_release_bin_bridge() {
  local target_root="$1"
  local profile_dir="$2"
  local src_root="${target_root%/}/${profile_dir}"
  local dest_bin="${target_root%/}/bin"
  mkdir -p "${dest_bin}" "${dest_bin}/linux"
  local -a binaries=("substrate" "substrate-shim" "substrate-forwarder" "host-proxy" "world-agent")
  for binary in "${binaries[@]}"; do
    local src="${src_root}/${binary}"
    local dest="${dest_bin}/${binary}"
    if [[ -x "${src}" ]]; then
      ln -sfn "${src}" "${dest}"
      if [[ "${binary}" == "world-agent" ]]; then
        ln -sfn "${src}" "${dest_bin}/linux/world-agent"
        ln -sfn "${src}" "${dest_bin}/world-agent-linux"
      fi
    fi
    local src_exe="${src}.exe"
    if [[ -x "${src_exe}" ]]; then
      ln -sfn "${src_exe}" "${dest}.exe"
    fi
  done
}

print_linger_guidance() {
  if [[ "${IS_LINUX}" -ne 1 || "${WORLD_ENABLED}" -ne 1 ]]; then
    return
  fi
  local invoking_user
  invoking_user="$(detect_invoking_user)"
  if [[ -z "${invoking_user}" || "${invoking_user}" == "root" ]]; then
    cat <<'MSG'
[dev-install-substrate] loginctl: Unable to detect a non-root user for lingering.
Enable lingering manually so socket-activated services stay available after logout:
  loginctl enable-linger <user>
MSG
    record_linger_state "${invoking_user}" "unknown" 0
    return
  fi

  if ! command -v loginctl >/dev/null 2>&1; then
    cat <<MSG
[dev-install-substrate] loginctl not found. To keep the socket-activated world-agent alive
across logouts/reboots, run this on a systemd host once:
  loginctl enable-linger ${invoking_user}
MSG
    record_linger_state "${invoking_user}" "unknown" 0
    return
  fi

  local linger_state
  linger_state="$(loginctl show-user "${invoking_user}" -p Linger 2>/dev/null | cut -d= -f2 || true)"
  record_linger_state "${invoking_user}" "${linger_state:-unknown}" 0
  if [[ "${linger_state}" == "yes" ]]; then
    log "loginctl reports lingering already enabled for ${invoking_user}."
  else
    cat <<MSG
[dev-install-substrate] loginctl status for ${invoking_user}: ${linger_state:-unknown}
Enable lingering to let systemd launch the socket after reboot/logout:
  loginctl enable-linger ${invoking_user}
MSG
  fi
}

PREFIX="${HOME}/.substrate"
PROFILE="debug"
DEPLOY_SHIMS=1
WORLD_ENABLED=1
ANCHOR_MODE="project"
ANCHOR_PATH=""
WORLD_CAGED=1
VERSION_LABEL="dev"
IS_LINUX=0
IS_MAC=0
HOST_STATE_PATH=""
HOST_STATE_GROUP_EXISTED=""
HOST_STATE_GROUP_CREATED=0
HOST_STATE_ADDED_USERS=()
HOST_STATE_LINGER_ENTRIES=()
if [[ "$(uname -s)" == "Linux" ]]; then
  IS_LINUX=1
fi
if [[ "$(uname -s)" == "Darwin" ]]; then
  IS_MAC=1
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      [[ $# -ge 2 ]] || fatal "--prefix requires a value"
      PREFIX="$2"
      shift 2
      ;;
    --profile)
      [[ $# -ge 2 ]] || fatal "--profile requires a value"
      PROFILE="$2"
      shift 2
      ;;
    --version-label)
      [[ $# -ge 2 ]] || fatal "--version-label requires a value"
      VERSION_LABEL="$2"
      shift 2
      ;;
    --no-world)
      WORLD_ENABLED=0
      shift
      ;;
    --anchor-mode|--world-root-mode)
      [[ $# -ge 2 ]] || fatal "--anchor-mode requires a value"
      ANCHOR_MODE="$2"
      shift 2
      ;;
    --anchor-path|--world-root-path)
      [[ $# -ge 2 ]] || fatal "--anchor-path requires a value"
      ANCHOR_PATH="$2"
      shift 2
      ;;
    --caged)
      WORLD_CAGED=1
      shift
      ;;
    --uncaged)
      WORLD_CAGED=0
      shift
      ;;
    --no-shims)
      DEPLOY_SHIMS=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fatal "Unknown argument: $1"
      ;;
  esac
done

HOST_STATE_PATH="${PREFIX%/}/install_state.json"

case "${PROFILE}" in
  debug|release) ;;
  *) fatal "Unsupported profile '${PROFILE}'. Use 'debug' or 'release'." ;;
esac

case "${ANCHOR_MODE}" in
  project|follow-cwd|custom) ;;
  *) fatal "Unsupported anchor mode '${ANCHOR_MODE}'. Use project, follow-cwd, or custom." ;;
esac

if [[ "${ANCHOR_MODE}" == "custom" && -z "${ANCHOR_PATH}" ]]; then
  fatal "--anchor-path is required when --anchor-mode=custom"
fi

if ! command -v cargo >/dev/null 2>&1; then
  fatal "cargo not found on PATH. Install the Rust toolchain before running this script."
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

TARGET_DIR="${PROFILE}"
BUILD_FLAGS=(build --bin substrate --bin substrate-shim)
if [[ "${PROFILE}" == "release" ]]; then
  BUILD_FLAGS+=(--release)
fi

log "Building Substrate (${PROFILE})..."
cargo "${BUILD_FLAGS[@]}"

if [[ "${WORLD_ENABLED}" -eq 1 && "${IS_LINUX}" -eq 1 ]]; then
  log "Building world-agent (${PROFILE})..."
  if [[ "${PROFILE}" == "release" ]]; then
    cargo build -p world-agent --release
  else
    cargo build -p world-agent
  fi
fi

SUBSTRATE_BIN="${REPO_ROOT}/target/${TARGET_DIR}/substrate"
if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
  fatal "Expected substrate binary at ${SUBSTRATE_BIN}, but it was not found."
fi

BIN_DIR="${PREFIX%/}/bin"
SHIMS_DIR="${PREFIX%/}/shims"
ENV_FILE="${PREFIX%/}/dev-shim-env.sh"
VERSION_DIR="${PREFIX%/}/versions/${VERSION_LABEL}"
VERSION_CONFIG_DIR="${VERSION_DIR}/config"
MANAGER_INIT_PATH="${PREFIX%/}/manager_init.sh"
MANAGER_ENV_PATH="${PREFIX%/}/manager_env.sh"
INSTALL_CONFIG_PATH="${PREFIX%/}/config.toml"

mkdir -p "${PREFIX}" "${BIN_DIR}" "${VERSION_CONFIG_DIR}"

# Stage config assets to mirror the production bundle layout.
if [[ -d "${REPO_ROOT}/config" ]]; then
  cp -R "${REPO_ROOT}/config/." "${VERSION_CONFIG_DIR}/"
fi
if [[ -f "${REPO_ROOT}/scripts/substrate/world-deps.yaml" ]]; then
  cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" "${VERSION_CONFIG_DIR}/world-deps.yaml"
fi
if [[ ! -f "${VERSION_CONFIG_DIR}/manager_hooks.yaml" ]]; then
  fatal "manager manifest missing (expected ${VERSION_CONFIG_DIR}/manager_hooks.yaml)"
fi
if [[ ! -f "${VERSION_CONFIG_DIR}/world-deps.yaml" ]]; then
  fatal "world-deps manifest missing (expected ${VERSION_CONFIG_DIR}/world-deps.yaml)"
fi

# Write install metadata (install + world tables) like the production installer.
cat > "${INSTALL_CONFIG_PATH}.tmp" <<EOF
[install]
world_enabled = $([[ "${WORLD_ENABLED}" -eq 1 ]] && echo "true" || echo "false")

[world]
anchor_mode = "${ANCHOR_MODE}"
anchor_path = "${ANCHOR_PATH}"
root_mode = "${ANCHOR_MODE}"
root_path = "${ANCHOR_PATH}"
caged = $([[ "${WORLD_CAGED}" -eq 1 ]] && echo "true" || echo "false")
EOF
mv "${INSTALL_CONFIG_PATH}.tmp" "${INSTALL_CONFIG_PATH}"
chmod 0644 "${INSTALL_CONFIG_PATH}" || true

# Write manager init placeholder + env exporter.
cat > "${MANAGER_INIT_PATH}.tmp" <<'EOF'
#!/usr/bin/env bash
# Managed by dev-install-substrate

# Place per-manager snippets here if you need them for debugging.
EOF
mv "${MANAGER_INIT_PATH}.tmp" "${MANAGER_INIT_PATH}"
chmod 0644 "${MANAGER_INIT_PATH}" || true

today="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
manager_env_literal="$(printf '%q' "${MANAGER_ENV_PATH}")"
manager_init_literal="$(printf '%q' "${MANAGER_INIT_PATH}")"
cat > "${MANAGER_ENV_PATH}.tmp" <<EOF
#!/usr/bin/env bash
# Managed by ${SCRIPT_NAME} on ${today}
export SUBSTRATE_WORLD=$([[ "${WORLD_ENABLED}" -eq 1 ]] && echo "enabled" || echo "disabled")
export SUBSTRATE_WORLD_ENABLED=$([[ "${WORLD_ENABLED}" -eq 1 ]] && echo "1" || echo "0")
export SUBSTRATE_CAGED=$([[ "${WORLD_CAGED}" -eq 1 ]] && echo "1" || echo "0")
export SUBSTRATE_ANCHOR_MODE="${ANCHOR_MODE}"
export SUBSTRATE_ANCHOR_PATH="${ANCHOR_PATH}"
export SUBSTRATE_WORLD_ROOT_MODE="${ANCHOR_MODE}"
export SUBSTRATE_WORLD_ROOT_PATH="${ANCHOR_PATH}"
export SUBSTRATE_MANAGER_ENV=${manager_env_literal}
export SUBSTRATE_MANAGER_INIT=${manager_init_literal}

manager_init_path=${manager_init_literal}
if [[ -f "\${manager_init_path}" ]]; then
  # shellcheck disable=SC1090
  source "\${manager_init_path}"
fi
EOF
mv "${MANAGER_ENV_PATH}.tmp" "${MANAGER_ENV_PATH}"
chmod 0644 "${MANAGER_ENV_PATH}" || true

shim_note=""
if [[ ${DEPLOY_SHIMS} -eq 1 ]]; then
  log "Deploying shims via ${SUBSTRATE_BIN}"
  if ! SHIM_ORIGINAL_PATH="${PATH}" SUBSTRATE_ROOT="${PREFIX}" "${SUBSTRATE_BIN}" --shim-deploy; then
    fatal "Shim deployment failed"
  fi
  shim_note="Dev shims deployed to ${SHIMS_DIR}."
else
  warn "Shim deployment skipped (--no-shims)."
  shim_note="Shims were not deployed (--no-shims). Binaries are available under ${BIN_DIR}."
fi

for binary in substrate substrate-shim substrate-forwarder host-proxy world-agent; do
  src="${REPO_ROOT}/target/${TARGET_DIR}/${binary}"
  if [[ -x "${src}" ]]; then
    ln -sfn "${src}" "${BIN_DIR}/${binary}"
  elif [[ -x "${src}.exe" ]]; then
    ln -sfn "${src}.exe" "${BIN_DIR}/${binary}.exe"
  fi
done

# Provide substrate-world-agent alias so CLI discovery works without extra config.
world_agent_src="${REPO_ROOT}/target/${TARGET_DIR}/world-agent"
if [[ -x "${world_agent_src}" ]]; then
  ln -sfn "${world_agent_src}" "${BIN_DIR}/substrate-world-agent"
elif [[ -x "${world_agent_src}.exe" ]]; then
  ln -sfn "${world_agent_src}.exe" "${BIN_DIR}/substrate-world-agent.exe"
fi

if [[ -d "${REPO_ROOT}/target" ]]; then
  version_root="$(cd "${REPO_ROOT}/target" && pwd)"
  ensure_world_enable_helper_bridge "${version_root}" "${REPO_ROOT}/scripts/substrate"
  ensure_release_bin_bridge "${version_root}" "${TARGET_DIR}"
fi

if [[ "${WORLD_ENABLED}" -eq 1 && "${IS_LINUX}" -eq 1 ]]; then
  ensure_substrate_group_membership
  PROVISION_SCRIPT="${REPO_ROOT}/scripts/linux/world-provision.sh"
  if [[ -x "${PROVISION_SCRIPT}" ]]; then
    log "Provisioning Linux world-agent service via ${PROVISION_SCRIPT} (sudo may prompt)..."
    if ! "${PROVISION_SCRIPT}" --profile "${PROFILE}" --skip-build; then
      warn "world-provision script reported an error; rerun ${PROVISION_SCRIPT} manually to enable the world-agent service."
    fi
  else
    warn "Linux world-provision script missing at ${PROVISION_SCRIPT}; world-agent service not configured."
  fi
  ensure_socket_group_alignment
elif [[ "${WORLD_ENABLED}" -eq 1 && "${IS_MAC}" -eq 1 ]]; then
  log "Provisioning macOS Lima world-agent service..."
  if ! command -v limactl >/dev/null 2>&1; then
    fatal "limactl not found; install Lima or rerun with --no-world to skip macOS world provisioning."
  fi
  LIMA_WARM="${REPO_ROOT}/scripts/mac/lima-warm.sh"
  if [[ ! -x "${LIMA_WARM}" ]]; then
    fatal "Expected Lima warm helper at ${LIMA_WARM}"
  fi
  (cd "${REPO_ROOT}" && SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1 "${LIMA_WARM}" "${REPO_ROOT}")

  build_flag=""
  target_dir="debug"
  if [[ "${PROFILE}" == "release" ]]; then
    build_flag="--release"
    target_dir="release"
  fi

  linux_agent="$(find_linux_world_agent "${REPO_ROOT}" "${TARGET_DIR}")" || true
  need_build_agent=0
  if [[ -z "${linux_agent:-}" ]]; then
    log "Linux world-agent binary not found under ${REPO_ROOT}/target/${TARGET_DIR}; building inside Lima."
    need_build_agent=1
  else
    file_type="$(file -b "${linux_agent}" 2>/dev/null || true)"
    if ! echo "${file_type}" | grep -q "ELF"; then
      log "Host world-agent candidate is not a Linux ELF; building inside Lima instead..."
      linux_agent=""
      need_build_agent=1
    else
      log "Linux world-agent install source: ${linux_agent}"
    fi
  fi

  lima_target_dir="/tmp/substrate-dev-target"
  log "Building Linux substrate inside Lima (target=${target_dir}; agent_build=${need_build_agent})..."
  if ! limactl shell substrate env BUILD_FLAG="${build_flag}" TARGET_DIR="${target_dir}" BUILD_AGENT="${need_build_agent}" CARGO_TARGET_DIR="${lima_target_dir}" bash <<'LIMA_BUILD_AGENT'; then
set -euo pipefail

ensure_cargo() {
  if command -v cargo >/dev/null 2>&1; then
    return 0
  fi
  fix_dns() {
    if getent hosts ports.ubuntu.com >/dev/null 2>&1; then
      return 0
    fi
    echo "[dev-install-substrate] DNS resolution failed in Lima; applying fallback resolv.conf (1.1.1.1 / 8.8.8.8)..." >&2
    local SUDO_CMD="sudo"
    if sudo -n true 2>/dev/null; then
      SUDO_CMD="sudo -n"
    fi
    $SUDO_CMD sh -c "printf 'nameserver 1.1.1.1\nnameserver 8.8.8.8\n' > /etc/resolv.conf" || true
    $SUDO_CMD systemctl restart dnsmasq 2>/dev/null || true
    $SUDO_CMD systemctl restart systemd-resolved 2>/dev/null || true
    getent hosts ports.ubuntu.com >/dev/null 2>&1
  }

  echo "[dev-install-substrate] cargo not found inside Lima VM; attempting apt install (rustc cargo)..." >&2
  local SUDO="sudo"
  if sudo -n true 2>/dev/null; then
    SUDO="sudo -n"
  fi
  fix_dns || true
  if $SUDO apt-get update && $SUDO apt-get install -y rustc cargo; then
    return 0
  fi
  echo "[dev-install-substrate] apt install failed; trying rustup via curl (IPv4, retries)..." >&2
  fix_dns || true
  if curl -4 --connect-timeout 10 --retry 3 --retry-delay 1 --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
    return 0
  fi
  return 1
}

if ! ensure_cargo; then
  echo "[dev-install-substrate][ERROR] Unable to install cargo inside Lima VM; install Rust manually (apt/rustup) or rerun with --no-world." >&2
  exit 1
fi
# Prefer a modern toolchain via rustup to satisfy lockfile version requirements.
if ! command -v rustup >/dev/null 2>&1; then
  echo "[dev-install-substrate] Installing rustup (stable toolchain)..." >&2
  if curl -4 --connect-timeout 10 --retry 3 --retry-delay 1 --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal; then
    :
  else
    echo "[dev-install-substrate][WARN] rustup installation failed; falling back to system cargo (may be too old)" >&2
  fi
fi
# shellcheck disable=SC1090
source "$HOME/.cargo/env" 2>/dev/null || true
cargo_cmd="$(command -v cargo || true)"
if command -v rustup >/dev/null 2>&1; then
  rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
  rustup default stable >/dev/null 2>&1 || true
  cargo_cmd="$HOME/.cargo/bin/cargo"
fi
if [[ -z "${cargo_cmd}" ]]; then
  echo "[dev-install-substrate][ERROR] cargo still unavailable after toolchain setup." >&2
  exit 1
fi
cd /src
"${cargo_cmd}" build --bin substrate ${BUILD_FLAG}
if [[ "${BUILD_AGENT}" == "1" ]]; then
  "${cargo_cmd}" build -p world-agent ${BUILD_FLAG}
fi
LIMA_BUILD_AGENT
    fatal "Failed to build Linux binaries inside Lima VM; ensure rustup/apt is available or rerun with --no-world."
  fi

  vm_substrate="${lima_target_dir}/${target_dir}/substrate"
  log "Installing Linux substrate CLI inside Lima..."
  limactl shell substrate sudo install -Dm0755 "${vm_substrate}" /usr/local/bin/substrate
  limactl shell substrate bash -lc 'set -euo pipefail; sudo install -d /usr/local/bin; sudo tee /usr/local/bin/world >/dev/null <<'"'"'EOF'"'"'
#!/usr/bin/env bash
exec substrate world "$@"
EOF
sudo chmod 755 /usr/local/bin/world'

  if [[ "${need_build_agent}" -eq 1 ]]; then
    linux_agent="${lima_target_dir}/${target_dir}/world-agent"
  fi

  log "Installing Linux world-agent inside Lima..."
  if [[ -n "${linux_agent:-}" && "${need_build_agent}" -eq 1 ]]; then
    limactl shell substrate sudo install -m0755 "${linux_agent}" /usr/local/bin/substrate-world-agent
  else
    limactl copy "${linux_agent}" substrate:/tmp/world-agent
    limactl shell substrate sudo install -m0755 /tmp/world-agent /usr/local/bin/substrate-world-agent
    limactl shell substrate sudo rm -f /tmp/world-agent
  fi
  limactl shell substrate sudo systemctl daemon-reload
  limactl shell substrate sudo systemctl enable substrate-world-agent.service
  limactl shell substrate sudo systemctl enable --now substrate-world-agent.socket
  limactl shell substrate sudo systemctl restart substrate-world-agent.service
fi

cat >"${ENV_FILE}" <<EOF_ENV
# Generated by ${SCRIPT_NAME} on $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Source this file to enable Substrate dev shims for the current shell session.
export SUBSTRATE_ROOT="${PREFIX}"
export SUBSTRATE_MANAGER_ENV="${MANAGER_ENV_PATH}"
export SUBSTRATE_MANAGER_INIT="${MANAGER_INIT_PATH}"
if [[ -z "\${SHIM_ORIGINAL_PATH:-}" ]]; then
  export SHIM_ORIGINAL_PATH="\$PATH"
fi
if [[ ":\$PATH:" != *":${BIN_DIR}:"* ]]; then
  export PATH="${BIN_DIR}:\$PATH"
fi
if [[ ":\$PATH:" != *":${SHIMS_DIR}:"* ]]; then
  export PATH="${SHIMS_DIR}:\$PATH"
fi
EOF_ENV
log "Wrote dev shim helper to ${ENV_FILE}"

cat <<MSG

${shim_note}
To add the dev binaries/shims to PATH for this shell, run:
  source ${ENV_FILE}

MSG
log "Substrate dev install complete."
log "manager_init placeholder: ${MANAGER_INIT_PATH}"
log "manager_env script: ${MANAGER_ENV_PATH}"
if [[ -f "${INSTALL_CONFIG_PATH}" ]]; then
  log "install metadata: ${INSTALL_CONFIG_PATH}"
else
  warn "install metadata missing at ${INSTALL_CONFIG_PATH}; run 'substrate config init' after installing to create defaults."
fi
print_linger_guidance
write_host_state_metadata
