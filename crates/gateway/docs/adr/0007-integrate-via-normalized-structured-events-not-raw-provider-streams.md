# ADR 0007: Integrate via Normalized Structured Events, Not Raw Provider Streams

- Status: Proposed
- Date: 2026-03-27

## Context

The reviewed Agent Hub output-routing pack assumes a strict distinction between:

- PTY bytes
- structured agent events

It requires that structured events be durable, separately rendered, and never injected into PTY byte streams during passthrough. That contract is downstream-friendly only if upstream systems provide a normalized event model rather than leaking raw provider stream frames into shell/REPL logic.

This project will consume provider-specific streaming behavior, including Azure Kimi cases where tool intent may be hidden in `reasoning_content` markers. Those provider stream details are too unstable and too vendor-specific to expose as the integration surface for Substrate.

## Decision

This gateway must normalize provider/model stream behavior into a structured internal event model before any downstream shell, REPL, or agent-hub integration boundary.

That means:

- raw upstream SSE or chunk formats are implementation details
- provider-specific artifacts such as hidden tool markers are implementation details
- downstream integrations should consume normalized structured events and stable final/tool/action semantics

## Consequences

Positive:

- The gateway aligns with Substrate’s structured-event and output-routing direction.
- Shell and Agent Hub consumers can depend on stable semantics rather than vendor-specific stream frames.
- Telemetry and persistence become easier because the normalized event model is durable and explicit.

Negative:

- We must define the internal event model early.
- Debug tooling may need separate raw-stream inspection paths for provider troubleshooting.

## Constraints on Implementation

- Do not expose raw provider stream chunks as the primary downstream contract.
- Do not let shell-facing output behavior depend on provider-specific marker syntax.
- Normalize tool intent, reasoning milestones, final content, and warning/error signals before handing events to downstream consumers.
- Preserve enough structured detail for observability without coupling consumers to provider transport.

## Deliverable Boundary

This ADR is complete when:

1. The gateway has a normalized internal event model that is distinct from raw provider transport.
2. Provider adapters translate into that model before downstream rendering/integration.
3. Design/docs make clear that shell or agent-hub integrations must consume normalized structured events rather than raw upstream frames.
