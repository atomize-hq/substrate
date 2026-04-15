---
pack_id: chatgpt-codex-oauth-backend-api-responses
pack_version: v1
pack_status: extracted
source_ref: docs/adr/0010-route-chatgpt-codex-oauth-via-backend-api-codex-responses.md plus current gateway OAuth/provider/auth surfaces in crates/gateway/src/
execution_horizon:
  active_seam: SEAM-1
  next_seam: SEAM-2
---

# Scope Brief - ChatGPT Codex OAuth Backend-API Responses

- **Goal**: route ChatGPT Codex OAuth traffic through a dedicated `backend-api/codex/responses` transport contract that preserves the gateway's normalized-core architecture, assembles results from semantic stream events, and keeps Substrate as the authoritative owner of integrated auth-state delivery.
- **Why now**: the current gateway is close enough to route ChatGPT Codex OAuth traffic, but still has material drift from the live upstream contract: sync and streaming hit different endpoints, the serializer still emits unsupported generic Responses fields/shapes, the sync parser trusts the wrong terminal envelope, and account identity still depends on JWT parsing instead of an explicit owner line.
- **Primary user(s) + JTBD**:
  - Gateway callers using `/v1/messages`, `/v1/chat/completions`, or `/v1/responses` want the selected ChatGPT Codex OAuth route to behave predictably for streaming, tool continuation, and image-bearing message turns without exposing route-specific quirks at public ingress.
  - Gateway and Substrate operators want an integrated deployment posture where account identity and access-token delivery stay policy-gated and in-world-compatible, rather than depending on host-local auth file reads inside the gateway runtime.
  - Gateway maintainers want deterministic drift guards for an undocumented upstream backend so future changes fail loudly instead of silently degrading caller-visible controls.
- **In-scope**:
  - freeze a dedicated ChatGPT Codex route contract for `https://chatgpt.com/backend-api/codex/responses`
  - freeze the route-local compatibility overlay for `pass | translate | force | reject` behavior across supported request controls
  - make the provider path stream-native for both sync and streaming callers, including semantic event assembly and sync-drain failure rules
  - freeze a Substrate-owned auth-handoff contract for integrated mode and a bounded standalone compatibility path
  - lock the route into deterministic conformance, regression, and documentation surfaces
- **Out-of-scope**:
  - redesigning public ingress or creating a second execution engine outside the normalized core
  - widening the public OpenAI-side contract beyond what ADR 0010 verifies for this route
  - exposing encrypted reasoning content as public OpenAI-visible output
  - making gateway-local token storage or local auth files into required trust inputs for Substrate-managed in-world operation
  - adding ChatGPT Codex built-in tools, non-function tools, or unverified structured-output features
- **Success criteria**:
  - sync and streaming both use `/backend-api/codex/responses` with `stream = true` and `store = false`
  - the Codex route serializer only emits the verified-supported field subset and route-local compatibility matrix, with flat tool/tool-choice shapes and typed `message` items
  - answer assembly and tool-argument assembly come from semantic stream events rather than `response.completed.response.output`
  - integrated mode resolves `ChatGPT-Account-ID` from Substrate-delivered auth context first, with JWT extraction kept as bounded fallback only
  - deterministic conformance proves request-shape gating, reasoning gating, continuation synthesis/order rules, sync-drain failure behavior, and auth-source boundaries
- **Constraints**:
  - ADR 0010 is the normative route decision
  - `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md` still forbids a second backend identity, host-local architectural assumptions, and raw-provider-stream coupling
  - the landed OpenAI-side public contracts under `docs/foundation/openai-side-*.md` remain upstream basis for public ingress behavior; this pack changes provider-side route handling only
  - the ChatGPT Codex upstream is undocumented and drift-prone, so new compatibility only lands when explicitly verified
  - canonical contract docs are reserved under `docs/contracts/` for this pack even though that tree does not yet exist in the repo
- **External systems / dependencies**:
  - `https://chatgpt.com/backend-api/codex/responses`
  - ChatGPT Codex OAuth auth material and account identity
  - Substrate host preflight, secret delivery, and in-world gateway deployment posture
  - current gateway provider/auth/server modules under `crates/gateway/src/`
  - current OpenAI-side conformance harness under `crates/gateway/tests/`
- **Known unknowns / risks**:
  - the exact integrated auth-handoff artifact and delivery surface are not yet frozen inside this repo
  - the current token store schema has no explicit `account_id`, so seam-local planning must decide how standalone compatibility resolves that gap without redefining integrated ownership
  - live upstream event ordering or accepted-field behavior may drift after the 2026-04-11 probe evidence captured in the ADR
  - sync-drain adaptation is easy to get almost-right while still returning partial or misassembled output on malformed streams
- **Assumptions**:
  - the ADR's gap list is correct: both the route contract and the auth-handoff contract must be authored before downstream implementation slices can promote
  - existing public ingress handlers remain thin adapters over the same normalized `GatewayRequest` and stream model
  - the right seam split is provider route contract first, then auth ownership, then conformance and docs
