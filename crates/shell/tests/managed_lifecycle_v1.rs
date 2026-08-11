use serde_json::json;
use sha2::{Digest, Sha256};
use substrate_common::{
    ExecutorBuildEvidenceV1, LifecycleSignatureV1, LimaStageOneAuthorizationV1, ManagedActionV1,
    ManagedArtifactIdentityV1, ManagedArtifactRoleV1, ManagedExecutorIdentityV1,
    ManagedLifecyclePublisherRequestV1,
};
use substrate_shell::{
    validate_mapped_lifecycle_control_request_v1, ManagedLifecycleControlRequestV1,
    MappedLifecycleTagV1,
};
use transport_api_types::{
    InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
};

#[allow(dead_code)]
#[path = "../../../src/bin/substrate-lifecycle-control.rs"]
mod substrate_lifecycle_control;

const SCOPE_ID: &str = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90";

fn exact_carrier_and_mapping_v1() -> (String, String, String, String) {
    let carrier = InstallBootstrapContextCarrierV1::from_context(
        InstallBootstrapContextV1::new_unix("/tmp/substrate-r5", "fixture", 501)
            .expect("fixture carrier context"),
    )
    .expect("fixture carrier");
    let mapping = PlatformBootstrapMappingV1::new_lima(
        &carrier,
        "substrate",
        "0123456789abcdef0123456789abcdef",
        "/Users/fixture/.lima",
        "/home/fixture/.substrate",
        "fixture",
        501,
        "/tmp/substrate-r5/sock/agent.sock",
        "/run/substrate.sock",
    )
    .expect("fixture Lima mapping");
    let encoded_mapping = mapping.encode(&carrier).expect("encode mapping");
    let mapping_commitment = format!("{:x}", Sha256::digest(encoded_mapping.as_bytes()));
    (
        carrier.encode().expect("encode carrier"),
        encoded_mapping,
        carrier.host_context_commitment,
        mapping_commitment,
    )
}

fn exact_build_evidence_v1() -> ExecutorBuildEvidenceV1 {
    ExecutorBuildEvidenceV1 {
        schema_owner: "substrate.executor-build-evidence".to_string(),
        schema_version: 1,
        source_commit: "a".repeat(40),
        source_tree: "b".repeat(40),
        source_ref: "refs/heads/r5-fixture".to_string(),
        artifact_sha256: "c".repeat(64),
        artifact_identity: "substrate-lifecycle-macos".to_string(),
        target_triple: "aarch64-apple-darwin".to_string(),
        tool_versions: Default::default(),
        code_identity: None,
    }
}

fn post_pm_request_v1(role: &str, action: ManagedActionV1) -> ManagedLifecycleControlRequestV1 {
    let (carrier, mapping, commitment, mapping_commitment) = exact_carrier_and_mapping_v1();
    let evidence = exact_build_evidence_v1();
    ManagedLifecycleControlRequestV1 {
        tag: Some(MappedLifecycleTagV1::PostPmAction),
        authority_domain: "mac_lima_guest".to_string(),
        scope_id: SCOPE_ID.to_string(),
        selected_host_prefix: "/tmp/substrate-r5".to_string(),
        requester_principal: "fixture".to_string(),
        host_context_commitment: Some(commitment.clone()),
        platform_mapping_commitment: Some(mapping_commitment.clone()),
        host_platform_control_root: Some("/Users/fixture/.lima".to_string()),
        manifest: None,
        action_receipt: None,
        publisher_protected_state: None,
        publisher_request: Some(ManagedLifecyclePublisherRequestV1 {
            host_context_commitment: commitment,
            platform_mapping_commitment: Some(mapping_commitment),
            scope_id: SCOPE_ID.to_string(),
            current_anchor_counter: 1,
            current_anchor_sha256: "e".repeat(64),
            manifest_generation: 1,
            manifest_sha256: "f".repeat(64),
            role: ManagedArtifactRoleV1(role.to_string()),
            action,
            object_identity: ManagedArtifactIdentityV1 {
                scope_id: SCOPE_ID.to_string(),
                parent_identity: "fixture-parent".to_string(),
                name_identity: "fixture-name".to_string(),
                physical_identity: "fixture-physical".to_string(),
                metadata: None,
            },
            requester_principal: "fixture".to_string(),
            attempt_nonce: "fixture-attempt".to_string(),
            expected_executor_build: ManagedExecutorIdentityV1 {
                source_commit: evidence.source_commit.clone(),
                source_tree: evidence.source_tree.clone(),
                source_ref: evidence.source_ref.clone(),
                target_triple: evidence.target_triple.clone(),
                artifact_sha256: evidence.artifact_sha256.clone(),
                artifact_path:
                    "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1"
                        .to_string(),
                toolchain: None,
                code_identity: None,
            },
        }),
        install_bootstrap_context_v1: Some(carrier),
        platform_bootstrap_mapping_v1: Some(mapping),
        executor_build_evidence: Some(evidence),
        lima_stage_one_authorization_v1: None,
        pairing_ticket: None,
        pairing_session_binding_v1: None,
        pairing_host_record_generation: None,
        pairing_record_expected_generation_v1: None,
        pairing_host_record_sha256: None,
    }
}

fn all_closed_post_pm_pairs_v1() -> Vec<(String, ManagedActionV1)> {
    let mut pairs = Vec::new();
    let mut add = |roles: Vec<String>, actions: &[ManagedActionV1]| {
        for role in roles {
            for action in actions {
                pairs.push((role.clone(), *action));
            }
        }
    };
    add(
        vec!["mac.lima.instance".to_string()],
        &[
            ManagedActionV1::Start,
            ManagedActionV1::Stop,
            ManagedActionV1::Remove,
            ManagedActionV1::Restore,
        ],
    );
    add(
        vec![
            "mac.lima.staged-workspace".to_string(),
            "mac.lima.layout-sentinel".to_string(),
            "mac.lima.publisher-executor".to_string(),
            "mac.host.known-hosts-entry".to_string(),
            "mac.lima.guest-binary(substrate-world-service)".to_string(),
            "mac.lima.guest-binary(substrate-gateway)".to_string(),
            "mac.lima.guest-binary(substrate)".to_string(),
            "mac.lima.guest-unit(service)".to_string(),
            "mac.lima.guest-unit(socket)".to_string(),
        ],
        &[
            ManagedActionV1::Create,
            ManagedActionV1::Replace,
            ManagedActionV1::Remove,
            ManagedActionV1::Restore,
        ],
    );
    add(
        vec![
            "mac.lima.guest-group".to_string(),
            "mac.lima.guest-private-home".to_string(),
            "mac.lima.guest-directory(/run/substrate)".to_string(),
            "mac.lima.guest-directory(/run/substrate/substrate-gateway-runtime)".to_string(),
            "mac.lima.guest-directory(/var/lib/substrate)".to_string(),
            "mac.lima.guest-directory(/var/lib/substrate/.substrate-lifecycle-v1)".to_string(),
            "mac.lima.guest-directory(/var/lib/substrate/staged-workspace)".to_string(),
            "mac.lima.guest-directory(/usr/libexec/substrate)".to_string(),
            "mac.lima.guest-membership(fixture)".to_string(),
        ],
        &[
            ManagedActionV1::Create,
            ManagedActionV1::Remove,
            ManagedActionV1::Restore,
        ],
    );
    add(
        vec!["mac.lima.publisher-state-directory".to_string()],
        &[ManagedActionV1::Create, ManagedActionV1::Remove],
    );
    add(
        vec![
            "mac.lima.publisher-service-unit".to_string(),
            "mac.lima.publisher-socket-unit".to_string(),
        ],
        &[ManagedActionV1::Create, ManagedActionV1::Restore],
    );
    add(
        vec!["mac.lima.publisher-signing-key".to_string()],
        &[ManagedActionV1::Create],
    );
    add(
        vec![
            "mac.lima.publisher-current-anchor".to_string(),
            "mac.lima.publisher-bootstrap-intent".to_string(),
        ],
        &[ManagedActionV1::Create, ManagedActionV1::Replace],
    );
    add(
        vec![
            "mac.lima.guest-service-state(service)".to_string(),
            "mac.lima.guest-service-state(socket)".to_string(),
        ],
        &[
            ManagedActionV1::Enable,
            ManagedActionV1::Disable,
            ManagedActionV1::Start,
            ManagedActionV1::Stop,
            ManagedActionV1::Restore,
        ],
    );
    pairs
}

#[test]
fn mapped_lifecycle_decoder_accepts_only_two_tags() {
    for tag in ["stage_one_create", "post_pm_action"] {
        let decoded: ManagedLifecycleControlRequestV1 =
            serde_json::from_value(json!({"tag": tag})).expect("the two ordinary tags must decode");
        assert_eq!(decoded.tag.is_some(), true);
    }
    for tag in [
        "publisher_bootstrap_direct_interactive",
        "lima-action",
        "unknown",
    ] {
        assert!(
            serde_json::from_value::<ManagedLifecycleControlRequestV1>(json!({"tag": tag}))
                .is_err(),
            "{tag} must reject before any client/XPC effect"
        );
    }
}

#[test]
fn r6_dual_session_tags_are_fixed_and_not_ordinary_admission() {
    let data: MappedLifecycleTagV1 = serde_json::from_value(json!("guest_pairing_data_session"))
        .expect("R6 data session tag remains a typed protocol discriminant");
    assert!(
        serde_json::from_value::<MappedLifecycleTagV1>(json!("guest_pairing_operator_tty_session"))
            .is_err(),
        "the direct-only operator tag must not decode through the public mapped request"
    );

    {
        let request = ManagedLifecycleControlRequestV1 {
            tag: Some(data),
            authority_domain: String::new(),
            scope_id: String::new(),
            selected_host_prefix: String::new(),
            requester_principal: String::new(),
            host_context_commitment: None,
            platform_mapping_commitment: None,
            host_platform_control_root: None,
            manifest: None,
            action_receipt: None,
            publisher_protected_state: None,
            publisher_request: None,
            install_bootstrap_context_v1: None,
            platform_bootstrap_mapping_v1: None,
            executor_build_evidence: None,
            lima_stage_one_authorization_v1: None,
            pairing_ticket: None,
            pairing_session_binding_v1: None,
            pairing_host_record_generation: None,
            pairing_record_expected_generation_v1: None,
            pairing_host_record_sha256: None,
        };
        assert!(validate_mapped_lifecycle_control_request_v1(&request).is_err());
    }
}

#[test]
fn r6_correction_uses_direct_terminal_launch_and_independent_proof() {
    let control = include_str!("../../../src/bin/substrate-lifecycle-control.rs");
    let client = include_str!("../src/execution/managed_lifecycle/macos_client.rs");
    let macos = include_str!("../../../src/bin/substrate-lifecycle-macos.rs");
    let linux = include_str!("../../../src/bin/substrate-lifecycle-linux.rs");

    for text in [
        "GuestPublisherPairingOperatorLaunchV1",
        "validate_guest_publisher_pairing_operator_launch",
        "direct duplicated /dev/tty",
        "guest-pairing-operator-tty-session-v1",
    ] {
        assert!(
            control.contains(text),
            "direct R6 control launch lacks {text}"
        );
    }
    for forbidden in [
        "with_terminal_v1",
        "open_mac_xpc_channel_with_terminal_v1",
        "guest-pairing-operator-tty-session",
        "xpc_dictionary_set_fd",
    ] {
        assert!(
            !client.contains(forbidden),
            "macOS client retains forbidden R6 terminal relay {forbidden}"
        );
    }
    for text in [
        "R6_DATA_FRAME_TIMEOUT_V1",
        "persist_r6_guest_pairing_failure_observation_v1",
        "active_r6_record_exactly_rejoins_issue_v1",
        "GuestPublisherPairingOperatorLaunchV1",
    ] {
        assert!(macos.contains(text), "macOS R6 correction lacks {text}");
    }
    for forbidden in [
        "operator_tty\".as_ptr()",
        "xpc_dictionary_get_fd",
        "relay raw retained terminal input",
        "relay fixed R6 guest PTY output",
        "guest_state_root_prepared",
    ] {
        assert!(
            !macos.contains(forbidden) && !linux.contains(forbidden),
            "R6 retains forbidden {forbidden} authority channel"
        );
    }
    for text in [
        "measure_r6_installed_guest_executor_v1",
        "R6 running guest executor digest does not match the staged binding",
        "R6TerminalEchoGuardV1",
        "GuestPublisherPairingOperatorProofV1",
        "validate_guest_publisher_pairing_operator_proof",
        "validate_r6_operator_proof_before_intent_v1",
    ] {
        assert!(
            linux.contains(text),
            "Lima guest R6 correction lacks {text}"
        );
    }
}

#[test]
fn mac_xpc_clients_register_a_no_capture_handler_before_activation() {
    let client = include_str!("../src/execution/managed_lifecycle/macos_client.rs");
    let executor = include_str!("../../../src/bin/substrate-lifecycle-macos.rs");

    let client_open = &client[client
        .find("pub fn open_mac_xpc_channel_v1")
        .expect("macOS shell XPC client")
        ..client
            .find("/// Verify the fixed publisher service")
            .expect("macOS shell XPC attestation boundary")];
    let executor_relay = &executor[executor
        .find("fn relay_mac_xpc_publisher_request_v1")
        .expect("macOS lifecycle XPC relay")
        ..executor
            .find("/// Direct System-Keychain/Security framework fence")
            .expect("macOS lifecycle XPC relay boundary")];

    for (name, source, full_source) in [
        ("shell XPC client", client_open, client),
        ("lifecycle XPC relay", executor_relay, executor),
    ] {
        let handler = source
            .find("install_mac_xpc_no_capture_event_handler_v1")
            .unwrap_or_else(|| panic!("{name} does not install a no-capture XPC handler"));
        let activation = source
            .find("xpc_connection_activate(connection)")
            .unwrap_or_else(|| panic!("{name} does not activate its XPC connection"));
        assert!(
            handler < activation,
            "{name} activates the XPC connection before registering its event handler"
        );
        assert!(
            full_source.contains("_Block_copy(")
                && full_source
                    .contains("xpc_connection_set_event_handler(connection, owned_block)")
                && full_source.contains("_Block_release(owned_block)"),
            "{name} does not make the handler's Blocks-ABI copy/release lifetime explicit"
        );
    }
}

#[test]
fn post_pm_enumerates_every_closed_mac_role_action_pair() {
    let pairs = all_closed_post_pm_pairs_v1();
    assert!(!pairs.is_empty());
    for (role, action) in pairs {
        validate_mapped_lifecycle_control_request_v1(&post_pm_request_v1(&role, action))
            .unwrap_or_else(|error| panic!("closed pair {role}/{action:?} rejected: {error:#}"));
    }

    let mut bad_top_level = post_pm_request_v1("mac.lima.instance", ManagedActionV1::Start);
    bad_top_level.platform_mapping_commitment = Some("d".repeat(64));
    assert!(
        validate_mapped_lifecycle_control_request_v1(&bad_top_level).is_err(),
        "the top-level mapping commitment must bind the exact canonical mapping"
    );

    let mut bad_publisher = post_pm_request_v1("mac.lima.instance", ManagedActionV1::Start);
    bad_publisher
        .publisher_request
        .as_mut()
        .expect("fixture request")
        .platform_mapping_commitment = Some("d".repeat(64));
    assert!(
        validate_mapped_lifecycle_control_request_v1(&bad_publisher).is_err(),
        "the canonical publisher request mapping commitment must bind the exact canonical mapping"
    );
}

#[test]
fn post_pm_rejects_generated_unlisted_pairs_and_stage_one_reuse() {
    let actions = [
        ManagedActionV1::Create,
        ManagedActionV1::Replace,
        ManagedActionV1::Remove,
        ManagedActionV1::Restore,
        ManagedActionV1::Enable,
        ManagedActionV1::Disable,
        ManagedActionV1::Start,
        ManagedActionV1::Stop,
    ];
    for role in [
        "mac.publisher.executor",
        "mac.publisher.mach-service",
        "mac.publisher.service-state",
        "mac.lima.publisher-endpoint",
        "mac.lima.publisher-service-state(service)",
        "mac.lima.guest-socket",
        "mac.host.forward-socket",
        "mac.host.ssh-forwarder",
        "mac.lima.guest-binary(unlisted)",
        "mac.lima.guest-binary(world)",
        "mac.lima.guest-directory(/tmp/unlisted)",
        "unlisted.role",
    ] {
        for action in actions {
            assert!(
                validate_mapped_lifecycle_control_request_v1(&post_pm_request_v1(role, action))
                    .is_err(),
                "unlisted pair {role}/{action:?} must fail before XPC"
            );
        }
    }

    let mut stage = post_pm_request_v1("mac.lima.instance", ManagedActionV1::Start);
    stage.tag = Some(MappedLifecycleTagV1::StageOneCreate);
    assert!(validate_mapped_lifecycle_control_request_v1(&stage).is_err());

    let mut post_with_stage = post_pm_request_v1("mac.lima.instance", ManagedActionV1::Stop);
    post_with_stage.lima_stage_one_authorization_v1 = Some(LimaStageOneAuthorizationV1 {
        schema_owner: "wrong".to_string(),
        schema_version: 0,
        host_context_commitment: "invalid".to_string(),
        lima_control_root_identity: "invalid".to_string(),
        instance_name: "invalid".to_string(),
        profile_sha256: "invalid".to_string(),
        expected_absent: false,
        source_commit: "invalid".to_string(),
        source_tree: "invalid".to_string(),
        source_ref: "invalid".to_string(),
        executor_receipt_sha256: "invalid".to_string(),
        requester_principal: "invalid".to_string(),
        peer_code_requirement: "invalid".to_string(),
        attempt_id: "invalid".to_string(),
        nonce: "invalid".to_string(),
        expires_at_unix_ns: 0,
        rendered_profile_b64: "e30".to_string(),
        rendered_profile_sha256: "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
            .to_string(),
        successor_template: serde_json::from_value(json!({
            "schema_owner": "substrate.mac-lima-stage-one-successor-template",
            "schema_version": 1,
            "derivation_algorithm": "mac_lima_stage_one_successor_template_v1_platform_mapping_then_manifest",
            "current_pre_pm_manifest_generation": 1,
            "current_pre_pm_manifest_sha256": "f".repeat(64),
            "current_anchor_sha256": "b".repeat(64),
            "current_anchor_counter": 0,
            "next_manifest_generation": 2,
            "previous_manifest_sha256": "f".repeat(64),
            "host_context_commitment": "e".repeat(64),
            "scope_id": SCOPE_ID,
            "installation_id": SCOPE_ID,
            "intended_principal": "fixture",
            "selected_host_prefix": "/tmp/substrate-r5",
            "host_platform_control_root": "/Users/fixture/.lima",
            "instance_name": "substrate",
            "profile_sha256": "a".repeat(64),
            "source_commit": "a".repeat(40),
            "source_tree": "b".repeat(40),
            "source_ref": "refs/heads/r5-fixture",
            "executor_build_evidence": {
                "schema_owner": "substrate.executor-build-evidence",
                "schema_version": 1,
                "source_commit": "a".repeat(40),
                "source_tree": "b".repeat(40),
                "source_ref": "refs/heads/r5-fixture",
                "artifact_sha256": "c".repeat(64),
                "artifact_identity": "substrate-lifecycle-macos",
                "target_triple": "aarch64-apple-darwin",
                "tool_versions": {},
                "code_identity": null
            },
            "attempt_id": "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91",
            "nonce": "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92",
            "expires_at_unix_ns": 2,
            "manifest_created_at_unix_ns": 1,
            "manifest_lifecycle_state": "manifest_durable",
            "profile_template_algorithm": "substrate.mac-lima-stage-one-profile-template",
            "profile_template_version": 1,
            "profile_template_sha256": "d".repeat(64),
            "ordered_non_machine_entries": [],
            "planned_receipt_id": "stage-one-fixture",
            "planned_receipt_relative_path": "receipts/2/stage-one-fixture.json",
            "post_effect_observation_slots": ["guest_machine_id"]
        })).expect("stage-one successor template fixture shape"),
        signature: LifecycleSignatureV1 {
            algorithm: "invalid".to_string(),
            public_key: "invalid".to_string(),
            signature: "invalid".to_string(),
        },
    });
    assert!(validate_mapped_lifecycle_control_request_v1(&post_with_stage).is_err());
}

#[test]
fn direct_bootstrap_dispatch_is_pre_stdin_and_fd3_only() {
    let control = include_str!("../../../src/bin/substrate-lifecycle-control.rs");
    let bootstrap_dispatch = control
        .find("\"publisher-bootstrap\" =>")
        .expect("hidden direct bootstrap dispatch");
    let stdin_dispatch = control
        .find("\"submit-mapped-lifecycle-v1\" =>")
        .expect("ordinary stdin dispatch");
    assert!(bootstrap_dispatch < stdin_dispatch);
    let direct = &control[control
        .find("pub fn publisher_bootstrap_direct_interactive_v1")
        .expect("direct bootstrap function")
        ..control
            .find("pub fn guest_publisher_pairing_direct_interactive_v1")
            .expect("next legacy function")];
    assert!(direct.contains("deliver_retained_publisher_bootstrap_authorization_v1"));
    assert!(direct.contains("MacPublisherBootstrapRequestV1"));
    assert!(!direct.contains("ManagedLifecycleControlRequestV1"));
    assert!(control.contains("read_exact_publisher_bootstrap_request_from_stdin_v1"));
    assert!(control.contains("parse_mac_publisher_bootstrap_request_v1"));
    assert!(control.contains("contains only the exact IH carrier"));
    assert!(!control.contains("read_exact_publisher_bootstrap_seed_from_stdin_v1"));

    let client = include_str!("../src/execution/managed_lifecycle/macos_client.rs");
    assert!(client.contains("libc::AF_UNIX"));
    assert!(client.contains("libc::SOCK_STREAM"));
    assert!(!client.contains("libc::SOCK_SEQPACKET"));
    assert!(client.contains("libc::FD_CLOEXEC"));
    assert!(client.contains("libc::SO_NOSIGPIPE"));
    assert!(client.contains("libc::MSG_DONTWAIT"));
    assert!(client.contains("libc::SHUT_WR"));
    assert!(client.contains("OwnedFd::from_raw_fd"));
    assert!(client.contains("parse_canonical_direct_bootstrap_response_v1"));
    assert!(client.contains("bootstrap_channel_bound"));
    assert!(client.contains("RetainedBootstrapChildGuardV1"));
    assert!(client.contains("--publisher-bootstrap-fd"));
    assert!(client.contains(".arg(\"3\")"));
    assert!(client.contains("canonical_publisher_bootstrap_authorization_v1"));
    assert!(client.contains("measure_bootstrap_image_v1"));
    assert!(client.contains("measure_codesign_cdhash_v1"));
    assert!(client.contains("control and executor artifacts must be distinct"));

    let executor = include_str!("../../../src/bin/substrate-lifecycle-macos.rs");
    let fd_dispatch = executor
        .find("if operation == \"--publisher-bootstrap-fd\"")
        .expect("FD3 entrypoint");
    let stdin_read = executor
        .find("read_to_end(&mut input)")
        .expect("ordinary stdin read");
    assert!(fd_dispatch < stdin_read);
    let fd3_consumer = &executor[executor
        .find("fn consume_publisher_bootstrap_fd3_v1")
        .expect("FD3 consumer")
        ..executor
            .find("struct MacBootstrapPeerIdentityV1")
            .expect("FD3 peer identity")];
    let fd_check = fd3_consumer.find("if fd != 3").expect("exact FD3 check");
    let cloexec = fd3_consumer
        .find("mac_rearm_bootstrap_fd3_cloexec_v1(fd)?")
        .expect("FD3 CLOEXEC re-arm");
    let socket_type = fd3_consumer
        .find("mac_require_stream_channel_v1(fd)?")
        .expect("FD3 stream check");
    let peer = fd3_consumer
        .find("mac_bootstrap_peer_identity_v1")
        .expect("FD3 peer check");
    let decode = fd3_consumer
        .find("parse_publisher_bootstrap_authorization_v1")
        .expect("FD3 canonical decode");
    assert!(fd_check < cloexec && cloexec < socket_type && socket_type < peer && peer < decode);
    assert!(executor.contains("libc::SOCK_STREAM"));
    assert!(executor.contains("libc::SO_NOSIGPIPE"));
    assert!(executor.contains("mac_read_single_stream_document_v1"));
    assert!(executor.contains("mac_send_single_stream_document_v1"));
    assert!(executor.contains("getpeereid"));
    assert!(executor.contains("parse_publisher_bootstrap_authorization_v1"));
    assert!(executor.contains("MacLimaStageOneCapsuleV1"));
    assert!(executor.contains("prepared_protected_state_sha256"));
    assert!(executor.contains("derive_mac_lima_stage_one_authorization_v1"));
    assert!(executor.contains("render_mac_lima_stage_one_profile_v1"));
    assert!(executor.contains("mac_run_fixed_lima_command_v1"));
    assert!(executor.contains("LimaStageOneObservationV1"));
    assert!(executor.contains("EffectStarted"));
    assert!(executor.contains("InstanceObserved"));
    assert!(executor.contains("PreservingBlocked"));
    assert!(executor.contains("bootstrap-intent"));
    assert!(executor.contains("Prepared"));
    assert!(executor.contains("Completed"));
    assert!(executor.contains("mac_ensure_system_keychain_p256_spki_der_v1"));
    assert!(executor.contains("SecKeyCreateRandomKey"));
    assert!(executor.contains("mac_transition_bootstrap_intent_v1"));
    assert!(executor.contains("manifest_generation: authorization.manifest_generation"));
    assert!(executor.contains("manifest_sha256: authorization.manifest_sha256.clone()"));
    assert!(executor.contains("mac_attest_running_executor_image_v1"));
    assert!(executor.contains("LOCAL_PEERPID"));
    assert!(executor.contains("publisher_bootstrap_authorization_sha256_v1"));

    let installer = include_str!("../../../scripts/substrate/dev-install-substrate.sh");
    let direct_parser_start = installer
        .find("stage_one_authorization=\"$(python3 - \"${bootstrap_response}\" <<'PY'")
        .expect("direct response parser");
    let direct_parser_end = installer[direct_parser_start..]
        .find("direct publisher-bootstrap returned an invalid Stage-1 result")
        .map(|offset| direct_parser_start + offset)
        .expect("direct response parser end");
    let direct_parser = &installer[direct_parser_start..direct_parser_end];
    assert!(direct_parser.contains("bootstrap_channel_bound"));
    assert!(!direct_parser.contains("xpc_attestation"));
    assert!(!direct_parser.contains("audit_token_bound"));
    let xpc_attestation = &client[client
        .find("pub fn attest_mac_publisher_response_v1")
        .expect("XPC attestation validator")..];
    assert!(xpc_attestation.contains("xpc_attestation"));
    assert!(xpc_attestation.contains("audit_token_bound"));
    let resume = &executor[executor
        .find("fn resume_mac_lima_stage_one_after_protected_state_cas_v1")
        .expect("Stage-1 post-CAS resume")
        ..executor
            .find("fn execute_closed_mac_lima_stage_one_effect_v1")
            .expect("Stage-1 effect")];
    assert!(resume.contains("mac_read_stage_one_transition_artifact_no_follow_v1"));
    assert!(resume.contains("canonical_action_receipt_index_bytes_v1"));
    assert!(resume.contains("replace_mac_control_admission_authority_v1"));
    assert!(
        !resume.contains("mac_run_fixed_lima_command_v1"),
        "a durable post-CAS retry must not repeat the Lima effect"
    );
    let receipt_retry = &executor[executor
        .find("fn load_or_sign_mac_lima_stage_one_receipt_v1")
        .expect("Stage-1 receipt retry")
        ..executor
            .find("fn prepare_mac_lima_stage_one_transition_v1")
            .expect("Stage-1 prepare")];
    assert!(receipt_retry.contains("retained Stage-1 receipt is not an exact retry"));
    assert!(receipt_retry.contains("validate_managed_action_receipt_signature_v1"));
}

#[test]
fn mac_publisher_service_state_retry_matrix_is_preserving_first() {
    use substrate_common::{
        classify_mac_publisher_service_install_v1, classify_mac_publisher_service_retirement_v1,
        MacPublisherServiceFilesObservationV1 as Files,
        MacPublisherServiceInstallDecisionV1 as Install,
        MacPublisherServiceRecordObservationV1 as Record,
        MacPublisherServiceRegistrationObservationV1 as Registration,
        MacPublisherServiceRetirementDecisionV1 as Retire,
    };

    assert_eq!(
        classify_mac_publisher_service_install_v1(
            Registration::Absent,
            Files::Exact,
            Record::InstallIntentMatching,
        ),
        Install::PrecommitAndBootstrap,
    );
    assert_eq!(
        classify_mac_publisher_service_install_v1(
            Registration::Absent,
            Files::Exact,
            Record::InstallPrecommittedMatching,
        ),
        Install::BootstrapFromPrecommit,
    );
    assert_eq!(
        classify_mac_publisher_service_install_v1(
            Registration::Registered,
            Files::Exact,
            Record::InstalledMatching,
        ),
        Install::IdempotentSuccess,
    );
    for (registration, files, record) in [
        (
            Registration::Registered,
            Files::Exact,
            Record::InstallPrecommittedMatching,
        ),
        (
            Registration::Absent,
            Files::Exact,
            Record::InstalledMatching,
        ),
        (Registration::Absent, Files::Exact, Record::None),
        (
            Registration::Absent,
            Files::Ambiguous,
            Record::InstallIntentMatching,
        ),
        (
            Registration::Indeterminate,
            Files::Exact,
            Record::InstalledMatching,
        ),
        (Registration::Registered, Files::Exact, Record::Mismatched),
    ] {
        assert_eq!(
            classify_mac_publisher_service_install_v1(registration, files, record),
            Install::PreserveAndStop,
        );
    }

    assert_eq!(
        classify_mac_publisher_service_retirement_v1(
            Registration::Registered,
            Files::Exact,
            Record::InstalledMatching,
        ),
        Retire::PrecommitAndBootout,
    );
    assert_eq!(
        classify_mac_publisher_service_retirement_v1(
            Registration::Registered,
            Files::Exact,
            Record::RetirementPrecommittedMatching,
        ),
        Retire::BootoutFromPrecommit,
    );
    assert_eq!(
        classify_mac_publisher_service_retirement_v1(
            Registration::Absent,
            Files::Exact,
            Record::RetirementPrecommittedMatching,
        ),
        Retire::DeleteExactFiles,
    );
    assert_eq!(
        classify_mac_publisher_service_retirement_v1(
            Registration::Absent,
            Files::Absent,
            Record::RetirementPrecommittedMatching,
        ),
        Retire::CommitRetired,
    );
    assert_eq!(
        classify_mac_publisher_service_retirement_v1(
            Registration::Absent,
            Files::Absent,
            Record::RetiredMatching,
        ),
        Retire::IdempotentSuccess,
    );
    for (registration, files, record) in [
        (Registration::Registered, Files::Exact, Record::None),
        (
            Registration::Registered,
            Files::Ambiguous,
            Record::InstalledMatching,
        ),
        (
            Registration::Indeterminate,
            Files::Exact,
            Record::InstalledMatching,
        ),
        (
            Registration::Absent,
            Files::Exact,
            Record::InstalledMatching,
        ),
        (Registration::Registered, Files::Exact, Record::Mismatched),
    ] {
        assert_eq!(
            classify_mac_publisher_service_retirement_v1(registration, files, record),
            Retire::PreserveAndStop,
        );
    }
}

#[test]
fn mac_publisher_service_state_record_is_canonical_and_closed() {
    use substrate_common::{
        canonical_mac_publisher_service_state_record_v1,
        mac_publisher_service_state_record_sha256_v1, MacPublisherServiceFileIdentityV1,
        MacPublisherServiceStatePhaseV1, MacPublisherServiceStateRecordV1,
    };

    let record = MacPublisherServiceStateRecordV1 {
        schema_owner: "substrate.mac-publisher-service-state".to_string(),
        schema_version: 1,
        scope_id: SCOPE_ID.to_string(),
        bootstrap_authorization_sha256: "a".repeat(64),
        bootstrap_intent_sha256: "9".repeat(64),
        initial_anchor_sha256: "b".repeat(64),
        protected_state_sha256: "c".repeat(64),
        stage_one_capsule_sha256: "d".repeat(64),
        install_provenance_sha256: "e".repeat(64),
        bootstrap_attempt_locator_account: "mac-publisher-bootstrap-attempt-locator-v1:"
            .to_string()
            + &"8".repeat(64),
        service_label: "com.substrate.lifecycle.publisher.v1".to_string(),
        launchd_domain: "system".to_string(),
        program_arguments: vec![
            "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1".to_string(),
            "run-publisher".to_string(),
        ],
        mach_services: vec!["com.substrate.lifecycle.publisher.v1".to_string()],
        helper: MacPublisherServiceFileIdentityV1 {
            path: "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1".to_string(),
            artifact_sha256: "f".repeat(64),
            physical_identity: "dev:1:ino:2".to_string(),
            owner_uid: 0,
            group_gid: 0,
            mode: "0755".to_string(),
            code_identity: Some("cdhash:0123456789abcdef0123456789abcdef01234567".to_string()),
            code_requirement: Some(
                "cdhash H\"0123456789abcdef0123456789abcdef01234567\"".to_string(),
            ),
        },
        plist: MacPublisherServiceFileIdentityV1 {
            path: "/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist".to_string(),
            artifact_sha256: "0".repeat(64),
            physical_identity: "dev:1:ino:3".to_string(),
            owner_uid: 0,
            group_gid: 0,
            mode: "0644".to_string(),
            code_identity: None,
            code_requirement: None,
        },
        provenance: MacPublisherServiceFileIdentityV1 {
            path: "/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json"
                .to_string(),
            artifact_sha256: "1".repeat(64),
            physical_identity: "dev:1:ino:4".to_string(),
            owner_uid: 0,
            group_gid: 0,
            mode: "0444".to_string(),
            code_identity: None,
            code_requirement: None,
        },
        phase: MacPublisherServiceStatePhaseV1::InstallPrecommitted,
        registration_observed: false,
        files_observed_present: true,
        record_revision: 1,
        retirement_delete_cursor: 0,
        previous_record_sha256: None,
    };
    let canonical = canonical_mac_publisher_service_state_record_v1(&record)
        .expect("canonical service-state record");
    let decoded: MacPublisherServiceStateRecordV1 =
        serde_json::from_slice(&canonical).expect("decode canonical record");
    assert_eq!(decoded, record);

    let installed = MacPublisherServiceStateRecordV1 {
        phase: MacPublisherServiceStatePhaseV1::Installed,
        registration_observed: true,
        record_revision: 2,
        previous_record_sha256: Some(
            mac_publisher_service_state_record_sha256_v1(&record).expect("precommit digest"),
        ),
        ..record.clone()
    };
    canonical_mac_publisher_service_state_record_v1(&installed).expect("installed revision two");
    let mut predecessor = installed.clone();
    for cursor in 0_u8..=3 {
        let retirement = MacPublisherServiceStateRecordV1 {
            phase: MacPublisherServiceStatePhaseV1::RetirementPrecommitted,
            record_revision: 3 + u64::from(cursor),
            retirement_delete_cursor: cursor,
            previous_record_sha256: Some(
                mac_publisher_service_state_record_sha256_v1(&predecessor)
                    .expect("retirement predecessor digest"),
            ),
            ..installed.clone()
        };
        canonical_mac_publisher_service_state_record_v1(&retirement)
            .expect("exact retirement cursor revision");
        predecessor = retirement;
    }
    let retired = MacPublisherServiceStateRecordV1 {
        phase: MacPublisherServiceStatePhaseV1::Retired,
        registration_observed: false,
        files_observed_present: false,
        record_revision: 7,
        retirement_delete_cursor: 3,
        previous_record_sha256: Some(
            mac_publisher_service_state_record_sha256_v1(&predecessor)
                .expect("retired predecessor digest"),
        ),
        ..installed.clone()
    };
    canonical_mac_publisher_service_state_record_v1(&retired).expect("retired revision seven");

    for invalid in [
        MacPublisherServiceStateRecordV1 {
            service_label: "foreign.label".to_string(),
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            launchd_domain: "user/501".to_string(),
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            program_arguments: vec!["/bin/sh".to_string(), "-c".to_string()],
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            mach_services: vec!["foreign.mach-service".to_string()],
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            helper: MacPublisherServiceFileIdentityV1 {
                path: "/tmp/foreign-helper".to_string(),
                ..record.helper.clone()
            },
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            bootstrap_intent_sha256: "stale".to_string(),
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            bootstrap_attempt_locator_account: "caller-selected".to_string(),
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            registration_observed: true,
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            phase: MacPublisherServiceStatePhaseV1::Installed,
            registration_observed: true,
            record_revision: 1,
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            phase: MacPublisherServiceStatePhaseV1::Installed,
            registration_observed: true,
            record_revision: 2,
            previous_record_sha256: Some("2".repeat(64)),
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            phase: MacPublisherServiceStatePhaseV1::RetirementPrecommitted,
            registration_observed: true,
            record_revision: 4,
            retirement_delete_cursor: 0,
            previous_record_sha256: Some("2".repeat(64)),
            ..record.clone()
        },
        MacPublisherServiceStateRecordV1 {
            phase: MacPublisherServiceStatePhaseV1::Retired,
            registration_observed: false,
            files_observed_present: false,
            record_revision: 7,
            retirement_delete_cursor: 2,
            previous_record_sha256: Some("2".repeat(64)),
            ..record.clone()
        },
    ] {
        assert!(canonical_mac_publisher_service_state_record_v1(&invalid).is_err());
    }

    let mut unknown = serde_json::to_value(&record).expect("serialize fixture record");
    unknown
        .as_object_mut()
        .expect("record object")
        .insert("action".to_string(), json!("arbitrary-launchctl"));
    assert!(serde_json::from_value::<MacPublisherServiceStateRecordV1>(unknown).is_err());
}

#[test]
#[ignore = "bounded native macOS launchd demand proof; requires disposable installed state"]
fn native_mac_publisher_service_demand_admits_control_before_stale_state_rejection() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let prefix = std::env::var("SUBSTRATE_R3_NATIVE_PREFIX").expect("native proof prefix");
    let carrier_encoded =
        std::env::var("SUBSTRATE_R3_NATIVE_CARRIER").expect("native proof carrier");
    let bootstrap: serde_json::Value = serde_json::from_str(
        &std::env::var("SUBSTRATE_R3_NATIVE_BOOTSTRAP_RESPONSE")
            .expect("native proof bootstrap response"),
    )
    .expect("decode native bootstrap response");
    let stage: LimaStageOneAuthorizationV1 = serde_json::from_value(
        bootstrap
            .get("lima_stage_one_authorization_v1")
            .cloned()
            .expect("native Stage-1 authorization"),
    )
    .expect("decode native Stage-1 authorization");
    let carrier = InstallBootstrapContextCarrierV1::decode(&carrier_encoded)
        .expect("decode native proof carrier");
    let mapping = PlatformBootstrapMappingV1::new_lima(
        &carrier,
        "substrate-r3-service-proof-never-create",
        "0123456789abcdef0123456789abcdef",
        &format!("{prefix}/proof-lima-control"),
        "/home/proof/.substrate",
        "proof",
        501,
        &format!("{prefix}/proof-agent.sock"),
        "/run/substrate-proof.sock",
    )
    .expect("construct inert native proof mapping");
    let mapping_encoded = mapping
        .encode(&carrier)
        .expect("encode inert proof mapping");
    let mapping_commitment = format!("{:x}", Sha256::digest(mapping_encoded.as_bytes()));
    let evidence = stage.successor_template.executor_build_evidence.clone();
    let scope_id = stage.successor_template.scope_id.clone();
    let requester = "proof".to_string();
    let publisher_request = ManagedLifecyclePublisherRequestV1 {
        host_context_commitment: carrier.host_context_commitment.clone(),
        platform_mapping_commitment: Some(mapping_commitment.clone()),
        scope_id: scope_id.clone(),
        current_anchor_counter: 0,
        // Deliberately stale but structurally valid: peer admission and request decode occur, then
        // the protected state join rejects before any Lima effect can be selected.
        current_anchor_sha256: "0".repeat(64),
        manifest_generation: bootstrap["manifest_generation"]
            .as_u64()
            .expect("native manifest generation"),
        manifest_sha256: bootstrap["manifest_sha256"]
            .as_str()
            .expect("native manifest digest")
            .to_string(),
        role: ManagedArtifactRoleV1("mac.lima.instance".to_string()),
        action: ManagedActionV1::Start,
        object_identity: ManagedArtifactIdentityV1 {
            scope_id: scope_id.clone(),
            parent_identity: "native-proof-no-effect-parent".to_string(),
            name_identity: "substrate-r3-service-proof-never-create".to_string(),
            physical_identity: "native-proof-no-effect-identity".to_string(),
            metadata: None,
        },
        requester_principal: requester.clone(),
        attempt_nonce: "native-proof-stale-state".to_string(),
        expected_executor_build: ManagedExecutorIdentityV1 {
            source_commit: evidence.source_commit.clone(),
            source_tree: evidence.source_tree.clone(),
            source_ref: evidence.source_ref.clone(),
            target_triple: evidence.target_triple.clone(),
            artifact_sha256: evidence.artifact_sha256.clone(),
            artifact_path: "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1"
                .to_string(),
            toolchain: None,
            code_identity: evidence
                .code_identity
                .clone()
                .map(serde_json::Value::String),
        },
    };
    let request = ManagedLifecycleControlRequestV1 {
        tag: Some(MappedLifecycleTagV1::PostPmAction),
        authority_domain: "mac_lima_guest".to_string(),
        scope_id,
        selected_host_prefix: carrier.context.selected_host_prefix.clone(),
        requester_principal: requester,
        host_context_commitment: Some(carrier.host_context_commitment.clone()),
        platform_mapping_commitment: Some(mapping_commitment),
        host_platform_control_root: Some(mapping.host_platform_control_root.clone()),
        manifest: None,
        action_receipt: None,
        publisher_protected_state: None,
        publisher_request: Some(publisher_request),
        install_bootstrap_context_v1: Some(carrier_encoded),
        platform_bootstrap_mapping_v1: Some(mapping_encoded),
        executor_build_evidence: Some(evidence),
        lima_stage_one_authorization_v1: None,
        pairing_ticket: None,
        pairing_session_binding_v1: None,
        pairing_host_record_generation: None,
        pairing_record_expected_generation_v1: None,
        pairing_host_record_sha256: None,
    };
    validate_mapped_lifecycle_control_request_v1(&request)
        .expect("native proof request is structurally admitted before protected-state rejection");
    let request_bytes = serde_json::to_vec(&request).expect("encode native proof request");
    if let Ok(path) = std::env::var("SUBSTRATE_R3_NATIVE_REQUEST_OUT") {
        std::fs::write(path, &request_bytes).expect("write native proof request fixture");
    }
    let mut child = std::process::Command::new(format!("{prefix}/bin/substrate-lifecycle-control"))
        .arg("submit-mapped-lifecycle-v1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn installed native proof control");
    use std::io::Write as _;
    child
        .stdin
        .take()
        .expect("native proof control stdin")
        .write_all(&request_bytes)
        .expect("write native proof control request");
    let output = child.wait_with_output().expect("wait native proof control");
    assert!(
        !output.status.success(),
        "stale protected-state request must reject"
    );
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    for transport_failure in [
        "connection-invalid",
        "connection-interrupted",
        "peer-code-signing-requirement",
        "malformed-frame",
        "missing peer_code_requirement",
    ] {
        assert!(
            !diagnostics.contains(transport_failure),
            "control peer was not admitted through demand activation: {diagnostics}"
        );
    }
    assert!(
        diagnostics.contains("audit-token") || diagnostics.contains("protected"),
        "native proof did not reach admitted protected-state rejection: {diagnostics}"
    );
}
