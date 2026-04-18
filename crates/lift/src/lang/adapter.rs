use serde::{Deserialize, Serialize};

use crate::kernel::{Diagnostic, FileId, Fingerprint, RepoPath};
use crate::lang::{
    LangError, LangResult, LanguageId, LocalEdgeDraft, LocalSymbolDraft, SurfaceMarkerDraft,
};

mod capabilities {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/lang/capabilities.rs"
    ));
}

pub(crate) use capabilities::AdapterCapabilities;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub(crate) struct AdapterName(String);

impl AdapterName {
    pub(crate) fn parse(input: &str) -> LangResult<Self> {
        if is_valid_adapter_name(input) {
            Ok(Self(input.to_owned()))
        } else {
            Err(LangError::InvalidAdapterName {
                input: input.to_owned(),
            })
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for AdapterName {
    type Error = LangError;

    fn try_from(value: String) -> LangResult<Self> {
        Self::parse(&value)
    }
}

impl From<AdapterName> for String {
    fn from(value: AdapterName) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct AdapterDescriptor {
    pub name: AdapterName,
    pub language: LanguageId,
    pub version: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ParseInput<'a> {
    pub path: &'a RepoPath,
    pub file_id: &'a FileId,
    pub blob_fingerprint: &'a Fingerprint,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct AdapterParseOutput {
    pub symbols: Vec<LocalSymbolDraft>,
    pub edges: Vec<LocalEdgeDraft>,
    pub surface_markers: Vec<SurfaceMarkerDraft>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum AdapterParseResult {
    Parsed(AdapterParseOutput),
    Failed { diagnostics: Vec<Diagnostic> },
}

pub(crate) trait LanguageAdapter: Send + Sync {
    fn descriptor(&self) -> AdapterDescriptor;
    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::default()
    }
    fn recognizes(&self, input: &ParseInput<'_>) -> bool;
    fn parse(&self, input: &ParseInput<'_>) -> AdapterParseResult;
}

fn is_valid_adapter_name(input: &str) -> bool {
    let mut segments = input.split('.');
    let Some(first) = segments.next() else {
        return false;
    };

    if !is_valid_head_segment(first) {
        return false;
    }

    let mut saw_tail_segment = false;
    for segment in segments {
        saw_tail_segment = true;
        if !is_valid_tail_segment(segment) {
            return false;
        }
    }

    saw_tail_segment
}

fn is_valid_head_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
}

fn is_valid_tail_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}
