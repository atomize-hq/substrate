# LACP0 Spec — Strict schema + agent inventory + restriction-only overlays (Phase 3)

This slice makes ADR-0027 execution-ready by landing the schema-first, strict, fail-closed surfaces for new `llm.*`, `agents.*`, and `workflow.router.*` keys, plus the agent inventory file format and restriction-only `policy_overlay` rules.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Contract: `docs/project_management/next/llm_and_agent_config_policy_surface/contract.md`
- Schema: `docs/project_management/next/llm_and_agent_config_policy_surface/SCHEMA.md`
- Decisions: `docs/project_management/next/llm_and_agent_config_policy_surface/decision_register.md`

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
- Agent inventory directory discovery and strict parsing for:
  - `$SUBSTRATE_HOME/agents/<agent_id>.yaml`
  - `<workspace_root>/.substrate/agents/<agent_id>.yaml`
- Restriction-only `policy_overlay` validation and composition rules per `SCHEMA.md`.
- `--explain` provenance output includes new keys in effective config/policy views.

## Out of scope (explicit)
- Implementing any LLM gateway, engine, agent hub, or orchestration behavior (those ADRs consume the surfaces defined here).
- Adding new config/policy file families or changing the existing precedence model.
- Defining provider-specific secret delivery mechanics (owned by gateway/engine ADRs; this slice only ensures the allowlist key paths exist and are strict).

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

### Agent inventory strictness (file format)
- Agent inventory files MUST be strict (unknown keys rejected).
- The filename-derived `<agent_id>` MUST equal the YAML `id:` field exactly; mismatch MUST be rejected (exit `2`).
- Agent files MUST NOT contain secrets; this slice enforces the “names-only” posture by schema (e.g., secret values must not appear in `config.api.auth.env`; those are names).

### Restriction-only `policy_overlay`
- If a `policy_overlay` is present, it MUST NOT broaden beyond base policy and MUST be validated per the composition rules in `SCHEMA.md`.
- Broadening attempts MUST be rejected (exit `2`) and MUST fail closed.

## Acceptance criteria

1) Config patch updates accept new keys:
   - `substrate config global set llm.enabled=true llm.gateway.enabled=true llm.routing.default_backend=cli:codex` exits `0`.
2) Policy patch updates accept new keys:
   - `substrate policy global set llm.allowed_backends+=cli:codex agents.allowed_backends+=cli:codex` exits `0`.
3) Unknown keys are rejected (exit `2`) for both config and policy patches.
4) `--explain` output includes the new key paths in both config and policy views.
5) Agent inventory strictness:
   - An agent file with unknown keys is rejected (exit `2`).
   - An agent file whose `id` does not match its filename is rejected (exit `2`).
6) Overlay broadening is rejected (exit `2`):
   - If base `llm.secrets.env_allowed` does not include a name and the overlay attempts to include it, agent load fails closed.
7) Cross-platform smoke passes using:
   - `docs/project_management/next/llm_and_agent_config_policy_surface/smoke/linux-smoke.sh`
   - `docs/project_management/next/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh`
   - `docs/project_management/next/llm_and_agent_config_policy_surface/smoke/windows-smoke.ps1`
