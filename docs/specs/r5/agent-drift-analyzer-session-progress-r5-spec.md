# Spec: Agent Drift Analyzer Session Progress R5

## Assumptions I'm Making

1. Live repo truth on `2026-06-09` is the authority: `R1`, `R2`, `R3`, `R3.5`, `R3.75`, and `R4`
   are landed. The current top-of-stack action is `R5` archetype-aware progress semantics.
2. `R4` currently exports analyzer checkpoint schema `v0.5` with `turn_context` and
   `session_archetype`. `R5` should widen this to `v0.6` by adding `session_progress`.
3. The shared `Checkpoint` DTO must remain legacy-safe. `session_progress` should be optional at
   serde shape level and required only by `v0.6` validation, mirroring the existing R4 pattern for
   `session_archetype`.
4. Public progress evidence should reuse the existing public `EvidenceRef { row, reason }` type.
   Rich attempt, path, symbol, signature, and diagnostic anchors should stay internal/debug-only in
   the first R5 landing.
5. R5 is analyzer-owned and descriptive. It must not retune `dead_end_thrash`, change scheduler
   policy, or reinterpret sentinel posture. Those belong to R6+.
6. R5 may parse diagnostic output more richly than R1 outcome evidence, but it must not change the
   existing R1 rule that generic `ToolOutput` is neutral for failure scoring unless it has
   unambiguous failure markers.
7. R5 should build on existing analyzer structures:
   - `CheckpointAnalysis`
   - `IntervalSlice`
   - `RepetitionSlice`
   - `RecoveryState`
   - `TaskFrameDelta`
   - `TurnContext`
   - `SessionArchetype`
   - private `DelegationContext`
8. R5 should remain deterministic and rule-based. The DoVer/AgentLens/TRAJEVAL/Lanser/AgentRx
   research informs the design but does not introduce learned monitors, online interventions, or
   external benchmark dependencies.
9. Delegated-session support remains bounded. R5 can cap confidence or report
   `parent_visible_orchestration`; R7 remains the first packet allowed to add full parent/child
   semantic linkage.
10. The implementation should be split so each task is reviewable and avoids touching too many
    files at once.

If any of these assumptions drift, update this spec before implementation.

## Packet R5-0 Locked Decisions

Packet `R5-0` closes the remaining family-shaping questions for `R5`.

1. The packet split is fixed at `R5-0` through `R5-7` exactly as described in the plan; there is
   no `R5.5` and no silent widening of packet scope.
2. The first public `ProgressSignalCode` set is the full initial enum from the DESIGN contract,
   including `VerificationScopeBroadened` and `VerificationScopeNarrowed`.
3. `PlanningConvergence` is capped at `medium` confidence throughout `R5`.
4. `ParentVisibleOrchestration` is a delegation-only fallback in `R5`, not a generic
   non-delegated workflow dimension.
5. The family verification story is fixed: `R5-0` is manual doc review only; `R5-1` through
   `R5-6` prove schema/behavior/presentation/compatibility; `R5-7` is the first packet allowed to
   claim bounded semantic acceptance on a dedicated progress corpus, and only after at least one
   annotated real-rollout acceptance pass.
6. `progress_debug.jsonl` is optional, debug-only, and never required for packet or family
   review-clean status in `R5`.
7. The frozen `R2` acceptance wall stays stable by default; `R5` semantic acceptance uses a
   dedicated `progress_acceptance.rs` harness plus `tests/fixtures/progress_acceptance/**`, with
   realistic bundle-shaped cases serving only as supporting coverage around the required annotated
   real-rollout case.

## Objective

Add analyzer-owned per-checkpoint `session_progress` state that measures progress relative to the
landed R4 `session_archetype`.

Primary users:

1. maintainers reading analyzer checkpoints and summary output,
2. sentinel operators viewing compact replay/live checkpoint surfaces,
3. future R6 scorer work that needs typed progress context instead of rediscovering progress from
   raw rows and repeated-command heuristics,
4. future R7 delegation work that needs R5 to avoid overclaiming under opaque child work.

Success means each `v0.6` checkpoint can answer:

```text
What progress dimension applies here?
Is the checkpoint advancing, mixed, stalled, regressing, or insufficiently evidenced?
How confident is that assessment?
Which typed signals support it?
Which evidence and counter-evidence rows justify it?
```

R5's first priority is troubleshooting frontier movement because it is the most machine-checkable
and the most directly connected to the current false-positive risk around repeated verification
loops. Planning convergence, implementation verification-wall progress, and closeout narrowing
should land as conservative deterministic rules after the attempt/signature foundation exists.

## Tech Stack

- Language: Rust 2021
- Existing crates:
  - `agent-session-compactor`
  - `agent-drift-analyzer`
  - `agent-drift-sentinel`
- Existing checkpoint schema before R5:
  - `v0.5`
- New checkpoint schema owned by R5:
  - `v0.6`
- Existing sentinel compatibility before R5:
  - `v0.2 | v0.3 | v0.4 | v0.5`
- Required sentinel compatibility after R5:
  - `v0.2 | v0.3 | v0.4 | v0.5 | v0.6`

No new external crate dependency is required by default. If a small helper dependency is proposed
for regex/canonicalization, ask first and justify it against the current dependency posture.

## Commands

Formatting:

```bash
cargo fmt --all -- --check
```

Analyzer-focused validation:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture # dedicated R5 semantic wall
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture # legacy-stability-only frozen R2 wall
cargo test -p agent-drift-analyzer -- --nocapture
```

Sentinel compatibility and operator validation:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Optional end-to-end manual wall when a real compactor bundle is available:

```bash
cargo run -p agent-drift-analyzer -- \
  --input-dir target/hybrid-drift-evals/<case>/compactor \
  --output-dir target/hybrid-drift-evals/<case>/analyzer-r5

cargo run -p agent-drift-sentinel -- \
  --mode replay \
  --checkpoint-dir target/hybrid-drift-evals/<case>/analyzer-r5
```

## Project Structure

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Add public R5 DTOs: SessionProgress, ProgressStatus, ProgressDimension, ProgressSignal,
  ProgressSignalCode, SignalPolarity, SignalStrength. Add optional Checkpoint.session_progress and
  v0.6 requiredness in RawCheckpoint validation.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Existing checkpoint analysis, R4 archetype building, checkpoint construction, turn context,
  repetition/recovery helpers. R5 should call progress construction during checkpoint assembly but
  should avoid dumping all new logic here.

crates/agent-drift-analyzer/src/checkpoint/attempt.rs
  New internal seam for CommandAttempt, VerificationAttempt, command/output pairing,
  progress-specific command roles, target scope, and exercise state.

crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs
  New internal seam for DiagnosticSignature, FailureClass, VerifierKind, canonicalization, parser
  coverage, matching ladder, fail-count extraction, and path/test/symbol extraction.

crates/agent-drift-analyzer/src/checkpoint/progress.rs
  New internal progress engine. Consumes CheckpointAnalysis, SessionArchetype, attempts,
  signatures, recovery/repetition, task-frame deltas, and delegation context to produce
  SessionProgress.

crates/agent-drift-analyzer/src/checkpoint/export.rs
  Add progress summary distribution and compact per-checkpoint progress line.

crates/agent-drift-analyzer/src/lib.rs
  Re-export public progress DTOs alongside existing checkpoint DTOs if sentinel tests construct
  checkpoints directly.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Schema v0.6 requiredness and deterministic builder-level progress tests.

crates/agent-drift-analyzer/tests/export_bundle.rs
  Summary rendering tests for progress distribution and checkpoint-local progress.

crates/agent-drift-analyzer/tests/end_to_end.rs
  v0.6 artifact stability.

crates/agent-drift-analyzer/tests/progress_acceptance.rs
  Dedicated R5 semantic wall so the legacy R2 `tests/acceptance_fixtures.rs` corpus remains
  legacy-stability-only unless intentionally widened on purpose.

crates/agent-drift-sentinel/src/input.rs
  Replay support for v0.6 and required session_progress contract checks.

crates/agent-drift-sentinel/src/live_input.rs
  Live support for v0.6 and required session_progress contract checks.

crates/agent-drift-sentinel/src/operator_surface.rs
  Compact Progress line after Archetype line. Presentation only.

crates/agent-drift-sentinel/tests/replay_input.rs
crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs
crates/agent-drift-sentinel/tests/operator_surface.rs
crates/agent-drift-sentinel/tests/live_end_to_end.rs
  v0.6 compatibility and presentation parity.

docs/specs/r5/DESIGN-r5-*.md
  Canonical design docs for contract, attempts/signatures, windows/frontiers, archetype rules,
  delegation guardrails, and validation.

docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
  Fixture labeling authority for R5.
```

## Code Style

Use the existing serde/enum style in `checkpoint/schema.rs`:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
    Advancing,
    Mixed,
    Stalled,
    Regressing,
    InsufficientEvidence,
}
```

Public DTOs should stay compact:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionProgress {
    pub status: ProgressStatus,
    pub dimension: ProgressDimension,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<ProgressSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_evidence: Vec<EvidenceRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counter_evidence: Vec<EvidenceRef>,
}
```

Checkpoint construction should remain readable:

```rust
let session_archetype = build_session_archetype(analysis);
let session_progress = build_session_progress(analysis, &session_archetype);

Checkpoint {
    schema_version: "v0.6".to_string(),
    session_archetype: Some(session_archetype),
    session_progress: Some(session_progress),
    ..
}
```

Conventions:

1. serialized enum values use `snake_case`,
2. public checkpoint fields remain backward-compatible at serde shape level,
3. version-specific requiredness is enforced after raw JSON parse,
4. internal diagnostic parsers should be conservative and confidence-bearing,
5. evidence reasons should describe observations, not causal certainty,
6. progress logic should be deterministic and side-effect-free,
7. presentation formatting stays out of analyzer scoring.

## Testing Strategy

### Analyzer Unit / Integration Tests

1. DTO and schema tests:
   - v0.2-v0.5 checkpoints without `session_progress` still deserialize/load where appropriate,
   - v0.6 without `session_progress` fails closed,
   - v0.6 with progress loads and round-trips deterministically.
2. Attempt/signature tests:
   - command/output pairing,
   - exit-code parsing,
   - role classification for cargo/npm/pnpm/pytest/vitest/replay/apply_patch,
   - diagnostic canonicalization strips volatile text,
   - exact/strong/frontier-related matching works.
3. Progress rule tests:
   - troubleshooting advanced/stalled/regressing/insufficient,
   - planning convergence/meander,
   - implementation verification-wall advance/stall/regression,
   - closeout narrowing/reopening,
   - delegation caps.
4. Export tests:
   - summary progress distribution,
   - per-checkpoint progress line.
5. Acceptance fixture tests:
   - small frozen corpus with expected status/dimension/confidence/evidence behavior.

### Sentinel Tests

1. replay input accepts v0.6 and rejects missing progress for v0.6,
2. live input accepts v0.6 and rejects missing progress for v0.6,
3. legacy v0.2-v0.5 compatibility remains intact,
4. operator surface renders the compact `Progress:` line,
5. replay/live parity remains intact for matching v0.6 checkpoint.

## Boundaries

### Always Do

- Keep R5 analyzer-owned and deterministic.
- Preserve legacy checkpoint compatibility.
- Reuse public `EvidenceRef` for public progress evidence.
- Emit `insufficient_evidence` when the trace does not exercise the relevant target.
- Cap confidence under opaque delegation.
- Keep `session_progress` additive and evidence-backed.
- Update docs before implementation details drift.
- Run focused analyzer and sentinel tests before calling the packet done.

### Ask First

- Adding external dependencies.
- Changing `CompactionRow` or upstream compactor schema.
- Making diagnostic signatures public checkpoint schema.
- Adding `progress_debug.jsonl` as a required artifact rather than optional/debug.
- Increasing planning-convergence confidence beyond medium in the first landing.
- Changing sentinel scheduler/adjudication behavior.

### Never Do

- Retune `dead_end_thrash` inside R5.
- Treat generic `ToolOutput` as scorer failure evidence again.
- Infer opaque child implementation or troubleshooting progress.
- Use LLM calls in the analyzer.
- Replace typed progress status with a scalar reward.
- Make the operator surface the source of semantic truth.
- Validate only through synthetic fixtures when claiming semantic honesty.

## Success Criteria

R5 is done when:

1. analyzer checkpoints emit `schema_version: "v0.6"`,
2. every v0.6 checkpoint includes `session_progress`,
3. v0.6 validation requires `turn_context`, `session_archetype`, and `session_progress`,
4. v0.2-v0.5 legacy checkpoints still load through replay/live paths,
5. analyzer summary includes progress distribution and per-checkpoint progress lines,
6. sentinel operator surface displays compact `Progress:` line after `Archetype:`,
7. troubleshooting fixtures distinguish repeated failure from frontier advancement,
8. planning fixtures distinguish convergence from meandering discussion at conservative confidence,
9. implementation fixtures distinguish verification-wall progress from churn,
10. closeout fixtures distinguish narrowing proof from scope reopening,
11. delegated opaque-parent fixtures do not claim child progress,
12. no drift scorer or scheduler behavior is retuned,
13. focused analyzer and sentinel tests pass,
14. the fixture manifest documents expected labels and evidence for the implemented cases.

## Locked After Packet R5-0

The DESIGN docs, plan, tasks, and fixture manifest now treat the decisions above as authoritative
inputs for `R5-1+`. No open design question remains in this spec that should block the first
implementation packet.
