# agent-hub-core-successor-identity-tuple-compatible — compatibility spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for the ADR-0025 supersession boundary, the additive rollout posture for the successor `substrate agent ...` command family, and the compatibility guarantee that existing agent-side `backend_id` values remain valid.
- This spec owns the migration rule from backend-id-centric wording to explicit agent identity-tuple wording for operator-facing docs and diagnostics.
- This spec does not redefine session-handle schema, telemetry field placement, policy evaluation order, or platform parity.

Canonical references:
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md`
- `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

## Contract boundary

Owned here:

- the exact semantic replacement boundary between ADR-0025 and ADR-0044
- the no-change guarantee for existing `agents.allowed_backends` entries and other agent-side allowlist usage keyed by derived `backend_id`
- the migration rule that operators stop reading `backend_id` as provider, auth authority, router, or protocol identity
- the rollout end state for `substrate agent list`, `substrate agent status`, `substrate agent doctor`, and the retained `substrate agents validate` compatibility leaf
- the invariants that downstream features, including ADR-0045 follow-on work, must preserve

Not owned here:

- CLI JSON shape, human-readable render rules, or exit codes already fixed by `contract.md`
- capability-descriptor and session-handle grammar from `agent-hub-session-protocol-spec.md`
- fail-closed routing, orchestrator eligibility, or deny semantics from `policy-spec.md`
- event or trace field placement from `telemetry-spec.md`
- Linux, macOS, and Windows parity guarantees from `platform-parity-spec.md`
- config-root or policy-root ownership from the implemented config-policy pack

## Compatibility posture

- The canonical successor namespace is `substrate agent ...`.
- `substrate agent list`, `substrate agent status`, and `substrate agent doctor` are the only canonical operator-facing inventory, session-status, and doctor commands introduced by ADR-0044.
- `substrate agents validate` remains supported as an additive compatibility leaf for inventory validation only.
- `substrate agents validate` does not become an alias for `substrate agent list`, `substrate agent status`, or `substrate agent doctor`.
- Existing config roots, policy roots, inventory file families, and `agents.allowed_backends` semantics remain unchanged.
- Existing traces and diagnostics remain compatible because ADR-0044 adds explicit identity fields without changing the meaning of agent-side `backend_id`.

## ADR-0025 supersession boundary

ADR-0044 supersedes ADR-0025 semantically, with these exact consequences:

- ADR-0025 is historical evidence only for the first-generation agent-hub framing.
- ADR-0025 is not a live authority for role-swappable backend identity, backend-id-as-semantics wording, or future additive config-key wording.
- The compatible rules carried forward from ADR-0025 survive only where ADR-0044, `contract.md`, or `agent-hub-session-protocol-spec.md` restate them explicitly.
- Any active draft, implementation doc, or operator guidance that cites ADR-0025 as the live source for successor command, identity, or role semantics is a compatibility defect.

Replacement summary:

- old framing: backend id, role selection, transport, and semantic identity were described too close together
- new framing: `backend_id` remains the adapter identifier while `client`, `router`, `protocol`, `provider`, and `auth_authority` carry semantic identity where applicable
- old framing: ADR-0025 described any backend as an interchangeable orchestrator/member identity surface
- new framing: the orchestrator is host-scoped, members may be host-scoped or world-scoped, and role assignment is separate from `backend_id`

## No-change guarantee for existing backend allowlists

- Existing `agents.allowed_backends` entries remain valid without rewriting because `backend_id` stays derived as `<kind>:<agent_id>`.
- Existing config and policy files that already use `backend_id` strictly as the adapter-selection token remain correct.
- Existing allowlist decisions do not need to be re-authored to account for `client`, `router`, `protocol`, `provider`, or `auth_authority`.
- Existing trace joins and inventory status readers that consume agent-side `backend_id` as adapter identity remain valid.
- Compatibility is broken only when a reader or doc overloads `backend_id` with tuple meaning that belongs to other fields.

## Operator wording migration rules

- New operator-facing docs, planning specs, manual playbooks, and diagnostics must use `backend_id` only when they mean the adapter identifier or allowlist token.
- New operator-facing docs, planning specs, manual playbooks, and diagnostics must use explicit identity vocabulary when they mean:
  - `client`
  - `router`
  - `protocol`
  - `provider`
  - `auth_authority`
- Pure-agent records must be described as omitting `provider` and `auth_authority`.
- Nested gateway-backed LLM records must be described as the only records that add `provider` and `auth_authority`.
- World-scoped member visibility must be described through `world_id` and `world_generation`, not through backend-id-specific wording.
- New docs must describe role assignment separately from `backend_id`.

Canonical wording consequences:

- correct: "`backend_id` identifies the selected adapter"
- correct: "`client`, `router`, and `protocol` identify the pure-agent orchestration record"
- correct: "`provider` and `auth_authority` identify the nested gateway-backed LLM fulfillment record"
- incorrect: "`backend_id` tells the operator which provider or auth authority fulfilled the request"
- incorrect: "`backend_id` identifies whether the session is an orchestrator or a member"

## Rollout end state

The compatibility rollout is complete only when all of these conditions hold:

- active docs treat ADR-0044 as the live successor authority and treat ADR-0025 as historical evidence only
- active docs and diagnostics use `substrate agent list`, `substrate agent status`, and `substrate agent doctor` as the canonical namespace
- `substrate agents validate` remains documented only as the inventory-validation compatibility leaf
- active docs and diagnostics stop overloading `backend_id` with provider, auth-authority, router, or protocol meaning
- active docs and diagnostics describe host-scoped orchestrator selection and world-scoped member visibility without reverting to role-swappable-backend wording

## Invariants for downstream features

Later work, including ADR-0045 follow-on tooling, must preserve all of these invariants:

- `backend_id` remains the agent-side adapter identifier and allowlist token
- role assignment stays separate from `backend_id`
- pure-agent records keep `router=agent_hub` and `protocol=substrate.agent.session`
- pure-agent records omit `provider` and `auth_authority`
- nested gateway-backed LLM records carry `router=substrate_gateway` plus `provider` and `auth_authority`
- `world_id` and `world_generation` remain attached to world-scoped pure-agent records rather than nested records
- toolbox, trace, or diagnostics work must consume the successor `substrate agent ...` surfaces instead of reopening ADR-0025 semantics

## Historical evidence handling

- ADR-0025 may be cited only as superseded historical evidence.
- Archived packs and older notes may retain ADR-0025-era wording only when they are clearly labeled historical or archived context.
- Active docs that preserve ADR-0025 wording without a historical marker are compatibility defects.
- Historical wording never overrides `contract.md`, `agent-hub-session-protocol-spec.md`, `telemetry-spec.md`, `platform-parity-spec.md`, or this compatibility spec.

## Verification anchors

- `contract.md`
- `agent-hub-session-protocol-spec.md`
- `policy-spec.md`
- `telemetry-spec.md`
- `platform-parity-spec.md`
- `manual_testing_playbook.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

## Acceptance criteria

- ADR-0025 is explicitly treated as superseded historical evidence and not as a live successor contract.
- Existing `backend_id` allowlists remain valid without translation or tuple-aware rewriting.
- Active operator-facing docs and diagnostics stop overloading `backend_id` with provider, auth-authority, router, or protocol meaning.
- The canonical successor namespace is `substrate agent ...`, while `substrate agents validate` remains only the inventory-validation compatibility leaf.
- Downstream follow-on work, including ADR-0045-dependent surfaces, has one compatibility rule set to preserve and does not reopen backend-id-centric role semantics.
