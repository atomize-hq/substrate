# ORCH_PLAN: Gateway-Mediated LLM Fulfillment Execution Controller

Authoritative plan source: [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)  
Source SOW: [28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md)  
Current workspace branch: `feat/gateway-mediated-llm-fulfillment`  
Execution type: lifecycle-frozen seam replacement, parent-frozen contract, one short host/world parallel window, one serialized reconvergence lane, one docs lane, one parent-only validation wall  
Live repo root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`  
Fresh worktree root: `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment`  
Run id: `plan-gateway-mediated-llm-fulfillment`  
Worker model: `GPT-5.4` with `reasoning_effort=high`  
Initial concurrent worker cap: `0` during parent-only freeze  
Peak concurrent worker cap: `2` during the host/world cutover window  
Parent role: sole integrator, sole gate owner, sole writer of `.runs/**` artifacts, sole launcher of worker lanes, sole merger of worker outputs, sole authority for GitNexus escalations, validation start, and final acceptance

## Summary

This controller executes [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md). It is not a restatement of that plan. It is the parent-run execution script for how the plan is launched, split, merged, validated, and closed.

Frozen run shape:

1. Parent freezes the lifecycle contract, seam boundary, grep wall, branch map, worker ownership map, and validation wall.
2. Parent opens one short honest parallel window only:
   - `H1` host prompt-bearing gateway cutover
   - `W1` world-member gateway cutover
3. Parent reconverges the merged runtime story in one serialized lane:
   - `R1` bootstrap removal, `async_repl.rs` cleanup, shared routing assertion settlement
4. Parent launches the docs truth-sync lane only after `R1` is accepted:
   - `D1` runtime-truth and usage convergence
5. Parent runs the full validation wall, final GitNexus scope check, and closeout on the merged tree only.

Ship this run only if the merged tree proves all of the following together:

1. host first prompt and host follow-up prompt fulfill through the gateway adapter seam,
2. world first targeted prompt and resumed follow-up prompt fulfill through the gateway adapter seam,
3. stable backend-id selection still happens before prompt-bearing execution begins,
4. typed `MemberTurnSubmitRequestV1` plus `/v1/member_turn/stream` remain unchanged,
5. the visible lifecycle contract from `start`, `turn`, `reattach`, `stop`, posture semantics, and `Accepted -> terminal` remains unchanged,
6. integrated auth still uses the FD auth-bundle handoff,
7. no production runtime path in the targeted shell or world surfaces directly instantiates `AgentWrapperGateway`, `CodexBackend`, or `ClaudeCodeBackend`,
8. no production bootstrap prompt survives,
9. docs and truth surfaces describe the bypass as removed steady-state behavior, not acceptable architecture,
10. the final validation wall proves seam movement and lifecycle stability on the same merged tree.

## Hard Guards

These are run-stopping invariants.

1. This slice is a seam replacement only. It is not a public lifecycle redesign, not a selector redesign, not a shared envelope redesign, and not a backend-matrix expansion project.
2. `substrate agent start`, `turn`, `reattach`, and `stop` keep their existing public meaning exactly.
3. Stable backend ids remain the only public routing selector for prompt-bearing follow-up work.
4. `MemberTurnSubmitRequestV1` and `POST /v1/member_turn/stream` remain the typed world follow-up seam.
5. Integrated auth remains on `SUBSTRATE_LLM_AUTH_BUNDLE_FD` and the existing auth-bundle contract.
6. No new auth fallback path through child secret env vars may be introduced.
7. No hidden bootstrap prompt, fake agent prompt, warm-up prompt, or replay-visible synthetic first prompt may survive on a production path.
8. No production code under `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/repl/`, or `crates/world-service/src/member_runtime.rs` may remain a direct backend-registration table above the gateway seam.
9. `reattach` remains recovery-only and must not become a prompt-bearing execution path.
10. Detached world follow-up remains fail-closed.
11. SOW 29 and SOW 30 remain out of scope:
    - no shared dispatch-envelope expansion,
    - no public world-scoped root start,
    - no new capability-override surface.
12. No worker may edit [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md), this controller, or `.runs/**`.
13. Parent is the only actor allowed to reopen a gate, reinterpret the contract freeze, accept a `HIGH` or `CRITICAL` GitNexus blast radius, or decide final scope fit.
14. Every symbol edit requires prior GitNexus impact analysis.
15. Every worker handoff must include `mcp__gitnexus__.detect_changes()` output before parent acceptance.
16. Parent runs a final `mcp__gitnexus__.detect_changes()` on the merged tree before closeout.

Stop the run and write `blocked.json` if any of these become true:

1. host and world cutover cannot be split into truly disjoint lanes without concurrent ownership of `async_repl.rs` or another shared hotspot,
2. the seam move requires a new public CLI flag, new public verb, new world follow-up route, or new schema version,
3. the only path to green requires retaining a production synthetic bootstrap prompt,
4. the only path to green requires retaining a production shell-local or member-local backend-registration table,
5. stable backend selection can no longer happen before execution begins,
6. integrated auth can only be made to work by reopening secret-bearing env paths,
7. docs can only be made truthful by contradicting the merged code,
8. the validation wall cannot prove grep, focused runtime tests, auth continuity, and workspace gates on the same merged tree.

## Fresh Worktrees / Branches / Worker Model / Concurrency Cap

Fresh worktree root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment`

Authoritative integration checkout:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- branch: `feat/gateway-mediated-llm-fulfillment`

Worker worktrees:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/host-cutover`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/world-cutover`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/reconvergence`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/docs-truth-sync`

Worker branches:

- `codex/feat-gateway-mediated-llm-fulfillment-host-cutover`
- `codex/feat-gateway-mediated-llm-fulfillment-world-cutover`
- `codex/feat-gateway-mediated-llm-fulfillment-reconvergence`
- `codex/feat-gateway-mediated-llm-fulfillment-docs-truth-sync`

Exact setup order:

```bash
mkdir -p /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate fetch origin
```

Create the host and world worktrees only after `G0` is accepted:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/host-cutover \
  -b codex/feat-gateway-mediated-llm-fulfillment-host-cutover \
  feat/gateway-mediated-llm-fulfillment

git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/world-cutover \
  -b codex/feat-gateway-mediated-llm-fulfillment-world-cutover \
  feat/gateway-mediated-llm-fulfillment
```

Create the reconvergence worktree only after `G1` is accepted:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/reconvergence \
  -b codex/feat-gateway-mediated-llm-fulfillment-reconvergence \
  feat/gateway-mediated-llm-fulfillment
```

Create the docs worktree only after `G2` is accepted:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate worktree add \
  /Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/substrate-gateway-mediated-llm-fulfillment/docs-truth-sync \
  -b codex/feat-gateway-mediated-llm-fulfillment-docs-truth-sync \
  feat/gateway-mediated-llm-fulfillment
```

Concurrency contract:

1. `P0` and `G0` are parent-only and fully serialized.
2. `H1` and `W1` are the only honest parallel window.
3. `R1` is single-owner and serialized because it owns the shared conflict zone:
   - `crates/shell/src/repl/async_repl.rs`
   - bootstrap semantics
   - shared routing assertions
4. `D1` starts only after `R1` is accepted.
5. `P1`, `P2`, and `P3` are parent-only.
6. Peak concurrency is `2`, not `3` or `4`, because the plan has one narrow host/world split and then forced reconvergence.

## Parent-Owned Run-State Surface And Required Artifacts

Canonical run root:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-gateway-mediated-llm-fulfillment/`

Required parent-owned top-level artifacts:

- `run-state.json`
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

Required sentinels:

- `sentinels/RUN_OPEN`
- `sentinels/RUN_BLOCKED` on blocked runs only
- `sentinels/RUN_COMPLETE` on successful closeout only

Frozen gate directories:

- `gates/G0-parent-contract-freeze/`
- `gates/G1-host-world-accept-and-reconvergence-launch/`
- `gates/G2-reconvergence-accept-and-doc-launch/`
- `gates/G3-doc-accept-and-validation-launch/`
- `gates/G4-final-acceptance/`

Every gate directory must contain all of the following before the gate may transition:

- `gate.json`
- `evidence.md`
- one sentinel exactly one of:
  - `OPEN`
  - `PASSED`
  - `FAILED`
  - `REOPENED`

Task map:

- `tasks/P0-parent-contract-freeze-and-run-init/`
- `tasks/H1-host-fulfillment-cutover/`
- `tasks/W1-world-fulfillment-cutover/`
- `tasks/G1-host-world-accept-and-reconvergence-launch/`
- `tasks/R1-reconvergence-bootstrap-removal/`
- `tasks/G2-reconvergence-accept-and-doc-launch/`
- `tasks/D1-doc-truth-sync/`
- `tasks/G3-doc-accept-and-validation-launch/`
- `tasks/P1-parent-lane-integration/`
- `tasks/P2-parent-validation-wall/`
- `tasks/G4-final-acceptance/`
- `tasks/P3-parent-closeout/`

Every task directory must contain all of the following before parent may mark the task accepted:

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
- `blocker-notes.md` if blocked
- one sentinel exactly one of:
  - `READY_FOR_REVIEW`
  - `ACCEPTED`
  - `REJECTED`
  - `BLOCKED`

Artifact enforcement rule:

1. Workers do not write `.runs/**`.
2. Workers return handoff material to the parent and nothing is considered launched, reviewed, accepted, replayed, or blocked until the parent records it in `.runs/**`.
3. Parent must transcribe every worker handoff into `.runs/**`, including changed files, commands, exit codes, GitNexus notes, detect-changes output, acceptance notes, rejections, and blockers.
4. Parent creates or replaces task sentinels after transcription. Workers never create task sentinels.
5. Parent must update the matching `task.json`, `status.txt`, `changed-files.txt`, `commands.txt`, `exit-codes.json`, `gitnexus-detect-changes.txt`, and `handoff-notes.md` in the same review step.
6. Parent must update the matching gate `gate.json` and `evidence.md` at the moment a gate changes state.
7. No task is complete until parent writes the artifacts and marks the task `ACCEPTED`.
8. No gate is complete until parent writes the artifacts and marks the gate `PASSED`.

`contract-freeze.json` must record at minimum:

1. `authoritative_branch: "feat/gateway-mediated-llm-fulfillment"`
2. locked public lifecycle:
   - `start`
   - `turn`
   - `reattach`
   - `stop`
3. locked routing and transport decisions:
   - stable backend ids remain `<kind>:<name>`
   - `MemberTurnSubmitRequestV1`
   - `POST /v1/member_turn/stream`
4. locked auth decision:
   - `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
5. forbidden production symbols:
   - `AgentWrapperGateway`
   - `CodexBackend`
   - `ClaudeCodeBackend`
   - `runtime_bootstrap_prompt`
6. preserved fail-closed rules:
   - detached world follow-up stays fail-closed
   - invalid backend selection stays invalid
   - blocked env auth does not reopen fallback execution
7. the exact validation wall commands
8. the concurrency contract:
   - initial worker cap `0`
   - peak worker cap `2`

## GitNexus Workflow And Required Impact Targets

GitNexus is a run-control requirement, not a best-effort check.

### Source-Lock Stage

1. Parent verifies GitNexus repo selection and index freshness before any worker edits.
2. If the index is stale, parent runs `npx gitnexus analyze` from the authoritative checkout before launching `H1` or `W1`.
3. Parent records the freshness result in `tasks/P0-parent-contract-freeze-and-run-init/impact-analysis-summary.md`.

### Minimum Required Targets By Task

`H1` host fulfillment cutover

1. Symbol targets that must be analyzed before edits:
   - `submit_host_prompt_turn`
   - `build_gateway_for_descriptor`
   - `ResolvedHostOrchestratorBootstrap`
   - `PreparedAgentRuntime`
2. File-level seam targets that must be analyzed if symbol names drift:
   - `crates/shell/src/execution/agent_runtime/control.rs`
   - `crates/shell/src/execution/agent_runtime/registry.rs`
   - `crates/shell/src/execution/agent_runtime/mapping.rs`
   - `crates/shell/src/execution/agent_runtime/validator.rs`
   - `crates/shell/src/repl/async_repl.rs`
   - `crates/shell/tests/agent_public_control_surface_v1.rs`
   - `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

`W1` world fulfillment cutover

1. Symbol targets that must be analyzed before edits:
   - `MemberRuntimeManager::launch`
   - `MemberRuntimeManager::submit_turn`
   - `build_gateway_for_backend`
   - `runtime_bootstrap_prompt`
2. File-level seam targets that must be analyzed if symbol names drift:
   - `crates/world-service/src/member_runtime.rs`
   - `crates/world-service/src/service.rs`
   - `crates/world-service/src/lib.rs`
   - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
   - `crates/shell/tests/repl_world_first_routing_v1.rs`
   - `crates/world-service/tests/streamed_execute_cancel_v1.rs`
   - `crates/world-service/tests/member_runtime_world_placement_v1.rs`

`R1` reconvergence bootstrap removal

1. Symbol targets that must be analyzed before edits:
   - `runtime_bootstrap_prompt`
   - the shell-side startup helper in `crates/shell/src/repl/async_repl.rs` that prepares world-member execution state
   - any new shared fulfillment helper introduced by `H1` or `W1`
2. File-level seam targets that must be analyzed if symbol names drift:
   - `crates/shell/src/repl/async_repl.rs`
   - `crates/world-service/src/member_runtime.rs`
   - `crates/shell/tests/repl_world_first_routing_v1.rs`
   - `crates/shell/tests/support/repl_world_service.rs`

`D1` docs truth-sync

1. File-level seam targets:
   - `llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md`
   - `AGENT_ORCHESTRATION_GAP_MATRIX.md`
   - `docs/contracts/substrate-gateway-runtime-parity.md`
   - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
   - `docs/USAGE.md`
   - `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`
   - relevant ADR wording under `docs/project_management/adrs/draft/` only if implementation evidence requires sync
2. This lane is docs-truth only. If no symbol-level targets are relevant, the worker must state that explicitly in the handoff.

### Escalation Rule

1. Any `HIGH` or `CRITICAL` impact result stops that worker before edits.
2. The worker returns a blocker handoff instead of proceeding.
3. Parent records the blocker, decides whether the run still fits the frozen contract, and either relaunches with a narrower brief or blocks the run.

## Workstream Plan With Parent-Owned Gates And Worker-Owned Lanes

### Workstream Map

| PLAN.md workstream | Orchestration tasks | Ownership |
| --- | --- | --- |
| Freeze lifecycle contract, seam boundary, grep wall, change budget | `P0`, `G0` | Parent only |
| Host prompt-bearing gateway cutover | `H1` | Host lane |
| World-member gateway cutover | `W1` | World lane |
| Shared reconvergence, bootstrap removal, REPL cleanup | `G1`, `R1`, `G2` | Parent gate plus serialized reconvergence lane |
| Truth docs and usage convergence | `D1`, `G3` | Docs lane plus parent gate |
| Integration, validation, and closeout | `P1`, `P2`, `G4`, `P3` | Parent only |

### Parent-Owned Gates

`G0`: Contract freeze

Intent: lock the run before any parallel work exists.

1. Parent locks the lifecycle contract, backend-selection contract, auth carrier, grep wall, validation wall, branch map, and lane ownership map.
2. Parent confirms later workers do not need to guess whether a behavior change is allowed.
3. Parent records the exact shared-hotspot no-split list before any code edits begin.

`G1`: Host/world acceptance and reconvergence launch

Intent: prove the only parallel window is individually sound, then stop parallelism before the shared conflict zone.

1. `H1` and `W1` are both accepted or explicitly replayed onto the authoritative branch.
2. Parent confirms the merged branch proves both cutovers independently before reconvergence work starts.
3. Parent confirms that major edits to `async_repl.rs` and shared routing assertions remain unclaimed until `R1`.
4. Only after `G1` is accepted may the reconvergence worktree be created.

`G2`: Reconvergence acceptance and docs launch

Intent: freeze one final runtime story before any truth-doc sync begins.

1. `R1` is accepted on the authoritative branch.
2. Parent confirms:
   - no production bootstrap prompt remains,
   - `async_repl.rs` no longer prepares a discarded shell-local authoritative member gateway,
   - shared routing assertions reflect the final runtime story.
3. Only after `G2` is accepted may `D1` start.

`G3`: Docs acceptance and validation launch

Intent: freeze the reviewer candidate SHA and stop all content movement before validation begins.

1. `D1` is accepted on the authoritative branch.
2. Parent confirms docs now describe direct wrapper/backend registration as historical bypass behavior rather than intended architecture.
3. Parent freezes the validation candidate SHA and starts `P2`.

`G4`: Final acceptance

Intent: convert a validated candidate into a closed run with explicit evidence, or block it explicitly.

1. Grep wall is green.
2. Focused runtime tests are green.
3. Auth continuity checks are green.
4. Workspace gates are green or explicitly environment-blocked with evidence.
5. Final GitNexus scope verification matches the frozen plan.

### Worker-Owned Lanes

`H` host lane

1. host first prompt cutover
2. host follow-up prompt cutover
3. host-local registry and bootstrap-state cleanup only within lane scope

`W` world lane

1. launch-time world first-turn cutover
2. resumed world follow-up cutover
3. retained-member validation preservation

`R` reconvergence lane

1. bootstrap prompt removal
2. `async_repl.rs` cleanup
3. final shared routing and prompt-capture assertions

`D` docs lane

1. SOW and gap-matrix truth sync
2. gateway-runtime and adapter-protocol doc sync
3. usage and truth-surface cleanup

## Task Execution Contracts

### `P0-parent-contract-freeze-and-run-init`

Primary owned surfaces:

- [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md)
- [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)
- `.runs/plan-gateway-mediated-llm-fulfillment/**`

Required actions:

1. Freeze the lifecycle contract, seam boundary, grep wall, validation wall, branch map, and lane ownership map from the current plan.
2. Create `branch-map.json`, `lane-ownership.json`, `merge-order.json`, and `contract-freeze.json`.
3. Record GitNexus freshness status and the concurrency contract.
4. Create the gate and task directories and initial sentinels.

Verification commands:

```bash
test -f /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md
test -f /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate branch --show-current
```

Acceptance conditions:

1. No worker would need to guess scope, authority, or validation commands.
2. `.runs/**` skeleton exists on paper with required artifacts and sentinel rules.
3. Parent has recorded whether GitNexus freshness work is required before worker launch.

### `H1-host-fulfillment-cutover`

Primary owned surfaces or file families:

- `crates/shell/src/execution/agent_runtime/**`
- the narrowest necessary host-owned bootstrap metadata paths in `crates/shell/src/repl/async_repl.rs`
- `crates/shell/tests/agent_public_control_surface_v1.rs`
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Required actions:

1. Replace the direct host prompt-bearing path so `submit_host_prompt_turn()` no longer builds a local authoritative gateway through `build_gateway_for_descriptor()`.
2. Route host prompt-bearing execution through the gateway-mediated seam using the already-selected stable backend id and existing resume metadata.
3. Remove shell-local `gateway` and `agent_kind` fields from host execution-state structures where they currently serve as runtime execution truth rather than routing metadata.
4. Preserve host public meaning exactly:
   - `start` uses the real user prompt as the first prompt,
   - `turn` uses the real follow-up prompt,
   - resume metadata still threads through execution,
   - posture, completion, and trace publication semantics remain unchanged.
5. Preserve failure buckets:
   - invalid selection stays invalid selection,
   - dependency unavailable stays dependency unavailable,
   - policy denial stays policy denial.

Verification commands:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
rg -n "AgentWrapperGateway|CodexBackend|ClaudeCodeBackend" \
  crates/shell/src/execution/agent_runtime \
  crates/shell/src/repl
```

Acceptance conditions:

1. No host prompt-bearing production path directly registers concrete backends above the gateway seam.
2. Host `start` and host `turn` still behave the same from the CLI surface.
3. Resume metadata still threads correctly through the new seam.
4. Any `async_repl.rs` edits are narrow plumbing only and do not pre-empt `R1`.

### `W1-world-fulfillment-cutover`

Primary owned surfaces or file families:

- `crates/world-service/src/member_runtime.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/src/lib.rs`
- the narrowest required transport plumbing in `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`
- `crates/world-service/tests/streamed_execute_cancel_v1.rs`
- `crates/world-service/tests/member_runtime_world_placement_v1.rs`

Required actions:

1. Replace both direct world execution call sites in member runtime:
   - `launch()`
   - `submit_turn()`
2. Ensure both launch-time first turn and resumed follow-up traverse the same gateway-mediated fulfillment seam.
3. Preserve the typed transport boundary exactly:
   - launch-time world prompt still enters through `member_dispatch.initial_prompt`,
   - resumed follow-up still enters through `MemberTurnSubmitRequestV1`,
   - `/v1/member_turn/stream` remains unchanged.
4. Preserve retained-member identity validation, world binding checks, participant/backend/world tuple validation, and detached-world fail-closed behavior.
5. Keep member stream event translation and completion framing stable from the shell perspective.

Verification commands:

```bash
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-service --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-service --test member_runtime_world_placement_v1 -- --nocapture
rg -n "AgentWrapperGateway|CodexBackend|ClaudeCodeBackend|runtime_bootstrap_prompt" \
  crates/world-service/src/member_runtime.rs
```

Acceptance conditions:

1. World-member production execution no longer locally constructs wrappers or backends in the targeted surfaces.
2. Launch-time first turn and resumed follow-up visibly use one fulfillment seam.
3. Typed transport contracts remain unchanged.
4. Retained-member invariants and detached-world fail-closed behavior still hold.

### `R1-reconvergence-bootstrap-removal`

Primary owned surfaces or file families:

- `crates/shell/src/repl/async_repl.rs`
- `crates/world-service/src/member_runtime.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`
- `crates/shell/tests/support/repl_world_service.rs`

Required actions:

1. Delete or demote `runtime_bootstrap_prompt()` so it is no longer part of production prompt semantics.
2. Ensure the first targeted world turn carries the real user prompt all the way to fulfillment.
3. Remove shell-local authoritative gateway preparation in `async_repl.rs` when that state exists only to be discarded before real world-member execution.
4. Update shared prompt-capture and routing assertions so the post-cutover runtime story settles in one lane.
5. Harmonize any helper shapes introduced independently by `H1` and `W1` so the merged runtime story is singular and boring.

Verification commands:

```bash
rg -n "runtime_bootstrap_prompt|Enter persistent Substrate world-scoped member mode" \
  crates/world-service/src/member_runtime.rs \
  crates/shell/src/repl \
  crates/shell/tests \
  crates/world-service/tests

cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Acceptance conditions:

1. The first prompt-bearing execution is always the real user prompt.
2. `async_repl.rs` no longer owns a discarded execution-time gateway for world members.
3. Shared routing assertions prove the post-merge story end to end.
4. `H1` and `W1` helper drift has been collapsed rather than layered.

### `D1-doc-truth-sync`

Primary owned surfaces or file families:

- `llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md`
- `AGENT_ORCHESTRATION_GAP_MATRIX.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
- `docs/USAGE.md`
- `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`
- relevant descriptive ADR wording only if code evidence now exists and the ADR meaning stays unchanged

Required actions:

1. Update truth docs so they describe direct wrapper/backend registration as historical bypass behavior rather than steady-state architecture.
2. Update the gap matrix so this seam is no longer described as outstanding once the code is landed.
3. Keep ADR-0040, ADR-0041, and ADR-0047 stable in ownership and lifecycle meaning.
4. Ensure usage text and truth docs reflect:
   - start uses the real first prompt,
   - turn uses the real follow-up prompt,
   - `reattach` is recovery-only,
   - gateway-mediated fulfillment is the production story for host and world prompt-bearing execution.

Verification commands:

```bash
rg -n "AgentWrapperGateway|CodexBackend|ClaudeCodeBackend|runtime_bootstrap_prompt" \
  llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md \
  AGENT_ORCHESTRATION_GAP_MATRIX.md \
  docs/contracts/substrate-gateway-runtime-parity.md \
  docs/contracts/substrate-gateway-backend-adapter-protocol.md \
  docs/USAGE.md \
  HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md
```

Acceptance conditions:

1. Live docs tell the same runtime story as the merged code.
2. No truth doc implies the bypass is still intended steady-state behavior.
3. ADR wording stays descriptive and does not reopen settled decisions.

### `P1-parent-lane-integration`

Primary owned surfaces:

- authoritative branch `feat/gateway-mediated-llm-fulfillment`
- `.runs/**`

Required actions:

1. Review worker handoffs against ownership boundaries and GitNexus scope evidence.
2. Integrate lanes in the frozen order only:
   - accept or reject `H1`
   - accept or reject `W1`
   - after both are accepted, launch and later integrate `R1`
   - after `R1` is accepted, launch and later integrate `D1`
3. For `H1` and `W1`, parent must review handoff evidence first, then diff file sets, then merge into `feat/gateway-mediated-llm-fulfillment`, then record the post-merge SHA.
4. Replay or quarantine any lane that reopens a forbidden hotspot without approval.
5. After every accepted lane merge, parent must update:
   - `changed-files.txt`
   - `commands.txt`
   - `exit-codes.json`
   - `gitnexus-detect-changes.txt`
   - `handoff-notes.md`
   - `summary.md`
   - `HEAD_SHA.txt`
6. Parent must update the active gate evidence immediately after each acceptance or rejection so the run log is reconstructable without external context.

Verification commands:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate status --short
```

Acceptance conditions:

1. The authoritative branch contains exactly the accepted lane outputs.
2. Any overlap is resolved by explicit replay or rejection, not silent manual blending.
3. `.runs/**` reflects the real merge, replay, rejection, and quarantine history in order.

### `P2-parent-validation-wall`

Primary owned surfaces:

- merged authoritative tree
- `.runs/**`

Required actions:

1. Validate only the candidate SHA frozen at `G3`.
2. Run the static seam-removal grep wall first.
3. Run the focused runtime and auth continuity tests second.
4. Run the workspace validation gates last.
5. Record every command and exit code in execution order.
6. Record environment blockers explicitly if any platform prerequisites are unavailable.
7. Produce one reviewer-facing validation artifact naming:
   - lifecycle invariants checked,
   - host and world prompt-bearing scenarios checked,
   - auth-bundle checks,
   - static seam-removal checks.
8. If any validation step fails, parent must stop the wall, record the failure point, and either reopen the relevant task or block the run. Parent must not continue to later validation stages as if the candidate were still clean.

Verification commands:

1. The exact grep, focused cargo, and workspace commands frozen in `Validation Wall`.

Acceptance conditions:

1. Validation is green or explicitly blocked by environment availability with no ambiguity about follow-up.
2. The merged tree proves seam convergence and lifecycle stability together.
3. Parent records exact commands, exit codes, evidence references, and follow-up disposition in `.runs/**`.

### `P3-parent-closeout`

Primary owned surfaces:

- `.runs/**`
- final authoritative branch state

Required actions:

1. Run final `mcp__gitnexus__.detect_changes()` on the merged tree.
2. Confirm the changed scope matches the frozen plan and that no unexpected execution flows were pulled in during merge or replay.
3. Write final parent-owned closeout artifacts in this order:
   - update `run-state.json`
   - update final task statuses
   - update final gate statuses
   - write `final-summary.md`
   - write terminal sentinel
4. If the run cannot close honestly, write `blocked.json` instead of `RUN_COMPLETE`.
5. Parent must leave the run in exactly one terminal state and must not leave both closeout artifacts present.

Verification commands:

```bash
git -C /Users/spensermcconnell/__Active_Code/atomize-hq/substrate status --short
```

Acceptance conditions:

1. Final run-state artifacts match the actual merged result.
2. Parent has recorded final scope verification and residual risks.
3. The run ends with exactly one terminal sentinel:
   - `RUN_COMPLETE`
   - `RUN_BLOCKED`

## Exact Lane Ownership Boundaries By Directories / Modules

### Lane H: Host Ownership

Owns these surfaces end to end:

- `crates/shell/src/execution/agent_runtime/**`
- host-owned metadata plumbing in `crates/shell/src/repl/async_repl.rs` only when unavoidable
- `crates/shell/tests/agent_public_control_surface_v1.rs`
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Lane H specifically owns:

1. host first prompt gateway mediation,
2. host follow-up prompt gateway mediation,
3. removal of host-local authoritative backend-registration behavior,
4. preservation of host lifecycle meanings and failure buckets.

Lane H may not edit:

- `crates/world-service/src/member_runtime.rs`
- world-member tuple validation logic
- `POST /v1/member_turn/stream` contract surfaces
- docs and truth surfaces

### Lane W: World Ownership

Owns these surfaces end to end:

- `crates/world-service/src/member_runtime.rs`
- minimum required `crates/world-service/src/service.rs`
- `crates/world-service/src/lib.rs`
- minimal launch-time request plumbing in `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs` only for world-routing assertions that do not pre-empt `R1`
- `crates/world-service/tests/streamed_execute_cancel_v1.rs`
- `crates/world-service/tests/member_runtime_world_placement_v1.rs`

Lane W specifically owns:

1. launch-time world first-turn cutover,
2. resumed follow-up cutover,
3. world-side gateway-mediated fulfillment seam convergence,
4. retained-member validation preservation.

Lane W may not edit:

- host lifecycle semantics
- host-only control tests
- docs and truth surfaces
- major `async_repl.rs` cleanup that belongs to `R1`

### Lane R: Reconvergence Ownership

Owns these surfaces after `G1`:

- `crates/shell/src/repl/async_repl.rs`
- final cleanup in `crates/world-service/src/member_runtime.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`
- `crates/shell/tests/support/repl_world_service.rs`

Lane R specifically owns:

1. bootstrap prompt removal,
2. removal of discarded shell-local authoritative gateway prep,
3. final prompt-capture assertions for world first turn,
4. harmonization of helper shapes created independently during `H1` and `W1`.

Lane R may not edit:

- docs
- non-targeted public lifecycle semantics
- typed world follow-up contract definitions

### Lane D: Docs Ownership

Owns these surfaces after `G2`:

- `llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md`
- `AGENT_ORCHESTRATION_GAP_MATRIX.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
- `docs/USAGE.md`
- `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`
- relevant descriptive ADR wording only if required

Lane D specifically owns:

1. runtime-truth convergence,
2. usage and operator guidance sync,
3. gap-matrix completion wording,
4. descriptive ADR sync that does not reopen decisions.

Lane D may not edit:

- code
- tests
- scripts
- manifests

### Hotspot No-Split List

These surfaces must not be split across concurrent workers:

1. `crates/shell/src/repl/async_repl.rs`
2. shared prompt-capture assertions in `crates/shell/tests/repl_world_first_routing_v1.rs`
3. any helper introduced to normalize host and world prompt-bearing inputs into the gateway seam
4. any auth-bundle handoff logic under `crates/world-service/src/gateway_runtime.rs` or gateway server startup that would widen scope
5. any file that redefines backend selection before execution begins

## Merge / Integration Order

Frozen integration order:

1. `P0-parent-contract-freeze-and-run-init`
2. `G0-parent-contract-freeze`
3. parallel launch:
   - `H1-host-fulfillment-cutover`
   - `W1-world-fulfillment-cutover`
4. `G1-host-world-accept-and-reconvergence-launch`
5. `R1-reconvergence-bootstrap-removal`
6. `G2-reconvergence-accept-and-doc-launch`
7. `D1-doc-truth-sync`
8. `G3-doc-accept-and-validation-launch`
9. `P1-parent-lane-integration`
10. `P2-parent-validation-wall`
11. `G4-final-acceptance`
12. `P3-parent-closeout`

Integration rules:

1. `H1` and `W1` must each merge into the authoritative branch before `R1` begins.
2. Parent must diff the `H1` and `W1` file sets before acceptance. This is mandatory.
3. Parent must merge `H1` first, then `W1`, unless the parent records a concrete reason to reverse them in `merge-order.json`. The default order is fixed because host lane scope is narrower and should settle first.
4. If either `H1` or `W1` expands into the shared conflict zone without explicit approval, parent rejects that handoff and requires replay.
5. `R1` must run on top of the accepted merged `H1` plus `W1` tree, not on one lane independently.
6. `D1` must run on top of the accepted `R1` tree.
7. Parent may not manually blend rejected worker fragments into the authoritative tree. Rejected lanes must replay as coherent passes.

## Validation Wall

Parent owns and runs the full validation wall on the merged tree only.

### Grep Gates

Zero-hit wall on the targeted production runtime surfaces:

```bash
rg -n "AgentWrapperGateway|CodexBackend|ClaudeCodeBackend" \
  crates/shell/src/execution/agent_runtime \
  crates/shell/src/repl \
  crates/world-service/src/member_runtime.rs
```

Synthetic bootstrap prompt gate:

```bash
rg -n "runtime_bootstrap_prompt|Enter persistent Substrate world-scoped member mode" \
  crates/world-service/src/member_runtime.rs \
  crates/shell/src/repl \
  crates/shell/tests \
  crates/world-service/tests
```

Expected result after the slice:

1. no production hits,
2. any remaining hits must be in explicitly historical or non-production tests only.

Positive guardrails that must still succeed:

```bash
rg -n "MemberTurnSubmitRequestV1|/v1/member_turn/stream|SUBSTRATE_LLM_AUTH_BUNDLE_FD" \
  crates/shell \
  crates/world-service \
  crates/common \
  crates/gateway \
  docs
```

### Focused Cargo Gates

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test world_gateway -- --nocapture
cargo test -p world-service --test streamed_execute_cancel_v1 -- --nocapture
cargo test -p world-service --test member_runtime_world_placement_v1 -- --nocapture
cargo test -p substrate-gateway --test openai_shared_parity -- --nocapture
```

### Full Workspace Gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

### Manual Validation Proof Points

Parent closeout must explicitly record proof of:

1. host `start` uses the real user prompt as the first prompt-bearing execution,
2. host `turn` uses the real user prompt as the follow-up prompt,
3. world first targeted turn uses the real user prompt and not a bootstrap prompt,
4. resumed world follow-up still travels through `/v1/member_turn/stream`,
5. detached world follow-up still fails closed,
6. gateway startup still consumes the FD auth bundle,
7. runtime artifacts or traces provide evidence that fulfillment is gateway-mediated rather than shell-local or member-local wrapper construction.

## Blocked-Run Contract

If the run blocks, parent writes:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.runs/plan-gateway-mediated-llm-fulfillment/blocked.json`

Required fields:

- `run_id`
- `authoritative_branch`
- `timestamp`
- `current_task_id`
- `gate_state`
- `summary`
- `stop_condition_id`
- `worker_lane`
- `blocking_files`
- `accepted_outputs`
- `quarantined_outputs`
- `next_required_parent_action`

Blocked-run rules:

1. `blocked.json` is parent-written only.
2. It is written exactly once at the stop point.
3. No further worker launches occur after it is written.
4. Existing worker outputs are either accepted and recorded or quarantined and named explicitly.

## Context-Control Rules For Parent And Workers

### Parent Rules

1. Parent keeps the canonical run state in `.runs/**` only.
2. Parent is the only actor allowed to reopen a gate or reinterpret the contract freeze.
3. Parent must keep the locked decisions, grep wall, and no-split list visible in every worker brief.
4. Parent owns all cross-lane rebases, merges, acceptance decisions, validation start, and closeout.
5. Parent records GitNexus findings, handoffs, merge outcomes, validation commands, gate transitions, and final acceptance artifacts.
6. Parent must be able to reconstruct the run from `.runs/**` alone without relying on chat history.

### Worker Rules

1. Read [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md), this controller, and only the lane-relevant files before editing.
2. Do not widen scope beyond the lane boundary.
3. Run GitNexus impact analysis before editing symbols in the owned lane.
4. Stop and escalate on `HIGH` or `CRITICAL` impact.
5. Do not write `.runs/**`.
6. Do not edit [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) or [ORCH_PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md).
7. Handoffs must include:
   - changed files
   - commands run
   - exit codes
   - GitNexus impact summary
   - `mcp__gitnexus__.detect_changes()` output
   - blockers or residual risks

### Context Hygiene

1. No worker loads broad unrelated surfaces just because the seam is cross-cutting.
2. Lane H reads only host runtime files, host tests, and the smallest unavoidable bootstrap metadata surfaces.
3. Lane W reads only world runtime files, world tests, and the typed launch/follow-up transport surfaces it owns.
4. Lane R reads only the reconvergence conflict zone and shared routing assertions.
5. Lane D reads only the named truth and usage documents.

## Acceptance / Completion Criteria

The run is complete only when all of the following are true on the merged tree:

1. `P0`, `H1`, `W1`, `R1`, `D1`, `P1`, `P2`, and `P3` are accepted by the parent.
2. The frozen contract in `.runs/**` still matches [PLAN.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/PLAN.md) exactly.
3. The host/world parallel window happened only between `H1` and `W1`.
4. `R1` happened only after both `H1` and `W1` were accepted.
5. The production runtime grep wall is green in the targeted shell and world surfaces.
6. Focused runtime and auth tests are green.
7. Workspace validation gates are green or explicitly environment-blocked with evidence.
8. Docs and truth surfaces match the merged code.
9. No production bootstrap prompt remains.
10. No production shell-local or member-local direct backend-registration table remains above the gateway seam in the targeted runtime files.
11. Parent runs final `mcp__gitnexus__.detect_changes()` and confirms the changed scope matches the plan.
12. Parent writes `final-summary.md` and marks `sentinels/RUN_COMPLETE`.

## Task Acceptance Checklist

| Task | Done means |
| --- | --- |
| `P0` | Lifecycle contract, seam boundary, grep wall, validation wall, branch map, lane ownership, and `.runs` artifact contract are frozen in parent-owned artifacts. |
| `H1` | Host first prompt and follow-up prompt fulfill through the gateway seam, host lifecycle meaning is unchanged, and host-local direct backend registration is removed from owned production paths. |
| `W1` | World launch-time first turn and resumed follow-up fulfill through the gateway seam, typed member transport is unchanged, and world-local direct backend registration is removed from owned production paths. |
| `R1` | Production bootstrap prompt behavior is gone, `async_repl.rs` no longer prepares a discarded authoritative member gateway, and shared routing assertions prove the final runtime story. |
| `D1` | Truth docs, usage text, and the gap matrix describe the same runtime story as the merged code and no longer present the bypass as intended architecture. |
| `P1` | Parent has integrated accepted lane outputs in the frozen order and recorded the real merge history in `.runs/**`. |
| `P2` | Grep, focused tests, auth checks, workspace gates, and manual proof points are executed and recorded against the merged validation candidate. |
| `P3` | Final GitNexus scope verification, final summary, terminal sentinel, and residual-risk notes are written by the parent. |

## Assumptions

1. The authoritative execution branch remains `feat/gateway-mediated-llm-fulfillment`.
2. Fresh worker worktrees can be created under `/Users/spensermcconnell/__Active_Code/atomize-hq/.worktrees/`.
3. GitNexus is available or can be refreshed before worker edits begin.
4. The host/world split is honest only until the shared reconvergence zone. After that, the plan must serialize.
5. The validation wall may depend on environment prerequisites already documented in the repo; if an environment is unavailable, parent records that explicitly rather than silently skipping it.
6. This controller supersedes the stale UAA cleanup orchestration topic previously present in this file.
