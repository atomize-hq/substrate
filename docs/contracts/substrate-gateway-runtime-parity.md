# Substrate Gateway Runtime and Platform Parity

This document is the durable canonical contract reference for the typed lifecycle/status runtime
boundary and cross-platform parity owned by the Substrate gateway integration. It mirrors the
feature-local parity spec while staying distinct from the operator command contract, the
machine-readable status schema, and the policy-evaluation surface.

## Contract

The owned runtime/parity surface covers:

- the typed lifecycle/status control path behind `substrate world gateway sync`, `status`, and `restart`
- the authority boundary between shell/operator surfaces and the world/backend runtime
- the Linux/macOS/Windows parity guarantees for lifecycle/status semantics
- the allowed divergence list for backend transport and bootstrap mechanics

Concrete rules:

- Shell/operator entrypoints consume a typed runtime surface; they must not rebuild gateway state via raw exec probing, backend-private config files, or log scraping.
- `crates/shell/src/builtins/world_gateway.rs` and later shell execution wiring are consumers of that typed surface. They may render operator output and exit codes, but they do not own runtime truth.
- The typed runtime surface is owned by the world/backend boundary: `crates/world-agent`, shared request/response models in `crates/agent-api-types`, and transport helpers in `crates/agent-api-client`.
- `substrate world gateway status --json` remains governed by `docs/contracts/substrate-gateway-status-schema.md`; this contract may not widen the JSON field list or redefine `client_wiring.*`.
- Policy placement, fail-closed routing, secret delivery, and trust-boundary rules remain governed by `docs/contracts/substrate-gateway-policy-evaluation.md`.
- The operator command family and exit taxonomy remain governed by `docs/contracts/substrate-gateway-operator-contract.md`, including exit `4` for the required gateway/world component unavailable posture.
- Linux, macOS, and Windows must present one operator-facing lifecycle/status contract even when the underlying world transport differs.
- Platform transport and bootstrap mechanics may differ only in the hidden backend layer:
  - Linux uses direct Unix socket transport to `/run/substrate.sock`.
  - macOS uses Lima-backed forwarding to the guest `world-agent`.
  - Windows uses the WSL-backed forwarder path, with named-pipe or TCP bridge transport as needed.
- Platform-specific provisioning helpers, doctor flows, and socket/forwarder mechanics are verification evidence, not separate operator contracts.
- Base URLs discovered through `status --json` or the stable env exports remain intended for in-world reachability, not as a guarantee of direct host reachability.

## Boundaries

- This contract does not define the operator command family or exit codes beyond consuming them from the operator contract.
- This contract does not define `status --json` fields beyond consuming the published schema contract.
- This contract does not define provider, planner, or executor internals inside `substrate-gateway`.
- This contract does not pull provisioning behavior into gateway lifecycle/status ownership.
- This contract does not replace `docs/WORLD.md`, `docs/INSTALLATION.md`, or the Windows/macOS world setup guides as the source of transport and provisioning detail.

## Verification surfaces

The later execution slices must keep the runtime/parity contract aligned across these surfaces:

- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`
- `crates/world-agent/src/lib.rs`
- `crates/world-agent/tests/socket_activation.rs`
- `crates/agent-api-types/src/lib.rs`
- `crates/agent-api-client/src/lib.rs`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
- `docs/WORLD.md`
