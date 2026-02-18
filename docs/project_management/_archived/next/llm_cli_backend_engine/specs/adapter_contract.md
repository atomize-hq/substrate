# spec — llm_cli_backend_engine: adapter contract (v1)

This spec defines the adapter boundary for fulfilling canonical LLM requests via subscription-authenticated CLIs.

Authoritative ADR: `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md`

## Backend identity
- Backend ids are `<kind>:<name>`.
- For this engine, `<kind>` is `cli`.
- `cli:<agent_id>` MUST resolve via ADR-0027 agent inventory to an agent with:
  - `config.kind=cli`
  - `config.capabilities.llm=true`

## v1 conformance target
- v1 implementations MUST support `cli:codex` (Codex wrapper/adapter).
- Other `cli:*` backends are expected to be added later; the adapter interface MUST remain generic (no Codex-only fields in the canonical request/response shape).

## Input (canonical request)
The gateway normalizes provider-dialect requests into a canonical request object. The exact canonical schema is defined by the implementation, but MUST support (at minimum):
- a list of role-tagged messages (`system|user|assistant`) or equivalent
- a max output token budget (best-effort)
- a “stream requested” boolean

Provider-specific features (tools/function calling, images, etc.) MUST be capability-gated and may be rejected.

## Output (canonical response)
Adapters MUST produce:
- final text content (required)
- optional usage estimates (best-effort)
- optional structured events for streaming (best-effort; see DR-0002)

## Streaming
If a CLI cannot produce true incremental tokens:
- the engine’s fallback semantics are selected by `docs/project_management/_archived/next/llm_cli_backend_engine/decision_register.md` (DR-0002).

## Failure modes
Adapters MUST fail clearly with a structured error containing:
- backend id
- failure class: `binary_missing|unsupported_feature|backend_error|policy_denied|world_unavailable`
- user-safe message (no secrets)
