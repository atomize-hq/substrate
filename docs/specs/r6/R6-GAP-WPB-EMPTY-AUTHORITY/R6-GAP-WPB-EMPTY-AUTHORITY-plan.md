# Plan: R6-GAP-WPB-EMPTY-AUTHORITY

Status: **COMPLETE — IMPLEMENTATION/REVIEW-FIX SERIES REVIEW CLEAN; AUTHORITY TRANSITION LANDED; FRESH TRANSITION REVIEW PENDING**. Implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` received fresh independent built-in `default` `REVIEW CLEAN` with the exact witness and protected proof wall green. The authority transition in this review series is landed, marks this gap complete, activates only `R6-REPLAY` with active packet `none`, and awaits fresh transition review. Do not start replay, assign a terminal scorer disposition, close R6, or begin R7/R8 work in this transition.

## Locked Decisions

1. Preserve witness `59f098b35` and `CTX-R6-15` as the historical red result `60 / Low / Active`, flagged, with command-action evidence. Separately preserve the locked target `0 / Low / Cleared`, unflagged, with empty evidence.
2. Treat `truth_artifacts` plus only non-`observed_command` working-set paths as authority. Observed command paths describe action; they cannot authorize themselves.
3. Keep the repair inside `score_wrong_plan_branch` and `wrong_plan_branch.rs`. The matching test file is optional only for a necessary distinct regression.
4. Make the minimum empty-effective-authority change; preserve all non-empty-authority scoring and evidence semantics.
5. Prove the exact witness and protected controls before the family, checkpoint, full-analyzer, and static walls.
6. Keep proof receipt and phase transition separate. The transition activates `R6-REPLAY` but does not execute it or assign a terminal scorer disposition.

## Ordered Execution

### 0. Commit And Review The Packet Docs — Complete

Packet-doc series `8734f4dbe` + `334e7c6ac` contains only the three canonical packet docs and received fresh independent built-in `default` `REVIEW CLEAN`.

### 1. Reconcile The Review-Clean Packet Across Current Authority — Complete / Historical Review-Clean Gate

Historical execution contract (completed): after Step 0 became review-clean, the separate packet-gate authority series reconciled every live surface whose current status or next action still said these packet files were absent or that packet creation was next:

- root `SPEC.md`, `tasks/plan.md`, and `tasks/todo.md`;
- `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md` and `docs/specs/r6/MAP.md`;
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`;
- control-pack `00-README.md`, `01-authority-and-status-map.md`, `02-phase-and-gate-map.md`, `05-proof-decision-regression-ledger.md`, and `06-operator-prompt-library.md`;
- this packet's SPEC, PLAN, and TASKS status/task mirrors;
- the completed `R6-GAP-TGG-TRUTH-PATH-ACTION` SPEC, PLAN, and TASKS current-status/next-action mirrors; and
- `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md`, `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-plan.md`, and `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-tasks.md`.

That historical reconciliation recorded the review-clean packet-doc series and exact packet paths, kept `R6-GAP-WPB-EMPTY-AUTHORITY` `ACTIVE`, set `ACTIVE_PACKET: R6-GAP-WPB-EMPTY-AUTHORITY`, kept `R6-REPLAY` `BLOCKED`, and made the exact `CTX-R6-15` witness reconfirmation the sole next action. It preserved historical witness `59f098b35` as `60 / Low / Active`, flagged, with command-action evidence, and the separate locked target as `0 / Low / Cleared`, unflagged, with empty evidence, without adding proof or a terminal disposition.

The historical block on witness execution cleared only after Steps 0 and 1 were each committed and review-clean.

Historical result: Step 0 is complete at review-clean series `8734f4dbe` + `334e7c6ac`. Step 1 is complete at packet-gate authority series `eb24b59da` + `e7006f4b2`, which received fresh independent built-in `default` `REVIEW CLEAN` and authorized Step 2. This gate is closed and does not route current operators backward.

### 2. Reconfirm The Witness And Select One Route

Run the exact `CTX-R6-15` control, then the full `wrong_plan_branch` family, and preserve all output.

- Red: proceed to Step 3A.
- Green with an identified earlier sequential named-gap commit that causally made it pass: proceed to Step 3B.
- Green without that causal receipt basis: preserve the witness output and escalate for authority reconciliation.

### 3A. Production-Fix Route

1. Run and record:

   ```bash
   npx gitnexus impact score_wrong_plan_branch -r 97a0-substrate --direction upstream --depth 3
   ```

   Impact every additional existing symbol before editing it. Stop on HIGH/CRITICAL or a required edit beyond the bounded scorer/test files.
2. Change only `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs`, plus `crates/agent-drift-analyzer/tests/wrong_plan_branch.rs` only if a distinct narrow regression is required. Empty effective authority must yield no claim; do not alter the authority model or adjacent branches.
3. Run the exact witness and four protected regressions, then the owning family, checkpoints, full analyzer, format, check, literal all-target clippy, and diff check in the SPEC order.
4. Record actual impact, proof, disposition, and failure details in TASKS and only the corresponding ledger evidence.

### 3B. Already-Green No-Code Receipt

Change no source or test. Cite the already-landed, review-clean earlier named-gap commit that causally made `CTX-R6-15` pass. Run the same complete verification wall as Step 3A and record actual results in this TASKS and the corresponding ledger cells. A merely green current result without causal proof is not eligible.

### 4. Commit And Fresh-Review The Closure Candidate

For production: stage only the scorer file, any justified focused test change, this TASKS, and the corresponding ledger evidence. For no-code: stage only this TASKS and that ledger. Run:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Commit atomically. Dispatch a fresh built-in `default` reviewer. Land each actionable finding in a new batch limited to the files allowed for the reviewed route, rerun affected focused proof before the full wall, and repeat with a fresh reviewer until `REVIEW CLEAN`.

### 5. Transition To Replay Authority And Stop

After Step 4 is committed and review-clean, use the complete Required Phase-Transition Authority Manifest from the landed R6-C.1 TASKS. In one authority-only transition:

- mark `R6-GAP-WPB-EMPTY-AUTHORITY` complete in its packet/status mirrors and named-gap subledger;
- make the generic `R6-GAP-*` row complete;
- activate only `R6-REPLAY` and record its Prompt 1 invocation as the next eligible interaction;
- reconcile root, R6-C.1, control-pack, ledger, operator-prompt, and conditional status/proof mirrors with actual hashes and proof;
- assign no terminal `wrong_plan_branch` disposition and make no R6-close claim;
- run no replay command and start no replay packet or fixture work.

Run the staged gate, commit, and dispatch a fresh independent reviewer. Apply transition-only findings in new transition-only commits and repeat until clean. Stop at that review-clean transition.

## Escalation Boundary

Escalate only for HIGH/CRITICAL impact, an unavoidable edit outside the scorer-local boundary, an already-green witness without an eligible causal earlier-gap commit, unavailable preserved evidence, unisolatable unrelated work, or review proving the locked authority semantics incomplete. Routine red proof, LOW/MEDIUM impact, tests, commits, and review fixes remain autonomous.
