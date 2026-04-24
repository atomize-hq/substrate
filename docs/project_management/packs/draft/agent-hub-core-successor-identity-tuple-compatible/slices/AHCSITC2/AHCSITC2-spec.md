# AHCSITC2-spec — fail-closed policy and telemetry publication lock

## Behavior delta (single)
- Existing: `policy-spec.md` and `telemetry-spec.md` fix the ordered deny flow, nested gateway reuse boundary, pure-agent versus nested-record split, and world-restart alert publication, but the pack does not yet have an execution-ready slice that binds those decisions to one implementation boundary across the shell, common event helpers, and trace persistence.
- New: `AHCSITC2` becomes the authoritative fail-closed and telemetry slice, fixing ordered deny evaluation, pure-agent versus nested gateway-backed record publication, top-level field placement, and the files that implement and test that behavior.
- Why: the successor contract is not safe until control-plane denial and observability publication follow one deterministic implementation unit.

## Scope
- Lock the fail-closed evaluation path for orchestrator selection, member dispatch, required world-boundary availability, and drift handling.
- Lock the handoff boundary where nested LLM work leaves agent-hub control-plane approval and reuses gateway policy gates.
- Lock the pure-agent versus nested gateway-backed telemetry split, including top-level field placement for `client`, `router`, `protocol`, `backend_id`, `provider`, `auth_authority`, `world_id`, and `world_generation`.
- Lock the implementation boundary on `crates/common/src/agent_events.rs`, `crates/shell/src/execution/agent_events.rs`, `crates/shell/src/execution/routing/telemetry.rs`, `crates/trace/src/span.rs`, and the related trace or event tests.
- Keep command-surface wording in `AHCSITC0`, session protocol grammar in `AHCSITC1`, and parity or compatibility proof in `AHCSITC3`.

## Inputs (authoritative)
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`

## Behavior (authoritative)

### Fail-closed routing lock
- `AHCSITC2` owns the execution unit that applies `agents.allowed_backends`, capability checks, host-scoped orchestrator requirements, world-boundary requirements, and drift posture in one ordered flow.
- Agent-hub control-plane approval stops at the agent boundary and does not authorize nested gateway work by implication.
- Nested LLM requests reuse gateway policy gates after the hub completes the agent-side control-plane decision.
- Event-plane and trace-plane observation remain non-authoritative for control-plane approval.

### Telemetry and alert publication lock
- Pure-agent orchestration records publish `client`, `router = "agent_hub"`, `protocol = "uaa.agent.session"`, `backend_id`, and world fields only when the record is world-scoped.
- Nested gateway-backed records publish `client`, `router = "substrate_gateway"`, `protocol`, `provider`, and `auth_authority`, and omit `world_id` plus `world_generation`.
- Restart alerts publish the active replacement world at the top level and keep prior world values inside `data.previous_*`.
- Redaction for nested request metadata remains on the existing shared redaction path and emits normalized ids only.

## Acceptance criteria
- AC-AHCSITC2-01: `AHCSITC2` applies ordered fail-closed evaluation for orchestrator selection, member dispatch, required world-boundary availability, and shared-world drift before any nested gateway handoff begins.
- AC-AHCSITC2-02: Agent-side allowlist checks use only derived `backend_id` values, and event-plane or trace-plane records do not authorize orchestrator or member control-plane actions.
- AC-AHCSITC2-03: Nested LLM requests leave the agent-hub control plane after the agent-side decision and reuse gateway policy gates without inheriting approval by implication from a prior agent-hub success.
- AC-AHCSITC2-04: Pure-agent orchestration records publish `client`, `router = "agent_hub"`, `protocol = "uaa.agent.session"`, and omit `provider` plus `auth_authority`, while nested gateway-backed records publish `provider` plus `auth_authority` and omit `world_id` plus `world_generation`.
- AC-AHCSITC2-05: World-scoped steady-state records and restart alerts publish `world_generation` at the top level, while previous world values remain inside restart-alert `data.previous_*` fields.
- AC-AHCSITC2-06: The implementation boundary for this slice stays on common event helpers, shell telemetry plumbing, trace persistence, and related tests instead of reopening command wording, protocol grammar, or parity proof.

## Out of scope
- Canonical CLI namespace, human-readable render order, and compatibility-leaf wording.
- Capability-descriptor grammar, session-handle lifecycle naming, and replacement-handle object shape.
- Linux, macOS, and Windows parity signoff, ADR-0025 supersession wording, and manual validation-closeout reporting.
