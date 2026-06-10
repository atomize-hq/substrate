use std::collections::BTreeMap;

use agent_session_compactor::{CompactionKind, CompactionRow, RowRef};
use camino::Utf8PathBuf;
use serde_json::Value;

use crate::context::CommandObservation;

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
    pub verifier: VerifierKind,
    pub target_scope: VerificationScope,
    pub exercise_state: ExerciseState,
    pub outcome: AttemptOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
            Some(VerificationAttempt {
                attempt_ordinal: attempt.ordinal,
                verifier,
                exercise_state: exercise_state(attempt, &target_scope, &output),
                target_scope,
                outcome: attempt.outcome,
            })
        })
        .collect()
}

fn pair_output_rows(rows: &[CompactionRow], command_index: usize) -> Vec<&CompactionRow> {
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
    rows.iter().find_map(|row| {
        row.text.lines().find_map(|line| {
            line.strip_prefix("Exit code: ")
                .and_then(|value| value.trim().parse::<i32>().ok())
        })
    })
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
    if matches!(
        tool_name,
        "spawn_agent" | "wait_agent" | "close_agent" | "multi_agent_v1"
    ) || normalized.contains("spawn_agent")
        || normalized.contains("wait_agent")
        || normalized.contains("close_agent")
        || normalized.contains("multi_agent_v1")
    {
        return CommandAttemptRole::Orchestration;
    }

    if normalized.contains("replay") || family.eq_ignore_ascii_case("replay") {
        return CommandAttemptRole::Replay;
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
        return git_role(&normalized_command_tokens(raw_command))
            .unwrap_or(CommandAttemptRole::Unknown);
    }

    if let Some(role) = cargo_role(&normalized_command_tokens(raw_command)) {
        return role;
    }

    if let Some(role) = npm_like_role(&normalized_command_tokens(raw_command), family) {
        return role;
    }

    if let Some(role) = shell_wrapped_test_role(&normalized_command_tokens(raw_command)) {
        return role;
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
    target_scope: &VerificationScope,
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
        CommandAttemptRole::Test => match attempt.outcome {
            AttemptOutcome::Clean => ExerciseState::TargetExercised,
            AttemptOutcome::Failed if output_contains_compile_blocker(output) => {
                ExerciseState::BlockedBeforeTarget
            }
            AttemptOutcome::Failed if output_shows_test_execution(output, target_scope) => {
                ExerciseState::TargetExercised
            }
            AttemptOutcome::Failed | AttemptOutcome::Unknown => ExerciseState::Unknown,
        },
        _ => ExerciseState::Unknown,
    }
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
        "error:",
        "cannot find",
        "unresolved import",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn output_shows_test_execution(output: &str, target_scope: &VerificationScope) -> bool {
    let lower = output.to_ascii_lowercase();
    if [
        "running ",
        "test result:",
        "failures:",
        "collected ",
        "passed",
        "failed",
        "ok",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return true;
    }

    target_scope
        .tests
        .iter()
        .any(|test| lower.contains(&test.to_ascii_lowercase()))
        || target_scope
            .paths
            .iter()
            .any(|path| lower.contains(&path.to_ascii_lowercase()))
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
    positional_targets(args, &["-k", "-m", "-t", "--filter", "--grep", "--project"])
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
            ("pnpm test -- --runInBand", CommandAttemptRole::Test),
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
                "=================== test session starts ===================\ncollected 1 item\n\ntests/checkpoints_test.py::test_pairing FAILED\nExit code: 1",
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

    fn tool_call(event_index: usize, tool_name: &str, text: &str) -> CompactionRow {
        row(
            event_index,
            CompactionKind::ToolCall,
            text,
            Some(format!(
                "{{\"call_id\":\"call-{event_index}\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
            )),
        )
    }

    fn tool_output(event_index: usize, text: &str) -> CompactionRow {
        row(event_index, CompactionKind::ToolOutput, text, None)
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
