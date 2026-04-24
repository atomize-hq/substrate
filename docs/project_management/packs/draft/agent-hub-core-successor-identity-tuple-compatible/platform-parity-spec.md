# agent-hub-core-successor-identity-tuple-compatible — platform parity spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for Linux, macOS, and Windows parity guarantees for the ADR-0044 successor agent-hub operator surfaces.
- This spec owns the cross-platform guarantee matrix for `substrate agent list`, `substrate agent status`, `substrate agent doctor`, and the parity-visible event and trace projections those commands depend on.
- This spec owns the allowed hidden-divergence list and the exact operator-visible behavior when a required world boundary is unavailable.
- This spec consumes `contract.md`, `agent-hub-session-protocol-spec.md`, and `telemetry-spec.md`. It does not redefine their field grammar, routing semantics, or lifecycle rules.

Canonical references:
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md`
- `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/platform-parity-spec.md`
- `docs/INSTALLATION.md`
- `docs/WORLD.md`

## Contract boundary

Owned here:

- one operator-visible parity contract for the successor command namespace on Linux, macOS, and Windows
- the rule that `backend_id`, `execution.scope`, `role`, capability summary, `world_id`, and `world_generation` mean the same thing on every platform
- the rule that pure-agent records and nested gateway-backed LLM records remain separated the same way on every platform
- the rule that a required world-boundary failure stays fail-closed and does not degrade into synthetic host-only success on any platform
- the validation evidence boundary for parity signoff

Not owned here:

- CLI spelling, JSON field names, omission rules, or exit-code taxonomy beyond the parity-visible guarantees already fixed in `contract.md`
- capability-descriptor, session-handle, or lifecycle object grammar from `agent-hub-session-protocol-spec.md`
- ordered deny semantics, orchestrator eligibility checks, or fail-closed control-plane policy from `policy-spec.md`
- top-level event and trace field placement from `telemetry-spec.md`
- ADR-0025 supersession wording or `backend_id` migration wording from `compatibility-spec.md`

## Required platforms

- Behavior platforms: `linux`, `macos`, `windows`
- Validation platforms: `linux`, `macos`, `windows`
- Windows parity includes the WSL-backed world path as hidden transport detail only. WSL is not a second operator-facing contract surface.
- Every platform must preserve the host-scoped orchestrator requirement and the world-scoped member visibility contract.

## Cross-platform guarantee matrix

| Platform | `substrate agent list` guarantee | `substrate agent status` guarantee | `substrate agent doctor` guarantee | Event and trace guarantee |
| --- | --- | --- | --- | --- |
| Linux | Same canonical `substrate agent ...` namespace, same JSON keys, same `backend_id = "<kind>:<agent_id>"` derivation, same `execution.scope` and capability-summary meaning. | Same pure-agent versus nested-record split. World-scoped pure-agent rows render `world_id` and `world_generation`; host-scoped rows omit both. | Same ordered fail-closed posture. Exit `3` when a required world dependency is unavailable; exit `4` when the build or platform cannot satisfy the required posture. | Same pure-agent top-level fields, same nested-record omission rules, same `world_generation` publication path, and same ADR-0028 correlation family. |
| macOS | Same canonical `substrate agent ...` namespace, same JSON keys, same `backend_id = "<kind>:<agent_id>"` derivation, same `execution.scope` and capability-summary meaning. | Same pure-agent versus nested-record split. World-scoped pure-agent rows render `world_id` and `world_generation`; host-scoped rows omit both. | Same ordered fail-closed posture. Exit `3` when a required world dependency is unavailable; exit `4` when the build or platform cannot satisfy the required posture. | Same pure-agent top-level fields, same nested-record omission rules, same `world_generation` publication path, and same ADR-0028 correlation family. |
| Windows | Same canonical `substrate agent ...` namespace, same JSON keys, same `backend_id = "<kind>:<agent_id>"` derivation, same `execution.scope` and capability-summary meaning. | Same pure-agent versus nested-record split. World-scoped pure-agent rows render `world_id` and `world_generation`; host-scoped rows omit both. | Same ordered fail-closed posture. Exit `3` when a required world dependency is unavailable; exit `4` when the build or platform cannot satisfy the required posture. | Same pure-agent top-level fields, same nested-record omission rules, same `world_generation` publication path, and same ADR-0028 correlation family. |

## Shared invariants

- `substrate agent list`, `substrate agent status`, and `substrate agent doctor` stay on the same canonical namespace on every platform.
- `substrate agents validate` remains a compatibility leaf on every platform and never becomes an alias for list, status, or doctor.
- `backend_id` remains the adapter identifier on every platform and never becomes a proxy for `provider`, `auth_authority`, or `protocol`.
- The selected orchestrator remains host-scoped on every platform.
- World-scoped member sessions remain additive on every platform and never relax the host-scoped orchestrator requirement.
- `world_id` and `world_generation` appear only on world-scoped pure-agent session rows and on the world-scoped event and trace records defined by `telemetry-spec.md`.
- Nested gateway-backed LLM records always omit `world_id` and `world_generation` on every platform.
- A platform must not synthesize success by hiding a missing world dependency, demoting world-scoped work to host-only work, or omitting required failure details.

## Allowed hidden divergences

- Linux may realize world-scoped member execution through native namespaces, cgroups, overlayfs, nftables, and a local `substrate-world-agent` systemd socket.
- macOS may realize world-scoped member execution through a Lima guest, forwarding, and warm-flow orchestration.
- Windows may realize world-scoped member execution through the WSL-backed world path with named-pipe or TCP forwarding.
- Platform-specific warm scripts, doctor helpers, artifact paths, and service managers may differ.
- Platform-specific bootstrap timing, socket paths, and transport internals may differ.
- These hidden divergences must not alter command spelling, JSON field names, omission rules, tuple semantics, `backend_id` derivation, or fail-closed outcomes.

## World-boundary unavailability behavior

The following rules are mandatory on every platform:

- If the effective command path requires a world-scoped member posture and the required world boundary is temporarily unavailable, `substrate agent doctor` returns exit `3`.
- If the current build or platform cannot satisfy the required world posture at all, `substrate agent doctor` returns exit `4`.
- `substrate agent list` and `substrate agent status` do not invent host-scoped success rows to mask a required world-boundary failure.
- `substrate agent status` may report the last known live state, but it must not claim an active world-scoped session when the required world boundary has already failed closed.
- Pure-agent event and trace records keep the active `world_id` and `world_generation` contract when a world exists and do not backfill those fields onto host-scoped rows during failure handling.

Platform evidence examples:

- Linux: missing socket activation, unavailable namespace or overlay prerequisites, or broken world-agent bootstrap is a world-boundary unavailability case.
- macOS: missing or unhealthy Lima guest, forwarding breakage, or world-agent bootstrap failure inside the guest is a world-boundary unavailability case.
- Windows: missing or unhealthy WSL world path, broken named-pipe or TCP forwarding, or unavailable world-agent bootstrap inside WSL is a world-boundary unavailability case.

## Validation evidence

Parity review for this pack consumes these surfaces:

- feature-local parity and compatibility surfaces:
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
  - `manual_testing_playbook.md`
- feature-local contract surfaces:
  - `contract.md`
  - `agent-hub-session-protocol-spec.md`
  - `policy-spec.md`
  - `telemetry-spec.md`
- durable runtime and install authorities:
  - `docs/INSTALLATION.md`
  - `docs/WORLD.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- implementation and verification surfaces that must stay aligned with this parity contract:
  - `crates/shell/tests/agents_validate.rs`
  - `crates/shell/tests/agent_hub_trace_persistence.rs`
  - `crates/shell/tests/repl_world_first_routing_v1.rs`
  - `scripts/linux/world-provision.sh`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/smoke.sh`
  - `scripts/windows/wsl-warm.ps1`
  - `scripts/windows/wsl-smoke.ps1`

Required parity signoff evidence:

- Linux doctor evidence showing the canonical namespace and fail-closed world-boundary handling
- macOS doctor evidence showing the same namespace, field meanings, and fail-closed posture
- Windows doctor evidence showing the same namespace, field meanings, and fail-closed posture
- Cross-platform status evidence showing that world-scoped rows expose `world_id` and `world_generation` while host-scoped rows omit them
- Cross-platform telemetry or trace evidence showing that nested gateway-backed records omit `world_id` and `world_generation`

## Acceptance criteria

- Linux, macOS, and Windows publish one operator-visible meaning for `backend_id`, `execution.scope`, `role`, `world_id`, and `world_generation`.
- Linux, macOS, and Windows publish one operator-visible separation between pure-agent records and nested gateway-backed LLM records.
- Linux, macOS, and Windows preserve the same fail-closed `substrate agent doctor` outcomes for missing world boundaries and unsupported required posture.
- Hidden transport, provisioning, and bootstrap differences stay hidden behind one parity contract and do not create a second operator-facing contract surface.
- The parity proof stays bounded to the evidence surfaces named in this spec and does not reopen ownership already fixed by `contract.md`, `agent-hub-session-protocol-spec.md`, `policy-spec.md`, `telemetry-spec.md`, or `compatibility-spec.md`.
