use std::collections::BTreeMap;

use agent_session_compactor::{CompactionKind, CompactionRow, RowRef};
use camino::Utf8PathBuf;
use serde::Serialize;
use serde_json::Value;

use crate::context::CommandObservation;

use super::diagnostics::{build_diagnostic_signatures, DiagnosticSignature};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommandAttempt {
    pub ordinal: usize,
    pub command_row: RowRef,
    pub output_rows: Vec<RowRef>,
    pub tool_name: String,
    pub family: String,
    pub raw_command: String,
    pub normalized_command: String,
    pub role: CommandAttemptRole,
    pub paths: Vec<String>,
    pub outcome: AttemptOutcome,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CommandAttemptRole {
    Read,
    Edit,
    Compile,
    Test,
    Lint,
    FormatCheck,
    FormatWrite,
    Build,
    DependencyMutation,
    VcsInspection,
    Replay,
    Orchestration,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum AttemptOutcome {
    Clean,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerificationAttempt {
    pub attempt_ordinal: usize,
    pub command_row: RowRef,
    pub verifier: VerifierKind,
    pub target_scope: VerificationScope,
    pub exercise_state: ExerciseState,
    pub outcome: AttemptOutcome,
    pub signatures: Vec<DiagnosticSignature>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub(crate) enum VerifierKind {
    CargoCheck,
    CargoTest,
    CargoClippy,
    CargoFmt,
    CargoBuild,
    NpmTest,
    PnpmTest,
    Pytest,
    Vitest,
    Jest,
    BunTest,
    DenoTest,
    Replay,
    GenericTest,
    GenericBuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ExerciseState {
    TargetExercised,
    BlockedBeforeTarget,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerificationScope {
    pub raw: String,
    pub paths: Vec<String>,
    pub tests: Vec<String>,
    pub broad: bool,
}

pub(crate) fn build_command_attempts(
    rows: &[CompactionRow],
    command_observations: &[CommandObservation],
) -> Vec<CommandAttempt> {
    let observations_by_row = command_observations
        .iter()
        .filter_map(|observation| {
            observation
                .evidence
                .first()
                .map(|evidence| (row_ref_key(&evidence.row), observation))
        })
        .collect::<BTreeMap<_, _>>();

    rows.iter()
        .enumerate()
        .filter(|(_, row)| row.kind == CompactionKind::ToolCall)
        .enumerate()
        .map(|(ordinal, (row_index, row))| {
            let observation = observations_by_row.get(&row_key(row)).copied();
            let tool_name = observation
                .map(|observation| observation.tool_name.clone())
                .unwrap_or_else(|| tool_name(row));
            let raw_command = observation
                .map(|observation| observation.raw_command.clone())
                .unwrap_or_else(|| row.text.clone());
            let family = observation
                .map(|observation| observation.family.clone())
                .unwrap_or_else(|| command_family(&raw_command).unwrap_or(tool_name.clone()));
            let mut paths = observation
                .map(|observation| observation.paths.clone())
                .unwrap_or_default();
            paths.sort();
            paths.dedup();

            let paired_rows = pair_output_rows(rows, row_index);
            let exit_code = parse_exit_code(&paired_rows);
            let outcome = derive_attempt_outcome(&paired_rows, exit_code);
            let role = classify_command_attempt_role(&raw_command, &tool_name, &family);

            CommandAttempt {
                ordinal: ordinal + 1,
                command_row: RowRef::from_row(row),
                output_rows: paired_rows
                    .iter()
                    .map(|row| RowRef::from_row(row))
                    .collect(),
                tool_name,
                family,
                raw_command: raw_command.clone(),
                normalized_command: normalize_command(&raw_command),
                role,
                paths,
                outcome,
                exit_code,
            }
        })
        .collect()
}

pub(crate) fn build_verification_attempts(
    attempts: &[CommandAttempt],
    rows: &[CompactionRow],
) -> Vec<VerificationAttempt> {
    attempts
        .iter()
        .filter_map(|attempt| {
            let verifier = verifier_kind(attempt)?;
            let target_scope = verification_scope(attempt);
            let output = attempt_output_text(attempt, rows);
            let execution_evidence = (attempt.role == CommandAttemptRole::Test
                && !test_attempt_uses_nonexecuting_flags(attempt))
            .then(|| test_execution_evidence(verifier, &output, &target_scope));
            let outcome = verification_outcome(attempt.outcome, execution_evidence);
            let mut diagnostic_attempt = attempt.clone();
            diagnostic_attempt.outcome = outcome;
            Some(VerificationAttempt {
                attempt_ordinal: attempt.ordinal,
                command_row: attempt.command_row.clone(),
                verifier,
                exercise_state: exercise_state(attempt, execution_evidence, &output),
                signatures: build_diagnostic_signatures(
                    &diagnostic_attempt,
                    &target_scope,
                    &output,
                ),
                target_scope,
                outcome,
            })
        })
        .collect()
}

pub(crate) fn verification_target_from_command(
    command: &str,
) -> Option<(VerifierKind, VerificationScope)> {
    let family = command_family(command)?;
    let tokens = normalized_command_tokens(command);
    let role = classify_command_attempt_role(command, &family, &family);
    let verifier = match role {
        CommandAttemptRole::Compile => Some(VerifierKind::CargoCheck),
        CommandAttemptRole::Lint => Some(VerifierKind::CargoClippy),
        CommandAttemptRole::FormatCheck => Some(VerifierKind::CargoFmt),
        CommandAttemptRole::Build => Some(if family == "cargo" {
            VerifierKind::CargoBuild
        } else {
            VerifierKind::GenericBuild
        }),
        CommandAttemptRole::Replay => Some(VerifierKind::Replay),
        CommandAttemptRole::Test => Some(match family.as_str() {
            "cargo" => VerifierKind::CargoTest,
            "npm" => VerifierKind::NpmTest,
            "pnpm" => VerifierKind::PnpmTest,
            "pytest" => VerifierKind::Pytest,
            "vitest" => VerifierKind::Vitest,
            "jest" => VerifierKind::Jest,
            "bun" => VerifierKind::BunTest,
            "deno" => VerifierKind::DenoTest,
            "npx" if tokens.iter().any(|token| token == "vitest") => VerifierKind::Vitest,
            "npx" if tokens.iter().any(|token| token == "jest") => VerifierKind::Jest,
            "python" | "uv" if tokens.iter().any(|token| token == "pytest") => VerifierKind::Pytest,
            _ => VerifierKind::GenericTest,
        }),
        _ => None,
    }?;

    let mut paths = Vec::new();
    let mut tests = Vec::new();

    match family.as_str() {
        "cargo" if role == CommandAttemptRole::Test => {
            tests.extend(cargo_test_targets(&tokens));
        }
        "pytest" => {
            let (more_paths, more_tests) =
                pytest_targets(tokens.iter().skip(1).map(String::as_str));
            paths.extend(more_paths);
            tests.extend(more_tests);
        }
        "python" | "uv"
            if role == CommandAttemptRole::Test && tokens.iter().any(|token| token == "pytest") =>
        {
            let pytest_index = tokens
                .iter()
                .position(|token| token == "pytest")
                .unwrap_or(tokens.len());
            let (more_paths, more_tests) =
                pytest_targets(tokens.iter().skip(pytest_index + 1).map(String::as_str));
            paths.extend(more_paths);
            tests.extend(more_tests);
        }
        "npm" | "pnpm" | "yarn" | "npx" | "vitest" | "jest" | "bun" | "deno"
            if role == CommandAttemptRole::Test =>
        {
            let (more_paths, more_tests) = js_test_targets(&tokens, &family);
            paths.extend(more_paths);
            tests.extend(more_tests);
        }
        _ if role == CommandAttemptRole::Test => {
            tests.extend(generic_test_targets(&tokens, &family));
        }
        _ => {}
    }

    paths.sort();
    paths.dedup();
    tests.sort();
    tests.dedup();

    Some((
        verifier,
        VerificationScope {
            raw: command.trim().to_string(),
            broad: paths.is_empty() && tests.is_empty(),
            paths,
            tests,
        },
    ))
}

fn pair_output_rows(rows: &[CompactionRow], command_index: usize) -> Vec<&CompactionRow> {
    if let Some(command_call_id) = tool_call_id(&rows[command_index]) {
        return rows
            .iter()
            .skip(command_index + 1)
            .take_while(|row| !row_starts_new_phase(row))
            .filter(|row| {
                row.kind == CompactionKind::ToolOutput
                    && tool_call_id(row).as_deref() == Some(command_call_id.as_str())
            })
            .collect();
    }

    let mut paired = Vec::new();
    for (offset, row) in rows.iter().enumerate().skip(command_index + 1) {
        if row.kind == CompactionKind::ToolCall || row_starts_new_phase(row) {
            break;
        }

        match row.kind {
            CompactionKind::ToolOutput | CompactionKind::Error => paired.push(row),
            CompactionKind::Unknown
                if offset == command_index + 1 && row_looks_like_attempt_output(&row.text) =>
            {
                paired.push(row);
            }
            _ => {}
        }
    }
    paired
}

fn tool_call_id(row: &CompactionRow) -> Option<String> {
    row.dedupe_identity
        .as_deref()
        .and_then(|identity| serde_json::from_str::<Value>(identity).ok())
        .and_then(|value| {
            value
                .get("call_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn derive_attempt_outcome(rows: &[&CompactionRow], exit_code: Option<i32>) -> AttemptOutcome {
    if let Some(code) = exit_code {
        return if code == 0 {
            AttemptOutcome::Clean
        } else {
            AttemptOutcome::Failed
        };
    }

    if rows.iter().any(|row| row.kind == CompactionKind::Error) {
        return AttemptOutcome::Failed;
    }

    AttemptOutcome::Unknown
}

fn parse_exit_code(rows: &[&CompactionRow]) -> Option<i32> {
    for row in rows {
        if let Some(code) = parse_exit_code_from_row_header(&row.text) {
            return Some(code);
        }
    }

    None
}

fn parse_exit_code_from_row_header(text: &str) -> Option<i32> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("Exit code: ") {
            return value.trim().parse::<i32>().ok();
        }
        if matches!(trimmed, "Output:") || trimmed.starts_with("Wall time: ") {
            continue;
        }

        break;
    }

    None
}

fn normalize_command(command: &str) -> String {
    normalized_command_tokens(command).join(" ")
}

fn classify_command_attempt_role(
    raw_command: &str,
    tool_name: &str,
    family: &str,
) -> CommandAttemptRole {
    let normalized = raw_command.to_ascii_lowercase();
    let tokens = normalized_command_tokens(raw_command);

    if matches!(
        tool_name,
        "spawn_agent" | "wait_agent" | "close_agent" | "multi_agent_v1"
    ) {
        return CommandAttemptRole::Orchestration;
    }

    if matches!(family, "apply_patch" | "mkdir" | "mv" | "cp")
        || normalized.contains("*** begin patch")
    {
        return CommandAttemptRole::Edit;
    }

    if matches!(
        family,
        "cat" | "sed" | "rg" | "ls" | "find" | "head" | "tail" | "jq"
    ) {
        return CommandAttemptRole::Read;
    }

    if family == "git" {
        return git_role(&tokens).unwrap_or(CommandAttemptRole::Unknown);
    }

    if let Some(role) = cargo_role(&tokens) {
        return role;
    }

    if let Some(role) = npm_like_role(&tokens, family) {
        return role;
    }

    if let Some(role) = shell_wrapped_test_role(&tokens) {
        return role;
    }

    if family.eq_ignore_ascii_case("replay") || tokens.iter().any(|token| token == "replay") {
        return CommandAttemptRole::Replay;
    }

    if matches!(
        family,
        "spawn_agent" | "wait_agent" | "close_agent" | "multi_agent_v1"
    ) || tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "spawn_agent" | "wait_agent" | "close_agent" | "multi_agent_v1"
        )
    }) {
        return CommandAttemptRole::Orchestration;
    }

    match family {
        "pytest" | "vitest" | "jest" => CommandAttemptRole::Test,
        "check" => CommandAttemptRole::Compile,
        "build" => CommandAttemptRole::Build,
        "lint" | "clippy" => CommandAttemptRole::Lint,
        "fmt" => CommandAttemptRole::FormatCheck,
        "test" => CommandAttemptRole::Test,
        _ => CommandAttemptRole::Unknown,
    }
}

fn verifier_kind(attempt: &CommandAttempt) -> Option<VerifierKind> {
    let tokens = normalized_command_tokens(&attempt.raw_command);
    match attempt.role {
        CommandAttemptRole::Compile => Some(VerifierKind::CargoCheck),
        CommandAttemptRole::Lint => Some(VerifierKind::CargoClippy),
        CommandAttemptRole::FormatCheck => Some(VerifierKind::CargoFmt),
        CommandAttemptRole::Build => Some(if attempt.family == "cargo" {
            VerifierKind::CargoBuild
        } else {
            VerifierKind::GenericBuild
        }),
        CommandAttemptRole::Replay => Some(VerifierKind::Replay),
        CommandAttemptRole::Test => Some(match attempt.family.as_str() {
            "cargo" => VerifierKind::CargoTest,
            "npm" => VerifierKind::NpmTest,
            "pnpm" => VerifierKind::PnpmTest,
            "pytest" => VerifierKind::Pytest,
            "vitest" => VerifierKind::Vitest,
            "jest" => VerifierKind::Jest,
            "bun" => VerifierKind::BunTest,
            "deno" => VerifierKind::DenoTest,
            "npx" if tokens.iter().any(|token| token == "vitest") => VerifierKind::Vitest,
            "npx" if tokens.iter().any(|token| token == "jest") => VerifierKind::Jest,
            "python" | "uv" if tokens.iter().any(|token| token == "pytest") => VerifierKind::Pytest,
            _ => VerifierKind::GenericTest,
        }),
        _ => None,
    }
}

fn verification_scope(attempt: &CommandAttempt) -> VerificationScope {
    let tokens = normalized_command_tokens(&attempt.raw_command);
    let mut paths = attempt.paths.clone();
    let mut tests = Vec::new();

    match attempt.family.as_str() {
        "cargo" if attempt.role == CommandAttemptRole::Test => {
            tests.extend(cargo_test_targets(&tokens));
        }
        "pytest" => {
            let (more_paths, more_tests) =
                pytest_targets(tokens.iter().skip(1).map(String::as_str));
            paths.extend(more_paths);
            tests.extend(more_tests);
        }
        "python" | "uv"
            if attempt.role == CommandAttemptRole::Test
                && tokens.iter().any(|token| token == "pytest") =>
        {
            let pytest_index = tokens
                .iter()
                .position(|token| token == "pytest")
                .unwrap_or(tokens.len());
            let (more_paths, more_tests) =
                pytest_targets(tokens.iter().skip(pytest_index + 1).map(String::as_str));
            paths.extend(more_paths);
            tests.extend(more_tests);
        }
        "npm" | "pnpm" | "yarn" | "npx" | "vitest" | "jest" | "bun" | "deno"
            if attempt.role == CommandAttemptRole::Test =>
        {
            let (more_paths, more_tests) = js_test_targets(&tokens, &attempt.family);
            paths.extend(more_paths);
            tests.extend(more_tests);
        }
        _ if attempt.role == CommandAttemptRole::Test => {
            tests.extend(generic_test_targets(&tokens, &attempt.family));
        }
        _ => {}
    }

    paths.sort();
    paths.dedup();
    tests.sort();
    tests.dedup();

    VerificationScope {
        raw: attempt.raw_command.clone(),
        broad: paths.is_empty() && tests.is_empty(),
        paths,
        tests,
    }
}

fn exercise_state(
    attempt: &CommandAttempt,
    execution_evidence: Option<TestExecutionEvidence>,
    output: &str,
) -> ExerciseState {
    match attempt.role {
        CommandAttemptRole::Compile
        | CommandAttemptRole::Lint
        | CommandAttemptRole::FormatCheck
        | CommandAttemptRole::Build
        | CommandAttemptRole::Replay => match attempt.outcome {
            AttemptOutcome::Clean | AttemptOutcome::Failed => ExerciseState::TargetExercised,
            AttemptOutcome::Unknown => ExerciseState::Unknown,
        },
        CommandAttemptRole::Test => {
            if test_attempt_uses_nonexecuting_flags(attempt) {
                return match attempt.outcome {
                    AttemptOutcome::Failed if output_contains_compile_blocker(output) => {
                        ExerciseState::BlockedBeforeTarget
                    }
                    _ => ExerciseState::Unknown,
                };
            }

            if execution_evidence.is_some_and(TestExecutionEvidence::target_exercised) {
                ExerciseState::TargetExercised
            } else if attempt.outcome == AttemptOutcome::Failed
                && output_contains_compile_blocker(output)
            {
                ExerciseState::BlockedBeforeTarget
            } else {
                ExerciseState::Unknown
            }
        }
        _ => ExerciseState::Unknown,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TestExecutionEvidence {
    Executed { failed: bool },
    Zero,
    Unknown,
    Contradictory { failed: bool },
}

impl TestExecutionEvidence {
    fn target_exercised(self) -> bool {
        matches!(self, Self::Executed { .. })
    }

    fn reports_failure(self) -> bool {
        matches!(
            self,
            Self::Executed { failed: true } | Self::Contradictory { failed: true }
        )
    }
}

fn verification_outcome(
    command_outcome: AttemptOutcome,
    execution_evidence: Option<TestExecutionEvidence>,
) -> AttemptOutcome {
    if execution_evidence.is_some_and(TestExecutionEvidence::reports_failure) {
        AttemptOutcome::Failed
    } else {
        command_outcome
    }
}

fn test_execution_evidence(
    verifier: VerifierKind,
    output: &str,
    target_scope: &VerificationScope,
) -> TestExecutionEvidence {
    let summary = match verifier {
        VerifierKind::CargoTest => cargo_test_execution_evidence(output),
        VerifierKind::Pytest => pytest_execution_evidence(output),
        VerifierKind::Vitest
        | VerifierKind::Jest
        | VerifierKind::BunTest
        | VerifierKind::DenoTest
        | VerifierKind::NpmTest
        | VerifierKind::PnpmTest => javascript_test_execution_evidence(output),
        VerifierKind::GenericTest => generic_test_execution_evidence(output),
        VerifierKind::CargoCheck
        | VerifierKind::CargoClippy
        | VerifierKind::CargoFmt
        | VerifierKind::CargoBuild
        | VerifierKind::Replay
        | VerifierKind::GenericBuild => TestExecutionEvidence::Unknown,
    };

    match per_test_execution_failure(verifier, output, target_scope) {
        Some(per_test_failed) => TestExecutionEvidence::Executed {
            failed: per_test_failed || summary.reports_failure(),
        },
        None => summary,
    }
}

fn generic_test_execution_evidence(output: &str) -> TestExecutionEvidence {
    let mut saw_zero = false;
    let mut saw_positive = false;
    let mut saw_failed = false;

    for evidence in [
        cargo_test_execution_evidence(output),
        pytest_execution_evidence(output),
        javascript_test_execution_evidence(output),
    ] {
        match evidence {
            TestExecutionEvidence::Executed { failed } => {
                saw_positive = true;
                saw_failed |= failed;
            }
            TestExecutionEvidence::Zero => saw_zero = true,
            TestExecutionEvidence::Contradictory { failed } => {
                return TestExecutionEvidence::Contradictory {
                    failed: saw_failed || failed,
                };
            }
            TestExecutionEvidence::Unknown => {}
        }
    }

    combined_summary_evidence(saw_zero || saw_positive, saw_zero, saw_positive, saw_failed)
}

fn cargo_test_execution_evidence(output: &str) -> TestExecutionEvidence {
    let mut pending_running = None;
    let mut saw_count = false;
    let mut executed = 0usize;
    let mut failed = false;

    for line in normalized_output_lines(output) {
        if let Some(count) = cargo_running_count(&line) {
            if let Some(previous) = pending_running.replace(count) {
                executed = executed.saturating_add(previous);
            }
            saw_count = true;
            continue;
        }

        if !line.starts_with("test result:") {
            continue;
        }

        let Some(summary) = cargo_test_result_summary(&line) else {
            return TestExecutionEvidence::Unknown;
        };
        saw_count = true;
        if let Some(running) = pending_running.take() {
            if running != summary.reported_total {
                return TestExecutionEvidence::Contradictory {
                    failed: summary.failed,
                };
            }
        }
        executed = executed.saturating_add(summary.executed);
        failed |= summary.failed;
    }

    if let Some(running) = pending_running {
        executed = executed.saturating_add(running);
    }

    execution_evidence_from_count(saw_count, executed, failed)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CargoTestResultSummary {
    reported_total: usize,
    executed: usize,
    failed: bool,
}

fn cargo_running_count(line: &str) -> Option<usize> {
    let mut tokens = line.split_whitespace();
    if tokens.next()? != "running" {
        return None;
    }
    let count = tokens.next()?.parse::<usize>().ok()?;
    match tokens.next()? {
        "test" | "tests" if tokens.next().is_none() => Some(count),
        _ => None,
    }
}

fn cargo_test_result_summary(line: &str) -> Option<CargoTestResultSummary> {
    let body = line.strip_prefix("test result:")?.trim();
    let passed = count_before_status(body, "passed")?;
    let failed = count_before_status(body, "failed")?;
    let ignored = count_before_status(body, "ignored").unwrap_or(0);
    let measured = count_before_status(body, "measured").unwrap_or(0);
    Some(CargoTestResultSummary {
        reported_total: passed
            .saturating_add(failed)
            .saturating_add(ignored)
            .saturating_add(measured),
        executed: passed.saturating_add(failed).saturating_add(measured),
        failed: failed > 0,
    })
}

fn pytest_execution_evidence(output: &str) -> TestExecutionEvidence {
    let mut saw_summary = false;
    let mut saw_zero = false;
    let mut saw_positive = false;
    let mut saw_failed = false;

    for line in normalized_output_lines(output) {
        if line == "collected 0 item" || line == "collected 0 items" {
            saw_summary = true;
            saw_zero = true;
            continue;
        }

        let Some(summary) = pytest_summary_body(&line) else {
            continue;
        };
        if summary.starts_with("no tests ran") {
            saw_summary = true;
            saw_zero = true;
            continue;
        }

        let passed = count_before_status(summary, "passed");
        let failed = count_before_status(summary, "failed");
        let xfailed = count_before_status(summary, "xfailed");
        let xpassed = count_before_status(summary, "xpassed");
        let skipped = count_before_status(summary, "skipped");
        if [passed, failed, xfailed, xpassed, skipped]
            .iter()
            .all(Option::is_none)
        {
            continue;
        }

        saw_summary = true;
        let executed = [passed, failed, xfailed, xpassed]
            .into_iter()
            .flatten()
            .sum::<usize>();
        if executed > 0 {
            saw_positive = true;
            saw_failed |= failed.unwrap_or(0) > 0;
        } else {
            saw_zero = true;
        }
    }

    combined_summary_evidence(saw_summary, saw_zero, saw_positive, saw_failed)
}

fn pytest_summary_body(line: &str) -> Option<&str> {
    let summary = line.trim_matches('=').trim();
    (summary.contains(" in ")
        && (summary.starts_with("no tests ran")
            || summary.chars().next().is_some_and(|ch| ch.is_ascii_digit())))
    .then_some(summary)
}

fn javascript_test_execution_evidence(output: &str) -> TestExecutionEvidence {
    let mut saw_zero = false;
    let mut saw_positive = false;
    let mut saw_failed = false;

    for line in normalized_output_lines(output) {
        let evidence = if javascript_no_tests_summary(&line) {
            Some(TestExecutionEvidence::Zero)
        } else if line.starts_with("tests:") || line.starts_with("tests ") {
            javascript_count_summary(&line)
        } else if let Some(count) = bun_ran_count(&line).or_else(|| deno_running_count(&line)) {
            Some(execution_evidence_from_count(true, count, false))
        } else {
            deno_summary_evidence(&line)
        };

        match evidence {
            Some(TestExecutionEvidence::Executed { failed }) => {
                saw_positive = true;
                saw_failed |= failed;
            }
            Some(TestExecutionEvidence::Zero) => saw_zero = true,
            Some(TestExecutionEvidence::Contradictory { failed }) => {
                return TestExecutionEvidence::Contradictory {
                    failed: saw_failed || failed,
                };
            }
            Some(TestExecutionEvidence::Unknown) | None => {}
        }
    }

    combined_summary_evidence(saw_zero || saw_positive, saw_zero, saw_positive, saw_failed)
}

fn javascript_no_tests_summary(line: &str) -> bool {
    line == "no tests found"
        || line.starts_with("no tests found, exiting with code ")
        || line == "no test files found"
        || line.starts_with("no test files found, exiting with code ")
}

fn javascript_count_summary(line: &str) -> Option<TestExecutionEvidence> {
    let passed = count_before_status(line, "passed");
    let failed = count_before_status(line, "failed");
    let skipped = count_before_status(line, "skipped");
    let todo = count_before_status(line, "todo");
    if [passed, failed, skipped, todo].iter().all(Option::is_none) {
        return None;
    }

    let total = if line.starts_with("tests:") {
        count_before_status(line, "total")?
    } else {
        let open = line.rfind('(')?;
        let close = line[open + 1..].find(')')? + open + 1;
        line[open + 1..close].trim().parse::<usize>().ok()?
    };
    let passed = passed.unwrap_or(0);
    let failed = failed.unwrap_or(0);
    let skipped = skipped.unwrap_or(0);
    let todo = todo.unwrap_or(0);
    let executed = passed.saturating_add(failed);
    let accounted = executed.saturating_add(skipped).saturating_add(todo);

    if accounted > total {
        Some(TestExecutionEvidence::Contradictory { failed: failed > 0 })
    } else if executed > 0 {
        Some(TestExecutionEvidence::Executed { failed: failed > 0 })
    } else if total == 0 || accounted == total {
        Some(TestExecutionEvidence::Zero)
    } else {
        Some(TestExecutionEvidence::Unknown)
    }
}

fn bun_ran_count(line: &str) -> Option<usize> {
    let mut tokens = line.split_whitespace();
    if tokens.next()? != "ran" {
        return None;
    }
    let count = tokens.next()?.parse::<usize>().ok()?;
    matches!(tokens.next(), Some("test") | Some("tests")).then_some(count)
}

fn deno_running_count(line: &str) -> Option<usize> {
    let mut tokens = line.split_whitespace();
    if tokens.next()? != "running" {
        return None;
    }
    let count = tokens.next()?.parse::<usize>().ok()?;
    if !matches!(tokens.next(), Some("test") | Some("tests")) {
        return None;
    }
    matches!(tokens.next(), Some("from")).then_some(count)
}

fn deno_summary_evidence(line: &str) -> Option<TestExecutionEvidence> {
    if !(line.starts_with("ok |") || line.starts_with("failed |")) {
        return None;
    }
    let passed = count_before_status(line, "passed")?;
    let failed = count_before_status(line, "failed")?;
    let executed = passed.saturating_add(failed);
    Some(execution_evidence_from_count(true, executed, failed > 0))
}

fn count_before_status(text: &str, status: &str) -> Option<usize> {
    let tokens = text
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    tokens
        .windows(2)
        .filter(|window| window[1] == status)
        .find_map(|window| window[0].parse::<usize>().ok())
}

fn normalized_output_lines(output: &str) -> impl Iterator<Item = String> + '_ {
    output
        .lines()
        .map(|line| line.trim().to_ascii_lowercase())
        .filter(|line| !line.is_empty())
}

fn framed_output_lines(output: &str) -> impl Iterator<Item = String> + '_ {
    output
        .lines()
        .map(|line| line.trim_end().to_ascii_lowercase())
        .filter(|line| !line.trim().is_empty())
}

fn combined_summary_evidence(
    saw_summary: bool,
    saw_zero: bool,
    saw_positive: bool,
    saw_failed: bool,
) -> TestExecutionEvidence {
    match (saw_summary, saw_zero, saw_positive) {
        (_, true, true) => TestExecutionEvidence::Contradictory { failed: saw_failed },
        (_, false, true) => TestExecutionEvidence::Executed { failed: saw_failed },
        (true, true, false) => TestExecutionEvidence::Zero,
        _ => TestExecutionEvidence::Unknown,
    }
}

fn execution_evidence_from_count(
    saw_count: bool,
    executed: usize,
    failed: bool,
) -> TestExecutionEvidence {
    if !saw_count {
        TestExecutionEvidence::Unknown
    } else if executed == 0 {
        TestExecutionEvidence::Zero
    } else {
        TestExecutionEvidence::Executed { failed }
    }
}

fn per_test_execution_failure(
    verifier: VerifierKind,
    output: &str,
    target_scope: &VerificationScope,
) -> Option<bool> {
    match verifier {
        VerifierKind::CargoTest => cargo_per_test_failure(output, target_scope),
        VerifierKind::Pytest => pytest_per_test_failure(output, target_scope),
        VerifierKind::Vitest
        | VerifierKind::Jest
        | VerifierKind::BunTest
        | VerifierKind::DenoTest
        | VerifierKind::NpmTest
        | VerifierKind::PnpmTest => javascript_per_test_failure(verifier, output, target_scope),
        VerifierKind::GenericTest => [
            cargo_per_test_failure(output, target_scope),
            pytest_per_test_failure(output, target_scope),
            javascript_per_test_failure(verifier, output, target_scope),
        ]
        .into_iter()
        .flatten()
        .reduce(|left, right| left || right),
        VerifierKind::CargoCheck
        | VerifierKind::CargoClippy
        | VerifierKind::CargoFmt
        | VerifierKind::CargoBuild
        | VerifierKind::Replay
        | VerifierKind::GenericBuild => None,
    }
}

fn cargo_per_test_failure(output: &str, target_scope: &VerificationScope) -> Option<bool> {
    aggregate_per_test_records(output, target_scope, cargo_per_test_record)
}

fn cargo_per_test_record(line: &str) -> Option<(&str, bool)> {
    let line = runner_line_body(line, 0, 0)?;
    let body = line.strip_prefix("test ")?;
    let (test_name, status) = body.rsplit_once(" ... ")?;
    let failed = match status.trim() {
        "ok" => false,
        "failed" => true,
        _ => return None,
    };
    (!test_name.trim().is_empty()).then_some((test_name.trim(), failed))
}

fn pytest_per_test_failure(output: &str, target_scope: &VerificationScope) -> Option<bool> {
    aggregate_per_test_records(output, target_scope, pytest_per_test_record)
}

fn pytest_per_test_record(line: &str) -> Option<(&str, bool)> {
    let line = runner_line_body(line, 0, 0)?;
    let mut tokens = line.split_whitespace();
    let node_id = tokens.next()?;
    let file = node_id.split("::").next()?;
    if !node_id.contains("::") || !file.ends_with(".py") {
        return None;
    }
    let failed = match tokens.next()? {
        "passed" | "xfailed" | "xpassed" => false,
        "failed" => true,
        _ => return None,
    };
    let trailer = tokens.collect::<Vec<_>>().join(" ");
    (trailer.is_empty() || valid_pytest_progress_trailer(&trailer)).then_some((node_id, failed))
}

fn valid_pytest_progress_trailer(trailer: &str) -> bool {
    trailer
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .and_then(|value| value.trim().strip_suffix('%'))
        .and_then(|value| value.trim().parse::<u8>().ok())
        .is_some_and(|percent| percent <= 100)
}

fn aggregate_per_test_records(
    output: &str,
    target_scope: &VerificationScope,
    parser: fn(&str) -> Option<(&str, bool)>,
) -> Option<bool> {
    let mut saw_record = false;
    let mut saw_failed = false;
    for line in framed_output_lines(output) {
        let Some((subject, failed)) = parser(&line) else {
            continue;
        };
        if record_matches_target(&[subject], target_scope) {
            saw_record = true;
            saw_failed |= failed;
        }
    }
    saw_record.then_some(saw_failed)
}

fn javascript_per_test_failure(
    verifier: VerifierKind,
    output: &str,
    target_scope: &VerificationScope,
) -> Option<bool> {
    let allow_vitest = matches!(
        verifier,
        VerifierKind::Vitest
            | VerifierKind::NpmTest
            | VerifierKind::PnpmTest
            | VerifierKind::GenericTest
    );
    let allow_jest = matches!(
        verifier,
        VerifierKind::Jest
            | VerifierKind::NpmTest
            | VerifierKind::PnpmTest
            | VerifierKind::GenericTest
    );
    let allow_bun = matches!(
        verifier,
        VerifierKind::BunTest
            | VerifierKind::NpmTest
            | VerifierKind::PnpmTest
            | VerifierKind::GenericTest
    );
    let allow_deno = matches!(
        verifier,
        VerifierKind::DenoTest
            | VerifierKind::NpmTest
            | VerifierKind::PnpmTest
            | VerifierKind::GenericTest
    );
    let mut contexts = JavascriptTestContexts::default();
    let mut capture_boundary_active = false;
    let mut saw_record = false;
    let mut saw_failed = false;

    for line in framed_output_lines(output) {
        if javascript_capture_or_diagnostic_boundary(&line) {
            contexts = JavascriptTestContexts::default();
            capture_boundary_active = true;
            continue;
        }
        if capture_boundary_active {
            match leading_space_count(&line) {
                Some(0) => capture_boundary_active = false,
                Some(_) | None => continue,
            }
        }
        if allow_vitest {
            if let Some((subject, failed)) =
                vitest_per_test_record(&line).or_else(|| direct_js_bullet_record(&line))
            {
                if record_matches_target(&[subject], target_scope) {
                    saw_record = true;
                    saw_failed |= failed;
                }
                continue;
            }
        }
        if allow_jest {
            if let Some(path) = jest_suite_path(&line) {
                contexts.jest = Some(path.to_string());
                continue;
            }
            if let Some((test_name, failed)) = jest_per_test_record(&line) {
                let direct_path = direct_javascript_path_record(&line, test_name, target_scope);
                let contextual_path = contexts
                    .jest
                    .as_deref()
                    .is_some_and(|path| record_matches_target(&[path, test_name], target_scope));
                if direct_path || contextual_path {
                    saw_record = true;
                    saw_failed |= failed;
                }
                continue;
            }
        }
        if allow_bun {
            if let Some(path) = bun_file_context(&line) {
                contexts.bun = Some(path.to_string());
                continue;
            }
            if let Some((test_name, failed)) = bun_per_test_record(&line) {
                let direct_path = direct_javascript_path_record(&line, test_name, target_scope);
                let contextual_path = contexts
                    .bun
                    .as_deref()
                    .is_some_and(|path| record_matches_target(&[path, test_name], target_scope));
                if direct_path || contextual_path {
                    saw_record = true;
                    saw_failed |= failed;
                }
                continue;
            }
        }
        if allow_deno {
            if let Some((path, count)) = deno_running_context(&line) {
                contexts.deno = Some((path.to_string(), count));
                continue;
            }
            if let Some((test_name, failed)) = deno_per_test_record(&line) {
                let contextual_path = contexts.deno.as_ref().is_some_and(|(path, count)| {
                    *count > 0 && record_matches_target(&[path, test_name], target_scope)
                });
                if contextual_path {
                    saw_record = true;
                    saw_failed |= failed;
                }
                continue;
            }
        }
        if verifier == VerifierKind::GenericTest {
            if let Some((test_name, failed)) = runner_line_body(&line, 0, 0)
                .and_then(|body| status_prefixed_record(body, "--- pass: ", "--- fail: "))
            {
                if record_matches_target(&[test_name], target_scope) {
                    saw_record = true;
                    saw_failed |= failed;
                }
            }
        }
    }

    saw_record.then_some(saw_failed)
}

#[derive(Default)]
struct JavascriptTestContexts {
    jest: Option<String>,
    bun: Option<String>,
    deno: Option<(String, usize)>,
}

fn vitest_per_test_record(line: &str) -> Option<(&str, bool)> {
    let line = runner_line_body(line, 0, 2)?;
    let (body, failed) = status_prefixed_record(line, "pass ", "fail ")?;
    let path = body.split(" > ").next()?;
    (body.contains(" > ") && is_javascript_test_path(path)).then_some((body, failed))
}

fn direct_js_bullet_record(line: &str) -> Option<(&str, bool)> {
    let line = runner_line_body(line, 0, 0)?;
    let (body, failed) = status_prefixed_record(line, "✓ ", "✕ ")
        .or_else(|| status_prefixed_record(line, "✓ ", "× "))?;
    let path = body.split_whitespace().next()?;
    is_javascript_test_path(path).then_some((body, failed))
}

fn jest_suite_path(line: &str) -> Option<&str> {
    let line = runner_line_body(line, 0, 2)?;
    let body = line
        .strip_prefix("pass ")
        .or_else(|| line.strip_prefix("fail "))?
        .trim_start();
    if body.contains(" > ") {
        return None;
    }
    let path = body.split_whitespace().next()?;
    line_looks_like_test_path(path).then_some(path)
}

fn jest_per_test_record(line: &str) -> Option<(&str, bool)> {
    let indent = leading_space_count(line)?;
    if indent != 0 && (indent < 2 || !indent.is_multiple_of(2)) {
        return None;
    }
    let line = &line[indent..];
    let (test_name, failed) = status_prefixed_record(line, "✓ ", "✕ ")
        .or_else(|| status_prefixed_record(line, "✓ ", "× "))?;
    if indent == 0
        && !test_name
            .split_whitespace()
            .next()
            .is_some_and(is_javascript_test_path)
    {
        return None;
    }
    Some((test_name, failed))
}

fn bun_file_context(line: &str) -> Option<&str> {
    let line = runner_line_body(line, 0, 0)?;
    let path = line.strip_suffix(':')?.trim();
    is_javascript_test_path(path).then_some(path)
}

fn bun_per_test_record(line: &str) -> Option<(&str, bool)> {
    let line = runner_line_body(line, 0, 0)?;
    status_prefixed_record(line, "(pass) ", "(fail) ")
}

fn deno_running_context(line: &str) -> Option<(&str, usize)> {
    let line = runner_line_body(line, 0, 0)?;
    let mut tokens = line.split_whitespace();
    if tokens.next()? != "running" {
        return None;
    }
    let count = tokens.next()?.parse::<usize>().ok()?;
    if !matches!(tokens.next(), Some("test") | Some("tests")) || tokens.next()? != "from" {
        return None;
    }
    let path = tokens.next()?;
    (tokens.next().is_none() && is_javascript_test_path(path)).then_some((path, count))
}

fn deno_per_test_record(line: &str) -> Option<(&str, bool)> {
    let line = runner_line_body(line, 0, 0)?;
    let (test_name, status) = line.rsplit_once(" ... ")?;
    let failed = exact_status_with_optional_duration(status)?;
    (!test_name.trim().is_empty()).then_some((test_name.trim(), failed))
}

fn exact_status_with_optional_duration(status: &str) -> Option<bool> {
    for (name, failed) in [("ok", false), ("failed", true)] {
        if status == name {
            return Some(failed);
        }
        let Some(duration) = status.strip_prefix(name).map(str::trim) else {
            continue;
        };
        if duration
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .is_some_and(|value| !value.trim().is_empty())
        {
            return Some(failed);
        }
    }
    None
}

fn javascript_capture_or_diagnostic_boundary(line: &str) -> bool {
    let Some(indent) = leading_space_count(line) else {
        return true;
    };
    let body = &line[indent..];
    let captured_output_header = [
        "console.log",
        "console.error",
        "console.warn",
        "console.info",
        "console.debug",
        "captured stdout",
        "captured stderr",
        "stdout |",
        "stderr |",
    ]
    .iter()
    .any(|prefix| framed_prefix(body, prefix));
    let diagnostic_header = indent > 0
        && (body.starts_with("at ")
            || body.starts_with("● console")
            || body.starts_with("diagnostic:")
            || body.starts_with("error:")
            || body.starts_with("note:")
            || body.starts_with("source:")
            || body.starts_with("stack:")
            || body.starts_with("warning:"));
    let indented_status_payload = (indent > 2
        && (body.starts_with("pass ") || body.starts_with("fail ")))
        || (indent > 0 && (body.starts_with("(pass) ") || body.starts_with("(fail) ")));

    captured_output_header || diagnostic_header || indented_status_payload
}

fn framed_prefix(body: &str, prefix: &str) -> bool {
    body == prefix
        || body.strip_prefix(prefix).is_some_and(|suffix| {
            suffix.starts_with(' ') || suffix.starts_with(':') || suffix.starts_with('(')
        })
}

fn runner_line_body(line: &str, min_indent: usize, max_indent: usize) -> Option<&str> {
    let indent = leading_space_count(line)?;
    (min_indent..=max_indent)
        .contains(&indent)
        .then_some(&line[indent..])
}

fn leading_space_count(line: &str) -> Option<usize> {
    let count = line.bytes().take_while(|byte| *byte == b' ').count();
    (!line[count..]
        .chars()
        .next()
        .is_some_and(char::is_whitespace))
    .then_some(count)
}

fn status_prefixed_record<'a>(
    line: &'a str,
    passed_prefix: &str,
    failed_prefix: &str,
) -> Option<(&'a str, bool)> {
    let (body, failed) = if let Some(body) = line.strip_prefix(passed_prefix) {
        (body, false)
    } else if let Some(body) = line.strip_prefix(failed_prefix) {
        (body, true)
    } else {
        return None;
    };
    let body = body.trim();
    (!body.is_empty()).then_some((body, failed))
}

fn direct_javascript_path_record(
    line: &str,
    test_name: &str,
    target_scope: &VerificationScope,
) -> bool {
    runner_line_body(line, 0, 0).is_some()
        && test_name
            .split_whitespace()
            .next()
            .is_some_and(is_javascript_test_path)
        && record_matches_target(&[test_name], target_scope)
}

fn record_matches_target(subjects: &[&str], target_scope: &VerificationScope) -> bool {
    (target_scope.paths.is_empty() && target_scope.tests.is_empty())
        || subjects
            .iter()
            .any(|subject| line_matches_requested_target(subject, target_scope))
}

fn is_javascript_test_path(path: &str) -> bool {
    !path.chars().any(char::is_whitespace) && line_looks_like_test_path(path)
}

fn line_looks_like_test_path(line: &str) -> bool {
    line.contains("tests/")
        || line.contains("test/")
        || line.contains("__tests__/")
        || line.contains(".test.")
        || line.contains(".spec.")
}

fn test_attempt_uses_nonexecuting_flags(attempt: &CommandAttempt) -> bool {
    let tokens = normalized_command_tokens(&attempt.raw_command);
    match attempt.family.as_str() {
        "cargo" => tokens.iter().any(|token| token == "--no-run"),
        "pytest" => pytest_collect_only_requested(&tokens),
        "python" | "uv" if tokens.iter().any(|token| token == "pytest") => {
            pytest_collect_only_requested(&tokens)
        }
        _ => false,
    }
}

fn pytest_collect_only_requested(tokens: &[String]) -> bool {
    tokens
        .iter()
        .any(|token| token == "--collect-only" || token == "--co")
}

fn attempt_output_text(attempt: &CommandAttempt, rows: &[CompactionRow]) -> String {
    let rows_by_key = rows
        .iter()
        .map(|row| (row_key(row), row))
        .collect::<BTreeMap<_, _>>();

    attempt
        .output_rows
        .iter()
        .filter_map(|row| {
            rows_by_key
                .get(&row_ref_key(row))
                .map(|row| row.text.as_str())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn output_contains_compile_blocker(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    [
        "could not compile",
        "failed to compile",
        "compilation failed",
        "error[",
        "cannot find",
        "unresolved import",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn line_matches_requested_target(line: &str, target_scope: &VerificationScope) -> bool {
    target_scope
        .tests
        .iter()
        .any(|test| line.contains(&test.to_ascii_lowercase()))
        || target_scope
            .paths
            .iter()
            .any(|path| line.contains(&path.to_ascii_lowercase()))
}

fn cargo_role(tokens: &[String]) -> Option<CommandAttemptRole> {
    let args = tokens_after_family(tokens, "cargo")?;
    let subcommand = next_positional_token(
        args,
        &[
            "-p",
            "--package",
            "--manifest-path",
            "--config",
            "-Z",
            "--target",
            "--target-dir",
            "--message-format",
            "--color",
            "--jobs",
        ],
    )?;

    Some(match subcommand {
        "check" => CommandAttemptRole::Compile,
        "test" => CommandAttemptRole::Test,
        "clippy" => CommandAttemptRole::Lint,
        "build" => CommandAttemptRole::Build,
        "fmt" | "format" => {
            if tokens.iter().any(|token| token == "--check") {
                CommandAttemptRole::FormatCheck
            } else {
                CommandAttemptRole::FormatWrite
            }
        }
        "add" | "update" | "remove" | "rm" => CommandAttemptRole::DependencyMutation,
        _ => return None,
    })
}

fn npm_like_role(tokens: &[String], family: &str) -> Option<CommandAttemptRole> {
    let args = tokens_after_family(tokens, family)?;
    let subcommand = next_positional_token(
        args,
        &[
            "-C",
            "--prefix",
            "--dir",
            "-w",
            "--workspace",
            "--filter",
            "-F",
        ],
    )?;

    let command = if subcommand == "run" || subcommand == "exec" || subcommand == "dlx" {
        let subcommand_index = args.iter().position(|token| token == subcommand)?;
        next_positional_token(&args[subcommand_index + 1..], &[])?
    } else {
        subcommand
    };

    Some(match command {
        "test" | "vitest" | "jest" | "lint" if subcommand == "test" => CommandAttemptRole::Test,
        "test" | "vitest" | "jest" => CommandAttemptRole::Test,
        "lint" => CommandAttemptRole::Lint,
        "typecheck" => CommandAttemptRole::Build,
        "install" | "add" | "i" | "ci" | "up" | "update" => CommandAttemptRole::DependencyMutation,
        "build" => CommandAttemptRole::Build,
        "fmt" | "format" => CommandAttemptRole::FormatCheck,
        _ => return None,
    })
}

fn shell_wrapped_test_role(tokens: &[String]) -> Option<CommandAttemptRole> {
    for window in tokens.windows(2) {
        if matches!(
            window,
            [first, second]
                if (first == "go"
                    || first == "swift"
                    || first == "mvn"
                    || first == "gradle"
                    || first == "make"
                    || first == "just"
                    || first == "bun"
                    || first == "deno")
                    && second == "test"
        ) {
            return Some(CommandAttemptRole::Test);
        }
    }

    if tokens.len() >= 3 && tokens[0] == "python" && tokens[1] == "-m" && tokens[2] == "pytest" {
        return Some(CommandAttemptRole::Test);
    }

    if tokens.len() >= 3 && tokens[0] == "uv" && tokens[1] == "run" && tokens[2] == "pytest" {
        return Some(CommandAttemptRole::Test);
    }

    None
}

fn git_role(tokens: &[String]) -> Option<CommandAttemptRole> {
    let args = tokens_after_family(tokens, "git")?;
    let subcommand = next_positional_token(
        args,
        &[
            "-C",
            "-c",
            "--git-dir",
            "--work-tree",
            "--namespace",
            "--exec-path",
            "--config-env",
        ],
    )?;

    Some(match subcommand {
        "status" | "diff" | "show" | "log" => CommandAttemptRole::VcsInspection,
        _ => return None,
    })
}

fn cargo_test_targets(tokens: &[String]) -> Vec<String> {
    let Some(args) = tokens_after_family(tokens, "cargo") else {
        return Vec::new();
    };

    let Some(test_index) = args.iter().position(|token| token == "test") else {
        return Vec::new();
    };

    positional_targets(
        &args[test_index + 1..],
        &[
            "-p",
            "--package",
            "--manifest-path",
            "--features",
            "--target",
            "--jobs",
            "--color",
            "--message-format",
        ],
    )
}

fn generic_test_targets(tokens: &[String], family: &str) -> Vec<String> {
    let Some(args) = tokens_after_family(tokens, family) else {
        return Vec::new();
    };
    let args = strip_generic_runner_verbs(args);
    positional_targets(args, &["-k", "-m", "-t", "--filter", "--grep", "--project"])
}

fn js_test_targets(tokens: &[String], family: &str) -> (Vec<String>, Vec<String>) {
    let Some(args) = tokens_after_family(tokens, family) else {
        return (Vec::new(), Vec::new());
    };

    let runner_args = strip_js_runner_verbs(args);
    let mut paths = Vec::new();
    let mut tests = Vec::new();
    let mut index = 0;

    while index < runner_args.len() {
        let token = runner_args[index].as_str();
        if token == "--" {
            index += 1;
            continue;
        }

        if token.starts_with('-') {
            if let Some((value, consumed)) = js_filter_value(runner_args, index) {
                tests.push(value);
                index += consumed;
                continue;
            }

            index += if js_option_takes_value(token) && !token.contains('=') {
                2
            } else {
                1
            };
            continue;
        }

        if js_token_looks_like_path(token) {
            paths.push(token.to_string());
        } else {
            tests.push(token.to_string());
        }
        index += 1;
    }

    (paths, tests)
}

fn pytest_targets<'a>(tokens: impl Iterator<Item = &'a str>) -> (Vec<String>, Vec<String>) {
    let mut paths = Vec::new();
    let mut tests = Vec::new();

    for token in tokens {
        if token.starts_with('-') {
            continue;
        }
        if let Some((path, test)) = token.split_once("::") {
            paths.push(path.to_string());
            tests.push(token.to_string());
            if !test.is_empty() {
                tests.push(test.to_string());
            }
            continue;
        }
        if token.contains('/') || token.ends_with(".py") {
            paths.push(token.to_string());
        } else {
            tests.push(token.to_string());
        }
    }

    (paths, tests)
}

fn positional_targets(tokens: &[String], options_with_values: &[&str]) -> Vec<String> {
    let mut targets = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index].as_str();
        if token == "--" {
            break;
        }
        if token.starts_with('-') {
            index += 1;
            if !token.contains('=') && options_with_values.contains(&token) {
                index += 1;
            }
            continue;
        }
        targets.push(token.to_string());
        index += 1;
    }
    targets
}

fn strip_js_runner_verbs(mut tokens: &[String]) -> &[String] {
    loop {
        let Some(index) = first_target_token_index(tokens) else {
            return tokens;
        };

        match tokens[index].as_str() {
            "test" | "run" | "exec" | "dlx" | "vitest" | "bun" | "deno" => {
                tokens = &tokens[index + 1..];
            }
            _ => return tokens,
        }
    }
}

fn strip_generic_runner_verbs(tokens: &[String]) -> &[String] {
    let Some(index) = first_target_token_index(tokens) else {
        return tokens;
    };

    match tokens[index].as_str() {
        "test" | "run" | "exec" | "dlx" => &tokens[index + 1..],
        _ => tokens,
    }
}

fn first_target_token_index(tokens: &[String]) -> Option<usize> {
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index].as_str();
        if token == "--" {
            return Some(index + 1).filter(|next| *next < tokens.len());
        }
        if token.starts_with('-') {
            index += if js_option_takes_value(token) && !token.contains('=') {
                2
            } else {
                1
            };
            continue;
        }
        return Some(index);
    }
    None
}

fn js_filter_value(tokens: &[String], index: usize) -> Option<(String, usize)> {
    let token = tokens.get(index)?.as_str();
    for option in ["-t", "--grep", "--filter", "--testnamepattern"] {
        if token == option {
            return tokens.get(index + 1).cloned().map(|value| (value, 2));
        }
        if let Some(value) = token.strip_prefix(&format!("{option}=")) {
            return Some((value.to_string(), 1));
        }
    }
    None
}

fn js_option_takes_value(token: &str) -> bool {
    matches!(
        token,
        "-C" | "--prefix"
            | "--dir"
            | "-w"
            | "--workspace"
            | "--filter"
            | "-F"
            | "-c"
            | "--config"
            | "--reporter"
            | "--project"
    )
}

fn js_token_looks_like_path(token: &str) -> bool {
    token.contains('/')
        || token.contains('\\')
        || [".js", ".jsx", ".mjs", ".cjs", ".ts", ".tsx", ".mts", ".cts"]
            .iter()
            .any(|suffix| token.ends_with(suffix))
}

fn normalized_command_tokens(command: &str) -> Vec<String> {
    command
        .split(['\n', ';', '|', '&'])
        .next()
        .unwrap_or(command)
        .split_whitespace()
        .map(|token| {
            token
                .trim_matches(|ch: char| matches!(ch, '(' | ')' | '"' | '\''))
                .to_ascii_lowercase()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn tokens_after_family<'a>(tokens: &'a [String], family: &str) -> Option<&'a [String]> {
    let family_index = tokens
        .iter()
        .position(|token| token == family && !token.contains('=') && !token.is_empty())?;
    Some(&tokens[family_index + 1..])
}

fn next_positional_token<'a>(
    tokens: &'a [String],
    options_with_values: &[&str],
) -> Option<&'a str> {
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens[index].as_str();
        if token == "--" {
            return tokens.get(index + 1).map(String::as_str);
        }
        if token.starts_with('-') {
            index += 1;
            if !token.contains('=') && options_with_values.contains(&token) {
                index += 1;
            }
            continue;
        }
        return Some(token);
    }
    None
}

fn row_starts_new_phase(row: &CompactionRow) -> bool {
    matches!(
        row.kind,
        CompactionKind::UserMessage
            | CompactionKind::AssistantMessage
            | CompactionKind::DeveloperMessage
            | CompactionKind::SystemMessage
    ) && row_text_is_focusable(row)
}

fn row_text_is_focusable(row: &CompactionRow) -> bool {
    row.text.len() <= 2_000
        && !row.text.trim().is_empty()
        && !row.text.contains("AGENTS.md instructions")
        && !row.text.contains("<skill>")
        && !row.text.contains("Available skills")
        && row.text != "[encrypted_reasoning]"
}

fn row_looks_like_attempt_output(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed
        .lines()
        .next()
        .is_some_and(|line| line.starts_with("Exit code: "))
        || trimmed
            .get(..6)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("error:"))
}

fn row_key(row: &CompactionRow) -> (Utf8PathBuf, usize, usize) {
    (row.source_file.clone(), row.event_index, row.row_ordinal)
}

fn row_ref_key(row: &RowRef) -> (Utf8PathBuf, usize, usize) {
    (row.source_file.clone(), row.event_index, row.row_ordinal)
}

fn tool_name(row: &CompactionRow) -> String {
    row.dedupe_identity
        .as_deref()
        .and_then(|identity| serde_json::from_str::<Value>(identity).ok())
        .and_then(|value| {
            value
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| "tool_call".to_string())
}

fn command_family(command: &str) -> Option<String> {
    command
        .split(['\n', ';', '|', '&'])
        .flat_map(str::split_whitespace)
        .find(|token| !token.contains('=') && !token.is_empty())
        .map(|token| {
            token
                .trim_matches(|ch: char| matches!(ch, '(' | ')' | '"' | '\''))
                .to_string()
        })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use agent_session_compactor::{CompactionKind, SourceKind};
    use camino::Utf8PathBuf;

    use super::{
        build_command_attempts, build_verification_attempts, AttemptOutcome, CommandAttemptRole,
        ExerciseState, VerificationScope, VerifierKind,
    };
    use crate::checkpoint::EvidenceRef;
    use crate::context::CommandObservation;

    use super::CompactionRow;

    #[test]
    fn checkpoints_pair_command_rows_and_parse_exit_codes_until_phase_boundary() {
        let rows = vec![
            tool_call(
                0,
                "functions.shell_command",
                "cargo check -p agent-drift-analyzer",
            ),
            tool_output(
                1,
                "Exit code: 1\nerror: could not compile `agent-drift-analyzer`",
            ),
            assistant_message(2, "Switching to explanation phase."),
            tool_output(3, "Exit code: 0"),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].role, CommandAttemptRole::Compile);
        assert_eq!(attempts[0].outcome, AttemptOutcome::Failed);
        assert_eq!(attempts[0].exit_code, Some(1));
        assert_eq!(attempts[0].output_rows.len(), 1);
    }

    #[test]
    fn checkpoints_pair_concurrent_tool_outputs_by_call_id() {
        let rows = vec![
            tool_call_for_call(
                10,
                "call-10",
                "functions.shell_command",
                "cargo test target_verifier",
            ),
            tool_call_for_call(
                11,
                "call-11",
                "functions.shell_command",
                "cargo test sibling_verifier",
            ),
            tool_output_for_call(
                12,
                "call-10",
                "Exit code: 101\nerror: target verifier failed",
            ),
            tool_output_for_call(13, "call-11", "Exit code: 0\nOutput:\nsibling passed"),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].outcome, AttemptOutcome::Failed);
        assert_eq!(attempts[0].exit_code, Some(101));
        assert_eq!(attempts[0].output_rows[0].event_index, 12);
        assert_eq!(attempts[1].outcome, AttemptOutcome::Clean);
        assert_eq!(attempts[1].exit_code, Some(0));
        assert_eq!(attempts[1].output_rows[0].event_index, 13);
    }

    #[test]
    fn checkpoints_pair_adjacent_unknown_rows_only_when_they_look_like_output() {
        let rows = vec![
            tool_call(0, "functions.shell_command", "cargo test checkpoints"),
            unknown_row(
                1,
                "Exit code: 101\nerror: could not compile `agent-drift-analyzer`",
            ),
            tool_call(2, "functions.shell_command", "cargo test dead_end_thrash"),
            unknown_row(3, "Plan updated"),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        assert_eq!(attempts.len(), 2);
        assert_eq!(attempts[0].outcome, AttemptOutcome::Failed);
        assert_eq!(attempts[0].output_rows.len(), 1);
        assert_eq!(attempts[1].outcome, AttemptOutcome::Unknown);
        assert!(attempts[1].output_rows.is_empty());
    }

    #[test]
    fn checkpoints_ignore_late_embedded_exit_code_mentions_inside_output_body() {
        let rows = vec![
            tool_call(0, "functions.shell_command", "cargo test checkpoints"),
            tool_output(
                1,
                "Output:\nreplaying historical notes\nA prior run reported Exit code: 0 yesterday",
            ),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].exit_code, None);
        assert_eq!(attempts[0].outcome, AttemptOutcome::Unknown);
    }

    #[test]
    fn checkpoints_parse_exit_codes_from_later_paired_output_rows_after_change_summaries() {
        let rows = vec![
            tool_call(0, "functions.shell_command", "cargo test checkpoints"),
            tool_output(
                1,
                "Updated files:\n- crates/agent-drift-analyzer/src/checkpoint/attempt.rs",
            ),
            tool_output(
                2,
                "Exit code: 101\nerror: could not compile `agent-drift-analyzer`",
            ),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0].exit_code, Some(101));
        assert_eq!(attempts[0].outcome, AttemptOutcome::Failed);
        assert_eq!(attempts[0].output_rows.len(), 2);
    }

    #[test]
    fn checkpoints_classify_progress_specific_command_roles_deterministically() {
        let expectations = [
            (
                "cargo check -p agent-drift-analyzer",
                CommandAttemptRole::Compile,
            ),
            (
                "cargo test -p agent-drift-analyzer checkpoints",
                CommandAttemptRole::Test,
            ),
            (
                "cargo clippy -p agent-drift-analyzer",
                CommandAttemptRole::Lint,
            ),
            (
                "cargo fmt --all -- --check",
                CommandAttemptRole::FormatCheck,
            ),
            ("cargo fmt --all", CommandAttemptRole::FormatWrite),
            (
                "cargo build -p agent-drift-analyzer",
                CommandAttemptRole::Build,
            ),
            (
                "npm install typescript",
                CommandAttemptRole::DependencyMutation,
            ),
            ("npm run typecheck", CommandAttemptRole::Build),
            ("npm run lint", CommandAttemptRole::Lint),
            ("pnpm run lint", CommandAttemptRole::Lint),
            ("yarn lint", CommandAttemptRole::Lint),
            ("yarn run lint", CommandAttemptRole::Lint),
            ("npm test -- --runInBand", CommandAttemptRole::Test),
            ("npm run test -- --runInBand", CommandAttemptRole::Test),
            ("pnpm test -- --runInBand", CommandAttemptRole::Test),
            ("pnpm run test -- --runInBand", CommandAttemptRole::Test),
            (
                "pnpm exec vitest run tests/checkpoints.test.ts",
                CommandAttemptRole::Test,
            ),
            (
                "yarn test tests/checkpoints.test.ts",
                CommandAttemptRole::Test,
            ),
            (
                "yarn run test tests/checkpoints.test.ts",
                CommandAttemptRole::Test,
            ),
            (
                "pytest tests/checkpoints_test.py::test_pairing",
                CommandAttemptRole::Test,
            ),
            (
                "uv run pytest tests/checkpoints_test.py::test_pairing",
                CommandAttemptRole::Test,
            ),
            (
                "vitest run tests/checkpoints.test.ts",
                CommandAttemptRole::Test,
            ),
            (
                "npx vitest run tests/checkpoints.test.ts",
                CommandAttemptRole::Test,
            ),
            (
                "bun test tests/checkpoints.test.ts",
                CommandAttemptRole::Test,
            ),
            ("git diff --stat", CommandAttemptRole::VcsInspection),
            (
                "sed -n '1,120p' crates/agent-drift-analyzer/tests/checkpoints.rs",
                CommandAttemptRole::Read,
            ),
            (
                "apply_patch <<'PATCH'\n*** Begin Patch",
                CommandAttemptRole::Edit,
            ),
            (
                "cargo run -p agent-drift-sentinel -- replay fixture",
                CommandAttemptRole::Replay,
            ),
            ("spawn_agent", CommandAttemptRole::Orchestration),
        ];

        for (command, expected_role) in expectations {
            let rows = vec![tool_call(0, tool_name_for(command), command)];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            assert_eq!(attempts[0].role, expected_role, "{command}");
        }
    }

    #[test]
    fn checkpoints_keep_read_and_vcs_commands_with_replay_or_spawn_agent_text_out_of_special_roles()
    {
        let expectations = [
            (
                "sed -n '1,120p' docs/REPLAY.md",
                CommandAttemptRole::Read,
            ),
            (
                "rg -n 'spawn_agent' docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md",
                CommandAttemptRole::Read,
            ),
            (
                "git show HEAD:docs/REPLAY.md",
                CommandAttemptRole::VcsInspection,
            ),
        ];

        for (command, expected_role) in expectations {
            let rows = vec![tool_call(0, tool_name_for(command), command)];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            assert_eq!(attempts[0].role, expected_role, "{command}");
        }
    }

    #[test]
    fn checkpoints_extract_js_verification_scope_without_treating_runner_verbs_as_targets() {
        let rows = vec![
            tool_call(0, "functions.shell_command", "npm test"),
            tool_call(1, "functions.shell_command", "npm run test -- --runInBand"),
            tool_call(2, "functions.shell_command", "pnpm test -- --runInBand"),
            tool_call(3, "functions.shell_command", "pnpm run test -- --runInBand"),
            tool_call(
                4,
                "functions.shell_command",
                "pnpm exec vitest run tests/checkpoints.test.ts",
            ),
            tool_call(
                5,
                "functions.shell_command",
                "yarn test tests/checkpoints.test.ts",
            ),
            tool_call(
                6,
                "functions.shell_command",
                "yarn run test tests/checkpoints.test.ts",
            ),
            tool_call(
                7,
                "functions.shell_command",
                "vitest run tests/checkpoints.test.ts",
            ),
            tool_call(
                8,
                "functions.shell_command",
                "npx vitest run tests/checkpoints.test.ts",
            ),
            tool_call(
                9,
                "functions.shell_command",
                "bun test tests/checkpoints.test.ts",
            ),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        let verification = build_verification_attempts(&attempts, &rows);
        assert_eq!(verification.len(), 10);

        assert_eq!(verification[0].target_scope.paths, Vec::<String>::new());
        assert_eq!(verification[0].target_scope.tests, Vec::<String>::new());
        assert!(verification[0].target_scope.broad);

        assert_eq!(verification[1].target_scope.paths, Vec::<String>::new());
        assert_eq!(verification[1].target_scope.tests, Vec::<String>::new());
        assert!(verification[1].target_scope.broad);

        assert_eq!(verification[2].target_scope.paths, Vec::<String>::new());
        assert_eq!(verification[2].target_scope.tests, Vec::<String>::new());
        assert!(verification[2].target_scope.broad);

        assert_eq!(verification[3].target_scope.paths, Vec::<String>::new());
        assert_eq!(verification[3].target_scope.tests, Vec::<String>::new());
        assert!(verification[3].target_scope.broad);

        assert_eq!(
            verification[4].target_scope.paths,
            vec!["tests/checkpoints.test.ts".to_string()]
        );
        assert_eq!(verification[4].target_scope.tests, Vec::<String>::new());
        assert!(!verification[4].target_scope.broad);

        assert_eq!(
            verification[5].target_scope.paths,
            vec!["tests/checkpoints.test.ts".to_string()]
        );
        assert_eq!(verification[5].target_scope.tests, Vec::<String>::new());
        assert!(!verification[5].target_scope.broad);

        assert_eq!(
            verification[6].target_scope.paths,
            vec!["tests/checkpoints.test.ts".to_string()]
        );
        assert_eq!(verification[6].target_scope.tests, Vec::<String>::new());
        assert!(!verification[6].target_scope.broad);

        assert_eq!(
            verification[7].target_scope.paths,
            vec!["tests/checkpoints.test.ts".to_string()]
        );
        assert_eq!(verification[7].target_scope.tests, Vec::<String>::new());
        assert!(!verification[7].target_scope.broad);

        assert_eq!(
            verification[8].target_scope.paths,
            vec!["tests/checkpoints.test.ts".to_string()]
        );
        assert_eq!(verification[8].target_scope.tests, Vec::<String>::new());
        assert!(!verification[8].target_scope.broad);

        assert_eq!(
            verification[9].target_scope.paths,
            vec!["tests/checkpoints.test.ts".to_string()]
        );
        assert_eq!(verification[9].target_scope.tests, Vec::<String>::new());
        assert!(!verification[9].target_scope.broad);
    }

    #[test]
    fn checkpoints_strip_generic_runner_verbs_before_extracting_test_targets() {
        let rows = vec![
            tool_call(0, "functions.shell_command", "go test ./..."),
            tool_call(1, "functions.shell_command", "just test"),
            tool_call(2, "functions.shell_command", "make test"),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        let verification = build_verification_attempts(&attempts, &rows);
        assert_eq!(verification.len(), 3);

        assert_eq!(
            verification[0].target_scope.paths,
            vec!["./...".to_string()]
        );
        assert_eq!(
            verification[0].target_scope.tests,
            vec!["./...".to_string()]
        );
        assert!(!verification[0].target_scope.broad);

        assert_eq!(verification[1].target_scope.paths, Vec::<String>::new());
        assert_eq!(verification[1].target_scope.tests, Vec::<String>::new());
        assert!(verification[1].target_scope.broad);

        assert_eq!(verification[2].target_scope.paths, Vec::<String>::new());
        assert_eq!(verification[2].target_scope.tests, Vec::<String>::new());
        assert!(verification[2].target_scope.broad);
    }

    #[test]
    fn checkpoints_derive_verification_attempts_and_target_exercise_state() {
        let blocked_rows = vec![
            tool_call(0, "functions.shell_command", "cargo test checkpoints"),
            tool_output(
                1,
                "Exit code: 101\nerror: could not compile `agent-drift-analyzer`",
            ),
        ];
        let blocked_attempts =
            build_command_attempts(&blocked_rows, &command_observations(&blocked_rows));
        let blocked_verification = build_verification_attempts(&blocked_attempts, &blocked_rows);
        assert_eq!(blocked_verification.len(), 1);
        assert_eq!(blocked_verification[0].verifier, VerifierKind::CargoTest);
        assert_eq!(
            blocked_verification[0].target_scope,
            VerificationScope {
                raw: "cargo test checkpoints".to_string(),
                paths: Vec::new(),
                tests: vec!["checkpoints".to_string()],
                broad: false,
            }
        );
        assert_eq!(
            blocked_verification[0].exercise_state,
            ExerciseState::BlockedBeforeTarget
        );

        let exercised_rows = vec![
            tool_call(0, "functions.shell_command", "pytest tests/checkpoints_test.py::test_pairing"),
            tool_output(
                1,
                "Exit code: 1\nOutput:\n=================== test session starts ===================\ncollected 1 item\n\ntests/checkpoints_test.py::test_pairing FAILED",
            ),
        ];
        let exercised_attempts =
            build_command_attempts(&exercised_rows, &command_observations(&exercised_rows));
        let exercised_verification =
            build_verification_attempts(&exercised_attempts, &exercised_rows);
        assert_eq!(exercised_verification.len(), 1);
        assert_eq!(
            exercised_verification[0].exercise_state,
            ExerciseState::TargetExercised
        );
        assert_eq!(
            exercised_verification[0].target_scope.tests,
            vec![
                "test_pairing".to_string(),
                "tests/checkpoints_test.py::test_pairing".to_string()
            ]
        );
    }

    #[test]
    fn checkpoints_leave_pytest_collection_import_failures_unknown_before_target_execution() {
        let rows = vec![
            tool_call(
                0,
                "functions.shell_command",
                "pytest tests/checkpoints_test.py::test_pairing",
            ),
            tool_output(
                1,
                "=================== test session starts ===================\ncollected 0 items / 1 error\n\n==================================== ERRORS ====================================\n____________ ERROR collecting tests/checkpoints_test.py ____________\nImportError while importing test module '/repo/tests/checkpoints_test.py'.\nExit code: 2",
            ),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        let verification = build_verification_attempts(&attempts, &rows);

        assert_eq!(verification.len(), 1);
        assert_eq!(verification[0].verifier, VerifierKind::Pytest);
        assert_eq!(verification[0].exercise_state, ExerciseState::Unknown);
    }

    #[test]
    fn checkpoints_leave_cargo_test_no_run_attempts_unknown_when_targets_never_execute() {
        let rows = vec![
            tool_call(
                0,
                "functions.shell_command",
                "cargo test --test checkpoints --no-run",
            ),
            tool_output(
                1,
                "Exit code: 0\nFinished `test` profile [unoptimized + debuginfo] target(s) in 0.42s",
            ),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        let verification = build_verification_attempts(&attempts, &rows);

        assert_eq!(verification.len(), 1);
        assert_eq!(verification[0].verifier, VerifierKind::CargoTest);
        assert_eq!(verification[0].outcome, AttemptOutcome::Clean);
        assert_eq!(verification[0].exercise_state, ExerciseState::Unknown);
    }

    #[test]
    fn checkpoints_leave_pytest_collect_only_attempts_unknown_when_targets_never_execute() {
        let cases = [
            (
                "pytest --collect-only tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\n=================== test session starts ===================\ncollected 1 item\n\n<Dir repo>\n  <Module tests/checkpoints_test.py>\n    <Function test_pairing>",
            ),
            (
                "python -m pytest --co tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\n=================== test session starts ===================\ncollected 1 item\n\n<Dir repo>\n  <Module tests/checkpoints_test.py>\n    <Function test_pairing>",
            ),
            (
                "uv run pytest --collect-only tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\n=================== test session starts ===================\ncollected 1 item\n\n<Dir repo>\n  <Module tests/checkpoints_test.py>\n    <Function test_pairing>",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];

            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].verifier, VerifierKind::Pytest, "{command}");
            assert_eq!(verification[0].outcome, AttemptOutcome::Clean, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::Unknown,
                "{command}"
            );
        }
    }

    #[test]
    fn checkpoints_require_positive_execution_evidence_for_clean_test_attempts() {
        let cases = [
            (
                "cargo test zero_target -- --exact",
                "Exit code: 0\nrunning 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out",
            ),
            (
                "cargo test mentioned_target -- --exact",
                "Exit code: 0\nFinished `test` profile [unoptimized + debuginfo] target(s) in 0.42s\nmentioned_target",
            ),
            (
                "cargo test unknown_summary -- --exact",
                "Exit code: 0\nTest command completed successfully",
            ),
            (
                "pytest tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\n=================== test session starts ===================\ncollected 0 items\n\n=================== no tests ran in 0.01s ===================",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].outcome, AttemptOutcome::Clean, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::Unknown,
                "{command}"
            );
        }
    }

    #[test]
    fn checkpoints_preserve_recognized_nonzero_clean_and_failed_test_execution() {
        let cases = [
            (
                "cargo test cargo_clean -- --exact",
                "Exit code: 0\nrunning 1 test\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
                AttemptOutcome::Clean,
            ),
            (
                "pytest tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\n=================== test session starts ===================\ncollected 1 item\n\n=================== 1 passed in 0.01s ===================",
                AttemptOutcome::Clean,
            ),
            (
                "yarn test tests/checkpoints.test.ts",
                "Exit code: 0\nTests: 1 passed, 1 total",
                AttemptOutcome::Clean,
            ),
            (
                "cargo test cargo_failed -- --exact",
                "Exit code: 101\nrunning 1 test\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
                AttemptOutcome::Failed,
            ),
        ];

        for (command, output, outcome) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].outcome, outcome, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::TargetExercised,
                "{command}"
            );
        }
    }

    #[test]
    fn checkpoints_reconcile_wrapper_masked_test_failures_before_clean_proof() {
        let cases = [
            (
                "cargo test cargo_failed -- --exact",
                "Exit code: 0\nrunning 1 test\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            ),
            (
                "pytest tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\n=================== test session starts ===================\ncollected 1 item\n\n=================== 1 failed in 0.01s ===================",
            ),
            (
                "yarn test tests/checkpoints.test.ts",
                "Exit code: 0\nTests: 1 failed, 1 total",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            assert_eq!(attempts[0].outcome, AttemptOutcome::Clean, "{command}");

            let verification = build_verification_attempts(&attempts, &rows);
            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].outcome, AttemptOutcome::Failed, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::TargetExercised,
                "{command}"
            );
        }
    }

    #[test]
    fn checkpoints_preserve_failure_polarity_for_contradictory_summaries_without_exercise() {
        let cases = [
            (
                "cargo test cargo_contradictory -- --exact",
                "Exit code: 0\nrunning 0 tests\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            ),
            (
                "pytest tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\ncollected 0 items\n\n=================== 1 failed in 0.01s ===================",
            ),
            (
                "yarn test tests/checkpoints.test.ts",
                "Exit code: 0\nTests: 1 failed, 0 total",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            assert_eq!(attempts[0].outcome, AttemptOutcome::Clean, "{command}");

            let verification = build_verification_attempts(&attempts, &rows);
            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].outcome, AttemptOutcome::Failed, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::Unknown,
                "{command}"
            );
        }
    }

    #[test]
    fn checkpoints_reject_multiline_javascript_captured_output_as_per_test_execution() {
        let cases = [
            (
                "jest-zero",
                "jest tests/foo.test.ts",
                "Exit code: 0\nPASS tests/foo.test.ts\n  console.log\n    PASS subtracts (2 ms)\n    ✓ subtracts (2 ms)\nTests: 0 passed, 0 total",
                AttemptOutcome::Clean,
            ),
            (
                "jest-contradictory",
                "jest tests/foo.test.ts",
                "Exit code: 0\nFAIL tests/foo.test.ts\n  console.log\n    PASS subtracts (2 ms)\n    ✓ subtracts (2 ms)\nTests: 1 failed, 0 total",
                AttemptOutcome::Failed,
            ),
            (
                "jest-path-prefixed-zero",
                "jest tests/foo.test.ts",
                "Exit code: 0\nPASS tests/foo.test.ts\n  console.log\n  ✓ tests/foo.test.ts subtracts (2 ms)\nTests: 0 passed, 0 total",
                AttemptOutcome::Clean,
            ),
            (
                "jest-path-prefixed-contradictory",
                "jest tests/foo.test.ts",
                "Exit code: 0\nFAIL tests/foo.test.ts\n  console.log\n  ✓ tests/foo.test.ts subtracts (2 ms)\nTests: 1 failed, 0 total",
                AttemptOutcome::Failed,
            ),
            (
                "bun-zero",
                "bun test tests/foo.test.ts",
                "Exit code: 0\ntests/foo.test.ts:\n  console.log\n    (pass) math > subtracts\nRan 0 tests across 1 file",
                AttemptOutcome::Clean,
            ),
            (
                "bun-contradictory",
                "bun test tests/foo.test.ts",
                "Exit code: 0\ntests/foo.test.ts:\n  console.log\n    (pass) math > subtracts\nTests: 1 failed, 0 total",
                AttemptOutcome::Failed,
            ),
            (
                "bun-path-prefixed-zero",
                "bun test tests/foo.test.ts",
                "Exit code: 0\ntests/foo.test.ts:\n  console.log\n  (pass) tests/foo.test.ts math > subtracts\nRan 0 tests across 1 file",
                AttemptOutcome::Clean,
            ),
            (
                "bun-path-prefixed-contradictory",
                "bun test tests/foo.test.ts",
                "Exit code: 0\ntests/foo.test.ts:\n  console.log\n  (pass) tests/foo.test.ts math > subtracts\nTests: 1 failed, 0 total",
                AttemptOutcome::Failed,
            ),
            (
                "vitest-path-prefixed-zero",
                "vitest run tests/foo.test.ts",
                "Exit code: 0\nPASS tests/foo.test.ts\n  console.log\n  PASS tests/foo.test.ts > math > subtracts\nTests: 0 passed, 0 total",
                AttemptOutcome::Clean,
            ),
        ];

        for (case, command, output, expected_outcome) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1, "{case}");
            assert_eq!(verification[0].outcome, expected_outcome, "{case}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::Unknown,
                "{case}"
            );
        }
    }

    #[test]
    fn checkpoints_accept_direct_javascript_records_outside_capture_blocks() {
        let cases = [
            (
                "jest tests/foo.test.ts",
                "Exit code: 0\n✓ tests/foo.test.ts subtracts (2 ms)\nTests: 0 passed, 0 total",
            ),
            (
                "bun test tests/foo.test.ts",
                "Exit code: 0\n(pass) tests/foo.test.ts math > subtracts\nRan 0 tests across 1 file",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].outcome, AttemptOutcome::Clean, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::TargetExercised,
                "{command}"
            );
        }
    }

    #[test]
    fn checkpoints_reject_parser_lookalikes_as_per_test_execution_records() {
        let cases = [
            (
                "cargo test zero_target -- --exact",
                "Exit code: 0\nrunning 0 tests\nconst NOTE: &str = \"test zero_target ... ok\";\nlog: test zero_target ... failed\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out",
            ),
            (
                "pytest tests/checkpoints_test.py::test_pairing",
                "Exit code: 0\ncollected 0 items\nconst NOTE: &str = \"tests/checkpoints_test.py::test_pairing PASSED\";\n=================== no tests ran in 0.01s ===================",
            ),
            (
                "vitest run tests/foo.test.ts",
                "Exit code: 0\nconst NOTE = \"FAIL tests/foo.test.ts > math > subtracts\";\nTests: 0 passed, 0 total",
            ),
            (
                "jest tests/foo.test.ts",
                "Exit code: 0\nFAIL tests/foo.test.ts\nconsole.log(\"✕ subtracts (2 ms)\")\nTests: 0 passed, 0 total",
            ),
            (
                "bun test tests/foo.test.ts",
                "Exit code: 0\ntests/foo.test.ts:\nconst NOTE = \"(fail) math > subtracts\";\nRan 0 tests across 1 file",
            ),
            (
                "deno test tests/foo.test.ts",
                "Exit code: 0\nrunning 0 tests from tests/foo.test.ts\nlog: subtracts ... FAILED (1ms)",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];
            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].outcome, AttemptOutcome::Clean, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::Unknown,
                "{command}"
            );
        }
    }

    #[test]
    fn checkpoints_fail_closed_on_contradictory_counts_without_per_test_evidence() {
        let contradictory = "Exit code: 0\nrunning 0 tests\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out";
        let rows = vec![
            tool_call(
                0,
                "functions.shell_command",
                "cargo test contradictory -- --exact",
            ),
            tool_output(1, contradictory),
        ];
        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        let verification = build_verification_attempts(&attempts, &rows);
        assert_eq!(verification[0].exercise_state, ExerciseState::Unknown);

        let rows_with_per_test_event = vec![
            tool_call(
                0,
                "functions.shell_command",
                "cargo test contradictory -- --exact",
            ),
            tool_output(
                1,
                "Exit code: 0\nrunning 0 tests\ntest contradictory ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
            ),
        ];
        let attempts = build_command_attempts(
            &rows_with_per_test_event,
            &command_observations(&rows_with_per_test_event),
        );
        let verification = build_verification_attempts(&attempts, &rows_with_per_test_event);
        assert_eq!(
            verification[0].exercise_state,
            ExerciseState::TargetExercised
        );
        assert_eq!(verification[0].outcome, AttemptOutcome::Clean);
    }

    #[test]
    fn checkpoints_reassemble_split_typed_output_segments_before_counting_execution() {
        let positive_rows = vec![
            tool_call_for_call(
                0,
                "call-split",
                "functions.shell_command",
                "cargo test split_target -- --exact",
            ),
            typed_tool_output_for_call(1, "call-split", 0, "Exit code: 0\nrunning 1 test"),
            typed_tool_output_for_call(
                2,
                "call-split",
                1,
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out",
            ),
        ];

        let attempts =
            build_command_attempts(&positive_rows, &command_observations(&positive_rows));
        assert_eq!(attempts[0].output_rows.len(), 2);
        let verification = build_verification_attempts(&attempts, &positive_rows);
        assert_eq!(
            verification[0].exercise_state,
            ExerciseState::TargetExercised
        );

        let masked_failure_rows = vec![
            tool_call_for_call(
                0,
                "call-split-failed",
                "functions.shell_command",
                "cargo test split_target -- --exact",
            ),
            typed_tool_output_for_call(1, "call-split-failed", 0, "Exit code: 0\nrunning 1 test"),
            typed_tool_output_for_call(
                2,
                "call-split-failed",
                1,
                "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out",
            ),
        ];
        let attempts = build_command_attempts(
            &masked_failure_rows,
            &command_observations(&masked_failure_rows),
        );
        assert_eq!(attempts[0].output_rows.len(), 2);
        let verification = build_verification_attempts(&attempts, &masked_failure_rows);
        assert_eq!(verification[0].outcome, AttemptOutcome::Failed);
        assert_eq!(
            verification[0].exercise_state,
            ExerciseState::TargetExercised
        );

        let zero_rows = vec![
            tool_call_for_call(
                0,
                "call-split-zero",
                "functions.shell_command",
                "cargo test split_target -- --exact",
            ),
            typed_tool_output_for_call(1, "call-split-zero", 0, "Exit code: 0\nrunning 0 tests"),
            typed_tool_output_for_call(
                2,
                "call-split-zero",
                1,
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out",
            ),
        ];
        let attempts = build_command_attempts(&zero_rows, &command_observations(&zero_rows));
        assert_eq!(attempts[0].output_rows.len(), 2);
        let verification = build_verification_attempts(&attempts, &zero_rows);
        assert_eq!(verification[0].exercise_state, ExerciseState::Unknown);
    }

    #[test]
    fn checkpoints_keep_failed_tests_with_error_text_as_target_exercised_once_output_shows_execution(
    ) {
        let rows = vec![
            tool_call(
                0,
                "functions.shell_command",
                "pytest tests/checkpoints_test.py::test_pairing",
            ),
            tool_output(
                1,
                "Exit code: 1\nOutput:\n=================== test session starts ===================\ncollected 1 item\n\ntests/checkpoints_test.py::test_pairing FAILED\nAssertionError: expected progress",
            ),
        ];

        let attempts = build_command_attempts(&rows, &command_observations(&rows));
        let verification = build_verification_attempts(&attempts, &rows);

        assert_eq!(verification.len(), 1);
        assert_eq!(verification[0].outcome, AttemptOutcome::Failed);
        assert_eq!(
            verification[0].exercise_state,
            ExerciseState::TargetExercised
        );
    }

    #[test]
    fn checkpoints_leave_truncated_runtime_failure_text_unknown_without_execution_evidence() {
        let cases = [
            (
                "pytest tests/checkpoints_test.py::test_pairing",
                "Exit code: 1\nAssertionError: expected progress",
            ),
            (
                "vitest run tests/foo.test.ts",
                "Exit code: 1\nTypeError: expected 2",
            ),
            (
                "jest tests/foo.test.ts",
                "Exit code: 1\nFAIL tests/foo.test.ts\nSyntaxError: failed to transform the suite",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];

            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1);
            assert_eq!(verification[0].exercise_state, ExerciseState::Unknown);
        }
    }

    #[test]
    fn checkpoints_recognize_failed_js_runner_output_shapes_as_target_exercised() {
        let cases = [
            (
                "vitest run tests/foo.test.ts",
                "Exit code: 1\n FAIL  tests/foo.test.ts > math > subtracts\n AssertionError: expected 2",
            ),
            (
                "jest tests/foo.test.ts",
                "Exit code: 1\n FAIL tests/foo.test.ts\n  math\n    ✕ subtracts (2 ms)",
            ),
            (
                "bun test tests/foo.test.ts",
                "Exit code: 1\ntests/foo.test.ts:\n(fail) math > subtracts\n  Expected: 2",
            ),
            (
                "deno test tests/foo.test.ts",
                "Exit code: 1\nrunning 1 test from tests/foo.test.ts\nsubtracts ... FAILED (1ms)",
            ),
        ];

        for (command, output) in cases {
            let rows = vec![
                tool_call(0, "functions.shell_command", command),
                tool_output(1, output),
            ];

            let attempts = build_command_attempts(&rows, &command_observations(&rows));
            let verification = build_verification_attempts(&attempts, &rows);

            assert_eq!(verification.len(), 1, "{command}");
            assert_eq!(verification[0].outcome, AttemptOutcome::Failed, "{command}");
            assert_eq!(
                verification[0].exercise_state,
                ExerciseState::TargetExercised,
                "{command}"
            );
        }
    }

    fn command_observations(rows: &[CompactionRow]) -> Vec<CommandObservation> {
        rows.iter()
            .filter(|row| row.kind == CompactionKind::ToolCall)
            .map(|row| CommandObservation {
                family: row
                    .text
                    .split_whitespace()
                    .next()
                    .unwrap_or("tool_call")
                    .to_string(),
                raw_command: row.text.clone(),
                tool_name: tool_name_for(&row.text).to_string(),
                paths: extract_paths(&row.text),
                read_like: false,
                write_like: false,
                verification_like: false,
                evidence: vec![EvidenceRef {
                    row: agent_session_compactor::RowRef::from_row(row),
                    reason: "command family".to_string(),
                }],
            })
            .collect()
    }

    fn extract_paths(command: &str) -> Vec<String> {
        command
            .split_whitespace()
            .filter(|token| token.contains('/') || token.ends_with(".py") || token.ends_with(".rs"))
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    fn tool_name_for(command: &str) -> &'static str {
        if command.starts_with("spawn_agent") {
            "spawn_agent"
        } else if command.starts_with("apply_patch") {
            "apply_patch"
        } else {
            "functions.shell_command"
        }
    }

    fn tool_call(event_index: usize, _tool_name: &str, text: &str) -> CompactionRow {
        row(event_index, CompactionKind::ToolCall, text, None)
    }

    fn tool_call_for_call(
        event_index: usize,
        call_id: &str,
        tool_name: &str,
        text: &str,
    ) -> CompactionRow {
        row(
            event_index,
            CompactionKind::ToolCall,
            text,
            Some(format!(
                "{{\"call_id\":\"{call_id}\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
            )),
        )
    }

    fn tool_output(event_index: usize, text: &str) -> CompactionRow {
        row(event_index, CompactionKind::ToolOutput, text, None)
    }

    fn tool_output_for_call(event_index: usize, call_id: &str, text: &str) -> CompactionRow {
        row(
            event_index,
            CompactionKind::ToolOutput,
            text,
            Some(format!(
                "{{\"call_id\":\"{call_id}\",\"type\":\"function_call_output\"}}"
            )),
        )
    }

    fn typed_tool_output_for_call(
        event_index: usize,
        call_id: &str,
        segment_index: usize,
        text: &str,
    ) -> CompactionRow {
        row(
            event_index,
            CompactionKind::ToolOutput,
            text,
            Some(format!(
                "{{\"call_id\":\"{call_id}\",\"name\":\"shell_command\",\"segment_index\":{segment_index},\"segment_type\":\"input_text\",\"type\":\"custom_tool_call_output\"}}"
            )),
        )
    }

    fn unknown_row(event_index: usize, text: &str) -> CompactionRow {
        row(event_index, CompactionKind::Unknown, text, None)
    }

    fn assistant_message(event_index: usize, text: &str) -> CompactionRow {
        row(event_index, CompactionKind::AssistantMessage, text, None)
    }

    fn row(
        event_index: usize,
        kind: CompactionKind,
        text: &str,
        dedupe_identity: Option<String>,
    ) -> CompactionRow {
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
            dedupe_identity,
            text: text.to_string(),
            canonical_text: text.to_string(),
            text_hash_hex: format!("{kind:?}-{event_index}"),
        }
    }
}
