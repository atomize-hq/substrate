//! Deterministic logical repository paths.

use serde::{Deserialize, Serialize};

use crate::kernel::error::{KernelError, KernelResult};

/// Canonical repo-root-relative logical path.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RepoPath(String);

impl RepoPath {
    /// Parses and validates a repo-root-relative logical path.
    pub fn parse(input: &str) -> KernelResult<Self> {
        validate_repo_path(input)?;
        Ok(Self(input.to_owned()))
    }

    /// Returns the normalized string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Joins this path with an already-normalized child path.
    pub fn join(&self, child: &RepoPath) -> KernelResult<Self> {
        Self::parse(&format!("{}/{}", self.0, child.0))
    }

    /// Returns the logical parent path when one exists.
    pub fn parent(&self) -> Option<Self> {
        let (parent, _) = self.0.rsplit_once('/')?;
        Some(Self(parent.to_owned()))
    }
}

impl TryFrom<String> for RepoPath {
    type Error = KernelError;

    fn try_from(value: String) -> KernelResult<Self> {
        Self::parse(&value)
    }
}

impl From<RepoPath> for String {
    fn from(value: RepoPath) -> Self {
        value.0
    }
}

fn validate_repo_path(input: &str) -> KernelResult<()> {
    if input.is_empty() {
        return Err(invalid_repo_path(input, "path must not be empty"));
    }
    if input.starts_with('/') {
        return Err(invalid_repo_path(input, "path must be relative"));
    }
    if input.ends_with('/') {
        return Err(invalid_repo_path(
            input,
            "path must not have a trailing slash",
        ));
    }
    if input.contains('\\') {
        return Err(invalid_repo_path(
            input,
            "path must use forward slashes only",
        ));
    }

    for segment in input.split('/') {
        if segment.is_empty() {
            return Err(invalid_repo_path(
                input,
                "path must not contain empty segments",
            ));
        }
        if segment == "." {
            return Err(invalid_repo_path(
                input,
                "path must not contain '.' segments",
            ));
        }
        if segment == ".." {
            return Err(invalid_repo_path(
                input,
                "path must not contain '..' segments",
            ));
        }
    }

    Ok(())
}

fn invalid_repo_path(input: &str, reason: &str) -> KernelError {
    KernelError::InvalidRepoPath {
        input: input.to_owned(),
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::RepoPath;
    use crate::kernel::error::KernelError;

    #[test]
    fn parses_valid_paths() {
        let path = RepoPath::parse("crates/lift/src/kernel/path.rs").expect("path should parse");
        assert_eq!(path.as_str(), "crates/lift/src/kernel/path.rs");
    }

    #[test]
    fn rejects_invalid_paths() {
        let cases = [
            ("", "path must not be empty"),
            ("/src/lib.rs", "path must be relative"),
            ("src/lib.rs/", "path must not have a trailing slash"),
            ("src//lib.rs", "path must not contain empty segments"),
            ("src/./lib.rs", "path must not contain '.' segments"),
            ("src/../lib.rs", "path must not contain '..' segments"),
            (r"src\lib.rs", "path must use forward slashes only"),
        ];

        for (input, reason) in cases {
            let error = RepoPath::parse(input).expect_err("path should fail");
            assert_eq!(
                error,
                KernelError::InvalidRepoPath {
                    input: input.to_owned(),
                    reason: reason.to_owned(),
                }
            );
        }
    }

    #[test]
    fn join_preserves_logical_normalization() {
        let base = RepoPath::parse("crates/lift").expect("base should parse");
        let child = RepoPath::parse("src/kernel/path.rs").expect("child should parse");

        let joined = base.join(&child).expect("join should succeed");
        assert_eq!(joined.as_str(), "crates/lift/src/kernel/path.rs");
    }

    #[test]
    fn parent_returns_none_for_single_segment_paths() {
        let path = RepoPath::parse("Cargo.toml").expect("path should parse");
        assert_eq!(path.parent(), None);
    }

    #[test]
    fn parent_returns_parent_for_nested_paths() {
        let path = RepoPath::parse("crates/lift/src/kernel").expect("path should parse");
        assert_eq!(
            path.parent().expect("parent should exist").as_str(),
            "crates/lift/src"
        );
    }

    #[test]
    fn serde_round_trip_uses_parser() {
        let parsed: RepoPath = serde_json::from_str("\"src/lib.rs\"").expect("path");
        assert_eq!(parsed.as_str(), "src/lib.rs");
        assert!(serde_json::from_str::<RepoPath>("\"src//lib.rs\"").is_err());
    }
}
