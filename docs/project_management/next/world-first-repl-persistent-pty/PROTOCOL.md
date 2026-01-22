# Protocol — Persistent World PTY Session (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`

It specifies the client↔world-agent protocol and the in-world “command boundary” scheme used to:
- keep a single long-lived in-world shell process,
- execute one REPL input line at a time,
- derive a per-line exit code and in-world cwd without terminating the session.

## Scope
- Applies to the interactive REPL only (no `-c/--command`).
- Applies to the interactive REPL on platforms where persistent world-agent `/v1/stream` sessions are supported (initially Linux/macOS).
- All world execution routes through world-agent (no in-process backend execution paths).
- Windows parity is out of scope.

## Terms
- **Host**: the Substrate shell process and the user’s terminal.
- **World-agent**: the in-world daemon handling `/v1/stream` (WebSocket) PTY sessions.
- **World session**: a single long-lived `/v1/stream` WebSocket connection for the REPL duration.
- **Session shell**: the long-lived in-world shell process (child of world-agent) attached to the PTY.
- **Command boundary marker**: a framed byte sequence emitted by the session shell, parsed by the host to determine per-command completion.

## Transport (WebSocket JSON PTY frames)
Substrate uses the existing world-agent `/v1/stream` WebSocket JSON protocol:
- Client → agent: `start`, `stdin`, `resize`, `signal`
- Agent → client: `stdout`, `exit`, `error`

`stdout` payloads are base64-encoded bytes; all parsing in this document occurs on the decoded byte stream.

## Correlation (Per-REPL-Line vs In-World Subprocess Telemetry)
This ADR’s v1 protocol provides per-line completion (exit code + cwd) by parsing client-side markers from the PTY output.
It does not, by itself, provide “host-shim parity” for in-world subprocess tracing in a long-lived session shell.

Authoritative v1 correlation guarantees:
- The host creates a trace command span per REPL line (see `STATE_MACHINE.md`).
- The world-agent sees only the session-level `/v1/stream` start request for the long-lived session shell.

Non-goal for this ADR (explicit):
- Correlating every in-world spawned subprocess to the specific REPL line that caused it is out of scope for this v1
  persistent-session model. That work is tracked as **P0 – In-world process execution tracing parity** in
  `docs/BACKLOG.md` and will likely require additional world-agent capture and/or protocol support beyond the marker scheme.

## Session Shell
The persistent session shell MUST be:
- `/bin/bash` (required dependency of the world image for this feature).

Invocation MUST be deterministic:
- `bash --noprofile --norc`

Notes:
- The shell is treated as a long-lived command interpreter fed via stdin.
- This mode intentionally avoids loading rcfiles; the REPL contract is implemented by Substrate (host), not by user dotfiles.

## Bootstrap (Session Initialization)
After sending the `start` frame and receiving `stdout` data, Substrate MUST bootstrap the session shell by sending a small script over stdin that:
1) Defines the marker emission helper `__substrate_cmd_end`.
2) Makes it read-only.
3) Emits a readiness marker with `seq=0`.

The bootstrap MUST NOT rely on prompts (`PS1`) or `PROMPT_COMMAND`.

### Session Nonce
Substrate MUST generate a per-session random nonce `nonce_hex` and embed it into the bootstrap definition.
- Format: lowercase hex string, 32 chars (16 bytes).
- Purpose: reduce accidental marker collisions; it is a robustness mechanism, not a security boundary.

### Per-Command Token
Substrate MUST generate a per-command random token `token_hex` and include it in the marker invocation appended to the submitted command and in the marker payload.
- Format: lowercase hex string, 32 chars (16 bytes).
- Purpose: prevent a command from spoofing a valid completion marker for the currently awaited command.
- Note: this is a protocol integrity mechanism for the host parser. It is not a general security boundary against a malicious
  in-world process that can read or consume PTY input bytes via `/dev/tty` or similar.

### Readiness Marker
Substrate MUST wait for the readiness marker (`seq=0`) before accepting user commands.
The readiness marker MUST include a token (the host chooses it and validates it like any other command).

## Command Submission (Per REPL Line)
### I/O Modes (Line vs PTY Passthrough)
The world-first REPL is line-editor driven, but it must retain support for interactive programs and TUIs.
Therefore, Substrate MUST support two per-command I/O modes for the persistent world session:

- **Line mode** (default for non-interactive commands):
  - The user enters a single line in the host line editor.
  - Substrate sends that line to the session shell and waits for the completion marker.
  - Substrate does not forward additional user keystrokes to the PTY while the command runs.
  - To prevent accidental hangs, stdin for the user command MUST be redirected to `/dev/null` in line mode.
  - Note: stdin redirection does not prevent reads from `/dev/tty`. Commands that read directly from the TTY (e.g., password prompts)
    must be classified into PTY passthrough mode (auto-PTY) or explicitly forced with `:pty`.

- **PTY passthrough mode** (for interactive commands / TUIs):
  - Substrate sends the command to the session shell and then temporarily switches the host terminal into raw mode.
  - While the command runs, Substrate forwards user keystrokes (stdin bytes) and resize events to the PTY until the completion marker is observed.
  - PTY passthrough mode MUST be selected:
    - automatically when Substrate’s existing “needs PTY” heuristic classifies the line as interactive, and
    - explicitly when the user prefixes the line with `:pty ` (force PTY).

This preserves current Substrate behavior where commands like `vim`, `lazygit`, `python` (REPL), and `sudo` prompts work as normal REPL lines (auto-PTY), without forcing users to learn separate workflows.

For each accepted REPL input line:
- If the line is empty/whitespace-only: Substrate MUST NOT send anything to the session shell.
- Otherwise, Substrate MUST send a framed “compound command” such that:
  - the session shell consumes (parses) the marker invocation bytes before starting the user command, and
  - the marker invocation runs only after the user command returns control to the session shell.

This is required to prevent interactive commands from consuming marker bytes as stdin.

#### Line mode submission (stdin redirected; no keystroke forwarding)
- Substrate MUST send a multi-line frame of the form:
  1) `{\n`
  2) `<user_line>\n`
  3) `} </dev/null; __substrate_cmd_end <seq> <token_hex>\n`

This preserves shell state semantics (the brace-group runs in the current shell), prevents stdin-consuming commands from hanging, and ensures the marker is executed after the user command completes.

#### PTY passthrough submission (stdin forwarded)
- Substrate MUST send a multi-line frame of the form:
  1) `{\n`
  2) `<user_line>\n`
  3) `}; __substrate_cmd_end <seq> <token_hex>\n`
  - and then enter PTY passthrough mode (forward stdin bytes + resize events) until the marker is observed.

Where `<seq>`:
- Is a strictly increasing unsigned integer starting at `1` for the first user command of the session.
- MUST be chosen by the host and is the “command id” for boundary correlation.

This design intentionally avoids wrapping the user command in an extra `bash -c` layer: quoting and parsing are the shell’s responsibility.

### Edge Cases (Command Shape)
Some shell constructs can undermine “one command == one completion boundary”:
- `exit`/`logout` within the session shell will terminate the session before the marker runs.
- `exec <cmd>` will replace the session shell process, likely terminating the protocol.
- Backgrounding (`cmd &`) returns control to the shell immediately; the marker will run even though a background job may continue producing output and mutating the filesystem.
- Multiline shell constructs that require additional input (e.g., heredocs, `if/for/while` blocks, unmatched quotes) require PTY passthrough mode so the operator can provide continuation input. If they are run in line mode, the session shell may block waiting for more input.

These constructs are not supported in the default contract; Substrate may treat them as session-terminating or “best effort” with reduced audit fidelity.

## Command Boundary Marker (Framing + Payload)
The marker is a byte sequence written by `__substrate_cmd_end` to stdout.

### Framing
- Start sentinel byte: `0x1E` (ASCII RS).
- End sentinel byte: `0x1F` (ASCII US).

All bytes between the sentinels are ASCII payload.

### Payload Format (tab-separated)
The payload MUST match:

`SUBSTRATE_CMD_END\t2\t<nonce_hex>\t<seq>\t<token_hex>\t<exit>\t<cwd_b64>`

Fields:
- `SUBSTRATE_CMD_END`: literal.
- `2`: protocol version literal.
- `<nonce_hex>`: the session nonce (lowercase hex, 32 chars).
- `<seq>`: the host-provided sequence number (base-10 `u64`).
- `<token_hex>`: the host-provided per-command token (lowercase hex, 32 chars).
- `<exit>`: exit code of the immediately preceding shell command (base-10 `i32`), derived from `$?`.
- `<cwd_b64>`: base64 of the UTF-8 bytes of `$PWD` (no newlines).

### Required Helper Semantics
`__substrate_cmd_end <seq> <token_hex>` MUST:
- Read `$?` as the exit code of the command that ran immediately before it.
- Read `$PWD` as the current in-world working directory after that command.
- Print exactly one framed marker corresponding to `<seq>` and `<token_hex>`.

The helper MUST be marked read-only:
- `readonly -f __substrate_cmd_end`

## Parsing Rules (Host)
The host MUST parse markers from the decoded `stdout` byte stream using a streaming buffer:
- Markers may be split across multiple `stdout` frames.
- Non-marker output MUST be forwarded to the user unchanged.
- Accepted marker bytes MUST NOT be forwarded to the user.

### Candidate Marker Detection (Prefix Gate)
Because the PTY output stream can contain arbitrary bytes (including control characters), the host MUST NOT treat every
`0x1E ... 0x1F` framed segment as a protocol marker.

Instead, when a framed segment is observed:
- If the payload begins with the ASCII prefix `SUBSTRATE_CMD_END\t2\t`, treat it as a marker candidate and apply the
  validation rules below.
- Otherwise, treat the entire framed segment (including the `0x1E` and `0x1F` sentinel bytes) as normal output and
  forward it to the user unchanged.

### Marker Acceptance Criteria
A candidate marker is accepted only if:
- Framing bytes `0x1E ... 0x1F` are present and ordered.
- Payload begins with `SUBSTRATE_CMD_END\t2\t` and parses into 7 tab-separated fields with version `2`.
- `nonce_hex` matches the session nonce for this world session.
- `seq` equals the host’s currently awaited sequence number.
- `token_hex` equals the host’s currently awaited per-command token.
- `cwd_b64` base64-decodes to valid UTF-8.

If a marker-candidate payload (prefix matches) fails validation, the host MUST treat it as a protocol error (see “Failures”).

## Failures (No Fallbacks)
If any of the following occur, Substrate MUST fail the REPL session (no host fallback):
- The world session WebSocket closes unexpectedly.
- The session shell exits unexpectedly (agent sends `exit`).
- A protocol error occurs (invalid marker framing/payload, nonce mismatch, seq mismatch).

The failure MUST be:
- high signal (clear error output),
- traceable (recorded in trace spans),
- and non-ambiguous (explicitly identify “protocol desync” vs “world backend unavailable”).

## Policy Snapshot Drift (Session Restart, Not Fallback)
The world session is configured at `start` via the policy snapshot included in the `start` frame.

To prevent drift between host-side policy evaluation and in-world enforcement:
- Before executing a new REPL command, Substrate MUST compute the effective policy snapshot for the current in-world cwd.
- If the effective snapshot hash differs from the snapshot hash used to start the current world session, Substrate MUST:
  1) Tear down the current world session.
  2) Start a new world session with the new snapshot.
     - CWD continuity: Substrate SHOULD request the new session start with `cwd` set to the previous session's last known
       `world_cwd` to preserve the operator's location.
     - If the requested `cwd` is invalid under the new session (e.g., it does not exist, it is outside the resolved world
       root/cage, or the backend rejects it), Substrate MUST instead start the new session in the new session's resolved
       project/root directory (as implied by the new effective snapshot) and MUST report that the session restarted and
       that the working directory changed.
  3) Re-bootstrap and emit a new readiness marker.

This restart is a correctness requirement; it is not optional and is not a “legacy mode” fallback.
