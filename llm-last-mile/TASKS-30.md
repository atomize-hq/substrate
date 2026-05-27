# TASKS-30: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Source spec: [SPEC-30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)  
Source plan: [PLAN-30.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: Packets 1-2 landed; narrowed for Packet 3+ work on 2026-05-27

## Execution Packets

This slice should be implemented as four separate `/incremental-implementation` sessions, but Packets 1-2 are already landed in code and now serve as the frozen floor for Packet 3.

- Packet 1 is landed and should not be reopened unless the contract changes.
- Packet 2 is landed and should not be reopened unless the contract changes.
- Packet 3 implements Phase 3 only.
- Packet 4 implements Phase 4 only.

Do not start Packet 3 until Packet 2’s checkpoint is green. Do not start Packet 4 until Packet 3’s checkpoint is green.

## Packet 1: Landed Public Input Contract And Resolver Wiring

Packet 1 is already landed in code. These tasks remain here only as frozen context for Packet 3 and later review.

Session goal:

1. add the approved public start flags,
2. map them plus omitted-scope resolution precedence into the shared dispatch-envelope contract,
3. pin fail-closed behavior for unsupported capability families.

### Tasks

- [x] Task 1.1: Add the public `agent start` CLI surface for `--scope`, `--disable-capability`, and `--disable-cap`
  - Acceptance: `AgentStartArgs` accepts `--scope host|world`; `--disable-capability <capability>` is repeatable; `--disable-cap <capability>` is the only alias; unsupported capability names fail at parse time; omitting `--scope` no longer hardcodes host.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

- [x] Task 1.2: Map scope inputs and omitted-scope resolution into one shared dispatch-envelope builder
  - Acceptance: public `agent start` builds `DispatchRequestEnvelope` from CLI inputs and resolved scope instead of hardcoded host-only defaults; omitted `--scope` resolves the preferred default scope, probes that scope first, falls back once to the alternate scope if needed, and stamps the resolved scope into the envelope; supported disable flags map to narrowing-only capability overrides; explicit `--scope host` bypass behavior stays unchanged.
  - Verify: `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 1.3: Pin fail-closed capability rejection for unsupported agent-level fields
  - Acceptance: public start rejects attempts to set or imply dispatch-time overrides for `session_start`, `llm`, and `mcp_client`; tests assert stable rejection classifiers and reasons.
  - Verify: `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the public CLI accepts the approved flags,
2. the shared dispatch envelope receives the new inputs and resolved-scope truth,
3. unsupported capability families still fail closed,
4. explicit `--scope host` behavior is unchanged.

Packet 3 should assume this checkpoint is already green.

## Packet 2: Host-First Start Birth And World Session Setup

Session goal:

1. preserve explicit host-scoped root start,
2. preserve the landed omitted-scope resolution contract,
3. replace the deferred-host-attach world-start success shape,
4. create host-first world-backed session birth.

### Tasks

- [x] Task 2.1: Refactor root-start planning so omitted scope resolves through config/policy and `--scope host` remains the bypass path
  - Acceptance: explicit `--scope host` still resolves through the existing public host prompt path; omitted `--scope` preserves the landed preferred-scope probe plus one alternate-scope fallback; the resolved scope remains authoritative for the request and operator-visible output; host-scoped regressions remain green after the new runtime work lands.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

- [x] Task 2.2: Implement host-first world-backed session birth
  - Acceptance: `agent start --scope world`, or omitted `--scope` that resolves to world, no longer returns the old deferred-host-attach `WorldBirth` / `born_unattached` success shape; it creates a durable host-rooted orchestration session that is already truthfully host-attached at return time, persists authoritative `HostAttachContract` truth at birth, establishes authoritative world session/binding truth before `start` returns, and routes the inaugural operator prompt through the host orchestration agent instead of a first world-worker conversation.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. host-scoped root start remains compatible,
2. world-backed root start creates a host-rooted attached durable session rather than a `born_unattached` success posture,
3. authoritative host attach truth is persisted at birth,
4. authoritative world session/binding truth is established for the world-backed path before `start` returns.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Canonical World Identity Reuse And Lazy Dispatch Readiness

Session goal:

1. treat Packet 2's persisted `world_id` and `world_generation` as the canonical durable projection of authoritative world session/binding truth,
2. require later host-decided world work to reuse that authoritative parent binding,
3. keep later world-worker allocation lazy until host orchestration chooses world work,
4. avoid inventing a world-first inaugural prompt dialect or reopening the Packet 2 start contract.

### Tasks

- [ ] Task 3.1: Reuse authoritative parent world binding for later world-member launch
  - Acceptance: later host-decided world-member launch treats the Packet-2 `world_id` and `world_generation` as the canonical parent binding for the orchestration session, reuses that same authoritative binding for member launch, and fails closed when the authoritative parent binding is missing or mismatched against the active world session.
  - Verify: `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)

- [ ] Task 3.2: Keep later world work lazy and preserve the Packet 2 host-first floor
  - Acceptance: Packet 3 does not introduce an eager first world-member conversation at public `start` return, does not revive `born_unattached` as the default happy path, and keeps later world work opt-in from host orchestration rather than background-triggered.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. later host-decided world work reuses the authoritative parent world binding established by Packet 2,
2. missing or mismatched authoritative world binding truth fails closed,
3. no eager world-member conversation or revived `born_unattached` default is introduced while wiring this readiness path.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Status Truth, Control Hardening, Docs, And Final Validation

Session goal:

1. preserve truthful host lifecycle semantics,
2. pin Linux-first and scope-resolution behavior,
3. align docs with shipped behavior,
4. run the full validation wall.

### Tasks

- [ ] Task 4.1: Preserve truthful host lifecycle/status semantics for the world-backed default path
  - Acceptance: world-backed root start uses the normal host lifecycle as the operator-facing happy path; existing `active_attached`, `parked_resumable`, and `awaiting_attention` semantics remain unchanged; no test or doc treats `born_unattached` as the default success posture for slice 30.
  - Verify: `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

- [ ] Task 4.2: Pin Linux-first world-backed start and scope-resolution behavior in the public control suite
  - Acceptance: Linux world-backed root start succeeds under the new contract; non-Linux world-backed root start fails closed with `unsupported_platform_or_posture`; omitted scope preserves the documented preferred-scope probe plus one alternate-scope fallback; obsolete deferred-host-attach / `born_unattached` root-start assertions are replaced with the new contract wall.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

- [ ] Task 4.3: Update planning docs to match the shipped host-first world-backed contract
  - Acceptance: planning docs describe omitted-scope preferred-scope resolution plus alternate-scope fallback, `--scope host` as the bypass-world path, `--scope world` as the explicit world-backed host-session path, host-first inaugural prompt handling, immediate start-time host/world truth versus lazy follow-on world work, and the explicit deferred list exactly as implemented; no slice-30 doc still treats `born_unattached` as the default thin-slice success posture.
  - Verify: manual diff review plus `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)
    - [`llm-last-mile/PLAN-30.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md)
    - [`llm-last-mile/TASKS-30.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-30.md)

- [ ] Task 4.4: Run the final validation wall for the full slice
  - Acceptance: formatting, clippy, targeted shell suites, and full workspace tests pass; any Linux manual smoke evidence needed for the slice is captured before closeout.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - No planned source edits; this is the validation gate after the implementation tasks above.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. Linux-first and non-Linux fail-closed behavior are both pinned,
2. omitted-scope resolution order is pinned,
3. docs, spec, and plan all match shipped behavior,
4. the full validation wall passes.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Notes For Implementation

- Packet 1 is already landed. Treat it as the contract floor for Packet 2 instead of reopening it.
- Packet 2 is landed floor. Do not reopen it while implementing Packet 3 unless the contract itself changes.
- Packet 3 should stay narrow. If it expands into specialized born-unattached policy, automatic attach triggers, or broad status-UI work, stop and defer that work to a later slice.
- Packet 4 is the integration packet. This is where obsolete deferred-host-attach assertions should be replaced and where wording should be aligned across runtime, tests, and llm-last-mile docs.
