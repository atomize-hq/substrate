//! Source descriptors for pack compilation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Wire format for a pack source.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PackFormat {
    Json,
    Toml,
}

/// Diagnostic/reporting identity for a pack source.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PackOrigin {
    Builtin { logical_name: String },
    File { display_path: String },
    Inline { logical_name: String },
}

impl PackOrigin {
    /// Returns a stable display string for errors and diagnostics.
    pub(crate) fn display(&self) -> String {
        match self {
            Self::Builtin { logical_name } => format!("builtin:{logical_name}"),
            Self::File { display_path } => display_path.clone(),
            Self::Inline { logical_name } => format!("inline:{logical_name}"),
        }
    }

    /// Returns true when the source is file-backed.
    pub(crate) fn is_file_backed(&self) -> bool {
        matches!(self, Self::File { .. })
    }
}

/// Selects bytes to feed through the pack compiler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PackSource {
    Builtin {
        logical_name: &'static str,
        format: PackFormat,
        bytes: &'static [u8],
    },
    File {
        path: PathBuf,
        format_hint: Option<PackFormat>,
    },
    Inline {
        logical_name: String,
        format: PackFormat,
        bytes: Vec<u8>,
    },
}
