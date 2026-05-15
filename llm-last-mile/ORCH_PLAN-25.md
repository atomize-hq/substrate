# ORCH_PLAN-25: Execute PLAN-25 Through Durable Host Session Truth Freeze, Single-Lane Lifecycle QA Hardening, And Late Inbox-Scope Cleanup

Live workspace branch: `feat/host-orchestrator-durable-session`  
Authoritative execution branch for this run: `feat/host-orchestrator-durable-session`  
Plan source: [PLAN-25.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-25.md)  
Style references: [ORCH_PLAN-20.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-20.md), [ORCH_PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-22.md)  
Packet index: [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)  
Truth anchors: [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)  
Execution type: fresh orchestration controller for durable host-session closeout, parent-frozen public contract, parent-only gates and integration, one real code lane, one real docs lane, one late cleanup lane  
Live root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Initial concurrent worker cap: `2`  
Total lanes across the full run: `3`

## Summary

This document is the execution controller for `PLAN-25`, not a restatement of it.

The public contract is already chosen on `feat/host-orchestrator-durable-session` and is frozen for this run:

- `turn` is prompt-taking follow-up on the same durable session.
- `reattach` is attached-owner recovery only.
- `stop` is durable closeout for attached and parked host sessions.
- `status --json` is the authoritative parked-session read surface.
- Detached-world follow-up stays fail closed until `reattach` restores an active host owner.

The orchestration job is to walk that frozen contract to completion without reopening product design. The run is honest only if it lands all of the following on the same merged tree:

- repo truth docs stop claiming parked `status`, `reattach`, or `stop` are unfinished,
- one same-session lifecycle regression proves parked `status`, parked `turn`, `reattach`, and `stop`,
- explicit command-level parked-status assertions prove `parked_resumable` and `awaiting_attention` rows retain live-runtime fields,
- inbox wording is narrowed to persistence, posture normalization, internal ack/dismiss, and dev-support/test ingress only,
- the final validation wall is published and green.

Real concurrency is capped at `2` at launch, not `3`, because there is only one real code hotspot:

1. [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) is a single ownership hotspot and must not be split across workers.
2. The docs lane is real and independent.
3. The inbox-comment cleanup lane is real but cannot start until code and docs converge, because it may touch runtime comment surfaces near the same contract seams and depends on the final wording chosen by the merged tree.

Frozen run shape:

1. `task/m25-p1-parent-contract-freeze-and-run-init`
2. `task/m25-g1-window-a-launch-gate`
3. parallel Window A
   - `task/m25-l1-lifecycle-qa-hardening`
   - `task/m25-l2-truth-doc-convergence`
4. `task/m25-g2-window-a-integration-gate`
5. `task/m25-p2-parent-window-a-integration`
6. `task/m25-g3-cleanup-launch-gate`
7. `task/m25-l3-inbox-scope-comment-tightening`
8. `task/m25-g4-validation-wall-gate`
9. `task/m25-p3-parent-validation-wall`
10. `task/m25-p4-parent-closeout-phase`

## Hard Guards

These are run-stopping invariants, not preferences.

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/host-orchestrator-durable-session`.
2. The parent agent is the only integrator, the only approval authority, and the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/**`.
3. The public contract is frozen exactly to:
   - `substrate agent start --backend <backend_id> --prompt ... --json` for root prompt-taking.
   - `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ... --json` for same-session prompt-taking follow-up.
   - `substrate agent reattach --session <orchestration_session_id> --json` for attached-owner recovery only.
   - `substrate agent stop --session <orchestration_session_id> --json` for durable closeout.
   - `substrate agent status --json` for authoritative parked-session truth.
4. `reattach` is never repurposed as a prompt-taking alias.
5. Detached-world follow-up remains fail closed until `reattach` restores an active host owner. No lane may weaken that rule.
6. The durable inbox contract stays narrow:
   - persistence exists,
   - posture normalization into `awaiting_attention` exists,
   - internal ack/dismiss support exists,
   - dev-support and test ingress exist,
   - no public inbox command surface is shipped,
   - no public inbox product workflow may be implied by docs or comments.
7. The implementation sequence is frozen:
   - freeze repo truth docs,
   - add one same-session lifecycle regression,
   - add explicit parked-status assertions,
   - tighten inbox comments only if needed,
   - publish the validation wall.
8. `agent_public_control_surface_v1.rs` is a single-owner hotspot. No parallel lane may touch it except `L1`.
9. `L1` is the only lane allowed to expose a production-code change, and any such change must be tiny, test-driven, and directly required by the new lifecycle/status assertions. No speculative runtime hardening is allowed.
10. `L2` may not touch Rust files, tests, `.runs/**`, or plan-controller files.
11. `L3` may only touch runtime comments if needed. It may not make semantic Rust changes, may not reopen docs truth, and may not write `.runs/**`.
12. No lane may add, rename, or widen public verbs, selectors, inbox surfaces, or recovery semantics.
13. No lane may claim public inbox productization, automatic resume from inbox items, or detached-world follow-up success without `reattach`.
14. Manual validation is host-only for this slice. `awaiting_attention` visibility and detached-world fail-closed behavior remain automated-only checks.
15. No lane may edit `PLAN-25.md` or this controller during execution.
16. If the frozen public contract, the narrow inbox contract, the single-owner hotspot rule, or the final validation wall becomes disputed, the run stops and the parent writes `blocked.json`.

Stop the run, write `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/blocked.json`, and do not advance if any of these occur:

1. A worker needs a second concurrent code lane touching [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs).
2. `L1` requires a broad runtime refactor instead of a tiny test-exposed fix.
3. `L2` can only make the docs truthful by inventing a public inbox surface or rewriting the recovery contract.
4. `L3` needs to make semantic runtime changes instead of comment tightening.
5. Parked or `awaiting_attention` command-level status assertions cannot be proven deterministically in the existing shell control suite.
6. Detached-world follow-up would need to succeed without `reattach` to keep tests green.
7. The final validation wall cannot prove targeted shell control coverage, targeted state-store coverage, and full workspace health on the same merged tree.

### Blocked-Run Record Contract

`blocked.json` is parent-written only, exactly once, at the moment the parent decides the run cannot advance.

Required fields in `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/blocked.json`:

- `run_id`
- `authoritative_branch`
- `plan_source`
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

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25/lifecycle-qa-hardening`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25/truth-doc-convergence`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25/inbox-scope-comment-tightening`

Worker branches:

- `codex/feat-host-orchestrator-durable-session-m25-lifecycle-qa-hardening`
- `codex/feat-host-orchestrator-durable-session-m25-truth-doc-convergence`
- `codex/feat-host-orchestrator-durable-session-m25-inbox-scope-comment-tightening`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25/lifecycle-qa-hardening \
  -b codex/feat-host-orchestrator-durable-session-m25-lifecycle-qa-hardening \
  feat/host-orchestrator-durable-session

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25/truth-doc-convergence \
  -b codex/feat-host-orchestrator-durable-session-m25-truth-doc-convergence \
  feat/host-orchestrator-durable-session

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-25/inbox-scope-comment-tightening \
  -b codex/feat-host-orchestrator-durable-session-m25-inbox-scope-comment-tightening \
  feat/host-orchestrator-durable-session
```

No separate parent integration worktree is introduced. The parent integrates only on `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/`:

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

Required per-task artifact roots:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-p1-parent-contract-freeze-and-run-init/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-g1-window-a-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-l1-lifecycle-qa-hardening/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-l2-truth-doc-convergence/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-g2-window-a-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-p2-parent-window-a-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-g3-cleanup-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-l3-inbox-scope-comment-tightening/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-g4-validation-wall-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-p3-parent-validation-wall/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-p4-parent-closeout-phase/`

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

If a worker task is intentionally skipped as `noop`, the parent still writes those three files in that task directory as parent-authored `noop` records, writes the `summary.md` rationale, and then writes the task sentinel. No worker is launched in that branch.

Required sentinels:

- `.runs/plan-25/sentinels/task-m25-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-25/sentinels/task-m25-g1-window-a-launch-gate.ok`
- `.runs/plan-25/sentinels/task-m25-l1-lifecycle-qa-hardening.ok`
- `.runs/plan-25/sentinels/task-m25-l2-truth-doc-convergence.ok`
- `.runs/plan-25/sentinels/task-m25-g2-window-a-integration-gate.ok`
- `.runs/plan-25/sentinels/task-m25-p2-parent-window-a-integration.ok`
- `.runs/plan-25/sentinels/task-m25-g3-cleanup-launch-gate.ok`
- `.runs/plan-25/sentinels/task-m25-l3-inbox-scope-comment-tightening.ok`
- `.runs/plan-25/sentinels/task-m25-g4-validation-wall-gate.ok`
- `.runs/plan-25/sentinels/task-m25-p3-parent-validation-wall.ok`
- `.runs/plan-25/sentinels/task-m25-p4-parent-closeout-phase.ok`

`contract-freeze.json` must record at minimum:

- `authoritative_branch: "feat/host-orchestrator-durable-session"`
- the frozen `turn` / `reattach` / `stop` / `status --json` contract
- the frozen narrow inbox contract
- the exact initial worker cap of `2`
- the single-owner hotspot file path for `L1`
- the rule that only `L1` may expose a tiny runtime fix
- the host-only manual validation ceiling
- the automated-only status for `awaiting_attention` and detached-world follow-up checks

`merge-order.json` must record at minimum:

- `integration_order: ["task/m25-l2-truth-doc-convergence", "task/m25-l1-lifecycle-qa-hardening", "task/m25-l3-inbox-scope-comment-tightening"]`
- `l2_acceptance_basis: "frozen_contract_only"`
- `l1_acceptance_basis: "accepted_l2_tree_if_no_contract_drift_else_replay_after_runtime_fix"`
- `l3_acceptance_basis: "accepted_p2_tree_only"`
- `replay_required_if_l1_changes_contract_relevant_wording: true`
- `quarantine_on_hotspot_collision: true`

## Workstream Plan

### PLAN-25 Workstream Mapping

| PLAN-25 workstream | Orchestration tasks | Why this mapping is exact |
| --- | --- | --- |
| Freeze repo truth docs | `task/m25-l2-truth-doc-convergence`, `task/m25-g2-window-a-integration-gate`, `task/m25-p2-parent-window-a-integration` | The docs lane owns only truth convergence in repo docs and packet docs. It does not invent runtime behavior or change tests. |
| Add one same-session lifecycle regression plus explicit parked-status assertions | `task/m25-l1-lifecycle-qa-hardening`, `task/m25-g2-window-a-integration-gate`, `task/m25-p2-parent-window-a-integration` | The code lane owns the shared shell control-suite hotspot and any tiny runtime fix directly exposed by those assertions. |
| Tighten inbox comments only if needed, then publish final validation wall | `task/m25-g3-cleanup-launch-gate`, `task/m25-l3-inbox-scope-comment-tightening`, `task/m25-g4-validation-wall-gate`, `task/m25-p3-parent-validation-wall`, `task/m25-p4-parent-closeout-phase` | Cleanup must wait for merged code plus docs truth. `L3` owns comments only if needed, and the parent owns all validation planning, execution, and run-state artifacts. |

### Concurrency And Merge Order

Concurrency rules:

1. Worker cap is exactly `2` until `g2` completes.
2. Only `L1` and `L2` may start in parallel.
3. `L3` waits for accepted and integrated `L1` plus `L2`.
4. No second code lane is authorized around [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs). That file is one hotspot and splitting it would create artificial parallelism, merge noise, and contract drift risk.
5. The docs lane is independent enough to start immediately because the public contract is already frozen. It may not change that contract.
6. The cleanup lane is deliberately late because its runtime comment surfaces may overlap the same contract seams `L1` uses, and it depends on the final merged wording from both code and docs truth.

Why the merge order is `L2` then `L1` then `L3`:

1. `PLAN-25` freezes repo truth first, so the docs lane is integrated first when it stays within the frozen contract.
2. `L1` is the only lane that can expose a real defect in the lifecycle/status path. If it does, the runtime/test fix lands after the truth freeze and the parent replays `L2` only if wording must change.
3. `L3` cannot run honestly until the merged tree tells one story about contract wording, lifecycle proof, and inbox boundaries.

Quarantine rules:

- Quarantine `L1` if it touches files outside its frozen ownership set, adds new public semantics, or makes a broad runtime refactor.
- Quarantine `L1` if it modifies more than one production Rust file without the new assertions proving both changes are necessary.
- Quarantine `L2` if it touches any Rust file, any test file, `PLAN-25.md`, `ORCH_PLAN-25.md`, or `.runs/**`.
- Quarantine `L2` if it claims public inbox productization, world-originated inbox workflows, or detached-world recovery beyond `reattach`.
- Quarantine `L3` if it touches tests, docs, `.runs/**`, or makes semantic code changes instead of comment-only tightening.
- Quarantine any lane that tries to create a third initial code stream around `agent_public_control_surface_v1.rs`.

### Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m25-p1-parent-contract-freeze-and-run-init` | parent | — | authoritative checkout | frozen contract, run artifacts, lane ownership, merge order |
| `task/m25-g1-window-a-launch-gate` | parent | `p1` | authoritative checkout | launch approval for `L1` and `L2` only |
| `task/m25-l1-lifecycle-qa-hardening` | worker | `g1` | `lifecycle-qa-hardening` / `codex/feat-host-orchestrator-durable-session-m25-lifecycle-qa-hardening` | same-session lifecycle regression, explicit parked-status assertions, tiny runtime fix only if exposed |
| `task/m25-l2-truth-doc-convergence` | worker | `g1` | `truth-doc-convergence` / `codex/feat-host-orchestrator-durable-session-m25-truth-doc-convergence` | repo truth freeze for public contract and honest inbox wording |
| `task/m25-g2-window-a-integration-gate` | parent | `l1`, `l2` | authoritative checkout | acceptance, rejection, or quarantine for Window A |
| `task/m25-p2-parent-window-a-integration` | parent | `g2` | authoritative checkout | merged docs truth then merged lifecycle/status proof |
| `task/m25-g3-cleanup-launch-gate` | parent | `p2` | authoritative checkout | decide whether inbox-scope comment tightening is needed |
| `task/m25-l3-inbox-scope-comment-tightening` | worker or parent-noop | `g3` | `inbox-scope-comment-tightening` / `codex/feat-host-orchestrator-durable-session-m25-inbox-scope-comment-tightening` | comment-only runtime tightening if launched; otherwise parent-authored `noop` records in the `L3` task artifact root |
| `task/m25-g4-validation-wall-gate` | parent | `g3` | authoritative checkout | permission to run the final validation wall after accepted `L3` or parent-recorded `noop` |
| `task/m25-p3-parent-validation-wall` | parent | `g4` | authoritative checkout | exact automated wall results plus recorded manual host-only smoke evidence |
| `task/m25-p4-parent-closeout-phase` | parent | `p3` | authoritative checkout | final artifact audit, acceptance record, terminal run state |

### Lane Ownership By File Set

| Lane | Allowed files | Forbidden touch surfaces |
| --- | --- | --- |
| `L1` / lifecycle QA hardening | [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs), `crates/shell/tests/support/**` only if directly required by that suite, [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) only for a tiny test-exposed lifecycle/status fix, [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) only for a tiny test-exposed lifecycle/status fix | every other Rust file, `docs/**`, repo-root truth docs, `llm-last-mile/**`, `.runs/**` |
| `L2` / truth-doc convergence | [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md), [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md), [llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md) | every Rust file, every test file, `PLAN-25.md`, `ORCH_PLAN-25.md`, `.runs/**` |
| `L3` / inbox-scope comment tightening | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) comments only, [crates/shell/src/execution/agent_dev_support.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs) comments only, [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) comments only if needed around the `turn` / `reattach` / `stop` split | every test file, every doc file, all run-state and validation artifacts under `.runs/**`, semantic Rust changes |

### Kickoff Initialization Order

The parent initializes the run in this exact order:

1. Create `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/`, `.runs/plan-25/sentinels/`, `.runs/plan-25/quarantine/`, and every `.runs/task-m25-*/` directory.
2. Create `task.json`, `commands.txt`, and `summary.md` in every task directory.
3. Create `gate-checklist.md` and `gate-result.json` in every gate task directory.
4. Create placeholder `worker-report.md`, `worker-output.patch`, and `evidence-manifest.json` in every worker task directory so a later parent-authored `noop` record can reuse the same artifact contract if a worker is never launched.
5. Write `tasks.json` as the canonical launch queue and execution ledger.
6. Write `run-state.json` with `current_phase: "kickoff"`, `worker_cap: 2`, `authoritative_branch: "feat/host-orchestrator-durable-session"`, every task in `pending`, and empty `accepted`, `rejected`, `quarantined`, and `blocked` arrays.
7. Write `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, and `validation-wall.md`.
8. Freeze the exact public contract and narrow inbox contract in `contract-freeze.json`.
9. Freeze the rule that [agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) belongs to `L1` alone.
10. Freeze the host-only manual validation sequence and the automated-only status of `awaiting_attention` and detached-world fail-closed checks.
11. Seed all worktrees from the exact same post-`p1` tree.
12. Record the authoritative branch, worktree paths, lane ownership, and retry budget in `session-log.md`.

### Launch And Integration Gates

#### `task/m25-p1-parent-contract-freeze-and-run-init`

Parent only.

Scope:

1. Create the full `.runs/plan-25` tree and every per-task artifact directory.
2. Freeze the exact `turn` / `reattach` / `stop` / `status --json` contract.
3. Freeze the narrow inbox contract and the ban on public inbox product claims.
4. Freeze the real concurrency cap of `2` for the opening window.
5. Freeze that `L1` alone owns the test hotspot and may expose only a tiny test-driven runtime fix.
6. Freeze that `L2` is docs-only and `L3` is comment-only if needed.

Acceptance:

- all parent-owned run-state artifacts exist,
- lane ownership is explicit,
- merge order is explicit,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-p1-parent-contract-freeze-and-run-init.ok` exists.

#### `task/m25-g1-window-a-launch-gate`

Parent only.

Checks:

1. `p1` is accepted.
2. `L1` and `L2` worktrees are seeded from the exact same post-`p1` tree.
3. `L1` prompt explicitly says:
   - one same-session lifecycle regression,
   - explicit parked and `awaiting_attention` status assertions,
   - detached-world fail-closed must stay green,
   - no speculative runtime changes,
   - no file ownership beyond the frozen set.
4. `L2` prompt explicitly says:
   - freeze docs truth only,
   - do not productize inbox,
   - do not rewrite the public recovery contract,
   - do not touch Rust, tests, or plan-controller files.
5. `L3` is not launched.

Acceptance:

- only `L1` and `L2` are authorized to start,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-g1-window-a-launch-gate.ok` exists.

#### `task/m25-g2-window-a-integration-gate`

Parent only.

Checks:

1. `L1` and `L2` each returned `worker-report.md`, `worker-output.patch`, `commands.txt`, and `evidence-manifest.json`.
2. `L1` added exactly the lifecycle/status proof asked for:
   - one same-session lifecycle regression on one orchestration session id,
   - explicit parked `status --json` field assertions,
   - explicit `awaiting_attention` `status --json` field assertions.
3. `L1` preserved detached-world fail-closed behavior.
4. Any `L1` production-code change is tiny, directly exposed by the new assertions, and limited to the frozen runtime files.
5. `L2` removed stale "unfinished" wording around parked `status`, `reattach`, and `stop`.
6. `L2` narrowed inbox wording to shipped persistence, posture normalization, internal ack/dismiss, and dev-support/test ingress only.
7. `L2` did not touch Rust files, tests, or plan-controller files.

Acceptance:

- each Window A lane is marked `accepted`, `rejected`, or `quarantined`,
- any required `L2` replay is decided here,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-g2-window-a-integration-gate.ok` exists.

#### `task/m25-p2-parent-window-a-integration`

Parent only.

Integration sequence:

1. Integrate accepted `L2` first.
2. Integrate accepted `L1` second.
3. If `L1` changed any contract-relevant wording basis, replay `L2` on top of the accepted `L1` tree before closing `p2`.
4. Run the targeted automated checks immediately after `L1` lands.

Minimum commands:

```bash
git cherry-pick <accepted-l2-commit>
git cherry-pick <accepted-l1-commit>
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
```

Acceptance:

- docs truth and lifecycle/status proof coexist on one merged tree,
- targeted shell control and state-store coverage are green,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-p2-parent-window-a-integration.ok` exists.

#### `task/m25-g3-cleanup-launch-gate`

Parent only.

Checks:

1. `p2` is accepted.
2. The parent decides whether comment tightening is actually needed.
3. If no comment drift exists, the parent does not launch a worker. Instead, the parent writes a `noop` record into `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m25-l3-inbox-scope-comment-tightening/`:
   - `summary.md` with the `noop` rationale,
   - `worker-report.md` marked parent-authored `noop`,
   - empty `worker-output.patch`,
   - `evidence-manifest.json` marked `noop`,
   - `.runs/plan-25/sentinels/task-m25-l3-inbox-scope-comment-tightening.ok`.
4. If comment drift exists, `L3` is launched on the accepted `p2` tree with comment-only scope.
5. `L3` prompt explicitly forbids semantic Rust changes and all doc/test changes.

Acceptance:

- `L3` is either launched narrowly or marked `noop`,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-g3-cleanup-launch-gate.ok` exists.

#### `task/m25-g4-validation-wall-gate`

Parent only.

Checks:

1. `L3` is accepted or recorded as `noop`.
2. `validation-wall.md` lists the exact automated commands and the exact host-only manual smoke path.
3. `validation-wall.md` explicitly records that `awaiting_attention` visibility and detached-world fail-closed remain automated-only checks.
4. No quarantined output remains unresolved.

Acceptance:

- the final wall can run once on the fully merged tree,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-g4-validation-wall-gate.ok` exists.

#### `task/m25-p3-parent-validation-wall`

Parent only.

Scope:

1. Run the exact automated validation wall.
2. Run and record the host-only manual lifecycle smoke path.
3. Confirm all acceptance checks before closeout.

Minimum automated commands:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test --workspace -- --nocapture
```

Acceptance:

- automated wall is green on the final merged tree,
- manual host-only smoke evidence is recorded,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-p3-parent-validation-wall.ok` exists.

#### `task/m25-p4-parent-closeout-phase`

Parent only.

Scope:

1. Audit all sentinels and required artifacts.
2. Confirm `blocked.json` is absent.
3. Record final accepted lanes, quarantines, command outcomes, and any residual follow-up.
4. Mark the run complete only if every acceptance check in this controller is satisfied.

Acceptance:

- terminal run state is written,
- accepted scope matches `PLAN-25`,
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-25/sentinels/task-m25-p4-parent-closeout-phase.ok` exists.

## Context-Control Rules

1. The parent owns full context. Workers get only the minimum contract excerpt, owned file list, forbidden surface list, exact commands, and artifact return format needed for their lane.
2. The parent never hands `L1` a docs brief and never hands `L2` the shared test hotspot. This preserves the real code/docs split.
3. `L1` receives the frozen lifecycle/status acceptance bullets and the tiny-runtime-fix rule, not the entire repo history.
4. `L2` receives only the frozen public contract, the frozen inbox contract, and the exact truth files it owns. It does not receive latitude to reinterpret product scope.
5. `L3` receives only the accepted `p2` diff summary plus the exact runtime comment targets that still drift. If there is no drift, the parent writes the `noop` record and `L3` is never launched.
6. Every worker prompt must include:
   - task ID,
   - attempt number,
   - worktree path,
   - branch,
   - allowed files,
   - forbidden files,
   - frozen contract clauses relevant to that lane,
   - exact commands to run,
   - retry budget,
   - required return artifacts,
   - sentinel path.
7. Every launched worker return must include:
   - changed files list,
   - commands run with exit codes,
   - explicit attempt classification: `clean`, `retryable`, or `blocked`,
   - unresolved assumptions,
   - `worker-output.patch`,
   - `worker-report.md`,
   - `evidence-manifest.json`.
8. For a `noop` worker task, the parent writes the task-local artifact trio instead of a worker. Those files must explicitly say `parent-authored noop` so the artifact contract stays intact without implying worker-owned output.
9. The parent writes all `.runs/plan-25/**` artifacts before, between, and after worker attempts. Workers never write orchestration state.
10. If a lane proposes widening scope, the parent does not negotiate in-branch. The parent stops the run or reissues a narrower prompt after updating the run-state record.
11. Parent integration notes must call out explicitly whether `L1` exposed a runtime defect or stayed test-only. That distinction matters for final review.

## Tests And Acceptance

### Exact Automated Validation Wall

The final automated wall is frozen to:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test --workspace -- --nocapture
```

These commands must run on the final merged tree, in that order, unless the parent records a stricter superset in `validation-wall.md`.

### Host-Only Manual Smoke Path

The manual smoke path is host-only and must be recorded by the parent. Use one real host backend and one real durable orchestration session:

```bash
substrate agent start --backend <host_backend_id> --prompt "first" --json
substrate agent status --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent status --json
substrate agent reattach --session <orchestration_session_id> --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent status --json
```

The parent records success only if all of the following are true:

1. `start` establishes one durable orchestration session.
2. The first `status --json` shows the session as a live parked session with authoritative runtime fields populated.
3. `turn` reuses the same orchestration-session id and does not require `reattach` first.
4. The second `status --json` still shows that same session with authoritative runtime fields populated.
5. `reattach` restores attached ownership without submitting a prompt.
6. `stop` closes that same durable session terminally.
7. The final `status --json` no longer presents the session as a live non-terminal durable session.

### Automated-Only Checks

These remain automated-only for this slice and must not be faked into a manual path:

1. `awaiting_attention` command-level visibility.
2. Detached-world follow-up fail-closed behavior with `reattach` guidance.

Reason:

- there is no public inbox producer surface for creating `awaiting_attention` manually without dev-support/test ingress,
- detached-world follow-up is fixture-heavy and already belongs in automated regression coverage.

### Acceptance Checklist

The run is complete only when every item below is true:

- repo truth docs no longer say parked `status`, `reattach`, or `stop` are unfinished,
- inbox docs say exactly what is shipped today and nothing more,
- one same-session lifecycle regression proves parked `status`, parked `turn`, `reattach`, and `stop` on one orchestration session id,
- command-level parked rows show live-runtime `posture`, `attached_participant_id`, and `pending_inbox_count`,
- command-level `awaiting_attention` rows show the same live-runtime fields,
- detached-world follow-up still fails closed with `reattach` guidance,
- any production-code change made by `L1` is tiny and directly exposed by the new assertions,
- cleanup comments, if any, do not imply a public inbox product surface,
- the exact automated validation wall is green,
- the manual host-only lifecycle validation sequence is recorded,
- all `.runs/plan-25` artifacts and sentinels required by this controller exist.

## Assumptions

1. The live branch `feat/host-orchestrator-durable-session` already contains the intended durable-session model; this run is closeout and contract hardening, not architectural expansion.
2. The existing shell control suite can absorb one more lifecycle/status regression without needing a new harness.
3. Any production runtime fix exposed by `L1` will be narrow enough to fit inside [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) or [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) without reopening the product model.
4. Repo truth can be made consistent without editing `PLAN-25.md` or this controller.
5. The parent has a usable host backend available for the manual smoke path.
6. If the parent discovers the docs cannot be made truthful without inventing new product behavior, the correct action is to block the run rather than widen scope.
