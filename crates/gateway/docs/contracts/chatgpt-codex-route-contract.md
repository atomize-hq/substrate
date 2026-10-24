# ChatGPT Codex Route Contract

This contract freezes the provider transport and the closed typed model-Responses HTTP/SSE component described by [ADR 0010](../adr/0010-route-chatgpt-codex-oauth-via-backend-api-codex-responses.md) and [C-11](../foundation/openai-side-responses-c11-contract.md). It uses the existing core/router/OpenAIProvider, not a raw proxy or separate native driver/executor.

## Scope and ownership

The component covers declarations, model calls, native/client results, subsequent requests, and serialized-history replay for the admitted pinned `rust-v0.156.1` source population (commit `b412ff32c417f855c2b2d1581b77058eed87c84b`, tree `0a0b0bf9acf2f45e9239e7a52ef79f7702aa5799`). Native/client owns tool execution, protocol follow-up, durable history, and cancellation. Substrate retains security/custody and auth-handoff ownership.

This does not qualify full native `0.156.1`, private OAuth acceptance, source-binary authenticity, installed/retained/production behavior, or F14. Standalone search/images, WebSockets, audio/file references, private controls, final-answer schema, other unlisted built-ins, and citation fidelity remain excluded and fail explicitly. The gateway does not change model/auth/tool mode to hide exclusions.

## Transport and account contract

- Both sync and stream target `https://chatgpt.com/backend-api/codex/responses`.
- Always send `stream = true` and `store = false`. Sync drains the same incremental semantic SSE assembly used by streaming.
- Required outbound headers remain `Authorization: Bearer <access_token>`, `ChatGPT-Account-ID: <account_id>`, and `Content-Type: application/json`. Omit `OpenAI-Beta`, `originator`, and browser-like parity headers unless separately revalidated.
- Preserve the [auth-handoff contract](chatgpt-codex-auth-handoff-contract.md): integrated resolved account context owns account ID, bounded JWT fallback remains subordinate, and provider credentials do not pass to native/client children. No new gateway-local token persistence or host-file trust input is introduced.
- Select and pin backend/endpoint, resolved account/credential context, and actual model within the request. Unsupported advanced combinations fail before I/O. No cross-context fallback, stripped-state retry, or automatic Responses redirect; 3xx fails safely. This does not prove DNS/IP/TLS/world confinement or cross-request ciphertext provenance.

## Compatibility matrix

### Force

- `stream = true`
- `store = false`
- `text.format.type = "text"`; non-text requests reject

### Translate

- Named bare-message input converts to typed `message` items while preserving roles/content/order; typed outbound form is canonical.
- Admitted inline image URL/data content preserves detail (absent/auto/low/high/original) in the closed Responses population. No fetching, uploading, or host-local path conversion.
- Existing normalized ordinary function provenance may supply a missing upstream call only from actual ID/name/arguments, immediately before its matching result; no invented placeholders.
- Actual cross-API result-error flags become factual typed result data, without invented tools-disabled/warmup instructions.
- Sync uses the same SSE transport and semantic parser as streaming.

### Pass

- Supplied `instructions` preserves missing/empty/nonempty; null/wrong types reject. Preserve ordered system/developer history and native Lite developer additional-tools/base-prefix placement and IDs. No bundled-version fallback/overwrite or todo/warmup prompt injection on any ingress.
- `prompt_cache_key: string` and `client_metadata: object<string,string>` are explicit typed fields, source-mandatory for the pinned native producer. Preserve supplied values on initial/subsequent/replayed requests. They are distinct from generic `metadata`. Preserve actual cache controls/usage when present, without fabricating cache hits.
- Optional `reasoning.effort`: `none`, `minimal`, `low`, `medium`, `high`, `xhigh`.
- Independent admitted `reasoning.context`: `current_turn`, `all_turns`.
- Independent `reasoning.summary`: `auto`, `concise`, `detailed`; `none` translates to omission. Summary and encrypted continuation do not require non-none effort solely for transport.
- `include`: omission, `[]`, or `["reasoning.encrypted_content"]`. All three use the same actual returned completed-item validation, recipient, and budget policy. Other entries/duplicates/mixed sets reject.
- `parallel_tool_calls`: preserve true/false and omission.
- `text.verbosity`: `low`, `medium`, `high`.
- Flat admitted function/custom/namespace/client-search/hosted WebSearch definitions and choices; exact declaration/member order, strict false versus omission, defer flags, kind/IDs/namespaces/raw versus structured payloads, loaded definitions, and Lite history under C-11. Pinned native `output_schema` is serde-skipped, not a wire field.
- `tool_choice = "none"`, `"auto"`, or admitted flat explicit selection; no nested Responses aliases.

### Reject

- `max_output_tokens`, generic `metadata`, `truncation`, `previous_response_id`, `temperature`, `top_p`, `user`, unverified `service_tier`, and `stream_options`
- nested Chat Completions-style Responses tools/choice and `tool_choice = "required"`
- unknown typed tool/item forms, recognized unsupported native controls/media/private content, and unrepresentable selected-provider combinations

Valid public controls unsupported on this route fail before I/O. They are not silently stripped, ignored, or degraded. Current public API/source support is not private-backend entitlement or acceptance proof for newly admitted forms.

## Continuation and semantic assembly

Function/custom/client-search result IDs must match an earlier authoritative same-kind call in the request; supplied name/namespace must agree. Missing optional call IDs remain absent/inert and cannot anchor results. Preserve exact raw function JSON Strings, custom Strings, structured discovery arguments, Text versus ContentItems results, encrypted result members, and history order. Identical repeated complete calls may coalesce at their first authoritative position; conflicts reject. Do not regenerate valid caller-supplied calls or sort history into a prelude.

Typed loaded definitions return from the native client and affect the next request without a gateway registry. History-only compaction/context-compaction retains its exact admitted encrypted payload, position, and optional ID; null/wrong forms/private triggers reject. Hosted WebSearch activity remains upstream activity, not client execution or standalone search.

One provider parser assembles `response.output_item.*`, applicable content-part/text/function-argument events, and admitted completed typed items. `response.completed` supplies lifecycle/usage; it is not an assumed answer snapshot. Authoritative complete `response.output_item.done` replaces provisional added opaque state without concatenation.

Completed reasoning preserves exact opaque bytes, ID/order, and required upstream-labelled `summary_text` Vec. Empty stays empty; nonempty summaries remain intact. Approved items return as typed Responses continuation to the same connected Responses caller under the actual access boundary. Include omission/empty/explicit and optional effort do not change that policy. Clone and normalized transport serde preserve history; safe diagnostic projections do not substitute for it.

Private `Reasoning.content`, `reasoning_text`, and private metadata never become public text, arguments, annotations, Debug/Display, traces, logs, or errors. Other public endpoints do not acquire opaque OpenAI state. Adapter-owned serde-skipped `responses_caller` is endpoint-origin bookkeeping, not generic metadata, authentication, native identity, or ciphertext attestation. No new persistence, key, ledger, or session/member binding is claimed; native remains the durable owner. Managed access checks remain required; standalone network access does not establish stronger identity/isolation.

## Terminal and failure semantics

- Sync success requires valid `response.completed`. Malformed/truncated/error/incomplete/oversize/EOF-without-terminal fails; no partial success.
- Preserve completed `end_turn` independently of `stop_reason`: absent/null becomes None/omission, false/true remain explicit, wrong type fails. Core/normalized terminal carries it; other endpoints consume the internal terminal. False cannot produce a misleading conversational `Final`; true/absence cannot clear pending native follow-up.
- Before headers, provider failures use safe `502 transport_drift` as applicable; malformed ingress/limits use safe 400 route errors. After SSE starts, fail/close without a successful completion terminal.
- First valid terminal fuses the stream; caller cancellation/drop releases owned upstream state, with no detached drain/retry. No rollback or at-most-once remote-execution claim follows.

The component's engineering ceilings are 2 MiB entity/frame/semantic state, 16 MiB cumulative wire, and 64 MiB accounted simultaneous working payload with reserved partitions (6 MiB normalized bundle, 8 MiB public queue). Bound before parse/extension and account held state/copies/queues; never truncate or rewrite ciphertext/history. These are not upstream limits, RSS/concurrency bounds, or empirically validated normal native capacity. See [typed transport constants](../../src/models/response_transport.rs) and C-11.

## Verification anchors and proof ceiling

- [Provider](../../src/providers/openai.rs), [framing](../../src/providers/streaming.rs), [public adapter](../../src/server/openai_responses.rs), and [core](../../src/core.rs)
- [Typed transport](../../src/models/response_transport.rs) and [complete component regressions](../../tests/openai_codex_0156_roundtrip.rs)
- [Responses conformance](../../tests/openai_responses_conformance.rs), [shared parity](../../tests/openai_shared_parity.rs), and existing Codex fixtures

Independent local literal HTTP/SSE evidence must cover actual adapter routing, exact tool/result/next/history, Lite/non-Lite instructions, source-mandatory cache fields, completed opaque summaries/default returns, safe sinks, terminal states, boundaries/copies, redirect/fallback refusal, and cancellation. Existing provider/caching regressions remain required. OAuth serializer-only evidence stays UNSENT; historical April probes retain their original scope and do not prove current native/private acceptance. Local cache-byte/semantic preservation is not real cache effectiveness.
