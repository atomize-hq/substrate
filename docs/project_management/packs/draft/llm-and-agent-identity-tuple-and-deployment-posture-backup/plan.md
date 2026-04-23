# llm-and-agent-identity-tuple-and-deployment-posture — plan

## Scope
- Feature directory: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Spec ownership map: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/spec_manifest.md`
- Impact map: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/impact_map.md`

## Goal
- Lock one authoritative operator-facing identity model for LLM and agent work:
  - `client`
  - `router`
  - `provider`
  - `auth_authority`
  - `protocol`
- Lock one authoritative placement posture model:
  - `in_world`
  - `host_only`
  - `host_to_world_bridge`
- Provide one pack-local contract that downstream ADRs can consume without redefining tuple semantics.

## Guardrails
- `backend_id` remains an adapter/backend selector only and is not a substitute for tuple fields.
- `host_to_world_bridge` is transport-only and must not be described as a router or second control plane.
- ADR-0043 may add tuple-axis policy constraints, but it must not redefine tuple field meanings.
- ADR-0044 may define pure-agent versus nested-LLM records, but it must not redefine the base tuple vocabulary.
- This pack introduces no code changes, no CLI changes, and no new config roots.

## Deliverables
- ADR: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Pack README: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/README.md`
- Spec manifest: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/spec_manifest.md`
- Contract: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- Impact map: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/impact_map.md`
- Decision register: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/decision_register.md`
- Manual testing playbook: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md`

## Explicit non-deliverables
- No `tasks.json`
- No execution slices
- No code test plan
- No seam extraction

Reason:
- This pack is a semantic lock. Execution belongs to follow-on lanes such as ADR-0043 and ADR-0044.
