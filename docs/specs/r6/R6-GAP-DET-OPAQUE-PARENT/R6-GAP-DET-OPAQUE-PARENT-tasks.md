# Tasks: R6-GAP-DET-OPAQUE-PARENT

Status: **ACTIVE — PACKET DOCS GATE**. Sole current action: commit these three docs atomically and obtain fresh-review-clean status. Production, witness rerun, proof receipt, transition, and successor work remain blocked until then.

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

- [ ] **R6-GAP-DET-OPAQUE-PARENT.1 — Reconfirm the preserved witness and select one closure path.**
  - Witness: commit `87409b39a`; control `CTX-R6-04`.
  - Run:

    ```bash
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
    ```
  - Red: use Task 2A. Green: Task 2B is allowed only with an identifiable already-landed causal commit.
  - Result: pending; preserve exact output.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.2A — Land the smallest scorer-local production fix.**
  - Prerequisite: Task 1 remains red and impact is below HIGH.
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

- [ ] **R6-GAP-DET-OPAQUE-PARENT.2B — Close by no-code proof receipt, if eligible.**
  - Prerequisite: exact witness green before a packet edit and a concrete already-landed causal commit recorded. This first gap has no earlier gap fix by default; absent a causal hash, this task is ineligible.
  - Files: this TASKS and proof ledger only.
  - Proof: exact witness plus owning `dead_end_thrash` family command pass; record outputs and causal hash; preserve witness `87409b39a`.
  - Commit/review: docs-only receipt; staged gate; fresh review/fix loop. Leave this gap active.
  - Result: pending / not yet eligible.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.3 — Obtain a review-clean fix or receipt.**
  - Require Task 2A or 2B committed with actual proof and a fresh `REVIEW CLEAN` verdict.
  - Record all commit hashes, exact command results, findings, dispositions, and review verdict here.
  - Result: pending.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.4 — Land and independently review the narrow transition.**
  - Prerequisite: Task 0 and Task 3 review-clean.
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
