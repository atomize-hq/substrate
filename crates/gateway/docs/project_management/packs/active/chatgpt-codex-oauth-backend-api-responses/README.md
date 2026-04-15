# ChatGPT Codex OAuth Backend-API Responses - seam extraction

Source: `docs/adr/0010-route-chatgpt-codex-oauth-via-backend-api-codex-responses.md`, `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`, and current gateway OAuth/provider/auth surfaces under `crates/gateway/src/`

This pack captures seam briefs, authoritative threading, pack-level review surfaces, seam-exit intent, and governance scaffolds. It is intentionally one level above seam-local decomposition.

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading: `threading.md`
- Pack review surfaces: `review_surfaces.md`
- Governance: `governance/remediation-log.md`

Execution horizon:

- Active seam: none
- Next seam: none

`SEAM-1`, `SEAM-2`, and `SEAM-3` are landed and have moved out of the forward window. `THR-16` is now published as the maintenance-facing drift-guard baseline, and the pack currently has no active or next seam.

Policy:

- no seam is currently eligible for authoritative downstream sub-slices because the forward planning window is empty
- a future next seam may later receive seam-local review + slices, and only provisional candidate-subslice hints
- active and next seams must eventually terminate in a dedicated final `S99` `seam-exit-gate` slice once seam-local planning begins
- seams that own undefined contracts may reserve `S00` as a contract-definition boundary slice once seam-local planning begins
- future Codex-route maintenance should consume `THR-16` instead of reopening this pack's completed conformance seam
- canonical contract docs are reserved under `crates/gateway/docs/contracts/` for this pack and must remain descriptive-only

Scope and assumptions restated before extraction:

- the ADR's scope is to make ChatGPT Codex OAuth a dedicated upstream transport contract over `https://chatgpt.com/backend-api/codex/responses`, not a generic OpenAI Responses variant
- the critical path is contract-first: the route contract and the Substrate-owned auth-handoff contract must become named verification surfaces before downstream implementation slices are considered ready
- public ingress remains thin over the normalized core: `/v1/messages`, `/v1/chat/completions`, and `/v1/responses` stay public surfaces, while provider-side transport changes are isolated below them
- the repo now reserves descriptive canonical contract targets under `crates/gateway/docs/contracts/`, starting with the ChatGPT Codex route contract owned by `SEAM-1`

Practical question this pack is intended to answer:

- what exact seams must land so ChatGPT Codex OAuth requests can use the verified `backend-api/codex/responses` route with correct stream-native behavior, Substrate-compatible auth ownership, and deterministic drift guards
