# ADR 0010: Route ChatGPT Codex OAuth via `/backend-api/codex/responses`

- Status: Proposed
- Date: 2026-04-11

## Context

The gateway already has a partially specialized OpenAI OAuth path in [gateway/src/providers/openai.rs](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/providers/openai.rs):

- OAuth-backed OpenAI providers switch their effective base URL to `https://chatgpt.com/backend-api`
- OAuth-backed requests are forced onto the Responses-family path
- the provider adds `ChatGPT-Account-ID` and Codex-flavored request defaults such as `instructions` and `store = false`

That is close to the desired ChatGPT Codex transport, but it is not yet a correct or complete implementation of the upstream contract.

### Current repo mismatches

The current implementation still has material drift from the live ChatGPT Codex backend:

1. sync and streaming do not use the same upstream endpoint
   - `send_message()` uses `/codex/responses`
   - `send_message_stream()` still uses `/responses`

2. the request serializer is still shaped like generic OpenAI Responses, not the ChatGPT Codex subset
   - it can emit unsupported fields such as `max_output_tokens`
   - it emits nested function-tool definitions (`{ type, function: { ... } }`) instead of the flat Responses tool shape
   - it has no dedicated handling for the upstream `tool_choice` shape

3. the OAuth sync parser assumes the final answer lives inside `response.completed.response.output`
   - live ChatGPT Codex streams emit the answer through `response.output_item.*`, `response.content_part.*`, and `response.output_text.*`
   - the terminal `response.completed` envelope may still contain `output: []`

4. account identity is currently derived by parsing the JWT access token
   - the stored auth material already contains a stable `account_id`
   - JWT parsing is a fallback technique, not the strongest source of truth

### Live upstream evidence

On 2026-04-11, a set of direct probes against `https://chatgpt.com/backend-api/codex/responses` using a local Codex OAuth token and `ChatGPT-Account-ID` established the following behavior:

#### Required transport posture

- `stream` must be `true`
- `store` must be `false`
- the request succeeds with only:
  - `Authorization: Bearer <access_token>`
  - `ChatGPT-Account-ID: <account_id>`
  - `Content-Type: application/json`
- the first-party parity headers currently used by this repo (`OpenAI-Beta`, `originator`, browser-like headers) are not required for basic request success

For this route, the outbound header contract is fixed to the minimal successful set:

- `Authorization: Bearer <access_token>`
- `ChatGPT-Account-ID: <account_id>`
- `Content-Type: application/json`

That same header contract applies to both sync and streaming calls.
That same header contract applies to both integrated and standalone auth modes; only the source of auth material differs.

The gateway MUST omit `OpenAI-Beta`, `originator`, and browser-like parity headers on this route unless a later route-specific ADR revalidates them as required.

#### Accepted request fields

- `model`
- `instructions`
- `input`
  - bare message items like `{ role, content }`
  - typed message items like `{ type: "message", role, content }`
- `reasoning.effort`
- `reasoning.summary`
  - probe value `auto` was accepted and normalized by the backend to `detailed`
- `parallel_tool_calls`
- `text.format.type = "text"`
- `text.verbosity`
- `include`
  - `["reasoning.encrypted_content"]` was accepted
- flat function tools
  - `{ "type": "function", "name": "...", "description": "...", "parameters": { ... } }`
- flat explicit `tool_choice`
  - `{ "type": "function", "name": "..." }`
- `function_call` plus `function_call_output` continuation items in `input`

#### Rejected request fields / shapes

- `max_output_tokens`
- `metadata`
- `truncation`
- `previous_response_id`
- `temperature`
- `top_p`
- `user`
- `service_tier = "default"`
- nested Chat Completions-style tools
  - `{ "type": "function", "function": { ... } }`
- nested Chat Completions-style `tool_choice`
  - `{ "type": "function", "function": { "name": "..." } }`
- a bare `function_call_output` item without a matching prior `function_call`

#### Streaming event shape

Live streams emit a semantic Responses event flow including:

- `response.created`
- `response.in_progress`
- `response.output_item.added`
- `response.content_part.added`
- `response.output_text.delta`
- `response.output_text.done`
- `response.function_call_arguments.delta`
- `response.function_call_arguments.done`
- `response.content_part.done`
- `response.output_item.done`
- `response.completed`

For this ADR, reasoning is enabled on this route only when `reasoning.effort` is present and its
value is not `"none"`.

When reasoning is enabled and `include = ["reasoning.encrypted_content"]`, the stream also emits
reasoning output items with encrypted payloads before the final answer item.

`reasoning.summary` does not enable reasoning by itself on this route; it only refines upstream
reasoning behavior when the enabled predicate above is already true.

This `reasoning.summary` gate is a route-local gateway normalization and validation rule for the
ChatGPT Codex transport. Official OpenAI Responses documentation grounds encrypted reasoning items
and stateless `include = ["reasoning.encrypted_content"]` behavior, but does not by itself define
this exact `reasoning.summary` predicate for this route.

For this route, streamed tool-call argument assembly depends on the Responses function-call argument event family (`response.function_call_arguments.delta` / `response.function_call_arguments.done`) that the gateway already treats as part of the supported semantic event surface.

Reasoning items remain upstream/internal transport state for this route unless a separate decision explicitly allows them to be surfaced; they MUST NOT become public OpenAI-visible reasoning text on this route merely because they appear in the upstream stream.

This upstream event surface aligns with a Responses-style semantic stream, but not with the repo's current OAuth sync parser assumption that `response.completed.response.output` contains the assembled answer.

### Supplemental implementation evidence

During refinement of this ADR, the official [`openai/codex`](https://github.com/openai/codex) repository was also consulted as implementation evidence for request shaping, auth-state handling, history compaction, and tool-continuation semantics.

That repository is evidence, not the normative source of truth for this gateway contract. When future route-shape, continuation, or auth-state decisions arise for this transport, the `openai/codex` repo should be re-consulted alongside direct upstream probes and local gateway code. In this workspace, the Deepwiki MCP server is the fastest path for targeted questions against that repo.

## Decision

When routing through ChatGPT/Codex OAuth credentials, the gateway will treat ChatGPT Codex as a dedicated upstream transport contract, not as generic OpenAI Responses with a different base URL.

### 1. Use one upstream endpoint for all ChatGPT Codex OAuth turns

For OAuth-backed ChatGPT Codex traffic, both sync and streaming execution MUST use:

- `https://chatgpt.com/backend-api/codex/responses`

The gateway MUST NOT send OAuth-backed streaming traffic to `/responses` while sending sync traffic to `/codex/responses`.

### 2. Treat the upstream as stream-native

The upstream ChatGPT Codex transport is stream-native.

- the gateway MUST always send `stream = true`
- the gateway MUST always send `store = false`
- sync callers in the gateway are satisfied by draining and assembling the upstream SSE stream into a `GatewayResponse`
- streaming callers are satisfied by transforming the upstream SSE stream into the normalized internal stream model

The gateway MUST NOT attempt a `stream = false` request against this upstream transport.

Sync SSE-drain failure rule:

- `response.completed` is mandatory before a sync call on this route can succeed
- if the upstream SSE stream is malformed, truncated, terminates without `response.completed`, or otherwise fails during sync drain, the gateway MUST fail the sync call rather than returning partial assembled output as success
- sync drain failures on this route MUST use the normal gateway error envelope with `class = "transport_drift"` and status `502`
- sync and streaming paths share the same transport-failure classification for this route, but sync collapses the failure into a terminal error instead of exposing a partial stream to the caller

### 3. Use an explicit account-identity owner line

This route-specific owner line inherits, and does not replace, the canonical boundary posture already established by:

- [ADR 0005: Present a Single Backend Identity to Substrate](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/docs/adr/0005-present-a-single-backend-identity-to-substrate.md)
- [ADR 0006: Preserve an In-World-Compatible Deployment Boundary](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md)
- [Substrate Boundary `C-05`](/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/docs/foundation/substrate-boundary-c05-contract.md)

This ADR applies those boundary rules to the ChatGPT Codex OAuth route; it does not define a second deployment model or a second public backend-identity contract.

For Substrate-managed in-world deployment, `ChatGPT-Account-ID` MUST come from the Substrate-owned auth and secret-delivery boundary rather than from gateway-local token persistence, admin state, or ad hoc host-file reads inside the in-world gateway runtime.

Integrated-mode owner line:

- Substrate owns policy-gated host credential reads, auth-state resolution, and host-to-world delivery for the auth material required by this route
- gateway bootstrap selects integrated versus standalone auth mode before provider construction, and the gateway consumes the selected auth context for the selected ChatGPT Codex OAuth provider route
- the provider request builder injects `ChatGPT-Account-ID` from that resolved auth context; it does not define the authoritative ownership of account identity for integrated mode
- integrated-mode account-id resolution order is:
  1. explicit `account_id` from the Substrate-delivered auth context
  2. JWT-derived `account_id` from the same Substrate-delivered OAuth access token, only if the explicit `account_id` is absent
  3. otherwise fail before the upstream call
- integrated mode MUST NOT depend on direct reads of `~/.codex/auth.json` or other host-local Codex auth files inside the in-world gateway runtime

Standalone compatibility mode:

- when the gateway is running outside a Substrate-managed in-world deployment, it MAY use locally available OAuth token material to resolve `ChatGPT-Account-ID`
- standalone-mode account-id resolution order is:
  1. explicit `account_id` from `~/.codex/auth.json`
  2. JWT-derived `account_id` from the same local OAuth access token represented by that auth state, only if the explicit `account_id` is absent
  3. otherwise fail before the upstream call with the normal gateway error envelope, `class = "auth"`, status `400`
- that standalone behavior is a gateway-local compatibility path subordinate to the integrated owner line above, not the trusted ownership model for integrated operation

Allowed compatibility fallback:

- if an integrated or standalone auth context lacks an explicit account id for the selected mode, JWT extraction from the OAuth access token MAY be used as a bounded compatibility fallback
- JWT extraction MUST remain non-authoritative for ownership and MUST NOT redefine the integrated owner line
- gateway-local token persistence or local auth files MUST NOT become required trust inputs for Substrate-managed operation
- if an explicit account id and a JWT-derived account id disagree, the explicit account id for the selected mode wins and the JWT-derived value MUST be treated only as lower-trust fallback material
- if the selected mode cannot resolve any account id after the allowed resolution order above, the gateway MUST fail before the upstream call with the normal gateway error envelope

### 4. Emit a ChatGPT Codex-specific allowlist of upstream fields

For this transport, the gateway MUST send only the verified-supported subset:

- `model`
- `instructions`
- `input`
  - typed `message` items
  - text content parts
  - image content parts
    - public gateway image inputs remain supported on this route
    - image inputs MUST be carried inside typed `message` items and translated into upstream image content items that use `image_url` with optional `detail`
    - supported public `detail` values on this route are `low`, `high`, and `auto`
    - host-local image-path semantics like the standalone Codex app's `localImage` input are out of scope for the gateway public contract unless a later ADR adds them explicitly
  - `function_call`
  - `function_call_output`
- `reasoning`
  - `effort`
  - optional `summary`
- `parallel_tool_calls`
- `text`
  - `format.type = "text"`
  - optional `verbosity`
- optional `include`
- flat function `tools`
- flat `tool_choice`
- `stream = true`
- `store = false`

The gateway MUST NOT forward the following generic Responses fields on this transport unless a later ADR revalidates them:

- `max_output_tokens`
- `metadata`
- `truncation`
- `previous_response_id`
- `temperature`
- `top_p`
- `user`
- nested Chat Completions-style tool or tool-choice shapes
- unverified `service_tier` overrides

When routing selects ChatGPT Codex OAuth, the gateway MUST apply a provider-conditional compatibility overlay between the normalized `GatewayRequest` and the Codex-specific upstream serializer.

That overlay MUST classify each normalized/public request control as exactly one of:

- `pass`
  - preserve the caller-visible semantics and forward the normalized value to the upstream request
- `translate`
  - reshape the request into the Codex-specific wire form while preserving the same caller-visible semantics
- `force`
  - override the normalized value because this transport has a stricter invariant than the generic OpenAI-side contract
- `reject`
  - fail the request before the upstream call because the caller-visible semantics cannot be preserved on this route

Compatibility rules for this route:

- `stream = true` and `store = false` are `force`
- bare upstream-compatible `message` objects, typed `message` items, image content parts, flat function-tool definitions, flat `tool_choice`, and sync drain over the upstream SSE transport are `translate`
- fields whose public semantics are compatible with the verified Codex subset are `pass`
- fields whose public semantics are not supported by this transport and do not have an explicitly defined translation are `reject`

Bare-message ingress rule for this route:

- bare upstream-compatible message objects are accepted at ingress on this route
- accepted bare message objects are `translate`, not `pass`
- the adapter MUST translate accepted bare message objects into typed `message` items before upstream submission
- that translation MUST preserve the same role and content semantics as the accepted bare message object
- the route's canonical outbound form remains typed `message` items only

Remaining supported-control matrix for this route:

- `reasoning.effort` is `pass`
  - allowed values: `none`, `minimal`, `low`, `medium`, `high`, `xhigh`
  - route-local enabled predicate: reasoning is enabled only when this field is present and not
    equal to `"none"`
  - any other value is `reject`
- `reasoning.summary` is `pass`
  - allowed values: `auto`, `concise`, `detailed`, `none`
  - `none` means the gateway omits the upstream summary field rather than sending a non-standard value
  - any non-`none` summary value is legal only when `reasoning.effort` is present and not equal to
    `"none"`
  - this enabled-gate for non-`none` summaries is a route-local gateway policy for the Codex route,
    not a claimed public OpenAI Responses invariant
  - `reasoning.summary` alone does not enable reasoning, reasoning-item assembly, or public
    reasoning output on this route
  - any other value is `reject`
- `parallel_tool_calls` is `pass`
  - allowed values: `true`, `false`
- `text.format.type` is `force`
  - the only supported value on this route is `"text"`
  - any non-text output format request is `reject`
  - this output-format rule does not restrict supported input modalities; image inputs remain allowed through typed `message` items as described above
- `text.verbosity` is `pass`
  - allowed values: `low`, `medium`, `high`
  - any other value is `reject`
- `include` is `pass` with restriction
  - allowed values are `[]` and `["reasoning.encrypted_content"]`
  - `["reasoning.encrypted_content"]` is legal only when `reasoning.effort` is present and not
    equal to `"none"`
  - `include = ["reasoning.encrypted_content"]` is the only predicate that intentionally requests
    encrypted reasoning items on this route
  - any other include entry, duplicate entry, or mixed include set is `reject` unless a later route-specific ADR revalidates it
- `stream_options` is `reject`
  - no `stream_options` member is part of the verified-supported ChatGPT Codex subset for this route
  - public Responses `stream_options.include_obfuscation` does not have a verified Codex-route equivalent with the same caller-visible semantics
  - the gateway MUST reject `stream_options` on this route rather than silently stripping, ignoring, or degrading it
  - future route-specific ADRs MAY widen this only by naming the accepted member(s), the preserved semantics, and the verification coverage

`tool_choice` compatibility matrix for this route:

- public `tool_choice = "none"` is `pass`
- public `tool_choice = "auto"` is `pass`
- public explicit function selection is `translate`
  - public OpenAI-side form: `{ "type": "function", "function": { "name": "..." } }`
  - Codex upstream form: `{ "type": "function", "name": "..." }`
- public `tool_choice = "required"` is `reject` unless a later route-specific ADR verifies a Codex-native equivalent with the same caller-visible semantics

The gateway MUST NOT silently collapse `tool_choice = "required"` into `"auto"` or explicit-function selection on this route.

For the avoidance of doubt, the gateway MUST NOT silently strip, ignore, or degrade a caller-visible control when ChatGPT Codex OAuth is the selected route unless this ADR or a later route-specific ADR explicitly defines that behavior.

This ADR intentionally chooses strict compatibility over silent best-effort behavior:

- if a public OpenAI-side field is valid at ingress but cannot be preserved on the Codex route, the request MUST fail explicitly instead of being forwarded with reduced semantics
- future adapters MAY widen compatibility by adding explicit route-specific `pass` or `translate` rules
- future adapters MUST NOT widen compatibility by silently changing or dropping semantics
- capability-aware rerouting onto a different provider is out of scope for this ADR and requires a separate decision

### 5. Canonicalize on typed message items for gateway emission

Although the upstream accepts both bare message objects and typed `message` items, the gateway MUST emit typed items as its canonical upstream form for ChatGPT Codex OAuth:

- `{ "type": "message", "role": "...", "content": [...] }`

Rationale:

- typed items align with the public `/v1/responses` contract already implemented in this repo
- they fit naturally with `function_call` and `function_call_output` items in the same `input` array
- they reduce ambiguity at the adapter boundary
- upstream tolerance for bare message objects does not widen the gateway's emitted wire contract for this route

### 6. Preserve tool continuation through `function_call` + `function_call_output`

Tool continuation for this upstream transport MUST use Responses-style continuation items:

1. emit `function_call` items when the model asks for tools
2. emit `function_call_output` items when the caller provides tool results
3. preserve `call_id` exactly across the round trip

Caller-supplied `function_call` rule:

- caller-supplied `function_call` items are legal on this route only in the canonical flat Responses shape:
  - `{ "type": "function_call", "call_id": "...", "name": "...", "arguments": "..." }`
- valid caller-supplied `function_call` items are preserved as the authoritative continuation item for that `call_id`; the adapter MUST NOT regenerate or replace them solely because synthesis is available
- the adapter MAY synthesize a `function_call` item only when no valid caller-supplied `function_call` item for that `call_id` is already present in the current request body
- the adapter MUST NOT emit duplicate `function_call` items for the same `call_id`

Authoritative provenance rule:

- the gateway MAY synthesize an upstream `function_call` item only when it already has authoritative normalized provenance for that same `call_id`
- authoritative normalized provenance means prior normalized `tool_use` state that already carries the tool `id`, `name`, and input payload needed to reproduce the upstream `function_call`
- when synthesis is allowed, the synthesized upstream `function_call.name` and serialized `function_call.arguments` MUST be copied from that authoritative normalized provenance rather than invented ad hoc

When the normalized gateway request contains a tool result without a corresponding prior upstream `function_call` item in the current request body, the gateway MUST synthesize the matching `function_call` item before the `function_call_output`.

If the gateway does not have authoritative normalized provenance for the referenced `call_id`, it MUST reject the continuation before the upstream call rather than fabricating placeholder tool metadata.

The gateway MUST NOT send a lone `function_call_output` item to this upstream transport.

Mixed-continuation ordering rule:

- the upstream `input` array for this route MUST preserve normalized conversation-history order as the primary ordering key
- when a `function_call_output` item requires synthesis of a missing prior `function_call`, the synthesized `function_call` MUST be inserted immediately before its matching `function_call_output`
- when a matching `function_call` item is already present earlier in the serialized request body, the gateway MUST preserve that earlier item and emit only the `function_call_output`
- when multiple tool continuations appear in one request, the gateway MUST preserve their normalized-history order; it MUST NOT reorder them by `call_id`, tool name, or synthesis status
- the gateway MUST perform only the minimum local repair needed to make each continuation legal for this upstream transport; it MUST NOT create a detached prelude or globally reshuffle the request body

### 7. Assemble answers from semantic stream events, not from `response.completed.output`

The ChatGPT Codex adapter MUST treat the event stream as the source of truth.

Specifically, it MUST assemble output from:

- `response.output_item.added`
- `response.output_item.done`
- `response.content_part.added`
- `response.content_part.done`
- `response.output_text.delta`
- `response.output_text.done`
- `response.function_call_arguments.delta`
- `response.function_call_arguments.done`
- reasoning items only when `reasoning.effort` is present and not `"none"` and
  `include = ["reasoning.encrypted_content"]`

`response.completed` remains the terminal lifecycle event and usage source, but MUST NOT be assumed to contain the full assembled `output` payload.

Route-local reasoning visibility rule:

- encrypted reasoning items are internal transport state only on this route
- the gateway MUST NOT convert encrypted reasoning items into public OpenAI-visible reasoning text,
  public `thinking` blocks, public `message.content` parts, or public annotations on this route
- `reasoning.summary` does not widen that visibility rule; it is request-shaping state only unless a
  later route-specific ADR explicitly permits a public reasoning-summary surface

Grounding note:

- official OpenAI Responses docs validate encrypted reasoning items, stateless `store = false`
  operation, and `include = ["reasoning.encrypted_content"]` as public API behavior
- this ADR's stricter `reasoning.summary` acceptance predicate remains a route-local Codex transport
  policy derived from gateway normalization goals and route verification scope

### 8. Keep public ingress surfaces thin over the normalized core

Public ingress does not change:

- `/v1/messages`
- `/v1/chat/completions`
- `/v1/responses`

All of them continue to normalize into the shared internal request model first.
Only the provider-side upstream transport changes when the routed provider is ChatGPT Codex OAuth.

## Contract-Ready Gap List

This ADR is directionally correct, but it is not clean seam-extraction input until the remaining
route-contract ownership work below is completed.

- a dedicated gateway-owned canonical contract note under `crates/gateway/docs/foundation/` is
  authored and linked from this ADR as the normative source for the ChatGPT Codex route's
  stream/tool-loop behavior
- that contract freezes the route-specific semantic-stream assembly rules, including the event
  families that are authoritative for output assembly and the rule that `response.completed` is a
  terminal lifecycle/usage event rather than the assembled-answer source of truth
- that contract freezes the `function_call` / `function_call_output` continuation contract for this
  route, including authoritative normalized provenance requirements for any synthesized
  `function_call`
- that contract states the route-specific reasoning visibility rule so encrypted/internal reasoning
  remains non-public unless a later explicit decision widens exposure
- that contract names the verification surfaces that own implementation proof for this route,
  including the provider adapter, normalized models, and deterministic fixtures/regressions
- a dedicated Substrate-owned auth-handoff contract note is authored and linked from this ADR as
  the normative source for integrated-mode host preflight, host-to-world secret delivery, and
  gateway consumption of ChatGPT Codex auth material
- that auth-handoff contract freezes the integrated-mode handoff posture already selected by the
  surrounding ADRs and standards:
  - host-side Substrate credential preflight remains policy-gated and authoritative
  - current v1 host-to-world delivery may place the closed `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*`
    auth values directly in the in-world gateway/manager process environment
  - the preferred additive direction remains a secret-channel payload with an inherited one-time
    FD/pipe auth bundle so secret values do not live in the in-world process environment by default
- that auth-handoff contract freezes the closed `cli:codex` auth field set and names the canonical
  auth field identifiers carried through the handoff artifact:
  - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
  - `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
- that auth-handoff contract freezes integrated-mode vs standalone-mode selection and the ownership
  split for extraction, validation, delivery, and request injection, so the gateway consumes the
  delivered auth material but does not become the owner of host credential reads or trust-boundary
  decisions
- that auth-handoff contract names the verification surfaces that prove the handoff boundary,
  including host-side preparation, world/backend delivery, gateway auth-context resolution, and the
  provider request builder that injects `ChatGPT-Account-ID`

Downstream seam planning implication:

- any seam planning pack that treats this ADR as implementation input MUST include explicit
  responsibility to write, review, and finalize that dedicated route contract before implementation
  slices that depend on Codex-route stream assembly, continuation synthesis, or reasoning-handling
  behavior are considered ready
- any seam planning pack that treats this ADR as implementation input MUST include explicit
  responsibility to write, review, and finalize the dedicated auth-handoff contract before
  implementation slices that depend on integrated-mode ChatGPT Codex auth delivery, account-id
  resolution, or provider-header injection behavior are considered ready
- seam extraction and slice planning MUST treat route-contract finalization as a required planning
  deliverable, not as incidental cleanup during implementation or closeout
- seam extraction and slice planning MUST treat auth-handoff-contract finalization as a required
  planning deliverable, not as incidental cleanup during implementation or closeout

## Consequences

Positive:

- the gateway can route API requests from any supported public/client shape onto the same ChatGPT Codex backend used by first-party Codex accounts
- sync and streaming behavior stop diverging at the upstream endpoint boundary
- the provider adapter stops sending unsupported generic Responses fields to ChatGPT Codex
- tool continuation remains aligned with the upstream Responses-style item contract
- the design stays consistent with the repo's normalized-core architecture and ADR 0008 thin-adapter posture

Negative:

- ChatGPT Codex OAuth becomes a genuinely distinct upstream contract that must be maintained separately from generic OpenAI and Azure OpenAI transports
- the adapter must maintain a dedicated allowlist and not rely on generic Responses parity
- sync mode becomes a stream-drain adaptation rather than a native sync upstream request
- integrated Substrate deployment now requires an explicit auth-context handoff for ChatGPT account identity instead of relying on gateway-local token state
- additional regression coverage is required because this backend is undocumented and may drift

## Constraints On Implementation

- do not expose ChatGPT Codex-specific quirks directly at the public ingress boundary
- do not bypass `GatewayRequest` / normalized stream handling to special-case one public route
- do not send nested Chat Completions tool shapes to this upstream transport
- do not rely on `response.completed.response.output` as the assembled answer source
- do not surface encrypted reasoning content on public OpenAI-compatible routes unless a separate decision explicitly permits it
- do not silently strip, ignore, or downgrade unsupported caller-visible controls on the Codex route unless a route-specific ADR explicitly allows that behavior
- do not make gateway-local token persistence, local auth files, or admin mutation state into required trust inputs for Substrate-managed in-world operation
- do not keep or introduce placeholder synthesized `function_call` metadata such as fallback tool names or empty `{}` arguments when authoritative normalized provenance is unavailable

## Verification Boundary

Extractor-readiness prerequisite:

- this ADR is not extractor-ready on its own until the dedicated foundation-level ChatGPT Codex
  route contract described in the gap list above exists and is adopted as a named verification
  surface
- this ADR is not extractor-ready on its own until the dedicated Substrate-owned auth-handoff
  contract described in the gap list above exists and is adopted as a named verification surface
- downstream seam planning packs that consume this ADR MUST carry an explicit contract-definition
  responsibility for writing and finalizing that route contract before implementation slices are
  promoted
- downstream seam planning packs that consume this ADR MUST carry an explicit contract-definition
  responsibility for writing and finalizing that auth-handoff contract before implementation slices
  are promoted

This ADR is complete when:

1. OAuth-backed ChatGPT Codex sync and streaming both target `/backend-api/codex/responses`
2. the route-compatibility overlay classifies Codex-routed request controls as `pass`, `translate`, `force`, or `reject`, and the upstream request serializer uses the verified-supported field subset and flat tool/tool-choice shapes
3. the adapter assembles results from semantic stream events rather than the final completed envelope's `output`
4. for Substrate-managed in-world deployment, account id is sourced from Substrate-delivered auth context first, with any JWT extraction limited to bounded compatibility fallback behavior rather than ownership
5. regression coverage proves:
   - stream-only upstream behavior
   - `store = false` enforcement
   - the outbound header contract is limited to `Authorization`, `ChatGPT-Account-ID`, and `Content-Type` unless a later route-specific ADR expands it
   - accepted vs rejected upstream parameter mapping
   - no caller-visible control is silently stripped or degraded on the Codex route without an explicit compatibility rule
   - public gateway image inputs remain supported on the Codex route through typed `message` items with upstream `image_url` content items
   - `reasoning.effort`, `reasoning.summary`, `parallel_tool_calls`, `text.format.type`, `text.verbosity`, and `include` follow the route-specific compatibility matrix and reject unverified values
   - reasoning is enabled on this route only when `reasoning.effort` is present and not `"none"`
   - non-`none` `reasoning.summary` is rejected unless reasoning is enabled on this route, as a
     route-local Codex validation rule rather than a claimed public OpenAI Responses invariant
   - `include = ["reasoning.encrypted_content"]` is rejected unless reasoning is enabled on this route
   - encrypted reasoning items are assembled only under that exact predicate and never surface as public OpenAI-visible reasoning output on this route
   - `stream_options` is rejected deterministically on the Codex route unless a later route-specific ADR explicitly adds a verified equivalent
   - flat function tool serialization
   - conversational turns on the Codex route serialize as typed `message` items rather than bare message objects
   - `function_call` + `function_call_output` continuation threading
   - synthesized `function_call` items copy `name` and arguments from authoritative normalized provenance when prior upstream items are absent from the current request body
   - orphaned tool-result continuations without authoritative normalized provenance fail deterministically before the upstream call
   - placeholder synthesized `function_call` metadata has been removed from the Codex continuation path
   - mixed continuation requests preserve normalized-history order, with any synthesized `function_call` inserted immediately before its matching `function_call_output`
   - sync drain and streaming transformations over the same upstream event source
   - sync success requires terminal `response.completed`, and malformed or truncated sync drains fail with `502 transport_drift` rather than returning partial output as success
   - integrated mode does not require gateway-local token persistence or host auth-file reads to establish `ChatGPT-Account-ID`
