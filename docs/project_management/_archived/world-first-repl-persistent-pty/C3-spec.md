# C3-spec — Interactive REPL routing + lifecycle (host-side; no rendering changes)

This slice wires the interactive REPL to the persistent world session client and implements directive routing and lifecycle behavior. It does not implement the byte-safe Reedline rendering requirements; that is C4.

Authoritative specs (do not diverge):
- `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md` (interactive REPL user contract)
- `docs/project_management/_archived/world-first-repl-persistent-pty/STATE_MACHINE.md` (host behavior, authoritative)
- `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md` (wire protocol, authoritative)
- `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md` (DR-04, DR-10, DR-11, DR-12, DR-13, DR-18, DR-19, DR-20)

Depends on:
- `C2` (persistent session client core).

Out of scope for C3:
- Byte-safe Reedline output rendering and out-of-band stdout while idle — C4.
- Non-interactive `-c/--command` and stdin pipe mode — C5.

## Contract (C3 deliverable)

Interactive REPL lifecycle (world-first):
- When world execution is enabled, the REPL MUST start a persistent world session (`start_session → ready`) before accepting input.
- If world execution is enabled but the world backend is unavailable, the REPL MUST fail closed at startup (no implicit host fallback).
- Unprefixed submissions MUST execute inside the persistent world session via `exec` and MUST complete only on an accepted `command_complete` (no stdout marker parsing; fail closed on protocol violations).

Directive routing (locked):
- `:host <command>` is REPL-only and MUST be gated:
  - It MUST NOT be available by default.
  - It MUST require explicit REPL startup opt-in (`--repl-host-escape` and/or `SUBSTRATE_REPL_HOST_ESCAPE=1`).
  - If not enabled, `:host ...` MUST be rejected and MUST NOT execute on host or world.
- `:pty <cmd>` forces PTY passthrough mode:
  - When world execution is enabled, it runs inside the persistent session (shares `world_cwd` and session persistence).
  - When world execution is disabled (`--no-world`), it runs on-host using the host PTY execution path.
- Directive parsing MUST be single-line only: directives are recognized only when the submission contains no embedded newlines; multiline submissions are treated as program text.

Routing correctness (locked):
- When world execution is enabled, REPL submissions MUST NOT execute host-only builtins (`cd`, `pwd`, `export`, `unset`) on the default path.
- `:host` builtins affect only host state (`host_cwd`, `host_env`) and MUST NOT affect world session persistence.

Policy snapshot drift restart (locked):
- Before executing a new submission, the host MUST recompute the effective policy snapshot hash and effective workspace root for the current `world_cwd`.
- On drift, host MUST restart the world session (best-effort cwd continuity per DR-17).
  - The host MUST emit an operator-visible message when a snapshot/workspace-root drift restart occurs, even if cwd continuity is preserved.
  - If cwd continuity cannot be preserved, the host MUST also emit an operator-visible message that the working directory changed.

Signals (locked; routing only):
- Typed `Ctrl+C` during PTY passthrough is forwarded as a byte (`0x03`) via `stdin` frames; it MUST NOT be translated into a `signal` message.
- Host-originated signals MAY be forwarded via `signal` frames and MUST target the Session PTY foreground process group (agent-side semantics).

## Acceptance criteria
- Interactive REPL matches `STATE_MACHINE.md` state transitions and routing invariants for:
  - session startup fail-closed behavior,
  - directive parsing and gating (`:host`, `:pty`, `exit`/`quit`),
  - persistent session submission/execution and completion,
  - and drift restart.

## Validation (C3-test scope)
Add tests that cover, at minimum:
- `:host` gating (enabled/disabled) and single-line directive parsing rule.
- `:pty` routing rules (world-enabled uses persistent session; `--no-world` uses host PTY path).
- No implicit host builtin path in world-enabled interactive REPL routing.
- Drift restart behavior (restart on effective snapshot/workspace-root changes).
  - Operator-visible restart messaging when drift restart occurs (including the case where cwd continuity is preserved).
