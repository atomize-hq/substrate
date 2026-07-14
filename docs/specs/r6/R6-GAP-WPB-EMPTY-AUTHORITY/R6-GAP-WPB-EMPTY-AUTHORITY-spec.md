# R6-GAP-WPB-EMPTY-AUTHORITY — Empty-Authority No-Claim Gap

Status: **ACTIVE — PACKET DOCS GATE; PRESERVED RED; NO GAP EXECUTION YET**. `CTX-R6-15` is preserved at witness `59f098b35`; transition series `2937dbe5a` + `91f55f6bf` made this the sole active phase with active packet `none`. This first atomic batch creates only this SPEC, PLAN, and TASKS. Witness reconfirmation, production work, and no-code receipt work remain blocked until these docs and the separate ledger reconciliation are each committed and fresh-review-clean.

## Objective And Locked Contract

Close only `CTX-R6-15` without weakening its committed witness:

`wrong_plan_branch_makes_no_claim_for_path_action_without_authority`

The locked input has empty `truth_artifacts`, no non-`observed_command` working-set authority, and one path-bearing write/verification observation. An `observed_command` path is evidence of what happened, not authority establishing what was allowed. Required result:

- `0 / Low / Cleared`;
- unflagged;
- empty evidence.

At witness `59f098b35`, the result was `60 / Low / Active`, flagged, with command-action evidence. Preserve that historical red and the exact expected result.

## Owning Seam, Scope, And Non-Goals

`score_wrong_plan_branch` in `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs` owns the disposition. The smallest valid repair makes no wrong-branch comparison or evidence claim when the effective authority set is empty. It must not reclassify an `observed_command` path as authority.

Default production boundary:

- production: only `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs`;
- test: `crates/agent-drift-analyzer/tests/wrong_plan_branch.rs` only if a distinct narrow regression is necessary beyond the preserved witness;
- proof/status: this TASKS and the changed `CTX-R6-15` / named-gap evidence in `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.

Do not edit context extraction, task-frame construction, command classification, scorer dispatch, another scorer, schemas/exports, fixtures, replay, sentinel presentation, R7, or public APIs. If source or GitNexus proof shows that the scorer-local boundary cannot satisfy the locked contract, preserve the evidence and stop under the escalation contract; do not widen silently.

Before any indexed-symbol edit, run exactly:

```bash
npx gitnexus impact score_wrong_plan_branch -r 97a0-substrate --direction upstream --depth 3
```

Record risk, direct callers, affected processes, and affected modules in TASKS. Run the same literal form for every other existing symbol proposed for edit. Warn and stop before edits on HIGH or CRITICAL impact.

## Acceptance And Protected Regressions

### Red production-fix route

Use this route only while the exact witness remains red. Make the smallest scorer-local change needed to make empty effective authority produce the locked no-claim result. Do not change non-empty-authority scoring, thresholds, evidence text, state resolution, or path matching.

Protect these existing behaviors:

- read-only out-of-scope exploration remains clear;
- a write under sanctioned replan authority remains clear;
- opaque parent orchestration without attributable child action remains clear;
- an off-scope interval remains active while a later in-scope interval clears.

### Already-green no-code route

Because this is a later sequential named gap, no-code closure is allowed only if an already-landed, review-clean fix commit from an earlier named gap demonstrably caused the exact witness to become green. Re-run and record the exact witness, protected controls, family wall, checkpoints, full analyzer, and static wall; cite the causal earlier commit; change no production or test file. If the witness is green but no eligible causal earlier-gap commit can be established, preserve the output and stop for authority reconciliation rather than manufacture a receipt.

## Exact Verification

Run focused proof before the family wall:

```bash
cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture
cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_ignores_read_only_out_of_scope_exploration -- --exact --nocapture
cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_accepts_write_under_sanctioned_replan_scope -- --exact --nocapture
cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action -- --exact --nocapture
cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_clears_after_a_later_interval_returns_in_scope -- --exact --nocapture
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo fmt --all -- --check
cargo check -p agent-drift-analyzer
cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings
git diff --check
```

Record exact counts, dispositions, command exits, and any unrelated failure honestly. Do not infer current results from the preserved witness or an earlier wall.

## Atomic Batches, Review, And Exit

1. Packet docs: exactly this SPEC, PLAN, and TASKS; commit and obtain fresh built-in `default` review until clean.
2. Ledger reconciliation: after Batch 1 is review-clean, touch exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`; replace this gap's three `TO CREATE` markers with actual paths, record the review-clean packet-docs commit, preserve the red disposition, and make witness reconfirmation next. Commit and fresh-review until clean.
3. Closure candidate: use either the bounded production-fix files or the TASKS-plus-ledger no-code receipt; record actual proof; commit atomically and fresh-review until clean.
4. Transition: only after the closure candidate is review-clean, apply the complete landed R6-C.1 phase-transition authority manifest in a separate authority-only commit. Mark this gap complete and activate `R6-REPLAY`; assign no terminal `wrong_plan_branch` disposition, execute no replay control, and start no replay work. Fresh-review fixes remain transition-only. Stop when the transition series is committed and review-clean.

Before every commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Unrelated dirty files remain unstaged. Every review boundary uses a fresh built-in `default` reviewer; every actionable finding lands in a new bounded commit and receives another fresh review.
