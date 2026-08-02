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
  HOME="${AMBIENT_B}" \
    SUBSTRATE_HOME="${SELECTED_A}" \
    SUBSTRATE_ROOT="${SELECTED_A}" \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_WRAPPER_COMMITMENT}" \
    SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
    SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
    SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_WRAPPER_CARRIER}" \
    bash -x "${UNINSTALL_CHILD}" "${uninstall_help_args[@]}" \
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
BLA_braille_fill_bar=(frame)
BLA::start_loading_animation() { return 97; }
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

assert_shim_dry_run_carrier_non_disclosure() (
  set -euo pipefail
  # shellcheck disable=SC1090
  source "${INSTALL_CHILD}"

  local fixture="${WORK_ROOT}/r1-carrier-non-disclosure"
  local dry_fixture="${fixture}/dry"
  local live_fixture="${fixture}/live"
  local cli_fixture="${fixture}/cli"
  local fake_child="${fixture}/argv-recorder"
  local placeholder="<redacted-authenticated-bootstrap-carrier-v1>"
  local carrier_flag="--install-bootstrap-context-v1"
  local carrier_prefix="R1_PREFIX_7f4c82b1"
  local carrier_middle="R1_MIDDLE_3a9de650"
  local carrier_suffix="R1_SUFFIX_c6150e29"
  local credential_sentinel="R1_CREDENTIAL_0ff8f91d"
  local token_sentinel="R1_TOKEN_7b2c9a41"
  local authorization_sentinel="R1_AUTHORIZATION_b478e5a2"
  local commitment_preimage_sentinel="R1_COMMITMENT_PREIMAGE_6e13d904"
  local prompt_request_sentinel="R1_PROMPT_REQUEST_29df73c6"
  local private_path_sentinel="R1_PRIVATE_PATH_5aa418ef"
  local carrier
  carrier="${carrier_prefix}"$' spaces quotes\'"and\\slashes\n'"unicode-雪-🧪_${carrier_middle}_${credential_sentinel}_${token_sentinel}_${authorization_sentinel}_${commitment_preimage_sentinel}_${prompt_request_sentinel}_${private_path_sentinel}_${carrier_suffix}"
  local carrier_digest
  carrier_digest="$(printf '%s' "${carrier}" | sha256sum | awk '{ print $1 }')"
  local -a sensitive_fragments=(
    "${carrier_prefix}"
    "${carrier_middle}"
    "${carrier_suffix}"
    "${credential_sentinel}"
    "${token_sentinel}"
    "${authorization_sentinel}"
    "${commitment_preimage_sentinel}"
    "${prompt_request_sentinel}"
    "${private_path_sentinel}"
    "${carrier_digest}"
    "carrier_length=${#carrier}"
  )
  local substrate_cli="${SUBSTRATE_R1_TEST_SUBSTRATE_BIN:-${CARGO_BIN_EXE_substrate:-${REPO_ROOT}/target/debug/substrate}}"
  [[ -x "${substrate_cli}" ]] \
    || fail "R1 actual substrate CLI unavailable; run cargo build --bin substrate"

  mkdir -p "${dry_fixture}" "${live_fixture}" "${cli_fixture}/home"
  cat > "${fake_child}" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
{
  printf '%s\0' "$#"
  printf '%s\0' "$@"
} > "${ARGV_RECORD:?}"
if [[ -n "${FAKE_STDOUT:-}" ]]; then
  printf '%s\n' "${FAKE_STDOUT}"
fi
if [[ -n "${FAKE_STDERR:-}" ]]; then
  printf '%s\n' "${FAKE_STDERR}" >&2
fi
exit "${FAKE_EXIT_STATUS:-0}"
STUB
  chmod +x "${fake_child}"

  DRY_RUN=1
  local separate_stdout="${dry_fixture}/separate.stdout"
  local separate_stderr="${dry_fixture}/separate.stderr"
  local separate_record="${dry_fixture}/separate.argv"
  ARGV_RECORD="${separate_record}" \
    run_cmd_with_redacted_install_bootstrap_carrier \
      "${fake_child}" \
      "before value" \
      "" \
      "${carrier_flag}" "${carrier}" \
      "after value" \
      "-nonsensitive-leading-dash" \
      >"${separate_stdout}" 2>"${separate_stderr}"
  [[ ! -s "${separate_stdout}" ]] || fail "R1 separate-form dry-run wrote stdout"
  [[ ! -e "${separate_record}" ]] || fail "R1 separate-form dry-run executed the child"
  [[ "$(grep -Fo -- "${placeholder}" "${separate_stderr}" | wc -l)" -eq 1 ]] \
    || fail "R1 separate-form dry-run did not render exactly one fixed placeholder"
  grep -Fq -- "${carrier_flag} ${placeholder}" "${separate_stderr}" \
    || fail "R1 separate-form dry-run did not preserve the redacted flag boundary"

  local equals_stdout="${dry_fixture}/equals.stdout"
  local equals_stderr="${dry_fixture}/equals.stderr"
  local equals_record="${dry_fixture}/equals.argv"
  ARGV_RECORD="${equals_record}" \
    run_cmd_with_redacted_install_bootstrap_carrier \
      "${fake_child}" \
      "${carrier_flag}==${carrier}" \
      "reordered suffix" \
      "" \
      >"${equals_stdout}" 2>"${equals_stderr}"
  [[ ! -s "${equals_stdout}" ]] || fail "R1 equals-form dry-run wrote stdout"
  [[ ! -e "${equals_record}" ]] || fail "R1 equals-form dry-run executed the child"
  [[ "$(grep -Fo -- "${placeholder}" "${equals_stderr}" | wc -l)" -eq 1 ]] \
    || fail "R1 equals-form dry-run did not render exactly one fixed placeholder"
  grep -Fq -- "${carrier_flag}=${placeholder}" "${equals_stderr}" \
    || fail "R1 equals-form dry-run did not preserve the redacted flag boundary"

  local structural_error_file="${dry_fixture}/structural-error.expected"
  printf '%s\n' \
    '[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments' \
    > "${structural_error_file}"
  local -a rejection_cases=(
    missing
    separate-missing
    separate-empty
    equals-empty
    duplicate-separate
    duplicate-mixed
    malformed-near-match
    after-end-separate
    after-end-equals
    after-end-near-match
    separate-leading-dash
  )
  local rejection_case
  for rejection_case in "${rejection_cases[@]}"; do
    local -a rejection_argv=("${fake_child}")
    case "${rejection_case}" in
      missing)
        rejection_argv+=("ordinary")
        ;;
      separate-missing)
        rejection_argv+=("${carrier_flag}")
        ;;
      separate-empty)
        rejection_argv+=("${carrier_flag}" "")
        ;;
      equals-empty)
        rejection_argv+=("${carrier_flag}=")
        ;;
      duplicate-separate)
        rejection_argv+=(
          "${carrier_flag}" "${carrier_prefix}"
          "${carrier_flag}" "${carrier_suffix}"
        )
        ;;
      duplicate-mixed)
        rejection_argv+=(
          "${carrier_flag}=${carrier_prefix}"
          "${carrier_flag}" "${carrier_suffix}"
        )
        ;;
      malformed-near-match)
        rejection_argv+=(
          "${carrier_flag}" "${carrier}"
          "${carrier_flag}-malformed-${carrier_middle}"
        )
        ;;
      after-end-separate)
        rejection_argv+=(
          "${carrier_flag}" "${carrier}"
          "--" "${carrier_flag}" "${carrier_prefix}"
        )
        ;;
      after-end-equals)
        rejection_argv+=(
          "${carrier_flag}" "${carrier}"
          "--" "${carrier_flag}=${carrier_middle}"
        )
        ;;
      after-end-near-match)
        rejection_argv+=(
          "${carrier_flag}" "${carrier}"
          "--" "${carrier_flag}-malformed-${carrier_suffix}"
        )
        ;;
      separate-leading-dash)
        rejection_argv+=("${carrier_flag}" "-${carrier_middle}")
        ;;
    esac

    local rejection_stdout="${dry_fixture}/${rejection_case}.stdout"
    local rejection_stderr="${dry_fixture}/${rejection_case}.stderr"
    local rejection_record="${dry_fixture}/${rejection_case}.argv"
    local rejection_status
    set +e
    ARGV_RECORD="${rejection_record}" \
      run_cmd_with_redacted_install_bootstrap_carrier "${rejection_argv[@]}" \
      >"${rejection_stdout}" 2>"${rejection_stderr}"
    rejection_status=$?
    set -e
    [[ "${rejection_status}" -eq 2 ]] \
      || fail "R1 ${rejection_case} rejection returned ${rejection_status}, expected 2"
    [[ ! -s "${rejection_stdout}" ]] \
      || fail "R1 ${rejection_case} rejection wrote stdout"
    cmp -s "${structural_error_file}" "${rejection_stderr}" \
      || fail "R1 ${rejection_case} rejection did not emit the fixed diagnostic"
    [[ ! -e "${rejection_record}" ]] \
      || fail "R1 ${rejection_case} rejection executed the child"
  done

  local invalid_carrier="R1_INVALID_CARRIER_8e2b06d7"
  local invalid_attached_dash="-R1_INVALID_DASH_CARRIER_52f60ac9"
  local invalid_attached_equals="=R1_INVALID_EQUALS_CARRIER_b307fc48"
  local cli_error_file="${cli_fixture}/invalid-carrier.expected"
  printf '%s\n' 'substrate: invalid install bootstrap context' > "${cli_error_file}"
  DRY_RUN=0
  local invalid_case
  for invalid_case in separate attached-dash attached-equals; do
    local -a invalid_argv=("${substrate_cli}")
    local invalid_marker
    case "${invalid_case}" in
      separate)
        invalid_marker="${invalid_carrier}"
        invalid_argv+=("${carrier_flag}" "${invalid_marker}" --shim-deploy)
        ;;
      attached-dash)
        invalid_marker="${invalid_attached_dash}"
        invalid_argv+=("${carrier_flag}=${invalid_marker}" --shim-deploy)
        ;;
      attached-equals)
        invalid_marker="${invalid_attached_equals}"
        invalid_argv+=("${carrier_flag}=${invalid_marker}" --shim-deploy)
        ;;
    esac

    local invalid_stdout="${cli_fixture}/${invalid_case}.stdout"
    local invalid_stderr="${cli_fixture}/${invalid_case}.stderr"
    local invalid_status
    set +e
    HOME="${cli_fixture}/home" \
      XDG_CONFIG_HOME="${cli_fixture}/xdg-config" \
      XDG_STATE_HOME="${cli_fixture}/xdg-state" \
      SUBSTRATE_HOME="${cli_fixture}/substrate-home" \
      SUBSTRATE_ROOT="${cli_fixture}/substrate-home" \
      SHIM_TRACE_LOG="${cli_fixture}/${invalid_case}.trace" \
      RUST_LOG=off \
      run_cmd_with_redacted_install_bootstrap_carrier "${invalid_argv[@]}" \
      >"${invalid_stdout}" 2>"${invalid_stderr}"
    invalid_status=$?
    set -e
    [[ "${invalid_status}" -eq 2 ]] \
      || fail "R1 actual CLI ${invalid_case} invalid carrier returned ${invalid_status}, expected 2"
    [[ ! -s "${invalid_stdout}" ]] \
      || fail "R1 actual CLI ${invalid_case} invalid carrier wrote stdout"
    cmp -s "${cli_error_file}" "${invalid_stderr}" \
      || fail "R1 actual CLI ${invalid_case} invalid carrier changed its fixed diagnostic"
    if grep -RFq -- "${invalid_marker}" "${cli_fixture}"; then
      fail "R1 actual CLI ${invalid_case} failure disclosed its carrier"
    fi
  done

  DRY_RUN=1
  INSTALL_BOOTSTRAP_CONTEXT_V1="${carrier}"
  local deploy_dry_record="${dry_fixture}/deploy.argv"
  local deploy_dry_trace
  deploy_dry_trace="$({
    set -x
    ARGV_RECORD="${deploy_dry_record}" deploy_shims "${fake_child}"
    if [[ $- != *x* ]]; then
      printf 'R1_XTRACE_NOT_RESTORED_AFTER_DRY_RUN\n'
      exit 98
    fi
    printf 'R1_TRACE_RESTORED_AFTER_DRY_RUN\n'
    { set +x; } 2>/dev/null
  } 2>&1)"
  printf '%s\n' "${deploy_dry_trace}" > "${dry_fixture}/deploy.trace"
  [[ ! -e "${deploy_dry_record}" ]] || fail "R1 deploy dry-run executed the child"
  [[ "${deploy_dry_trace}" == *"R1_TRACE_RESTORED_AFTER_DRY_RUN"* ]] \
    || fail "R1 deploy dry-run did not restore inherited xtrace"
  [[ "$(grep -Fo -- "${placeholder}" "${dry_fixture}/deploy.trace" | wc -l)" -eq 1 ]] \
    || fail "R1 deploy dry-run did not render exactly one fixed placeholder"

  local sensitive_fragment
  for sensitive_fragment in "${sensitive_fragments[@]}"; do
    if grep -RFq -- "${sensitive_fragment}" "${dry_fixture}"; then
      fail "R1 dry-run outputs or generated files disclosed protected carrier material"
    fi
  done
  if grep -RFq -- "carrier_length=" "${dry_fixture}"; then
    fail "R1 dry-run output disclosed a carrier length"
  fi

  DRY_RUN=0
  local live_record="${live_fixture}/separate.argv"
  local -a expected_live_argv=(
    "arg-before"
    ""
    "space stays one argument"
    "quote'\"mix"
    "back\\slash"
    "unicode-雪-🧪"
    $'line-one\nline-two'
    "-nonsensitive-leading-dash"
    "${carrier_flag}"
    "${carrier}"
    "--shim-deploy"
    "arg-after"
  )
  ARGV_RECORD="${live_record}" \
    run_cmd_with_redacted_install_bootstrap_carrier \
      "${fake_child}" "${expected_live_argv[@]}"
  local -a live_capture=()
  mapfile -d '' -t live_capture < "${live_record}"
  [[ "${live_capture[0]}" == "${#expected_live_argv[@]}" ]] \
    || fail "R1 live separate-form argv count changed"
  local live_index
  for live_index in "${!expected_live_argv[@]}"; do
    [[ "${live_capture[live_index + 1]}" == "${expected_live_argv[live_index]}" ]] \
      || fail "R1 live separate-form argv changed at index ${live_index}"
  done
  if grep -aFq -- "${placeholder}" "${live_record}"; then
    fail "R1 live execution received the display placeholder"
  fi

  local leading_dash_carrier="-R1_LEADING_DASH_52f60ac9"
  local equals_live_record="${live_fixture}/equals.argv"
  local -a expected_equals_argv=(
    "arg-before"
    "${carrier_flag}=${leading_dash_carrier}"
    ""
    "arg-after"
  )
  ARGV_RECORD="${equals_live_record}" \
    run_cmd_with_redacted_install_bootstrap_carrier \
      "${fake_child}" "${expected_equals_argv[@]}"
  local -a equals_live_capture=()
  mapfile -d '' -t equals_live_capture < "${equals_live_record}"
  [[ "${equals_live_capture[0]}" == "${#expected_equals_argv[@]}" ]] \
    || fail "R1 live equals-form argv count changed"
  for live_index in "${!expected_equals_argv[@]}"; do
    [[ "${equals_live_capture[live_index + 1]}" == "${expected_equals_argv[live_index]}" ]] \
      || fail "R1 live equals-form argv changed at index ${live_index}"
  done

  local leading_equals_carrier="=R1_LEADING_EQUALS_b307fc48"
  local leading_equals_record="${live_fixture}/leading-equals.argv"
  ARGV_RECORD="${leading_equals_record}" \
    run_cmd_with_redacted_install_bootstrap_carrier \
      "${fake_child}" "${carrier_flag}=${leading_equals_carrier}" --shim-deploy
  local -a leading_equals_capture=()
  mapfile -d '' -t leading_equals_capture < "${leading_equals_record}"
  [[ "${leading_equals_capture[0]}" == "2" \
      && "${leading_equals_capture[1]}" == "${carrier_flag}=${leading_equals_carrier}" \
      && "${leading_equals_capture[2]}" == "--shim-deploy" ]] \
    || fail "R1 live leading-equals carrier argv changed"

  local child_failure_stdout="${live_fixture}/child-failure.stdout"
  local child_failure_stderr="${live_fixture}/child-failure.stderr"
  local child_failure_record="${live_fixture}/child-failure.argv"
  local child_failure_status
  set +e
  ARGV_RECORD="${child_failure_record}" \
    FAKE_EXIT_STATUS=37 \
    FAKE_STDERR="R1 fixed fake child failure" \
    run_cmd_with_redacted_install_bootstrap_carrier \
      "${fake_child}" "${carrier_flag}" "${carrier}" --shim-deploy \
      >"${child_failure_stdout}" 2>"${child_failure_stderr}"
  child_failure_status=$?
  set -e
  [[ "${child_failure_status}" -eq 37 ]] \
    || fail "R1 helper changed the valid child exit status"
  local child_failure_expected="${live_fixture}/child-failure.expected"
  printf '%s\n' 'R1 fixed fake child failure' > "${child_failure_expected}"
  cmp -s "${child_failure_expected}" "${child_failure_stderr}" \
    || fail "R1 helper changed the valid child error rendering"
  for sensitive_fragment in "${sensitive_fragments[@]}"; do
    if grep -Fq -- "${sensitive_fragment}" "${child_failure_stdout}" "${child_failure_stderr}"; then
      fail "R1 valid child failure output disclosed protected carrier material"
    fi
  done

  local deploy_success_trace
  local deploy_success_record="${live_fixture}/deploy-success.argv"
  deploy_success_trace="$({
    set -x
    ARGV_RECORD="${deploy_success_record}" deploy_shims "${fake_child}"
    if [[ $- != *x* ]]; then
      printf 'R1_XTRACE_NOT_RESTORED_AFTER_SUCCESS\n'
      exit 98
    fi
    printf 'R1_TRACE_RESTORED_AFTER_SUCCESS\n'
    { set +x; } 2>/dev/null
  } 2>&1)"
  [[ "${deploy_success_trace}" == *"R1_TRACE_RESTORED_AFTER_SUCCESS"* ]] \
    || fail "R1 deploy success did not restore inherited xtrace"
  for sensitive_fragment in "${sensitive_fragments[@]}"; do
    if [[ "${deploy_success_trace}" == *"${sensitive_fragment}"* ]]; then
      fail "R1 deploy success trace disclosed protected carrier material"
    fi
  done

  local deploy_failure_trace
  local deploy_failure_record="${live_fixture}/deploy-failure.argv"
  local deploy_failure_status
  set +e
  deploy_failure_trace="$({
    set -x
    ARGV_RECORD="${deploy_failure_record}" FAKE_EXIT_STATUS=37 deploy_shims "${fake_child}"
    local status=$?
    if [[ $- != *x* ]]; then
      printf 'R1_XTRACE_NOT_RESTORED_AFTER_FAILURE status=%s\n' "${status}"
      exit 98
    fi
    printf 'R1_TRACE_RESTORED_AFTER_FAILURE status=%s\n' "${status}"
    { set +x; } 2>/dev/null
    exit "${status}"
  } 2>&1)"
  deploy_failure_status=$?
  set -e
  [[ "${deploy_failure_status}" -eq 37 ]] \
    || fail "R1 deploy failure changed the child exit status"
  [[ "${deploy_failure_trace}" == *"R1_TRACE_RESTORED_AFTER_FAILURE status=37"* ]] \
    || fail "R1 deploy failure did not restore inherited xtrace"
  for sensitive_fragment in "${sensitive_fragments[@]}"; do
    if [[ "${deploy_failure_trace}" == *"${sensitive_fragment}"* ]]; then
      fail "R1 deploy failure trace disclosed protected carrier material"
    fi
  done

  local saved_carrier="${INSTALL_BOOTSTRAP_CONTEXT_V1}"
  local structural_trace
  local structural_status
  INSTALL_BOOTSTRAP_CONTEXT_V1=""
  set +e
  structural_trace="$({
    set -x
    deploy_shims "${fake_child}"
    local status=$?
    if [[ $- != *x* ]]; then
      printf 'R1_XTRACE_NOT_RESTORED_AFTER_REJECTION status=%s\n' "${status}"
      exit 98
    fi
    printf 'R1_TRACE_RESTORED_AFTER_REJECTION status=%s\n' "${status}"
    { set +x; } 2>/dev/null
    exit "${status}"
  } 2>&1)"
  structural_status=$?
  set -e
  INSTALL_BOOTSTRAP_CONTEXT_V1="${saved_carrier}"
  [[ "${structural_status}" -eq 2 ]] \
    || fail "R1 deploy structural rejection did not return 2"
  [[ "${structural_trace}" == *"R1_TRACE_RESTORED_AFTER_REJECTION status=2"* ]] \
    || fail "R1 deploy structural rejection did not restore inherited xtrace"
  for sensitive_fragment in "${sensitive_fragments[@]}"; do
    if [[ "${structural_trace}" == *"${sensitive_fragment}"* ]]; then
      fail "R1 deploy structural rejection trace disclosed protected carrier material"
    fi
  done

  [[ $- != *x* ]] || fail "R1 test expected xtrace to be initially disabled"
  DRY_RUN=1
  deploy_shims "${fake_child}" >/dev/null 2>"${dry_fixture}/initially-disabled.stderr"
  [[ $- != *x* ]] || fail "R1 deploy enabled xtrace when it was initially disabled"

  DRY_RUN=0
  local initially_disabled_failure_status
  set +e
  ARGV_RECORD="${live_fixture}/initially-disabled-failure.argv" \
    FAKE_EXIT_STATUS=37 \
    deploy_shims "${fake_child}" \
    >"${live_fixture}/initially-disabled-failure.stdout" \
    2>"${live_fixture}/initially-disabled-failure.stderr"
  initially_disabled_failure_status=$?
  set -e
  [[ "${initially_disabled_failure_status}" -eq 37 ]] \
    || fail "R1 initially-disabled deploy failure changed child status"
  [[ $- != *x* ]] || fail "R1 deploy failure enabled initially-disabled xtrace"
  for sensitive_fragment in "${sensitive_fragments[@]}"; do
    if grep -Fq -- "${sensitive_fragment}" \
      "${live_fixture}/initially-disabled-failure.stdout" \
      "${live_fixture}/initially-disabled-failure.stderr"; then
      fail "R1 initially-disabled failure output disclosed protected carrier material"
    fi
  done

  DRY_RUN=1
  local run_cmd_stdout="${dry_fixture}/run-cmd.stdout"
  local run_cmd_stderr="${dry_fixture}/run-cmd.stderr"
  local run_cmd_expected="${dry_fixture}/run-cmd.expected"
  run_cmd printf compat value >"${run_cmd_stdout}" 2>"${run_cmd_stderr}"
  printf '[%s][dry-run] printf compat value\n' "${INSTALLER_NAME}" > "${run_cmd_expected}"
  [[ ! -s "${run_cmd_stdout}" ]] || fail "R1 changed nonsensitive run_cmd dry-run stdout"
  cmp -s "${run_cmd_expected}" "${run_cmd_stderr}" \
    || fail "R1 changed nonsensitive run_cmd dry-run rendering"

  DRY_RUN=0
  local run_cmd_record="${live_fixture}/run-cmd.argv"
  ARGV_RECORD="${run_cmd_record}" run_cmd "${fake_child}" "compat" "" "two words"
  local -a run_cmd_capture=()
  mapfile -d '' -t run_cmd_capture < "${run_cmd_record}"
  [[ "${run_cmd_capture[0]}" == "3" \
      && "${run_cmd_capture[1]}" == "compat" \
      && "${run_cmd_capture[2]}" == "" \
      && "${run_cmd_capture[3]}" == "two words" ]] \
    || fail "R1 changed nonsensitive run_cmd live argv"

  for sensitive_fragment in "${sensitive_fragments[@]}"; do
    if grep -RFq -- "${sensitive_fragment}" "${dry_fixture}"; then
      fail "R1 final dry-run output scan found protected carrier material"
    fi
  done
)

assert_shim_dry_run_carrier_non_disclosure

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
  local path_rc="${path_home}/.bashrc"
  local path_rc_first="${WORK_ROOT}/path-rc-first"
  local path_rc_expected="${WORK_ROOT}/path-rc-expected"
  local stale_path="${WORK_ROOT}/stale/bin"
  mkdir -p "${path_home}"
  printf '%s\n' \
    'unrelated-before' \
    "${PATH_SNIPPET_START}" \
    "export PATH=\"${stale_path}:\${PATH}\"" \
    "${PATH_SNIPPET_END}" \
    'unrelated-after' > "${path_rc}"
  SHELL=/bin/bash update_shell_path "${PREFIX}/bin" "${path_home}"
  {
    printf '%s\n' 'unrelated-before' 'unrelated-after'
    printf '\n'
    render_path_snippet_sh "${PREFIX}/bin"
  } > "${path_rc_expected}"
  cmp -s "${path_rc_expected}" "${path_rc}" \
    || fail "PATH upsert did not preserve the exact unrelated profile bytes"
  cp "${path_rc}" "${path_rc_first}"
  grep -Fxq 'unrelated-before' "${path_rc}" \
    || fail "PATH upsert changed unrelated bytes before the managed block"
  grep -Fxq 'unrelated-after' "${path_rc}" \
    || fail "PATH upsert changed unrelated bytes after the managed block"
  [[ "$(grep -Fxc -- "${PATH_SNIPPET_START}" "${path_rc}")" -eq 1 \
      && "$(grep -Fxc -- "${PATH_SNIPPET_END}" "${path_rc}")" -eq 1 ]] \
    || fail "PATH upsert did not leave exactly one managed block"
  grep -Fq "${PREFIX}/bin" "${path_rc}" \
    || fail "PATH upsert did not target the intended account home"
  if grep -Fq "${stale_path}" "${path_rc}"; then
    fail "PATH upsert retained the stale managed block"
  fi
  SHELL=/bin/bash update_shell_path "${PREFIX}/bin" "${path_home}"
  cmp -s "${path_rc_first}" "${path_rc}" \
    || fail "repeat PATH upsert changed the profile bytes"
  [[ "$(grep -Fxc -- "${PATH_SNIPPET_START}" "${path_rc}")" -eq 1 \
      && "$(grep -Fxc -- "${PATH_SNIPPET_END}" "${path_rc}")" -eq 1 ]] \
    || fail "repeat PATH upsert did not leave exactly one managed block"
  grep -Fq "${PREFIX}/bin" "${path_rc}" \
    || fail "repeat PATH upsert did not retain the intended account home"
  grep -Fxq 'unrelated-before' "${path_rc}" \
    || fail "repeat PATH upsert changed unrelated bytes before the managed block"
  grep -Fxq 'unrelated-after' "${path_rc}" \
    || fail "repeat PATH upsert changed unrelated bytes after the managed block"
  if grep -Fq "${stale_path}" "${path_rc}"; then
    fail "repeat PATH upsert restored the stale managed block"
  fi
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
  PKG_MANAGER=pacman
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
