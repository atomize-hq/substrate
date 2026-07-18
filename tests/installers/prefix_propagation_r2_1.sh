#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="prefix-propagation-r2-1"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_SCRIPT="${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh"
UNINSTALL_SCRIPT="${REPO_ROOT}/scripts/substrate/dev-uninstall-substrate.sh"
SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
CURRENT_UID="$(id -u)"
FIXTURE_PARENT="${XDG_RUNTIME_DIR:-/run/user/${CURRENT_UID}}"
ACCOUNT_HOME="$(getent passwd "${CURRENT_UID}" | awk -F: 'NR == 1 { print $6 }')"

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Linux" ]]; then
  printf '[%s] SKIP: Linux-only dev install/uninstall proof\n' "${SCRIPT_NAME}"
  exit 0
fi
if [[ "${CURRENT_UID}" -eq 0 ]]; then
  fail "the R2-1 product matrix must run as an unprivileged Unix principal"
fi
if [[ ! -x "${INSTALL_SCRIPT}" || ! -x "${UNINSTALL_SCRIPT}" ]]; then
  fail "development installer scripts are not executable"
fi
if [[ ! -d "${FIXTURE_PARENT}" || ! -O "${FIXTURE_PARENT}" ]]; then
  fail "secure audit-owned fixture parent is unavailable: ${FIXTURE_PARENT}"
fi
if [[ -z "${ACCOUNT_HOME}" || ! -d "${ACCOUNT_HOME}" ]]; then
  fail "current account-database home is unavailable"
fi

WORK_ROOT="$(mktemp -d "${FIXTURE_PARENT%/}/substrate-r2-1-prefix.XXXXXX")"
chmod 0700 "${WORK_ROOT}"
cleanup() {
  chmod -R u+rwX "${WORK_ROOT}" 2>/dev/null || true
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

SELECTED_A="${WORK_ROOT}/selected-\"-a"
AMBIENT_B="${WORK_ROOT}/ambient-b"
TOOL_CONFIG_HOME="${WORK_ROOT}/tool-config"
mkdir "${AMBIENT_B}" "${TOOL_CONFIG_HOME}"
chmod 0700 "${AMBIENT_B}"
chmod 0700 "${TOOL_CONFIG_HOME}"
printf 'ambient-b-sentinel\n' > "${AMBIENT_B}/sentinel"

assert_b_untouched() {
  local entries
  chmod 0700 "${AMBIENT_B}"
  entries="$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -printf '%f\n' | sort)"
  [[ "${entries}" == "sentinel" ]] \
    || fail "dev install/uninstall mutated ambient B (entries: ${entries})"
  [[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
    || fail "ambient B sentinel changed"
}

context_hash_from_script() {
  local script="$1"
  chmod 0000 "${AMBIENT_B}"
  HOME="${AMBIENT_B}" \
    SUBSTRATE_HOME="${AMBIENT_B}" \
    SUBSTRATE_ROOT="${AMBIENT_B}" \
    bash -s -- "${script}" "${SELECTED_A}" <<'BASH'
set -euo pipefail
script="$1"
selected="$2"
# Source only the resolver prelude; do not execute the installer body.
# shellcheck disable=SC1090
source <(awk '/^usage\(\)/ { exit } { print }' "${script}")
resolve_install_bootstrap_context 1 "${selected}" ""
printf '%s' "${INSTALL_BOOTSTRAP_CONTEXT_V1}" | sha256sum | awk '{ print $1 }'
BASH
  chmod 0700 "${AMBIENT_B}"
}

INSTALL_CONTEXT_HASH="$(context_hash_from_script "${INSTALL_SCRIPT}")"
UNINSTALL_CONTEXT_HASH="$(context_hash_from_script "${UNINSTALL_SCRIPT}")"
[[ "${INSTALL_CONTEXT_HASH}" == "${UNINSTALL_CONTEXT_HASH}" ]] \
  || fail "dev install and uninstall constructed different IH carriers for A"
assert_b_untouched

run_dev_install() {
  local label="$1"
  chmod 0000 "${AMBIENT_B}"
  set +e
  HOME="${AMBIENT_B}" \
    USERPROFILE="${AMBIENT_B}" \
    SUBSTRATE_HOME="${AMBIENT_B}" \
    SUBSTRATE_ROOT="${AMBIENT_B}" \
    SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" \
    CARGO_HOME="${ACCOUNT_HOME%/}/.cargo" \
    RUSTUP_HOME="${ACCOUNT_HOME%/}/.rustup" \
    XDG_CONFIG_HOME="${TOOL_CONFIG_HOME}" \
    GIT_CONFIG_GLOBAL=/dev/null \
    GIT_CONFIG_NOSYSTEM=1 \
    "${INSTALL_SCRIPT}" --prefix "${SELECTED_A}" --profile debug \
      --version-label dev --no-world --no-shims \
      >"${WORK_ROOT}/install-${label}.out" 2>"${WORK_ROOT}/install-${label}.err"
  INSTALL_RC=$?
  set -e
  chmod 0700 "${AMBIENT_B}"
}

run_dev_install first
[[ "${INSTALL_RC}" -eq 0 ]] || {
  sed -n '1,160p' "${WORK_ROOT}/install-first.err" >&2
  fail "first dev install failed with exit ${INSTALL_RC}"
}

for required in \
  env.sh \
  manager_env.sh \
  manager_init.sh \
  manager_hooks.yaml \
  dev-shim-env.sh \
  config.yaml \
  deps/README.md \
  versions/dev/config/manager_hooks.yaml \
  versions/dev/config/world-deps.yaml \
  bin/substrate; do
  [[ -e "${SELECTED_A}/${required}" || -L "${SELECTED_A}/${required}" ]] \
    || fail "dev install omitted A-scoped projection ${required}"
done
bash -n "${SELECTED_A}/dev-shim-env.sh" \
  || fail "dev-install helper is invalid for a contract-valid quoted prefix"
cmp "${SELECTED_A}/manager_hooks.yaml" \
  "${SELECTED_A}/versions/dev/config/manager_hooks.yaml" \
  || fail "A manager base is not the selected generated manifest projection"
if grep -R -F -q -- "${AMBIENT_B}" "${SELECTED_A}"; then
  fail "a generated install projection captured ambient B"
fi
if grep -Eq '\$\{?HOME\}?' "${SELECTED_A}/manager_env.sh"; then
  fail "generated manager environment retained ambient HOME path selection"
fi
grep -Fq "${ACCOUNT_HOME%/}/.substrate_bashenv" "${SELECTED_A}/manager_env.sh" \
  || fail "manager environment did not bind legacy Bash input to account-database home"

# A normal config mutation rewrites env.sh through the shell path; it must retain the complete
# authenticated A projection rather than collapsing back to a lone ambient home export.
chmod 0000 "${AMBIENT_B}"
set +e
HOME="${AMBIENT_B}" \
  USERPROFILE="${AMBIENT_B}" \
  SUBSTRATE_HOME="${AMBIENT_B}" \
  SUBSTRATE_ROOT="${AMBIENT_B}" \
  SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" \
  "${SELECTED_A}/bin/substrate" --no-world --shim-skip \
    config global init --force >"${WORK_ROOT}/config-init.out" \
    2>"${WORK_ROOT}/config-init.err"
CONFIG_INIT_RC=$?
set -e
chmod 0700 "${AMBIENT_B}"
[[ "${CONFIG_INIT_RC}" -eq 0 ]] || fail "A-bound config init failed with exit ${CONFIG_INIT_RC}"
for projection in \
  SUBSTRATE_HOME \
  SUBSTRATE_ROOT \
  SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT \
  SUBSTRATE_INSTALL_PRIMARY_USER \
  SUBSTRATE_INSTALL_PRIMARY_UID \
  SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1; do
  grep -q "^export ${projection}=" "${SELECTED_A}/env.sh" \
    || fail "config init stripped ${projection} from A/env.sh"
done
assert_b_untouched

# Exercise normal shell generation with physical shims skipped; failure to reach a world service is
# not privileged proof, but manager and Bash-preexec projections must already be bound to A.
chmod 0000 "${AMBIENT_B}"
set +e
HOME="${AMBIENT_B}" \
  USERPROFILE="${AMBIENT_B}" \
  SUBSTRATE_HOME="${AMBIENT_B}" \
  SUBSTRATE_ROOT="${AMBIENT_B}" \
  SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" \
  "${SELECTED_A}/bin/substrate" --world --shim-skip -c true \
  >"${WORK_ROOT}/shell-projections.out" 2>"${WORK_ROOT}/shell-projections.err"
SHELL_PROJECTION_RC=$?
set -e
chmod 0700 "${AMBIENT_B}"
[[ "${SHELL_PROJECTION_RC}" -eq 0 || "${SHELL_PROJECTION_RC}" -eq 1 ]] \
  || fail "shell projection probe returned unexpected exit ${SHELL_PROJECTION_RC}"
[[ -f "${SELECTED_A}/manager_env.sh" ]] || fail "normal shell omitted A manager projection"
[[ -f "${SELECTED_A}/.substrate_preexec" ]] || fail "normal shell omitted A Bash-preexec projection"
if grep -R -F -q -- "${AMBIENT_B}" "${SELECTED_A}"; then
  fail "normal shell generated a projection containing B"
fi
[[ -f "${SELECTED_A}/trace.jsonl" ]] || fail "normal shell did not initialize A trace"

run_dev_install repeat
[[ "${INSTALL_RC}" -eq 0 ]] || {
  sed -n '1,160p' "${WORK_ROOT}/install-repeat.err" >&2
  fail "repeat dev install failed with exit ${INSTALL_RC}"
}
assert_b_untouched

chmod 0000 "${AMBIENT_B}"
set +e
HOME="${AMBIENT_B}" \
  USERPROFILE="${AMBIENT_B}" \
  SUBSTRATE_HOME="${AMBIENT_B}" \
  SUBSTRATE_ROOT="${AMBIENT_B}" \
  SHIM_TRACE_LOG="${AMBIENT_B}/trace.jsonl" \
  "${UNINSTALL_SCRIPT}" --prefix "${SELECTED_A}" --bin "${SUBSTRATE_BIN}" \
    --version-label dev >"${WORK_ROOT}/uninstall.out" 2>"${WORK_ROOT}/uninstall.err"
UNINSTALL_RC=$?
set -e
chmod 0700 "${AMBIENT_B}"
[[ "${UNINSTALL_RC}" -eq 0 ]] || {
  sed -n '1,160p' "${WORK_ROOT}/uninstall.err" >&2
  fail "dev uninstall failed with exit ${UNINSTALL_RC}"
}
for removed in dev-shim-env.sh versions/dev config.yaml manager_env.sh manager_init.sh trace.jsonl; do
  [[ ! -e "${SELECTED_A}/${removed}" ]] \
    || fail "dev uninstall did not select A for ${removed}"
done
assert_b_untouched
[[ ! -e "${AMBIENT_B}/trace.jsonl" ]] || fail "dev propagation wrote trace output under B"

printf '[%s] PASS: equal IH, custom A, repeat install, projections, trace, and uninstall symmetry\n' \
  "${SCRIPT_NAME}"
