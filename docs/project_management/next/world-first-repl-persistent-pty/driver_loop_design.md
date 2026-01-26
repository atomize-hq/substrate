## 0) Grounding: what exists today vs what v1 must become

  - World-agent one-shot PTY today: /v1/stream currently supports only a legacy first frame {"type":"start", ...} and then streams PTY bytes
    (stdout) plus a final exit (crates/world-agent/src/pty.rs:84, crates/world-agent/src/pty.rs:569). It forwards stdin/resize/signal directly
    to the spawned child PID (crates/world-agent/src/pty.rs:629), not to a foreground process group.
  - Shell REPL today: both sync and async REPLs use Reedline and rely on ExternalPrinter<String> to print concurrent “agent events” without
    corrupting the input buffer (crates/shell/src/execution/invocation/runtime.rs:34, crates/shell/src/repl/async_repl.rs:42). That printing
    pipeline is string-based (crates/shell/src/execution/agent_events.rs:24).
  - Policy/workspace anchoring: policy snapshot is computed from a host-side cwd and cached; workspace root detection walks ancestors looking
    for .substrate/workspace.yaml (crates/shell/src/execution/policy_snapshot.rs:63, crates/shell/src/execution/workspace.rs:40).
  - Shim correlation: shims read SHIM_PARENT_CMD_ID and log it as parent_cmd_id (crates/shim/src/context.rs:12, crates/shim/src/logger.rs:61).
  - Authoritative v1 contract: persistent /v1/stream sessions must add start_session/exec/ready/command_complete, keep program text off PTY
    stdin, preserve stdin_mode=eof|passthrough, and enforce control-plane handle privacy (DR-21/DR-22) (see `PROTOCOL.md` and
    `decision_register.md`).

  The hard part is DR-22: untrusted user submissions must not be able to read/write the control-plane endpoints at all, even from the shell
  itself (no “evaluator can see session infrastructure and we hope they won’t redirect to it”) (see PROTOCOL.md “Control-plane handle
  privacy (DR-22)”).

  ———

  # 1) Reference architecture (concrete + viable with PTY + bash semantics)

  ### Summary of the recommended shape

  Use an in-process trusted driver loop (owned by world-agent) that:

  - owns the session PTY,
  - runs each submission in a separate untrusted bash evaluation process attached to the same PTY,
  - persists only the ADR-required state across submissions (physical cwd + exported env),
  - emits command_complete only after PTY output for the foreground command is drained/forwarded.

  This matches PROTOCOL.md’s evaluation model (per-submission evaluator shells with driver-managed persistence) while satisfying the MUST-level
  invariants: no stdout marker parsing, no command bytes over PTY stdin, control-plane handle privacy, and watermark-based post-exit drain
  before `command_complete`.

  ———

  ## 1.1 Process model diagram (world side)

                   (in-world) world-agent /v1/stream session handler (single process)
                     ┌────────────────────────────────────────────────────────┐
  WS (JSON frames)   │  - validates protocol v1                                │
  <───────────────►  │  - fail-closed on errors                                │
                     │  - owns Session PTY (master+slave)                      │
                     │  - runs trusted driver loop (in-process)                │
                     │  - spawns per-exec untrusted evaluator shells           │
                     │  - emits an ordered stream of stdout + command_complete │
                     └───────────────────────────────┬────────────────────────┘
                                                     │ stdio = PTY slave
                                                     ▼
                                           untrusted evaluator (per exec)
                                           /bin/bash --noprofile --norc
                                           evaluates the submission program

  Key point (DR-22): the evaluator process must not have access to session infrastructure or control-plane handles (e.g., must not inherit the
  `/v1/stream` WebSocket FD). If user code can see such handles (including via `/proc/self/fd`), it can attempt to spoof completion or desync
  the protocol.

  ———

  ## 1.2 Control-plane handle privacy (no OS-level “FD 8/FD 9”)

  v1 does not specify OS-level “FD 8/FD 9” pipes. Instead, control-plane separation is achieved by:
  - Keeping session control-plane state in-process in world-agent, and
  - Spawning the untrusted evaluator process in an execution context that cannot access session infrastructure / control-plane handles.

  Practical invariant (normative in PROTOCOL.md):
  - The evaluator process MUST NOT inherit world-agent’s WebSocket FD or any other session control-plane endpoints/handles.
  - Attempts to access inherited non-stdio file descriptors (including numeric redirections and `/proc/self/fd` enumeration where available)
    must fail harmlessly and must not affect protocol correctness.

  ———

  ## 1.3 How submissions execute while preserving persistence (cd + exported env)

  ### Driver-internal persistent state

  - session_env: map of exported env vars that should apply to subsequent commands (starts from start_session.env; then updated after each
    exec)
  - session_cwd: physical, world-absolute cwd (starts from ready.cwd; then updated after each exec)
  - reserved_env_keys: at minimum includes SHIM_PARENT_CMD_ID (must never persist) and should also strip shim “bypass” variables (SHIM_ACTIVE,
    SHIM_CALLER, SHIM_CALL_STACK, SHIM_DEPTH) to avoid accidentally disabling shim tracing across commands (host-side already removes those
    per command in today’s runner code, e.g. cmd.env_remove("SHIM_ACTIVE") patterns).

  ### Evaluator launch per exec

  For each exec(seq, token_hex, cmd_id, stdin_mode, program):

  1. Build env_for_exec = session_env + { SHIM_PARENT_CMD_ID = cmd_id } (and session-level PS1/PS2/PROMPT_COMMAND suppression per protocol).
  2. Set working directory to session_cwd.
  3. Spawn /bin/bash --noprofile --norc to run the submission program:
      - Program delivery: recommended to avoid argv leakage by using a memfd/temp file and executing it as a script file; the important
        invariant is “not via PTY stdin”.
      - PTY wiring:
          - Always attach stdout+stderr to the session PTY slave.
          - If stdin_mode=passthrough: attach stdin to PTY slave.
          - If stdin_mode=eof: make stdin effectively EOF by attaching fd0 to /dev/null, while still keeping the controlling TTY as the PTY
            (so /dev/tty exists if a program insists; this matches the protocol’s intent of EOF-by-default without hanging on stdin
            consumers).
  4. Put the evaluator into its own foreground process group on the PTY (required so signal targets pgrp and doesn’t kill the driver) (docs/
     project_management/next/world-first-repl-persistent-pty/PROTOCOL.md, “signal targeting semantics”).

  ### Capturing post-exec state safely (physical cwd + exported env)

  This is the key to making persistence correct without trusting user output.

  Recommended mechanism (Linux guest, so applies to Linux and macOS-via-Lima):

  - Use ptrace exit-stop (e.g., PTRACE_O_TRACEEXIT) so that just before the evaluator fully exits (while it still has an mm and cwd), the
    driver can read:
      - /proc/<pid>/cwd → physical cwd (kernel-backed, symlink-resolving)
      - /proc/<pid>/environ → exported env snapshot (this matches the “export/unset persist” requirement exactly)
  - Then update:
      - session_cwd = cwd_from_proc
      - session_env = environ_from_proc minus reserved_env_keys
  - Compute exit:
      - normal: evaluator exit status
      - signaled: 128 + signal_number (bash convention; aligns with protocol guidance) (docs/project_management/next/world-first-repl-
        persistent-pty/PROTOCOL.md, “command_complete”).

  This produces persistence that matches the design goal and avoids “untrusted code writes its own state record” pitfalls.

  ———

  ## 1.4 Output, ordering, and command completion (no stdout marker parsing)

  ### Why ordering is tricky

  Protocol requires: do not emit command_complete until all PTY bytes for that foreground command have been forwarded (docs/
  project_management/next/world-first-repl-persistent-pty/PROTOCOL.md, “Output ordering / drain guarantee”).

  ### Concrete ordering strategy (recommended)

  - Driver owns PTY master and reads it continuously.
  - Driver sends an internal event stream to world-agent:
      - stdout(bytes…) events (raw PTY bytes)
      - command_complete(seq, token_hex, exit, cwd) event
  - Driver emits command_complete only after:
      - it has observed evaluator termination, and
      - it has performed the post-exit PTY drain barrier (prefer a watermark-based drain, not “drain-to-quiescence”), then sends command_complete.

  Because stdout and completion share one ordered channel, world-agent can forward:

  - each stdout as {"type":"stdout","data_b64":...}
  - command_complete as {"type":"command_complete",...}

  …and ordering is preserved by construction.

  ———

  ## 1.5 stdin, resize, signal handling (and what is visible when)

  World-agent must implement the acceptance rules in the protocol (see `PROTOCOL.md` “stdin acceptance / boundary rules” and “signal targeting
  semantics”):

  - Drop stdin before ready, while idle, while stdin_mode=eof, and after command_complete until next passthrough.
  - Drop signal before ready and while idle; while running, target foreground process group (not a specific PID) (see `PROTOCOL.md` “signal
    targeting semantics”).
  - Always forward resize to the PTY owner (driver).

  Driver responsibilities:

  - Track whether current command is passthrough to decide whether to write stdin bytes to PTY master.
  - Maintain foreground_pgid so signals can be delivered with kill(-pgid, SIGINT) semantics.
  - Ensure signals do not hit driver/session infrastructure.

  ———

  ## 1.6 Session startup / world mounting / anchor semantics (reuse existing world-agent reality)

  Use the same “enter world overlay and enforce mount rules” machinery that today’s one-shot PTY path uses:

  - world-agent already uses PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT (crates/world/src/exec.rs:71) to establish the mount namespace and then
    exec sh -lc "$SUBSTRATE_INNER_CMD" inside it.
  - For persistent sessions, the session handler must keep the per-session world context alive for the duration of the REPL session (rather
    than “one command per world entry”). The trusted driver loop is an in-process component of that session handler; it is not a separate
    helper process.

  This keeps the new session semantics aligned with current in-world PTY execution behavior and avoids inventing a new world setup pathway.

  ———

  # 2) Two viable approaches + tradeoffs + recommendation

  ## Approach A (recommended): trusted driver + per-exec evaluator + kernel-derived persistence

  What it is

  - Exactly the reference architecture above.
  - Evaluator is “untrusted”; driver persists only cwd + exported env across execs.
  - State capture uses kernel sources (/proc, ptrace exit-stop).

  Pros

  - Satisfies DR-22 cleanly: evaluator cannot access session infrastructure / control-plane handles.
  - Satisfies “program text off PTY stdin” (DR-21) without fragile shell-marker tricks.
  - Persistence requirement is met precisely for the stated minimum (cd/pwd + exported env).
  - Output ordering and no-stdout-parsing completion story is robust.
  - Naturally enforces “SHIM_PARENT_CMD_ID per command” by injection into evaluator env only (and stripping from persisted env afterward),
    matching shim expectations (crates/shim/src/logger.rs:61).

  Cons

  - Does not preserve all shell-local state (aliases/functions/traps/history/job control). This must be explicitly documented as out-of-
    contract v1 (consistent with the protocol appendix’s “persist ADR-required state” guidance).
  - Requires careful driver engineering around ptrace/process groups/PTY management.

  Recommendation

  - Choose this as v1. It is the most straightforward way to meet DR-22 without relying on brittle bash-internal behavior.

  ———

  ## Approach B (also viable): trusted driver + per-exec evaluator + in-process state capture via LD_PRELOAD (no ptrace)

  What it is

  - Same high-level structure as A (evaluator cannot access session infrastructure/control-plane handles).
  - Instead of ptrace, preload a trusted shared library into the evaluator bash that intercepts:
      - chdir/fchdir
      - setenv/unsetenv/putenv/clearenv
  - The library reports “current cwd” and env mutations to the driver over a private, authenticated channel (e.g., a socketpair whose peer is
    only in the driver).

  Pros

  - Avoids ptrace complexity and ptrace policy variability.
  - Can stream env/cwd changes as they occur (driver can update state even if process is about to die).

  Cons

  - More moving parts (build + ship shared library into the world image / filesystem view).
  - Authentication still needs care: user code can write to any file descriptor it has, so the reporting channel must be protected (e.g.,
    passed in via a channel the evaluator cannot access). This tends to reintroduce DR-22-like control-plane handle privacy problems unless
    designed carefully.
  - Harder to reason about across different distros/libc edge cases.

  When to pick

  - If ptrace is disallowed in the world environment (security policy, seccomp profiles, etc.) and you want a deterministic alternative.

  ———

  # 3) Tricky / friction points (implementation “gotchas”)

  1. Raw PTY bytes vs Reedline printing
      - The protocol requires forwarding PTY output bytes “unchanged” (docs/project_management/next/world-first-repl-persistent-pty/
        PROTOCOL.md, “stdout” semantics), but today’s concurrent-print mechanism is ExternalPrinter<String> (crates/shell/src/repl/async_repl.rs:10), which
        cannot losslessly represent arbitrary bytes.
      - Selected v1 approach: implement a new byte-capable terminal write path for world stdout while Reedline is active; do not assume PTY output is UTF-8.
      - This is likely the highest-friction host-side issue.
  2. Foreground process group correctness
      - World-agent currently forwards signals to a PID in one-shot mode (crates/world-agent/src/pty.rs:663), but v1 requires targeting the
        PTY foreground process group to avoid killing the driver/session (docs/project_management/next/world-first-repl-persistent-pty/
        PROTOCOL.md, “signal targeting semantics”).
      - Without correct setpgid + tcsetpgrp, Ctrl+C and host signal messages will be flaky or session-killing.
  3. Output-drain guarantee
      - If stdout forwarding and completion forwarding are in different tasks without explicit sequencing, you will violate the “no
        command_complete before foreground output is forwarded” invariant (docs/project_management/next/world-first-repl-persistent-pty/
        PROTOCOL.md, “Output ordering / drain guarantee”).
      - This is why the “single ordered event channel from driver → agent” is recommended.
  4. Policy snapshot drift restarts will reset in-world env
      - Per DR-09, session restart on snapshot/workspace-root drift is mandatory (docs/project_management/next/world-first-repl-persistent-
        pty/decision_register.md:160).
      - That restart necessarily resets the driver state; the state machine already calls out that exported env may be lost on restart (docs/
        project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:93).
      - The UX must make this explicit (operator-visible message) and tests must assert restart behavior.
  5. Reserved env vars that must not persist
      - SHIM_PARENT_CMD_ID must never persist across submissions (docs/project_management/next/world-first-repl-persistent-pty/
        PROTOCOL.md, “Execution semantics (stdin modes)”).
      - If the driver uses “persist entire exported env” naïvely, SHIM_PARENT_CMD_ID will persist because it’s exported in the evaluator env.
        You must explicitly strip it before updating session_env.
  6. Exit/exec/logout/kill $$
      - In a true long-lived shell interpreter model (decision register DR-07 Option B), these can kill the persistent session infrastructure.
      - Under the selected v1 model (per-exec evaluator shells; decision register DR-07 Option A), these are not “session-terminating” because the evaluator is disposable:
          - `exit` ends that submission with an exit code (bare `exit` is reserved as a REPL directive unless embedded in multiline program text),
          - `exec ...` replaces the evaluator process for that submission,
          - `kill $$` kills the evaluator, and the driver reports the resulting exit status.

  ———

  # 4) Design alternatives / backup plans (engineering pivots, not runtime fallbacks)

  These are “if Approach A proves too complex to implement”, not things you silently fall back to at runtime (DR-06).

  1. Make the session evaluator a true long-lived bash interpreter
      - Only acceptable if you can provably satisfy DR-22 (user code can’t access session infrastructure/control-plane handles even inside
        that bash).
      - In practice, any design where the evaluator can access session infrastructure (including inherited file descriptors and
        /proc/self/fd discovery) is extremely hard to harden against redirections, DEBUG traps, `read -u`, etc.
      - Reminder: v1 explicitly does not require a long-lived interpreter; the selected model is per-submission evaluators with driver-managed persistence (see `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md` DR-07).
  2. Replace ptrace with a “self-stop barrier” wrapper
      - Have a trusted wrapper around evaluation that ensures the evaluator process stops (SIGSTOP) at a known completion boundary before
        exiting, so the driver can read /proc/<pid>/{cwd,environ}.
      - This is simpler than ptrace but is vulnerable to user code that execs/exits before the barrier runs (which may be acceptable if
        treated as “command did not complete → fatal” and fail-closed, but it changes semantics).
  3. Keep program text out of argv
      - Use memfd or an anonymous temp file for the program, execute as bash /proc/self/fd/N.
      - This is recommended regardless, to avoid leaking program text into process listings and future “in-world process execution tracing
        parity” plumbing (docs/BACKLOG.md:8).

  ———

  # 5) Explicit invariants + how to test them (mapped to plan bullets)

  Below are “MUST” invariants plus a concrete test strategy (unit/integration), aligned to docs/project_management/next/world-first-repl-
  persistent-pty/plan.md (see its validation bullet list).

  ## Protocol + sequencing invariants

  1. No pipelining
      - Agent rejects exec while another is running (error.code=exec_while_busy, fatal).
      - Test: send two exec frames back-to-back; assert error and WS closes.
  2. Token binding
      - Host accepts command_complete only if (seq, token_hex) matches awaited (docs/project_management/next/world-first-repl-persistent-pty/
        PROTOCOL.md, “command_complete” acceptance rules).
      - Test: fuzz a driver in a harness to send mismatched `token_hex`; host must fail closed.
  3. Output ordering
      - command_complete must not arrive before all foreground stdout bytes were forwarded (docs/project_management/next/world-first-repl-
        persistent-pty/PROTOCOL.md, “Output ordering / drain guarantee”).
      - Test: run a command that writes a large output then exits; assert the next prompt/render only happens after the final bytes.

  ## Persistence invariants (design goal)

  4. CWD persistence (physical)
      - cd persists across submissions; reported cwd is physical (pwd -P) (docs/project_management/next/world-first-repl-persistent-pty/
        PROTOCOL.md, “Working directory (`cwd`) semantics”).
      - Test: mkdir -p real; ln -s real link; cd link; pwd must yield resolved physical path in command_complete.cwd.
  5. Export/unset persistence
      - export FOO=bar then echo "$FOO" prints bar; unset clears.
      - Test: multi-command integration test over a single persistent session.
  6. SHIM_PARENT_CMD_ID non-persistence
      - After command, env | rg SHIM_PARENT_CMD_ID must be empty (plan bullet).
      - Test: run a command that prints env; ensure it’s absent on the next submission. Also ensure it was present during the command by
        checking in-world shim logs later (when parity ships).

  ## Security invariants (DR-22)

  7. Control-plane handle privacy
      - Attempts to access inherited non-stdio file descriptors (including via `/proc/self/fd` where available and numeric redirections like
        `>&$FD` / `<&$FD`) must not be able to reach session infrastructure/control-plane endpoints or spoof completion (see DR-22).
      - Test: execute a command that enumerates `/proc/self/fd` (where available) and attempts to read/write any non-stdio fds; session must
        remain correct and completion must not be spoofable.

  ## Stdin-mode invariants (DR-20 / DR-14)

  8. Line mode EOF (no hangs)
      - stdin_mode=eof must not hang on stdin-consuming commands (cat, read).
      - Test: run cat in line mode; it must exit promptly.
  9. Passthrough correctness
      - In passthrough, typed Ctrl+C is a byte 0x03 forwarded as stdin bytes, not translated into a signal frame (docs/project_management/
        next/world-first-repl-persistent-pty/PROTOCOL.md, “signal targeting semantics”).
      - Test: run an in-world program that prints received bytes; assert 0x03 observed.

  ## REPL integration invariants (host-side, per STATE_MACHINE)

  10. Out-of-band output rendering while idle

  - stdout arriving while Idle must be rendered without corrupting input (docs/project_management/next/world-first-repl-persistent-pty/
    STATE_MACHINE.md:57).
  - Test: start a background writer (even if unsupported) and verify prompt restoration. This likely requires a new byte-capable renderer (see
    friction point #1).

  11. No structured message injection during passthrough

  - Agent events like :demo-agent must not corrupt TUI output; buffer and flush after passthrough (plan bullet, and current event pipeline is
    ExternalPrinter<String>: crates/shell/src/execution/agent_events.rs:24).
  - Test: run a passthrough TUI and concurrently emit demo agent events; terminal output should not be corrupted.

  ———

  # 6) Required callouts: exit/exec/logout/kill $$, backgrounding, out-of-band output

  ## Exit/exec/logout/kill $$

  - Under the recommended Approach A, these are not session-terminating because the evaluator bash is per-exec and disposable.
      - exit becomes “end this submission with exit code”, not “kill the REPL world session”.
      - exec … replaces the evaluator bash for that one submission; still fine.
      - logout in non-login bash is typically an error; treat as normal command.
      - kill $$ kills the evaluator; driver reports exit code accordingly.
  - This matches the v1 protocol’s evaluation model (per-submission evaluator shells with driver-managed persistence). If a future revision wants a true long-lived shell interpreter for broader “native bash session” semantics, it must adopt a DR-22-compliant architecture (see decision register DR-07 Option B notes).

  ## Backgrounding

  - Still unsupported (v1 doesn’t do job control), but:
      - background processes can write to the controlling TTY and create out-of-band stdout (docs/project_management/next/world-first-repl-
        persistent-pty/PROTOCOL.md, “Out-of-band PTY output”).
      - host must render out-of-band bytes and must not attribute them to a cmd_id.

  ## Out-of-band output

  - The driver should forward PTY bytes regardless of whether an exec is in-flight; world-agent forwards as stdout frames.
  - The host must be able to print these bytes while Reedline is waiting (requires byte-capable concurrent rendering, not just
    ExternalPrinter<String>).
