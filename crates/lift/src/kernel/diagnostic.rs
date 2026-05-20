//! Machine-stable diagnostics.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::kernel::{KernelError, KernelResult, Locator};

/// Diagnostic severity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Hard validation or contract failure.
    Error,
    /// Recoverable issue that should be addressed.
    Warning,
    /// Informational note.
    Info,
}

impl Severity {
    fn rank(self) -> u8 {
        match self {
            Self::Error => 0,
            Self::Warning => 1,
            Self::Info => 2,
        }
    }
}

/// Stable machine-readable diagnostic code.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DiagnosticCode(String);

impl DiagnosticCode {
    /// Parses and validates a dot-namespaced diagnostic code.
    pub fn parse(input: &str) -> KernelResult<Self> {
        if is_valid_diagnostic_code(input) {
            Ok(Self(input.to_owned()))
        } else {
            Err(KernelError::SchemaViolation {
                reason: format!("invalid diagnostic code: {input}"),
            })
        }
    }

    /// Returns the code string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for DiagnosticCode {
    type Error = KernelError;

    fn try_from(value: String) -> KernelResult<Self> {
        Self::parse(&value)
    }
}

impl From<DiagnosticCode> for String {
    fn from(value: DiagnosticCode) -> Self {
        value.0
    }
}

/// Secondary location attached to a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct RelatedLocation {
    /// The related locator.
    pub locator: Locator,
    /// Human-readable relationship context.
    pub message: String,
}

/// Structured diagnostic record.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Diagnostic severity.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Primary subject of the diagnostic, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Locator>,
    /// Additional related locations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedLocation>,
    /// Optional fix or remediation hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl Ord for Diagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        diagnostic_sort_key(self)
            .cmp(&diagnostic_sort_key(other))
            .then_with(|| self.subject.cmp(&other.subject))
            .then_with(|| self.related.cmp(&other.related))
            .then_with(|| self.help.cmp(&other.help))
    }
}

impl PartialOrd for Diagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn is_valid_diagnostic_code(input: &str) -> bool {
    let mut segments = input.split('.');
    let Some(first) = segments.next() else {
        return false;
    };

    if !is_valid_first_segment(first) {
        return false;
    }

    let mut saw_additional_segment = false;
    for segment in segments {
        saw_additional_segment = true;
        if !is_valid_tail_segment(segment) {
            return false;
        }
    }

    saw_additional_segment
}

fn is_valid_first_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
}

fn is_valid_tail_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

type DiagnosticSortKey<'a> = (
    u8,
    Option<&'a str>,
    Option<u64>,
    Option<u64>,
    &'a str,
    &'a str,
    Option<&'a str>,
);

fn diagnostic_sort_key(diagnostic: &Diagnostic) -> DiagnosticSortKey<'_> {
    let subject_path = diagnostic
        .subject
        .as_ref()
        .map(|locator| locator.path.as_str());
    let subject_start = diagnostic
        .subject
        .as_ref()
        .and_then(|locator| locator.span.map(|span| span.start_byte));
    let subject_end = diagnostic
        .subject
        .as_ref()
        .and_then(|locator| locator.span.map(|span| span.end_byte));
    let subject_pointer = diagnostic.subject.as_ref().and_then(|locator| {
        locator
            .json_pointer
            .as_ref()
            .map(|pointer| pointer.as_str())
    });

    (
        diagnostic.severity.rank(),
        subject_path,
        subject_start,
        subject_end,
        diagnostic.code.as_str(),
        diagnostic.message.as_str(),
        subject_pointer,
    )
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticCode, RelatedLocation, Severity};
    use crate::kernel::{ByteSpan, JsonPointer, Locator, RepoPath};

    #[test]
    fn parses_valid_diagnostic_codes() {
        let code =
            DiagnosticCode::parse("kernel.repo_path.invalid_absolute").expect("code should parse");
        assert_eq!(code.as_str(), "kernel.repo_path.invalid_absolute");
    }

    #[test]
    fn rejects_invalid_diagnostic_codes() {
        let invalid = [
            "",
            "kernel",
            "Kernel.repo_path.invalid",
            "kernel.repo- path.invalid",
            "kernel._segment.invalid",
            "kernel.repo_path.Invalid",
            "kernel.repo_path.",
        ];

        for candidate in invalid {
            assert!(
                DiagnosticCode::parse(candidate).is_err(),
                "{candidate} should be rejected"
            );
        }
    }

    #[test]
    fn diagnostics_sort_by_contract_order() {
        let code =
            DiagnosticCode::parse("kernel.repo_path.invalid_absolute").expect("code should parse");
        let path = RepoPath::parse("crates/lift/src/kernel/path.rs").expect("path");

        let info = Diagnostic {
            code: code.clone(),
            severity: Severity::Info,
            message: "info".to_owned(),
            subject: None,
            related: Vec::new(),
            help: None,
        };
        let warning = Diagnostic {
            code: code.clone(),
            severity: Severity::Warning,
            message: "warning".to_owned(),
            subject: Some(Locator {
                path: path.clone(),
                span: Some(ByteSpan::new(7, 8).expect("span")),
                json_pointer: None,
            }),
            related: Vec::new(),
            help: None,
        };
        let error = Diagnostic {
            code,
            severity: Severity::Error,
            message: "error".to_owned(),
            subject: Some(Locator {
                path,
                span: Some(ByteSpan::new(1, 2).expect("span")),
                json_pointer: Some(JsonPointer::parse("/touch/crates_touched").expect("pointer")),
            }),
            related: vec![RelatedLocation {
                locator: Locator {
                    path: RepoPath::parse("README.md").expect("path"),
                    span: None,
                    json_pointer: None,
                },
                message: "context".to_owned(),
            }],
            help: Some("fix it".to_owned()),
        };

        let mut diagnostics = [info, warning, error.clone()];
        diagnostics.sort();

        assert_eq!(diagnostics[0], error);
        assert_eq!(diagnostics[1].severity, Severity::Warning);
        assert_eq!(diagnostics[2].severity, Severity::Info);
    }

    #[test]
    fn serde_round_trip_uses_validation() {
        let parsed: DiagnosticCode =
            serde_json::from_str("\"kernel.repo_path.invalid_absolute\"").expect("code");
        assert_eq!(parsed.as_str(), "kernel.repo_path.invalid_absolute");
        assert!(serde_json::from_str::<DiagnosticCode>("\"kernel\"").is_err());
    }
}
