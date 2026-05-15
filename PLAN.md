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

Fix the async persistent-session startup seam so REPL world-session bootstrap no longer crosses the synchronous readiness bridge on macOS, while preserving the existing synchronous bootstrap and request-builder surfaces.

This is a design correction, not a trait rewrite. The slice lands one explicit async readiness path for persistent-session startup, keeps `WorldBackend` synchronous, preserves fail-closed behavior, and makes the Windows parity decision explicit.

## Acceptance Criteria

This plan is complete only when all of the following are true:

1. Persistent-session startup no longer calls `PlatformWorldContext.ensure_ready` from [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs).
2. macOS/Lima REPL startup under the current-thread Tokio runtime no longer panics with the `block_in_place` runtime invariant.
3. Readiness failures surface as normal `Result` errors, not panic unwinds.
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

### Repo-truth findings

1. [`world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs) and the REPL world-session entrypoints in [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) are compiled only on Linux and macOS today. The shell-side persistent-session seam is not a current Windows caller.
2. `PlatformWorldContext` still exposes one synchronous `ensure_ready` closure, and it is actively consumed by sync bootstrap and request-builder paths. Deleting or widening that surface in this slice is cross-cutting churn with no user payoff.
3. Both non-Linux backends already contain the same risk pattern:
   - Lima uses `block_on_compat(...)` in [`world-mac-lima/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs).
   - WSL uses `block_on(...)` in [`world-windows-wsl/src/backend.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs).
4. The real defect is not the websocket protocol and not `WorldBackend` itself. The defect is that an already-async caller is still forced through a sync bootstrap seam that uses sync-over-async capability verification.

### Complexity, completeness, and distribution verdict

1. This is a medium slice. It touches more than 8 files once tests and docs are counted, but the spread is justified because the bug spans one async caller, one shared shell context, and two backend adapter implementations.
2. The complete fix is still the right fix. A hotfix inside `block_on_compat(...)` alone is cheaper to type, but it leaves the wrong architecture in place and guarantees the same confusion later.
3. No new binary, package, service, or distribution pipeline is introduced. Distribution is unchanged in this slice.
4. There is no repo-root `TODOS.md` today. Deferred work stays in `NOT in scope` and `Deferred follow-ups` instead of inventing a backlog surface mid-slice.

### Scope ruling

The scope is accepted as-is. It is bigger than a one-line hotfix, but smaller than a trait migration, and that is the right middle.

## Architecture Review

### Problem statement

The repo already has a real async persistent-session startup path:

```text
CURRENT BROKEN STARTUP
======================
async REPL current-thread runtime
    -> open_world_session(...)
    -> ReplPersistentSessionClient::start_with(...)
    -> build_ws_and_start_session_frame(...)
    -> ctx.ensure_ready()                    [sync bridge]
    -> backend.ensure_session(...)
    -> ensure_agent_ready(...)
    -> block_on_compat(client.capabilities())
    -> Tokio panic on macOS current-thread runtime
```

The mismatch is narrow but real:

1. the caller is async,
2. the readiness bridge is sync,
3. the backend uses sync-over-async capability verification that is only safe under some runtime shapes.

The fix is not "teach `block_in_place` a new trick." The fix is to stop sending an async caller through the sync bridge.

### Architecture thesis

Caller shape decides the bootstrap adapter. Readiness ownership stays with the backend, but the shell must stop pretending sync and async callers are the same thing.

### Target architecture

```text
TARGET STARTUP MODEL
====================
async REPL current-thread runtime
    -> open_world_session(...)
    -> build_ws_and_start_session_frame(...)
    -> ensure_persistent_session_ready_async(...)
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

### Readiness ownership split

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

### Caller contract matrix

| Caller shape | Current entrypoint | Required readiness path after this slice | Notes |
| --- | --- | --- | --- |
| Async REPL persistent-session startup | [`open_world_session(...)` in async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs) | `ensure_persistent_session_ready_async(...)` | Must never call `ctx.ensure_ready()` on macOS again. |
| Linux persistent-session startup | [`build_ws_and_start_session_frame(...)` in world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs) | existing Linux readiness path | Linux already uses the world-agent readiness flow and stays boring. |
| Sync platform bootstrap | [`routing/world.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs) | existing `ctx.ensure_ready()` | No signature or behavior rewrite. |
| Sync request builders | [`world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | existing `ctx.ensure_ready()` | Regression coverage required because these are easy to break silently. |
| Workspace sync capabilities checks | [`workspace_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs) | dedicated runtime `block_on(...)` already local to workspace sync | Not the main bug, but still part of the sync-surface preservation check. |

### Frozen contract

If implementation wants to violate any rule below, stop and revise the plan first.

1. `WorldBackend` stays synchronous in this slice.
2. `PlatformWorldContext.ensure_ready` remains available and continues to serve sync callers.
3. Persistent-session startup must not call `ensure_ready` on macOS. Linux may keep its existing direct world-agent readiness path.
4. Explicit `SUBSTRATE_WORLD_SOCKET` overrides continue to bypass Lima detection and startup exactly as they do today.
5. VM/forwarder/client/capabilities rules remain backend-owned. Shell code may orchestrate, not duplicate.
6. All readiness failures remain fail closed. A missing VM, failed forwarder, failed capabilities probe, or bad websocket connect returns an error, never a best-effort degraded session.
7. Windows/WSL does not gain a new persistent-session product surface in this slice. Only internal readiness parity and hardening land there.

## Code Quality Review

### Explicit design choices

1. Add one shell-facing async helper for persistent-session startup. Do not generalize this into a broad "all world readiness is async now" abstraction.
2. Keep backend-owned readiness logic central. The shell helper dispatches to backend adapters; it does not reimplement VM boot, forwarding, or capability checks.
3. Split shared backend internals by behavior, not by platform folklore:
   - shared setup,
   - shared async verification,
   - sync wrapper,
   - async wrapper.
4. Harden sync bridge helpers as defense-in-depth only. If `block_on_compat(...)` or WSL `block_on(...)` are touched, the change is there to produce a controlled error instead of a panic, not to justify leaving the async caller on the sync bridge.
5. Preserve Linux simplicity. Do not backdoor a second non-Linux abstraction into Linux just for symmetry.

### Naming and ownership expectations

The plan is intentionally opinionated about the shape of the new seam:

1. The shell helper should be named for the real caller, for example `ensure_persistent_session_ready_async(...)`, not a generic future-looking async world API.
2. macOS and WSL should each expose backend-local async readiness wrappers that reuse the same internal setup/verification steps as the sync wrapper.
3. `PlatformWorldContext` remains the sync compatibility carrier. The new async path can be implemented as a free helper plus backend dispatch rather than an async closure stored inside the context.

### DRY guardrails

The following are non-negotiable:

1. No shell-side duplication of Lima forwarding or agent capability logic.
2. No one-off macOS-only bypass that leaves WSL with the same trap hidden in a different crate.
3. No separate definitions of "ready" for sync and async startup.

## Implementation Plan

### Phase summary

| Phase | Purpose | Modules touched | Hard dependency |
| --- | --- | --- | --- |
| 1 | Define async shell bootstrap contract | `crates/shell/src/execution/platform_world/`, `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs` | — |
| 2 | Split macOS readiness internals into shared setup plus sync/async adapters | `crates/world-mac-lima/` | 1 |
| 3 | Migrate persistent-session caller to async seam | `crates/shell/src/execution/routing/dispatch/`, `crates/shell/src/repl/` | 1, 2 |
| 4 | Harden WSL parity without adding new product surface | `crates/world-windows-wsl/` | 1 |
| 5 | Prove sync surfaces still behave the same | `crates/shell/tests/`, `crates/shell/src/execution/routing/` | 3 |
| 6 | Document the split and run smoke validation | `docs/`, `llm-last-mile/` | 3, 5 |

### Phase 1: Add the async shell bootstrap seam

Files:

1. [platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs)
2. [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)

Actions:

1. Introduce one shell-facing async helper for persistent-session startup. Its job is to:
   - preserve the existing Linux readiness behavior,
   - detect or reuse the platform world context for macOS,
   - dispatch to the concrete backend async adapter,
   - and return only when readiness is sufficient to open the transport.
2. Keep `PlatformWorldContext` usable by existing sync callers. Do not change its current `ensure_ready` contract.
3. Freeze the shell-to-backend async contract before parallel backend work starts. This is the merge point every other lane depends on.

Exit criteria:

1. The shell has one explicit async readiness seam for persistent-session startup.
2. No sync caller changes signature.
3. The shell contract is specific enough that macOS and WSL work can proceed in parallel without guessing.

### Phase 2: Refactor macOS/Lima readiness into shared internals plus sync/async adapters

Files:

1. [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs)

Actions:

1. Keep `ensure_vm_running()` and `ensure_forwarding()` as shared sync-safe setup steps.
2. Extract the capabilities verification portion of `ensure_agent_ready()` into an async helper that can be awaited directly after client construction.
3. Keep `ensure_agent_ready()` as the sync wrapper used by `ensure_session(...)`.
4. Add a dedicated async readiness wrapper for persistent-session startup that reuses the same setup and verification rules.
5. Harden `block_on_compat(...)` so accidental future misuse returns a controlled error with context instead of a runtime panic.

Exit criteria:

1. macOS sync `ensure_session(...)` still works.
2. macOS async persistent-session bootstrap no longer touches `block_on_compat(...)` on the happy path.
3. Sync and async Lima readiness share the same setup and verification logic.

### Phase 3: Migrate persistent-session startup to the async seam

Files:

1. [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs)
2. [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Actions:

1. Leave `ReplSessionStartParams`, policy/env normalization, shared-world request wiring, and websocket framing alone.
2. Replace the macOS `ctx.ensure_ready` call with the new async readiness helper.
3. Preserve `SUBSTRATE_WORLD_SOCKET` short-circuiting exactly. The override path is already the clean escape hatch for tests and explicit custom sockets.
4. Keep Linux on its existing direct world-agent readiness path.

Exit criteria:

1. REPL startup remains one straight async flow.
2. The only semantic change is the readiness boundary before transport connect.
3. The macOS path no longer crosses the sync bridge.

### Phase 4: WSL parity hardening, not product-surface expansion

Files:

1. [world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs)
2. [world-windows-wsl/src/tests.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/tests.rs)

Actions:

1. Split WSL capabilities verification and warm-retry logic into shared async internals plus a sync wrapper.
2. Harden `block_on(...)` the same way macOS is hardened.
3. Do not add a Windows persistent-session shell caller in this slice.
4. Document in code comments and docs that this is backend-local parity work for future async callers, not shipped Windows persistent-session support.

Exit criteria:

1. WSL remains compile-safe and behaviorally unchanged for existing sync callers.
2. Future async callers have a safe backend-local seam waiting for them.
3. The Windows parity decision is explicit and test-backed.

### Phase 5: Prove sync callers still work

Files:

1. [routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs)
2. [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs)
3. [workspace_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs)

Actions:

1. Preserve call sites unless a tiny adapter shim is required for compilation.
2. Add regression tests that prove sync bootstrap and sync request-building still succeed and still fail closed.
3. Confirm that workspace sync capability checks remain unaffected by the new persistent-session path.

Exit criteria:

1. Sync callers remain boring.
2. No hidden async creep enters these surfaces.
3. Existing fail-closed semantics are preserved.

### Phase 6: Docs and manual validation

Files:

1. [persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md)
2. [WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
3. optionally [llm-last-mile/README.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md) if the packet index needs a note

Actions:

1. Document the caller-shape split explicitly.
2. Call out that persistent-session startup on macOS uses an async readiness adapter while sync request-builders still use the legacy sync seam.
3. Record the Windows parity decision so nobody mistakes "hardened internals" for "Windows persistent REPL shipped."
4. Run the manual repro smoke and confirm the failure mode is now either success or a normal readiness error.

Exit criteria:

1. The docs reflect the real architecture.
2. The next person reading the code does not have to reverse-engineer why the split exists.
3. The manual repro no longer panics.

## Test Review

Rust is the project runtime and `cargo test` remains the authoritative framework for this slice.

### Code path coverage

```text
CODE PATH COVERAGE
==================
[+] Async REPL startup
    async_repl::start_world_session(...)
        -> open_world_session(...)
        -> ReplPersistentSessionClient::start_with(...)
        -> build_ws_and_start_session_frame(...)
        -> ensure_persistent_session_ready_async(...)
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

| Target | Test requirement | Type |
| --- | --- | --- |
| [`crates/shell/tests/repl_world_first_routing_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs) | Add a current-thread REPL startup regression that executes the persistent-session startup path without panic. | integration |
| [`world_persistent_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs) tests | Cover async readiness success, readiness error propagation, and socket-override bypass. | focused unit/integration |
| [`world-mac-lima/src/lib.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs) tests | Use the existing `session_setup_override` seam to prove sync and async adapters share readiness rules. | crate unit |
| [`world-windows-wsl/src/tests.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/tests.rs) | Cover async-parity helper behavior and sync-wrapper preservation. | crate unit |
| [`routing/world.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs) and [`world_ops.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs) | Add sync regression coverage that proves unchanged success and fail-closed behavior. | integration |

### Regression rule for this slice

The macOS REPL panic is a regression against an existing command surface. That makes the no-panic REPL startup test mandatory. No defer, no follow-up issue, no "manual smoke is enough."

## Failure Modes Registry

| Failure mode | Where it happens | Required handling | Test requirement | Critical gap if missing |
| --- | --- | --- | --- | --- |
| Tokio current-thread panic | macOS REPL startup | Eliminate by moving async caller off sync bridge | REPL startup regression | Yes |
| VM starts but forwarding is absent | macOS async adapter | Return fail-closed readiness error | macOS adapter test | Yes |
| Capabilities probe hangs or fails | macOS + WSL adapters | Return normal error with context, no panic | adapter timeout/failure tests | Yes |
| Explicit socket override accidentally triggers platform bootstrap | macOS startup override path | Preserve short-circuit bypass | override regression test | Yes |
| Sync bootstrap silently starts using async path | `routing/world.rs` or `world_ops.rs` | Keep sync callers on `ensure_ready` | sync regression tests | Yes |
| Future Windows async caller repeats the same trap | WSL backend | Land async-parity internals and harden sync bridge | WSL parity tests | No, but document if partially deferred |

Any failure mode that has no test, no error handling, and only a silent failure is a release blocker for this slice.

## Performance Review

This slice is about startup correctness, but there are still performance guardrails:

1. No duplicate readiness passes. The async path should perform one readiness preparation before transport connect, not re-probe capabilities again immediately after connect unless that is already part of the existing handshake contract.
2. No extra VM warm cycles. Shared setup must remain idempotent enough that moving to an async adapter does not accidentally start the Lima VM twice.
3. No hidden runtime creation on the async happy path. The point of the async seam is to avoid bridging through a blocking helper.
4. Timeout and retry policy must remain backend-owned and unchanged in spirit. Do not silently lengthen startup latency just to mask the design bug.
5. Steady-state REPL throughput is unchanged. There is no new hot-path work after session start.

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
| 1. Define async shell bootstrap contract | `crates/shell/src/execution/platform_world/`, `crates/shell/src/execution/routing/dispatch/` | — |
| 2. Refactor macOS shared readiness internals | `crates/world-mac-lima/` | 1 |
| 3. Harden WSL parity internals | `crates/world-windows-wsl/` | 1 |
| 4. Migrate persistent-session caller to async seam | `crates/shell/src/execution/routing/dispatch/`, `crates/shell/src/repl/` | 1, 2 |
| 5. Add sync-regression coverage | `crates/shell/tests/`, `crates/shell/src/execution/routing/` | 4 |
| 6. Update docs and smoke notes | `docs/`, `llm-last-mile/` | 4, 5 |

### Parallel lanes

Lane A: Step 1 -> Step 4 -> Step 5  
Reason: shared shell modules, sequential changes, one owner

Lane B: Step 2  
Reason: isolated macOS backend work once Step 1 contract is frozen

Lane C: Step 3  
Reason: isolated WSL backend hardening once Step 1 contract is frozen

Lane D: Step 6  
Reason: doc work can start after Step 4 stabilizes, then finish after Step 5 validation lands

### Execution order

1. Land Step 1 first. This defines the contract every other lane depends on.
2. Launch Lane B and Lane C in parallel worktrees after Step 1 merges, or after Step 1 is frozen enough that function names and result shapes will not churn.
3. Merge Lane B before Lane A Step 4 if the shell migration depends on the final macOS adapter names.
4. Run Lane A Step 4 after Lane B is stable, because the shell caller migration is the user-visible fix.
5. Run Lane A Step 5 after Step 4, because sync-regression coverage must validate the final shell wiring.
6. Finish Lane D last, after test names and final behavior are known.

### Conflict flags

1. Lane A and Lane B share the shell-to-backend contract even though they do not share the same files. Freeze function names and error-shape expectations before parallelizing.
2. Lane A Step 5 touches shell routing modules that may also pick up compile fixes from Step 4. Keep those changes in the same lane to avoid merge churn.
3. Lane B and Lane C are safe in parallel. They touch different backend crates.
4. Do not split Step 4 and Step 5 across different worktrees. Both touch the shell routing surface and will conflict.

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
3. Code quality: backend-owned readiness logic shared, no shell-side duplication, no generic future-API abstraction.
4. Tests: REPL startup regression, socket override regression, sync surface regressions, macOS adapter coverage, WSL parity coverage.
5. Performance: no duplicate readiness pass, no extra VM warm cycle, no blocking bridge on the async happy path.
6. `NOT in scope`: written.
7. `What already exists`: written.
8. Failure modes: six named production failures accounted for, five of them critical if left untested.
9. Parallelization: four lanes, two backend lanes safely parallel after the shell contract lands.
10. Lake score: the complete fix wins over the hotfix. The repo pays the right engineering cost once instead of paying the bug twice.
