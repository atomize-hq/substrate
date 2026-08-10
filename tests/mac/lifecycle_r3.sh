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
    'architecture': root / 'llm-last-mile/runtime-refactor/01-target-architecture.md',
    'slice_map': root / 'llm-last-mile/runtime-refactor/03-phase-slice-map.md',
    'contracts': root / 'llm-last-mile/runtime-refactor/04-contracts-and-gates.md',
    'ledger': root / 'llm-last-mile/runtime-refactor/05-debug-regression-ledger.md',
    'recovery_spec': root / 'llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/SPEC.md',
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

# The fixed-install membrane has one signed Stage-1 branch and preserves the existing, separately
# validated ordinary post-PM branch for lima-stop.  The installer/warm route can use only the
# Stage-1 branch; it cannot select a catalogue request or a post-PM role/action.
wrapper = source['wrapper']
if 'lima-action' in wrapper or 'PublisherBootstrapAuthorizationV1' in wrapper:
    fail('wrapper exposes a raw or direct-bootstrap authority path')
require('wrapper', 'submit-mapped-lifecycle-v1')
require('wrapper', 'stage_one_create')
require('wrapper', 'post_pm_action')
require('wrapper', 'stage_value["successor_template"]["executor_build_evidence"]')
for text in ('case "${TAG}" in', 'stage_one_create|post_pm_action)',
             '--publisher-request-v1)', '--platform-bootstrap-mapping-v1)',
             '--executor-build-evidence-v1)', 'if tag == "stage_one_create":',
             'if tag == "post_pm_action":'):
    require('wrapper', text)
if '[[ "${TAG}" == "stage_one_create" ]]' in wrapper or \
        'guest_pairing_data_session' in wrapper or 'guest_pairing_operator_tty_session' in wrapper:
    fail('wrapper admits an unlisted lifecycle tag')
if 'mac.lima.guest-binary(world)' in wrapper or 'post_pm_requests_v1' in wrapper:
    fail('wrapper admits the removed world role or catalogue traversal')

# Ordinary client XPC operations are fixed.  R6 moves the independent operator session
# completely out of the public shell/XPC surface; only its data child has a typed relay.
client = source['client']
for text in ('submit_stage_one_absent_instance_create_v1',
             'submit_post_pm_managed_action_v1',
             'submit_guest_pairing_data_session_v1',
             'open_mac_xpc_channel_v1',
             'attest_mac_publisher_response_v1',
             'libc::AF_UNIX', 'libc::SOCK_STREAM', 'libc::FD_CLOEXEC',
             'libc::SO_NOSIGPIPE', 'libc::MSG_DONTWAIT', 'libc::SHUT_WR',
             'OwnedFd::from_raw_fd', 'RetainedBootstrapChildGuardV1',
             'parse_canonical_direct_bootstrap_response_v1', 'bootstrap_channel_bound',
             'canonical_publisher_bootstrap_authorization_v1',
             '"/usr/bin/sudo"', '--publisher-bootstrap-fd', '.arg("3")'):
    require('client', text)
if 'libc::SOCK_SEQPACKET' in client:
    fail('Darwin direct-bootstrap client still selects SOCK_SEQPACKET')
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

# The R3 signer is one software P-256 private key in the explicitly opened legacy System
# Keychain.  Searches are scoped with kSecMatchSearchList; only adds use kSecUseKeychain.  No
# private selector, ambient/default/user/file fallback, authorization UI, or non-exportability
# assertion may re-enter this boundary.
ffi_start = executor.index('mod mac_system_keychain_ffi_v1')
ffi_end = executor.index('\n#[cfg(target_os = "macos")]\n#[repr(C)]\nstruct MacAuditTokenFfiV1', ffi_start)
keychain_ffi = executor[ffi_start:ffi_end]
for forbidden in ('kSecUse' + 'SystemKeychain', 'SecKeychainCopyDefault',
                  'kSecUseDataProtectionKeychain', 'kSecAttrTokenIDSecureEnclave',
                  'non-exportable P-256 key', 'not explicitly non-exportable'):
    if forbidden in executor:
        fail(f'macOS signer retains forbidden private/ambient/non-exportability path {forbidden}')
for text in ('MAC_SYSTEM_KEYCHAIN_PATH_V1', '"/Library/Keychains/System.keychain"',
             'SecKeychainOpen', 'SecKeychainGetPath',
             'open_explicit_system_keychain', 'verify_explicit_system_keychain_path',
             'kSecUseKeychain', 'kSecMatchSearchList',
             'kSecUseAuthenticationUI', 'kSecUseAuthenticationUIFail',
             'kSecMatchLimitAll', 'kSecReturnAttributes', 'kSecValueRef',
             'CFArrayGetCount',
             'exact_private_key_matches', 'validate_private_key_identity',
             'retire_mac_system_keychain_software_signer_v1',
             'refuses signer deletion while protected state survives',
             'delete_p256_key_exact', 'SecItemDelete',
             'verify final System Keychain P-256 key absence'):
    if text not in keychain_ffi and text not in executor:
        fail(f'explicit System-Keychain software signer is missing {text}')
if keychain_ffi.count('kSecUseKeychain') < 3:
    fail('explicit System-Keychain adds do not all name their destination keychain')
if keychain_ffi.count('kSecMatchSearchList') < 2:
    fail('System-Keychain reads/deletes can escape the one-keychain search list')
if keychain_ffi.count('kSecUseAuthenticationUIFail') < 2:
    fail('System-Keychain operations can use the default UI-permitting policy')
if 'ERR_SEC_DUPLICATE_ITEM' not in keychain_ffi or 'ambiguous' not in keychain_ffi:
    fail('System-Keychain signer does not preserve-first on duplicate/ambiguous keys')
if 'SecKeyCopyExternalRepresentation(private_key' in keychain_ffi:
    fail('product signer path exports private key bytes')

retire_start = executor.index('pub fn retire_mac_system_keychain_software_signer_v1')
retire_end = executor.index('\nfn mac_verified_control_peer_requirement_v1', retire_start)
retire = executor[retire_start:retire_end]
for earlier, later in (
    ('mac_keychain_protected_state_account_v1', 'mac_keychain_durable_cas_guard_v1'),
    ('mac_keychain_durable_cas_guard_v1',
     'read_system_keychain_protected_state_for_scope_unbound_v1'),
    ('read_system_keychain_protected_state_for_scope_unbound_v1', 'delete_p256_key_exact'),
):
    if earlier not in retire or later not in retire or retire.index(earlier) > retire.index(later):
        fail('signer retirement is not serialized with protected-wrapper CAS')

cas_start = executor.index('pub fn compare_and_swap_mac_publisher_protected_state_v1')
cas_end = executor.index('\n/// Establish the designated service', cas_start)
protected_cas = executor[cas_start:cas_end]
if protected_cas.index('mac_keychain_durable_cas_guard_v1') > protected_cas.index(
        'mac_open_system_keychain_p256_spki_der_v1'):
    fail('protected-wrapper CAS validates the signer before taking the retirement lock')
for text in ('kSecMatchItemList', 'exact_private_key_item_delete_query'):
    if text not in keychain_ffi:
        fail(f'exact signer retirement does not delete the validated item: missing {text}')
for name in ('architecture', 'slice_map', 'contracts', 'ledger'):
    require(name, 'AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION')
for stale in (
    'macOS host | root LaunchDaemon label `com.substrate.lifecycle.publisher.v1`; non-exportable',
    '`mac.publisher.signing-key` | System-Keychain service `com.substrate.lifecycle.v1`, account `<scope-id>:signing-key` | non-exportable',
    'exported public SPKI even though the private key is non-exportable',
):
    if stale in source['contracts']:
        fail(f'R3 control pack retains stale macOS signer claim {stale}')
for stale in ('System-Keychain non-exportable P-256 signer',
              'non-exportable host P-256 signing'):
    if stale in source['recovery_spec']:
        fail(f'active R3 recovery specification retains stale signer claim {stale}')
for required in ('explicitly opened legacy `/Library/Keychains/System.keychain`',
                 'software P-256 signer', 'sufficiently privileged', 'root process may export',
                 'fixed root LaunchDaemon', 'Apple Silicon macOS only',
                 'Secure Enclave/Data Protection Keychain',
                 'user-LaunchAgent signer', 'deferred hardening'):
    require('recovery_spec', required)

fd3_consumer = executor[executor.index('fn consume_publisher_bootstrap_fd3_v1'):executor.index('struct MacBootstrapPeerIdentityV1')]
fd_check = fd3_consumer.index('if fd != 3')
cloexec = fd3_consumer.index('mac_rearm_bootstrap_fd3_cloexec_v1(fd)?')
socket_type = fd3_consumer.index('mac_require_stream_channel_v1(fd)?')
peer_check = fd3_consumer.index('mac_bootstrap_peer_identity_v1')
decode = fd3_consumer.index('parse_publisher_bootstrap_authorization_v1')
if not fd_check < cloexec < socket_type < peer_check < decode:
    fail('executor does not re-arm FD3 CLOEXEC before socket/peer/decode work')
for text in ('consume_publisher_bootstrap_fd3_v1', 'getpeereid', 'LOCAL_PEERPID',
             'parse_publisher_bootstrap_authorization_v1', 'descriptor 3',
             'libc::SOCK_STREAM', 'libc::SO_NOSIGPIPE', 'libc::MSG_DONTWAIT',
             'mac_read_single_stream_document_v1', 'mac_send_single_stream_document_v1',
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

installer = source['installer']
parser_start = installer.index('stage_one_authorization="$(python3 - "${bootstrap_response}" <<\'PY\'')
parser_end = installer.index('direct publisher-bootstrap returned an invalid Stage-1 result', parser_start)
direct_response_parser = installer[parser_start:parser_end]
if 'bootstrap_channel_bound' not in direct_response_parser:
    fail('installer direct-response parser does not require bootstrap_channel_bound')
if 'xpc_attestation' in direct_response_parser or 'audit_token_bound' in direct_response_parser:
    fail('installer direct-response parser fabricates an XPC-attestation requirement')
xpc_validator = client[client.index('pub fn attest_mac_publisher_response_v1'):]
for text in ('xpc_attestation', 'audit_token_bound'):
    if text not in xpc_validator:
        fail(f'actual XPC response validator no longer requires {text}')
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

# A private fixed install advances the protected anchor once per durable receipt.  The ordinary
# catalogue remains available to lima-stop, so its opaque requests must be reissued only from the
# final current anchor, in both the normal and post-CAS-resume Stage-1 paths.
refresh_start = executor.index('fn mac_refresh_closed_post_pm_requests_after_fixed_install_v1')
refresh_end = executor.index('\n#[cfg(not(target_os = "macos"))]', refresh_start)
refresh = executor[refresh_start:refresh_end]
for text in ('open_system_keychain_protected_state_for_scope_v1',
             'mac_issue_closed_post_pm_requests_after_stage_one_v1',
             'fixed install final protected state does not exact-join',
             'post_pm_requests_v1'):
    if text not in refresh:
        fail('fixed-install post-PM refresh is missing its current-anchor proof')
stage_runner_start = executor.index('fn execute_closed_mac_lima_stage_one_effect_v1')
stage_runner_end = executor.index('\n#[cfg(not(target_os = "macos"))]\nfn execute_closed_mac_lima_stage_one_effect_v1', stage_runner_start)
stage_runner = executor[stage_runner_start:stage_runner_end]
if stage_runner.count('mac_execute_fixed_install_sequence_v1') != 2 \
        or stage_runner.count('mac_refresh_closed_post_pm_requests_after_fixed_install_v1') != 2 \
        or 'mac_issue_closed_post_pm_requests_after_stage_one_v1(' in stage_runner:
    fail('Stage-1 normal/resume paths do not refresh ordinary requests after the private install')
fixed_start = executor.index('fn mac_execute_fixed_install_sequence_v1')
fixed_end = executor.index('\n#[cfg(target_os = "macos")]\nfn execute_mac_post_pm_action_with_policy_v1', fixed_start)
fixed = executor[fixed_start:fixed_end]
for text in ('index.authority_domain != "mac_host_shared"',
             'index.scope_id != capsule.scope_id',
             'sha256_hex_bootstrap_v1(&canonical_index)',
             'head.v1.json'):
    if text not in fixed:
        fail('fixed install receipts do not exact-join the current protected-state anchor')
for text in ('let next_anchor_revision = state',
             '.action_receipt_index_revision\n            .checked_add(1)',
             'index.previous_index_sha256.as_deref()',
             'head.previous_head_sha256.as_deref()',
             'fixed install retry tail is not the exact next planned receipt',
             'fixed install recovery has a mismatched prepared counter',
             'current_anchor_counter: state.current_anchor.action_receipt_index_revision'):
    if text not in fixed:
        fail('fixed install does not recover the exact one-receipt-ahead prepared tail')
if 'current_anchor_counter: state.counter' in fixed:
    fail('fixed install derives a recovery request counter from mutable prepared state')

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

# The warm wrapper can submit only the signed Stage-1 effect. The completion response is a
# terminal boundary: the executor privately consumes the fixed forward-only installation plan.
warm = source['warm']
stage_one = function_body(warm, 'ensure_vm_ready')
if 'lima-lifecycle.sh" stage_one_create' not in stage_one or '--lima-stage-one-authorization-v1' not in stage_one:
    fail('ensure_vm_ready is not the sole Stage-1 mapped caller')
if any(selector in stage_one for selector in (
    '--platform-bootstrap-mapping-v1', '--publisher-request-v1', '--executor-build-evidence-v1',
    'post_pm_action',
)):
    fail('ensure_vm_ready forwards a caller-selected post-PM input into Stage-1')
if 'stage_response="$(' not in stage_one or 'validate_stage_one_completion_response_v1 "${stage_response}"' not in stage_one:
    fail('ensure_vm_ready does not verify the terminal Stage-1 completion response')
completion = function_body(warm, 'validate_stage_one_completion_response_v1')
for text in ('post_pm_requests_v1 remains a catalogue', 'fixed installation sequence',
             'The privileged executor owns', '"status"', '"receipt"',
             '"manifest_generation"', '"manifest_sha256"', '"xpc_attestation"'):
    if text not in completion:
        fail('Stage-1 completion validation is not fixed to the completed executor response')
if any(forbidden in warm for forbidden in (
    'adopt_stage_one_mapping_response_v1', 'PUBLISHER_REQUEST_V1',
    'PLATFORM_BOOTSTRAP_MAPPING_V1', 'EXECUTOR_BUILD_EVIDENCE_V1',
    'lima-lifecycle.sh" post_pm_action',
)):
    fail('warm wrapper retains a caller-selected ordinary post-PM path')
configure = function_body(warm, 'configure_guest')
if any(raw in configure for raw in (
    'observe_lima_mapping_v1', 'verify_lima_mapping_v1', 'current_layout_version',
    'socket_summary', 'linger_guidance', 'limactl',
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


# Execute the fixed Stage-1 completion parser against a canonical completed response and two
# one-field tamper cases. This parser-only fixture does not invoke XPC, Lima, or any native action.
completion_fixture="$(mktemp -d)"
trap 'rm -rf "${completion_fixture}"' EXIT
python3 - "${REPO_ROOT}/scripts/mac/lima-warm.sh" "${completion_fixture}/validate-stage-one-completion" <<'PY'
from pathlib import Path
import re
import sys

source = Path(sys.argv[1]).read_text()
match = re.search(
    r'(?ms)^validate_stage_one_completion_response_v1\(\) \{\n.*?(?=^[A-Za-z_][A-Za-z0-9_]*\(\) \{|\Z)',
    source,
)
if not match:
    raise SystemExit('missing fixed Stage-1 completion validator')
Path(sys.argv[2]).write_text(
    '#!/usr/bin/env bash\n'
    'set -euo pipefail\n'
    'fatal() { printf "%s\\n" "$1" >&2; exit 1; }\n'
    + match.group(0)
    + '\nvalidate_stage_one_completion_response_v1 "$1"\n'
)
PY
chmod 700 "${completion_fixture}/validate-stage-one-completion"
canonical_completion_response='{"status":"completed","receipt":{},"manifest_generation":2,"manifest_sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","xpc_attestation":{"mach_service":"com.substrate.lifecycle.publisher.v1","audit_token_bound":true}}'
"${completion_fixture}/validate-stage-one-completion" "${canonical_completion_response}"
for tampered in \
    "${canonical_completion_response/\"status\":\"completed\"/\"status\":\"prepared\"}" \
    "${canonical_completion_response/\"audit_token_bound\":true/\"audit_token_bound\":false}"; do
    if "${completion_fixture}/validate-stage-one-completion" "${tampered}" >/dev/null 2>&1; then
        fail 'fixed Stage-1 completion validator accepted a tampered terminal response'
    fi
done
printf 'A1.1d-5R3-MAC fixed Stage-1 completion fixture: PASS\n'
