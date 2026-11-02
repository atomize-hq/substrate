use std::path::PathBuf;

use tokio::process::Command;

use crate::{
    capabilities::{guard_is_supported, log_guard_skip},
    process::{preferred_output_channel, spawn_with_retry},
    CodexClient, CodexError,
};

/// Current authentication state reported by `codex login status`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodexAuthStatus {
    /// The CLI reports an active session.
    LoggedIn(CodexAuthMethod),
    /// No credentials stored locally.
    LoggedOut,
}

/// Authentication mechanism used to sign in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodexAuthMethod {
    ChatGpt,
    ApiKey {
        masked_key: Option<String>,
    },
    /// CLI reported a logged-in state but the auth method could not be parsed (e.g., new wording).
    Unknown {
        raw: String,
    },
}

/// Result of invoking `codex logout`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CodexLogoutStatus {
    LoggedOut,
    AlreadyLoggedOut,
}

/// Helper for checking Codex auth state and triggering login flows with an app-scoped `CODEX_HOME`.
///
/// All commands run with per-process env overrides; the parent process env is never mutated.
#[derive(Clone, Debug)]
pub struct AuthSessionHelper {
    client: CodexClient,
}

impl AuthSessionHelper {
    /// Creates a helper that pins `CODEX_HOME` to `app_codex_home` for every login call.
    pub fn new(app_codex_home: impl Into<PathBuf>) -> Self {
        let client = CodexClient::builder()
            .codex_home(app_codex_home)
            .create_home_dirs(true)
            .build();
        Self { client }
    }

    /// Wraps an existing `CodexClient` (useful when you already configured the binary path).
    pub fn with_client(client: CodexClient) -> Self {
        Self { client }
    }

    /// Returns the underlying `CodexClient`.
    pub fn client(&self) -> CodexClient {
        self.client.clone()
    }

    /// Reports the current login status under the configured `CODEX_HOME`.
    pub async fn status(&self) -> Result<CodexAuthStatus, CodexError> {
        self.client.login_status().await
    }

    /// Logs in with an API key when logged out; otherwise returns the current status.
    pub async fn ensure_api_key_login(
        &self,
        api_key: impl AsRef<str>,
    ) -> Result<CodexAuthStatus, CodexError> {
        match self.status().await? {
            logged @ CodexAuthStatus::LoggedIn(_) => Ok(logged),
            CodexAuthStatus::LoggedOut => self.client.login_with_api_key(api_key).await,
        }
    }

    /// Starts the ChatGPT OAuth login flow when no credentials are present.
    ///
    /// Returns `Ok(None)` when already logged in; otherwise returns the spawned login child so the
    /// caller can surface output/URLs. Dropping the child kills the login helper.
    pub async fn ensure_chatgpt_login(&self) -> Result<Option<tokio::process::Child>, CodexError> {
        match self.status().await? {
            CodexAuthStatus::LoggedIn(_) => Ok(None),
            CodexAuthStatus::LoggedOut => self.client.spawn_login_process().map(Some),
        }
    }

    /// Directly spawns the ChatGPT login process.
    pub fn spawn_chatgpt_login(&self) -> Result<tokio::process::Child, CodexError> {
        self.client.spawn_login_process()
    }

    /// Directly logs in with an API key without checking prior state.
    pub async fn login_with_api_key(
        &self,
        api_key: impl AsRef<str>,
    ) -> Result<CodexAuthStatus, CodexError> {
        self.client.login_with_api_key(api_key).await
    }
}

impl CodexClient {
    /// Spawns a `codex login` session using the default ChatGPT OAuth flow.
    ///
    /// The returned child inherits `kill_on_drop` so abandoning the handle cleans up the login helper.
    pub fn spawn_login_process(&self) -> Result<tokio::process::Child, CodexError> {
        let mut command = Command::new(self.command_env.binary_path());
        command
            .arg("login")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        self.command_env.apply(&mut command)?;

        spawn_with_retry(&mut command, self.command_env.binary_path())
    }

    /// Spawns a `codex login --device-auth` session.
    ///
    /// The returned child inherits `kill_on_drop` so abandoning the handle cleans up the login helper.
    pub fn spawn_device_auth_login_process(&self) -> Result<tokio::process::Child, CodexError> {
        let mut command = Command::new(self.command_env.binary_path());
        command
            .arg("login")
            .arg("--device-auth")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        self.command_env.apply(&mut command)?;

        spawn_with_retry(&mut command, self.command_env.binary_path())
    }

    /// Spawns a `codex login --with-api-key` session (interactive API-key flow).
    ///
    /// The returned child inherits `kill_on_drop` so abandoning the handle cleans up the login helper.
    pub fn spawn_with_api_key_login_process(&self) -> Result<tokio::process::Child, CodexError> {
        let mut command = Command::new(self.command_env.binary_path());
        command
            .arg("login")
            .arg("--with-api-key")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        self.command_env.apply(&mut command)?;

        spawn_with_retry(&mut command, self.command_env.binary_path())
    }

    /// Spawns a `codex login --with-access-token` session.
    ///
    /// The returned child inherits `kill_on_drop` so abandoning the handle cleans up the login helper.
    pub fn spawn_with_access_token_login_process(
        &self,
    ) -> Result<tokio::process::Child, CodexError> {
        let mut command = Command::new(self.command_env.binary_path());
        command
            .arg("login")
            .arg("--with-access-token")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        self.command_env.apply(&mut command)?;

        spawn_with_retry(&mut command, self.command_env.binary_path())
    }

    /// Spawns `codex login --mcp` when the probed binary advertises support.
    ///
    /// Returns `Ok(None)` when the capability is unknown or unsupported so
    /// callers can degrade gracefully without attempting the flag.
    pub async fn spawn_mcp_login_process(
        &self,
    ) -> Result<Option<tokio::process::Child>, CodexError> {
        let capabilities = self.probe_capabilities().await;
        let guard = capabilities.guard_mcp_login();
        if !guard_is_supported(&guard) {
            log_guard_skip(&guard);
            return Ok(None);
        }

        let mut command = Command::new(self.command_env.binary_path());
        command
            .arg("login")
            .arg("--mcp")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        self.command_env.apply(&mut command)?;

        let child = spawn_with_retry(&mut command, self.command_env.binary_path())?;

        Ok(Some(child))
    }

    /// Logs in with a provided API key by invoking `codex login --api-key <key>`.
    pub async fn login_with_api_key(
        &self,
        api_key: impl AsRef<str>,
    ) -> Result<CodexAuthStatus, CodexError> {
        let api_key = api_key.as_ref().trim();
        if api_key.is_empty() {
            return Err(CodexError::EmptyApiKey);
        }

        let output = self
            .run_basic_command(["login", "--api-key", api_key])
            .await?;
        let combined = preferred_output_channel(&output);

        if output.status.success() {
            Ok(parse_login_success(&combined).unwrap_or_else(|| {
                CodexAuthStatus::LoggedIn(CodexAuthMethod::Unknown {
                    raw: combined.clone(),
                })
            }))
        } else {
            Err(CodexError::NonZeroExit {
                status: output.status,
                stderr: combined,
            })
        }
    }

    /// Returns the current Codex authentication state by invoking `codex login status`.
    pub async fn login_status(&self) -> Result<CodexAuthStatus, CodexError> {
        let output = self.run_basic_command(["login", "status"]).await?;
        let combined = preferred_output_channel(&output);

        if output.status.success() {
            Ok(parse_login_success(&combined).unwrap_or_else(|| {
                CodexAuthStatus::LoggedIn(CodexAuthMethod::Unknown {
                    raw: combined.clone(),
                })
            }))
        } else if combined.to_lowercase().contains("not logged in") {
            Ok(CodexAuthStatus::LoggedOut)
        } else {
            Err(CodexError::NonZeroExit {
                status: output.status,
                stderr: combined,
            })
        }
    }

    /// Removes cached credentials via `codex logout`.
    pub async fn logout(&self) -> Result<CodexLogoutStatus, CodexError> {
        let output = self.run_basic_command(["logout"]).await?;
        let combined = preferred_output_channel(&output);

        if !output.status.success() {
            return Err(CodexError::NonZeroExit {
                status: output.status,
                stderr: combined,
            });
        }

        let normalized = combined.to_lowercase();
        if normalized.contains("successfully logged out") {
            Ok(CodexLogoutStatus::LoggedOut)
        } else if normalized.contains("not logged in") {
            Ok(CodexLogoutStatus::AlreadyLoggedOut)
        } else {
            Ok(CodexLogoutStatus::LoggedOut)
        }
    }
}

pub(crate) fn parse_login_success(output: &str) -> Option<CodexAuthStatus> {
    let lower = output.to_lowercase();
    if lower.contains("chatgpt") {
        return Some(CodexAuthStatus::LoggedIn(CodexAuthMethod::ChatGpt));
    }
    if lower.contains("api key") || lower.contains("apikey") {
        // Prefer everything after the first " - " so we do not chop the key itself.
        let masked = output
            .split_once(" - ")
            .map(|(_, value)| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .or_else(|| output.split_whitespace().last().map(|v| v.to_string()));
        return Some(CodexAuthStatus::LoggedIn(CodexAuthMethod::ApiKey {
            masked_key: masked,
        }));
    }
    None
}
