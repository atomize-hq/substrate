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
    'mac_resume_exact_publisher_service_file_retirement_v1',
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
for fixed in ('--publisher-service-state-install', '--publisher-service-state-retire'):
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

retire_fn = executor.index('fn execute_closed_mac_publisher_service_state_retirement_v1')
retire_body = executor[retire_fn:]
if retire_body.index('mac_launchctl_bootout_publisher_v1') >= retire_body.index(
        'mac_resume_exact_publisher_service_file_retirement_v1'):
    raise SystemExit('publisher retirement deletes before fixed bootout')
resume_fn = executor.index('fn mac_resume_exact_publisher_service_file_retirement_v1')
resume_end = executor.index('fn execute_closed_mac_publisher_service_state_retirement_v1', resume_fn)
resume_body = executor[resume_fn:resume_end]
if resume_body.index('mac_cas_publisher_service_state_record_v1') >= resume_body.index(
        'fs::remove_file'):
    raise SystemExit('publisher retirement unlink is not preceded by protected cursor CAS')
if 'stdout(Stdio::null())' not in executor or 'stderr(Stdio::null())' not in executor:
    raise SystemExit('launchctl print body is not discarded as non-authoritative')
if 'Some(113)' not in executor or 'Some(113)' not in control:
    raise SystemExit('launchctl print absence is not restricted to exact not-found status 113')
if executor.count('publisher service bootout failed; preserving retirement precommit') < 2:
    raise SystemExit('publisher retirement does not preserve precommit on failed bootout')
if 'publisher service-state predecessor hash chain is invalid' not in common:
    raise SystemExit('publisher service receipt does not validate its exact predecessor chain')
for token in (
        'publisher service receipt does not exact-join its completed bootstrap locator',
        'publisher service receipt does not exact-join its completed bootstrap intent'):
    if token not in executor:
        raise SystemExit(f'publisher service receipt lacks retained authority revalidation: {token}')
if 'fixed_present_count' not in installer or 'partial ambiguous prestate' not in installer:
    raise SystemExit('installer lacks preserving-first fixed-path collision admission')

print('AUX-R3-MAC publisher service-state static regression: PASS')
PY
