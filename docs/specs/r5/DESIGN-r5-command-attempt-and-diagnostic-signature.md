# Design: R5 Command Attempt And Diagnostic Signature

Status: canonical design authority locked in Packet R5-0 on 2026-06-09.

## Why This Doc Exists

R5 cannot honestly answer "is the troubleshooting frontier moving?" directly from raw rows,
command-family counts, or repeated text hashes. The analyzer needs a small internal intermediate
representation that pairs commands with outputs, classifies verification attempts, normalizes
machine diagnostics, and compares comparable attempts over time.

This doc freezes the R5 internal mechanism for:

1. `CommandAttempt`
2. `VerificationAttempt`
3. `DiagnosticSignature`
4. diagnostic matching
5. edit-overlap evidence

The public checkpoint schema should expose only `SessionProgress`; these attempt/signature objects
are internal or debug-only in the first landing.

## Current Repo Reality

The current analyzer already has useful raw material:

1. `agent_session_compactor::CompactionRow` contains:
   - `source_file`
   - `event_index`
   - `row_ordinal`
   - optional `timestamp`
   - `kind`
   - optional `turn_id`
   - `text`
   - `canonical_text`
   - `text_hash_hex`
2. `collect_command_observations(...)` currently scans `ToolCall` rows and extracts:
   - `family`
   - `raw_command`
   - `tool_name`
   - paths from command text and apply-patch text
   - coarse `read_like`, `write_like`, and `verification_like` flags
   - row evidence
3. R4 added richer private command-role and file-role classification inside
   `checkpoint/mod.rs`, including subcommand-aware handling for `cargo`, `npm`, `pnpm`, and `git`.
4. Existing failure evidence remains intentionally narrow: generic `ToolOutput` is neutral unless
   it starts with `error:` or carries a non-zero `Exit code:` line. R5 diagnostic parsing must not
   widen scorer failure semantics before R6.
5. `repetition_slice(...)` currently groups repeated verification commands and repeated failure
   rows by `text_hash_hex`. R5 should keep this as legacy context but add normalized diagnostics for
   progress interpretation.

## Internal Module Placement

Recommended module split:

```text
crates/agent-drift-analyzer/src/checkpoint/attempt.rs
  CommandAttempt, CommandAttemptRole, AttemptOutcome, VerificationAttempt, ExerciseState,
  command/output pairing, target-scope extraction.

crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs
  DiagnosticSignature, FailureClass, VerifierKind, canonicalization, matching, fail-count parsing,
  path/test/symbol extraction.

crates/agent-drift-analyzer/src/checkpoint/progress.rs
  Consumes attempts, signatures, task-frame deltas, recovery/repetition, archetype, and delegation
  to produce SessionProgress.
```

If implementation pressure favors keeping helpers in `checkpoint/mod.rs` for the first commit, the
logical seams still need to remain explicit and tested. Do not grow another monolithic classifier
without these boundaries.

## CommandAttempt

A `CommandAttempt` is the analyzer's internal unit for "one visible tool command plus its immediate
visible result span."

Recommended shape:

```rust
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
```

### Pairing Rule

The first implementation should pair a `ToolCall` row to subsequent visible rows until one of these
boundaries appears:

1. the next `ToolCall`,
2. the next user/developer/system/assistant message that starts a new focusable phase,
3. the end of the current checkpoint interval.

Rows eligible for `output_rows`:

1. `ToolOutput`,
2. `Error`,
3. `Unknown` only when adjacent to the command and carrying an obvious exit/diagnostic shape.

This is intentionally conservative. If command-output pairing is ambiguous, produce an attempt with
`AttemptOutcome::Unknown` and low-confidence downstream progress.

### Exit Code Rule

Parse `Exit code: N` from the first lines of the paired output span. The parser should prefer the
first explicit exit code after the command and ignore unrelated historical exit-code mentions in
later prose.

```text
Exit code: 0  -> AttemptOutcome::Clean
Exit code: nonzero -> AttemptOutcome::Failed
No exit code but Error row -> AttemptOutcome::Failed
No exit code and no explicit failure -> AttemptOutcome::Unknown, unless a known verifier emits a
stable success phrase that the parser explicitly supports.
```

Do not use richer diagnostic parsing to change `classify_outcome_evidence(...)` in R5.

## Command Role Classification

R4's private `CommandRole` is sufficient for archetype, but R5 needs progress-specific role
granularity. The first `CommandAttemptRole` classifier should recognize:

| Family / pattern | Role |
|---|---|
| `cargo check` | `Compile` |
| `cargo test` | `Test` |
| `cargo clippy` | `Lint` |
| `cargo fmt --check` | `FormatCheck` |
| `cargo fmt` without `--check` | `FormatWrite` |
| `cargo build` | `Build` |
| `npm test`, `pnpm test`, `vitest`, `jest`, `bun test`, `deno test` | `Test` |
| `pytest`, `python -m pytest`, `uv run pytest` | `Test` |
| `go test`, `swift test`, `mvn test`, `gradle test`, `make test`, `just test` | `Test` |
| `npm install`, `pnpm install`, `cargo add`, `cargo update` | `DependencyMutation` |
| `apply_patch`, apply-patch-like text, `mv`, `cp`, `mkdir` | `Edit` |
| `git diff`, `git status`, `git show`, `git log` | `VcsInspection` |
| command containing `replay` or sentinel/analyzer replay invocation | `Replay` |
| `spawn_agent`, `wait_agent`, `close_agent`, `multi_agent_v1` | `Orchestration` |
| `cat`, `sed`, `rg`, `ls`, `find`, `head`, `tail`, `jq` | `Read` |

Unknown is acceptable. Unknown should lower confidence, not block checkpoint export.

## VerificationAttempt

A `VerificationAttempt` is a command attempt that actually tries to exercise a build/test/lint/replay
frontier.

Recommended shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerificationAttempt {
    pub attempt_ordinal: usize,
    pub verifier: VerifierKind,
    pub target_scope: VerificationScope,
    pub exercise_state: ExerciseState,
    pub outcome: AttemptOutcome,
    pub signatures: Vec<DiagnosticSignature>,
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
```

`ExerciseState` imports the key DoVer idea: distinguish "the hypothesis was refuted" from "the
system never actually exercised the intended target." For example:

1. `cargo test foo` that fails to compile before running tests should be `BlockedBeforeTarget`.
2. `cargo test foo` that runs and fails `foo` should be `TargetExercised`.
3. `cargo test` with unclear output can be `Unknown`.

## DiagnosticSignature

A `DiagnosticSignature` is the normalized, comparable representation of a failed attempt.

Recommended shape:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
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
pub(crate) enum VerifierKind {
    CargoCheck,
    CargoTest,
    CargoClippy,
    CargoFmt,
    NpmTest,
    PnpmTest,
    Pytest,
    GenericTest,
    GenericBuild,
    Replay,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
```

### Canonicalization Rules

R5 should canonicalize paired output text before hashing:

1. strip ANSI control sequences,
2. normalize line endings and repeated whitespace,
3. fold absolute repo paths to repo-relative paths when possible,
4. drop timestamps, durations, random seeds, temp dirs, cache ids, and ephemeral ports,
5. normalize line and column numbers unless they are the only discriminative feature,
6. sort multiple diagnostic units by `(path, symbol/test, code/class, normalized message)`,
7. hash canonical JSON with a declared `normalization_version`.

Do not hash the raw `ToolOutput` body as the primary progress identity. Exact raw hashes are too
brittle and repeat the limitation of the current `text_hash_hex` loop detection.

### Initial Parser Coverage

The first parser does not need full language-server accuracy. It should cover obvious shapes:

| Tool | Patterns to detect |
|---|---|
| Cargo compile/check | `error[E....]`, `could not compile`, Rust file paths, module/test names where available |
| Cargo test | `test result: FAILED`, `N passed; M failed`, failing test names, assertion/golden hints |
| Cargo clippy | `warning:` / `error:` with clippy-ish command, file paths |
| Cargo fmt | `Diff in`, format check failure, changed file paths |
| Pytest | `N failed`, `N passed`, pytest node ids, file paths |
| Vitest/Jest | failed test counts, suite/test names, file paths |
| Replay/sentinel/analyzer | checkpoint contract errors, schema version errors, expected/missing field names |
| Generic | non-zero exit-code only with low parser confidence |

When parsing is weak, emit `FailureClass::ExitCodeOnly` or `FailureClass::Unknown` with low parser
confidence. That is still useful as counter-evidence but should not drive high-confidence progress.

## Matching Ladder

Use a ladder instead of one equality check:

| Match level | Rule | Progress use |
|---|---|---|
| `exact` | same verifier, same failure class, same target fingerprint, same payload hash | repeated same failure |
| `strong_fuzzy` | same verifier/class and high path/test/symbol overlap | probably same frontier |
| `frontier_related` | same verifier family or target, but failure class moved along pipeline order | possible advancement/regression |
| `weak_related` | same family only, shallow path overlap | context only |
| `unrelated` | no useful overlap | do not compare as one frontier |

R5 should prefer `insufficient_evidence` over pretending unrelated attempts prove progress.

## Failure-Class Partial Order

For comparable targets, use this rough frontier order:

```text
EnvSetup / DependencyResolution
  -> CompileType / BuildLink
  -> TestDiscovery
  -> TestExecution
  -> AssertionOrGolden / ReplayMismatch
  -> Clean
```

Examples:

1. `CompileType` -> `AssertionOrGolden` on the same target = advancement.
2. `AssertionOrGolden` -> `CompileType` after edits = regression.
3. `ExitCodeOnly` -> `ExitCodeOnly` with same hash and no overlapping edits = stalled.
4. `Unknown` should rarely produce more than low-confidence mixed/stalled evidence.

Do not compare unrelated branches or broad target changes as a single frontier.

## Edit-Overlap Evidence

R5 should compute overlap between failed diagnostic scope and intervening write attempts.

Overlap strength:

| Strength | Rule |
|---|---|
| `strong` | exact failing path or symbol/test path edited between comparable attempts |
| `moderate` | same directory/module or test/source counterpart edited |
| `weak` | same crate/package or broad working-set overlap |
| `none` | no observed related edit |

Positive signal:

```text
signature changed or failure class advanced + strong/moderate edit overlap
```

Negative signal:

```text
exact or strong-fuzzy repeated signature + no overlap
```

Keep the public reason non-causal:

```text
"intervening edit overlapped the failing scope before diagnostic frontier changed"
```

not:

```text
"the edit fixed the failing scope"
```

## Debug Output

Optional and explicitly non-blocking for R5:

```text
progress_debug.jsonl
```

One record per checkpoint could include:

1. checkpoint id,
2. progress window id,
3. command attempts,
4. verification attempts,
5. diagnostic signatures,
6. match decisions,
7. final `SessionProgress`.

This debug artifact is not part of the public checkpoint contract, but it will make semantic review
much easier and mirrors the audit-log pattern from DoVer, AgentRx, and Lanser.

## Non-Goals

This design does not require:

1. language-server integration,
2. AST-level selectors,
3. learned parsers,
4. perfect test-name extraction,
5. cross-session child-rollout stitching,
6. changes to upstream compactor rows,
7. changing drift scorer failure semantics before R6.

## Locked Decisions After Packet R5-0

1. If `progress_debug.jsonl` is emitted at all, it lands no earlier than the export-facing stage
   and remains optional; no R5 packet is gated on it.
2. Exact signature hashes stay out of public `ProgressSignal.before/after`; those fields remain
   human-readable summaries, while hashes stay internal or debug-only.
3. `cargo fmt` without `--check` is treated as `FormatWrite` in the first implementation.
