**Warning: Pre-Planning Only. Delete or retire this document during full planning.**

# agent-hub-core-successor-identity-tuple-compatible minimal spec draft

## Scope + authority

This draft defines only pack-level alignment rules for ADR-0044 work in `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/`.

Allowed scope:
- Cross-cutting defaults that multiple slice specs must share.
- Precedence rules already inherited from the current config and policy contract.
- High-level failure posture, security invariants, and naming rules.
- The draft slice skeleton that downstream pre-planning steps reference.

Disallowed scope:
- Slice-specific behavior details.
- Detailed protocol, schema, telemetry, policy, or compatibility tables.
- Implementation tasks, branch plans, `tasks.json` wiring, or code-level design.

Authoritative inputs for this draft:
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`

## Defaults + precedence

This feature adds no new config file family, no new env var family, and no feature-local precedence override.

Source-of-truth paths:
- Existing config and policy precedence: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- Feature-local contract surfaces to create during full planning:
  - `contract.md`
  - `agent-hub-session-protocol-spec.md`
  - `policy-spec.md`
  - `telemetry-spec.md`
  - `platform-parity-spec.md`
  - `compatibility-spec.md`

Effective precedence inherited by this feature:
1. CLI flags when an existing world-related flag applies.
2. Workspace config patch when inside an enabled workspace.
3. `SUBSTRATE_OVERRIDE_*` environment overrides when no enabled workspace exists.
4. Global config patch.
5. Built-in defaults.

Policy precedence inherited by this feature:
1. Workspace policy patch when inside an enabled workspace.
2. Global policy patch.
3. Built-in defaults.

Feature-local defaults that all slice specs must preserve:
- `agents.hub.orchestrator_agent_id` selects the orchestrator by agent inventory id.
- The selected orchestrator is host-scoped.
- `backend_id = "<kind>:<agent_id>"` remains the only agent-side allowlist and trace-attribution identifier.
- Pure agent runs expose `client`, `router`, `protocol`, and `backend_id`.
- Pure agent runs omit `provider` and `auth_authority`.
- Nested LLM requests publish a separate correlated record that carries `provider` and `auth_authority`.

## Failure posture + invariants

Failure posture:
- Control-plane decisions fail closed.
- Event-plane observation never authorizes a control-plane action.
- Invalid orchestrator state causes `substrate agent doctor` failure, not a degraded success path.
- Missing required world boundary, missing required session capability, denied backend selection, and invalid world-scoped orchestrator state terminate the control-plane path.

Security and redaction invariants:
- `backend_id` never stands in for `provider`, `auth_authority`, `router`, or `protocol`.
- Nested LLM approval remains on the gateway policy path; agent-hub approval does not grant nested gateway access by implication.
- Pure agent records do not synthesize provider or auth-authority fields.
- Secret-adjacent nested request metadata follows the shared telemetry redaction path and stays out of additive ad hoc fields.
- World reuse and drift restart visibility stays explicit through `world_id`, `world_generation`, and alert/event publication.

## Exit-code posture

Exit code taxonomy reference:
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

This pre-planning draft requires no new exit codes. Full planning keeps the canonical taxonomy unless a later spec records an explicit override in both the ADR and the feature-local contract surfaces.

## Cross-cutting seams / constraints

All slice specs and downstream planning artifacts align on these cross-cutting constraints:
- CLI namespace stays on `substrate agent ...` for the successor command family.
- `substrate agents validate` remains a compatibility leaf until compatibility planning closes its rollout decision.
- `backend_id` formatting stays stable as `<kind>:<agent_id>`.
- The pure-agent record and the nested gateway-backed LLM record stay separate artifacts.
- `router=agent_hub` identifies pure agent orchestration records.
- `router=substrate_gateway` identifies nested LLM records triggered by an agent.
- `protocol=substrate.agent.session` remains the pure-agent session protocol token unless the protocol spec records a narrower additive extension.
- Host-scoped orchestrator selection and world-scoped member dispatch remain distinct roles.
- `world_id` and `world_generation` remain operator-visible on world-scoped member records.
- No slice creates a new config root, policy root, inventory family, or gateway ownership surface.

## Follow-ups for full planning

- Resolve the exact compatibility timeline and operator wording for `substrate agents validate` versus the `substrate agent ...` command family, then pin that decision in `compatibility-spec.md` and `contract.md`.
- Pin the exact publication path for `world_generation` across status output, event envelopes, and trace spans in `telemetry-spec.md`.
- Pin the exact capability descriptor and session-handle schema in `agent-hub-session-protocol-spec.md`.
- Pin the exact fail-closed deny cases and doctor output wording in `policy-spec.md` and `contract.md`.
- Confirm the exact existing-crate ownership split across `crates/shell`, `crates/common`, `crates/trace`, and `crates/agent-api-*`, then mirror that split in slice specs and `tasks.json`.

## Draft slice skeleton (pre-planning only)

Draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): `AHCSITC`

CI-checkpoint must prefer this slice list when populating the machine-readable slices list in `pre-planning/ci_checkpoint_plan.md`.

Workstream triage may propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md` and leave this file unchanged.

### Slice entries

#### `AHCSITC0`
- `slice_id`: `AHCSITC0`
- `name`: Lock operator contract surfaces
- `intent`: Stabilize the operator-visible successor contract for list, status, and doctor. Lock the pure-agent identity tuple presentation and the additive compatibility boundary for the plural validate command.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC0/AHCSITC0-spec.md`
  - `docs/USAGE.md`
  - `crates/shell/src/execution/cli.rs`
  - `crates/shell/src/execution/agents_cmd.rs`
  - `crates/shell/src/execution/agent_inventory.rs`

#### `AHCSITC1`
- `slice_id`: `AHCSITC1`
- `name`: Lock session protocol and placement rules
- `intent`: Stabilize capability discovery, session-handle lifecycle, host-scoped orchestrator selection, and world-scoped member dispatch. Lock shared-world reuse and restart semantics at the protocol boundary.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC1/AHCSITC1-spec.md`
  - `crates/transport-api-types/src/lib.rs`
  - `crates/transport-api-client/src/lib.rs`
  - `crates/transport-api-core/src/lib.rs`
  - `crates/shell/src/execution/agent_inventory.rs`

#### `AHCSITC2`
- `slice_id`: `AHCSITC2`
- `name`: Lock fail-closed policy and telemetry split
- `intent`: Stabilize the deny path for invalid orchestrator and member routing state. Lock the pure-agent versus nested-LLM publication split, including redaction and correlation requirements.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC2/AHCSITC2-spec.md`
  - `crates/common/src/agent_events.rs`
  - `crates/shell/src/execution/agent_events.rs`
  - `crates/shell/src/execution/routing/telemetry.rs`
  - `crates/trace/src/span.rs`

#### `AHCSITC3`
- `slice_id`: `AHCSITC3`
- `name`: Lock parity, compatibility, and validation closure
- `intent`: Stabilize Linux, macOS, and Windows operator-visible parity, retire ADR-0025 semantics, and close the manual validation and rollout boundary for the successor pack.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md`
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md`
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/slices/AHCSITC3/AHCSITC3-spec.md`
  - `docs/CONFIGURATION.md`
  - `docs/TRACE.md`
  - `crates/shell/tests/agents_validate.rs`
  - `crates/shell/tests/agent_hub_trace_persistence.rs`
  - `crates/shell/tests/repl_world_first_routing_v1.rs`
