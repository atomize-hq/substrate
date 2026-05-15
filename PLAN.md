# PLAN: Async Persistent-Session Bootstrap Readiness Split

Source SOW: [26-async-persistent-session-bootstrap-readiness.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/26-async-persistent-session-bootstrap-readiness.md)  
Related slices: [21-macos-lima-shared-owner-and-member-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/21-macos-lima-shared-owner-and-member-runtime-parity.md), [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md), [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)  
Primary protocol doc: [persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md)  
World backend docs: [WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)  
Supersedes: previous root `PLAN.md`, which tracked durable host-orchestrator closeout instead of the slice-26 readiness split  
Current workspace branch: `feat/host-orchestrator-durable-session`  
Base branch: `main`  
Plan type: backend/runtime correction, no UI scope, developer-facing runtime scope  
Status: unified implementation plan, 2026-05-15

## Objective

Fix the async persistent-session startup seam so REPL world-session bootstrap no longer crosses the synchronous readiness bridge on macOS, while keeping the existing synchronous bootstrap and request-builder surfaces intact.

This is a design correction, not a trait rewrite. The plan lands one explicit async readiness path for persistent-session startup, keeps `WorldBackend` synchronous in this slice, preserves fail-closed behavior, and makes the Windows parity decision explicit instead of hand-waving it.

## Acceptance Criteria

This plan is complete only when all of the following are true:

1. Persistent-session startup no longer calls `PlatformWorldContext.ensure_ready` from [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs).
2. macOS/Lima REPL startup under the current-thread Tokio runtime no longer panics with the `block_in_place` runtime invariant.
3. Readiness failures now surface as normal `Result` errors, not panic unwinds.
4. Existing synchronous bootstrap and request-builder callers keep using `ensure_ready` with no async signature expansion.
5. Shared readiness rules remain consistent across sync and async entrypoints for VM start, forwarding, client construction, and capabilities verification.
6. The Windows/WSL backend is either hardened with the same split-ready internals or explicitly documented as a future-only consumer with no current persistent-session caller.
7. Tests and docs make the caller-shape split obvious enough that a future async caller does not regress back through the sync bridge.

## Step 0: Scope Challenge

### What already exists

| Sub-problem | Existing code | Plan decision |
| --- | --- | --- |
| REPL async caller | [`open_world_session(...)` in async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), [`ReplPersistentSessionClient::start_with(...)` in world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs) | Reuse. The caller is already async and stays async. |
| Persistent-session transport/bootstrap | [`build_ws_and_start_session_frame(...)` in world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs) | Reuse transport and handshake logic. Replace only the readiness bridge. |
| Sync platform-world context | [`PlatformWorldContext` in platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs) | Keep `ensure_ready` for sync callers. Add a separate async-ready path instead of widening the existing one. |
| macOS readiness ownership | [`MacLimaBackend` in world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs) | Reuse backend-owned VM/forwarding/client logic. Split adapters, not ownership. |
| Windows readiness ownership | [`WindowsWslBackend` in world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs) | Reuse. Harden parity logic now, but do not invent a Windows persistent-session feature in this slice. |
| Sync bootstrap callers | [`routing/world.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs), [`world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs), [`workspace_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs) | Preserve behavior. Regression-test them instead of rewriting them. |

### Minimum honest diff

The minimum honest implementation touches these module groups:

1. [`crates/shell/src/execution/platform_world/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/)
2. [`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)
3. [`crates/world-mac-lima/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)
4. [`crates/world-windows-wsl/src/backend.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs)
5. targeted tests in [`crates/shell/tests/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/), macOS backend tests in [`crates/world-mac-lima/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs), and WSL tests in [`crates/world-windows-wsl/src/tests.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/tests.rs)
6. docs in [persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md) and [WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)

Anything smaller is a shortcut. It would either leave the async caller on the sync bridge, duplicate readiness rules in shell code, or fail to prove the sync surfaces still behave correctly.

### Complexity, completeness, and distribution verdict

1. This is a medium slice. It touches more than 8 files once tests and docs are counted, but the spread is justified because the bug spans one async caller, one shared shell context, and two backend adapter implementations.
2. The complete fix is still the right fix. A hotfix inside `block_on_compat(...)` alone is cheaper to type, but it leaves the wrong architecture in place and guarantees the same confusion later.
3. No new binary, package, service, or distribution pipeline is introduced. Distribution is unchanged in this slice.
4. There is no repo-root `TODOS.md` today. Deferred work stays in `NOT in scope` and `Deferred follow-ups` instead of inventing a backlog surface mid-slice.

### Repo-truth parity decision

`world_persistent_session.rs` is compiled only on Linux and macOS today. That means the shell-side async persistent-session seam is a macOS problem right now, not a live Windows user path.

The parity decision for this plan is:

1. land the shell-side async readiness split for macOS now,
2. harden and refactor WSL readiness internals now so future async callers do not hit the same sync-over-async trap,
3. do **not** add Windows persistent-session support in this slice.

That is the boring choice. It fixes the real bug without pretending the Windows REPL caller already exists.

## Problem Statement

The repo already has a real async persistent-session startup path:

```text
CURRENT BROKEN STARTUP
======================
async REPL current-thread runtime
    -> open_world_session(...)
    -> ReplPersistentSessionClient::start_with(...)
    -> build_ws_and_start_session_frame(...)
    -> ctx.ensure_ready()                    [sync closure]
    -> backend.ensure_session(...)
    -> ensure_agent_ready(...)
    -> block_on_compat(client.capabilities())
    -> Tokio panic on macOS current-thread runtime
```

The mismatch is narrow but real:

1. the caller is async,
2. the readiness bridge is sync,
3. the backend uses sync-over-async capability verification that is only safe under some runtime shapes.

The fix is not “teach `block_in_place` a new trick.” The fix is to stop sending an async caller through the sync bridge.

## Architecture

### Architecture thesis

Caller shape should decide the bootstrap adapter. Readiness ownership stays with the backend, but the shell must stop pretending sync and async callers are the same thing.

### Target architecture

```text
TARGET STARTUP MODEL
====================
async REPL current-thread runtime
    -> open_world_session(...)
    -> build_ws_and_start_session_frame(...)
    -> ensure_persistent_session_ready_async(ctx)
    -> backend-specific async readiness adapter
    -> transport connect
    -> websocket handshake
    -> ready frame

sync bootstrap / sync request-builders
    -> ctx.ensure_ready()
    -> backend.ensure_session(...)
    -> backend-specific sync adapter
    -> unchanged caller contract
```

### Shared-readiness split

```text
BACKEND READINESS OWNERSHIP
===========================
shared sync-safe steps
    - ensure_vm_running() / warm backend
    - ensure_forwarding()
    - build_agent_client()

shared async verification step
    - capabilities probe
    - timeout / retry policy
    - fail-closed error mapping

sync adapter
    -> shared setup
    -> block_on(private_runtime_or_safe_bridge, async verification)

async adapter
    -> shared setup
    -> await async verification directly
```

This is the whole design. No async `WorldBackend`, no shell-side Lima logic clone, no new protocol.

## Frozen Contract

If implementation wants to violate any rule below, stop and revise the plan first.

1. `WorldBackend` stays synchronous in this slice.
2. `PlatformWorldContext.ensure_ready` remains available and continues to serve sync callers.
3. Persistent-session startup must not call `ensure_ready` except on Linux, where the current startup path is already sync-safe and directly guarded by `ensure_world_agent_ready()`.
4. Explicit `SUBSTRATE_WORLD_SOCKET` overrides continue to bypass Lima detection and startup exactly as they do today.
5. VM/forwarder/client/capabilities rules remain backend-owned. Shell code may orchestrate, not duplicate.
6. All readiness failures remain fail closed. A missing VM, failed forwarder, failed capabilities probe, or bad websocket connect returns an error, never a best-effort degraded session.
7. Windows/WSL does not gain new persistent-session product surface in this slice. Only internal readiness parity and hardening land there.

## Implementation Plan

### Phase 1: Add the async shell bootstrap seam

Files:

1. [platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs)
2. [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)

Actions:

1. Extend `PlatformWorldContext` with an explicit async bootstrap handle or enum that preserves access to the concrete backend needed for persistent-session readiness.
2. Add one shell-facing async helper, named for persistent-session startup rather than a generic future rewrite. The helper should perform detection/context hydration exactly once, then dispatch to the concrete backend async adapter.
3. Keep Linux behavior simple. Linux continues using the existing world-agent readiness flow and does not need the new non-Linux forwarded-bootstrap path.

Exit criteria:

1. `build_ws_and_start_session_frame(...)` can await an async readiness helper on macOS.
2. No sync caller changes signature.

### Phase 2: Refactor macOS/Lima readiness into shared internals plus sync/async adapters

Files:

1. [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)

Actions:

1. Keep `ensure_vm_running()` and `ensure_forwarding()` as shared sync-safe setup steps.
2. Extract capabilities verification into an async helper that can be awaited directly.
3. Make `ensure_agent_ready()` the sync wrapper and add a dedicated async wrapper for persistent-session startup.
4. Tighten `block_on_compat(...)` so accidental future misuse fails predictably instead of panicking in obscure ways.

Exit criteria:

1. macOS sync `ensure_session(...)` still works.
2. macOS async persistent-session bootstrap no longer touches `block_on_compat(...)`.

### Phase 3: Migrate persistent-session startup to the async seam

Files:

1. [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)
2. [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Actions:

1. Leave `ReplSessionStartParams`, policy/env normalization, shared-world request wiring, and websocket framing alone.
2. Replace the macOS `ctx.ensure_ready` call with the new async readiness helper.
3. Preserve socket-override short-circuiting exactly. The override path is already the clean escape hatch for tests and explicit custom agent sockets.

Exit criteria:

1. REPL startup path is still one straight async flow.
2. The only semantic change is the readiness boundary before transport connect.

### Phase 4: WSL parity hardening, not product-surface expansion

Files:

1. [world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs)
2. [world-windows-wsl/src/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/tests.rs)

Actions:

1. Split WSL capabilities verification and warm-retry logic into shared async internals plus a sync wrapper.
2. Harden `block_on(...)` the same way macOS is hardened.
3. Do **not** thread a Windows async persistent-session caller through shell code in this slice.

Exit criteria:

1. WSL remains compile-safe and behaviorally unchanged for existing sync callers.
2. Future async callers have a safe backend-local seam waiting for them.

### Phase 5: Prove sync callers still work

Files:

1. [routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs)
2. [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
3. [workspace_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs)

Actions:

1. Preserve call sites unless a tiny adapter shim is required for compilation.
2. Add regression tests that prove sync bootstrap and sync request-building still succeed and still fail closed.

Exit criteria:

1. Sync callers remain boring.
2. No hidden async creep enters these surfaces.

### Phase 6: Docs and manual validation

Files:

1. [persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md)
2. [WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
3. optionally [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if the packet index needs a note

Actions:

1. Document the caller-shape split explicitly.
2. Call out that persistent-session startup on macOS uses an async readiness adapter while sync request-builders still use the legacy sync seam.
3. Record the Windows parity decision so nobody mistakes “hardened internals” for “Windows persistent REPL shipped.”

Exit criteria:

1. The docs reflect the real architecture.
2. The next person reading the code does not have to reverse-engineer why the split exists.

## Test Strategy

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] Async REPL startup
    async_repl::start_world_session(...)
        -> open_world_session(...)
        -> ReplPersistentSessionClient::start_with(...)
        -> build_ws_and_start_session_frame(...)
        -> async readiness helper
        -> websocket connect / ready frame
    Required tests:
        - macOS current-thread startup succeeds without panic
        - readiness failure returns Result error
        - explicit SUBSTRATE_WORLD_SOCKET override still bypasses bootstrap

[+] Sync bootstrap callers
    routing/world.rs
        -> ctx.ensure_ready()
        -> backend.ensure_session(...)
    world_ops.rs request builders
        -> ctx.ensure_ready()
        -> AgentClient transport selection
    workspace_cmd.rs capabilities checks
        -> dedicated runtime.block_on(...)
    Required tests:
        - sync bootstrap unchanged
        - sync request builders unchanged
        - fail-closed behavior preserved

[+] Backend adapters
    MacLimaBackend
        -> ensure_vm_running()
        -> ensure_forwarding()
        -> async capabilities probe
    WindowsWslBackend
        -> warm / retry path
        -> async capabilities probe
    Required tests:
        - async adapter success
        - probe failure returns normal error
        - sync adapter still works
```

### User-flow and error-state coverage

```text
USER FLOW COVERAGE
==================
[+] macOS REPL startup, default transport
    - world session starts under current-thread runtime
    - user sees normal REPL startup, not Tokio panic

[+] macOS REPL startup, readiness failure
    - VM unavailable / forwarder unavailable / capabilities probe fails
    - user sees explicit readiness failure

[+] macOS REPL startup, explicit socket override
    - no Lima bootstrap side effects
    - custom socket still connects

[+] Sync command surfaces
    - world bootstrap still works
    - world_ops request builders still work
    - workspace capabilities checks still work

[+] WSL backend hardening
    - existing sync flows unaffected
    - future async callers no longer depend on unsafe sync-over-async behavior
```

### Concrete test additions

1. Extend [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) with a current-thread REPL startup regression that executes the persistent-session startup path without panic.
2. Add or extend focused tests around [`world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs) for:
   - async readiness success,
   - readiness error propagation,
   - socket override bypass.
3. Extend macOS backend tests in [`world-mac-lima/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs) using the existing `session_setup_override` seam to prove sync and async adapters share readiness rules.
4. Extend WSL tests in [`world-windows-wsl/src/tests.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/tests.rs) for async-parity helper behavior and sync-wrapper preservation.
5. Add sync regression coverage for [`routing/world.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs) and the request-builder helpers in [`world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs).

## Failure Modes Registry

| Failure mode | Where it happens | Required handling | Test requirement | Critical gap if missing |
| --- | --- | --- | --- | --- |
| Tokio current-thread panic | macOS REPL startup | Eliminate by moving async caller off sync bridge | REPL startup regression | Yes |
| VM starts but forwarding is absent | macOS async adapter | Return fail-closed readiness error | macOS adapter test | Yes |
| Capabilities probe hangs or fails | macOS + WSL adapters | Return normal error with context, no panic | adapter timeout/failure tests | Yes |
| Explicit socket override accidentally triggers platform bootstrap | macOS startup override path | Preserve short-circuit bypass | override regression test | Yes |
| Sync bootstrap silently starts using async path | routing/world.rs or world_ops.rs | Keep sync callers on `ensure_ready` | sync regression tests | Yes |
| Future Windows async caller repeats the same trap | WSL backend | Land async-parity internals and harden sync bridge | WSL parity tests | No, but document if partially deferred |

## NOT in scope

1. Converting [`WorldBackend`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs) into an async trait.
2. Rewriting LinuxLocalBackend.
3. Redesigning the persistent-session protocol or `ready` frame contract.
4. Broad transport refactors unrelated to readiness.
5. Shipping Windows persistent-session REPL support as a new product surface.
6. Changing public CLI behavior outside of replacing the panic with normal readiness errors.

## Deferred Follow-Ups

1. Full async `WorldBackend` migration, if future async callers justify the cost.
2. Broader unification of sync request-builders if those surfaces ever move fully async.
3. Any Windows persistent-session feature work beyond backend hardening.

## Worktree Parallelization Strategy

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| 1. Define async shell bootstrap contract | `crates/shell/src/execution/platform_world/` | — |
| 2. Refactor macOS shared readiness internals | `crates/world-mac-lima/` | 1 |
| 3. Harden WSL parity internals | `crates/world-windows-wsl/` | 1 |
| 4. Migrate persistent-session caller to async seam | `crates/shell/src/execution/routing/dispatch/`, `crates/shell/src/repl/` | 1, 2 |
| 5. Add sync-regression coverage | `crates/shell/tests/`, `crates/shell/src/execution/routing/` | 4 |
| 6. Update docs and smoke notes | `docs/`, `llm-last-mile/` | 4, 5 |

### Parallel lanes

Lane A: Step 1 -> Step 4 -> Step 5  
Reason: shared shell modules, sequential changes

Lane B: Step 2  
Reason: isolated macOS backend work once Step 1 contract is set

Lane C: Step 3  
Reason: isolated WSL backend hardening once Step 1 contract is set

Lane D: Step 6  
Reason: can start after Step 4 is stable, then finish after Step 5 validation lands

### Execution order

1. Land Step 1 first. This defines the contract every other lane depends on.
2. Launch Lane B and Lane C in parallel worktrees after Step 1 merges or at least the contract is frozen.
3. Merge B first if macOS is the primary blocker, then complete Lane A Step 4 against the landed contract.
4. Run Step 5 after Step 4 because sync-regression coverage needs the final shell wiring in place.
5. Finish docs last, after test names and behavior are final.

### Conflict flags

1. Lane A and Lane B both conceptually depend on the shell-to-backend contract. Freeze function names and result types before parallelizing.
2. Lane A Step 5 touches shell routing modules that may also pick up small compile fixes from Step 4. Keep those changes in the same lane to avoid churn.
3. Lane B and Lane C are safe in parallel. They touch different backend crates.

## Validation Commands

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell -- --nocapture
cargo test -p world-mac-lima -- --nocapture
cargo test -p world-windows-wsl -- --nocapture
script -q /dev/null zsh -lc 'RUST_BACKTRACE=1 ~/.substrate/bin/substrate'
```

If the macOS manual smoke still fails, it must fail as a normal readiness error. A Tokio runtime panic means the slice is not done.

## Completion Summary

This plan is ready to implement when the work lands with this end state:

1. Step 0: scope accepted as-is, no trait rewrite, no protocol rewrite.
2. Architecture: one async persistent-session readiness seam added, sync seam preserved.
3. Code quality: backend-owned readiness logic shared, no shell-side duplication.
4. Tests: REPL startup regression, socket override regression, sync surface regressions, macOS adapter coverage, WSL parity coverage.
5. Performance: unchanged in the steady state. This slice is about correctness and startup safety, not throughput tuning.
6. `NOT in scope`: written.
7. `What already exists`: written.
8. Failure modes: six named production failures accounted for, four of them critical if left untested.
9. Parallelization: four lanes, two backend lanes safely parallel after the shell contract lands.
10. Lake score: the complete fix wins over the hotfix. The repo pays the right engineering cost once instead of paying the bug twice.
