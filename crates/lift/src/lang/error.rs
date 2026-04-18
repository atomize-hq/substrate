use thiserror::Error;

use crate::lang::LanguageId;

pub(crate) type LangResult<T> = Result<T, LangError>;

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub(crate) enum LangError {
    #[error("duplicate adapter name")]
    DuplicateAdapterName { name: String },

    #[error("duplicate language adapter registration")]
    DuplicateLanguageAdapter {
        language: LanguageId,
        existing: String,
        duplicate: String,
    },

    #[error("invalid adapter name")]
    InvalidAdapterName { input: String },

    #[error("parse cache invariant failure")]
    CacheInvariant { reason: String },

    #[error("lang schema validation failure")]
    SchemaViolation {
        schema_id: &'static str,
        reason: String,
    },
}
