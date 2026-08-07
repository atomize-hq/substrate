#!/usr/bin/env bash
# Non-native A1.1d-5R3-MAC contract fixture. It uses only a disposable stdin mock.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LIFECYCLE="${REPO_ROOT}/scripts/mac/lima-lifecycle.sh"
WARM="${REPO_ROOT}/scripts/mac/lima-warm.sh"
STOP="${REPO_ROOT}/scripts/mac/lima-stop.sh"
SOCKET_UNIT="${REPO_ROOT}/scripts/mac/lima/units/substrate-world-service.socket"
MAC_CLIENT="${REPO_ROOT}/crates/shell/src/execution/managed_lifecycle/macos_client.rs"
MAC_EXECUTOR="${REPO_ROOT}/src/bin/substrate-lifecycle-macos.rs"
MAC_PLIST="${REPO_ROOT}/scripts/mac/com.substrate.lifecycle.publisher.v1.plist"

fail() {
    printf 'lifecycle_r3.sh: %s\n' "$*" >&2
    exit 1
}

require_text() {
    local path="$1" text="$2"
    grep -Fq -- "${text}" "${path}" || fail "${path#"${REPO_ROOT}/"} is missing ${text}"
}

for function_name in \
    main MacManagedArtifactExecutorV1 open_mac_lifecycle_capsule_v1 join_mac_role_identity_v1 \
    execute_mac_managed_action_v1 restore_mac_managed_role_v1 publish_mac_action_receipt_v1 \
    MacLifecyclePublisherServiceV1 open_system_keychain_protected_state_v1 \
    compare_and_swap_mac_publisher_protected_state_v1 bootstrap_mac_publisher_v1 \
    export_mac_p256_spki_der_v1 normalize_mac_p256_signature_p1363_low_s_v1 \
    resume_mac_publisher_bootstrap_v1 run_mac_xpc_publisher_v1 accept_mac_xpc_connection_v1 \
    attest_mac_xpc_audit_token_v1 verify_mac_control_designated_requirement_v1 \
    handle_mac_publisher_request_v1 issue_lima_guest_pairing_ticket_v1 \
    consume_lima_guest_pairing_ticket_v1 open_mac_guest_pairing_record_v1 \
    compare_and_swap_mac_guest_pairing_record_v1 prove_lima_guest_reservation_unused_v1 \
    commit_lima_guest_reservation_unused_acknowledgement_v1 retire_mac_test_publisher_v1 \
    publish_lima_stage_one_intent_v1 attach_lima_stage_one_machine_identity_v1 \
    close_lima_stage_one_intent_v1 mac_xpc_listener_ffi_v1 mac_audit_token_ffi_v1 \
    mac_security_key_ffi_v1 mac_keychain_anchor_ffi_v1 mac_atomic_file_ffi_v1; do
    require_text "${MAC_EXECUTOR}" "${function_name}"
done
for function_name in bootstrap_publisher_v1 submit_publisher_request_v1 open_mac_xpc_channel_v1 \
    attest_mac_publisher_response_v1 issue_guest_publisher_pairing_ticket_v1; do
    require_text "${MAC_CLIENT}" "${function_name}"
done
if grep -Eq 'SUBSTRATE_(LIFECYCLE_TEST_MODE|MAC_LIFECYCLE_EXECUTOR|MAC_NATIVE_EVIDENCE)' "${LIFECYCLE}" "${MAC_EXECUTOR}"; then
    fail 'production MAC lifecycle introduces an ambient executor or evidence selector'
fi
require_text "${MAC_CLIENT}" 'com.substrate.lifecycle.publisher.v1'
require_text "${MAC_CLIENT}" 'xpc_connection_send_message_with_reply_sync'
if grep -Fq 'Command::new' "${MAC_CLIENT}"; then
    fail 'macOS publisher client retains a direct helper subprocess relay'
fi
require_text "${MAC_EXECUTOR}" '"run-publisher"'
require_text "${MAC_EXECUTOR}" 'run_mac_xpc_publisher_v1(&service)'
require_text "${MAC_EXECUTOR}" 'xpc_connection_get_audit_token'
require_text "${MAC_EXECUTOR}" 'dispatch_peer_request(peer, event)'
require_text "${MAC_EXECUTOR}" '"audit_token_bound": true'
require_text "${MAC_PLIST}" '<key>MachServices</key>'
require_text "${MAC_PLIST}" '<string>run-publisher</string>'
require_text "${MAC_PLIST}" '<key>com.substrate.lifecycle.publisher.v1</key>'

python3 - "${MAC_EXECUTOR}" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text()
peer_dispatch = source.index('unsafe fn dispatch_peer_request(')
peer_token = source.index('let audit_token = mac_audit_token_ffi_v1(peer)?;', peer_dispatch)
accepted = source.index('accept_mac_xpc_connection_v1(&service, &audit_token)?;', peer_dispatch)
operation = source.index('let operation = xpc_dictionary_get_string(', peer_dispatch)
if not peer_token < accepted < operation:
    raise SystemExit('XPC publisher reads a request before binding the accepted peer audit token')
listener_peer = source.index('let peer = (*(block as *mut PeerBlockV1)).peer;')
listener_dispatch = source.index('let result = dispatch_peer_request(peer, event);', listener_peer)
if listener_peer > listener_dispatch:
    raise SystemExit('XPC publisher does not dispatch the listener-owned peer')
audit_helper = source[source.index('pub fn attest_mac_xpc_audit_token_v1'):source.index('pub fn verify_mac_control_designated_requirement_v1')]
if 'std::process::id' in audit_helper:
    raise SystemExit('XPC audit attestation falls back to the publisher process PID')

def accepts_peer_audit_token(words):
    return any(words) and words[5] != 0

assert not accepts_peer_audit_token([0] * 8)
assert not accepts_peer_audit_token([1, 0, 0, 0, 0, 0, 0, 0])
assert accepts_peer_audit_token([1, 0, 0, 0, 0, 501, 0, 0])
PY

python3 - "${MAC_EXECUTOR}" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text()

def body(name, following):
    start = source.index(name)
    end = source.index(following, start)
    return source[start:end]

for literal in (
    'MAC_KEYCHAIN_SERVICE_V1: &str = "com.substrate.lifecycle.v1"',
    'MAC_CONTROL_ADMISSION_ACCOUNT_V1: &str = "mac-control-admission-authority.v1"',
    'kSecUseSystemKeychain',
    'kSecAttrIsExtractable',
    '!= kCFBooleanFalse',
    'SecKeyCopyExternalRepresentation(public_key',
):
    if literal not in source:
        raise SystemExit(f'R4 System Keychain foundation is missing {literal}')

unscoped_open = body(
    'pub fn open_system_keychain_protected_state_v1',
    '/// Compare-and-swap protected state',
)
if 'exact scope-bound System Keychain account' not in unscoped_open or 'fs::read' in unscoped_open:
    raise SystemExit('R4 protected-state open is not scope-bound System Keychain only')

listener = body('unsafe extern "C" fn listener_event', 'unsafe extern "C" fn peer_event')
admission = listener.index('mac_verified_control_peer_requirement_v1()')
peer_requirement = listener.index('xpc_connection_set_peer_code_signing_requirement')
if admission > peer_requirement:
    raise SystemExit('R4 XPC listener sets peer requirement before loading fixed Keychain authority')
authority_validator = Path(sys.argv[1].replace('src/bin/substrate-lifecycle-macos.rs', 'crates/common/src/managed_artifact.rs')).read_text()
authority_start = authority_validator.index('pub fn validate_mac_publisher_control_authority_v1')
authority_end = authority_validator.index('\n/// Encode the fixed macOS XPC-control admission record', authority_start)
authority = authority_validator[authority_start:authority_end]
if ('CONTROL_REQUIREMENT_PREFIX' not in authority
        or 'and cdhash H\\"' not in authority
        or 'control_cdhash.len() != 40' not in authority):
    raise SystemExit('R4 XPC authority does not pin the exact lifecycle-control CodeDirectory hash')
if ('LISTENER_BLOCK_DESCRIPTOR_V1' not in source
        or 'size: std::mem::size_of::<BlockV1>()' not in source
        or 'PEER_BLOCK_DESCRIPTOR_V1' not in source
        or 'size: std::mem::size_of::<PeerBlockV1>()' not in source):
    raise SystemExit('R4 XPC Block descriptors do not match their captured layout')
if ('xpc_connection_set_peer_code_signing_requirement(event, requirement.as_ptr()) != 0'
        not in listener
        or 'xpc_connection_cancel(event)' not in listener):
    raise SystemExit('R4 XPC listener activates a peer after peer-requirement setup failure')

dispatch = body('unsafe fn dispatch_peer_request', '\n    }\n}')
audit = dispatch.index('let audit_token = mac_audit_token_ffi_v1(peer)?;')
accepted = dispatch.index('accept_mac_xpc_connection_v1(&service, &audit_token)?;')
control_image = dispatch.index('attest_mac_xpc_control_image_v1(event, &authority)?;')
decode = dispatch.index('xpc_dictionary_get_string(event')
if not audit < accepted < control_image < decode:
    raise SystemExit('R4 XPC request decodes bytes before actual peer and exact artifact admission')
if 'proc_pidpath' in source:
    raise SystemExit('R4 XPC control-image admission resolves a mutable executable path by PID')
for required in ('SecRequirementCreateWithString', 'SecCodeCreateWithXPCMessage', 'SecCodeCheckValidity'):
    if required not in source:
        raise SystemExit(f'R4 XPC control-image admission is missing {required}')

issuer = body('pub fn issue_lima_guest_pairing_ticket_v1', '/// Consume one signed ticket')
stage_open = issuer.index('open_mac_lima_guest_pairing_stage_one_record_v1')
state_open = issuer.index('read_system_keychain_protected_state_for_scope_unbound_v1')
stage_validate = issuer.index('validate_mac_lima_guest_pairing_stage_one_record_for_issue_v1')
key_binding = issuer.index('validate_system_keychain_protected_state_key_binding_v1')
if not stage_open < state_open < stage_validate < key_binding:
    raise SystemExit('R4 ticket issuer does not validate Stage-1 joins before binding the signing key')
if 'R5 typed issue contract' not in issuer:
    raise SystemExit('R4 ticket issuer does not fail closed without the separately authorized typed input')

for forbidden in (
    'lima-stdio-v1',
    'GuestPublisherPairingGuestChannelV1',
    'GuestPublisherPairingIssueRequestV1',
    'MacPublisherXpcOperationV1',
):
    if forbidden in source:
        raise SystemExit(f'R4 source introduces forbidden channel or generic issue surface: {forbidden}')
PY

python3 - "${MAC_EXECUTOR}" "${MAC_CLIENT}" "${REPO_ROOT}/crates/world-mac-lima/src/forwarding.rs" <<'PY'
from pathlib import Path
import sys

executor, client, forwarding = map(lambda value: Path(value).read_text(), sys.argv[1:])
carrier_start = executor.index('fn validate_mac_carrier_mapping_join_v1(')
carrier_end = executor.index('\n/// Execute only an XPC-authorized', carrier_start)
carrier = executor[carrier_start:carrier_end]
if 'Sha256::digest(carrier_commitment_input.as_bytes())' not in carrier:
    raise SystemExit('privileged mapped action does not recompute the canonical carrier commitment')
if carrier.index('calculated_carrier_commitment') > carrier.index('if carrier_values[0] !='):
    raise SystemExit('carrier commitment comparison is not reached before privileged action validation')

bootstrap_start = client.index('pub fn bootstrap_publisher_v1(')
bootstrap_end = client.index('\npub fn submit_publisher_request_v1(', bootstrap_start)
if 'attest_mac_publisher_response_v1(&response)?' not in client[bootstrap_start:bootstrap_end]:
    raise SystemExit('bootstrap client accepts an unattested publisher response')

socket_start = forwarding.index('fn remove_exact_mapped_ssh_socket_v1(')
socket_end = forwarding.index('\nfn restore_exact_mapped_known_hosts_entry_v1(', socket_start)
socket_cleanup = forwarding[socket_start:socket_end]
if 'std::fs::remove_file' in socket_cleanup:
    raise SystemExit('mapped SSH teardown path-unlinks after a separately checked identity')
known_start = socket_end
known_end = forwarding.index('\nfn mapped_ssh_attempt_v1(', known_start)
known_cleanup = forwarding[known_start:known_end]
if 'custom_flags(O_NOFOLLOW_V1)' not in known_cleanup or 'file.write_all' not in known_cleanup:
    raise SystemExit('known-host restoration does not write through a verified no-follow descriptor')
capture_start = forwarding.index('pub fn record_mapped_known_hosts_entry_v1(')
capture_end = forwarding.index('\n/// Restore the recorded A-local', capture_start)
capture = forwarding[capture_start:capture_end]
if 'custom_flags(O_NOFOLLOW_V1 | O_NONBLOCK_V1)' not in capture or 'file.read_to_end' not in capture:
    raise SystemExit('known-host snapshot does not capture bytes and identity through one no-follow descriptor')
if 'std::fs::read' in capture:
    raise SystemExit('known-host snapshot still reads by pathname after an identity observation')
PY

for function_name in \
    load_mapped_lifecycle_v1 invoke_mac_lifecycle_executor install_mapped_lima_state_v1 \
    restore_mapped_lima_state_v1 install_mac_publisher_v1 bootstrap_lima_guest_publisher_v1 \
    retain_lima_guest_bootstrap_channel_v1 display_lima_guest_pairing_challenge_v1 \
    join_lima_guest_bootstrap_transcript_v1 record_lima_guest_pre_state_v1 \
    install_exact_guest_artifacts_v1 persist_lima_guest_unused_proof_v1 \
    acknowledge_lima_guest_unused_proof_v1 restore_lima_guest_state_v1 \
    retire_lima_guest_test_publisher_v1 main; do
    require_text "${LIFECYCLE}" "${function_name}()"
done

if grep -Eq '^PartOf=' "${SOCKET_UNIT}"; then
    fail 'socket unit retains a service propagation relationship'
fi
if grep -Eq '(^|[^[:alnum:]_])limactl([[:space:]]|$)' "${STOP}"; then
    fail 'lima-stop retains a direct destructive Lima primitive'
fi
if grep -Fq 'list substrate' "${STOP}"; then
    fail 'lima-stop retains a default instance selection'
fi

python3 - "${WARM}" <<'PY'
from pathlib import Path
import re
import sys

source = Path(sys.argv[1]).read_text()
required = (
    'destroy_vm', 'ensure_vm_ready', 'ensure_substrate_group', 'stage_workspace',
    'install_agent_from_host', 'install_cli_from_host', 'install_gateway_from_host',
    'build_missing_components_inside_vm', 'install_guest_binaries',
    'bootstrap_guest_private_home', 'write_systemd_units', 'enable_socket_activation',
    'write_layout_sentinel', 'configure_guest',
)
for name in required:
    match = re.search(rf'(?ms)^{name}\(\) \{{\n.*?(?=^[A-Za-z_][A-Za-z0-9_]*\(\) \{{|^if \[\[|\Z)', source)
    if not match:
        raise SystemExit(f'missing named lifecycle function {name}')
    body = match.group(0)
    if name == 'build_missing_components_inside_vm':
        for forbidden in ('apt-get', 'rustup', 'cargo build', 'fix_dns'):
            if forbidden in body:
                raise SystemExit(f'{name} retains prohibited {forbidden}')
    elif name != 'configure_guest' and name not in ('destroy_vm', 'ensure_vm_ready', 'stage_workspace'):
        if 'lima-lifecycle.sh' not in body and 'tombstoned' not in body:
            raise SystemExit(f'{name} is not delegated to the mapped executor')
    if name in ('destroy_vm', 'stage_workspace', 'configure_guest'):
        if 'lima-lifecycle.sh' not in body:
            raise SystemExit(f'{name} does not use the mapped executor')
    if name == 'ensure_vm_ready':
        if 'lima-lifecycle.sh' not in body or 'ensure-vm-ready' not in body:
            raise SystemExit('ensure_vm_ready does not delegate the selected R2 Stage-1 path')
        if 'LIMA_STAGE_ONE_AUTHORIZATION_V1' not in body:
            raise SystemExit('ensure_vm_ready does not bind LimaStageOneAuthorizationV1')
    if re.search(r'\blimactl\b', body):
        raise SystemExit(f'{name} retains direct Lima mutation')

configure = re.search(r'(?ms)^configure_guest\(\) \{\n.*?(?=^if \[\[)', source).group(0)
if configure.count('lima-lifecycle.sh') != 1:
    raise SystemExit('configure_guest must make one mapped lifecycle request')
if 'OBSERVED_PLATFORM_MAPPING_V1' not in configure:
    raise SystemExit('configure_guest does not bind the observed selected mapping')
PY

workdir="$(mktemp -d)"
trap 'rm -rf "${workdir}"' EXIT
mock="${workdir}/executor"
test_lifecycle="${workdir}/lima-lifecycle.sh"
received="${workdir}/received.json"
called="${workdir}/called"
cat >"${mock}" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == "lima-action" ]]
touch "${SUBSTRATE_TEST_CALLED:?}"
cat >"${SUBSTRATE_TEST_RECEIVED:?}"
printf '{"xpc_attestation":{"mach_service":"com.substrate.lifecycle.publisher.v1","caller_pid":0}}\n'
MOCK
chmod +x "${mock}"
python3 - "${LIFECYCLE}" "${test_lifecycle}" "${mock}" <<'TEST_COPY'
from pathlib import Path
import sys
source, destination, mock = map(Path, sys.argv[1:])
text = source.read_text()
needle = '/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1'
if text.count(needle) != 1:
    raise SystemExit('unexpected fixed executor literal count')
destination.write_text(text.replace(needle, str(mock)))
destination.chmod(0o755)
TEST_COPY

VALUES="$(python3 - <<'FIXTURE_VALUES'
import base64
import hashlib
import json

prefix = '/tmp/substrate-r3'
def b64(value):
    return base64.urlsafe_b64encode(value.encode()).rstrip(b'=').decode()
carrier_lines = [
    'domain=substrate.install_bootstrap_context', 'version=1',
    f'selected_host_prefix={b64(prefix)}', f'host_substrate_home={b64(prefix)}',
    f'host_substrate_root={b64(prefix)}', 'principal_kind=unix',
    f'principal_account={b64("fixture")}', 'principal_uid=501',
]
commitment = hashlib.sha256(('\n'.join(carrier_lines) + '\n').encode()).hexdigest()
carrier = '\n'.join(carrier_lines + [f'host_context_commitment={commitment}']) + '\n'
mapping = '\n'.join([
    'domain=substrate.platform_bootstrap_mapping', 'version=1',
    f'host_context_commitment={commitment}', 'platform_kind=lima',
    f'instance_name={b64("substrate")}', 'guest_machine_id=0123456789abcdef0123456789abcdef',
    f'host_platform_control_root={b64("/Users/fixture/.lima")}',
    f'realized_substrate_home={b64("/home/fixture/.substrate")}',
    f'realized_principal_account={b64("fixture")}', 'realized_principal_uid=501',
    'transport_kind=lima', f'transport_host={b64(prefix + "/sock/agent.sock")}',
    f'transport_guest_socket={b64("/run/substrate.sock")}',
]) + '\n'
evidence = {
    'schema_owner': 'substrate.executor-build-evidence', 'schema_version': 1,
    'source_commit': 'a' * 40, 'source_tree': 'b' * 40, 'source_ref': 'refs/heads/mock',
    'artifact_sha256': '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
    'artifact_identity': 'mock-host-executor', 'target_triple': 'aarch64-apple-darwin',
    'tool_versions': {'fixture': 'non-executing'},
}
publisher_request = {
    'host_context_commitment': commitment,
    'platform_mapping_commitment': 'c' * 64,
    'scope_id': '018f0000-0000-7000-8000-000000000001',
    'current_anchor_counter': 1,
    'current_anchor_sha256': 'c' * 64,
    'manifest_generation': 1,
    'manifest_sha256': 'd' * 64,
    'role': 'mac.lima.instance',
    'action': 'create',
    'object_identity': {
        'scope_id': '018f0000-0000-7000-8000-000000000001',
        'parent_identity': 'fixture-parent',
        'name_identity': 'fixture-name',
        'physical_identity': 'fixture-physical',
    },
    'requester_principal': 'fixture',
    'attempt_nonce': 'fixture-attempt',
    'expected_executor_build': {
        'source_commit': evidence['source_commit'],
        'source_tree': evidence['source_tree'],
        'source_ref': evidence['source_ref'],
        'target_triple': evidence['target_triple'],
        'artifact_sha256': evidence['artifact_sha256'],
        'artifact_path': '/fixture/substrate-lifecycle-macos',
    },
}
stage_one = {
    'schema_owner': 'substrate.lima-stage-one-authorization',
    'schema_version': 1,
    'host_context_commitment': commitment,
    'lima_control_root_identity': '/Users/fixture/.lima',
    'instance_name': 'substrate',
    'profile_sha256': 'e' * 64,
    'expected_absent': True,
    'source_commit': evidence['source_commit'],
    'source_tree': evidence['source_tree'],
    'source_ref': evidence['source_ref'],
    'executor_receipt_sha256': 'f' * 64,
    'requester_principal': 'fixture',
    'attempt_id': '018f0000-0000-7000-8000-000000000002',
    'nonce': '018f0000-0000-7000-8000-000000000003',
    'expires_at_unix_ns': 999999999999999999,
    'signature': {'algorithm': 'fixture', 'public_key': 'fixture', 'signature': 'fixture'},
}
print(b64(carrier))
print(b64(mapping))
print(json.dumps(evidence, separators=(',', ':')))
print(json.dumps(publisher_request, separators=(',', ':')))
print(json.dumps(stage_one, separators=(',', ':')))
FIXTURE_VALUES
)"
CARRIER="$(printf '%s\n' "${VALUES}" | sed -n '1p')"
MAPPING="$(printf '%s\n' "${VALUES}" | sed -n '2p')"
EVIDENCE="$(printf '%s\n' "${VALUES}" | sed -n '3p')"
PUBLISHER_REQUEST="$(printf '%s\n' "${VALUES}" | sed -n '4p')"
STAGE_ONE="$(printf '%s\n' "${VALUES}" | sed -n '5p')"

# Missing evidence must stop before even a disposable-copy mock can be invoked.
if SUBSTRATE_TEST_CALLED="${called}" \
    SUBSTRATE_TEST_RECEIVED="${received}" \
    "${test_lifecycle}" stop \
        --install-prefix /tmp/substrate-r3 \
        --install-bootstrap-context-v1 "${CARRIER}" \
        --platform-bootstrap-mapping-v1 "${MAPPING}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST}" \
        --lima-stage-one-authorization-v1 "${STAGE_ONE}" >/dev/null 2>&1; then
    fail 'lifecycle accepted missing ExecutorBuildEvidenceV1'
fi
[[ ! -e "${called}" ]] || fail 'missing evidence reached the mock executor'

# Replaced or malformed authority must be rejected before any executor dispatch.
for invalid_kind in mapping evidence publisher stage-one; do
    rm -f "${called}" "${received}"
    invalid_mapping="${MAPPING}"
    invalid_evidence="${EVIDENCE}"
    invalid_publisher_request="${PUBLISHER_REQUEST}"
    invalid_stage_one="${STAGE_ONE}"
    if [[ "${invalid_kind}" == mapping ]]; then
        invalid_mapping='not-base64url'
    elif [[ "${invalid_kind}" == evidence ]]; then
        invalid_evidence='{"schema_owner":"wrong"}'
    elif [[ "${invalid_kind}" == publisher ]]; then
        invalid_publisher_request='{"host_context_commitment":"wrong"}'
    else
        invalid_stage_one='{"schema_owner":"wrong"}'
    fi
    if SUBSTRATE_TEST_CALLED="${called}" \
        SUBSTRATE_TEST_RECEIVED="${received}" \
        "${test_lifecycle}" stop \
            --install-prefix /tmp/substrate-r3 \
            --install-bootstrap-context-v1 "${CARRIER}" \
            --platform-bootstrap-mapping-v1 "${invalid_mapping}" \
            --executor-build-evidence-v1 "${invalid_evidence}" \
            --publisher-request-v1 "${invalid_publisher_request}" \
            --lima-stage-one-authorization-v1 "${invalid_stage_one}" >/dev/null 2>&1; then
        fail "lifecycle accepted malformed ${invalid_kind} authority"
    fi
    [[ ! -e "${called}" ]] || fail "malformed ${invalid_kind} reached the mock executor"
done

SUBSTRATE_TEST_CALLED="${called}" \
SUBSTRATE_TEST_RECEIVED="${received}" \
    "${test_lifecycle}" stop \
        --install-prefix /tmp/substrate-r3 \
        --install-bootstrap-context-v1 "${CARRIER}" \
        --platform-bootstrap-mapping-v1 "${MAPPING}" \
        --executor-build-evidence-v1 "${EVIDENCE}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST}" \
        --lima-stage-one-authorization-v1 "${STAGE_ONE}" \
        >/dev/null

python3 - "${received}" "${CARRIER}" "${MAPPING}" "${EVIDENCE}" "${PUBLISHER_REQUEST}" "${STAGE_ONE}" <<'PY'
import json
import sys
record = json.load(open(sys.argv[1]))
assert record == {
    'action': 'stop',
    'install_prefix': '/tmp/substrate-r3',
    'install_bootstrap_context_v1': sys.argv[2],
    'platform_bootstrap_mapping_v1': sys.argv[3],
    'executor_build_evidence': json.loads(sys.argv[4]),
    'publisher_request_v1': json.loads(sys.argv[5]),
    'lima_stage_one_authorization_v1': json.loads(sys.argv[6]),
}
PY

# Stage-one authority is mandatory for every mapped mutation and must stop before dispatch.
rm -f "${called}" "${received}"
if SUBSTRATE_TEST_CALLED="${called}" \
    SUBSTRATE_TEST_RECEIVED="${received}" \
    "${test_lifecycle}" stop \
        --install-prefix /tmp/substrate-r3 \
        --install-bootstrap-context-v1 "${CARRIER}" \
        --platform-bootstrap-mapping-v1 "${MAPPING}" \
        --executor-build-evidence-v1 "${EVIDENCE}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST}" >/dev/null 2>&1; then
    fail 'lifecycle accepted a mutation without LimaStageOneAuthorizationV1'
fi
[[ ! -e "${called}" ]] || fail 'missing Stage-1 authority reached the mock executor'

bash -n "${LIFECYCLE}"
bash -n "${WARM}"
bash -n "${STOP}"
printf 'A1.1d-5R3-MAC non-native lifecycle fixture: PASS\n'
