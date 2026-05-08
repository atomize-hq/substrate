# SOW: macOS/Lima shared-owner and member-runtime parity

Status: implementation-oriented draft. This SOW covers item `#1` from the recommended v1 runtime slice in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:175): bring macOS/Lima onto the same shared-owner/member-runtime path as Linux before widening scope to a different non-Linux backend model. The goal is to extend the already-landed Linux contract across the existing Lima forwarded backend seam, not to invent a different orchestration model. This remains a Linux-first / REPL-first parity slice; it does not broaden non-REPL caller surfaces, but it does include the broader backend-trait and backend-handle work needed to keep macOS/Lima honest with Linux instead of preserving a second-class seam indefinitely.

## Objective

Land one honest macOS/Lima path where:

- the shell remains the owner of the host orchestrator runtime and the authoritative writer of orchestration session and participant state,
- shared-world ownership continues to use the existing `SharedWorldOwnerSpec` / `SharedWorldBindingSnapshot` contract on the REPL persistent-session seam,
- world-scoped member launch, reuse, targeted follow-up turns, and cancel continue to use the existing Linux `member_dispatch`, `/v1/member_turn/stream`, and `/v1/execute/cancel` contract,
- and the macOS host reaches that same contract through the forwarded Lima guest `world-agent` rather than through a lossy backend trait seam or a host-local fallback.

The required outcome is parity with the Linux shared-owner/member-runtime model. The required outcome is not “a mac-specific orchestration model.”

## Implementation Shape Choice

This SOW chooses the broader parity slice, landed in layers but scoped as one coherent slice.

### Chosen shape: broader parity slice with layered landing

In scope:

- macOS REPL persistent-session shared-owner parity over the forwarded `/v1/stream` transport,
- macOS shell parity for world-member bootstrap, follow-up turns, and cancel over the forwarded guest `world-agent` transport,
- shell-side `#[cfg(target_os = "linux")]` parity work needed to make those paths compile and run on macOS,
- expanding `crates/world-api` so backend-level execution and handles can carry the Linux orchestration-sensitive ownership and dispatch contract instead of silently dropping it,
- teaching the generic `WorldBackend::exec(ExecRequest)` path and related backend APIs to preserve Linux orchestration-sensitive request fields end to end,
- making `WorldHandle.shared_binding` parity part of the required backend contract for the Lima backend,
- live Lima validation and doc updates.

### Landing posture

This work should still land in layers for risk control, but those layers belong to one SOW and one parity target:

1. remove macOS host-side shared-owner pre-bootstrap rejection and enable forwarded persistent-session shared-owner proof,
2. widen shell-side member-runtime code paths beyond Linux-only cfg gates,
3. expand `crates/world-api` / backend execution contracts so `world-mac-lima` can preserve the same ownership and dispatch information Linux already relies on,
4. bring backend-level `WorldHandle.shared_binding` and execution-field parity up to the same standard,
5. validate the whole stack together with reproducible Lima orchestration smoke.

## Current Repo Truth

### Linux is already the baseline contract

- The gap matrix already says the intended v1 slice is to keep the existing shell-owned orchestrator runtime, reuse shared-world owner binding and world-generation invalidation, keep Linux world-owned member launch over the existing host↔world seam, and bring macOS/Lima onto that same path before widening scope elsewhere in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:171).
- The shell already requests explicit shared-world ownership when opening and replacing REPL world sessions:
  - attach/create in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:5804)
  - replace-expected-generation in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:5575)
- The persistent-session protocol already validates echoed shared-world proof fail-closed, including generation advancement on replacement, in [repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308).
- Linux world-member bootstrap already builds a typed `MemberDispatchTransportRequest` carrying `orchestration_session_id`, participant lineage, exact `backend_id`, `world_id`, and `world_generation`, then sends it over `/v1/execute/stream` in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3541) and [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:203).
- Linux `world-agent` already validates authoritative shared-world binding for `member_dispatch`, resolves placement context, launches the retained member runtime, exposes `/v1/member_turn/stream`, and delivers `/v1/execute/cancel` in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1238), [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1471), and [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1488).
- The retained world-member identity model is already explicit in `world-agent`: retained members are keyed by `(orchestration_session_id, world_generation, backend_id)` and follow-up turns validate exact orchestration/backend/world identity in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:97), [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869), and [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:1014).

### The macOS/Lima forwarded transport is already partly there

- macOS already has a forwarded typed `member_dispatch` request builder in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1369). The typed request shape itself is not missing on the host.
- Inference from current code: because Lima forwards into a Linux guest `world-agent`, the guest-side Linux routes are already the intended execution target. The missing work is host-side parity and forwarded-seam honesty, not a second mac-native `world-agent` design.

### The real macOS shared-owner blocker is host-side pre-bootstrap rejection

- The REPL builds `SharedWorldOwnerSpec`, but macOS persistent-session bootstrap rejects explicit shared-owner requests before guest bootstrap in [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:723).
- That rejection goes through the common non-Linux guard in [platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:61).
- `docs/WORLD.md` already states the same truth twice:
  - non-Linux shared-owner requests are rejected before bootstrap in [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:86)
  - and again in the transport/API section in [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:286)

### The shell member-runtime path is still Linux-compiled

- `async_repl` imports `AgentClient`, `ExecuteCancelRequestV1`, `ExecuteStreamFrame`, `MemberTurnSubmitRequestV1`, and the member-dispatch helper surface only on Linux in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:11) and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:71).
- Transport request construction for member bootstrap is Linux-only in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3540).
- Non-Linux member launch still hard-errors in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4297).
- Non-Linux world-targeted follow-up turns still hard-error in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4683).
- Bootstrap cancel and remote retained-control shutdown are still Linux-only in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3592), [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3988), and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4776).

### The backend trait seam is lossy today and is part of this slice’s fix target

- `crates/world-api` `WorldBackend::exec` takes `ExecRequest`, and that type carries only `cmd`, `cwd`, `env`, `pty`, and optional `span_id` in [world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:139). It does not carry `shared_world` or `member_dispatch`.
- That is why `world-mac-lima` converts ordinary backend exec requests to agent requests with `shared_world: None` and `member_dispatch: None` in [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:301).
- `world-mac-lima` also returns `WorldHandle { shared_binding: None }` in [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:388).
- If parity with Linux is the real target, this lossy backend seam cannot remain a permanent exception. This draft therefore treats `crates/world-api`, `WorldBackend::exec`, `ExecRequest`, and `WorldHandle.shared_binding` expansion as in-scope work rather than a separate follow-on.

## Scope

- Remove the host-side pre-bootstrap rejection for macOS REPL persistent-session shared-owner requests and route them through the forwarded guest `world-agent` transport.
- Preserve the existing `SharedWorldOwnerSpec` request and `ready.shared_world` proof contract on the persistent-session seam.
- Extend shell-side member-runtime code paths so the existing Linux `member_dispatch`, `/v1/member_turn/stream`, and `/v1/execute/cancel` flow compiles and runs on macOS.
- Reuse the existing forwarded typed request builders in `world_ops.rs` rather than inventing a second request model.
- Expand `crates/world-api` so backend-level execution requests and results can preserve Linux orchestration-sensitive ownership and member-dispatch data instead of silently dropping it.
- Update the generic `WorldBackend` execution path and the Lima backend so `shared_world`, `member_dispatch`, and authoritative shared binding can survive the backend seam.
- Make backend-level `WorldHandle.shared_binding` parity a required part of the Lima contract, not an optional nice-to-have.
- Keep member identity, reuse, replacement, and fail-closed validation exactly aligned with the existing guest-Linux `world-agent` contract.
- Add reproducible macOS/Lima orchestration smoke coverage that explicitly exercises attach/create, replacement, targeted turns, and member cancel.
- Update docs, including [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md), to stop claiming non-Linux rejection once this slice lands.

## Non-Goals

- Replacing the shell-owned orchestrator runtime.
- Redesigning `substrate -c` or creating a new one-shot agent caller surface.
- Broadening or redesigning the public `substrate agent start|resume|fork|stop` contract from [19-public-agent-control-surfaces.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/19-public-agent-control-surfaces.md:3).
- Weakening exact `backend_id`, `orchestration_session_id`, `world_id`, or `world_generation` checks.
- Adding Windows/WSL parity in the same slice.
- Introducing host-local fallback for a selected world-scoped member when forwarded world dispatch is unavailable.

## Main Code Seams

### Persistent-session shared-owner proof seam

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:5804)
  - shell-owned attach/create and replace-expected-generation request construction
- [crates/shell/src/execution/repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308)
  - fail-closed validation for echoed `ready.shared_world`
- [crates/shell/src/execution/routing/dispatch/world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:723)
  - actual macOS host-side blocker: rejects explicit shared-owner request before guest bootstrap
- [crates/shell/src/execution/platform_world/mod.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform_world/mod.rs:61)
  - common non-Linux shared-owner rejection helper
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:61)
  - documents the current Linux-first shared-owner rule and non-Linux rejection posture

### Member dispatch / member turn / cancel seam

- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:11)
  - Linux-only imports and compile gates that currently block macOS parity
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3540)
  - Linux-only member transport request construction
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4297)
  - non-Linux member launch hard-error
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4683)
  - non-Linux follow-up hard-error
- [crates/shell/src/repl/async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3592)
  - Linux-only bootstrap cancel support
- [crates/shell/src/execution/routing/dispatch/world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1369)
  - existing macOS forwarded typed `member_dispatch` request builder
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1238)
  - guest-Linux `member_dispatch` entrypoint and authoritative binding validation
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1471)
  - guest-Linux `/v1/member_turn/stream`
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1488)
  - guest-Linux `/v1/execute/cancel`
- [crates/world-agent/src/member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869)
  - guest-Linux exact identity validation for retained-member follow-up turns

### Backend trait / backend handle parity seam

- [crates/world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:139)
  - current trait/type boundary that does not carry orchestration-sensitive request fields and must be widened
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:301)
  - ordinary backend exec conversion that currently drops `shared_world` and `member_dispatch`
- [crates/world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:388)
  - `WorldHandle.shared_binding` remains `None` today and must gain parity

## Required Semantics

### 1. Linux remains the contract source of truth

macOS/Lima parity must match the Linux request/response contract and failure posture already in the repo. This slice must not create a second orchestration truth.

### 2. Shared-world ownership stays explicit and authoritative on the persistent-session seam

Where an orchestration session is opening or replacing a shared world through the REPL persistent-session flow, the request must continue to use `SharedWorldOwnerSpec`, the reply must continue to surface `ready.shared_world`, and missing or malformed proof must continue to fail closed.

### 3. No host-side pre-bootstrap rejection on macOS for this supported path

The current non-Linux rejection path must be removed for the supported macOS/Lima persistent-session shared-owner flow. After that change, macOS must either complete the forwarded shared-owner handshake or fail closed based on the guest response. It must not be rejected before bootstrap solely because the host OS is macOS.

### 4. No host-local fallback for world-scoped members

If the selected runtime is world-scoped and the Lima-forwarded guest contract cannot be established, startup and follow-up turns must fail closed. The shell must not degrade to a host-local retained member runtime.

### 5. Exact identity and reuse rules must stay the same

macOS/Lima world-member reuse and follow-up turns must remain keyed by the same identity tuple Linux already enforces:

- `orchestration_session_id`
- retained participant lineage
- exact `backend_id`
- authoritative `world_id`
- authoritative `world_generation`

### 6. Targeted follow-up turns must reuse the existing route

macOS/Lima follow-up turns must go through the existing typed `/v1/member_turn/stream` path and retained `internal.uaa_session_id` flow. This slice must not invent a different resume transport for macOS.

### 7. Cancel must remain transport-owned in the guest

Bootstrap cancel and submitted-turn cancel must continue to route through `/v1/execute/cancel` and the retained guest-side control owner. This slice must not move world-member cancel ownership back into the shell.

### 8. Replacement semantics must remain shell-owned and fail-closed

On world rollover, the shell must continue to own replacement intent, persisted authoritative binding, stale-member invalidation, and replacement-participant lineage. macOS/Lima must match Linux on:

- `ReplaceExpectedGeneration` request semantics,
- world-generation advancement,
- invalidation of stale retained members,
- exact-backend relaunch into the replacement world,
- and rollback/fail-closed behavior if the replacement proof or replacement launch cannot be proven.

### 9. Backend-level parity must stop dropping Linux contract fields

The generic backend execution seam must no longer silently erase orchestration-sensitive Linux fields on macOS/Lima. If backend-level execution or handle surfaces participate in a shared-owner or member-runtime path, they must carry:

- explicit shared-world ownership input,
- member-dispatch input,
- and authoritative shared binding output

with the same fail-closed posture Linux already expects.

## Recommended Implementation Shape

### 1. Use the forwarded-client path first, but remove the lossy backend exception in the same slice

Route shared-owner REPL bootstrap and member-runtime orchestration through the existing forwarded host→guest client paths that already speak the Linux guest `world-agent` contract. But do not leave the generic backend seam as a permanent parity hole; widen `crates/world-api` and the Lima backend so backend-level surfaces can preserve the same ownership and dispatch data instead of hardcoding `None`.

### 2. Remove the macOS persistent-session shared-owner rejection

Replace the guard at [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:736) for the Lima-backed path so macOS can send `StartSession { shared_world: ... }` over the forwarded transport and consume `ready.shared_world` through the existing fail-closed validator in [repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308).

### 3. Extend shell-side member-runtime code past Linux-only cfg gates

Make the shell’s world-member orchestration path compile and run on macOS by widening the currently Linux-only import, request-building, launch, follow-up, and cancel code in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:11), [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:3540), [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4297), and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4683).

### 4. Reuse the existing macOS forwarded typed request builder

The macOS forwarded `member_dispatch` request builder already exists in [world_ops.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs:1369). Reuse it rather than introducing a second macOS request shape.

### 5. Reuse guest-side Linux `world-agent` semantics unchanged

The guest Linux `world-agent` already validates binding and owns retained member runtime behavior in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1238), [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1471), [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1488), and [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:869). The macOS/Lima slice should consume that contract, not fork it.

### 6. Expand backend-level world contracts to preserve Linux orchestration semantics

Update [world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:139) and downstream backend implementations so the generic backend path can carry:

- explicit shared-world ownership input,
- member-dispatch input,
- and authoritative shared binding output

without requiring Linux-only side channels or lossy conversion at the Lima seam.

## Acceptance Criteria

### A. Persistent-session shared-owner proof / restart proof

1. On a macOS host with a provisioned Lima guest, REPL world startup with an orchestration context sends `StartSession { shared_world: Some(AttachOrCreate) }` over the forwarded transport instead of failing at the host-side non-Linux rejection gate.
2. The macOS REPL consumes `ready.shared_world` through the existing validator and persists authoritative `world_id` / `world_generation` into `WorldSession` and shell runtime state.
3. macOS world restart sends `ReplaceExpectedGeneration` over the same persistent-session seam and accepts only a reply whose echoed proof advances `world_generation`.
4. Missing, malformed, mismatched, or non-`active` `ready.shared_world` proof still fails closed on macOS exactly as it does today on Linux.

### B. Member dispatch / member-turn / cancel parity

1. macOS/Lima world-member lazy launch no longer errors with “supported on Linux only” and instead sends the same typed `member_dispatch` payload Linux already uses.
2. A retained macOS/Lima world member can accept targeted REPL follow-up turns through `/v1/member_turn/stream` using exact `backend_id`, `world_id`, and `world_generation`.
3. macOS/Lima bootstrap cancel reaches `/v1/execute/cancel` for a forwarded retained member bootstrap span.
4. macOS/Lima submitted-turn cancel reaches `/v1/execute/cancel` for a forwarded retained member submitted-turn span.
5. Stale or mismatched `orchestration_session_id`, `backend_id`, `world_id`, or `world_generation` still fail closed on macOS/Lima exactly as they do on Linux.
6. World rollover on macOS/Lima invalidates stale retained members and relaunches the exact replacement backend into the new authoritative world generation.
7. No macOS/Lima world-sensitive path silently falls back to host-local member runtime ownership.

### C. Backend trait parity

1. `crates/world-api` grows a backend-level request/handle contract that can preserve the Linux shared-owner/member-dispatch shape rather than forcing `world-mac-lima` to zero those fields.
2. `WorldBackend::exec` callers that participate in orchestration-sensitive flows can carry the required shared-owner/member-dispatch data without an out-of-band workaround.
3. `world-mac-lima` no longer hardcodes `shared_world: None` and `member_dispatch: None` for the relevant parity paths.
4. `WorldHandle.shared_binding` is populated for the Lima-backed parity path when explicit shared-owner mode is active.
5. Docs and tests do not describe backend-level Linux parity as optional once this slice lands.

## Testing Expectations

### Required automated coverage

- Add or extend shell tests around [world_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_persistent_session.rs:723), [repl_persistent_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/repl_persistent_session.rs:308), and [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4297) to prove:
  - macOS persistent-session shared-owner requests are forwarded instead of pre-rejected,
  - replacement proof must advance generation,
  - member launch/follow-up/cancel no longer fail at non-Linux cfg stubs.
- Add or extend backend-level tests around [world-api/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-api/src/lib.rs:139) and [world-mac-lima/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-mac-lima/src/lib.rs:301) to prove:
  - backend request/handle types preserve shared-owner/member-dispatch semantics,
  - Lima no longer zeroes those fields for parity paths,
  - `WorldHandle.shared_binding` is populated when explicit shared-owner mode is active.
- Keep `world-agent` member-runtime tests in [member_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/member_runtime.rs:1071) and service tests around [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1238) and [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1471) passing without relaxing identity validation. These are host-side regression checks; authoritative macOS/Lima behavior must still be proven by the Lima-backed orchestration smoke below.
- Run at minimum:
  - `cargo test -p world-agent member_runtime`
  - `cargo test -p shell`
  - `cargo test -p world-api`
  - `cargo test -p world-mac-lima`
  - `cargo test --workspace -- --nocapture`

### Required guest-Linux behavior validation

The guest `world-agent` behavior that macOS consumes must be validated in the guest-backed path, not assumed from host-only unit tests. At minimum this means exercising the forwarded macOS path against the real Lima guest `world-agent` service that owns:

- shared-owner persistent-session replies,
- `member_dispatch`,
- `/v1/member_turn/stream`,
- and `/v1/execute/cancel`.

### Required reproducible macOS/Lima orchestration smoke

Existing [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh) is not sufficient because [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:196) only claims non-PTY, PTY, and replay coverage today.

This slice requires one new reproducible harness: [scripts/mac/orchestration-smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/orchestration-smoke.sh).

It must explicitly cover:

1. shared-owner attach/create over the persistent-session seam,
2. shared-owner replacement with generation advancement,
3. world-member lazy launch through `member_dispatch`,
4. targeted `::<backend_id> <prompt>` follow-up turn through `/v1/member_turn/stream`,
5. member bootstrap or submitted-turn cancel through `/v1/execute/cancel`,
6. and at least one fail-closed mismatch case for stale `world_generation` or wrong `backend_id`.

The smoke must be runnable from the current checkout on a macOS host with Lima provisioned, without relying on ad hoc manual steps.

## Validation Playback

The implementation is not done until one reproducible macOS/Lima orchestration playback proves the full seam end to end.

1. Warm the guest from the current checkout with [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh).
2. Start a REPL orchestration session on the macOS host and enter a world-backed path that requests shared-world attach/create.
3. Verify the request is not rejected before bootstrap and that the returned ready state carries authoritative `ready.shared_world`.
4. Trigger lazy world-member launch for one exact backend and verify the retained member is keyed to the authoritative generation.
5. Submit one targeted follow-up turn as `::<backend_id> <prompt>` and verify it routes through the retained world member rather than spawning a new host-local runtime.
6. Deliver cancel once during bootstrap or a submitted turn and verify the forwarded guest-side owner receives it.
7. Force world restart or drift so replacement uses `ReplaceExpectedGeneration`, then verify generation advancement, stale-member invalidation, and exact-backend relaunch.
8. Repeat a targeted follow-up turn after rollover and verify stale-generation submission is rejected while the replacement generation succeeds.
9. Exercise one failure case where shared-world proof or generation continuity cannot be established and verify the macOS host fails closed instead of silently downgrading behavior.

## Documentation Expectations

- Update [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md) so the macOS/Lima row no longer claims the parity gap once this slice lands.
- Update [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md) to remove the current statement that macOS rejects explicit shared-owner requests before bootstrap in [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:86) and [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:286), and to document the new orchestration validation harness.
- If any caller-facing help, smoke instructions, or control-surface docs mention Linux-only world-member follow-up semantics, update them to reflect the new Lima-backed parity.
- Update any backend/world contract docs that describe `WorldBackend::exec` / `ExecRequest` / `WorldHandle.shared_binding` as insufficient for Linux-parity orchestration semantics, because this slice now includes that rewrite.

## Done Means

This slice is done when a macOS host can use the forwarded Lima guest path to participate in the same shared-owner persistent-session proof flow and the same guest-owned retained member-runtime flow that Linux already uses, and when the backend-level world contract no longer forces Lima to zero the ownership and dispatch fields Linux depends on. Completion requires reproducible attach/create, replacement, targeted turns, cancel, fail-closed mismatch validation, and backend-level shared-binding / request-field parity. Until then, the repo should continue to describe world-sensitive agent orchestration as Linux-first.
