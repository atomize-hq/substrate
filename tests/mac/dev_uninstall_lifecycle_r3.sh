#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
UNINSTALLER="${ROOT}/scripts/substrate/dev-uninstall-substrate.sh"
CONTROL="${ROOT}/src/bin/substrate-lifecycle-control.rs"
EXECUTOR="${ROOT}/src/bin/substrate-lifecycle-macos.rs"

bash -n "${UNINSTALLER}"
/bin/bash "${UNINSTALLER}" --help >/dev/null
if grep -Eq 'declare -A|exec \{' "${UNINSTALLER}"; then
  echo "dev uninstaller must remain compatible with the macOS system Bash" >&2
  exit 1
fi

python3 - "${UNINSTALLER}" "${CONTROL}" "${EXECUTOR}" <<'PY'
from pathlib import Path
import sys

uninstaller, control, executor = [Path(path).read_text() for path in sys.argv[1:]]

for token in (
    'MANAGED_MAC_CONTROL_BINARIES_PATH="${MANAGED_STATE_DIR}/mac-control-binaries.txt"',
    'prepare_managed_mac_product_retirement',
    'publisher-install-retire',
    'remove_managed_mac_control_binary_copies',
    'mac-control-binaries.txt',
    'bootstrap-provenance.v1.json',
    'substrate-lifecycle-control',
    'substrate-lifecycle-macos',
):
    if token not in uninstaller:
        raise SystemExit(f'dev uninstaller lacks managed macOS lifecycle token: {token}')

prepare = uninstaller.index('prepare_managed_mac_product_retirement()')
invoke = uninstaller.index('publisher-install-retire', prepare)
remove = uninstaller.index('remove_managed_mac_control_binary_copies', invoke)
ordinary_delete = uninstaller.index('if [[ -d "${SHIMS_DIR}" ]]', invoke)
if not prepare < invoke < remove < ordinary_delete:
    raise SystemExit('dev uninstall does not retire the installed product before deleting prefix files')

prepare_body = uninstaller[prepare:invoke]
for token in (
    'set(lines) != expected_paths',
    'record["selected_host_prefix"] != prefix',
    'record["host_context_commitment"] != host_context_commitment',
    'control_authority["artifact_sha256"]',
    'executor_image["artifact_sha256"]',
    'launch_daemon_plist_sha256',
    'fixed publisher helper does not match retained provenance',
):
    if token not in prepare_body:
        raise SystemExit(f'macOS retirement preparation lacks fail-closed binding: {token}')

if '"publisher-install-retire" =>' not in control:
    raise SystemExit('installed lifecycle control lacks product retirement dispatch')
if '--publisher-install-retire-fd' not in control or '--publisher-install-retire-fd' not in executor:
    raise SystemExit('installed lifecycle product lacks closed retirement FD3 route')

if '.arg("3")' not in control or 'immediate EOF is the only input' not in control:
    raise SystemExit('complete retirement is not carried over the existing no-caller-data FD3 boundary')

classifier_start = control.index('mod mac_publisher_service_state_classifier_ffi_v1')
classifier_end = control.index('\n#[cfg(target_os = "macos")]\nfn execute_closed_mac_publisher_service_state_v1',
                               classifier_start)
classifier = control[classifier_start:classifier_end]
for token in (
    '"/Library/Keychains/System.keychain"',
    '"com.substrate.lifecycle.v1"',
    '"mac-publisher-service-state.v1"',
    'SecKeychainOpen',
    'SecKeychainGetPath',
    'returned System Keychain path drifted from the fixed store',
    '(kSecClass, kSecClassGenericPassword)',
    '(kSecAttrService, service)',
    '(kSecAttrAccount, account)',
    '(kSecMatchSearchList, search_list)',
    '(kSecUseAuthenticationUI, kSecUseAuthenticationUIFail)',
    '(kSecReturnAttributes, kCFBooleanTrue)',
    '(kSecMatchLimit, kSecMatchLimitAll)',
    'CFArrayGetCount(result) != 1',
    'ERR_SEC_ITEM_NOT_FOUND',
):
    if token not in classifier and token not in control[:classifier_start]:
        raise SystemExit(f'ordinary fixed-record classifier lacks exact fence: {token}')
for forbidden in ('kSecReturnData', 'kSecValueData', 'SecItemDelete', 'SecItemAdd',
                  'SecItemUpdate', 'std::env::args', 'std::env::var'):
    if forbidden in classifier:
        raise SystemExit(f'ordinary fixed-record classifier has forbidden input/data/effect: {forbidden}')

product_start = control.index('fn execute_closed_mac_publisher_install_retirement_v1')
product_end = control.index('\n#[cfg(target_os = "macos")]\nfn observe_fixed_mac_publisher_service_absent_v1',
                            product_start)
product = control[product_start:product_end]
for earlier, later in (
    ('classify_fixed_record', 'select_mac_publisher_product_retirement_route_v1'),
    ('MacPublisherProductRetirementRouteV1::PrivilegedRetirement',
     'execute_closed_mac_publisher_service_state_v1'),
    ('MacPublisherProductRetirementRouteV1::ValidateFixedAbsence',
     'observe_fixed_mac_publisher_product_absent_v1'),
    ('observe_fixed_mac_publisher_product_absent_v1', '"already_uninstalled"'),
):
    if product.index(earlier) >= product.index(later):
        raise SystemExit(f'ordinary uninstall route ordering is unsafe: {earlier} before {later}')

closed_start = control.index('fn execute_closed_mac_publisher_service_state_v1')
closed_end = control.index('\n#[cfg(target_os = "macos")]\nfn execute_closed_mac_publisher_service_state_install_v1',
                           closed_start)
if 'already_uninstalled' in control[closed_start:closed_end]:
    raise SystemExit('privileged closed result still authorizes already_uninstalled')

absence_start = control.index('fn observe_fixed_mac_publisher_product_absent_v1')
absence_end = control.index('\nfn usage_error_v1', absence_start)
absence = control[absence_start:absence_end]
for token in ('observe_fixed_mac_publisher_service_absent_v1',
              'MAC_PUBLISHER_HELPER_PATH_V1',
              '/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist',
              '/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json',
              'ErrorKind::NotFound', 'fixed publisher artifact remains after terminal commit'):
    if token not in absence:
        raise SystemExit(f'record absence lacks complete fixed-product absence proof: {token}')

for token in ('fixed_present = [path.exists() or path.is_symlink()',
              'if any(fixed_present) and not all(fixed_present)',
              'if all(fixed_present)', 'response["status"] == "already_uninstalled"',
              'response["record_sha256"] is None'):
    if token not in uninstaller:
        raise SystemExit(f'dev uninstaller lacks terminal retry containment: {token}')

print('AUX-R3-MAC dev uninstall lifecycle static regression: PASS')
PY
