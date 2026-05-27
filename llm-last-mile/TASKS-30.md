# TASKS-30: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Source spec: [SPEC-30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md)  
Source plan: [PLAN-30.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: ready for implementation

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
2. map them into the shared dispatch-envelope contract,
3. pin fail-closed behavior for unsupported capability families.

### Tasks

- [ ] Task 1.1: Add the public `agent start` CLI surface for `--scope`, `--disable-capability`, and `--disable-cap`
  - Acceptance: `AgentStartArgs` accepts `--scope host|world`; `--disable-capability <capability>` is repeatable; `--disable-cap <capability>` is the only alias; unsupported capability names fail at parse time; omitting `--scope` preserves the host default.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

- [ ] Task 1.2: Map the new start flags into one shared dispatch-envelope builder
  - Acceptance: public `agent start` builds `DispatchRequestEnvelope` from CLI inputs instead of hardcoded host-only defaults; supported disable flags map to narrowing-only capability overrides; host default behavior stays unchanged.
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
2. the shared dispatch envelope receives the new inputs,
3. unsupported capability families still fail closed,
4. host-default root-start behavior is unchanged.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Host-Rooted World-Start Session Birth

Session goal:

1. preserve current host-scoped root start,
2. add scope-aware root-start planning,
3. create host-rooted world-start session birth with deferred host attach.

### Tasks

- [ ] Task 2.1: Refactor root-start planning so `--scope host` and omitted scope preserve current behavior
  - Acceptance: host-scoped root start still resolves through the existing public prompt path, keeps eager host execution-client startup where already required, and existing host-scoped regressions remain green after the new flag surface lands.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

- [ ] Task 2.2: Implement host-rooted world-start session birth with deferred host attach
  - Acceptance: `agent start --scope world` creates a durable host-rooted orchestration session, persists authoritative `HostAttachContract` truth at birth, and records deferred host execution-client startup rather than manufacturing an attached host owner.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. host-scoped root start remains compatible,
2. world-scoped root start creates a host-rooted durable session,
3. authoritative host attach truth is persisted at birth,
4. host execution-client startup is deferred for the world-scoped path.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: World-Member Launch And Binding Persistence

Session goal:

1. launch the world worker/member under the new host-rooted session,
2. persist authoritative world binding,
3. reuse existing world runtime semantics rather than inventing a start-only dialect.

### Tasks

- [ ] Task 3.1: Launch the world worker under the new session and persist authoritative world binding
  - Acceptance: Linux world-scoped root start launches the world member/worker through existing runtime plumbing, persists `world_id` and `world_generation`, and does not invent a second start-only world launch dialect.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. Linux world-scoped root start succeeds end to end,
2. authoritative `world_id` and `world_generation` are persisted,
3. the implementation still reuses the shared-contract world runtime path.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Status Truth, Control Hardening, Docs, And Final Validation

Session goal:

1. expose `born_unattached`,
2. preserve detached host semantics,
3. pin Linux-first and pre-attach fail-closed behavior,
4. align docs with shipped behavior,
5. run the full validation wall.

### Tasks

- [ ] Task 4.1: Add truthful `born_unattached` status and preserve existing detached host semantics
  - Acceptance: a never-attached host-rooted world-start session surfaces `born_unattached`; existing `parked_resumable`, `awaiting_attention`, and detached continuity semantics for previously attached host sessions remain unchanged; status JSON and human-readable output use the frozen vocabulary.
  - Verify: `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

- [ ] Task 4.2: Pin Linux-first and pre-attach fail-closed behavior in the public control suite
  - Acceptance: Linux world-scoped root start succeeds under the new contract; non-Linux world-scoped root start fails closed with `unsupported_platform_or_posture`; pre-attach world follow-up remains fail closed until sanctioned host attach; obsolete host-only world-start rejection assertions are replaced with the new contract wall.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)

- [ ] Task 4.3: Update operator docs and planning docs to match the shipped contract
  - Acceptance: public docs describe `--scope world`, `--disable-capability` / `--disable-cap`, `born_unattached`, Linux-first rollout, and pre-attach fail-closed behavior exactly as implemented; no doc still claims host-only root start as the only contract for this slice.
  - Verify: manual diff review plus `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
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

1. `born_unattached` is visible and truthful,
2. Linux-first and non-Linux fail-closed behavior are both pinned,
3. pre-attach follow-up remains fail closed,
4. docs, spec, and plan all match shipped behavior,
5. the full validation wall passes.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Notes For Implementation

- Packet 1 is the contract-freeze packet. Do not leak runtime behavior changes into it.
- Packet 2 is the highest-risk runtime packet. Keep it focused on session birth and deferred host attach.
- Packet 3 should stay narrow. If it expands into status or docs, stop and defer that work to Packet 4.
- Packet 4 is the integration packet. This is where obsolete host-only world-start assertions should be replaced and where wording should be aligned across runtime, tests, and docs.
