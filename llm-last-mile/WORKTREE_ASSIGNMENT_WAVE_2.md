# Worktree Assignment Plan — Wave 2

Status: second implementation wave. This wave starts after Wave 1, especially after backend/adaptor realization semantics from Stream 6 are stable.

## Objective

Build the next contract layer on top of the Wave 1 baseline:

- tuple-axis policy and provenance
- tuple projection on status/trace surfaces
- the actual internal toolbox MCP server, still read-only

## Start Condition

Do not begin full Wave 2 implementation until Stream 6 is settled enough that backend ids, adapter realization behavior, and integrated auth/runtime expectations are no longer moving targets.

## Streams

### Stream 7a: Tuple-axis policy and broker/schema

- Worktree: `codex/sow-7a-tuple-policy`
- Goal: add the additive tuple-axis policy layer under `llm.constraints.*`

#### Owns

- [docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md:1)
- [docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md:1)
- broker/config/policy model files under `crates/broker`
- relevant config/policy plumbing under `crates/shell/src/execution`
- policy/config tests

#### Intended outcomes

- `llm.constraints.routers`
- `llm.constraints.providers`
- `llm.constraints.protocols`
- `llm.constraints.auth_authorities`
- `--explain` provenance for the new policy surface

#### Non-goals

- no status rendering changes
- no toolbox server implementation

### Stream 7b: Tuple projection on status and trace

- Worktree: `codex/sow-7b-tuple-status-trace`
- Goal: project the accepted tuple vocabulary consistently into status and trace surfaces

#### Owns

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:1)
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:1)
- trace/status tests in `crates/shell/tests` and `crates/common/tests`

#### Depends on

- field names and semantics from Stream 7a
- backend realization behavior from Stream 6

#### Non-goals

- no new policy keys
- no toolbox server implementation

### Stream 8: Internal toolbox MCP server, still read-only

- Worktree: `codex/sow-8-toolbox-server`
- Goal: turn today’s `toolbox status|env` introspection surface into the actual host-scoped internal MCP service, while keeping v1 read-only

#### Owns

- [docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md:1)
- toolbox implementation files under `crates/shell/src/execution` or a new toolbox module
- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1372) for CLI wiring if needed
- toolbox trace emission and `tool_call_id` joinability

#### Shared contract dependency

This stream must align with the tuple and trace vocabulary settled by Streams 7a and 7b.

#### Non-goals

- no mutation tools
- no router daemon behavior

## Parallelism Rule

The safest execution shape is:

- Stream 7a begins first in Wave 2
- Stream 7b and Stream 8 may proceed in parallel after the tuple field names and basic semantics are settled

## Conflict Hotspots

- [crates/shell/src/execution/agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1)
  - Stream 7b owns tuple/status projection
  - Stream 8 owns toolbox CLI wiring only
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md:1)
  - Stream 7b should be the final editor once field names settle
- [docs/USAGE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md:1)
  - batch final doc merge late in the wave

## Exit Criteria

- tuple policy keys are real and explainable
- tuple status/trace vocabulary is visible and coherent
- toolbox server exists as a read-only host-scoped MCP surface
- no stream in this wave invents a second execution plane

