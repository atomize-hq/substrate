# ADR 0005: Present a Single Backend Identity to Substrate

- Status: Proposed
- Date: 2026-03-27

## Context

The reviewed Substrate LLM policy/config planning pack defines backend selection and allowlisting through generic backend ids like `<kind>:<name>`, while explicitly deferring detailed backend semantics to later gateway/engine contracts.

This gateway will likely need multiple internal behaviors:

- Azure Foundry provider normalization
- Kimi hidden-tool parsing
- `Kimi-K2-Thinking` planning
- `Kimi-K2.5` execution
- possibly additional providers later

If those internal seams are exposed directly as separate Substrate backend ids, Substrate policy and configuration would become coupled to internal implementation details instead of selecting one stable capability surface.

## Decision

When this project is integrated into Substrate, it must present one logical backend identity to Substrate for this gateway capability.

Examples of what this means:

- Acceptable:
  - one stable backend id such as `api:kimi_gateway`
  - internal routing between planner/executor/provider implementations
- Not acceptable:
  - making Substrate choose between `api:kimi_k2_thinking`, `api:kimi_k2_5`, `api:azure_kimi_parser`, or similar internal roles

Substrate-facing backend ids must remain stable capability labels. Model choice, provider quirks, and orchestration strategy remain internal implementation details of this gateway.

## Consequences

Positive:

- Substrate config/policy stays simple and aligned with its planned backend abstraction.
- Internal gateway evolution does not force policy surface churn.
- Planner/executor changes remain a gateway concern rather than an operator concern.

Negative:

- More routing policy must live inside this project.
- Debugging may require explicit internal diagnostics because external config will not expose all internal seams.

## Constraints on Implementation

- Do not design the config model around externally selecting separate planner and executor backends.
- Do not leak provider-normalization modes into the public backend identity.
- Keep public backend naming capability-oriented, not implementation-oriented.

## Deliverable Boundary

This ADR is complete when:

1. Public configuration/docs for this gateway describe one stable external backend identity.
2. Internal planner/executor/provider seams remain internal code paths or internal config.
3. No public examples require Substrate to reason about individual Kimi model roles.
