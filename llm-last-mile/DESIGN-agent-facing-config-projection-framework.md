# Design: Agent-Facing Config Projection Framework

Status: draft design input. This document defines the generic Substrate-owned framework for projecting agent-facing configuration and state into host- and world-scoped runtime environments. It is not a Codex-only document, not a final storage-schema document, and not an implementation-ready slice plan. It freezes the authority model, projection planes, identity rules, and persistence boundaries that later adapter-specific DESIGN docs and numbered implementation SPECs must follow.

## Why This Doc Exists

The repo already has:

1. placement-qualified agent inventory with exact backend ids,
2. durable orchestration-session authority,
3. world-backed runtime realization,
4. a managed in-world gateway seam with one-time auth-bundle handoff,
5. a growing need for richer agent-local configuration such as models, providers, MCP servers, skills, and runtime caches.

What the repo does not yet have is one frozen answer to:

1. who owns the authoritative configuration for CLI agents,
2. where that configuration should live conceptually,
3. how it should be projected into agent-native roots,
4. which parts are durable versus request-time only,
5. how that projection composes with retained workers and future multi-lane worlds.

That gap now matters because the current direct `cli:codex-world` path still uses a narrow compatibility posture: it seeds an isolated per-member `CODEX_HOME` from host `~/.codex` artifacts for launch-time bootstrap. That is useful as a short-term bridge, but it is not the intended architecture.

This document closes the generic framework gap above any one adapter-specific mapping.

## Relationship To Existing Decisions

This design composes with:

1. [ADR-0040](../docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md): `substrate-gateway` is the LLM/runtime boundary and Substrate owns the runtime seam.
2. [ADR-0046](../docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md): backend selection and runtime integration stay Substrate-owned.
3. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): orchestration session remains the durable authority.
4. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): host-to-world work allocation is explicit and exact-targeted.
5. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md): `ephemeral` and `retained` world work are distinct lifecycle classes.
6. [docs/ideas/multi-lane-worlds.md](../docs/ideas/multi-lane-worlds.md): the future product model is lane-first, not raw world-id-first.
7. [docs/contracts/gateway/runtime-parity.md](../docs/contracts/gateway/runtime-parity.md): runtime families should converge on one truthful gateway-mediated architecture rather than bespoke execution planes.
8. [docs/internals/world/gateway_auth_handoff.md](../docs/internals/world/gateway_auth_handoff.md): one-time auth-bundle FD handoff into the in-world gateway is already a landed security seam.

## Problem Statement

How might Substrate own agent-facing configuration and state so that:

1. Substrate, not the agent, remains the authority for auth, policy, placement, and routing,
2. agent-native config roots can still be realized truthfully for each runtime family,
3. retained workers and future named lanes get isolated state by default,
4. durable projected state does not silently drift away from Substrate-owned inventories,
5. future adapters with different discovery models can fit under one generic framework?

## Frozen Direction

This design freezes the following:

1. Substrate owns **logical inventories** and **effective policy**, not agent-native config files.
2. Agent-native files and directories are **projection targets**, not primary source of truth.
3. Projection happens in at least two planes:
   - durable logical inventory / state authority,
   - concrete runtime projection into agent-facing roots.
4. Projection identity must include lane- or worker-local state ownership; workspace/backend alone is too coarse.
5. Secrets and other sensitive auth material are not treated as general durable config and remain on stricter handoff paths.
6. Durable projection and request-time launch rendering are distinct concerns and must stay distinct.
7. This framework is generic; adapter-specific mapping belongs in follow-on DESIGN docs.

## Non-Goals

This design does not:

1. define Codex-specific `config.toml` keys in final detail,
2. define the final on-disk storage schema for projected homes,
3. define the final public CLI for managing projected inventories,
4. require every runtime family to share identical native file layouts,
5. solve full multi-lane orchestration UX in this document,
6. declare that current compatibility bridges are already retired.

## Core Principle

The authority boundary is:

1. Substrate owns what the runtime is allowed to know and do.
2. The adapter/runtime family owns how that truth must be rendered natively.

That means:

1. Substrate does not hand responsibility back to `~/.codex`, `.claude`, or equivalent host-native state as the architectural truth.
2. Substrate may still project selected compatible material into agent-native roots when needed.
3. The runtime sees a **managed view** of its home/project/config state, not ambient host state discovery by default.

## Conceptual Model

There are four distinct layers:

### 1. Logical inventory layer

This is the Substrate-owned source of truth.

Examples:

1. enabled agent backends,
2. provider profiles,
3. MCP server definitions,
4. adapter-specific settings allowed by policy,
5. lane-local enablement and overrides,
6. future skills or plugin inventory records.

This layer is not stored primarily as agent-native config files.

### 2. Effective projection layer

This is the resolved, policy-filtered, precedence-applied view for one concrete runtime target.

Inputs include:

1. global inventory,
2. workspace inventory,
3. exact backend selection,
4. orchestration session,
5. lane or retained-worker identity,
6. world binding,
7. narrowing policy and runtime constraints.

This layer is still Substrate-owned truth.

### 3. Native projection layer

This is the concrete materialization needed by the chosen adapter.

Examples:

1. home-root files,
2. project-root overlays,
3. generated config fragments,
4. app-runtime registries,
5. launch env,
6. scratch/runtime dirs.

This layer is where agent-native compatibility happens.

### 4. Launch-time secret handoff layer

Sensitive auth material is handled separately from ordinary config projection.

Examples:

1. one-time FD auth-bundle handoff into the in-world gateway,
2. explicit env injection when a runtime contract truly requires it,
3. ephemeral process-local auth caches.

This layer should not be silently collapsed into ordinary durable config projection.

## Projection Roots

The framework must distinguish at least two native root classes:

### 1. Home-root projection

Used for adapter-native state normally discovered from an app home root.

Examples:

1. `CODEX_HOME`,
2. auth/session/history/log/cache state,
3. MCP server registries and app-runtime registries when the adapter stores them under home state.

### 2. Project-root projection

Used for adapter-native state discovered from cwd/repo-local overlays.

Examples:

1. project `.codex/config.toml`,
2. future `.claude/**`,
3. `.mcp.json`,
4. project-local skills or plugin overlays.

Hard rule:

1. the framework must not assume home-root projection alone is enough for every runtime family.

## Identity Model

Projection identity must not be keyed only by:

1. workspace root,
2. backend id,
3. world generation.

That key is too coarse for same-backend concurrent lanes.

The framework should instead assume:

1. orchestration session remains the durable authority root,
2. authoritative world binding remains a routability boundary,
3. lane or retained-worker identity joins the projected-state key.

### Scope boundary versus owner identity

These are not the same thing.

1. The **scope boundary** answers "which session/world/backend does this projection belong to?"
2. The **owner identity** answers "which exact retained worker or future named lane owns this mutable projected state?"

In practical terms:

1. `orchestration_session_id + authoritative world binding + backend_id` is a good scope boundary,
2. it is not a sufficient owner identity for parallel same-backend workers,
3. retained-worker identity in v1, and future lane identity later, must supply that owner dimension.

### Why this cannot be deferred fully

If two retained Codex workers share the same backend and same authoritative world, but are supposed to run in parallel with isolated mutable state, then:

1. `backend_id + world_generation` collapses them together,
2. they will alias the same projected home unless lane/worker identity joins the key,
3. multi-lane semantics break before the product ever reaches full named-lane UX.

For v1, this means:

1. per-retained-worker projected state is the right approximation of future per-lane state.

## Lifetime Classes

The framework should recognize three conceptual lifetime classes:

### 1. Ephemeral projection

Used for one-shot work.

Properties:

1. disposable,
2. no future routing obligation,
3. may use temporary projected roots,
4. should not be relied on for durable session/app-runtime state.

### 2. Retained-worker projection

Used for retained world workers in the current architecture.

Properties:

1. exact-targeted,
2. durable for the life of the retained worker under one authoritative session/world binding,
3. isolated from sibling retained workers by default,
4. predecessor of future named-lane projection.

### 3. Future named-lane projection

Used once lanes become first-class.

Properties:

1. may survive worker process replacement,
2. remains subordinate to session/world authority,
3. should preserve the same isolation rules as retained-worker projection.

## Durable Versus Request-Time Projection

The framework must keep these separate.

### Durable projection examples

1. logical inventories,
2. lane-local enablement and overrides,
3. retained projected home roots,
4. retained project-root overlays,
5. non-secret app/runtime registries when policy allows them to persist.

### Request-time projection examples

1. launch env,
2. one-time auth handoff,
3. selected active profile for one run,
4. generated runtime-only config fragments,
5. launcher scripts and scratch files,
6. filtered effective view after final policy narrowing.

Hard rule:

1. a runtime must not gain new durable authority simply because it can edit its own native config file during execution.

## Managed Versus Mutable Projection

The framework must explicitly decide whether a projected native file is:

1. managed and render-only,
2. mutable but reconciled back into Substrate-owned truth,
3. mutable in a runtime-local overlay that never becomes authority.

### Default direction

The safest default is:

1. Substrate-owned logical inventories remain authoritative,
2. native projected files are managed outputs by default,
3. adapter-native writes are either scratch-only or routed through explicit future reconciliation surfaces.

This is especially important for:

1. MCP add/remove flows,
2. remembered approvals,
3. provider/profile changes,
4. future plugin/skill registration flows.

## Secrets and Auth Posture

Projection of ordinary config and delivery of secrets are separate by design.

### Frozen security direction

1. Auth authority remains Substrate-owned.
2. One-time runtime auth delivery should prefer the already-landed gateway handoff seam where possible.
3. A runtime-local projected home may contain compatible auth artifacts only when the adapter contract truly requires them.
4. Keyring-backed or ambient host credential discovery must not become an implicit requirement for world execution.

### Important distinction

The one-time FD handoff is not “Codex reads auth from an FD directly.”

It is:

1. Substrate hands auth once to the in-world `substrate-gateway`,
2. the runtime talks to that local gateway,
3. the gateway applies the auth and forwards provider traffic.

That architecture keeps Codex as a client of the gateway rather than the authority for auth resolution.

## Placement And Realization

Projection authority should be resolved on the host side where:

1. placement-aware backend selection,
2. effective inventory layering,
3. policy evaluation,
4. orchestration-session truth

already live.

Concrete native materialization should happen at the runtime realization layer where:

1. world runtime dirs,
2. launch paths,
3. in-world `HOME`,
4. generated runtime config,
5. secret handoff

already live.

This preserves one authority boundary while allowing host and world realizations to differ mechanically.

### Gateway endpoint posture

When a local Substrate gateway is present, the intended model is:

1. the agent runtime talks to the local Substrate gateway,
2. the gateway is the component that consumes the one-time auth handoff,
3. the gateway, not the agent runtime, owns upstream provider auth application and forwarding.

This matters because:

1. auth-bundle FD handoff is a gateway-facing seam,
2. it should not be misread as a generic agent-native auth ingestion contract,
3. adapter-specific docs should preserve this distinction when describing host and world realizations.

## Multi-Lane Compatibility

This framework must be compatible with the lane-first future documented in [docs/ideas/multi-lane-worlds.md](../docs/ideas/multi-lane-worlds.md).

That means:

1. lane is the operator-facing concept,
2. runtime slot/binding is still a separate runtime identity concept,
3. projected state must not collapse sibling lanes together,
4. lane-local mutable state is the default expectation for parallel agent work.

The framework therefore must not encode “one workspace = one projected agent home” as a permanent assumption.

## Workspace Sync Boundary

Projected agent state must not be assumed safe for normal workspace sync or commit surfaces.

Default direction:

1. retained projected homes should live outside the ordinary workspace tree unless a later design explicitly proves otherwise,
2. scratch/runtime state should remain outside repo-sync surfaces by default,
3. any project-root projection that does exist must be deliberately scoped and treated as compatibility surface, not as a dumping ground for all retained runtime state.

This avoids:

1. accidental sync-back of runtime caches, approvals, or auth-adjacent files,
2. repo contamination from daemon/runtime artifacts,
3. false assumptions that agent-native writable state is equivalent to ordinary source-tree state.

## V1 Summary

For the first implementation-bearing slice that follows this design, the intended direction is:

1. Substrate owns logical inventory and policy truth.
2. Agent-native config/state is projected from that truth.
3. Projection identity includes retained-worker state ownership under one authoritative session/world binding.
4. Request-time secret delivery remains stricter than ordinary config projection.
5. Scratch-by-default mutable runtime state is acceptable initially.
6. Adapter-specific persistence and compatibility details are deferred to adapter-specific DESIGN docs.

## Follow-On Design Dependencies

This framework is intentionally generic. It should be followed by at least:

1. a Codex-specific mapping doc that defines how this framework maps into `CODEX_HOME`, provider/profile posture, MCP/app-runtime projection, and world auth/gateway realization,
2. a future project-overlay design for adapters that depend on repo-local config discovery beyond home-root state.

## Open Design Tensions To Preserve

The next DESIGN docs should answer, not erase, these tensions:

1. when a projected native file may be mutable,
2. whether remembered MCP/app approvals may persist and at what scope,
3. how much project-local `.codex` compatibility should exist,
4. when retained-worker projection should be promoted to first-class named-lane projection,
5. how non-Codex adapters with project-root discovery fit the same framework cleanly.
