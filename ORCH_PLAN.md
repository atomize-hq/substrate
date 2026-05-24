# ORCH_PLAN: Explicit Control-Only Recovery And Honest Successor Allocation Execution Controller

Authoritative execution branch: `feat/gateway-mediated-llm-fulfillment`  
Plan source: [PLAN.md](PLAN.md)  
Style reference only: [llm-last-mile/ORCH_PLAN-25.md](llm-last-mile/ORCH_PLAN-25.md)  
Workspace root: `/home/azureuser/__Active_Code/atomize-hq/substrate`  
Worktree root: `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery`  
Run id: `plan-control-only-session-recovery`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Initial concurrent worker cap: `0` during parent freeze  
Peak concurrent worker cap: `2` after A2  
Parent role: sole integrator, sole gate owner, sole writer of `.runs/**`, sole authority for merge order, hotspot ownership, acceptance, blockage, and final validation

## Summary

This document executes the current [PLAN.md](PLAN.md). It is an execution controller, not a restatement of the plan.

This run is complete only if one merged tree proves all of the following together:

1. A0 removes the local `unified-agent-api` patch override and the workspace resolves against the published dependency.
2. A1 persists a durable host attach contract on the orchestration session.
3. A2 removes the hidden owner-helper convergence and splits internal control-only attach, prompt-bearing turn launch, and successor allocation.
4. A3 makes `reattach` continuity-only for this slice:
   - no prompt,
   - same durable session id,
   - fail closed when continuity is absent.
5. A4 makes `fork` a real successor allocator:
   - successor copies attach-contract shape,
   - successor clears `continuity_uaa_session_id`,
   - successor returns `parked_resumable`,
   - successor returns `attached_participant_id = null`,
   - no synthetic prompt,
   - no borrowed parent continuity.
6. A5 is preserved:
   - public `start` and `turn` remain the only prompt-bearing public verbs,
   - detached-world follow-up remains fail-closed,
   - no new public verb, schema, or policy surface appears.
7. A6 truth-syncs docs and downstream slices 29/30/31 to the landed runtime architecture.
8. A7 proves the whole tree with focused grep gates, focused tests, and full workspace validation.

Frozen orchestration shape:

1. Parent-only freeze and source lock.
2. `L0` executes A0 and A1 on one branch.
3. Parent integrates A0/A1 and opens A2.
4. `L1` executes A2 on one branch.
5. Parent integrates A2 and then opens the only low-risk parallel window:
   - `L2` executes A3 reattach work.
   - `L3` drafts A6 docs sync in parallel.
6. Parent merges `L2` first, then replays and merges `L3` onto the accepted runtime tree.
7. `L4` executes A4 after `L2` is accepted and after the parent reassigns the `agents_cmd.rs` hotspot.
8. Parent verifies A5 invariants on the merged runtime tree.
9. Parent runs A7 validation and closeout last.

Default rule: do not run A4 concurrently with A3. The real merge-risk seam is `crates/shell/src/execution/agents_cmd.rs`, and this controller treats that as a serialized ownership handoff instead of pretending it is safe parallelism.

## Hard Guards

These are run-stopping invariants.

1. The authoritative integration checkout remains `/home/azureuser/__Active_Code/atomize-hq/substrate` on `feat/gateway-mediated-llm-fulfillment`.
2. The parent is the only integrator and the only approval authority.
3. The parent is the only writer of `/home/azureuser/__Active_Code/atomize-hq/substrate/.runs/plan-control-only-session-recovery/**`.
4. Public `start` and `turn` remain the only prompt-bearing public verbs.
5. Public `reattach` is control-only in this slice and may use continuity privately only when that continuity already exists.
6. Public `reattach` fails closed when `continuity_uaa_session_id` is absent, stale, or invalid.
7. Public `fork` is successor durable-session allocation only:
   - no prompt,
   - no immediate attachment,
   - no false `active` truth,
   - no inherited live continuity token.
8. Successor normalization is frozen:
   - copy attach-contract shape,
   - clear `continuity_uaa_session_id`,
   - set `attached_participant_id = null`,
   - set `posture = parked_resumable`,
   - leave `pending_inbox_count = 0`.
9. Detached-world follow-up remains fail-closed until valid host ownership returns.
10. `MemberTurnSubmitRequestV1` and `/v1/member_turn/stream` remain unchanged.
11. No new public verb, schema, selector grammar, policy surface, or capability flag may be introduced.
12. No production path may use `prompt: ""` or `NoPromptRecovery` as the architectural meaning of public `reattach` or `fork`.
13. No production `fork` path may route through `agent_api.session.resume.v1`.
14. No worker may edit [PLAN.md](PLAN.md), this controller, or `.runs/**`.
15. No docs lane may touch Rust or test files.
16. No runtime lane may touch docs except where the parent explicitly replays or resolves merge drift after acceptance.
17. `crates/shell/src/execution/agents_cmd.rs` is a single-owner hotspot with explicit parent-controlled ownership transfer.
18. `crates/shell/src/execution/agent_runtime/state_store.rs` is foundation-owned in A1 and cannot be touched by A3 or A4 until A1 is merged.
19. A6 may draft in parallel with A3, but docs may not merge before the accepted runtime truth exists.
20. Every lane must run GitNexus impact analysis before editing any function, method, or other symbol it touches.
21. Any `HIGH` or `CRITICAL` GitNexus impact result must be escalated to the parent before edits proceed; no worker may absorb that risk silently.
22. Every worker handoff must include GitNexus `detect_changes` output.
23. The parent must run a final GitNexus `detect_changes` pass on the merged tree before final acceptance.
24. If `PLAN.md` changes materially during execution, stop the run and restart from parent freeze.

## Blocked-Run Conditions

Stop the run, write `blocked.json`, and do not advance if any of these occur:

1. A0 cannot resolve `unified-agent-api = "=0.3.5"` from the published source.
2. A1 requires inventing a second durable attach-truth model instead of extending `OrchestrationSessionRecord`.
3. A2 cannot remove the hidden convergence without adding a new public verb, new schema, or new policy/config surface.
4. A3 can only pass by submitting a prompt or by guessing a fresh attach when continuity is absent.
5. A4 can only pass by claiming the parent's continuity token, attaching a live client, or returning false `active` truth.
6. The only path to green requires retaining `prompt: ""`, `NoPromptRecovery`, or `agent_api.session.resume.v1` as the implementation substrate for public `reattach` or `fork`.
7. Docs can only be made truthful by contradicting the merged code.
8. The validation wall cannot prove grep gates, focused tests, world fail-closed behavior, and full workspace gates on the same merged tree.
9. Any worker touches files outside its frozen ownership set and the parent cannot cleanly quarantine the output.
10. Any merge requires concurrent edits to `crates/shell/src/execution/agents_cmd.rs` from more than one live worker lane.

`blocked.json` must include:

1. `run_id`
2. `authoritative_branch`
3. `plan_source`
4. `timestamp`
5. `current_gate`
6. `current_task`
7. `stop_condition`
8. `blocking_files`
9. `accepted_tasks`
10. `rejected_or_quarantined_tasks`
11. `required_parent_action`

## Fresh Worktrees And Branches

Fresh worktree root:

- `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery`

Authoritative integration checkout:

- `/home/azureuser/__Active_Code/atomize-hq/substrate`
- branch: `feat/gateway-mediated-llm-fulfillment`

Worker worktrees:

- `L0` A0/A1 foundation:
  - `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a0-a1-foundation`
  - `codex/feat-gateway-mediated-llm-fulfillment-a0-a1-foundation`
- `L1` A2 launch split:
  - `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a2-launch-split`
  - `codex/feat-gateway-mediated-llm-fulfillment-a2-launch-split`
- `L2` A3 control-only reattach:
  - `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a3-control-reattach`
  - `codex/feat-gateway-mediated-llm-fulfillment-a3-control-reattach`
- `L3` A6 docs and downstream sync:
  - `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a6-docs-sync`
  - `codex/feat-gateway-mediated-llm-fulfillment-a6-docs-sync`
- `L4` A4 fork successor allocator:
  - `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a4-successor-allocator`
  - `codex/feat-gateway-mediated-llm-fulfillment-a4-successor-allocator`

Exact setup commands:

```bash
mkdir -p /home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery

git -C /home/azureuser/__Active_Code/atomize-hq/substrate fetch origin
```

Create `L0` only after `G0` passes:

```bash
git -C /home/azureuser/__Active_Code/atomize-hq/substrate worktree add \
  /home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a0-a1-foundation \
  -b codex/feat-gateway-mediated-llm-fulfillment-a0-a1-foundation \
  feat/gateway-mediated-llm-fulfillment
```

Create `L1` only after `G1` passes:

```bash
git -C /home/azureuser/__Active_Code/atomize-hq/substrate worktree add \
  /home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a2-launch-split \
  -b codex/feat-gateway-mediated-llm-fulfillment-a2-launch-split \
  feat/gateway-mediated-llm-fulfillment
```

Create `L2` and `L3` only after `G2` passes:

```bash
git -C /home/azureuser/__Active_Code/atomize-hq/substrate worktree add \
  /home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a3-control-reattach \
  -b codex/feat-gateway-mediated-llm-fulfillment-a3-control-reattach \
  feat/gateway-mediated-llm-fulfillment

git -C /home/azureuser/__Active_Code/atomize-hq/substrate worktree add \
  /home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a6-docs-sync \
  -b codex/feat-gateway-mediated-llm-fulfillment-a6-docs-sync \
  feat/gateway-mediated-llm-fulfillment
```

Create `L4` only after `G3` passes and only from the then-current authoritative branch tip:

```bash
git -C /home/azureuser/__Active_Code/atomize-hq/substrate worktree add \
  /home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-control-only-session-recovery/a4-successor-allocator \
  -b codex/feat-gateway-mediated-llm-fulfillment-a4-successor-allocator \
  feat/gateway-mediated-llm-fulfillment
```

Concurrency contract:

1. Parent freeze and all gates are serialized.
2. `L0` and `L1` are serialized.
3. Peak low-risk parallelism is exactly `L2 + L3`.
4. `L4` is serialized after `L2` by default.
5. Worker cap is `2`; do not open a third active worker.

## Parent-Owned Run-State Surface

Canonical run root:

- `/home/azureuser/__Active_Code/atomize-hq/substrate/.runs/plan-control-only-session-recovery/`

Required top-level artifacts:

- `run-state.json`
- `tasks.json`
- `source-lock.json`
- `contract-freeze.json`
- `branch-map.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `session-log.md`
- `final-summary.md`
- `blocked.json` on blocked runs only
- `sentinels/`
- `tasks/`
- `gates/`

Required run sentinels:

- `sentinels/RUN_OPEN`
- `sentinels/RUN_BLOCKED` on blocked runs only
- `sentinels/RUN_COMPLETE` on successful closeout only

Required task directories:

- `tasks/P0-parent-freeze-and-run-init/`
- `tasks/L0-a0-a1-foundation/`
- `tasks/G1-foundation-accept/`
- `tasks/L1-a2-launch-split/`
- `tasks/G2-launch-split-accept/`
- `tasks/L2-a3-control-reattach/`
- `tasks/L3-a6-docs-sync/`
- `tasks/G3-parallel-window-accept/`
- `tasks/L4-a4-successor-allocator/`
- `tasks/G4-fork-accept/`
- `tasks/P1-a5-prompt-bearing-guard/`
- `tasks/P2-a7-validation-wall/`
- `tasks/G5-final-acceptance/`
- `tasks/P3-parent-closeout/`

Required gate directories:

- `gates/G0-run-freeze/`
- `gates/G1-foundation/`
- `gates/G2-launch-split/`
- `gates/G3-parallel-window/`
- `gates/G4-fork/`
- `gates/G5-final/`

Each task directory must contain:

- `task.json`
- `owner.txt`
- `status.txt`
- `scope.txt`
- `deliverable.txt`
- `dependencies.json`
- `changed-files.txt`
- `commands.txt`
- `exit-codes.json`
- `impact-analysis-summary.md`
- `gitnexus-detect-changes.txt`
- `handoff-notes.md`
- `summary.md`
- `HEAD_SHA.txt`
- `blocker-notes.md` when blocked
- one sentinel:
  - `READY_FOR_REVIEW`
  - `ACCEPTED`
  - `REJECTED`
  - `BLOCKED`

Each gate directory must contain:

- `gate.json`
- `evidence.md`
- one sentinel:
  - `OPEN`
  - `PASSED`
  - `FAILED`
  - `REOPENED`

Artifact rules:

1. Workers never write `.runs/**`.
2. Workers return branch, head SHA, changed files, commands run, exit codes, and unresolved blockers to the parent.
3. Workers must also return a GitNexus impact summary for every edited symbol and a `detect_changes` result for the lane branch.
4. The parent writes or updates every task and gate artifact before a task or gate changes state.
5. Parent transcription must include `impact-analysis-summary.md` and `gitnexus-detect-changes.txt` for every accepted or rejected worker lane.
6. Nothing is accepted until the parent has transcribed the handoff and written the sentinel.

`contract-freeze.json` must record at minimum:

1. `authoritative_branch: "feat/gateway-mediated-llm-fulfillment"`
2. `plan_source: "/home/azureuser/__Active_Code/atomize-hq/substrate/PLAN.md"`
3. the frozen public contract:
   - `start`
   - `turn`
   - `reattach`
   - `fork`
   - `stop`
4. `public_prompt_verbs: ["start", "turn"]`
5. `reattach_mode: "continuity_only_fail_closed"`
6. `successor_posture: "parked_resumable"`
7. `successor_attached_participant_id: null`
8. `successor_continuity_uaa_session_id: null`
9. `world_follow_up_mode: "fail_closed_without_host_ownership"`
10. `worker_cap: 2`
11. `hotspot_file: "crates/shell/src/execution/agents_cmd.rs"`

## Parent-Only Critical Path

The parent-only critical path is fixed:

1. `P0` freeze the run:
   - confirm branch,
   - confirm plan source,
   - write source lock,
   - write contract freeze,
   - write lane ownership,
   - write merge order,
   - open `RUN_OPEN`.
2. `G0` authorize `L0`.
3. Review and integrate `L0` A0/A1.
4. `G1` authorize `L1`.
5. Review and integrate `L1` A2.
6. `G2` authorize the only parallel window:
   - `L2` A3
   - `L3` A6 draft
7. Review and integrate `L2` first.
8. Replay and integrate `L3` against the accepted runtime tree.
9. `G3` authorize `L4`.
10. Review and integrate `L4` A4.
11. `P1` verify A5 invariants on the merged runtime tree.
12. `P2` execute A7 validation wall.
13. `G5` issue final acceptance or block.
14. `P3` write closeout and `RUN_COMPLETE`.

The parent never delegates:

1. source lock updates,
2. hotspot ownership changes,
3. merge decisions,
4. gate changes,
5. final validation,
6. blocked-run decisions.

## Worker Lanes

### Lane Ledger

| Lane | PLAN phases | Owner | Starts after | Ends at | Purpose |
| --- | --- | --- | --- | --- | --- |
| `L0` | A0, A1 | worker | `G0` | `G1` review | dependency floor plus durable attach-contract foundation |
| `L1` | A2 | worker | `G1` | `G2` review | split internal control-only attach, prompt turn, and successor allocation paths |
| `L2` | A3 | worker | `G2` | `G3` review | make `reattach` continuity-only and fail closed |
| `L3` | A6 | worker | `G2` | `G3` review | doc and downstream truth sync drafted against accepted A2 shape |
| `L4` | A4 | worker | `G3` | `G4` review | honest successor allocation and normalization |
| `P1` | A5 | parent | `G4` | `P2` | verify prompt-bearing invariants remain intact |
| `P2` | A7 | parent | `P1` | `G5` | full validation wall |

### File Ownership

| Lane | Allowed files | Forbidden touch surfaces |
| --- | --- | --- |
| `L0` | `Cargo.toml`, `Cargo.lock`, `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, directly related state-store/orchestration tests | `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/repl/async_repl.rs`, docs, `.runs/**` |
| `L1` | `crates/shell/src/repl/async_repl.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/agents_cmd.rs`, directly related `async_repl.rs` tests | docs, downstream packets, `.runs/**`, world-service files |
| `L2` | `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/repl/async_repl.rs`, `crates/shell/tests/agent_public_control_surface_v1.rs`, any minimal supporting test helpers | docs, downstream packets, world-service files, `.runs/**` |
| `L3` | `docs/USAGE.md`, `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`, `docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`, `UAA_PROMPTLESS_RESUME_FORK_SYNTHESIS.md`, `llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`, `llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md`, `llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md` | all Rust files, all test files, `.runs/**` |
| `L4` | `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`, successor-related control tests | docs, downstream packets, `.runs/**`, world-service transport schema changes |

Hotspot ownership rule:

1. `crates/shell/src/execution/agents_cmd.rs` belongs to exactly one live lane at a time.
2. Ownership transitions:
   - `L1` owns it during A2.
   - `L2` owns it during A3 after `G2`.
   - `L4` owns it during A4 after `G3`.
3. `L4` must not start until the parent records the ownership transfer in `lane-ownership.json`.

## Gate Sequencing

### `G0` Run Freeze

Pass only if:

1. branch is `feat/gateway-mediated-llm-fulfillment`,
2. `PLAN.md` is the source lock,
3. worktree root is empty or intentionally prepared,
4. parent artifacts and sentinels are initialized,
5. lane ownership and merge order are written.

### `G1` Foundation Accept

Pass only if `L0` proves:

1. no `[patch.crates-io]` override remains,
2. no lockfile path points at `../unified-agent-api/crates/agent_api`,
3. durable `host_attach_contract` exists on `OrchestrationSessionRecord`,
4. backward-compatible deserialize behavior remains,
5. successor-safe copy helper clears successor continuity,
6. focused foundation checks pass.

### `G2` Launch-Split Accept

Pass only if `L1` proves:

1. A2 lands on top of accepted A0/A1,
2. `InitialExecPromptPlan::NoPromptRecovery` no longer carries public `reattach` or `fork` meaning,
3. internal control flow explicitly separates:
   - control-only attach,
   - prompt-bearing resumed turn launch,
   - successor allocation,
4. `agents_cmd.rs` remains understandable and not broadened into a second hidden framework.

### `G3` Parallel Window Accept

Pass only if:

1. `L2` proves `reattach` is continuity-only and fail-closed,
2. `L2` does not introduce any prompt-bearing drift in `start` or `turn`,
3. `L3` docs are replayed onto the accepted `L2` tree when needed,
4. `L3` does not describe blank-prompt control semantics as live architecture,
5. the parent records the ownership handoff from `L2` to `L4` before opening `L4`.

### `G4` Fork Accept

Pass only if `L4` proves:

1. `fork` allocates successor durable truth first,
2. successor copies attach-contract shape,
3. successor clears `continuity_uaa_session_id`,
4. successor returns `parked_resumable`,
5. successor returns `attached_participant_id = null`,
6. successor preserves source lineage truth,
7. no synthetic prompt path exists,
8. no `resume.v1` route remains for public `fork`.

### `G5` Final Accept

Pass only if the merged tree proves:

1. A5 prompt-bearing invariants remain true,
2. A7 focused grep gates are green,
3. A7 focused tests are green,
4. world fail-closed behavior remains intact,
5. docs and downstream packets match the merged code,
6. full workspace gates are green,
7. the parent has reviewed worker `detect_changes` outputs and run a final merged-tree GitNexus `detect_changes` pass.

## Workstream Plan

| PLAN phase | Orchestration task | Owner | Merge rule | Required result |
| --- | --- | --- | --- | --- |
| A0 | `L0` foundation | worker | merge first | published `unified-agent-api` dependency floor only |
| A1 | `L0` foundation | worker | merged with A0 | durable attach contract persisted and successor-safe copy helper present |
| A2 | `L1` launch split | worker | merge second | explicit internal path split, no hidden control-via-prompt shaping |
| A3 | `L2` reattach | worker | merge before docs and before fork | continuity-only attach, fail closed without continuity |
| A4 | `L4` fork | worker | merge after `L2` by default | honest successor allocation and normalization |
| A5 | `P1` prompt guard | parent | no lane | start/turn prompt-bearing behavior preserved exactly |
| A6 | `L3` docs sync | worker | draft in parallel, merge after `L2` | docs and downstream packets match landed control architecture |
| A7 | `P2` validation wall | parent | final only | same-tree proof across grep, focused tests, and workspace gates |

## Merge Order

The merge order is fixed unless the parent explicitly blocks and restarts:

1. merge `L0` A0/A1,
2. merge `L1` A2,
3. merge `L2` A3,
4. replay and merge `L3` A6 onto the accepted `L2` tree,
5. create `L4` from the current authoritative tip,
6. merge `L4` A4,
7. run `P1` and `P2`.

Why this order is fixed:

1. A0/A1 must land before any runtime fan-out.
2. A2 is the shared launch-path split and must land before all later semantic work.
3. A3 is the low-risk runtime lane after A2.
4. A6 can draft in parallel after A2 but cannot merge before accepted runtime truth exists.
5. A4 is the real merge-risk seam because it shares `crates/shell/src/execution/agents_cmd.rs` with A3.
6. Creating `L4` only after `G3` eliminates fake parallelism and reduces rework.

## Context-Control Rules

1. Every worker reads only the current accepted tree plus the parent-written task scope.
2. Workers do not reopen the public contract.
3. Workers do not reinterpret plan language; ambiguities escalate to the parent.
4. Workers do not edit outside their lane ownership table.
5. Workers do not write `.runs/**`, `PLAN.md`, or this controller.
6. Workers do not create new helper abstractions that hide whether work is control-only or prompt-bearing.
7. Workers keep changes small, phase-local, and ASCII unless the file already requires otherwise.
8. Workers report blockers immediately instead of speculating around locked decisions.
9. Parent resets context at each gate by writing accepted scope and current branch tip into the next task artifact.
10. If a worker observes drift from `PLAN.md`, it returns blocked status instead of silently absorbing extra scope.
11. If a worker receives a `HIGH` or `CRITICAL` GitNexus impact result, it stops, records the blast radius, and waits for explicit parent direction.

## Worker Return Contract

Every worker returns all of the following to the parent:

1. branch name,
2. head SHA,
3. changed file list,
4. concise summary of what changed,
5. commands run,
6. exit codes,
7. tests run and results,
8. grep gates run and results,
9. GitNexus impact analysis summary for every edited symbol, including any elevated blast radius,
10. GitNexus `detect_changes` output for the lane branch,
11. unresolved blockers or uncertainties,
12. explicit statement that no out-of-scope files were edited.

Parent acceptance checklist for every worker:

1. compare changed files against lane ownership,
2. verify the branch tip is based on the expected authoritative parent tip,
3. confirm GitNexus impact analysis was run before every edited symbol,
4. escalate and explicitly accept or reject any `HIGH` or `CRITICAL` blast radius before merge,
5. review for contract drift,
6. transcribe `impact-analysis-summary.md` and `gitnexus-detect-changes.txt` into the task artifact root,
7. review GitNexus `detect_changes` output for lane-scope drift,
8. record artifacts in `.runs/**`,
9. merge or quarantine,
10. update gate state.

## Validation Commands

### A0 Dependency Floor Gates

```bash
rg -n "^\\[patch\\.crates-io\\]|unified-agent-api = \\{ path = " \
  /home/azureuser/__Active_Code/atomize-hq/substrate/Cargo.toml

rg -n "unified-agent-api|path\\+file:.*/unified-agent-api/crates/agent_api" \
  /home/azureuser/__Active_Code/atomize-hq/substrate/Cargo.lock

cargo tree -p shell | rg "unified-agent-api"
```

Expected result:

1. no local patch override remains,
2. no lockfile entry points at the neighboring checkout,
3. the dependency still resolves.

### A2-A4 Static Control-Semantics Gates

```bash
rg -n 'prompt: ""|NoPromptRecovery' \
  /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution \
  /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl \
  /home/azureuser/__Active_Code/atomize-hq/substrate/crates/world-service/src

rg -n 'agent_api.session.resume.v1' \
  /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution \
  /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl

rg -n 'continuity_uaa_session_id|parked_resumable|attached_participant_id' \
  /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs \
  /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime
```

Expected result:

1. no production `prompt: ""` or `NoPromptRecovery` path backs public `reattach` or `fork`,
2. no public `fork` path routes through `agent_api.session.resume.v1`,
3. successor normalization surfaces are explicit in the merged implementation.

### Focused Test Gates

Parent-resolved focused unit selectors:

1. Resolve the exact focused unit selectors for `state_store.rs` attach-contract persistence coverage against the current tree at execution time.
2. Resolve the exact focused unit selectors for `async_repl.rs` launch-path split coverage against the current tree at execution time.
3. Record the exact selector strings the parent chose in the relevant task `commands.txt` before those checks run.
4. Do not present placeholder filter strings as canonical if the repo's current test names differ.

Candidate command shape only, to be refined by the parent at execution time:

```bash
cargo test -p shell <parent-resolved-state-store-selector> -- --nocapture
cargo test -p shell <parent-resolved-async-repl-selector> -- --nocapture
```

Locked named suites from `PLAN.md`:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-service -- --nocapture
```

### Full Workspace Gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

### Manual Proof Points

Parent closeout must also record operator-legible proof for all of the following:

1. host `start` still submits the real first prompt,
2. host `turn` still submits the real follow-up prompt,
3. `reattach` restores the same durable session without submitting a prompt,
4. `reattach` fails closed when continuity is absent,
5. `fork` allocates a new durable session without submitting a prompt,
6. the fork result is honest about parked successor posture,
7. the successor does not retain the parent's continuity token,
8. the successor preserves source lineage truth,
9. the durable attach contract exists after host session birth and survives detach,
10. detached-world follow-up still fails closed until host ownership returns,
11. downstream slices 29/30/31 no longer describe blank-prompt control semantics as live architecture.

## Tests And Acceptance

The acceptance wall is organized by area and each area must go green on the same merged tree:

1. Dependency floor:
   - patch override removed,
   - lockfile no longer points at the neighboring checkout,
   - dependency resolution still works.
2. Launch-path split:
   - focused `async_repl.rs` and related launch-path coverage proves control-only attach, prompt-bearing turn launch, and successor allocation are separated.
3. Reattach control-only truth:
   - `agent_public_control_surface_v1` proves no-prompt reattach and fail-closed continuity behavior.
4. Fork successor truth:
   - `agent_successor_contract_ahcsitc0` plus related control checks prove successor normalization, honest posture, and no borrowed continuity token.
5. Prompt-bearing invariants:
   - `start`, `turn`, and world-member follow-up remain prompt-bearing and detached-world follow-up remains fail-closed.
6. Docs and downstream sync:
   - `docs/USAGE.md`, truth docs, ADR-0047, and slices 29/30/31 match the landed runtime architecture.
7. Workspace health:
   - grep gates, focused suites, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --nocapture` all pass.

## Acceptance Criteria

The controller may close only if all of the following are true on the merged authoritative tree:

1. no `[patch.crates-io]` override remains,
2. no live Substrate path treats public `reattach` as blank-prompt resume,
3. no live Substrate path treats public `fork` as blank-prompt resume,
4. runtime code clearly separates control-only attach, prompt-bearing turn launch, and successor allocation,
5. `OrchestrationSessionRecord` persists the host attach contract,
6. successor copies attach-contract shape but clears `continuity_uaa_session_id`,
7. successor posture is `parked_resumable`,
8. successor `attached_participant_id` is `null`,
9. successor preserves source lineage truth,
10. `start` and `turn` remain prompt-bearing only,
11. detached-world follow-up remains fail-closed,
12. no new public verb, schema, or policy surface exists,
13. docs and downstream slices 29/30/31 tell the same story as the code,
14. every edited symbol had GitNexus impact analysis recorded and any `HIGH` or `CRITICAL` result was explicitly parent-approved or blocked,
15. the parent has reviewed worker `detect_changes` outputs and run a final merged-tree `detect_changes` pass before acceptance,
16. focused tests, manual proof points, and full workspace gates pass,
17. the parent has written closeout artifacts and `RUN_COMPLETE`.

## Assumptions

1. `PLAN.md` remains the authoritative source for this slice throughout the run.
2. The authoritative branch remains `feat/gateway-mediated-llm-fulfillment`.
3. Published `unified-agent-api = "=0.3.5"` remains available during A0.
4. Current paths under `/home/azureuser/__Active_Code/atomize-hq` remain stable for worktree creation.
5. The doc and downstream packet paths named in `PLAN.md` still exist at launch time.
6. The parent can refuse optional A4 parallelism and still satisfy schedule and scope.
7. No follow-up slice work from 29, 30, or 31 is pulled into this controller beyond truth-sync wording.
