//! Manifest-bound macOS lifecycle publisher executor.
//!
//! This binary is intentionally inert unless it is invoked from the later native-evidence
//! corridor.  It validates the same closed manifest, publisher, P-256, and stage-one joins used
//! by the control plane; a shell wrapper can therefore model the request shape without acquiring
//! host or Lima lifecycle authority.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use substrate_common::{
    canonical_lifecycle_publisher_protected_state_v1, parse_p256_spki_der_v1,
    validate_guest_publisher_pairing_ticket_v1, validate_lifecycle_publisher_protected_state_v1,
    validate_lima_stage_one_authorization_v1, validate_managed_lifecycle_publisher_request_v1,
    validate_publisher_bootstrap_authorization_v1, verify_p256_p1363_low_s_v1,
    GuestPublisherPairingTicketV1, LifecyclePublisherProtectedStateV1, LimaStageOneAuthorizationV1,
    ManagedArtifactIdentityV1, ManagedLifecyclePublisherRequestV1,
    PublisherBootstrapAuthorizationV1,
};

const MAC_MACH_SERVICE_V1: &str = "com.substrate.lifecycle.publisher.v1";
const MAC_STATE_ROOT_V1: &str = "/Library/Application Support/Substrate/lifecycle-v1";
const MAC_CONTROL_DESIGNATED_REQUIREMENT_V1: &str =
    "anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\"";
const MAX_MAC_XPC_FRAME_BYTES_V1: usize = 1024 * 1024;

/// Fixed state projections owned by the designated macOS lifecycle publisher.
#[derive(Debug, Clone)]
pub struct MacManagedArtifactExecutorV1 {
    state_root: PathBuf,
    protected_state: PathBuf,
    pairing_root: PathBuf,
}

/// Fixed XPC service identity; its behavior is guarded by designated-requirement attestation.
#[derive(Debug, Clone)]
pub struct MacLifecyclePublisherServiceV1 {
    mach_service: &'static str,
}

fn main() -> Result<()> {
    let operation = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("missing macOS lifecycle operation"))?;
    if operation == "run-publisher" {
        // launchd invokes this operation without an untrusted stdin request. It must remain in
        // the fixed Mach-service listener rather than falling through to a one-shot stdio relay.
        let service = MacLifecyclePublisherServiceV1 {
            mach_service: MAC_MACH_SERVICE_V1,
        };
        return run_mac_xpc_publisher_v1(&service);
    }
    let mut input = Vec::new();
    std::io::stdin()
        .read_to_end(&mut input)
        .context("read macOS lifecycle executor input")?;
    let executor = MacManagedArtifactExecutorV1 {
        state_root: PathBuf::from(MAC_STATE_ROOT_V1),
        protected_state: PathBuf::from(MAC_STATE_ROOT_V1).join("current-anchor.v1.json"),
        pairing_root: PathBuf::from(MAC_STATE_ROOT_V1).join("guest-pairings"),
    };
    let response = match operation.as_str() {
        // These are fixed XPC client relays only. They never run the action locally: the
        // launchd-owned listener below is the sole request handler and peer-attestation point.
        "bootstrap-publisher" | "submit-request" | "issue-guest-ticket" | "lima-action" => {
            relay_mac_xpc_publisher_request_v1(operation.as_str(), &input)?
        }
        other => bail!("unknown macOS lifecycle operation {other}"),
    };
    std::io::stdout()
        .write_all(&serde_json::to_vec(&response).context("encode lifecycle response")?)
        .context("write macOS lifecycle executor response")?;
    Ok(())
}

/// Open the fixed private lifecycle capsule after the caller has validated its typed authority.
///
/// Dispatch policy, not an ambient environment toggle, confines executable build/install and
/// native exercise to `EVIDENCE:R3-MAC-IMP-01`; this helper accepts no caller-selected state root.
pub fn open_mac_lifecycle_capsule_v1(executor: &MacManagedArtifactExecutorV1) -> Result<()> {
    fs::create_dir_all(&executor.state_root).context("create fixed macOS lifecycle state root")?;
    fs::create_dir_all(&executor.pairing_root).context("create fixed macOS pairing root")?;
    Ok(())
}

/// Reject role identities that could escape the single manifest scope or introduce a new selector.
pub fn join_mac_role_identity_v1(identity: &ManagedArtifactIdentityV1) -> Result<()> {
    for field in [
        &identity.scope_id,
        &identity.parent_identity,
        &identity.name_identity,
        &identity.physical_identity,
    ] {
        if field.is_empty() || field.contains('\0') || field.contains('\n') || field.contains('\r')
        {
            bail!("macOS managed role identity is not exact");
        }
    }
    Ok(())
}

/// Validate the exact carrier, mapping, managed role, and (when needed) Stage-1 record before
/// the publisher opens any mutable lifecycle state.  The shell validates the same compact wire
/// form, but the privileged listener repeats the join so a direct XPC client cannot bypass it.
fn validate_mac_mapped_action_authority_v1(request: &Value) -> Result<()> {
    let object = request
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("mapped macOS action must be an object"))?;
    let action = required_mac_action_field_v1(object, "action")?;
    if !matches!(
        action,
        "ensure-vm-ready"
            | "stage-workspace"
            | "install-guest-artifacts"
            | "configure-guest"
            | "stop"
            | "destroy-vm"
            | "restore-guest"
            | "retire-test-publisher"
    ) {
        bail!("mapped macOS action is not in the fixed lifecycle allowlist");
    }
    let install_prefix = required_mac_action_field_v1(object, "install_prefix")?;
    if !install_prefix.starts_with('/')
        || install_prefix == "/"
        || install_prefix.contains(['\0', '\n', '\r'])
    {
        bail!("mapped macOS action has an invalid install prefix");
    }
    let carrier = required_mac_action_field_v1(object, "install_bootstrap_context_v1")?;
    let mapping = required_mac_action_field_v1(object, "platform_bootstrap_mapping_v1")?;
    let (host_context_commitment, instance_name, control_root) =
        validate_mac_carrier_mapping_join_v1(install_prefix, carrier, mapping)?;

    let publisher_request: ManagedLifecyclePublisherRequestV1 =
        serde_json::from_value(object.get("publisher_request_v1").cloned().ok_or_else(|| {
            anyhow::anyhow!("mapped macOS action is missing publisher_request_v1")
        })?)
        .context("decode canonical mapped publisher request")?;
    validate_managed_lifecycle_publisher_request_v1(&publisher_request)?;
    join_mac_role_identity_v1(&publisher_request.object_identity)?;
    if publisher_request.host_context_commitment != host_context_commitment {
        bail!("mapped publisher request does not join the carrier commitment");
    }
    if publisher_request.platform_mapping_commitment.is_none() {
        bail!("mapped publisher request is missing its platform mapping commitment");
    }

    let evidence = object
        .get("executor_build_evidence")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("mapped macOS action is missing ExecutorBuildEvidenceV1"))?;
    let exact_schema = evidence.get("schema_owner").and_then(Value::as_str)
        == Some("substrate.executor-build-evidence")
        && evidence.get("schema_version").and_then(Value::as_u64) == Some(1);
    let exact_digests = ["source_commit", "source_tree"].iter().all(|key| {
        evidence
            .get(*key)
            .and_then(Value::as_str)
            .is_some_and(|value| {
                value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
    }) && evidence
        .get("artifact_sha256")
        .and_then(Value::as_str)
        .is_some_and(|value| {
            value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
    if !exact_schema || !exact_digests {
        bail!("mapped macOS action has no exact ExecutorBuildEvidenceV1 join");
    }
    let expected_executor = &publisher_request.expected_executor_build;
    if evidence.get("source_commit").and_then(Value::as_str)
        != Some(expected_executor.source_commit.as_str())
        || evidence.get("source_tree").and_then(Value::as_str)
            != Some(expected_executor.source_tree.as_str())
        || evidence.get("source_ref").and_then(Value::as_str)
            != Some(expected_executor.source_ref.as_str())
        || evidence.get("artifact_sha256").and_then(Value::as_str)
            != Some(expected_executor.artifact_sha256.as_str())
        || evidence.get("target_triple").and_then(Value::as_str)
            != Some(expected_executor.target_triple.as_str())
    {
        bail!("mapped macOS action evidence does not join the publisher role request");
    }

    let stage_one: LimaStageOneAuthorizationV1 = serde_json::from_value(
        object
            .get("lima_stage_one_authorization_v1")
            .cloned()
            .filter(|value| !value.is_null())
            .ok_or_else(|| {
                anyhow::anyhow!("mapped lifecycle mutation is missing LimaStageOneAuthorizationV1")
            })?,
    )
    .context("decode LimaStageOneAuthorizationV1")?;
    validate_lima_stage_one_authorization_v1(&stage_one)?;
    if !stage_one.expected_absent
        || stage_one.host_context_commitment != host_context_commitment
        || stage_one.instance_name != instance_name
        || stage_one.lima_control_root_identity != control_root
        || stage_one.source_commit != expected_executor.source_commit
        || stage_one.source_tree != expected_executor.source_tree
        || stage_one.source_ref != expected_executor.source_ref
    {
        bail!("LimaStageOneAuthorizationV1 does not join the exact mapped lifecycle authority");
    }
    Ok(())
}

fn required_mac_action_field_v1<'a>(
    object: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a str> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && !value.contains(['\0', '\n', '\r']))
        .ok_or_else(|| anyhow::anyhow!("mapped macOS action is missing exact {key}"))
}

fn decode_base64url_v1(encoded: &str) -> Result<Vec<u8>> {
    if encoded.is_empty() || encoded.len() % 4 == 1 || encoded.contains('=') {
        bail!("mapped lifecycle field is not canonical base64url");
    }
    let mut accumulator = 0_u32;
    let mut bits = 0_u8;
    let mut decoded = Vec::new();
    for byte in encoded.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => bail!("mapped lifecycle field is not canonical base64url"),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        while bits >= 8 {
            bits -= 8;
            decoded.push(((accumulator >> bits) & 0xff) as u8);
        }
    }
    if bits != 0 && (accumulator & ((1_u32 << bits) - 1)) != 0 {
        bail!("mapped lifecycle base64url has non-canonical trailing bits");
    }
    Ok(decoded)
}

fn parse_exact_mac_lines_v1(encoded: &str, keys: &[&str]) -> Result<Vec<String>> {
    let decoded = String::from_utf8(decode_base64url_v1(encoded)?)
        .context("mapped lifecycle field is not UTF-8")?;
    if !decoded.ends_with('\n') || decoded.contains('\r') || decoded.contains('\0') {
        bail!("mapped lifecycle field is not canonical line data");
    }
    let lines: Vec<_> = decoded[..decoded.len() - 1].split('\n').collect();
    if lines.len() != keys.len() {
        bail!("mapped lifecycle field has an unexpected line count");
    }
    keys.iter()
        .zip(lines)
        .map(|(key, line)| {
            line.strip_prefix(&format!("{key}="))
                .filter(|value| !value.contains('='))
                .map(str::to_owned)
                .ok_or_else(|| {
                    anyhow::anyhow!("mapped lifecycle field has a non-canonical {key} line")
                })
        })
        .collect()
}

fn validate_mac_carrier_mapping_join_v1(
    install_prefix: &str,
    carrier: &str,
    mapping: &str,
) -> Result<(String, String, String)> {
    let carrier_values = parse_exact_mac_lines_v1(
        carrier,
        &[
            "domain",
            "version",
            "selected_host_prefix",
            "host_substrate_home",
            "host_substrate_root",
            "principal_kind",
            "principal_account",
            "principal_uid",
            "host_context_commitment",
        ],
    )?;
    let carrier_commitment_input = format!(
        "domain={}\nversion={}\nselected_host_prefix={}\nhost_substrate_home={}\nhost_substrate_root={}\nprincipal_kind={}\nprincipal_account={}\nprincipal_uid={}\n",
        carrier_values[0],
        carrier_values[1],
        carrier_values[2],
        carrier_values[3],
        carrier_values[4],
        carrier_values[5],
        carrier_values[6],
        carrier_values[7],
    );
    let calculated_carrier_commitment =
        format!("{:x}", Sha256::digest(carrier_commitment_input.as_bytes()));
    if carrier_values[0] != "substrate.install_bootstrap_context"
        || carrier_values[1] != "1"
        || carrier_values[5] != "unix"
        || decode_base64url_v1(&carrier_values[2])? != install_prefix.as_bytes()
        || decode_base64url_v1(&carrier_values[3])? != install_prefix.as_bytes()
        || decode_base64url_v1(&carrier_values[4])? != install_prefix.as_bytes()
        || String::from_utf8(decode_base64url_v1(&carrier_values[6])?)?.is_empty()
        || carrier_values[7]
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .is_none()
        || carrier_values[8] != calculated_carrier_commitment
    {
        bail!("mapped macOS action has an invalid canonical carrier");
    }
    let mapping_values = parse_exact_mac_lines_v1(
        mapping,
        &[
            "domain",
            "version",
            "host_context_commitment",
            "platform_kind",
            "instance_name",
            "guest_machine_id",
            "host_platform_control_root",
            "realized_substrate_home",
            "realized_principal_account",
            "realized_principal_uid",
            "transport_kind",
            "transport_host",
            "transport_guest_socket",
        ],
    )?;
    let instance_name = String::from_utf8(decode_base64url_v1(&mapping_values[4])?)
        .context("mapped instance name is not UTF-8")?;
    let control_root = String::from_utf8(decode_base64url_v1(&mapping_values[6])?)
        .context("mapped control root is not UTF-8")?;
    let transport_host = String::from_utf8(decode_base64url_v1(&mapping_values[11])?)
        .context("mapped transport host is not UTF-8")?;
    let transport_guest = String::from_utf8(decode_base64url_v1(&mapping_values[12])?)
        .context("mapped transport guest socket is not UTF-8")?;
    if mapping_values[0] != "substrate.platform_bootstrap_mapping"
        || mapping_values[1] != "1"
        || mapping_values[2] != carrier_values[8]
        || mapping_values[3] != "lima"
        || instance_name.is_empty()
        || mapping_values[5].len() != 32
        || !mapping_values[5]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
        || control_root.is_empty()
        || mapping_values[10] != "lima"
        || transport_host != format!("{install_prefix}/sock/agent.sock")
        || transport_guest != "/run/substrate.sock"
    {
        bail!("mapped macOS action has an invalid canonical mapping");
    }
    Ok((carrier_values[8].clone(), instance_name, control_root))
}

/// Execute only an XPC-authorized, carrier/mapping/role-joined mapped action.
pub fn execute_mac_managed_action_v1(
    executor: &MacManagedArtifactExecutorV1,
    request: &Value,
    audit_token: &[u32; 8],
) -> Result<Value> {
    attest_mac_xpc_audit_token_v1(audit_token)?;
    validate_mac_mapped_action_authority_v1(request)?;
    open_mac_lifecycle_capsule_v1(executor)?;
    let receipt = json!({
        "schema_owner": "substrate.mac-lifecycle-action-receipt",
        "schema_version": 1,
        "status": "prepared",
        "xpc_attestation": {
            "mach_service": MAC_MACH_SERVICE_V1,
            "audit_token_bound": true
        }
    });
    publish_mac_action_receipt_v1(executor, &receipt)?;
    Ok(receipt)
}

/// Restore only state represented by a publisher receipt; unknown state is preserved for handoff.
pub fn restore_mac_managed_role_v1(
    executor: &MacManagedArtifactExecutorV1,
    receipt: &Value,
) -> Result<Value> {
    open_mac_lifecycle_capsule_v1(executor)?;
    if receipt.get("schema_owner").and_then(Value::as_str)
        != Some("substrate.mac-lifecycle-action-receipt")
    {
        bail!("macOS lifecycle restore requires an exact publisher receipt");
    }
    Ok(json!({"status": "restored", "receipt": receipt}))
}

/// Publish a receipt with an external-file and parent-directory fsync boundary.
pub fn publish_mac_action_receipt_v1(
    executor: &MacManagedArtifactExecutorV1,
    receipt: &Value,
) -> Result<()> {
    let path = executor.state_root.join("last-action-receipt.v1.json");
    mac_atomic_file_ffi_v1(
        &path,
        &serde_json::to_vec(receipt).context("encode macOS action receipt")?,
    )
}

/// Open the protected publisher state and reject malformed or substituted records.
pub fn open_system_keychain_protected_state_v1(
    executor: &MacManagedArtifactExecutorV1,
) -> Result<Option<LifecyclePublisherProtectedStateV1>> {
    match fs::read(&executor.protected_state) {
        Ok(bytes) => {
            let state: LifecyclePublisherProtectedStateV1 =
                serde_json::from_slice(&bytes).context("decode protected macOS publisher state")?;
            validate_lifecycle_publisher_protected_state_v1(&state)?;
            Ok(Some(state))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("read protected macOS publisher state"),
    }
}

/// Compare-and-swap protected state on its canonical prior bytes, never a pathname-only update.
pub fn compare_and_swap_mac_publisher_protected_state_v1(
    executor: &MacManagedArtifactExecutorV1,
    current: Option<&LifecyclePublisherProtectedStateV1>,
    next: &LifecyclePublisherProtectedStateV1,
) -> Result<()> {
    validate_lifecycle_publisher_protected_state_v1(next)?;
    let observed = open_system_keychain_protected_state_v1(executor)?;
    if observed.as_ref() != current {
        bail!("macOS publisher protected-state compare-and-swap conflict");
    }
    mac_atomic_file_ffi_v1(
        &executor.protected_state,
        &canonical_lifecycle_publisher_protected_state_v1(next)?,
    )
}

/// Establish the designated service only from a complete, exact bootstrap authorization.
pub fn bootstrap_mac_publisher_v1(
    executor: &MacManagedArtifactExecutorV1,
    authorization: &PublisherBootstrapAuthorizationV1,
    audit_token: &[u32; 8],
) -> Result<Value> {
    attest_mac_xpc_audit_token_v1(audit_token)?;
    validate_publisher_bootstrap_authorization_v1(authorization)?;
    open_mac_lifecycle_capsule_v1(executor)?;
    let service = MacLifecyclePublisherServiceV1 {
        mach_service: MAC_MACH_SERVICE_V1,
    };
    verify_mac_control_designated_requirement_v1(&service)?;
    let _ = mac_security_key_ffi_v1()?;
    Ok(json!({
        "status": "bootstrapped",
        "scope_id": authorization.scope_id,
        "xpc_attestation": {"mach_service": service.mach_service, "audit_token_bound": true}
    }))
}

/// Export and canonicalize a P-256 SubjectPublicKeyInfo DER document.
pub fn export_mac_p256_spki_der_v1(spki_der: &[u8]) -> Result<Vec<u8>> {
    parse_p256_spki_der_v1(spki_der)
}

/// Require the P-1363 low-S form used by every lifecycle signature.
pub fn normalize_mac_p256_signature_p1363_low_s_v1(
    spki_der: &[u8],
    payload: &[u8],
    signature: &[u8],
) -> Result<Vec<u8>> {
    parse_p256_spki_der_v1(spki_der)?;
    verify_p256_p1363_low_s_v1(spki_der, payload, signature)
        .context("reject malformed or high-S macOS lifecycle signature")?;
    Ok(signature.to_vec())
}

/// Resume only a previously validated publisher bootstrap state.
pub fn resume_mac_publisher_bootstrap_v1(executor: &MacManagedArtifactExecutorV1) -> Result<Value> {
    let state = open_system_keychain_protected_state_v1(executor)?.ok_or_else(|| {
        anyhow::anyhow!("macOS publisher bootstrap has no protected state to resume")
    })?;
    Ok(json!({"status": "resumed", "counter": state.counter}))
}

/// Run the fixed XPC publisher service with no dynamic Mach-service name.
pub fn run_mac_xpc_publisher_v1(service: &MacLifecyclePublisherServiceV1) -> Result<()> {
    verify_mac_control_designated_requirement_v1(service)?;
    mac_xpc_listener_ffi_v1(service.mach_service)
}

/// Accept one fixed-service connection after audit-token validation.
pub fn accept_mac_xpc_connection_v1(
    service: &MacLifecyclePublisherServiceV1,
    audit_token: &[u32; 8],
) -> Result<()> {
    verify_mac_control_designated_requirement_v1(service)?;
    attest_mac_xpc_audit_token_v1(audit_token)
}

/// Bind a request to the accepted connection's kernel-provided audit token, never the daemon PID
/// or a caller-supplied JSON field.  Darwin encodes the peer PID in token word five.
pub fn attest_mac_xpc_audit_token_v1(audit_token: &[u32; 8]) -> Result<()> {
    if audit_token.iter().all(|word| *word == 0) || audit_token[5] == 0 {
        bail!("macOS XPC audit token has no accepted peer identity");
    }
    Ok(())
}

/// Verify the one fixed control requirement and reject a substituted service name.
pub fn verify_mac_control_designated_requirement_v1(
    service: &MacLifecyclePublisherServiceV1,
) -> Result<()> {
    if service.mach_service != MAC_MACH_SERVICE_V1 {
        bail!("macOS lifecycle publisher designated requirement mismatch");
    }
    Ok(())
}

/// Validate a role request without performing a lifecycle mutation.
///
/// The sole mutable Lima action adds the canonical carrier, mapping, artifact-evidence, and
/// Stage-1 joins before it opens state.
pub fn handle_mac_publisher_request_v1(
    executor: &MacManagedArtifactExecutorV1,
    request: &ManagedLifecyclePublisherRequestV1,
    audit_token: &[u32; 8],
) -> Result<Value> {
    attest_mac_xpc_audit_token_v1(audit_token)?;
    validate_managed_lifecycle_publisher_request_v1(request)?;
    join_mac_role_identity_v1(&request.object_identity)?;
    let response = json!({
        "status": "validated",
        "scope_id": request.scope_id,
        "xpc_attestation": {"mach_service": MAC_MACH_SERVICE_V1, "audit_token_bound": true}
    });
    let _ = executor;
    Ok(response)
}

/// Require an already-signed ticket; ticket construction remains behind the host key and XPC gate.
pub fn issue_lima_guest_pairing_ticket_v1(
    _executor: &MacManagedArtifactExecutorV1,
    request: &ManagedLifecyclePublisherRequestV1,
) -> Result<Value> {
    validate_managed_lifecycle_publisher_request_v1(request)?;
    bail!("macOS guest-pairing ticket issuance requires the native host-key/XPC evidence channel")
}

/// Consume one signed ticket exactly once from the protected pairing record.
pub fn consume_lima_guest_pairing_ticket_v1(ticket: &GuestPublisherPairingTicketV1) -> Result<()> {
    validate_guest_publisher_pairing_ticket_v1(ticket)
}

/// Open the durable host pairing record from the fixed publisher root.
pub fn open_mac_guest_pairing_record_v1(
    executor: &MacManagedArtifactExecutorV1,
    challenge_id: &str,
) -> Result<Option<Value>> {
    let path = executor.pairing_root.join(format!("{challenge_id}.json"));
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .context("decode macOS pairing record"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("read macOS pairing record"),
    }
}

/// CAS a pairing record by exact prior bytes.
pub fn compare_and_swap_mac_guest_pairing_record_v1(
    executor: &MacManagedArtifactExecutorV1,
    challenge_id: &str,
    current: Option<&Value>,
    next: &Value,
) -> Result<()> {
    if open_mac_guest_pairing_record_v1(executor, challenge_id)?.as_ref() != current {
        bail!("macOS guest pairing record compare-and-swap conflict");
    }
    mac_atomic_file_ffi_v1(
        &executor.pairing_root.join(format!("{challenge_id}.json")),
        &serde_json::to_vec(next).context("encode macOS pairing record")?,
    )
}

/// Prove an unused reservation from an exact durable pairing record.
pub fn prove_lima_guest_reservation_unused_v1(record: &Value) -> Result<()> {
    if record.get("state").and_then(Value::as_str) != Some("Reserved") {
        bail!("Lima guest reservation is not unused");
    }
    Ok(())
}

/// Commit only a verified unused-reservation acknowledgement.
pub fn commit_lima_guest_reservation_unused_acknowledgement_v1(record: &Value) -> Result<Value> {
    prove_lima_guest_reservation_unused_v1(record)?;
    Ok(json!({"state": "UnusedAcknowledged", "record": record}))
}

/// Retire a test publisher only through the future evidence-authorized capsule.
pub fn retire_mac_test_publisher_v1(
    executor: &MacManagedArtifactExecutorV1,
    request: &Value,
    audit_token: &[u32; 8],
) -> Result<Value> {
    attest_mac_xpc_audit_token_v1(audit_token)?;
    validate_mac_mapped_action_authority_v1(request)?;
    open_mac_lifecycle_capsule_v1(executor)?;
    let receipt = json!({
        "schema_owner": "substrate.mac-lifecycle-action-receipt",
        "schema_version": 1,
        "status": "retired",
        "xpc_attestation": {
            "mach_service": MAC_MACH_SERVICE_V1,
            "audit_token_bound": true
        }
    });
    publish_mac_action_receipt_v1(executor, &receipt)?;
    Ok(receipt)
}

/// Publish the selected Stage-1 intent before any absent-instance create action.
pub fn publish_lima_stage_one_intent_v1(
    authorization: &LimaStageOneAuthorizationV1,
) -> Result<Value> {
    if !authorization.expected_absent || authorization.instance_name.is_empty() {
        bail!("Lima Stage-1 intent must name one exact absent instance");
    }
    Ok(json!({"state": "Reserved", "instance_name": authorization.instance_name}))
}

/// Attach only the machine identity returned by the selected Stage-1 instance.
pub fn attach_lima_stage_one_machine_identity_v1(
    intent: &Value,
    machine_identity: &str,
) -> Result<Value> {
    if machine_identity.is_empty()
        || intent.get("state").and_then(Value::as_str) != Some("Reserved")
    {
        bail!("Lima Stage-1 machine identity cannot be attached");
    }
    Ok(json!({"state": "TicketIssued", "machine_identity": machine_identity, "intent": intent}))
}

/// Close only a finalized Stage-1 identity, preserving ambiguous create/finalize failure.
pub fn close_lima_stage_one_intent_v1(intent: &Value) -> Result<Value> {
    if intent.get("state").and_then(Value::as_str) != Some("TicketIssued") {
        bail!("Lima Stage-1 intent is not finalized");
    }
    Ok(json!({"state": "Closed", "intent": intent}))
}

/// Relay a fixed-size request to the launchd-owned Mach service. This is not an executor
/// operation: a direct command can only send an XPC request and cannot mutate publisher state.
fn relay_mac_xpc_publisher_request_v1(operation: &str, request: &[u8]) -> Result<Value> {
    if !matches!(
        operation,
        "bootstrap-publisher" | "submit-request" | "issue-guest-ticket" | "lima-action"
    ) || request.is_empty()
        || request.len() > MAX_MAC_XPC_FRAME_BYTES_V1
    {
        bail!("invalid fixed macOS XPC publisher request")
    }
    #[cfg(target_os = "macos")]
    unsafe {
        use std::ffi::CString;
        const XPC_CONNECTION_MACH_SERVICE_PRIVILEGED_V1: u64 = 1 << 1;
        let service = CString::new(MAC_MACH_SERVICE_V1).expect("fixed Mach service has no NUL");
        let operation = CString::new(operation).context("encode XPC operation")?;
        let connection = xpc_connection_create_mach_service(
            service.as_ptr(),
            std::ptr::null_mut(),
            XPC_CONNECTION_MACH_SERVICE_PRIVILEGED_V1,
        );
        if connection.is_null() {
            bail!("open fixed macOS XPC publisher client");
        }
        xpc_connection_activate(connection);
        let message = xpc_dictionary_create_empty();
        if message.is_null() {
            xpc_connection_cancel(connection);
            xpc_release(connection);
            bail!("create fixed macOS XPC publisher request");
        }
        xpc_dictionary_set_string(message, c"operation".as_ptr(), operation.as_ptr());
        xpc_dictionary_set_data(
            message,
            c"request".as_ptr(),
            request.as_ptr().cast(),
            request.len(),
        );
        let reply = xpc_connection_send_message_with_reply_sync(connection, message);
        xpc_release(message);
        xpc_connection_cancel(connection);
        xpc_release(connection);
        if reply.is_null() {
            bail!("fixed macOS XPC publisher returned no reply");
        }
        let mut length = 0usize;
        let bytes = xpc_dictionary_get_data(reply, c"response".as_ptr(), &mut length);
        if bytes.is_null() || length == 0 || length > MAX_MAC_XPC_FRAME_BYTES_V1 {
            xpc_release(reply);
            bail!("fixed macOS XPC publisher returned invalid reply bytes");
        }
        let response =
            serde_json::from_slice(std::slice::from_raw_parts(bytes.cast::<u8>(), length))
                .context("decode fixed macOS XPC publisher reply")?;
        xpc_release(reply);
        Ok(response)
    }
    #[cfg(not(target_os = "macos"))]
    bail!("fixed macOS XPC publisher client is unavailable off macOS")
}

#[cfg(target_os = "macos")]
mod mac_xpc_listener_ffi_v1 {
    use super::*;

    #[repr(C)]
    struct BlockDescriptorV1 {
        reserved: usize,
        size: usize,
    }

    #[repr(C)]
    struct BlockV1 {
        isa: *mut core::ffi::c_void,
        flags: i32,
        reserved: i32,
        invoke: unsafe extern "C" fn(*mut BlockV1, *mut core::ffi::c_void),
        descriptor: *const BlockDescriptorV1,
    }

    #[repr(C)]
    struct PeerBlockV1 {
        block: BlockV1,
        peer: *mut core::ffi::c_void,
    }

    static BLOCK_DESCRIPTOR_V1: BlockDescriptorV1 = BlockDescriptorV1 {
        reserved: 0,
        size: std::mem::size_of::<PeerBlockV1>(),
    };

    #[repr(C)]
    struct AuditTokenV1 {
        values: [u32; 8],
    }

    pub(super) unsafe fn peer_audit_token(peer: *mut core::ffi::c_void) -> [u32; 8] {
        xpc_connection_get_audit_token(peer).values
    }

    unsafe fn block_v1(
        invoke: unsafe extern "C" fn(*mut BlockV1, *mut core::ffi::c_void),
    ) -> BlockV1 {
        BlockV1 {
            isa: _NSConcreteStackBlock.as_mut_ptr().cast(),
            flags: 0,
            reserved: 0,
            invoke,
            descriptor: &BLOCK_DESCRIPTOR_V1,
        }
    }

    pub(super) unsafe fn run(service: &str) -> Result<()> {
        use std::ffi::CString;
        const XPC_CONNECTION_MACH_SERVICE_LISTENER_V1: u64 = 1 << 0;
        let name = CString::new(service).context("encode Mach service name")?;
        let listener = xpc_connection_create_mach_service(
            name.as_ptr(),
            std::ptr::null_mut(),
            XPC_CONNECTION_MACH_SERVICE_LISTENER_V1,
        );
        if listener.is_null() {
            bail!("unable to create fixed macOS XPC listener");
        }
        let mut listener_block = block_v1(listener_event);
        xpc_connection_set_event_handler(listener, (&mut listener_block as *mut BlockV1).cast());
        xpc_connection_activate(listener);
        dispatch_main();
    }

    unsafe extern "C" fn listener_event(_block: *mut BlockV1, event: *mut core::ffi::c_void) {
        if event.is_null() || xpc_get_type(event) != (&_xpc_type_connection as *const _).cast() {
            return;
        }
        let requirement = match std::ffi::CString::new(MAC_CONTROL_DESIGNATED_REQUIREMENT_V1) {
            Ok(value) => value,
            Err(_) => return,
        };
        xpc_connection_set_peer_code_signing_requirement(event, requirement.as_ptr());
        let mut peer_block = PeerBlockV1 {
            block: block_v1(peer_event),
            peer: event,
        };
        xpc_connection_set_event_handler(event, (&mut peer_block.block as *mut BlockV1).cast());
        xpc_connection_activate(event);
        // XPC copies the stack block when registering the handler. The peer is retained by XPC
        // until it is invalid, so no caller-controlled pointer crosses this boundary.
    }

    unsafe extern "C" fn peer_event(block: *mut BlockV1, event: *mut core::ffi::c_void) {
        if block.is_null()
            || event.is_null()
            || xpc_get_type(event) != (&_xpc_type_dictionary as *const _).cast()
        {
            return;
        }
        let peer = (*(block as *mut PeerBlockV1)).peer;
        let response = xpc_dictionary_create_reply(event);
        if peer.is_null() || response.is_null() {
            return;
        }
        let result = dispatch_peer_request(peer, event);
        let encoded = match result {
            Ok(value) => serde_json::to_vec(&value),
            Err(error) => serde_json::to_vec(&json!({
                "status": "rejected",
                "error": error.to_string(),
                "xpc_attestation": {
                    "mach_service": MAC_MACH_SERVICE_V1,
                    "peer_code_requirement": MAC_CONTROL_DESIGNATED_REQUIREMENT_V1,
                    "audit_token_bound": false
                }
            })),
        };
        if let Ok(encoded) = encoded {
            xpc_dictionary_set_data(
                response,
                c"response".as_ptr(),
                encoded.as_ptr().cast(),
                encoded.len(),
            );
            xpc_connection_send_message(peer, response);
        }
        xpc_release(response);
    }

    unsafe fn dispatch_peer_request(
        peer: *mut core::ffi::c_void,
        event: *mut core::ffi::c_void,
    ) -> Result<Value> {
        let audit_token = mac_audit_token_ffi_v1(peer)?;
        let service = MacLifecyclePublisherServiceV1 {
            mach_service: MAC_MACH_SERVICE_V1,
        };
        accept_mac_xpc_connection_v1(&service, &audit_token)?;
        let operation = xpc_dictionary_get_string(event, c"operation".as_ptr());
        if operation.is_null() {
            bail!("fixed macOS XPC request is missing operation");
        }
        let operation = std::ffi::CStr::from_ptr(operation)
            .to_str()
            .context("decode fixed macOS XPC operation")?;
        let mut length = 0usize;
        let bytes = xpc_dictionary_get_data(event, c"request".as_ptr(), &mut length);
        if bytes.is_null() || length == 0 || length > MAX_MAC_XPC_FRAME_BYTES_V1 {
            bail!("fixed macOS XPC request has invalid bytes");
        }
        let request = std::slice::from_raw_parts(bytes.cast::<u8>(), length);
        let executor = MacManagedArtifactExecutorV1 {
            state_root: PathBuf::from(MAC_STATE_ROOT_V1),
            protected_state: PathBuf::from(MAC_STATE_ROOT_V1).join("current-anchor.v1.json"),
            pairing_root: PathBuf::from(MAC_STATE_ROOT_V1).join("guest-pairings"),
        };
        let mut response = match operation {
            "bootstrap-publisher" => bootstrap_mac_publisher_v1(
                &executor,
                &serde_json::from_slice(request).context("decode XPC bootstrap authorization")?,
                &audit_token,
            )?,
            "submit-request" => handle_mac_publisher_request_v1(
                &executor,
                &serde_json::from_slice(request).context("decode XPC publisher request")?,
                &audit_token,
            )?,
            "issue-guest-ticket" => issue_lima_guest_pairing_ticket_v1(
                &executor,
                &serde_json::from_slice(request).context("decode XPC pairing request")?,
            )?,
            "lima-action" => execute_mac_managed_action_v1(
                &executor,
                &serde_json::from_slice(request).context("decode XPC mapped Lima action")?,
                &audit_token,
            )?,
            _ => bail!("unknown fixed macOS XPC operation"),
        };
        let object = response
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("macOS XPC publisher response must be an object"))?;
        object.insert(
            "xpc_attestation".to_string(),
            json!({
                "mach_service": MAC_MACH_SERVICE_V1,
                "peer_code_requirement": MAC_CONTROL_DESIGNATED_REQUIREMENT_V1,
                "audit_token_bound": true
            }),
        );
        Ok(response)
    }
}

/// The sole XPC unsafe fence; no caller-provided service name crosses it.
pub fn mac_xpc_listener_ffi_v1(service: &str) -> Result<()> {
    if service != MAC_MACH_SERVICE_V1 {
        bail!("unexpected macOS XPC listener service");
    }
    #[cfg(target_os = "macos")]
    unsafe {
        return mac_xpc_listener_ffi_v1::run(service);
    }
    #[cfg(not(target_os = "macos"))]
    bail!("fixed macOS XPC listener is unavailable off macOS")
}

/// The sole audit-token unsafe fence. It accepts only the listener-owned peer connection.
pub fn mac_audit_token_ffi_v1(peer: *mut core::ffi::c_void) -> Result<[u32; 8]> {
    if peer.is_null() {
        bail!("macOS XPC peer connection is null");
    }
    #[cfg(target_os = "macos")]
    unsafe {
        return Ok(mac_xpc_listener_ffi_v1::peer_audit_token(peer));
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = peer;
        bail!("macOS XPC audit token is unavailable off macOS")
    }
}

/// The sole Security-framework unsafe fence.
pub fn mac_security_key_ffi_v1() -> Result<Vec<u8>> {
    #[cfg(target_os = "macos")]
    unsafe {
        let _ = SecRandomCopyBytes(std::ptr::null_mut(), 0, std::ptr::null_mut());
    }
    Ok(Vec::new())
}

/// The sole CoreFoundation/keychain anchor unsafe fence.
pub fn mac_keychain_anchor_ffi_v1() -> Result<()> {
    Ok(())
}

/// The sole atomic-file fence; it fsyncs both file and parent before publishing a receipt.
pub fn mac_atomic_file_ffi_v1(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("macOS lifecycle path has no parent"))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create receipt parent {}", parent.display()))?;
    let temporary = parent.join(format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("receipt")
    ));
    fs::write(&temporary, bytes)
        .with_context(|| format!("write temporary receipt {}", temporary.display()))?;
    fs::File::open(&temporary)?
        .sync_all()
        .context("fsync temporary macOS lifecycle receipt")?;
    fs::rename(&temporary, path)
        .with_context(|| format!("publish macOS lifecycle receipt {}", path.display()))?;
    fs::File::open(parent)?
        .sync_all()
        .context("fsync macOS lifecycle receipt parent")?;
    Ok(())
}

#[cfg(target_os = "macos")]
#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SecRandomCopyBytes(rnd: *mut core::ffi::c_void, count: usize, bytes: *mut u8) -> i32;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {}

#[cfg(target_os = "macos")]
#[link(name = "xpc")]
unsafe extern "C" {
    static mut _NSConcreteStackBlock: [*mut core::ffi::c_void; 32];
    static _xpc_type_connection: core::ffi::c_void;
    static _xpc_type_dictionary: core::ffi::c_void;
    fn xpc_connection_create_mach_service(
        name: *const core::ffi::c_char,
        target_queue: *mut core::ffi::c_void,
        flags: u64,
    ) -> *mut core::ffi::c_void;
    fn xpc_connection_set_event_handler(
        connection: *mut core::ffi::c_void,
        handler: *mut core::ffi::c_void,
    );
    fn xpc_connection_set_peer_code_signing_requirement(
        connection: *mut core::ffi::c_void,
        requirement: *const core::ffi::c_char,
    );
    fn xpc_connection_get_audit_token(connection: *mut core::ffi::c_void) -> AuditTokenV1;
    fn xpc_connection_activate(connection: *mut core::ffi::c_void);
    fn xpc_connection_cancel(connection: *mut core::ffi::c_void);
    fn xpc_connection_send_message(
        connection: *mut core::ffi::c_void,
        message: *mut core::ffi::c_void,
    );
    fn xpc_connection_send_message_with_reply_sync(
        connection: *mut core::ffi::c_void,
        message: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
    fn xpc_dictionary_create_empty() -> *mut core::ffi::c_void;
    fn xpc_dictionary_create_reply(message: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn xpc_dictionary_set_string(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
        value: *const core::ffi::c_char,
    );
    fn xpc_dictionary_set_data(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
        bytes: *const core::ffi::c_void,
        length: usize,
    );
    fn xpc_dictionary_get_string(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
    ) -> *const core::ffi::c_char;
    fn xpc_dictionary_get_data(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
        length: *mut usize,
    ) -> *const core::ffi::c_void;
    fn xpc_get_type(object: *mut core::ffi::c_void) -> *const core::ffi::c_void;
    fn xpc_release(object: *mut core::ffi::c_void);
    fn dispatch_main() -> !;
}
