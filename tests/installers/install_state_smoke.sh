#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
HOST_PATH="${PATH}"
REAL_PYTHON3="$(command -v python3)"
REAL_MV="$(command -v mv)"
REAL_LS="$(command -v ls)"
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
if [[ "${cmd}" == "-v" || "${cmd}" == "--validate" ]]; then
  exit 0
fi
while [[ "${cmd}" == -* && $# -gt 0 ]]; do
  case "${cmd}" in
    -v|--validate)
      exit 0
      ;;
  esac
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

write_stub_python3() {
  cat >"${STUB_BIN}/python3" <<EOF_STUB
#!/usr/bin/env bash
set -euo pipefail
if [[ "\${SUBSTRATE_TEST_FAIL_HOST_STATE_WRITE:-0}" -eq 1 && "\${1:-}" == "-" && -n "\${STATE_EVENTS:-}" ]]; then
  exit 1
fi
if [[ "\${1:-}" == "-" ]]; then
  script_file="\$(mktemp)"
  cat >"\${script_file}"
  if [[ "\${SUBSTRATE_TEST_REJECT_DATETIME_UTC:-0}" -eq 1 ]] && grep -q 'datetime\\.UTC' "\${script_file}"; then
    rm -f "\${script_file}"
    echo "AttributeError: module 'datetime' has no attribute 'UTC'" >&2
    exit 1
  fi
  "${REAL_PYTHON3}" "\${script_file}" "\${@:2}"
  status=\$?
  rm -f "\${script_file}"
  exit "\${status}"
fi
exec "${REAL_PYTHON3}" "\$@"
EOF_STUB
  chmod +x "${STUB_BIN}/python3"
}

write_stub_mv() {
  cat >"${STUB_BIN}/mv" <<EOF_STUB
#!/usr/bin/env bash
set -euo pipefail
if [[ "\${SUBSTRATE_TEST_FAIL_HOST_STATE_REPLACE:-0}" -eq 1 && "\$#" -eq 2 ]]; then
  if [[ "\$1" == "\${SUBSTRATE_TEST_FAIL_MV_SOURCE:-}" && "\$2" == "\${SUBSTRATE_TEST_FAIL_MV_DEST:-}" ]]; then
    exit 1
  fi
fi
exec "${REAL_MV}" "\$@"
EOF_STUB
  chmod +x "${STUB_BIN}/mv"
}

write_stub_ls() {
  cat >"${STUB_BIN}/ls" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail
for arg in "$@"; do
  if [[ "${arg}" == "/run/substrate.sock" ]]; then
    printf 'srwxrwx--- 1 root substrate 0 /run/substrate.sock\n'
    exit 0
  fi
done
exec "__REAL_LS__" "$@"
EOF_STUB
  sed -i "s#__REAL_LS__#${REAL_LS}#g" "${STUB_BIN}/ls"
  chmod +x "${STUB_BIN}/ls"
}

write_stub_uname() {
  local uname_s="$1"
  cat >"${STUB_BIN}/uname" <<EOF_STUB
#!/usr/bin/env bash
set -euo pipefail
if [[ "\${1:-}" == "-s" ]]; then
  printf '%s\n' "${uname_s}"
  exit 0
fi
if [[ "\${1:-}" == "-m" ]]; then
  printf '%s\n' "${SUBSTRATE_TEST_UNAME_M:-x86_64}"
  exit 0
fi
printf '%s\n' "${uname_s}"
EOF_STUB
  chmod +x "${STUB_BIN}/uname"
}

write_stub_cargo() {
  cat >"${STUB_BIN}/cargo" <<'EOF_STUB'
#!/usr/bin/env bash
set -euo pipefail

target_root="${CARGO_TARGET_DIR:-${PWD}/target}"
profile="debug"
build_world=0
build_substrate=0
build_shim=0

for arg in "$@"; do
  case "${arg}" in
    --release)
      profile="release"
      ;;
    --bin)
      build_substrate=1
      ;;
    -p)
      build_world=1
      ;;
    substrate-shim)
      build_shim=1
      ;;
  esac
done

mkdir -p "${target_root}/${profile}"

write_bin() {
  local path="$1"
  cat >"${path}" <<'EOF_BIN'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  --shim-deploy)
    if [[ -n "${SUBSTRATE_ROOT:-}" ]]; then
      mkdir -p "${SUBSTRATE_ROOT}/shims"
    fi
    exit 0
    ;;
  world)
    if [[ "${2:-}" == "doctor" ]]; then
      printf '{"status":"ok"}\n'
      exit 0
    fi
    ;;
esac
exit 0
EOF_BIN
  chmod +x "${path}"
}

if [[ "${build_substrate}" -eq 1 ]]; then
  write_bin "${target_root}/${profile}/substrate"
  write_bin "${target_root}/${profile}/substrate-shim"
fi
if [[ "${build_world}" -eq 1 ]]; then
  write_bin "${target_root}/${profile}/world-agent"
fi
if [[ "${build_shim}" -eq 1 ]]; then
  write_bin "${target_root}/${profile}/substrate-shim"
fi

exit 0
EOF_STUB
  chmod +x "${STUB_BIN}/cargo"
}

load_current_os_release() {
  local source="${1:-/etc/os-release}"
  CURRENT_OS_RELEASE_ID="$(python3 - <<'PY' "${source}"
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
data = {}
if path.exists():
    for raw in path.read_text().splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#") or "=" not in raw:
            continue
        key, value = raw.split("=", 1)
        value = value.strip().strip('"').strip("'")
        data[key] = value.lower()

print(data.get("ID", "<unknown>"))
PY
)"
  CURRENT_OS_RELEASE_ID_LIKE="$(python3 - <<'PY' "${source}"
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
data = {}
if path.exists():
    for raw in path.read_text().splitlines():
        raw = raw.strip()
        if not raw or raw.startswith("#") or "=" not in raw:
            continue
        key, value = raw.split("=", 1)
        value = value.strip().strip('"').strip("'")
        data[key] = value.lower()

print(data.get("ID_LIKE", "<unknown>"))
PY
)"
}

choose_expected_pkg_manager() {
  local distro_id="$1"
  local distro_like="$2"
  case "${distro_id}" in
    debian|ubuntu|linuxmint|pop)
      printf 'apt-get'
      return 0
      ;;
    fedora|rhel|centos|rocky|almalinux|ol|amzn)
      printf 'dnf'
      return 0
      ;;
    arch|manjaro|endeavouros|arcolinux|artix|garuda)
      printf 'pacman'
      return 0
      ;;
    *suse*)
      printf 'zypper'
      return 0
      ;;
  esac

  case "${distro_like}" in
    *debian*|*ubuntu*)
      printf 'apt-get'
      return 0
      ;;
    *fedora*|*rhel*)
      printf 'dnf'
      return 0
      ;;
    *arch*)
      printf 'pacman'
      return 0
      ;;
    *suse*)
      printf 'zypper'
      return 0
      ;;
  esac

  printf 'pacman'
}

write_os_release_fixture() {
  local path="$1"
  local distro_id="$2"
  local distro_like="$3"
  mkdir -p "$(dirname "${path}")"
  cat >"${path}" <<EOF_OS
ID=${distro_id}
ID_LIKE=${distro_like}
EOF_OS
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
  write_stub_python3
  write_stub_mv
  write_stub_ls
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
  local expect_member_recorded="$3"
  local expect_linger_state="$4"

  python3 - <<'PY' "${path}" "${expected_user}" "${expect_member_recorded}" "${expect_linger_state}"
import json, sys
from pathlib import Path

path = Path(sys.argv[1])
expected_user = sys.argv[2]
expect_member = sys.argv[3].lower() == "true"
expected_linger = sys.argv[4]

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

if not group.get("name"):
    raise SystemExit("group.name missing")

members = group.get("members_added") or []
if expect_member:
    if expected_user not in members:
        raise SystemExit(f"{expected_user} missing from members_added: {members}")
else:
    if members:
        raise SystemExit(f"members_added should be empty, found {members}")

linger_users = (linger.get("users") or {})
if expected_linger == "__skip__":
    print(f"metadata ok: members={members} linger skipped")
    raise SystemExit(0)

entry = linger_users.get(expected_user, {})
state = entry.get("state_at_install")
enabled = entry.get("enabled_by_substrate", False)
if state != expected_linger:
    raise SystemExit(f"linger state {state} != expected {expected_linger}")
if enabled not in (True, False):
    raise SystemExit(f"enabled_by_substrate not set for {expected_user}")

print(f"metadata ok: members={members} linger={state}/{enabled}")
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

assert_platform_metadata() {
  local path="$1"
  local expected_id="$2"
  local expected_id_like="$3"
  local expected_pkg_manager="$4"
  local expected_pkg_manager_source="$5"

  python3 - <<'PY' "${path}" "${expected_id}" "${expected_id_like}" "${expected_pkg_manager}" "${expected_pkg_manager_source}"
import json, sys
from pathlib import Path

path = Path(sys.argv[1])
expected_id = sys.argv[2]
expected_id_like = sys.argv[3]
expected_pkg_manager = sys.argv[4]
expected_pkg_manager_source = sys.argv[5]

data = json.loads(path.read_text())
platform = (data.get("host_state") or {}).get("platform") or {}
os_release = platform.get("os_release") or {}
pkg_manager = platform.get("pkg_manager") or {}

checks = {
    "host_state.platform.os_release.id": (os_release.get("id"), expected_id),
    "host_state.platform.os_release.id_like": (os_release.get("id_like"), expected_id_like),
    "host_state.platform.pkg_manager.selected": (pkg_manager.get("selected"), expected_pkg_manager),
    "host_state.platform.pkg_manager.source": (pkg_manager.get("source"), expected_pkg_manager_source),
}
for label, (actual, expected) in checks.items():
    if actual != expected:
        raise SystemExit(f"{label}={actual!r} expected {expected!r}")

print("platform metadata ok")
PY
}

seed_platform_fixture() {
  local path="$1"
  local distro_id="$2"
  local distro_id_like="$3"
  local pkg_manager="$4"
  local pkg_manager_source="$5"
  local unknown_top="${6:-keep-me}"
  local extra_note="${7:-keep-host-note}"
  mkdir -p "$(dirname "${path}")"
  cat >"${path}" <<EOF_FIXTURE
{
  "schema_version": 1,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z",
  "unknown_top": "${unknown_top}",
  "host_state": {
    "extra_note": "${extra_note}",
    "group": {
      "name": "substrate",
      "existed_before": true,
      "created_by_installer": false,
      "members_added": []
    },
    "linger": {
      "users": {}
    },
    "platform": {
      "os_release": {
        "id": "${distro_id}",
        "id_like": "${distro_id_like}"
      },
      "pkg_manager": {
        "selected": "${pkg_manager}",
        "source": "${pkg_manager_source}"
      }
    }
  }
}
EOF_FIXTURE
}

assert_additive_compatibility_preserved() {
  local path="$1"
  local expected_unknown_top="$2"
  local expected_nested_note="$3"
  local expected_platform_id="$4"
  local expected_platform_like="$5"
  local expected_pkg_manager="$6"
  local expected_pkg_manager_source="$7"

  python3 - <<'PY' "${path}" "${expected_unknown_top}" "${expected_nested_note}" "${expected_platform_id}" "${expected_platform_like}" "${expected_pkg_manager}" "${expected_pkg_manager_source}"
import json, sys
from pathlib import Path

path = Path(sys.argv[1])
expected_unknown_top = sys.argv[2]
expected_nested_note = sys.argv[3]
expected_platform_id = sys.argv[4]
expected_platform_like = sys.argv[5]
expected_pkg_manager = sys.argv[6]
expected_pkg_manager_source = sys.argv[7]

data = json.loads(path.read_text())
if data.get("unknown_top") != expected_unknown_top:
    raise SystemExit(f"unknown_top={data.get('unknown_top')!r} expected {expected_unknown_top!r}")

host = data.get("host_state") or {}
if host.get("extra_note") != expected_nested_note:
    raise SystemExit(f"host_state.extra_note={host.get('extra_note')!r} expected {expected_nested_note!r}")

platform = host.get("platform") or {}
os_release = platform.get("os_release") or {}
pkg_manager = platform.get("pkg_manager") or {}
checks = {
    "host_state.platform.os_release.id": (os_release.get("id"), expected_platform_id),
    "host_state.platform.os_release.id_like": (os_release.get("id_like"), expected_platform_like),
    "host_state.platform.pkg_manager.selected": (pkg_manager.get("selected"), expected_pkg_manager),
    "host_state.platform.pkg_manager.source": (pkg_manager.get("source"), expected_pkg_manager_source),
}
for label, (actual, expected) in checks.items():
    if actual != expected:
        raise SystemExit(f"{label}={actual!r} expected {expected!r}")

print("additive compatibility preserved")
PY
}

assert_log_contains() {
  local path="$1"
  local needle="$2"
  if ! grep -Fq -- "${needle}" "${path}"; then
    fatal "expected ${path} to contain: ${needle}"
  fi
}

assert_log_not_contains() {
  local path="$1"
  local needle="$2"
  if grep -Fq -- "${needle}" "${path}"; then
    fatal "did not expect ${path} to contain: ${needle}"
  fi
}

assert_no_state_written() {
  local path="$1"
  if [[ -e "${path}" ]]; then
    fatal "expected no install_state.json at ${path}"
  fi
}

run_hosted_install_branch() {
  local prefix="$1"
  local version="$2"
  local artifacts="$3"
  local install_log="$4"
  local state_path="$5"
  local world_mode="$6"
  local os_release_path="$7"

  local harness_path="${STUB_BIN}:${HOST_PATH}"
  local args=("--prefix" "${prefix}" "--version" "${version}" "--artifact-dir" "${artifacts}")
  if [[ "${world_mode}" == "no-world" ]]; then
    args+=("--no-world")
  fi
  if [[ -n "${DRY_RUN_MODE:-}" && "${DRY_RUN_MODE}" -eq 1 ]]; then
    args+=("--dry-run")
  fi

  local prefix_home="${prefix}/home"
  mkdir -p "${prefix_home}" "$(dirname "${state_path}")"

  if [[ -n "${os_release_path}" ]]; then
    SUBSTRATE_INSTALL_OS_RELEASE_PATH="${os_release_path}" \
    SUBSTRATE_INSTALL_NO_PATH=1 \
    SUBSTRATE_TEST_SYSTEMCTL_LOG="${prefix}/systemctl.log" \
    SUBSTRATE_TEST_GROUP_LOG="${prefix}/group-ops.log" \
    SUBSTRATE_TEST_LINGER_LOG="${prefix}/linger.log" \
    SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
    SUBSTRATE_TEST_USER_GROUPS="wheel docker" \
    SUBSTRATE_TEST_GROUP_EXISTS=0 \
    FAKE_ROOT="${prefix}/fakeroot" \
    HOME="${prefix_home}" \
    PATH="${harness_path}" \
    SHIM_ORIGINAL_PATH="${harness_path}" \
    "${REPO_ROOT}/scripts/substrate/install-substrate.sh" "${args[@]}" \
    >"${install_log}" 2>&1 || {
      cat "${install_log}" >&2
      fatal "hosted install branch failed (see ${install_log})"
    }
  else
    SUBSTRATE_INSTALL_NO_PATH=1 \
    SUBSTRATE_TEST_SYSTEMCTL_LOG="${prefix}/systemctl.log" \
    SUBSTRATE_TEST_GROUP_LOG="${prefix}/group-ops.log" \
    SUBSTRATE_TEST_LINGER_LOG="${prefix}/linger.log" \
    SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
    SUBSTRATE_TEST_USER_GROUPS="wheel docker" \
    SUBSTRATE_TEST_GROUP_EXISTS=0 \
    FAKE_ROOT="${prefix}/fakeroot" \
    HOME="${prefix_home}" \
    PATH="${harness_path}" \
    SHIM_ORIGINAL_PATH="${harness_path}" \
    "${REPO_ROOT}/scripts/substrate/install-substrate.sh" "${args[@]}" \
    >"${install_log}" 2>&1 || {
      cat "${install_log}" >&2
      fatal "hosted install branch failed (see ${install_log})"
    }
  fi

  if [[ "${world_mode}" == "no-world" ]]; then
    assert_metadata_file "${state_path}" "substrate-state" "false" "__skip__"
  else
    assert_metadata_file "${state_path}" "substrate-state" "true" "no"
  fi
}

run_dev_install_branch() {
  local prefix="$1"
  local install_log="$2"
  local state_path="$3"
  local world_mode="$4"
  local expected_unknown_top="$5"
  local expected_nested_note="$6"
  local expected_state_id="$7"
  local expected_state_id_like="$8"
  local expected_state_pkg_manager="$9"
  local expected_state_pkg_manager_source="${10}"

  local args=("--prefix" "${prefix}" "--version-label" "smoke-dev")
  if [[ "${world_mode}" == "no-world" ]]; then
    args+=("--no-world")
  fi

  local harness_path="${REPO_ROOT}/target/debug:${STUB_BIN}:${HOST_PATH}"
  mkdir -p "${prefix}/home" "$(dirname "${state_path}")"
  HOME="${prefix}/home" \
  USER="substrate-state" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  FAKE_ROOT="${prefix}/fakeroot" \
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${prefix}/systemctl.log" \
  SUBSTRATE_TEST_GROUP_LOG="${prefix}/group-ops.log" \
  SUBSTRATE_TEST_LINGER_LOG="${prefix}/linger.log" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
  SUBSTRATE_TEST_USER_GROUPS="wheel docker" \
  SUBSTRATE_TEST_GROUP_EXISTS=0 \
  SUBSTRATE_TEST_LINGER_STATE="no" \
  "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh" "${args[@]}" \
  >"${install_log}" 2>&1 || {
    cat "${install_log}" >&2
    fatal "dev install branch failed (see ${install_log})"
  }

  if [[ "${world_mode}" == "no-world" ]]; then
    assert_metadata_file "${state_path}" "substrate-state" "false" "__skip__"
  else
    assert_metadata_file "${state_path}" "substrate-state" "true" "no"
  fi
  if [[ "${expected_unknown_top}" == "__skip__" ]]; then
    assert_platform_metadata "${state_path}" "${expected_state_id}" "${expected_state_id_like}" "${expected_state_pkg_manager}" "${expected_state_pkg_manager_source}"
  else
    assert_additive_compatibility_preserved "${state_path}" "${expected_unknown_top}" "${expected_nested_note}" "${expected_state_id}" "${expected_state_id_like}" "${expected_state_pkg_manager}" "${expected_state_pkg_manager_source}"
  fi
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

  local artifacts="${work_root}/artifacts"
  local stub_bin="${work_root}/stub-bin"
  local runtime_os_release="${work_root}/runtime-os-release"
  local missing_os_release="${work_root}/missing-os-release"
  local fake_version="0.0.0-state"
  local harness_path="${stub_bin}:${HOST_PATH}"
  mkdir -p "${artifacts}" "${stub_bin}"
  STUB_BIN="${stub_bin}"
  prepare_stub_bin
  write_stub_cargo
  load_current_os_release /etc/os-release
  local expected_pkg_manager
  expected_pkg_manager="$(choose_expected_pkg_manager "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}")"
  write_stub_command "${expected_pkg_manager}"
  write_os_release_fixture "${runtime_os_release}" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}"
  build_fake_release "${work_root}" "${artifacts}" "${fake_version}"

  local hosted_prefix="${work_root}/hosted"
  local hosted_state="${hosted_prefix}/install_state.json"
  local hosted_log="${work_root}/hosted/install.log"
  mkdir -p "${hosted_prefix}" "$(dirname "${hosted_log}")"
  run_hosted_install_branch "${work_root}/hosted" "${fake_version}" "${artifacts}" "${hosted_log}" "${hosted_state}" "with-world" "${runtime_os_release}"
  assert_env_sh_has_no_override_exports "${hosted_prefix}" "${HOST_PATH}"
  assert_log_not_contains "${hosted_log}" "warning: unable to parse"
  assert_log_not_contains "${hosted_log}" "unsupported schema_version"
  assert_platform_metadata "${hosted_state}" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"

  local hosted_noworld_prefix="${work_root}/hosted-noworld"
  local hosted_noworld_state="${hosted_noworld_prefix}/install_state.json"
  local hosted_noworld_log="${work_root}/hosted-noworld/install.log"
  mkdir -p "${hosted_noworld_prefix}" "$(dirname "${hosted_noworld_log}")"
  run_hosted_install_branch "${work_root}/hosted-noworld" "${fake_version}" "${artifacts}" "${hosted_noworld_log}" "${hosted_noworld_state}" "no-world" "${runtime_os_release}"
  assert_env_sh_has_no_override_exports "${hosted_noworld_prefix}" "${HOST_PATH}"
  assert_log_not_contains "${hosted_noworld_log}" "warning: unable to parse"
  assert_platform_metadata "${hosted_noworld_state}" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"

  local invalid_prefix="${work_root}/invalid"
  local invalid_state="${invalid_prefix}/install_state.json"
  local invalid_log="${work_root}/invalid/install.log"
  local invalid_os_release="${work_root}/invalid/os-release"
  mkdir -p "${invalid_prefix}" "${work_root}/invalid/home" "$(dirname "${invalid_log}")"
  write_os_release_fixture "${invalid_os_release}" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}"
  cat >"${invalid_state}" <<'EOF_INVALID'
{invalid
EOF_INVALID
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${invalid_os_release}" \
  SUBSTRATE_INSTALL_NO_PATH=1 \
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${work_root}/invalid/systemctl.log" \
  SUBSTRATE_TEST_GROUP_LOG="${work_root}/invalid/group-ops.log" \
  SUBSTRATE_TEST_LINGER_LOG="${work_root}/invalid/linger.log" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
  SUBSTRATE_TEST_USER_GROUPS="wheel docker" \
  SUBSTRATE_TEST_GROUP_EXISTS=0 \
  FAKE_ROOT="${work_root}/invalid/fakeroot" \
  HOME="${work_root}/invalid/home" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --prefix "${invalid_prefix}" \
    --version "${fake_version}" \
    --artifact-dir "${artifacts}" \
    >"${invalid_log}" 2>&1 || {
      cat "${invalid_log}" >&2
      fatal "invalid-json metadata install failed (see ${invalid_log})"
    }
  assert_log_contains "${invalid_log}" "warning: unable to parse"
  assert_metadata_file "${invalid_state}" "substrate-state" "true" "no"
  assert_platform_metadata "${invalid_state}" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"

  local missing_prefix="${work_root}/missing"
  local missing_state="${missing_prefix}/install_state.json"
  local missing_log="${work_root}/missing/install.log"
  local unknown_distro="<unknown>"
  mkdir -p "${missing_prefix}" "$(dirname "${missing_log}")"
  seed_platform_fixture "${missing_state}" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"
  run_hosted_install_branch "${work_root}/missing" "${fake_version}" "${artifacts}" "${missing_log}" "${missing_state}" "with-world" "${missing_os_release}"
  assert_env_sh_has_no_override_exports "${missing_prefix}" "${HOST_PATH}"
  assert_log_not_contains "${missing_log}" "unsupported schema_version"
  assert_additive_compatibility_preserved "${missing_state}" "keep-me" "keep-host-note" "${unknown_distro}" "${unknown_distro}" "${expected_pkg_manager}" "path_probe"

  local dev_fresh_prefix="${work_root}/dev-fresh"
  local dev_fresh_state="${dev_fresh_prefix}/install_state.json"
  local dev_fresh_log="${work_root}/dev-fresh/install.log"
  mkdir -p "${dev_fresh_prefix}" "$(dirname "${dev_fresh_log}")"
  run_dev_install_branch "${work_root}/dev-fresh" "${dev_fresh_log}" "${dev_fresh_state}" "with-world" "__skip__" "__skip__" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"
  assert_env_sh_has_no_override_exports "${dev_fresh_prefix}" "${HOST_PATH}"

  local pycompat_prefix="${work_root}/pycompat"
  local pycompat_state="${pycompat_prefix}/install_state.json"
  local pycompat_log="${work_root}/pycompat/install.log"
  mkdir -p "${pycompat_prefix}" "$(dirname "${pycompat_log}")"
  SUBSTRATE_TEST_REJECT_DATETIME_UTC=1 \
    run_hosted_install_branch "${work_root}/pycompat" "${fake_version}" "${artifacts}" "${pycompat_log}" "${pycompat_state}" "with-world" "${missing_os_release}"
  assert_env_sh_has_no_override_exports "${pycompat_prefix}" "${HOST_PATH}"
  assert_log_not_contains "${pycompat_log}" "Failed to write host state metadata"

  local dev_pycompat_prefix="${work_root}/dev-pycompat"
  local dev_pycompat_state="${dev_pycompat_prefix}/install_state.json"
  local dev_pycompat_log="${work_root}/dev-pycompat/install.log"
  mkdir -p "${dev_pycompat_prefix}" "$(dirname "${dev_pycompat_log}")"
  SUBSTRATE_TEST_REJECT_DATETIME_UTC=1 \
    run_dev_install_branch "${work_root}/dev-pycompat" "${dev_pycompat_log}" "${dev_pycompat_state}" "with-world" "__skip__" "__skip__" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"
  assert_env_sh_has_no_override_exports "${dev_pycompat_prefix}" "${HOST_PATH}"
  assert_log_not_contains "${dev_pycompat_log}" "Failed to write host state metadata"

  local dev_prefix="${work_root}/dev"
  local dev_state="${dev_prefix}/install_state.json"
  local dev_log="${work_root}/dev/install.log"
  mkdir -p "${dev_prefix}" "$(dirname "${dev_log}")"
  cat >"${dev_state}" <<EOF_DEV
{
  "schema_version": 1,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z",
  "unknown_top": "keep-me",
  "host_state": {
    "extra_note": "keep-host-note",
    "group": {
      "name": "substrate",
      "existed_before": true,
      "created_by_installer": false,
      "members_added": []
    },
    "linger": {
      "users": {}
    },
    "platform": {
      "os_release": {
        "id": "${CURRENT_OS_RELEASE_ID}",
        "id_like": "${CURRENT_OS_RELEASE_ID_LIKE}"
      },
      "pkg_manager": {
        "selected": "${expected_pkg_manager}",
        "source": "os_release"
      }
    }
  }
}
EOF_DEV
  run_dev_install_branch "${work_root}/dev" "${dev_log}" "${dev_state}" "with-world" "keep-me" "keep-host-note" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"
  assert_env_sh_has_no_override_exports "${dev_prefix}" "${HOST_PATH}"

  local dev_noworld_prefix="${work_root}/dev-noworld"
  local dev_noworld_state="${dev_noworld_prefix}/install_state.json"
  local dev_noworld_log="${work_root}/dev-noworld/install.log"
  mkdir -p "${dev_noworld_prefix}" "$(dirname "${dev_noworld_log}")"
  cat >"${dev_noworld_state}" <<EOF_DEV
{
  "schema_version": 1,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z",
  "unknown_top": "keep-me",
  "host_state": {
    "extra_note": "keep-host-note",
    "group": {
      "name": "substrate",
      "existed_before": true,
      "created_by_installer": false,
      "members_added": []
    },
    "linger": {
      "users": {}
    },
    "platform": {
      "os_release": {
        "id": "${CURRENT_OS_RELEASE_ID}",
        "id_like": "${CURRENT_OS_RELEASE_ID_LIKE}"
      },
      "pkg_manager": {
        "selected": "${expected_pkg_manager}",
        "source": "os_release"
      }
    }
  }
}
EOF_DEV
  run_dev_install_branch "${work_root}/dev-noworld" "${dev_noworld_log}" "${dev_noworld_state}" "no-world" "keep-me" "keep-host-note" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}" "${expected_pkg_manager}" "os_release"
  assert_env_sh_has_no_override_exports "${dev_noworld_prefix}" "${HOST_PATH}"

  local dry_run_prefix="${work_root}/dry-run"
  local dry_run_state="${dry_run_prefix}/install_state.json"
  local dry_run_log="${work_root}/dry-run/install.log"
  mkdir -p "${dry_run_prefix}" "$(dirname "${dry_run_log}")"
  write_os_release_fixture "${work_root}/dry-run/os-release" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}"
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${work_root}/dry-run/os-release" \
  SUBSTRATE_INSTALL_NO_PATH=1 \
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${work_root}/dry-run/systemctl.log" \
  SUBSTRATE_TEST_GROUP_LOG="${work_root}/dry-run/group-ops.log" \
  SUBSTRATE_TEST_LINGER_LOG="${work_root}/dry-run/linger.log" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
  SUBSTRATE_TEST_USER_GROUPS="wheel docker" \
  SUBSTRATE_TEST_GROUP_EXISTS=0 \
  FAKE_ROOT="${work_root}/dry-run/fakeroot" \
  HOME="${work_root}/dry-run/home" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --prefix "${dry_run_prefix}" \
    --version "${fake_version}" \
    --artifact-dir "${artifacts}" \
    --dry-run \
    >"${dry_run_log}" 2>&1 || {
      cat "${dry_run_log}" >&2
      fatal "dry-run install unexpectedly failed (see ${dry_run_log})"
    }
  assert_no_state_written "${dry_run_state}"
  assert_log_contains "${dry_run_log}" "Skipping host state metadata during dry-run"

  local non_linux_root="${work_root}/non-linux"
  local non_linux_stub="${non_linux_root}/stub-bin"
  local non_linux_prefix="${non_linux_root}"
  local non_linux_state="${non_linux_prefix}/install_state.json"
  local non_linux_log="${non_linux_root}/install.log"
  mkdir -p "${non_linux_stub}" "${non_linux_prefix}" "${non_linux_root}/home"
  STUB_BIN="${non_linux_stub}"
  prepare_stub_bin
  write_stub_uname "Plan9"
  write_stub_command "${expected_pkg_manager}"
  if HOME="${non_linux_root}/home" \
    PATH="${non_linux_stub}:${HOST_PATH}" \
    SHIM_ORIGINAL_PATH="${non_linux_stub}:${HOST_PATH}" \
    FAKE_ROOT="${non_linux_root}/fakeroot" \
    SUBSTRATE_INSTALL_NO_PATH=1 \
    "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
      --prefix "${non_linux_prefix}" \
      --version "${fake_version}" \
      --artifact-dir "${artifacts}" \
      >"${non_linux_log}" 2>&1; then
    cat "${non_linux_log}" >&2
    fatal "non-Linux install unexpectedly succeeded"
  fi
  assert_no_state_written "${non_linux_state}"
  assert_log_contains "${non_linux_log}" "Unsupported operating system: Plan9"

  local legacy_prefix="${work_root}/legacy"
  local legacy_state="${legacy_prefix}/install_state.json"
  local legacy_log="${work_root}/legacy/install.log"
  local legacy_os_release="${work_root}/legacy/os-release"
  local legacy_baseline="${work_root}/legacy/baseline.json"
  mkdir -p "${legacy_prefix}" "${work_root}/legacy/home" "$(dirname "${legacy_log}")"
  write_os_release_fixture "${legacy_os_release}" "${CURRENT_OS_RELEASE_ID}" "${CURRENT_OS_RELEASE_ID_LIKE}"
  cat >"${legacy_state}" <<EOF_LEGACY
{
  "schema_version": 0,
  "created_at": "legacy-created",
  "updated_at": "legacy-updated",
  "unknown_top": "keep-me",
  "host_state": {
    "extra_note": "keep-host-note",
    "group": {
      "members_added": ["legacy-user"]
    },
    "platform": {
      "os_release": {
        "id": "${CURRENT_OS_RELEASE_ID}",
        "id_like": "${CURRENT_OS_RELEASE_ID_LIKE}"
      },
      "pkg_manager": {
        "selected": "${expected_pkg_manager}",
        "source": "os_release"
      }
    }
  }
}
EOF_LEGACY
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${legacy_os_release}" \
  SUBSTRATE_INSTALL_NO_PATH=1 \
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${work_root}/legacy/systemctl.log" \
  SUBSTRATE_TEST_GROUP_LOG="${work_root}/legacy/group-ops.log" \
  SUBSTRATE_TEST_LINGER_LOG="${work_root}/legacy/linger.log" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
  SUBSTRATE_TEST_USER_GROUPS="substrate wheel" \
  SUBSTRATE_TEST_GROUP_EXISTS=1 \
  SUBSTRATE_TEST_LINGER_STATE="yes" \
  FAKE_ROOT="${work_root}/legacy/fakeroot" \
  HOME="${work_root}/legacy/home" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --prefix "${legacy_prefix}" \
    --version "${fake_version}" \
    --artifact-dir "${artifacts}" \
    >"${legacy_log}" 2>&1 || {
      cat "${legacy_log}" >&2
      fatal "metadata upgrade install failed (see ${legacy_log})"
  }
  assert_metadata_upgraded "${legacy_state}" "substrate-state"
  assert_log_contains "${legacy_log}" "unsupported schema_version 0"
  cp "${legacy_state}" "${legacy_baseline}"

  local write_fail_log="${work_root}/legacy/write-fail.log"
  local replace_fail_log="${work_root}/legacy/replace-fail.log"
  : >"${write_fail_log}"; : >"${replace_fail_log}"

  mkdir -p "${work_root}/legacy/home-write"
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${legacy_os_release}" \
  SUBSTRATE_INSTALL_NO_PATH=1 \
  SUBSTRATE_TEST_FAIL_HOST_STATE_WRITE=1 \
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${work_root}/legacy/systemctl-write.log" \
  SUBSTRATE_TEST_GROUP_LOG="${work_root}/legacy/group-write.log" \
  SUBSTRATE_TEST_LINGER_LOG="${work_root}/legacy/linger-write.log" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
  SUBSTRATE_TEST_USER_GROUPS="substrate wheel" \
  SUBSTRATE_TEST_GROUP_EXISTS=1 \
  SUBSTRATE_TEST_LINGER_STATE="yes" \
  FAKE_ROOT="${work_root}/legacy/fakeroot-write" \
  HOME="${work_root}/legacy/home-write" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --prefix "${legacy_prefix}" \
    --version "${fake_version}" \
    --artifact-dir "${artifacts}" \
    >"${write_fail_log}" 2>&1 || {
      cat "${write_fail_log}" >&2
      fatal "host-state write failure scenario unexpectedly failed (see ${write_fail_log})"
    }
  assert_log_contains "${write_fail_log}" "Failed to write host state metadata"
  cmp -s "${legacy_baseline}" "${legacy_state}" || fatal "canonical metadata changed after write failure"
  [[ ! -e "${legacy_state}.tmp" ]] || fatal "temp file should be removed after write failure"

  mkdir -p "${work_root}/legacy/home-replace"
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${legacy_os_release}" \
  SUBSTRATE_INSTALL_NO_PATH=1 \
  SUBSTRATE_TEST_FAIL_HOST_STATE_REPLACE=1 \
  SUBSTRATE_TEST_FAIL_MV_SOURCE="${legacy_state}.tmp" \
  SUBSTRATE_TEST_FAIL_MV_DEST="${legacy_state}" \
  SUBSTRATE_TEST_SYSTEMCTL_LOG="${work_root}/legacy/systemctl-replace.log" \
  SUBSTRATE_TEST_GROUP_LOG="${work_root}/legacy/group-replace.log" \
  SUBSTRATE_TEST_LINGER_LOG="${work_root}/legacy/linger-replace.log" \
  SUBSTRATE_TEST_PRIMARY_USER="substrate-state" \
  SUBSTRATE_TEST_USER_GROUPS="substrate wheel" \
  SUBSTRATE_TEST_GROUP_EXISTS=1 \
  SUBSTRATE_TEST_LINGER_STATE="yes" \
  FAKE_ROOT="${work_root}/legacy/fakeroot-replace" \
  HOME="${work_root}/legacy/home-replace" \
  PATH="${harness_path}" \
  SHIM_ORIGINAL_PATH="${harness_path}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" \
    --prefix "${legacy_prefix}" \
    --version "${fake_version}" \
    --artifact-dir "${artifacts}" \
    >"${replace_fail_log}" 2>&1 || {
      cat "${replace_fail_log}" >&2
      fatal "host-state replace failure scenario unexpectedly failed (see ${replace_fail_log})"
    }
  assert_log_contains "${replace_fail_log}" "Failed to replace host state metadata"
  cmp -s "${legacy_baseline}" "${legacy_state}" || fatal "canonical metadata changed after replace failure"
  [[ ! -e "${legacy_state}.tmp" ]] || fatal "temp file should be removed after replace failure"

  log "Metadata scenario completed"
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
  local prefix="${work_root}/custom-substrate-home"
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
  SUBSTRATE_HOME="${prefix}" \
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

assert_env_sh_has_no_override_exports() {
  local prefix="$1"
  local host_path="$2"
  local env_sh="${prefix}/env.sh"
  if [[ ! -f "${env_sh}" ]]; then
    fatal "env.sh missing after install: ${env_sh}"
  fi
  if grep -Eq '^export[[:space:]]+SUBSTRATE_OVERRIDE_' "${env_sh}"; then
    log "env.sh contains SUBSTRATE_OVERRIDE_* exports:"
    sed -n '1,120p' "${env_sh}" >&2 || true
    fatal "env.sh must not export SUBSTRATE_OVERRIDE_* by default (WCU4)"
  fi
  if env -i PATH="${host_path}" bash -lc "source \"${env_sh}\"; env | grep -q '^SUBSTRATE_OVERRIDE_'" >/dev/null 2>&1; then
    fatal "Sourcing env.sh must not export SUBSTRATE_OVERRIDE_* (WCU4)"
  fi
  log "Verified env.sh does not export SUBSTRATE_OVERRIDE_* (${env_sh})"
}

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
