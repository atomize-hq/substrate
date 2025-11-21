#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

log() {
  printf '[install-smoke] %s\n' "$*" >&2
}

fatal() {
  log "ERROR: $*"
  exit 1
}

usage() {
  cat <<'USAGE' >&2
Usage: tests/installers/install_smoke.sh --scenario <default|no-world> [--keep-root]

Scenarios:
  default   Full install with world provisioning.
  no-world  Install with --no-world.

The harness stubs system commands and uses a fake release bundle so it never
touches the host. It verifies the manager manifest is installed and that
`substrate health --json` succeeds against the installed prefix.
USAGE
}

SCENARIO="default"
KEEP_ROOT=0
FAKE_VERSION="${FAKE_VERSION:-0.0.0-test}"

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
  default|no-world) ;;
  *) usage; fatal "Unsupported scenario: ${SCENARIO}" ;;
esac

if [[ "$(uname -s)" != "Linux" ]]; then
  log "Installer smoke harness currently targets Linux only; skipping."
  exit 0
fi

WORK_ROOT="$(mktemp -d "/tmp/substrate-installer-${SCENARIO}.XXXXXX")"
PREFIX="${WORK_ROOT}/prefix"
ARTIFACT_DIR="${WORK_ROOT}/artifacts"
FAKE_ROOT="${WORK_ROOT}/fakeroot"
STUB_BIN="${WORK_ROOT}/stub-bin"
HOME_DIR="${WORK_ROOT}/home"
mkdir -p "${PREFIX}" "${ARTIFACT_DIR}" "${FAKE_ROOT}" "${STUB_BIN}" "${HOME_DIR}"

cleanup() {
  if [[ "${KEEP_ROOT}" -eq 0 ]]; then
    rm -rf "${WORK_ROOT}"
  else
    log "Preserving ${WORK_ROOT} for inspection (--keep-root set)."
  fi
}
trap cleanup EXIT

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

write_stub_command() {
  local name="$1"
  cat >"${STUB_BIN}/${name}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${STUB_BIN}/${name}"
}

write_stub_jq() {
  cat >"${STUB_BIN}/jq" <<'EOF'
#!/usr/bin/env bash
cat
EOF
  chmod +x "${STUB_BIN}/jq"
}

write_stub_sudo() {
  cat >"${STUB_BIN}/sudo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
FAKE_ROOT="${FAKE_ROOT:-}"
if [[ $# -lt 1 ]]; then
  exit 0
fi
cmd="$1"
shift || true

rewrite_dest_args() {
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

case "${cmd}" in
  systemctl)
    exit 0
    ;;
  install)
    args=("$@")
    rewrite_dest_args args
    exec install "${args[@]}"
    ;;
  *)
    args=("$@")
    exec "${cmd}" "${args[@]}"
    ;;
esac
EOF
  chmod +x "${STUB_BIN}/sudo"
}

prepare_stub_bin() {
  write_stub_sudo
  write_stub_jq
  for cmd in curl fusermount fuse-overlayfs ip nft systemctl limactl; do
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

write_stub_binary() {
  local path="$1"
  cat >"${path}" <<'EOF'
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
EOF
  chmod +x "${path}"
}

build_fake_release() {
  local label
  label="$(detect_bundle_label)"
  local bundle_root="${WORK_ROOT}/bundle/${label}"
  mkdir -p "${bundle_root}/bin" "${bundle_root}/config"

  cp "${REPO_ROOT}/config/manager_hooks.yaml" "${bundle_root}/config/manager_hooks.yaml"
  cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" "${bundle_root}/config/world-deps.yaml"

  write_stub_binary "${bundle_root}/bin/substrate"
  write_stub_binary "${bundle_root}/bin/host-proxy"
  write_stub_binary "${bundle_root}/bin/world-agent"

  local archive="substrate-v${FAKE_VERSION}-${label}.tar.gz"
  tar -czf "${ARTIFACT_DIR}/${archive}" -C "${WORK_ROOT}/bundle" "${label}"

  local checksum
  checksum="$(compute_sha256 "${ARTIFACT_DIR}/${archive}")"
  printf '%s  %s\n' "${checksum}" "${archive}" >"${ARTIFACT_DIR}/SHA256SUMS"
}

assert_install_config() {
  local config="${PREFIX}/config.toml"
  local expected_flag="true"
  if [[ "${SCENARIO}" == "no-world" ]]; then
    expected_flag="false"
  fi

  if [[ ! -f "${config}" ]]; then
    if [[ -f "${PREFIX}/config.json" ]]; then
      fatal "expected ${config} but found legacy config.json; installer should emit TOML"
    fi
    fatal "install config missing after install: ${config}"
  fi

  python3 - <<'PY' "${config}" "${expected_flag}"
import sys
from pathlib import Path

path = Path(sys.argv[1])
expected_enabled = sys.argv[2].lower() == "true"
body = path.read_text()
world_enabled = None
root_mode = None
root_path = None
section = None


def parse_bool(raw):
    val = raw.strip().lower()
    if val in ("true", "false"):
        return val == "true"
    raise SystemExit(f"invalid boolean value: {raw}")


def parse_string(raw):
    val = raw.strip()
    if val.startswith('"') and val.endswith('"') and len(val) >= 2:
        return val[1:-1]
    return val


for raw in body.splitlines():
    line = raw.split("#", 1)[0].strip()
    if not line:
        continue
    if line.startswith("[") and line.endswith("]"):
        section = line[1:-1].strip()
        continue
    if "=" not in line or section is None:
        continue
    key, value = line.split("=", 1)
    key = key.strip()
    value = value.strip()
    if section == "install" and key == "world_enabled":
        world_enabled = parse_bool(value)
    elif section == "world":
        if key == "root_mode":
            root_mode = parse_string(value)
        elif key == "root_path":
            root_path = parse_string(value)

if world_enabled is None:
    raise SystemExit("world_enabled missing under [install]")
if world_enabled != expected_enabled:
    raise SystemExit(f"world_enabled={world_enabled} (expected {expected_enabled})")
if root_mode is None:
    raise SystemExit("world.root_mode missing under [world]")
if root_mode != "project":
    raise SystemExit(f"world.root_mode={root_mode} (expected project)")
if root_path is None:
    raise SystemExit("world.root_path missing under [world]")
if root_path != "":
    raise SystemExit(f"world.root_path={root_path!r} (expected empty string)")
PY

  log "Verified install config at ${config} (world_enabled=${expected_flag}; root_mode=project root_path=\"\")"
}

assert_manifest_present() {
  local manifest="${PREFIX}/versions/${FAKE_VERSION}/config/manager_hooks.yaml"
  if [[ ! -f "${manifest}" ]]; then
    fatal "manager manifest missing after install: ${manifest}"
  fi
  log "Verified manager manifest at ${manifest}"
}

run_health_smoke() {
  local substrate_bin="${PREFIX}/bin/substrate"
  local output
  if ! output="$(SUBSTRATE_ROOT="${PREFIX}" PATH="${PREFIX}/bin:${PATH}" "${substrate_bin}" health --json)"; then
    fatal "substrate health failed"
  fi
  python3 - <<'PY' "${output}"
import json, sys
try:
    json.loads(sys.argv[1])
except json.JSONDecodeError as err:
    sys.stderr.write(f"health JSON parse error: {err}\n")
    sys.exit(1)
PY
  log "Health check succeeded: ${output}"
}

run_install() {
  local args=("--prefix" "${PREFIX}" "--version" "${FAKE_VERSION}" "--artifact-dir" "${ARTIFACT_DIR}")
  if [[ "${SCENARIO}" == "no-world" ]]; then
    args+=("--no-world")
  fi

  HOME="${HOME_DIR}" \
  PATH="${STUB_BIN}:${PATH}" \
  FAKE_ROOT="${FAKE_ROOT}" \
  "${REPO_ROOT}/scripts/substrate/install-substrate.sh" "${args[@]}"
}

prepare_stub_bin
build_fake_release
run_install
assert_install_config
assert_manifest_present
run_health_smoke

log "Scenario ${SCENARIO} completed using PREFIX=${PREFIX}"
log "Temp root: ${WORK_ROOT}"
