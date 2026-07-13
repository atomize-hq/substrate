# Plan: R6-GAP-DET-OPAQUE-PARENT

Status: **ACTIVE — PACKET DOCS GATE**. This plan becomes executable only after the three canonical packet docs are committed and fresh-review-clean.

## Decisions

1. Preserve `CTX-R6-04` and witness commit `87409b39a`; do not rewrite the expected `0 / Low / Cleared`, unflagged, empty-evidence contract.
2. Keep the repair scorer-local. `score_dead_end_thrash` is the owner. `build_session_progress` / `parent_visible_orchestration_progress` supply read-only upstream context for the already-correct `ParentVisibleOrchestration` seam.
3. Do not interpret parent `spawn_agent` / `wait_agent` observations as attributable child thrash. Do not add R7 child linkage.
4. Use one existing regression pattern: `dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity`, locked at `30 / Medium / Active`, flagged.
5. Run focused proof before the authorized R6-C.1 family/checkpoint walls.
6. Keep the fix or no-code receipt commit separate from the final authority transition.

## Ordered Execution

### 0. Lock The Packet

Create and stage only this SPEC/PLAN/TASKS family, run the staged commit gate, commit atomically, and send a fresh built-in `default` reviewer. Make docs-only fix commits and repeat with a fresh reviewer until clean. No gap execution occurs first.

### 1. Reconfirm The Preserved Seam

After the docs gate is review-clean, run the exact `CTX-R6-04` command. Then run the owning family command to establish live pre-edit scope. Preserve all output.

- If red, continue to the production-fix path.
- If green, use the no-code path only when a concrete already-landed causal commit is identifiable; otherwise the closure evidence is insufficient.

### 2A. Production-Fix Path

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

### 2B. No-Code Path

Do not touch production. Cite the already-landed causal commit, rerun the exact witness and owning `dead_end_thrash` family command, and record exact outputs in TASKS and the ledger. Stage only those docs. This receipt leaves the gap active and does not replace the separate transition.

### 3. Commit And Fresh Review

For the fix or receipt, and for every review-fix commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit atomically. Dispatch a fresh built-in `default` reviewer. Apply actionable findings in a new bounded commit and repeat with a fresh reviewer until clean.

### 4. Transition And Stop

Only after the gap proof is committed and review-clean, make a separate authority-only transition using the complete landed R6-C.1 transition manifest. At minimum it must:

- mark this gap `COMPLETE` in the named-gap subledger and phase-current packet statuses;
- activate only `R6-GAP-TGG-TRUTH-PATH-ACTION`, leaving later gaps blocked;
- set the successor's sole next action to atomic creation/review of its three `TO CREATE` canonical docs;
- reconcile root, R6-C.1, control-pack, ledger, operator-prompt, and proof/status mirrors with actual hashes and results;
- contain no production change and no successor packet file.

Run the staged gate, commit, and dispatch a fresh independent reviewer; use new narrow commits and fresh reviewers until clean. Stop at that review-clean transition. Do not start the successor.

## Escalation Boundary

Escalate only for a material authority/product choice, HIGH/CRITICAL impact, unavailable preserved evidence, unisolatable unrelated work, or review proving this scorer-local packet invalid. Routine red proof, LOW/MEDIUM impact, tests, commits, and review fixes remain autonomous.
