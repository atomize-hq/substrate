//! Shim execution context and environment detection.

use anyhow::{anyhow, Context, Result};
use std::collections::HashSet;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use transport_api_types::{
    InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
    PlatformPrincipalV1,
};

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(windows)]
use std::process::Command;
#[cfg(windows)]
use transport_api_types::WindowsForwarderScopeV1;
#[cfg(windows)]
use transport_api_types::{normalize_windows_install_bootstrap_path, normalize_windows_pipe_path};
#[cfg(windows)]
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_INSUFFICIENT_BUFFER, HANDLE, INVALID_HANDLE_VALUE,
    RPC_E_CHANGED_MODE,
};
#[cfg(windows)]
use windows_sys::Win32::Security::{
    GetSidIdentifierAuthority, GetSidSubAuthority, GetSidSubAuthorityCount, GetTokenInformation,
    IsValidSid, IsWellKnownSid, LookupAccountSidW, TokenUser, WinAnonymousSid, TOKEN_IMPERSONATE,
    TOKEN_QUERY, TOKEN_USER,
};
#[cfg(windows)]
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FileAttributeTagInfo, FileIdInfo, GetFileInformationByHandleEx,
    GetFinalPathNameByHandleW, FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_ID_INFO, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(windows)]
use windows_sys::Win32::System::Com::{
    CoInitializeEx, CoTaskMemFree, CoUninitialize, COINIT_APARTMENTTHREADED,
};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::{FOLDERID_LocalAppData, SHGetKnownFolderPath};

/// Environment variable names used by the shim system.
pub const SHIM_ACTIVE_VAR: &str = "SHIM_ACTIVE";
pub const SHIM_DEPTH_VAR: &str = "SHIM_DEPTH";
pub const SHIM_SESSION_VAR: &str = "SHIM_SESSION_ID";
pub const ORIGINAL_PATH_VAR: &str = "SHIM_ORIGINAL_PATH";
pub const TRACE_LOG_VAR: &str = "SHIM_TRACE_LOG";
pub const CACHE_BUST_VAR: &str = "SHIM_CACHE_BUST";
pub const SHIM_CALLER_VAR: &str = "SHIM_CALLER";
pub const SHIM_CALL_STACK_VAR: &str = "SHIM_CALL_STACK";
pub const SHIM_PARENT_CMD_VAR: &str = "SHIM_PARENT_CMD_ID";
pub const SUBSTRATE_WORLD_VAR: &str = "SUBSTRATE_WORLD";
pub const SUBSTRATE_WORLD_ENABLED_VAR: &str = "SUBSTRATE_WORLD_ENABLED";
pub const SUBSTRATE_WORLD_ID_VAR: &str = "SUBSTRATE_WORLD_ID";
pub const SUBSTRATE_WORLD_PROJECT_DIR_VAR: &str = "SUBSTRATE_WORLD_PROJECT_DIR";

const INSTALL_BOOTSTRAP_CONTEXT_ENV: &str = "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1";
const INSTALL_BOOTSTRAP_COMMITMENT_ENV: &str = "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT";
const INSTALL_BOOTSTRAP_ACCOUNT_ENV: &str = "SUBSTRATE_INSTALL_PRIMARY_USER";
#[cfg(unix)]
const INSTALL_BOOTSTRAP_UID_ENV: &str = "SUBSTRATE_INSTALL_PRIMARY_UID";

#[cfg(target_os = "macos")]
const DEFAULT_LIMA_VM_NAME: &str = "substrate";
#[cfg(target_os = "macos")]
const CANONICAL_GUEST_SOCKET_PATH: &str = "/run/substrate.sock";
#[cfg(windows)]
const DEFAULT_WSL_DISTRO: &str = "Substrate-WSL";
#[cfg(windows)]
const DEFAULT_WSL_PIPE: &str = r"\\.\pipe\substrate-agent";
#[cfg(windows)]
const CANONICAL_WSL_GUEST_SOCKET_PATH: &str = "/run/substrate.sock";

/// Execution context for a shim invocation.
#[derive(Debug)]
pub struct ShimContext {
    /// The command name this shim was invoked as (e.g., "git", "npm").
    pub command_name: String,
    /// Directory containing shim binaries.
    pub shim_dir: PathBuf,
    /// Clean search paths (excluding shim directory).
    pub search_paths: Vec<PathBuf>,
    /// Optional log file path.
    pub log_file: Option<PathBuf>,
    /// Session ID for command chain correlation.
    pub session_id: String,
    /// Execution depth for nested commands.
    pub depth: u32,
}

impl ShimContext {
    /// Create context from current executable and environment.
    pub fn from_current_exe() -> Result<Self> {
        let exe = env::current_exe().context("Failed to get current executable path")?;
        let invoked = env::args_os()
            .next()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("shim invocation witness is missing"))?;
        let invoked_path = resolve_invoked_path(&invoked, &exe)?;

        let shim_dir = invoked_path
            .parent()
            .ok_or_else(|| anyhow!("invocation witness has no parent directory"))?
            .to_path_buf();
        let command_name = invoked_path
            .file_name()
            .ok_or_else(|| anyhow!("invocation witness has no command name"))?
            .to_string_lossy()
            .to_string();

        let original_path = env::var(ORIGINAL_PATH_VAR).ok();
        let merged_path = merge_path_sources(original_path);
        let search_paths = build_clean_search_path(&shim_dir, merged_path)?;
        let log_file = env::var(TRACE_LOG_VAR).ok().map(PathBuf::from);
        let depth = env::var(SHIM_DEPTH_VAR)
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        let session_id =
            env::var(SHIM_SESSION_VAR).unwrap_or_else(|_| uuid::Uuid::now_v7().to_string());

        Ok(Self {
            command_name,
            shim_dir,
            search_paths,
            log_file,
            session_id,
            depth,
        })
    }

    /// Check if we should skip execution (already shimmed).
    pub fn should_skip_shimming(&self) -> bool {
        env::var(SHIM_ACTIVE_VAR).is_ok()
    }

    /// Check if bypass mode is enabled.
    pub fn is_bypass_enabled() -> bool {
        env::var("SHIM_BYPASS").as_deref() == Ok("1")
    }

    /// Set up environment for command execution (idempotent).
    pub fn setup_execution_env(&self) {
        env::set_var(SHIM_SESSION_VAR, &self.session_id);

        if env::var(SHIM_CALLER_VAR).is_err() {
            env::set_var(SHIM_CALLER_VAR, &self.command_name);
            env::set_var(SHIM_CALL_STACK_VAR, &self.command_name);
        } else {
            let current_stack = env::var(SHIM_CALL_STACK_VAR).unwrap_or_default();
            let new_stack = build_safe_call_stack(&current_stack, &self.command_name);
            env::set_var(SHIM_CALL_STACK_VAR, new_stack);
        }

        if env::var(SHIM_ACTIVE_VAR).is_err() {
            env::set_var(SHIM_ACTIVE_VAR, "1");
        }

        let current_depth = env::var(SHIM_DEPTH_VAR)
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        env::set_var(SHIM_DEPTH_VAR, (current_depth + 1).to_string());
    }
}

/// Build safe call stack with limits to prevent loops and memory issues.
fn build_safe_call_stack(current: &str, new_cmd: &str) -> String {
    const MAX_STACK_ITEMS: usize = 8;

    if current.is_empty() {
        return new_cmd.to_string();
    }

    let mut items: Vec<&str> = current.split(',').collect();
    if items.last() == Some(&new_cmd) {
        return current.to_string();
    }

    items.push(new_cmd);
    if items.len() > MAX_STACK_ITEMS {
        items = items[items.len() - MAX_STACK_ITEMS + 1..].to_vec();
        items.insert(0, "...");
    }

    items.join(",")
}

pub(crate) fn resolve_invoked_path(invoked: &OsStr, running_executable: &Path) -> Result<PathBuf> {
    if invoked.is_empty() {
        return Err(anyhow!("shim invocation witness is empty"));
    }

    let invoked_path = PathBuf::from(invoked);
    if invoked_path.is_absolute() || invoked_path.components().count() > 1 {
        let candidate = if invoked_path.is_absolute() {
            invoked_path
        } else {
            env::current_dir()
                .context("failed to resolve invocation current directory")?
                .join(invoked_path)
        };
        return check_candidate(&candidate, running_executable);
    }

    let path_var = env::var_os("PATH").ok_or_else(|| anyhow!("PATH is unavailable"))?;
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();
    for dir in env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() || !dir.is_absolute() {
            continue;
        }
        if let Some(found) = find_candidate_in_dir(&dir, invoked, running_executable) {
            if seen.insert(found.clone()) {
                candidates.push(found);
            }
        }
    }

    match candidates.len() {
        1 => Ok(candidates.remove(0)),
        0 => Err(anyhow!(
            "bare shim invocation did not have exactly one installed witness"
        )),
        _ => Err(anyhow!(
            "bare shim invocation matched multiple installed witnesses"
        )),
    }
}

fn check_candidate(candidate: &Path, running_executable: &Path) -> Result<PathBuf> {
    let candidate = normalized_invocation_candidate_path(candidate)?;
    host_prefix_from_invocation_witness(&candidate, running_executable)?;
    Ok(candidate)
}

#[cfg(windows)]
fn find_candidate_in_dir(dir: &Path, base: &OsStr, running_executable: &Path) -> Option<PathBuf> {
    let base_path = dir.join(base);
    if let Ok(found) = check_candidate(&base_path, running_executable) {
        return Some(found);
    }

    if Path::new(base).extension().is_some() {
        return None;
    }

    let pathext = env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string());
    for ext in pathext.split(';') {
        let trimmed = ext.trim_start_matches('.').trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut candidate = base_path.clone();
        candidate.set_extension(trimmed);
        if let Ok(found) = check_candidate(&candidate, running_executable) {
            return Some(found);
        }
    }

    None
}

#[cfg(not(windows))]
fn find_candidate_in_dir(dir: &Path, base: &OsStr, running_executable: &Path) -> Option<PathBuf> {
    let candidate = dir.join(base);
    check_candidate(&candidate, running_executable).ok()
}

pub(crate) fn resolve_install_bootstrap_context_from_invocation(
    invoked_path: &Path,
    running_executable: &Path,
) -> Result<InstallBootstrapContextCarrierV1> {
    let prefix = host_prefix_from_invocation_witness(invoked_path, running_executable)?;
    let principal = current_platform_principal_v1()?;
    let context = match &principal {
        PlatformPrincipalV1::Unix { account, uid } => InstallBootstrapContextV1::new_unix(
            prefix
                .to_str()
                .ok_or_else(|| anyhow!("install prefix is not valid UTF-8"))?,
            account,
            *uid,
        )
        .context("invalid Unix install bootstrap context")?,
        PlatformPrincipalV1::Windows { account, sid } => InstallBootstrapContextV1::new_windows(
            prefix
                .to_str()
                .ok_or_else(|| anyhow!("install prefix is not valid UTF-8"))?,
            account,
            sid,
        )
        .context("invalid Windows install bootstrap context")?,
    };
    let carrier = InstallBootstrapContextCarrierV1::from_context(context)
        .context("failed to commit install bootstrap context")?;
    validate_inherited_bootstrap_projections(&carrier)?;
    Ok(carrier)
}

pub(crate) fn current_platform_principal_v1() -> Result<PlatformPrincipalV1> {
    #[cfg(unix)]
    {
        use nix::libc;

        // SAFETY: geteuid has no preconditions.
        let uid = unsafe { libc::geteuid() };
        if uid == 0 {
            return Err(anyhow!(
                "effective UID 0 requires the R2-2 intended-principal boundary"
            ));
        }

        let (account, home) = unix_account_record_by_uid(uid)?;
        let (roundtrip_account, roundtrip_uid, roundtrip_home) =
            unix_account_record_by_name(&account)?;
        if roundtrip_uid == 0
            || roundtrip_account != account
            || roundtrip_uid != uid
            || roundtrip_home != home
        {
            return Err(anyhow!(
                "current Unix principal does not round-trip through the account database"
            ));
        }

        return Ok(PlatformPrincipalV1::Unix { account, uid });
    }

    #[cfg(windows)]
    {
        let (principal, _) = current_windows_principal_and_known_folder()?;
        return Ok(principal);
    }

    #[allow(unreachable_code)]
    Err(anyhow!(
        "install bootstrap principal is unavailable on this platform"
    ))
}

#[cfg(windows)]
struct OwnedWindowsHandle(HANDLE);

#[cfg(windows)]
impl Drop for OwnedWindowsHandle {
    fn drop(&mut self) {
        // SAFETY: this wrapper is created only for a real owned handle and closes it exactly once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
struct WindowsComInitialization(bool);

#[cfg(windows)]
impl Drop for WindowsComInitialization {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: true records a successful CoInitializeEx call on this thread.
            unsafe {
                CoUninitialize();
            }
        }
    }
}

#[cfg(windows)]
extern "system" {
    fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
}

fn install_bootstrap_projection_values(
    carrier: &InstallBootstrapContextCarrierV1,
) -> Result<Vec<(&'static str, OsString)>> {
    carrier
        .validate()
        .context("invalid install bootstrap carrier")?;
    let encoded = carrier.encode().context("failed to encode carrier")?;
    if carrier.encode().context("failed to encode carrier")? != encoded {
        return Err(anyhow!(
            "install bootstrap carrier spelling is noncanonical"
        ));
    }

    let mut projections = vec![
        (
            "SUBSTRATE_HOME",
            OsString::from(&carrier.context.selected_host_prefix),
        ),
        (
            "SUBSTRATE_ROOT",
            OsString::from(&carrier.context.selected_host_prefix),
        ),
        (
            INSTALL_BOOTSTRAP_COMMITMENT_ENV,
            OsString::from(&carrier.host_context_commitment),
        ),
        (INSTALL_BOOTSTRAP_CONTEXT_ENV, OsString::from(encoded)),
    ];

    match &carrier.context.intended_host_principal {
        PlatformPrincipalV1::Unix { account, uid } => {
            projections.push((INSTALL_BOOTSTRAP_ACCOUNT_ENV, OsString::from(account)));
            #[cfg(unix)]
            projections.push((INSTALL_BOOTSTRAP_UID_ENV, OsString::from(uid.to_string())));
            #[cfg(not(unix))]
            let _ = uid;
        }
        PlatformPrincipalV1::Windows { account, .. } => {
            projections.push((INSTALL_BOOTSTRAP_ACCOUNT_ENV, OsString::from(account)));
        }
    }

    Ok(projections)
}

fn validate_inherited_bootstrap_projections(
    carrier: &InstallBootstrapContextCarrierV1,
) -> Result<()> {
    let projections = install_bootstrap_projection_values(carrier)?;
    let inherited_present = [
        INSTALL_BOOTSTRAP_CONTEXT_ENV,
        INSTALL_BOOTSTRAP_COMMITMENT_ENV,
        INSTALL_BOOTSTRAP_ACCOUNT_ENV,
        #[cfg(unix)]
        INSTALL_BOOTSTRAP_UID_ENV,
    ]
    .into_iter()
    .all(|key| env::var_os(key).is_none());
    if inherited_present {
        return Ok(());
    }

    for (key, expected) in projections {
        if env::var_os(key).as_deref() != Some(expected.as_os_str()) {
            return Err(anyhow!(
                "install bootstrap environment projection is missing or conflicting"
            ));
        }
    }

    Ok(())
}

pub(crate) fn default_platform_bootstrap_mapping_v1(
    host_carrier: &InstallBootstrapContextCarrierV1,
) -> Result<Option<PlatformBootstrapMappingV1>> {
    #[cfg(target_os = "linux")]
    {
        let _ = host_carrier;
        return Ok(None);
    }

    #[cfg(target_os = "macos")]
    {
        return observe_default_lima_platform_bootstrap_mapping(host_carrier).map(Some);
    }

    #[cfg(windows)]
    {
        return observe_default_wsl_platform_bootstrap_mapping(host_carrier).map(Some);
    }

    #[allow(unreachable_code)]
    Ok(None)
}

fn host_prefix_from_invocation_witness(
    witness: &Path,
    running_executable: &Path,
) -> Result<PathBuf> {
    #[cfg(unix)]
    {
        return validate_unix_shim_invocation_witness(witness, running_executable);
    }

    #[cfg(windows)]
    {
        return validate_windows_shim_invocation_witness(witness, running_executable);
    }

    #[allow(unreachable_code)]
    Err(anyhow!(
        "platform invocation witness validation is unavailable"
    ))
}

fn normalized_invocation_candidate_path(candidate: &Path) -> Result<PathBuf> {
    #[cfg(unix)]
    {
        let normalized = lexically_normalize_absolute_invocation_path(candidate)?;
        if normalized != candidate {
            return Err(anyhow!("invocation pathname is not canonical"));
        }
        return Ok(normalized);
    }

    #[cfg(windows)]
    {
        let raw = candidate
            .to_str()
            .ok_or_else(|| anyhow!("Windows invocation pathname is not valid UTF-8"))?;
        let normalized = normalize_windows_install_bootstrap_path(raw)
            .context("invalid Windows invocation pathname")?;
        if normalized != raw {
            return Err(anyhow!("Windows invocation pathname is not canonical"));
        }
        return Ok(PathBuf::from(normalized));
    }

    #[allow(unreachable_code)]
    Err(anyhow!("invocation pathname validation is unavailable"))
}

#[cfg(windows)]
fn validate_windows_shim_invocation_witness(
    witness: &Path,
    running_executable: &Path,
) -> Result<PathBuf> {
    let witness = physical_windows_shim_invocation_path(witness)?;
    let command = witness
        .file_stem()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("invocation witness has no command name"))?;
    if command == "." || command == ".." {
        return Err(anyhow!("invocation witness command name is invalid"));
    }

    let shims = witness
        .parent()
        .filter(|path| path.file_name() == Some(OsStr::new("shims")))
        .ok_or_else(|| anyhow!("installed invocation witness is outside A/shims"))?;
    let prefix = shims
        .parent()
        .ok_or_else(|| anyhow!("installed invocation witness has no prefix"))?;
    let prefix_handle = open_windows_path_no_follow(prefix, true)?;
    let shims_handle = open_windows_path_no_follow(shims, true)?;
    let witness_handle = open_windows_path_no_follow(&witness, false)?;
    let running_handle = open_windows_path_no_follow(running_executable, false)?;
    require_same_windows_file_identity(&witness, running_executable)?;
    if windows_file_identity(witness_handle.0)? != windows_file_identity(running_handle.0)? {
        return Err(anyhow!(
            "installed invocation witness does not match running executable"
        ));
    }

    let physical_witness = PathBuf::from(final_windows_handle_path(witness_handle.0)?);
    let physical_command = physical_witness
        .file_stem()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("physical invocation witness has no command name"))?;
    if physical_command == "." || physical_command == ".." {
        return Err(anyhow!(
            "physical invocation witness command name is invalid"
        ));
    }

    let physical_shims = physical_witness
        .parent()
        .filter(|path| path.file_name() == Some(OsStr::new("shims")))
        .ok_or_else(|| anyhow!("physical invocation witness is outside A/shims"))?;
    let physical_prefix = physical_shims
        .parent()
        .ok_or_else(|| anyhow!("physical invocation witness has no prefix"))?;

    let _prefix_ancestors = open_windows_ancestors_no_follow(prefix)?;
    let _shims_ancestors = open_windows_ancestors_no_follow(shims)?;
    let _witness_ancestors = open_windows_ancestors_no_follow(&witness)?;
    let _running_ancestors = open_windows_ancestors_no_follow(running_executable)?;
    let physical_prefix_handle = open_windows_path_no_follow(physical_prefix, true)?;
    let physical_shims_handle = open_windows_path_no_follow(physical_shims, true)?;
    if windows_file_identity(prefix_handle.0)? != windows_file_identity(physical_prefix_handle.0)?
        || windows_file_identity(shims_handle.0)? != windows_file_identity(physical_shims_handle.0)?
    {
        return Err(anyhow!(
            "installed invocation witness aliases a different prefix"
        ));
    }

    let raw_prefix = physical_prefix
        .to_str()
        .ok_or_else(|| anyhow!("installed invocation prefix is not valid UTF-8"))?;
    let normalized_prefix = normalize_windows_install_bootstrap_path(raw_prefix)
        .context("installed invocation prefix is invalid")?;
    Ok(PathBuf::from(normalized_prefix))
}

#[cfg(windows)]
fn require_same_windows_file_identity(left: &Path, right: &Path) -> Result<()> {
    let _left_ancestors = open_windows_ancestors_no_follow(left)?;
    let _right_ancestors = open_windows_ancestors_no_follow(right)?;
    let left_handle = open_windows_path_no_follow(left, false)?;
    let right_handle = open_windows_path_no_follow(right, false)?;
    if windows_file_identity(left_handle.0)? != windows_file_identity(right_handle.0)? {
        return Err(anyhow!(
            "installed invocation witness does not match running executable"
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn open_current_windows_token() -> Result<OwnedWindowsHandle> {
    let mut token = 0;
    // SAFETY: GetCurrentProcess returns a valid pseudo-handle and token points to writable storage.
    let opened = unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY | TOKEN_IMPERSONATE,
            &mut token,
        )
    };
    if opened == 0 || token == 0 || token == INVALID_HANDLE_VALUE {
        return Err(anyhow!("current Windows process token could not be opened"));
    }
    Ok(OwnedWindowsHandle(token))
}

#[cfg(windows)]
fn windows_principal_from_token(token: HANDLE) -> Result<PlatformPrincipalV1> {
    let mut required = 0_u32;
    // SAFETY: the zero-length sizing call permits a null output buffer.
    let sized =
        unsafe { GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut required) };
    if sized != 0
        || required < std::mem::size_of::<TOKEN_USER>() as u32
        || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
    {
        return Err(anyhow!("current Windows token user could not be sized"));
    }
    let words =
        (required as usize + std::mem::size_of::<usize>() - 1) / std::mem::size_of::<usize>();
    let mut buffer = vec![0_usize; words];
    // SAFETY: buffer is aligned, writable for at least required bytes, and return length is valid.
    let loaded = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            required,
            &mut required,
        )
    };
    if loaded == 0 {
        return Err(anyhow!("current Windows token user could not be read"));
    }
    // SAFETY: GetTokenInformation initialized a TOKEN_USER at the aligned start of buffer.
    let token_user = unsafe { std::ptr::read(buffer.as_ptr().cast::<TOKEN_USER>()) };
    let sid = token_user.User.Sid;
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err(anyhow!("current Windows token SID is missing or malformed"));
    }
    // SAFETY: sid was validated and WinAnonymousSid is a valid well-known SID selector.
    if unsafe { IsWellKnownSid(sid, WinAnonymousSid) } != 0 {
        return Err(anyhow!(
            "anonymous Windows token is not an install principal"
        ));
    }
    let sid_text = canonical_windows_sid(sid)?;
    let account = windows_account_for_sid(sid)?;
    Ok(PlatformPrincipalV1::Windows {
        account,
        sid: sid_text,
    })
}

#[cfg(windows)]
fn canonical_windows_sid(sid: *mut std::ffi::c_void) -> Result<String> {
    // SAFETY: caller validated sid; the revision is the first byte of every SID.
    let revision = unsafe { std::ptr::read(sid.cast::<u8>()) };
    if revision != 1 {
        return Err(anyhow!(
            "current Windows token SID has unsupported revision"
        ));
    }
    // SAFETY: caller validated sid, so SID accessor pointers are valid for the SID lifetime.
    let authority = unsafe { GetSidIdentifierAuthority(sid) };
    let count = unsafe { GetSidSubAuthorityCount(sid) };
    if authority.is_null() || count.is_null() {
        return Err(anyhow!("current Windows token SID is incomplete"));
    }
    // SAFETY: accessor pointers reference fields within the validated SID.
    let authority_bytes = unsafe { (*authority).Value };
    let authority_value = authority_bytes
        .into_iter()
        .fold(0_u64, |value, byte| (value << 8) | u64::from(byte));
    if authority_value >= (1_u64 << 32) {
        return Err(anyhow!(
            "current Windows token SID authority is not representable"
        ));
    }
    let mut text = format!("S-{revision}-{authority_value}");
    // SAFETY: the count pointer references the validated SID's SubAuthorityCount.
    let count = unsafe { *count };
    for index in 0..u32::from(count) {
        // SAFETY: IsValidSid guarantees every indexed subauthority is in bounds.
        let subauthority = unsafe { GetSidSubAuthority(sid, index) };
        if subauthority.is_null() {
            return Err(anyhow!("current Windows token SID is incomplete"));
        }
        // SAFETY: accessor returned a pointer to the indexed subauthority.
        text.push_str(&format!("-{}", unsafe { *subauthority }));
    }
    Ok(text)
}

#[cfg(windows)]
fn windows_account_for_sid(sid: *mut std::ffi::c_void) -> Result<String> {
    let mut name_len = 0_u32;
    let mut domain_len = 0_u32;
    let mut use_type = 0;
    // SAFETY: the sizing call permits null output buffers and sid is validated by the caller.
    let sized = unsafe {
        LookupAccountSidW(
            std::ptr::null(),
            sid,
            std::ptr::null_mut(),
            &mut name_len,
            std::ptr::null_mut(),
            &mut domain_len,
            &mut use_type,
        )
    };
    if sized != 0 || name_len == 0 || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER {
        return Err(anyhow!("current Windows token SID could not be translated"));
    }
    let mut name = vec![0_u16; name_len as usize];
    let mut domain = vec![0_u16; domain_len as usize];
    let domain_ptr = if domain.is_empty() {
        std::ptr::null_mut()
    } else {
        domain.as_mut_ptr()
    };
    // SAFETY: the buffers have the exact sizes returned by the sizing call.
    let translated = unsafe {
        LookupAccountSidW(
            std::ptr::null(),
            sid,
            name.as_mut_ptr(),
            &mut name_len,
            domain_ptr,
            &mut domain_len,
            &mut use_type,
        )
    };
    if translated == 0 || name_len == 0 {
        return Err(anyhow!("current Windows token SID could not be translated"));
    }
    let name_end = name
        .iter()
        .position(|value| *value == 0)
        .ok_or_else(|| anyhow!("current Windows account name is not terminated"))?;
    let domain_end = domain
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(domain.len());
    let name = String::from_utf16(&name[..name_end])
        .context("current Windows account name is malformed")?;
    let domain = String::from_utf16(&domain[..domain_end])
        .context("current Windows account domain is malformed")?;
    if !valid_windows_account_component(&name)
        || (!domain.is_empty() && !valid_windows_account_component(&domain))
    {
        return Err(anyhow!("current Windows account name is invalid"));
    }
    if domain.is_empty() {
        Ok(name)
    } else {
        Ok(format!("{domain}\\{name}"))
    }
}

#[cfg(windows)]
fn valid_windows_account_component(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|character| matches!(character, '\0' | '\n' | '\r' | '\\'))
}

#[cfg(windows)]
fn windows_known_folder_from_token(token: HANDLE) -> Result<PathBuf> {
    let _com = initialize_windows_com()?;
    let mut raw = std::ptr::null_mut();
    // SAFETY: raw points to writable PWSTR storage and token is the held current-process token.
    let status = unsafe { SHGetKnownFolderPath(&FOLDERID_LocalAppData, 0, token, &mut raw) };
    if status < 0 || raw.is_null() {
        // SAFETY: CoTaskMemFree accepts null and SHGetKnownFolderPath owns any returned allocation.
        unsafe {
            CoTaskMemFree(raw.cast());
        }
        return Err(anyhow!("current token LocalApplicationData is unavailable"));
    }
    let mut wide = Vec::new();
    for index in 0..32_768 {
        // SAFETY: SHGetKnownFolderPath returned a NUL-terminated UTF-16 allocation.
        let value = unsafe { *raw.add(index) };
        if value == 0 {
            break;
        }
        wide.push(value);
    }
    // SAFETY: raw is the allocation returned by SHGetKnownFolderPath and is freed exactly once.
    unsafe {
        CoTaskMemFree(raw.cast());
    }
    if wide.is_empty() || wide.len() == 32_768 {
        return Err(anyhow!("current token LocalApplicationData is malformed"));
    }
    let path =
        String::from_utf16(&wide).context("current token LocalApplicationData is malformed")?;
    let normalized = transport_api_types::normalize_windows_install_bootstrap_path(&path)
        .context("current token LocalApplicationData is invalid")?;
    Ok(PathBuf::from(normalized))
}

#[cfg(windows)]
fn initialize_windows_com() -> Result<WindowsComInitialization> {
    // SAFETY: the reserved pointer is null and the apartment flag is valid.
    let status = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    if status >= 0 {
        return Ok(WindowsComInitialization(true));
    }
    if status == RPC_E_CHANGED_MODE {
        return Ok(WindowsComInitialization(false));
    }
    Err(anyhow!(
        "COM could not be initialized for Windows Known Folder observation"
    ))
}

#[cfg(windows)]
fn absolute_windows_wsl_executable_path() -> Result<PathBuf> {
    let mut capacity = 260_usize;
    loop {
        let mut buffer = vec![0_u16; capacity];
        // SAFETY: the buffer is valid for `capacity` UTF-16 code units.
        let written = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), capacity as u32) };
        if written == 0 {
            return Err(anyhow!("current Windows system directory is unavailable"));
        }
        if written as usize >= capacity {
            capacity = written as usize + 1;
            if capacity > 32_768 {
                return Err(anyhow!("current Windows system directory is too long"));
            }
            continue;
        }

        buffer.truncate(written as usize);
        let system_dir =
            String::from_utf16(&buffer).context("current Windows system directory is malformed")?;
        let system_dir = normalize_windows_install_bootstrap_path(&system_dir)
            .context("current Windows system directory is invalid")?;
        let wsl = PathBuf::from(system_dir).join("wsl.exe");
        let _ancestors = open_windows_ancestors_no_follow(&wsl)?;
        let _handle = open_windows_path_no_follow(&wsl, false)?;
        return Ok(wsl);
    }
}

#[cfg(windows)]
fn physical_windows_shim_invocation_path(path: &Path) -> Result<PathBuf> {
    match path.file_name() {
        Some(name) if name.to_string_lossy().is_empty() => {
            Err(anyhow!("invocation witness has no command name"))
        }
        Some(name) if Path::new(name).extension().is_some() => {
            if Path::new(name)
                .extension()
                .and_then(OsStr::to_str)
                .is_some_and(|value| value.eq_ignore_ascii_case("exe"))
            {
                Ok(path.to_path_buf())
            } else {
                Err(anyhow!(
                    "invocation witness has the wrong command extension"
                ))
            }
        }
        Some(name) => Ok(path.with_file_name(format!("{}.exe", name.to_string_lossy()))),
        None => Err(anyhow!("invocation witness has no command name")),
    }
}

#[cfg(windows)]
fn open_windows_ancestors_no_follow(path: &Path) -> Result<Vec<OwnedWindowsHandle>> {
    if !path.is_absolute() {
        return Err(anyhow!("Windows pathname is not absolute"));
    }

    let mut handles = Vec::new();
    for ancestor in path.ancestors().skip(1) {
        if ancestor.parent().is_none() {
            break;
        }
        handles.push(
            open_windows_path_no_follow(ancestor, true)
                .context("Windows pathname has an unavailable or reparse ancestor")?,
        );
    }
    Ok(handles)
}

#[cfg(windows)]
fn open_windows_path_no_follow(path: &Path, directory: bool) -> Result<OwnedWindowsHandle> {
    let wide = windows_wide_path(path)?;
    let flags = FILE_FLAG_OPEN_REPARSE_POINT
        | if directory {
            FILE_FLAG_BACKUP_SEMANTICS
        } else {
            0
        };
    // SAFETY: wide is NUL-terminated and all optional pointer arguments are null.
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            flags,
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(anyhow!(
            "Windows path could not be opened without following"
        ));
    }

    let handle = OwnedWindowsHandle(handle);
    let mut attributes = std::mem::MaybeUninit::<FILE_ATTRIBUTE_TAG_INFO>::zeroed();
    // SAFETY: handle is valid and attributes points to writable storage of the declared size.
    let loaded = unsafe {
        GetFileInformationByHandleEx(
            handle.0,
            FileAttributeTagInfo,
            attributes.as_mut_ptr().cast(),
            std::mem::size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    };
    if loaded == 0 {
        return Err(anyhow!("Windows path attributes could not be read"));
    }

    // SAFETY: successful GetFileInformationByHandleEx initialized attributes.
    let attributes = unsafe { attributes.assume_init() };
    if attributes.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(anyhow!("Windows invocation witness uses a reparse point"));
    }
    Ok(handle)
}

#[cfg(windows)]
fn windows_wide_path(path: &Path) -> Result<Vec<u16>> {
    let mut wide = path.as_os_str().encode_wide().collect::<Vec<_>>();
    if wide.is_empty() || wide.contains(&0) {
        return Err(anyhow!("Windows pathname is empty or contains NUL"));
    }
    wide.push(0);
    Ok(wide)
}

#[cfg(windows)]
fn windows_file_identity(handle: HANDLE) -> Result<(u64, [u8; 16])> {
    let mut identity = std::mem::MaybeUninit::<FILE_ID_INFO>::zeroed();
    // SAFETY: handle is valid and identity points to writable storage of the declared size.
    let loaded = unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            identity.as_mut_ptr().cast(),
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    };
    if loaded == 0 {
        return Err(anyhow!("Windows file identity could not be read"));
    }

    // SAFETY: successful GetFileInformationByHandleEx initialized identity.
    let identity = unsafe { identity.assume_init() };
    if identity.FileId.Identifier.iter().all(|byte| *byte == 0) {
        return Err(anyhow!("Windows file identity is unavailable"));
    }
    Ok((identity.VolumeSerialNumber, identity.FileId.Identifier))
}

#[cfg(windows)]
fn final_windows_handle_path(handle: HANDLE) -> Result<String> {
    let mut capacity = 260_usize;
    loop {
        let mut buffer = vec![0_u16; capacity];
        // SAFETY: handle is valid and buffer is writable for its declared length.
        let length = unsafe {
            GetFinalPathNameByHandleW(handle, buffer.as_mut_ptr(), buffer.len() as u32, 0)
        };
        if length == 0 {
            return Err(anyhow!("Windows final pathname could not be read"));
        }
        if length as usize >= buffer.len() {
            capacity = length as usize + 1;
            if capacity > 32_768 {
                return Err(anyhow!("Windows final pathname is too long"));
            }
            continue;
        }

        let path = String::from_utf16(&buffer[..length as usize])
            .context("Windows final pathname is malformed")?;
        let path = if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
            format!(r"\\{unc}")
        } else if let Some(dos) = path.strip_prefix(r"\\?\") {
            dos.to_string()
        } else {
            path
        };
        return normalize_windows_install_bootstrap_path(&path)
            .context("Windows final pathname is invalid");
    }
}

#[cfg(unix)]
fn validate_unix_shim_invocation_witness(
    witness: &Path,
    running_executable: &Path,
) -> Result<PathBuf> {
    use std::path::Component;

    let command = witness
        .file_name()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("invocation witness has no command name"))?;
    if command == "." || command == ".." {
        return Err(anyhow!("invocation witness command name is invalid"));
    }

    let shims = witness
        .parent()
        .filter(|path| path.file_name() == Some(OsStr::new("shims")))
        .ok_or_else(|| anyhow!("invocation witness is outside A/shims"))?;
    let prefix = shims
        .parent()
        .ok_or_else(|| anyhow!("invocation witness has no install prefix"))?;

    if prefix
        .components()
        .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Err(anyhow!("invocation witness prefix is not canonical"));
    }

    ensure_unix_non_symlink_ancestors(witness)?;

    use nix::libc;
    // SAFETY: geteuid has no preconditions.
    let current_uid = unsafe { libc::geteuid() };
    let prefix_identity = secure_owned_directory(prefix, current_uid)?;
    let shims_identity = secure_owned_directory(shims, current_uid)?;
    let witness_meta =
        fs::symlink_metadata(witness).context("installed invocation witness is unavailable")?;

    if witness_meta.file_type().is_symlink() {
        if witness_meta.uid() != current_uid {
            return Err(anyhow!("installed symlink witness has the wrong owner"));
        }
        require_same_file_identity(witness, running_executable)?;
        revalidate_owned_directory(prefix, current_uid, prefix_identity)?;
        revalidate_owned_directory(shims, current_uid, shims_identity)?;
        revalidate_symlink(witness, current_uid, &witness_meta)?;
        return Ok(prefix.to_path_buf());
    }

    if !witness_meta.file_type().is_file()
        || witness_meta.uid() != current_uid
        || witness_meta.mode() & 0o022 != 0
    {
        return Err(anyhow!("physical invocation witness is not trusted"));
    }

    require_same_file_identity(witness, running_executable)?;
    revalidate_owned_directory(prefix, current_uid, prefix_identity)?;
    revalidate_owned_directory(shims, current_uid, shims_identity)?;
    revalidate_regular_file(witness, current_uid, &witness_meta)?;
    Ok(prefix.to_path_buf())
}

#[cfg(unix)]
fn lexically_normalize_absolute_invocation_path(path: &Path) -> Result<PathBuf> {
    use std::path::Component;

    if !path.is_absolute() {
        return Err(anyhow!("invocation pathname is not absolute"));
    }

    let mut normalized = PathBuf::from("/");
    for component in path.components() {
        match component {
            Component::RootDir => {}
            Component::Normal(value) => normalized.push(value),
            Component::CurDir | Component::ParentDir => {
                return Err(anyhow!("invocation pathname is not canonical"));
            }
            Component::Prefix(_) => return Err(anyhow!("invalid Unix invocation pathname")),
        }
    }
    Ok(normalized)
}

#[cfg(unix)]
fn ensure_unix_non_symlink_ancestors(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(anyhow!("invocation pathname is not absolute"));
    }

    for ancestor in path.ancestors().skip(1) {
        if ancestor.parent().is_none() {
            break;
        }
        let metadata = fs::symlink_metadata(ancestor).with_context(|| {
            format!("invocation ancestor {} is unavailable", ancestor.display())
        })?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err(anyhow!("invocation ancestor is not trusted"));
        }
    }

    Ok(())
}

#[cfg(unix)]
fn secure_owned_directory(path: &Path, uid: nix::libc::uid_t) -> Result<(u64, u64)> {
    let metadata =
        fs::symlink_metadata(path).context("installed witness directory is unavailable")?;
    if !metadata.file_type().is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != uid
        || metadata.mode() & 0o022 != 0
    {
        return Err(anyhow!("installed witness directory is not trusted"));
    }
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn revalidate_owned_directory(
    path: &Path,
    uid: nix::libc::uid_t,
    expected: (u64, u64),
) -> Result<()> {
    if secure_owned_directory(path, uid)? != expected {
        return Err(anyhow!("installed witness directory identity changed"));
    }
    Ok(())
}

#[cfg(unix)]
fn revalidate_symlink(path: &Path, uid: nix::libc::uid_t, expected: &fs::Metadata) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("installed symlink witness disappeared")?;
    if !metadata.file_type().is_symlink()
        || metadata.uid() != uid
        || metadata.dev() != expected.dev()
        || metadata.ino() != expected.ino()
    {
        return Err(anyhow!("installed symlink witness identity changed"));
    }
    Ok(())
}

#[cfg(unix)]
fn revalidate_regular_file(
    path: &Path,
    uid: nix::libc::uid_t,
    expected: &fs::Metadata,
) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("physical invocation witness disappeared")?;
    if !metadata.file_type().is_file()
        || metadata.uid() != uid
        || metadata.mode() & 0o022 != 0
        || metadata.dev() != expected.dev()
        || metadata.ino() != expected.ino()
    {
        return Err(anyhow!("physical invocation witness identity changed"));
    }
    Ok(())
}

#[cfg(unix)]
fn require_same_file_identity(left: &Path, right: &Path) -> Result<()> {
    let left = fs::metadata(left).context("failed to resolve installed invocation witness")?;
    let right = fs::metadata(right).context("failed to resolve running executable identity")?;
    if !left.file_type().is_file()
        || !right.file_type().is_file()
        || left.dev() != right.dev()
        || left.ino() != right.ino()
    {
        return Err(anyhow!(
            "installed invocation witness does not match running executable"
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn unix_account_record_by_uid(uid: u32) -> Result<(String, PathBuf)> {
    use nix::libc;

    let mut capacity = {
        // SAFETY: sysconf with a documented name has no pointer preconditions.
        let configured = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
        if configured > 0 {
            usize::try_from(configured).unwrap_or(16 * 1024)
        } else {
            16 * 1024
        }
    }
    .clamp(1024, 1024 * 1024);

    loop {
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::zeroed();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; capacity];
        // SAFETY: pwd/result point to writable storage and buffer is valid for its length.
        let code = unsafe {
            libc::getpwuid_r(
                uid as libc::uid_t,
                pwd.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if code == libc::ERANGE && capacity < 1024 * 1024 {
            capacity = (capacity * 2).min(1024 * 1024);
            continue;
        }
        if code != 0 || result.is_null() {
            return Err(anyhow!("current Unix account could not be resolved"));
        }

        // SAFETY: getpwuid_r succeeded and result points to initialized storage.
        let pwd = unsafe { pwd.assume_init() };
        if pwd.pw_name.is_null() || pwd.pw_dir.is_null() {
            return Err(anyhow!("current Unix account record is incomplete"));
        }

        // SAFETY: successful getpwuid_r returns NUL-terminated strings backed by buffer.
        let account = unsafe { std::ffi::CStr::from_ptr(pwd.pw_name) }
            .to_str()
            .context("Unix account name is not valid UTF-8")?
            .to_string();
        // SAFETY: same getpwuid_r guarantee as pw_name.
        let home = unsafe { std::ffi::CStr::from_ptr(pwd.pw_dir) }
            .to_str()
            .context("Unix account home is not valid UTF-8")?;
        if account.is_empty() || !home.starts_with('/') {
            return Err(anyhow!("current Unix account record is invalid"));
        }

        return Ok((account, PathBuf::from(home)));
    }
}

#[cfg(unix)]
fn unix_account_record_by_name(account: &str) -> Result<(String, u32, PathBuf)> {
    use nix::libc;

    let account_cstr = std::ffi::CString::new(account)
        .map_err(|_| anyhow!("current Unix account is malformed"))?;
    let mut capacity = {
        // SAFETY: sysconf with a documented name has no pointer preconditions.
        let configured = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
        if configured > 0 {
            usize::try_from(configured).unwrap_or(16 * 1024)
        } else {
            16 * 1024
        }
    }
    .clamp(1024, 1024 * 1024);

    loop {
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::zeroed();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; capacity];
        // SAFETY: pwd/result point to writable storage and buffer is valid for its length.
        let code = unsafe {
            libc::getpwnam_r(
                account_cstr.as_ptr(),
                pwd.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if code == libc::ERANGE && capacity < 1024 * 1024 {
            capacity = (capacity * 2).min(1024 * 1024);
            continue;
        }
        if code != 0 || result.is_null() {
            return Err(anyhow!("current Unix account could not be resolved"));
        }

        // SAFETY: getpwnam_r succeeded and result points to initialized storage.
        let pwd = unsafe { pwd.assume_init() };
        if pwd.pw_name.is_null() || pwd.pw_dir.is_null() || pwd.pw_uid == 0 {
            return Err(anyhow!("current Unix account record is incomplete"));
        }

        // SAFETY: successful getpwnam_r returns NUL-terminated strings backed by buffer.
        let resolved_account = unsafe { std::ffi::CStr::from_ptr(pwd.pw_name) }
            .to_str()
            .context("Unix account name is not valid UTF-8")?
            .to_string();
        // SAFETY: same getpwnam_r guarantee as pw_name.
        let home = unsafe { std::ffi::CStr::from_ptr(pwd.pw_dir) }
            .to_str()
            .context("Unix account home is not valid UTF-8")?;
        if resolved_account.is_empty() || !home.starts_with('/') {
            return Err(anyhow!("current Unix account record is invalid"));
        }

        return Ok((resolved_account, pwd.pw_uid, PathBuf::from(home)));
    }
}

#[cfg(target_os = "macos")]
fn unix_account_home_for_principal(principal: &PlatformPrincipalV1) -> Result<PathBuf> {
    let PlatformPrincipalV1::Unix { account, uid } = principal else {
        return Err(anyhow!("expected Unix install principal"));
    };

    let (name_account, name_uid, name_home) = unix_account_record_by_name(account)?;
    let (uid_account, uid_home) = unix_account_record_by_uid(*uid)?;
    if name_uid == 0
        || name_account != *account
        || uid_account != *account
        || name_uid != *uid
        || name_home != uid_home
    {
        return Err(anyhow!(
            "install principal account does not round-trip through the account database"
        ));
    }

    Ok(name_home)
}

#[cfg(target_os = "macos")]
fn observe_default_lima_platform_bootstrap_mapping(
    host_carrier: &InstallBootstrapContextCarrierV1,
) -> Result<PlatformBootstrapMappingV1> {
    host_carrier
        .validate()
        .context("invalid install bootstrap carrier")?;
    if env::var_os("SUBSTRATE_WORLD_SOCKET").is_some() {
        return Err(anyhow!(
            "macOS platform telemetry requires the authenticated Lima mapping; SUBSTRATE_WORLD_SOCKET is non-authoritative"
        ));
    }

    let host_account_home =
        unix_account_home_for_principal(&host_carrier.context.intended_host_principal)?;
    let host_control_root = host_account_home.join(".lima");
    let host_socket = PathBuf::from(&host_carrier.context.selected_host_prefix)
        .join("sock")
        .join("agent.sock");
    let limactl_path = resolved_limactl_path()?;
    let output = Command::new(&limactl_path)
        .env("HOME", &host_account_home)
        .env("LIMA_HOME", &host_control_root)
        .args([
            "shell",
            "--workdir=/",
            DEFAULT_LIMA_VM_NAME,
            "bash",
            "-lc",
            "set -euo pipefail\nmachine_id=\"$(tr -d '\\n' </etc/machine-id)\"\naccount=\"$(id -un)\"\nuid=\"$(id -u)\"\npasswd_by_name=\"$(getent passwd \"$account\")\"\npasswd_by_uid=\"$(getent passwd \"$uid\")\"\nprintf 'machine_id=%s\\n' \"$machine_id\"\nprintf 'account=%s\\n' \"$account\"\nprintf 'uid=%s\\n' \"$uid\"\nprintf 'passwd_by_name=%s\\n' \"$passwd_by_name\"\nprintf 'passwd_by_uid=%s\\n' \"$passwd_by_uid\"\n",
        ])
        .output()
        .context("failed to observe Lima guest identity")?;
    if !output.status.success() {
        return Err(anyhow!(
            "unable to observe Lima guest identity\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim(),
        ));
    }

    let (guest_machine_id, guest_account, guest_uid, guest_home) =
        parse_guest_identity_output("Lima", &output.stdout)?;
    PlatformBootstrapMappingV1::new_lima(
        host_carrier,
        DEFAULT_LIMA_VM_NAME,
        &guest_machine_id,
        host_control_root
            .to_str()
            .ok_or_else(|| anyhow!("macOS Lima control root is not valid UTF-8"))?,
        &format!("{guest_home}/.substrate"),
        &guest_account,
        guest_uid,
        host_socket
            .to_str()
            .ok_or_else(|| anyhow!("macOS host socket projection is not valid UTF-8"))?,
        CANONICAL_GUEST_SOCKET_PATH,
    )
    .map_err(anyhow::Error::from)
    .context("failed to construct canonical Lima platform bootstrap mapping")
}

#[cfg(target_os = "macos")]
fn resolved_limactl_path() -> Result<PathBuf> {
    if let Some(test_override) = env::var_os("SUBSTRATE_TEST_LIMACTL_PATH") {
        return Ok(PathBuf::from(test_override));
    }

    for candidate in [
        "/opt/homebrew/bin/limactl",
        "/usr/local/bin/limactl",
        "/opt/homebrew/sbin/limactl",
        "/usr/local/sbin/limactl",
    ] {
        let path = Path::new(candidate);
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
    }

    Err(anyhow!(
        "limactl not found. Install Lima with: brew install lima"
    ))
}

#[cfg(target_os = "macos")]
fn parse_guest_identity_output(
    label: &str,
    stdout: &[u8],
) -> Result<(String, String, u32, String)> {
    let stdout = String::from_utf8(stdout.to_vec())
        .with_context(|| format!("{label} guest output is not valid UTF-8"))?;
    let mut machine_id = None;
    let mut account = None;
    let mut uid = None;
    let mut passwd_by_name = None;
    let mut passwd_by_uid = None;
    for line in stdout.lines() {
        let trimmed = line.trim_matches('\r');
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        match key {
            "machine_id" => machine_id = Some(value.to_string()),
            "account" => account = Some(value.to_string()),
            "uid" => uid = Some(value.to_string()),
            "passwd_by_name" => passwd_by_name = Some(value.to_string()),
            "passwd_by_uid" => passwd_by_uid = Some(value.to_string()),
            _ => {}
        }
    }

    let machine_id = machine_id.ok_or_else(|| anyhow!("{label} guest machine ID is missing"))?;
    if machine_id.len() != 32
        || !machine_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(anyhow!("{label} guest machine ID is malformed"));
    }

    let account = account.ok_or_else(|| anyhow!("{label} guest account is missing"))?;
    if account.is_empty()
        || account
            .chars()
            .any(|ch| matches!(ch, '\0' | '\n' | '\r' | ':' | '/'))
    {
        return Err(anyhow!("{label} guest account is malformed"));
    }

    let uid = uid
        .ok_or_else(|| anyhow!("{label} guest UID is missing"))?
        .parse::<u32>()
        .with_context(|| format!("{label} guest UID is malformed"))?;
    let passwd_by_name =
        passwd_by_name.ok_or_else(|| anyhow!("{label} passwd entry is missing"))?;
    let passwd_by_uid =
        passwd_by_uid.ok_or_else(|| anyhow!("{label} passwd UID entry is missing"))?;
    if passwd_by_name != passwd_by_uid {
        return Err(anyhow!(
            "{label} account-database lookup by name and UID did not round-trip"
        ));
    }

    let passwd_fields = passwd_by_name.split(':').collect::<Vec<_>>();
    if passwd_fields.len() < 7 {
        return Err(anyhow!("{label} passwd entry is malformed"));
    }
    if passwd_fields[0] != account {
        return Err(anyhow!(
            "{label} passwd entry account does not match the active guest account"
        ));
    }
    if passwd_fields[2]
        .parse::<u32>()
        .with_context(|| format!("{label} passwd UID is malformed"))?
        != uid
    {
        return Err(anyhow!(
            "{label} passwd entry UID does not match the active guest UID"
        ));
    }

    let guest_home = transport_api_types::normalize_unix_install_bootstrap_path(passwd_fields[5])
        .with_context(|| format!("{label} passwd home directory is invalid"))?;
    if guest_home.starts_with("/mnt/") {
        return Err(anyhow!(
            "{label} passwd home directory may not resolve to a host-mounted path"
        ));
    }

    Ok((machine_id, account, uid, guest_home))
}

#[cfg(windows)]
fn observe_default_wsl_platform_bootstrap_mapping(
    host_carrier: &InstallBootstrapContextCarrierV1,
) -> Result<PlatformBootstrapMappingV1> {
    host_carrier
        .validate()
        .context("invalid install bootstrap carrier")?;
    let PlatformPrincipalV1::Windows { sid, .. } = &host_carrier.context.intended_host_principal
    else {
        return Err(anyhow!(
            "Windows platform telemetry requires a Windows install principal"
        ));
    };

    let (current_principal, local_app_data) = current_windows_principal_and_known_folder()?;
    if current_principal != host_carrier.context.intended_host_principal {
        return Err(anyhow!(
            "Windows platform telemetry carrier does not match the current Windows principal"
        ));
    }

    let default_pipe = normalize_windows_pipe_path(DEFAULT_WSL_PIPE)
        .map_err(anyhow::Error::from)
        .context("default Windows platform telemetry pipe path is invalid")?;
    if let Some(conflicting_pipe) = env::var_os("SUBSTRATE_FORWARDER_PIPE") {
        let conflicting_pipe = conflicting_pipe
            .to_str()
            .ok_or_else(|| anyhow!("SUBSTRATE_FORWARDER_PIPE is not valid UTF-8"))?;
        let conflicting_pipe = normalize_windows_pipe_path(conflicting_pipe)
            .map_err(anyhow::Error::from)
            .context("SUBSTRATE_FORWARDER_PIPE is invalid")?;
        if conflicting_pipe != default_pipe {
            return Err(anyhow!(
                "Windows platform telemetry pipe projection conflicts with the authenticated mapping"
            ));
        }
    }

    let (exact_distro_name, guest_machine_id, guest_account, guest_uid, guest_home) =
        observe_wsl_mapping_v1(DEFAULT_WSL_DISTRO)?;
    let scope =
        WindowsForwarderScopeV1::derive(sid, &exact_distro_name, &guest_machine_id, &default_pipe)
            .context("Windows platform telemetry forwarder scope is invalid")?;
    let control_root = normalize_windows_install_bootstrap_path(&format!(
        r"{}\Substrate\forwarder\{}",
        local_app_data.display(),
        scope.0
    ))
    .context("Windows platform telemetry control root is invalid")?;

    PlatformBootstrapMappingV1::new_wsl(
        host_carrier,
        &exact_distro_name,
        &guest_machine_id,
        &control_root,
        &format!("{guest_home}/.substrate"),
        &guest_account,
        guest_uid,
        &default_pipe,
        CANONICAL_WSL_GUEST_SOCKET_PATH,
    )
    .map_err(anyhow::Error::from)
    .context("failed to construct canonical WSL platform bootstrap mapping")
}

#[cfg(windows)]
fn observe_wsl_mapping_v1(
    declared_distro_name: &str,
) -> Result<(String, String, String, u32, String)> {
    let wsl_executable = absolute_windows_wsl_executable_path()?;
    let list_names = |args: &[&str], context: &str| -> Result<String> {
        let output = Command::new(&wsl_executable)
            .args(args)
            .output()
            .with_context(|| {
                format!(
                    "failed to observe WSL state via `wsl.exe {}`",
                    args.join(" ")
                )
            })?;
        if !output.status.success() {
            return Err(anyhow!(
                "{context}\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim(),
            ));
        }
        String::from_utf8(output.stdout).context("WSL observation produced non-UTF-8 output")
    };

    let registered = list_names(&["-l", "-q"], "unable to enumerate registered WSL distros")?;
    let registered_matches = registered
        .lines()
        .map(|line| {
            line.trim_matches(|ch| ch == '\r' || ch == '\u{feff}')
                .trim()
        })
        .filter(|line| !line.is_empty())
        .filter(|line| line.eq_ignore_ascii_case(declared_distro_name))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if registered_matches.is_empty() {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` is not registered"
        ));
    }
    if registered_matches.len() != 1 {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` matched multiple registered distros"
        ));
    }
    let exact_distro_name = registered_matches[0].clone();

    let running = list_names(&["-l", "-v"], "unable to enumerate running WSL distros")?;
    let ordered_names = registered_matches
        .iter()
        .cloned()
        .chain([exact_distro_name.clone()])
        .collect::<Vec<_>>();
    let running_matches = running
        .lines()
        .map(|line| line.trim_matches('\r'))
        .filter_map(|line| {
            let mut trimmed = line.trim_start();
            if trimmed.is_empty()
                || trimmed.starts_with("NAME")
                || trimmed.starts_with("Windows Subsystem for Linux")
            {
                return None;
            }
            if let Some(stripped) = trimmed.strip_prefix('*') {
                trimmed = stripped.trim_start();
            }
            let matched_name = ordered_names
                .iter()
                .filter(|name| trimmed.starts_with(name.as_str()))
                .max_by_key(|name| name.len())?;
            let remainder = &trimmed[matched_name.len()..];
            let state = remainder
                .split_once(char::is_whitespace)
                .map(|_| remainder.trim_start().split_whitespace().next())
                .flatten()?;
            state
                .eq_ignore_ascii_case("Running")
                .then(|| matched_name.to_string())
        })
        .filter(|name| name.eq_ignore_ascii_case(declared_distro_name))
        .collect::<Vec<_>>();
    if running_matches.is_empty() {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` is not running"
        ));
    }
    if running_matches.len() != 1 || running_matches[0] != exact_distro_name {
        return Err(anyhow!(
            "declared WSL distro `{declared_distro_name}` resolved to an ambiguous running spelling"
        ));
    }

    let output = Command::new(&wsl_executable)
        .args([
            "-d",
            &exact_distro_name,
            "--",
            "bash",
            "-lc",
            "set -euo pipefail\nmachine_id=\"$(tr -d '\\n' </etc/machine-id)\"\naccount=\"$(id -un)\"\nuid=\"$(id -u)\"\npasswd_by_name=\"$(getent passwd \"$account\")\"\npasswd_by_uid=\"$(getent passwd \"$uid\")\"\nprintf 'machine_id=%s\\n' \"$machine_id\"\nprintf 'account=%s\\n' \"$account\"\nprintf 'uid=%s\\n' \"$uid\"\nprintf 'passwd_by_name=%s\\n' \"$passwd_by_name\"\nprintf 'passwd_by_uid=%s\\n' \"$passwd_by_uid\"\n",
        ])
        .output()
        .context("failed to observe WSL guest identity")?;
    if !output.status.success() {
        return Err(anyhow!(
            "unable to observe WSL guest identity\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim(),
        ));
    }

    let stdout =
        String::from_utf8(output.stdout).context("WSL guest identity output is not valid UTF-8")?;
    let mut machine_id = None;
    let mut account = None;
    let mut uid = None;
    let mut passwd_by_name = None;
    let mut passwd_by_uid = None;
    for line in stdout.lines() {
        let trimmed = line.trim_matches('\r');
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        match key {
            "machine_id" => machine_id = Some(value.to_string()),
            "account" => account = Some(value.to_string()),
            "uid" => uid = Some(value.to_string()),
            "passwd_by_name" => passwd_by_name = Some(value.to_string()),
            "passwd_by_uid" => passwd_by_uid = Some(value.to_string()),
            _ => {}
        }
    }

    let machine_id = machine_id.ok_or_else(|| anyhow!("WSL guest machine ID is missing"))?;
    if machine_id.len() != 32
        || !machine_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(anyhow!("WSL guest machine ID is malformed"));
    }

    let account = account.ok_or_else(|| anyhow!("WSL guest account is missing"))?;
    if account.is_empty()
        || account
            .chars()
            .any(|ch| matches!(ch, '\0' | '\n' | '\r' | ':' | '/'))
    {
        return Err(anyhow!("WSL guest account is malformed"));
    }

    let uid = uid
        .ok_or_else(|| anyhow!("WSL guest UID is missing"))?
        .parse::<u32>()
        .context("WSL guest UID is malformed")?;
    let passwd_by_name = passwd_by_name.ok_or_else(|| anyhow!("WSL passwd entry is missing"))?;
    let passwd_by_uid = passwd_by_uid.ok_or_else(|| anyhow!("WSL passwd UID entry is missing"))?;
    if passwd_by_name != passwd_by_uid {
        return Err(anyhow!(
            "WSL account-database lookup by name and UID did not round-trip"
        ));
    }

    let passwd_fields = passwd_by_name.split(':').collect::<Vec<_>>();
    if passwd_fields.len() < 7 {
        return Err(anyhow!("WSL passwd entry is malformed"));
    }
    if passwd_fields[0] != account {
        return Err(anyhow!(
            "WSL passwd entry account does not match the active guest account"
        ));
    }
    if passwd_fields[2]
        .parse::<u32>()
        .context("WSL passwd UID is malformed")?
        != uid
    {
        return Err(anyhow!(
            "WSL passwd entry UID does not match the active guest UID"
        ));
    }

    let normalized_home =
        transport_api_types::normalize_unix_install_bootstrap_path(passwd_fields[5])
            .context("WSL passwd home directory is invalid")?;
    if normalized_home.starts_with("/mnt/") {
        return Err(anyhow!(
            "WSL passwd home directory may not resolve to a host-mounted path"
        ));
    }

    Ok((exact_distro_name, machine_id, account, uid, normalized_home))
}

#[cfg(windows)]
fn current_windows_principal_and_known_folder() -> Result<(PlatformPrincipalV1, PathBuf)> {
    let token = open_current_windows_token()?;
    let principal = windows_principal_from_token(token.0)?;
    let local_app_data = windows_known_folder_from_token(token.0)?;
    Ok((principal, local_app_data))
}

/// Build clean search path excluding shim directory.
pub fn build_clean_search_path(
    shim_dir: &Path,
    original_path: Option<String>,
) -> Result<Vec<PathBuf>> {
    let path_str = original_path
        .or_else(|| env::var("PATH").ok())
        .ok_or_else(|| anyhow!("No PATH or SHIM_ORIGINAL_PATH found"))?;

    let separator = if cfg!(windows) { ';' } else { ':' };

    fn is_good_dir(path: &str) -> bool {
        let path = Path::new(path);
        path.is_absolute() && path.is_dir()
    }

    let mut seen = HashSet::new();
    let paths: Vec<PathBuf> = path_str
        .split(separator)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_end_matches('/'))
        .filter(|value| !Path::new(value).starts_with(shim_dir))
        .filter(|value| is_good_dir(value))
        .filter(|value| seen.insert(value.to_string()))
        .map(PathBuf::from)
        .collect();

    if paths.is_empty() {
        return Err(anyhow!("No valid search paths found after filtering"));
    }

    Ok(paths)
}

/// Merge the live PATH with the stored SHIM_ORIGINAL_PATH so that runtime
/// managers that rewrite PATH remain visible to shims.
pub fn merge_path_sources(original_path: Option<String>) -> Option<String> {
    let mut sources = Vec::new();

    if let Ok(current) = env::var("PATH") {
        if !current.is_empty() {
            sources.push(current);
        }
    }

    if let Some(original) = original_path {
        if !original.is_empty() {
            sources.push(original);
        }
    }

    match sources.len() {
        0 => None,
        1 => sources.into_iter().next(),
        _ => {
            let separator = if cfg!(windows) { ';' } else { ':' };
            let separator = separator.to_string();
            Some(sources.join(&separator))
        }
    }
}

fn is_disabled_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "0" | "false" | "off" | "disabled"
    )
}

/// True when pass-through mode was requested via SUBSTRATE world flags.
pub fn world_disabled() -> bool {
    matches!(env::var(SUBSTRATE_WORLD_VAR).as_deref(), Ok("disabled"))
        || env::var(SUBSTRATE_WORLD_ENABLED_VAR)
            .map(|value| is_disabled_flag(&value))
            .unwrap_or(false)
}

/// True when shim should enable world-aware policy + telemetry features.
pub fn world_features_enabled() -> bool {
    if world_disabled() {
        return false;
    }
    env::var(SUBSTRATE_WORLD_VAR)
        .map(|value| value == "enabled")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[cfg(unix)]
    use serial_test::serial;
    #[cfg(unix)]
    use std::os::unix::fs::{symlink, PermissionsExt};

    #[cfg(unix)]
    struct EnvGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    #[cfg(unix)]
    impl EnvGuard {
        fn set(key: &'static str, value: impl Into<OsString>) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value.into());
            Self { key, previous }
        }
    }

    #[cfg(unix)]
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.previous.take() {
                env::set_var(self.key, value);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[cfg(unix)]
    fn install_witness(command: &str) -> (TempDir, PathBuf, PathBuf, PathBuf) {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("prefix");
        let shims = prefix.join("shims");
        fs::create_dir_all(&shims).unwrap();
        let running = temp.path().join("substrate-shim");
        fs::copy(std::env::current_exe().unwrap(), &running).unwrap();
        let mut running_perms = fs::metadata(&running).unwrap().permissions();
        running_perms.set_mode(0o755);
        fs::set_permissions(&running, running_perms).unwrap();

        let witness = shims.join(command);
        symlink(&running, &witness).unwrap();
        (temp, prefix, running, witness)
    }

    #[cfg(unix)]
    fn prepend_path(entries: &[&Path]) -> (EnvGuard, String) {
        let previous = env::var("PATH").unwrap_or_default();
        let joined = entries
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(":");
        let combined = if previous.is_empty() {
            joined
        } else {
            format!("{joined}:{previous}")
        };
        (EnvGuard::set("PATH", combined.clone()), combined)
    }

    #[test]
    fn test_clean_search_path_filters_shim_dir() {
        let temp = TempDir::new().unwrap();
        let shim_dir = temp.path().join("shims");
        fs::create_dir(&shim_dir).unwrap();

        #[cfg(windows)]
        {
            let a = temp.path().join("A");
            let b = temp.path().join("B");
            fs::create_dir(&a).unwrap();
            fs::create_dir(&b).unwrap();
            let original_path = format!("{};{};{}", a.display(), shim_dir.display(), b.display());
            let paths = build_clean_search_path(&shim_dir, Some(original_path)).unwrap();
            assert_eq!(paths, vec![a, b]);
        }

        #[cfg(unix)]
        {
            let original_path = format!("/usr/bin:{}:/bin", shim_dir.display());
            let paths = build_clean_search_path(&shim_dir, Some(original_path)).unwrap();
            assert_eq!(paths.len(), 2);
            assert_eq!(paths[0], PathBuf::from("/usr/bin"));
            assert_eq!(paths[1], PathBuf::from("/bin"));
        }
    }

    #[test]
    fn test_path_deduplication() {
        let temp = TempDir::new().unwrap();
        let shim_dir = temp.path().join("shims");
        fs::create_dir(&shim_dir).unwrap();

        #[cfg(windows)]
        {
            let a = temp.path().join("A");
            let b = temp.path().join("B");
            let c = temp.path().join("C");
            fs::create_dir(&a).unwrap();
            fs::create_dir(&b).unwrap();
            fs::create_dir(&c).unwrap();
            let original_path = format!(
                "{};{};{};{};{}",
                a.display(),
                b.display(),
                a.display(),
                c.display(),
                b.display()
            );
            let paths = build_clean_search_path(&shim_dir, Some(original_path)).unwrap();
            assert_eq!(paths, vec![a, b, c]);
        }

        #[cfg(unix)]
        {
            let a = temp.path().join("a");
            let b = temp.path().join("b");
            let c = temp.path().join("c");
            fs::create_dir(&a).unwrap();
            fs::create_dir(&b).unwrap();
            fs::create_dir(&c).unwrap();
            let original_path = format!(
                "{}:{}:{}:{}:{}",
                a.display(),
                b.display(),
                a.display(),
                c.display(),
                b.display()
            );
            let paths = build_clean_search_path(&shim_dir, Some(original_path)).unwrap();
            assert_eq!(paths, vec![a, b, c]);
        }
    }

    #[test]
    fn test_safe_call_stack() {
        assert_eq!(build_safe_call_stack("", "npm"), "npm");
        assert_eq!(build_safe_call_stack("npm", "node"), "npm,node");
        assert_eq!(build_safe_call_stack("npm,node", "node"), "npm,node");
        assert_eq!(build_safe_call_stack("npm", "npm"), "npm");

        let stack = build_safe_call_stack("", "A");
        let stack = build_safe_call_stack(&stack, "B");
        let stack = build_safe_call_stack(&stack, "A");
        let stack = build_safe_call_stack(&stack, "B");
        assert_eq!(stack, "A,B,A,B");

        let mut stack = String::new();
        for i in 1..=10 {
            stack = build_safe_call_stack(&stack, &format!("cmd{i}"));
        }
        assert!(stack.starts_with("..."));
        assert!(stack.contains("cmd10"));
        let parts: Vec<&str> = stack.split(',').collect();
        assert_eq!(parts.len(), 8);
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn resolve_install_bootstrap_context_ignores_conflicting_ambient_home() {
        let (_temp, prefix, running, witness) = install_witness("git");
        let _home_guard = EnvGuard::set("SUBSTRATE_HOME", "/tmp/ambient-substrate");
        let _root_guard = EnvGuard::set("SUBSTRATE_ROOT", "/tmp/ambient-substrate");
        let carrier =
            resolve_install_bootstrap_context_from_invocation(&witness, &running).unwrap();
        assert_eq!(
            carrier.context.selected_host_prefix,
            prefix.display().to_string()
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn resolve_install_bootstrap_context_rejects_conflicting_inherited_projection() {
        let (_temp, _prefix, running, witness) = install_witness("git");
        let _guard = EnvGuard::set(
            INSTALL_BOOTSTRAP_CONTEXT_ENV,
            "not-a-canonical-install-bootstrap-carrier",
        );
        let err =
            resolve_install_bootstrap_context_from_invocation(&witness, &running).unwrap_err();
        assert!(err
            .to_string()
            .contains("install bootstrap environment projection"));
    }

    #[cfg(unix)]
    #[test]
    fn current_platform_principal_round_trips_on_unix() {
        let principal = current_platform_principal_v1().unwrap();
        let PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        assert!(!account.is_empty());
        assert!(uid > 0);
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn resolve_invoked_path_accepts_unique_bare_witness() {
        let (_temp, _prefix, running, witness) = install_witness("npm");
        let unrelated = TempDir::new().unwrap();
        fs::write(unrelated.path().join("npm"), "#!/bin/sh\nexit 0\n").unwrap();
        let (_path_guard, _path) = prepend_path(&[unrelated.path(), witness.parent().unwrap()]);

        let resolved = resolve_invoked_path(OsStr::new("npm"), &running).unwrap();
        assert_eq!(resolved, witness);
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn resolve_invoked_path_rejects_zero_bare_candidates() {
        let temp = TempDir::new().unwrap();
        let running = temp.path().join("substrate-shim");
        fs::copy(std::env::current_exe().unwrap(), &running).unwrap();
        let empty_dir = temp.path().join("bin");
        fs::create_dir_all(&empty_dir).unwrap();
        let (_path_guard, _path) = prepend_path(&[&empty_dir]);

        let err = resolve_invoked_path(OsStr::new("git"), &running).unwrap_err();
        assert!(err.to_string().contains("exactly one installed witness"));
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn resolve_invoked_path_rejects_multiple_bare_candidates_regardless_of_path_order() {
        let (_temp_a, _prefix_a, running, witness_a) = install_witness("node");
        let temp_b = TempDir::new().unwrap();
        let prefix_b = temp_b.path().join("prefix");
        let shims_b = prefix_b.join("shims");
        fs::create_dir_all(&shims_b).unwrap();
        let witness_b = shims_b.join("node");
        symlink(&running, &witness_b).unwrap();

        let (_path_guard, _path) =
            prepend_path(&[witness_b.parent().unwrap(), witness_a.parent().unwrap()]);
        let err = resolve_invoked_path(OsStr::new("node"), &running).unwrap_err();
        assert!(err.to_string().contains("multiple installed witnesses"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_invoked_path_rejects_wrong_target() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("prefix");
        let shims = prefix.join("shims");
        fs::create_dir_all(&shims).unwrap();
        let running = temp.path().join("substrate-shim");
        let other = temp.path().join("other-shim");
        fs::copy(std::env::current_exe().unwrap(), &running).unwrap();
        fs::copy(std::env::current_exe().unwrap(), &other).unwrap();
        symlink(&other, shims.join("git")).unwrap();

        let err = resolve_invoked_path(shims.join("git").as_os_str(), &running).unwrap_err();
        assert!(err
            .to_string()
            .contains("does not match running executable"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_invoked_path_rejects_noncanonical_relative_path() {
        let (_temp, _prefix, running, witness) = install_witness("cargo");
        let cwd = witness.parent().unwrap().parent().unwrap().to_path_buf();
        let _cwd_guard = EnvGuard::set("PWD", cwd.display().to_string());
        let previous = env::current_dir().unwrap();
        env::set_current_dir(&cwd).unwrap();
        let err = resolve_invoked_path(OsStr::new("shims/../shims/cargo"), &running).unwrap_err();
        env::set_current_dir(previous).unwrap();
        assert!(err.to_string().contains("not canonical"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_install_bootstrap_context_rejects_empty_command_shape() {
        let (_temp, _prefix, running, witness) = install_witness("cargo");
        let err =
            resolve_install_bootstrap_context_from_invocation(witness.parent().unwrap(), &running)
                .unwrap_err();
        assert!(
            err.to_string().contains("outside A/shims")
                || err.to_string().contains("no command name")
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolve_invoked_path_rejects_symlink_ancestor() {
        let temp = TempDir::new().unwrap();
        let real_prefix = temp.path().join("real-prefix");
        let shims = real_prefix.join("shims");
        fs::create_dir_all(&shims).unwrap();
        let alias_prefix = temp.path().join("alias-prefix");
        symlink(&real_prefix, &alias_prefix).unwrap();
        let running = temp.path().join("substrate-shim");
        fs::copy(std::env::current_exe().unwrap(), &running).unwrap();
        symlink(&running, shims.join("git")).unwrap();

        let err =
            resolve_invoked_path(alias_prefix.join("shims/git").as_os_str(), &running).unwrap_err();
        assert!(err.to_string().contains("ancestor"));
    }
}
