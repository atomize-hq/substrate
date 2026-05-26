# Tuple Constraint Policy Reference

This document is the stable operator-facing reference for ADR-0043, the additive tuple-axis policy
surface under `llm.constraints.*`.

Related references:
- `docs/reference/policy/contract.md`
- `docs/reference/policy/schema.md`
- `docs/contracts/gateway/policy-evaluation.md`

## Decision locks

- `substrate policy current show --explain` is the authoritative merged inspection surface for
  `llm.constraints.*`.
- Tuple-policy publication reuses the existing `identity_tuple` and `placement_posture` field
  family.
- This additive surface does not introduce a standalone `client` policy key in v1.

## Owned keys and defaults

This document owns only the additive tuple-axis schema:

| Key path | Type | Effective default | Valid token grammar | Meaning when non-empty |
| --- | --- | --- | --- | --- |
| `llm.constraints.routers` | `[string]` | `[]` | lowercase snake_case id | effective `router` must match one listed value |
| `llm.constraints.providers` | `[string]` | `[]` | lowercase snake_case id | effective `provider` must match one listed value |
| `llm.constraints.protocols` | `[string]` | `[]` | lowercase dotted id | effective `protocol` must match one listed value |
| `llm.constraints.auth_authorities` | `[string]` | `[]` | lowercase snake_case id | effective `auth_authority` must match one listed value |

Merge rules:
- workspace patch replaces the same global key
- omitted key inherits the next lower layer
- each tuple-axis key resolves independently

Effective meaning of `[]`:
- unconstrained on that axis

`client` is not a standalone policy key in v1.

The following key family is invalid for this feature:
- `llm.constraints.clients`

## Exact grammar

Snake-case ids apply to:
- `llm.constraints.routers[*]`
- `llm.constraints.providers[*]`
- `llm.constraints.auth_authorities[*]`

Accepted examples:
- `substrate_gateway`
- `openai`
- `azure_openai`
- `codex_subscription`
- `openai_api_key`

Rejected examples:
- `Substrate_Gateway`
- `openai-responses`
- `_openai`
- `openai__api`
- `openai_`

Validation error family:
- `invalid <key> entry '<value>'; expected lowercase snake_case id`

Dotted ids apply to:
- `llm.constraints.protocols[*]`

Accepted examples:
- `openai.responses`
- `openai.chat_completions`
- `anthropic.messages`
- `uaa.agent_session`

Rejected examples:
- `openai`
- `OpenAI.responses`
- `openai..responses`
- `openai.responses_v1.`
- `openai.responses-v1`

Validation error family:
- `invalid llm.constraints.protocols entry '<value>'; expected lowercase dotted id`

## Ordered runtime evaluation

Tuple-aware gateway policy evaluation follows this order:

1. Validate gateway lifecycle config.
2. Resolve the selected backend inventory entry and apply `llm.allowed_backends` before tuple
   derivation begins.
3. Derive the candidate identity tuple from the selected backend and integrated auth source.
4. Apply tuple-axis narrowing in this exact order:
   - `llm.constraints.routers`
   - `llm.constraints.protocols`
   - `llm.constraints.providers`
   - `llm.constraints.auth_authorities`
5. Resolve integrated auth source material:
   - blocked env auth is a policy denial
   - partial env auth is invalid integration
6. Apply world-boundary posture.
7. Apply downstream transport and egress gates.

If the selected backend id is absent from `llm.allowed_backends`, evaluation stops before
tuple-axis narrowing.

## Deny wording and failure buckets

Tuple-policy schema invalidity maps to `2`.
Tuple-axis mismatch denial maps to `5`.

Backend allowlist denial:
- `"<backend_id> is not allowlisted by effective policy llm.allowed_backends"`

Tuple-axis mismatch denials:
- `effective gateway routing authority 'substrate_gateway' is not allowlisted by llm.constraints.routers`
- `effective gateway protocol '<protocol>' is not allowlisted by llm.constraints.protocols`
- `effective gateway provider is unresolved while llm.constraints.providers is constrained`
- `effective gateway provider '<provider>' is not allowlisted by llm.constraints.providers`
- `effective gateway auth authority is unresolved while llm.constraints.auth_authorities is constrained`
- `effective gateway auth authority '<auth_authority>' is not allowlisted by llm.constraints.auth_authorities`

Component unavailable examples:
- the required world or gateway socket is missing

Transient runtime failure examples:
- connection refused
- timeout

Explain-surface rule:
- Explain output for tuple-aware denials must identify the exact policy key that denied the route.

## Platform guarantee

Linux, macOS, and Windows expose the same tuple-axis policy semantics:

- the same four policy keys
- the same precedence posture
- the same authoritative inspection command
- the same exit-code mapping for schema invalidity and policy denial
- the same deny wording family for tuple-axis mismatch
