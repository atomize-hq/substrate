use std::{
    env, fs as std_fs,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use thiserror::Error;

/// Specification for resolving an app-bundled Codex binary.
///
/// Callers supply a bundle root plus the pinned version they expect. Platform
/// defaults to the current target triple label (e.g., `darwin-arm64` or
/// `linux-x64`) but can be overridden when hosts manage their own layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundledBinarySpec<'a> {
    /// Root containing `<platform>/<version>/codex` slices managed by the host.
    pub bundle_root: &'a Path,
    /// Pinned Codex version directory to resolve (semantic version or channel/build id).
    pub version: &'a str,
    /// Optional platform label override; defaults to [`default_bundled_platform_label`].
    pub platform: Option<&'a str>,
}

/// Resolved bundled Codex binary details.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundledBinary {
    /// Canonicalized path to the bundled Codex binary (`codex` or `codex.exe`).
    pub binary_path: PathBuf,
    /// Platform slice resolved under the bundle root.
    pub platform: String,
    /// Version slice resolved under the platform directory.
    pub version: String,
}

/// Errors that may occur while resolving a bundled Codex binary.
#[derive(Debug, Error)]
pub enum BundledBinaryError {
    #[error("bundled Codex version cannot be empty")]
    EmptyVersion,
    #[error("bundled Codex platform label cannot be empty")]
    EmptyPlatform,
    #[error("bundle root `{bundle_root}` does not exist or is unreadable")]
    BundleRootUnreadable {
        bundle_root: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("bundle root `{bundle_root}` is not a directory")]
    BundleRootNotDirectory { bundle_root: PathBuf },
    #[error("bundle platform directory `{platform_dir}` for `{platform}` does not exist or is unreadable")]
    PlatformUnreadable {
        platform: String,
        platform_dir: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("bundle platform directory `{platform_dir}` for `{platform}` is not a directory")]
    PlatformNotDirectory {
        platform: String,
        platform_dir: PathBuf,
    },
    #[error(
        "bundle version directory `{version_dir}` for `{version}` does not exist or is unreadable"
    )]
    VersionUnreadable {
        version: String,
        version_dir: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("bundle version directory `{version_dir}` for `{version}` is not a directory")]
    VersionNotDirectory {
        version: String,
        version_dir: PathBuf,
    },
    #[error("bundled Codex binary `{binary}` is missing or unreadable")]
    BinaryUnreadable {
        binary: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("bundled Codex binary `{binary}` is not a file")]
    BinaryNotFile { binary: PathBuf },
    #[error("bundled Codex binary `{binary}` is not executable")]
    BinaryNotExecutable { binary: PathBuf },
    #[error("failed to canonicalize bundled Codex binary `{path}`: {source}")]
    Canonicalize {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Resolves a bundled Codex binary under `<bundle_root>/<platform>/<version>/`.
///
/// The helper never consults `PATH` or `CODEX_BINARY`; missing slices are hard
/// errors. The resolved path is canonicalized and should be passed to
/// [`CodexClientBuilder::binary`] to keep behavior isolated from any global
/// Codex install.
pub fn resolve_bundled_binary(
    spec: BundledBinarySpec<'_>,
) -> Result<BundledBinary, BundledBinaryError> {
    let platform = match spec.platform {
        Some(label) => {
            super::normalize_non_empty(label).ok_or(BundledBinaryError::EmptyPlatform)?
        }
        None => default_bundled_platform_label(),
    };
    let version =
        super::normalize_non_empty(spec.version).ok_or(BundledBinaryError::EmptyVersion)?;

    require_directory(
        spec.bundle_root,
        |source| BundledBinaryError::BundleRootUnreadable {
            bundle_root: spec.bundle_root.to_path_buf(),
            source,
        },
        || BundledBinaryError::BundleRootNotDirectory {
            bundle_root: spec.bundle_root.to_path_buf(),
        },
    )?;

    let platform_dir = spec.bundle_root.join(&platform);
    require_directory(
        &platform_dir,
        |source| BundledBinaryError::PlatformUnreadable {
            platform: platform.clone(),
            platform_dir: platform_dir.clone(),
            source,
        },
        || BundledBinaryError::PlatformNotDirectory {
            platform: platform.clone(),
            platform_dir: platform_dir.clone(),
        },
    )?;

    let version_dir = platform_dir.join(&version);
    require_directory(
        &version_dir,
        |source| BundledBinaryError::VersionUnreadable {
            version: version.clone(),
            version_dir: version_dir.clone(),
            source,
        },
        || BundledBinaryError::VersionNotDirectory {
            version: version.clone(),
            version_dir: version_dir.clone(),
        },
    )?;

    let binary_path = version_dir.join(bundled_binary_filename(&platform));
    let metadata =
        std_fs::metadata(&binary_path).map_err(|source| BundledBinaryError::BinaryUnreadable {
            binary: binary_path.clone(),
            source,
        })?;
    if !metadata.is_file() {
        return Err(BundledBinaryError::BinaryNotFile {
            binary: binary_path.clone(),
        });
    }
    ensure_executable(&metadata, &binary_path)?;

    let canonical =
        std_fs::canonicalize(&binary_path).map_err(|source| BundledBinaryError::Canonicalize {
            path: binary_path.clone(),
            source,
        })?;

    Ok(BundledBinary {
        binary_path: canonical,
        platform,
        version,
    })
}

/// Default bundled platform label for the current target (e.g., `darwin-arm64`, `linux-x64`, `windows-x64`).
pub fn default_bundled_platform_label() -> String {
    let os = match env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}-{arch}")
}

fn require_directory(
    path: &Path,
    on_read_error: impl FnOnce(std::io::Error) -> BundledBinaryError,
    on_wrong_type: impl FnOnce() -> BundledBinaryError,
) -> Result<(), BundledBinaryError> {
    let metadata = std_fs::metadata(path).map_err(on_read_error)?;
    if !metadata.is_dir() {
        return Err(on_wrong_type());
    }
    Ok(())
}

fn ensure_executable(metadata: &std_fs::Metadata, binary: &Path) -> Result<(), BundledBinaryError> {
    if binary_is_executable(metadata) {
        return Ok(());
    }
    Err(BundledBinaryError::BinaryNotExecutable {
        binary: binary.to_path_buf(),
    })
}

fn binary_is_executable(metadata: &std_fs::Metadata) -> bool {
    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        // Windows does not use executable bits; existence is sufficient.
        true
    }
}

pub(super) fn bundled_binary_filename(platform: &str) -> &'static str {
    if platform.to_ascii_lowercase().contains("windows") {
        "codex.exe"
    } else {
        "codex"
    }
}
