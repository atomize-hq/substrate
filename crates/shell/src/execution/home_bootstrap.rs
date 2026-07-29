use std::fmt;
#[cfg(not(unix))]
use std::fs;
#[cfg(not(unix))]
use std::io::{self, Write};
use std::path::Path;
#[cfg(unix)]
use std::{ffi::CString, mem::MaybeUninit};

use substrate_common::paths as substrate_paths;
#[cfg(unix)]
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformPrincipalV1};

#[derive(Debug, Clone)]
pub(crate) struct HomeBootstrapError {
    exit_code: i32,
    message: String,
}

impl HomeBootstrapError {
    fn new(exit_code: i32, message: impl Into<String>) -> Self {
        Self {
            exit_code,
            message: message.into(),
        }
    }

    fn io(message: impl Into<String>) -> Self {
        Self::new(1, message)
    }

    fn denied(message: impl Into<String>) -> Self {
        Self::new(5, message)
    }

    #[cfg(unix)]
    fn unsupported_private_home(
        path: &Path,
        _uid: libc::uid_t,
        error: &crate::execution::agent_runtime::host_session_authority::trusted_fs::PrivateHomeError,
    ) -> Self {
        let requested = error.requested_home().unwrap_or(path);
        let offending = error.offending_path().unwrap_or(path);
        let role = error
            .object_role()
            .map(|role| role.as_str())
            .unwrap_or("final-root");
        let mut provenance = format!("offending path '{}' (role={role}", offending.display());
        if let Some(kind) = error.acl_kind() {
            provenance.push_str(&format!(", acl-kind={}", kind.as_str()));
        }
        if let Some(authority) = error.acl_authority() {
            provenance.push_str(&format!(", authority={}", authority.as_str()));
        }
        if let Some(class) = error.acl_diagnostic_class() {
            provenance.push_str(&format!(", acl-observation={}", class.as_str()));
        }
        provenance.push_str(&format!(
            ", reason={}, candidate-created={})",
            error.reason().as_str(),
            error.candidate_provenance().creation_as_str()
        ));

        use crate::execution::agent_runtime::host_session_authority::trusted_fs::PrivateHomeCandidateProvenance;
        let instruction = match error.candidate_provenance() {
            PrivateHomeCandidateProvenance::Created => {
                "The rejected candidate was created by this attempt and remains in place; this attempt performs no cleanup."
            }
            PrivateHomeCandidateProvenance::Unknown => {
                "Candidate creation could not be determined after a failed creation wait; this attempt performs no cleanup and gives no existing-root reset instruction."
            }
            PrivateHomeCandidateProvenance::PreExisting => {
                "Existing roots are never repaired; reset it manually and retry."
            }
            PrivateHomeCandidateProvenance::NotCreated if role == "ancestor" => {
                "The offending ancestor was not modified; correct that object and retry."
            }
            PrivateHomeCandidateProvenance::NotCreated => {
                "No candidate was created; correct the requested path or parent and retry."
            }
        };
        Self::denied(format!(
            "substrate: unsupported SUBSTRATE_HOME '{}': expected exact owner/mode authority with no effective external write authority; observable final-root access/default ACLs and unavailable ACL validation are rejected; {provenance}. {instruction}",
            requested.display()
        ))
    }

    #[cfg(unix)]
    #[allow(dead_code)]
    fn ambiguous_private_home_owner(path: &Path) -> Self {
        Self::denied(format!(
            "substrate: unsupported SUBSTRATE_HOME '{}': cannot determine the intended per-user owner while running with effective uid 0; found owner-ambiguous. Set the supported explicit user input or run as the intended user; no home was created or modified.",
            path.display()
        ))
    }

    pub(crate) fn exit_code(&self) -> i32 {
        self.exit_code
    }
}

impl fmt::Display for HomeBootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HomeBootstrapError {}

const DEPS_README_MD: &str = r#"# Substrate world deps inventory (global)

This directory is the **global inventory** for `substrate world deps`.

## Inventory vs enabled vs applied

- **Inventory**: definitions that exist (packages + bundles). Inventory comes from:
  - built-in items shipped with Substrate (may appear in `available` even without this directory)
  - global inventory: `$SUBSTRATE_HOME/deps/` (this directory)
  - workspace inventory: `<workspace_root>/.substrate/deps/`
- **Enabled**: the list of item names you want applied.
  - Global enabled list: `$SUBSTRATE_HOME/config.yaml` (`world.deps.enabled`)
  - Workspace enabled list: `<workspace_root>/.substrate/workspace.yaml` (`world.deps.enabled`)
- **Applied**: what is actually present in the world.

Applying enabled deps is world-backed and happens via:

- `substrate world deps current sync`

There is no host fallback for applying deps.

## Examples

The example files under `packages/`, `bundles/`, and `scripts/` are “shape only”.
They are **not auto-enabled**. You can edit or delete them.
"#;

const EXAMPLE_APT_YAML: &str = r#"version: 1
name: example-apt
description: Example apt package (shape only; not auto-enabled).
runnable: true
entrypoints:
  - example
install:
  method: apt
  apt:
    - name: example
"#;

const EXAMPLE_SCRIPT_YAML: &str = r#"version: 1
name: example-script
description: Example script-installed package (shape only; not auto-enabled).
runnable: true
entrypoints:
  - example-script
install:
  method: script
  script_path: ../scripts/example-install.sh
"#;

const EXAMPLE_MANUAL_YAML: &str = r#"version: 1
name: example-manual
description: Example manual package (shape only; not auto-enabled).
runnable: false
install:
  method: manual
  manual_instructions: |
    This is an example. Replace this file with real instructions, or delete it.

    Manual packages block `install`/`sync` until satisfied.
    Put the steps required to make the tool available in-world here.
"#;

const EXAMPLE_BUNDLE_YAML: &str = r#"version: 1
name: example-bundle
description: Example bundle of packages (shape only; not auto-enabled).
packages:
  - example-apt
  - example-script
"#;

const EXAMPLE_INSTALL_SH: &str = r#"#!/usr/bin/env bash
set -euo pipefail

echo "example-install.sh: this is a scaffolded example script (shape only)." >&2
echo "Edit this script and the corresponding package YAML to install a real tool." >&2

exit 1
"#;

#[allow(dead_code)]
pub(crate) fn ensure_substrate_home_deps_scaffold() -> Result<(), HomeBootstrapError> {
    let substrate_home = substrate_paths::substrate_home().map_err(|err| {
        HomeBootstrapError::io(format!(
            "substrate: failed to resolve SUBSTRATE_HOME: {err}"
        ))
    })?;

    #[cfg(unix)]
    let owner = BootstrapOwner {
        uid: resolve_intended_owner_uid(&substrate_home)?,
    };
    #[cfg(not(unix))]
    let owner = BootstrapOwner {};
    ensure_substrate_home_deps_scaffold_at(&substrate_home, owner)
}

#[cfg(unix)]
pub(crate) fn ensure_substrate_home_deps_scaffold_for_context(
    carrier: &InstallBootstrapContextCarrierV1,
) -> Result<(), HomeBootstrapError> {
    super::install_bootstrap::bind_unix_install_bootstrap_context(carrier).map_err(|_| {
        HomeBootstrapError::denied(
            "substrate: install bootstrap context does not match the current Unix principal",
        )
    })?;
    let PlatformPrincipalV1::Unix { uid, .. } = &carrier.context.intended_host_principal else {
        return Err(HomeBootstrapError::denied(
            "substrate: install bootstrap context has the wrong principal kind",
        ));
    };
    ensure_substrate_home_deps_scaffold_at(
        Path::new(&carrier.context.host_substrate_home),
        BootstrapOwner {
            uid: libc::uid_t::try_from(*uid).map_err(|_| {
                HomeBootstrapError::denied(
                    "substrate: install bootstrap context has an unsupported Unix UID",
                )
            })?,
        },
    )
}

fn ensure_substrate_home_deps_scaffold_at(
    substrate_home: &Path,
    owner: BootstrapOwner,
) -> Result<(), HomeBootstrapError> {
    let substrate_home = substrate_home.to_path_buf();
    #[cfg(unix)]
    let trusted_home =
        crate::execution::agent_runtime::host_session_authority::trusted_fs::ensure_private_substrate_home(
            &substrate_home, owner.uid,
        )
        .map_err(|error| {
            HomeBootstrapError::unsupported_private_home(
                &substrate_home,
                owner.uid,
                &error,
            )
        })?;

    #[cfg(not(unix))]
    ensure_dir(&substrate_home, owner)?;

    #[cfg(unix)]
    {
        let deps = trusted_home
            .ensure_scaffold_directory("deps")
            .map_err(|error| map_trusted_scaffold_error("deps", "directory", error))?;
        let packages = deps
            .ensure_directory("packages")
            .map_err(|error| map_trusted_scaffold_error("deps/packages", "directory", error))?;
        let bundles = deps
            .ensure_directory("bundles")
            .map_err(|error| map_trusted_scaffold_error("deps/bundles", "directory", error))?;
        let scripts = deps
            .ensure_directory("scripts")
            .map_err(|error| map_trusted_scaffold_error("deps/scripts", "directory", error))?;

        deps.ensure_file_if_missing("README.md", DEPS_README_MD.as_bytes())
            .map_err(|error| map_trusted_scaffold_error("deps/README.md", "file", error))?;
        packages
            .ensure_file_if_missing("example-manual.yaml", EXAMPLE_MANUAL_YAML.as_bytes())
            .map_err(|error| {
                map_trusted_scaffold_error("deps/packages/example-manual.yaml", "file", error)
            })?;
        packages
            .ensure_file_if_missing("example-script.yaml", EXAMPLE_SCRIPT_YAML.as_bytes())
            .map_err(|error| {
                map_trusted_scaffold_error("deps/packages/example-script.yaml", "file", error)
            })?;
        packages
            .ensure_file_if_missing("example-apt.yaml", EXAMPLE_APT_YAML.as_bytes())
            .map_err(|error| {
                map_trusted_scaffold_error("deps/packages/example-apt.yaml", "file", error)
            })?;
        bundles
            .ensure_file_if_missing("example-bundle.yaml", EXAMPLE_BUNDLE_YAML.as_bytes())
            .map_err(|error| {
                map_trusted_scaffold_error("deps/bundles/example-bundle.yaml", "file", error)
            })?;
        scripts
            .ensure_file_if_missing("example-install.sh", EXAMPLE_INSTALL_SH.as_bytes())
            .map_err(|error| {
                map_trusted_scaffold_error("deps/scripts/example-install.sh", "file", error)
            })?;

        trusted_home.revalidate_private_home().map_err(|error| {
            HomeBootstrapError::unsupported_private_home(&substrate_home, owner.uid, &error)
        })?;
    }

    #[cfg(not(unix))]
    {
        let deps_root = substrate_home.join("deps");
        ensure_dir(&deps_root, owner)?;
        let packages_dir = deps_root.join("packages");
        let bundles_dir = deps_root.join("bundles");
        let scripts_dir = deps_root.join("scripts");
        ensure_dir(&packages_dir, owner)?;
        ensure_dir(&bundles_dir, owner)?;
        ensure_dir(&scripts_dir, owner)?;
        ensure_file_if_missing(
            &deps_root.join("README.md"),
            DEPS_README_MD.as_bytes(),
            owner,
        )?;
        ensure_file_if_missing(
            &packages_dir.join("example-manual.yaml"),
            EXAMPLE_MANUAL_YAML.as_bytes(),
            owner,
        )?;
        ensure_file_if_missing(
            &packages_dir.join("example-script.yaml"),
            EXAMPLE_SCRIPT_YAML.as_bytes(),
            owner,
        )?;
        ensure_file_if_missing(
            &packages_dir.join("example-apt.yaml"),
            EXAMPLE_APT_YAML.as_bytes(),
            owner,
        )?;
        ensure_file_if_missing(
            &bundles_dir.join("example-bundle.yaml"),
            EXAMPLE_BUNDLE_YAML.as_bytes(),
            owner,
        )?;
        ensure_file_if_missing(
            &scripts_dir.join("example-install.sh"),
            EXAMPLE_INSTALL_SH.as_bytes(),
            owner,
        )?;
    }

    Ok(())
}

#[cfg(unix)]
fn map_trusted_scaffold_error(
    relative_path: &str,
    expected_kind: &str,
    error: crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedFsError,
) -> HomeBootstrapError {
    HomeBootstrapError::io(format!(
        "substrate: failed to scaffold {relative_path}: expected {expected_kind}; accepted private SUBSTRATE_HOME handle validation failed: {error}"
    ))
}

#[derive(Clone, Copy)]
struct BootstrapOwner {
    #[cfg(unix)]
    uid: libc::uid_t,
}

#[cfg(unix)]
#[allow(dead_code)]
fn resolve_intended_owner_uid(path: &Path) -> Result<libc::uid_t, HomeBootstrapError> {
    // SAFETY: geteuid has no preconditions.
    let effective_uid = unsafe { libc::geteuid() };
    if effective_uid != 0 {
        return Ok(effective_uid);
    }

    let intended_user = select_intended_owner_name(
        std::env::var_os("SUBSTRATE_INSTALL_PRIMARY_USER"),
        std::env::var_os("SUDO_USER"),
    )
    .ok_or_else(|| HomeBootstrapError::ambiguous_private_home_owner(path))?;
    let intended_user = CString::new(std::os::unix::ffi::OsStrExt::as_bytes(
        intended_user.as_os_str(),
    ))
    .map_err(|_| HomeBootstrapError::ambiguous_private_home_owner(path))?;
    let mut record = MaybeUninit::<libc::passwd>::uninit();
    let mut result = std::ptr::null_mut();
    let mut buffer = vec![0_u8; 16 * 1024];
    // SAFETY: pointers reference initialized name/buffer storage and writable result slots.
    let status = unsafe {
        libc::getpwnam_r(
            intended_user.as_ptr(),
            record.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut result,
        )
    };
    if status != 0 || result.is_null() {
        return Err(HomeBootstrapError::ambiguous_private_home_owner(path));
    }
    // SAFETY: successful getpwnam_r with non-null result initialized record.
    let record = unsafe { record.assume_init() };
    if record.pw_uid == 0 {
        return Err(HomeBootstrapError::ambiguous_private_home_owner(path));
    }
    Ok(record.pw_uid)
}

#[cfg(unix)]
#[allow(dead_code)]
fn select_intended_owner_name(
    explicit: Option<std::ffi::OsString>,
    sudo_user: Option<std::ffi::OsString>,
) -> Option<std::ffi::OsString> {
    explicit
        .filter(|value| !value.is_empty() && value != "root")
        .or_else(|| sudo_user.filter(|value| !value.is_empty() && value != "root"))
}

#[cfg(not(unix))]
fn ensure_dir(path: &Path, owner: BootstrapOwner) -> Result<(), HomeBootstrapError> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.is_dir() {
                return Ok(());
            }
            Err(HomeBootstrapError::io(format!(
                "substrate: failed to scaffold deps: expected directory at {}, found a file; remove or rename it and retry",
                path.display()
            )))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            #[cfg(unix)]
            {
                crate::execution::agent_runtime::host_session_authority::trusted_fs::ensure_private_substrate_home(
                    path, owner.uid,
                )
                .map(|_| ())
                .map_err(|error| {
                    HomeBootstrapError::unsupported_private_home(
                        path,
                        owner.uid,
                        error.reason().as_str(),
                    )
                })
            }
            #[cfg(not(unix))]
            {
                let _ = owner;
                fs::create_dir_all(path)
                    .map_err(|err| map_io_err(err, format!("create_dir_all {}", path.display())))
            }
        }
        Err(err) => Err(map_io_err(err, format!("metadata {}", path.display()))),
    }
}

#[cfg(not(unix))]
fn ensure_file_if_missing(
    path: &Path,
    contents: &[u8],
    owner: BootstrapOwner,
) -> Result<(), HomeBootstrapError> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.is_file() {
                return Ok(());
            }
            Err(HomeBootstrapError::io(format!(
                "substrate: failed to scaffold deps: expected file at {}, found a directory; remove or rename it and retry",
                path.display()
            )))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            let parent = path.parent().ok_or_else(|| {
                HomeBootstrapError::io(format!(
                    "substrate: failed to scaffold deps: {} has no parent directory",
                    path.display()
                ))
            })?;
            ensure_dir(parent, owner)?;

            let mut file = match fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(path)
            {
                Ok(file) => file,
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                    return ensure_file_exists_as_file(path);
                }
                Err(err) => {
                    return Err(map_io_err(err, format!("create {}", path.display())));
                }
            };

            #[cfg(unix)]
            {
                // Ownership/mode convergence is limited to the file this call created with
                // O_EXCL. Existing scaffold files are never modified.
                // SAFETY: geteuid has no preconditions and file owns a live descriptor.
                let effective_uid = unsafe { libc::geteuid() };
                if effective_uid != owner.uid
                    && (effective_uid != 0
                        // SAFETY: the descriptor is the exclusively created file; gid -1 is kept.
                        || unsafe { libc::fchown(file.as_raw_fd(), owner.uid, !0 as libc::gid_t) }
                            != 0)
                {
                    return Err(HomeBootstrapError::unsupported_private_home(
                        path,
                        owner.uid,
                        "wrong-owner",
                    ));
                }
                // SAFETY: the descriptor is the exclusively created file.
                if unsafe { libc::fchmod(file.as_raw_fd(), 0o600) } != 0 {
                    return Err(map_io_err(
                        io::Error::last_os_error(),
                        format!("set private mode on {}", path.display()),
                    ));
                }
            }

            file.write_all(contents)
                .map_err(|err| map_io_err(err, format!("write {}", path.display())))?;
            Ok(())
        }
        Err(err) => Err(map_io_err(err, format!("metadata {}", path.display()))),
    }
}

#[cfg(not(unix))]
fn ensure_file_exists_as_file(path: &Path) -> Result<(), HomeBootstrapError> {
    match fs::metadata(path) {
        Ok(meta) if meta.is_file() => Ok(()),
        Ok(_) => Err(HomeBootstrapError::io(format!(
            "substrate: failed to scaffold deps: expected file at {}, found a directory; remove or rename it and retry",
            path.display()
        ))),
        Err(err) => Err(map_io_err(err, format!("metadata {}", path.display()))),
    }
}

#[cfg(not(unix))]
fn map_io_err(err: io::Error, context: String) -> HomeBootstrapError {
    if is_denied_io(&err) {
        return HomeBootstrapError::denied(format!(
            "substrate: denied writing to SUBSTRATE_HOME while scaffolding deps ({context}): {err}"
        ));
    }
    HomeBootstrapError::io(format!(
        "substrate: failed to scaffold deps ({context}): {err}"
    ))
}

#[cfg(not(unix))]
fn is_denied_io(err: &io::Error) -> bool {
    if err.kind() == io::ErrorKind::PermissionDenied {
        return true;
    }
    #[cfg(unix)]
    {
        if let Some(code) = err.raw_os_error() {
            return code == libc::EACCES || code == libc::EPERM || code == libc::EROFS;
        }
    }
    false
}

#[cfg(all(test, unix))]
mod tests {
    use super::{select_intended_owner_name, HomeBootstrapError};
    use crate::execution::agent_runtime::host_session_authority::trusted_fs::{
        PrivateHomeAclAuthority, PrivateHomeAclKind, PrivateHomeCandidateProvenance,
        PrivateHomeError, PrivateHomeObjectRole,
    };
    use std::ffi::OsString;
    #[cfg(target_os = "linux")]
    use std::fs::{self, File};
    #[cfg(target_os = "linux")]
    use std::os::fd::AsRawFd;
    #[cfg(target_os = "linux")]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(target_os = "linux")]
    fn set_zero_effective_default_acl(path: &std::path::Path, principal_id: libc::uid_t) {
        let mut default_acl = 2_u32.to_le_bytes().to_vec();
        for (tag, permissions, identifier) in [
            (0x0001_u16, 0o7_u16, u32::MAX),
            (0x0002, 0, principal_id),
            (0x0004, 0, u32::MAX),
            (0x0010, 0, u32::MAX),
            (0x0020, 0, u32::MAX),
        ] {
            default_acl.extend_from_slice(&tag.to_le_bytes());
            default_acl.extend_from_slice(&permissions.to_le_bytes());
            default_acl.extend_from_slice(&identifier.to_le_bytes());
        }
        let candidate = File::open(path).unwrap();
        // SAFETY: candidate is live and default_acl points to initialized storage.
        assert_eq!(
            unsafe {
                libc::fsetxattr(
                    candidate.as_raw_fd(),
                    c"system.posix_acl_default".as_ptr(),
                    default_acl.as_ptr().cast(),
                    default_acl.len(),
                    0,
                )
            },
            0,
            "{}",
            std::io::Error::last_os_error()
        );
    }

    #[test]
    fn malformed_unavailable_and_unknown_acl_diagnostics_are_typed_and_conservative() {
        let requested = std::path::Path::new("/private/requested-home");
        let offending = std::path::Path::new("/private");
        for (kind, authority, provenance, expected_kind, expected_authority) in [
            (
                PrivateHomeAclKind::Access,
                PrivateHomeAclAuthority::Malformed,
                PrivateHomeCandidateProvenance::NotCreated,
                "acl-kind=access",
                "authority=malformed-acl",
            ),
            (
                PrivateHomeAclKind::Unavailable,
                PrivateHomeAclAuthority::Unavailable,
                PrivateHomeCandidateProvenance::NotCreated,
                "acl-kind=unavailable",
                "authority=acl-validation-unavailable",
            ),
            (
                PrivateHomeAclKind::Unavailable,
                PrivateHomeAclAuthority::Unavailable,
                PrivateHomeCandidateProvenance::Unknown,
                "acl-kind=unavailable",
                "authority=acl-validation-unavailable",
            ),
        ] {
            let private_error = PrivateHomeError::acl_diagnostic_for_test(
                requested,
                offending,
                PrivateHomeObjectRole::Ancestor,
                kind,
                authority,
                provenance,
            );
            let rendered = HomeBootstrapError::unsupported_private_home(
                requested,
                // This value must never be rendered.
                4_294_000_001,
                &private_error,
            )
            .to_string();
            assert!(rendered.contains(expected_kind));
            assert!(rendered.contains(expected_authority));
            let expected_observation = "acl-observation=failed-or-unavailable";
            assert!(rendered.contains(expected_observation));
            assert!(rendered.contains(&format!(
                "candidate-created={}",
                provenance.creation_as_str()
            )));
            assert!(!rendered.to_ascii_lowercase().contains("acl absent"));
            assert!(!rendered.to_ascii_lowercase().contains("acl-free"));
            assert!(!rendered.contains("4294000001"));
            assert!(!rendered.contains("principal"));
            assert!(!rendered.contains("credential"));
            assert!(!rendered.contains("session"));
            if provenance == PrivateHomeCandidateProvenance::Unknown {
                assert!(rendered.contains("performs no cleanup"));
                assert!(!rendered.contains("reset it manually"));
            }
        }
    }

    #[test]
    fn absent_final_root_creation_failure_does_not_claim_an_existing_root() {
        let requested = std::path::Path::new("/private/not-created-home");
        let private_error = PrivateHomeError::acl_diagnostic_for_test(
            requested,
            requested,
            PrivateHomeObjectRole::FinalRoot,
            PrivateHomeAclKind::Unavailable,
            PrivateHomeAclAuthority::Unavailable,
            PrivateHomeCandidateProvenance::NotCreated,
        );

        let rendered =
            HomeBootstrapError::unsupported_private_home(requested, 1000, &private_error)
                .to_string();
        assert!(rendered.contains("candidate-created=no"));
        assert!(rendered.contains("No candidate was created"));
        assert!(!rendered.contains("Existing roots are never repaired"));
        assert!(!rendered.contains("reset it manually"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn private_home_diagnostic_identifies_ancestor_acl_without_principal_disclosure() {
        // SAFETY: geteuid has no preconditions.
        let owner_uid = unsafe { libc::geteuid() };
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(format!("/run/user/{owner_uid}")));
        let temp = tempfile::Builder::new()
            .prefix("substrate-a1-home-diagnostic-")
            .tempdir_in(safe_parent)
            .unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let principal_id = owner_uid.saturating_add(7_919);
        let mut acl = 2_u32.to_le_bytes().to_vec();
        for (tag, permissions, identifier) in [
            (0x0001_u16, 0o7_u16, u32::MAX),
            (0x0002, 0o2, principal_id),
            (0x0004, 0, u32::MAX),
            (0x0010, 0o2, u32::MAX),
            (0x0020, 0, u32::MAX),
        ] {
            acl.extend_from_slice(&tag.to_le_bytes());
            acl.extend_from_slice(&permissions.to_le_bytes());
            acl.extend_from_slice(&identifier.to_le_bytes());
        }
        let directory = File::open(temp.path()).unwrap();
        // SAFETY: directory is live and acl points to initialized storage.
        assert_eq!(
            unsafe {
                libc::fsetxattr(
                    directory.as_raw_fd(),
                    c"system.posix_acl_access".as_ptr(),
                    acl.as_ptr().cast(),
                    acl.len(),
                    0,
                )
            },
            0,
            "{}",
            std::io::Error::last_os_error()
        );
        let requested = temp.path().join("home");
        let private_error = crate::execution::agent_runtime::host_session_authority::trusted_fs::ensure_private_substrate_home(
            &requested,
            owner_uid,
        )
        .unwrap_err();

        let diagnostic =
            HomeBootstrapError::unsupported_private_home(&requested, owner_uid, &private_error);
        assert_eq!(diagnostic.exit_code(), 5);
        let rendered = diagnostic.to_string();
        assert!(rendered.contains(&requested.display().to_string()));
        assert!(rendered.contains(&temp.path().display().to_string()));
        assert!(rendered.contains("role=ancestor"));
        assert!(rendered.contains("acl-kind=access"));
        assert!(rendered.contains("authority=effective-write"));
        assert!(rendered.contains("candidate-created=no"));
        assert!(rendered.contains("offending ancestor was not modified"));
        assert!(!rendered.contains("reset it manually"));
        assert!(!rendered.contains(&principal_id.to_string()));
        assert!(!rendered.contains("session"));
        assert!(!rendered.contains("credential"));
        assert!(!requested.exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn current_attempt_candidate_diagnostic_omits_existing_root_reset_instruction() {
        // SAFETY: geteuid has no preconditions.
        let owner_uid = unsafe { libc::geteuid() };
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(format!("/run/user/{owner_uid}")));
        let temp = tempfile::Builder::new()
            .prefix("substrate-a1-created-home-diagnostic-")
            .tempdir_in(safe_parent)
            .unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let requested = temp.path().join("home");
        let mut default_acl = 2_u32.to_le_bytes().to_vec();
        for (tag, permissions, identifier) in [
            (0x0001_u16, 0o7_u16, u32::MAX),
            (0x0002, 0, owner_uid.saturating_add(1)),
            (0x0004, 0, u32::MAX),
            (0x0010, 0, u32::MAX),
            (0x0020, 0, u32::MAX),
        ] {
            default_acl.extend_from_slice(&tag.to_le_bytes());
            default_acl.extend_from_slice(&permissions.to_le_bytes());
            default_acl.extend_from_slice(&identifier.to_le_bytes());
        }
        let private_error = crate::execution::agent_runtime::host_session_authority::trusted_fs::ensure_private_substrate_home_with_after_create_for_test(
            &requested,
            owner_uid,
            || {
                let candidate = File::open(&requested).unwrap();
                // SAFETY: candidate is live and default_acl points to initialized storage.
                assert_eq!(unsafe {
                    libc::fsetxattr(
                        candidate.as_raw_fd(),
                        c"system.posix_acl_default".as_ptr(),
                        default_acl.as_ptr().cast(),
                        default_acl.len(),
                        0,
                    )
                }, 0, "{}", std::io::Error::last_os_error());
            },
        )
        .unwrap_err();

        let diagnostic =
            HomeBootstrapError::unsupported_private_home(&requested, owner_uid, &private_error);
        assert_eq!(diagnostic.exit_code(), 5);
        let rendered = diagnostic.to_string();
        assert!(rendered.contains("role=final-root"));
        assert!(rendered.contains("acl-kind=default"));
        assert!(rendered.contains("candidate-created=yes"));
        assert!(rendered.contains("performs no cleanup"));
        assert!(!rendered.contains("reset it manually"));
        assert!(requested.is_dir());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn construction_revalidation_preserves_typed_acl_race_diagnostic() {
        // SAFETY: geteuid has no preconditions.
        let owner_uid = unsafe { libc::geteuid() };
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(format!("/run/user/{owner_uid}")));
        let temp = tempfile::Builder::new()
            .prefix("substrate-a1-construction-revalidate-")
            .tempdir_in(safe_parent)
            .unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let requested = temp.path().join("home");

        let private_error = crate::execution::agent_runtime::host_session_authority::trusted_fs::ensure_private_substrate_home_with_before_from_opened_for_test(
            &requested,
            owner_uid,
            || set_zero_effective_default_acl(&requested, owner_uid.saturating_add(1)),
        )
        .unwrap_err();

        assert_eq!(
            private_error.object_role(),
            Some(PrivateHomeObjectRole::FinalRoot)
        );
        assert_eq!(private_error.acl_kind(), Some(PrivateHomeAclKind::Default));
        assert_eq!(
            private_error.acl_authority(),
            Some(PrivateHomeAclAuthority::DefaultAclPresent)
        );
        assert_eq!(
            private_error.candidate_provenance(),
            PrivateHomeCandidateProvenance::Created
        );
        let rendered =
            HomeBootstrapError::unsupported_private_home(&requested, owner_uid, &private_error)
                .to_string();
        assert!(rendered.contains("role=final-root"));
        assert!(rendered.contains("acl-kind=default"));
        assert!(rendered.contains("authority=default-acl-present"));
        assert!(rendered.contains("candidate-created=yes"));
        assert!(rendered.contains("performs no cleanup"));
        assert!(!rendered.contains("reset it manually"));
        assert!(requested.is_dir());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn post_scaffold_revalidation_preserves_typed_candidate_provenance() {
        // SAFETY: geteuid has no preconditions.
        let owner_uid = unsafe { libc::geteuid() };
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(format!("/run/user/{owner_uid}")));
        let temp = tempfile::Builder::new()
            .prefix("substrate-a1-revalidate-diagnostic-")
            .tempdir_in(safe_parent)
            .unwrap();
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700)).unwrap();

        for (label, preexisting, expected_provenance) in [
            ("created", false, PrivateHomeCandidateProvenance::Created),
            (
                "existing",
                true,
                PrivateHomeCandidateProvenance::PreExisting,
            ),
        ] {
            let requested = temp.path().join(label);
            if preexisting {
                fs::create_dir(&requested).unwrap();
                fs::set_permissions(&requested, fs::Permissions::from_mode(0o700)).unwrap();
            }
            let trusted_home = crate::execution::agent_runtime::host_session_authority::trusted_fs::ensure_private_substrate_home(
                &requested,
                owner_uid,
            )
            .unwrap();
            set_zero_effective_default_acl(&requested, owner_uid.saturating_add(1));

            let private_error = trusted_home.revalidate_private_home().unwrap_err();
            assert_eq!(
                private_error.object_role(),
                Some(PrivateHomeObjectRole::FinalRoot)
            );
            assert_eq!(private_error.acl_kind(), Some(PrivateHomeAclKind::Default));
            assert_eq!(
                private_error.acl_authority(),
                Some(PrivateHomeAclAuthority::DefaultAclPresent)
            );
            assert_eq!(private_error.candidate_provenance(), expected_provenance);
            let rendered =
                HomeBootstrapError::unsupported_private_home(&requested, owner_uid, &private_error)
                    .to_string();
            assert!(rendered.contains("role=final-root"));
            assert!(rendered.contains("acl-kind=default"));
            assert!(rendered.contains(&format!(
                "candidate-created={}",
                expected_provenance.creation_as_str()
            )));
            if preexisting {
                assert!(rendered.contains("reset it manually"));
            } else {
                assert!(rendered.contains("performs no cleanup"));
                assert!(!rendered.contains("reset it manually"));
            }
        }
    }

    #[test]
    fn explicit_install_owner_precedes_sudo_user() {
        assert_eq!(
            select_intended_owner_name(
                Some(OsString::from("explicit")),
                Some(OsString::from("sudo"))
            ),
            Some(OsString::from("explicit"))
        );
    }

    #[test]
    fn sudo_user_is_the_privileged_owner_fallback() {
        assert_eq!(
            select_intended_owner_name(Some(OsString::from("root")), Some(OsString::from("sudo"))),
            Some(OsString::from("sudo"))
        );
    }

    #[test]
    fn privileged_owner_selection_fails_when_both_signals_are_ambiguous() {
        assert_eq!(
            select_intended_owner_name(Some(OsString::new()), Some(OsString::from("root"))),
            None
        );
    }
}
