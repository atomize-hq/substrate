# plan — llm_gateway_in_world

Historical evidence only. This plan captures ADR-0023-era implementation intent and does not define the current operator boundary.
The live operator contract is `docs/contracts/gateway/operator-contract.md`.

Goal: Make ADR-0023 execution-ready by defining the minimal, explicit contracts required to implement an in-world LLM gateway without reshaping config/policy surfaces beyond ADR-0027.

Scope:
- Gateway “front door” HTTP surfaces (OpenAI-compat + Anthropic-compat; subset, capability-gated)
- Transport/bind strategy (in-world listener + host/guest access path)
- Policy gates and fail-closed semantics (ADR-0027 + existing `net_allowed`)
- Trace/event attribution requirements (ADR-0017 + Phase 8 ADR-0028 circle-back)

Out of scope:
- Enumerating a canonical backend registry list (defer to ADR-0024 + Phase 8 reference appendix)
- Provider-specific feature parity beyond the defined subset

Steps:
1. Lock bind/transport strategy in decision register.
2. Specify HTTP endpoints + streaming semantics (subset).
3. Specify request/response redaction + logging defaults.
4. Specify attribution fields and trace event family naming (with Phase 8 circle-back note).
5. Draft manual testing playbook + smoke expectations (later; once code exists).
