# llm-and-agent-identity-tuple-and-deployment-posture — impact map

## Inputs

- Feature directory:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- ADR:
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Spec manifest:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/spec_manifest.md`

## Touch set

### Create

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/README.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/plan.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/spec_manifest.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/impact_map.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/decision_register.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md`

### Edit

- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - fix stale config-policy pack refs
  - add pack-local contract/spec references
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
  - fix stale config-policy pack refs
  - add ADR-0042 pack references so tuple semantics are consumed rather than restated

### No code changes

- No crates
- No tests
- No CLI behavior
- No config schema mutations

## Drift and dependency scan

### ADR-0027 and implemented config-policy pack

- Overlap:
  - config/policy roots, fail-closed semantics, backend allowlists, and host-side secret-read gates
- Required resolution:
  - ADR-0042 must point at `packs/implemented/llm_and_agent_config_policy_surface/*`, not the stale `packs/active/*` paths.
- Risk if left unresolved:
  - downstream ADRs inherit broken cross-links and lose the actual config/policy authority set.

### ADR-0041

- Overlap:
  - stable backend identity and the rule that backend ids remain adapter selectors only
- Required resolution:
  - ADR-0042 must preserve `backend_id` as distinct from tuple fields.

### ADR-0043

- Overlap:
  - router/provider/protocol/auth-authority terminology
- Required resolution:
  - ADR-0043 should consume tuple meanings from this pack and only define the additive
    `llm.constraints.*` policy surface.
- Risk if left unresolved:
  - ADR-0043 becomes a second semantic owner for tuple fields.

### ADR-0044

- Overlap:
  - pure-agent versus nested-LLM identity stories
- Required resolution:
  - ADR-0044 may define presence and absence rules for pure-agent versus nested records, but it
    must not redefine base field meanings.

### Implemented agent-hub output routing pack

- Overlap:
  - tuple-compatible metadata can appear on structured events, while `backend_id` remains
    adapter-only
- Evidence:
  - `docs/project_management/packs/implemented/agent-hub-concurrent-execution-output-routing/contract.md`
- Required resolution:
  - keep tuple semantics here and keep output-routing semantics in the implemented pack.

## Follow-ons

- ADR-0043 is the next additive policy implementation lane.
- ADR-0044 is the next agent-hub successor semantics lane.
- This pack should remain semantic and planning-only.
