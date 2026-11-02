use std::{
    ffi::OsString,
    fs as std_fs,
    path::{Path, PathBuf},
};

use thiserror::Error;
use tokio::process::Command;

use crate::defaults::{default_rust_log_value, CODEX_BINARY_ENV, CODEX_HOME_ENV, RUST_LOG_ENV};
use crate::CodexError;

#[derive(Clone, Debug)]
pub(super) struct CommandEnvironment {
    binary: PathBuf,
    codex_home: Option<CodexHomeLayout>,
    create_home_dirs: bool,
}

impl CommandEnvironment {
    pub(super) fn new(
        binary: PathBuf,
        codex_home: Option<PathBuf>,
        create_home_dirs: bool,
    ) -> Self {
        Self {
            binary,
            codex_home: codex_home.map(CodexHomeLayout::new),
            create_home_dirs,
        }
    }

    pub(super) fn binary_path(&self) -> &Path {
        &self.binary
    }

    pub(super) fn codex_home_layout(&self) -> Option<CodexHomeLayout> {
        self.codex_home.clone()
    }

    pub(super) fn environment_overrides(&self) -> Result<Vec<(OsString, OsString)>, CodexError> {
        if let Some(home) = &self.codex_home {
            home.materialize(self.create_home_dirs)?;
        }

        let mut envs = Vec::new();
        envs.push((
            OsString::from(CODEX_BINARY_ENV),
            self.binary.as_os_str().to_os_string(),
        ));

        if let Some(home) = &self.codex_home {
            envs.push((
                OsString::from(CODEX_HOME_ENV),
                home.root().as_os_str().to_os_string(),
            ));
        }

        if let Some(value) = default_rust_log_value() {
            envs.push((OsString::from(RUST_LOG_ENV), OsString::from(value)));
        }

        Ok(envs)
    }

    pub(super) fn apply(&self, command: &mut Command) -> Result<(), CodexError> {
        for (key, value) in self.environment_overrides()? {
            command.env(key, value);
        }
        Ok(())
    }
}

/// Describes the on-disk layout used by the Codex CLI when `CODEX_HOME` is set.
///
/// Files are rooted next to `config.toml`, `auth.json`, `.credentials.json`, and
/// `history.jsonl`; `conversations/` holds transcript JSONL files and `logs/`
/// holds `codex-*.log` outputs. Call [`Self::materialize`] to create the
/// directories when standing up an app-scoped home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodexHomeLayout {
    root: PathBuf,
}

impl CodexHomeLayout {
    /// Creates a new layout description rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the `CODEX_HOME` root.
    pub fn root(&self) -> &Path {
        self.root.as_path()
    }

    /// Path to `config.toml` under `CODEX_HOME`.
    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }

    /// Path to `auth.json` under `CODEX_HOME`.
    pub fn auth_path(&self) -> PathBuf {
        self.root.join("auth.json")
    }

    /// Path to `.credentials.json` under `CODEX_HOME`.
    pub fn credentials_path(&self) -> PathBuf {
        self.root.join(".credentials.json")
    }

    /// Path to `history.jsonl` under `CODEX_HOME`.
    pub fn history_path(&self) -> PathBuf {
        self.root.join("history.jsonl")
    }

    /// Directory containing conversation transcripts.
    pub fn conversations_dir(&self) -> PathBuf {
        self.root.join("conversations")
    }

    /// Directory containing Codex log files.
    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    /// Creates the `CODEX_HOME` root and its known subdirectories when
    /// `create_home_dirs` is `true`. No-op when disabled.
    pub fn materialize(&self, create_home_dirs: bool) -> Result<(), CodexError> {
        if !create_home_dirs {
            return Ok(());
        }

        let conversations = self.conversations_dir();
        let logs = self.logs_dir();
        for path in [self.root(), conversations.as_path(), logs.as_path()] {
            std_fs::create_dir_all(path).map_err(|source| CodexError::PrepareCodexHome {
                path: path.to_path_buf(),
                source,
            })?;
        }
        Ok(())
    }

    /// Copies login artifacts (`auth.json` and `.credentials.json`) from a trusted seed home into
    /// this layout. History and logs are intentionally excluded.
    ///
    /// This is opt-in and leaves defaults untouched. Missing files raise errors only when marked
    /// as required in `options`; otherwise they are skipped. Target directories are created when
    /// `create_target_dirs` is `true`.
    pub fn seed_auth_from(
        &self,
        seed_home: impl AsRef<Path>,
        options: AuthSeedOptions,
    ) -> Result<AuthSeedOutcome, AuthSeedError> {
        let seed_home = seed_home.as_ref();
        let seed_meta =
            std_fs::metadata(seed_home).map_err(|source| AuthSeedError::SeedHomeUnreadable {
                seed_home: seed_home.to_path_buf(),
                source,
            })?;
        if !seed_meta.is_dir() {
            return Err(AuthSeedError::SeedHomeNotDirectory {
                seed_home: seed_home.to_path_buf(),
            });
        }

        let mut outcome = AuthSeedOutcome::default();
        let targets = [
            (
                "auth.json",
                options.require_auth,
                &mut outcome.copied_auth,
                self.auth_path(),
            ),
            (
                ".credentials.json",
                options.require_credentials,
                &mut outcome.copied_credentials,
                self.credentials_path(),
            ),
        ];

        for (name, required, copied, destination) in targets {
            let source = seed_home.join(name);
            match std_fs::metadata(&source) {
                Ok(metadata) => {
                    if !metadata.is_file() {
                        return Err(AuthSeedError::SeedFileNotFile { path: source });
                    }

                    if options.create_target_dirs {
                        if let Some(parent) = destination.parent() {
                            std_fs::create_dir_all(parent).map_err(|source_err| {
                                AuthSeedError::CreateTargetDir {
                                    path: parent.to_path_buf(),
                                    source: source_err,
                                }
                            })?;
                        }
                    }

                    std_fs::copy(&source, &destination).map_err(|error| AuthSeedError::Copy {
                        source: source.clone(),
                        destination: destination.to_path_buf(),
                        error,
                    })?;
                    *copied = true;
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    if required {
                        return Err(AuthSeedError::SeedFileMissing { path: source });
                    }
                }
                Err(err) => {
                    return Err(AuthSeedError::SeedFileUnreadable {
                        path: source,
                        source: err,
                    })
                }
            }
        }

        Ok(outcome)
    }
}

/// Options controlling how auth files are seeded from a trusted home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthSeedOptions {
    /// Whether missing `auth.json` is an error (default: false, skip when missing).
    pub require_auth: bool,
    /// Whether missing `.credentials.json` is an error (default: false, skip when missing).
    pub require_credentials: bool,
    /// Create destination directories when needed (default: true).
    pub create_target_dirs: bool,
}

impl Default for AuthSeedOptions {
    fn default() -> Self {
        Self {
            require_auth: false,
            require_credentials: false,
            create_target_dirs: true,
        }
    }
}

/// Result of seeding Codex auth files into a target home.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AuthSeedOutcome {
    /// `true` when `auth.json` was copied.
    pub copied_auth: bool,
    /// `true` when `.credentials.json` was copied.
    pub copied_credentials: bool,
}

/// Errors that may occur while seeding Codex auth files into a target home.
#[derive(Debug, Error)]
pub enum AuthSeedError {
    #[error("seed CODEX_HOME `{seed_home}` does not exist or is unreadable")]
    SeedHomeUnreadable {
        seed_home: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("seed CODEX_HOME `{seed_home}` is not a directory")]
    SeedHomeNotDirectory { seed_home: PathBuf },
    #[error("seed file `{path}` is missing")]
    SeedFileMissing { path: PathBuf },
    #[error("seed file `{path}` is not a file")]
    SeedFileNotFile { path: PathBuf },
    #[error("seed file `{path}` is unreadable")]
    SeedFileUnreadable {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to create target directory `{path}`")]
    CreateTargetDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to copy `{source}` to `{destination}`")]
    Copy {
        source: PathBuf,
        destination: PathBuf,
        #[source]
        error: std::io::Error,
    },
}
