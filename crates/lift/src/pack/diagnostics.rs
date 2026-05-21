//! Machine-readable pack diagnostics.

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::kernel::{DiagnosticCode, JsonPointer, Severity};
use crate::pack::source::PackOrigin;

/// Pack-local location that does not require repo-relative paths.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct PackLocation {
    /// Pack source identity.
    pub origin: PackOrigin,
    /// Optional JSON pointer into the structured pack document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<JsonPointer>,
}

/// Additional context location attached to a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct PackRelatedLocation {
    /// Related pack-local location.
    pub location: PackLocation,
    /// Human-readable relationship context.
    pub message: String,
}

/// Structured pack compiler diagnostic.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct PackDiagnostic {
    /// Stable diagnostic code.
    pub code: DiagnosticCode,
    /// Diagnostic severity.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Primary subject of the diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<PackLocation>,
    /// Additional related locations.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<PackRelatedLocation>,
    /// Optional remediation hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl PackDiagnostic {
    pub(crate) fn error(
        code: &str,
        message: impl Into<String>,
        subject: Option<PackLocation>,
    ) -> Self {
        Self {
            code: DiagnosticCode::parse(code).expect("pack diagnostic code should be valid"),
            severity: Severity::Error,
            message: message.into(),
            subject,
            related: Vec::new(),
            help: None,
        }
    }
}

impl Ord for PackDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        diagnostic_sort_key(self)
            .cmp(&diagnostic_sort_key(other))
            .then_with(|| self.subject.cmp(&other.subject))
            .then_with(|| self.related.cmp(&other.related))
            .then_with(|| self.help.cmp(&other.help))
    }
}

impl PartialOrd for PackDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

type DiagnosticSortKey<'a> = (
    u8,
    Option<&'a PackOrigin>,
    Option<&'a str>,
    &'a str,
    &'a str,
);

fn diagnostic_sort_key(diagnostic: &PackDiagnostic) -> DiagnosticSortKey<'_> {
    (
        severity_rank(diagnostic.severity),
        diagnostic.subject.as_ref().map(|subject| &subject.origin),
        diagnostic
            .subject
            .as_ref()
            .and_then(|subject| subject.path.as_ref().map(|path| path.as_str())),
        diagnostic.code.as_str(),
        diagnostic.message.as_str(),
    )
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::{PackDiagnostic, PackLocation};
    use crate::kernel::{JsonPointer, Severity};
    use crate::pack::source::PackOrigin;

    #[test]
    fn diagnostics_sort_deterministically() {
        let earlier = PackDiagnostic {
            code: crate::kernel::DiagnosticCode::parse("pack.schema.invalid_version")
                .expect("code"),
            severity: Severity::Error,
            message: "first".to_owned(),
            subject: Some(PackLocation {
                origin: PackOrigin::Inline {
                    logical_name: "a".to_owned(),
                },
                path: Some(JsonPointer::parse("/version").expect("pointer")),
            }),
            related: Vec::new(),
            help: None,
        };
        let later = PackDiagnostic {
            code: crate::kernel::DiagnosticCode::parse("pack.schema.invalid_version")
                .expect("code"),
            severity: Severity::Warning,
            message: "second".to_owned(),
            subject: None,
            related: Vec::new(),
            help: None,
        };

        assert!(earlier < later);
    }
}
