Grounding (what exists today)

  - World-agent /v1/stream is currently one-shot only: the first WS frame must be {"type":"start", ...} and crates/world-agent/src/pty.rs
    spawns a single child, streams PTY reads as stdout, then sends exit. The PTY reader runs in a separate task and is aborted after sending
    exit, so “final stdout before completion” is not currently guaranteed.
  - Host-side PTY passthrough that the REPL will reuse is execute_world_pty_over_ws* in crates/shell/src/execution/routing/dispatch/
    world_ops.rs (raw-mode, stdin byte forwarding, resize forwarding, optional signal forwarding).
  - The authoritative requirement is in docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:
      - command_complete MUST NOT be emitted until all foreground PTY output bytes for that command have been forwarded.
      - stdin gating rules, close/exit semantics, and signal targeting (foreground process group, not a PID).
  - The repo already sketches the correct shape in docs/project_management/next/world-first-repl-persistent-pty/driver_loop_design.md: make a
    trusted driver that owns the PTY and emits a single ordered stream of (stdout bytes | command_complete) events.
  - Important nuance (host UX + future agent hub):
      - PTY `stdout` frames are raw bytes from the session PTY.
      - Concurrent Substrate-managed output (e.g., `:demo-agent`, future AgentHub events) is **not PTY bytes** and MUST NOT be injected into
        the PTY stream. It must be rendered via a separate host path (Reedline external printer / buffered renderer), and during PTY
        passthrough it SHOULD be buffered to avoid corrupting TUIs.
      - See `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`.

  ———

  ## 1) Algorithm: ordering-safe PTY read + message emission

  ### Key architectural decision (to make ordering provable)

  Make the “ordering boundary” internal and single-stream:

  1. A trusted driver component (new, in-world, owned by world-agent) owns:
      - the session PTY master (read output / write stdin bytes),
      - resize application,
      - foreground process group tracking for signals,
      - per-session persisted state required by the ADR (at minimum: physical cwd + exported env), per PROTOCOL.md appendix and
        driver_loop_design.md.
  2. The world-agent WS handler owns:
      - protocol validation and fail-closed behavior,
      - a single WebSocket writer loop,
      - forwarding host frames (stdin/resize/signal/close) to the driver.
  3. All PTY output and completion events flow from the driver to the handler over one ordered channel:
      - DriverEvent::Stdout(bytes)
      - DriverEvent::CommandComplete { seq, token_hex, exit, cwd }
      - DriverEvent::Exit { code }
      - DriverEvent::FatalError { code, message, seq? }

  This avoids trying to “merge” PTY output and completion from unrelated tasks in world-agent (which is where ordering bugs usually come
  from).

  ### World-agent WS handler state machine (persistent-session mode)

  States match the protocol (and fail closed on violations):

  - AwaitFirstFrame
  - BootingSession (spawning driver, PTY, mounts; before ready)
  - Idle (ready, no exec in-flight)
  - Executing { seq, token_hex, stdin_mode }
  - ShuttingDown (after close or fatal error)

  Inbound frames (client → agent):

  - start_session: only allowed in AwaitFirstFrame
  - exec: only allowed in Idle (else fatal exec_while_busy)
  - stdin: forwarded to driver, but driver enforces gating (below)
  - resize: forwarded to driver always
  - signal: forwarded to driver, but driver enforces gating (below)
  - close: transitions to ShuttingDown

  Outbound frames (agent → client):

  - ready: emitted once, after driver is fully running
  - stdout: forwarded from driver Stdout(bytes) events
  - command_complete: forwarded from driver CommandComplete
  - exit: forwarded from driver Exit
  - error: on any fatal protocol/runtime violation (per PROTOCOL.md persistent-session error schema, fatal:true)

  ### WebSocket emission pipeline (world-agent side)

  To make “forwarded-before-complete” meaningful, the handler uses exactly one writer task:

  WS writer task

  - Input: mpsc::Receiver<ServerMessage> (FIFO).
  - Behavior: serialize → await ws_sink.send(Message::Text(...)) for each message.
  - No other task writes to the WebSocket sink.

  Driver forwarder task

  - Reads DriverEvent stream (FIFO).
  - Maps to ServerMessage:
      - Stdout(bytes) → {"type":"stdout","data_b64":...}
      - CommandComplete{...} → {"type":"command_complete",...}
      - Exit{code} → {"type":"exit","code":...}
      - FatalError{...} → {"type":"error","code":...,"fatal":true,"seq":...}
  - Enqueues to WS writer queue in the same order received.

  Because the writer awaits each send, a command_complete frame cannot be emitted until all prior stdout frames have been sent on the WS.

  ### Driver algorithm (where the “pending completion” lives)

  Driver states

  - Idle { session_cwd, session_env }
  - Running { seq, token_hex, stdin_mode, evaluator_handle, fg_pgid }
  - Closing

  Driver main loop (pseudo-code)

  loop:
    select over:
      A) control messages from world-agent:
         - Exec(seq, token_hex, cmd_id, stdin_mode, program_utf8)
         - Stdin(bytes)
         - Resize(cols, rows)
         - Signal(sig)
         - Close

      B) PTY master readable

      C) evaluator process exit (only when Running)

  on Exec in Idle:
    - validate no NUL, UTF-8/no-NUL MUST be validated by the handler per `PROTOCOL.md`; driver MAY defensively re-check
    - spawn evaluator (per-exec) attached to PTY slave:
        - cwd = session_cwd
        - envp = session_env (+ required session overrides) + SHIM_PARENT_CMD_ID=cmd_id
        - stdin wiring:
            - eof: stdin = /dev/null (NOT PTY)
            - passthrough: stdin = PTY slave
        - ensure the evaluator does not inherit session infrastructure/control-plane handles (DR-22)
        - put evaluator into its own process group and make it foreground on the PTY
    - transition -> Running{...}

  on PTY readable:
    - read as many bytes as immediately available (best-effort batching)
    - for each chunk: emit DriverEvent::Stdout(chunk_bytes)

  on Stdin(bytes):
    - if state == Running && stdin_mode == passthrough:
        write bytes to PTY master (these become PTY slave input)
      else:
        drop (per PROTOCOL.md stdin boundary rules)

  on Resize(cols, rows):
    - apply PTY size (best-effort; errors become FatalError only if they break invariants)

  on Signal(sig):
    - if state != Running: drop (per PROTOCOL.md)
    - else: deliver to foreground process group (kill(-fg_pgid, signo) or tcgetpgrp-based)
            MUST NOT target session infrastructure (including world-agent) directly; target the foreground process group

  on evaluator exit (Running):
    - record exit status as protocol requires (bash convention 128+signal if signaled)
    - IMPORTANT: enter "pending completion" barrier:
        1) Drain PTY output (foreground drain barrier; does not require quiescence):
           - Snapshot a PTY-read “watermark” at the moment evaluator termination is observed:
               - Linux: `ioctl(FIONREAD)` on the PTY master to get `available_bytes_at_exit`.
               - If the platform cannot support a watermark query, the session MUST fail closed (do not silently switch to timing heuristics).
           - Drain/read the PTY master until at least `available_bytes_at_exit` bytes have been read and emitted as DriverEvent::Stdout.
           - The driver MUST NOT wait for global PTY quiescence (would-block) as a prerequisite for `command_complete`, because continuous
             out-of-band writers may prevent quiescence indefinitely.
           - If drain errors in a way that indicates a broken stream invariant: FatalError + session close.
        2) Update persisted state:
           - session_cwd = physical cwd after exec (getcwd()/pwd -P semantics)
           - session_env = exported env mutations
           - ensure SHIM_PARENT_CMD_ID does not persist
        3) Emit DriverEvent::CommandComplete{seq, token_hex, exit, cwd=session_cwd}
    - transition -> Idle{...}

  on Close:
    - terminate evaluator if Running (best-effort), stop accepting exec
    - optionally drain PTY once more (same drain primitive)
    - emit DriverEvent::Exit{code:0 or best-effort}
    - return

  What “pending completion” means precisely

  - “Pending completion” begins when the evaluator is known finished (exit observed), but before CommandComplete is emitted.
  - During this phase the driver only does one thing that matters for ordering: drain the PTY output to a deterministic *foreground watermark*
    (bytes readable at evaluator exit), and only then emits CommandComplete.

  This is the repository’s recommended strategy in driver_loop_design.md §1.4, updated to avoid non-terminating drains under continuous
  out-of-band output.

  ———

  ## 2) Correctness argument (no late foreground stdout after completion)

  We need: for a given exec(seq, token_hex), the host must not receive any PTY bytes produced by that foreground execution after
  command_complete(seq, token_hex, ...).

  Assumptions that are explicitly in-scope per the docs

  - Job control/backgrounding is unsupported and considered out-of-scope for per-line auditability (STATE_MACHINE.md, PROTOCOL.md note under
    ordering). Output after completion may happen, but it is out-of-band/unattributed.

  Invariants established by the design

  1. Single total order of driver events: the driver emits a single FIFO sequence of Stdout(bytes) and CommandComplete(...) events.
  2. PTY byte order preserved: Stdout(bytes) events are emitted in the same order bytes are read from the PTY master.
  3. Completion barrier: for a given exec, the driver emits CommandComplete only after:
      - the evaluator has terminated, and
      - the driver has drained at least the PTY-read watermark captured at evaluator exit (`available_bytes_at_exit`).
  4. Single WS writer, FIFO: the world-agent forwards driver events to the WebSocket in FIFO order, and the writer awaits each send.

  Why this implies the required guarantee

  - Any PTY bytes “produced by the foreground execution” that successfully reach the PTY device are in the PTY master’s read stream.
  - After the evaluator is observed terminated, it cannot produce additional output bytes.
  - The watermark drain step ensures that all bytes that were already readable in the PTY stream at the moment evaluator termination was
    observed are read and emitted as Stdout events before CommandComplete.
  - Because the WS writer is FIFO and awaited, every stdout frame corresponding to those drained bytes is sent before the command_complete
    frame is sent.
  - Therefore the host cannot observe “late foreground stdout after command_complete” unless:
      - the PTY stream itself is broken (read errors, descriptor misuse), or
      - the system violates basic PTY semantics.
        In those cases the protocol requires failing closed; we send a fatal error and terminate the session.

  No timing sleeps or heuristic delays are required for correctness; the barrier is defined by (a) evaluator termination and (b) a
  deterministic PTY-read watermark (not global quiescence).

  ———

  ## 3) Boundary of the guarantee + handling out-of-band bytes

  What is guaranteed

  - For each exec(seq, token_hex), all PTY bytes that are in the PTY output stream as a consequence of that foreground execution and that are
    readable by the driver before it emits CommandComplete are forwarded as stdout frames before command_complete.

  Operationally in this design:

  - “Foreground bytes” = bytes emitted by the PTY stream from the start of Running through the watermark drain barrier after evaluator exit.

  What is explicitly not guaranteed (and why)

  - command_complete is not a promise of “no more stdout will ever arrive” (this is stated in PROTOCOL.md and STATE_MACHINE.md):
      - background jobs started by the command may continue and write later,
      - other in-world processes may write asynchronously.

  How we identify/handle out-of-band bytes

  - The v1 protocol does not attribute stdout to a command.
  - Therefore, we define:
      - stdout while Idle is out-of-band by definition and is forwarded as normal stdout frames.
      - stdout after a given command’s command_complete is also out-of-band; it is forwarded, unattributed.
  - The important property is that out-of-band bytes must not be “tail bytes from the foreground command”. The drain barrier is what prevents
    that.

  Interaction with host-side concurrent output (AgentHub / `:demo-agent`)

  - This document’s “stdout” refers to the **session PTY byte stream** only.
  - Substrate-managed concurrent output on the host (agent/task events) is a distinct stream and MUST be rendered separately (see
    `docs/project_management/next/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`).
  - In particular, during PTY passthrough mode, the host SHOULD buffer structured events and render them only after the foreground PTY
    command completes, so TUIs/REPLs are not corrupted by interleaved host text.

  ———

  ## 4) Cross-platform considerations (Linux vs macOS PTY behaviors)

  Even though macOS REPL usage likely goes through a Linux guest today, the protocol scope names Linux/macOS, and portable_pty behavior
  differs across OSes. Likely friction points:

  - Drain primitive implementation
      - Linux: use a watermark query (`ioctl(FIONREAD)` on the PTY master) to bound the post-exit drain, then drain at least that many bytes.
      - macOS: PTY reads may return EIO in some “slave closed” situations where Linux returns 0. Persistent sessions shouldn’t close the slave
        except on shutdown; but shutdown paths must treat EIO as “PTY ended” and fail/exit consistently.
      - portable_pty exposes Read/Write traits; watermark queries require raw-fd access and OS calls (ioctl). If raw-fd access is not
        available, the persistent-session feature MUST fail closed on that platform (do not substitute timing heuristics).
  - Foreground process group signaling
      - The spec requires signaling the foreground process group of the session PTY, not a single PID (PROTOCOL.md).
      - Linux/macOS both support this via tcgetpgrp on the PTY slave and kill(-pgid, sig). You need correct session/job-control setup so the
        evaluator’s group is foreground while running, and the driver isn’t.
  - Resize semantics
      - Generally stable; but macOS terminal size ioctls sometimes need to be applied to the master vs slave consistently. The driver should
        pick one target and test it (plan.md resize bullet).
  - Exit/drain ordering on shutdown
      - Recommendation (non-normative): apply the same “drain before completion-like boundary” logic to session exit where practical (or else you recreate the same bug
        at session end). The v1 protocol is normative for shutdown semantics; see `PROTOCOL.md` `close`/`exit`.

  ———

  ## 5) Design alternatives / backup plans (if the above is hard)

  1. World-agent-local “pending completion” (no separate driver event stream)
      - Keep PTY reading and completion generation inside the WS handler, but still enforce:
          - a single outbound WS writer,
          - a completion barrier that triggers a bounded drain before command_complete (prefer watermark-based).
      - Risk: you’re back to coordinating multiple tasks (PTY read loop, exec lifecycle, WS receive), which is exactly where ordering bugs
        happen unless you carefully serialize all state transitions.
  2. Use tcdrain() on the PTY slave before emitting completion
      - After evaluator exit, call tcdrain(slave_fd) to wait until the terminal output queue is empty, then emit completion.
      - Pros: very explicit “output drained” semantic.
      - Cons: can hang if other writers keep writing; you’d need a fail-closed timeout (acceptable only if timeout triggers a fatal error, not
        a degraded continuation). Watermark-based drain avoids this class of hang.
      - v1 note: not permitted as a substitute for the watermark barrier required by DR-23 unless DR-23 / `PROTOCOL.md` is explicitly revised.
  3. Instrumented PTY proxy with an internal “barrier marker” that never reaches the user
      - Disallowed for v1: v1 explicitly avoids stdout marker parsing and forbids injecting non-PTY bytes into the Session PTY stream; the watermark barrier is the selected ordering mechanism (DR-23 / `PROTOCOL.md`).
  4. Separate PTY per exec (not a shared persistent PTY)
      - Avoids out-of-band entirely, but breaks the persistent-session UX and conflicts with the protocol’s session model.

  The recommended “trusted driver emits a single ordered (stdout|complete) stream after draining” is the cleanest path already endorsed by
  driver_loop_design.md.

  ———

  ## 6) Test plan mapping (to plan.md) + additional tests

  Directly satisfied by this mechanism

  - Output ordering: “no late stdout after command_complete” (plan.md output ordering bullet, PROTOCOL.md ordering section).
  - Out-of-band stdout forwarding: driver emits Stdout even while Idle; handler forwards as stdout frames (plan.md out-of-band stdout bullet).
  - Stdin boundary: driver drops stdin unless Running && stdin_mode=passthrough, and drops after completion (PROTOCOL.md stdin boundary rules;
    plan.md stdin boundary bullet).
  - No pipelining: handler rejects exec while Executing (plan.md no pipelining bullet).
  - Signal targeting: driver signals foreground process group only while Running (plan.md signal targeting bullet; PROTOCOL.md signal
    semantics).
  - PTY passthrough Ctrl+C semantics: preserved because typed Ctrl+C is just stdin bytes; host must not translate to signal (STATE_MACHINE.md
    + PROTOCOL.md).

  Additional tests to lock the ordering guarantee

  1. Protocol-level ordering integration test (WS harness)
      - Start persistent session (start_session → ready).
      - Exec a command that writes a large deterministic stream ending with a **test-only output sentinel** (e.g., ...; printf '__SENTINEL__\\n').
        This sentinel is not a completion marker and is not parsed by either side; it is only used by the test harness to assert ordering.
      - Assert:
          - the reconstructed stdout byte stream before command_complete contains the sentinel,
          - no additional bytes from that command arrive after its command_complete in the absence of backgrounding.
      - Then immediately exec a second command with a different sentinel and assert the first sentinel never appears after the first
        completion (catches inter-command interleaving).
  2. Passthrough boundary test
      - Exec stdin_mode=passthrough with a small program that echoes raw input and exits on a key.
      - Verify that bytes typed right at completion do not leak into the next command (stdin drop-after-complete rule); this is explicitly
        allowed to drop keystrokes, but not to misdeliver them.
  3. Shutdown drain test
      - While a command is running and producing output, send close.
      - Assert session terminates with a fatal/expected exit path and does not reorder exit ahead of already-produced stdout (same ordering
        principle applied to shutdown).
