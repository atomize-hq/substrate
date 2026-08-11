use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[cfg(target_os = "macos")]
use std::fs::OpenOptions;
#[cfg(target_os = "macos")]
use std::io::Read as _;
#[cfg(target_os = "macos")]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
#[cfg(target_os = "macos")]
use std::path::Path;
#[cfg(target_os = "macos")]
use std::process::Command;
use substrate_common::{
    canonical_mac_publisher_install_provenance_v1, mac_publisher_control_requirement_cdhash_v1,
    validate_mac_publisher_install_provenance_v1, validate_publisher_bootstrap_authorization_v1,
    ExecutorBuildEvidenceV1, GuestPublisherPairingTicketV1, MacPublisherBootstrapRequestV1,
    MacPublisherInstallProvenanceV1, ManagedArtifactEntryV1, ManagedArtifactManifestV1,
    ManagedLifecyclePublisherRequestV1, ManagedLifecycleStateV1, PublisherBootstrapAuthorizationV1,
    PublisherBootstrapComponentRoleV1, PublisherBootstrapComponentV1,
};
use transport_api_types::InstallBootstrapContextCarrierV1;

use super::{ManagedLifecycleControlRequestV1, MappedLifecycleTagV1};
const MAC_MACH_SERVICE_V1: &str = "com.substrate.lifecycle.publisher.v1";
const MAX_MAC_XPC_FRAME_BYTES_V1: usize = 1024 * 1024;
#[cfg(target_os = "macos")]
const MAC_BOOTSTRAP_PROVENANCE_PATH_V1: &str =
    "/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json";
#[cfg(target_os = "macos")]
const MAC_BOOTSTRAP_FD3_MAX_BYTES_V1: usize = 1024 * 1024;
#[cfg(target_os = "macos")]
const MAC_BOOTSTRAP_FD3_REQUEST_INGRESS_TIMEOUT_V1: std::time::Duration =
    std::time::Duration::from_secs(5);
#[cfg(target_os = "macos")]
const MAC_BOOTSTRAP_FD3_RESPONSE_PRODUCTION_TIMEOUT_V1: std::time::Duration =
    std::time::Duration::from_secs(30);
#[cfg(target_os = "macos")]
const MAC_BOOTSTRAP_FD3_RESPONSE_TRANSFER_TIMEOUT_V1: std::time::Duration =
    std::time::Duration::from_secs(5);
#[cfg(target_os = "macos")]
const MAC_BOOTSTRAP_FD3_CHILD_EXIT_TIMEOUT_V1: std::time::Duration =
    std::time::Duration::from_secs(5);

pub fn bootstrap_publisher_v1(_authorization: &PublisherBootstrapAuthorizationV1) -> Result<Value> {
    bail!(
        "macOS publisher bootstrap is available only through the retained direct-interactive FD3 channel"
    )
}

pub fn submit_publisher_request_v1(_request: &ManagedLifecyclePublisherRequestV1) -> Result<Value> {
    bail!("raw macOS publisher requests are unavailable; use one closed mapped lifecycle branch")
}

/// Submit the sole absent-instance Stage-1 branch.  The control decoder has already rejected a
/// publisher request, forwarding authority, and every non-Stage-1 tag.
pub fn submit_stage_one_absent_instance_create_v1(
    request: &ManagedLifecycleControlRequestV1,
) -> Result<Value> {
    let bytes = serde_json::to_vec(request).context("encode macOS Stage-1 mapped request")?;
    let response = open_mac_xpc_channel_v1("stage-one-create", &bytes)?;
    attest_mac_publisher_response_v1(&response)?;
    serde_json::from_slice(&response).context("decode macOS Stage-1 mapped response")
}

/// Submit one canonical post-PM role/action request.  The fixed operation is not caller chosen.
pub fn submit_post_pm_managed_action_v1(
    request: &ManagedLifecycleControlRequestV1,
) -> Result<Value> {
    let bytes = serde_json::to_vec(request).context("encode macOS post-PM mapped request")?;
    let response = open_mac_xpc_channel_v1("post-pm-action", &bytes)?;
    attest_mac_publisher_response_v1(&response)?;
    serde_json::from_slice(&response).context("decode macOS post-PM mapped response")
}

/// Relay the only R6 ticket/frame-bearing child through its fixed XPC operation.
pub fn submit_guest_pairing_data_session_v1(
    request: &ManagedLifecycleControlRequestV1,
) -> Result<Value> {
    submit_closed_guest_pairing_session_v1(
        request,
        MappedLifecycleTagV1::GuestPairingDataSession,
        "guest-pairing-data-session",
    )
}

fn submit_closed_guest_pairing_session_v1(
    request: &ManagedLifecycleControlRequestV1,
    expected_tag: MappedLifecycleTagV1,
    operation: &str,
) -> Result<Value> {
    if request.tag.as_ref() != Some(&expected_tag) {
        bail!("R6 fixed XPC pairing relay received the wrong protocol tag");
    }
    let bytes = serde_json::to_vec(request).context("encode closed R6 macOS pairing request")?;
    let response = open_mac_xpc_channel_v1(operation, &bytes)?;
    attest_mac_publisher_response_v1(&response)?;
    serde_json::from_slice(&response).context("decode closed R6 macOS pairing response")
}

/// Derive the one nonserialized bootstrap authorization after terminal confirmation.
///
/// The seed has already passed the direct command's closed decoder. It can name no authority of
/// its own: this function independently measures the control and fixed executor images, requires
/// their physical/code identities to join, and derives the complete component map from the
/// canonical manifest in memory.
#[cfg(target_os = "macos")]
pub fn derive_retained_publisher_bootstrap_authorization_v1(
    request: &MacPublisherBootstrapRequestV1,
) -> Result<PublisherBootstrapAuthorizationV1> {
    // The hidden request contains an IH carrier and nothing else. Source, build, image, manifest,
    // scope, attempt, expiry, control authority, and profile provenance are all derived from the
    // installer-retained record plus secure local allocation below.
    let provenance = load_retained_mac_publisher_bootstrap_provenance_v1()?;
    let carrier = InstallBootstrapContextCarrierV1::decode(&request.install_bootstrap_context_v1)
        .context("decode direct bootstrap exact IH carrier")?;
    if carrier.host_context_commitment != provenance.host_context_commitment
        || carrier.context.selected_host_prefix != provenance.selected_host_prefix
    {
        bail!("direct bootstrap IH carrier does not exact-join install provenance");
    }
    let control_path =
        std::env::current_exe().context("resolve current lifecycle-control image")?;
    let control = measure_bootstrap_image_v1(&control_path)
        .context("measure current substrate-lifecycle-control image")?;
    let executor = measure_bootstrap_image_v1(Path::new(
        "/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1",
    ))
    .context("measure fixed substrate-lifecycle-macos image")?;
    if control.artifact_sha256 == executor.artifact_sha256
        || control.artifact_identity == executor.artifact_identity
        || control.cdhash == executor.cdhash
    {
        bail!("control and executor artifacts must be distinct independently measured images");
    }
    let control_authority = provenance.control_authority.clone();
    if control_authority.target_triple != provenance.executor_image.target_triple
        || provenance.control_image.artifact_sha256 != control.artifact_sha256
        || provenance.control_image.physical_identity != control.artifact_identity
        || provenance.control_image.code_identity != control.cdhash
        || provenance.executor_image.artifact_sha256 != executor.artifact_sha256
        || provenance.executor_image.physical_identity != executor.artifact_identity
        || provenance.executor_image.code_identity != executor.cdhash
    {
        bail!("measured direct bootstrap images do not exact-join retained install provenance");
    }
    let evidence = ExecutorBuildEvidenceV1 {
        schema_owner: "substrate.executor-build-evidence".to_string(),
        schema_version: 1,
        source_commit: provenance.source_commit.clone(),
        source_tree: provenance.source_tree.clone(),
        source_ref: provenance.source_ref.clone(),
        artifact_sha256: executor.artifact_sha256.clone(),
        artifact_identity: executor.artifact_identity.clone(),
        target_triple: provenance.executor_image.target_triple.clone(),
        tool_versions: Default::default(),
        code_identity: Some(executor.cdhash.clone()),
    };
    let scope_id = uuid::Uuid::now_v7().to_string();
    let attempt_id = uuid::Uuid::now_v7().to_string();
    let now: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("read direct bootstrap time")?
        .as_nanos()
        .try_into()
        .map_err(|_| anyhow!("direct bootstrap time is outside u64 nanoseconds"))?;
    let expires_at_unix_ns = now
        .checked_add(300_000_000_000)
        .ok_or_else(|| anyhow!("direct bootstrap expiry overflow"))?;
    let mut manifest = ManagedArtifactManifestV1 {
        schema_owner: "substrate.managed-artifact-manifest".to_string(),
        schema_version: 1,
        manifest_id: format!("m1:{scope_id}:1"),
        manifest_sha256: String::new(),
        host_context_commitment: carrier.host_context_commitment.clone(),
        selected_host_prefix: carrier.context.selected_host_prefix.clone(),
        intended_principal: match &carrier.context.intended_host_principal {
            transport_api_types::PlatformPrincipalV1::Unix { account, .. } => account.clone(),
            _ => bail!("direct macOS bootstrap requires a UNIX IH principal"),
        },
        platform_kind: "mac_lima".to_string(),
        platform_mapping_commitment: None,
        authority_domain: "mac_host_shared".to_string(),
        installation_id: scope_id.clone(),
        attempt_id: attempt_id.clone(),
        manifest_generation: 1,
        created_at_unix_ns: now,
        lifecycle_state: ManagedLifecycleStateV1::ManifestDurable,
        previous_manifest_sha256: None,
        entries: Vec::new(),
        planned_action_receipts: Vec::new(),
    };
    manifest.manifest_sha256 = substrate_common::managed_artifact_manifest_sha256_v1(&manifest)?;
    let authorization = PublisherBootstrapAuthorizationV1 {
        schema_owner: "substrate.publisher-bootstrap-authorization".to_string(),
        schema_version: 1,
        authority_domain: "mac_host_shared".to_string(),
        scope_id: scope_id.clone(),
        host_context_commitment: carrier.host_context_commitment.clone(),
        install_bootstrap_context_v1: request.install_bootstrap_context_v1.clone(),
        platform_mapping_commitment: None,
        requester_principal: manifest.intended_principal.clone(),
        source_commit: provenance.source_commit.clone(),
        source_tree: provenance.source_tree.clone(),
        source_ref: provenance.source_ref.clone(),
        manifest_generation: 1,
        manifest_sha256: manifest.manifest_sha256.clone(),
        pre_pm_manifest: Some(manifest.clone()),
        issued_at_unix_ns: now,
        expires_at_unix_ns,
        attempt_nonce: attempt_id.clone(),
        publisher_expected_absent: true,
        executor_build_evidence: evidence.clone(),
        mac_control_authority: Some(control_authority),
        components: derive_exact_mac_bootstrap_components_v1(
            &manifest.entries,
            &scope_id,
            &manifest.attempt_id,
            &evidence,
        )?,
        test_retirement_commitment: None,
        confirmation: "CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER".to_string(),
    };
    validate_publisher_bootstrap_authorization_v1(&authorization)
        .context("validate independently derived direct bootstrap authorization")?;
    Ok(authorization)
}

/// Open the only installer-owned bootstrap provenance record through no-follow descriptors and
/// reject a substituted parent, file, or noncanonical record.  The fixed path is deliberately
/// not configurable by stdin, argv, environment, CWD, or repository state.
#[cfg(target_os = "macos")]
fn load_retained_mac_publisher_bootstrap_provenance_v1() -> Result<MacPublisherInstallProvenanceV1>
{
    let path = Path::new(MAC_BOOTSTRAP_PROVENANCE_PATH_V1);
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("fixed bootstrap provenance has no parent"))?;
    let parent_metadata = std::fs::symlink_metadata(parent)
        .with_context(|| format!("inspect bootstrap provenance parent {}", parent.display()))?;
    if !parent_metadata.is_dir()
        || parent_metadata.uid() != 0
        || parent_metadata.gid() != 0
        || parent_metadata.mode() & 0o022 != 0
    {
        bail!("bootstrap provenance parent is not retained root-owned no-write identity");
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("open fixed bootstrap provenance {}", path.display()))?;
    let metadata = file
        .metadata()
        .context("inspect fixed bootstrap provenance")?;
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.uid() != 0
        || metadata.gid() != 0
        || metadata.mode() & 0o777 != 0o444
    {
        bail!("bootstrap provenance is not the exact root:wheel 0444 regular file");
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .context("read retained bootstrap provenance")?;
    let path_after =
        std::fs::symlink_metadata(path).context("reinspect fixed bootstrap provenance path")?;
    if path_after.dev() != metadata.dev() || path_after.ino() != metadata.ino() {
        bail!("bootstrap provenance identity changed during retained read");
    }
    let provenance: MacPublisherInstallProvenanceV1 =
        serde_json::from_slice(&bytes).context("decode retained bootstrap provenance")?;
    validate_mac_publisher_install_provenance_v1(&provenance)?;
    if canonical_mac_publisher_install_provenance_v1(&provenance)? != bytes {
        bail!("retained bootstrap provenance is not canonical");
    }
    Ok(provenance)
}

#[cfg(target_os = "macos")]
struct MeasuredBootstrapImageV1 {
    artifact_sha256: String,
    artifact_identity: String,
    cdhash: String,
}

/// Measure a fixed image through a no-follow file descriptor before comparing it to build
/// evidence. `codesign` is used only to query the CodeDirectory hash of that already-opened
/// immutable image path; no signing or mutation operation is attempted.
#[cfg(target_os = "macos")]
fn measure_bootstrap_image_v1(path: &Path) -> Result<MeasuredBootstrapImageV1> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(path)
        .with_context(|| format!("open no-follow bootstrap image {}", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("stat bootstrap image {}", path.display()))?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        bail!("bootstrap image must be a single-link regular file");
    }
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("hash bootstrap image {}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    let cdhash = measure_codesign_cdhash_v1(path)?;
    let retained_after = file
        .metadata()
        .with_context(|| format!("reinspect retained bootstrap image {}", path.display()))?;
    let path_after = std::fs::metadata(path)
        .with_context(|| format!("reinspect bootstrap image path {}", path.display()))?;
    if retained_after.dev() != metadata.dev()
        || retained_after.ino() != metadata.ino()
        || path_after.dev() != metadata.dev()
        || path_after.ino() != metadata.ino()
    {
        bail!("bootstrap image identity changed during retained hash/code measurement");
    }
    Ok(MeasuredBootstrapImageV1 {
        artifact_sha256: lower_hex_v1(&digest.finalize()),
        artifact_identity: format!("dev:{}:ino:{}", metadata.dev(), metadata.ino()),
        cdhash,
    })
}

#[cfg(target_os = "macos")]
fn measure_codesign_cdhash_v1(path: &Path) -> Result<String> {
    let output = Command::new("/usr/bin/codesign")
        .arg("-d")
        .arg("-v")
        .arg("-v")
        .arg("-v")
        .arg(path)
        .env_clear()
        .output()
        .with_context(|| format!("query CodeDirectory hash for {}", path.display()))?;
    if !output.status.success() {
        bail!("codesign cannot attest fixed bootstrap image identity");
    }
    let mut text = String::from_utf8(output.stdout).context("decode codesign stdout")?;
    text.push_str(&String::from_utf8(output.stderr).context("decode codesign stderr")?);
    let mut hashes = text
        .lines()
        .filter_map(|line| line.trim().strip_prefix("CDHash="))
        .map(|hash| hash.to_ascii_lowercase());
    let hash = hashes
        .next()
        .ok_or_else(|| anyhow!("codesign output has no CodeDirectory hash"))?;
    if hashes.next().is_some()
        || hash.len() != 40
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        bail!("codesign output has no exact CodeDirectory hash");
    }
    Ok(format!("cdhash:{hash}"))
}

#[cfg(target_os = "macos")]
fn derive_exact_mac_bootstrap_components_v1(
    entries: &[ManagedArtifactEntryV1],
    scope_id: &str,
    attempt_id: &str,
    evidence: &ExecutorBuildEvidenceV1,
) -> Result<Vec<PublisherBootstrapComponentV1>> {
    const ROLES: [(&str, &str); 7] = [
        ("mac.publisher.executor", "MacExecutor"),
        ("mac.publisher.plist", "MacPlist"),
        ("mac.publisher.mach-service", "MacMachService"),
        ("mac.publisher.signing-key", "MacSigningKey"),
        ("mac.publisher.current-anchor", "MacProtectedState"),
        ("mac.publisher.bootstrap-intent", "MacBootstrapIntent"),
        ("mac.publisher.service-state", "MacServiceState"),
    ];
    if entries.is_empty() {
        let ids: BTreeMap<_, _> = ROLES
            .iter()
            .map(|(_, component_role)| {
                (
                    *component_role,
                    deterministic_bootstrap_component_id_v1(scope_id, attempt_id, component_role),
                )
            })
            .collect();
        let dependencies = |component_role: &str| -> Vec<String> {
            match component_role {
                "MacPlist" => vec![ids["MacExecutor"].clone()],
                "MacMachService" => vec![ids["MacPlist"].clone()],
                "MacProtectedState" => vec![ids["MacSigningKey"].clone()],
                "MacServiceState" => vec![ids["MacMachService"].clone()],
                _ => Vec::new(),
            }
        };
        return ROLES
            .iter()
            .map(|(managed_role, component_role)| {
                let role = PublisherBootstrapComponentRoleV1((*component_role).to_string());
                Ok(PublisherBootstrapComponentV1 {
                    component_id: ids[*component_role].clone(),
                    role: role.clone(),
                    target_identity:
                        substrate_common::derive_publisher_bootstrap_component_target_v1(
                            &role, scope_id,
                        )?,
                    object_type: match *component_role {
                        "MacExecutor" | "MacPlist" => "regular-file".to_string(),
                        "MacMachService" => "mach-service".to_string(),
                        "MacSigningKey" => "keychain-key".to_string(),
                        "MacProtectedState" | "MacBootstrapIntent" => "keychain-record".to_string(),
                        "MacServiceState" => "service-state".to_string(),
                        _ => unreachable!("closed MAC component set"),
                    },
                    expected_before: json!({"kind": "ExactAbsentCreate"}),
                    source_artifact_sha256: (*managed_role == "mac.publisher.executor")
                        .then(|| evidence.artifact_sha256.clone()),
                    source_signature_sha256: None,
                    owner: None,
                    group_name: None,
                    mode: None,
                    acl_or_security: None,
                    code_requirement: (*managed_role == "mac.publisher.executor")
                        .then(|| evidence.code_identity.clone())
                        .flatten(),
                    dependency_component_ids: dependencies(component_role),
                    durability_method: match *component_role {
                        "MacSigningKey" | "MacProtectedState" | "MacBootstrapIntent" => {
                            "keychain_cas".to_string()
                        }
                        "MacServiceState" => "launchd_readback".to_string(),
                        _ => "descriptor_readback".to_string(),
                    },
                })
            })
            .collect();
    }
    if entries.len() != ROLES.len() {
        bail!("direct bootstrap manifest does not contain the exhaustive MAC component set");
    }
    let by_role: BTreeMap<_, _> = entries
        .iter()
        .map(|entry| (entry.logical_role.0.as_str(), entry))
        .collect();
    if by_role.len() != ROLES.len()
        || by_role
            .keys()
            .any(|role| !ROLES.iter().any(|(expected, _)| role == expected))
    {
        bail!("direct bootstrap manifest has an unlisted or duplicate MAC component role");
    }
    let ids: BTreeMap<_, _> = ROLES
        .iter()
        .map(|(role, _)| (*role, by_role[role].object_id.as_str()))
        .collect();
    let dependencies = |role: &str| -> Vec<String> {
        match role {
            "mac.publisher.plist" => vec![ids["mac.publisher.executor"].to_string()],
            "mac.publisher.mach-service" => vec![ids["mac.publisher.plist"].to_string()],
            "mac.publisher.current-anchor" => vec![ids["mac.publisher.signing-key"].to_string()],
            "mac.publisher.service-state" => vec![ids["mac.publisher.mach-service"].to_string()],
            _ => Vec::new(),
        }
    };
    ROLES
        .iter()
        .map(|(managed_role, component_role)| {
            let entry = by_role[managed_role];
            let role = PublisherBootstrapComponentRoleV1((*component_role).to_string());
            let target_identity =
                substrate_common::derive_publisher_bootstrap_component_target_v1(&role, scope_id)?;
            Ok(PublisherBootstrapComponentV1 {
                component_id: entry.object_id.clone(),
                role,
                target_identity,
                object_type: entry.object_type.clone(),
                expected_before: entry.before_state.clone(),
                source_artifact_sha256: (*managed_role == "mac.publisher.executor")
                    .then(|| evidence.artifact_sha256.clone()),
                source_signature_sha256: None,
                owner: entry.owner.clone(),
                group_name: entry.group_name.clone(),
                mode: entry.mode.clone(),
                acl_or_security: entry.acl_or_security.clone(),
                code_requirement: (*managed_role == "mac.publisher.executor")
                    .then(|| evidence.code_identity.clone())
                    .flatten(),
                dependency_component_ids: dependencies(managed_role),
                durability_method: match *component_role {
                    "MacSigningKey" | "MacProtectedState" | "MacBootstrapIntent" => {
                        "keychain_cas".to_string()
                    }
                    "MacServiceState" => "launchd_readback".to_string(),
                    _ => "descriptor_readback".to_string(),
                },
            })
        })
        .collect()
}

#[cfg(target_os = "macos")]
fn deterministic_bootstrap_component_id_v1(scope_id: &str, attempt_id: &str, role: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"SUBSTRATE-R5-MAC-BOOTSTRAP-COMPONENT-V1\0");
    digest.update(scope_id.as_bytes());
    digest.update([0]);
    digest.update(attempt_id.as_bytes());
    digest.update([0]);
    digest.update(role.as_bytes());
    let mut bytes: [u8; 16] = digest.finalize()[..16]
        .try_into()
        .expect("SHA-256 prefix has sixteen bytes");
    bytes[6] = (bytes[6] & 0x0f) | 0x70;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

#[cfg(target_os = "macos")]
fn lower_hex_v1(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

/// Create the Darwin AF_UNIX stream channel, retain one endpoint, pass only the peer as FD3 to the
/// exact elevated executor, and exchange one bounded EOF-delimited canonical document each way.
#[cfg(target_os = "macos")]
pub fn send_publisher_bootstrap_authorization_fd3_v1(
    authorization: &PublisherBootstrapAuthorizationV1,
) -> Result<Value> {
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
    use std::os::unix::process::CommandExt;
    use std::process::Command;
    use substrate_common::canonical_publisher_bootstrap_authorization_v1;

    initialize_retained_bootstrap_milestone_logger_v1();
    let milestones_started = std::time::Instant::now();
    emit_retained_bootstrap_milestone_v1(milestones_started, "client.start");
    let request = canonical_publisher_bootstrap_authorization_v1(authorization)
        .context("canonicalize direct bootstrap authorization")?;
    if request.is_empty() || request.len() > MAC_BOOTSTRAP_FD3_MAX_BYTES_V1 {
        bail!("direct bootstrap authorization has an invalid frame size");
    }

    let mut pair = [-1; 2];
    // SAFETY: socketpair initializes both entries on success. Ownership transfers immediately
    // to OwnedFd below, before any later fallible operation.
    if unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, pair.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("create bootstrap stream pair");
    }
    // SAFETY: socketpair returned two distinct, live descriptors owned only by this function.
    let retained = unsafe { OwnedFd::from_raw_fd(pair[0]) };
    // SAFETY: ownership of the second live socketpair descriptor is likewise unique.
    let peer = unsafe { OwnedFd::from_raw_fd(pair[1]) };
    set_retained_bootstrap_fd_cloexec_v1(retained.as_raw_fd())?;
    set_retained_bootstrap_fd_cloexec_v1(peer.as_raw_fd())?;
    set_retained_bootstrap_no_sigpipe_v1(retained.as_raw_fd())?;
    set_retained_bootstrap_no_sigpipe_v1(peer.as_raw_fd())?;

    let retained_fd = retained.as_raw_fd();
    let peer_fd = peer.as_raw_fd();
    let mut command = Command::new("/usr/bin/sudo");
    command
        .arg("-C")
        .arg("4")
        .arg("--")
        .arg("/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1")
        .arg("--publisher-bootstrap-fd")
        .arg("3")
        .env_clear();
    // SAFETY: this pre-exec hook performs only async-signal-safe descriptor operations.  The
    // socket peer is the only inherited bootstrap endpoint and FD_CLOEXEC is cleared only on 3.
    unsafe {
        command.pre_exec(move || {
            if libc::dup2(peer_fd, 3) != 3 {
                return Err(std::io::Error::last_os_error());
            }
            let flags = libc::fcntl(3, libc::F_GETFD);
            if flags < 0 || libc::fcntl(3, libc::F_SETFD, flags & !libc::FD_CLOEXEC) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if retained_fd != 3 {
                libc::close(retained_fd);
            }
            if peer_fd != 3 {
                libc::close(peer_fd);
            }
            Ok(())
        });
    }
    // Establish the wait-only reaper before creating the elevated child. If the worker cannot
    // start, no child exists to leak; if it starts, the guard retains its sender until handoff.
    let child_reaper_sender = start_retained_bootstrap_child_reaper_v1()
        .context("start exact elevated bootstrap child reaper")?;
    let child = command
        .spawn()
        .context("launch exact elevated bootstrap executor")?;
    let mut child = RetainedBootstrapChildGuardV1::new(child, child_reaper_sender);
    // The child has its duped peer. The parent must retain only its endpoint from this point.
    drop(peer);

    let request_ingress_started = std::time::Instant::now();
    let request_ingress_deadline = retained_bootstrap_request_ingress_deadline_v1(
        request_ingress_started,
        MAC_BOOTSTRAP_FD3_REQUEST_INGRESS_TIMEOUT_V1,
    );
    write_retained_bootstrap_stream_v1(
        retained.as_raw_fd(),
        &request,
        request_ingress_deadline,
        "direct bootstrap authorization",
    )
    .map_err(|error| {
        emit_retained_bootstrap_milestone_v1(milestones_started, "client.write.timeout_or_eof");
        error
    })?;
    emit_retained_bootstrap_milestone_v1(milestones_started, "client.write.complete");
    // EOF is the sole request boundary. It is emitted only after every canonical byte was sent.
    if retained_bootstrap_deadline_expired_v1(std::time::Instant::now(), request_ingress_deadline) {
        emit_retained_bootstrap_milestone_v1(
            milestones_started,
            "client.request_ingress.timeout_or_eof",
        );
        bail!("direct bootstrap authorization timed out before EOF");
    }
    if unsafe { libc::shutdown(retained.as_raw_fd(), libc::SHUT_WR) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("terminate direct bootstrap authorization frame");
    }
    let request_eof_observed = std::time::Instant::now();
    let response_production_deadline = retained_bootstrap_response_production_deadline_v1(
        request_eof_observed,
        MAC_BOOTSTRAP_FD3_RESPONSE_PRODUCTION_TIMEOUT_V1,
    );
    emit_retained_bootstrap_milestone_v1(milestones_started, "client.request.eof");
    let mut response_first_byte_observed = false;
    let response = match read_retained_bootstrap_stream_v1(
        retained.as_raw_fd(),
        response_production_deadline,
        MAC_BOOTSTRAP_FD3_RESPONSE_TRANSFER_TIMEOUT_V1,
        "direct bootstrap response",
        || {
            response_first_byte_observed = true;
            emit_retained_bootstrap_milestone_v1(milestones_started, "client.response.first_byte");
        },
    ) {
        Ok(response) => response,
        Err(error) => {
            emit_retained_bootstrap_milestone_v1(
                milestones_started,
                if response_first_byte_observed {
                    "client.response.transfer.timeout_or_eof"
                } else {
                    "client.response.production.timeout_or_eof"
                },
            );
            return Err(error);
        }
    };
    emit_retained_bootstrap_milestone_v1(milestones_started, "client.response.eof");
    let response = parse_canonical_direct_bootstrap_response_v1(&response)?;
    let child_exit_deadline = retained_bootstrap_child_exit_deadline_v1(
        std::time::Instant::now(),
        MAC_BOOTSTRAP_FD3_CHILD_EXIT_TIMEOUT_V1,
    );
    match child.wait_for_success_v1(child_exit_deadline)? {
        RetainedBootstrapChildExitV1::Reaped => {
            emit_retained_bootstrap_milestone_v1(milestones_started, "client.child_exit.complete");
        }
        RetainedBootstrapChildExitV1::TimedOut => {
            emit_retained_bootstrap_milestone_v1(milestones_started, "client.child_exit.timeout");
            bail!("exact elevated bootstrap executor did not terminate after its response");
        }
    }
    Ok(response)
}

#[cfg(target_os = "macos")]
fn set_retained_bootstrap_fd_cloexec_v1(fd: i32) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error()).context("read bootstrap socket flags");
    }
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } != 0 {
        return Err(std::io::Error::last_os_error()).context("set bootstrap socket CLOEXEC");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn retained_bootstrap_request_ingress_deadline_v1(
    ingress_started: std::time::Instant,
    timeout: std::time::Duration,
) -> std::time::Instant {
    ingress_started + timeout
}

#[cfg(target_os = "macos")]
fn retained_bootstrap_response_production_deadline_v1(
    request_eof_observed: std::time::Instant,
    timeout: std::time::Duration,
) -> std::time::Instant {
    request_eof_observed + timeout
}

#[cfg(target_os = "macos")]
fn retained_bootstrap_response_transfer_deadline_v1(
    first_response_byte_observed: std::time::Instant,
    timeout: std::time::Duration,
) -> std::time::Instant {
    first_response_byte_observed + timeout
}

#[cfg(target_os = "macos")]
fn retained_bootstrap_child_exit_deadline_v1(
    response_parsed: std::time::Instant,
    timeout: std::time::Duration,
) -> std::time::Instant {
    response_parsed + timeout
}

#[cfg(target_os = "macos")]
fn retained_bootstrap_deadline_expired_v1(
    observed: std::time::Instant,
    deadline: std::time::Instant,
) -> bool {
    observed >= deadline
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetainedBootstrapResponseReadPhaseV1 {
    Production,
    Transfer,
}

#[cfg(target_os = "macos")]
fn retained_bootstrap_response_read_deadline_v1(
    production_deadline: std::time::Instant,
    first_response_byte_observed: Option<std::time::Instant>,
    transfer_timeout: std::time::Duration,
) -> (RetainedBootstrapResponseReadPhaseV1, std::time::Instant) {
    match first_response_byte_observed {
        Some(first_response_byte_observed) => (
            RetainedBootstrapResponseReadPhaseV1::Transfer,
            retained_bootstrap_response_transfer_deadline_v1(
                first_response_byte_observed,
                transfer_timeout,
            ),
        ),
        None => (
            RetainedBootstrapResponseReadPhaseV1::Production,
            production_deadline,
        ),
    }
}

#[cfg(target_os = "macos")]
const RETAINED_BOOTSTRAP_MILESTONE_QUEUE_CAPACITY_V1: usize = 32;

#[cfg(target_os = "macos")]
static RETAINED_BOOTSTRAP_MILESTONE_SENDER_V1: std::sync::OnceLock<
    std::sync::mpsc::SyncSender<String>,
> = std::sync::OnceLock::new();

/// Start the best-effort diagnostic writer before the FD3 protocol deadlines begin.
///
/// The worker is deliberately detached: a blocked stderr pipe can only stall diagnostics, never
/// the bootstrap exchange, child reaping, or process exit. Callers use `try_send` and drop a
/// milestone when the bounded queue is full or the writer has disconnected.
#[cfg(target_os = "macos")]
fn initialize_retained_bootstrap_milestone_logger_v1() {
    if RETAINED_BOOTSTRAP_MILESTONE_SENDER_V1.get().is_some() {
        return;
    }
    let (sender, receiver) =
        std::sync::mpsc::sync_channel(RETAINED_BOOTSTRAP_MILESTONE_QUEUE_CAPACITY_V1);
    if std::thread::Builder::new()
        .name("substrate-bootstrap-stderr".to_string())
        .spawn(move || {
            run_retained_bootstrap_milestone_sink_v1(receiver, |line| {
                use std::io::Write as _;

                let _ = std::io::stderr().write_all(line.as_bytes());
            })
        })
        .is_ok()
    {
        let _ = RETAINED_BOOTSTRAP_MILESTONE_SENDER_V1.set(sender);
    }
}

#[cfg(target_os = "macos")]
fn run_retained_bootstrap_milestone_sink_v1(
    receiver: std::sync::mpsc::Receiver<String>,
    mut sink: impl FnMut(String),
) {
    while let Ok(line) = receiver.recv() {
        sink(line);
    }
}

#[cfg(target_os = "macos")]
fn try_enqueue_retained_bootstrap_milestone_v1(
    sender: &std::sync::mpsc::SyncSender<String>,
    line: String,
) {
    let _ = sender.try_send(line);
}

#[cfg(target_os = "macos")]
fn retained_bootstrap_milestone_line_v1(
    milestone: &'static str,
    elapsed: std::time::Duration,
) -> String {
    format!(
        "substrate.bootstrap.milestone={milestone} elapsed_ms={}",
        elapsed.as_millis()
    )
}

#[cfg(target_os = "macos")]
fn emit_retained_bootstrap_milestone_v1(started: std::time::Instant, milestone: &'static str) {
    if let Some(sender) = RETAINED_BOOTSTRAP_MILESTONE_SENDER_V1.get() {
        let mut line = retained_bootstrap_milestone_line_v1(milestone, started.elapsed());
        line.push('\n');
        try_enqueue_retained_bootstrap_milestone_v1(sender, line);
    }
}

#[cfg(target_os = "macos")]
fn set_retained_bootstrap_no_sigpipe_v1(fd: i32) -> Result<()> {
    let enabled = 1_i32;
    if unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_NOSIGPIPE,
            (&enabled as *const i32).cast(),
            std::mem::size_of::<i32>() as libc::socklen_t,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error()).context("set retained bootstrap SO_NOSIGPIPE");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn wait_for_retained_bootstrap_fd3_event_v1(
    fd: i32,
    events: i16,
    deadline: std::time::Instant,
    phase: &str,
) -> Result<()> {
    loop {
        let now = std::time::Instant::now();
        if now >= deadline {
            bail!("{phase} timed out");
        }
        let timeout_ms = deadline
            .saturating_duration_since(now)
            .as_millis()
            .saturating_add(1)
            .min(i32::MAX as u128) as i32;
        let mut poll_fd = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut poll_fd, 1, timeout_ms) };
        if ready == 0 {
            bail!("{phase} timed out");
        }
        if ready < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error).with_context(|| format!("wait for {phase}"));
        }
        if poll_fd.revents & (libc::POLLNVAL | libc::POLLERR) != 0 {
            bail!("{phase} retained FD3 channel failed");
        }
        if poll_fd.revents & events != 0
            || (events == libc::POLLIN && poll_fd.revents & libc::POLLHUP != 0)
        {
            return Ok(());
        }
        if poll_fd.revents & libc::POLLHUP != 0 {
            bail!("{phase} retained FD3 channel disconnected");
        }
    }
}

#[cfg(target_os = "macos")]
fn write_retained_bootstrap_stream_v1(
    fd: i32,
    bytes: &[u8],
    deadline: std::time::Instant,
    phase: &str,
) -> Result<()> {
    let mut written = 0usize;
    while written < bytes.len() {
        wait_for_retained_bootstrap_fd3_event_v1(fd, libc::POLLOUT, deadline, phase)?;
        let count = unsafe {
            libc::send(
                fd,
                bytes[written..].as_ptr().cast(),
                bytes.len() - written,
                libc::MSG_DONTWAIT,
            )
        };
        if count > 0 {
            written += count as usize;
            continue;
        }
        if count == 0 {
            bail!("{phase} made no forward progress");
        }
        let error = std::io::Error::last_os_error();
        if matches!(
            error.kind(),
            std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
        ) {
            continue;
        }
        return Err(error).with_context(|| format!("write {phase}"));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn read_retained_bootstrap_stream_v1(
    fd: i32,
    production_deadline: std::time::Instant,
    transfer_timeout: std::time::Duration,
    phase: &str,
    mut on_first_response_byte: impl FnMut(),
) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(MAC_BOOTSTRAP_FD3_MAX_BYTES_V1);
    let mut buffer = [0_u8; 16 * 1024];
    let mut first_response_byte_observed = None;
    let (mut read_phase, mut active_deadline) = retained_bootstrap_response_read_deadline_v1(
        production_deadline,
        first_response_byte_observed,
        transfer_timeout,
    );
    loop {
        let read_phase_name = match read_phase {
            RetainedBootstrapResponseReadPhaseV1::Production => "first response byte",
            RetainedBootstrapResponseReadPhaseV1::Transfer => "complete response frame",
        };
        wait_for_retained_bootstrap_fd3_event_v1(
            fd,
            libc::POLLIN,
            active_deadline,
            &format!("{phase} {read_phase_name}"),
        )?;
        let remaining = MAC_BOOTSTRAP_FD3_MAX_BYTES_V1 - bytes.len();
        let capacity = if remaining == 0 {
            1
        } else {
            remaining.min(buffer.len())
        };
        let count =
            unsafe { libc::recv(fd, buffer.as_mut_ptr().cast(), capacity, libc::MSG_DONTWAIT) };
        let response_event_observed = std::time::Instant::now();
        if retained_bootstrap_deadline_expired_v1(response_event_observed, active_deadline) {
            bail!("{phase} {read_phase_name} timed out");
        }
        if count == 0 {
            if bytes.is_empty() {
                bail!("{phase} was empty");
            }
            return Ok(bytes);
        }
        if count > 0 {
            if remaining == 0 {
                bail!("{phase} exceeded the 1 MiB bound");
            }
            bytes.extend_from_slice(&buffer[..count as usize]);
            if first_response_byte_observed.is_none() {
                first_response_byte_observed = Some(response_event_observed);
                active_deadline = retained_bootstrap_response_transfer_deadline_v1(
                    response_event_observed,
                    transfer_timeout,
                );
                read_phase = RetainedBootstrapResponseReadPhaseV1::Transfer;
                on_first_response_byte();
            }
            continue;
        }
        let error = std::io::Error::last_os_error();
        if matches!(
            error.kind(),
            std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
        ) {
            continue;
        }
        return Err(error).with_context(|| format!("read {phase}"));
    }
}

#[cfg(target_os = "macos")]
fn parse_canonical_direct_bootstrap_response_v1(bytes: &[u8]) -> Result<Value> {
    fn write_value(value: &Value, output: &mut String) -> Result<()> {
        match value {
            Value::Null => output.push_str("null"),
            Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
            Value::Number(value) if value.is_i64() || value.is_u64() => {
                output.push_str(&value.to_string())
            }
            Value::Number(_) => bail!("direct bootstrap response contains a non-integer number"),
            Value::String(value) => output.push_str(
                &serde_json::to_string(value).context("encode direct bootstrap response string")?,
            ),
            Value::Array(values) => {
                output.push('[');
                for (index, value) in values.iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    write_value(value, output)?;
                }
                output.push(']');
            }
            Value::Object(values) => {
                output.push('{');
                let mut entries: Vec<_> = values.iter().collect();
                entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
                for (index, (key, value)) in entries.into_iter().enumerate() {
                    if index != 0 {
                        output.push(',');
                    }
                    output.push_str(
                        &serde_json::to_string(key)
                            .context("encode direct bootstrap response key")?,
                    );
                    output.push(':');
                    write_value(value, output)?;
                }
                output.push('}');
            }
        }
        Ok(())
    }

    let value: Value =
        serde_json::from_slice(bytes).context("decode direct bootstrap FD3 response")?;
    let mut canonical = String::new();
    write_value(&value, &mut canonical)?;
    if canonical.as_bytes() != bytes {
        bail!("direct bootstrap FD3 response is not canonical JSON");
    }
    if value
        .as_object()
        .and_then(|object| object.get("bootstrap_channel_bound"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        bail!("direct bootstrap FD3 response is not bound to the bootstrap channel");
    }
    Ok(value)
}

/// Reap the exact elevated child after a successful response without treating a timeout as
/// cancellation. Its durable Prepared intent remains the only retry authority; the direct
/// channel is never automatically resent or used to hard-kill an in-flight transaction. If the
/// direct exchange returns while the executor is still live, ownership moves to a detached
/// reaper so its eventual exit cannot accumulate a zombie in the long-lived caller.
#[cfg(target_os = "macos")]
struct RetainedBootstrapChildGuardV1 {
    child: Option<std::process::Child>,
    reaper_sender: std::sync::mpsc::Sender<std::process::Child>,
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetainedBootstrapChildExitV1 {
    Reaped,
    TimedOut,
}

/// Start the detached, wait-only child reaper before the elevated executor exists.
///
/// The worker owns its receiver and cannot exit while a guard retains a sender. Its loop never
/// panics or cancels a child: every guard owns one sender and can hand off at most one child, so
/// its wait is independent from every other direct-bootstrap exchange.
#[cfg(target_os = "macos")]
fn start_retained_bootstrap_child_reaper_v1(
) -> std::io::Result<std::sync::mpsc::Sender<std::process::Child>> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("substrate-bootstrap-child-reaper".to_string())
        .spawn(move || {
            run_retained_bootstrap_child_reaper_v1(receiver);
        })
        .map(|_| sender)
}

/// Wait-only detached reaper loop. It never exits while any guard retains a sender, which makes
/// `Sender::send` available for every handoff. Every sender belongs to exactly one guard and that
/// guard can send at most one child after taking it from its `Option`.
#[cfg(target_os = "macos")]
fn run_retained_bootstrap_child_reaper_v1(
    receiver: std::sync::mpsc::Receiver<std::process::Child>,
) {
    loop {
        match receiver.recv() {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(_) => return,
        }
    }
}

/// Hand a still-running executor to the already-live wait-only worker without delaying return.
#[cfg(target_os = "macos")]
fn handoff_retained_bootstrap_child_to_reaper_v1(
    reaper_sender: &std::sync::mpsc::Sender<std::process::Child>,
    child: std::process::Child,
) {
    // The receiver cannot disconnect while this guard owns `reaper_sender`: its worker only exits
    // after every sender is dropped and any queued children have been reaped.
    reaper_sender
        .send(child)
        .expect("retained bootstrap child reaper stays live while a guard owns its sender");
}

#[cfg(target_os = "macos")]
impl RetainedBootstrapChildGuardV1 {
    fn new(
        child: std::process::Child,
        reaper_sender: std::sync::mpsc::Sender<std::process::Child>,
    ) -> Self {
        Self {
            child: Some(child),
            reaper_sender,
        }
    }

    fn wait_for_success_v1(
        &mut self,
        deadline: std::time::Instant,
    ) -> Result<RetainedBootstrapChildExitV1> {
        loop {
            let observed = self
                .child
                .as_mut()
                .expect("retained bootstrap child guard is live")
                .try_wait()
                .context("observe exact elevated bootstrap executor")?;
            match observed {
                Some(status) => {
                    self.child.take();
                    if status.success() {
                        return Ok(RetainedBootstrapChildExitV1::Reaped);
                    }
                    bail!("exact elevated bootstrap executor failed");
                }
                None if std::time::Instant::now() >= deadline => {
                    let child = self
                        .child
                        .take()
                        .expect("retained bootstrap child guard is live");
                    handoff_retained_bootstrap_child_to_reaper_v1(&self.reaper_sender, child);
                    return Ok(RetainedBootstrapChildExitV1::TimedOut);
                }
                None => std::thread::sleep(
                    deadline
                        .saturating_duration_since(std::time::Instant::now())
                        .min(std::time::Duration::from_millis(10)),
                ),
            }
        }
    }
}

#[cfg(target_os = "macos")]
impl Drop for RetainedBootstrapChildGuardV1 {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            handoff_retained_bootstrap_child_to_reaper_v1(&self.reaper_sender, child);
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod retained_bootstrap_fd3_deadline_tests {
    use super::*;
    use std::io::Write;
    use std::net::Shutdown;
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;
    use std::process::{Command, Stdio};

    fn assert_child_reaped_exactly_once_v1(pid: libc::pid_t) {
        let mut status = 0;
        assert_eq!(
            unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) },
            -1,
            "a second wait unexpectedly observed the child"
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD),
            "a second wait did not report an already-reaped child"
        );
    }

    fn wait_for_detached_reaper_v1(pid: libc::pid_t) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        loop {
            if unsafe { libc::kill(pid, 0) } != 0 {
                assert_eq!(
                    std::io::Error::last_os_error().raw_os_error(),
                    Some(libc::ESRCH),
                    "child reaper observed an unexpected process error"
                );
                return;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "detached child reaper did not reap the completed executor"
            );
            std::thread::yield_now();
        }
    }

    #[test]
    fn response_production_window_outlives_the_legacy_shared_bound() {
        let request_eof = std::time::Instant::now();
        let legacy_shared_deadline = request_eof + std::time::Duration::from_millis(5);
        let production_deadline = retained_bootstrap_response_production_deadline_v1(
            request_eof,
            std::time::Duration::from_millis(30),
        );
        let first_byte_after_legacy_bound = request_eof + std::time::Duration::from_millis(6);
        let (phase, selected_deadline) = retained_bootstrap_response_read_deadline_v1(
            production_deadline,
            None,
            std::time::Duration::from_millis(5),
        );

        assert_eq!(phase, RetainedBootstrapResponseReadPhaseV1::Production);
        assert!(retained_bootstrap_deadline_expired_v1(
            first_byte_after_legacy_bound,
            legacy_shared_deadline,
        ));
        assert!(!retained_bootstrap_deadline_expired_v1(
            first_byte_after_legacy_bound,
            selected_deadline,
        ));
        assert!(retained_bootstrap_deadline_expired_v1(
            request_eof + std::time::Duration::from_millis(31),
            selected_deadline,
        ));
    }

    #[test]
    fn response_transfer_window_starts_at_first_byte_and_never_slides() {
        let request_eof = std::time::Instant::now();
        let production_deadline = retained_bootstrap_response_production_deadline_v1(
            request_eof,
            std::time::Duration::from_millis(30),
        );
        let first_byte = request_eof + std::time::Duration::from_millis(29);
        let transfer_timeout = std::time::Duration::from_millis(5);
        let (phase, transfer_deadline) = retained_bootstrap_response_read_deadline_v1(
            production_deadline,
            Some(first_byte),
            transfer_timeout,
        );

        assert_eq!(phase, RetainedBootstrapResponseReadPhaseV1::Transfer);
        assert!(retained_bootstrap_deadline_expired_v1(
            request_eof + std::time::Duration::from_millis(31),
            production_deadline,
        ));
        assert!(!retained_bootstrap_deadline_expired_v1(
            request_eof + std::time::Duration::from_millis(33),
            transfer_deadline,
        ));
        assert!(retained_bootstrap_deadline_expired_v1(
            request_eof + std::time::Duration::from_millis(35),
            transfer_deadline,
        ));
    }

    #[test]
    fn response_reader_requires_nonempty_eof_delimited_frame() {
        let (empty_reader, empty_writer) = UnixStream::pair().expect("Darwin empty stream pair");
        empty_writer
            .shutdown(Shutdown::Write)
            .expect("finish empty response");
        assert!(read_retained_bootstrap_stream_v1(
            empty_reader.as_raw_fd(),
            std::time::Instant::now() + std::time::Duration::from_millis(10),
            std::time::Duration::ZERO,
            "empty response",
            || {},
        )
        .is_err());

        let (disconnected_reader, disconnected_writer) =
            UnixStream::pair().expect("Darwin disconnected stream pair");
        drop(disconnected_writer);
        assert!(read_retained_bootstrap_stream_v1(
            disconnected_reader.as_raw_fd(),
            std::time::Instant::now() + std::time::Duration::from_millis(10),
            std::time::Duration::ZERO,
            "disconnected response",
            || {},
        )
        .is_err());

        let (partial_reader, mut partial_writer) =
            UnixStream::pair().expect("Darwin partial stream pair");
        partial_writer
            .write_all(b"{")
            .expect("write partial response byte");
        assert!(read_retained_bootstrap_stream_v1(
            partial_reader.as_raw_fd(),
            std::time::Instant::now() + std::time::Duration::from_millis(10),
            std::time::Duration::ZERO,
            "missing EOF response",
            || {},
        )
        .is_err());
    }

    #[test]
    fn child_exit_and_request_ingress_keep_their_own_deadlines() {
        let start = std::time::Instant::now();
        let ingress_deadline = retained_bootstrap_request_ingress_deadline_v1(
            start,
            MAC_BOOTSTRAP_FD3_REQUEST_INGRESS_TIMEOUT_V1,
        );
        let parsed_response = start + std::time::Duration::from_millis(29);
        let child_exit_deadline = retained_bootstrap_child_exit_deadline_v1(
            parsed_response,
            std::time::Duration::from_millis(5),
        );

        assert_eq!(
            MAC_BOOTSTRAP_FD3_REQUEST_INGRESS_TIMEOUT_V1,
            std::time::Duration::from_secs(5),
        );
        assert_eq!(
            MAC_BOOTSTRAP_FD3_RESPONSE_PRODUCTION_TIMEOUT_V1,
            std::time::Duration::from_secs(30),
        );
        assert_eq!(
            MAC_BOOTSTRAP_FD3_RESPONSE_TRANSFER_TIMEOUT_V1,
            std::time::Duration::from_secs(5),
        );
        assert_eq!(
            MAC_BOOTSTRAP_FD3_CHILD_EXIT_TIMEOUT_V1,
            std::time::Duration::from_secs(5),
        );
        assert_ne!(
            MAC_BOOTSTRAP_FD3_RESPONSE_PRODUCTION_TIMEOUT_V1,
            MAC_BOOTSTRAP_FD3_RESPONSE_TRANSFER_TIMEOUT_V1,
        );
        assert_eq!(
            ingress_deadline,
            start + MAC_BOOTSTRAP_FD3_REQUEST_INGRESS_TIMEOUT_V1,
        );
        assert_eq!(
            child_exit_deadline,
            parsed_response + std::time::Duration::from_millis(5),
        );
        assert!(child_exit_deadline > parsed_response);
    }

    #[test]
    fn child_timeout_handoff_never_kills_and_reaps_once_after_completion() {
        let reaper_sender = start_retained_bootstrap_child_reaper_v1()
            .expect("start retained executor reaper before child launch");
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("IFS= read -r _")
            .stdin(Stdio::piped())
            .spawn()
            .expect("spawn harmless retained executor");
        let pid = child.id() as libc::pid_t;
        let mut child_stdin = child.stdin.take().expect("retain executor stdin");
        {
            let mut guard = RetainedBootstrapChildGuardV1::new(child, reaper_sender);
            assert_eq!(
                guard
                    .wait_for_success_v1(std::time::Instant::now())
                    .expect("observe still-running executor"),
                RetainedBootstrapChildExitV1::TimedOut
            );
            assert!(
                guard.child.is_none(),
                "timed-out guard did not hand off its child to the detached reaper"
            );
        }
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            0,
            "detached handoff cancelled the still-running executor"
        );
        child_stdin
            .write_all(b"release\n")
            .expect("release harmless retained executor");
        drop(child_stdin);
        wait_for_detached_reaper_v1(pid);
        assert_child_reaped_exactly_once_v1(pid);
    }

    #[test]
    fn child_drop_handoff_never_kills_and_reaps_once_after_completion() {
        let reaper_sender = start_retained_bootstrap_child_reaper_v1()
            .expect("start retained executor reaper before child launch");
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("IFS= read -r _")
            .stdin(Stdio::piped())
            .spawn()
            .expect("spawn harmless retained executor");
        let pid = child.id() as libc::pid_t;
        let mut child_stdin = child.stdin.take().expect("retain executor stdin");
        drop(RetainedBootstrapChildGuardV1::new(child, reaper_sender));
        assert_eq!(
            unsafe { libc::kill(pid, 0) },
            0,
            "drop handoff cancelled the still-running executor"
        );
        child_stdin
            .write_all(b"release\n")
            .expect("release harmless retained executor");
        drop(child_stdin);
        wait_for_detached_reaper_v1(pid);
        assert_child_reaped_exactly_once_v1(pid);
    }

    #[test]
    fn terminal_try_wait_paths_reap_once_without_detached_handoff() {
        for (program, should_succeed) in [("/usr/bin/true", true), ("/usr/bin/false", false)] {
            let reaper_sender = start_retained_bootstrap_child_reaper_v1()
                .expect("start retained executor reaper before child launch");
            let child = Command::new(program)
                .spawn()
                .expect("spawn terminal retained executor");
            let pid = child.id() as libc::pid_t;
            let mut guard = RetainedBootstrapChildGuardV1::new(child, reaper_sender);
            let result = guard
                .wait_for_success_v1(std::time::Instant::now() + std::time::Duration::from_secs(1));
            if should_succeed {
                assert_eq!(
                    result.expect("successful executor should be reaped"),
                    RetainedBootstrapChildExitV1::Reaped
                );
            } else {
                assert!(result.is_err(), "failed executor should report failure");
            }
            assert!(
                guard.child.is_none(),
                "terminal try_wait path retained a child for the drop handoff"
            );
            drop(guard);
            assert_child_reaped_exactly_once_v1(pid);
        }
    }
}

pub fn issue_guest_publisher_pairing_ticket_v1(
    _request: &ManagedLifecyclePublisherRequestV1,
) -> Result<GuestPublisherPairingTicketV1> {
    bail!("guest publisher pairing is not part of the R5 MAC control bridge")
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct MacXpcNoCaptureEventHandlerBlockV1 {
    isa: *mut core::ffi::c_void,
    flags: i32,
    reserved: i32,
    invoke: unsafe extern "C" fn(*mut MacXpcNoCaptureEventHandlerBlockV1, *mut core::ffi::c_void),
    descriptor: *const MacXpcNoCaptureEventHandlerBlockDescriptorV1,
}

#[cfg(target_os = "macos")]
#[repr(C)]
struct MacXpcNoCaptureEventHandlerBlockDescriptorV1 {
    reserved: usize,
    size: usize,
}

#[cfg(target_os = "macos")]
static MAC_XPC_NO_CAPTURE_EVENT_HANDLER_BLOCK_DESCRIPTOR_V1:
    MacXpcNoCaptureEventHandlerBlockDescriptorV1 = MacXpcNoCaptureEventHandlerBlockDescriptorV1 {
    reserved: 0,
    size: std::mem::size_of::<MacXpcNoCaptureEventHandlerBlockV1>(),
};

/// Register a valid Blocks-ABI event handler before an XPC connection becomes active.
///
/// The handler deliberately captures nothing and ignores asynchronous connection errors because
/// this synchronous client obtains its one result through the reply object. `_Block_copy` makes
/// a heap-owned handler for registration; XPC retains its own copy before this function releases
/// ours, so later cancellation cannot invoke a stack-lifetime handler.
#[cfg(target_os = "macos")]
unsafe fn install_mac_xpc_no_capture_event_handler_v1(connection: *mut core::ffi::c_void) {
    let mut stack_block = MacXpcNoCaptureEventHandlerBlockV1 {
        isa: std::ptr::addr_of_mut!(_NSConcreteStackBlock).cast(),
        flags: 0,
        reserved: 0,
        invoke: ignore_mac_xpc_event_v1,
        descriptor: &MAC_XPC_NO_CAPTURE_EVENT_HANDLER_BLOCK_DESCRIPTOR_V1,
    };
    let owned_block =
        _Block_copy((&mut stack_block as *mut MacXpcNoCaptureEventHandlerBlockV1).cast());
    assert!(
        !owned_block.is_null(),
        "copy macOS XPC no-capture event handler"
    );
    xpc_connection_set_event_handler(connection, owned_block);
    _Block_release(owned_block);
}

#[cfg(target_os = "macos")]
unsafe extern "C" fn ignore_mac_xpc_event_v1(
    _block: *mut MacXpcNoCaptureEventHandlerBlockV1,
    _event: *mut core::ffi::c_void,
) {
}

/// The reply object and its bounded `response` field are classified separately so XPC transport
/// errors cannot be misreported as publisher protocol frames.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MacXpcReplyObjectV1 {
    ConnectionInvalid,
    ConnectionInterrupted,
    PeerCodeSigningRequirementRejected,
    Dictionary,
    UnexpectedXpcType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MacXpcReplyDataV1 {
    Missing,
    Present(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MacXpcReplyClassificationV1 {
    ConnectionInvalid,
    ConnectionInterrupted,
    PeerCodeSigningRequirementRejected,
    UnexpectedXpcType,
    DictionaryMissingResponse,
    DictionaryEmptyResponse,
    DictionaryOversizedResponse,
    DictionaryResponse,
}

impl MacXpcReplyClassificationV1 {
    const fn label(self) -> &'static str {
        match self {
            Self::ConnectionInvalid => "connection-invalid",
            Self::ConnectionInterrupted => "connection-interrupted",
            Self::PeerCodeSigningRequirementRejected => "peer-code-signing-requirement-rejected",
            Self::UnexpectedXpcType => "unexpected-xpc-type",
            Self::DictionaryMissingResponse => "dictionary-missing-response",
            Self::DictionaryEmptyResponse => "dictionary-empty-response",
            Self::DictionaryOversizedResponse => "dictionary-oversized-response",
            Self::DictionaryResponse => "dictionary-response",
        }
    }
}

fn classify_mac_xpc_reply_v1(
    object: MacXpcReplyObjectV1,
    data: MacXpcReplyDataV1,
    max_frame_bytes: usize,
) -> MacXpcReplyClassificationV1 {
    match object {
        MacXpcReplyObjectV1::ConnectionInvalid => MacXpcReplyClassificationV1::ConnectionInvalid,
        MacXpcReplyObjectV1::ConnectionInterrupted => {
            MacXpcReplyClassificationV1::ConnectionInterrupted
        }
        MacXpcReplyObjectV1::PeerCodeSigningRequirementRejected => {
            MacXpcReplyClassificationV1::PeerCodeSigningRequirementRejected
        }
        MacXpcReplyObjectV1::UnexpectedXpcType => MacXpcReplyClassificationV1::UnexpectedXpcType,
        MacXpcReplyObjectV1::Dictionary => match data {
            MacXpcReplyDataV1::Missing => MacXpcReplyClassificationV1::DictionaryMissingResponse,
            MacXpcReplyDataV1::Present(0) => MacXpcReplyClassificationV1::DictionaryEmptyResponse,
            MacXpcReplyDataV1::Present(length) if length > max_frame_bytes => {
                MacXpcReplyClassificationV1::DictionaryOversizedResponse
            }
            MacXpcReplyDataV1::Present(_) => MacXpcReplyClassificationV1::DictionaryResponse,
        },
    }
}

#[cfg(target_os = "macos")]
unsafe fn mac_xpc_reply_object_v1(reply: *mut core::ffi::c_void) -> MacXpcReplyObjectV1 {
    if reply == std::ptr::addr_of!(_xpc_error_connection_invalid).cast_mut() {
        MacXpcReplyObjectV1::ConnectionInvalid
    } else if reply == std::ptr::addr_of!(_xpc_error_connection_interrupted).cast_mut() {
        MacXpcReplyObjectV1::ConnectionInterrupted
    } else if reply == std::ptr::addr_of!(_xpc_error_peer_code_signing_requirement).cast_mut() {
        MacXpcReplyObjectV1::PeerCodeSigningRequirementRejected
    } else if xpc_get_type(reply) == std::ptr::addr_of!(_xpc_type_dictionary).cast() {
        MacXpcReplyObjectV1::Dictionary
    } else {
        MacXpcReplyObjectV1::UnexpectedXpcType
    }
}

/// Open exactly the fixed privileged Mach XPC service and return one bounded reply frame.
///
/// No executable, socket, environment value, or transport can be supplied by the caller. The
/// service validates the peer's code requirement before its handler accepts a publisher request.
pub fn open_mac_xpc_channel_v1(operation: &str, request: &[u8]) -> Result<Vec<u8>> {
    if !matches!(
        operation,
        "stage-one-create" | "post-pm-action" | "guest-pairing-data-session"
    ) {
        bail!("unknown fixed macOS XPC publisher operation {operation}");
    }
    if request.is_empty() || request.len() > MAX_MAC_XPC_FRAME_BYTES_V1 {
        bail!("macOS XPC publisher request has an invalid frame size");
    }
    #[cfg(target_os = "macos")]
    unsafe {
        use std::ffi::CString;

        const XPC_CONNECTION_MACH_SERVICE_PRIVILEGED_V1: u64 = 1 << 1;
        let service = CString::new(MAC_MACH_SERVICE_V1).expect("fixed Mach service has no NUL");
        let operation = CString::new(operation).context("encode fixed macOS XPC operation")?;
        let connection = xpc_connection_create_mach_service(
            service.as_ptr(),
            std::ptr::null_mut(),
            XPC_CONNECTION_MACH_SERVICE_PRIVILEGED_V1,
        );
        if connection.is_null() {
            bail!("open fixed macOS XPC publisher service");
        }
        install_mac_xpc_no_capture_event_handler_v1(connection);
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
        let object = mac_xpc_reply_object_v1(reply);
        if object != MacXpcReplyObjectV1::Dictionary {
            let classification = classify_mac_xpc_reply_v1(
                object,
                MacXpcReplyDataV1::Missing,
                MAX_MAC_XPC_FRAME_BYTES_V1,
            );
            xpc_release(reply);
            bail!(
                "fixed macOS XPC publisher reply classification: {}",
                classification.label()
            );
        }
        let mut length = 0usize;
        let bytes = xpc_dictionary_get_data(reply, c"response".as_ptr(), &mut length);
        let data = if bytes.is_null() {
            MacXpcReplyDataV1::Missing
        } else {
            MacXpcReplyDataV1::Present(length)
        };
        let classification = classify_mac_xpc_reply_v1(object, data, MAX_MAC_XPC_FRAME_BYTES_V1);
        if classification != MacXpcReplyClassificationV1::DictionaryResponse {
            xpc_release(reply);
            bail!(
                "fixed macOS XPC publisher reply classification: {}",
                classification.label()
            );
        }
        let response = std::slice::from_raw_parts(bytes.cast::<u8>(), length).to_vec();
        xpc_release(reply);
        Ok(response)
    }
    #[cfg(not(target_os = "macos"))]
    {
        bail!("fixed macOS XPC publisher client is unavailable off macOS")
    }
}

/// Verify the fixed publisher service and XPC-enforced designated peer requirement in its reply.
pub fn attest_mac_publisher_response_v1(response: &[u8]) -> Result<()> {
    let value: Value =
        serde_json::from_slice(response).context("decode macOS publisher attestation")?;
    let attestation = value
        .get("xpc_attestation")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("macOS publisher response is missing xpc_attestation"))?;
    let service = attestation
        .get("mach_service")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("macOS publisher response is missing mach_service"))?;
    if service != MAC_MACH_SERVICE_V1 {
        bail!("macOS publisher response names an unexpected Mach service");
    }
    let requirement = attestation
        .get("peer_code_requirement")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            anyhow::anyhow!("macOS publisher response is missing peer_code_requirement")
        })?;
    mac_publisher_control_requirement_cdhash_v1(requirement)
        .context("macOS publisher response has a noncanonical peer code requirement")?;
    if attestation
        .get("audit_token_bound")
        .and_then(Value::as_bool)
        != Some(true)
    {
        bail!("macOS publisher response is missing accepted-peer audit-token binding");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response_with_requirement(requirement: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "xpc_attestation": {
                "mach_service": MAC_MACH_SERVICE_V1,
                "peer_code_requirement": requirement,
                "audit_token_bound": true,
            }
        }))
        .expect("encode test publisher response")
    }

    #[test]
    fn publisher_response_accepts_only_closed_control_requirement_forms() {
        let cdhash = "0123456789abcdef0123456789abcdef01234567";
        let ad_hoc = format!("cdhash H\"{cdhash}\"");
        let production = format!(
            "anchor apple generic and identifier \"com.substrate.lifecycle.publisher.v1\" and cdhash H\"{cdhash}\""
        );
        attest_mac_publisher_response_v1(&response_with_requirement(&ad_hoc))
            .expect("ad-hoc control response");
        attest_mac_publisher_response_v1(&response_with_requirement(&production))
            .expect("Apple-publisher control response");

        for invalid_requirement in [
            format!("cdhash H\"{}\"", cdhash.to_ascii_uppercase()),
            format!("cdhash H\"{cdhash}\" and identifier \"attacker\""),
            "identifier \"com.substrate.lifecycle.publisher.v1\"".to_string(),
        ] {
            assert!(attest_mac_publisher_response_v1(&response_with_requirement(
                &invalid_requirement
            ))
            .is_err());
        }
    }

    #[test]
    fn mac_xpc_reply_classifier_is_fail_closed_and_bounded() {
        const MAX_FRAME_BYTES: usize = 8;
        use MacXpcReplyClassificationV1 as Classification;
        use MacXpcReplyDataV1 as Data;
        use MacXpcReplyObjectV1 as Object;

        for (object, data, expected) in [
            (
                Object::ConnectionInvalid,
                Data::Missing,
                Classification::ConnectionInvalid,
            ),
            (
                Object::ConnectionInterrupted,
                Data::Missing,
                Classification::ConnectionInterrupted,
            ),
            (
                Object::PeerCodeSigningRequirementRejected,
                Data::Missing,
                Classification::PeerCodeSigningRequirementRejected,
            ),
            (
                Object::UnexpectedXpcType,
                Data::Missing,
                Classification::UnexpectedXpcType,
            ),
            (
                Object::Dictionary,
                Data::Missing,
                Classification::DictionaryMissingResponse,
            ),
            (
                Object::Dictionary,
                Data::Present(0),
                Classification::DictionaryEmptyResponse,
            ),
            (
                Object::Dictionary,
                Data::Present(MAX_FRAME_BYTES + 1),
                Classification::DictionaryOversizedResponse,
            ),
            (
                Object::Dictionary,
                Data::Present(MAX_FRAME_BYTES),
                Classification::DictionaryResponse,
            ),
        ] {
            assert_eq!(
                classify_mac_xpc_reply_v1(object, data, MAX_FRAME_BYTES),
                expected
            );
        }
    }
}

#[cfg(target_os = "macos")]
#[link(name = "System")]
unsafe extern "C" {
    static mut _NSConcreteStackBlock: [*mut core::ffi::c_void; 32];
    static _xpc_type_dictionary: core::ffi::c_void;
    static _xpc_error_connection_invalid: core::ffi::c_void;
    static _xpc_error_connection_interrupted: core::ffi::c_void;
    static _xpc_error_peer_code_signing_requirement: core::ffi::c_void;
    fn _Block_copy(block: *const core::ffi::c_void) -> *mut core::ffi::c_void;
    fn _Block_release(block: *const core::ffi::c_void);
    fn xpc_connection_create_mach_service(
        name: *const core::ffi::c_char,
        target_queue: *mut core::ffi::c_void,
        flags: u64,
    ) -> *mut core::ffi::c_void;
    fn xpc_connection_set_event_handler(
        connection: *mut core::ffi::c_void,
        handler: *mut core::ffi::c_void,
    );
    fn xpc_connection_activate(connection: *mut core::ffi::c_void);
    fn xpc_connection_cancel(connection: *mut core::ffi::c_void);
    fn xpc_connection_send_message_with_reply_sync(
        connection: *mut core::ffi::c_void,
        message: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
    fn xpc_get_type(object: *mut core::ffi::c_void) -> *const core::ffi::c_void;
    fn xpc_dictionary_create_empty() -> *mut core::ffi::c_void;
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
    fn xpc_dictionary_get_data(
        dictionary: *mut core::ffi::c_void,
        key: *const core::ffi::c_char,
        length: *mut usize,
    ) -> *const core::ffi::c_void;
    fn xpc_release(object: *mut core::ffi::c_void);
}
