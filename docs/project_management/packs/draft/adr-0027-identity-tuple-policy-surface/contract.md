# adr-0027-identity-tuple-policy-surface — contract

This document is the operator-facing contract summary for ADR-0043.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Semantic owner: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Existing config/policy root: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- Schema owner for this feature: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- Exit-code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Decision lock

Accepted decisions for this contract lane:
- `DR-ITPS-01`: tuple-policy publication reuses the existing `identity_tuple` and `placement_posture` field family. This feature does not define a second trace-only tuple schema.
- `DR-ITPS-02`: `substrate policy current show --explain` is the authoritative merged inspection surface for `llm.constraints.*`. `substrate config ...` remains the config-root inspection surface and does not become the authoritative tuple-policy view.

## Scope

This feature is an additive extension of the existing ADR-0027 policy surface.

It does not introduce:
- a new config file family
- a new policy file family
- tuple-policy CLI flags
- tuple-policy environment variables
- a standalone `client` policy key

It does introduce the policy-key family owned by `tuple-policy-schema-spec.md`:
- `llm.constraints.routers`
- `llm.constraints.providers`
- `llm.constraints.protocols`
- `llm.constraints.auth_authorities`

## Source-of-truth files and precedence

Unchanged policy roots:
1. Workspace policy patch: `<workspace_root>/.substrate/policy.yaml`
2. Global policy patch: `$SUBSTRATE_HOME/policy.yaml`
3. Built-in defaults

Unchanged config roots:
1. Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`
2. Global config patch: `$SUBSTRATE_HOME/config.yaml`
3. Built-in defaults

Tuple-axis policy remains on the policy ladder above. This feature does not move any tuple-axis surface onto config, CLI, or environment-override ladders.

## Authoritative operator surfaces

### Effective policy inspection

The authoritative merged inspection command for tuple-axis policy is:
- `substrate policy current show --explain`

Rendered contract:
- `stdout` renders the effective merged policy.
- `stderr` renders the explain payload.
- `llm.constraints.routers`
- `llm.constraints.providers`
- `llm.constraints.protocols`
- `llm.constraints.auth_authorities`

When `--json --explain` is present:
- `stdout` is one JSON object containing the effective merged policy.
- `stderr` is one JSON object with `kind = "substrate.policy.explain.v1"`.
- The explain payload records per-key provenance under `/keys/<policy_key>/sources`.

Authoritative tuple-policy explain keys:
- `llm.constraints.routers`
- `llm.constraints.providers`
- `llm.constraints.protocols`
- `llm.constraints.auth_authorities`

`substrate config show --explain` remains valid for config-root inspection. It is not the authoritative merged view for `llm.constraints.*`.

### Gateway lifecycle enforcement surface

Tuple-axis policy denials surface through the existing gateway lifecycle commands:
- `substrate world gateway status`
- `substrate world gateway status --json`
- `substrate world gateway sync`
- `substrate world gateway restart`

The gateway lifecycle keeps the existing router identity:
- `router = substrate_gateway`

The gateway lifecycle derives the effective identity tuple and then enforces tuple-axis allowlists in this order:
1. `llm.constraints.routers`
2. `llm.constraints.protocols`
3. `llm.constraints.providers`
4. `llm.constraints.auth_authorities`

## Exact deny wording

Tuple-axis mismatch denials use exit code `5` and the gateway policy-failure wrapper.

Command wrapper line:
- `substrate world gateway <action>: policy or safety failure`

Cause line prefix:
- `substrate world gateway: gateway_policy_blocked:`

Exact mismatch-detail patterns:
- `effective gateway routing authority 'substrate_gateway' is not allowlisted by llm.constraints.routers`
- `effective gateway protocol '<protocol>' is not allowlisted by llm.constraints.protocols`
- `effective gateway provider '<provider>' is not allowlisted by llm.constraints.providers`
- `effective gateway auth authority '<auth_authority>' is not allowlisted by llm.constraints.auth_authorities`

Exact unresolved-detail pattern for constrained-but-missing auth authority:
- `effective gateway auth authority is unresolved while llm.constraints.auth_authorities is constrained`

The contract does not define a second deny wording family for the same condition.

## Exit codes

Exit-code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

- `0`: successful policy inspection or successful gateway lifecycle command
- `2`: invalid tuple-policy schema, invalid tuple-policy value grammar, unknown tuple-policy key, or invalid `policy ... set` update affecting `llm.constraints.*`
- `3`: unchanged dependency-unavailable behavior owned by the existing CLI contracts
- `4`: unchanged component-unavailable behavior owned by the existing gateway/world contracts
- `5`: tuple-axis policy denial on the gateway lifecycle surface

Feature-local mapping:
- tuple-policy schema invalidity maps to `2`
- tuple-axis mismatch denial maps to `5`

This feature introduces no new exit codes.

## Platform guarantee

Linux, macOS, and Windows expose the same tuple-axis policy semantics:
- the same four policy keys
- the same precedence posture
- the same authoritative inspection command
- the same exit-code mapping for schema invalidity and policy denial
- the same deny wording family for tuple-axis mismatch

Platform-specific transport differences do not change tuple-policy meaning.

## Unchanged owner boundaries

External owners that remain authoritative:
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - file families
  - precedence model
  - fail-closed baseline
  - backend allowlist posture
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - semantic meaning of `client`, `router`, `provider`, `protocol`, `auth_authority`
  - semantic meaning of `identity_tuple`, `placement_posture`, and `host_to_world_bridge`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - base trace envelope
  - correlation vocabulary

This contract reuses those owners. It does not redefine them.
