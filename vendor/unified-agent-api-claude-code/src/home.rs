use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::ClaudeCodeError;

/// Wrapper-managed "home" layout for Claude Code CLI state.
///
/// This is similar in spirit to Codex's `CODEX_HOME`: callers can point the wrapper at an
/// application-scoped directory and have the Claude CLI write config/cache/data beneath it
/// by overriding environment variables (`HOME` + `XDG_*`, and Windows equivalents).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeHomeLayout {
    root: PathBuf,
}

impl ClaudeHomeLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    pub fn xdg_config_home(&self) -> PathBuf {
        self.root.join(".config")
    }

    pub fn xdg_data_home(&self) -> PathBuf {
        self.root.join(".local").join("share")
    }

    pub fn xdg_cache_home(&self) -> PathBuf {
        self.root.join(".cache")
    }

    #[cfg(windows)]
    pub fn userprofile_dir(&self) -> PathBuf {
        self.root.clone()
    }

    #[cfg(windows)]
    pub fn appdata_dir(&self) -> PathBuf {
        self.root.join("AppData").join("Roaming")
    }

    #[cfg(windows)]
    pub fn localappdata_dir(&self) -> PathBuf {
        self.root.join("AppData").join("Local")
    }

    pub fn materialize(&self, create_dirs: bool) -> Result<(), ClaudeCodeError> {
        if !create_dirs {
            return Ok(());
        }

        for path in [
            self.root.as_path(),
            self.xdg_config_home().as_path(),
            self.xdg_data_home().as_path(),
            self.xdg_cache_home().as_path(),
        ] {
            fs::create_dir_all(path).map_err(|source| ClaudeCodeError::PrepareClaudeHome {
                path: path.to_path_buf(),
                source,
            })?;
        }

        #[cfg(windows)]
        for path in [self.appdata_dir(), self.localappdata_dir()] {
            fs::create_dir_all(&path).map_err(|source| ClaudeCodeError::PrepareClaudeHome {
                path: path.to_path_buf(),
                source,
            })?;
        }

        Ok(())
    }

    pub fn seed_from_user_home(
        &self,
        seed_home: &Path,
        level: ClaudeHomeSeedLevel,
    ) -> Result<ClaudeHomeSeedOutcome, ClaudeCodeError> {
        let mut outcome = ClaudeHomeSeedOutcome::default();

        match level {
            ClaudeHomeSeedLevel::MinimalAuth => {
                seed_minimal(seed_home, self.root(), &mut outcome)?;
            }
            ClaudeHomeSeedLevel::FullProfile => {
                seed_minimal(seed_home, self.root(), &mut outcome)?;
                seed_full_profile(seed_home, self.root(), &mut outcome)?;
            }
        }

        Ok(outcome)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaudeHomeSeedLevel {
    MinimalAuth,
    FullProfile,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClaudeHomeSeedOutcome {
    pub copied_paths: Vec<PathBuf>,
    pub skipped_paths: Vec<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaudeHomeSeedRequest {
    pub seed_user_home: PathBuf,
    pub level: ClaudeHomeSeedLevel,
}

fn seed_minimal(
    seed_home: &Path,
    target_home: &Path,
    outcome: &mut ClaudeHomeSeedOutcome,
) -> Result<(), ClaudeCodeError> {
    let mappings = [
        (
            seed_home.join(".claude.json"),
            target_home.join(".claude.json"),
        ),
        (
            seed_home.join(".claude").join("settings.json"),
            target_home.join(".claude").join("settings.json"),
        ),
        (
            seed_home.join(".claude").join("settings.local.json"),
            target_home.join(".claude").join("settings.local.json"),
        ),
    ];

    for (src, dst) in mappings {
        copy_if_exists(&src, &dst, outcome)?;
    }

    copy_dir_if_exists(
        &seed_home.join(".claude").join("plugins"),
        &target_home.join(".claude").join("plugins"),
        outcome,
    )?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn seed_full_profile(
    seed_home: &Path,
    target_home: &Path,
    outcome: &mut ClaudeHomeSeedOutcome,
) -> Result<(), ClaudeCodeError> {
    copy_dir_if_exists(
        &seed_home
            .join("Library")
            .join("Application Support")
            .join("Claude"),
        &target_home
            .join("Library")
            .join("Application Support")
            .join("Claude"),
        outcome,
    )?;
    Ok(())
}

#[cfg(windows)]
fn seed_full_profile(
    seed_home: &Path,
    target_home: &Path,
    outcome: &mut ClaudeHomeSeedOutcome,
) -> Result<(), ClaudeCodeError> {
    copy_dir_if_exists(
        &seed_home.join("AppData").join("Roaming").join("Claude"),
        &target_home.join("AppData").join("Roaming").join("Claude"),
        outcome,
    )?;
    copy_dir_if_exists(
        &seed_home.join("AppData").join("Local").join("Claude"),
        &target_home.join("AppData").join("Local").join("Claude"),
        outcome,
    )?;
    Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn seed_full_profile(
    seed_home: &Path,
    target_home: &Path,
    outcome: &mut ClaudeHomeSeedOutcome,
) -> Result<(), ClaudeCodeError> {
    copy_dir_if_exists(
        &seed_home.join(".config").join("claude"),
        &target_home.join(".config").join("claude"),
        outcome,
    )?;
    copy_dir_if_exists(
        &seed_home.join(".local").join("share").join("claude"),
        &target_home.join(".local").join("share").join("claude"),
        outcome,
    )?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", windows, all(unix, not(target_os = "macos")))))]
fn seed_full_profile(
    seed_home: &Path,
    target_home: &Path,
    outcome: &mut ClaudeHomeSeedOutcome,
) -> Result<(), ClaudeCodeError> {
    let _ = (seed_home, target_home, outcome);
    Ok(())
}

fn copy_if_exists(
    src: &Path,
    dst: &Path,
    outcome: &mut ClaudeHomeSeedOutcome,
) -> Result<(), ClaudeCodeError> {
    match fs::metadata(src) {
        Ok(meta) => {
            if !meta.is_file() {
                outcome.skipped_paths.push(src.to_path_buf());
                Ok(())
            } else {
                copy_file(src, dst)?;
                outcome.copied_paths.push(dst.to_path_buf());
                Ok(())
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            outcome.skipped_paths.push(src.to_path_buf());
            Ok(())
        }
        Err(source) => Err(ClaudeCodeError::ClaudeHomeSeedIo {
            path: src.to_path_buf(),
            source,
        }),
    }
}

fn copy_dir_if_exists(
    src: &Path,
    dst: &Path,
    outcome: &mut ClaudeHomeSeedOutcome,
) -> Result<(), ClaudeCodeError> {
    match fs::metadata(src) {
        Ok(meta) => {
            if !meta.is_dir() {
                outcome.skipped_paths.push(src.to_path_buf());
                Ok(())
            } else {
                copy_dir_recursive(src, dst)?;
                outcome.copied_paths.push(dst.to_path_buf());
                Ok(())
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            outcome.skipped_paths.push(src.to_path_buf());
            Ok(())
        }
        Err(source) => Err(ClaudeCodeError::ClaudeHomeSeedIo {
            path: src.to_path_buf(),
            source,
        }),
    }
}

fn copy_file(src: &Path, dst: &Path) -> Result<(), ClaudeCodeError> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|source| ClaudeCodeError::ClaudeHomeSeedIo {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::copy(src, dst).map_err(|source| ClaudeCodeError::ClaudeHomeSeedCopy {
        from: src.to_path_buf(),
        to: dst.to_path_buf(),
        error: source,
    })?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), ClaudeCodeError> {
    fs::create_dir_all(dst).map_err(|source| ClaudeCodeError::ClaudeHomeSeedIo {
        path: dst.to_path_buf(),
        source,
    })?;

    for entry in fs::read_dir(src).map_err(|source| ClaudeCodeError::ClaudeHomeSeedIo {
        path: src.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ClaudeCodeError::ClaudeHomeSeedIo {
            path: src.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target_path = dst.join(file_name);

        let meta =
            fs::symlink_metadata(&path).map_err(|source| ClaudeCodeError::ClaudeHomeSeedIo {
                path: path.clone(),
                source,
            })?;

        if meta.is_dir() {
            copy_dir_recursive(&path, &target_path)?;
            continue;
        }

        if meta.is_file() {
            copy_file(&path, &target_path)?;
            continue;
        }

        if meta.file_type().is_symlink() {
            // Best-effort: resolve link and copy target contents. If unreadable, skip.
            if let Ok(link_target) = fs::read_link(&path) {
                let resolved = if link_target.is_absolute() {
                    link_target
                } else {
                    path.parent()
                        .unwrap_or_else(|| Path::new("/"))
                        .join(link_target)
                };
                if let Ok(target_meta) = fs::metadata(&resolved) {
                    if target_meta.is_dir() {
                        copy_dir_recursive(&resolved, &target_path)?;
                    } else if target_meta.is_file() {
                        copy_file(&resolved, &target_path)?;
                    }
                }
            }
            continue;
        }
    }

    Ok(())
}
