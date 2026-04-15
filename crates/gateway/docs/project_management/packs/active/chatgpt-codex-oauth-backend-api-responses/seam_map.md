# Seam Map - ChatGPT Codex OAuth Backend-API Responses

This seam map extracts the route-specific work required to make ChatGPT Codex OAuth a first-class upstream transport without reopening the landed public OpenAI-side ingress design or the broader Substrate boundary posture.

Constraint posture:

- `docs/foundation/openai-side-responses-c11-contract.md`, `docs/foundation/openai-side-adapter-invariants-c12-contract.md`, and `docs/foundation/openai-side-conformance-suite-c13-contract.md` remain upstream basis and are not re-owned here
- ADR 0010 already makes the route gap explicit: this pack does not rediscover the route problem, it turns the remaining contract, auth, and drift-guard work into governance-ready seams
- `docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md`, ADR 0005, and ADR 0006 still forbid host-local auth assumptions from becoming integrated-mode architecture
- the remaining work is below public ingress: the route contract, auth-handoff ownership, and conformance proof for the selected ChatGPT Codex OAuth path

## Horizon summary

- **Active seam**: `SEAM-3`
- **Next seam**: none yet
- **Future seams**: `SEAM-1`, `SEAM-2`

The default v2.5 horizon policy is explicit here:

- only `SEAM-3` is eligible for authoritative downstream deep planning by default
- `SEAM-3` has entered active planning because both route and auth ownership are now published
- `SEAM-1` and `SEAM-2` are landed and live outside the forward window

## Seam roster

| Seam | Horizon / state | Type | Why this is a seam | Likely value | Touch surface | Verification path |
| --- | --- | --- | --- | --- | --- | --- |
| `SEAM-1` `chatgpt-codex-route-contract-and-stream-native-transport` | `future` / `landed` | `integration` | it owns the provider-side route contract, compatibility overlay, stream-native endpoint selection, and semantic event assembly as one coherent upstream boundary instead of scattering those decisions across parser, serializer, and SSE handling | ChatGPT Codex OAuth stops being a near-match and becomes a deterministic upstream transport the gateway can trust for sync, streaming, tools, images, and reasoning gating | `crates/gateway/src/providers/openai.rs`, route-facing docs under `crates/gateway/docs/`, conformance harness anchors in `crates/gateway/tests/` | seam-local review can freeze the contract, prove endpoint/header/body rules, and make sync/stream share one semantic event source |
| `SEAM-2` `substrate-auth-handoff-and-account-id-provenance` | `future` / `landed` | `integration` | it owns the trust boundary for `ChatGPT-Account-ID` and access-token delivery, separating integrated Substrate ownership from standalone compatibility fallback instead of leaving account identity implicit inside provider code | in-world deployments can consume delivered auth material without gateway-local host reads, while standalone mode remains a bounded fallback instead of the architectural truth | `crates/gateway/src/auth/*`, `crates/gateway/src/server/oauth_handlers.rs`, `crates/gateway/src/providers/openai.rs`, OAuth docs, and Substrate-facing contract notes | closeout-backed auth-handoff truth now publishes `THR-15` for downstream conformance work |
| `SEAM-3` `codex-route-conformance-and-drift-guards` | `active` / `exec-ready` | `conformance` | it turns the route and auth decisions into durable deterministic evidence instead of relying on ADR prose or live one-off probes | future edits fail against fixtures and route-local regressions before caller-visible drift ships | `crates/gateway/tests/*`, `crates/gateway/src/server/openai_conformance_test_support.rs`, `crates/gateway/docs/openai-compatibility.md`, and route-specific contract notes | closeout-backed tests and docs prove the route matrix, sync/stream parity, auth-source rules, and no-silent-degradation posture |

## Ordering rationale

1. `SEAM-3` is active because `SEAM-1` and `SEAM-2` have both published the route and auth truth it consumes.
2. `SEAM-3` is `exec-ready` because the conformance seam now has current upstream basis, published inbound threads, and concrete seam-local review plus slice planning.
3. `SEAM-1` and `SEAM-2` are future because they are already landed and now sit outside the forward window.

## Non-seams and pruned candidates

- A new public `/v1/responses` seam was rejected because the landed OpenAI-side public contract already owns that surface; this pack changes the routed provider boundary below it.
- A separate "tool continuation" seam was rejected because continuation synthesis, ordering, and semantic event assembly are inseparable parts of the same route contract owned by `SEAM-1`.
- A generic OAuth UI or browser-flow seam was rejected because the remaining gap is not the login UX; it is auth ownership, field provenance, and in-world delivery.
- A broad Substrate deployment seam was rejected because this pack only needs the gateway-side auth-handoff contract and verification surfaces, not a full deployment program.
- A live-probe or operator-smoke seam was rejected because live evidence is valuable, but the immediate gap is deterministic contract and regression truth, not ad hoc runtime testing alone.
