use assert_cmd::Command;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use substrate_common::{
    lifecycle_anchor_sha256_v1, managed_action_prepared_record_sha256_v1,
    sign_lifecycle_anchor_for_test_v1, sign_managed_action_prepared_record_for_test_v1,
    sign_managed_action_receipt_for_test_v1, GuestPublisherPairingChallengeV1,
    GuestPublisherPairingTicketV1, LifecyclePublisherAnchorV1, LifecyclePublisherProtectedStateV1,
    LifecycleSignatureV1, ManagedActionPreparedRecordV1, ManagedActionReceiptV1, ManagedActionV1,
    ManagedArtifactIdentityV1, ManagedArtifactManifestV1, ManagedArtifactRoleV1,
    ManagedExecutorIdentityV1, ManagedLifecyclePublisherRequestV1, ManagedLifecycleStateV1,
};
use substrate_shell::ManagedLifecycleControlRequestV1;
use tempfile::tempdir;

#[allow(dead_code)]
#[path = "../../../src/bin/substrate-lifecycle-control.rs"]
mod substrate_lifecycle_control;

#[path = "common.rs"]
mod common;

fn control_binary_path() -> PathBuf {
    common::ensure_substrate_built();
    let binary_name = if cfg!(windows) {
        "substrate-lifecycle-control.exe"
    } else {
        "substrate-lifecycle-control"
    };
    if let Ok(workspace_dir) = std::env::var("CARGO_WORKSPACE_DIR") {
        PathBuf::from(workspace_dir)
            .join("target")
            .join("debug")
            .join(binary_name)
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug")
            .join(binary_name)
    }
}

fn sample_manifest(selected_host_prefix: &Path) -> ManagedArtifactManifestV1 {
    let selected_host_prefix = selected_host_prefix.display().to_string();
    let mut manifest = ManagedArtifactManifestV1 {
        schema_owner: "substrate.managed-artifact-manifest".to_string(),
        schema_version: 1,
        manifest_id: "m1:018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90:1".to_string(),
        manifest_sha256: String::new(),
        host_context_commitment: "1".repeat(64),
        selected_host_prefix: selected_host_prefix.clone(),
        intended_principal: "alice:1000".to_string(),
        platform_kind: "unix".to_string(),
        platform_mapping_commitment: None,
        authority_domain: "unix_a_local".to_string(),
        installation_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
        attempt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91".to_string(),
        manifest_generation: 1,
        created_at_unix_ns: 1,
        lifecycle_state: ManagedLifecycleStateV1::ManifestDurable,
        previous_manifest_sha256: None,
        entries: vec![substrate_common::ManagedArtifactEntryV1 {
            object_id: "entry-1".to_string(),
            logical_role: ManagedArtifactRoleV1("unix.prefix.projection(config.yaml)".to_string()),
            object_type: "regular-file".to_string(),
            identity: ManagedArtifactIdentityV1 {
                scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
                parent_identity: selected_host_prefix.clone(),
                name_identity: "config.yaml".to_string(),
                physical_identity: format!("{selected_host_prefix}/config.yaml"),
                metadata: None,
            },
            disposition: substrate_common::ManagedArtifactDispositionV1::Created,
            bytes_or_target: Some("f".repeat(64)),
            owner: Some("alice".to_string()),
            group_name: None,
            mode: Some("0600".to_string()),
            acl_or_security: None,
            service_or_platform_state: None,
            before_state: Value::Null,
            intended_after_state: Value::Null,
            restoration_state: Value::Null,
            dependency_object_ids: Vec::new(),
            subtree_members: Vec::new(),
            lifecycle_state: ManagedLifecycleStateV1::ManifestDurable,
            last_durable_transition: Some("ManifestDurable".to_string()),
            error_class: None,
        }],
        planned_action_receipts: vec![json!({
            "receipt_id": "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92",
            "entry_id": "entry-1",
            "action": "create",
            "attempt_id": "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a93",
            "receipt_relative_path": "receipts/1/receipt.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92.json"
        })],
    };
    manifest.manifest_sha256 = manifest_digest_v1(&manifest);
    manifest
}

fn manifest_digest_v1(manifest: &ManagedArtifactManifestV1) -> String {
    let mut value = serde_json::to_value(manifest).expect("serialize manifest");
    value
        .as_object_mut()
        .expect("manifest object")
        .remove("manifest_sha256");
    lower_hex(&Sha256::digest(canonical_json_value_to_vec(&value)))
}

fn canonical_json_value_to_vec(value: &Value) -> Vec<u8> {
    let mut output = Vec::new();
    encode_json_value(value, &mut output);
    output
}

fn encode_json_value(value: &Value, output: &mut Vec<u8>) {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => output.extend_from_slice(number.to_string().as_bytes()),
        Value::String(string) => output.extend_from_slice(
            serde_json::to_string(string)
                .expect("encode string")
                .as_bytes(),
        ),
        Value::Array(array) => {
            output.push(b'[');
            for (index, item) in array.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                encode_json_value(item, output);
            }
            output.push(b']');
        }
        Value::Object(object) => {
            output.push(b'{');
            let mut first = true;
            let sorted: BTreeMap<_, _> = object.iter().collect();
            for (key, item) in sorted {
                if !first {
                    output.push(b',');
                }
                first = false;
                output
                    .extend_from_slice(serde_json::to_string(key).expect("encode key").as_bytes());
                output.push(b':');
                encode_json_value(item, output);
            }
            output.push(b'}');
        }
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn sample_pairing_ticket() -> GuestPublisherPairingTicketV1 {
    GuestPublisherPairingTicketV1 {
        schema_owner: "substrate.guest-publisher-pairing-ticket".to_string(),
        schema_version: 1,
        challenge: GuestPublisherPairingChallengeV1 {
            schema_owner: "substrate.guest-publisher-pairing-challenge".to_string(),
            schema_version: 1,
            challenge_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99".to_string(),
            challenge: "challenge-value".to_string(),
            expires_at_unix_ns: 42,
            host_key_fingerprint_sha256: "7".repeat(64),
            current_anchor_sha256: "8".repeat(64),
            host_context_commitment: "9".repeat(64),
            platform_mapping_commitment: Some("a".repeat(64)),
            guest_machine_identity: "machine-1".to_string(),
            source_commit: "b".repeat(40),
            source_tree: "c".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            executor_build_evidence_sha256: "d".repeat(64),
            guest_component_commitment_sha256: "e".repeat(64),
        },
        signer_spki_der: "spki".to_string(),
        current_anchor: LifecyclePublisherAnchorV1 {
            schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
            schema_version: 1,
            authority_domain: "mac_host_shared".to_string(),
            host_context_commitment: "1".repeat(64),
            platform_mapping_commitment: Some("2".repeat(64)),
            scope_id: "scope-1".to_string(),
            manifest_generation: 1,
            manifest_sha256: "3".repeat(64),
            action_receipt_index_revision: 0,
            action_receipt_index_sha256: "4".repeat(64),
            head_sha256: "5".repeat(64),
            previous_anchor_sha256: None,
            request_sha256: "6".repeat(64),
            requester_principal: "alice".to_string(),
            attempt_nonce: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a95".to_string(),
            executor_identity: ManagedExecutorIdentityV1 {
                source_commit: "a".repeat(40),
                source_tree: "b".repeat(40),
                source_ref: "refs/heads/test".to_string(),
                target_triple: "x86_64-unknown-linux-gnu".to_string(),
                artifact_sha256: "c".repeat(64),
                artifact_path: "/tmp/substrate-lifecycle-linux".to_string(),
                toolchain: Some("rustc 1.89.0".to_string()),
                code_identity: None,
            },
            signature: LifecycleSignatureV1 {
                algorithm: "ed25519-v1".to_string(),
                public_key: "public-key".to_string(),
                signature: "signature".to_string(),
            },
        },
        current_anchor_sha256: "f".repeat(64),
        challenge_sha256: "0".repeat(64),
        host_generation: 1,
        host_counter: 1,
        guest_test_retirement_commitment: None,
        signature: LifecycleSignatureV1 {
            algorithm: "ecdsa-p256-sha256-p1363-low-s-v1".to_string(),
            public_key: "public-key".to_string(),
            signature: "signature".to_string(),
        },
    }
}

fn sample_prepared_record(manifest: &ManagedArtifactManifestV1) -> ManagedActionPreparedRecordV1 {
    let mut record = ManagedActionPreparedRecordV1 {
        schema_owner: "substrate.managed-action-prepared-record".to_string(),
        schema_version: 1,
        authority_domain: manifest.authority_domain.clone(),
        scope_id: manifest.installation_id.clone(),
        installation_id: manifest.installation_id.clone(),
        manifest_generation: manifest.manifest_generation,
        manifest_sha256: manifest.manifest_sha256.clone(),
        receipt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92".to_string(),
        receipt_relative_path: "receipts/1/receipt.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92.json"
            .to_string(),
        entry_id: "entry-1".to_string(),
        action: ManagedActionV1::Create,
        attempt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a93".to_string(),
        request_sha256: "e".repeat(64),
        before_observation: Value::Object(serde_json::Map::new()),
        executor_identity: ManagedExecutorIdentityV1 {
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            artifact_sha256: "c".repeat(64),
            artifact_path: "/tmp/substrate-lifecycle-linux".to_string(),
            toolchain: Some("rustc 1.89.0".to_string()),
            code_identity: None,
        },
        allocated_counter: 1,
        previous_record_sha256: None,
        state: "Prepared".to_string(),
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    sign_managed_action_prepared_record_for_test_v1(&mut record).unwrap();
    record
}

fn sample_protected_state(
    manifest: &ManagedArtifactManifestV1,
    prepared_record: &ManagedActionPreparedRecordV1,
) -> LifecyclePublisherProtectedStateV1 {
    let mut anchor = LifecyclePublisherAnchorV1 {
        schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
        schema_version: 1,
        authority_domain: manifest.authority_domain.clone(),
        platform_mapping_commitment: None,
        host_context_commitment: manifest.host_context_commitment.clone(),
        scope_id: manifest.installation_id.clone(),
        manifest_generation: manifest.manifest_generation,
        manifest_sha256: manifest.manifest_sha256.clone(),
        action_receipt_index_revision: 0,
        action_receipt_index_sha256: "4".repeat(64),
        head_sha256: "5".repeat(64),
        previous_anchor_sha256: None,
        request_sha256: prepared_record.request_sha256.clone(),
        requester_principal: manifest.intended_principal.clone(),
        attempt_nonce: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a95".to_string(),
        executor_identity: ManagedExecutorIdentityV1 {
            source_commit: prepared_record.executor_identity.source_commit.clone(),
            source_tree: prepared_record.executor_identity.source_tree.clone(),
            source_ref: prepared_record.executor_identity.source_ref.clone(),
            target_triple: prepared_record.executor_identity.target_triple.clone(),
            artifact_sha256: prepared_record.executor_identity.artifact_sha256.clone(),
            artifact_path: prepared_record.executor_identity.artifact_path.clone(),
            toolchain: prepared_record.executor_identity.toolchain.clone(),
            code_identity: prepared_record.executor_identity.code_identity.clone(),
        },
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    sign_lifecycle_anchor_for_test_v1(&mut anchor).unwrap();
    LifecyclePublisherProtectedStateV1 {
        schema_owner: "substrate.lifecycle-publisher-protected-state".to_string(),
        schema_version: 1,
        current_anchor: anchor,
        counter: prepared_record.allocated_counter,
        prepared_record: Some(prepared_record.clone()),
        previous_protected_state_sha256: None,
        state_revision: 1,
    }
}

fn sample_publisher_request(
    manifest: &ManagedArtifactManifestV1,
    protected_state: &LifecyclePublisherProtectedStateV1,
) -> ManagedLifecyclePublisherRequestV1 {
    ManagedLifecyclePublisherRequestV1 {
        host_context_commitment: manifest.host_context_commitment.clone(),
        platform_mapping_commitment: manifest.platform_mapping_commitment.clone(),
        scope_id: manifest.installation_id.clone(),
        current_anchor_counter: protected_state.counter,
        current_anchor_sha256: lifecycle_anchor_sha256_v1(&protected_state.current_anchor).unwrap(),
        manifest_generation: manifest.manifest_generation,
        manifest_sha256: manifest.manifest_sha256.clone(),
        role: ManagedArtifactRoleV1("unix.prefix.projection(config.yaml)".to_string()),
        action: ManagedActionV1::Create,
        object_identity: ManagedArtifactIdentityV1 {
            scope_id: manifest.installation_id.clone(),
            parent_identity: manifest.selected_host_prefix.clone(),
            name_identity: "config.yaml".to_string(),
            physical_identity: format!("{}/config.yaml", manifest.selected_host_prefix),
            metadata: None,
        },
        requester_principal: manifest.intended_principal.clone(),
        attempt_nonce: "nonce-1".to_string(),
        expected_executor_build: protected_state
            .prepared_record
            .as_ref()
            .unwrap()
            .executor_identity
            .clone(),
    }
}

fn sample_receipt(
    manifest: &ManagedArtifactManifestV1,
    prepared_record: &ManagedActionPreparedRecordV1,
) -> ManagedActionReceiptV1 {
    let mut receipt = ManagedActionReceiptV1 {
        schema_owner: "substrate.managed-action-receipt".to_string(),
        schema_version: 1,
        authority_domain: manifest.authority_domain.clone(),
        scope_id: manifest.installation_id.clone(),
        installation_id: manifest.installation_id.clone(),
        manifest_generation: manifest.manifest_generation,
        manifest_sha256: manifest.manifest_sha256.clone(),
        receipt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92".to_string(),
        receipt_relative_path: "receipts/1/receipt.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92.json"
            .to_string(),
        entry_id: "entry-1".to_string(),
        action: ManagedActionV1::Create,
        attempt_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a93".to_string(),
        prepared_record_sha256: managed_action_prepared_record_sha256_v1(prepared_record).unwrap(),
        allocated_counter: prepared_record.allocated_counter,
        request_sha256: prepared_record.request_sha256.clone(),
        pre_observation: Value::Object(serde_json::Map::new()),
        effect_observation: Value::Object(serde_json::Map::new()),
        post_observation: Value::Object(serde_json::Map::new()),
        restoration_status: None,
        error_class: None,
        executor_identity: prepared_record.executor_identity.clone(),
        signature: LifecycleSignatureV1 {
            algorithm: "ed25519-v1".to_string(),
            public_key: String::new(),
            signature: String::new(),
        },
    };
    sign_managed_action_receipt_for_test_v1(&mut receipt).unwrap();
    receipt
}

#[test]
fn submit_command_publishes_manifest_and_reports_capsule_root() {
    let temp = tempdir().expect("tempdir");
    let manifest = sample_manifest(temp.path());
    let request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
    });

    let assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("request bytes"))
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let response: Value = serde_json::from_str(output.trim()).expect("response JSON");
    let capsule_root = temp.path().join(".substrate-lifecycle-v1");
    let expected_capsule_root = capsule_root.display().to_string();
    assert_eq!(
        response["capsule_root"].as_str(),
        Some(expected_capsule_root.as_str())
    );
    assert_eq!(response["manifest"]["generation"].as_u64(), Some(1));
    let manifest_path = capsule_root.join("manifest.1.json");
    assert!(
        manifest_path.exists(),
        "manifest must be published to the lifecycle capsule"
    );
    let manifest_json: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read published manifest"))
            .expect("published manifest JSON");
    assert_eq!(
        manifest_json["manifest_sha256"].as_str(),
        response["manifest"]["manifest_sha256"].as_str()
    );
}

#[test]
fn submit_command_resumes_receipt_commit_from_persisted_manifest() {
    let temp = tempdir().expect("tempdir");
    let manifest = sample_manifest(temp.path());
    let prepared_record = sample_prepared_record(&manifest);
    let protected_state = sample_protected_state(&manifest, &prepared_record);
    let receipt = sample_receipt(&manifest, &prepared_record);

    let publish_request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
    });
    Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&publish_request).expect("publish request bytes"))
        .assert()
        .success();

    let resume_request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": receipt.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "publisher_protected_state": protected_state,
        "action_receipt": receipt,
    });

    let assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&resume_request).expect("resume request bytes"))
        .assert()
        .success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).expect("stdout utf8");
    let response: Value = serde_json::from_str(output.trim()).expect("response JSON");
    assert_eq!(
        response["action_receipt_head"]["action_receipt_index_revision"].as_u64(),
        Some(1)
    );

    let retry_assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&resume_request).expect("retry request bytes"))
        .assert()
        .success();
    let retry_output =
        String::from_utf8(retry_assert.get_output().stdout.clone()).expect("retry stdout utf8");
    let retry_response: Value =
        serde_json::from_str(retry_output.trim()).expect("retry response JSON");
    assert_eq!(
        retry_response["action_receipt_head"],
        response["action_receipt_head"]
    );
}

#[test]
fn submit_command_exact_retry_with_manifest_and_receipt_is_idempotent() {
    let temp = tempdir().expect("tempdir");
    let manifest = sample_manifest(temp.path());
    let prepared_record = sample_prepared_record(&manifest);
    let protected_state = sample_protected_state(&manifest, &prepared_record);
    let receipt = sample_receipt(&manifest, &prepared_record);
    let request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
        "publisher_protected_state": protected_state,
        "action_receipt": receipt,
    });

    let first = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("first request bytes"))
        .assert()
        .success();
    let first_output = String::from_utf8(first.get_output().stdout.clone()).expect("stdout utf8");
    let first_response: Value = serde_json::from_str(first_output.trim()).expect("response JSON");

    let second = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("second request bytes"))
        .assert()
        .success();
    let second_output =
        String::from_utf8(second.get_output().stdout.clone()).expect("second stdout utf8");
    let second_response: Value =
        serde_json::from_str(second_output.trim()).expect("second response JSON");

    assert_eq!(
        second_response["action_receipt_head"],
        first_response["action_receipt_head"]
    );
}

#[test]
fn submit_command_rejects_cross_prefix_manifest_envelope() {
    let temp = tempdir().expect("tempdir");
    let mut manifest = sample_manifest(Path::new("/tmp/substrate"));
    manifest.manifest_sha256 = manifest_digest_v1(&manifest);
    let request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
    });

    let assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("request bytes"))
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr utf8");
    assert!(stderr.contains("manifest selected_host_prefix does not match"));
}

#[test]
fn submit_command_rejects_publisher_request_with_mismatched_principal() {
    let temp = tempdir().expect("tempdir");
    let manifest = sample_manifest(temp.path());
    let prepared_record = sample_prepared_record(&manifest);
    let protected_state = sample_protected_state(&manifest, &prepared_record);
    let publish_request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
    });
    Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&publish_request).expect("publish request bytes"))
        .assert()
        .success();

    let mut publisher_request = sample_publisher_request(&manifest, &protected_state);
    publisher_request.requester_principal = "bob:1000".to_string();

    let request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "publisher_protected_state": protected_state,
        "publisher_request": publisher_request,
    });
    let assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("request bytes"))
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr utf8");
    assert!(stderr.contains("publisher_request requester_principal does not match"));
}

#[test]
fn submit_command_rejects_receipt_with_mismatched_scope() {
    let temp = tempdir().expect("tempdir");
    let manifest = sample_manifest(temp.path());
    let prepared_record = sample_prepared_record(&manifest);
    let protected_state = sample_protected_state(&manifest, &prepared_record);
    let publish_request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
    });
    Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&publish_request).expect("publish request bytes"))
        .assert()
        .success();

    let mut receipt = sample_receipt(&manifest, &prepared_record);
    receipt.scope_id = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a98".to_string();
    receipt.installation_id = receipt.scope_id.clone();
    sign_managed_action_receipt_for_test_v1(&mut receipt).unwrap();

    let request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "publisher_protected_state": protected_state,
        "action_receipt": receipt,
    });
    let assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("request bytes"))
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr utf8");
    assert!(stderr.contains("scope_id does not match"));
}

#[test]
fn submit_command_rejects_publisher_request_with_mismatched_anchor_digest() {
    let temp = tempdir().expect("tempdir");
    let manifest = sample_manifest(temp.path());
    let prepared_record = sample_prepared_record(&manifest);
    let protected_state = sample_protected_state(&manifest, &prepared_record);
    let publish_request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
    });
    Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&publish_request).expect("publish request bytes"))
        .assert()
        .success();

    let mut publisher_request = sample_publisher_request(&manifest, &protected_state);
    publisher_request.current_anchor_sha256 = "f".repeat(64);

    let request = json!({
        "authority_domain": "unix_a_local",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "publisher_protected_state": protected_state,
        "publisher_request": publisher_request,
    });
    let assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("request bytes"))
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr utf8");
    assert!(stderr.contains("current_anchor_sha256 does not match"));
}

#[test]
fn submit_command_rejects_linux_system_until_platform_packet() {
    let temp = tempdir().expect("tempdir");
    let mut manifest = sample_manifest(temp.path());
    manifest.authority_domain = "linux_system".to_string();
    manifest.manifest_sha256 = manifest_digest_v1(&manifest);
    let request = json!({
        "authority_domain": "linux_system",
        "scope_id": manifest.installation_id,
        "selected_host_prefix": temp.path().display().to_string(),
        "requester_principal": "alice:1000",
        "manifest": manifest,
    });
    let assert = Command::new(control_binary_path())
        .arg("submit")
        .write_stdin(serde_json::to_vec(&request).expect("request bytes"))
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("stderr utf8");
    assert!(stderr.contains("dedicated platform packet lands"));
}

#[test]
fn bootstrap_confirmation_requires_exact_literal() {
    let mut accepted_input = Cursor::new(b"CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER\n".to_vec());
    let mut accepted_output = Vec::new();
    substrate_lifecycle_control::read_exact_bootstrap_confirmation_v1(
        &mut accepted_input,
        &mut accepted_output,
    )
    .expect("accept exact literal");
    let prompt = String::from_utf8(accepted_output).expect("prompt utf8");
    assert!(prompt.contains("CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER"));

    let mut rejected_input = Cursor::new(b"WRONG\n".to_vec());
    let mut rejected_output = Vec::new();
    let error = substrate_lifecycle_control::read_exact_bootstrap_confirmation_v1(
        &mut rejected_input,
        &mut rejected_output,
    )
    .expect_err("reject mismatched literal");
    assert!(error.to_string().contains("literal mismatch"));
}

#[test]
fn display_guest_pairing_challenge_writes_fingerprint_challenge_and_literal() {
    let ticket = sample_pairing_ticket();
    let mut output = Vec::new();
    substrate_lifecycle_control::display_guest_pairing_challenge_v1(&mut output, &ticket)
        .expect("display guest pairing challenge");
    let text = String::from_utf8(output).expect("output utf8");
    assert!(text.contains(&ticket.challenge.host_key_fingerprint_sha256));
    assert!(text.contains(&ticket.challenge.challenge_id));
    assert!(text.contains(&ticket.challenge.challenge));
    assert!(text.contains("PAIR EXACT SUBSTRATE GUEST PUBLISHER"));
}

#[test]
fn guest_pairing_direct_interactive_reports_provider_unavailable_without_ticket() {
    let manifest = sample_manifest(Path::new("/tmp/substrate"));
    let prepared_record = sample_prepared_record(&manifest);
    let request = ManagedLifecycleControlRequestV1 {
        authority_domain: "linux_system".to_string(),
        scope_id: "scope-1".to_string(),
        selected_host_prefix: "/tmp/substrate".to_string(),
        requester_principal: "alice".to_string(),
        host_context_commitment: None,
        platform_mapping_commitment: None,
        host_platform_control_root: None,
        manifest: None,
        action_receipt: None,
        publisher_protected_state: None,
        publisher_request: Some(sample_publisher_request(
            &manifest,
            &sample_protected_state(&manifest, &prepared_record),
        )),
        bootstrap_authorization: None,
        pairing_ticket: None,
    };
    let mut output = Vec::new();
    let error = substrate_lifecycle_control::guest_publisher_pairing_direct_interactive_v1(
        &mut output,
        &request,
    )
    .expect_err("provider unavailable");
    assert!(error.to_string().contains("provider_unavailable"));
    assert!(
        output.is_empty(),
        "no challenge should be written when ticket issuance fails"
    );
}
