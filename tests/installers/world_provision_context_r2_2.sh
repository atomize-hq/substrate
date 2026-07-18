#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="world-provision-context-r2-2"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORLD_ENABLE="${REPO_ROOT}/scripts/substrate/world-enable.sh"
CURRENT_UID="$(id -u)"
CURRENT_ACCOUNT="$(id -un)"
FIXTURE_PARENT="${XDG_RUNTIME_DIR:-/run/user/${CURRENT_UID}}"

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Linux" ]]; then
  printf '[%s] SKIP: Linux-only world-enable carrier proof\n' "${SCRIPT_NAME}"
  exit 0
fi
if [[ "${CURRENT_UID}" -eq 0 ]]; then
  fail "the world-enable carrier matrix must run as an unprivileged Unix principal"
fi
if [[ ! -d "${FIXTURE_PARENT}" || ! -O "${FIXTURE_PARENT}" ]]; then
  fail "secure audit-owned fixture parent is unavailable: ${FIXTURE_PARENT}"
fi

WORK_ROOT="$(mktemp -d "${FIXTURE_PARENT%/}/substrate-r2-2-world-context.XXXXXX")"
chmod 0700 "${WORK_ROOT}"
cleanup() {
  chmod -R u+rwX "${WORK_ROOT}" 2>/dev/null || true
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

SELECTED_A="${WORK_ROOT}/selected-a"
CONFLICT_C="${WORK_ROOT}/conflict-c"
AMBIENT_B="${WORK_ROOT}/ambient-b"
mkdir "${SELECTED_A}" "${CONFLICT_C}" "${AMBIENT_B}"
chmod 0700 "${SELECTED_A}" "${CONFLICT_C}" "${AMBIENT_B}"
printf 'ambient-b-sentinel\n' > "${AMBIENT_B}/sentinel"

make_context_values() {
  local prefix="$1"
  local account="$2"
  local uid="$3"
  python3 - "${prefix}" "${account}" "${uid}" <<'PY'
import base64
import hashlib
import sys


def encode(value):
    return base64.urlsafe_b64encode(value.encode("utf-8")).rstrip(b"=").decode("ascii")


prefix, account, uid = sys.argv[1:]
prefix_encoded = encode(prefix)
frame = (
    "domain=substrate.install_bootstrap_context\n"
    "version=1\n"
    f"selected_host_prefix={prefix_encoded}\n"
    f"host_substrate_home={prefix_encoded}\n"
    f"host_substrate_root={prefix_encoded}\n"
    "principal_kind=unix\n"
    f"principal_account={encode(account)}\n"
    f"principal_uid={uid}\n"
).encode("ascii")
commitment = hashlib.sha256(frame).hexdigest()
record = frame + f"host_context_commitment={commitment}\n".encode("ascii")
print(base64.urlsafe_b64encode(record).rstrip(b"=").decode("ascii"))
print(commitment)
PY
}

tamper_commitment() {
  python3 - "$1" <<'PY'
import base64
import sys


carrier = sys.argv[1].encode("ascii")
record = base64.urlsafe_b64decode(carrier + b"=" * ((-len(carrier)) % 4))
marker = b"host_context_commitment="
before, commitment = record.split(marker, 1)
first = b"0" if commitment[:1] != b"0" else b"1"
print(base64.urlsafe_b64encode(before + marker + first + commitment[1:]).rstrip(b"=").decode("ascii"))
PY
}

reorder_carrier() {
  python3 - "$1" <<'PY'
import base64
import sys


carrier = sys.argv[1].encode("ascii")
record = base64.urlsafe_b64decode(carrier + b"=" * ((-len(carrier)) % 4))
lines = record.rstrip(b"\n").split(b"\n")
lines[2], lines[3] = lines[3], lines[2]
print(base64.urlsafe_b64encode(b"\n".join(lines) + b"\n").rstrip(b"=").decode("ascii"))
PY
}

mapfile -t A_CONTEXT < <(make_context_values "${SELECTED_A}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}")
A_CARRIER="${A_CONTEXT[0]}"
A_COMMITMENT="${A_CONTEXT[1]}"
TAMPERED_CARRIER="$(tamper_commitment "${A_CARRIER}")"
REORDERED_CARRIER="$(reorder_carrier "${A_CARRIER}")"
mapfile -t FORGED_CONTEXT < <(make_context_values "${SELECTED_A}" "forged-principal" "${CURRENT_UID}")
FORGED_CARRIER="${FORGED_CONTEXT[0]}"
FORGED_COMMITMENT="${FORGED_CONTEXT[1]}"

STDOUT_PATH="${WORK_ROOT}/stdout"
STDERR_PATH="${WORK_ROOT}/stderr"
WORLD_ENABLE_RC=0

run_world_enable() {
  local projected_prefix="$1"
  local projected_carrier="$2"
  local projected_commitment="$3"
  local projected_account="$4"
  local projected_uid="$5"
  shift 5

  chmod 0000 "${AMBIENT_B}"
  set +e
  HOME="${AMBIENT_B}" \
    USERPROFILE="${AMBIENT_B}" \
    SUBSTRATE_HOME="${projected_prefix}" \
    SUBSTRATE_ROOT="${projected_prefix}" \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${projected_commitment}" \
    SUBSTRATE_INSTALL_PRIMARY_USER="${projected_account}" \
    SUBSTRATE_INSTALL_PRIMARY_UID="${projected_uid}" \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${projected_carrier}" \
    "${WORLD_ENABLE}" "$@" >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
  WORLD_ENABLE_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

run_world_enable_with_inherited_xtrace() {
  chmod 0000 "${AMBIENT_B}"
  set +e
  HOME="${AMBIENT_B}" \
    USERPROFILE="${AMBIENT_B}" \
    SUBSTRATE_HOME="${SELECTED_A}" \
    SUBSTRATE_ROOT="${SELECTED_A}" \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
    SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
    SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${A_CARRIER}" \
    bash -x "${WORLD_ENABLE}" \
      --home "${SELECTED_A}" \
      --install-bootstrap-context-v1 "${A_CARRIER}" \
      --dry-run >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
  WORLD_ENABLE_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

assert_failure() {
  local label="$1"
  [[ "${WORLD_ENABLE_RC}" -ne 0 ]] || fail "${label} unexpectedly succeeded"
  [[ ! -s "${STDOUT_PATH}" ]] || fail "${label} emitted normal output before validation"
}

assert_no_carrier_disclosure() {
  local label="$1"
  if grep -Fq -- "${A_CARRIER}" "${STDOUT_PATH}" "${STDERR_PATH}"; then
    fail "${label} disclosed carrier bytes"
  fi
}

assert_fixture_unchanged() {
  local entries
  entries="$(find "${SELECTED_A}" "${CONFLICT_C}" -mindepth 1 -print -quit)"
  [[ -z "${entries}" ]] || fail "world-enable carrier validation mutated a selected prefix"
  [[ "$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -printf '%f\n')" == "sentinel" ]] \
    || fail "ambient B was mutated"
  [[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
    || fail "ambient B sentinel changed"
}

run_world_enable \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" \
  --dry-run --verbose
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "exact internal carrier/home projection failed"
assert_no_carrier_disclosure "exact internal carrier"
grep -Eq '^\+ (sanitize_env_path|detect_platform)$' "${STDERR_PATH}" \
  || fail "verbose mode suppressed nonsensitive command tracing"

run_world_enable_with_inherited_xtrace
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "exact internal carrier failed under inherited xtrace"
assert_no_carrier_disclosure "inherited xtrace internal carrier"

run_world_enable \
  "${AMBIENT_B}" "not-an-authority" "ambient" "ambient" "999999" \
  --help
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "plain public help failed"
grep -Fq -- "Substrate World Enable Helper" "${STDOUT_PATH}" \
  || fail "plain public help did not emit usage"
if grep -Fq -- '$SUBSTRATE_HOME' "${STDOUT_PATH}"; then
  fail "plain public help advertised ambient SUBSTRATE_HOME authority"
fi
grep -Fq -- "account-database home/.substrate" "${STDOUT_PATH}" \
  || fail "plain public help did not describe the account-database-home default"

run_world_enable \
  "${AMBIENT_B}" "not-an-authority" "ambient" "ambient" "999999" \
  --home "${SELECTED_A}" --dry-run
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "public --home did not override environment-only state"
grep -Fq -- "home=${SELECTED_A}" "${STDERR_PATH}" \
  || fail "public --home A was not the selected projection"

run_world_enable \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" --dry-run
assert_failure "internal carrier without --home"

run_world_enable \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" --dry-run
assert_failure "duplicate internal carrier"

run_world_enable \
  "${SELECTED_A}" "not!base64" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "not!base64" --dry-run
assert_failure "malformed internal carrier"

run_world_enable \
  "${SELECTED_A}" "not!base64" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "not!base64" --help
assert_failure "malformed internal carrier before help"

run_world_enable \
  "${SELECTED_A}" "not!base64" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --help --home "${SELECTED_A}" --install-bootstrap-context-v1 "not!base64"
assert_failure "help before malformed internal carrier"

run_world_enable \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1
assert_failure "hidden carrier option without value"

run_world_enable \
  "${SELECTED_A}" "${TAMPERED_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "${TAMPERED_CARRIER}" --dry-run
assert_failure "tampered internal carrier"

run_world_enable \
  "${SELECTED_A}" "${REORDERED_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "${REORDERED_CARRIER}" --dry-run
assert_failure "reordered internal carrier"

run_world_enable \
  "${SELECTED_A}" "${FORGED_CARRIER}" "${FORGED_COMMITMENT}" "forged-principal" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "${FORGED_CARRIER}" --dry-run
assert_failure "forged-principal internal carrier"

run_world_enable \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${CONFLICT_C}" --install-bootstrap-context-v1 "${A_CARRIER}" --dry-run
assert_failure "conflicting internal carrier/home"

run_world_enable \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --home "${SELECTED_A}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" --dry-run
assert_failure "duplicate internal home projection"

assert_fixture_unchanged
printf '[%s] PASS\n' "${SCRIPT_NAME}"
