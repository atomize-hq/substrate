use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Selects whether a world filesystem should be writable (overlay/copy-diff)
/// or mounted read-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldFsMode {
    Writable,
    ReadOnly,
}

impl WorldFsMode {
    /// Convert the mode to its canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Writable => "writable",
            Self::ReadOnly => "read_only",
        }
    }

    /// Parse a world fs mode string (case-insensitive).
    pub fn parse(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl Default for WorldFsMode {
    fn default() -> Self {
        Self::Writable
    }
}

impl FromStr for WorldFsMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim().to_ascii_lowercase().replace(['_', ' '], "-");
        match normalized.as_str() {
            "writable" | "writeable" => Ok(Self::Writable),
            "read-only" | "readonly" | "ro" => Ok(Self::ReadOnly),
            other => Err(format!(
                "invalid world fs mode: {} (expected writable or read_only)",
                other
            )),
        }
    }
}

impl<'de> Deserialize<'de> for WorldFsMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

impl Serialize for WorldFsMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Configures how the world determines its root directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldRootMode {
    Project,
    FollowCwd,
    Custom,
}

impl WorldRootMode {
    /// Convert mode to its canonical string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::FollowCwd => "follow-cwd",
            Self::Custom => "custom",
        }
    }

    /// Parse a world root mode string (case-insensitive).
    pub fn parse(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromStr for WorldRootMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "project" => Ok(Self::Project),
            "follow-cwd" | "follow_cwd" => Ok(Self::FollowCwd),
            "custom" => Ok(Self::Custom),
            other => Err(format!("invalid world root mode: {}", other)),
        }
    }
}

impl fmt::Display for WorldRootMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_world_fs_mode_variants() {
        assert_eq!(WorldFsMode::Writable.as_str(), "writable");
        assert_eq!(WorldFsMode::ReadOnly.as_str(), "read_only");
        for value in ["writable", "writeable"] {
            assert_eq!(value.parse::<WorldFsMode>().unwrap(), WorldFsMode::Writable);
        }
        for value in ["read-only", "readonly", "ro", "read_only"] {
            assert_eq!(value.parse::<WorldFsMode>().unwrap(), WorldFsMode::ReadOnly);
        }
    }

    #[test]
    fn rejects_invalid_world_fs_mode() {
        let err = "maybe".parse::<WorldFsMode>().unwrap_err();
        assert!(
            err.contains("invalid world fs mode"),
            "unexpected error message: {}",
            err
        );
    }

    #[test]
    fn world_fs_mode_default_writable() {
        assert_eq!(WorldFsMode::default(), WorldFsMode::Writable);
    }
}
