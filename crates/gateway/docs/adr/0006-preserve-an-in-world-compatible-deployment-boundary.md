# ADR 0006: Preserve an In-World-Compatible Deployment Boundary

- Status: Proposed
- Date: 2026-03-27

## Context

The reviewed Substrate LLM policy/config planning pack assumes fail-closed routing by default and treats `llm.gateway.mode=in_world` as the normal posture. It also models API agents with an `https://` base URL and secrets delivered from host policy-controlled env names into an in-world gateway/engine.

This project will likely start life as a host-local development process, potentially listening on loopback HTTP during early development. That is useful for iteration, but it does not match the steady-state boundary implied by the Substrate packs.

If we build the gateway around assumptions that only work as a host-local loopback service, later Substrate integration will require unnecessary redesign.

## Decision

The project may use a host-local development topology, but its architecture must preserve a clear path to an in-world-compatible deployment boundary.

Specifically:

- host-local loopback is a development convenience, not a contract
- auth, secret handling, and upstream provider access must be factored so they can operate behind a Substrate-controlled in-world boundary later
- transport assumptions must not be hard-coded into the core engine

In practical terms, the team should read this as:

- acceptable for development:
  - running a local process on `127.0.0.1`
  - exposing Anthropic-compatible HTTP locally so Claude Code can talk to it
  - using local env vars or local config to get the first working loop established
- not acceptable as a permanent architectural assumption:
  - making loopback HTTP the only viable deployment shape
  - requiring direct host credential access inside the core request pipeline
  - assuming the gateway always has unrestricted access to host-local files, sockets, or long-lived process state
  - intertwining provider auth, transport, and orchestration logic so tightly that an in-world wrapper or deployment boundary would require a rewrite

## Consequences

Positive:

- The project remains compatible with Substrate’s planned fail-closed world-boundary posture.
- Secret handling can later align with policy-gated host-to-world delivery instead of being embedded into ad hoc local process assumptions.
- A future in-world deployment or wrapper becomes an integration task, not a rewrite.

Negative:

- Some convenience shortcuts for a localhost-only tool may be off-limits.
- Early local-dev docs must be careful not to imply that loopback HTTP is the permanent architecture.

## Constraints on Implementation

- Do not make `127.0.0.1` HTTP the only supported architectural assumption.
- Keep provider auth handling and transport adapters separable from the normalized engine.
- Treat local dev transport and production/integration transport as replaceable outer layers.
- Avoid baking host-only filesystem or credential assumptions into the core request pipeline.

## Deliverable Boundary

This ADR is complete when:

1. The internal architecture clearly separates core normalization/orchestration from deployment transport.
2. Local development can run host-local, but the design docs explicitly state that this is not the only intended deployment boundary.
3. Secret and provider-auth handling are factored so a future Substrate in-world deployment remains plausible without architectural inversion.
