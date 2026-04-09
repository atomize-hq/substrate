# llm_and_agent_config_policy_surface — plan

## Scope
- Feature directory: `docs/project_management/packs/active/llm_and_agent_config_policy_surface`
- Orchestration branch: `feat/llm-and-agent-config-policy-surface`
- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Spec ownership map: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/spec_manifest.md`
- Impact map: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/impact_map.md`

## Goal
- Land ADR-0027 Phase 3: a strict, fail-closed, operator-governed config/policy surface for LLM + agent features using existing patch files, plus an agent inventory directory model with restriction-only per-agent overlays.

## Guardrails (non-negotiable)
- Specs are the single source of truth; integration reconciles code/tests to the specs.
- Strict schema is required: unknown keys and invalid values are hard errors (exit `2`).
- Fail-closed defaults are required:
  - config disables features by default.
  - policy allowlists are deny-by-default (`[]` denies).
- No secrets in Substrate YAML:
  - config/policy patches and agent inventory files must not contain secret values.
  - allowlists contain names only.

## Deliverables (authoritative)
- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Decision Register: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/decision_register.md`
- Contract: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/contract.md`
- Schema: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/SCHEMA.md`
- Slice specs:
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP0-spec.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`
- Manual testing playbook: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/manual_testing_playbook.md`
- Smoke scripts: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/`
- Planning Pack:
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/tasks.json`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/kickoff_prompts/`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/session_log.md`
  - `docs/project_management/packs/active/llm_and_agent_config_policy_surface/quality_gate_report.md`

## Platforms (current scope)
- Behavior platforms required:
  - Linux
  - macOS
- CI parity platforms required:
  - Linux
  - macOS

## Triads (slices)
- LACP0: strict config/policy schema + dotted updates + `--explain` stability for new key families.
- LACP1: agent inventory directory parsing + restriction-only `policy_overlay` validation, with `substrate agents validate`.

