//! Filesystem diff tracking for command execution.
//!
//! This module provides a unified filesystem diff type used by both
//! the trace and world modules to track filesystem changes.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Filesystem changes detected after command execution.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FsDiff {
    /// Files that were written/created.
    /// Serialized as strings for JSON compatibility.
    #[serde(
        serialize_with = "serialize_paths",
        deserialize_with = "deserialize_paths"
    )]
    pub writes: Vec<PathBuf>,

    /// Files that were modified.
    #[serde(
        serialize_with = "serialize_paths",
        deserialize_with = "deserialize_paths"
    )]
    pub mods: Vec<PathBuf>,

    /// Files that were deleted.
    #[serde(
        serialize_with = "serialize_paths",
        deserialize_with = "deserialize_paths"
    )]
    pub deletes: Vec<PathBuf>,

    /// Whether the diff was truncated due to size limits.
    #[serde(default, skip_serializing_if = "is_false")]
    pub truncated: bool,

    /// Hash of the directory tree when truncated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree_hash: Option<String>,

    /// Human-readable summary of changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl FsDiff {
    /// Check if the diff is empty (no changes).
    pub fn is_empty(&self) -> bool {
        self.writes.is_empty() && self.mods.is_empty() && self.deletes.is_empty()
    }

    /// Get total number of changes.
    pub fn total_changes(&self) -> usize {
        self.writes.len() + self.mods.len() + self.deletes.len()
    }

    /// Create a simple diff for testing.
    #[cfg(test)]
    pub fn simple(writes: Vec<&str>, mods: Vec<&str>, deletes: Vec<&str>) -> Self {
        Self {
            writes: writes.into_iter().map(PathBuf::from).collect(),
            mods: mods.into_iter().map(PathBuf::from).collect(),
            deletes: deletes.into_iter().map(PathBuf::from).collect(),
            ..Default::default()
        }
    }
}

fn is_false(b: &bool) -> bool {
    !b
}

fn serialize_paths<S>(paths: &[PathBuf], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let strings: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    strings.serialize(serializer)
}

fn deserialize_paths<'de, D>(deserializer: D) -> Result<Vec<PathBuf>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    Ok(strings.into_iter().map(PathBuf::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_diff_empty() {
        let diff = FsDiff::default();
        assert!(diff.is_empty());
        assert_eq!(diff.total_changes(), 0);
    }

    #[test]
    fn test_fs_diff_with_changes() {
        let diff = FsDiff::simple(
            vec!["new_file.txt"],
            vec!["modified.rs"],
            vec!["old_file.log"],
        );
        assert!(!diff.is_empty());
        assert_eq!(diff.total_changes(), 3);
    }

    #[test]
    fn test_fs_diff_serialization() {
        let diff = FsDiff {
            writes: vec![PathBuf::from("/tmp/test.txt")],
            mods: vec![],
            deletes: vec![],
            truncated: false,
            tree_hash: None,
            summary: None,
        };

        let json = serde_json::to_string(&diff).unwrap();
        assert!(json.contains("\"/tmp/test.txt\""));
        assert!(!json.contains("truncated")); // Should be skipped when false

        let deserialized: FsDiff = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.writes.len(), 1);
        assert_eq!(deserialized.writes[0], PathBuf::from("/tmp/test.txt"));
    }
}
