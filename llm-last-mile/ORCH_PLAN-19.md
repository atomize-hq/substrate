# ORCH_PLAN-19: Execute PLAN-19 Through A Parent-Frozen Owner Plane, Exact Session Selectors, And Linux-First Public Control Productization

Branch: `feat/session-centric-state-store`  
Plan source: [PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-19.md)  
Style reference: [ORCH_PLAN-18.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-18.md)  
Structure reference: [M26 Orchestration Plan](/Users/spensermcconnell/__Active_Code/atomize-hq/spec/docs/m26_orchestration_kickoff_prompt.md)  
Packet index: [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Execution type: fresh orchestration plan, Linux-first, public control-plane productization, parent-frozen contract, parent-only integration and approval  
Worker model: GPT-5.4 workers with `reasoning_effort=high`  
Max concurrent code workers before integration: `2`

## Summary

This document is the execution controller for `PLAN-19`, not a restatement of it.

- Execute from the current branch `feat/session-centric-state-store`, because that is the authoritative integration checkout in this workspace.
- Keep the critical path local to the parent agent for contract freeze, owner-model freeze, worker launch gates, both integration windows, the validation wall, and final closeout.
- Use dedicated worker worktrees under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/`.
- Use GPT-5.4 with `reasoning_effort=high` for all workers and cap true code concurrency at `2`.
- Keep orchestration state in one local parent-owned source of truth:
  - queue and task ledger: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/tasks.json`
  - run state: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/run-state.json`
  - session log: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/session-log.md`
  - sentinels: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/`

The run shape is frozen up front:

1. `task/m19-p1-parent-contract-freeze-owner-model-and-run-init` is parent-only and freezes the public verb contract, exact-selector contract, hidden owner-helper contract, private stop transport contract, strict-vs-permissive surface split, Linux-first posture, lane ownership, merge order, retry budget, stop conditions, and validation wall.
2. `task/m19-g1-implementation-window-a-launch-gate` is parent-only and is the only gate that may launch the first implementation window.
3. The only honest initial parallel window is exactly two lanes:
   - `task/m19-l1-strict-resolver-and-naming-guardrails`
   - `task/m19-l2-owner-plane-and-repl-integration`
4. `task/m19-g2-window-a-integration-gate` and `task/m19-p2-parent-window-a-integration` are parent-only.
5. Public CLI wiring is downstream and does not start until the accepted resolver and owner-plane truth exist:
   - `task/m19-g3-cli-lane-launch-gate`
   - `task/m19-l3-public-cli-and-handler-wiring`
   - `task/m19-g4-cli-integration-gate`
   - `task/m19-p3-parent-cli-integration`
6. Tests/docs closeout is not parallel with code lanes. It starts only after `p3` lands merged code truth:
   - `task/m19-g5-closeout-launch-gate`
   - `task/m19-l4-tests-docs-gap-matrix-and-qa-closeout`
7. `task/m19-g6-validation-wall-gate` and `task/m19-p4-parent-validation-wall-and-closeout` are parent-only and finish the run.

Canonical task IDs:

- `task/m19-p1-parent-contract-freeze-owner-model-and-run-init`
- `task/m19-g1-implementation-window-a-launch-gate`
- `task/m19-l1-strict-resolver-and-naming-guardrails`
- `task/m19-l2-owner-plane-and-repl-integration`
- `task/m19-g2-window-a-integration-gate`
- `task/m19-p2-parent-window-a-integration`
- `task/m19-g3-cli-lane-launch-gate`
- `task/m19-l3-public-cli-and-handler-wiring`
- `task/m19-g4-cli-integration-gate`
- `task/m19-p3-parent-cli-integration`
- `task/m19-g5-closeout-launch-gate`
- `task/m19-l4-tests-docs-gap-matrix-and-qa-closeout`
- `task/m19-g6-validation-wall-gate`
- `task/m19-p4-parent-validation-wall-and-closeout`

## Hard Guards

These are run-stopping invariants, not preferences:

1. The authoritative integration checkout remains the current workspace checkout on `feat/session-centric-state-store`.
2. The parent agent is the only integrator, the only approval authority, and the only writer of run-state artifacts under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/`.
3. Public verbs are frozen to exactly:
   - `substrate agent start --backend <backend_id> [--json]`
   - `substrate agent resume --session <orchestration_session_id> [--json]`
   - `substrate agent fork --session <orchestration_session_id> [--json]`
   - `substrate agent stop --session <orchestration_session_id> [--json]`
4. Public existing-session targeting accepts only `--session <orchestration_session_id>`.
5. Public root-session creation accepts only `--backend <backend_id>`.
6. No public command accepts `participant_id`, `session_handle_id`, `active_session_handle_id`, or `internal.uaa_session_id` as input.
7. `start`, `resume`, and `fork` are short-lived launch commands and must return only after authoritative readiness is visible in the state store.
8. `stop` must reach the live owner loop through the private owner transport and must not fake success through `session.json` mutation, toolbox mutation, or PID-only signaling.
9. The only new ownership surface allowed in this run is one per-session owner loop plus one private per-session stop transport. No general daemon, no shared multi-session broker, no global listener.
10. Root public session creation is host-orchestrator only in v1. World-scoped root start is an explicit rejection path.
11. World-sensitive reuse and stop posture is Linux-first. If exact posture cannot be proven on the current platform, return `unsupported_platform_or_posture`.
12. `status` remains the only permissive public surface in this family. `doctor`, `toolbox status`, `toolbox env`, `start`, `resume`, `fork`, and `stop` remain strict and fail closed.
13. `resume` stays in the same parent orchestration session. `fork` allocates a new parent orchestration session.
14. Public success and failure output is frozen before worker launch. Lane workers may not invent new fields or vague error names.
15. No worker may widen into prompt submission, `substrate -c` redesign, member-level public selectors, root world-session start, toolbox mutation tools, macOS/Lima parity, or Windows/WSL parity.
16. No worker may edit `.runs/**`.
17. If the frozen owner-helper and stop-transport contract cannot be expressed without broadening scope, the run stops and writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/blocked.json`.

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/strict-resolver-and-naming-guardrails`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/owner-plane-and-repl-integration`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/public-cli-and-handler-wiring`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/tests-docs-gap-matrix-and-qa-closeout`

Worker branches:

- `codex/feat-session-centric-state-store-m19-strict-resolver-and-naming-guardrails`
- `codex/feat-session-centric-state-store-m19-owner-plane-and-repl-integration`
- `codex/feat-session-centric-state-store-m19-public-cli-and-handler-wiring`
- `codex/feat-session-centric-state-store-m19-tests-docs-gap-matrix-and-qa-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/strict-resolver-and-naming-guardrails -b codex/feat-session-centric-state-store-m19-strict-resolver-and-naming-guardrails feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/owner-plane-and-repl-integration -b codex/feat-session-centric-state-store-m19-owner-plane-and-repl-integration feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/public-cli-and-handler-wiring -b codex/feat-session-centric-state-store-m19-public-cli-and-handler-wiring feat/session-centric-state-store
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-19/tests-docs-gap-matrix-and-qa-closeout -b codex/feat-session-centric-state-store-m19-tests-docs-gap-matrix-and-qa-closeout feat/session-centric-state-store
```

Parent integration surface:

- The parent integrates only on `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/session-centric-state-store`.
- No separate parent integration worktree is introduced.

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/`:

- `run-state.json`
- `tasks.json`
- `session-log.md`
- `contract-freeze.json`
- `owner-model-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `validation-wall.md`
- `blocked.json` on failure only
- `quarantine/`
- `sentinels/`

Canonical per-task artifact roots:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-p1-parent-contract-freeze-owner-model-and-run-init/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-g1-implementation-window-a-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-l1-strict-resolver-and-naming-guardrails/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-l2-owner-plane-and-repl-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-g2-window-a-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-p2-parent-window-a-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-g3-cli-lane-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-l3-public-cli-and-handler-wiring/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-g4-cli-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-p3-parent-cli-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-g5-closeout-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-l4-tests-docs-gap-matrix-and-qa-closeout/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-g6-validation-wall-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-p4-parent-validation-wall-and-closeout/`

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

Sentinel convention:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/<task-id>.ok`

`contract-freeze.json` is the single source of truth for:

- frozen public verb list
- success JSON fields
- failure taxonomy
- strict-vs-permissive surface split
- exact selector rules
- host-root-only start rule
- Linux-first world-sensitive rejection rule
- no-prompt-surface rule
- validation commands
- stop conditions

`owner-model-freeze.json` is the single source of truth for:

- hidden owner-helper modes: `start`, `resume`, `fork`
- required helper inputs
- per-session private stop transport contract
- `{"version":1,"action":"stop"}` request shape
- accepted stop outcomes: `accepted`, `already_terminal`, `owner_unreachable`, `protocol_error`
- readiness wait requirements
- parent-session state machine
- REPL-owned and helper-owned owner-plane parity rule

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m19-p1-parent-contract-freeze-owner-model-and-run-init` | parent | — | authoritative checkout | frozen contract, owner-model freeze, run artifacts, seeded worker basis |
| `task/m19-g1-implementation-window-a-launch-gate` | parent | `p1` | authoritative checkout | launch approval for Window A |
| `task/m19-l1-strict-resolver-and-naming-guardrails` | worker | `g1` | `strict-resolver-and-naming-guardrails` / `codex/feat-session-centric-state-store-m19-strict-resolver-and-naming-guardrails` | exact public-control resolver and canonical naming guardrails |
| `task/m19-l2-owner-plane-and-repl-integration` | worker | `g1` | `owner-plane-and-repl-integration` / `codex/feat-session-centric-state-store-m19-owner-plane-and-repl-integration` | shared control module, hidden owner-helper, private stop transport, REPL parity |
| `task/m19-g2-window-a-integration-gate` | parent | `l1`, `l2` | authoritative checkout | acceptance or quarantine decision for Window A |
| `task/m19-p2-parent-window-a-integration` | parent | `g2` | authoritative checkout | merged resolver plus owner-plane truth |
| `task/m19-g3-cli-lane-launch-gate` | parent | `p2` | authoritative checkout | launch approval for CLI lane |
| `task/m19-l3-public-cli-and-handler-wiring` | worker | `g3` | `public-cli-and-handler-wiring` / `codex/feat-session-centric-state-store-m19-public-cli-and-handler-wiring` | public CLI surface and strict command handling |
| `task/m19-g4-cli-integration-gate` | parent | `l3` | authoritative checkout | acceptance or quarantine decision for CLI lane |
| `task/m19-p3-parent-cli-integration` | parent | `g4` | authoritative checkout | merged public command surface |
| `task/m19-g5-closeout-launch-gate` | parent | `p3` | authoritative checkout | launch approval for tests/docs closeout |
| `task/m19-l4-tests-docs-gap-matrix-and-qa-closeout` | worker | `g5` | `tests-docs-gap-matrix-and-qa-closeout` / `codex/feat-session-centric-state-store-m19-tests-docs-gap-matrix-and-qa-closeout` | public-control integration tests, doc truth, QA artifact |
| `task/m19-g6-validation-wall-gate` | parent | `l4` | authoritative checkout | permission to run final command wall |
| `task/m19-p4-parent-validation-wall-and-closeout` | parent | `g6` | authoritative checkout | final validation, closeout, terminal state |

## Merge Order

`merge-order.json` is frozen during `p1` and governs integration behavior.

It must state:

1. `L1` integrates before `L2` acceptance is finalized if `L2` consumed resolver seams.
2. `L2` is replayed on top of the accepted `L1` tree before acceptance.
3. `L3` starts only after accepted `L1` and accepted `L2` are integrated.
4. `L4` starts only after accepted `L3` is integrated.
5. The parent never hand-merges a hybrid owner model out of conflicting worker assumptions.

Operational rule:

- If `L2` assumes a different exact resolver contract than the accepted `L1` tree, quarantine `L2`.
- If `L3` invents output fields, selector behavior, or failure semantics beyond the frozen contract, quarantine `L3`.

## Kickoff Initialization Order

The parent initializes the run in this exact order:

1. Create `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/`, `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/`, `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/quarantine/`, and every `.runs/task-m19-*/` directory.
2. Inside each `.runs/task-m19-*/` directory, create `task.json`, `commands.txt`, `summary.md`, and `artifacts/`.
3. For each gate task, also create `gate-checklist.md` and `gate-result.json`.
4. For each worker task, also create placeholders for `worker-report.md`, `worker-output.patch`, and `evidence-manifest.json`.
5. Write `tasks.json` as the canonical launch queue and execution ledger for the whole run.
6. Write `run-state.json` with `current_phase: "kickoff"`, `worker_cap: 2`, every task in `pending`, and empty accepted, rejected, quarantined, and blocked arrays.
7. Write `contract-freeze.json`, `owner-model-freeze.json`, `lane-ownership.json`, `merge-order.json`, and `validation-wall.md`.
8. Review the frozen hotspots:
   - [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
   - [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
   - [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   - [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
   - [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   - [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)
   - [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
9. If needed, take one narrow compile-first scaffold before worker launch in:
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs`
10. The allowed scaffold may freeze signatures, types, and internal entrypoints only. It must not weaken strict selector behavior, must not expose public help, and must be recorded in both freeze artifacts.
11. Seed worker worktrees only after the above artifacts and any allowed scaffold are in place.
12. Write `session-log.md` with kickoff timestamp, authoritative branch, worktree roots, worker cap, and the explicit statement that the only honest initial parallel window is `L1` plus `L2`.

## Lane Ownership By File Set

| Lane | Allowed files | Forbidden escalation surfaces |
| --- | --- | --- |
| `L1` / strict resolver | [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs), [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | `cli.rs`, `agents_cmd.rs`, `async_repl.rs`, docs, `.runs/**`, public output schema changes |
| `L2` / owner plane | `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs`, [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs), [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | `cli.rs`, `agents_cmd.rs`, docs, `.runs/**`, toolbox mutation paths, global daemon work |
| `L3` / public CLI | [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs), [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | runtime storage files, `control.rs`, tests, docs, `.runs/**`, prompt-taking surfaces |
| `L4` / tests docs closeout | `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs`, [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) | all production Rust files, `.runs/**`, overclaiming docs |

## Workstream Plan

### `task/m19-p1-parent-contract-freeze-owner-model-and-run-init`

Owner:

- parent only

Scope:

1. Freeze the public verb contract, exact-selector contract, success JSON fields, and failure taxonomy.
2. Freeze the owner model:
   - hidden owner-helper is internal only
   - modes are `start`, `resume`, and `fork`
   - one private per-session stop transport exists
   - REPL-owned and helper-owned live sessions both register the same stop transport contract
3. Freeze the parent-session state machine and readiness rules.
4. Freeze the strict-vs-permissive split:
   - permissive: `substrate agent status`
   - strict: `doctor`, `toolbox status`, `toolbox env`, `start`, `resume`, `fork`, `stop`
5. Freeze lane ownership, merge order, retry budget, and stop conditions.
6. If needed, take the narrow compile-first scaffold described above and reseed workers from that exact commit.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

Acceptance:

1. `contract-freeze.json`, `owner-model-freeze.json`, `lane-ownership.json`, `merge-order.json`, `tasks.json`, and `run-state.json` exist.
2. The freeze artifacts record the hidden owner-helper, private stop transport, exact selector rules, Linux-first posture, and strict-vs-permissive split explicitly.
3. Any allowed scaffold is signature-first only and compile-clean.
4. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-p1-parent-contract-freeze-owner-model-and-run-init.ok`.

### `task/m19-g1-implementation-window-a-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. `L1` and `L2` worktrees were seeded from the exact same post-`p1` tree.
3. Worker prompts name only owned files, forbidden surfaces, command gates, retry budget, and sentinel paths.
4. `L1` prompt explicitly forbids status degradation, trace fallback selection, or non-canonical selectors.
5. `L2` prompt explicitly forbids global-daemon work, toolbox mutation, shared multi-session broker work, or public CLI exposure.

Acceptance:

1. No worker starts before this gate is green.
2. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-g1-implementation-window-a-launch-gate.ok`.

### `task/m19-l1-strict-resolver-and-naming-guardrails`

Owner:

- single worker on `codex/feat-session-centric-state-store-m19-strict-resolver-and-naming-guardrails`

Scope:

1. Add the exact public-control resolver on authoritative state-store reads.
2. Enforce exact `orchestration_session_id` resolution for `resume`, `fork`, and `stop`.
3. Reject `participant_id`, `session_handle_id`, and `internal.uaa_session_id` as public selectors.
4. Require exact active participant linkage where action semantics require it.
5. Require `internal.uaa_session_id` presence for `resume` and `fork`.
6. Preserve `orchestration_session_id` as the only public parent handle and keep compatibility alias reads intact.
7. Keep `status` degradation logic untouched and fail closed for public control actions.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

Acceptance:

1. The lane touches only its owned files.
2. Resolver behavior is exact and never consults fuzzy or trace-history selectors.
3. Canonical naming remains `orchestration_session_id` public, `participant_id` lineage/debug, `internal.uaa_session_id` internal only.
4. Non-Linux or unprovable world-sensitive posture yields `unsupported_platform_or_posture`.
5. The worker writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-l1-strict-resolver-and-naming-guardrails.ok`.

### `task/m19-l2-owner-plane-and-repl-integration`

Owner:

- single worker on `codex/feat-session-centric-state-store-m19-owner-plane-and-repl-integration`

Scope:

1. Extract the shared control module under `crates/shell/src/execution/agent_runtime/control.rs`.
2. Move shared host lifecycle logic out of `async_repl.rs` into the shared control module.
3. Add exact resume and fork extension shaping with the existing UAA selector grammar.
4. Add the hidden owner-helper entrypoint and internal execution plan handoff.
5. Add one private per-session stop transport with exactly one v1 mutation request:
   - `{"version":1,"action":"stop"}`
6. Ensure REPL-owned sessions register the same stop transport that helper-owned sessions register.
7. Preserve authoritative shutdown through the existing owner path. Do not add JSON mutation shortcuts.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell async_repl --no-run
```

Acceptance:

1. The lane touches only its owned files.
2. No general daemon or global listener is introduced.
3. The owner-helper stays hidden from public help and consumes resolved internal inputs only.
4. Stop transport is per-session, exact, private, and owner-mediated.
5. REPL startup, targeted-turn resume wiring, and shutdown semantics remain green in principle and testable.
6. The worker writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-l2-owner-plane-and-repl-integration.ok`.

### `task/m19-g2-window-a-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned patch, report, command transcript, and evidence manifest.
2. Every touched file is inside the lane’s ownership boundary.
3. `L1` landed the exact resolver and naming contract without reopening status semantics.
4. `L2` landed the owner plane without introducing a global broker, toolbox mutation, or public exposure.
5. `L2` is not accepted until the parent proves it replays cleanly against the accepted `L1` tree if it consumed resolver seams.

Acceptance:

1. Accepted, rejected, or quarantined status for both Window A lanes is recorded in `run-state.json`.
2. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-g2-window-a-integration-gate.ok`.

### `task/m19-p2-parent-window-a-integration`

Owner:

- parent only

Scope:

1. Integrate accepted `L1` output first.
2. Re-run `L1` command gates on the authoritative checkout.
3. Replay `L2` on top of accepted `L1`. If `L2` assumed a different resolver shape or naming contract, quarantine `L2` instead of hand-editing around it.
4. Integrate accepted `L2` output second.
5. Re-run combined Window A gates on the authoritative checkout.
6. Freeze the merged resolver plus owner-plane truth before CLI work begins.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell async_repl --no-run
```

Acceptance:

1. The parent remains the sole integrator.
2. The authoritative tree now contains the exact resolver, canonical naming guardrails, shared control module, hidden owner-helper, and private stop transport contract.
3. No hybrid truth was invented during integration.
4. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-p2-parent-window-a-integration.ok`.

### `task/m19-g3-cli-lane-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is green.
2. The CLI worktree is reseeded or rebased to the exact post-`p2` tree.
3. The worker prompt names only `cli.rs` and `agents_cmd.rs`.
4. The prompt explicitly freezes:
   - public verb names
   - success JSON fields
   - failure taxonomy
   - no prompt-taking surface
   - host-root-only start
   - exact `--session` selector semantics

Acceptance:

1. No CLI worker starts before this gate is green.
2. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-g3-cli-lane-launch-gate.ok`.

### `task/m19-l3-public-cli-and-handler-wiring`

Owner:

- single worker on `codex/feat-session-centric-state-store-m19-public-cli-and-handler-wiring`

Scope:

1. Add public `Start`, `Resume`, `Fork`, and `Stop` actions plus exact arg structs.
2. Add the hidden owner-helper entrypoint wiring without exposing it in public help.
3. Wire `agents_cmd.rs` to the accepted control module and accepted resolver behavior only.
4. Emit stable JSON and deterministic text output.
5. Keep `start` host-backend exact and reject world-scoped root start explicitly.
6. Keep `resume` same-parent and `fork` new-parent semantics explicit.
7. Keep `stop` waiting for terminal parent state, not mere request acceptance.

Command gates:

```bash
cargo fmt --all -- --check
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 --no-run
```

Acceptance:

1. The lane touches only its owned files.
2. Public help exposes only the four frozen public verbs.
3. No public selector broadening occurs.
4. No prompt-taking public surface is added.
5. Output fields and error names match the frozen contract exactly.
6. The worker writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-l3-public-cli-and-handler-wiring.ok`.

### `task/m19-g4-cli-integration-gate`

Owner:

- parent only

Checks:

1. `L3` returned a patch, report, command transcript, and evidence manifest.
2. Every touched file is inside `L3` ownership.
3. `L3` consumed the accepted control-module and resolver truth and did not invent branch-local semantics.
4. `L3` did not widen into prompt submission, `substrate -c`, or member-level selectors.

Acceptance:

1. Accepted, rejected, or quarantined status for the CLI lane is recorded in `run-state.json`.
2. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-g4-cli-integration-gate.ok`.

### `task/m19-p3-parent-cli-integration`

Owner:

- parent only

Scope:

1. Integrate accepted `L3` output onto the authoritative checkout.
2. Re-run `L3` command gates on the authoritative checkout.
3. Freeze the merged public command surface before tests/docs begin.

Command gates:

```bash
cargo fmt --all -- --check
cargo test -p shell --lib -- --nocapture
```

Acceptance:

1. The authoritative tree now contains the full public command surface.
2. The merged tree still reflects the exact-selector contract, strict-vs-permissive split, Linux-first posture, and owner-plane contract.
3. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-p3-parent-cli-integration.ok`.

### `task/m19-g5-closeout-launch-gate`

Owner:

- parent only

Checks:

1. `p3` is green.
2. The closeout worktree is reseeded or rebased to the exact post-`p3` tree.
3. The worker prompt names only the allowed test and doc files.
4. The worker prompt explicitly forbids reopening production runtime files.

Acceptance:

1. No tests/docs worker starts before this gate is green.
2. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-g5-closeout-launch-gate.ok`.

### `task/m19-l4-tests-docs-gap-matrix-and-qa-closeout`

Owner:

- single worker on `codex/feat-session-centric-state-store-m19-tests-docs-gap-matrix-and-qa-closeout`

Scope:

1. Add or complete `agent_public_control_surface_v1.rs` as the dedicated public-control integration suite.
2. Extend `agent_successor_contract_ahcsitc0.rs` only for non-regression proof where the public-control slice touches prior status and lineage truth.
3. Cover:
   - public host `start`
   - public `resume` of an orphaned session
   - public `fork` creating a new orchestration session
   - public `stop` through the owner transport
   - selector rejection for wrong handle types
   - world-sensitive rejection on unsupported platform or posture
   - REPL-owned session stoppability through the same private owner plane
4. Update the gap matrix to mark only the shipped host-orchestrator control surface as landed.
5. Update the packet README index if needed so `PLAN-19` and `ORCH_PLAN-19` are discoverable.
6. Write the QA-facing artifact to the frozen pattern:
   - `~/.gstack/projects/<slug>/<user>-feat-session-centric-state-store-eng-review-test-plan-<timestamp>.md`

Command gates:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
rg -n "start|resume|fork|stop|orchestration_session_id|participant_id|Linux-first" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md
```

Acceptance:

1. The lane touches only its owned files.
2. Tests prove all four public verbs and the rejection matrix directly.
3. Docs reflect host-orchestrator public control landed, Linux-first world-sensitive posture, and no prompt-taking caller surface.
4. The gap matrix does not overclaim world-root start, macOS parity, Windows parity, or toolbox mutation.
5. The worker writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-l4-tests-docs-gap-matrix-and-qa-closeout.ok`.

### `task/m19-g6-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L4` returned and is classified.
2. `L4` is accepted.
3. No quarantined or blocked output remains unresolved.
4. `validation-wall.md` names the exact final command order and manual spot checks.
5. The parent can enumerate every `PLAN-19` definition-of-done clause and the command or artifact that proves it.

Acceptance:

1. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-g6-validation-wall-gate.ok`.
2. The validation wall is permitted to run exactly once on the final merged tree.

### `task/m19-p4-parent-validation-wall-and-closeout`

Owner:

- parent only

Scope:

1. Integrate only accepted `L4` output.
2. Run the full validation wall in exact order.
3. Record final command results and artifact paths in the task artifact directory.
4. Confirm the gap matrix and README match the validated runtime truth.
5. Mark the run complete only if the validation wall proves owner-plane honesty, exact selectors, Linux-first fail-closed posture, and strict-vs-permissive surface discipline.

Required final artifacts under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m19-p4-parent-validation-wall-and-closeout/artifacts/`:

- `fmt.txt`
- `clippy.txt`
- `shell-lib-tests.txt`
- `agent-public-control-surface-v1.txt`
- `agent-successor-contract.txt`
- `async-repl-tests.txt`
- `agent-start-spot-check.txt`
- `agent-resume-spot-check.txt`
- `agent-fork-spot-check.txt`
- `agent-stop-spot-check.txt`
- `agent-status-spot-check.txt`
- `agent-doctor-spot-check.txt`
- `agent-toolbox-status-spot-check.txt`
- `agent-toolbox-env-spot-check.txt`
- `contract-audit.md`
- `closeout.md`

Acceptance:

1. All validation commands succeed on the authoritative checkout.
2. Manual spot checks are captured with expected operator-visible outcomes.
3. `run-state.json` records a successful terminal state.
4. The parent writes `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/sentinels/task-m19-p4-parent-validation-wall-and-closeout.ok`.

## Quarantine, Retry, And Blocked-Run Posture

1. Each worker lane has retry budget `1`.
2. Retry is allowed only for lane-local defects inside owned files.
3. Non-retryable violations include:
   - public selector broadening
   - status-style degradation on strict control surfaces
   - world-root public start
   - JSON mutation or toolbox mutation pretending to be stop
   - hidden owner-helper becoming public help
   - global daemon or shared broker introduction
   - macOS/Lima or Windows/WSL parity broadening
   - prompt-taking public surface addition
4. If `L1` cannot keep exact selector discipline or canonical naming, quarantine it immediately.
5. If `L2` can only work by adding a general daemon or global listener, quarantine it immediately.
6. If `L3` can only pass by inventing output fields or fuzzy selection, quarantine it immediately.
7. If `L4` can only make docs pass by overclaiming landed behavior, reject `L4` and stop the run.

Blocked termination minimum contents:

1. task or gate where execution stopped
2. classification: `rejected`, `blocked`, `quarantined`, or `merge_refused`
3. exact contract clause or ownership rule that stopped the run
4. whether retry remained available
5. artifact paths for patch, report, command output, and semantic-drift evidence
6. explicit statement that no blocked output was integrated

## Context-Control Rules

1. The parent keeps only a bounded live context:
   - [PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-19.md)
   - this orchestration plan
   - `tasks.json`
   - `run-state.json`
   - `contract-freeze.json`
   - `owner-model-freeze.json`
   - `merge-order.json`
   - latest integration diff summary
2. Worker prompts contain only:
   - owned file set
   - exact frozen contract excerpts relevant to that lane
   - required commands
   - forbidden touch surfaces
   - retry budget
   - sentinel path
3. Workers return summaries and artifacts only. They do not become independent approval or truth surfaces.
4. Workers do not write `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/*`.
5. The parent reviews summaries plus narrow diffs only and does not ingest full worker transcripts into the main run context.
6. Each worker is closed after accept, reject, or quarantine to keep context bounded.
7. No external human approval gate is assumed in this run. Every gate here is parent-only. If a true unresolved product decision appears, the run stops rather than inventing one.

## Tests And Acceptance

### A. Frozen-contract acceptance

- `contract-freeze.json` records the public verbs, exact selectors, success JSON fields, failure taxonomy, strict-vs-permissive split, and host-root-only start rule.
- `owner-model-freeze.json` records the hidden owner-helper modes, required inputs, private stop transport request and outcome contract, readiness rules, and state machine.
- No worker prompt reopens those decisions.

### B. `L1` acceptance

- Exact `orchestration_session_id` resolution exists for public control.
- `participant_id`, `session_handle_id`, and `internal.uaa_session_id` are rejected as public selectors.
- `internal.uaa_session_id` remains readable internally for exact resume and fork.
- Linux-first world-sensitive rejection is explicit and fail closed.
- Status degradation semantics remain untouched.

### C. `L2` acceptance

- Shared lifecycle logic is extracted to `control.rs`.
- Hidden owner-helper exists only as an internal entrypoint.
- REPL-owned and helper-owned live sessions both register the same stop transport contract.
- Stop remains owner-mediated and authoritative.
- No general daemon, shared broker, or toolbox mutation plane exists.

### D. `L3` acceptance

- `substrate agent` exposes exactly `start`, `resume`, `fork`, and `stop`.
- `start` accepts only exact host `backend_id`.
- `resume`, `fork`, and `stop` accept only exact `orchestration_session_id`.
- JSON output is stable and scriptable.
- No prompt-taking public caller surface exists.

### E. `L4` acceptance

- The dedicated public-control integration suite proves success and failure paths for all four public verbs.
- Existing strict doctor/toolbox regressions remain green.
- REPL targeted-turn resume behavior remains green.
- Docs reflect only landed host-orchestrator public control with Linux-first world-sensitive posture.
- The QA-facing artifact exists.

### F. Integration acceptance

- `L1` integrates first.
- `L2` is replayed on top of accepted `L1` truth before acceptance if needed.
- `L3` starts only after accepted `L1` and `L2` are integrated.
- No parent-only hybrid edits are required to explain merged behavior.
- No quarantined or blocked output is partially integrated.

## Validation Wall

Parent-owned validation commands, executed only after `L4` is integrated:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell async_repl -- --nocapture
```

Manual spot checks after the command wall is green:

```bash
substrate agent start --backend <host_backend_id> --json
substrate agent resume --session <orchestration_session_id> --json
substrate agent fork --session <orchestration_session_id> --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent status --json
substrate agent doctor --json
substrate agent toolbox status --json
substrate agent toolbox env --json
```

Validation-wall requirements:

1. formatting and clippy stay green
2. shell library tests stay green
3. the dedicated public-control suite proves success and rejection behavior for all four verbs
4. targeted successor and REPL tests prove no regression in prior runtime truth
5. manual spot checks confirm the same operator contract the tests prove
6. docs do not overclaim beyond what the green runtime and tests prove

## Completion Criteria Tied To PLAN-19 Definition Of Done

The run is complete only if all of these are true:

1. `substrate agent` publicly exposes `start`, `resume`, `fork`, and `stop`.
2. `start` accepts only exact host-scoped `backend_id`.
3. `resume`, `fork`, and `stop` accept only exact `orchestration_session_id`.
4. No public command accepts or emits `internal.uaa_session_id` as a selector.
5. `start`, `resume`, and `fork` return only after authoritative readiness is visible in the store.
6. `stop` routes through the live owner and reaches a terminal parent-session state.
7. `resume` rebinds the same parent session and `fork` creates a new parent session.
8. REPL-owned live sessions expose the same private owner plane for public stop.
9. Strict `doctor` and `toolbox` behavior remain fail closed.
10. Root world-session start is rejected explicitly.
11. World-sensitive control remains Linux-first and fail closed elsewhere.
12. Repo-truth docs reflect landed behavior and nothing broader.

## Final State

Success requires all of:

1. every required sentinel exists
2. no blocked artifact exists under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/`
3. accepted outputs were integrated in the prescribed order only
4. final validation commands pass
5. manual spot checks confirm the same operator contract the tests prove
6. `contract-freeze.json`, `owner-model-freeze.json`, and the final merged tree still agree on the frozen `PLAN-19` contract

Blocked termination requires any of:

1. hard-guard violation
2. non-retryable rejection
3. exhausted retry budget
4. merge refusal with no legal redrive path
5. failed validation wall
6. docs requiring overclaim to appear complete

On blocked termination the parent must write:

1. `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/blocked.json`
2. terminal state and rationale in `run-state.json`
3. gate and failure summary in `session-log.md`
4. preserved evidence under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-19/quarantine/` and the relevant task artifact directory

## Assumptions

1. [PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-19.md) remains the authoritative dependency graph for this run.
2. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/session-centric-state-store`.
3. The core production hotspots are `control.rs`, `state_store.rs`, `async_repl.rs`, `cli.rs`, and `agents_cmd.rs`.
4. Linux-first public control posture is the only honest platform goal for this run.
5. Docs move last, after integrated code truth and green regressions exist.
6. The hidden owner-helper and private stop transport can be expressed without introducing a new daemon or new public mutation surface.
7. If the compile-first scaffold is unnecessary, `p1` records that explicitly and workers still branch from the exact same post-`p1` commit.
