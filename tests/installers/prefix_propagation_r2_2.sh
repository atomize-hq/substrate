#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="prefix-propagation-r2-2"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_WRAPPER="${REPO_ROOT}/scripts/substrate/install.sh"
INSTALL_CHILD="${REPO_ROOT}/scripts/substrate/install-substrate.sh"
UNINSTALL_WRAPPER="${REPO_ROOT}/scripts/substrate/uninstall.sh"
UNINSTALL_CHILD="${REPO_ROOT}/scripts/substrate/uninstall-substrate.sh"
CURRENT_UID="$(id -u)"
CURRENT_ACCOUNT="$(id -un)"
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

release_entry_stdout="${WORK_ROOT}/release-entry.stdout"
release_entry_stderr="${WORK_ROOT}/release-entry.stderr"
set +e
HOME="${AMBIENT_B}" \
  SUBSTRATE_HOME="${SELECTED_A}" \
  SUBSTRATE_ROOT="${SELECTED_A}" \
  SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_WRAPPER_COMMITMENT}" \
  SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
  SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
  SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_WRAPPER_CARRIER}" \
  bash -x "${INSTALL_CHILD}" \
    --prefix "${SELECTED_A}" \
    --install-bootstrap-context-v1 "${INSTALL_WRAPPER_CARRIER}" \
    --help >"${release_entry_stdout}" 2>"${release_entry_stderr}"
release_entry_status=$?
set -e
[[ "${release_entry_status}" -eq 0 ]] || fail "validated release internal help failed"
if grep -Fq -- "${INSTALL_WRAPPER_CARRIER}" "${release_entry_stdout}" "${release_entry_stderr}"; then
  fail "release full-entry inherited xtrace disclosed the carrier"
fi
grep -Fq -- "Usage:" "${release_entry_stdout}" \
  || fail "validated release internal help did not print usage"
grep -Eq '^\+ .*print_usage' "${release_entry_stderr}" \
  || fail "release internal help did not resume nonsensitive tracing after validation"

set +e
HOME="${AMBIENT_B}" \
  SUBSTRATE_HOME="${SELECTED_A}" \
  SUBSTRATE_ROOT="${SELECTED_A}" \
  SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_WRAPPER_COMMITMENT}" \
  SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
  SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
  SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="not!base64" \
  bash "${INSTALL_CHILD}" \
    --prefix "${SELECTED_A}" \
    --install-bootstrap-context-v1 "not!base64" \
    --help >"${release_entry_stdout}" 2>"${release_entry_stderr}"
release_malformed_help_status=$?
set -e
[[ "${release_malformed_help_status}" -ne 0 ]] \
  || fail "release malformed internal carrier fell through to public help"

for uninstall_order in carrier-first help-first; do
  uninstall_help_args=(--prefix "${SELECTED_A}")
  if [[ "${uninstall_order}" == "carrier-first" ]]; then
    uninstall_help_args+=(--install-bootstrap-context-v1 "${INSTALL_WRAPPER_CARRIER}" --help)
  else
    uninstall_help_args+=(--help --install-bootstrap-context-v1 "${INSTALL_WRAPPER_CARRIER}")
  fi
  set +e
  HOME="${AMBIENT_B}" bash -x "${UNINSTALL_CHILD}" "${uninstall_help_args[@]}" \
    >"${release_entry_stdout}" 2>"${release_entry_stderr}"
  uninstall_help_status=$?
  set -e
  [[ "${uninstall_help_status}" -eq 0 ]] \
    || fail "validated uninstall internal help failed (${uninstall_order})"
  if grep -Fq -- "${INSTALL_WRAPPER_CARRIER}" "${release_entry_stdout}" "${release_entry_stderr}"; then
    fail "uninstall internal help disclosed the carrier (${uninstall_order})"
  fi
  grep -Fq -- "Usage:" "${release_entry_stdout}" \
    || fail "validated uninstall internal help did not print usage (${uninstall_order})"

  malformed_help_args=(--prefix "${SELECTED_A}")
  if [[ "${uninstall_order}" == "carrier-first" ]]; then
    malformed_help_args+=(--install-bootstrap-context-v1 "not!base64" --help)
  else
    malformed_help_args+=(--help --install-bootstrap-context-v1 "not!base64")
  fi
  if HOME="${AMBIENT_B}" "${UNINSTALL_CHILD}" "${malformed_help_args[@]}" \
    >"${release_entry_stdout}" 2>"${release_entry_stderr}"; then
    fail "uninstall malformed carrier fell through to public help (${uninstall_order})"
  fi
done

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
for wrapper_kind in install uninstall; do
  if [[ "${wrapper_kind}" == "install" ]]; then
    fixture_dir="${WORK_ROOT}/wrapper-install-substrate.sh"
  else
    fixture_dir="${WORK_ROOT}/wrapper-uninstall-substrate.sh"
  fi
  wrapper_trace="${WORK_ROOT}/${wrapper_kind}-wrapper.trace"
  HOME="${AMBIENT_B}" \
    SUBSTRATE_HOME="${AMBIENT_B}" \
    SUBSTRATE_ROOT="${AMBIENT_B}" \
    WRAPPER_CAPTURE="${WORK_ROOT}/${wrapper_kind}-wrapper-xtrace-capture" \
    bash -x "${fixture_dir}/wrapper.sh" --prefix "${SELECTED_A}///" \
      >"${WORK_ROOT}/${wrapper_kind}-wrapper.stdout" 2>"${wrapper_trace}"
  if grep -Fq -- "${INSTALL_WRAPPER_CARRIER}" "${wrapper_trace}"; then
    fail "${wrapper_kind} wrapper disclosed the carrier under inherited xtrace"
  fi
  grep -Eq '^\+ (stop_loader|printf)' "${wrapper_trace}" \
    || fail "${wrapper_kind} wrapper did not restore inherited xtrace after child delegation"
done
# shellcheck disable=SC2034 # referenced dynamically through the capture nameref below
mapfile -t install_capture < "${INSTALL_WRAPPER_CAPTURE}"
# shellcheck disable=SC2034 # referenced dynamically through the capture nameref below
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

  local release_projection_trace
  release_projection_trace="$({
    set -x
    write_env_sh_script 1
    write_manager_env_script 1
    { set +x; } 2>/dev/null
  } 2>&1)"
  if [[ "${release_projection_trace}" == *"${INSTALL_BOOTSTRAP_CONTEXT_V1}"* ]]; then
    fail "release generated projections disclosed the authenticated carrier under inherited xtrace"
  fi
  [[ "${release_projection_trace}" == *"write_env_sh_script 1"* \
    && "${release_projection_trace}" == *"write_manager_env_script 1"* ]] \
    || fail "release generated projections did not resume nonsensitive tracing"
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
  local release_shim_trace
  release_shim_trace="$({
    set -x
    ARGV_RECORD="${argv_record}" deploy_shims "${substrate_stub}"
    { set +x; } 2>/dev/null
  } 2>&1)"
  if [[ "${release_shim_trace}" == *"${INSTALL_BOOTSTRAP_CONTEXT_V1}"* ]]; then
    fail "release shim leaf disclosed the authenticated carrier under inherited xtrace"
  fi
  [[ "${release_shim_trace}" == *"deploy_shims ${substrate_stub}"* ]] \
    || fail "release shim leaf did not resume nonsensitive tracing"
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

  local release_sudo_capture="${WORK_ROOT}/release-sudo-capture"
  local release_sudo_stub="${WORK_ROOT}/release-sudo"
  local release_malicious_bin="${WORK_ROOT}/release-malicious-bin"
  local release_malicious_systemctl="${release_malicious_bin}/systemctl"
  local release_malicious_env="${release_malicious_bin}/env"
  local release_malicious_true="${release_malicious_bin}/true"
  local release_malicious_pacman="${release_malicious_bin}/pacman"
  mkdir -p "${release_malicious_bin}"
  cat > "${release_sudo_stub}" <<'STUB'
#!/usr/bin/env bash
{
  printf '%s\n' '--call--'
  printf '%s\n' "$@"
} >> "${SUDO_CAPTURE:?}"
exit 0
STUB
  printf '#!/usr/bin/env bash\nexit 97\n' > "${release_malicious_systemctl}"
  printf '#!/usr/bin/env bash\nexit 98\n' > "${release_malicious_env}"
  printf '#!/usr/bin/env bash\nexit 99\n' > "${release_malicious_true}"
  printf '#!/usr/bin/env bash\nexit 96\n' > "${release_malicious_pacman}"
  chmod +x "${release_sudo_stub}" "${release_malicious_systemctl}" \
    "${release_malicious_env}" "${release_malicious_true}" "${release_malicious_pacman}"
  # shellcheck disable=SC2034 # consumed by the sourced run_with_sudo implementation
  SUDO_CMD=("${release_sudo_stub}")
  # shellcheck disable=SC2034 # consumed by the sourced run_with_sudo implementation
  SUDO_INITIALIZED=1
  : > "${release_sudo_capture}"
  local release_fixed_systemctl
  local release_fixed_env
  local release_fixed_true
  local release_fixed_pacman
  release_fixed_systemctl="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P systemctl)"
  release_fixed_env="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P env)"
  release_fixed_true="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P true)"
  release_fixed_pacman="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P pacman)"
  # shellcheck disable=SC2218 # sourced production definition precedes the test-local override below
  PATH="${release_malicious_bin}:${PATH}" SUDO_CAPTURE="${release_sudo_capture}" run_with_sudo \
    systemctl \
    --selected-prefix "${PREFIX}" \
    --account "${INSTALL_BOOTSTRAP_ACCOUNT}" \
    --uid "${INSTALL_BOOTSTRAP_UID}"
  for expected in \
    -- "${release_fixed_env}" -i \
    "PATH=${PRIVILEGED_TOOL_PATH}" \
    HOME=/root USER=root LOGNAME=root \
    "${release_fixed_systemctl}" \
    --selected-prefix "${PREFIX}" \
    --account "${INSTALL_BOOTSTRAP_ACCOUNT}" \
    --uid "${INSTALL_BOOTSTRAP_UID}"
  do
    grep -Fxq -- "${expected}" "${release_sudo_capture}" \
      || fail "release sudo boundary omitted '${expected}'"
  done
  if grep -Eq -- '(^-E$|SUBSTRATE_(HOME|ROOT|INSTALL_BOOTSTRAP_CONTEXT_V1)=)' "${release_sudo_capture}"; then
    fail "release arbitrary privileged tool inherited installation authority"
  fi
  if grep -Fq -- "${INSTALL_BOOTSTRAP_CONTEXT_V1}" "${release_sudo_capture}"; then
    fail "release arbitrary privileged tool received the authenticated carrier"
  fi
  if grep -Fq -- "${release_malicious_systemctl}" "${release_sudo_capture}"; then
    fail "release sudo boundary selected a privileged tool from ambient PATH"
  fi
  if grep -Fq -- "${release_malicious_env}" "${release_sudo_capture}"; then
    fail "release sudo boundary selected env from ambient PATH"
  fi
  grep -Fxq -- "${release_fixed_true}" "${release_sudo_capture}" \
    || fail "release sudo probe did not use fixed-path true"
  if grep -Fq -- "${release_malicious_true}" "${release_sudo_capture}"; then
    fail "release sudo probe selected true from ambient PATH"
  fi
  local release_trace_output
  release_trace_output="$({
    set -x
    PATH="${release_malicious_bin}:${PATH}" SUDO_CAPTURE="${release_sudo_capture}" \
      run_with_sudo systemctl --selected-prefix "${PREFIX}"
    { set +x; } 2>/dev/null
  } 2>&1)"
  if [[ "${release_trace_output}" == *"${INSTALL_BOOTSTRAP_CONTEXT_V1}"* ]]; then
    fail "release inherited xtrace disclosed the authenticated carrier"
  fi
  [[ "${release_trace_output}" == *"run_with_sudo systemctl"* ]] \
    || fail "release inherited xtrace did not resume for nonsensitive dispatch"

  : > "${release_sudo_capture}"
  # shellcheck disable=SC2034 # consumed by sourced install_packages
  PKG_MANAGER=pacman
  # shellcheck disable=SC2034 # consumed by sourced install_packages
  DRY_RUN=0
  PATH="${release_malicious_bin}:${PATH}" SUDO_CAPTURE="${release_sudo_capture}" \
    install_packages review-package
  [[ "$(grep -Fxc -- "${release_fixed_pacman}" "${release_sudo_capture}")" -eq 1 ]] \
    || fail "release package installation bypassed fixed-path privileged dispatch"
  [[ "$(grep -Fxc -- "${release_fixed_env}" "${release_sudo_capture}")" -ge 1 ]] \
    || fail "release package installation bypassed the fixed environment scrubber"
  if grep -Fq -- "${release_malicious_pacman}" "${release_sudo_capture}"; then
    fail "release package installation selected pacman from ambient PATH"
  fi
  local release_sudo_lines_before
  local release_sudo_lines_after
  release_sudo_lines_before="$(wc -l < "${release_sudo_capture}")"
  set +e
  (
    # shellcheck disable=SC2034 # consumed by sourced run_privileged projection validation
    SUBSTRATE_ROOT="${AMBIENT_B}"
    SUDO_CAPTURE="${release_sudo_capture}" run_with_sudo \
      true --selected-prefix "${PREFIX}"
  ) >/dev/null 2>&1
  local release_tamper_status=$?
  set -e
  release_sudo_lines_after="$(wc -l < "${release_sudo_capture}")"
  [[ "${release_tamper_status}" -ne 0 ]] \
    || fail "release sudo boundary accepted a conflicting checked projection"
  [[ "${release_sudo_lines_after}" == "${release_sudo_lines_before}" ]] \
    || fail "release sudo boundary crossed after projection validation failed"

  local service_fixture="${WORK_ROOT}/release-service"
  local service_version="${service_fixture}/version"
  local service_root="${service_fixture}/root"
  local service_tmp="${service_fixture}/tmp"
  local sudo_log="${service_fixture}/sudo.log"
  mkdir -p \
    "${service_version}/bin" \
    "${service_version}/scripts/linux" \
    "${service_root}" \
    "${service_tmp}"
  for binary in world-service substrate-gateway; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "${service_version}/bin/${binary}"
    chmod +x "${service_version}/bin/${binary}"
  done
  cp "${REPO_ROOT}/scripts/linux/substrate-apply-socket-acl.sh" \
    "${service_version}/scripts/linux/substrate-apply-socket-acl.sh"

  # shellcheck disable=SC2329 # invoked indirectly by the sourced provision_linux_world implementation
  run_with_sudo() {
    local tool="$1"
    shift
    printf '%s %s\n' "${tool}" "$*" >> "${sudo_log}"
    case "${tool}" in
      install)
        if [[ "${1:-}" == -Dm* ]]; then
          local source_path="$2"
          local destination="$3"
          mkdir -p "$(dirname "${service_root}${destination}")"
          cp "${source_path}" "${service_root}${destination}"
        elif [[ "${1:-}" == "-d" ]]; then
          local destination="${!#}"
          mkdir -p "${service_root}${destination}"
        else
          fail "unexpected release service install tuple: $*"
        fi
        ;;
      systemctl|rm)
        ;;
      /usr/libexec/substrate/substrate-apply-socket-acl)
        case "$*" in
          "--socket /run/substrate.sock substrate"|\
          "--directory-traverse /var/lib/substrate substrate"|\
          "--tree-readonly /var/lib/substrate/world-deps substrate")
            ;;
          *)
            fail "release service emitted a noncanonical ACL tuple: $*"
            ;;
        esac
        ;;
      *)
        fail "unexpected release privileged tool: ${tool}"
        ;;
    esac
  }

  TMPDIR="${service_tmp}"
  # shellcheck disable=SC2034 # consumed by the sourced provision_linux_world implementation
  ENABLE_WORLD_NETFILTER=0
  provision_linux_world "${service_version}"

  local service_unit="${service_root}/etc/systemd/system/substrate-world-service.service"
  local socket_dropin="${service_root}/etc/systemd/system/substrate-world-service.socket.d/20-substrate-group-acl.conf"
  [[ -f "${service_unit}" ]] || fail "release service unit was not generated"
  [[ -f "${socket_dropin}" ]] || fail "release socket ACL drop-in was not generated"
  grep -Fq "Environment=\"SUBSTRATE_HOME=${SELECTED_A}\"" "${service_unit}" \
    || fail "release service H did not derive from A"
  grep -Fq "Environment=\"SUBSTRATE_ROOT=${SELECTED_A}\"" "${service_unit}" \
    || fail "release service R did not derive from A"
  grep -Fq "Environment=\"SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${INSTALL_BOOTSTRAP_COMMITMENT}\"" "${service_unit}" \
    || fail "release service commitment projection changed"
  grep -Fq "Environment=\"SUBSTRATE_INSTALL_PRIMARY_USER=${INSTALL_BOOTSTRAP_ACCOUNT}\"" "${service_unit}" \
    || fail "release service intended account projection changed"
  grep -Fq "Environment=\"SUBSTRATE_INSTALL_PRIMARY_UID=${INSTALL_BOOTSTRAP_UID}\"" "${service_unit}" \
    || fail "release service intended UID projection changed"
  grep -Fq "Environment=\"SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=${INSTALL_BOOTSTRAP_CONTEXT_V1}\"" "${service_unit}" \
    || fail "release service carrier projection changed"
  grep -Fq "ReadWritePaths=\"${SELECTED_A}\" /var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp" "${service_unit}" \
    || fail "release ReadWritePaths did not derive from A"
  if grep -Fq -- "${AMBIENT_B}" "${service_unit}"; then
    fail "ambient B retargeted the release service unit"
  fi
  grep -Fq 'ExecStartPost=-/usr/libexec/substrate/substrate-apply-socket-acl --socket /run/substrate.sock substrate' "${socket_dropin}" \
    || fail "release socket ACL drop-in tuple changed"
  grep -Fq 'rm -f /run/substrate.sock' "${sudo_log}" \
    || fail "release fixed same-attempt socket restart unlink changed"
)

verify_release_projections

verify_dev_privileged_boundary() (
  set -euo pipefail
  # shellcheck disable=SC1090
  source <(awk '/^while \[\[ \$# -gt 0 \]\]; do/ { exit } { print }' \
    "${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh")
  resolve_install_bootstrap_context 1 "${SELECTED_A}" ""

  local fixture="${WORK_ROOT}/dev-sudo"
  local stub_bin="${fixture}/bin"
  local sudo_capture="${fixture}/sudo-capture"
  local malicious_systemctl="${stub_bin}/systemctl"
  local malicious_env="${stub_bin}/env"
  local malicious_true="${stub_bin}/true"
  mkdir -p "${stub_bin}"
  cat > "${stub_bin}/sudo" <<'STUB'
#!/usr/bin/env bash
{
  printf '%s\n' '--call--'
  printf '%s\n' "$@"
} >> "${SUDO_CAPTURE:?}"
exit 0
STUB
  printf '#!/usr/bin/env bash\nexit 97\n' > "${malicious_systemctl}"
  printf '#!/usr/bin/env bash\nexit 98\n' > "${malicious_env}"
  printf '#!/usr/bin/env bash\nexit 99\n' > "${malicious_true}"
  chmod +x "${stub_bin}/sudo" "${malicious_systemctl}" "${malicious_env}" "${malicious_true}"
  : > "${sudo_capture}"
  local fixed_systemctl
  local fixed_env
  local fixed_true
  fixed_systemctl="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P systemctl)"
  fixed_env="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P env)"
  fixed_true="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P true)"

  PATH="${stub_bin}:${PATH}" SUDO_CAPTURE="${sudo_capture}" \
    run_privileged \
      systemctl \
      --selected-prefix "${SELECTED_A}" \
      --account "${CURRENT_ACCOUNT}" \
      --uid "${CURRENT_UID}"

  for expected in \
    -- "${fixed_env}" -i \
    "PATH=${PRIVILEGED_TOOL_PATH}" \
    HOME=/root USER=root LOGNAME=root \
    "${fixed_systemctl}" \
    --selected-prefix "${SELECTED_A}" \
    --account "${CURRENT_ACCOUNT}" \
    --uid "${CURRENT_UID}"
  do
    grep -Fxq -- "${expected}" "${sudo_capture}" \
      || fail "dev sudo boundary omitted '${expected}'"
  done
  if grep -Eq -- '(^-E$|SUBSTRATE_(HOME|ROOT|INSTALL_BOOTSTRAP_CONTEXT_V1)=)' "${sudo_capture}"; then
    fail "dev arbitrary privileged tool inherited installation authority"
  fi
  if grep -Fq -- "${INSTALL_WRAPPER_CARRIER}" "${sudo_capture}"; then
    fail "dev arbitrary privileged tool received the authenticated carrier"
  fi
  if grep -Fq -- "${malicious_systemctl}" "${sudo_capture}"; then
    fail "dev sudo boundary selected a privileged tool from ambient PATH"
  fi
  if grep -Fq -- "${malicious_env}" "${sudo_capture}"; then
    fail "dev sudo boundary selected env from ambient PATH"
  fi
  grep -Fxq -- "${fixed_true}" "${sudo_capture}" \
    || fail "dev sudo probe did not use fixed-path true"
  if grep -Fq -- "${malicious_true}" "${sudo_capture}"; then
    fail "dev sudo probe selected true from ambient PATH"
  fi
  local sudo_lines_before
  local sudo_lines_after
  sudo_lines_before="$(wc -l < "${sudo_capture}")"
  set +e
  (
    # shellcheck disable=SC2034 # consumed by sourced run_privileged projection validation
    SUBSTRATE_ROOT="${AMBIENT_B}"
    PATH="${stub_bin}:${PATH}" SUDO_CAPTURE="${sudo_capture}" \
      run_privileged true --selected-prefix "${SELECTED_A}"
  ) >/dev/null 2>&1
  local tamper_status=$?
  set -e
  sudo_lines_after="$(wc -l < "${sudo_capture}")"
  [[ "${tamper_status}" -ne 0 ]] \
    || fail "dev sudo boundary accepted a conflicting checked projection"
  [[ "${sudo_lines_after}" == "${sudo_lines_before}" ]] \
    || fail "dev sudo boundary crossed after projection validation failed"
)

verify_dev_privileged_boundary

[[ "$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -printf '%f\n')" == "sentinel" ]] \
  || fail "release context construction mutated ambient B"
[[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
  || fail "ambient B sentinel changed"

printf '[%s] PASS: release IH symmetry, checked A projections, intended-home PATH, carrier leaves, and ambient-B proof\n' "${SCRIPT_NAME}"
