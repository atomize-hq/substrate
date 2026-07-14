# Tasks: R6-GAP-TGG-TRUTH-PATH-ACTION

Status: **ACTIVE — REVIEW FINDING PRESERVED / DECISION REQUIRED**. Production commit `52c9ab296` plus proof receipt `73132aead` received fresh `REVIEW FINDINGS` P1: `prior_grounding` conflates historical posture with grounding, while `lib.rs` drops clean `Cleared` grounding, producing two opposite cross-checkpoint defects. Test-only commit `6409ae072` preserves both exact regression witnesses in the allowed test file and independently received `REVIEW CLEAN`. The correct path-scoped provenance repair requires upstream internal edits beyond this packet's boundary, so Option A remains operator-decision-blocked; Task 4, the transition, and the successor remain open or blocked as applicable.

## Required Gates

Staged commit gate for every commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Indexed-symbol gate before production edits:

```bash
npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
```

Impact every additional proposed symbol with the identical command form; stop and warn on HIGH/CRITICAL.

## Task Ledger

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.0 — Commit and independently review the packet docs.**
  - Files: exactly this SPEC, PLAN, and TASKS.
  - Verify: staged gate and complete staged-diff inspection; do not run scorer tests.
  - Review: fresh built-in `default`; docs-only fix commits and fresh reviewers until clean.
  - Stop rule: Task 1 remains unauthorized until the packet-docs commit is fresh-review-clean.
  - Result: `03754a2de` (`docs: lock truth path action gap packet`) contains the three packet docs and received fresh built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.1 — Reconcile the review-clean packet into the canonical ledger.**
  - Prerequisite: Task 0 committed and fresh-review-clean.
  - Separate batch: exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.
  - Replace the `R6-GAP-TGG-TRUTH-PATH-ACTION` row's three `TO CREATE` markers with the actual SPEC/PLAN/TASKS paths; record the review-clean packet commit; preserve `CTX-R6-12` as `PRESERVED RED / GAP ACTIVE`; make witness reconfirmation the sole next action.
  - Commit/review: staged gate; ledger-only commit; fresh built-in `default`; ledger-only fixes and fresh reviewers until clean.
  - Blocking rule: no Task 2 witness, production edit, or no-code proof before this task is review-clean.
  - Result: `a5380c04e` (`docs: record truth path packet gate`) contains the ledger-only reconciliation and received fresh built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-TGG-TRUTH-PATH-ACTION.2 — Reconfirm the witness and select red versus no-code.**
  - Prerequisite: Tasks 0 and 1 committed and fresh-review-clean.
  - Witness: `e67d8b214`; control: `CTX-R6-12`.
  - Run focused before family:

    ```bash
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_flags_truth_path_action_before_read -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test truth_grounding_gap truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes -- --exact --nocapture
    cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
    ```
  - Red: select Task 3A. Green: select Task 3B only with attributable red-before/green-after proof at an earlier sequential named-gap production boundary; otherwise stop/escalate.
  - Result: the exact pre-edit `CTX-R6-12` witness remained red at `0 / Medium / Cleared`, unflagged, versus required `80 / High / Active`, flagged (`0 passed; 1 failed`). The exact analogous archetype regression passed (`1 passed; 0 failed`), and the owning `truth_grounding_gap` family reported `9 passed; 1 failed`, with only `CTX-R6-12` red. Task 3A was selected; Task 3B was ineligible and not selected.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.3A — Land the smallest scorer-local production fix.**
  - Prerequisite: Task 2 remains red and impact is below HIGH.
  - Allowed files: `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs`; this TASKS; canonical ledger. `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs` only for a necessary distinct narrow regression; never alter the witness.
  - Acceptance: truth-path-touching write/verification before any truth read yields exactly `80 / High / Active`, flagged, with authority and action evidence; the analogous archetype regression stays green; event order, history, grounded-read recovery, and unrelated scorer behavior remain intact.
  - Verify: repeat Task 2 in order, then:

    ```bash
    cargo test -p agent-drift-analyzer checkpoints -- --nocapture
    cargo fmt --all -- --check
    cargo check -p agent-drift-analyzer
    ```
  - Commit/review: staged gate; atomic commit; fresh built-in `default`; new bounded fixes and fresh reviewers until clean.
  - Review finding preserved: `52c9ab296` (`fix: retain truth path actions before grounding`) landed the only production edit in `score_truth_grounding_gap`, and `73132aead` (`docs: record truth path action proof`) recorded its receipt. Fresh review of that production-plus-receipt series returned **`REVIEW FINDINGS` P1**, not `REVIEW CLEAN`: `prior_grounding` conflates historical posture with grounding, while `lib.rs` drops clean `Cleared` grounding.
    - Impact: LOW, with `0` upstream callers, `0` affected processes, and `0` affected modules. No HIGH/CRITICAL risk was reported.
    - The receipt's focused `CTX-R6-12`, analogous archetype, family, checkpoint, same-interval, format/check, and diff-check results remain a historical record of the incomplete wall; they do not establish proof or closure after the P1 finding.
    - `6409ae072` (`test: preserve cross-checkpoint grounding gaps`) changes only `crates/agent-drift-analyzer/tests/truth_grounding_gap.rs`, preserves both exact missing regression witnesses, and independently received **`REVIEW CLEAN`**.
    - `truth_grounding_gap_reactivates_truth_path_action_after_historical_only_recovery`: actual `20 / Medium / HistoricalOnly`, unflagged, versus required `80 / High / Active`, flagged.
    - `truth_grounding_gap_carries_clean_read_to_next_checkpoint_truth_path_action`: actual `80 / High / Active`, flagged, versus required `0 / Medium / Cleared`, unflagged.
    - The original `CTX-R6-12` control remains passing; the owning family is `10 passed; 2 failed`, with exactly the two preserved cross-checkpoint witnesses red.
    - Fresh read-only design concluded that correct path-scoped grounding provenance requires upstream internal edits beyond the authorized scorer-local packet boundary. Option A remains blocked on an operator decision; no production fix is authorized by this record. Task 3A is therefore unchecked and not accepted complete.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.3B — Attributed no-code proof receipt — ineligible and not selected.**
  - Eligibility: an already-landed production commit from earlier sequential gap `R6-GAP-DET-OPAQUE-PARENT` must demonstrably make this witness green.
  - Proof: in clean checkouts, run the exact witness at the candidate commit's first parent and candidate commit, proving red-before/green-after; at the green boundary run the analogous focused regression before the owning family.
  - If attribution is absent, ambiguous, or unrelated, stop/escalate; current-HEAD green alone is insufficient.
  - Allowed files: this TASKS and canonical ledger only; no source or test change.
  - Commit/review: staged gate; receipt-only commit; fresh built-in `default`; receipt-only fixes and fresh reviewers until clean.
  - Result: Task 2 remained red, so the eligible red production path was Task 3A. No no-code attribution or receipt was attempted, and this unchecked item does not claim completion.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.4 — Obtain a review-clean closure path.**
  - Require exactly one of Tasks 3A/3B committed with exact proof and fresh `REVIEW CLEAN`.
  - Record commit hashes, command results, findings, dispositions, and review verdict here. Do not check this task before the verdict exists.
  - Current state: **OPEN / DECISION REQUIRED**. Production `52c9ab296` plus receipt `73132aead` has a preserved P1 `REVIEW FINDINGS` verdict; witness commit `6409ae072` is independently `REVIEW CLEAN` but proves only that the two opposite regressions are preserved. Option A remains operator-decision-blocked because the correct path-scoped provenance repair exceeds the packet boundary. No review-clean closure path exists yet.

- [ ] **R6-GAP-TGG-TRUTH-PATH-ACTION.5 — Land and independently review the narrow transition.**
  - Prerequisite: Tasks 0, 1, and 4 review-clean.
  - Current state: **BLOCKED** on Task 4 and the operator decision; no transition or successor work is authorized.
  - Exact authority-only manifest:
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - `SPEC.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/00-README.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/01-authority-and-status-map.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/02-phase-and-gate-map.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`
    - `docs/specs/hybrid-drift-r6-r8-control-pack/06-operator-prompt-library.md`
    - `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`
    - `docs/specs/r6/MAP.md`
    - `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-plan.md`
    - `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md`
    - `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-tasks.md`
    - this packet's SPEC, PLAN, and TASKS
    - `tasks/plan.md`
    - `tasks/todo.md`
  - Apply actual hashes/results. Mark this gap `COMPLETE`; activate only `R6-GAP-WPB-EMPTY-AUTHORITY` at its docs-only gate; keep `R6-REPLAY` blocked.
  - Successor boundary: record only these non-link paths:
    - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-spec.md`
    - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-plan.md`
    - TO CREATE `docs/specs/r6/R6-GAP-WPB-EMPTY-AUTHORITY/R6-GAP-WPB-EMPTY-AUTHORITY-tasks.md`
  - Do not create, link, cite as existing, or execute the successor packet.
  - Commit/review: staged gate; separate authority-only commit; fresh independent built-in `default`; transition-only fix commits and fresh reviewers until clean.
  - Stop: transition review-clean; no successor work.

## Explicit Exclusions

No R7, replay, wrong-plan-branch implementation, context/command-classification edit, dispatcher edit, unrelated scorer, witness rewrite, export/presentation change, or successor packet creation belongs in this phase.
