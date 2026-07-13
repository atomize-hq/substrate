# Tasks: R6-GAP-DET-OPAQUE-PARENT

Status: **ACTIVE — PACKET DOCS THEN LEDGER-RECONCILIATION GATES**. Sole current action: commit these three docs atomically and obtain fresh-review-clean status. A separate post-docs ledger reconciliation is then required. Production, witness rerun, proof receipt, transition, and successor work remain blocked until both gates are committed and fresh-review-clean.

## Required Gates

Staged commit gate for every commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Indexed-symbol gate before edits:

```bash
npx gitnexus impact score_dead_end_thrash -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_confidence -r 97a0-substrate --direction upstream --depth 3
```

The second command is required if `score_confidence` changes. Impact every additional symbol before editing; stop on HIGH/CRITICAL. `build_session_progress` and `parent_visible_orchestration_progress` are read-only context, not allowed edit scope.

## Task Ledger

- [ ] **R6-GAP-DET-OPAQUE-PARENT.0 — Commit and independently review the packet docs.**
  - Files: exactly this SPEC, PLAN, and TASKS.
  - Verify: staged gate; complete staged-diff inspection.
  - Review: fresh built-in `default`; docs-only fix commits; fresh reviewer until clean.
  - Result: pending. Do not start Task 1 first.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.1 — Reconcile the review-clean packet into the canonical ledger.**
  - Prerequisite: Task 0 is committed and fresh-review-clean.
  - Separate orchestration batch: touch exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.
  - Required update: in the `R6-GAP-DET-OPAQUE-PARENT` named-gap subledger row, replace all three `TO CREATE` markers with the actual SPEC/PLAN/TASKS paths and record the review-clean packet-docs commit. Reconcile only ledger-local current-status/next-action wording needed to make witness reconfirmation next; preserve the `CTX-R6-04` red disposition.
  - Commit/review: staged gate; separate ledger-only commit; fresh built-in `default` review; ledger-only fix commits and fresh reviewers until clean.
  - Blocking rule: do not start Task 2, rerun any witness, or edit production until this task is committed and fresh-review-clean.
  - Result: pending.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.2 — Reconfirm the preserved witness and select the authorized closure path.**
  - Prerequisite: Tasks 0 and 1 are each committed and fresh-review-clean.
  - Witness: commit `87409b39a`; control `CTX-R6-04`.
  - Run:

    ```bash
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
    ```
  - Red: use Task 3A. Unexpectedly green: preserve exact output and stop/escalate for authority reconciliation. This first named gap has no earlier sequential named-gap fix, so no-code closure is unavailable; do not attribute green to an unrelated already-landed commit.
  - Result: pending; preserve exact output.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.3A — Land the smallest scorer-local production fix.**
  - Prerequisite: Task 2 remains red and impact is below HIGH.
  - Files: `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`; the existing test file only for a necessary additional narrow regression; this TASKS; and the proof ledger's changed `CTX-R6-04` / named-gap evidence.
  - Acceptance: opaque parent stays `ParentVisibleOrchestration`; thrash becomes exactly `0 / Low / Cleared`, unflagged, empty evidence; parent orchestration does not become child misconduct; the regression control remains `30 / Medium / Active`, flagged.
  - Verify in order:

    ```bash
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
    cargo test -p agent-drift-analyzer checkpoints -- --nocapture
    ```
  - Commit/review: staged gate; atomic commit; fresh built-in `default` review; new fix commit and fresh reviewer until clean.
  - Result: pending.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.3B — No-code proof receipt — ineligible for this first gap.**
  - Eligibility rule: only a later named gap may use no-code closure, and only when an already-landed fix commit from an earlier named gap in the prescribed sequence demonstrably made that later gap's preserved witness green. An unrelated earlier commit is never eligible.
  - This packet is the first sequential named gap, so it cannot satisfy the prerequisite. An unexpected pre-edit green result must be preserved and escalated for authority reconciliation under Task 2, not converted into a receipt.
  - Result: ineligible for this phase.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.4 — Obtain a review-clean production fix.**
  - Require Task 3A committed with actual proof and a fresh `REVIEW CLEAN` verdict.
  - Record all commit hashes, exact command results, findings, dispositions, and review verdict here.
  - Result: pending.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.5 — Land and independently review the narrow transition.**
  - Prerequisite: Tasks 0, 1, and 4 review-clean.
  - Apply the full R6-C.1 gap-transition authority manifest. Mark this gap `COMPLETE`; activate only `R6-GAP-TGG-TRUTH-PATH-ACTION`; keep later gaps blocked.
  - Successor boundary: record only these non-link `TO CREATE` paths and authorize their atomic docs gate:
    - `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-spec.md`
    - `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-plan.md`
    - `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-tasks.md`
  - Do not create, link, cite as existing, or execute the successor packet.
  - Commit/review: authority docs only; staged gate; separate commit; fresh independent review and new transition-only fixes until clean.
  - Stop: review-clean transition committed; no next-phase work.
  - Result: pending.

## Explicit Exclusions

No R7, replay, truth-grounding implementation, wrong-plan-branch implementation, upstream progress/delegation edit, fixture rewrite, unrelated scorer, export/presentation change, or successor packet creation belongs in this phase.
