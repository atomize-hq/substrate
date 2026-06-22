# Design: Workspace-Scoped Adapter Overlay Model

Status: draft design input. This document defines the workspace-root overlay model that sits beneath the generic [DESIGN-agent-facing-config-projection-framework.md](./DESIGN-agent-facing-config-projection-framework.md) and beside the Codex-specific [DESIGN-codex-world-home-auth-and-config-mapping.md](./DESIGN-codex-world-home-auth-and-config-mapping.md). It is not a Codex-only document, not a final storage-schema document, and not an implementation-ready slice plan. It freezes how Substrate should handle adapter-native config and state that are discovered from the workspace tree rather than from an agent home root.

## Why This Doc Exists

The repo now has two important frozen truths:

1. Substrate-owned logical inventories and effective policy are the architectural authority.
2. Codex-specific home-root projection is not enough to describe every adapter family or every future compatibility surface.

What the repo still does not have is one frozen answer to:

1. when an adapter-native workspace overlay is legitimate,
2. how workspace overlays stay subordinate to Substrate-owned inventories,
3. how mutable adapter-native workspace files avoid silently becoming authority,
4. how workspace overlays compose with retained workers, future named lanes, and multi-lane worlds,
5. how home-root projection and workspace-root projection stay distinct instead of collapsing into one vague “agent state” bucket.

That gap matters now because:

1. future runtime families will not all fit a home-root-only model,
2. even Codex has workspace compatibility surfaces such as workspace `.codex/config.toml`,
3. the lane-first direction in [docs/ideas/multi-lane-worlds.md](../docs/ideas/multi-lane-worlds.md) requires us to answer how sibling workers avoid aliasing the same workspace-discovered adapter state,
4. Substrate’s normal workspace sync model must not accidentally treat adapter-native runtime state as ordinary source-tree state.

This document closes that workspace-overlay gap.

## Relationship To Existing Decisions

This design composes with:

1. [DESIGN-agent-facing-config-projection-framework.md](./DESIGN-agent-facing-config-projection-framework.md): generic authority, identity, lifetime, and durable-versus-request-time rules are inherited from there.
2. [DESIGN-codex-world-home-auth-and-config-mapping.md](./DESIGN-codex-world-home-auth-and-config-mapping.md): Codex-specific home-root mapping remains separate from the generic workspace-overlay rules here.
3. [ADR-0040](../docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md): runtime authority remains Substrate-owned.
4. [ADR-0046](../docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md): backend selection and realization stay Substrate-owned.
5. [ADR-0047](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md): orchestration session remains the durable authority root.
6. [docs/ideas/multi-lane-worlds.md](../docs/ideas/multi-lane-worlds.md): future product direction is lane-first, with sibling mutable state isolated by default.
7. [docs/internals/world/workspace_sync_filesystem_model.md](../docs/internals/world/workspace_sync_filesystem_model.md): workspace sync and protected-path posture remain distinct from adapter runtime state.
8. [docs/contracts/gateway/runtime-parity.md](../docs/contracts/gateway/runtime-parity.md): adapter families should converge on one truthful authority model even if their native discovery roots differ.

## Problem Statement

How might Substrate support adapter-native workspace-root config discovery so that:

1. workspace-native compatibility surfaces can exist when the adapter truly requires them,
2. Substrate-owned logical inventories and effective policy remain the architectural authority,
3. sibling retained workers or future named lanes do not alias each other’s mutable workspace-discovered state,
4. workspace sync and commit surfaces do not accidentally absorb adapter runtime garbage, approvals, caches, or auth-adjacent state,
5. home-root projection and workspace-root projection can coexist cleanly for the same runtime family?

## Frozen Direction

This design freezes the following:

1. Workspace-root adapter files are compatibility surfaces, not primary architectural authority.
2. Substrate-owned logical inventories remain authoritative even when adapter-native workspace files are projected.
3. Workspace overlays are a separate projection class from home-root projection and must not be collapsed together.
4. Managed render-only overlay is the default; mutable adapter-native workspace writes require explicit bounded rules.
5. Workspace overlays must key mutable ownership at the retained-worker or future lane level when the adapter would otherwise discover shared workspace state.
6. Retained adapter state that is not legitimate source-tree material must not persist inside the ordinary workspace sync surface by default.
7. A workspace overlay may be absent entirely for runtime families that do not need one.

## Non-Goals

This design does not:

1. define the final public CLI for editing overlay-backed inventories,
2. define the final file-system layout for every adapter family,
3. declare that every adapter must support workspace overlays,
4. solve Codex home-root mapping again in this document,
5. define final reconciliation semantics for mutable adapter-native writes,
6. promise a v1 implementation slice for non-Codex families.

## Core Principle

Workspace-root discovery is a compatibility obligation, not an authority transfer.

That means:

1. Substrate may project selected truth into workspace-native files when an adapter needs that shape.
2. The existence of workspace `.codex`, `.claude`, `.mcp.json`, or similar files does not grant those files architectural authority parity with Substrate-owned inventories.
3. Mutable adapter-native workspace files must be treated as scratch, overlay, or explicitly reconciled surfaces, never as ambient authority by default.

## Why Workspace Overlay Exists As A Separate Model

Home-root projection and workspace-root projection solve different problems.

### Home-root projection

Used when the adapter primarily discovers state from an app home.

Examples:

1. `CODEX_HOME`,
2. adapter-local auth/session/history/log state,
3. adapter-local MCP or app-runtime registries stored under home state.

### Workspace-root projection

Used when the adapter discovers state from the current workspace tree.

Examples:

1. workspace `.codex/config.toml`,
2. future `.claude/**`,
3. `.mcp.json`,
4. workspace-local skills or plugin overlays.

Hard rule:

1. a runtime family may need one, the other, or both;
2. Substrate must not assume that a home-root projection alone is enough for every adapter family.

## Overlay Classes

The model should distinguish at least three workspace-overlay classes.

### 1. Managed render-only overlay

Properties:

1. generated by Substrate,
2. adapter may read it,
3. adapter writes are ignored, blocked, or overwritten,
4. authoritative truth remains outside the workspace tree.

This should be the default posture for v1.

### 2. Runtime-local mutable overlay

Properties:

1. adapter may mutate it at runtime,
2. mutations remain scoped to one retained worker or future lane,
3. mutations do not automatically reconcile back into Substrate-owned truth,
4. overlay may be discarded, reset, or replaced without implying loss of architectural authority.

This is acceptable only when the overlay is explicitly bounded and not treated as normal workspace content.

### 3. Explicit reconciliation overlay

Properties:

1. adapter-native writes may be ingested back into Substrate-owned truth,
2. reconciliation is explicit rather than ambient,
3. policy and drift detection decide what is accepted,
4. resulting logical inventory changes remain host-authoritative.

This is future work, not the default posture.

## Authority And Precedence

When a workspace overlay exists, precedence should still be:

1. Substrate-owned logical inventory,
2. effective host-side resolution for one exact backend/session/lane target,
3. generated native workspace overlay,
4. runtime-local mutable overlay effects only if explicitly allowed,
5. no silent promotion of adapter-native overlay writes back into authority.

In other words:

1. workspace-native files are outputs of effective Substrate truth,
2. they are not peers of that truth.

## Identity And Ownership

Workspace overlay ownership must follow the same lane-first direction as projected home state.

### Scope boundary

Good scope boundary inputs include:

1. `orchestration_session_id`,
2. authoritative world binding,
3. exact backend id,
4. workspace root.

### Owner dimension

That scope boundary is still too coarse for parallel same-backend work.

Mutable ownership must additionally include:

1. retained-worker identity in v1,
2. future named-lane identity later.

Hard rule:

1. sibling same-backend retained workers must not alias the same mutable workspace overlay by default.

## Lifetime Classes

Workspace overlay lifetime should align with the generic framework.

### 1. Ephemeral overlay

Used for one-shot work.

Properties:

1. disposable,
2. no future routing obligation,
3. may be rendered on demand,
4. should not be relied on for durable adapter-local state.

### 2. Retained-worker overlay

Used in v1 when a runtime genuinely needs mutable workspace-discovered state.

Properties:

1. exact-targeted,
2. durable only for the life of that retained worker under one authoritative session/world binding,
3. isolated from sibling retained workers by default,
4. predecessor of future named-lane overlay.

### 3. Future named-lane overlay

Used when lanes become first-class.

Properties:

1. may survive worker process replacement,
2. remains subordinate to session/world authority,
3. preserves the same isolation rules as retained-worker overlay.

## Workspace Overlay Placement

Substrate must not assume the correct placement is “inside the ordinary source tree.”

Default direction:

1. managed runtime overlays should prefer Substrate-controlled paths outside the ordinary synced workspace tree,
2. if the adapter contract truly requires a repo-visible file shape, that shape should be rendered deliberately and minimally,
3. retained runtime garbage, caches, daemon state, approvals, and auth-adjacent material should remain outside normal workspace sync surfaces by default.

This matters because:

1. ordinary source-tree sync is not the same thing as adapter runtime state management,
2. runtime artifacts can leak back into commits or host sync unexpectedly,
3. multi-lane isolation breaks if one shared repo-visible overlay is reused casually.

## Import And Export Compatibility

This design allows for bounded compatibility with existing workspace-native adapter files.

### Import compatibility

Possible future behavior:

1. Substrate may ingest selected workspace-native adapter files into logical inventory,
2. ingestion remains explicit and bounded,
3. imported values are filtered by policy and runtime-family support.

### Export compatibility

Possible future behavior:

1. Substrate may render effective inventory into workspace-native files for adapter compatibility,
2. rendered files remain subordinate outputs,
3. export does not create a second authority plane.

Hard rule:

1. import/export compatibility must not become ambient bidirectional drift.

## Managed Versus Mutable Overlay

### Default direction

By default:

1. workspace overlay is managed and render-only,
2. runtime-local adapter writes are scratch or bounded worker/lane-local overlay behavior,
3. no automatic reconciliation back into Substrate-owned truth exists.

### Why this matters

Without this rule:

1. remembered approvals,
2. adapter-native MCP edits,
3. provider/profile changes,
4. future skill/plugin mutations

would silently move architectural authority into workspace-local files.

## Sync And Commit Boundary

Workspace overlay must stay compatible with Substrate’s normal sync model.

Default direction:

1. retained adapter overlays should live outside ordinary workspace sync surfaces unless a later design proves otherwise,
2. repo-visible compatibility files should be minimal, deliberate, and bounded,
3. protected-path and sync posture may need to grow to recognize adapter overlay roots explicitly when they are materialized.

This avoids:

1. accidental commit of runtime-local adapter state,
2. sync-back of approvals, caches, or daemon files,
3. false assumptions that adapter-native writable state is equivalent to ordinary source files.

## Codex-Specific Implication To Preserve

For Codex specifically:

1. the primary steady-state mapping remains explicit projected `CODEX_HOME`,
2. workspace `.codex/config.toml` should remain compatibility-only unless a later explicit slice grants it a bounded managed role,
3. Codex home-root mapping and Codex workspace-overlay compatibility should remain distinct concerns.

This is why the Codex mapping doc and this workspace-overlay doc remain separate.

First-pass note:

1. repo-visible Codex compatibility surfaces should not be used to compensate for missing
   home-root/state projection constraints,
2. if worker- or lane-local Codex isolation fails because non-`CODEX_HOME` discovery paths remain
   ambient, the fix belongs first in the Codex home/auth/config mapping and generic projection
   boundary, not in granting broader authority to workspace `.codex`,
3. workspace-visible projection may still become a useful bounded lever for things like projected
   MCP or skills compatibility later, but only as a subordinate compatibility surface beneath the
   managed home-root/state model.

## Adapter-Family Stretch Goal To Preserve

This document exists largely because future adapters may depend more heavily on workspace discovery than Codex does.

Examples:

1. adapters that store most config under workspace-local directories,
2. adapters that discover MCP configuration from repo-visible files,
3. adapters that require workspace-local skill, plugin, or tool overlays.

The generic overlay model here should stretch to them without forcing Codex-specific assumptions onto every runtime family.

## V1 Summary

For the first implementation-bearing slices that depend on this design, the intended direction is:

1. Substrate-owned logical inventories remain the only architectural source of truth.
2. Workspace-root overlay is a compatibility projection class, not a second authority plane.
3. Managed render-only overlay is the default posture.
4. Mutable overlay, when needed, is scoped per retained worker under one authoritative session/world binding.
5. Adapter runtime garbage, caches, and auth-adjacent material remain outside normal workspace sync surfaces by default.
6. Import/export compatibility with workspace-native files is explicit future work rather than ambient drift.

## Follow-On Design Dependencies

This design should feed:

1. adapter-specific DESIGN docs that need workspace-overlay rules beyond home-root mapping,
2. numbered implementation SPECs that choose the first bounded adapter family or compatibility slice,
3. possible future design work for explicit reconciliation of selected mutable adapter-native workspace writes.

## Open Design Tensions To Preserve

The next numbered SPECs should answer, not erase, these tensions:

1. when a repo-visible compatibility file is justified at all,
2. which adapter-native workspace writes may ever reconcile back into Substrate-owned truth,
3. whether protected-path and sync rules need explicit adapter-overlay expansion,
4. how future non-Codex adapters with heavy workspace discovery fit without making workspace overlays universal,
5. when retained-worker overlay should promote to first-class named-lane overlay,
6. how workspace overlay and home-root projection coexist for one runtime family without split-brain drift.
