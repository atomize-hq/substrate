# Spec: Agent Drift Analyzer Zero-Verifier Anti-Flap Gate (R5.75-4)

Status: draft spec created on 2026-06-23 after `R5.75-3` was promoted in `docs/specs/r5/R5_75/MAP.md`, using the live `R5.75` map, the root `structured-objective-bug-map.md`, and the current analyzer seams in `crates/agent-drift-analyzer/src/checkpoint/progress.rs`, `crates/agent-drift-analyzer/tests/checkpoints.rs`, and `crates/agent-drift-analyzer/tests/progress_acceptance.rs`.

## Assumptions I'm Making

1. `R5.75-3` is already promoted history and must stay intact; `R5.75-4` is the active packet exactly as stated in `docs/specs/r5/R5_75/MAP.md`.
2. This packet stays analyzer-local to `session_progress` classification. It must not widen into `R6` scorer retuning, `dead_end_thrash` redesign, compactor changes, replay/schema changes, or the deeper structured-native consumer migration.
3. The structured `primary_intent` produced by `R5.75-1` remains observational-only in this packet. If anti-flap behavior would require wiring structured state into `progress.rs`, that is a new packet decision, not an implicit part of `R5.75-4`.
4. Adapted sessions `097d97e914ca220f` and `da59436e63915185` are the packet's manual-smoke authority. Committed adapted fixture-family work still belongs to `R5.75-5`, not here.
5. The user asked for the SPEC/PLAN/TASKS triplet only, so packet prompts are intentionally out of scope for this docs pass.

If any assumption above is wrong, correct it before implementation starts.

## Objective

Prevent long browse/read/tool-output-heavy zero-verifier sessions from flapping into stronger troubleshooting or failure posture when the analyzer does not have decisive proof work, concrete source-edit progress, or explicit failure evidence.

Users of this packet are engineers and operators who read `session_progress`, `summary.md`, and `checkpoints.jsonl` to decide whether a session is truly troubleshooting vs. merely exploratory. Success means the analyzer stays boring and conservative by default on low-signal exploratory sessions, while still preserving real troubleshooting escalation when verifier-backed evidence exists.

## Scope Classification

- **In scope**
  - analyzer-local anti-flap gating in `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
  - fast checkpoint regressions for zero-verifier exploratory behavior
  - bounded `progress_acceptance` updates only if needed to encode the packet honestly without starting the adapted fixture-family packet early
  - manual smoke review of canonical zero-verifier exploratory witness `097d97e914ca220f` and mixed delegated/exploratory witness `da59436e63915185`
  - preserving the `R5.75-3` delegated parent-visible stability bar while applying the new conservative gate
- **Out of scope**
  - scorer retuning or `dead_end_thrash` redesign (`R6`)
  - adapted committed fixture-family expansion (`R5.75-5`)
  - structured-objective consumer wiring (`R5.75-6` / later migration work)
  - packet prompts in this docs pass
  - public schema/version changes

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Primary implementation seam:
  - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
- Primary fast-regression seam:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
- Bounded semantic acceptance seam:
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
- Manual smoke evidence roots:
  - `target/r5_75-smoke/R5.75-4/<session-id>/`
  - `target/ranga-validation/codex-home/`

No new dependencies, no schema bump, and no new top-level analyzer surface belong in this packet.

## Commands

Focused automated gates:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

Full analyzer closeout wall:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Useful seam inspection while implementing:

```bash
rg -n "assess_planning_progress|assess_troubleshooting_progress|insufficient_progress|verifier|plan_artifact_edits|parent_visible_orchestration|dead_end_thrash" \
  crates/agent-drift-analyzer/src/checkpoint/progress.rs \
  crates/agent-drift-analyzer/tests/checkpoints.rs \
  crates/agent-drift-analyzer/tests/progress_acceptance.rs
```

Adapted smoke harness for `097d97e914ca220f`:

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
export SESSION_ID="097d97e914ca220f"
export SMOKE_ROOT="target/r5_75-smoke/R5.75-4/$SESSION_ID"
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
sed -n '1,8p' "$ANALYZER_OUT/checkpoints.jsonl"
```

Adapted smoke harness for `da59436e63915185`:

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
export SESSION_ID="da59436e63915185"
export SMOKE_ROOT="target/r5_75-smoke/R5.75-4/$SESSION_ID"
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
sed -n '1,8p' "$ANALYZER_OUT/checkpoints.jsonl"
```

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  Landing-order authority. The R5.75-4 packet definition here decides the problem statement,
  smoke sessions, and promotion gate.

docs/specs/r5/R5_75/structured-objective-bug-map.md
  Root diagnosis context. Useful background, but not the packet's live execution authority.

docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md
docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md
docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md
  This packet's implementation authority.

crates/agent-drift-analyzer/src/checkpoint/progress.rs
  Live progress classification seam. This is where anti-flap gating must stay additive and bounded.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Fast synthetic and bundle-shaped regression wall for packet-local behavior.

crates/agent-drift-analyzer/tests/progress_acceptance.rs
  Bounded semantic acceptance harness. Use only for packet-local proof that does not prematurely start
  the adapted fixture-family packet.

target/r5_75-smoke/R5.75-4/<session-id>/
  Manual smoke outputs for canonical zero-verifier exploratory witness `097d97e914ca220f` and mixed delegated/exploratory witness `da59436e63915185` used to promote the packet.
```

## Code Style

Prefer narrow, evidence-driven guards over broad heuristic suppression. The packet should explain why it
stayed conservative, not merely mute the output.

```rust
if no_verifier_density
    && !has_concrete_source_edit_progress
    && !has_explicit_failure_evidence
{
    return insufficient_progress(
        ProgressDimension::PlanningConvergence,
        Some(progress_signal(/* explain the missing proof */)),
        None,
    );
}
```

Conventions to preserve:

- small additive conditionals instead of wide scoring rewrites
- explicit limiting/counter-evidence when confidence is capped
- preserve `parent_visible_orchestration` semantics from `R5.75-3` when delegation evidence exists
- do not let exploratory anti-flap logic erase real verifier-backed troubleshooting progression

## Testing Strategy

- **Fast packet-local regression wall:** `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - zero-verifier exploratory broad-scan / read-output shapes stay low-confidence and conservative
  - real verifier/failure evidence still allows troubleshooting advancement
  - delegated parent-visible cases do not regress while the anti-flap gate lands
- **Bounded semantic acceptance:** `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - only add or refine packet-local proof if needed
  - do not admit adapted committed fixtures yet; that belongs to `R5.75-5`
- **Full analyzer wall:** `cargo test -p agent-drift-analyzer -- --nocapture`
- **Manual smoke gate:** adapted sessions `097d97e914ca220f` and `da59436e63915185`
  - `097d97e914ca220f` must end boring/conservative, not troubleshooting-frontier or similar overclaim
  - `da59436e63915185` must preserve the conservative delegated parent-visible bar from `R5.75-3` while
    also not flapping into unrelated stronger failure posture elsewhere in the run

## Boundaries

- **Always**
  - keep the change analyzer-local to `progress.rs` and packet-local tests
  - preserve the shared R5.75 verification ladder and adapted smoke review
  - preserve the `R5.75-3` delegated-parent stability bar while adding the new anti-flap guard
  - record exact smoke expectations in the tasks ledger before claiming promotion
- **Ask first**
  - wiring structured `primary_intent` or `comparison_key` into `progress.rs`
  - widening into `dead_end_thrash` / scorer logic
  - admitting adapted committed fixtures or changing corpus shape beyond a bounded packet-local need
  - changing replay/export/schema contracts
- **Never**
  - silently absorb `R5.75-5` or `R5.75-6` scope into this packet
  - claim troubleshooting/implementation progress without verifier-backed or failure-backed evidence
  - let the anti-flap guard erase genuine verifier/failure signals that should still escalate
  - mark the packet promoted from green commands alone without inspecting smoke outputs

## Success Criteria

1. Long exploratory zero-verifier sessions default to `planning_convergence` / `insufficient_evidence`
   with low confidence unless decisive verifier/failure/edit signals appear.
2. `097d97e914ca220f` no longer surfaces troubleshooting-frontier or equivalent strong failure posture
   unless the smoke evidence shows real verifier/failure signals.
3. `da59436e63915185` preserves the `R5.75-3`-owned conservative delegated
   `parent_visible_orchestration` behavior where delegation evidence justifies it, and `R5.75-4`
   only preserves that bar while eliminating unrelated stronger failure posture elsewhere in the run.
4. Fast checkpoint regressions encode both sides of the boundary: conservative exploratory sessions stay
   conservative, while real verifier-backed troubleshooting still advances.
5. Any `progress_acceptance` update stays bounded and does not prematurely turn `R5.75-4` into the
   adapted external fixture-family packet.
6. `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`,
   `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`, and
   `cargo test -p agent-drift-analyzer -- --nocapture` are all green before promotion.

## Open Questions

1. Is one existing/native/synthetic `progress_acceptance` proof sufficient for this packet, or does the
   most honest implementation need a small additional bounded case?
2. Does `da59436e63915185` require a narrow guard that preserves its late delegated parent-visible block
   while suppressing earlier exploratory overclaim, or can the broader zero-verifier gate solve both
   adapted sessions cleanly?
3. If manual smoke shows the only remaining failure shape lives in scorer state rather than
   `session_progress`, should the packet stop and open `R6` follow-on work instead of widening here?
