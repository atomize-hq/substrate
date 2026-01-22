# State Machine — World-First REPL (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`

It defines the observable behavior of the interactive REPL, including routing rules, lifecycle, and signal handling.

## High-Level Model
The interactive REPL has two execution origins:
- **World** (default): execute inside the persistent world session.
- **Host** (explicit): execute only when the input line begins with `:host ` and host escape is enabled.

The REPL is line-editor driven but supports auto-PTY for interactive programs:
- Each submitted REPL line is treated as one “command submission” to the session shell.
- Substrate selects a per-command I/O mode:
  - **Line mode** for non-interactive commands (stdin not forwarded; redirected to `/dev/null`).
  - **PTY passthrough mode** for interactive commands/TUIs (stdin forwarded; raw terminal mode).
- PTY passthrough mode is selected automatically by Substrate’s existing “needs PTY” heuristic, or explicitly via the `:pty ` prefix (force PTY).

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
  - If world execution is enabled: transition to `ExecutingWorldPty(line_without_prefix)`.
  - If world execution is disabled: transition to `ExecutingHostPty(line_without_prefix)`.
- Otherwise:
  - If world execution is enabled:
    - If the command is classified as “needs PTY”: transition to `ExecutingWorldPty(line)`.
    - Otherwise: transition to `ExecutingWorldLine(line)`.
  - If world execution is disabled:
    - If the command is classified as “needs PTY”: transition to `ExecutingHostPty(line)`.
    - Otherwise: transition to `ExecutingHost(line)`.

### State: `ExecutingWorldLine(line)`
The REPL executes one command line inside the persistent world session in **line mode** (stdin redirected to `/dev/null`, no keystroke forwarding).

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
   - submits the line using the brace-framed line-mode submission (`</dev/null` on the closing line),
   - consumes `stdout` until the accepted marker for `(seq, token)` arrives.

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

Edge case:
- If a command is misclassified into line mode but attempts to read from the controlling TTY (e.g., via `/dev/tty`), it may block.
  The operator can abort with `Ctrl+C` and retry with `:pty <cmd>` to force PTY passthrough.

### State: `ExecutingWorldPty(line)`
The REPL executes one command line inside the persistent world session in **PTY passthrough mode** (raw terminal, stdin forwarded).

Actions:
1) Perform the same pre-step as `ExecutingWorldLine` for policy snapshot drift (restart-on-change, preserving `world_cwd` when possible).
2) Submit the user line to the session shell using the protocol in `PROTOCOL.md`:
   - host assigns the next `seq` and a per-command token,
   - submits the line using the brace-framed PTY passthrough submission (no stdin redirection),
   - switches the host terminal into raw mode and begins forwarding stdin bytes and resize events to the session PTY.
3) Continue forwarding until the accepted marker for `(seq, token)` arrives in stdout.

Completion:
- On accepted marker:
  - stop forwarding stdin bytes immediately,
  - restore the host terminal to non-raw mode,
  - update `world_cwd` from `cwd_b64`,
  - set `last_exit` to `exit`,
  - transition back to `Idle`.

Edge case:
- There may be a small race where user keystrokes typed at completion time are dropped or partially delivered to the session shell.
  Substrate should prefer correctness (no unintended extra input to the shell) over preserving every keystroke in this boundary.
- Job control and backgrounding remain unsupported. In particular, `cmd &` can cause the completion marker to fire while work continues in the background,
  undermining per-line auditability and command boundaries.

### State: `ExecutingHost(line)`
The REPL executes one command line on the host.

Working directory:
- The command MUST execute with `host_cwd` as its working directory.
- `:host cd <path>` MUST update `host_cwd` on success (it MUST NOT affect `world_cwd`).

Completion:
- Capture the host command exit code into `last_exit`.
- Transition back to `Idle`.

### State: `ExecutingHostPty(line)`
Runs a command on the host using a host PTY execution path (raw terminal, stdin forwarded).

Contract:
- Intended for full-screen TUIs and programs that require continuous stdin/tty interaction.
- This is the same user-facing “auto-PTY” behavior Substrate provides today when world execution is disabled.

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
- In `ExecutingWorldLine`: ignored (stdin is not forwarded in line mode).
- In `ExecutingWorldPty` / `ExecutingHostPty`: forwarded as a raw stdin byte (so interactive commands can receive EOF).

### Terminal resize
- If a world session exists: Substrate MUST forward resize events (`cols`, `rows`) to world-agent.

## Observability Requirements
For each executed command line, Substrate MUST emit a trace command span that records at minimum:
- `execution_origin`: `world` or `host`
- `exit_code`
- `cwd`: `world_cwd` or `host_cwd` as appropriate
- `world_id` and policy snapshot hash when world execution is used
- A stable per-line correlation identifier (at minimum the span id; command-id correlation is optional but recommended).

The persistent session’s internal bootstrap commands MUST NOT be recorded as user commands.

### Known Limitation (v1)
The v1 persistent-session model does not provide per-REPL-line correlation for in-world subprocess execution events “for free.”
Closing that observability gap is tracked separately as **P0 – In-world process execution tracing parity** in `docs/BACKLOG.md`.
