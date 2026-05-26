# Persistent Session REPL: Execution Notes

Substrate’s world-first REPL uses the world-service WebSocket `/v1/stream` “persistent session”
protocol (v1) to execute interactive commands while persisting a small amount of state across
commands (not a long-lived login shell).

This doc is intentionally implementation-focused and is the stable internal reference for the
world-first persistent-session REPL behavior that was previously documented only in planning and
archived pack artifacts.

## Mental Model

- A REPL “session” is long-lived (WebSocket stays up), but each command is executed as a separate
  `bash -c "<program>"` process inside the world.
- After each command completes, world-service captures:
  - physical cwd (`pwd -P` equivalent), and
  - exported environment variables
  and persists them for the next command.

This yields familiar interactive behavior (cwd/env continuity) without keeping a single shell
process alive.

## Caging vs Uncaged (Anchor Guard)

When caging is enabled (`SUBSTRATE_CAGED=1` and anchor mode is not `follow-cwd`), Substrate enforces
an “anchor guard” that prevents leaving the configured anchor root (usually the workspace/project
root).

The guard is implemented as shell code that:
- defines a `cd()` function wrapper, and
- immediately runs a `cd .`-like check to snap back into the anchor if already outside.

### Important: Where to Inject the Guard

The guard must be evaluated in the *same shell process* that interprets the user’s program text.

Gotcha (fixed in `world-service` persistent-session exec):
- If you wrap the *outer* command (e.g. the “inner command” that does `exec /bin/bash -c "$PROGRAM"`),
  the `exec` replaces the shell process, and the guard’s `cd()` override is lost.
- Correct approach: wrap the *program string* itself (the thing passed to `bash -c ...`) so the guard
  runs in the shell that executes the user’s command and any `cd` operations within it.

Implication:
- Caged sessions reliably prevent `cd ..` (and similar) from escaping the anchor.
- Uncaged sessions (`SUBSTRATE_CAGED=0`) may persist cwd outside the workspace, which can trigger
  host-side policy/workspace drift handling on the next command if the effective workspace root or
  policy snapshot differs for that new cwd.

## Drift Restarts (Host-Side)

The host REPL may restart the world session when the effective policy snapshot hash or workspace
root for the current in-world cwd changes (“snapshot/workspace drift”).

When uncaged traversal moves outside the workspace:
- a drift restart can be expected (new cwd ⇒ potentially different workspace root and policy), and
- the REPL should attempt best-effort cwd continuity on restart (see DR-17 in the project docs).

## Host Readiness Split

Persistent-session startup now preserves a caller-shape split between sync bootstrap code and the
async REPL startup path.

- Sync callers still use `PlatformWorldContext.ensure_ready()`. That remains the shell-facing sync
  bridge for request builders and bootstrap flows that are not already inside async startup.
- The macOS async persistent-session startup path no longer calls that sync bridge. When
  `build_ws_and_start_session_frame(...)` needs to bring the Lima-backed world path online without a
  `SUBSTRATE_WORLD_SOCKET` override, it now awaits
  `PlatformWorldContext.ensure_persistent_session_ready_async()`, which delegates to backend-owned
  readiness.
- On macOS, the backend-owned async path is `MacLimaBackend::ensure_persistent_session_ready_async`.
  The shell does not duplicate VM startup, forwarding, or readiness verification logic.
- `SUBSTRATE_WORLD_SOCKET` keeps exact bypass semantics. If callers override the socket path, the
  persistent-session client connects directly to that socket and does not invoke platform readiness
  helpers first.
- The Windows/WSL backend now mirrors the same split internally for parity, but this slice does not
  ship a Windows persistent-session shell caller. The parity work only keeps backend-owned
  readiness helpers aligned with the macOS/Linux contract.
