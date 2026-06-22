# Design: Codex World Home, Auth, and Config Mapping

Status: draft design input. This document defines the Codex-specific mapping that sits under the generic [DESIGN-agent-facing-config-projection-framework.md](./DESIGN-agent-facing-config-projection-framework.md). It is not a generic adapter document, not a final storage-schema document, and not an implementation-ready slice plan. It freezes how Substrate-owned authority should map into Codex-native home/config/runtime surfaces for world-scoped execution.

## Why This Doc Exists

The repo now has a generic projection framework that freezes:

1. Substrate-owned logical inventory and policy authority,
2. lane- or worker-local projected state ownership,
3. durable-versus-request-time projection boundaries,
4. stricter secret handoff posture than ordinary config projection.

What the repo still does not have is the Codex-specific answer to:

1. which Codex-native roots Substrate should project,
2. how world-scoped Codex auth should arrive,
3. which Codex-native state is scratch, retained, or forbidden by default,
4. how Codex-specific config domains such as models, providers, MCP servers, and app runtimes fit under Substrate-owned inventories,
5. how this mapping should compose with retained workers now and future named lanes later.

That gap matters because current `cli:codex-world` execution still depends on a narrow compatibility bridge: it seeds an isolated per-member `CODEX_HOME` from host `~/.codex` artifacts at launch time. That bridge is useful for continuity, but it is not the intended steady-state architecture.

This document closes the Codex-specific mapping gap.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-agent-facing-config-projection-framework.md](./DESIGN-agent-facing-config-projection-framework.md): generic authority, projection, identity, and persistence rules are inherited from there.
2. [ADR-0040](../docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md): `substrate-gateway` remains the runtime/auth boundary.
3. [ADR-0046](../docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md): backend selection and runtime integration stay Substrate-owned.
4. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): orchestration session remains the durable authority root.
5. [docs/contracts/gateway/runtime-parity.md](../docs/contracts/gateway/runtime-parity.md): prompt-bearing world execution should converge on one truthful gateway-mediated runtime seam rather than bespoke backend-registration paths.
6. [docs/internals/world/gateway_auth_handoff.md](../docs/internals/world/gateway_auth_handoff.md): one-time auth-bundle FD handoff into the in-world gateway is already the landed secure delivery seam.
7. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md): world work remains exact-targeted and lifecycle-explicit.
8. [docs/ideas/multi-lane-worlds.md](../docs/ideas/multi-lane-worlds.md): future product direction is lane-first, not one shared world home per workspace.
9. [docs/CONFIGURATION.md](../docs/CONFIGURATION.md): placement-qualified exact backend ids such as `cli:codex-host` and `cli:codex-world` remain policy and routing truth today.

## Problem Statement

How might Substrate map its own authoritative inventories, policy, and auth posture into Codex-native home/config/runtime surfaces so that:

1. world-scoped Codex execution does not depend on ambient host `~/.codex` discovery,
2. the in-world runtime can still see a truthful Codex-native state root,
3. retained workers and future named lanes get isolated mutable state by default,
4. MCP/app-runtime/provider/profile configuration can be projected intentionally rather than copied ad hoc,
5. secret delivery stays stricter than ordinary config projection,
6. Codex-specific mutable writes do not silently become the architectural source of truth.

## Frozen Direction

This design freezes the following:

1. Codex world projection is rooted around an explicit projected `CODEX_HOME`.
2. Rebasing `HOME` alone is not the primary Codex projection mechanism; it is only a fallback/default-path concern.
3. Substrate-owned inventories remain authoritative; Codex-native `config.toml` and related files are projection outputs, not primary source of truth.
4. V1 mutable state ownership is per retained worker under one authoritative orchestration session and one authoritative world binding.
5. Future named lanes should promote that owner dimension from retained worker to lane without collapsing back to one shared home per workspace or per world.
6. Auth authority remains Substrate-owned and the landed gateway handoff seam is the core product stance for world-scoped Codex execution.
7. A Codex-projected home may contain compatible auth artifacts only as a bounded compatibility mechanism when the runtime contract truly requires them.
8. Scratch-by-default writable Codex state is acceptable in v1; durable writable Codex homes must be explicit, scoped, and remain outside normal workspace sync surfaces.
9. Runtime-local Codex writes do not become architectural truth merely because Codex can mutate its own native files.

## Non-Goals

This design does not:

1. define the final host-side UI or public CLI for editing Codex inventories,
2. define the final on-disk path for retained projected Codex homes,
3. define non-Codex adapter mappings,
4. promise the exact implementation slice that retires the current direct seed-home compatibility bridge,
5. solve app-server-first product UX or IDE integration in this document,
6. define the final project-overlay contract for adapters that depend on repo-local config discovery beyond home-root state,
7. declare that placement-qualified identity cleanup is fully resolved in this document.

## Core Principle

Codex should see a **managed Codex-native home**, not ambient host state.

That means:

1. Substrate owns the logical config and auth authority.
2. Codex sees a projected `CODEX_HOME` and any other runtime inputs required by the selected realization.
3. Codex-native writes may exist, but they are managed outputs, scratch state, or bounded overlays unless a later explicit reconciliation contract says otherwise.

## Current Codex Reality This Mapping Must Respect

This design assumes current Codex behavior as follows:

1. `CODEX_HOME` remains the explicit override for the Codex home/state root.
2. When `CODEX_HOME` is absent, Codex falls back to a default home under the user home directory.
3. Codex-native state spans more than one file:
   - `config.toml`,
   - auth artifacts,
   - session and rollout state,
   - logs and caches,
   - MCP OAuth state,
   - app-server daemon state,
   - skills and related agent-local material where supported.
4. Codex supports multiple credential-store postures including file, keyring, automatic fallback, and ephemeral process-local handling.
5. Codex-native config layering and mutable writes can otherwise drift away from a host-owned authority if left unmanaged.

The mapping here is designed around that reality.

## Codex Projection Layers

This Codex-specific mapping divides into four Codex-facing layers under the generic framework.

### 1. Logical Codex inventory layer

Substrate-owned truth may include:

1. allowed model/provider selections,
2. provider profiles,
3. MCP server definitions,
4. app-runtime definitions,
5. allowed adapter-specific feature toggles,
6. lane- or worker-local enablement and narrowing,
7. future Codex-specific compatibility settings.

This layer is not stored primarily as raw Codex-native files.

### 2. Effective Codex mapping layer

This is the Codex-specific resolved view for one exact runtime target.

Inputs include:

1. exact backend id,
2. orchestration session,
3. retained-worker or future lane identity,
4. authoritative world binding,
5. policy narrowing,
6. runtime-family-specific capabilities and restrictions.

This layer remains Substrate-owned truth.

### 3. Native Codex home projection layer

This is the concrete Codex-native rendering.

Examples:

1. projected `CODEX_HOME`,
2. projected `config.toml`,
3. projected app-runtime and MCP registry entries,
4. projected non-secret compatibility files,
5. Codex-local scratch state roots,
6. optional retained Codex home roots when explicitly allowed.

### 4. Launch-time auth and secret delivery layer

This remains stricter than ordinary Codex config projection.

Examples:

1. gateway-facing one-time auth-bundle FD handoff,
2. bounded env injection only where a runtime contract truly requires it,
3. bounded compatibility auth artifacts in projected home roots when the runtime cannot avoid them.

Hard rule:

1. the existence of a projected `CODEX_HOME` does not mean Codex itself becomes the authority for credential discovery.

## Root Mapping Posture

### 1. `CODEX_HOME` is the explicit home-root projection knob

The Codex-specific projected home should be realized through an explicit `CODEX_HOME` value.

Why:

1. it is the Codex-native explicit override,
2. it gives Substrate a direct home-root compatibility seam,
3. it avoids relying on implicit default-path behavior from rebased `HOME`.

### 2. `HOME` rebasing is secondary

The world runtime may still rebase `HOME` for world isolation and generic process hygiene, but for Codex mapping:

1. `HOME` rebasing is not sufficient by itself,
2. `CODEX_HOME` remains the primary explicit projection handle,
3. any Codex design that depends only on `HOME` is too weak for precise authority and compatibility control.

### 3. Workspace-root projection is compatibility-only

Codex-related project overlays such as project `.codex/config.toml` should be treated as compatibility surfaces, not primary authority.

Default direction:

1. Substrate-owned logical inventories remain the only architectural source of truth for config, policy, and allowed runtime posture,
2. project `.codex` compatibility, if present at all, is import/export/overlay territory beneath that authority rather than a second source of truth,
3. a projected Codex home must not require repo-root `.codex` writes in v1,
4. later slices may define bounded compatibility import or export behavior, but they must not grant project `.codex` authority parity with Substrate-owned inventories.

## Lifetime And Owner Model

### V1 owner model

V1 projected Codex state should be owned per retained worker.

That means:

1. not per launch for retained work,
2. not one shared `CODEX_HOME` for the whole orchestration session,
3. not one shared `CODEX_HOME` for the whole authoritative world.

Instead:

1. `orchestration_session_id + authoritative world binding + backend_id` supplies the scope boundary,
2. retained-worker identity supplies the mutable owner dimension.

Note:

1. the repo has already landed some real shared-world and multi-member posture, and that existing truth is part of why one session/world scope boundary can contain multiple worker-owned projections;
2. this document does not attempt to restate or re-prove that posture in full;
3. the exact repo-truth closeout for shared-world multi-member posture should be researched explicitly in a later focused pass rather than invented here from memory.

### Future promotion

When lanes become first-class, that owner dimension should promote from retained worker to named lane.

Hard rule:

1. same-backend sibling workers must not alias the same projected Codex home by default.

## Durable Versus Request-Time Codex Mapping

### Durable Codex mapping examples

Potentially durable, when explicitly enabled and scoped:

1. rendered `config.toml`,
2. rendered MCP server definitions,
3. rendered app-runtime definitions,
4. retained projected `CODEX_HOME` roots,
5. retained non-secret compatibility state allowed at worker or future lane scope.

### Request-time Codex mapping examples

Request-time only:

1. launcher env,
2. one-time auth handoff,
3. selected active model/provider narrowing for one run,
4. generated runtime-only fragments,
5. launcher scripts and scratch directories,
6. final filtered view after policy application.

### V1 default

The v1 default should remain conservative:

1. scratch-by-default writable Codex home behavior,
2. no assumption that general Codex writes persist durably,
3. any retained writable Codex home must be explicit and bounded.

## Codex Auth Mapping Posture

### Gateway-front-door steady state

The intended steady state, and the core product stance, is:

1. Substrate resolves auth on the host side,
2. Substrate delivers auth once to the in-world gateway through the landed handoff seam,
3. Codex talks to the in-world `substrate-gateway`,
4. the gateway applies upstream auth and forwarding.

This is the required target architecture because:

1. it keeps auth authority inside Substrate,
2. it avoids ambient host credential discovery inside the world,
3. it preserves one truthful secret-delivery seam.

Hard rule:

1. once this gateway-front-door realization is fully landed for world-scoped Codex execution, compatibility home-auth projection must be removed rather than kept as a normal fallback;
2. world-scoped Codex execution should fail closed rather than silently re-enter ambient host-home discovery or long-lived copied-auth behavior.

### Important distinction

The FD handoff is not “Codex reads auth from an FD directly.”

It is:

1. Substrate hands auth once to the in-world gateway,
2. the gateway consumes it,
3. Codex uses the local gateway as client.

### Compatibility bridge posture

When a Codex realization truly cannot avoid home-compatible auth artifacts, Substrate may project bounded compatible auth files into `CODEX_HOME`.

That is a compatibility bridge, not the target authority model.

It therefore has these limits:

1. it exists only to bridge from current repo reality into the gateway-front-door architecture,
2. it must not be described as a parallel steady-state auth model,
3. it must not become a permanent fallback once the gateway-front-door path is complete,
4. later implementation planning should include explicit retirement criteria for this bridge.

### Keyring posture

World execution must not require ambient host keyring access as an implicit dependency.

V1 default direction:

1. prefer gateway handoff where possible,
2. prefer file- or ephemeral-compatible in-world auth postures over automatic keyring discovery,
3. treat keyring bridging as explicit future design work, not ambient behavior.

## Codex Config Domain Mapping

The Codex mapping should treat different config domains differently.

### 1. Model and provider posture

Substrate may own:

1. allowed models,
2. provider profiles,
3. per-lane or per-worker narrowing,
4. safety constraints around active provider selection.

Codex-native realization may use:

1. projected `config.toml` values,
2. request-time launch overrides,
3. gateway-mediated upstream routing posture.

Hard rule:

1. there must be one authoritative answer for “which provider/model is active now.”

Substrate must not allow projected native config and gateway/runtime routing to drift into split-brain truth.

### 2. MCP server posture

MCP definitions are legitimate inventory candidates for Substrate ownership.

Default direction:

1. Substrate owns the logical MCP inventory,
2. Codex-native `[mcp_servers]` entries are projected outputs,
3. runtime-local MCP writes are scratch or bounded overlay behavior unless a later explicit reconciliation contract exists.

### 3. App-runtime posture

App-runtime definitions should follow the same authority rule as MCP definitions:

1. Substrate-owned logical inventory remains primary,
2. Codex-native app-runtime entries are mapped outputs,
3. daemon/runtime files remain scratch/runtime-local by default.

### 4. Skills and plugin posture

V1 should stay conservative.

Default direction:

1. do not assume a fully solved durable Codex-native skills/plugin authority plane in this document,
2. treat any future skills/plugin mapping as follow-on work unless it can be expressed cleanly through the same logical-inventory model,
3. do not let ambient host extension roots become implicit world dependencies.

## Managed Versus Mutable Codex Projection

### Default direction

By default:

1. projected `config.toml` is a managed output,
2. projected MCP/app-runtime definitions are managed outputs,
3. runtime-local Codex writes are scratch or overlay state,
4. remembered approvals or mutable Codex-native edits do not automatically reconcile back into Substrate-owned truth.

### Why this matters

Without this rule:

1. `codex mcp add/remove`,
2. remembered MCP approvals,
3. provider/profile mutations,
4. app-server config writes

would silently move architectural authority out of Substrate and into runtime-local files.

### Future reconciliation

A later slice may add explicit reconciliation surfaces.

That future work must answer:

1. which Codex-native writes are allowed to reconcile,
2. what scope they reconcile at,
3. what policy approval is required,
4. how drift is detected and rendered.

This design does not grant that behavior by default.

## Persistence And Workspace Sync Boundary

### V1 default

V1 should not treat writable Codex-native state as normal workspace state.

Default direction:

1. projected retained Codex homes should live outside ordinary workspace sync surfaces,
2. scratch/runtime Codex state should stay outside repo-sync and commit surfaces,
3. Codex daemon/runtime artifacts should remain outside the workspace tree by default.

### Why

This avoids:

1. accidental sync-back of runtime caches, approvals, or auth-adjacent files,
2. repo contamination from daemon and runtime artifacts,
3. false equivalence between Codex-native state and ordinary source-tree state.

### Persistence modes to preserve

This design preserves room for at least:

1. scratch/ephemeral Codex homes,
2. retained per-worker or future per-lane Codex homes,
3. later explicit per-project retained homes,
4. later stronger cross-session retained-home models if separately justified.

It does not endorse a shared user/global retained world home in v1.

## Placement And Identity Tension To Preserve

This document intentionally does not fully retire the current placement-qualified identity tension.

Today:

1. exact backend ids such as `cli:codex-world` remain policy and routing truth,
2. some gateway-facing seams normalize toward the canonical runtime family.

This design freezes only the part needed now:

1. Codex projected-state ownership must not alias sibling same-backend workers,
2. the eventual identity cleanup may be deferred,
3. the owner key must still include retained-worker identity now and future lane identity later.

## Current Gap Versus Target Architecture

Current direct world Codex posture:

1. shell injects a host seed-home hint,
2. `world-service` creates a temporary isolated `CODEX_HOME`,
3. host Codex artifacts are copied into that home,
4. direct member execution still bypasses the fully managed gateway-facing auth architecture.

Target posture:

1. Substrate-owned logical inventories resolve on the host,
2. Codex-native home/config state is projected intentionally,
3. secrets prefer gateway handoff,
4. projected mutable state is lane- or worker-local,
5. compatibility bridges do not become architectural truth.

## V1 Summary

For the first implementation-bearing Codex-specific slice after this design:

1. explicit projected `CODEX_HOME` is the Codex home-root mapping seam,
2. world-scoped mutable Codex state is owned per retained worker under one authoritative session/world binding,
3. scratch-by-default writable Codex state is acceptable initially,
4. durable writable Codex homes must be explicit and remain outside workspace sync surfaces,
5. gateway-front-door auth delivery is the target steady state and compatibility home-auth projection is transitional only,
6. Codex-native mutable writes do not become authority by default,
7. project `.codex` compatibility, if it exists, remains strictly subordinate to Substrate-owned inventories.

## Follow-On Design Dependencies

This document should be followed by:

1. a future project-overlay design for adapters that depend on repo-local discovery beyond home-root projection,
2. later implementation SPECs that choose the first bounded Codex mapping slice,
3. possible future design work for explicit reconciliation of selected mutable Codex-native writes.

## Open Design Tensions To Preserve

The next numbered SPECs should answer, not erase, these tensions:

1. when retained writable Codex homes become necessary beyond scratch-by-default posture,
2. whether remembered MCP/app approvals may persist and at what scope,
3. whether project `.codex/config.toml` remains compatibility-only or gains a bounded managed role,
4. how keyring-derived auth, if ever needed, is bridged without reintroducing ambient host discovery,
5. how app-server daemon state should be treated when app-server UX becomes a first-class product surface,
6. when retained-worker ownership should be promoted to first-class named-lane ownership,
7. the exact repo-truth description of the currently landed shared-world / multi-member posture that this Codex mapping must compose with.
