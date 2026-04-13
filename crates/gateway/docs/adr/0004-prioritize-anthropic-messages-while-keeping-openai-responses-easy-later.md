# ADR 0004: Prioritize Anthropic Messages While Keeping OpenAI Responses Easy Later

- Status: Proposed
- Date: 2026-03-27

## Context

The immediate target client is Claude Code, which expects Anthropic-compatible Messages behavior. A future requirement is to expose an OpenAI Responses-compatible surface as well, but that is not the primary delivery target for the first implementation phase.

The risk is building a Claude-only code path that later makes Responses support expensive, or building Responses first and delaying the real user value.

## Decision

Prioritize Anthropic Messages compatibility for the first production-quality gateway path, while designing the core so OpenAI Responses can be added as a thin outer surface later.

This means:

- Anthropic Messages is the first-class ingress and streaming contract.
- Internal state, tool events, and provider normalization must remain client-agnostic.
- OpenAI Responses support should be treated as a later adapter around the same normalized core, not as a separate execution engine.

## Consequences

Positive:

- Work is aligned with the immediate Claude Code integration goal.
- We avoid duplicating provider logic for each client API shape.
- Later Responses support becomes a bounded adapter project instead of a second gateway.

Negative:

- Some API-generalization work must be done up front, even though Responses is deferred.
- Early validation will focus more on Anthropic semantics than on OpenAI Responses semantics.

## Deliverable Boundary

This ADR is complete when:

1. Claude Code can use the gateway through an Anthropic-compatible Messages surface.
2. The internal engine does not depend on Anthropic-only data structures at its core boundary.
3. A short design note identifies the future seam where OpenAI Responses can be added without refactoring provider normalization.
