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
    assert!(client.contains("libc::SOCK_SEQPACKET"));
    assert!(client.contains("libc::FD_CLOEXEC"));
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
