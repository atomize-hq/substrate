#!/bin/bash
set -euo pipefail

SCRIPT_NAME="dev-install-limactl-provenance-home-r3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALLER_PATH="${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh"
BASE_COMMIT="731938cfe7fafb3129df0383ca7ae0566e011185"
FIXED_PATH="/usr/bin:/bin:/usr/sbin:/sbin"

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
sudo -n /usr/bin/true 2>/dev/null \
  || fail "focused privileged regression requires a live non-interactive sudo authorization"

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-limactl-uid-r3.XXXXXX")"
cleanup() {
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

BOUND_UID="$(id -u)"
BOUND_ACCOUNT="$(id -un)"
[[ "${BOUND_UID}" =~ ^[0-9]+$ && "${BOUND_UID}" -ne 0 && -n "${BOUND_ACCOUNT}" ]] \
  || fail "focused regression requires a bound non-root installer principal"
[[ "$(id -u "${BOUND_ACCOUNT}")" == "${BOUND_UID}" && "$(id -un "${BOUND_UID}")" == "${BOUND_ACCOUNT}" ]] \
  || fail "focused regression cannot bind installer account and UID"

FAKE_LIMACTL="${WORK_ROOT}/fixed-limactl"
FAKE_ENV_RECORD="${WORK_ROOT}/fixed-limactl.env"
CALLER_HOME="${WORK_ROOT}/caller-home"
mkdir -p "${CALLER_HOME}"

cat >"${FAKE_LIMACTL}" <<EOF
#!/bin/bash
set -euo pipefail

if [[ "\${EUID}" -eq 0 ]]; then
  printf 'limactl: must not run as the root user.\n' >&2
  exit 77
fi
if [[ "\${EUID}" -ne "${BOUND_UID}" || "\$(id -un)" != "${BOUND_ACCOUNT}" ]]; then
  printf 'fixed limactl ran as the wrong installer principal\n' >&2
  exit 78
fi
if [[ "\${HOME-}" != "/var/empty" ]]; then
  printf 'fixed limactl HOME is not the safe validated home\n' >&2
  exit 79
fi
if [[ "\${PATH-}" != "${FIXED_PATH}" ]]; then
  printf 'fixed privileged PATH is not preserved\n' >&2
  exit 80
fi
if [[ -n "\${CALLER_HOME-}" || -n "\${AMBIENT_LEAK_SENTINEL-}" || -n "\${USER-}" || -n "\${LOGNAME-}" || -n "\${SUDO_USER-}" ]]; then
  printf 'ambient caller environment leaked into limactl probe\n' >&2
  exit 81
fi
[[ "\${1-}" == "--version" && \$# -eq 1 ]] || exit 64
{
  printf 'EUID=%s\n' "\${EUID}"
  printf 'ACCOUNT=%s\n' "\$(id -un)"
  printf 'HOME=%s\n' "\${HOME-}"
  printf 'PATH=%s\n' "\${PATH-}"
  printf 'CALLER_HOME=%s\n' "\${CALLER_HOME-}"
  printf 'AMBIENT_LEAK_SENTINEL=%s\n' "\${AMBIENT_LEAK_SENTINEL-}"
  printf 'USER=%s\n' "\${USER-}"
  printf 'LOGNAME=%s\n' "\${LOGNAME-}"
  printf 'SUDO_USER=%s\n' "\${SUDO_USER-}"
} >"${FAKE_ENV_RECORD}"
printf '%s\n' 'limactl version fixture 2.1.1'
EOF
chmod 0755 "${FAKE_LIMACTL}"
sudo -n /usr/sbin/chown root:wheel "${FAKE_LIMACTL}"

probe_line_from() {
  local source_path="$1"
  python3 - "${source_path}" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text(encoding="utf-8")
start = source.find("publish_mac_publisher_install_provenance_v1() {")
end = source.find("\nclear_managed_prefix_linux_binary_cache() {", start)
if start < 0 or end < 0:
    raise SystemExit("canonical publisher provenance function is absent")
lines = [
    line.strip()
    for line in source[start:end].splitlines()
    if line.strip().startswith('lima_version="$(')
]
if len(lines) != 1:
    raise SystemExit("canonical publisher provenance probe is absent or ambiguous")
print(lines[0])
PY
}

assert_current_source_shape() {
  local source_path="$1"
  local probe_line="$2"
  python3 - "${source_path}" "${probe_line}" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text(encoding="utf-8")
actual_probe = sys.argv[2]
expected_probe = 'lima_version="$(/usr/bin/sudo -u "#${installer_uid}" -- /usr/bin/env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin HOME=/var/empty "$limactl_path" --version)"'
if actual_probe != expected_probe:
    raise SystemExit("limactl --version is not the exact scrubbed bound-UID privilege transition")
start = source.find("publish_mac_publisher_install_provenance_v1() {")
end = source.find("\nclear_managed_prefix_linux_binary_cache() {", start)
if start < 0 or end < 0:
    raise SystemExit("canonical publisher provenance function is absent")
source = source[start:end]
required_before_probe = (
    'installer_account="${28}"',
    'installer_uid="${29}"',
    'case "$installer_uid" in',
    'test "$installer_uid" -ne 0',
    'test "$(/usr/bin/id -u "$installer_account")" = "$installer_uid"',
    'test "$(/usr/bin/id -un "$installer_uid")" = "$installer_account"',
    'require_root_owned_immutable_path "$limactl_path"',
    'test -d /var/empty && test ! -L /var/empty || {',
    'limactl_home_owner="$(stat -f \'%u\' /var/empty)"',
    'limactl_home_mode="$(stat -f \'%Lp\' /var/empty)"',
    'test "$limactl_home_owner" -eq 0 && test $((0$limactl_home_mode & 022)) -eq 0 || {',
)
probe_index = source.index(expected_probe)
for token in required_before_probe:
    if token not in source:
        raise SystemExit(f"privileged limactl identity or fixed-image validation is incomplete: {token}")
    if source.index(token) > probe_index:
        raise SystemExit(f"privileged limactl validation follows the probe: {token}")
expected_handoff = '"${retained_substrate_sha}" "${retained_substrate_identity}" \\\n    "${INSTALL_BOOTSTRAP_ACCOUNT}" "${INSTALL_BOOTSTRAP_UID}" \\'
if expected_handoff not in source:
    raise SystemExit("privileged probe does not receive the already-validated install bootstrap identity")
PY
}

run_probe_as_privileged_root() {
  local probe_line="$1"
  local stdout_path="$2"
  local stderr_path="$3"
  local harness="${WORK_ROOT}/privileged-probe.sh"

  cat >"${harness}" <<EOF
#!/bin/bash
set -euo pipefail
[[ "\${EUID}" -eq 0 ]] || exit 90
installer_account="\$1"
installer_uid="\$2"
limactl_path="\$3"
${probe_line}
printf '%s\n' "\${lima_version}"
EOF
  chmod 0755 "${harness}"

  set +e
  sudo -n -- /usr/bin/env \
    PATH="${FIXED_PATH}" \
    HOME="${CALLER_HOME}" \
    USER=root \
    LOGNAME=root \
    SUDO_USER="caller-selected-alternate-must-not-survive" \
    CALLER_HOME="${CALLER_HOME}" \
    AMBIENT_LEAK_SENTINEL="must-not-reach-fixed-limactl" \
    /bin/bash "${harness}" "${BOUND_ACCOUNT}" "${BOUND_UID}" "${FAKE_LIMACTL}" \
    >"${stdout_path}" 2>"${stderr_path}"
  local status=$?
  set -e
  return "${status}"
}

source_path="${INSTALLER_PATH}"
if [[ "${MODE}" == "base-red" ]]; then
  source_path="${WORK_ROOT}/base-dev-install-substrate.sh"
  git -C "${REPO_ROOT}" show "${BASE_COMMIT}:scripts/substrate/dev-install-substrate.sh" >"${source_path}" \
    || fail "cannot read exact base installer ${BASE_COMMIT}"
fi

PROBE_LINE="$(probe_line_from "${source_path}")" \
  || fail "installer source lacks a unique privileged limactl --version probe"

if [[ "${MODE}" == "base-red" ]]; then
  if run_probe_as_privileged_root "${PROBE_LINE}" "${WORK_ROOT}/base.out" "${WORK_ROOT}/base.err"; then
    fail "exact base root probe unexpectedly survived the Lima non-root requirement"
  else
    base_status=$?
  fi
  [[ "${base_status}" -eq 77 ]] \
    || fail "exact base probe did not fail as root with fixture status 77 (got ${base_status})"
  grep -Fxq -- 'limactl: must not run as the root user.' "${WORK_ROOT}/base.err" \
    || fail "exact base probe did not reproduce the Lima 2.1.1 root-user rejection"
  printf '[%s] PASS: exact base privileged path reproduced root EUID rejection\n' "${SCRIPT_NAME}"
  exit 0
fi

if ! run_probe_as_privileged_root "${PROBE_LINE}" "${WORK_ROOT}/green.out" "${WORK_ROOT}/green.err"; then
  sed -n '1,80p' "${WORK_ROOT}/green.err" >&2
  fail "corrected privileged limactl probe did not transition to the bound non-root UID"
fi
assert_current_source_shape "${INSTALLER_PATH}" "${PROBE_LINE}" \
  || fail "canonical installer lost the bound-UID fixed limactl probe"
grep -Fxq -- 'limactl version fixture 2.1.1' "${WORK_ROOT}/green.out" \
  || fail "corrected limactl probe did not retain the fixed image version output"
{
  printf 'EUID=%s\n' "${BOUND_UID}"
  printf 'ACCOUNT=%s\n' "${BOUND_ACCOUNT}"
  printf 'HOME=/var/empty\n'
  printf 'PATH=%s\n' "${FIXED_PATH}"
  printf 'CALLER_HOME=\n'
  printf 'AMBIENT_LEAK_SENTINEL=\n'
  printf 'USER=\n'
  printf 'LOGNAME=\n'
  printf 'SUDO_USER=\n'
} >"${WORK_ROOT}/expected.env"
cmp -s "${WORK_ROOT}/expected.env" "${FAKE_ENV_RECORD}" \
  || fail "corrected limactl probe used the wrong UID or inherited caller environment"

printf '[%s] PASS: root-owned fixed limactl ran as bound uid=%s account=%s with scrubbed HOME/PATH/ambient environment\n' \
  "${SCRIPT_NAME}" "${BOUND_UID}" "${BOUND_ACCOUNT}"
