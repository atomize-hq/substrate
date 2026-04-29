# SOW: Shared-World Ownership Explicitness, Linux-First

## Objective

Make shared-world ownership explicit for agent-hub world-scoped execution on Linux by binding exactly one active `world_id` to exactly one `orchestration_session_id`, with monotonic `world_generation` tracking starting at `0` and incrementing only on hub-driven world replacement.

This slice is a prerequisite for the ADR-0044 / successor agent-hub contract already reflected in:

- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md`

The implementation target is Linux first. macOS and Windows must accept the additive model surface introduced here, but they are explicitly constrained from inventing independent shared-world ownership behavior during this slice.

## Scope

In scope:

- establish an explicit shared-world binding contract for Linux world-scoped member sessions
- persist the binding so world reuse is keyed by `orchestration_session_id`, not only `WorldSpec` compatibility
- track `world_generation` as authoritative runtime state
- thread the binding through shell runtime state, world backend state, and world-agent request handling
- make status/telemetry surfaces capable of reporting the bound `world_id` and `world_generation`

Out of scope:

- redesigning the full agent-hub control plane
- implementing all drift-restart triggers end-to-end for every platform
- changing host-scoped orchestrator semantics
- broad reuse-policy changes for non-agent shell flows that do not opt into explicit shared-world ownership

## Current State and Blockers

### 1. Linux world reuse is spec-compatible, not owner-compatible

Current Linux reuse is driven by `WorldSpec` compatibility and on-disk `session.json` discovery:

- `crates/world/src/lib.rs`
- `crates/world/src/session.rs`

`LinuxLocalBackend::ensure_session()` currently reuses a world when `project_dir`, network flags, and allowed domains match. `SessionWorldMetadata` does not record `orchestration_session_id` or `world_generation`. That means the current backend can reuse the same world across unrelated orchestrations if they share the same `WorldSpec`.

This is the primary blocker. The agent-hub contract requires one shared world per orchestration session, not one shared world per compatible spec.

### 2. Shell runtime manifests already expose world fields, but they are not authoritative yet

The shell-side live-session manifest model already carries optional world identity:

- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`

`AgentRuntimeSessionHandle` includes `world_id` and `world_generation`, and `substrate agent status` already expects world-scoped rows to have both:

- `crates/shell/src/execution/agents_cmd.rs`

However, there is no authoritative store that binds those fields to a Linux world allocation. The manifest can report these fields, but the backend currently has no matching ownership record to prove them.

### 3. Current orchestration identity is too incidental for shared-world ownership

`crates/shell/src/execution/agent_events.rs` currently mints a process-global `orchestration_session_id` via `OnceLock<String>`. That is acceptable for lightweight event correlation, but it is not sufficient as the ownership anchor for a reusable shared world that must survive multiple commands and explicit restart/generation transitions.

The shared-world binding must be driven by an explicit runtime/session owner, not by opportunistic event emission.

### 4. World-agent request paths do not carry explicit shared-world ownership intent

The relevant execution paths:

- `crates/agent-api-types/src/lib.rs`
- `crates/world-agent/src/service.rs`
- `crates/world-agent/src/pty.rs`
- `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`

Today, `ExecuteRequest` and the persistent-session `start_session` message carry policy, cwd, env, and world network data, but not an explicit shared-world owner binding. The world-agent therefore cannot distinguish:

- generic world reuse for ordinary shell execution
- explicit agent-hub shared-world reuse for one orchestration session

### 5. Placeholder world attribution exists in event paths

`WorldAgentService` currently emits some event data with placeholder correlation behavior, including using `span_id` as `orchestration_session_id` in one execution-stream path:

- `crates/world-agent/src/service.rs`

That is not acceptable once shared-world ownership is explicit. The shared-world binding must be sourced from the orchestrator/session model, not reconstructed from span-level execution identifiers.

### 6. Cross-platform bootstrap code can allocate worlds without agent-hub ownership semantics

Platform bootstrapping currently calls `ensure_session()` for generic world readiness:

- `crates/shell/src/execution/routing/world.rs`
- `crates/shell/src/execution/platform_world/mod.rs`
- `crates/shell/src/execution/platform_world/windows.rs`

That startup path must not accidentally become the source of truth for agent-hub shared-world ownership. This slice must keep explicit shared-world ownership opt-in and scoped to world-scoped member execution.

## Target Outcome

After this slice:

- Linux world-scoped member sessions have an explicit shared-world binding keyed by `orchestration_session_id`.
- Exactly one active `world_id` is bound to one `orchestration_session_id` at a time.
- `world_generation` is persisted and starts at `0` for first allocation.
- Reuse on Linux for agent-hub shared worlds is by exact owner match plus compatible world inputs, not by spec compatibility alone.
- The shell live-manifest and status surfaces read from data that the backend can prove.
- macOS and Windows accept additive schema/plumbing changes but do not introduce independent shared-world ownership logic in this slice.

## Proposed Design

### Authoritative Binding Model

Introduce a Linux-first shared-world binding record owned by the shell runtime and mirrored into Linux world metadata.

Authoritative shell-side binding path:

- `~/.substrate/run/agent-hub/world-bindings/<orchestration_session_id>.json`
- implementation owner: new helper under `crates/shell/src/execution/agent_runtime/`

Linux world-side mirrored metadata:

- `/tmp/substrate-worlds/<world_id>/session.json`
- implementation owner: `crates/world/src/session.rs`

Required binding fields:

- `schema_version`
- `orchestration_session_id`
- `current_world_id`
- `world_generation`
- `workspace_root`
- `policy_snapshot_hash`
- `world_fs_mode`
- `isolate_network`
- `allowed_domains_hash`
- `state`
- `created_at`
- `last_bound_at`
- `last_restart_reason`

Rules:

- exactly one authoritative binding file exists per live `orchestration_session_id`
- exactly one active `current_world_id` exists in that binding at a time
- `world_generation` is monotonic per `orchestration_session_id`
- Linux world metadata must mirror the owning `orchestration_session_id` and `world_generation`
- a world with missing or conflicting ownership metadata is not reusable for agent-hub shared-world flows

## Backend and Model Changes

### 1. `world-api` contract becomes ownership-aware

Primary files:

- `crates/world-api/src/lib.rs`
- `crates/agent-api-types/src/lib.rs`

Add additive fields so explicit shared-world ownership can be requested without overloading generic world reuse:

- `WorldSpec` gains an optional shared-world ownership request, for example:
  - `shared_owner.orchestration_session_id`
  - `shared_owner.expected_generation` optional for restart/validation paths
  - `shared_owner.mode = shared_orchestration_session`
- `WorldHandle` gains additive ownership metadata, at minimum:
  - `generation: Option<u64>`
  - `orchestration_session_id: Option<String>`
- `ExecuteRequest` gains additive shared-world ownership fields for non-PTY paths
- persistent-session `start_session` gains the same additive ownership fields for PTY/session paths in `crates/world-agent/src/pty.rs`

Constraint:

- the new fields are optional so legacy/non-agent callers remain unchanged
- explicit shared-world behavior only activates when ownership fields are present

### 2. Linux backend distinguishes generic reuse from shared-world reuse

Primary files:

- `crates/world/src/lib.rs`
- `crates/world/src/session.rs`

Behavior change:

- keep existing spec-compatible reuse for generic non-agent flows with no explicit owner
- add a separate exact-owner reuse path when `shared_owner.orchestration_session_id` is present

Required Linux semantics:

- first look up an existing world by bound `orchestration_session_id`
- verify ownership metadata, generation, and world-input compatibility
- reuse only when both ownership and compatible world inputs match
- create a new world when no valid binding exists
- never reuse a world already owned by a different `orchestration_session_id`

### 3. Session metadata becomes ownership-bearing

Primary file:

- `crates/world/src/session.rs`

Extend `SessionWorldMetadata` with:

- `owner_mode`
- `orchestration_session_id`
- `world_generation`
- `binding_state`

Required changes:

- `metadata_matches_spec()` must also validate owner identity when shared-world ownership is requested
- `recover_compatible_from_root()` must reject stale or cross-owned worlds for shared-world flows
- `persist_metadata()` must atomically write the new ownership fields

## Shell Runtime and Persistence Changes

### 1. Add a dedicated shared-world binding store

Primary owner directory:

- `crates/shell/src/execution/agent_runtime/`

Suggested new surface:

- `shared_world_store.rs`

Responsibilities:

- allocate the first binding for an `orchestration_session_id`
- read/update the authoritative `current_world_id`
- persist `world_generation`
- invalidate old bindings during restart
- expose a narrow API used by agent runtime/session code

This store should use the same atomic-write posture already used in:

- `crates/shell/src/execution/agent_runtime/state_store.rs`

### 2. Make live session manifests authoritative consumers of the binding

Primary files:

- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agents_cmd.rs`

Required behavior:

- world-scoped manifests must populate `world_id` and `world_generation` from the shared-world binding path, not from ad hoc environment data
- restart replacement handles must set:
  - new `session_handle_id`
  - same `orchestration_session_id`
  - `resumed_from_session_handle_id = <old handle>`
  - new `world_id`
  - incremented `world_generation`
- host-scoped manifests continue to omit both fields

## World-Agent and Routing Changes

Primary files:

- `crates/world-agent/src/service.rs`
- `crates/world-agent/src/pty.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`

Required changes:

- thread explicit shared-world ownership from shell dispatch into world-agent requests
- build `WorldSpec` for Linux shared-world flows with ownership fields attached
- return the bound `world_id` and generation-aware handle data to the shell runtime
- stop using placeholder execution identifiers as substitutes for `orchestration_session_id`

Constraint:

- do not widen unrelated shell command execution paths
- only world-scoped member/agent-hub execution should use the explicit shared-world binding path in this slice

## Telemetry and Status Changes

Primary files:

- `crates/common/src/agent_events.rs`
- `crates/shell/src/execution/agent_events.rs`
- `crates/shell/src/execution/agents_cmd.rs`
- `docs/TRACE.md`

Required outcome:

- world-scoped pure-agent records can publish top-level `world_id` and `world_generation` from explicit ownership state
- nested gateway-backed records continue omitting those fields
- status output remains fail-closed if a world-scoped session lacks both fields

This slice does not need to finish every future drift alert, but it must establish the data path that later `world_restarted` and `world_restart_required` alerts will consume.

## Linux-First Platform Constraints

### Linux

Linux is the only platform that changes world ownership behavior in this slice.

Implementation authority:

- `crates/world`
- `crates/world-agent`
- shell-side agent runtime persistence

### macOS

Primary files likely touched later for additive compatibility only:

- `crates/world-mac-lima/src/lib.rs`
- `crates/shell/src/execution/platform_world/mod.rs`
- `crates/shell/src/execution/routing/world.rs`

Constraints for this slice:

- accept additive request/model fields needed for shared-world ownership
- preserve existing Lima-backed world realization
- do not implement a separate shared-world ownership store in the macOS host path
- do not claim platform-complete shared-world restart/generation behavior beyond the Linux-backed contract

### Windows

Primary files likely touched later for additive compatibility only:

- `crates/shell/src/execution/platform_world/windows.rs`
- `crates/world-windows-wsl/`

Constraints for this slice:

- accept additive request/model fields needed for shared-world ownership
- preserve existing WSL-backed world realization
- do not implement an independent Windows-native shared-world ownership algorithm
- fail closed at the agent-hub/shared-world boundary if explicit Linux-first ownership behavior is required but not yet supported

## Sequencing

1. Freeze the ownership contract and file layout.
   - finalize shared-world binding schema
   - finalize Linux `session.json` additions
   - finalize additive `WorldSpec` / request-shape fields

2. Land model changes.
   - update `crates/world-api/src/lib.rs`
   - update `crates/agent-api-types/src/lib.rs`
   - update shell/runtime structs that hold `world_id` and `world_generation`

3. Land Linux persistence and reuse behavior.
   - implement binding store in shell runtime
   - extend `SessionWorldMetadata`
   - teach `LinuxLocalBackend` to resolve by explicit owner binding

4. Thread request plumbing.
   - shell dispatch passes explicit shared-world owner data
   - world-agent builds ownership-aware `WorldSpec`
   - shell receives authoritative `world_id` and generation

5. Publish runtime state.
   - persist live manifest fields from authoritative binding data
   - remove placeholder correlation where it conflicts with explicit ownership

6. Add validation coverage and doc updates.
   - targeted Linux tests first
   - additive status/trace checks
   - update `docs/WORLD.md` and `docs/TRACE.md` after implementation lands

## Acceptance Criteria

- Linux world-scoped member allocation with a given `orchestration_session_id` creates one shared world with `world_generation = 0`.
- A second world-scoped member allocation for the same `orchestration_session_id` reuses the same `world_id` when world inputs are unchanged.
- A different `orchestration_session_id` never reuses that world, even if `WorldSpec` inputs are otherwise identical.
- Linux persisted world metadata includes `orchestration_session_id` and `world_generation`.
- Shell authoritative runtime state includes one binding file per orchestration session and one active `current_world_id`.
- World-scoped `AgentRuntimeSessionManifest` rows publish both `world_id` and `world_generation`.
- Host-scoped rows omit both fields.
- `substrate agent status` continues to fail closed if the selected newest world-scoped session lacks either field.
- macOS and Windows compile with the additive model changes and do not invent independent shared-world ownership semantics in this slice.

## Validation and Testing Suggestions

Targeted unit/integration coverage:

- `crates/world/src/session.rs`
  - persist and recover owned metadata
  - reject reuse when `orchestration_session_id` mismatches
  - reject reuse when generation metadata is missing for explicit shared-world flows
- `crates/world/src/lib.rs`
  - exact-owner lookup wins over generic spec-compatible reuse for explicit shared-world requests
  - generic callers without ownership fields preserve current behavior
- `crates/world-agent/src/pty.rs`
  - persistent-session start accepts explicit shared-world binding fields
  - ready frame returns the correct bound `world_id`
- `crates/world-agent/src/service.rs`
  - non-PTY execute path uses ownership-aware `WorldSpec`
  - no placeholder `span_id -> orchestration_session_id` behavior remains on shared-world paths
- `crates/shell/src/execution/agent_runtime/*`
  - binding-store atomic write/read behavior
  - live-manifest projection of `world_id` and `world_generation`
  - restart replacement handle semantics
- `crates/shell/src/execution/agents_cmd.rs`
  - world-scoped status rows render both fields
  - malformed selected rows still fail closed

Suggested command validation once implemented:

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace -- --nocapture`
- targeted Linux-first runs for `crates/world`, `crates/world-agent`, and shell agent-runtime/status tests

Manual validation suggestions:

- start one orchestration session with a world-scoped member, record `world_id` and `world_generation`
- start a second world-scoped member in the same orchestration, verify the same `world_id`
- start a different orchestration session with identical workspace/policy inputs, verify a different `world_id`
- force a hub-driven restart path and verify generation increments exactly by `1`
- verify `substrate agent status --json` shows world fields only for world-scoped rows

## Risks

- `WorldHandle` shape changes will fan out across Linux, macOS, and Windows call sites.
- Generic shell startup currently allocates worlds in several places; if the explicit binding boundary is not kept narrow, unrelated execution paths may change behavior.
- There is a risk of dual truth if shell binding files and Linux `session.json` can diverge. The implementation should define one authority and one mirrored copy with explicit conflict handling.
- Existing persisted `session.json` files under `/tmp/substrate-worlds` may lack the new ownership fields. Shared-world reuse must fail closed for explicit ownership flows rather than silently adopting legacy worlds.
- The current global `orchestration_session_id` helper in `crates/shell/src/execution/agent_events.rs` may remain useful for simple event flows, but it must not remain the authority for shared-world ownership.

## Open Questions

1. Should the authoritative binding store live only under `~/.substrate/run/agent-hub/world-bindings/`, or should Linux `session.json` be treated as co-authoritative for recovery after shell restart?
2. Should `world_generation` live on `WorldHandle` directly, or should the shell read it from a separate binding snapshot after `ensure_session()` returns?
3. What is the exact restart API boundary for the later drift slice: a new backend method, or an ownership-aware `ensure_session()` with an explicit replacement flag?
4. How should shell startup worlds created by `initialize_world()` be prevented from colliding with later agent-hub shared-world bindings when both target the same workspace?
5. For non-Linux platforms in this slice, should explicit shared-world requests fail at shell routing before backend invocation, or should backends return a uniform unsupported/shared-world-unavailable error?

## Recommended Decision Posture for This Slice

- Treat shell-side binding files as the primary authority for orchestration ownership.
- Treat Linux `session.json` as mirrored proof required for backend reuse and crash recovery.
- Keep explicit shared-world ownership opt-in and limited to world-scoped agent-hub execution.
- Preserve legacy spec-compatible reuse for non-agent callers until a later cleanup slice intentionally retires it.
