use std::{env, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverOptions {
    pub codex_home: Option<Utf8PathBuf>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredSessionArtifact {
    pub path: Utf8PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("failed to resolve Codex home: neither --codex-home, CODEX_HOME, nor HOME were available")]
    MissingCodexHome,
    #[error("resolved Codex home from {origin} was not valid UTF-8: {path}")]
    NonUtf8Path {
        origin: &'static str,
        path: PathBuf,
    },
    #[error("Codex sessions directory does not exist: {0}")]
    MissingSessionsDirectory(Utf8PathBuf),
    #[error("failed to walk Codex sessions directory {root}: {source}")]
    Walk {
        root: Utf8PathBuf,
        #[source]
        source: walkdir::Error,
    },
}

pub fn resolve_codex_home(explicit: Option<Utf8PathBuf>) -> Result<Utf8PathBuf, DiscoveryError> {
    resolve_codex_home_from(
        explicit,
        env::var_os("CODEX_HOME").map(PathBuf::from),
        env::var_os("HOME").map(PathBuf::from),
    )
}

pub fn resolve_codex_home_from(
    explicit: Option<Utf8PathBuf>,
    env_codex_home: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> Result<Utf8PathBuf, DiscoveryError> {
    if let Some(codex_home) = explicit {
        return Ok(codex_home);
    }

    if let Some(codex_home) = env_codex_home {
        return utf8_path("CODEX_HOME", codex_home);
    }

    let home_dir = home_dir.ok_or(DiscoveryError::MissingCodexHome)?;
    let home_dir = utf8_path("HOME", home_dir)?;
    Ok(home_dir.join(".codex"))
}

pub fn discover_session_artifacts(
    options: &DiscoverOptions,
) -> Result<Vec<DiscoveredSessionArtifact>, DiscoveryError> {
    let codex_home = resolve_codex_home(options.codex_home.clone())?;
    discover_session_artifacts_in_home(&codex_home, options.session_id.as_deref())
}

pub fn discover_session_artifacts_in_home(
    codex_home: &Utf8Path,
    session_id: Option<&str>,
) -> Result<Vec<DiscoveredSessionArtifact>, DiscoveryError> {
    let sessions_root = codex_home.join("sessions");
    if !sessions_root.is_dir() {
        return Err(DiscoveryError::MissingSessionsDirectory(sessions_root));
    }

    let mut artifacts = Vec::new();
    let normalized_filter = normalize_session_filter(session_id);

    for entry in WalkDir::new(&sessions_root) {
        let entry = entry.map_err(|source| DiscoveryError::Walk {
            root: sessions_root.clone(),
            source,
        })?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = utf8_path("sessions entry", entry.path().to_path_buf())?;
        if matches_session_filter(&path, normalized_filter.as_deref()) {
            artifacts.push(DiscoveredSessionArtifact { path });
        }
    }

    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(artifacts)
}

fn normalize_session_filter(session_id: Option<&str>) -> Option<String> {
    session_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.strip_prefix("urn:uuid:").unwrap_or(value).to_string())
}

fn matches_session_filter(path: &Utf8Path, session_id: Option<&str>) -> bool {
    match session_id {
        Some(session_id) => path.as_str().contains(session_id),
        None => true,
    }
}

fn utf8_path(source: &'static str, path: PathBuf) -> Result<Utf8PathBuf, DiscoveryError> {
    Utf8PathBuf::from_path_buf(path.clone()).map_err(|path| DiscoveryError::NonUtf8Path {
        origin: source,
        path,
    })
}
