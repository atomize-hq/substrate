#![allow(dead_code)]

use anyhow::{Context, Result};
use secrecy::SecretString;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Standalone Codex auth state loaded from a local auth file.
///
/// The loader is intentionally liberal about the JSON shape so it can support
/// the local Codex auth file and equivalent compatibility carriers.
#[derive(Debug, Clone)]
pub struct CodexAuthState {
    pub account_id: Option<String>,
    pub access_token: SecretString,
}

impl CodexAuthState {
    pub fn new(account_id: Option<String>, access_token: SecretString) -> Self {
        Self {
            account_id,
            access_token,
        }
    }

    pub fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        Ok(home.join(".codex").join("auth.json"))
    }

    pub fn load_default() -> Result<Self> {
        Self::load(Self::default_path()?)
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read Codex auth state from {}", path.display()))?;
        let json: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse Codex auth state at {}", path.display()))?;

        Self::from_json_value(&json)
    }

    pub fn from_json_value(value: &serde_json::Value) -> Result<Self> {
        let account_id = find_string_field(value, &["account_id"]);
        let access_token = find_string_field(value, &["access_token"])
            .context("Codex auth state is missing access_token")?;

        Ok(Self::new(account_id, SecretString::new(access_token)))
    }
}

fn find_string_field(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    match value {
        serde_json::Value::Object(map) => {
            for key in keys {
                if let Some(value) = map.get(*key).and_then(|v| v.as_str()) {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }

            for nested in map.values() {
                if let Some(found) = find_string_field(nested, keys) {
                    return Some(found);
                }
            }

            None
        }
        serde_json::Value::Array(items) => {
            for item in items {
                if let Some(found) = find_string_field(item, keys) {
                    return Some(found);
                }
            }

            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
    use tempfile::TempDir;

    #[test]
    fn load_codex_auth_state_prefers_explicit_fields() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("auth.json");
        fs::write(
            &path,
            r#"{
                "account_id": "acct_local_explicit",
                "access_token": "header.payload.signature",
                "oauth": {
                    "account_id": "acct_nested",
                    "access_token": "nested-token"
                }
            }"#,
        )
        .unwrap();

        let state = CodexAuthState::load(&path).unwrap();
        assert_eq!(state.account_id.as_deref(), Some("acct_local_explicit"));
        assert_eq!(
            state.access_token.expose_secret(),
            "header.payload.signature"
        );
    }

    #[test]
    fn load_codex_auth_state_supports_nested_equivalent_carriers() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("auth.json");
        fs::write(
            &path,
            r#"{
                "session": {
                    "account_id": "acct_nested",
                    "access_token": "nested.header.signature"
                }
            }"#,
        )
        .unwrap();

        let state = CodexAuthState::load(&path).unwrap();
        assert_eq!(state.account_id.as_deref(), Some("acct_nested"));
        assert_eq!(
            state.access_token.expose_secret(),
            "nested.header.signature"
        );
    }

    #[test]
    fn load_codex_auth_state_rejects_missing_access_token() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("auth.json");
        fs::write(
            &path,
            r#"{
                "account_id": "acct_local_explicit"
            }"#,
        )
        .unwrap();

        let err = CodexAuthState::load(&path).unwrap_err();
        assert_eq!(err.to_string(), "Codex auth state is missing access_token");
    }
}
