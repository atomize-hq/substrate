#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALLER="${ROOT}/scripts/substrate/dev-install-substrate.sh"
CONTROL="${ROOT}/src/bin/substrate-lifecycle-control.rs"
EXECUTOR="${ROOT}/src/bin/substrate-lifecycle-macos.rs"
COMMON="${ROOT}/crates/common/src/managed_artifact.rs"

python3 - "${INSTALLER}" "${CONTROL}" "${EXECUTOR}" "${COMMON}" <<'PY'
from pathlib import Path
import sys

installer, control, executor, common = [Path(path).read_text() for path in sys.argv[1:]]

required_executor = (
    'execute_closed_mac_publisher_service_state_install_v1',
    'execute_closed_mac_publisher_service_state_retirement_v1',
    'execute_closed_mac_publisher_product_retirement_v1',
    'mac_revalidate_publisher_product_retirement_v1',
    'retirement_delete_cursor',
    'bootstrap_attempt_locator_account',
    'mac_revalidate_publisher_service_record_authority_v1',
    '"/bin/launchctl"',
    '"bootstrap"',
    '"bootout"',
    '"system"',
    '"/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist"',
    '"system/com.substrate.lifecycle.publisher.v1"',
    'canonical_mac_publisher_service_state_record_v1',
)
for token in required_executor:
    if token not in executor:
        raise SystemExit(f"missing closed publisher service-state executor token: {token}")

for forbidden in ('"kickstart"', 'launchctl load', 'launchctl unload'):
    if forbidden in executor or forbidden in installer:
        raise SystemExit(f"forbidden launchd mutation retained: {forbidden}")

if 'launchctl' in installer:
    raise SystemExit('canonical installer contains raw shell launchctl mutation')

required_closed_control = (
    'Command::new("/usr/bin/sudo")',
    '.arg("-C")',
    '.arg("4")',
    'libc::shutdown(retained.as_raw_fd(), libc::SHUT_WR)',
    'immediate EOF is the only input',
    'publisher service-state operation accepts exactly retained descriptor 3',
    'publisher service-state peer sent forbidden caller data',
    'publisher-install-retire',
    '--publisher-install-retire-fd',
)
for token in required_closed_control:
    if token not in control and token not in executor:
        raise SystemExit(f'missing closed no-input service-state boundary: {token}')

install_call = 'publisher-service-state-install'
if install_call not in installer or install_call not in control:
    raise SystemExit('installer/control lacks the fixed local service-state install route')
stage_one = installer.index('direct publisher-bootstrap returned an invalid Stage-1 result')
service_state = installer.index(install_call, stage_one)
first_xpc = installer.index('"${LIMA_WARM}"', service_state)
if not stage_one < service_state < first_xpc:
    raise SystemExit('fixed service-state install is not between signed Stage-1 and first XPC Create')

main_dispatch = executor.index('fn main()')
stdin_read = executor.index('read_to_end(&mut input)', main_dispatch)
for fixed in ('--publisher-service-state-install', '--publisher-service-state-retire',
              '--publisher-install-retire'):
    offset = executor.index(fixed, main_dispatch)
    if offset > stdin_read:
        raise SystemExit(f'{fixed} is not dispatched before ordinary stdin handling')

install_fn = executor.index('fn execute_closed_mac_publisher_service_state_install_v1')
install_end = executor.index('fn mac_launchctl_bootout_publisher_v1', install_fn)
install_body = executor[install_fn:install_end]
for earlier, later in (
    ('mac_cas_publisher_service_state_record_v1', 'mac_launchctl_bootstrap_publisher_v1'),
    ('mac_launchctl_bootstrap_publisher_v1', 'mac_observe_publisher_registration_v1'),
    ('mac_observe_publisher_registration_v1', 'MacPublisherServiceStatePhaseV1::Installed'),
):
    if install_body.index(earlier) >= install_body.rindex(later):
        raise SystemExit(f'publisher install ordering violated: {earlier} before {later}')

retire_fn = executor.index('fn execute_closed_mac_publisher_product_retirement_v1')
retire_end = executor.index('\n#[cfg(target_os = "macos")]\nfn execute_closed_mac_publisher_service_state_retirement_v1', retire_fn)
retire_body = executor[retire_fn:retire_end]
for token in (
        'mac_revalidate_publisher_product_retirement_v1(&current)',
        'mac_publisher_retirement_effect_v1(current.retirement_delete_cursor)',
        'mac_resume_publisher_bootout_v1',
        'mac_resume_publisher_keychain_item_retirement_v1',
        'mac_resume_publisher_signer_retirement_v1',
        'mac_resume_publisher_file_retirement_v1',
        'mac_cas_publisher_service_state_record_v1',
        'mac_keychain_delete_item_exact_under_guard_v1'):
    if token not in retire_body:
        raise SystemExit(f'publisher product retirement lacks cursor-bound step: {token}')
if retire_body.index('mac_resume_publisher_bootout_v1') >= retire_body.index(
        'mac_resume_publisher_keychain_item_retirement_v1'):
    raise SystemExit('publisher retirement does not order service bootout before Keychain state')
terminal = retire_body[retire_body.index('MacPublisherRetirementEffectV1::TerminalServiceState'):]
delete = terminal.index('mac_keychain_delete_item_exact_under_guard_v1')
terminal_arm = terminal[:terminal.index('\n            }', delete)]
if 'return Ok(response)' not in terminal_arm[delete:]:
    raise SystemExit('terminal publisher record deletion does not return immediately')
if any(token in terminal_arm[delete:] for token in ('mac_keychain_compare_and_swap_item_v1',
                                                     'retire_mac_system_keychain_software_signer_v1',
                                                     'fs::remove_file', 'mac_launchctl_bootout_publisher_v1')):
    raise SystemExit('publisher retirement performs an effect after terminal record deletion')
if 'stdout(Stdio::null())' not in executor or 'stderr(Stdio::null())' not in executor:
    raise SystemExit('launchctl print body is not discarded as non-authoritative')
if 'Some(113)' not in executor or 'Some(113)' not in control:
    raise SystemExit('launchctl print absence is not restricted to exact not-found status 113')
if 'publisher bootout did not reach exact absence' not in executor:
    raise SystemExit('publisher retirement does not fail closed on bootout uncertainty')
if 'publisher service-state predecessor hash chain is invalid' not in common:
    raise SystemExit('publisher service receipt does not validate its exact predecessor chain')
for token in (
        'publisher service receipt does not exact-join its completed bootstrap locator',
        'publisher service receipt does not exact-join its completed bootstrap intent'):
    if token not in executor:
        raise SystemExit(f'publisher service receipt lacks retained authority revalidation: {token}')

keychain_retire_start = executor.index('fn mac_collect_exact_publisher_retirement_items_v1')
keychain_retire = executor[keychain_retire_start:retire_end]
for token in (
        'mac_verified_control_admission_v1',
        'bootstrap_attempt_locator_account',
        'mac_keychain_delete_item_exact_v1',
        'publisher retirement Keychain item disappeared before authorization',
        'publisher retirement signer disappeared before authorization'):
    if token not in keychain_retire:
        raise SystemExit(f'complete product retirement lacks exact retained authority: {token}')
attest_start = executor.index('fn mac_attest_closed_service_state_fd3_v1')
attest_end = executor.index('\n#[cfg(not(target_os = "macos"))]', attest_start)
attest = executor[attest_start:attest_end]
if 'publisher retirement has no authenticated service-state record' not in attest:
    raise SystemExit('privileged product retirement does not reject absent service state')
if 'peer.uid == 0' in attest or 'already_uninstalled' in executor:
    raise SystemExit('privileged product retirement retains an absent-record success authority')
if 'fixed_present_count' not in installer or 'partial ambiguous prestate' not in installer:
    raise SystemExit('installer lacks preserving-first fixed-path collision admission')

print('AUX-R3-MAC publisher service-state static regression: PASS')
PY
