# Protocol — Persistent World PTY Session (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`

It specifies the client↔world-agent protocol and the in-world “command boundary” scheme used to:
- keep a single long-lived in-world shell process,
- execute one REPL input line at a time,
- derive a per-line exit code and in-world cwd without terminating the session.

## Scope
- Applies to the interactive REPL only (no `-c/--command`).
- Applies to Linux/macOS world backends that use world-agent `/v1/stream`.
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

### Readiness Marker
Substrate MUST wait for the readiness marker (`seq=0`) before accepting user commands.

## Command Submission (Per REPL Line)
For each accepted REPL input line:
- If the line is empty/whitespace-only: Substrate MUST NOT send anything to the session shell.
- Otherwise, Substrate MUST send the following bytes to the session shell (in order):
  1) The user line exactly as typed, followed by `\n`.
  2) A marker request line: `__substrate_cmd_end <seq>\n`

Where `<seq>`:
- Is a strictly increasing unsigned integer starting at `1` for the first user command of the session.
- MUST be chosen by the host and is the “command id” for boundary correlation.

This design intentionally avoids wrapping the user command in an extra `bash -c` layer: quoting and parsing are the shell’s responsibility.

## Command Boundary Marker (Framing + Payload)
The marker is a byte sequence written by `__substrate_cmd_end` to stdout.

### Framing
- Start sentinel byte: `0x1E` (ASCII RS).
- End sentinel byte: `0x1F` (ASCII US).

All bytes between the sentinels are ASCII payload.

### Payload Format (tab-separated)
The payload MUST match:

`SUBSTRATE_CMD_END\t1\t<nonce_hex>\t<seq>\t<exit>\t<cwd_b64>`

Fields:
- `SUBSTRATE_CMD_END`: literal.
- `1`: protocol version literal.
- `<nonce_hex>`: the session nonce (lowercase hex, 32 chars).
- `<seq>`: the host-provided sequence number (base-10 `u64`).
- `<exit>`: exit code of the immediately preceding shell command (base-10 `i32`), derived from `$?`.
- `<cwd_b64>`: base64 of the UTF-8 bytes of `$PWD` (no newlines).

### Required Helper Semantics
`__substrate_cmd_end <seq>` MUST:
- Read `$?` as the exit code of the command that ran immediately before it.
- Read `$PWD` as the current in-world working directory after that command.
- Print exactly one framed marker corresponding to `<seq>`.

The helper MUST be marked read-only:
- `readonly -f __substrate_cmd_end`

## Parsing Rules (Host)
The host MUST parse markers from the decoded `stdout` byte stream using a streaming buffer:
- Markers may be split across multiple `stdout` frames.
- Non-marker output MUST be forwarded to the user unchanged.
- Marker bytes MUST NOT be forwarded to the user.

### Marker Acceptance Criteria
A candidate marker is accepted only if:
- Framing bytes `0x1E ... 0x1F` are present and ordered.
- Payload parses into 6 tab-separated fields with version `1`.
- `nonce_hex` matches the session nonce for this world session.
- `seq` equals the host’s currently awaited sequence number.
- `cwd_b64` base64-decodes to valid UTF-8.

If a framed payload fails validation, the host MUST treat it as a protocol error (see “Failures”).

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
  3) Re-bootstrap and emit a new readiness marker.

This restart is a correctness requirement; it is not optional and is not a “legacy mode” fallback.
