# Plan: R6-GAP-DET-OPAQUE-PARENT

Status: **COMPLETE — PRODUCTION SERIES THROUGH `d13f0a71c` REVIEW CLEAN; SUCCESSOR DOCS-ONLY GATE ACTIVE**. Packet docs `59092df2a`, ledger reconciliation `beed76446`, and production series `bcd94bf4f` + `931e50c85` + `d13f0a71c` satisfied the required gates. This authority update activates only `R6-GAP-TGG-TRUTH-PATH-ACTION` for atomic successor-doc creation/review without claiming an unknown transition commit or review verdict.

## Decisions

1. Preserve `CTX-R6-04` and witness commit `87409b39a`; do not rewrite the expected `0 / Low / Cleared`, unflagged, empty-evidence contract.
2. Keep the repair scorer-local. `score_dead_end_thrash` is the owner. `build_session_progress` / `parent_visible_orchestration_progress` supply read-only upstream context for the already-correct `ParentVisibleOrchestration` seam.
3. Do not interpret parent `spawn_agent` / `wait_agent` observations as attributable child thrash. Do not add R7 child linkage.
4. Use one existing regression pattern: `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity`, locked at `30 / Medium / Active`, flagged.
5. Run focused proof before the authorized R6-C.1 family/checkpoint walls.
6. Keep the production-fix commit separate from the final authority transition.

## Ordered Execution

### 0. Lock The Packet

Create and stage only this SPEC/PLAN/TASKS family, run the staged commit gate, commit atomically, and send a fresh built-in `default` reviewer. Make docs-only fix commits and repeat with a fresh reviewer until clean. No gap execution occurs first.

### 1. Reconcile The Review-Clean Packet Into The Ledger

After the docs gate is review-clean, make a separate orchestration batch touching exactly `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`. In its active named-gap subledger row, replace the three `TO CREATE` markers with the actual packet paths and record the review-clean packet-docs commit. Reconcile only ledger-local current-status/next-action wording needed to remove the completed packet-docs gate and make witness reconfirmation next; preserve the red witness disposition. Run the staged gate, commit, and dispatch a fresh built-in `default` reviewer. Use ledger-only fix commits and fresh reviewers until clean.

No witness rerun, production edit, or proof receipt is authorized until this reconciliation is committed and fresh-review-clean.

### 2. Reconfirm The Preserved Seam

After both entry gates are review-clean, run the exact `CTX-R6-04` command. Then run the owning family command to establish live pre-edit scope. Preserve all output.

- If red, continue to the production-fix path.
- If unexpectedly green, preserve the exact output and stop/escalate for authority reconciliation. This is the first sequential named gap, so no earlier named-gap fix can make it eligible for no-code closure; an unrelated already-landed commit is never a valid receipt basis.

### 3A. Production-Fix Path

1. Run GitNexus impact before edits:

   ```bash
   npx gitnexus impact score_dead_end_thrash -r 97a0-substrate --direction upstream --depth 3
   npx gitnexus impact score_confidence -r 97a0-substrate --direction upstream --depth 3
   ```

   The helper command is mandatory if that helper changes. Impact every additional symbol before editing it. Stop on HIGH/CRITICAL.
2. Make the smallest change in `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`. The upstream progress file is not editable in this packet.
3. Do not alter the preserved control. Add a test only if a distinct narrow regression is necessary.
4. Run, in order:

   ```bash
   cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity -- --exact --nocapture
   cargo test -p agent-drift-analyzer --test dead_end_thrash dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity -- --exact --nocapture
   cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
   cargo test -p agent-drift-analyzer checkpoints -- --nocapture
   ```
5. Record exact results in this packet's TASKS and only the `CTX-R6-04` / named-gap ledger evidence that changed.

### 3B. No-Code Path — Not Eligible Here

No-code closure is reserved for a later named gap whose witness is made green by an already-landed fix commit from an earlier named gap in the prescribed sequence. This first gap cannot meet that prerequisite. If its pre-edit witness is unexpectedly green, preserve the output and stop/escalate for authority reconciliation; do not create a receipt from an unrelated already-landed commit.

### 4. Commit And Fresh Review

For the fix, and for every review-fix commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit atomically. Dispatch a fresh built-in `default` reviewer. Apply actionable findings in a new bounded commit and repeat with a fresh reviewer until clean.

### 5. Transition And Stop

Only after the gap proof is committed and review-clean, make a separate authority-only transition using the complete landed R6-C.1 transition manifest. At minimum it must:

- mark this gap `COMPLETE` in the named-gap subledger and phase-current packet statuses;
- activate only `R6-GAP-TGG-TRUTH-PATH-ACTION`, leaving later gaps blocked;
- set the successor's sole next action to atomic creation/review of its three `TO CREATE` canonical docs;
- reconcile root, R6-C.1, control-pack, ledger, operator-prompt, and proof/status mirrors with actual hashes and results;
- contain no production change and no successor packet file.

Run the staged gate, commit, and dispatch a fresh independent reviewer; use new narrow commits and fresh reviewers until clean. Stop at that review-clean transition. Do not start the successor.

## Escalation Boundary

Escalate only for a material authority/product choice, HIGH/CRITICAL impact, unavailable preserved evidence, unisolatable unrelated work, or review proving this scorer-local packet invalid. Routine red proof, LOW/MEDIUM impact, tests, commits, and review fixes remain autonomous.
