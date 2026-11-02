use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use crate::client::ClaudeClient;
use crate::home::{ClaudeHomeLayout, ClaudeHomeSeedLevel, ClaudeHomeSeedRequest};

#[derive(Debug, Clone)]
pub struct ClaudeClientBuilder {
    pub(crate) binary: Option<PathBuf>,
    pub(crate) working_dir: Option<PathBuf>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) claude_home: Option<PathBuf>,
    pub(crate) create_home_dirs: bool,
    pub(crate) home_seed: Option<ClaudeHomeSeedRequest>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) mirror_stdout: bool,
    pub(crate) mirror_stderr: bool,
}

impl Default for ClaudeClientBuilder {
    fn default() -> Self {
        Self {
            binary: None,
            working_dir: None,
            env: BTreeMap::new(),
            claude_home: None,
            create_home_dirs: true,
            home_seed: None,
            timeout: Some(Duration::from_secs(120)),
            mirror_stdout: false,
            mirror_stderr: false,
        }
    }
}

impl ClaudeClientBuilder {
    pub fn binary(mut self, binary: impl Into<PathBuf>) -> Self {
        self.binary = Some(binary.into());
        self
    }

    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Sets an application-scoped Claude home directory (wrapper-managed).
    ///
    /// When set, the wrapper injects environment variables (`HOME`, `XDG_*`, and Windows
    /// equivalents) so the real `claude` CLI writes state beneath this directory.
    ///
    /// If you do not call this, the wrapper will also honor `CLAUDE_HOME` when present.
    pub fn claude_home(mut self, home: impl Into<PathBuf>) -> Self {
        self.claude_home = Some(home.into());
        self
    }

    /// Controls whether the wrapper should create the Claude home directory tree when
    /// [`Self::claude_home`] (or `CLAUDE_HOME`) is set.
    pub fn create_home_dirs(mut self, enable: bool) -> Self {
        self.create_home_dirs = enable;
        self
    }

    /// Opt-in seeding of an isolated Claude home from an existing user profile.
    ///
    /// This is best-effort for missing sources; copy failures are surfaced when running
    /// commands (the wrapper will return an error before spawning the real binary).
    pub fn seed_profile_from(
        mut self,
        seed_user_home: impl Into<PathBuf>,
        level: ClaudeHomeSeedLevel,
    ) -> Self {
        self.home_seed = Some(ClaudeHomeSeedRequest {
            seed_user_home: seed_user_home.into(),
            level,
        });
        self
    }

    /// Convenience helper that seeds from the current user's home directory (best-effort).
    ///
    /// If the home directory cannot be inferred from environment variables, this is a no-op.
    pub fn seed_profile_from_current_user_home(mut self, level: ClaudeHomeSeedLevel) -> Self {
        let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
        if let Some(home) = home {
            self.home_seed = Some(ClaudeHomeSeedRequest {
                seed_user_home: PathBuf::from(home),
                level,
            });
        }
        self
    }

    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn mirror_stdout(mut self, enabled: bool) -> Self {
        self.mirror_stdout = enabled;
        self
    }

    pub fn mirror_stderr(mut self, enabled: bool) -> Self {
        self.mirror_stderr = enabled;
        self
    }

    pub fn build(mut self) -> ClaudeClient {
        // Avoid any updater side effects by default; callers may override explicitly.
        self.env
            .entry("DISABLE_AUTOUPDATER".to_string())
            .or_insert_with(|| "1".to_string());

        let claude_home_path = self
            .claude_home
            .take()
            .or_else(|| std::env::var_os("CLAUDE_HOME").map(PathBuf::from));
        let claude_home = claude_home_path.map(ClaudeHomeLayout::new);

        if let Some(layout) = claude_home.as_ref() {
            let root = layout.root().to_string_lossy().to_string();
            self.env
                .entry("CLAUDE_HOME".to_string())
                .or_insert(root.clone());
            self.env.entry("HOME".to_string()).or_insert(root.clone());
            self.env
                .entry("XDG_CONFIG_HOME".to_string())
                .or_insert(layout.xdg_config_home().to_string_lossy().to_string());
            self.env
                .entry("XDG_DATA_HOME".to_string())
                .or_insert(layout.xdg_data_home().to_string_lossy().to_string());
            self.env
                .entry("XDG_CACHE_HOME".to_string())
                .or_insert(layout.xdg_cache_home().to_string_lossy().to_string());

            #[cfg(windows)]
            {
                self.env
                    .entry("USERPROFILE".to_string())
                    .or_insert(root.clone());
                self.env
                    .entry("APPDATA".to_string())
                    .or_insert(layout.appdata_dir().to_string_lossy().to_string());
                self.env
                    .entry("LOCALAPPDATA".to_string())
                    .or_insert(layout.localappdata_dir().to_string_lossy().to_string());
            }
        }

        let home_materialize_status = std::sync::Arc::new(std::sync::OnceLock::new());
        let home_seed_status = std::sync::Arc::new(std::sync::OnceLock::new());

        if let Some(layout) = claude_home.as_ref() {
            let res = layout
                .materialize(self.create_home_dirs)
                .map_err(|e| e.to_string());
            let _ = home_materialize_status.set(res);

            if let Some(seed_req) = self.home_seed.as_ref() {
                let res = layout
                    .seed_from_user_home(&seed_req.seed_user_home, seed_req.level)
                    .map(|_| ())
                    .map_err(|e| e.to_string());
                let _ = home_seed_status.set(res);
            }
        }

        ClaudeClient {
            binary: self.binary,
            working_dir: self.working_dir,
            env: self.env,
            claude_home,
            create_home_dirs: self.create_home_dirs,
            home_seed: self.home_seed,
            home_materialize_status,
            home_seed_status,
            timeout: self.timeout,
            mirror_stdout: self.mirror_stdout,
            mirror_stderr: self.mirror_stderr,
        }
    }
}
