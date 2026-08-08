#!/usr/bin/env bash
# Non-native R5 MAC lifecycle fixture.  It performs static contract checks only: it never opens
# XPC, runs an elevated helper, touches Lima, or mutates a socket/known-hosts/Keychain artifact.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

bash -n "${REPO_ROOT}/scripts/mac/lima-lifecycle.sh"
bash -n "${REPO_ROOT}/scripts/mac/lima-warm.sh"
bash -n "${REPO_ROOT}/scripts/mac/lima-stop.sh"

python3 - "${REPO_ROOT}" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
paths = {
    'wrapper': root / 'scripts/mac/lima-lifecycle.sh',
    'warm': root / 'scripts/mac/lima-warm.sh',
    'stop': root / 'scripts/mac/lima-stop.sh',
    'control': root / 'src/bin/substrate-lifecycle-control.rs',
    'executor': root / 'src/bin/substrate-lifecycle-macos.rs',
    'client': root / 'crates/shell/src/execution/managed_lifecycle/macos_client.rs',
    'managed': root / 'crates/shell/src/execution/managed_lifecycle.rs',
    'common': root / 'crates/common/src/managed_artifact.rs',
    'installer': root / 'scripts/substrate/dev-install-substrate.sh',
}
source = {name: path.read_text() for name, path in paths.items()}

def fail(message):
    raise SystemExit(f'lifecycle_r3.sh: {message}')

def require(name, text):
    if text not in source[name]:
        fail(f'{paths[name].relative_to(root)} is missing {text}')

def function_body(text, name):
    match = re.search(rf'(?ms)^{re.escape(name)}\(\) \{{\n.*?(?=^[A-Za-z_][A-Za-z0-9_]*\(\) \{{|^if \[\[|\Z)', text)
    if not match:
        fail(f'missing named caller function {name}')
    return match.group(0)

# The shell membrane has exactly the two ordinary tags and no direct/bootstrap or raw selector.
wrapper = source['wrapper']
if 'lima-action' in wrapper or 'PublisherBootstrapAuthorizationV1' in wrapper:
    fail('wrapper exposes a raw or direct-bootstrap authority path')
require('wrapper', 'submit-mapped-lifecycle-v1')
for tag in ('stage_one_create', 'post_pm_action'):
    require('wrapper', tag)
if 'case "${TAG}" in' not in wrapper or '        *) printf' not in wrapper:
    fail('wrapper does not reject an unlisted tag')
for text in ('"current_anchor_counter"', 'set(request) != required'):
    require('wrapper', text)
wrapper_tags = re.findall(r'^\s*(stage_one_create|post_pm_action)\)\s+invoke_mac_lifecycle_control_v1', wrapper, re.MULTILINE)
if sorted(set(wrapper_tags)) != ['post_pm_action', 'stage_one_create'] \
        or 'guest_pairing_data_session' in wrapper \
        or 'guest_pairing_operator_tty_session' in wrapper:
    fail('ordinary R5 wrapper exposes an R6 tag or more than its exact two branches')

# Ordinary client XPC operations are fixed.  R6 moves the independent operator session
# completely out of the public shell/XPC surface; only its data child has a typed relay.
client = source['client']
for text in ('submit_stage_one_absent_instance_create_v1',
             'submit_post_pm_managed_action_v1',
             'submit_guest_pairing_data_session_v1',
             'open_mac_xpc_channel_v1',
             'attest_mac_publisher_response_v1',
             'libc::AF_UNIX', 'libc::SOCK_SEQPACKET', 'libc::FD_CLOEXEC',
             'canonical_publisher_bootstrap_authorization_v1',
             '"/usr/bin/sudo"', '--publisher-bootstrap-fd', '.arg("3")'):
    require('client', text)
for legacy in ('"bootstrap-publisher"', '"submit-request"', '"issue-guest-ticket"',
               '"lima-action"', 'guest-pairing-operator-tty-session',
               'with_terminal_v1', 'xpc_dictionary_set_fd'):
    if legacy in client:
        fail(f'client retains generic or operator-terminal relay {legacy}')
xpc_open = client[client.index('pub fn open_mac_xpc_channel_v1'):client.index('/// Verify the fixed publisher service')]
for operation in ('"stage-one-create"', '"post-pm-action"', '"guest-pairing-data-session"'):
    if operation not in xpc_open:
        fail(f'client XPC operation decoder is missing fixed operation {operation}')
if 'submit_closed_guest_pairing_session_v1' not in client:
    fail('client has no typed closed R6 data relay helper')

# Direct bootstrap dispatch occurs before stdin and the executor accepts only FD3 canonical bytes.
control = source['control']
if control.index('"publisher-bootstrap" =>') > control.index('"submit-mapped-lifecycle-v1" =>'):
    fail('control dispatches ordinary stdin before hidden direct bootstrap')
direct = control[control.index('pub fn publisher_bootstrap_direct_interactive_v1'):control.index('pub fn guest_publisher_pairing_direct_interactive_v1')]
if 'submit_mapped_lifecycle_v1' in direct:
    fail('direct bootstrap falls back to an ordinary mapped submit route')
require('control', 'deliver_retained_publisher_bootstrap_authorization_v1')
require('control', 'read_exact_publisher_bootstrap_request_from_stdin_v1')
require('control', 'parse_mac_publisher_bootstrap_request_v1')
for text in ('guest-publisher-pairing-direct-interactive-v1',
             'R6 pairing protocol tags are accepted only by the hidden direct-interactive path',
             'issue_lima_guest_pairing_ticket_v1',
             'advance_lima_guest_pairing_record_v1',
             'display_guest_pairing_challenge_v1',
             'GuestPublisherPairingOperatorLaunchV1',
             'validate_guest_publisher_pairing_operator_launch_against_ticket_at_v1',
             'direct duplicated /dev/tty', '--tty=true',
             'guest-pairing-operator-tty-session-v1',
             'DirectOperatorChildGuardV1'):
    require('control', text)
for forbidden in ('consume_lima_guest_pairing_ticket_v1', 'with_terminal_v1',
                  'submit_guest_pairing_operator_tty_session_v1'):
    if forbidden in control:
        fail(f'control retains forbidden operator relay {forbidden}')
if control.index('"guest-publisher-pairing-direct-interactive-v1" =>') > control.index('"submit-mapped-lifecycle-v1" =>'):
    fail('R6 direct pairing is dispatched after ordinary mapped stdin')

executor = source['executor']
if executor.index('if operation == "--publisher-bootstrap-fd"') > executor.index('read_to_end(&mut input)'):
    fail('executor reads stdin before exact FD3 bootstrap dispatch')
for text in ('consume_publisher_bootstrap_fd3_v1', 'getpeereid', 'LOCAL_PEERPID',
             'parse_publisher_bootstrap_authorization_v1', 'descriptor 3',
             'mac_attest_running_executor_image_v1', 'SecKeyCreateRandomKey',
             'mac_transition_bootstrap_intent_v1', 'manifest_generation: authorization.manifest_generation',
             'manifest_sha256: authorization.manifest_sha256.clone()',
             'mac_attest_bootstrap_control_peer_predecode_v1',
             'mac_kernel_peer_executable_path_v1', 'proc_pidpath',
             'MacPublisherControlAdmissionV1', 'mac_verified_control_admission_v1',
             'derive_mac_lima_stage_one_successor_manifest_v1',
             'MacLimaStageOneCapsuleV1', 'render_mac_lima_stage_one_profile_v1',
             'execute_closed_mac_lima_stage_one_effect_v1',
             'resume_mac_lima_stage_one_after_protected_state_cas_v1',
             '"platform_bootstrap_mapping_v1": mapping.encode(carrier)?',
             'mac_read_stage_one_transition_artifact_no_follow_v1',
             'load_or_sign_mac_lima_stage_one_receipt_v1',
             'mac_require_root_owned_immutable_tool_path_v1',
             'mac_measure_root_owned_immutable_lima_tool_v1',
             'limactl', 'EffectStarted', 'InstanceObserved', 'PreservingBlocked'):
    require('executor', text)
for text in ('"guest-pairing-data-session"',
             'execute_mac_guest_pairing_session_v1',
             'issue_lima_guest_pairing_ticket_v1',
             'advance_lima_guest_pairing_record_v1',
             'open_pm_bound_guest_pairing_data_session_v1',
             'mac_guest_pairing_record_account_v1',
             'GuestPublisherPairingOperatorLaunchV1',
             'GuestPublisherPairingOperatorProofV1',
             'operator_proof_verified',
             'operator_proof_accepted'):
    require('executor', text)
for forbidden in ('consume_lima_guest_pairing_ticket_v1',
                  'open_pm_bound_guest_pairing_operator_tty_session_v1',
                  'operator_tty_session.confirmation_commitment'):
    if forbidden in executor:
        fail(f'privileged executor retains forbidden R6 operator relay {forbidden}')
require('executor', '"guest-pairing-data-session" => MappedLifecycleTagV1::GuestPairingDataSession')
for text in ('MacPublisherBootstrapAttemptLocatorV1',
             'mac_open_or_allocate_bootstrap_attempt_locator_v1',
             'mac_reconstruct_bootstrap_authorization_from_locator_v1',
             'mac_complete_bootstrap_attempt_locator_v1',
             'mac_validate_completed_bootstrap_attempt_locator_joins_v1',
             'completed bootstrap locator protected state no longer exact-joins',
             'completed bootstrap locator Stage-1 capsule no longer exact-joins',
             'mac_keychain_bootstrap_attempt_locator_account_v1',
             'bootstrap attempt locator authorization digest does not reconstruct',
             'incomplete bootstrap attempt locator has expired; preserving state',
             'canonical_response_b64'):
    require('executor', text)
# Correction-01: every ordinary post-PM request is recoverable from a signed prepared CAS,
# and the privileged executor has a closed literal plan rather than an R6-owned guest relay.
for text in ('execute_mac_post_pm_action_v1',
             'mac_execute_closed_post_pm_effect_v1',
             'mac_post_pm_planned_receipt_v1',
             'mac_lima_closed_post_pm_receipt_plan_v1',
             'post-pm:mac.lima.instance:',
             'classify_mac_post_pm_transaction_v1',
             'MacPostPmTransactionClassV1',
             'mac_open_or_create_post_pm_attempt_journal_v1',
             'mac_persist_post_pm_journal_state_absent_or_exact_v1',
             'post-pm-attempts/',
             'EffectStarted', 'EffectObserved', 'Completed',
             'mac_post_pm_effect_plan_v1',
             'MacPostPmEffectPrimitiveV1',
             'mac_post_pm_artifact_binding_v1',
             'mac_measure_post_pm_artifact_source_v1',
             'mac_stage_measured_post_pm_artifact_absent_or_exact_v1',
             'mac_persist_completed_post_pm_journal_from_receipt_v1',
             'mac_exact_control_admission_successor_v1',
             'MacFixedLimaCommandOutcomeV1',
             'MAC_LIMA_EFFECT_TIMEOUT_V1',
             'mac_observe_closed_post_pm_effect_v1',
             'mac_post_pm_effect_retry_decision_v1',
             'mac_require_post_pm_after_state_v1',
             'MacPostPmTargetIntegrityV1',
             'kind: "regular file"',
             'kind: "directory"',
             '"%F:%U:%G:%a".to_string()',
             'post-PM effect observation is ambiguous; preserving first',
             'resume_action_receipt_commit_v1',
             'replace_mac_control_admission_authority_v1',
             'mac_open_exact_completed_post_pm_receipt_v1',
             'completed-retry'):
    require('executor', text)
for forbidden in ('r5-post' + '-pm', 'mac_mark_post_pm_effect_started_v1',
                  'closed post-PM transaction requires its exact authoritative receipt pipeline'):
    if forbidden in executor:
        fail(f'executor retains forbidden post-PM relay/stop {forbidden}')
artifact_exec_start = executor.index('fn mac_execute_post_pm_effect_plan_v1')
artifact_exec_end = executor.index('\n/// Read only the exact action-specific observation', artifact_exec_start)
artifact_exec = executor[artifact_exec_start:artifact_exec_end]
if 'mac_stage_measured_post_pm_artifact_absent_or_exact_v1' not in artifact_exec or \
   'staged.display().to_string()' not in artifact_exec or 'source_path.clone()' in artifact_exec:
    fail('post-PM artifact copy can reopen a mutable prefix source instead of retained staged bytes')
target_observer_start = executor.index('fn mac_observe_closed_post_pm_effect_v1')
target_observer_end = executor.index('\n#[cfg(target_os = "macos")]\nfn mac_execute_closed_post_pm_effect_v1', target_observer_start)
target_observer = executor[target_observer_start:target_observer_end]
for text in ('target.kind', 'target.sha256.as_deref()', '"%F:%U:%G:%a"', '"/usr/bin/stat"'):
    if text not in target_observer:
        fail('post-PM target observer does not require exact type/metadata integrity')
# The Rust fixture executes the pure receipt-plan, state-classification, retry-decision, and
# exhaustive literal effect-plan helpers; the shell check keeps their proof hooks from regressing
# into a lexical-only assertion.
for text in ('correction_post_pm_instance_receipt_plans_are_exhaustive_and_create_is_sole_cross_generation',
             'correction_post_pm_classifier_is_recoverable_across_each_journal_boundary',
             'correction_post_pm_journal_binds_prepared_request_and_observation_boundaries',
             'correction_post_pm_effect_planner_is_exhaustive_literal_and_has_no_guest_relay',
             'MacPostPmTransactionClassV1::EffectStartedRetry',
             'MacPostPmEffectObservationStateV1::Ambiguous',
             'mac_require_post_pm_after_state_v1'):
    require('executor', text)
for text in ('direct bootstrap authorization has expired before FD3 admission',
             'Stage-1 authorization expired before the selected absent-instance effect'):
    require('executor', text)
issuer_start = executor.index('fn mac_issue_closed_post_pm_requests_after_stage_one_v1')
issuer_end = executor.index('\nfn mac_stage_one_profile_path_v1', issuer_start)
issuer = executor[issuer_start:issuer_end]
for text in ('state.current_anchor.requester_principal',
             'state.current_anchor.attempt_nonce', 'state.current_anchor.executor_identity',
             'mac_post_pm_planned_receipt_v1'):
    if text not in issuer:
        fail('Stage-1 successor post-PM request issue does not derive from durable successor state')
if 'stage_one.' in issuer:
    fail('Stage-1 authorization is improperly retained as post-PM authority')
if 'last-action-receipt.v1.json' in executor:
    fail('executor retains the overwriteable last-action receipt shortcut')
resume_start = executor.index('fn resume_mac_lima_stage_one_after_protected_state_cas_v1')
resume_end = executor.index('\n#[cfg(target_os = "macos")]\nfn execute_closed_mac_lima_stage_one_effect_v1', resume_start)
resume = executor[resume_start:resume_end]
for text in ('canonical_lifecycle_publisher_protected_state_v1',
             'canonical_action_receipt_bytes_v1',
             'canonical_action_receipt_index_bytes_v1',
             'canonical_managed_manifest_head_v1',
             'replace_mac_control_admission_authority_v1',
             'prepared_protected_state_sha256'):
    if text not in resume:
        fail(f'Stage-1 post-CAS resume does not retain {text}')
if 'mac_run_fixed_lima_command_v1' in resume:
    fail('Stage-1 post-CAS resume can re-run the Lima effect')
receipt_retry_start = executor.index('fn load_or_sign_mac_lima_stage_one_receipt_v1')
receipt_retry_end = executor.index('\nfn prepare_mac_lima_stage_one_transition_v1', receipt_retry_start)
receipt_retry = executor[receipt_retry_start:receipt_retry_end]
for text in ('canonical_action_receipt_bytes_v1',
             'validate_managed_action_receipt_signature_v1',
             'retained Stage-1 receipt is not an exact retry'):
    if text not in receipt_retry:
        fail(f'Stage-1 receipt retry does not preserve exact signed bytes: {text}')
effect_start = executor.index('fn execute_closed_mac_lima_stage_one_effect_v1')
resume_call = executor.index('resume_mac_lima_stage_one_after_protected_state_cas_v1', effect_start)
profile_write = executor.index('mac_write_stage_one_profile_absent_or_exact_v1', effect_start)
if resume_call > profile_write:
    fail('Stage-1 post-CAS resume rewrites effect input before proving durable completion')

# The common authorization binds macOS control identity only for mac_host_shared and carries the
# canonical digest required for exact retry/anchor joining.
common = source['common']
for text in ('pub mac_control_authority: Option<MacPublisherControlAuthorityV1>',
             'pub manifest_generation: u64', 'pub manifest_sha256: String',
             'publisher_bootstrap_authorization_sha256_v1',
             'publisher bootstrap manifest_generation must be positive',
             'publisher bootstrap manifest_sha256',
             'mac_host_shared bootstrap authorization requires mac_control_authority',
             'mac_control_authority does not join authorization source identity',
             'mac_control_authority does not join authorization executor build',
             'mac_control_authority is valid only for mac_host_shared'):
    require('common', text)
for text in ('pub struct MacPublisherBootstrapRequestV1',
             'pub install_bootstrap_context_v1: String',
             'pub struct MacPublisherBootstrapAttemptLocatorV1',
             'mac_publisher_bootstrap_attempt_locator_sha256_v1',
             'validate_mac_publisher_bootstrap_attempt_locator_v1',
             'pub pre_pm_manifest: Option<ManagedArtifactManifestV1>',
             'pub struct MacPublisherInstallProvenanceV1',
             'pub struct MacLimaStageOneCapsuleV1',
             'pub prepared_protected_state_sha256: Option<String>',
             'pub struct LimaStageOneObservationV1',
             'canonical_mac_lima_stage_one_capsule_v1'):
    require('common', text)
for text in ('pub struct MacLimaStageOneSuccessorTemplateV1',
             'current_pre_pm_manifest_generation', 'current_anchor_counter',
             'post_effect_observation_slots', 'guest_machine_id',
             'validate_mac_lima_stage_one_successor_template_v1'):
    require('common', text)
protected_start = common.index('pub struct LifecyclePublisherProtectedStateV1')
protected_end = common.index('\n}\n', protected_start)
if 'mac_control_authority' in common[protected_start:protected_end]:
    fail('R5 illegally extends LifecyclePublisherProtectedStateV1')
if 'manifest_generation' in common[protected_start:protected_end] or 'manifest_sha256' in common[protected_start:protected_end]:
    fail('R5 illegally extends LifecyclePublisherProtectedStateV1 with manifest identity')

# Both privileged and control decoders repeat the same closed tag/role fence.
managed = source['managed']
for text in ('MappedLifecycleTagV1', 'closed_mapped_mac_role_action_v1',
             'InstallBootstrapContextCarrierV1::decode',
             'PlatformBootstrapMappingV1::decode',
             'post_pm_action must not carry LimaStageOneAuthorizationV1',
             'complete_mac_lima_stage_one_transition_v1',
             'Stage-1 receipt does not complete the exact signed pre-PM to PM transition'):
    require('managed', text)
for text in ('validate_mac_mapped_action_authority_v1', 'closed_mac_role_action_v1',
             'audit_token = mac_audit_token_ffi_v1(peer)?'):
    require('executor', text)
for text in ('pairing_record_expected_generation_v1',
             'fixed macOS XPC operation does not match its closed protocol tag',
             'persist_r6_guest_pairing_record_transition_v1',
             'validate_guest_publisher_bootstrap_hello_v1',
             'validate_guest_publisher_bootstrap_transcript_v1',
             '"ticket_consumed"',
             'guest-anchor-accepted',
             'R6_DATA_FRAME_TIMEOUT_V1',
             'operator_proof_verified',
             'validate_guest_publisher_pairing_operator_proof_against_ticket_at_v1',
             'R6 data child exceeded its fixed frame deadline'):
    require('executor', text)
for forbidden in ('guest_state_root_prepared', 'confirmation_bound',
                  'xpc_dictionary_get_fd', 'relay raw retained terminal input',
                  'relay fixed R6 guest PTY output'):
    if forbidden in executor:
        fail(f'R6 executor retains forbidden relay/state {forbidden}')
for forbidden in ('binding FD3 pipe', 'response FD4 pipe', 'host-confirmed'):
    if forbidden in source['executor']:
        fail(f'R6 executor retains forbidden {forbidden}')
for text in ('pairing_record_expected_generation_v1',
             'R6 pairing session expected record generation must be positive'):
    require('managed', text)

linux = (root / 'src/bin/substrate-lifecycle-linux.rs').read_text()
for text in ('guest-pairing-data-session-v1', 'guest-pairing-operator-tty-session-v1',
             'R6 data guest entrypoint accepts no caller-selected argument or selector',
             'R6 operator TTY entrypoint accepts exactly one signed launch envelope',
             'generic guest pairing commands are unreachable',
             'run_pm_bound_guest_pairing_data_session_v1',
             'run_pm_bound_guest_pairing_operator_tty_session_v1',
             'read_r6_operator_tty_confirmation_commitment_v1',
             'R6TerminalEchoGuardV1',
             'require_r6_pm_bound_lima_guest_v1',
             'require_current_guest_pairing_ticket_v1',
             'R6_LIMA_STAGE_ONE_MARKER_PATH_V1',
             'parse_r6_operator_launch_argument_v1',
             'measure_r6_installed_guest_executor_v1',
             'R6 running guest executor digest does not match the staged binding',
             'persist_r6_guest_pairing_transcript_v1',
             'prepare_r6_guest_publisher_anchor_v1',
             'R6 data session received a substituted guest-anchor acknowledgement',
             'validate_r6_operator_proof_before_intent_v1',
             'GUEST_PAIRING_LITERAL_V1'):
    if text not in linux:
        fail(f'Linux guest R6 boundary is missing {text}')
r6_start = linux.index('if command == "guest-pairing-data-session-v1"')
r6_end = linux.index('let mut state_root:', r6_start)
for forbidden in ('--state-root', '--tty-path', '--transport', '--ticket-file', '--transcript-file'):
    if forbidden in linux[r6_start:r6_end]:
        fail(f'R6 guest entrypoint accepts caller-selected {forbidden}')

# Exact listed callers use no direct Lima primitive and select the correct tag without passing a
# Stage-1 authorization on post-PM requests.
warm = source['warm']
stage_one = function_body(warm, 'ensure_vm_ready')
if 'lima-lifecycle.sh" stage_one_create' not in stage_one or '--lima-stage-one-authorization-v1' not in stage_one:
    fail('ensure_vm_ready is not the sole Stage-1 mapped caller')
if '--platform-bootstrap-mapping-v1' in stage_one or '--publisher-request-v1' in stage_one:
    fail('ensure_vm_ready forwards a post-PM mapping or request into Stage-1')
if 'stage_response="$(' not in stage_one or 'adopt_stage_one_mapping_response_v1 "${stage_response}"' not in stage_one:
    fail('ensure_vm_ready does not retain the finalized mapping returned by Stage-1')
adopted_mapping = function_body(warm, 'adopt_stage_one_mapping_response_v1')
for text in ('RESPONSE_KEYS', 'platform_bootstrap_mapping_v1', 'post_pm_requests_v1',
             'SEED_EXACT_KEYS', 'audit_token_bound',
             'Caller mapping does not exactly match the Stage-1 successor mapping',
             'PLATFORM_BOOTSTRAP_MAPPING_V1="${adopted_mapping}"',
             'PUBLISHER_REQUEST_V1="${adopted_request}"'):
    if text not in adopted_mapping:
        fail('Stage-1 mapping adoption is not closed to the attested completed response')
for name in ('destroy_vm', 'stage_workspace', 'ensure_substrate_group', 'install_agent_from_host',
             'install_cli_from_host', 'install_gateway_from_host', 'install_guest_binaries',
             'bootstrap_guest_private_home', 'write_systemd_units', 'enable_socket_activation',
             'write_layout_sentinel', 'configure_guest'):
    body = function_body(warm, name)
    if 'lima-lifecycle.sh" post_pm_action' not in body:
        fail(f'{name} is not a post_pm_action caller')
    if '--lima-stage-one-authorization-v1' in body or re.search(r'\blimactl\b', body):
        fail(f'{name} retains Stage-1 or direct Lima authority')
    if name == 'configure_guest' and any(raw in body for raw in (
        'observe_lima_mapping_v1', 'verify_lima_mapping_v1', 'current_layout_version',
        'socket_summary', 'linger_guidance',
    )):
        fail('configure_guest reaches a raw guest observation or projection helper')

stop = source['stop']
for name in ('resolve_lima_stop_authority_v1', 'invoke_mapped_lima_stop_v1'):
    require('stop', f'{name}()')
if 'post_pm_action' not in stop or '--lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"' in stop:
    fail('lima-stop is not a Stage-1-free mapped post-PM call')
if re.search(r'(?<![A-Za-z0-9_])limactl(?![A-Za-z0-9_])', stop):
    fail('lima-stop retains a direct destructive Lima primitive')

installer = source['installer']
for text in ('publish_mac_publisher_install_provenance_v1',
             'bootstrap-provenance.v1.json', 'root:wheel 0444',
             'substrate.mac-publisher-install-provenance', 'profile_template_sha256',
             '/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1',
             '/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist',
             'require_root_owned_immutable_path',
             'control_expected_sha', 'executor_expected_sha',
             'privileged executor copy does not match pre-elevation digest',
             'retained limactl path segment is not root-owned immutable state',
             'absent-or-exact semantics'):
    require('installer', text)

print('A1.1d-5R3-MAC non-native lifecycle fixture: PASS')
PY

# Execute the closed Stage-1 response adoption helper against a canonical completed response and
# a one-field tamper. This is a parser-only fixture: it does not invoke the control binary, XPC,
# Lima, or any native lifecycle action.
mapping_fixture="$(mktemp -d)"
trap 'rm -rf "${mapping_fixture}"' EXIT
python3 - "${REPO_ROOT}/scripts/mac/lima-warm.sh" "${mapping_fixture}/adopt-stage-one-mapping" <<'PY'
from pathlib import Path
import re
import sys

source = Path(sys.argv[1]).read_text()
match = re.search(
    r'(?ms)^adopt_stage_one_mapping_response_v1\(\) \{\n.*?(?=^[A-Za-z_][A-Za-z0-9_]*\(\) \{|\Z)',
    source,
)
if not match:
    raise SystemExit('missing Stage-1 mapping adoption helper')
Path(sys.argv[2]).write_text(
    '#!/usr/bin/env bash\n'
    'set -euo pipefail\n'
    'fatal() { printf "%s\\n" "$1" >&2; exit 1; }\n'
    'INSTALL_BOOTSTRAP_COMMITMENT="$1"\n'
    'PLATFORM_BOOTSTRAP_MAPPING_V1="$2"\n'
    'PUBLISHER_REQUEST_V1="$3"\n'
    + match.group(0)
    + '\nadopt_stage_one_mapping_response_v1 "$4"\nprintf "%s\\n%s" "${PLATFORM_BOOTSTRAP_MAPPING_V1}" "${PUBLISHER_REQUEST_V1}"\n'
)
PY
chmod 700 "${mapping_fixture}/adopt-stage-one-mapping"
stage_mapping_fixture="$(python3 - <<'PY'
import base64
import copy
import hashlib
import json

def b64(value):
    return base64.urlsafe_b64encode(value.encode()).rstrip(b'=').decode()

commitment = 'a' * 64
lines = (
    'domain=substrate.platform_bootstrap_mapping',
    'version=1',
    f'host_context_commitment={commitment}',
    'platform_kind=lima',
    f'instance_name={b64("substrate")}',
    'guest_machine_id=' + 'b' * 32,
    f'host_platform_control_root={b64("/Users/alice/.lima")}',
    f'realized_substrate_home={b64("/home/alice/.substrate")}',
    f'realized_principal_account={b64("alice")}',
    'realized_principal_uid=501',
    'transport_kind=lima',
    f'transport_host={b64("/tmp/substrate.sock")}',
    f'transport_guest_socket={b64("/run/substrate.sock")}',
)
mapping = b64('\n'.join(lines) + '\n')
candidate = {
    'host_context_commitment': commitment,
    'platform_mapping_commitment': hashlib.sha256(mapping.encode('ascii')).hexdigest(),
    'scope_id': '018f1234-5678-7abc-8def-0123456789ab',
    'current_anchor_counter': 1,
    'current_anchor_sha256': 'd' * 64,
    'manifest_generation': 2,
    'manifest_sha256': 'c' * 64,
    'role': 'mac.lima.layout-sentinel',
    'action': 'create',
    'object_identity': {
        'scope_id': '018f1234-5678-7abc-8def-0123456789ab',
        'parent_identity': 'mac-lima-fixed-role-table',
        'name_identity': 'mac.lima.layout-sentinel',
        'physical_identity': '/etc/substrate-lima-layout',
    },
    'requester_principal': 'alice',
    'attempt_nonce': '018f1234-5678-7abc-8def-0123456789ac',
    'expected_executor_build': {
        'source_commit': 'a' * 40,
        'source_tree': 'b' * 40,
        'source_ref': 'refs/heads/r5-fixture',
        'target_triple': 'aarch64-apple-darwin',
        'artifact_sha256': 'e' * 64,
        'artifact_path': '/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1',
    },
}
seed = copy.deepcopy(candidate)
seed.update({
    'platform_mapping_commitment': 'f' * 64,
    'current_anchor_counter': 0,
    'current_anchor_sha256': '1' * 64,
    'manifest_generation': 1,
    'manifest_sha256': '2' * 64,
})
response = {
    'status': 'completed',
    'receipt': {},
    'platform_bootstrap_mapping_v1': mapping,
    'post_pm_requests_v1': [candidate],
    'manifest_generation': 2,
    'manifest_sha256': 'c' * 64,
    'xpc_attestation': {
        'mach_service': 'com.substrate.lifecycle.publisher.v1',
        'audit_token_bound': True,
    },
}
print(mapping)
print(json.dumps(response, separators=(',', ':')))
print(json.dumps(seed, separators=(',', ':')))
print(json.dumps(candidate, sort_keys=True, separators=(',', ':')))
PY
)"
stage_mapping="$(sed -n '1p' <<<"${stage_mapping_fixture}")"
stage_response="$(sed -n '2p' <<<"${stage_mapping_fixture}")"
stage_seed="$(sed -n '3p' <<<"${stage_mapping_fixture}")"
expected_request="$(sed -n '4p' <<<"${stage_mapping_fixture}")"
adopted_response="$("${mapping_fixture}/adopt-stage-one-mapping" "$(printf 'a%.0s' {1..64})" '' "${stage_seed}" "${stage_response}")"
adopted_mapping="$(sed -n '1p' <<<"${adopted_response}")"
adopted_request="$(sed -n '2p' <<<"${adopted_response}")"
[[ "${adopted_mapping}" == "${stage_mapping}" ]] || fail 'canonical Stage-1 response mapping was not adopted'
[[ "${adopted_request}" == "${expected_request}" ]] || fail 'canonical Stage-1 response did not replace the stale post-PM request'
tampered_response="${stage_response/\"manifest_generation\":2/\"manifest_generation\":1}"
if "${mapping_fixture}/adopt-stage-one-mapping" "$(printf 'a%.0s' {1..64})" '' "${stage_seed}" "${tampered_response}" >/dev/null 2>&1; then
    fail 'non-successor Stage-1 response mapping was accepted'
fi
ambiguous_response="$(python3 - "${stage_response}" <<'PY'
import json
import sys

response = json.loads(sys.argv[1])
response['post_pm_requests_v1'].append(response['post_pm_requests_v1'][0])
print(json.dumps(response, separators=(',', ':')))
PY
)"
if "${mapping_fixture}/adopt-stage-one-mapping" "$(printf 'a%.0s' {1..64})" '' "${stage_seed}" "${ambiguous_response}" >/dev/null 2>&1; then
    fail 'ambiguous Stage-1 successor post-PM request set was accepted'
fi
printf 'A1.1d-5R3-MAC Stage-1 mapping/request adoption fixture: PASS\n'
