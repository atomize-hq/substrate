---
seam_id: SEAM-1
seam_slug: chatgpt-codex-route-contract-and-stream-native-transport
status: landed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-chatgpt-codex-route-contract-and-stream-native-transport.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-2-closeout.md
    - crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-3-closeout.md
  required_threads: []
  stale_triggers:
    - the live ChatGPT Codex backend changes accepted fields, required headers, or semantic event ordering relative to the 2026-04-11 probe evidence captured in ADR 0010
    - the normalized `GatewayRequest` or stream model changes in a way that invalidates the route-local compatibility matrix or continuation provenance assumptions
    - public OpenAI-side contracts widen or tighten supported controls in a way that forces this route to reinterpret `pass`, `translate`, `force`, or `reject`
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---
# SEAM-1 - ChatGPT Codex Route Contract And Stream-Native Transport

## Seam Brief (Restated)

- **Goal / value**: freeze the ChatGPT Codex OAuth provider route as one explicit upstream transport contract so the gateway can preserve caller-visible semantics while speaking the verified `backend-api/codex/responses` subset.
- **Type**: `integration`
- **Scope**
  - **In**:
    - one upstream endpoint for sync and streaming: `https://chatgpt.com/backend-api/codex/responses`
    - the route-local `pass | translate | force | reject` compatibility matrix
    - typed `message` item emission, image-part translation, flat function-tool serialization, and flat explicit `tool_choice`
    - semantic stream assembly from `response.output_item.*`, `response.content_part.*`, `response.output_text.*`, and `response.function_call_arguments.*`
    - sync-drain failure posture, including mandatory `response.completed` and `502 transport_drift` on malformed or truncated drains
    - route-local reasoning gating, continuation synthesis rules, and the rule that encrypted reasoning stays non-public on this route
  - **Out**:
    - widening public ingress beyond the landed OpenAI-side contract basis
    - built-in tools, structured-output expansion, or generic Responses fields not explicitly revalidated for this route
    - auth-handoff ownership beyond consuming a resolved auth context and injecting `ChatGPT-Account-ID`
- **Touch surface**:
  - `crates/gateway/src/providers/openai.rs`
  - `crates/gateway/src/providers/streaming.rs`
  - `crates/gateway/src/server/openai_responses.rs`
  - `crates/gateway/src/models/mod.rs`
  - `crates/gateway/tests/openai_responses_conformance.rs`
  - `crates/gateway/tests/openai_shared_parity.rs`
  - `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md`
- **Verification**:
  - for the owned route contract, pre-exec readiness means the endpoint/header rules, compatibility matrix, continuation legality, semantic event families, sync-drain failure behavior, and reasoning-visibility rule are concrete enough to implement without guesswork
  - the canonical contract note and slice set must make explicit what is passed, translated, forced, or rejected for this route
  - pre-exec readiness does not require landed implementation or post-exec publication evidence; those remain seam-exit concerns
  - deterministic provider-focused verification anchors must be named for sync/stream parity, field rejection posture, continuation ordering, and semantic assembly from the same upstream event source
- **Canonical contract refs**:
  - `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md`
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**:
    - `crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-2-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-3-closeout.md`
  - **Required threads**: none
  - **Stale triggers**:
    - the live ChatGPT Codex backend changes accepted fields, required headers, or semantic event ordering relative to the 2026-04-11 probe evidence captured in ADR 0010
    - the normalized `GatewayRequest` or stream model changes in a way that invalidates the route-local compatibility matrix or continuation provenance assumptions
    - public OpenAI-side contracts widen or tighten supported controls in a way that forces this route to reinterpret `pass`, `translate`, `force`, or `reject`
- **Threading constraints**
  - **Upstream blockers**: none for pre-exec planning; the landed OpenAI-side public contract and conformance closeouts are already current basis for this seam
  - **Downstream blocked seams**: `SEAM-2`, `SEAM-3`
  - **Contracts produced**: `C-14`
  - **Contracts consumed**: landed OpenAI-side public ingress and conformance basis from the upstream pack
  - **Canonical contract refs**: `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream seams must consume closeout-backed truth that the route contract actually landed, the provider uses one stream-native upstream event source for sync and streaming, and the published compatibility matrix is backed by deterministic route evidence rather than ADR prose.
- **Expected contracts to publish**: `C-14`
- **Expected threads to publish / advance**: `THR-14` from `defined` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-2` if the landed minimal header contract, account-id input expectation, or continuation legality differs from plan
  - `SEAM-3` if the landed semantic event set, sync-drain failure behavior, or reject posture differs from the planned route contract
- **Expected closeout evidence**:
  - the canonical route contract note at `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md`
  - landed provider implementation and tests proving endpoint parity, minimal-header behavior, compatibility classification, semantic stream assembly, and sync-drain failure rules
  - explicit publication accounting for `THR-14`

## Slice index

- `S00` -> `slice-00-freeze-chatgpt-codex-route-contract.md`
- `S1` -> `slice-1-unify-codex-route-request-shaping-and-header-contract.md`
- `S2` -> `slice-2-deliver-stream-native-codex-transport-and-semantic-assembly.md`
- `S3` -> `slice-3-lock-route-fixtures-and-drift-guards.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
