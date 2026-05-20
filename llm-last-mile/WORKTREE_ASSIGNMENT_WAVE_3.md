# Worktree Assignment Plan — Wave 3

Status: third implementation wave. This wave is intentionally more serialized because it depends on the contracts stabilized in the earlier waves.

## Objective

Build the first real router-daemon implementation on top of stabilized gateway, tuple, trace, and toolbox contracts.

## Stream

### Stream 9: Router daemon v1

- Worktree: `codex/sow-9-router-daemon`
- Goal: implement the ADR-0029 trace-driven router with durable queueing, workspace registry, idempotent triggers, and policy re-evaluation

## Owns

- [docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md:1)
- router daemon/runtime code
- durable inbox/work-queue or equivalent request-queue implementation
- cursor and dedupe state
- workspace registry and target-workspace resolution
- router-derived event tests and policy re-evaluation tests

## Depends On

- Stream 6 for stable backend realization semantics
- Stream 7a for tuple/policy vocabulary
- Stream 7b for trace/status field projection and correlation consistency
- Stream 8 for the toolbox/control-plane read contract if router interactions rely on it

## Parallelism Rule

This wave should be treated as mostly solo implementation.

You may run small exploratory or scaffolding work in parallel, but the real router implementation should not start until the earlier contracts are merged and stable.

## Non-Goals

- no workflow-engine or forge composition
- no mutation expansion of the toolbox
- no reopening of accepted trace/config foundations
- no redefinition of backend-id semantics

## Risks

- contract drift if router event families are implemented before tuple/trace naming is stable
- hidden coupling to pre-stable toolbox behavior
- idempotency and cause-reference bugs if derived-event join keys are invented ad hoc

## Merge Policy

- Merge only after Wave 2 is materially complete.
- Keep this wave focused on router v1 itself, not on workflow composition.

## Exit Criteria

- router daemon has durable request semantics
- trigger evaluation is trace-driven and policy-rechecked
- workspace targeting is explicit
- derived events use the accepted correlation vocabulary
- the repo is ready for later workflow-engine composition work without reopening router foundations

