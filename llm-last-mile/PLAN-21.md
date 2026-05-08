# PLAN-21: macOS/Lima Shared-Owner and Member-Runtime Parity

Source SOW: [21-macos-lima-shared-owner-and-member-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/21-macos-lima-shared-owner-and-member-runtime-parity.md)  
Gap matrix anchor: [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:171)  
Adjacent landed slices: [PLAN-15.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-15.md), [PLAN-17.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-17.md), [PLAN-19.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-19.md)  
Branch: `feat/session-centric-state-store`  
Base branch: `main`  
Plan type: cross-platform orchestration parity, Linux contract extended across the Lima forwarded seam  
Review posture: unified execution plan, tightened to `/autoplan` and `/plan-eng-review` rigor  
Status: execution-ready planning pass on 2026-05-08

## Objective

Bring macOS/Lima onto the same explicit shared-owner and retained member-runtime contract Linux already uses.

This slice is complete only when a macOS host can:

1. open or replace a shared REPL world through the forwarded Lima guest without host-side pre-rejection,
2. lazily launch and retain a world-scoped member runtime through the existing guest `member_dispatch` path,
3. send targeted follow-up turns through `/v1/member_turn/stream`,
4. cancel bootstrap or submitted turns through `/v1/execute/cancel`,
5. preserve authoritative shared-world binding and dispatch fields across backend-level `world-api` seams instead of silently zeroing them.

This is parity work, not a new orchestration model.

## Plan Summary

The repo already has the right Linux contract and most of the host-to-guest plumbing. The gap is narrower than it looks and more structural than it looks.

The real blockers are:

1. the macOS host still rejects explicit shared-owner bootstrap before the forwarded guest can answer,
2. shell member-runtime orchestration is still compiled behind Linux-only gates and hard-errors on macOS,
3. the backend seam still drops orchestration-sensitive fields because [`ExecRequest`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:141) does not carry them and [`MacLimaBackend::convert_exec_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:275) hardcodes `shared_world: None` and `member_dispatch: None`.

The minimum honest implementation is one cohesive slice with four ordered workstreams:

1. remove the macOS shared-owner bootstrap rejection and prove forwarded `ready.shared_world`,
2. widen backend contracts so `world-api`, `world-agent`, and `world-mac-lima` can preserve shared-owner and member-dispatch semantics,
3. widen shell member-runtime orchestration off Linux-only cfg stubs onto the forwarded guest path,
4. add reproducible Lima orchestration validation and update docs to match reality.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Shared-owner request construction for REPL attach/create and replacement | [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:5596), [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:5816) | Reuse. Do not invent a second ownership shape for macOS. |
| Fail-closed validation of echoed owner proof | [`validate_shared_world_echo(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308) | Reuse exactly. This remains the authoritative shell-side validator. |
| macOS forwarded persistent-session transport | [`build_ws_and_start_session_frame(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:731) | Reuse the forwarded guest path. Remove only the non-Linux rejection. |
| macOS forwarded member-dispatch request builder | [`world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1369) | Reuse. Do not create a separate macOS request model. |
| Guest-side authoritative binding validation | [`validate_member_dispatch_binding(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2565) | Reuse exactly. The guest remains the world-sensitive authority. |
| Guest-side retained follow-up turn validation | [`validate_submit_turn_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869) | Reuse exactly. No loosening of identity checks. |
| Backend shared-binding surface | [`WorldHandle.shared_binding`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:131) | Reuse the field, but make Lima populate it when explicit shared-owner mode is active. |
| Existing session cache in Lima backend | [`MacLimaBackend::ensure_session(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:389) | Preserve. Do not regress session reuse. |

### Exact remaining gap

1. [`reject_non_linux_shared_owner_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:61) still blocks the supported macOS path before the guest can prove anything.
2. [`build_ws_and_start_session_frame(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:731) already forwards on macOS, but the guard means explicit shared-owner mode never reaches that transport.
3. [`ensure_member_runtime_ready_for_descriptor(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4298) and [`submit_world_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4684) still hard-error on non-Linux.
4. [`ExecRequest`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:141) still lacks `shared_world` and `member_dispatch`, so backend-level parity is impossible without widening the contract.
5. [`MacLimaBackend::convert_exec_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:275) and [`MacLimaBackend::ensure_session(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:389) still erase exactly the fields Linux relies on.

### Scope decision

Proceed as one parity slice.

Do not split this into a "bootstrap only" PR plus a later "member runtime maybe" PR. That would ship a partial contract and make the docs lie for another cycle. The shell either has real macOS/Lima parity for shared-owner plus retained members, or it does not.

## Frozen Execution Contract

If implementation wants to do something else, revise this plan first.

### Non-negotiable invariants

1. Linux remains the contract source of truth.
2. Shared-world ownership stays explicit via `SharedWorldOwnerSpec` and authoritative via `SharedWorldBindingSnapshot`.
3. macOS must stop rejecting the supported shared-owner path before guest bootstrap.
4. World-scoped member launch, follow-up, and cancel must fail closed if the forwarded guest contract cannot be established.
5. No host-local fallback is allowed for a world-scoped member runtime.
6. Exact identity checks remain unchanged: `orchestration_session_id`, `orchestrator_participant_id`, `backend_id`, `world_id`, and `world_generation` must still match.
7. Targeted follow-up turns keep using `/v1/member_turn/stream`.
8. Cancel stays guest-owned through `/v1/execute/cancel`.
9. Replacement remains shell-owned and fail-closed through `ReplaceExpectedGeneration`.
10. Backend-level execution and handles must stop silently dropping Linux orchestration fields.

### Blast radius

GitNexus marks both core seam types as `CRITICAL` blast radius changes:

| Symbol | Risk | Why it matters |
| --- | --- | --- |
| [`ExecRequest`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:141) | `CRITICAL` | Direct callers span `world-mac-lima`, replay, examples, and tests. Any field addition must update every constructor and serializer path. |
| [`WorldHandle`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:131) | `CRITICAL` | Direct callers span Linux, macOS, Windows stubs, `world-agent`, and one indexed execution flow. Semantic changes to `shared_binding` affect cross-platform world ownership assumptions. |

Implication: contract widening must land early, stay additive, and keep Linux plus Windows stubs compiling the whole time.

## Step 0: Scope Challenge

### 0A. Minimum honest diff

The minimum honest implementation is:

1. remove the host-side non-Linux shared-owner rejection for the Lima-backed persistent-session path,
2. make backend-level request and handle types capable of carrying shared-owner and member-dispatch state,
3. propagate those fields end to end through `world-agent` and `world-mac-lima`,
4. widen shell member-runtime orchestration so macOS uses the forwarded guest contract instead of Linux-only stubs,
5. add one reproducible macOS/Lima orchestration smoke harness and update docs.

Anything smaller lies about parity. Anything larger is scope creep.

### 0B. Complexity check

This slice touches more than eight files and crosses shell, backend, and guest boundaries. That is a smell. It is still the right scope because the alternative is shipping a fake feature boundary.

Expected primary modules:

1. `crates/shell/src/execution/platform_world/`
2. `crates/shell/src/execution/routing/dispatch/`
3. `crates/shell/src/execution/repl_persistent_session.rs`
4. `crates/shell/src/repl/async_repl.rs`
5. `crates/world-api/src/lib.rs`
6. `crates/world-mac-lima/src/lib.rs`
7. `crates/world-agent/src/service.rs`
8. `scripts/mac/`
9. `docs/WORLD.md`
10. `AGENT_ORCHESTRATION_GAP_MATRIX.md`

That is above the smell threshold, so the plan must stay boring:

1. no new orchestration service,
2. no new request shape,
3. no new resume transport,
4. no platform-specific special case beyond enabling Lima to use the same guest contract Linux already uses.

### 0C. Search and reuse check

The repo already has the core mechanisms this slice needs:

- **[Layer 1]** reuse shell-side proof validation in [`validate_shared_world_echo(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308),
- **[Layer 1]** reuse forwarded guest request construction in [`world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1369),
- **[Layer 1]** reuse guest-side shared binding and member identity validation in [`service.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:2565) and [`member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869),
- **[EUREKA]** the missing piece is not transport invention. The missing piece is host-side honesty plus backend seam parity.

### 0D. TODOS cross-reference

There is no `TODOS.md` in the repo root today. This plan therefore needs to be explicit about deferrals inside its own `NOT in scope` section instead of assuming a separate backlog artifact will preserve intent.

### 0E. Completeness check

This cannot be a happy-path-only parity pass.

The complete version includes:

1. attach/create success,
2. replacement success,
3. retained member lazy launch,
4. retained member follow-up reuse,
5. cancel,
6. mismatch and stale-generation rejection,
7. backend contract preservation,
8. Lima-backed smoke validation,
9. docs and gap-matrix truth updates.

Skipping any one of those saves little implementation time and creates a misleading product claim. Boil the lake.

### 0F. Distribution check

No new binary, package, or container artifact is introduced.

Distribution still matters here:

1. the repo must ship a reproducible `scripts/mac/orchestration-smoke.sh`,
2. docs must stop claiming non-Linux shared-owner rejection once this lands,
3. validation instructions must show how parity is proven from a real macOS host with Lima provisioned.

## Architecture Review

### Architecture thesis

The shell remains the orchestrator owner. The Linux guest remains the world-sensitive authority. Lima remains a transport and backend adapter, not a second orchestration model.

### Data flow

```text
CURRENT
=======
macOS host shell
  |
  | StartSession { shared_world }
  v
reject_non_linux_shared_owner_request()
  |
  `- fails before guest bootstrap

world-member launch on macOS
  |
  v
Linux-only cfg stub
  |
  `- unsupported hard error

backend exec path
  |
  v
ExecRequest / WorldHandle
  |
  `- silently erase shared_world, member_dispatch, shared_binding


TARGET
======
macOS host shell
  |
  | StartSession { shared_world }
  v
forwarded /v1/stream -> Lima guest world-agent
  |
  v
ready.shared_world
  |
  v
validate_shared_world_echo()
  |
  +--> authoritative world_id + world_generation persisted by shell
  |
  +--> member_dispatch launch over forwarded guest path
  |
  +--> /v1/member_turn/stream for targeted turns
  |
  `--> /v1/execute/cancel for bootstrap and submitted-turn cancel

backend trait seam
  |
  v
ExecRequest carries shared_world + member_dispatch
WorldHandle carries authoritative shared_binding
```

### Implementation workstreams

#### Workstream 1: persistent-session shared-owner parity

Goal: allow macOS REPL shared-owner attach/create and replacement to reach the forwarded guest path, then prove the reply through the existing fail-closed validator.

Files:

- [`crates/shell/src/execution/platform_world/mod.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:61)
- [`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:731)
- [`crates/shell/src/execution/repl_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308)

Required changes:

1. Replace the blanket non-Linux shared-owner rejection with a capability check that allows the Lima-backed persistent-session path and still rejects unsupported platforms.
2. Keep [`validate_shared_world_echo(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308) as the only shell-side proof validator.
3. Ensure replacement still uses `ReplaceExpectedGeneration` and still requires generation advancement.
4. Keep failure posture identical to Linux for missing, malformed, mismatched, or inactive proof.

Exit criteria:

1. macOS `StartSession { shared_world }` reaches the forwarded guest path.
2. `ready.shared_world` is required and validated on macOS exactly as on Linux.
3. unsupported non-Lima paths still reject explicitly.

#### Workstream 2: backend contract parity

Goal: stop losing shared-owner and member-dispatch semantics at the `world-api` boundary.

Files:

- [`crates/world-api/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:131)
- [`crates/world-mac-lima/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:275)
- [`crates/world-agent/src/service.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1230)
- direct callers identified by repo search: replay, macOS example, Windows tests and stubs

Required changes:

1. Extend [`ExecRequest`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:141) with additive optional fields for `shared_world` and `member_dispatch`.
2. Update all `ExecRequest` constructors to preserve existing behavior when those fields are absent.
3. Pass the new fields from `world-agent` into backend execution when the request is orchestration-sensitive.
4. Update [`MacLimaBackend::convert_exec_request(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:275) so the forwarded guest request preserves those fields instead of zeroing them.
5. Update [`MacLimaBackend::ensure_session(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:389) so `WorldHandle.shared_binding` is populated when explicit shared-owner mode is active.
6. Keep Linux and Windows implementations additive and compile-clean even if they do not use the new fields yet.

Exit criteria:

1. `ExecRequest` carries optional shared-owner and member-dispatch data.
2. `world-agent` can forward that data through backend exec when relevant.
3. Lima no longer zeros those fields.
4. owner-mode Lima sessions return an authoritative `shared_binding`.

#### Workstream 3: shell member-runtime parity on macOS

Goal: make macOS use the same guest-owned member runtime lifecycle Linux already uses.

Files:

- [`crates/shell/src/repl/async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3274)
- [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1369)

Required changes:

1. Widen Linux-only imports and helper access so the macOS build can construct forwarded member-dispatch requests.
2. Replace the current macOS hard-error stubs in [`ensure_member_runtime_ready_for_descriptor(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4298) and [`submit_world_targeted_turn(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4684) with the forwarded guest path.
3. Preserve exact identity requirements for lazy launch, retained follow-up, replacement, and cancel.
4. Keep world-sensitive failures explicit and fail-closed. Unsupported posture must still error, not silently downgrade to host-local ownership.

Exit criteria:

1. macOS no longer hits Linux-only hard-error stubs for supported world-scoped orchestration.
2. lazy launch, targeted follow-up turns, and cancel use the guest-owned contract.
3. mismatch and stale-generation paths still fail closed.

#### Workstream 4: validation and documentation

Goal: prove the whole seam end to end and update repo truth.

Files:

- [`scripts/mac/lima-warm.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
- `scripts/mac/orchestration-smoke.sh` (new)
- [`scripts/mac/smoke.sh`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
- [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:86)
- [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:175)

Required changes:

1. Add a reproducible orchestration smoke harness for macOS/Lima.
2. Document the new validation workflow and remove stale Linux-only claims where this slice changes reality.
3. Record the parity outcome in the gap matrix.

Exit criteria:

1. there is a single commandable macOS/Lima orchestration smoke path,
2. docs no longer claim pre-bootstrap rejection on macOS for the supported path,
3. the gap matrix reflects reality, not intent.

## Code Quality Review

### Boring-by-default rules

1. One request shape. The shell must keep using the existing Linux `member_dispatch` contract.
2. One proof validator. Do not duplicate `ready.shared_world` validation logic in a macOS-specific helper.
3. One backend seam. If `ExecRequest` is widened, every backend caller updates to the same additive type instead of creating a Lima-only side channel.
4. One fallback posture. World-scoped member orchestration either proves the guest contract or fails closed.

### DRY and abstraction guardrails

1. Prefer widening existing helpers over adding parallel macOS versions.
2. The platform helper in [`platform_world/mod.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:61) should become capability-aware, not disappear into duplicated call-site conditionals.
3. Keep member dispatch request assembly in one place so Linux and macOS cannot drift on field population.
4. Do not introduce a new orchestration service, daemon, or abstract trait tower. This slice is already spending its complexity budget on cross-platform parity.

### Diagram maintenance

No nearby ASCII diagrams were identified in the cited seams during this planning pass. If implementation finds any local diagrams in touched files, updating them is part of the same commit, not optional cleanup.

## Test Review

Rust test infrastructure is already the source of truth here: targeted `cargo test -p ...` plus workspace coverage, then Lima-backed smoke validation.

### Code path coverage plan

```text
CODE PATH COVERAGE PLAN
=======================
[+] Shared-owner persistent-session bootstrap
    |
    |- attach/create on macOS forwarded path
    |  |- [GAP -> UNIT] request no longer rejected before guest bootstrap
    |  `- [GAP -> SMOKE] real Lima guest returns authoritative ready.shared_world
    |
    `- replacement on macOS forwarded path
       |- [GAP -> UNIT] ReplaceExpectedGeneration reply must advance generation
       `- [GAP -> SMOKE] stale generation fails closed after rollover

[+] Backend contract parity
    |
    |- ExecRequest carries shared_world/member_dispatch
    |  |- [GAP -> UNIT] world-api round-trip and constructor coverage
    |  `- [GAP -> UNIT] world-agent passes new fields into backend exec
    |
    `- WorldHandle.shared_binding preserved on Lima path
       |- [GAP -> UNIT] ensure_session populates shared_binding when owner mode active
       `- [GAP -> UNIT] no regression for generic non-owner sessions

[+] Member dispatch / retained runtime
    |
    |- lazy launch on macOS
    |  |- [GAP -> UNIT] shell no longer hard-errors on non-Linux cfg path
    |  `- [GAP -> SMOKE] guest receives typed member_dispatch launch
    |
    |- targeted follow-up turn
    |  |- [GAP -> UNIT] submit path routes through member_turn API on macOS
    |  `- [GAP -> SMOKE] ::<backend_id> prompt reuses retained guest member
    |
    `- cancel
       |- [GAP -> UNIT] bootstrap cancel routes through execute/cancel
       `- [GAP -> SMOKE] submitted-turn cancel reaches guest owner

[+] Fail-closed mismatch handling
    |
    |- shared-world proof missing/malformed/inactive
    |  `- [GAP -> UNIT] validator rejects all invalid proof states
    |
    `- backend_id / world_id / world_generation mismatch
       |- [GAP -> UNIT] member dispatch binding rejects mismatches
       `- [GAP -> SMOKE] stale generation or wrong backend stays rejected on macOS
```

### Required test files and coverage

| Area | File | Required assertions |
| --- | --- | --- |
| Persistent-session bootstrap | existing tests in [`world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs) and [`repl_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs) | forwarded macOS attach/create, replacement generation advancement, invalid proof rejection |
| Shell member-runtime parity | existing tests around [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | no non-Linux hard-error on supported Lima posture, follow-up routing, cancel routing |
| `world-api` contract | tests adjacent to [`crates/world-api/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs) | round-trip new fields, additive backward compatibility |
| Lima backend | tests adjacent to [`crates/world-mac-lima/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs) | `convert_exec_request(...)` preserves fields, `ensure_session(...)` returns shared binding in owner mode |
| Guest validation | existing tests in [`service.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs) and [`member_runtime.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs) | no identity relaxation, no binding relaxation |
| End-to-end macOS proof | `scripts/mac/orchestration-smoke.sh` (new) | attach/create, replacement, launch, targeted turn, cancel, mismatch failure |

### Required commands

```bash
cargo test -p world-agent member_runtime
cargo test -p shell
cargo test -p world-api
cargo test -p world-mac-lima
cargo test --workspace -- --nocapture
scripts/mac/lima-warm.sh
scripts/mac/orchestration-smoke.sh
```

### Test plan artifact

Implementation should write the normal eng-review test artifact alongside validation work so QA-style follow-up has a durable input:

`~/.gstack/projects/<slug>/<user>-feat-session-centric-state-store-eng-review-test-plan-<datetime>.md`

It should list:

1. shared-owner attach/create on macOS,
2. replacement with generation advancement,
3. world-member lazy launch,
4. targeted follow-up reuse,
5. bootstrap cancel and submitted-turn cancel,
6. mismatch rejection paths,
7. authoritative shared-binding surfacing on the Lima path.

## Performance Review

No major algorithmic or database risk exists here. The real performance risks are accidental session churn and extra guest round trips.

Guardrails:

1. preserve Lima session caching in [`MacLimaBackend::ensure_session(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:389),
2. do not introduce duplicate guest bootstrap calls when a retained member already matches the authoritative binding,
3. keep added request fields additive and optional so non-orchestration backend exec paths do not pay new work beyond serialization,
4. make the smoke harness assert reuse behavior so silent re-spawn regressions are caught.

## Failure Modes Registry

| Codepath | Realistic production failure | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| macOS shared-owner attach/create | guest reply lacks `ready.shared_world` | yes | existing fail-closed validator | explicit bootstrap failure |
| macOS replacement | reply reuses stale `world_generation` | yes | existing generation-advance check | explicit replacement failure |
| member dispatch launch | shell constructs request without authoritative `world_id` or `world_generation` | yes | shell-side bootstrap failure | explicit launch failure |
| retained follow-up turn | stale member survives after rollover | yes | guest identity mismatch rejection | explicit targeted-turn failure, no silent reuse |
| cancel | cancel routed to wrong span or not routed at all | yes | explicit cancel error plus retained task cleanup | explicit cancel failure, not silent hang |
| backend parity | Lima backend still zeros `shared_world` or `member_dispatch` | yes | unit regression coverage | explicit test failure before ship |
| shared binding parity | `WorldHandle.shared_binding` stays `None` in owner mode | yes | unit regression coverage | shared-owner flows fail closed instead of silently downgrading |

Critical gap rule for this plan:

Any path that both depends on authoritative world identity and would otherwise silently fall back to host-local behavior is a release blocker. This slice only ships with explicit proof or explicit failure.

## Detailed Implementation Plan

### Single-worktree execution order

If one engineer is doing this in one worktree, use this order:

1. shared-owner bootstrap parity,
2. backend contract parity,
3. shell member-runtime parity,
4. validation and docs.

Why this order:

1. Workstream 1 is the smallest visible user-facing parity win and proves the forwarded path is real.
2. Workstream 2 is the highest blast-radius change and should land before shell follow-up logic depends on it.
3. Workstream 3 depends on both 1 and 2.
4. Workstream 4 should validate final semantics, not an intermediate state.

### Phase 1: enable forwarded shared-owner bootstrap on macOS

Implementation steps:

1. change the guard in [`platform_world/mod.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:61) from "non-Linux always reject" to "reject only when the backend cannot prove the forwarded shared-owner path",
2. thread that capability into [`world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:731) without introducing a second request path,
3. keep [`validate_shared_world_echo(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308) unchanged except for any additive test coverage needed for macOS call sites,
4. add or update unit tests for attach/create, replacement, and invalid proof on the forwarded macOS path.

Phase-1 exit criteria:

1. macOS explicit shared-owner requests reach the forwarded guest transport,
2. attach/create and replacement both validate authoritative reply data,
3. malformed or missing proof still fails closed.

### Phase 2: widen backend request and handle contracts

Implementation steps:

1. add optional `shared_world` and `member_dispatch` fields to [`ExecRequest`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:141),
2. update every constructor and test that instantiates `ExecRequest` to remain explicit about absence or presence,
3. ensure `world-agent` can pass those fields into backend execution when the request is orchestration-sensitive,
4. update Lima conversion so those fields survive host-to-guest forwarding,
5. update Lima ensure-session handling so `WorldHandle.shared_binding` is populated in owner mode,
6. keep Linux and Windows backends compiling by making the new fields additive, not required.

Phase-2 exit criteria:

1. all `ExecRequest` call sites compile cleanly,
2. owner-sensitive requests preserve their fields across the backend seam,
3. owner-mode Lima handles surface authoritative binding data.

### Phase 3: widen shell member-runtime orchestration for macOS

Implementation steps:

1. remove the macOS hard-error posture from member-runtime readiness for supported Lima-backed world orchestration,
2. route lazy launch through the existing forwarded request builder in [`world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1369),
3. route targeted turns through `/v1/member_turn/stream` on macOS exactly as Linux already does,
4. route bootstrap and submitted-turn cancel through `/v1/execute/cancel`,
5. add or update unit coverage for reuse, stale generation, wrong backend, and cancel paths.

Phase-3 exit criteria:

1. macOS can lazily launch a world-scoped member through the guest,
2. targeted follow-up turns reuse that retained member when the binding still matches,
3. stale or mismatched identity fails closed,
4. cancel remains guest-owned.

### Phase 4: validate and document parity

Implementation steps:

1. add `scripts/mac/orchestration-smoke.sh`,
2. make the script cover attach/create, replacement, lazy launch, targeted follow-up, cancel, and mismatch rejection,
3. update [`docs/WORLD.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md) so its shared-owner section no longer claims macOS pre-rejection for the supported path,
4. update [`AGENT_ORCHESTRATION_GAP_MATRIX.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) to describe the remaining gap accurately after this lands.

Phase-4 exit criteria:

1. a real macOS host with Lima can run the smoke path successfully,
2. documentation matches shipped behavior,
3. the gap matrix no longer treats macOS/Lima parity as open for this seam.

## Validation Matrix

| Promise | Proof |
| --- | --- |
| macOS shared-owner attach/create no longer pre-rejects | unit tests on persistent-session dispatch plus Lima smoke |
| replacement requires generation advancement | unit test plus smoke case with stale generation rejection |
| backend seam preserves shared-owner and member-dispatch data | `world-api`, `world-agent`, and `world-mac-lima` tests |
| owner-mode Lima handles surface shared binding | `world-mac-lima` unit test |
| macOS can launch and reuse retained members | shell unit tests plus smoke targeted-turn scenario |
| cancel stays guest-owned | shell unit tests plus smoke cancel scenario |
| failure posture remains explicit and fail-closed | validator tests plus smoke mismatch scenarios |
| docs and gap matrix tell the truth | manual diff review in the same PR |

## NOT in scope

- redesigning `substrate -c`, because this slice is parity for existing REPL-first orchestration semantics
- creating a new mac-native orchestration service, because Lima already forwards into the Linux guest authority
- adding Windows/WSL parity, because that is a separate backend rollout
- weakening `backend_id`, `world_id`, `world_generation`, or `orchestration_session_id` checks
- introducing host-local fallback for world-scoped members when guest proof is unavailable
- broadening public control-surface semantics beyond the already frozen session/runtime model

## Worktree Parallelization Strategy

This plan has real parallelization room, but only if ownership boundaries stay clean.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| Shared-owner bootstrap parity | `crates/shell/src/execution/platform_world/`, `crates/shell/src/execution/routing/dispatch/`, `crates/shell/src/execution/repl_persistent_session.rs` | — |
| Backend contract widening | `crates/world-api/`, `crates/world-mac-lima/`, `crates/world-agent/`, backend callers and tests | — |
| Shell member-runtime parity | `crates/shell/src/repl/`, `crates/shell/src/execution/routing/dispatch/world_ops.rs` | shared-owner bootstrap parity, backend contract widening |
| Smoke harness and docs | `scripts/mac/`, `docs/`, `AGENT_ORCHESTRATION_GAP_MATRIX.md` | shell member-runtime parity |

### Parallel lanes

Lane A: shared-owner bootstrap parity  
Sequential within lane because the platform guard and persistent-session transport must agree.

Lane B: backend contract widening  
Sequential within lane because `world-api`, `world-agent`, and `world-mac-lima` all depend on the additive request shape.

Lane C: shell member-runtime parity  
Waits for A + B, then proceeds sequentially through request construction, launch, follow-up, and cancel.

Lane D: smoke harness and docs  
Can draft script structure and docs skeleton early, but final assertions and wording wait for C.

### Execution order

1. Launch Lane A and Lane B in parallel worktrees.
2. Merge A and B.
3. Launch Lane C on top of the merged result.
4. Finish with Lane D once C is stable enough to validate end to end.

### Conflict flags

1. Lane A and Lane C both depend on shell orchestration semantics, but they should avoid merge conflict if A stays inside `platform_world/` and `world_persistent_session.rs` while C owns `async_repl.rs` and `world_ops.rs`.
2. Lane B should not touch `async_repl.rs`. If it does, parallelization value drops fast.
3. Lane D will conflict with anyone still changing the documented contract. Keep docs last.

## Completion Summary

- Step 0: Scope Challenge, accepted as-is. This is the minimum honest parity slice.
- Architecture Review: resolved in-plan. Four workstreams, one contract, no second macOS model.
- Code Quality Review: one request shape, one validator, one backend seam, one fail-closed posture.
- Test Review: coverage diagram produced, all parity-critical gaps called out explicitly.
- Performance Review: low algorithmic risk, high churn-risk if session reuse regresses.
- NOT in scope: written.
- What already exists: written.
- Failure modes: critical-gap rule frozen.
- Parallelization: 4 lanes total, 2 immediately parallel, 2 sequentially dependent.
- Lake Score: the complete option won everywhere that mattered.

## Completion Checklist

- [ ] macOS forwarded persistent-session shared-owner attach/create works
- [ ] macOS replacement requires generation advancement
- [ ] macOS world-member lazy launch uses forwarded `member_dispatch`
- [ ] macOS targeted follow-up turns use `/v1/member_turn/stream`
- [ ] macOS bootstrap and submitted-turn cancel use `/v1/execute/cancel`
- [ ] `ExecRequest` preserves shared-owner and member-dispatch semantics end to end
- [ ] `WorldHandle.shared_binding` is authoritative on the Lima owner path
- [ ] no supported macOS path silently falls back to host-local member ownership
- [ ] new orchestration smoke harness passes on a real macOS host with Lima provisioned
- [ ] docs and gap matrix match the shipped behavior

## Done Means

This slice is done when macOS/Lima stops being a second-class exception and instead uses the same shared-owner proof, retained member launch, targeted turn, cancel, and fail-closed mismatch contract Linux already uses.

Not "mostly there." Not "transport exists but cfg gates still lie." Real parity, proven by code-level tests and a reproducible Lima-backed validation path.
