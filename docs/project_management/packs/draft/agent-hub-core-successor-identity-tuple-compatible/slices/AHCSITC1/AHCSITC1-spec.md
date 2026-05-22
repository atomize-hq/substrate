# AHCSITC1-spec — orchestrator and member session protocol lock

## Behavior delta (single)
- Existing: `agent-hub-session-protocol-spec.md` defines the capability descriptor, session-handle object, lifecycle states, and host-versus-world placement rules, but the pack does not yet have an execution-ready slice that binds those protocol rules to one implementation boundary across `crates/agent-api-*` and the shell inventory and session surfaces.
- New: `AHCSITC1` becomes the authoritative session-protocol slice, fixing orchestrator eligibility, member dispatch semantics, shared-world reuse, replacement-handle rules after restart, and the files that carry that work.
- Why: fail-closed routing and telemetry publication depend on one frozen session model for handles, capabilities, scope, and world-generation movement.

## Scope
- Lock the capability descriptor and session-handle model that `substrate agent list --json` and `substrate agent status --json` project.
- Lock the host-scoped orchestrator rule and the world-scoped member dispatch rule.
- Lock the shared-world reuse boundary, including when `world_generation` increments and when replacement handles inherit prior orchestration identity.
- Lock the implementation boundary on `crates/transport-api-types`, `crates/transport-api-client`, `crates/transport-api-core`, and the matching `crates/shell/src/execution/agent_inventory.rs` integration surface.
- Keep command-surface wording in `AHCSITC0`, ordered deny flow and telemetry split in `AHCSITC2`, and parity or compatibility proof in `AHCSITC3`.

## Inputs (authoritative)
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/minimal_spec_draft.md`

## Behavior (authoritative)

### Protocol object and lifecycle lock
- `AHCSITC1` owns the first execution unit that materializes `AgentBackendCapabilityDescriptorV1`, `AgentSessionHandleV1`, lifecycle states, and status-object projection.
- The selected orchestrator remains host-scoped and must advertise the required `substrate.agent.session` capabilities before the hub opens work.
- Member handles inherit the parent `orchestration_session_id` and keep `backend_id`, `agent_id`, `role`, and `protocol` stable for the life of one handle.
- World-scoped member handles publish `world_id` and `world_generation`; host-scoped handles omit both.

### Shared-world reuse boundary
- One orchestration session reuses one shared world for all world-scoped members while the protocol inputs remain unchanged.
- A hub-driven restart invalidates the old world-scoped handle, allocates a replacement handle, preserves orchestration identity, and increments `world_generation` by exactly `1`.
- Replacement handles publish `resumed_from_session_handle_id` back to the invalidated handle.
- Host-scoped handles do not enter the restart path.

## Acceptance criteria
- AC-AHCSITC1-01: `AHCSITC1` locks the capability descriptor and session-handle objects that feed `substrate agent list --json` and `substrate agent status --json`, including stable `agent_id`, `backend_id`, `protocol`, and `execution.scope` projection.
- AC-AHCSITC1-02: Orchestrator eligibility in this slice requires a host-scoped inventory item that advertises the full `substrate.agent.session` capability set for start, resume, fork, stop, status snapshot, and event streaming.
- AC-AHCSITC1-03: Member dispatch in this slice keeps host-scoped members free of `world_id` and `world_generation` and requires both fields on world-scoped member handles.
- AC-AHCSITC1-04: Shared-world reuse remains one-world-per-`orchestration_session_id` for world-scoped members until restart criteria fire, and `world_generation` starts at `0` for the first shared-world allocation.
- AC-AHCSITC1-05: A hub-driven world restart invalidates the old world-scoped handle, allocates a replacement handle with a new handle id, preserves orchestration identity, and increments `world_generation` by exactly `1`.
- AC-AHCSITC1-06: The implementation boundary for this slice stays on `crates/transport-api-types`, `crates/transport-api-client`, `crates/transport-api-core`, and the shell inventory or status integration points instead of expanding into telemetry or parity surfaces.

## Out of scope
- Human-readable command wording and compatibility-leaf naming.
- Ordered deny explanations, gateway policy reuse, and top-level trace publication rules.
- Cross-platform parity proof, manual validation steps, and ADR-0025 supersession closure.
