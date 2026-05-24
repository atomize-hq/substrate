# PLAN: Explicit Control-Only Session Recovery And Host-Rooted World-Start Alignment

Source SOW: [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](llm-last-mile/28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md)  
Primary truth anchors: [docs/USAGE.md](docs/USAGE.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [ADR-0047](docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Primary code anchors: [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs), [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs), [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs), [crates/world-service/src/member_runtime.rs](crates/world-service/src/member_runtime.rs)  
Adjacent slices: [28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md](llm-last-mile/28-gateway-mediated-llm-fulfillment-without-lifecycle-regression.md), [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), [30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md), [31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Execution branch: `feat/gateway-mediated-llm-fulfillment`  
Base branch: `main`  
Plan type: control-plane seam split and downstream architecture freeze  
Review posture: unified execution plan tightened to `/autoplan` and `/plan-eng-review` rigor  
Status: implementation-ready planning pass on 2026-05-24

## Objective

Make Substrate's public control contract truthful by removing the hidden owner-helper convergence that currently uses blank-prompt UAA resume semantics as the implementation substrate for `reattach` and `fork`.

This slice is complete only when all of the following are true:

1. `start` and `turn` remain the only public prompt-bearing verbs.
2. `reattach` is implemented as a Substrate-owned control-only attach action, not as public blank-prompt resume semantics.
3. `fork` is implemented as a Substrate-owned successor durable-session allocator, not as a disguised backend resume path.
4. each durable host orchestration session persists the minimum host attach contract required for later attach without guessing from the currently active participant,
5. the runtime clearly separates control-only attach, prompt-bearing turn launch, and successor allocation,
6. the downstream stack is frozen to one architecture:
   - host-rooted durable orchestration authority,
   - persisted host attach contract,
   - world workers under that authority,
   - lazy host attach later without synthetic prompts.

This is a control-plane correction and architecture freeze. It is not a gateway redesign, not a public lifecycle expansion, and not a UAA contract standardization project.

## Acceptance Criteria

This plan is only done when all of the following are true in code, tests, and docs:

1. the workspace no longer resolves `unified-agent-api` through the local `[patch.crates-io]` override in [Cargo.toml](Cargo.toml),
2. no live Substrate path treats public `reattach` as `agent_api.session.resume.v1` plus `prompt: ""`,
3. no live Substrate path treats public `fork` as `agent_api.session.resume.v1` plus `prompt: ""`,
4. the runtime has explicit internal separation between:
   - control-only attach,
   - prompt-bearing resumed turn launch,
   - successor durable-session allocation,
5. [OrchestrationSessionRecord](crates/shell/src/execution/agent_runtime/orchestration_session.rs) persists a host attach contract that survives detach and is copied forward with successor-safe normalization,
6. public `fork` returns honest successor truth:
   - new orchestration session id,
   - copied attach contract shape,
   - `continuity_uaa_session_id = None` on the successor,
   - `attached_participant_id = null`,
   - `posture = parked_resumable`,
   - no synthetic prompt submission,
7. host `start` and host `turn` remain prompt-bearing,
8. Linux world-member follow-up keeps the existing typed `MemberTurnSubmitRequestV1` plus `/v1/member_turn/stream` contract and still requires a non-empty prompt,
9. detached-world follow-up remains fail-closed until valid host ownership is restored,
10. [docs/USAGE.md](docs/USAGE.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [ADR-0047](docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md), and slices 29/30/31 all tell the same story.

## Locked Decisions

These decisions are already made. Implementation does not reopen them.

| Topic | Locked decision | Why |
| --- | --- | --- |
| Public prompt verbs | `start` and `turn` stay the only prompt-bearing public verbs | The operator contract is already frozen there |
| Public `reattach` | Recovery only, no prompt | Public recovery must be control-only |
| Public `fork` | Allocate successor durable session first, do not synthesize a backend turn | Successor allocation is Substrate lifecycle work |
| UAA forward role | UAA stays prompt-bearing for real prompt execution only | Blank prompt is not durable contract meaning |
| World follow-up seam | Keep `MemberTurnSubmitRequestV1` and `/v1/member_turn/stream` | That contract is already landed and proven |
| Durable authority | Orchestration session remains the authority, not the currently attached backend process | Existing truth docs already commit to that model |
| Reattach attach mode in this slice | `reattach` is continuity-only in this slice; missing continuity fails closed | Fresh attach is a later slice 31 concern, not part of this correction |
| Successor continuity token | Successors inherit the attach contract shape but not the parent's live `continuity_uaa_session_id` | One backend-native continuity token cannot be dual-owned by source and successor durable sessions |
| Successor posture | Successful public `fork` returns `parked_resumable` with `attached_participant_id = null` | A successor that has not attached a host client must not claim active attachment |
| World-scoped root future | `--scope world` remains host-rooted only in the downstream stack | Standalone world-root continuity is not the chosen direction |
| Scope boundary | 29 generalizes the attach contract, 30 adds public world-root start, 31 adds lazy attach | This slice must freeze their inputs, not absorb their work |

## Scope

### In scope

1. remove the local crates.io patch override for `unified-agent-api` and prove Substrate no longer depends on the neighboring promptless branch,
2. split the hidden owner-helper convergence into explicit Substrate-owned control and prompt paths,
3. introduce the minimal persisted host attach contract under durable session state,
4. implement control-only `reattach` as a Substrate attach action,
5. rework public `fork` into successor durable-session allocation with honest detached truth,
6. preserve prompt-bearing `start` and `turn` behavior without widening any public prompt semantics,
7. align docs and downstream slices 29/30/31 to the validated architecture,
8. add regression coverage that proves no blank-prompt control semantics remain in live Substrate runtime code.

### NOT in scope

This slice explicitly does not include:

1. reopening SOW 28 gateway-mediated prompt fulfillment decisions,
2. inventing a new public UAA control-only API,
3. public world-root `start` shipping in this slice,
4. lazy host attach shipping in this slice,
5. standalone world-root continuity,
6. public inbox workflow expansion or auto-resume semantics,
7. new backend selector grammar, new policy surface, or new capability flags,
8. moving durable authority into the world or making world participants public control authorities.

## Step 0: Scope Challenge

### What already exists

| Sub-problem | Existing code or contract | Reuse decision |
| --- | --- | --- |
| Public prompt-taking host lifecycle | [run_start(...)](crates/shell/src/execution/agents_cmd.rs), [run_turn(...)](crates/shell/src/execution/agents_cmd.rs), [run_public_prompt_command(...)](crates/shell/src/execution/agent_runtime/control.rs) | Reuse exactly. Do not widen prompt semantics. |
| Public control target resolution | [resolve_public_control_target(...)](crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse as the authoritative control selector seam. |
| Public turn target resolution | [resolve_public_turn_target(...)](crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse exactly. Detached host continuity and world-member routing rules stay intact. |
| Public world follow-up transport | [MemberTurnSubmitRequestV1](crates/transport-api-types/src/lib.rs), [crates/world-service/src/member_runtime.rs](crates/world-service/src/member_runtime.rs), [crates/world-service/src/service.rs](crates/world-service/src/service.rs) | Reuse exactly. This seam is frozen. |
| Durable orchestration posture truth | [OrchestrationSessionRecord](crates/shell/src/execution/agent_runtime/orchestration_session.rs) and posture helpers in [state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse and extend. Do not add a second durable truth model. |
| Current runtime descriptor snapshot | [ResolvedRuntimeDescriptor](crates/shell/src/execution/agent_runtime/control.rs) | Reuse as the baseline launch descriptor payload for the attach contract. |
| Current continuity selector | `internal_uaa_session_id` on retained participants and launch plans | Reuse as private continuity state, but stop treating it as the public contract itself. |
| Existing lifecycle contract docs | [docs/USAGE.md](docs/USAGE.md), [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md), [ADR-0047](docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md) | Reuse as the public truth floor and sync them together. |

### Minimum honest change

The minimum honest implementation is still cross-cutting:

1. remove the local UAA patch override first so this slice proves it does not depend on unmerged promptless behavior,
2. persist a real host attach contract on the durable session,
3. split control-only attach from prompt-bearing turn launch,
4. make `fork` allocate successor durable truth without performing a synthetic prompt-free backend turn,
5. normalize successor attach state explicitly:
   - copy reusable attach-contract fields,
   - clear successor `continuity_uaa_session_id`,
   - leave source continuity ownership unchanged,
6. prove the behavior with targeted lifecycle, state-store, public-control, and world-member regression coverage,
7. rewrite the downstream SOW stack so no later plan assumes blank-prompt control semantics are still available.

Anything smaller keeps the repo in its current contradictory state: the docs already say Substrate owns durable lifecycle truth, but the implementation still smuggles control work through prompt-shaped UAA semantics.

### Complexity check

This slice touches more than eight files and crosses shell runtime, durable state, tests, and docs. That is acceptable because the current contradiction already spans those layers.

The smell to avoid is not "touching many files." The smell to avoid is adding new architecture while fixing the old architecture. The plan must stay boring:

1. no new public verb,
2. no new transport schema,
3. no new policy or config surface,
4. no second durable-session model,
5. no new synthetic bootstrap prompt path,
6. no new helper abstraction that hides whether work is control-only or prompt-bearing.

### Search and reuse conclusion

This is a straight Layer 1 reuse slice:

1. reuse [ResolvedRuntimeDescriptor](crates/shell/src/execution/agent_runtime/control.rs) for the launch descriptor payload,
2. reuse [OrchestrationSessionRecord](crates/shell/src/execution/agent_runtime/orchestration_session.rs) as the durable source of truth,
3. reuse [resolve_public_control_target(...)](crates/shell/src/execution/agent_runtime/state_store.rs) for public control selection,
4. reuse [run_public_prompt_command(...)](crates/shell/src/execution/agent_runtime/control.rs) and [submit_host_prompt_turn(...)](crates/shell/src/execution/agent_runtime/control.rs) for prompt-bearing host turns,
5. reuse typed world-member follow-up exactly as-is.

The missing seam is durable attach truth, not a different backend trick.

### Distribution check

This slice does not introduce a new distributable artifact. No new binary, package, or container publication work is required. The only distribution-sensitive requirement is that the workspace dependency graph resolves the published `unified-agent-api = "=0.3.5"` crate cleanly after the local patch override is removed.

## Current Repo Truth

These facts are concrete in the repository today. This plan is anchored to them.

1. [Cargo.toml](Cargo.toml) still contains `[patch.crates-io] unified-agent-api = { path = "../unified-agent-api/crates/agent_api" }`.
2. [owner_helper_startup_extensions(...)](crates/shell/src/repl/async_repl.rs) still maps `Resume`, `ResumeOneTurn`, and `Fork` to the same `agent_api.session.resume.v1` extension path.
3. [start_host_orchestrator_runtime_with_prepared_prompt(...)](crates/shell/src/repl/async_repl.rs) still converts `InitialExecPromptPlan::NoPromptRecovery` into `prompt: ""`.
4. [run_reattach(...)](crates/shell/src/execution/agents_cmd.rs) still launches through the hidden owner-helper startup path rather than a distinct control-only attach action.
5. [run_fork(...)](crates/shell/src/execution/agents_cmd.rs) still launches through that same helper path and reports the result as immediately active.
6. [build_successor_launch_plan(...)](crates/shell/src/execution/agents_cmd.rs) still infers successor launch truth from the currently active participant and its `internal_uaa_session_id`, not from durable session-level attach state.
7. [OrchestrationSessionRecord](crates/shell/src/execution/agent_runtime/orchestration_session.rs) does not yet persist a host attach contract.
8. [resolve_public_control_target(...)](crates/shell/src/execution/agent_runtime/state_store.rs) still uses retained participant continuity as the live prerequisite for detached `reattach` and `fork`.
9. [resolve_public_turn_target(...)](crates/shell/src/execution/agent_runtime/state_store.rs) already preserves detached host continuity and exact world-member linkage rules. Those must stay intact.
10. [MemberTurnSubmitRequestV1](crates/transport-api-types/src/lib.rs) and `/v1/member_turn/stream` are already the real world follow-up seam and must not change.
11. the repo is already conceptually host-rooted. The implementation bug is that control-only actions still tunnel through prompt-shaped runtime startup semantics.

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first.

### Public operator contract

1. `substrate agent start --backend <backend_id> --prompt ... --json` remains the host-only root prompt surface.
2. `substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt ... --json` remains the prompt-bearing follow-up surface.
3. `substrate agent reattach --session <orchestration_session_id> --json` remains recovery only. It does not submit a prompt.
4. `substrate agent fork --session <orchestration_session_id> --json` allocates a successor durable session. It does not submit a prompt.
5. `substrate agent stop --session <orchestration_session_id> --json` remains canonical closeout.
6. detached host sessions remain valid durable sessions,
7. detached-world follow-up remains fail-closed until host ownership is restored through the sanctioned control path.

### Durable attach contract

The attach contract introduced in this slice is the minimum durable truth required to attach later. It is not a new user-facing object and it must not carry secrets or prompt text.

Minimum required contents:

1. baseline launch descriptor sufficient to reconstruct host runtime selection,
2. explicit host execution scope and protocol truth,
3. the resolved backend identity,
4. the current private continuity selector when one exists on the owning durable session.

Fields that do not belong in this contract:

1. prompt text,
2. auth material,
3. mutable live participant identity as the sole source of truth,
4. world binding, which stays on the orchestration session itself.

### Reattach semantics in this slice

`reattach` is continuity-only in this slice.

That means:

1. the attach worker reads the durable attach contract from the orchestration session,
2. if `continuity_uaa_session_id` is present and valid for the selected session, the private attach action may use it internally,
3. if `continuity_uaa_session_id` is absent, stale, or invalid, `reattach` fails closed,
4. the failure class stays operator-legible:
   - prefer preserving `owner_unreachable`-style failure wording unless a better existing public error category already exists,
5. fresh attach from the baseline launch descriptor is explicitly deferred to slice 31,
6. success means the same durable session returns to truthful attached ownership.

### Fork semantics in this slice

`fork` allocates durable successor truth only. It does not create or attach a host execution client.

Success means:

1. a new orchestration session id is allocated first,
2. lineage is persisted on durable state first,
3. the source session's reusable attach-contract fields are copied forward,
4. the successor's `continuity_uaa_session_id` is cleared to `None`,
5. the successor's `attached_participant_id` is `null`,
6. the successor's `pending_inbox_count` is `0`,
7. the successor's posture is `parked_resumable`,
8. no prompt is submitted,
9. no backend-native session is attached or claimed on behalf of the successor.

### Prompt-bearing paths stay prompt-bearing

1. host `start` must continue to send the real user prompt as the first prompt,
2. host `turn` must continue to send the real user prompt as the follow-up prompt,
3. world-member follow-up must continue to require a non-empty prompt through `MemberTurnSubmitRequestV1`,
4. no control-only recovery action may reappear as an empty prompt or hidden bootstrap prompt.

## Target Architecture

### Canonical ownership model after this slice

```text
Public lifecycle
    start | turn | reattach | fork | stop
                   |
                   v
        Substrate durable orchestration session
        - session identity
        - lineage
        - posture
        - world binding
        - persisted host attach contract
                   |
      +------------+------------+
      |                         |
      v                         v
control-only attach worker   prompt-turn launcher
reattach only                start / turn only
no prompt                    real prompt required
      |                         |
      +------------+------------+
                   |
                   v
         attached host execution client
         optional, replaceable, not authoritative
                   |
                   v
               UAA runtime
         prompt-bearing execution only

fork path:
source durable session
    |
    v
successor allocator
    |
    v
new durable session
- copied attach contract shape
- cleared successor continuity token
- no attached participant
- parked_resumable
```

### Current-to-target delta

| Surface | Current state | Required target state |
| --- | --- | --- |
| Control attach | Hidden inside owner-helper startup using blank-prompt resume semantics | Explicit Substrate attach action that may use private continuity internally |
| Prompt-bearing host follow-up | Shares launch shaping with control-only paths | Isolated prompt-bearing path only |
| Successor allocation | Hidden helper resume path that returns `active` | Real durable successor allocation that returns honest successor posture |
| Durable attach truth | Inferred from active participant plus `internal_uaa_session_id` | Persisted on the orchestration session itself |
| Successor continuity truth | Implicitly copied by helper startup side effects | Explicitly normalized: copy structural contract, clear successor continuity token |
| UAA role | Carries both prompt execution and control meaning | Carries prompt execution only |
| Downstream 29/30/31 stack | Still partially shaped around hidden helper semantics | Frozen to the persisted attach-contract architecture |

### Recommended durable data model

Implement the attach contract as a nested durable session field rather than a separate store:

```text
OrchestrationSessionRecord
  orchestration_session_id
  posture
  world binding
  ...
  host_attach_contract
    - backend_id
    - execution_scope
    - launch_descriptor: ResolvedRuntimeDescriptor
    - continuity_uaa_session_id: Option<String>
```

Rules:

1. make the field optional and backward-compatible on deserialize,
2. persist it at host session birth,
3. update its continuity selector only when a newly attached host client becomes authoritative for that same durable session,
4. on fork:
   - copy `backend_id`,
   - copy `execution_scope`,
   - copy `launch_descriptor`,
   - set successor `continuity_uaa_session_id = None`,
5. never treat a live participant record as a substitute for this durable field again.

Why the successor continuity token is cleared:

1. the parent session still owns the existing backend-native continuity token,
2. copying that token unchanged would falsely let two durable sessions claim one backend-native lineage,
3. slice 31 is the right place to introduce explicit fresh-attach or continuity-materialization logic for born-unattached successors.

## Implementation Plan

### A0. Remove the local UAA patch override first

Primary files:

1. [Cargo.toml](Cargo.toml)
2. [Cargo.lock](Cargo.lock)
3. [crates/shell/Cargo.toml](crates/shell/Cargo.toml)
4. [crates/gateway/Cargo.toml](crates/gateway/Cargo.toml)
5. [crates/world-service/Cargo.toml](crates/world-service/Cargo.toml)

Required change:

1. delete the `[patch.crates-io] unified-agent-api = { path = "../unified-agent-api/crates/agent_api" }` override,
2. regenerate the lockfile so `unified-agent-api = "=0.3.5"` resolves from crates.io,
3. prove the workspace still compiles and tests against the published dependency source.

Why this lands first:

1. the neighboring checkout is exactly where promptless control behavior lived,
2. keeping the patch in place would let this slice accidentally depend on the wrong semantics and still appear green,
3. every later regression result is untrustworthy unless the dependency floor is corrected first.

Exit gate:

1. no local path override remains in repo manifests,
2. dependency inspection shows `unified-agent-api` is no longer sourced from `../unified-agent-api/crates/agent_api`.

### A1. Persist the minimal host attach contract on the durable session

Primary files:

1. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
2. [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
3. [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)

Required change:

1. add an optional durable `host_attach_contract` field to [OrchestrationSessionRecord](crates/shell/src/execution/agent_runtime/orchestration_session.rs),
2. define a minimal serializable contract using the existing [ResolvedRuntimeDescriptor](crates/shell/src/execution/agent_runtime/control.rs),
3. persist the contract at host session creation,
4. update the continuity selector on successful host reattach or authoritative host runtime replacement,
5. add a dedicated helper for successor-safe copy that clears the successor continuity token.

Backward-compatibility rules:

1. older session records without `host_attach_contract` must still deserialize,
2. reads must fail closed with a clear user-facing error when a control operation requires a contract that is absent,
3. migration logic must not fabricate contract state from ambiguous live runtime state.

Exit gate:

1. durable session JSON contains attach truth after host session birth,
2. detach and restart preserve the contract,
3. successor sessions inherit the contract shape with cleared successor continuity,
4. no code path reconstructs attach truth from "whatever participant is active now."

### A2. Split hidden owner-helper convergence into explicit internal launch paths

Primary files:

1. [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
2. [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
3. [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)

Required change:

1. stop modeling `Resume`, `ResumeOneTurn`, and `Fork` as one hidden startup shape,
2. introduce explicit internal paths for:
   - control-only attach,
   - prompt-bearing resumed turn launch,
   - successor durable-session allocation,
3. remove "blank prompt means control" as a shaping rule.

Implementation guidance:

1. keep public CLI verbs unchanged,
2. prefer a narrow split over a large new framework,
3. move decision-making higher, not lower:
   - `agents_cmd.rs` chooses which path is needed,
   - `control.rs` executes prompt-bearing paths,
   - `async_repl.rs` must no longer invent control semantics from prompt plans.

Explicit non-goals:

1. do not add a second prompt launcher,
2. do not widen `OwnerHelperMode` into more hidden states if that still leaves control and prompt meanings conflated.

Exit gate:

1. no launch path for `reattach` or `fork` depends on `InitialExecPromptPlan::NoPromptRecovery`,
2. the code clearly separates the three responsibilities in type and control flow.

### A3. Implement `reattach` as a control-only Substrate attach action

Primary files:

1. [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
2. [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
3. [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)

Required change:

1. make `run_reattach(...)` execute an attach action that consumes the persisted host attach contract,
2. if a valid continuity selector exists, the private attach action may use it internally to resume backend-native continuity,
3. if continuity is impossible, fail closed with an explicit user-facing error,
4. on success, converge the same durable session back to truthful attached ownership.

Important boundary:

1. "private continuity attach" is acceptable,
2. "public `reattach` means blank-prompt resume" is not,
3. fresh attach from baseline descriptor is not part of this slice.

Exit gate:

1. public `reattach` submits no prompt,
2. the same durable session id survives the round trip,
3. the action is expressed as control work in code, tests, and docs,
4. missing continuity fails closed rather than guessing.

### A4. Rework `fork` into a real successor durable-session allocator

Primary files:

1. [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
2. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
3. [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)

Required change:

1. `run_fork(...)` allocates a new orchestration session id up front,
2. successor lineage is persisted on Substrate durable state first,
3. the source session's attach-contract shape is copied to the successor,
4. the successor continuity selector is cleared to `None`,
5. no synthetic prompt-free backend turn is performed,
6. the returned successor state is honest:
   - `posture = parked_resumable`,
   - `attached_participant_id = null`,
   - `pending_inbox_count = 0`,
   - `source_orchestration_session_id` preserved for lineage truth.

Explicit non-goals:

1. do not auto-attach a new host client,
2. do not let the successor claim the parent's backend-native session,
3. do not widen `fork` into a prompt-bearing operation.

Recommended response truth:

1. `action: "fork"`
2. new `orchestration_session_id`
3. `state: "parked_resumable"`
4. `attached_participant_id: null`
5. `source_orchestration_session_id: <source>`

Exit gate:

1. no live fork path routes through `agent_api.session.resume.v1`,
2. fork no longer reports false `active` truth,
3. successor lineage and attach truth are durable before any later attach,
4. successor continuity is explicitly cleared rather than implicitly inherited.

### A5. Preserve prompt-bearing `start` and `turn` semantics exactly

Primary files:

1. [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
2. [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
3. [crates/world-service/src/member_runtime.rs](crates/world-service/src/member_runtime.rs)

Required change:

1. host `start` continues to use the real user prompt as the first prompt,
2. host `turn` continues to use the real user prompt as the follow-up prompt,
3. world-member turns continue to use the existing non-empty prompt contract,
4. no control-plane cleanup here may change `resolve_public_turn_target(...)` semantics,
5. detached-world follow-up must remain fail-closed.

This is a guardrail step, not a new feature step.

Exit gate:

1. prompt-bearing paths still look identical from the operator's point of view,
2. the only changed behavior is that `reattach` and `fork` stop pretending to be prompt-shaped runtime work.

### A6. Truth-sync docs and freeze the downstream stack

Primary files:

1. [docs/USAGE.md](docs/USAGE.md)
2. [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
3. [ADR-0047](docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
4. [UAA_PROMPTLESS_RESUME_FORK_SYNTHESIS.md](UAA_PROMPTLESS_RESUME_FORK_SYNTHESIS.md)
5. [llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
6. [llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)
7. [llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)

Required change:

1. document `reattach` as control-only attach,
2. document `fork` as successor allocation with honest detached truth,
3. document the successor-normalization rule:
   - reusable contract copies forward,
   - successor continuity token is cleared,
4. treat `UAA_PROMPTLESS_RESUME_FORK_SYNTHESIS.md` as migration evidence, not as ongoing architecture truth,
5. rewrite slice 29 as the shared dispatch-envelope generalization of the persisted attach contract,
6. keep slice 30 clearly scoped to public host-rooted world start only,
7. keep slice 31 clearly scoped to lazy attach only.

Exit gate:

1. no doc or downstream slice still lists blank-prompt control semantics as a live option,
2. no doc suggests standalone world-root continuity is still in the stack,
3. no doc implies a forked successor can reattach to the parent's backend-native continuity token as-is.

### A7. Final validation and closeout

Required change:

1. run targeted regression suites for control surfaces, successor truth, state-store persistence, and world-member routing,
2. run static grep gates for patch override removal and promptless control elimination,
3. run workspace formatting, clippy, and full tests,
4. confirm docs and downstream packets match the landed runtime truth.

Exit gate:

1. all targeted gates pass,
2. full workspace gates pass,
3. the repo tells one coherent story from code to docs to downstream plan stack.

## Code Quality Review

1. keep one durable attach contract type. Do not create parallel "launch snapshot" structs in both `agents_cmd.rs` and `state_store.rs`.
2. prefer explicit type names over mode flags. A narrow `AttachAction` or `SuccessorAllocationPlan` is better than more conditional branches on `OwnerHelperMode`.
3. do not let [async_repl.rs](crates/shell/src/repl/async_repl.rs) keep architecture ownership it should not have. It should orchestrate runtime launch, not define public control meaning.
4. keep backward-compatible serde fields optional on the durable session record. This is a state model extension, not a store migration project.
5. do not duplicate continuity validation logic. Reuse the existing posture and selector validation helpers in [state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs).
6. add an inline ASCII comment near the new durable attach contract explaining the authority split:

```text
orchestration session
  owns: identity, posture, world binding, attach contract
attached participant
  owns: current live backend client only
```

7. add an inline ASCII comment near the public control entrypoints showing the verb split:

```text
start/turn -> prompt-bearing execution
reattach   -> control-only attach
fork       -> successor allocation
stop       -> closeout
```

## Test Review

100 percent coverage is the goal for new and changed control paths. This slice changes lifecycle truth, so tests are non-negotiable.

### Coverage diagram

```text
CONTROL PATHS                                         REQUIRED TEST COVERAGE

[+] agent start (host)
  └── real user prompt -> host runtime start          [KEEP] existing public control coverage

[+] agent turn (host attached)
  └── real user prompt -> prompt-bearing resume       [KEEP] existing public control coverage

[+] agent turn (host parked)
  └── continuity-aware prompt-bearing follow-up       [KEEP] detached continuity coverage

[+] agent reattach
  └── control-only attach, no prompt                  [NEW] explicit no-prompt regression

[+] agent reattach (missing continuity)
  └── fail closed, no synthetic attach                [NEW] owner_unreachable regression

[+] agent fork
  └── successor durable session allocation            [NEW] no resume.v1, no prompt, honest posture

[+] durable session birth
  └── attach contract persisted                       [NEW] state-store persistence coverage

[+] durable session detach / restart
  └── attach contract survives                        [NEW] persistence regression

[+] successor lineage
  └── attach contract copied forward                  [NEW] successor-copy regression

[+] successor normalization
  └── continuity token cleared on child               [NEW] successor-safety regression

[+] world follow-up
  └── MemberTurnSubmitRequestV1 non-empty prompt      [KEEP] exact transport contract

[+] detached world follow-up
  └── fail closed until host ownership returns        [KEEP] fail-closed regression

COVERAGE TARGET:
- Control-only paths: 100 percent
- Durable attach-contract persistence: 100 percent
- Successor allocation semantics: 100 percent
- World prompt-bearing contract: no regressions allowed
```

### Required test files

| File or suite | Required additions | Type |
| --- | --- | --- |
| `crates/shell/tests/agent_public_control_surface_v1.rs` | prove `reattach` performs control-only attach with no prompt submission; prove `reattach` fails closed when continuity is absent; prove `fork` returns successor parked truth and no synthetic prompt path | focused integration |
| `crates/shell/tests/agent_successor_contract_ahcsitc0.rs` | extend successor-truth coverage so forked successors advertise honest posture and cleared successor continuity state | focused integration |
| `crates/shell/src/execution/agent_runtime/state_store.rs` tests | persist and reload `host_attach_contract`; fail closed when absent; copy forward on fork; clear successor continuity token | focused unit |
| `crates/shell/src/repl/async_repl.rs` tests | prove no remaining `NoPromptRecovery` path backs `reattach` or `fork` | focused unit/integration |
| `crates/shell/tests/repl_world_first_routing_v1.rs` | prove world first-turn and resumed follow-up still require real prompts and still use typed member-turn submit | focused integration |
| `crates/world-service/src/member_runtime.rs` tests | preserve retained-member tuple validation and fail-closed world semantics | focused unit |
| static repo invariant | no production path uses `prompt: ""` for public `reattach` or `fork` | grep-backed invariant |
| static repo invariant | no production path routes `fork` through `agent_api.session.resume.v1` | grep-backed invariant |
| static repo invariant | no manifest or lockfile path resolves `unified-agent-api` from the neighboring checkout | grep-backed invariant |

### Regression rules

These are mandatory blockers:

1. any change that causes `reattach` to submit a prompt,
2. any change that causes `fork` to submit a prompt,
3. any change that reports successor `active` truth without a real attached successor client,
4. any change that lets a successor inherit and reuse the parent's continuity token unchanged,
5. any change that widens or changes `MemberTurnSubmitRequestV1` follow-up semantics,
6. any change that reintroduces local path resolution to `../unified-agent-api/crates/agent_api`,
7. any change that fabricates attach truth from the current participant when the durable contract is absent.

## Error & Rescue Registry

| Situation | Expected behavior in this slice | Operator-visible result | Recovery path |
| --- | --- | --- | --- |
| `reattach` called on a parked host session with valid continuity | control-only private attach runs and restores attached ownership | success on same orchestration session id | normal `reattach` success |
| `reattach` called on a parked host session without continuity | fail closed, do not guess, do not fresh-attach | explicit failure, preferably existing `owner_unreachable` class | slice 31 later adds explicit fresh-attach path |
| `fork` called on a valid source session | allocate successor durable session only | success with `parked_resumable`, `attached_participant_id = null`, lineage included | later prompt-bearing work or later lazy attach slice |
| source session disappears during fork allocation | fail closed | explicit control failure | operator retries against a valid live source session |
| session record lacks `host_attach_contract` due to old persisted state | fail closed for control actions that require it | explicit missing-contract failure | re-create session through fresh `start`; do not synthesize state |
| docs still imply blank-prompt control semantics | block ship | review failure | update docs in same slice |

## Failure Modes Registry

| Failure mode | Where it happens | Required handling | Test requirement | Critical gap if missing |
| --- | --- | --- | --- | --- |
| `reattach` still tunnels through blank-prompt UAA resume | `agents_cmd.rs`, `async_repl.rs` | fail review and remove the convergence | explicit no-prompt reattach test plus grep gate | Yes |
| `fork` still routes through `resume.v1` | `agents_cmd.rs`, `async_repl.rs` | fail review and replace with allocator | explicit fork regression plus grep gate | Yes |
| attach contract missing on durable session birth | `orchestration_session.rs`, `state_store.rs` | fail closed and do not guess | persistence unit test | Yes |
| successor does not copy attach contract shape | `state_store.rs`, `agents_cmd.rs` | fail review and fix lineage copy | successor-copy test | Yes |
| successor retains parent's continuity token | `state_store.rs`, `agents_cmd.rs` | clear it during successor normalization | successor-safety test | Yes |
| successor reported as active without a real attached client | `agents_cmd.rs` result rendering | fix response truth to detached posture | public control test | Yes |
| world follow-up starts accepting promptless recovery semantics | `state_store.rs`, `member_runtime.rs` | fail closed and preserve typed prompt contract | world routing regression | Yes |
| docs still tell two contradictory stories | docs and llm-last-mile packets | update in same slice | doc review | No, but unacceptable at ship |

Any silent drift in control semantics, successor truth, or world follow-up routing is a release blocker for this slice.

## Performance And Complexity Review

1. this slice should add almost no CPU cost. The dominant risk is behavioral complexity, not runtime overhead.
2. do not add new per-turn inventory or policy discovery when the resolved runtime descriptor can be persisted once and reused.
3. do not add a second durable state lookup path for control actions. Reuse the existing session store and selector flow.
4. do not introduce speculative fresh attach in this slice. Continuity-only attach is simpler and keeps the change budget honest.
5. spend zero innovation tokens on new architecture. This is a truth-alignment slice. Boring is correct.

## Cross-Phase Themes

1. durable truth beats ambient inference: if a control action needs attach truth, it must read durable session state rather than guessing from the currently active participant,
2. boring beats clever: this slice removes hidden meaning instead of introducing a more elaborate helper abstraction,
3. freeze before extend: 29, 30, and 31 should build on a stable attach-contract seam, not on today's helper accident,
4. fail closed beats guess-and-heal: missing continuity or missing attach contract is a hard stop in this slice,
5. one durable session cannot dual-own one backend-native continuity token: that rule is what makes successor normalization non-negotiable,
6. prompt-bearing flows stay sacred: `start`, `turn`, and world-member follow-up keep their existing prompt semantics exactly.

## Deferred Follow-Ups / TODO Candidates

These are real follow-ons, but they are not part of this slice:

1. slice 29 generalization of the minimal attach contract into a shared dispatch envelope,
2. slice 30 public `--scope world` root start above a world worker,
3. slice 31 lazy host attach and explicit continuity-vs-fresh attach mode selection,
4. any future generic UAA control-only primitive for non-Substrate consumers,
5. any public inbox or auto-resume workflow that sits above durable pending work.

## Worktree Parallelization Strategy

This slice has limited honest parallelization. The runtime changes are coupled, but there is still a narrow safe split if module ownership stays disciplined.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A0. Dependency floor cutover | workspace manifests, lockfile | - |
| A1. Attach-contract data model | `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs` | A0 |
| A2. Launch-path split | `crates/shell/src/repl/async_repl.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/agents_cmd.rs` | A1 |
| A3. Control-only reattach path | `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/repl/async_repl.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs` | A1, A2 |
| A4. Successor allocator and fork truth | `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, control-surface tests, successor tests | A1, A2 |
| A5. Downstream truth-doc sync | `docs/`, `HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md`, `llm-last-mile/29*.md`, `30*.md`, `31*.md` | A1 |
| A6. Final validation and closeout | targeted tests, grep gates, full workspace gates | A3, A4, A5 |

### Parallel lanes

Lane 0: A0 -> A1  
Reason: the dependency floor and durable attach-contract schema must stabilize first.

Lane R: A2 -> A3  
Reason: the reattach lane depends on the explicit launch-path split and then owns the control-only attach behavior.

Lane F: A2 -> A4  
Reason: the fork lane also depends on the launch-path split, but its distinct output is successor truth and normalization.

Lane D: A5  
Reason: once the attach-contract shape is frozen, the downstream docs can be rewritten in parallel with runtime work.

Lane V: A6  
Reason: validation is the merge gate and must run after runtime and docs converge.

### Safe parallel split

The safest split is:

1. land A0 and A1 first,
2. land the shared launch-path split in A2 next,
3. after A2, run A3 and A5 in parallel,
4. run A4 in parallel with A3 only if one owner has exclusive merge authority over `agents_cmd.rs`,
5. otherwise serialize A4 immediately after A3,
6. run A6 last.

### Conflict flags

1. A3 and A4 both touch `crates/shell/src/execution/agents_cmd.rs`. That is the main merge-risk seam.
2. A1, A3, and A4 all touch `state_store.rs`. Do not start A3 or A4 until A1's attach-contract schema is merged.
3. docs must not merge before runtime truth is settled. A5 can draft early, but it should land only after A3/A4 semantics are confirmed.
4. successor tests and public-control tests are shared validation surfaces. Keep one owner per file when parallelizing.

### Execution order

1. A0 dependency floor cutover.
2. A1 attach-contract schema and persistence.
3. A2 launch-path split.
4. Launch A3 and A5 in parallel.
5. Launch A4 either in parallel with strict `agents_cmd.rs` ownership or immediately after A3.
6. Merge runtime lanes.
7. Run A6 validation and closeout.

### Parallelization verdict

This slice has:

1. one required foundation phase,
2. one shared launch-path split that must land before runtime lane fan-out,
3. one honest runtime lane for reattach,
4. one fork lane with merge-risk,
5. one docs lane,
6. one final validation lane.

Peak low-risk parallelism is `A3 + A5`. `A4` can be parallelized only if `agents_cmd.rs` ownership is tightly coordinated after A2 lands.

## Validation Commands

### Dependency resolution gates

```bash
rg -n "^\\[patch\\.crates-io\\]|unified-agent-api = \\{ path = " Cargo.toml
rg -n "unified-agent-api|path\\+file:.*/unified-agent-api/crates/agent_api" Cargo.lock
cargo tree -p shell | rg "unified-agent-api"
```

Expected result after this slice:

1. no `[patch.crates-io]` override remains,
2. `Cargo.lock` no longer references the neighboring local checkout,
3. `cargo tree` still resolves the dependency successfully.

### Static control-semantics gates

```bash
rg -n 'prompt: ""|NoPromptRecovery' \
  crates/shell/src/execution \
  crates/shell/src/repl \
  crates/world-service/src

rg -n 'agent_api.session.resume.v1' \
  crates/shell/src/execution \
  crates/shell/src/repl
```

Expected result after this slice:

1. no production `NoPromptRecovery` or `prompt: ""` path backing public `reattach` or `fork`,
2. no production `fork` path routed through `agent_api.session.resume.v1`,
3. any remaining `resume.v1` usage must be private continuity plumbing for actual prompt-bearing flows only, never the architectural meaning of public control verbs.

### Focused cargo gates

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p world-service -- --nocapture
```

### Full workspace gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

### Manual validation proof points

Manual validation for this slice must prove:

1. host `start` still uses the real user prompt as the first prompt,
2. host `turn` still uses the real user prompt as the follow-up prompt,
3. `reattach` restores the same durable session without submitting a prompt,
4. `reattach` fails closed when no continuity contract exists,
5. `fork` allocates a new durable session without submitting a prompt,
6. the fork result is honest about parked successor posture,
7. the successor does not retain the parent's continuity token,
8. the durable attach contract exists after host session birth and survives detach,
9. detached-world follow-up still fails closed until host ownership returns,
10. downstream slices 29/30/31 no longer describe blank-prompt control semantics as live architecture.

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope freeze | `reattach` remains continuity-only in this slice | locked decision | fail closed beats guess-and-heal | avoids smuggling slice 31 fresh-attach work into this correction | auto-fresh-attach during `reattach` |
| 2 | Successor truth | forked successors return `parked_resumable` with `attached_participant_id = null` | locked decision | durable truth over ambient inference | successor has not attached a client, so it must not claim active posture | helper-driven fake `active` response |
| 3 | Successor normalization | copy attach-contract shape but clear successor `continuity_uaa_session_id` | ambiguity resolution | one token, one owner | prevents two durable sessions from claiming one backend-native continuity token | copying parent continuity token unchanged |
| 4 | Sequencing | A0 and A1 land before launch-path split and runtime fan-out | execution decision | freeze before extend | later work is untrustworthy until dependency source and durable contract are stable | parallelizing schema work with runtime edits |
| 5 | Validation | successor truth gets its own targeted test coverage, not only broad workspace tests | quality gate | regressions require proof | the highest-risk behavior change is successor/control truth drift | relying on full workspace tests alone |

## Completion Summary

This plan is implementation-ready because it now freezes one architecture and one execution order:

1. Objective: Substrate owns control-only recovery and successor allocation explicitly; UAA stays prompt-bearing only.
2. Locked decisions: public verbs, durable authority, successor normalization, world follow-up transport, and downstream stack direction are fixed.
3. Step 0: the minimum honest change, reuse surfaces, complexity budget, and dependency floor are explicit.
4. Current repo truth: the exact blank-prompt convergence points and missing durable attach seam are named.
5. Target architecture: the authority split between durable session, attach contract, attach worker, prompt-turn launcher, and successor allocator is explicit.
6. Implementation plan: there is a concrete A0-A7 sequence with clear ownership boundaries, exact success/failure semantics, and exit gates.
7. Code quality review: the plan constrains abstraction growth and keeps the change boring.
8. Test review: the required coverage, regression rules, and exact suites are defined.
9. Failure modes: silent control drift, false successor truth, copied continuity aliasing, and world-follow-up regressions are all blockers.
10. Parallelization: one foundation phase, one shared launch split, one low-risk doc lane, one reattach lane, one merge-risk fork lane, and one final validation lane.
11. Validation: dependency-source proof, grep gates, targeted cargo tests, and full workspace gates are all specified.

After this slice lands, the repo should read and behave as one coherent system:

1. durable orchestration session is the authority,
2. persisted host attach contract is the durable launch truth,
3. `reattach` is control-only attach,
4. `fork` is successor allocation,
5. `start` and `turn` remain prompt-bearing,
6. the downstream stack builds on that architecture instead of working around it.

## GSTACK REVIEW REPORT

Review type: final cohesion pass with `/plan-eng-review` structure applied manually  
Outcome: approved for implementation after ambiguity closure  
Most important ambiguity resolved: successor sessions copy the attach-contract shape but do not inherit the parent's live continuity token  
Highest-risk implementation seam: shared edits in `crates/shell/src/execution/agents_cmd.rs` across reattach and fork lanes  
Parallelization included: yes  
Remaining architectural ambiguity: none intentionally left open for this slice
