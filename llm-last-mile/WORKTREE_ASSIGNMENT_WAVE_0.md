# Worktree Assignment Plan — Wave 0

Status: planning document. This wave is decision-only and docs-only. It exists to reduce ambiguity before parallel implementation waves begin.

## Objective

Lock the naming and boundary decisions that affect later streams without introducing cross-cutting code churn during active implementation.

This wave exists to answer:

- whether `uaa.agent.session` remains the local protocol-family label,
- whether the local `agent-api-*` crate names are acceptable for now,
- what should be treated as docs-only clarification now,
- and what should be deferred to a later rename cleanup stream.

## Stream

### Stream 5a: Naming decision only

- Worktree: `codex/sow-5-decision`
- Type: docs-only
- Goal: decide the naming posture for external `agent_api` versus local `agent-api-*`, and decide whether `uaa.agent.session` stays or is scheduled for later rename

## File Ownership

This stream owns only docs and decision artifacts:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:1)
- [docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md:1)
- [docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md:1)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:1)

## Deliverables

- one explicit decision on `uaa.agent.session`
- one explicit decision on whether local `agent-api-*` names are:
  - acceptable for now,
  - scheduled for later cleanup,
  - or blocked on a rename before further rollout
- repo-facing wording that distinguishes:
  - external `agent_api` / Unified Agent API
  - local host/world transport crates
- a short follow-on note describing whether a future rename should be:
  - docs-only,
  - code + docs,
  - or full crate/package migration

## Non-Goals

- no code symbol renames
- no crate renames
- no protocol-label migration in trace or runtime code
- no public-surface behavior change

## Merge Policy

- Merge this wave before or alongside Wave 1, but keep it docs-only.
- Do not let this wave expand into a broad refactor.

## Exit Criteria

- downstream wave owners can reference one settled naming decision
- no implementation stream needs to guess whether `uaa.agent.session` is canonical, transitional, or slated for removal

