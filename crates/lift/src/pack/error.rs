//! Typed pack compiler errors.

use thiserror::Error;

use crate::kernel::JsonPointer;
use crate::pack::diagnostics::PackDiagnostic;
use crate::pack::raw::PackKind;

/// Result type used by pack compiler contracts.
pub(crate) type PackResult<T> = Result<T, PackError>;

/// Typed failures emitted by the pack compiler seam.
#[allow(dead_code)]
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub(crate) enum PackError {
    #[error("pack I/O failure: {reason}")]
    Io { origin: String, reason: String },

    #[error("unsupported pack format")]
    UnsupportedFormat { origin: String },

    #[error("pack parse failure")]
    ParseFailure {
        origin: String,
        diagnostics: Vec<PackDiagnostic>,
    },

    #[error("schema validation failure")]
    SchemaViolation {
        origin: String,
        schema_id: &'static str,
        diagnostics: Vec<PackDiagnostic>,
    },

    #[error("invalid pack name")]
    InvalidPackName { input: String },

    #[error("invalid pack reference")]
    InvalidPackRef { input: String },

    #[error("duplicate pack id")]
    DuplicatePackId { kind: PackKind, id: String },

    #[error("duplicate entry id")]
    DuplicateEntryId {
        pack_kind: PackKind,
        pack_id: String,
        entry_kind: &'static str,
        entry_id: String,
    },

    #[error("unknown pack reference")]
    UnknownPackReference {
        referring_pack: String,
        reference: String,
    },

    #[error("pack reference kind mismatch")]
    RefKindMismatch {
        reference: String,
        expected: PackKind,
        actual: PackKind,
    },

    #[error("glob compile failure")]
    GlobCompile { pattern: String, reason: String },

    #[error("expression compile failure")]
    ExpressionCompile {
        pack_id: String,
        path: JsonPointer,
        reason: String,
    },
}
