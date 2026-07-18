#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="standalone-cli-prefix-r2-1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
CURRENT_UID="$(id -u)"
CURRENT_ACCOUNT="$(id -un)"
FIXTURE_PARENT="${XDG_RUNTIME_DIR:-/run/user/${CURRENT_UID}}"

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Linux" ]]; then
  printf '[%s] SKIP: Linux-only R2-1 witness proof\n' "${SCRIPT_NAME}"
  exit 0
fi
if [[ "${CURRENT_UID}" -eq 0 ]]; then
  fail "the R2-1 product matrix must run as an unprivileged Unix principal"
fi
if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
  fail "missing ${SUBSTRATE_BIN}; build the debug substrate binary first"
fi
if [[ ! -d "${FIXTURE_PARENT}" || ! -O "${FIXTURE_PARENT}" ]]; then
  fail "secure audit-owned fixture parent is unavailable: ${FIXTURE_PARENT}"
fi

WORK_ROOT="$(mktemp -d "${FIXTURE_PARENT%/}/substrate-r2-1-standalone.XXXXXX")"
chmod 0700 "${WORK_ROOT}"
cleanup() {
  chmod -R u+rwX "${WORK_ROOT}" 2>/dev/null || true
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

AMBIENT_B="${WORK_ROOT}/ambient-b"
mkdir "${AMBIENT_B}"
chmod 0700 "${AMBIENT_B}"
printf 'ambient-b-sentinel\n' > "${AMBIENT_B}/sentinel"

assert_b_untouched() {
  local entries
  chmod 0700 "${AMBIENT_B}"
  entries="$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -printf '%f\n' | sort)"
  [[ "${entries}" == "sentinel" ]] || fail "ambient B was read as writable authority or mutated"
  [[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
    || fail "ambient B sentinel changed"
}

run_product_command() {
  local stdout_path="$1"
  local stderr_path="$2"
  shift 2
  chmod 0000 "${AMBIENT_B}"
  set +e
  HOME="${AMBIENT_B}" \
    USERPROFILE="${AMBIENT_B}" \
    SUBSTRATE_HOME="${AMBIENT_B}" \
    SUBSTRATE_ROOT="${AMBIENT_B}" \
    SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" \
    "$@" >"${stdout_path}" 2>"${stderr_path}"
  PRODUCT_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

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
carrier = base64.urlsafe_b64encode(record).rstrip(b"=").decode("ascii")
print(carrier)
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
tampered = before + marker + first + commitment[1:]
print(base64.urlsafe_b64encode(tampered).rstrip(b"=").decode("ascii"))
PY
}

assert_no_scaffold() {
  local prefix="$1"
  [[ ! -e "${prefix}" ]] || fail "non-mutating invocation created ${prefix}"
}

# Parse/help/version/version-json must all return without selecting or creating a product root.
for mode in help version version_json parse_failure; do
  case "${mode}" in
    help) nonmutating_root="${WORK_ROOT}/nonmutating-help"; args=(--install-prefix "${nonmutating_root}" --help) ;;
    version) nonmutating_root="${WORK_ROOT}/nonmutating-version"; args=(--install-prefix "${nonmutating_root}" --version) ;;
    version_json) nonmutating_root="${WORK_ROOT}/nonmutating-version-json"; args=(--install-prefix "${nonmutating_root}" --version-json) ;;
    parse_failure) nonmutating_root="${WORK_ROOT}/nonmutating-parse"; args=(--install-prefix "${nonmutating_root}" --not-a-real-option) ;;
  esac
  run_product_command "${WORK_ROOT}/${mode}.out" "${WORK_ROOT}/${mode}.err" \
    "${SUBSTRATE_BIN}" "${args[@]}"
  if [[ "${mode}" == "parse_failure" ]]; then
    [[ "${PRODUCT_RC}" -eq 2 ]] || fail "parse failure returned ${PRODUCT_RC}, expected 2"
  else
    [[ "${PRODUCT_RC}" -eq 0 ]] || fail "${mode} returned ${PRODUCT_RC}, expected 0"
  fi
  assert_no_scaffold "${nonmutating_root}"
done
assert_b_untouched

# The hidden bootstrap action requires an argv carrier; environment state alone is insufficient.
MISSING_ROOT="${WORK_ROOT}/missing-carrier"
run_product_command "${WORK_ROOT}/missing.out" "${WORK_ROOT}/missing.err" \
  "${SUBSTRATE_BIN}" --install-prefix "${MISSING_ROOT}" --install-bootstrap-home-v1
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "missing hidden carrier did not fail with exit 2"
assert_no_scaffold "${MISSING_ROOT}"

ENV_ONLY_ROOT="${WORK_ROOT}/environment-only"
mapfile -t ENV_ONLY_VALUES < <(make_context_values "${ENV_ONLY_ROOT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}")
ENV_ONLY_CARRIER="${ENV_ONLY_VALUES[0]}"
ENV_ONLY_COMMITMENT="${ENV_ONLY_VALUES[1]}"
chmod 0000 "${AMBIENT_B}"
set +e
HOME="${AMBIENT_B}" \
  SUBSTRATE_HOME="${ENV_ONLY_ROOT}" \
  SUBSTRATE_ROOT="${ENV_ONLY_ROOT}" \
  SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${ENV_ONLY_COMMITMENT}" \
  SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
  SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
  SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${ENV_ONLY_CARRIER}" \
  "${SUBSTRATE_BIN}" --install-prefix "${ENV_ONLY_ROOT}" \
    --install-bootstrap-home-v1 >"${WORK_ROOT}/env-only.out" 2>"${WORK_ROOT}/env-only.err"
ENV_ONLY_RC=$?
set -e
chmod 0700 "${AMBIENT_B}"
[[ "${ENV_ONLY_RC}" -eq 2 ]] || fail "environment alone selected hidden bootstrap mode"
assert_no_scaffold "${ENV_ONLY_ROOT}"

# Exact carrier retry is idempotent; malformed, tampered, and forged principals reject first.
VALID_ROOT="${WORK_ROOT}/valid-hidden-bootstrap"
mapfile -t VALID_VALUES < <(make_context_values "${VALID_ROOT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}")
VALID_CARRIER="${VALID_VALUES[0]}"
VALID_COMMITMENT="${VALID_VALUES[1]}"
for attempt in 1 2; do
  HOME="${AMBIENT_B}" \
    SUBSTRATE_HOME="${VALID_ROOT}" \
    SUBSTRATE_ROOT="${VALID_ROOT}" \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${VALID_COMMITMENT}" \
    SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
    SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${VALID_CARRIER}" \
    "${SUBSTRATE_BIN}" --install-bootstrap-context-v1 "${VALID_CARRIER}" \
      --install-bootstrap-home-v1 >"${WORK_ROOT}/valid-${attempt}.out" \
      2>"${WORK_ROOT}/valid-${attempt}.err"
done
[[ -d "${VALID_ROOT}/deps" ]] || fail "valid hidden bootstrap did not create A-scoped deps"
[[ ! -e "${VALID_ROOT}/trace.jsonl" ]] || fail "hidden bootstrap unexpectedly initialized tracing"

MALFORMED_ROOT="${WORK_ROOT}/malformed-carrier"
run_product_command "${WORK_ROOT}/malformed.out" "${WORK_ROOT}/malformed.err" \
  "${SUBSTRATE_BIN}" --install-bootstrap-context-v1 'not!base64' \
  --install-bootstrap-home-v1
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "malformed carrier did not fail with exit 2"
assert_no_scaffold "${MALFORMED_ROOT}"

TAMPERED_ROOT="${WORK_ROOT}/tampered-carrier"
mapfile -t TAMPERED_VALUES < <(make_context_values "${TAMPERED_ROOT}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}")
TAMPERED_CARRIER="$(tamper_commitment "${TAMPERED_VALUES[0]}")"
run_product_command "${WORK_ROOT}/tampered.out" "${WORK_ROOT}/tampered.err" \
  "${SUBSTRATE_BIN}" --install-bootstrap-context-v1 "${TAMPERED_CARRIER}" \
  --install-bootstrap-home-v1
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "tampered commitment did not fail with exit 2"
assert_no_scaffold "${TAMPERED_ROOT}"

FORGED_ROOT="${WORK_ROOT}/forged-principal"
FORGED_UID="$((CURRENT_UID + 1))"
mapfile -t FORGED_VALUES < <(make_context_values "${FORGED_ROOT}" "${CURRENT_ACCOUNT}" "${FORGED_UID}")
run_product_command "${WORK_ROOT}/forged.out" "${WORK_ROOT}/forged.err" \
  "${SUBSTRATE_BIN}" --install-bootstrap-context-v1 "${FORGED_VALUES[0]}" \
  --install-bootstrap-home-v1
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "forged principal did not fail with exit 2"
assert_no_scaffold "${FORGED_ROOT}"
if grep -Fq "${VALID_CARRIER}" "${WORK_ROOT}"/*.out "${WORK_ROOT}"/*.err; then
  fail "a hidden carrier was emitted in normal output or diagnostics"
fi

# A dev A/bin symlink and an installed release link self-derive A under conflicting B.
DEV_A="${WORK_ROOT}/dev-a"
mkdir -p "${DEV_A}/bin"
chmod 0700 "${DEV_A}"
chmod 0755 "${DEV_A}/bin"
ln -s "${SUBSTRATE_BIN}" "${DEV_A}/bin/substrate"
run_product_command "${WORK_ROOT}/dev.out" "${WORK_ROOT}/dev.err" \
  "${DEV_A}/bin/substrate" --no-world --shim-skip -c true
[[ "${PRODUCT_RC}" -eq 0 ]] || fail "dev A/bin witness failed with exit ${PRODUCT_RC}"
[[ -s "${DEV_A}/trace.jsonl" ]] || fail "dev witness did not bind trace output to A"

RELATIVE_TRACE_SIZE="$(stat -c %s "${DEV_A}/trace.jsonl")"
(
  cd "${DEV_A}"
  HOME="${AMBIENT_B}" SUBSTRATE_HOME="${AMBIENT_B}" SUBSTRATE_ROOT="${AMBIENT_B}" \
    SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" ./bin/substrate --no-world --shim-skip -c true
) >"${WORK_ROOT}/relative.out" 2>"${WORK_ROOT}/relative.err"
[[ "$(stat -c %s "${DEV_A}/trace.jsonl")" -gt "${RELATIVE_TRACE_SIZE}" ]] \
  || fail "explicitly relative invocation did not reuse its derived A"

RELEASE_A="${WORK_ROOT}/release-a"
mkdir -p "${RELEASE_A}/versions/v1/bin" "${RELEASE_A}/bin"
chmod 0700 "${RELEASE_A}"
chmod 0755 "${RELEASE_A}/versions" "${RELEASE_A}/versions/v1" \
  "${RELEASE_A}/versions/v1/bin" "${RELEASE_A}/bin"
cp "${SUBSTRATE_BIN}" "${RELEASE_A}/versions/v1/bin/substrate"
chmod 0755 "${RELEASE_A}/versions/v1/bin/substrate"
ln -s ../versions/v1/bin/substrate "${RELEASE_A}/bin/substrate"
run_product_command "${WORK_ROOT}/release-link.out" "${WORK_ROOT}/release-link.err" \
  "${RELEASE_A}/bin/substrate" --no-world --shim-skip -c true
[[ "${PRODUCT_RC}" -eq 0 ]] || fail "release A/bin link failed with exit ${PRODUCT_RC}"
[[ -s "${RELEASE_A}/trace.jsonl" ]] || fail "release link did not bind trace output to A"
run_product_command "${WORK_ROOT}/release-payload.out" "${WORK_ROOT}/release-payload.err" \
  "${RELEASE_A}/versions/v1/bin/substrate" --no-world --shim-skip -c true
[[ "${PRODUCT_RC}" -eq 0 ]] || fail "release payload witness failed with exit ${PRODUCT_RC}"

# Repository, bare-zero, bare-multiple, and identity-mismatched witnesses fail closed.
run_product_command "${WORK_ROOT}/repo-direct.out" "${WORK_ROOT}/repo-direct.err" \
  "${SUBSTRATE_BIN}" --shim-status-json
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "direct repository binary became an installed witness"

EMPTY_PATH="${WORK_ROOT}/empty-path"
mkdir "${EMPTY_PATH}"
# shellcheck disable=SC2016
run_product_command "${WORK_ROOT}/bare-zero.out" "${WORK_ROOT}/bare-zero.err" \
  /bin/bash -c 'PATH="$2" exec -a substrate "$1" --shim-status-json' \
  r2-1-bare-zero "${SUBSTRATE_BIN}" "${EMPTY_PATH}"
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "bare invocation with zero candidates did not fail"

BARE_ONE="${WORK_ROOT}/bare-one"
BARE_TWO="${WORK_ROOT}/bare-two"
mkdir -p "${BARE_ONE}/bin" "${BARE_TWO}/bin"
chmod 0700 "${BARE_ONE}" "${BARE_TWO}"
chmod 0755 "${BARE_ONE}/bin" "${BARE_TWO}/bin"
ln -s "${SUBSTRATE_BIN}" "${BARE_ONE}/bin/substrate"
ln -s "${SUBSTRATE_BIN}" "${BARE_TWO}/bin/substrate"
# shellcheck disable=SC2016
run_product_command "${WORK_ROOT}/bare-multiple.out" "${WORK_ROOT}/bare-multiple.err" \
  /bin/bash -c 'PATH="$2:$3" exec -a substrate "$1" --shim-status-json' \
  r2-1-bare-multiple "${SUBSTRATE_BIN}" "${BARE_ONE}/bin" "${BARE_TWO}/bin"
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "PATH order granted precedence to one of multiple witnesses"

MISMATCH_A="${WORK_ROOT}/identity-mismatch"
mkdir -p "${MISMATCH_A}/bin"
chmod 0700 "${MISMATCH_A}"
chmod 0755 "${MISMATCH_A}/bin"
cp "${SUBSTRATE_BIN}" "${WORK_ROOT}/different-substrate"
chmod 0755 "${WORK_ROOT}/different-substrate"
ln -s "${WORK_ROOT}/different-substrate" "${MISMATCH_A}/bin/substrate"
# shellcheck disable=SC2016
run_product_command "${WORK_ROOT}/identity-mismatch.out" "${WORK_ROOT}/identity-mismatch.err" \
  /bin/bash -c 'exec -a "$2" "$1" --shim-status-json' \
  r2-1-identity-mismatch "${SUBSTRATE_BIN}" "${MISMATCH_A}/bin/substrate"
[[ "${PRODUCT_RC}" -eq 2 ]] || fail "identity-mismatched invocation witness was accepted"

assert_b_untouched
for selected in "${DEV_A}" "${RELEASE_A}"; do
  [[ ! -e "${AMBIENT_B}/trace.jsonl" ]] || fail "trace output escaped A"
  if grep -R -F -q -- "${AMBIENT_B}" "${selected}"; then
    fail "an A-scoped generated artifact captured ambient B"
  fi
done

printf '[%s] PASS: hidden carrier, nonmutation, witness, trace, and A/B authority proof\n' \
  "${SCRIPT_NAME}"
