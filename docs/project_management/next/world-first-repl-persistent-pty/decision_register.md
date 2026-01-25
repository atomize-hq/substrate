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
- Use a non-interactive `bash` process as the session command interpreter, launched once per REPL session and driven by a world-agent owned driver loop (not an interactive prompt).

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

## DR-08 — Per-command completion protocol (exit status + cwd)

### Option A
- Implement a client-side stdout marker scheme: the session shell emits a boundary marker (with nonce/seq/token/exit/cwd) and the host parses stdout to determine completion.
  - Status: not selected (retained for historical comparison only).

### Option B
- Extend the `/v1/stream` WebSocket protocol with explicit per-command execution and completion messages generated by world-agent (host waits for `command_complete`, not stdout markers).
  - Status: selected.

### Tradeoffs
- A:
  - Pros: no wire protocol changes; client-only iteration.
  - Cons: requires careful framing to avoid collisions and partial-frame issues.
- B:
  - Pros: command completion becomes first-class; avoids marker filtering and spoofing concerns.
  - Cons: increases world-agent complexity and coordination across platforms; requires a clear protocol/versioning story.

### Decision
- Selected: Option B.

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
  - Cons: may lose in-session shell state at restart boundaries (env mutations, history, shell-local state); Substrate attempts best-effort cwd continuity (see DR-17).
- B:
  - Pros: simplest implementation.
  - Cons: policy ambiguity and potential enforcement mismatch; reduces audit clarity for long sessions.
- C:
  - Pros: preserves session state while applying policy changes.
  - Cons: complex, security-sensitive contract; must be proven fail-closed across backends.

### Decision
- Selected: Option A.

## DR-10 — `:host` enablement mechanism and behavior when disabled

Scope:
- This DR applies to the interactive REPL only.
- In `-c/--command` (and other non-interactive flows), `:host` MUST NOT be parsed as a directive and MUST NOT function as a bypass; it is treated as part of the command string.

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
  - each line-editor “submission” is a bounded shell program string (it may contain embedded newlines),
  - Substrate does not rely on bash PS2-style interactive continuation prompts; incomplete shell constructs are evaluated as bounded strings and result in a syntax error rather than blocking the persistent session,
  - PTY passthrough mode supports interactive stdin/TTY programs (e.g., Python REPL, TUIs, password prompts) when explicitly forced or auto-classified as PTY,
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
- Keep interactive stdin support (auto-PTY) while preserving deterministic per-line completion:
  - Line mode (default): Substrate does not forward keystrokes; the command executes with `stdin_mode=eof` (stdin is effectively EOF).
  - PTY passthrough mode (auto or forced): Substrate forwards keystrokes/resizes (raw terminal) via `stdin_mode=passthrough` so TUIs/REPLs/password prompts work.

### Option B
- Always redirect stdin to `/dev/null` in the persistent session (no interactive stdin support).

### Tradeoffs
- A:
  - Pros: preserves current Substrate UX (TUIs “just work”); still prevents line-mode hangs/desync; keeps protocol robust without losing interactivity.
  - Cons: requires correct PTY classification and careful raw-mode handling while stdin is forwarded.
- B:
  - Pros: simplest implementation.
  - Cons: breaks TUIs/interactive programs as normal REPL commands; diverges from existing Substrate behavior.

### Decision
- Selected: Option A.

## DR-15 — Marker candidate detection (reduce false protocol fatals on binary output)

### Decision
- Deprecated by DR-08 (Option B). With explicit `command_complete` frames from world-agent, the host does not parse stdout for completion boundaries.

## DR-16 — Per-command token in the persistent-session protocol (spoof resistance)

### Option A
- Include a per-command random token in the `exec` request and `command_complete` response, and require the host to validate the awaited `(seq, token)` pair.

### Option B
- Do not include a per-command token; rely on nonce + seq only.

### Tradeoffs
- A:
  - Pros: prevents premature completion spoofing; reduces protocol desync and policy/cwd mismatch risks.
  - Cons: slightly more protocol complexity (host must track `(seq, token)` per in-flight command).
- B:
  - Pros: simpler.
  - Cons: allows completion spoofing (DoS and potential policy/cwd mismatch within a session).

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

## DR-19 — Per-REPL-line correlation for in-world subprocess tracing

### Option A
- Treat per-process in-world tracing parity (and per-line correlation into those events) as out of scope for this ADR’s v1 persistent session.
  Track and implement under `docs/BACKLOG.md` **P0 – In-world process execution tracing parity**.

### Option B
- Require v1 to deliver host-level parity: every in-world spawned process is captured and correlated to the originating REPL line span.

### Tradeoffs
- A:
  - Pros: keeps the persistent-session v1 surface area bounded; avoids forcing immediate end-to-end telemetry plumbing changes.
  - Cons: reduced observability for complex in-world workflows until the P0 backlog item ships (even though the protocol provides a clean correlation hook via `SHIM_PARENT_CMD_ID`).
- B:
  - Pros: strongest audit/debug story for persistent sessions.
  - Cons: large scope increase; likely requires new world-agent capture mechanisms and/or protocol support.

### Decision
- Selected: Option A.

## DR-20 — Auto-PTY in the persistent world session

### Option A
- Retain Substrate’s existing “needs PTY” heuristic in the world-first REPL:
  - interactive/TUI commands automatically run in PTY passthrough mode within the persistent session (stdin forwarded, raw terminal),
  - non-interactive commands run in line mode (stdin treated as EOF via `stdin_mode=eof`, no forwarding).

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
- Do not send program text over PTY stdin at all.
  Instead:
  - The host sends programs to world-agent via an explicit `exec` message.
  - World-agent delivers program bytes to the trusted driver component via a private command-control channel that is not the PTY stdin.
  - World-agent receives completion events from the trusted driver component via a private completion channel that is not inherited by user programs.
  - World-agent emits `command_complete` to the host; the host never parses stdout for completion boundaries.

### Option B
- Append marker invocation (or any other control bytes) on PTY stdin and rely on shell grammar / stdin redirection to avoid desync.
  - Status: not selected (retained for historical comparison only).

### Tradeoffs
- A:
  - Pros: prevents interactive programs from consuming REPL control bytes; avoids shell-syntax splice edge cases; supports multiline; makes completion unambiguous and robust against arbitrary stdout bytes.
  - Cons: requires world-agent protocol extension and an internal driver loop; more upfront work.
- B:
  - Pros: simpler framing.
  - Cons: fragile; risks hangs/desync; conflicts with auto-PTY and multiline; completion becomes dependent on shell parsing and user output behavior.

### Decision
- Selected: Option A.

## DR-22 — Control-plane FD privacy (prevent shell-level access to FD 8/FD 9)

### Option A
- Rely on `FD_CLOEXEC` only (FD 8/FD 9 are not inherited by exec’d subprocesses), and accept that user-submitted programs evaluated by the session shell can still access those FDs via redirections (e.g., `>&8`, `<&9`).

### Option B
- Require that **user-submitted programs cannot read/write the private control-plane FDs at all**, including when the program is evaluated by the session shell itself:
  - `FD_CLOEXEC` is necessary but not sufficient.
  - The driver loop must be structured so FD 8/FD 9 are inaccessible during untrusted program execution (implementation-defined mechanism, but MUST be enforced).
  - The implementation SHOULD allocate high-numbered, reserved FDs for the control plane (e.g., `>= 200`) to reduce collisions with user workflows that use explicit low-numbered file descriptors.
  - Invariant reminder: satisfying FD privacy MUST NOT be done by silently weakening the persistent-session contract (e.g., switching to per-command fresh shells that lose ADR-0016 persistence guarantees) unless the ADR/DR is explicitly revised.

### Tradeoffs
- A:
  - Pros: simpler implementation.
  - Cons: breaks the “completion channel not spoofable” integrity claim; allows deliberate session desync/DoS and can cause incorrect per-command completion/cwd tracking.
- B:
  - Pros: makes command completion integrity robust against shell-level redirections; aligns with hardened-driver-loop security posture.
  - Cons: requires more careful driver architecture and explicit tests.

### Decision
- Selected: Option B.
