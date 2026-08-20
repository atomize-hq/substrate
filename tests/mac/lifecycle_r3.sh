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
require_text "${MAC_PLIST}" '<key>MachServices</key>'
require_text "${MAC_PLIST}" '<string>run-publisher</string>'
require_text "${MAC_PLIST}" '<key>com.substrate.lifecycle.publisher.v1</key>'

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
    if name == 'ensure_vm_ready' and 'create_vm' not in body:
        raise SystemExit('ensure_vm_ready no longer preserves the selected R2 Stage-1 create path')
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
print(b64(carrier))
print(b64(mapping))
print(json.dumps(evidence, separators=(',', ':')))
FIXTURE_VALUES
)"
CARRIER="$(printf '%s\n' "${VALUES}" | sed -n '1p')"
MAPPING="$(printf '%s\n' "${VALUES}" | sed -n '2p')"
EVIDENCE="$(printf '%s\n' "${VALUES}" | sed -n '3p')"

# Missing evidence must stop before even a disposable-copy mock can be invoked.
if SUBSTRATE_TEST_CALLED="${called}" \
    SUBSTRATE_TEST_RECEIVED="${received}" \
    "${test_lifecycle}" stop \
        --install-prefix /tmp/substrate-r3 \
        --install-bootstrap-context-v1 "${CARRIER}" \
        --platform-bootstrap-mapping-v1 "${MAPPING}" >/dev/null 2>&1; then
    fail 'lifecycle accepted missing ExecutorBuildEvidenceV1'
fi
[[ ! -e "${called}" ]] || fail 'missing evidence reached the mock executor'

# Replaced or malformed authority must be rejected before any executor dispatch.
for invalid_kind in mapping evidence; do
    rm -f "${called}" "${received}"
    invalid_mapping="${MAPPING}"
    invalid_evidence="${EVIDENCE}"
    if [[ "${invalid_kind}" == mapping ]]; then
        invalid_mapping='not-base64url'
    else
        invalid_evidence='{"schema_owner":"wrong"}'
    fi
    if SUBSTRATE_TEST_CALLED="${called}" \
        SUBSTRATE_TEST_RECEIVED="${received}" \
        "${test_lifecycle}" stop \
            --install-prefix /tmp/substrate-r3 \
            --install-bootstrap-context-v1 "${CARRIER}" \
            --platform-bootstrap-mapping-v1 "${invalid_mapping}" \
            --executor-build-evidence-v1 "${invalid_evidence}" >/dev/null 2>&1; then
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
        >/dev/null

python3 - "${received}" "${CARRIER}" "${MAPPING}" "${EVIDENCE}" <<'PY'
import json
import sys
record = json.load(open(sys.argv[1]))
assert record == {
    'action': 'stop',
    'install_prefix': '/tmp/substrate-r3',
    'install_bootstrap_context_v1': sys.argv[2],
    'platform_bootstrap_mapping_v1': sys.argv[3],
    'executor_build_evidence': json.loads(sys.argv[4]),
}
PY

bash -n "${LIFECYCLE}"
bash -n "${WARM}"
bash -n "${STOP}"
printf 'A1.1d-5R3-MAC non-native lifecycle fixture: PASS\n'
