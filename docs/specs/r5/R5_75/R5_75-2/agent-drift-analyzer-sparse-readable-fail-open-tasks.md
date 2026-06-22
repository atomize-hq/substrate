# Tasks: Agent Drift Analyzer Sparse Readable Session Fail-Open (R5.75-2)

Status: draft task ledger created on 2026-06-21 from the `R5.75-2` SPEC/PLAN in this directory, after
reproducing the abort on the live crate. This is the active implementation queue for `R5.75-2`.

Packet prerequisite rule: this packet names `R5.75-1` as landed. Verify it in live code/tests before
editing (it is, as of commit `68216bf02` — the structured-objective anchoring fix). If a named
prerequisite were missing, stop and report it instead of compensating inside this packet.

## R5.75-2.0: Docs Lock

- [ ] Task R5.75-2.0.1: Commit the SPEC/PLAN/TASKS family.
  - Acceptance: `docs/specs/r5/R5_75/R5_75-2/` contains the spec, plan, and this tasks ledger, and they
    record the corruption-vs-sparse split, the reuse of `InsufficientEvidence`/`Confidence::Low`, and
    the additive-only (no schema bump) boundary.
  - Verify: Manual review against the `R5.75-2` packet in `docs/specs/r5/R5_75/MAP.md` and live `input.rs`.
  - Files:
    - `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md`
    - `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md`
    - `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md`

## R5.75-2.1: Characterize Downstream Behavior (Investigation, No Committed Code)

- [ ] Task R5.75-2.1.1: Determine what the analysis path emits for a de-aborted sparse session.
  - Acceptance: a recorded finding (in this ledger) of whether `analyze_loaded_bundle` /
    `checkpoint_analyses` already emits ≥1 conservative (`InsufficientEvidence` / low-confidence)
    checkpoint for `f47b81f39f2495dd` when `validate_surface` does not abort, or whether it emits
    nothing / over-claims. Resolves Spec Open Question 1 and sizes Task R5.75-2.3.1.
  - Verify: local experiment (temporary `validate_surface` relax or a hand-built sparse bundle unit
    test) run against the repro; the experiment is reverted — no code from this task is committed.
  - Files:
    - (read-only) `crates/agent-drift-analyzer/src/lib.rs`,
      `crates/agent-drift-analyzer/src/checkpoint/mod.rs`

## R5.75-2.2: Split validate_surface (Corruption Hard-Fail vs Sparse Fail-Open)

- [ ] Task R5.75-2.2.1: Stop aborting on the sparse-but-readable conditions.
  - Acceptance: `validate_surface` no longer returns `Err` for the path-hint
    (`truth_artifact_hints`) and tool-payload (`working_set_hints` / `tool_argument_json`) conditions
    — each independently, neither gated on the other; it returns `Ok(AnalyzerSurface { … })` with those
    flags `false`. `repetition_preserved`, `stable_row_refs`, `literal_objective_rows`, and all upstream
    `InputError` variants still return their existing errors verbatim. `AnalyzerSurface` shape is
    unchanged (no new fields, no schema bump).
  - Verify:
    - `cargo test -p agent-drift-analyzer --test input_contract -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/input.rs`

## R5.75-2.3: Guarantee The Conservative Checkpoint

- [ ] Task R5.75-2.3.1: Ensure a sparse session yields one conservative checkpoint.
  - Acceptance: a de-aborted sparse session emits ≥1 checkpoint with `ProgressStatus::InsufficientEvidence`
    (or equivalently low-confidence), never a troubleshooting/strong-progress posture. If R5.75-2.1
    showed the pipeline already self-conservatizes, this task is assertion-only (no source change); else
    `analyze_loaded_bundle` reads `bundle.surface` to cap confidence using existing surfaces only.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/lib.rs` (only if the probe showed over-claiming)

## R5.75-2.4: Regressions

- [ ] Task R5.75-2.4.1: Input-contract split regression (both sparsity axes + corruption).
  - Acceptance: `tests/input_contract.rs` proves (a) the tool-payload axis — a readable bundle with
    objective rows + path hints but zero parseable tool-call payloads (the `f47b81f39f2495dd` shape) —
    returns `Ok` from `load_bundle`; (a') the path-hint axis — a readable bundle with an objective row
    but no path hints and no tool calls (the conceptual-ask shape) — also returns `Ok`; and (b) a
    corrupt bundle (unstable/duplicate row refs or broken dedupe) still returns the exact existing
    `InputError` variant. The split must be provable, not just the relaxation, and each sparsity axis is
    pinned independently.
  - Verify: `cargo test -p agent-drift-analyzer --test input_contract -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/input_contract.rs`

- [ ] Task R5.75-2.4.2: Conservative-checkpoint regression.
  - Acceptance: `tests/checkpoints.rs` proves a sparse readable session (steer ask + pasted `<skill>`
    body + no tool calls, the minimized `f47b81f39f2495dd` shape) emits exactly one checkpoint that is
    `InsufficientEvidence`, with a `structured_objective` anchored to the steer ask and weak fields
    (`success_conditions` / `deliverables` / `target`) unknown — never anchored to the `<skill>` body.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.75-2.5: Smoke And Closeout

- [ ] Task R5.75-2.5.1: Named smoke — adapted fail-open + native control.
  - Acceptance: the adapted repro `f47b81f39f2495dd` runs compactor→analyzer→sentinel without aborting
    and emits ≥1 conservative checkpoint; the native control `019eb430-6f9a-7a03-9a63-cb451b654795`
    still produces normal output.
  - Verify: the manual smoke pipeline from the SPEC Commands section, run for both sessions; inspect
    `summary.md` (objective line conservative/unknown) and `checkpoints.jsonl`.
  - Files:
    - none (smoke only; outputs under `target/r5_75-smoke/R5.75-2/`)

- [ ] Task R5.75-2.5.2: Full + touched sentinel walls, then MAP status update.
  - Acceptance: the full analyzer wall and the touched sentinel spot-checks are green; the `R5.75-2`
    packet in the MAP is updated (promotion status + routing note pointing to `R5.75-3` as the next
    active seam).
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - `cargo test -p agent-drift-sentinel warning_policy -- --nocapture`
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files:
    - `docs/specs/r5/R5_75/MAP.md`

## Deferred / Ask-First

- [ ] Task R5.75-2.X.1: Relax `literal_objective_rows` to fail-open with a fully-unknown objective.
  - Acceptance: only after a real readable-but-objectiveless repro appears and the change is approved;
    keeps the conservative-checkpoint contract intact.
  - Verify: to be defined when approved.
  - Files:
    - `crates/agent-drift-analyzer/src/input.rs`

- [ ] Task R5.75-2.X.2: Thread `AnalyzerSurface` sparsity into richer downstream behavior beyond capping
      confidence.
  - Acceptance: only if a later packet proves a need; must stay additive and must not pre-empt the
    `R5.75-6` objective-faithfulness work or the deferred consumer migration.
  - Verify: to be defined when approved.
  - Files:
    - `crates/agent-drift-analyzer/src/lib.rs`
