# adr-0027-identity-tuple-policy-surface — compatibility spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for the additive rollout posture of `llm.constraints.*`.
- This spec owns the no-change rule when the new keys are absent, the rule that backend ids remain adapter gates, and the promotion boundary into the implemented ADR-0027 pack.
- This spec does not redefine tuple semantics, backend-id grammar, policy ordering, or trace correlation vocabulary.

Canonical references:
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`

## Additive rollout rule

`llm.constraints.routers`, `llm.constraints.providers`, `llm.constraints.protocols`, and `llm.constraints.auth_authorities` extend the existing ADR-0027 policy surface.

They do not create:

- a second policy file family
- a second config root
- a second backend-selection system
- a second tuple publication family

## No-change behavior when keys are absent

When all four `llm.constraints.*` keys are absent or resolve to `[]`:

- backend selection still uses the existing inventory and `llm.allowed_backends`
- `substrate policy current show --explain` still renders the effective merged policy
- gateway lifecycle commands keep their existing availability and error-classification behavior
- `backend_id` keeps its existing meaning as adapter selector
- tuple-aware telemetry remains additive metadata only and does not change routing

Absent or empty tuple-axis keys are therefore compatibility-preserving and do not alter existing operator workflows.

## Backend-id boundary

`backend_id` remains:

- the adapter-selection token
- the inventory identity used by backend allowlists
- the correlation field used by existing status and trace consumers

`backend_id` does not become:

- `router`
- `provider`
- `protocol`
- `auth_authority`
- a surrogate for any missing tuple field

Compatibility rule:

- a route may be backend-allowed and still fail tuple-axis narrowing
- a tuple-axis allow does not widen a backend denied by `llm.allowed_backends`

## Promotion boundary into the implemented ADR-0027 pack

Promotion is complete only when the implemented ADR-0027 pack absorbs these additive surfaces:

- tuple-policy contract wording for `substrate policy current show --explain`
- schema tables for the four `llm.constraints.*` keys
- additive rollout wording that keeps policy files and precedence unchanged

Promotion does not move tuple semantics out of ADR-0042 or trace-envelope ownership out of ADR-0028.

## Future-extension invariants

Any later tuple-axis addition must preserve all of these invariants:

- the key lives under `llm.constraints.*`
- absence and `[]` remain compatibility-preserving
- backend ids remain adapter gates rather than tuple surrogates
- tuple-aware telemetry reuses `identity_tuple` and `placement_posture`
- `substrate policy current show --explain` remains the authoritative merged inspection surface for tuple-policy keys
- secret-bearing auth-source details remain redacted or omitted

## Out-of-scope compatibility moves

This feature does not:

- add `llm.constraints.clients`
- make `client` a policy key in v1
- redefine `host_to_world_bridge` as a policy override
- introduce config, CLI, or environment-variable overrides for tuple-axis keys
- split host-only routing into a second permanent gateway identity

## Verification anchors

- `contract.md`
- `tuple-policy-schema-spec.md`
- `policy-spec.md`
- `telemetry-spec.md`
- `manual_testing_playbook.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

## Acceptance criteria

- Operators that do not set `llm.constraints.*` observe no semantic change beyond additive visibility of the keys on policy surfaces.
- `backend_id` remains adapter identity only and never becomes a tuple surrogate.
- Promotion into the implemented ADR-0027 pack extends the existing policy system instead of creating a second one.
- Future tuple-axis additions must preserve the additive invariants defined here.
