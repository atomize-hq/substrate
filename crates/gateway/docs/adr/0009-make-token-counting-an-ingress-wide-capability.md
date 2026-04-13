# ADR 0009: Make Token Counting an Ingress-Wide Capability

- Status: Proposed
- Date: 2026-04-03

## Context

The gateway currently exposes token counting through `POST /v1/messages/count_tokens`.
That handler parses a messages-shaped count request and routes it through provider token-count support.

This is workable for the Anthropic-compatible `/v1/messages` surface, but it does not yet match the broader direction of the gateway:

- the repo already exposes more than one public ingress family
- the OpenAI-side roadmap now includes expanding public `/v1/chat/completions` and later adding public `/v1/responses`
- the gateway's core architecture is based on normalized internal request state rather than on one public request shape

If token counting remains bound only to the Anthropic messages-shaped request contract, later public ingress expansion will create an unnecessary feature gap:

- users of one ingress family will have token counting
- users of another ingress family will need to manually translate requests or lose token-count support

That would make token counting an accidental artifact of the first public API shape instead of a gateway capability.

## Decision

Token counting should be treated as an ingress-wide gateway capability rather than as a messages-only feature.

This means:

1. every supported public ingress family should be able to use token counting through a thin request-shape adapter
2. token counting should converge on one internal counting path rather than separate per-API counting engines
3. public request-shape differences must be handled at the API adapter boundary, not inside provider-specific counting logic

The canonical implementation posture is:

- parse each public ingress shape into the same internal request representation used for routing and provider selection
- derive the provider-facing count request from that normalized internal representation
- keep token counting semantics aligned across ingress families unless a specific public contract requires a documented difference

## Rationale

This fits the gateway's existing architecture because:

- provider selection is already centralized above the provider layer
- public API families are already expected to be thin adapters over a shared internal core
- token counting is conceptually a capability of the request being sent, not of one specific public wire format

This also avoids duplicated behavior:

- no separate token-count engine for Anthropic, chat-completions, and responses-shaped public APIs
- no requirement for callers to translate request shapes manually just to obtain token estimates
- no drift where one public surface counts tokens differently because it bypasses the shared internal model

## Consequences

Positive:

- token counting becomes consistently available across the gateway's supported public ingress families
- later OpenAI-side surface expansion does not create a token-counting regression or gap
- the implementation remains aligned with the normalized-core architecture

Negative:

- each public ingress family will need its own token-count adapter logic
- some public API families may require explicit decisions about which fields participate in token counting when their request semantics differ
- docs will need to explain token counting as a cross-surface capability rather than only as an Anthropic-compatible endpoint detail

## Constraints On Implementation

- keep provider token counting behind one shared provider-facing capability
- do not create separate provider-counting logic per public API family
- treat Anthropic Messages, OpenAI Chat Completions, and future OpenAI Responses as adapter inputs to one internal token-count model
- keep routing, model mapping, and provider fallback behavior consistent with the main request path whenever feasible
- if an ingress family cannot represent token counting with exact parity, document the difference explicitly rather than silently diverging

## Open Design Boundary

This ADR does not freeze the exact public route layout for every ingress family.

Later implementation may choose either of these patterns, as long as the shared-capability rule holds:

- route-local token counting endpoints per public ingress family
- one canonical token-counting entrypoint that accepts multiple supported ingress shapes through explicit adapter logic

What is fixed by this ADR is the capability rule:

- token counting must not remain messages-only once the gateway supports multiple public ingress families

## Deliverable Boundary

This ADR is complete when:

1. the repo has a clear architectural decision that token counting is an ingress-wide capability rather than a `/v1/messages`-only feature
2. later implementation work can cite this ADR to add token-count support for expanded `/v1/chat/completions` and future `/v1/responses` surfaces through thin adapters over one internal counting path
3. the implementation boundary stays aligned with the shared normalized-core architecture instead of duplicating count logic per public API family
