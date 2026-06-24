# Tasks: Agent Drift Analyzer Structured Goal Anchor Faithfulness (R5.75-6)

Status: Task `R5.75-6.0.1` docs lock finalized on 2026-06-24 after verifying that `R5.75-5` is promoted and the live
`docs/specs/r5/R5_75/MAP.md` names `R5.75-6` as the active seam.

Keep each task narrow, reviewable, and scoped to the bounded Issue 7 bridge repair. Do not widen
into the deferred structured-native consumer migration, new adapted-fixture work, or `R6` scorer
retuning.

Packet prerequisite rule: this packet names `R5.75-5` as landed. Verify that in live docs/tests and
smoke outputs before editing. If the prerequisite is missing, stop and report it instead of
compensating inside `R5.75-6`.

## R5.75-6.0: Docs Lock

- [x] Task R5.75-6.0.1: Commit the SPEC/PLAN/TASKS family for `R5.75-6`.
  - Acceptance: `docs/specs/r5/R5_75/R5_75-6/` contains the spec, plan, and this tasks ledger, and
    they record:
    - the bounded bridge-fidelity scope,
    - the named wrong-imperative repro and anchored-review smoke witnesses,
    - the rule that structured goal authority wins when grounded,
    - the explicit out-of-scope boundary against the full structured-native migration, and
    - the requirement to rerun the full named `R5.75-5` smoke set before promotion.
  - Verify: manual review against `docs/specs/r5/R5_75/MAP.md`,
    `docs/specs/r5/R5_75/structured-objective-bug-map.md`,
    `docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md`, and the
    live checkpoint/inference bridge code.
  - Files:
    - `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-spec.md`
    - `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-plan.md`
    - `docs/specs/r5/R5_75/R5_75-6/agent-drift-analyzer-structured-goal-anchor-faithfulness-tasks.md`
  - Closeout note (2026-06-24): this task is a docs-only landing. The SPEC/PLAN/TASKS authority lock
    is committed for `R5.75-6` after cross-checking the active map routing, the Issue 7 diagnosis,
    the current checkpoint compatibility overlay in `checkpoint/mod.rs`, the current
    `infer_task_frame(...)` consumer, and the migration doc that defers the broader
    structured-native cutover. Implementation tasks `R5.75-6.1+` remain open.

## R5.75-6.1: Repair The Bounded Compatibility Bridge

- [ ] Task R5.75-6.1.1: Make the checkpoint compatibility bridge defer to the grounded structured goal
      anchor when it exists.
  - Acceptance:
    - when `context.objective.structured` contains grounded `Goal` evidence, the effective objective
      text used downstream is faithful to that goal instead of a later optional-nit / closeout
      imperative line,
    - legacy narrowing remains available as fallback when structured grounding is absent or weak,
    - the change stays bounded to the checkpoint/inference bridge and does not widen into the full
      consumer migration,
    - `comparison_key` and the structured sidecar remain semantically authoritative.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/inference/mod.rs`
    - optionally `crates/agent-drift-analyzer/src/context/objective.rs` only if a shared grounding
      helper is required

- [ ] Task R5.75-6.1.2: Demote optional reviewer nits and similar non-goal imperative bullets so they
      cannot become the effective objective after a real ask is already anchored.
  - Acceptance:
    - the specific wrong-imperative class from `019eddaa-e8b2-74b2-9f45-e4ce17aaab55` is blocked,
    - optional-nit / not-taken / review-closeout bullets no longer outrank the real validate/readiness
      ask once a grounded structured goal exists,
    - the repair does not suppress legitimate fallback narrowing for genuinely non-structured cases.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - optionally `crates/agent-drift-analyzer/src/context/objective.rs` if shared classification is
      needed

## R5.75-6.2: Add Durable Regression Coverage

- [ ] Task R5.75-6.2.1: Add a minimized checkpoint regression for the wrong-imperative follow-up
      shape.
  - Acceptance:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs` contains a minimized repro modeled on
      `019eddaa-e8b2-74b2-9f45-e4ce17aaab55`,
    - the regression asserts the effective objective matches the validate/readiness ask rather than
      `add extra task-local grep checks for transition routing / outcome/final-marker meaning`, and
    - the exported structured objective remains aligned to the same real ask.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5.75-6.2.2: Add or preserve a fallback guard so the packet does not over-disable legacy
      narrowing when structured grounding is absent.
  - Acceptance:
    - either an existing regression still proves fallback behavior after the bridge repair, or a new
      focused regression does so explicitly,
    - the packet does not trade the wrong-imperative fix for loss of legitimate non-structured
      compatibility behavior.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.75-6.3: Prove No Automated Downstream Regressions

- [ ] Task R5.75-6.3.1: Rerun the progress acceptance wall after the bridge repair.
  - Acceptance:
    - `progress_acceptance` stays green,
    - no earlier `R5.75-3` / `R5.75-4` / `R5.75-5` packet-owned expectation regresses because the
      effective objective text changed.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - no source changes required for closeout; if the run exposes a true packet-owned regression,
      keep fixes inside the bounded `R5.75-6` bridge seam or stop and re-scope

- [ ] Task R5.75-6.3.2: Rerun the full analyzer wall and sentinel spot-checks.
  - Acceptance:
    - `cargo test -p agent-drift-analyzer -- --nocapture` is green,
    - `cargo test -p agent-drift-sentinel warning_policy -- --nocapture` is green,
    - `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture` is green.
  - Verify:
    - the three commands above
  - Files:
    - no source changes required for closeout; if these gates fail, fix only bounded packet-owned
      fallout or stop and report the broader migration pressure

## R5.75-6.4: Manual Smoke And Honest Closeout

- [ ] Task R5.75-6.4.1: Rerun native smoke for the wrong-imperative repro and the anchored-review
      witness.
  - Acceptance:
    - `target/r5_75-smoke/R5.75-6/019eddaa-e8b2-74b2-9f45-e4ce17aaab55/` is regenerated and reviewed,
    - `task_frame.objective` / summary output no longer narrows to the reviewer nit on `019eddaa`,
    - `target/r5_75-smoke/R5.75-6/019eb47f-0118-7e90-8291-30a1fb93769e/` is regenerated and reviewed,
    - the anchored-review/evaluate semantics from `R5.75-1` still hold on `019eb47f`.
  - Verify:
    - rerun the smoke commands from the spec and inspect `summary.md` plus the first
      `checkpoints.jsonl` rows for both sessions
  - Files:
    - `target/r5_75-smoke/R5.75-6/019eddaa-e8b2-74b2-9f45-e4ce17aaab55/**`
    - `target/r5_75-smoke/R5.75-6/019eb47f-0118-7e90-8291-30a1fb93769e/**`

- [ ] Task R5.75-6.4.2: Rerun the full named `R5.75-5` smoke set and close the family honestly.
  - Acceptance:
    - the full carried-forward native + adapted smoke set is rerun under `target/r5_75-smoke/R5.75-6/`,
    - no earlier `R5.75-1` through `R5.75-5` expectation regresses,
    - `docs/specs/r5/R5_75/MAP.md` is updated only after the packet is honestly promoted and `R5.75`
      is truly complete.
  - Verify:
    - rerun the named native + adapted smoke set from `docs/specs/r5/R5_75/MAP.md`
    - inspect `summary.md` plus the first `checkpoints.jsonl` rows for each named witness
  - Files:
    - `target/r5_75-smoke/R5.75-6/**`
    - `docs/specs/r5/R5_75/MAP.md`
