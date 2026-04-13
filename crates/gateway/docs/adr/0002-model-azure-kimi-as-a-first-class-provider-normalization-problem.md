# ADR 0002: Model Azure Kimi as a First-Class Provider Normalization Problem

- Status: Proposed
- Date: 2026-03-27

## Context

Prior investigation established that Azure-hosted `Kimi-K2-Thinking` does not reliably emit explicit OpenAI-style `tool_calls` in the cases that matter most for Claude Code. Instead, it can place tool intent inside `reasoning_content` using Kimi-specific sentinel markers such as:

- `<|tool_calls_section_begin|>`
- `<|tool_call_begin|>`
- `<|tool_call_argument_begin|>`
- `<|tool_call_end|>`
- `<|tool_calls_section_end|>`

This is not a generic OpenAI-compatible transport issue. It is a provider-behavior issue specific to Azure Foundry Kimi as currently observed. Treating it as a thin format-translation problem would keep the system fragile.

The `claude-code-mux` foundation may already contain Kimi-related fixes, including the upstream commit `5a372fb`, but those fixes were not proven against the Azure Foundry hidden-tool behavior captured in prior CCR logs.

## Decision

Treat Azure Foundry Kimi as a first-class provider normalization layer inside the gateway.

The Azure Kimi provider path must:

- Consume Azure chat-completions responses directly.
- Normalize explicit `tool_calls` when present.
- Parse hidden tool intent from `reasoning_content` when explicit `tool_calls` are absent.
- Produce stable internal tool/action events for the rest of the gateway.

The rest of the gateway should operate on normalized events and should not depend on Azure Kimi sentinel syntax.

## Consequences

Positive:

- Azure-specific behavior is isolated behind a provider boundary.
- Anthropic-facing and future OpenAI-facing surfaces can reuse the same normalized model events.
- Regression testing can target the real provider quirk directly.

Negative:

- We will need Azure-specific parser and test coverage rather than configuration-only support.
- Existing generic OpenAI-compatible provider logic may need to be extended or bypassed for Azure Kimi.

## Deliverable Boundary

This ADR is complete when:

1. Azure Kimi has an explicit provider or provider-mode implementation boundary.
2. The gateway can normalize both explicit tool calls and hidden tool markers into one internal representation.
3. Regression fixtures cover the observed Azure Kimi hidden-tool patterns.
