# TASKS-30: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Source spec: [SPEC-30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)  
Source plan: [PLAN-30.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: draft realigned to host-first product intent on 2026-05-27

## Execution Packets

This slice should be implemented as four separate `/incremental-implementation` sessions.

- Packet 1 implements Phase 1 only.
- Packet 2 implements Phase 2 only.
- Packet 3 implements Phase 3 only.
- Packet 4 implements Phase 4 only.

Do not start a later packet until the prior packet’s checkpoint is green.

## Packet 1: Public Input Contract And Resolver Wiring

Session goal:

1. add the approved public start flags,
2. map them plus omitted-scope resolution precedence into the shared dispatch-envelope contract,
3. pin fail-closed behavior for unsupported capability families.

### Tasks

- [ ] Task 1.1: Add the public `agent start` CLI surface for `--scope`, `--disable-capability`, and `--disable-cap`
  - Acceptance: `AgentStartArgs` accepts `--scope host|world`; `--disable-capability <capability>` is repeatable; `--disable-cap <capability>` is the only alias; unsupported capability names fail at parse time; omitting `--scope` routes through the documented workspace-local then global config/policy resolution path instead of hardcoding host.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

- [ ] Task 1.2: Map scope inputs and omitted-scope resolution into one shared dispatch-envelope builder
  - Acceptance: public `agent start` builds `DispatchRequestEnvelope` from CLI inputs and resolved default scope instead of hardcoded host-only defaults; supported disable flags map to narrowing-only capability overrides; explicit `--scope host` bypass behavior stays unchanged.
  - Verify: `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 1.3: Pin fail-closed capability rejection for unsupported agent-level fields
  - Acceptance: public start rejects attempts to set or imply dispatch-time overrides for `session_start`, `llm`, and `mcp_client`; tests assert stable rejection classifiers and reasons.
  - Verify: `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the public CLI accepts the approved flags,
2. the shared dispatch envelope receives the new inputs and omitted-scope resolution truth,
3. unsupported capability families still fail closed,
4. explicit `--scope host` behavior is unchanged.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Host-First Start Birth And World Session Setup

Session goal:

1. preserve explicit host-scoped root start,
2. add scope-aware root-start planning,
3. create host-first world-backed session birth.

### Tasks

- [ ] Task 2.1: Refactor root-start planning so omitted scope resolves through config/policy and `--scope host` remains the bypass path
  - Acceptance: explicit `--scope host` still resolves through the existing public host prompt path, omitted scope honors the documented workspace-local then global config/policy resolution order, and host-scoped regressions remain green after the new flag surface lands.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

- [ ] Task 2.2: Implement host-first world-backed session birth
  - Acceptance: `agent start --scope world` creates a durable host-rooted orchestration session, persists authoritative `HostAttachContract` truth at birth, establishes authoritative world session/binding truth, and routes the inaugural operator prompt through the host orchestration agent instead of a first world-worker conversation.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. host-scoped root start remains compatible,
2. world-backed root start creates a host-rooted attached durable session,
3. authoritative host attach truth is persisted at birth,
4. authoritative world session/binding truth is established for the world-backed path.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: World Binding Persistence And Host Lifecycle Truth

Session goal:

1. persist authoritative world binding,
2. preserve normal host lifecycle semantics for the new default path,
3. avoid inventing a world-first inaugural prompt dialect.

### Tasks

- [ ] Task 3.1: Persist authoritative world binding for the world-backed start path
  - Acceptance: Linux world-backed root start persists `world_id` and `world_generation`, keeps that binding attached to the same host-rooted orchestration session, and does not invent a second inaugural world-launch dialect.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. Linux world-backed root start succeeds end to end,
2. authoritative `world_id` and `world_generation` are persisted,
3. the operator-facing lifecycle remains the normal host lifecycle rather than a `born_unattached` default.

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
  - Acceptance: Linux world-backed root start succeeds under the new contract; non-Linux world-backed root start fails closed with `unsupported_platform_or_posture`; omitted scope resolves through the documented workspace-local then global config/policy order; obsolete host-only world-start rejection assertions are replaced with the new contract wall.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

- [ ] Task 4.3: Update planning docs to match the shipped host-first world-backed contract
  - Acceptance: planning docs describe omitted-scope resolution order, `--scope host` as the bypass-world path, `--scope world` as the explicit world-backed host-session path, and host-first inaugural prompt handling exactly as implemented; no slice-30 doc still treats `born_unattached` as the default thin-slice success posture.
  - Verify: manual diff review plus `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
    - [`llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)
    - [`llm-last-mile/PLAN-30.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md)

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

- Packet 1 is the contract-freeze packet. Do not leak runtime behavior changes into it.
- Packet 2 is the highest-risk runtime packet. Keep it focused on host-first session birth and world session/binding setup.
- Packet 3 should stay narrow. If it expands into specialized born-unattached or lazy-attach policy, stop and defer that work to a later slice.
- Packet 4 is the integration packet. This is where obsolete host-only and born-unattached-default assertions should be replaced and where wording should be aligned across runtime, tests, and docs.
