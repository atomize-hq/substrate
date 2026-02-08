# Manual testing playbook — llm_gateway_in_world

Placeholder manual playbook for ADR-0023. Will be expanded once the gateway implementation exists.

Intended checks (v1):
- Fail-closed behavior when `llm.fail_closed.routing=true` and world is unavailable.
- Deny-by-default allowlist behavior when `llm.allowed_backends=[]`.
- `substrate llm env` outputs base URLs without leaking secrets.
- Gateway request emits trace/event record with correlation fields and `world_id` when in-world.

