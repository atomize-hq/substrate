//! Pack reference contracts.

use serde::{Deserialize, Serialize};

use crate::pack::error::{PackError, PackResult};
use crate::pack::names::PackName;

/// Logical relative file reference inside a pack source tree.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct PackFileRef(String);

impl PackFileRef {
    /// Parses and validates a file reference payload without the `file:` prefix.
    pub(crate) fn parse(input: &str) -> PackResult<Self> {
        validate_pack_file_ref(input)?;
        Ok(Self(input.to_owned()))
    }

    /// Returns the canonical string form.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PackFileRef {
    type Error = PackError;

    fn try_from(value: String) -> PackResult<Self> {
        Self::parse(&value)
    }
}

impl From<PackFileRef> for String {
    fn from(value: PackFileRef) -> Self {
        value.0
    }
}

/// Cross-pack reference.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) enum PackRef {
    Builtin(PackName),
    File(PackFileRef),
}

impl PackRef {
    /// Parses and validates a pack reference.
    pub(crate) fn parse(input: &str) -> PackResult<Self> {
        if let Some(name) = input.strip_prefix("builtin:") {
            return Ok(Self::Builtin(PackName::parse(name)?));
        }
        if let Some(path) = input.strip_prefix("file:") {
            return Ok(Self::File(PackFileRef::parse(path)?));
        }
        Err(PackError::InvalidPackRef {
            input: input.to_owned(),
        })
    }

    /// Returns the canonical string form.
    pub(crate) fn as_str(&self) -> String {
        match self {
            Self::Builtin(name) => format!("builtin:{}", name.as_str()),
            Self::File(path) => format!("file:{}", path.as_str()),
        }
    }
}

impl TryFrom<String> for PackRef {
    type Error = PackError;

    fn try_from(value: String) -> PackResult<Self> {
        Self::parse(&value)
    }
}

impl From<PackRef> for String {
    fn from(value: PackRef) -> Self {
        value.as_str()
    }
}

fn validate_pack_file_ref(input: &str) -> PackResult<()> {
    if input.is_empty()
        || input.starts_with('/')
        || input.ends_with('/')
        || input.contains('\\')
        || has_windows_drive_prefix(input)
    {
        return Err(PackError::InvalidPackRef {
            input: format!("file:{input}"),
        });
    }

    for segment in input.split('/') {
        if segment.is_empty()
            || matches!(segment, "." | "..")
            || !is_valid_pack_file_segment(segment)
        {
            return Err(PackError::InvalidPackRef {
                input: format!("file:{input}"),
            });
        }
    }

    Ok(())
}

fn has_windows_drive_prefix(input: &str) -> bool {
    let bytes = input.as_bytes();
    bytes.len() >= 3 && bytes[1] == b':' && bytes[2] == b'/'
}

fn is_valid_pack_file_segment(segment: &str) -> bool {
    let mut bytes = segment.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };

    is_pack_file_segment_start(first) && bytes.all(is_pack_file_segment_continue)
}

fn is_pack_file_segment_start(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
}

fn is_pack_file_segment_continue(byte: u8) -> bool {
    is_pack_file_segment_start(byte) || matches!(byte, b'.' | b'_' | b'-')
}

#[cfg(test)]
mod tests {
    use super::{PackFileRef, PackRef};

    #[test]
    fn parses_builtin_and_file_refs() {
        assert!(matches!(
            PackRef::parse("builtin:generic/default").expect("builtin"),
            PackRef::Builtin(_)
        ));
        for input in [
            "file:rules/security.v1.json",
            "file:_profiles/default.toml",
            "file:rules/0-security.toml",
        ] {
            assert!(matches!(
                PackRef::parse(input).expect("file"),
                PackRef::File(_)
            ));
        }
        for input in [
            "profiles/default.toml",
            "_profiles/default.toml",
            "rules/0-security.toml",
        ] {
            assert_eq!(PackFileRef::parse(input).expect("path").as_str(), input);
        }
    }

    #[test]
    fn rejects_absolute_or_traversal_refs() {
        for input in [
            "file:/rules/security.v1.json",
            "file:./rules/security.v1.json",
            "file:../rules/security.v1.json",
            "file:rules//security.v1.json",
            "file:C:/rules/security.v1.json",
        ] {
            assert!(PackRef::parse(input).is_err(), "{input} should fail");
        }
    }

    #[test]
    fn rejects_invalid_segment_shapes() {
        for input in [
            "file:-bad.toml",
            "file:.bad.toml",
            "file:rules/-bad.toml",
            "file:rules/.bad.toml",
        ] {
            assert!(PackRef::parse(input).is_err(), "{input} should fail");
        }
    }
}
