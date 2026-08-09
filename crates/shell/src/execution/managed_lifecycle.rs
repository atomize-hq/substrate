mod linux_client;
mod macos_client;
mod windows_client;

use anyhow::{anyhow, bail, Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use substrate_common::{
    action_receipt_artifact_sha256_v1, canonical_action_receipt_bytes_v1,
    canonical_action_receipt_index_bytes_v1, canonical_manifest_bytes_v1,
    commit_action_receipt_index_to_head_v1, compare_and_swap_action_receipt_index_v1,
    managed_action_prepared_record_sha256_v1, parse_and_validate_manifest_v1,
    validate_guest_publisher_pairing_session_binding_v1,
    validate_guest_publisher_pairing_ticket_v1, validate_lifecycle_publisher_protected_state_v1,
    validate_managed_action_receipt_signature_v1, validate_managed_lifecycle_publisher_request_v1,
    CanonicalManifestBytesV1, ExecutorBuildEvidenceV1, GuestPublisherPairingSessionBindingV1,
    GuestPublisherPairingTicketV1, LifecyclePublisherProtectedStateV1, LimaStageOneAuthorizationV1,
    MacPublisherBootstrapRequestV1, ManagedActionReceiptIndexEntryV1, ManagedActionReceiptIndexV1,
    ManagedActionReceiptV1, ManagedActionV1, ManagedArtifactManifestV1,
    ManagedLifecyclePublisherRequestV1, ManagedLifecycleStateV1, ManagedManifestHeadV1,
    ManagedSharedClaimsV1, PublisherBootstrapAuthorizationV1,
};
use transport_api_types::{
    InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1, PlatformInstanceIdentityV1,
    PlatformPrincipalV1,
};

const MANIFEST_HEAD_FILENAME_V1: &str = "head.v1.json";
const MANAGED_SHARED_CLAIMS_FILENAME_V1: &str = "shared-claims.v1.json";
const MANIFEST_CAPSULE_DIRECTORY_V1: &str = ".substrate-lifecycle-v1";
const PROVIDER_UNAVAILABLE_KIND_V1: &str = "provider_unavailable";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MappedLifecycleTagV1 {
    #[serde(rename = "stage_one_create")]
    StageOneCreate,
    #[serde(rename = "post_pm_action")]
    PostPmAction,
    #[serde(rename = "guest_pairing_data_session")]
    GuestPairingDataSession,
}

/// The only ordinary control payload accepted by the R5 mapped-lifecycle bridge.
///
/// Legacy fields remain solely to decode and preserve pre-R5 records inside the existing shell
/// crate.  The control binary admits only a request with one of the two `tag` values below and
/// never dispatches those legacy fields as a compatibility route.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManagedLifecycleControlRequestV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<MappedLifecycleTagV1>,
    #[serde(default)]
    pub authority_domain: String,
    #[serde(default)]
    pub scope_id: String,
    #[serde(default)]
    pub selected_host_prefix: String,
    #[serde(default)]
    pub requester_principal: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_context_commitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_mapping_commitment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_platform_control_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<ManagedArtifactManifestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_receipt: Option<ManagedActionReceiptV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher_protected_state: Option<LifecyclePublisherProtectedStateV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher_request: Option<ManagedLifecyclePublisherRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_bootstrap_context_v1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform_bootstrap_mapping_v1: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor_build_evidence: Option<ExecutorBuildEvidenceV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lima_stage_one_authorization_v1: Option<LimaStageOneAuthorizationV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairing_ticket: Option<GuestPublisherPairingTicketV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairing_session_binding_v1: Option<GuestPublisherPairingSessionBindingV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairing_host_record_generation: Option<u64>,
    /// The current protected-record generation required for this operation.  The adjacent
    /// `pairing_host_record_generation` remains the immutable generation in the session binding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairing_record_expected_generation_v1: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pairing_host_record_sha256: Option<String>,
}

pub trait LifecyclePublisherClientV1: Send + Sync {
    fn bootstrap_publisher_v1(
        &self,
        authorization: &PublisherBootstrapAuthorizationV1,
    ) -> Result<Value>;

    fn submit_publisher_request_v1(
        &self,
        request: &ManagedLifecyclePublisherRequestV1,
    ) -> Result<Value>;

    fn issue_guest_publisher_pairing_ticket_v1(
        &self,
        request: &ManagedLifecyclePublisherRequestV1,
    ) -> Result<GuestPublisherPairingTicketV1>;
}

pub fn publish_manifest_v1(
    capsule_root: &Path,
    manifest: &ManagedArtifactManifestV1,
) -> Result<CanonicalManifestBytesV1> {
    let capsule_root = open_trusted_lifecycle_capsule_v1(capsule_root)?;
    let canonical = canonical_manifest_bytes_v1(manifest)?;
    let manifest_path = manifest_file_path(&capsule_root, manifest.manifest_generation);
    write_bytes_if_absent_or_equal(&manifest_path, &canonical.bytes)
        .with_context(|| format!("publish manifest {}", manifest_path.display()))?;

    let receipts_dir = receipts_generation_directory(&capsule_root, manifest.manifest_generation);
    fs::create_dir_all(&receipts_dir)
        .with_context(|| format!("create receipts directory {}", receipts_dir.display()))?;
    sync_parent_directory(&receipts_dir)?;

    let index = load_or_initialize_index_for_manifest(&capsule_root, manifest)?;
    load_or_initialize_head_for_manifest(&capsule_root, manifest, &index)?;
    Ok(canonical)
}

pub fn load_manifest_v1(
    capsule_root: &Path,
    manifest_generation: u64,
) -> Result<ManagedArtifactManifestV1> {
    let manifest_path = manifest_file_path(capsule_root, manifest_generation);
    let bytes = fs::read(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    parse_and_validate_manifest_v1(&bytes)
}

pub fn derive_lifecycle_capsule_locator_v1(
    authority_domain: &str,
    selected_host_prefix: &Path,
    platform_mapping_commitment: Option<&str>,
    host_platform_control_root: Option<&Path>,
) -> Result<PathBuf> {
    Ok(match authority_domain {
        "unix_a_local" | "windows_a_local" => {
            selected_host_prefix.join(MANIFEST_CAPSULE_DIRECTORY_V1)
        }
        "linux_system" | "mac_lima_guest" | "windows_wsl_guest" => {
            PathBuf::from("/var/lib/substrate").join(MANIFEST_CAPSULE_DIRECTORY_V1)
        }
        "mac_host_shared" => {
            let root = host_platform_control_root
                .ok_or_else(|| anyhow!("mac_host_shared requires host_platform_control_root"))?;
            let commitment = platform_mapping_commitment
                .ok_or_else(|| anyhow!("mac_host_shared requires platform_mapping_commitment"))?;
            root.join(MANIFEST_CAPSULE_DIRECTORY_V1).join(commitment)
        }
        "windows_host_shared" => {
            let root = host_platform_control_root.ok_or_else(|| {
                anyhow!("windows_host_shared requires host_platform_control_root")
            })?;
            root.join(MANIFEST_CAPSULE_DIRECTORY_V1)
        }
        other => bail!("unknown lifecycle authority_domain {other}"),
    })
}

pub fn open_trusted_lifecycle_capsule_v1(capsule_root: &Path) -> Result<PathBuf> {
    fs::create_dir_all(capsule_root)
        .with_context(|| format!("create lifecycle capsule {}", capsule_root.display()))?;
    let canonical = fs::canonicalize(capsule_root)
        .with_context(|| format!("canonicalize lifecycle capsule {}", capsule_root.display()))?;
    if !canonical.is_dir() {
        bail!("lifecycle capsule root is not a directory");
    }
    Ok(canonical)
}

pub fn compare_and_swap_head_v1(
    capsule_root: &Path,
    expected: Option<&ManagedManifestHeadV1>,
    next: &ManagedManifestHeadV1,
) -> Result<ManagedManifestHeadV1> {
    let head_path = head_file_path(capsule_root);
    with_locked_target(&head_path, || {
        match fs::read(&head_path) {
            Ok(bytes) => {
                let current: ManagedManifestHeadV1 =
                    serde_json::from_slice(&bytes).context("decode current head")?;
                if current == *next {
                    return Ok(current);
                }
                if let Some(expected) = expected {
                    if &current != expected {
                        bail!("manifest head compare-and-swap mismatch");
                    }
                } else {
                    bail!("manifest head already exists");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if expected.is_some() {
                    bail!("manifest head compare-and-swap expected an existing head");
                }
            }
            Err(error) => return Err(error).context("read current manifest head"),
        }

        let bytes = canonical_json_to_vec(next).context("encode next manifest head")?;
        write_bytes_atomically(&head_path, &bytes)?;
        Ok(next.clone())
    })
}

pub fn transition_manifest_v1(
    manifest: &ManagedArtifactManifestV1,
    next_state: ManagedLifecycleStateV1,
) -> Result<ManagedArtifactManifestV1> {
    let mut next = manifest.clone();
    next.lifecycle_state = next_state;
    next.manifest_sha256 = manifest_digest_v1(&next)?;
    canonical_manifest_bytes_v1(&next).context("validate transitioned manifest")?;
    Ok(next)
}

pub fn update_shared_claims_v1(
    capsule_root: &Path,
    expected: Option<&ManagedSharedClaimsV1>,
    shared_claims: &ManagedSharedClaimsV1,
) -> Result<ManagedSharedClaimsV1> {
    let path = capsule_root.join(MANAGED_SHARED_CLAIMS_FILENAME_V1);
    with_locked_target(&path, || {
        let head = load_required_head(capsule_root)?;
        validate_shared_claims_against_head_v1(&head, shared_claims)?;
        match fs::read(&path) {
            Ok(bytes) => {
                let current: ManagedSharedClaimsV1 =
                    serde_json::from_slice(&bytes).context("decode current shared claims")?;
                if current == *shared_claims {
                    return Ok(current);
                }
                if let Some(expected) = expected {
                    if &current != expected {
                        bail!("shared claims compare-and-swap mismatch");
                    }
                } else {
                    bail!("shared claims already exist");
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if expected.is_some() {
                    bail!("shared claims compare-and-swap expected an existing value");
                }
            }
            Err(error) => return Err(error).context("read current shared claims"),
        }

        let bytes = canonical_json_to_vec(shared_claims).context("encode shared claims")?;
        write_bytes_atomically(&path, &bytes)?;
        Ok(shared_claims.clone())
    })
}

pub fn publish_action_receipt_index_v1(
    capsule_root: &Path,
    index: &ManagedActionReceiptIndexV1,
) -> Result<()> {
    let path = action_receipt_index_path(capsule_root, index.manifest_generation);
    with_locked_target(&path, || {
        let bytes = canonical_action_receipt_index_bytes_v1(index)?;
        match fs::read(&path) {
            Ok(current_bytes) => {
                if current_bytes == bytes {
                    return Ok(());
                }
                let current: ManagedActionReceiptIndexV1 =
                    serde_json::from_slice(&current_bytes)
                        .context("decode current action receipt index")?;
                compare_and_swap_action_receipt_index_v1(&current, index)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if index.index_revision != 0 {
                    bail!("action receipt index compare-and-swap expected an existing index");
                }
            }
            Err(error) => return Err(error).context("read current action receipt index"),
        }
        write_bytes_atomically(&path, &bytes)
    })
}

pub fn resume_action_receipt_commit_v1(
    capsule_root: &Path,
    manifest: &ManagedArtifactManifestV1,
    protected_state: &LifecyclePublisherProtectedStateV1,
    receipt: &ManagedActionReceiptV1,
) -> Result<ManagedManifestHeadV1> {
    validate_lifecycle_publisher_protected_state_v1(protected_state)?;
    ensure_protected_state_matches_manifest(protected_state, manifest)?;
    let prepared_record = required_prepared_record(protected_state)?;
    validate_managed_action_receipt_signature_v1(receipt)?;
    ensure_receipt_signer_matches_prepared_record(prepared_record, receipt)?;
    ensure_receipt_matches_prepared_record(prepared_record, receipt)?;
    ensure_receipt_matches_manifest_plan(manifest, receipt)?;

    let receipt_bytes = canonical_action_receipt_bytes_v1(receipt)?;
    let receipt_path = capsule_root.join(&receipt.receipt_relative_path);
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create receipt parent {}", parent.display()))?;
    }
    write_bytes_if_absent_or_equal(&receipt_path, &receipt_bytes)
        .with_context(|| format!("publish receipt {}", receipt_path.display()))?;

    let current_index = load_or_initialize_index_for_manifest(capsule_root, manifest)?;
    let existing_entry = current_index
        .entries
        .iter()
        .find(|entry| entry.receipt_id == receipt.receipt_id);
    let next_index = if let Some(existing) = existing_entry {
        ensure_index_entry_matches_receipt(existing, &receipt_path, receipt)?;
        current_index.clone()
    } else {
        let mut next = current_index.clone();
        next.index_revision += 1;
        next.previous_index_sha256 = Some(lower_hex(&Sha256::digest(
            canonical_action_receipt_index_bytes_v1(&current_index)?,
        )));
        next.entries
            .push(build_receipt_index_entry(&receipt_path, receipt)?);
        compare_and_swap_action_receipt_index_v1(&current_index, &next)?;
        publish_action_receipt_index_v1(capsule_root, &next)?;
        next
    };

    let existing_head = load_head(capsule_root)?;
    if let Some(head) = existing_head.as_ref() {
        ensure_head_matches_manifest(head, manifest)?;
        if head_matches_index(head, &next_index)? {
            return Ok(head.clone());
        }
    } else {
        bail!("manifest head is missing for the published manifest generation");
    }

    let current_head = existing_head.clone().expect("checked above");
    let next_head = commit_action_receipt_index_to_head_v1(&current_head, &next_index)?;
    compare_and_swap_head_v1(capsule_root, existing_head.as_ref(), &next_head)?;
    Ok(next_head)
}

/// Complete the one R5-only Lima Stage-1 cross-generation transition.
///
/// The ordinary receipt validator intentionally remains same-manifest: its use here would make
/// a pre-PM anchor appear to authorize a fabricated post-PM mapping.  This narrowly scoped
/// completer instead verifies `N -> N+1`, publishes the PM manifest's empty index/head first,
/// then publishes its single signed receipt/index/head with absent-or-exact writes.  The caller
/// performs the immediately following signed-anchor and protected-state Keychain CAS.
pub fn complete_mac_lima_stage_one_transition_v1(
    capsule_root: &Path,
    source_protected_state: &LifecyclePublisherProtectedStateV1,
    source_manifest_generation: u64,
    source_manifest_sha256: &str,
    next_manifest: &ManagedArtifactManifestV1,
    receipt: &ManagedActionReceiptV1,
) -> Result<ManagedManifestHeadV1> {
    validate_lifecycle_publisher_protected_state_v1(source_protected_state)?;
    let prepared = required_prepared_record(source_protected_state)?;
    let source_anchor = &source_protected_state.current_anchor;
    if source_anchor.authority_domain != "mac_host_shared"
        || source_anchor.platform_mapping_commitment.is_some()
        || source_anchor.manifest_generation != source_manifest_generation
        || source_anchor.manifest_sha256 != source_manifest_sha256
        || next_manifest.authority_domain != "mac_host_shared"
        || next_manifest.platform_kind != "mac_lima"
        || next_manifest.platform_mapping_commitment.is_none()
        || next_manifest.manifest_generation
            != source_manifest_generation
                .checked_add(1)
                .ok_or_else(|| anyhow!("Stage-1 manifest generation overflow"))?
        || next_manifest.previous_manifest_sha256.as_deref() != Some(source_manifest_sha256)
        || next_manifest.host_context_commitment != source_anchor.host_context_commitment
        || prepared.manifest_generation != source_manifest_generation
        || prepared.manifest_sha256 != source_manifest_sha256
        // The durable prepared-state CAS has already advanced `counter` exactly once from the
        // pre-PM anchor state and Common validates `prepared.allocated_counter == counter`.
        // Requiring another increment here would make the stated N -> N+1 transition
        // unsatisfiable and would invite a second effect allocation on retry.
        || prepared.allocated_counter != source_protected_state.counter
        || receipt.manifest_generation != next_manifest.manifest_generation
        || receipt.manifest_sha256 != next_manifest.manifest_sha256
        || receipt.allocated_counter != prepared.allocated_counter
        || receipt.prepared_record_sha256 != managed_action_prepared_record_sha256_v1(prepared)?
        || receipt.scope_id != source_anchor.scope_id
        || receipt.installation_id != source_anchor.scope_id
        || receipt.authority_domain != source_anchor.authority_domain
        || receipt.attempt_id != prepared.attempt_id
        || receipt.request_sha256 != prepared.request_sha256
        || receipt.executor_identity != prepared.executor_identity
        || receipt.signature.algorithm != prepared.signature.algorithm
        || receipt.signature.public_key != prepared.signature.public_key
    {
        bail!("Stage-1 receipt does not complete the exact signed pre-PM to PM transition");
    }
    validate_managed_action_receipt_signature_v1(receipt)?;
    ensure_receipt_matches_manifest_plan(next_manifest, receipt)?;

    // This creates only the generation-N+1 manifest, empty index, and empty head. Any existing
    // byte must be canonical-identical; a mismatched partial transition is never repaired.
    publish_manifest_v1(capsule_root, next_manifest)?;
    let receipt_bytes = canonical_action_receipt_bytes_v1(receipt)?;
    let receipt_path = capsule_root.join(&receipt.receipt_relative_path);
    if !receipt_path.starts_with(capsule_root) {
        bail!("Stage-1 receipt path escapes the lifecycle capsule");
    }
    if let Some(parent) = receipt_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create Stage-1 receipt parent {}", parent.display()))?;
    }
    write_bytes_if_absent_or_equal(&receipt_path, &receipt_bytes)
        .with_context(|| format!("publish Stage-1 receipt {}", receipt_path.display()))?;

    let current_index = load_or_initialize_index_for_manifest(capsule_root, next_manifest)?;
    let next_index = match current_index
        .entries
        .iter()
        .find(|entry| entry.receipt_id == receipt.receipt_id)
    {
        Some(existing) => {
            ensure_index_entry_matches_receipt(existing, &receipt_path, receipt)?;
            current_index.clone()
        }
        None => {
            if !current_index.entries.is_empty() || current_index.index_revision != 0 {
                bail!(
                    "Stage-1 generation-N+1 receipt index is not the required empty initial index"
                );
            }
            let mut next = current_index.clone();
            next.index_revision = 1;
            next.previous_index_sha256 = Some(lower_hex(&Sha256::digest(
                canonical_action_receipt_index_bytes_v1(&current_index)?,
            )));
            next.entries
                .push(build_receipt_index_entry(&receipt_path, receipt)?);
            compare_and_swap_action_receipt_index_v1(&current_index, &next)?;
            publish_action_receipt_index_v1(capsule_root, &next)?;
            next
        }
    };
    let current_head = load_required_head(capsule_root)?;
    ensure_head_matches_manifest(&current_head, next_manifest)?;
    if head_matches_index(&current_head, &next_index)? {
        return Ok(current_head);
    }
    if current_head.action_receipt_index_revision != 0 {
        bail!("Stage-1 generation-N+1 head is not the required empty initial head");
    }
    let next_head = commit_action_receipt_index_to_head_v1(&current_head, &next_index)?;
    compare_and_swap_head_v1(capsule_root, Some(&current_head), &next_head)?;
    Ok(next_head)
}

/// Reject ordinary payloads except closed mapped-lifecycle tags before a client/XPC operation.
/// Publisher bootstrap authorization and direct-only R6 operator TTY admission have no representation here.
pub fn validate_mapped_lifecycle_control_request_v1(
    request: &ManagedLifecycleControlRequestV1,
) -> Result<MappedLifecycleTagV1> {
    let tag = request
        .tag
        .clone()
        .ok_or_else(|| anyhow!("ordinary mapped lifecycle request has no closed tag"))?;
    let carrier = InstallBootstrapContextCarrierV1::decode(
        request
            .install_bootstrap_context_v1
            .as_deref()
            .ok_or_else(|| anyhow!("mapped lifecycle request is missing its carrier"))?,
    )
    .context("decode exact InstallBootstrapContextCarrierV1")?;
    let evidence = request
        .executor_build_evidence
        .as_ref()
        .ok_or_else(|| anyhow!("mapped lifecycle request is missing its build evidence"))?;
    validate_mapped_mac_executor_build_evidence_v1(evidence)?;
    if request.manifest.is_some()
        || request.action_receipt.is_some()
        || request.publisher_protected_state.is_some()
    {
        bail!("ordinary mapped lifecycle request carries a forbidden legacy authority field");
    }
    match tag {
        MappedLifecycleTagV1::StageOneCreate => {
            if request.pairing_ticket.is_some()
                || request.pairing_session_binding_v1.is_some()
                || request.pairing_host_record_generation.is_some()
                || request.pairing_record_expected_generation_v1.is_some()
                || request.pairing_host_record_sha256.is_some()
            {
                bail!("stage_one_create must not carry R6 pairing authority");
            }
            if request.publisher_request.is_some()
                || request.platform_bootstrap_mapping_v1.is_some()
                || request.platform_mapping_commitment.is_some()
                || request.host_platform_control_root.is_some()
            {
                bail!("stage_one_create must remain the sole pre-PM authority and cannot carry PM or publisher fields");
            }
            let stage_one = request
                .lima_stage_one_authorization_v1
                .as_ref()
                .ok_or_else(|| {
                    anyhow!("stage_one_create is missing LimaStageOneAuthorizationV1")
                })?;
            substrate_common::validate_lima_stage_one_authorization_v1(stage_one)?;
            if !stage_one.expected_absent
                || stage_one.host_context_commitment != carrier.host_context_commitment
                || stage_one.source_commit != evidence.source_commit
                || stage_one.source_tree != evidence.source_tree
                || stage_one.source_ref != evidence.source_ref
                || stage_one.successor_template.executor_build_evidence != *evidence
                || stage_one.successor_template.selected_host_prefix
                    != carrier.context.selected_host_prefix
            {
                bail!("stage_one_create does not join the exact pre-PM IH, template, and build evidence");
            }
            if request.authority_domain.is_empty() {
                // The closed wrapper deliberately supplies no duplicate caller-selected domain.
            } else if request.authority_domain != "mac_host_shared" {
                bail!("stage_one_create has a conflicting authority domain");
            }
            for (value, expected, field) in [
                (
                    request.scope_id.as_str(),
                    stage_one.successor_template.scope_id.as_str(),
                    "scope_id",
                ),
                (
                    request.selected_host_prefix.as_str(),
                    stage_one.successor_template.selected_host_prefix.as_str(),
                    "selected_host_prefix",
                ),
                (
                    request.requester_principal.as_str(),
                    stage_one.successor_template.intended_principal.as_str(),
                    "requester_principal",
                ),
            ] {
                if !value.is_empty() && value != expected {
                    bail!("stage_one_create {field} conflicts with its signed pre-PM template");
                }
            }
            if let Some(commitment) = &request.host_context_commitment {
                if commitment != &carrier.host_context_commitment {
                    bail!("stage_one_create host_context_commitment conflicts with its IH carrier");
                }
            }
        }
        MappedLifecycleTagV1::PostPmAction => {
            if request.pairing_ticket.is_some()
                || request.pairing_session_binding_v1.is_some()
                || request.pairing_host_record_generation.is_some()
                || request.pairing_record_expected_generation_v1.is_some()
                || request.pairing_host_record_sha256.is_some()
            {
                bail!("post_pm_action must not carry R6 pairing authority");
            }
            let publisher_request = request.publisher_request.as_ref().ok_or_else(|| {
                anyhow!("post_pm_action is missing its canonical publisher request")
            })?;
            validate_managed_lifecycle_publisher_request_v1(publisher_request)?;
            if request.lima_stage_one_authorization_v1.is_some() {
                bail!("post_pm_action must not carry LimaStageOneAuthorizationV1");
            }
            let mapping = PlatformBootstrapMappingV1::decode(
                request
                    .platform_bootstrap_mapping_v1
                    .as_deref()
                    .ok_or_else(|| anyhow!("post_pm_action is missing its finalized mapping"))?,
                &carrier,
            )
            .context("decode exact post-PM PlatformBootstrapMappingV1")?;
            let canonical_mapping = mapping
                .encode(&carrier)
                .context("re-encode exact post-PM PlatformBootstrapMappingV1")?;
            let mapping_commitment = format!("{:x}", Sha256::digest(canonical_mapping.as_bytes()));
            if request.platform_mapping_commitment.as_deref() != Some(mapping_commitment.as_str())
                || publisher_request.platform_mapping_commitment.as_deref()
                    != Some(mapping_commitment.as_str())
            {
                bail!(
                    "post_pm_action mapping commitment does not bind the exact finalized mapping"
                );
            }
            let PlatformInstanceIdentityV1::Lima { .. } = &mapping.platform_instance else {
                bail!("post_pm_action does not name a finalized Lima mapping");
            };
            if publisher_request.host_context_commitment != carrier.host_context_commitment
                || publisher_request.expected_executor_build.source_commit != evidence.source_commit
                || publisher_request.expected_executor_build.source_tree != evidence.source_tree
                || publisher_request.expected_executor_build.source_ref != evidence.source_ref
                || publisher_request.expected_executor_build.target_triple != evidence.target_triple
                || publisher_request.expected_executor_build.artifact_sha256
                    != evidence.artifact_sha256
            {
                bail!("post_pm_action does not join carrier, mapping, and build evidence");
            }
            if request
                .host_context_commitment
                .as_deref()
                .is_some_and(|value| value != carrier.host_context_commitment)
                || (!request.scope_id.is_empty() && request.scope_id != publisher_request.scope_id)
                || (!request.selected_host_prefix.is_empty()
                    && request.selected_host_prefix != carrier.context.selected_host_prefix)
                || (!request.requester_principal.is_empty()
                    && request.requester_principal != publisher_request.requester_principal)
                || request
                    .host_platform_control_root
                    .as_deref()
                    .is_some_and(|value| value != mapping.host_platform_control_root)
            {
                bail!("post_pm_action top-level fields conflict with canonical carrier/mapping/request joins");
            }
            let PlatformPrincipalV1::Unix { account, .. } = &mapping.realized_principal else {
                bail!("post_pm_action mapping does not retain a UNIX principal");
            };
            if account != &publisher_request.requester_principal {
                bail!("post_pm_action requester principal does not join finalized mapping");
            }
            if !closed_mapped_mac_role_action_v1(
                &publisher_request.role.0,
                publisher_request.action,
            ) {
                bail!("post_pm_action role/action is outside the closed MAC map");
            }
        }
        MappedLifecycleTagV1::GuestPairingDataSession => {
            validate_r6_pairing_control_request_v1(request, &tag, &carrier, evidence)?;
        }
    }
    Ok(tag)
}

/// Validate one internally constructed R6 session request.  The tag is a closed protocol
/// discriminant: this function accepts no operation, transport, command, or caller-owned record.
fn validate_r6_pairing_control_request_v1(
    request: &ManagedLifecycleControlRequestV1,
    tag: &MappedLifecycleTagV1,
    carrier: &InstallBootstrapContextCarrierV1,
    evidence: &ExecutorBuildEvidenceV1,
) -> Result<()> {
    if request.publisher_request.is_some()
        || request.lima_stage_one_authorization_v1.is_some()
        || request.manifest.is_some()
        || request.action_receipt.is_some()
        || request.publisher_protected_state.is_some()
    {
        bail!("R6 pairing session request carries forbidden generic lifecycle authority");
    }
    if request.authority_domain != "mac_lima_guest" {
        bail!("R6 pairing session request must use the fixed mac_lima_guest authority");
    }
    let issue_data_session = matches!(tag, MappedLifecycleTagV1::GuestPairingDataSession)
        && request.pairing_session_binding_v1.is_none()
        && request.pairing_host_record_generation.is_none()
        && request.pairing_record_expected_generation_v1.is_none()
        && request.pairing_host_record_sha256.is_none()
        && request.pairing_ticket.is_none();
    // This is not an operator transport or caller-controlled status: the hidden direct control
    // uses this ticket-less, fully record-bound form only to record its own fixed operator-child
    // failure. It carries no observation text, terminal bytes, selector, or authority material.
    let operator_failure_observation = matches!(tag, MappedLifecycleTagV1::GuestPairingDataSession)
        && request.pairing_ticket.is_none()
        && request.pairing_session_binding_v1.is_some()
        && request.pairing_host_record_generation.is_some()
        && request.pairing_record_expected_generation_v1.is_some()
        && request.pairing_host_record_sha256.is_some();
    if !issue_data_session
        && !operator_failure_observation
        && (request.pairing_session_binding_v1.is_none()
            || request.pairing_host_record_generation.is_none()
            || request.pairing_record_expected_generation_v1.is_none()
            || request.pairing_host_record_sha256.is_none())
    {
        bail!("R6 pairing session request must carry a complete immutable record binding");
    }
    let mapping = PlatformBootstrapMappingV1::decode(
        request
            .platform_bootstrap_mapping_v1
            .as_deref()
            .ok_or_else(|| anyhow!("R6 pairing session request is missing finalized mapping"))?,
        carrier,
    )
    .context("decode exact R6 PlatformBootstrapMappingV1")?;
    let canonical_mapping = mapping
        .encode(carrier)
        .context("re-encode exact R6 PlatformBootstrapMappingV1")?;
    let mapping_commitment = format!("{:x}", Sha256::digest(canonical_mapping.as_bytes()));
    if !matches!(
        &mapping.platform_instance,
        PlatformInstanceIdentityV1::Lima { .. }
    ) || request.platform_mapping_commitment.as_deref() != Some(mapping_commitment.as_str())
    {
        bail!("R6 pairing session does not bind the exact finalized Lima mapping");
    }
    if request.scope_id.is_empty()
        || request.selected_host_prefix != carrier.context.selected_host_prefix
        || request.requester_principal.is_empty()
        || request.host_context_commitment.as_deref()
            != Some(carrier.host_context_commitment.as_str())
        || request.host_platform_control_root.as_deref()
            != Some(mapping.host_platform_control_root.as_str())
    {
        bail!("R6 pairing session does not join exact PM, source, and artifact identity");
    }
    let PlatformPrincipalV1::Unix { account, .. } = &mapping.realized_principal else {
        bail!("R6 pairing session mapping does not retain a UNIX principal");
    };
    if &request.requester_principal != account {
        bail!("R6 pairing session principal does not join the finalized mapping");
    }
    if issue_data_session {
        return Ok(());
    }
    let binding = request
        .pairing_session_binding_v1
        .as_ref()
        .expect("complete binding checked above");
    validate_guest_publisher_pairing_session_binding_v1(binding)?;
    let record_generation = request
        .pairing_host_record_generation
        .expect("complete generation checked above");
    if record_generation != binding.host_record_generation {
        bail!("R6 pairing session record generation does not match immutable binding");
    }
    if request
        .pairing_record_expected_generation_v1
        .expect("complete current generation checked above")
        == 0
    {
        bail!("R6 pairing session expected record generation must be positive");
    }
    let record_sha256 = request
        .pairing_host_record_sha256
        .as_deref()
        .expect("complete record digest checked above");
    if !is_lower_hex_v1(record_sha256, 64) {
        bail!("R6 pairing session record digest is not canonical");
    }
    if binding.platform_mapping_commitment.as_deref() != Some(mapping_commitment.as_str())
        || request.scope_id != binding.scope_id
        || binding.source_commit != evidence.source_commit
        || binding.source_tree != evidence.source_tree
        || binding.source_ref != evidence.source_ref
        || binding.staged_executor_sha256 != evidence.artifact_sha256
    {
        bail!("R6 pairing session immutable binding does not join PM, source, and artifact");
    }
    match tag {
        MappedLifecycleTagV1::GuestPairingDataSession => {
            if operator_failure_observation {
                return Ok(());
            }
            let ticket = request
                .pairing_ticket
                .as_ref()
                .ok_or_else(|| anyhow!("R6 data session is missing its signed ticket"))?;
            validate_guest_publisher_pairing_ticket_v1(ticket)?;
            if ticket.current_anchor.scope_id != binding.scope_id
                || ticket.challenge.challenge_id != binding.ticket_challenge_id
                || ticket.challenge.guest_machine_identity != binding.guest_machine_identity
                || ticket.challenge.source_commit != binding.source_commit
                || ticket.challenge.source_tree != binding.source_tree
                || ticket.challenge.source_ref != binding.source_ref
                || ticket.challenge.executor_build_evidence_sha256 != binding.staged_executor_sha256
                || ticket.host_generation != binding.host_record_generation
            {
                bail!("R6 data session ticket does not match immutable binding");
            }
        }
        _ => bail!("R6 session validator received a non-data tag"),
    }
    Ok(())
}

/// Validate the exact build fields carried by the R4 `ExecutorBuildEvidenceV1` wire form.
///
/// The common validator is deliberately private to its bootstrap authorization API, so the R5
/// ordinary bridge repeats the same structural fence instead of accepting an untyped JSON value.
fn validate_mapped_mac_executor_build_evidence_v1(
    evidence: &ExecutorBuildEvidenceV1,
) -> Result<()> {
    if evidence.schema_owner != "substrate.executor-build-evidence"
        || evidence.schema_version != 1
        || !is_lower_hex_v1(&evidence.source_commit, 40)
        || !is_lower_hex_v1(&evidence.source_tree, 40)
        || !is_lower_hex_v1(&evidence.artifact_sha256, 64)
        || evidence.source_ref.is_empty()
        || evidence.artifact_identity.is_empty()
        || evidence.target_triple != "aarch64-apple-darwin"
        || [
            &evidence.source_ref,
            &evidence.artifact_identity,
            &evidence.target_triple,
        ]
        .iter()
        .any(|value| value.contains(['\0', '\n', '\r']))
    {
        bail!("mapped lifecycle request has no exact ExecutorBuildEvidenceV1 join");
    }
    Ok(())
}

fn is_lower_hex_v1(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

/// The ordinary request map is intentionally narrower than the common role registry. The latter
/// includes bootstrap endpoints, forwarding, and guest-pairing rows which cannot become R5
/// caller authority merely because they have a well-formed common role syntax.
fn closed_mapped_mac_role_action_v1(role: &str, action: ManagedActionV1) -> bool {
    let create_replace_remove_restore = matches!(
        action,
        ManagedActionV1::Create
            | ManagedActionV1::Replace
            | ManagedActionV1::Remove
            | ManagedActionV1::Restore
    );
    let create_remove_restore = matches!(
        action,
        ManagedActionV1::Create | ManagedActionV1::Remove | ManagedActionV1::Restore
    );
    match role {
        "mac.lima.instance" => matches!(
            action,
            ManagedActionV1::Start
                | ManagedActionV1::Stop
                | ManagedActionV1::Remove
                | ManagedActionV1::Restore
        ),
        "mac.lima.staged-workspace"
        | "mac.lima.layout-sentinel"
        | "mac.lima.publisher-executor"
        | "mac.host.known-hosts-entry" => create_replace_remove_restore,
        "mac.lima.guest-group" | "mac.lima.guest-private-home" => create_remove_restore,
        "mac.lima.publisher-state-directory" => {
            matches!(action, ManagedActionV1::Create | ManagedActionV1::Remove)
        }
        "mac.lima.publisher-service-unit" | "mac.lima.publisher-socket-unit" => {
            matches!(action, ManagedActionV1::Create | ManagedActionV1::Restore)
        }
        "mac.lima.publisher-signing-key" => matches!(action, ManagedActionV1::Create),
        "mac.lima.publisher-current-anchor" | "mac.lima.publisher-bootstrap-intent" => {
            matches!(action, ManagedActionV1::Create | ManagedActionV1::Replace)
        }
        "mac.lima.guest-binary(substrate-world-service)"
        | "mac.lima.guest-binary(substrate-gateway)"
        | "mac.lima.guest-binary(substrate)"
        | "mac.lima.guest-unit(service)"
        | "mac.lima.guest-unit(socket)" => create_replace_remove_restore,
        _ if role.starts_with("mac.lima.guest-directory(")
            || role.starts_with("mac.lima.guest-membership(") =>
        {
            create_remove_restore
        }
        _ if role.starts_with("mac.lima.guest-service-state(") => matches!(
            action,
            ManagedActionV1::Enable
                | ManagedActionV1::Disable
                | ManagedActionV1::Start
                | ManagedActionV1::Stop
                | ManagedActionV1::Restore
        ),
        _ => false,
    }
}

/// The hidden direct command passes one already-decoded canonical seed after terminal
/// confirmation. On macOS the retained-state issuer derives the authorization in memory and this
/// function gives it exactly one FD3 seqpacket delivery opportunity.
pub fn deliver_retained_publisher_bootstrap_authorization_v1(
    request: &MacPublisherBootstrapRequestV1,
) -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        let authorization =
            macos_client::derive_retained_publisher_bootstrap_authorization_v1(request)?;
        return macos_client::send_publisher_bootstrap_authorization_fd3_v1(&authorization);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = request;
        Err(provider_unavailable_error(
            "macos",
            "deliver_retained_publisher_bootstrap_authorization_v1",
        ))
    }
}

pub fn submit_stage_one_absent_instance_create_v1(
    request: &ManagedLifecycleControlRequestV1,
) -> Result<Value> {
    macos_client::submit_stage_one_absent_instance_create_v1(request)
}

pub fn submit_post_pm_managed_action_v1(
    request: &ManagedLifecycleControlRequestV1,
) -> Result<Value> {
    macos_client::submit_post_pm_managed_action_v1(request)
}

/// Relay the only ticket/frame-bearing R6 child after its closed-tag validation.
pub fn submit_guest_pairing_data_session_v1(
    request: &ManagedLifecycleControlRequestV1,
) -> Result<Value> {
    if validate_mapped_lifecycle_control_request_v1(request)?
        != MappedLifecycleTagV1::GuestPairingDataSession
    {
        bail!("R6 data-session relay received a non-data tag");
    }
    macos_client::submit_guest_pairing_data_session_v1(request)
}

pub fn open_publisher_bootstrap_channel_v1(
    authority_domain: &str,
) -> Result<Box<dyn LifecyclePublisherClientV1>> {
    let platform = match authority_domain {
        "linux_system" => ManagedLifecyclePlatformV1::Linux,
        "mac_host_shared" | "mac_lima_guest" => ManagedLifecyclePlatformV1::Macos,
        "windows_host_shared" | "windows_wsl_guest" => ManagedLifecyclePlatformV1::Windows,
        other => bail!("no lifecycle publisher client for authority_domain {other}"),
    };
    Ok(Box::new(StubLifecyclePublisherClientV1 { platform }))
}

pub fn validate_publisher_response_v1(
    response_bytes: &[u8],
    request: &ManagedLifecyclePublisherRequestV1,
) -> Result<Value> {
    validate_managed_lifecycle_publisher_request_v1(request)?;
    let value: Value =
        serde_json::from_slice(response_bytes).context("decode publisher response JSON")?;
    let response_object = value
        .as_object()
        .ok_or_else(|| anyhow!("publisher response must be a JSON object"))?;
    let request_sha256 = response_object
        .get("request_sha256")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("publisher response is missing request_sha256"))?;
    let expected = canonical_request_sha256(request)?;
    if request_sha256 != expected {
        bail!("publisher response request_sha256 does not match the submitted request");
    }
    Ok(value)
}

#[derive(Debug, Clone, Copy)]
enum ManagedLifecyclePlatformV1 {
    Linux,
    Macos,
    Windows,
}

struct StubLifecyclePublisherClientV1 {
    platform: ManagedLifecyclePlatformV1,
}

impl LifecyclePublisherClientV1 for StubLifecyclePublisherClientV1 {
    fn bootstrap_publisher_v1(
        &self,
        authorization: &PublisherBootstrapAuthorizationV1,
    ) -> Result<Value> {
        match self.platform {
            ManagedLifecyclePlatformV1::Linux => {
                linux_client::bootstrap_publisher_v1(authorization)
            }
            ManagedLifecyclePlatformV1::Macos => {
                macos_client::bootstrap_publisher_v1(authorization)
            }
            ManagedLifecyclePlatformV1::Windows => {
                windows_client::bootstrap_publisher_v1(authorization)
            }
        }
    }

    fn submit_publisher_request_v1(
        &self,
        request: &ManagedLifecyclePublisherRequestV1,
    ) -> Result<Value> {
        match self.platform {
            ManagedLifecyclePlatformV1::Linux => linux_client::submit_publisher_request_v1(request),
            ManagedLifecyclePlatformV1::Macos => macos_client::submit_publisher_request_v1(request),
            ManagedLifecyclePlatformV1::Windows => {
                windows_client::submit_publisher_request_v1(request)
            }
        }
    }

    fn issue_guest_publisher_pairing_ticket_v1(
        &self,
        request: &ManagedLifecyclePublisherRequestV1,
    ) -> Result<GuestPublisherPairingTicketV1> {
        match self.platform {
            ManagedLifecyclePlatformV1::Linux => {
                linux_client::issue_guest_publisher_pairing_ticket_v1(request)
            }
            ManagedLifecyclePlatformV1::Macos => {
                macos_client::issue_guest_publisher_pairing_ticket_v1(request)
            }
            ManagedLifecyclePlatformV1::Windows => {
                windows_client::issue_guest_publisher_pairing_ticket_v1(request)
            }
        }
    }
}

fn provider_unavailable_error(platform: &str, operation: &str) -> anyhow::Error {
    anyhow!(
        "{{\"kind\":\"{PROVIDER_UNAVAILABLE_KIND_V1}\",\"platform\":\"{platform}\",\"operation\":\"{operation}\",\"message\":\"{platform} lifecycle provider is not available until its dedicated platform packet lands\"}}"
    )
}

fn manifest_file_path(capsule_root: &Path, manifest_generation: u64) -> PathBuf {
    capsule_root.join(format!("manifest.{manifest_generation}.json"))
}

fn head_file_path(capsule_root: &Path) -> PathBuf {
    capsule_root.join(MANIFEST_HEAD_FILENAME_V1)
}

fn action_receipt_index_path(capsule_root: &Path, manifest_generation: u64) -> PathBuf {
    capsule_root.join(format!("action-receipts.{manifest_generation}.v1.json"))
}

fn receipts_generation_directory(capsule_root: &Path, manifest_generation: u64) -> PathBuf {
    capsule_root
        .join("receipts")
        .join(manifest_generation.to_string())
}

fn build_receipt_index_entry(
    receipt_path: &Path,
    receipt: &ManagedActionReceiptV1,
) -> Result<ManagedActionReceiptIndexEntryV1> {
    let metadata = fs::metadata(receipt_path)
        .with_context(|| format!("metadata for receipt {}", receipt_path.display()))?;
    let parent = receipt_path
        .parent()
        .ok_or_else(|| anyhow!("receipt path has no parent"))?;
    Ok(ManagedActionReceiptIndexEntryV1 {
        receipt_id: receipt.receipt_id.clone(),
        entry_id: receipt.entry_id.clone(),
        action: receipt.action,
        attempt_id: receipt.attempt_id.clone(),
        receipt_relative_path: receipt.receipt_relative_path.clone(),
        canonical_byte_length: metadata.len(),
        receipt_artifact_sha256: action_receipt_artifact_sha256_v1(
            &fs::read(receipt_path)
                .with_context(|| format!("read published receipt {}", receipt_path.display()))?,
        ),
        retained_receipt_file_identity: receipt_path.display().to_string(),
        retained_receipt_parent_identity: parent.display().to_string(),
        prepared_record_sha256: receipt.prepared_record_sha256.clone(),
        allocated_counter: receipt.allocated_counter,
        durable_observation: Value::Object(Map::new()),
    })
}

fn load_head(capsule_root: &Path) -> Result<Option<ManagedManifestHeadV1>> {
    let path = head_file_path(capsule_root);
    match fs::read(&path) {
        Ok(bytes) => {
            let head = serde_json::from_slice(&bytes).context("decode manifest head")?;
            Ok(Some(head))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("read manifest head"),
    }
}

fn load_required_head(capsule_root: &Path) -> Result<ManagedManifestHeadV1> {
    load_head(capsule_root)?
        .ok_or_else(|| anyhow!("manifest head is missing for the published manifest generation"))
}

fn load_or_initialize_index_for_manifest(
    capsule_root: &Path,
    manifest: &ManagedArtifactManifestV1,
) -> Result<ManagedActionReceiptIndexV1> {
    let path = action_receipt_index_path(capsule_root, manifest.manifest_generation);
    with_locked_target(&path, || match fs::read(&path) {
        Ok(bytes) => {
            let current: ManagedActionReceiptIndexV1 =
                serde_json::from_slice(&bytes).context("decode action receipt index")?;
            canonical_action_receipt_index_bytes_v1(&current)?;
            ensure_index_matches_manifest(&current, manifest)?;
            Ok(current)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let index = empty_action_receipt_index_for_manifest(manifest);
            let bytes = canonical_action_receipt_index_bytes_v1(&index)?;
            write_bytes_atomically(&path, &bytes)?;
            Ok(index)
        }
        Err(error) => Err(error).context("read action receipt index"),
    })
}

fn empty_action_receipt_index_for_manifest(
    manifest: &ManagedArtifactManifestV1,
) -> ManagedActionReceiptIndexV1 {
    ManagedActionReceiptIndexV1 {
        schema_owner: "substrate.managed-action-receipt-index".to_string(),
        schema_version: 1,
        authority_domain: manifest.authority_domain.clone(),
        scope_id: manifest.installation_id.clone(),
        manifest_generation: manifest.manifest_generation,
        manifest_sha256: manifest.manifest_sha256.clone(),
        index_revision: 0,
        previous_index_sha256: None,
        entries: Vec::new(),
    }
}

fn initial_head_for_manifest(
    manifest: &ManagedArtifactManifestV1,
    current_index: &ManagedActionReceiptIndexV1,
) -> Result<ManagedManifestHeadV1> {
    Ok(ManagedManifestHeadV1 {
        schema_owner: "substrate.managed-manifest-head".to_string(),
        schema_version: 1,
        scope_id: manifest.installation_id.clone(),
        manifest_generation: manifest.manifest_generation,
        manifest_sha256: manifest.manifest_sha256.clone(),
        lifecycle_state: manifest.lifecycle_state,
        previous_head_sha256: None,
        action_receipt_index_revision: current_index.index_revision,
        action_receipt_index_sha256: lower_hex(&Sha256::digest(
            canonical_action_receipt_index_bytes_v1(current_index)?,
        )),
    })
}

fn load_or_initialize_head_for_manifest(
    capsule_root: &Path,
    manifest: &ManagedArtifactManifestV1,
    current_index: &ManagedActionReceiptIndexV1,
) -> Result<ManagedManifestHeadV1> {
    let path = head_file_path(capsule_root);
    with_locked_target(&path, || match fs::read(&path) {
        Ok(bytes) => {
            let head: ManagedManifestHeadV1 =
                serde_json::from_slice(&bytes).context("decode manifest head")?;
            ensure_head_matches_manifest(&head, manifest)?;
            if !head_matches_index(&head, current_index)? {
                bail!("manifest head does not match the current action receipt index");
            }
            Ok(head)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if current_index.index_revision != 0 {
                bail!("manifest head is missing for a non-empty action receipt index");
            }
            let head = initial_head_for_manifest(manifest, current_index)?;
            let bytes = canonical_json_to_vec(&head).context("encode manifest head")?;
            write_bytes_atomically(&path, &bytes)?;
            Ok(head)
        }
        Err(error) => Err(error).context("read manifest head"),
    })
}

fn ensure_head_matches_manifest(
    head: &ManagedManifestHeadV1,
    manifest: &ManagedArtifactManifestV1,
) -> Result<()> {
    if head.scope_id != manifest.installation_id
        || head.manifest_generation != manifest.manifest_generation
        || head.manifest_sha256 != manifest.manifest_sha256
        || head.lifecycle_state != manifest.lifecycle_state
    {
        bail!("manifest head does not match the manifest scope");
    }
    Ok(())
}

fn ensure_protected_state_matches_manifest(
    protected_state: &LifecyclePublisherProtectedStateV1,
    manifest: &ManagedArtifactManifestV1,
) -> Result<()> {
    let anchor = &protected_state.current_anchor;
    if anchor.authority_domain != manifest.authority_domain {
        bail!("protected-state authority_domain does not match the manifest");
    }
    if anchor.scope_id != manifest.installation_id {
        bail!("protected-state scope_id does not match the manifest installation_id");
    }
    if anchor.manifest_generation != manifest.manifest_generation {
        bail!("protected-state manifest_generation does not match the manifest");
    }
    if anchor.manifest_sha256 != manifest.manifest_sha256 {
        bail!("protected-state manifest_sha256 does not match the manifest");
    }
    if anchor.host_context_commitment != manifest.host_context_commitment {
        bail!("protected-state host_context_commitment does not match the manifest");
    }
    if anchor.platform_mapping_commitment != manifest.platform_mapping_commitment {
        bail!("protected-state platform_mapping_commitment does not match the manifest");
    }
    Ok(())
}

fn required_prepared_record(
    protected_state: &LifecyclePublisherProtectedStateV1,
) -> Result<&substrate_common::ManagedActionPreparedRecordV1> {
    protected_state
        .prepared_record
        .as_ref()
        .ok_or_else(|| anyhow!("publisher protected state must retain the prepared record"))
}

fn ensure_receipt_signer_matches_prepared_record(
    prepared_record: &substrate_common::ManagedActionPreparedRecordV1,
    receipt: &ManagedActionReceiptV1,
) -> Result<()> {
    if prepared_record.signature.algorithm != receipt.signature.algorithm
        || prepared_record.signature.public_key != receipt.signature.public_key
    {
        bail!("receipt signer must match the prepared-record signer");
    }
    Ok(())
}

fn ensure_receipt_matches_prepared_record(
    prepared_record: &substrate_common::ManagedActionPreparedRecordV1,
    receipt: &ManagedActionReceiptV1,
) -> Result<()> {
    if receipt.authority_domain != prepared_record.authority_domain {
        bail!("receipt authority_domain does not match the prepared record");
    }
    if receipt.scope_id != prepared_record.scope_id {
        bail!("receipt scope_id does not match the prepared record");
    }
    if receipt.installation_id != prepared_record.installation_id {
        bail!("receipt installation_id does not match the prepared record");
    }
    if receipt.manifest_generation != prepared_record.manifest_generation {
        bail!("receipt manifest_generation does not match the prepared record");
    }
    if receipt.manifest_sha256 != prepared_record.manifest_sha256 {
        bail!("receipt manifest_sha256 does not match the prepared record");
    }
    if receipt.receipt_id != prepared_record.receipt_id {
        bail!("receipt receipt_id does not match the prepared record");
    }
    if receipt.receipt_relative_path != prepared_record.receipt_relative_path {
        bail!("receipt path does not match the prepared record");
    }
    if receipt.entry_id != prepared_record.entry_id {
        bail!("receipt entry_id does not match the prepared record");
    }
    if receipt.action != prepared_record.action {
        bail!("receipt action does not match the prepared record");
    }
    if receipt.attempt_id != prepared_record.attempt_id {
        bail!("receipt attempt_id does not match the prepared record");
    }
    if receipt.allocated_counter != prepared_record.allocated_counter {
        bail!("receipt allocated_counter does not match the prepared record");
    }
    if receipt.request_sha256 != prepared_record.request_sha256 {
        bail!("receipt request_sha256 does not match the prepared record");
    }
    if receipt.executor_identity != prepared_record.executor_identity {
        bail!("receipt executor_identity does not match the prepared record");
    }
    if receipt.prepared_record_sha256 != managed_action_prepared_record_sha256_v1(prepared_record)?
    {
        bail!("receipt prepared_record_sha256 does not match the prepared record");
    }
    Ok(())
}

fn validate_shared_claims_against_head_v1(
    head: &ManagedManifestHeadV1,
    shared_claims: &ManagedSharedClaimsV1,
) -> Result<()> {
    if shared_claims.manifest_generation != head.manifest_generation {
        bail!("shared claims manifest_generation does not match the manifest head");
    }
    let head_sha256 = lower_hex(&Sha256::digest(canonical_json_to_vec(head)?));
    if shared_claims.manifest_head_sha256 != head_sha256 {
        bail!("shared claims manifest_head_sha256 does not match the manifest head");
    }
    for claim in &shared_claims.claims {
        if claim.installation_id != head.scope_id {
            bail!("shared claim installation_id does not match the manifest head");
        }
        if claim.manifest_sha256 != head.manifest_sha256 {
            bail!("shared claim manifest_sha256 does not match the manifest head");
        }
    }
    Ok(())
}

fn ensure_receipt_matches_manifest_plan(
    manifest: &ManagedArtifactManifestV1,
    receipt: &ManagedActionReceiptV1,
) -> Result<()> {
    if receipt.authority_domain != manifest.authority_domain {
        bail!("receipt authority_domain does not match manifest");
    }
    if receipt.scope_id != manifest.installation_id {
        bail!("receipt scope_id does not match manifest installation_id");
    }
    if receipt.installation_id != manifest.installation_id {
        bail!("receipt installation_id does not match manifest installation_id");
    }
    if receipt.manifest_generation != manifest.manifest_generation {
        bail!("receipt manifest_generation does not match manifest");
    }
    if receipt.manifest_sha256 != manifest.manifest_sha256 {
        bail!("receipt manifest_sha256 does not match manifest");
    }
    let planned_match = manifest.planned_action_receipts.iter().any(|planned| {
        let Some(object) = planned.as_object() else {
            return false;
        };
        object.get("receipt_id").and_then(Value::as_str) == Some(receipt.receipt_id.as_str())
            && object.get("entry_id").and_then(Value::as_str) == Some(receipt.entry_id.as_str())
            && object.get("attempt_id").and_then(Value::as_str) == Some(receipt.attempt_id.as_str())
            && object.get("receipt_relative_path").and_then(Value::as_str)
                == Some(receipt.receipt_relative_path.as_str())
            && object.get("action").and_then(Value::as_str) == Some(action_name_v1(receipt.action))
    });
    if !planned_match {
        bail!("receipt is not present in the manifest planned_action_receipts");
    }
    Ok(())
}

fn ensure_index_matches_manifest(
    index: &ManagedActionReceiptIndexV1,
    manifest: &ManagedArtifactManifestV1,
) -> Result<()> {
    if index.authority_domain != manifest.authority_domain
        || index.scope_id != manifest.installation_id
        || index.manifest_generation != manifest.manifest_generation
        || index.manifest_sha256 != manifest.manifest_sha256
    {
        bail!("action receipt index does not match the manifest scope");
    }
    Ok(())
}

fn ensure_index_entry_matches_receipt(
    entry: &ManagedActionReceiptIndexEntryV1,
    receipt_path: &Path,
    receipt: &ManagedActionReceiptV1,
) -> Result<()> {
    let expected = build_receipt_index_entry(receipt_path, receipt)?;
    if entry != &expected {
        bail!("existing action receipt index entry differs from the canonical receipt artifact");
    }
    Ok(())
}

fn action_name_v1(action: ManagedActionV1) -> &'static str {
    match action {
        ManagedActionV1::Create => "create",
        ManagedActionV1::Replace => "replace",
        ManagedActionV1::Remove => "remove",
        ManagedActionV1::Restore => "restore",
        ManagedActionV1::Enable => "enable",
        ManagedActionV1::Disable => "disable",
        ManagedActionV1::Start => "start",
        ManagedActionV1::Stop => "stop",
    }
}

fn write_bytes_if_absent_or_equal(path: &Path, bytes: &[u8]) -> Result<()> {
    with_locked_target(path, || match fs::read(path) {
        Ok(existing) => {
            if existing == bytes {
                sync_parent_directory(path)?;
                return Ok(());
            }
            bail!("existing file differs from the requested canonical bytes");
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            write_bytes_atomically(path, bytes)
        }
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    })
}

fn write_bytes_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create parent directory {}", parent.display()))?;
    reject_existing_temp_files(parent, path)?;
    let (mut temp, temp_path) = create_unique_temp_file(parent, path)?;
    temp.write_all(bytes)
        .with_context(|| format!("write temporary file {}", temp_path.display()))?;
    temp.sync_all()
        .with_context(|| format!("sync temporary file {}", temp_path.display()))?;
    drop(temp);
    replace_path_atomically(&temp_path, path)?;
    sync_parent_directory(path)
}

fn reject_existing_temp_files(parent: &Path, destination: &Path) -> Result<()> {
    let label = destination
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "managed-lifecycle".to_string());
    let prefix = format!(".{label}.");
    for entry in fs::read_dir(parent)
        .with_context(|| format!("read temporary directory {}", parent.display()))?
    {
        let entry =
            entry.with_context(|| format!("read temporary entry in {}", parent.display()))?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if !file_name.starts_with(&prefix) || !file_name.ends_with(".tmp") {
            continue;
        }
        let temp_path = entry.path();
        let metadata = entry
            .metadata()
            .with_context(|| format!("read metadata for {}", temp_path.display()))?;
        if !metadata.is_file() {
            bail!("temporary lifecycle artifact is not a regular file");
        }
        bail!(
            "temporary lifecycle artifact residue blocks publication for {}: {}",
            destination.display(),
            temp_path.display()
        );
    }
    Ok(())
}

fn create_unique_temp_file(parent: &Path, destination: &Path) -> Result<(File, PathBuf)> {
    let label = destination
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "managed-lifecycle".to_string());
    for _ in 0..128 {
        let temp_path = parent.join(format!(".{label}.{:016x}.tmp", rand::random::<u64>()));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((file, temp_path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("create temporary file {}", temp_path.display()));
            }
        }
    }
    bail!(
        "failed to allocate a unique temporary file for {}",
        destination.display()
    );
}

fn with_locked_target<T>(path: &Path, operation: impl FnOnce() -> Result<T>) -> Result<T> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create parent directory {}", parent.display()))?;
    let lock_path = target_lock_path(path);
    let lock = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .with_context(|| format!("open lock file {}", lock_path.display()))?;
    lock.lock_exclusive()
        .with_context(|| format!("lock {}", lock_path.display()))?;
    let result = operation();
    drop(lock);
    result
}

fn target_lock_path(path: &Path) -> PathBuf {
    let parent = path.parent().expect("path parent already validated");
    let digest = lower_hex(&Sha256::digest(path.display().to_string().as_bytes()));
    parent.join(format!(".{}.lock", &digest[..16]))
}

fn head_matches_index(
    head: &ManagedManifestHeadV1,
    index: &ManagedActionReceiptIndexV1,
) -> Result<bool> {
    if head.scope_id != index.scope_id
        || head.manifest_generation != index.manifest_generation
        || head.manifest_sha256 != index.manifest_sha256
        || head.action_receipt_index_revision != index.index_revision
    {
        return Ok(false);
    }
    Ok(head.action_receipt_index_sha256
        == lower_hex(&Sha256::digest(canonical_action_receipt_index_bytes_v1(
            index,
        )?)))
}

#[cfg(windows)]
fn replace_path_atomically(from: &Path, to: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let from_wide: Vec<u16> = from
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let to_wide: Vec<u16> = to
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let flags = MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH;
    let result = unsafe { MoveFileExW(from_wide.as_ptr(), to_wide.as_ptr(), flags) };
    if result == 0 {
        return Err(std::io::Error::last_os_error()).with_context(|| {
            format!(
                "rename temporary file {} -> {}",
                from.display(),
                to.display()
            )
        });
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_path_atomically(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to).with_context(|| {
        format!(
            "rename temporary file {} -> {}",
            from.display(),
            to.display()
        )
    })
}

fn sync_parent_directory(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent directory", path.display()))?;
    #[cfg(unix)]
    {
        let directory = File::open(parent)
            .with_context(|| format!("open parent directory {}", parent.display()))?;
        directory
            .sync_all()
            .with_context(|| format!("sync parent directory {}", parent.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = parent;
    }
    Ok(())
}

fn manifest_digest_v1(manifest: &ManagedArtifactManifestV1) -> Result<String> {
    let mut value = serde_json::to_value(manifest).context("serialize manifest for digest")?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow!("manifest must serialize to an object"))?;
    object.remove("manifest_sha256");
    Ok(lower_hex(&Sha256::digest(canonical_json_value_to_vec(
        &value,
    )?)))
}

fn canonical_request_sha256(request: &ManagedLifecyclePublisherRequestV1) -> Result<String> {
    Ok(lower_hex(&Sha256::digest(
        canonical_json_to_vec(request).context("encode publisher request")?,
    )))
}

fn canonical_json_to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value).context("serialize canonical JSON value")?;
    canonical_json_value_to_vec(&value)
}

fn canonical_json_value_to_vec(value: &Value) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    encode_json_value(value, &mut output)?;
    Ok(output)
}

fn encode_json_value(value: &Value, output: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Null => output.extend_from_slice(b"null"),
        Value::Bool(true) => output.extend_from_slice(b"true"),
        Value::Bool(false) => output.extend_from_slice(b"false"),
        Value::Number(number) => {
            if number.is_f64() {
                bail!("canonical JSON rejects floating-point values");
            }
            output.extend_from_slice(number.to_string().as_bytes());
        }
        Value::String(string) => {
            output.extend_from_slice(
                serde_json::to_string(string)
                    .context("encode canonical string")?
                    .as_bytes(),
            );
        }
        Value::Array(array) => {
            output.push(b'[');
            for (index, item) in array.iter().enumerate() {
                if index > 0 {
                    output.push(b',');
                }
                encode_json_value(item, output)?;
            }
            output.push(b']');
        }
        Value::Object(object) => {
            output.push(b'{');
            let mut first = true;
            let sorted: std::collections::BTreeMap<_, _> = object.iter().collect();
            for (key, item) in sorted {
                if !first {
                    output.push(b',');
                }
                first = false;
                output.extend_from_slice(
                    serde_json::to_string(key)
                        .context("encode canonical object key")?
                        .as_bytes(),
                );
                output.push(b':');
                encode_json_value(item, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use substrate_common::{
        managed_action_prepared_record_sha256_v1, sign_lifecycle_anchor_for_test_v1,
        sign_managed_action_prepared_record_for_test_v1, sign_managed_action_receipt_for_test_v1,
        LifecyclePublisherProtectedStateV1, LifecycleSignatureV1, ManagedActionPreparedRecordV1,
        ManagedExecutorIdentityV1,
    };
    use substrate_common::{ManagedArtifactIdentityV1, ManagedArtifactRoleV1};
    use tempfile::tempdir;

    fn sample_manifest() -> ManagedArtifactManifestV1 {
        let mut manifest = ManagedArtifactManifestV1 {
            schema_owner: "substrate.managed-artifact-manifest".to_string(),
            schema_version: 1,
            manifest_id: "m1:018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90:1".to_string(),
            manifest_sha256: String::new(),
            host_context_commitment: "1".repeat(64),
            selected_host_prefix: "/tmp/substrate".to_string(),
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
                logical_role: ManagedArtifactRoleV1(
                    "unix.prefix.projection(config.yaml)".to_string(),
                ),
                object_type: "regular-file".to_string(),
                identity: ManagedArtifactIdentityV1 {
                    scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
                    parent_identity: "/tmp/substrate".to_string(),
                    name_identity: "config.yaml".to_string(),
                    physical_identity: "/tmp/substrate/config.yaml".to_string(),
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
        manifest.manifest_sha256 = manifest_digest_v1(&manifest).unwrap();
        manifest
    }

    fn sample_executor() -> ManagedExecutorIdentityV1 {
        ManagedExecutorIdentityV1 {
            source_commit: "a".repeat(40),
            source_tree: "b".repeat(40),
            source_ref: "refs/heads/test".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            artifact_sha256: "c".repeat(64),
            artifact_path: "/tmp/substrate-lifecycle-linux".to_string(),
            toolchain: Some("rustc 1.89.0".to_string()),
            code_identity: None,
        }
    }

    fn sample_prepared_record(
        manifest: &ManagedArtifactManifestV1,
    ) -> ManagedActionPreparedRecordV1 {
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
            before_observation: Value::Object(Map::new()),
            executor_identity: sample_executor(),
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
        let mut anchor = substrate_common::LifecyclePublisherAnchorV1 {
            schema_owner: "substrate.lifecycle-publisher-anchor".to_string(),
            schema_version: 1,
            authority_domain: manifest.authority_domain.clone(),
            host_context_commitment: manifest.host_context_commitment.clone(),
            platform_mapping_commitment: manifest.platform_mapping_commitment.clone(),
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
            executor_identity: prepared_record.executor_identity.clone(),
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

    fn signed_receipt(
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
            prepared_record_sha256: managed_action_prepared_record_sha256_v1(prepared_record)
                .unwrap(),
            allocated_counter: prepared_record.allocated_counter,
            request_sha256: prepared_record.request_sha256.clone(),
            pre_observation: Value::Object(Map::new()),
            effect_observation: Value::Object(Map::new()),
            post_observation: Value::Object(Map::new()),
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

    fn sample_shared_claims(
        manifest: &ManagedArtifactManifestV1,
        manifest_head_sha256: String,
    ) -> ManagedSharedClaimsV1 {
        ManagedSharedClaimsV1 {
            schema_owner: "substrate.managed-shared-claims".to_string(),
            schema_version: 1,
            manifest_generation: manifest.manifest_generation,
            manifest_head_sha256,
            claims: vec![substrate_common::ManagedSharedClaimV1 {
                claim_id: "claim-1".to_string(),
                logical_role: ManagedArtifactRoleV1(
                    "unix.prefix.projection(config.yaml)".to_string(),
                ),
                compatible_desired_state_sha256: "2".repeat(64),
                before_state: Value::Object(Map::new()),
                installation_id: manifest.installation_id.clone(),
                host_context_commitment: manifest.host_context_commitment.clone(),
                manifest_sha256: manifest.manifest_sha256.clone(),
                lifecycle_state: ManagedLifecycleStateV1::ManifestDurable,
            }],
        }
    }

    #[test]
    fn publish_and_load_manifest_round_trip() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        let canonical = publish_manifest_v1(temp.path(), &manifest).expect("publish manifest");
        assert_eq!(canonical.manifest_sha256, manifest.manifest_sha256);
        let loaded = load_manifest_v1(temp.path(), 1).expect("load manifest");
        assert_eq!(loaded, manifest);
    }

    #[test]
    fn publish_manifest_publishes_initial_head() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        publish_manifest_v1(temp.path(), &manifest).unwrap();
        let head = load_required_head(temp.path()).expect("manifest head");
        assert_eq!(head.action_receipt_index_revision, 0);
        assert_eq!(head.manifest_sha256, manifest.manifest_sha256);
    }

    #[test]
    fn transition_manifest_recomputes_digest() {
        let manifest = sample_manifest();
        let next = transition_manifest_v1(&manifest, ManagedLifecycleStateV1::Committed)
            .expect("transition manifest");
        assert_eq!(next.lifecycle_state, ManagedLifecycleStateV1::Committed);
        assert_ne!(next.manifest_sha256, manifest.manifest_sha256);
    }

    #[test]
    fn resume_action_receipt_commit_publishes_receipt_index_and_head() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        publish_manifest_v1(temp.path(), &manifest).unwrap();
        let prepared_record = sample_prepared_record(&manifest);
        let protected_state = sample_protected_state(&manifest, &prepared_record);
        let receipt = signed_receipt(&manifest, &prepared_record);
        let head =
            resume_action_receipt_commit_v1(temp.path(), &manifest, &protected_state, &receipt)
                .expect("resume receipt commit");
        assert_eq!(head.action_receipt_index_revision, 1);
        let receipt_path = temp.path().join(&receipt.receipt_relative_path);
        assert!(receipt_path.exists(), "receipt file must be published");
        let index_path = action_receipt_index_path(temp.path(), manifest.manifest_generation);
        assert!(
            index_path.exists(),
            "action receipt index must be published"
        );
        let head_path = head_file_path(temp.path());
        assert!(head_path.exists(), "head must be published");
    }

    #[test]
    fn publish_manifest_accepts_existing_receipt_index_for_same_manifest() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        publish_manifest_v1(temp.path(), &manifest).unwrap();
        let prepared_record = sample_prepared_record(&manifest);
        let protected_state = sample_protected_state(&manifest, &prepared_record);
        let receipt = signed_receipt(&manifest, &prepared_record);
        resume_action_receipt_commit_v1(temp.path(), &manifest, &protected_state, &receipt)
            .unwrap();
        publish_manifest_v1(temp.path(), &manifest).expect("idempotent manifest publish");
    }

    #[test]
    fn resume_action_receipt_commit_is_idempotent_after_head_publication() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        publish_manifest_v1(temp.path(), &manifest).unwrap();
        let prepared_record = sample_prepared_record(&manifest);
        let protected_state = sample_protected_state(&manifest, &prepared_record);
        let receipt = signed_receipt(&manifest, &prepared_record);
        let first =
            resume_action_receipt_commit_v1(temp.path(), &manifest, &protected_state, &receipt)
                .unwrap();
        let second =
            resume_action_receipt_commit_v1(temp.path(), &manifest, &protected_state, &receipt)
                .unwrap();
        assert_eq!(second, first);
    }

    #[test]
    fn resume_action_receipt_commit_requires_existing_manifest_head() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        publish_manifest_v1(temp.path(), &manifest).unwrap();
        fs::remove_file(head_file_path(temp.path())).unwrap();
        let prepared_record = sample_prepared_record(&manifest);
        let protected_state = sample_protected_state(&manifest, &prepared_record);
        let receipt = signed_receipt(&manifest, &prepared_record);
        let error =
            resume_action_receipt_commit_v1(temp.path(), &manifest, &protected_state, &receipt)
                .expect_err("missing head must fail");
        assert!(error.to_string().contains("manifest head is missing"));
    }

    #[test]
    fn staged_head_temp_residue_requires_manual_recovery() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        let empty_index = empty_action_receipt_index_for_manifest(&manifest);
        let next = initial_head_for_manifest(&manifest, &empty_index).unwrap();
        let head_bytes = canonical_json_to_vec(&next).unwrap();
        let head_path = head_file_path(temp.path());
        let temp_path = head_path
            .parent()
            .unwrap()
            .join(".head.v1.json.deadbeefdeadbeef.tmp");
        fs::create_dir_all(head_path.parent().unwrap()).unwrap();
        fs::write(&temp_path, head_bytes).unwrap();
        let error = compare_and_swap_head_v1(temp.path(), None, &next)
            .expect_err("temp residue must fail closed");
        assert!(error
            .to_string()
            .contains("temporary lifecycle artifact residue"));
        assert!(!head_path.exists());
        assert!(temp_path.exists());
    }

    #[test]
    fn update_shared_claims_requires_matching_expected_state() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        publish_manifest_v1(temp.path(), &manifest).unwrap();
        let head = load_required_head(temp.path()).unwrap();
        let head_sha256 = lower_hex(&Sha256::digest(canonical_json_to_vec(&head).unwrap()));
        let current = sample_shared_claims(&manifest, head_sha256.clone());
        let mut next = current.clone();
        next.claims[0].compatible_desired_state_sha256 = "3".repeat(64);

        update_shared_claims_v1(temp.path(), None, &current).expect("create shared claims");
        update_shared_claims_v1(temp.path(), Some(&current), &next).expect("advance shared claims");
        let exact_retry = update_shared_claims_v1(temp.path(), Some(&current), &next)
            .expect("exact retry joins current claims");
        assert_eq!(exact_retry, next);

        let error = update_shared_claims_v1(temp.path(), Some(&current), &current)
            .expect_err("stale expected claims must fail");
        assert!(error
            .to_string()
            .contains("shared claims compare-and-swap mismatch"));
    }

    #[test]
    fn update_shared_claims_requires_current_head_digest() {
        let temp = tempdir().unwrap();
        let manifest = sample_manifest();
        publish_manifest_v1(temp.path(), &manifest).unwrap();
        let mut claims = sample_shared_claims(&manifest, "3".repeat(64));
        let error = update_shared_claims_v1(temp.path(), None, &claims)
            .expect_err("wrong manifest head digest must fail");
        assert!(error
            .to_string()
            .contains("manifest_head_sha256 does not match"));

        let head = load_required_head(temp.path()).unwrap();
        let head_sha256 = lower_hex(&Sha256::digest(canonical_json_to_vec(&head).unwrap()));
        claims.manifest_head_sha256 = head_sha256;
        update_shared_claims_v1(temp.path(), None, &claims).expect("matching head digest succeeds");
    }

    #[test]
    fn provider_stub_client_reports_unavailable() {
        let manifest = sample_manifest();
        let request = ManagedLifecyclePublisherRequestV1 {
            host_context_commitment: manifest.host_context_commitment.clone(),
            platform_mapping_commitment: None,
            scope_id: manifest.installation_id.clone(),
            current_anchor_counter: 0,
            current_anchor_sha256: "1".repeat(64),
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
            expected_executor_build: sample_executor(),
        };
        let client = open_publisher_bootstrap_channel_v1("linux_system").unwrap();
        let error = client.submit_publisher_request_v1(&request).unwrap_err();
        assert!(error.to_string().contains(PROVIDER_UNAVAILABLE_KIND_V1));
    }

    #[test]
    fn validate_publisher_response_requires_matching_request_digest() {
        let request = ManagedLifecyclePublisherRequestV1 {
            host_context_commitment: "1".repeat(64),
            platform_mapping_commitment: None,
            scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
            current_anchor_counter: 0,
            current_anchor_sha256: "2".repeat(64),
            manifest_generation: 1,
            manifest_sha256: "3".repeat(64),
            role: ManagedArtifactRoleV1("unix.prefix.projection(config.yaml)".to_string()),
            action: ManagedActionV1::Create,
            object_identity: ManagedArtifactIdentityV1 {
                scope_id: "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90".to_string(),
                parent_identity: "/tmp/substrate".to_string(),
                name_identity: "config.yaml".to_string(),
                physical_identity: "/tmp/substrate/config.yaml".to_string(),
                metadata: None,
            },
            requester_principal: "alice:1000".to_string(),
            attempt_nonce: "nonce-1".to_string(),
            expected_executor_build: sample_executor(),
        };
        let response = serde_json::to_vec(&json!({"request_sha256": "0".repeat(64)})).unwrap();
        assert!(validate_publisher_response_v1(&response, &request).is_err());
    }
}
