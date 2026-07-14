# Tasks: R6-GAP-DET-OPAQUE-PARENT

Status: **COMPLETE — PRODUCTION SERIES THROUGH `d13f0a71c` REVIEW CLEAN; SUCCESSOR DOCS-ONLY GATE ACTIVE**. The packet-docs gate is review-clean at `59092df2a`, the separate ledger-reconciliation gate is review-clean at `beed76446`, and production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh built-in `default` `REVIEW CLEAN`. `CTX-R6-04` is proven focused and this gap is complete. This authority update activates only `R6-GAP-TGG-TRUTH-PATH-ACTION` at its docs-only gate without claiming an unknown transition commit or review verdict.

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

- [x] **R6-GAP-DET-OPAQUE-PARENT.0 — Commit and independently review the packet docs.**
  - Files: exactly this SPEC, PLAN, and TASKS.
  - Verify: staged gate; complete staged-diff inspection.
  - Review: fresh built-in `default`; docs-only fix commits; fresh reviewer until clean.
  - Result: `59092df2a` (`docs: tighten opaque parent gap gates`) contains the three packet docs and received fresh built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-DET-OPAQUE-PARENT.1 — Reconcile the review-clean packet into the canonical ledger.**
  - Prerequisite: Task 0 is committed and fresh-review-clean.
  - Separate orchestration batch: touch exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.
  - Required update: in the `R6-GAP-DET-OPAQUE-PARENT` named-gap subledger row, replace all three `TO CREATE` markers with the actual SPEC/PLAN/TASKS paths and record the review-clean packet-docs commit. Reconcile only ledger-local current-status/next-action wording needed to make witness reconfirmation next; preserve the `CTX-R6-04` red disposition.
  - Commit/review: staged gate; separate ledger-only commit; fresh built-in `default` review; ledger-only fix commits and fresh reviewers until clean.
  - Blocking rule: do not start Task 2, rerun any witness, or edit production until this task is committed and fresh-review-clean.
  - Result: `beed76446` (`docs: record opaque parent packet gate`) contains the ledger-only reconciliation and received fresh built-in `default` `REVIEW CLEAN`.

- [x] **R6-GAP-DET-OPAQUE-PARENT.2 — Reconfirm the preserved witness and select the authorized closure path.**
  - Prerequisite: Tasks 0 and 1 are each committed and fresh-review-clean.
  - Witness: commit `87409b39a`; control `CTX-R6-04`.
  - Run:

    ```bash
    cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture
    cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
    ```
  - Red: use Task 3A. Unexpectedly green: preserve exact output and stop/escalate for authority reconciliation. This first named gap has no earlier sequential named-gap fix, so no-code closure is unavailable; do not attribute green to an unrelated already-landed commit.
  - Result: the pre-edit focused control remained red at `0 / Medium / Cleared`, unflagged, with empty evidence, versus required `0 / Low / Cleared`; the owning integration family reported `16 passed; 1 failed`, with only `CTX-R6-04` failing. The authorized path is Task 3A; no-code closure remains ineligible.

- [x] **R6-GAP-DET-OPAQUE-PARENT.3A — Land the smallest scorer-local production fix.**
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
  - Historical result at `bcd94bf4f`: committed as `bcd94bf4f` (`fix: cap opaque parent thrash
    confidence`) with the ordered proof green. Fresh review found two required fixes, so that commit
    remained review-blocked and Task 4 stayed open at that review point.
    - Impact: `score_dead_end_thrash` LOW with no upstream callers; `score_confidence` LOW with one direct caller (`score_dead_end_thrash`), two affected processes, and one affected `Scoring` module. No HIGH/CRITICAL risk was reported.
    - Production at `bcd94bf4f`: `score_confidence` receives `SessionProgress` and attempted to keep opaque parent-only activity at Low confidence without editing upstream progress construction. Review later found that the exception covered every no-history `ParentVisibleOrchestration` case rather than only the typed Low-confidence seam.
    - Focused `CTX-R6-04`: `1 passed; 0 failed; 16 filtered out`; the locked result is `0 / Low / Cleared`, unflagged, empty evidence, with `ParentVisibleOrchestration` preserved.
    - Focused `CTX-R6-03`: `1 passed; 0 failed; 16 filtered out`; the regression result remains `30 / Medium / Active`, flagged.
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`: all matching tests passed — owning integration target `17 passed; 0 failed`, acceptance target `1 passed; 0 failed`, and export target `2 passed; 0 failed`.
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`: all matching tests passed — unit target `35 passed; 0 failed`, checkpoints integration target `131 passed; 0 failed`, and export target `1 passed; 0 failed`.

- [ ] **R6-GAP-DET-OPAQUE-PARENT.3B — No-code proof receipt — ineligible for this first gap.**
  - Eligibility rule: only a later named gap may use no-code closure, and only when an already-landed fix commit from an earlier named gap in the prescribed sequence demonstrably made that later gap's preserved witness green. An unrelated earlier commit is never eligible.
  - This packet is the first sequential named gap, so it cannot satisfy the prerequisite. An unexpected pre-edit green result must be preserved and escalated for authority reconciliation under Task 2, not converted into a receipt.
  - Result: ineligible for this phase.

- [x] **R6-GAP-DET-OPAQUE-PARENT.4 — Obtain a review-clean production fix.**
  - Require Task 3A committed with actual proof and a fresh `REVIEW CLEAN` verdict.
  - Record all commit hashes, exact command results, findings, dispositions, and review verdict here.
  - Review of `bcd94bf4f`: **CHANGES REQUIRED** with two required findings.
    1. **P1 — confidence exception was too broad.** Every no-history `ParentVisibleOrchestration` result ignored command observations, including partial/mixed visibility whose typed `SessionProgress` confidence is Medium. Review-fix candidate `931e50c85` suppresses command-observation escalation only for `ParentVisibleOrchestration + Low`; it does not duplicate raw delegation inference. A regression derived from the existing partial-child-visibility synthesis proves `ParentVisibleOrchestration / Mixed / Medium` progress retains `dead_end_thrash` at `0 / Medium / Cleared`, unflagged, with empty evidence.
    2. **P2 — landed state was recorded as pending worktree.** The TASKS and control-pack ledger recorded `bcd94bf4f` as landed but review-blocked while that bounded review fix was still uncommitted. Review-fix candidate `931e50c85` landed the bounded production/test/docs change while the named gap remained ACTIVE.
  - Review-fix candidate proof at `931e50c85`:
    - Focused `CTX-R6-04`: `1 passed; 0 failed; 17 filtered out`; remains `0 / Low / Cleared`, unflagged, empty evidence.
    - New partial/mixed parent-visible exact regression: `1 passed; 0 failed; 17 filtered out`; result `0 / Medium / Cleared`, unflagged, empty evidence.
    - Focused `CTX-R6-03`: `1 passed; 0 failed; 17 filtered out`; remains `30 / Medium / Active`, flagged.
    - `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`: all `21` matching tests passed — owning integration target `18 passed; 0 failed`, acceptance target `1 passed; 0 failed`, export target `2 passed; 0 failed`.
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`: all `167` matching tests passed — unit target `35 passed; 0 failed`, checkpoints integration target `131 passed; 0 failed`, export target `1 passed; 0 failed`.
    - `cargo fmt --all -- --check` and `cargo check -p agent-drift-analyzer`: passed.
  - Fresh re-review of the production series through `931e50c85`: **CHANGES REQUIRED** with one finding.
    1. **P2 — landed review-fix candidate was still described as an uncommitted worktree.** The TASKS and control-pack ledger called the now-landed `931e50c85` review fix pending worktree work. This bounded bookkeeping correction records `931e50c85` as the landed review-fix candidate without changing its proof, the ACTIVE gap posture, or successor authority.
  - Final review result: production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` received fresh
    built-in `default` **REVIEW CLEAN**. Final proof remained exact: focused `CTX-R6-04` passed at
    `0 / Low / Cleared`, unflagged, empty evidence; the partial/mixed parent-visible regression passed
    at `0 / Medium / Cleared`, unflagged, empty evidence; focused `CTX-R6-03` remained
    `30 / Medium / Active`, flagged; all `21` matching `dead_end_thrash` tests and all `167` matching
    checkpoint tests passed; and `cargo fmt --all -- --check` plus
    `cargo check -p agent-drift-analyzer` passed.

- [x] **R6-GAP-DET-OPAQUE-PARENT.5 — Land and independently review the narrow transition.**
  - Prerequisite: Tasks 0, 1, and 4 review-clean.
  - Apply the full R6-C.1 gap-transition authority manifest. Mark this gap `COMPLETE`; activate only `R6-GAP-TGG-TRUTH-PATH-ACTION`; keep later gaps blocked.
  - Successor boundary: record only these non-link `TO CREATE` paths and authorize their atomic docs gate:
    - TO CREATE `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-spec.md`
    - TO CREATE `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-plan.md`
    - TO CREATE `docs/specs/r6/R6-GAP-TGG-TRUTH-PATH-ACTION/R6-GAP-TGG-TRUTH-PATH-ACTION-tasks.md`
  - Do not create, link, cite as existing, or execute the successor packet.
  - Commit/review: authority docs only; staged gate; separate commit; fresh independent review and new transition-only fixes until clean.
  - Stop: review-clean transition committed; no next-phase work.
  - Transition result (2026-07-13): `c3202d3f3` landed this authority transition and applies the
    required statuses: marks
    `R6-GAP-DET-OPAQUE-PARENT` complete; activates only `R6-GAP-TGG-TRUTH-PATH-ACTION` at its
    docs-only gate with the three exact non-link `TO CREATE` paths above; and keeps
    `R6-GAP-WPB-EMPTY-AUTHORITY` plus `R6-REPLAY` blocked. Fresh independent review of `c3202d3f3`
    found a P2 bookkeeping issue: Task 5 was checked before the transition review verdict. Commit
    `720336de8` corrected that premature checkbox without altering the applied phase statuses or
    successor boundary, and a fresh built-in `default` reviewer returned **REVIEW CLEAN** for the
    transition series `c3202d3f3` + `720336de8`. This is the bookkeeping receipt of that
    already-obtained verdict. The sole next action is atomic creation and fresh review of those three
    successor docs; they do not exist yet and no successor implementation is authorized.

## Explicit Exclusions

No R7, replay, truth-grounding implementation, wrong-plan-branch implementation, upstream progress/delegation edit, fixture rewrite, unrelated scorer, export/presentation change, or successor packet creation belongs in this phase.
