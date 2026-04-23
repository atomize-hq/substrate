# llm-and-agent-identity-tuple-and-deployment-posture — spec manifest

This file enumerates every semantic surface touched by ADR-0042 and assigns each surface to
exactly one authoritative document.

## Inputs

- Feature directory:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- ADR:
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Related/upstream authorities reused by this pack:
  - `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
  - `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/contract.md`

## Required pack documents

- `README.md`
  - Role: orientation
- `plan.md`
  - Role: scope and guardrails
- `spec_manifest.md`
  - Role: ownership map
- `contract.md`
  - Role: authoritative pack-local tuple and placement contract
- `impact_map.md`
  - Role: downstream dependency and drift scan
- `decision_register.md`
  - Role: A/B semantic decisions
- `manual_testing_playbook.md`
  - Role: deterministic cross-doc validation

This pack does not require execution slices, seam-planning docs, `tasks.json`, or code/test
implementation artifacts.

## Coverage matrix

| Surface | Authoritative doc | What is explicitly defined |
| --- | --- | --- |
| Tuple field semantics: `client`, `router`, `provider`, `auth_authority`, `protocol` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | field meanings, absence rules, and operator-visible boundaries |
| Canonical tokenization for tuple values | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | snake_case and dotted-id rules plus examples |
| Placement posture: `in_world`, `host_only`, `host_to_world_bridge` | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | posture meanings and the transport-only rule |
| Routing-hint semantics | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md` | request-versus-authority boundary and rejection behavior |
| Semantic ownership split across ADR-0042, ADR-0043, and ADR-0044 | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/spec_manifest.md` | what later ADRs may consume without redefining |
| Semantic A/B decisions | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/decision_register.md` | explicit selections and tradeoffs |
| Cross-doc drift and dependency scan | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/impact_map.md` | overlap with ADR-0027, ADR-0041, ADR-0043, ADR-0044, and agent event routing |
| Manual cross-doc validation | `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md` | checklist for semantic alignment and non-overlap |

## Downstream ownership rules

- ADR-0043 owns additive tuple-axis policy constraints under `llm.constraints.*`.
  - It must consume tuple meanings from this pack’s `contract.md`.
  - It must not redefine `client`, `router`, `provider`, `auth_authority`, `protocol`, or placement posture.
- ADR-0044 owns pure-agent versus nested-LLM identity behavior for the agent-hub successor.
  - It must consume base tuple meanings from this pack’s `contract.md`.
  - It may define where `provider` and `auth_authority` are absent or present, but not change what those fields mean.
