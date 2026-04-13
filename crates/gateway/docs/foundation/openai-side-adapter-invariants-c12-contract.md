# OpenAI-Side Shared Adapter Invariants `C-12`

## Purpose

This note is the canonical landing artifact for `C-12`.
It freezes the shared invariants that all OpenAI-facing public endpoints must follow so `/v1/chat/completions` and `/v1/responses` remain **thin adapters** over the same normalized core.

`C-12` exists to prevent "endpoint-specific engines" and provider-specific streaming logic from creeping into public handlers.

## Canonical Sources Of Truth

- [ADR 0008](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md) (Shared Behavior + Adapter Boundaries sections).
- Core request/response types:
  - [core.rs](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/core.rs)
  - [models/mod.rs](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/models/mod.rs)

## Invariants

### Single Internal Execution Model

All OpenAI-facing public endpoints MUST:

- parse into the same internal `GatewayRequest` shape used by routing and providers
- emit responses from the same internal `GatewayResponse` and normalized stream model
- avoid introducing endpoint-only runtime behavior, parallel core models, or provider-conditional public semantics

### Tool Semantics Are Internal Content Blocks

Public OpenAI requests and responses MUST map to the internal tool representation:

- tool requests map to internal `tool_use` content blocks (see `KnownContentBlock::ToolUse`)
- tool results map to internal `tool_result` content blocks (see `KnownContentBlock::ToolResult`)

ID mapping rule:

- OpenAI `tool_call_id` / `call_id` MUST be preserved as the internal `tool_use.id`/`tool_use_id` so continuation can round-trip deterministically.

### Function Tools Only

OpenAI-side public ingress supports **function tools only**:

- built-in tools and non-function tool call types MUST be rejected with `400` and the gateway error envelope
- tool-call arguments MUST round-trip as JSON strings in the OpenAI-shaped public contract

### Chain-Of-Thought Suppression

Public OpenAI outputs MUST NOT expose chain-of-thought or provider "thinking" material as user-visible text.

- Internal `thinking` blocks (or equivalent provider-specific reasoning content) MUST be dropped from OpenAI-facing output transforms.
- A short reasoning summary MAY be exposed only if the upstream protocol provides one explicitly and it is safe to surface.

### Model Echo

The gateway MUST echo the original request `model` string in OpenAI-facing responses, even when routing maps to a different provider model internally.

### Provider Selection Override

Optional request header: `X-Provider: <provider_name>`

- when present: routing MUST constrain to that provider or fail routing
- when absent: routing MUST follow configured provider priority/fallback

### Error Envelope

All error responses MUST use the gateway error envelope:

```json
{
  "error": {
    "type": "error",
    "class": "auth|url|deployment|route|transport_drift",
    "message": "human-readable summary"
  }
}
```

Status code mapping:

- `400` for invalid request shapes, unsupported parameters, or routing failures
- `502` for provider failures after routing succeeds

### Streaming Boundary Discipline

Streaming adapters MUST be pure transforms over the provider-normalized stream:

- public OpenAI handlers MUST NOT parse provider-specific streaming framing
- when normalized stream output is insufficient for an OpenAI-facing transform, the fix must be:
  - extend/adjust the normalized stream model, or
  - tighten the OpenAI-facing contract deliberately

## Verification Checklist

`C-12` is complete only if a reviewer can answer yes to all of the following without reading provider parsing code:

- do `/v1/chat/completions` and `/v1/responses` both parse into `GatewayRequest` and emit from `GatewayResponse`/normalized streaming
- are tool requests/results represented as internal `tool_use`/`tool_result` content blocks with stable id mapping
- are non-function tools rejected at the public boundary, with no fallback behavior that silently widens scope
- do OpenAI-facing transforms suppress chain-of-thought content deterministically
- do streaming handlers avoid provider-specific public stream parsing and instead transform normalized output

