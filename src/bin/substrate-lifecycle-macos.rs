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
#[cfg(target_os = "macos")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "macos")]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use substrate_common::{
    canonical_lifecycle_publisher_protected_state_v1, canonical_mac_publisher_control_authority_v1,
    parse_p256_spki_der_v1, validate_guest_publisher_pairing_ticket_v1,
    validate_lifecycle_publisher_protected_state_v1, validate_lima_stage_one_authorization_v1,
    validate_mac_publisher_control_authority_v1, validate_managed_lifecycle_publisher_request_v1,
    validate_publisher_bootstrap_authorization_v1, verify_p256_p1363_low_s_v1,
    GuestPublisherPairingTicketV1, LifecyclePublisherProtectedStateV1, LimaStageOneAuthorizationV1,
    MacPublisherControlAuthorityV1, ManagedArtifactIdentityV1, ManagedLifecyclePublisherRequestV1,
    PublisherBootstrapAuthorizationV1,
};

const MAC_MACH_SERVICE_V1: &str = "com.substrate.lifecycle.publisher.v1";
const MAC_STATE_ROOT_V1: &str = "/Library/Application Support/Substrate/lifecycle-v1";
const MAC_CONTROL_DESIGNATED_REQUIREMENT_V1: &str =
    "anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\"";
const MAC_KEYCHAIN_SERVICE_V1: &str = "com.substrate.lifecycle.v1";
const MAC_CONTROL_ADMISSION_ACCOUNT_V1: &str = "mac-control-admission-authority.v1";
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r4_fixed_keychain_accounts_and_xpc_admission_reject_substitution() {
        let scope = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90";
        assert_eq!(
            mac_keychain_protected_state_account_v1(scope).unwrap(),
            "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90:current-anchor"
        );
        assert_eq!(
            mac_keychain_signing_key_tag_v1(scope).unwrap(),
            b"018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90:signing-key"
        );
        assert!(mac_keychain_protected_state_account_v1("not-a-v7-uuid").is_err());
        let service = MacLifecyclePublisherServiceV1 {
            mach_service: MAC_MACH_SERVICE_V1,
        };
        verify_mac_control_designated_requirement_v1(&service).unwrap();
        assert!(attest_mac_xpc_audit_token_v1(&[0; 8]).is_err());
        assert!(attest_mac_xpc_audit_token_v1(&[1, 0, 0, 0, 0, 0, 0, 0]).is_err());
        assert!(attest_mac_xpc_audit_token_v1(&[1, 0, 0, 0, 0, 501, 0, 0]).is_ok());
    }
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

/// The minimum durable record which binds a prior Lima Stage-1 authorization to a host ticket.
///
/// It deliberately contains no channel, peer-frame, or mapped-action state. R4 only consumes a
/// record already written by an independently authorized predecessor; it does not create a guest
/// or invoke Lima.
#[derive(Debug, Clone)]
struct MacLimaGuestPairingStageOneRecordV1 {
    scope_id: String,
    stage_one: LimaStageOneAuthorizationV1,
    guest_machine_identity: String,
    staged_executor_sha256: String,
    record_generation: u64,
}

fn parse_mac_lima_guest_pairing_stage_one_record_v1(
    value: &Value,
) -> Result<MacLimaGuestPairingStageOneRecordV1> {
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("protected Lima Stage-1 record must be an object"))?;
    const KEYS: [&str; 7] = [
        "schema_owner",
        "schema_version",
        "scope_id",
        "stage_one",
        "guest_machine_identity",
        "staged_executor_sha256",
        "record_generation",
    ];
    if object.len() != KEYS.len() || KEYS.iter().any(|key| !object.contains_key(*key)) {
        bail!("protected Lima Stage-1 record has unknown or missing fields");
    }
    if object.get("schema_owner").and_then(Value::as_str)
        != Some("substrate.mac-lima-guest-pairing-stage-one-record")
        || object.get("schema_version").and_then(Value::as_u64) != Some(1)
    {
        bail!("protected Lima Stage-1 record schema mismatch");
    }
    Ok(MacLimaGuestPairingStageOneRecordV1 {
        scope_id: object
            .get("scope_id")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("protected Lima Stage-1 record scope is invalid"))?
            .to_string(),
        stage_one: serde_json::from_value(
            object.get("stage_one").cloned().ok_or_else(|| {
                anyhow::anyhow!("protected Lima Stage-1 record lacks authorization")
            })?,
        )
        .context("decode protected Lima Stage-1 authorization")?,
        guest_machine_identity: object
            .get("guest_machine_identity")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("protected Lima Stage-1 machine identity is invalid"))?
            .to_string(),
        staged_executor_sha256: object
            .get("staged_executor_sha256")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("protected Lima Stage-1 artifact digest is invalid"))?
            .to_string(),
        record_generation: object
            .get("record_generation")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow::anyhow!("protected Lima Stage-1 generation is invalid"))?,
    })
}

fn canonical_mac_lima_guest_pairing_stage_one_record_value_v1(
    record: &MacLimaGuestPairingStageOneRecordV1,
) -> Result<Value> {
    Ok(json!({
        "schema_owner": "substrate.mac-lima-guest-pairing-stage-one-record",
        "schema_version": 1,
        "scope_id": record.scope_id,
        "stage_one": serde_json::to_value(&record.stage_one)
            .context("encode protected Lima Stage-1 authorization")?,
        "guest_machine_identity": record.guest_machine_identity,
        "staged_executor_sha256": record.staged_executor_sha256,
        "record_generation": record.record_generation,
    }))
}

#[cfg(target_os = "macos")]
struct MacKeychainCasGuardV1 {
    file: fs::File,
}

#[cfg(not(target_os = "macos"))]
struct MacKeychainCasGuardV1;

#[cfg(target_os = "macos")]
impl Drop for MacKeychainCasGuardV1 {
    fn drop(&mut self) {
        // SAFETY: the descriptor is owned solely by this guard and flock only releases its lock.
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

fn mac_keychain_durable_cas_guard_v1(account: &str) -> Result<MacKeychainCasGuardV1> {
    #[cfg(target_os = "macos")]
    {
        if account.is_empty() || account.contains('\0') {
            bail!("macOS Keychain CAS account is invalid");
        }
        let root = Path::new(MAC_STATE_ROOT_V1);
        fs::create_dir_all(root).context("create fixed macOS Keychain CAS root")?;
        let lock_path = root.join(format!(
            "keychain-cas-{:x}.lock",
            Sha256::digest(account.as_bytes())
        ));
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(&lock_path)
            .with_context(|| format!("open macOS Keychain CAS lock {}", lock_path.display()))?;
        let metadata = file
            .metadata()
            .with_context(|| format!("inspect macOS Keychain CAS lock {}", lock_path.display()))?;
        if !metadata.file_type().is_file() || metadata.uid() != 0 || metadata.mode() & 0o077 != 0 {
            bail!("macOS Keychain CAS lock must be a root-owned private regular file");
        }
        // SAFETY: flock serializes the fixed account's read/validate/write/readback sequence.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("acquire macOS Keychain inter-process CAS lock");
        }
        return Ok(MacKeychainCasGuardV1 { file });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = account;
        bail!("macOS Keychain CAS is unavailable off macOS")
    }
}

fn mac_require_uuid_v7_component_v1(value: &str, field: &str) -> Result<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || ![8_usize, 13, 18, 23]
            .iter()
            .all(|offset| bytes[*offset] == b'-')
        || bytes[14] != b'7'
        || !matches!(bytes[19], b'8' | b'9' | b'a' | b'b')
        || bytes.iter().enumerate().any(|(offset, byte)| {
            !matches!(offset, 8 | 13 | 18 | 23)
                && (!byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
        })
    {
        bail!("{field} must be an exact UUIDv7");
    }
    Ok(())
}

fn mac_keychain_account_v1(scope_id: &str, suffix: &str) -> Result<String> {
    mac_require_uuid_v7_component_v1(scope_id, "System Keychain scope")?;
    if suffix.is_empty() || suffix.contains(['\0', '\n', '\r']) {
        bail!("System Keychain account suffix is invalid");
    }
    Ok(format!("{scope_id}:{suffix}"))
}

fn mac_keychain_protected_state_account_v1(scope_id: &str) -> Result<String> {
    mac_keychain_account_v1(scope_id, "current-anchor")
}

fn mac_keychain_signing_key_tag_v1(scope_id: &str) -> Result<Vec<u8>> {
    Ok(mac_keychain_account_v1(scope_id, "signing-key")?.into_bytes())
}

fn mac_keychain_stage_one_record_account_v1(scope_id: &str) -> Result<String> {
    mac_keychain_account_v1(scope_id, "lima-guest-pairing-stage-one")
}

fn mac_keychain_read_item_v1(service: &str, account: &str) -> Result<Option<Vec<u8>>> {
    if service != MAC_KEYCHAIN_SERVICE_V1 || account.is_empty() || account.contains('\0') {
        bail!("System Keychain item address is not fixed");
    }
    #[cfg(target_os = "macos")]
    {
        return mac_system_keychain_ffi_v1::read_generic_password(service, account);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (service, account);
        bail!("System Keychain lifecycle state is unavailable off macOS")
    }
}

fn mac_keychain_compare_and_swap_item_v1(
    service: &str,
    account: &str,
    expected: Option<&[u8]>,
    next: &[u8],
) -> Result<()> {
    if service != MAC_KEYCHAIN_SERVICE_V1
        || account.is_empty()
        || account.contains('\0')
        || next.is_empty()
    {
        bail!("System Keychain CAS item address or bytes are invalid");
    }
    #[cfg(target_os = "macos")]
    {
        return mac_system_keychain_ffi_v1::compare_and_swap_generic_password(
            service, account, expected, next,
        );
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (service, account, expected, next);
        bail!("System Keychain lifecycle CAS is unavailable off macOS")
    }
}

fn base64url_encode_mac_v1(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((bytes.len() * 4).div_ceil(3));
    for chunk in bytes.chunks(3) {
        let word = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        output.push(TABLE[((word >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((word >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[((word >> 6) & 0x3f) as usize] as char);
        }
        if chunk.len() > 2 {
            output.push(TABLE[(word & 0x3f) as usize] as char);
        }
    }
    output
}

fn mac_open_system_keychain_p256_spki_der_v1(scope_id: &str) -> Result<Vec<u8>> {
    let key_tag = mac_keychain_signing_key_tag_v1(scope_id)?;
    #[cfg(target_os = "macos")]
    {
        return export_mac_p256_spki_der_v1(&mac_system_keychain_ffi_v1::open_p256_spki_der(
            &key_tag,
        )?);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = key_tag;
        bail!("System Keychain P-256 key is unavailable off macOS")
    }
}

fn mac_verified_control_peer_requirement_v1() -> Result<String> {
    Ok(mac_verified_control_authority_v1()?.designated_requirement)
}

fn mac_verified_control_authority_v1() -> Result<MacPublisherControlAuthorityV1> {
    let bytes =
        mac_keychain_read_item_v1(MAC_KEYCHAIN_SERVICE_V1, MAC_CONTROL_ADMISSION_ACCOUNT_V1)?
            .ok_or_else(|| {
                anyhow::anyhow!("macOS XPC listener has no fixed admission authority")
            })?;
    let authority: MacPublisherControlAuthorityV1 =
        serde_json::from_slice(&bytes).context("decode fixed macOS XPC admission authority")?;
    validate_mac_publisher_control_authority_v1(&authority)?;
    if canonical_mac_publisher_control_authority_v1(&authority)? != bytes {
        bail!("fixed macOS XPC admission authority is not canonical");
    }
    Ok(authority)
}

fn attest_mac_xpc_control_image_v1(
    message: *mut core::ffi::c_void,
    authority: &MacPublisherControlAuthorityV1,
) -> Result<()> {
    if message.is_null() {
        bail!("macOS XPC peer message is null");
    }
    #[cfg(target_os = "macos")]
    {
        mac_system_keychain_ffi_v1::verify_xpc_message_requirement(
            message,
            &authority.designated_requirement,
        )?;
        return Ok(());
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (message, authority);
        bail!("macOS XPC control-image attestation is unavailable off macOS")
    }
}

fn read_system_keychain_protected_state_for_scope_unbound_v1(
    scope_id: &str,
) -> Result<Option<LifecyclePublisherProtectedStateV1>> {
    let account = mac_keychain_protected_state_account_v1(scope_id)?;
    let Some(bytes) = mac_keychain_read_item_v1(MAC_KEYCHAIN_SERVICE_V1, &account)? else {
        return Ok(None);
    };
    let state: LifecyclePublisherProtectedStateV1 =
        serde_json::from_slice(&bytes).context("decode System Keychain protected state")?;
    validate_lifecycle_publisher_protected_state_v1(&state)?;
    if state.current_anchor.scope_id != scope_id
        || canonical_lifecycle_publisher_protected_state_v1(&state)? != bytes
    {
        bail!("System Keychain protected state does not match its exact account");
    }
    Ok(Some(state))
}

fn open_system_keychain_protected_state_for_scope_v1(
    scope_id: &str,
) -> Result<Option<LifecyclePublisherProtectedStateV1>> {
    let Some(state) = read_system_keychain_protected_state_for_scope_unbound_v1(scope_id)? else {
        return Ok(None);
    };
    validate_system_keychain_protected_state_key_binding_v1(scope_id, &state)?;
    Ok(Some(state))
}

fn validate_system_keychain_protected_state_key_binding_v1(
    scope_id: &str,
    state: &LifecyclePublisherProtectedStateV1,
) -> Result<()> {
    let spki_der = mac_open_system_keychain_p256_spki_der_v1(scope_id)?;
    if state.current_anchor.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || state.current_anchor.signature.public_key != base64url_encode_mac_v1(&spki_der)
    {
        bail!("System Keychain protected state is not bound to its non-exportable P-256 key");
    }
    Ok(())
}

fn canonical_mac_lima_guest_pairing_stage_one_record_v1(
    record: &MacLimaGuestPairingStageOneRecordV1,
) -> Result<Vec<u8>> {
    validate_mac_lima_guest_pairing_stage_one_record_v1(record)?;
    serde_json::to_vec(&canonical_mac_lima_guest_pairing_stage_one_record_value_v1(
        record,
    )?)
    .context("encode canonical protected Lima Stage-1 record")
}

fn validate_mac_lima_guest_pairing_stage_one_record_v1(
    record: &MacLimaGuestPairingStageOneRecordV1,
) -> Result<()> {
    mac_require_uuid_v7_component_v1(&record.scope_id, "protected Lima Stage-1 scope")?;
    validate_lima_stage_one_authorization_v1(&record.stage_one)?;
    if !record.stage_one.expected_absent
        || record.guest_machine_identity.is_empty()
        || record.guest_machine_identity.contains(['\0', '\n', '\r'])
        || record.staged_executor_sha256.len() != 64
        || !record
            .staged_executor_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        || record.record_generation == 0
    {
        bail!("protected Lima Stage-1 record is malformed");
    }
    Ok(())
}

fn open_mac_lima_guest_pairing_stage_one_record_v1(
    scope_id: &str,
) -> Result<Option<MacLimaGuestPairingStageOneRecordV1>> {
    let account = mac_keychain_stage_one_record_account_v1(scope_id)?;
    let Some(bytes) = mac_keychain_read_item_v1(MAC_KEYCHAIN_SERVICE_V1, &account)? else {
        return Ok(None);
    };
    let value: Value =
        serde_json::from_slice(&bytes).context("decode protected Lima Stage-1 record")?;
    let record = parse_mac_lima_guest_pairing_stage_one_record_v1(&value)?;
    if record.scope_id != scope_id
        || canonical_mac_lima_guest_pairing_stage_one_record_v1(&record)? != bytes
    {
        bail!("protected Lima Stage-1 record does not match its exact account");
    }
    Ok(Some(record))
}

fn mac_now_unix_ns_v1() -> Result<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("read macOS clock for Stage-1 expiry")?;
    u64::try_from(duration.as_nanos()).context("macOS Stage-1 clock exceeds u64 nanoseconds")
}

fn validate_mac_lima_guest_pairing_stage_one_record_for_issue_v1(
    record: &MacLimaGuestPairingStageOneRecordV1,
    protected_state: &LifecyclePublisherProtectedStateV1,
) -> Result<()> {
    validate_mac_lima_guest_pairing_stage_one_record_v1(record)?;
    validate_lifecycle_publisher_protected_state_v1(protected_state)?;
    if record.stage_one.expires_at_unix_ns <= mac_now_unix_ns_v1()? {
        bail!("protected Lima Stage-1 authorization has expired");
    }
    let anchor = &protected_state.current_anchor;
    if record.stage_one.host_context_commitment != anchor.host_context_commitment
        || record.stage_one.source_commit != anchor.executor_identity.source_commit
        || record.stage_one.source_tree != anchor.executor_identity.source_tree
        || record.stage_one.source_ref != anchor.executor_identity.source_ref
        || record.staged_executor_sha256 != anchor.executor_identity.artifact_sha256
    {
        bail!("protected Lima Stage-1 record does not join the protected host anchor");
    }
    Ok(())
}

/// Open the protected publisher state and reject malformed or substituted records.
pub fn open_system_keychain_protected_state_v1(
    executor: &MacManagedArtifactExecutorV1,
) -> Result<Option<LifecyclePublisherProtectedStateV1>> {
    let _ = executor;
    bail!("macOS protected-state open requires an exact scope-bound System Keychain account")
}

/// Compare-and-swap protected state on its canonical prior bytes, never a pathname-only update.
pub fn compare_and_swap_mac_publisher_protected_state_v1(
    executor: &MacManagedArtifactExecutorV1,
    current: Option<&LifecyclePublisherProtectedStateV1>,
    next: &LifecyclePublisherProtectedStateV1,
) -> Result<()> {
    let _ = executor;
    validate_lifecycle_publisher_protected_state_v1(next)?;
    let scope_id = current
        .map(|state| state.current_anchor.scope_id.as_str())
        .unwrap_or(next.current_anchor.scope_id.as_str());
    if next.current_anchor.scope_id != scope_id {
        bail!("next protected state does not match the System Keychain scope");
    }
    let spki_der = mac_open_system_keychain_p256_spki_der_v1(scope_id)?;
    if next.current_anchor.signature.algorithm != "ecdsa-p256-sha256-p1363-low-s-v1"
        || next.current_anchor.signature.public_key != base64url_encode_mac_v1(&spki_der)
    {
        bail!("next protected state is not bound to its System Keychain P-256 key");
    }
    let account = mac_keychain_protected_state_account_v1(scope_id)?;
    let _guard = mac_keychain_durable_cas_guard_v1(&account)?;
    let observed = open_system_keychain_protected_state_for_scope_v1(scope_id)?;
    if observed.as_ref() != current {
        bail!("macOS publisher protected-state compare-and-swap conflict");
    }
    let expected = current
        .map(canonical_lifecycle_publisher_protected_state_v1)
        .transpose()?;
    let next = canonical_lifecycle_publisher_protected_state_v1(next)?;
    mac_keychain_compare_and_swap_item_v1(
        MAC_KEYCHAIN_SERVICE_V1,
        &account,
        expected.as_deref(),
        &next,
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
    let _ = mac_verified_control_peer_requirement_v1()?;
    mac_xpc_listener_ffi_v1(service.mach_service)
}

/// Accept one fixed-service connection after audit-token validation.
pub fn accept_mac_xpc_connection_v1(
    service: &MacLifecyclePublisherServiceV1,
    audit_token: &[u32; 8],
) -> Result<()> {
    verify_mac_control_designated_requirement_v1(service)?;
    let _ = mac_verified_control_peer_requirement_v1()?;
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
    let stage_one = open_mac_lima_guest_pairing_stage_one_record_v1(&request.scope_id)?
        .ok_or_else(|| anyhow::anyhow!("macOS ticket issue has no protected Stage-1 record"))?;
    let protected_state =
        read_system_keychain_protected_state_for_scope_unbound_v1(&request.scope_id)?
            .ok_or_else(|| anyhow::anyhow!("macOS ticket issue has no protected host state"))?;
    if request.host_context_commitment != protected_state.current_anchor.host_context_commitment
        || request.current_anchor_sha256
            != substrate_common::lifecycle_anchor_sha256_v1(&protected_state.current_anchor)?
        || request.manifest_generation != protected_state.current_anchor.manifest_generation
        || request.manifest_sha256 != protected_state.current_anchor.manifest_sha256
    {
        bail!("macOS ticket issue request does not join protected host state");
    }
    validate_mac_lima_guest_pairing_stage_one_record_for_issue_v1(&stage_one, &protected_state)?;
    validate_system_keychain_protected_state_key_binding_v1(&request.scope_id, &protected_state)?;
    bail!("macOS ticket issue requires the separately authorized R5 typed issue contract")
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

/// Direct System-Keychain/Security framework fence. It admits only fixed service/account/key-tag
/// values prepared by the R4 helpers above; it exposes no private-key export operation.
#[cfg(target_os = "macos")]
mod mac_system_keychain_ffi_v1 {
    use super::*;
    use std::ffi::c_void;
    use std::ptr;

    type CfType = *const c_void;
    type CfMutableDictionary = *mut c_void;
    type CfData = *const c_void;
    type SecCode = *const c_void;
    type SecKey = *const c_void;
    type SecRequirement = *const c_void;
    type OsStatus = i32;

    const ERR_SEC_SUCCESS: OsStatus = 0;
    const ERR_SEC_DUPLICATE_ITEM: OsStatus = -25299;
    const ERR_SEC_ITEM_NOT_FOUND: OsStatus = -25300;

    struct OwnedCf(Vec<CfType>);

    impl OwnedCf {
        fn new() -> Self {
            Self(Vec::new())
        }

        unsafe fn hold<T>(&mut self, value: *const T) -> *const T {
            if !value.is_null() {
                self.0.push(value.cast());
            }
            value
        }
    }

    impl Drop for OwnedCf {
        fn drop(&mut self) {
            unsafe {
                for value in self.0.drain(..).rev() {
                    CFRelease(value);
                }
            }
        }
    }

    unsafe fn cf_string(owned: &mut OwnedCf, value: &str) -> Result<CfType> {
        let result = CFStringCreateWithBytes(
            kCFAllocatorDefault,
            value.as_bytes().as_ptr(),
            value.len() as isize,
            0x0800_0100,
            0,
        );
        if result.is_null() {
            bail!("allocate System Keychain UTF-8 string");
        }
        Ok(owned.hold(result).cast())
    }

    unsafe fn cf_data(owned: &mut OwnedCf, value: &[u8]) -> Result<CfType> {
        let result = CFDataCreate(kCFAllocatorDefault, value.as_ptr(), value.len() as isize);
        if result.is_null() {
            bail!("allocate System Keychain data");
        }
        Ok(owned.hold(result).cast())
    }

    unsafe fn dictionary(
        owned: &mut OwnedCf,
        entries: &[(CfType, CfType)],
    ) -> Result<CfMutableDictionary> {
        let dictionary = CFDictionaryCreateMutable(
            kCFAllocatorDefault,
            entries.len() as isize,
            ptr::null(),
            ptr::null(),
        );
        if dictionary.is_null() {
            bail!("allocate System Keychain query");
        }
        owned.hold(dictionary);
        for (key, value) in entries {
            CFDictionarySetValue(dictionary, *key, *value);
        }
        Ok(dictionary)
    }

    unsafe fn generic_query(
        owned: &mut OwnedCf,
        service: &str,
        account: &str,
        return_data: bool,
    ) -> Result<CfMutableDictionary> {
        let service = cf_string(owned, service)?;
        let account = cf_string(owned, account)?;
        let mut entries = vec![
            (kSecClass, kSecClassGenericPassword),
            (kSecAttrService, service),
            (kSecAttrAccount, account),
            (kSecUseSystemKeychain, kCFBooleanTrue),
        ];
        if return_data {
            entries.push((kSecReturnData, kCFBooleanTrue));
            entries.push((kSecMatchLimit, kSecMatchLimitOne));
        }
        dictionary(owned, &entries)
    }

    pub(super) fn read_generic_password(service: &str, account: &str) -> Result<Option<Vec<u8>>> {
        unsafe {
            let mut owned = OwnedCf::new();
            let query = generic_query(&mut owned, service, account, true)?;
            let mut result: CfType = ptr::null();
            match SecItemCopyMatching(query, &mut result) {
                ERR_SEC_ITEM_NOT_FOUND => Ok(None),
                ERR_SEC_SUCCESS if !result.is_null() => {
                    owned.hold(result);
                    let length = CFDataGetLength(result.cast());
                    let bytes = CFDataGetBytePtr(result.cast());
                    if length < 0 || bytes.is_null() {
                        bail!("System Keychain returned invalid item data");
                    }
                    Ok(Some(
                        std::slice::from_raw_parts(bytes, length as usize).to_vec(),
                    ))
                }
                status => bail!("System Keychain read failed with OSStatus {status}"),
            }
        }
    }

    pub(super) fn compare_and_swap_generic_password(
        service: &str,
        account: &str,
        expected: Option<&[u8]>,
        next: &[u8],
    ) -> Result<()> {
        let observed = read_generic_password(service, account)?;
        if observed.as_deref() != expected {
            bail!("System Keychain generic-password compare-and-swap conflict");
        }
        unsafe {
            let mut owned = OwnedCf::new();
            let query = generic_query(&mut owned, service, account, false)?;
            let data = cf_data(&mut owned, next)?;
            let update = dictionary(&mut owned, &[(kSecValueData, data)])?;
            let status = if expected.is_some() {
                SecItemUpdate(query, update)
            } else {
                let service = cf_string(&mut owned, service)?;
                let account = cf_string(&mut owned, account)?;
                let add = dictionary(
                    &mut owned,
                    &[
                        (kSecClass, kSecClassGenericPassword),
                        (kSecAttrService, service),
                        (kSecAttrAccount, account),
                        (kSecUseSystemKeychain, kCFBooleanTrue),
                        (kSecValueData, data),
                    ],
                )?;
                SecItemAdd(add, ptr::null_mut())
            };
            if status == ERR_SEC_DUPLICATE_ITEM && expected.is_none() {
                bail!("System Keychain generic-password compare-and-swap conflict");
            }
            if status != ERR_SEC_SUCCESS {
                bail!("System Keychain generic-password update failed with OSStatus {status}");
            }
        }
        if read_generic_password(service, account)?.as_deref() != Some(next) {
            bail!("System Keychain generic-password write verification failed");
        }
        Ok(())
    }

    unsafe fn open_private_key(owned: &mut OwnedCf, key_tag: &[u8]) -> Result<SecKey> {
        let tag = cf_data(owned, key_tag)?;
        let query = dictionary(
            owned,
            &[
                (kSecClass, kSecClassKey),
                (kSecAttrApplicationTag, tag),
                (kSecAttrKeyClass, kSecAttrKeyClassPrivate),
                (kSecReturnRef, kCFBooleanTrue),
                (kSecUseSystemKeychain, kCFBooleanTrue),
            ],
        )?;
        let mut result: CfType = ptr::null();
        let status = SecItemCopyMatching(query, &mut result);
        if status == ERR_SEC_ITEM_NOT_FOUND {
            bail!("System Keychain lifecycle P-256 signing key is absent");
        }
        if status != ERR_SEC_SUCCESS || result.is_null() {
            bail!("open System Keychain lifecycle P-256 signing key failed with OSStatus {status}");
        }
        owned.hold(result);
        let attributes = SecKeyCopyAttributes(result.cast());
        if attributes.is_null() {
            bail!("System Keychain lifecycle P-256 key has no inspectable attributes");
        }
        owned.hold(attributes);
        if CFDictionaryGetValue(attributes.cast(), kSecAttrIsExtractable) != kCFBooleanFalse {
            bail!("System Keychain lifecycle P-256 key is not explicitly non-exportable");
        }
        Ok(result.cast())
    }

    unsafe fn spki_for_private_key(owned: &mut OwnedCf, private_key: SecKey) -> Result<Vec<u8>> {
        let public_key = SecKeyCopyPublicKey(private_key);
        if public_key.is_null() {
            bail!("derive System Keychain lifecycle public key");
        }
        owned.hold(public_key);
        let mut error: CfType = ptr::null();
        let point = SecKeyCopyExternalRepresentation(public_key, &mut error);
        if !error.is_null() {
            owned.hold(error);
        }
        if point.is_null() {
            bail!("export System Keychain P-256 public point");
        }
        owned.hold(point);
        let length = CFDataGetLength(point);
        let bytes = CFDataGetBytePtr(point);
        if length != 65 || bytes.is_null() || *bytes != 0x04 {
            bail!("System Keychain public key is not an uncompressed P-256 point");
        }
        let mut spki = vec![
            0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06,
            0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
        ];
        spki.extend_from_slice(std::slice::from_raw_parts(bytes, length as usize));
        parse_p256_spki_der_v1(&spki)?;
        Ok(spki)
    }

    pub(super) fn open_p256_spki_der(key_tag: &[u8]) -> Result<Vec<u8>> {
        unsafe {
            let mut owned = OwnedCf::new();
            let key = open_private_key(&mut owned, key_tag)?;
            spki_for_private_key(&mut owned, key)
        }
    }

    /// Verify the code identity bound by XPC to this exact incoming message.  Security creates
    /// the `SecCode` from the message's kernel-provided audit token, so this never resolves a
    /// mutable executable path by PID.
    pub(super) fn verify_xpc_message_requirement(
        message: *mut c_void,
        designated_requirement: &str,
    ) -> Result<()> {
        if message.is_null() || designated_requirement.is_empty() {
            bail!("macOS XPC control-image requirement has no message or requirement");
        }
        unsafe {
            let mut owned = OwnedCf::new();
            let requirement_text = cf_string(&mut owned, designated_requirement)?;
            let mut requirement: SecRequirement = ptr::null();
            let status = SecRequirementCreateWithString(requirement_text, 0, &mut requirement);
            if status != ERR_SEC_SUCCESS || requirement.is_null() {
                bail!("compile fixed macOS XPC code requirement failed with OSStatus {status}");
            }
            owned.hold(requirement);

            let mut code: SecCode = ptr::null();
            let status = SecCodeCreateWithXPCMessage(message, 0, &mut code);
            if status != ERR_SEC_SUCCESS || code.is_null() {
                bail!("resolve macOS XPC peer code from the audited message failed with OSStatus {status}");
            }
            owned.hold(code);
            let status = SecCodeCheckValidity(code, 0, requirement);
            if status != ERR_SEC_SUCCESS {
                bail!("fixed macOS XPC peer did not satisfy the protected code requirement with OSStatus {status}");
            }
            Ok(())
        }
    }

    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        static kSecClass: CfType;
        static kSecClassGenericPassword: CfType;
        static kSecClassKey: CfType;
        static kSecAttrService: CfType;
        static kSecAttrAccount: CfType;
        static kSecAttrApplicationTag: CfType;
        static kSecAttrKeyClass: CfType;
        static kSecAttrKeyClassPrivate: CfType;
        static kSecAttrIsExtractable: CfType;
        static kSecValueData: CfType;
        static kSecReturnData: CfType;
        static kSecReturnRef: CfType;
        static kSecMatchLimit: CfType;
        static kSecMatchLimitOne: CfType;
        static kSecUseSystemKeychain: CfType;
        fn SecItemCopyMatching(query: CfType, result: *mut CfType) -> OsStatus;
        fn SecItemAdd(attributes: CfType, result: *mut CfType) -> OsStatus;
        fn SecItemUpdate(query: CfType, attributes: CfType) -> OsStatus;
        fn SecRequirementCreateWithString(
            text: CfType,
            flags: u32,
            requirement: *mut SecRequirement,
        ) -> OsStatus;
        fn SecCodeCreateWithXPCMessage(
            message: *mut c_void,
            flags: u32,
            code: *mut SecCode,
        ) -> OsStatus;
        fn SecCodeCheckValidity(code: SecCode, flags: u32, requirement: SecRequirement)
            -> OsStatus;
        fn SecKeyCopyPublicKey(key: SecKey) -> SecKey;
        fn SecKeyCopyAttributes(key: SecKey) -> CfType;
        fn SecKeyCopyExternalRepresentation(key: SecKey, error: *mut CfType) -> CfData;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        static kCFAllocatorDefault: CfType;
        static kCFBooleanTrue: CfType;
        static kCFBooleanFalse: CfType;
        fn CFRelease(value: CfType);
        fn CFStringCreateWithBytes(
            allocator: CfType,
            bytes: *const u8,
            num_bytes: isize,
            encoding: u32,
            is_external_representation: u8,
        ) -> CfType;
        fn CFDataCreate(allocator: CfType, bytes: *const u8, length: isize) -> CfData;
        fn CFDataGetLength(data: CfData) -> isize;
        fn CFDataGetBytePtr(data: CfData) -> *const u8;
        fn CFDictionaryCreateMutable(
            allocator: CfType,
            capacity: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> CfMutableDictionary;
        fn CFDictionarySetValue(dictionary: CfMutableDictionary, key: CfType, value: CfType);
        fn CFDictionaryGetValue(dictionary: CfType, key: CfType) -> CfType;
    }
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct MacAuditTokenFfiV1 {
    values: [u32; 8],
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

    static LISTENER_BLOCK_DESCRIPTOR_V1: BlockDescriptorV1 = BlockDescriptorV1 {
        reserved: 0,
        size: std::mem::size_of::<BlockV1>(),
    };

    static PEER_BLOCK_DESCRIPTOR_V1: BlockDescriptorV1 = BlockDescriptorV1 {
        reserved: 0,
        size: std::mem::size_of::<PeerBlockV1>(),
    };

    pub(super) unsafe fn peer_audit_token(peer: *mut core::ffi::c_void) -> [u32; 8] {
        xpc_connection_get_audit_token(peer).values
    }

    unsafe fn block_v1(
        invoke: unsafe extern "C" fn(*mut BlockV1, *mut core::ffi::c_void),
        descriptor: *const BlockDescriptorV1,
    ) -> BlockV1 {
        BlockV1 {
            isa: std::ptr::addr_of_mut!(_NSConcreteStackBlock).cast(),
            flags: 0,
            reserved: 0,
            invoke,
            descriptor,
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
        let mut listener_block = block_v1(listener_event, &LISTENER_BLOCK_DESCRIPTOR_V1);
        xpc_connection_set_event_handler(listener, (&mut listener_block as *mut BlockV1).cast());
        xpc_connection_activate(listener);
        dispatch_main();
    }

    unsafe extern "C" fn listener_event(_block: *mut BlockV1, event: *mut core::ffi::c_void) {
        if event.is_null() || xpc_get_type(event) != std::ptr::addr_of!(_xpc_type_connection).cast()
        {
            return;
        }
        let requirement = match mac_verified_control_peer_requirement_v1()
            .and_then(|value| std::ffi::CString::new(value).context("encode fixed XPC requirement"))
        {
            Ok(value) => value,
            Err(_) => return,
        };
        if xpc_connection_set_peer_code_signing_requirement(event, requirement.as_ptr()) != 0 {
            xpc_connection_cancel(event);
            return;
        }
        let mut peer_block = PeerBlockV1 {
            block: block_v1(peer_event, &PEER_BLOCK_DESCRIPTOR_V1),
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
            || xpc_get_type(event) != std::ptr::addr_of!(_xpc_type_dictionary).cast()
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
        let authority = mac_verified_control_authority_v1()?;
        attest_mac_xpc_control_image_v1(event, &authority)?;
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
#[link(name = "System")]
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
    ) -> i32;
    fn xpc_connection_get_audit_token(connection: *mut core::ffi::c_void) -> MacAuditTokenFfiV1;
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
