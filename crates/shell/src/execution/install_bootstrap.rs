use anyhow::{anyhow, Context, Result};
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
#[cfg(any(unix, all(test, windows)))]
use std::fs;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformPrincipalV1};
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

pub(crate) const INSTALL_BOOTSTRAP_CONTEXT_ENV: &str = "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1";
pub(crate) const INSTALL_BOOTSTRAP_COMMITMENT_ENV: &str =
    "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT";
pub(crate) const INSTALL_BOOTSTRAP_ACCOUNT_ENV: &str = "SUBSTRATE_INSTALL_PRIMARY_USER";
#[cfg(unix)]
pub(crate) const INSTALL_BOOTSTRAP_UID_ENV: &str = "SUBSTRATE_INSTALL_PRIMARY_UID";

#[cfg(unix)]
pub(crate) fn construct_unix_install_bootstrap_context(
    declared_prefix: Option<&Path>,
    argv0: &OsStr,
    running_executable: &Path,
    path_env: Option<&OsStr>,
    cwd: &Path,
) -> Result<InstallBootstrapContextCarrierV1> {
    let prefix = match declared_prefix {
        Some(prefix) => prefix.to_path_buf(),
        None => {
            resolve_unix_install_prefix_from_invocation(argv0, running_executable, path_env, cwd)?
        }
    };
    let prefix = prefix
        .to_str()
        .ok_or_else(|| anyhow!("install prefix is not valid UTF-8"))?;
    let (principal, _) = current_unix_principal_and_home()?;
    let PlatformPrincipalV1::Unix { account, uid } = principal else {
        return Err(anyhow!("current principal is not Unix"));
    };
    let context = transport_api_types::InstallBootstrapContextV1::new_unix(prefix, &account, uid)
        .context("invalid Unix install bootstrap context")?;
    InstallBootstrapContextCarrierV1::from_context(context)
        .context("failed to commit Unix install bootstrap context")
}

#[cfg(unix)]
pub(crate) fn decode_and_bind_unix_install_bootstrap_context(
    encoded: &str,
    declared_prefix: Option<&Path>,
) -> Result<InstallBootstrapContextCarrierV1> {
    let carrier = InstallBootstrapContextCarrierV1::decode(encoded)
        .context("invalid install bootstrap carrier")?;
    bind_unix_install_bootstrap_context(&carrier)?;
    if let Some(prefix) = declared_prefix {
        let raw = prefix
            .to_str()
            .ok_or_else(|| anyhow!("declared install prefix is not valid UTF-8"))?;
        let normalized = transport_api_types::normalize_unix_install_bootstrap_path(raw)
            .context("invalid declared install prefix")?;
        if normalized != carrier.context.selected_host_prefix {
            return Err(anyhow!("declared install prefix does not match carrier"));
        }
    }
    reject_conflicting_install_bootstrap_projections(&carrier, encoded, |key| {
        std::env::var_os(key)
    })?;
    Ok(carrier)
}

#[cfg(unix)]
pub(crate) fn bind_unix_install_bootstrap_context(
    carrier: &InstallBootstrapContextCarrierV1,
) -> Result<()> {
    carrier
        .validate()
        .context("invalid install bootstrap carrier")?;
    let (current, _) = current_unix_principal_and_home()?;
    if carrier.context.intended_host_principal != current {
        return Err(anyhow!(
            "install bootstrap principal does not match current Unix principal"
        ));
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn current_unix_principal_and_home() -> Result<(PlatformPrincipalV1, PathBuf)> {
    // SAFETY: geteuid has no preconditions.
    let uid = unsafe { libc::geteuid() };
    if uid == 0 {
        return Err(anyhow!(
            "effective UID 0 requires the R2-2 intended-principal boundary"
        ));
    }
    let (account, home) = lookup_unix_account_by_uid(uid)?;
    Ok((PlatformPrincipalV1::Unix { account, uid }, home))
}

#[cfg(unix)]
pub(crate) fn unix_account_home_for_principal(principal: &PlatformPrincipalV1) -> Result<PathBuf> {
    let PlatformPrincipalV1::Unix { account, uid } = principal else {
        return Err(anyhow!("expected Unix install principal"));
    };
    let (resolved_account, home) = lookup_unix_account_by_uid(*uid)?;
    if &resolved_account != account {
        return Err(anyhow!("Unix install principal no longer round-trips"));
    }
    Ok(home)
}

pub(crate) fn expected_install_bootstrap_projections(
    carrier: &InstallBootstrapContextCarrierV1,
    encoded: &str,
) -> Result<HashMap<&'static str, OsString>> {
    carrier
        .validate()
        .context("invalid install bootstrap carrier")?;
    if carrier.encode().context("failed to encode carrier")? != encoded {
        return Err(anyhow!(
            "install bootstrap carrier spelling is noncanonical"
        ));
    }
    let mut projections = HashMap::from([
        (
            "SUBSTRATE_HOME",
            OsString::from(&carrier.context.host_substrate_home),
        ),
        (
            "SUBSTRATE_ROOT",
            OsString::from(&carrier.context.host_substrate_root),
        ),
        (
            INSTALL_BOOTSTRAP_COMMITMENT_ENV,
            OsString::from(&carrier.host_context_commitment),
        ),
    ]);
    #[cfg(unix)]
    match &carrier.context.intended_host_principal {
        PlatformPrincipalV1::Unix { account, uid } => {
            projections.insert(INSTALL_BOOTSTRAP_ACCOUNT_ENV, OsString::from(account));
            projections.insert(INSTALL_BOOTSTRAP_UID_ENV, OsString::from(uid.to_string()));
        }
        PlatformPrincipalV1::Windows { .. } => {
            return Err(anyhow!("expected Unix install principal"));
        }
    }
    #[cfg(windows)]
    match &carrier.context.intended_host_principal {
        PlatformPrincipalV1::Windows { account, .. } => {
            projections.insert(INSTALL_BOOTSTRAP_ACCOUNT_ENV, OsString::from(account));
        }
        PlatformPrincipalV1::Unix { .. } => {
            return Err(anyhow!("expected Windows install principal"));
        }
    }
    projections.insert(INSTALL_BOOTSTRAP_CONTEXT_ENV, OsString::from(encoded));
    Ok(projections)
}

pub(crate) fn validate_install_bootstrap_projections<F>(
    carrier: &InstallBootstrapContextCarrierV1,
    encoded: &str,
    mut lookup: F,
) -> Result<()>
where
    F: FnMut(&str) -> Option<OsString>,
{
    for (key, expected) in expected_install_bootstrap_projections(carrier, encoded)? {
        if lookup(key).as_deref() != Some(expected.as_os_str()) {
            return Err(anyhow!(
                "install bootstrap environment projection is missing or conflicting"
            ));
        }
    }
    Ok(())
}

pub(crate) fn reject_conflicting_install_bootstrap_projections<F>(
    carrier: &InstallBootstrapContextCarrierV1,
    encoded: &str,
    mut lookup: F,
) -> Result<()>
where
    F: FnMut(&str) -> Option<OsString>,
{
    for (key, expected) in expected_install_bootstrap_projections(carrier, encoded)? {
        if lookup(key).is_some_and(|actual| actual != expected) {
            return Err(anyhow!(
                "install bootstrap environment projection is conflicting"
            ));
        }
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn install_bootstrap_projections(
    carrier: &InstallBootstrapContextCarrierV1,
) -> Result<String> {
    bind_unix_install_bootstrap_context(carrier)?;
    let encoded = carrier.encode().context("failed to encode carrier")?;
    for (key, value) in expected_install_bootstrap_projections(carrier, &encoded)? {
        std::env::set_var(key, value);
    }
    Ok(encoded)
}

#[cfg(unix)]
pub(crate) fn checked_install_bootstrap_context_from_projections(
) -> Result<InstallBootstrapContextCarrierV1> {
    let encoded = std::env::var(INSTALL_BOOTSTRAP_CONTEXT_ENV)
        .context("checked install bootstrap projection is missing")?;
    let carrier = InstallBootstrapContextCarrierV1::decode(&encoded)
        .context("checked install bootstrap projection is invalid")?;
    bind_unix_install_bootstrap_context(&carrier)?;
    validate_install_bootstrap_projections(&carrier, &encoded, |key| std::env::var_os(key))?;
    Ok(carrier)
}

#[cfg(unix)]
fn lookup_unix_account_by_uid(uid: u32) -> Result<(String, PathBuf)> {
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
        // SAFETY: pwd/result point to writable storage and buffer is valid for its declared length.
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
        // SAFETY: getpwuid_r succeeded and result points to the initialized pwd storage.
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
fn resolve_unix_install_prefix_from_invocation(
    argv0: &OsStr,
    running_executable: &Path,
    path_env: Option<&OsStr>,
    cwd: &Path,
) -> Result<PathBuf> {
    if argv0.as_bytes().contains(&b'/') {
        let path = if Path::new(argv0).is_absolute() {
            PathBuf::from(argv0)
        } else {
            if !cwd.is_absolute() {
                return Err(anyhow!("invocation CWD is not absolute"));
            }
            cwd.join(argv0)
        };
        return validate_unix_substrate_invocation_witness(
            &lexically_normalize_absolute_invocation_path(&path)?,
            running_executable,
        );
    }

    if argv0 != OsStr::new("substrate") {
        return Err(anyhow!("bare invocation is not substrate"));
    }
    let mut candidates = Vec::new();
    let mut candidate_paths = HashSet::new();
    if let Some(path_env) = path_env {
        for entry in std::env::split_paths(path_env) {
            if entry.as_os_str().is_empty() || !entry.is_absolute() {
                continue;
            }
            let candidate = entry.join("substrate");
            if !candidate_paths.insert(candidate.clone()) {
                continue;
            }
            if let Ok(prefix) =
                validate_unix_substrate_invocation_witness(&candidate, running_executable)
            {
                candidates.push(prefix);
            }
        }
    }
    if candidates.len() != 1 {
        return Err(anyhow!(
            "bare substrate invocation did not have exactly one installed witness"
        ));
    }
    Ok(candidates.remove(0))
}

#[cfg(unix)]
fn validate_unix_substrate_invocation_witness(
    witness: &Path,
    running_executable: &Path,
) -> Result<PathBuf> {
    if witness.file_name() != Some(OsStr::new("substrate")) {
        return Err(anyhow!("invocation witness has the wrong command name"));
    }
    let witness_meta =
        fs::symlink_metadata(witness).context("installed invocation witness is unavailable")?;
    let current_uid = unsafe { libc::geteuid() };

    if witness_meta.file_type().is_symlink() {
        let bin = witness
            .parent()
            .filter(|path| path.file_name() == Some(OsStr::new("bin")))
            .ok_or_else(|| anyhow!("installed symlink witness is outside A/bin"))?;
        let prefix = bin
            .parent()
            .ok_or_else(|| anyhow!("installed symlink witness has no prefix"))?;
        let prefix_identity = secure_owned_directory(prefix, current_uid)?;
        let bin_identity = secure_owned_directory(bin, current_uid)?;
        if witness_meta.uid() != current_uid {
            return Err(anyhow!("installed symlink witness has the wrong owner"));
        }
        require_same_file_identity(witness, running_executable)?;
        revalidate_owned_directory(prefix, current_uid, prefix_identity)?;
        revalidate_owned_directory(bin, current_uid, bin_identity)?;
        revalidate_symlink(witness, current_uid, &witness_meta)?;
        return Ok(prefix.to_path_buf());
    }

    if !witness_meta.file_type().is_file()
        || witness_meta.uid() != current_uid
        || witness_meta.mode() & 0o022 != 0
    {
        return Err(anyhow!("physical invocation witness is not trusted"));
    }
    let bin = witness
        .parent()
        .filter(|path| path.file_name() == Some(OsStr::new("bin")))
        .ok_or_else(|| anyhow!("physical invocation witness is outside a bin directory"))?;
    let version = bin
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| !name.is_empty()))
        .ok_or_else(|| anyhow!("physical release witness has no version"))?;
    let versions = version
        .parent()
        .filter(|path| path.file_name() == Some(OsStr::new("versions")))
        .ok_or_else(|| anyhow!("physical release witness is outside A/versions"))?;
    let prefix = versions
        .parent()
        .ok_or_else(|| anyhow!("physical release witness has no prefix"))?;
    let identities =
        [prefix, versions, version, bin].map(|path| secure_owned_directory(path, current_uid));
    let [prefix_identity, versions_identity, version_identity, bin_identity] = identities;
    let prefix_identity = prefix_identity?;
    let versions_identity = versions_identity?;
    let version_identity = version_identity?;
    let bin_identity = bin_identity?;
    require_same_file_identity(witness, running_executable)?;
    let public_link = prefix.join("bin/substrate");
    let public_link_meta = fs::symlink_metadata(&public_link)
        .context("physical release witness has no public A/bin link")?;
    if !public_link_meta.file_type().is_symlink() || public_link_meta.uid() != current_uid {
        return Err(anyhow!("physical release public link is not trusted"));
    }
    let public_bin = prefix.join("bin");
    let public_bin_identity = secure_owned_directory(&public_bin, current_uid)?;
    require_same_file_identity(&public_link, witness)?;
    revalidate_owned_directory(prefix, current_uid, prefix_identity)?;
    revalidate_owned_directory(versions, current_uid, versions_identity)?;
    revalidate_owned_directory(version, current_uid, version_identity)?;
    revalidate_owned_directory(bin, current_uid, bin_identity)?;
    revalidate_owned_directory(&public_bin, current_uid, public_bin_identity)?;
    revalidate_symlink(&public_link, current_uid, &public_link_meta)?;
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
            Component::RootDir | Component::CurDir => {}
            Component::Normal(value) => normalized.push(value),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(anyhow!("invocation pathname escapes root"));
                }
            }
            Component::Prefix(_) => return Err(anyhow!("invalid Unix invocation pathname")),
        }
    }
    Ok(normalized)
}

#[cfg(unix)]
fn secure_owned_directory(path: &Path, uid: libc::uid_t) -> Result<(u64, u64)> {
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
fn revalidate_owned_directory(path: &Path, uid: libc::uid_t, expected: (u64, u64)) -> Result<()> {
    if secure_owned_directory(path, uid)? != expected {
        return Err(anyhow!("installed witness directory identity changed"));
    }
    Ok(())
}

#[cfg(unix)]
fn revalidate_symlink(path: &Path, uid: libc::uid_t, expected: &fs::Metadata) -> Result<()> {
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
fn revalidate_regular_file(path: &Path, uid: libc::uid_t, expected: &fs::Metadata) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("physical release witness disappeared")?;
    if !metadata.file_type().is_file()
        || metadata.uid() != uid
        || metadata.mode() & 0o022 != 0
        || metadata.dev() != expected.dev()
        || metadata.ino() != expected.ino()
    {
        return Err(anyhow!("physical release witness identity changed"));
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
pub(crate) fn construct_windows_install_bootstrap_context(
    declared_prefix: Option<&Path>,
    argv0: &OsStr,
    running_executable: &Path,
    path_env: Option<&OsStr>,
    cwd: &Path,
) -> Result<InstallBootstrapContextCarrierV1> {
    let prefix = match declared_prefix {
        Some(prefix) => prefix.to_path_buf(),
        None => resolve_windows_install_prefix_from_invocation(
            argv0,
            running_executable,
            path_env,
            cwd,
        )?,
    };
    let prefix = prefix
        .to_str()
        .ok_or_else(|| anyhow!("install prefix is not valid UTF-8"))?;
    let principal = current_windows_principal()?;
    let PlatformPrincipalV1::Windows { account, sid } = principal else {
        return Err(anyhow!("current principal is not Windows"));
    };
    let context =
        transport_api_types::InstallBootstrapContextV1::new_windows(prefix, &account, &sid)
            .context("invalid Windows install bootstrap context")?;
    InstallBootstrapContextCarrierV1::from_context(context)
        .context("failed to commit Windows install bootstrap context")
}

#[cfg(windows)]
pub(crate) fn decode_and_bind_windows_install_bootstrap_context(
    encoded: &str,
    declared_prefix: Option<&Path>,
) -> Result<InstallBootstrapContextCarrierV1> {
    let carrier = InstallBootstrapContextCarrierV1::decode(encoded)
        .context("invalid install bootstrap carrier")?;
    if carrier.encode().context("failed to encode carrier")? != encoded {
        return Err(anyhow!(
            "install bootstrap carrier spelling is noncanonical"
        ));
    }
    bind_windows_install_bootstrap_context(&carrier)?;
    if let Some(prefix) = declared_prefix {
        let raw = prefix
            .to_str()
            .ok_or_else(|| anyhow!("declared install prefix is not valid UTF-8"))?;
        let normalized = transport_api_types::normalize_windows_install_bootstrap_path(raw)
            .context("invalid declared install prefix")?;
        if normalized != carrier.context.selected_host_prefix {
            return Err(anyhow!("declared install prefix does not match carrier"));
        }
    }
    reject_conflicting_install_bootstrap_projections(&carrier, encoded, |key| {
        std::env::var_os(key)
    })?;
    Ok(carrier)
}

#[cfg(windows)]
pub(crate) fn bind_windows_install_bootstrap_context(
    carrier: &InstallBootstrapContextCarrierV1,
) -> Result<()> {
    carrier
        .validate()
        .context("invalid install bootstrap carrier")?;
    let PlatformPrincipalV1::Windows { .. } = &carrier.context.intended_host_principal else {
        return Err(anyhow!("expected Windows install principal"));
    };
    if carrier.context.intended_host_principal != current_windows_principal()? {
        return Err(anyhow!(
            "install bootstrap principal does not match current Windows principal"
        ));
    }
    Ok(())
}

#[cfg(windows)]
pub(crate) fn current_windows_principal_and_known_folder() -> Result<(PlatformPrincipalV1, PathBuf)>
{
    let token = open_current_windows_token()?;
    let principal = windows_principal_from_token(token.0)?;
    let known_folder = windows_known_folder_from_token(token.0)?;
    Ok((principal, known_folder))
}

#[cfg(windows)]
pub(crate) fn windows_known_folder_for_principal(
    principal: &PlatformPrincipalV1,
) -> Result<PathBuf> {
    let PlatformPrincipalV1::Windows { .. } = principal else {
        return Err(anyhow!("expected Windows install principal"));
    };
    let token = open_current_windows_token()?;
    let current = windows_principal_from_token(token.0)?;
    if principal != &current {
        return Err(anyhow!(
            "Windows Known Folder principal does not match current token"
        ));
    }
    windows_known_folder_from_token(token.0)
}

#[cfg(windows)]
fn resolve_windows_install_prefix_from_invocation(
    argv0: &OsStr,
    running_executable: &Path,
    path_env: Option<&OsStr>,
    cwd: &Path,
) -> Result<PathBuf> {
    let argv0_text = argv0
        .to_str()
        .ok_or_else(|| anyhow!("Windows invocation pathname is not valid UTF-8"))?;
    if argv0_text.contains('\\') || argv0_text.contains('/') {
        let path = if Path::new(argv0).is_absolute() {
            PathBuf::from(argv0)
        } else {
            if !cwd.is_absolute() {
                return Err(anyhow!("invocation CWD is not absolute"));
            }
            cwd.join(argv0)
        };
        let raw = path
            .to_str()
            .ok_or_else(|| anyhow!("Windows invocation pathname is not valid UTF-8"))?;
        let normalized = transport_api_types::normalize_windows_install_bootstrap_path(raw)
            .context("invalid Windows invocation pathname")?;
        let normalized = physical_windows_invocation_path(Path::new(&normalized))?;
        return validate_windows_substrate_invocation_witness(&normalized, running_executable);
    }

    if argv0 != OsStr::new("substrate") && argv0 != OsStr::new("substrate.exe") {
        return Err(anyhow!("bare invocation is not substrate"));
    }
    let mut candidates = HashSet::new();
    let mut candidate_paths = HashSet::new();
    if let Some(path_env) = path_env {
        for entry in std::env::split_paths(path_env) {
            if entry.as_os_str().is_empty() || !entry.is_absolute() {
                continue;
            }
            let candidate = entry.join("substrate.exe");
            if !candidate_paths.insert(candidate.clone()) {
                continue;
            }
            let Some(raw) = candidate.to_str() else {
                continue;
            };
            let Ok(normalized) = transport_api_types::normalize_windows_install_bootstrap_path(raw)
            else {
                continue;
            };
            if let Ok(prefix) = validate_windows_substrate_invocation_witness(
                Path::new(&normalized),
                running_executable,
            ) {
                candidates.insert(prefix);
            }
        }
    }
    if candidates.len() != 1 {
        return Err(anyhow!(
            "bare substrate invocation did not have exactly one installed witness"
        ));
    }
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("installed witness candidate disappeared"))
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
fn current_windows_principal() -> Result<PlatformPrincipalV1> {
    let token = open_current_windows_token()?;
    windows_principal_from_token(token.0)
}

#[cfg(windows)]
fn physical_windows_invocation_path(path: &Path) -> Result<PathBuf> {
    match path.file_name() {
        Some(name) if name == OsStr::new("substrate.exe") => Ok(path.to_path_buf()),
        Some(name) if name == OsStr::new("substrate") => Ok(path.with_file_name("substrate.exe")),
        _ => Err(anyhow!("invocation witness has the wrong command name")),
    }
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
    let authority_text = authority_value.to_string();
    let mut text = format!("S-{revision}-{authority_text}");
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
fn validate_windows_substrate_invocation_witness(
    witness: &Path,
    running_executable: &Path,
) -> Result<PathBuf> {
    if witness.file_name() != Some(OsStr::new("substrate.exe")) {
        return Err(anyhow!("invocation witness has the wrong command name"));
    }
    let bin = witness
        .parent()
        .filter(|path| path.file_name() == Some(OsStr::new("bin")))
        .ok_or_else(|| anyhow!("installed invocation witness is outside A/bin"))?;
    let prefix = bin
        .parent()
        .ok_or_else(|| anyhow!("installed invocation witness has no prefix"))?;
    let prefix_handle = open_windows_path_no_follow(prefix, true)?;
    let bin_handle = open_windows_path_no_follow(bin, true)?;
    let witness_handle = open_windows_path_no_follow(witness, false)?;
    let running_handle = open_windows_path_no_follow(running_executable, false)?;
    require_same_windows_file_identity(witness, running_executable)?;
    if windows_file_identity(witness_handle.0)? != windows_file_identity(running_handle.0)? {
        return Err(anyhow!(
            "installed invocation witness does not match running executable"
        ));
    }

    let physical_witness = PathBuf::from(final_windows_handle_path(witness_handle.0)?);
    if physical_witness.file_name() != Some(OsStr::new("substrate.exe")) {
        return Err(anyhow!(
            "physical invocation witness has the wrong command name"
        ));
    }
    let physical_bin = physical_witness
        .parent()
        .filter(|path| path.file_name() == Some(OsStr::new("bin")))
        .ok_or_else(|| anyhow!("physical invocation witness is outside A/bin"))?;
    let physical_prefix = physical_bin
        .parent()
        .ok_or_else(|| anyhow!("physical invocation witness has no prefix"))?;
    if physical_prefix
        .parent()
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .is_some_and(|name| name.eq_ignore_ascii_case("versions"))
    {
        return Err(anyhow!(
            "version payload is not the physical public invocation witness"
        ));
    }
    let _prefix_ancestors = open_windows_ancestors_no_follow(prefix)?;
    let _bin_ancestors = open_windows_ancestors_no_follow(bin)?;
    let _witness_ancestors = open_windows_ancestors_no_follow(witness)?;
    let _running_ancestors = open_windows_ancestors_no_follow(running_executable)?;
    let physical_prefix_handle = open_windows_path_no_follow(physical_prefix, true)?;
    let physical_bin_handle = open_windows_path_no_follow(physical_bin, true)?;
    if windows_file_identity(prefix_handle.0)? != windows_file_identity(physical_prefix_handle.0)?
        || windows_file_identity(bin_handle.0)? != windows_file_identity(physical_bin_handle.0)?
    {
        return Err(anyhow!(
            "installed invocation witness aliases a different prefix"
        ));
    }
    let raw_prefix = physical_prefix
        .to_str()
        .ok_or_else(|| anyhow!("installed invocation prefix is not valid UTF-8"))?;
    let normalized_prefix =
        transport_api_types::normalize_windows_install_bootstrap_path(raw_prefix)
            .context("installed invocation prefix is invalid")?;
    Ok(PathBuf::from(normalized_prefix))
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
        return transport_api_types::normalize_windows_install_bootstrap_path(&path)
            .context("Windows final pathname is invalid");
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use tempfile::TempDir;

    fn secure_dir(path: &Path) {
        fs::create_dir_all(path).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn dev_witness() -> (TempDir, PathBuf, PathBuf) {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let bin = prefix.join("bin");
        secure_dir(&prefix);
        secure_dir(&bin);
        let witness = bin.join("substrate");
        let running = std::env::current_exe().unwrap();
        symlink(&running, &witness).unwrap();
        (temp, prefix, witness)
    }

    #[test]
    fn current_unix_principal_constructs_and_binds() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let carrier = construct_unix_install_bootstrap_context(
            Some(&prefix),
            OsStr::new("substrate"),
            &std::env::current_exe().unwrap(),
            None,
            temp.path(),
        )
        .unwrap();

        assert_eq!(
            carrier.context.selected_host_prefix,
            prefix.to_string_lossy()
        );
        bind_unix_install_bootstrap_context(&carrier).unwrap();
    }

    #[test]
    fn forged_unix_principal_rejects() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let mut carrier = construct_unix_install_bootstrap_context(
            Some(&prefix),
            OsStr::new("substrate"),
            &std::env::current_exe().unwrap(),
            None,
            temp.path(),
        )
        .unwrap();
        let PlatformPrincipalV1::Unix { uid, .. } = &mut carrier.context.intended_host_principal
        else {
            panic!("expected Unix principal")
        };
        *uid = uid.checked_add(1).unwrap();
        carrier = InstallBootstrapContextCarrierV1::from_context(carrier.context).unwrap();

        assert!(bind_unix_install_bootstrap_context(&carrier).is_err());
    }

    #[test]
    fn dev_bin_symlink_self_derives_prefix() {
        let (_temp, prefix, witness) = dev_witness();
        let carrier = construct_unix_install_bootstrap_context(
            None,
            witness.as_os_str(),
            &std::env::current_exe().unwrap(),
            None,
            Path::new("/"),
        )
        .unwrap();

        assert_eq!(
            carrier.context.selected_host_prefix,
            prefix.to_string_lossy()
        );
    }

    #[test]
    fn release_payload_self_derives_only_with_matching_public_link() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("release-prefix");
        let version_bin = prefix.join("versions/v1/bin");
        secure_dir(&prefix);
        secure_dir(&prefix.join("versions"));
        secure_dir(&prefix.join("versions/v1"));
        secure_dir(&version_bin);
        secure_dir(&prefix.join("bin"));
        let running = std::env::current_exe().unwrap();
        let payload = version_bin.join("substrate");
        fs::copy(&running, &payload).unwrap();
        symlink(&payload, prefix.join("bin/substrate")).unwrap();

        let carrier = construct_unix_install_bootstrap_context(
            None,
            payload.as_os_str(),
            &payload,
            None,
            Path::new("/"),
        )
        .unwrap();
        assert_eq!(
            carrier.context.selected_host_prefix,
            prefix.to_string_lossy()
        );

        fs::remove_file(prefix.join("bin/substrate")).unwrap();
        assert!(construct_unix_install_bootstrap_context(
            None,
            payload.as_os_str(),
            &payload,
            None,
            Path::new("/"),
        )
        .is_err());
    }

    #[test]
    fn direct_repository_binary_is_not_an_installed_witness() {
        let running = std::env::current_exe().unwrap();
        assert!(construct_unix_install_bootstrap_context(
            None,
            running.as_os_str(),
            &running,
            None,
            Path::new("/"),
        )
        .is_err());
    }

    #[test]
    fn bare_invocation_requires_exactly_one_absolute_path_candidate() {
        let (_first_temp, first_prefix, first_witness) = dev_witness();
        let (_second_temp, second_prefix, _second_witness) = dev_witness();
        let first_path = first_witness.parent().unwrap();
        let second_path = second_prefix.join("bin");
        let running = std::env::current_exe().unwrap();

        let one = construct_unix_install_bootstrap_context(
            None,
            OsStr::new("substrate"),
            &running,
            Some(first_path.as_os_str()),
            Path::new("/"),
        )
        .unwrap();
        assert_eq!(
            one.context.selected_host_prefix,
            first_prefix.to_string_lossy()
        );

        let multiple = std::env::join_paths([first_path, second_path.as_path()]).unwrap();
        assert!(construct_unix_install_bootstrap_context(
            None,
            OsStr::new("substrate"),
            &running,
            Some(&multiple),
            Path::new("/"),
        )
        .is_err());
        assert!(construct_unix_install_bootstrap_context(
            None,
            OsStr::new("substrate"),
            &running,
            Some(OsStr::new("relative/path")),
            Path::new("/"),
        )
        .is_err());
    }

    #[test]
    fn explicit_relative_invocation_uses_cwd_only_for_that_spelling() {
        let (_temp, prefix, witness) = dev_witness();
        let cwd = prefix.parent().unwrap();
        let relative = witness.strip_prefix(cwd).unwrap();
        let carrier = construct_unix_install_bootstrap_context(
            None,
            relative.as_os_str(),
            &std::env::current_exe().unwrap(),
            None,
            cwd,
        )
        .unwrap();
        assert_eq!(
            carrier.context.selected_host_prefix,
            prefix.to_string_lossy()
        );
    }

    #[test]
    fn internal_projection_validation_requires_exact_tuple() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let carrier = construct_unix_install_bootstrap_context(
            Some(&prefix),
            OsStr::new("substrate"),
            &std::env::current_exe().unwrap(),
            None,
            temp.path(),
        )
        .unwrap();
        let encoded = carrier.encode().unwrap();
        let mut projections = expected_install_bootstrap_projections(&carrier, &encoded).unwrap();
        validate_install_bootstrap_projections(&carrier, &encoded, |key| {
            projections.get(key).cloned()
        })
        .unwrap();
        projections.insert("SUBSTRATE_HOME", OsString::from("/conflicting/home"));
        assert!(
            validate_install_bootstrap_projections(&carrier, &encoded, |key| {
                projections.get(key).cloned()
            })
            .is_err()
        );
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;
    use serial_test::serial;
    use std::os::windows::fs::{symlink_dir, symlink_file};
    use tempfile::TempDir;

    struct EnvironmentGuard(Vec<(&'static str, Option<OsString>)>);

    impl EnvironmentGuard {
        fn set(values: &[(&'static str, &OsStr)]) -> Self {
            let previous = values
                .iter()
                .map(|(key, _)| (*key, std::env::var_os(key)))
                .collect();
            for (key, value) in values {
                std::env::set_var(key, value);
            }
            Self(previous)
        }
    }

    impl Drop for EnvironmentGuard {
        fn drop(&mut self) {
            for (key, previous) in self.0.drain(..) {
                if let Some(value) = previous {
                    std::env::set_var(key, value);
                } else {
                    std::env::remove_var(key);
                }
            }
        }
    }

    fn installed_witness(prefix: &Path) -> PathBuf {
        let bin = prefix.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let witness = bin.join("substrate.exe");
        fs::copy(std::env::current_exe().unwrap(), &witness).unwrap();
        witness
    }

    #[test]
    #[serial]
    fn canonical_current_windows_principal_constructs_and_binds() {
        let _environment = EnvironmentGuard::set(&[
            ("LOCALAPPDATA", OsStr::new(r"C:\conflicting-local")),
            ("USERPROFILE", OsStr::new(r"C:\conflicting-profile")),
            ("HOME", OsStr::new(r"C:\conflicting-home")),
        ]);
        let (principal, known_folder) = current_windows_principal_and_known_folder().unwrap();
        let PlatformPrincipalV1::Windows { account, sid } = &principal else {
            panic!("expected Windows principal");
        };
        assert!(!account.is_empty());
        assert!(sid.starts_with("S-1-"));
        assert_eq!(
            windows_known_folder_for_principal(&principal).unwrap(),
            known_folder
        );

        let prefix = known_folder.join("Substrate");
        let expected_prefix =
            transport_api_types::normalize_windows_install_bootstrap_path(prefix.to_str().unwrap())
                .unwrap();
        let carrier = construct_windows_install_bootstrap_context(
            Some(&prefix),
            OsStr::new("substrate"),
            &std::env::current_exe().unwrap(),
            None,
            Path::new(r"C:\"),
        )
        .unwrap();
        bind_windows_install_bootstrap_context(&carrier).unwrap();
        assert_eq!(carrier.context.selected_host_prefix, expected_prefix);
        assert!(expected_prefix.ends_with(r"\Substrate"));
    }

    #[test]
    fn declared_windows_prefix_normalizes_and_round_trips() {
        let temp = TempDir::new().unwrap();
        let raw = format!(
            "{}\\\\declared-prefix\\\\",
            temp.path().to_string_lossy().replace('\\', "/")
        );
        let carrier = construct_windows_install_bootstrap_context(
            Some(Path::new(&raw)),
            OsStr::new("substrate.exe"),
            &std::env::current_exe().unwrap(),
            None,
            temp.path(),
        )
        .unwrap();
        let encoded = carrier.encode().unwrap();
        let decoded =
            decode_and_bind_windows_install_bootstrap_context(&encoded, Some(Path::new(&raw)))
                .unwrap();
        assert_eq!(decoded, carrier);
        assert_eq!(
            carrier.context.selected_host_prefix,
            transport_api_types::normalize_windows_install_bootstrap_path(&raw).unwrap()
        );
    }

    #[test]
    #[serial]
    fn physical_public_release_copy_self_derives_under_conflicting_ambient_home() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let witness = installed_witness(&prefix);
        let _environment = EnvironmentGuard::set(&[
            ("LOCALAPPDATA", OsStr::new(r"C:\conflicting-local")),
            ("USERPROFILE", OsStr::new(r"C:\conflicting-profile")),
            ("HOME", OsStr::new(r"C:\conflicting-home")),
        ]);

        let carrier = construct_windows_install_bootstrap_context(
            None,
            witness.as_os_str(),
            &witness,
            None,
            Path::new(r"C:\"),
        )
        .unwrap();
        assert_eq!(
            carrier.context.selected_host_prefix,
            transport_api_types::normalize_windows_install_bootstrap_path(prefix.to_str().unwrap())
                .unwrap()
        );

        let payload_bin = prefix.join(r"versions\v1\bin");
        fs::create_dir_all(&payload_bin).unwrap();
        let payload = payload_bin.join("substrate.exe");
        fs::copy(std::env::current_exe().unwrap(), &payload).unwrap();
        assert!(construct_windows_install_bootstrap_context(
            None,
            payload.as_os_str(),
            &payload,
            None,
            Path::new(r"C:\"),
        )
        .is_err());
    }

    #[test]
    fn explicit_absolute_and_relative_windows_invocations_select_only_their_spelling() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let witness = installed_witness(&prefix);

        let extensionless = witness.with_file_name("substrate");
        let absolute = resolve_windows_install_prefix_from_invocation(
            extensionless.as_os_str(),
            &witness,
            None,
            Path::new(r"C:\"),
        )
        .unwrap();
        let cwd = prefix.parent().unwrap();
        let relative = witness.strip_prefix(cwd).unwrap();
        let relative = resolve_windows_install_prefix_from_invocation(
            relative.as_os_str(),
            &witness,
            None,
            cwd,
        )
        .unwrap();
        assert_eq!(absolute, prefix);
        assert_eq!(relative, prefix);
    }

    #[test]
    fn bare_windows_path_requires_exactly_one_installed_witness() {
        let temp = TempDir::new().unwrap();
        let first_prefix = temp.path().join("first-prefix");
        let first = installed_witness(&first_prefix);
        let one_path = first.parent().unwrap();

        let selected = resolve_windows_install_prefix_from_invocation(
            OsStr::new("substrate"),
            &first,
            Some(one_path.as_os_str()),
            Path::new(r"C:\"),
        )
        .unwrap();
        assert_eq!(selected, first_prefix);

        assert!(resolve_windows_install_prefix_from_invocation(
            OsStr::new("substrate.exe"),
            &first,
            Some(OsStr::new("relative-path")),
            Path::new(r"C:\"),
        )
        .is_err());

        let second_prefix = temp.path().join("second-prefix");
        fs::create_dir_all(second_prefix.join("bin")).unwrap();
        fs::hard_link(&first, second_prefix.join(r"bin\substrate.exe")).unwrap();
        let multiple =
            std::env::join_paths([one_path, second_prefix.join("bin").as_path()]).unwrap();
        assert!(resolve_windows_install_prefix_from_invocation(
            OsStr::new("substrate.exe"),
            &first,
            Some(&multiple),
            Path::new(r"C:\"),
        )
        .is_err());
    }

    #[test]
    fn windows_file_identity_is_exact_and_reparse_points_reject() {
        let temp = TempDir::new().unwrap();
        let witness = installed_witness(&temp.path().join("selected-prefix"));
        let hard_link = temp.path().join("same-file.exe");
        fs::hard_link(&witness, &hard_link).unwrap();
        require_same_windows_file_identity(&witness, &hard_link).unwrap();

        let mismatch = temp.path().join("different-file.exe");
        fs::copy(std::env::current_exe().unwrap(), &mismatch).unwrap();
        assert!(require_same_windows_file_identity(&witness, &mismatch).is_err());

        let reparse = temp.path().join("reparse.exe");
        symlink_file(&witness, &reparse).unwrap();
        assert!(require_same_windows_file_identity(&witness, &reparse).is_err());
    }

    #[test]
    fn windows_ancestor_reparse_alias_rejects() {
        let temp = TempDir::new().unwrap();
        let real_root = temp.path().join("real");
        let prefix = real_root.join("selected-prefix");
        let witness = installed_witness(&prefix);
        let alias_root = temp.path().join("alias");
        symlink_dir(&real_root, &alias_root).unwrap();
        let alias_witness = alias_root.join(r"selected-prefix\bin\substrate.exe");

        assert!(resolve_windows_install_prefix_from_invocation(
            alias_witness.as_os_str(),
            &witness,
            None,
            Path::new(r"C:\"),
        )
        .is_err());
    }

    #[test]
    fn forged_or_non_windows_principal_rejects() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let carrier = construct_windows_install_bootstrap_context(
            Some(&prefix),
            OsStr::new("substrate.exe"),
            &std::env::current_exe().unwrap(),
            None,
            temp.path(),
        )
        .unwrap();
        let mut forged_context = carrier.context.clone();
        let PlatformPrincipalV1::Windows { account, .. } =
            &mut forged_context.intended_host_principal
        else {
            panic!("expected Windows principal");
        };
        account.push_str("-forged");
        let forged = InstallBootstrapContextCarrierV1::from_context(forged_context).unwrap();
        assert!(bind_windows_install_bootstrap_context(&forged).is_err());

        let unix = transport_api_types::InstallBootstrapContextV1::new_unix(
            "/tmp/substrate",
            "not-windows",
            1000,
        )
        .unwrap();
        let unix = InstallBootstrapContextCarrierV1::from_context(unix).unwrap();
        assert!(bind_windows_install_bootstrap_context(&unix).is_err());
    }

    #[test]
    #[serial]
    fn malformed_tampered_or_conflicting_windows_carrier_rejects() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let carrier = construct_windows_install_bootstrap_context(
            Some(&prefix),
            OsStr::new("substrate.exe"),
            &std::env::current_exe().unwrap(),
            None,
            temp.path(),
        )
        .unwrap();
        let encoded = carrier.encode().unwrap();
        assert!(decode_and_bind_windows_install_bootstrap_context("not+canonical", None).is_err());
        let mut tampered = encoded.into_bytes();
        tampered[0] = if tampered[0] == b'A' { b'B' } else { b'A' };
        let tampered = String::from_utf8(tampered).unwrap();
        assert!(decode_and_bind_windows_install_bootstrap_context(&tampered, None).is_err());

        let encoded = carrier.encode().unwrap();
        assert!(decode_and_bind_windows_install_bootstrap_context(
            &encoded,
            Some(&temp.path().join("other-prefix")),
        )
        .is_err());

        let _environment =
            EnvironmentGuard::set(&[("SUBSTRATE_HOME", OsStr::new(r"C:\conflicting-home"))]);
        assert!(decode_and_bind_windows_install_bootstrap_context(&encoded, None).is_err());
    }

    #[test]
    fn path_order_and_executable_parent_spelling_cannot_select_windows_prefix() {
        let temp = TempDir::new().unwrap();
        let prefix = temp.path().join("selected-prefix");
        let witness = installed_witness(&prefix);
        let arbitrary_bin = temp.path().join("arbitrary/bin");
        fs::create_dir_all(&arbitrary_bin).unwrap();
        fs::copy(
            std::env::current_exe().unwrap(),
            arbitrary_bin.join("substrate.exe"),
        )
        .unwrap();
        let path =
            std::env::join_paths([arbitrary_bin.as_path(), witness.parent().unwrap()]).unwrap();

        let selected = resolve_windows_install_prefix_from_invocation(
            OsStr::new("substrate.exe"),
            &witness,
            Some(&path),
            Path::new(r"C:\"),
        )
        .unwrap();
        assert_eq!(selected, prefix);

        let repository = std::env::current_exe().unwrap();
        assert!(construct_windows_install_bootstrap_context(
            None,
            repository.as_os_str(),
            &repository,
            None,
            Path::new(r"C:\"),
        )
        .is_err());
    }
}
