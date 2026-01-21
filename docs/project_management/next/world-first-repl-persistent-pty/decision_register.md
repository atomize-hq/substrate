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

## DR-06 — No fallbacks (no legacy mode, no hidden switches)

### Option A
- Ship with a hidden or time-boxed fallback (flag/env) that re-enables the legacy REPL routing behavior.

### Option B
- Ship with no fallbacks: no legacy mode, no hidden switches, no feature-flagged “old REPL” path.

### Tradeoffs
- A:
  - Pros: easier emergency mitigation for regressions.
  - Cons: increases long-term maintenance burden and drift risk; undermines the “hard cutover” posture.
- B:
  - Pros: smallest implementation surface area and smallest long-term maintenance burden; enforces a single contract.
  - Cons: increases rollout risk if regressions are discovered late.

### Decision
- Selected: Option B (no fallbacks).

## DR-07 — In-world shell process used for the persistent session

### Option A
- Use a non-interactive `bash` process as the session command interpreter (single long-lived process reading commands from stdin).

### Option B
- Use an interactive `bash -i` session and rely on prompt hooks (e.g. `PROMPT_COMMAND`) to emit command boundary markers.

### Tradeoffs
- A:
  - Pros: deterministic (no rcfiles/prompts required); avoids prompt/PS2 complexity; works with Reedline line-oriented input.
  - Cons: users do not see an in-world shell prompt; multiline/continuation input remains constrained by the host REPL model.
- B:
  - Pros: closer to a “native” shell session model.
  - Cons: prompt hooks can be modified by user configuration; multiline/PS2 behavior is difficult to represent safely via a host line editor; higher risk of desync.

### Decision
- Selected: Option A.

## DR-08 — Command boundary marker protocol (per-line exit status + cwd)

### Option A
- Implement a shell-emitted boundary marker framed by sentinel bytes and containing a fixed, parseable payload (nonce, seq, exit code, cwd).

### Option B
- Extend the `/v1/stream` WebSocket protocol with explicit “command complete” server messages generated by world-agent.

### Tradeoffs
- A:
  - Pros: no wire protocol changes; client-only iteration; marker parsing is deterministic when framed.
  - Cons: requires careful framing to avoid collisions and partial-frame issues.
- B:
  - Pros: command completion becomes first-class; avoids marker filtering and spoofing concerns.
  - Cons: increases world-agent complexity and coordination across platforms; slower to iterate.

### Decision
- Selected: Option A.

## DR-09 — Policy snapshot drift handling for persistent sessions

### Option A
- Restart the persistent world session when the effective policy snapshot hash changes (or when the effective workspace root changes), before running the next command.

### Option B
- Keep the session running with the original snapshot and only apply new policy on the next REPL invocation.

### Option C
- Add an in-session “reconfigure snapshot” control protocol to update enforcement without restarting the shell.

### Tradeoffs
- A:
  - Pros: fail-closed correctness; keeps host evaluation and in-world enforcement aligned.
  - Cons: loses in-session shell state (cwd/env/history) at restart boundaries.
- B:
  - Pros: simplest implementation.
  - Cons: policy ambiguity and potential enforcement mismatch; reduces audit clarity for long sessions.
- C:
  - Pros: preserves session state while applying policy changes.
  - Cons: complex, security-sensitive contract; must be proven fail-closed across backends.

### Decision
- Selected: Option A.

## DR-10 — `:host` enablement mechanism and behavior when disabled

### Option A
- Require explicit REPL startup opt-in via a dedicated CLI flag and/or REPL-only env/config knob; when disabled, `:host ...` is rejected with a clear error and is never executed on host or world.

### Option B
- When disabled, treat `:host ...` as a normal world command string (i.e. attempt to execute a binary named `:host`).

### Tradeoffs
- A:
  - Pros: fail-closed and unambiguous; reduces accidental bypass discovery; avoids confusing “command not found” in-world behavior.
  - Cons: requires one more knob.
- B:
  - Pros: simpler parser rules.
  - Cons: confusing UX; harder to detect accidental bypass attempts; less explicit safety posture.

### Decision
- Selected: Option A.

## DR-11 — Host-side state for `:host` commands

### Option A
- Maintain a host-only `cwd` for `:host` commands, initialized from the host process cwd at REPL start, and mutated only by `:host cd ...`; world cwd is tracked separately.

### Option B
- Do not maintain host cwd; each `:host` command executes in the original REPL launch directory.

### Tradeoffs
- A:
  - Pros: `:host` behaves like a coherent “host context” for operators; `:host pwd` and subsequent `:host` commands are predictable.
  - Cons: introduces additional state that must never be confused with world cwd.
- B:
  - Pros: simpler; no host state.
  - Cons: makes `:host` less useful for real operator workflows.

### Decision
- Selected: Option A.

## DR-12 — `:pty` directive behavior under world-first REPL

### Option A
- Keep `:pty <cmd>` as an explicit “interactive/TTY passthrough” escape for running a command in a one-shot in-world PTY stream (not the persistent session) when full terminal interaction is required.

### Option B
- Reinterpret `:pty` as “attach to the persistent world session” and temporarily hand terminal control to the in-world shell.

### Tradeoffs
- A:
  - Pros: minimal change; preserves known behavior; avoids mixing a line editor and raw TTY in the same session.
  - Cons: `:pty` does not share the persistent session’s cwd/env state.
- B:
  - Pros: best UX for interactive programs; shares session state.
  - Cons: substantially more complex; requires a robust host↔PTY raw mode state machine and clear detachment semantics.

### Decision
- Selected: Option A.

## DR-13 — Multiline input and job control in the world-first REPL

### Option A
- Define the persistent-session REPL contract as single-line commands only; multiline continuations and job control are out of scope and treated as unsupported.

### Option B
- Support multiline and job control inside the persistent session.

### Tradeoffs
- A:
  - Pros: crisp, testable contract; matches the host line-editor model; reduces PTY desync risk.
  - Cons: users cannot rely on PS2-style continuations or backgrounding within the default mode.
- B:
  - Pros: closer to a full terminal shell experience.
  - Cons: large scope increase; requires terminal-first UX and robust job control semantics.

### Decision
- Selected: Option A.
