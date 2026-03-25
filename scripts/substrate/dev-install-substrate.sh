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
  --world-netfilter         Enable Linux nftables egress scoping (sets WORLD_NETFILTER_ENABLE=1 for substrate-world-agent.service)
  --anchor-mode <mode>      Default anchor mode (workspace|follow-cwd|custom; default: workspace)
  --anchor-path <path>      Default anchor path (for custom mode)
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

write_install_metadata() {
  local enabled="$1"

  local legacy_config="${PREFIX%/}/config.toml"
  if [[ -f "${legacy_config}" ]]; then
    fatal "Unsupported legacy TOML config detected at ${legacy_config}. YAML config is now required at ${INSTALL_CONFIG_PATH}. Delete the TOML file and re-run dev-install."
  fi

  mkdir -p "$(dirname "${INSTALL_CONFIG_PATH}")"

  local default_anchor_mode="workspace"
  local default_anchor_path=""
  local default_caged=1

  local need_world_patch=0
  local patch_world_enabled=""
  local patch_anchor_mode=""
  local patch_anchor_path_yaml=""
  local patch_caged=""

  if [[ "${enabled}" -ne 1 ]]; then
    need_world_patch=1
    patch_world_enabled="false"
  fi

  if [[ "${ANCHOR_MODE}" != "${default_anchor_mode}" ]]; then
    need_world_patch=1
    patch_anchor_mode="${ANCHOR_MODE}"
  fi

  if [[ "${ANCHOR_PATH}" != "${default_anchor_path}" ]]; then
    need_world_patch=1
    local escaped_anchor_path
    escaped_anchor_path="$(printf '%s' "${ANCHOR_PATH}" | sed "s/'/''/g")"
    patch_anchor_path_yaml="'${escaped_anchor_path}'"
  fi

  if [[ "${WORLD_CAGED}" -ne "${default_caged}" ]]; then
    need_world_patch=1
    patch_caged="false"
  fi

  cat > "${INSTALL_CONFIG_PATH}.tmp" <<EOF
# Substrate global config patch (sparse overrides).
# - This file is a YAML mapping of global-scoped overrides.
# - Omitted keys inherit from defaults.
EOF

  if [[ "${need_world_patch}" -eq 0 ]]; then
    printf '{}\n' >> "${INSTALL_CONFIG_PATH}.tmp"
  else
    printf 'world:\n' >> "${INSTALL_CONFIG_PATH}.tmp"
    if [[ -n "${patch_world_enabled}" ]]; then
      printf '  enabled: %s\n' "${patch_world_enabled}" >> "${INSTALL_CONFIG_PATH}.tmp"
    fi
    if [[ -n "${patch_anchor_mode}" ]]; then
      printf '  anchor_mode: %s\n' "${patch_anchor_mode}" >> "${INSTALL_CONFIG_PATH}.tmp"
    fi
    if [[ -n "${patch_anchor_path_yaml}" ]]; then
      printf '  anchor_path: %s\n' "${patch_anchor_path_yaml}" >> "${INSTALL_CONFIG_PATH}.tmp"
    fi
    if [[ -n "${patch_caged}" ]]; then
      printf '  caged: %s\n' "${patch_caged}" >> "${INSTALL_CONFIG_PATH}.tmp"
    fi
  fi

  mv "${INSTALL_CONFIG_PATH}.tmp" "${INSTALL_CONFIG_PATH}"
  chmod 0644 "${INSTALL_CONFIG_PATH}" || true
}

write_manager_env_script() {
  local enabled="$1"
  local today
  today="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  cat > "${MANAGER_ENV_PATH}.tmp" <<EOF
#!/usr/bin/env bash
# Managed by ${SCRIPT_NAME} on ${today}
if [[ -n "\${SUBSTRATE_MANAGER_ENV_ACTIVE:-}" ]]; then
  return 0
fi
export SUBSTRATE_MANAGER_ENV_ACTIVE=1

substrate_home="\${SUBSTRATE_HOME:-}"
if [[ -z "\${substrate_home}" ]]; then
  substrate_home="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
fi

substrate_env="\${substrate_home}/env.sh"
if [[ -f "\${substrate_env}" ]]; then
  # shellcheck disable=SC1090
  source "\${substrate_env}"
fi

manager_init_path="\${substrate_home}/manager_init.sh"
if [[ -f "\${manager_init_path}" ]]; then
  # shellcheck disable=SC1090
  source "\${manager_init_path}"
fi

substrate_original="\${SUBSTRATE_ORIGINAL_BASH_ENV:-}"
if [[ -n "\${substrate_original}" && -f "\${substrate_original}" ]]; then
  # shellcheck disable=SC1090
  source "\${substrate_original}"
fi

legacy_bashenv="\${HOME}/.substrate_bashenv"
if [[ -f "\${legacy_bashenv}" ]]; then
  # shellcheck disable=SC1090
  source "\${legacy_bashenv}"
fi
EOF
  mv "${MANAGER_ENV_PATH}.tmp" "${MANAGER_ENV_PATH}"
  chmod 0644 "${MANAGER_ENV_PATH}" || true
}

write_env_sh_script() {
  local enabled="$1"
  local state="disabled"
  if [[ "${enabled}" -eq 1 ]]; then
    state="enabled"
  fi

  local substrate_home_literal world_literal anchor_mode_literal anchor_path_literal policy_mode_literal caged_literal
  substrate_home_literal="$(printf '%q' "${PREFIX%/}")"
  world_literal="$(printf '%q' "${state}")"
  caged_literal="$(printf '%q' "$([[ "${WORLD_CAGED}" -eq 1 ]] && echo "1" || echo "0")")"
  anchor_mode_literal="$(printf '%q' "${ANCHOR_MODE}")"
  anchor_path_literal="$(printf '%q' "${ANCHOR_PATH}")"
  policy_mode_literal="$(printf '%q' "observe")"

  mkdir -p "$(dirname "${ENV_SH_PATH}")"
  cat > "${ENV_SH_PATH}.tmp" <<EOF
#!/usr/bin/env bash
export SUBSTRATE_HOME=${substrate_home_literal}
export SUBSTRATE_WORLD=${world_literal}
export SUBSTRATE_CAGED=${caged_literal}
export SUBSTRATE_ANCHOR_MODE=${anchor_mode_literal}
export SUBSTRATE_ANCHOR_PATH=${anchor_path_literal}
export SUBSTRATE_POLICY_MODE=${policy_mode_literal}
EOF
  mv "${ENV_SH_PATH}.tmp" "${ENV_SH_PATH}"
  chmod 0644 "${ENV_SH_PATH}" || true
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

find_linux_substrate_cli() {
  local root="$1"
  local target_dir="$2"
  local candidates=(
    "${root}/bin/linux/substrate"
    "${root}/bin/substrate-linux"
    "${root}/bin/substrate"
    "${root}/target/x86_64-unknown-linux-gnu/${target_dir}/substrate"
    "${root}/target/aarch64-unknown-linux-gnu/${target_dir}/substrate"
    "${root}/target/${target_dir}/substrate"
  )
  for candidate in "${candidates[@]}"; do
    if [[ -x "${candidate}" ]]; then
      local file_type
      file_type="$(file -b "${candidate}" 2>/dev/null || true)"
      if [[ -z "${file_type}" ]] || echo "${file_type}" | grep -qi "ELF"; then
        printf '%s\n' "${candidate}"
        return 0
      fi
    fi
  done
  return 1
}

find_linux_world_agent_elf() {
  local root="$1"
  local target_dir="$2"
  local candidate
  candidate="$(find_linux_world_agent "${root}" "${target_dir}")" || return 1
  local file_type
  file_type="$(file -b "${candidate}" 2>/dev/null || true)"
  if [[ -n "${file_type}" ]] && ! echo "${file_type}" | grep -qi "ELF"; then
    return 1
  fi
  printf '%s\n' "${candidate}"
  return 0
}

is_linux_elf() {
  local path="$1"
  if [[ ! -f "${path}" ]]; then
    return 1
  fi
  local file_type
  file_type="$(file -b "${path}" 2>/dev/null || true)"
  if [[ -n "${file_type}" ]] && ! echo "${file_type}" | grep -qi "ELF"; then
    return 1
  fi
  return 0
}

clear_managed_prefix_linux_binary_cache() {
  if [[ ! -f "${MANAGED_MAC_LINUX_BINARIES_PATH}" ]]; then
    return 0
  fi

  while IFS= read -r cached_path; do
    case "${cached_path}" in
      "${BIN_DIR}/linux/substrate"|\
      "${BIN_DIR}/linux/world-agent")
        if [[ -f "${cached_path}" && ! -L "${cached_path}" ]]; then
          rm -f "${cached_path}"
          log "Removed cached Linux guest binary ${cached_path}"
        fi
        ;;
      *)
        ;;
    esac
  done < "${MANAGED_MAC_LINUX_BINARIES_PATH}"

  rm -f "${MANAGED_MAC_LINUX_BINARIES_PATH}"
  rmdir "${MANAGED_STATE_DIR}" 2>/dev/null || true
}

record_managed_prefix_linux_binary() {
  local binary_path="$1"
  mkdir -p "${MANAGED_STATE_DIR}"

  local tmp="${MANAGED_MAC_LINUX_BINARIES_PATH}.tmp"
  : > "${tmp}"
  if [[ -f "${MANAGED_MAC_LINUX_BINARIES_PATH}" ]]; then
    grep -Fxv -- "${binary_path}" "${MANAGED_MAC_LINUX_BINARIES_PATH}" > "${tmp}" || true
  fi
  printf '%s\n' "${binary_path}" >> "${tmp}"
  mv "${tmp}" "${MANAGED_MAC_LINUX_BINARIES_PATH}"
}

cache_linux_binary_from_lima() {
  local vm_path="$1"
  local dest_path="$2"
  local label="$3"

  mkdir -p "$(dirname "${dest_path}")"
  rm -f "${dest_path}"
  if ! limactl copy "substrate:${vm_path}" "${dest_path}"; then
    warn "Failed to copy Linux ${label} from Lima into ${dest_path}"
    return 1
  fi
  chmod 0755 "${dest_path}" 2>/dev/null || true
  if ! is_linux_elf "${dest_path}"; then
    warn "Copied Linux ${label} at ${dest_path} is not a Linux ELF"
    rm -f "${dest_path}"
    return 1
  fi
  record_managed_prefix_linux_binary "${dest_path}"
  log "Cached Linux ${label} into ${dest_path}"
}

verify_prefix_linux_bundle() {
  local missing=0
  local binary path
  for binary in substrate world-agent; do
    path="${BIN_DIR}/linux/${binary}"
    if ! is_linux_elf "${path}"; then
      warn "Expected cached Linux ${binary} at ${path}, but it is missing or not a Linux ELF."
      missing=1
    fi
  done
  return "${missing}"
}

stage_dev_world_runtime_bundle() {
  local prefix_root="$1"
  local repo_root="$2"
  local target_dir="$3"
  local scripts_substrate_dir="${prefix_root%/}/scripts/substrate"
  local scripts_mac_dir="${prefix_root%/}/scripts/mac"
  local scripts_mac_lima_dir="${scripts_mac_dir}/lima"
  local bin_linux_dir="${prefix_root%/}/bin/linux"
  mkdir -p "${scripts_substrate_dir}" "${scripts_mac_dir}" "${scripts_mac_lima_dir}" "${bin_linux_dir}"

  local -a script_pairs=(
    "${repo_root}/scripts/substrate/world-enable.sh:${scripts_substrate_dir}/world-enable.sh"
    "${repo_root}/scripts/substrate/install-substrate.sh:${scripts_substrate_dir}/install-substrate.sh"
    "${repo_root}/scripts/substrate/world-deps.yaml:${scripts_substrate_dir}/world-deps.yaml"
    "${repo_root}/scripts/mac/lima-warm.sh:${scripts_mac_dir}/lima-warm.sh"
    "${repo_root}/scripts/mac/lima/substrate.yaml:${scripts_mac_lima_dir}/substrate.yaml"
    "${repo_root}/scripts/mac/lima/substrate-dev.yaml:${scripts_mac_lima_dir}/substrate-dev.yaml"
  )
  local pair src dest
  for pair in "${script_pairs[@]}"; do
    src="${pair%%:*}"
    dest="${pair#*:}"
    if [[ -f "${src}" ]]; then
      ln -sfn "${src}" "${dest}"
      log "Linked runtime bundle artifact into ${dest}"
    else
      warn "Runtime bundle source missing at ${src}; world enable may fail."
    fi
  done

  local linux_cli
  linux_cli="$(find_linux_substrate_cli "${repo_root}" "${target_dir}")" || true
  if [[ -n "${linux_cli:-}" ]]; then
    ln -sfn "${linux_cli}" "${bin_linux_dir}/substrate"
    log "Linked Linux substrate CLI into ${bin_linux_dir}/substrate"
  else
    rm -f "${bin_linux_dir}/substrate"
    log "Linux substrate CLI not available; Lima warm will build guest CLI only if needed."
  fi

  local linux_agent
  linux_agent="$(find_linux_world_agent_elf "${repo_root}" "${target_dir}")" || true
  if [[ -n "${linux_agent:-}" ]]; then
    ln -sfn "${linux_agent}" "${bin_linux_dir}/world-agent"
    log "Linked Linux world-agent into ${bin_linux_dir}/world-agent"
  else
    rm -f "${bin_linux_dir}/world-agent"
    log "Linux world-agent not available; Lima warm will fall back to in-guest build."
  fi
}

cleanup_legacy_world_enable_helper_bridge() {
  local target_root="$1"
  local repo_root="$2"
  local legacy_dir="${target_root%/}/scripts/substrate"
  local helper target
  for helper in world-enable.sh install-substrate.sh; do
    local path="${legacy_dir}/${helper}"
    if [[ -L "${path}" ]]; then
      target="$(readlink "${path}" || true)"
      if [[ "${target}" == "${repo_root}/scripts/substrate/"* ]]; then
        rm -f "${path}"
        log "Removed legacy helper bridge ${path}"
      fi
    fi
  done
  rmdir "${legacy_dir}" 2>/dev/null || true
  rmdir "${target_root%/}/scripts" 2>/dev/null || true
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
ANCHOR_MODE="workspace"
ANCHOR_PATH=""
WORLD_CAGED=1
VERSION_LABEL="dev"
ENABLE_WORLD_NETFILTER=0
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
    --world-netfilter)
      ENABLE_WORLD_NETFILTER=1
      shift
      ;;
    --world-root-mode)
      fatal "--world-root-mode was removed; use --anchor-mode"
      ;;
    --anchor-mode)
      [[ $# -ge 2 ]] || fatal "--anchor-mode requires a value"
      ANCHOR_MODE="$2"
      shift 2
      ;;
    --world-root-path)
      fatal "--world-root-path was removed; use --anchor-path"
      ;;
    --anchor-path)
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
  workspace|follow-cwd|custom) ;;
  *) fatal "Unsupported anchor mode '${ANCHOR_MODE}'. Use workspace, follow-cwd, or custom." ;;
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
INSTALL_CONFIG_PATH="${PREFIX%/}/config.yaml"
ENV_SH_PATH="${PREFIX%/}/env.sh"
MANAGED_STATE_DIR="${PREFIX%/}/.dev-install-managed"
MANAGED_MAC_LINUX_BINARIES_PATH="${MANAGED_STATE_DIR}/mac-linux-binaries.txt"

mkdir -p "${PREFIX}" "${BIN_DIR}" "${VERSION_CONFIG_DIR}"
clear_managed_prefix_linux_binary_cache

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

# Write manager init placeholder + env exporter.
cat > "${MANAGER_INIT_PATH}.tmp" <<'EOF'
#!/usr/bin/env bash
# Managed by dev-install-substrate

# Place per-manager snippets here if you need them for debugging.
EOF
mv "${MANAGER_INIT_PATH}.tmp" "${MANAGER_INIT_PATH}"
chmod 0644 "${MANAGER_INIT_PATH}" || true

# Write install metadata (install + world mappings) like the production installer.
write_install_metadata "${WORLD_ENABLED}"
write_env_sh_script "${WORLD_ENABLED}"
write_manager_env_script "${WORLD_ENABLED}"

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
  cleanup_legacy_world_enable_helper_bridge "${version_root}" "${REPO_ROOT}"
  ensure_release_bin_bridge "${version_root}" "${TARGET_DIR}"
fi
stage_dev_world_runtime_bundle "${PREFIX}" "${REPO_ROOT}" "${TARGET_DIR}"

if [[ "${WORLD_ENABLED}" -eq 1 && "${IS_LINUX}" -eq 1 ]]; then
  if [[ ${EUID} -ne 0 ]] && command -v sudo >/dev/null 2>&1; then
    log "Caching sudo credentials for world provisioning (you may be prompted)..."
    if ! sudo -v; then
      WORLD_PROVISION_FAILED=1
      WORLD_ENABLED=0
      write_install_metadata "${WORLD_ENABLED}"
      write_env_sh_script "${WORLD_ENABLED}"
      write_manager_env_script "${WORLD_ENABLED}"
      warn "Unable to cache sudo credentials; world-agent service not provisioned."
      warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures. Re-run provisioning, then run `substrate world enable --home \"${PREFIX}\"` to flip it back on."
    fi
  fi
  if [[ "${WORLD_ENABLED}" -eq 0 ]]; then
    : # world provisioning failed above; skip the remainder of the provisioning block.
  else
  ensure_substrate_group_membership
	  PROVISION_SCRIPT="${REPO_ROOT}/scripts/linux/world-provision.sh"
	  if [[ -x "${PROVISION_SCRIPT}" ]]; then
	    log "Provisioning Linux world-agent service via ${PROVISION_SCRIPT} (sudo may prompt)..."
	    provision_args=(--profile "${PROFILE}" --skip-build)
	    if [[ "${ENABLE_WORLD_NETFILTER}" -eq 1 ]]; then
	      provision_args+=(--world-netfilter)
	    fi
	    if ! SUBSTRATE_HOME="${PREFIX}" "${PROVISION_SCRIPT}" "${provision_args[@]}"; then
	      WORLD_PROVISION_FAILED=1
	      WORLD_ENABLED=0
	      write_install_metadata "${WORLD_ENABLED}"
	      write_env_sh_script "${WORLD_ENABLED}"
      write_manager_env_script "${WORLD_ENABLED}"
      warn "world-provision script reported an error; rerun ${PROVISION_SCRIPT} manually to enable the world-agent service."
      warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures. Re-run provisioning, then run `substrate world enable --home \"${PREFIX}\"` to flip it back on."
    fi
  else
    WORLD_PROVISION_FAILED=1
    WORLD_ENABLED=0
    write_install_metadata "${WORLD_ENABLED}"
    write_env_sh_script "${WORLD_ENABLED}"
    write_manager_env_script "${WORLD_ENABLED}"
    warn "Linux world-provision script missing at ${PROVISION_SCRIPT}; world-agent service not configured."
    warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures."
  fi
  ensure_socket_group_alignment
  fi
elif [[ "${WORLD_ENABLED}" -eq 1 && "${IS_MAC}" -eq 1 ]]; then
  log "Provisioning macOS Lima world-agent service..."
  if ! command -v limactl >/dev/null 2>&1; then
    fatal "limactl not found; install Lima or rerun with --no-world to skip macOS world provisioning."
  fi
  LIMA_WARM="${REPO_ROOT}/scripts/mac/lima-warm.sh"
  if [[ ! -x "${LIMA_WARM}" ]]; then
    fatal "Expected Lima warm helper at ${LIMA_WARM}"
  fi
  if [[ "${ENABLE_WORLD_NETFILTER}" -eq 1 ]]; then
    (cd "${REPO_ROOT}" && SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1 SUBSTRATE_WORLD_NETFILTER_ENABLE=1 "${LIMA_WARM}" "${REPO_ROOT}")
  else
    (cd "${REPO_ROOT}" && SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1 "${LIMA_WARM}" "${REPO_ROOT}")
  fi

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

  cache_ok=1
  if ! cache_linux_binary_from_lima /usr/local/bin/substrate "${BIN_DIR}/linux/substrate" "substrate CLI"; then
    cache_ok=0
  fi
  if ! cache_linux_binary_from_lima /usr/local/bin/substrate-world-agent "${BIN_DIR}/linux/world-agent" "world-agent"; then
    cache_ok=0
  fi

  if [[ "${cache_ok}" -eq 0 ]] || ! verify_prefix_linux_bundle; then
    WORLD_ENABLED=0
    write_install_metadata "${WORLD_ENABLED}"
    write_env_sh_script "${WORLD_ENABLED}"
    write_manager_env_script "${WORLD_ENABLED}"
    warn "macOS dev-install did not produce a reusable Linux guest-binary bundle under ${BIN_DIR}/linux."
    warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures. Re-run dev-install after fixing Lima provisioning."
  fi
fi

cat >"${ENV_FILE}" <<EOF_ENV
# Generated by ${SCRIPT_NAME} on $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Source this file to enable Substrate dev shims for the current shell session.
export SUBSTRATE_ROOT="${PREFIX}"
export SUBSTRATE_HOME="${PREFIX}"
if [[ -f "${PREFIX}/env.sh" ]]; then
  # shellcheck disable=SC1090
  source "${PREFIX}/env.sh"
fi
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
if [[ "${WORLD_PROVISION_FAILED:-0}" -eq 1 ]]; then
  fatal "Substrate dev install finished, but world provisioning failed. Re-run with an interactive sudo session (or pass --no-world to skip provisioning)."
fi
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
