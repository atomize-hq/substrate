# Policy Contract

This document is the stable operator-facing contract for the ADR-0027 config and policy surface.

Related references:
- `docs/reference/policy/schema.md`
- `docs/reference/policy/tuple_constraints.md`
- `docs/contracts/gateway/policy-evaluation.md`

## Authoritative file families

Substrate's LLM and agent surfaces stay on the existing layered config and policy files:

- Config:
  - Global: `$SUBSTRATE_HOME/config.yaml` (default `~/.substrate/config.yaml`)
  - Workspace: `<workspace_root>/.substrate/workspace.yaml`
- Policy:
  - Global: `$SUBSTRATE_HOME/policy.yaml` (default `~/.substrate/policy.yaml`)
  - Workspace: `<workspace_root>/.substrate/policy.yaml`
- Agent inventory:
  - Global: `$SUBSTRATE_HOME/agents/<agent_id>.yaml` (default `~/.substrate/agents/<agent_id>.yaml`)
  - Workspace: `<workspace_root>/.substrate/agents/<agent_id>.yaml`

This contract does not introduce a second config system or new root file family.

## Core invariants

- Unknown config or policy keys are hard errors.
- Invalid values for known keys are hard errors.
- Config and policy schema invalidity maps to exit code `2`.
- LLM and agent routing remain fail-closed by default.
- Backend allowlists remain deny-by-default:
  - `llm.allowed_backends=[]` denies LLM routing.
  - `agents.allowed_backends=[]` denies agent routing.
- Backend ids remain adapter selectors only. They must not be overloaded with router, provider,
  auth-authority, protocol, planner, executor, or wrapper identity.
- Per-agent `policy_overlay` remains restriction-only.
- Secrets must not be stored in Substrate YAML patches.
- Host credential reads remain explicitly policy-gated by
  `agents.host_credentials.read.allowed_backends`.
- Router daemon indirect execution remains explicitly policy-gated by `workflow.router.*`.

## Precedence

Config precedence remains unchanged and applies per key:
1. CLI flags for keys that explicitly support them
2. Workspace config patch
3. `SUBSTRATE_OVERRIDE_*` environment overrides when no workspace is active
4. Global config patch
5. Built-in defaults

Policy precedence remains unchanged and applies per key:
1. Workspace policy patch
2. Global policy patch
3. Built-in defaults

## Tuple-policy additive alignment

ADR-0027 remains the root contract for:

- config and policy file families
- precedence
- fail-closed posture
- backend allowlists
- host-side secret and credential-read gates

ADR-0042 remains the semantic owner of the operator-facing tuple fields:

- `client`
- `router`
- `provider`
- `auth_authority`
- `protocol`
- `identity_tuple`
- `placement_posture`

ADR-0043 extends the policy surface additively under `llm.constraints.*`.

- `substrate policy current show --explain` is the authoritative merged inspection surface for
  `llm.constraints.*`.
- Tuple-policy publication reuses the existing `identity_tuple` and `placement_posture` field
  family.
- Tuple-axis mismatch denial maps to exit code `5`.

The stable schema and runtime ownership for that additive surface lives in
`docs/reference/policy/tuple_constraints.md`.
