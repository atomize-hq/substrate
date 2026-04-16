//! Pack-local name contracts.

use serde::{Deserialize, Serialize};

use crate::pack::error::{PackError, PackResult};

/// Canonical pack identifier.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct PackName(String);

impl PackName {
    /// Parses and validates a pack name.
    pub(crate) fn parse(input: &str) -> PackResult<Self> {
        if is_valid_pack_name(input) {
            Ok(Self(input.to_owned()))
        } else {
            Err(PackError::InvalidPackName {
                input: input.to_owned(),
            })
        }
    }

    /// Returns the canonical string form.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PackName {
    type Error = PackError;

    fn try_from(value: String) -> PackResult<Self> {
        Self::parse(&value)
    }
}

impl From<PackName> for String {
    fn from(value: PackName) -> Self {
        value.0
    }
}

/// App-level entrypoint name used by profiles.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct AppName(String);

impl AppName {
    /// Parses and validates an app name.
    pub(crate) fn parse(input: &str) -> PackResult<Self> {
        if is_valid_label(input) {
            Ok(Self(input.to_owned()))
        } else {
            Err(PackError::SchemaViolation {
                origin: input.to_owned(),
                schema_id: "pack.app_name",
                diagnostics: Vec::new(),
            })
        }
    }

    /// Returns the app name string.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AppName {
    type Error = PackError;

    fn try_from(value: String) -> PackResult<Self> {
        Self::parse(&value)
    }
}

impl From<AppName> for String {
    fn from(value: AppName) -> Self {
        value.0
    }
}

/// Supported analysis language identifiers for profile defaults.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LanguageId {
    Json,
    Toml,
    Yaml,
    #[default]
    Rust,
    Python,
    Javascript,
    Typescript,
}

impl LanguageId {
    /// Parses a language identifier from its canonical string form.
    pub(crate) fn parse(input: &str) -> PackResult<Self> {
        match input {
            "json" => Ok(Self::Json),
            "toml" => Ok(Self::Toml),
            "yaml" => Ok(Self::Yaml),
            "rust" => Ok(Self::Rust),
            "python" => Ok(Self::Python),
            "javascript" => Ok(Self::Javascript),
            "typescript" => Ok(Self::Typescript),
            _ => Err(PackError::SchemaViolation {
                origin: input.to_owned(),
                schema_id: "pack.language_id",
                diagnostics: Vec::new(),
            }),
        }
    }

    /// Returns the canonical string form.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Javascript => "javascript",
            Self::Typescript => "typescript",
        }
    }
}

fn is_valid_pack_name(input: &str) -> bool {
    let mut saw_segment = false;
    for segment in input.split('/') {
        if segment.is_empty() || !is_valid_segment(segment) {
            return false;
        }
        saw_segment = true;
    }
    saw_segment
        && !input.starts_with('/')
        && !input.ends_with('/')
        && input.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'/' | b'.' | b'_' | b'-')
        })
}

fn is_valid_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-')
        })
        && !segment.ends_with(['.', '_', '-'])
        && !segment.contains("..")
        && !segment.contains("__")
        && !segment.contains("--")
        && !segment.contains("._")
        && !segment.contains(".-")
}

fn is_valid_label(input: &str) -> bool {
    let mut chars = input.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::{AppName, LanguageId, PackName};

    #[test]
    fn pack_name_accepts_valid_values() {
        for name in [
            "generic",
            "generic/default",
            "substrate/default",
            "rust/core",
        ] {
            let parsed = PackName::parse(name).expect("pack name");
            assert_eq!(parsed.as_str(), name);
        }
    }

    #[test]
    fn pack_name_rejects_invalid_values() {
        for name in [
            "Generic",
            "/generic",
            "generic/",
            "generic//default",
            "generic default",
        ] {
            assert!(PackName::parse(name).is_err(), "{name} should fail");
        }
    }

    #[test]
    fn app_name_and_language_id_parse() {
        assert_eq!(AppName::parse("score").expect("app").as_str(), "score");
        assert_eq!(
            LanguageId::parse("typescript").expect("lang").as_str(),
            "typescript"
        );
    }
}
