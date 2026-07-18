#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="prefix-propagation-r2-2"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_WRAPPER="${REPO_ROOT}/scripts/substrate/install.sh"
INSTALL_CHILD="${REPO_ROOT}/scripts/substrate/install-substrate.sh"
UNINSTALL_WRAPPER="${REPO_ROOT}/scripts/substrate/uninstall.sh"
UNINSTALL_CHILD="${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh"
CURRENT_UID="$(id -u)"
FIXTURE_PARENT="${XDG_RUNTIME_DIR:-/run/user/${CURRENT_UID}}"
ACCOUNT_HOME="$(getent passwd "${CURRENT_UID}" | awk -F: 'NR == 1 { print $6 }')"

fail() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

if [[ "$(uname -s)" != "Linux" ]]; then
  printf '[%s] SKIP: Linux-only Unix release context proof\n' "${SCRIPT_NAME}"
  exit 0
fi
if [[ "${CURRENT_UID}" -eq 0 ]]; then
  fail "the R2-2 release context matrix must run as an unprivileged Unix principal"
fi
if [[ ! -d "${FIXTURE_PARENT}" || ! -O "${FIXTURE_PARENT}" ]]; then
  fail "secure audit-owned fixture parent is unavailable: ${FIXTURE_PARENT}"
fi
if [[ -z "${ACCOUNT_HOME}" || ! -d "${ACCOUNT_HOME}" ]]; then
  fail "current account-database home is unavailable"
fi

WORK_ROOT="$(mktemp -d "${FIXTURE_PARENT%/}/substrate-r2-2-release.XXXXXX")"
chmod 0700 "${WORK_ROOT}"
cleanup() {
  chmod -R u+rwX "${WORK_ROOT}" 2>/dev/null || true
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

SELECTED_A="${WORK_ROOT}/selected-a"
SELECTED_C="${WORK_ROOT}/selected-c"
AMBIENT_B="${WORK_ROOT}/ambient-b"
mkdir "${AMBIENT_B}"
chmod 0700 "${AMBIENT_B}"
printf 'ambient-b-sentinel\n' > "${AMBIENT_B}/sentinel"

source_wrapper_prelude() {
  local script="$1"
  # shellcheck disable=SC1090
  source <(awk '/^ASSET_TMP_DIR=/ { exit } { print }' "${script}")
}

source_install_child_prelude() {
  # shellcheck disable=SC1090
  source <(awk '/^print_usage\(\)/ { exit } { print }' "${INSTALL_CHILD}")
}

source_uninstall_child_prelude() {
  # shellcheck disable=SC1090
  source <(awk '/^PATH_SNIPPET_START=/ { exit } { print }' "${UNINSTALL_CHILD}")
}

wrapper_context() {
  local script="$1"
  local declared="$2"
  local prefix="$3"
  HOME="${AMBIENT_B}" \
    SUBSTRATE_HOME="${AMBIENT_B}" \
    SUBSTRATE_ROOT="${AMBIENT_B}" \
    bash -s -- "${script}" "${declared}" "${prefix}" <<'BASH'
set -euo pipefail
script="$1"
declared="$2"
prefix="$3"
source_wrapper_prelude() {
  # shellcheck disable=SC1090
  source <(awk '/^ASSET_TMP_DIR=/ { exit } { print }' "$1")
}
source_wrapper_prelude "${script}"
resolve_public_install_bootstrap_context "${declared}" "${prefix}"
printf '%s\0%s\0%s\0%s\0%s\0%s\0' \
  "${PREFIX}" \
  "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
  "${INSTALL_BOOTSTRAP_COMMITMENT}" \
  "${INSTALL_BOOTSTRAP_ACCOUNT}" \
  "${INSTALL_BOOTSTRAP_UID}" \
  "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}"
BASH
}

child_context() {
  local script="$1"
  local declared="$2"
  local prefix="$3"
  local carrier="$4"
  local prelude_marker="$5"
  local carrier_declared="${6:-}"
  if [[ -z "${carrier_declared}" ]]; then
    if [[ -n "${carrier}" ]]; then
      carrier_declared=1
    else
      carrier_declared=0
    fi
  fi
  local projected_prefix="${AMBIENT_B}"
  local projected_commitment="ambient"
  local projected_account="ambient"
  local projected_uid="999999"
  local projected_carrier="${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}"
  if [[ -n "${carrier}" ]]; then
    projected_prefix="${CHILD_PROJECT_PREFIX}"
    projected_commitment="${CHILD_PROJECT_COMMITMENT}"
    projected_account="${CHILD_PROJECT_ACCOUNT}"
    projected_uid="${CHILD_PROJECT_UID}"
    projected_carrier="${carrier}"
  fi
  HOME="${AMBIENT_B}" \
    SUBSTRATE_HOME="${projected_prefix}" \
    SUBSTRATE_ROOT="${projected_prefix}" \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${projected_commitment}" \
    SUBSTRATE_INSTALL_PRIMARY_USER="${projected_account}" \
    SUBSTRATE_INSTALL_PRIMARY_UID="${projected_uid}" \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${projected_carrier}" \
    bash -s -- "${script}" "${declared}" "${prefix}" "${carrier}" "${prelude_marker}" "${carrier_declared}" <<'BASH'
set -euo pipefail
script="$1"
declared="$2"
prefix="$3"
carrier="$4"
prelude_marker="$5"
carrier_declared="$6"
# shellcheck disable=SC1090
source <(awk -v marker="${prelude_marker}" 'index($0, marker) == 1 { exit } { print }' "${script}")
resolve_install_bootstrap_context "${declared}" "${prefix}" "${carrier}" "${carrier_declared}"
printf '%s\0%s\0%s\0%s\0%s\0%s\0' \
  "${PREFIX}" \
  "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
  "${INSTALL_BOOTSTRAP_COMMITMENT}" \
  "${INSTALL_BOOTSTRAP_ACCOUNT}" \
  "${INSTALL_BOOTSTRAP_UID}" \
  "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}"
BASH
}

read_context() {
  local producer="$1"
  shift
  local fd
  exec {fd}< <("${producer}" "$@")
  IFS= read -r -d '' CTX_PREFIX <&"${fd}" || fail "context prefix missing"
  IFS= read -r -d '' CTX_CARRIER <&"${fd}" || fail "context carrier missing"
  IFS= read -r -d '' CTX_COMMITMENT <&"${fd}" || fail "context commitment missing"
  IFS= read -r -d '' CTX_ACCOUNT <&"${fd}" || fail "context account missing"
  IFS= read -r -d '' CTX_UID <&"${fd}" || fail "context uid missing"
  IFS= read -r -d '' CTX_ACCOUNT_HOME <&"${fd}" || fail "context account home missing"
  exec {fd}<&-
}

read_context wrapper_context "${INSTALL_WRAPPER}" 1 "${SELECTED_A}///"
INSTALL_WRAPPER_CARRIER="${CTX_CARRIER}"
INSTALL_WRAPPER_COMMITMENT="${CTX_COMMITMENT}"
CHILD_PROJECT_PREFIX="${CTX_PREFIX}"
CHILD_PROJECT_COMMITMENT="${CTX_COMMITMENT}"
CHILD_PROJECT_ACCOUNT="${CTX_ACCOUNT}"
CHILD_PROJECT_UID="${CTX_UID}"
[[ "${CTX_PREFIX}" == "${SELECTED_A}" ]] || fail "install wrapper did not normalize A"

read_context child_context "${INSTALL_CHILD}" 1 "${SELECTED_A}" "${INSTALL_WRAPPER_CARRIER}" 'print_usage()'
[[ "${CTX_CARRIER}" == "${INSTALL_WRAPPER_CARRIER}" ]] || fail "install child changed the wrapper carrier"

read_context wrapper_context "${UNINSTALL_WRAPPER}" 1 "${SELECTED_A}"
[[ "${CTX_CARRIER}" == "${INSTALL_WRAPPER_CARRIER}" ]] || fail "install/uninstall wrappers constructed different IH"
[[ "${CTX_COMMITMENT}" == "${INSTALL_WRAPPER_COMMITMENT}" ]] || fail "install/uninstall wrapper commitments differ"

read_context child_context "${UNINSTALL_CHILD}" 1 "${SELECTED_A}" "${CTX_CARRIER}" 'PATH_SNIPPET_START='
[[ "${CTX_CARRIER}" == "${INSTALL_WRAPPER_CARRIER}" ]] || fail "uninstall child changed the wrapper carrier"

run_wrapper_capture() {
  local wrapper="$1"
  local child_name="$2"
  local capture_path="$3"
  local fixture_dir="${WORK_ROOT}/wrapper-${child_name}"
  mkdir -p "${fixture_dir}/loader"
  cp "${wrapper}" "${fixture_dir}/wrapper.sh"
  chmod +x "${fixture_dir}/wrapper.sh"
  cat > "${fixture_dir}/${child_name}" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
{
  printf '%s\n' \
    "${SUBSTRATE_HOME:-}" \
    "${SUBSTRATE_ROOT:-}" \
    "${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT:-}" \
    "${SUBSTRATE_INSTALL_PRIMARY_USER:-}" \
    "${SUBSTRATE_INSTALL_PRIMARY_UID:-}" \
    "${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}"
  printf 'argv=%s\n' "$*"
} > "${WRAPPER_CAPTURE:?}"
STUB
  chmod +x "${fixture_dir}/${child_name}"
  cat > "${fixture_dir}/loader/bash_loading_animations.sh" <<'LOADER'
BLA_braille_fill_bar=()
BLA::start_loading_animation() { :; }
BLA::stop_loading_animation() { :; }
LOADER

  HOME="${AMBIENT_B}" \
    SUBSTRATE_HOME="${AMBIENT_B}" \
    SUBSTRATE_ROOT="${AMBIENT_B}" \
    WRAPPER_CAPTURE="${capture_path}" \
    "${fixture_dir}/wrapper.sh" --prefix "${SELECTED_A}///" >/dev/null
}

INSTALL_WRAPPER_CAPTURE="${WORK_ROOT}/install-wrapper-capture"
UNINSTALL_WRAPPER_CAPTURE="${WORK_ROOT}/uninstall-wrapper-capture"
run_wrapper_capture "${INSTALL_WRAPPER}" install-substrate.sh "${INSTALL_WRAPPER_CAPTURE}"
run_wrapper_capture "${UNINSTALL_WRAPPER}" uninstall-substrate.sh "${UNINSTALL_WRAPPER_CAPTURE}"
mapfile -t install_capture < "${INSTALL_WRAPPER_CAPTURE}"
mapfile -t uninstall_capture < "${UNINSTALL_WRAPPER_CAPTURE}"
for capture_name in install_capture uninstall_capture; do
  declare -n capture="${capture_name}"
  [[ "${capture[0]}" == "${SELECTED_A}" && "${capture[1]}" == "${SELECTED_A}" ]] \
    || fail "${capture_name} did not export exact H=R=A"
  [[ "${capture[2]}" == "${INSTALL_WRAPPER_COMMITMENT}" ]] \
    || fail "${capture_name} exported a different commitment"
  [[ "${capture[5]}" == "${INSTALL_WRAPPER_CARRIER}" ]] \
    || fail "${capture_name} exported a different carrier"
  [[ "${capture[6]}" == *"--install-bootstrap-context-v1 ${INSTALL_WRAPPER_CARRIER}"* ]] \
    || fail "${capture_name} omitted the explicit child carrier argv"
  unset -n capture
done

read_context wrapper_context "${INSTALL_WRAPPER}" 0 ""
[[ "${CTX_PREFIX}" == "${ACCOUNT_HOME%/}/.substrate" ]] \
  || fail "release default did not derive from the account database"
[[ "${CTX_ACCOUNT_HOME}" == "${ACCOUNT_HOME%/}" ]] || fail "account home projection mismatch"

forged_carrier="${INSTALL_WRAPPER_CARRIER%?}A"
if child_context "${INSTALL_CHILD}" 1 "${SELECTED_A}" "${forged_carrier}" 'print_usage()' >/dev/null 2>&1; then
  fail "install child accepted a forged carrier"
fi
if child_context "${UNINSTALL_CHILD}" 1 "${SELECTED_A}" "${forged_carrier}" 'PATH_SNIPPET_START=' >/dev/null 2>&1; then
  fail "uninstall child accepted a forged carrier"
fi
if child_context "${INSTALL_CHILD}" 1 "${SELECTED_A}" "" 'print_usage()' 1 >/dev/null 2>&1; then
  fail "install child treated an explicit empty carrier as a public invocation"
fi
if child_context "${UNINSTALL_CHILD}" 1 "${SELECTED_A}" "" 'PATH_SNIPPET_START=' 1 >/dev/null 2>&1; then
  fail "uninstall child treated an explicit empty carrier as a public invocation"
fi
for child in "${INSTALL_CHILD}" "${UNINSTALL_CHILD}"; do
  if "${child}" --prefix "${SELECTED_A}" \
    --install-bootstrap-context-v1 "" >/dev/null 2>&1; then
    fail "$(basename "${child}") accepted an explicit empty internal carrier option"
  fi
  if "${child}" --prefix "${SELECTED_A}" \
    --install-bootstrap-context-v1 "${INSTALL_WRAPPER_CARRIER}" \
    --install-bootstrap-context-v1 "${INSTALL_WRAPPER_CARRIER}" \
    --dry-run >/dev/null 2>&1; then
    fail "$(basename "${child}") accepted duplicate internal carrier options"
  fi
done

SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_WRAPPER_CARRIER}" \
  read_context child_context "${INSTALL_CHILD}" 1 "${SELECTED_C}" "" 'print_usage()'
[[ "${CTX_PREFIX}" == "${SELECTED_C}" ]] || fail "environment-only carrier selected internal mode"
[[ "${CTX_CARRIER}" != "${INSTALL_WRAPPER_CARRIER}" ]] || fail "environment-only carrier replaced public construction"

verify_release_projections() (
  set -euo pipefail
  # shellcheck disable=SC1090
  source "${INSTALL_CHILD}"

  resolve_install_bootstrap_context 1 "${SELECTED_A}" "" 0
  INSTALL_BOOTSTRAP_ACCOUNT_HOME="${WORK_ROOT}/account-home"
  mkdir -p "${PREFIX}" "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}"
  initialize_metadata_paths
  DRY_RUN=0

  local version_dir="${PREFIX}/versions/test"
  mkdir -p "${version_dir}/config"
  printf 'hooks: selected-a\n' > "${version_dir}/config/manager_hooks.yaml"
  printf 'version: 1\nitems: []\n' > "${version_dir}/config/world-deps.yaml"
  ensure_version_config_present "${version_dir}"
  cmp "${version_dir}/config/manager_hooks.yaml" "${PREFIX}/manager_hooks.yaml" \
    || fail "release manager manifest projection did not derive from A"

  write_env_sh_script 1
  write_manager_env_script 1
  bash -n "${PREFIX}/env.sh" "${PREFIX}/manager_env.sh"
  for projection in \
    SUBSTRATE_HOME \
    SUBSTRATE_ROOT \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT \
    SUBSTRATE_INSTALL_PRIMARY_USER \
    SUBSTRATE_INSTALL_PRIMARY_UID \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1
  do
    grep -q "^export ${projection}=" "${PREFIX}/env.sh" \
      || fail "release env projection omitted ${projection}"
  done
  if grep -Eq '\$\{?HOME\}?' "${PREFIX}/manager_env.sh"; then
    fail "release manager projection retained ambient HOME authority"
  fi
  grep -Fq "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}/.substrate_bashenv" "${PREFIX}/manager_env.sh" \
    || fail "legacy bash environment did not target the intended account home"

  local manager_output
  manager_output="$(env -i PATH="${PATH}" HOME="${AMBIENT_B}" bash -c '
    source "$1"
    printf "%s\n%s\n%s\n" "$SUBSTRATE_HOME" "$SUBSTRATE_ROOT" "$SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT"
  ' _ "${PREFIX}/manager_env.sh")"
  [[ "${manager_output}" == "${SELECTED_A}"$'\n'"${SELECTED_A}"$'\n'"${INSTALL_BOOTSTRAP_COMMITMENT}" ]] \
    || fail "release manager projection did not reconstruct exact checked projections from A"
  local relocated_manager="${WORK_ROOT}/relocated-manager"
  mkdir -p "${relocated_manager}"
  cp "${PREFIX}/manager_env.sh" "${PREFIX}/env.sh" "${relocated_manager}/"
  if env -i PATH="${PATH}" HOME="${AMBIENT_B}" \
    bash -c 'source "$1"' _ "${relocated_manager}/manager_env.sh" >/dev/null 2>&1; then
    fail "relocated manager projection was not bound to its committed self-directory A"
  fi
  if env -i PATH="${PATH}" HOME="${AMBIENT_B}" SUBSTRATE_HOME="${AMBIENT_B}" \
    bash -c 'source "$1"' _ "${PREFIX}/manager_env.sh" >/dev/null 2>&1; then
    fail "conflicting ambient B retargeted the release manager projection"
  fi
  cp "${PREFIX}/env.sh" "${PREFIX}/env.sh.saved"
  grep -v '^export SUBSTRATE_ROOT=' "${PREFIX}/env.sh.saved" > "${PREFIX}/env.sh"
  printf 'export SUBSTRATE_ROOT=%q\n' "${AMBIENT_B}" >> "${PREFIX}/env.sh"
  env -i PATH="${PATH}" HOME="${AMBIENT_B}" bash -c '
    set +e
    source "$1" >/dev/null 2>&1
    first_status=$?
    set -e
    [[ "${first_status}" -ne 0 ]]
    [[ -z "${SUBSTRATE_MANAGER_ENV_ACTIVE:-}" ]]
    cp "$2" "$3"
    set +e
    source "$1" >/dev/null 2>&1
    second_status=$?
    set -e
    [[ "${second_status}" -ne 0 ]]
    [[ -z "${SUBSTRATE_MANAGER_ENV_ACTIVE:-}" ]]
    unset SUBSTRATE_HOME SUBSTRATE_ROOT \
      SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT \
      SUBSTRATE_INSTALL_PRIMARY_USER \
      SUBSTRATE_INSTALL_PRIMARY_UID \
      SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1
    source "$1"
    [[ "${SUBSTRATE_HOME}" == "$4" && "${SUBSTRATE_ROOT}" == "$4" ]]
  ' _ \
    "${PREFIX}/manager_env.sh" \
    "${PREFIX}/env.sh.saved" \
    "${PREFIX}/env.sh" \
    "${SELECTED_A}" \
    || fail "failed manager source poisoned or bypassed a later exact projection"
  rm -f "${PREFIX}/env.sh.saved"

  cp "${PREFIX}/env.sh" "${PREFIX}/env.sh.saved"
  printf 'false\n' >> "${PREFIX}/env.sh"
  env -i PATH="${PATH}" HOME="${AMBIENT_B}" bash -c '
    set +e
    source "$1" >/dev/null 2>&1
    first_status=$?
    set -e
    [[ "${first_status}" -ne 0 ]]
    [[ -z "${SUBSTRATE_MANAGER_ENV_ACTIVE:-}" ]]
    cp "$2" "$3"
    source "$1"
    [[ "${SUBSTRATE_HOME}" == "$4" && "${SUBSTRATE_ROOT}" == "$4" ]]
  ' _ \
    "${PREFIX}/manager_env.sh" \
    "${PREFIX}/env.sh.saved" \
    "${PREFIX}/env.sh" \
    "${SELECTED_A}" \
    || fail "nonzero exact projection poisoned or bypassed a later manager source"
  rm -f "${PREFIX}/env.sh.saved"

  local path_home="${WORK_ROOT}/path-home"
  mkdir -p "${path_home}"
  SHELL=/bin/bash update_shell_path "${PREFIX}/bin" "${path_home}"
  grep -Fq "${PREFIX}/bin" "${path_home}/.bashrc" \
    || fail "PATH upsert did not target the intended account home"
  [[ ! -e "${AMBIENT_B}/.bashrc" ]] || fail "PATH upsert mutated ambient B"

  local argv_record="${WORK_ROOT}/shim-argv"
  local substrate_stub="${WORK_ROOT}/substrate-stub"
  cat > "${substrate_stub}" <<'STUB'
#!/usr/bin/env bash
printf '%s\n' "$@" > "${ARGV_RECORD}"
STUB
  chmod +x "${substrate_stub}"
  ARGV_RECORD="${argv_record}" deploy_shims "${substrate_stub}"
  mapfile -t shim_argv < "${argv_record}"
  [[ "${shim_argv[0]}" == "--install-bootstrap-context-v1" ]] \
    || fail "shim leaf omitted the explicit carrier option"
  [[ "${shim_argv[1]}" == "${INSTALL_BOOTSTRAP_CONTEXT_V1}" ]] \
    || fail "shim leaf received a different carrier"
  [[ "${shim_argv[2]}" == "--shim-deploy" ]] || fail "shim leaf action changed"

  : > "${argv_record}"
  ARGV_RECORD="${argv_record}" run_world_checks "${substrate_stub}"
  mapfile -t doctor_argv < "${argv_record}"
  [[ "${doctor_argv[0]}" == "--install-bootstrap-context-v1" ]] \
    || fail "doctor leaf omitted the explicit carrier option"
  [[ "${doctor_argv[1]}" == "${INSTALL_BOOTSTRAP_CONTEXT_V1}" ]] \
    || fail "doctor leaf received a different carrier"
  [[ "${doctor_argv[2]}" == "world" && "${doctor_argv[3]}" == "doctor" && "${doctor_argv[4]}" == "--json" ]] \
    || fail "doctor leaf action changed"
)

verify_release_projections

[[ "$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -printf '%f\n')" == "sentinel" ]] \
  || fail "release context construction mutated ambient B"
[[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
  || fail "ambient B sentinel changed"

printf '[%s] PASS: release IH symmetry, checked A projections, intended-home PATH, carrier leaves, and ambient-B proof\n' "${SCRIPT_NAME}"
