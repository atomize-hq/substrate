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
- Implement a shell-emitted boundary marker framed by sentinel bytes and containing a fixed, parseable payload (nonce, seq, per-command token, exit code, cwd).

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
- Keep `:pty <cmd>` as an explicit “force PTY” directive that runs the command in PTY passthrough mode (raw terminal, stdin forwarded).
  - When the persistent world session is active, `:pty` forces PTY passthrough within that session (so it shares `world_cwd` and world session state).
  - When world execution is disabled, `:pty` runs the command on the host using the host PTY execution path (same as current host-only behavior).

### Option B
- Keep `:pty` as a one-shot separate PTY execution path (a separate `/v1/stream` invocation) and do not share persistent session state.

### Tradeoffs
- A:
  - Pros: preserves current Substrate UX where TUIs “just work” as normal commands; maintains world-first state continuity for interactive commands; allows explicit forcing when heuristics are wrong.
  - Cons: requires careful terminal mode switching and command-boundary handling while stdin is forwarded.
- B:
  - Pros: simpler persistent session model; avoids raw passthrough state inside the same session.
  - Cons: breaks state continuity (cwd/env) for interactive commands; diverges from current Substrate “auto-PTY” behavior.

### Decision
- Selected: Option A.

## DR-13 — Multiline input and job control in the world-first REPL

### Option A
- Define the persistent-session REPL contract as:
  - line-editor submissions are single-line (no PS2 continuations in line mode),
  - PTY passthrough mode supports interactive multiline input for both invoked programs (e.g., Python REPL) and shell continuations/heredocs when explicitly forced or auto-classified as PTY,
  - job control/backgrounding is out of scope and treated as unsupported.

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

## DR-14 — Stdin contract for the persistent world session

### Option A
- Execute each user command line with an explicit per-command I/O mode:
  - Line mode: stdin redirected to `/dev/null` to prevent hangs for stdin-consuming commands.
  - PTY passthrough mode: stdin forwarded to support TUIs and interactive programs.

### Option B
- Always redirect stdin to `/dev/null` in the persistent session (no interactive stdin support).

### Tradeoffs
- A:
  - Pros: prevents hangs in line mode while retaining current Substrate “auto-PTY” behavior for interactive commands; keeps the protocol robust without losing TUI support.
  - Cons: requires correct PTY classification and careful raw-mode handling during passthrough.
- B:
  - Pros: simplest implementation.
  - Cons: breaks TUIs/interactive programs as normal REPL commands; diverges from existing Substrate behavior.

### Decision
- Selected: Option A.

## DR-15 — Marker candidate detection (reduce false protocol fatals on binary output)

### Option A
- Only treat a `0x1E ... 0x1F` framed segment as a marker candidate if its payload begins with `SUBSTRATE_CMD_END\t2\t`.
  If the prefix does not match, forward the bytes unchanged as normal output.

### Option B
- Treat any `0x1E ... 0x1F` framed segment as a marker candidate and fail the session on any validation error.

### Tradeoffs
- A:
  - Pros: reduces false-positive protocol errors when user commands output arbitrary bytes (including control characters).
  - Cons: requires a slightly more complex parser; relies on prefix uniqueness in addition to sentinel framing.
- B:
  - Pros: simplest parser; strict fail-closed behavior.
  - Cons: brittle in the presence of binary output that happens to include sentinel bytes; can cause unnecessary session failures.

### Decision
- Selected: Option A.

## DR-16 — Per-command token in the marker protocol (spoof resistance)

### Option A
- Include a per-command random token in the marker invocation (appended to the submitted command line) and marker payload, and require the host to validate the awaited `(seq, token)` pair.

### Option B
- Do not include a per-command token; rely on nonce + seq only.

### Tradeoffs
- A:
  - Pros: prevents a command from printing a valid-looking marker early to induce premature completion; reduces protocol desync and policy/cwd mismatch risks.
  - Cons: slightly more complexity in the session bootstrap and marker parser; requires a protocol version bump.
- B:
  - Pros: simpler.
  - Cons: allows early-marker spoofing (DoS and potential policy/cwd mismatch within a session).

### Decision
- Selected: Option A.

## DR-17 — CWD continuity on snapshot-driven session restarts

### Option A
- On restart due to policy snapshot drift, request the new session start with `cwd` equal to the previous session’s last known in-world cwd.
  If the requested cwd is invalid/rejected, start in the new session’s resolved project/root directory and report the cwd change.

### Option B
- Always restart into the new session’s resolved project/root directory (do not attempt cwd continuity).

### Tradeoffs
- A:
  - Pros: preserves operator location across restarts; reduces “jarring” restarts while keeping enforcement correct.
  - Cons: requires explicit handling of invalid cwd under the new session and a clear UX message when cwd changes anyway.
- B:
  - Pros: simpler implementation.
  - Cons: less usable; restarts feel arbitrary even when the prior cwd would have been valid.

### Decision
- Selected: Option A.

## DR-18 — `:pty` policy snapshot + availability semantics

### Option A
- `:pty` recomputes policy snapshot when world execution is enabled (same pre-step as other world commands).
- When world execution is disabled (explicit `--no-world`), `:pty` runs on the host using the host PTY execution path.
- When world execution is enabled but unavailable, the REPL must fail closed (no implicit host fallback).

### Option B
- `:pty` is world-only and errors when world execution is disabled.

### Tradeoffs
- A:
  - Pros: matches existing Substrate behavior in host-only mode while keeping fail-closed posture when world execution is selected; avoids stale policy for world PTY.
  - Cons: increases surface area of `:pty` semantics (world vs host depends on explicit world selection).
- B:
  - Pros: simplest safety story (“:pty is always world”).
  - Cons: diverges from current host-only behavior; less useful when world is explicitly disabled.

### Decision
- Selected: Option A.

## DR-20 — Auto-PTY in the persistent world session

### Option A
- Retain Substrate’s existing “needs PTY” heuristic in the world-first REPL:
  - interactive/TUI commands automatically run in PTY passthrough mode within the persistent session (stdin forwarded, raw terminal),
  - non-interactive commands run in line mode (stdin redirected, no forwarding).

### Option B
- Disable auto-PTY and require explicit `:pty` for interactive/TUI commands.

### Tradeoffs
- A:
  - Pros: preserves current Substrate UX; avoids surprising regressions where `vim`/`lazygit`/`python` stop working as normal commands.
  - Cons: requires careful terminal mode switching; depends on heuristic correctness (mitigated by `:pty` forcing).
- B:
  - Pros: simpler execution model.
  - Cons: regressions vs current behavior; higher cognitive load for operators.

### Decision
- Selected: Option A.

## DR-21 — Command submission framing (prevent marker bytes becoming stdin)

### Option A
- Submit each REPL line as a brace-framed compound command that includes the marker invocation on the closing line (so bash parses the marker before starting the user command):
  - `{\n <user_line>\n } ... ; __substrate_cmd_end <seq> <token>\n`
  - Line mode adds `</dev/null` on the closing line; PTY passthrough omits it.

### Option B
- Append marker invocation as a separate subsequent stdin line (`<user_line>\n` then `__substrate_cmd_end ...\n`) and rely on stdin redirection to prevent consumption.

### Tradeoffs
- A:
  - Pros: prevents marker bytes from being consumed as stdin by interactive programs while still allowing stdin forwarding in PTY passthrough mode.
  - Cons: slightly more complex framing; introduces a small set of shell-syntax edge cases (e.g., user-provided unmatched braces).
- B:
  - Pros: simpler framing.
  - Cons: incompatible with auto-PTY/stdin forwarding; risks hangs/desync when stdin is forwarded.

### Decision
- Selected: Option A.

## DR-19 — Per-REPL-line correlation for in-world subprocess tracing

### Option A
- Treat per-process in-world tracing parity (and per-line correlation into those events) as out of scope for this ADR’s v1 persistent session.
  Track and implement under `docs/BACKLOG.md` **P0 – In-world process execution tracing parity**.

### Option B
- Require v1 to deliver host-level parity: every in-world spawned process is captured and correlated to the originating REPL line span.

### Tradeoffs
- A:
  - Pros: keeps the persistent-session v1 surface area bounded; avoids forcing immediate world-agent protocol/event work.
  - Cons: reduced observability for complex in-world workflows until the P0 backlog item ships.
- B:
  - Pros: strongest audit/debug story for persistent sessions.
  - Cons: large scope increase; likely requires new world-agent capture mechanisms and/or protocol support.

### Decision
- Selected: Option A.
