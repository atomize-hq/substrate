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
- `/v1/responses` is implemented; its closed typed component is specified by [C-11](../foundation/openai-side-responses-c11-contract.md).
- Both endpoints MUST remain thin adapters over the same internal request/response core (see “Adapter Boundaries” below).

### Shared Behavior (Both Endpoints)

#### Model, routing, and provider selection

- `model` is a **public gateway model name** used for routing and mapping.
- The gateway MAY map `model` to provider-specific deployments/models with fallback for ordinary representable requests. Advanced Responses requests pin backend/account/model within the request and fail without fallback or state stripping.
- The gateway MUST echo the original request `model` back in the response payload (even if a different provider model is used internally).
- Optional header: `X-Provider: <provider_name>`
  - When present, the gateway MUST constrain provider selection to that provider (or fail routing).
  - When absent, ordinary requests follow configured provider priority/fallback; advanced Responses admission preserves the destination pin above.

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
- Approved typed completed Responses continuation preserves opaque state and the exact required upstream-labelled `summary_text` Vec to the same connected permitted Responses caller under C-11. Private reasoning content and metadata never become public text or diagnostics; other endpoints do not acquire this state.

#### Tooling compatibility scope

Chat Completions supports function tools with its actual nested declarations and JSON-string call arguments. Responses admits the closed C-11 function/custom/namespace/client-search/loaded-definition/Lite/hosted-activity population, with exact typed call/result/history preservation. Unlisted built-ins reject. Native/client code executes client tools; the gateway does not acquire an executor or registry.

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

#### Request, tools, and history

[C-11](../foundation/openai-side-responses-c11-contract.md) is the normative field/item admission and streaming contract for this endpoint. Its closed subset supersedes the original function-only Responses scope of this ADR:

- `model` and `input` are required; shorthand/bare messages convert to ordered typed items without changing roles or content.
- Instructions preserve missing/empty/nonempty and reject null/wrong types; ordered system/developer history and native Lite additional-tools/base-prefix placement remain intact. No bundled-version instruction fallback/overwrite or invented todo/warmup instruction is added on any ingress.
- Flat function/custom/namespace/client-search declarations and explicit choices preserve kind, IDs, namespaces, order, optional strict false versus omission, and raw versus structured payloads. Pinned native tool `output_schema` is serde-skipped source state, not a wire feature.
- Typed loaded definitions, messages/media, function/custom results, reasoning/compaction history, and hosted Responses WebSearch activity follow the closed C-11 admission and provenance rules. Results need an earlier same-kind authoritative call; placeholders and `Unknown` bypasses are forbidden.
- Source-native `prompt_cache_key: string` and `client_metadata: object<string,string>` preserve supplied values through initial, subsequent, and serialized-history requests. Actual cache controls/usage are preserved when present; this is not a cache-effectiveness claim.
- Optional effort/context/summary controls and include omission/empty/explicit remain independent. All admitted completed opaque returns use the same validation, disclosure, and budget policy.
- Ordinary API-key stored continuation may use its supported `previous_response_id` anchor. The advanced component uses complete stateless history; OAuth rejects that field and other unsupported controls under the [route contract](../contracts/chatgpt-codex-route-contract.md).

Unknown top-level ignore MUST NOT discard known unsupported native semantics. Unsupported type/control/provider combinations fail before I/O, without silently changing model, auth, tool mode, or history. Final-answer JSON schema, other built-ins, standalone search/images, WebSockets, audio/file references, private controls, and citation fidelity remain excluded.

#### Output and terminal semantics

Responses output uses `object: "response"`, public model echo, ordered admitted typed items, truthful status, and provider usage including cache usage when present. The existing normalized core/provider owns semantic assembly for both sync and SSE; the public handler does not parse upstream framing.

Named SSE events have matching `data.type`. Item-added precedes applicable text/function deltas and done events; content/argument done precedes item done; `response.completed` is the successful terminal carrying the assembled public response. Authoritative complete `output_item.done` replaces provisional opaque state; exact required summary Vec and opaque bytes survive Clone and transport serde. Private reasoning remains excluded from public text and diagnostics.

`end_turn` absent/null maps to None; false/true remain explicit, independently of stop reason. Wrong types fail. Core and normalized terminal preserve it; Responses emits the optional member, and other endpoints consume the internal terminal. False cannot become a conversational `Final`; true/absence cannot clear pending native follow-up. Error, incomplete, malformed/oversize, EOF without terminal, and cancel cannot fabricate successful completion. First terminal fuses the stream; caller drop releases upstream ownership without detached drain or retry.

Native owns execution, follow-up, and durable history. Adapter-origin bookkeeping is not authentication or native/ciphertext attestation. This component adds no persistence, key, session registry, or stronger cross-request binding. Responses disables automatic redirects with safe 3xx failure; this does not prove network confinement. C-11 engineering ceilings are admission budgets, not upstream/RSS maxima or validated normal-history capacity.

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
