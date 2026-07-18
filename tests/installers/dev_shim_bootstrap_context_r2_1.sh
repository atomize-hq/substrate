#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-shim-bootstrap-context-r2-1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
BOOTSTRAP_SCRIPT="${REPO_ROOT}/scripts/substrate/dev-shim-bootstrap.sh"
CURRENT_UID="$(id -u)"
FIXTURE_PARENT="${XDG_RUNTIME_DIR:-/run/user/${CURRENT_UID}}"

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Linux" ]]; then
  printf '[%s] SKIP: Linux-only dev-shim proof\n' "${SCRIPT_NAME}"
  exit 0
fi
if [[ "${CURRENT_UID}" -eq 0 ]]; then
  fail "the R2-1 product matrix must run as an unprivileged Unix principal"
fi
if [[ ! -x "${SUBSTRATE_BIN}" || ! -x "${BOOTSTRAP_SCRIPT}" ]]; then
  fail "build the debug substrate binary and retain the executable dev-shim helper"
fi
if [[ ! -d "${FIXTURE_PARENT}" || ! -O "${FIXTURE_PARENT}" ]]; then
  fail "secure audit-owned fixture parent is unavailable: ${FIXTURE_PARENT}"
fi

WORK_ROOT="$(mktemp -d "${FIXTURE_PARENT%/}/substrate-r2-1-dev-shim.XXXXXX")"
chmod 0700 "${WORK_ROOT}"
cleanup() {
  chmod -R u+rwX "${WORK_ROOT}" 2>/dev/null || true
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

SELECTED_A="${WORK_ROOT}/selected-a"
AMBIENT_B="${WORK_ROOT}/ambient-b"
mkdir "${AMBIENT_B}"
chmod 0700 "${AMBIENT_B}"
printf 'ambient-b-sentinel\n' > "${AMBIENT_B}/sentinel"

assert_b_untouched() {
  local entries
  chmod 0700 "${AMBIENT_B}"
  entries="$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -printf '%f\n' | sort)"
  [[ "${entries}" == "sentinel" ]] \
    || fail "dev-shim propagation mutated ambient B (entries: ${entries})"
  [[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
    || fail "ambient B sentinel changed"
}

run_helper() {
  local action="$1"
  local output="$2"
  chmod 0000 "${AMBIENT_B}"
  set +e
  HOME="${AMBIENT_B}" \
    USERPROFILE="${AMBIENT_B}" \
    SUBSTRATE_HOME="${AMBIENT_B}" \
    SUBSTRATE_ROOT="${AMBIENT_B}" \
    SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" \
    "${BOOTSTRAP_SCRIPT}" "${action}" --bin "${SUBSTRATE_BIN}" --prefix "${SELECTED_A}" \
    >"${output}.out" 2>"${output}.err"
  HELPER_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

run_bound_substrate() {
  local output="$1"
  shift
  chmod 0000 "${AMBIENT_B}"
  set +e
  HOME="${AMBIENT_B}" \
    USERPROFILE="${AMBIENT_B}" \
    SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" \
    SUBSTRATE_HOME="${SELECTED_A}" \
    SUBSTRATE_ROOT="${SELECTED_A}" \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${BOUND_COMMITMENT}" \
    SUBSTRATE_INSTALL_PRIMARY_USER="${BOUND_ACCOUNT}" \
    SUBSTRATE_INSTALL_PRIMARY_UID="${BOUND_UID}" \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${BOUND_CARRIER}" \
    SHIM_ORIGINAL_PATH="${PATH}" \
    "${SUBSTRATE_BIN}" --install-bootstrap-context-v1 "${BOUND_CARRIER}" "$@" \
    >"${output}.out" 2>"${output}.err"
  BOUND_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

run_helper --install "${WORK_ROOT}/explicit-install"
[[ "${HELPER_RC}" -eq 0 ]] || fail "explicit dev-shim install failed"
[[ -d "${SELECTED_A}/shims" ]] || fail "dev-shim install did not deploy under A"
[[ -f "${SELECTED_A}/dev-shim-env.sh" ]] || fail "dev-shim install did not generate A env projection"
[[ -f "${SELECTED_A}/trace.jsonl" ]] || fail "explicit shim deploy did not initialize A trace"
if grep -R -F -q -- "${AMBIENT_B}" "${SELECTED_A}"; then
  fail "dev-shim generated projection captured ambient B"
fi

# The generated file is a projection of A; capture its checked tuple before uninstall removes it.
AUDIT_PATH="${PATH}"
# shellcheck disable=SC1090,SC1091
source "${SELECTED_A}/dev-shim-env.sh"
BOUND_CARRIER="${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1}"
BOUND_COMMITMENT="${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT}"
BOUND_ACCOUNT="${SUBSTRATE_INSTALL_PRIMARY_USER}"
BOUND_UID="${SUBSTRATE_INSTALL_PRIMARY_UID}"
[[ "${SUBSTRATE_HOME}" == "${SELECTED_A}" && "${SUBSTRATE_ROOT}" == "${SELECTED_A}" ]] \
  || fail "generated dev-shim environment did not project A"
# Physical-shim trace migration belongs to R2-3; keep this R2-1 matrix on shell-owned paths.
PATH="${AUDIT_PATH}"
export PATH

run_bound_substrate "${WORK_ROOT}/status-deployed" --no-world --shim-status-json
[[ "${BOUND_RC}" -eq 0 ]] || fail "deployed shim status failed with exit ${BOUND_RC}"
grep -Fq "${SELECTED_A}/shims" "${WORK_ROOT}/status-deployed.out" \
  || fail "shim status did not report A"

cp "${REPO_ROOT}/config/manager_hooks.yaml" "${SELECTED_A}/manager_hooks.yaml"
chmod 0644 "${SELECTED_A}/manager_hooks.yaml"
run_bound_substrate "${WORK_ROOT}/doctor" --no-world shim doctor --json
[[ "${BOUND_RC}" -eq 0 || "${BOUND_RC}" -eq 1 ]] \
  || fail "shim doctor returned unexpected exit ${BOUND_RC}"
grep -Fq "\"selected_host_prefix\": \"${SELECTED_A}\"" "${WORK_ROOT}/doctor.out" \
  || fail "shim doctor did not report selected A"
grep -Fq "\"host_context_commitment\": \"${BOUND_COMMITMENT}\"" \
  "${WORK_ROOT}/doctor.out" || fail "shim doctor did not report the bound commitment"

run_helper --uninstall "${WORK_ROOT}/explicit-uninstall"
[[ "${HELPER_RC}" -eq 0 ]] || fail "explicit dev-shim uninstall failed"
[[ ! -e "${SELECTED_A}/shims" ]] || fail "explicit shim remove left the A shim tree"
[[ ! -e "${SELECTED_A}/dev-shim-env.sh" ]] || fail "dev-shim env survived explicit uninstall"

# A normal shell entry with the same authenticated tuple performs automatic deployment at A.
run_bound_substrate "${WORK_ROOT}/automatic-deploy" --no-world -c true
[[ "${BOUND_RC}" -eq 0 ]] || fail "automatic shim deploy failed with exit ${BOUND_RC}"
[[ -d "${SELECTED_A}/shims" ]] || fail "automatic shim deploy did not target A"
if grep -R -F -q -- "${AMBIENT_B}" "${SELECTED_A}"; then
  fail "automatic deploy captured B"
fi

run_bound_substrate "${WORK_ROOT}/automatic-status" --no-world --shim-status-json
[[ "${BOUND_RC}" -eq 0 ]] || fail "automatic deploy status failed with exit ${BOUND_RC}"
run_bound_substrate "${WORK_ROOT}/explicit-remove" --no-world --shim-remove
[[ "${BOUND_RC}" -eq 0 ]] || fail "explicit shim remove failed with exit ${BOUND_RC}"
[[ ! -e "${SELECTED_A}/shims" ]] || fail "explicit shim remove did not remove A shims"
run_bound_substrate "${WORK_ROOT}/status-removed" --no-world --shim-status-json
[[ "${BOUND_RC}" -eq 1 ]] || fail "removed shim status returned ${BOUND_RC}, expected 1"

assert_b_untouched
[[ ! -e "${AMBIENT_B}/trace.jsonl" ]] || fail "shim lifecycle wrote trace output under B"

# Contract-valid Unix prefixes may contain shell metacharacters. The generated helper must remain
# valid Bash and self-derive that exact A instead of interpolating it into shell syntax.
SELECTED_A="${WORK_ROOT}/selected-\"-a"
run_helper --install "${WORK_ROOT}/quoted-install"
[[ "${HELPER_RC}" -eq 0 ]] || fail "quoted-prefix dev-shim install failed"
bash -n "${SELECTED_A}/dev-shim-env.sh" \
  || fail "dev-shim helper is invalid for a contract-valid quoted prefix"
run_helper --uninstall "${WORK_ROOT}/quoted-uninstall"
[[ "${HELPER_RC}" -eq 0 ]] || fail "quoted-prefix dev-shim uninstall failed"
[[ ! -e "${SELECTED_A}/dev-shim-env.sh" ]] \
  || fail "quoted-prefix dev-shim helper survived uninstall"

printf '[%s] PASS: explicit/automatic deploy, status, doctor, remove, generated, and A/B proof\n' \
  "${SCRIPT_NAME}"
