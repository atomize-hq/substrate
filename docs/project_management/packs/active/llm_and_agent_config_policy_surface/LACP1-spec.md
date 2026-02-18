# LACP1 Spec — Agent inventory strict parsing + restriction-only overlays (Phase 3b)

This slice completes ADR-0027 Phase 3 by implementing the agent inventory directory model and enforcing restriction-only `policy_overlay` validation, with an operator-visible validation command.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Contract: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`
- Schema: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
- Decisions: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md`

## Scope (in-slice)
- Agent inventory directory discovery and strict parsing for:
  - `$SUBSTRATE_HOME/agents/<agent_id>.yaml`
  - `<workspace_root>/.substrate/agents/<agent_id>.yaml`
- Restriction-only `policy_overlay` validation and composition rules per `SCHEMA.md`.
- Operator-visible validation CLI:
  - `substrate agents validate`
- Cross-platform smoke coverage for inventory + overlay (Linux + macOS only):
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/linux-smoke.sh`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh`

## Out of scope (explicit)
- Implementing any LLM gateway, engine, agent hub, orchestration toolbox, router daemon, or workflow behavior.
- Adding new config/policy file families or changing the existing precedence model.
- Windows execution for this Planning Pack.

## User-visible behavior (authoritative)

### Agent inventory strictness (file format)
- Agent inventory files MUST be strict (unknown keys rejected; exit `2`).
- The filename-derived `<agent_id>` MUST equal the YAML `id:` field exactly; mismatch MUST be rejected (exit `2`).
- Agent files MUST NOT contain secrets; the schema enforces a names-only posture where applicable (for example: `config.api.auth.env` contains env var names only).

### Restriction-only `policy_overlay`
- If an agent file contains `policy_overlay`, it MUST NOT broaden beyond effective base policy and MUST be validated per the composition rules in `SCHEMA.md`.
- Broadening attempts MUST be rejected (exit `2`) and MUST fail closed.

### `substrate agents validate` (new)
Command:
- `substrate agents validate`

Behavior:
- Discovers agent inventory roots:
  - Global root: `$SUBSTRATE_HOME/agents/`.
  - Workspace root: `<workspace_root>/.substrate/agents/` when the current directory resolves to an enabled workspace root.
- Validates every discovered `*.yaml` agent file under the discovered roots:
  - strict schema (unknown keys rejected),
  - filename `<agent_id>` equals YAML `id:`,
  - `policy_overlay` is restriction-only relative to the effective base policy for the current directory.
- Exit codes:
  - `0` when all discovered agent files are valid.
  - `2` when any discovered agent file is invalid for any reason above.
- Error output MUST include the failing file path and a single actionable reason per failure.

## Acceptance criteria

1) Valid agent inventory file passes validation:
   - Create `~/.substrate/agents/codex.yaml` with `id: codex` and a valid `config` section.
   - `substrate agents validate` exits `0`.
2) Agent file strictness:
   - An agent file with an unknown key is rejected (exit `2`).
3) Agent id mismatch:
   - `~/.substrate/agents/mismatch.yaml` with YAML `id: other` is rejected (exit `2`).
4) Overlay broadening is rejected (exit `2`):
   - If effective base policy has `llm.secrets.env_allowed=[]` and an agent overlay attempts `llm.secrets.env_allowed=["OPENAI_API_KEY"]`, validation fails closed.
5) Linux + macOS smoke passes:
   - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/linux-smoke.sh` exits `0`.
   - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh` exits `0`.

