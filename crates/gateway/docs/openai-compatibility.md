# OpenAI API Compatibility

This gateway now exposes two public OpenAI-shaped surfaces:

- `POST /v1/chat/completions`
- `POST /v1/responses`

Both are implemented as thin adapters over the same internal gateway core. They share routing, model mapping, a normalized core, private-reasoning suppression, and the same redacted error envelope. Ordinary representable requests retain configured fallback; advanced Responses requests pin their selected backend/account/model and fail without fallback or stripped history.

`/v1/messages` remains the primary Claude Code-facing surface. The OpenAI routes exist for OpenAI-compatible clients and SDKs.

## Current Status

| Surface | Status | Notes |
| --- | --- | --- |
| `POST /v1/chat/completions` | Implemented | Supports sync + streaming, function tools, tool continuation, images, model echo, `X-Provider` forcing |
| `POST /v1/responses` | Implemented | Supports sync + streaming and the closed typed declaration/call/result/next/history component in C-11, admitted inline media, model echo, `X-Provider` forcing |
| OpenAI-side token counting | Not yet exposed | `POST /v1/messages/count_tokens` exists today; ADR 0009 sets direction for ingress-wide token counting, but no OpenAI count-tokens route has landed yet |

Repo anchors:

- [ADR 0008](./adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md)
- [Chat Completions contract (`C-10`)](./foundation/openai-side-chat-completions-c10-contract.md)
- [Responses contract (`C-11`)](./foundation/openai-side-responses-c11-contract.md)
- [Shared adapter invariants (`C-12`)](./foundation/openai-side-adapter-invariants-c12-contract.md)
- [Conformance suite contract (`C-13`)](./foundation/openai-side-conformance-suite-c13-contract.md)

## Shared Behavior

These behaviors are shared across both OpenAI endpoints:

- `model` is a public gateway model name. The gateway may route to a different provider model internally, but the response echoes the original request `model`.
- Optional `X-Provider: <provider_name>` constrains routing to one configured provider. If the named provider is missing or unmapped for the requested model, the request fails with a route error.
- Ordinary representable requests use configured model mappings and fallback when `X-Provider` is absent. Advanced Responses requests reject unsupported destinations and preserve the per-request backend/account/model pin; no fallback or state stripping.
- Private provider reasoning never becomes public text or diagnostics. Approved completed opaque continuation and required upstream summary Vec may return as typed Responses items to the same connected permitted caller under C-11; other endpoints do not acquire that state.
- Unknown top-level ignore does not apply to recognized unsupported native semantics. Known unsupported typed items/controls/provider combinations fail before I/O; generic metadata or `Unknown` blocks cannot bypass admission.

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

The public `message` value is intentionally redacted to the stable gateway summary, not the raw upstream/provider error text. Reached malformed-success/error diagnostics must use fixed finite classifications rather than raw decoder errors, full CodeAssist request dumps, or raw 404 bodies. Local API-key observations and UNSENT OAuth/CodeAssist source or pure-helper checks have distinct proof scopes; this is not universal log/security qualification.

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

### Closed supported component

[C-11](foundation/openai-side-responses-c11-contract.md) specifies the normative subset: flat function/custom/namespace/client-search and loaded definitions, Lite developer additional-tools placement, exact model call/client result/subsequent request/serialized history, admitted inline media/detail, completed opaque/summary and compaction history, and hosted Responses WebSearch activity. Function JSON Strings, custom raw Strings, structured discovery arguments, IDs/kinds/namespaces, optional strict false versus omission, and history order remain distinct. Native owns client execution and durable history; the gateway adds no executor or registry.

Supplied instructions preserve missing/empty/nonempty; null/wrong types reject. Ordered system/developer/Lite base-prefix history stays intact. Bundled-version fallback/overwrite and invented todo/warmup prompt injection are removed across ingresses; factual result-error translation remains data. Typed source-native `prompt_cache_key: string` and `client_metadata: object<string,string>` preserve supplied values through initial/replayed requests; actual cache controls/usage remain when present. No cache hit/effectiveness is invented.

Responses declarations/explicit choices are flat; Chat retains actual nesting. The ordinary API-key stored-continuation and representable Chat translation subset retains its actual controls and image semantics; advanced state cannot be dropped to use it. OAuth keeps its endpoint/account/minimal-header rules, stream=true/store=false/text format invariants, and explicit unsupported-control rejects in the [route contract](contracts/chatgpt-codex-route-contract.md). Optional effort does not gate admitted summary/context/include transport. Include omission, empty, and explicit encrypted include share the same actual completed-item policy.

Actual API-key Responses serialization must preserve supplied supported `temperature` and `top_p`, including requests that select Responses through instructions or advanced state. Stop sequences reject before upstream I/O on actual Responses because that endpoint schema has no `stop` field. Ordinary accepted Chat translation keeps its stop/sampling controls, and OAuth keeps its existing unsupported-control rejects; no silent stripping or route substitution is an equivalent result.

The provider's semantic parser serves both sync and streaming. Authoritative complete output-item done replaces provisional opaque state and preserves exact opaque bytes plus required upstream `summary_text` Vec; empty/nonempty summaries remain intact. Private reasoning content/private metadata is excluded from text, logs, traces, errors, and other endpoint outputs. Safe projections are separate from wire/history serde.

An upstream complete function-call item may omit lifecycle status or supply `status: "completed"`. The completed-item parser must validate a supplied status and normalize it away before closed native replay: pinned native FunctionCall retains exact identity/kind/namespace/raw arguments and history, but does not retain that envelope lifecycle member. Null, wrong-type, unknown, and noncompleted status at a complete-item boundary fail. This is a narrow upstream-envelope normalization, not an open schema or new durable lifecycle field.

Responses emits the assembled public response/usage and optional `end_turn` at completion. Absent/null becomes omission, false/true remain explicit, wrong types fail independently of stop reason. Other serializers consume the internal terminal. False cannot become conversational `Final`; errors/incomplete/EOF/cancel cannot manufacture completion. First terminal fuses the stream, and drop releases upstream ownership without detached drain/retry. Responses rejects automatic redirects safely; this and the destination pin do not establish DNS/IP/TLS/world confinement.

Known excluded forms fail explicitly: standalone search/images, WebSockets, audio/file references, private controls/passthrough, final-answer JSON schema, unlisted built-ins, and citation fidelity. No model/auth/tool-mode downgrade or truncation hides these populations. This is a closed model-Responses HTTP/SSE component, not full ordinary native `0.156.1` or private-backend support.

### API, serialization, and resource costs

Public Rust request/response fields and typed tool/item variants change struct literals and exhaustive matches; direct callers must adopt the shared types. Normalized transport Clone/serde preserves admitted instructions, cache fields, call/result/history, opaque state, and terminal metadata. `GatewayRequest.responses_caller` is serde-skipped adapter-origin bookkeeping, set again by the Responses adapter, never caller metadata/authentication/native identity. No dependency version change is required. The shared `ImageDetail` adds the admitted Responses original detail. `KnownContentBlock::Image` uses `core_image_source` serde to retain the same authoritative `ImageSource.detail` through `Message`/`GatewayRequest` history reload. Direct `ImageSource` serde still omits/ignores detail; endpoint serializers preserve their own vocabularies.

Normalized Responses result arrays use the distinct `ToolResultContent::ResponseItems { items }` object shape (`{"items": [...]}`) so serde reload cannot reinterpret them as generic Anthropic result blocks. Public native Responses result arrays remain arrays; ordinary Anthropic empty `[]` retains its endpoint-owned meaning. Discovered definitions likewise use their typed wrapper. These are normalized/public Rust interface costs, not migration profiles or a second history store.

Typed completed opaque/summary return is an explicit HTTP disclosure/admission surface to the same connected Responses caller under the actual access boundary. Native remains the durable history owner. No gateway persistence/key/session registry or stronger cross-request/member/ciphertext attestation is introduced; actual managed access checks remain, and standalone network access is not that stronger assurance.

Engineering ceilings are 2 MiB encoded entity/SSE frame/semantic state, 16 MiB cumulative upstream wire, and 64 MiB accounted simultaneous working payload with reserved partitions (6 MiB normalized bundle, 8 MiB public queue). [Transport constants](../src/models/response_transport.rs), bounded preparse/pre-extension paths, held state/copy/queue accounting, and one-frame-per-poll framing define the mechanism. These figures do not bound RSS, headers/TLS/allocator/concurrency or establish upstream maxima/normal native-history fit. Over-limit handling fails safely without lost-history success, ciphertext rewriting, fallback, or detached drain. Final source accounting and boundary tests own proof of the implementation. Mapping selection must borrow configuration entries instead of cloning every mapping String, including unused entries. Chosen model/provider identifier payload must be admitted before owned copies or directly reached payload-bearing selection diagnostics; routing priorities, forcing, ordinary fallback, and the advanced one-destination pin retain their contracted behavior.

### ChatGPT Codex route maintenance

Use the [route contract](contracts/chatgpt-codex-route-contract.md), [auth handoff](contracts/chatgpt-codex-auth-handoff-contract.md), and existing [drift guard](contracts/chatgpt-codex-conformance-and-drift-guard.md). Historical probes and old fixtures retain their dated population; neither public-current docs nor new local source tests establish private OAuth acceptance.

### Minimal example

```json
{
  "model": "gpt-4.1-compatible",
  "input": "Hello",
  "stream": false
}
```

## Conformance Coverage

The implementation has these targeted offline conformance surfaces (their presence is not a claim that current candidate gates have passed):

- [gateway/tests/openai_chat_completions_conformance.rs](../tests/openai_chat_completions_conformance.rs)
- [gateway/tests/openai_responses_conformance.rs](../tests/openai_responses_conformance.rs)
- [gateway/tests/openai_shared_parity.rs](../tests/openai_shared_parity.rs)
- [gateway/tests/openai_codex_0156_roundtrip.rs](../tests/openai_codex_0156_roundtrip.rs)

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

Provider request regressions require flat function definitions (`type`, `name`, optional `description` and `parameters`) on upstream Responses requests and nested `function` definitions on upstream Chat Completions requests. Keep these serializers separate when maintaining tool support.

The earlier A06 evidence covered the following original scope; the new component adds admitted Responses original-detail and typed transport-serde behavior rather than treating that evidence as proof of them.

The shared Rust `ImageSource` in that earlier scope had `detail: Option<ImageDetail>` (`Low`, `High`, `Auto`). Rust struct-literal callers must add `detail: None` when no control was supplied. Request moves and clones preserve the annotation, and OpenAI endpoint serializers emit it explicitly. The historical A06 serde limitation applies to direct `ImageSource` serialization/deserialization, whose canonical Anthropic-shaped DTO still omits/ignores detail. Current core image blocks use `core_image_source` serde to retain that same field through `Message`/`GatewayRequest` history reload; the historical direct-DTO limitation is not a current core-history limitation. `AnthropicCompatibleProvider` clears the OpenAI-only annotation in endpoint-owned request/count-token projections for message, system, and result images; Chat and Gemini retain their actual wire semantics. Safe traces remain separate from authoritative history. Public Chat Completions and Messages input vocabularies are unchanged.

The `image_detail_survives_core_history_reload_and_actual_responses_wire` regression in `openai_codex_0156_roundtrip` covers absent/auto/low/high/original detail with URL/data images through public HTTP and reloaded core history sent by the actual provider. Existing `image_detail_provider_isolation` literal regressions retain their endpoint-isolation role. These are source/test anchors, not a claim that final candidate gates have passed.

The `gwa06_*` tests check exact independently expected local HTTP bodies for high, low, auto, and omission with URL/data inputs across both OpenAI protocols and sync/stream dispatch, plus typed ingress/clones and rejection before I/O. They also capture actual Anthropic-compatible sync, stream, and count-token requests and Gemini sync/stream requests, verifying unchanged image bodies without OpenAI detail leakage. OAuth serializer coverage is explicitly **unsent**; local HTTP tests prove request translation, not live vision processing, private backend acceptance, native Codex compatibility, or installed/production behavior.

The request regressions serialize an OAuth provider instance without sending and inspect real API-key provider requests against local HTTP mocks in sync and streaming modes. They verify outbound tool definitions and controls, including the existing continuation route selection and ordinary Chat fallback. They do not establish private OAuth backend acceptance, live provider compatibility, native Codex compatibility, or API-key Responses streaming output fidelity.

The accepted A03–A05 evidence established the following earlier ordinary subset; new typed/default-opaque/history behavior requires the component regressions above.

Responses streams select their decoder from the actual upstream endpoint for both API-key and OAuth requests. Supported item, text, content-part, and function-argument done payloads reconcile the same semantic assembly used for sync drains. Streaming emits each missing suffix once before closing its normalized block; a final payload that contradicts already emitted content fails as transport drift. OAuth completion envelopes supply lifecycle/usage only. The ordinary API-key completed-snapshot path remains subject to terminal validation; it cannot bypass authoritative done requirements for observed opaque items.

The shared SSE adapter frames bytes before UTF-8 decoding, including characters split across transport chunks. Invalid or truncated UTF-8 fails explicitly. Its exported Rust `SseStream` helper now yields the existing `ProviderError` rather than `reqwest::Error`: transport errors use `HttpError`, and invalid bytes use `ApiError` with status `502`. The two OpenAI consumers propagate that error; public HTTP schemas remain the same. Streams end on their first error or semantic completion, and caller drop releases the owned upstream stream without creating a completion event.

The `gwa03_*`, `gwa04_*`, and `gwa05_*` regressions cover exact public/normalized text and arguments, semantic done/delta reconciliation, real API-key Responses and Chat requests against local HTTP servers, continuation from observed function-call provenance, all byte splits around representative 2/3/4-byte characters, malformed input, error/EOF, and component drop/terminal polling. OAuth-shared parser fixtures are unsent; local adapter tests do not establish live OAuth/private-backend acceptance, native Codex compatibility, installed behavior, or retained production behavior.

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

OpenAI token counting remains unexposed; only `/v1/messages/count_tokens` exists. The closed component exclusions and native/private/installed/retained/production proof ceilings above remain material limits.

The `openai_codex_0156_roundtrip` target exercises the real local handler/core/router/OpenAIProvider path with independent literal requests/events, tool/result/next/replayed history, Lite instructions, strict/custom/namespaces/discovery, default completed opaque/summary, typed cache fields/usage, terminal states, safe sinks, limits, redirects/fallback, and cancellation. Existing provider suites protect actual Chat nesting, Anthropic caching/signed thinking, Azure, Kimi, and Gemini semantics. No pending test count or clean/live-provider result is asserted here. Pinned source evidence is `openai/codex` `rust-v0.156.1` commit `b412ff32c417f855c2b2d1581b77058eed87c84b`, tree `0a0b0bf9acf2f45e9239e7a52ef79f7702aa5799`; this is not binary/runtime authenticity. Current [OpenAI reasoning guidance](https://developers.openai.com/api/docs/guides/reasoning) describes default encrypted continuation with store=false/ZDR and optional explicit include, not private backend acceptance. Local cache preservation is not cached-token/latency effectiveness proof.

## Managed standalone search component

The managed E3 router additionally registers exactly `POST /v1/alpha/search` behind the
same connection authority and three-field member middleware as Responses, before reading
any body. The standalone router does not register search. `GatewayHttpSurfaceV1` still
identifies the exact model route; the newly admitted immutable gateway artifact binds this
fixed additional route. Older artifact identities do not acquire it automatically.

Search snapshots the configured default model's first mapping and holds that exact provider
instance. The request model must equal the mapping's actual inference slug and the provider's
admitted model list. It is forwarded unchanged; member aliases, prompt routing, provider/auth/
account/base headers, Origin, retries and fallback are not used. Only the selected integrated
Codex OAuth credential can construct the exact private destination
`https://chatgpt.com/backend-api/codex/alpha/search`; redirects are refused. The gateway does
not resolve command URLs or fetch images, files or result references itself.

The closed pinned `b412ff32c417f855c2b2d1581b77058eed87c84b` command/settings vocabulary,
empty commands, optional reasoning, body ID, ordered native message input and named opaque
response fields are handled separately from Responses IR. Unknown/unsupported fields and
explicit null request options are rejected. The input array population is the Message items
emitted by pinned `recent_input`; unrelated general ResponseItem variants are not admitted.
The exact native `internal_chat_message_metadata_passthrough` fields are preserved only on
Message items: turn_id, create_time, content_item_kinds, cell_id, executed_tool_calls and
tool_calls_complete. Each call has typed name and tool_result_sources[{type,id}], with opaque
JSON authorized only at arguments and tool_result_metadata. Explicit null is retained at those
opaque leaves; other typed optional request fields omit None and reject null. Native
truncation/omission markers pass unchanged. Received metadata is untrusted transport data,
never verified host provenance, execution/completion evidence, identity, authorization,
routing, headers or executable instructions. Native skip_deserializing does not apply to this
transport decoder; no field is discarded to imitate native harness trust semantics.

Search IDs/reference IDs are body data, with no new remote-state isolation or durability claim.

Limits are 2 MiB request wire, 16 MiB cumulative response wire, four owned concurrent permits,
10 seconds connect and 60 seconds for request read through public serialization. A conservative
64 MiB logical payload preflight charges three wire representations, 256 bytes per JSON token/
node and 8 MiB for the outbound request plus fixed metadata. It runs before typed/opaque
parsing; bounded serialization checks before copying. Dense trees can fail below the wire
limit. This accounts application payloads, not allocator capacity, HTTP/TLS buffers or RSS.
The copy graph is request chunks -> bounded raw -> typed request -> owned HTTP bytes, then
response chunks -> bounded raw -> typed result -> bounded public bytes. Raw aggregates are
dropped before the next public serialization. The permit stays owned by the response body
until consumption/drop, including after the single data chunk is yielded. Handler cancellation,
deadline or errors drop network futures, buffers and permits with no detached drain.

Failures use fixed safe envelopes; raw prompts, upstream errors, credentials, opaque results
and encrypted output are not logged or used in error text. One gateway send is not exactly-once:
the pinned native client may retry a source request up to five times. Disconnect/deadline drops
local work and cannot prove remote cancellation, rollback or absence of billed work.

Local protected-handler/actual-provider/HTTP tests prove component transport, reference
continuation, limits and cleanup with synthetic credentials. Exact production OAuth request
construction is separately **UNSENT**. Neither establishes actual private-service entitlement,
real pinned native execution, .156 renderer/artifact adoption or broad E3 production success.
Image operations, actor issuance, native/UAA execution and lifecycle remain outside this packet.
