# State Machine — World-First REPL (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`

It defines the observable behavior of the interactive REPL, including routing rules, lifecycle, and signal handling.

## High-Level Model
The interactive REPL has two execution origins:
- **World** (default): execute inside the persistent world session.
- **Host** (explicit): execute only when the input line begins with `:host ` and host escape is enabled.

The REPL is line-oriented:
- Each submitted REPL line is treated as one shell command line.
- Multiline continuations (PS2), job control, and backgrounding are not supported in the default REPL mode.

Full terminal-interactive workloads are handled via the explicit `:pty <cmd>` directive (see “Directives”).

## Persistent State
Substrate maintains the following state for the lifetime of the REPL:
- `world_session`: active/inactive + metadata (session nonce, policy snapshot hash, world id if available).
- `world_cwd`: last known in-world cwd (from the most recent valid command boundary marker).
- `last_exit`: last command exit code (world or host).
- `host_escape_enabled`: boolean (default `false`).
- `host_cwd`: host-only working directory for `:host` commands.

`world_cwd` and `host_cwd` are independent by design.

## Startup
On REPL startup:
1) Resolve whether world execution is enabled (based on existing `--world/--no-world` and config).
2) If world execution is enabled:
   - Start the persistent world session and wait for readiness (marker `seq=0`).
   - Initialize `world_cwd` from the readiness marker.
3) If world execution is disabled:
   - No world session exists; all commands execute on host.

If world execution is enabled but the world session cannot be started, the REPL MUST exit with an error (no silent host fallback).

## Prompt
When waiting for input, the prompt MUST be derived from `world_cwd` when world execution is enabled, otherwise from `host_cwd`.

The prompt SHOULD visually distinguish world vs host contexts. At minimum:
- world-first prompt includes `world_cwd`,
- host-only prompt includes `host_cwd`.

## Main Loop States

### State: `Idle`
The REPL is waiting for the next input line via the line editor (Reedline).

Input handling:
- `exit` / `quit` (exact match after trimming): transition to `ShuttingDown`.
- Empty/whitespace-only line: stay in `Idle`.
- Line begins with `:host`:
  - If `host_escape_enabled=false`: print a clear error (“host escape not enabled”) and stay in `Idle`.
  - If `host_escape_enabled=true`: transition to `ExecutingHost(line_without_prefix)`.
- Line begins with `:pty `:
  - If world execution is disabled: print a clear error (“:pty requires world execution”) and stay in `Idle`.
  - If world execution is enabled: transition to `ExecutingPty(cmd_string)`.
- Otherwise:
  - If world execution is enabled: transition to `ExecutingWorld(line)`.
  - If world execution is disabled: transition to `ExecutingHost(line)`.

### State: `ExecutingWorld(line)`
The REPL executes one command line inside the persistent world session.

Actions:
1) Recompute the effective policy snapshot hash for `world_cwd`.
2) If the snapshot hash differs from the current session’s snapshot hash:
   - Tear down the current world session.
   - Start a new world session and wait for readiness (`seq=0`).
     - Substrate SHOULD request the new session start with `cwd` set to the previous session's `world_cwd` to preserve the
       operator's location.
     - If that `cwd` is rejected/invalid under the new session, Substrate MUST start in the new session's resolved
       project/root directory and MUST report the cwd change.
   - Update `world_cwd` from readiness.
3) Submit the user line to the session shell using the protocol in `PROTOCOL.md`:
   - host assigns the next `seq` and a per-command token,
   - executes the line with stdin redirected to `/dev/null` (to prevent stdin-consuming commands from consuming the marker line),
   - consumes `stdout` until the accepted marker for `seq` arrives.

Outputs:
- Non-marker output is streamed to the user.
- Marker output is filtered.

Completion:
- On accepted marker:
  - update `world_cwd` from `cwd_b64`,
  - set `last_exit` to `exit`,
  - transition back to `Idle`.

Errors:
- On protocol error, world session exit, or WebSocket close:
  - print a clear error,
  - transition to `Error` (fatal) and then `ShuttingDown`.

### State: `ExecutingHost(line)`
The REPL executes one command line on the host.

Working directory:
- The command MUST execute with `host_cwd` as its working directory.
- `:host cd <path>` MUST update `host_cwd` on success (it MUST NOT affect `world_cwd`).

Completion:
- Capture the host command exit code into `last_exit`.
- Transition back to `Idle`.

### State: `ExecutingPty(cmd)`
Runs a command in a one-shot in-world PTY stream (not the persistent session).

Contract:
- Intended for full-screen TUIs and programs that require continuous stdin/tty interaction.
- Does not share the persistent session’s shell state (`world_cwd`/exports) beyond the current effective `world_cwd` used as the starting cwd for the PTY command.
- Policy snapshot MUST be recomputed immediately before starting `:pty` using the current `world_cwd` (same rule as `ExecutingWorld`), so `:pty` never runs under a stale policy snapshot.
- The PTY command SHOULD start with `cwd` set to the current `world_cwd`. If that `cwd` is invalid/rejected for the computed snapshot, Substrate MUST start in the resolved project/root directory and MUST report the cwd change.

Completion:
- On PTY exit, return to `Idle` without modifying `world_cwd` unless explicitly specified by the PTY implementation (default: unchanged).

### State: `ShuttingDown`
Actions:
- If a world session exists: close it and release resources.
- Exit the REPL process with code `0` for normal user-initiated exit, otherwise a non-zero code for startup/protocol failures (per `EXIT_CODE_TAXONOMY.md`).

### State: `Error` (fatal)
Any entry into `Error` MUST be followed by `ShuttingDown` (no continuation in a degraded mode).

## Signals and Terminal Events

### `Ctrl+C` (SIGINT)
- In `Idle`: cancels the current line edit (Reedline standard behavior) and stays in `Idle`.
- In `ExecutingWorld`: Substrate MUST forward `SIGINT` to the PTY child via the world-agent `signal` message, then continue waiting for the command boundary marker.
- In `ExecutingHost`: Substrate SHOULD forward `SIGINT` to the running host child process.

### `Ctrl+D` (EOF)
- In `Idle`: treated as `exit` → `ShuttingDown`.
- In other states: ignored (the REPL does not forward EOF into the persistent world session).

### Terminal resize
- If a world session exists: Substrate MUST forward resize events (`cols`, `rows`) to world-agent.

## Observability Requirements
For each executed command line, Substrate MUST emit a trace command span that records at minimum:
- `execution_origin`: `world` or `host`
- `exit_code`
- `cwd`: `world_cwd` or `host_cwd` as appropriate
- `world_id` and policy snapshot hash when world execution is used

The persistent session’s internal bootstrap commands MUST NOT be recorded as user commands.
