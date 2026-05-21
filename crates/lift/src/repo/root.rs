use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::repo::{RepoError, RepoResult};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) struct RootMarker(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoRootDetectionOptions {
    pub markers: BTreeSet<RootMarker>,
    pub ceiling_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepoRoot {
    absolute_path: PathBuf,
}

impl RootMarker {
    pub(crate) fn parse(input: &str) -> RepoResult<Self> {
        if input.is_empty()
            || input.contains('/')
            || input.contains('\\')
            || input == "."
            || input == ".."
        {
            return Err(RepoError::InvalidRootMarker {
                input: input.to_owned(),
            });
        }

        Ok(Self(input.to_owned()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl RepoRootDetectionOptions {
    pub(crate) fn git_default() -> Self {
        let mut markers = BTreeSet::new();
        markers.insert(RootMarker::parse(".git").expect("default root marker should parse"));
        Self {
            markers,
            ceiling_dir: None,
        }
    }
}

impl RepoRoot {
    pub(crate) fn from_dir(path: &Path) -> RepoResult<Self> {
        if !path.exists() {
            return Err(RepoError::StartPathNotFound {
                path: path.display().to_string(),
            });
        }

        let absolute_path = canonicalize_path(path, "canonicalize_repo_root")?;
        if !absolute_path.is_dir() {
            return Err(RepoError::RootNotDirectory {
                path: absolute_path.display().to_string(),
            });
        }

        Ok(Self { absolute_path })
    }

    pub(crate) fn as_path(&self) -> &Path {
        &self.absolute_path
    }

    pub(crate) fn display(&self) -> String {
        self.absolute_path.display().to_string()
    }
}

pub(crate) fn detect_repo_root(
    start_path: &Path,
    options: &RepoRootDetectionOptions,
) -> RepoResult<RepoRoot> {
    if !start_path.exists() {
        return Err(RepoError::StartPathNotFound {
            path: start_path.display().to_string(),
        });
    }

    let start_absolute = canonicalize_path(start_path, "canonicalize_start_path")?;
    let start_dir = if start_absolute.is_dir() {
        start_absolute
    } else {
        start_absolute
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| RepoError::RootNotFound {
                start_path: start_path.display().to_string(),
                markers: marker_names(options),
            })?
    };

    let ceiling_dir = options
        .ceiling_dir
        .as_deref()
        .map(|path| canonicalize_path(path, "canonicalize_ceiling_dir"))
        .transpose()?;

    let mut current = start_dir.as_path();
    loop {
        if directory_matches_any_marker(current, &options.markers) {
            return RepoRoot::from_dir(current);
        }

        if Some(current) == ceiling_dir.as_deref() {
            break;
        }

        let Some(parent) = current.parent() else {
            break;
        };
        current = parent;
    }

    Err(RepoError::RootNotFound {
        start_path: start_path.display().to_string(),
        markers: marker_names(options),
    })
}

fn canonicalize_path(path: &Path, op: &'static str) -> RepoResult<PathBuf> {
    fs::canonicalize(path).map_err(|error| RepoError::Io {
        op,
        path: path.display().to_string(),
        reason: error.to_string(),
    })
}

fn directory_matches_any_marker(directory: &Path, markers: &BTreeSet<RootMarker>) -> bool {
    markers
        .iter()
        .any(|marker| directory.join(marker.as_str()).exists())
}

fn marker_names(options: &RepoRootDetectionOptions) -> Vec<String> {
    options
        .markers
        .iter()
        .map(|marker| marker.as_str().to_owned())
        .collect()
}
