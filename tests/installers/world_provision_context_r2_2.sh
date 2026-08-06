#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="world-provision-context-r2-2"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORLD_ENABLE="${REPO_ROOT}/scripts/substrate/world-enable.sh"
WORLD_PROVISION="${REPO_ROOT}/scripts/linux/world-provision.sh"
ACL_HELPER="${REPO_ROOT}/scripts/linux/substrate-apply-socket-acl.sh"
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

run_world_provision() {
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
    "${WORLD_PROVISION}" "$@" >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
  WORLD_ENABLE_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

run_world_provision_with_inherited_xtrace() {
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
    bash -x "${WORLD_PROVISION}" \
      --home "${SELECTED_A}" \
      --install-bootstrap-context-v1 "${A_CARRIER}" \
      --profile r2-2-context-fixture --skip-build --dry-run >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
  WORLD_ENABLE_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

assert_acl_tuple_rejected() {
  local label="$1"
  shift
  set +e
  "${ACL_HELPER}" "$@" >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
  local status=$?
  set -e
  [[ "${status}" -eq 2 ]] || fail "${label} returned ${status}, expected parser rejection 2"
  [[ ! -s "${STDOUT_PATH}" ]] || fail "${label} emitted normal output"
  grep -Fq -- "rejected noncanonical ACL tuple" "${STDERR_PATH}" \
    || fail "${label} did not report closed-tuple rejection"
}

assert_failure() {
  local label="$1"
  [[ "${WORLD_ENABLE_RC}" -ne 0 ]] || fail "${label} unexpectedly succeeded"
  [[ ! -s "${STDOUT_PATH}" ]] || fail "${label} emitted normal output before validation"
}

assert_world_provision_rejected_pre_action() {
  local label="$1"
  assert_failure "${label}"
  if grep -Eq -- 'Skipping build|Validating private|sudo may prompt|Provisioning complete|\[dry-run\]' \
      "${STDOUT_PATH}" "${STDERR_PATH}"; then
    fail "${label} reached a world-provision action before rejection"
  fi
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
  [[ -z "${entries}" ]] || fail "carrier validation mutated a selected prefix: ${entries}"
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
# shellcheck disable=SC2016
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

run_world_provision \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" \
  --profile r2-2-context-fixture --skip-build --dry-run
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "exact world-provision internal carrier/home projection failed"
assert_no_carrier_disclosure "world-provision exact internal carrier"
grep -Fq -- "SUBSTRATE_HOME=${SELECTED_A}" "${STDOUT_PATH}" "${STDERR_PATH}" \
  || fail "world-provision dry-run did not project A"

run_world_provision_with_inherited_xtrace
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "world-provision failed under inherited xtrace"
assert_no_carrier_disclosure "world-provision inherited xtrace carrier"

run_world_provision \
  "${SELECTED_A}" "not!base64" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --help --home "${SELECTED_A}" --install-bootstrap-context-v1 "not!base64"
assert_failure "world-provision help before malformed internal carrier"

run_world_provision \
  "${SELECTED_A}" "not!base64" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "not!base64" --help
assert_failure "world-provision malformed internal carrier before help"

run_world_provision \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" --profile r2-2-context-fixture --skip-build --dry-run
assert_failure "world-provision internal carrier without home"

run_world_provision \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" \
  --profile r2-2-context-fixture --skip-build --dry-run
assert_failure "world-provision duplicate internal carrier"

run_world_provision \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1
assert_failure "world-provision hidden carrier without value"

run_world_provision \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${CONFLICT_C}" \
  --install-bootstrap-context-v1 "${A_CARRIER}" \
  --profile r2-2-context-fixture --skip-build --dry-run
assert_failure "world-provision conflicting internal carrier/home"

run_world_provision \
  "${SELECTED_A}" "${TAMPERED_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "${TAMPERED_CARRIER}" \
  --profile r2-2-context-fixture --skip-build --dry-run
assert_world_provision_rejected_pre_action "world-provision tampered carrier"

run_world_provision \
  "${SELECTED_A}" "${REORDERED_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "${REORDERED_CARRIER}" \
  --profile r2-2-context-fixture --skip-build --dry-run
assert_world_provision_rejected_pre_action "world-provision reordered carrier"

run_world_provision \
  "${SELECTED_A}" "${FORGED_CARRIER}" "${FORGED_COMMITMENT}" "forged-principal" "${CURRENT_UID}" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "${FORGED_CARRIER}" \
  --profile r2-2-context-fixture --skip-build --dry-run
assert_world_provision_rejected_pre_action "world-provision forged-principal carrier"

run_world_provision \
  "${AMBIENT_B}" "${A_CARRIER}" "ambient-commitment" "forged-principal" "999999" \
  --home "${SELECTED_A}" --install-bootstrap-context-v1 "${A_CARRIER}" \
  --profile r2-2-context-fixture --skip-build --dry-run
assert_world_provision_rejected_pre_action "world-provision conflicting checked projections"

run_world_provision \
  "${AMBIENT_B}" "not-an-authority" "ambient" "ambient" "999999" \
  --home "${SELECTED_A}" --profile r2-2-context-fixture --skip-build --dry-run
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "world-provision public --home failed under ambient B"
grep -Fq -- "SUBSTRATE_HOME=${SELECTED_A}" "${STDOUT_PATH}" "${STDERR_PATH}" \
  || fail "world-provision public --home did not select A"

run_world_provision \
  "${SELECTED_A}" "${A_CARRIER}" "${A_COMMITMENT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}" \
  --profile r2-2-context-fixture --skip-build --dry-run
[[ "${WORLD_ENABLE_RC}" -eq 0 ]] || fail "world-provision public default failed with environment-only carrier"
if grep -Fq -- "SUBSTRATE_HOME=${SELECTED_A}" "${STDOUT_PATH}" "${STDERR_PATH}"; then
  fail "world-provision environment-only carrier selected A"
fi

assert_acl_tuple_rejected "ACL tuple missing all fields"
assert_acl_tuple_rejected "ACL tuple missing group" --socket /run/substrate.sock
assert_acl_tuple_rejected "ACL tuple extra field" --socket /run/substrate.sock substrate extra
assert_acl_tuple_rejected "ACL tuple wrong socket target" --socket /tmp/substrate.sock substrate
assert_acl_tuple_rejected "ACL tuple wrong directory target" --directory-traverse /run/substrate.sock substrate
assert_acl_tuple_rejected "ACL tuple wrong tree group" --tree-readonly /var/lib/substrate/world-deps root

assert_fixture_unchanged
printf '[%s] PASS\n' "${SCRIPT_NAME}"
