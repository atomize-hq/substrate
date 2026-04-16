#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use secrecy::{ExposeSecret, SecretString};
use std::{env, path::PathBuf};

use super::codex_auth_state::CodexAuthState;

pub const SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID: &str =
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID";
pub const SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN: &str =
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexAuthMode {
    Integrated,
    Standalone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexAccountIdSource {
    Explicit,
    JwtFallback,
}

#[derive(Debug, Clone)]
pub struct ResolvedCodexAuthContext {
    pub mode: CodexAuthMode,
    pub account_id: String,
    pub account_id_source: CodexAccountIdSource,
    pub access_token: SecretString,
}

#[derive(Debug, Clone)]
pub struct CodexIntegratedAuthHandoff {
    pub account_id: Option<String>,
    pub access_token: SecretString,
}

impl CodexIntegratedAuthHandoff {
    pub fn new(account_id: Option<String>, access_token: SecretString) -> Self {
        Self {
            account_id,
            access_token,
        }
    }

    pub fn from_env() -> Result<Option<Self>> {
        let access_token = read_env_trimmed(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN)?;
        let Some(access_token) = access_token else {
            let account_id = read_env_trimmed(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID)?;
            if account_id.is_some() {
                return Err(anyhow!(
                    "integrated Codex auth handoff is incomplete: {} is set without {}",
                    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
                    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN
                ));
            }

            return Ok(None);
        };

        let account_id = read_env_trimmed(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID)?;
        Ok(Some(Self::new(account_id, SecretString::new(access_token))))
    }
}

#[derive(Debug, Clone)]
pub enum CodexAuthSource {
    Integrated,
    StandaloneLocal { path: PathBuf },
}

impl CodexAuthSource {
    pub fn mode(&self) -> CodexAuthMode {
        match self {
            Self::Integrated => CodexAuthMode::Integrated,
            Self::StandaloneLocal { .. } => CodexAuthMode::Standalone,
        }
    }

    pub fn resolve(&self) -> Result<ResolvedCodexAuthContext> {
        match self {
            Self::Integrated => {
                let Some(handoff) = CodexIntegratedAuthHandoff::from_env()? else {
                    return Err(anyhow!(
                        "integrated Codex auth source is unavailable: Substrate-delivered auth handoff is missing"
                    ));
                };

                resolve_selected_mode(
                    CodexAuthMode::Integrated,
                    handoff.account_id,
                    handoff.access_token,
                )
            }
            Self::StandaloneLocal { path } => {
                let state = CodexAuthState::load(path).with_context(|| {
                    format!(
                        "failed to load standalone Codex auth state from {}",
                        path.display()
                    )
                })?;

                resolve_selected_mode(
                    CodexAuthMode::Standalone,
                    state.account_id,
                    state.access_token,
                )
            }
        }
    }
}

fn resolve_selected_mode(
    mode: CodexAuthMode,
    explicit_account_id: Option<String>,
    access_token: SecretString,
) -> Result<ResolvedCodexAuthContext> {
    if let Some(account_id) = explicit_account_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(ResolvedCodexAuthContext {
            mode,
            account_id: account_id.to_string(),
            account_id_source: CodexAccountIdSource::Explicit,
            access_token,
        });
    }

    let jwt_account_id = extract_account_id(access_token.expose_secret())
        .context("Codex auth context could not resolve account_id from JWT fallback")?;

    Ok(ResolvedCodexAuthContext {
        mode,
        account_id: jwt_account_id,
        account_id_source: CodexAccountIdSource::JwtFallback,
        access_token,
    })
}

fn extract_account_id(access_token: &str) -> Option<String> {
    let parts: Vec<&str> = access_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let decoded = URL_SAFE_NO_PAD.decode(parts[1]).ok()?;
    let json_str = String::from_utf8(decoded).ok()?;
    let json: serde_json::Value = serde_json::from_str(&json_str).ok()?;

    json.get("https://api.openai.com/auth")?
        .get("chatgpt_account_id")?
        .as_str()
        .map(|s| s.to_string())
}

fn read_env_trimmed(key: &str) -> Result<Option<String>> {
    match env::var(key) {
        Ok(value) => {
            let value = value.trim().to_string();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(err).with_context(|| format!("Failed to read {}", key)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use secrecy::ExposeSecret;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_LOCK: once_cell::sync::Lazy<Mutex<()>> =
        once_cell::sync::Lazy::new(|| Mutex::new(()));

    fn codex_access_token(account_id: &str) -> SecretString {
        let payload = serde_json::json!({
            "https://api.openai.com/auth": {
                "chatgpt_account_id": account_id
            }
        });
        let encoded_payload = URL_SAFE_NO_PAD.encode(payload.to_string());
        SecretString::new(format!("header.{}.signature", encoded_payload))
    }

    #[test]
    fn integrated_mode_uses_substrate_account_id_first() {
        let resolved = resolve_selected_mode(
            CodexAuthMode::Integrated,
            Some("acct_explicit".to_string()),
            codex_access_token("acct_jwt"),
        )
        .unwrap();

        assert_eq!(resolved.mode, CodexAuthMode::Integrated);
        assert_eq!(resolved.account_id, "acct_explicit");
        assert_eq!(resolved.account_id_source, CodexAccountIdSource::Explicit);
        assert_eq!(
            resolved.access_token.expose_secret(),
            codex_access_token("acct_jwt").expose_secret()
        );
    }

    #[test]
    fn integrated_mode_uses_jwt_fallback_when_explicit_account_id_is_absent() {
        let resolved = resolve_selected_mode(
            CodexAuthMode::Integrated,
            None,
            codex_access_token("acct_jwt"),
        )
        .unwrap();

        assert_eq!(resolved.mode, CodexAuthMode::Integrated);
        assert_eq!(resolved.account_id, "acct_jwt");
        assert_eq!(
            resolved.account_id_source,
            CodexAccountIdSource::JwtFallback
        );
    }

    #[test]
    fn standalone_mode_uses_explicit_account_id_first() {
        let resolved = resolve_selected_mode(
            CodexAuthMode::Standalone,
            Some("acct_local_explicit".to_string()),
            codex_access_token("acct_local_jwt"),
        )
        .unwrap();

        assert_eq!(resolved.mode, CodexAuthMode::Standalone);
        assert_eq!(resolved.account_id, "acct_local_explicit");
        assert_eq!(resolved.account_id_source, CodexAccountIdSource::Explicit);
    }

    #[test]
    fn standalone_mode_uses_jwt_fallback_when_explicit_account_id_is_absent() {
        let resolved = resolve_selected_mode(
            CodexAuthMode::Standalone,
            None,
            codex_access_token("acct_local_jwt"),
        )
        .unwrap();

        assert_eq!(resolved.mode, CodexAuthMode::Standalone);
        assert_eq!(resolved.account_id, "acct_local_jwt");
        assert_eq!(
            resolved.account_id_source,
            CodexAccountIdSource::JwtFallback
        );
    }

    #[test]
    fn auth_context_resolution_fails_when_account_id_is_unresolvable() {
        let err = resolve_selected_mode(
            CodexAuthMode::Standalone,
            None,
            SecretString::new("not-a-jwt".to_string()),
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("could not resolve account_id from JWT fallback"));
    }

    #[test]
    fn integrated_source_uses_canonical_field_names() {
        let _env_lock_guard = ENV_LOCK.lock().unwrap();

        let _account_id_guard = EnvGuard::set(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "acct_env_explicit",
        );
        let _access_token_guard = EnvGuard::set(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            "header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF9lbnZfand0In19.signature",
        );

        let resolved = CodexAuthSource::Integrated.resolve().unwrap();
        assert_eq!(resolved.mode, CodexAuthMode::Integrated);
        assert_eq!(resolved.account_id, "acct_env_explicit");
        assert_eq!(resolved.account_id_source, CodexAccountIdSource::Explicit);
    }

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var(key).ok();
            env::set_var(key, value);
            Self { key, previous }
        }

        fn clear(key: &'static str) -> Self {
            let previous = env::var(key).ok();
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                env::set_var(self.key, previous);
            } else {
                env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn integrated_source_requires_substrate_handoff() {
        let _env_lock_guard = ENV_LOCK.lock().unwrap();

        let _account_id_guard = EnvGuard::clear(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID);
        let _access_token_guard =
            EnvGuard::clear(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);

        let err = CodexAuthSource::Integrated.resolve().unwrap_err();
        assert!(
            err.to_string()
                .contains("Substrate-delivered auth handoff is missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn integrated_source_does_not_read_local_auth_files() {
        let _env_lock_guard = ENV_LOCK.lock().unwrap();

        let _account_id_guard = EnvGuard::clear(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID);
        let _access_token_guard =
            EnvGuard::clear(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN);

        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("auth.json");
        fs::write(
            &path,
            r#"{
                "account_id": "acct_local",
                "access_token": "header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF9sb2NhbF9qd3QifX0.signature"
            }"#,
        )
        .unwrap();

        let err = CodexAuthSource::Integrated.resolve().unwrap_err();
        assert!(
            err.to_string()
                .contains("Substrate-delivered auth handoff is missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn standalone_local_source_uses_explicit_path() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("auth.json");
        fs::write(
            &path,
            r#"{
                "account_id": "acct_local",
                "access_token": "header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF9sb2NhbF9qd3QifX0.signature"
            }"#,
        )
        .unwrap();

        let resolved = CodexAuthSource::StandaloneLocal { path }.resolve().unwrap();
        assert_eq!(resolved.mode, CodexAuthMode::Standalone);
        assert_eq!(resolved.account_id, "acct_local");
        assert_eq!(resolved.account_id_source, CodexAccountIdSource::Explicit);
    }
}
