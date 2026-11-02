use serde::{de::DeserializeOwned, Serialize};
use std::fs as std_fs;
use std::path::Path;

use super::{
    capability_cache_key, current_fingerprint, fingerprints_match, has_fingerprint_metadata,
    CapabilityOverrides, CapabilitySnapshotError, CapabilitySnapshotFormat, CodexCapabilities,
};

/// Serializes a capability snapshot to a JSON or TOML string.
pub fn serialize_capabilities_snapshot(
    snapshot: &CodexCapabilities,
    format: CapabilitySnapshotFormat,
) -> Result<String, CapabilitySnapshotError> {
    serialize_snapshot(snapshot, format)
}

/// Parses a capability snapshot from serialized JSON or TOML.
pub fn deserialize_capabilities_snapshot(
    input: &str,
    format: CapabilitySnapshotFormat,
) -> Result<CodexCapabilities, CapabilitySnapshotError> {
    deserialize_snapshot(input, format)
}

/// Writes a capability snapshot to disk, inferring format from the file extension when absent.
pub fn write_capabilities_snapshot(
    path: impl AsRef<Path>,
    snapshot: &CodexCapabilities,
    format: Option<CapabilitySnapshotFormat>,
) -> Result<(), CapabilitySnapshotError> {
    let path = path.as_ref();
    let resolved_format = resolve_snapshot_format(format, path)?;
    let contents = serialize_capabilities_snapshot(snapshot, resolved_format)?;
    std_fs::write(path, contents).map_err(|source| CapabilitySnapshotError::WriteSnapshot {
        path: path.to_path_buf(),
        source,
    })
}

/// Loads a capability snapshot from disk, inferring format from the file extension when absent.
pub fn read_capabilities_snapshot(
    path: impl AsRef<Path>,
    format: Option<CapabilitySnapshotFormat>,
) -> Result<CodexCapabilities, CapabilitySnapshotError> {
    let path = path.as_ref();
    let resolved_format = resolve_snapshot_format(format, path)?;
    let contents =
        std_fs::read_to_string(path).map_err(|source| CapabilitySnapshotError::ReadSnapshot {
            path: path.to_path_buf(),
            source,
        })?;
    deserialize_capabilities_snapshot(&contents, resolved_format)
}

/// Serializes capability overrides (snapshot, version, feature flags) to a JSON or TOML string.
pub fn serialize_capability_overrides(
    overrides: &CapabilityOverrides,
    format: CapabilitySnapshotFormat,
) -> Result<String, CapabilitySnapshotError> {
    serialize_snapshot(overrides, format)
}

/// Parses capability overrides from serialized JSON or TOML.
pub fn deserialize_capability_overrides(
    input: &str,
    format: CapabilitySnapshotFormat,
) -> Result<CapabilityOverrides, CapabilitySnapshotError> {
    deserialize_snapshot(input, format)
}

/// Writes capability overrides to disk, inferring format from the file extension when absent.
pub fn write_capability_overrides(
    path: impl AsRef<Path>,
    overrides: &CapabilityOverrides,
    format: Option<CapabilitySnapshotFormat>,
) -> Result<(), CapabilitySnapshotError> {
    let path = path.as_ref();
    let resolved_format = resolve_snapshot_format(format, path)?;
    let contents = serialize_capability_overrides(overrides, resolved_format)?;
    std_fs::write(path, contents).map_err(|source| CapabilitySnapshotError::WriteSnapshot {
        path: path.to_path_buf(),
        source,
    })
}

/// Reads capability overrides from disk, inferring format from the file extension when absent.
pub fn read_capability_overrides(
    path: impl AsRef<Path>,
    format: Option<CapabilitySnapshotFormat>,
) -> Result<CapabilityOverrides, CapabilitySnapshotError> {
    let path = path.as_ref();
    let resolved_format = resolve_snapshot_format(format, path)?;
    let contents =
        std_fs::read_to_string(path).map_err(|source| CapabilitySnapshotError::ReadSnapshot {
            path: path.to_path_buf(),
            source,
        })?;
    deserialize_capability_overrides(&contents, resolved_format)
}

/// True when the snapshot was captured for the same binary path and fingerprint.
///
/// Hosts can consult this before applying a serialized snapshot to avoid
/// reusing stale capability data after binary upgrades.
pub fn capability_snapshot_matches_binary(snapshot: &CodexCapabilities, binary: &Path) -> bool {
    let cache_key = capability_cache_key(binary);
    if snapshot.cache_key != cache_key {
        return false;
    }
    let current = current_fingerprint(&cache_key);
    has_fingerprint_metadata(&snapshot.fingerprint)
        && has_fingerprint_metadata(&current)
        && fingerprints_match(&snapshot.fingerprint, &current)
}

fn serialize_snapshot<T: Serialize>(
    value: &T,
    format: CapabilitySnapshotFormat,
) -> Result<String, CapabilitySnapshotError> {
    match format {
        CapabilitySnapshotFormat::Json => serde_json::to_string_pretty(value)
            .map_err(|source| CapabilitySnapshotError::JsonEncode { source }),
        CapabilitySnapshotFormat::Toml => toml::to_string_pretty(value)
            .map_err(|source| CapabilitySnapshotError::TomlEncode { source }),
    }
}

fn deserialize_snapshot<T: DeserializeOwned>(
    input: &str,
    format: CapabilitySnapshotFormat,
) -> Result<T, CapabilitySnapshotError> {
    match format {
        CapabilitySnapshotFormat::Json => serde_json::from_str(input)
            .map_err(|source| CapabilitySnapshotError::JsonDecode { source }),
        CapabilitySnapshotFormat::Toml => {
            toml::from_str(input).map_err(|source| CapabilitySnapshotError::TomlDecode { source })
        }
    }
}

fn resolve_snapshot_format(
    format: Option<CapabilitySnapshotFormat>,
    path: &Path,
) -> Result<CapabilitySnapshotFormat, CapabilitySnapshotError> {
    format
        .or_else(|| CapabilitySnapshotFormat::from_path(path))
        .ok_or_else(|| CapabilitySnapshotError::UnsupportedFormat {
            path: path.to_path_buf(),
        })
}
