# substrate-gateway-boundary-and-runtime-ownership - platform parity spec

This document defines the feature-local contract surface for `C-04`.
It is intentionally narrow: it names the typed lifecycle/status runtime boundary, the Linux/macOS/Windows
parity guarantees, the allowed divergence list, and the verification surfaces that later slices must land.

## Contract boundary

Owned here:

- the typed lifecycle/status surface behind `substrate world gateway sync`, `status`, and `restart`
- the rule that shell/operator surfaces consume a typed runtime path rather than raw exec probing
- the Linux/macOS/Windows parity guarantees for lifecycle/status semantics
- the allowed divergence list for backend transport and bootstrap mechanics
- the named verification surfaces required to land and publish `C-04`

Not owned here:

- the operator command family or exit taxonomy already owned by `docs/contracts/substrate-gateway-operator-contract.md`
- the `status --json` envelope or `client_wiring.*` field family already owned by `docs/contracts/substrate-gateway-status-schema.md`
- fail-closed policy, secret delivery, or trust-boundary rules already owned by `docs/contracts/substrate-gateway-policy-evaluation.md`
- provisioning changes, warm-flow orchestration, or gateway-internal provider/planner/executor behavior

## Required platforms

- Behavior platforms (runtime parity required): `linux`, `macos`, `windows`
- Validation platforms (CI or smoke parity required): `linux`, `macos`, `windows`
- Windows runtime path: WSL-backed world transport remains part of the Windows parity guarantee rather than a second operator-facing contract

## Guarantees

- `substrate world gateway sync`, `status`, `status --json`, and `restart` preserve one operator-facing lifecycle/status meaning across Linux, macOS, and Windows.
- The authoritative runtime path is typed and world-backed; shell code must not recreate lifecycle/status truth from raw exec probes or backend-private state.
- Exit `4` remains the absent-state result when the required gateway/world component is unavailable.
- `status --json` remains the machine-readable wiring authority and keeps the `C-02` omission rules for `client_wiring.*`.
- Shell builtins, shared request/response models, and runtime client/server transport must consume the same `C-04` boundary.
- Provisioning remains out of scope for this pack; parity statements may describe runtime prerequisites but must not redefine install or warm-flow behavior as part of the operator contract.

## Permitted divergences

- Linux may use direct Unix socket transport to `/run/substrate.sock`.
- macOS may use Lima-backed forwarding to the guest `world-agent`.
- Windows may use the WSL-backed forwarder path, including named-pipe or TCP bridge transport as needed.
- Platform-specific doctor commands, forwarder diagnostics, or remediation hints may differ so long as the operator-facing lifecycle/status meaning stays unchanged.

## Known platform hazards

- The current shell builtin only exposes the unavailable posture, so later slices must resist drifting back toward shell-side probing when the typed runtime path lands.
- macOS and Windows transport warm-up/fallback mechanics differ from Linux socket activation and can accidentally leak backend-private behavior if parity is not kept explicit.
- `docs/WORLD.md` and install docs contain platform transport detail; later slices must keep those docs aligned without turning them into a second source of operator truth.

## Validation evidence

- **Doc publication surfaces**:
  - this feature-local parity spec
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Code surfaces that later slices must implement against**:
  - `crates/world-agent/src/lib.rs`
  - `crates/agent-api-types/src/lib.rs`
  - `crates/agent-api-client/src/lib.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
- **Tests to add or extend**:
  - extend `crates/shell/tests/world_gateway.rs` with `world_gateway_status_uses_typed_runtime_contract`
  - extend `crates/shell/tests/world_gateway.rs` with `world_gateway_status_json_preserves_client_wiring_absence_rules`
  - extend `crates/shell/tests/world_gateway.rs` with `world_gateway_sync_and_restart_follow_typed_lifecycle_contract`
  - add `crates/world-agent/tests/gateway_runtime_parity.rs` with `gateway_runtime_status_route_matches_socket_activation_transport`
  - add `crates/world-agent/tests/gateway_runtime_parity.rs` with `gateway_runtime_restart_route_preserves_component_unavailable_classification`

## Acceptance criteria

- The typed runtime authority boundary is explicit in both the feature-local parity spec and the durable contract mirror.
- Later slices have one concrete checklist for code loci, test loci, edge cases, and pass/fail conditions.
- Linux, macOS, and Windows transport differences remain hidden behind one operator-facing lifecycle/status contract.
- `status --json` keeps `C-02` authority and omission semantics while the runtime/parity contract stays explicit.
