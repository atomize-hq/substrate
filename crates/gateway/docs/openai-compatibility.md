# OpenAI API Compatibility

This gateway now exposes two public OpenAI-shaped surfaces:

- `POST /v1/chat/completions`
- `POST /v1/responses`

Both are implemented as thin adapters over the same internal gateway core. They share routing, model mapping, provider fallback, reasoning suppression, and the same redacted error envelope.

`/v1/messages` remains the primary Claude Code-facing surface. The OpenAI routes exist for OpenAI-compatible clients and SDKs.

## Current Status

| Surface | Status | Notes |
| --- | --- | --- |
| `POST /v1/chat/completions` | Implemented | Supports sync + streaming, function tools, tool continuation, images, model echo, `X-Provider` forcing |
| `POST /v1/responses` | Implemented | Supports sync + streaming, function tools, `function_call_output` continuation, images, model echo, `X-Provider` forcing |
| OpenAI-side token counting | Not yet exposed | `POST /v1/messages/count_tokens` exists today; ADR 0009 sets direction for ingress-wide token counting, but no OpenAI count-tokens route has landed yet |

Repo anchors:

- [ADR 0008](../../docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md)
- [Chat Completions contract (`C-10`)](../../docs/foundation/openai-side-chat-completions-c10-contract.md)
- [Responses contract (`C-11`)](../../docs/foundation/openai-side-responses-c11-contract.md)
- [Shared adapter invariants (`C-12`)](../../docs/foundation/openai-side-adapter-invariants-c12-contract.md)
- [Conformance suite contract (`C-13`)](../../docs/foundation/openai-side-conformance-suite-c13-contract.md)

## Shared Behavior

These behaviors are shared across both OpenAI endpoints:

- `model` is a public gateway model name. The gateway may route to a different provider model internally, but the response echoes the original request `model`.
- Optional `X-Provider: <provider_name>` constrains routing to one configured provider. If the named provider is missing or unmapped for the requested model, the request fails with a route error.
- Routing still uses the configured model mappings and provider fallback order when `X-Provider` is not set.
- Public OpenAI output suppresses internal/provider reasoning blocks (`thinking`, hidden reasoning markers, etc.). Reasoning content is not surfaced as user-visible text.
- Unknown top-level request fields are ignored unless they are part of the gateway's explicit reject list for that endpoint.

## Error Envelope

All public OpenAI errors use the same redacted gateway envelope:

```json
{
  "error": {
    "type": "error",
    "class": "auth|url|deployment|route|transport_drift",
    "message": "human-readable summary"
  }
}
```

Status behavior:

- `400` for invalid request shape, unsupported public fields, invalid tool-loop continuation, or route selection failures
- `502` for provider failures after routing succeeds

The public `message` value is intentionally redacted to the stable gateway summary, not the raw upstream/provider error text.

## `POST /v1/chat/completions`

### Supported request subset

- `model`
- `messages`
  - roles: `system`, `developer`, `user`, `assistant`, `tool`
  - `system` / `developer` content:
    - string
    - array of text parts only: `{ "type": "text", "text": "..." }`
  - `user` / `assistant` content:
    - string
    - array of text and image parts:
      - `{ "type": "text", "text": "..." }`
      - `{ "type": "image_url", "image_url": { "url": "https://..." } }`
      - `data:` image URLs are also accepted
  - `tool` content:
    - string
    - array of text parts only
    - must include `tool_call_id`
- `max_tokens`
- `temperature`
- `top_p`
- `stop`
- `stream`
- `stream_options.include_usage`
- `tools`
  - function tools only
- `tool_choice`
  - `"none"`, `"auto"`, `"required"`
  - `{ "type": "function", "function": { "name": "..." } }`

### Implemented behavior

- Non-streaming responses return one `chat.completion` with exactly one choice.
- Streaming responses return `text/event-stream` with `data:` lines containing `chat.completion.chunk` objects and a terminal `data: [DONE]`.
- If `stream_options.include_usage=true`, the stream includes the final usage chunk with empty `choices` before `[DONE]`.
- Assistant tool calls are emitted as OpenAI `tool_calls` with function arguments serialized as JSON strings.
- Tool continuation uses `tool` role messages and preserves `tool_call_id`.
- The response `finish_reason` maps as follows:
  - `end_turn` -> `stop`
  - `stop_sequence` -> `stop`
  - `max_tokens` -> `length`
  - `tool_use` -> `tool_calls`

### Rejected/unsupported behavior

The route rejects these known-but-unsupported top-level fields with `400`:

- `n`
- `logprobs`
- `audio`
- `modalities`

Other rejected cases:

- non-function tools
- non-function `tool_choice`
- `tool_choice="required"` with no tools
- `tool_choice` selecting an unknown function
- `system` or `developer` messages containing `image_url`
- `tool` messages without `tool_call_id`
- `tool` messages that do not reference a prior assistant tool call
- `tool_calls[].function.arguments` that are not valid JSON strings

### Minimal example

```json
{
  "model": "gpt-4.1-compatible",
  "messages": [
    { "role": "system", "content": "You are a helpful assistant." },
    { "role": "user", "content": "Hello" }
  ],
  "stream": false
}
```

## `POST /v1/responses`

### Supported request subset

- `model`
- `input`
  - string shorthand
  - array of items
- message input items:
  - `{ "type": "message", "role": "...", "content": [...] }`
  - roles: `system`, `developer`, `user`, `assistant`
  - content parts:
    - `{ "type": "input_text", "text": "..." }`
    - `{ "type": "input_image", "image_url": "...", "detail": "low|high|auto" }`
    - `data:` image URLs are also accepted
- tool continuation input items:
  - `{ "type": "function_call_output", "call_id": "...", "output": "..." }`
- `previous_response_id`
- `tools`
  - function tools only
- `tool_choice`
  - `"none"`, `"auto"`, `"required"`
  - `{ "type": "function", "function": { "name": "..." } }`
- `parallel_tool_calls`
- `text.format.type = "text"`
- `max_output_tokens`
- `temperature`
- `top_p`
- `stop`
- `stream`

### Implemented behavior

- Non-streaming responses return an OpenAI `response` object.
- Sync `status` is currently:
  - `completed` for normal responses
  - `incomplete` when the provider stop reason is `max_tokens`
  - `failed` when the provider stop reason is `error`
- Sync output items currently include:
  - `message` with `output_text`
  - `function_call`
- Streaming responses return `text/event-stream` with named `event:` frames and matching `data.type` values.
- The current streaming event subset is:
  - `response.created`
  - `response.output_item.added`
  - `response.content_part.added`
  - `response.output_text.delta`
  - `response.output_text.done`
  - `response.content_part.done`
  - `response.function_call_arguments.delta`
  - `response.function_call_arguments.done`
  - `response.output_item.done`
  - `response.completed`
- `response.completed` carries the final assembled Response object, including usage.
- `parallel_tool_calls` is preserved into gateway metadata; it does not widen tool support beyond function tools.
- Tool continuation uses `function_call_output` items, preserves `call_id`, and accepts either replayed `function_call` history or `previous_response_id` as the continuation anchor.

### Rejected/unsupported behavior

The route rejects these cases with `400`:

- built-in tools or any non-function tool type
- non-function `tool_choice`
- `tool_choice="required"` with no tools
- `tool_choice` selecting an unknown function
- `text.format.type` other than `"text"`
- empty `function_call_output.call_id`
- `input_image.detail` values outside `low`, `high`, or `auto`

JSON schema output, built-in tools, and non-function tool calls are not implemented on this public surface.

### ChatGPT Codex route maintenance note

This page describes the generic public `/v1/responses` surface. When that surface routes through the ChatGPT Codex OAuth backend, maintainers must revalidate against the Codex route contracts instead of assuming the generic route description applies unchanged.

On the Codex route specifically:

- `stream = true` is forced upstream
- `store = false` is forced upstream
- `text.format.type = "text"` is forced on the route
- bare messages, supported image inputs, flat function tools, explicit function `tool_choice`, and sync-drain execution follow route-specific translate rules
- `reasoning.effort`, `reasoning.summary`, `parallel_tool_calls`, `text.verbosity`, `include = []`, `include = ["reasoning.encrypted_content"]`, `tool_choice = "none"`, and `tool_choice = "auto"` are route-specific pass cases subject to the published constraints
- `max_output_tokens`, `metadata`, `truncation`, `previous_response_id`, `temperature`, `top_p`, `user`, unverified `service_tier` overrides, nested Chat Completions-style tool definitions, nested Chat Completions-style `tool_choice`, `tool_choice = "required"`, and `stream_options` are explicit Codex-route rejects

Codex route maintenance should use these canonical references:

- [`chatgpt-codex-route-contract.md`](contracts/chatgpt-codex-route-contract.md)
- [`chatgpt-codex-auth-handoff-contract.md`](contracts/chatgpt-codex-auth-handoff-contract.md)
- [`chatgpt-codex-conformance-and-drift-guard.md`](contracts/chatgpt-codex-conformance-and-drift-guard.md)

### Minimal example

```json
{
  "model": "gpt-4.1-compatible",
  "input": "Hello",
  "stream": false
}
```

## Conformance Coverage

The current implementation is guarded by targeted offline conformance tests:

- [gateway/tests/openai_chat_completions_conformance.rs](../tests/openai_chat_completions_conformance.rs)
- [gateway/tests/openai_responses_conformance.rs](../tests/openai_responses_conformance.rs)
- [gateway/tests/openai_shared_parity.rs](../tests/openai_shared_parity.rs)

Those tests cover, among other things:

- sync and streaming response shape
- tool-loop continuity
- explicit function `tool_choice`
- reject-vs-ignore posture
- model echo
- `X-Provider` forcing
- reasoning suppression
- shared error-envelope behavior

For ChatGPT Codex route maintenance, the owned deterministic evidence anchors are:

- [gateway/tests/openai_responses_conformance.rs](../tests/openai_responses_conformance.rs)
- [gateway/tests/openai_shared_parity.rs](../tests/openai_shared_parity.rs)
- [gateway/src/server/openai_conformance_test_support.rs](../src/server/openai_conformance_test_support.rs)
- `gateway/tests/fixtures/openai_responses/codex-*.json`

Reopen Codex-route review when any of the following drift materially:

- the Codex route compatibility matrix or supported-control classification changes
- the semantic SSE event family, assembly rules, or sync-drain terminal requirements change
- auth-handoff ownership, field identifiers, precedence rules, or fallback constraints change
- normalized-core behavior changes in a way that invalidates Codex fixture expectations
- fixture namespaces or maintenance-doc evidence anchors move in a way that obscures what the route is proving

## What Changed From The Old Doc

This page used to describe `/v1/chat/completions` as a limited non-streaming, no-tools compatibility shim. That is no longer true.

The repo now ships:

- streaming `chat/completions`
- function tools on `chat/completions`
- public `responses`
- shared OpenAI-side invariants and conformance coverage

The main remaining OpenAI-side gap is token counting: the ADR direction is in place, but only `/v1/messages/count_tokens` is exposed today.
