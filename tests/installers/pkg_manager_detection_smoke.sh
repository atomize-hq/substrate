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

tmpdir="$(mktemp -d -t substrate-pkg-manager-detection.XXXXXX)"
trap 'rm -rf "${tmpdir}"' EXIT

valid_alt="${tmpdir}/valid-os-release"
unreadable_alt="${tmpdir}/unreadable-os-release"
non_regular_alt="${tmpdir}/os-release-dir"
missing_alt="${tmpdir}/missing-os-release"

cat > "${valid_alt}" <<'EOF'
ID=ubuntu
ID_LIKE=debian
EOF

parser_fixture="${tmpdir}/parser-os-release"
missing_id_fixture="${tmpdir}/missing-id-os-release"

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

cat > "${unreadable_alt}" <<'EOF'
ID=fedora
EOF
chmod 000 "${unreadable_alt}"
mkdir -p "${non_regular_alt}"

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
