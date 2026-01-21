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
Substrate MUST generate a per-command random token `token_hex` and include it in the marker request line and marker payload.
- Format: lowercase hex string, 32 chars (16 bytes).
- Purpose: prevent a command from spoofing a valid completion marker for the currently awaited command.
- Note: this is a protocol integrity mechanism for the host parser. It is not a general security boundary against a malicious
  in-world process that can read or consume PTY input bytes via `/dev/tty` or similar.

### Readiness Marker
Substrate MUST wait for the readiness marker (`seq=0`) before accepting user commands.
The readiness marker MUST include a token (the host chooses it and validates it like any other command).

## Command Submission (Per REPL Line)
### Stdin Contract (Persistent Session Mode)
In persistent-session mode, stdin is part of the REPL command channel. Therefore:
- User commands MUST NOT be permitted to read from the session shell stdin.
- Substrate MUST execute user lines with stdin redirected to `/dev/null` so stdin-consuming commands observe EOF and cannot consume the marker line.

Implications:
- Commands that require interactive stdin (e.g. `cat` with no args, `python` REPL, `read`, password prompts, etc.) are unsupported in the default REPL mode and MUST use `:pty <cmd>`.

For each accepted REPL input line:
- If the line is empty/whitespace-only: Substrate MUST NOT send anything to the session shell.
- Otherwise, Substrate MUST send the following bytes to the session shell (in order):
  1) A brace-group start line: `{\n`
  2) The user line exactly as typed, followed by `\n`.
  3) A brace-group end line with stdin redirected: `} </dev/null\n`
  4) A marker request line: `__substrate_cmd_end <seq> <token_hex>\n`

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
