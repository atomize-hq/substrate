//! Source and document locators.

use serde::{Deserialize, Serialize};

use crate::kernel::{ByteSpan, JsonPointer, RepoPath};

/// A deterministic location inside a repository file or structured document.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Locator {
    /// The repo-relative logical path.
    pub path: RepoPath,
    /// Optional byte range within the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<ByteSpan>,
    /// Optional JSON Pointer into a structured payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_pointer: Option<JsonPointer>,
}

#[cfg(test)]
mod tests {
    use super::Locator;
    use crate::kernel::{ByteSpan, JsonPointer, RepoPath};

    #[test]
    fn locator_is_ordered_by_path_then_span_then_pointer() {
        let path = RepoPath::parse("crates/lift/src/kernel/path.rs").expect("path should parse");
        let earlier = Locator {
            path: path.clone(),
            span: Some(ByteSpan::new(0, 4).expect("span should parse")),
            json_pointer: None,
        };
        let later = Locator {
            path,
            span: Some(ByteSpan::new(4, 8).expect("span should parse")),
            json_pointer: Some(JsonPointer::parse("/touch/crates_touched").expect("pointer")),
        };

        assert!(earlier < later);
    }
}
