//! Shared read-once auth-bundle contract for integrated gateway startup.
//!
//! ```
//! use std::collections::HashMap;
//!
//! use substrate_common::{
//!     GatewayAuthBundleV1, SUBSTRATE_LLM_AUTH_BUNDLE_FD,
//!     SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
//! };
//!
//! let mut fields = HashMap::new();
//! fields.insert(
//!     SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.to_string(),
//!     "header.payload.signature".to_string(),
//! );
//!
//! let bundle = GatewayAuthBundleV1 {
//!     schema_version: 1,
//!     backend_id: "cli:codex".to_string(),
//!     fields,
//! };
//!
//! bundle.validate().unwrap();
//! assert_eq!(SUBSTRATE_LLM_AUTH_BUNDLE_FD, "SUBSTRATE_LLM_AUTH_BUNDLE_FD");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SUBSTRATE_LLM_AUTH_BUNDLE_FD: &str = "SUBSTRATE_LLM_AUTH_BUNDLE_FD";
pub const SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID: &str =
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID";
pub const SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN: &str =
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN";
pub const SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY: &str =
    "SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY";

pub const GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION: u32 = 1;
pub const GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX: &str = "cli:codex";
pub const GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI: &str = "api:openai";

pub const CLI_CODEX_GATEWAY_AUTH_ALLOWED_FIELDS: &[&str] = &[
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
    SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN,
];
pub const CLI_CODEX_GATEWAY_AUTH_REQUIRED_FIELDS: &[&str] =
    &[SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN];
pub const API_OPENAI_GATEWAY_AUTH_ALLOWED_FIELDS: &[&str] =
    &[SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY];
pub const API_OPENAI_GATEWAY_AUTH_REQUIRED_FIELDS: &[&str] =
    &[SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GatewayAuthBundleV1 {
    #[serde(default = "gateway_auth_bundle_schema_version")]
    pub schema_version: u32,
    pub backend_id: String,
    pub fields: HashMap<String, String>,
}

impl GatewayAuthBundleV1 {
    pub fn validate(&self) -> Result<(), String> {
        validate_gateway_auth_bundle(self)
    }

    pub fn required_field_names(&self) -> Result<&'static [&'static str], String> {
        required_gateway_auth_fields(self.backend_id.as_str())
    }

    pub fn allowed_field_names(&self) -> Result<&'static [&'static str], String> {
        allowed_gateway_auth_fields(self.backend_id.as_str())
    }
}

pub fn gateway_auth_bundle_schema_version() -> u32 {
    GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION
}

pub fn validate_gateway_auth_bundle(bundle: &GatewayAuthBundleV1) -> Result<(), String> {
    if bundle.schema_version != GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported gateway auth bundle schema_version: {} (expected {})",
            bundle.schema_version, GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION
        ));
    }

    let backend_id = bundle.backend_id.trim();
    if backend_id != bundle.backend_id {
        return Err(
            "gateway auth bundle backend_id must not contain surrounding whitespace".into(),
        );
    }
    if backend_id.is_empty() {
        return Err("gateway auth bundle backend_id is required".into());
    }

    let allowed_fields = allowed_gateway_auth_fields(backend_id)?;
    let required_fields = required_gateway_auth_fields(backend_id)?;

    if bundle.fields.is_empty() {
        return Err(format!(
            "gateway auth bundle for '{}' must include at least one field",
            backend_id
        ));
    }

    for (name, value) in &bundle.fields {
        if !allowed_fields.contains(&name.as_str()) {
            return Err(format!(
                "gateway auth bundle for '{}' contains unsupported field '{}'",
                backend_id, name
            ));
        }
        if name.trim() != name || name.contains(char::is_whitespace) || name.contains('=') {
            return Err(format!(
                "gateway auth bundle for '{}' contains invalid field name '{}'",
                backend_id, name
            ));
        }
        if value.trim().is_empty() {
            return Err(format!(
                "gateway auth bundle for '{}' contains blank value for '{}'",
                backend_id, name
            ));
        }
    }

    for required_field in required_fields {
        if !bundle.fields.contains_key(*required_field) {
            return Err(format!(
                "gateway auth bundle for '{}' is missing required field '{}'",
                backend_id, required_field
            ));
        }
    }

    Ok(())
}

pub fn required_gateway_auth_fields(backend_id: &str) -> Result<&'static [&'static str], String> {
    match backend_id {
        GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX => Ok(CLI_CODEX_GATEWAY_AUTH_REQUIRED_FIELDS),
        GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI => Ok(API_OPENAI_GATEWAY_AUTH_REQUIRED_FIELDS),
        other => Err(format!(
            "unsupported gateway auth bundle backend_id '{}'",
            other
        )),
    }
}

pub fn allowed_gateway_auth_fields(backend_id: &str) -> Result<&'static [&'static str], String> {
    match backend_id {
        GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX => Ok(CLI_CODEX_GATEWAY_AUTH_ALLOWED_FIELDS),
        GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI => Ok(API_OPENAI_GATEWAY_AUTH_ALLOWED_FIELDS),
        other => Err(format!(
            "unsupported gateway auth bundle backend_id '{}'",
            other
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_codex_bundle() -> GatewayAuthBundleV1 {
        GatewayAuthBundleV1 {
            schema_version: GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION,
            backend_id: GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX.to_string(),
            fields: HashMap::from([(
                SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN.to_string(),
                "header.payload.signature".to_string(),
            )]),
        }
    }

    #[test]
    fn validate_accepts_cli_codex_bundle_with_required_field() {
        cli_codex_bundle().validate().unwrap();
    }

    #[test]
    fn validate_accepts_api_openai_bundle_with_canonical_key() {
        let bundle = GatewayAuthBundleV1 {
            schema_version: GATEWAY_AUTH_BUNDLE_SCHEMA_VERSION,
            backend_id: GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI.to_string(),
            fields: HashMap::from([(
                SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY.to_string(),
                "sk-test".to_string(),
            )]),
        };

        bundle.validate().unwrap();
    }

    #[test]
    fn validate_rejects_unknown_backend() {
        let mut bundle = cli_codex_bundle();
        bundle.backend_id = "api:anthropic".to_string();

        let err = bundle.validate().unwrap_err();
        assert!(err.contains("unsupported gateway auth bundle backend_id"));
    }

    #[test]
    fn validate_rejects_missing_required_field() {
        let mut bundle = cli_codex_bundle();
        bundle.fields.clear();

        let err = bundle.validate().unwrap_err();
        assert!(err.contains("must include at least one field"));
    }

    #[test]
    fn validate_rejects_unknown_field_for_backend() {
        let mut bundle = cli_codex_bundle();
        bundle.fields.insert(
            SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY.to_string(),
            "sk-test".to_string(),
        );

        let err = bundle.validate().unwrap_err();
        assert!(err.contains("unsupported field"));
    }

    #[test]
    fn validate_rejects_wrong_schema_version() {
        let mut bundle = cli_codex_bundle();
        bundle.schema_version = 2;

        let err = bundle.validate().unwrap_err();
        assert!(err.contains("unsupported gateway auth bundle schema_version"));
    }
}
