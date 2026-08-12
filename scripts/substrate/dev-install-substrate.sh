#!/usr/bin/env bash
if [[ $- == *x* ]]; then
  set +x
fi
set -euo pipefail

SCRIPT_NAME="dev-install-substrate"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }
fatal_with_code() {
  local code="$1"
  shift
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit "${code}"
}

readonly DISTRO_UNKNOWN_SENTINEL="<unknown>"
readonly SUPPORTED_PKG_MANAGERS=(apt-get dnf yum pacman zypper)
readonly PRIVILEGED_TOOL_PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0

resolve_install_bootstrap_context() {
  local declared="$1"
  local raw_prefix="$2"
  local supplied_carrier="$3"

  if ! {
    IFS= read -r -d '' PREFIX
    IFS= read -r -d '' INSTALL_BOOTSTRAP_CONTEXT_V1
    IFS= read -r -d '' INSTALL_BOOTSTRAP_COMMITMENT
    IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT
    IFS= read -r -d '' INSTALL_BOOTSTRAP_UID
    IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT_HOME
  } < <(python3 - "${declared}" "${raw_prefix}" "${supplied_carrier}" <<'PY'
import base64
import hashlib
import os
import pwd
import re
import sys

DOMAIN = "substrate.install_bootstrap_context"
KEYS = (
    "domain",
    "version",
    "selected_host_prefix",
    "host_substrate_home",
    "host_substrate_root",
    "principal_kind",
    "principal_account",
    "principal_uid",
    "host_context_commitment",
)


def fail():
    raise ValueError("invalid install bootstrap context")


def normalize_path(raw):
    if not raw or raw == "/" or not raw.startswith("/") or raw.startswith("//") or "\0" in raw:
        fail()
    parts = []
    for part in raw[1:].split("/"):
        if not part:
            continue
        if part in (".", ".."):
            fail()
        parts.append(part)
    if not parts:
        fail()
    return "/" + "/".join(parts)


def b64_encode(value):
    return base64.urlsafe_b64encode(value).rstrip(b"=")


def b64_decode(value):
    if not value or not re.fullmatch(rb"[A-Za-z0-9_-]+", value):
        fail()
    decoded = base64.urlsafe_b64decode(value + b"=" * ((-len(value)) % 4))
    if b64_encode(decoded) != value:
        fail()
    return decoded.decode("utf-8")


def frame(prefix, account, uid):
    return (
        f"domain={DOMAIN}\n"
        "version=1\n"
        f"selected_host_prefix={b64_encode(prefix.encode('utf-8')).decode('ascii')}\n"
        f"host_substrate_home={b64_encode(prefix.encode('utf-8')).decode('ascii')}\n"
        f"host_substrate_root={b64_encode(prefix.encode('utf-8')).decode('ascii')}\n"
        "principal_kind=unix\n"
        f"principal_account={b64_encode(account.encode('utf-8')).decode('ascii')}\n"
        f"principal_uid={uid}\n"
    ).encode("ascii")


def current_principal():
    uid = os.geteuid()
    if uid == 0 or uid > 0xFFFFFFFF:
        fail()
    entry = pwd.getpwuid(uid)
    if not entry.pw_name or pwd.getpwnam(entry.pw_name).pw_uid != uid:
        fail()
    if any(ch in entry.pw_name for ch in "\0\r\n"):
        fail()
    return entry, uid


def decode_carrier(encoded):
    raw_encoded = encoded.encode("ascii")
    if not raw_encoded or not re.fullmatch(rb"[A-Za-z0-9_-]+", raw_encoded):
        fail()
    record = base64.urlsafe_b64decode(raw_encoded + b"=" * ((-len(raw_encoded)) % 4))
    if b64_encode(record) != raw_encoded or not record.endswith(b"\n") or b"\r" in record or b"\0" in record:
        fail()
    lines = record[:-1].split(b"\n")
    if len(lines) != len(KEYS):
        fail()
    values = {}
    for expected, line in zip(KEYS, lines):
        if line.count(b"=") != 1:
            fail()
        key, value = line.split(b"=", 1)
        if key.decode("ascii") != expected:
            fail()
        values[expected] = value
    if values["domain"] != DOMAIN.encode("ascii") or values["version"] != b"1" or values["principal_kind"] != b"unix":
        fail()
    prefix = normalize_path(b64_decode(values["selected_host_prefix"]))
    if b64_decode(values["host_substrate_home"]) != prefix or b64_decode(values["host_substrate_root"]) != prefix:
        fail()
    account = b64_decode(values["principal_account"])
    raw_uid = values["principal_uid"]
    if not re.fullmatch(rb"0|[1-9][0-9]*", raw_uid):
        fail()
    uid = int(raw_uid)
    if uid == 0 or uid > 0xFFFFFFFF:
        fail()
    commitment = values["host_context_commitment"].decode("ascii")
    if not re.fullmatch(r"[0-9a-f]{64}", commitment):
        fail()
    expected_frame = frame(prefix, account, uid)
    if record != expected_frame + b"host_context_commitment=" + commitment.encode("ascii") + b"\n":
        fail()
    if hashlib.sha256(expected_frame).hexdigest() != commitment:
        fail()
    return prefix, account, uid, commitment


try:
    declared = sys.argv[1] == "1"
    raw_prefix = sys.argv[2]
    supplied = sys.argv[3]
    entry, current_uid = current_principal()
    if supplied:
        prefix, account, uid, commitment = decode_carrier(supplied)
        if account != entry.pw_name or uid != current_uid:
            fail()
        if declared and normalize_path(raw_prefix) != prefix:
            fail()
        expected = {
            "SUBSTRATE_HOME": prefix,
            "SUBSTRATE_ROOT": prefix,
            "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT": commitment,
            "SUBSTRATE_INSTALL_PRIMARY_USER": account,
            "SUBSTRATE_INSTALL_PRIMARY_UID": str(uid),
            "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1": supplied,
        }
        for key, value in expected.items():
            if key in os.environ and os.environ[key] != value:
                fail()
        carrier = supplied
    else:
        prefix = normalize_path(raw_prefix if declared else entry.pw_dir.rstrip("/") + "/.substrate")
        account = entry.pw_name
        uid = current_uid
        commitment_input = frame(prefix, account, uid)
        commitment = hashlib.sha256(commitment_input).hexdigest()
        carrier = b64_encode(commitment_input + f"host_context_commitment={commitment}\n".encode("ascii")).decode("ascii")
    account_home = normalize_path(entry.pw_dir)
    for value in (prefix, carrier, commitment, account, str(uid), account_home):
        sys.stdout.buffer.write(value.encode("utf-8") + b"\0")
except Exception:
    print("invalid install bootstrap context", file=sys.stderr)
    raise SystemExit(2)
PY
  ); then
    fatal "Unable to resolve install bootstrap context."
  fi

  export SUBSTRATE_HOME="${PREFIX}"
  export SUBSTRATE_ROOT="${PREFIX}"
  export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}"
  export SUBSTRATE_INSTALL_PRIMARY_USER="${INSTALL_BOOTSTRAP_ACCOUNT}"
  export SUBSTRATE_INSTALL_PRIMARY_UID="${INSTALL_BOOTSTRAP_UID}"
  export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_BOOTSTRAP_CONTEXT_V1}"
}

usage() {
  cat <<'USAGE'
Substrate Dev Installer

Build Substrate from the current repository and wire development shims to the
freshly built binaries. This is intended for local iteration after removing any
production installation.

Usage:
  dev-install-substrate.sh [--prefix <path>] [--profile <debug|release>] [--version-label <name>] [--no-world] [--anchor-mode <mode>] [--anchor-path <path>] [--caged|--uncaged] [--no-shims] [--provision-agent-runtime <runtime_family>]
  dev-install-substrate.sh --help

Options:
  --prefix <path>           Installation prefix for shims/env helper (default: ~/.substrate)
  --profile <name>          Cargo profile to build (debug or release; default: debug)
  --version-label <name>    Version directory label under <prefix>/versions (default: dev)
  --no-world                Mark install metadata as world_disabled (skips provisioning entirely)
  --world-netfilter         Enable Linux nftables egress scoping (sets WORLD_NETFILTER_ENABLE=1 for substrate-world-service.service)
  --provision-agent-runtime <runtime_family>
                            Enable a world runtime globally (codex only in this slice), then run
                            'substrate world deps current sync' before the dev install completes
  --anchor-mode <mode>      Default anchor mode (workspace|follow-cwd|custom; default: workspace)
  --anchor-path <path>      Default anchor path (for custom mode)
  --caged                   Write caged=true to install metadata (default)
  --uncaged                 Write caged=false to install metadata
  --no-shims                Skip shim deployment (only run cargo build)
  --help                    Show this message
USAGE
}

supported_agent_runtime_list() {
  printf 'codex'
}

world_deps_global_remove_scope_note() {
  printf 'It does not remove any guest-side state that may already have been applied in the world.'
}

world_deps_item_for_agent_runtime() {
  local runtime_family="$1"

  case "${runtime_family}" in
    codex)
      printf 'codex-runtime\n'
      ;;
    *)
      fatal_with_code 2 "Unsupported value for --provision-agent-runtime: '${runtime_family}'. This slice supports: $(supported_agent_runtime_list)."
      ;;
  esac
}

validate_agent_runtime_provision_request() {
  if [[ -z "${PROVISION_AGENT_RUNTIME}" ]]; then
    return
  fi

  world_deps_item_for_agent_runtime "${PROVISION_AGENT_RUNTIME}" >/dev/null

  if [[ "${WORLD_ENABLED}" -ne 1 ]]; then
    fatal_with_code 2 "--provision-agent-runtime requires world provisioning. Remove --no-world or omit the runtime flag."
  fi
}

fail_closed_world_provisioning_for_runtime_request() {
  local detail="$1"
  local remediation="$2"

  if [[ -z "${PROVISION_AGENT_RUNTIME}" ]]; then
    return 0
  fi

  WORLD_ENABLED=0
  if [[ -n "${INSTALL_CONFIG_PATH:-}" && -n "${ENV_SH_PATH:-}" && -n "${MANAGER_ENV_PATH:-}" ]]; then
    write_install_metadata "${WORLD_ENABLED}"
    write_env_sh_script "${WORLD_ENABLED}"
    write_manager_env_script "${WORLD_ENABLED}"
  fi

  fatal_with_code 1 "Cannot continue --provision-agent-runtime ${PROVISION_AGENT_RUNTIME} because world provisioning failed: ${detail} ${remediation}"
}

rollback_agent_runtime_after_failed_sync() {
  local substrate_bin="$1"
  local deps_item="$2"
  local runtime_path="$3"
  local original_path="${PATH}"
  local trace_was_active=0
  local rollback_succeeded=0

  if [[ $- == *x* ]]; then
    trace_was_active=1
    set +x
  fi
  if PATH="${runtime_path}" SHIM_ORIGINAL_PATH="${original_path}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" \
    "${substrate_bin}" --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world deps global remove "${deps_item}"; then
    rollback_succeeded=1
  fi
  if [[ "${trace_was_active}" -eq 1 ]]; then
    set -x
  fi

  if [[ "${rollback_succeeded}" -eq 1 ]]; then
    warn "Rolled back world deps global enable for '${deps_item}' after sync failure."
    return 0
  fi

  warn "Unable to roll back world deps global enable for '${deps_item}' after sync failure."
  return 1
}

dev_install_retry_after_sync_failure() {
  local deps_item
  deps_item="$(world_deps_item_for_agent_runtime "${PROVISION_AGENT_RUNTIME}")"
  printf "Re-run the dev install with '--provision-agent-runtime %s' to re-add '%s' and retry the sync." "${PROVISION_AGENT_RUNTIME}" "${deps_item}"
}

provision_agent_runtime_with_sync() {
  local substrate_bin="$1"
  if [[ -z "${PROVISION_AGENT_RUNTIME}" ]]; then
    return
  fi

  local deps_item
  local runtime_path
  local original_path="${PATH}"
  local trace_was_active=0
  deps_item="$(world_deps_item_for_agent_runtime "${PROVISION_AGENT_RUNTIME}")"
  runtime_path="${BIN_DIR}:${PATH}"

  log "Enabling agent runtime '${PROVISION_AGENT_RUNTIME}' globally via world deps item '${deps_item}'. The dev installer will run 'substrate world deps current sync' immediately after this step."
  local add_output
  local add_status=0
  if [[ $- == *x* ]]; then
    trace_was_active=1
    set +x
  fi
  add_output="$(PATH="${runtime_path}" SHIM_ORIGINAL_PATH="${original_path}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" \
    "${substrate_bin}" --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world deps global add --json "${deps_item}")" || add_status=$?
  if [[ "${trace_was_active}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${add_status}" -ne 0 ]]; then
    return "${add_status}"
  fi
  if grep -Fq "\"${deps_item}\"" <<<"${add_output}"; then
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=1
  else
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0
  fi
  printf '%s\n' "${add_output}"
  log "Syncing world dependencies via 'substrate world deps current sync' for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}..."
  log "This step may download the guest runtime inside the world and can take several minutes. Current world-deps script installs return output only after the guest command exits."
  local rc=0
  local sync_succeeded=0
  trace_was_active=0
  if [[ $- == *x* ]]; then
    trace_was_active=1
    set +x
  fi
  if PATH="${runtime_path}" SHIM_ORIGINAL_PATH="${original_path}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" \
    "${substrate_bin}" --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world deps current sync; then
    sync_succeeded=1
  else
    rc=$?
  fi
  if [[ "${trace_was_active}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${sync_succeeded}" -eq 1 ]]; then
    return
  fi

  if [[ "${PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER}" -ne 1 ]]; then
    if [[ "${rc}" -eq 4 ]]; then
      fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; '${deps_item}' was already globally enabled before this dev install, so the dev installer left that enable in place. Run 'substrate world enable --provision-deps', then rerun 'substrate world deps current sync'."
    fi
    fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; '${deps_item}' was already globally enabled before this dev install, so the dev installer left that enable in place. Fix the sync failure and rerun 'substrate world deps current sync'."
  fi

  if rollback_agent_runtime_after_failed_sync "${substrate_bin}" "${deps_item}" "${runtime_path}"; then
    if [[ "${rc}" -eq 4 ]]; then
      fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; the dev installer removed the global enable because provisioning-time system packages are still required. Clearing the global enable does not roll back any guest-side state that may already have been applied in the world. Run 'substrate world enable --provision-deps', then $(dev_install_retry_after_sync_failure)"
    fi
    fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; the dev installer removed the global enable so the runtime is not left persistently enabled for future syncs. $(world_deps_global_remove_scope_note) Then $(dev_install_retry_after_sync_failure)"
  fi

  if [[ "${rc}" -eq 4 ]]; then
    fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; provisioning-time system packages are still required, and rollback also failed so '${deps_item}' remains globally enabled. Run 'substrate world deps global remove ${deps_item}' to clear the global enable only. $(world_deps_global_remove_scope_note) Then run 'substrate world enable --provision-deps', then $(dev_install_retry_after_sync_failure)"
  fi
  fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; rollback also failed so '${deps_item}' remains globally enabled. Run 'substrate world deps global remove ${deps_item}' after fixing the sync failure to clear the global enable only. $(world_deps_global_remove_scope_note) Then $(dev_install_retry_after_sync_failure)"
}

run_privileged() {
  local trace_was_active=0
  if [[ $- == *x* ]]; then
    trace_was_active=1
    set +x
  fi
  if [[ -z "${PREFIX}" \
      || "${SUBSTRATE_HOME:-}" != "${PREFIX}" \
      || "${SUBSTRATE_ROOT:-}" != "${PREFIX}" \
      || "${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT:-}" != "${INSTALL_BOOTSTRAP_COMMITMENT}" \
      || "${SUBSTRATE_INSTALL_PRIMARY_USER:-}" != "${INSTALL_BOOTSTRAP_ACCOUNT}" \
      || "${SUBSTRATE_INSTALL_PRIMARY_UID:-}" != "${INSTALL_BOOTSTRAP_UID}" \
      || "${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}" != "${INSTALL_BOOTSTRAP_CONTEXT_V1}" ]]; then
    if [[ "${trace_was_active}" -eq 1 ]]; then
      set -x
    fi
    warn "Install bootstrap context projection changed before privileged tool dispatch."
    return 2
  fi
  local tool="$1"
  shift
  local tool_path=""
  if [[ "${tool}" == */* ]]; then
    if [[ "${trace_was_active}" -eq 1 ]]; then
      set -x
    fi
    warn "Unsupported absolute privileged tool path: ${tool}"
    return 126
  else
    tool_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- "${tool}" 2>/dev/null || true)"
  fi
  if [[ -z "${tool_path}" ]]; then
    if [[ "${trace_was_active}" -eq 1 ]]; then
      set -x
    fi
    warn "Unable to resolve privileged tool: ${tool}"
    return 127
  fi
  local env_path=""
  env_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- env 2>/dev/null || true)"
  if [[ -z "${env_path}" ]]; then
    if [[ "${trace_was_active}" -eq 1 ]]; then
      set -x
    fi
    warn "Unable to resolve privileged environment scrubber: env"
    return 127
  fi
  local -a scrubbed_command=(
    "${env_path}" -i
    "PATH=${PRIVILEGED_TOOL_PATH}"
    "HOME=/root"
    "USER=root"
    "LOGNAME=root"
    "${tool_path}"
    "$@"
  )
  if [[ "${trace_was_active}" -eq 1 ]]; then
    set -x
  fi
  if [[ ${EUID} -eq 0 ]]; then
    "${scrubbed_command[@]}"
    return $?
  fi
  if command -v sudo >/dev/null 2>&1; then
    local true_path
    true_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- true 2>/dev/null || true)"
    if [[ -z "${true_path}" ]]; then
      warn "Unable to resolve privileged sudo probe tool: true"
      return 127
    fi
    if sudo -n -- "${env_path}" -i "PATH=${PRIVILEGED_TOOL_PATH}" "${true_path}" >/dev/null 2>&1; then
      sudo -n -- "${scrubbed_command[@]}"
      return $?
    fi
    if [[ ! -t 0 && ! -t 1 && ! -t 2 ]]; then
      warn "Command requires elevated privileges but no interactive sudo prompt is possible: $*"
      return 1
    fi
    sudo -- "${scrubbed_command[@]}"
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
  chmod 0600 "${INSTALL_CONFIG_PATH}" || true
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

substrate_home="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export SUBSTRATE_HOME="\${substrate_home}"
export SUBSTRATE_ROOT="\${substrate_home}"
export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=$(printf '%q' "${INSTALL_BOOTSTRAP_COMMITMENT}")
export SUBSTRATE_INSTALL_PRIMARY_USER=$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT}")
export SUBSTRATE_INSTALL_PRIMARY_UID=$(printf '%q' "${INSTALL_BOOTSTRAP_UID}")
export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=$(printf '%q' "${INSTALL_BOOTSTRAP_CONTEXT_V1}")

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

legacy_bashenv=$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT_HOME%/}/.substrate_bashenv")
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
  local commitment_literal account_literal uid_literal carrier_literal
  substrate_home_literal="$(printf '%q' "${PREFIX%/}")"
  commitment_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_COMMITMENT}")"
  account_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT}")"
  uid_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_UID}")"
  carrier_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_CONTEXT_V1}")"
  world_literal="$(printf '%q' "${state}")"
  caged_literal="$(printf '%q' "$([[ "${WORLD_CAGED}" -eq 1 ]] && echo "1" || echo "0")")"
  anchor_mode_literal="$(printf '%q' "${ANCHOR_MODE}")"
  anchor_path_literal="$(printf '%q' "${ANCHOR_PATH}")"
  policy_mode_literal="$(printf '%q' "observe")"

  mkdir -p "$(dirname "${ENV_SH_PATH}")"
  cat > "${ENV_SH_PATH}.tmp" <<EOF
#!/usr/bin/env bash
export SUBSTRATE_HOME=${substrate_home_literal}
export SUBSTRATE_ROOT=${substrate_home_literal}
export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${commitment_literal}
export SUBSTRATE_INSTALL_PRIMARY_USER=${account_literal}
export SUBSTRATE_INSTALL_PRIMARY_UID=${uid_literal}
export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=${carrier_literal}
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

bootstrap_private_substrate_home() {
  local substrate_bin="$1"
  if ! "${substrate_bin}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    --install-bootstrap-home-v1 >/dev/null; then
    fatal "Private SUBSTRATE_HOME bootstrap rejected ${PREFIX}; no existing root was repaired."
  fi
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

find_linux_world_service() {
  local root="$1"
  local target_dir="$2"
  local candidates=(
    "${root}/bin/linux/world-service"
    "${root}/bin/world-service-linux"
    "${root}/bin/world-service"
    "${root}/target/x86_64-unknown-linux-gnu/${target_dir}/world-service"
    "${root}/target/aarch64-unknown-linux-gnu/${target_dir}/world-service"
    "${root}/target/${target_dir}/world-service"
  )
  for candidate in "${candidates[@]}"; do
    if [[ -x "${candidate}" ]]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done
  return 1
}

reset_os_release_input_state() {
  OS_RELEASE_SELECTED_PATH=""
  OS_RELEASE_INPUT_STATE="unavailable"
  DETECTED_DISTRO_ID="${DISTRO_UNKNOWN_SENTINEL}"
  DETECTED_DISTRO_ID_LIKE="${DISTRO_UNKNOWN_SENTINEL}"
}

resolve_selected_os_release_input() {
  local selected_path="${SUBSTRATE_INSTALL_OS_RELEASE_PATH:-}"
  local os_release_fd

  reset_os_release_input_state

  if [[ -z "${selected_path}" ]]; then
    selected_path="/etc/os-release"
  fi

  if [[ "${selected_path}" != /* ]]; then
    return 1
  fi

  if [[ ! -f "${selected_path}" || ! -r "${selected_path}" ]]; then
    return 1
  fi

  if ! exec {os_release_fd}<"${selected_path}"; then
    return 1
  fi
  exec {os_release_fd}<&-

  OS_RELEASE_SELECTED_PATH="${selected_path}"
  OS_RELEASE_INPUT_STATE="selected"
  return 0
}

trim_ascii_whitespace() {
  local value="$1"
  value="${value#"${value%%[!$' \t\r']*}"}"
  value="${value%"${value##*[!$' \t\r']}"}"
  printf '%s' "${value}"
}

strip_matching_quotes() {
  local value="$1"
  if [[ ${#value} -ge 2 ]]; then
    case "${value:0:1}${value: -1}" in
      "''"|'""')
        value="${value:1:${#value}-2}"
        ;;
    esac
  fi
  printf '%s' "${value}"
}

parse_selected_os_release_fields() {
  local line=""
  local key=""
  local raw_value=""
  local normalized_value=""

  DETECTED_DISTRO_ID="${DISTRO_UNKNOWN_SENTINEL}"
  DETECTED_DISTRO_ID_LIKE="${DISTRO_UNKNOWN_SENTINEL}"

  if [[ "${OS_RELEASE_INPUT_STATE}" != "selected" || -z "${OS_RELEASE_SELECTED_PATH}" ]]; then
    return 1
  fi

  while IFS= read -r line || [[ -n "${line}" ]]; do
    if [[ "${line}" =~ ^[[:space:]]*$ ]]; then
      continue
    fi
    if [[ "${line}" =~ ^[[:space:]]*# ]]; then
      continue
    fi

    key="${line%%=*}"
    if [[ "${key}" == "${line}" ]]; then
      continue
    fi

    raw_value="${line#*=}"
    raw_value="$(trim_ascii_whitespace "${raw_value}")"
    normalized_value="$(strip_matching_quotes "${raw_value}")"
    normalized_value="${normalized_value,,}"
    if [[ -z "${normalized_value}" ]]; then
      normalized_value="${DISTRO_UNKNOWN_SENTINEL}"
    fi

    case "${key}" in
      ID)
        DETECTED_DISTRO_ID="${normalized_value}"
        ;;
      ID_LIKE)
        DETECTED_DISTRO_ID_LIKE="${normalized_value}"
        ;;
    esac
  done < "${OS_RELEASE_SELECTED_PATH}"

  return 0
}

os_release_id_like_has_token() {
  local needle="$1"
  local token=""

  if [[ -z "${needle}" || "${DETECTED_DISTRO_ID_LIKE}" == "${DISTRO_UNKNOWN_SENTINEL}" ]]; then
    return 1
  fi

  for token in ${DETECTED_DISTRO_ID_LIKE}; do
    if [[ "${token}" == "${needle}" ]]; then
      return 0
    fi
  done

  return 1
}

os_release_matches_debian_family() {
  case "${DETECTED_DISTRO_ID}" in
    debian|ubuntu|linuxmint|pop)
      return 0
      ;;
  esac

  os_release_id_like_has_token "debian" || os_release_id_like_has_token "ubuntu"
}

os_release_matches_fedora_rhel_family() {
  case "${DETECTED_DISTRO_ID}" in
    fedora|rhel|centos|rocky|almalinux|ol|amzn)
      return 0
      ;;
  esac

  os_release_id_like_has_token "fedora" || os_release_id_like_has_token "rhel"
}

os_release_matches_arch_family() {
  case "${DETECTED_DISTRO_ID}" in
    arch|manjaro|endeavouros|arcolinux|artix|garuda)
      return 0
      ;;
  esac

  os_release_id_like_has_token "arch"
}

os_release_matches_suse_family() {
  local token=""

  case "${DETECTED_DISTRO_ID}" in
    *suse*)
      return 0
      ;;
  esac

  if [[ "${DETECTED_DISTRO_ID_LIKE}" == "${DISTRO_UNKNOWN_SENTINEL}" ]]; then
    return 1
  fi

  for token in ${DETECTED_DISTRO_ID_LIKE}; do
    case "${token}" in
      *suse*)
        return 0
        ;;
    esac
  done

  return 1
}

select_package_manager_from_os_release() {
  PKG_MANAGER=""
  PKG_MANAGER_SOURCE=""

  if os_release_matches_debian_family; then
    if command -v apt-get >/dev/null 2>&1; then
      PKG_MANAGER="apt-get"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  if os_release_matches_fedora_rhel_family; then
    if command -v dnf >/dev/null 2>&1; then
      PKG_MANAGER="dnf"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    if command -v yum >/dev/null 2>&1; then
      PKG_MANAGER="yum"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  if os_release_matches_arch_family; then
    if command -v pacman >/dev/null 2>&1; then
      PKG_MANAGER="pacman"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  if os_release_matches_suse_family; then
    if command -v zypper >/dev/null 2>&1; then
      PKG_MANAGER="zypper"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  return 1
}

select_package_manager_from_path_probe() {
  local manager=""
  for manager in "${SUPPORTED_PKG_MANAGERS[@]}"; do
    if command -v "${manager}" >/dev/null 2>&1; then
      PKG_MANAGER="${manager}"
      PKG_MANAGER_SOURCE="path_probe"
      return 0
    fi
  done
  return 1
}

detect_platform_metadata() {
  if [[ "${IS_LINUX}" -ne 1 ]]; then
    return 1
  fi

  PKG_MANAGER=""
  PKG_MANAGER_SOURCE=""
  resolve_selected_os_release_input || true
  parse_selected_os_release_fields || true

  if select_package_manager_from_os_release; then
    return 0
  fi
  if select_package_manager_from_path_probe; then
    return 0
  fi

  return 1
}

resolve_package_for_runtime_library() {
  local library="$1"

  case "${PKG_MANAGER}" in
    apt-get)
      case "${library}" in
        libseccomp) echo "libseccomp2" ;;
        *) echo "" ;;
      esac
      ;;
    dnf|yum)
      case "${library}" in
        libseccomp) echo "libseccomp" ;;
        *) echo "" ;;
      esac
      ;;
    pacman)
      case "${library}" in
        libseccomp) echo "libseccomp" ;;
        *) echo "" ;;
      esac
      ;;
    zypper)
      case "${library}" in
        libseccomp) echo "libseccomp2" ;;
        *) echo "" ;;
      esac
      ;;
    *)
      echo ""
      ;;
  esac
}

seccomp_runtime_available() {
  if command -v ldconfig >/dev/null 2>&1; then
    if ldconfig -p 2>/dev/null | grep -Eq 'libseccomp\.so(\.2)?([[:space:]]|$)'; then
      return 0
    fi
  fi

  if compgen -G '/lib*/libseccomp.so*' >/dev/null; then
    return 0
  fi
  if compgen -G '/usr/lib*/libseccomp.so*' >/dev/null; then
    return 0
  fi

  return 1
}

install_packages() {
  local packages=("$@")
  if [[ ${#packages[@]} -eq 0 ]]; then
    return
  fi

  log "Installing packages: ${packages[*]}"
  case "${PKG_MANAGER}" in
    apt-get)
      run_privileged apt-get update
      run_privileged apt-get install -y "${packages[@]}"
      ;;
    dnf)
      run_privileged dnf install -y "${packages[@]}"
      ;;
    yum)
      run_privileged yum install -y "${packages[@]}"
      ;;
    pacman)
      run_privileged pacman -Sy --noconfirm --needed "${packages[@]}"
      ;;
    zypper)
      run_privileged zypper --non-interactive install "${packages[@]}"
      ;;
    *)
      fatal "Unsupported package manager '${PKG_MANAGER}'. Install required runtime libraries manually and re-run."
      ;;
  esac
}

ensure_linux_runtime_libraries() {
  local libraries=("$@")
  local missing=()
  local library=""

  if [[ "${IS_LINUX}" -ne 1 || "${WORLD_ENABLED}" -ne 1 ]]; then
    return
  fi

  for library in "${libraries[@]}"; do
    case "${library}" in
      libseccomp)
        if ! seccomp_runtime_available; then
          missing+=("${library}")
        fi
        ;;
      *)
        warn "No runtime library probe implemented for '${library}'; install it manually if required."
        ;;
    esac
  done

  if [[ ${#missing[@]} -eq 0 ]]; then
    return
  fi

  if ! detect_platform_metadata; then
    fatal "Unable to detect a supported package manager for runtime library installation. Install ${missing[*]} manually and re-run."
  fi

  declare -A pkg_set=()
  local pkg_list pkg
  for library in "${missing[@]}"; do
    pkg_list="$(resolve_package_for_runtime_library "${library}")"
    if [[ -z "${pkg_list}" ]]; then
      fatal "No package mapping for runtime library '${library}' under ${PKG_MANAGER}. Install it manually and re-run."
    fi
    for pkg in ${pkg_list}; do
      pkg_set["${pkg}"]=1
    done
  done

  local packages=()
  for pkg in "${!pkg_set[@]}"; do
    packages+=("${pkg}")
  done

  install_packages "${packages[@]}"

  local remaining=()
  for library in "${missing[@]}"; do
    case "${library}" in
      libseccomp)
        if ! seccomp_runtime_available; then
          remaining+=("${library}")
        fi
        ;;
    esac
  done

  if [[ ${#remaining[@]} -gt 0 ]]; then
    fatal "Unable to install required runtime libraries: ${remaining[*]}. Install them manually and re-run."
  fi
}

write_host_state_metadata() {
  if [[ "${IS_LINUX}" -ne 1 ]]; then
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
  if [[ -n "${PKG_MANAGER}" && -n "${PKG_MANAGER_SOURCE}" ]]; then
    events+=("platform_os_release_id:${DETECTED_DISTRO_ID:-${DISTRO_UNKNOWN_SENTINEL}}")
    events+=("platform_os_release_id_like:${DETECTED_DISTRO_ID_LIKE:-${DISTRO_UNKNOWN_SENTINEL}}")
    events+=("platform_pkg_manager_selected:${PKG_MANAGER}")
    events+=("platform_pkg_manager_source:${PKG_MANAGER_SOURCE}")
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
timestamp = datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z")

base = {}
parsed_existing = False
if path.exists():
    try:
        with path.open() as f:
            base = json.load(f)
        parsed_existing = True
    except Exception as exc:  # noqa: BLE001
        sys.stderr.write(f"[dev-install-substrate] warning: unable to parse {path}: {exc}\n")
        base = {}

if parsed_existing and base.get("schema_version") != schema_version:
    sys.stderr.write(
        f"[dev-install-substrate] warning: unsupported schema_version {base.get('schema_version')} at {path}; rebuilding metadata\n"
    )
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
platform = host.get("platform") or {}
os_release = platform.get("os_release") or {}
pkg_manager = platform.get("pkg_manager") or {}


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
    elif kind == "platform_os_release_id" and len(parts) >= 2:
        os_release["id"] = parts[1]
    elif kind == "platform_os_release_id_like" and len(parts) >= 2:
        os_release["id_like"] = parts[1]
    elif kind == "platform_pkg_manager_selected" and len(parts) >= 2:
        pkg_manager["selected"] = parts[1]
    elif kind == "platform_pkg_manager_source" and len(parts) >= 2:
        pkg_manager["source"] = parts[1]

group["members_added"] = sorted(members)
if os_release:
    platform["os_release"] = os_release
if pkg_manager:
    platform["pkg_manager"] = pkg_manager
if platform:
    host["platform"] = platform
json.dump(base, sys.stdout, indent=2, sort_keys=True)
PY
  then
    warn "Failed to write host state metadata to ${HOST_STATE_PATH}; continuing without blocking install."
    rm -f "${tmp}" || true
    return
  fi

  if ! mv "${tmp}" "${HOST_STATE_PATH}"; then
    warn "Failed to replace host state metadata at ${HOST_STATE_PATH}; continuing without blocking install."
    rm -f "${tmp}" || true
    return
  fi
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
    warn "${invoking_user} added to ${target_group}. Current-shell access should work immediately when the Linux socket ACL bridge is healthy; if 'substrate host doctor --json' reports degraded ACL state, run 'exec newgrp ${target_group}' or start a fresh login shell."
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
    warn "systemctl not found; verify /run/substrate.sock is root:substrate 0660 and /run/substrate is root:substrate 0750 after provisioning."
    return
  fi
  local socket_unit="/etc/systemd/system/substrate-world-service.socket"
  local service_unit="/etc/systemd/system/substrate-world-service.service"
  if [[ ! -f "${socket_unit}" ]]; then
    warn "Socket unit missing at ${socket_unit}; rerun scripts/linux/world-provision.sh to install it."
    return
  fi
  if [[ ! -f "${service_unit}" ]]; then
    warn "Service unit missing at ${service_unit}; rerun scripts/linux/world-provision.sh to install it."
    return
  fi
  if grep -q '^SocketGroup=substrate' "${socket_unit}"; then
    log "substrate-world-service.socket already sets SocketGroup=substrate."
  else
    log "Updating ${socket_unit} to enforce SocketGroup=substrate (sudo may prompt)..."
    if ! run_privileged sed -i 's/^SocketGroup=.*/SocketGroup=substrate/' "${socket_unit}"; then
      warn "Failed to update ${socket_unit}; edit it manually so SocketGroup=substrate and rerun 'sudo systemctl daemon-reload'."
      return
    fi
  fi
  if grep -q '^Group=substrate$' "${service_unit}" && grep -q '^UMask=0027$' "${service_unit}"; then
    log "substrate-world-service.service already sets Group=substrate and UMask=0027."
  else
    log "Updating ${service_unit} to enforce Group=substrate and UMask=0027 (sudo may prompt)..."
    if ! run_privileged python3 - "${service_unit}" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
lines = path.read_text(encoding="utf-8").splitlines()
service_idx = next((i for i, line in enumerate(lines) if line.strip() == "[Service]"), None)
if service_idx is None:
    raise SystemExit("missing [Service] section")

group_idx = next((i for i, line in enumerate(lines) if line.startswith("Group=")), None)
umask_idx = next((i for i, line in enumerate(lines) if line.startswith("UMask=")), None)
if group_idx is not None:
    lines[group_idx] = "Group=substrate"
else:
    insert_at = next(
        (i + 1 for i, line in enumerate(lines[service_idx + 1:], start=service_idx + 1)
         if line.startswith("Environment=") or line.startswith("RestartSec=")),
        service_idx + 1,
    )
    while insert_at < len(lines) and (
        lines[insert_at].startswith("Environment=") or lines[insert_at].startswith("RestartSec=")
    ):
        insert_at += 1
    lines.insert(insert_at, "Group=substrate")
    if umask_idx is not None and umask_idx >= insert_at:
        umask_idx += 1

if umask_idx is not None:
    lines[umask_idx] = "UMask=0027"
else:
    group_idx = next(i for i, line in enumerate(lines) if line == "Group=substrate")
    lines.insert(group_idx + 1, "UMask=0027")

path.write_text("\n".join(lines) + "\n", encoding="utf-8")
PY
    then
      warn "Failed to update ${service_unit}; edit it manually so it contains Group=substrate and UMask=0027, then rerun 'sudo systemctl daemon-reload'."
      return
    fi
  fi

  log "Restarting world-service units to apply socket ownership (sudo may prompt)..."
  run_privileged systemctl stop substrate-world-service.service substrate-world-service.socket || true
  run_privileged install -d -m0750 -o root -g substrate /run/substrate || true
  run_privileged rm -f /run/substrate.sock || true
  run_privileged systemctl daemon-reload || true
  run_privileged systemctl start substrate-world-service.socket || true
  run_privileged systemctl start substrate-world-service.service || true
  log "Reloaded socket/service units so /run/substrate is root:substrate 0750 and /run/substrate.sock is recreated as root:substrate 0660."
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

find_linux_world_service_elf() {
  local root="$1"
  local target_dir="$2"
  candidate="$(find_linux_world_service "${root}" "${target_dir}")" || return 1
  local file_type
  file_type="$(file -b "${candidate}" 2>/dev/null || true)"
  if [[ -n "${file_type}" ]] && ! echo "${file_type}" | grep -qi "ELF"; then
    return 1
  fi
  printf '%s\n' "${candidate}"
  return 0
}

find_linux_substrate_gateway() {
  local root="$1"
  local target_dir="$2"
  local candidates=(
    "${root}/bin/linux/substrate-gateway"
    "${root}/bin/substrate-gateway-linux"
    "${root}/bin/substrate-gateway"
    "${root}/target/x86_64-unknown-linux-gnu/${target_dir}/substrate-gateway"
    "${root}/target/aarch64-unknown-linux-gnu/${target_dir}/substrate-gateway"
    "${root}/target/${target_dir}/substrate-gateway"
  )
  local candidate
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

path_is_repo_managed_symlink() {
  local path="$1"
  local repo_root="$2"

  if [[ ! -L "${path}" ]]; then
    return 1
  fi

  local target
  target="$(readlink "${path}" 2>/dev/null || true)"
  if [[ -z "${target}" ]]; then
    return 1
  fi

  if [[ "${target}" != /* ]]; then
    target="${path%/*}/${target}"
  fi

  local target_dir
  target_dir="$(dirname "${target}")"
  if target_dir="$(cd "${target_dir}" 2>/dev/null && pwd -P)"; then
    target="${target_dir}/$(basename "${target}")"
  fi

  case "${target}" in
    "${repo_root%/}/"*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

path_is_recorded_managed_linux_binary() {
  local path="$1"
  local manifest_path="$2"

  if [[ -z "${manifest_path}" || ! -f "${manifest_path}" ]]; then
    return 1
  fi

  grep -Fxq -- "${path}" "${manifest_path}"
}

path_is_managed_bundle_entry() {
  local path="$1"
  local repo_root="$2"
  local manifest_path="$3"

  if path_is_repo_managed_symlink "${path}" "${repo_root}"; then
    return 0
  fi

  if path_is_recorded_managed_linux_binary "${path}" "${manifest_path}"; then
    return 0
  fi

  return 1
}

stage_managed_bundle_symlink() {
  local src="$1"
  local dest="$2"
  local repo_root="$3"
  local manifest_path="$4"
  local label="$5"

  if [[ ! -e "${src}" ]]; then
    warn "${label} source missing at ${src}; leaving ${dest} unchanged."
    return 0
  fi

  mkdir -p "$(dirname "${dest}")"

  if [[ -e "${dest}" || -L "${dest}" ]]; then
    if path_is_managed_bundle_entry "${dest}" "${repo_root}" "${manifest_path}"; then
      rm -f "${dest}"
    else
      fatal "Refusing to overwrite unmanaged ${label} at ${dest}"
    fi
  fi

  ln -s "${src}" "${dest}"
  log "Linked ${label} into ${dest}"
}

stage_managed_linux_binary_copy() {
  local vm_path="$1"
  local dest_path="$2"
  local repo_root="$3"
  local manifest_path="$4"
  local label="$5"

  mkdir -p "$(dirname "${dest_path}")"

  if [[ -e "${dest_path}" || -L "${dest_path}" ]]; then
    if path_is_managed_bundle_entry "${dest_path}" "${repo_root}" "${manifest_path}"; then
      rm -f "${dest_path}"
    else
      fatal "Refusing to overwrite unmanaged ${label} at ${dest_path}"
    fi
  fi

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

# The macOS control and lifecycle executor are copied, not linked, because their exact bytes are
# later committed by the publisher-bootstrap authorization. A source-tree target symlink could
# change after installation and must not become control authority.
stage_managed_mac_control_binary_copy() {
  local src="$1"
  local dest="$2"
  local repo_root="$3"
  local manifest_path="$4"
  local label="$5"
  local binary_tmp
  local manifest_tmp
  local manifest_filter_status

  [[ -f "${src}" && -x "${src}" ]] || fatal "Expected ${label} source at ${src}"
  mkdir -p "$(dirname "${dest}")" "$(dirname "${manifest_path}")"

  if [[ -e "${dest}" || -L "${dest}" ]]; then
    if ! path_is_managed_bundle_entry "${dest}" "${repo_root}" "${manifest_path}"; then
      fatal "Refusing to overwrite unmanaged ${label} at ${dest}"
    fi
  fi

  binary_tmp="${dest}.tmp.$$"
  manifest_tmp="${manifest_path}.tmp.$$"
  if ! cp "${src}" "${binary_tmp}"; then
    rm -f "${binary_tmp}" "${manifest_tmp}"
    return 1
  fi
  if ! chmod 0755 "${binary_tmp}"; then
    rm -f "${binary_tmp}" "${manifest_tmp}"
    return 1
  fi
  if ! : > "${manifest_tmp}"; then
    rm -f "${binary_tmp}" "${manifest_tmp}"
    return 1
  fi
  if [[ -f "${manifest_path}" ]]; then
    if grep -Fxv -- "${dest}" "${manifest_path}" > "${manifest_tmp}"; then
      :
    else
      manifest_filter_status=$?
      if [[ "${manifest_filter_status}" -gt 1 ]]; then
        rm -f "${binary_tmp}" "${manifest_tmp}"
        return 1
      fi
    fi
  fi
  if ! printf '%s\n' "${dest}" >> "${manifest_tmp}"; then
    rm -f "${binary_tmp}" "${manifest_tmp}"
    return 1
  fi
  if ! mv "${manifest_tmp}" "${manifest_path}"; then
    rm -f "${binary_tmp}" "${manifest_tmp}"
    return 1
  fi
  if ! mv "${binary_tmp}" "${dest}"; then
    rm -f "${binary_tmp}"
    return 1
  fi
  log "Copied immutable ${label} into ${dest}"
}

# The macOS installer owns exactly one closed AArch64/Linux bundle before it admits direct
# bootstrap.  Build roots and linker wrappers are external to every checkout; only the four
# no-follow, mode-0755 outputs are atomically retained below the selected prefix.  A retry either
# finds the exact complete bundle or fails before replacing anything -- it never treats a partial
# or caller-selected prefix file as an authority source.
build_and_stage_mac_aarch64_lima_artifacts_v1() {
  [[ "${IS_MAC}" -eq 1 ]] || return 0

  local zig="/opt/homebrew/opt/zig/bin/zig"
  local target="aarch64-unknown-linux-gnu"
  local build_command="cargo build --locked --offline --target aarch64-unknown-linux-gnu --release -p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service -p substrate-gateway --bin substrate-gateway -p substrate --bin substrate"
  local bundle_dir="${BIN_DIR}/linux"
  local external_root linker_wrapper artifact_stage toolchain cargo_lock
  local name source digest existing_digest mode file_type
  local -a names=(substrate-lifecycle-linux world-service substrate-gateway substrate)

  [[ -x "${zig}" && ! -L "${zig}" ]] || fatal "fixed macOS AArch64 Zig compiler is absent or linked: ${zig}"
  [[ -f "${REPO_ROOT}/Cargo.lock" && ! -L "${REPO_ROOT}/Cargo.lock" ]] || fatal "Cargo.lock is absent or linked"
  toolchain="$(rustc --version)" || fatal "cannot determine fixed Rust toolchain"
  [[ "${toolchain}" == rustc\ 1.89.0\ * ]] || fatal "macOS AArch64 staging requires rustc 1.89.0"
  cargo_lock="$(shasum -a 256 -- "${REPO_ROOT}/Cargo.lock" | awk '{print $1}')" || fatal "cannot hash Cargo.lock"
  [[ "${cargo_lock}" =~ ^[0-9a-f]{64}$ ]] || fatal "Cargo.lock digest is not canonical"

  external_root="$(mktemp -d "/private/tmp/substrate-mac-aarch64-build.XXXXXX")" || fatal "cannot allocate external AArch64 build root"
  linker_wrapper="${external_root}/aarch64-linux-gnu-zig-cc"
  cat > "${linker_wrapper}" <<EOF
#!/usr/bin/env bash
linker_args=()
for arg in "\$@"; do
  [[ "\${arg}" == "--target=aarch64-unknown-linux-gnu" ]] || linker_args+=("\${arg}")
done
exec "${zig}" cc -target aarch64-linux-gnu "\${linker_args[@]}"
EOF
  chmod 0700 "${linker_wrapper}" || fatal "cannot harden fixed AArch64 linker wrapper"

  log "Building the fixed macOS Lima AArch64 Linux artifact bundle..."
  CARGO_TARGET_DIR="${external_root}/target" \
  CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="${linker_wrapper}" \
  CC_aarch64_unknown_linux_gnu="${linker_wrapper}" \
    cargo build --locked --offline --target "${target}" --release \
      -p substrate --bin substrate-lifecycle-linux \
      -p world-service --bin world-service \
      -p substrate-gateway --bin substrate-gateway \
      -p substrate --bin substrate \
    || fatal "fixed macOS AArch64 Linux artifact build failed"

  for name in "${names[@]}"; do
    source="${external_root}/target/${target}/release/${name}"
    [[ -f "${source}" && ! -L "${source}" && -x "${source}" ]] || fatal "fixed AArch64 build omitted ${name}"
    file_type="$(LC_ALL=C file -b -- "${source}" 2>/dev/null || true)"
    [[ "${file_type}" == *ELF* && ( "${file_type}" == *aarch64* || "${file_type}" == *AArch64* ) ]] \
      || fatal "fixed AArch64 build produced a non-Linux-AArch64 ${name}"
  done

  if [[ -e "${bundle_dir}" || -L "${bundle_dir}" ]]; then
    [[ -d "${bundle_dir}" && ! -L "${bundle_dir}" ]] || fatal "retained Linux bundle path is linked or not a directory"
    for name in "${names[@]}"; do
      source="${external_root}/target/${target}/release/${name}"
      digest="$(shasum -a 256 -- "${source}" | awk '{print $1}')" || fatal "cannot hash fixed ${name}"
      [[ -f "${bundle_dir}/${name}" && ! -L "${bundle_dir}/${name}" && -x "${bundle_dir}/${name}" ]] \
        || fatal "retained Linux bundle is incomplete at ${bundle_dir}/${name}"
      mode="$(stat -f '%Lp' -- "${bundle_dir}/${name}")" || fatal "cannot inspect retained ${name} mode"
      existing_digest="$(shasum -a 256 -- "${bundle_dir}/${name}" | awk '{print $1}')" || fatal "cannot hash retained ${name}"
      [[ "${mode}" == "755" && "${existing_digest}" == "${digest}" ]] \
        || fatal "retained Linux bundle contains a mismatched prior ${name}"
    done
    for source in "${bundle_dir}"/*; do
      [[ -f "${source}" && ! -L "${source}" ]] || fatal "retained Linux bundle contains an unknown entry"
      name="$(basename "${source}")"
      case "${name}" in
        substrate-lifecycle-linux|world-service|substrate-gateway|substrate) ;;
        *) fatal "retained Linux bundle contains an unknown entry: ${name}" ;;
      esac
    done
    rm -rf -- "${external_root}"
    return 0
  fi

  artifact_stage="$(mktemp -d "${BIN_DIR}/.linux-stage.XXXXXX")" || fatal "cannot allocate atomic retained Linux bundle staging"
  for name in "${names[@]}"; do
    source="${external_root}/target/${target}/release/${name}"
    cp -- "${source}" "${artifact_stage}/${name}" || fatal "cannot stage fixed ${name}"
    chmod 0755 "${artifact_stage}/${name}" || fatal "cannot set retained ${name} mode"
    [[ -f "${artifact_stage}/${name}" && ! -L "${artifact_stage}/${name}" ]] || fatal "staged ${name} is linked or not regular"
    mode="$(stat -f '%Lp' -- "${artifact_stage}/${name}")" || fatal "cannot inspect staged ${name} mode"
    digest="$(shasum -a 256 -- "${source}" | awk '{print $1}')" || fatal "cannot hash fixed ${name}"
    existing_digest="$(shasum -a 256 -- "${artifact_stage}/${name}" | awk '{print $1}')" || fatal "cannot hash staged ${name}"
    [[ "${mode}" == "755" && "${existing_digest}" == "${digest}" ]] || fatal "staged ${name} failed exact verification"
  done
  mv "${artifact_stage}" "${bundle_dir}" || fatal "cannot atomically publish retained Linux bundle"
  rm -rf -- "${external_root}"
  log "Published the exact four-artifact macOS Lima AArch64 Linux bundle."
}

# Publish the one root-owned install-time provenance record only after the prefix control binary
# and managed-copy list are durable. The direct bootstrap never consults this checkout, PATH,
# Git, or a caller value; it opens this fixed root:wheel 0444 record and remeasures the recorded
# images/tool itself. This installer path is the sole producer and uses absent-or-exact semantics.
publish_mac_publisher_install_provenance_v1() {
  local control_src="$1"
  local executor_src="$2"
  local managed_manifest="$3"
  local retained_linux_bundle="$4"
  local source_commit
  local source_tree
  local source_ref
  local review_path="${REPO_ROOT}/llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-cycle-record.json"
  local review_sha
  local limactl_path=""
  local candidate
  local control_src_sha
  local control_src_identity
  local executor_src_sha
  local executor_src_identity
  local plist_src_sha
  local plist_src_identity
  local cargo_lock_sha
  local rust_toolchain
  local fixed_build_command="cargo build --locked --offline --target aarch64-unknown-linux-gnu --release -p substrate --bin substrate-lifecycle-linux -p world-service --bin world-service -p substrate-gateway --bin substrate-gateway -p substrate --bin substrate"
  local retained_substrate_lifecycle_sha retained_substrate_lifecycle_identity
  local retained_world_service_sha retained_world_service_identity
  local retained_gateway_sha retained_gateway_identity
  local retained_substrate_sha retained_substrate_identity
  local retained_spec retained_name retained_sha_var retained_identity_var retained_path
  local retained_digest retained_identity
  local profile_template_sha256="560178a5de3b32fe2628d2ab6c6f7036e8f024509dfc5d9e9cc995c2f7df9559"
  local launch_daemon_src="${REPO_ROOT}/scripts/mac/com.substrate.lifecycle.publisher.v1.plist"

  [[ -f "${control_src}" && ! -L "${control_src}" && -x "${control_src}" ]] \
    || fatal "macOS lifecycle control managed copy is not durable"
  [[ -f "${executor_src}" && ! -L "${executor_src}" && -x "${executor_src}" ]] \
    || fatal "macOS lifecycle executor managed copy is not durable"
  [[ -f "${managed_manifest}" ]] \
    || fatal "macOS lifecycle managed-copy record is absent"
  grep -Fxq -- "${control_src}" "${managed_manifest}" \
    || fatal "macOS lifecycle control is not recorded as a managed copy"
  grep -Fxq -- "${executor_src}" "${managed_manifest}" \
    || fatal "macOS lifecycle executor is not recorded as a managed copy"
  [[ -f "${launch_daemon_src}" && ! -L "${launch_daemon_src}" ]] \
    || fatal "fixed macOS lifecycle LaunchDaemon source is absent or linked"
  [[ -f "${review_path}" && ! -L "${review_path}" ]] \
    || fatal "R5 review record is absent or linked"
  [[ -d "${retained_linux_bundle}" && ! -L "${retained_linux_bundle}" ]] \
    || fatal "retained macOS Lima Linux bundle is absent or linked"
  cargo_lock_sha="$(shasum -a 256 -- "${REPO_ROOT}/Cargo.lock" | awk '{print $1}')" \
    || fatal "cannot measure Cargo.lock for retained Linux bundle"
  rust_toolchain="$(rustc --version)" || fatal "cannot measure Rust toolchain for retained Linux bundle"
  [[ "${cargo_lock_sha}" =~ ^[0-9a-f]{64} && "${rust_toolchain}" == rustc\ 1.89.0\ * ]] \
    || fatal "retained Linux bundle build provenance is not canonical"
  for retained_spec in \
    "substrate-lifecycle-linux:retained_substrate_lifecycle_sha:retained_substrate_lifecycle_identity" \
    "world-service:retained_world_service_sha:retained_world_service_identity" \
    "substrate-gateway:retained_gateway_sha:retained_gateway_identity" \
    "substrate:retained_substrate_sha:retained_substrate_identity"; do
    IFS=':' read -r retained_name retained_sha_var retained_identity_var <<<"${retained_spec}"
    retained_path="${retained_linux_bundle}/${retained_name}"
    [[ -f "${retained_path}" && ! -L "${retained_path}" && -x "${retained_path}" ]] \
      || fatal "retained Linux bundle member is absent or linked: ${retained_name}"
    [[ "$(stat -f '%Lp' -- "${retained_path}")" == "755" ]] \
      || fatal "retained Linux bundle member mode is not 0755: ${retained_name}"
    retained_digest="$(shasum -a 256 -- "${retained_path}" | awk '{print $1}')" \
      || fatal "cannot measure retained Linux bundle member: ${retained_name}"
    retained_identity="$(stat -f 'dev:%d:ino:%i' -- "${retained_path}")" \
      || fatal "cannot identify retained Linux bundle member: ${retained_name}"
    printf -v "${retained_sha_var}" '%s' "${retained_digest}"
    printf -v "${retained_identity_var}" '%s' "${retained_identity}"
  done

  # Bind the user-prefix copies before privilege elevation.  The privileged side receives these
  # exact source digests/identities, rechecks them, and verifies the copied root helper before
  # provenance is derived; it must never silently bless a path swapped between check and install.
  control_src_sha="$(shasum -a 256 -- "${control_src}" | awk '{print $1}')" \
    || fatal "cannot measure macOS lifecycle control source"
  control_src_identity="$(stat -f 'dev:%d:ino:%i' -- "${control_src}")" \
    || fatal "cannot identify macOS lifecycle control source"
  executor_src_sha="$(shasum -a 256 -- "${executor_src}" | awk '{print $1}')" \
    || fatal "cannot measure macOS lifecycle executor source"
  executor_src_identity="$(stat -f 'dev:%d:ino:%i' -- "${executor_src}")" \
    || fatal "cannot identify macOS lifecycle executor source"
  plist_src_sha="$(shasum -a 256 -- "${launch_daemon_src}" | awk '{print $1}')" \
    || fatal "cannot measure macOS lifecycle LaunchDaemon source"
  plist_src_identity="$(stat -f 'dev:%d:ino:%i' -- "${launch_daemon_src}")" \
    || fatal "cannot identify macOS lifecycle LaunchDaemon source"
  [[ "${control_src_sha}" =~ ^[0-9a-f]{64}$ && "${executor_src_sha}" =~ ^[0-9a-f]{64}$ && \
     "${plist_src_sha}" =~ ^[0-9a-f]{64}$ && \
     "${control_src_identity}" == dev:*:ino:* && "${executor_src_identity}" == dev:*:ino:* && \
     "${plist_src_identity}" == dev:*:ino:* ]] \
    || fatal "macOS lifecycle source measurement is not canonical"

  source_commit="$(git rev-parse HEAD)" || fatal "cannot resolve installer source commit"
  source_tree="$(git rev-parse HEAD^{tree})" || fatal "cannot resolve installer source tree"
  source_ref="$(git symbolic-ref -q HEAD || printf '%s' detached)"
  review_sha="$(shasum -a 256 "${review_path}" | awk '{print $1}')" \
    || fatal "cannot measure R5 review record"
  limactl_path="$(type -P -- limactl 2>/dev/null || true)"
  [[ -n "${limactl_path}" && -f "${limactl_path}" && ! -L "${limactl_path}" && -x "${limactl_path}" ]] \
    || fatal "installer cannot resolve one fixed no-follow limactl image for macOS publisher provenance"

  run_privileged sh -ceu '
set -eu
PATH=/usr/bin:/bin:/usr/sbin:/sbin
export PATH
control_src="$1"
executor_src="$2"
plist_src="$3"
limactl_path="$4"
source_commit="$5"
source_tree="$6"
source_ref="$7"
review_sha="$8"
host_context_commitment="$9"
selected_prefix="${10}"
profile_template_sha256="${11}"
control_expected_sha="${12}"
control_expected_identity="${13}"
executor_expected_sha="${14}"
executor_expected_identity="${15}"
retained_linux_bundle="${16}"
cargo_lock_sha="${17}"
rust_toolchain="${18}"
fixed_build_command="${19}"
retained_substrate_lifecycle_sha="${20}"
retained_substrate_lifecycle_identity="${21}"
retained_world_service_sha="${22}"
retained_world_service_identity="${23}"
retained_gateway_sha="${24}"
retained_gateway_identity="${25}"
retained_substrate_sha="${26}"
retained_substrate_identity="${27}"
installer_account="${28}"
installer_uid="${29}"
plist_expected_sha="${30}"
plist_expected_identity="${31}"
provenance_path="/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json"
executor_path="/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1"
plist_path="/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist"
case "$installer_uid" in
  ""|*[!0-9]*) printf "bound installer UID is not canonical\n" >&2; exit 1 ;;
esac
test "$installer_uid" -ne 0 || { printf "bound installer UID must be non-root\n" >&2; exit 1; }
test "$(/usr/bin/id -u "$installer_account")" = "$installer_uid" && \
  test "$(/usr/bin/id -un "$installer_uid")" = "$installer_account" || {
    printf "bound installer account and UID do not resolve to one principal\n" >&2; exit 1;
  }
for path in "$control_src" "$executor_src" "$plist_src" "$limactl_path"; do
  test -f "$path" && test ! -L "$path" || { printf "retained install input is linked or absent: %s\n" "$path" >&2; exit 1; }
done
sha() { shasum -a 256 "$1" | awk "{print \$1}"; }
identity() { stat -f "dev:%d:ino:%i" "$1"; }
test -d "$retained_linux_bundle" && test ! -L "$retained_linux_bundle" || {
  printf "retained Linux bundle is absent or linked\n" >&2; exit 1;
}
measure_retained_linux_artifact() {
  name="$1"
  expected_sha="$2"
  expected_identity="$3"
  candidate="$retained_linux_bundle/$name"
  test -f "$candidate" && test ! -L "$candidate" && test -x "$candidate" || {
    printf "retained Linux artifact is absent or linked: %s\n" "$name" >&2; exit 1;
  }
  test "$(stat -f '%Lp' "$candidate")" = 755 || {
    printf "retained Linux artifact mode is not 0755: %s\n" "$name" >&2; exit 1;
  }
  test "$(sha "$candidate")" = "$expected_sha" && test "$(identity "$candidate")" = "$expected_identity" || {
    printf "retained Linux artifact changed before root provenance publication: %s\n" "$name" >&2; exit 1;
  }
}
measure_retained_linux_artifact substrate-lifecycle-linux "$retained_substrate_lifecycle_sha" "$retained_substrate_lifecycle_identity"
measure_retained_linux_artifact world-service "$retained_world_service_sha" "$retained_world_service_identity"
measure_retained_linux_artifact substrate-gateway "$retained_gateway_sha" "$retained_gateway_identity"
measure_retained_linux_artifact substrate "$retained_substrate_sha" "$retained_substrate_identity"
test "$(sha "$control_src")" = "$control_expected_sha" && \
  test "$(identity "$control_src")" = "$control_expected_identity" || {
    printf "retained control source changed before privileged provenance copy\n" >&2; exit 1;
  }
test "$(sha "$executor_src")" = "$executor_expected_sha" && \
  test "$(identity "$executor_src")" = "$executor_expected_identity" || {
    printf "retained executor source changed before privileged provenance copy\n" >&2; exit 1;
  }
test "$(sha "$plist_src")" = "$plist_expected_sha" && \
  test "$(identity "$plist_src")" = "$plist_expected_identity" || {
    printf "retained LaunchDaemon source changed before privileged publication\n" >&2; exit 1;
  }
require_root_owned_immutable_path() {
  retained_path="$1"
  case "$retained_path" in
    /*) ;;
    *) printf "retained limactl path is not absolute\n" >&2; exit 1 ;;
  esac
  candidate="$retained_path"
  while :; do
    test -e "$candidate" && test ! -L "$candidate" || {
      printf "retained limactl path segment is linked or absent: %s\n" "$candidate" >&2
      exit 1
    }
    owner="$(stat -f '%u' "$candidate")"
    mode="$(stat -f '%Lp' "$candidate")"
    test "$owner" -eq 0 && test $((0$mode & 022)) -eq 0 || {
      printf "retained limactl path segment is not root-owned immutable state: %s\n" "$candidate" >&2
      exit 1
    }
    if test "$candidate" = "$retained_path"; then
      test -f "$candidate" || { printf "retained limactl is not a regular file\n" >&2; exit 1; }
    else
      test -d "$candidate" || { printf "retained limactl parent is not a directory\n" >&2; exit 1; }
    fi
    test "$candidate" = / && break
    candidate="${candidate%/*}"
    test -n "$candidate" || candidate=/
  done
}
require_root_owned_immutable_path "$limactl_path"
test -d /var/empty && test ! -L /var/empty || {
  printf "privileged limactl HOME is absent or linked: /var/empty\n" >&2; exit 1;
}
limactl_home_owner="$(stat -f '%u' /var/empty)"
limactl_home_mode="$(stat -f '%Lp' /var/empty)"
test "$limactl_home_owner" -eq 0 && test $((0$limactl_home_mode & 022)) -eq 0 || {
  printf "privileged limactl HOME is not root-controlled state: /var/empty\n" >&2; exit 1;
}
fixed_present_count=0
for fixed_path in "$executor_path" "$plist_path" "$provenance_path"; do
  if test -e "$fixed_path" || test -L "$fixed_path"; then
    fixed_present_count=$((fixed_present_count + 1))
  fi
done
test "$fixed_present_count" -eq 0 || test "$fixed_present_count" -eq 3 || {
  printf "publisher fixed paths are a partial ambiguous prestate; preserving all state\n" >&2
  exit 1
}
if test "$fixed_present_count" -eq 0; then
  "$control_src" publisher-service-state-preflight-absent >/dev/null || {
    printf "fresh publisher file publication requires a definitely absent system service\n" >&2
    exit 1
  }
fi
ensure_root_directory() {
  directory="$1"
  required_mode="$2"
  if test -e "$directory" || test -L "$directory"; then
    test -d "$directory" && test ! -L "$directory" && \
      test "$(stat -f '%u' "$directory")" -eq 0 && \
      test "$(stat -f '%g' "$directory")" -eq 0 && \
      test "$(stat -f '%Lp' "$directory")" = "$required_mode" || {
        printf "publisher fixed directory is foreign or mismatched: %s\n" "$directory" >&2
        exit 1
      }
  else
    install -d -o root -g wheel -m "0$required_mode" "$directory"
  fi
}
ensure_root_directory /Library/PrivilegedHelperTools 755
ensure_root_directory /Library/LaunchDaemons 755
ensure_root_directory "/Library/Application Support/Substrate" 755
ensure_root_directory "/Library/Application Support/Substrate/lifecycle" 755
if test "$fixed_present_count" -eq 0; then
  install -o root -g wheel -m 0755 "$executor_src" "$executor_path"
  install -o root -g wheel -m 0644 "$plist_src" "$plist_path"
else
  for fixed_spec in "$executor_path:$executor_expected_sha:755" "$plist_path:$plist_expected_sha:644"; do
    IFS=: read -r fixed_path fixed_sha fixed_mode <<EOF
$fixed_spec
EOF
    test -f "$fixed_path" && test ! -L "$fixed_path" && \
      test "$(stat -f '%l' "$fixed_path")" -eq 1 && \
      test "$(stat -f '%u' "$fixed_path")" -eq 0 && \
      test "$(stat -f '%g' "$fixed_path")" -eq 0 && \
      test "$(stat -f '%Lp' "$fixed_path")" = "$fixed_mode" && \
      test "$(sha "$fixed_path")" = "$fixed_sha" || {
        printf "publisher fixed file is foreign or mismatched: %s\n" "$fixed_path" >&2
        exit 1
      }
  done
fi
test "$(sha "$executor_path")" = "$executor_expected_sha" || {
  printf "privileged executor copy does not match pre-elevation digest\n" >&2; exit 1;
}
cdhash() { codesign -d -vvv -- "$1" 2>&1 | sed -n "s/^CDHash=//p" | head -n 1; }
designated_requirement() { codesign -d -r- -- "$1" 2>&1 | sed -n "s/^designated => //p" | head -n 1; }
canonical_code_requirement() {
  image_path="$1"
  measured_cdhash="$2"
  test "${#measured_cdhash}" -eq 40 && test -z "$(printf "%s" "$measured_cdhash" | tr -d "0-9a-f")" || {
    printf "cannot derive a canonical code requirement from the measured CDHash: %s\n" "$image_path" >&2
    return 1
  }
  image_requirement="$(designated_requirement "$image_path")"
  if test -z "$image_requirement"; then
    image_requirement="cdhash H\"${measured_cdhash}\""
  fi
  codesign --verify --strict "-R=${image_requirement}" -- "$image_path" >/dev/null 2>&1 || {
    printf "code requirement does not match the exact signed image: %s\n" "$image_path" >&2
    return 1
  }
  printf "%s\n" "$image_requirement"
}
canonical_control_code_requirement() {
  image_path="$1"
  measured_cdhash="$2"
  test "${#measured_cdhash}" -eq 40 && test -z "$(printf "%s" "$measured_cdhash" | tr -d "0-9a-f")" || {
    printf "cannot derive a canonical control requirement from the measured CDHash: %s\n" "$image_path" >&2
    return 1
  }
  adhoc_requirement="cdhash H\"${measured_cdhash}\""
  production_requirement="anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\" and cdhash H\"${measured_cdhash}\""
  image_requirement="$(designated_requirement "$image_path")"
  case "$image_requirement" in
    "") image_requirement="$adhoc_requirement" ;;
    "$adhoc_requirement"|"$production_requirement") ;;
    *)
      printf "control image designated requirement is not one of the two closed canonical forms: %s\n" "$image_path" >&2
      return 1
      ;;
  esac
  codesign --verify --strict "-R=${image_requirement}" -- "$image_path" >/dev/null 2>&1 || {
    printf "control code requirement does not match the exact signed image: %s\n" "$image_path" >&2
    return 1
  }
  printf "%s\n" "$image_requirement"
}
control_cdhash="$(cdhash "$control_src")"
executor_cdhash="$(cdhash "$executor_path")"
lima_cdhash="$(cdhash "$limactl_path")"
control_requirement="$(canonical_control_code_requirement "$control_src" "$control_cdhash")"
executor_requirement="$(canonical_code_requirement "$executor_path" "$executor_cdhash")"
lima_requirement="$(canonical_code_requirement "$limactl_path" "$lima_cdhash")"
test "${#control_cdhash}" -eq 40 && test "${#executor_cdhash}" -eq 40 && test "${#lima_cdhash}" -eq 40
test -n "$control_requirement" && test -n "$executor_requirement" && test -n "$lima_requirement"
lima_version="$(/usr/bin/sudo -u "#${installer_uid}" -- /usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin HOME=/var/empty "$limactl_path" --version)"
test -n "$lima_version"
plist_sha="$(sha "$plist_path")"
tmp="${provenance_path}.tmp.$$"
python3 - "$tmp" "$source_commit" "$source_tree" "$source_ref" "$review_sha" "$host_context_commitment" "$selected_prefix" \
  "$control_expected_sha" "$control_expected_identity" "$control_cdhash" "$control_requirement" \
  "$(sha "$executor_path")" "$(identity "$executor_path")" "$executor_cdhash" "$executor_requirement" \
  "$plist_sha" "$limactl_path" "$(sha "$limactl_path")" "$(identity "$limactl_path")" "$lima_cdhash" "$lima_requirement" "$lima_version" "$profile_template_sha256" \
  "$cargo_lock_sha" "$rust_toolchain" "$fixed_build_command" \
  "$retained_substrate_lifecycle_sha" "$retained_substrate_lifecycle_identity" \
  "$retained_world_service_sha" "$retained_world_service_identity" \
  "$retained_gateway_sha" "$retained_gateway_identity" \
  "$retained_substrate_sha" "$retained_substrate_identity" <<"PY"
import json
import sys
(
    output, source_commit, source_tree, source_ref, review_sha, hcc, prefix,
    control_sha, control_identity, control_cdhash, control_requirement,
    executor_sha, executor_identity, executor_cdhash, executor_requirement,
    plist_sha, lima_path, lima_sha, lima_identity, lima_cdhash, lima_requirement,
    lima_version, profile_template_sha, cargo_lock_sha, rust_toolchain, build_command,
    lifecycle_sha, lifecycle_identity, world_service_sha, world_service_identity,
    gateway_sha, gateway_identity, substrate_sha, substrate_identity,
) = sys.argv[1:]
target = "aarch64-apple-darwin"
def image(sha, identity, cdhash, requirement):
    return {"target_triple": target, "artifact_sha256": sha, "physical_identity": identity, "code_identity": "cdhash:" + cdhash, "code_requirement": requirement}
record = {
    "schema_owner": "substrate.mac-publisher-install-provenance", "schema_version": 2,
    "source_commit": source_commit, "source_tree": source_tree, "source_ref": source_ref,
    "review_record_sha256": review_sha, "host_context_commitment": hcc, "selected_host_prefix": prefix,
    "control_authority": {"schema_owner": "substrate.mac-publisher-control-authority", "schema_version": 1, "control_binary": "substrate-lifecycle-control", "source_commit": source_commit, "source_tree": source_tree, "source_ref": source_ref, "target_triple": target, "artifact_sha256": control_sha, "designated_requirement": control_requirement},
    "control_image": image(control_sha, control_identity, control_cdhash, control_requirement),
    "executor_image": image(executor_sha, executor_identity, executor_cdhash, executor_requirement),
    "launch_daemon_plist_sha256": plist_sha,
    "lima_tool": {"absolute_path": lima_path, "image": image(lima_sha, lima_identity, lima_cdhash, lima_requirement), "version": lima_version},
    "retained_linux_artifacts": [
        {"logical_role": "mac.lima.publisher-executor", "retained_relative_path": "bin/linux/substrate-lifecycle-linux", "cargo_package": "substrate", "cargo_binary": "substrate-lifecycle-linux", "target_triple": "aarch64-unknown-linux-gnu", "cargo_lock_sha256": cargo_lock_sha, "toolchain": rust_toolchain, "build_command": build_command, "artifact_sha256": lifecycle_sha, "physical_identity": lifecycle_identity, "mode": "0755"},
        {"logical_role": "mac.lima.guest-binary(substrate-world-service)", "retained_relative_path": "bin/linux/world-service", "cargo_package": "world-service", "cargo_binary": "world-service", "target_triple": "aarch64-unknown-linux-gnu", "cargo_lock_sha256": cargo_lock_sha, "toolchain": rust_toolchain, "build_command": build_command, "artifact_sha256": world_service_sha, "physical_identity": world_service_identity, "mode": "0755"},
        {"logical_role": "mac.lima.guest-binary(substrate-gateway)", "retained_relative_path": "bin/linux/substrate-gateway", "cargo_package": "substrate-gateway", "cargo_binary": "substrate-gateway", "target_triple": "aarch64-unknown-linux-gnu", "cargo_lock_sha256": cargo_lock_sha, "toolchain": rust_toolchain, "build_command": build_command, "artifact_sha256": gateway_sha, "physical_identity": gateway_identity, "mode": "0755"},
        {"logical_role": "mac.lima.guest-binary(substrate)", "retained_relative_path": "bin/linux/substrate", "cargo_package": "substrate", "cargo_binary": "substrate", "target_triple": "aarch64-unknown-linux-gnu", "cargo_lock_sha256": cargo_lock_sha, "toolchain": rust_toolchain, "build_command": build_command, "artifact_sha256": substrate_sha, "physical_identity": substrate_identity, "mode": "0755"},
    ],
    "profile_template_algorithm": "substrate.mac-lima-stage-one-profile-template", "profile_template_version": 1, "profile_template_sha256": profile_template_sha,
}
with open(output, "w", encoding="utf-8", newline="") as fh:
    fh.write(json.dumps(record, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
PY
chown root:wheel "$tmp"
chmod 0444 "$tmp"
sync "$tmp"
if test -e "$provenance_path" || test -L "$provenance_path"; then
  test -f "$provenance_path" && test ! -L "$provenance_path" || { rm -f "$tmp"; exit 1; }
  cmp -s "$tmp" "$provenance_path" || { rm -f "$tmp"; exit 1; }
  rm -f "$tmp"
else
  mv "$tmp" "$provenance_path"
fi
sync "$(dirname "$provenance_path")"
' mac-publisher-install-provenance \
    "${control_src}" "${executor_src}" "${launch_daemon_src}" "${limactl_path}" \
    "${source_commit}" "${source_tree}" "${source_ref}" "${review_sha}" \
    "${INSTALL_BOOTSTRAP_COMMITMENT}" "${PREFIX}" "${profile_template_sha256}" \
    "${control_src_sha}" "${control_src_identity}" "${executor_src_sha}" "${executor_src_identity}" \
    "${retained_linux_bundle}" "${cargo_lock_sha}" "${rust_toolchain}" "${fixed_build_command}" \
    "${retained_substrate_lifecycle_sha}" "${retained_substrate_lifecycle_identity}" \
    "${retained_world_service_sha}" "${retained_world_service_identity}" \
    "${retained_gateway_sha}" "${retained_gateway_identity}" \
    "${retained_substrate_sha}" "${retained_substrate_identity}" \
    "${INSTALL_BOOTSTRAP_ACCOUNT}" "${INSTALL_BOOTSTRAP_UID}" \
    "${plist_src_sha}" "${plist_src_identity}" \
    || fatal "failed to publish root-owned exact macOS publisher install provenance"
}

clear_managed_prefix_linux_binary_cache() {
  if [[ ! -f "${MANAGED_MAC_LINUX_BINARIES_PATH}" ]]; then
    return 0
  fi

  while IFS= read -r cached_path; do
    case "${cached_path}" in
      "${BIN_DIR}/linux/substrate"|\
      "${BIN_DIR}/linux/world-service"|\
      "${BIN_DIR}/linux/substrate-gateway")
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
  stage_managed_linux_binary_copy "${vm_path}" "${dest_path}" "${PREFIX}" "${MANAGED_MAC_LINUX_BINARIES_PATH}" "${label}"
}

link_prefix_lima_socket() {
  local default_socket="${HOME}/.substrate/sock/agent.sock"
  local prefix_socket_dir="${PREFIX}/sock"
  local prefix_socket="${prefix_socket_dir}/agent.sock"

  if [[ ! -S "${default_socket}" ]]; then
    warn "Expected managed Lima host socket at ${default_socket}, but it is not present."
    return 1
  fi

  mkdir -p "${prefix_socket_dir}"
  ln -sfn "${default_socket}" "${prefix_socket}"
  log "Linked managed Lima host socket into ${prefix_socket}"
}

verify_prefix_linux_bundle() {
  local missing_status=0
  local binary path
  if [[ "$#" -eq 0 ]]; then
    set -- substrate world-service substrate-gateway
  fi
  for binary in "$@"; do
    path="${BIN_DIR}/linux/${binary}"
    if ! is_linux_elf "${path}"; then
      warn "Expected cached Linux ${binary} at ${path}, but it is missing or not a Linux ELF."
      missing_status=1
    fi
  done
  return "${missing_status}"
}

stage_dev_world_runtime_bundle() {
  local prefix_root="$1"
  local repo_root="$2"
  local target_dir="$3"
  local scripts_substrate_dir="${prefix_root%/}/scripts/substrate"
  local scripts_mac_dir="${prefix_root%/}/scripts/mac"
  local scripts_mac_lima_dir="${scripts_mac_dir}/lima"
  local bin_linux_dir="${prefix_root%/}/bin/linux"
  mkdir -p "${scripts_substrate_dir}" "${scripts_mac_dir}" "${scripts_mac_lima_dir}"
  if [[ "${IS_MAC}" -ne 1 ]]; then
    mkdir -p "${bin_linux_dir}"
  fi

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
    stage_managed_bundle_symlink "${src}" "${dest}" "${repo_root}" "${MANAGED_MAC_LINUX_BINARIES_PATH}" "runtime bundle artifact"
  done

  # macOS owns the closed copied AArch64 bundle before direct bootstrap.  This older developer
  # convenience bridge must not create, replace, or symlink anything below bin/linux there.
  if [[ "${IS_MAC}" -eq 1 ]]; then
    return 0
  fi

  local linux_cli
  linux_cli="$(find_linux_substrate_cli "${repo_root}" "${target_dir}")" || true
  if [[ -n "${linux_cli:-}" ]]; then
    stage_managed_bundle_symlink "${linux_cli}" "${bin_linux_dir}/substrate" "${repo_root}" "${MANAGED_MAC_LINUX_BINARIES_PATH}" "Linux substrate CLI"
  else
    warn "Linux substrate CLI not available; leaving ${bin_linux_dir}/substrate unchanged."
  fi

  local linux_agent
  linux_agent="$(find_linux_world_service_elf "${repo_root}" "${target_dir}")" || true
  if [[ -n "${linux_agent:-}" ]]; then
    stage_managed_bundle_symlink "${linux_agent}" "${bin_linux_dir}/world-service" "${repo_root}" "${MANAGED_MAC_LINUX_BINARIES_PATH}" "Linux world-service"
  else
    warn "Linux world-service not available; leaving ${bin_linux_dir}/world-service unchanged."
  fi

  local linux_gateway
  linux_gateway="$(find_linux_substrate_gateway "${repo_root}" "${target_dir}")" || true
  if [[ -n "${linux_gateway:-}" ]]; then
    stage_managed_bundle_symlink "${linux_gateway}" "${bin_linux_dir}/substrate-gateway" "${repo_root}" "${MANAGED_MAC_LINUX_BINARIES_PATH}" "Linux substrate-gateway"
  else
    warn "Linux substrate-gateway not available; leaving ${bin_linux_dir}/substrate-gateway unchanged."
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
  local -a binaries=("substrate" "substrate-shim" "substrate-forwarder" "host-proxy" "world-service" "substrate-gateway")
  for binary in "${binaries[@]}"; do
    local src="${src_root}/${binary}"
    local dest="${dest_bin}/${binary}"
    if [[ -x "${src}" ]]; then
      ln -sfn "${src}" "${dest}"
      if [[ "${binary}" == "world-service" ]]; then
        ln -sfn "${src}" "${dest_bin}/linux/world-service"
        ln -sfn "${src}" "${dest_bin}/world-service-linux"
      elif [[ "${binary}" == "substrate-gateway" ]]; then
        ln -sfn "${src}" "${dest_bin}/linux/substrate-gateway"
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
[dev-install-substrate] loginctl not found. To keep the socket-activated world-service alive
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

PREFIX=""
PREFIX_DECLARED=0
INSTALL_BOOTSTRAP_CONTEXT_V1=""
PROFILE="debug"
DEPLOY_SHIMS=1
WORLD_ENABLED=1
ANCHOR_MODE="workspace"
ANCHOR_PATH=""
WORLD_CAGED=1
VERSION_LABEL="dev"
ENABLE_WORLD_NETFILTER=0
PROVISION_AGENT_RUNTIME=""
IS_LINUX=0
IS_MAC=0
IS_WSL=0
HOST_STATE_PATH=""
HOST_STATE_GROUP_EXISTED=""
HOST_STATE_GROUP_CREATED=0
HOST_STATE_ADDED_USERS=()
HOST_STATE_LINGER_ENTRIES=()
OS_RELEASE_SELECTED_PATH=""
OS_RELEASE_INPUT_STATE="unavailable"
DETECTED_DISTRO_ID="${DISTRO_UNKNOWN_SENTINEL}"
DETECTED_DISTRO_ID_LIKE="${DISTRO_UNKNOWN_SENTINEL}"
PKG_MANAGER=""
PKG_MANAGER_SOURCE=""
if [[ "$(uname -s)" == "Linux" ]]; then
  IS_LINUX=1
  if grep -qi microsoft /proc/version 2>/dev/null; then
    IS_WSL=1
  fi
fi
if [[ "$(uname -s)" == "Darwin" ]]; then
  IS_MAC=1
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      [[ $# -ge 2 ]] || fatal "--prefix requires a value"
      PREFIX="$2"
      PREFIX_DECLARED=1
      shift 2
      ;;
    --install-bootstrap-context-v1)
      [[ $# -ge 2 ]] || fatal "--install-bootstrap-context-v1 requires a value"
      INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
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
    --provision-agent-runtime)
      [[ $# -ge 2 ]] || fatal "--provision-agent-runtime requires a value"
      PROVISION_AGENT_RUNTIME="$2"
      shift 2
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

resolve_install_bootstrap_context "${PREFIX_DECLARED}" "${PREFIX}" "${INSTALL_BOOTSTRAP_CONTEXT_V1}"

validate_agent_runtime_provision_request

if [[ "${IS_WSL}" -eq 1 && "${WORLD_ENABLED}" -eq 1 ]]; then
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "WSL world provisioning is intentionally fail-closed in this slice because the WSL helper path is not aligned with the Linux/macOS placement contract. Re-run with --no-world for a CLI-only dev install inside WSL." >&2
  exit 4
fi

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
BUILD_FLAGS=(build -p substrate --bin substrate --bin substrate-shim -p substrate-gateway --bin substrate-gateway)
if [[ "${IS_MAC}" -eq 1 ]]; then
  BUILD_FLAGS+=(--bin substrate-lifecycle-control --bin substrate-lifecycle-macos)
fi
if [[ "${PROFILE}" == "release" ]]; then
  BUILD_FLAGS+=(--release)
fi

log "Building Substrate (${PROFILE})..."
cargo "${BUILD_FLAGS[@]}"

# Linux dev-install always builds world-service so the accepted staging bridge can
# be refreshed even when --no-world skips provisioning.
if [[ "${IS_LINUX}" -eq 1 ]]; then
  log "Building world-service (${PROFILE})..."
  if [[ "${PROFILE}" == "release" ]]; then
    cargo build -p world-service --release
  else
    cargo build -p world-service
  fi
fi

SUBSTRATE_BIN="${REPO_ROOT}/target/${TARGET_DIR}/substrate"
if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
  fatal "Expected substrate binary at ${SUBSTRATE_BIN}, but it was not found."
fi

bootstrap_private_substrate_home "${SUBSTRATE_BIN}"

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
MANAGED_MAC_CONTROL_BINARIES_PATH="${MANAGED_STATE_DIR}/mac-control-binaries.txt"

mkdir -p "${PREFIX}" "${BIN_DIR}" "${VERSION_CONFIG_DIR}"
if [[ "${IS_LINUX}" -eq 1 ]]; then
  clear_managed_prefix_linux_binary_cache
fi

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
cp "${VERSION_CONFIG_DIR}/manager_hooks.yaml" "${PREFIX%/}/manager_hooks.yaml.tmp"
mv "${PREFIX%/}/manager_hooks.yaml.tmp" "${PREFIX%/}/manager_hooks.yaml"
chmod 0644 "${PREFIX%/}/manager_hooks.yaml" || true

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
  if ! SHIM_ORIGINAL_PATH="${PATH}" "${SUBSTRATE_BIN}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    --shim-deploy; then
    fatal "Shim deployment failed"
  fi
  shim_note="Dev shims deployed to ${SHIMS_DIR}."
else
  warn "Shim deployment skipped (--no-shims)."
  shim_note="Shims were not deployed (--no-shims). Binaries are available under ${BIN_DIR}."
fi

for binary in substrate substrate-shim substrate-forwarder host-proxy world-service substrate-gateway; do
  src="${REPO_ROOT}/target/${TARGET_DIR}/${binary}"
  if [[ -x "${src}" ]]; then
    stage_managed_bundle_symlink "${src}" "${BIN_DIR}/${binary}" "${REPO_ROOT}" "" "host binary ${binary}"
  elif [[ -x "${src}.exe" ]]; then
    stage_managed_bundle_symlink "${src}.exe" "${BIN_DIR}/${binary}.exe" "${REPO_ROOT}" "" "host binary ${binary}.exe"
  fi
done

if [[ "${IS_MAC}" -eq 1 ]]; then
  build_and_stage_mac_aarch64_lima_artifacts_v1
  mkdir -p "${MANAGED_STATE_DIR}"
  for binary in substrate-lifecycle-control substrate-lifecycle-macos; do
    src="${REPO_ROOT}/target/${TARGET_DIR}/${binary}"
    stage_managed_mac_control_binary_copy \
      "${src}" "${BIN_DIR}/${binary}" "${REPO_ROOT}" \
      "${MANAGED_MAC_CONTROL_BINARIES_PATH}" "macOS ${binary}"
  done
  publish_mac_publisher_install_provenance_v1 \
    "${BIN_DIR}/substrate-lifecycle-control" \
    "${BIN_DIR}/substrate-lifecycle-macos" \
    "${MANAGED_MAC_CONTROL_BINARIES_PATH}" \
    "${BIN_DIR}/linux"
fi

# Provide substrate-world-service alias so CLI discovery works without extra config.
world_service_src="${REPO_ROOT}/target/${TARGET_DIR}/world-service"
if [[ -x "${world_service_src}" ]]; then
  stage_managed_bundle_symlink "${world_service_src}" "${BIN_DIR}/substrate-world-service" "${REPO_ROOT}" "" "host binary substrate-world-service"
elif [[ -x "${world_service_src}.exe" ]]; then
  stage_managed_bundle_symlink "${world_service_src}.exe" "${BIN_DIR}/substrate-world-service.exe" "${REPO_ROOT}" "" "host binary substrate-world-service.exe"
fi

if [[ -d "${REPO_ROOT}/target" ]]; then
  version_root="$(cd "${REPO_ROOT}/target" && pwd)"
  cleanup_legacy_world_enable_helper_bridge "${version_root}" "${REPO_ROOT}"
  ensure_release_bin_bridge "${version_root}" "${TARGET_DIR}"
fi
stage_dev_world_runtime_bundle "${PREFIX}" "${REPO_ROOT}" "${TARGET_DIR}"

if [[ "${WORLD_ENABLED}" -eq 1 && "${IS_LINUX}" -eq 1 ]]; then
  ensure_linux_runtime_libraries libseccomp
  use_noninteractive_world_provision=0
  if [[ ${EUID} -ne 0 ]] && command -v sudo >/dev/null 2>&1; then
    true_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- true 2>/dev/null || true)"
    privileged_env_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- env 2>/dev/null || true)"
    if [[ -n "${true_path}" && -n "${privileged_env_path}" ]] \
        && sudo -n -- "${privileged_env_path}" -i "PATH=${PRIVILEGED_TOOL_PATH}" "${true_path}" >/dev/null 2>&1; then
      use_noninteractive_world_provision=1
      log "Detected non-interactive sudo for world provisioning."
    else
      log "Checking sudo access for world provisioning (you may be prompted)..."
      if [[ ! -t 0 && ! -t 1 && ! -t 2 ]]; then
        WORLD_PROVISION_FAILED=1
        fail_closed_world_provisioning_for_runtime_request \
          "sudo is required, but no interactive prompt is available in this session." \
          "Re-run from a TTY, pre-authenticate with 'sudo -v', or omit --provision-agent-runtime."
        WORLD_ENABLED=0
        write_install_metadata "${WORLD_ENABLED}"
        write_env_sh_script "${WORLD_ENABLED}"
        write_manager_env_script "${WORLD_ENABLED}"
        warn "World provisioning requires sudo, but no interactive prompt is available in this session."
        warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures. Re-run provisioning from a TTY, pre-authenticate with 'sudo -v', or run with --no-world."
      elif ! sudo -v; then
        WORLD_PROVISION_FAILED=1
        fail_closed_world_provisioning_for_runtime_request \
          "unable to cache sudo credentials for world provisioning." \
          "Re-run after 'sudo -v' succeeds, or omit --provision-agent-runtime."
        WORLD_ENABLED=0
        write_install_metadata "${WORLD_ENABLED}"
        write_env_sh_script "${WORLD_ENABLED}"
        write_manager_env_script "${WORLD_ENABLED}"
        warn "Unable to cache sudo credentials; world-service service not provisioned."
        warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures. Re-run provisioning, then run 'substrate world enable --home \"${PREFIX}\"' to flip it back on."
      fi
    fi
  fi
  if [[ "${WORLD_ENABLED}" -eq 0 ]]; then
    : # world provisioning failed above; skip the remainder of the provisioning block.
  else
  ensure_substrate_group_membership
	  PROVISION_SCRIPT="${REPO_ROOT}/scripts/linux/world-provision.sh"
	  if [[ -x "${PROVISION_SCRIPT}" ]]; then
	    log "Provisioning Linux world-service service via ${PROVISION_SCRIPT} (sudo may prompt if needed)..."
	    provision_status=0
	    provision_args=(
          --home "${PREFIX}"
          --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}"
          --profile "${PROFILE}"
          --skip-build
        )
	    if [[ "${ENABLE_WORLD_NETFILTER}" -eq 1 ]]; then
	      provision_args+=(--world-netfilter)
	    fi
	    if [[ "${use_noninteractive_world_provision}" -eq 1 ]]; then
	      provision_args+=(--sudo-noninteractive)
	    fi
          SUBSTRATE_HOME="${PREFIX}" \
          SUBSTRATE_ROOT="${PREFIX}" \
          SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}" \
          SUBSTRATE_INSTALL_PRIMARY_USER="${INSTALL_BOOTSTRAP_ACCOUNT}" \
          SUBSTRATE_INSTALL_PRIMARY_UID="${INSTALL_BOOTSTRAP_UID}" \
          SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
          "${PROVISION_SCRIPT}" "${provision_args[@]}" || provision_status=$?
        if [[ "${provision_status}" -ne 0 ]]; then
	      WORLD_PROVISION_FAILED=1
	      fail_closed_world_provisioning_for_runtime_request \
	        "the Linux world-provision helper reported an error." \
	        "Re-run ${PROVISION_SCRIPT} successfully, then retry the dev install."
	      WORLD_ENABLED=0
	      write_install_metadata "${WORLD_ENABLED}"
	      write_env_sh_script "${WORLD_ENABLED}"
      write_manager_env_script "${WORLD_ENABLED}"
      warn "world-provision script reported an error; rerun ${PROVISION_SCRIPT} manually to enable the world-service service."
      warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures. Re-run provisioning, then run 'substrate world enable --home \"${PREFIX}\"' to flip it back on."
    fi
  else
    WORLD_PROVISION_FAILED=1
    fail_closed_world_provisioning_for_runtime_request \
      "the Linux world-provision helper is missing at ${PROVISION_SCRIPT}." \
      "Restore that helper or omit --provision-agent-runtime."
    WORLD_ENABLED=0
    write_install_metadata "${WORLD_ENABLED}"
    write_env_sh_script "${WORLD_ENABLED}"
    write_manager_env_script "${WORLD_ENABLED}"
    warn "Linux world-provision script missing at ${PROVISION_SCRIPT}; world-service service not configured."
    warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures."
  fi
  ensure_socket_group_alignment
  fi
elif [[ "${WORLD_ENABLED}" -eq 1 && "${IS_MAC}" -eq 1 ]]; then
  log "Provisioning macOS Lima world-service service..."
  if ! command -v limactl >/dev/null 2>&1; then
    fatal "limactl not found; install Lima or rerun with --no-world to skip macOS world provisioning."
  fi
  LIMA_WARM="${REPO_ROOT}/scripts/mac/lima-warm.sh"
  if [[ ! -x "${LIMA_WARM}" ]]; then
    fatal "Expected Lima warm helper at ${LIMA_WARM}"
  fi
  bootstrap_request="$(python3 - "${INSTALL_BOOTSTRAP_CONTEXT_V1}" <<'PY'
import json
import sys
print(json.dumps({"install_bootstrap_context_v1": sys.argv[1]}, sort_keys=True, separators=(",", ":")))
PY
)" || fatal "cannot encode the closed direct publisher-bootstrap request"
  bootstrap_response="$(printf '%s' "${bootstrap_request}" | "${BIN_DIR}/substrate-lifecycle-control" publisher-bootstrap)" \
    || fatal "direct publisher-bootstrap did not produce a signed Stage-1 result"
  stage_one_authorization="$(python3 - "${bootstrap_response}" <<'PY'
import json
import re
import sys

try:
    response = json.loads(sys.argv[1])
    required = {
        "status", "scope_id", "manifest_generation", "manifest_sha256",
        "authorization_sha256", "anchor_sha256", "lima_stage_one_authorization_v1",
        "bootstrap_channel_bound",
    }
    if not isinstance(response, dict) or set(response) != required:
        raise ValueError()
    stage = response["lima_stage_one_authorization_v1"]
    if (response["status"] != "bootstrapped" or response["bootstrap_channel_bound"] is not True
            or not isinstance(stage, dict)
            or stage.get("schema_owner") != "substrate.lima-stage-one-authorization"
            or stage.get("schema_version") != 1
            or stage.get("expected_absent") is not True):
        raise ValueError()
    if not isinstance(response["scope_id"], str) or not re.fullmatch(r"[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}", response["scope_id"]):
        raise ValueError()
    for field in ("manifest_sha256", "authorization_sha256", "anchor_sha256"):
        if not isinstance(response[field], str) or not re.fullmatch(r"[0-9a-f]{64}", response[field]):
            raise ValueError()
    print(json.dumps(stage, sort_keys=True, separators=(",", ":")))
except Exception:
    raise SystemExit("invalid direct publisher-bootstrap response")
PY
)" || fatal "direct publisher-bootstrap returned an invalid Stage-1 result"
  publisher_service_state_response="$("${BIN_DIR}/substrate-lifecycle-control" publisher-service-state-install)" \
    || fatal "closed publisher service-state install did not register the fixed system service"
  python3 - "${publisher_service_state_response}" <<'PY' >/dev/null || \
    fatal "closed publisher service-state install returned a non-canonical result"
import json
import re
import sys

try:
    response = json.loads(sys.argv[1])
    required = {"launchd_domain", "record_sha256", "service_label", "status"}
    if not isinstance(response, dict) or set(response) != required:
        raise ValueError()
    if (response["launchd_domain"] != "system"
            or response["service_label"] != "com.substrate.lifecycle.publisher.v1"
            or response["status"] != "installed"
            or not isinstance(response["record_sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", response["record_sha256"])):
        raise ValueError()
except Exception:
    raise SystemExit(1)
PY
  lima_warm_env=(LIMA_BUILD_PROFILE="${PROFILE}")
  if [[ "${ENABLE_WORLD_NETFILTER}" -eq 1 ]]; then
    lima_warm_env+=(SUBSTRATE_WORLD_NETFILTER_ENABLE=1)
  fi
  lima_home="${INSTALL_BOOTSTRAP_ACCOUNT_HOME%/}/.lima"
  (
    cd "${REPO_ROOT}" &&
    env HOME="${INSTALL_BOOTSTRAP_ACCOUNT_HOME}" \
      LIMA_HOME="${lima_home}" \
      "${lima_warm_env[@]}" \
      "${LIMA_WARM}" \
      --install-prefix "${PREFIX}" \
      --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
      --lima-stage-one-authorization-v1 "${stage_one_authorization}" \
      "${REPO_ROOT}"
  )

  if ! verify_prefix_linux_bundle substrate-lifecycle-linux world-service substrate-gateway substrate; then
    fail_closed_world_provisioning_for_runtime_request \
      "the pre-Stage-1 macOS Linux artifact bundle verification step failed." \
      "Fix Lima provisioning and rerun the dev install, or omit --provision-agent-runtime."
    WORLD_ENABLED=0
    write_install_metadata "${WORLD_ENABLED}"
    write_env_sh_script "${WORLD_ENABLED}"
    write_manager_env_script "${WORLD_ENABLED}"
    warn "macOS dev-install did not retain the exact pre-Stage-1 Linux artifact bundle under ${BIN_DIR}/linux."
    warn "World has been disabled in ${INSTALL_CONFIG_PATH} to avoid confusing runtime failures. Re-run dev-install after fixing Lima provisioning."
  fi
fi

if [[ "${WORLD_ENABLED}" -eq 1 ]]; then
  provision_agent_runtime_with_sync "${SUBSTRATE_BIN}"
fi

cat >"${ENV_FILE}" <<EOF_ENV
# Generated by ${SCRIPT_NAME} on $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Source this file to enable Substrate dev shims for the current shell session.
substrate_home="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export SUBSTRATE_ROOT="\${substrate_home}"
export SUBSTRATE_HOME="\${substrate_home}"
export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=$(printf '%q' "${INSTALL_BOOTSTRAP_COMMITMENT}")
export SUBSTRATE_INSTALL_PRIMARY_USER=$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT}")
export SUBSTRATE_INSTALL_PRIMARY_UID=$(printf '%q' "${INSTALL_BOOTSTRAP_UID}")
export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=$(printf '%q' "${INSTALL_BOOTSTRAP_CONTEXT_V1}")
if [[ -f "\${substrate_home%/}/env.sh" ]]; then
  # shellcheck disable=SC1090
  source "\${substrate_home%/}/env.sh"
fi
export SUBSTRATE_MANAGER_INIT="\${substrate_home%/}/manager_init.sh"
if [[ -z "\${SHIM_ORIGINAL_PATH:-}" ]]; then
  export SHIM_ORIGINAL_PATH="\$PATH"
fi
substrate_bin_dir="\${substrate_home%/}/bin"
substrate_shims_dir="\${substrate_home%/}/shims"
if [[ ":\$PATH:" != *":\${substrate_bin_dir}:"* ]]; then
  export PATH="\${substrate_bin_dir}:\$PATH"
fi
if [[ ":\$PATH:" != *":\${substrate_shims_dir}:"* ]]; then
  export PATH="\${substrate_shims_dir}:\$PATH"
fi
EOF_ENV
log "Wrote dev shim helper to ${ENV_FILE}"

cat <<MSG

${shim_note}
To add the dev binaries/shims to PATH for this shell, run:
  source ${ENV_FILE}

MSG
if [[ "${IS_LINUX}" -eq 1 && "${WORLD_ENABLED}" -eq 1 ]] && ((${#HOST_STATE_ADDED_USERS[@]} > 0)); then
  cat <<MSG
[${SCRIPT_NAME}][WARN] This install added your user to the 'substrate' group.
[${SCRIPT_NAME}][WARN] The Linux socket ACL bridge should make /run/substrate.sock usable immediately.
[${SCRIPT_NAME}][WARN] If 'substrate host doctor --json' reports degraded ACL state, refresh this shell with:
  exec newgrp substrate
Then re-run:
  source ${ENV_FILE}

MSG
fi
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
detect_platform_metadata || true
write_host_state_metadata
