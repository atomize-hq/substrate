# Plan: Agent Drift Analyzer Session Progress R5

Status: Packet R5-7 landed on 2026-06-10, closing the dedicated acceptance-fixture wall after the already-landed R5-0 through R5-6 packets.

Current family state on 2026-06-10:

- Packets `R5-0` through `R5-6` were already landed before this packet.
- Packet `R5-7` is now the landed dedicated acceptance-fixture wall for the family.
- The bounded semantic acceptance bar is met with a committed progress corpus that includes annotated
  real-rollout cases plus supporting bundle-shaped cases across the core progress dimensions.

## Objective

Implement `R5` as an analyzer-owned `session_progress` layer under checkpoint schema `v0.6`, with
sentinel replay/live compatibility and compact operator presentation.

The implementation must follow the R5 spec and DESIGN docs:

- `DESIGN-r5-session-progress-contract.md`
- `DESIGN-r5-command-attempt-and-diagnostic-signature.md`
- `DESIGN-r5-progress-window-and-frontier-model.md`
- `DESIGN-r5-archetype-progress-rules.md`
- `DESIGN-r5-delegation-progress-guardrails.md`
- `DESIGN-r5-validation-and-rollout-protocol.md`
- `agent-drift-analyzer-session-progress-r5-fixtures.md`

## Implementation Principles

1. Add the public `v0.6` checkpoint field before using it in sentinel.
2. Build internal attempt/signature seams before writing broad progress heuristics.
3. Land troubleshooting frontier first because it is the most machine-checkable.
4. Keep planning/implementation/closeout rules conservative and evidence-backed.
5. Preserve all legacy schema compatibility.
6. Do not touch drift scorer weights or scheduler policy.
7. Keep each packet small enough to review.

## Dependency Graph

```text
R5 docs lock
  -> public schema v0.6
    -> analyzer checkpoint construction
      -> attempts/signatures
        -> progress engine
          -> analyzer summary
            -> sentinel replay/live support
              -> operator rendering
                -> acceptance fixtures / real-rollout check
```

Sentinel v0.6 work depends on analyzer public DTOs being exported from `agent-drift-analyzer`.
Progress rules depend on attempts/signatures. Summary/operator rendering depends on the public
`SessionProgress` shape.

## Packet Split

## R5-0: Docs Lock And Fixture Manifest

### Scope

- Add canonical DESIGN docs.
- Add R5 spec/plan/tasks.
- Add fixture manifest.
- Explicitly record non-goals, schema/versioning direction, packet split, and fixture-manifest
  direction.

### Files

```text
docs/specs/r5/DESIGN-r5-*.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
```

### Verification

Manual doc review only. No Rust code, test, or fixture-directory changes are part of `R5-0`.

## R5-1: Public Schema Skeleton And Compatibility

### Scope

- Add public R5 DTOs to `checkpoint/schema.rs`.
- Add optional `Checkpoint.session_progress`.
- Make `v0.6` require `turn_context`, `session_archetype`, and `session_progress`.
- Keep `v0.2` through `v0.5` loadable.
- Re-export public DTOs from analyzer `lib.rs` and `checkpoint/mod.rs`.
- Temporarily build a minimal `SessionProgress` so analyzer can emit `v0.6` before richer rules
  land.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/src/lib.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

### Notes

The minimal progress builder may emit `insufficient_evidence` with low confidence. That is
acceptable only inside R5-1; R5-3/R5-4 replace it with real progress assessment.

## R5-2: Attempt Builder Foundation

### Scope

- Add internal `checkpoint/attempt.rs`.
- Pair `ToolCall` rows with immediate output/error rows.
- Parse exit codes.
- Classify progress-specific command roles.
- Extract verification target scope from command text and existing command-observation paths.
- Add focused tests for command/output pairing and role classification.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/src/checkpoint/attempt.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

### Acceptance

- `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt --check`, `pytest`, `pnpm test`,
  `vitest`, `replay`, `apply_patch`, and `spawn_agent`-style rows classify deterministically.
- A command with `Exit code: 1` paired output becomes failed.
- A command with `Exit code: 0` paired output becomes clean.
- Ambiguous pairing becomes unknown rather than panic or false confidence.

## R5-3: Diagnostic Signature And Frontier Matching

### Scope

- Add internal `checkpoint/diagnostics.rs`.
- Normalize paired output and compute stable diagnostic payload hashes.
- Parse initial failure classes and fail counts for common cargo/test/replay patterns.
- Implement exact, strong-fuzzy, frontier-related, weak-related, and unrelated matching.
- Implement edit-overlap between diagnostic scope and intervening edit attempts.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/attempt.rs
crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

### Acceptance

- Volatile output changes do not produce different signatures.
- Compile failure -> test/assertion failure on same target is frontier-related advancement.
- Same normalized signature repeated is exact/strong repeated evidence.
- Fail count reduction is parsed for at least cargo test and pytest-like shapes.
- Edit overlap distinguishes overlapping versus unrelated edits.

## R5-4: Progress Engine And Archetype Rules

### Scope

- Add internal `checkpoint/progress.rs`.
- Build `SessionProgress` from `CheckpointAnalysis`, R4 `SessionArchetype`, attempts,
  signatures, task-frame delta, recovery/repetition state, and delegation context.
- Implement troubleshooting frontier first.
- Add conservative planning, implementation, and closeout progress rules.
- Add delegation caps and `ParentVisibleOrchestration` fallback.
- Replace temporary R5-1 progress builder.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/src/checkpoint/progress.rs
crates/agent-drift-analyzer/src/checkpoint/attempt.rs
crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Acceptance

- Required fixture matrix rows in `agent-drift-analyzer-session-progress-r5-fixtures.md` have
  corresponding tests or explicit deferral notes.
- Non-`insufficient_evidence` statuses carry supporting evidence.
- Delegated opaque-parent cases cap confidence and avoid child progress claims.
- R5 does not change drift-score expectations.

## R5-5: Analyzer Export Summary

### Scope

- Add progress distributions to analyzer `summary.md`.
- Add per-checkpoint compact progress lines.
- Optionally add debug-only attempt/signature rendering if implementation scope allows.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/export.rs
crates/agent-drift-analyzer/tests/export_bundle.rs
crates/agent-drift-analyzer/tests/end_to_end.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

### Acceptance

- Summary includes progress status distribution.
- Summary includes progress dimension distribution.
- Each checkpoint line includes compact progress inspection.
- Existing turn-context, delegation, and archetype lines remain intact.

## R5-6: Sentinel v0.6 Compatibility And Operator Surface

### Scope

- Add `v0.6` to replay and live supported schemas.
- Require `session_progress` for `v0.6` in replay and live validation.
- Keep `v0.2` through `v0.5` compatibility.
- Render compact `Progress:` line after `Archetype:`.
- Keep presentation-only; no scheduler/adjudication changes.

### Files

```text
crates/agent-drift-sentinel/src/input.rs
crates/agent-drift-sentinel/src/live_input.rs
crates/agent-drift-sentinel/src/operator_surface.rs
crates/agent-drift-sentinel/tests/replay_input.rs
crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs
crates/agent-drift-sentinel/tests/operator_surface.rs
crates/agent-drift-sentinel/tests/live_end_to_end.rs
```

### Verification

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

### Acceptance

- v0.6 loads in replay and live.
- v0.6 missing `session_progress` fails closed.
- v0.5 missing `session_progress` still loads.
- Operator block renders progress without changing posture/disposition.
- Replay/live surfaces match for the same v0.6 checkpoint.

## R5-7: Acceptance Fixture Wall And Final Verification

### Scope

- Add or extend committed analyzer acceptance fixtures for progress semantics.
- Use a dedicated progress-acceptance harness/corpus as the R5 semantic wall; keep the frozen R2
  `tests/fixtures/acceptance` wall legacy-stability-only unless intentionally widened.
- Include at least one annotated real-rollout case for the bounded semantic acceptance claim;
  realistic bundle-shaped cases may extend supporting coverage across the core dimensions.
- Allow the narrow delegated guardrail follow-through in
  `crates/agent-drift-analyzer/src/checkpoint/progress.rs` so the
  `parent_visible_orchestration` case records delegated child-opaque limiting evidence in
  `counter_evidence` without widening R5 into positive child-progress semantics.
- Update fixture manifest with actual fixture locations and deferrals.
- Run focused and full test walls.
- This is the only packet allowed to claim bounded semantic acceptance / real-rollout proof for
  the `R5` family.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/progress.rs
crates/agent-drift-analyzer/tests/progress_acceptance.rs
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**
docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
```

### Verification

```bash
cargo fmt --all -- --check
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture # dedicated R5 semantic wall
cargo test -p agent-drift-analyzer acceptance_fixtures -- --nocapture # legacy-stability-only frozen R2 wall
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

### Acceptance

- A bounded semantic corpus proves R5 outputs are not only synthetic.
- The landed corpus shape is 3 annotated real-rollout cases (`019e899c-453f-71f2-a99d-155848c7b081`, `019e940c-a91b-7fe0-a967-b0bdd595b581`, `019e8b42-42bd-7b10-baae-3265edb65f4b`) plus 3 committed bundle-shaped support cases (`synthetic-planning-advancing`, `synthetic-implementation-advancing`, `synthetic-parent-visible-opaque`).
- Known legacy acceptance fixtures remain stable.
- No drift scorer retuning sneaks into R5.
- Docs match implemented behavior.
- On 2026-06-10, the full analyzer and sentinel test walls passed; `cargo fmt --all -- --check` remained red only because tracked preexisting formatting drift still exists in `crates/agent-drift-analyzer/src/checkpoint/export.rs`, `crates/agent-drift-analyzer/src/checkpoint/mod.rs`, and `crates/agent-drift-analyzer/tests/export_bundle.rs` outside the Packet R5-7 diff.

## Parallelization Notes

Can proceed in parallel after R5-1:

1. attempt role classification tests and diagnostic parser tests,
2. sentinel fixture update scaffolding using public DTOs,
3. fixture manifest refinement.

Must be sequential:

1. public DTO before sentinel v0.6 construction,
2. attempts before diagnostic signatures,
3. signatures before troubleshooting frontier confidence,
4. progress engine before analyzer summary/operator formatting,
5. analyzer v0.6 before replay/live compatibility proof.

## Risks And Mitigations

| Risk | Mitigation |
|---|---|
| R5 becomes a scorer retune | Explicitly forbid changes to `scoring/*` except tests proving unchanged behavior. |
| Public schema overexpands | Keep signatures/debug anchors internal; public DTO uses existing `EvidenceRef`. |
| Diagnostic parsing overclaims | Use parser confidence and `insufficient_evidence`. |
| Planning convergence becomes prose magic | Use structural proxies only; cap confidence conservatively. |
| Delegation gets over-modeled | Use guardrails and defer real parent/child semantics to R7. |
| Sentinel compatibility duplicates grow | Accept duplication for R5; R8 owns consolidation. |
| Synthetic tests hide semantic bugs | Add at least one annotated real-rollout acceptance case plus supporting bundle-shaped cases. |

## Verification Checkpoints

1. After R5-1: schema round-trip and v0.6 requiredness pass.
2. After R5-2: command/output pairing and role classification pass.
3. After R5-3: signature matching and edit-overlap tests pass.
4. After R5-4: progress fixture matrix passes in analyzer tests.
5. After R5-5: summary output tests pass.
6. After R5-6: sentinel replay/live v0.6 tests pass.
7. After R5-7: full analyzer and sentinel walls pass.

## Non-Goals For This Plan

- No R6 scorer cutover.
- No R7 child/parent linkage.
- No R8 sentinel interpretation consolidation.
- No learned/LLM progress monitor.
- No required upstream compactor schema changes.
