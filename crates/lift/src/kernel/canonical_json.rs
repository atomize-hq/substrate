//! Canonical JSON helpers.

use serde::Serialize;

use super::error::{KernelError, KernelResult};

/// Serializes a value to an RFC 8785 canonical JSON string.
pub fn canonical_json_string<T: Serialize>(value: &T) -> KernelResult<String> {
    serde_jcs::to_string(value).map_err(|error| KernelError::CanonicalJson {
        reason: error.to_string(),
    })
}

/// Serializes a value to RFC 8785 canonical JSON bytes.
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> KernelResult<Vec<u8>> {
    serde_jcs::to_vec(value).map_err(|error| KernelError::CanonicalJson {
        reason: error.to_string(),
    })
}
