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
- Each submitted REPL input (a line-editor submission; it may contain embedded newlines) is treated as one “command submission” to the session shell.
- Substrate selects a per-command I/O mode:
  - **Line mode** for non-interactive commands (stdin not forwarded; treated as EOF via `stdin_mode=eof`).
  - **PTY passthrough mode** for interactive commands/TUIs (stdin forwarded; raw terminal mode).
- PTY passthrough mode is selected automatically by Substrate’s existing “needs PTY” heuristic, or explicitly via the `:pty ` prefix (force PTY).

## Persistent State
Substrate maintains the following state for the lifetime of the REPL:
- `world_session`: active/inactive + metadata (session nonce, policy snapshot hash, effective workspace root if any, world id if available).
- `world_cwd`: last known in-world cwd (from the most recent valid `command_complete`), using the **physical** directory semantics defined in `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md` (symlinks resolved; `pwd -P` / `getcwd()`).
- `last_exit`: last command exit code (world or host).
- `host_escape_enabled`: boolean (default `false`).
- `host_cwd`: host-only working directory for `:host` commands.

`world_cwd` and `host_cwd` are independent by design.

## Startup
On REPL startup:
1) Resolve whether world execution is enabled (based on existing `--world/--no-world` and config).
2) If world execution is enabled:
   - Start the persistent world session and wait for readiness (`ready`).
   - Validate `ready.protocol_version` matches `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md` (fail closed if unsupported).
   - Initialize `world_cwd` from `ready.cwd`.
   - If the requested `start_session.cwd` could not be honored and `ready.cwd` differs, Substrate MUST report that the REPL started in a different working directory (see `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`).
   - After `ready`, Substrate MUST recompute the effective policy snapshot hash and effective workspace root for `ready.cwd`. If either differs from what was used to start the session, Substrate MUST immediately restart the world session (same drift restart behavior as in `ExecutingWorldLine`) before accepting user commands. This avoids starting a session under a snapshot that is inconsistent with the actual in-world starting directory.
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
The REPL is waiting for the next input submission via the line editor (Reedline).

Out-of-band world PTY output:
- `stdout` bytes from the world session may arrive while `Idle` (see `PROTOCOL.md` “Out-of-band PTY output”).
- Substrate MUST render/forward those bytes without corrupting the line editor state.
  - Terminal UX guidance (non-normative): suspend prompt rendering, write bytes, then restore the prompt and the current input buffer.
  - Future UX note: a block-based terminal UI may choose to render these as separate “output blocks”, but the byte stream semantics remain the same.

Directive parsing rule (multiline submissions):
- REPL directives (`exit`/`quit`, `:host ...`, `:pty ...`) are recognized only when the submission contains **no embedded newlines**.
  If a submission contains embedded newlines (e.g., paste/multiline input), it MUST be treated as program text and routed by the normal execution rules (world/host + line/PTY heuristics), rather than being parsed as a directive.

Input handling:
- `exit` / `quit`: transition to `ShuttingDown`.
  - `exit` MAY include an optional numeric argument (e.g., `exit 2`). The REPL treats this as an operator termination request, not as a world command. The REPL process exit code remains governed by `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (normal exit stays `0`).
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

Reserved / unsafe shell builtins (session-terminating):
- Submissions that terminate or replace the session shell (e.g., `exit`, `exec ...`, `logout`, `kill $$`) can cause the world session to exit before `command_complete`.
- If you intend to exit the Substrate REPL, use the REPL `exit`/`quit` directives (handled in `Idle`). Do not run `exit` as an in-world submission.
- If the world session exits unexpectedly while a command is in-flight, Substrate MUST treat it as a fatal error and fail closed (see `PROTOCOL.md` Failures).

### State: `ExecutingWorldLine(line)`
The REPL executes one REPL submission inside the persistent world session in **line mode** (`stdin_mode=eof`, no keystroke forwarding).

Actions:
1) Recompute, for `world_cwd`, the effective policy snapshot hash AND the effective workspace root (if any).
2) If either differs from the current session’s values:
   - Tear down the current world session.
   - Start a new world session and wait for readiness (`ready`).
     - Substrate MUST emit an operator-visible message that the world session restarted due to snapshot/workspace-root drift (and record the reason, prior/new snapshot hash, and prior/new workspace root in trace metadata), even if cwd continuity is preserved.
     - Substrate SHOULD request the new session start with `cwd` set to the previous session's `world_cwd` to preserve the
       operator's location.
     - If that `cwd` is rejected/invalid under the new session, Substrate MUST start in the new session's resolved
       project/root directory and MUST report the cwd change.
   - Update `world_cwd` from `ready.cwd`.
   - Note: this restart reinitializes the session shell. Only `world_cwd` continuity is best-effort; other in-session state (exported env mutations, history, shell-local state, etc.) may be lost (see ADR-0016 and decision register DR-09/DR-17).
3) Submit the user line to the session shell using the protocol in `PROTOCOL.md`:
   - host assigns the next `seq`, per-command `token_hex`, and `cmd_id`,
   - sends an `exec` message with `stdin_mode=eof` (stdin is treated as EOF for the duration of the program),
   - streams `stdout` to the user while waiting for the accepted `command_complete(seq, token_hex)`.
4) The host MUST NOT pipeline: it MUST NOT send a second `exec` until the current `exec` completes (sequential in-flight semantics; see `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`).

Outputs:
- `stdout` is streamed to the user unchanged.

Completion:
- On accepted `command_complete`:
  - (Ordering assumption) By protocol, all PTY output from the just-finished foreground command has already been forwarded before `command_complete` is delivered, so it is safe to render the next prompt / resume Reedline after this point (see `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`).
  - update `world_cwd` from `cwd`,
  - set `last_exit` to `exit`,
  - transition back to `Idle`.

Policy resolution note:
- `world_cwd` is a world path. Substrate MUST NOT require `world_cwd` to exist on the host filesystem (e.g., by calling `fs::canonicalize`) when recomputing the effective snapshot/workspace-root; it must use the path namespace semantics defined in `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`.

Errors:
- On protocol error, world session exit, or WebSocket close:
  - print a clear error,
  - transition to `Error` (fatal) and then `ShuttingDown`.
  - Exception: if the REPL is already in `ShuttingDown` and the host has initiated session closure, receiving a world-agent `exit` is expected and treated as graceful shutdown (see `PROTOCOL.md` `close`/`exit` semantics).

Edge case:
- If a command is misclassified into line mode but attempts to read from the controlling TTY (e.g., via `/dev/tty`), it may block.
  The operator can abort with `Ctrl+C` and retry with `:pty <cmd>` to force PTY passthrough.

### State: `ExecutingWorldPty(line)`
The REPL executes one REPL submission inside the persistent world session in **PTY passthrough mode** (raw terminal, stdin forwarded).

Actions:
1) Perform the same pre-step as `ExecutingWorldLine` for drift (restart-on-change for policy snapshot hash OR workspace root, preserving `world_cwd` when possible).
2) Submit the user line to the session shell using the protocol in `PROTOCOL.md`:
   - host assigns the next `seq`, per-command `token_hex`, and `cmd_id`,
   - sends an `exec` message with `stdin_mode=passthrough`,
   - switches the host terminal into raw mode and begins forwarding stdin bytes and resize events to the session PTY.
3) Continue forwarding until the accepted `command_complete(seq, token_hex)` arrives.
4) The host MUST NOT pipeline: it MUST NOT send a second `exec` until the current `exec` completes (sequential in-flight semantics; see `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`).

Completion:
- On accepted `command_complete`:
  - stop forwarding stdin bytes immediately,
  - restore the host terminal to non-raw mode,
  - note: world-agent will ignore any `stdin` frames outside `stdin_mode=passthrough` by protocol, so any “late keystrokes” are dropped rather than leaking into the next command (see `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`),
  - (Ordering assumption) By protocol, all PTY output from the just-finished foreground command has already been forwarded before `command_complete` is delivered, so it is safe to render the next prompt / resume Reedline after this point (see `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`).
  - update `world_cwd` from `cwd`,
  - set `last_exit` to `exit`,
  - transition back to `Idle`.

Edge case:
- There may be a small race where user keystrokes typed at completion time are dropped or partially delivered to the session shell.
  Substrate should prefer correctness (no unintended extra input to the shell) over preserving every keystroke in this boundary.
- Job control and backgrounding remain unsupported. In particular, `cmd &` can cause `command_complete` to fire while work continues in the background,
  undermining per-line auditability and command boundaries.
  - Out-of-band PTY output may continue after `command_complete` (see `PROTOCOL.md`). This is allowed but unattributed in v1.
  - Structured host/agent messages MUST NOT be injected into the PTY byte stream during passthrough (it would corrupt TUIs). If the REPL has concurrent structured events, implementations SHOULD buffer and flush them after passthrough ends (guidance; not a fallback).

### State: `ExecutingHost(line)`
The REPL executes one REPL submission on the host.

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
- Exit the REPL process with code `0` for normal user-initiated exit, otherwise a non-zero code for startup/protocol failures (per `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`).

### State: `Error` (fatal)
Any entry into `Error` MUST be followed by `ShuttingDown` (no continuation in a degraded mode).

## Signals and Terminal Events

### `Ctrl+C` (typed) vs `SIGINT` (host-originated)
In PTY passthrough states, Substrate switches the local terminal into raw mode and forwards keystroke bytes. This means:
- **Typed `Ctrl+C` is a byte** (`0x03`) and MUST be forwarded via `stdin` like any other keystroke; it MUST NOT be translated into a `signal` message. The remote PTY line discipline / foreground program decides what `0x03` means.

In non-PTY line mode, the local terminal is not in raw passthrough. This means:
- **Typed `Ctrl+C` is typically delivered to the Substrate process as `SIGINT`** by the OS, and Substrate must handle it as a host-originated signal (below).

Host-originated `SIGINT` handling:
- In `Idle`: cancels the current line edit (Reedline standard behavior) and stays in `Idle`.
- In `ExecutingWorldLine`: Substrate MUST forward `SIGINT` to the world session via the world-agent `signal` message, then continue waiting for `command_complete`.
  - Signal semantics are session-specific: `signal` targets the currently executing foreground program via the session PTY foreground process group, and must not kill the session driver loop (see `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`).
- In `ExecutingWorldPty`: Substrate MAY receive a host-originated `SIGINT` (e.g., external `kill -INT <pid>`). If it does, it MUST forward it via the world-agent `signal` message and continue waiting for `command_complete`.
- In `ExecutingHost`: Substrate SHOULD forward `SIGINT` to the running host child process.
- In `ExecutingHostPty`: typed `Ctrl+C` is forwarded as a raw byte; host-originated `SIGINT` MAY be forwarded to the host child process.

### `Ctrl+D` (EOF)
- In `Idle`: treated as `exit` → `ShuttingDown`.
- In `ExecutingWorldLine`: ignored (stdin is not forwarded in line mode; `stdin_mode=eof`).
- In `ExecutingWorldPty` / `ExecutingHostPty`: forwarded as a raw stdin byte (so interactive commands can receive EOF).

### Terminal resize
- If a world session exists: Substrate MUST forward resize events (`cols`, `rows`) to world-agent.

## Observability Requirements
For each executed REPL submission, Substrate MUST emit a trace command span that records at minimum:
- `execution_origin`: `world` or `host`
- `exit_code`
- `cwd`: `world_cwd` or `host_cwd` as appropriate
- `world_id` and policy snapshot hash when world execution is used
- A stable per-line correlation identifier (`cmd_id`), which MUST be propagated in-world as `SHIM_PARENT_CMD_ID` for subprocess correlation.

The persistent session’s internal bootstrap commands MUST NOT be recorded as user commands.

Out-of-band output observability:
- If `stdout` bytes are observed while no `exec` is in-flight (or are otherwise out-of-band relative to the awaited foreground command), Substrate SHOULD emit a trace event indicating out-of-band output (e.g., `stdout_out_of_band=true`, `byte_len`, and `repl_state`), without attributing it to a `cmd_id`.

### Known Limitation (v1)
Subprocess correlation depends on shim coverage and telemetry plumbing:
- The protocol provides a clean correlation mechanism (`SHIM_PARENT_CMD_ID`), but end-to-end “parity” (host spans ↔ in-world exec events ↔ replay tooling)
  is still tracked under **P0 – In-world process execution tracing parity** in `docs/BACKLOG.md`.
