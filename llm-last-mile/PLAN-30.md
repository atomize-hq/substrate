# PLAN-30: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Source spec: [SPEC-30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)  
Adjacent landed slices: [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md)  
Follow-on slice: [31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Proposed branch: `feat/public-world-scoped-agent-start`  
Base branch: `main`  
Plan type: public caller-surface expansion with host-first world-backed delivery  
Status: draft realigned to host-first product intent on 2026-05-27

## Objective

Ship a truthful public `substrate agent start` surface that starts a host orchestration session first and uses world as the default execution substrate when scope resolution selects it.

This slice is complete only when all of the following are true:

1. `substrate agent start` accepts explicit scope selection and bare `start` resolves requested scope through workspace-local config/profile/policy first, then global config/policy.
2. `substrate agent start --scope world` creates a host-rooted durable orchestration session, persists authoritative host attach truth at session birth, and establishes world binding/session truth for later host-dispatched world work.
3. `substrate agent start --scope host` is the explicit bypass-world path.
4. Public dispatch-time capability narrowing is available only through:
   - `--disable-capability <capability>`
   - `--disable-cap <capability>`
5. The only supported narrowing targets remain:
   - `session_resume`
   - `session_fork`
   - `session_stop`
   - `status_snapshot`
   - `event_stream`
6. The inaugural operator prompt is handled by the host orchestration agent rather than being sent directly to a first world worker/member.
7. The default world-backed path uses the normal host lifecycle rather than `born_unattached` as the happy-path operator state.
8. Public world-scoped root start is Linux-first in this slice; non-Linux platforms fail closed with explicit posture guidance.

This is productization of a host-first orchestration model. It is not a world-first inaugural prompt model.

## Plan Summary

The repo already has the key ingredients:

1. public `agent start` and `agent turn` entrypoints in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs),
2. one shared dispatch-envelope contract in [`dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
3. authoritative persisted host attach truth in [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs),
4. Linux world binding/session plumbing plus later world-member dispatch seams in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs),
5. integration suites that already pin most public control behavior in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) and [`agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs).

What is still missing is narrower:

1. the public CLI still hardcodes host-only root start,
2. public capability narrowing has no caller-facing syntax for `agent start`,
3. bare `start` does not yet express the desired workspace/global scope-resolution order,
4. world-scoped root start is still framed as world-first / deferred-host-attach instead of host-first orchestration,
5. docs and tests still describe host-only public root start as the only shipped contract.

The minimum honest implementation is one ordered slice with four workstreams:

1. freeze the public start input contract in code and tests,
2. deliver host-rooted start birth plus world-backed session/binding setup,
3. preserve truthful host lifecycle/status semantics while enabling world-backed default scope,
4. update docs and land the end-to-end validation wall.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Public `agent start` entrypoint | [`run_start(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:377) | Reuse and extend. Do not invent a second public root-start verb. |
| Shared dispatch envelope | [`DispatchRequestEnvelope`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:113) | Reuse exactly. All new public scope/capability behavior must map here. |
| Supported narrowing family | [`validate_capability_override_shape(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:784) | Reuse exactly. Do not broaden the allowed family in this slice. |
| Persisted attach truth | [`HostAttachContract`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:72) | Reuse exactly. World-scoped root start must persist this truth at birth. |
| Current host-only root-start guard | [`build_start_launch_plan(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1066) | Replace the host-only hardcoding and add documented scope-resolution precedence. |
| Public session posture vocabulary | [`PublicSessionPosture`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:103) | Preserve current host lifecycle semantics for the thin slice. |
| Durable orchestration posture vocabulary | [`OrchestrationSessionPosture`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:69) | Reuse current attached/detached host lifecycle truth; do not make `born_unattached` the default happy path. |
| Linux world-member dispatch path | [`submit_world_prompt_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1511) | Keep for later host-dispatched world work rather than inaugural prompt handling. |
| Existing rejection coverage | [`public_root_start_rejects_world_scoped_backends_in_v1()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:3757) | Replace with the new approved world-start contract and preserve equivalent fail-closed coverage where still required. |

### Exact remaining gap

1. Public root-start CLI arguments do not yet carry `scope` or capability narrowing input.
2. Bare `start` does not yet resolve scope through the documented workspace/global config-policy precedence.
3. There is no launch path that creates a host-rooted attached orchestration session while also establishing the world-backed session/binding the host will later use.
4. The runtime and docs still lean on a world-first / deferred-host-attach contract that no longer matches the intended product.
5. Docs and integration tests still reflect the old host-only public root-start contract.

### Scope decision

Proceed as one cohesive slice.

Do not split this into separate “CLI flags first,” “runtime birth later,” and “status/doc cleanup last” branches. The contract is only honest when:

1. parsing and resolution precedence,
2. runtime behavior,
3. host lifecycle/status truth, and
4. docs/tests

all converge at the same time.

## Frozen Execution Contract

If implementation wants to deviate from this contract, update the spec and this plan first.

### Public start contract

Public root-start syntax becomes:

```text
substrate agent start --backend <backend_id> [--scope host|world] (--prompt <text> | --prompt-file <path> | --prompt-file -) [--disable-capability <capability> ...] [--disable-cap <capability> ...] [--json]
```

Rules:

1. Omitting `--scope` resolves requested execution substrate through workspace-local config/profile/policy first, then global config/policy.
2. `--scope host` means explicit bypass-world host start.
3. `--scope world` means host-rooted durable session plus authoritative world session/binding setup, never standalone world-root continuity.
4. `--disable-capability` is canonical, `--disable-cap` is the only alias, and there is no single-letter short flag.
5. Public capability narrowing is dispatch-time narrowing only and cannot set or broaden baseline capability truth.

### Capability contract

Public callers may narrow only:

1. `session_resume`
2. `session_fork`
3. `session_stop`
4. `status_snapshot`
5. `event_stream`

Public callers may not set or broaden:

1. `session_start`
2. `llm`
3. `mcp_client`

Those remain agent/inventory-level capability truth.

### Lifecycle contract

For world-backed start:

1. durable authority remains host-rooted,
2. authoritative host attach truth is persisted at birth,
3. the inaugural operator prompt is handled by the host orchestration agent,
4. world binding/session truth is established so later host-dispatched world work has an authoritative substrate,
5. this slice does not add a second inaugural prompt or direct world-agent bootstrap conversation.

### Status contract

The thin-slice happy path uses the normal host lifecycle:

1. host-attached start is truthful at birth,
2. later clean detach may normalize to `parked_resumable`,
3. pending inbox work may normalize to `awaiting_attention`.

`born_unattached` may remain a specialized or future posture, but it is not the primary operator-facing acceptance state for default world-backed start in this slice.

### Platform contract

This slice is Linux-first.

Rules:

1. public `--scope world` root start is supported on Linux,
2. non-Linux platforms fail closed with `unsupported_platform_or_posture`,
3. docs and tests must say this explicitly rather than implying unproven parity.

## Implementation Order

### Phase 1: Public input contract and resolver wiring

Goal:

1. add `--scope` to `AgentStartArgs`,
2. add `--disable-capability` and `--disable-cap` parsing,
3. thread that input plus omitted-scope resolution precedence into one start-envelope builder using the existing shared dispatch contract.

Why first:

1. later runtime work depends on stable caller input,
2. this phase defines the exact accepted and rejected public surface,
3. it localizes parser and resolver failures before runtime behavior changes.

Primary touch surface:

1. [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
2. [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
3. [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

Verification checkpoint:

1. parser tests and dispatch-contract tests cover accepted narrowing and fail-closed rejection,
2. host-scope root-start regressions still pass.

### Phase 2: Host-first start birth plus world-backed session setup

Goal:

1. replace host-only start planning with scope-aware resolution,
2. create a host-rooted attached durable session for the world-backed path,
3. persist authoritative host attach truth at birth,
4. establish authoritative world session/binding truth without turning the inaugural prompt into a world-first interaction.

Why second:

1. this is the core product behavior shift,
2. it must happen before any status or doc work can be truthful,
3. it is the highest-risk runtime seam and should be isolated before later world-dispatch wiring assumptions.

Primary touch surface:

1. `agents_cmd.rs`
2. `agent_runtime/control.rs`
3. `agent_runtime/orchestration_session.rs`

Verification checkpoint:

1. world-backed root start produces a durable host session record with persisted attach truth,
2. the new session is truthfully host-attached at birth,
3. authoritative world session/binding truth is persisted,
4. missing or invalid attach truth still fails closed.

### Phase 3: World binding/session persistence and later dispatch readiness

Goal:

1. persist authoritative `world_id` and `world_generation`,
2. make the world-backed path ready for later host-dispatched world work,
3. avoid inventing a second inaugural world-start dialect.

Why third:

1. once host-first birth is stable, world binding/session setup can attach to that frozen contract,
2. this keeps host session truth and later world-dispatch truth separable during implementation,
3. it minimizes cross-file conflicts until the final integration pass.

Primary touch surface:

1. `agent_runtime/control.rs`
2. `agent_runtime/session.rs`
3. `agent_runtime/state_store.rs`
4. `routing/dispatch/*`

Verification checkpoint:

1. Linux world-backed root start succeeds end to end,
2. authoritative world binding is persisted,
3. fail-closed behavior remains explicit on unsupported platforms or invalid world runtime state.

### Phase 4: Status truth, docs, and integration hardening

Goal:

1. preserve truthful host lifecycle semantics,
2. document omitted-scope resolution and `--scope host` bypass behavior,
3. update docs and end-to-end tests to match the new contract.

Why last:

1. status vocabulary should describe the final runtime behavior, not an intermediate implementation state,
2. docs should be written against the proven behavior,
3. the integration pass is the right place to replace the old host-only world-start rejection tests with the new contract wall.

Primary touch surface:

1. `agent_runtime/control.rs`
2. `agent_runtime/orchestration_session.rs`
3. `agent_runtime/state_store.rs`
4. `agent_public_control_surface_v1.rs`
5. `agent_successor_contract_ahcsitc0.rs`
6. `docs/USAGE.md`
7. `llm-last-mile/README.md`

Verification checkpoint:

1. `born_unattached` is visible and distinct from detached continuity states,
2. detached-world follow-up before host attach still fails closed,
3. docs and tests all describe the same public contract.

## Workstreams

### WS-A: CLI And Shared Contract

Scope:

1. add public start flags,
2. map them to the shared dispatch-envelope contract,
3. pin parser and resolver behavior.

Touch surface:

1. `crates/shell/src/execution/cli.rs`
2. `crates/shell/src/execution/agents_cmd.rs`
3. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`

Parallelization note:

This stream can start first and should freeze the exact caller-facing contract before other streams wire behavior behind it.

### WS-B: Runtime Birth And World Launch

Scope:

1. implement host-rooted world-start session birth,
2. persist attach truth,
3. launch the world member and persist world binding.

Touch surface:

1. `crates/shell/src/execution/agents_cmd.rs`
2. `crates/shell/src/execution/agent_runtime/control.rs`
3. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
4. `crates/shell/src/execution/agent_runtime/session.rs`
5. `crates/shell/src/execution/agent_runtime/state_store.rs`
6. `crates/shell/src/execution/routing/dispatch/*`

Parallelization note:

This stream depends on WS-A’s frozen public input contract.

### WS-C: Status Truth And Docs

Scope:

1. publish `born_unattached`,
2. preserve detached host semantics,
3. update docs and operator-visible wording.

Touch surface:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. `docs/USAGE.md`
5. `llm-last-mile/README.md`

Parallelization note:

This stream should begin after WS-B establishes the final runtime behavior for world-scoped start.

### WS-INT: Integration And Validation

Scope:

1. reconcile shared-file changes,
2. replace obsolete host-only world-start rejection coverage,
3. land end-to-end regression and documentation alignment.

Depends on:

1. WS-A
2. WS-B
3. WS-C

Touch surface:

1. `crates/shell/src/execution/agents_cmd.rs`
2. `crates/shell/tests/agent_public_control_surface_v1.rs`
3. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
4. `docs/USAGE.md`
5. `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`

## Risks And Mitigations

### Risk 1: Born-unattached state accidentally reuses detached continuity semantics

Why it matters:

1. it would lie to operators about prior attach history,
2. it would blur the boundary with slice 31.

Mitigation:

1. freeze `born_unattached` as the only approved status label in this slice,
2. add direct integration assertions that distinguish it from `parked_resumable` and `detached_reattachable`.

### Risk 2: Public capability flags imply a broader override model than the runtime supports

Why it matters:

1. it would misstate the agent capability boundary,
2. it would invite later contract drift.

Mitigation:

1. use explicit `--disable-capability` / `--disable-cap` spelling,
2. parse against a closed enum,
3. keep rejection tests for `session_start`, `llm`, and `mcp_client`.

### Risk 3: World-scoped root start drifts away from the shared dispatch contract

Why it matters:

1. it would create a second launch dialect,
2. it would undercut slices 29 and 29.75.

Mitigation:

1. require all new public input to map into `DispatchRequestEnvelope`,
2. keep provenance and rejection behavior in the shared resolver,
3. avoid start-only capability or scope logic outside that module.

### Risk 4: Non-Linux behavior accidentally appears partially supported

Why it matters:

1. it would create false parity promises,
2. it would complicate future support boundaries.

Mitigation:

1. freeze Linux-first in docs and tests,
2. keep explicit `unsupported_platform_or_posture` coverage on non-Linux world-scoped root start.

## Verification Wall

### Automated

Run, at minimum:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test --workspace -- --nocapture
```

### Manual

On Linux:

1. run host-scoped public root start and confirm current behavior is unchanged,
2. run `substrate agent start --scope world ... --json` and confirm:
   - durable session exists,
   - host attach truth is persisted,
   - host execution client is not attached,
   - world worker is launched,
   - status reports `born_unattached`,
3. confirm `substrate agent turn ...` fails closed before sanctioned host attach,
4. confirm the session remains `born_unattached` until a later sanctioned host-attach slice lands.

On non-Linux:

1. run `substrate agent start --scope world ... --json`,
2. confirm explicit `unsupported_platform_or_posture` failure.

## Exit Criteria

This plan is complete when:

1. the public start input contract is frozen in code,
2. host-scoped root start stays compatible,
3. world-scoped root start works on Linux with host-rooted authority and deferred host attach,
4. `born_unattached` is the truthful pre-attach operator state,
5. capability narrowing is explicit and bounded,
6. non-Linux world-scoped root start fails closed,
7. docs, tests, and runtime behavior all tell the same story.

## Not In Scope

This plan does not include:

1. lazy host attach trigger policy or continuity-vs-fresh attach semantics from slice 31,
2. broadening dispatch-time capability overrides beyond the five narrowing-only fields,
3. standalone world-root continuity,
4. macOS or Windows parity for public world-scoped root start,
5. a public inbox workflow or any other new public control surface.
