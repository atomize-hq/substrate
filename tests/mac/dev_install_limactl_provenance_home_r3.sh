#!/bin/bash
set -euo pipefail

SCRIPT_NAME="dev-install-limactl-provenance-home-r3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALLER_PATH="${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh"
BASE_COMMIT="1126b907df6e38043e9071da222f6c6a377b341c"
FIXED_PATH="/usr/bin:/bin:/usr/sbin:/sbin"
EXPECTED_PROBE="lima_version=\"\$(env -i PATH=${FIXED_PATH} HOME=/var/empty \"\$limactl_path\" --version)\""

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '[%s] SKIP: macOS provenance probe fixture\n' "${SCRIPT_NAME}"
  exit 0
fi

MODE="green"
if [[ "${1-}" == "--expect-base-failure" ]]; then
  MODE="base-red"
  shift
fi
[[ $# -eq 0 ]] || fail "usage: ${0##*/} [--expect-base-failure]"

[[ -f "${INSTALLER_PATH}" && ! -L "${INSTALLER_PATH}" ]] \
  || fail "canonical installer is absent or linked"

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-limactl-home-r3.XXXXXX")"
cleanup() {
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

FAKE_LIMACTL="${WORK_ROOT}/fixed-limactl"
FAKE_ENV_RECORD="${WORK_ROOT}/fixed-limactl.env"
CALLER_HOME="${WORK_ROOT}/caller-home"
mkdir -p "${CALLER_HOME}"

cat >"${FAKE_LIMACTL}" <<EOF
#!/bin/bash
set -euo pipefail

if [[ "\${HOME-}" != "/var/empty" ]]; then
  printf 'panic: \$HOME is not defined\\n' >&2
  exit 2
fi
if [[ "\${PATH-}" != "${FIXED_PATH}" ]]; then
  printf 'fixed privileged PATH is not preserved\\n' >&2
  exit 3
fi
if [[ -n "\${CALLER_HOME-}" || -n "\${AMBIENT_LEAK_SENTINEL-}" ]]; then
  printf 'ambient caller environment leaked into limactl probe\\n' >&2
  exit 4
fi
[[ "\${1-}" == "--version" && \$# -eq 1 ]] || exit 64
{
  printf 'HOME=%s\\n' "\${HOME-}"
  printf 'PATH=%s\\n' "\${PATH-}"
  printf 'CALLER_HOME=%s\\n' "\${CALLER_HOME-}"
  printf 'AMBIENT_LEAK_SENTINEL=%s\\n' "\${AMBIENT_LEAK_SENTINEL-}"
} >"${FAKE_ENV_RECORD}"
printf '%s\\n' 'limactl version fixture 2.1.1'
EOF
chmod 0755 "${FAKE_LIMACTL}"

probe_line_from() {
  local source_path="$1"
  python3 - "${source_path}" <<'PY'
from pathlib import Path
import re
import sys

source = Path(sys.argv[1]).read_text(encoding="utf-8")
start = source.find("publish_mac_publisher_install_provenance_v1() {")
end = source.find("\nclear_managed_prefix_linux_binary_cache() {", start)
if start < 0 or end < 0:
    raise SystemExit("canonical publisher provenance function is absent")
lines = [
    line.strip()
    for line in source[start:end].splitlines()
    if line.strip().startswith('lima_version="$(env -i ')
]
if len(lines) != 1:
    raise SystemExit("canonical publisher provenance probe is absent or ambiguous")
print(lines[0])
PY
}

assert_current_source_shape() {
  local source_path="$1"
  local probe_line="$2"
  python3 - "${source_path}" "${EXPECTED_PROBE}" "${probe_line}" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text(encoding="utf-8")
expected_probe = sys.argv[2]
actual_probe = sys.argv[3]
if actual_probe != expected_probe:
    raise SystemExit("privileged limactl --version probe is not the exact scrubbed /var/empty command")
start = source.find("publish_mac_publisher_install_provenance_v1() {")
end = source.find("\nclear_managed_prefix_linux_binary_cache() {", start)
if start < 0 or end < 0:
    raise SystemExit("canonical publisher provenance function is absent")
source = source[start:end]
required = (
    'test -d /var/empty && test ! -L /var/empty || {',
    'privileged limactl HOME is absent or linked: /var/empty\\n" >&2; exit 1;',
    'limactl_home_owner="$(stat -f \'%u\' /var/empty)"',
    'limactl_home_mode="$(stat -f \'%Lp\' /var/empty)"',
    'test "$limactl_home_owner" -eq 0 && test $((0$limactl_home_mode & 022)) -eq 0 || {',
    'privileged limactl HOME is not root-controlled state: /var/empty\\n" >&2; exit 1;',
)
for token in required:
    if token not in source:
        raise SystemExit(f"privileged limactl HOME validation is incomplete: {token}")
if source.index(required[-1]) > source.index(expected_probe):
    raise SystemExit("privileged limactl HOME validation must precede the version probe")
PY
}

run_probe_line() {
  local probe_line="$1"
  local stdout_path="$2"
  local stderr_path="$3"

  set +e
  (
    HOME="${CALLER_HOME}"
    CALLER_HOME="${CALLER_HOME}"
    AMBIENT_LEAK_SENTINEL="must-not-reach-fixed-limactl"
    export HOME CALLER_HOME AMBIENT_LEAK_SENTINEL
    limactl_path="${FAKE_LIMACTL}"
    set +e
    eval "${probe_line}"
    probe_status=$?
    set -e
    if [[ "${probe_status}" -ne 0 ]]; then
      exit "${probe_status}"
    fi
    printf '%s\n' "${lima_version}"
  ) >"${stdout_path}" 2>"${stderr_path}"
  local status=$?
  set -e
  return "${status}"
}

if [[ "${MODE}" == "base-red" ]]; then
  BASE_INSTALLER="${WORK_ROOT}/base-dev-install-substrate.sh"
  git -C "${REPO_ROOT}" show "${BASE_COMMIT}:scripts/substrate/dev-install-substrate.sh" >"${BASE_INSTALLER}" \
    || fail "cannot read exact base installer ${BASE_COMMIT}"
  BASE_PROBE="$(probe_line_from "${BASE_INSTALLER}")" \
    || fail "exact base lacks a unique privileged limactl --version probe"
  if run_probe_line "${BASE_PROBE}" "${WORK_ROOT}/base.out" "${WORK_ROOT}/base.err"; then
    fail "exact base probe unexpectedly survived the Lima 2.1.1 HOME requirement"
  else
    base_status=$?
  fi
  [[ "${base_status}" -eq 2 ]] \
    || fail "exact base probe did not reproduce the HOME panic status (got ${base_status})"
  grep -Fxq -- 'panic: $HOME is not defined' "${WORK_ROOT}/base.err" \
    || fail "exact base probe did not reproduce the Lima 2.1.1 HOME panic"
  if [[ "${BASE_PROBE}" == *'HOME='* ]]; then
    fail "exact base probe unexpectedly supplies HOME"
  fi
  printf '[%s] PASS: exact base reproduces the scrubbed-HOME panic\n' "${SCRIPT_NAME}"
  exit 0
fi

PROBE_LINE="$(probe_line_from "${INSTALLER_PATH}")" \
  || fail "canonical installer lacks a unique privileged limactl --version probe"
assert_current_source_shape "${INSTALLER_PATH}" "${PROBE_LINE}" \
  || fail "canonical installer lost the root-controlled /var/empty provenance probe"
if ! run_probe_line "${PROBE_LINE}" "${WORK_ROOT}/green.out" "${WORK_ROOT}/green.err"; then
  sed -n '1,80p' "${WORK_ROOT}/green.err" >&2
  fail "corrected limactl probe did not survive the HOME requirement"
fi
grep -Fxq -- 'limactl version fixture 2.1.1' "${WORK_ROOT}/green.out" \
  || fail "corrected limactl probe did not retain the fixed image version output"
{
  printf 'HOME=/var/empty\n'
  printf 'PATH=%s\n' "${FIXED_PATH}"
  printf 'CALLER_HOME=\n'
  printf 'AMBIENT_LEAK_SENTINEL=\n'
} >"${WORK_ROOT}/expected.env"
cmp -s "${WORK_ROOT}/expected.env" "${FAKE_ENV_RECORD}" \
  || fail "corrected limactl probe inherited caller HOME or an ambient environment value"

printf '[%s] PASS: fixed limactl probe uses validated /var/empty with no caller HOME leak\n' "${SCRIPT_NAME}"
