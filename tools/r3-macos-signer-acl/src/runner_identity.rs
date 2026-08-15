use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::{c_void, CString};
use std::fs::File;
use std::io::Read;
use std::os::fd::FromRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::ptr;
use std::sync::OnceLock;

use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, document_sha256_v2, sha256_hex_v2, validate_executable_identity_v2,
    ExecutableIdentityV2,
};
use substrate_r3_macos_finalizer::experiment::freeze_manifest::{
    CandidateFreezeArtifactRoleV2, CandidateFreezeManifestV2,
};
use substrate_r3_macos_finalizer::experiment::{
    CodeSignatureKindV2, CodeSigningPostureV2, AD_HOC_HARDENED_RUNTIME_FLAGS_V2,
    EMPTY_ENTITLEMENTS_SHA256_V2, EXPERIMENT_OWNER_V2, EXPERIMENT_VERSION_V2,
};

type CfType = *const c_void;
type SecStaticCode = *const c_void;
type SecRequirement = *const c_void;
type OsStatus = i32;

const ERR_SEC_SUCCESS: OsStatus = 0;
const K_SEC_CS_SIGNING_AND_REQUIREMENT_INFORMATION: u32 = (1 << 1) | (1 << 2);
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;
const K_CF_NUMBER_SINT64_TYPE: i32 = 4;

include!(concat!(env!("OUT_DIR"), "/runner_identity.rs"));

#[derive(Debug, Clone, Copy)]
pub(crate) struct FrozenCode<'a> {
    pub path: &'a str,
    pub signing_identifier: &'a str,
    pub role: CandidateFreezeArtifactRoleV2,
}

static CANDIDATE_FREEZE_AUTHORITY: OnceLock<CandidateFreezeManifestV2> = OnceLock::new();

pub(crate) fn install_candidate_freeze_authority(
    manifest: CandidateFreezeManifestV2,
) -> Result<()> {
    manifest.validate()?;
    if let Some(existing) = CANDIDATE_FREEZE_AUTHORITY.get() {
        if existing != &manifest {
            bail!("candidate freeze authority changed within one root-runner process")
        }
        return Ok(());
    }
    CANDIDATE_FREEZE_AUTHORITY
        .set(manifest)
        .map_err(|_| anyhow::anyhow!("candidate freeze authority was concurrently initialized"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct ExecutableFileIdentity {
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    link_count: u64,
    size: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct StaticCodeObservationV2 {
    executable_identity_sha256: String,
    cdhash: String,
    code_directory_flags: u32,
    team_identifier: Option<String>,
    entitlements_blob_size: u64,
    entitlements_blob_sha256: String,
    entitlement_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticCodeFactsV2 {
    signing_identifier: String,
    designated_requirement: String,
    cdhash: String,
    code_directory_flags: u32,
    team_identifier: Option<String>,
    entitlements_blob_size: u64,
    entitlements_blob_sha256: String,
    entitlement_keys: Vec<String>,
}

pub(crate) fn measure_frozen_executable(code: FrozenCode<'_>) -> Result<ExecutableIdentityV2> {
    measure_frozen_executable_and_posture(code).map(|(identity, _)| identity)
}

/// Join one installed artifact's role, fixed pathname, signing identifier, size, and bytes to the
/// reviewed manifest without making a Security.framework call. The root runner preflights every
/// installed child this way before beginning the sequence of SecStaticCode measurements, so no
/// child code measurement can precede the complete byte/path/role join.
pub(crate) fn preflight_frozen_executable(code: FrozenCode<'_>) -> Result<()> {
    let manifest = CANDIDATE_FREEZE_AUTHORITY
        .get()
        .context("candidate freeze authority was not installed before executable preflight")?;
    let entry = manifest
        .artifacts
        .iter()
        .find(|entry| entry.role == code.role)
        .context("candidate freeze manifest lacks the requested executable role")?;
    if entry.intended_path != code.path
        || entry.signing_identifier.as_deref() != Some(code.signing_identifier)
    {
        bail!("runtime executable request differs from its candidate freeze role")
    }
    let (file_identity, executable_sha256) = hash_immutable_executable(Path::new(code.path))?;
    if executable_sha256 != entry.sha256 || file_identity.size != entry.size {
        bail!("installed executable bytes differ from the reviewed candidate manifest")
    }
    Ok(())
}

pub(crate) fn measure_frozen_executable_and_posture(
    code: FrozenCode<'_>,
) -> Result<(ExecutableIdentityV2, CodeSigningPostureV2)> {
    let manifest = CANDIDATE_FREEZE_AUTHORITY
        .get()
        .context("candidate freeze authority was not installed before executable measurement")?;
    let entry = manifest
        .artifacts
        .iter()
        .find(|entry| entry.role == code.role)
        .context("candidate freeze manifest lacks the requested executable role")?;
    if entry.intended_path != code.path
        || entry.signing_identifier.as_deref() != Some(code.signing_identifier)
    {
        bail!("runtime executable request differs from its candidate freeze role")
    }
    let path = Path::new(code.path);
    let (file_identity, executable_sha256) = hash_immutable_executable(path)?;
    if executable_sha256 != entry.sha256 || file_identity.size != entry.size {
        bail!("installed executable bytes differ from the reviewed candidate manifest")
    }
    let signing = verify_static_code(
        path,
        code.signing_identifier,
        entry.designated_requirement.as_deref(),
        entry.cdhash.as_deref(),
    )?;
    let build_inputs_sha256 = if matches!(
        code.role,
        CandidateFreezeArtifactRoleV2::CoordinatorExecutable
            | CandidateFreezeArtifactRoleV2::AlternateCoordinatorExecutable
    ) {
        manifest.coordinator_build_digest.clone()
    } else {
        manifest.global_build_digest.clone()
    };
    let identity = ExecutableIdentityV2 {
        source_commit: manifest.source_commit.clone(),
        source_tree: manifest.source_tree.clone(),
        source_hashes_sha256: manifest.source_hashes_manifest_sha256.clone(),
        build_inputs_sha256,
        executable_sha256,
        executable_size: file_identity.size,
        intended_path: code.path.to_owned(),
        physical_identity_sha256: sha256_hex_v2(&canonical_bytes_v2(&file_identity)?),
        signing_identifier: code.signing_identifier.to_owned(),
        designated_requirement: signing.designated_requirement.clone(),
        cdhash: signing.cdhash.clone(),
    };
    validate_executable_identity_v2(&identity, code.path, code.signing_identifier)?;
    let posture = signing_posture(&identity, signing)?;
    Ok((identity, posture))
}

/// Measure the runner without an impossible executable self-hash literal. The resulting actual
/// bytes/physical identity/CDHash/DR are externally frozen before any child can perform an effect.
pub(crate) fn measure_runner_executable(
    path: &'static str,
    signing_identifier: &'static str,
) -> Result<(ExecutableIdentityV2, CodeSigningPostureV2)> {
    let manifest = CANDIDATE_FREEZE_AUTHORITY
        .get()
        .context("candidate freeze authority was not installed before runner measurement")?;
    let entry = manifest
        .artifacts
        .iter()
        .find(|entry| {
            entry.role == CandidateFreezeArtifactRoleV2::DisposableExperimentRunnerExecutable
        })
        .context("candidate freeze manifest lacks the root runner artifact")?;
    if entry.intended_path != path
        || entry.signing_identifier.as_deref() != Some(signing_identifier)
    {
        bail!("root runner path or signing identity differs from the candidate freeze manifest")
    }
    let path = Path::new(path);
    let (file_identity, executable_sha256) = hash_immutable_executable(path)?;
    if executable_sha256 != entry.sha256 || file_identity.size != entry.size {
        bail!("root runner bytes differ from the candidate freeze manifest")
    }
    let signing = verify_static_code(
        path,
        signing_identifier,
        entry.designated_requirement.as_deref(),
        entry.cdhash.as_deref(),
    )?;
    let identity = ExecutableIdentityV2 {
        source_commit: manifest.source_commit.clone(),
        source_tree: manifest.source_tree.clone(),
        source_hashes_sha256: manifest.source_hashes_manifest_sha256.clone(),
        build_inputs_sha256: manifest.global_build_digest.clone(),
        executable_sha256,
        executable_size: file_identity.size,
        intended_path: path.to_string_lossy().into_owned(),
        physical_identity_sha256: sha256_hex_v2(&canonical_bytes_v2(&file_identity)?),
        signing_identifier: signing.signing_identifier.clone(),
        designated_requirement: signing.designated_requirement.clone(),
        cdhash: signing.cdhash.clone(),
    };
    validate_executable_identity_v2(
        &identity,
        path.to_string_lossy().as_ref(),
        signing_identifier,
    )?;
    let posture = signing_posture(&identity, signing)?;
    Ok((identity, posture))
}

pub(crate) fn capability_digest() -> &'static str {
    CAPABILITY_DIGEST
}

pub(crate) fn launch_plist_sha256() -> &'static str {
    LAUNCH_PLIST_SHA256
}

pub(crate) fn creator_code(path: &'static str, identifier: &'static str) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::CreatorExecutable,
    )
}

pub(crate) fn wrong_code(path: &'static str, identifier: &'static str) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::WrongIdentityExecutable,
    )
}

pub(crate) fn publisher_code(path: &'static str, identifier: &'static str) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::DisposablePublisherExecutable,
    )
}

pub(crate) fn harness_code(path: &'static str, identifier: &'static str) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::DisposableHarnessExecutable,
    )
}

pub(crate) fn finalizer_code(path: &'static str, identifier: &'static str) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::FinalizerExecutable,
    )
}

pub(crate) fn coordinator_code(
    path: &'static str,
    identifier: &'static str,
) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::CoordinatorExecutable,
    )
}

pub(crate) fn peer_probe_code(path: &'static str, identifier: &'static str) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::PeerCodeProbeExecutable,
    )
}

pub(crate) fn nobody_owner_probe_code(
    path: &'static str,
    identifier: &'static str,
) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::NobodyOwnerProbeExecutable,
    )
}

pub(crate) fn benign_injection_library_code(
    path: &'static str,
    identifier: &'static str,
) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::BenignInjectionLibrary,
    )
}

pub(crate) fn observer_code(path: &'static str, identifier: &'static str) -> FrozenCode<'static> {
    common_code(
        path,
        identifier,
        CandidateFreezeArtifactRoleV2::SecurityAgentObserverExecutable,
    )
}

fn common_code(
    path: &'static str,
    signing_identifier: &'static str,
    role: CandidateFreezeArtifactRoleV2,
) -> FrozenCode<'static> {
    FrozenCode {
        path,
        signing_identifier,
        role,
    }
}

pub(crate) fn require_non_placeholder_digest(value: &str, label: &str) -> Result<()> {
    if is_zero_hex(value, 64) {
        bail!("{label} was not frozen into the experiment runner")
    }
    Ok(())
}

fn is_zero_hex(value: &str, length: usize) -> bool {
    value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || value.bytes().all(|byte| byte == b'0')
}

fn hash_immutable_executable(path: &Path) -> Result<(ExecutableFileIdentity, String)> {
    require_root_owned_immutable_path(path)?;
    let encoded = CString::new(path.as_os_str().as_bytes())?;
    // SAFETY: exact path and O_NOFOLLOW protects the terminal component.
    let raw = unsafe {
        libc::open(
            encoded.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if raw < 0 {
        return Err(std::io::Error::last_os_error()).context("open frozen executable");
    }
    // SAFETY: successful open transferred ownership.
    let mut file = unsafe { File::from_raw_fd(raw) };
    let before = file_identity(&file.metadata()?);
    require_executable_file(&before)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let after = file_identity(&file.metadata()?);
    if before != after {
        bail!("frozen executable changed while hashing")
    }
    require_root_owned_immutable_path(path)?;
    Ok((before, hex_lower(&digest.finalize())))
}

pub(crate) fn require_root_owned_immutable_path(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        bail!("frozen executable path is not absolute")
    }
    let mut current = PathBuf::from("/");
    for component in path.components().skip(1) {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current)
            .with_context(|| format!("inspect immutable path component {}", current.display()))?;
        if metadata.file_type().is_symlink() || metadata.uid() != 0 || metadata.mode() & 0o022 != 0
        {
            bail!("frozen executable path is not root-owned immutable no-follow state")
        }
        if current == path {
            if !metadata.file_type().is_file() || metadata.nlink() != 1 {
                bail!("frozen executable is not one regular-file identity")
            }
        } else if !metadata.is_dir() {
            bail!("frozen executable parent is not a directory")
        }
    }
    Ok(())
}

fn file_identity(metadata: &std::fs::Metadata) -> ExecutableFileIdentity {
    ExecutableFileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        owner_uid: metadata.uid(),
        owner_gid: metadata.gid(),
        mode: metadata.mode(),
        link_count: metadata.nlink(),
        size: metadata.size(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
    }
}

fn require_executable_file(value: &ExecutableFileIdentity) -> Result<()> {
    if value.owner_uid != 0
        || value.owner_gid != 0
        || value.mode & 0o022 != 0
        || value.link_count != 1
        || value.mode & u32::from(libc::S_IFMT) != u32::from(libc::S_IFREG)
        || value.size == 0
    {
        bail!("frozen executable is not root:wheel immutable regular nlink=1")
    }
    Ok(())
}

fn verify_static_code(
    path: &Path,
    expected_identifier: &str,
    expected_requirement: Option<&str>,
    expected_cdhash: Option<&str>,
) -> Result<StaticCodeFactsV2> {
    if expected_identifier.is_empty() || expected_identifier.contains(['\0', '"']) {
        bail!("frozen signing identifier is invalid")
    }
    let expected_cdhash = expected_cdhash
        .map(|value| decode_exact_hex(value, 20))
        .transpose()?;
    // SAFETY: all create/copy rule objects remain retained in `owned` through last use.
    unsafe {
        let mut owned = OwnedCf::default();
        let url = CFURLCreateFromFileSystemRepresentation(
            kCFAllocatorDefault,
            path.as_os_str().as_bytes().as_ptr(),
            path.as_os_str().as_bytes().len() as isize,
            0,
        );
        if url.is_null() {
            bail!("create frozen executable URL")
        }
        owned.hold(url);
        let mut code: SecStaticCode = ptr::null();
        let status = SecStaticCodeCreateWithPath(url, 0, &mut code);
        if status != ERR_SEC_SUCCESS || code.is_null() {
            bail!("resolve frozen static code failed with OSStatus {status}")
        }
        owned.hold(code);
        let validity_requirement = expected_requirement
            .map(str::to_owned)
            .unwrap_or_else(|| format!("identifier \"{expected_identifier}\""));
        let requirement_string = cf_string(&mut owned, &validity_requirement)?;
        let mut validity: SecRequirement = ptr::null();
        let status = SecRequirementCreateWithString(requirement_string, 0, &mut validity);
        if status != ERR_SEC_SUCCESS || validity.is_null() {
            bail!("compile frozen requirement failed with OSStatus {status}")
        }
        owned.hold(validity);
        let status = SecStaticCodeCheckValidity(code, 0, validity);
        if status != ERR_SEC_SUCCESS {
            bail!("frozen static code requirement failed with OSStatus {status}")
        }
        let mut information: CfType = ptr::null();
        let status = SecCodeCopySigningInformation(
            code,
            K_SEC_CS_SIGNING_AND_REQUIREMENT_INFORMATION,
            &mut information,
        );
        if status != ERR_SEC_SUCCESS || information.is_null() {
            bail!("copy frozen static-code information failed with OSStatus {status}")
        }
        owned.hold(information);
        if CFGetTypeID(information) != CFDictionaryGetTypeID() {
            bail!("frozen signing information is not a dictionary")
        }
        let unique = CFDictionaryGetValue(information, kSecCodeInfoUnique);
        if unique.is_null() || CFGetTypeID(unique) != CFDataGetTypeID() {
            bail!("frozen signing information has no CDHash")
        }
        let length = CFDataGetLength(unique);
        let bytes = CFDataGetBytePtr(unique);
        if length != 20 || bytes.is_null() {
            bail!("frozen signing CDHash is not exactly 20 bytes")
        }
        let observed = std::slice::from_raw_parts(bytes, length as usize);
        if expected_cdhash
            .as_deref()
            .is_some_and(|expected| observed != expected)
        {
            bail!("frozen signing CDHash differs from the compiled literal")
        }
        let identifier = CFDictionaryGetValue(information, kSecCodeInfoIdentifier);
        if identifier.is_null() || CFGetTypeID(identifier) != CFStringGetTypeID() {
            bail!("frozen signing information has no signing identifier")
        }
        let signing_identifier = cf_string_value(identifier)?;
        if signing_identifier != expected_identifier {
            bail!("frozen signing identifier differs from the compiled literal")
        }
        let mut designated: SecRequirement = ptr::null();
        let status = SecCodeCopyDesignatedRequirement(code, 0, &mut designated);
        if status != ERR_SEC_SUCCESS || designated.is_null() {
            bail!("copy frozen designated requirement failed with OSStatus {status}")
        }
        owned.hold(designated);
        let mut designated_string: CfType = ptr::null();
        let status = SecRequirementCopyString(designated, 0, &mut designated_string);
        if status != ERR_SEC_SUCCESS || designated_string.is_null() {
            bail!("copy frozen designated requirement text failed with OSStatus {status}")
        }
        owned.hold(designated_string);
        let designated_requirement = cf_string_value(designated_string)?;
        if expected_requirement.is_some_and(|expected| expected != designated_requirement) {
            bail!("actual designated requirement differs from the compiled literal")
        }
        let flags = CFDictionaryGetValue(information, kSecCodeInfoFlags);
        if flags.is_null() || CFGetTypeID(flags) != CFNumberGetTypeID() {
            bail!("frozen signing information has no CodeDirectory flags")
        }
        let mut signed_flags = 0_i64;
        if CFNumberGetValue(
            flags,
            K_CF_NUMBER_SINT64_TYPE,
            (&mut signed_flags as *mut i64).cast(),
        ) == 0
            || signed_flags < 0
            || signed_flags > i64::from(u32::MAX)
        {
            bail!("frozen CodeDirectory flags are not a bounded unsigned value")
        }
        let code_directory_flags = signed_flags as u32;
        if code_directory_flags != AD_HOC_HARDENED_RUNTIME_FLAGS_V2 {
            bail!("frozen executable is not exact ad-hoc Hardened Runtime code")
        }
        let team = CFDictionaryGetValue(information, kSecCodeInfoTeamIdentifier);
        if !team.is_null() {
            bail!("ad-hoc experiment executable unexpectedly carries a TeamIdentifier")
        }
        let entitlements = CFDictionaryGetValue(information, kSecCodeInfoEntitlements);
        let entitlement_dictionary =
            CFDictionaryGetValue(information, kSecCodeInfoEntitlementsDict);
        if !entitlements.is_null() || !entitlement_dictionary.is_null() {
            bail!("experiment executable carries an entitlement blob or entitlement dictionary")
        }
        Ok(StaticCodeFactsV2 {
            signing_identifier,
            designated_requirement,
            cdhash: hex_lower(observed),
            code_directory_flags,
            team_identifier: None,
            entitlements_blob_size: 0,
            entitlements_blob_sha256: EMPTY_ENTITLEMENTS_SHA256_V2.to_owned(),
            entitlement_keys: Vec::new(),
        })
    }
}

fn signing_posture(
    identity: &ExecutableIdentityV2,
    signing: StaticCodeFactsV2,
) -> Result<CodeSigningPostureV2> {
    let observation = StaticCodeObservationV2 {
        executable_identity_sha256: document_sha256_v2(identity)?,
        cdhash: signing.cdhash.clone(),
        code_directory_flags: signing.code_directory_flags,
        team_identifier: signing.team_identifier.clone(),
        entitlements_blob_size: signing.entitlements_blob_size,
        entitlements_blob_sha256: signing.entitlements_blob_sha256.clone(),
        entitlement_keys: signing.entitlement_keys.clone(),
    };
    let posture = CodeSigningPostureV2 {
        schema_owner: EXPERIMENT_OWNER_V2.to_owned(),
        schema_version: EXPERIMENT_VERSION_V2,
        executable_identity_sha256: document_sha256_v2(identity)?,
        signature_kind: CodeSignatureKindV2::AdHoc,
        code_directory_flags: signing.code_directory_flags,
        hardened_runtime: signing.code_directory_flags == AD_HOC_HARDENED_RUNTIME_FLAGS_V2,
        library_validation: signing.code_directory_flags == AD_HOC_HARDENED_RUNTIME_FLAGS_V2,
        team_identifier: signing.team_identifier,
        entitlements_blob_size: signing.entitlements_blob_size,
        entitlements_blob_sha256: signing.entitlements_blob_sha256,
        entitlement_keys: signing.entitlement_keys,
        verification_observation_sha256: document_sha256_v2(&observation)?,
    };
    posture.validate_for(identity)?;
    Ok(posture)
}

unsafe fn cf_string_value(value: CfType) -> Result<String> {
    let length = unsafe { CFStringGetLength(value) };
    if length < 0 {
        bail!("frozen signing string length is invalid")
    }
    let capacity = unsafe { CFStringGetMaximumSizeForEncoding(length, K_CF_STRING_ENCODING_UTF8) };
    if !(0..=1024 * 1024).contains(&capacity) {
        bail!("frozen signing string exceeds its bound")
    }
    let mut bytes = vec![0_i8; capacity as usize + 1];
    if unsafe {
        CFStringGetCString(
            value,
            bytes.as_mut_ptr(),
            bytes.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
        )
    } == 0
    {
        bail!("decode frozen signing string")
    }
    // SAFETY: successful CFStringGetCString wrote a NUL-terminated string in the live buffer.
    Ok(unsafe { std::ffi::CStr::from_ptr(bytes.as_ptr()) }
        .to_str()?
        .to_owned())
}

fn decode_exact_hex(value: &str, expected_bytes: usize) -> Result<Vec<u8>> {
    if value.len() != expected_bytes * 2
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("frozen identity contains invalid lowercase hexadecimal")
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok((hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?))
        .collect()
}

fn hex_nibble(value: u8) -> Result<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => bail!("invalid lowercase hexadecimal nibble"),
    }
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(char::from(HEX[(byte >> 4) as usize]));
        result.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
    result
}

#[derive(Default)]
struct OwnedCf(Vec<CfType>);

impl OwnedCf {
    fn hold<T>(&mut self, value: *const T) {
        if !value.is_null() {
            self.0.push(value.cast());
        }
    }
}

impl Drop for OwnedCf {
    fn drop(&mut self) {
        // SAFETY: each entry follows one CoreFoundation/Security create/copy rule.
        unsafe {
            for value in self.0.drain(..).rev() {
                CFRelease(value);
            }
        }
    }
}

unsafe fn cf_string(owned: &mut OwnedCf, value: &str) -> Result<CfType> {
    // SAFETY: bytes are live for the call and result follows create rule.
    let string = unsafe {
        CFStringCreateWithBytes(
            kCFAllocatorDefault,
            value.as_bytes().as_ptr(),
            value.len() as isize,
            K_CF_STRING_ENCODING_UTF8,
            0,
        )
    };
    if string.is_null() {
        bail!("allocate frozen requirement string")
    }
    owned.hold(string);
    Ok(string)
}

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    static kSecCodeInfoUnique: CfType;
    static kSecCodeInfoIdentifier: CfType;
    static kSecCodeInfoFlags: CfType;
    static kSecCodeInfoTeamIdentifier: CfType;
    static kSecCodeInfoEntitlements: CfType;
    static kSecCodeInfoEntitlementsDict: CfType;
    fn SecStaticCodeCreateWithPath(
        path: CfType,
        flags: u32,
        static_code: *mut SecStaticCode,
    ) -> OsStatus;
    fn SecRequirementCreateWithString(
        requirement_text: CfType,
        flags: u32,
        requirement: *mut SecRequirement,
    ) -> OsStatus;
    fn SecStaticCodeCheckValidity(
        static_code: SecStaticCode,
        flags: u32,
        requirement: SecRequirement,
    ) -> OsStatus;
    fn SecCodeCopySigningInformation(
        code: SecStaticCode,
        flags: u32,
        information: *mut CfType,
    ) -> OsStatus;
    fn SecCodeCopyDesignatedRequirement(
        code: SecStaticCode,
        flags: u32,
        requirement: *mut SecRequirement,
    ) -> OsStatus;
    fn SecRequirementCopyString(
        requirement: SecRequirement,
        flags: u32,
        requirement_text: *mut CfType,
    ) -> OsStatus;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFAllocatorDefault: CfType;
    fn CFURLCreateFromFileSystemRepresentation(
        allocator: CfType,
        buffer: *const u8,
        length: isize,
        is_directory: u8,
    ) -> CfType;
    fn CFStringCreateWithBytes(
        allocator: CfType,
        bytes: *const u8,
        length: isize,
        encoding: u32,
        external_representation: u8,
    ) -> CfType;
    fn CFStringGetTypeID() -> usize;
    fn CFStringGetLength(string: CfType) -> isize;
    fn CFStringGetMaximumSizeForEncoding(length: isize, encoding: u32) -> isize;
    fn CFStringGetCString(string: CfType, buffer: *mut i8, buffer_size: isize, encoding: u32)
        -> u8;
    fn CFRelease(value: CfType);
    fn CFGetTypeID(value: CfType) -> usize;
    fn CFDictionaryGetTypeID() -> usize;
    fn CFDataGetTypeID() -> usize;
    fn CFNumberGetTypeID() -> usize;
    fn CFDictionaryGetValue(dictionary: CfType, key: CfType) -> CfType;
    fn CFNumberGetValue(number: CfType, number_type: i32, value: *mut c_void) -> u8;
    fn CFDataGetLength(data: CfType) -> isize;
    fn CFDataGetBytePtr(data: CfType) -> *const u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn capability_digest_is_recomputed_from_the_shared_canonical_manifest() {
        let bytes = include_bytes!("../../r3-macos-finalizer/capability-v2.json");
        assert_eq!(capability_digest(), format!("{:x}", Sha256::digest(bytes)));
        assert_eq!(capability_digest().len(), 64);
    }

    #[test]
    fn child_identity_authority_is_runtime_manifest_only() {
        let code = FrozenCode {
            path: "/fixed/runner",
            signing_identifier: "fixed.runner",
            role: CandidateFreezeArtifactRoleV2::DisposableExperimentRunnerExecutable,
        };
        assert_eq!(code.path, "/fixed/runner");
        assert_eq!(code.signing_identifier, "fixed.runner");
        assert_eq!(
            code.role,
            CandidateFreezeArtifactRoleV2::DisposableExperimentRunnerExecutable
        );
        assert!(require_non_placeholder_digest(&"0".repeat(64), "test").is_err());
        let build = include_str!("../build.rs");
        for forbidden in [
            "R3_CREATOR_SHA256",
            "R3_CREATOR_CDHASH",
            "R3_CREATOR_REQUIREMENT",
            "R3_RUNNER_SHA256",
            "R3_RUNNER_CDHASH",
            "R3_RUNNER_REQUIREMENT",
            "R3_SOURCE_COMMIT",
            "R3_SOURCE_TREE",
            "R3_SOURCE_HASHES_SHA256",
            "R3_BUILD_INPUTS_SHA256",
        ] {
            assert!(
                !build.contains(forbidden),
                "forbidden build input {forbidden}"
            );
        }
        assert!(include_str!("runner_identity.rs").contains(
            "candidate freeze authority was not installed before executable measurement"
        ));
    }

    #[test]
    fn measurement_surface_requires_nofollow_hash_code_and_physical_join() {
        assert_eq!(AD_HOC_HARDENED_RUNTIME_FLAGS_V2, 0x0001_2002);
        let source = include_str!("runner_identity.rs");
        assert!(source.contains("O_NOFOLLOW"));
        assert!(source.contains("SecStaticCodeCheckValidity"));
        assert!(source.contains("kSecCodeInfoUnique"));
        assert!(source.contains("physical_identity_sha256"));
        assert!(source.contains("AD_HOC_HARDENED_RUNTIME_FLAGS_V2"));
        assert!(source.contains("kSecCodeInfoEntitlementsDict"));
        assert!(source.contains("kSecCodeInfoTeamIdentifier"));
        assert!(source.contains(
            "let status = SecRequirementCopyString(designated, 0, &mut designated_string);"
        ));
        assert!(source.contains("requirement_text: *mut CfType"));
        assert!(!include_str!("../build.rs").contains("\"RUNNER\","));
    }

    #[test]
    #[ignore = "requires an externally ad-hoc-signed copy of this test executable"]
    fn native_static_code_measurement_uses_status_and_out_parameter() {
        let path = std::env::current_exe().expect("resolve standalone proof executable");
        let signing = verify_static_code(
            &path,
            "com.atomize.substrate.r3-macos-runner-identity-standalone-proof.v1",
            None,
            None,
        )
        .expect("measure standalone proof executable");
        assert_eq!(
            signing.signing_identifier,
            "com.atomize.substrate.r3-macos-runner-identity-standalone-proof.v1"
        );
        assert_eq!(signing.cdhash.len(), 40);
        assert_eq!(
            signing.code_directory_flags,
            AD_HOC_HARDENED_RUNTIME_FLAGS_V2
        );
        assert!(signing.designated_requirement.contains("cdhash H\""));
        assert_eq!(signing.team_identifier, None);
        assert_eq!(signing.entitlements_blob_size, 0);
        assert_eq!(
            signing.entitlements_blob_sha256,
            EMPTY_ENTITLEMENTS_SHA256_V2
        );
        assert!(signing.entitlement_keys.is_empty());
    }
}
