# Worktree Assignment Plan — Wave 1

Status: first implementation wave. These streams can run in parallel because they touch mostly different runtime boundaries.

## Objective

Land the next two highest-leverage slices in parallel:

- operator recovery and session-remediation ergonomics on the shell/control plane
- inventory-backed integrated gateway backend realization on the gateway/world-gateway path

## Streams

### Stream 4: Session selector and remediation ergonomics

- Worktree: `codex/sow-4-session-remediation`
- Goal: improve operator recovery without changing the public session handle contract
- Public handle remains `orchestration_session_id`

#### Owns

- [crates/shell/src/execution/agent_runtime/state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1)
- [crates/shell/src/execution/agent_runtime/control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1)
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1)
- [crates/shell/src/execution/cli.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs:1)
- [crates/shell/tests/agent_public_control_surface_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs:1)
- [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs:1)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:1)

#### Intended outcomes

- better current-session discovery
- clearer stale-owner and torn-state diagnostics
- bounded repair/reap/invalidate flows where appropriate
- no expansion of public handles beyond `orchestration_session_id`

#### Non-goals

- no tuple policy work
- no gateway adapter realization work
- no toolbox server implementation

### Stream 6: Inventory-backed integrated gateway backend realization

- Worktree: `codex/sow-6-gateway-backend-realization`
- Goal: land the ADR-0046 runtime slice and remove the hardcoded integrated `cli:codex` realization path while keeping Codex as the regression floor

#### Owns

- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1)
- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:1)
- [crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md:1)
- [crates/gateway/docs/contracts/chatgpt-codex-conformance-and-drift-guard.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/docs/contracts/chatgpt-codex-conformance-and-drift-guard.md:1)
- [crates/gateway/docs/contracts/chatgpt-codex-route-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/gateway/docs/contracts/chatgpt-codex-route-contract.md:1)
- [docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md:1)
- integrated gateway/world-agent tests

#### Intended outcomes

- adapter binding resolved by inventory/backend id
- explicit capability gating per integrated backend
- adapter-driven runtime config rendering
- backend-aware auth handoff
- at least one path beyond hardcoded `cli:codex`, without regressing Codex

#### Non-goals

- no tuple-axis policy
- no toolbox server
- no shell session-remediation UX

## Parallelism Rule

These two streams may proceed in parallel.

The intended boundary is:

- Stream 4 owns shell/operator recovery behavior
- Stream 6 owns gateway/world-gateway/backend-realization behavior

## Known Conflict Hotspots

- documentation overlap in [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:1) if Stream 6 needs operator docs
- possible shared references to backend-id wording in ADR/docs

When in doubt:

- Stream 4 owns user-facing session-control wording
- Stream 6 owns integrated gateway/backend wording

## Exit Criteria

- Stream 4 lands without changing backend realization semantics
- Stream 6 lands without inventing new public session-control semantics
- both merge cleanly and establish the dependency floor for Wave 2

