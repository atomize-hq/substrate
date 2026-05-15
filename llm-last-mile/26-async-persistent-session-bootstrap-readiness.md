# SOW: Async Persistent-Session Bootstrap Readiness Split

Status: implementation-oriented corrective draft. This SOW defines the medium-scope fix for the non-Linux persistent-session startup seam after the macOS/Lima interactive startup panic traced through [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:78), [platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:51), [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:730), and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:443). It is a follow-on to the macOS/Lima parity work in [21-macos-lima-shared-owner-and-member-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/21-macos-lima-shared-owner-and-member-runtime-parity.md), but it intentionally does not reopen the larger `WorldBackend` abstraction or the persistent-session protocol itself.

This slice exists because the repo now has a real async caller for persistent world-session startup, but that caller still routes through a synchronous readiness hook built for older sync-oriented bootstrap surfaces.

## Objective

Land one explicit split between:

- synchronous world-readiness used by legacy/bootstrap/sync request-building code, and
- async world-readiness used by the already-async persistent-session startup path.

This slice is done only when all of the following are true:

1. Persistent-session startup no longer calls the synchronous `ctx.ensure_ready` closure from inside the async REPL runtime.
2. macOS/Lima interactive startup no longer panics when launched under the current-thread Tokio runtime used by the REPL.
3. The sync `PlatformWorldContext.ensure_ready` surface remains available for the existing synchronous bootstrap and request-builder call sites that still rely on it.
4. The new async startup seam reuses the same fail-closed transport/bootstrap rules as the sync path instead of inventing a second backend model.
5. The fix is scoped below the `WorldBackend` trait boundary. `WorldBackend` remains synchronous in this slice.
6. Tests and docs make the split explicit so future async callers do not regress back through the synchronous bridge.

## Why This Follow-On Exists

The crash is the symptom of a real design mismatch, not just an unlucky Tokio call.

### Current startup path

- The interactive REPL starts on a current-thread Tokio runtime in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:452).
- REPL startup immediately opens a world session through [start_world_session(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:6792) and [open_world_session(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:6506).
- `open_world_session(...)` calls [ReplPersistentSessionClient::start_with(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:125), which calls [build_ws_and_start_session_frame(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:730).
- That async bootstrap helper still uses `(ctx.ensure_ready.as_ref())()?` in [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:775).
- On macOS, `ctx.ensure_ready` is the closure installed in [platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:151), which calls `backend.ensure_session(&spec)`.
- On Lima, `ensure_session(...)` immediately calls `ensure_agent_ready()` in [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:512).
- `ensure_agent_ready()` performs a capabilities probe via `block_on_compat(...)` in [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:274).
- `block_on_compat(...)` currently assumes any active Tokio runtime can use `block_in_place(...)` in [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:79), which is false for the REPL’s current-thread runtime and causes the panic.

### Why the sync seam was probably intentional

This is not good evidence that the sync hook was a mistake when introduced. The repo still has real synchronous consumers for it:

- platform bootstrap in [routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:100),
- agent/world request builders in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1316), [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1411), and [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1496),
- and world-dependent workspace sync in [workspace_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs:557).

The design error is narrower:

- an async startup caller is now real,
- but the async caller still crosses the old sync bridge,
- and that bridge reaches backend-local sync-over-async logic that is only safe under some runtime flavors.

## Chosen Implementation Shape

This SOW chooses the medium-scope fix.

### Keep in this slice

- keep `WorldBackend` synchronous in [world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:263),
- keep `PlatformWorldContext.ensure_ready` for sync callers,
- keep the persistent-session websocket protocol and `ready` validation contract,
- keep platform-specific backend ownership of VM/forwarder/agent readiness,
- and keep fail-closed behavior when readiness, forwarding, or agent connectivity cannot be proven.

### Add in this slice

- an explicit async readiness/bootstrap seam for persistent-session startup,
- shared internal readiness logic that can be consumed from both sync and async entrypoints,
- backend-specific async readiness support for non-Linux forwarded backends,
- and tests that prove the async path no longer routes through the sync bridge.

### Deliberately avoid in this slice

- converting `WorldBackend` into an async trait,
- widening every backend call site to async,
- rewriting LinuxLocalBackend,
- redesigning world bootstrap for every command surface,
- or collapsing all readiness into one immediate grand refactor.

## Sizing And Rationale

This is a medium slice, not a tiny hotfix and not a trait-level rewrite.

The sizing rationale is:

- GitNexus shows the async REPL seam itself is relatively contained:
  - [ReplPersistentSessionClient::start_with(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:125) has low upstream fan-in and is primarily reached from [open_world_session(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:6506).
  - [open_world_session(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:6506) is primarily consumed by REPL world start/restart flows.
- The broader sync readiness surface is clearly shared across bootstrap and request-building seams, so deleting it outright would have a larger cross-cutting blast radius.
- `WorldBackend` itself is implemented by Linux, macOS/Lima, Windows/WSL, and the shell test stub, so widening that trait now would force a much larger migration than this fix needs.

The right scope is therefore:

- split the async caller off the sync readiness hook,
- share backend-local readiness internals where possible,
- preserve the sync API for existing sync callers,
- and defer any full trait conversion until there is a separately justified backend-API slice.

## Current Repo Truth

### 1. The async caller is already real

The repo is not planning for an async persistent-session caller in the abstract. It already has one:

- the REPL runs on Tokio in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:452),
- world session startup is awaited in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:527),
- persistent-session websocket bootstrap is async in [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:125),
- and the websocket connect/handshake path is already async through `UnixStream::connect`, `TcpStream::connect`, and `tungs::client_async` in [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:742).

This means the sync bridge in that path is no longer merely a compatibility convenience. It is now an architectural mismatch.

### 2. The sync readiness surface is shared and still needed

The sync readiness hook is not only a macOS artifact:

- `PlatformWorldContext` stores synchronous `ensure_ready` at [platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:51).
- macOS world bootstrap uses it in [routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:103).
- sync request builders use it in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1327), [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1420), and [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1505).

This SOW must not break those call sites in the name of fixing the REPL panic.

### 3. The backend-local readiness logic already exists, but only behind sync bridges

macOS/Lima already owns the right sub-steps:

- VM boot/verification,
- forwarding setup,
- client construction,
- and agent capabilities probe

through [ensure_vm_running(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:117), [ensure_forwarding(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:260), [build_agent_client(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:485), and [ensure_agent_ready(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:274).

Windows/WSL mirrors the same overall pattern through [ensure_agent_ready(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:251) and its own sync-over-async helper in [backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:159).

This is why the chosen fix should extract/shared-split readiness logic rather than inventing a shell-side second implementation of VM/forwarder bootstrap.

### 4. The protocol itself is not the problem

The persistent-session protocol contract in [docs/internals/repl/persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md) remains valid. The problem is not:

- the `StartSession` frame,
- the `ready` handshake,
- cwd/env persistence,
- or shared-world semantics.

The problem is the readiness seam used immediately before opening that websocket.

## In Scope

- add an async readiness/bootstrap seam for persistent-session startup,
- route [build_ws_and_start_session_frame(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:730) through that async seam instead of `ctx.ensure_ready`,
- refactor macOS/Lima backend readiness internals so the async path and sync path share the same VM/forwarding/client/bootstrap rules,
- provide Windows/WSL parity for the same async readiness seam if that path can reach the same sync-over-async bridge,
- keep the existing sync `ensure_ready` closure for synchronous callers,
- keep websocket transport selection, shared-world request construction, and `ready` validation behavior unchanged,
- and add regression coverage and docs proving the split.

## Out Of Scope

This slice does not include:

- converting `WorldBackend` methods to async,
- changing LinuxLocalBackend semantics,
- redesigning `substrate world enable` or bootstrap CLI flows,
- replacing `PlatformWorldContext` wholesale,
- changing `ready.shared_world` or world-generation semantics,
- changing member-runtime placement or durable-session posture logic,
- or broadening unrelated public agent control surfaces.

## Main Code Seams

### Async caller seam

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:443)
  - creates the current-thread runtime and starts the world session during REPL startup
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:6506)
  - `open_world_session(...)`
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:6792)
  - `start_world_session(...)`
- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:125)
  - `ReplPersistentSessionClient::start_with(...)`
- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:730)
  - `build_ws_and_start_session_frame(...)`

### Shared sync readiness seam

- [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:51)
  - `PlatformWorldContext`
- [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:151)
  - macOS `ensure_ready` closure
- [crates/shell/src/execution/routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:100)
  - sync world bootstrap
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1327)
  - sync world execute request builder
- [crates/shell/src/execution/workspace_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs:557)
  - separate sync Tokio runtime used for workspace sync capabilities checks

### Backend-local readiness seam

- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:78)
  - `block_on_compat(...)`
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:274)
  - `ensure_agent_ready(...)`
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:511)
  - `WorldBackend::ensure_session(...)`
- [crates/world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:159)
  - `block_on(...)`
- [crates/world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:251)
  - `ensure_agent_ready(...)`

## Required Semantics

### 1. The async startup path must stop crossing the sync bridge

For persistent-session startup:

- `build_ws_and_start_session_frame(...)` must no longer call `(ctx.ensure_ready.as_ref())()?`,
- it must instead call an async readiness/bootstrap seam that is safe to await from the current-thread runtime,
- and that seam must return only after platform readiness is sufficiently established to open the websocket transport.

### 2. The sync startup path must remain supported

The existing sync world bootstrap and sync request-builder surfaces must continue to work:

- `PlatformWorldContext.ensure_ready` stays available,
- sync callers do not become async in this slice,
- and the repo does not require a Tokio runtime just to use those existing sync APIs.

### 3. Readiness logic must stay shared in substance

The async path and sync path must not drift into two separately maintained definitions of readiness.

At minimum, both paths must preserve the same platform-specific rules for:

- VM existence/running checks,
- forwarder establishment,
- agent endpoint discovery,
- capabilities verification,
- and fail-closed error behavior.

The preferred design is to share backend-local readiness internals and expose:

- one sync adapter for existing sync callers,
- and one async adapter for async callers.

### 4. The fix must preserve fail-closed behavior

The new async path must not weaken failure handling. It must still fail closed when:

- the VM cannot be started,
- forwarding cannot be established,
- the agent transport is unavailable,
- the capabilities probe fails,
- the socket path or TCP endpoint cannot be reached,
- or the websocket handshake fails.

This SOW is about runtime-correct readiness, not about making startup more permissive.

### 5. The fix must not require widening `WorldBackend`

In this slice, `WorldBackend` in [world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:263) remains synchronous.

That means:

- no `async_trait` conversion,
- no blanket migration of Linux/macOS/Windows backend interfaces,
- and no trait-level API churn outside the chosen async readiness seam.

### 6. Windows parity must be considered, even if macOS is the reproducer

The same sync-over-async helper pattern exists in WSL:

- `block_on(...)` uses `block_in_place(...)` in [backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:163).

This slice should either:

- migrate Windows persistent-session startup to the same async readiness seam at the same time, or
- explicitly prove that the Windows persistent-session path cannot hit the same issue and leave a documented follow-on if not fixed here.

The default expectation for this SOW is parity, not a macOS-only special case.

## Recommended Implementation Shape

### 1. Split platform-world readiness by caller shape, not by protocol

Do not move readiness logic into `async_repl.rs` or hardcode Lima-specific startup inside `world_persistent_session.rs`.

Instead:

- keep readiness owned by the platform-world/backend seam,
- add an async entrypoint specifically for async callers,
- and let persistent-session bootstrap consume that entrypoint.

Possible acceptable shapes include:

- adding an async helper function in `platform_world/mod.rs` that resolves/bootstraps an already-detected context for async use,
- adding backend-specific async readiness methods and dispatching them from platform-world code,
- or adding an async-ready transport/bootstrap descriptor returned from the platform-world layer.

The design must not depend on storing an async closure directly inside `PlatformWorldContext` unless that shape stays ergonomic and testable across platforms. A free async helper plus the existing sync closure is likely simpler.

### 2. Extract backend-local readiness internals so sync and async adapters share them

For macOS/Lima, refactor the internals around:

- [ensure_vm_running(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:117),
- [ensure_forwarding(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:260),
- [build_agent_client(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:485),
- and the capabilities verification part of [ensure_agent_ready(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:274)

so that:

- the sync adapter can still implement `ensure_session(...)`,
- and the async adapter can await readiness without routing through `block_on_compat(...)`.

The same principle applies to Windows/WSL readiness internals.

### 3. Persistent-session bootstrap should prepare transport asynchronously, then open websocket

`build_ws_and_start_session_frame(...)` should continue to own:

- env normalization,
- explicit socket override handling,
- transport selection for Unix/TCP/VSock-forwarded endpoints,
- websocket handshake,
- and `StartSession` frame creation.

But before those steps it should obtain readiness through the new async seam rather than the sync closure.

This preserves the existing protocol logic while fixing the readiness boundary.

### 4. Keep sync request builders on the old surface for now

The sync builders in `world_ops.rs` and similar surfaces should continue to use sync readiness in this slice. If they later become async, that should be a separate refactor.

### 5. Do not paper over the issue by only special-casing runtime flavor

A runtime-flavor guard inside `block_on_compat(...)` would be a useful tactical guardrail, but by itself it does not solve the design bug that an async caller is using the sync bridge.

This SOW is complete only when the async persistent-session caller has its own async readiness path.

It is acceptable to also harden `block_on_compat(...)` and the WSL equivalent as defense-in-depth, but that is not the primary success criterion.

## Concrete Work Breakdown

### 1. Add an async platform-world readiness/bootstrap seam

Primary anchors:

- [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:51)
- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:730)

Required outcome:

- there is one explicit async path for preparing a non-Linux forwarded world session transport for persistent-session startup,
- that path is callable from async REPL/bootstrap code without going through `ctx.ensure_ready`,
- and it composes cleanly with existing `pw::detect()` / `pw::get_context()` behavior.

### 2. Refactor macOS/Lima readiness internals behind sync + async adapters

Primary anchors:

- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:117)
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:260)
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:274)
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:511)

Required outcome:

- VM boot, forwarding, client construction, and capabilities verification are shared internal steps,
- the sync adapter still satisfies `ensure_session(...)`,
- the async adapter is await-safe under the REPL runtime,
- and persistent-session startup no longer depends on `block_on_compat(...)`.

### 3. Decide and implement Windows/WSL parity for the same seam

Primary anchors:

- [crates/world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:159)
- [crates/world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:251)

Required outcome:

- the repo either adopts the same async readiness split for WSL persistent-session bootstrap now,
- or proves with tests and comments that the current async path cannot hit the same sync-over-async bridge and documents the follow-on.

The preferred outcome for this SOW is to implement the parity split now.

### 4. Rewrite persistent-session bootstrap to consume the async seam

Primary anchors:

- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:730)
- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:766)

Required outcome:

- the async bootstrap helper gets the context,
- ensures platform readiness through the new async seam,
- then opens the chosen transport and websocket,
- with no call to `ctx.ensure_ready`.

### 5. Preserve sync readiness call sites unchanged in behavior

Primary anchors:

- [crates/shell/src/execution/routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:100)
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1327)
- [crates/shell/src/execution/workspace_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/workspace_cmd.rs:557)

Required outcome:

- sync bootstrap and sync request-builder paths continue to work,
- no caller is forced async accidentally,
- and the only behavior change they see is any intentional backend-local hardening that falls out of sharing readiness internals.

### 6. Add defense-in-depth runtime hardening

Primary anchors:

- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:78)
- [crates/world-windows-wsl/src/backend.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-windows-wsl/src/backend.rs:159)

Required outcome:

- backend-local sync-over-async helpers are hardened so they do not panic if another async caller accidentally routes through them in the future,
- but the main architecture still keeps persistent-session bootstrap off those helpers.

This is defense-in-depth, not the primary fix target.

### 7. Update docs to explain the split

Primary anchors:

- [docs/internals/repl/persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)

Required outcome:

- repo docs explain that persistent-session startup is an async bootstrap path with its own readiness adapter,
- docs stop implying that all world readiness always flows through the sync closure,
- and any platform-specific startup notes reflect the real Lima/WSL bootstrap split.

## Required Test Additions Or Tightening

### 1. REPL startup regression proving no current-thread panic

Required scenario:

- launch the interactive startup path under the same current-thread Tokio runtime shape used by [run_async_repl(...)](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:443),
- force the persistent-session bootstrap path to execute,
- and prove startup no longer panics when platform readiness is established.

The test does not need to use a real Lima VM if a targeted platform/backing stub can exercise the same async seam honestly.

### 2. Persistent-session bootstrap test proving the async seam is used

Required scenarios:

- async bootstrap succeeds through the new readiness path and reaches websocket connect,
- explicit socket overrides still bypass platform bootstrap as before,
- and bootstrap failures in readiness propagate as ordinary `Result` errors instead of Tokio runtime panics.

### 3. Sync readiness regression tests

Required scenarios:

- sync bootstrap in `routing/world.rs` still works,
- sync request-builders in `world_ops.rs` still work,
- and sharing internal readiness logic does not remove or weaken the existing sync surface.

### 4. macOS/Lima-focused backend tests

Required scenarios:

- forwarded readiness can be established through the async adapter,
- capabilities probe failures fail closed,
- sync adapter still satisfies `ensure_session(...)`,
- and caching/shared-owner behavior is not regressed by the readiness refactor.

Use or extend the existing macOS test seams around the `session_setup_override` path in [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:44) rather than inventing a less honest test seam.

### 5. Windows/WSL parity tests if included

Required scenarios:

- async bootstrap does not route through the sync `block_on(...)` helper,
- warm-script and capabilities failure behavior remains fail closed,
- and sync `ensure_session(...)` behavior is preserved.

### 6. Manual smoke expectation

At minimum, the manual repro used during investigation must stop failing on macOS:

```bash
script -q /dev/null zsh -lc 'RUST_BACKTRACE=1 ~/.substrate/bin/substrate'
```

Expected outcome:

- no `can call blocking only when running on the multi-threaded runtime` panic,
- REPL reaches normal startup or a normal fail-closed readiness error,
- and any failure is a plain user-facing readiness failure rather than a Tokio runtime invariant panic.

## Acceptance Criteria

### A. Async startup correctness

1. Persistent-session startup no longer calls `ctx.ensure_ready` from [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:775).
2. The REPL current-thread runtime can start a world session on macOS/Lima without the runtime panic observed during investigation.
3. Readiness/connectivity failures surface as normal `Result` failures rather than panic unwinds.

### B. Sync surface preservation

1. `PlatformWorldContext.ensure_ready` remains available for sync callers.
2. Sync bootstrap in [routing/world.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/world.rs:100) still behaves as before.
3. Sync request-builders in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1316) still behave as before.

### C. Backend-local readiness sharing

1. macOS/Lima readiness logic is not duplicated between shell and backend layers.
2. Sync and async readiness adapters share the same platform rules for VM/forwarding/client/capabilities setup.
3. Defense-in-depth hardening prevents future runtime-flavor panics even if another async caller accidentally hits the sync bridge.

### D. Windows/WSL parity decision

1. The same readiness-split decision is applied to WSL, or
2. the repo explicitly documents and proves why WSL cannot hit the same failure path and records the follow-on if parity is deferred.

### E. Documentation

1. [docs/internals/repl/persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md) reflects the async bootstrap split.
2. Any affected world/bootstrap docs describe the real caller-shape split instead of implying one universal sync readiness path.

## Validation Checklist

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p substrate-shell -- --nocapture`
- `cargo test -p world-mac-lima -- --nocapture`
- `cargo test -p world-windows-wsl -- --nocapture`
- targeted REPL/persistent-session tests covering startup and restart flows
- macOS manual repro smoke under a pseudo-TTY
- if Windows parity lands in this slice, run the relevant WSL smoke coverage too

## Important Related Docs And Prior Slices

- [21-macos-lima-shared-owner-and-member-runtime-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/21-macos-lima-shared-owner-and-member-runtime-parity.md)
- [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)
- [docs/internals/repl/persistent_session.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/repl/persistent_session.md)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md)

## Explicit Non-Decision

This SOW intentionally does not answer whether `WorldBackend` should eventually become async. It only freezes the medium-scope near-term answer:

- async persistent-session bootstrap gets its own async readiness seam now,
- sync world bootstrap and sync request-builders keep their sync seam now,
- and any full backend-trait migration is deferred to a later, separately justified slice.
