# OpenAI-Side Chat Completions `C-10` Contract

## Purpose

This note is the canonical landing artifact for `C-10`.
It defines the gateway's **public** OpenAI-compatible `POST /v1/chat/completions` subset, with explicit compatibility boundaries so the endpoint remains a thin adapter over the normalized core.

This contract is intentionally narrow:

- it defines the accepted request subset, tool loop, and response shapes for `/v1/chat/completions`
- it defines the minimum streaming guarantees (SSE chunk semantics and `[DONE]` termination)
- it defines the reject/ignore posture needed for deterministic compatibility

It does not define:

- provider parsing or upstream OpenAI API nuances beyond the frozen subset
- a second execution engine or endpoint-specific runtime semantics
- non-function tools (web search, file search, code interpreter, MCP, etc.)

## Canonical Sources Of Truth

- Normative public contract: [ADR 0008](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md) (Chat Completions section).
- Shared adapter invariants: [openai-side-adapter-invariants-c12-contract.md](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/openai-side-adapter-invariants-c12-contract.md).

If a detail is not derivable from the shared adapter invariants and the frozen subset in ADR 0008, it is not part of `C-10`.

## Request Contract (Supported Subset)

The gateway accepts the following top-level fields:

- `model: string` (required)
- `messages: array` (required)
- `max_tokens: integer` (optional)
- `temperature: number` (optional)
- `top_p: number` (optional)
- `stop: string[]` (optional)
- `tools: array` (optional; function tools only)
- `tool_choice: "none"|"auto"|"required"|object` (optional)
- `stream: boolean` (optional; default `false`)
- `stream_options: { include_usage?: boolean, include_obfuscation?: boolean }` (optional; only meaningful when `stream=true`)

### Messages

`messages[]` supports these roles:

- `system`, `developer`, `user`, `assistant`, `tool`

`content` for `system|developer`:

- a string, or
- `system` and `developer` messages MUST be text-only and MUST reject `image_url` parts with `400`
- array of text parts only:
  - `{ "type": "text", "text": string }`

`content` for `user|assistant`:

- a string, or
- an array of parts:
  - `{ "type": "text", "text": string }`
  - `{ "type": "image_url", "image_url": { "url": string } }` (`url` may be an external URL or a `data:` URL)

`user` and `assistant` messages MAY include `image_url` parts.

`content` for `tool`:

- a string, or
- an array of text parts only (same `{ "type": "text", ... }` structure)

Tool-role messages MUST include:

- `tool_call_id: string` (must match a prior tool call `id`)

### Tooling

Function tools only:

- `tools[]` entries MUST be function tool definitions.
- Any non-function tool definition MUST be rejected with `400` and the gateway error envelope.

`tool_choice`:

- string values are accepted per the subset: `"none"|"auto"|"required"`
- object form is accepted only for explicit function selection:
  - `{ "type": "function", "function": { "name": string } }`

## Ignore vs Reject Posture

Forward-compat posture:

- Unknown top-level fields MUST be ignored.

Deterministic rejection posture:

- The following known-but-unsupported top-level fields MUST be rejected with `400`:
  - `n`
  - `logprobs`
  - `audio`
  - `modalities`

Note: this reject list is intentionally small and explicit. Expanding it is allowed, but must be done deliberately with fixtures and conformance coverage so behavior does not drift silently.

## Response Contract (Non-Streaming)

The gateway returns an OpenAI-compatible Chat Completion object:

- `object: "chat.completion"`
- `model` MUST echo the request `model` (even if a different provider model is used internally)
- `choices` MUST contain exactly one element (`n=1` contract)
- `choices[0].message.role = "assistant"`
- `choices[0].message.content` MAY be `null`/empty when the assistant produces tool calls
- `choices[0].message.tool_calls` MUST be present when the assistant requests tools:
  - `{ "id": string, "type": "function", "function": { "name": string, "arguments": string } }`
  - `arguments` MUST be a JSON string
- `finish_reason` mapping:
  - natural stop: `stop`
  - max tokens: `length`
  - tool requested: `tool_calls`
- `usage` MUST be present when the provider returns token usage; otherwise it MAY be present with zero values

## Response Contract (Streaming)

When `stream=true`, the gateway returns `text/event-stream` and terminates the stream with:

- `data: [DONE]`

Minimum streaming guarantees:

- each `data:` line (before `[DONE]`) MUST be a JSON object with `object: "chat.completion.chunk"`
- `model` in every chunk MUST echo the request `model`
- tool calls MUST stream via `delta.tool_calls`, including incremental `arguments` string deltas when produced by the model
- if `stream_options.include_usage=true`, the gateway MUST emit one additional final usage chunk with empty `choices` before `[DONE]`

Chunk shape requirement (minimum for client assembly):

- each chunk MUST include `choices: [{ "index": 0, "delta": { ... }, "finish_reason": null|string }]`
- `delta.content` is used for text deltas
- `delta.tool_calls` is used for tool-call deltas

The contract does not require specific chunk `id`/`created` values beyond being valid JSON; those are implementation details.

## Tool Loop Contract

Tool continuation uses `tool` role messages:

1. Client sends a request.
2. Gateway returns an assistant message with `tool_calls` and `finish_reason = "tool_calls"`.
3. Client executes tools and appends one `tool` message per tool call:
   - `role: "tool"`
   - `tool_call_id: <matching tool call id>`
   - `content: <tool output text>`
4. Client sends the next request with the extended `messages` array.

## Verification Checklist

`C-10` is complete only if a reviewer can answer yes to all of the following without reading provider parsing code:

- is the request subset allowlisted, with unknown top-level fields ignored and the explicit reject list rejected
- are non-function tools rejected deterministically with the contracted error envelope
- does the tool loop use `tool` role messages with `tool_call_id` and preserve the call id across the continuation
- do non-streaming responses always return `choices` with exactly one element and the right `finish_reason` mapping
- do streaming responses emit `chat.completion.chunk` objects, tool-call deltas in `delta.tool_calls`, optional final usage chunk, and `[DONE]`
- does the response always echo the request `model` and suppress chain-of-thought content
