# Substrate Structured-Events `C-06` Contract

## Purpose

This note is the canonical landing artifact for `C-06`.
It defines the downstream structured-event contract for the gateway as a normalized semantic boundary, not as a provider-transport contract.

This contract is intentionally narrow:

- it defines the downstream event boundary in terms of normalized gateway semantics
- it keeps raw SSE framing, provider chunk shape, and provider-specific marker syntax out of the consumer contract
- it preserves the normalized gateway as the source of truth for downstream structured events
- it gives later Substrate-facing work a stable contract to depend on without reverse-engineering provider parsing

It does not define:

- raw provider stream formats
- SSE framing details
- hidden marker syntax as public or downstream contract truth
- planner/executor policy
- public gateway identity or deployment boundary rules

## Canonical Source Of Truth

This contract is grounded in the normalized-event and public-surface notes plus the downstream integration ADR:

- `docs/foundation/azure-kimi-c02-normalized-event-contract.md`
- `docs/foundation/anthropic-messages-c03-contract.md`
- `docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`

The current runtime and fixture anchors are:

- `gateway/src/providers/openai.rs`
- `gateway/src/providers/streaming.rs`
- `gateway/src/server/mod.rs`
- `gateway/tests/fixtures/azure_kimi/`

If this note and those anchors disagree, the note or the upstream seam evidence must be revalidated before downstream use.

## Downstream Event Boundary

`C-06` states that downstream consumers receive structured events derived from the gateway's normalized semantics, not raw provider transport.

Required behavior:

- normalized `C-02` semantics are the source of downstream event meaning
- `C-03` public surface behavior remains the first public rendering layer over the normalized core
- downstream consumers may depend on stable structured-event meaning, ordering, and durability
- downstream consumers must not need to inspect raw provider chunks to interpret the contract

Contract meaning:

- `tool_intent` remains a normalized intent signal, not a provider-framing artifact
- `action` remains a normalized intermediate progress signal, not a raw stream frame
- `final` remains a normalized terminal signal, not a provider-specific completion frame
- provenance may exist for debugging, but it is not the downstream contract

Explicit exclusions:

- raw Azure SSE bytes are not the downstream contract
- provider chunk ordering is not the downstream contract
- hidden `reasoning_content` marker syntax is not the downstream contract
- internal role selection is not the downstream contract

## Runtime And Fixture Anchors

These are the repo surfaces that currently express the normalized boundary and therefore need to remain consistent with this contract:

- `gateway/src/providers/openai.rs`
- `gateway/src/providers/streaming.rs`
- `gateway/src/server/mod.rs`
- `gateway/tests/fixtures/azure_kimi/explicit-tool-calls-k2-thinking-stream.json`
- `gateway/tests/fixtures/azure_kimi/hidden-markers-k2-thinking-stream.json`
- `gateway/tests/fixtures/azure_kimi/hidden-markers-k2-thinking-nonstream.json`
- `gateway/tests/fixtures/azure_kimi/mixed-reasoning-and-tool-calls-k2-thinking.json`
- `gateway/tests/fixtures/azure_kimi/no-tool-control-k2-5-stream.json`

This note is the source of truth for how those anchors should be read.

## Drift Guards

Downstream revalidation is required if any of the following changes:

- downstream docs or schema work starts depending on raw provider transport, raw SSE framing, or provider chunk shape
- hidden marker syntax becomes visible as downstream contract truth instead of debug-only provenance
- normalized `C-02` event semantics or stable field guarantees change in a way that alters downstream event meaning
- `C-03` public surface behavior changes in a way that changes how normalized events are rendered downstream
- provider parsing, event rendering, or downstream schema starts exposing internal role truth

The guard is intentionally conservative:

- if a change forces downstream consumers to reason about provider-specific framing, the boundary has drifted
- if a change forces downstream consumers to know whether intent came from explicit `tool_calls` or hidden markers, the boundary has drifted
- if a change makes raw provider transport necessary for interpretation, the boundary has drifted

## Verification Checklist

`C-06` is complete only if a reviewer can answer yes to all of the following without reading provider parsing code:

- can the downstream event boundary be described as normalized gateway semantics rather than provider transport
- can `tool_intent`, `action`, and `final` be explained as downstream-usable structured events
- are raw SSE framing, provider chunk shape, and hidden marker syntax explicitly excluded or debug-only
- do the listed runtime anchors and Azure fixtures support the contract without broadening it into implementation prose
- do the drift guards make raw-transport dependence and marker leakage explicit revalidation triggers

## Compatibility Notes

- This note is compatible with the landed `C-02` normalization boundary and does not broaden it.
- This note is compatible with the landed `C-03` public surface and does not redefine it.
- This note is intentionally capability-oriented for downstream consumers: it describes the structured-event contract only, not public identity or deployment policy.
