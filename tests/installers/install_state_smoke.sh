#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
HOST_PATH="${PATH}"
STUB_BIN=""

log() {
  printf '[install-state] %s\n' "$*" >&2
}

fatal() {
  log "ERROR: $*"
  exit 1
}

usage() {
  cat <<'USAGE' >&2
Usage: tests/installers/install_state_smoke.sh [--scenario <all|metadata|cleanup|missing>] [--keep-root]

Scenarios:
  metadata  Validate installer metadata creation + schema upgrade handling.
  cleanup   Exercise uninstall --cleanup-state with multi-user metadata.
  missing   Verify cleanup flag fallback when metadata is absent or corrupt.
  all       Run every scenario (default).

This harness stubs privileged/systemd commands so it never mutates the host.
It records group/linger operations and metadata files under a temp root.
USAGE
}

record_skip() {
  local reason="$1"
  if [[ -n "${SKIP_LOG:-}" ]]; then
    printf 'skipped: %s\n' "${reason}" >"${SKIP_LOG}"
  fi
  log "Skipping: ${reason}"
  if [[ -n "${SKIP_LOG:-}" ]]; then
    log "Skip details written to ${SKIP_LOG}"
  fi
}

write_stub_command() {
  local name="$1"
  cat >"${STUB_BIN}/${name}" <<'EOF_STUB'
#!/usr/bin/env bash
exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/${name}"
}

write_stub_sudo() {
  cat >"${STUB_BIN}/sudo" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
FAKE_ROOT="${FAKE_ROOT:-}"
SYSTEMCTL_LOG="${SUBSTRATE_TEST_SYSTEMCTL_LOG:-}"
if [[ $# -lt 1 ]]; then
  exit 0
fi
cmd="$1"
shift || true
while [[ "${cmd}" == -* && $# -gt 0 ]]; do
  cmd="$1"
  shift || true

done

rewrite_dest_arg() {
  local -n src_args=$1
  local rewritten=()
  local last_index=$((${#src_args[@]} - 1))
  for i in "${!src_args[@]}"; do
    local val="${src_args[$i]}"
    if [[ "${i}" -eq "${last_index}" && "${val}" == /* && -n "${FAKE_ROOT}" ]]; then
      rewritten+=("${FAKE_ROOT}${val}")
    else
      rewritten+=("${val}")
    fi
  done
  src_args=("${rewritten[@]}")
}

rewrite_all_paths() {
  local -n src_args=$1
  local rewritten=()
  for val in "${src_args[@]}"; do
    if [[ "${val}" == /* && -n "${FAKE_ROOT}" ]]; then
      rewritten+=("${FAKE_ROOT}${val}")
    else
      rewritten+=("${val}")
    fi
  done
  src_args=("${rewritten[@]}")
}

log_systemctl() {
  if [[ -z "${SYSTEMCTL_LOG}" ]]; then
    return
  fi
  printf 'systemctl %s\n' "$*" >>"${SYSTEMCTL_LOG}"
}

case "${cmd}" in
  systemctl)
    log_systemctl "$@"
    exit 0
    ;;
  install|cp|mv|ln)
    args=("$@")
    rewrite_dest_arg args
    exec "${cmd}" "${args[@]}"
    ;;
  rm|mkdir|chmod|chown)
    args=("$@")
    rewrite_all_paths args
    exec "${cmd}" "${args[@]}"
    ;;
  *)
    args=("$@")
    exec "${cmd}" "${args[@]}"
    ;;
esac
EOF_STUB
  chmod +x "${STUB_BIN}/sudo"
}

normalize_user_key() {
  local user="$1"
  local key="${user//[^A-Za-z0-9]/_}"
  printf '%s' "${key^^}"
}

lookup_user_groups() {
  local user="$1"
  local key
  key="$(normalize_user_key "${user}")"
  local env_name="SUBSTRATE_TEST_USER_GROUPS_${key}"
  local groups="${!env_name:-}"
  if [[ -n "${groups}" ]]; then
    printf '%s\n' "${groups}"
  else
    printf '%s\n' "${SUBSTRATE_TEST_USER_GROUPS:-wheel docker}"
  fi
}

write_stub_id() {
  cat >"${STUB_BIN}/id" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
normalize_user_key() {
  local user="$1"
  local key="${user//[^A-Za-z0-9]/_}"
  printf '%s' "${key^^}"
}
lookup_user_groups() {
  local user="$1"
  local key
  key="$(normalize_user_key "${user}")"
  local env_name="SUBSTRATE_TEST_USER_GROUPS_${key}"
  local groups="${!env_name:-}"
  if [[ -n "${groups}" ]]; then
    printf '%s\n' "${groups}"
  else
    printf '%s\n' "${SUBSTRATE_TEST_USER_GROUPS:-wheel docker}"
  fi
}

primary="${SUBSTRATE_TEST_PRIMARY_USER:-substrate-state}"
if [[ $# -eq 0 ]]; then
  printf '%s\n' "${primary}"
  exit 0
fi

case "$1" in
  -un)
    printf '%s\n' "${primary}"
    exit 0
    ;;
  -nG)
    shift || true
    user="${1:-${primary}}"
    lookup_user_groups "${user}"
    exit 0
    ;;
esac

printf '%s\n' "${primary}"
exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/id"
}

write_stub_getent() {
  cat >"${STUB_BIN}/getent" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ge 2 && "$1" == "group" ]]; then
  group="$2"
  if [[ "${group}" == "substrate" ]]; then
    if [[ "${SUBSTRATE_TEST_GROUP_EXISTS:-0}" -eq 1 ]]; then
      members="${SUBSTRATE_TEST_GROUP_MEMBERS:-}"
      entry="${SUBSTRATE_TEST_GROUP_ENTRY:-substrate:x:999:${members}}"
      printf '%s\n' "${entry}"
      exit 0
    fi
    exit 2
  fi
fi
exit 2
EOF_STUB
  chmod +x "${STUB_BIN}/getent"
}

write_stub_groupadd() {
  cat >"${STUB_BIN}/groupadd" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GROUP_LOG:-}"
if [[ -n "${log}" ]]; then
  printf 'groupadd %s\n' "$*" >>"${log}"
fi
exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/groupadd"
}

write_stub_usermod() {
  cat >"${STUB_BIN}/usermod" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GROUP_LOG:-}"
if [[ -n "${log}" ]]; then
  printf 'usermod %s\n' "$*" >>"${log}"
fi
exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/usermod"
}

write_stub_gpasswd() {
  cat >"${STUB_BIN}/gpasswd" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GROUP_LOG:-}"
if [[ -n "${log}" ]]; then
  printf 'gpasswd %s\n' "$*" >>"${log}"
fi
exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/gpasswd"
}

write_stub_groupdel() {
  cat >"${STUB_BIN}/groupdel" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
log="${SUBSTRATE_TEST_GROUP_LOG:-}"
if [[ -n "${log}" ]]; then
  printf 'groupdel %s\n' "$*" >>"${log}"
fi
exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/groupdel"
}

lookup_linger_state() {
  local user="$1"
  local key
  key="$(normalize_user_key "${user}")"
  local env_name="SUBSTRATE_TEST_LINGER_STATE_${key}"
  local state="${!env_name:-}"
  if [[ -n "${state}" ]]; then
    printf '%s\n' "${state}"
  else
    printf '%s\n' "${SUBSTRATE_TEST_LINGER_STATE:-no}"
  fi
}

write_stub_loginctl() {
  cat >"${STUB_BIN}/loginctl" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail

normalize_user_key() {
  local user="$1"
  local key="${user//[^A-Za-z0-9]/_}"
  printf '%s' "${key^^}"
}

lookup_linger_state() {
  local user="$1"
  local key
  key="$(normalize_user_key "${user}")"
  local env_name="SUBSTRATE_TEST_LINGER_STATE_${key}"
  local state="${!env_name:-}"
  if [[ -n "${state}" ]]; then
    printf '%s\n' "${state}"
  else
    printf '%s\n' "${SUBSTRATE_TEST_LINGER_STATE:-no}"
  fi
}

log="${SUBSTRATE_TEST_LINGER_LOG:-}"

if [[ $# -ge 1 && "$1" == "show-user" ]]; then
  user="${2:-unknown}"
  state="$(lookup_linger_state "${user}")"
  if [[ "${3:-}" == "-p" && "${4:-}" == "Linger" ]]; then
    if [[ -n "${log}" ]]; then
      printf '%s %s\n' "${user}" "${state}" >>"${log}"
    fi
    printf 'Linger=%s\n' "${state}"
    exit 0
  fi
fi

if [[ $# -ge 1 && "$1" == "enable-linger" ]]; then
  user="${2:-unknown}"
  if [[ -n "${log}" ]]; then
    printf '%s enable\n' "${user}" >>"${log}"
  fi
  exit 0
fi

if [[ $# -ge 1 && "$1" == "disable-linger" ]]; then
  user="${2:-unknown}"
  if [[ -n "${log}" ]]; then
    printf '%s disable\n' "${user}" >>"${log}"
  fi
  exit 0
fi

exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/loginctl"
}

prepare_stub_bin() {
  write_stub_sudo
  write_stub_id
  write_stub_getent
  write_stub_groupadd
  write_stub_usermod
  write_stub_gpasswd
  write_stub_groupdel
  write_stub_loginctl
  for cmd in curl fusermount fuse-overlayfs ip nft systemctl limactl pkill pgrep; do
    write_stub_command "${cmd}"
  done
}

detect_bundle_label() {
  local arch
  arch="$(uname -m)"
  case "${arch}" in
    x86_64|amd64) printf 'linux_x86_64' ;;
    aarch64|arm64) printf 'linux_aarch64' ;;
    *) fatal "Unsupported architecture for harness: ${arch}" ;;
  esac
}

compute_sha256() {
  local path="$1"
  python3 - <<'PY' "${path}"
import hashlib, pathlib, sys
path = pathlib.Path(sys.argv[1])
h = hashlib.sha256()
with path.open('rb') as f:
    for chunk in iter(lambda: f.read(1024 * 1024), b''):
        h.update(chunk)
print(h.hexdigest())
PY
}

write_stub_binary() {
  local path="$1"
  cat >"${path}" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail

require_manifest() {
  local root="${SUBSTRATE_ROOT:-}"
  if [[ -z "${root}" ]]; then
    root="$(cd "$(dirname "$0")/.." && pwd)"
  fi
  local latest
  latest="$(ls -1 "${root}/versions" 2>/dev/null | sort | tail -n1 || true)"
  local manifest="${latest:+${root}/versions/${latest}/config/manager_hooks.yaml}"
  if [[ -z "${manifest}" || ! -f "${manifest}" ]]; then
    echo '{"error":"missing manager manifest"}' >&2
    exit 1
  fi
}

cmd="${1:-}"
shift || true

case "${cmd}" in
  --shim-deploy)
    if [[ -n "${SUBSTRATE_ROOT:-}" ]]; then
      mkdir -p "${SUBSTRATE_ROOT}/shims"
    fi
    exit 0
    ;;
  world)
    sub="${1:-}"
    shift || true
    case "${sub}" in
      doctor)
        require_manifest
        printf '{"status":"ok"}\n'
        exit 0
        ;;
      deps)
        exit 0
        ;;
    esac
    ;;
  health)
    require_manifest
    printf '{"health":"ok"}\n'
    exit 0
    ;;
esac

exit 0
EOF_STUB
  chmod +x "${path}"
}

build_fake_release() {
  local work_root="$1"
  local artifacts="$2"
  local version="$3"

  local label
  label="$(detect_bundle_label)"
  local bundle_root="${work_root}/bundle/${label}"
  mkdir -p "${bundle_root}/bin" "${bundle_root}/config"

  cp "${REPO_ROOT}/config/manager_hooks.yaml" "${bundle_root}/config/manager_hooks.yaml"
  cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" "${bundle_root}/config/world-deps.yaml"

  write_stub_binary "${bundle_root}/bin/substrate"
  write_stub_binary "${bundle_root}/bin/host-proxy"
  write_stub_binary "${bundle_root}/bin/world-agent"

  local archive="substrate-v${version}-${label}.tar.gz"
  tar -czf "${artifacts}/${archive}" -C "${work_root}/bundle" "${label}"

  local checksum
  checksum="$(compute_sha256 "${artifacts}/${archive}")"
  printf '%s  %s\n' "${checksum}" "${archive}" >"${artifacts}/SHA256SUMS"
}

assert_metadata_file() {
  local path="$1"
  local expected_user="$2"
  local expect_group_created="$3"
  local expect_member_recorded="$4"
  local expect_linger_state="$5"

  python3 - <<'PY' "${path}" "${expected_user}" "${expect_group_created}" "${expect_member_recorded}" "${expect_linger_state}"
import json, sys
from pathlib import Path

path = Path(sys.argv[1])
expected_user = sys.argv[2]
expect_group_created = sys.argv[3].lower() == "true"
expect_member = sys.argv[4].lower() == "true"
expected_linger = sys.argv[5]

if not path.exists():
    raise SystemExit(f"{path} missing")

data = json.loads(path.read_text())
if data.get("schema_version") != 1:
    raise SystemExit(f"schema_version={data.get('schema_version')} (expected 1)")

created_at = data.get("created_at")
updated_at = data.get("updated_at")
if not created_at or not updated_at:
    raise SystemExit("created_at/updated_at missing")

host = data.get("host_state") or {}
group = host.get("group") or {}
linger = host.get("linger") or {}

existed = group.get("existed_before")
if existed not in (True, False, None):
    raise SystemExit(f"unexpected existed_before value: {existed}")

if bool(group.get("created_by_installer")) != expect_group_created:
    raise SystemExit(f"created_by_installer={group.get('created_by_installer')} expected {expect_group_created}")

members = group.get("members_added") or []
if expect_member:
    if expected_user not in members:
        raise SystemExit(f"{expected_user} missing from members_added: {members}")
else:
    if members:
        raise SystemExit(f"members_added should be empty, found {members}")

linger_users = (linger.get("users") or {})
entry = linger_users.get(expected_user, {})
state = entry.get("state_at_install")
enabled = entry.get("enabled_by_substrate", False)
if state != expected_linger:
    raise SystemExit(f"linger state {state} != expected {expected_linger}")
if enabled not in (True, False):
    raise SystemExit(f"enabled_by_substrate not set for {expected_user}")

print(f"metadata ok: created_by_installer={expect_group_created} members={members} linger={state}/{enabled}")
PY
}

assert_metadata_upgraded() {
  local path="$1"
  local expected_user="$2"

  python3 - <<'PY' "${path}" "${expected_user}"
import json, sys
from pathlib import Path

path = Path(sys.argv[1])
user = sys.argv[2]
data = json.loads(path.read_text())
if data.get("schema_version") != 1:
    raise SystemExit(f"schema_version={data.get('schema_version')} (expected 1)")

created_at = data.get("created_at")
updated_at = data.get("updated_at")
if not created_at or not updated_at:
    raise SystemExit("timestamps missing after upgrade")
legacy_markers = ("legacy-created", "legacy-updated")
if created_at in legacy_markers or updated_at in legacy_markers:
    raise SystemExit("legacy timestamps persisted unexpectedly")

host = data.get("host_state") or {}
group = host.get("group") or {}
linger = host.get("linger") or {}

if group.get("created_by_installer") is True:
    raise SystemExit("created_by_installer should not stay true when group preexisted during upgrade")

members = group.get("members_added") or []
if members:
    raise SystemExit(f"members_added should be empty after upgrade run, found {members}")

entry = (linger.get("users") or {}).get(user, {})
if entry.get("state_at_install") != "yes":
    raise SystemExit(f"expected linger state 'yes' after upgrade, got {entry.get('state_at_install')}")

print("upgrade metadata ok")
PY
}

assert_cleanup_logs() {
  local group_log="$1"
  local linger_log="$2"
  local uninstall_log="$3"

  python3 - <<'PY' "${group_log}" "${linger_log}" "${uninstall_log}"
import pathlib, sys
group_log, linger_log, uninstall_log = map(pathlib.Path, sys.argv[1:])

group_lines = group_log.read_text().splitlines() if group_log.exists() else []
expected_group_ops = ["gpasswd -d alice substrate", "gpasswd -d bob substrate", "groupdel substrate"]
for expected in expected_group_ops:
    if not any(expected in line for line in group_lines):
        raise SystemExit(f"missing group op: {expected}")

linger_lines = linger_log.read_text().splitlines() if linger_log.exists() else []
disable = [line for line in linger_lines if "disable" in line]
if not disable:
    raise SystemExit("expected disable-linger entry for recorded user")
if any("carol" in line for line in disable):
    raise SystemExit("linger disable should not target unrecorded users")

text = uninstall_log.read_text()
if "Disabled lingering for alice" not in text:
    raise SystemExit("uninstall output missing disable-linger notice")
PY
}

assert_no_cleanup_ops() {
  local group_log="$1"
  local linger_log="$2"
  local uninstall_log="$3"
  local expected_phrase="$4"

  python3 - <<'PY' "${group_log}" "${linger_log}" "${uninstall_log}" "${expected_phrase}"
import pathlib, sys
group_log, linger_log, uninstall_log = [pathlib.Path(p) for p in sys.argv[1:4]]
phrase = sys.argv[4]

group_lines = group_log.read_text().splitlines() if group_log.exists() else []
if group_lines:
    raise SystemExit(f"expected no group operations, found: {group_lines}")

linger_lines = linger_log.read_text().splitlines() if linger_log.exists() else []
if any("disable" in line for line in linger_lines):
    raise SystemExit("disable-linger should not run without valid metadata")

text = uninstall_log.read_text()
if phrase not in text:
    raise SystemExit(f"expected uninstall output to mention '{phrase}'")
PY
}

run_metadata_scenario() (
  local work_root
  work_root="$(mktemp -d "/tmp/substrate-install-state.metadata.XXXXXX")"
  if [[ "${KEEP_ROOT}" -eq 1 ]]; then
    log "Keeping work root for metadata scenario: ${work_root}"
  else
    trap 'rm -rf "${work_root}"' EXIT
  fi

  SKIP_LOG="${work_root}/skip.log"
  if [[ "$(uname -s)" != "Linux" ]]; then
    record_skip "metadata scenario requires Linux (detected $(uname -s))"
    exit 0
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    record_skip "python3 not available for metadata assertions"
    exit 0
  fi

  local prefix="${work_root}/home/.substrate"
  local home_dir="${work_root}/home"
  local artifacts="${work_root}/artifacts"
  local stub_bin="${work_root}/stub-bin"
  local fake_root="${work_root}/fakeroot"
  local systemctl_log="${work_root}/systemctl.log"
  local group_log="${work_root}/group-ops.log"
  local linger_log="${work_root}/linger.log"
  local install_log="${work_root}/install.log"
  local fake_version="0.0.0-state"
  local state_path="${prefix}/install_state.json"

  mkdir -p "${prefix}" "${artifacts}" "${stub_bin}" "${fake_root}" "${home_dir}"
  : >"${systemctl_log}"; : >"${group_log}"; : >"${linger_log}"
  STUB_BIN="${stub_bin}" prepare_stub_bin
  build_fake_release "${work_root}" "${artifacts}" "${fake_version}"

  local harness_path="${stub_bin}:${HOST_PATH}"
  local fake_user="substrate-state"

  # Initial install to create metadata (group created, user added, linger logged).
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${systemctl_log}" \
  SUBSTRATE_TEST_GROUP_LOG="${group_log}" \
  SUBSTRATE_TEST_LINGER_LOG="${linger_log}" \
  SUBSTRATE_TEST_PRIMARY_USER="${fake_user}" \
  SUBSTRATE_TEST_USER_GROUPS="wheel docker" \
  SUBSTRATE_TEST_GROUP_EXISTS=0 \
  FAKE_ROOT="${fake_root}" \
  HOME="${home_dir}" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --prefix "${prefix}" \
    --version "${fake_version}" \
    --artifact-dir "${artifacts}" \
    >"${install_log}" 2>&1 || {
      cat "${install_log}" >&2
      fatal "metadata install failed (see ${install_log})"
    }

  assert_metadata_file "${state_path}" "${fake_user}" "true" "true" "no"

  # Simulate legacy metadata and rerun install to ensure upgrade path rewrites it.
  cat >"${state_path}" <<'EOF_STUB'
{"schema_version":0,"created_at":"legacy-created","updated_at":"legacy-updated","host_state":{"group":{"members_added":["legacy-user"]}}}
EOF_STUB
  sleep 1
  : >"${group_log}"; : >"${linger_log}"

  SUBSTRATE_TEST_SYSTEMCTL_LOG="${systemctl_log}" \
  SUBSTRATE_TEST_GROUP_LOG="${group_log}" \
  SUBSTRATE_TEST_LINGER_LOG="${linger_log}" \
  SUBSTRATE_TEST_PRIMARY_USER="${fake_user}" \
  SUBSTRATE_TEST_USER_GROUPS="substrate wheel" \
  SUBSTRATE_TEST_GROUP_EXISTS=1 \
  SUBSTRATE_TEST_LINGER_STATE="yes" \
  FAKE_ROOT="${fake_root}" \
  HOME="${home_dir}" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --prefix "${prefix}" \
    --version "${fake_version}" \
    --artifact-dir "${artifacts}" \
    >"${install_log}" 2>&1 || {
      cat "${install_log}" >&2
      fatal "metadata upgrade install failed (see ${install_log})"
    }

  assert_metadata_upgraded "${state_path}" "${fake_user}"

  log "Metadata scenario completed (state: ${state_path})"
)

run_cleanup_scenario() (
  local work_root
  work_root="$(mktemp -d "/tmp/substrate-install-state.cleanup.XXXXXX")"
  if [[ "${KEEP_ROOT}" -eq 1 ]]; then
    log "Keeping work root for cleanup scenario: ${work_root}"
  else
    trap 'rm -rf "${work_root}"' EXIT
  fi

  SKIP_LOG="${work_root}/skip.log"
  if [[ "$(uname -s)" != "Linux" ]]; then
    record_skip "cleanup scenario requires Linux (detected $(uname -s))"
    exit 0
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    record_skip "python3 not available for cleanup assertions"
    exit 0
  fi

  local home_dir="${work_root}/home"
  local prefix="${home_dir}/.substrate"
  local stub_bin="${work_root}/stub-bin"
  local fake_root="${work_root}/fakeroot"
  local systemctl_log="${work_root}/systemctl.log"
  local group_log="${work_root}/group-ops.log"
  local linger_log="${work_root}/linger.log"
  local uninstall_log="${work_root}/uninstall.log"
  local state_path="${prefix}/install_state.json"

  mkdir -p "${prefix}" "${stub_bin}" "${fake_root}" "${home_dir}"
  : >"${systemctl_log}"; : >"${group_log}"; : >"${linger_log}"
  STUB_BIN="${stub_bin}" prepare_stub_bin
  local harness_path="${stub_bin}:${HOST_PATH}"

  cat >"${state_path}" <<'EOF_STUB'
{
  "schema_version": 1,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "existed_before": false,
      "created_by_installer": true,
      "members_added": ["alice", "bob"]
    },
    "linger": {
      "users": {
        "alice": {"state_at_install": "no", "enabled_by_substrate": true},
        "carol": {"state_at_install": "yes", "enabled_by_substrate": false}
      }
    }
  }
}
EOF_STUB

  SUBSTRATE_TEST_SYSTEMCTL_LOG="${systemctl_log}" \
  SUBSTRATE_TEST_GROUP_LOG="${group_log}" \
  SUBSTRATE_TEST_LINGER_LOG="${linger_log}" \
  SUBSTRATE_TEST_PRIMARY_USER="alice" \
  SUBSTRATE_TEST_USER_GROUPS="wheel" \
  SUBSTRATE_TEST_USER_GROUPS_ALICE="substrate wheel" \
  SUBSTRATE_TEST_USER_GROUPS_BOB="substrate" \
  SUBSTRATE_TEST_USER_GROUPS_CAROL="wheel" \
  SUBSTRATE_TEST_GROUP_EXISTS=1 \
  SUBSTRATE_TEST_GROUP_MEMBERS="" \
  SUBSTRATE_TEST_GROUP_ENTRY="substrate:x:999:" \
  SUBSTRATE_TEST_LINGER_STATE_ALICE="yes" \
  FAKE_ROOT="${fake_root}" \
  HOME="${home_dir}" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh" \
    --cleanup-state \
    >"${uninstall_log}" 2>&1 || {
      cat "${uninstall_log}" >&2
      fatal "cleanup scenario uninstall failed (see ${uninstall_log})"
    }

  assert_cleanup_logs "${group_log}" "${linger_log}" "${uninstall_log}"
  log "Cleanup scenario completed (state: ${state_path})"
)

run_missing_metadata_scenario() (
  local work_root
  work_root="$(mktemp -d "/tmp/substrate-install-state.missing.XXXXXX")"
  if [[ "${KEEP_ROOT}" -eq 1 ]]; then
    log "Keeping work root for missing/corrupt scenario: ${work_root}"
  else
    trap 'rm -rf "${work_root}"' EXIT
  fi

  SKIP_LOG="${work_root}/skip.log"
  if [[ "$(uname -s)" != "Linux" ]]; then
    record_skip "missing/corrupt scenario requires Linux (detected $(uname -s))"
    exit 0
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    record_skip "python3 not available for missing/corrupt assertions"
    exit 0
  fi

  local home_dir="${work_root}/home"
  local prefix="${home_dir}/.substrate"
  local stub_bin="${work_root}/stub-bin"
  local fake_root="${work_root}/fakeroot"
  local systemctl_log="${work_root}/systemctl.log"
  local group_log="${work_root}/group-ops.log"
  local linger_log="${work_root}/linger.log"
  local uninstall_log="${work_root}/uninstall.log"
  local state_path="${prefix}/install_state.json"

  mkdir -p "${prefix}" "${stub_bin}" "${fake_root}" "${home_dir}"
  : >"${systemctl_log}"; : >"${group_log}"; : >"${linger_log}"; : >"${uninstall_log}"
  STUB_BIN="${stub_bin}" prepare_stub_bin
  local harness_path="${stub_bin}:${HOST_PATH}"

  # Missing metadata path: expect fallback guidance only.
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${systemctl_log}" \
  SUBSTRATE_TEST_GROUP_LOG="${group_log}" \
  SUBSTRATE_TEST_LINGER_LOG="${linger_log}" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-missing" \
  SUBSTRATE_TEST_USER_GROUPS="wheel substrate" \
  SUBSTRATE_TEST_GROUP_EXISTS=1 \
  SUBSTRATE_TEST_GROUP_ENTRY="substrate:x:999:substrate-missing" \
  SUBSTRATE_TEST_LINGER_STATE="yes" \
  FAKE_ROOT="${fake_root}" \
  HOME="${home_dir}" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh" \
    --cleanup-state \
    >"${uninstall_log}" 2>&1 || {
      cat "${uninstall_log}" >&2
      fatal "uninstall (missing metadata) failed (see ${uninstall_log})"
    }

  assert_no_cleanup_ops "${group_log}" "${linger_log}" "${uninstall_log}" "Host-state metadata missing or unreadable"

  # Corrupt metadata path: expect same fallback path.
  mkdir -p "$(dirname "${state_path}")"
  echo '{invalid' >"${state_path}"
  : >"${group_log}"; : >"${linger_log}"; : >"${uninstall_log}"

  SUBSTRATE_TEST_SYSTEMCTL_LOG="${systemctl_log}" \
  SUBSTRATE_TEST_GROUP_LOG="${group_log}" \
  SUBSTRATE_TEST_LINGER_LOG="${linger_log}" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-missing" \
  SUBSTRATE_TEST_USER_GROUPS="wheel substrate" \
  SUBSTRATE_TEST_GROUP_EXISTS=1 \
  SUBSTRATE_TEST_GROUP_ENTRY="substrate:x:999:substrate-missing" \
  SUBSTRATE_TEST_LINGER_STATE="yes" \
  FAKE_ROOT="${fake_root}" \
  HOME="${home_dir}" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh" \
    --cleanup-state \
    >"${uninstall_log}" 2>&1 || {
      cat "${uninstall_log}" >&2
      fatal "uninstall (corrupt metadata) failed (see ${uninstall_log})"
    }

  assert_no_cleanup_ops "${group_log}" "${linger_log}" "${uninstall_log}" "Host-state metadata missing or unreadable"
  log "Missing/corrupt metadata scenario completed (state: ${state_path})"
)

SCENARIO="all"
KEEP_ROOT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      [[ $# -lt 2 ]] && fatal "Missing value for --scenario"
      SCENARIO="$2"
      shift 2
      ;;
    --keep-root)
      KEEP_ROOT=1
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

case "${SCENARIO}" in
  metadata|cleanup|missing|all) ;;
  *)
    usage
    fatal "Unsupported scenario: ${SCENARIO}"
    ;;
esac

if [[ "${SCENARIO}" == "metadata" || "${SCENARIO}" == "all" ]]; then
  run_metadata_scenario
fi
if [[ "${SCENARIO}" == "cleanup" || "${SCENARIO}" == "all" ]]; then
  run_cleanup_scenario
fi
if [[ "${SCENARIO}" == "missing" || "${SCENARIO}" == "all" ]]; then
  run_missing_metadata_scenario
fi

log "All requested scenarios completed."
