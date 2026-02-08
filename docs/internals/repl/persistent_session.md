# Persistent Session REPL: Execution Notes

Substrate’s world-first REPL uses the world-agent WebSocket `/v1/stream` “persistent session”
protocol (v1) to execute interactive commands while persisting a small amount of state across
commands (not a long-lived login shell).

This doc is intentionally implementation-focused. For the authoritative project plan/specs, see
`docs/project_management/_archived/world-first-repl-persistent-pty/`.

## Mental Model

- A REPL “session” is long-lived (WebSocket stays up), but each command is executed as a separate
  `bash -c "<program>"` process inside the world.
- After each command completes, world-agent captures:
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

Gotcha (fixed in `world-agent` persistent-session exec):
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

