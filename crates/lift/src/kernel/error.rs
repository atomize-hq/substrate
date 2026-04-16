//! Typed kernel errors.

use thiserror::Error;

/// Result type used by kernel primitives.
pub type KernelResult<T> = Result<T, KernelError>;

/// Typed failures emitted by kernel contracts.
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum KernelError {
    /// The provided logical repository path is invalid.
    #[error("invalid repo path: {reason}")]
    InvalidRepoPath {
        /// The original user-provided input.
        input: String,
        /// The validation failure reason.
        reason: String,
    },
    /// The provided JSON Pointer is invalid.
    #[error("invalid JSON pointer")]
    InvalidJsonPointer {
        /// The original user-provided input.
        input: String,
    },
    /// The provided stable identifier is invalid.
    #[error("invalid stable id")]
    InvalidStableId {
        /// The original user-provided input.
        input: String,
        /// The expected identifier kind when a typed wrapper rejects the value.
        expected_kind: Option<&'static str>,
    },
    /// The provided fingerprint is invalid.
    #[error("invalid fingerprint")]
    InvalidFingerprint {
        /// The original user-provided input.
        input: String,
    },
    /// The provided byte span is invalid.
    #[error("invalid byte span: start {start_byte} > end {end_byte}")]
    InvalidByteSpan {
        /// The requested start byte.
        start_byte: u64,
        /// The requested end byte.
        end_byte: u64,
    },
    /// Canonical JSON serialization failed.
    #[error("canonical JSON failure: {reason}")]
    CanonicalJson {
        /// Human-readable failure context.
        reason: String,
    },
    /// Schema validation failed.
    #[error("schema validation failure: {reason}")]
    SchemaViolation {
        /// Human-readable failure context.
        reason: String,
    },
}
