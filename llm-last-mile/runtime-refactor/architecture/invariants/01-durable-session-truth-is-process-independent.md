**Kind:** architecture
**Stable ID:** `shared-invariant-01`
**Canonical for:** invariant 1 only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted invariant source body only; no implementation authority
**Source span:** [`../../01-target-architecture.md#1-durable-session-truth-is-process-independent`](../../01-target-architecture.md#1-durable-session-truth-is-process-independent) lines 87–132
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md), [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md)

# Invariant 1: Durable session truth is process-independent

### 1. Durable session truth is process-independent

Durable truth is the exact session identity, authoritative lineage, workspace/world binding, attach contract, retained-worker refs, resume handles, posture, and policy revision. Helper PID, attached client, socket reachability, startup stream state, and owner-process liveness are observations only.

Runtime execution scope and durable session world binding are distinct authority dimensions.
`AgentDescriptorV1.execution_scope` and the matching launch knob describe runtime placement;
`DurableSessionAuthorityV1.world_binding` describes the parent orchestration session's exact
available world substrate. Runtime placement and session binding are not bijective: a host-executing
orchestrator may own a world-backed durable session, while a world-executing runtime requires that
exact binding. The frozen Start acceptance matrix is:

| Descriptor and launch scope | Session world binding | Result |
|---|---|---|
| `Host` | `None` | accept |
| `Host` | `Some(exact binding)` | accept |
| `World` | `Some(exact binding)` | accept |
| `World` | `None` | reject |

Descriptor scope must still equal requested launch scope, and any present binding must contain the
exact non-empty world ID and generation supplied by session truth. `Host + Some` does not place the
host runtime in the world. Host participant manifests remain host-scoped and do not acquire
participant-level world placement fields; the binding stays on the durable session authority.
World filesystem, network, caging, policy, capability, and enforcement semantics are unchanged.

#### Exact bound-world physical ownership

The durable session binding and the backend's physical ownership metadata are separate truths.
`HostSessionAuthority` alone says which exact world ID and generation belong to the session. The
runtime-family/world backend may only realize that already-authoritative tuple physically. For the
bounded B3.2a-WA prerequisite, `ExactBoundWorldOwnershipAdoptionV1` is an internal operation
contract, not a `world-api` field and not a persisted wire-schema version. It permits one exact
transition:

```text
GenericExactBoundWorld -> SharedSessionOwnerExactBoundWorld
```

The transition preserves world ID and generation, exact-joins the validated HSA session/policy and
project/world-spec identity, and durably publishes ownership metadata before member process
creation. An exact already-adopted tuple joins without rewrite; any foreign owner, changed session,
generation, policy, project, spec, missing/corrupt metadata, or ambiguous publication fails closed.
Shell state, helper state, PID, timeout, socket, caller disappearance, process liveness, prompt
content, and compatibility projections are never adoption authority. Adoption changes no HSA or
RetainedWorkerRuntime record and proves neither transport submission nor member launch, Registered,
routability, or terminal success. Compatibility requests without the exact authority-managed proof
and ordinary world execution retain their existing physical-realization behavior.
