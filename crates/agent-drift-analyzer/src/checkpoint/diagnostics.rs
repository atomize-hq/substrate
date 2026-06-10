#![allow(dead_code)]

use std::collections::BTreeSet;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::attempt::{
    AttemptOutcome, CommandAttempt, CommandAttemptRole, VerificationAttempt, VerificationScope,
    VerifierKind,
};
use super::schema::{Confidence, EvidenceRef};

const NORMALIZATION_VERSION: u8 = 1;
const PREVIEW_LIMIT: usize = 160;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FailureClass {
    EnvSetup,
    DependencyResolution,
    CompileType,
    BuildLink,
    LintStyle,
    FormatStyle,
    TestDiscovery,
    TestExecution,
    AssertionOrGolden,
    TimeoutOrHang,
    ToolCrash,
    ReplayMismatch,
    ExitCodeOnly,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct DiagnosticSignature {
    pub normalization_version: u8,
    pub verifier: VerifierKind,
    pub failure_class: FailureClass,
    pub normalized_command: String,
    pub target_fingerprint: Option<String>,
    pub payload_hash_hex: String,
    pub preview: String,
    pub exit_code: Option<i32>,
    pub failing_paths: Vec<String>,
    pub failing_symbols: Vec<String>,
    pub failing_tests: Vec<String>,
    pub failing_count: Option<u32>,
    pub parser_confidence: Confidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DiagnosticMatchKind {
    Exact,
    StrongFuzzy,
    FrontierRelated,
    WeakRelated,
    Unrelated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EditOverlap {
    pub strength: EditOverlapStrength,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum EditOverlapStrength {
    None,
    Weak,
    Moderate,
    Strong,
}

pub(crate) fn build_diagnostic_signatures(
    attempt: &CommandAttempt,
    target_scope: &VerificationScope,
    output: &str,
) -> Vec<DiagnosticSignature> {
    if attempt.outcome == AttemptOutcome::Clean {
        return Vec::new();
    }

    let normalized_lines = canonical_output_lines(output);
    if normalized_lines.is_empty() && attempt.exit_code.is_none() {
        return Vec::new();
    }

    let mut failing_paths = extract_paths_from_lines(&normalized_lines);
    let mut failing_tests = extract_tests_from_lines(&normalized_lines);
    let failing_symbols = extract_symbols_from_lines(&normalized_lines);

    if failing_paths.is_empty() {
        failing_paths = target_scope.paths.clone();
    }
    if failing_tests.is_empty() {
        failing_tests = target_scope.tests.clone();
    }

    let failure_class = classify_failure(attempt, output, &normalized_lines, &failing_tests);
    let failing_count = parse_fail_count(output)
        .or_else(|| (failure_class == FailureClass::ReplayMismatch).then_some(1));
    let parser_confidence = parser_confidence(
        failure_class,
        &failing_paths,
        &failing_tests,
        &failing_symbols,
        failing_count,
    );
    let location_hints =
        discriminative_location_hints(output, &failing_paths, &failing_tests, &failing_symbols);
    let target_fingerprint =
        target_fingerprint(attempt, target_scope, &failing_paths, &failing_tests);
    let payload_hash_hex = hash_payload(
        attempt,
        failure_class,
        failing_count,
        &failing_paths,
        &failing_symbols,
        &failing_tests,
        &normalized_lines,
        &location_hints,
    );
    let preview = normalized_lines
        .first()
        .map(|line| truncate_preview(line))
        .unwrap_or_else(|| format!("{failure_class:?}"));

    vec![DiagnosticSignature {
        normalization_version: NORMALIZATION_VERSION,
        verifier: verifier_for_signature(attempt),
        failure_class,
        normalized_command: attempt.normalized_command.clone(),
        target_fingerprint,
        payload_hash_hex,
        preview,
        exit_code: attempt.exit_code,
        failing_paths,
        failing_symbols,
        failing_tests,
        failing_count,
        parser_confidence,
    }]
}

pub(crate) fn match_diagnostic_signatures(
    previous: &DiagnosticSignature,
    current: &DiagnosticSignature,
) -> DiagnosticMatchKind {
    if previous.verifier == current.verifier
        && previous.failure_class == current.failure_class
        && matching_target_fingerprint(previous, current)
        && previous.payload_hash_hex == current.payload_hash_hex
    {
        return DiagnosticMatchKind::Exact;
    }

    let scope_overlap = scope_overlap(previous, current);
    let verifier_family_matches =
        verifier_family(previous.verifier) == verifier_family(current.verifier);
    let target_matches = scope_overlap >= EditOverlapStrength::Moderate
        || matching_target_fingerprint(previous, current);

    if previous.verifier == current.verifier
        && previous.failure_class == current.failure_class
        && scope_overlap >= EditOverlapStrength::Moderate
    {
        return DiagnosticMatchKind::StrongFuzzy;
    }

    if target_matches
        && comparable_frontier(previous.failure_class, current.failure_class)
        && previous.failure_class != current.failure_class
    {
        return DiagnosticMatchKind::FrontierRelated;
    }

    if verifier_family_matches || scope_overlap >= EditOverlapStrength::Weak {
        return DiagnosticMatchKind::WeakRelated;
    }

    DiagnosticMatchKind::Unrelated
}

pub(crate) fn classify_edit_overlap(
    previous: &VerificationAttempt,
    current: &VerificationAttempt,
    edits: &[CommandAttempt],
) -> EditOverlap {
    let Some((previous_signature, current_signature)) =
        best_comparable_signature_pair(previous, current)
    else {
        return EditOverlap {
            strength: EditOverlapStrength::None,
            evidence: Vec::new(),
        };
    };

    let scope_paths = previous_signature
        .failing_paths
        .iter()
        .chain(current_signature.failing_paths.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let scope_tests = previous_signature
        .failing_tests
        .iter()
        .chain(current_signature.failing_tests.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let scope_symbols = previous_signature
        .failing_symbols
        .iter()
        .chain(current_signature.failing_symbols.iter())
        .cloned()
        .chain(
            scope_tests
                .iter()
                .flat_map(|test| test_scope_tokens(test).into_iter()),
        )
        .collect::<BTreeSet<_>>();

    let mut strongest = EditOverlapStrength::None;
    let mut edit_evidence = Vec::new();

    for edit in edits.iter().filter(|attempt| is_write_attempt(attempt)) {
        let overlap = overlap_for_edit(edit, &scope_paths, &scope_tests, &scope_symbols);
        if overlap > strongest {
            strongest = overlap;
            edit_evidence.clear();
        }
        if overlap == strongest && overlap != EditOverlapStrength::None {
            edit_evidence.push(EvidenceRef {
                row: edit.command_row.clone(),
                reason: "intervening edit overlapped the failing scope before diagnostic frontier changed"
                    .to_string(),
            });
        }
    }

    if strongest == EditOverlapStrength::None {
        return EditOverlap {
            strength: strongest,
            evidence: Vec::new(),
        };
    }

    let mut evidence = vec![
        EvidenceRef {
            row: previous.command_row.clone(),
            reason: "comparable failing verification command".to_string(),
        },
        EvidenceRef {
            row: current.command_row.clone(),
            reason: "later comparable failing verification command".to_string(),
        },
    ];
    evidence.extend(edit_evidence);
    evidence.sort_by(|left, right| {
        (
            left.row.source_file.as_str(),
            left.row.event_index,
            left.reason.as_str(),
        )
            .cmp(&(
                right.row.source_file.as_str(),
                right.row.event_index,
                right.reason.as_str(),
            ))
    });
    evidence.dedup();

    EditOverlap {
        strength: strongest,
        evidence,
    }
}

fn best_comparable_signature_pair<'a>(
    previous: &'a VerificationAttempt,
    current: &'a VerificationAttempt,
) -> Option<(&'a DiagnosticSignature, &'a DiagnosticSignature)> {
    let mut best: Option<(
        &DiagnosticSignature,
        &DiagnosticSignature,
        DiagnosticMatchKind,
    )> = None;

    for previous_signature in &previous.signatures {
        for current_signature in &current.signatures {
            let kind = match_diagnostic_signatures(previous_signature, current_signature);
            if kind == DiagnosticMatchKind::Unrelated {
                continue;
            }
            if best
                .as_ref()
                .is_none_or(|(_, _, best_kind)| kind < *best_kind)
            {
                best = Some((previous_signature, current_signature, kind));
            }
        }
    }

    best.map(|(previous_signature, current_signature, _)| (previous_signature, current_signature))
}

fn verifier_for_signature(attempt: &CommandAttempt) -> VerifierKind {
    match attempt.role {
        CommandAttemptRole::Compile => VerifierKind::CargoCheck,
        CommandAttemptRole::Lint => VerifierKind::CargoClippy,
        CommandAttemptRole::FormatCheck => VerifierKind::CargoFmt,
        CommandAttemptRole::Build if attempt.family == "cargo" => VerifierKind::CargoBuild,
        CommandAttemptRole::Build => VerifierKind::GenericBuild,
        CommandAttemptRole::Replay => VerifierKind::Replay,
        CommandAttemptRole::Test => match attempt.family.as_str() {
            "cargo" => VerifierKind::CargoTest,
            "npm" => VerifierKind::NpmTest,
            "pnpm" => VerifierKind::PnpmTest,
            "pytest" => VerifierKind::Pytest,
            "vitest" => VerifierKind::Vitest,
            "jest" => VerifierKind::Jest,
            "bun" => VerifierKind::BunTest,
            "deno" => VerifierKind::DenoTest,
            _ => VerifierKind::GenericTest,
        },
        _ => VerifierKind::GenericBuild,
    }
}

fn classify_failure(
    attempt: &CommandAttempt,
    output: &str,
    normalized_lines: &[String],
    failing_tests: &[String],
) -> FailureClass {
    let lower = output.to_ascii_lowercase();
    let verifier = verifier_for_signature(attempt);

    if verifier == VerifierKind::Replay {
        if contains_any(
            &lower,
            &[
                "missing field",
                "schema_version",
                "contract gap",
                "fixturecontractgap",
                "expected checkpoint",
            ],
        ) {
            return FailureClass::ReplayMismatch;
        }
    }

    if contains_any(
        &lower,
        &[
            "timed out",
            "timeout",
            "hang detected",
            "killed after",
            "deadline exceeded",
        ],
    ) {
        return FailureClass::TimeoutOrHang;
    }

    if contains_any(
        &lower,
        &[
            "panicked at",
            "segmentation fault",
            "core dumped",
            "internal compiler error",
        ],
    ) {
        return FailureClass::ToolCrash;
    }

    if contains_any(
        &lower,
        &[
            "no matching package named",
            "failed to get",
            "failed to select a version",
            "unable to update",
            "could not resolve host",
        ],
    ) {
        return FailureClass::DependencyResolution;
    }

    if contains_any(
        &lower,
        &[
            "command not found",
            "toolchain",
            "rustup",
            "no such file or directory",
            "permission denied",
        ],
    ) {
        return FailureClass::EnvSetup;
    }

    if verifier == VerifierKind::CargoFmt
        || contains_any(&lower, &["diff in ", "cargo fmt --check", "rustfmt"])
    {
        return FailureClass::FormatStyle;
    }

    if verifier == VerifierKind::CargoClippy || contains_any(&lower, &["clippy", "warning: this"]) {
        return FailureClass::LintStyle;
    }

    if contains_any(
        &lower,
        &[
            "linking with",
            "linker",
            "ld:",
            "undefined reference",
            "unable to find library",
        ],
    ) {
        return FailureClass::BuildLink;
    }

    if contains_any(
        &lower,
        &[
            "could not compile",
            "failed to compile",
            "compilation failed",
            "error[",
            "mismatched types",
            "cannot find",
            "unresolved import",
            "borrow of moved value",
        ],
    ) {
        return FailureClass::CompileType;
    }

    if contains_any(
        &lower,
        &[
            "collected 0 items /",
            "no tests found",
            "did not match any tests",
            "error collecting",
            "test binary failed to start",
        ],
    ) {
        return FailureClass::TestDiscovery;
    }

    if !failing_tests.is_empty()
        && contains_any(
            &lower,
            &[
                "assertionerror",
                "assert_eq!",
                "expected:",
                "snapshot",
                "golden",
                "failures:",
                "test result: failed",
                "replay mismatch",
            ],
        )
    {
        return if verifier == VerifierKind::Replay {
            FailureClass::ReplayMismatch
        } else {
            FailureClass::AssertionOrGolden
        };
    }

    if !failing_tests.is_empty() || contains_any(&lower, &[" failed", " fail ", "failed tests"]) {
        return FailureClass::TestExecution;
    }

    if attempt.exit_code.is_some_and(|code| code != 0) {
        return FailureClass::ExitCodeOnly;
    }

    if normalized_lines.is_empty() {
        FailureClass::Unknown
    } else {
        FailureClass::ExitCodeOnly
    }
}

fn parser_confidence(
    failure_class: FailureClass,
    failing_paths: &[String],
    failing_tests: &[String],
    failing_symbols: &[String],
    failing_count: Option<u32>,
) -> Confidence {
    match failure_class {
        FailureClass::ExitCodeOnly | FailureClass::Unknown => Confidence::Low,
        FailureClass::ReplayMismatch
        | FailureClass::CompileType
        | FailureClass::AssertionOrGolden
            if !failing_paths.is_empty()
                || !failing_tests.is_empty()
                || !failing_symbols.is_empty() =>
        {
            Confidence::High
        }
        _ if failing_count.is_some()
            || !failing_paths.is_empty()
            || !failing_tests.is_empty()
            || !failing_symbols.is_empty() =>
        {
            Confidence::Medium
        }
        _ => Confidence::Low,
    }
}

fn target_fingerprint(
    attempt: &CommandAttempt,
    target_scope: &VerificationScope,
    failing_paths: &[String],
    failing_tests: &[String],
) -> Option<String> {
    let mut parts = BTreeSet::new();
    if let Some(package) = cargo_package_name(&attempt.raw_command) {
        parts.insert(format!("pkg:{package}"));
    }
    for path in failing_paths.iter().chain(target_scope.paths.iter()) {
        parts.insert(format!("path:{path}"));
    }
    for test in failing_tests.iter().chain(target_scope.tests.iter()) {
        parts.insert(format!("test:{test}"));
    }
    if parts.is_empty() && !target_scope.broad {
        parts.insert(format!("cmd:{}", attempt.normalized_command));
    }
    (!parts.is_empty()).then(|| parts.into_iter().collect::<Vec<_>>().join("|"))
}

fn hash_payload(
    attempt: &CommandAttempt,
    failure_class: FailureClass,
    failing_count: Option<u32>,
    failing_paths: &[String],
    failing_symbols: &[String],
    failing_tests: &[String],
    normalized_lines: &[String],
    location_hints: &[String],
) -> String {
    let payload = serde_json::json!({
        "normalization_version": NORMALIZATION_VERSION,
        "verifier": verifier_for_signature(attempt),
        "failure_class": failure_class,
        "command": attempt.normalized_command,
        "exit_code": attempt.exit_code,
        "failing_paths": failing_paths,
        "failing_symbols": failing_symbols,
        "failing_tests": failing_tests,
        "failing_count": failing_count,
        "lines": normalized_lines,
        "location_hints": location_hints,
    });
    let mut hasher = Sha256::new();
    hasher
        .update(serde_json::to_vec(&payload).expect("diagnostic signature payload must serialize"));
    format!("{:x}", hasher.finalize())
}

fn canonical_output_lines(output: &str) -> Vec<String> {
    let sanitized = strip_ansi(output).replace("\r\n", "\n").replace('\r', "\n");
    let mut lines = sanitized
        .lines()
        .map(normalize_line)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    lines.sort();
    lines.dedup();
    lines
}

fn discriminative_location_hints(
    output: &str,
    failing_paths: &[String],
    failing_tests: &[String],
    failing_symbols: &[String],
) -> Vec<String> {
    if !failing_tests.is_empty() || !failing_symbols.is_empty() {
        return Vec::new();
    }

    let locations = extract_location_hints(output);
    if locations.is_empty() {
        return Vec::new();
    }

    let normalized_paths = failing_paths.iter().cloned().collect::<BTreeSet<_>>();
    let filtered = locations
        .into_iter()
        .filter(|location| {
            normalized_paths.is_empty()
                || normalized_paths.contains(&strip_line_column_suffix(location))
        })
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Vec::new();
    }

    let collapsed = filtered
        .iter()
        .map(|location| strip_line_column_suffix(location))
        .collect::<BTreeSet<_>>();
    if collapsed.len() < filtered.len() || normalized_paths.len() == 1 {
        return filtered;
    }

    Vec::new()
}

fn extract_location_hints(output: &str) -> Vec<String> {
    strip_ansi(output)
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .flat_map(|line| {
            line.split_whitespace()
                .filter_map(normalize_location_token)
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalize_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed == "Output:"
        || trimmed.starts_with("Wall time:")
        || trimmed.starts_with("Exit code:")
    {
        return String::new();
    }

    let normalized = trimmed
        .replace('\t', " ")
        .split_whitespace()
        .map(normalize_token)
        .collect::<Vec<_>>()
        .join(" ");
    normalized.trim().to_string()
}

fn normalize_token(token: &str) -> String {
    let without_ansi = strip_ansi(token);
    let mut normalized = without_ansi.replace('\\', "/");
    if let Some(path) = normalize_path_token(&normalized) {
        normalized = path;
    } else if looks_like_duration(&normalized) {
        normalized = "<duration>".to_string();
    } else if looks_like_timestamp(&normalized) {
        normalized = "<timestamp>".to_string();
    } else if looks_like_port(&normalized) {
        normalized = "<port>".to_string();
    } else if looks_like_seed(&normalized) {
        normalized = "<seed>".to_string();
    }
    normalized
}

fn normalize_path_token(token: &str) -> Option<String> {
    let cleaned = token
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
            )
        })
        .replace('\\', "/");
    if cleaned.is_empty() {
        return None;
    }

    let with_repo_root_folded = fold_repo_root_prefix(&cleaned);
    if with_repo_root_folded == cleaned
        && (cleaned.contains("/tmp/")
            || cleaned.contains("/var/folders/")
            || cleaned.contains("AppData/Local/Temp/"))
    {
        return Some("<tmp>".to_string());
    }
    let stripped = strip_line_column_suffix(&with_repo_root_folded);
    if stripped.contains('/') && stripped.chars().any(|ch| ch == '.') {
        return Some(stripped);
    }

    None
}

fn normalize_location_token(token: &str) -> Option<String> {
    let cleaned = token
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
            )
        })
        .replace('\\', "/");
    if cleaned.is_empty() {
        return None;
    }

    let normalized = fold_repo_root_prefix(&cleaned);
    if let Some(preserved) = preserve_line_column_suffix(&normalized) {
        return Some(preserved);
    }

    None
}

fn fold_repo_root_prefix(path: &str) -> String {
    let components = path.split('/').collect::<Vec<_>>();
    for marker in [
        "crates", "src", "tests", "docs", "scripts", "examples", "benches", "fixtures", "target",
    ] {
        if let Some(index) = components.iter().position(|component| *component == marker) {
            return components[index..].join("/");
        }
    }
    path.to_string()
}

fn strip_line_column_suffix(path: &str) -> String {
    let parts = path.split(':').collect::<Vec<_>>();
    if parts.len() >= 3
        && parts[parts.len() - 1].chars().all(|ch| ch.is_ascii_digit())
        && parts[parts.len() - 2].chars().all(|ch| ch.is_ascii_digit())
    {
        return format!("{}:<line>:<col>", parts[..parts.len() - 2].join(":"));
    }
    if parts.len() >= 2 && parts[parts.len() - 1].chars().all(|ch| ch.is_ascii_digit()) {
        return format!("{}:<line>", parts[..parts.len() - 1].join(":"));
    }
    path.to_string()
}

fn preserve_line_column_suffix(path: &str) -> Option<String> {
    let parts = path.split(':').collect::<Vec<_>>();
    if parts.len() >= 3
        && parts[parts.len() - 1].chars().all(|ch| ch.is_ascii_digit())
        && parts[parts.len() - 2].chars().all(|ch| ch.is_ascii_digit())
    {
        let base = parts[..parts.len() - 2].join(":");
        if base.contains('/') && base.chars().any(|ch| ch == '.') {
            return Some(format!(
                "{}:{}:{}",
                base,
                parts[parts.len() - 2],
                parts[parts.len() - 1]
            ));
        }
    }
    if parts.len() >= 2 && parts[parts.len() - 1].chars().all(|ch| ch.is_ascii_digit()) {
        let base = parts[..parts.len() - 1].join(":");
        if base.contains('/') && base.chars().any(|ch| ch == '.') {
            return Some(format!("{}:{}", base, parts[parts.len() - 1]));
        }
    }
    None
}

fn extract_paths_from_lines(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .flat_map(|line| {
            line.split_whitespace()
                .filter_map(normalize_path_token)
                .collect::<Vec<_>>()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn extract_tests_from_lines(lines: &[String]) -> Vec<String> {
    let mut tests = BTreeSet::new();
    let mut active_js_file = None::<String>;
    let mut active_js_suite = None::<String>;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            active_js_suite = None;
            continue;
        }

        for token in line.split_whitespace() {
            let cleaned = trim_test_token(token);
            if cleaned.contains("::") {
                tests.insert(cleaned.to_string());
                if let Some(name) = cleaned.rsplit("::").next() {
                    if name.starts_with("test") {
                        tests.insert(name.to_string());
                    }
                }
            } else if looks_like_test_path(cleaned) {
                tests.insert(cleaned.to_string());
            }
        }

        if let Some(js_file) = extract_js_test_path(trimmed) {
            tests.insert(js_file.clone());
            active_js_file = Some(js_file.clone());
            active_js_suite = None;

            if let Some(descriptor) = extract_inline_js_test_descriptor(trimmed, &js_file) {
                insert_js_test_descriptor(&mut tests, Some(&js_file), &descriptor);
                continue;
            }
        }

        if let Some(descriptor) = extract_standalone_js_test_descriptor(trimmed) {
            insert_js_test_descriptor(&mut tests, active_js_file.as_deref(), &descriptor);
            continue;
        }

        if let Some(test_name) = extract_js_leaf_name(trimmed) {
            let descriptor = active_js_suite
                .as_ref()
                .map(|suite| format!("{suite} > {test_name}"))
                .unwrap_or(test_name);
            insert_js_test_descriptor(&mut tests, active_js_file.as_deref(), &descriptor);
            continue;
        }

        if active_js_file.is_some() && looks_like_js_suite_heading(trimmed) {
            active_js_suite = Some(trimmed.to_string());
        }
    }
    tests.into_iter().collect()
}

fn trim_test_token(token: &str) -> &str {
    token.trim_matches(|ch: char| {
        matches!(
            ch,
            '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':'
        )
    })
}

fn looks_like_test_path(token: &str) -> bool {
    token.contains(".test.")
        || token.contains(".spec.")
        || token.ends_with("_test.py")
        || token.ends_with("_spec.py")
}

fn is_testish_path(path: &str) -> bool {
    path.starts_with("tests/") || path.contains("/tests/") || looks_like_test_path(path)
}

fn extract_js_test_path(line: &str) -> Option<String> {
    line.split_whitespace().find_map(|token| {
        let cleaned = trim_test_token(token);
        let normalized = normalize_path_token(cleaned)?;
        looks_like_test_path(&normalized).then_some(normalized)
    })
}

fn extract_inline_js_test_descriptor(line: &str, file: &str) -> Option<String> {
    let (_, remainder) = line.split_once(file)?;
    let descriptor = remainder
        .trim()
        .trim_start_matches(':')
        .trim()
        .trim_start_matches('>')
        .trim();
    normalize_js_test_descriptor(descriptor)
}

fn extract_standalone_js_test_descriptor(line: &str) -> Option<String> {
    let descriptor = line
        .trim_start_matches("(fail)")
        .trim()
        .trim_start_matches("FAIL")
        .trim()
        .trim_start_matches("✕")
        .trim()
        .trim_start_matches("×")
        .trim();
    if descriptor == line || !descriptor.contains(" > ") {
        return None;
    }
    normalize_js_test_descriptor(descriptor)
}

fn extract_js_leaf_name(line: &str) -> Option<String> {
    for marker in ["✕", "×"] {
        if let Some(leaf) = line.strip_prefix(marker) {
            return normalize_js_test_descriptor(leaf.trim());
        }
    }
    None
}

fn looks_like_js_suite_heading(line: &str) -> bool {
    !line.is_empty()
        && !line.contains('/')
        && !line.contains(':')
        && !line.contains(" > ")
        && !matches!(
            line,
            line if line.starts_with("FAIL")
                || line.starts_with("(fail)")
                || line.starts_with("✕")
                || line.starts_with("×")
                || line.starts_with("Expected")
                || line.starts_with("Assertion")
        )
}

fn normalize_js_test_descriptor(descriptor: &str) -> Option<String> {
    let normalized = descriptor
        .split(" (")
        .next()
        .unwrap_or(descriptor)
        .trim()
        .trim_matches(|ch: char| matches!(ch, '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}'))
        .trim()
        .to_string();
    (!normalized.is_empty()).then_some(normalized)
}

fn insert_js_test_descriptor(tests: &mut BTreeSet<String>, file: Option<&str>, descriptor: &str) {
    if let Some(normalized) = normalize_js_test_descriptor(descriptor) {
        if let Some(file) = file {
            tests.insert(format!("{file} > {normalized}"));
        }
        tests.insert(normalized.clone());
        if let Some(name) = normalized.rsplit(" > ").next() {
            tests.insert(name.to_string());
        }
    }
}

fn extract_symbols_from_lines(lines: &[String]) -> Vec<String> {
    let mut symbols = BTreeSet::new();
    for line in lines {
        if let Some(symbol) = line
            .split("cannot find ")
            .nth(1)
            .and_then(|tail| tail.split_whitespace().next())
        {
            symbols.insert(
                symbol
                    .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '_')
                    .to_string(),
            );
        }
    }
    symbols.retain(|symbol| !symbol.is_empty());
    symbols.into_iter().collect()
}

fn parse_fail_count(output: &str) -> Option<u32> {
    let normalized = strip_ansi(output);
    for line in normalized.lines() {
        if let Some(count) = parse_count_from_line(line, &["failed", "failures"]) {
            return Some(count);
        }
        if let Some(count) = parse_count_from_line(line, &["errors", "error"]) {
            return Some(count);
        }
    }
    None
}

fn parse_count_from_line(line: &str, keywords: &[&str]) -> Option<u32> {
    let tokens = line
        .split_whitespace()
        .map(|token| token.trim_matches(|ch: char| matches!(ch, ',' | ';' | '.' | ')' | '(')))
        .collect::<Vec<_>>();
    for window in tokens.windows(2) {
        if let [count, keyword] = window {
            if let Ok(value) = count.parse::<u32>() {
                let normalized = keyword.to_ascii_lowercase();
                if keywords.iter().any(|candidate| normalized == *candidate) {
                    return Some(value);
                }
            }
        }
    }
    None
}

fn scope_overlap(
    previous: &DiagnosticSignature,
    current: &DiagnosticSignature,
) -> EditOverlapStrength {
    let exact = intersects(&previous.failing_paths, &current.failing_paths)
        || intersects(&previous.failing_tests, &current.failing_tests)
        || intersects(&previous.failing_symbols, &current.failing_symbols);
    if exact {
        return EditOverlapStrength::Strong;
    }

    let same_directory = previous.failing_paths.iter().any(|left| {
        current
            .failing_paths
            .iter()
            .any(|right| same_counterpart_scope(left, right))
    });
    if same_directory {
        return EditOverlapStrength::Moderate;
    }

    let same_crate = previous.failing_paths.iter().any(|left| {
        current
            .failing_paths
            .iter()
            .any(|right| crate_root(left) == crate_root(right))
    });
    if same_crate || matching_target_fingerprint(previous, current) {
        return EditOverlapStrength::Weak;
    }

    EditOverlapStrength::None
}

fn matching_target_fingerprint(
    previous: &DiagnosticSignature,
    current: &DiagnosticSignature,
) -> bool {
    matches!(
        (&previous.target_fingerprint, &current.target_fingerprint),
        (Some(previous), Some(current)) if previous == current
    )
}

fn overlap_for_edit(
    edit: &CommandAttempt,
    failing_paths: &BTreeSet<String>,
    failing_tests: &BTreeSet<String>,
    failing_symbols: &BTreeSet<String>,
) -> EditOverlapStrength {
    if edit
        .paths
        .iter()
        .any(|path| failing_paths.contains(path) || failing_tests.contains(path))
    {
        return EditOverlapStrength::Strong;
    }

    if edit.paths.iter().any(|path| {
        failing_paths
            .iter()
            .any(|failing_path| same_module_or_counterpart_scope(path, failing_path))
            || failing_tests
                .iter()
                .filter_map(|test| normalize_path_token(test))
                .any(|test_path| same_module_or_counterpart_scope(path, &test_path))
            || path_matches_symbol(path, failing_symbols)
    }) {
        return EditOverlapStrength::Moderate;
    }

    if edit.paths.iter().any(|path| {
        failing_paths
            .iter()
            .any(|failing_path| crate_root(path) == crate_root(failing_path))
    }) {
        return EditOverlapStrength::Weak;
    }

    EditOverlapStrength::None
}

fn is_write_attempt(attempt: &CommandAttempt) -> bool {
    matches!(
        attempt.role,
        CommandAttemptRole::Edit
            | CommandAttemptRole::FormatWrite
            | CommandAttemptRole::DependencyMutation
    ) || (attempt.role == CommandAttemptRole::Unknown && !attempt.paths.is_empty())
}

fn comparable_frontier(left: FailureClass, right: FailureClass) -> bool {
    frontier_rank(left).is_some() && frontier_rank(right).is_some()
}

fn frontier_rank(class: FailureClass) -> Option<u8> {
    match class {
        FailureClass::EnvSetup | FailureClass::DependencyResolution => Some(1),
        FailureClass::CompileType | FailureClass::BuildLink => Some(2),
        FailureClass::TestDiscovery => Some(3),
        FailureClass::TestExecution => Some(4),
        FailureClass::AssertionOrGolden | FailureClass::ReplayMismatch => Some(5),
        FailureClass::ExitCodeOnly | FailureClass::Unknown => None,
        FailureClass::LintStyle | FailureClass::FormatStyle => Some(2),
        FailureClass::TimeoutOrHang | FailureClass::ToolCrash => Some(4),
    }
}

fn verifier_family(verifier: VerifierKind) -> &'static str {
    match verifier {
        VerifierKind::CargoCheck
        | VerifierKind::CargoClippy
        | VerifierKind::CargoFmt
        | VerifierKind::CargoBuild => "cargo_build",
        VerifierKind::CargoTest
        | VerifierKind::NpmTest
        | VerifierKind::PnpmTest
        | VerifierKind::Pytest
        | VerifierKind::Vitest
        | VerifierKind::Jest
        | VerifierKind::BunTest
        | VerifierKind::DenoTest
        | VerifierKind::GenericTest => "test",
        VerifierKind::Replay => "replay",
        VerifierKind::GenericBuild => "generic_build",
    }
}

fn cargo_package_name(command: &str) -> Option<String> {
    let tokens = command.split_whitespace().collect::<Vec<_>>();
    for index in 0..tokens.len() {
        if matches!(tokens[index], "-p" | "--package") && tokens.get(index + 1).is_some() {
            return Some(tokens[index + 1].to_string());
        }
    }
    None
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn truncate_preview(text: &str) -> String {
    let mut preview = text.chars().take(PREVIEW_LIMIT).collect::<String>();
    if text.chars().count() > PREVIEW_LIMIT {
        preview.push('…');
    }
    preview
}

fn strip_ansi(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek().copied() == Some('[') {
                chars.next();
                while let Some(next) = chars.next() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        result.push(ch);
    }
    result
}

fn looks_like_duration(token: &str) -> bool {
    let trimmed = token.trim_matches(|ch: char| matches!(ch, ',' | ';' | '.'));
    let mut digits = false;
    let mut suffix = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            digits = true;
            continue;
        }
        suffix.push(ch);
    }
    digits
        && matches!(
            suffix.as_str(),
            "ms" | "s" | "sec" | "secs" | "seconds" | "m" | "min" | "mins"
        )
}

fn looks_like_timestamp(token: &str) -> bool {
    let trimmed = token.trim_matches(|ch: char| matches!(ch, ',' | ';'));
    trimmed.len() >= 8
        && (trimmed.contains('T') || trimmed.matches(':').count() >= 2)
        && trimmed.chars().any(|ch| ch.is_ascii_digit())
}

fn looks_like_port(token: &str) -> bool {
    token.starts_with("127.0.0.1:")
        || token.starts_with("localhost:")
        || (token.starts_with(':')
            && token[1..].chars().all(|ch| ch.is_ascii_digit())
            && token.len() > 2)
}

fn looks_like_seed(token: &str) -> bool {
    token.starts_with("seed=") || token.starts_with("seed:")
}

fn intersects(left: &[String], right: &[String]) -> bool {
    left.iter().any(|item| right.contains(item))
}

fn parent_dir(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn file_stem(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(_, tail)| tail)
        .unwrap_or(path)
        .split('.')
        .next()
        .unwrap_or(path)
        .to_string()
}

fn same_counterpart_scope(left: &str, right: &str) -> bool {
    parent_dir(left) == parent_dir(right)
        && normalize_counterpart_stem(&file_stem(left))
            == normalize_counterpart_stem(&file_stem(right))
}

fn same_module_or_counterpart_scope(left: &str, right: &str) -> bool {
    parent_dir(left) == parent_dir(right) || same_test_source_counterpart(left, right)
}

fn same_test_source_counterpart(left: &str, right: &str) -> bool {
    crate_root(left) == crate_root(right)
        && normalize_counterpart_stem(&file_stem(left))
            == normalize_counterpart_stem(&file_stem(right))
        && is_testish_path(left) != is_testish_path(right)
}

fn path_matches_symbol(path: &str, symbols: &BTreeSet<String>) -> bool {
    let normalized = normalize_counterpart_stem(&file_stem(path));
    symbols.iter().any(|symbol| normalized == *symbol)
}

fn normalize_counterpart_stem(stem: &str) -> String {
    stem.replace("_test", "")
        .replace(".test", "")
        .replace("-test", "")
        .replace("_spec", "")
        .replace(".spec", "")
        .replace("-spec", "")
}

fn test_scope_tokens(test: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let parts = test.split("::").collect::<Vec<_>>();
    if parts.len() > 1 {
        for token in &parts[..parts.len() - 1] {
            let normalized = normalize_counterpart_stem(token);
            if !normalized.is_empty() {
                tokens.push(normalized);
            }
        }
    }
    tokens
}

fn crate_root(path: &str) -> String {
    let mut components = path.split('/');
    match (components.next(), components.next(), components.next()) {
        (Some("crates"), Some(crate_name), _) => format!("crates/{crate_name}"),
        (Some(first), Some(second), _) => format!("{first}/{second}"),
        (Some(first), None, None) => first.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, SourceKind};
    use camino::Utf8PathBuf;

    use super::{
        build_diagnostic_signatures, classify_edit_overlap, match_diagnostic_signatures,
        DiagnosticMatchKind, EditOverlapStrength, FailureClass,
    };
    use crate::checkpoint::attempt::{
        AttemptOutcome, CommandAttempt, CommandAttemptRole, ExerciseState, VerificationAttempt,
        VerificationScope, VerifierKind,
    };

    #[test]
    fn checkpoints_canonicalize_diagnostics_across_ansi_paths_and_temp_root_noise() {
        let attempt = test_attempt(
            1,
            10,
            "cargo check -p agent-drift-analyzer",
            CommandAttemptRole::Compile,
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
        );
        let scope = VerificationScope {
            raw: attempt.raw_command.clone(),
            paths: attempt.paths.clone(),
            tests: Vec::new(),
            broad: false,
        };
        let left = build_diagnostic_signatures(
            &attempt,
            &scope,
            "\u{1b}[31merror[E0308]\u{1b}[0m mismatched types\n --> /Users/spensermcconnell/.codex/worktrees/97a0/substrate/crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs:12:34\nWall time: 0.12 seconds\nExit code: 101",
        );
        let right = build_diagnostic_signatures(
            &attempt,
            &scope,
            "error[E0308] mismatched types\n --> /private/tmp/build-48271/crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs:12:34\nWall time: 9.48 seconds\nExit code: 101",
        );

        assert_eq!(left.len(), 1);
        assert_eq!(right.len(), 1);
        assert_eq!(left[0].payload_hash_hex, right[0].payload_hash_hex);
        assert_eq!(left[0].failure_class, FailureClass::CompileType);
        assert_eq!(left[0].failing_paths, right[0].failing_paths);
    }

    #[test]
    fn checkpoints_keep_location_when_it_is_the_only_discriminative_signal() {
        let attempt = test_attempt(
            1,
            10,
            "cargo check -p agent-drift-analyzer",
            CommandAttemptRole::Compile,
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
        );
        let scope = VerificationScope {
            raw: attempt.raw_command.clone(),
            paths: attempt.paths.clone(),
            tests: Vec::new(),
            broad: false,
        };
        let left = build_diagnostic_signatures(
            &attempt,
            &scope,
            "error[E0308] mismatched types\n --> crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs:12:34\nExit code: 101",
        );
        let right = build_diagnostic_signatures(
            &attempt,
            &scope,
            "error[E0308] mismatched types\n --> crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs:77:9\nExit code: 101",
        );

        assert_eq!(left[0].failing_paths, right[0].failing_paths);
        assert_ne!(left[0].payload_hash_hex, right[0].payload_hash_hex);
    }

    #[test]
    fn checkpoints_parse_initial_failure_classes_and_fail_counts() {
        let cargo_attempt = test_attempt(
            1,
            10,
            "cargo test -p agent-drift-analyzer checkpoints",
            CommandAttemptRole::Test,
            vec![],
        );
        let cargo_scope = VerificationScope {
            raw: cargo_attempt.raw_command.clone(),
            paths: Vec::new(),
            tests: vec!["checkpoints".to_string()],
            broad: false,
        };
        let cargo_signature = build_diagnostic_signatures(
            &cargo_attempt,
            &cargo_scope,
            "running 3 tests\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\nAssertionError: expected advancing\nExit code: 101",
        );
        assert_eq!(
            cargo_signature[0].failure_class,
            FailureClass::AssertionOrGolden
        );
        assert_eq!(cargo_signature[0].failing_count, Some(1));
        assert!(cargo_signature[0]
            .failing_tests
            .contains(&"checkpoints::captures_progress".to_string()));

        let pytest_attempt = test_attempt(
            2,
            20,
            "pytest tests/checkpoints_test.py::test_progress",
            CommandAttemptRole::Test,
            vec!["tests/checkpoints_test.py".to_string()],
        );
        let pytest_scope = VerificationScope {
            raw: pytest_attempt.raw_command.clone(),
            paths: vec!["tests/checkpoints_test.py".to_string()],
            tests: vec![
                "tests/checkpoints_test.py::test_progress".to_string(),
                "test_progress".to_string(),
            ],
            broad: false,
        };
        let pytest_signature = build_diagnostic_signatures(
            &pytest_attempt,
            &pytest_scope,
            "=========================== short test summary info ============================\nFAILED tests/checkpoints_test.py::test_progress - AssertionError: expected advancing\n========================= 1 failed, 4 passed in 0.32s =========================",
        );
        assert_eq!(
            pytest_signature[0].failure_class,
            FailureClass::AssertionOrGolden
        );
        assert_eq!(pytest_signature[0].failing_count, Some(1));

        let replay_attempt = test_attempt(
            3,
            30,
            "cargo run -p agent-drift-sentinel -- replay fixture",
            CommandAttemptRole::Replay,
            vec!["target/hybrid-drift-evals/session-alpha".to_string()],
        );
        let replay_scope = VerificationScope {
            raw: replay_attempt.raw_command.clone(),
            paths: replay_attempt.paths.clone(),
            tests: Vec::new(),
            broad: false,
        };
        let replay_signature = build_diagnostic_signatures(
            &replay_attempt,
            &replay_scope,
            "InputError::FixtureContractGap: missing field `session_progress` for v0.6 checkpoint\nExit code: 1",
        );
        assert_eq!(
            replay_signature[0].failure_class,
            FailureClass::ReplayMismatch
        );
        assert_eq!(replay_signature[0].failing_count, Some(1));
    }

    #[test]
    fn checkpoints_match_exact_strong_frontier_weak_and_unrelated_signatures() {
        let exact = diagnostic_signature(
            VerifierKind::CargoTest,
            FailureClass::AssertionOrGolden,
            Some("pkg:agent-drift-analyzer|test:checkpoints".to_string()),
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
            vec!["checkpoints::captures_progress".to_string()],
            "same-hash",
        );
        assert_eq!(
            match_diagnostic_signatures(&exact, &exact),
            DiagnosticMatchKind::Exact
        );

        let strong_left = diagnostic_signature(
            VerifierKind::CargoTest,
            FailureClass::AssertionOrGolden,
            Some("pkg:agent-drift-analyzer|test:checkpoints".to_string()),
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
            vec!["checkpoints::captures_progress".to_string()],
            "hash-a",
        );
        let strong_right = diagnostic_signature(
            VerifierKind::CargoTest,
            FailureClass::AssertionOrGolden,
            Some("pkg:agent-drift-analyzer|test:checkpoints".to_string()),
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
            vec!["checkpoints::captures_progress".to_string()],
            "hash-b",
        );
        assert_eq!(
            match_diagnostic_signatures(&strong_left, &strong_right),
            DiagnosticMatchKind::StrongFuzzy
        );

        let frontier_left = diagnostic_signature(
            VerifierKind::CargoCheck,
            FailureClass::CompileType,
            Some("pkg:agent-drift-analyzer|path:crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()),
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
            Vec::new(),
            "compile-hash",
        );
        let frontier_right = diagnostic_signature(
            VerifierKind::CargoTest,
            FailureClass::AssertionOrGolden,
            Some("pkg:agent-drift-analyzer|path:crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()),
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
            vec!["checkpoints::captures_progress".to_string()],
            "assertion-hash",
        );
        assert_eq!(
            match_diagnostic_signatures(&frontier_left, &frontier_right),
            DiagnosticMatchKind::FrontierRelated
        );

        let broad_left = diagnostic_signature(
            VerifierKind::CargoCheck,
            FailureClass::CompileType,
            None,
            Vec::new(),
            Vec::new(),
            "broad-compile",
        );
        let broad_right = diagnostic_signature(
            VerifierKind::CargoTest,
            FailureClass::AssertionOrGolden,
            None,
            Vec::new(),
            Vec::new(),
            "broad-test",
        );
        assert_eq!(
            match_diagnostic_signatures(&broad_left, &broad_right),
            DiagnosticMatchKind::Unrelated
        );

        let weak_left = diagnostic_signature(
            VerifierKind::Pytest,
            FailureClass::AssertionOrGolden,
            Some("path:tests/checkpoints_test.py".to_string()),
            vec!["tests/checkpoints_test.py".to_string()],
            vec!["tests/checkpoints_test.py::test_progress".to_string()],
            "pytest-a",
        );
        let weak_right = diagnostic_signature(
            VerifierKind::Pytest,
            FailureClass::TestExecution,
            Some("path:tests/other_test.py".to_string()),
            vec!["tests/other_test.py".to_string()],
            vec!["tests/other_test.py::test_progress".to_string()],
            "pytest-b",
        );
        assert_eq!(
            match_diagnostic_signatures(&weak_left, &weak_right),
            DiagnosticMatchKind::WeakRelated
        );

        let unrelated = diagnostic_signature(
            VerifierKind::Replay,
            FailureClass::ReplayMismatch,
            Some("path:target/hybrid-drift-evals/session-alpha".to_string()),
            vec!["target/hybrid-drift-evals/session-alpha".to_string()],
            Vec::new(),
            "replay",
        );
        assert_eq!(
            match_diagnostic_signatures(&strong_left, &unrelated),
            DiagnosticMatchKind::Unrelated
        );
    }

    #[test]
    fn checkpoints_classify_edit_overlap_for_comparable_attempts() {
        let previous = verification_attempt(
            1,
            10,
            diagnostic_signature(
                VerifierKind::CargoCheck,
                FailureClass::CompileType,
                Some("pkg:agent-drift-analyzer|path:crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()),
                vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
                Vec::new(),
                "compile",
            ),
        );
        let current = verification_attempt(
            2,
            20,
            diagnostic_signature(
                VerifierKind::CargoTest,
                FailureClass::AssertionOrGolden,
                Some("pkg:agent-drift-analyzer|path:crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()),
                vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
                vec!["checkpoints::captures_progress".to_string()],
                "assertion",
            ),
        );

        let overlapping_edit = test_attempt(
            3,
            15,
            "apply_patch <<'PATCH'\n*** Begin Patch",
            CommandAttemptRole::Edit,
            vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
        );
        let unrelated_edit = test_attempt(
            4,
            16,
            "apply_patch <<'PATCH'\n*** Begin Patch",
            CommandAttemptRole::Edit,
            vec!["docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md".to_string()],
        );

        let strong = classify_edit_overlap(
            &previous,
            &current,
            &[overlapping_edit.clone(), unrelated_edit.clone()],
        );
        assert_eq!(strong.strength, EditOverlapStrength::Strong);
        assert_eq!(strong.evidence.len(), 3);

        let none = classify_edit_overlap(&previous, &current, &[unrelated_edit]);
        assert_eq!(none.strength, EditOverlapStrength::None);
        assert!(none.evidence.is_empty());
    }

    #[test]
    fn checkpoints_classify_edit_overlap_for_write_attempts_and_test_symbol_scope() {
        let previous = verification_attempt(
            1,
            10,
            diagnostic_signature(
                VerifierKind::CargoTest,
                FailureClass::AssertionOrGolden,
                Some("pkg:agent-drift-analyzer|test:checkpoints".to_string()),
                Vec::new(),
                vec!["checkpoints::captures_progress".to_string()],
                "before",
            ),
        );
        let current = verification_attempt(
            2,
            20,
            diagnostic_signature(
                VerifierKind::CargoTest,
                FailureClass::AssertionOrGolden,
                Some("pkg:agent-drift-analyzer|test:checkpoints".to_string()),
                Vec::new(),
                vec!["checkpoints::captures_progress".to_string()],
                "after",
            ),
        );

        let format_write = test_attempt(
            3,
            15,
            "cargo fmt --all",
            CommandAttemptRole::FormatWrite,
            vec!["crates/agent-drift-analyzer/tests/checkpoints.rs".to_string()],
        );

        let overlap = classify_edit_overlap(&previous, &current, &[format_write]);
        assert_eq!(overlap.strength, EditOverlapStrength::Moderate);
        assert_eq!(overlap.evidence.len(), 3);
    }

    #[test]
    fn checkpoints_classify_edit_overlap_for_same_module_sibling_file_edits() {
        let previous = verification_attempt(
            1,
            10,
            diagnostic_signature(
                VerifierKind::CargoCheck,
                FailureClass::CompileType,
                Some("pkg:agent-drift-analyzer|path:crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()),
                vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
                Vec::new(),
                "compile",
            ),
        );
        let current = verification_attempt(
            2,
            20,
            diagnostic_signature(
                VerifierKind::CargoCheck,
                FailureClass::CompileType,
                Some("pkg:agent-drift-analyzer|path:crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()),
                vec!["crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs".to_string()],
                Vec::new(),
                "compile-next",
            ),
        );

        let sibling_module_edit = test_attempt(
            3,
            15,
            "apply_patch <<'PATCH'\n*** Begin Patch",
            CommandAttemptRole::Edit,
            vec!["crates/agent-drift-analyzer/src/checkpoint/mod.rs".to_string()],
        );

        let overlap = classify_edit_overlap(&previous, &current, &[sibling_module_edit]);
        assert_eq!(overlap.strength, EditOverlapStrength::Moderate);
        assert_eq!(overlap.evidence.len(), 3);
    }

    #[test]
    fn checkpoints_parse_js_test_like_failure_counts_and_names() {
        let cases = [
            (
                test_attempt(1, 10, "vitest run", CommandAttemptRole::Test, vec![]),
                VerificationScope {
                    raw: "vitest run".to_string(),
                    paths: Vec::new(),
                    tests: Vec::new(),
                    broad: true,
                },
                "FAIL tests/checkpoints.test.ts > progress window > rejects stale frontier\nTests 1 failed (1)\nAssertionError: expected frontier advance",
                vec![
                    "tests/checkpoints.test.ts".to_string(),
                    "tests/checkpoints.test.ts > progress window > rejects stale frontier"
                        .to_string(),
                    "progress window > rejects stale frontier".to_string(),
                    "rejects stale frontier".to_string(),
                ],
            ),
            (
                test_attempt(2, 20, "jest", CommandAttemptRole::Test, vec![]),
                VerificationScope {
                    raw: "jest".to_string(),
                    paths: Vec::new(),
                    tests: Vec::new(),
                    broad: true,
                },
                "FAIL tests/checkpoints.test.ts\n  progress window\n    ✕ rejects stale frontier (2 ms)\nAssertionError: expected frontier advance\nTests:       1 failed, 1 total",
                vec![
                    "tests/checkpoints.test.ts".to_string(),
                    "tests/checkpoints.test.ts > progress window > rejects stale frontier"
                        .to_string(),
                    "progress window > rejects stale frontier".to_string(),
                    "rejects stale frontier".to_string(),
                ],
            ),
        ];

        for (attempt, scope, output, expected_tests) in cases {
            let signatures = build_diagnostic_signatures(&attempt, &scope, output);

            assert_eq!(signatures[0].failure_class, FailureClass::AssertionOrGolden);
            assert_eq!(signatures[0].failing_count, Some(1));
            assert_eq!(
                signatures[0].parser_confidence,
                crate::checkpoint::Confidence::High
            );
            for expected in expected_tests {
                assert!(signatures[0].failing_tests.contains(&expected));
            }
        }
    }

    fn diagnostic_signature(
        verifier: VerifierKind,
        failure_class: FailureClass,
        target_fingerprint: Option<String>,
        failing_paths: Vec<String>,
        failing_tests: Vec<String>,
        payload_hash_hex: &str,
    ) -> super::DiagnosticSignature {
        super::DiagnosticSignature {
            normalization_version: 1,
            verifier,
            failure_class,
            normalized_command: "cargo test -p agent-drift-analyzer checkpoints".to_string(),
            target_fingerprint,
            payload_hash_hex: payload_hash_hex.to_string(),
            preview: "preview".to_string(),
            exit_code: Some(1),
            failing_paths,
            failing_symbols: Vec::new(),
            failing_tests,
            failing_count: Some(1),
            parser_confidence: crate::checkpoint::Confidence::High,
        }
    }

    fn verification_attempt(
        ordinal: usize,
        event_index: usize,
        signature: super::DiagnosticSignature,
    ) -> VerificationAttempt {
        VerificationAttempt {
            attempt_ordinal: ordinal,
            command_row: RowRef::from_row(&row(
                event_index,
                CompactionKind::ToolCall,
                "cargo test -p agent-drift-analyzer checkpoints",
            )),
            verifier: signature.verifier,
            target_scope: VerificationScope {
                raw: "cargo test -p agent-drift-analyzer checkpoints".to_string(),
                paths: signature.failing_paths.clone(),
                tests: signature.failing_tests.clone(),
                broad: false,
            },
            exercise_state: ExerciseState::TargetExercised,
            outcome: AttemptOutcome::Failed,
            signatures: vec![signature],
        }
    }

    fn test_attempt(
        ordinal: usize,
        event_index: usize,
        raw_command: &str,
        role: CommandAttemptRole,
        paths: Vec<String>,
    ) -> CommandAttempt {
        let row = row(event_index, CompactionKind::ToolCall, raw_command);
        let family = raw_command
            .split_whitespace()
            .next()
            .unwrap_or("cargo")
            .to_string();
        CommandAttempt {
            ordinal,
            command_row: RowRef::from_row(&row),
            output_rows: Vec::new(),
            tool_name: "functions.shell_command".to_string(),
            family,
            raw_command: raw_command.to_string(),
            normalized_command: raw_command.to_string(),
            role,
            paths,
            outcome: AttemptOutcome::Failed,
            exit_code: Some(1),
        }
    }

    fn row(event_index: usize, kind: CompactionKind, text: &str) -> CompactionRow {
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-alpha".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index,
            line_number: event_index + 1,
            row_ordinal: event_index,
            timestamp: None,
            kind,
            user_message_role: None,
            dedupe_identity: None,
            text: text.to_string(),
            canonical_text: text.to_string(),
            text_hash_hex: format!("{kind:?}-{event_index}"),
        }
    }
}
