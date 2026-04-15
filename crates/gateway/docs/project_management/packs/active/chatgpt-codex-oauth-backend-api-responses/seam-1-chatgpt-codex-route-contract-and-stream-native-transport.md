---
seam_id: SEAM-1
seam_slug: chatgpt-codex-route-contract-and-stream-native-transport
type: integration
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
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

- **Goal / value**: freeze the ChatGPT Codex OAuth provider route as one explicit upstream transport contract so the gateway can preserve caller-visible semantics while speaking the verified `backend-api/codex/responses` subset.
- **Scope**
  - In:
    - one endpoint for both sync and streaming: `https://chatgpt.com/backend-api/codex/responses`
    - the route-local `pass | translate | force | reject` compatibility matrix
    - typed `message` item emission, flat function tool serialization, flat explicit `tool_choice`, and ordered `function_call` + `function_call_output` continuation handling
    - semantic stream assembly from `response.output_item.*`, `response.content_part.*`, `response.output_text.*`, and `response.function_call_arguments.*`
    - sync-drain failure rules, including `502 transport_drift` on malformed or truncated drains
    - route-local reasoning gating and the rule that encrypted reasoning stays internal on this route
  - Out:
    - redesigning public `/v1/messages`, `/v1/chat/completions`, or `/v1/responses`
    - widening route support to unverified generic Responses fields or built-in tools
    - changing auth ownership beyond what is required to define the route contract inputs this seam consumes
- **Primary interfaces**
  - Inputs:
    - normalized `GatewayRequest` values routed to the ChatGPT Codex OAuth provider path
    - ADR 0010 route evidence and the landed OpenAI-side public contract basis
    - current provider implementation in `crates/gateway/src/providers/openai.rs`
  - Outputs:
    - `C-14` ChatGPT Codex route contract
    - explicit route-local compatibility rules for supported and rejected controls
    - provider request/response behavior that uses one stream-native upstream event source for sync and streaming
- **Key invariants / rules**:
  - sync and streaming use the same upstream endpoint and the same minimal header contract
  - `stream = true` and `store = false` are route-forced invariants
  - typed `message` items are the canonical outbound form on this route
  - continuation repair is minimal and provenance-based: synthesized `function_call` items are only allowed when authoritative normalized provenance already exists
  - `response.completed` is lifecycle and usage truth, not the assembled-answer source of truth
  - encrypted reasoning content never becomes public OpenAI-visible text on this route
- **Dependencies**
  - Direct blockers:
    - none inside this pack; the seam is first because the ADR already identified the route contract as the prerequisite gap
  - Transitive blockers:
    - later conformance work depends on the route contract freezing exact accepted and rejected controls
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - future Codex-route maintenance and any later provider-side feature work that must preserve this route's public semantics
- **Touch surface**:
  - `crates/gateway/src/providers/openai.rs`
  - `crates/gateway/src/providers/streaming.rs`
  - `crates/gateway/src/server/openai_responses.rs`
  - `crates/gateway/src/models/mod.rs`
  - `crates/gateway/tests/openai_responses_conformance.rs`
  - `crates/gateway/tests/openai_shared_parity.rs`
  - route-specific contract docs reserved under `crates/gateway/docs/contracts/`
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify the route contract is concrete enough that seam-local planning can name exact request fields, header rules, semantic event families, continuation legality, and sync-drain failure behavior.
  - Verify deterministic provider-focused tests can prove endpoint parity, field allowlist/reject posture, reasoning gating, continuation ordering, and semantic assembly from the stream event family.
  - Verify unsupported caller-visible controls fail explicitly rather than being silently stripped or degraded on this route.
- **Canonical contract refs**:
  - `crates/gateway/docs/contracts/chatgpt-codex-route-contract.md`
- **Risks / unknowns**:
  - Risk: the undocumented upstream backend drifts after the probes captured in ADR 0010.
  - De-risk plan: freeze the route contract and bind it to deterministic request/stream fixtures instead of leaving behavior implicit in provider code.
  - Risk: route work bleeds upward into public ingress or sideways into auth ownership.
  - De-risk plan: keep public ingress as consumed basis and leave auth provenance to `SEAM-2`.
  - Risk: semantic event assembly returns apparently valid but partial sync output on truncated streams.
  - De-risk plan: make `response.completed` mandatory and force `502 transport_drift` on malformed sync drains.
- **Rollout / safety**:
  - keep the route-specific logic below public ingress and inside the selected provider path
  - preserve explicit request rejection for unsupported controls
  - keep reasoning payloads internal and non-public on this route
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `future` because both auth ownership and conformance now consume the published route contract as basis and the seam is out of the forward window
  - Which threads matter most: `THR-14`
  - What the first seam-local review should focus on: route-local control classification, serializer legality, semantic event assembly, sync-drain failure posture, and keeping public ingress thin
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-14`
  - Threads likely to advance: `THR-14`
  - Review-surface areas likely to shift after landing: `R1` and `R2` will gain concrete named route/fixture anchors; `R3` may tighten around the exact auth-context input that `SEAM-1` consumes
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
