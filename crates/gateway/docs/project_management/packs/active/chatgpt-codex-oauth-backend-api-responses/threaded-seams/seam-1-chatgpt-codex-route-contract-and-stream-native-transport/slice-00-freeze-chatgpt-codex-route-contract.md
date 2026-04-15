---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - ADR 0010 changes the accepted-field matrix, minimal header contract, or semantic event family before implementation lands
    - the normalized `GatewayRequest` or normalized stream model changes such that the route compatibility matrix or continuation rules must be re-planned
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-14
contracts_produced:
  - C-14
contracts_consumed: []
open_remediations: []
---
### S00 - Freeze ChatGPT Codex Route Contract

- **User/system value**: implementation starts from one concrete owned route contract instead of rediscovering endpoint, header, field-classification, continuation, and semantic-event rules inside provider code.
- **Scope (in/out)**:
  - In: freeze the canonical route contract note, the minimal successful header contract, the `pass | translate | force | reject` matrix, typed message and image translation rules, flat tool and `tool_choice` rules, continuation legality, reasoning gating, and sync-drain failure posture.
  - Out: landed runtime evidence, downstream auth-handoff ownership, and seam-exit publication accounting.
- **Acceptance criteria**:
  - `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md` exists and is descriptive-only
  - the contract note names one upstream endpoint for sync and streaming, one minimal header contract, and one route-local compatibility matrix
  - the contract note makes continuation synthesis, semantic event assembly, reasoning visibility, and `502 transport_drift` failure posture concrete enough that implementation does not need to guess
  - the verification checklist names exact code and test anchors that later prove the route contract without relying on live upstream probes in the core regression path
- **Dependencies**: `../../threading.md`, `../../scope_brief.md`, `../../seam-1-chatgpt-codex-route-contract-and-stream-native-transport.md`, `crates/gateway/docs/adr/0010-route-chatgpt-codex-oauth-via-backend-api-codex-responses.md`, `crates/gateway/docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`
- **Verification**:
  - a reviewer can answer which controls are passed, translated, forced, or rejected, how continuation synthesis is constrained, and which event families are authoritative without inspecting implementation diffs
  - pass condition: the contract is concrete enough that `SEAM-1` satisfies `gates.pre_exec.contract` before any runtime code lands
- **Rollout/safety**: keep the route contract narrow and explicit; do not smuggle auth ownership, rollout policy, or generic Responses expansion into this contract definition.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`Likely mismatch hotspots`)

#### Frozen canonical artifacts (this slice output)

- Owned route contract: `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md`
- Normative route decision: `crates/gateway/docs/adr/0010-route-chatgpt-codex-oauth-via-backend-api-codex-responses.md`
- Boundary guardrail: `crates/gateway/docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`

#### Execution-grade freeze for the route contract

- **Endpoint and header contract**:
  - sync and streaming both target `https://chatgpt.com/backend-api/codex/responses`
  - the minimal successful header set is:
    - `Authorization: Bearer <access_token>`
    - `ChatGPT-Account-ID: <account_id>`
    - `Content-Type: application/json`
  - extra parity headers remain out of contract unless a later route-specific ADR revalidates them
- **Forced controls**:
  - `stream = true`
  - `store = false`
  - `text.format.type = "text"`
- **Translated controls**:
  - bare message objects become typed `message` items
  - image inputs remain supported only through typed `message` items with upstream `image_url` content parts
  - flat function-tool definitions and flat explicit `tool_choice` are emitted as Codex-native wire shapes
  - sync callers are served by draining and assembling the upstream SSE stream rather than by a separate sync upstream call
- **Passed controls**:
  - `reasoning.effort` with values `none | minimal | low | medium | high | xhigh`
  - `reasoning.summary` with values `auto | concise | detailed | none`, subject to the route-local enabled predicate
  - `parallel_tool_calls`
  - `text.verbosity` with values `low | medium | high`
  - `include` only as `[]` or `["reasoning.encrypted_content"]`, and only when reasoning is enabled for the latter
  - `tool_choice = "none"` and `tool_choice = "auto"`
- **Rejected controls**:
  - `max_output_tokens`
  - `metadata`
  - `truncation`
  - `previous_response_id`
  - `temperature`
  - `top_p`
  - `user`
  - unverified `service_tier` overrides
  - nested Chat Completions-style tool or `tool_choice` shapes
  - `tool_choice = "required"`
  - `stream_options`
- **Continuation rules**:
  - `function_call` and `function_call_output` use flat Responses-style items
  - a missing prior `function_call` may be synthesized only from authoritative normalized provenance for the same `call_id`
  - synthesized `function_call` items are inserted immediately before their matching `function_call_output`
  - orphaned `function_call_output` items without authoritative provenance reject before the upstream call
- **Semantic assembly rules**:
  - assembled output and tool arguments come from `response.output_item.*`, `response.content_part.*`, `response.output_text.*`, and `response.function_call_arguments.*`
  - `response.completed` is terminal lifecycle and usage truth, not the assembled-answer source of truth
  - sync success requires terminal `response.completed`; malformed or truncated drains fail with `502 transport_drift`
- **Reasoning visibility rules**:
  - reasoning is enabled only when `reasoning.effort` is present and not `"none"`
  - non-`none` `reasoning.summary` values require reasoning to be enabled first
  - encrypted reasoning items remain internal transport state and do not become public OpenAI-visible output on this route

#### S00.T1 - Freeze The Request, Header, And Compatibility Matrix

- **Outcome**: the route contract names one explicit request-shaping and header rule set for Codex OAuth.
- **Inputs/outputs**: inputs are ADR 0010, the seam brief, current provider behavior, and normalized request shapes; outputs are the contract note plus the field-classification matrix.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: make explicit which controls are passed, translated, forced, or rejected; keep `ChatGPT-Account-ID` as a consumed input rather than an owned auth decision.
- **Acceptance criteria**: endpoint parity, minimal headers, typed-message emission, flat tool shapes, and reject posture are all named with exact pass/fail rules.
- **Test notes**: name positive and negative verification cases for field shaping, header emission, typed-message translation, image inputs, and deterministic rejects.
- **Risk/rollback notes**: leaving the matrix implicit will let runtime code silently degrade unsupported controls.

Checklist:
- Implement: freeze the route matrix and name the canonical contract artifact path
- Test: enumerate positive and negative verification cases tied to the matrix
- Validate: confirm every supported or rejected field in ADR 0010 has an explicit disposition
- Cleanup: remove ambiguity about headers, images, and `tool_choice`

#### S00.T2 - Freeze Continuation, Semantic Assembly, And Failure Rules

- **Outcome**: the route contract names one concrete continuation and semantic-event rule set for sync and streaming.
- **Inputs/outputs**: inputs are ADR 0010, current normalized tool/stream behavior, and seam review findings; outputs are the continuation and semantic-assembly sections of the contract note.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: make continuation synthesis provenance-based, keep normalized-history order authoritative, and define `response.completed` as mandatory terminal lifecycle truth for sync drain success.
- **Acceptance criteria**: one reviewer can explain tool continuation legality, event-family authority, reasoning suppression, and `502 transport_drift` behavior without consulting runtime code.
- **Test notes**: name fixtures for orphaned tool results, synthesized continuation order, text/tool/mixed event streams, and truncated sync drains.
- **Risk/rollback notes**: ambiguity here will make sync and streaming drift or expose partial output as success.

Checklist:
- Implement: freeze continuation legality, semantic event authority, and sync failure posture
- Test: identify route fixtures and assertions for tool continuation, event ordering, and truncated drains
- Validate: confirm reasoning stays non-public in both sync and streaming paths
- Cleanup: remove any assumption that `response.completed.response.output` is the assembled-answer source
