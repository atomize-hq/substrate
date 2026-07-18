use anyhow::{anyhow, Context, Result};
use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformPrincipalV1};

pub(crate) const INSTALL_BOOTSTRAP_CONTEXT_ENV: &str = "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1";
pub(crate) const INSTALL_BOOTSTRAP_COMMITMENT_ENV: &str =
    "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT";
pub(crate) const INSTALL_BOOTSTRAP_ACCOUNT_ENV: &str = "SUBSTRATE_INSTALL_PRIMARY_USER";
pub(crate) const INSTALL_BOOTSTRAP_UID_ENV: &str = "SUBSTRATE_INSTALL_PRIMARY_UID";

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

pub(crate) fn current_unix_principal_and_home() -> Result<(PlatformPrincipalV1, PathBuf)> {
    // SAFETY: geteuid has no preconditions.
    let uid = unsafe { libc::geteuid() };
    if uid == 0 {
        return Err(anyhow!(
            "effective UID 0 requires the R2-2 intended-principal boundary"
        ));
    }
    let uid = u32::try_from(uid).context("effective UID does not fit u32")?;
    let (account, home) = lookup_unix_account_by_uid(uid)?;
    Ok((PlatformPrincipalV1::Unix { account, uid }, home))
}

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
    let PlatformPrincipalV1::Unix { account, uid } = &carrier.context.intended_host_principal
    else {
        return Err(anyhow!("expected Unix install principal"));
    };
    Ok(HashMap::from([
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
        (INSTALL_BOOTSTRAP_ACCOUNT_ENV, OsString::from(account)),
        (INSTALL_BOOTSTRAP_UID_ENV, OsString::from(uid.to_string())),
        (INSTALL_BOOTSTRAP_CONTEXT_ENV, OsString::from(encoded)),
    ]))
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

fn revalidate_owned_directory(path: &Path, uid: libc::uid_t, expected: (u64, u64)) -> Result<()> {
    if secure_owned_directory(path, uid)? != expected {
        return Err(anyhow!("installed witness directory identity changed"));
    }
    Ok(())
}

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

#[cfg(test)]
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
