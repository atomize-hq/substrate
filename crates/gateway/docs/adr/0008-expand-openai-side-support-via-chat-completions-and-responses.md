# ADR 0008: Expand OpenAI-Side Support via `chat/completions` and `responses`

- Status: Accepted
- Date: 2026-04-03

## Context

The gateway's current public priority is the Anthropic-compatible `/v1/messages` surface used by Claude Code.
The repo also exposes a limited public `/v1/chat/completions` compatibility route, while the provider layer already speaks upstream OpenAI-family protocols such as `chat/completions`, `responses`, and `codex/responses` when needed.

This creates a natural question for future API expansion:

- should the gateway add broader OpenAI-compatible public support
- if so, which public surfaces should grow first

The current architecture is already centered on:

- multi-message conversations
- tool use and tool-result continuation
- streaming behavior
- normalized internal events rather than raw provider framing

That architecture fits modern OpenAI-shaped APIs better than legacy prompt-only completions.

## Decision

If the gateway expands its public OpenAI-side API support, it should do so in this order:

1. expand public `/v1/chat/completions` when broader compatibility is needed for existing SDKs, older OpenAI-style clients, or drop-in compatibility scenarios
2. add public `/v1/responses` as the preferred modern OpenAI-shaped surface for tool use, streaming, and richer response semantics

## Rationale

`/v1/chat/completions` remains useful because:

- many existing SDKs and integrations still target it
- it is the lowest-friction compatibility bridge for existing OpenAI-shaped clients
- it can be expanded incrementally from the gateway's current limited implementation

`/v1/responses` is the right longer-term OpenAI-facing surface because:

- it better matches the gateway's structured, multi-turn, tool-capable architecture
- it is a better fit for streaming and richer response semantics
- it aligns more cleanly with the normalized-core design already established by prior ADRs

## Consequences

Positive:

- OpenAI-side expansion stays aligned with the existing normalized-core architecture
- compatibility work can serve both legacy SDK adoption and modern API direction without inventing a second gateway model
- the public API roadmap becomes clearer: compatibility-first via `chat/completions`, modern capability-first via `responses`

Negative:

- supporting both public `/v1/chat/completions` and public `/v1/responses` will require careful boundary discipline so they remain thin adapters over the same core
- documentation will need to clearly distinguish primary, compatibility, and modern expansion surfaces

## Contract (Target Public API)

This section is the **normative, implementable** contract for OpenAI-side public ingress.

Notes:

- The current repo already exposes `/v1/chat/completions`, but its implementation may lag this contract while it is expanded.
- `/v1/responses` may not exist yet; this contract freezes what it must look like once added.
- Both endpoints MUST remain thin adapters over the same internal request/response core (see “Adapter Boundaries” below).

### Shared Behavior (Both Endpoints)

#### Model, routing, and provider selection

- `model` is a **public gateway model name** used for routing and mapping.
- The gateway MAY map `model` to provider-specific deployments/models with fallback.
- The gateway MUST echo the original request `model` back in the response payload (even if a different provider model is used internally).
- Optional header: `X-Provider: <provider_name>`
  - When present, the gateway MUST constrain provider selection to that provider (or fail routing).
  - When absent, the gateway MUST follow configured provider priority/fallback.

#### Error envelope

All error responses MUST be JSON in the gateway error envelope:

```json
{
  "error": {
    "type": "error",
    "class": "auth|url|deployment|route|transport_drift",
    "message": "human-readable summary"
  }
}
```

Status code mapping (contracted):

- `400` for invalid request shapes, unsupported parameters, or routing failures
- `502` for provider failures after routing succeeds

#### Content constraints and safety

- The gateway MUST NOT return chain-of-thought content (provider “thinking”, “reasoning_content”, etc.) as user-visible text.
- The gateway MAY expose a short “reasoning summary” only if the upstream protocol provides one explicitly and it is safe to surface.

#### Tooling compatibility scope

For OpenAI-side public ingress, the gateway supports **function tools only**:

- Accept function tools in request `tools`.
- Emit function calls in the response as OpenAI-compatible tool/function call objects.
- Accept tool outputs from the caller and continue the conversation.

Built-in tools (web search, file search, code interpreter, MCP, etc.) are out of scope for this gateway contract unless a later ADR adds them.

### Endpoint: `POST /v1/chat/completions` (Compatibility Surface)

This endpoint exists to support OpenAI SDKs/integrations that still target Chat Completions.

#### Request (supported fields)

The gateway MUST accept a subset of OpenAI’s Chat Completions request:

- `model: string` (required)
- `messages: array` (required)
  - Supported roles: `system`, `developer`, `user`, `assistant`, `tool`
  - `content`:
    - For `system|developer`: either a string OR an array of text parts only
      - Supported part type:
        - `{ "type": "text", "text": string }`
      - `system` and `developer` messages MUST reject `image_url` parts with `400`
    - For `user|assistant`: either a string OR an array of parts
      - Supported part types:
        - `{ "type": "text", "text": string }`
        - `{ "type": "image_url", "image_url": { "url": string } }` (URL or `data:` URL)
    - For `tool`: either a string OR an array of text parts ONLY
      - Tool messages MUST include `tool_call_id: string`
- Sampling/limits:
  - `max_tokens: integer` (optional; default determined by gateway)
  - `temperature: number` (optional)
  - `top_p: number` (optional)
  - `stop: string[]` (optional)
- Tool calling:
  - `tools: array` (optional) — function tool definitions only
  - `tool_choice: "none"|"auto"|"required"|object` (optional)
- Streaming:
  - `stream: boolean` (optional; default `false`)
  - `stream_options: { include_usage?: boolean, include_obfuscation?: boolean }` (optional; only meaningful when `stream=true`)

Unsupported/ignored fields:

- Unknown top-level fields MUST be ignored (forward-compat posture).
- Known but unsupported fields MUST be rejected with a `400` error (e.g. `n`, `logprobs`, audio modalities), unless a later ADR adds them.

#### Response (non-streaming)

The gateway MUST return an OpenAI-compatible Chat Completion object:

- `object: "chat.completion"`
- `choices` MUST contain exactly one element (`n=1` contract)
- `choices[0].message.role = "assistant"`
- `choices[0].message.content` MAY be `null`/empty when the assistant produces tool calls
- `choices[0].message.tool_calls` MUST be present when the assistant requests tools
  - Function tool call shape:
    - `{ "id": string, "type": "function", "function": { "name": string, "arguments": string } }`
    - `arguments` MUST be a JSON string (callers must parse/validate it before execution)
- `finish_reason` mapping:
  - natural stop → `stop`
  - max tokens hit → `length`
  - tool requested → `tool_calls`
- `usage` MUST be present when the provider returns token usage; otherwise it MAY be present with `null`/zero values (documented per implementation).

#### Response (streaming)

If `stream=true`, the gateway MUST return `text/event-stream` with OpenAI-compatible chunks and a terminating `data: [DONE]`.

Minimum streaming guarantees:

- Each `data:` line MUST be a JSON object with `object: "chat.completion.chunk"`.
- Tool calls MUST stream via `delta.tool_calls` (incremental `arguments` string deltas) when produced by the model.
- If `stream_options.include_usage=true`, the gateway MUST emit the additional final usage chunk (with empty `choices`) before `[DONE]`.

#### Tool loop contract (chat completions)

Tool continuation MUST be performed using `tool` role messages:

1. Client sends request.
2. Gateway returns assistant message with `tool_calls` and `finish_reason = "tool_calls"`.
3. Client executes tools, then appends one `tool` message per tool call:
   - `role: "tool"`
   - `tool_call_id: <matching tool call id>`
   - `content: <tool output text>`
4. Client sends the next request with the extended `messages` array.

### Endpoint: `POST /v1/responses` (Preferred Modern Surface)

This endpoint is the preferred OpenAI-facing surface for tool use, richer response semantics, and structured streaming.

The gateway’s `/v1/responses` contract is based on OpenAI’s Responses API, with an explicit compatibility subset.

#### Request (supported fields)

- `model: string` (required)
- `input: string | array` (required)
  - `string` is shorthand for a single user `input_text`.
  - `array` MUST be an array of **items**. Supported item types:
    - `message` (input messages):
      - `{ "type": "message", "role": "system"|"developer"|"user"|"assistant", "content": [ ... ] }`
      - Content parts supported:
        - `{ "type": "input_text", "text": string }`
        - `{ "type": "input_image", "image_url": string, "detail"?: "low"|"high"|"auto" }` (URL or `data:` URL)
    - `function_call_output` (tool outputs provided by the caller):
      - `{ "type": "function_call_output", "call_id": string, "output": string }`
- Tool calling:
  - `tools: array` (optional) — function tool definitions only
  - `tool_choice: "none"|"auto"|"required"|object` (optional)
  - `parallel_tool_calls: boolean` (optional; default `true`)
- Output shape:
  - `text: { format: { type: "text" } }` (optional; default text)
  - JSON schema outputs are NOT supported unless a later ADR adds them.
- Sampling/limits:
  - `max_output_tokens: integer` (optional)
  - `temperature: number` (optional)
  - `top_p: number` (optional)
  - `stop: string[]` (optional)
- Streaming:
  - `stream: boolean` (optional; default `false`)
  - `stream_options: { include_obfuscation?: boolean }` (optional)

Unsupported/ignored fields:

- Unknown top-level fields MUST be ignored (forward-compat posture).
- Known but unsupported built-in tools and tool call types MUST be rejected with a `400` error unless explicitly supported by a later ADR.

#### Response (non-streaming)

The gateway MUST return an OpenAI-compatible Response object:

- `object: "response"`
- `status: "completed"|"in_progress"|"failed"|"incomplete"` (the gateway will normally return `completed` for synchronous mode)
- `output: array` containing zero or more output items
  - Supported output item types:
    - `message` (assistant output):
      - `{ "type": "message", "role": "assistant", "content": [ { "type": "output_text", "text": string, "annotations": [] } ] }`
    - `function_call` (assistant tool request):
      - `{ "type": "function_call", "call_id": string, "name": string, "arguments": string }`
- `usage` MUST be present when the provider returns token usage.

#### Response (streaming)

If `stream=true`, the gateway MUST return `text/event-stream` using OpenAI Responses streaming events.

The gateway MUST support the following minimum event set (in typical order):

- `event: response.created`
- `event: response.output_item.added`
- `event: response.content_part.added`
- `event: response.output_text.delta` (for text streaming)
- `event: response.output_text.done`
- `event: response.content_part.done`
- `event: response.output_item.done`
- `event: response.completed` (final response object)

Tool streaming MUST be supported via:

- `event: response.function_call_arguments.delta`
- `event: response.function_call_arguments.done`

Every SSE `data:` payload MUST be a JSON object with a `type` string that matches the event name (e.g. `{"type":"response.output_text.delta", ...}`).

#### Tool loop contract (responses)

Tool continuation MUST be performed using `function_call_output` input items:

1. Client sends a request.
2. Gateway emits `function_call` output items (non-streaming: in `output`; streaming: via events).
3. Client executes tools, then sends a follow-up `/v1/responses` request whose `input` appends one `function_call_output` item per tool call:
   - `call_id` MUST match the tool call id emitted by the model.
   - `output` MUST be a string (the client is responsible for serializing structured outputs).

### Adapter Boundaries (Implementation Constraints)

Public `/v1/chat/completions` and `/v1/responses` MUST:

- Parse into the same internal `GatewayRequest` shape used by routing and providers.
- Represent tool requests as internal `tool_use` blocks and tool results as `tool_result` blocks (or equivalent internal representation).
- Convert from internal `GatewayResponse` (content blocks + stop semantics + usage) back into OpenAI-shaped output objects.
- Keep streaming adapters as pure transforms over the provider-normalized stream (no provider-specific streaming logic in the public adapter).

### References (Non-Normative)

- OpenAI Chat Completions API reference: https://platform.openai.com/docs/api-reference/chat
- OpenAI Responses API reference: https://platform.openai.com/docs/api-reference/responses
- OpenAI streaming guide (SSE patterns): https://platform.openai.com/docs/guides/streaming

## Constraints On Implementation

- keep `/v1/messages` as the primary public contract for the current Claude Code path unless a later ADR changes that priority
- treat public `/v1/chat/completions` and public `/v1/responses` as thin outer adapters over the same normalized internal core
- do not create separate execution engines or provider-normalization branches per public API family

## Deliverable Boundary

This ADR is complete when:

1. the repo has a clear architectural decision that future OpenAI-side expansion should favor public `/v1/chat/completions` enhancement and public `/v1/responses` addition
2. later implementation work can cite this ADR as the policy boundary for OpenAI-side public API expansion
3. the target OpenAI-side contract is fully specified (request fields, response fields, streaming behavior, error envelope, and tool loop semantics) so it can be implemented as thin adapters without inventing a second gateway model
