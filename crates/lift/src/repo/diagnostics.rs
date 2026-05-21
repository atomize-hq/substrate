use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::kernel::{DiagnosticCode, RepoPath, Severity};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct RepoLocation {
    pub display_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<RepoPath>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub(crate) struct RepoRelatedLocation {
    pub location: RepoLocation,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct RepoDiagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<RepoLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RepoRelatedLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
}

impl Ord for RepoDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        diagnostic_sort_key(self)
            .cmp(&diagnostic_sort_key(other))
            .then_with(|| self.subject.cmp(&other.subject))
            .then_with(|| self.related.cmp(&other.related))
            .then_with(|| self.help.cmp(&other.help))
    }
}

impl PartialOrd for RepoDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

type RepoDiagnosticSortKey<'a> = (u8, Option<&'a str>, Option<&'a str>, &'a str, &'a str);

fn diagnostic_sort_key(diagnostic: &RepoDiagnostic) -> RepoDiagnosticSortKey<'_> {
    (
        severity_rank(diagnostic.severity),
        diagnostic
            .subject
            .as_ref()
            .map(|subject| subject.display_path.as_str()),
        diagnostic
            .subject
            .as_ref()
            .and_then(|subject| subject.repo_path.as_ref().map(RepoPath::as_str)),
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
