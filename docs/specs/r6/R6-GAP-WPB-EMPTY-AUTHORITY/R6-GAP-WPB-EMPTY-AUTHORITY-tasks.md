# Tasks: R6-GAP-WPB-EMPTY-AUTHORITY

Status: **COMPLETE — TASK 3A AND CLOSURE SERIES REVIEW CLEAN; AUTHORITY TRANSITION REVIEW-FIX SERIES LANDED; FRESH RE-REVIEW PENDING**. Witness `59f098b35` remains the historical `CTX-R6-15` red receipt: `60 / Low / Active`, flagged, with command-action evidence, against the separate locked target `0 / Low / Cleared`, unflagged, with empty evidence. Packet-doc series `8734f4dbe` + `334e7c6ac`, packet-gate authority series `eb24b59da` + `e7006f4b2`, and implementation/review-fix series `6b42e5476` + `e65df2561` + `cd4e24119` each received fresh independent built-in `default` `REVIEW CLEAN`. The implementation reviewer found no code or test defect and independently confirmed the exact witness, all four protected controls, `6 / 6` family, `35` unit + `131` integration checkpoint matches plus matching tests, the full analyzer at the recorded counts, format, check, literal all-target clippy with `-D warnings`, and diff check green; comparison detect was `LOW` across exactly `3` files and `8` changed symbols with `0` affected processes. Landed transition candidate `56bb9966f` marks this final named gap and aggregate `R6-GAP-*` complete and activates only `R6-REPLAY` with active packet `none`; its first fresh transition review found only one P2 stale historical Step 1 gate description in the packet PLAN. The follow-up correction in this review series is landed, and fresh re-review is the sole next action before replay work may start. Task 5 remains unchecked until that re-review is clean; no terminal `wrong_plan_branch` disposition, replay execution, R6 close, or R7/R8 work is claimed.

## Required Gates

Before every commit:

```bash
git add -- <intended-files-only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached
```

Before editing the owning symbol:

```bash
npx gitnexus impact score_wrong_plan_branch -r 97a0-substrate --direction upstream --depth 3
```

Record risk, callers, processes, and modules. Impact every other existing symbol separately before editing it. Warn and stop on HIGH/CRITICAL.

## Task Ledger

- [x] **R6-GAP-WPB-EMPTY-AUTHORITY.0 — Commit and independently review the packet docs.**
  - Files: exactly this SPEC, PLAN, and TASKS.
  - Sanity: confirm all three canonical paths exist; every named source/test path exists; every named test is present exactly once in `crates/agent-drift-analyzer/tests/wrong_plan_branch.rs`; SPEC/PLAN/TASKS use the same control, scope, commands, route rules, historical red result, and separate locked target.
  - Verify: path/link sanity, `git diff --check`, then the staged gate and complete staged-diff inspection.
  - Commit/review: first commit contains only these three files; fresh built-in `default` review; docs-only fix commits and fresh reviewers until clean.
  - Result: **COMPLETE**. Packet-doc series `8734f4dbe` + `334e7c6ac` contains only this SPEC, PLAN, and TASKS family and received fresh independent built-in `default` `REVIEW CLEAN`. This result adds no behavior proof and does not execute the gap.

- [x] **R6-GAP-WPB-EMPTY-AUTHORITY.1 — Reconcile the review-clean packet across current authority.**
  - Prerequisite: Task 0 committed and fresh-review-clean.
  - Files: exactly root `SPEC.md`, `tasks/plan.md`, `tasks/todo.md`; `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`; `docs/specs/r6/MAP.md`; `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`; control-pack `00-README.md`, `01-authority-and-status-map.md`, `02-phase-and-gate-map.md`, `05-proof-decision-regression-ledger.md`, and `06-operator-prompt-library.md`; this packet's SPEC, PLAN, and TASKS status/task mirrors; the completed `R6-GAP-TGG-TRUTH-PATH-ACTION` SPEC, PLAN, and TASKS current-status/next-action mirrors; and `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md`, `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-plan.md`, and `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-tasks.md`.
  - Update: replace this named-gap row's three `TO CREATE` markers with actual SPEC/PLAN/TASKS paths; record the actual review-clean packet-doc series; remove only current-status/next-action claims that the files are absent or packet creation is next; keep this gap `ACTIVE`; set `ACTIVE_PACKET: R6-GAP-WPB-EMPTY-AUTHORITY`; keep `R6-REPLAY` `BLOCKED`; and make exact `CTX-R6-15` witness reconfirmation the sole next action.
  - Honesty: preserve witness `59f098b35` as historical `60 / Low / Active`, flagged, with command-action evidence; preserve target `0 / Low / Cleared`, unflagged, with empty evidence; add no new proof result, terminal scorer disposition, or gap-complete claim.
  - Commit/review: separate authority-only commit, staged gate, fresh built-in `default` review, authority-only fixes and fresh reviewers until clean.
  - Blocking rule: no Task 2 witness, Task 3 impact/edit, or no-code receipt begins first.
  - Result: **COMPLETE**. Packet-gate authority series `eb24b59da` + `e7006f4b2` received fresh independent built-in `default` `REVIEW CLEAN`. That review-clean boundary authorized Task 2 without adding behavior proof, closing the gap, or unblocking replay.

- [x] **R6-GAP-WPB-EMPTY-AUTHORITY.2 — Reconfirm the preserved witness and select one route.**
  - Prerequisite: Tasks 0 and 1 each committed and fresh-review-clean.
  - Witness: `CTX-R6-15` at commit `59f098b35`.
  - Run:

    ```bash
    cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture
    cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
    ```

  - Red: select Task 3A.
  - Green: select Task 3B only when an already-landed, review-clean earlier sequential named-gap commit is demonstrably causal. Otherwise preserve output and escalate for authority reconciliation.
  - Result: **COMPLETE — TASK 3A SELECTED**. Before the edit, the exact witness exited `101` with `0 passed; 1 failed` and reproduced actual `60 / Low / Active`, flagged, versus required `0 / Low / Cleared`, unflagged; the owning family also exited `101` with `5 passed; 1 failed`, and only `CTX-R6-15` failed. The preserved red therefore selected the bounded production-fix route; Task 3B is ineligible.

- [x] **R6-GAP-WPB-EMPTY-AUTHORITY.3A — Land the smallest scorer-local production fix.**
  - Eligibility: Task 2 remains red; pre-edit impact is below HIGH; no wider symbol/file is required.
  - Files: `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs`; `crates/agent-drift-analyzer/tests/wrong_plan_branch.rs` only for a necessary distinct regression; this TASKS; and only changed `CTX-R6-15` / named-gap evidence in the canonical ledger.
  - Minimal rule: when `truth_artifacts` and all non-`observed_command` working-set authority are empty, a path-bearing command cannot be judged out of scope against an empty allowed set. Produce `0 / Low / Cleared`, unflagged, empty evidence. Preserve every non-empty-authority path.
  - Required proof, in order:

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

  - Record: impact details; exact per-control tuples; family/checkpoint/full-suite counts; static command exits; staged detect scope/risk; changed files; commit hash; and any failure disposition.
  - Commit/review: atomic staged gate, commit, fresh built-in `default` reviewer; new bounded fix commit and fresh reviewer until clean.
  - Result: **COMPLETE — IMPLEMENTATION/REVIEW-FIX SERIES `REVIEW CLEAN`**. Exact pre-edit GitNexus impact for `score_wrong_plan_branch` was `LOW`: `0` direct callers, `0` affected processes, and `0` affected modules. Commit `6b42e5476` changes only `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs` in production: command observations are skipped when the deduplicated effective-authority set is empty. No test file changed because the preserved exact witness already guards the behavior. Post-edit proof is green: exact `CTX-R6-15` and each of the four protected controls passed `1 / 1` with exit `0`; the target now asserts `0 / Low / Cleared`, unflagged, with empty evidence; read-only, sanctioned-replan, and opaque-parent controls each retain `0 / Medium / Cleared`, unflagged, with empty evidence; the later-return control retains first-interval `60 / Active`, flagged, then later `0 / Cleared`, unflagged. The full `wrong_plan_branch` family passed `6 / 6`; checkpoint proof passed `35` unit + `131` integration plus the matching export/truth tests; the full analyzer passed `156` unit, `6` acceptance, `133` checkpoint, `2` context, `18` dead-end, `5` end-to-end, `21` export, `12` input, `6` objective, `3` progress, `2` semantic, `1` task-frame, `22` truth-grounding, and `6` wrong-branch tests. Format, check, literal all-target clippy with `-D warnings`, and diff check each exited `0`. Pre-commit staged GitNexus detect reported `LOW` risk across exactly `3` files and `8` changed symbols with `0` affected processes; cached diff check passed. The first two fresh reviews found only receipt-state wording defects, with no code or test defect; commits `e65df2561` and `cd4e24119` corrected them. A fresh independent built-in `default` reviewer then returned `REVIEW CLEAN` for full series `6b42e5476` + `e65df2561` + `cd4e24119`.

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.3B — Record an eligible already-green no-code proof receipt.**
  - Eligibility: Task 2 is green because an identified, already-landed, review-clean earlier sequential named-gap fix changed the live result. A green result alone is insufficient.
  - Files: only this TASKS and the corresponding `CTX-R6-15` / named-gap ledger evidence. No source or test edit.
  - Verify: run the complete Task 3A proof wall unchanged and record the causal commit plus exact output.
  - Commit/review: atomic docs-only proof receipt, staged gate, fresh built-in `default` review; bounded docs-only fixes and fresh reviewers until clean. The gap remains active; transition stays separate.
  - Ineligible handling: if no causal commit can be proven, preserve output and escalate rather than check this task.
  - Result: **NOT SELECTED**. The exact witness was red before the candidate edit, so only Task 3A is eligible.

- [x] **R6-GAP-WPB-EMPTY-AUTHORITY.4 — Obtain a review-clean closure candidate.**
  - Require exactly one of Tasks 3A/3B committed with actual proof and a fresh `REVIEW CLEAN` verdict.
  - Record all closure-series hashes, proof results, reviewer findings, fix dispositions, and final verdict here and in only the corresponding ledger evidence.
  - Do not mark the gap complete or activate replay in a closure candidate/receipt commit.
  - Result: **COMPLETE — FRESH INDEPENDENT `REVIEW CLEAN`**. Task 3A closure candidate `6b42e5476` and docs-only receipt-state corrections `e65df2561` + `cd4e24119` are landed. The first two fresh reviewers returned P2 findings only for stale/self-future landed-state bookkeeping; neither found a code or test defect. A fresh independent built-in `default` reviewer returned `REVIEW CLEAN` for the full implementation/review-fix series, completing Tasks 3A and 4 and authorizing only the separate authority transition.

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.5 — Land and independently review the narrow transition to `R6-REPLAY`.**
  - Prerequisite: Tasks 0, 1, and 4 review-clean.
  - Apply every surface in the landed R6-C.1 Required Phase-Transition Authority Manifest.
  - Required status: mark only this final named gap complete; mark the generic gap row complete; activate only `R6-REPLAY`; record actual commits/proof and the filled Prompt 1 replay invocation as the sole next interaction.
  - Exclusions: no production/test edit, replay command, replay fixture/packet work, terminal `wrong_plan_branch` disposition, R6-close claim, R7 work, or R8 work.
  - Commit/review: authority-only staged gate, separate commit, fresh independent reviewer, transition-only fix commits and fresh reviewers until clean.
  - Stop: review-clean transition committed; do not start `R6-REPLAY`.
  - Result: **LANDED TRANSITION CANDIDATE; FIRST REVIEW `REVIEW FINDINGS` P2; FOLLOW-UP CORRECTION LANDED; FRESH RE-REVIEW PENDING**. Transition candidate `56bb9966f` marks `R6-GAP-WPB-EMPTY-AUTHORITY` and aggregate `R6-GAP-*` complete, activates only `R6-REPLAY` with active packet `none`, and records Prompt 1 replay invocation as the sole next interaction after review-clean. Its first fresh independent built-in `default` reviewer found only one P2 receipt-state defect: this packet PLAN still described Step 1 and `eb24b59da` as pending/not review-clean although packet-gate series `eb24b59da` + `e7006f4b2` is review-clean. The follow-up correction in this review series is landed and makes Step 1 a closed historical gate. This task remains unchecked until a fresh independent reviewer returns `REVIEW CLEAN` for `56bb9966f` plus that correction; no replay command or successor work has started.

## Proof And Review Receipt Template

Fill only from live output:

- route and causal commit, if no-code: production-fix Task 3A selected from the reproduced red; no-code route not eligible.
- pre-edit impact risk/callers/processes/modules: `LOW`; `0` direct callers; `0` affected processes; `0` affected modules.
- focused `CTX-R6-15` result: `1 / 1`, exit `0`; `0 / Low / Cleared`, unflagged, empty evidence.
- read-only regression result: `1 / 1`, exit `0`; `0 / Medium / Cleared`, unflagged, empty evidence.
- sanctioned-replan regression result: `1 / 1`, exit `0`; `0 / Medium / Cleared`, unflagged, empty evidence.
- opaque-parent regression result: `1 / 1`, exit `0`; `0 / Medium / Cleared`, unflagged, empty evidence.
- later-return regression result: `1 / 1`, exit `0`; first interval `60 / Active`, flagged, then later `0 / Cleared`, unflagged.
- full `wrong_plan_branch` family result: `6 / 6`, exit `0`.
- checkpoint result: `35` unit + `131` integration plus matching export/truth tests, exit `0`.
- full analyzer result: `156` unit + `6` acceptance + `133` checkpoint + `2` context + `18` dead-end + `5` end-to-end + `21` export + `12` input + `6` objective + `3` progress + `2` semantic + `1` task-frame + `22` truth-grounding + `6` wrong-branch tests, all passed; exit `0`.
- fmt/check/literal-clippy/diff results: each exit `0`.
- staged `detect-changes` scope/risk: `LOW`; exactly `3` files and `8` changed symbols; `0` affected processes; cached diff check passed.
- closure commit(s): `6b42e5476` is the landed Task 3A closure candidate; `e65df2561` + `cd4e24119` are the landed docs-only receipt-state corrections.
- review findings/fixes/final verdict: the first two fresh independent built-in `default` reviews returned P2 findings only for stale/self-future landed-state bookkeeping; neither found a code or test defect. A fresh independent built-in `default` reviewer returned `REVIEW CLEAN` for series `6b42e5476` + `e65df2561` + `cd4e24119`.
- ledger/status updates: this TASKS plus `CTX-R6-15`, the named-gap row, and the update record in the canonical ledger now record the review-clean closure. Transition candidate `56bb9966f` is landed; its first fresh review found only the stale historical Step 1 gate wording, the follow-up correction in this review series is landed, and fresh re-review is pending. The gap and aggregate gap phase remain complete, only `R6-REPLAY` is active with active packet `none`, and replay cannot begin before review-clean.

## Explicit Exclusions

No context/task-frame builder edit, command-classification change, other scorer, dispatcher, schema/export, fixture, replay, sentinel/presentation, public API, terminal scorer disposition, R7, R8, or successor execution belongs in this gap.
