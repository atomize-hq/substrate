<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-session-centric-state-store-autoplan-restore-20260501-142115.md -->

# PLAN-08: Explicit Orchestration Authority for Event Emission

Source file: [08-explicit-orchestration-authority-event-emission.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/08-explicit-orchestration-authority-event-emission.md)  
Branch: `feat/session-centric-state-store`  
Plan type: shell/runtime authority cleanup slice, no UI scope  
Review posture: `/autoplan`-style scope tightening with `/plan-eng-review` structure and rigor  
Status: execution-ready after [PLAN-07](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-07.md), before [09-live-state-authority-and-compatibility-cutover.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/09-live-state-authority-and-compatibility-cutover.md)

## Objective

This slice is not about making more events. It is about making existing shell-owned event
emission honest.

The repo already has the right authority boundary:

- runtime-owned orchestration identity in
  [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
  via `RuntimeOrchestrationContext`
- canonical session-root state in
  [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- explicit trace contract in
  [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
  that says runtime-owned producers must emit a real `orchestration_session_id` or suppress
  the row

What is still wrong is the shell-owned emitter seam. Production code still does live PID-based
recovery on event emission paths in:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

That is the wrong dependency direction. The shell already owns orchestration truth at command
launch time. It should pass that truth into emitters directly, not rediscover it later from
mutable runtime state and a shell PID.

`PLAN-08` fixes that in one bounded slice:

1. introduce one shell-local explicit event-emission context type,
2. thread that context from REPL/runtime-owned authority into host and world execution helpers,
3. require caller-owned `cmd_id` / `run_id` / `span_id` correlation for orchestration-scoped
   shell rows,
4. suppress orchestration-scoped `agent_event` rows when context is absent instead of guessing,
5. keep stdout/stderr and ordinary trace spans working even when orchestration-scoped rows are
   suppressed.

This is the whole game. If a shell-owned emitter can still say "I will figure out who I belong to
later," the live-state and trace contracts built in `PLAN-04`, `PLAN-06`, and `PLAN-07` stay
soft.

## Step 0: Scope Challenge

### 0A. Repo truth and why this slice exists

The SOW is directionally correct, and current repo truth backs it up.

What the code already proves:

1. `RuntimeOrchestrationContext` already exists in
   [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   and already holds authoritative orchestration session state.
2. `translate_wrapper_event(...)` and `build_runtime_message_event(...)` already emit fully
   explicit event rows from manifest plus orchestration-session authority.
3. world-restart alert helpers already fail closed when orchestration context is absent.
4. `docs/TRACE.md` already says runtime-owned producers must emit a real
   `orchestration_session_id` or suppress the row.

What is still broken:

1. `resolve_active_orchestration_session_id()` exists in both
   [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   and
   [dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs),
   and a third copy lives in
   [dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs).
2. those helpers call
   `find_active_orchestration_session_for_pid(std::process::id())` at emit time.
3. `stream_non_pty_via_agent(...)` still falls back to `run_id = parent_cmd_id.unwrap_or("unknown")`.
4. `spawn_host_stream_thread(...)` still emits orchestration-scoped rows with `run_id = "unknown"`.

That means this slice is not greenfield. It is contract repair.

### 0B. Existing code to reuse

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| Runtime-owned orchestration authority | [RuntimeOrchestrationContext in async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse |
| Participant lineage and backend identity | [AgentRuntimeSessionManifest](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) plus event builders in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse |
| Canonical session-root state authority | [AgentRuntimeStateStore](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse, but stop consulting it on emit paths |
| Shell event channel and suppression posture | [publish_agent_event(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs) and current `publish_command_completion(None, ...)` behavior | Reuse |
| Stream chunk terminal passthrough | [emit_stream_chunk(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Reuse stdout/stderr behavior, tighten event gating |
| Existing restart-alert explicit-context model | [build_world_restart_required_alert(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and [emit_world_restarted_alert(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse |
| Existing regression anchors | unit tests in [agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs), [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Extend |

### 0C. Chosen approach

| Approach | Summary | Effort | Risk | Decision |
| --- | --- | --- | --- | --- |
| A. Introduce one explicit shell event context and thread it through current helpers | Model shell-owned event authority directly, keep current execution flow mostly intact | Medium | Medium | **Accepted** |
| B. Keep `Option<&str>` plus add more loose optional params | Smaller diff up front, but keeps authority vague and drift-prone | Small | High | Rejected |
| C. Centralize PID lookup in one helper and keep using it | Looks tidy, still violates the trace contract | Small | Unacceptable | Rejected |
| D. Recover identity from trace/env/history when runtime context is absent | Clever fallback, wrong authority boundary | Small | Unacceptable | Rejected |

The accepted path is the smallest correct slice. It does not redesign the event schema or the
runtime store. It simply stops letting shell emitters lie about how they know who they are.

### 0D. Complexity check

Naive scope here balloons fast. If you let the cleanup spill into runtime-store redesign, CLI
status semantics, or trace-family schema work, this becomes a 10-plus-file fog bank.

The minimal correct production seam is six files:

1. [crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
2. [crates/shell/src/execution/routing/dispatch/exec.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
3. [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
4. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
5. [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
6. [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)

Clarification so this does not turn into scope theater:

- the **core behavior seam** is really the first four files,
- `state_store.rs` is a mechanical cleanup/guard seam because production callers stop using the
  PID helper even if the helper itself remains for tests or diagnostics,
- `docs/TRACE.md` is the contract lock, not new runtime logic,
- `crates/shell/src/execution/invocation/runtime.rs` and compile-fix test call sites are allowed
  one-line fallout if `execute_command(...)` gains an explicit optional context parameter.

Tests will expand the touch set. New production behavior should not.

So the scope reduction is explicit:

- no new services,
- no new registry or cache,
- no live-state authority redesign,
- no schema rewrite,
- no operator-surface selection changes.

That keeps the diff engineered enough. Not under-engineered, not a new framework to solve one
honest plumbing problem.

### 0E. Search and built-in check

`[Layer 1]` wins.

The repo already contains the right primitives:

- authoritative orchestration-session state
- participant lineage fields
- explicit runtime event builders
- fail-closed suppression behavior
- `cmd_id` and `span_id` already threaded in major execution paths

The correct move is to reuse those primitives and unify shell-owned emitters under them.

No external concurrency or framework trick is needed here. In fact, adding one would be spending
an innovation token on the wrong thing.

### 0F. What already exists

- `translate_wrapper_event(...)` already stamps `role`, `backend_id`, `world_id`,
  `world_generation`, and participant lineage onto explicit runtime-owned rows.
- `build_runtime_message_event(...)` already uses manifest plus orchestration-session authority
  without PID lookup.
- restart alerts already suppress rows when orchestration context is absent.
- `publish_command_completion(None, ...)` already suppresses the orchestration-scoped row instead
  of crashing.
- `emit_stream_chunk(...)` already preserves terminal stdout/stderr even when no
  orchestration-scoped row is emitted.

That means the plan is not "invent explicit authority." It is "make the weaker shell helpers obey
the same contract the stronger runtime helpers already obey."

### 0G. NOT in scope

- redesigning [substrate_common::agent_events::AgentEvent](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs)
- changing `substrate agent status`, toolbox selection, or live-session ambiguity behavior
- moving or removing compatibility session-root readers introduced by
  [PLAN-06](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-06.md)
- world restart/invalidation semantics already owned by
  [PLAN-05](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-05.md)
- live-state authority cutover already owned by
  [09-live-state-authority-and-compatibility-cutover.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/09-live-state-authority-and-compatibility-cutover.md)
- member runtime launch and lifecycle work already owned by
  [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md)
- UI work

## Architecture Contract

### Explicit shell event authority type

Add one shell-local context model in
[crates/shell/src/execution/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
or an immediately adjacent shell-local module.

The exact Rust names can stay flexible. The contract cannot.

Minimum required shape:

```rust
struct ShellEventEmissionContext {
    orchestration_session_id: String,
    agent_id: String,
    role: Option<String>,
    backend_id: Option<String>,
    participant_id: Option<String>,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    world_id: Option<String>,
    world_generation: Option<u64>,
}

struct ShellCommandEventContext {
    emission: ShellEventEmissionContext,
    cmd_id: String,
    run_id: Option<String>,
    span_id: Option<String>,
}
```

Rules:

1. shell-owned orchestration-scoped rows are built from this context, not from loose optional
   strings.
2. `run_id` stays optional at the type level so suppression can happen cleanly when the caller
   does not have one.
3. once this context exists, production paths must not emit orchestration-scoped shell rows with
   synthetic correlation like `"unknown"`.

### Authority flow diagram

```text
runtime-owned authority
    │
    ├── RuntimeOrchestrationContext
    ├── active manifest snapshot
    └── command-owned correlation (cmd_id, run_id, span_id)
            │
            ▼
    ShellEventEmissionContext / ShellCommandEventContext
            │
            ├── publish_command_completion(...)
            ├── execute_external(...) / spawn_host_stream_thread(...)
            └── stream_non_pty_via_agent(...) / emit_stream_chunk(...)
                    │
                    ├── context complete ----------> emit orchestration-scoped AgentEvent
                    └── context absent/incomplete -> suppress AgentEvent, keep stdout/stderr + spans
```

### No PID-based recovery on production emit paths

After this slice, the following production pattern is forbidden in shell event-emission control
flow:

```rust
AgentRuntimeStateStore::new()?
    .find_active_orchestration_session_for_pid(std::process::id())
```

That ban applies to:

- REPL command-completion call sites
- host external command stream emission
- world non-PTY stream emission
- any future restart-alert or shell wrapper around those helpers

The helper may remain in
[state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
for tests or diagnostics. Production emitters must stop consulting it.

### Suppress, do not guess

If a caller cannot provide explicit orchestration authority:

1. terminal stdout/stderr must still be written,
2. trace span emission must still proceed,
3. the orchestration-scoped `agent_event` row must be suppressed,
4. the shell must not scan runtime state, shell PID ownership, env vars, trace history, or any
   other ambient source to reconstruct identity.

This matches the posture already used by:

- `publish_command_completion(None, ...)`
- `build_world_restart_required_alert(None, ...)`
- `emit_stream_chunk(..., None, ...)`

### Caller-owned correlation contract

For shell-owned orchestration-scoped rows:

- command completion requires a real `cmd_id`
- stream chunk emission requires a real `run_id`
- `span_id` is attached when known and must not be synthesized

Hard rule:

- if the caller cannot supply a real `run_id` for an orchestration-scoped stream row, suppress the
  row instead of emitting `"unknown"`.

That matters because `docs/TRACE.md` already treats `run_id` as a required join key for structured
agent events.

### Per-path authority and correlation matrix

This is the part that has to be explicit or the implementation will drift.

| Path | Authority source | `run_id` source | `span_id` source | If authority/correlation is missing |
| --- | --- | --- | --- | --- |
| REPL command completion | `RuntimeOrchestrationContext` plus active manifest/world binding snapshot | caller-owned `cmd_id` for that command-completion row | attach only if the caller already has one | suppress the orchestration-scoped row |
| Host external command stream | caller-owned explicit shell event context passed into `execute_command(...)` / `execute_external(...)` | stable caller-owned command/run id captured before stream threads start, never `"unknown"` | attach if command-span or downstream span is known | suppress the orchestration-scoped row, keep stdout/stderr |
| World non-PTY deny before agent start | caller-owned explicit shell event context from launch boundary | stable caller-owned command/run id from launch boundary | parent command span when known | suppress the orchestration-scoped row, still print deny text |
| World non-PTY started stream frames | same explicit caller-owned context captured before frame processing | same stable caller-owned run id as launch, do not swap it mid-stream | attach `ExecuteStreamFrame::Start { span_id }` when it arrives | suppress the orchestration-scoped row until both context and stable run correlation are real |
| Non-REPL host invocation callers | none, by design | none | none | always pass `None`; this slice does not fabricate orchestration context for wrap/pipe/runtime helpers |

Two hard clarifications:

1. this slice does **not** re-derive orchestration context for
   [crates/shell/src/execution/invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs);
   those call sites remain non-orchestrator callers and pass `None`,
2. once the launch boundary chooses the command/run correlation for a shell-owned row, downstream
   frame processing may attach more metadata like `span_id`, but it may not silently replace the
   chosen `run_id` with a later convenience value.

### Existing explicit runtime translation remains the model

The stronger paths are already correct in principle:

- `translate_wrapper_event(...)`
- `build_runtime_message_event(...)`
- world-restart alert construction from `startup_context`

This slice aligns weaker host/world shell emitters to that model. It does not invent a second
authority path.

## Concrete File Touch Plan

### 1. `crates/shell/src/execution/agent_events.rs`

Primary seams:

- [publish_agent_event(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)
- [publish_command_completion(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_events.rs)

Required behavior:

- define the explicit shell event-emission context type,
- update `publish_command_completion(...)` to accept explicit context rather than
  `Option<&str>`,
- stamp `role`, `backend_id`, lineage, `world_id`, and `world_generation` when the caller
  supplies them,
- preserve suppression semantics when context is absent.

Must not do:

- keep the old helper shape as the preferred production path,
- let command completion remain only "session id plus message" when richer authority is already
  available,
- move the context type into `substrate-common`; this is shell-local plumbing, not a wire-schema
  expansion.

### 2. `crates/shell/src/execution/routing/dispatch/exec.rs`

Primary seams:

- [execute_command(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
- [execute_external(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)
- [spawn_host_stream_thread(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/exec.rs)

Required behavior:

- remove `resolve_active_orchestration_session_id()`,
- thread optional explicit event context into `execute_command(...)` and `execute_external(...)`,
- capture caller-owned command/run correlation before any background stream thread starts,
- thread `None` explicitly from non-orchestrator callers such as
  [crates/shell/src/execution/invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs),
- stop emitting orchestration-scoped host stream rows with `run_id = "unknown"`,
- keep host output behavior unchanged when no orchestration context exists.

Must not do:

- reintroduce the PID lookup behind a new helper name,
- require host-only mode to fabricate orchestration context,
- drop stdout/stderr just because the orchestration-scoped row is suppressed.

### 3. `crates/shell/src/execution/routing/dispatch/world_ops.rs`

Primary seams:

- [stream_non_pty_via_agent(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [process_agent_stream_body(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [emit_stream_chunk(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)

Required behavior:

- remove `resolve_active_orchestration_session_id()`,
- accept explicit event context from the caller,
- emit stream rows only when both orchestration authority and real run correlation exist,
- continue to mirror stdout/stderr regardless,
- preserve current world-guard deny behavior while suppressing orchestration-scoped rows when
  required context is missing.

Must not do:

- treat `active_span_id.unwrap_or("unknown")` as acceptable once explicit context plumbing exists,
- let background frame processing infer the orchestration session after the fact.

### 4. `crates/shell/src/repl/async_repl.rs`

Primary seams:

- REPL command-completion call sites
- [RuntimeOrchestrationContext](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [exec_host_line(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [exec_world_line(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [exec_world_pty(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required behavior:

- stop calling `resolve_active_orchestration_session_id()` for command completion,
- derive shell event context from `RuntimeOrchestrationContext` and the live manifest snapshot
  when the runtime exists,
- pass that context into host/world execution helpers,
- keep the command/run correlation choice stable at the launch boundary instead of letting world
  stream frame handlers invent or swap it later,
- keep host-only `--no-world` behavior suppression-only when there is no orchestrator runtime,
- preserve existing restart-alert explicit-context behavior.

Must not do:

- add new runtime-global state just for emitter lookup,
- turn `async_repl.rs` into a second event builder with duplicated field assignment logic.

### 5. `crates/shell/src/execution/agent_runtime/state_store.rs`

Required behavior:

- leave `find_active_orchestration_session_for_pid(...)` available only if tests or diagnostics
  still need it,
- ensure production event-emission paths no longer call it.

This is intentionally a small file-touch item. The goal is not more state-store behavior. The goal
is to narrow who depends on the helper.

### 6. `docs/TRACE.md` and test anchors

Required behavior:

- keep the documented contract aligned with the code,
- make shell-owned command completion and stream emitters explicit examples of the
  "real id or suppress" rule,
- add bounded regression assertions so future diffs cannot quietly reintroduce PID lookup or
  synthetic fallback correlation.

## Execution Plan

### Ordered implementation sequence

Implement in this order. Each phase creates one invariant the next phase can safely consume.

1. **Freeze the shell event context contract**
   - add the explicit shell event-emission context type,
   - update `publish_command_completion(...)`,
   - keep suppression semantics intact.

2. **Thread authority through the REPL boundary**
   - derive optional explicit context from `RuntimeOrchestrationContext` plus live manifest
     snapshots,
   - pass it into host and world command entrypoints,
   - remove REPL-local PID lookup.

3. **Freeze host command stream semantics**
   - update `execute_command(...)` and `execute_external(...)`,
   - remove host-path PID lookup,
   - require caller-owned correlation before host stream rows emit.

4. **Freeze world non-PTY stream semantics**
   - update `stream_non_pty_via_agent(...)`, `process_agent_stream_body(...)`, and
     `emit_stream_chunk(...)`,
   - remove world-path PID lookup,
   - suppress orchestration-scoped rows when `run_id` is not authoritative.

5. **Widen regression coverage and doc wording**
   - lock command-completion suppression and explicit context behavior,
   - lock stream suppression and no-`unknown` fallback behavior,
   - lock the repo-wide no-PID-lookup production rule,
   - clarify `docs/TRACE.md`.

### Phase-by-phase acceptance gates

| Phase | Acceptance gate |
| --- | --- |
| 1. Context contract | shell-owned event helpers accept explicit authority rather than loose optional strings |
| 2. REPL boundary | command-completion paths no longer resolve orchestration identity from shell PID |
| 3. Host streams | host stdout/stderr still work, but orchestration-scoped rows emit only with explicit authority plus real correlation |
| 4. World streams | non-PTY world stream rows emit only from caller-owned authority, never from ambient lookup or `"unknown"` correlation |
| 5. Regression closeout | tests and docs prove "real id or suppress" across shell-owned emitters |

## Architecture Review

### Locked architecture decisions

1. **Keep event authority at the caller boundary.**
   - The caller that launches the command already knows the runtime/session owner. The emitter
     should consume that authority, not rediscover it.

2. **Keep shell event context shell-local.**
   - This is not a wire-schema change. It is an internal authority-plumbing type.

3. **Keep suppression fail closed.**
   - Missing context suppresses the orchestration-scoped row. It does not block terminal output
     or spans, and it does not trigger a fallback scan.

4. **Keep correlation explicit.**
   - `run_id = "unknown"` is no longer acceptable for orchestration-scoped shell stream rows once
     explicit context exists.

5. **Keep runtime-owned event builders as the gold standard.**
   - `translate_wrapper_event(...)` and `build_runtime_message_event(...)` already show the right
     posture. Follow them.

### Authority and correlation diagram

```text
host/world command launch
    │
    ├── cmd_id always known
    ├── run_id known for orchestrator-owned runtime flows
    ├── span_id known after stream start or command-span creation
    └── orchestration authority known only if runtime/manifest says so
            │
            ▼
    if orchestration authority && real run_id
        emit orchestration-scoped AgentEvent
    else
        write stdout/stderr only
        keep trace spans only
        emit no orchestration-scoped AgentEvent
```

### Architecture acceptance gates

1. **Authority gate**
   - no production event-emission path in `async_repl`, `dispatch/exec`, or `dispatch/world_ops`
     resolves orchestration identity from `shell_owner_pid`.

2. **Correlation gate**
   - orchestration-scoped stream rows never emit with synthetic `run_id = "unknown"`.

3. **Suppression gate**
   - missing context suppresses only the orchestration-scoped row, not terminal output or command
     tracing.

## Code Quality Review

### Implementation guardrails

1. one explicit context type for shell-owned event authority, not more raw `Option<&str>` sprawl,
2. one place that stamps lineage and backend identity for shell-owned rows, not field drift across
   call sites,
3. one suppression posture across command completion and stream emission,
4. no new ambient lookup helper,
5. no new state-store contract or trace schema just to support this slice.

### Minimal-diff rules

- keep the production diff inside the six files already named,
- prefer extending current tests over inventing a new harness,
- if a helper extraction is needed, keep it local to the shell crate,
- do not broaden this slice into `PLAN-09` live-state authority work.

## Error & Rescue Registry

| Failure point | What goes wrong | Required rescue |
| --- | --- | --- |
| caller lacks orchestration authority | shell would be tempted to guess via PID lookup | suppress orchestration-scoped row, keep stdout/stderr and spans |
| caller lacks real `run_id` for stream row | event is emitted with fake correlation and poisons trace joins | suppress orchestration-scoped row instead of using `"unknown"` |
| REPL runtime snapshot is stale or absent | host/world completion path cannot build rich authority payload | emit no orchestration-scoped completion row |
| background stream thread outlives command setup assumptions | late chunks try to reconstruct session ownership ambiently | capture immutable explicit context before thread/frame loop starts |
| a future diff reintroduces PID lookup | trace contract silently regresses | add bounded repo/test guard proving production emitters no longer call the helper |
| docs drift from implementation | operators trust a contract the code no longer satisfies | tighten `docs/TRACE.md` in the same slice |

## Test Review

This slice already has some decent regression anchors. Good. The missing part is that the current
tests still prove "optional session id works" instead of "explicit authority is required."

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_events.rs
    │
    ├── [★★★ TESTED] command completion emits when context/session id is present
    ├── [★★★ TESTED] command completion suppresses when context/session id is absent
    ├── [GAP]         explicit shell event context stamps role/backend/lineage correctly
    └── [GAP]         success + failure completion both preserve suppression-only posture when context is absent

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [★★★ TESTED] restart alerts already require explicit orchestration context
    ├── [GAP]         host escape completion no longer reads PID-owned runtime state
    ├── [GAP]         world PTY completion no longer reads PID-owned runtime state
    ├── [GAP]         world line completion no longer reads PID-owned runtime state
    └── [GAP]         host-only completion suppresses event rows without affecting terminal behavior

[+] crates/shell/src/execution/routing/dispatch/exec.rs
    │
    ├── [GAP]         execute_command accepts optional explicit event context
    ├── [GAP]         host stream threads emit orchestration-scoped rows only with real run_id
    ├── [GAP]         no host stream row emits with run_id="unknown"
    └── [GAP]         stdout/stderr still mirror normally when context is absent

[+] crates/shell/src/execution/routing/dispatch/world_ops.rs
    │
    ├── [★★★ TESTED] emit_stream_chunk emits with orchestration context
    ├── [★★★ TESTED] emit_stream_chunk suppresses without orchestration context
    ├── [GAP]         stream_non_pty_via_agent no longer resolves orchestration session via PID lookup
    ├── [GAP]         world stream rows suppress when run_id would otherwise be synthetic
    └── [GAP]         world-guard deny path still prints error text while suppressing orchestration row if context is incomplete

[+] repo-wide production guard
    │
    └── [GAP]         only tests/diagnostics may still reference find_active_orchestration_session_for_pid(...)

─────────────────────────────────
COVERAGE: 5/16 paths tested (31%)
QUALITY:  ★★★: 5  ★★: 0  ★: 0
GAPS: 11 paths need regression tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] REPL host escape inside active orchestrator runtime
    │
    ├── [GAP] [→INTEGRATION] command completes and emits a completion row from explicit runtime context
    └── [GAP]         completion does not consult PID-owned runtime state

[+] REPL host-only command with no active orchestrator runtime
    │
    ├── [GAP]         stdout/stderr and trace span still appear
    └── [GAP]         no orchestration-scoped completion row is emitted

[+] World non-PTY command stream
    │
    ├── [GAP] [→INTEGRATION] stream chunks emit with caller-owned orchestration authority and real run correlation
    ├── [GAP]         missing start/run correlation suppresses orchestration row only
    └── [★★★ TESTED] chunk helper itself preserves terminal output

[+] Host external command stream
    │
    ├── [GAP]         background stdout/stderr threads use captured explicit context, not late ambient lookup
    └── [GAP]         no stream row emits with synthetic run_id="unknown"

[+] Restart alert behavior
    │
    └── [★★★ TESTED] explicit-context-only posture already holds and must remain unchanged

[+] Repo contract guard
    │
    └── [GAP]         production code no longer references find_active_orchestration_session_for_pid(...)

─────────────────────────────────
COVERAGE: 2/11 flows tested (18%)
GAPS: 9 flows need tests (2 deserve integration-style crate coverage rather than unit-only assertions)
─────────────────────────────────
```

### Required test additions by file

#### `crates/shell/src/execution/agent_events.rs`

Add regression coverage for:

- command completion built from explicit shell event context rather than a bare session id,
- failure and success completion rows preserving `cmd_id`,
- suppression when context is absent even if success events are enabled,
- lineage and backend identity stamping when context includes them.

#### `crates/shell/src/repl/async_repl.rs`

Add coverage for:

- REPL host escape completion no longer consulting PID-owned runtime state,
- world PTY completion no longer consulting PID-owned runtime state,
- host-only completion suppressing the orchestration-scoped row without affecting terminal
  behavior,
- restart-alert explicit-context behavior remaining unchanged.

#### `crates/shell/src/execution/routing/dispatch/exec.rs`

Add regression coverage for:

- explicit event context threaded into `execute_command(...)` / `execute_external(...)`,
- non-REPL callers that pass `None` continuing to execute normally while suppressing
  orchestration-scoped rows,
- host stream rows suppressing when real run correlation is absent,
- host stream rows keeping the launch-owned correlation stable instead of inventing a later one,
- no host stream row emitting with `run_id = "unknown"`,
- stdout/stderr still being mirrored when the orchestration-scoped row is suppressed.

#### `crates/shell/src/execution/routing/dispatch/world_ops.rs`

Add regression coverage for:

- `stream_non_pty_via_agent(...)` no longer consulting PID lookup,
- `process_agent_stream_body(...)` suppressing orchestration rows if a real run id is not
  authoritative yet,
- `ExecuteStreamFrame::Start { span_id }` enriching later chunk rows with `span_id` without
  replacing the launch-owned run correlation,
- deny-path stderr printing surviving row suppression,
- repo-contract tests for no synthetic correlation.

#### `docs/TRACE.md` or shell-level bounded repo guard

Add one bounded guard that proves:

- production shell emitters no longer reference
  `find_active_orchestration_session_for_pid(...)`,
- any remaining references are tests or diagnostics only.

### Test commands

Run at minimum:

```bash
cargo test -p shell publish_command_completion -- --nocapture
cargo test -p shell emit_stream_chunk -- --nocapture
cargo test -p shell build_world_restart_required_alert_only_builds_with_orchestration_context -- --nocapture
cargo test -p shell emit_world_restarted_alert_only_emits_with_orchestration_context -- --nocapture
cargo test -p shell start_host_orchestrator_runtime_persists_participant_snapshots_across_lifecycle_states -- --nocapture
cargo test -p shell -- --nocapture
```

Then run:

```bash
cargo test -p world-agent -- --nocapture
cargo test -p world-api -- --nocapture
cargo test -p agent-api-types -- --nocapture
```

Only treat that second block as required if the implementation actually changes shared request,
stream-frame, or cross-crate event-shape assumptions. If this stays inside shell-local authority
plumbing, `cargo test -p shell -- --nocapture` is the non-negotiable gate and the cross-crate
runs are blast-radius confidence.

### QA artifact

Generate the standard eng-review handoff artifact at:

```text
~/.gstack/projects/$SLUG/{user}-{branch}-eng-review-test-plan-{datetime}.md
```

Required contents for this slice:

- affected shell execution paths: REPL completion, host external stream, world non-PTY stream,
- the suppression-only scenarios that must preserve stdout/stderr and trace spans,
- the negative assertions that no orchestration-scoped row is emitted when explicit context is
  absent or correlation would be synthetic.

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| REPL completion authority plumbing | completion row is attached to the wrong orchestration session after a stale PID-based lookup | no | partial today | no | yes until PID lookup is removed and tested |
| host stream emission | stdout/stderr rows get emitted with `run_id="unknown"` and poison trace joins | no | no | no | yes until suppression-only behavior is tested |
| world non-PTY stream emission | late stream chunks reconstruct orchestration ownership ambiently after command launch | no | partial today | no | yes until explicit context is captured and tested |
| host-only no-runtime command | shell suppresses event row but accidentally drops terminal output or trace span | no | yes | partial today | yes until regression-covered |
| repo-wide authority contract | a future diff reintroduces `find_active_orchestration_session_for_pid(...)` into production emitters | no | no | no | yes until bounded guard lands |
| restart alert helpers | explicit-context-only behavior regresses during refactor | yes | yes | yes | no, but keep the existing tests green |

Critical gap rule:

If production shell emitters can still reconstruct orchestration identity from shell PID, or if
they can still emit orchestration-scoped stream rows with synthetic run correlation, this slice is
not done.

## Performance Review

This is a correctness and authority slice, not a throughput slice.

Still, four performance rules matter:

1. capture immutable event context once per command, not once per stream chunk,
2. do not add runtime-store scans to hot chunk loops,
3. cloning a small explicit context object is cheap and dramatically cheaper than doing ambient
   session scans,
4. do not add a cache or registry to "speed up" the wrong architecture.

The performance footgun here would be spending more code to avoid cloning a few strings while
keeping the wrong authority path alive. That would be classic software: a 200-line optimization to
protect a bug.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Freeze the context contract and launch-boundary mapping | `crates/shell/src/execution/`, `crates/shell/src/repl/` | — |
| B. Re-plumb host and world stream emitters onto the frozen contract | `crates/shell/src/execution/routing/dispatch/` | A |
| C. Land regression guards, test-plan artifact, and trace wording | `crates/shell/src/`, `docs/`, `~/.gstack/projects/` | A, B |

### Parallel lanes

- Lane A: step A
- Lane B: step B, only after Lane A lands
- Lane C: step C, can split from Lane B only after signatures stop moving

### Execution order

1. land Lane A first because it freezes both the context type and the per-path
   authority/correlation mapping,
2. land Lane B next because `exec.rs` and `world_ops.rs` both depend on that exact contract,
3. once Lane B settles, Lane C can be handled as a short follow-up or parallel cleanup across
   tests/docs only.

### Conflict flags

- `agent_events.rs`, `async_repl.rs`, `exec.rs`, and `world_ops.rs` are one coupled seam.
- `execute_command(...)` signature fallout into
  [invocation/runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/invocation/runtime.rs)
  is mechanically small, but it is still contract fallout and belongs with Lane A or B, not a
  free-floating branch.
- Production-code parallelization before the context contract freezes is asking for merge-conflict
  soup.

### Parallelization verdict

This slice is mostly sequential.

- There are **three lanes** on paper.
- There are **zero safe parallel production-code lanes** before Lane A finishes.
- The only realistic post-freeze split is: one branch finishes execution plumbing, one branch
  finishes regression/doc guardrails after the helper signatures stop moving.

## Deferred Work

There is no `TODOS.md` in the repo root, so explicit deferrals stay here:

1. live-state authority cutover and compatibility-write removal
   - owned by
     [09-live-state-authority-and-compatibility-cutover.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/09-live-state-authority-and-compatibility-cutover.md)
2. production member runtime launch and world-scoped lifecycle wiring
   - owned by
     [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md)
3. any broader agent-event schema redesign
   - intentionally out of scope for this authority-plumbing slice
4. nicer operator-facing diagnostics for suppressed orchestration-scoped rows
   - useful later, not required to make authority honest now

## Definition of Done

This slice is done only when all of the following are true:

1. no production event-emission path in `async_repl`, `dispatch/exec`, or `dispatch/world_ops`
   resolves orchestration identity from `shell_owner_pid`,
2. shell-owned command-completion rows consume explicit caller-provided authority,
3. shell-owned stream-chunk rows consume explicit caller-provided authority and real run
   correlation,
4. missing orchestration context suppresses the orchestration-scoped `agent_event` row without
   suppressing stdout/stderr or trace spans,
5. orchestration-scoped shell stream rows no longer emit with synthetic fallback correlation like
   `"unknown"`,
6. runtime-owned event translation paths remain explicit and unchanged in principle,
7. docs and tests reflect the "real id or suppress" contract,
8. `find_active_orchestration_session_for_pid(...)`, if retained, is no longer part of production
   emission control flow.

## Completion Summary

- Step 0: scope reduced to the six-file production seam, no schema or live-state redesign
- Architecture Review: 3 issues found, all authority/correlation contract gaps rather than a
  need for new infrastructure
- Code Quality Review: 2 issues found, weak helper signatures and duplicated ambient lookup
  posture
- Test Review: diagrams produced, 11 concrete regression gaps identified
- Performance Review: 0 issues found
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0, repo has no `TODOS.md`, deferrals captured here
- Failure modes: 5 critical gaps flagged
- Outside voice: skipped, `claude` CLI is installed but not authenticated on 2026-05-01 and no
  separate subagent review was run
- Parallelization: 3 lanes, 0 safe production-code parallel lanes before context freeze
- Lake Score: complete option chosen for every in-slice decision

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Treat this as authority-plumbing repair, not a trace-schema redesign | Mechanical | Pragmatic | The repo already has the right event schema and runtime authority sources | Redesigning `AgentEvent` |
| 2 | Authority | Introduce one shell-local explicit event context type | Mechanical | Explicit over clever | One context object is easier to audit than many raw optional strings | Extending the current `Option<&str>` pattern |
| 3 | Safety | Suppress orchestration-scoped rows when context or run correlation is absent | Mechanical | Completeness | Fail-closed behavior preserves correctness without breaking terminal output | PID lookup fallback or `run_id=\"unknown\"` |
| 4 | Reuse | Keep runtime-owned event builders as the model | Mechanical | DRY | The repo already has correct explicit event builders in `async_repl.rs` | Building a second shell-specific field-stamping path |
| 5 | Scope reduction | Keep production edits inside six files plus bounded docs/tests | Mechanical | Minimal diff | This slice is already sharp enough and should not swallow `PLAN-09` | Folding in live-state cutover or status-surface work |
| 6 | Tests | Add a bounded guard proving production emitters no longer call the PID helper | Mechanical | Systems over heroes | Without a guard, this regression will come back in a future refactor | Relying on reviewer memory |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 0 | SKIPPED | This is a bounded shell/runtime authority cleanup slice, not a product-scope decision |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate Codex outside-voice review run |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Locked the slice to explicit shell event authority, removed ambient PID lookup from the target design, and identified the remaining regression gaps around no-fallback correlation, suppression-only behavior, and bounded repo guards |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 blocking architecture decisions remain inside slice `08`. The remaining work is
authority plumbing plus regression coverage.

**VERDICT:** ENG CLEARED. `PLAN-08` is ready to execute as the shell event-authority cleanup slice
between the session-centric state-store work and the later live-state authority cutover.
