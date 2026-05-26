#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

# shellcheck disable=SC1091
source "${repo_root}/scripts/substrate/install-substrate.sh"

assert_eq() {
  local actual="$1"
  local expected="$2"
  local label="$3"

  if [[ "${actual}" != "${expected}" ]]; then
    printf '[pkg-manager-detection-smoke] expected %s=%q, got %q\n' "${label}" "${expected}" "${actual}" >&2
    exit 1
  fi
}

assert_selected() {
  local expected_path="$1"
  assert_eq "${OS_RELEASE_INPUT_STATE}" "selected" "OS_RELEASE_INPUT_STATE"
  assert_eq "${OS_RELEASE_SELECTED_PATH}" "${expected_path}" "OS_RELEASE_SELECTED_PATH"
  assert_eq "${DETECTED_DISTRO_ID}" "${DISTRO_UNKNOWN_SENTINEL}" "DETECTED_DISTRO_ID"
  assert_eq "${DETECTED_DISTRO_ID_LIKE}" "${DISTRO_UNKNOWN_SENTINEL}" "DETECTED_DISTRO_ID_LIKE"
}

assert_unavailable() {
  assert_eq "${OS_RELEASE_INPUT_STATE}" "unavailable" "OS_RELEASE_INPUT_STATE"
  assert_eq "${OS_RELEASE_SELECTED_PATH}" "" "OS_RELEASE_SELECTED_PATH"
  assert_eq "${DETECTED_DISTRO_ID}" "${DISTRO_UNKNOWN_SENTINEL}" "DETECTED_DISTRO_ID"
  assert_eq "${DETECTED_DISTRO_ID_LIKE}" "${DISTRO_UNKNOWN_SENTINEL}" "DETECTED_DISTRO_ID_LIKE"
}

assert_parsed_fields() {
  local expected_id="$1"
  local expected_id_like="$2"

  assert_eq "${DETECTED_DISTRO_ID}" "${expected_id}" "DETECTED_DISTRO_ID"
  assert_eq "${DETECTED_DISTRO_ID_LIKE}" "${expected_id_like}" "DETECTED_DISTRO_ID_LIKE"
}

assert_detected_manager() {
  local expected_manager="$1"
  local expected_source="$2"

  assert_eq "${PKG_MANAGER}" "${expected_manager}" "PKG_MANAGER"
  assert_eq "${PKG_MANAGER_SOURCE}" "${expected_source}" "PKG_MANAGER_SOURCE"
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"

  if [[ "${haystack}" != *"${needle}"* ]]; then
    printf '[pkg-manager-detection-smoke] expected %s to contain %q\n' "${label}" "${needle}" >&2
    exit 1
  fi
}

assert_not_contains() {
  local haystack="$1"
  local needle="$2"
  local label="$3"

  if [[ "${haystack}" == *"${needle}"* ]]; then
    printf '[pkg-manager-detection-smoke] expected %s to omit %q\n' "${label}" "${needle}" >&2
    exit 1
  fi
}

assert_contains_once() {
  local haystack="$1"
  local needle="$2"
  local label="$3"
  local remainder=""

  assert_contains "${haystack}" "${needle}" "${label}"
  remainder="${haystack#*"${needle}"}"
  if [[ "${remainder}" == *"${needle}"* ]]; then
    printf '[pkg-manager-detection-smoke] expected %s to contain %q exactly once\n' "${label}" "${needle}" >&2
    exit 1
  fi
}

assert_in_order() {
  local haystack="$1"
  local first="$2"
  local second="$3"
  local label="$4"

  if [[ "${haystack}" != *"${first}"*"${second}"* ]]; then
    printf '[pkg-manager-detection-smoke] expected %s to place %q before %q\n' "${label}" "${first}" "${second}" >&2
    exit 1
  fi
}

capture_failure_output() {
  local expected_status="$1"
  local label="$2"
  shift 2

  local output=""
  local status=0

  set +e
  output="$(
    (
      "$@"
    ) 2>&1
  )"
  status=$?
  set -e

  assert_eq "${status}" "${expected_status}" "${label} status"
  printf '%s' "${output}"
}

run_wrapper_fixture_case() {
  local expected_status="$1"
  local output=""
  local actual_status=0

  set +e
  output="$(
    HOME="${wrapper_fixture_home}" \
    SHELL="/bin/bash" \
    SUBSTRATE_TEST_UPSTREAM_STATUS="${expected_status}" \
    "${wrapper_fixture_root}/install.sh" --prefix "${wrapper_fixture_prefix}" 2>&1
  )"
  actual_status=$?
  set -e

  assert_eq "${actual_status}" "${expected_status}" "wrapper exit status ${expected_status}"
  if [[ "${expected_status}" -eq 0 ]]; then
    assert_contains "${output}" "Substrate install successful!" "wrapper success output"
  else
    assert_contains "${output}" "[substrate-install] Failed." "wrapper failure prefix ${expected_status}"
    assert_contains "${output}" "upstream-status=${expected_status}" "wrapper upstream stderr ${expected_status}"
  fi
}

reset_detected_manager() {
  PKG_MANAGER=""
  PKG_MANAGER_SOURCE=""
  PKG_MANAGER_ENV_OVERRIDE=""
  PKG_MANAGER_FLAG_OVERRIDE=""
  PATH_PROBE_DETECTED_MANAGERS=()
  PKG_MANAGER_PATH_PROBE_WARNING_EMITTED=0
}

reset_installer_state() {
  reset_detected_manager
  PKG_MANAGER_DECISION_LINE_EMITTED=0
  APT_UPDATED=0
  SUDO_CMD=()
  DRY_RUN=0
}

make_stub_command() {
  local path="$1"

  cat > "${path}" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "${path}"
}

run_invalid_flag_case() {
  PATH="${manager_bin}"
  reset_installer_state
  parse_args --pkg-manager apk
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
  ensure_linux_packages_for_commands curl
}

run_invalid_env_case() {
  PATH="${manager_bin}"
  reset_installer_state
  PKG_MANAGER_ENV_OVERRIDE="apk"
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
  ensure_linux_packages_for_commands curl
}

run_missing_flag_manager_case() {
  local case_bin="${tmpdir}/missing-flag-bin"

  rm -rf "${case_bin}"
  mkdir -p "${case_bin}"
  cp -R "${manager_bin}"/. "${case_bin}"/
  rm -f "${case_bin}/pacman"

  PATH="${case_bin}"
  reset_installer_state
  parse_args --pkg-manager pacman
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
  ensure_linux_packages_for_commands curl
}

run_missing_env_manager_case() {
  local case_bin="${tmpdir}/missing-env-bin"

  rm -rf "${case_bin}"
  mkdir -p "${case_bin}"
  cp -R "${manager_bin}"/. "${case_bin}"/
  rm -f "${case_bin}/dnf"

  PATH="${case_bin}"
  reset_installer_state
  PKG_MANAGER_ENV_OVERRIDE="dnf"
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
  ensure_linux_packages_for_commands curl
}

run_no_manager_case() {
  local case_bin="${tmpdir}/no-manager-bin"

  rm -rf "${case_bin}"
  mkdir -p "${case_bin}"
  cp "${manager_bin}/sudo" "${case_bin}/sudo"

  PATH="${case_bin}"
  reset_installer_state
  DRY_RUN=1
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${arch_fixture}"
  ensure_linux_packages_for_commands curl tar
}

run_no_manager_without_sudo_case() {
  local case_bin="${tmpdir}/no-manager-no-sudo-bin"

  rm -rf "${case_bin}"
  mkdir -p "${case_bin}"

  PATH="${case_bin}"
  reset_installer_state
  DRY_RUN=1
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="${arch_fixture}"
  ensure_linux_packages_for_commands curl tar
}

tmpdir="$(mktemp -d -t substrate-pkg-manager-detection.XXXXXX)"
trap 'rm -rf "${tmpdir}"' EXIT

valid_alt="${tmpdir}/valid-os-release"
unreadable_alt="${tmpdir}/unreadable-os-release"
non_regular_alt="${tmpdir}/os-release-dir"
missing_alt="${tmpdir}/missing-os-release"
manager_bin="${tmpdir}/bin"
wrapper_fixture_root="${tmpdir}/wrapper-fixture"
wrapper_fixture_prefix="${tmpdir}/wrapper-prefix"
wrapper_fixture_home="${tmpdir}/wrapper-home"

cat > "${valid_alt}" <<'EOF'
ID=ubuntu
ID_LIKE=debian
EOF

parser_fixture="${tmpdir}/parser-os-release"
missing_id_fixture="${tmpdir}/missing-id-os-release"
empty_value_fixture="${tmpdir}/empty-value-os-release"
debian_like_fixture="${tmpdir}/debian-like-os-release"
fedora_fixture="${tmpdir}/fedora-os-release"
arch_fixture="${tmpdir}/arch-os-release"
suse_fixture="${tmpdir}/suse-os-release"

cat > "${parser_fixture}" <<'EOF'
  # comment line
NAME=ignored
ID=ubuntu
ID_LIKE="Debian Ubuntu"

ID='$(printf UBUNTU)'
ID_LIKE='RHEL Fedora'
EOF

cat > "${missing_id_fixture}" <<'EOF'
ID_LIKE="Debian"
EOF

cat > "${empty_value_fixture}" <<'EOF'
ID=""
ID_LIKE='`APT-GET`'
EOF

cat > "${debian_like_fixture}" <<'EOF'
ID=custom
ID_LIKE="ubuntu debian"
EOF

cat > "${fedora_fixture}" <<'EOF'
ID=rocky
ID_LIKE="rhel fedora"
EOF

cat > "${arch_fixture}" <<'EOF'
ID=endeavouros
ID_LIKE=arch
EOF

cat > "${suse_fixture}" <<'EOF'
ID=opensuse-tumbleweed
ID_LIKE="suse opensuse"
EOF

cat > "${unreadable_alt}" <<'EOF'
ID=fedora
EOF
chmod 000 "${unreadable_alt}"
mkdir -p "${non_regular_alt}"
mkdir -p "${manager_bin}"
make_stub_command "${manager_bin}/apt-get"
make_stub_command "${manager_bin}/dnf"
make_stub_command "${manager_bin}/yum"
make_stub_command "${manager_bin}/pacman"
make_stub_command "${manager_bin}/zypper"
make_stub_command "${manager_bin}/sudo"

mkdir -p "${wrapper_fixture_root}/loader" "${wrapper_fixture_prefix}" "${wrapper_fixture_home}"
cp "${repo_root}/scripts/substrate/install.sh" "${wrapper_fixture_root}/install.sh"
chmod +x "${wrapper_fixture_root}/install.sh"

cat > "${wrapper_fixture_root}/install-substrate.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
status="${SUBSTRATE_TEST_UPSTREAM_STATUS:-0}"
printf 'upstream-status=%s\n' "${status}" >&2
exit "${status}"
EOF
chmod +x "${wrapper_fixture_root}/install-substrate.sh"

cat > "${wrapper_fixture_root}/loader/bash_loading_animations.sh" <<'EOF'
#!/usr/bin/env bash
BLA_braille_fill_bar=()
BLA::start_loading_animation() { :; }
BLA::stop_loading_animation() { :; }
EOF

unset SUBSTRATE_INSTALL_OS_RELEASE_PATH
resolve_selected_os_release_input
assert_selected "/etc/os-release"

SUBSTRATE_INSTALL_OS_RELEASE_PATH=""
resolve_selected_os_release_input
assert_selected "/etc/os-release"

SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
resolve_selected_os_release_input
assert_selected "${valid_alt}"
parse_selected_os_release_fields
assert_parsed_fields "ubuntu" "debian"

SUBSTRATE_INSTALL_OS_RELEASE_PATH="${parser_fixture}"
resolve_selected_os_release_input
assert_selected "${parser_fixture}"
parse_selected_os_release_fields
assert_parsed_fields '$(printf ubuntu)' "rhel fedora"

SUBSTRATE_INSTALL_OS_RELEASE_PATH="${missing_id_fixture}"
resolve_selected_os_release_input
assert_selected "${missing_id_fixture}"
parse_selected_os_release_fields
assert_parsed_fields "${DISTRO_UNKNOWN_SENTINEL}" "debian"

original_path="${PATH}"
PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
detect_package_manager
assert_detected_manager "apt-get" "os_release"
assert_eq "${OS_RELEASE_INPUT_STATE}" "selected" "OS_RELEASE_INPUT_STATE"
assert_eq "${OS_RELEASE_SELECTED_PATH}" "${valid_alt}" "OS_RELEASE_SELECTED_PATH"
assert_parsed_fields "ubuntu" "debian"
PATH="${original_path}"

PATH="${manager_bin}"
reset_installer_state
parse_args --pkg-manager pacman
PKG_MANAGER_ENV_OVERRIDE="apt-get"
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
detect_package_manager
assert_detected_manager "pacman" "flag"
assert_parsed_fields "ubuntu" "debian"
PATH="${original_path}"

PATH="${manager_bin}"
reset_detected_manager
PKG_MANAGER_ENV_OVERRIDE="dnf"
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
detect_package_manager
assert_detected_manager "dnf" "env"
assert_eq "${OS_RELEASE_INPUT_STATE}" "selected" "OS_RELEASE_INPUT_STATE"
assert_eq "${OS_RELEASE_SELECTED_PATH}" "${valid_alt}" "OS_RELEASE_SELECTED_PATH"
assert_parsed_fields "ubuntu" "debian"
PATH="${original_path}"

PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${debian_like_fixture}"
detect_package_manager
assert_detected_manager "apt-get" "os_release"
PATH="${original_path}"

PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${fedora_fixture}"
detect_package_manager
assert_detected_manager "dnf" "os_release"
PATH="${original_path}"

rm -f "${manager_bin}/dnf"
PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${fedora_fixture}"
detect_package_manager
assert_detected_manager "yum" "os_release"
PATH="${original_path}"
make_stub_command "${manager_bin}/dnf"

PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${arch_fixture}"
detect_package_manager
assert_detected_manager "pacman" "os_release"
PATH="${original_path}"

PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${suse_fixture}"
detect_package_manager
assert_detected_manager "zypper" "os_release"
PATH="${original_path}"

rm -f "${manager_bin}/pacman" "${manager_bin}/dnf" "${manager_bin}/yum" "${manager_bin}/zypper"
PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${arch_fixture}"
detect_package_manager
assert_detected_manager "apt-get" "path_probe"
PATH="${original_path}"
make_stub_command "${manager_bin}/dnf"
make_stub_command "${manager_bin}/yum"
make_stub_command "${manager_bin}/zypper"

PATH="${manager_bin}"
reset_detected_manager
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${arch_fixture}"
detect_package_manager
assert_detected_manager "apt-get" "path_probe"
PATH="${original_path}"
make_stub_command "${manager_bin}/pacman"

flag_decision_line='Detected distro: ubuntu (like: debian), using package manager: pacman (source: flag)'
PATH="${manager_bin}"
reset_installer_state
DRY_RUN=1
parse_args --pkg-manager pacman
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
flag_output="$(ensure_linux_packages_for_commands curl 2>&1)"
assert_contains_once "${flag_output}" "${flag_decision_line}" "flag decision line"
assert_in_order "${flag_output}" "${flag_decision_line}" "[substrate-install] Installing packages: curl" "flag decision-line ordering"
PATH="${original_path}"

env_decision_line='Detected distro: ubuntu (like: debian), using package manager: dnf (source: env)'
PATH="${manager_bin}"
reset_installer_state
DRY_RUN=1
PKG_MANAGER_ENV_OVERRIDE="dnf"
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
env_output="$(ensure_linux_packages_for_commands curl 2>&1)"
assert_contains_once "${env_output}" "${env_decision_line}" "env decision line"
assert_in_order "${env_output}" "${env_decision_line}" "[substrate-install] Installing packages: curl" "env decision-line ordering"
assert_not_contains "${env_output}" "apt-get update" "env no os_release install command"
PATH="${original_path}"

invalid_flag_output="$(capture_failure_output 2 "invalid flag" run_invalid_flag_case)"
assert_contains "${invalid_flag_output}" "--pkg-manager" "invalid flag source"
assert_contains "${invalid_flag_output}" "apk" "invalid flag value"
assert_contains "${invalid_flag_output}" "Allowed values: apt-get, dnf, yum, pacman, zypper." "invalid flag allowed values"
assert_contains "${invalid_flag_output}" "Re-run with one of the allowed values or remove the invalid override." "invalid flag remediation"
assert_not_contains "${invalid_flag_output}" "Detected distro:" "invalid flag no decision line"

invalid_env_output="$(capture_failure_output 2 "invalid env" run_invalid_env_case)"
assert_contains "${invalid_env_output}" "PKG_MANAGER" "invalid env source"
assert_contains "${invalid_env_output}" "apk" "invalid env value"
assert_contains "${invalid_env_output}" "Allowed values: apt-get, dnf, yum, pacman, zypper." "invalid env allowed values"
assert_contains "${invalid_env_output}" "Re-run with one of the allowed values or remove the invalid override." "invalid env remediation"
assert_not_contains "${invalid_env_output}" "Detected distro:" "invalid env no decision line"

missing_flag_output="$(capture_failure_output 3 "missing flag manager" run_missing_flag_manager_case)"
missing_flag_decision_line='Detected distro: ubuntu (like: debian), using package manager: pacman (source: flag)'
assert_contains "${missing_flag_output}" "--pkg-manager" "missing flag source"
assert_contains "${missing_flag_output}" "pacman" "missing flag value"
assert_contains "${missing_flag_output}" "was not found in PATH" "missing flag path text"
assert_contains "${missing_flag_output}" "Install that manager or rerun with another allowed manager (apt-get, dnf, yum, pacman, zypper)." "missing flag remediation"
assert_contains_once "${missing_flag_output}" "${missing_flag_decision_line}" "missing flag decision line"
assert_in_order "${missing_flag_output}" "${missing_flag_decision_line}" "was not found in PATH" "missing flag decision-line ordering"

missing_env_output="$(capture_failure_output 3 "missing env manager" run_missing_env_manager_case)"
missing_env_decision_line='Detected distro: ubuntu (like: debian), using package manager: dnf (source: env)'
assert_contains "${missing_env_output}" "PKG_MANAGER" "missing env source"
assert_contains "${missing_env_output}" "dnf" "missing env value"
assert_contains "${missing_env_output}" "was not found in PATH" "missing env path text"
assert_contains "${missing_env_output}" "Install that manager or rerun with another allowed manager (apt-get, dnf, yum, pacman, zypper)." "missing env remediation"
assert_contains_once "${missing_env_output}" "${missing_env_decision_line}" "missing env decision line"
assert_in_order "${missing_env_output}" "${missing_env_decision_line}" "was not found in PATH" "missing env decision-line ordering"

no_manager_output="$(capture_failure_output 4 "no supported manager" run_no_manager_case)"
assert_contains "${no_manager_output}" "No supported package manager was detected." "no manager posture"
assert_contains "${no_manager_output}" "Missing prerequisite commands for this installer branch: curl tar." "no manager missing commands"
assert_contains "${no_manager_output}" "Install them manually and rerun." "no manager remediation"
assert_contains "${no_manager_output}" "--pkg-manager <apt-get|dnf|yum|pacman|zypper>" "no manager flag override"
assert_contains "${no_manager_output}" "PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>" "no manager env override"
assert_not_contains "${no_manager_output}" "Detected distro:" "no manager no decision line"
assert_not_contains "${no_manager_output}" "Installing packages:" "no manager no install attempt"

no_manager_without_sudo_output="$(capture_failure_output 4 "no supported manager without sudo" run_no_manager_without_sudo_case)"
assert_contains "${no_manager_without_sudo_output}" "No supported package manager was detected." "no manager without sudo posture"
assert_not_contains "${no_manager_without_sudo_output}" "requires 'sudo'" "no manager without sudo bypasses sudo setup"

decision_line='Detected distro: ubuntu (like: debian), using package manager: apt-get (source: os_release)'
PATH="${manager_bin}"
reset_installer_state
DRY_RUN=1
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${valid_alt}"
decision_output="$(
  {
    ensure_linux_packages_for_commands curl
    ensure_linux_packages_for_commands tar
  } 2>&1
)"
assert_contains_once "${decision_output}" "${decision_line}" "os_release decision line"
assert_in_order "${decision_output}" "${decision_line}" "[substrate-install][dry-run] sudo apt-get update" "os_release decision-line ordering"
PATH="${original_path}"

rm -f "${manager_bin}/pacman" "${manager_bin}/dnf" "${manager_bin}/yum" "${manager_bin}/zypper"
PATH="${manager_bin}"
reset_installer_state
DRY_RUN=1
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${arch_fixture}"
single_path_probe_output="$(ensure_linux_packages_for_commands curl 2>&1)"
path_probe_decision_line='Detected distro: endeavouros (like: arch), using package manager: apt-get (source: path_probe)'
multi_path_probe_warning='Multiple supported package managers found in PATH: apt-get, dnf, yum, zypper; selecting apt-get by fixed probe order (apt-get -> dnf -> yum -> pacman -> zypper). Override with --pkg-manager <apt-get|dnf|yum|pacman|zypper> or PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>.'
assert_contains_once "${single_path_probe_output}" "${path_probe_decision_line}" "single path_probe decision line"
assert_in_order "${single_path_probe_output}" "${path_probe_decision_line}" "[substrate-install] Installing packages: curl" "single path_probe decision-line ordering"
assert_not_contains "${single_path_probe_output}" "${multi_path_probe_warning}" "single path_probe warning omission"
PATH="${original_path}"
make_stub_command "${manager_bin}/dnf"
make_stub_command "${manager_bin}/yum"
make_stub_command "${manager_bin}/zypper"

PATH="${manager_bin}"
reset_installer_state
DRY_RUN=1
SUBSTRATE_INSTALL_OS_RELEASE_PATH="${arch_fixture}"
multi_path_probe_output="$(ensure_linux_packages_for_commands curl 2>&1)"
assert_contains_once "${multi_path_probe_output}" "${multi_path_probe_warning}" "multi path_probe warning"
assert_contains_once "${multi_path_probe_output}" "${path_probe_decision_line}" "multi path_probe decision line"
assert_in_order "${multi_path_probe_output}" "${multi_path_probe_warning}" "${path_probe_decision_line}" "multi path_probe warning ordering"
assert_in_order "${multi_path_probe_output}" "${path_probe_decision_line}" "[substrate-install] Installing packages: curl" "multi path_probe decision-line ordering"
PATH="${original_path}"
make_stub_command "${manager_bin}/pacman"

for wrapper_status in 0 2 3 4; do
  run_wrapper_fixture_case "${wrapper_status}"
done

SUBSTRATE_INSTALL_OS_RELEASE_PATH="${empty_value_fixture}"
resolve_selected_os_release_input
assert_selected "${empty_value_fixture}"
parse_selected_os_release_fields
assert_parsed_fields "${DISTRO_UNKNOWN_SENTINEL}" '`apt-get`'

SUBSTRATE_INSTALL_OS_RELEASE_PATH="relative-os-release"
if resolve_selected_os_release_input; then
  printf '[pkg-manager-detection-smoke] relative path unexpectedly resolved\n' >&2
  exit 1
fi
assert_unavailable
if parse_selected_os_release_fields; then
  printf '[pkg-manager-detection-smoke] unavailable input unexpectedly parsed\n' >&2
  exit 1
fi

SUBSTRATE_INSTALL_OS_RELEASE_PATH="${missing_alt}"
if resolve_selected_os_release_input; then
  printf '[pkg-manager-detection-smoke] missing path unexpectedly resolved\n' >&2
  exit 1
fi
assert_unavailable
if parse_selected_os_release_fields; then
  printf '[pkg-manager-detection-smoke] missing path unexpectedly parsed\n' >&2
  exit 1
fi

SUBSTRATE_INSTALL_OS_RELEASE_PATH="${unreadable_alt}"
if resolve_selected_os_release_input; then
  printf '[pkg-manager-detection-smoke] unreadable path unexpectedly resolved\n' >&2
  exit 1
fi
assert_unavailable
if parse_selected_os_release_fields; then
  printf '[pkg-manager-detection-smoke] unreadable path unexpectedly parsed\n' >&2
  exit 1
fi

SUBSTRATE_INSTALL_OS_RELEASE_PATH="${non_regular_alt}"
if resolve_selected_os_release_input; then
  printf '[pkg-manager-detection-smoke] directory path unexpectedly resolved\n' >&2
  exit 1
fi
assert_unavailable
if parse_selected_os_release_fields; then
  printf '[pkg-manager-detection-smoke] directory path unexpectedly parsed\n' >&2
  exit 1
fi

printf '[pkg-manager-detection-smoke] OK\n' >&2
