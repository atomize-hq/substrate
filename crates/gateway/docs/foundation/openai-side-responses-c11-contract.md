# OpenAI-Side Responses `C-11` Contract

## Purpose

This note is the canonical landing artifact for `C-11`.
It defines the gateway's **public** OpenAI-compatible `POST /v1/responses` subset, with explicit compatibility boundaries so the endpoint remains a thin adapter over the normalized core.

This contract is intentionally narrow:

- it defines the accepted request subset, tool loop, and response shapes for `/v1/responses`
- it defines the minimum streaming guarantees, including event sequencing and payload conventions
- it defines the reject/ignore posture needed for deterministic compatibility

It does not define:

- provider parsing or upstream OpenAI API nuances beyond the frozen subset
- a second execution engine or endpoint-specific runtime semantics
- built-in tools, schema outputs, or non-function tool call types

## Canonical Sources Of Truth

- Normative public contract: [ADR 0008](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md) (Responses section).
- Shared adapter invariants: [openai-side-adapter-invariants-c12-contract.md](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/openai-side-adapter-invariants-c12-contract.md).
- Upstream reference for event semantics: [OpenAI Streaming API responses guide](https://developers.openai.com/api/docs/guides/streaming-responses).

If a detail is not derivable from the shared adapter invariants and the frozen subset in ADR 0008, it is not part of `C-11`.

## Request Contract (Supported Subset)

The gateway accepts the following top-level fields:

- `model: string` (required)
- `input: string | array` (required)
- `tools: array` (optional; function tools only)
- `tool_choice: "none"|"auto"|"required"|object` (optional)
- `parallel_tool_calls: boolean` (optional; default `true`)
- `text: { format: { type: "text" } }` (optional; default text)
- `max_output_tokens: integer` (optional)
- `temperature: number` (optional)
- `top_p: number` (optional)
- `stop: string[]` (optional)
- `stream: boolean` (optional; default `false`)
- `stream_options: { include_obfuscation?: boolean }` (optional; only meaningful when `stream=true`)

### Input

`input` supports:

- `string` shorthand, which is treated as a single user `input_text`
- `array` of items, with the following supported item types:
  - `message`
  - `function_call`
  - `function_call_output`

`message` items support:

- `role: "system"|"developer"|"user"|"assistant"`
- `content: array`

`message.content` parts support:

- `{ "type": "input_text", "text": string }`
- `{ "type": "input_image", "image_url": string, "detail"?: "low"|"high"|"auto" }`

`function_call` items support:

- `{ "type": "function_call", "call_id": string, "name": string, "arguments": string }`
- `call_id` and `name` MUST be non-empty strings.
- `arguments` MUST be a string containing valid JSON.

`function_call_output` items support:

- `{ "type": "function_call_output", "call_id": string, "output": string }`
- `function_call_output.call_id` MUST either reference a prior `function_call` item in the same request or be anchored by `previous_response_id` for a valid follow-up continuation request.

## Tooling

Function tools only:

- `tools[]` entries MUST be function tool definitions.
- Any non-function tool definition MUST be rejected with `400` and the gateway error envelope.

`tool_choice`:

- string values are accepted per the subset: `"none"|"auto"|"required"`
- object form is accepted only for explicit function selection:
  - `{ "type": "function", "function": { "name": string } }`

`parallel_tool_calls`:

- defaults to `true`
- MUST be preserved as part of the public request contract
- MUST NOT widen scope to non-function tool execution

Tool-loop continuation:

1. Client sends a request.
2. Gateway emits `function_call` output items.
3. Client executes tools and sends a follow-up request that either:
   - preserves the authoritative prior `function_call` item and appends one `function_call_output` item per tool call, or
   - supplies `previous_response_id` and appends one `function_call_output` item per tool call without replaying the prior `function_call`.
4. `function_call_output.call_id` MUST match the emitted tool id exactly.
5. `function_call_output.output` MUST be a string.
6. Orphaned `function_call_output` items reject before any upstream call; the gateway MUST NOT fabricate placeholder tool metadata.

## Ignore vs Reject Posture

Forward-compat posture:

- Unknown top-level fields MUST be ignored.

Deterministic rejection posture:

- The following known-but-unsupported behaviors MUST be rejected with `400`:
  - built-in tools
  - non-function tool call types
  - any `text.format.type` value other than `"text"`; JSON-schema output requests remain out of scope

Notes:

- `text.format.type` is frozen to `text`; JSON schema outputs remain out of scope until a later ADR.
- Expanding the explicit reject list is allowed, but must be done deliberately with fixtures and conformance coverage so behavior does not drift silently.

## Response Contract (Non-Streaming)

The gateway returns an OpenAI-compatible Response object:

- `object: "response"`
- `status: "completed"|"in_progress"|"failed"|"incomplete"`
- `model` MUST echo the request `model` even if a different provider model is used internally
- `output: array` containing zero or more output items
- `usage` MUST be present when the provider returns token usage

Supported output item types:

- `message`
  - `{ "type": "message", "role": "assistant", "content": [ { "type": "output_text", "text": string, "annotations": [] } ] }`
- `function_call`
  - `{ "type": "function_call", "call_id": string, "name": string, "arguments": string }`

Mapping rules:

- text-only results MAY produce a single assistant `message`
- tool-request results MUST produce `function_call` output items
- mixed results MUST preserve stable item ordering
- `function_call.arguments` MUST remain a JSON string
- `finish` / status semantics MUST stay within the contracted `status` set and reflect completion, in-progress, failure, or incompleteness without inventing endpoint-specific states

## Response Contract (Streaming)

When `stream=true`, the gateway returns `text/event-stream` using OpenAI Responses streaming events.

OpenAI's current Responses documentation describes streaming as a semantic event surface with typed events, not one universal event trace that every stream must contain. This contract follows that posture by freezing one supported event subset plus ordering rules for the events that actually occur in a given stream.

Supported streaming event subset:

- lifecycle events:
  - `response.created`
  - `response.completed`
- output-item events:
  - `response.output_item.added`
  - `response.output_item.done`
- text-content events, when the streamed item is assistant text:
  - `response.content_part.added`
  - `response.output_text.delta`
  - `response.output_text.done`
  - `response.content_part.done`
- function-call argument events, when the streamed item is a function call:
  - `response.function_call_arguments.delta`
  - `response.function_call_arguments.done`
- each SSE `data:` payload MUST be a JSON object with a `type` string that exactly matches the event name
- `response.output_text.delta` MUST carry text delta payloads only, with enough information for client-side reconstruction of the text stream
- `response.function_call_arguments.delta` / `response.function_call_arguments.done` MUST carry `call_id` continuity plus tool-argument delta/completion payloads for function calls
- `response.completed` MUST carry the final Response object
- required payload fields MUST be sufficient for client-side assembly of output items without consulting provider-specific framing

Ordering guarantees:

- `response.created` MUST precede all other public output events
- when text content is streamed, `response.output_item.added` MUST precede `response.content_part.added`, `response.output_text.delta`, `response.output_text.done`, `response.content_part.done`, and `response.output_item.done` for that item
- when function-call arguments are streamed, `response.output_item.added` MUST precede `response.function_call_arguments.delta`, `response.function_call_arguments.done`, and `response.output_item.done` for that item
- when text content is streamed, `response.content_part.done` MUST precede `response.output_item.done`
- when function-call arguments are streamed, `response.function_call_arguments.done` MUST precede `response.output_item.done`
- `response.completed` MUST be the terminal event before stream termination

## Thin Adapter Boundary

Public `/v1/responses` MUST:

- parse into the same internal `GatewayRequest` shape used by routing and providers
- represent tool requests as internal `tool_use` blocks and tool results as `tool_result` blocks
- convert from internal `GatewayResponse` plus the normalized stream model back into OpenAI-shaped output objects
- keep streaming adapters as pure transforms over the provider-normalized stream

The public handler MUST NOT:

- parse provider-specific streaming framing
- create a second execution engine
- introduce endpoint-specific semantics that bypass the normalized core

## Verification Checklist

`C-11` is complete only if a reviewer can answer yes to all of the following without reading provider parsing code:

- is the request subset allowlisted, with unknown top-level fields ignored and known unsupported built-in/tool-call behavior rejected deterministically
- are `input` shorthand, `message`, and `function_call_output` shapes frozen with explicit `call_id` threading rules
- are non-function tools rejected deterministically with the contracted gateway error envelope
- do non-streaming responses return `object: "response"` with the right `status`, `output` item mapping, `model` echo, and usage behavior
- do streaming responses emit the contracted `response.*` events that apply to the streamed output, with `data.type` matches, payload minimums, ordering guarantees, tool-argument deltas/done when function calls are streamed, and terminal `response.completed`
- does the response always suppress chain-of-thought content
- do `/v1/responses` handlers remain thin adapters over `GatewayRequest` / `GatewayResponse` / normalized stream output

## Verification Anchors

These anchors are the repo surfaces later slices should use for fixtures, regression coverage, and closeout evidence:

- request parsing and route wiring: `gateway/src/server/mod.rs`
- shared boundary types: `gateway/src/core.rs`
- internal tool/content block shapes: `gateway/src/models/mod.rs`
- Responses fixtures and regressions: `gateway/tests/`
- adapter invariant baseline: `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- policy baseline: `docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`
