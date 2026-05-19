# ORCH_PLAN-22: Execute PLAN-22 Through A Parent-Frozen Public Turn Hardening Contract, Linux World-Member Follow-Up Proof, And Explicit Fail-Closed Coverage

Live workspace branch: `feat/macos-lima-shared-owner-member-runtime-parity`  
Recorded branch in [PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md): `feat/broaden-caller-surfaces-from-repl`  
Authoritative execution branch for this run: `feat/macos-lima-shared-owner-member-runtime-parity`  
Plan source: [PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md)  
Source SOW: [22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/22-broaden-caller-surfaces-from-repl-first-to-public-session-member-turns.md)  
Style references: [ORCH_PLAN-20.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-20.md), [ORCH_PLAN-21.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/ORCH_PLAN-21.md)  
Packet index: [README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:107), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:112)  
Execution type: fresh orchestration controller for public caller-surface hardening, Linux-first world-member follow-up proof, parent-frozen contract, parent-only gates and integration, docs closeout only after merged code truth  
Live root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Max concurrent code workers before integration: `2`

## Summary

This document is the execution controller for `PLAN-22`, not a restatement of it.

The branch recorded inside `PLAN-22.md` is stale relative to the live workspace. The current checkout on `feat/macos-lima-shared-owner-member-runtime-parity` already contains later merged truth from the prompt-taking public surface and the macOS/Lima parity follow-on. Rewinding execution to `feat/broaden-caller-surfaces-from-repl` would force speculative backporting and would no longer represent repo truth. This run therefore treats the current live checkout as authoritative and records the `PLAN-22` branch as historical planning context only.

The slice is honest only if it lands all of these on the same merged tree:

- `substrate agent start` remains the canonical public root prompt-taking surface.
- `substrate agent turn` remains the canonical public follow-up surface.
- Linux world-member public follow-up is directly proven from exact selector resolution through typed `MemberTurnSubmitRequestV1` submission into `world-agent`.
- detached host follow-up recovery and detached world follow-up rejection are both explicit parts of the public contract.
- the fail-closed public-turn taxonomy is pinned by concrete tests rather than implied by implementation.
- `crates/shell/tests/repl_world_first_routing_v1.rs` remains an explicit non-regression gate for the already-landed REPL-first exact targeted follow-up contract.
- docs and gap-matrix truth are updated only after merged code and tests prove the shipped contract.

The only honest opening parallelism is `2` lanes:

1. shell public-turn contract hardening over the existing prompt-taking and selector seams
2. world-agent retained-member identity-drift proof hardening at the submit boundary

A third concurrent code lane would either collide with the same shell control-suite files or would start docs and operator truth before merged runtime behavior exists.

Frozen run shape:

1. `task/m22-p1-parent-contract-freeze-and-run-init`
2. `task/m22-g1-window-a-launch-gate`
3. parallel Window A
   - `task/m22-l1-shell-public-turn-contract-hardening`
   - `task/m22-l2-world-agent-retained-member-proof`
4. `task/m22-g2-window-a-integration-gate`
5. `task/m22-p2-parent-window-a-integration`
6. `task/m22-g3-closeout-launch-gate`
7. `task/m22-l3-public-surface-tests-docs-closeout`
8. `task/m22-g4-validation-wall-gate`
9. `task/m22-p3-parent-validation-wall`
10. `task/m22-p4-parent-closeout-phase`

## Hard Guards

These are run-stopping invariants, not preferences.

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/macos-lima-shared-owner-member-runtime-parity`.
2. The branch recorded in [PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md) is preserved only as historical plan metadata. No worker may switch the run back to `feat/broaden-caller-surfaces-from-repl`.
3. The parent agent is the only integrator, the only approval authority, and the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-22/**`.
4. `substrate agent start` remains the canonical public root prompt-taking surface.
5. `substrate agent turn` remains the canonical public follow-up prompt-taking surface.
6. Root `start` remains host-only in v1. Public world-root start stays fail-closed.
7. Public `turn` requires exact `--session <orchestration_session_id>` and exact `--backend <backend_id>`.
8. No fuzzy session routing, no fuzzy backend routing, no latest-session fallback, and no acceptance of `participant_id`, `active_session_handle_id`, `session_handle_id`, or `internal.uaa_session_id` as public inputs.
9. Linux world-sensitive public follow-up must continue to use `MemberTurnSubmitRequestV1` and `/v1/member_turn/stream`.
10. Detached host follow-up may recover only through the existing detached-host posture path. Detached world follow-up must fail closed until `substrate agent reattach --session ...` restores an active host owner.
11. `substrate -c`, `--command`, pipe mode, and plain stdin remain shell-wrap semantics. This run must not reinterpret them as agent prompting.
12. `crates/shell/tests/repl_world_first_routing_v1.rs` remains a frozen non-regression surface for the exact REPL targeted-turn contract and must stay green on the merged tree.
13. The public-turn fail-closed taxonomy is frozen and must be explicitly tested for:
   - `missing_backend`
   - `unknown_session`
   - `noncanonical_session_selector` via `active_session_handle_id`
   - `noncanonical_session_selector` via `participant_id` / legacy `session_handle_id`
   - `noncanonical_session_selector` via `internal.uaa_session_id`
   - `missing_active_parent`
   - `backend_not_in_session`
   - `stale_linkage`
   - `ambiguous_backend_slot`
   - `unsupported_platform_or_posture`
   - `owner_unreachable`
14. Docs closeout is late-only. No worker may edit docs or gap-matrix truth before merged code truth exists.
15. No worker may widen into default-agent routing, new public selector types, new prompt verbs, toolbox mutation, daemon work, or non-Linux parity claims.
16. No worker may edit `.runs/**`.
17. If the authoritative branch choice, exact selector contract, Linux member-turn endpoint, detached-world fail-closed posture, or REPL-first non-regression contract becomes disputed during implementation, the run stops and the parent writes `blocked.json`.

Stop the run, write `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-22/blocked.json`, and do not advance if any of these occur:

1. A lane requires execution on the older `feat/broaden-caller-surfaces-from-repl` branch instead of the live authoritative checkout.
2. A lane requires a new public selector surface beyond exact `(orchestration_session_id, backend_id)`.
3. A lane requires changing away from `MemberTurnSubmitRequestV1` or `/v1/member_turn/stream`.
4. A lane requires detached-world follow-up success without `reattach`.
5. A lane requires weakening or renaming the frozen failure taxonomy.
6. A lane requires changing `substrate -c` semantics.
7. A lane requires relaxing the exact REPL-first routing contract covered by `repl_world_first_routing_v1.rs`.
8. A worker touches files outside its frozen ownership surface.
9. `L3` starts before merged Window A truth exists.
10. The final validation wall cannot prove Linux world-member follow-up success, detached-world rejection, fail-closed taxonomy coverage, and REPL-first non-regression on the same merged tree.
11. Docs would need to claim world-root public start, default routing, Windows parity, or any unsupported non-Linux world-follow-up behavior.

### Blocked-Run Record Contract

`blocked.json` is parent-written only, exactly once, at the moment the parent decides the run cannot advance.

Required fields in `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-22/blocked.json`:

- `run_id`
- `authoritative_branch`
- `recorded_plan_branch`
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

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22/shell-public-turn-contract-hardening`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22/world-agent-retained-member-proof`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22/public-surface-tests-docs-closeout`

Worker branches:

- `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-shell-public-turn-contract-hardening`
- `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-world-agent-retained-member-proof`
- `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-public-surface-tests-docs-closeout`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22/shell-public-turn-contract-hardening \
  -b codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-shell-public-turn-contract-hardening \
  feat/macos-lima-shared-owner-member-runtime-parity

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22/world-agent-retained-member-proof \
  -b codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-world-agent-retained-member-proof \
  feat/macos-lima-shared-owner-member-runtime-parity

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-plan-22/public-surface-tests-docs-closeout \
  -b codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-public-surface-tests-docs-closeout \
  feat/macos-lima-shared-owner-member-runtime-parity
```

No separate parent integration worktree is introduced. The parent integrates only on `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`.

## Parent-Owned Run-State Surface

Canonical parent-owned state under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-22/`:

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

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-p1-parent-contract-freeze-and-run-init/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-g1-window-a-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-l1-shell-public-turn-contract-hardening/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-l2-world-agent-retained-member-proof/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-g2-window-a-integration-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-p2-parent-window-a-integration/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-g3-closeout-launch-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-l3-public-surface-tests-docs-closeout/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-g4-validation-wall-gate/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-p3-parent-validation-wall/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-p4-parent-closeout-phase/`

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

- `.runs/plan-22/sentinels/task-m22-p1-parent-contract-freeze-and-run-init.ok`
- `.runs/plan-22/sentinels/task-m22-g1-window-a-launch-gate.ok`
- `.runs/plan-22/sentinels/task-m22-l1-shell-public-turn-contract-hardening.ok`
- `.runs/plan-22/sentinels/task-m22-l2-world-agent-retained-member-proof.ok`
- `.runs/plan-22/sentinels/task-m22-g2-window-a-integration-gate.ok`
- `.runs/plan-22/sentinels/task-m22-p2-parent-window-a-integration.ok`
- `.runs/plan-22/sentinels/task-m22-g3-closeout-launch-gate.ok`
- `.runs/plan-22/sentinels/task-m22-l3-public-surface-tests-docs-closeout.ok`
- `.runs/plan-22/sentinels/task-m22-g4-validation-wall-gate.ok`
- `.runs/plan-22/sentinels/task-m22-p3-parent-validation-wall.ok`
- `.runs/plan-22/sentinels/task-m22-p4-parent-closeout-phase.ok`

`merge-order.json` is frozen during `p1` and must record:

- `authoritative_branch: "feat/macos-lima-shared-owner-member-runtime-parity"`
- `recorded_plan_branch: "feat/broaden-caller-surfaces-from-repl"`
- `integration_order: ["task/m22-l1-shell-public-turn-contract-hardening", "task/m22-l2-world-agent-retained-member-proof", "task/m22-l3-public-surface-tests-docs-closeout"]`
- `l2_acceptance_basis: "accepted_l1_tree_if_replay_needed_otherwise_original_post_p1_tree"`
- `l3_acceptance_basis: "accepted_p2_tree_only"`
- `replay_required_before_acceptance: true`
- `quarantine_on_branch_local_assumption: true`

## PLAN-22 Workstream Mapping

| PLAN-22 workstream | Orchestration tasks | Why this mapping is exact |
| --- | --- | --- |
| Tighten and prove shell-side exact public turn contract | `task/m22-l1-shell-public-turn-contract-hardening`, `task/m22-g2-window-a-integration-gate`, `task/m22-p2-parent-window-a-integration` | This lane owns the public `turn` posture logic, exact selector resolver seam, and shared prompt bridge contract in production code. |
| Widen retained-member drift and world-boundary proof | `task/m22-l2-world-agent-retained-member-proof`, `task/m22-g2-window-a-integration-gate`, `task/m22-p2-parent-window-a-integration` | This lane owns explicit retained-member identity-drift rejection proof at the `world-agent` submit boundary without changing the request schema. |
| Add explicit Linux world-follow-up proof, fail-closed coverage, REPL-first non-regression proof, and late docs truth | `task/m22-g3-closeout-launch-gate`, `task/m22-l3-public-surface-tests-docs-closeout`, `task/m22-g4-validation-wall-gate`, `task/m22-p3-parent-validation-wall`, `task/m22-p4-parent-closeout-phase` | This closeout lane owns the public control suite, `repl_world_first_routing_v1.rs` non-regression, Linux world-member follow-up proof, explicit fail-closed classifier coverage, detached host/world posture proof, eng-review test-plan artifact, and repo-truth docs after merged code truth exists. |

## Concurrency And Merge Order

Concurrency rules:

1. Worker cap is exactly `2` until `g2` completes.
2. `p1` must finish before any worker starts.
3. `g1` must be green before any worker starts.
4. The only honest initial parallel window is `L1` plus `L2`.
5. `L3` waits for accepted and integrated `L1` plus `L2`.
6. The validation wall runs exactly once on the final merged tree.
7. No docs work starts before `p2`.
8. No extra shell test lane is authorized because the decisive proof surfaces are one shared public control suite plus one shared REPL non-regression suite, not disjoint test islands.

Why `L1` integrates before `L2`:

1. `L1` freezes the dominant public-shell semantics that `L3` must prove: exact selector resolution, detached posture handling, and prompt-submit routing.
2. `L2` is intentionally narrower and should remain a boundary-proof lane at `world-agent`; it can replay cleanly if it accidentally assumed branch-local shell behavior.
3. `L3` depends much more on accepted shell public-contract truth than on branch-local world-agent assumptions.
4. Integrating `L2` first would not unblock `L3`; integrating `L1` first does.

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task/m22-p1-parent-contract-freeze-and-run-init` | parent | — | authoritative checkout | frozen contract, run artifacts, branch-resolution record |
| `task/m22-g1-window-a-launch-gate` | parent | `p1` | authoritative checkout | launch approval for `L1` and `L2` |
| `task/m22-l1-shell-public-turn-contract-hardening` | worker | `g1` | `shell-public-turn-contract-hardening` / `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-shell-public-turn-contract-hardening` | production shell hardening for public `turn` contract only |
| `task/m22-l2-world-agent-retained-member-proof` | worker | `g1` | `world-agent-retained-member-proof` / `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-world-agent-retained-member-proof` | explicit retained-member identity-drift proof at the world-agent boundary |
| `task/m22-g2-window-a-integration-gate` | parent | `l1`, `l2` | authoritative checkout | acceptance, rejection, or quarantine for Window A |
| `task/m22-p2-parent-window-a-integration` | parent | `g2` | authoritative checkout | merged shell contract truth then merged world-agent boundary proof |
| `task/m22-g3-closeout-launch-gate` | parent | `p2` | authoritative checkout | launch approval for `L3` |
| `task/m22-l3-public-surface-tests-docs-closeout` | worker | `g3` | `public-surface-tests-docs-closeout` / `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-public-surface-tests-docs-closeout` | public control tests, REPL non-regression proof, Linux world-follow-up proof, docs/gap-matrix closeout, eng-review test-plan artifact |
| `task/m22-g4-validation-wall-gate` | parent | `l3` | authoritative checkout | permission to run the final validation wall |
| `task/m22-p3-parent-validation-wall` | parent | `g4` | authoritative checkout | exact PLAN-22 validation wall results |
| `task/m22-p4-parent-closeout-phase` | parent | `p3` | authoritative checkout | terminal run-state, closeout, and artifact audit |

## Lane Ownership By File Set

| Lane | Allowed files | Forbidden touch surfaces |
| --- | --- | --- |
| `L1` / shell public-turn contract hardening | [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) only if a narrow posture helper change is required, inline unit tests inside those owned files only | `crates/shell/tests/**`, `crates/world-agent/**`, `crates/agent-api-types/**`, `docs/**`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `llm-last-mile/**`, `.runs/**` |
| `L2` / world-agent retained-member proof | [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs), [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) only if a test hook or boundary assertion cannot live solely in `member_runtime.rs`, inline tests in those owned files only | `crates/shell/**`, `crates/agent-api-types/**`, `docs/**`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, `llm-last-mile/**`, `.runs/**` |
| `L3` / public-surface tests docs closeout | [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs), [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs), `crates/shell/tests/support/**` only when directly required by those suites, [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md), [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md), [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) | every production Rust file, `.runs/**`, any claim of world-root public start, default routing, Windows/WSL parity, or detached-world follow-up success |

## Worker Interfaces

Every worker prompt must include:

1. task ID
2. attempt number
3. worktree path
4. branch
5. allowed files
6. forbidden files
7. frozen contract clauses relevant to that lane
8. exact required commands
9. retry budget
10. required return artifacts
11. sentinel path
12. the branch-mismatch ruling that the live workspace branch is authoritative

Every worker return must include:

1. changed files list
2. commands run with exit codes
3. explicit attempt classification: `clean`, `retryable`, or `blocked`
4. unresolved assumptions or blockers
5. `worker-output.patch`
6. `worker-report.md`
7. `evidence-manifest.json`

## Kickoff Initialization Order

The parent initializes the run in this exact order:

1. Create `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-22/`, `.runs/plan-22/sentinels/`, `.runs/plan-22/quarantine/`, and every `.runs/task-m22-*/` directory.
2. Create `task.json`, `commands.txt`, `summary.md`, and `artifacts/` in every task directory.
3. Create `gate-checklist.md` and `gate-result.json` in every gate task directory.
4. Create placeholder `worker-report.md`, `worker-output.patch`, and `evidence-manifest.json` in every worker task directory.
5. Write `tasks.json` as the canonical launch queue and execution ledger.
6. Write `run-state.json` with `current_phase: "kickoff"`, `worker_cap: 2`, `authoritative_branch: "feat/macos-lima-shared-owner-member-runtime-parity"`, `recorded_plan_branch: "feat/broaden-caller-surfaces-from-repl"`, every task in `pending`, and empty accepted, rejected, quarantined, and blocked arrays.
7. Write `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, and `validation-wall.md`.
8. Freeze the exact public-turn failure taxonomy and the exact detached-host versus detached-world posture rules in `contract-freeze.json`.
9. Freeze the exact docs truth ceiling in `contract-freeze.json`:
   - public `start|turn|reattach|fork|stop` exists
   - root `start` remains host-only
   - public `turn` remains exact `(orchestration_session_id, backend_id)`
   - Linux world-member follow-up is source-of-truth and must be proven
   - detached world follow-up still requires `reattach`
   - no default routing
   - no public world-root start
   - no Windows/WSL parity claim
10. Freeze the branch-resolution ruling in `contract-freeze.json` and `session-log.md`.
11. Freeze `repl_world_first_routing_v1.rs` as a mandatory non-regression gate in `contract-freeze.json` and `validation-wall.md`.
12. Review the frozen hotspots:
   - [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
   - [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   - [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
   - [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
   - [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
   - [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
   - [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
13. No pre-launch production scaffold is authorized. The existing public caller surface and world-agent boundary already exist. If a lane claims it needs parent-seeded API invention, block the run rather than create hybrid semantics.
14. Seed `L1` and `L2` worktrees from the exact same post-`p1` tree.
15. Write `session-log.md` with kickoff timestamp, both branch names, the authoritative branch ruling, the worktree roots, the worker cap, and the statement that the only honest initial parallel window is `L1` plus `L2`.

## Parent Phases And Worker Packets

### `task/m22-p1-parent-contract-freeze-and-run-init`

Owner:

- parent only

Scope:

1. Freeze the branch-mismatch disposition in favor of the live workspace branch.
2. Freeze the public caller-surface contract, REPL-first non-regression contract, and fail-closed taxonomy.
3. Freeze exact file ownership, merge order, retry budget, stop conditions, and validation wall.
4. Seed `L1` and `L2` from the same post-`p1` tree.

Command gates:

```bash
cargo test -p shell --no-run
cargo test -p world-agent --no-run
cargo test -p agent-api-types --no-run
```

Acceptance:

1. `contract-freeze.json`, `lane-ownership.json`, `merge-order.json`, `tasks.json`, and `run-state.json` exist.
2. The freeze artifact explicitly records that the current workspace branch is authoritative.
3. The frozen contract records canonical `start`, canonical `turn`, host-only root start, Linux world-member follow-up through `MemberTurnSubmitRequestV1`, detached-world fail-closed posture, unchanged `-c`, mandatory `repl_world_first_routing_v1.rs` non-regression, and late-only docs work.
4. The parent writes `.runs/plan-22/sentinels/task-m22-p1-parent-contract-freeze-and-run-init.ok`.

### `task/m22-g1-window-a-launch-gate`

Owner:

- parent only

Checks:

1. `p1` is accepted.
2. `L1` and `L2` were seeded from the exact same post-`p1` tree.
3. `L1` prompt explicitly forbids tests and docs work.
4. `L2` prompt explicitly forbids shell and docs work.
5. Both prompts repeat the frozen prohibitions on branch rewinding, selector widening, endpoint drift, detached-world success broadening, REPL-first contract regression, and `-c` reinterpretation.

Acceptance:

1. No worker starts before this gate is green.
2. The parent writes `.runs/plan-22/sentinels/task-m22-g1-window-a-launch-gate.ok`.

### `task/m22-l1-shell-public-turn-contract-hardening`

Owner:

- single worker on `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-shell-public-turn-contract-hardening`

Packet fields:

- Owned files:
  - [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
  - [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
  - [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
  - [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) only if a narrow posture helper change is required
  - inline unit tests in those owned files only
- Forbidden touch surfaces:
  - `crates/shell/tests/**`
  - `crates/world-agent/**`
  - `crates/agent-api-types/**`
  - `docs/**`
  - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
  - `llm-last-mile/**`
  - `.runs/**`
- Exact scope:
  1. Tighten `run_turn(...)`, `resolve_public_turn_target(...)`, and `run_public_prompt_command(...)` only where repo truth still leaves the public contract implied instead of explicit.
  2. Preserve exact `(orchestration_session_id, backend_id)` routing.
  3. Preserve detached-host recovery semantics.
  4. Preserve detached-world follow-up rejection with explicit reattach posture.
  5. Preserve Linux world-member follow-up through the existing public prompt bridge and retained-member submit path.
  6. Preserve classifier stability for the frozen fail-closed taxonomy.
  7. Preserve the already-landed REPL-first routing contract by not perturbing shared follow-up assumptions.
  8. Do not invent new public verbs, selectors, transport, or platform parity.
- Exact required commands:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 --no-run
cargo test -p shell --test repl_world_first_routing_v1 --no-run
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. Exact selector routing remains authoritative and fail-closed.
  3. Detached host and detached world postures remain explicit and distinct.
  4. Linux world-member follow-up still flows through the existing typed boundary.
  5. No classifier drift, no public selector widening, and no `-c` reinterpretation are introduced.
  6. The lane does not create a regression that blocks `repl_world_first_routing_v1.rs` from remaining green on the merged tree.
  7. The worker writes `.runs/plan-22/sentinels/task-m22-l1-shell-public-turn-contract-hardening.ok`.

### `task/m22-l2-world-agent-retained-member-proof`

Owner:

- single worker on `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-world-agent-retained-member-proof`

Packet fields:

- Owned files:
  - [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs)
  - [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) only if a boundary assertion or test hook cannot stay inside `member_runtime.rs`
  - inline tests in those owned files only
- Forbidden touch surfaces:
  - `crates/shell/**`
  - `crates/agent-api-types/**`
  - `docs/**`
  - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
  - `llm-last-mile/**`
  - `.runs/**`
- Exact scope:
  1. Add explicit retained-member identity-drift rejection proof at the `validate_submit_turn_request(...)` / `submit_turn(...)` boundary.
  2. Make the hidden tuple contract obvious in tests:
     - `participant_id`
     - `orchestrator_participant_id`
     - `backend_id`
     - `world_id`
     - `world_generation`
  3. Keep the request schema unchanged.
  4. Keep `/v1/member_turn/stream` unchanged.
  5. Do not broaden into shell behavior, docs, or public-surface claims.
- Exact required commands:

```bash
cargo test -p world-agent member_runtime -- --nocapture
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. At least one explicit retained-member identity-drift case is rejected at the world-agent submit boundary.
  3. The proof makes tuple continuity, not backend id alone, the obvious contract.
  4. No request schema or endpoint changes are introduced.
  5. The worker writes `.runs/plan-22/sentinels/task-m22-l2-world-agent-retained-member-proof.ok`.

### `task/m22-g2-window-a-integration-gate`

Owner:

- parent only

Checks:

1. `L1` and `L2` both returned patch, report, command transcript, and evidence manifest.
2. Every touched file is inside the lane’s ownership boundary.
3. `L1` preserved exact selector rules, detached-host recovery, detached-world rejection, Linux world follow-up, classifier stability, and REPL-first non-regression assumptions.
4. `L2` stayed inside world-agent boundary proof and did not widen the host/world contract.
5. If either lane assumed the old `feat/broaden-caller-surfaces-from-repl` branch instead of the authoritative current branch, quarantine that lane immediately.
6. `L2` is replayed only if it accidentally consumed shell-local assumptions from an unaccepted `L1` tree.

Quarantine and retry behavior:

1. If `L1` changes public verbs, handle types, `-c` semantics, `repl_world_first_routing_v1` assumptions, or doc surfaces, quarantine `L1` immediately.
2. If `L2` changes request schema, endpoint ownership, or shell files, quarantine `L2` immediately.
3. Retry budget remains `1` per lane and is available only for lane-local defects inside owned files.
4. If either lane violates branch authority, ownership, or frozen endpoint/selector constraints, mark non-retryable and quarantine rather than redrive.

Acceptance:

1. Accepted, rejected, or quarantined status for both lanes is recorded in `run-state.json`.
2. The parent writes `.runs/plan-22/sentinels/task-m22-g2-window-a-integration-gate.ok`.

### `task/m22-p2-parent-window-a-integration`

Owner:

- parent only

Scope:

1. Integrate accepted `L1` output first.
2. Re-run `L1` command gates on the authoritative checkout.
3. Replay `L2` on top of accepted `L1` only if necessary. If `L2` assumed different shell truth, quarantine `L2` instead of hand-editing around the mismatch.
4. Integrate accepted `L2` output second.
5. Freeze the merged shell-plus-world-boundary truth before public control tests and docs start.

Command gates:

```bash
cargo test -p shell --lib -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 --no-run
```

Acceptance:

1. The parent remains the sole integrator.
2. The authoritative tree now contains any required shell contract hardening and explicit world-agent retained-member proof.
3. `repl_world_first_routing_v1.rs` remains green after merged Window A truth.
4. No hybrid contract was invented during integration.
5. The parent writes `.runs/plan-22/sentinels/task-m22-p2-parent-window-a-integration.ok`.

### `task/m22-g3-closeout-launch-gate`

Owner:

- parent only

Checks:

1. `p2` is green.
2. `L3` worktree is seeded from the exact accepted post-`p2` tree.
3. The prompt names only the public control test suite, the REPL non-regression suite, test support, `docs/USAGE.md`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, and `llm-last-mile/README.md`.
4. The prompt explicitly forbids reopening production Rust files or overclaiming branch or platform truth.
5. The prompt explicitly assigns the eng-review test-plan artifact to `L3`.

Acceptance:

1. No `L3` worker starts before this gate is green.
2. The parent writes `.runs/plan-22/sentinels/task-m22-g3-closeout-launch-gate.ok`.

### `task/m22-l3-public-surface-tests-docs-closeout`

Owner:

- single worker on `codex/feat-macos-lima-shared-owner-member-runtime-parity-m22-public-surface-tests-docs-closeout`

Packet fields:

- Owned files:
  - [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
  - [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
  - `crates/shell/tests/support/**` only when directly required by those suites
  - [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
  - [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
  - [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
- Forbidden touch surfaces:
  - every production Rust file
  - `.runs/**`
  - any claim of world-root public start, default routing, Windows/WSL parity, or detached-world follow-up success
- Exact scope:
  1. Add explicit public Linux world-member follow-up success proof from exact public selector resolution to typed `MemberTurnSubmitRequestV1` submission evidence.
  2. Add explicit fail-closed tests for the frozen taxonomy:
     - `missing_backend`
     - `unknown_session`
     - `missing_active_parent`
     - `backend_not_in_session`
     - `stale_linkage`
     - `ambiguous_backend_slot`
     - noncanonical selector via `active_session_handle_id`
     - noncanonical selector via `participant_id` / legacy `session_handle_id`
     - noncanonical selector via `internal.uaa_session_id`
     - `unsupported_platform_or_posture`
     - `owner_unreachable`
  3. Add explicit detached host recovery proof and detached world rejection proof with reattach guidance.
  4. Preserve and, if needed, extend `repl_world_first_routing_v1.rs` and its support fixtures as an explicit non-regression surface for the exact REPL targeted-turn contract.
  5. Keep `substrate -c` non-regression explicit.
  6. Update `docs/USAGE.md`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, and `llm-last-mile/README.md` only to the level proven by the merged tree.
  7. Produce the usual eng-review test-plan artifact at:
     - `~/.gstack/projects/<slug>/<user>-feat-macos-lima-shared-owner-member-runtime-parity-eng-review-test-plan-<timestamp>.md`
- Exact required commands:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p agent-api-types member_turn_submit -- --nocapture
rg -n "start|turn|reattach|detached|member_turn/stream|world-root|default-agent|-c|PLAN-22|ORCH_PLAN-22" \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md
```

- Exact acceptance:
  1. The lane touches only its owned files.
  2. `agent_public_control_surface_v1.rs` explicitly proves Linux world-member public follow-up success.
  3. The frozen fail-closed taxonomy is pinned by explicit tests rather than implied behavior.
  4. Detached host recovery and detached world rejection are both part of the public contract proof.
  5. `repl_world_first_routing_v1.rs` remains an explicit owned non-regression suite and stays green, with any necessary support fixture edits confined to test-support surfaces.
  6. `substrate -c` remains explicitly protected.
  7. `llm-last-mile/README.md` remains discoverable for both `PLAN-22` and `ORCH_PLAN-22`, following the packet-index pattern used by prior orchestration plans.
  8. The eng-review test-plan artifact exists at the required `~/.gstack/projects/<slug>/...eng-review-test-plan-<timestamp>.md` path and covers:
     - host root `start`
     - host follow-up `turn`
     - Linux world-member follow-up `turn`
     - detached host recovery
     - detached world rejection
     - fail-closed selector taxonomy
     - `repl_world_first_routing_v1` non-regression
     - `substrate -c` non-regression
  9. Docs and gap-matrix truth do not overclaim beyond the merged validation wall.
  10. The worker writes `.runs/plan-22/sentinels/task-m22-l3-public-surface-tests-docs-closeout.ok`.

### `task/m22-g4-validation-wall-gate`

Owner:

- parent only

Checks:

1. `L3` returned and is accepted.
2. No quarantined or blocked output remains unresolved.
3. `validation-wall.md` names the exact final command order.
4. The parent can map every frozen completion promise to a command, test, or artifact in the merged tree.
5. The eng-review test-plan artifact path is recorded in the `L3` evidence manifest.

Quarantine and retry behavior:

1. If `L3` overclaims docs truth, drops REPL-first non-regression coverage, or omits the eng-review test-plan artifact, quarantine `L3` instead of editing by hand.
2. If `L3` failed only within owned files and stayed inside validated scope, retry budget `1` remains available.
3. No validation wall starts until `L3` is either accepted or the run is blocked.

Acceptance:

1. The parent writes `.runs/plan-22/sentinels/task-m22-g4-validation-wall-gate.ok`.
2. The validation wall is permitted to run exactly once.

### `task/m22-p3-parent-validation-wall`

Owner:

- parent only

Scope:

1. Integrate only accepted `L3` output.
2. Run the exact PLAN-22 validation wall on the authoritative checkout in this order.
3. Record command results and artifact paths.
4. Confirm docs, gap matrix, packet index, and test-plan artifact match the validated runtime truth.

Validation wall commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --lib -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-agent member_runtime -- --nocapture
cargo test -p agent-api-types member_turn_submit -- --nocapture
cargo test --workspace -- --nocapture
substrate agent doctor --json
substrate shim doctor --json
substrate world doctor --json
substrate health --json
```

Manual spot checks after command wall, on the same merged tree:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
substrate -c "echo hi"
```

Linux source-of-truth spot check when a retained world member exists:

```bash
substrate agent turn --session <orchestration_session_id> --backend <world_backend_id> --prompt "continue" --json
```

Required artifacts under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-p3-parent-validation-wall/artifacts/`:

- `fmt.txt`
- `clippy.txt`
- `shell-lib.txt`
- `shell-state-store.txt`
- `agent-public-control-surface-v1.txt`
- `repl-world-first-routing-v1.txt`
- `world-agent-member-runtime.txt`
- `agent-api-types-member-turn-submit.txt`
- `workspace.txt`
- `agent-doctor.txt`
- `shim-doctor.txt`
- `world-doctor.txt`
- `health.txt`
- `manual-spot-checks.md`
- `contract-audit.md`
- `eng-review-test-plan-path.txt`

Acceptance:

1. All validation wall commands succeed in order.
2. The validation wall proves Linux world-member public follow-up success, detached host recovery, detached world rejection, explicit fail-closed taxonomy coverage, retained-member identity-drift rejection, unchanged `-c`, and `repl_world_first_routing_v1.rs` non-regression on the same merged tree.
3. The stronger repo-standard wall including `cargo test --workspace -- --nocapture` is green.
4. The parent writes `.runs/plan-22/sentinels/task-m22-p3-parent-validation-wall.ok`.

### `task/m22-p4-parent-closeout-phase`

Owner:

- parent only

Scope:

1. Confirm all required sentinels exist and `blocked.json` does not.
2. Confirm `tasks.json` and `run-state.json` match actual accepted, rejected, and quarantined outcomes.
3. Confirm no quarantined or blocked output was partially integrated.
4. Write terminal `closeout.md`.
5. Mark the run complete only if the final validated state matches the frozen public-turn hardening contract and the authoritative branch ruling.

Required artifacts under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/task-m22-p4-parent-closeout-phase/artifacts/`:

- `closeout.md`
- `final-run-state.json`
- `final-task-ledger.json`
- `final-sentinel-audit.md`

Acceptance:

1. `run-state.json` records a successful terminal state.
2. `closeout.md` states exactly what public caller-surface hardening landed, that `repl_world_first_routing_v1.rs` stayed green as a non-regression surface, that the eng-review test-plan artifact was produced, and what remains out of scope.
3. The parent writes `.runs/plan-22/sentinels/task-m22-p4-parent-closeout-phase.ok`.

## Quarantine, Retry, And Blocked-Run Posture

1. Each worker lane has retry budget `1`.
2. Retry is allowed only for lane-local defects inside owned files.
3. Non-retryable violations include:
   - branch-authority reversal
   - selector widening
   - endpoint drift away from `/v1/member_turn/stream`
   - detached-world success broadening
   - `-c` reinterpretation
   - request schema changes
   - REPL-first routing contract regression
   - early docs work
   - any cross-lane file touch
4. If `L1` cannot stay inside the existing shell public-turn seams, quarantine it immediately.
5. If `L2` can only pass by changing request schema or shell behavior, quarantine it immediately.
6. If `L3` needs production Rust edits to make tests pass, quarantine it and bounce the issue back to the parent rather than reopening runtime code late.
7. The parent never hand-merges a hybrid truth from conflicting worker guesses.

When a lane is quarantined, the parent must preserve the returned materials in both places:

1. The original task artifact directory.
2. `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-22/quarantine/<task-id>/`

`quarantine/<task-id>/quarantine-reason.json` must record:

- `task_id`
- `classification`
- `summary`
- `files_touched`
- `frozen_contract_clause_violated`
- `retry_available`
- `next_parent_action`

## Validation Wall

The parent may run the final validation wall exactly once, only after `g4` is green.

The wall is green only when all of these are true:

1. `substrate agent start` still remains the canonical public root prompt-taking surface.
2. `substrate agent turn` still remains the canonical public follow-up surface.
3. Linux world-member public follow-up is directly proven from exact public selectors into typed retained-member submission.
4. Detached host follow-up recovery is proven explicitly.
5. Detached world follow-up rejection with reattach guidance is proven explicitly.
6. The frozen public-turn fail-closed taxonomy is covered explicitly by tests.
7. At least one retained-member identity-drift case is rejected at the world-agent boundary.
8. `substrate -c` remains shell-wrap behavior.
9. `repl_world_first_routing_v1.rs` is green and still proves the exact REPL-first targeted follow-up contract did not regress while public caller surfaces were hardened.
10. `docs/USAGE.md`, `AGENT_ORCHESTRATION_GAP_MATRIX.md`, and `llm-last-mile/README.md` match the validated runtime truth.
11. The eng-review test-plan artifact exists and matches the final proof surfaces.
12. No quarantined or blocked output remains unresolved.

## Tests And Acceptance

### Shell Contract Hardening

Acceptance requires all of these to be true:

- exact `(orchestration_session_id, backend_id)` resolution stays authoritative
- detached host and detached world postures remain explicit and distinct
- Linux world follow-up still uses the existing retained-member public prompt path
- no public selector widening is introduced
- `cargo test -p shell --lib -- --nocapture` and `cargo test -p shell agent_runtime::state_store -- --nocapture` are green
- `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` remains green after integration

### World-Agent Retained-Member Proof

Acceptance requires all of these to be true:

- retained-member identity drift is proven as an explicit submit-boundary rejection
- the test makes the exact tuple continuity contract obvious
- request schema and endpoint ownership stay unchanged
- `cargo test -p world-agent member_runtime -- --nocapture` is green

### Public Control Suite, REPL Non-Regression, And Docs Truth

Acceptance requires all of these to be true:

- `agent_public_control_surface_v1.rs` explicitly covers:
  - host-scoped public start
  - host follow-up public turn
  - Linux world-member public turn success
  - detached host recovery
  - detached world rejection
  - `missing_backend`
  - `unknown_session`
  - `missing_active_parent`
  - `backend_not_in_session`
  - `stale_linkage`
  - `ambiguous_backend_slot`
  - each noncanonical selector variant
  - `unsupported_platform_or_posture`
  - `owner_unreachable`
  - root world-only start rejection
  - `substrate -c` non-regression
- `repl_world_first_routing_v1.rs` explicitly remains green as a required non-regression surface, with any fixture updates constrained to test-support files
- the docs say only what the merged tree proves:
  - public `start|turn|reattach|fork|stop` exists
  - root `start` is still host-only
  - follow-up `turn` requires exact `(orchestration_session_id, backend_id)`
  - Linux world-member follow-up is proven
  - detached world follow-up still requires `reattach`
  - no default routing
  - no public world-root start
  - no Windows/WSL parity claim
- `llm-last-mile/README.md` remains discoverable for both `PLAN-22` and `ORCH_PLAN-22`
- the eng-review test-plan artifact exists under `~/.gstack/projects/<slug>/...eng-review-test-plan-<timestamp>.md`

### Operator Flow And Run-State Artifacts

Acceptance requires all of these to be true:

- the parent is the sole writer of `.runs/plan-22/**` and `.runs/task-m22-*/**`
- `tasks.json` and `run-state.json` accurately reflect accepted, rejected, quarantined, and blocked outcomes
- every required sentinel exists on the green path
- `blocked.json` is absent on the green path
- final validation artifacts exist under the `p3` and `p4` task directories
- `closeout.md` states exactly what landed and what remains out of scope without overclaim

## Closeout Phase

The run is complete only when:

1. Every required sentinel exists.
2. `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-22/blocked.json` does not exist.
3. The exact validation wall is green.
4. `closeout.md` names the authoritative branch ruling, the landed public caller-surface hardening contract, the preserved `repl_world_first_routing_v1.rs` non-regression contract, the eng-review test-plan artifact, and any remaining out-of-scope gaps without overclaim.
5. The final merged tree reflects hardening and proof of the already-shipped public surface, not a new orchestration model.
