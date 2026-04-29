# PLAN-03: Shared-World Ownership Contract, Linux-First

Source file: [03-shared-world-ownership-linux-first.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/03-shared-world-ownership-linux-first.md)  
Branch: `feat/shared-world-ownership-contract`  
Plan type: backend-only, no UI scope  
Review posture: `/autoplan`-style scope tightening with `/plan-eng-review` structure and rigor  
Status: execution-ready after slice-boundary correction

## What This Plan Does

`PLAN-01` established real orchestration-session identity. `PLAN-02` established a participant-capable runtime model. The next failure point is not "more runtime state." It is that Linux shared worlds still have no authoritative owner contract.

Today the repo already exposes world identity in a few places, but none of them are the right authority:

- Linux world reuse still keys off compatible `WorldSpec` inputs in [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:39) and [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:90).
- PTY startup returns only `world_id` from [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:993), not an owner-proof snapshot.
- Non-PTY execution still builds a plain `WorldSpec` and calls `ensure_session()` in [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:470).
- The REPL already increments `world_generation` locally in [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2723), which means shell-visible generation exists before the backend can prove it.

That is the actual problem.

This plan rewrites slice `03` to do one job only:

1. establish the authoritative shared-world owner tuple in `world-api`,
2. make Linux reuse owner-aware instead of spec-only,
3. thread that owner tuple through world-agent PTY and non-PTY request/response contracts,
4. fail closed on non-Linux shared-world requests, and
5. stop synthetic world ownership attribution such as `span_id -> orchestration_session_id`.

This plan explicitly does **not** take ownership of runtime-state projection or restart invalidation. Those are already the point of slices `04` and `05`:

- [04-thread-world-binding-into-runtime-state.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md)
- [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)

That scope cut matters. It prevents slice `03` from fighting the packet instead of enabling it.

## Scope Challenge

### Why this is the right third slice

After `PLAN-02`, the runtime can represent world-scoped participants honestly. What it still cannot do is prove that a Linux world belongs to exactly one orchestration session.

That ownership proof has to exist before runtime-state projection is meaningful. Otherwise `PLAN-04` would just persist a prettier lie, and `PLAN-05` would invalidate generations that the backend never actually committed.

The practical bottleneck is simple:

- generic Linux world reuse is already live,
- world-scoped member semantics need stricter ownership than generic reuse,
- the world-agent transport does not carry that distinction today.

Fix that seam first. Then the later slices have a real source of truth to consume.

### What already exists

| Sub-problem | Existing code | Reuse or replace |
|---|---|---|
| Orchestration identity | [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:18) | Reuse exactly |
| Participant model with world fields | [session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs:11) | Reuse, but do not make authoritative in this slice |
| Linux world reuse machinery | [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:27) | Extend |
| Linux persisted `session.json` metadata | [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:13) | Extend |
| PTY persistent-session protocol | [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:163) and [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:367) | Extend |
| Non-PTY execute transport | [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:657) and [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:432) | Extend |
| REPL restart/generation signaling | [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2378) | Reuse later in `PLAN-05`, do not deepen here |
| Fail-closed status expectations for world-scoped rows | [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:541) | Reuse later in `PLAN-04` |
| Cross-platform world bootstrap | [routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:13), [platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:60), [platform_world/windows.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/windows.rs:52) | Keep generic, add fail-closed gate for explicit owner mode |

### Minimum change that achieves the goal

Do this, and only this:

- add an explicit shared-world owner mode to `world-api` and agent API request surfaces,
- return authoritative owner/generation metadata from Linux world allocation,
- persist that metadata in Linux `session.json`,
- make Linux owner-bound reuse a separate code path from generic compatible reuse,
- fail closed for explicit owner-bound requests on non-Linux backends,
- remove synthetic shared-world attribution in world-agent event paths.

Do not:

- create a shell-side authoritative world-binding store in this slice,
- project world binding into runtime manifests here,
- redefine participant replacement semantics here,
- redesign `substrate agent status` selection here,
- invent a new agent-hub daemon,
- add a cache to avoid scanning a small number of Linux world metadata files.

### Complexity check

This slice crosses more than 8 files. That is acceptable because the authority seam is already spread across:

- `crates/world-api`
- `crates/agent-api-types`
- `crates/world`
- `crates/world-agent`
- `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
- `crates/shell/src/execution/platform_world/*`
- `crates/shell/src/repl/async_repl.rs`
- tests and docs

The guardrail is strict:

- no new daemon,
- no new generic registry abstraction,
- no runtime-manifest write path here,
- no speculative non-Linux ownership algorithm,
- no full restart invalidation policy here.

That keeps the diff large enough to solve the real problem, but still boring.

### Search-before-building posture

`[Layer 1]` wins here.

The repo already has:

- a portable `world-api` contract,
- Linux persisted world metadata,
- PTY and non-PTY agent transport seams,
- participant/runtime structs that can eventually consume world identity,
- explicit later slices for runtime-state projection and restart invalidation.

The right move is not to create a new shell authority and mirror it down. That would be a second database disguised as JSON files. Software loves doing that. The pager loves it less.

The clean path is:

1. make Linux world metadata and backend allocation authoritative for owner-bound reuse,
2. make transport contracts echo that authority,
3. let later slices project it upward.

### Hard non-goals

- runtime-manifest binding persistence from [04-thread-world-binding-into-runtime-state.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md)
- participant invalidation and generation cutover from [05-restart-invalidation-semantics.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md)
- grouped session-centric state-store reshaping from slice `06`
- non-Linux shared-world parity beyond model acceptance plus fail-closed rejection
- status/toolbox schema redesign
- trace schema redesign beyond removing invalid placeholder attribution

## Architecture Contract

### No-ambiguity rules

These are hard rules:

1. Slice `03` owns the **world ownership contract**, not runtime-state projection and not restart invalidation.
2. Linux world metadata and backend allocation are the authoritative source of truth for owner-bound shared worlds in this slice.
3. Explicit shared-world requests must be structurally distinct from generic compatible-reuse requests. Missing owner context is an error, not a fallback.
4. Generic shell/world bootstrap behavior remains generic. It must not silently become agent-hub ownership authority.
5. `world_generation` starts at `0` on first owner-bound allocation and changes only when the backend accepts an explicit replacement action.
6. PTY and non-PTY paths must both carry the same owner tuple and receive the same authoritative binding snapshot back.
7. Legacy Linux `session.json` without owner metadata may remain reusable for generic callers, but must be rejected for explicit owner-bound requests.
8. Non-Linux backends may accept additive schema changes, but explicit shared-world owner mode must fail closed before generic reuse can happen.
9. No world-agent event path may synthesize `orchestration_session_id` from `span_id` for shared-world flows.
10. Runtime manifests, selected status rows, and participant invalidation remain follow-on work for `PLAN-04` and `PLAN-05`.

### Target owner contract

Add one explicit owner-bound mode to `world-api`. Do not overload "plain `reuse_session=true` plus maybe some optional fields."

Use this shape:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldReuseMode {
    GenericCompatible,
    SharedOrchestration(SharedWorldOwnerSpec),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SharedWorldOwnerSpec {
    pub orchestration_session_id: String,
    pub action: SharedWorldOwnerAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SharedWorldOwnerAction {
    AttachOrCreate,
    ReplaceExpectedGeneration {
        expected_generation: u64,
        reason: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SharedWorldBindingSnapshot {
    pub orchestration_session_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub binding_state: SharedWorldBindingState,
}
```

Then:

- `WorldSpec` gains `reuse_mode: WorldReuseMode`
- `WorldHandle` gains `shared_binding: Option<SharedWorldBindingSnapshot>`
- non-PTY `ExecuteRequest` gains `shared_world: Option<SharedWorldOwnerSpec>`
- PTY `start_session` gains the same `shared_world` payload
- PTY `ready` and non-PTY `ExecuteResponse` echo the authoritative `SharedWorldBindingSnapshot`

This is more explicit than the source SOW, and that is intentional. Optional loose fields are how silent downgrade bugs sneak in.

### Linux metadata becomes the authority seam

Primary file:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:13)

Extend `SessionWorldMetadata` with a minimal ownership-bearing shape:

```rust
struct SessionWorldMetadata {
    world_id: String,
    project_dir: PathBuf,
    isolate_network: bool,
    always_isolate: bool,
    allowed_domains: Vec<String>,
    cgroup_path: PathBuf,
    started_at_unix_millis: u64,
    owner_mode: SessionWorldOwnerMode,
    orchestration_session_id: Option<String>,
    world_generation: Option<u64>,
    binding_state: Option<SharedWorldBindingState>,
    policy_snapshot_hash: Option<String>,
    world_fs_mode: Option<WorldFsMode>,
    last_restart_reason: Option<String>,
}
```

Rules:

- generic worlds persist `owner_mode=generic` and omit owner fields
- explicit shared worlds persist `owner_mode=shared_orchestration`, `orchestration_session_id`, `world_generation`, and `binding_state`
- `binding_state` is one of:
  - `active`
  - `replacing`
  - `replaced`
  - `abandoned`
- shared-owner reuse is allowed only from `binding_state=active`
- incomplete or contradictory owner metadata is never reusable for shared-owner flows

### Linux reuse path splits cleanly

Primary file:

- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:39)

The backend must stop treating shared-world ownership as "generic reuse, but with extra checks later."

Required behavior:

1. `WorldReuseMode::GenericCompatible`
   - preserve existing compatibility behavior
   - legacy `session.json` remains eligible

2. `WorldReuseMode::SharedOrchestration(AttachOrCreate)`
   - scan persisted metadata for `owner_mode=shared_orchestration`
   - require exact `orchestration_session_id` match
   - require compatible world inputs
   - require `binding_state=active`
   - reuse matching world if present
   - otherwise create a new world with `world_generation=0`
   - never reuse a world owned by another orchestration session

3. `WorldReuseMode::SharedOrchestration(ReplaceExpectedGeneration { expected_generation, reason })`
   - resolve the currently active world for that owner
   - require exact generation match
   - create a fresh world with generation `expected_generation + 1`
   - mark prior world `binding_state=replaced`
   - return the new authoritative binding snapshot

4. Any shared-owner request with ownerless legacy metadata
   - fail closed for that candidate
   - do not silently adopt it

This split is the whole game.

### Minimal stale/abandoned world rule

The source SOW left crash/abandonment too vague. This plan tightens it without stealing slice `05`.

Required minimal rule in this slice:

- if a shared-owner world root exists but its metadata is incomplete, unreadable, or references missing critical directories, it is not reusable
- such a world may be marked `abandoned` by Linux recovery logic
- `abandoned` worlds are never reused for shared-owner requests
- cleanup automation can remain manual or follow-on work, but reuse semantics must be deterministic now

This prevents the worst case: shell crash leaves half-written owner metadata, and the next attach accidentally reuses it because the spec happened to match.

### Transport contract becomes proof-bearing

Primary files:

- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:657)
- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:163)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:432)
- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:367)

Required outcome:

- shared-world owner context is carried into both PTY and non-PTY allocation calls
- Linux returns authoritative owner proof back on both paths
- the shell can validate that proof immediately

That means:

- PTY `Ready` must stop being only `{ world_id, cwd, protocol_version }`
- non-PTY `ExecuteResponse` must stop returning only a brand-new span id with no owner proof

Use this PTY `Ready` shape:

```rust
Ready {
    session_nonce: String,
    world_id: String,
    cwd: PathBuf,
    protocol_version: u32,
    shared_world: Option<SharedWorldBindingSnapshot>,
}
```

Add this to non-PTY `ExecuteResponse`:

```rust
pub shared_world: Option<SharedWorldBindingSnapshot>
```

### Shell-side scope in this slice

Shell changes are intentionally narrow:

- resolve the active `orchestration_session_id` when building explicit world-scoped member requests
- populate `shared_world` request payloads for shared-owner flows only
- validate returned binding snapshots from PTY/non-PTY responses
- do not persist runtime manifests here
- do not treat REPL-local `world_generation` arithmetic as authoritative here

In practice:

- [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2677) may still track a local generation for in-memory session continuity, but that value must be replaced by the backend-returned generation as soon as the contract exists
- runtime-state persistence remains the responsibility of `PLAN-04`

### Cross-platform boundary

Primary files:

- [crates/shell/src/execution/routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:13)
- [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:60)
- [crates/shell/src/execution/platform_world/windows.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/windows.rs:52)
- `crates/world-mac-lima/src/*`
- `crates/world-windows-wsl/*`

Required rule:

- explicit shared-owner mode on non-Linux platforms returns one uniform unsupported error before any generic compatible reuse can occur

That means:

- macOS and Windows may deserialize the new fields
- compile compatibility is required
- behavior compatibility is **not** the same as shared-owner support

If a shared-owner request reaches a generic `ensure_session()` path on non-Linux, the slice is wrong.

## Architecture Diagrams

### Linux owner-bound allocation flow

```text
shell world-scoped member request
        │
        │ resolve orchestration_session_id + owner action
        ▼
world-agent PTY/non-PTY transport
        │
        │ shared_world = { owner, action }
        ▼
world-api::WorldSpec { reuse_mode = SharedOrchestration(...) }
        │
        ▼
LinuxLocalBackend::ensure_session()
        │
        ├── scan /tmp/substrate-worlds/*/session.json
        │
        ├── if exact owner + compatible inputs + active binding
        │       └── reuse existing world
        │
        ├── if replace_expected_generation + generation matches
        │       └── create new world, old => replaced
        │
        └── else
                └── create new world generation 0
        ▼
authoritative SharedWorldBindingSnapshot
        │
        ├── PTY ready frame echo
        └── non-PTY execute response echo
```

### Generic vs shared-owner reuse boundary

```text
incoming world request
        │
        ├── reuse_mode = GenericCompatible
        │       └── old compatibility path, legacy metadata allowed
        │
        └── reuse_mode = SharedOrchestration(owner)
                │
                ├── owner metadata missing on candidate
                │       └── reject candidate
                │
                ├── owner matches + inputs compatible + active
                │       └── reuse
                │
                ├── owner differs
                │       └── never reuse
                │
                ├── replace generation mismatch
                │       └── fail closed
                │
                └── no valid active match
                        └── create new owned world
```

## Implementation Plan

### Ordered execution checklist

1. Narrow slice `03` to owner contract only.
2. Add `WorldReuseMode`, `SharedWorldOwnerSpec`, and `SharedWorldBindingSnapshot`.
3. Extend Linux `session.json` metadata with owner-bearing fields.
4. Split generic reuse from shared-owner reuse in `LinuxLocalBackend`.
5. Extend PTY and non-PTY request/response contracts to carry owner proof.
6. Add fail-closed non-Linux capability gates.
7. Remove placeholder `span_id -> orchestration_session_id` behavior on shared-owner paths.
8. Add Linux-first tests, then docs.

### Execution sequencing and merge gates

1. Land the contract types first.
   - No backend behavior change yet.
   - Merge gate: serde round-trip and compile green.

2. Land Linux metadata plus owner-aware reuse.
   - This is the true behavior change.
   - Merge gate: Linux unit/integration coverage for attach/create/replace/reject.

3. Land PTY and non-PTY transport plumbing.
   - The shell must be able to request and validate the new contract.
   - Merge gate: world-agent protocol tests green.

4. Land non-Linux fail-closed routing gates.
   - Prevent accidental shared-owner downgrade.
   - Merge gate: platform-specific unsupported-path tests green.

5. Land docs and cleanup.
   - Update packet docs to reflect the corrected slice boundary.

### Workstream 1: Contract types and slice-boundary correction

Primary files:

- [llm-last-mile/PLAN-03.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:14)
- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:650)

Tasks:

- add explicit owner-bound reuse mode
- add binding snapshot response type
- extend API payload structs
- update doc comments so generic vs shared-owner behavior is obvious

### Workstream 2: Linux metadata and exact-owner reuse

Primary files:

- [crates/world/src/session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/session.rs:13)
- [crates/world/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world/src/lib.rs:27)

Tasks:

- extend `SessionWorldMetadata`
- implement owner-aware metadata matching and rejection rules
- implement shared-owner attach/create logic
- implement replace-expected-generation logic
- mark old world metadata `replaced`

### Workstream 3: PTY and non-PTY transport plumbing

Primary files:

- [crates/world-agent/src/pty.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/pty.rs:163)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:432)
- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:367)

Tasks:

- parse `shared_world` on PTY start
- build owner-aware `WorldSpec`
- echo authoritative binding snapshot in PTY `Ready`
- echo authoritative binding snapshot in non-PTY `ExecuteResponse`
- validate missing/malformed snapshots fail closed

### Workstream 4: Shell routing gates and REPL alignment

Primary files:

- [crates/shell/src/execution/routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:13)
- [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:60)
- [crates/shell/src/execution/platform_world/windows.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/windows.rs:52)
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2677)

Tasks:

- resolve owner context for shared-world member flows
- reject non-Linux shared-owner mode before backend allocation
- consume backend-returned generation instead of trusting local increment-only state where possible
- keep runtime-manifest persistence out of this slice

### Workstream 5: Tests and docs

Primary files:

- `crates/world/tests/*`
- `crates/world-agent/tests/*`
- [crates/shell/tests/repl_world_first_routing_v1.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

Tasks:

- add missing Linux-first behavior tests
- document shared-owner mode and non-Linux rejection
- document that runtime-state projection is still `PLAN-04`

## Architecture Review

This section is the final engineering lock-in for slice `03`. The prior review findings are now resolved into hard implementation requirements, not open recommendations.

### Locked architecture decisions

1. **Slice boundary is fixed.**
   - `PLAN-03` owns only the shared-world owner contract, Linux reuse semantics, transport proof, and non-Linux fail-closed rejection.
   - `PLAN-04` owns runtime-manifest projection of `world_id` and `world_generation`.
   - `PLAN-05` owns invalidation and replacement semantics after generation changes.

2. **Linux backend metadata is the only authority in this slice.**
   - The authoritative seam is persisted Linux world metadata plus `crates/world` allocation behavior.
   - This plan does not introduce a shell-owned binding file or second authority store under `~/.substrate/run/agent-hub/`.

3. **Shared-owner reuse is structurally distinct from generic reuse.**
   - Generic compatible reuse remains the current path.
   - Explicit shared-world reuse must go through `WorldReuseMode::SharedOrchestration(...)`.
   - Missing owner context is a hard error, never an implicit downgrade to generic reuse.

4. **Backend-returned generation is the committed truth.**
   - The REPL may stage local restart intent, but it does not commit `world_generation`.
   - PTY and non-PTY responses must echo the authoritative `SharedWorldBindingSnapshot`, and that echoed generation is the only value later slices may persist.

5. **Linux-first is enforced behaviorally, not cosmetically.**
   - macOS and Windows may deserialize additive fields.
   - They must reject explicit shared-owner requests before any generic `ensure_session()`-style fallback can occur.

### Architecture acceptance gates

The plan is only correctly implemented if all five gates pass:

1. **Contract gate**
   - `world-api`, `agent-api-types`, PTY ready frames, and non-PTY execute responses all use the same owner-binding vocabulary.

2. **Linux authority gate**
   - `crates/world` can decide attach/create/replace using persisted owner-bearing metadata alone, without consulting shell-local authority.

3. **Transport proof gate**
   - Both PTY and non-PTY callers receive authoritative owner proof back from the backend, not just `world_id`.

4. **Fail-closed gate**
   - Any malformed, missing, contradictory, or cross-owned metadata path rejects or allocates fresh. It never silently reuses.

5. **Platform boundary gate**
   - Non-Linux explicit shared-owner requests reject uniformly and early.

## Code Quality Review

This slice stays boring on purpose. The minimum diff that closes the ownership hole is the right diff.

### Implementation guardrails

1. **One place owns reuse compatibility logic.**
   - Compatibility hashing, policy fingerprint comparison, and owner-match rules stay in `crates/world`.
   - Shell and world-agent code only build requests, pass them through, and validate returned snapshots.

2. **One snapshot type travels through every layer.**
   - `SharedWorldBindingSnapshot` is the reusable proof object.
   - `WorldHandle`, PTY `Ready`, and non-PTY `ExecuteResponse` all reuse it instead of inventing adjacent mini-schemas.

3. **No synthetic ownership attribution.**
   - `span_id` remains span correlation only.
   - Shared-world ownership fields are populated from explicit owner context or omitted. They are never inferred.

4. **No second persistence system.**
   - No new shell-owned registry, no side cache, no background daemon, no duplicate JSON authority file.
   - The implementation extends existing persistence seams instead of adding new ones.

5. **Provisional REPL state stays clearly provisional.**
   - Any local generation arithmetic in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2723) must be treated as pre-commit state only.
   - Backend echo is the boundary between "requested" and "committed."

### Minimal-diff rules

- Reuse existing serde surfaces instead of creating wrapper payloads for one slice.
- Reuse existing Linux metadata recovery paths instead of adding a new world lookup registry.
- Reuse existing platform-world routing seams for fail-closed rejection instead of creating a special shared-world launcher.
- Add comments only where the generic/shared-owner boundary would otherwise be easy to misread.

## Error & Rescue Registry

| Failure point | What goes wrong | Expected rescue / fail-closed behavior |
|---|---|---|
| shared-world request missing owner tuple | request enters generic reuse path | reject request before backend allocation |
| same spec, different owner | backend reuses another session’s world | allocate a fresh world; never cross-reuse |
| replace request generation mismatch | stale restart request increments the wrong chain | reject with explicit generation-conflict error |
| owner-bearing PTY request hits non-Linux | backend falls through to generic ensure-session | return uniform unsupported-shared-world-owner error |
| legacy ownerless `session.json` encountered for shared-owner flow | backend silently adopts a pre-owner world | treat candidate as non-reusable for shared-owner mode |
| partial metadata write after crash | future request reuses a half-bound world | mark candidate abandoned or ignore it; never reuse |
| PTY ready frame omits generation echo | shell invents generation anyway | close session fail closed and surface protocol error |
| world-agent event path uses `span_id` as owner id | trace shows false world lineage | omit owner attribution or use explicit owner context only |

## Test Review

100% new-path coverage is the goal. The current branch has partial generic world reuse coverage, but almost none of the owner-bound behavior this slice requires.

### Code path coverage

```text
CODE PATH COVERAGE
===========================
[+] crates/world/src/session.rs
    │
    ├── [★★  TESTED] Generic persisted metadata recovery / compatibility path exists today
    │                via current recover_compatible_from_root tests
    ├── [GAP]        Shared-owner metadata parse + persist round-trip
    ├── [GAP]        Shared-owner candidate rejected when owner mismatches
    ├── [GAP]        Legacy ownerless metadata rejected for shared-owner flows
    └── [GAP]        Replaced/abandoned metadata never reused

[+] crates/world/src/lib.rs
    │
    ├── [★★  TESTED] Generic cache miss repopulates backend cache from persisted metadata
    ├── [GAP]        Shared-owner attach_or_create bypasses generic compatibility lookup
    ├── [GAP]        Same owner + same inputs reuses same world
    ├── [GAP]        Different owner + same inputs allocates distinct world
    └── [GAP]        ReplaceExpectedGeneration increments generation and returns new binding

[+] crates/world-agent/src/pty.rs
    │
    ├── [GAP]        start_session accepts shared_world owner payload
    ├── [GAP]        Ready echoes shared binding snapshot
    └── [GAP]        missing/invalid shared binding snapshot fails protocol closed

[+] crates/world-agent/src/service.rs
    │
    ├── [GAP]        ExecuteRequest shared_world payload builds owner-aware WorldSpec
    ├── [GAP]        ExecuteResponse echoes shared binding snapshot
    └── [GAP]        streamed event path never aliases span_id into orchestration ownership

[+] crates/shell/src/execution/routing/dispatch/world_persistent_session.rs
    │
    ├── [GAP]        shell validates ready.shared_world for owner-bound sessions
    └── [GAP]        missing generation echo aborts session instead of trusting local state

[+] crates/shell/src/execution/platform_world/*
    │
    └── [GAP]        macOS/Windows explicit shared-owner mode rejects before generic reuse

─────────────────────────────────
COVERAGE: 2/15 paths tested (13%)
  Code paths: 2/15 (13%)
QUALITY:  ★★★: 0  ★★: 2  ★: 0
GAPS: 13 paths need tests
─────────────────────────────────
```

### User flow coverage

```text
USER FLOW COVERAGE
===========================
[+] Linux shared-world attach/create
    │
    ├── [GAP] [→E2E] First member for orchestration session creates generation 0 world
    ├── [GAP] [→E2E] Second member same orchestration reuses same world_id
    └── [GAP] [→E2E] Different orchestration with identical spec gets a different world_id

[+] Linux replacement flow
    │
    ├── [GAP] [→E2E] ReplaceExpectedGeneration succeeds and returns generation+1
    └── [GAP]        ReplaceExpectedGeneration with stale generation fails closed

[+] Crash / recovery edges
    │
    ├── [GAP]        Half-written owner metadata is ignored or marked abandoned
    └── [GAP]        Legacy ownerless world never adopted by shared-owner path

[+] Cross-platform boundary
    │
    ├── [GAP]        macOS explicit shared-owner request is rejected before generic allocation
    └── [GAP]        Windows explicit shared-owner request is rejected before generic allocation

[+] Transport proof
    │
    ├── [GAP]        PTY ready frame carries world_id + world_generation + owner echo
    └── [GAP]        Non-PTY execute response carries same owner proof

─────────────────────────────────
COVERAGE: 0/10 flows tested (0%)
  User flows: 0/10 (0%)
GAPS: 10 flows need tests (5 need E2E-style integration)
─────────────────────────────────
```

### Required test additions by file

#### `crates/world/src/session.rs`

Add unit coverage for:

- owned metadata round-trip with generation `0`
- reject shared-owner reuse when `orchestration_session_id` mismatches
- reject ownerless legacy metadata for shared-owner mode
- reject `binding_state=replaced` and `binding_state=abandoned`
- ignore malformed partial owner metadata after crash

#### `crates/world/src/lib.rs`

Add Linux backend coverage for:

- `GenericCompatible` still preserves current behavior
- `SharedOrchestration(AttachOrCreate)` reuses only exact owner match
- different owner with identical inputs gets a new world
- `ReplaceExpectedGeneration` increments generation and returns new binding
- generation mismatch returns explicit error, not fallback reuse

#### `crates/world-agent/src/pty.rs`

Add protocol tests for:

- `start_session` shared-world payload parse
- `Ready.shared_world` echo shape
- protocol fail-closed when owner-bound session gets `Ready` without generation

#### `crates/world-agent/src/service.rs`

Add transport tests for:

- non-PTY `ExecuteRequest.shared_world` to owner-aware `WorldSpec`
- response echo of `SharedWorldBindingSnapshot`
- no `span_id -> orchestration_session_id` fallback on shared-owner streamed events

#### `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`

Add host-side protocol tests for:

- owner-bound PTY session validates `Ready.shared_world`
- missing or wrong owner echo aborts the session
- backend-returned generation replaces provisional local generation

#### `crates/shell/src/execution/platform_world/windows.rs`

Add tests for:

- explicit shared-owner requests reject before `ensure_session()` generic fallback
- generic bootstraps remain unchanged

#### `crates/shell/src/execution/platform_world/mod.rs`

Add tests for:

- Linux path allows explicit shared-owner mode
- macOS and Windows paths reject it uniformly

### Test commands

Run at minimum:

```bash
cargo test -p world session -- --nocapture
cargo test -p world --lib ensure_session -- --nocapture
cargo test -p world-agent repl_persistent_session_bootstrap_v1 -- --nocapture
cargo test -p world-agent --lib execute_stream -- --nocapture
cargo test -p substrate-shell repl_world_first_routing_v1 -- --nocapture
```

Then run:

```bash
cargo test -p world-agent -- --nocapture
cargo test -p substrate-shell -- --nocapture
cargo test --workspace -- --nocapture
```

### QA artifact

Primary QA artifact for follow-up verification:

[spensermcconnell-feat-session-participant-record-cutover-eng-review-test-plan-20260429-164041.md](/Users/spensermcconnell/.gstack/projects/atomize-hq-substrate/spensermcconnell-feat-session-participant-record-cutover-eng-review-test-plan-20260429-164041.md)

## Failure Modes Registry

| New codepath | Real production failure | Test covers it? | Error handling exists? | User sees clear error? | Critical gap? |
|---|---|---|---|---|---|
| owner-bound attach/create | different orchestration silently reuses same Linux world because spec matched | planned | planned | yes | yes until fixed |
| owner-bound replace | stale expected generation replaces the wrong world chain | planned | planned | yes | yes until fixed |
| PTY ready proof | shell accepts `world_id` without generation and invents authority locally | planned | planned | yes | yes until fixed |
| non-PTY owner proof | API caller cannot tell whether execution used the intended owned world | planned | planned | yes | yes until fixed |
| legacy metadata path | old `session.json` is adopted by owner-bound flow | planned | planned | yes | yes until fixed |
| stale partial metadata | crash leaves a half-bound world that becomes reusable later | planned | planned | yes | yes until fixed |
| non-Linux boundary | explicit shared-owner request downgrades to generic reuse on macOS/Windows | planned | planned | yes | yes until fixed |
| streamed event attribution | trace records fake orchestration ownership from `span_id` | planned | planned | partial | no, but must still be fixed |

Critical gap rule:

If any owner-bound request can silently fall back to generic compatible reuse, the slice is not done.

## Performance Review

No new cache is justified here.

Hard rules:

- owner-bound lookup may scan persisted Linux world metadata because this is a startup/control-plane path, not a hot inner loop
- keep generic and shared-owner lookup codepaths separate, but share compatibility helpers
- do not add a shell-side binding cache to "speed up" status or REPL reads in this slice
- compute compatibility fingerprints once per backend path, not separately in shell and backend

Footguns to avoid:

1. Scanning world metadata on every streamed event. Wrong layer.
2. Maintaining both shell-authoritative JSON and Linux-authoritative JSON. Two sources of truth is not performance, it is archaeology.
3. Making non-Linux backends emulate Linux ownership logic in this slice. That spends an innovation token for negative value.

## Worktree Parallelization Strategy

There is a clean split after the contract surface lands.

### Dependency table

| Step | Modules touched | Depends on |
|---|---|---|
| Contract types | `crates/world-api/`, `crates/agent-api-types/`, `llm-last-mile/` | — |
| Linux metadata + reuse | `crates/world/` | contract types |
| PTY + non-PTY world-agent plumbing | `crates/world-agent/` | contract types, Linux binding snapshot |
| Shell routing gates | `crates/shell/src/execution/platform_world/`, `crates/shell/src/execution/routing/`, `crates/shell/src/repl/` | contract types, world-agent plumbing |
| Tests + docs | `crates/world/tests/`, `crates/world-agent/tests/`, `crates/shell/tests/`, `docs/` | all above |

### Parallel lanes

Lane A: contract types -> Linux metadata + reuse  
Why: `WorldReuseMode` and `SharedWorldBindingSnapshot` define the semantics every other lane consumes.

Lane B: PTY + non-PTY world-agent plumbing  
Why: can branch once Lane A finalizes the shared binding snapshot shape.

Lane C: shell routing gates  
Why: can branch once Lane B finalizes what PTY/non-PTY responses echo.

Lane D: tests + docs cleanup  
Why: last lane, validates the final public contract instead of a moving target.

### Lane ownership rules

- Lane A owns `crates/world-api/`, `crates/agent-api-types/`, and `crates/world/`
- Lane B owns `crates/world-agent/`
- Lane C owns `crates/shell/src/execution/platform_world/`, `crates/shell/src/execution/routing/`, and the minimal response-consumer seams in `async_repl.rs`
- Lane D owns tests and docs after interfaces settle

### Execution order

1. Launch Lane A first.
2. After Lane A lands type signatures, launch Lane B.
3. After Lane B lands response shapes, launch Lane C.
4. Merge B before C if shell response validation depends on final transport fields.
5. Run Lane D last.

### Conflict flags

- Lane A and Lane B both care about `SharedWorldBindingSnapshot` shape
- Lane B and Lane C both care about PTY `Ready` and non-PTY response fields
- Lane D can step on any lane if started too early

Safe rule:

Do not start shell routing validation before world-agent response shapes are frozen.

## Deferred Work

There is no `TODOS.md` in this repo root, so deferrals stay here explicitly.

1. Runtime-manifest persistence of `world_id` and `world_generation`  
Why: this is the explicit job of [PLAN-04](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/04-thread-world-binding-into-runtime-state.md).

2. Participant invalidation and registry cutover on generation change  
Why: this is the explicit job of [PLAN-05](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/05-restart-invalidation-semantics.md).

3. Session-centric grouped store layout and query reshaping  
Why: this belongs to slice `06`, not the owner-contract slice.

4. Non-Linux shared-world ownership semantics beyond fail-closed rejection  
Why: Linux-first means Linux-first. Compile support plus explicit rejection is enough here.

## NOT in Scope

- runtime-manifest projection of owner binding
- `substrate agent status` live-selection redesign
- participant invalidation or replacement registry semantics
- generalized cleanup daemon for abandoned worlds
- non-Linux owned shared-world support
- gateway/nested status schema redesign

## Definition of Done

The slice is done when all of these are true:

1. `world-api` can represent a structurally explicit shared-owner world request.
2. Linux `session.json` persists owner-bearing metadata plus generation.
3. Generic callers preserve current reuse semantics.
4. Shared-owner attach/create reuses only exact owner match plus compatible inputs.
5. Shared-owner replace increments generation only through an explicit replace action.
6. PTY `Ready` and non-PTY `ExecuteResponse` both echo authoritative shared binding snapshots.
7. Non-Linux explicit shared-owner requests fail closed before generic world reuse.
8. No shared-owner world-agent event path aliases `span_id` into orchestration ownership.
9. Docs make it explicit that runtime-state projection and invalidation remain later slices.

## Completion Summary

- Step 0: Scope Challenge - scope reduced from the source SOW; slice `03` now owns owner contract only
- Architecture Review: 5 locked decisions, 5 acceptance gates
- Code Quality Review: 5 implementation guardrails, 4 minimal-diff rules
- Test Review: coverage diagrams produced, 23 concrete gaps/assertions identified
- Performance Review: 0 major issues, 3 no-cache/no-dual-truth rules
- Error & Rescue Registry: written
- NOT in scope: written
- What already exists: written
- TODOS.md updates: deferred scope captured in-plan because no `TODOS.md` exists
- Failure modes: 7 critical gaps flagged until owner-bound reuse is implemented
- Outside voice: ran (`codex exec` strategy + architecture passes), findings incorporated
- Parallelization: 4 lanes, 1 foundational + 2 sequential follow-on lanes + final validation lane
- Lake Score: complete version chosen for every in-slice decision

## Decision Audit Trail

| # | Phase | Decision | Classification | Principle | Rationale | Rejected |
|---|---|---|---|---|---|---|
| 1 | Scope | Narrow slice `03` to owner contract only | Mechanical | Explicit over clever | Prevents overlap with slices `04` and `05` | Keeping runtime-state projection and invalidation in this slice |
| 2 | Authority | Linux metadata/backend is the authoritative owner seam | Mechanical | Pragmatic | Reuse decisions already happen there | Shell-side authoritative binding file |
| 3 | API | Add explicit `WorldReuseMode` | Mechanical | Completeness | Missing owner context must be impossible to confuse with generic reuse | Optional loose owner fields on generic requests |
| 4 | Linux reuse | Split generic and shared-owner paths | Mechanical | Explicit over clever | Silent downgrade is the main risk | One codepath with optional owner checks |
| 5 | Generation | Backend-returned generation is the committed truth | Mechanical | Systems over heroes | Prevents REPL-local arithmetic from becoming authority | Trusting shell-local increment only |
| 6 | Legacy | Ownerless legacy metadata remains generic-only | Mechanical | Minimal diff | No destructive migration required | Adopting legacy ownerless worlds into shared-owner flows |
| 7 | Cross-platform | Non-Linux shared-owner mode rejects before generic ensure-session | Mechanical | Fail closed | Linux-first must be behaviorally true | Letting non-Linux fall through to generic reuse |
| 8 | Telemetry | Remove `span_id -> orchestration_session_id` on shared-owner paths | Mechanical | Explicit over clever | Span correlation is not owner identity | Synthetic ownership attribution |
| 9 | Runtime state | Defer manifest/status binding writes to `PLAN-04` | Mechanical | Scope discipline | Packet already has a dedicated slice for it | Writing runtime authority here too |
| 10 | Restart semantics | Defer participant invalidation to `PLAN-05` | Mechanical | Scope discipline | Owner contract must exist before invalidation rules consume it | Defining full invalidation semantics here |

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | SKIPPED | No separate CEO skill run; strategic scope tightening was incorporated directly into this plan |
| Codex Review | `/codex review` | Independent 2nd opinion | 2 | CLEAR | Outside voice forced the slice-boundary correction, rejected shell-authoritative binding, and flagged silent fallback + stale-world rules |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | CLEAR | Linux owner authority, explicit owner-mode contract, PTY/non-PTY proof echo, non-Linux fail-closed boundary, and full gap-driven test matrix are now locked in |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | SKIPPED | No UI scope |

**CODEX:** Two outside-voice passes materially improved the plan. The first caught dual truth, silent fallback, local-generation authority, and test gaps. The second forced the more important correction: slice `03` should own the Linux owner contract only, while projection and invalidation remain slices `04` and `05`.

**UNRESOLVED:** 0 blocking design decisions remain inside this slice. Intentional deferrals are explicitly recorded above.

**VERDICT:** ENG CLEARED. Outside-voice findings incorporated. `PLAN-03` is ready to implement after `PLAN-02`, and before runtime-state projection, restart invalidation, and grouped-store follow-on work.
