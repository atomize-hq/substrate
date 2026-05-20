# llm-and-agent-identity-tuple-and-deployment-posture — platform parity spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for Linux, macOS, and Windows parity guarantees for the ADR-0042 identity tuple and placement-posture semantics.
- This spec owns the cross-platform guarantee matrix, the allowed hidden divergence list, and the rule that `host_to_world_bridge` does not alter in-world `net_allowed` governance.
- This spec consumes the tuple, policy, and telemetry contracts from this pack. It does not redefine them.

Canonical references:
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`

## Contract boundary

Owned here:

- one operator-visible tuple and placement-posture meaning across Linux, macOS, and Windows
- the rule that `router=direct_provider_path` and `placement_posture.execution="host_only"` keep the same meaning on every platform
- the rule that `host_to_world_bridge` remains transport-only on every platform
- the rule that in-world `net_allowed` governance remains unchanged when bridge transport participates
- the hidden-divergence list for transport and bootstrap mechanics
- the validation evidence boundary for parity review

Not owned here:

- the operator command family or exit taxonomy from `docs/contracts/substrate-gateway-operator-contract.md`
- the `status --json` envelope or `client_wiring.*` field family from `docs/contracts/substrate-gateway-status-schema.md`
- routing-hint evaluation, tuple-axis constraint semantics, or host credential-read gates from `policy-spec.md`, `docs/contracts/substrate-gateway-policy-evaluation.md`, and ADR-0043
- the object names, field grammar, and omission rules from `identity-tuple-schema-spec.md`
- backend-id grammar or backend-selection realization from ADR-0027, ADR-0041, or ADR-0046

## Required platforms

- Behavior platforms: `linux`, `macos`
- Validation platforms: `linux`, `macos`, `windows`
- Windows parity includes the WSL-backed world path as hidden transport detail only. WSL is not a second operator-facing contract surface.
- Windows remains a required CI parity and review platform, but it is not a required feature-smoke platform for this pack.

## Cross-platform guarantee matrix

| Platform | Tuple and posture guarantee | Allowed hidden divergence | Verification anchor |
| --- | --- | --- | --- |
| Linux | `identity_tuple` and `placement_posture` keep the ADR-0042 meanings, token grammar, omission rules, and router/posture invariants unchanged. | Direct Unix socket transport, cgroup or namespace wiring, and Linux provisioning mechanics may differ under the hood. | `contract.md`, `identity-tuple-schema-spec.md`, `policy-spec.md`, `telemetry-spec.md`, `docs/contracts/substrate-gateway-runtime-parity.md` |
| macOS | `identity_tuple` and `placement_posture` keep the same meanings, token grammar, omission rules, and router/posture invariants as Linux. | Lima-backed guest transport, forwarding, and warm-flow mechanics may differ under the hood. | `contract.md`, `identity-tuple-schema-spec.md`, `policy-spec.md`, `telemetry-spec.md`, `docs/contracts/substrate-gateway-runtime-parity.md` |
| Windows | `identity_tuple` and `placement_posture` keep the same meanings, token grammar, omission rules, and router/posture invariants as Linux. | WSL-backed transport, named-pipe or TCP bridge mechanics, and Windows warm-flow mechanics may differ under the hood. | `contract.md`, `identity-tuple-schema-spec.md`, `policy-spec.md`, `telemetry-spec.md`, `docs/contracts/substrate-gateway-runtime-parity.md` |

## Shared invariants

- `identity_tuple` keeps the field names `client`, `router`, `provider`, `auth_authority`, and `protocol` on every platform.
- `placement_posture` keeps the field names `execution` and optional `host_to_world_bridge` on every platform.
- `router=direct_provider_path` requires `placement_posture.execution="host_only"` on every platform.
- `router=direct_provider_path` is invalid with `placement_posture.host_to_world_bridge=true` on every platform.
- `router=substrate_gateway` remains valid with `placement_posture.execution="in_world"` or `placement_posture.execution="host_only"` on every platform.
- `host_only` does not imply `direct_provider_path` on any platform.
- `host_to_world_bridge` is transport-only on every platform. It does not define router identity, provider authority, or a second control plane.
- In-world `net_allowed` governance remains owned by the existing policy/runtime surfaces and does not change when bridge transport participates.
- `backend_id` remains an adapter selector distinct from tuple semantics on every platform.
- Additive tuple publication outside `client_wiring.*` uses the same object names and omission rules on every platform.

## Allowed hidden divergences

- Linux may use direct Unix socket transport to `/run/substrate.sock`.
- macOS may use Lima-backed guest transport and forwarding.
- Windows may use the WSL-backed world path with named-pipe or TCP bridge transport.
- Platform-specific doctor flows, warm scripts, and provisioning helpers remain evidence surfaces only.
- Platform-specific managed artifact paths may differ, but those differences do not change tuple meaning, placement-posture meaning, or router/posture invariants.

## Validation evidence

Parity review for this pack consumes these surfaces:

- feature-local parity and compatibility surfaces:
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
  - `manual_testing_playbook.md`
- feature-local contract surfaces:
  - `contract.md`
  - `identity-tuple-schema-spec.md`
  - `policy-spec.md`
  - `telemetry-spec.md`
- durable external authorities:
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
- implementation and verification surfaces that execute against these guarantees:
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `crates/world-agent/src/gateway_runtime.rs`
  - `crates/world-agent/tests/gateway_runtime_parity.rs`
  - `scripts/mac/lima-doctor.sh`
  - `scripts/mac/smoke.sh`
  - Windows compile-parity and targeted test evidence captured by the checkpoint tasks

## Acceptance criteria

- Linux, macOS, and Windows publish one operator-visible tuple and placement-posture meaning.
- The router/posture invariants remain unchanged across all three platforms.
- `host_to_world_bridge` remains transport-only and leaves in-world `net_allowed` governance unchanged across all three platforms.
- Platform-specific transport mechanics stay hidden behind one parity contract and do not create a second operator-facing contract surface.
- The parity proof stays bounded to the surfaces named in this spec and does not widen status-schema, policy, or backend-selection ownership.
