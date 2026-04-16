# ChatGPT Codex Conformance And Drift-Guard Contract

This document is the descriptive source of truth for deterministic conformance and drift-guard obligations on the gateway's ChatGPT Codex route.
It freezes the regression and maintenance contract for this route without redefining the upstream route behavior contract or the auth-handoff owner line.

## Scope

This contract covers:

- deterministic sync and streaming conformance obligations for the ChatGPT Codex route
- offline-first regression posture for route-local compatibility, semantic assembly, and auth provenance
- the exact caller-visible behaviors that later verification must prove or reject explicitly
- maintenance-facing revalidation triggers and evidence anchors for future route work

This contract does not own:

- redesign of public ingress behavior beyond the already-published route contract
- changes to integrated-versus-standalone auth ownership beyond the already-published auth-handoff contract
- live upstream probe automation as part of the core regression path
- broad OpenAI compatibility expansion outside the verified Codex route subset

## Contract Basis

This contract depends on two previously published canonical contracts:

- the route contract in `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md`
- the auth-handoff contract in `crates/gateway/docs/contracts/chatgpt-codex-auth-handoff-contract.md`

The route contract remains the source of truth for:

- the dedicated `https://chatgpt.com/backend-api/codex/responses` transport
- the `pass | translate | force | reject` compatibility matrix
- typed `message` shaping, flat function tool rules, continuation legality, semantic stream assembly, and sync-drain failure posture
- the rule that encrypted reasoning remains non-public on this route

The auth-handoff contract remains the source of truth for:

- the integrated-mode owner line between Substrate delivery and gateway consumption
- the bounded standalone compatibility path
- explicit-over-JWT `account_id` precedence and pre-upstream failure behavior
- the rule that integrated mode must not require host-local auth-file reads inside the gateway runtime

This contract freezes downstream verification obligations against that published basis. It must not silently widen, narrow, or reinterpret either upstream contract.

## Deterministic Suite Posture

- The core drift-guard suite stays offline and deterministic wherever possible.
- Fixture-backed and harness-backed verification is the default posture for this route.
- Live upstream network dependence is not part of the core regression path.
- Captured probe evidence may inform future refresh decisions, but the maintained suite should fail on local deterministic regressions before any live revalidation is needed.
- Sync and streaming verification must derive from the same semantic upstream event model rather than from separate route-specific truths.

## Route Conformance Obligations

Later implementation and verification for this route must prove the following caller-visible obligations from the route contract:

- later slices must implement the published route matrix exactly as frozen below rather than inferring it from current code shape alone

### Forced controls

The following controls remain fixed on this route:

- `stream = true`
- `store = false`
- `text.format.type = "text"`

### Translated controls

The following caller-visible controls remain accepted only through the published translate behavior:

- bare message objects become typed `message` items before upstream submission
- supported image inputs become upstream `image_url` content parts, with optional `detail`
- function tool definitions use the published flat shape for this route
- explicit function `tool_choice` uses the published flat shape for this route
- sync callers are satisfied through stream-drain execution over the same upstream SSE transport used by streaming callers

### Passed controls

The following controls remain passed through subject to the published route constraints:

- `reasoning.effort` with values `none`, `minimal`, `low`, `medium`, `high`, `xhigh`
- `reasoning.summary` with values `auto`, `concise`, `detailed`, `none` only when reasoning is enabled
- `parallel_tool_calls`
- `text.verbosity` with values `low`, `medium`, `high`
- `include = []`
- `include = ["reasoning.encrypted_content"]` only when reasoning is enabled
- `tool_choice = "none"`
- `tool_choice = "auto"`

### Rejected controls

The following controls remain explicit rejects on this route:

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

Unsupported caller-visible controls fail explicitly before the upstream call rather than being silently stripped, ignored, or degraded.

### Additional route obligations

- typed `message` shaping remains the canonical outbound message form
- supported image input translation remains bounded to the published typed-message and `image_url` rules
- function tools and explicit function `tool_choice` remain in the published flat shape for this route
- continuation handling preserves the published legality, synthesis, and ordering rules for `function_call` and `function_call_output`
- sync and streaming remain semantically aligned because they are derived from the same upstream SSE event family
- sync-drain success still requires terminal `response.completed`, and malformed or truncated drains still fail with the published transport-drift posture
- encrypted reasoning remains internal transport state and does not become public OpenAI-visible output on this route

## Auth Provenance Obligations

Later implementation and verification for this route must prove the following auth-source obligations from the auth-handoff contract:

- integrated mode resolves `ChatGPT-Account-ID` from Substrate-delivered auth context first
- explicit `account_id` remains authoritative over JWT-derived fallback when both exist
- JWT-derived account identity remains bounded compatibility fallback only
- unresolved account identity fails before the upstream call using the normal auth failure posture
- integrated mode does not require gateway-local token persistence or host-local auth-file reads to establish `ChatGPT-Account-ID`
- standalone local auth state remains compatibility-only and must not be treated as the integrated trust boundary
- provider-side header injection remains a consumer of resolved auth context rather than the owner of auth-source selection

## Verification Anchors

Implementation and regression evidence for this contract should land against:

- `crates/gateway/tests/openai_responses_conformance.rs`
- `crates/gateway/tests/openai_shared_parity.rs`
- `crates/gateway/src/server/openai_conformance_test_support.rs`
- `crates/gateway/docs/openai-compatibility.md`
- `crates/gateway/docs/OAUTH_SETUP.md`
- `crates/gateway/docs/OAUTH_TESTING.md`

The authoritative deterministic fixture family for this seam's route regressions is:

- `crates/gateway/tests/fixtures/openai_responses/codex-*.json`

Later slices should treat that Codex route fixture namespace as the owned fixture family for deterministic sync/stream route coverage, continuation ordering, and reject-vs-accept regression cases on this seam.

Verification for this contract must make it possible for a maintainer to answer, without reverse-engineering implementation diffs:

- which route controls are accepted, forced, translated, or rejected
- which sync and streaming assertions prove semantic parity on this route
- which assertions prove continuation legality and ordering
- which assertions prove reasoning remains non-public
- which assertions prove integrated auth provenance and bounded standalone fallback
- which docs name the same stale triggers and evidence anchors that the regressions protect

Later documentation alignment for this seam must land in:

- `crates/gateway/docs/openai-compatibility.md`
- `crates/gateway/docs/OAUTH_SETUP.md`
- `crates/gateway/docs/OAUTH_TESTING.md`

Those route-facing maintenance docs must mirror the same stale triggers and evidence anchors that the deterministic regressions protect, so future maintainers do not need to reconstruct drift conditions from code or planning artifacts alone.

## Maintenance And Revalidation Triggers

This contract should be treated as stale and reopened for review when any of the following change materially:

- the route compatibility matrix or supported control classifications in the route contract
- the semantic upstream event families, assembly rules, or sync-drain terminal requirements
- the auth-handoff owner line, field identifiers, precedence rules, or fallback constraints
- normalized-core behavior in a way that invalidates route-local fixture expectations
- fixture namespaces, conformance harness behavior, or documentation anchors in a way that obscures what the route is proving
- future route-specific controls are added or widened without explicit Codex-route revalidation

Future maintenance should consume this contract as the route's drift-guard baseline rather than rediscovering obligations from scattered tests, implementation details, or ADR prose.
