# Plan: R6-C.1 — Scorer Context Applicability Acceptance Controls

Status: **DRAFT / DOCS ONLY** on 2026-07-13. This plan is ordered and test-first. It does not
authorize control implementation until the `R6-C.1-SPEC` docs lock is committed and review-clean.

## Plan Decisions

1. Land the docs lock first; then implement controls one scorer family at a time in this order:
   `dead_end_thrash`, `truth_grounding_gap`, `wrong_plan_branch`.
2. Each scorer-control task is one atomic test-only commit and one fresh independent review boundary.
   A red control remains in that commit as a preserved witness; production does not change in the same
   commit.
3. Each failed test opens exactly one conditional packet for that scorer/failure seam. Gap packets run
   sequentially and never batch fixes, even when several tests fail in one scorer family.
4. The two likely red semantics are already adjudicated by scorer responsibility: truth-path action
   before read is `80 / High / Active`; empty-authority wrong-branch is no-claim
   `0 / Low / Cleared / empty evidence`. Current source behavior does not override these contracts.
5. Equal `SessionProgress` yields equal `dead_end_thrash` scoring across turn shapes. The contract is
   narrowed to upstream consumption rather than inventing an independent turn counter in the scorer.
6. Dispatcher exact ordering is source-proven infrastructure, not a closure behavior contract. No
   focused ordering test is planned.
7. Integrated advancing/true-stall replay and frozen-corpus preservation execute later in `R6-REPLAY`.

## Dependency Graph

```text
R6-C.1-SPEC docs lock + fresh review
  -> dead_end_thrash controls/witness commit + fresh review
  -> truth_grounding_gap controls/witness commit + fresh review
  -> wrong_plan_branch controls/witness commit + fresh review
  -> packet checkpoint (three focused families + checkpoints + diff check)
  -> one sequential R6-GAP-* packet per red scorer/failure seam
       -> red witness already committed
       -> GitNexus impact for exact symbol
       -> smallest production fix
       -> focused test + scorer family wall
       -> detect-changes + atomic fix commit + fresh review
  -> controls exit-gate reconciliation + fresh review
  -> R6-REPLAY (separate phase; not started here)
```

## Ordered Execution

### R6-C.1.0 — Docs Lock

- Commit only the three files in this directory.
- Validate all named tests, all `CTX` IDs, current source/test paths, commands, and `git diff --check`.
- Run `npx gitnexus detect-changes -r 97a0-substrate`, inspect the staged diff, commit atomically, and
  dispatch a fresh built-in `default` reviewer.
- Fix docs findings in a new commit and repeat fresh review until clean.
- Transition authority may activate `R6-C.1-CONTROLS` only after that review-clean docs commit.

**Atomic boundary:** one docs commit; no root/control-pack status edit is part of the authoring batch.

### R6-C.1.1 — `dead_end_thrash` Acceptance Controls

1. Load only the landed packet docs, `dead_end_thrash.rs`, `tests/dead_end_thrash.rs`, the smallest
   named checkpoint/progress builders, and one analogous existing test.
2. Add the three exact tests from the SPEC in matrix order: regression, opaque parent, equal-progress
   turn-shape equivalence.
3. Run each exact focused command after adding its test, then:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
```

4. Record each deterministic PASS or exact red witness in TASKS and the corresponding ledger row.
5. Run detect-changes and `git diff --check`; commit test-only controls/witnesses atomically; fresh review.

**Stop:** do not edit `score_dead_end_thrash` in this task. Each red routes to its named, separate
`R6-GAP-DET-*` packet.

### R6-C.1.2 — `truth_grounding_gap` Acceptance Controls

1. Load only the landed packet docs, `truth_grounding_gap.rs`, `tests/truth_grounding_gap.rs`, the
   smallest command-observation/task-frame seam, and one analogous existing test.
2. Add the six exact tests from the SPEC in ledger order: no-action planning, typed-success ungrounded
   verification, turn-shape invariance, opaque parent, truth-path action before read, and action-bound
   archetype invariance.
3. Run each exact focused command, then:

```bash
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
```

4. Preserve any red result, including the expected current-source conflict for truth-path action before
   read; update TASKS/ledger with actual output, not intended behavior.
5. Run detect-changes and `git diff --check`; commit test-only controls/witnesses atomically; fresh review.

**Stop:** do not edit `score_truth_grounding_gap` here. Each red routes to one named
`R6-GAP-TGG-*` packet.

### R6-C.1.3 — `wrong_plan_branch` Acceptance Controls

1. Load only the landed packet docs, `wrong_plan_branch.rs`, `tests/wrong_plan_branch.rs`, the smallest
   working-set/replan/delegation seam, and one analogous existing test.
2. Add the four exact tests from the SPEC in matrix order: read-only exploration, sanctioned replan
   scope, opaque parent, empty authority no-claim.
3. Run each exact focused command, then:

```bash
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
```

4. Preserve any red result, including the expected current-source conflict for empty authority; update
   TASKS/ledger with actual output.
5. Run detect-changes and `git diff --check`; commit test-only controls/witnesses atomically; fresh review.

**Stop:** do not edit `score_wrong_plan_branch` here. Each red routes to one named
`R6-GAP-WPB-*` packet.

### R6-C.1.4 — Controls Checkpoint And Gap Routing

Run the family checkpoint once after all three test-only commits exist:

```bash
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer wrong_plan_branch -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
git diff --check
```

- Green controls become fit-for-purpose proof.
- Red controls remain committed witnesses. For each red, create one bounded packet using its exact gap
  route; do not batch by scorer or family.
- If no control is red, make no production change.
- Do not run the full analyzer/replay closeout as a substitute for unresolved focused failures.

### R6-C.1.5+ — Conditional Gap Packets, Sequential

For each red witness, in plan order:

1. Create the narrow gap SPEC/PLAN/TASKS using only the witness, owning scorer, directly relevant
   upstream context, and one regression pattern. Exclude R7 and unrelated scorers.
2. Run before editing the exact indexed symbol:

```bash
npx gitnexus impact score_dead_end_thrash -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_truth_grounding_gap -r 97a0-substrate --direction upstream --depth 3
npx gitnexus impact score_wrong_plan_branch -r 97a0-substrate --direction upstream --depth 3
```

Run only the command for the active scorer, plus a separate exact-symbol impact command for any
directly relevant upstream function the gap packet proposes to edit.

3. Warn and stop on HIGH/CRITICAL impact. Otherwise make the smallest production change that turns the
   preserved witness green without weakening other controls.
4. Run the exact witness, the owning scorer family, then the packet checkpoint commands affected by the
   change.
5. Update the gap TASKS and corresponding `CTX-R6-*` row with actual proof.
6. Run `npx gitnexus detect-changes -r 97a0-substrate` and `git diff --check`; inspect staged scope;
   commit the fix atomically.
7. Dispatch a fresh built-in `default` reviewer. Fix findings in a new commit and repeat with another
   fresh reviewer until clean.

**No batching:** one scorer/failure seam, one fix packet, one atomic fix/review loop.

### R6-C.1.6 — Controls Exit Gate And Transition

The controls exit gate is met only when every control has a deterministic recorded result and every
red control is preserved and routed. Re-run the packet checkpoint after conditional fixes. Then update
canonical TASKS/ledger/status wording with actual results in a narrow transition commit, run
detect-changes and `git diff --check`, and dispatch a fresh independent transition reviewer.

The transition may activate `R6-REPLAY` only when all conditional gaps are review-clean. Do not start
replay. Replay selects trusted real-rollout-derived advancing and true-stall fixtures and owns
`CTX-R6-01`, `CTX-R6-02`, and frozen invariance `CTX-R6-06`.

## Ledger And Status Update Points

| Boundary | Allowed updates |
|---|---|
| Docs lock | These three docs; phase transition separately marks `R6-C.1-SPEC` complete and `R6-C.1-CONTROLS` active. No control result changes. |
| Scorer control commit | Only its TASKS rows and `CTX-R6-*` result rows justified by exact output. |
| Gap witness/fix | Owning gap docs, owning control row, actual proof receipts. No terminal scorer disposition. |
| Controls exit | Canonical task/ledger/status agreement; route to `R6-REPLAY` if no conditional gap remains. |
| Replay / close | Deferred to `R6-REPLAY` / `R6-CLOSE`; never anticipated in this phase. |

## Exact Planned Test Inventory

```text
dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity
dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity
dead_end_thrash_scores_equal_progress_equally_across_turn_shapes
truth_grounding_gap_keeps_no_action_planning_clear
truth_grounding_gap_flags_successful_verification_without_truth_reads
truth_grounding_gap_is_event_order_invariant_across_turn_shapes
truth_grounding_gap_keeps_opaque_parent_orchestration_clear_without_child_action
truth_grounding_gap_flags_truth_path_action_before_read
truth_grounding_gap_scores_equivalent_actions_equally_across_archetypes
wrong_plan_branch_ignores_read_only_out_of_scope_exploration
wrong_plan_branch_accepts_write_under_sanctioned_replan_scope
wrong_plan_branch_keeps_opaque_parent_orchestration_clear_without_child_action
wrong_plan_branch_makes_no_claim_for_path_action_without_authority
acceptance_fixtures_integrated_advancing_repeated_failures_stay_unflagged
acceptance_fixtures_integrated_true_stall_stays_active
```

The first thirteen names belong to `R6-C.1-CONTROLS`; the last two are specified here but owned and
executed by `R6-REPLAY`.

## Risks And Mitigations

| Risk | Mitigation |
|---|---|
| Current behavior is mistaken for desired semantics. | The SPEC locks responsibility-derived expectations before tests are written. |
| A failing control prompts an immediate production patch. | Commit and review the red witness first; open a separate gap packet. |
| Several failures are fixed together. | One named gap route per matrix row; sequential packets only. |
| Turn/archetype context is forced into irrelevant scorers. | Assert invariance/equivalence at the public score; consume only existing upstream evidence. |
| R7 absorbs an R6 baseline gap. | Gap routes remain `R6-GAP-*`; R7 stays blocked until R6 closes. |
| Source order is overclaimed as behavior proof. | Keep `CTX-R6-16` source-only; no order test absent load-bearing evidence. |
| Replay claims are promoted from synthetic controls. | `CTX-R6-01`/`02` remain owned by `R6-REPLAY` with trusted fixtures. |

## Stop And Escalation Rules

Stop only the affected scope and use the operator library's structured escalation when authority cannot
choose between materially different product meanings, GitNexus is HIGH/CRITICAL, unrelated work cannot
be isolated, a trusted replay artifact is unavailable, or review proves the packet boundary invalid.
Ordinary red controls, bounded fixes, tests, commits, and review findings are autonomous work. Do not
ask for routine approval.

## Non-Goals

No code/test/fixture change in `R6-C.1-SPEC`; no replay closeout; no R6 closure; no R7/R8 work; no
`semantic_goal_drift`; no dispatcher-order contract.
