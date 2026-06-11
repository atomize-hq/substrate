# Tasks: Agent Drift Analyzer Session Progress R5

Status: Packet R5-7 landed on 2026-06-10, closing the dedicated acceptance-fixture wall after the already-landed R5-0 through R5-6 packets.

Completion status on 2026-06-10:

- Packets `R5-0` through `R5-6` were already landed before this packet.
- Packet `R5-7` is now landed as the dedicated acceptance-fixture wall.
- The remaining unchecked global items below are legacy checklist text from earlier packet planning, not open R5-7 implementation tasks.

Each task should be completable in one focused implementation session. Keep each task as close as
possible to five touched files or fewer. Do not advance from one packet to the next until the
verification command for the current packet is green or the failure is documented in the task note.

## R5-0: Docs Lock

- [x] Task R5-0.1: Add canonical R5 DESIGN docs.
  - Acceptance: `DESIGN-r5-session-progress-contract.md`,
    `DESIGN-r5-command-attempt-and-diagnostic-signature.md`,
    `DESIGN-r5-progress-window-and-frontier-model.md`,
    `DESIGN-r5-archetype-progress-rules.md`,
    `DESIGN-r5-delegation-progress-guardrails.md`, and
    `DESIGN-r5-validation-and-rollout-protocol.md` exist under `docs/specs/r5/`.
  - Verify: Manual review confirms all docs agree on v0.6, public DTO shape, non-goals, and
    R6/R7 boundaries.
  - Files: `docs/specs/r5/DESIGN-r5-*.md`

- [x] Task R5-0.2: Add R5 spec, plan, tasks, and fixture manifest.
  - Acceptance: R5 has `spec`, `plan`, `tasks`, and `fixtures` docs with commands, boundaries,
    success criteria, packet split, and required fixture matrix.
  - Verify: Manual review against the R5 spec / plan / fixture manifest / DESIGN stack,
    including `DESIGN-r5-validation-and-rollout-protocol.md`, and `AGENTS.md`.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

## R5-1: Public Schema Skeleton And Compatibility

- [ ] Task R5-1.1: Add public `SessionProgress` DTOs to analyzer schema.
  - Acceptance: `SessionProgress`, `ProgressStatus`, `ProgressDimension`, `ProgressSignal`,
    `ProgressSignalCode`, `SignalPolarity`, and `SignalStrength` exist in `checkpoint/schema.rs`
    with serde `snake_case` enum serialization.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/lib.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-1.2: Add optional `Checkpoint.session_progress` with v0.6 requiredness.
  - Acceptance: v0.2-v0.5 deserialize without `session_progress`; v0.6 missing
    `session_progress` fails closed; v0.6 with progress round-trips.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-1.3: Emit temporary low-confidence `session_progress` and bump analyzer output to
      v0.6.
  - Acceptance: analyzer-created checkpoints use `schema_version = "v0.6"` and contain
    `Some(session_progress)`; the temporary progress status is explicitly conservative.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5-2: CommandAttempt / VerificationAttempt Foundation

- [ ] Task R5-2.1: Add `checkpoint/attempt.rs` and wire module exports internally.
  - Acceptance: `CommandAttempt`, `CommandAttemptRole`, `AttemptOutcome`, `VerificationAttempt`,
    `ExerciseState`, and `VerificationScope` exist as `pub(crate)` types.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`

- [ ] Task R5-2.2: Implement command/output pairing.
  - Acceptance: each visible `ToolCall` in a checkpoint interval can produce a `CommandAttempt`
    with paired `ToolOutput`/`Error` rows until the next command or focusable phase boundary;
    ambiguous output yields `Unknown` rather than false failure/success.
  - Verify: focused tests in `checkpoints.rs`.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-2.3: Implement progress-specific command role classification.
  - Acceptance: cargo/npm/pnpm/pytest/vitest/replay/apply_patch/git/delegation commands classify
    into R5 roles deterministically.
  - Verify: focused role tests in `checkpoints.rs`.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-2.4: Build verification attempts and target exercise state.
  - Acceptance: verification-like attempts have verifier kind, target scope, and `ExerciseState`;
    compile-blocked tests can be marked `BlockedBeforeTarget`.
  - Verify: focused verification-attempt tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5-3: DiagnosticSignature And Matching

- [ ] Task R5-3.1: Add `checkpoint/diagnostics.rs` with signature types and canonicalization.
  - Acceptance: diagnostic types exist; canonicalization removes ANSI, volatile whitespace,
    timestamps/durations/temp paths, and line/column volatility where appropriate.
  - Verify: diagnostic canonicalization tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-3.2: Parse initial cargo/test/replay diagnostic classes and fail counts.
  - Acceptance: parser recognizes common cargo compile/test, pytest-like, JS test-like, and replay
    contract-failure shapes; weak/unknown output yields low-confidence signatures.
  - Verify: focused parser tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-3.3: Implement diagnostic matching ladder.
  - Acceptance: exact, strong-fuzzy, frontier-related, weak-related, and unrelated matching are
    deterministic and covered by tests.
  - Verify: focused matching tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-3.4: Implement edit-overlap classification.
  - Acceptance: overlapping path/symbol/test edits produce strong/moderate overlap; unrelated edits
    produce none/weak; signals can cite the command and edit rows.
  - Verify: focused overlap tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5-4: Progress Engine

- [ ] Task R5-4.1: Add `checkpoint/progress.rs` and dimension selection.
  - Acceptance: dimension derives from `session_archetype.label`, with delegation allowed to
    override only the dimension to `parent_visible_orchestration`; the resulting status stays
    `insufficient_evidence`, `stalled`, or `mixed` according to the visible parent evidence.
  - Verify: focused dimension tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-4.2: Implement troubleshooting frontier progress.
  - Acceptance: compile-to-test advancement, fail-count reduction, repeated same signature, prior
    clean broken, and sparse evidence cases match fixture manifest expectations.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-4.3: Implement planning convergence rules conservatively.
  - Acceptance: plan/spec artifact creation/refinement and candidate-set narrowing can produce
    low/medium advancing; broad meander produces stalled/mixed/insufficient according to fixture
    manifest.
  - Verify: focused planning progress tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-4.4: Implement implementation verification-wall progress.
  - Acceptance: concentrated source/test edits plus advancing verifier produce advancing; unrelated
    churn plus same verifier failure produces stalled or mixed unless a previously clean or later
    frontier was actually broken.
  - Verify: focused implementation progress tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-4.5: Implement verification closeout narrowing.
  - Acceptance: closeout proof that narrows the residual scope emits
    `VerificationScopeNarrowed` plus `ResidualScopeShrank` and advances under
    `verification_closeout_narrowing`; source churn or proof break in closeout yields
    mixed/regressing; closeout prose without proof yields insufficient evidence.
  - Verify: focused closeout progress tests, including the narrowed-residual closeout case.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5-4.6: Apply delegation caps and evidence limiting.
  - Acceptance: opaque delegated-parent checkpoints never claim child progress and never emit high
    confidence; limiting signal/evidence is present.
  - Verify: focused delegated-progress tests.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5-5: Analyzer Export Summary

- [ ] Task R5-5.1: Add progress distributions to analyzer summary.
  - Acceptance: top summary includes status and dimension distributions.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [ ] Task R5-5.2: Add per-checkpoint progress line to summary.
  - Acceptance: checkpoint lines include compact `progress:` inspection with status, dimension,
    confidence, support, and counter-evidence.
  - Verify: `cargo test -p agent-drift-analyzer export_bundle -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/export.rs`
    - `crates/agent-drift-analyzer/tests/export_bundle.rs`

- [ ] Task R5-5.3: Preserve end-to-end analyzer artifact stability.
  - Acceptance: end-to-end tests assert v0.6 checkpoints and progress summary without breaking
    existing turn/delegation/archetype summary expectations.
  - Verify: `cargo test -p agent-drift-analyzer end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/end_to_end.rs`

## R5-6: Sentinel v0.6 Compatibility

- [ ] Task R5-6.1: Add replay input v0.6 support and requiredness.
  - Acceptance: replay supports v0.6; v0.6 missing `session_progress` fails with contract gap;
    v0.5 without progress still loads.
  - Verify: `cargo test -p agent-drift-sentinel replay_input -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/input.rs`
    - `crates/agent-drift-sentinel/tests/replay_input.rs`

- [ ] Task R5-6.2: Add live input v0.6 support and requiredness.
  - Acceptance: live checkpoint compatibility supports v0.6; v0.6 missing progress fails; fixture
    validation enforces the same requiredness.
  - Verify: `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/live_input.rs`
    - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

- [ ] Task R5-6.3: Render compact Progress line in operator surface.
  - Acceptance: replay/live console blocks render `Progress:` after `Archetype:`; presentation does
    not change posture/disposition/severity.
  - Verify: `cargo test -p agent-drift-sentinel operator_surface -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/src/operator_surface.rs`
    - `crates/agent-drift-sentinel/tests/operator_surface.rs`

- [ ] Task R5-6.4: Preserve replay/live parity.
  - Acceptance: matching v0.6 checkpoint surfaces show the same progress line in replay and live.
  - Verify: `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

## R5-7: Acceptance Fixture Wall

- [x] Task R5-7.1: Add frozen progress acceptance fixture README and expected-case docs.
  - Acceptance: fixture README now documents the included six-case corpus, excluded delegated/redundant bundles, and the R5 rule that delegated cases remain guardrail-only until R7; each case now carries committed `expected.json` docs.
  - Verify: Manual review.
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**/expected.json`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

- [x] Task R5-7.2: Add bounded progress acceptance tests.
  - Acceptance: `progress_acceptance.rs` now asserts selected-checkpoint archetype, dimension, status, confidence bounds, required signal codes, and evidence minima across a committed six-case corpus that spans troubleshooting, planning, implementation, closeout, and parent-visible delegation guardrails; the legacy R2 acceptance wall stays separate and unchanged; and the landed packet’s narrow `crates/agent-drift-analyzer/src/checkpoint/progress.rs` refinement records delegated child-opaque limiting evidence in `counter_evidence` for the `parent_visible_orchestration` guardrail case.
  - Verify: `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`

- [x] Task R5-7.3: Run full verification wall and update docs with final status.
  - Acceptance: the analyzer and sentinel full test walls pass, the docs now name the landed corpus shape and concrete fixture locations, and the annotated real-rollout acceptance bar is explicitly marked met; `cargo fmt --all -- --check` remains red only because tracked preexisting formatting drift persists in `crates/agent-drift-analyzer/src/checkpoint/export.rs`, `crates/agent-drift-analyzer/src/checkpoint/mod.rs`, and `crates/agent-drift-analyzer/tests/export_bundle.rs` outside the Packet R5-7 diff.
  - Verify:
    ```bash
    cargo fmt --all -- --check
    cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture
    cargo test -p agent-drift-analyzer -- --nocapture
    cargo test -p agent-drift-sentinel -- --nocapture
    ```
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

## Global Completion Checklist

- [ ] Analyzer emits `schema_version = "v0.6"`.
- [ ] v0.6 requires `session_progress`.
- [ ] v0.2-v0.5 remain compatible.
- [x] Progress statuses and dimensions match fixture manifest.
- [x] Progress evidence/counter-evidence is populated.
- [ ] Delegation opacity caps progress claims.
- [ ] Analyzer summary includes progress output.
- [ ] Sentinel replay/live accept and render v0.6.
- [ ] No scorer retuning landed.
- [x] Full analyzer and sentinel tests pass.
