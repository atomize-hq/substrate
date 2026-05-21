use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::kernel::RepoPath;
use crate::repo::{RepoError, RepoResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum SymlinkPolicy {
    Skip,
    Follow,
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

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WellKnownExclude {
    RustTarget,
    NodeModules,
    PythonHiddenVenv,
    PythonVenv,
    PythonPycache,
    WebDist,
    WebBuild,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SnapshotOptions {
    pub symlink_policy: SymlinkPolicy,
    pub well_known_excludes: Vec<WellKnownExclude>,
    pub exclude_globs: Vec<String>,
    pub non_utf8_path_policy: NonUtf8PathPolicy,
    pub max_file_bytes: Option<u64>,
    pub large_file_policy: LargeFilePolicy,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            symlink_policy: SymlinkPolicy::Skip,
            well_known_excludes: Vec::new(),
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
    well_known_excludes: Vec<WellKnownExclude>,
}

impl PartialEq for CompiledIgnoreSet {
    fn eq(&self, other: &Self) -> bool {
        self.patterns == other.patterns && self.well_known_excludes == other.well_known_excludes
    }
}

impl Eq for CompiledIgnoreSet {}

impl CompiledIgnoreSet {
    pub(crate) fn compile(options: &SnapshotOptions) -> RepoResult<Self> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &options.exclude_globs {
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
            patterns: options.exclude_globs.clone(),
            matcher,
            well_known_excludes: options.well_known_excludes.clone(),
        })
    }

    pub(crate) fn is_ignored(&self, repo_path: &RepoPath, is_dir: bool) -> bool {
        if is_intrinsic_git_path(repo_path) {
            return true;
        }
        if self
            .well_known_excludes
            .iter()
            .any(|exclude| exclude.matches(repo_path, is_dir))
        {
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

impl WellKnownExclude {
    pub(crate) fn canonical_dir(self) -> &'static str {
        match self {
            WellKnownExclude::RustTarget => "target",
            WellKnownExclude::NodeModules => "node_modules",
            WellKnownExclude::PythonHiddenVenv => ".venv",
            WellKnownExclude::PythonVenv => "venv",
            WellKnownExclude::PythonPycache => "__pycache__",
            WellKnownExclude::WebDist => "dist",
            WellKnownExclude::WebBuild => "build",
        }
    }

    fn matches(self, repo_path: &RepoPath, is_dir: bool) -> bool {
        let segments = repo_path.as_str().split('/').collect::<Vec<_>>();
        let directory_segments = if is_dir {
            &segments[..]
        } else {
            &segments[..segments.len().saturating_sub(1)]
        };

        directory_segments
            .iter()
            .any(|segment| *segment == self.canonical_dir())
    }
}
