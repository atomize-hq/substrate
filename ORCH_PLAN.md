# ORCH_PLAN: Durable Host Session Closeout

Authoritative execution branch: `feat/host-orchestrator-durable-session`  
Plan source: [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Truth record: [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)  
Controller path: [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)  
Live workspace root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout`  
Run id: `durable-host-session-closeout`  
Worker model: `GPT-5.4`  
Reasoning effort: `high`  
Max concurrent workers: `2`  
Parent integrator: `only integrator, only gate authority, only writer of .runs artifacts`

## Summary

This document is the authoritative orchestration controller for executing [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) to completion. It is not another design plan. It tells the parent agent exactly how to execute the implementation across workers, gates, worktrees, validations, and final acceptance.

The execution decision is:

1. keep runtime correction work single-owner until the durable control model is fully integrated and stable
2. open exactly one late parallel window, with two workers only, after the runtime lane is merged
3. keep final integration, manual CLI proof, and acceptance wall parent-only

That is the highest honest concurrency for this plan. `agents_cmd.rs`, `state_store.rs`, `control.rs`, `async_repl.rs`, and `agent_dev_support.rs` are one semantic seam under the frozen contract. Splitting them earlier would create merge churn and ambiguous ownership around the same control-truth hotspots. Only after that seam is merged can tests and docs proceed safely in parallel on disjoint surfaces.

## Hard Guards

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/host-orchestrator-durable-session`.
2. The parent is the only integrator, the only approval authority, and the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/durable-host-session-closeout/**`.
3. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) is the frozen implementation contract. This controller operationalizes that contract and may not broaden it.
4. No new public verbs, no selector broadening, no “latest session” fallback, and no shortcuts that contradict `PLAN.md`.
5. The durable authority remains the persisted orchestration session record, authoritative participant linkage, durable inbox state, and persisted lifecycle metadata.
6. `active_attached`, `parked_resumable`, and `awaiting_attention` all remain active durable-session postures. `terminal` is the only non-routable posture family.
7. `reattach` may report success only when attached ownership is durably restored for that exact session.
8. `stop` must stop the durable session model, not merely a reachable attached owner transport.
9. `status --json` must surface valid parked and attention-needed sessions from authoritative session truth even when no live owner process is attached.
10. Parked durable inbox items must continue to persist and normalize posture to `awaiting_attention`.
11. The prompt-bridge invariant stays frozen: after `Accepted`, the public bridge must emit `Completed` or `Failed`.
12. Detached world-member follow-up remains fail closed. This run must not weaken that contract.
13. No worker edits `.runs/**`, `PLAN.md`, or this `ORCH_PLAN.md`.
14. Docs stay late. No worker edits docs before the merged runtime tree proves the shipped behavior.
15. Any need for a new timeout or stale-session lifecycle, new durable inbox schema, or broader world recovery policy is an immediate blocker and must produce `blocked.json`.
16. Any required `gitnexus_impact` result that comes back `HIGH` or `CRITICAL` must be escalated to the parent before edits proceed.

Stop the run immediately and write `blocked.json` if any of these occur:

1. the runtime lane cannot complete Workstreams 1 through 5 without reopening the public contract
2. the CLI regression lane needs edits in runtime ownership files
3. the docs lane finds unresolved behavior ambiguity after runtime integration
4. the final merged tree cannot prove the exact manual CLI wall required by `PLAN.md`
5. `reattach`, `stop`, `status`, or parked inbox behavior can only be made green by introducing a new public behavior not already frozen in `PLAN.md`

## Workstream Plan

### Parent-only sequential initialization and contract freeze

This phase is strictly sequential and parent-owned.

Purpose:

1. initialize run-state artifacts
2. source-lock `PLAN.md` and the truth record
3. freeze lane ownership
4. freeze merge order
5. freeze the validation wall
6. launch the runtime lane with unambiguous boundaries

Why parent-only:

1. this phase defines the contract the workers must obey
2. it decides the only honest concurrency window
3. it records blocker conditions and acceptance surfaces before any worker edits code

### Runtime control lane

This is the main implementation lane and remains single-owner.

Owner:

- Worker A only

Scope:

1. Workstream 1: freeze one session-control posture contract in code
2. Workstream 2: make `reattach` prove actual attachment
3. Workstream 3: make `stop` session-centric
4. Workstream 4: make `status` reflect durable parked truth
5. Workstream 5: prove parked inbox responsibility operationally

Execution decision:

- no parallel workers are allowed in this phase because all required runtime changes share the same control-model seam and the same acceptance invariants

### Late parallel window

This is the only honest parallel window.

Owners:

- Worker B: CLI regression proof
- Worker C: docs closeout

Entry condition:

- the runtime control lane is merged and targeted runtime validation is green

Why this window is safe:

1. the CLI regression surface is `crates/shell/tests/agent_public_control_surface_v1.rs`
2. the docs surface is late and disjoint
3. neither lane is allowed to reopen runtime ownership files

Why there is no earlier or broader parallelism:

1. before runtime integration, tests would chase moving semantics
2. before runtime integration, docs would document intent instead of shipped truth
3. adding a third worker would overlap either the runtime seam or the parent’s integration and validation wall, which would be dishonest concurrency

### Parent-only integration and validation closeout

This phase is strictly parent-owned.

Purpose:

1. integrate accepted worker outputs in frozen order
2. run the full validation wall from `PLAN.md`
3. run the manual CLI proof on the merged tree
4. run doctor and health evidence capture
5. run `gitnexus_detect_changes()`
6. close the run with a complete artifact trail

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout`

Parent integration surface:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/host-orchestrator-durable-session`

Worker worktrees and branches:

- Worker A worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout/runtime-control`
- Worker A branch: `codex/feat-host-orchestrator-durable-session-runtime-control`
- Worker B worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout/cli-regressions`
- Worker B branch: `codex/feat-host-orchestrator-durable-session-cli-regressions`
- Worker C worktree: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout/docs-closeout`
- Worker C branch: `codex/feat-host-orchestrator-durable-session-docs-closeout`

Launch commands for Worker A:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout/runtime-control \
  -b codex/feat-host-orchestrator-durable-session-runtime-control \
  HEAD
```

Late-window launch commands, to run only after runtime integration is accepted:

```bash
git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout/cli-regressions \
  -b codex/feat-host-orchestrator-durable-session-cli-regressions \
  HEAD

git worktree add /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-durable-host-session-closeout/docs-closeout \
  -b codex/feat-host-orchestrator-durable-session-docs-closeout \
  HEAD
```

## Parent-Owned Run-State Surface

Canonical run root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/durable-host-session-closeout/`

Required top-level artifacts:

- `run-state.json`
- `task-ledger.json`
- `session-log.md`
- `contract-freeze.json`
- `lane-ownership.json`
- `merge-order.json`
- `impact/`
- `validation/`
- `quarantine/`
- `gates/`
- `qa/`
- `sentinels/`
- `blocked.json` on blocked termination only
- `closeout.md` on successful completion only

Required task roots:

- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-p0-parent-run-init-and-source-lock/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-p1-parent-contract-freeze-and-lane-lock/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-g1-parent-runtime-lane-launch-gate/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-a1-worker-runtime-control-implementation/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-a2-worker-runtime-control-validation-and-handoff/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-g2-parent-runtime-acceptance-gate/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-p2-parent-runtime-integration-and-stabilization/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-g3-parent-late-window-launch-gate/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-b1-worker-cli-regression-implementation/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-b2-worker-cli-regression-validation-and-handoff/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-c1-worker-docs-closeout-implementation/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-c2-worker-docs-closeout-validation-and-handoff/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-g4-parent-late-window-acceptance-gate/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-p3-parent-validation-wall-and-manual-cli/`
- `.runs/durable-host-session-closeout/tasks/task-durable-host-session-closeout-p4-parent-final-closeout/`

Each task directory must contain:

- `task.json`
- `commands.txt`
- `summary.md`

Each gate directory must also contain:

- `gate-checklist.md`
- `gate-result.json`

Each worker task directory must also contain:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`
- `impact-analysis.md`

Validation artifact roots:

- `.runs/durable-host-session-closeout/validation/lane-a/`
- `.runs/durable-host-session-closeout/validation/lane-b/`
- `.runs/durable-host-session-closeout/validation/lane-c/`
- `.runs/durable-host-session-closeout/validation/integration/`
- `.runs/durable-host-session-closeout/validation/final/`
- `.runs/durable-host-session-closeout/validation/validation-wall.md`

Required sentinels:

- `01--task-durable-host-session-closeout-p0-parent-run-init-and-source-lock.ok`
- `02--task-durable-host-session-closeout-p1-parent-contract-freeze-and-lane-lock.ok`
- `03--task-durable-host-session-closeout-g1-parent-runtime-lane-launch-gate.ok`
- `04--task-durable-host-session-closeout-a2-worker-runtime-control-validation-and-handoff.ok`
- `05--task-durable-host-session-closeout-g2-parent-runtime-acceptance-gate.ok`
- `06--task-durable-host-session-closeout-p2-parent-runtime-integration-and-stabilization.ok`
- `07--task-durable-host-session-closeout-g3-parent-late-window-launch-gate.ok`
- `08--task-durable-host-session-closeout-b2-worker-cli-regression-validation-and-handoff.ok`
- `09--task-durable-host-session-closeout-c2-worker-docs-closeout-validation-and-handoff.ok`
- `10--task-durable-host-session-closeout-g4-parent-late-window-acceptance-gate.ok`
- `11--task-durable-host-session-closeout-p3-parent-validation-wall-and-manual-cli.ok`
- `12--task-durable-host-session-closeout-p4-parent-final-closeout.ok`

Ledger rules:

1. `run-state.json` is the authoritative run ledger
2. `task-ledger.json` is the machine-readable task status map
3. `session-log.md` is the parent-authored decision log
4. workers never write sentinels
5. the parent writes a sentinel only after the corresponding task or gate is accepted
6. `blocked.json` is written once, by the parent only, at the exact stop decision

## Frozen Runtime Contract For This Run

`contract-freeze.json` must freeze these exact truths before Worker A launches:

1. one shared session-control posture truth governs `reattach`, `stop`, and parked-session `status`
2. `reattach` success means the exact session is durably `active_attached`
3. `stop` must dispatch by session posture, not live-owner availability alone
4. parked and `awaiting_attention` sessions remain active durable sessions
5. parked inbox truth is authoritative via `pending_inbox_count` and persisted inbox state
6. `turn` remains exact `(session, backend)` follow-up prompt-taking only
7. `reattach` remains non-prompt-taking attached-owner recovery only
8. `start` and `turn` semantics already landed must be preserved
9. broken startup remains fail closed as `runtime_start_failed`
10. the public prompt bridge remains `Accepted -> Completed|Failed`
11. detached-world follow-up remains fail closed
12. no new public behavior may be introduced to get tests green

`contract-freeze.json` must also record:

- runtime lane ownership files
- late CLI regression lane ownership files
- docs lane ownership files
- parent-only integration surfaces
- the exact validation wall from `PLAN.md`
- blocker conditions that force `blocked.json`

## Frozen Ownership Boundaries

Parent-only for the entire run:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/**`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md`
- final integration in `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- final manual CLI validation and doctor and health captures

Worker A owned files:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_dev_support.rs`
- inline tests inside those files

Worker B owned files:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs`
- harness-only test helper files created or modified strictly for that integration test, if and only if they do not overlap Worker A or Worker C surfaces

Worker C owned files:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md`
- any additional doc file only if the parent explicitly adds it to `lane-ownership.json` after runtime integration

Explicit prohibitions:

1. Worker A may not edit docs, `.runs/**`, or `crates/shell/tests/agent_public_control_surface_v1.rs`
2. Worker B may not edit runtime code files
3. Worker C may not edit runtime code files or test files
4. no worker may widen ownership without a parent gate decision recorded in `session-log.md`
5. parent integration fixes must remain narrow and recorded as parent-owned glue

## Task Ledger

| Task ID | Owner | Depends on | Worktree / branch | Deliverable |
| --- | --- | --- | --- | --- |
| `task-durable-host-session-closeout-p0-parent-run-init-and-source-lock` | parent | — | main checkout / `feat/host-orchestrator-durable-session` | run scaffold, source lock, preflight impact root, QA artifact path |
| `task-durable-host-session-closeout-p1-parent-contract-freeze-and-lane-lock` | parent | `p0` | main checkout / `feat/host-orchestrator-durable-session` | frozen contract, lane ownership, merge order, validation wall |
| `task-durable-host-session-closeout-g1-parent-runtime-lane-launch-gate` | parent | `p1` | main checkout / `feat/host-orchestrator-durable-session` | launch Worker A |
| `task-durable-host-session-closeout-a1-worker-runtime-control-implementation` | Worker A | `g1` | runtime-control / `codex/feat-host-orchestrator-durable-session-runtime-control` | runtime code implementing Workstreams 1 through 5 |
| `task-durable-host-session-closeout-a2-worker-runtime-control-validation-and-handoff` | Worker A | `a1` | same | validated patch, impact record, lane evidence |
| `task-durable-host-session-closeout-g2-parent-runtime-acceptance-gate` | parent | `a2` | main checkout / `feat/host-orchestrator-durable-session` | accept, reject, or quarantine runtime lane |
| `task-durable-host-session-closeout-p2-parent-runtime-integration-and-stabilization` | parent | `g2` | main checkout / `feat/host-orchestrator-durable-session` | merged runtime tree, targeted stabilization, late-window launch base |
| `task-durable-host-session-closeout-g3-parent-late-window-launch-gate` | parent | `p2` | main checkout / `feat/host-orchestrator-durable-session` | launch Workers B and C |
| `task-durable-host-session-closeout-b1-worker-cli-regression-implementation` | Worker B | `g3` | cli-regressions / `codex/feat-host-orchestrator-durable-session-cli-regressions` | CLI regression proof in integration test surface |
| `task-durable-host-session-closeout-b2-worker-cli-regression-validation-and-handoff` | Worker B | `b1` | same | validated patch, impact record, lane evidence |
| `task-durable-host-session-closeout-c1-worker-docs-closeout-implementation` | Worker C | `g3` | docs-closeout / `codex/feat-host-orchestrator-durable-session-docs-closeout` | docs aligned to merged runtime truth |
| `task-durable-host-session-closeout-c2-worker-docs-closeout-validation-and-handoff` | Worker C | `c1` | same | validated patch and lane evidence |
| `task-durable-host-session-closeout-g4-parent-late-window-acceptance-gate` | parent | `b2`, `c2` | main checkout / `feat/host-orchestrator-durable-session` | accept, reject, or quarantine each late lane |
| `task-durable-host-session-closeout-p3-parent-validation-wall-and-manual-cli` | parent | `g4` | main checkout / `feat/host-orchestrator-durable-session` | final command wall, manual CLI evidence, repo runtime evidence |
| `task-durable-host-session-closeout-p4-parent-final-closeout` | parent | `p3` | main checkout / `feat/host-orchestrator-durable-session` | `gitnexus_detect_changes()` record, closeout, run completion |

## Task Graph And Control Points

### `task-durable-host-session-closeout-p0-parent-run-init-and-source-lock`

Required actions:

1. create the full `.runs/durable-host-session-closeout/` tree
2. write initial `run-state.json`, `task-ledger.json`, `lane-ownership.json`, and `validation/validation-wall.md`
3. source-lock these inputs:
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md`
   - `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md`
4. write the QA-facing test-plan artifact required by `PLAN.md` under `~/.gstack/projects/<slug>/`
5. record the QA artifact path in `.runs/durable-host-session-closeout/qa/test-plan-artifact-path.txt`
6. inventory expected symbol surfaces for Worker A and Worker B

Acceptance:

- source lock recorded in `session-log.md`
- run-state scaffolding exists
- QA artifact path recorded
- `01--task-durable-host-session-closeout-p0-parent-run-init-and-source-lock.ok`

### `task-durable-host-session-closeout-p1-parent-contract-freeze-and-lane-lock`

Required actions:

1. freeze the runtime contract from `PLAN.md` into `contract-freeze.json`
2. freeze lane ownership and prohibitions into `lane-ownership.json`
3. freeze `merge-order.json`
4. freeze the exact validation wall and artifact names into `validation/validation-wall.md`
5. record that runtime code is a single-owner lane and late parallelism is tests plus docs only
6. record blocker conditions that force `blocked.json`

Acceptance:

- the contract freeze matches `PLAN.md` exactly
- lane boundaries are narrow and auditable
- no early fake parallelism remains in the plan
- `02--task-durable-host-session-closeout-p1-parent-contract-freeze-and-lane-lock.ok`

### `task-durable-host-session-closeout-g1-parent-runtime-lane-launch-gate`

Gate must reject launch if:

1. contract freeze is incomplete
2. runtime ownership is still ambiguous
3. late-lane ownership overlaps runtime files
4. the parent has not recorded the exact validation commands Worker A must run

Acceptance:

- Worker A worktree created from the accepted `p1` launch base
- `03--task-durable-host-session-closeout-g1-parent-runtime-lane-launch-gate.ok`

### `task-durable-host-session-closeout-a1-worker-runtime-control-implementation`

Scope:

1. implement the shared session-control posture contract
2. make `reattach` prove durable attached ownership
3. make `stop` session-centric for attached and parked sessions
4. make `status` surface parked and `awaiting_attention` sessions from authoritative truth
5. prove parked inbox responsibility operationally
6. preserve `start` and `turn` semantics already frozen in `PLAN.md`
7. preserve detached-world fail-closed behavior
8. preserve the prompt-bridge invariant

Before first edit:

1. run `gitnexus_impact` for each production symbol to be changed
2. record direct callers, affected processes, and risk level in `impact-analysis.md`
3. stop and escalate to the parent if any required edit is `HIGH` or `CRITICAL`

### `task-durable-host-session-closeout-a2-worker-runtime-control-validation-and-handoff`

Minimum lane-A validation:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell async_repl -- --nocapture
```

If cargo filters need syntactic adjustment, Worker A must run the narrowest equivalent commands and record exact substitutions in `commands.txt`.

Required handoff artifacts:

- `worker-report.md`
- `worker-output.patch`
- `evidence-manifest.json`
- `impact-analysis.md`

Acceptance target for parent:

- Worker A stayed inside owned files
- Workstreams 1 through 5 are implemented in one coherent lane
- the lane includes inline runtime tests needed for changed runtime files
- `04--task-durable-host-session-closeout-a2-worker-runtime-control-validation-and-handoff.ok`

### `task-durable-host-session-closeout-g2-parent-runtime-acceptance-gate`

Gate rules:

1. Worker A output must originate from the accepted `g1` launch base
2. Worker A is rejected or quarantined if it widened into docs, `.runs/**`, or the late integration test surface
3. any output that weakens the frozen contract is rejected
4. any unresolved `HIGH` or `CRITICAL` impact escalation must be parent-resolved before acceptance

Acceptance:

- accepted or quarantined state recorded in `run-state.json`
- `05--task-durable-host-session-closeout-g2-parent-runtime-acceptance-gate.ok`

### `task-durable-host-session-closeout-p2-parent-runtime-integration-and-stabilization`

Required parent actions:

1. integrate the accepted Worker A lane
2. run targeted runtime validation on the merged tree
3. apply only narrow parent-owned glue if needed
4. verify the merged runtime tree is stable enough that the CLI regression lane will not need runtime edits
5. record the late-window launch base in `merge-order.json`

Acceptance:

- runtime lane merged into the authoritative branch
- targeted runtime validation is green
- late-window launch base recorded
- `06--task-durable-host-session-closeout-p2-parent-runtime-integration-and-stabilization.ok`

### `task-durable-host-session-closeout-g3-parent-late-window-launch-gate`

Gate must reject launch if:

1. the merged runtime tree is not stable
2. Worker B would need runtime file edits
3. docs truth is still unsettled
4. parent integration work is still open in runtime files

Acceptance:

- Worker B and Worker C worktrees created from the accepted `p2` launch base
- `07--task-durable-host-session-closeout-g3-parent-late-window-launch-gate.ok`

### `task-durable-host-session-closeout-b1-worker-cli-regression-implementation`

Scope:

1. extend `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs`
2. prove the exact CLI flows from `PLAN.md`
3. keep runtime file ownership closed
4. request parent escalation instead of reopening runtime files

Before first edit:

1. run `gitnexus_impact` on the production symbols under test
2. record the blast radius in `impact-analysis.md`
3. stop and escalate if the test lane would require production file edits

### `task-durable-host-session-closeout-b2-worker-cli-regression-validation-and-handoff`

Minimum lane-B validation:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Required proof surface in the test file:

1. `start -> parked_resumable`
2. parked `status`
3. parked session receives inbox work and normalizes to `awaiting_attention`
4. `turn` resumes the same exact session
5. `reattach` succeeds only on durable attached truth
6. attached and parked `stop` both succeed
7. broken bootstrap still fails as `runtime_start_failed`
8. post-`Accepted` helper loss still yields explicit `Failed`
9. detached-world follow-up remains fail closed

Acceptance target for parent:

- Worker B stayed out of runtime files
- the CLI regression proof matches the frozen contract
- `08--task-durable-host-session-closeout-b2-worker-cli-regression-validation-and-handoff.ok`

### `task-durable-host-session-closeout-c1-worker-docs-closeout-implementation`

Scope:

1. remove attached-live-only wording from owned docs
2. state that `reattach` success means durable attached truth
3. state that `stop` is the canonical closeout path for attached and parked sessions
4. state that `status` surfaces parked durable sessions
5. update nearby diagrams only if runtime truth is already proven and the diagrams would otherwise be stale

### `task-durable-host-session-closeout-c2-worker-docs-closeout-validation-and-handoff`

Minimum lane-C validation:

1. record every changed doc path in `worker-report.md`
2. include quoted before and after truth statements for each changed behavioral claim
3. confirm no doc claims exceed what the merged runtime tree already proves

Acceptance target for parent:

- Worker C stayed inside owned docs
- docs do not introduce new behavior
- `09--task-durable-host-session-closeout-c2-worker-docs-closeout-validation-and-handoff.ok`

### `task-durable-host-session-closeout-g4-parent-late-window-acceptance-gate`

Gate rules:

1. Worker B and Worker C outputs must originate from the accepted `p2` launch base
2. Worker B is quarantined if it requires runtime edits
3. Worker C is quarantined if it documents behavior not yet proven
4. either lane may be rejected independently without invalidating the other

Acceptance:

- accepted late-lane set recorded in `run-state.json`
- `10--task-durable-host-session-closeout-g4-parent-late-window-acceptance-gate.ok`

### `task-durable-host-session-closeout-p3-parent-validation-wall-and-manual-cli`

Required parent actions:

1. integrate accepted late-lane outputs in frozen order: Worker B first, Worker C second
2. run the full command wall from `PLAN.md`
3. run the full manual CLI wall from `PLAN.md`
4. capture repo-level runtime evidence
5. write `command-mapping.md` if any `PLAN.md` command required a narrow equivalent

Acceptance:

- the exact CLI contract from `PLAN.md` is proven on the final merged tree
- evidence is recorded under `validation/final/`
- `11--task-durable-host-session-closeout-p3-parent-validation-wall-and-manual-cli.ok`

### `task-durable-host-session-closeout-p4-parent-final-closeout`

Required final actions:

1. run `gitnexus_detect_changes()` before any commit or landing prep
2. record final symbol and flow drift
3. verify final validation artifacts exist
4. resolve or explicitly retain no quarantined outputs
5. write `closeout.md`
6. mark the run complete in `run-state.json`

Acceptance:

- `blocked.json` is absent
- `12--task-durable-host-session-closeout-p4-parent-final-closeout.ok`

## Merge Order And Quarantine Rules

`merge-order.json` must record:

- `authoritative_branch: "feat/host-orchestrator-durable-session"`
- `runtime_launch_base: "post-p1-head"`
- `late_window_launch_base: "post-p2-integrated-runtime-head"`
- `integration_order: ["task-durable-host-session-closeout-a1-worker-runtime-control-implementation", "task-durable-host-session-closeout-p2-parent-runtime-integration-and-stabilization", "task-durable-host-session-closeout-b1-worker-cli-regression-implementation", "task-durable-host-session-closeout-c1-worker-docs-closeout-implementation", "task-durable-host-session-closeout-p3-parent-validation-wall-and-manual-cli"]`
- `quarantine_on_scope_drift: true`

Quarantine rules:

1. quarantined output is copied under `.runs/durable-host-session-closeout/quarantine/<task-id>/`
2. quarantined output must include the patch, report, impact artifact, and evidence manifest
3. quarantined output is never treated as partially accepted without an explicit parent reconciliation note in `session-log.md`
4. if Worker B needs runtime edits, the entire late-window CLI lane is quarantined automatically
5. if Worker C documents behavior beyond the merged runtime tree, the docs lane is quarantined automatically

## GitNexus Operating Procedure

GitNexus is mandatory for the execution run described by this controller.

During `task-durable-host-session-closeout-p0-parent-run-init-and-source-lock`:

1. inspect index freshness
2. if stale, run `npx gitnexus analyze`
3. record index status under `.runs/durable-host-session-closeout/impact/preflight/`

Before Worker A edits runtime symbols:

1. run `gitnexus_impact` for each target symbol in `agents_cmd.rs`, `state_store.rs`, `control.rs`, `async_repl.rs`, and `agent_dev_support.rs`
2. record direct callers, affected processes, and risk level in `impact-analysis.md`
3. escalate any `HIGH` or `CRITICAL` result before editing

Before Worker B edits the integration test surface:

1. run `gitnexus_impact` for the production symbols the regression file is proving
2. record the blast radius in `impact-analysis.md`
3. do not use the test lane to silently drive production behavior changes

During `task-durable-host-session-closeout-p4-parent-final-closeout`:

1. run `gitnexus_detect_changes()`
2. verify only expected symbols and execution flows changed
3. record the result in `.runs/durable-host-session-closeout/validation/final/`
4. do not declare completion without that record

## Context-Control Rules

Parent live context limit:

1. [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
2. `run-state.json`
3. `contract-freeze.json`
4. the latest integrated diff summary
5. worker reports only for the lane currently at gate review

Worker A prompt contents only:

1. its task brief
2. the frozen runtime contract
3. exact owned files
4. explicit forbidden touch surfaces
5. validation commands

Worker B prompt contents only:

1. its task brief
2. the frozen CLI contract and required flows
3. the exact owned test surface
4. explicit prohibition on runtime edits
5. validation commands

Worker C prompt contents only:

1. its task brief
2. the frozen runtime truths already proven by runtime integration
3. exact owned docs
4. explicit prohibition on behavior invention
5. validation expectations

Every worker must return:

1. changed files
2. symbols touched or production symbols proved
3. commands run and exit codes
4. blockers or scope-drift requests
5. any disputed assumptions

## Validation And Acceptance

### Required command wall from `PLAN.md`

These commands are mandatory on the final merged tree:

```bash
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell async_repl -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

If any command needs a syntactic adjustment, the parent may substitute the narrowest equivalent command only if:

1. it covers the same behavior
2. the substitution is written to `.runs/durable-host-session-closeout/validation/final/command-mapping.md`
3. both the original and replacement commands are recorded

### Required manual CLI wall from `PLAN.md`

The parent must rerun the real CLI flow and capture persisted runtime evidence:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent status --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent status --json
```

The parent must also prove:

1. one detached inbox item moves the same session to `awaiting_attention`
2. broken bootstrap still fails as `runtime_start_failed`
3. post-`Accepted` helper loss still yields explicit `Failed`
4. detached-world follow-up still fails closed with reattach guidance

Required final artifacts:

- `manual-start.json`
- `manual-status-after-start.json`
- `manual-session-after-start.json`
- `manual-session-after-awaiting-attention.json`
- `manual-turn.json`
- `manual-reattach.json`
- `manual-stop.json`
- `manual-final-status.json`
- `manual-broken-bootstrap.json`
- `manual-post-accepted-late-failure.json`
- `manual-detached-world-fail-closed.json`
- `manual-cli-summary.md`

### Required repo-level shell-runtime evidence

Because this run changes shell runtime behavior, the parent must also record:

```bash
substrate shim doctor --json
substrate health --json
```

These outputs must be captured as:

- `.runs/durable-host-session-closeout/validation/final/shim-doctor.json`
- `.runs/durable-host-session-closeout/validation/final/health.json`

World-doctor evidence is conditional:

1. if the merged diff touches world backends, world transport, or world-facing runtime ownership behavior, run `substrate world doctor --json` and capture `world-doctor.json`
2. if the merged diff stays outside world-owned surfaces, write `world-doctor-rationale.md` naming the touched files and the reason `world doctor` was not required

### Repo hygiene wall

Before completion, the parent should also run and record:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

If full-workspace execution is not feasible in the local environment, the parent must record the exact reason and unresolved risk in `validation/final/repo-hygiene-rationale.md`.

## Blocked-Run Artifact Behavior

`blocked.json` is parent-written only and must include:

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
- `impact_artifacts`
- `next_required_parent_action`

Blocked-run rules:

1. parent writes `blocked.json` before any later-phase sentinel is created
2. parent updates `run-state.json` to blocked in the same decision window
3. parent records the reason in `session-log.md`
4. `closeout.md` is not written on a blocked run

## Completion Conditions

This orchestration controller succeeds only when the parent can say all of the following are true on the same merged tree:

1. the current [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) drove the run from start to finish
2. the authoritative branch stayed `feat/host-orchestrator-durable-session`
3. the run used one honest late two-worker window and no dishonest early parallelism
4. the parent remained the only integrator and the only gate authority
5. `reattach` success means durable `active_attached`, not helper launch
6. `stop` cleanly closes attached and parked sessions through authoritative durable truth
7. `status --json` surfaces valid `parked_resumable` and `awaiting_attention` sessions from session truth
8. a detached inbox item persists and drives `awaiting_attention` on the same session
9. `start` and `turn` semantics remain preserved
10. broken bootstrap still fails closed as `runtime_start_failed`
11. the prompt bridge still guarantees `Accepted -> Completed|Failed`
12. detached-world follow-up remains fail closed
13. the CLI regression wall proves `start -> status -> turn -> reattach -> stop` on one durable session
14. docs reflect shipped truth and do not preserve the attached-live-only mental model
15. the run leaves behind a complete `.runs/durable-host-session-closeout/` audit trail
16. `gitnexus_detect_changes()` is recorded and consistent with expected scope

## Assumptions

1. the checked-out authoritative branch remains `feat/host-orchestrator-durable-session` throughout the run
2. the repository’s current GitNexus index is available or can be refreshed with `npx gitnexus analyze`
3. the manual CLI proof can run against sanctioned host backend fixtures already expected by `PLAN.md`
4. no hidden repo policy requires editing files outside the ownership boundaries listed above
5. full workspace validation may expose unrelated pre-existing failures; if so, those are recorded rather than silently folded into this slice
