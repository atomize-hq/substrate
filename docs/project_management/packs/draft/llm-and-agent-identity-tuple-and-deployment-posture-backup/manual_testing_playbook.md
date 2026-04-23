# llm-and-agent-identity-tuple-and-deployment-posture — manual testing playbook

This pack is semantic and planning-only. Validation is a deterministic cross-document review.

## Validation checklist

1. Verify ADR-0042 and this pack’s `contract.md` agree on the meanings of:
   - `client`
   - `router`
   - `provider`
   - `auth_authority`
   - `protocol`

2. Verify ADR-0042 points at the current config-policy authorities:
   - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
   - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`

3. Verify ADR-0043 points at this pack for tuple semantics and does not redefine the base meanings
   of tuple fields or placement posture.

4. Verify `host_to_world_bridge` is described only as transport:
   - not a router
   - not a second control plane
   - not a second permanent gateway

5. Verify every example keeps `backend_id` separate from tuple fields.

6. Verify tuple-compatible agent event docs remain aligned with this pack’s boundary:
   - `backend_id` stays adapter-only
   - tuple metadata is additive when present

## Pass condition

- ADR-0042 has current authority links.
- The ADR-0042 pack exists and provides one pack-local contract for downstream consumption.
- ADR-0043 clearly consumes tuple semantics from the ADR-0042 pack rather than restating them as
  its own authoritative source.

## Fail condition

- stale `packs/active/...` references remain in ADR-0042 or ADR-0043
- ADR-0043 claims ownership of tuple meanings instead of policy keys only
- any doc describes `host_to_world_bridge` as a router or second control plane
