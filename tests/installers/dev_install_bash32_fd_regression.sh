#!/bin/bash
set -euo pipefail

SCRIPT_NAME="dev-install-bash32-fd-regression"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE_INSTALLER="${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh"
DEFAULT_BASH="/bin/bash"

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '[%s] SKIP: macOS /bin/bash 3.2 regression\n' "${SCRIPT_NAME}"
  exit 0
fi
if [[ ! -x "${DEFAULT_BASH}" ]]; then
  fail "macOS default shell is unavailable: ${DEFAULT_BASH}"
fi
case "$("${DEFAULT_BASH}" -c 'printf %s "${BASH_VERSION}"')" in
  3.2.*) ;;
  *) fail "this regression must exercise macOS /bin/bash 3.2" ;;
esac
if [[ ! -x "${SOURCE_INSTALLER}" ]]; then
  fail "development installer is not executable: ${SOURCE_INSTALLER}"
fi
if [[ "$(id -u)" -eq 0 ]]; then
  printf '[%s] SKIP: installer bootstrap context rejects root principals\n' "${SCRIPT_NAME}"
  exit 0
fi

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-dev-install-bash32.XXXXXX")"
WORK_ROOT="$(cd "${WORK_ROOT}" && pwd)"
FIXTURE_REPO="${WORK_ROOT}/repo"
FIXTURE_PREFIX="${WORK_ROOT}/prefix"
FIXTURE_TMPDIR="${WORK_ROOT}/bootstrap-tmp"
FIXTURE_BIN_DIR="${WORK_ROOT}/fake-bin"
FIXTURE_INSTALLER="${FIXTURE_REPO}/scripts/substrate/dev-install-substrate.sh"
FAKE_SUBSTRATE_PATH="${FIXTURE_REPO}/target/debug/substrate"
FAKE_SUBSTRATE_ARGV="${WORK_ROOT}/fake-substrate.argv"
FAKE_SUBSTRATE_ENV="${WORK_ROOT}/fake-substrate.env"
FAKE_SUBSTRATE_FD="${WORK_ROOT}/fake-substrate.fd"
CALLER_FD_SENTINEL="${WORK_ROOT}/caller-owned-fd-sentinel"

cleanup() {
  exec 9<&- 2>/dev/null || true
  rm -rf "${WORK_ROOT}"
}
trap cleanup EXIT

mkdir -p \
  "${FIXTURE_REPO}/scripts/substrate" \
  "${FIXTURE_REPO}/config" \
  "${FIXTURE_PREFIX}" \
  "${FIXTURE_TMPDIR}" \
  "${FIXTURE_BIN_DIR}"
cp "${SOURCE_INSTALLER}" "${FIXTURE_INSTALLER}"
cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" \
  "${FIXTURE_REPO}/scripts/substrate/world-deps.yaml"
cp -R "${REPO_ROOT}/config/." "${FIXTURE_REPO}/config/"
chmod 0755 "${FIXTURE_INSTALLER}"

cat >"${FIXTURE_BIN_DIR}/cargo" <<'FAKE_CARGO'
#!/bin/bash
set -euo pipefail

: "${FAKE_SUBSTRATE_PATH:?}"
: "${FAKE_SUBSTRATE_ARGV:?}"
: "${FAKE_SUBSTRATE_ENV:?}"
: "${FAKE_SUBSTRATE_FD:?}"

mkdir -p "$(dirname "${FAKE_SUBSTRATE_PATH}")"
cat >"${FAKE_SUBSTRATE_PATH}" <<'FAKE_SUBSTRATE'
#!/bin/bash
set -euo pipefail

: "${FAKE_SUBSTRATE_ARGV:?}"
: "${FAKE_SUBSTRATE_ENV:?}"
: "${FAKE_SUBSTRATE_FD:?}"
: "${FAKE_SUBSTRATE_CALLER_FD_SENTINEL:?}"

if python3 -c '
import os
import sys

actual = os.fstat(9)
expected = os.stat(sys.argv[1])
raise SystemExit(
    0
    if (actual.st_dev, actual.st_ino) == (expected.st_dev, expected.st_ino)
    else 1
)
' "${FAKE_SUBSTRATE_CALLER_FD_SENTINEL}"; then
  printf 'sentinel\n' >"${FAKE_SUBSTRATE_FD}"
else
  printf 'missing-or-replaced\n' >"${FAKE_SUBSTRATE_FD}"
  exit 91
fi

{
  for argument in "$@"; do
    printf '%s\n' "${argument}"
  done
} >"${FAKE_SUBSTRATE_ARGV}"

{
  printf 'SUBSTRATE_HOME=%s\n' "${SUBSTRATE_HOME-}"
  printf 'SUBSTRATE_ROOT=%s\n' "${SUBSTRATE_ROOT-}"
  printf 'SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=%s\n' \
    "${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT-}"
  printf 'SUBSTRATE_INSTALL_PRIMARY_USER=%s\n' "${SUBSTRATE_INSTALL_PRIMARY_USER-}"
  printf 'SUBSTRATE_INSTALL_PRIMARY_UID=%s\n' "${SUBSTRATE_INSTALL_PRIMARY_UID-}"
  printf 'SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=%s\n' \
    "${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1-}"
} >"${FAKE_SUBSTRATE_ENV}"
FAKE_SUBSTRATE
chmod 0755 "${FAKE_SUBSTRATE_PATH}"
FAKE_CARGO
chmod 0755 "${FIXTURE_BIN_DIR}/cargo"

base64url() {
  printf '%s' "$1" | base64 | tr '+/' '_-' | tr -d '=\n'
}

CURRENT_ACCOUNT="$(id -un)"
CURRENT_UID="$(id -u)"
ENCODED_PREFIX="$(base64url "${FIXTURE_PREFIX}")"
ENCODED_ACCOUNT="$(base64url "${CURRENT_ACCOUNT}")"
CONTEXT_FRAME=$'domain=substrate.install_bootstrap_context\nversion=1\n'
CONTEXT_FRAME+="selected_host_prefix=${ENCODED_PREFIX}"$'\n'
CONTEXT_FRAME+="host_substrate_home=${ENCODED_PREFIX}"$'\n'
CONTEXT_FRAME+="host_substrate_root=${ENCODED_PREFIX}"$'\n'
CONTEXT_FRAME+=$'principal_kind=unix\n'
CONTEXT_FRAME+="principal_account=${ENCODED_ACCOUNT}"$'\n'
CONTEXT_FRAME+="principal_uid=${CURRENT_UID}"$'\n'
EXPECTED_COMMITMENT="$(printf '%s' "${CONTEXT_FRAME}" | shasum -a 256 | awk '{ print $1 }')"
EXPECTED_CARRIER="$(base64url "${CONTEXT_FRAME}host_context_commitment=${EXPECTED_COMMITMENT}"$'\n')"

printf 'caller-owned-fd-sentinel\n' >"${CALLER_FD_SENTINEL}"
exec 9<"${CALLER_FD_SENTINEL}"

set +e
env -i \
  PATH="${FIXTURE_BIN_DIR}:${PATH}" \
  HOME="${FIXTURE_PREFIX}/caller-home" \
  TMPDIR="${FIXTURE_TMPDIR}" \
  CARGO_TARGET_DIR="${FIXTURE_REPO}/target" \
  FAKE_SUBSTRATE_PATH="${FAKE_SUBSTRATE_PATH}" \
  FAKE_SUBSTRATE_ARGV="${FAKE_SUBSTRATE_ARGV}" \
  FAKE_SUBSTRATE_ENV="${FAKE_SUBSTRATE_ENV}" \
  FAKE_SUBSTRATE_FD="${FAKE_SUBSTRATE_FD}" \
  FAKE_SUBSTRATE_CALLER_FD_SENTINEL="${CALLER_FD_SENTINEL}" \
  "${DEFAULT_BASH}" "${FIXTURE_INSTALLER}" \
    --prefix "${FIXTURE_PREFIX}" \
    --profile debug \
    --version-label dev \
    --no-world \
    --no-shims \
    --install-bootstrap-context-v1 "${EXPECTED_CARRIER}" \
    >"${WORK_ROOT}/installer.out" 2>"${WORK_ROOT}/installer.err"
INSTALLER_RC=$?
set -e
[[ "${INSTALLER_RC}" -eq 0 ]] || {
  sed -n '1,160p' "${WORK_ROOT}/installer.err" >&2
  fail "fixture installer failed with exit ${INSTALLER_RC}"
}

[[ -f "${FAKE_SUBSTRATE_FD}" ]] || fail "fixture installer did not invoke fake substrate"
[[ "$(<"${FAKE_SUBSTRATE_FD}")" == "sentinel" ]] \
  || fail "caller-owned descriptor 9 was replaced or unavailable to fake substrate"
IFS= read -r CALLER_FD_VALUE <&9 || fail "caller-owned descriptor 9 was consumed"
[[ "${CALLER_FD_VALUE}" == "caller-owned-fd-sentinel" ]] \
  || fail "caller-owned descriptor 9 did not retain its unread sentinel"

{
  printf '%s\n' \
    '--install-bootstrap-context-v1' \
    "${EXPECTED_CARRIER}" \
    '--install-bootstrap-home-v1'
} >"${WORK_ROOT}/expected.argv"
cmp -s "${WORK_ROOT}/expected.argv" "${FAKE_SUBSTRATE_ARGV}" \
  || fail "fake substrate did not receive canonical bootstrap argv"

{
  printf 'SUBSTRATE_HOME=%s\n' "${FIXTURE_PREFIX}"
  printf 'SUBSTRATE_ROOT=%s\n' "${FIXTURE_PREFIX}"
  printf 'SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=%s\n' "${EXPECTED_COMMITMENT}"
  printf 'SUBSTRATE_INSTALL_PRIMARY_USER=%s\n' "${CURRENT_ACCOUNT}"
  printf 'SUBSTRATE_INSTALL_PRIMARY_UID=%s\n' "${CURRENT_UID}"
  printf 'SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=%s\n' "${EXPECTED_CARRIER}"
} >"${WORK_ROOT}/expected.env"
cmp -s "${WORK_ROOT}/expected.env" "${FAKE_SUBSTRATE_ENV}" \
  || fail "fake substrate did not receive canonical bootstrap environment"

if find "${FIXTURE_TMPDIR}" -mindepth 1 -print -quit | grep -q .; then
  find "${FIXTURE_TMPDIR}" -mindepth 1 -maxdepth 1 -print >&2
  fail "bootstrap left a temporary file under the fixture prefix"
fi

printf '[%s] PASS: Bash 3.2 preserves caller FD, canonical bootstrap argv/env, and fixture temp state\n' \
  "${SCRIPT_NAME}"
