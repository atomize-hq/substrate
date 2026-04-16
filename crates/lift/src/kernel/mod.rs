//! Kernel contracts shared across the lift engine.
//!
//! This module owns deterministic, portable primitives that remain valid even
//! if higher-level apps disappear. Later seams can depend on these contracts,
//! but the kernel does not depend on them.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod canonical_json;
pub mod diagnostic;
pub mod error;
pub mod fingerprint;
pub mod id;
pub mod json_pointer;
pub mod locator;
pub mod path;
pub mod schema;
pub mod span;

pub use canonical_json::{canonical_json_bytes, canonical_json_string};
pub use diagnostic::{Diagnostic, DiagnosticCode, RelatedLocation, Severity};
pub use error::{KernelError, KernelResult};
pub use fingerprint::{sha256_bytes, sha256_canonical_json, Fingerprint};
pub use id::{
    BoundaryId, ComponentId, EvidenceId, FactId, FileId, MatchId, QueryId, RecipeId, RuleId,
    StableId, SymbolId,
};
pub use json_pointer::{FieldPath, JsonPointer};
pub use locator::Locator;
pub use path::RepoPath;
pub use schema::{
    PRIMITIVES_V1_SCHEMA_FILE, PRIMITIVES_V1_SCHEMA_ID, PRIMITIVES_V1_SCHEMA_JSON,
    PRIMITIVES_V1_SCHEMA_VERSION,
};
pub use span::ByteSpan;

/// Byte offset within a file-like payload.
pub type ByteOffset = u64;
