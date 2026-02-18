# Integration Map — World-First REPL With Persistent World PTY

This integration map is anchored by:
- `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`
- `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md`
- `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md`

## Scope
- Implement world-first interactive REPL semantics via a persistent in-world Session PTY and explicit completion protocol (no stdout marker parsing).
- Add REPL-only host escape hatch `:host <command>` with explicit startup gating.
- Make non-interactive `-c/--command` (and stdin pipe mode) world-consistent when world is enabled (no host-only builtins).

## Out of scope
- Windows parity for interactive world PTY streaming (ADR-0016).
- Job control/background attribution for out-of-band Session PTY output (v1 forwards bytes but does not attribute them).
- New runtime fallbacks/legacy modes (DR-06).
- Replacing the existing “needs PTY” heuristic (DR-20 retains it).

## End-to-end flow (interactive REPL, world enabled)
Inputs → derived state → actions:
1) Inputs
   - CLI: world enablement (`--world` / `--no-world`), `--repl-host-escape`, `:host`, `:pty`, `exit`/`quit`.
   - Environment: `SUBSTRATE_REPL_HOST_ESCAPE=1` (REPL-only), world anchor env (`SUBSTRATE_ANCHOR_MODE`, `SUBSTRATE_ANCHOR_PATH`), trace/log env.
   - Policy/config: effective policy snapshot derived from `world_cwd` and workspace root detection.
2) Derived state (host)
   - `world_cwd` (world-absolute, physical, from `ready.cwd` and `command_complete.cwd`).
   - `host_cwd` and `host_env` for `:host` commands (independent from `world_cwd`).
   - `world_session` metadata: session nonce, snapshot hash, workspace root.
3) Actions
   - Start session: host → world-agent `start_session` with `cwd`, `env`, `policy_snapshot`, terminal size.
   - Wait for `ready` (fail-closed version negotiation).
   - Per submission: host → `exec` with `(seq, token_hex, cmd_id, stdin_mode, program_b64)`.
   - Stream output: world-agent → `stdout` (raw Session PTY bytes).
   - Completion: world-agent → `command_complete` (exit + physical cwd); host validates `(seq, token_hex)`.
   - Drift restart: host compares effective snapshot hash + workspace root; restarts session on change.

## Component map (what changes where)

World-agent (server-side, C0/C1):
- `crates/world-agent`
  - C0: session bootstrap:
    - accept `start_session` as the first frame (or legacy `start` for one-shot),
    - validate and emit `ready` (fail-closed version negotiation),
    - validate DR-22 and DR-23 preconditions before `ready` (fail closed if not satisfiable).
  - C1: per-submission execution:
    - accept `exec` and emit `command_complete` (no stdout marker parsing),
    - spawn per-submission evaluator shells (`/bin/bash --noprofile --norc`),
    - persist ADR-guaranteed state (physical cwd + exported env),
    - enforce DR-22 (evaluator cannot access session infrastructure/control-plane handles),
    - enforce DR-23 (watermark drain barrier via `ioctl(FIONREAD)`; fail closed if unsupported),
    - forward `stdin` only in passthrough mode, and target `signal` to the Session PTY foreground process group.

Shell (host-side, C2/C3/C4/C5):
- `crates/shell`
  - C2: persistent session client core:
    - `start_session` handshake and `ready` validation (`protocol_version=1`),
    - sequential `exec` submission and `(seq, token_hex)` validation,
    - fail-closed on protocol violations and unexpected `exit`,
    - `stdout` as raw bytes forwarded without modification.
  - C3: interactive REPL routing and lifecycle:
    - world-first by default when world is enabled,
    - `:host` directive gating and host-only state (`host_cwd`, `host_env`),
    - `:pty` semantics within persistent session (or host PTY in `--no-world`),
    - policy snapshot drift restart behavior.
  - C4: rendering and concurrent output:
    - byte-capable PTY output rendering while Reedline is active,
    - out-of-band `stdout` rendering while idle,
    - buffering of structured host output during PTY passthrough.
  - C5: non-interactive routing:
    - `-c/--command` and stdin pipe mode use in-world shell semantics for `cd`/`pwd`/`export`/`unset` when world is enabled,
    - `:host` is never recognized in non-interactive flows.

Shared/types (as needed):
- `crates/agent-api-types` (if a shared frame schema is introduced for persistent sessions; otherwise world-agent uses its `/v1/stream` JSON schema as specified in `PROTOCOL.md`).
- `crates/common` for redaction/log schema if new log fields are added (token redaction requirement).

## Cross-track dependencies
- Concurrent host output routing: `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` (structured events MUST NOT be injected into PTY bytes; buffering during passthrough).
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (no overrides declared for this feature).

## Engineering pivots (non-runtime; aligned with DR-06)
These are build-time/implementation pivots only. They MUST NOT be silently enabled at runtime as fallbacks.
- Driver implementation pivot: replace ptrace-based state capture with an alternative, but only if DR-22 remains satisfied and ADR persistence guarantees remain intact (`driver_loop_design.md` §4).
- Drain implementation pivot: reorganize driver/handler event ordering, but preserve the single ordered stream and the DR-23 watermark barrier (`drain_design.md` §5).

## Sequencing alignment
- This feature is registered in `docs/project_management/next/sequencing.json` under id `world_first_repl_persistent_pty`.
- Slice dependencies:
  - `C1` depends on `C0`.
  - `C2` depends on `C1`.
  - `C3` depends on `C2`.
  - `C4` depends on `C3`.
  - `C5` depends on `C3` (and may execute concurrently with `C4`).
