use std::fmt;
use std::str::FromStr;

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
