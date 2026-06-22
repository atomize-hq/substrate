# Tasks: Agent Drift Analyzer Sparse Readable Session Fail-Open (R5.75-2)

Status: draft task ledger created on 2026-06-21 from the `R5.75-2` SPEC/PLAN in this directory, after
reproducing the abort on the live crate. This is the active implementation queue for `R5.75-2`.

Packet prerequisite rule: this packet names `R5.75-1` as landed. Verify it in live code/tests before
editing (it is, as of commit `68216bf02` — the structured-objective anchoring fix). If a named
prerequisite were missing, stop and report it instead of compensating inside this packet.

## R5.75-2.0: Docs Lock

- [x] Task R5.75-2.0.1: Commit the SPEC/PLAN/TASKS family. **Done** — committed in `92b0a930a` and
      revised in the Codex-review pass (per-session decision, `repetition_preserved` split,
      `InsufficientEvidence`-status contract, packet-boundary cleanups).
  - Acceptance: `docs/specs/r5/R5_75/R5_75-2/` contains the spec, plan, this tasks ledger, and the
    packet-prompts, and they record the corruption-vs-sparse split (incl. the `repetition_preserved`
    split), the `ProgressStatus::InsufficientEvidence` status contract, the per-session conservative
    decision, and the additive-only (no schema bump) boundary.
  - Verify: Manual review against the `R5.75-2` packet in `docs/specs/r5/R5_75/MAP.md` and live `input.rs`.
  - Files:
    - `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md`
    - `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md`
    - `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md`

## R5.75-2.1: Characterize Downstream Behavior (Investigation, No Committed Code)

- [ ] Task R5.75-2.1.1: Determine what the analysis path emits for a de-aborted sparse session.
  - Acceptance: a recorded finding (in this ledger) of whether `analyze_loaded_bundle` /
    `checkpoint_analyses` scores `f47b81f39f2495dd` **conservatively on its own**
    (`ProgressStatus::InsufficientEvidence`) when `validate_surface` does not abort, or whether it
    **over-claims**. The pipeline emits ≥1 window for any non-empty session, so "emits nothing" is not a
    realistic outcome — frame the finding as over-claim vs self-conservative. Resolves Spec Open
    Question 1 and sizes Task R5.75-2.3.1.
  - Verify: local experiment (temporary `validate_surface` relax or a hand-built sparse bundle unit
    test) run against the repro; the experiment is reverted — no code from this task is committed.
  - Files:
    - (read-only) `crates/agent-drift-analyzer/src/lib.rs`,
      `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
  - Finding (2026-06-22): **self-conservatizes**. With a temporary local probe that removed only the
    two sparse aborts in `validate_surface` (`truth_artifact_hints` and
    `working_set_hints`/`tool_argument_json`) and then reran the existing adapted repro bundle for
    `f47b81f39f2495dd`, the analyzer emitted exactly **one** checkpoint, not zero, and that
    checkpoint's progress status was **`ProgressStatus::InsufficientEvidence`** (summary:
    `Progress status distribution: advancing=0, mixed=0, stalled=0, regressing=0, insufficient_evidence=1`).
    The summary/checkpoint also stayed non-escalatory (`flagged=no`, drift=`none`, progress
    `status=insufficient_evidence dimension=planning_convergence confidence=low`), so the downstream
    path does **not** over-claim troubleshooting/strong-progress once the sparse abort is removed.
    Therefore **Packet `R5.75-2.3` is assertion-only**: it should lock this existing conservative
    behavior in tests, **not** add a new per-session conservative cap in `analyze_loaded_bundle`.
    Investigation run:
    `cargo run -p agent-session-compactor -- --codex-home "$(pwd)/target/ranga-validation/codex-home" --session-id f47b81f39f2495dd --output-dir target/r5_75-smoke/R5.75-2/f47b81f39f2495dd/compactor`
    then (after the temporary local relax)
    `cargo run -p agent-drift-analyzer -- --input-dir target/r5_75-smoke/R5.75-2/f47b81f39f2495dd/compactor --output-dir target/r5_75-smoke/R5.75-2/f47b81f39f2495dd/analyzer`.
    The temporary source relaxation was then fully reverted; no production-source diff remains from
    this task.

## R5.75-2.2: Split validate_surface (Corruption Hard-Fail vs Sparse Fail-Open)

- [ ] Task R5.75-2.2.1: Stop aborting on the sparse-but-readable conditions, and split `repetition_preserved`.
  - Acceptance: `validate_surface` no longer returns `Err` for the path-hint
    (`truth_artifact_hints`) and tool-payload (`working_set_hints` / `tool_argument_json`) conditions
    — each independently, neither gated on the other; it returns `Ok(AnalyzerSurface { … })` with those
    flags `false`. **`repetition_preserved` is split**: the archival-coverage half (`archival >=
    compact`) stays a hard-fail, but the `!dedupe_groups.is_empty()` half is dropped so a no-duplicate
    bundle no longer aborts. `stable_row_refs`, `literal_objective_rows` (directive-row floor),
    archival-coverage, and all upstream `InputError` variants (incl. `MissingDedupeRepresentative`)
    still return their existing errors. `AnalyzerSurface` shape is unchanged (no new fields, no schema
    bump). Add the minimal tool-payload-axis `input_contract` proof here (TDD); the full matrix is
    R5.75-2.4.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test input_contract -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/input.rs`
    - `crates/agent-drift-analyzer/tests/input_contract.rs` (minimal tool-payload-axis proof only)

## R5.75-2.3: Guarantee The Conservative Checkpoint

- [ ] Task R5.75-2.3.1: Ensure a sparse session yields a conservative checkpoint.
  - Acceptance: a de-aborted sparse session emits ≥1 checkpoint whose status is
    `ProgressStatus::InsufficientEvidence` (the status field, not merely a low `Confidence`), never a
    troubleshooting/strong-progress posture. If R5.75-2.1 showed the pipeline already self-conservatizes,
    this task is assertion-only (no source change); else `analyze_loaded_bundle` applies a **per-session**
    cap derived from that session's own rows (never the bundle-wide `AnalyzerSurface`), using existing
    surfaces only.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/lib.rs` (only if the probe showed over-claiming; per-session, not bundle.surface)

## R5.75-2.4: Regressions

- [ ] Task R5.75-2.4.1: Complete the input-contract matrix (do not re-add R5.75-2.2's tool-payload test).
  - Acceptance: building on R5.75-2.2's tool-payload-axis proof, `tests/input_contract.rs` adds: (a') the
    path-hint axis — a readable bundle with an objective row but no path hints and no tool calls (the
    conceptual-ask shape) — returns `Ok`; (b) a clean **no-duplicate** bundle (objective rows present,
    `dedupe_groups` empty) returns `Ok`, locking the `repetition_preserved` split; and (c) a corrupt
    bundle (non-unique/unstable row refs, a dedupe-audit entry referencing a missing archival row, or
    `archival < compact`) still returns the exact existing `InputError` variant. Each sparsity axis is
    pinned independently; the split must be provable, not just the relaxation.
  - Verify: `cargo test -p agent-drift-analyzer --test input_contract -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/input_contract.rs`

- [ ] Task R5.75-2.4.2: Conservative-checkpoint regression.
  - Acceptance: `tests/checkpoints.rs` proves a sparse readable session (steer ask + pasted `<skill>`
    body + no tool calls, the minimized `f47b81f39f2495dd` shape) emits ≥1 checkpoint (this minimized
    fixture yields exactly one) whose status is `ProgressStatus::InsufficientEvidence`, with a
    `structured_objective` anchored to the steer ask and weak fields (`success_conditions` /
    `deliverables` / `target`) unknown — never anchored to the `<skill>` body.
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
