# PLAN-19: Publicize Agent Control Surfaces With Exact Session Selectors

Source SOW: [19-public-agent-control-surfaces.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/19-public-agent-control-surfaces.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Adjacent landed slices: [PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-15.md), [PLAN-16.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-16.md), [PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-17.md), [PLAN-18.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-18.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: public CLI control-plane productization for orchestration sessions  
Review posture: unified execution plan, tightened to `/plan-eng-review` structure and rigor  
Status: execution-ready planning pass on 2026-05-05

## Objective

Expose one narrow public control surface under `substrate agent` that turns orchestration session lifecycle into an explicit product contract instead of a REPL-only side effect.

This slice does four things and only four things:

1. adds public `start`, `resume`, `fork`, and `stop` verbs under `substrate agent`,
2. freezes `orchestration_session_id` as the only public selector for existing sessions,
3. keeps backend-native `internal.uaa_session_id` internal while still using it for exact resume and fork,
4. fails closed anywhere Substrate cannot prove backend exactness, ownership, linkage, platform posture, or policy allowability.

This slice does not add a prompt-taking caller surface. It does not redesign `substrate -c`. It does not redesign the REPL `::<backend_id> <prompt>` grammar. It does not turn the toolbox into a mutation plane. It does not introduce a general daemon.

## Plan Summary

The repo already has most of the raw mechanics this feature needs. The missing piece is not "how to launch an agent." The missing piece is honest retained ownership once a short-lived public CLI command exits.

Repo truth today:

- the public CLI still exposes inspection only in [`AgentAction`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:471),
- status already uses permissive read-side enumeration through [`list_status_sessions_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:423) while strict control selectors stay fail closed through [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:444),
- host startup and shutdown already exist inside the REPL in [`start_host_orchestrator_runtime(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1802) and [`shutdown_host_orchestrator_runtime(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4090),
- targeted follow-up turns already shape exact resume extensions through [`build_session_resume_extension(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3776),
- the state store already treats [`orchestration_session_id`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24) as the public parent-session authority and keeps [`participant_id`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:45) and [`internal.uaa_session_id`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:75) separate,
- inventory and validator code already model `session_start`, `session_resume`, `session_fork`, and `session_stop` capability bits in [`agent_inventory.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:89) and [`validator.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:246).

The hard constraint is simple:

Public `substrate agent start|resume|fork` commands cannot return success unless something keeps the cancel handle, event stream, completion observer, and ownership heartbeat alive after the calling CLI exits. Today that owner is still process-bound to the live REPL.

This plan resolves that with one narrow owner plane:

1. extract shared orchestration lifecycle logic out of `async_repl.rs`,
2. introduce one private per-session owner loop that can run either inside the REPL or inside a hidden helper subprocess,
3. expose one private per-session control transport with exactly one v1 mutation verb, `stop`,
4. make public `start`, `resume`, and `fork` launch or reconnect that owner loop, wait for authoritative readiness, emit the public handle, then exit,
5. keep `stop` routed through the live owner process so shutdown stays authoritative instead of pretending JSON mutation is control.

That is the minimum honest diff. Anything smaller ships a surface that lies.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public inspection CLI namespace | [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs) | Extend `AgentAction`. Do not create a second top-level namespace. |
| Status vs strict control split | [`build_status_report(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:131), [`list_status_sessions_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:423), [`resolve_single_live_session_for_agent(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:444) | Reuse. Status may degrade, mutating controls may not. |
| Host orchestrator startup | [`prepare_host_orchestrator_runtime_startup(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1813), [`start_host_orchestrator_runtime_with_prepared(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1930) | Extract into shared control code. Do not duplicate in `agents_cmd.rs`. |
| Host targeted resume extension shaping | [`AGENT_API_SESSION_RESUME_V1`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1572), [`build_session_resume_extension(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3776) | Reuse the exact resume shape. |
| World-member readiness and exact-backend reuse | [`dispatch_targeted_follow_up_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2691), [`ensure_member_runtime_ready_for_descriptor(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3619) | Reuse only for Linux world-sensitive reuse/stop posture. Do not broaden into public world-root start. |
| Authoritative parent session model | [`OrchestrationSessionRecord`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:24) | Keep `orchestration_session_id` as the public parent handle. |
| Participant lineage model and internal UAA-handle separation | [`AgentRuntimeParticipantHandle`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:45), [`AgentRuntimeSessionInternal`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:75) | Preserve. Public commands may read `internal.uaa_session_id`, never accept it as input. |
| Exact backend selection | [`validate_exact_backend_selection(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:246) | Reuse as the only public `start --backend` selector. |
| Capability gates | [`missing_required_orchestrator_capability(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:96) | Reuse for public start, resume, fork, and stop eligibility checks. |
| Unified Agent API selector grammar | local dependency `unified-agent-api 0.2.3` | Freeze `resume` and `fork` to the same `{selector,id}` object shape. Never send both together. |
| Internal shutdown path | [`shutdown_host_orchestrator_runtime(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4090) | Reuse through the owner loop only. Never fake stop by rewriting `session.json`. |

### Exact remaining gap

The remaining gap is narrower than the SOW implied:

1. the public CLI namespace still has no `start|resume|fork|stop` verbs,
2. the lifecycle code public verbs need is still trapped inside `async_repl.rs` private types and functions,
3. there is no durable owner loop outside the live REPL, so a short-lived public `start` would currently lose authoritative control on exit,
4. there is no strict public-action resolver that takes exact `orchestration_session_id`, rejects non-canonical handles, validates linkage, then selects action-specific behavior,
5. there is no public stop transport to reach the retained owner,
6. `fork` capability is advertised by inventory and UAA backend bindings, but the shell does not yet shape or route a fork extension on the orchestration path,
7. the docs still describe public control-plane productization as missing and do not explain the owner-model constraint that makes a naive immediate-return CLI invalid.

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first.

### Non-negotiable invariants

1. Public existing-session targeting accepts only `--session <orchestration_session_id>`.
2. Public root-session creation accepts only `--backend <backend_id>`.
3. No public command accepts `participant_id`, `active_session_handle_id`, `session_handle_id`, or `internal.uaa_session_id` as input.
4. `start`, `resume`, and `fork` are short-lived launch commands. They return only after the owner loop has established authoritative state or failed explicitly.
5. `stop` never rewrites state-store rows directly to terminal. It must reach the live owner loop and let that owner perform authoritative shutdown.
6. Strict control-plane actions stay fail closed. No status-style warning degradation is allowed in `start`, `resume`, `fork`, or `stop`.
7. Public root session creation is host-orchestrator only in v1. A world-scoped backend is not a valid public root target.
8. World-sensitive stop or reuse logic remains Linux-first. If the selected live session depends on authoritative shared-world/member-runtime posture that the current platform cannot prove, return `unsupported_platform_or_posture`.
9. `resume` and `fork` use exact UAA selector objects and are mutually exclusive.
10. Public output may surface `participant_id` for debugging, but `orchestration_session_id` is always the public session handle.
11. This slice does not add prompt submission. A session can be created, resumed, forked, or stopped without widening caller semantics.
12. No new general daemon is introduced. The only new always-on ownership surface is a per-session owner loop.

### Public verb contract

```text
substrate agent start  --backend <backend_id> [--json]
substrate agent resume --session <orchestration_session_id> [--json]
substrate agent fork   --session <orchestration_session_id> [--json]
substrate agent stop   --session <orchestration_session_id> [--json]
```

### Exact selector rules

| Command | Required selector | Resolution rule | Explicitly rejected |
| --- | --- | --- | --- |
| `start` | `--backend <backend_id>` | exact host-scoped backend match through `validate_exact_backend_selection(...)` | agent id, label, default backend, world-scoped root start |
| `resume` | `--session <orchestration_session_id>` | exact parent session id, exact active participant lookup, exact internal `uaa_session_id` reuse | `participant_id`, `uaa_session_id`, fuzzy "latest" |
| `fork` | `--session <orchestration_session_id>` | exact parent session id, exact active participant lookup, exact internal `uaa_session_id` fork source | `participant_id`, `uaa_session_id`, fuzzy "latest" |
| `stop` | `--session <orchestration_session_id>` | exact parent session id, exact live owner transport, exact owner-mediated shutdown | direct store mutation, PID-only heuristic, fuzzy "latest" |

### Success output contract

| Field | `start` | `resume` | `fork` | `stop` |
| --- | --- | --- | --- | --- |
| `action` | `start` | `resume` | `fork` | `stop` |
| `orchestration_session_id` | new session id | existing session id | new session id | existing session id |
| `source_orchestration_session_id` | absent | absent | present | absent |
| `backend_id` | present | present | present | present |
| `scope` | `host` | `host` or `host_with_world_posture` | `host` or `host_with_world_posture` | same as resolved live session |
| `participant_id` | new active participant | new active participant | new active participant | final active participant before stop, optional |
| `state` | `active` | `active` | `active` | `stopped` or `invalidated` |
| `warnings` | empty | empty | empty | empty |

Rules:

1. no success output includes `internal.uaa_session_id`,
2. text mode prints the same information as a single deterministic summary line,
3. JSON mode is stable and exact so tests can pin it,
4. no success path reports readiness until authoritative state-store proof exists.

### Failure taxonomy to freeze now

Use stable operator-facing reason names. These are contract values, not implementation comments:

- `unknown_backend`
- `ambiguous_backend`
- `unknown_session`
- `session_already_owned`
- `missing_active_parent`
- `stale_linkage`
- `missing_internal_session_id`
- `unsupported_platform_or_posture`
- `world_boundary_unavailable`
- `policy_disallow`
- `owner_unreachable`
- `fork_not_supported`

No command in this slice should return a vague "not available" when one of the exact cases above is true.

## Step 0: Scope Challenge

### 0A. Minimum honest diff

The minimum honest implementation is:

1. add public `AgentAction::{Start, Resume, Fork, Stop}` plus exact arg structs in [`cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs),
2. add one shared orchestration control module under `crates/shell/src/execution/agent_runtime/` that owns:
   - shared host bootstrap and shutdown extraction from `async_repl.rs`,
   - exact public-control session resolution,
   - resume and fork extension shaping,
   - owner-loop bootstrap,
   - private per-session stop transport,
   - public result rendering inputs,
3. teach the REPL owner path to register the same private owner transport so public `stop` can target REPL-owned sessions too,
4. add a hidden helper-owner entrypoint so public `start`, `resume`, and `fork` can launch a retained owner loop without keeping the invoking CLI process alive,
5. add strict tests for selectors, owner-loop launch, stop routing, resume, fork, and unsupported posture.

Anything smaller leaves the public surface fake.

### 0B. Complexity check

This slice trips the file-count smell, but it does not justify splitting the feature.

Expected production files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)
4. `crates/shell/src/execution/agent_runtime/control.rs` (new)
5. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
6. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
7. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
8. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Expected tests and docs:

1. `crates/shell/tests/agent_public_control_surface_v1.rs` (new)
2. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
3. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
4. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

Why the scope still stands:

1. there is only one new real abstraction, the control module plus owner loop,
2. splitting `start|stop` and `resume|fork` into separate PRs duplicates the owner model and selector contract,
3. not solving ownership now means the public surface ships with a known lie.

### 0C. Search and completeness check

Search-before-building result, in practical terms:

- **[Layer 1]** reuse exact backend selection and capability gates from [`validator.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs),
- **[Layer 1]** reuse authoritative parent and participant persistence from [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs),
- **[Layer 1]** reuse host runtime startup and shutdown machinery from [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs),
- **[Layer 1]** reuse UAA resume wire shaping exactly,
- **[Layer 1]** reuse the already-supported UAA fork selector grammar instead of inventing a Substrate-only fork shape,
- **[EUREKA]** the blocker is not "missing start/stop code." The blocker is that retained control ownership is still process-local,
- **[EUREKA]** the smallest complete version is not a daemon. It is one per-session owner loop with one private stop transport plus extracted shared lifecycle logic.

### 0D. Distribution and runtime contract check

No new downloadable artifact type is introduced.

The distribution requirement here is behavioral truth:

1. public verbs must exist in CLI help,
2. they must emit exact stable JSON,
3. they must not promise root world-session start,
4. they must not imply prompt submission exists,
5. they must keep live ownership real after the launching command exits.

### 0E. What already exists

Sub-problem reuse is locked:

| Sub-problem | Existing code | Reuse rule |
| --- | --- | --- |
| exact backend targeting | [`validate_exact_backend_selection(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:246) | Reuse. No new selector logic. |
| parent session lookup by exact id | [`load_session(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:371) | Reuse as the read-side base, then layer a strict control resolver on top. |
| retained owner lifecycle | [`RuntimeOrchestrationContext`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1601), [`PreparedAgentRuntime`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1627), [`AsyncReplAgentRuntime`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1654) | Extract, do not clone. |
| exact resume extension wire shape | [`build_session_resume_extension(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3776) | Reuse for `resume`. |
| authoritative stop path | [`shutdown_host_orchestrator_runtime(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4090) | Reuse through the owner loop only. |
| strict control-surface posture | [`run_doctor(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:138), toolbox commands in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:74) | Preserve. Public control verbs join this strict family. |

### 0F. NOT in scope

- redesigning `substrate -c`
- adding any prompt-taking public caller surface
- default-agent routing
- member-level public selectors
- root world-session start for world-scoped backends
- using toolbox as a mutation plane
- a general daemonized control plane or long-lived agent-hub service
- macOS/Lima parity for world-sensitive control semantics
- storage flag-day renames away from `active_session_handle_id`
- broader session-history or list-sessions product work

## Architecture Review

### Locked architecture decisions

1. Add one new shared control module, `crates/shell/src/execution/agent_runtime/control.rs`.
2. That module owns public lifecycle orchestration, exact session resolution, owner-loop bootstrap, private stop transport, and public result rendering inputs.
3. `async_repl.rs` stops being the sole owner of host bootstrap and shutdown logic and instead consumes the shared control module.
4. Public `start`, `resume`, and `fork` spawn a hidden owner helper or attach to the REPL owner path, then return only after authoritative readiness is visible in the state store.
5. Public `stop` connects to the private owner transport for the exact `orchestration_session_id` and asks the live owner to stop authoritatively.
6. Root public session creation is host-orchestrator only. World-sensitive posture matters only when existing live sessions already own authoritative world/member state.
7. `resume` stays in the existing parent orchestration session.
8. `fork` creates a fresh parent orchestration session and links provenance through participant lineage, not a brand-new lineage schema.

### Architecture findings resolved in-plan

**Issue 1. The current runtime owner dies with the calling process.**

That is the blocker. A public lifecycle command that returns immediately without transferring control ownership is fake productization. The plan fixes that with a per-session owner loop.

**Issue 2. Public `stop` cannot be implemented honestly by mutating JSON.**

The retained cancel handle lives in memory. The public stop path must hit the process that owns it. That is why the plan introduces a private owner transport and routes `stop` through the owner.

**Issue 3. `start` cannot mean "any backend".**

The parent public handle is an orchestration session, and the current runtime architecture makes the orchestrator the parent authority. A world member is a subordinate participant, not a valid v1 root target.

**Issue 4. `resume` and `fork` are close, but not the same thing.**

`resume` rebinds the existing orchestration session to a new live participant in the same parent row. `fork` creates a new parent session, new live participant, and a lineage link back to the source active participant. If those semantics blur, operators lose track of which session id they should stop later.

**Issue 5. Public control verbs belong in the strict family, not the status family.**

This slice lands after `PLAN-18`. Read-side inspection may degrade. The control side still fails closed. Any implementation that lets `start|resume|fork|stop` "pick the only likely session" is wrong.

### Hidden owner-helper contract

V1 needs one private owner-helper entrypoint. That entrypoint is implementation-private but behaviorally frozen.

Required contract:

1. it is not documented in public help,
2. it can be reached only by internal command dispatch from `substrate agent start|resume|fork`,
3. it receives a fully resolved execution plan, not fuzzy CLI inputs,
4. it owns the live cancel handle, event stream task, completion observer, and readiness transition,
5. it writes authoritative state transitions into the existing session store,
6. it registers the same private stop transport that REPL-owned sessions register.

Required modes:

- `start`
- `resume`
- `fork`

Required inputs after public resolution:

- `orchestration_session_id`
- `backend_id`
- `workspace_root`
- `selector_mode` as `start|resume|fork`
- `resume_or_fork_source_uaa_session_id` when needed
- any validated world posture metadata when the action is allowed to proceed

The helper must not perform fuzzy selection. By the time it starts, selection is done.

### Private owner transport contract

There is exactly one v1 owner mutation request: `stop`.

Required transport rules:

1. one private endpoint per live `orchestration_session_id`,
2. derived from Substrate-owned runtime state plus exact session id,
3. platform-native transport is allowed, but the addressing contract is session-exact,
4. no global listener or shared multi-session broker is introduced,
5. no toolbox reuse,
6. no PID-only signaling contract.

Required request and response shape:

```json
{"version":1,"action":"stop"}
```

Response outcomes:

- `accepted`
- `already_terminal`
- `owner_unreachable`
- `protocol_error`

Public `substrate agent stop` does not report success on `accepted` alone. It must wait until the parent orchestration session becomes `stopped` or `invalidated`.

### Parent-session state machine

Parent-session state transitions for this slice:

```text
allocating -> active -> stopping -> stopped
allocating -> failed
active -> invalidated
stopping -> invalidated
```

Rules:

1. public `start`, `resume`, and `fork` return success only after `active`,
2. public `stop` returns success only after `stopped` or `invalidated`,
3. if the owner process dies before readiness, the parent session becomes `failed` or `invalidated`, never silent,
4. an active parent must point at exactly one active participant for control operations,
5. `fork` creates a new parent session row. It never mutates the source parent into the fork target.

### Architecture ASCII diagrams

### Public command to owner flow

```text
PUBLIC CLI
==========
substrate agent start|resume|fork|stop
    |
    +--> start --backend <backend_id>
    |      |
    |      +--> validate_exact_backend_selection(host, backend_id)
    |      +--> allocate orchestration_session_id
    |      +--> spawn hidden owner helper
    |      +--> wait for authoritative parent active + active participant + surfaced uaa_session_id
    |      +--> emit public result
    |
    +--> resume --session <orchestration_session_id>
    |      |
    |      +--> resolve exact parent session
    |      +--> reject if already owner-live
    |      +--> read active participant internal.uaa_session_id
    |      +--> spawn hidden owner helper in resume mode
    |      +--> wait for same parent session to rebind active participant
    |      +--> emit public result
    |
    +--> fork --session <orchestration_session_id>
    |      |
    |      +--> resolve exact source session
    |      +--> read active participant internal.uaa_session_id
    |      +--> allocate new orchestration_session_id
    |      +--> spawn hidden owner helper in fork mode
    |      +--> wait for new parent session active
    |      +--> emit public result with source_orchestration_session_id
    |
    +--> stop --session <orchestration_session_id>
           |
           +--> resolve exact live session
           +--> derive exact owner transport from orchestration_session_id
           +--> owner transport: stop
           +--> wait for terminal parent state
           +--> emit public result
```

### Owner-model split

```text
LIVE ORCHESTRATION OWNERSHIP
============================

interactive REPL session
    |
    +--> shared control module
    +--> retained cancel/event/completion handles
    +--> private owner transport for stop

public start/resume/fork helper
    |
    +--> shared control module
    +--> retained cancel/event/completion handles
    +--> private owner transport for stop

public stop
    |
    +--> exact orchestration_session_id
    +--> exact owner transport
    +--> authoritative shutdown
```

### Resume vs fork lineage

```text
RESUME
======
existing orchestration_session_id = sess_A
existing active participant       = ash_old
existing internal.uaa_session_id  = uaa_123

resume(sess_A)
    |
    +--> launch new attached owner with session.resume(id = uaa_123)
    +--> create successor participant ash_new
    +--> ash_new.resumed_from_participant_id = ash_old
    +--> parent session stays sess_A
    +--> active participant moves to ash_new

FORK
====
source orchestration_session_id   = sess_A
source active participant         = ash_old
source internal.uaa_session_id    = uaa_123

fork(sess_A)
    |
    +--> allocate new orchestration_session_id = sess_B
    +--> launch new attached owner with session.fork(id = uaa_123)
    +--> create new active participant ash_fork
    +--> ash_fork.resumed_from_participant_id = ash_old
    +--> parent session is new sess_B
    +--> source sess_A remains unchanged
```

## Code Quality Review

### Findings resolved in-plan

1. Do not copy host bootstrap, state persistence, or shutdown logic out of `async_repl.rs` into `agents_cmd.rs`. That creates two lifecycle implementations that drift.
2. Do not implement separate ad hoc validators for `resume`, `fork`, and `stop`. Add one strict resolver in the control module and make each verb layer only its extra action-specific checks.
3. Do not let public command handlers know the shape of UAA extension payloads directly. Add one helper for `resume`, one helper for `fork`, and pin the mutual exclusion rule in tests.
4. Do not conflate "public selector" and "debugging field". `participant_id` may be surfaced. It must never become an accepted target.
5. Do not hard-code PID signaling as the control-plane contract. PID liveness remains a validation signal, not the stop transport.
6. Keep compatibility alias reads exactly where they are today, at serde and read boundaries. Public control naming moves forward without a storage flag day.

### Required code comments and diagrams

Add or update nearby ASCII comments in these places if code lands there:

1. `control.rs`, owner loop and stop transport flow
2. `async_repl.rs`, if the REPL path starts registering the shared owner transport
3. `state_store.rs`, if a new public-control resolver introduces non-obvious linkage validation

Stale diagrams are worse than no diagrams. If a nearby runtime-ownership diagram becomes wrong, update it in the same change.

## Test Review

### Test framework detection

This repo is Rust-first and the relevant review surface is `cargo test`.

Primary suites for this slice:

1. `crates/shell/tests/agent_public_control_surface_v1.rs` (new integration suite)
2. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
3. targeted unit tests in [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
4. targeted unit tests in [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) when extraction leaves local invariants behind

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/cli.rs
    |
    ├── AgentAction::Start / Resume / Fork / Stop parse
    │   ├── [GAP] exact required selector flags
    │   ├── [GAP] no prompt argument accepted
    │   └── [GAP] hidden owner-helper entrypoint stays hidden

[+] crates/shell/src/execution/agent_runtime/control.rs
    |
    ├── start path
    │   ├── [GAP] exact host backend match launches owner helper
    │   ├── [GAP] world-scoped backend root start rejects explicitly
    │   ├── [GAP] policy-disallowed backend rejects explicitly
    │   └── [GAP] readiness succeeds only after parent active + active participant + surfaced uaa_session_id
    |
    ├── resume path
    │   ├── [GAP] exact session id resolves active participant + internal.uaa_session_id
    │   ├── [GAP] already-owned live session rejects with session_already_owned
    │   ├── [GAP] missing internal.uaa_session_id rejects explicitly
    │   └── [GAP] resumed participant rebinds the same orchestration_session_id
    |
    ├── fork path
    │   ├── [GAP] exact session id resolves active participant + internal.uaa_session_id
    │   ├── [GAP] fork extension uses exact {selector,id} object
    │   ├── [GAP] new orchestration_session_id is allocated
    │   └── [GAP] new participant links back to source active participant
    |
    ├── stop path
    │   ├── [GAP] exact session id resolves live owner transport
    │   ├── [GAP] owner transport stop request triggers authoritative shutdown
    │   ├── [GAP] owner unreachable rejects explicitly
    │   └── [GAP] command waits for terminal state, not just request acceptance
    |
    └── selector rejection
        ├── [GAP] participant_id is not accepted as session selector
        ├── [GAP] internal.uaa_session_id is not accepted as session selector
        └── [GAP] fuzzy or latest lookup is never attempted

[+] crates/shell/src/repl/async_repl.rs
    |
    ├── REPL owner path
    │   ├── [★★★ TESTED] host startup and shutdown already work
    │   ├── [GAP] REPL-owned live session registers the shared owner transport
    │   └── [GAP] stop over owner transport shuts down authoritatively without prompt-path regressions
    |
    ├── targeted follow-up submit
    │   └── [★★★ TESTED] resume extension wiring already exists and must stay green
    |
    └── member runtime ownership
        └── [★★★ TESTED] Linux retained-member reuse and shutdown remain the source of world-sensitive truth

[+] crates/shell/src/execution/agent_runtime/state_store.rs
    |
    ├── exact public-control session resolution
    │   ├── [GAP] missing active parent rejects
    │   ├── [GAP] stale linkage rejects
    │   ├── [GAP] inactive owner rejects when required
    │   └── [GAP] exact session-id lookup does not consult trace history
    |
    └── canonical naming
        └── [★★★ TESTED] orchestration_session_id remains the parent authority

---------------------------------
COVERAGE TARGET
- every public verb has a direct success-path test
- every public verb has exact selector rejection tests
- owner-model honesty is proven, not assumed
- existing REPL targeted-turn tests remain green
---------------------------------
```

### Operator flow coverage

```text
OPERATOR FLOW COVERAGE
===========================
[+] Operator runs `substrate agent start --backend <host_backend> --json`
    |
    ├── [GAP] receives orchestration_session_id only after authoritative readiness
    └── [GAP] receives explicit rejection for world-scoped root backend

[+] Operator runs `substrate agent resume --session <orchestration_session_id> --json`
    |
    ├── [GAP] exact orphaned session resumes successfully
    ├── [GAP] already-owned live session rejects with clear reason
    └── [GAP] missing internal session id rejects with clear reason

[+] Operator runs `substrate agent fork --session <orchestration_session_id> --json`
    |
    ├── [GAP] returns a new orchestration_session_id
    ├── [GAP] preserves source session unchanged
    └── [GAP] rejects if backend/runtime does not support session.fork.v1

[+] Operator runs `substrate agent stop --session <orchestration_session_id> --json`
    |
    ├── [GAP] owner transport accepts stop and session becomes terminal
    ├── [GAP] unreachable owner rejects explicitly
    └── [GAP] no silent state-store mutation occurs

[+] Operator tries non-canonical handles
    |
    ├── [GAP] participant_id is rejected
    └── [GAP] internal.uaa_session_id is rejected

[+] Operator targets a world-sensitive live session on non-Linux
    |
    └── [GAP] returns unsupported_platform_or_posture, not a fake partial success
```

### Required tests to add or extend

1. Add a public start integration test proving exact host backend startup returns only after authoritative session activation and surfaced session-handle persistence.
2. Add a public start rejection test proving a world-scoped backend is not a valid v1 root selector.
3. Add a public resume integration test proving an exact historical session with persisted `internal.uaa_session_id` rebinds the same `orchestration_session_id` to a new active participant.
4. Add a public resume rejection test for `session_already_owned`.
5. Add a public fork integration test proving a new orchestration session is created and source-session lineage is preserved through `resumed_from_participant_id`.
6. Add a public fork rejection test for missing fork capability or unsupported backend.
7. Add a public stop integration test proving the owner transport drives authoritative shutdown and the command waits for a terminal parent-session state.
8. Add selector rejection tests proving `participant_id`, `session_handle_id`, and `internal.uaa_session_id` are not accepted as public selectors.
9. Extend REPL tests to prove REPL-owned live sessions register the shared owner transport and remain stoppable through the same private owner-plane contract.
10. Keep all targeted-turn resume tests green. That wire shape is shared truth now.

### QA-facing test artifact

During implementation, write a QA-facing artifact to:

```text
~/.gstack/projects/<slug>/<user>-feat-session-centric-state-store-eng-review-test-plan-<timestamp>.md
```

Required contents:

1. public host start flow,
2. public resume flow for an orphaned session,
3. public fork flow producing a new session id,
4. public stop flow against a live owner,
5. selector rejection flows for wrong handle types,
6. non-Linux world-sensitive rejection flow.

This artifact is for `/qa` and `/qa-only`. Keep it operator-journey oriented, not implementation-oriented.

### Regression rule for this slice

These tests are mandatory:

1. existing REPL targeted-turn resume coverage stays green,
2. strict doctor and toolbox behavior stays green,
3. public `start` never reports success before authoritative readiness,
4. public `stop` never marks success from a dead owner transport,
5. `resume` and `fork` never accept both selector extensions at once,
6. no public command accepts `internal.uaa_session_id` as input.

## Failure Modes Registry

| Failure mode | Test required | Error handling exists | Operator sees clear result | Critical gap before this slice lands |
| --- | --- | --- | --- | --- |
| public `start` returns before ownership is established | yes | no | no | yes |
| public root start accepts a world-scoped backend and creates an unusable session | yes | no | no | yes |
| `resume` targets a still-owned live session and creates split brain | yes | no | no | yes |
| `resume` or `fork` silently uses `participant_id` or `uaa_session_id` as fallback selector | yes | no | no | yes |
| `stop` rewrites state store directly while retained control is still live | yes | no | no | yes |
| `stop` cannot reach the live owner and still reports success | yes | no | no | yes |
| `fork` reuses the source orchestration session id instead of creating a new one | yes | no | no | yes |
| world-sensitive live session appears stoppable or resumable on unsupported platform | yes | partial today via Linux-only runtime behavior | no | yes |
| REPL-owned live session is not reachable by the public stop path | yes | no | no | yes |
| canonical naming regresses and output surfaces `session_handle_id` as the public handle | yes | partial today in comments only | no | yes |

Critical-gap rule for this plan:

No public control action is allowed to be both state-changing and unowned. If the command cannot prove who owns the retained control plane, it must fail closed.

## Performance Review

Performance is not the main risk here, but the owner model can still get sloppy if the plan overbuilds.

### Findings resolved in-plan

1. Public control resolution must read exact session state from the authoritative store, not rescan trace history or broad inventory multiple times.
2. Readiness wait loops must poll bounded state transitions with backoff, not spin.
3. The owner transport is per-session and human-paced. It does not need a global listener registry or background reconciler.
4. Reusing the shared lifecycle module is cheaper and safer than cloning runtime launch paths into two executors.
5. World-sensitive rejection must happen before any expensive ready or rebind path when platform posture is unsupported.

### Performance posture

- no new trace scan is acceptable on public control paths,
- no new global daemon or watchdog is needed,
- no background polling loop beyond bounded readiness waits is needed,
- state-store lookups remain exact and small,
- correctness and explicit failure win over shaving a few milliseconds from human-paced commands.

## DX Guardrails

This is a developer and operator surface even though it is backend-heavy.

Required operator experience:

1. every command names either the exact `backend_id` or exact `orchestration_session_id` it acted on,
2. no command tells the operator to use `participant_id` or `uaa_session_id`,
3. stop failures distinguish `owner_unreachable` from `unknown_session`,
4. host-root-only start is explicit, not buried behind a confusing runtime-realizability error,
5. world-sensitive unsupported posture is explicit and named,
6. JSON output is stable and scriptable from day one.

## Worktree Parallelization Strategy

This plan has real parallelization opportunities once the owner model is frozen.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Freeze public verb contract, owner model, output schema, and error taxonomy | `crates/shell/src/execution/`, `crates/shell/src/repl/`, repo docs | — |
| Shared control module extraction and REPL integration | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | Freeze public verb contract, owner model, output schema, and error taxonomy |
| Public CLI and command-handler wiring | `crates/shell/src/execution/` | Freeze public verb contract, owner model, output schema, and error taxonomy |
| Exact session resolver and canonical naming guardrails | `crates/shell/src/execution/agent_runtime/` | Freeze public verb contract, owner model, output schema, and error taxonomy |
| Integration tests and repo-truth closeout | `crates/shell/tests/`, repo docs | Shared control module extraction and REPL integration, Public CLI and command-handler wiring, Exact session resolver and canonical naming guardrails |

### Parallel lanes

- Lane A: shared control module extraction and REPL integration
  - sequential inside the lane because these steps share `crates/shell/src/repl/` and the new control module
- Lane B: public CLI and command-handler wiring
  - sequential inside the lane because these steps share `crates/shell/src/execution/cli.rs` and `agents_cmd.rs`
- Lane C: exact session resolver and canonical naming guardrails
  - sequential inside the lane because these steps share `crates/shell/src/execution/agent_runtime/`
- Lane D: integration tests and repo-truth closeout
  - starts after A, B, and C merge

### Execution order

1. Freeze the public contract and owner model.
2. Launch Lane A, Lane B, and Lane C in parallel worktrees.
3. Merge A, B, and C.
4. Run Lane D for public control integration tests, REPL non-regression, and doc closeout.

### Conflict flags

- Lane A and Lane C both touch `agent_runtime/` semantics. Freeze the resolver contract before parallel work starts or there will be two equally plausible but incompatible owner models.
- Lane B must not invent public output fields independently. The output schema is frozen up front.
- Lane D owns the new `agent_public_control_surface_v1.rs` suite. Keep that ownership there to avoid test churn across implementation lanes.
- Docs move last. Updating the gap matrix before the owner model and tests land will drift repo truth again.

### Parallelization verdict

Four workstreams, three parallel implementation lanes, one final integration lane.

## Implementation Sequence

### Step 1. Freeze the public control contract

Files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)
4. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) for language freeze only if needed during implementation

Deliver:

1. add public `Start`, `Resume`, `Fork`, and `Stop` actions plus exact args,
2. freeze root-start as host-only in v1,
3. freeze the public JSON result fields and exact error taxonomy,
4. freeze the owner-loop and owner-transport contract,
5. freeze the rule that `resume` stays in the same parent session and `fork` creates a new parent session.

Done means the surface contract is explicit before lifecycle extraction begins.

### Step 2. Extract the shared control module

Files:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. [crates/shell/src/execution/agent_runtime/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)
3. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Deliver:

1. extract shared host lifecycle pieces out of `async_repl.rs`,
2. move resume-extension shaping into the shared control module,
3. add fork-extension shaping with the exact UAA selector object,
4. add the private per-session owner loop and owner transport,
5. keep REPL startup and targeted-turn behavior functionally unchanged apart from reusing the shared control module.

Done means there is one lifecycle implementation, not one REPL copy and one CLI copy.

### Step 3. Add the strict public-control resolver

Files:

1. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
2. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
3. [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
4. `crates/shell/src/execution/agent_runtime/control.rs`

Deliver:

1. add an exact public-control session resolver layered on authoritative state-store reads,
2. validate:
   - parent session exists,
   - active participant exists when the action requires it,
   - owner linkage is alive when the action requires a live owner,
   - `internal.uaa_session_id` exists for resume and fork,
   - world-sensitive posture is Linux-supported when the resolved live session requires it,
3. reject all non-canonical handle forms explicitly,
4. keep status degradation untouched.

Done means public actions can target exact session ids without consulting trace fallback or guessing.

### Step 4. Wire public `start`

Files:

1. [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. `crates/shell/src/execution/agent_runtime/control.rs`
4. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) if helper launch shares REPL owner-state machinery

Deliver:

1. validate exact host backend selection,
2. allocate a new `orchestration_session_id`,
3. launch the hidden owner helper,
4. wait for authoritative readiness, not process spawn alone,
5. emit the stable public result.

Done means `substrate agent start --backend <backend_id>` is real and does not lose ownership when the command exits.

### Step 5. Wire public `resume` and `fork`

Files:

1. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Deliver:

1. `resume`
   - resolve exact session,
   - reject if already owner-live,
   - read exact `internal.uaa_session_id`,
   - launch hidden owner helper with `agent_api.session.resume.v1`,
   - rebind the same parent session to a new active participant,
2. `fork`
   - resolve exact source session,
   - read exact `internal.uaa_session_id`,
   - allocate a new `orchestration_session_id`,
   - launch hidden owner helper with `agent_api.session.fork.v1`,
   - create a new parent session and participant lineage back to the source active participant,
3. enforce resume and fork mutual exclusion in request shaping and tests.

Done means `resume` and `fork` both exist, mean different things, and remain exact.

### Step 6. Wire public `stop`

Files:

1. [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Deliver:

1. derive the exact owner transport from `orchestration_session_id`,
2. teach REPL-owned sessions and helper-owned sessions to register that transport,
3. accept exactly one v1 owner request, `stop`,
4. route `stop` to the authoritative shutdown path,
5. wait for terminal state and emit stable result.

Done means `stop` is real for any live session that advertises the owner plane and never fakes success through local JSON mutation.

### Step 7. Freeze the contract with tests and repo-truth docs

Files:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
3. [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)
4. [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

Deliver:

1. add success and rejection coverage for all four public verbs,
2. keep strict doctor and toolbox behavior green,
3. keep REPL targeted-turn resume behavior green,
4. update the gap matrix to say the public control family is landed for host-orchestrator sessions, with Linux-first world-sensitive stop and reuse posture,
5. update the planning index to include `PLAN-19`.

Done means the repo says what the runtime actually does.

## Recommended Verification Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell async_repl -- --nocapture
```

Manual spot checks after tests are green:

```bash
substrate agent start --backend <host_backend_id> --json
substrate agent resume --session <orchestration_session_id> --json
substrate agent fork --session <orchestration_session_id> --json
substrate agent stop --session <orchestration_session_id> --json
substrate agent status --json
substrate agent doctor --json
substrate agent toolbox env --json
```

## Definition of Done

1. `substrate agent` publicly exposes `start`, `resume`, `fork`, and `stop`.
2. `start` accepts only exact host-scoped `backend_id` selectors.
3. `resume`, `fork`, and `stop` accept only exact `orchestration_session_id` selectors.
4. no public command accepts or emits `internal.uaa_session_id` as a selector.
5. `start`, `resume`, and `fork` return only after authoritative readiness is visible in the store.
6. `stop` routes through the live owner and reaches a terminal parent-session state.
7. `resume` rebinds the same parent session. `fork` creates a new parent session.
8. REPL-owned live sessions expose the same private owner plane for public stop.
9. strict doctor and toolbox behavior remain fail closed.
10. root world-session start is rejected explicitly.
11. world-sensitive control remains Linux-first and fail closed elsewhere.
12. repo-truth docs reflect landed behavior.

## Deferred Work

- prompt-taking public caller surfaces
- `substrate -c` redesign
- member-level public control selectors
- root world-session start
- world-sensitive control parity on macOS/Lima and Windows/WSL
- any general daemonized agent-hub service
- toolbox mutation tools
- broader session-history and list-sessions product work

## Completion Summary

- Step 0: Scope Challenge, scope accepted as-is after correcting the owner-model gap
- Architecture Review: 5 issues found, all resolved in-plan
- Code Quality Review: 6 issues found, all resolved in-plan
- Test Review: diagrams produced, 10 concrete regression gaps identified
- Performance Review: 5 issues found, all resolved in-plan
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0 items proposed, no `TODOS.md` exists in this repo today
- Failure modes: 10 critical gaps flagged until the public owner plane and regression wall land
- Outside voice: skipped for this document pass
- Parallelization: 4 workstreams, 3 parallel implementation lanes, 1 final integration lane
- Lake Score: 10/10 recommendations chose the complete option

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Step 0 | Treat retained ownership as the primary blocker, not missing clap variants | Mechanical | Pragmatic | Public verbs are fake if ownership dies on CLI exit | wiring CLI actions first and hoping lifecycle details sort themselves out |
| 2 | Architecture | Add one shared control module instead of cloning `async_repl.rs` logic into `agents_cmd.rs` | Mechanical | DRY | One lifecycle truth is cheaper than two drifting ones | duplicated lifecycle code |
| 3 | Architecture | Add one private owner transport and keep toolbox introspection-only | Mechanical | Explicit over clever | Stop needs a real owner-plane transport, but toolbox must not become that plane | repurposing toolbox or direct JSON mutation |
| 4 | Architecture | Keep root public session creation host-only in v1 | Mechanical | Boring by default | The parent orchestration model is host-rooted today, and world members are subordinate participants | implicit world-root session creation |
| 5 | Architecture | Keep `resume` in the same parent session and `fork` in a new parent session | Mechanical | Explicit over clever | Operators need different handles and different semantics for those verbs | fuzzy "resume or fork, same end result" behavior |
| 6 | Code Quality | Keep `participant_id` visible but never public-selectable | Mechanical | Minimal diff | It is useful for debugging and lineage, not as an operator target | hiding participant lineage completely or accepting it as input |
| 7 | Code Quality | Use exact UAA selector extension objects for both resume and fork | Mechanical | Layer 1 | The dependency already supports the grammar, so Substrate should not invent a new one | Substrate-only extension shapes |
| 8 | Test Review | Require a new dedicated public-control integration suite | Mechanical | Systems over heroes | This surface deserves its own regression wall and subprocess coverage | burying all new cases inside existing successor tests |
| 9 | Test Review | Make owner-unreachable stop a mandatory failure-path test | Mechanical | Completeness | The stop path is the easiest place to fake success if the transport is wrong | assuming owner registration is obviously fine |
| 10 | Parallelization | Freeze the owner model before parallel implementation lanes | Mechanical | Pragmatic | The file split is clean, but the ownership contract is shared and central | parallel edits before the model is explicit |
