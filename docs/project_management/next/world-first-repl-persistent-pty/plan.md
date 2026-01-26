# Plan — World-First REPL With Persistent World PTY

This plan is anchored by:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md`

## Guardrails (non-negotiable)
- No fallbacks (DR-06): no legacy mode, no hidden switches, no silent host fallback when world is enabled.
- Completion is explicit: `start_session → ready → exec → command_complete` (no stdout marker parsing; fail closed on protocol violations).
- v1 evaluator model: per-submission evaluator shells (`/bin/bash --noprofile --norc`), not a single long-lived interactive bash interpreter (DR-07).
- v1 persistence scope is limited and explicit (ADR-0016 + PROTOCOL v1):
  - MUST persist: physical in-world cwd (`pwd -P` / `getcwd()` semantics) and exported env mutations (`export`/`unset`).
  - Other shell-local state is not guaranteed.
- DR-22 control-plane handle privacy is MUST-level: untrusted evaluator MUST NOT be able to access session infrastructure/control-plane handles/endpoints. Fail closed during `start_session` if this cannot be guaranteed.
- Output ordering is MUST-level (DR-23 + PROTOCOL v1): `command_complete` MUST NOT be emitted until all foreground Session PTY bytes for that command have been forwarded; v1 requires a watermark drain barrier (Linux `ioctl(FIONREAD)`) and MUST fail closed if unsupported (no quiescence/would-block drains; no timing heuristics).
- No pipelining: only one `exec` in flight; concurrent `exec` is fatal protocol error.
- macOS v1 runs through Lima (Linux guest).

## Platform scope (planning pack)
- Behavior platforms (smoke required): `linux`, `macos`
- CI parity platforms (compile/test parity): `linux`, `macos`, `windows`
- WSL: not required for this feature (Windows world PTY parity is out of scope per ADR-0016).

## Slice plan (triads)
- `C0`: world-agent persistent session bootstrap + fail-closed preflight (server-side).
- `C1`: world-agent per-submission `exec` + `command_complete` (server-side).
- `C2`: shell persistent session client core (protocol correctness; no REPL UX yet).
- `C3`: interactive REPL routing + lifecycle (`:host` gating, `:pty`, drift restart; no rendering changes).
- `C4`: interactive REPL byte-safe rendering + structured host output buffering + out-of-band stdout handling.
- `C5`: non-interactive `-c/--command` and stdin pipe mode world-consistency when world is enabled; `:host` never recognized in non-interactive modes.

## Planning Pack artifact index (this directory)
- `docs/project_management/next/world-first-repl-persistent-pty/plan.md`
- `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`
- `docs/project_management/next/world-first-repl-persistent-pty/session_log.md`
- Specs (slice-level): `docs/project_management/next/world-first-repl-persistent-pty/C0-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/C1-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/C2-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/C3-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/C4-spec.md`, `docs/project_management/next/world-first-repl-persistent-pty/C5-spec.md`
- Authoritative spec pack:
  - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
  - `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`
  - `docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md`
  - `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md`
  - `docs/project_management/next/world-first-repl-persistent-pty/driver_loop_design.md`
  - `docs/project_management/next/world-first-repl-persistent-pty/drain_design.md`
  - `docs/project_management/next/world-first-repl-persistent-pty/RESEARCH.md` (historical)
- `docs/project_management/next/world-first-repl-persistent-pty/integration_map.md`
- `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`
- `docs/project_management/next/world-first-repl-persistent-pty/requirements_traceability.md`
- Smoke scripts:
  - `docs/project_management/next/world-first-repl-persistent-pty/smoke/linux-smoke.sh`
  - `docs/project_management/next/world-first-repl-persistent-pty/smoke/macos-smoke.sh`
  - `docs/project_management/next/world-first-repl-persistent-pty/smoke/windows-smoke.ps1`
- Kickoff prompts: `docs/project_management/next/world-first-repl-persistent-pty/kickoff_prompts/`

## Execution Phases (high-level)

1) REPL command routing
- Add `:host` prefix routing and ensure unprefixed commands are world-first.
- Enforce `:host` gating (interactive REPL only): when disabled, `:host ...` must error and must not execute on host or world; in `-c/--command`, `:host` must never be recognized as a directive (treated as a normal command string, not a bypass).
  - Canonical enablement knobs (REPL-only): CLI `--repl-host-escape`, env `SUBSTRATE_REPL_HOST_ESCAPE=1`.
- Make `-c/--command` world-consistent when world is enabled (no host-only builtins for `cd`/`pwd`/`export`/`unset`), since agent/automation integrations are expected to primarily use `-c`.

2) Persistent world session
- Introduce a long-lived world-agent `/v1/stream` PTY session abstraction (initially Linux/macOS).
- Extend the `/v1/stream` WebSocket protocol for persistent REPL sessions with explicit per-command execution and completion messages (`exec` → `command_complete`), so the host never parses stdout to find completion boundaries.
- Implement per-command I/O modes:
  - Line mode: stdin is treated as EOF (implemented in-world by the driver loop, e.g. `</dev/null`) to avoid hangs for stdin-consuming commands.
  - PTY passthrough mode: raw terminal + stdin forwarding for TUIs/interactive programs (auto-PTY).
  - Reuse the existing host-side PTY attach primitives (raw-mode guard, stdin forward loop, resize forwarding) currently used by one-shot world-agent PTY execution.
- Implement the in-world driver loop owned by world-agent:
  - maintain a private command-control channel (separate from PTY stdin) so user programs cannot consume REPL control bytes,
  - maintain a private completion channel (not inherited by user programs) so completion events are not spoofable by untrusted output,
  - support multiline submissions by sending program bytes via the control channel.
  - See `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md` (“Internal Driver Loop (World-Agent Owned)”) for the v1 in-process/no-OS-FD-pipes constraint.
- Add per-command token validation (`seq` + `token_hex`) to bind `command_complete` to the awaited command.
- Preserve in-world cwd across snapshot-driven session restarts when possible.
- Implement restart-on-snapshot-hash-change (and workspace root changes), with explicit operator-visible messaging when a restart occurs and when cwd continuity cannot be preserved.
- Define `:pty` semantics: force PTY passthrough mode (in-world when world enabled; host PTY when `--no-world`).

3) Trace and diagnostics
- Ensure every REPL-entered command produces a trace span with correct `execution_origin`, exit code, and policy snapshot metadata.
- Provide high-signal failure modes when world is required but unavailable.
- Document v1 correlation limits for persistent sessions and ensure the design does not preclude `docs/BACKLOG.md` P0 “in-world process execution tracing parity”.

4) Validation
- Unit tests for routing and state invariants.
- Integration harness for multi-command REPL sessions (cwd/env persistence).
- Manual playbook per ADR.
- Add targeted tests for protocol robustness and security invariants:
  - `command_complete` correlation: mismatched `seq/token_hex` must be treated as a protocol error (fail closed).
  - Binary output containing arbitrary bytes must never interfere with command completion (no stdout marker parsing).
  - Output ordering: prompt/resume input must not occur before all foreground PTY stdout for that command has been forwarded (no “late stdout after command_complete” for non-backgrounded commands). See DR-23 and `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md` (“Output ordering / drain guarantee”, watermark barrier).
  - Out-of-band stdout: `stdout` bytes arriving while no `exec` is in-flight (idle/out-of-band output) must be forwarded/rendered without corrupting the prompt/input buffer, and SHOULD emit an explicit trace event (unattributed; no `cmd_id` guessing).
  - Structured concurrent events during PTY passthrough: structured host/agent messages MUST NOT be injected into the PTY byte stream during `stdin_mode=passthrough`; they SHOULD be buffered and flushed after passthrough ends (e.g., verify `:demo-agent` output does not corrupt a running TUI).
  - Stdin boundary: world-agent must ignore/drop `stdin` frames before `ready`, while `Idle`, and during `stdin_mode=eof` commands (i.e., unless the current command is `stdin_mode=passthrough`), and must drop “late keystrokes” after `command_complete` until the next passthrough command begins.
  - Resize forwarding: terminal resize events must be forwarded to world-agent and must take effect for the in-world foreground program (e.g., a `passthrough` command that prints cols/rows on `SIGWINCH` observes updated values).
  - No pipelining: world-agent must reject concurrent `exec` requests as protocol error; host must not send a new `exec` until the prior completes.
  - Signal targeting: host-originated `SIGINT` must interrupt the currently executing foreground program without killing the session driver loop (Ctrl+C MUST NOT “brick” the REPL).
  - PTY passthrough Ctrl+C semantics: in `stdin_mode=passthrough`, typed `Ctrl+C` must be forwarded as byte `0x03` via `stdin` frames (not translated into a `signal` message).
  - Stdin-consuming commands in line mode (`cat`, `read`): must not hang the session (stdin treated as EOF, e.g. via `/dev/null`).
  - Session-state persistence: `export FOO=bar` then `echo "$FOO"` must print `bar`; `unset FOO` then `echo "$FOO"` must print empty (validates ADR persistence goals).
  - `:pty` shares persistent state: `cd /tmp` then `:pty pwd` must report `/tmp` (enforces DR-12 “:pty runs within the persistent session”).
  - Control-plane handle privacy (DR-22): user submissions attempting to access inherited non-stdio file descriptors/handles (e.g., via `/proc/self/fd` scanning where available and numeric redirections like `echo hi >&$FD` / `<&$FD`) must not be able to reach session infrastructure/control-plane endpoints, spoof completion, read tokens/future submissions, or desync the protocol.
  - Version negotiation: if `ready.protocol_version != 1`, the host must treat the session as unsupported and fail closed with a high-signal error (no partial compatibility).
  - Auto-PTY commands (vim/python REPL): must receive stdin and function interactively in PTY passthrough mode.
  - Directive parsing (multiline submissions): directives are recognized only when the submission contains no embedded newlines; a pasted multiline submission beginning with `:pty`/`:host` must be treated as program text (and must not trigger directive routing).
  - Multiline and incomplete shell syntax: incomplete constructs (e.g., `if true; then`) must produce a syntax error and return to idle (no session hang).
  - Session termination:
    - REPL `exit` and `exit <code>` must cleanly shut down the world session (REPL process exit code remains `0` on normal user exit).
    - Shell-level constructs like `exec ...` within a submission must not hang and must not terminate the persistent session (v1 uses per-submission evaluator shells).
  - Graceful vs unexpected exit: if the host initiates shutdown (`close`), the subsequent `exit` is treated as expected/graceful; if `exit` arrives unexpectedly (no shutdown in progress), the host must fail closed with a high-signal error.
  - `:host` disabled: must error and must not execute.
  - Snapshot drift restart: policy file/workspace root change triggers restart (cwd continuity preserved when possible).
  - Startup cwd resolution: if requested `start_session.cwd` is invalid, world-agent must start in the resolved session root/project dir and `ready.cwd` must reflect that; host must surface the change.
  - Startup snapshot consistency: after `ready`, host recomputes effective snapshot/workspace root for `ready.cwd`; if it differs from what was used to start the session (e.g., because `start_session.cwd` was not honored), the host must restart immediately before accepting input.
  - Error framing: world-agent `error` frames must include a stable `code` and optional `seq`; host must surface and fail closed (enables assertable harness tests).
  - Program rejection cases (explicit, testable):
    - Program contains NUL: `error.code=program_contains_nul`, fail closed.
    - Program bytes not valid UTF-8: `error.code=program_invalid_utf8`, fail closed.
    - `program_b64` invalid base64: `error.code=bad_request`, fail closed.
  - Session initiation: if the first WS frame is not `start` or `start_session`, world-agent must fail the connection with `error.code=bad_request` (no “partial compat”).
  - Exit code semantics: when `SIGINT` terminates the foreground program, `command_complete.exit` SHOULD follow bash conventions (typically `130`).
  - CWD semantics (physical): `command_complete.cwd` must be the physical working directory (equivalent to `pwd -P` / `getcwd()`); e.g., `cd` into a symlinked path and assert `command_complete.cwd` resolves symlinks.
  - Token redaction: `token_hex` MUST NOT be printed in full to the operator terminal; protocol error messaging/traces SHOULD redact or hash tokens (e.g., ensure a token-mismatch failure message does not include the full 32-hex token).
  - Correlation env scoping: `SHIM_PARENT_CMD_ID` must not persist as an exported variable across submissions (e.g., after a command, `env | rg SHIM_PARENT_CMD_ID` SHOULD be empty).
  - Signal test coverage:
    - Line mode: run a long command (e.g., `sleep 10`) in `stdin_mode=eof`, deliver host-originated `SIGINT`, and assert the session recovers with a clean `command_complete` and stable prompt.
    - PTY passthrough: run a raw-input test program in `stdin_mode=passthrough` and assert typed `Ctrl+C` is observed as byte `0x03` (not termination via injected `SIGINT`).

## Dependency ordering (execution)
- `C0` must land before `C1` (session bootstrap is a prerequisite for exec/completion).
- `C1` must land before `C2` (shell client depends on the server contract).
- `C2` must land before `C3` (REPL lifecycle/routing depends on the client).
- `C4` and `C5` may execute concurrently after `C3`:
  - `C4` is REPL-rendering heavy.
  - `C5` is non-interactive routing heavy.
