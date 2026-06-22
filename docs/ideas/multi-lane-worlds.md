# Multi-Lane Worlds

## Status

Draft idea capture consolidating the full context from the original multi-world discovery session and the follow-on lane-first refinement discussion.

## Problem Statement

How might we let a single Substrate workspace host multiple concurrent execution lanes with distinct policy, isolation, and environment behavior, without breaking today’s shared-world orchestration model, workspace sync semantics, or backend portability?

## Why this matters now

Substrate is getting closer to being a fully functioning host that orchestrates in-world agents. The next meaningful refinement is not just “can agents run in a world,” but “can one workspace safely support multiple concurrent worlds or world-like execution contexts without those agents stepping on each other?”

This matters for at least three classes of use cases:

1. **Parallel agent work lanes**
   - One orchestration session spawns multiple agents that should work concurrently without sharing the same writable state by default.
   - Some shared code may need to be frozen by policy while lane-local work remains writable.

2. **Environment lanes**
   - The same workspace may want stable named lanes such as `dev`, `staging`, `production`, `review`, or `research`.
   - These lanes should be operator-visible and intentionally targetable.

3. **Platform or image lanes**
   - The same code may need to run against different rootfs, distro, backend, or image realizations.
   - This becomes more important as guest-rootfs and richer backend selection mature.

The key product question is therefore broader than “multiple worlds.” The real question is how Substrate should model **parallel execution contexts inside a workspace** in a way that scales across agent work, environment variants, and eventually distro/image variants.

## Current repo reality

The current codebase is still structurally biased toward either:

1. **one generic reusable world per workspace-shaped spec**, or
2. **one active shared world per orchestration session**,

with multiple member agents intentionally attaching into that same shared world.

### Grounded findings

- `project_dir` is the root identity input, and under normal workspace anchoring it resolves to the workspace root rather than the leaf cwd. Many calls inside one workspace therefore collapse onto the same world root.
- Generic world reuse is currently keyed narrowly around compatibility of `project_dir`, network posture, and isolation posture.
- Reused worlds currently mutate `fs_mode` in place on reuse rather than treating it as a stable identity dimension for concurrently live sibling worlds.
- Shared-owner reuse is explicitly keyed by `orchestration_session_id`, and recovery fails closed if it finds multiple active shared worlds for that owner.
- Member dispatch currently routes member runtimes into the same authoritative shared world using the orchestration session id.
- Member placement validation depends on exact binding proof: `orchestration_session_id + world_id + world_generation`.
- Pending diff, workspace sync, and world file read flows are still modeled like there is one current world for a workspace-shaped spec rather than multiple sibling worlds that need explicit targeting.
- Backend locking is still coarse enough that true parallel execution across sibling worlds is not a first-class property today.

### Bottom line

Today’s model does **not** naturally represent “one workspace can own multiple separate concurrent sibling worlds.” It represents:

- a single generic reusable world for a workspace-shaped spec, or
- a single shared owner-bound world for an orchestration session,

plus multiple member agents living inside that one authoritative shared world.

## Core conceptual shift

The primary operator concept should **not** be “world instance.”

The primary operator concept should be **lane**.

### Proposed model

- **Lane** = the operator-facing execution context or intent
- **World** = the runtime realization of that lane

That lets the same abstraction span:

- parallel agent work lanes
- environment lanes like `dev | staging | production`
- distro/image/rootfs/backend-specific lanes

This is cleaner than exposing raw world ids as the main UX. Operators think in named contexts; the runtime can still realize those contexts as concrete worlds with explicit proof surfaces.

## Terminology and separation of concerns

To preserve both the early architectural findings and the later product refinement, the model should keep two layers distinct:

### 1. Lane (product and configuration primitive)
A lane is what users and orchestrators define and target.

A lane may eventually carry:

- a name / selector
- policy overrides
- overlay mode (`shared`, `isolated`, or future variants)
- diff/sync/merge behavior
- optional rootfs / image / distro / backend pin
- optional agent affinity or allowed runtime rules
- optional ownership hints or usage constraints

### 2. World slot or lane runtime identity (runtime primitive)
A lane still needs a runtime identity dimension so Substrate can distinguish sibling realizations safely.

The earlier session described this as a first-class `world_slot_id` or `world_instance_id`. That remains important.

In other words:

- **Lane** is the operator-facing concept.
- **World slot identity** is the runtime allocation/binding concept.

A good design should preserve both rather than collapsing them into one vague term.

## Recommended direction

### High-level direction

Adopt a **lane-first model** where a workspace can define multiple named lanes, and each lane has a world realization strategy.

### Why this is the right direction

- It extends the existing shared-owner/world model instead of fighting it.
- It gives operators a stable and understandable abstraction.
- It can support both near-term parallel agent workflows and longer-term environment/image matrices.
- It provides a natural place to attach policy, filesystem behavior, and future versioning/sync semantics.

## V1 prioritization

The current conversation refined the v1 target beyond the earlier “quick win” framing.

### Initial quick-win thought

An earlier staging idea was:

- Phase A: shared-base lanes with selective freeze
- Phase B: isolated overlay lanes with explicit merge-back
- Phase C: image/rootfs-backed lanes

That staging is still useful as a broad roadmap, but it is **not enough to define true v1** given the clarified product goal.

### Updated v1 bar

For v1 to count as real, it should support:

- **one orchestration session spawning multiple agent lanes**
- **distinct writable filesystem state per lane**
- **policy/isolation differences per lane**
- **explicit lane targeting**
- **no automatic lane selection or routing**

So the v1 recommendation is:

> **Session-owned named lanes with per-lane isolated writable overlay state for safe parallel agent work.**

That is a stricter and more meaningful definition than a shared-base-only control-plane feature.

## Primary v1 user and success criteria

### Primary v1 user

The primary first-version operator is:

- **one orchestration session spawning multiple agent lanes**

This does not exclude broader users later, but it is the sharpest first product target.

### Success for v1

V1 is successful if:

- parallel agents can safely work in the same workspace
- those agents do not step on each other’s writable state by default
- policy and isolation can differ per lane
- lane identity is visible and explicit in the control plane and runtime surfaces
- the model can later grow into environment lanes and image-backed lanes without changing the primary abstraction

## Ownership and targeting model

Both the orchestration layer and the operator should be able to target lanes, but the authority boundary should be explicit.

### Recommended ownership model

- **Canonical authority**: the orchestration session owns the lane namespace used during that orchestration.
- **Operator control**: users/operators can still explicitly choose or inspect lanes.
- **No hidden routing in v1**: lanes should not be auto-created or auto-routed behind the operator’s back.

This keeps “both can target lanes” while preventing authority confusion.

## Lane modes and long-term generality

The broader context absolutely includes more than just agent lanes.

### Mode 1: Parallel agent work lanes
Examples:

- `builder-a`
- `builder-b`
- `review`
- `research`

These are the main v1 focus.

### Mode 2: Environment lanes
Examples:

- `dev`
- `staging`
- `production`
- `qa`

These are important future operator workflows and should fit naturally into the same abstraction.

### Mode 3: Platform or image lanes
Examples:

- `ubuntu-24`
- `debian-12`
- `guest-rootfs`
- `host-native`

These become important as rootfs/image-backed realization matures.

### Mode 4: Hybrid or policy-shaped lanes
Examples:

- isolated lane for code generation
- shared-base lane with frozen core paths
- read-only review lane
- environment lane with stricter network or filesystem posture

These are not necessarily first-class product modes, but they show why the lane abstraction must be able to carry multiple behavior dimensions.

## Lane capabilities the model should eventually carry

A lane may eventually define or influence:

- lane identity / name
- shared vs isolated overlay behavior
- per-lane writable state
- per-lane pending diff and sync surface
- merge-back strategy
- policy overrides
- frozen/shared code rules
- allowed writable path classes
- network posture
- backend selection
- rootfs/image/distro pinning
- agent affinity or allowed runtime selection
- orchestration ownership or visibility rules
- future git/versioning integration semantics

Not all of these belong in v1, but the model should be shaped so they can attach naturally later.

## Configuration direction

A promising configuration shape is to mirror the existing style used for other Substrate concepts such as agents and deps.

### Proposed config locations

- global: `~/.substrate/lanes/`
- workspace: `<workspace>/.substrate/lanes/`

### Lane definition shape (conceptual)

Each lane definition may eventually include:

- `name`
- `base_mode`: shared or isolated
- `policy_overrides`
- `rootfs` / `image` / `backend` pin
- `sync_behavior`
- `merge_behavior`
- `agent_affinity`
- optional visibility or ownership metadata

The main point is not the exact schema yet. The main point is that the operator should define named lanes in a familiar global/workspace-scoped config model.

## Runtime identity and allocation model

The runtime still needs an unambiguous identity model for sibling worlds.

### Required principle

**Lane identity must become part of world allocation identity.**

If lane identity does not participate in allocation identity, lanes are mostly cosmetic and Substrate will recreate today’s ambiguity under a new name.

### Minimum identity dimensions to plan for

At minimum, the allocation model wants to distinguish by some combination of:

- orchestration session id
- lane id / lane selector
- effective isolation mode
- effective policy identity
- backend/rootfs/image identity
- network posture
- world realization mode

The exact final key can evolve, but the architecture must stop assuming that one workspace-shaped spec implies one current world.

## Filesystem model

### V1 requirement

Each lane in true v1 should have:

- its own writable overlay state
- its own pending diff surface
- its own sync target
- its own eventual merge-back semantics

### Why this matters

This is the essential difference between:

- a control-plane-only lane label, and
- a real multi-lane world model.

If all lanes share one writable state, then agents are still fundamentally cohabiting the same mutable world.

### Future flexibility

Longer term, the system may support:

- fully isolated overlays
- shared-base overlays with selective freeze
- opt-in shared overlay groups
- read-only review lanes

But the product must not confuse “shared-base selective freeze” with “true isolated lane state.”

## Policy model

Per-lane policy is central to the design.

### Recommended direction

Use:

- base session or workspace policy
- plus lane-local policy overrides

### Strong bias

Lane-local policy composition should prefer:

- **restrictive or additive narrowing**, rather than arbitrary widening or wholesale replacement

That makes policy easier to reason about and reduces the chance that lane configuration becomes a source of surprising access expansion.

### Important use case

Selective freeze should remain part of the design even if v1 prioritizes isolated writable lanes, because selective freeze still matters for:

- protecting shared/core code
- review-only or read-only lanes
- hybrid future modes

## Sync, diff, merge, and future versioning

The conversation explicitly included the need for future richer versioning via deeper git integration.

### Immediate implication

Pending diff, sync, and world file read surfaces must become **lane-targeted** rather than implicitly addressing “the current world.”

### Future implication

Future git/versioning should understand lane outputs as lane-scoped artifacts or change surfaces rather than only generic workspace diffs.

This means the lane abstraction should leave room for:

- explicit merge-back workflows
- lane-aware sync and reconcile operations
- versioned promotion or comparison between lanes
- richer environment or distro matrix workflows later

### What not to do

Do not attempt to ship a full multi-lane merge engine in v1.

## Shared-base selective freeze vs isolated overlays

This is an important nuance from the discussion.

### Shared-base selective freeze

Pros:

- fastest path
- lower implementation cost
- immediately useful for some parallel workflows
- good precursor contract for the larger design

Cons:

- not true isolation
- still leaves meaningful collision risk
- not enough to satisfy the clarified v1 requirement on its own

### Isolated overlays with explicit merge-back

Pros:

- real multi-world behavior
- safer for parallel agents
- clearer ownership of writes and diffs
- stronger long-term foundation

Cons:

- requires better allocation identity
- requires lane-targeted sync/diff semantics
- requires more explicit merge/versioning design

### Final take

Shared-base selective freeze is still worth keeping in the roadmap, but the clarified v1 target is **isolated writable lane state**, not shared-base-only behavior.

## ADR and dependency posture

Two ADRs were explicitly part of the original discovery session and remain important context.

### ADR-0010
`/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/adr/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`

This should land **first or alongside** the multi-lane work.

Why:

- multi-lane changes allocation identity
- multi-lane changes capability surfacing
- multi-lane changes backend contract expectations
- this should not become Linux-only implementation truth with contract cleanup deferred indefinitely

### ADR-0009
`/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/adr/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

This is **not a hard blocker**, but it should shape the design now.

Why:

- future guest-rootfs or image-backed lanes must not alias host-native lanes
- backend/image identity needs to be part of world allocation identity early enough to avoid repainting the model later

## Key constraints from current architecture

A lane-first feature is not just a config addition. It requires structural changes across multiple surfaces.

The architecture must account for:

- current workspace-root-derived `project_dir` collapsing many requests into one world root
- generic reuse assumptions that are too singular for sibling worlds
- shared-owner semantics that intentionally allow one active world per orchestration session today
- member placement that intentionally lands in the authoritative shared world’s overlay/cgroup
- pending diff/workspace sync semantics that assume one current world
- coarse locking that may undercut true parallel world execution

## Things the design should explicitly not do

- **Do not** use `world_generation` to represent sibling worlds.
  - `world_generation` should remain a replacement/version concept within one owner-bound world slot.

- **Do not** begin with one world per participant as the default product model.
  - That may be useful in some cases, but it is too expensive and too specific as the core abstraction.

- **Do not** let lanes become mere labels with no effect on allocation identity.

- **Do not** ship the semantics as purely Linux implementation truth without contract work keeping pace.

- **Do not** try to solve selective freeze, full isolation, image matrices, rich versioning, and automatic routing all in one step.

## Hidden assumptions to validate

- Operators will prefer named lanes over raw world instances.
- One orchestration session can cleanly own multiple isolated lane realizations without making operator state confusing.
- Per-lane policy overrides can remain understandable and safe.
- Pending diff, sync, and read surfaces can become lane-targeted without breaking usability.
- The lane abstraction can later stretch to environment and platform/image use cases without forcing a second abstraction.
- Backend/image identity can join lane allocation identity without exploding the compatibility surface.

## What could kill this idea

- If the current reuse and binding model is too singular around `orchestration_session_id`
- If pending diff/workspace sync semantics are too deeply one-world-shaped to generalize cleanly
- If lane policy composition becomes opaque or too magical
- If operators cannot tell which lane owns which writes, diffs, or runtime state
- If “concurrent lanes” are marketed before lock granularity and backend concurrency support actually justify the claim

## Staged roadmap

### Phase 0: Concept and contract freeze

- define lane as the operator primitive
- define the relationship between lane and runtime world identity
- define the no-auto-routing rule for v1
- define how orchestration authority and operator targeting coexist
- align backend contract expectations with ADR-0010

### Phase 1: True v1 multi-agent isolated lanes

- session-owned named lanes
- per-lane isolated writable overlay state
- lane identity in allocation, status, trace, and orchestration surfaces
- explicit lane targeting
- lane-aware pending diff and sync targeting
- basic per-lane policy overrides

### Phase 2: Shared-base and hybrid lane behavior

- shared-base selective freeze
- read-only or review lanes
- optional shared overlay groups where warranted
- better operator UX around lane classes

### Phase 3: Image and environment realization

- rootfs/image/distro/backend-pinned lanes
- environment matrix workflows
- stronger backend-aware compatibility and capability surfaces

### Phase 4: Richer merge and versioning

- explicit merge-back workflows
- lane-aware git integration
- promotion/comparison flows
- more advanced sync and reconciliation semantics

## Open questions

- Should the runtime primitive be named `lane_id`, `world_slot_id`, or something that cleanly preserves both operator and backend concerns?
- Should some future lanes share writable overlays intentionally, or should sharing always be an explicit advanced mode?
- What is the exact precedence and restriction model for lane policy overrides versus workspace/global policy?
- How much of the lane namespace is session-owned versus workspace-defined reusable config?
- What minimum status/trace/operator surfaces are required so lane ownership of writes is obvious?
- What concurrency guarantees are required before claiming “concurrent” beyond merely “simultaneously live”?

## Recommended one-sentence summary

Substrate should adopt a **lane-first execution model** where named lanes are the operator-facing abstraction, worlds are the runtime realization of those lanes, and true v1 focuses on one orchestration session safely spawning multiple isolated writable agent lanes while leaving room for future environment and image-backed lane modes.

## References

- `/home/spenser/__Active_Code/substrate/docs/WORLD.md`
- `/home/spenser/__Active_Code/substrate/docs/internals/config/world_root_and_caging.md`
- `/home/spenser/__Active_Code/substrate/docs/internals/world/workspace_sync_filesystem_model.md`
- `/home/spenser/__Active_Code/substrate/docs/adr/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
- `/home/spenser/__Active_Code/substrate/docs/adr/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`
- `/home/spenser/__Active_Code/substrate/crates/world/src/session.rs`
- `/home/spenser/__Active_Code/substrate/crates/world/src/lib.rs`
- `/home/spenser/__Active_Code/substrate/crates/world-service/src/service.rs`
- `/home/spenser/__Active_Code/substrate/crates/world-service/tests/member_runtime_world_placement_v1.rs`
- `/home/spenser/__Active_Code/substrate/.codex/handoffs/2026-06-10-120158-multi-world-workspace-refinement.md`
