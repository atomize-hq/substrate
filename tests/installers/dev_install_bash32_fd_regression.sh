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
WORK_ROOT="$(cd "${WORK_ROOT}" && pwd -P)"
FIXTURE_REPO="${WORK_ROOT}/repo"
FIXTURE_PREFIX="${WORK_ROOT}/prefix"
FIXTURE_TMPDIR="${WORK_ROOT}/bootstrap-tmp"
FIXTURE_BIN_DIR="${WORK_ROOT}/fake-bin"
FIXTURE_INSTALLER="${FIXTURE_REPO}/scripts/substrate/dev-install-substrate.sh"
FAKE_SUBSTRATE_PATH="${FIXTURE_REPO}/target/debug/substrate"
FAKE_SUBSTRATE_ARGV="${WORK_ROOT}/fake-substrate.argv"
FAKE_SUBSTRATE_ENV="${WORK_ROOT}/fake-substrate.env"
FAKE_SUBSTRATE_FD="${WORK_ROOT}/fake-substrate.fd"
FAKE_CARGO_ARGV="${WORK_ROOT}/fake-cargo.argv"
FAKE_AARCH64_CARGO_ARGV="${WORK_ROOT}/fake-aarch64-cargo.argv"
FAKE_AARCH64_CARGO_TARGET_DIR="${WORK_ROOT}/fake-aarch64-cargo-target-dir"
FAKE_PRIVILEGED_LOG="${WORK_ROOT}/fake-privileged.log"
CALLER_FD_SENTINEL="${WORK_ROOT}/caller-owned-fd-sentinel"

cleanup() {
  exec 9<&- 2>/dev/null || true
  rm -rf "${WORK_ROOT}"
}
trap cleanup EXIT

mkdir -p \
  "${FIXTURE_REPO}/scripts/substrate" \
  "${FIXTURE_REPO}/scripts/mac" \
  "${FIXTURE_REPO}/llm-last-mile/runtime-refactor/review-control" \
  "${FIXTURE_REPO}/config" \
  "${FIXTURE_PREFIX}" \
  "${FIXTURE_TMPDIR}" \
  "${FIXTURE_BIN_DIR}"
cp "${SOURCE_INSTALLER}" "${FIXTURE_INSTALLER}"
cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" \
  "${FIXTURE_REPO}/scripts/substrate/world-deps.yaml"
cp "${REPO_ROOT}/scripts/mac/com.substrate.lifecycle.publisher.v1.plist" \
  "${FIXTURE_REPO}/scripts/mac/com.substrate.lifecycle.publisher.v1.plist"
cp -R "${REPO_ROOT}/config/." "${FIXTURE_REPO}/config/"
cp "${REPO_ROOT}/Cargo.lock" "${FIXTURE_REPO}/Cargo.lock"
cp "${REPO_ROOT}/llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-cycle-record.json" \
  "${FIXTURE_REPO}/llm-last-mile/runtime-refactor/review-control/r3-mac-evidence-recovery-r5-review-cycle-record.json"
chmod 0755 "${FIXTURE_INSTALLER}"

cat >"${FIXTURE_BIN_DIR}/cargo" <<'FAKE_CARGO'
#!/bin/bash
set -euo pipefail

: "${FAKE_SUBSTRATE_PATH:?}"
: "${FAKE_SUBSTRATE_ARGV:?}"
: "${FAKE_SUBSTRATE_ENV:?}"
: "${FAKE_SUBSTRATE_FD:?}"
: "${FAKE_CARGO_ARGV:?}"
: "${FAKE_AARCH64_CARGO_ARGV:?}"
: "${FAKE_AARCH64_CARGO_TARGET_DIR:?}"

previous=""
is_aarch64=0
for argument in "$@"; do
  if [[ "${previous}" == "--target" && "${argument}" == "aarch64-unknown-linux-gnu" ]]; then
    is_aarch64=1
  fi
  previous="${argument}"
done

if [[ "${is_aarch64}" -eq 1 ]]; then
  {
    for argument in "$@"; do
      printf '%s\n' "${argument}"
    done
  } >"${FAKE_AARCH64_CARGO_ARGV}"
else
  {
    for argument in "$@"; do
      printf '%s\n' "${argument}"
    done
  } >"${FAKE_CARGO_ARGV}"
fi

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
cp "${FAKE_SUBSTRATE_PATH}" "${FAKE_SUBSTRATE_PATH%/substrate}/substrate-lifecycle-control"
cp "${FAKE_SUBSTRATE_PATH}" "${FAKE_SUBSTRATE_PATH%/substrate}/substrate-lifecycle-macos"
chmod 0755 \
  "${FAKE_SUBSTRATE_PATH%/substrate}/substrate-lifecycle-control" \
  "${FAKE_SUBSTRATE_PATH%/substrate}/substrate-lifecycle-macos"

if [[ "${is_aarch64}" -eq 1 ]]; then
  : "${CARGO_TARGET_DIR:?}"
  printf '%s\n' "${CARGO_TARGET_DIR}" >"${FAKE_AARCH64_CARGO_TARGET_DIR}"
  fake_cross_root="${CARGO_TARGET_DIR}/aarch64-unknown-linux-gnu/release"
  mkdir -p "${fake_cross_root}"
  for fake_cross_binary in substrate-lifecycle-linux world-service substrate-gateway substrate; do
    printf '#!/bin/bash\nexit 0\n' >"${fake_cross_root}/${fake_cross_binary}"
    chmod 0755 "${fake_cross_root}/${fake_cross_binary}"
  done
fi
FAKE_CARGO
chmod 0755 "${FIXTURE_BIN_DIR}/cargo"

cat >"${FIXTURE_BIN_DIR}/file" <<'FAKE_FILE'
#!/bin/bash
set -euo pipefail

last=""
for argument in "$@"; do
  last="${argument}"
done
case "${last}" in
  */aarch64-unknown-linux-gnu/release/substrate-lifecycle-linux|\
  */aarch64-unknown-linux-gnu/release/world-service|\
  */aarch64-unknown-linux-gnu/release/substrate-gateway|\
  */aarch64-unknown-linux-gnu/release/substrate)
    printf '%s\n' 'ELF 64-bit LSB executable, ARM aarch64'
    ;;
  *) exec /usr/bin/file "$@" ;;
esac
FAKE_FILE
chmod 0755 "${FIXTURE_BIN_DIR}/file"

cat >"${FIXTURE_BIN_DIR}/rustc" <<'FAKE_RUSTC'
#!/bin/bash
set -euo pipefail

if [[ "${1-}" == "--version" ]]; then
  printf '%s\n' 'rustc 1.89.0 (fixture)'
  exit 0
fi
exit 64
FAKE_RUSTC
chmod 0755 "${FIXTURE_BIN_DIR}/rustc"

# The fixture proves the installer requests the fixed privileged publication without invoking
# sudo or writing any host /Library state. The production dispatcher keeps its scrubbed absolute
# tool paths; only this PATH-local fake observes and accepts the test invocation.
cat >"${FIXTURE_BIN_DIR}/sudo" <<'FAKE_SUDO'
#!/bin/bash
set -euo pipefail

: "${FAKE_PRIVILEGED_LOG:?}"
printf '%s\n' "$*" >>"${FAKE_PRIVILEGED_LOG}"
exit 0
FAKE_SUDO
chmod 0755 "${FIXTURE_BIN_DIR}/sudo"

cat >"${FIXTURE_BIN_DIR}/limactl" <<'FAKE_LIMACTL'
#!/bin/bash
set -euo pipefail

if [[ "${1-}" == "--version" ]]; then
  printf '%s\n' 'limactl version fixture'
  exit 0
fi
exit 64
FAKE_LIMACTL
chmod 0755 "${FIXTURE_BIN_DIR}/limactl"

cat >"${FIXTURE_BIN_DIR}/git" <<'FAKE_GIT'
#!/bin/bash
set -euo pipefail

case "${1-}:${2-}" in
  rev-parse:HEAD) printf '%040d\n' 1 ;;
  'rev-parse:HEAD^{tree}') printf '%040d\n' 2 ;;
  symbolic-ref:-q) printf '%s\n' 'refs/heads/r5-fixture' ;;
  *) exit 64 ;;
esac
FAKE_GIT
chmod 0755 "${FIXTURE_BIN_DIR}/git"

cat >"${FIXTURE_BIN_DIR}/cp" <<'FAKE_CP'
#!/bin/bash
set -euo pipefail

if [[ -n "${FAKE_FAIL_MAC_COPY_BINARY-}" && -n "${FAKE_FAILURE_MARKER-}" && \
  ! -e "${FAKE_FAILURE_MARKER}" && "${1-}" == */target/debug/"${FAKE_FAIL_MAC_COPY_BINARY}" && \
  "${2-}" == */bin/"${FAKE_FAIL_MAC_COPY_BINARY}".tmp.* ]]; then
  : >"${FAKE_FAILURE_MARKER}"
  exit 81
fi

exec /bin/cp "$@"
FAKE_CP
chmod 0755 "${FIXTURE_BIN_DIR}/cp"

cat >"${FIXTURE_BIN_DIR}/mv" <<'FAKE_MV'
#!/bin/bash
set -euo pipefail

if [[ "${FAKE_FAIL_MAC_MANIFEST_ONCE-}" == "1" && -n "${FAKE_FAILURE_MARKER-}" && \
  ! -e "${FAKE_FAILURE_MARKER}" && -n "${FAKE_MAC_MANIFEST_PATH-}" && \
  "${1-}" == "${FAKE_MAC_MANIFEST_PATH}".tmp.* && "${2-}" == "${FAKE_MAC_MANIFEST_PATH}" ]]; then
  : >"${FAKE_FAILURE_MARKER}"
  exit 82
fi

exec /bin/mv "$@"
FAKE_MV
chmod 0755 "${FIXTURE_BIN_DIR}/mv"

cat >"${FIXTURE_BIN_DIR}/grep" <<'FAKE_GREP'
#!/bin/bash
set -euo pipefail

if [[ "${FAKE_FAIL_MAC_MANIFEST_READ_ONCE-}" == "1" && -n "${FAKE_FAILURE_MARKER-}" && \
  ! -e "${FAKE_FAILURE_MARKER}" && -n "${FAKE_MAC_MANIFEST_PATH-}" && \
  "${1-}" == "-Fxv" && "${2-}" == "--" && "${4-}" == "${FAKE_MAC_MANIFEST_PATH}" && \
  -f "${FAKE_MAC_MANIFEST_PATH}" ]]; then
  : >"${FAKE_FAILURE_MARKER}"
  printf 'injected macOS managed-manifest read failure\n' >&2
  exit 83
fi

exec /usr/bin/grep "$@"
FAKE_GREP
chmod 0755 "${FIXTURE_BIN_DIR}/grep"

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

bootstrap_carrier_for_prefix() {
  local selected_prefix="$1"
  local encoded_prefix
  local frame
  local commitment

  encoded_prefix="$(base64url "${selected_prefix}")"
  frame=$'domain=substrate.install_bootstrap_context\nversion=1\n'
  frame+="selected_host_prefix=${encoded_prefix}"$'\n'
  frame+="host_substrate_home=${encoded_prefix}"$'\n'
  frame+="host_substrate_root=${encoded_prefix}"$'\n'
  frame+=$'principal_kind=unix\n'
  frame+="principal_account=${ENCODED_ACCOUNT}"$'\n'
  frame+="principal_uid=${CURRENT_UID}"$'\n'
  commitment="$(printf '%s' "${frame}" | shasum -a 256 | awk '{ print $1 }')"
  base64url "${frame}host_context_commitment=${commitment}"$'\n'
}

run_fixture_installer() {
  local install_prefix="$1"
  local carrier="$2"
  local run_label="$3"
  local fail_copy_binary="$4"
  local fail_manifest_once="$5"
  local failure_marker="$6"
  local fail_manifest_read_once="${7-}"
  local fixture_tmpdir="${8:-${FIXTURE_TMPDIR}}"
  local manifest_path="${install_prefix}/.dev-install-managed/mac-control-binaries.txt"

  env -i \
    PATH="${FIXTURE_BIN_DIR}:${PATH}" \
    HOME="${install_prefix}/caller-home" \
    TMPDIR="${fixture_tmpdir}" \
    CARGO_TARGET_DIR="${FIXTURE_REPO}/target" \
    FAKE_SUBSTRATE_PATH="${FAKE_SUBSTRATE_PATH}" \
    FAKE_SUBSTRATE_ARGV="${FAKE_SUBSTRATE_ARGV}" \
    FAKE_SUBSTRATE_ENV="${FAKE_SUBSTRATE_ENV}" \
    FAKE_SUBSTRATE_FD="${FAKE_SUBSTRATE_FD}" \
    FAKE_CARGO_ARGV="${FAKE_CARGO_ARGV}" \
    FAKE_AARCH64_CARGO_ARGV="${FAKE_AARCH64_CARGO_ARGV}" \
    FAKE_AARCH64_CARGO_TARGET_DIR="${FAKE_AARCH64_CARGO_TARGET_DIR}" \
    FAKE_PRIVILEGED_LOG="${FAKE_PRIVILEGED_LOG}" \
    FAKE_SUBSTRATE_CALLER_FD_SENTINEL="${CALLER_FD_SENTINEL}" \
    FAKE_FAIL_MAC_COPY_BINARY="${fail_copy_binary}" \
    FAKE_FAIL_MAC_MANIFEST_ONCE="${fail_manifest_once}" \
    FAKE_FAIL_MAC_MANIFEST_READ_ONCE="${fail_manifest_read_once}" \
    FAKE_FAILURE_MARKER="${failure_marker}" \
    FAKE_MAC_MANIFEST_PATH="${manifest_path}" \
    "${DEFAULT_BASH}" "${FIXTURE_INSTALLER}" \
      --prefix "${install_prefix}" \
      --profile debug \
      --version-label dev \
      --no-world \
      --no-shims \
      --install-bootstrap-context-v1 "${carrier}" \
      >"${WORK_ROOT}/${run_label}.out" 2>"${WORK_ROOT}/${run_label}.err"
}

assert_no_mac_copy_temps() {
  local install_prefix="$1"

  if find "${install_prefix}" -name '*.tmp.*' -print -quit | grep -q .; then
    find "${install_prefix}" -name '*.tmp.*' -print >&2
    fail "macOS managed-copy failure left a temporary artifact"
  fi
}

assert_mac_pair_converged() {
  local install_prefix="$1"
  local label="$2"
  local binary
  local installed
  local built
  local expected_manifest="${WORK_ROOT}/expected-managed-${label}"

  for binary in substrate-lifecycle-control substrate-lifecycle-macos; do
    installed="${install_prefix}/bin/${binary}"
    built="${FIXTURE_REPO}/target/debug/${binary}"
    [[ -f "${installed}" && ! -L "${installed}" ]] \
      || fail "${label}: macOS ${binary} was not copied to a fixed prefix path"
    cmp -s "${built}" "${installed}" \
      || fail "${label}: macOS ${binary} fixed-prefix bytes differ from the built artifact"
  done
  cat >"${expected_manifest}" <<EOF
${install_prefix}/bin/substrate-lifecycle-control
${install_prefix}/bin/substrate-lifecycle-macos
EOF
  cmp -s "${expected_manifest}" "${install_prefix}/.dev-install-managed/mac-control-binaries.txt" \
    || fail "${label}: macOS lifecycle binaries were not recorded exactly as managed copies"
  assert_no_mac_copy_temps "${install_prefix}"
}

assert_mac_lima_bundle_converged() {
  local install_prefix="$1"
  local label="$2"
  local bundle="${install_prefix}/bin/linux"
  local binary
  local observed_count

  [[ -d "${bundle}" && ! -L "${bundle}" ]] \
    || fail "${label}: fixed AArch64 Linux bundle is absent or linked"
  for binary in substrate-lifecycle-linux world-service substrate-gateway substrate; do
    [[ -f "${bundle}/${binary}" && ! -L "${bundle}/${binary}" && -x "${bundle}/${binary}" ]] \
      || fail "${label}: retained ${binary} is not a regular executable"
    [[ "$(stat -f '%Lp' "${bundle}/${binary}")" == "755" ]] \
      || fail "${label}: retained ${binary} mode is not 0755"
  done
  observed_count="$(find "${bundle}" -mindepth 1 -maxdepth 1 -type f | wc -l | tr -d '[:space:]')"
  [[ "${observed_count}" == "4" ]] \
    || fail "${label}: retained Linux bundle does not contain exactly four files"
}

seed_exact_mac_lima_bundle_v1() {
  local install_prefix="$1"
  local bundle="${install_prefix}/bin/linux"
  local binary

  mkdir -p "${bundle}"
  for binary in substrate-lifecycle-linux world-service substrate-gateway substrate; do
    printf '#!/bin/bash\nexit 0\n' >"${bundle}/${binary}"
    chmod 0755 "${bundle}/${binary}"
  done
}

bundle_content_manifest_v1() {
  local bundle="$1"
  local entry

  find "${bundle}" -mindepth 1 -maxdepth 1 -type f -print | LC_ALL=C sort | while IFS= read -r entry; do
    printf '%s %s %s\n' "$(basename "${entry}")" "$(stat -f '%Lp' "${entry}")" \
      "$(shasum -a 256 -- "${entry}" | awk '{print $1}')"
  done
}

printf 'caller-owned-fd-sentinel\n' >"${CALLER_FD_SENTINEL}"
exec 9<"${CALLER_FD_SENTINEL}"

if ! run_fixture_installer "${FIXTURE_PREFIX}" "${EXPECTED_CARRIER}" \
  installer "" "" ""; then
  sed -n '1,160p' "${WORK_ROOT}/installer.err" >&2
  fail "fixture installer failed"
fi

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

cat >"${WORK_ROOT}/expected-cargo.argv" <<'EXPECTED_CARGO'
build
-p
substrate
--bin
substrate
--bin
substrate-shim
-p
substrate-gateway
--bin
substrate-gateway
--bin
substrate-lifecycle-control
--bin
substrate-lifecycle-macos
EXPECTED_CARGO
cmp -s "${WORK_ROOT}/expected-cargo.argv" "${FAKE_CARGO_ARGV}" \
  || fail "installer did not build exactly the macOS lifecycle binaries"

cat >"${WORK_ROOT}/expected-aarch64-cargo.argv" <<'EXPECTED_AARCH64_CARGO'
build
--locked
--offline
--target
aarch64-unknown-linux-gnu
--release
-p
substrate
--bin
substrate-lifecycle-linux
-p
world-service
--bin
world-service
-p
substrate-gateway
--bin
substrate-gateway
-p
substrate
--bin
substrate
EXPECTED_AARCH64_CARGO
cmp -s "${WORK_ROOT}/expected-aarch64-cargo.argv" "${FAKE_AARCH64_CARGO_ARGV}" \
  || fail "installer did not make the fixed four-binary AArch64 build"

assert_mac_pair_converged "${FIXTURE_PREFIX}" primary
assert_mac_lima_bundle_converged "${FIXTURE_PREFIX}" primary
grep -Fq -- 'mac-publisher-install-provenance' "${FAKE_PRIVILEGED_LOG}" \
  || fail "fixture installer did not request fixed root install-provenance publication"
# This fixture intentionally does not invoke sudo.  Pin the fixed source-to-root-copy
# verification contract so a shell-only regression cannot drop the pre-elevation digest join.
for required in \
  'executor_expected_sha="${14}"' \
  'test "$(sha "$executor_path")" = "$executor_expected_sha"' \
  'privileged executor copy does not match pre-elevation digest' \
  '"${control_src_sha}" "${control_src_identity}" "${executor_src_sha}" "${executor_src_identity}"'; do
  grep -Fq -- "${required}" "${SOURCE_INSTALLER}" \
    || fail "installer lost fixed provenance source/copy integrity guard: ${required}"
done

COPY_FAILURE_PREFIX="${WORK_ROOT}/copy-failure-prefix"
COPY_FAILURE_MARKER="${WORK_ROOT}/copy-failure.marker"
COPY_FAILURE_CARRIER="$(bootstrap_carrier_for_prefix "${COPY_FAILURE_PREFIX}")"
if run_fixture_installer "${COPY_FAILURE_PREFIX}" "${COPY_FAILURE_CARRIER}" \
  copy-failure-control substrate-lifecycle-control "" "${COPY_FAILURE_MARKER}"; then
  fail "fixture copy failure unexpectedly succeeded"
fi
[[ -f "${COPY_FAILURE_MARKER}" ]] || fail "fixture did not inject lifecycle-control copy failure"
[[ ! -e "${COPY_FAILURE_PREFIX}/bin/substrate-lifecycle-control" ]] \
  || fail "copy failure stranded a lifecycle-control destination"
[[ ! -f "${COPY_FAILURE_PREFIX}/.dev-install-managed/mac-control-binaries.txt" ]] \
  || fail "copy failure published a lifecycle manifest entry before the binary"
assert_no_mac_copy_temps "${COPY_FAILURE_PREFIX}"
if ! run_fixture_installer "${COPY_FAILURE_PREFIX}" "${COPY_FAILURE_CARRIER}" \
  copy-failure-retry "" "" ""; then
  sed -n '1,160p' "${WORK_ROOT}/copy-failure-retry.err" >&2
  fail "copy failure retry did not converge"
fi
assert_mac_pair_converged "${COPY_FAILURE_PREFIX}" copy-failure-retry
assert_mac_lima_bundle_converged "${COPY_FAILURE_PREFIX}" copy-failure-retry

MANIFEST_FAILURE_PREFIX="${WORK_ROOT}/manifest-failure-prefix"
MANIFEST_FAILURE_MARKER="${WORK_ROOT}/manifest-failure.marker"
MANIFEST_FAILURE_CARRIER="$(bootstrap_carrier_for_prefix "${MANIFEST_FAILURE_PREFIX}")"
if run_fixture_installer "${MANIFEST_FAILURE_PREFIX}" "${MANIFEST_FAILURE_CARRIER}" \
  manifest-failure-control "" 1 "${MANIFEST_FAILURE_MARKER}"; then
  fail "fixture manifest failure unexpectedly succeeded"
fi
[[ -f "${MANIFEST_FAILURE_MARKER}" ]] || fail "fixture did not inject lifecycle manifest failure"
[[ ! -e "${MANIFEST_FAILURE_PREFIX}/bin/substrate-lifecycle-control" ]] \
  || fail "manifest failure stranded an unmanifested lifecycle-control destination"
[[ ! -f "${MANIFEST_FAILURE_PREFIX}/.dev-install-managed/mac-control-binaries.txt" ]] \
  || fail "manifest failure published a lifecycle manifest entry"
assert_no_mac_copy_temps "${MANIFEST_FAILURE_PREFIX}"
if ! run_fixture_installer "${MANIFEST_FAILURE_PREFIX}" "${MANIFEST_FAILURE_CARRIER}" \
  manifest-failure-retry "" "" ""; then
  sed -n '1,160p' "${WORK_ROOT}/manifest-failure-retry.err" >&2
  fail "manifest failure retry did not converge"
fi
assert_mac_pair_converged "${MANIFEST_FAILURE_PREFIX}" manifest-failure-retry
assert_mac_lima_bundle_converged "${MANIFEST_FAILURE_PREFIX}" manifest-failure-retry

PAIR_FAILURE_PREFIX="${WORK_ROOT}/pair-failure-prefix"
PAIR_FAILURE_MARKER="${WORK_ROOT}/pair-failure.marker"
PAIR_FAILURE_CARRIER="$(bootstrap_carrier_for_prefix "${PAIR_FAILURE_PREFIX}")"
if run_fixture_installer "${PAIR_FAILURE_PREFIX}" "${PAIR_FAILURE_CARRIER}" \
  pair-failure-macos substrate-lifecycle-macos "" "${PAIR_FAILURE_MARKER}"; then
  fail "fixture second-pair-member copy failure unexpectedly succeeded"
fi
[[ -f "${PAIR_FAILURE_MARKER}" ]] || fail "fixture did not inject lifecycle-macos copy failure"
[[ -f "${PAIR_FAILURE_PREFIX}/bin/substrate-lifecycle-control" ]] \
  || fail "first pair member did not remain safely managed after second member failure"
[[ ! -e "${PAIR_FAILURE_PREFIX}/bin/substrate-lifecycle-macos" ]] \
  || fail "second-pair-member copy failure stranded a lifecycle-macos destination"
grep -Fxq -- "${PAIR_FAILURE_PREFIX}/bin/substrate-lifecycle-control" \
  "${PAIR_FAILURE_PREFIX}/.dev-install-managed/mac-control-binaries.txt" \
  || fail "first pair member lost its manifest entry after second member failure"
assert_no_mac_copy_temps "${PAIR_FAILURE_PREFIX}"
if ! run_fixture_installer "${PAIR_FAILURE_PREFIX}" "${PAIR_FAILURE_CARRIER}" \
  pair-failure-retry "" "" ""; then
  sed -n '1,160p' "${WORK_ROOT}/pair-failure-retry.err" >&2
  fail "second-pair-member failure retry did not converge"
fi
assert_mac_pair_converged "${PAIR_FAILURE_PREFIX}" pair-failure-retry
assert_mac_lima_bundle_converged "${PAIR_FAILURE_PREFIX}" pair-failure-retry

MANIFEST_READ_FAILURE_PREFIX="${WORK_ROOT}/manifest-read-failure-prefix"
MANIFEST_READ_FAILURE_MARKER="${WORK_ROOT}/manifest-read-failure.marker"
MANIFEST_READ_FAILURE_CARRIER="$(bootstrap_carrier_for_prefix "${MANIFEST_READ_FAILURE_PREFIX}")"
if ! run_fixture_installer "${MANIFEST_READ_FAILURE_PREFIX}" "${MANIFEST_READ_FAILURE_CARRIER}" \
  manifest-read-failure-initial "" "" ""; then
  sed -n '1,160p' "${WORK_ROOT}/manifest-read-failure-initial.err" >&2
  fail "manifest read failure fixture initial install did not converge"
fi
assert_mac_pair_converged "${MANIFEST_READ_FAILURE_PREFIX}" manifest-read-failure-initial
assert_mac_lima_bundle_converged "${MANIFEST_READ_FAILURE_PREFIX}" manifest-read-failure-initial
if run_fixture_installer "${MANIFEST_READ_FAILURE_PREFIX}" "${MANIFEST_READ_FAILURE_CARRIER}" \
  manifest-read-failure-control "" "" "${MANIFEST_READ_FAILURE_MARKER}" 1; then
  fail "fixture managed-manifest read failure unexpectedly succeeded"
fi
[[ -f "${MANIFEST_READ_FAILURE_MARKER}" ]] \
  || fail "fixture did not inject managed-manifest read failure"
assert_mac_pair_converged "${MANIFEST_READ_FAILURE_PREFIX}" manifest-read-failure-preserved
assert_mac_lima_bundle_converged "${MANIFEST_READ_FAILURE_PREFIX}" manifest-read-failure-preserved
if ! run_fixture_installer "${MANIFEST_READ_FAILURE_PREFIX}" "${MANIFEST_READ_FAILURE_CARRIER}" \
  manifest-read-failure-retry "" "" ""; then
  sed -n '1,160p' "${WORK_ROOT}/manifest-read-failure-retry.err" >&2
  fail "managed-manifest read failure retry did not converge"
fi
assert_mac_pair_converged "${MANIFEST_READ_FAILURE_PREFIX}" manifest-read-failure-retry
assert_mac_lima_bundle_converged "${MANIFEST_READ_FAILURE_PREFIX}" manifest-read-failure-retry

# The fixed build root cannot be redirected into a checkout through caller TMPDIR.  The fake cargo
# records the actual CARGO_TARGET_DIR so this is a direct boundary check, not a lexical source test.
REPO_TMPDIR="${FIXTURE_REPO}/caller-controlled-tmp"
REPO_TMPDIR_PREFIX="${WORK_ROOT}/repo-tmpdir-prefix"
REPO_TMPDIR_CARRIER="$(bootstrap_carrier_for_prefix "${REPO_TMPDIR_PREFIX}")"
mkdir -p "${REPO_TMPDIR}"
if ! run_fixture_installer "${REPO_TMPDIR_PREFIX}" "${REPO_TMPDIR_CARRIER}" \
  repo-tmpdir-boundary "" "" "" "" "${REPO_TMPDIR}"; then
  sed -n '1,160p' "${WORK_ROOT}/repo-tmpdir-boundary.err" >&2
  fail "repo TMPDIR boundary fixture did not converge through the fixed external build root"
fi
[[ -f "${FAKE_AARCH64_CARGO_TARGET_DIR}" ]] \
  || fail "repo TMPDIR boundary fixture did not record the AArch64 build root"
if [[ "$(<"${FAKE_AARCH64_CARGO_TARGET_DIR}")" == "${FIXTURE_REPO}"/* ]]; then
  fail "caller-controlled repo TMPDIR redirected AArch64 build output into the checkout"
fi
if find "${REPO_TMPDIR}" -mindepth 1 -print -quit | grep -q .; then
  find "${REPO_TMPDIR}" -mindepth 1 -maxdepth 1 -print >&2
  fail "caller-controlled repo TMPDIR retained an AArch64 build artifact"
fi
assert_mac_lima_bundle_converged "${REPO_TMPDIR_PREFIX}" repo-tmpdir-boundary

# A partial, unknown, or mismatched retained bundle is not an authority source. Each attempt
# must fail before changing the caller-provided bundle bytes; a subsequent exact retry is allowed
# only when the complete four-file bundle was already published atomically.
PARTIAL_BUNDLE_PREFIX="${WORK_ROOT}/partial-bundle-prefix"
PARTIAL_BUNDLE_CARRIER="$(bootstrap_carrier_for_prefix "${PARTIAL_BUNDLE_PREFIX}")"
mkdir -p "${PARTIAL_BUNDLE_PREFIX}/bin/linux"
printf 'partial-retained-artifact\n' >"${PARTIAL_BUNDLE_PREFIX}/bin/linux/substrate"
chmod 0755 "${PARTIAL_BUNDLE_PREFIX}/bin/linux/substrate"
bundle_content_manifest_v1 "${PARTIAL_BUNDLE_PREFIX}/bin/linux" >"${WORK_ROOT}/partial-bundle.before"
if run_fixture_installer "${PARTIAL_BUNDLE_PREFIX}" "${PARTIAL_BUNDLE_CARRIER}" \
  partial-bundle-rejected "" "" ""; then
  fail "partial retained Linux bundle was incorrectly accepted"
fi
bundle_content_manifest_v1 "${PARTIAL_BUNDLE_PREFIX}/bin/linux" >"${WORK_ROOT}/partial-bundle.after"
cmp -s "${WORK_ROOT}/partial-bundle.before" "${WORK_ROOT}/partial-bundle.after" \
  || fail "partial retained Linux bundle changed after fail-closed rejection"

UNKNOWN_BUNDLE_PREFIX="${WORK_ROOT}/unknown-bundle-prefix"
UNKNOWN_BUNDLE_CARRIER="$(bootstrap_carrier_for_prefix "${UNKNOWN_BUNDLE_PREFIX}")"
seed_exact_mac_lima_bundle_v1 "${UNKNOWN_BUNDLE_PREFIX}"
printf 'unknown-retained-artifact\n' >"${UNKNOWN_BUNDLE_PREFIX}/bin/linux/unexpected"
chmod 0755 "${UNKNOWN_BUNDLE_PREFIX}/bin/linux/unexpected"
bundle_content_manifest_v1 "${UNKNOWN_BUNDLE_PREFIX}/bin/linux" >"${WORK_ROOT}/unknown-bundle.before"
if run_fixture_installer "${UNKNOWN_BUNDLE_PREFIX}" "${UNKNOWN_BUNDLE_CARRIER}" \
  unknown-bundle-rejected "" "" ""; then
  fail "unknown retained Linux bundle entry was incorrectly accepted"
fi
bundle_content_manifest_v1 "${UNKNOWN_BUNDLE_PREFIX}/bin/linux" >"${WORK_ROOT}/unknown-bundle.after"
cmp -s "${WORK_ROOT}/unknown-bundle.before" "${WORK_ROOT}/unknown-bundle.after" \
  || fail "unknown retained Linux bundle changed after fail-closed rejection"

MISMATCH_BUNDLE_PREFIX="${WORK_ROOT}/mismatch-bundle-prefix"
MISMATCH_BUNDLE_CARRIER="$(bootstrap_carrier_for_prefix "${MISMATCH_BUNDLE_PREFIX}")"
seed_exact_mac_lima_bundle_v1 "${MISMATCH_BUNDLE_PREFIX}"
printf 'substituted-world-service\n' >"${MISMATCH_BUNDLE_PREFIX}/bin/linux/world-service"
chmod 0755 "${MISMATCH_BUNDLE_PREFIX}/bin/linux/world-service"
bundle_content_manifest_v1 "${MISMATCH_BUNDLE_PREFIX}/bin/linux" >"${WORK_ROOT}/mismatch-bundle.before"
if run_fixture_installer "${MISMATCH_BUNDLE_PREFIX}" "${MISMATCH_BUNDLE_CARRIER}" \
  mismatch-bundle-rejected "" "" ""; then
  fail "mismatched retained Linux bundle was incorrectly accepted"
fi
bundle_content_manifest_v1 "${MISMATCH_BUNDLE_PREFIX}/bin/linux" >"${WORK_ROOT}/mismatch-bundle.after"
cmp -s "${WORK_ROOT}/mismatch-bundle.before" "${WORK_ROOT}/mismatch-bundle.after" \
  || fail "mismatched retained Linux bundle changed after fail-closed rejection"

UNMANAGED_PREFIX="${WORK_ROOT}/unmanaged-prefix"
UNMANAGED_CARRIER="$(bootstrap_carrier_for_prefix "${UNMANAGED_PREFIX}")"
mkdir -p "${UNMANAGED_PREFIX}/bin"
printf 'unmanaged-lifecycle-control\n' >"${UNMANAGED_PREFIX}/bin/substrate-lifecycle-control"
if run_fixture_installer "${UNMANAGED_PREFIX}" "${UNMANAGED_CARRIER}" unmanaged-control "" "" ""; then
  fail "fixture unmanaged lifecycle-control overwrite unexpectedly succeeded"
fi
[[ "$(<"${UNMANAGED_PREFIX}/bin/substrate-lifecycle-control")" == "unmanaged-lifecycle-control" ]] \
  || fail "unmanaged lifecycle-control destination was overwritten"
[[ ! -f "${UNMANAGED_PREFIX}/.dev-install-managed/mac-control-binaries.txt" ]] \
  || fail "unmanaged lifecycle-control destination was claimed as managed"

if find "${FIXTURE_TMPDIR}" -mindepth 1 -print -quit | grep -q .; then
  find "${FIXTURE_TMPDIR}" -mindepth 1 -maxdepth 1 -print >&2
  fail "bootstrap left a temporary file under the fixture prefix"
fi

printf '[%s] PASS: Bash 3.2 preserves caller FD, canonical bootstrap argv/env, and fixture temp state\n' \
  "${SCRIPT_NAME}"
