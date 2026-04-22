# llm-and-agent-identity-tuple-and-deployment-posture — contract

This document is the pack-local operator-facing contract summary for ADR-0042.

Authoritative inputs:
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

## What this pack locks

- One operator-facing identity tuple:
  - `client`
  - `router`
  - `provider`
  - `auth_authority`
  - `protocol`
- One placement posture model:
  - `in_world`
  - `host_only`
  - `host_to_world_bridge`
- One downstream ownership split:
  - ADR-0043 may add policy keys for tuple-axis constraints.
  - ADR-0044 may define agent-hub behavior on top of the tuple.
  - Neither follow-on owns the base tuple vocabulary defined here.

## Identity tuple

- `client`
  - Meaning: the originating runtime or caller surface.
  - Examples: `codex`, `claude_code`

- `router`
  - Meaning: the routing authority that accepts the request and decides fulfillment.
  - Examples: `substrate_gateway`, later `agent_hub` for pure orchestration records

- `provider`
  - Meaning: the upstream service that actually fulfills a routed LLM request.
  - Examples: `openai`, `anthropic`, `azure_openai`

- `auth_authority`
  - Meaning: the credential or billing authority under which the request is authorized.
  - Examples: `codex_subscription`, `anthropic_api_key`, `gateway_delegated_secret`

- `protocol`
  - Meaning: the request/response contract or capability surface being spoken.
  - Examples: `openai.responses`, `anthropic.messages`, `uaa.agent.session`

## Canonical tokenization

- `client`, `router`, `provider`, and `auth_authority` use normalized lowercase snake_case ids.
- `protocol` uses a normalized lowercase dotted id, optionally with a version suffix.
- Human-readable prose may appear around these values, but operator-visible status, policy, and
  trace surfaces should use the normalized ids above.

## Placement posture

- `in_world`
  - Canonical fulfillment posture when world execution is required.

- `host_only`
  - A deployment mode for host-only or explicitly permitted host execution.
  - It is not a second permanent router.

- `host_to_world_bridge`
  - A transport bridge for host-scoped control surfaces reaching world-scoped resources.
  - It is not a router identity.
  - It is not a second control plane.

Non-negotiable interpretation:
- We do not run a second permanent host gateway alongside the in-world gateway.
- Host execution, when allowed, is a mode of the same routing authority rather than a peer router.
- Bridge transport may change reachability, but it must not change routing authority.

## Routing hints

- A routing hint is a request, not authority.
- The router validates the requested provider against policy and capability.
- A rejected hint does not change `client`.
- A rejected hint does not create implicit provider authority.

## Boundary rules

- `backend_id` remains an adapter/backend selector only in `<kind>:<name>` form.
- `backend_id` must not be overloaded to mean `client`, `router`, `provider`, `auth_authority`,
  or `protocol`.
- Tuple fields are operator-visible metadata, not replacements for ADR-0017/ADR-0028 correlation
  or join fields.
- Secrets must not appear in trace by default.
- `auth_authority` is distinct from both `client` and `provider`.

## Downstream dependency rules

- ADR-0043 consumes this contract for tuple meanings and only adds additive policy keys under
  `llm.constraints.*`.
- ADR-0044 consumes this contract for tuple meanings and only adds pure-agent versus nested-LLM
  behavior, including when `provider` or `auth_authority` are absent.

## Concrete examples

### Claude Code via `substrate_gateway`

- `client=claude_code`
- `router=substrate_gateway`
- `provider` varies by request or config
- `auth_authority` varies by approved credential path
- `protocol` reflects the surface actually spoken

Rule:
- the client remains `claude_code` even if provider or protocol changes.

### Codex using Responses API and `~/.codex/auth.json`

- `client=codex`
- `router=substrate_gateway` or another explicitly policy-permitted router
- `provider=openai`
- `auth_authority=codex_subscription` or another approved authority
- `protocol=openai.responses`

Rule:
- the protocol does not replace the client identity, and the credential source does not replace the
  provider identity.
