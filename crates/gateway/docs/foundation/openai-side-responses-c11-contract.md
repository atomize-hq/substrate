# OpenAI-Side Responses `C-11` Contract

## Purpose and boundary

This is the normative public `POST /v1/responses` contract. The endpoint is a thin adapter over the existing `GatewayRequest`, router, provider, `GatewayResponse`, and normalized stream. The closed component covers typed declarations, model calls, client results, subsequent requests, and reconstructed history over HTTP/SSE. It does not introduce an executor, native driver, session registry, or durable gateway history.

Native/client code owns tool execution, history persistence, cancellation, and conversational follow-up. Hosted Responses WebSearch executes upstream. Local component evidence is not full native `0.156.1`, private OAuth backend, installed, retained, production, or F14 qualification.

Canonical companions:

- [ADR 0008](../adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md)
- [Shared adapter invariants (`C-12`)](openai-side-adapter-invariants-c12-contract.md)
- [ChatGPT Codex route](../contracts/chatgpt-codex-route-contract.md)
- [OpenAI semantic streaming guide](https://developers.openai.com/api/docs/guides/streaming-responses)

## Request and admission

`model: string` and `input: string | array` are required. String input is shorthand for one user `input_text`. Ordered typed items are the authoritative history representation; named bare-message input converts to typed messages without changing role, content, or order.

- `instructions` preserves missing, empty, and nonempty distinctly; null and wrong types reject. Ordered system/developer history remains distinct from top-level instructions. Native Lite developer `additional_tools`, prefix IDs, and the separate developer base-instruction message retain their original placement and omissions.
- No bundled-version fallback/overwrite or invented todo/warmup instruction is added on any ingress. A factual cross-API tool-error flag may be represented as typed result data; it is not model instruction text.
- `prompt_cache_key: string` and `client_metadata: object<string, string>` are explicit typed fields. They are source-mandatory in the pinned native producer and MUST survive initial, subsequent, and replayed requests when supplied; omission-only fixtures do not establish native acceptance.
- Optional `reasoning.effort`, admitted `reasoning.summary`, and `reasoning.context` (`current_turn` or `all_turns`) are independent controls. Missing effort is not filled in to transport continuation.
- `include` omission, `[]`, and `["reasoning.encrypted_content"]` follow the same completed-item admission, recipient, and resource policy. Include does not gate accounting or handling of encrypted items actually returned upstream.
- `parallel_tool_calls` preserves supplied true/false and omission. Omission retains upstream defaults. `tool_choice` admits `none`, `auto`, `required`, or an admitted flat explicit selection, subject to the selected route's stricter controls.
- `text.format.type = "text"`, admitted verbosity, `max_output_tokens`, `stream`, and `stream_options` remain subject to selected-provider admission. Actual API-key Responses requests MUST preserve supplied supported `temperature` and `top_p`. The actual Responses endpoint has no `stop` field: supplied stop sequences MUST reject before I/O on that path. Ordinary accepted Chat Completions translation retains its stop/sampling controls; OAuth restrictions remain in the route contract. No accepted control may be silently stripped or a different route selected to hide unsupported semantics.
- `previous_response_id` remains an ordinary API-key stored-continuation anchor only. The advanced component uses complete stateless history; OAuth rejects `previous_response_id`.

Unknown top-level ignore does not apply to recognized unsupported native semantics. Known unsupported types, controls, selected-provider combinations, and malformed typed members reject before upstream I/O with the safe gateway envelope. An `Unknown` content block or generic metadata MUST NOT bypass admission.

## Closed tool and history population

Responses definitions and explicit selection use the flat Responses shape; nested Chat Completions aliases reject here. Chat Completions keeps its actual nested wire shape.

| Population | Required preservation and admission |
| --- | --- |
| Flat function | Name, description, input schema, declaration order, optional `strict` and `defer_loading`; explicit `strict: false` differs from omission. Arguments remain the original validated JSON String, without parse/reserialize mutation. An upstream complete function item accepts absent status or `status: "completed"`; supplied status must validate as completed and is normalized away before closed native replay because pinned native FunctionCall does not retain that lifecycle field. Null, wrong-type, unknown, or noncompleted status on a complete item fails; identity, namespace, and raw arguments remain exact. Pinned native `output_schema` is serde-skipped source state, not a new wire field. |
| Custom and namespace | Text/raw custom input and Lark grammar format/definition; namespace/member order, exact names, IDs, status, and namespace identity. No dotted-name flattening or tool execution. |
| Client `tool_search` | `execution = "client"`, structured arguments, optional call/item IDs, status, ordered loaded definitions and defer flags. Missing call ID stays absent/inert and cannot anchor a result. Native returns loaded tools for the next request; no gateway registry or hosted discovery substitution. |
| Lite `additional_tools` | Developer role, exact prefix ID, definition placement/order, separate base message, top-level omissions, and parallel/store controls. |
| Messages | Ordered role, optional ID and phase; `input_text` and assistant `output_text`. Inline URL/data images preserve absent/auto/low/high/original detail in the admitted Responses population. No image fetch, upload, or local-path conversion. |
| Function/custom result | Exact nonempty call ID, call kind, supplied name/namespace, and Text versus ordered ContentItems; admitted inline media and encrypted result members remain typed. |
| Reasoning | Completed item with optional ID, required `summary` Vec of upstream-labelled `summary_text`, and admitted opaque `encrypted_content`. Empty stays empty; nonempty summaries are preserved under the approved typed Responses policy. |
| Compaction | History-only `compaction` requires an encrypted String; `context_compaction` preserves absent versus String, including empty. Preserve position and optional ID. Null/wrong type, `compaction_summary` aliases, and private triggers/controls reject. |
| Hosted Responses WebSearch | Named access, filter/location, context-size/content-type options and activity action/status/ID/order, plus assistant text and activity history. Selected destination must admit the requested option. This is not standalone search or a fake client call/result. |

A result MUST reference an earlier same-request authoritative call of the same kind; supplied name/namespace MUST agree. Identity spelling and raw payload bytes are preserved. Identical repeated complete calls may be coalesced at their first authoritative position; conflicting duplicates reject. Ordinary stored API-key continuation retains its separate supported anchor. Normalized provenance may supply a missing ordinary function-call wire item only when it contains the actual ID, name, and arguments; placeholders are forbidden.

Explicitly excluded: standalone search/images, WebSocket parity, audio, image/file-reference input, encrypted agent messages/function arguments, internal chat passthrough/private controls, final-answer JSON schema, and citation-annotation fidelity. Other unlisted built-ins are unsupported. Exclusions fail explicitly; no model/auth/tool-mode downgrade or state stripping manufactures completion.

## Completed continuation and safe projection

Authoritative `response.output_item.done` replaces provisional added state for opaque replay; ciphertext and summary snapshots are not concatenated. Clone and normalized transport serde MUST retain exact admitted state for the next request and reconstructed history. Normalized `ToolResultContent::ResponseItems { items }` uses an object wrapper to distinguish typed Responses result arrays on reload; public Responses arrays stay arrays, and ordinary Anthropic empty result arrays keep their meaning. Private `Reasoning.content`, `reasoning_text`, and private metadata never become public text, tool arguments, logs, traces, Debug/Display output, or error messages.

Approved completed opaque items and their required summary Vec may return as typed Responses items to the same connected Responses caller under the actual access boundary. They are not generic message text, annotations, or reasoning deltas. Other public endpoints do not acquire this OpenAI continuation state. Diagnostic projections are separate from authoritative wire/history serializers.

Adapter-owned `GatewayRequest.responses_caller` is serde-skipped endpoint-origin bookkeeping. It is not caller metadata, authentication, a native principal, or ciphertext-origin attestation. Native remains the durable history owner. No gateway persistence, keys, cross-request ledger, or stronger session/member/cryptographic binding is introduced; standalone network access does not establish those assurances.

Before I/O, select and pin one backend/endpoint, account/credential context, and actual model for the request. Unsupported advanced combinations fail. No failure may trigger a different destination or retry with stripped opaque state. Responses transport disables automatic redirects and fails 3xx safely. This does not prove DNS/IP/TLS/world confinement or cross-request ciphertext provenance.

## Response and streaming contract

Non-streaming returns `object: "response"`, the original public model name, ordered admitted output items, applicable usage/cache usage, and a truthful status (`completed`, `in_progress`, `failed`, or `incomplete`). No cache hit or effectiveness is fabricated.

SSE uses named `response.*` events whose JSON `data.type` matches the event name. The semantic subset includes:

- `response.created`, preceding public output
- `response.output_item.added` and `response.output_item.done`
- text `response.content_part.added`, `response.output_text.delta`, `response.output_text.done`, and `response.content_part.done`
- function `response.function_call_arguments.delta` and `response.function_call_arguments.done`
- typed completed custom-call, history, and activity items
- terminal `response.completed`, carrying the final assembled public Response and usage

Item added precedes its deltas/done events; text content-part done and function-argument done precede the corresponding item done. Item/call IDs and output ordering permit reconstruction without provider framing. The provider owns semantic assembly for sync and streaming; public handlers transform normalized output only. OAuth completed envelopes supply lifecycle/usage, not an assumed answer snapshot.

`response.completed.response.end_turn` is independent of `stop_reason`: absent/null maps to None and is omitted publicly; false and true remain explicit; wrong types fail. Core, Clone, normalized serde, and the internal `gateway.response_terminal` preserve it. Other endpoint serializers consume the internal terminal rather than exposing it. False cannot become a conversational `Final`; true/absence cannot erase pending native follow-up.

Sync success requires a valid terminal. Error, incomplete, conflicting done payload, invalid UTF-8, missing terminal/EOF, oversize, or cancellation MUST NOT synthesize completed/Final success. Before headers, provider failures use safe 502 errors as applicable; after streaming starts, fail/close the body without a success terminal. The first valid terminal fuses the stream. Consumer drop releases owned upstream state with no detached drain or retry; it does not promise rollback or at-most-once remote execution.

## Engineering ceilings

The component uses 2 MiB encoded ingress entity, SSE frame, and assembled semantic-state ceilings; 16 MiB cumulative upstream wire; and 64 MiB accounted simultaneous working payload with reserved partitions, including a 6 MiB normalized bundle and 8 MiB public queue. See `models/response_transport.rs` for constants and the implementation's accounting sites. Bounds apply before parse/extension and to held state, copies, and queued payload; default encrypted returns count too. Mapping selection MUST borrow configuration entries rather than clone all mapping Strings; selected model/provider identifier payload must be admitted before owned copies are created, including directly reached selection diagnostics.

These are engineering admission limits, not upstream maxima, measured normal-history capacity, allocator/RSS bounds, or concurrency/deployment qualification. Transport may already own a received chunk. Over-limit handling fails and drops/cancels without truncation, ciphertext rewriting, fallback, lost-history success, or detached draining. Representative native workload fit remains unproved.

## Verification anchors and limits

- [Public adapter](../../src/server/openai_responses.rs), [core](../../src/core.rs), [typed transport](../../src/models/response_transport.rs)
- [OpenAI provider](../../src/providers/openai.rs) and [framing](../../src/providers/streaming.rs)
- [Complete component regressions](../../tests/openai_codex_0156_roundtrip.rs), [Responses conformance](../../tests/openai_responses_conformance.rs), and [shared parity](../../tests/openai_shared_parity.rs)

Required proof uses independent literal local HTTP/SSE through handler/core/router/OpenAIProvider, subsequent/history replay, mandatory cache fields, exact kinds/IDs/raw payloads, default opaque/summary handling, safe sinks, terminal states, budgets, redirects, and cancellation. Existing Chat/Anthropic signed-thinking and caching/Azure/Kimi/Gemini protocol regressions remain meaningful. Local transport preservation is not actual native execution/resume, private OAuth acceptance, or cache-effectiveness proof; OAuth serializer-only checks remain UNSENT.
