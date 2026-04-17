use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::kernel::RepoPath;
use crate::repo::{RepoError, RepoResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum SymlinkPolicy {
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum NonUtf8PathPolicy {
    Error,
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum LargeFilePolicy {
    Error,
    Skip,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotOptions {
    pub symlink_policy: SymlinkPolicy,
    pub exclude_globs: Vec<String>,
    pub non_utf8_path_policy: NonUtf8PathPolicy,
    pub max_file_bytes: Option<u64>,
    pub large_file_policy: LargeFilePolicy,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            symlink_policy: SymlinkPolicy::Skip,
            exclude_globs: Vec::new(),
            non_utf8_path_policy: NonUtf8PathPolicy::Error,
            max_file_bytes: None,
            large_file_policy: LargeFilePolicy::Error,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompiledIgnoreSet {
    patterns: Vec<String>,
    matcher: GlobSet,
}

impl PartialEq for CompiledIgnoreSet {
    fn eq(&self, other: &Self) -> bool {
        self.patterns == other.patterns
    }
}

impl Eq for CompiledIgnoreSet {}

impl CompiledIgnoreSet {
    pub(crate) fn compile(exclude_globs: &[String]) -> RepoResult<Self> {
        let mut builder = GlobSetBuilder::new();
        for pattern in exclude_globs {
            let glob = Glob::new(pattern).map_err(|error| RepoError::IgnoreGlobCompile {
                pattern: pattern.clone(),
                reason: error.to_string(),
            })?;
            builder.add(glob);
        }

        let matcher = builder
            .build()
            .map_err(|error| RepoError::IgnoreGlobCompile {
                pattern: "<set>".to_owned(),
                reason: error.to_string(),
            })?;
        Ok(Self {
            patterns: exclude_globs.to_vec(),
            matcher,
        })
    }

    pub(crate) fn is_ignored(&self, repo_path: &RepoPath, is_dir: bool) -> bool {
        if is_intrinsic_git_path(repo_path) {
            return true;
        }

        let path = Path::new(repo_path.as_str());
        self.matcher.is_match(path)
            || (is_dir
                && self
                    .matcher
                    .is_match(Path::new(&format!("{}/", repo_path.as_str()))))
    }
}

fn is_intrinsic_git_path(repo_path: &RepoPath) -> bool {
    repo_path.as_str() == ".git" || repo_path.as_str().starts_with(".git/")
}
