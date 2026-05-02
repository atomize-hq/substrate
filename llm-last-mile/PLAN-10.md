<!-- /autoplan restore point: /Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/feat-session-centric-state-store-autoplan-restore-20260501-194949.md -->

# PLAN-10: Production World-Scoped Member Runtime Launch

Source file: [10-member-runtime-launch-seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/10-member-runtime-launch-seam.md)  
Branch: `feat/session-centric-state-store`  
Plan type: shell/runtime launch seam, no UI scope, strong DX scope  
Review posture: `/autoplan`-style scope tightening with `/plan-eng-review` structure and rigor  
Status: execution-ready after [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md), with outside voice skipped on 2026-05-01 because `claude` CLI auth is missing

## Objective

This slice is not a scheduler.

It is the first real production seam that opens one world-scoped member runtime through the same
UAA control boundary already used by the host orchestrator path, persists that member under the
existing session-centric runtime store, and keeps status, restart invalidation, and trace output
honest.

The repo already has the right bones:

- host orchestrator runtime ownership in
  [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
- participant constructors and live-state invariants in
  [crates/shell/src/execution/agent_runtime/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
- session-centric persistence and stale-generation invalidation in
  [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- live operator projections in
  [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

What is missing is the actual producer.

`PLAN-10` lands that producer in the smallest complete vertical slice:

1. choose one world-scoped member backend from inventory, fail closed if the choice is ambiguous,
2. lazily launch it from the world-backed REPL command path under an already-live orchestrator
   session,
3. persist the member in `allocating`, then only advertise it live after the same retained-control
   checks already used by the orchestrator path pass,
4. replace it on world-generation rollover with fresh lineage,
5. prove the result through runtime, restart, status, and trace tests.

That is enough to move the repo from "fixture-only member truth" to "real member launch exists in
production code" without quietly smuggling in a multi-member control plane.

## Step 0: Scope Challenge

### 0A. Repo truth and why this slice exists

The SOW is directionally right, but parts of its implementation shape are more ambitious than the
codebase needs.

What the repo already proves today:

1. `prepare_host_orchestrator_runtime_startup(...)` and
   `start_host_orchestrator_runtime_with_prepared(...)` in
   [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   already perform effective-config resolution, policy allowlisting, runtime realizability checks,
   gateway construction, participant persistence, and retained-control lifecycle management.
2. `AsyncReplAgentRuntime` plus `RetainedRunControl` in
   [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   are already generic runtime-ownership carriers. They are not host-only in any structural sense.
3. `AgentRuntimeParticipantRecord::new_member_participant(...)` and
   `AgentRuntimeParticipantRecord::new_replacement_participant(...)` in
   [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   already enforce:
   - `role=member`
   - `execution.scope=world`
   - required `orchestrator_participant_id`
   - required `world_id` + `world_generation`
   - valid replacement lineage
4. `can_advertise_live()`, `has_valid_ownership()`, and `is_authoritative_live()` in
   [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
   already encode the exact live-authority rule this slice needs:
   no UAA session id, no retained control, no active event stream, or no completion observer means
   the participant is not live.
5. `resolve_live_orchestrator_participant(...)`,
   `list_live_participants_for_session(...)`, and
   `invalidate_stale_world_members_for_session(...)` in
   [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   already provide the parent lookup and invalidation primitives needed for launch and replacement.
6. `build_status_report(...)`, `build_toolbox_status_report(...)`, and the doctor posture in
   [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
   already consume member rows correctly once a real runtime producer exists.

What is still missing:

1. no real member-selection rule exists,
2. no world-backed REPL command path ensures a member runtime exists before work needs it,
3. no production caller instantiates a member participant outside tests and fixtures,
4. no runtime replacement path creates a fresh member participant on the new world generation,
5. doctor and launch preflight can still drift because member selection truth lives in prose and
   scattered helpers rather than one shared rule.

### 0B. Premise challenge

Premise check, one by one:

1. **The first shipped slice should be one host orchestrator plus one in-world member, not a
   generalized `/v1/agents` service.**
   - Accepted.
   - This matches the v1 runtime slice in
     [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md).

2. **The launch seam should reuse the same `run_control(...)` boundary and retained-control live
   gating as the host orchestrator path.**
   - Accepted.
   - Anything weaker would be a fake live row with better spelling.

3. **This slice needs a new global member-runtime registry or gateway cache.**
   - Rejected.
   - `build_gateway_for_descriptor(...)` in
     [registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
     is already descriptor-generic, and `AsyncReplAgentRuntime` already carries the retained
     runtime ownership that a member session needs.
   - Add a small member-launch helper, not a hidden second control plane.

4. **The shell should auto-launch arbitrary world members at REPL startup.**
   - Rejected.
   - The clean first caller is the world-backed command path in
     [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
     immediately before `execute_command(...)`. Launch lazily on first need, not on session open.

5. **The selection rule can stay vague for now.**
   - Rejected.
   - This is the biggest hidden gap. Without an explicit v1 selection rule, "select an allowed
     member backend" just means future code gets to guess.

6. **Replacement launch belongs in this slice, not later.**
   - Accepted.
   - Restart invalidation without a replacement producer leaves the live model half-real.

### 0C. Existing code to reuse

| Sub-problem | Existing code | Plan |
| --- | --- | --- |
| Unique host orchestrator discovery | `resolve_live_orchestrator_participant(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse directly |
| Runtime descriptor shape | `RuntimeSelectionDescriptor` in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse, keep generic |
| Runtime realizability | `validate_runtime_realizability(...)` in [validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Reuse after wording cleanup |
| Gateway construction | `build_gateway_for_descriptor(...)` in [registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs) | Reuse, likely no logic change |
| Member record construction | `new_member_participant(...)` and `new_replacement_participant(...)` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Reuse, no parallel record shape |
| Live-state persistence | `persist_participant(...)` and `persist_orchestration_session(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse |
| World binding authority | `persist_world_binding_authority(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse |
| Restart invalidation | `invalidate_stale_world_members_after_binding(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) and `invalidate_stale_world_members_for_session(...)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse, then add replacement launch on top |
| Lifecycle ownership gating | `can_advertise_live()` and `is_authoritative_live()` in [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Reuse exactly |
| Event and snapshot machinery | `translate_wrapper_event(...)`, `build_runtime_message_event(...)`, `persist_runtime_snapshots(...)` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | Reuse, parameterize role-specific messages only |
| Operator projections | `build_status_report(...)` and `build_toolbox_status_report(...)` in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs) | Reuse, touch only if launch preflight must align |

### 0D. Dream state and 12-month ideal

```text
CURRENT REPO
    │
    ├── host orchestrator runtime launches for real
    ├── member participant records exist mostly in fixtures/tests
    ├── status/toolbox can project member truth if a producer exists
    ├── restart invalidation can tombstone stale members
    └── no production member launch or replacement producer
            │
            ▼
THIS PLAN
    │
    ├── one explicit v1 member-selection rule
    ├── one lazy launch caller in the REPL world-backed command path
    ├── one real member runtime using the same retained-control contract
    ├── one replacement flow after world-generation rollover
    └── one test wall proving status, restart, and trace all consume the real producer
            │
            ▼
12-MONTH IDEAL
    │
    ├── multiple world members can exist safely under the same session
    ├── selection is explicit and productized
    ├── routing/dispatch chooses among members intentionally
    ├── restart replacement is routine, not special
    └── live operator surfaces and trace stay boring because the authority model never forked
```

### 0E. Implementation alternatives

| Approach | Summary | Effort | Risk | Decision |
| --- | --- | --- | --- | --- |
| A. Lazy, single-member v1 seam in the REPL world-backed command path | One explicit member-selection rule, one real launch path, one replacement path, minimal new abstractions | Medium | Low | **Accepted** |
| B. Auto-launch a member at REPL startup whenever world is enabled | Easier trigger, surprising behavior, launches work the user may never need | Medium | Medium | Rejected |
| C. General member manager with cache, scheduler, or `/v1/agents` surface | Future-looking, spends an innovation token too early | Large | High | Rejected |
| D. Keep fixtures/docs only and defer the real producer again | Avoids code now, keeps the hardest seam imaginary | Small | Unacceptable | Rejected |

### 0F. Complexity, search, completeness, and distribution checks

The full diff will still touch more than 8 files because the tests are non-negotiable. That is
fine.

The production seam must stay bounded anyway:

1. [crates/shell/src/execution/agent_runtime/validator.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
2. [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
3. optional small alignment in
   [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
4. test files:
   - [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
   - [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
   - [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
   - [crates/shell/tests/agent_hub_trace_persistence.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_hub_trace_persistence.rs)

Default no-touch posture unless the implementation proves otherwise:

- [crates/shell/src/execution/agent_runtime/registry.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/registry.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

`[Layer 1]` wins:

- reuse `RuntimeSelectionDescriptor`
- reuse `build_gateway_for_descriptor(...)`
- reuse `AsyncReplAgentRuntime`
- reuse `invalidate_stale_world_members_for_session(...)`
- reuse `build_status_report(...)`

The complete version is still the obvious one:

- define the v1 selection rule now,
- wire the real launch seam now,
- add the replacement producer now,
- add the regression wall now.

Distribution check:

- no new binary, package, container image, or artifact type is introduced here,
- distribution work is not applicable.

### 0G. What already exists

1. host orchestrator runtime ownership is already real,
2. member manifest shape and validation are already real,
3. authoritative live-state storage is already real,
4. stale-generation invalidation is already real,
5. status and toolbox projection logic are already real,
6. trace schema already expects pure-agent world rows to carry top-level `world_id` and
   `world_generation`,
7. the remaining gap is one live production caller and its replacement path.

### 0H. NOT in scope

- a generalized multi-member scheduler
- a public `/v1/agents` service
- host-scoped member product work
- automatic startup-time launch of arbitrary members
- new world ownership or rollback semantics already frozen by
  [PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
  and
  [07-world-replacement-ordering-rollback-atomic-metadata.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/07-world-replacement-ordering-rollback-atomic-metadata.md)
- live-state authority-cutover changes already frozen by
  [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md)
- reusing
  [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)
  as member-session truth
- new UI

## Architecture Contract

### V1 member-selection rule

This plan has to decide what "select an allowed member backend" means in code.

V1 rule:

1. read the effective inventory,
2. collect enabled pure-agent entries that resolve to:
   - `execution.scope=world`
   - `protocol=uaa.agent.session`
   - CLI persistent mode that passes `validate_runtime_realizability(...)`
3. require **exactly one** matching member candidate,
4. if there are zero, skip launch and keep current host-only behavior,
5. if there are more than one, fail closed with an explicit "ambiguous world member selection"
   error until a later slice adds a selector,
6. if the selected member backend is not allowlisted by `agents.allowed_backends`, fail closed
   before any runtime is started.

This does two useful things:

- it gives the runtime a real production choice rule now,
- it prevents this slice from inventing a selector config surface by accident.

### Launch preflight contract

A member launch may begin only when all of the following are true:

1. the REPL is in a world-backed command path,
2. `RuntimeOrchestrationContext` exists,
3. the runtime store resolves exactly one live orchestrator parent via
   `resolve_live_orchestrator_participant(...)`,
4. the parent session has authoritative `world_id` plus `world_generation`,
5. the unique selected member passes inventory, capability, runtime-realizability, and policy
   allowlist checks,
6. there is no already-live member runtime bound to the same generation for the same selected
   member.

Fail-closed rules:

- missing parent session: hard error
- inactive/ambiguous parent: hard error
- missing authoritative binding: hard error
- ambiguous member candidate: hard error
- denied backend: hard error
- no synthetic fallback to host execution

### First production caller

The first production caller is the world-backed command path in
[crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
immediately before `execute_command(...)`.

New internal helper:

```text
ensure_member_runtime_ready(startup_context, world_session, selected_command_context)
```

Behavior:

1. if no eligible world member exists, no-op,
2. if an eligible member exists and a matching live member runtime is already authoritative-live
   on the current `world_generation`, reuse it,
3. otherwise launch a new member runtime lazily and persist it through the store,
4. after world restart, if the stored member runtime is from the stale generation, invalidate old
   state first, then create a replacement participant and launch again on the new generation.

This is the right owner because `async_repl.rs` already holds:

- the live world session,
- the orchestration context,
- the host orchestrator runtime,
- the restart boundary,
- and the command loop that actually needs the member runtime.

### Runtime ownership and lifecycle

Do not add a second member-only runtime holder unless reuse proves impossible.

Plan:

1. reuse `AsyncReplAgentRuntime` and `RetainedRunControl` for member sessions too,
2. add a tiny role-specific prepared-launch wrapper and role-specific helper functions,
3. share the existing event/completion/heartbeat/shutdown machinery instead of copying it.

Minimum new runtime state in the REPL loop:

```text
host_orchestrator_runtime: Option<AsyncReplAgentRuntime>
member_runtime: Option<AsyncReplAgentRuntime>
selected_member_agent_id: Option<String>
```

That is enough for the v1 "one member per session" slice.

### Lifecycle state machine

```text
member selected
    │
    ▼
prepare_member_runtime_startup(...)
    │
    ├── resolve live parent orchestrator
    ├── read authoritative world binding
    ├── allocate participant_id + lease_token + run_id
    └── new_member_participant(... state=allocating)
            │
            ▼
persist allocating snapshot
            │
            ▼
gateway.run_control(...)
            │
            ├── failure before retained ownership
            │      └── state=failed, parent remains honest
            │
            └── event + completion observers retained
                    │
                    ├── UAA session id observed
                    ├── control retained
                    ├── event stream active
                    └── completion observer retained
                            │
                            ▼
                  can_advertise_live() == true
                            │
                            ▼
                  state=ready -> running
                            │
                            ├── normal shutdown -> stopping -> stopped
                            ├── stream/completion loss -> invalidated or failed
                            └── world restart -> invalidated old, launch replacement
```

Critical invariant:

`new_member_participant(...)` does **not** make the member live. Only the retained UAA ownership
boundary does. This must stay identical to the host orchestrator posture.

### Dependency graph

```text
effective inventory + policy
        │
        ▼
validate_member_selection(...)
        │
        ▼
validate_runtime_realizability(...)
        │
        ▼
build_gateway_for_descriptor(...)
        │
        ▼
async_repl ensure_member_runtime_ready(...)
        │
        ├── resolve_live_orchestrator_participant(...)
        ├── authoritative parent world binding
        ├── new_member_participant(...)
        ├── persist_participant(...)
        └── gateway.run_control(...)
                │
                ├── event stream -> translate_wrapper_event(...)
                ├── completion observer
                └── persist_runtime_snapshots(...)
                        │
                        ▼
        state_store + canonical participant snapshot
                        │
                        ├── agent status
                        ├── trace persistence
                        ├── tombstone suppression
                        └── restart invalidation + replacement
```

### Restart replacement flow

```text
world restart committed
        │
        ├── persist_world_binding_authority(...)
        ├── invalidate_stale_world_members_after_binding(...)
        └── if live member runtime existed for old generation:
                │
                ├── allocate replacement participant_id
                ├── new_replacement_participant(...)
                ├── resumed_from_participant_id = old participant
                ├── world_generation = new authoritative generation
                └── relaunch through same retained-control path
                        │
                        ├── success -> replacement becomes live
                        └── failure -> absence remains honest, stale member stays invalidated
```

### Error & Rescue Registry

| Failure | Expected behavior | Persisted state | Operator-facing result | Rescue posture |
| --- | --- | --- | --- | --- |
| No unique world member candidate | fail before launch | no new live member | explicit selection error | fix inventory or add selector in later slice |
| Parent orchestrator missing or ambiguous | fail before launch | no new member | explicit live-parent error | repair parent session first |
| Parent world binding missing | fail before launch | no new member | explicit missing binding error | world must be restarted or rebound |
| Backend denied by policy | fail before launch | no new member | explicit allowlist error | update policy intentionally |
| `run_control(...)` bootstrap failure | mark member failed | `failed` participant with error bucket | clear launch failure | retry only after cause is fixed |
| Event stream ends after liveness | invalidate member | `invalidated` participant + terminal reason | status no longer shows it live | replacement on next use or after restart |
| Restart replacement fails | keep old member invalidated, do not resurrect | stale participant remains tombstoned | honest absence | retry replacement on next eligible command |

## File Plan

### Primary implementation surfaces

#### 1. `validator.rs`

Add two explicit helpers:

1. `validate_member_selection(...)`
2. `validate_member_launch_preconditions(...)` or equivalent narrow helper set

Required behavior:

- require exactly one enabled world-scoped UAA member candidate,
- require the same conservative UAA capability floor as the first host caller for v1,
- reuse `validate_runtime_realizability(...)` after its user-facing wording is made neutral enough
  to talk about "selected runtime" instead of always "selected orchestrator",
- return a normal `RuntimeSelectionDescriptor` so the rest of the runtime stays generic.

#### 2. `async_repl.rs`

This is the real seam.

Add:

1. lazy `ensure_member_runtime_ready(...)` before the world-backed `execute_command(...)` path,
2. `prepare_member_runtime_startup(...)`,
3. `start_member_runtime_with_prepared(...)`,
4. `shutdown_member_runtime(...)`,
5. replacement launch after world restart when an old-generation member existed.

Implementation rule:

- factor common retained-control lifecycle logic out of the host path enough to avoid copying the
  entire startup/completion/shutdown state machine,
- keep role-specific messages explicit,
- do not weaken the host path to make the member path "more generic."

#### 3. `agents_cmd.rs`

Touch only if needed to align operator preflight with reality.

Preferred alignment:

- doctor should fail closed when multiple enabled world-scoped members exist, because the runtime
  will fail closed too,
- doctor should use the same member-selection truth as launch preflight instead of a looser
  boolean-only posture.

`status` and `toolbox` logic should not need redesign. They are consumers, not the seam owner.

### Validation-first, likely no-change surfaces

#### `registry.rs`

Default expectation: no logic change.

`build_gateway_for_descriptor(...)` is already descriptor-generic. Only touch if the member path
reveals a concrete backend-kind registration gap.

#### `state_store.rs`

Default expectation: no new semantics.

The store already has:

- parent resolution,
- authoritative persistence,
- session-local live participant listing,
- stale-generation invalidation.

Only add a helper if the launch code would otherwise duplicate a real store rule. Do not move
launch ownership into the store.

#### `world_gateway.rs`

Do not touch unless a test proves unavoidable. It is adjacent infrastructure, not the member
runtime lifecycle owner.

## Code Quality Review

### Issue 1. Do not duplicate the host retained-control state machine

The biggest DRY trap is copying 300-400 lines of host lifecycle code and changing three strings.

Recommendation:

- extract shared internal runtime-startup and runtime-shutdown helpers,
- keep role-specific preflight separate,
- keep role-specific messages explicit.

Why:

- this repo already has the right live-gating logic in one place,
- if host and member lifecycle rules drift, the user gets two subtly different definitions of
  "live" in the same product.

### Issue 2. Do not overload "orchestrator selection" into vague dual-purpose helpers

`validate_orchestrator_selection(...)` is specific and that is good.

Recommendation:

- add parallel member-specific selection helpers,
- keep the low-level descriptor and realizability helpers generic.

Why:

- explicit beats clever,
- future contributors should see "select orchestrator" and "select member" as separate product
  decisions even if they share lower-level runtime plumbing.

### Issue 3. Keep the store as the only persistence owner

Do not let the launch seam write compatibility files, leases, or canonical session roots directly.

Recommendation:

- all parent/participant updates continue to flow through existing store helpers and existing
  runtime snapshot choke points.

Why:

- `PLAN-09` already paid to freeze the authority ladder,
- this slice should consume that contract, not reopen it.

### Issue 4. Minimize the production touch set

The source SOW names `registry.rs`, `state_store.rs`, and `agents_cmd.rs` as primary surfaces.
That is too broad for the first implementation pass.

Recommendation:

- treat `validator.rs` and `async_repl.rs` as the only default production logic owners,
- touch other files only if tests or alignment require it.

Why:

- minimal diff,
- easier verification,
- less chance of spending a week rediscovering a problem the runtime did not actually have.

## Test Review

### Test framework detection

- Runtime: Rust
- Framework: `cargo test`
- Primary package: `shell`
- No LLM prompt/eval suite expansion is required for this slice

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/shell/src/execution/agent_runtime/validator.rs
    │
    ├── validate_member_selection()
    │   ├── [GAP]         Zero world members -> no-op path documented and tested
    │   ├── [GAP]         Exactly one eligible world member -> success
    │   ├── [GAP]         Multiple eligible world members -> fail closed
    │   ├── [GAP]         Wrong protocol/scope -> fail closed
    │   └── [GAP]         Denied backend -> fail closed before launch
    │
    └── validate_runtime_realizability()
        ├── [★★  TESTED] Host orchestrator CLI persistent checks already covered
        └── [GAP]         Same checks exercised through member selection path

[+] crates/shell/src/repl/async_repl.rs
    │
    ├── ensure_member_runtime_ready()
    │   ├── [GAP] [->E2E] First world-backed command launches member lazily
    │   ├── [GAP]         Existing live member on same generation is reused
    │   └── [GAP]         Missing live parent or binding fails before launch
    │
    ├── prepare_member_runtime_startup()
    │   ├── [GAP]         Allocating snapshot persisted with parent linkage
    │   └── [GAP]         Replacement participant uses fresh lineage on restart
    │
    ├── start_member_runtime_with_prepared()
    │   ├── [GAP]         UAA session id + retained control -> Ready
    │   ├── [GAP]         Ready -> Running after event flow
    │   ├── [GAP]         Bootstrap error -> Failed
    │   ├── [GAP]         Stream loss after liveness -> Invalidated
    │   └── [GAP]         Shutdown -> Stopped
    │
    └── restart replacement hook
        ├── [★★  TESTED] Old-generation invalidation behavior already covered with fixtures
        ├── [GAP] [->E2E] Real replacement member launches on new generation
        └── [GAP] [->E2E] Replacement failure leaves honest absence

[+] crates/shell/tests/agent_successor_contract_ahcsitc0.rs
    │
    ├── agent status projections
    │   ├── [★★★ TESTED] Fixture/member rows keep world_id + world_generation
    │   ├── [GAP] [->E2E] Real launched member appears from runtime state, not trace fallback
    │   └── [★★★ TESTED] Invalidated tombstones suppress stale trace fallback
    │
    └── agent doctor
        ├── [★★★ TESTED] Allowlist and world-boundary fail-closed posture exists
        └── [GAP]         Ambiguous world-member selection fails closed

[+] crates/shell/tests/agent_hub_trace_persistence.rs
    │
    ├── host orchestrator runtime trace rows
    │   └── [★★★ TESTED] Host path persists runtime-owned pure-agent rows
    │
    └── member runtime trace rows
        ├── [GAP] [->E2E] Registered/status/terminal rows persist for member runtime
        └── [GAP]         Replacement lineage persists in trace-compatible fields

─────────────────────────────────
COVERAGE: 4/19 paths tested (~21%)
  Code paths: 2/13
  Integration/user flows: 2/6
QUALITY:  ★★★: 3  ★★: 2  ★: 0
GAPS: 15 paths need tests (4 need E2E/integration)
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] First world-backed command in REPL
    ├── [GAP] [->E2E] Launch member lazily on first need
    ├── [GAP]         Reuse existing live member on second command
    └── [GAP]         Missing binding prints clear fail-closed error

[+] Shared-world restart with live member
    ├── [★★  TESTED] Fixture invalidation of stale generation exists
    ├── [GAP] [->E2E] Real replacement member becomes live on new generation
    └── [GAP] [->E2E] Replacement failure leaves no fake liveness

[+] Operator inspection
    ├── [GAP]         `substrate agent doctor --json` reports ambiguous member selection
    ├── [GAP]         `substrate agent status --json` shows live member from store
    └── [GAP]         `substrate agent toolbox status --json` stays orchestrator-anchored
```

### Missing tests to add to the plan

1. **`crates/shell/src/repl/async_repl.rs`**
   - Add unit/integration-style runtime tests for:
     - first lazy member launch success under a live orchestrator + authoritative binding
     - launch failure when binding is missing
     - launch failure when parent orchestrator is missing/inactive
     - ambiguous world-member selection failure
     - replacement participant creation after restart with fresh `participant_id` and
       `resumed_from_participant_id`
     - clean shutdown and unexpected control-stream loss

2. **`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`**
   - Add contract cases for:
     - real launched member rows in `agent status --json`
     - top-level `world_id` + `world_generation` coming from runtime state
     - doctor failure on ambiguous world-member selection
     - toolbox remaining anchored to the orchestrator session even when the member is live

3. **`crates/shell/tests/repl_world_first_routing_v1.rs`**
   - Add integration cases for:
     - shared-world startup followed by first-command member launch
     - shared-world restart causing member replacement on the new generation
     - replacement failure leaving absence rather than resurrecting stale liveness

4. **`crates/shell/tests/agent_hub_trace_persistence.rs`**
   - Add trace persistence cases for:
     - member `Registered` / `Status` / terminal events
     - replacement lineage fields
     - terminal member rows staying auditable without becoming live again

### Test artifact

The eng-review QA artifact for this plan is:

[spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260501-194949.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-centric-state-store-eng-review-test-plan-20260501-194949.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
| --- | --- | --- | --- | --- | --- |
| unique world-member selection | two enabled world-scoped members exist and the runtime guesses | no | not yet | no | yes until ambiguous-selection test and error land |
| lazy member launch preflight | parent orchestrator or world binding is missing but a partial member row is still persisted | no | partial | partial | yes until preflight-fails-before-launch test lands |
| retained-control member liveness | constructor-created member is treated as live before UAA ownership is retained | no | yes in model helpers | no | yes until launch-state progression test lands |
| restart replacement | stale member is invalidated but replacement never starts and a trace row brings it back | partial | partial | no | yes until replacement + suppression integration lands |
| toolbox anchoring | active member causes toolbox to drift away from orchestrator session | no | yes in current design | partial | yes until real-launched-member toolbox regression lands |
| trace persistence | terminal member rows omit lineage or world metadata | no | partial | yes | yes until trace persistence coverage lands |

Critical gap rule:

If the implementation can guess among multiple members, persist a half-live member before retained
ownership exists, or let restart replacement drift into stale trace resurrection, this seam is not
actually done.

## Performance Review

This slice is still correctness-first, but there are performance footguns worth naming.

Performance rules:

1. do not add a global gateway cache just to avoid building one backend wrapper per member launch,
2. do not rescan the world or rerun launch preflight on every command once a matching live member
   already exists for the current generation,
3. do not add extra live-state scans to `agent status` or `toolbox`,
4. do not let restart replacement loop over arbitrary historical members when the v1 slice only
   manages one selected live member.

Performance issues found:

- 0 material throughput issues

The real performance bug here would be architectural. Building a cache or manager to "speed up" a
single-member launch seam is exactly how a 50-line production need turns into a 500-line future
maintenance problem.

## DX Review

This slice has no UI scope. It absolutely has developer/operator scope.

The user here is the engineer trying to answer:
"If a world member is supposed to be live right now, can I prove that without guessing?"

### Developer journey map

| Stage | What the developer is doing | Current friction | Target after this slice |
| --- | --- | --- | --- |
| 1 | Enable one world-scoped member in inventory | medium, selection rule is implied not encoded | explicit unique-member rule |
| 2 | Start a REPL with world enabled | low | keep low |
| 3 | Run the first world-backed command | high, no real member producer exists | lazy member launch on first need |
| 4 | Inspect `substrate agent doctor --json` | medium, preflight is looser than launch reality | doctor matches launch truth |
| 5 | Inspect `substrate agent status --json` | medium, member truth is mostly fixture-based today | live member row comes from runtime state |
| 6 | Restart the shared world | medium-high, invalidation exists but replacement does not | replacement member or honest absence |
| 7 | Inspect `substrate agent toolbox status --json` | medium | unchanged orchestrator anchoring |
| 8 | Debug terminal failure or control-stream loss | medium | persisted member terminal reason is explicit |
| 9 | Extend to future multi-member work | high today | clear boundary: this slice is single-member by design |

### Developer empathy narrative

I turned on one world-scoped member because I wanted a real agent running inside the selected
shared world, not another promising test fixture.

When I run the first world-backed command, the system should either launch that member cleanly or
tell me exactly why it refused. Missing parent, missing world binding, ambiguous selection, denied
backend. Real reasons. No folklore.

Then I run `substrate agent status --json` and I should see the member there because the runtime
store says it is live, not because trace history looked optimistic. If the world restarted and the
replacement failed, I should see honest absence. Painful maybe. Honest definitely.

### DX Scorecard

| Dimension | Score | Notes |
| --- | --- | --- |
| Getting started | 6/10 | inventory works, member launch rule is not yet explicit |
| Naming guessability | 8/10 | `doctor`, `status`, and runtime terms are good |
| Error messages | 7/10 | fail-closed posture exists, but member-specific causes still need first-class wording |
| Docs findability | 6/10 | planning docs exist, runtime truth still needs this seam to become real |
| Upgrade path safety | 8/10 | single-member v1 slice keeps the blast radius small |
| Observability | 8/10 | trace + store model are good once the producer exists |
| Recovery guidance | 7/10 | restart invalidation is already clear, replacement flow needs to match |
| Escape hatches | 6/10 | ambiguity deliberately fails closed until a real selector exists |

Overall DX score: **7/10**

### DX Implementation Checklist

- make member selection explicit and deterministic,
- keep launch errors as problem + cause + fix style messages,
- keep `doctor` aligned with launch truth,
- keep `status` and `toolbox` consuming the existing live-state authority ladder,
- add one trace example or operator note only if the real member launch semantics drift from
  current `TRACE.md` wording.

### TTHW assessment

Current TTHW for "understand whether a real world member can launch and stay live" is about
**12-15 minutes**. You have to read the seam doc, the host runtime code, the state-store rules,
and the test fixtures to reconstruct the missing piece.

Target after this slice: **under 7 minutes**.

That means a maintainer can:

1. read `PLAN-10.md`,
2. run the targeted shell tests,
3. trust that `doctor`, `status`, and restart behavior all point at the same runtime truth.

## Cross-Phase Themes

These themes showed up across scope review, engineering review, and DX review:

1. **Reuse the existing retained-control runtime seam.**
   - The repo already paid for one honest definition of liveness. Use it.

2. **Define the member-selection rule now.**
   - Hidden selection logic is how runtime bugs become "config mysteries."

3. **Make the first caller lazy and explicit.**
   - Launch on the world-backed command path, not at startup and not in a hidden daemon.

4. **Keep restart replacement honest.**
   - Invalidating the stale generation without a replacement producer is only half a product.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Member selection and preflight contract | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/execution/` | — |
| B. Lazy launch + retained-control lifecycle reuse | `crates/shell/src/repl/` | A |
| C. Status/doctor/trace contract tests | `crates/shell/tests/`, `crates/shell/src/execution/` | A, B |
| D. Restart replacement integration tests | `crates/shell/tests/`, `crates/shell/src/repl/` | A, B |
| E. Optional docs alignment + final validation | `docs/`, `crates/shell/` | C, D |

### Parallel lanes

- Lane A: member-selection and preflight helper work
- Lane B: runtime lifecycle work after Lane A
- Lane C: status/doctor/trace tests after Lane B
- Lane D: restart replacement integration tests after Lane B, in parallel with Lane C
- Lane E: final validation and optional docs after C and D

### Execution order

1. Launch Lane A first.
2. Merge Lane A into the working base.
3. Launch Lane B.
4. Once Lane B compiles, launch Lanes C and D in parallel worktrees.
5. Merge C and D, then run Lane E for cleanup and the targeted command wall.

### Conflict flags

- `async_repl.rs` belongs to Lane B. Do not let test lanes patch it casually.
- `agents_cmd.rs` belongs to Lane A only if doctor alignment is required; otherwise leave it to
  Lane C test fallout only.
- `agent_successor_contract_ahcsitc0.rs` belongs to Lane C.
- `repl_world_first_routing_v1.rs` belongs to Lane D.
- `agent_hub_trace_persistence.rs` belongs to Lane C.

### Parallelization verdict

This slice is sequential at the runtime seam, then parallelizable at the test wall.

- **5 lanes total**
- **2 lanes can run in parallel after lifecycle work lands**
- **2 sequential checkpoints remain non-negotiable: selection/preflight first, full validation last**

## Deferred Work

There is no repo-root `TODOS.md`, so explicit deferrals stay here.

1. multi-member selection and scheduling
2. explicit member selector config surface if the product needs more than one world member
3. gateway pooling or caching if repeated member launch cost becomes measurable
4. host-scoped member product work
5. any public control-plane surface beyond the current REPL-owned seam

## Definition of Done

This slice is done only when all of the following are true:

1. one unique eligible world-scoped member can be selected deterministically from inventory,
2. the first world-backed REPL command can lazily launch that member under a live orchestrator
   session,
3. the launched member is persisted in `allocating` first and only becomes authoritative-live after
   retained UAA ownership is proven,
4. `substrate agent status --json` can surface the live launched member from runtime state with
   top-level `world_id` and `world_generation`,
5. `substrate agent toolbox status|env` remains orchestrator-scoped,
6. world restart invalidates the stale member and can launch a replacement participant on the new
   generation,
7. failed replacement leaves honest absence, not stale liveness,
8. doctor selection/preflight matches the real runtime launch rules,
9. the targeted shell tests and the four recommended commands below pass.

### Recommended verification commands

```bash
cargo test -p shell async_repl -- --nocapture
cargo test -p shell agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell repl_world_first_routing_v1 -- --nocapture
cargo test -p shell agent_hub_trace_persistence -- --nocapture
```

## Completion Summary

- Step 0: scope reduced to one lazy world-member launch seam, no scheduler
- Architecture Review: 4 issues found, all resolved in-plan by making ownership, selection, and the first caller explicit
- Code Quality Review: 4 issues found, all resolved in-plan with a smaller production touch set and shared lifecycle reuse
- Test Review: diagram produced, 15 direct coverage gaps identified
- Performance Review: 0 material performance issues found
- DX Review: 7/10 overall, TTHW 12-15 min to target under 7 min
- NOT in scope: written
- What already exists: written
- TODOS.md updates: 0, repo has no root `TODOS.md`, deferrals captured here
- Failure modes: 6 critical gaps flagged
- Outside voice: skipped, `claude` CLI is installed but unauthenticated on 2026-05-01
- Parallelization: 5 lanes, with test lanes parallel only after lifecycle work lands
- Lake Score: complete option chosen for every in-slice decision

<!-- AUTONOMOUS DECISION LOG -->
## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Scope | Ship one lazy world-member seam, not a scheduler | Mechanical | Pragmatic | The repo needs one real producer before it needs a manager | Scheduler or `/v1/agents` service |
| 2 | Selection | Require exactly one eligible world member for v1 | Taste | Explicit over clever | No selector config exists yet, so guessing would be the bug | Hidden heuristic selection |
| 3 | Caller | Launch lazily from the world-backed REPL command path | Mechanical | Bias toward action | This is the narrowest real production owner with all needed authority already in hand | Startup-time auto launch |
| 4 | Runtime ownership | Reuse `AsyncReplAgentRuntime` and retained-control lifecycle gates | Mechanical | DRY | The host path already encodes the only honest live-state rule | New member-only runtime holder |
| 5 | Persistence | Keep store-owned canonical persistence as the only write path | Mechanical | Systems over heroes | `PLAN-09` already froze the authority ladder | Direct caller writes to compatibility files |
| 6 | Restart | Include replacement launch in the same slice | Mechanical | Completeness | Invalidation without replacement leaves the product half-real | Defer replacement producer |
| 7 | Validation | Keep orchestrator and member selection helpers distinct, while sharing low-level realizability plumbing | Mechanical | Explicit over clever | The product concepts differ even if the runtime descriptor is generic | One vague dual-purpose selector |
| 8 | Tests | Treat real-launch integration coverage as mandatory, not optional cleanup | Mechanical | Boil the lake | This seam is exactly where fixture-only truth stops being enough | Happy-path-only unit coverage |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
| --- | --- | --- | --- | --- | --- |
| CEO Review | `/plan-ceo-review` | Scope and strategy | 1 | CLEAR | Reduced the slice to one lazy world-member launch seam, rejected startup auto-launch and scheduler drift, and made the v1 selection rule explicit |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | SKIPPED | No separate outside-model review run |
| Eng Review | `/plan-eng-review` | Architecture and tests (required) | 1 | CLEAR | Locked the first caller to the REPL world-backed command path, reused retained-control lifecycle machinery, and identified 15 direct coverage gaps that this plan closes |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**UNRESOLVED:** 0 plan-level decision points remain. The remaining work is implementation plus the
targeted runtime, restart, status, and trace tests already enumerated above.

**VERDICT:** CEO + ENG CLEARED. `PLAN-10` is ready to execute as the first real production
world-scoped member runtime launch seam on top of the session-centric runtime authority already
frozen by [PLAN-09.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-09.md).
