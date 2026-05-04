#![allow(dead_code)]

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use once_cell::sync::Lazy;
use secrecy::{ExposeSecret, SecretString};
use std::{collections::HashMap, env, path::PathBuf, sync::RwLock};

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

static INTEGRATED_CODEX_AUTH_HANDOFF: Lazy<RwLock<Option<CodexIntegratedAuthHandoff>>> =
    Lazy::new(|| RwLock::new(None));

impl CodexIntegratedAuthHandoff {
    pub fn new(account_id: Option<String>, access_token: SecretString) -> Self {
        Self {
            account_id,
            access_token,
        }
    }

    pub fn from_fields(fields: &HashMap<String, String>) -> Result<Self> {
        let access_token = fields
            .get(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                anyhow!(
                    "integrated Codex auth handoff is missing required field {}",
                    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN
                )
            })?;

        let account_id = fields
            .get(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        Ok(Self::new(account_id, SecretString::new(access_token)))
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

pub(crate) fn install_integrated_codex_auth_handoff(
    handoff: Option<CodexIntegratedAuthHandoff>,
) -> Result<()> {
    let mut guard = INTEGRATED_CODEX_AUTH_HANDOFF
        .write()
        .map_err(|_| anyhow!("integrated Codex auth handoff lock poisoned"))?;
    *guard = handoff;
    Ok(())
}

fn integrated_codex_auth_handoff() -> Result<Option<CodexIntegratedAuthHandoff>> {
    INTEGRATED_CODEX_AUTH_HANDOFF
        .read()
        .map(|guard| guard.clone())
        .map_err(|_| anyhow!("integrated Codex auth handoff lock poisoned"))
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
                let Some(handoff) = integrated_codex_auth_handoff()? else {
                    return Err(anyhow!(
                        "integrated Codex auth source is unavailable: startup-owned integrated auth handoff is missing"
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
        let _handoff_guard = InstalledHandoffGuard::set(Some(
            CodexIntegratedAuthHandoff::from_fields(&HashMap::from([
                (
                    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.to_string(),
                    "acct_env_explicit".to_string(),
                ),
                (
                    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.to_string(),
                    "header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF9lbnZfand0In19.signature".to_string(),
                ),
            ]))
            .unwrap(),
        ));

        let resolved = CodexAuthSource::Integrated.resolve().unwrap();
        assert_eq!(resolved.mode, CodexAuthMode::Integrated);
        assert_eq!(resolved.account_id, "acct_env_explicit");
        assert_eq!(resolved.account_id_source, CodexAccountIdSource::Explicit);
    }

    #[test]
    fn from_env_reads_canonical_cli_codex_field_names() {
        let _env_lock_guard = ENV_LOCK.lock().unwrap();
        let _account_id_guard = EnvGuard::set(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
            "acct_env_explicit",
        );
        let _access_token_guard = EnvGuard::set(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
            "header.eyJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF9lbnZfand0In19.signature",
        );

        let handoff = CodexIntegratedAuthHandoff::from_env().unwrap().unwrap();
        assert_eq!(handoff.account_id.as_deref(), Some("acct_env_explicit"));
        assert!(handoff.access_token.expose_secret().contains('.'));
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

    struct InstalledHandoffGuard {
        previous: Option<CodexIntegratedAuthHandoff>,
    }

    impl InstalledHandoffGuard {
        fn set(next: Option<CodexIntegratedAuthHandoff>) -> Self {
            let previous = integrated_codex_auth_handoff().unwrap();
            install_integrated_codex_auth_handoff(next).unwrap();
            Self { previous }
        }
    }

    impl Drop for InstalledHandoffGuard {
        fn drop(&mut self) {
            install_integrated_codex_auth_handoff(self.previous.take()).unwrap();
        }
    }

    #[test]
    fn integrated_source_requires_substrate_handoff() {
        let _env_lock_guard = ENV_LOCK.lock().unwrap();
        let _handoff_guard = InstalledHandoffGuard::set(None);

        let err = CodexAuthSource::Integrated.resolve().unwrap_err();
        assert!(
            err.to_string()
                .contains("startup-owned integrated auth handoff is missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn integrated_source_does_not_read_local_auth_files() {
        let _env_lock_guard = ENV_LOCK.lock().unwrap();
        let _handoff_guard = InstalledHandoffGuard::set(None);

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
                .contains("startup-owned integrated auth handoff is missing"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn integrated_handoff_from_fields_accepts_required_cli_codex_fields() {
        let handoff = CodexIntegratedAuthHandoff::from_fields(&HashMap::from([
            (
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.to_string(),
                codex_access_token("acct_jwt").expose_secret().to_string(),
            ),
            (
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.to_string(),
                "acct_explicit".to_string(),
            ),
        ]))
        .unwrap();

        assert_eq!(handoff.account_id.as_deref(), Some("acct_explicit"));
        assert!(handoff.access_token.expose_secret().contains('.'));
    }

    #[test]
    fn integrated_handoff_from_fields_rejects_missing_access_token() {
        let err = CodexIntegratedAuthHandoff::from_fields(&HashMap::from([(
            SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID.to_string(),
            "acct_explicit".to_string(),
        )]))
        .unwrap_err();

        assert!(err
            .to_string()
            .contains(SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN));
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
