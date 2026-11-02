use crate::{CliOverridesPatch, ConfigOverride, FlagState};
use std::{path::PathBuf, process::ExitStatus};

/// Target for app-server code generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppServerCodegenTarget {
    /// Emits TypeScript bindings for the app-server protocol. Optionally formats the output with Prettier.
    TypeScript { prettier: Option<PathBuf> },
    /// Emits a JSON schema bundle for the app-server protocol.
    JsonSchema,
}

impl AppServerCodegenTarget {
    pub(crate) fn subcommand(&self) -> &'static str {
        match self {
            AppServerCodegenTarget::TypeScript { .. } => "generate-ts",
            AppServerCodegenTarget::JsonSchema => "generate-json-schema",
        }
    }

    pub(crate) fn prettier(&self) -> Option<&PathBuf> {
        match self {
            AppServerCodegenTarget::TypeScript { prettier } => prettier.as_ref(),
            AppServerCodegenTarget::JsonSchema => None,
        }
    }
}

/// Request for `codex app-server generate-ts` or `generate-json-schema`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppServerCodegenRequest {
    /// Codegen target and optional Prettier path (TypeScript only).
    pub target: AppServerCodegenTarget,
    /// Output directory passed to `--out`; created if missing.
    pub out_dir: PathBuf,
    /// Passes `--experimental` to the app-server codegen subcommand when enabled.
    pub experimental: bool,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl AppServerCodegenRequest {
    /// Generates TypeScript bindings into `out_dir`.
    pub fn typescript(out_dir: impl Into<PathBuf>) -> Self {
        Self {
            target: AppServerCodegenTarget::TypeScript { prettier: None },
            out_dir: out_dir.into(),
            experimental: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Generates a JSON schema bundle into `out_dir`.
    pub fn json_schema(out_dir: impl Into<PathBuf>) -> Self {
        Self {
            target: AppServerCodegenTarget::JsonSchema,
            out_dir: out_dir.into(),
            experimental: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Controls whether `--experimental` is passed to the codegen subcommand.
    pub fn experimental(mut self, enable: bool) -> Self {
        self.experimental = enable;
        self
    }

    /// Formats TypeScript output with the provided Prettier executable (no-op for JSON schema).
    pub fn prettier(mut self, prettier: impl Into<PathBuf>) -> Self {
        if let AppServerCodegenTarget::TypeScript { prettier: slot } = &mut self.target {
            *slot = Some(prettier.into());
        }
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }

    /// Adds a `--config key=value` override for this request.
    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::new(key, value));
        self
    }

    /// Adds a raw `--config key=value` override without validation.
    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::from_raw(raw));
        self
    }

    /// Sets the config profile (`--profile`) for this request.
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.overrides.profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    /// Requests the CLI `--oss` flag for this codegen call.
    pub fn oss(mut self, enable: bool) -> Self {
        self.overrides.oss = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    /// Adds a `--enable <feature>` toggle for this codegen call.
    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.enable.push(name.into());
        self
    }

    /// Adds a `--disable <feature>` toggle for this codegen call.
    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.disable.push(name.into());
        self
    }

    /// Controls whether `--search` is passed through to Codex.
    pub fn search(mut self, enable: bool) -> Self {
        self.overrides.search = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }
}

/// Request for `codex app-server proxy`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppServerProxyRequest {
    /// Optional socket path passed via `--sock`.
    pub socket_path: Option<PathBuf>,
    /// Optional working directory override for the spawned process.
    pub working_dir: Option<PathBuf>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl AppServerProxyRequest {
    /// Creates a request with no socket override.
    pub fn new() -> Self {
        Self {
            socket_path: None,
            working_dir: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Sets the optional socket path passed via `--sock`.
    pub fn socket_path(mut self, socket_path: impl Into<PathBuf>) -> Self {
        self.socket_path = Some(socket_path.into());
        self
    }

    /// Sets the working directory used to resolve relative paths.
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }

    /// Adds a `--config key=value` override for this request.
    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::new(key, value));
        self
    }

    /// Adds a raw `--config key=value` override without validation.
    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::from_raw(raw));
        self
    }

    /// Sets the config profile (`--profile`) for this request.
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.overrides.profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    /// Requests the CLI `--oss` flag for this call.
    pub fn oss(mut self, enable: bool) -> Self {
        self.overrides.oss = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    /// Adds a `--enable <feature>` toggle for this call.
    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.enable.push(name.into());
        self
    }

    /// Adds a `--disable <feature>` toggle for this call.
    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.disable.push(name.into());
        self
    }

    /// Controls whether `--search` is passed through to Codex.
    pub fn search(mut self, enable: bool) -> Self {
        self.overrides.search = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }
}

impl Default for AppServerProxyRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex app-server`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppServerRequest {
    /// Optional address passed via `--listen`.
    pub listen: Option<String>,
    /// Optional websocket audience passed via `--ws-audience`.
    pub ws_audience: Option<String>,
    /// Optional websocket auth mode passed via `--ws-auth`.
    pub ws_auth: Option<String>,
    /// Optional websocket issuer passed via `--ws-issuer`.
    pub ws_issuer: Option<String>,
    /// Optional max clock skew seconds passed via `--ws-max-clock-skew-seconds`.
    pub ws_max_clock_skew_seconds: Option<u64>,
    /// Optional shared secret file passed via `--ws-shared-secret-file`.
    pub ws_shared_secret_file: Option<PathBuf>,
    /// Optional token file passed via `--ws-token-file`.
    pub ws_token_file: Option<PathBuf>,
    /// Optional token SHA-256 fingerprint passed via `--ws-token-sha256`.
    pub ws_token_sha256: Option<String>,
    /// Optional working directory override for the spawned process.
    pub working_dir: Option<PathBuf>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl AppServerRequest {
    /// Creates a request with no listener or websocket overrides.
    pub fn new() -> Self {
        Self {
            listen: None,
            ws_audience: None,
            ws_auth: None,
            ws_issuer: None,
            ws_max_clock_skew_seconds: None,
            ws_shared_secret_file: None,
            ws_token_file: None,
            ws_token_sha256: None,
            working_dir: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn listen(mut self, listen: impl Into<String>) -> Self {
        let listen = listen.into();
        self.listen = (!listen.trim().is_empty()).then_some(listen);
        self
    }

    pub fn ws_audience(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.ws_audience = (!value.trim().is_empty()).then_some(value);
        self
    }

    pub fn ws_auth(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.ws_auth = (!value.trim().is_empty()).then_some(value);
        self
    }

    pub fn ws_issuer(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.ws_issuer = (!value.trim().is_empty()).then_some(value);
        self
    }

    pub fn ws_max_clock_skew_seconds(mut self, value: u64) -> Self {
        self.ws_max_clock_skew_seconds = Some(value);
        self
    }

    pub fn ws_shared_secret_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.ws_shared_secret_file = Some(path.into());
        self
    }

    pub fn ws_token_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.ws_token_file = Some(path.into());
        self
    }

    pub fn ws_token_sha256(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.ws_token_sha256 = (!value.trim().is_empty()).then_some(value);
        self
    }

    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }

    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::new(key, value));
        self
    }

    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::from_raw(raw));
        self
    }

    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.overrides.profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    pub fn oss(mut self, enable: bool) -> Self {
        self.overrides.oss = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.enable.push(name.into());
        self
    }

    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.disable.push(name.into());
        self
    }

    pub fn search(mut self, enable: bool) -> Self {
        self.overrides.search = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }
}

impl Default for AppServerRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Captured output from app-server codegen commands.
#[derive(Clone, Debug)]
pub struct AppServerCodegenOutput {
    /// Exit status returned by the subcommand.
    pub status: ExitStatus,
    /// Captured stdout (mirrored to the console when `mirror_stdout` is true).
    pub stdout: String,
    /// Captured stderr (mirrored unless `quiet` is set).
    pub stderr: String,
    /// Output directory passed to `--out`.
    pub out_dir: PathBuf,
}
