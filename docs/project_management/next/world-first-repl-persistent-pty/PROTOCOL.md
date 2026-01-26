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
- **Session PTY**: the long-lived in-world PTY stream for the REPL session (stdout/stderr bytes streamed to the host; stdin reserved for user keystrokes during PTY passthrough).
- **Trusted driver component**: an implementation-defined component owned by world-agent that holds the private control-plane state and mediates execution/completion; it is trusted infrastructure, not user-submitted code.
- **Driver loop**: the control loop inside the trusted driver component that:
  - receives per-submission programs over the private command-control channel,
  - executes them under the stdin-mode semantics below,
  - and reports structured completion events to the host.
- **Evaluator shell**: the untrusted shell interpreter used to evaluate a submission program (v1 requires `/bin/bash --noprofile --norc`).
- **hex32**: 32 lowercase hex characters (16 bytes).
- **Session nonce**: a world-agent-generated, per-session random identifier (`hex32`) returned as `ready.session_nonce`.
  - It is intended for observability and correlation (e.g., tying out-of-band `stdout` or errors to a specific persistent session instance, including across restart boundaries).
  - It is not an authentication secret and MUST NOT be used as a capability/credential.

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

Host rendering note:
- Because `stdout` is a raw byte stream (may be non-UTF8) and may arrive while the host line editor is active, host rendering MUST follow `docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md` (`Idle` → “Out-of-band world PTY output”).

Important separation (host concurrent output vs PTY bytes):
- This protocol’s `stdout` stream is the **session PTY byte stream** only.
- Substrate-managed concurrent output on the host (e.g., `:demo-agent`, future AgentHub events) MUST NOT be injected into the PTY byte stream.
  - In PTY passthrough mode, the host SHOULD buffer structured events and render them only after the foreground PTY command completes, so TUIs/REPLs are not corrupted by interleaved host text.
  - See `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`.

Compatibility note:
- Existing one-shot PTY execution over `/v1/stream` is not replaced by this ADR.
- If world-agent does not support `start_session`/`exec`, the world-first REPL MUST fail closed (no fallback to legacy client-side parsing).
- In the interactive REPL, auto-PTY and `:pty` use `stdin_mode=passthrough` within the persistent session; one-shot `/v1/stream` remains for non-REPL contexts.

Session initiation (mode distinction):
- The first client frame on a `/v1/stream` WebSocket connection MUST be exactly one of:
  - `{"type":"start", ...}` (legacy one-shot mode), or
  - `{"type":"start_session", ...}` (persistent REPL session mode, `protocol_version=1`).
- World-agent MUST treat any other first frame as `error.code=bad_request` (or equivalent fatal error) and MUST fail/close the session.

## Evaluator Shell
The evaluator shell MUST be `/bin/bash` with deterministic invocation:
- `/bin/bash --noprofile --norc`

World-agent SHOULD suppress in-world prompt output by setting:
- `PS1=""`, `PS2=""`, and `PROMPT_COMMAND=""` in the evaluator shell environment (and any other in-world shell contexts used to execute submissions).

Evaluation model (v1):
- To satisfy the control-plane handle privacy requirements in this protocol (DR-22), world-agent MUST evaluate each `exec` in an untrusted evaluator context that does not have access to session control-plane endpoints or other session infrastructure.
- In v1, the trusted driver component MUST evaluate each `exec` by spawning a fresh evaluator shell process attached to the Session PTY.
  - The trusted driver component MUST persist and re-apply the ADR-0016 guaranteed state across submissions (at minimum: physical cwd + exported env).
  - Persistence scope note (v1): no other shell-local state is guaranteed to persist across submissions (e.g., aliases, functions, traps, `set -o` / `shopt`, non-exported vars, history, or job control). See `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md` (“Persistence guarantees”) and `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md` (DR-07).

## Key Design Invariant: Separate Command Channel vs User Stdin
Persistent sessions must support auto-PTY (interactive stdin forwarding) without allowing interactive programs to consume REPL control bytes.

Therefore:
- The host MUST NOT send per-submission program text over the PTY stdin stream.
- Program text MUST be sent over a dedicated **command-control channel** that is not the PTY stdin.
- The PTY stdin stream is reserved for **user keystrokes** during PTY passthrough mode.

This eliminates the “REPL control bytes consumed by stdin” failure mode and avoids shell-syntax splice edge cases (`#` comments, trailing `;`, etc.).

## Internal Driver Loop (World-Agent Owned)
In protocol version 1, the **trusted driver component** is an **in-process world-agent component** (not a separate helper process).

World-agent MUST:
1) Allocate one Session PTY (stdout/stderr stream to the host; stdin used for user keystrokes during passthrough).
2) Initialize the trusted driver component (in-process) and wait until it is ready to accept `exec`s, then send `ready`.
3) For each `exec`, decode and validate `program_b64` in the WebSocket handler (see `exec` validation rules below), and then deliver the decoded UTF-8 program string and per-command metadata to the trusted driver component via an internal command-control interface (e.g., an in-memory queue). The program text is never sent over PTY stdin.
4) Ensure the untrusted evaluator process execution context cannot access session infrastructure or control-plane endpoints (see DR-22). In particular, the evaluator process MUST NOT inherit:
   - the `/v1/stream` WebSocket connection,
   - any internal world-agent session control endpoints used to coordinate `exec`/`command_complete`,
   - or any other file descriptors/handles that would allow untrusted code to spoof completion, read future submissions/tokens, or desynchronize the session.

### Control-plane handle privacy (DR-22)
This protocol relies on the **session control plane being private to world-agent**.

Therefore, it is a MUST-level requirement that user-submitted programs cannot:
- write bytes that are interpreted as a completion event, nor
- read/peek future submissions, tokens, or other control-plane state.

If a user submission attempts to access inherited non-stdio file descriptors (e.g., via `/proc/self/fd` where available, or via shell redirections to numeric FDs), it MUST fail harmlessly and MUST NOT cause premature `command_complete` acceptance or protocol desynchronization.

Implementation guidance (non-normative but posture-aligned):
- The evaluator process should be spawned with a minimal FD table: only `stdin/stdout/stderr` plus the PTY slave and any explicit `stdin_mode=eof` redirections required for the evaluator.
- Selected v1 mechanism (Linux, normative): the evaluator process MUST NOT inherit any non-stdio file descriptors/handles.
  - On Linux, this MUST be enforced by closing all file descriptors other than the explicitly required stdio/PTY fds in the child before `exec` (e.g., `close_range(3, ~0)` or an equivalent “close everything” mechanism).
  - If the platform/runtime cannot provide an equivalent guarantee in-process, world-agent SHOULD use a separate spawn helper (Option B portability fallback) that itself starts with a minimal FD table and then `exec`s the evaluator with only the required fds (the trusted driver component remains in-process; the helper exists only to spawn the evaluator with a minimal FD table).

Fail-closed posture (v1):
- If world-agent cannot guarantee the required minimal FD table / non-inheritance properties for the evaluator process on the current platform/runtime, it MUST fail closed during `start_session` (before emitting `ready`), rather than starting a session that violates DR-22.

Invariant reminder:
- Meeting the control-plane handle privacy requirement MUST NOT be achieved by silently weakening ADR-0016’s persistence guarantees (at minimum: physical cwd + exported env persistence across submissions) or the REPL’s auto-PTY contract, unless the ADR/decision register is explicitly revised.

### Execution semantics (stdin modes)
For each `(seq, token_hex)`:
- The driver loop MUST set `SHIM_PARENT_CMD_ID=<cmd_id>` for the duration of executing the program, so in-world shim logs can be correlated to the host command span.
- `SHIM_PARENT_CMD_ID` MUST be applied only to the foreground program evaluation and its descendants, and MUST NOT persist as a user-visible exported variable across subsequent submissions (the driver MUST restore/unset it after the command completes).
- The driver loop MUST ensure the ADR-0016 persistence guarantees hold across subsequent `exec`s:
  - physical working directory (`cd`/`pwd` semantics), and
  - exported env mutations (`export`/`unset` semantics).
  The implementation is allowed to be driver-managed (recommended for v1) rather than relying on a single long-lived shell interpreter.
- The driver loop MUST ensure the executed program cannot access session control-plane endpoints or other session infrastructure (DR-22).
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
`start_session.env` is the **full environment** (`envp`) used to initialize the persistent session state, and to launch evaluator processes for `exec`s.

Therefore:
- World-agent MUST treat `start_session.env` as authoritative by default: evaluator processes MUST start from a cleared environment and then apply exactly `start_session.env` (plus documented session-level overrides below).
- World-agent MUST NOT implicitly inherit environment variables from the world-agent process when launching evaluator processes.
- World-agent MAY apply documented session-level overrides needed for correctness/UX (e.g., forcing `PS1/PS2/PROMPT_COMMAND` empty to suppress in-world prompts, and normalizing XDG paths to a writable in-world location when necessary).
- World-agent MUST strip shim/runtime control variables from the persisted session environment (defense-in-depth and correctness), at minimum:
  - `SHIM_ACTIVE`, `SHIM_CALLER`, `SHIM_CALL_STACK`, `SHIM_DEPTH`.
  - Rationale: these may be present in the host process environment due to shim interception and must not leak into the persistent in-world session state.

#### `cwd` resolution (startup)
`start_session.cwd` is a requested starting working directory for the persistent session state (and for the first evaluator invocation).

Definition: “resolved session root/project directory”
- This refers to the world backend’s anchor/project root for the session (often called `project_dir` in the codebase), i.e., the root directory that the world filesystem view is anchored to.
- World-agent MUST derive this root using the same anchor semantics as the backend configuration (see `docs/CONFIGURATION.md` “anchor mode/path”):
  - If `SUBSTRATE_ANCHOR_MODE=custom`: use `SUBSTRATE_ANCHOR_PATH` (required).
  - If `SUBSTRATE_ANCHOR_MODE=follow-cwd`: use the requested `start_session.cwd`.
  - If `SUBSTRATE_ANCHOR_MODE=project` (default): use `SUBSTRATE_ANCHOR_PATH` if set; otherwise use the requested `start_session.cwd`.

World-agent MUST:
- attempt to initialize the persistent session working directory to `start_session.cwd`, and
- if it cannot honor `start_session.cwd` (e.g., path does not exist, is outside the world root/cage, or is rejected by backend/policy constraints), start the session in the **resolved session root/project directory** for that world session (e.g., workspace root / configured anchor root, consistent with the provided `policy_snapshot`), and return that as `ready.cwd`.
- Additionally, if `start_session.cwd` is outside the resolved session root/project directory, world-agent MUST start the session in the resolved session root/project directory and return that as `ready.cwd`.

The host MUST treat `ready.cwd` as authoritative for prompt/state.
If `ready.cwd` differs from the requested `start_session.cwd`, the host MUST emit an operator-visible message that the session started in a different working directory (and record it in trace metadata).

### `ready`
World-agent MUST reply:
- `{"type":"ready","session_nonce":<hex32>,"cwd":<path>,"protocol_version":1}`

The host MUST wait for `ready` before accepting user commands.

`session_nonce` semantics:
- MUST be freshly generated for each accepted `start_session` (i.e., it MUST change across session restarts).
- SHOULD be recorded in host trace/session metadata for correlation (see Terms).

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
- reject any program containing NUL (`0x00`) bytes.
  - Rationale: NUL cannot be safely represented in the common POSIX process-string interfaces (`execve` argv/envp are NUL-terminated), and it creates ambiguity/fragility in program delivery mechanisms (argv/env/script-file/text) and observability tooling. v1 treats a submission as a UTF-8 program string without embedded NUL.
- and then deliver the decoded program to the trusted driver component, which executes it in an evaluator shell context under the persistent session state.

Non-interactive execution note:
- The persistent-session REPL contract MUST NOT rely on interactive shell continuation prompts (PS2). Incomplete constructs must fail as a bounded submission (syntax error) and return to `Idle` (see DR-13 and `plan.md` validation).

### `command_complete`
World-agent MUST send:
- `{"type":"command_complete","seq":<u64>,"token_hex":<hex32>,"exit":<i32>,"cwd":<path>}`

Exit code semantics:
- `exit` MUST reflect the evaluator shell’s standard `$?`/wait-status semantics for the just-finished foreground submission.
- If the submission terminates due to a signal, `exit` SHOULD follow bash conventions (typically `128 + signal_number`, e.g., `SIGINT` → `130`) so audit/replay expectations are stable.

Working directory (`cwd`) semantics:
- `cwd` MUST be the **physical** working directory of the persistent session state after the submission completes (i.e., symlinks resolved; equivalent to `pwd -P` / `getcwd()` semantics).
- `cwd` MUST be an absolute path string.

Path namespace requirement:
- All `cwd` values (`ready.cwd` and `command_complete.cwd`) are **world-absolute paths in the session’s filesystem view**.
- They MUST be in the same path namespace as `start_session.cwd` (so the host can use `world_cwd` for policy/workspace resolution without requiring the path to exist on the host filesystem).
- The host MUST NOT `fs::canonicalize()` or otherwise require host-side existence of `cwd` for policy snapshot resolution; doing so reintroduces the original “exists in world but cd fails” class of bugs.

#### Output ordering / drain guarantee (PTY stdout vs `command_complete`)
For a given `exec(seq, token_hex)`:
- World-agent MUST preserve PTY byte ordering when emitting `stdout` frames.
- World-agent MUST NOT emit `command_complete(seq, token_hex, ...)` until it has forwarded all PTY output bytes produced by that command’s foreground execution.

This requirement exists to prevent post-completion output from interleaving with the next REPL prompt or Reedline input rendering.

Persistent-session v1 ordering barrier (watermark-based, not quiescence):
- After observing evaluator termination for the foreground command, world-agent MUST snapshot a PTY-read watermark representing “bytes readable at exit” and MUST drain at least that many PTY bytes (forwarding them as `stdout` frames) before emitting `command_complete`.
  - Linux: use `ioctl(FIONREAD)` on the PTY master to obtain `available_bytes_at_exit`.
  - If the platform cannot support a watermark query needed for this barrier, persistent sessions under `protocol_version=1` MUST fail closed on that platform (do not substitute timing heuristics or “drain until would-block/quiescence”).
  - Selected v1 behavior (fail-closed early): world-agent MUST validate watermark-query support during `start_session` before emitting `ready`, and MUST fail the session early with a fatal `error` if the barrier cannot be satisfied.
- world-agent MUST NOT wait for global PTY quiescence (e.g., “drain until would-block”) as a prerequisite for `command_complete`, because continuous out-of-band writers can prevent quiescence indefinitely.

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
In persistent sessions, `signal` is intended to control the currently executing *foreground program*, not the trusted driver component itself.

Important distinction:
- During `stdin_mode=passthrough`, typed control keys (including `Ctrl+C` / `0x03`) are transported as `stdin` bytes. The host MUST NOT translate typed keystrokes into `signal` messages; the remote PTY line discipline / foreground program decides what those bytes mean.

Therefore:
- Before `ready` is sent, world-agent MUST ignore/drop all `signal` messages.
- While no command is running, world-agent MUST ignore/drop all `signal` messages.
- While a command is running, world-agent MUST deliver the signal to the **foreground process group** of the session PTY (or the closest equivalent on the platform).
  - World-agent MUST NOT target session infrastructure (including world-agent itself) for interactive control signals like `SIGINT`, because that can terminate the persistent session and cause a fatal session loss.
  - The driver loop/session PTY setup MUST ensure the “currently executing command” runs in a distinct foreground process group so that `SIGINT` interrupts the active program without terminating the session.

Operational intent:
- `SIGINT` must interrupt the currently executing foreground command and the session must remain usable afterward (host continues waiting for `command_complete`).
- `SIGTERM`/`SIGHUP` should terminate the currently executing foreground command when possible; session termination should use `close` / WebSocket shutdown.

### `close` / `exit`
- On REPL shutdown, the host SHOULD send `{"type":"close"}` and close the WebSocket.
- World-agent MAY send `{"type":"exit","code":<i32>}` if the persistent session infrastructure exits (driver/PTY teardown).

Expected vs unexpected session exit:
- If the host has initiated shutdown (sent `close` and/or is intentionally closing the WebSocket), receiving `exit` is expected and SHOULD be treated as a graceful shutdown acknowledgement.
- If world-agent sends `exit` while the host is not shutting down (i.e., the session ends unexpectedly, especially while a command is in-flight), the host MUST treat it as fatal and fail closed.

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
- The persistent session infrastructure exits unexpectedly (`exit`).
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
- and DR-22 is satisfied (the untrusted evaluation context cannot access session infrastructure/control-plane endpoints).

This is a recommended reference design, not a normative requirement. Alternative implementations are allowed as long as they satisfy the MUST-level invariants above.

### Reference Sketch: Dedicated trusted driver + isolated evaluation context + persisted “ADR-required state”

Key idea:
- Keep the **trusted driver component** as an in-process world-agent component that owns the session PTY and all control-plane state.
- Ensure untrusted program evaluation runs in an execution context that never has access to session control-plane endpoints (even transiently).

One viable shape:
1) World-agent initializes a long-lived **trusted driver component** (in-process).
2) The driver component owns:
   - the session PTY master (for output streaming and for PTY stdin in passthrough mode),
   - and the persistent session state required by ADR-0016 (at minimum: `world_cwd` and exported environment mutations).
3) For each `exec`:
   - The WebSocket handler base64-decodes `program_b64` and validates UTF-8 + “no NUL”.
   - It determines stdin wiring:
     - `stdin_mode=eof`: child stdin is `/dev/null`.
     - `stdin_mode=passthrough`: child stdin is the session PTY (host forwards keystrokes via `stdin` frames).
   - It spawns an **isolated evaluation context** that executes the submission and streams output to the PTY.
     - Recommended: spawn `/bin/bash --noprofile --norc` to evaluate the submission, with:
       - cwd set to the driver’s current `world_cwd`,
       - envp set to the driver’s current exported env state plus documented session-level overrides,
       - `SHIM_PARENT_CMD_ID` set only for this exec,
       - and with no access to session control-plane endpoints or other session infrastructure (e.g., no inherited WebSocket FDs).
4) After the evaluation context completes, the driver component computes and persists the ADR-required state for the next command:
   - update `world_cwd` to the post-exec physical directory (equivalent to `pwd -P` / `getcwd()` semantics),
   - and update the exported env mutations (export/unset) to reflect the completed exec.
   - Note: ADR-0016 explicitly does not require preserving all shell-local state (functions/aliases/traps/options), so the reference design persists only what is required.
5) The driver component emits `command_complete(seq, token_hex, exit, cwd)` to the host.

Security/integrity notes:
- In this reference design, the **completion event** and `(seq, token_hex)` binding come from the trusted driver component, not from user-output parsing.
- The untrusted evaluation context MUST NOT be able to read tokens/future submissions nor spoof completion (DR-22).

### Anti-patterns (do not implement; violate DR-22 and/or core invariants)

These approaches often look plausible but violate the spec:
- **“The evaluator can see session infrastructure”**: if the evaluator process can access world-agent session infrastructure (e.g., inherited WebSocket FDs or internal control endpoints), user code can potentially spoof completion, read/peek control-plane state, or desynchronize the protocol (DR-22).
- **“Send program text over PTY stdin”** (even if “only sometimes”): this violates the command/control separation invariant and reintroduces the class of failure modes this ADR is designed to eliminate.
