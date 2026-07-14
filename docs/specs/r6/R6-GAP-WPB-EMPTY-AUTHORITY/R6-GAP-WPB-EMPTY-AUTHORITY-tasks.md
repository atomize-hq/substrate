# Tasks: R6-GAP-WPB-EMPTY-AUTHORITY

Status: **ACTIVE — PACKET DOCS REVIEW CLEAN; PACKET-GATE AUTHORITY RECONCILIATION CANDIDATE; PRESERVED RED; NO GAP EXECUTION YET**. Witness `59f098b35` remains the only `CTX-R6-15` behavior receipt: historical `60 / Low / Active`, flagged, with command-action evidence, against the separate locked target `0 / Low / Cleared`, unflagged, with empty evidence. Packet-doc series `8734f4dbe` + `334e7c6ac` received fresh independent built-in `default` `REVIEW CLEAN`; active packet is `R6-GAP-WPB-EMPTY-AUTHORITY`. Task 1 is the current uncommitted authority-only candidate, not a review-clean result. Do not check execution or transition tasks until their exact commit, verification, and fresh-review results exist.

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

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.1 — Reconcile the review-clean packet across current authority.**
  - Prerequisite: Task 0 committed and fresh-review-clean.
  - Files: exactly root `SPEC.md`, `tasks/plan.md`, `tasks/todo.md`; `docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md`; `docs/specs/r6/MAP.md`; `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`; control-pack `00-README.md`, `01-authority-and-status-map.md`, `02-phase-and-gate-map.md`, `05-proof-decision-regression-ledger.md`, and `06-operator-prompt-library.md`; this packet's SPEC, PLAN, and TASKS status/task mirrors; the completed `R6-GAP-TGG-TRUTH-PATH-ACTION` SPEC, PLAN, and TASKS current-status/next-action mirrors; and `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-spec.md`, `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-plan.md`, and `docs/specs/r6/R6-C.1/agent-drift-analyzer-scorer-context-applicability-acceptance-controls-tasks.md`.
  - Update: replace this named-gap row's three `TO CREATE` markers with actual SPEC/PLAN/TASKS paths; record the actual review-clean packet-doc series; remove only current-status/next-action claims that the files are absent or packet creation is next; keep this gap `ACTIVE`; set `ACTIVE_PACKET: R6-GAP-WPB-EMPTY-AUTHORITY`; keep `R6-REPLAY` `BLOCKED`; and make exact `CTX-R6-15` witness reconfirmation the sole next action.
  - Honesty: preserve witness `59f098b35` as historical `60 / Low / Active`, flagged, with command-action evidence; preserve target `0 / Low / Cleared`, unflagged, with empty evidence; add no new proof result, terminal scorer disposition, or gap-complete claim.
  - Commit/review: separate authority-only commit, staged gate, fresh built-in `default` review, authority-only fixes and fresh reviewers until clean.
  - Blocking rule: no Task 2 witness, Task 3 impact/edit, or no-code receipt begins first.
  - Result: **CURRENT UNCOMMITTED CANDIDATE**. The authority-only reconciliation is present in the worktree but has no commit or review verdict yet. Exact `CTX-R6-15` witness reconfirmation remains blocked until this candidate is committed and fresh-review-clean.

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.2 — Reconfirm the preserved witness and select one route.**
  - Prerequisite: Tasks 0 and 1 each committed and fresh-review-clean.
  - Witness: `CTX-R6-15` at commit `59f098b35`.
  - Run:

    ```bash
    cargo test -p agent-drift-analyzer --test wrong_plan_branch wrong_plan_branch_makes_no_claim_for_path_action_without_authority -- --exact --nocapture
    cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
    ```

  - Red: select Task 3A.
  - Green: select Task 3B only when an already-landed, review-clean earlier sequential named-gap commit is demonstrably causal. Otherwise preserve output and escalate for authority reconciliation.
  - Result: pending; do not rerun during the packet-docs gate.

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.3A — Land the smallest scorer-local production fix.**
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
  - Result: pending.

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.3B — Record an eligible already-green no-code proof receipt.**
  - Eligibility: Task 2 is green because an identified, already-landed, review-clean earlier sequential named-gap fix changed the live result. A green result alone is insufficient.
  - Files: only this TASKS and the corresponding `CTX-R6-15` / named-gap ledger evidence. No source or test edit.
  - Verify: run the complete Task 3A proof wall unchanged and record the causal commit plus exact output.
  - Commit/review: atomic docs-only proof receipt, staged gate, fresh built-in `default` review; bounded docs-only fixes and fresh reviewers until clean. The gap remains active; transition stays separate.
  - Ineligible handling: if no causal commit can be proven, preserve output and escalate rather than check this task.
  - Result: pending route selection.

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.4 — Obtain a review-clean closure candidate.**
  - Require exactly one of Tasks 3A/3B committed with actual proof and a fresh `REVIEW CLEAN` verdict.
  - Record all closure-series hashes, proof results, reviewer findings, fix dispositions, and final verdict here and in only the corresponding ledger evidence.
  - Do not mark the gap complete or activate replay in a closure candidate/receipt commit.
  - Result: pending.

- [ ] **R6-GAP-WPB-EMPTY-AUTHORITY.5 — Land and independently review the narrow transition to `R6-REPLAY`.**
  - Prerequisite: Tasks 0, 1, and 4 review-clean.
  - Apply every surface in the landed R6-C.1 Required Phase-Transition Authority Manifest.
  - Required status: mark only this final named gap complete; mark the generic gap row complete; activate only `R6-REPLAY`; record actual commits/proof and the filled Prompt 1 replay invocation as the sole next interaction.
  - Exclusions: no production/test edit, replay command, replay fixture/packet work, terminal `wrong_plan_branch` disposition, R6-close claim, R7 work, or R8 work.
  - Commit/review: authority-only staged gate, separate commit, fresh independent reviewer, transition-only fix commits and fresh reviewers until clean.
  - Stop: review-clean transition committed; do not start `R6-REPLAY`.
  - Result: pending.

## Proof And Review Receipt Template

Fill only from live output:

- route and causal commit, if no-code:
- pre-edit impact risk/callers/processes/modules:
- focused `CTX-R6-15` result:
- read-only regression result:
- sanctioned-replan regression result:
- opaque-parent regression result:
- later-return regression result:
- full `wrong_plan_branch` family result:
- checkpoint result:
- full analyzer result:
- fmt/check/literal-clippy/diff results:
- staged `detect-changes` scope/risk:
- closure commit(s):
- review findings/fixes/final verdict:
- ledger/status updates:

## Explicit Exclusions

No context/task-frame builder edit, command-classification change, other scorer, dispatcher, schema/export, fixture, replay, sentinel/presentation, public API, terminal scorer disposition, R7, R8, or successor execution belongs in this gap.
