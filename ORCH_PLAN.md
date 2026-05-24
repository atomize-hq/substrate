# ORCH_PLAN: Slice 29 Shared Dispatch Contract Execution Controller

Authoritative execution branch: `feat/gateway-mediated-llm-fulfillment`  
Plan source: [PLAN.md](PLAN.md)  
Current controller target: [ORCH_PLAN.md](ORCH_PLAN.md)  
Workspace root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract`  
Run id: `slice-29-shared-dispatch-contract`  
Worker model: `GPT-5.4 high`  
Initial concurrent worker cap: `0` during parent freeze  
Peak concurrent worker cap: `2`  
Parent role: sole integrator, sole gate owner, sole writer of `.runs/**`, sole authority for hotspot transfer, merge order, validation, blockage, and final acceptance

## Summary And Frozen Execution Shape

This document executes the current [PLAN.md](PLAN.md). It is an execution controller, not a restatement of the plan.

This run is complete only when one merged tree proves all of the following:

1. one shared internal dispatch contract module exists under `crates/shell/src/execution/agent_runtime/`;
2. one inventory-backed and one persisted-attach-backed baseline domain are modeled explicitly;
3. runtime materialization flows from the resolved contract instead of caller-specific planning;
4. `HostAttachContract` persists the generalized host launch truth derived from that resolved contract;
5. human caller surfaces consume the shared contract or the persisted attach truth derived from it;
6. orchestrator-controlled dispatch consumes the same shared contract without a REPL-only dialect;
7. docs and `llm-last-mile/` truth surfaces explain the same contract and point slices 30 and 31 at it;
8. targeted tests and full workspace gates pass on the same merged tree.

Frozen execution shape:

1. `P0` parent freeze:
   - lock the current working-tree `PLAN.md` contents,
   - freeze contract vocabulary,
   - freeze durable attach schema,
   - write lane ownership and merge order under `.runs/`.
2. `L0` executes `A1 + A2` on one branch:
   - shared resolver module,
   - baseline projection helpers,
   - runtime materialization convergence.
3. Parent integrates `L0` and closes the contract/mapping hotspot before opening durable-state work.
4. `L1` executes `A3` on one branch:
   - generalized persisted host attach contract,
   - state-store consumption of persisted resolved truth.
5. Parent integrates `L1` and freezes the durable-state schema.
6. Only then does the safe parallel window open:
   - `L2` executes `A4` human caller adoption.
   - `L3` executes `A5` orchestrator-controlled dispatch adoption.
7. Parent merges `L2` first, then merges `L3` onto the accepted runtime tree.
8. `L4` executes `A6` docs and downstream truth-sync only after both runtime lanes are accepted.
9. Parent executes `P1` / `A7` validation wall and `G5` final acceptance last.

Default rule: there is exactly one safe parallel window in this slice, and it is `L2 + L3` after the contract vocabulary and durable-state schema are frozen. No other overlap is treated as safe.

## Hard Guards

These are run-stopping invariants.

1. The authoritative integration checkout remains `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate` on `feat/gateway-mediated-llm-fulfillment`.
2. The parent is the only integrator and the only approval authority.
3. The parent is the only writer of `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/slice-29-shared-dispatch-contract/**`.
4. `PLAN.md` is currently dirty in the authoritative checkout; the parent must source-lock the current file contents and must not revert or normalize that drift.
5. No worker edits `PLAN.md`, `ORCH_PLAN.md`, or `.runs/**`.
6. `A0` is operationally satisfied by parent-owned freeze artifacts before any worker branch opens; workers do not reopen contract vocabulary.
7. `dispatch_contract.rs` is created and owned only by `L0`.
8. `crates/shell/src/execution/agent_inventory.rs`, `validator.rs`, and `control.rs` are serialized hotspots during `L0`; no other lane touches them before `G1` passes.
9. `crates/shell/src/execution/agent_runtime/orchestration_session.rs` and `state_store.rs` are serialized durable-state hotspots during `L1`; no parallel lane touches them before `G2` passes.
10. `crates/shell/src/execution/agent_runtime/control.rs` is a single-owner hotspot:
    - `L0` owns it during `A2`,
    - `L2` owns it during `A4`,
    - no overlap is allowed.
11. `crates/shell/src/execution/agents_cmd.rs` is a single-owner hotspot and belongs only to `L2` in this run.
12. `crates/shell/src/repl/async_repl.rs` and optional additive `routing/dispatch/world_ops.rs` belong only to `L3` in this run.
13. Docs and `llm-last-mile/` truth surfaces belong only to `L4`; runtime lanes do not edit docs.
14. Policy remains narrowing-only and fail-closed.
15. Persisted attach truth remains `HostAttachContract`; no second durable attach object may be invented.
16. Persisted JSON changes must remain additive or migration-safe through `OrchestrationSessionRecord::validate_persisted_invariants(...)`.
17. Equivalent inventory-backed CLI and REPL inputs must resolve to equivalent backend, scope, capability, and attach-knob truth.
18. Unknown override families, unsupported overrides, baseline broadening, invalid policy overlays, policy denials, unrealizable runtime results, and missing required continuity all fail closed.
19. No worker may broaden public scope or invent public CLI flags beyond what [PLAN.md](PLAN.md) allows.
20. `world_ops.rs` may be edited only if `L3` proves an additive transport field is required by a concrete missing contract field.
21. Every worker must run GitNexus impact analysis before editing any function, method, or other symbol.
22. Any `HIGH` or `CRITICAL` GitNexus impact result must be escalated to the parent before edits proceed.
23. Every worker handoff must include GitNexus `detect_changes` output.
24. The parent must run a final merged-tree GitNexus `detect_changes` pass before final acceptance.
25. If the source-locked `PLAN.md` content changes materially during execution, stop the run and reopen from `P0`.

## Blocked-Run Conditions

Stop the run, write `blocked.json`, and do not advance if any of these occur:

1. `L0` cannot express the shared contract without inventing a second contract owner outside `dispatch_contract.rs`.
2. `L0` requires `A4` or `A5` files to define the core contract vocabulary or merge precedence.
3. `L1` cannot generalize `HostAttachContract` without a non-additive persisted-state break or a second durable attach model.
4. `L1` cannot derive successor attach truth while clearing only continuity-specific state.
5. `L2` can only make human caller adoption pass by re-deriving attach truth from ambient participant state.
6. `L3` can only make REPL parity pass by reopening `dispatch_contract.rs`, `agent_inventory.rs`, `validator.rs`, `control.rs`, `orchestration_session.rs`, or `state_store.rs`.
7. `L3` requires a breaking wire-contract change in `MemberDispatchTransportRequest`.
8. Equivalent CLI and REPL inventory-backed inputs still produce different resolved contract fields after `L2` and `L3`.
9. Docs can only be made truthful by contradicting the merged code.
10. The validation wall cannot prove targeted contract coverage, persistence coverage, parity coverage, and full workspace gates on the same merged tree.
11. Any lane edits files outside its frozen ownership set and the parent cannot cleanly quarantine the output.
12. `PLAN.md` source lock changes after `G0` and the current run cannot be trusted against the new plan text.

`blocked.json` must include:

1. `run_id`
2. `authoritative_branch`
3. `plan_source`
4. `source_lock_sha256`
5. `timestamp`
6. `current_gate`
7. `current_task`
8. `stop_condition`
9. `blocking_files`
10. `accepted_tasks`
11. `rejected_or_quarantined_tasks`
12. `required_parent_action`

## Fresh Worktrees And Branches

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract`

Authoritative integration checkout:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/gateway-mediated-llm-fulfillment`

Worker worktrees:

- `L0` `A1+A2` contract foundation:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a1-a2-contract-foundation`
  - `codex/feat-gateway-mediated-llm-fulfillment-s29-a1-a2-contract-foundation`
- `L1` `A3` durable attach freeze:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a3-durable-attach-freeze`
  - `codex/feat-gateway-mediated-llm-fulfillment-s29-a3-durable-attach-freeze`
- `L2` `A4` human caller adoption:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a4-human-caller-adoption`
  - `codex/feat-gateway-mediated-llm-fulfillment-s29-a4-human-caller-adoption`
- `L3` `A5` repl dispatch adoption:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a5-repl-dispatch-adoption`
  - `codex/feat-gateway-mediated-llm-fulfillment-s29-a5-repl-dispatch-adoption`
- `L4` `A6` docs truth-sync:
  - `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a6-docs-truth-sync`
  - `codex/feat-gateway-mediated-llm-fulfillment-s29-a6-docs-truth-sync`

Exact setup commands:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate fetch origin
```

Create `L0` only after `G0` passes:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a1-a2-contract-foundation \
  -b codex/feat-gateway-mediated-llm-fulfillment-s29-a1-a2-contract-foundation \
  feat/gateway-mediated-llm-fulfillment
```

Create `L1` only after `G1` passes. The parent must first capture the accepted authoritative commit SHA from the integrated tree and record it in `branch-map.json` as `accepted_tip_after_G1`:

```bash
ACCEPTED_TIP_AFTER_G1="$(git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate rev-parse HEAD)"

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a3-durable-attach-freeze \
  -b codex/feat-gateway-mediated-llm-fulfillment-s29-a3-durable-attach-freeze \
  "$ACCEPTED_TIP_AFTER_G1"
```

Create `L2` and `L3` only after `G2` passes. The parent must first capture the accepted authoritative commit SHA from the integrated tree and record it in `branch-map.json` as `accepted_tip_after_G2`. Both lanes must branch from that same accepted SHA:

```bash
ACCEPTED_TIP_AFTER_G2="$(git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate rev-parse HEAD)"

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a4-human-caller-adoption \
  -b codex/feat-gateway-mediated-llm-fulfillment-s29-a4-human-caller-adoption \
  "$ACCEPTED_TIP_AFTER_G2"

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a5-repl-dispatch-adoption \
  -b codex/feat-gateway-mediated-llm-fulfillment-s29-a5-repl-dispatch-adoption \
  "$ACCEPTED_TIP_AFTER_G2"
```

Create `L4` only after `G3` passes. The parent must first capture the accepted authoritative commit SHA from the integrated tree and record it in `branch-map.json` as `accepted_tip_after_G3`:

```bash
ACCEPTED_TIP_AFTER_G3="$(git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate rev-parse HEAD)"

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a6-docs-truth-sync \
  -b codex/feat-gateway-mediated-llm-fulfillment-s29-a6-docs-truth-sync \
  "$ACCEPTED_TIP_AFTER_G3"
```

Concurrency contract:

1. Parent freeze and all gates are serialized.
2. `L0` and `L1` are serialized.
3. Peak safe parallelism is exactly `L2 + L3`.
4. `L4` is serialized after both runtime lanes are accepted.
5. Worker cap is `2`; do not open a third live worker.

## Parent-Owned Run-State Surface

Canonical run root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/slice-29-shared-dispatch-contract/`

Required top-level artifacts:

- `run-state.json`
- `tasks.json`
- `source-lock.json`
- `contract-freeze.json`
- `durable-state-freeze.json`
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

- `tasks/P0-parent-freeze/`
- `tasks/L0-a1-a2-contract-foundation/`
- `tasks/G1-contract-foundation-accept/`
- `tasks/L1-a3-durable-attach-freeze/`
- `tasks/G2-durable-attach-accept/`
- `tasks/L2-a4-human-caller-adoption/`
- `tasks/L3-a5-repl-dispatch-adoption/`
- `tasks/G3-parallel-window-accept/`
- `tasks/L4-a6-docs-truth-sync/`
- `tasks/G4-docs-accept/`
- `tasks/P1-a7-validation-wall/`
- `tasks/G5-final-acceptance/`
- `tasks/P2-parent-closeout/`

Required gate directories:

- `gates/G0-run-freeze/`
- `gates/G1-contract-foundation/`
- `gates/G2-durable-attach/`
- `gates/G3-parallel-window/`
- `gates/G4-docs/`
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
2. Workers return branch, head SHA, changed files, commands run, exit codes, tests, blockers, GitNexus impact summaries, and `detect_changes` output to the parent.
3. The parent writes or updates every task and gate artifact before a task or gate changes state.
4. Nothing is accepted until the parent has transcribed the handoff and written the sentinel.
5. `source-lock.json` must record at minimum:
   - authoritative branch,
   - authoritative HEAD SHA,
   - `PLAN.md` absolute path,
   - `PLAN.md` content hash from the current working tree,
   - `git status --short PLAN.md ORCH_PLAN.md`,
   - timestamp.
6. `contract-freeze.json` must record at minimum:
   - `DispatchRequestEnvelope`,
   - `DispatchCapabilityOverrideSet`,
   - `AttachLaunchKnobs`,
   - `ResolvedLaunchContract`,
   - `FieldProvenance`,
   - `DispatchResolutionError`,
   - baseline kinds,
   - caller kinds,
   - capability families,
   - attach-knob vocabulary,
   - merge precedence,
   - fail-closed categories.
7. `durable-state-freeze.json` must record at minimum:
   - persisted `HostAttachContract` ownership,
   - fields required for later continuity attach and fresh attach,
   - continuity-clearing rule for successor copy,
   - additive/migration-safe constraint,
   - prohibition on persisting full provenance trees.
8. `branch-map.json` must record at minimum:
   - `authoritative_branch`,
   - `authoritative_head_at_P0`,
   - `accepted_tip_after_G1`,
   - `accepted_tip_after_G2`,
   - `accepted_tip_after_G3`,
   - lane branch names,
   - lane base SHAs,
   - replay target SHA for any replayed lane.

## Parent-Only Critical Path

The parent-only critical path is fixed:

1. `P0` freeze the run:
   - confirm branch,
   - confirm the source-locked `PLAN.md`,
   - write contract and durable-state freezes,
   - write lane ownership,
   - write merge order,
   - open `RUN_OPEN`.
2. `G0` authorize `L0`.
3. Review and integrate `L0` `A1+A2`.
4. `G1` authorize `L1`.
5. Review and integrate `L1` `A3`.
6. `G2` authorize the only parallel window:
   - `L2` `A4`
   - `L3` `A5`
7. Review and integrate `L2` first.
8. Replay and integrate `L3` onto the accepted `L2` tree if needed.
9. `G3` authorize `L4`.
10. Review and integrate `L4` `A6`.
11. `P1` execute `A7` validation wall.
12. `G5` issue final acceptance or block.
13. `P2` write closeout and `RUN_COMPLETE`.

The parent never delegates:

1. source-lock updates,
2. contract vocabulary freeze,
3. durable-state schema freeze,
4. hotspot ownership transfer,
5. merge decisions,
6. gate changes,
7. blocked-run decisions,
8. final validation,
9. final acceptance.

## Task And Gate Decomposition

| Task | PLAN phases | Owner | Starts after | Ends at | Purpose |
| --- | --- | --- | --- | --- | --- |
| `P0` | `A0` operational freeze | parent | run start | `G0` | freeze vocabulary and durable-state schema before any worker opens |
| `L0` | `A1`, `A2` | worker | `G0` | `G1` review | land shared resolver and downstream runtime materialization boundary |
| `L1` | `A3` | worker | `G1` | `G2` review | generalize persisted host attach truth and state-store consumption |
| `L2` | `A4` | worker | `G2` | `G3` review | adopt the contract in human caller surfaces |
| `L3` | `A5` | worker | `G2` | `G3` review | adopt the contract in orchestrator-controlled dispatch |
| `L4` | `A6` | worker | `G3` | `G4` review | truth-sync docs and downstream slices to merged runtime truth |
| `P1` | `A7` | parent | `G4` | `G5` | validation wall on the merged tree |
| `P2` | closeout | parent | `G5` | run end | final summary, sentinels, and artifacts |

### Gate Pass Criteria

`G0` Run Freeze passes only if:

1. branch is `feat/gateway-mediated-llm-fulfillment`;
2. `PLAN.md` working-tree contents are hashed and source-locked;
3. `contract-freeze.json` and `durable-state-freeze.json` are written;
4. lane ownership and merge order are written;
5. no worker worktree exists yet.

`G1` Contract Foundation passes only if `L0` proves:

1. `dispatch_contract.rs` exists and is exported from `agent_runtime/mod.rs`;
2. inventory-backed and persisted-attach-backed baseline domains are explicit;
3. override validation, provenance, and denial taxonomy live in the shared contract layer;
4. `validator.rs` and `control.rs` consume `ResolvedLaunchContract`-style output instead of acting as top-level merge owners;
5. new unit coverage exists for baseline projection, denial, narrowing, and runtime-unrealizable cases.

`G2` Durable Attach passes only if `L1` proves:

1. session birth persists generalized host attach truth derived from the resolved host launch contract;
2. successor copy preserves launch truth while clearing only continuity-specific state;
3. `state_store.rs` uses persisted attach truth rather than ambient participant state for detached attach planning;
4. persisted-state validation remains additive and fail-closed;
5. the durable-state schema freeze can be finalized without reopening `L0`.

`G3` Parallel Window passes only if:

1. `L2` proves human `start` uses the inventory-backed resolver;
2. `L2` proves `reattach`, detached-turn attach planning, and `fork` use persisted attach truth derived from the resolver;
3. `L2` denial messages name field + layer + reason;
4. `L3` proves orchestrator-controlled member dispatch consumes the same inventory-backed resolution semantics;
5. `L3` only touches `world_ops.rs` if a concrete additive field was required;
6. neither lane reopened contract vocabulary or durable-state schema.

`G4` Docs passes only if `L4` proves:

1. ADR-0027, slice 29, slice 30, slice 31, and `llm-last-mile/README.md` explain the same contract once;
2. downstream slices reference the frozen knob vocabulary instead of inventing synonyms;
3. no doc still implies attach truth can be reconstructed from the last live participant.

`G5` Final passes only if the merged tree proves:

1. all acceptance criteria below are true;
2. targeted tests are green;
3. full workspace gates are green;
4. docs match the merged code;
5. final GitNexus `detect_changes` review is transcribed.

## Workstream Execution Briefs

### `P0` Parent Freeze

Exact purpose:

1. source-lock the current working-tree `PLAN.md`;
2. freeze contract vocabulary and durable-state schema before any worker opens;
3. initialize `.runs/` state, lane ownership, merge order, and branch mapping.

Exact owned files / hotspots:

1. `.runs/slice-29-shared-dispatch-contract/**`
2. parent-owned authoritative checkout metadata only

Exact forbidden surfaces:

1. all runtime code files
2. all docs files
3. worker worktrees before `G0`

Required commands or tests:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate status --short --branch
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate rev-parse HEAD
shasum -a 256 /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md
```

Lane acceptance outcomes:

1. `source-lock.json` records the current `PLAN.md` content hash and authoritative `HEAD`.
2. `contract-freeze.json` records the frozen slice-29 contract vocabulary.
3. `durable-state-freeze.json` records the frozen attach-contract persistence rules.
4. `branch-map.json`, `lane-ownership.json`, and `merge-order.json` are written.
5. `G0` can open `L0`.

### `L0` Contract Foundation

Exact purpose:

1. land the shared dispatch contract owner;
2. project baseline truth into that contract;
3. move runtime materialization under the resolved contract boundary.

Exact owned files / hotspots:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/agent_runtime/mod.rs`
3. `crates/shell/src/execution/agent_inventory.rs`
4. `crates/shell/src/execution/agent_runtime/validator.rs`
5. `crates/shell/src/execution/agent_runtime/control.rs`
6. directly related unit tests

Exact forbidden surfaces:

1. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agents_cmd.rs`
4. `crates/shell/src/repl/async_repl.rs`
5. `crates/shell/src/execution/routing/dispatch/world_ops.rs`
6. all docs surfaces
7. `.runs/**`

Required commands or tests:

1. GitNexus impact analysis for every edited symbol in the owned files.
2. Parent-resolved focused unit selectors for `dispatch_contract.rs`, `validator.rs`, and `control.rs`.

Candidate command shape:

```bash
cargo test -p shell <parent-resolved-dispatch-contract-selector> -- --nocapture
cargo test -p shell <parent-resolved-validator-control-selector> -- --nocapture
```

Lane acceptance outcomes:

1. `dispatch_contract.rs` exists and is exported.
2. baseline kinds, override families, provenance, and denial taxonomy are owned in one place.
3. `validator.rs` and `control.rs` consume resolved-contract output rather than owning merge semantics.
4. focused `L0` tests pass.
5. parent records `accepted_tip_after_G1` after integrating `L0`.

### `L1` Durable Attach Freeze

Exact purpose:

1. generalize persisted `HostAttachContract` from the resolved host launch contract;
2. freeze successor-copy semantics;
3. make detached attach planning consume persisted resolved truth.

Exact owned files / hotspots:

1. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. directly related unit tests

Exact forbidden surfaces:

1. `dispatch_contract.rs`
2. `agent_inventory.rs`
3. `validator.rs`
4. `control.rs`
5. `agents_cmd.rs`
6. `async_repl.rs`
7. `world_ops.rs`
8. all docs surfaces
9. `.runs/**`

Required commands or tests:

1. GitNexus impact analysis for every edited symbol in the owned files.
2. Parent-resolved focused selector for generalized attach-contract invariant coverage.
3. Locked control-target and state-store checks:

```bash
cargo test -p shell resolve_public_control_target -- --nocapture
cargo test -p shell new_session_starts_active_attached -- --nocapture
cargo test -p shell detached_postures_enforce_pending_inbox_truth -- --nocapture
```

Lane acceptance outcomes:

1. session birth persists generalized host attach truth derived from the resolved contract.
2. successor copy preserves launch truth and clears only continuity-specific state.
3. detached attach planning consumes persisted truth rather than ambient participant state.
4. persisted-state validation remains additive and fail-closed.
5. parent records `accepted_tip_after_G2` after integrating `L1`.

### `L2` Human Caller Adoption

Exact purpose:

1. adopt the shared contract in human caller surfaces;
2. route `start`, `reattach`, detached-turn attach planning, and `fork` through the correct resolved or persisted truth;
3. preserve explicit denial messaging.

Exact owned files / hotspots:

1. `crates/shell/src/execution/agents_cmd.rs`
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. `crates/shell/src/execution/prompt_fulfillment.rs`
4. `crates/shell/tests/agent_public_control_surface_v1.rs`
5. minimal supporting tests

Exact forbidden surfaces:

1. `dispatch_contract.rs`
2. `agent_inventory.rs`
3. `validator.rs`
4. `orchestration_session.rs`
5. `state_store.rs`
6. `async_repl.rs`
7. `world_ops.rs`
8. all docs surfaces
9. `.runs/**`

Required commands or tests:

1. GitNexus impact analysis for every edited symbol in the owned files.
2. Locked caller-surface checks:

```bash
cargo test -p shell public_turn_prompt_requests_require_exact_session_and_backend_contract -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Lane acceptance outcomes:

1. host `start` uses the inventory-backed resolver.
2. `reattach`, detached-turn attach planning, and `fork` consume persisted attach truth derived from the resolver.
3. denial messages name field, rejecting layer, and reason.
4. no out-of-scope edits or hotspot violations exist.
5. if out-of-scope edits, contract drift, or hotspot violations are found, the parent rejects and quarantines `L2`, marks the lane task `REJECTED`, records the reason in task artifacts, and requires a rerun from `accepted_tip_after_G2` before any merge.

### `L3` REPL Dispatch Adoption

Exact purpose:

1. adopt the shared contract in orchestrator-controlled dispatch;
2. prove REPL parity with CLI inventory-backed launch semantics;
3. touch transport only if a concrete additive field is required.

Exact owned files / hotspots:

1. `crates/shell/src/repl/async_repl.rs`
2. `crates/shell/src/execution/routing/dispatch/world_ops.rs` only if additive and required
3. `crates/shell/tests/repl_world_first_routing_v1.rs`
4. minimal supporting tests

Exact forbidden surfaces:

1. `dispatch_contract.rs`
2. `agent_inventory.rs`
3. `validator.rs`
4. `control.rs`
5. `agents_cmd.rs`
6. `orchestration_session.rs`
7. `state_store.rs`
8. all docs surfaces
9. `.runs/**`

Required commands or tests:

1. GitNexus impact analysis for every edited symbol in the owned files.
2. Locked parity check:

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Lane acceptance outcomes:

1. world-member dispatch consumes the same inventory-backed resolution semantics as CLI launch.
2. any `world_ops.rs` change is additive and justified by a concrete missing field.
3. no second REPL-only launch dialect survives.
4. no out-of-scope edits or hotspot violations exist.
5. if out-of-scope edits, contract drift, hotspot violations, or forbidden contract/durable-state reopenings are found, the parent rejects and quarantines `L3`, marks the lane task `REJECTED`, records the reason in task artifacts, and requires a rerun from `accepted_tip_after_G2` or replay onto the current accepted authoritative SHA before merge.

### `L4` Docs Truth-Sync

Exact purpose:

1. truth-sync docs to the accepted runtime tree;
2. publish the contract once and point downstream slices to it;
3. keep adjacent ADR surfaces stable unless contradiction is discovered.

Exact owned files / hotspots:

1. `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
2. `llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`
3. `llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md`
4. `llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md`
5. `llm-last-mile/README.md`

Exact forbidden surfaces:

1. all Rust files
2. all test files
3. `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md` unless a contradiction to merged runtime truth is discovered and the parent explicitly reopens scope
4. `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md` unless a contradiction to merged runtime truth is discovered and the parent explicitly reopens scope
5. `.runs/**`

Required commands or tests:

1. diff review against accepted runtime files and merged semantics from `G3`
2. grep or text checks as needed to confirm no stale attach-truth or override-vocabulary wording remains

Candidate command shape:

```bash
rg -n "last live participant|reconstruct|inventing synonyms|scope world|attach-mode" \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile
```

Lane acceptance outcomes:

1. ADR-0027, slice 29, slice 30, slice 31, and `llm-last-mile/README.md` match merged runtime truth.
2. ADR-0025 and ADR-0026 remain read-only unless a contradiction is explicitly proven.
3. no doc reintroduces stale attach-truth reconstruction semantics.
4. if docs contradict merged runtime truth, the parent rejects and quarantines `L4`, marks the lane task `REJECTED`, records the contradiction in task artifacts, and requires a rerun from `accepted_tip_after_G3` before merge.

### `P1` Validation Wall

Exact purpose:

1. run the final focused and full-repo validation on one merged tree;
2. verify caller parity, persistence truth, and docs alignment;
3. decide final acceptance or block.

Exact owned files / hotspots:

1. `.runs/slice-29-shared-dispatch-contract/validation-wall.md`
2. parent-owned acceptance artifacts only

Exact forbidden surfaces:

1. all runtime and docs edits unless the parent explicitly blocks and reopens a lane

Required commands or tests:

```bash
cargo test -p shell resolve_public_control_target -- --nocapture
cargo test -p shell public_turn_prompt_requests_require_exact_session_and_backend_contract -- --nocapture
cargo test -p shell new_session_starts_active_attached -- --nocapture
cargo test -p shell detached_postures_enforce_pending_inbox_truth -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Lane acceptance outcomes:

1. focused contract, persistence, caller-surface, and parity coverage is green.
2. full workspace gates are green.
3. final GitNexus `detect_changes` pass is recorded.
4. `G5` can pass or the run blocks with evidence.

### `P2` Parent Closeout

Exact purpose:

1. finalize the run-state record;
2. record what merged, what was rejected, and what was quarantined;
3. close the controller with `RUN_COMPLETE` or `RUN_BLOCKED`.

Exact owned files / hotspots:

1. `.runs/slice-29-shared-dispatch-contract/final-summary.md`
2. `.runs/slice-29-shared-dispatch-contract/session-log.md`
3. `.runs/slice-29-shared-dispatch-contract/tasks/**`
4. `.runs/slice-29-shared-dispatch-contract/gates/**`
5. `.runs/slice-29-shared-dispatch-contract/sentinels/**`

Exact forbidden surfaces:

1. all product code files
2. all docs files

Required commands or tests:

1. final artifact completeness review
2. final branch and SHA transcription review

Lane acceptance outcomes:

1. accepted lanes, rejected lanes, and quarantined lanes are all transcribed.
2. `final-summary.md` states the final authoritative SHA and merge order actually used.
3. `RUN_COMPLETE` or `RUN_BLOCKED` is written exactly once.

## Lane Ownership And Hotspots

### Lane Ledger

| Lane | PLAN phases | Purpose | Worker cap impact |
| --- | --- | --- | --- |
| `L0` | `A1+A2` | shared contract owner and runtime materialization boundary | serialized |
| `L1` | `A3` | durable attach truth and state-store freeze | serialized |
| `L2` | `A4` | human caller adoption | parallel window slot 1 |
| `L3` | `A5` | REPL dispatch parity adoption | parallel window slot 2 |
| `L4` | `A6` | docs and truth-sync | serialized |

### File Ownership

| Lane | Allowed files | Forbidden touch surfaces |
| --- | --- | --- |
| `L0` | `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/agent_runtime/mod.rs`, `crates/shell/src/execution/agent_inventory.rs`, `crates/shell/src/execution/agent_runtime/validator.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, directly related unit tests | `orchestration_session.rs`, `state_store.rs`, `agents_cmd.rs`, `async_repl.rs`, `routing/dispatch/world_ops.rs`, docs, `.runs/**` |
| `L1` | `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, directly related unit tests | `dispatch_contract.rs`, `agent_inventory.rs`, `validator.rs`, `control.rs`, `agents_cmd.rs`, `async_repl.rs`, docs, `.runs/**` |
| `L2` | `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/prompt_fulfillment.rs`, `crates/shell/tests/agent_public_control_surface_v1.rs`, minimal supporting tests | `dispatch_contract.rs`, `agent_inventory.rs`, `validator.rs`, `orchestration_session.rs`, `state_store.rs`, `async_repl.rs`, `world_ops.rs`, docs, `.runs/**` |
| `L3` | `crates/shell/src/repl/async_repl.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs` only if additive and required, `crates/shell/tests/repl_world_first_routing_v1.rs`, minimal supporting tests | `dispatch_contract.rs`, `agent_inventory.rs`, `validator.rs`, `control.rs`, `agents_cmd.rs`, `orchestration_session.rs`, `state_store.rs`, docs, `.runs/**` |
| `L4` | `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`, `llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`, `llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md`, `llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md`, `llm-last-mile/README.md` | all Rust files, all tests, `.runs/**` |

Hotspot transfer rules:

1. `dispatch_contract.rs` never leaves `L0`.
2. `agent_inventory.rs` and `validator.rs` never reopen after `G1` unless the parent blocks and restarts from `P0`.
3. `control.rs` transfers from `L0` to `L2` only after `G1` passes.
4. `orchestration_session.rs` and `state_store.rs` never reopen after `G2` unless the parent blocks and restarts from `P0`.
5. `agents_cmd.rs` belongs only to `L2`.
6. `async_repl.rs` belongs only to `L3`.
7. `world_ops.rs` is optional and additive-only; if touched, it remains `L3`-owned for the duration of the run.
8. Docs never merge before runtime truth is accepted through `G3`.
9. Any lane that violates hotspot ownership is rejected and quarantined until the parent records a corrected rerun from the appropriate accepted authoritative SHA.

## Merge-Order Protocol

The merge order is fixed unless the parent explicitly blocks and restarts:

1. merge `L0` `A1+A2`;
2. merge `L1` `A3`;
3. open the only parallel window;
4. merge `L2` `A4`;
5. replay and merge `L3` `A5` onto the accepted `L2` tree if needed;
6. create `L4` from the current authoritative tip;
7. merge `L4` `A6`;
8. run `P1` validation wall;
9. run `P2` closeout.

Accepted-tip SHA protocol:

1. `L0` may branch from `feat/gateway-mediated-llm-fulfillment` after `G0`.
2. After integrating `L0`, the parent records `accepted_tip_after_G1` and `L1` must branch from that exact SHA.
3. After integrating `L1`, the parent records `accepted_tip_after_G2` and both `L2` and `L3` must branch from that same exact SHA.
4. If `L3` is replayed after `L2` merge, the replay target SHA is the accepted authoritative SHA after `L2` integration and must be recorded in `branch-map.json`.
5. After integrating `L2` and `L3`, the parent records `accepted_tip_after_G3` and `L4` must branch from that exact SHA.
6. No downstream lane may be opened from a moving branch ref when the accepted authoritative SHA is already known.

Why this order is fixed:

1. contract vocabulary and merge precedence must stabilize before downstream adoption.
2. runtime materialization must stabilize before persisted attach truth is generalized.
3. durable attach truth must stabilize before any caller surface can safely consume it.
4. human and REPL adoption are the only safe independent consumers once the contract and durable schema are frozen.
5. docs must trail accepted runtime truth, not speculate ahead of it.
6. rejected or quarantined lanes must rerun from the last accepted authoritative SHA, not from a stale lane base.

## Worker Prompt Scaffolding Requirements

Every worker prompt must contain all of the following:

1. lane id, worktree path, branch name, and expected base SHA;
2. exact allowed files and explicit forbidden files;
3. the frozen contract vocabulary from `contract-freeze.json`;
4. the frozen durable-state rules from `durable-state-freeze.json` when relevant;
5. the locked merge order and whether the lane is serialized or parallel;
6. the GitNexus requirement:
   - run impact analysis before editing every symbol,
   - escalate `HIGH` or `CRITICAL` blast radius before proceeding,
   - run `detect_changes` before handoff;
7. the lane-specific required tests and commands;
8. the worker return contract below;
9. the instruction that the parent is the sole integrator and `.runs/**` owner.

Minimum prompt scaffold:

```text
Lane:
Goal:
Allowed files:
Forbidden files:
Frozen contract points:
Frozen durable-state points:
Required tests:
Required GitNexus steps:
Return contract:
Block immediately if:
```

### Worker Return Contract

Every worker returns all of the following to the parent:

1. branch name;
2. head SHA;
3. changed file list;
4. concise summary of what changed;
5. commands run;
6. exit codes;
7. tests run and results;
8. GitNexus impact-analysis summary for every edited symbol;
9. GitNexus `detect_changes` output;
10. unresolved blockers or uncertainties;
11. explicit statement that no out-of-scope files were edited.

Parent acceptance checklist for every worker:

1. compare changed files against lane ownership;
2. verify the branch tip is based on the expected authoritative tip;
3. confirm GitNexus impact analysis was run before every edited symbol;
4. explicitly accept or reject any `HIGH` or `CRITICAL` blast radius;
5. review for contract drift or durable-state drift;
6. transcribe `impact-analysis-summary.md` and `gitnexus-detect-changes.txt`;
7. if out-of-scope edits, hotspot violations, contract drift, durable-state drift, or contradictory docs are found:
   - mark the lane task `REJECTED`,
   - quarantine the lane branch/worktree,
   - record the rejection reason and required rerun base SHA in task artifacts,
   - do not merge the lane;
8. only merge accepted lanes;
9. update task and gate state.

## Validation Wall

### Parent-Resolved Unit Selectors

The parent must resolve the exact current-tree selectors before running focused unit tests for:

1. `dispatch_contract.rs` baseline-domain, override-denial, narrowing-provenance, and runtime-unrealizable coverage;
2. `validator.rs` / `control.rs` materialization-from-resolved-contract coverage;
3. `orchestration_session.rs` generalized attach-contract invariants and successor-copy continuity clearing.

The chosen selectors must be written to the relevant `commands.txt` before execution. Placeholder strings are not canonical if current test names differ.

### Locked Commands

Targeted state-store and control coverage:

```bash
cargo test -p shell resolve_public_control_target -- --nocapture
cargo test -p shell public_turn_prompt_requests_require_exact_session_and_backend_contract -- --nocapture
cargo test -p shell new_session_starts_active_attached -- --nocapture
cargo test -p shell detached_postures_enforce_pending_inbox_truth -- --nocapture
```

Caller-surface and parity coverage:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Workspace gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

### Manual Proof Points

Parent closeout must record operator-legible proof for all of the following:

1. inventory-backed defaults resolve deterministically from effective inventory plus effective config;
2. persisted attach truth is used for `reattach`, `fork`, and detached-turn attach planning;
3. supported overrides narrow or select the effective launch contract correctly;
4. policy-denied overrides fail closed with field-specific reasons;
5. persisted host attach truth matches the resolved host launch contract;
6. equivalent human and orchestrator-controlled inventory-backed inputs resolve to equivalent launch truth;
7. downstream slices 30 and 31 point at the frozen contract instead of inventing their own semantics.

## Final Acceptance

The controller may close only if all of the following are true on the merged authoritative tree:

1. one shared internal dispatch request envelope exists.
2. one resolved launch contract exists and owns merge semantics.
3. inventory-backed and persisted-attach-backed baseline domains are explicit.
4. supported override families are explicit, bounded, and fail closed on unknown input.
5. policy narrowing is deterministic, auditable, and never broadens.
6. runtime materialization is downstream of the resolved contract.
7. `HostAttachContract` persists generalized host launch truth without a second durable attach object.
8. successor attach-contract copy clears continuity while preserving exact launch truth.
9. human `start` uses the inventory-backed resolver.
10. `reattach`, detached-turn attach planning, and `fork` consume persisted attach truth derived from the resolver.
11. orchestrator-controlled world-member dispatch consumes the same shared resolver semantics.
12. no caller-specific launch dialect survives in CLI or REPL code.
13. docs publish baseline domains, merge precedence, override families, denial taxonomy, and downstream dependencies exactly once.
14. slices 30 and 31 reference the frozen contract instead of inventing synonyms.
15. targeted contract, persistence, caller-surface, and parity tests are green.
16. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --nocapture` are green.
17. every edited symbol had GitNexus impact analysis recorded and any `HIGH` or `CRITICAL` result was explicitly accepted or blocked by the parent.
18. the parent has reviewed worker `detect_changes` outputs and run a final merged-tree `detect_changes` pass.
19. closeout artifacts are written and `RUN_COMPLETE` exists.

## Assumptions And Chosen Gaps

1. The current working-tree `PLAN.md` content, not necessarily `HEAD`, is the authoritative slice brief for this run.
2. The controller treats `A0` as a parent-owned freeze step because no worker is permitted to edit plan/controller files.
3. The only safe parallel window is `A4 + A5`; docs truth-sync is intentionally serialized after runtime convergence.
4. `world_ops.rs` remains optional and additive-only because [PLAN.md](PLAN.md) allows transport changes only when a concrete missing field is proven.
5. If later implementation proves `A5` needs `control.rs` or durable-state files, this run blocks and restarts rather than widening the parallel window.
