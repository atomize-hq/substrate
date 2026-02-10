# ADR-0016 — World-First REPL With Persistent World PTY (Host Escape)

## Status
- Status: Draft
- Date (UTC): 2026-01-21
- Owner(s): Substrate maintainers

## Scope
- Feature directory: `docs/project_management/_archived/world-first-repl-persistent-pty/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs
- Plan: `docs/project_management/_archived/world-first-repl-persistent-pty/plan.md`
- Decision Register: `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md`
- Protocol (authoritative): `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`
- State machine (authoritative): `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md`
- Research (historical context): `docs/project_management/_archived/world-first-repl-persistent-pty/RESEARCH.md`
- Driver design (implementation sketch): `docs/project_management/_archived/world-first-repl-persistent-pty/driver_loop_design.md`
- Drain/ordering design: `docs/project_management/_archived/world-first-repl-persistent-pty/drain_design.md`
- Context (previous REPL + world routing behavior):
  - `crates/shell/src/execution/invocation/runtime.rs`
  - `crates/shell/src/execution/routing/dispatch/exec.rs`
  - `crates/shell/src/execution/routing/builtin/utility.rs`
  - `crates/shell/src/execution/routing/builtin/world_deps.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - Note: paths are accurate as of 2026-01-24; if the repo layout changes, search for identifiers like `execute_command`, `handle_builtin`, and `execute_world_pty_over_ws`.

## Executive Summary (Operator)

ADR_BODY_SHA256: b695a30f0ae29abcda629bf32a771fc67c717c9808c222694fdd09337d70b0b8
### Changes (operator-facing)
- Make interactive `substrate` behave like a normal in-world shell by default (persistent world PTY session)
  - Existing: In the REPL, most commands run in the world overlay view, but stateful builtins (`cd`, `pwd`, `export`, `unset`) run on the host and operate on host paths/env; this can yield surprising “exists in world but cd fails” behavior.
- New: The default REPL command path executes inside a persistent world Session PTY (driver-managed; per-submission evaluator shells), so `cd`/`pwd`/`export`/`unset` behave like a normal shell within the same in-world filesystem view the user sees.
  - Why: Align REPL ergonomics with the isolation model and user expectations: a “world REPL” should be internally consistent and not require users to know which tokens are host-only.
  - Links:
    - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md#user-contract-authoritative`
    - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md#architecture-shape`

- Add explicit host escape hatch for interactive sessions (`:host ...`)
  - Existing: Host-only semantics are implicit via builtins and routing, and are not obvious when the overlay view diverges from the host filesystem.
  - New: `:host <command>` runs on the host (current behavior), while unprefixed commands run in the persistent world PTY session.
  - Why: Preserve operator access to host tooling while making the default path world-first and predictable.
  - Links:
    - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md#cli`

- Ensure `-c/--command` is world-consistent when world is enabled
  - Existing: `-c/--command` uses the same “lightweight builtin” fast-path as the REPL, so `cd`/`pwd`/`export`/`unset` can be executed on the host even when the command is otherwise world-backed.
  - New: When world is enabled, `-c/--command` MUST interpret `cd`/`pwd`/`export`/`unset` in-world (shell semantics) and MUST NOT execute them as host-only builtins; `:host` is never recognized in `-c/--command`.
  - Why: Prevent “mixed-context” surprises and avoid accidental host-path evaluation when the operator expects world semantics.
  - Links:
    - `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md#cli`

## Problem / Context
- The current interactive REPL mixes execution contexts:
  - Many commands are executed “in world” (overlay view) via world-agent request/response paths.
  - A small set of “lightweight builtins” (`cd`, `pwd`, `export`, `unset`) are executed by the host process to mutate host cwd/env.
- This yields a user-visible inconsistency: a path can exist in the world overlay view (e.g., created during a prior in-world command), but a subsequent `cd <path>` fails because the host cannot resolve the path on the host filesystem.
- The workaround (`:pty bash`) works because it enters a real shell inside the world, but it is:
  - non-default,
  - not obvious to new users,
  - and does not persist Substrate’s own prompt/cwd/env semantics once the PTY exits.

## Goals
- Default interactive `substrate` REPL behaves like a normal shell *inside the world*:
  - `cd` changes the in-world working directory for subsequent commands.
  - `pwd` reports the in-world working directory.
  - `export`/`unset` affect subsequent in-world commands.
- Non-interactive `-c/--command` is consistent with world-first semantics when world is enabled:
  - `cd`/`pwd`/`export`/`unset` MUST NOT be implemented as host-only “lightweight builtins” when world is enabled.
  - Any state changes from `cd`/`export`/`unset` exist only within the invoked command’s shell/process (standard shell semantics).
- Preserve an explicit host escape hatch for interactive work: `:host <command>`.
- Retain high-signal diagnostics when the world backend is unavailable and `world_fs.require_world=true`.
- Maintain trace/event correctness:
  - Each REPL-entered command produces a trace command span with an accurate execution origin (`world` vs `host`) and exit status.

## Non-Goals
- Broad changes to non-interactive execution (`-c/--command`) beyond eliminating host-only “lightweight builtin” behavior when world is enabled.
- Implementing `world_fs.read_allowlist` enforcement in `world_fs.isolation=workspace` (reads remain unrestricted in workspace isolation).
- Replacing the “needs PTY?” heuristic for non-REPL executions (this ADR is REPL-focused).
- Capturing per-command `fs_diff` for commands executed inside the persistent world session (this requires additional agent/world support and is explicitly deferred).
- Achieving in-world per-process execution tracing parity with host-level shim events (tracked as **P0 – In-world process execution tracing parity** in `docs/BACKLOG.md`).
- Providing Windows parity for interactive world PTY streaming (Windows work, if any, requires a separate platform-specific design).

## User Contract (Authoritative)

### CLI
- Interactive REPL (`substrate` with no `--command`/`-c`):
  - Default: Substrate starts and maintains a persistent in-world world session (WebSocket) + Session PTY (PTY stream) when world execution is enabled and available.
    - Terms: see `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` (World session vs Session PTY).
  - All unprefixed input submissions are executed inside the world session.
  - Auto-PTY is preserved: interactive/TUI commands run in PTY passthrough mode automatically, using the existing “needs PTY” heuristic; `:pty` forces PTY passthrough for a line when heuristics are wrong (see `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md`).
    - If a command is misclassified into line mode but reads from the controlling TTY (e.g., via `/dev/tty`), it may block; `Ctrl+C` is the supported escape, then rerun with `:pty <cmd>`.
    - Operator recovery contract (no fallback): if a line-mode command appears “hung” due to needing a TTY, abort with `Ctrl+C` and rerun with `:pty <cmd>`. Implementations MAY print a non-fatal hint after a short delay (guidance only; not a completion fallback).
  - The prompt is line-editor driven (no interactive PS2 continuation prompts), but a single submission MAY contain embedded newlines (e.g., pastes/multiline input). Submissions are sent to world-agent via explicit `exec` messages (not as raw PTY stdin bytes), and completion is reported via `command_complete`. Job control/backgrounding remains out of scope (see `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md` and decision register DR-13).
    - Output note: PTY `stdout` bytes MAY still arrive while the REPL is idle (e.g., from background writers); Substrate forwards/renders them, but they are **unattributed** to a specific command by default.
    - Output ordering note (MUST): for a given `exec`, `command_complete` MUST NOT be emitted until all foreground Session PTY output bytes for that command have been forwarded to the host. v1 uses a watermark-based post-exit drain barrier (Linux: `ioctl(FIONREAD)`); if the platform cannot support the watermark query, v1 persistent sessions MUST fail closed (see `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`).
    - Concurrent output note (MUST): Session PTY `stdout` frames are raw PTY bytes only (stdout+stderr combined). Substrate-managed concurrent output (e.g., `:demo-agent`, future AgentHub events) MUST NOT be injected into PTY bytes; during PTY passthrough, such structured events SHOULD be buffered and rendered only after the foreground PTY command completes to avoid corrupting TUIs (see `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`).
    - Rendering note (MUST): PTY output is a byte stream and may be non-UTF8. While waiting for input (Reedline active), Substrate MUST use a byte-capable output path to render PTY bytes without corrupting the input buffer; it MUST NOT route PTY bytes through string-only printers.
  - Persistence guarantees (within a single world session):
    - MUST persist: physical in-world working directory (`cd`/`pwd -P` / `getcwd()` semantics) and exported environment mutations (`export`/`unset`) across subsequent submissions (see `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` cwd semantics).
    - MAY persist / not guaranteed: shell-local variables (non-exported), functions, aliases, traps, `set -o` / `shopt` options, and other session-internal shell state.
    - Implementation note: v1 does not require a single long-lived bash interpreter process; persistence may be implemented by a trusted driver component that applies `cwd` + exported env to per-submission evaluator shells.
  - Snapshot-driven restart note: Substrate MUST restart the world session when the effective policy snapshot hash (or workspace root) changes, before executing the next submission (see decision register DR-09 and `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` “Policy Snapshot Drift”). It attempts best-effort cwd continuity, but other in-session state (exported env mutations, history, shell-local state) may be lost (see decision register DR-17).
  - Hardened protocol invariant (DR-22): user-submitted programs MUST NOT be able to access session infrastructure or control-plane endpoints/handles (e.g., inherited `/v1/stream` WebSocket FDs or other session control endpoints). Close-on-exec alone is necessary but not sufficient; the evaluator execution context must not have access in the first place. See decision register DR-22 and `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`.
  - `exit` / `quit`: exits the REPL; Substrate shuts down the world session as part of cleanup.
    - `exit` may include an optional numeric argument (e.g., `exit 2`), but this is treated as an operator request to end the REPL (it is not sent into the world session as a command). The REPL process exit code remains `0` on normal user exit (see `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`).
  - Protocol and state machine are authoritative:
    - `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`
    - `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md`

- Non-interactive single-command mode (`substrate -c <CMD>` / `substrate --command <CMD>`):
  - When world is enabled and available, `<CMD>` MUST execute inside the world (non-PTY by default) and must observe the in-world filesystem view.
  - In this mode, `cd`/`pwd`/`export`/`unset` MUST NOT be executed as host-only builtins when world is enabled; they must be interpreted by the in-world shell/process.
  - When world is disabled, `<CMD>` executes on the host using host shell semantics.
  - `:host` MUST NOT be recognized in this mode.

- Host escape hatch:
  - `:host <command>` is the explicit host escape hatch in the Substrate REPL, but it MUST NOT be available by default.
  - Enablement rules (fail-closed):
    - `:host` MUST be recognized only in the interactive REPL (never in `--command` / `-c`, CI, or agent/automation flows).
    - `:host` MUST require explicit opt-in at REPL startup via a dedicated CLI flag and/or a REPL-only env/config knob. If not enabled, `:host ...` MUST be rejected and MUST NOT execute on host or world.
    - Canonical enablement knobs (REPL-only):
      - CLI: `--repl-host-escape`
      - Env: `SUBSTRATE_REPL_HOST_ESCAPE=1`
  - Directive parsing rule (multiline submissions): REPL directives (`exit`/`quit`, `:host ...`, `:pty ...`) are recognized only when the submission contains no embedded newlines; multiline submissions are treated as program text and are not parsed as directives.
  - `:host cd <path>` / `:host pwd` / `:host export ...` / `:host unset ...` are supported as host operations when `:host` is enabled.
  - Rationale: `:host` is a bypass of world isolation; it must be gated to prevent accidental or programmatic host execution.

- Interactive/TTY passthrough:
  - `:pty <cmd>` forces PTY passthrough mode for a command line (raw terminal + stdin forwarding).
    - When world execution is enabled, `:pty` runs in-world within the persistent session (sharing `world_cwd` and session state).
    - When world execution is disabled via explicit `--no-world`, `:pty` runs on-host using the host PTY execution path.
  - World-enabled fail-closed posture remains: if world execution is enabled but unavailable, the REPL must not fall back to host execution (see decision register DR-18).

- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - The interactive REPL process exit code remains `0` on normal `exit`/`quit`, and non-zero only on startup/config errors that prevent starting the REPL.
  - Per-command exit codes are surfaced via existing REPL printing behavior and recorded in trace spans.

### Config
- Existing world configuration continues to apply:
  - `world.enabled`, `world.anchor_mode`, `world.anchor_path`, `world.caged`
  - `world_fs.isolation`, `world_fs.mode`, `world_fs.require_world`
- A new REPL-only setting MUST exist to gate `:host` enablement (default disabled).
  - It MUST NOT be honored in non-interactive `--command`/`-c` or any CI/agent automation flow.
  - This ADR explicitly disallows any legacy/compatibility mode that restores the old REPL routing behavior (see decision register DR-06).

### Platform guarantees
- Linux:
  - Interactive REPL uses the Linux world backend (world-agent over UDS) when enabled and available.
  - When world execution is enabled (e.g., `--world` or effective config enables world), the REPL must fail closed on startup if the world backend is not available (no implicit host fallback). `world_fs.require_world=true` strengthens this by ensuring world execution cannot be disabled by policy/config.
- macOS:
  - Interactive REPL uses Lima-backed world-agent streaming when enabled and available.
  - When world execution is enabled (e.g., `--world` or effective config enables world), the REPL must fail closed on startup if the world backend is not available (no implicit host fallback). `world_fs.require_world=true` strengthens this by ensuring world execution cannot be disabled by policy/config.
- Windows:
  - No changes required by this ADR; world PTY parity is explicitly out of scope.

## Architecture Shape
- `crates/shell` remains the REPL front-end:
  - Reads user input lines (Reedline).
  - Implements prefix routing for `:host`.
  - Maintains session lifecycle (start/stop) and trace spans per command.

- World session abstraction (new):
  - A long-lived PTY-backed session exists inside the world for the duration of the REPL session (Session PTY + trusted driver component).
  - Substrate submits each REPL “submission” to world-agent as an explicit `exec` request (not as raw PTY stdin bytes) and receives streamed PTY output.
  - Substrate derives per-submission exit status and updates its in-world cwd tracking from world-agent `command_complete` messages.

- World backend requirements (Linux/macOS):
  - The world backend must support a long-lived interactive session (PTY stream) with:
    - a per-session identifier (`ready.session_nonce`, `hex32`) that is freshly generated per `start_session` and changes on restart (observability/correlation only; not a capability/credential; see `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` `ready` semantics),
    - reliable stream framing,
    - and explicit per-command completion messages (e.g., `command_complete`) so the host can obtain an exit code + cwd per submission without parsing stdout markers.

- Compatibility and safety invariants:
  - Host execution remains available only via explicit `:host` routing (no silent host builtins when world is enabled).
  - A command that mutates state (cwd/env) in the world session must have effects visible to subsequent unprefixed commands.

## Sequencing / Dependencies
- This ADR depends on the existing world-agent streaming (`/v1/stream`) and REPL routing layers but introduces a new “persistent session” requirement.
- No explicit triad dependencies are declared in this Draft; once scheduled, this work must be integrated into:
  - `docs/project_management/next/sequencing.json`
  - `docs/project_management/_archived/world-first-repl-persistent-pty/tasks.json`

## Security / Safety Posture
- Default behavior is world-first to avoid accidental host execution while the UI appears “world-like”.
- Fail-closed behavior when world execution is enabled:
  - If world execution is enabled and the world backend is unavailable at REPL startup, Substrate must exit with a clear error (no implicit host fallback).
  - If an operator explicitly disables world execution (e.g., `--no-world`), Substrate runs on-host (this is not a fallback; it is an explicit mode selection).
- `:host` bypass controls:
  - `:host` MUST NOT be recognized in `-c/--command` or any non-interactive flow (CI/automation).
  - `:host` MUST require explicit opt-in at REPL startup and must fail closed when not enabled.
- Observability:
  - Every input line must produce a trace command span with:
    - execution origin (`world` vs `host`),
    - exit code,
    - and policy snapshot metadata when applicable.
  - Any new session-layer protocol must avoid logging secrets and must reuse existing redaction helpers.

## Validation Plan (Authoritative)
- Unit tests:
  - Add tests covering `:host` routing and ensuring no implicit host builtins are used when world is enabled.
  - Add tests covering the “world-only path exists” scenario:
    - Create a directory in the world overlay view.
    - Verify `cd <dir>` works in the REPL default mode and subsequent `pwd` reflects the new directory.
  - Add tests covering `-c/--command` consistency:
    - When world is enabled, `substrate -c "cd <dir>"` must not fail solely because `<dir>` exists only in the world overlay view.
    - Assert `:host` is not recognized in `-c/--command` (must fail closed / treat as a normal command string, not a bypass).
- Integration tests (Linux/macOS):
  - A REPL interaction harness must validate that a sequence of commands maintains in-world cwd state across multiple commands without requiring `:pty bash`.
- Manual playbook:
  - Add a manual playbook under the feature directory verifying:
    - `cd/pwd/export/unset` behave in-world by default.
    - `:host` can run host tooling without changing the in-world session state.

## Rollout / Backwards Compatibility
- Greenfield: legacy REPL behavior is removed.
- No compatibility switch, warnings, or transitional UX is permitted by this ADR.
- No hidden switches (flags/env) are permitted to restore legacy REPL routing behavior (see decision register DR-06).
- Any operator need for host execution must use the explicitly gated `:host` escape hatch described in this ADR.

## Decision Summary
- Decision register: `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md`
- This ADR’s decisions are captured there and referenced during implementation reviews to prevent drift.
