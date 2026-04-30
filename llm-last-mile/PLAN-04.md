<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-shared-world-ownership-contract-autoplan-restore-20260429-213754.md -->
# PLAN-04: Thread World Binding Into Runtime State

Source file: [04-thread-world-binding-into-runtime-state.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md)  
Branch: `feat/shared-world-ownership-contract`  
Plan type: backend-only, no UI scope  
Review posture: `/autoplan` consolidation pass with `/plan-eng-review` structure and rigor  
Status: execution-ready

## What This Plan Does

`PLAN-03` made shared-world ownership and generation backend-authoritative.

The original slice-04 SOW had the right intent and the wrong host-side storage target. The current participant model still rejects any host-scoped runtime record with `world_id/world_generation` in [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:355). So this plan does **not** force world binding onto the host participant manifest.

Instead, `PLAN-04` does four precise jobs:

1. persist a pending `OrchestrationSessionRecord` before the first shared-world attach/create request,
2. use that persisted parent session record as the session-scoped bridge authority for active `world_id/world_generation`,
3. require every startup/restart/fail-closed drift path to persist parent binding before publishing alerts or host runtime events,
4. expose one live operator proof point through `substrate agent toolbox status --json` while preserving the existing selected-orchestrator `agent status --json` contract.

That is the smallest correct bridge between:

- backend-authoritative shared-world proof from `PLAN-03`,
- member invalidation semantics coming in `PLAN-05`,
- and the future grouped `agent-sessions/<orchestration_session_id>.json` layout planned in [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md:3).

## Scope Challenge

### 0A. Premise Challenge

These are the premises this plan accepts after CEO and engineering review.

1. **The host participant manifest is not the right authority surface for slice `04`.**
   - Accepted.
   - The code rejects host-scoped participant records with world fields. Fighting that invariant here would create a bigger, riskier contract change than the slice needs.

2. **The first shared-world attach/create request must already be owner-bound.**
   - Accepted.
   - If the shell creates world state before the orchestration session id exists durably, slice `04` never gets authoritative startup proof.

3. **A persisted pending parent session record is necessary before the first world start, not just an in-memory preallocated id.**
   - Accepted.
   - Pre-live drift, restart, and bootstrap failure paths all need a durable session-scoped authority anchor before the host runtime becomes active.

4. **The parent session record is the narrowest bridge authority that matches current code.**
   - Accepted.
   - It already stores `world_id/world_generation` and can be lifted into slice `06` later.

5. **A live proof surface should exist, but `agent doctor` is the wrong contract for it.**
   - Accepted.
   - `toolbox status` is already a live-session-oriented surface. `doctor` is still primarily a static readiness / fail-closed preflight contract.

### 0B. Existing Code Leverage

| Sub-problem | Existing code | Reuse or replace |
|---|---|---|
| Session-scoped runtime record already stores world fields | [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:20) | Reuse, extend with explicit mutators |
| Parent-session persistence already exists | [AgentRuntimeStateStore::persist_orchestration_session(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:271) | Reuse as the binding authority write path |
| Shared-world echo validation already exists | [validate_shared_world_echo(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:314) | Reuse, but make first-start requests actually send owner proof |
| Startup currently opens the world before host runtime bootstrap | [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:433) | Keep the overall shape, change the startup context and persistence ordering |
| Pre-live drift/restart logic already exists | [handle_detected_world_drift(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2586) and [restart_world_session(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2707) | Reuse, thread explicit startup context through them |
| Post-start active session lookup already exists | [resolve_active_orchestration_session_id()](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1255) | Reuse after runtime activation, do not rely on it for first start or pre-live drift |
| Live orchestrator resolution already returns parent + child | [resolve_live_orchestrator_session(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:410) | Reuse for toolbox live authority |
| Status contract already suppresses host world fields | [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:1850) | Reuse exactly |
| Runtime event schema already supports top-level world fields | [crates/common/src/agent_events.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/src/agent_events.rs:66) | Reuse unchanged |

### 0C. Dream State Mapping

```text
CURRENT
-------
startup world attach can happen before durable orchestration identity exists
        │
        ├── REPL memory knows current world binding
        ├── alerts may publish that binding
        └── persisted live runtime state can still lag or be absent

THIS PLAN
---------
persisted pending OrchestrationSessionRecord
        │
        ▼
first attach/create request includes shared-world owner proof
        │
        ▼
parent session record persists active world binding
        │
        ├── restart/fail-closed alerts read persisted parent truth
        ├── host runtime events read persisted parent truth
        ├── toolbox status can prove current binding live
        └── selected status row stays host-scoped

12-MONTH IDEAL
--------------
grouped agent-sessions registry
        │
        ├── orchestrator and member live records share one lineage model
        ├── generation invalidation is explicit and queryable
        ├── trace remains historical only
        └── slice 04 is an obvious bridge, not dead-end storage debt
```

### 0C-bis. Implementation Alternatives

| Approach | What it does | Effort | Risk | Recommendation |
|---|---|---:|---:|---|
| A. Keep reconstructing from trace or REPL memory | No durable authority fix | S | High | Reject |
| B. Force world binding onto the host participant manifest | Uses the fields the source SOW pointed at | M | Critical | Reject, violates current host invariant |
| C. Persist binding on the parent session record, create that record before first attach, preserve host participant invariants | Smallest correct bridge | M | Low | Recommended |
| D. Collapse `PLAN-04`, `PLAN-05`, and slice `06` into a thin session-registry vertical slice | Better end-state alignment | L | Medium | Strategic alternative, defer for this packet |

Recommendation: **C**.

Approach D is a real strategic alternative. It is also a user-direction change to the packet, not a silent implementation refinement. This plan stays inside the current packet order.

### 0D. Mode-Specific Analysis

Mode: `SELECTIVE EXPANSION`

Scope held:

- no grouped registry layout yet
- no member invalidation yet
- no selected-status schema change
- no host participant world-field exception

Expansion accepted inside the blast radius:

- persist the pending parent session record before first attach/create
- add one live proof field to `toolbox status --json`

### 0E. Temporal Interrogation

**Hour 1:** the first world attach/create request is owner-bound and a durable parent runtime record already exists on disk.

**Hour 6:** after one auto-restart and one fail-closed drift event, the parent session record, emitted alerts, and toolbox status all agree on the current binding.

**Month 3:** `PLAN-05` invalidates prior-generation member records by consuming the already-persisted active binding instead of inventing a separate startup-recovery rule.

**Year 1:** slice `06` lifts the same session-scoped truth into `agent-sessions/<orchestration_session_id>.json` without undoing host participant rules.

### 0F. Mode Selection

`SELECTIVE EXPANSION` is the right mode because the source SOW needed correction, not abandonment:

- startup identity has to be durable before first attach/create
- the authority surface must be session-scoped, not host-participant-scoped
- the live proof surface should be toolbox status, not doctor or selected status

## What Already Exists

- A real backend-authoritative shared-world proof shape from `PLAN-03`
- A parent session record that already stores `world_id/world_generation`
- A live orchestrator resolution path that already joins parent and child runtime state
- A runtime event schema that already carries top-level world fields
- A status contract that already forbids leaking host world fields on the selected orchestrator row

## Architecture Contract

### No-ambiguity rules

1. Host-scoped participant manifests remain world-empty in this slice.
2. The authoritative live binding record for the host orchestrator path is `OrchestrationSessionRecord.world_id/world_generation`.
3. A persisted `Allocating` parent session record must exist before the first world attach/create request.
4. Pre-live drift/restart paths must thread that startup context explicitly. They must not rely on active-session lookup.
5. Binding-only persistence writes only the parent session record. It does not depend on rewriting the host participant or lease sidecar.
6. Alerts and host runtime events that claim world binding must read from a successfully persisted parent session snapshot.
7. `substrate agent status --json` keeps the current selected-orchestrator contract: host scope, no world fields.
8. `substrate agent toolbox status --json` is the live proof surface for current binding, via an optional best-effort `active_world_binding` field.
9. Public proof surfaces omit `orchestration_session_id`. That identifier remains an internal same-user control-plane correlation key in this slice.
10. This slice does not define member invalidation or replacement lineage. `PLAN-05` consumes the persisted active binding later.

### Target state model

Keep `OrchestrationSessionRecord.world_id` and `world_generation` as the session-scoped active binding fields.

Add explicit mutators in [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs):

- `set_world_binding(world_id: impl Into<String>, world_generation: u64)`
- `clear_world_binding()`

Clear rules:

- clear on normal world teardown after the parent session transitions terminal
- clear on bootstrap failure after world attach once close succeeds
- do **not** clear during `world_restart_required` fail-closed handling while the current binding is still authoritative

Do **not** add host-manifest world-binding mutators in this slice.

### Startup identity contract

`run_async_repl(...)` changes from:

```text
start_world_session()
start_host_orchestrator_runtime()
```

to:

```text
prepare_orchestration_session_context()
    ├── generate orchestration_session_id
    ├── persist parent session state=Allocating
    └── carry startup context for pre-live drift/restart paths

start_world_session(startup_context)
start_host_orchestrator_runtime(startup_context, initial_world_binding)
```

That startup context must carry at least:

- `orchestration_session_id`
- `shell_trace_session_id`
- `workspace_root`
- persisted parent-record path or resolved handle

### Ordered synchronization contract

Add one helper in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs):

```text
persist_active_world_binding(store, orchestration_session, world_session)
    1. copy world_id/world_generation from WorldSession to parent session record
    2. persist the parent session record
    3. return the persisted parent snapshot
```

Important:

- this helper is the binding authority barrier
- host participant and lease writes remain separate lifecycle writes, not prerequisites for binding truth
- if later lifecycle transitions also update the participant manifest, those are follow-on writes after the binding barrier, not part of it

### Bootstrap failure contract

If world startup succeeds but host runtime bootstrap fails later:

1. close the world session explicitly,
2. mark the pending parent session record terminal with the failure reason,
3. clear binding only after the close succeeds,
4. exit fail-closed.

This prevents orphaned worlds owned by a session id that never acquired a durable live runtime.

### Event and alert contract

Required changes:

- `build_runtime_message_event(...)` accepts an optional parent-session binding snapshot for host/orchestrator events
- `translate_wrapper_event(...)` does the same
- startup `registered` / `task_start` runtime events are emitted only after the initial binding barrier succeeds when the world session exists
- `emit_world_restarted_alert(...)` and `build_world_restart_required_alert(...)` run only after `persist_active_world_binding(...)` succeeds

No `AgentEvent` schema expansion is required.

### Toolbox and status contract

Keep status conservative:

- selected orchestrator row stays host-scoped
- no `world_id/world_generation` on that row

Add a proof field in `substrate agent toolbox status --json`:

```json
"active_world_binding": {
  "world_id": "wld_active_0002",
  "world_generation": 7
}
```

Rules:

- field appears only when a live parent + child orchestrator resolution succeeds
- field omission is non-fatal
- toolbox status retains its normal eligibility and transport semantics

## Architecture Diagrams

### Startup flow

```text
run_async_repl()
    │
    ├── prepare_orchestration_session_context()
    │      ├── orchestration_session_id allocated
    │      └── parent session persisted as Allocating
    │
    ├── start_world_session(startup_context)
    │      ├── first AttachOrCreate request carries owner proof
    │      └── startup drift paths reuse the same context
    │
    └── start_host_orchestrator_runtime(startup_context, initial_world_binding)
            │
            ├── create host participant manifest
            ├── persist_active_world_binding(...)
            ├── emit startup runtime events
            └── advertise runtime live
```

### Restart and fail-closed drift flow

```text
world drift detected
    │
    ├── AutoRestart
    │      ├── restart_world_session(context, old_session) -> new WorldSession
    │      ├── persist_active_world_binding(...)
    │      └── emit world_restarted
    │
    └── FailClosed
           ├── persist_active_world_binding(...current authoritative binding...)
           ├── emit world_restart_required
           └── stop before more world work executes
```

### Operator truth surfaces

```text
backend-authoritative world binding
        │
        ▼
OrchestrationSessionRecord
        │
        ├── toolbox status can prove live binding
        ├── host runtime events carry top-level world fields
        ├── restart alerts carry top-level world fields
        └── selected status row stays host-scoped
```

## Implementation Plan

### Ordered execution checklist

1. Add persisted pending startup session context.
2. Thread startup owner proof into the first `start_world_session(...)` call.
3. Thread the same startup context through pre-live drift/restart paths.
4. Add parent-session binding mutators and clear rules.
5. Add parent-only binding persistence helper.
6. Move startup/restart/fail-closed alerts behind successful parent binding persistence.
7. Explicitly close the world and mark the parent record terminal on bootstrap failure after attach.
8. Stamp host/orchestrator runtime events from persisted parent binding.
9. Add `toolbox status --json` `active_world_binding`.
10. Add regression coverage for startup proof, pre-live drift, bootstrap cleanup, and contract preservation.

### Workstream 1: Persisted startup context

Primary file:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Tasks:

- introduce `PendingOrchestrationSessionContext` or equivalent
- generate `orchestration_session_id` before world startup
- persist parent session state=`Allocating` before first attach/create
- pass the same context into:
  - `start_world_session(...)`
  - `handle_detected_world_drift(...)`
  - `restart_world_session(...)`

### Workstream 2: Parent-only binding authority

Primary files:

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Tasks:

- add `set_world_binding` / `clear_world_binding`
- add `persist_active_world_binding(...)`
- keep host participant manifests world-empty
- keep lease-sidecar world binding out of slice `04` unless a same-slice reader appears

### Workstream 3: Bootstrap failure and drift ordering

Primary file:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Tasks:

- after `restart_world_session(...)`, persist parent binding before `world_restarted`
- in fail-closed drift, re-persist the current authoritative binding before `world_restart_required`
- on bootstrap failure after world attach:
  - close the world session
  - mark the parent record terminal
  - clear binding only after close

### Workstream 4: Live proof surface and event stamping

Primary files:

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

Tasks:

- derive host/orchestrator event world fields from the persisted parent snapshot
- add `active_world_binding` to `toolbox status --json`
- preserve selected status rows exactly as-is
- make proof-field omission non-fatal when live resolution is ambiguous or unavailable

### Workstream 5: Tests and docs

Primary files:

- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

Tasks:

- add first-start owner-proof coverage
- add startup-drift-before-first-command coverage
- add bootstrap-failure cleanup coverage
- add toolbox proof-surface coverage
- document that slice `04` is the session-scoped binding bridge before slice `06`

## Architecture Review

### Locked architecture decisions

1. **No host-manifest exception.**
   - Host participant invariants stay intact.

2. **Persist the parent session before the first world attach.**
   - In-memory preallocation is not enough for pre-live drift and bootstrap failure.

3. **Parent-only writes are the binding authority barrier.**
   - Binding truth must not depend on unrelated participant/lease rewrites.

4. **Persist before publish.**
   - Alerts and host runtime events that claim binding must read from persisted parent truth.

5. **`toolbox status` is the visible live proof surface.**
   - `doctor` stays a preflight/readiness surface.

### Architecture acceptance gates

1. **Startup gate**
   - First shared-world attach/create is owner-bound and a pending parent record already exists on disk.

2. **Pre-live drift gate**
   - Startup drift/restart paths retain owner proof and alert attribution before the host runtime becomes active.

3. **Bootstrap cleanup gate**
   - If bootstrap fails after world attach, the world is closed and the parent record is marked terminal.

4. **Ordering gate**
   - `world_restarted` and `world_restart_required` only publish after parent binding persistence succeeds.

5. **Contract gate**
   - Selected status rows remain unchanged while toolbox status gains optional proof.

## Code Quality Review

### Implementation guardrails

1. One helper owns parent binding persistence ordering.
2. Startup context is explicit and durable, not inferred from active-session lookup.
3. Host participant manifests stay world-empty with no special-case hole.
4. Public proof surfaces omit `orchestration_session_id`.
5. No trace reconstruction in live runtime paths.
6. No new registry layout or daemon lands in this slice.

### Minimal-diff rules

- reuse `OrchestrationSessionRecord`
- reuse `persist_orchestration_session(...)` for binding authority writes
- reuse `resolve_live_orchestrator_session(...)` for toolbox live authority
- avoid lease-sidecar expansion until there is a reader

## Error & Rescue Registry

| Failure point | What goes wrong | Expected rescue / fail-closed behavior |
|---|---|---|
| no pending parent record before first attach | startup shared world has no durable authority anchor | hard failure, slice not done |
| bootstrap fails after world attach | orphaned shared world remains bound to a session that never became live | close world, mark parent terminal, clear binding after close |
| startup drift before runtime activation | restart/fail-closed path loses owner proof because no active session exists yet | thread explicit startup context through drift paths |
| host manifest accidentally gets world fields | runtime violates its own invariant | compile/test failure, revert to parent-only storage |
| alert published before parent persist | event and live runtime state disagree | fail test, fix ordering |
| toolbox proof field flakes on ambiguous live resolution | operators lose trust in the proof surface | omit field, keep command non-fatal |

## Test Review

100% new-path coverage is the goal. This slice is all lifecycle edge conditions. Those are the bugs that wake you up late.

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/repl/async_repl.rs
    │
    ├── [★★  TESTED] world startup / restart flows already exist
    ├── [GAP]        startup persists pending parent session before first world attach
    ├── [GAP]        first shared-world request carries AttachOrCreate owner proof
    ├── [GAP]        startup drift paths reuse the same startup context
    ├── [GAP]        persist_active_world_binding persists parent snapshot before alerts/events
    ├── [GAP]        bootstrap failure after attach closes world and marks parent terminal
    └── [GAP]        fail-closed re-persists current binding before world_restart_required

[+] crates/shell/src/execution/agent_runtime/orchestration_session.rs
    │
    ├── [GAP]        set_world_binding updates only parent-session binding fields
    └── [GAP]        clear_world_binding is used only at approved clear points

[+] crates/shell/src/execution/agents_cmd.rs
    │
    ├── [★★★ TESTED] selected host orchestrator rows omit world fields today
    ├── [GAP]        toolbox status JSON shows active_world_binding from live parent session
    └── [GAP]        proof-field omission is non-fatal on ambiguous live resolution

[+] crates/shell/src/execution/repl_persistent_session.rs
    │
    ├── [★★  TESTED] shared-world echo validator already exists
    └── [GAP]        first-start startup path no longer relies on “no request means no proof”

─────────────────────────────────
COVERAGE: 3/13 paths tested (23%)
  Code paths: 3/13 (23%)
QUALITY:  ★★★: 1  ★★: 2  ★: 0
GAPS: 10 paths need tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] First shared-world startup
    │
    ├── [GAP] [→E2E] pending parent session persists before first attach/create
    ├── [GAP]         active binding is persisted before startup runtime events
    └── [GAP]         bootstrap failure after attach closes the world and marks parent terminal

[+] Auto-restart on drift
    │
    ├── [GAP] [→E2E] parent binding updates before world_restarted
    └── [GAP]         subsequent host runtime events carry the new binding

[+] Fail-closed drift
    │
    ├── [GAP] [→E2E] second command is blocked
    └── [GAP]         toolbox status omits or reports binding consistently without flaking

[+] Operator surfaces
    │
    ├── [GAP]         agent status selected row remains host-scoped with no world fields
    └── [GAP]         toolbox status publishes active_world_binding when live resolution succeeds

─────────────────────────────────
COVERAGE: 0/9 flows tested (0%)
  User flows: 0/9 (0%)
GAPS: 9 flows need tests (3 need integration coverage)
─────────────────────────────────
```

### Required test additions by file

#### `crates/shell/src/repl/async_repl.rs`

Add focused coverage for:

- persisting pending startup context before first world attach
- threading that context through startup drift/restart
- persisting parent binding before startup runtime events
- closing the world and marking parent terminal on bootstrap failure after attach
- ordering restart/fail-closed alerts behind parent binding persistence

#### `crates/shell/src/execution/agent_runtime/orchestration_session.rs`

Add unit coverage for:

- `set_world_binding("wld_123", 7)`
- approved `clear_world_binding()` call points

#### `crates/shell/src/execution/repl_persistent_session.rs`

Add validation coverage proving:

- first-start shared-world requests send owner proof
- missing proof on first attach/create is rejected by the startup path

#### `crates/shell/tests/repl_world_first_routing_v1.rs`

Add integration coverage for:

- first-start owner proof
- startup drift before first command still retains owner proof
- parent binding persisted before `world_restarted`
- bootstrap failure cleanup after first attach

#### `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Add contract coverage for:

- `toolbox status --json` `active_world_binding`
- selected host status rows still omit world fields
- proof-field omission remains non-fatal on ambiguous live resolution

### Test commands

Run at minimum:

```bash
cargo test -p substrate-shell start_host_orchestrator_runtime -- --nocapture
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
cargo test -p substrate-shell agent_successor_contract_ahcsitc0 -- --nocapture
```

Then run:

```bash
cargo test -p substrate-shell -- --nocapture
cargo test --workspace -- --nocapture
```

### QA artifact

Primary QA artifact for follow-up verification:

[spensermcconnell-feat-shared-world-ownership-contract-eng-review-test-plan-20260429-220341.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-shared-world-ownership-contract-eng-review-test-plan-20260429-220341.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
|---|---|---|---|---|---|
| first-start owner proof | world attaches without durable orchestration ownership proof | planned | planned | partial today | yes until fixed |
| bootstrap failure after attach | world stays bound after host runtime never becomes live | planned | planned | yes | yes until fixed |
| pre-live drift | startup restart/fail-closed loses owner proof or alert attribution | planned | planned | yes | yes until fixed |
| alert ordering | restart alert outruns persisted parent truth | planned | planned | yes | yes until fixed |
| toolbox proof surface | proof field flakes on ambiguous live resolution | planned | planned | partial | no, but trust regression |
| selected status contract | host-scoped row leaks world fields | planned | planned | yes | no, contract regression |

Critical gap rule:

If the first shared-world startup still happens without a persisted pending parent session and explicit owner proof, this slice is not done.

## Performance Review

No new cache is justified.

Hard rules:

- startup context is created once per REPL startup
- parent binding writes happen on startup and restart boundaries, not per wrapper event
- no live trace scans are introduced
- no new registry layout or daemon lands here

Footguns to avoid:

1. Making active-session lookup guess pending startup sessions.
2. Rewriting participant + lease files as part of binding-only persistence.
3. Using `doctor` as a live runtime truth surface.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
|---|---|---|
| Pending startup context | `crates/shell/src/repl/` | — |
| Parent binding mutators + ordering | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/repl/` | pending startup context |
| Toolbox proof surface | `crates/shell/src/execution/` | parent binding mutators |
| Integration tests + docs | `crates/shell/tests/`, `llm-last-mile/` | all above |

### Parallel lanes

Lane A: pending startup context -> parent binding ordering  
Lane B: toolbox proof surface after parent binding API settles  
Lane C: integration tests + docs last

### Execution order

1. Land Lane A first.
2. Launch Lane B after the parent binding API exists.
3. Run Lane C last.

### Conflict flags

- Lane A and Lane C both touch `repl_world_first_routing_v1.rs`
- Lane B and Lane C both touch status/toolbox contract tests

## Deferred Work

There is no `TODOS.md` in this repo root, so deferrals stay here explicitly.

1. Member invalidation and replacement lineage on generation changes  
Why: explicit job of [PLAN-05](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)

2. Full grouped session-centric registry layout under `agent-sessions/`  
Why: explicit job of slice `06`

3. Any change to selected-orchestrator status JSON  
Why: deliberately preserved contract in this slice

4. Lease-sidecar world-binding expansion  
Why: defer until a concrete reader requires it before slice `06`

5. Broader packet reframe into a single thin session-registry vertical slice  
Why: valid strategic alternative, but outside this packet’s current user direction

## NOT in Scope

- host participant manifest world-field population
- selected-orchestrator status schema changes
- member invalidation semantics
- grouped session-centric registry layout
- public trace schema changes
- UI work

## Definition of Done

This slice is done when all of these are true:

1. A pending parent session record is persisted before the first shared-world attach/create request.
2. That first attach/create request is owner-bound.
3. Startup drift/restart paths retain that startup context before the host runtime becomes active.
4. `OrchestrationSessionRecord` persists the active `world_id/world_generation`.
5. Host participant manifests remain world-empty.
6. If bootstrap fails after world attach, the world is closed and the parent record is marked terminal.
7. `world_restarted` emits only after the new parent binding is persisted.
8. `world_restart_required` emits only after the current authoritative binding is re-persisted.
9. Host/orchestrator runtime events stamp top-level world fields from the persisted parent snapshot.
10. `toolbox status --json` proves or safely omits the live active binding without flaking.
11. `agent status --json` selected host orchestrator rows remain unchanged.

## Completion Summary

- Step 0: Scope Challenge - scope accepted with two corrective expansions: persisted pending startup context and toolbox-status proof surface
- Architecture Review: 5 locked decisions, 5 acceptance gates
- Code Quality Review: 6 implementation guardrails, 4 minimal-diff rules
- Test Review: coverage diagrams produced, 19 concrete gaps/assertions identified
- Performance Review: 0 major issues, 3 no-cache/no-guessing rules
- Error & Rescue Registry: written
- NOT in scope: written
- What already exists: written
- Dream state delta: written
- TODOS.md updates: deferred scope captured in-plan because no `TODOS.md` exists
- Failure modes: 4 critical gaps flagged until startup proof, bootstrap cleanup, and parent-only binding persistence land
- Outside voice: completed and incorporated
- Parallelization: 3 lanes, 1 foundational lane, 1 surface lane, 1 validation lane
- Lake Score: complete version chosen for every in-slice decision

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | Premise | Reject host-manifest world binding as the slice-04 authority surface | Mechanical | Explicit over clever | Current host participant invariant forbids it | Forcing world fields into host manifests |
| 2 | Startup | Persist a pending parent session before first attach/create | Mechanical | Completeness | Pre-live drift and bootstrap failure need a durable anchor | In-memory preallocation only |
| 3 | Authority | Use `OrchestrationSessionRecord` as the bridge store | Mechanical | Pragmatic | Smallest correct bridge before slice `06` | New registry layout in this slice |
| 4 | Ordering | Make parent-only persistence the binding authority barrier | Mechanical | Systems over heroes | Avoids non-atomic participant/lease rewrites for binding-only truth | Tying binding truth to participant + lease writes |
| 5 | Visibility | Add `toolbox status --json` proof surface | Taste | Bias toward action | Gives operators one live, queryable win without breaking selected status | Keeping the slice entirely invisible |
| 6 | Scope | Defer member invalidation to `PLAN-05` | Mechanical | Scope discipline | Packet already gives it a dedicated slice | Pulling invalidation into `PLAN-04` |
| 7 | Storage | Defer lease-sidecar world-binding expansion until a reader exists | Taste | Minimal diff | Avoids bridge debt with no consumer | Expanding lease sidecars preemptively |

## CEO DUAL VOICES — CONSENSUS TABLE

| Dimension | Claude Subagent | Codex | Consensus |
|---|---|---|---|
| Premises valid? | flagged host-manifest contradiction and startup-proof gap | flagged same plus packet-end-state tension | CONFIRMED on host-manifest contradiction and startup proof |
| Right problem to solve? | yes, but only with corrected authority layer | yes, but packet-wide vertical-slice alternative exists | CONFIRMED for current packet, strategic alternative deferred |
| Scope calibration correct? | bridge-store approach okay after correction | warned against over-investing in transitional stores | CONFIRMED after narrowing to parent-only bridge and deferring registry reshape |
| Alternatives sufficiently explored? | requested parent-only and no-lease variants | requested thin session-registry vertical slice as strategic alternative | CONFIRMED that alternatives had to be expanded |
| Competitive / operator risk covered? | wanted one visible proof point | wanted a visible proof point but not on status | CONFIRMED via toolbox-status proof surface |
| 6-month trajectory sound? | okay if startup and lifecycle edges are fixed | okay if clearly framed as bridge, not end-state | CONFIRMED after bridge framing |

## ENG DUAL VOICES — CONSENSUS TABLE

| Dimension | Claude Subagent | Codex | Consensus |
|---|---|---|---|
| Architecture sound? | yes after shifting to parent-session authority | yes after parent-only authority and persisted startup context | CONFIRMED |
| Test coverage sufficient? | no, bootstrap failure + startup drift gaps missing | no, same gaps missing | CONFIRMED gaps added |
| Performance risks addressed? | mostly yes, but parent+child atomicity was overstated | same | CONFIRMED by parent-only binding writes |
| Security threats covered? | wanted explicit trust-boundary note and less public id exposure | same concern indirectly | CONFIRMED via proof-surface omission of orchestration_session_id |
| Error paths handled? | bootstrap failure, pre-live drift, and clear points missing | same | CONFIRMED gaps added |
| Deployment / operational risk manageable? | yes after cleanup + ordering rules | yes after doctor/toolbox surface correction | CONFIRMED |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 2 | CLEAR | Corrected host-manifest premise, added startup-proof requirement, expanded alternatives, and reframed slice `04` as a bridge to the session-centric registry |
| Codex Review | `/codex review` | Independent 2nd opinion | 2 | CLEAR | Outside voices forced the parent-session authority model, pushed live proof onto toolbox status instead of doctor, and tightened bridge vs end-state framing |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 2 | CLEAR | Added persisted pending startup context, pre-live drift threading, bootstrap-failure cleanup, parent-only binding writes, and full regression matrix |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**CODEX:** Two Codex passes materially improved this plan. The first rejected host-manifest storage and pushed the bridge-store framing. The second forced the more important lifecycle corrections: persist the pending parent record before first attach, keep binding writes parent-only, and move the proof surface from doctor to toolbox status.

**CROSS-MODEL:** Both Claude and Codex independently flagged the same four risks: host-manifest contradiction, startup owner-proof gap, lifecycle cleanup on bootstrap failure, and the need for a visible but contract-safe proof surface. That is high-confidence signal.

**UNRESOLVED:** 0 blocking implementation decisions remain inside slice `04`. The only deferred item is the larger packet-level alternative of collapsing slices `04`/`05`/`06` into one thinner session-registry vertical slice.

**VERDICT:** CEO + ENG CLEARED. `PLAN-04` is ready to implement after `PLAN-03`, before `PLAN-05`, and as an explicit bridge toward slice `06`.
