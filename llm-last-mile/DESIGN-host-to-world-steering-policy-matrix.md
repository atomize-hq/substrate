# Design: Host-To-World Steering Policy Matrix

Status: draft design input. This document defines the policy matrix for host-orchestrator control of world-side agent work after the dispatch, messaging, lifecycle, and durable obligation-ledger contracts have been frozen. It is not a final config-schema document. It freezes the policy categories, decision axes, default posture, and minimum gating rules that any implementation must honor.

## Why This Doc Exists

The other design docs now define:

1. how host orchestrators allocate world work,
2. how retained workers exchange messages with the host,
3. how worker lifecycle behaves,
4. how durable obligations and `awaiting_attention` work.

What is still missing is one explicit answer to:

1. who may do what,
2. against which workers,
3. in which modes,
4. under which world/session boundaries,
5. with what fork autonomy.

Without a dedicated policy design:

1. protocol docs will start carrying scattered authorization assumptions,
2. fork and escalation rules will drift,
3. implementation will re-invent control-plane allow/deny logic ad hoc,
4. "control plane" versus "execution plane" will blur again.

This document closes that gap.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): action vocabulary, exact identity, and explicit mode.
2. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md): retained-worker message classes, fork requests, and operational steering.
3. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): worker lifecycle states and invalidation semantics.
4. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md): canonical durable obligation kinds and attach/review substates.
5. [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md): review/inbox projection and attention behavior.
6. [ADR-0026](../docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md) and [ADR-0045](../docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md): the future toolbox remains an internal control-plane surface rather than a second execution plane.
7. [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](./29-shared-agent-dispatch-envelope-and-capability-override-contract.md): capability narrowing remains restriction-only.

## Problem Statement

How might Substrate gate host-orchestrator steering of world-side agent work so that:

1. the host may plan and steer without becoming the execution surface,
2. all world work remains fail-closed by default,
3. mode, action, backend, and fork permissions are explicit,
4. world-binding and orchestration-session boundaries remain authoritative,
5. worker-requested fork and escalation are controlled rather than implicit?

## Frozen Direction

This design freezes the following:

1. host-to-world steering is deny-by-default,
2. control-plane permissions are distinct from backend runtime capabilities,
3. policy must gate action, mode, backend, identity scope, and fork autonomy separately,
4. exact `orchestration_session_id`, `participant_id`, `backend_id`, `world_id`, and `world_generation` remain the default authority boundaries,
5. worker-requested fork is permitted only through explicit policy and explicit host or sanctioned control-plane acceptance,
6. capability overrides remain narrowing-only.

## Non-Goals

This design does not:

1. define the final config file syntax or key paths,
2. define the public human policy surface,
3. decide every future backend-specific policy nuance,
4. replace exact targeting with heuristic routing,
5. define execution-plane backend feature flags.

## Core Principle

Host-to-world steering permission is not one boolean.

The policy space has to answer at least:

1. may this orchestrator dispatch world work at all?
2. which backends may it target?
3. which actions may it take?
4. which lifecycle modes may it request?
5. under which session/world boundaries?
6. may workers request fork, recommend fork, or auto-fork?

If those are not separate decisions, the policy model will become too blunt to be safe.

## Policy Layers

There are three policy layers relevant to host-to-world steering:

### 1. Inventory baseline

Defines:

1. backend identity,
2. execution-scope defaults,
3. baseline capability declarations,
4. baseline worker launch permissions when a worker is created.

### 2. Control-plane steering policy

Defines:

1. whether the host orchestrator may dispatch or steer world work,
2. which actions and modes are permitted,
3. targeting boundaries,
4. fork autonomy.

### 3. Execution-plane runtime capability policy

Defines:

1. what the target backend/runtime may actually do once launched,
2. capability narrowing for the worker itself.

### Hard separation

A request must pass both:

1. control-plane steering authorization,
2. execution-plane runtime capability checks.

Passing one must never imply the other.

## Policy Decision Axes

Implementation must support separate policy decisions for the following axes.

### A. Global steering enablement

Question:

1. may host-orchestrator world steering exist in this environment at all?

Default:

1. deny.

### B. Allowed target backends

Question:

1. which exact `backend_id` values may be used for world work?

Default:

1. empty allowlist.

### C. Allowed modes

Question:

1. may the orchestrator request `ephemeral`?
2. may it request `retained`?

Default:

1. `retained` allowed only when explicitly enabled,
2. `ephemeral` allowed only when explicitly enabled.

The design does not assume `ephemeral` is safer by default; it is merely a different lifecycle contract.

### D. Allowed actions

Question:

1. which control-plane actions are allowed?

Frozen action set:

1. `run_world_task`
2. `spawn_world_worker`
3. `continue_world_worker`
4. `inspect_world_worker`
5. `fork_world_worker`
6. `cancel_world_work`
7. `stop_world_worker`

Default:

1. all denied until explicitly allowlisted.

### E. Session boundary

Question:

1. may the host steer only workers in its own `orchestration_session_id`, or may it cross session boundaries?

Default:

1. same session only.

### F. World-binding boundary

Question:

1. may the host steer only workers in the same authoritative `world_id` and `world_generation`?

Default:

1. same world binding only.

### G. Capability override allowance

Question:

1. may the host request capability narrowing at dispatch time?

Default:

1. deny unless explicitly enabled.

If enabled:

1. narrowing-only,
2. never broadening,
3. field-scoped denial remains explanation-ready.

### H. Fork request autonomy

Question:

1. may retained workers emit `fork_request`?
2. may retained workers emit only `fork_recommendation`?
3. may retained workers auto-fork under any narrow rubric?

Default:

1. recommendations and requests both denied,
2. auto-fork denied.

### I. Fork concurrency and depth

Question:

1. how many live child workers may one source worker spawn?
2. how deep may fork lineage go?
3. how many live retained workers may one orchestration session hold at once?

Default:

1. explicit low caps,
2. fork depth 0 unless enabled,
3. session-level live retained worker cap required.

### J. Notification-driving permissions

Question:

1. may the worker emit approval requests?
2. may the worker emit follow-up questions?
3. may the worker emit fork requests?
4. may it emit only non-attention-driving recommendations?

Default:

1. deny-by-default until enabled for the worker role/backend combination.

## Recommended Conceptual Policy Surface

The exact syntax can be chosen later, but the implementation should support policy categories conceptually equivalent to:

```text
agents.world_dispatch.enabled
agents.world_dispatch.allowed_backends[]
agents.world_dispatch.allowed_modes[]
agents.world_dispatch.allowed_actions[]
agents.world_dispatch.same_session_only
agents.world_dispatch.same_world_binding_only
agents.world_dispatch.allow_capability_narrowing
agents.world_dispatch.max_live_retained_workers
agents.world_dispatch.max_concurrent_ephemeral
agents.world_dispatch.max_fork_depth

agents.world_dispatch.fork.requests_allowed
agents.world_dispatch.fork.recommendations_allowed
agents.world_dispatch.fork.auto_fork_allowed
agents.world_dispatch.fork.max_children_per_worker

agents.world_dispatch.obligations.follow_up_allowed
agents.world_dispatch.obligations.approval_allowed
agents.world_dispatch.obligations.blocked_allowed
agents.world_dispatch.obligations.fork_request_allowed
```

This is not a final key-path freeze. It is the minimum policy dimensionality the repo should preserve.

## Default Posture

The safest default policy posture is:

1. host-to-world steering disabled,
2. no allowed backends,
3. no allowed actions,
4. no allowed modes,
5. same-session-only required when steering is enabled,
6. same-world-binding-only required when steering is enabled,
7. capability narrowing disabled unless explicitly enabled,
8. worker fork requests, recommendations, and auto-fork all denied unless explicitly enabled.

## Action Matrix

The implementation should evaluate at least the following checks per action.

### `run_world_task`

Required checks:

1. steering enabled,
2. action allowed,
3. `mode=ephemeral` allowed,
4. `target_backend_id` allowed,
5. same-session boundary satisfied,
6. same-world-binding boundary satisfied,
7. ephemeral concurrency cap not exceeded.

### `spawn_world_worker`

Required checks:

1. steering enabled,
2. action allowed,
3. `mode=retained` allowed,
4. `target_backend_id` allowed,
5. same-session boundary satisfied,
6. same-world-binding boundary satisfied,
7. retained worker cap not exceeded.

### `continue_world_worker`

Required checks:

1. steering enabled,
2. action allowed,
3. exact retained `target_participant_id`,
4. same-session boundary satisfied,
5. same-world-binding boundary satisfied,
6. worker not invalidated or terminal.

### `inspect_world_worker`

Required checks:

1. steering enabled,
2. action allowed,
3. exact task or worker identity,
4. same-session boundary satisfied,
5. same-world-binding boundary satisfied.

### `fork_world_worker`

Required checks:

1. steering enabled,
2. action allowed,
3. source retained `target_participant_id` exact,
4. `mode=retained` allowed for child unless policy explicitly allows ephemeral child fork,
5. source worker not invalidated or terminal,
6. same-session boundary satisfied,
7. same-world-binding boundary satisfied,
8. fork depth cap not exceeded,
9. session retained-worker cap not exceeded,
10. worker child-count cap not exceeded.

### `cancel_world_work`

Required checks:

1. steering enabled,
2. action allowed,
3. exact task or worker identity,
4. same-session boundary satisfied,
5. same-world-binding boundary satisfied.

### `stop_world_worker`

Required checks:

1. steering enabled,
2. action allowed,
3. exact retained worker identity,
4. same-session boundary satisfied,
5. same-world-binding boundary satisfied,
6. worker not already terminal.

## Worker-Initiated Fork Policy

Fork is the sharpest policy edge in this design.

### Three autonomy tiers

1. `none`
   - worker may not recommend or request fork.
2. `recommend_only`
   - worker may emit `fork_recommendation`,
   - cannot emit `fork_request`.
3. `request_only`
   - worker may emit `fork_request`,
   - host review required before allocation.
4. `auto_fork_limited`
   - worker may request and execute fork automatically only under a tightly bounded rubric.

### Recommended v1 posture

Use:

1. `none` by default,
2. `recommend_only` or `request_only` for explicitly trusted worker classes,
3. no `auto_fork_limited` in the first shipping slice unless there is an exceptionally tight rubric and strong caps.

### Why

Auto-fork is not just a convenience feature. It is:

1. concurrency allocation,
2. context duplication,
3. potentially unbounded cost expansion,
4. a lineage explosion risk,
5. a policy and observability burden.

So it should start heavily constrained.

## Approval and Follow-Up Policy

The same "not one boolean" rule applies to host re-engagement events.

Separate policy should be able to gate whether workers may:

1. ask follow-up questions,
2. request approvals,
3. emit blocked obligations,
4. emit fork requests.

This matters because some worker roles may be allowed to:

1. report results only,
2. recommend but not request fork,
3. ask clarifying questions but not block on approval,
4. operate only as tightly bounded one-shot retained workers.

## Invalidated Worker Policy

Workers that are `invalidated` must fail closed for steering.

Required rule:

1. policy must not permit `continue`, `fork`, or `stop` to silently recover an invalidated worker as if the world-binding mismatch never happened.

Allowed next steps after invalidation:

1. inspect,
2. record failure or alert,
3. allocate replacement work explicitly.

## Policy and Exact Identity

The policy model must preserve exact identity routing.

It must not permit:

1. "continue whichever worker seems active",
2. "fork the most recent worker for this backend",
3. "cancel all workers in this role" as a generic fuzzy control action,
4. world-bound actions without authoritative `world_id` and `world_generation`.

This is not just a runtime preference. It is part of the policy safety contract.

## Explanation-Ready Denials

Every denial should be attributable to one or more explicit policy dimensions, such as:

1. `world_dispatch_disabled`
2. `backend_not_allowed`
3. `mode_not_allowed`
4. `action_not_allowed`
5. `cross_session_steering_denied`
6. `cross_world_binding_steering_denied`
7. `capability_narrowing_not_allowed`
8. `fork_request_not_allowed`
9. `fork_depth_exceeded`
10. `worker_concurrency_cap_exceeded`
11. `invalidated_worker_not_routable`

That keeps control-plane behavior auditable and operator-explainable.

## v1 Recommended Simplifications

To keep the first shipping slice tractable:

1. same-session-only and same-world-binding-only should be hard defaults,
2. only explicit backend allowlists should be supported,
3. `retained` should be the common allowed mode,
4. `ephemeral` should require explicit enablement,
5. worker fork should start as `recommend_only` or `request_only`,
6. auto-fork should remain disabled,
7. explanation-ready denial buckets should be preserved from day one.

## v1 Summary

The first shippable host-to-world steering policy matrix should therefore be:

1. deny-by-default,
2. exact-identity,
3. separate across action, mode, backend, and boundary dimensions,
4. fork-aware and conservative,
5. explicit about worker re-engagement permissions,
6. fail-closed on invalidated or boundary-mismatched work.

## Follow-On Design Dependencies

This policy matrix should feed directly into:

1. the implementation SOW for host-orchestrator world dispatch,
2. later config/policy schema design,
3. any future narrow auto-fork rubric if that is ever reopened.
