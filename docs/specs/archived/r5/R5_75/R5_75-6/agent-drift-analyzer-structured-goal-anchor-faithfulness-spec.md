# Spec: Agent Drift Analyzer Structured Goal Anchor Faithfulness (R5.75-6)

Status: Task `R5.75-6.0.1` docs lock finalized on 2026-06-24 after re-reading the live `R5.75-6` packet in
`docs/specs/r5/R5_75/MAP.md`, the Issue 7 diagnosis in
`docs/specs/r5/R5_75/structured-objective-bug-map.md`, the current checkpoint/objective bridge in
`crates/agent-drift-analyzer/src/checkpoint/mod.rs`, the current task-frame consumer in
`crates/agent-drift-analyzer/src/inference/mod.rs`, and the current migration boundary in
`docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md`.

## Assumptions I'm Making

1. `R5.75-5` is promoted history, and `R5.75-6` is now the only active pre-`R6` seam exactly as
   routed in `docs/specs/r5/R5_75/MAP.md`.
2. This packet is a **bounded analyzer-local bridge packet**, not the full structured-native
   consumer migration. `TaskFrame` / `context/working_set.rs` / `checkpoint/progress.rs`
   migration onto typed structured fields or `comparison_key` remains a later phase unless this
   packet proves a tiny additive bridge is unavoidable.
3. The current structured goal anchor in `context.objective.structured` is the semantic authority
   when it exists with grounded `Goal` evidence; legacy narrowing is compatibility fallback only and
   must not replace a correct goal with an optional nit or closeout bullet.
4. The real repro `019eddaa-e8b2-74b2-9f45-e4ce17aaab55` and the earlier anchored-review witness
   `019eb47f-0118-7e90-8291-30a1fb93769e` are the minimum required native smoke sessions for this
   packet, and the full named `R5.75-5` smoke set must still pass before promotion.
5. The user asked for the SPEC/PLAN/TASKS triplet only, so packet prompts are intentionally out of
   scope for this docs pass.

If any assumption above is wrong, correct it before implementation starts.

## Objective

Make the **effective checkpoint objective** faithful to the already-correct structured goal anchor so
that downstream drift reasoning no longer keys off a legacy-narrowed wrong imperative line before
`R6` scorer work begins.

Primary users:

1. engineers reading `task_frame.objective` and checkpoint exports during replay, smoke, and
   downstream debugging;
2. reviewers validating that the analyzer's public compatibility surface still matches the real user
   ask when structured extraction has already anchored correctly; and
3. future `R6` scorer work, which must inherit an objective surface that is no longer vulnerable to
   optional-nit / closeout-line takeover.

Success means:

1. when a grounded structured goal span exists, the effective objective seen by checkpoint/task-frame
   consumers stays aligned to that goal instead of a later optional reviewer nit or closeout bullet;
2. the fix stays additive and analyzer-local, without widening into the deferred full
   structured-native consumer migration;
3. the named wrong-imperative repro (`019eddaa-...`) stops narrowing to
   `add extra task-local grep checks for transition routing / outcome/final-marker meaning` and
   instead reflects the real validate/readiness ask;
4. previously landed packet behavior remains intact, especially `R5.75-3` delegated stability,
   `R5.75-4` zero-verifier anti-flap conservatism, and the `R5.75-1` anchored-review semantics; and
5. the full analyzer wall, sentinel spot-checks, and full named smoke rerun are green before `R5.75`
   is declared complete.

## Scope Classification

- **In scope**
  - change the checkpoint compatibility bridge so a grounded structured goal anchor wins over legacy
    narrowing when both are present
  - demote optional-nit / not-taken / review-closeout imperative bullets so they cannot become the
    effective objective after a real ask has already anchored the structured goal
  - preserve or tighten fallback behavior when no structured goal is present, rather than removing
    legacy narrowing outright
  - add focused regression coverage in `crates/agent-drift-analyzer/tests/checkpoints.rs` for the
    wrong-imperative follow-up shape and any required fallback guard
  - rerun the downstream acceptance and smoke ladders that prove earlier `R5.75` packets still hold
- **Out of scope**
  - broad structured-native consumer migration (`TaskFrame` coexistence schema changes,
    `context/working_set.rs` migration, `checkpoint/progress.rs` comparison redesign)
  - `R6` scorer retuning, `dead_end_thrash` redesign, or other score-policy changes
  - new adapted-external corpus work beyond re-running the already adopted `R5.75-5` smoke set
  - packet prompts in this docs pass
  - opportunistic refactors outside the objective-faithfulness seam

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Primary implementation seams:
  - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - `crates/agent-drift-analyzer/src/inference/mod.rs`
  - `crates/agent-drift-analyzer/src/context/objective.rs` (structured goal authority only)
- Primary regression surfaces:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
- Promotion authority docs:
  - `docs/specs/r5/R5_75/MAP.md`
  - `docs/specs/r5/R5_75/structured-objective-bug-map.md`
  - `docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md`

No new runtime dependency, no schema-version bump, and no broader consumer migration belong in this
packet unless a tiny additive bridge is proven necessary and explicitly approved.

## Commands

Focused checkpoint regression wall:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Downstream guardrails:

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Sentinel spot-checks required before promotion:

```bash
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Targeted native repro smoke for the wrong-imperative session:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="019eddaa-e8b2-74b2-9f45-e4ce17aaab55"
export SMOKE_ROOT="target/r5_75-smoke/R5.75-6/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Anchored-review guard smoke:

```bash
export CODEX_HOME="$HOME/.codex"
export SESSION_ID="019eb47f-0118-7e90-8291-30a1fb93769e"
export SMOKE_ROOT="target/r5_75-smoke/R5.75-6/$SESSION_ID"
export COMPACTOR_OUT="$SMOKE_ROOT/compactor"
export ANALYZER_OUT="$SMOKE_ROOT/analyzer"

rm -rf "$SMOKE_ROOT"

cargo run -p agent-session-compactor -- \
  --codex-home "$CODEX_HOME" \
  --session-id "$SESSION_ID" \
  --output-dir "$COMPACTOR_OUT"

cargo run -p agent-drift-analyzer -- \
  --input-dir "$COMPACTOR_OUT" \
  --output-dir "$ANALYZER_OUT"

cargo run -p agent-drift-sentinel -- \
  --checkpoint-dir "$ANALYZER_OUT"

sed -n '1,80p' "$ANALYZER_OUT/summary.md"
sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"
```

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  Landing-order authority. Declares R5.75-6 as the active seam and defines the bounded Issue 7 scope.

docs/specs/r5/R5_75/structured-objective-bug-map.md
  Root diagnosis reference for the wrong-imperative / legacy-narrowing failure and its named repro.

docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md
docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md
docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md
  This packet's implementation authority.

crates/agent-drift-analyzer/src/context/objective.rs
  Structured objective extraction and compatibility rendering authority. This packet should defer to
  its grounded goal anchor rather than re-deciding the mission from raw text.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Current compatibility overlay (`narrowed_objective_summary`, `normalized_objective_text`) that can
  still overwrite a good structured goal with the wrong imperative line.

crates/agent-drift-analyzer/src/inference/mod.rs
  Current `TaskFrame` consumer that reads `context.objective.text`; this packet must make that text
  faithful without widening into the full migration.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Best home for the minimized wrong-imperative regression(s) and bridge-fallback guards.

crates/agent-drift-analyzer/tests/progress_acceptance.rs
  Downstream guardrail suite to ensure earlier R5.75 progress semantics do not regress.

target/r5_75-smoke/R5.75-6/<session-id>/
  Promotion reruns for the named native repros and the full carried-forward smoke set.
```

## Code Style

Preserve the additive bridge style already used in `ObjectiveSummary`: prefer the grounded
structured authority first, then apply compatibility fallback only when that fallback does not erase
correct semantics.

```rust
if structured_goal_anchor_is_grounded(&context.objective) {
    // Preserve the structured-goal-compatible display surface.
    context.objective = context.objective.clone();
} else if let Some(objective) = narrowed_objective_summary(&window.compact_rows) {
    context.objective = context.objective.with_compatibility_display_from(&objective);
}
```

Conventions to preserve:

- keep the change explicit and local to the compatibility bridge
- prefer helper names that describe the semantic guard (`grounded goal anchor`, `fallback only`)
- preserve `comparison_key` and the structured sidecar; do not let a display fallback rewrite them
- keep fallback behavior deterministic when the structured sidecar is absent or weak
- encode the real repro in tests instead of relying on prose-only rationale

## Testing Strategy

- **Primary regression surface**
  - add at least one minimized checkpoint regression for the `019eddaa` wrong-imperative shape
  - assert both the effective objective string and the exported structured objective remain aligned
    to the real ask
- **Bridge safety checks**
  - keep or add a case proving legacy fallback still works when no grounded structured goal exists
  - keep the earlier anchored-review regression green so `R5.75-1` semantics do not regress
- **Downstream regression wall**
  - rerun `progress_acceptance` because objective changes can affect progress continuity indirectly
  - rerun the full analyzer suite to catch any closeout/review/no-code or export regressions
- **Operator/replay compatibility guard**
  - rerun sentinel `warning_policy` and `live_end_to_end` because this packet changes the effective
    objective exposed to downstream consumers
- **Manual smoke**
  - rerun `019eddaa-...` and `019eb47f-...`
  - rerun the full named `R5.75-5` native + adapted smoke set to prove no earlier packet regresses

## Boundaries

- **Always do**
  - verify `R5.75-5` is promoted before editing
  - keep the packet analyzer-local and bounded to effective-objective faithfulness
  - preserve the structured sidecar and `comparison_key` as semantic authority
  - run the checkpoint wall, progress wall, full analyzer wall, sentinel spot-checks, and manual smoke
    before promotion
- **Ask first**
  - schema-version bumps or public `TaskFrame` contract changes
  - migration work in `context/working_set.rs` or `checkpoint/progress.rs`
  - adding new adapted fixtures or widening the acceptance corpus beyond what this bug requires
  - any fix that appears to require `R6` scorer changes instead of the bounded bridge repair
- **Never do**
  - silently widen into the full structured-native consumer migration
  - retune scorer policy or progress heuristics as a shortcut for the wrong-objective bug
  - weaken the manual smoke gate or skip the carried-forward `R5.75-5` smoke set
  - let optional reviewer nits become the effective objective once a grounded structured goal exists

## Success Criteria

`R5.75-6` is complete only when all of the following are true:

1. on the named repro `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`, the effective checkpoint objective no
   longer narrows to the optional reviewer nit and instead reflects the actual validate/readiness ask;
2. on the anchored-review witness `019eb47f-0118-7e90-8291-30a1fb93769e`, the effective objective
   still matches the correct evaluate/review ask and does not regress to boilerplate or path noise;
3. `crates/agent-drift-analyzer/tests/checkpoints.rs` contains durable regression coverage for the
   bounded wrong-imperative failure and any necessary fallback guard;
4. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green;
5. `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` is green;
6. `cargo test -p agent-drift-analyzer -- --nocapture` is green;
7. `cargo test -p agent-drift-sentinel warning_policy -- --nocapture` and
   `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture` are green; and
8. the full named native + adapted smoke rerun under `target/r5_75-smoke/R5.75-6/` proves
   `R5.75-1` through `R5.75-5` still hold, allowing `R5.75` to close honestly and `R6` to open.

## Open Questions

1. Can the packet stay entirely inside `checkpoint/mod.rs` by changing the compatibility overlay rule,
   or does a tiny additive `TaskFrame` bridge become necessary to keep downstream consumers faithful?
2. Is a single minimized checkpoint regression sufficient for the wrong-imperative repro, or should a
   second guard explicitly prove fallback behavior when no grounded structured goal exists?
3. If the fix needs to consult structured evidence more directly, should the helper live in
   `checkpoint/mod.rs` or be exposed from `context/objective.rs` to keep the grounding rule shared?
