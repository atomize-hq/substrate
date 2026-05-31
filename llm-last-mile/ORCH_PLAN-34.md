# ORCH_PLAN-34: Execute TASKS-34 Packet 1 Through Parent-Frozen Steering Policy Surface, Stable Denial Vocabulary Freeze, And Packet-Scoped Validation

Live workspace branch: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Authoritative execution branch for this run: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Authoritative HEAD at controller draft time: `de01faaefa7d2f25d159f2ad0f8e804c1e422e89`  
Plan source: [PLAN-34.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-34.md)  
Spec source: [SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md)  
Validation note: [NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md)  
Tasks source: [TASKS-34.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-34.md)  
Structure reference: [/Users/spensermcconnell/.codex/attachments/89461a37-0f87-4d2c-89e2-cbace7cd879f/pasted-text.txt](/Users/spensermcconnell/.codex/attachments/89461a37-0f87-4d2c-89e2-cbace7cd879f/pasted-text.txt)  
Style references: [ORCH_PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-22.md), [ORCH_PLAN-25.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-25.md)  
Current root controller for context only: [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)  
Execution type: packet-specific orchestration controller for `TASKS-34` Packet 1 only, parent-frozen contract, parent-only gates and integration, no Packet 2-4 execution inside this run  
Live root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-34-packet-1`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Initial concurrent worker cap: `2`  
Total live code lanes in Packet 1: `2`

Live checkout truth frozen for this controller:

- root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/internal-host-orchestrator-world-dispatch-bootstrap`
- HEAD: `de01faaefa7d2f25d159f2ad0f8e804c1e422e89`
- current untracked packet inputs:
  - `llm-last-mile/NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md`
  - `llm-last-mile/SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md`
  - `llm-last-mile/PLAN-34.md`
  - `llm-last-mile/TASKS-34.md`

## Summary

This document is the execution controller for `TASKS-34` Packet 1 only. It is not a restatement of `PLAN-34`, and it does not authorize Packet 2, Packet 3, or Packet 4 work.

Packet 1 is honest only if the merged tree lands exactly these outcomes and then stops:

1. a narrow implementation-bearing steering policy surface exists for the current family-1 world-dispatch verbs,
2. defaults remain deny-by-default,
3. stable steering denial vocabulary and helpers exist for later packets to consume,
4. no pre-routing enforcement beyond helper scaffolding is claimed as landed,
5. no lifecycle-aware invalidation or concurrency behavior beyond Packet 1 surface freezing is claimed as landed,
6. no config/docs closeout is claimed as landed,
7. the Packet 1 validation wall is green on the same merged tree.

The only honest parallelism in Packet 1 begins after the parent freezes the exact policy dimensions and denial-bucket list. At that point there are two disjoint code lanes:

1. `L1` policy-surface freeze in broker and shell policy-model code,
2. `L2` denial-vocabulary and helper freeze in dispatch-contract code plus helper-only plumbing in orchestrator dispatch.

There is no safe third lane. Test work and doc work are not separate Packet 1 lanes because:

1. Packet 1 verification is narrow and tied to the two production ownership islands,
2. a standalone test lane would race the same symbols and assertions it needs to validate,
3. docs closeout belongs to Packet 4, not to this packet.

Frozen Packet 1 execution choices that resolve source ambiguity:

1. `docs/CONFIGURATION.md` does not move in Packet 1 even if internal config keys land; docs closeout stays in Packet 4.
2. Packet 1 freezes two separate first-cap denial buckets:
   - `session_concurrency_cap_exceeded`
   - `worker_concurrency_cap_exceeded`
3. `crates/shell/src/execution/orchestrator_world_dispatch.rs` may change in Packet 1 only for helper scaffolding that centralizes denial vocabulary. It may not perform Packet 2 enforcement in this run.

Frozen run shape:

1. `task/m34p1-p1-parent-contract-freeze-and-run-init`
2. `task/m34p1-g1-window-a-launch-gate`
3. parallel Window A
   - `task/m34p1-l1-policy-surface-freeze`
   - `task/m34p1-l2-denial-vocabulary-freeze`
4. `task/m34p1-g2-window-a-integration-gate`
5. `task/m34p1-p2-parent-window-a-integration`
6. `task/m34p1-g3-validation-wall-gate`
7. `task/m34p1-p3-parent-validation-wall`
8. `task/m34p1-p4-parent-closeout-phase`

## Hard Guards

These are run-stopping invariants, not preferences.

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/internal-host-orchestrator-world-dispatch-bootstrap` at base `de01faaefa7d2f25d159f2ad0f8e804c1e422e89` plus only accepted Packet 1 work.
2. The parent agent is the sole integrator, the sole gate owner, the sole acceptance authority, and the sole writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/**`.
3. This run owns Packet 1 only. No worker may start Packet 2 enforcement, Packet 3 lifecycle or concurrency behavior, or Packet 4 docs closeout.
4. The current untracked Packet 34 planning docs are source inputs only. Workers must not edit them, normalize them, track them, or assume they exist inside fresh worktrees.
5. Packet 1 production scope is limited to:
   - `crates/broker/src/policy.rs`
   - `crates/shell/src/execution/policy_model.rs`
   - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - `crates/shell/src/execution/orchestrator_world_dispatch.rs` helper-only scaffolding if needed
   - inline unit tests in those owned files only
6. Packet 1 must not edit:
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - `crates/shell/tests/**`
   - `docs/**`
   - `ORCH_PLAN.md`
   - `README.md`
   - any `llm-last-mile/*.md` file other than this controller
   - `.runs/**`
7. Defaults remain fail closed. No Packet 1 lane may add permissive fallback behavior, compatibility widening, or a default-allow branch for world dispatch.
8. The stable steering denial vocabulary is frozen to exactly these Packet 1 bucket ids:
   - `world_dispatch_disabled`
   - `backend_not_allowed`
   - `action_not_allowed`
   - `mode_not_allowed`
   - `cross_session_steering_denied`
   - `cross_world_binding_steering_denied`
   - `capability_narrowing_not_allowed`
   - `session_concurrency_cap_exceeded`
   - `worker_concurrency_cap_exceeded`
   - `invalidated_worker_not_routable`
9. Packet 1 may freeze names, helpers, parsing, and contract shape for the in-scope concurrency caps, but it may not claim that concurrency behavior is fully enforced before Packet 3.
10. Packet 1 may freeze names, helpers, and helper call sites for invalidated-worker denial, but it may not claim that retained-worker routability behavior is fully enforced before Packet 2 and Packet 3 complete.
11. `L2` may not turn helper plumbing inside `orchestrator_world_dispatch.rs` into real pre-routing gating. Any actual gate behavior belongs to Packet 2.
12. No worker may widen into later verbs:
   - `inspect_world_worker`
   - `cancel_world_work`
   - `stop_world_worker`
   - `fork_world_worker`
13. No worker may widen into approval/fork autonomy policy, router-owned attach execution, or obligation-ledger work.
14. No worker may edit `.runs/**`, and no worker may create or rewrite run-state artifacts on the parent’s behalf.
15. If Packet 1 cannot land without touching forbidden surfaces, renaming frozen bucket ids, or starting Packet 2 behavior, the run stops and the parent writes `blocked.json`.

Stop the run, write `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-34-packet-1/blocked.json`, and do not advance if any of these occur:

1. A lane requires `state_store.rs`, integration-test files, or docs files to make Packet 1 land.
2. The narrow world-dispatch policy surface cannot be expressed without a broader policy-schema redesign unrelated to current verbs.
3. The stable denial vocabulary cannot be frozen without deciding later-verb or family-2 semantics first.
4. `L2` needs to enforce real pre-routing behavior in `orchestrator_world_dispatch.rs` instead of helper-only scaffolding.
5. The packet can only be made green by documenting newly landed keys in `docs/CONFIGURATION.md` now instead of deferring docs to Packet 4.
6. A worker touches files outside its frozen ownership set.
7. The live checkout truth recorded above changes before Packet 1 completes and the parent cannot re-freeze cleanly.
8. The final Packet 1 validation wall cannot prove deny-by-default policy shape plus stable denial helpers on the same merged tree.

### Blocked-Run Record Contract

`blocked.json` is parent-written only, exactly once, at the moment the parent decides Packet 1 cannot advance.

Required fields in `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-34-packet-1/blocked.json`:

- `run_id`
- `authoritative_root`
- `authoritative_branch`
- `authoritative_head`
- `plan_source`
- `spec_source`
- `tasks_source`
- `timestamp`
- `current_task_id`
- `gate_state`
- `stop_condition_id`
- `summary`
- `blocking_files`
- `worker_lane`
- `accepted_sentinels`
- `rejected_or_quarantined_outputs`
- `next_required_parent_action`

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-34-packet-1`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-34-packet-1/policy-surface-freeze`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-34-packet-1/denial-vocabulary-freeze`

Worker branches:

- `codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-policy-surface-freeze`
- `codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-denial-vocabulary-freeze`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-34-packet-1

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-34-packet-1/policy-surface-freeze \
  -b codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-policy-surface-freeze \
  feat/internal-host-orchestrator-world-dispatch-bootstrap

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-34-packet-1/denial-vocabulary-freeze \
  -b codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-denial-vocabulary-freeze \
  feat/internal-host-orchestrator-world-dispatch-bootstrap
```

Fresh-worktree protocol rules:

1. The parent integrates only in `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.
2. Both worker worktrees are seeded from the exact same post-`p1` tree.
3. The untracked Packet 34 planning inputs are snapshot into `.runs/plan-34-packet-1/source-lock/` before any worker starts so worktrees do not depend on untracked repo state.
4. Workers consume Packet 34 source truth through the parent prompt and source-lock snapshots, not by editing or recreating those packet docs.
5. There is no third worker worktree for Packet 1.

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-34-packet-1/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `authoritative-checkout.json`
- `contract-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `packet-2-handoff.md`
- `blocked.json` on failure only
- `quarantine/`
- `sentinels/`
- `source-lock/`

Required `source-lock/` entries:

- `NOTE-34-family-1-policy-hardening-after-continue-bootstrap.md`
- `SPEC-34-host-to-world-steering-policy-hardening-for-landed-dispatch-surface.md`
- `PLAN-34.md`
- `TASKS-34.md`
- `ORCH_PLAN-22.md`
- `ORCH_PLAN-25.md`
- `ORCH_PLAN.root-context.md`
- `pasted-text.txt`
- `checkout-truth.json`

Required per-task artifact roots:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-p1-parent-contract-freeze-and-run-init/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-g1-window-a-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-l1-policy-surface-freeze/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-l2-denial-vocabulary-freeze/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-g2-window-a-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-p2-parent-window-a-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-g3-validation-wall-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-p3-parent-validation-wall/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m34p1-p4-parent-closeout-phase/`

Each task directory must contain:

- `task.json`
- `commands.txt`
- `summary.md`

Each gate task directory must also contain:

- `gate-checklist.md`
- `gate-result.json`

Each worker task directory must also contain:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`

Required sentinels:

- `.runs/plan-34-packet-1/sentinels/task-m34p1-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-g1-window-a-launch-gate.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-l1-policy-surface-freeze.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-l2-denial-vocabulary-freeze.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-g2-window-a-integration-gate.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-p2-parent-window-a-integration.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-g3-validation-wall-gate.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-p3-parent-validation-wall.ok`
- `.runs/plan-34-packet-1/sentinels/task-m34p1-p4-parent-closeout-phase.ok`

`authoritative-checkout.json` must record at minimum:

- `root: "/Users/spensermcconnell/__Active_Code/atomize-hq/substrate"`
- `branch: "feat/internal-host-orchestrator-world-dispatch-bootstrap"`
- `head: "de01faaefa7d2f25d159f2ad0f8e804c1e422e89"`
- `untracked_inputs`
- `controller_path`
- `packet_scope: "TASKS-34 Packet 1 only"`

`contract-freeze.json` must record at minimum:

- the authoritative root, branch, and head
- the exact Packet 1-only scope boundary
- the frozen in-scope verbs:
  - `run_world_task`
  - `spawn_world_worker`
  - `continue_world_worker`
- the frozen deny bucket ids listed in Hard Guard 8
- the frozen execution choice that docs stay deferred to Packet 4
- the frozen execution choice that `orchestrator_world_dispatch.rs` changes are helper-only in Packet 1
- the exact initial worker cap of `2`

`merge-order.json` must record at minimum:

- `integration_order: ["task/m34p1-l1-policy-surface-freeze", "task/m34p1-l2-denial-vocabulary-freeze"]`
- `l1_acceptance_basis: "frozen_packet1_contract_only"`
- `l2_acceptance_basis: "accepted_l1_tree_if_policy_names_or_defaults_change_else_post_p1_tree"`
- `replay_required_if_l1_changes_policy_shape_or_bucket_spelling: true`
- `quarantine_on_packet2_behavior_leak: true`

## Workstream Plan

### TASKS-34 Packet 1 Mapping

| Packet 1 workstream | Orchestration tasks | Why this mapping is exact |
| --- | --- | --- |
| Add the narrow world-dispatch steering policy surface to the effective policy model | `task/m34p1-l1-policy-surface-freeze`, `task/m34p1-g2-window-a-integration-gate`, `task/m34p1-p2-parent-window-a-integration` | This lane owns the policy keys, deny-by-default defaults, and effective-model contract. It does not own enforcement behavior. |
| Add stable steering denial vocabulary and contract helpers | `task/m34p1-l2-denial-vocabulary-freeze`, `task/m34p1-g2-window-a-integration-gate`, `task/m34p1-p2-parent-window-a-integration` | This lane owns the stable bucket ids, helper constructors, and helper-only dispatch plumbing for later packets. It does not own state-store truth or Packet 2 gates. |

### Concurrency And Merge Order

Concurrency rules:

1. Worker cap is exactly `2` after `g1` and before `g2`.
2. The only honest Packet 1 parallel window is `L1` plus `L2`.
3. No worker starts before `p1` and `g1` are green.
4. No test-only or docs-only worker lane is authorized in Packet 1.
5. No lane may touch `.runs/**`.
6. The final validation wall runs once on the fully merged Packet 1 tree.

Why `2` is honest here:

1. `L1` and `L2` touch disjoint production files.
2. The parent can freeze exact policy dimensions and exact bucket ids before launch, which removes the main semantic collision risk.
3. There is still not enough independent work for a third lane because all remaining Packet 1 verification is coupled to the same two ownership islands and would create merge noise rather than throughput.

Why the merge order is `L1` then `L2`:

1. `L1` freezes the effective policy shape and default semantics that `L2` helpers must reference truthfully.
2. `L2` is narrower and may need replay only if `L1` changes naming or default decisions after launch.
3. Integrating `L2` first would risk anchoring helpers to stale policy-field naming.

Replay rules:

1. If `L1` changes any Packet 1 policy field name, default value, or bucket spelling from the frozen `contract-freeze.json`, `L2` must replay on the accepted `L1` tree before merge.
2. If `L2` assumes Packet 2 enforcement behavior or touches `state_store.rs`, quarantine `L2` instead of replaying it.
3. The parent does not hand-edit around a lane that leaked Packet 2 behavior. It quarantines or blocks.

### Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m34p1-p1-parent-contract-freeze-and-run-init` | parent | — | authoritative checkout | frozen Packet 1 contract, source-lock snapshots, run-state surface, lane map |
| `task/m34p1-g1-window-a-launch-gate` | parent | `p1` | authoritative checkout | launch approval for `L1` and `L2` only |
| `task/m34p1-l1-policy-surface-freeze` | worker | `g1` | `policy-surface-freeze` / `codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-policy-surface-freeze` | narrow deny-by-default policy surface in broker and shell effective model |
| `task/m34p1-l2-denial-vocabulary-freeze` | worker | `g1` | `denial-vocabulary-freeze` / `codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-denial-vocabulary-freeze` | stable denial buckets and helper-only contract plumbing |
| `task/m34p1-g2-window-a-integration-gate` | parent | `l1`, `l2` | authoritative checkout | acceptance, rejection, replay, or quarantine for Window A |
| `task/m34p1-p2-parent-window-a-integration` | parent | `g2` | authoritative checkout | merged Packet 1 code truth with targeted verification rerun |
| `task/m34p1-g3-validation-wall-gate` | parent | `p2` | authoritative checkout | permission to run the Packet 1-only validation wall |
| `task/m34p1-p3-parent-validation-wall` | parent | `g3` | authoritative checkout | exact Packet 1 validation wall results and Packet 2 handoff freeze |
| `task/m34p1-p4-parent-closeout-phase` | parent | `p3` | authoritative checkout | terminal Packet 1 state, acceptance record, and Packet 2 boundary note |

### Lane Ownership By File Set

| Lane | Allowed files | Forbidden touch surfaces |
| --- | --- | --- |
| `L1` / policy-surface freeze | [crates/broker/src/policy.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/policy.rs), [crates/shell/src/execution/policy_model.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_model.rs), inline unit tests in those files only | `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/tests/**`, `docs/**`, `llm-last-mile/**`, `.runs/**` |
| `L2` / denial-vocabulary freeze | [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs), [crates/shell/src/execution/orchestrator_world_dispatch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs) helper-only, inline unit tests in those files only | `crates/broker/src/policy.rs`, `crates/shell/src/execution/policy_model.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/tests/**`, `docs/**`, `llm-last-mile/**`, `.runs/**` |

## Required Worker Final-Report Contract

Every worker prompt must include:

1. task ID
2. attempt number
3. worktree path
4. branch
5. authoritative base SHA
6. allowed files
7. forbidden files
8. the exact frozen Packet 1 bucket list
9. the exact frozen Packet 1 boundary that Packet 2-4 work is forbidden
10. exact commands to run
11. retry budget
12. required return artifacts
13. sentinel path

Every worker return must include:

1. changed files list
2. commands run with exit codes
3. explicit attempt classification: `clean`, `retryable`, or `blocked`
4. whether the lane stayed inside Packet 1 only
5. whether any Packet 2 behavior seemed necessary
6. unresolved assumptions or blockers
7. `worker-output.patch`
8. `worker-report.md`
9. `evidence-manifest.json`

`worker-report.md` must contain at minimum:

1. `task_id`
2. `branch`
3. `base_head`
4. `result`
5. `files_touched`
6. `frozen_bucket_ids_used`
7. `packet_scope_confirmation`
8. `commands_run`
9. `verification_result`
10. `open_risks_for_parent`

`evidence-manifest.json` must contain at minimum:

1. `task_id`
2. `base_head`
3. `head_after_changes`
4. `files_touched`
5. `tests_run`
6. `artifacts_written`
7. `requires_replay`
8. `packet2_leak_detected`

## Kickoff Initialization Order

The parent initializes the run in this exact order:

1. Create `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-34-packet-1/`, `.runs/plan-34-packet-1/sentinels/`, `.runs/plan-34-packet-1/quarantine/`, `.runs/plan-34-packet-1/source-lock/`, and every `.runs/task-m34p1-*/` directory.
2. Create `task.json`, `commands.txt`, and `summary.md` in every task directory.
3. Create `gate-checklist.md` and `gate-result.json` in every gate task directory.
4. Create placeholder `worker-report.md`, `worker-output.patch`, and `evidence-manifest.json` in every worker task directory.
5. Snapshot the untracked Packet 34 planning docs plus the structure and style references into `.runs/plan-34-packet-1/source-lock/`.
6. Write `authoritative-checkout.json` and `source-lock/checkout-truth.json` with the live root, branch, head, and untracked-input list.
7. Write `tasks.json` as the canonical launch queue and execution ledger.
8. Write `run-state.json` with `current_phase: "kickoff"`, `worker_cap: 2`, the authoritative checkout truth, every task in `pending`, and empty `accepted`, `rejected`, `quarantined`, and `blocked` arrays.
9. Write `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, `validation-wall.md`, and a stub `packet-2-handoff.md`.
10. Freeze the exact Packet 1 denial bucket ids and the rule that Packet 1 docs stay deferred.
11. Freeze the rule that `orchestrator_world_dispatch.rs` is helper-only in Packet 1.
12. Seed both worker worktrees from the exact same post-`p1` tree.
13. Record the authoritative checkout truth, worktree paths, lane ownership, and retry budget in `session-log.md`.

## Parent Phases And Worker Packets

### `task/m34p1-p1-parent-contract-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Freeze the live checkout truth from this workspace, not any older branch metadata.
2. Freeze Packet 1-only scope, denial bucket ids, concurrency cap naming, and helper-only dispatch rule.
3. Freeze the rule that docs remain deferred to Packet 4.
4. Snapshot all untracked Packet 34 planning inputs into `source-lock/`.
5. Seed `L1` and `L2` from the same post-`p1` tree.

Acceptance:

1. `authoritative-checkout.json`, `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, `tasks.json`, and `run-state.json` exist.
2. The freeze artifact explicitly records the authoritative root, branch, head, and untracked planning inputs.
3. The freeze artifact explicitly records that Packet 1 does not edit docs or start Packet 2 enforcement.
4. The parent writes `.runs/plan-34-packet-1/sentinels/task-m34p1-p1-parent-contract-freeze-and-run-init.ok`.

### `task/m34p1-g1-window-a-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. `L1` and `L2` worktrees are seeded from the exact same post-`p1` tree.
3. `L1` prompt explicitly says:
   - freeze the narrow world-dispatch policy surface only,
   - keep defaults deny by default,
   - do not touch dispatch-contract, orchestrator, tests, docs, or `.runs/**`.
4. `L2` prompt explicitly says:
   - freeze stable denial vocabulary and helper scaffolding only,
   - do not implement real pre-routing enforcement,
   - do not touch broker, policy-model, state-store, tests, docs, or `.runs/**`.
5. Both prompts repeat the frozen Packet 1-only boundary and exact bucket ids.

Acceptance:

1. Only `L1` and `L2` are authorized to start.
2. The parent writes `.runs/plan-34-packet-1/sentinels/task-m34p1-g1-window-a-launch-gate.ok`.

### `task/m34p1-l1-policy-surface-freeze`

Owner:

- single worker on `codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-policy-surface-freeze`

Packet fields:

- Owned files:
  - [crates/broker/src/policy.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/policy.rs)
  - [crates/shell/src/execution/policy_model.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_model.rs)
  - inline unit tests in those owned files only
- Forbidden touch surfaces:
  - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - `crates/shell/src/execution/agent_runtime/state_store.rs`
  - `crates/shell/tests/**`
  - `docs/**`
  - `llm-last-mile/**`
  - `.runs/**`
- Exact scope:
  1. Add the narrow world-dispatch steering policy surface to the effective policy model.
  2. Freeze deny-by-default defaults for:
     - steering enabled
     - allowed backends
     - allowed actions
     - allowed modes
     - same-session boundary
     - same-world-binding boundary
     - capability narrowing permission
     - first in-scope session and worker concurrency caps as named Packet 1 fields
  3. Keep the surface current-verb-only for:
     - `run_world_task`
     - `spawn_world_worker`
     - `continue_world_worker`
  4. Do not add docs or public UX wording.
  5. Do not implement routing enforcement.
- Exact required commands:

```bash
cargo test -p shell policy_model -- --nocapture
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. The effective policy model can represent the Packet 1 dimensions without widening into later verbs or family-2 behavior.
  3. Defaults remain deny by default.
  4. The lane does not claim Packet 2 enforcement.
  5. The worker writes `.runs/plan-34-packet-1/sentinels/task-m34p1-l1-policy-surface-freeze.ok`.

### `task/m34p1-l2-denial-vocabulary-freeze`

Owner:

- single worker on `codex/feat-internal-host-orchestrator-world-dispatch-bootstrap-m34p1-denial-vocabulary-freeze`

Packet fields:

- Owned files:
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [crates/shell/src/execution/orchestrator_world_dispatch.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs) helper-only
  - inline unit tests in those owned files only
- Forbidden touch surfaces:
  - `crates/broker/src/policy.rs`
  - `crates/shell/src/execution/policy_model.rs`
  - `crates/shell/src/execution/agent_runtime/state_store.rs`
  - `crates/shell/tests/**`
  - `docs/**`
  - `llm-last-mile/**`
  - `.runs/**`
- Exact scope:
  1. Add stable steering denial vocabulary using the frozen Packet 1 bucket ids.
  2. Add helper constructors or helper mapping surfaces that later packets can reuse.
  3. If `orchestrator_world_dispatch.rs` changes are needed, keep them strictly helper-only and non-enforcing.
  4. Do not invent new bucket ids, later-verb categories, or family-2 denial classes.
  5. Do not implement real pre-routing enforcement.
- Exact required commands:

```bash
cargo test -p shell dispatch_contract -- --nocapture
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. The stable denial vocabulary exactly matches the frozen Packet 1 bucket list.
  3. Any `orchestrator_world_dispatch.rs` change is helper-only and does not gate runtime behavior yet.
  4. The lane does not touch `state_store.rs` or test suites outside inline unit tests.
  5. The worker writes `.runs/plan-34-packet-1/sentinels/task-m34p1-l2-denial-vocabulary-freeze.ok`.

### `task/m34p1-g2-window-a-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned `worker-report.md`, `worker-output.patch`, `commands.txt`, and `evidence-manifest.json`.
2. Every touched file is inside the lane ownership boundary.
3. `L1` stayed on deny-by-default policy shape only and did not widen into docs or Packet 2 enforcement.
4. `L2` stayed on frozen bucket ids and helper-only scaffolding and did not widen into Packet 2 enforcement.
5. If `L1` changed Packet 1 field naming or bucket spelling from the frozen contract, `L2` is marked for replay before merge.
6. If `L2` touched `state_store.rs`, test suites, or docs, quarantine `L2` immediately.

Quarantine and retry behavior:

1. Retry budget is `1` per lane and is available only for lane-local defects inside owned files.
2. Any Packet 2 behavior leak is non-retryable and is quarantined rather than redriven.
3. Any ownership violation is non-retryable and is quarantined rather than redriven.

Acceptance:

1. Accepted, rejected, replay-required, or quarantined status for both lanes is recorded in `run-state.json`.
2. The parent writes `.runs/plan-34-packet-1/sentinels/task-m34p1-g2-window-a-integration-gate.ok`.

### `task/m34p1-p2-parent-window-a-integration`

Owner:

- parent only

Scope:

1. Integrate accepted `L1` first.
2. Re-run the `L1` command gate on the authoritative checkout.
3. Replay `L2` on top of accepted `L1` if the `g2` gate required it.
4. Integrate accepted `L2` second.
5. Re-run the `L2` command gate on the authoritative checkout.
6. Freeze the merged Packet 1 tree before the validation wall.

Minimum commands:

```bash
git cherry-pick <accepted-l1-commit>
cargo test -p shell policy_model -- --nocapture
git cherry-pick <accepted-l2-commit>
cargo test -p shell dispatch_contract -- --nocapture
```

Acceptance:

1. The parent remains the sole integrator.
2. The authoritative tree now contains the narrow Packet 1 policy surface plus stable denial helpers.
3. No docs, state-store, or Packet 2 enforcement drift entered the merged tree.
4. The parent writes `.runs/plan-34-packet-1/sentinels/task-m34p1-p2-parent-window-a-integration.ok`.

### `task/m34p1-g3-validation-wall-gate`

Owner:

- parent only

Checks:

1. `p2` is accepted.
2. No quarantined output remains unresolved.
3. `validation-wall.md` names the exact Packet 1-only command order.
4. `packet-2-handoff.md` is ready to record the frozen Packet 1 surface and bucket ids after the wall passes.

Acceptance:

1. The Packet 1 validation wall is permitted to run exactly once.
2. The parent writes `.runs/plan-34-packet-1/sentinels/task-m34p1-g3-validation-wall-gate.ok`.

### `task/m34p1-p3-parent-validation-wall`

Owner:

- parent only

Scope:

1. Run the exact Packet 1-only validation wall.
2. Record command results.
3. Freeze the Packet 2 handoff note from the validated merged tree only.

## Validation Wall

### Exact Packet 1 Automated Validation Wall

The final Packet 1 automated wall is frozen to:

```bash
cargo fmt --all -- --check
cargo test -p shell policy_model -- --nocapture
cargo test -p shell dispatch_contract -- --nocapture
```

These commands must run on the final merged Packet 1 tree, in that order, unless the parent records a stricter Packet 1-only superset in `validation-wall.md`.

Acceptance:

1. The Packet 1-only automated wall is green on the final merged tree.
2. `packet-2-handoff.md` records the frozen Packet 1 surface and exact bucket ids from the validated tree.
3. The parent writes `.runs/plan-34-packet-1/sentinels/task-m34p1-p3-parent-validation-wall.ok`.

### `task/m34p1-p4-parent-closeout-phase`

Owner:

- parent only

Scope:

1. Audit all sentinels and required artifacts.
2. Confirm `blocked.json` is absent.
3. Record final accepted lanes, quarantines, replay events, command outcomes, and frozen Packet 2 boundaries.
4. Mark Packet 1 complete only if every acceptance check in this controller is satisfied.

Acceptance:

1. Terminal Packet 1 run state is written.
2. Accepted scope matches `TASKS-34` Packet 1 only.
3. Packet 2 is marked `unblocked` only if it can consume the validated Packet 1 surface without reopening Packet 1 naming or scope.
4. The parent writes `.runs/plan-34-packet-1/sentinels/task-m34p1-p4-parent-closeout-phase.ok`.

## Context-Control Rules

1. The parent owns full context. Workers get only the minimum contract excerpt, owned file list, forbidden surface list, exact commands, and artifact return format needed for their lane.
2. The parent never hands `L1` authority over denial helper plumbing and never hands `L2` authority over policy-model parsing. This preserves the real code split.
3. The parent never hands either lane permission to touch docs, tests outside inline units, or `.runs/**`.
4. Workers consume Packet 34 source truth through the parent prompt plus `.runs/plan-34-packet-1/source-lock/`, not from mutable repo docs in their worktrees.
5. If a lane proposes widening scope, the parent does not negotiate in-branch. The parent stops the run, reissues a narrower prompt, or blocks.
6. Parent integration notes must say explicitly whether `L2` remained helper-only in `orchestrator_world_dispatch.rs`.

## Tests And Acceptance

The run is complete only when every item below is true:

- the effective policy model has a narrow implementation-bearing world-dispatch surface for current verbs,
- defaults remain deny by default,
- stable steering denial buckets exist and exactly match the frozen Packet 1 list,
- no docs or trace closeout was pulled forward,
- no state-store or integration-test behavior was pulled into Packet 1,
- any `orchestrator_world_dispatch.rs` change is helper-only and non-enforcing,
- the exact Packet 1 validation wall is green,
- all `.runs/plan-34-packet-1` artifacts and sentinels required by this controller exist.

## Completion Criteria And Packet 2 Handoff Posture

Packet 1 is complete only when:

1. the merged tree validates the Packet 1 policy surface and stable denial vocabulary,
2. the merged tree still does not claim Packet 2 enforcement,
3. `packet-2-handoff.md` records the exact validated bucket ids, policy-field names, and helper entry points that Packet 2 must consume unchanged,
4. `packet-2-handoff.md` explicitly says Packet 2 may touch enforcement surfaces such as `orchestrator_world_dispatch.rs`, `state_store.rs`, and targeted shell integration tests only after starting from the accepted Packet 1 tree,
5. `packet-2-handoff.md` explicitly says Packet 2 must not rename Packet 1 bucket ids or reopen the Packet 1 docs-deferral decision unless it first blocks and replans.

Packet 2 boundary only, not orchestration for Packet 2:

1. Packet 2 may begin only from the accepted post-`p4` tree.
2. Packet 2 owns actual pre-routing enforcement.
3. Packet 2 does not reopen Packet 1 policy parsing or bucket naming unless one of this controller’s blocked-run conditions becomes true.

## Assumptions

1. The current tree already contains the landed three-verb family-1 dispatch surface described by the Packet 34 source docs.
2. The effective policy model can absorb a narrow world-dispatch surface without requiring public CLI or docs changes in Packet 1.
3. Stable denial helpers can be added without forcing `state_store.rs` or integration-test work into Packet 1.
4. The parent can snapshot the untracked Packet 34 planning docs into `.runs/plan-34-packet-1/source-lock/` before workers start.
5. If Packet 1 proves docs must land immediately for correctness, the right move is to block and replan rather than silently widen the packet.
