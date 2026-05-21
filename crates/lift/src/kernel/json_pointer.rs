//! RFC 6901 JSON Pointer handling.

use serde::{Deserialize, Serialize};

use crate::kernel::error::{KernelError, KernelResult};

/// Canonical JSON Pointer field path.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct JsonPointer(String);

/// Canonical field-path alias used throughout the engine.
pub type FieldPath = JsonPointer;

impl JsonPointer {
    /// Parses and validates a canonical JSON Pointer.
    pub fn parse(input: &str) -> KernelResult<Self> {
        validate_pointer(input)?;
        Ok(Self(input.to_owned()))
    }

    /// Returns the root JSON Pointer.
    pub fn root() -> Self {
        Self(String::new())
    }

    /// Returns the canonical string form.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a new pointer with one escaped token appended.
    pub fn push_token(&self, token: &str) -> Self {
        let escaped = escape_token(token);
        if self.0.is_empty() {
            Self(format!("/{escaped}"))
        } else {
            Self(format!("{}/{escaped}", self.0))
        }
    }
}

impl TryFrom<String> for JsonPointer {
    type Error = KernelError;

    fn try_from(value: String) -> KernelResult<Self> {
        Self::parse(&value)
    }
}

impl From<JsonPointer> for String {
    fn from(value: JsonPointer) -> Self {
        value.0
    }
}

fn validate_pointer(input: &str) -> KernelResult<()> {
    if input.is_empty() {
        return Ok(());
    }
    if !input.starts_with('/') {
        return Err(invalid_json_pointer(input));
    }

    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '~' {
            match chars.next() {
                Some('0' | '1') => {}
                _ => return Err(invalid_json_pointer(input)),
            }
        }
    }

    Ok(())
}

fn escape_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

fn invalid_json_pointer(input: &str) -> KernelError {
    KernelError::InvalidJsonPointer {
        input: input.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::JsonPointer;
    use crate::kernel::error::KernelError;

    #[test]
    fn parses_root_and_nested_pointers() {
        assert_eq!(
            JsonPointer::parse("").expect("root should parse"),
            JsonPointer::root()
        );
        assert_eq!(
            JsonPointer::parse("/touch/crates_touched")
                .expect("nested pointer should parse")
                .as_str(),
            "/touch/crates_touched"
        );
        assert_eq!(
            JsonPointer::parse("/a//b")
                .expect("empty token pointer should parse")
                .as_str(),
            "/a//b"
        );
    }

    #[test]
    fn rejects_invalid_pointers() {
        let cases = ["field", "~0", "/~", "/~2", "/foo~bar"];

        for input in cases {
            let error = JsonPointer::parse(input).expect_err("pointer should fail");
            assert_eq!(
                error,
                KernelError::InvalidJsonPointer {
                    input: input.to_owned(),
                }
            );
        }
    }

    #[test]
    fn push_token_escapes_reserved_characters() {
        let pointer = JsonPointer::root().push_token("a~/b");
        assert_eq!(pointer.as_str(), "/a~0~1b");

        let nested = pointer.push_token("tail");
        assert_eq!(nested.as_str(), "/a~0~1b/tail");
    }

    #[test]
    fn serde_round_trip_uses_parser() {
        let parsed: JsonPointer =
            serde_json::from_str("\"/touch/crates_touched\"").expect("pointer");
        assert_eq!(parsed.as_str(), "/touch/crates_touched");
        assert!(serde_json::from_str::<JsonPointer>("\"touch.crates_touched\"").is_err());
    }
}
