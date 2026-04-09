# LACP0 Spec — Strict config/policy schema + dotted updates + explain (Phase 3a)

This slice makes ADR-0027 execution-ready by landing the schema-first, strict, fail-closed surfaces for new `llm.*`, `agents.*`, and `workflow.router.*` keys in config/policy patches, including dotted update support and stable `--explain` output.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Contract: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`
- Schema: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
- Decisions: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md`
- Additive follow-ons (not owned by this slice):
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Scope (in-slice)
- Strict schema acceptance for new config keys in:
  - `$SUBSTRATE_HOME/config.yaml`
  - `<workspace_root>/.substrate/workspace.yaml`
- Strict schema acceptance for new policy keys in:
  - `$SUBSTRATE_HOME/policy.yaml`
  - `<workspace_root>/.substrate/policy.yaml`
- Dotted update support (`key=value`, `key+=value`, `key-=value`) for the new keys via existing CLI surfaces:
  - `substrate config global|workspace set …`
  - `substrate policy global|workspace set …`
- `--explain` provenance output includes new keys in effective config/policy views.

## Out of scope (explicit)
- Implementing any LLM gateway, engine, agent hub, or orchestration behavior (those ADRs consume the surfaces defined here).
- Adding new config/policy file families or changing the existing precedence model.
- Agent inventory parsing and `policy_overlay` validation/composition (owned by `LACP1-spec.md`).
- Implementing tuple-axis policy keys under `llm.constraints.*`; that additive surface belongs to ADR-0043 and must not be silently absorbed by this slice.

## User-visible behavior (authoritative)

### Strict schema (fail closed)
- Unknown keys in config/policy patches MUST be rejected with exit code `2`.
- Invalid values/types for known keys MUST be rejected with exit code `2`.
- Accepted key paths MUST include (non-exhaustive; see `SCHEMA.md` for the full list):
  - Config: `llm.enabled`, `llm.gateway.enabled`, `llm.gateway.mode`, `llm.routing.default_backend`, `agents.enabled`, `agents.defaults.*`, `agents.hub.*`, `agents.toolbox.*`
  - Policy: `llm.fail_closed.routing`, `llm.allowed_backends`, `llm.secrets.env_allowed`, `agents.allowed_backends`, `agents.fail_closed.routing`, `agents.host_credentials.read.allowed_backends`, `workflow.router.*`

### Explain/provenance stability
- `substrate config current show --explain` MUST include the new `llm.*` and `agents.*` keys with correct provenance (global/workspace/env/default).
- `substrate policy current show --explain` MUST include the new `llm.*`, `agents.*`, and `workflow.router.*` keys with correct provenance (global/workspace/default).
- Explain output for this slice MUST NOT imply that `llm.routing.default_backend` alone determines the full operator-facing tuple from ADR-0042.

## Acceptance criteria

1) Config patch updates accept new keys:
   - `substrate config global set llm.enabled=true llm.gateway.enabled=true llm.routing.default_backend=cli:codex` exits `0`.
2) Policy patch updates accept new keys:
   - `substrate policy global set llm.allowed_backends+=cli:codex agents.allowed_backends+=cli:codex` exits `0`.
3) Unknown keys are rejected (exit `2`) for both config and policy patches.
4) `--explain` output includes the new key paths in both config and policy views.
