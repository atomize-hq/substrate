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

reset_detected_manager() {
  PKG_MANAGER=""
  PKG_MANAGER_SOURCE=""
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

tmpdir="$(mktemp -d -t substrate-pkg-manager-detection.XXXXXX)"
trap 'rm -rf "${tmpdir}"' EXIT

valid_alt="${tmpdir}/valid-os-release"
unreadable_alt="${tmpdir}/unreadable-os-release"
non_regular_alt="${tmpdir}/os-release-dir"
missing_alt="${tmpdir}/missing-os-release"
manager_bin="${tmpdir}/bin"

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
assert_detected_manager "apt-get" ""
PATH="${original_path}"
make_stub_command "${manager_bin}/dnf"
make_stub_command "${manager_bin}/yum"
make_stub_command "${manager_bin}/pacman"
make_stub_command "${manager_bin}/zypper"

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
fallback_output="$(ensure_linux_packages_for_commands curl 2>&1)"
assert_not_contains "${fallback_output}" "Detected distro:" "path_probe decision-line suppression"
PATH="${original_path}"
make_stub_command "${manager_bin}/dnf"
make_stub_command "${manager_bin}/yum"
make_stub_command "${manager_bin}/pacman"
make_stub_command "${manager_bin}/zypper"

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
