# Manual testing playbook — llm_gateway_in_world

Historical evidence only. This placeholder manual playbook preserves ADR-0023-era checks and does not define the current operator boundary.
The live operator contract is `docs/contracts/gateway/operator-contract.md`.

Intended checks (v1):
- Fail-closed behavior when `llm.fail_closed.routing=true` and world is unavailable.
- Deny-by-default allowlist behavior when `llm.allowed_backends=[]`.
- Historical `substrate world status gateway` outputs base URLs without leaking secrets.
- Gateway request emits trace/event record with correlation fields and `world_id` when in-world.
