# Decision Register — World-First REPL With Persistent World PTY

This decision register supports:
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`

## DR-01 — Host escape prefix syntax

### Option A
- Use `:host <command>` as the explicit host escape hatch in the Substrate REPL.

### Option B
- Use `host: <command>` as the explicit host escape hatch in the Substrate REPL.

### Tradeoffs
- A:
  - Pros: aligns with existing Substrate REPL directive style (`:pty ...`, `:demo-agent`); easy to parse; does not resemble shell syntax.
  - Cons: not a typical shell idiom; requires learning one more “directive” convention.
- B:
  - Pros: resembles other tools’ “namespace prefix” idioms; visually obvious.
  - Cons: conflicts with common shell parsing assumptions; less consistent with existing Substrate `:` directives.

### Decision
- Selected: Option A (`:host <command>`).

## DR-02 — Default REPL semantics (world-first implementation strategy)

### Option A
- Persistent in-world PTY-backed session is the default REPL execution model; Substrate runs a long-lived shell in the world and evaluates each input line within that session.

### Option B
- Virtual “world state” is maintained by Substrate without a persistent PTY; Substrate uses per-command `/v1/execute` requests and updates its internal cwd/env state by running helper commands (e.g., `pwd`) to simulate shell state.

### Tradeoffs
- A:
  - Pros: matches user expectations of a normal shell; `cd/export` semantics are naturally correct; fewer special cases.
  - Cons: requires robust command-boundary protocol to extract per-command exit codes without terminating the session; interactive programs and job control require careful handling; may complicate per-command fs diff capture.
- B:
  - Pros: preserves current per-command execution model, exit codes, and fs diff collection; easier to keep per-command trace fidelity.
  - Cons: “shell semantics” must be reimplemented (cwd/env tracking, quoting edge cases); still risks surprises if Substrate state diverges from the world.

### Decision
- Selected: Option A (persistent PTY-backed world session).

## DR-03 — Legacy compatibility switch

### Option A
- Provide an opt-in legacy mode (env/config) that restores the current behavior for debugging and regression bisects only; default remains world-first.

### Option B
- No legacy mode; ship the behavior change as a hard cutover.

### Tradeoffs
- A:
  - Pros: faster operator unblocking if regressions occur; provides a path for phased adoption.
  - Cons: additional code paths to maintain; risk of “permanent compat” if not time-boxed.
- B:
  - Pros: simpler implementation and testing matrix.
  - Cons: higher rollout risk; fewer tools for diagnosis during early adoption.

### Decision
- Selected: Option B (no legacy mode).

## DR-04 — Gating `:host` (prevent automation bypass)

### Option A
- `:host` is available only in the interactive REPL and requires an explicit REPL startup opt-in (flag/config/env); it is never honored in `--command` / `-c` or CI/automation modes.

### Option B
- `:host` is always available in the REPL (no extra opt-in), but is not honored in `--command` / `-c`.

### Tradeoffs
- A:
  - Pros: prevents accidental host execution and prevents agents/CI scripts from using `:host` as a bypass surface; aligns with “world-first by default” posture.
  - Cons: adds one more knob and documentation burden; requires operator intent to use host escape.
- B:
  - Pros: simpler UX for operators; fewer knobs.
  - Cons: increases the risk of unintended host execution during interactive use (and increases the chance that automation discovers and uses the bypass in unexpected ways via pseudo-interactive wrappers).

### Decision
- Selected: Option A (explicit opt-in; REPL-only; never in non-interactive/CI).

## DR-05 — Non-interactive `-c/--command` semantics when world is enabled

### Option A
- Keep current behavior: `-c/--command` uses the same “lightweight builtin” fast-path, so `cd`/`pwd`/`export`/`unset` may execute on the host even when the command is otherwise world-backed.

### Option B
- When world is enabled, `-c/--command` MUST NOT execute `cd`/`pwd`/`export`/`unset` as host-only builtins; they must be interpreted in-world (shell semantics), and `:host` is never recognized in `-c/--command`.

### Tradeoffs
- A:
  - Pros: minimal implementation change; preserves current performance shortcuts.
  - Cons: mixed-context surprises; host-path evaluation can fail for overlay-only paths; violates “world-first” mental model.
- B:
  - Pros: consistent world semantics; avoids accidental host-path evaluation; removes a class of confusing failures.
  - Cons: requires routing changes and tests; `cd` in a one-shot command has only intra-command effect (standard shell semantics).

### Decision
- Selected: Option B.
