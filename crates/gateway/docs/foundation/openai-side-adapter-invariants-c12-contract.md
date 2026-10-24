# OpenAI-Side Shared Adapter Invariants `C-12`

## Purpose

This note is the canonical landing artifact for `C-12`.
It freezes the shared invariants that all OpenAI-facing public endpoints must follow so `/v1/chat/completions` and `/v1/responses` remain **thin adapters** over the same normalized core.

`C-12` exists to prevent "endpoint-specific engines" and provider-specific streaming logic from creeping into public handlers.

## Canonical Sources Of Truth

- [ADR 0008](../adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md) (Shared Behavior + Adapter Boundaries sections).
- Core request/response types:
  - [core.rs](../../src/core.rs)
  - [models/mod.rs](../../src/models/mod.rs)

## Invariants

### Single Internal Execution Model

All OpenAI-facing public endpoints MUST:

- parse into the same internal `GatewayRequest` shape used by routing and providers
- emit responses from the same internal `GatewayResponse` and normalized stream model
- avoid introducing endpoint-only runtime behavior, parallel core models, or uncontracted provider-conditional public semantics

### Tool Semantics Are Internal Content Blocks

Public OpenAI requests and responses MUST map to the internal tool representation:

- tool requests map to internal `tool_use` content blocks (see `KnownContentBlock::ToolUse`)
- tool results map to internal `tool_result` content blocks (see `KnownContentBlock::ToolResult`)

ID mapping rule:

- Supplied OpenAI `tool_call_id` / `call_id` MUST be preserved as the internal `tool_use.id`/`tool_use_id` so continuation can round-trip deterministically. Admitted client-search calls with absent optional IDs remain absent/inert and cannot anchor results; no ID is invented.

### Endpoint-owned typed tool contracts

Chat Completions retains function-only nested declarations and JSON-string arguments. Responses uses the [C-11 closed typed subset](openai-side-responses-c11-contract.md): flat function/custom/namespace/client-search declarations, loaded definitions, Lite additional tools, admitted history/opaque/hosted activity, and exact call/result identity and order. Function JSON Strings and custom raw Strings retain their distinct representations. No raw `Unknown` or metadata bypass is permitted.

Each serializer preserves its actual contracted protocol. Advanced unsupported combinations reject before provider I/O; routing does not strip state or downgrade model/auth/tool mode to make them fit. Anthropic signed-thinking/caching and Azure/Kimi/Gemini endpoint semantics remain owned by their functional serializers.

### Chain-Of-Thought Suppression

Public OpenAI outputs MUST NOT expose chain-of-thought or provider "thinking" material as user-visible text.

- Private plaintext reasoning, `Reasoning.content`, `reasoning_text`, and private metadata MUST NOT become public OpenAI text or diagnostics.
- Approved typed completed continuation, including exact opaque bytes and its required upstream `summary_text` Vec, may return on Responses to the same connected permitted caller under C-11. Empty and nonempty summaries retain their source shape; neither becomes generic text/log content.
- Include omission/empty/explicit and optional effort do not change that completed-item policy. Other public endpoint outputs do not acquire OpenAI opaque state; transport serde and diagnostic projections remain distinct.

### Model Echo

The gateway MUST echo the original request `model` string in OpenAI-facing responses, even when routing maps to a different provider model internally.

### Provider Selection Override

Optional request header: `X-Provider: <provider_name>`

- when present: routing MUST constrain to that provider or fail routing
- when absent: ordinary representable requests follow configured provider priority/fallback. Advanced Responses requests pin the selected backend/account/model per request and fail without fallback or stripping. Responses redirects fail safely rather than automatically forwarding.

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
- are each endpoint’s admitted typed tools preserved, with unsupported combinations rejected before I/O and no silent fallback, stripping, or `Unknown` bypass
- do OpenAI-facing transforms exclude private reasoning and diagnostics while preserving the approved typed Responses continuation/summary contract
- do streaming handlers avoid provider-specific public stream parsing and instead transform normalized output

### Instructions, cache, and terminal ownership

Supplied Responses instructions preserve missing/empty/nonempty and reject null/wrong types. Ordered roles and Lite placement stay intact; no bundled prompt fallback/overwrite or invented todo/warmup instruction is introduced. Typed source-native `prompt_cache_key` and `client_metadata` survive initial/subsequent/replayed requests, along with actual cache controls and usage when present. This is transport preservation, not evidence of cache effectiveness.

The internal `gateway.response_terminal` and `GatewayResponse.end_turn` preserve absent/null/false/true independently of stop reason (wrong types fail). Public Responses emits the optional completed member; other serializers consume the internal terminal. False cannot produce a misleading conversational `Final`; error/EOF/cancel/incomplete cannot fabricate success. Native owns follow-up and durable history.

`responses_caller` is serde-skipped adapter-origin bookkeeping, never caller metadata or authentication. Same-connected-caller transport introduces no new native-session/ciphertext attestation, persistence, key, or ledger. Existing managed access and credential ownership remain required; local component tests do not qualify standalone exposure.
