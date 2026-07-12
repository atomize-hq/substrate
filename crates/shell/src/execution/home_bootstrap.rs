use std::fmt;
#[cfg(not(unix))]
use std::fs;
#[cfg(not(unix))]
use std::io::{self, Write};
use std::path::Path;
#[cfg(unix)]
use std::{ffi::CString, mem::MaybeUninit};

use substrate_common::paths as substrate_paths;

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
    fn unsupported_private_home(path: &Path, uid: libc::uid_t, reason: &str) -> Self {
        Self::denied(format!(
            "substrate: unsupported SUBSTRATE_HOME '{}': expected a private directory owned by intended uid {uid} with exact mode 0700 and no foreign ACL grants; found {reason}. Existing roots are never repaired; reset it manually and retry.",
            path.display()
        ))
    }

    #[cfg(unix)]
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
    let owner = BootstrapOwner;
    #[cfg(unix)]
    let trusted_home =
        crate::execution::agent_runtime::host_session_authority::trusted_fs::ensure_private_substrate_home(
            &substrate_home, owner.uid,
        )
        .map_err(|error| {
            HomeBootstrapError::unsupported_private_home(
                &substrate_home,
                owner.uid,
                error.reason().as_str(),
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

        trusted_home.revalidate().map_err(|_| {
            HomeBootstrapError::unsupported_private_home(&substrate_home, owner.uid, "replaced")
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
fn resolve_intended_owner_uid(path: &Path) -> Result<libc::uid_t, HomeBootstrapError> {
    // SAFETY: geteuid has no preconditions.
    let effective_uid = unsafe { libc::geteuid() };
    if effective_uid != 0 {
        return Ok(effective_uid);
    }

    let intended_user = std::env::var_os("SUBSTRATE_INSTALL_PRIMARY_USER")
        .filter(|value| !value.is_empty() && value != "root")
        .or_else(|| {
            std::env::var_os("SUDO_USER").filter(|value| !value.is_empty() && value != "root")
        })
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
