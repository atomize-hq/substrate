# ADR 0003: Separate Reasoning Normalization from Planner/Executor Orchestration

- Status: Proposed
- Date: 2026-03-27

## Context

The current operating assumption is:

- `Kimi-K2-Thinking` is useful as a planner.
- `Kimi-K2.5` is the safer execution model for tool-enabled work.

That split is practical, but it is still only one orchestration strategy. The deeper underlying problem is provider normalization: Azure Kimi may encode reasoning and tool intent in non-standard ways. If planning and provider parsing are coupled together too early, the code will become hard to extend and hard to reuse for future routers or models.

## Decision

Keep these as separate layers:

1. Provider normalization
2. Internal event and message model
3. Optional planner/executor orchestration policy
4. External API surfaces

The planner/executor policy may default to `Kimi-K2-Thinking` for hidden planning and `Kimi-K2.5` for execution, but that policy must sit above provider normalization rather than inside it.

## Consequences

Positive:

- The Azure Kimi parser can be reused regardless of model-routing policy.
- Planner/executor behavior can evolve without rewriting provider code.
- Future routers or clients can reuse the same normalized engine with different orchestration rules.

Negative:

- Initial implementation may feel more abstract than a one-off fix.
- We need a clear internal event model early, before all client surfaces are built.

## Deliverable Boundary

This ADR is complete when:

1. Provider parsing and normalization are implemented without hard-coding planner/executor switching into the parser layer.
2. Planner/executor selection is configurable as a higher-level policy.
3. At least one end-to-end path proves `Kimi-K2-Thinking` planning can feed `Kimi-K2.5` execution through the normalized internal model.
