use std::fmt;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

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

    ensure_dir(&substrate_home)?;

    let deps_root = substrate_home.join("deps");
    ensure_dir(&deps_root)?;

    let packages_dir = deps_root.join("packages");
    let bundles_dir = deps_root.join("bundles");
    let scripts_dir = deps_root.join("scripts");
    ensure_dir(&packages_dir)?;
    ensure_dir(&bundles_dir)?;
    ensure_dir(&scripts_dir)?;

    ensure_file_if_missing(&deps_root.join("README.md"), DEPS_README_MD.as_bytes())?;
    ensure_file_if_missing(
        &packages_dir.join("example-manual.yaml"),
        EXAMPLE_MANUAL_YAML.as_bytes(),
    )?;
    ensure_file_if_missing(
        &packages_dir.join("example-script.yaml"),
        EXAMPLE_SCRIPT_YAML.as_bytes(),
    )?;
    ensure_file_if_missing(
        &packages_dir.join("example-apt.yaml"),
        EXAMPLE_APT_YAML.as_bytes(),
    )?;
    ensure_file_if_missing(
        &bundles_dir.join("example-bundle.yaml"),
        EXAMPLE_BUNDLE_YAML.as_bytes(),
    )?;
    ensure_file_if_missing(
        &scripts_dir.join("example-install.sh"),
        EXAMPLE_INSTALL_SH.as_bytes(),
    )?;

    Ok(())
}

fn ensure_dir(path: &Path) -> Result<(), HomeBootstrapError> {
    match fs::metadata(path) {
        Ok(meta) => {
            if meta.is_dir() {
                return Ok(());
            }
            Err(HomeBootstrapError::io(format!(
                "substrate: failed to scaffold deps: expected directory at {}, found a file; remove or rename it and retry",
                path.display()
            )))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => fs::create_dir_all(path)
            .map_err(|err| map_io_err(err, format!("create_dir_all {}", path.display()))),
        Err(err) => Err(map_io_err(err, format!("metadata {}", path.display()))),
    }
}

fn ensure_file_if_missing(path: &Path, contents: &[u8]) -> Result<(), HomeBootstrapError> {
    match fs::metadata(path) {
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
            ensure_dir(parent)?;

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

            file.write_all(contents)
                .map_err(|err| map_io_err(err, format!("write {}", path.display())))?;
            Ok(())
        }
        Err(err) => Err(map_io_err(err, format!("metadata {}", path.display()))),
    }
}

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
