# ORCH_PLAN-18: Execute PLAN-18 Through A Parent-Frozen Degraded-Status Contract And One Honest Two-Lane Status Split

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-18.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-18.md)  
Style reference: [ORCH_PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-17.md)  
Style reference: [ORCH_PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-16.md)  
Structure reference: [M26 Orchestration Plan](/Users/spensermcconnell/__Active_Code/atomize-hq/spec/docs/m26_orchestration_kickoff_prompt.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Execution type: fresh orchestration plan, Linux-first, status/read-side hardening, state-store plus `agents_cmd` split, tests/docs closeout after integration, parent-frozen contract, parent-only integration and approval  
Worker model: GPT-5.4 workers with `reasoning_effort=high`  
Max concurrent code workers before integration: `2`

## Summary

This document is the execution controller for `PLAN-18`, not a restatement of it.

The run shape is frozen up front:

1. `task/m18-p1-parent-contract-freeze-and-run-init` is parent-only and freezes the degraded-status contract, the naming contract, the participant-aware identity contract, lane ownership, merge order, retry budget, stop conditions, and the validation wall.
2. `task/m18-g1-implementation-lane-launch-gate` is parent-only and is the only gate that may launch the implementation workers.
3. The only honest parallel implementation window is exactly two lanes after `p1`:
   - `task/m18-l1-status-store-enumeration-and-naming-cleanup`
   - `task/m18-l2-status-rendering-fallback-and-nested-correlation`
4. `task/m18-g2-code-lane-integration-gate` and `task/m18-p2-parent-code-lane-integration` are parent-only.
5. Tests/docs closeout is not parallel with code lanes. It starts only after `p2` lands merged code truth:
   - `task/m18-g3-closeout-launch-gate`
   - `task/m18-l3-tests-docs-gap-closeout`
6. `task/m18-g4-validation-wall-gate` and `task/m18-p3-parent-validation-wall-and-closeout` are parent-only and finish the run.

Canonical task IDs:

- `task/m18-p1-parent-contract-freeze-and-run-init`
- `task/m18-g1-implementation-lane-launch-gate`
- `task/m18-l1-status-store-enumeration-and-naming-cleanup`
- `task/m18-l2-status-rendering-fallback-and-nested-correlation`
- `task/m18-g2-code-lane-integration-gate`
- `task/m18-p2-parent-code-lane-integration`
- `task/m18-g3-closeout-launch-gate`
- `task/m18-l3-tests-docs-gap-closeout`
- `task/m18-g4-validation-wall-gate`
- `task/m18-p3-parent-validation-wall-and-closeout`

## Assumptions

1. [PLAN-18.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-18.md) remains the authoritative dependency graph for this run.
2. The authoritative integration checkout remains the current workspace checkout on `feat/session-centric-state-store`.
3. The authoritative production hotspots are [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) and [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs).
4. The tests/docs hotspot is [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs) plus [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) and [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md).
5. Linux-first status and naming cohesion is the only platform goal for this run. No macOS/Lima parity work is honest here.
6. Docs move last, after integrated code and green regressions exist.
7. Lane B is allowed to depend on a parent-frozen seam contract, but it is not allowed to invent a duplicate status enumeration path in `agents_cmd.rs`.

## Immutable Run Shape

### Frozen contract truth

These are run-stopping invariants, not preferences:

1. `substrate agent status` becomes permissive in rendering only.
2. `substrate agent doctor`, `substrate agent toolbox status`, `substrate agent toolbox env`, and any future public `start|resume|fork|stop` surface remain strict and fail closed.
3. `StatusSessionJson.source_kind` is mandatory and exact. Allowed values are `live_runtime` and `trace_fallback`.
4. `StatusReportJson.warnings` is mandatory as `Vec<String>`.
5. Warning strings are sorted, deduplicated, and human-readable before render.
6. Record-local degradation warnings come from `AgentRuntimeSessionRecord.warnings`.
7. Cross-record ambiguity warnings are emitted by `build_status_report(...)`.
8. One and only one identity family is allowed for status selection and suppression:

```text
StatusIdentityKey {
    orchestration_session_id: String,
    agent_id: String,
    execution_scope: "host" | "world",
    participant_id: Option<String>,
}
```

9. The same `StatusIdentityKey` contract is used for pure-agent trace selection, live/tombstone fallback suppression, and nested selected-parent bucketing.
10. Legacy coarse grouping is allowed only for rows that truly lack `participant_id`.
11. The canonical state-store seam is exactly `list_status_sessions_for_agent(&self, orchestrator_agent_id: &str) -> Result<Vec<AgentRuntimeSessionRecord>>`.
12. Live authoritative rows and invalidated tombstones continue to win over trace fallback.
13. Public/operator-facing naming stays frozen:
    - public selector: `orchestration_session_id`
    - runtime lineage id: `participant_id`
    - internal-only backend-native handle: `internal.uaa_session_id`
14. Legacy compatibility reads for `session_handle_id`, `parent_session_handle_id`, and `resumed_from_session_handle_id` remain supported.
15. This run must not widen into:
    - public `start|resume|fork|stop` surfaces
    - `substrate -c` redesign
    - world-member follow-up or replacement semantics
    - weakening doctor/toolbox fail-closed behavior
    - trace producer schema expansion without concrete proof
    - storage flag-day rename away from `active_session_handle_id`
    - macOS/Lima parity
16. The parent agent is the only integrator and the only approval authority.

### Parent-only versus worker-owned authority

Parent-only for the full run:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-18/*`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m18-*/**`
- worker launch, output classification, merge approval, quarantine, retry approval, final validation, and blocked-run termination
- any decision to stop rather than invent cross-lane semantics

Parent-only during `p1`, then frozen by artifact:

- degraded-status contract
- naming contract
- `StatusIdentityKey` contract
- the exact `list_status_sessions_for_agent(...)` seam
- worker cap and lane ownership
- merge order
- validation-wall command list
- stop conditions
- whether a compile-only seam scaffold is needed before worker seeding

Worker-owned after `p1`:

- `L1` / Lane A: state-store enumeration seam, Linux-first naming/comment cleanup, and any inline unit-test changes inside its owned runtime files only if required by that work
- `L2` / Lane B: `agents_cmd.rs` status rendering, warnings wiring, participant-aware fallback suppression, and nested parent correlation only
- `L3` / Lane C: integration regressions in `agent_successor_contract_ahcsitc0.rs` and repo-truth docs only

No worker may edit parent-owned run-state. No worker may edit another lane’s owned files. No worker may reopen the frozen contract.

### Stop conditions

Stop the run, write `.runs/plan-18/blocked.json`, and do not advance if any of these occur:

1. The degraded-status contract cannot be frozen without changing public control-surface shape.
2. Lane B can only work by duplicating status enumeration logic locally instead of consuming the frozen `list_status_sessions_for_agent(...)` seam.
3. Any implementation weakens `resolve_single_live_session_for_agent(...)` for doctor, toolbox, or future mutating control paths.
4. Any implementation broadens into `start|resume|fork|stop`, `substrate -c`, world-member follow-up semantics, storage rename, or macOS/Lima work.
5. Any implementation requires trace producer schema expansion without a concrete missing-field regression proving the need.
6. Any worker edits another lane’s owned files or any parent-owned run-state.
7. `L3` starts before `p2` freezes merged code truth.
8. The parent would need to invent hybrid warning, suppression, or naming semantics during integration.
9. The validation wall cannot prove the permissive-vs-strict split, `source_kind`, `warnings`, sorted/dedup warnings, participant-aware suppression, and strict doctor/toolbox non-regression.
10. The gap matrix or README would need to overclaim beyond what the integrated tests and final validation wall actually prove.

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18/status-store-enumeration-and-naming-cleanup`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18/status-rendering-fallback-and-nested-correlation`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18/tests-docs-gap-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m18-status-store-enumeration-and-naming-cleanup`
- `codex/feat-session-centric-state-store-m18-status-rendering-fallback-and-nested-correlation`
- `codex/feat-session-centric-state-store-m18-tests-docs-gap-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18/status-store-enumeration-and-naming-cleanup -b codex/feat-session-centric-state-store-m18-status-store-enumeration-and-naming-cleanup feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18/status-rendering-fallback-and-nested-correlation -b codex/feat-session-centric-state-store-m18-status-rendering-fallback-and-nested-correlation feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-18/tests-docs-gap-closeout -b codex/feat-session-centric-state-store-m18-tests-docs-gap-closeout feat/session-centric-state-store
```

### Parent integration surface

The parent integrates on the authoritative checkout already on `feat/session-centric-state-store`.

No separate parent integration worktree is introduced because:

1. this run has one serialized integrator
2. `.runs/plan-18/*` is parent-owned state and should stay co-located with the authoritative branch context
3. the only true merge activity is parent-owned and serialized

## Phase Graph And Concurrency

Concurrency rules:

1. Worker cap is `2` until `g2` completes.
2. `p1` must finish before any worker starts.
3. `g1` must accept before any worker starts.
4. The only real parallel window is:
   - `task/m18-l1-status-store-enumeration-and-naming-cleanup`
   - `task/m18-l2-status-rendering-fallback-and-nested-correlation`
5. `g2` classifies both code-lane outputs before integration.
6. `p2` integrates accepted code lanes in this order:
   - Lane A / `L1` first
   - Lane B / `L2` second
7. `g3` starts only after `p2` is green.
8. `L3` runs alone on the exact post-`p2` tree.
9. `g4` and `p3` are parent-only.
10. No third concurrent implementation worker is honest because tests/docs must describe the integrated code truth, not lane-local guesses.

### Why Lane A integrates before Lane B

1. Lane A owns the canonical seam `list_status_sessions_for_agent(...)` that Lane B must consume.
2. Lane A also owns the Linux-first naming/comment cleanup in the runtime files that define the public-vs-internal naming split B must preserve.
3. Integrating Lane B first would force either a duplicate enumeration path inside `agents_cmd.rs` or parent-invented temporary semantics. Both violate the frozen contract.
4. If the combined state reveals semantic drift, Lane B is the correct quarantine target because it is the downstream consumer of the frozen seam, warning split, and identity-key contract.

## PLAN-18 Step Mapping

| Orchestration task | PLAN-18 step alignment |
| --- | --- |
| `task/m18-p1-parent-contract-freeze-and-run-init` | Step 1: freeze the degraded-status, naming, warning, and identity contracts |
| `task/m18-l1-status-store-enumeration-and-naming-cleanup` | Step 2 plus the runtime-file naming/comment portion of Step 1 |
| `task/m18-l2-status-rendering-fallback-and-nested-correlation` | Step 3, Step 4, and Step 5 |
| `task/m18-p2-parent-code-lane-integration` | enforce the end-to-end contract across merged Lane A plus Lane B |
| `task/m18-l3-tests-docs-gap-closeout` | Step 6 and Step 7 |
| `task/m18-p3-parent-validation-wall-and-closeout` | recommended verification commands plus Definition of Done enforcement |

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-18/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `contract-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `blocked.json` on failure only
- `quarantine/`
- `sentinels/`

`contract-freeze.json` is the single source of truth for:

- `plan_id: "PLAN-18"`
- `plan_source: "llm-last-mile/PLAN-18.md"`
- `orchestration_plan_source: "llm-last-mile/ORCH_PLAN-18.md"`
- `branch: "feat/session-centric-state-store"`
- `status_surface_mode: "permissive_rendering_only"`
- `strict_surfaces: ["agent doctor", "agent toolbox status", "agent toolbox env", "future agent start|resume|fork|stop"]`
- `status_session_source_kind_field: "StatusSessionJson.source_kind"`
- `status_report_warnings_field: "StatusReportJson.warnings"`
- `warning_ordering: "sorted_deduplicated_human_readable"`
- `record_warning_owner: "AgentRuntimeSessionRecord.warnings"`
- `set_warning_owner: "build_status_report"`
- `status_identity_key: "orchestration_session_id + agent_id + execution_scope + participant_id?"`
- `status_list_seam: "AgentRuntimeStateStore::list_status_sessions_for_agent"`
- `trace_schema_expansion_allowed: false`
- `public_control_surfaces_allowed: false`
- `substrate_c_redesign_allowed: false`
- `world_follow_up_semantics_reopened: false`
- `doctor_toolbox_relaxation_allowed: false`
- `storage_flag_day_rename_allowed: false`
- `macos_parity_allowed: false`
- `validation_commands`
- `manual_spot_checks`
- `stop_conditions`

`lane-ownership.json` is the single source of truth for:

- task ID
- owner
- worktree path
- branch
- allowed files
- forbidden files
- command gates
- retry budget
- merge order
- sentinel filename

`merge-order.json` is a frozen artifact, not a narrative note. It records:

- `plan_id`
- `integration_order: ["task/m18-l1-status-store-enumeration-and-naming-cleanup", "task/m18-l2-status-rendering-fallback-and-nested-correlation", "task/m18-l3-tests-docs-gap-closeout"]`
- `lane_b_acceptance_basis: "accepted_lane_a_tree_only"`
- `lane_b_must_consume_seam: "AgentRuntimeStateStore::list_status_sessions_for_agent"`
- `replay_required_before_acceptance: true`
- `quarantine_on_branch_local_assumption: true`

`run-state.json` is the single source of truth for:

- `current_phase`
- `active_task_ids`
- `worker_cap`
- `contract_freeze_status`
- `lane_status`
- `accepted_outputs`
- `rejected_outputs`
- `quarantined_outputs`
- `blocked_outputs`
- `retry_budget_by_lane`
- `attempt_counts`
- `integration_order`
- `gate_status`
- `validation_wall_status`
- `termination_reason`
- `terminal_state`

### `tasks.json` required row fields

`tasks.json` is both the launch queue and the execution ledger for the entire run.

Each task row must include at least:

- `task_id`
- `title`
- `owner`
- `status`
- `phase_class`
- `depends_on`
- `worktree_path`
- `branch`
- `allowed_files`
- `forbidden_files`
- `command_gates`
- `acceptance_checks`
- `sentinel_path`
- `artifact_dir`

Recommended status vocabulary:

- `pending`
- `ready`
- `running`
- `accepted`
- `rejected`
- `quarantined`
- `blocked`
- `completed`

Recommended phase classes:

- `parent_freeze`
- `gate`
- `worker_lane`
- `parent_integration`
- `validation`
- `closeout`

Required sentinels:

- `.runs/plan-18/sentinels/task-m18-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-18/sentinels/task-m18-g1-implementation-lane-launch-gate.ok`
- `.runs/plan-18/sentinels/task-m18-l1-status-store-enumeration-and-naming-cleanup.ok`
- `.runs/plan-18/sentinels/task-m18-l2-status-rendering-fallback-and-nested-correlation.ok`
- `.runs/plan-18/sentinels/task-m18-g2-code-lane-integration-gate.ok`
- `.runs/plan-18/sentinels/task-m18-p2-parent-code-lane-integration.ok`
- `.runs/plan-18/sentinels/task-m18-g3-closeout-launch-gate.ok`
- `.runs/plan-18/sentinels/task-m18-l3-tests-docs-gap-closeout.ok`
- `.runs/plan-18/sentinels/task-m18-g4-validation-wall-gate.ok`
- `.runs/plan-18/sentinels/task-m18-p3-parent-validation-wall-and-closeout.ok`

### Per-task artifact directories

Required task directories:

- `.runs/task-m18-p1-parent-contract-freeze-and-run-init/`
- `.runs/task-m18-g1-implementation-lane-launch-gate/`
- `.runs/task-m18-l1-status-store-enumeration-and-naming-cleanup/`
- `.runs/task-m18-l2-status-rendering-fallback-and-nested-correlation/`
- `.runs/task-m18-g2-code-lane-integration-gate/`
- `.runs/task-m18-p2-parent-code-lane-integration/`
- `.runs/task-m18-g3-closeout-launch-gate/`
- `.runs/task-m18-l3-tests-docs-gap-closeout/`
- `.runs/task-m18-g4-validation-wall-gate/`
- `.runs/task-m18-p3-parent-validation-wall-and-closeout/`

Each task directory must contain at least:

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

Each task directory may also contain:

- `artifacts/`
- `rejected.json` on rejection only
- `blocked.json` on blocked task return only
- `quarantine/` when the parent quarantines that task’s output

### Gate artifact semantics

Every gate task must produce `gate-result.json` with at least:

- `task_id`
- `checked_prerequisites`
- `artifacts_inspected`
- `classification`
- `timestamp`
- `approver: "parent"`

`classification` must be one of:

- `pass`
- `fail`

A gate is green only when:

1. all prerequisite sentinels exist
2. all required artifacts for the previous phase exist
3. ownership compliance is verified
4. pass/fail is recorded explicitly in `gate-result.json`

## Kickoff Initialization Order

The parent initializes the run in this exact order before any worker prompt is written:

1. Create `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-18/`, `.runs/plan-18/sentinels/`, `.runs/plan-18/quarantine/`, and every `.runs/task-m18-*/` directory.
2. Inside each `.runs/task-m18-*/` directory, create:
   - `task.json`
   - `commands.txt`
   - `summary.md`
   - `artifacts/`
3. For each gate task, also create:
   - `gate-checklist.md`
   - `gate-result.json`
4. For each worker task, also create placeholders for:
   - `worker-report.md`
   - `worker-output.patch`
   - `evidence-manifest.json`
5. Write `tasks.json` as the canonical launch queue and execution ledger for the whole run.
6. Write `run-state.json` with `current_phase: "kickoff"`, `worker_cap: 2`, every task in `pending`, and empty accepted/rejected/quarantined output arrays.
7. Write `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, and `validation-wall.md`.
8. Review the frozen hotspots:
   - [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   - [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
   - [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   - [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
9. If the exact seam `list_status_sessions_for_agent(...)` does not already exist, the parent may take one narrow shared-seam scaffold in `state_store.rs` before worker launch so Lane B can compile against the frozen signature. That scaffold must be signature-first, must not weaken any strict selector behavior, and must be recorded in `contract-freeze.json`. If the parent uses this scaffold, all worker branches must be reseeded from that exact post-scaffold commit.
10. Seed worker worktrees only after the above artifacts and any allowed scaffold are in place.
11. Write `session-log.md` with kickoff timestamp, authoritative branch, worktree roots, worker cap, and the explicit statement that the only honest parallel window is Lane A plus Lane B.
12. Mark `task/m18-p1-parent-contract-freeze-and-run-init` complete only after the frozen contract and seeded worker basis are identical.

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m18-p1-parent-contract-freeze-and-run-init` | parent | — | authoritative checkout | frozen run artifacts and seeded worker basis |
| `task/m18-g1-implementation-lane-launch-gate` | parent | `p1` | authoritative checkout | launch approval for Lane A and Lane B |
| `task/m18-l1-status-store-enumeration-and-naming-cleanup` | worker | `g1` | `status-store-enumeration-and-naming-cleanup` / `codex/feat-session-centric-state-store-m18-status-store-enumeration-and-naming-cleanup` | canonical status-store seam and naming/comment cleanup |
| `task/m18-l2-status-rendering-fallback-and-nested-correlation` | worker | `g1` | `status-rendering-fallback-and-nested-correlation` / `codex/feat-session-centric-state-store-m18-status-rendering-fallback-and-nested-correlation` | status rendering refactor, warnings, participant-aware fallback, nested correlation |
| `task/m18-g2-code-lane-integration-gate` | parent | `l1`, `l2` | authoritative checkout | acceptance/quarantine decision for both code lanes |
| `task/m18-p2-parent-code-lane-integration` | parent | `g2` | authoritative checkout | merged Lane A then Lane B code truth |
| `task/m18-g3-closeout-launch-gate` | parent | `p2` | authoritative checkout | launch approval for tests/docs closeout |
| `task/m18-l3-tests-docs-gap-closeout` | worker | `g3` | `tests-docs-gap-closeout` / `codex/feat-session-centric-state-store-m18-tests-docs-gap-closeout` | regressions and repo-truth docs |
| `task/m18-g4-validation-wall-gate` | parent | `l3` | authoritative checkout | permission to run final command wall |
| `task/m18-p3-parent-validation-wall-and-closeout` | parent | `g4` | authoritative checkout | final validation, closeout, terminal state |

## Gate Tasks And Parent-Only Tasks

Gate tasks are parent-only classification or launch-control phases. They do not mutate production files:

- `task/m18-g1-implementation-lane-launch-gate`
- `task/m18-g2-code-lane-integration-gate`
- `task/m18-g3-closeout-launch-gate`
- `task/m18-g4-validation-wall-gate`

Parent-only tasks are the only phases allowed to freeze contracts, integrate, approve, quarantine, or terminate the run:

- `task/m18-p1-parent-contract-freeze-and-run-init`
- `task/m18-p2-parent-code-lane-integration`
- `task/m18-p3-parent-validation-wall-and-closeout`

## Lane Ownership By File Set

| Lane | Allowed files | Forbidden escalation surfaces |
| --- | --- | --- |
| Lane A / `L1` | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs), [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs), public control surfaces, docs, `agent_successor_contract_ahcsitc0.rs`, `.runs/**` |
| Lane B / `L2` | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | every runtime storage file, every docs file, [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs), toolbox/doctor contract broadening, `.runs/**` |
| Lane C / `L3` | [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if needed | every production Rust file, `.runs/**`, any public control-surface doc overclaim |

Rule for inline unit tests:

- If Lane A needs unit-test adjustments inside `state_store.rs` or `session.rs` to keep the naming or seam contract local, those edits remain Lane A-owned and must stay minimal.
- Lane C does not reopen runtime production files just to move proofs around.

## Merge-Order Artifact

`merge-order.json` is frozen during `p1` and governs integration behavior.

It must state:

- Lane A integrates first
- Lane B is evaluated only against the accepted Lane A seam
- Lane B is not accepted against its own branch-local assumption set
- Lane C starts only after final parent-integrated code truth exists

Operational rule:

- If Lane B’s patch merges mechanically but assumes a different seam signature, different warning ownership split, or different identity-key contract than the accepted Lane A tree, the parent quarantines Lane B rather than editing around the mismatch.

## Worker Interfaces

### Worker prompt contract

Every worker prompt sent by the parent must include:

1. task ID
2. attempt number
3. worktree path
4. branch name
5. owned files
6. forbidden files
7. exact frozen contract clauses relevant to that lane
8. command gates
9. retry budget
10. required return artifacts
11. sentinel name

### Worker return contract

Every worker must return exactly:

1. changed files list
2. commands run, each with exit code
3. explicit attempt classification: `clean`, `retryable`, or `blocked`
4. unresolved assumptions or blockers
5. `worker-output.patch`
6. `worker-report.md`
7. `evidence-manifest.json`

The parent may accept a worker return only if it can verify:

1. the patch stays within owned files
2. command gates actually ran
3. exit codes are present
4. the attempt classification matches the evidence
5. no frozen contract clause was reopened

## Parent Phase Details

### `task/m18-p1-parent-contract-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Initialize `.runs/plan-18/*` and `.runs/task-m18-*/**`.
2. Freeze the permissive-vs-strict surface split, warning ownership split, exact `source_kind` and `warnings` fields, `StatusIdentityKey`, and the exact `list_status_sessions_for_agent(...)` seam into `contract-freeze.json`.
3. Freeze worker file ownership, merge order, retry budget, and stop conditions.
4. Freeze the exact validation-wall commands and manual spot checks.
5. If needed, take the one narrow shared-seam scaffold in `state_store.rs` before worker seeding and record the scaffold commit.
6. Seed all worker worktrees from the exact same post-`p1` tree.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

Acceptance:

1. `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, `run-state.json`, and `tasks.json` exist.
2. The frozen contract explicitly records that this is a status/read-side hardening run, not a public control-surface run.
3. Any allowed shared-seam scaffold is recorded and reseeded before worker launch.
4. The parent writes `.runs/plan-18/sentinels/task-m18-p1-parent-contract-freeze-and-run-init.ok`.

### `task/m18-g1-implementation-lane-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. Both implementation worktrees were seeded from the exact same post-`p1` tree.
3. Lane A and Lane B prompts name only owned files, forbidden surfaces, command gates, sentinel, and retry budget.
4. Lane B’s prompt explicitly forbids inventing a duplicate enumeration path inside `agents_cmd.rs`.
5. The prompt explicitly repeats the frozen prohibitions on public control-surface work, `substrate -c`, world-member semantics, trace schema expansion, storage rename, and macOS parity.

Acceptance:

1. No worker starts before this gate is green.
2. The parent writes `.runs/plan-18/sentinels/task-m18-g1-implementation-lane-launch-gate.ok`.

### `task/m18-l1-status-store-enumeration-and-naming-cleanup`

Owner:

- single worker on `codex/feat-session-centric-state-store-m18-status-store-enumeration-and-naming-cleanup`

Owned files:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)

Scope:

1. Land `list_status_sessions_for_agent(...)` as the canonical permissive status enumeration seam.
2. Keep `resolve_single_live_session_for_agent(...)` strict and unchanged in meaning.
3. Route the new seam through `build_session_record(...)`; do not create a second record-construction path.
4. Preserve `warnings`, `complete`, and `has_authoritative_parent`.
5. Add only the naming/comment cleanup needed to make `orchestration_session_id`, `participant_id`, and `internal.uaa_session_id` responsibilities explicit.
6. Preserve compatibility alias reads.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

Acceptance:

1. The lane touches only its owned files.
2. The new seam exists with the exact frozen signature.
3. The seam enumerates incomplete-but-readable status records without authorizing control.
4. Strict selector behavior for doctor/toolbox remains unchanged.
5. Naming/comment cleanup does not perform a storage rename.
6. No trace schema, world-member, `substrate -c`, or public control-surface drift is introduced.
7. The worker writes `.runs/plan-18/sentinels/task-m18-l1-status-store-enumeration-and-naming-cleanup.ok`.

### `task/m18-l2-status-rendering-fallback-and-nested-correlation`

Owner:

- single worker on `codex/feat-session-centric-state-store-m18-status-rendering-fallback-and-nested-correlation`

Owned files:

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Scope:

1. Remove the unconditional strict preflight from `build_status_report(...)`.
2. Consume the frozen `list_status_sessions_for_agent(...)` seam rather than re-filtering raw session storage locally.
3. Add `StatusSessionJson.source_kind`.
4. Add `StatusReportJson.warnings`.
5. Populate warnings as the sorted, deduplicated union of record-local warnings and cross-record ambiguity warnings.
6. Replace coarse fallback keys with `StatusIdentityKey`.
7. Preserve coarse grouping only where `participant_id` is truly absent.
8. Make nested parent correlation participant-aware without weakening malformed-tuple fail-closed behavior.

Command gates:

```bash
cargo fmt --all -- --check
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

Acceptance:

1. The lane touches only its owned file.
2. `source_kind` and `warnings` land exactly as frozen.
3. Lane B does not duplicate status enumeration logic.
4. Same-agent sibling participants stay distinct when lineage exists.
5. Live/tombstone suppression remains more specific, not looser.
6. Doctor/toolbox behavior is not relaxed as collateral damage.
7. The worker writes `.runs/plan-18/sentinels/task-m18-l2-status-rendering-fallback-and-nested-correlation.ok`.

### `task/m18-g2-code-lane-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned a patch, report, command transcript, and evidence manifest.
2. Every touched file is inside the lane’s ownership boundary.
3. Lane A landed the exact seam and did not weaken strict selector semantics.
4. Lane B landed the exact status fields and key contract and did not introduce a duplicate enumeration path.
5. Lane B is not marked `accepted` until the parent proves it replays cleanly against the accepted Lane A seam on a parent-owned A-applied tree.

Acceptance:

1. Accepted, rejected, or quarantined status for each code lane is recorded explicitly in `run-state.json`.
2. The parent writes `.runs/plan-18/sentinels/task-m18-g2-code-lane-integration-gate.ok` only after both code lanes are classified.

### `task/m18-p2-parent-code-lane-integration`

Owner:

- parent only

Scope:

1. Integrate accepted Lane A output first.
2. Re-run Lane A command gates on the authoritative checkout.
3. Replay Lane B output onto the accepted-A tree. If Lane B assumed a different seam signature, invented local filtering, or otherwise drifted from the frozen contract, quarantine Lane B instead of hand-editing around it.
4. Integrate accepted Lane B output second.
5. Re-run combined code-lane gates on the authoritative checkout.
6. Freeze the merged code truth before any tests/docs lane starts.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

Acceptance:

1. The parent remains the sole integrator.
2. The authoritative tree now contains the canonical seam, the status-report refactor, and the participant-aware key contract.
3. No hybrid truth was invented during integration.
4. The parent writes `.runs/plan-18/sentinels/task-m18-p2-parent-code-lane-integration.ok`.

### `task/m18-g3-closeout-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is green.
2. The closeout worktree is reseeded or rebased to the exact post-`p2` tree.
3. The worker prompt names only the allowed test and doc files.
4. The worker prompt explicitly forbids reopening production runtime files.

Acceptance:

1. No tests/docs worker starts before this gate is green.
2. The parent writes `.runs/plan-18/sentinels/task-m18-g3-closeout-launch-gate.ok`.

### `task/m18-l3-tests-docs-gap-closeout`

Owner:

- single worker on `codex/feat-session-centric-state-store-m18-tests-docs-gap-closeout`

Owned files:

- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if needed

Scope:

1. Add degraded-status regressions for missing `active_session_handle_id`.
2. Add degraded-status regressions for multiple active parent candidates.
3. Add trace-only sibling-participant visibility regressions.
4. Add sibling-specific suppression regressions.
5. Add parent-participant-aware nested correlation regressions.
6. Preserve strict doctor/toolbox fail-closed regressions.
7. Update repo-truth docs only after the integrated code and regressions agree on the same shipped truth.

Command gates:

```bash
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
rg -n "Status ambiguity handling|Trace-only participant-aware fallback|orchestration_session_id|participant_id" AGENT_ORCHESTRATION_GAP_MATRIX.md llm-last-mile/README.md
```

Acceptance:

1. The lane touches only its owned files.
2. Tests prove degraded status succeeds with warnings while doctor/toolbox remain strict.
3. Docs say `orchestration_session_id` and `participant_id` plainly and do not imply public control-surface completion.
4. The gap matrix does not mark anything closed beyond what the integrated tests and final validation wall prove.
5. The worker writes `.runs/plan-18/sentinels/task-m18-l3-tests-docs-gap-closeout.ok`.

## Quarantine, Retry, And Blocked-Run Posture

1. Each worker lane has retry budget `1`.
2. Retry is allowed only for lane-local defects inside owned files.
3. Non-retryable violations include:
   - strict-surface relaxation
   - duplicate enumeration logic in `agents_cmd.rs`
   - trace producer schema expansion without proof
   - public control-surface broadening
   - `substrate -c` redesign
   - world-member semantic drift
   - storage rename
   - macOS/Lima broadening
4. If Lane A changes the seam signature or weakens `resolve_single_live_session_for_agent(...)`, quarantine it immediately.
5. If Lane B can only pass by broadening ownership into runtime storage files or by recreating status enumeration logic locally, quarantine it immediately.
6. If Lane C can only make docs pass by overclaiming landed behavior, reject Lane C and stop the run instead of publishing optimistic docs.
7. The parent never hand-merges a hybrid contract out of conflicting worker guesses. When semantics drift, quarantine the output and either redrive within the same lane or stop.

### Quarantine artifact handling

When a lane is quarantined, the parent must preserve the returned materials in both places:

1. keep the original files in the lane task directory:
   - `.runs/task-<task-id>/worker-output.patch`
   - `.runs/task-<task-id>/worker-report.md`
   - `.runs/task-<task-id>/evidence-manifest.json`
2. copy them into the plan-level quarantine surface:
   - `.runs/plan-18/quarantine/<task-id>/worker-output.patch`
   - `.runs/plan-18/quarantine/<task-id>/worker-report.md`
   - `.runs/plan-18/quarantine/<task-id>/evidence-manifest.json`
   - `.runs/plan-18/quarantine/<task-id>/commands.txt`
   - `.runs/plan-18/quarantine/<task-id>/summary.md`
   - `.runs/plan-18/quarantine/<task-id>/quarantine-reason.json`

`quarantine-reason.json` must record:

- `task_id`
- `attempt`
- `classification`
- `frozen_contract_clause_violated`
- `ownership_violation` if any
- `merge_replay_failure` if any
- `retry_available`
- `timestamp`
- `approver: "parent"`

Blocked termination minimum contents:

1. task or gate where execution stopped
2. classification: `rejected`, `blocked`, `quarantined`, or `merge_refused`
3. exact contract clause or ownership rule that stopped the run
4. whether retry remained available
5. artifact paths for patch, report, command output, and semantic-drift evidence
6. explicit statement that no blocked output was integrated

## Context-Control Rules

1. The parent keeps only a bounded live context:
   - [PLAN-18.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-18.md)
   - this orchestration plan
   - `tasks.json`
   - `run-state.json`
   - `contract-freeze.json`
   - `merge-order.json`
   - latest integration diff summary
2. Worker prompts contain only:
   - owned file set
   - exact frozen contract excerpts relevant to that lane
   - required commands
   - forbidden touch surfaces
   - the recorded merge-order rule when relevant
3. Workers return summaries and artifacts only. They do not become independent approval or truth surfaces.
4. Workers do not write `.runs/plan-18/*`.
5. The parent reviews summaries plus narrow diffs only. It does not ingest full worker transcripts into the main run context.
6. Each worker is closed after accept/reject/quarantine to keep context bounded.

## Tests And Acceptance

### A. Frozen-contract acceptance

- The permissive-vs-strict split is frozen before worker launch.
- `source_kind`, `warnings`, warning ordering, warning ownership split, `StatusIdentityKey`, and the exact seam signature are all recorded in `contract-freeze.json`.
- No worker prompt reopens those decisions.

### B. Lane A acceptance

- `list_status_sessions_for_agent(...)` exists with the exact frozen signature.
- The seam routes through `build_session_record(...)`.
- The seam preserves warnings and completeness truth without authorizing control.
- Strict selector behavior remains fail closed for doctor/toolbox.

### C. Lane B acceptance

- `StatusSessionJson.source_kind` is present and exact.
- `StatusReportJson.warnings` is present and exact.
- Warning output is sorted and deduplicated.
- Trace-only fallback, live/tombstone suppression, and nested parent correlation all use the same `StatusIdentityKey` family.
- Lane B does not invent a second status-record construction path.

### D. Integration acceptance

- Lane A integrates first and remains green.
- Lane B is replayed on top of the accepted Lane A seam before acceptance.
- The merged tree preserves strict doctor/toolbox behavior while making `agent status` permissive.
- No parent-only hybrid edits are required to explain the merged behavior.

### E. Lane C acceptance

- `agent_successor_contract_ahcsitc0.rs` proves degraded status with warnings, sibling visibility, sibling-specific suppression, and nested participant-aware correlation.
- Existing strict doctor/toolbox fail-closed regressions remain green.
- Docs reflect the landed Linux-first status and naming truth and leave public control-surface work deferred.

### F. Run-control acceptance

- Every completed task has its sentinel.
- Every gate has a `gate-result.json`.
- `tasks.json` and `run-state.json` match the actual accepted/rejected/quarantined outcomes.
- No quarantined or blocked output was partially integrated.

## Validation Wall

Parent-owned validation commands, executed only after `L3` is integrated:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

Manual spot checks after the command wall is green:

```bash
substrate agent status --json
substrate agent doctor --json
substrate agent toolbox status --json
substrate agent toolbox env --json
```

Validation-wall requirements:

1. formatting and clippy stay green
2. shell library tests stay green
3. the targeted successor/status integration suite proves degraded status success with warnings and strict doctor/toolbox non-regression
4. manual spot checks confirm the same operator contract on real CLI output:
   - status reports `source_kind`
   - status reports `warnings`
   - doctor/toolbox remain strict
5. docs do not overclaim beyond what the green runtime and tests prove

### `task/m18-g4-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L3` returned and is classified before final validation.
2. `L3` is `accepted`.
3. No quarantined or blocked output remains unresolved.
4. `validation-wall.md` names the exact final command order and manual spot checks.
5. The parent can enumerate every `PLAN-18` definition-of-done clause and the command or artifact that proves it.

Acceptance:

1. The parent writes `.runs/plan-18/sentinels/task-m18-g4-validation-wall-gate.ok`.
2. The validation wall is permitted to run exactly once on the final merged tree.

### `task/m18-p3-parent-validation-wall-and-closeout`

Owner:

- parent only

Scope:

1. Integrate only accepted `L3` output.
2. Run the full validation wall in exact order.
3. Record final command results and artifact paths in `.runs/task-m18-p3-parent-validation-wall-and-closeout/artifacts/`.
4. Confirm the gap matrix and optional README index match the validated runtime truth.
5. Mark the run complete only if the validation wall proves the frozen contract, not merely compilation.

Required final artifacts under `.runs/task-m18-p3-parent-validation-wall-and-closeout/artifacts/`:

- `fmt.txt`
- `clippy.txt`
- `shell-lib-tests.txt`
- `agent-successor-contract.txt`
- `status-json-spot-check.txt`
- `doctor-json-spot-check.txt`
- `toolbox-status-json-spot-check.txt`
- `toolbox-env-json-spot-check.txt`
- `contract-audit.md`
- `closeout.md`

Acceptance:

1. All validation commands succeed on the authoritative checkout.
2. Manual spot checks are captured with expected operator-visible outcomes.
3. `run-state.json` records a successful terminal state.
4. The parent writes `.runs/plan-18/sentinels/task-m18-p3-parent-validation-wall-and-closeout.ok`.

## Completion Criteria Tied To PLAN-18 Definition Of Done

The run is complete only if all of these are true:

1. `agent status` no longer aborts on strict parent-selection ambiguity or stale linkage that can be rendered truthfully with warnings.
2. doctor and toolbox surfaces remain strict and fail closed.
3. status rows explicitly identify whether they come from `live_runtime` or `trace_fallback`.
4. same-agent sibling participants remain distinct on the status surface when lineage evidence exists.
5. live/tombstone suppression only suppresses matching participant-aware fallback rows.
6. nested parent correlation respects `parent_participant_id` when present and still rejects malformed tuples.
7. canonical naming in docs and status/report surfaces uses `orchestration_session_id` and `participant_id`.
8. legacy alias reads remain green.
9. no new public `start|resume|fork|stop` surface is introduced.
10. repo-truth docs reflect the landed behavior and nothing broader.

## Final State

Success requires all of:

1. every required sentinel exists
2. no blocked artifact exists under `.runs/plan-18/`
3. accepted outputs were integrated in the prescribed order only
4. final validation commands pass
5. manual spot checks confirm the same operator contract the tests prove
6. `contract-freeze.json` and the final merged tree still agree on the frozen `PLAN-18` contract

Blocked termination requires any of:

1. hard-guard violation
2. non-retryable rejection
3. exhausted retry budget
4. merge refusal with no legal redrive path
5. failed validation wall
6. docs requiring overclaim to appear complete

On blocked termination the parent must write:

1. `.runs/plan-18/blocked.json`
2. terminal state and rationale in `run-state.json`
3. gate and failure summary in `session-log.md`
4. preserved evidence under `.runs/plan-18/quarantine/` and the relevant task artifact directory
