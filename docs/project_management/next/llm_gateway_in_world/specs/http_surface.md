# spec — llm_gateway_in_world: HTTP surface (subset)

This spec describes the minimum HTTP surfaces exposed by the in-world gateway in v1. It is intentionally a subset and is capability-gated.

Authoritative ADR: `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md`

## Common requirements
- The gateway MUST reject unknown/unsupported endpoints clearly (e.g., `404`/`501`) and MUST emit a trace/event record for the denial.
- The gateway MUST NOT require real provider API keys; it MAY accept and ignore `Authorization` headers for compatibility.
- The gateway MUST enforce the effective policy gates before routing:
  - `llm.enabled` + `llm.gateway.enabled` (config)
  - `llm.allowed_backends` contains the selected backend id (policy)
  - `llm.fail_closed.routing` honored (policy)
  - `net_allowed` enforced at the world boundary for actual outbound egress

## OpenAI-compatible (subset)
Supported:
- `POST /v1/chat/completions`
  - Non-streaming: `stream=false|omitted` returns a single JSON response.
  - Streaming: `stream=true` returns an SSE stream (`text/event-stream`) with:
    - incremental `delta` events when backend supports true streaming
    - otherwise, behavior is defined by the CLI engine streaming decision (ADR-0024 DR-0002)

Optional (capability-gated):
- `GET /v1/models` (may be empty/minimal; intended for client “sanity checks” only)

## Anthropic-compatible (subset)
Supported:
- `POST /v1/messages`
  - Non-streaming: returns a single JSON response.
  - Streaming: returns an event stream when backend supports it; otherwise uses the same fallback semantics as above.

## Health
Supported:
- `GET /healthz` (or equivalent) for readiness checks (no policy evaluation beyond basic “running”).

