# ChatGPT Codex Route Contract

This document is the descriptive source of truth for the gateway's ChatGPT Codex OAuth provider route.
It freezes the provider-side transport contract described by ADR 0010 without leaking planning IDs into the canonical artifact.

## Scope

This contract covers:

- the dedicated upstream endpoint for ChatGPT Codex OAuth traffic
- the minimal successful header contract
- the route-local `pass | translate | force | reject` compatibility matrix
- typed `message` and image-part translation rules
- flat function-tool and `tool_choice` rules
- semantic stream assembly, continuation legality, and sync-drain failure posture
- the rule that encrypted reasoning remains non-public on this route

This contract does not own:

- public ingress redesign for `/v1/messages`, `/v1/chat/completions`, or `/v1/responses`
- integrated auth-handoff ownership beyond consuming a resolved `account_id` and access token
- built-in tools, structured-output expansion, or unverified generic Responses fields

## Transport Contract

- Sync and streaming both target `https://chatgpt.com/backend-api/codex/responses`.
- The gateway always sends `stream = true`.
- The gateway always sends `store = false`.
- Sync callers are satisfied by draining and assembling the upstream SSE stream into a `GatewayResponse`.
- Streaming callers are satisfied by transforming the same upstream SSE stream into the normalized internal stream model.

### Minimal Header Contract

The only required outbound headers for this route are:

- `Authorization: Bearer <access_token>`
- `ChatGPT-Account-ID: <account_id>`
- `Content-Type: application/json`

The gateway omits `OpenAI-Beta`, `originator`, and browser-like parity headers on this route unless a later route-specific ADR revalidates them as required.

## Compatibility Matrix

### `force`

- `stream = true`
- `store = false`
- `text.format.type = "text"`

### `translate`

- bare message objects are accepted at ingress but are translated into typed `message` items before upstream submission
- supported image inputs are translated into upstream `image_url` content parts, with optional `detail`
- flat function-tool definitions are emitted in the Codex wire shape
- explicit function `tool_choice` is translated from the public OpenAI-side shape to the Codex-native flat shape
- sync requests are translated into stream-drain execution over the same upstream SSE transport used by streaming requests

### `pass`

- `reasoning.effort` with values `none`, `minimal`, `low`, `medium`, `high`, `xhigh`
- `reasoning.summary` with values `auto`, `concise`, `detailed`, `none`, subject to the reasoning-enabled predicate below
- `parallel_tool_calls`
- `text.verbosity` with values `low`, `medium`, `high`
- `include` as either `[]` or `["reasoning.encrypted_content"]`, subject to the reasoning-enabled predicate below
- `tool_choice = "none"`
- `tool_choice = "auto"`

### `reject`

- `max_output_tokens`
- `metadata`
- `truncation`
- `previous_response_id`
- `temperature`
- `top_p`
- `user`
- unverified `service_tier` overrides
- nested Chat Completions-style tool definitions
- nested Chat Completions-style `tool_choice`
- `tool_choice = "required"`
- `stream_options`

Unsupported caller-visible controls fail before the upstream call. The route does not silently strip, ignore, or degrade them.

## Message, Tool, And Continuation Rules

- The canonical outbound form is a typed `message` item: `{ "type": "message", "role": "...", "content": [...] }`.
- Supported image parts remain allowed only inside typed `message` items and become upstream `image_url` items.
- Function tools use the flat Responses shape: `{ "type": "function", "name": "...", "description": "...", "parameters": { ... } }`.
- Explicit function `tool_choice` uses the flat Codex shape: `{ "type": "function", "name": "..." }`.

### Continuation Rules

- Tool continuation uses flat Responses items:
  - `function_call`
  - `function_call_output`
- `function_call_output.call_id` must match the model-emitted tool call id exactly.
- A missing prior `function_call` may be synthesized only when the gateway already has authoritative normalized provenance for the same `call_id`.
- When synthesis is allowed, the synthesized `function_call` copies `name` and serialized arguments from that authoritative provenance.
- A synthesized `function_call` is inserted immediately before its matching `function_call_output`.
- Orphaned tool-result continuations without authoritative provenance reject before the upstream call.
- The gateway never emits duplicate `function_call` items for the same `call_id`.
- Normalized conversation-history order remains the primary ordering key for mixed continuation requests.

## Semantic Stream Assembly

The event stream is the source of truth for assembled output on this route.

The gateway assembles output from:

- `response.output_item.added`
- `response.output_item.done`
- `response.content_part.added`
- `response.content_part.done`
- `response.output_text.delta`
- `response.output_text.done`
- `response.function_call_arguments.delta`
- `response.function_call_arguments.done`

`response.completed` is terminal lifecycle and usage truth. It is not the assembled-answer source of truth.

### Sync-Drain Failure Rule

- Sync success requires terminal `response.completed`.
- If the upstream SSE stream is malformed, truncated, or terminates without `response.completed`, the gateway fails the sync call.
- Sync-drain failures use the normal gateway error envelope with `class = "transport_drift"` and status `502`.

## Reasoning Rules

- Reasoning is enabled only when `reasoning.effort` is present and not equal to `"none"`.
- Non-`none` `reasoning.summary` values are legal only when reasoning is enabled.
- `include = ["reasoning.encrypted_content"]` is legal only when reasoning is enabled.
- Encrypted reasoning items remain internal transport state.
- The gateway does not surface encrypted reasoning as public OpenAI-visible text, public message content, or public reasoning summaries on this route.

## Verification Anchors

Implementation and regression evidence for this contract should land against:

- `crates/gateway/src/providers/openai.rs`
- `crates/gateway/src/providers/streaming.rs`
- `crates/gateway/src/server/openai_responses.rs`
- `crates/gateway/src/models/mod.rs`
- `crates/gateway/tests/openai_responses_conformance.rs`
- `crates/gateway/tests/openai_shared_parity.rs`
- `crates/gateway/tests/fixtures/openai_responses/codex-*.json`

Verification must prove:

- sync/stream endpoint parity for Codex OAuth
- minimal-header behavior
- accepted versus rejected request controls
- typed-message and image-part translation
- flat tool and `tool_choice` shaping
- semantic event assembly for text, tool, and mixed streams
- deterministic continuation legality and ordering
- `502 transport_drift` on malformed or truncated sync drains
- non-public reasoning behavior on the route
