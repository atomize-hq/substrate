//! Fingerprint primitives.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::canonical_json::canonical_json_bytes;
use super::error::{KernelError, KernelResult};

const PREFIX: &str = "sha256:";
const DIGEST_LEN: usize = 64;

/// A content fingerprint with the wire format `sha256:<64-lower-hex>`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Fingerprint(String);

impl Fingerprint {
    /// Parses and validates a fingerprint.
    pub fn parse(input: &str) -> KernelResult<Self> {
        let digest = input
            .strip_prefix(PREFIX)
            .ok_or_else(|| KernelError::InvalidFingerprint {
                input: input.to_owned(),
            })?;

        if digest.len() != DIGEST_LEN || !digest.bytes().all(is_lower_hex) {
            return Err(KernelError::InvalidFingerprint {
                input: input.to_owned(),
            });
        }

        Ok(Self(input.to_owned()))
    }

    /// Returns the fingerprint as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Fingerprint {
    type Error = KernelError;

    fn try_from(value: String) -> KernelResult<Self> {
        Self::parse(&value)
    }
}

impl From<Fingerprint> for String {
    fn from(value: Fingerprint) -> Self {
        value.0
    }
}

/// Computes a SHA-256 fingerprint from raw bytes.
pub fn sha256_bytes(bytes: &[u8]) -> Fingerprint {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = lower_hex(&hasher.finalize());
    Fingerprint(format!("{PREFIX}{digest}"))
}

/// Computes a SHA-256 fingerprint from RFC 8785 canonical JSON bytes.
pub fn sha256_canonical_json<T: Serialize>(value: &T) -> KernelResult<Fingerprint> {
    Ok(sha256_bytes(&canonical_json_bytes(value)?))
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nibble_to_hex(byte >> 4));
        out.push(nibble_to_hex(byte & 0x0f));
    }
    out
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + (nibble - 10)) as char,
        _ => unreachable!("nibble must be in 0..=15"),
    }
}

#[cfg(test)]
mod tests {
    use super::Fingerprint;

    #[test]
    fn parses_valid_fingerprint() {
        let fingerprint = Fingerprint::parse(
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect("fingerprint");
        assert_eq!(
            fingerprint.as_str(),
            "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
    }

    #[test]
    fn rejects_invalid_fingerprint() {
        assert!(Fingerprint::parse("md5:abcd").is_err());
        assert!(serde_json::from_str::<Fingerprint>("\"sha256:nothex\"").is_err());
    }
}
