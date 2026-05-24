# ORCH_PLAN: Shared Dispatch Contract Closeout And Parity Hardening

Authoritative plan source: [PLAN.md](PLAN.md)  
Controller file: [ORCH_PLAN.md](ORCH_PLAN.md)  
Authoritative branch: `feat/gateway-mediated-llm-fulfillment`  
Authoritative checkout: `/home/azureuser/__Active_Code/atomize-hq/substrate`  
Worktree root: `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout`  
Run id: `slice-29-5-shared-dispatch-closeout-parity-hardening`  
Parent role: sole integrator, sole gate owner, sole writer of `.runs/**`, sole owner of final acceptance

## Summary

This document executes the current [PLAN.md](PLAN.md). It is an execution controller, not a restatement of the plan.

This slice closes only when the merged tree proves one truthful contract floor across five ordered outcomes:

1. durable attach truth is persisted from resolved launch truth;
2. inventory `policy_overlay` is merged into real effective policy;
3. bounded capability override narrowing works for the approved family and fails closed for the rest;
4. retained member follow-up turns consume a shared-contract-derived subset instead of a hidden second dialect;
5. docs, tests, and final validation all confirm the same shipped semantics.

Execution shape is intentionally stricter than the stale controller:

1. parent freeze first;
2. durable attach truth first;
3. policy overlay merge second;
4. capability override closeout third;
5. only then open one narrow safe parallel window;
6. docs lane last;
7. final validation and acceptance remain parent-only.

## Hard Guards

1. The parent is the sole integrator, sole gate owner, sole writer of `.runs/**`, and sole final acceptance authority.
2. Workers must not edit `.runs/**`, `PLAN.md`, or `ORCH_PLAN.md`.
3. The authoritative checkout is already dirty because `PLAN.md` is modified; this run must not assume a clean tree and must not revert that drift.
4. The authoritative plan text is the current working-tree contents of `PLAN.md`, not whatever is present in a fresh worker worktree.
5. All worker worktrees must branch from committed gate tips only; the dirty plan/controller files stay parent-owned in the authoritative checkout.
6. `dispatch_contract.rs` is the primary hotspot and is serialized through the first three implementation phases.
7. `orchestration_session.rs` durable-state ownership is serialized before any parallel work opens.
8. `policy_model.rs` and `agent_inventory.rs` are serialized under the policy overlay phase only.
9. `state_store.rs`, `validator.rs`, and `agents_cmd.rs` are serialized under the capability override phase only.
10. `async_repl.rs` and `world_ops.rs` are retained-parity-only hotspots.
11. Docs and `llm-last-mile/` updates happen last, after runtime semantics are accepted.
12. Parent-owned freeze artifacts define contract vocabulary, durable-state schema, approved override matrix, and hotspot ownership before any worker may proceed.
13. Any need to reopen a frozen hotspot after its gate passes is a stop-and-reassess event, not an informal follow-up edit.
14. Honest parallelism only: no overlapping write lanes unless their file ownership is disjoint and their prerequisites are already frozen.
15. Parent acceptance is against merged-tree behavior and PLAN acceptance criteria, never against lane-local green tests alone.

## Blocked-Run Conditions

Stop the run and mark it blocked if any of the following become true:

1. Phase 1 cannot make `HostAttachContract` authoritative without introducing a second durable attach object.
2. Phase 2 cannot merge `policy_overlay` inside shared resolver semantics and instead requires caller-specific patch logic.
3. Phase 3 cannot implement the approved narrowing-only family without broadening scope, policy, or unsupported capability fields.
4. Retained parity work needs to reopen `dispatch_contract.rs`, `orchestration_session.rs`, `policy_model.rs`, `agent_inventory.rs`, `state_store.rs`, `validator.rs`, or `agents_cmd.rs` after those files are frozen.
5. Equivalent human and orchestrator cold starts still resolve to different contract truth after Phases 1 through 4 land.
6. Persisted attach flows still regain permissive defaults from ambient runtime state after Phase 1 or Phase 3.
7. Docs can only be made truthful by contradicting the merged runtime behavior.
8. The parent cannot prove the validation wall on the same merged tree that contains the final docs.
9. `PLAN.md` changes materially after the source lock is taken.
10. Any worker edits files outside its assigned lane ownership and the parent cannot quarantine the drift cleanly.

## Fresh Worktree And Branch Plan

All worktrees live under:

- `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout`

The parent integrates only in:

- `/home/azureuser/__Active_Code/atomize-hq/substrate`

The parent must record the committed base SHA before cutting worker worktrees:

```bash
BASE_HEAD="$(git -C /home/azureuser/__Active_Code/atomize-hq/substrate rev-parse HEAD)"
mkdir -p /home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout
```

Worker branches and cut points:

| Lane | Purpose | Worktree | Branch | Cut point |
| --- | --- | --- | --- | --- |
| `L1` | Phase 1 durable attach truth | `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout/p1-durable-attach-truth` | `codex/feat-gateway-mediated-llm-fulfillment-p1-durable-attach-truth` | `BASE_HEAD` |
| `L2` | Phase 2 policy overlay merge | `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout/p2-policy-overlay-merge` | `codex/feat-gateway-mediated-llm-fulfillment-p2-policy-overlay-merge` | accepted tip after `G1` |
| `L3` | Phase 3 capability override closeout | `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout/p3-capability-override-closeout` | `codex/feat-gateway-mediated-llm-fulfillment-p3-capability-override-closeout` | accepted tip after `G2` |
| `L4` | Phase 4 retained member parity | `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout/p4-retained-member-parity` | `codex/feat-gateway-mediated-llm-fulfillment-p4-retained-member-parity` | accepted tip after `G3` |
| `L4T` | Safe parallel regression lane for Phases 1 to 3 only | `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout/p4-phase123-regression-tests` | `codex/feat-gateway-mediated-llm-fulfillment-p4-phase123-regression-tests` | accepted tip after `G3` |
| `L5` | Phase 5 docs truth sync | `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-5-shared-dispatch-closeout/p5-docs-truth-sync` | `codex/feat-gateway-mediated-llm-fulfillment-p5-docs-truth-sync` | accepted tip after `G4` |

Branch cut rules:

1. `L2` is not created until `G1` passes.
2. `L3` is not created until `G2` passes.
3. `L4` and `L4T` are the only concurrent write lanes and both branch from the same accepted tip after `G3`.
4. `L5` is not created until `G4` passes and retained parity is already integrated.
5. Worker branches are short-lived and may not self-merge.

## Parent-Owned Run-State Surface

Canonical run root:

- `/home/azureuser/__Active_Code/atomize-hq/substrate/.runs/slice-29-5-shared-dispatch-closeout-parity-hardening/`

Parent-only artifacts:

1. `run-state.json`
2. `source-lock.json`
3. `source-lock/PLAN.authoritative.md`
4. `source-lock/ORCH_PLAN.authoritative.md`
5. `source-lock/git-status.txt`
6. `lane-ownership.json`
7. `hotspot-freeze.json`
8. `branch-map.json`
9. `merge-order.json`
10. `validation-wall.md`
11. `session-log.md`
12. `final-summary.md`
13. `blocked.json` when blocked

Required parent behavior at run open:

1. Record `git status --short` from the authoritative checkout before any worktree is cut.
2. Hash and snapshot the current dirty `PLAN.md` contents into `source-lock/PLAN.authoritative.md`.
3. Hash and snapshot the current `ORCH_PLAN.md` contents into `source-lock/ORCH_PLAN.authoritative.md`.
4. Record that workers must treat the source-lock snapshot as authoritative even if their clean worktrees contain older committed plan text.
5. Record lane ownership and hotspot freeze before assigning any worker task.

## Hotspot Ownership And Serialization

| File or area | Owner phase | Rule after gate |
| --- | --- | --- |
| `crates/shell/src/execution/agent_runtime/orchestration_session.rs` | Phase 1 | frozen after `G1` |
| `crates/shell/src/execution/agent_runtime/dispatch_contract.rs` | Phases 1 to 3 | frozen after `G3`; Phase 4 may not reopen it |
| `crates/shell/src/execution/agents_cmd.rs` | Phases 1 and 3 | frozen after `G3` |
| `crates/shell/src/execution/agent_runtime/control.rs` | parent review only if touched incidentally | no worker ownership drift |
| `crates/shell/src/execution/policy_model.rs` | Phase 2 | frozen after `G2` |
| `crates/shell/src/execution/agent_inventory.rs` | Phase 2 | frozen after `G2` |
| `crates/shell/src/execution/agent_runtime/state_store.rs` | Phase 3 | frozen after `G3` |
| `crates/shell/src/execution/agent_runtime/validator.rs` | Phase 3 | frozen after `G3` |
| `crates/shell/src/repl/async_repl.rs` | Phase 4 | frozen after `G4` |
| `crates/shell/src/execution/routing/dispatch/world_ops.rs` | Phase 4 if needed | frozen after `G4` |
| `crates/shell/tests/repl_world_first_routing_v1.rs` | Phase 4 | kept with retained-parity lane |
| `crates/shell/tests/agent_public_control_surface_v1.rs` | `L4T` or parent | no code-lane ownership |
| `crates/shell/tests/agent_successor_contract_ahcsitc0.rs` | `L4T` or parent | no code-lane ownership |
| `llm-last-mile/29*.md`, `30*.md`, `31*.md`, nearby runtime comments | Phase 5 | docs last only |

Hotspot rule:

1. If a later lane needs a frozen hotspot, stop and re-plan.
2. Do not "borrow" serialized hotspots to save time.
3. The safe parallel window exists only because the hotspots above stay closed after `G3`.

## Workstream Plan

### Phase 1: Durable Attach Truth First

Lane: `L1`

Scope:

1. derive `HostAttachContract` from resolved host launch truth;
2. persist attach-relevant capabilities and attach knobs from resolved truth;
3. keep sync continuity-only;
4. keep successor copy as generalized-truth plus cleared continuity only.

Owned files:

1. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/execution/agents_cmd.rs`

Gate `G1` must freeze:

1. resolved-to-attach conversion vocabulary;
2. durable attach schema fields;
3. successor-copy contract;
4. no ambient-state reconstruction rule.

### Phase 2: Policy Overlay Merge Second

Lane: `L2`

Scope:

1. expose or reuse one shared policy patch helper;
2. apply validated inventory `policy_overlay` into `ResolvedLaunchContract.effective_policy`;
3. return exact policy-layer diagnostics and provenance.

Owned files:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/shell/src/execution/agent_inventory.rs`

Gate `G2` must freeze:

1. one merge semantics source;
2. narrowing-only overlay behavior;
3. overlay acceptance and denial diagnostics.

### Phase 3: Capability Override Closeout Third

Lane: `L3`

Scope:

1. implement field-by-field override handling;
2. allow only the approved narrowing family:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, `event_stream`;
3. keep `session_start`, `llm`, and `mcp_client` rejected with field-scoped fail-closed diagnostics;
4. keep persisted attach launches rejecting dispatch-time capability overrides;
5. ensure persisted narrowed truth drives later state-store gates.

Owned files:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/validator.rs`
4. `crates/shell/src/execution/agents_cmd.rs`

Gate `G3` must freeze:

1. approved support matrix;
2. field-scoped denial wording;
3. persisted narrowed-truth expectations for later control flows;
4. final shared contract vocabulary for Phase 4.

### Safe Parallel Window: After `G3` Only

This is the only approved write-parallel window.

Lane `L4` retained member parity:

1. build the shared-contract-derived retained-turn subset;
2. feed typed transport from that subset;
3. keep retained follow-up turns off inventory and config re-resolution;
4. own parity-specific code and parity-specific integration coverage.

Owned files:

1. `crates/shell/src/repl/async_repl.rs`
2. `crates/shell/src/execution/routing/dispatch/world_ops.rs` if additive wiring is required
3. `crates/shell/tests/repl_world_first_routing_v1.rs`

Lane `L4T` regression coverage for frozen Phases 1 to 3:

1. add or extend tests for durable attach truth, overlay merge, override persistence, and successor truth;
2. do not edit any runtime hotspot files frozen at `G1`, `G2`, or `G3`;
3. do not edit parity lane files.

Owned files:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
3. other external shell test files only if parent assigns them explicitly

Parallel merge order:

1. integrate `L4` first;
2. replay or rebase `L4T` onto the accepted `G4` tip if needed;
3. if `L4T` needs a frozen runtime hotspot, stop and collapse the lane back to parent-owned follow-up work.

### Phase 5: Docs And Truth Sync Last

Lane: `L5`

Scope:

1. update slice docs so 29, 29.5, 30, and 31 agree on shipped semantics;
2. update nearby runtime comments or ASCII diagrams only if they were touched or made stale by merged code;
3. no runtime semantic changes in this lane.

Owned files:

1. `llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`
2. `llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md`
3. `llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md`
4. nearby comments or ASCII diagrams only if required for truth sync

Gate `G5` must confirm:

1. docs describe shipped semantics, not intended semantics;
2. downstream slices can cite 29.5 as the truthful floor;
3. no runtime file ownership is reopened.

## Task And Gate Sequencing

1. `P0` parent freeze
   - record dirty-tree state;
   - source-lock `PLAN.md` and `ORCH_PLAN.md`;
   - publish lane ownership, hotspot map, branch plan, and gate criteria.
2. `G0` run-open gate
   - parent confirms the frozen plan text and current branch context are stable enough to proceed.
3. `L1` Phase 1 durable attach truth
4. `G1` parent accepts or rejects Phase 1
5. `L2` Phase 2 policy overlay merge
6. `G2` parent accepts or rejects Phase 2
7. `L3` Phase 3 capability override closeout
8. `G3` parent accepts or rejects Phase 3 and freezes the shared contract vocabulary
9. `L4` retained member parity
10. `L4T` phase-1-to-3 regression tests in parallel
11. `G4` parent accepts retained parity first, then lands compatible regression coverage
12. `L5` docs truth sync
13. `G5` parent accepts docs truth sync
14. `P1` parent validation wall
15. `G6` final acceptance
16. `P2` parent closeout

## Context-Control Rules

1. Every worker brief must reference the parent-owned source-lock artifacts, not the worker worktree copy of `PLAN.md`.
2. Each lane gets only the files, acceptance criteria, and frozen assumptions it needs.
3. Workers must treat all frozen hotspot files outside their ownership set as read-only.
4. Workers must stop immediately on unexpected edits already present in their lane files.
5. Workers must not normalize formatting or touch unrelated comments outside their assignment.
6. Workers must hand back:
   - files changed,
   - tests run,
   - tests not run,
   - assumptions made,
   - any reason the frozen gate assumptions may no longer hold.
7. Parent must inspect diffs and re-check ownership boundaries before merging any lane.
8. Any request for broader context is routed through the parent so the run-state surface stays canonical.
9. If the parent sees PLAN drift, hotspot drift, or ownership drift, it pauses new worker starts until the freeze is re-issued.

## Tests And Acceptance

The validation wall is parent-only and must prove merged-tree acceptance, not lane-local completeness.

Minimum targeted commands:

```bash
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell agent_public_control_surface_v1 -- --nocapture
cargo test -p shell repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
```

Required package-level gates:

```bash
cargo test -p shell -- --nocapture
cargo clippy -p shell --all-targets -- -D warnings
```

Recommended hygiene gate before final acceptance:

```bash
cargo fmt --all -- --check
```

Merged-tree proof obligations:

1. host-scoped resolved launch truth persists into `HostAttachContract`;
2. persisted attach resolution reuses persisted capabilities and attach knobs instead of permissive defaults;
3. overlay-backed inventory resolution returns materially narrower `effective_policy`;
4. supported capability narrowing changes later attach and control behavior where the state store consumes persisted truth;
5. unsupported capability override fields fail closed with exact bounded reasons;
6. retained member follow-up turns use the shared-contract-derived subset and do not re-run hidden baseline selection;
7. equivalent human and orchestrator cold starts produce equivalent contract truth for backend, scope, capabilities, attach knobs, and effective policy;
8. docs for 29, 29.5, 30, and 31 all match the shipped behavior on the merged tree.

Acceptance rule:

1. A lane is not done because its own tests pass.
2. The slice is done only when the parent can check every proof obligation above on the final merged tree.

## Assumptions

1. The current working-tree `PLAN.md` rewrite is the authoritative planning source for this run.
2. The branch stays `feat/gateway-mediated-llm-fulfillment` for parent integration.
3. Worker worktrees can be created under `/home/azureuser/__Active_Code/atomize-hq/.worktrees/`.
4. The slice remains scoped to shared dispatch contract closeout and parity hardening, with no new public scope surface and no lazy-attach product work.
5. Parent can keep the dirty authoritative checkout intact while cutting clean worker worktrees from committed gate tips.
6. Any required `.runs/**` bookkeeping is parent-owned and can be updated without worker involvement.

## Completion Behavior

1. Parent merges accepted lanes in gate order only.
2. Parent runs the validation wall after docs land and before declaring completion.
3. Parent writes final run-state artifacts, including acceptance evidence and any residual risk notes.
4. Parent marks the run complete only after `G6` passes.
5. If blocked, parent writes `blocked.json`, leaves rejected worker branches unmerged, and records the exact stop condition.
6. Workers never self-certify completion, never write `.runs/**`, and never perform final acceptance.

## Completion Target

The target outcome is one merged tree where durable attach truth, overlay truth, override truth, retained member parity, and downstream docs all agree, with no hidden second contract dialect left for slices 30 or 31 to rediscover.
