# Protocol — World-First REPL Persistent Session (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`

It specifies the host↔world-agent protocol for the world-first interactive REPL, including:
- a long-lived in-world PTY session,
- per-submission execution with explicit completion messages (no stdout marker parsing),
- robust multiline support,
- and a hardened separation between the **command-control channel** and **user stdin**.

## Scope
- Applies to the interactive REPL only (no `-c/--command`).
- Applies to platforms where persistent world-agent streaming sessions are supported (initially Linux/macOS).
- All world execution routes through world-agent.
- Windows parity is out of scope.

## Terms
- **Host**: the Substrate shell process and the user’s terminal.
- **World-agent**: the in-world daemon that exposes `/v1/stream` (WebSocket).
- **World session**: a single long-lived WebSocket connection for the duration of the REPL.
- **Session shell**: a long-lived `/bin/bash` process attached to a PTY, managed by world-agent.
- **Driver loop**: a small in-world control loop (owned by world-agent) that:
  - receives per-submission programs over a control channel,
  - executes them in the session shell,
  - and reports structured completion events to the host.
- **Trusted driver component**: an implementation-defined component owned by world-agent that holds the private control-plane endpoints and mediates execution/completion; it is trusted infrastructure, not user-submitted code.

## Transport (WebSocket JSON frames)
Substrate uses world-agent `/v1/stream` over WebSocket.

This protocol extends the `/v1/stream` JSON frame protocol for persistent REPL sessions:
- Client → agent:
  - `start_session`
  - `exec`
  - `stdin`, `resize`, `signal`
  - `close`
- Agent → client:
  - `ready`
  - `stdout`
  - `command_complete`
  - `exit`
  - `error`

`stdout` frame schema (authoritative):
- `{"type":"stdout","data_b64":<base64 bytes>}`

Semantics:
- `stdout.data_b64` is base64-encoded bytes of the raw PTY output stream (stdout+stderr combined).
- `stdout` frames may be arbitrarily chunked/split/coalesced; the host MUST treat them as a byte stream and forward decoded bytes to the user unchanged.
- Ordering guarantees:
  - World-agent MUST preserve PTY byte ordering when emitting `stdout` frames.
  - For a given `exec`, `command_complete` MUST NOT be emitted until all foreground PTY output bytes for that command have been forwarded (see `command_complete` ordering rules below).

Out-of-band PTY output (session-level stdout):
- `stdout` frames MAY arrive while no `exec` is in-flight (e.g., due to background processes, other in-world writers to the controlling TTY, or future Substrate-managed background activity).
- The host MUST still forward/render these bytes (no suppression).
- Such bytes are **unattributed** to a specific `cmd_id` by default. The v1 protocol does not provide job control or background attribution, and implementations MUST NOT guess attribution.
- Forward-compat: `command_complete` is a foreground completion boundary, not a guarantee that no further `stdout` bytes will occur on the session PTY stream.

Compatibility note:
- Existing one-shot PTY execution over `/v1/stream` is not replaced by this ADR.
- If world-agent does not support `start_session`/`exec`, the world-first REPL MUST fail closed (no fallback to legacy client-side parsing).
- In the interactive REPL, auto-PTY and `:pty` use `stdin_mode=passthrough` within the persistent session; one-shot `/v1/stream` remains for non-REPL contexts.

Session initiation (mode distinction):
- The first client frame on a `/v1/stream` WebSocket connection MUST be exactly one of:
  - `{"type":"start", ...}` (legacy one-shot mode), or
  - `{"type":"start_session", ...}` (persistent REPL session mode, `protocol_version=1`).
- World-agent MUST treat any other first frame as `error.code=bad_request` (or equivalent fatal error) and MUST fail/close the session.

## Session Shell
The session shell MUST be `/bin/bash` with deterministic invocation:
- `bash --noprofile --norc`

World-agent SHOULD suppress in-world prompt output by setting:
- `PS1=""`, `PS2=""`, and `PROMPT_COMMAND=""` in the session shell environment.

## Key Design Invariant: Separate Command Channel vs User Stdin
Persistent sessions must support auto-PTY (interactive stdin forwarding) without allowing interactive programs to consume REPL control bytes.

Therefore:
- The host MUST NOT send per-submission program text over the PTY stdin stream.
- Program text MUST be sent over a dedicated **command-control channel** that is not the PTY stdin.
- The PTY stdin stream is reserved for **user keystrokes** during PTY passthrough mode.

This eliminates the “REPL control bytes consumed by stdin” failure mode and avoids shell-syntax splice edge cases (`#` comments, trailing `;`, etc.).

## Internal Driver Loop (World-Agent Owned)
When a REPL world session starts, world-agent MUST:
1) Allocate one PTY for the session shell (stdout/stderr stream to the host; stdin used for user keystrokes during passthrough).
2) Create two additional OS pipes (or equivalent) for a private control plane:
   - **FD 9 (command channel)**: world-agent → trusted driver component (read-only in the driver).
   - **FD 8 (completion channel)**: trusted driver component → world-agent (write-only in the driver).
   - FD numbering note: the specific FD numbers shown above are illustrative. The implementation SHOULD allocate high-numbered, reserved FDs (e.g., `>= 200`) to minimize collisions with user workflows that use explicit low-numbered file descriptors. Regardless of the chosen numbers, user-submitted programs MUST NOT be able to read/write these control-plane FDs (see below).
3) Ensure **FD 8 and FD 9 are not inherited by exec’d subprocesses** spawned by untrusted programs (e.g., set close-on-exec).
4) Ensure **the untrusted evaluation context cannot read from FD 9 or write to FD 8**:
   - Closing on exec is necessary but not sufficient, because user-submitted shell programs evaluated in the session shell can still reference open file descriptors via redirections (e.g., `>&8`, `<&9`) if those FDs are present in the evaluation context.
   - The driver loop MUST be structured such that the control-plane FDs are inaccessible to untrusted user code during program execution (mechanism is implementation-defined, but this invariant is required).
5) Start the session shell and driver loop such that the trusted driver component:
   - receives programs from the private command-control channel,
   - executes them in the session shell with the stdin-mode semantics below,
   - and reports structured completions over the private completion channel.

### Control-plane FD privacy (beyond `FD_CLOEXEC`)
This protocol relies on the control-plane FDs being private to the trusted driver loop.

Therefore, it is a MUST-level requirement that user-submitted programs cannot:
- write bytes that are interpreted as a completion record (FD 8), or
- read/peek future submissions, tokens, or framing (FD 9).

If a user submission attempts to access these channels (e.g., `echo hi >&8`), it MUST fail harmlessly (e.g., `EBADF`) and MUST NOT cause premature `command_complete` acceptance or desync.

Invariant reminder:
- Meeting the control-plane FD privacy requirement MUST NOT be achieved by silently weakening the persistent-session contract (e.g., switching to per-command fresh shells that lose the persistence guarantees in ADR-0016) unless the ADR/decision register is explicitly revised.

> Implementation note (non-normative)
>
> The requirement above is subtle: `FD_CLOEXEC` prevents *child process* inheritance, but it does not prevent shell-evaluated code in the session shell from using redirections to access still-open FDs.
>
> Example implementation patterns that can satisfy the MUST invariant (pick one; not exhaustive):
> - **Keep the control-plane FDs out of the session shell entirely**: put the command/control plane in a separate helper process (owned by world-agent) and ensure the bash process that evaluates user submissions never has readable/writable handles to those FDs.
> - **Evaluate user code in an execution context where the control-plane FDs are closed/disabled**: e.g., run the submission in a confined subshell/process with the reserved FDs closed, and propagate only the necessary persistent session state back to the driver (e.g., cwd/env deltas), rather than evaluating directly in the FD-holding context.
>
### Command framing (FD 9)
For each submitted REPL program, world-agent writes the following **NUL-delimited** fields to FD 9:
1) `seq` (ASCII base-10 `u64`)
2) `token_hex` (lowercase hex, 32 chars)
3) `stdin_mode` (`eof` or `passthrough`)
4) `cmd_id` (UUIDv7 string from the host; used for tracing correlation)
5) `program` (UTF-8 bytes; may contain newlines; MUST NOT contain `0x00`)

Each field is terminated by a single `0x00` byte.

Constraint:
- Because framing is NUL-delimited, `program` MUST NOT contain NUL bytes. If the decoded program contains `0x00`, world-agent MUST reject it as a protocol error.

### Completion framing (FD 8)
After running a program, the driver loop writes these NUL-delimited fields to FD 8:
1) `seq` (ASCII base-10 `u64`)
2) `token_hex` (lowercase hex, 32 chars)
3) `exit` (ASCII base-10 `i32`)
4) `cwd` (UTF-8 absolute path, no NUL)

Each field is terminated by a single `0x00` byte.

### Execution semantics (stdin modes)
For each `(seq, token)`:
- The driver loop MUST set `SHIM_PARENT_CMD_ID=<cmd_id>` for the duration of executing the program, so in-world shim logs can be correlated to the host command span.
- `SHIM_PARENT_CMD_ID` MUST be applied only to the foreground program evaluation and its descendants, and MUST NOT persist as a user-visible exported variable across subsequent submissions (the driver MUST restore/unset it after the command completes).
- The driver loop MUST execute the program in the session shell context (so `cd`/`export`/`unset` semantics persist).
- The driver loop MUST ensure the executed program cannot access the private control-plane FDs (FD 8/FD 9).
- If `stdin_mode=eof`, the driver loop MUST execute the program with stdin effectively EOF (e.g., by applying a temporary `</dev/null` redirection around the evaluation).
- If `stdin_mode=passthrough`, the driver loop MUST execute the program with stdin connected to the PTY, and the host will forward user keystrokes to the PTY until completion.

The driver loop MUST emit exactly one completion record per accepted `exec` request.

## Host Protocol (Authoritative)

### `start_session`
The host MUST start a world session by sending:
- `{"type":"start_session","cwd":<path>,"env":{...},"policy_snapshot":<...>,"cols":<u16>,"rows":<u16>}`

World-agent MUST respond with `ready` once the driver loop is running.

#### `policy_snapshot` schema (startup)
`start_session.policy_snapshot` is semantically meaningful during session bootstrap and MUST use the same JSON schema as `agent_api_types::PolicySnapshotV1` (see `crates/agent-api-types/src/lib.rs`), i.e., the same snapshot type used by `/v1/execute` as `ExecuteRequest.policy_snapshot`.

Minimal shape (schema version 1):
- `{"schema_version":1,"world_fs":{...},"net_allowed":[...],"limits":{...}}`
- `world_fs`: `{"mode":<WorldFsMode>,"isolation":"workspace|full","require_world":<bool>,"read_allowlist":[...],"write_allowlist":[...]}`
- `limits`: `{"max_memory_mb":<optional u64>,"max_cpu_percent":<optional u32>,"max_runtime_ms":<optional u64>,"max_egress_bytes":<optional u64>}`

Fail-closed validation:
- The snapshot type is `deny_unknown_fields`; unknown fields MUST be rejected.
- If `policy_snapshot` is missing or fails schema validation, world-agent MUST treat the request as invalid (e.g., `error.code=bad_request`) and MUST fail/close the session.

#### `env` semantics (startup)
`start_session.env` is the **full environment** (`envp`) for the session shell.

Therefore:
- World-agent MUST NOT implicitly inherit environment variables from the world-agent process when launching the session shell.
- World-agent MAY apply documented session-level overrides needed for correctness/UX (e.g., forcing `PS1/PS2/PROMPT_COMMAND` empty to suppress in-world prompts, and normalizing XDG paths to a writable in-world location when necessary).

#### `cwd` resolution (startup)
`start_session.cwd` is a requested starting working directory for the session shell.

Definition: “resolved session root/project directory”
- This refers to the world backend’s anchor/project root for the session (often called `project_dir` in the codebase), i.e., the root directory that the world filesystem view is anchored to.
- World-agent MUST derive this root using the same anchor semantics as the backend configuration (see `docs/CONFIGURATION.md` “anchor mode/path”):
  - If `SUBSTRATE_ANCHOR_MODE=custom`: use `SUBSTRATE_ANCHOR_PATH` (required).
  - If `SUBSTRATE_ANCHOR_MODE=follow-cwd`: use the requested `start_session.cwd`.
  - If `SUBSTRATE_ANCHOR_MODE=project` (default): use `SUBSTRATE_ANCHOR_PATH` if set; otherwise use the requested `start_session.cwd`.

World-agent MUST:
- attempt to start the session shell in `start_session.cwd`, and
- if it cannot honor `start_session.cwd` (e.g., path does not exist, is outside the world root/cage, or is rejected by backend/policy constraints), start the session in the **resolved session root/project directory** for that world session (e.g., workspace root / configured anchor root, consistent with the provided `policy_snapshot`), and return that as `ready.cwd`.
- Additionally, if `start_session.cwd` is outside the resolved session root/project directory, world-agent MUST start the session in the resolved session root/project directory and return that as `ready.cwd`.

The host MUST treat `ready.cwd` as authoritative for prompt/state.
If `ready.cwd` differs from the requested `start_session.cwd`, the host MUST emit an operator-visible message that the session started in a different working directory (and record it in trace metadata).

### `ready`
World-agent MUST reply:
- `{"type":"ready","session_nonce":<hex32>,"cwd":<path>,"protocol_version":1}`

The host MUST wait for `ready` before accepting user commands.

#### Version negotiation (fail-closed)
This document defines **protocol version 1** (`protocol_version=1`).

Therefore:
- If `ready.protocol_version != 1`, the host MUST treat the session as unsupported and MUST fail closed.
- While operating under `protocol_version=1`, the host MUST treat any server frame with an unknown `type` (i.e., not one of `ready`, `stdout`, `command_complete`, `exit`, `error`) as a protocol error and MUST fail closed.

### `exec`
For each REPL submission, the host MUST:
1) Choose a new `seq` (strictly increasing `u64`, starting at `1`).
2) Generate a per-command `token_hex` (lowercase hex, 32 chars).
3) Generate a per-command `cmd_id` (UUIDv7 string).
4) Decide `stdin_mode`:
   - `eof` for line mode (default)
   - `passthrough` for PTY passthrough mode (auto-PTY or `:pty`)
5) Send:
   - `{"type":"exec","seq":<u64>,"token_hex":<hex32>,"cmd_id":<uuid>,"stdin_mode":"eof|passthrough","program_b64":<base64 utf-8 bytes>}`

Logging/redaction guidance:
- `token_hex` is anti-spoofing material; hosts SHOULD redact or hash it in logs/traces and MUST NOT print full tokens to the operator terminal. World-agent SHOULD avoid logging full tokens as well.

#### In-flight semantics (no pipelining)
The world-first REPL execution model is sequential:
- The host MUST NOT send a new `exec` while another `exec` is in-flight (i.e., before receiving the accepted `command_complete` for the prior `(seq, token_hex)`).
- World-agent MUST treat a concurrent `exec` (received while another command is running) as a protocol error and MUST fail the session (e.g., by sending `error` and/or closing the session).

World-agent MUST:
- base64-decode `program_b64` to bytes,
- validate the bytes are UTF-8 (the “program” string),
- reject any program containing NUL (`0x00`) bytes (because the internal FD framing is NUL-delimited),
- and then deliver the decoded program to the session shell via the private command-control channel (FD 9) for execution by the driver loop.

Non-interactive execution note:
- The persistent-session REPL contract MUST NOT rely on interactive shell continuation prompts (PS2). Incomplete constructs must fail as a bounded submission (syntax error) and return to `Idle` (see DR-13 and `plan.md` validation).

### `command_complete`
World-agent MUST send:
- `{"type":"command_complete","seq":<u64>,"token_hex":<hex32>,"exit":<i32>,"cwd":<path>}`

Exit code semantics:
- `exit` MUST reflect the session shell’s standard `$?`/wait-status semantics for the just-finished foreground submission.
- If the submission terminates due to a signal, `exit` SHOULD follow bash conventions (typically `128 + signal_number`, e.g., `SIGINT` → `130`) so audit/replay expectations are stable.

Working directory (`cwd`) semantics:
- `cwd` MUST be the **physical** working directory of the session shell after the submission completes (i.e., symlinks resolved; equivalent to `pwd -P` / `getcwd()` semantics).
- `cwd` MUST be an absolute path string.

Path namespace requirement:
- All `cwd` values (`ready.cwd` and `command_complete.cwd`) are **world-absolute paths in the session’s filesystem view**.
- They MUST be in the same path namespace as `start_session.cwd` (so the host can use `world_cwd` for policy/workspace resolution without requiring the path to exist on the host filesystem).
- The host MUST NOT `fs::canonicalize()` or otherwise require host-side existence of `cwd` for policy snapshot resolution; doing so reintroduces the original “exists in world but cd fails” class of bugs.

#### Output ordering / drain guarantee (PTY stdout vs `command_complete`)
For a given `exec(seq, token)`:
- World-agent MUST preserve PTY byte ordering when emitting `stdout` frames.
- World-agent MUST NOT emit `command_complete(seq, token, ...)` until it has forwarded all PTY output bytes produced by that command’s foreground execution.

This requirement exists to prevent post-completion output from interleaving with the next REPL prompt or Reedline input rendering.

Note:
- Output produced *after* `command_complete` may still occur if the command started background work (unsupported) or if other in-world processes write to the session PTY asynchronously. Such behavior undermines per-line auditability and is treated as out of scope (see `STATE_MACHINE.md` job control notes).

The host MUST treat a `command_complete` as accepted only if:
- `seq` equals the currently awaited sequence number, and
- `token_hex` equals the currently awaited per-command token.

If `seq/token_hex` do not match the awaited command, the host MUST treat it as a protocol error (no continuation in a degraded mode).

### `stdin`, `resize`, `signal`
- During PTY passthrough mode, the host MUST forward user keystrokes to world-agent using existing:
  - `{"type":"stdin","data_b64":<base64 bytes>}`
- The host MUST forward terminal resizes:
  - `{"type":"resize","cols":<u16>,"rows":<u16>}`
- The host MAY forward host-originated signals (e.g., OS-delivered signals to the Substrate process, or an explicit UI “Interrupt” action) to the current running program via the PTY:
  - `{"type":"signal","sig":"INT|TERM|HUP|QUIT|..."}`

#### `stdin` acceptance / boundary rules (agent-side, normative)
World-agent receives `stdin` frames at the session level, but per-command stdin forwarding is only meaningful in PTY passthrough mode.

Therefore:
- Before `ready` is sent, world-agent MUST ignore/drop all `stdin` frames.
- While no command is running, world-agent MUST ignore/drop all `stdin` frames.
- While a command is running with `stdin_mode=eof`, world-agent MUST ignore/drop all `stdin` frames (they MUST NOT be delivered to the session PTY).
- While a command is running with `stdin_mode=passthrough`, world-agent MUST forward decoded `stdin` bytes to the session PTY stdin stream.
- After emitting `command_complete` for a command, world-agent MUST ignore/drop subsequent `stdin` frames until a new `exec` begins with `stdin_mode=passthrough`.

This ensures that “late keystrokes” near completion cannot leak into the next command and cannot mutate shell state unexpectedly. It also prevents minor host/terminal races from becoming protocol-fatal.

#### `signal` targeting semantics (agent-side, normative)
In persistent sessions, `signal` is intended to control the currently executing *foreground program*, not the session shell / driver loop itself.

Important distinction:
- During `stdin_mode=passthrough`, typed control keys (including `Ctrl+C` / `0x03`) are transported as `stdin` bytes. The host MUST NOT translate typed keystrokes into `signal` messages; the remote PTY line discipline / foreground program decides what those bytes mean.

Therefore:
- Before `ready` is sent, world-agent MUST ignore/drop all `signal` messages.
- While no command is running, world-agent MUST ignore/drop all `signal` messages.
- While a command is running, world-agent MUST deliver the signal to the **foreground process group** of the session PTY (or the closest equivalent on the platform).
  - World-agent MUST NOT target the session shell PID directly for interactive control signals like `SIGINT`, because that can terminate the driver loop and cause a fatal session loss.
  - The driver loop/session shell setup MUST ensure the “currently executing command” runs in a distinct foreground process group so that `SIGINT` interrupts the active program without terminating the session driver.

Operational intent:
- `SIGINT` must interrupt the currently executing foreground command and the session must remain usable afterward (host continues waiting for `command_complete`).
- `SIGTERM`/`SIGHUP` should terminate the currently executing foreground command when possible; session termination should use `close` / WebSocket shutdown.

### `close` / `exit`
- On REPL shutdown, the host SHOULD send `{"type":"close"}` and close the WebSocket.
- World-agent MAY send `{"type":"exit","code":<i32>}` if the session shell exits.

Expected vs unexpected session exit:
- If the host has initiated shutdown (sent `close` and/or is intentionally closing the WebSocket), receiving `exit` is expected and SHOULD be treated as a graceful shutdown acknowledgement.
- If world-agent sends `exit` while the host is not shutting down (i.e., a session shell dies unexpectedly, especially while a command is in-flight), the host MUST treat it as fatal and fail closed.

### `error`
World-agent MAY send an error frame to report a fatal failure for the session or for a specific `exec`.

Frame shape (authoritative for `protocol_version=1` persistent sessions):
- `{"type":"error","code":<string>,"message":<string>,"fatal":<bool>,"seq":<optional u64>}`

Compatibility note:
- This `error` frame schema applies to `start_session` persistent sessions under `protocol_version=1`.
- Legacy one-shot `/v1/stream` `start` executions may continue using the legacy error frame shape for backwards compatibility.

Fields:
- `code`: stable machine-readable identifier (snake_case string).
- `message`: human-readable diagnostic intended for display.
- `fatal`: whether the session must terminate. For `protocol_version=1` persistent sessions, `fatal` MUST be `true`.
- `seq` (optional): the `exec.seq` this error corresponds to, when applicable (e.g., `exec_while_busy`, invalid `program_b64`).

The host MUST treat any `error` frame as fatal for the REPL session (no continuation in a degraded mode), record it in trace, and surface `code` + `message` to the operator.
If the `error` frame is missing required fields or fails validation, the host MUST treat it as a protocol error and fail closed.

Recommended error codes (non-exhaustive):
- `unsupported_protocol_version`
- `bad_request`
- `program_invalid_utf8`
- `program_contains_nul`
- `exec_while_busy`
- `protocol_violation`
- `internal_error`

Recommended error code mapping (for testability; non-normative but strongly preferred):
- If `program_b64` is invalid base64: `bad_request`
- If `program_b64` decodes but is not valid UTF-8: `program_invalid_utf8`
- If the decoded program contains NUL (`0x00`): `program_contains_nul`

## Failures (No Fallbacks)
If any of the following occur, Substrate MUST fail the REPL session (no host fallback):
- The world session WebSocket closes unexpectedly.
- World-agent reports `error`.
- The session shell exits unexpectedly (`exit`).
- A protocol error occurs (unexpected/mismatched `command_complete`, invalid `ready`, invalid framing, etc.).

The failure MUST be:
- high signal (clear error output),
- traceable (recorded in trace spans),
- and non-ambiguous (explicitly identify “protocol desync” vs “world backend unavailable”).

## Policy Snapshot Drift (Session Restart, Not Fallback)
The world session is configured at `start_session` via the policy snapshot.

To prevent drift between host-side policy evaluation and in-world enforcement:
- Before executing a new REPL command, Substrate MUST compute, for the current `world_cwd`:
  - the effective policy snapshot hash, and
  - the effective workspace root (if any; see `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md` DR-09).
- If either the effective snapshot hash OR the effective workspace root differs from what was used to start the current world session, Substrate MUST:
  1) Tear down the current world session.
  2) Start a new world session with the new snapshot.
     - CWD continuity: Substrate SHOULD request the new session start with `cwd` set to the previous session's last known `world_cwd`.
     - If that `cwd` is rejected/invalid under the new session, Substrate MUST start in the new session's resolved root/project directory (as determined by the world backend configuration for that snapshot) and MUST report that the working directory changed.

This restart is a correctness requirement; it is not optional and is not a “legacy mode” fallback.

## Appendix A — Reference Implementation Sketch (Non-Normative)

This appendix provides one concrete architecture sketch that satisfies the MUST-level invariants in this protocol:
- program text is not sent over PTY stdin (command/control separation),
- `stdin_mode` semantics are preserved (EOF vs passthrough),
- and DR-22 is satisfied (the untrusted evaluation context cannot read/write the private control-plane FDs).

This is a recommended reference design, not a normative requirement. Alternative implementations are allowed as long as they satisfy the MUST-level invariants above.

### Reference Sketch: Dedicated trusted driver + isolated evaluation context + persisted “ADR-required state”

Key idea:
- Keep the **trusted driver component** as the only holder of the private control-plane endpoints (FD 8/FD 9).
- Ensure untrusted program evaluation runs in an execution context that never has access to those endpoints (even transiently).

One viable shape:
1) World-agent starts a long-lived **trusted driver component** (recommended: a small Rust helper owned by world-agent).
2) The driver component owns:
   - the private command-control channel (FD 9) and completion channel (FD 8),
   - the session PTY master (for output streaming and for PTY stdin in passthrough mode),
   - and the persistent session state required by ADR-0016 (at minimum: `world_cwd` and exported environment mutations).
3) For each `exec`:
   - The driver component base64-decodes `program_b64` and validates UTF-8 + “no NUL”.
   - It determines stdin wiring:
     - `stdin_mode=eof`: child stdin is `/dev/null`.
     - `stdin_mode=passthrough`: child stdin is the session PTY (host forwards keystrokes via `stdin` frames).
   - It spawns an **isolated evaluation context** that executes the submission and streams output to the PTY.
     - Recommended: spawn `/bin/bash --noprofile --norc` to evaluate the submission, with:
       - cwd set to the driver’s current `world_cwd`,
       - envp set to the driver’s current exported env state plus documented session-level overrides,
       - `SHIM_PARENT_CMD_ID` set only for this exec,
       - and with the reserved control-plane FDs *not present* (closed/never inherited).
4) After the evaluation context completes, the driver component computes and persists the ADR-required state for the next command:
   - update `world_cwd` to the post-exec physical directory (equivalent to `pwd -P` / `getcwd()` semantics),
   - and update the exported env mutations (export/unset) to reflect the completed exec.
   - Note: ADR-0016 explicitly does not require preserving all shell-local state (functions/aliases/traps/options), so the reference design persists only what is required.
5) The driver component emits `command_complete(seq, token, exit, cwd)` to the host.

Security/integrity notes:
- In this reference design, the **completion event** and `(seq, token)` binding come from the trusted driver component, not from user-output parsing.
- The untrusted evaluation context MUST NOT be able to read tokens/future submissions (FD 9) nor spoof completion (FD 8).

### Anti-patterns (do not implement; violate DR-22 and/or core invariants)

These approaches often look plausible but violate the spec:
- **“bash reads commands from FD 9”**: if the bash process that evaluates untrusted submissions can access FD 9, user code can potentially read/peek future submissions/tokens via redirections or `/proc/self/fd` and break the integrity story (DR-22).
- **“FD_CLOEXEC is enough”**: `FD_CLOEXEC` prevents exec-child inheritance, but it does not prevent shell-level redirections in the evaluation process (DR-22).
- **“Send program text over PTY stdin”** (even if “only sometimes”): this violates the command/control separation invariant and reintroduces the class of failure modes this ADR is designed to eliminate.
