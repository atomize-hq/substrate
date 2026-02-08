# Research — Builtins, Routing, and State Replacement for World-First REPL

This document captures the codebase sweep and implementation options supporting:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`

## Current Behavior (Observed)

Substrate’s interactive experience mixes execution contexts:
- Many commands (e.g. `ls`, `cat`) are executed in the world overlay view (world-agent).
- A small set of “lightweight builtins” (`cd`, `pwd`, `export`, `unset`) are executed by the host process to mutate host cwd/env.

This creates a mismatch:
- A path can exist in the world overlay view (upperdir), but `cd <path>` fails because it canonicalizes on the host and the host path does not exist.

This mismatch affects:
- interactive REPL
- `-c/--command` (wrap mode)
- pipe mode (stdin-driven execution)

## Code Map — Builtins and Their Call Paths
Note: paths are accurate as of 2026-01-24; if the repo layout changes, search for identifiers like `execute_command`, `handle_builtin`, and `execute_world_pty_over_ws`.

### Builtin interception (host-side)
- Interception happens in shared routing:
  - `crates/shell/src/execution/routing/dispatch/exec.rs` (calls `handle_builtin(...)` when `!needs_shell(trimmed)`).
- Builtin implementations:
  - `crates/shell/src/execution/routing/builtin/utility.rs` (`pwd`, `export`, `unset`, dispatch for `cd`)
  - `crates/shell/src/execution/routing/builtin/world_deps.rs` (`cd` implementation)
  - `crates/shell/src/execution/routing/path_env.rs` (`canonicalize_cd_target` uses `fs::canonicalize`)

### Entry points that all reach `execute_command(...)`
- Sync REPL loop:
  - `crates/shell/src/execution/invocation/runtime.rs`
- Async REPL loop:
  - `crates/shell/src/repl/async_repl.rs`
- `-c/--command` wrap mode:
  - `crates/shell/src/execution/invocation/runtime.rs` (`run_wrap_mode`)
- Pipe mode:
  - `crates/shell/src/execution/invocation/runtime.rs` (`run_pipe_mode`)

## What “State” Means Today (and Who Consumes It)

Today, the “shell state” is the host process:
- cwd state:
  - read via `std::env::current_dir()`
  - mutated via `env::set_current_dir(...)` (builtin `cd`)
- env state:
  - read via `std::env::vars()`
  - mutated via `env::set_var/remove_var` (builtin `export`/`unset`)

Key consumers that assume host cwd/env is authoritative:
- Per-command policy/config resolution:
  - `crates/shell/src/execution/routing/dispatch/exec.rs` uses `cwd_for_profile = env::current_dir()` then calls:
    - `config_model::resolve_effective_config(...)`
    - `substrate_broker::detect_profile(...)` / broker evaluation
    - `policy_snapshot::resolve_policy_snapshot_for_cwd(...)`
- World-agent request building:
  - Non-PTY (`/v1/execute`) request building uses host cwd/env:
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`build_agent_client_and_request_impl`)
    - env uses `build_world_env_map()` from `crates/shell/src/execution/routing/dispatch/shim_ops.rs`
  - PTY WS (`/v1/stream`) client uses host cwd/env:
    - Linux: `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`execute_world_pty_over_ws`) currently uses `std::env::vars().collect()`
    - macOS: `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`execute_world_pty_over_ws_macos`) uses `build_world_env_map()` + normalization
- Trace/event logging records host cwd:
  - `crates/shell/src/execution/routing/telemetry.rs` (`"cwd": env::current_dir()?`)

Implication:
- Achieving “world-first semantics” requires introducing an internal state source that replaces host cwd/env for routing, request building, and telemetry.

## PTY and Streaming Protocol Reality Check (Current Implementation)

### Shell-side PTY behavior today
- `:pty <cmd>` is treated as “force PTY”:
  - `crates/shell/src/execution/routing/dispatch/registry.rs` (`is_force_pty_command`)
- When PTY is selected, Substrate attempts world-agent PTY over WS on Linux/macOS:
  - `crates/shell/src/execution/routing/dispatch/exec.rs`
  - Linux WS client: `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`execute_world_pty_over_ws`)
  - macOS WS client: `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`execute_world_pty_over_ws_macos`)

### World-agent WS PTY protocol today
- Implemented separately from `agent_api_types::ExecuteStreamFrame`:
  - World-agent uses a custom JSON frame protocol in `crates/world-agent/src/pty.rs`:
    - Client frames: `start`, `stdin`, `resize`, `signal`
    - Server frames: `stdout`, `exit`, `error`

This feature adds *new* persistent-session frames (without breaking the existing one-shot `start` flow):
- Persistent REPL sessions use `start_session` + per-submission `exec` and `command_complete` (see `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`).

Important limitation:
- The current client usage is effectively “one interactive command per WS session”.
- A world-first REPL requires a *persistent* session and a *command-boundary protocol* for per-line exit status + cwd without tearing down the session.

## Broker + World-Agent Implications (State, Policy, and Enforcement)

This feature is not “shell-only”: it changes how the shell, broker, and world-agent collaborate to
enforce policy.

### Where the broker participates (host-side)
- Every command path (REPL, `-c/--command`, pipe) routes through:
  - `crates/shell/src/execution/routing/dispatch/exec.rs` (`execute_command`)
- For each command, the shell:
  - resolves effective config for a chosen cwd (`config_model::resolve_effective_config`)
  - loads the profile and runs broker evaluation (`substrate_broker::detect_profile`, `evaluate`)
  - builds a `PolicySnapshotV1` for the world-agent:
    - `crates/shell/src/execution/policy_snapshot.rs` (`resolve_policy_snapshot_for_cwd`)
- This means the “cwd” Substrate uses is security-significant: it controls which workspace policy
  applies and which snapshot is sent to the world-agent.

### Where the world-agent participates (in-world enforcement)
- Non-PTY `/v1/execute` path:
  - Shell sends `ExecuteRequest { cmd, cwd, env, policy_snapshot, world_fs_mode }`
    via `crates/shell/src/execution/routing/dispatch/world_ops.rs`.
  - World-agent consumes that snapshot during request handling (`crates/world-agent/src/service.rs`)
    and derives enforcement (mount strategy, Landlock allowlists, writable prefixes).
- PTY `/v1/stream` path:
  - Shell uses a custom WS PTY protocol (`crates/world-agent/src/pty.rs`) where the `start` frame
    includes `cmd`, `cwd`, `env`, and `policy_snapshot`.
  - The snapshot in practice becomes “session configuration” for the world preparation that
    happens before the interactive child starts.

### The hard constraint for persistent sessions
A persistent world PTY session introduces snapshot drift risk:
- Policy and config are resolved per command today (host-side) based on cwd and on-disk policy files.
- A long-lived persistent session has one `start_session.policy_snapshot` unless we add an update mechanism.

Therefore, any “persistent world session” design must explicitly choose one:

1) Restart-on-change
- If the effective policy snapshot hash changes (or if the workspace root changes), tear down the
  current world session and start a new one with the new snapshot.

2) Session reconfigure protocol
- Extend the WS PTY protocol to allow updating the effective snapshot (and any derived enforcement)
  mid-session (requires a careful, fail-closed contract for what can change and when).

3) Avoid persistent PTY as the default
- Keep per-command `/v1/execute` as the primary execution path and maintain state in the shell
  (virtual state approach), so each command naturally carries the correct snapshot.

This choice is central to correctness and is now locked by decision register DR-09 (restart-on-change) and `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` (“Policy Snapshot Drift”).

## Options for Replacing Host-State Builtins

These are implementation options for how Substrate “replaces the state” currently maintained by host builtins.

### Option A (recommended under ADR constraints): Persistent Session PTY + per-submission evaluator shells + explicit `command_complete` protocol

Idea:
- Start a persistent world session PTY owned by world-agent and keep it alive for the REPL duration.
- Use a trusted driver component to enforce DR-22 (control-plane handle privacy) and mediate execution.
- Extend `/v1/stream` so world-agent can accept per-submission `exec` messages and emit structured `command_complete` messages.
- Use a small in-world “driver loop” owned by world-agent so:
  - user programs cannot consume REPL control bytes (command text is not sent over PTY stdin),
  - and completion events cannot be spoofed by user output.
- Evaluate each submission via an untrusted evaluator shell (`/bin/bash --noprofile --norc`) attached to the session PTY; persist only the ADR-guaranteed state across submissions (at minimum: physical cwd + exported env).

State replacement:
- Substrate maintains `WorldSessionState` (at minimum: `cwd`, `last_exit`) by waiting for `command_complete` from world-agent.
- Substrate uses that internal `cwd` for:
  - policy/config resolution (workspace detection, snapshot derivation)
  - world-agent request `cwd` for subsequent commands
  - trace logging (`cwd` field)

Pros:
- Shell semantics for each submission (bash interpreter), with persistence for the ADR-guaranteed state (`cd/pwd` + exported env).
- Overlay-only paths behave as expected.

Cons / risks:
- PTY echo/line discipline complexity for a line-editor prompt.
- Requires world-agent protocol changes and a hardened driver loop.
- Requires a correlation story for shim logs within a persistent session (the protocol provides this cleanly via per-command `cmd_id` propagated as `SHIM_PARENT_CMD_ID`).

### Option B: Virtual state in Substrate + per-command non-PTY `/v1/execute`

Idea:
- Keep a line-editor REPL; maintain `virtual_cwd` + `virtual_env` inside Substrate.
- Execute each command via `/v1/execute` with those fields.
- Implement `cd/pwd/export/unset` as *world-side semantics* by updating the virtual state, not host state.

Pros:
- No PTY raw-mode/echo problems.
- Clear per-command spans and existing `/v1/execute` shape stays primary.

Cons:
- Re-implements a shell state model; edge cases grow over time.
- Less faithful “normal shell” experience unless the virtual shell becomes a full interpreter.

### Option C: Make the interactive mode “attach to world shell” (terminal-first)

Idea:
- Make interactive `substrate` essentially always attach to an in-world shell (like always doing `:pty bash`).

Pros:
- Most natural shell experience; TUIs “just work”.

Cons:
- Conflicts with a line-editor REPL UX and makes a gated `:host` escape hatch harder to express safely.
- Per-line Substrate spans become less meaningful (shim spans become primary).

## Concrete Changes Required Regardless of Option

1) Remove host builtins from the default path when world is enabled
- `crates/shell/src/execution/routing/dispatch/exec.rs` must stop applying `handle_builtin(...)` (for `cd/pwd/export/unset`) in world-enabled contexts.

2) Introduce an internal “effective cwd” for routing, request building, and telemetry
- `crates/shell/src/execution/routing/dispatch/exec.rs` must not rely on `env::current_dir()` as the authoritative cwd for policy/config when running world-first.
- `crates/shell/src/execution/routing/dispatch/world_ops.rs` must build requests using the internal state rather than host cwd/env.
- `crates/shell/src/execution/routing/telemetry.rs` logs the internal cwd (not host cwd) for world-first modes.

3) Implement `:host` gating as REPL-only opt-in
- Parsing must occur in REPL loops only (sync + async):
  - `crates/shell/src/execution/invocation/runtime.rs`
  - `crates/shell/src/repl/async_repl.rs`
- `:host` must never be recognized in `-c/--command` or non-interactive modes.

## Recommendation (historical)

If the priority is “Substrate feels like a normal shell inside the world” with correct semantics:
- Prefer Option A (persistent Session PTY + per-submission evaluator shells + explicit `command_complete` protocol) and accept the PTY/session complexity.

If the priority is fastest correctness with minimal PTY work:
- Prefer Option B (virtual state + per-command exec) and accept that the REPL is not a full shell.

## Next Decisions to Lock (historical; resolved)

The major design choices that were previously open here are now locked by the decision register:
- Line-editor REPL + multiline submissions (no PS2 continuation): see DR-13.
- Persistent PTY-backed session with explicit `exec` → `command_complete` (no stdout marker parsing): see DR-08 and `PROTOCOL.md`.
- Driver-loop control plane separation (no program text over PTY stdin): see DR-21.
- Control-plane handle privacy beyond close-on-exec: see DR-22.
- Auto-PTY + `:pty` passthrough behavior: see DR-12 and DR-14.
- `:host` scope and gating (REPL-only; never in `-c`): see DR-10.

Unresolved items are tracked as explicit DR entries or issues (not implied as “pending” here).
