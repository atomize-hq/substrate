# PLAN-30: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Source spec: [SPEC-30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)  
Adjacent landed slices: [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md)  
Follow-on slice: [31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Proposed branch: `feat/public-world-scoped-agent-start`  
Base branch: `main`  
Plan type: public caller-surface expansion with host-first world-backed delivery  
Status: draft narrowed for the Packet 3 runtime pass on 2026-05-27

## Objective

Ship a truthful public `substrate agent start` surface that starts a host orchestration session first and uses world as the default execution substrate when scope resolution selects it.

This slice is complete only when all of the following are true:

1. `substrate agent start` accepts explicit scope selection and bare `start` resolves a preferred default scope through workspace-local config/profile/policy first, then global config/policy, probes for an exact backend in that preferred scope, and falls back once to the alternate scope only if the preferred scope has no exact match.
2. The resolved scope from item 1 is the authoritative scope stamped into the request and reported back to the operator.
3. `substrate agent start --scope world`, or omitted `--scope` that resolves to world, creates a host-rooted durable orchestration session, persists authoritative host attach truth at session birth, and establishes world binding/session truth for later host-dispatched world work before `start` returns.
4. The same successful world-backed start is already truthfully host-attached at return time rather than surfacing a participant-less `born_unattached` success posture.
5. `substrate agent start --scope host` is the explicit bypass-world path.
6. Public dispatch-time capability narrowing is available only through:
   - `--disable-capability <capability>`
   - `--disable-cap <capability>`
7. The only supported narrowing targets remain:
   - `session_resume`
   - `session_fork`
   - `session_stop`
   - `status_snapshot`
   - `event_stream`
8. The inaugural operator prompt is handled by the host orchestration agent rather than being sent directly to a first world worker/member.
9. The default world-backed path uses the normal host lifecycle rather than `born_unattached` as the happy-path operator state.
10. Public world-scoped root start is Linux-first in this slice; non-Linux platforms fail closed with explicit posture guidance.

This is productization of a host-first orchestration model. It is not a world-first inaugural prompt model.

## Plan Summary

The repo already has the key ingredients:

1. public `agent start` and `agent turn` entrypoints in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs),
2. one shared dispatch-envelope contract in [`dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs),
3. landed Packet-1 caller surface for `--scope`, capability narrowing, and omitted-scope resolution in [`cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs) and [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs),
4. authoritative persisted host attach truth in [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs),
5. Linux world binding/session plumbing plus later world-member dispatch seams in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs),
6. integration suites that already pin most public control behavior in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) and [`agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs).

What is still missing is narrower:

1. Packet 1 and Packet 2 are now landed floor, but the slice docs still describe Packet 2 as future work instead of the current runtime truth.
2. Packet 3 ownership is blurry because top-level `world_id` and `world_generation` are already persisted at world-backed start time.
3. The remaining runtime contract to freeze is how later host-decided world work must reuse the authoritative parent world binding and fail closed on missing or mismatched truth.
4. Packet 3 still needs a cleaner boundary between runtime/readiness work and the broader status/doc hardening that belongs in Packet 4.

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
| Omitted-scope resolver floor | [`resolve_requested_start_scope(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1077) | Freeze the preferred-scope probe plus one alternate-scope fallback as intended Packet-1 behavior. |
| Current world-start planner | [`build_world_start_session_birth_plan(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1348) | Packet 2 landed the host-first world-backed start floor; Packet 3 should build on it rather than reopen it. |
| Public session posture vocabulary | [`PublicSessionPosture`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:103) | Preserve current host lifecycle semantics for the thin slice. |
| Durable orchestration posture vocabulary | [`OrchestrationSessionPosture`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:69) | Reuse current attached/detached host lifecycle truth; do not make `born_unattached` the default happy path. |
| Linux world-member dispatch path | [`submit_world_prompt_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1511) | Keep for later host-dispatched world work rather than inaugural prompt handling. |
| Existing old-model world-start coverage | [`public_root_start_world_scope_persists_deferred_host_attach_session()`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:4020) | Replace these deferred-host-attach assertions with the new host-first success contract. |

### Exact remaining gap

1. The repo has landed omitted-scope fallback plus host-first world-backed start behavior, but the slice docs do not yet treat that as the floor.
2. The remaining Packet 3 runtime question is not first-time world identity persistence, but how later host-decided world work must consume the authoritative parent world binding already established by Packet 2.
3. The fail-closed rules for missing or mismatched authoritative world binding truth are visible in runtime code, but not yet frozen as the Packet 3 contract.
4. Packet 3 and Packet 4 boundaries are still blurry around lifecycle/status hardening versus runtime/readiness work.

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

1. Omitting `--scope` resolves a preferred default scope through workspace-local config/profile/policy first, then global config/policy.
2. The runtime probes for an exact backend match in that preferred scope and falls back once to the alternate scope only if the preferred scope has no exact match.
3. The resolved scope after that probe/fallback sequence is the authoritative scope for the request and operator-visible output.
4. `--scope host` means explicit bypass-world host start.
5. `--scope world` means host-rooted durable session plus authoritative world session/binding setup, never standalone world-root continuity.
6. `--disable-capability` is canonical, `--disable-cap` is the only alias, and there is no single-letter short flag.
7. Public capability narrowing is dispatch-time narrowing only and cannot set or broaden baseline capability truth.

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
3. the orchestration session is already truthfully host-attached when `start` returns,
4. the inaugural operator prompt is handled by the host orchestration agent,
5. authoritative world binding/session truth is established so later host-dispatched world work has an authoritative substrate before `start` returns,
6. this slice does not add a second inaugural prompt or direct world-agent bootstrap conversation.

### Immediate versus lazy truth

Must exist before successful `start` return:

1. durable host-rooted orchestration session,
2. attached host owner/participant,
3. persisted host attach truth,
4. persisted authoritative world session/binding truth.

May remain lazy:

1. the first host-decided world dispatch after the inaugural prompt,
2. any later world worker/member conversation created by host orchestration policy.

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

Status: landed in Packet 1. Treat this as the frozen floor for Packet 3; do not reopen unless the contract changes.

Goal:

1. add `--scope` to `AgentStartArgs`,
2. add `--disable-capability` and `--disable-cap` parsing,
3. thread that input plus omitted-scope resolution precedence plus alternate-scope fallback into one start-envelope builder using the existing shared dispatch contract.

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
2. omitted-scope preferred-scope probe plus alternate-scope fallback are pinned,
3. host-scope root-start regressions still pass.

### Phase 2: Host-first start birth plus world-backed session setup

Status: landed in Packet 2. Treat this as the frozen floor for Packet 3; do not reopen unless the contract changes.

Goal:

1. replace host-only start planning with scope-aware resolution,
2. replace the current deferred-host-attach `WorldBirth` success shape,
3. create a host-rooted attached durable session for the world-backed path,
4. persist authoritative host attach truth at birth,
5. establish authoritative world session/binding truth without turning the inaugural prompt into a world-first interaction.

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
2. the new session is truthfully host-attached at birth rather than `born_unattached`,
3. authoritative world session/binding truth is persisted before `start` returns,
4. the success shape is no longer participant-less deferred attach,
5. missing or invalid attach truth still fails closed.

### Phase 3: Canonical world identity reuse and later dispatch readiness

Goal:

1. treat Packet 2's persisted `world_id` and `world_generation` as the canonical durable projection of authoritative world session/binding truth,
2. make the world-backed path ready for later host-dispatched world work without re-opening Packet 2's session-birth contract,
3. keep the first dispatched world worker/member lazy until the host actually chooses world work,
4. avoid inventing a second inaugural world-start dialect.

Why third:

1. Packet 2 already established the start-time world binding/session floor, so Packet 3 can focus on later reuse rather than first-time creation,
2. this keeps host session truth and later world-dispatch truth separable during implementation,
3. it minimizes cross-file conflicts until the final integration pass.

Primary touch surface:

1. `agent_runtime/control.rs`
2. `agent_runtime/session.rs`
3. `agent_runtime/state_store.rs`
4. `routing/dispatch/*`

Verification checkpoint:

1. later host-decided world work reuses the authoritative parent world binding established by Packet 2,
2. missing or mismatched authoritative world binding truth fails closed,
3. no eager world-member conversation or revived `born_unattached` happy path is introduced as part of this readiness work.

### Phase 4: Status truth, docs, and integration hardening

Goal:

1. preserve truthful host lifecycle semantics,
2. document omitted-scope resolution and `--scope host` bypass behavior,
3. update docs and end-to-end tests to match the new contract.

Why last:

1. status vocabulary should describe the final runtime behavior, not an intermediate implementation state,
2. docs should be written against the proven behavior,
3. the integration pass is the right place to replace the old deferred-host-attach world-start assertions with the new contract wall.

Primary touch surface:

1. `agent_runtime/control.rs`
2. `agent_runtime/orchestration_session.rs`
3. `agent_runtime/state_store.rs`
4. `agent_public_control_surface_v1.rs`
5. `agent_successor_contract_ahcsitc0.rs`
6. `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`
7. `llm-last-mile/PLAN-30.md`
8. `llm-last-mile/TASKS-30.md`

Verification checkpoint:

1. world-backed start reports the normal host lifecycle instead of `born_unattached`,
2. omitted-scope probe/fallback behavior is pinned in docs and tests,
3. docs and tests all describe the same Packet-2 public contract.

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
3. establish authoritative world substrate/binding truth without requiring an inaugural world-worker conversation.

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

1. preserve normal attached/detached host semantics for the new world-backed happy path,
2. replace obsolete deferred-host-attach assertions,
3. update llm-last-mile docs and operator-visible wording.

Touch surface:

1. `crates/shell/src/execution/agent_runtime/control.rs`
2. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`
5. `llm-last-mile/PLAN-30.md`
6. `llm-last-mile/TASKS-30.md`

Parallelization note:

This stream should begin after WS-B establishes the final runtime behavior for world-scoped start.

### WS-INT: Integration And Validation

Scope:

1. reconcile shared-file changes,
2. replace obsolete deferred-host-attach world-start coverage,
3. land end-to-end regression and documentation alignment.

Depends on:

1. WS-A
2. WS-B
3. WS-C

Touch surface:

1. `crates/shell/src/execution/agents_cmd.rs`
2. `crates/shell/tests/agent_public_control_surface_v1.rs`
3. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
4. `llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`
5. `llm-last-mile/PLAN-30.md`
6. `llm-last-mile/TASKS-30.md`

## Risks And Mitigations

### Risk 1: Packet 2 leaves the old deferred-host-attach success shape partially alive

Why it matters:

1. the repo would expose two contradictory meanings of successful world-backed start,
2. docs and tests would keep drifting from the intended host-first product model.

Mitigation:

1. replace `WorldBirth` / `born_unattached` root-start assertions in the integration wall,
2. require the Packet-2 happy path to return with normal host-attached lifecycle truth.

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
   - host execution client is already attached,
   - authoritative world binding is already persisted,
   - inaugural prompt is handled through the host path,
   - status reflects the normal host lifecycle rather than `born_unattached`,
3. confirm omitted `--scope` honors preferred-scope resolution plus one alternate-scope fallback,
4. confirm later world dispatch remains host-mediated rather than public world-first bootstrap behavior.

On non-Linux:

1. run `substrate agent start --scope world ... --json`,
2. confirm explicit `unsupported_platform_or_posture` failure.

## Exit Criteria

This plan is complete when:

1. the public start input contract is frozen in code,
2. host-scoped root start stays compatible,
3. omitted-scope preferred-scope probe plus alternate-scope fallback are pinned as intentional behavior,
4. world-scoped root start works on Linux with host-rooted authority, immediate host-attach truth, and authoritative world binding already in place,
5. capability narrowing is explicit and bounded,
6. non-Linux world-scoped root start fails closed,
7. docs, tests, and runtime behavior all tell the same story.

## Not In Scope

This plan does not include:

1. lazy host attach trigger policy or continuity-vs-fresh attach semantics from slice 31,
2. broadening dispatch-time capability overrides beyond the five narrowing-only fields,
3. any explicit world-first or `born_unattached` public start mode,
4. macOS or Windows parity for public world-scoped root start,
5. a public inbox workflow or any other new public control surface.
