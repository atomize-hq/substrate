# Stage 5: Async REPL Editor Parity Plan

## Purpose and Scope
Stage 5 brings the async REPL up to feature parity with the legacy Reedline-driven shell while preserving zero-idle CPU usage and prompt-safe agent streaming. The focus areas are:

- History navigation (previous/next search + persistent storage)
- Cursor-aware editing (left/right movement, word jumps, deletion)
- Multiline input editing with proper caret rendering
- Compatibility with streaming agent output via the shared `AgentEvent` channel

## Current State
- `run_async_repl` (see `crates/shell/src/async_repl.rs`) owns a bespoke crossterm loop that only supports append, backspace, tab literal, and submit.
- Reedline features (history, completions, transient prompt, undo stack) remain exclusive to the sync REPL (`crates/shell/src/lib.rs`).
- Shared infrastructure from Stage 3 (event channel, redraw helpers, telemetry) already works in both modes.

## Requirements
1. Restore Reedline-grade editing in async mode without reintroducing busy-spin or prompt corruption.
2. Keep agent streaming behavior identical to Stage 3 (agent events land above the prompt, prompt/input restored).
3. Maintain shared history storage, completions, and keybindings definitions.
4. Remain cross-platform (Linux/macOS/Windows) using crossterm-compatible APIs only.
5. Preserve tracing/telemetry hooks introduced in Stage 2–4.

## Approach Overview
- Reuse Reedline’s editing engine by extracting reusable components (history, keymaps, completer, prompt) into a shared `editor` module.
- Convert async crossterm events into `ReedlineEvent`s and pass them through the engine using a new `AsyncReedlineAdapter` that mirrors `read_line` without blocking.
- Manage raw-mode transitions and prompt redraw by delegating to Reedline’s `Painter`/`suspend_guard` equivalents via adapter helpers.
- Propagate agent events through the existing redraw pipeline; ensure the adapter exposes hooks to temporarily pause editing when commands execute.

## Task Breakdown
1. **stage5-design-async-reedline** – Finalize adapter architecture, document event conversion strategy, and compare alternatives (full Reedline loop in blocking thread vs. async adapter).
2. **stage5-shared-editor-core** – Extract shared editor construction (`Reedline::create` setup, prompts, completer) into a reusable builder consumed by both sync and async paths.
3. **stage5-history-completions** – Wire async mode to use shared history backend and completion menus; confirm history persistence and menu rendering still work while streaming.
4. **stage5-cursor-multiline** – Implement Reedline event handling inside async loop with cursor movement, multi-line caret painting, and undo/redo. Ensure redraw integrates with agent events.
5. **stage5-regression-validation** – Update documentation (`docs/USAGE.md`, release notes), add regression tests/fixtures, and run manual prompt-with-stream drills across all OS backends.

## Detailed Task Guidance

### Design Decision Record {#design-decision}
- **Decision (2025-11-01):** Implement an in-process `AsyncReedlineAdapter` that wraps Reedline's editor state and painter while the Tokio main loop drives crossterm input and agent output. We will keep the async shell single-threaded for stdin/agent processing and use a shared stdout lock (via `std::sync::Mutex<Stdout>`) to serialize redraws with Reedline's painter helpers.
- **Status:** Accepted. Implementation will begin with `stage5-shared-editor-core`.
- **Alternatives considered:**
  - *Run the full Reedline event loop on a blocking thread and proxy input/output over channels.* Rejected because it re-introduces a busy wait on macOS (Reedline polls within its loop) and complicates shutdown/TTY handoff across platforms.
  - *Rewrite the async editor without Reedline.* Rejected due to the high cost of re-creating history, completion, and undo stacks, and because it would diverge from our vendored Reedline fork (`third_party/reedline`).
- **Consequences:**
  - Requires extracting Reedline configuration into a shared builder so the adapter can borrow the same keymaps/history/completion pipeline (`crates/shell/src/lib.rs:1736-1928`).
  - Demands explicit stdout coordination with existing helpers (see `render_agent_event` in `crates/shell/src/async_repl.rs:240-254`) to avoid prompt corruption.
  - Permits future optimization (e.g., alternative adapters) without touching command execution paths.
- **Follow-up:**
  - Add a hidden `--minimal-async-editor` debug flag only if adapter instability is observed in later stages; until then, rely on `--legacy-repl` as the escape hatch. Should we ship the debug flag, update `docs/USAGE.md` and `docs/CONFIGURATION.md` in `stage5-regression-validation`.

### stage5-design-async-reedline
#### Async Reedline Adapter
- **Deliverables**: This section documents the adapter design and the design decision record above. The adapter plan includes diagrams/text covering event flow, concurrency, stdout coordination, and fallback strategy.
- **Architecture Summary**:
  - The async shell (`run_async_repl`, `crates/shell/src/async_repl.rs:35-199`) keeps ownership of the Tokio loop. We introduce an `AsyncReedlineAdapter` that encapsulates Reedline state (editor, painter, prompt) and exposes non-blocking methods to mutate that state.
  - The adapter holds:
    - `editor`: wrapped Reedline engine configured via the shared builder extracted in `stage5-shared-editor-core`.
    - `painter`: borrowed from Reedline to handle redraws after edits/agent events.
    - `stdout_lock`: `Arc<Mutex<Stdout>>` shared with agent-event rendering to serialize writes.
    - `pending_menu_state`: tracks whether completion UI is visible so agent output can defer repaint until menu is hidden.
  - Event queues:
    - `input_rx`: async `Stream` of `crossterm::Event` (already produced by `EventStream::new().fuse()`).
    - `agent_rx`: `tokio::sync::mpsc::Receiver<AgentEvent>` from `init_event_channel()`.
    - Both feed into `tokio::select!` (existing loop) but the adapter converts each event and returns a set of painting commands.
- **Adapter API Outline** (minimal surface):
  ```rust
  pub struct AsyncReedlineAdapter {
      editor: Reedline,
      painter: ReedlinePainter,
      stdout: Arc<Mutex<Stdout>>, // shared with agent renderer
      prompt: Box<dyn Prompt>,
      buffer_cache: String,
      menu_active: bool,
  }

  impl AsyncReedlineAdapter {
      pub fn handle_key_event(&mut self, event: KeyEvent) -> AdapterAction;
      pub fn handle_control(&mut self, event: KeyEvent) -> AdapterAction; // keeps Ctrl+C/D semantics
      pub fn handle_agent_event(&mut self, evt: &AgentEvent) -> io::Result<()>;
      pub fn suspend_for_command<F>(&mut self, f: F) -> Result<ExitStatus>
          where F: FnOnce() -> Result<ExitStatus>;
      pub fn redraw_prompt(&mut self) -> io::Result<()>;
      pub fn drain_history(&mut self) -> io::Result<()>; // flush history on exit
  }
  ```
  `AdapterAction` enumerates responses for the outer loop: `Continue`, `Submit(String)`, `Exit`, `Redraw`.
- **Event Flow**:
  1. Tokio reads `KeyEvent` from crossterm and calls `handle_key_event`. The adapter translates to `ReedlineEvent` via the same parsing logic used in `third_party/reedline/src/engine.rs:874-1010`, but without blocking.
  2. `handle_key_event` mutates the Reedline editor and uses `painter` to repaint if necessary. Stdout writes occur within a `stdout_lock` guard so that concurrent agent output (see `render_agent_event` helper) cannot interleave mid-draw.
  3. When `AdapterAction::Submit(cmd)` is returned, the outer loop pauses raw mode (via `RawTerminalGuard`) exactly as today, executes the command (spawn_blocking), and resumes through `suspend_for_command`.
  4. Agent events arrive via `agent_rx`; `handle_agent_event` formats lines with existing `format_event_line` but uses the adapter's painter to temporarily hide the prompt, print the event, and restore the buffer.
- **Stdout / Locking**: Adopt a shared `Arc<Mutex<Stdout>>` created in `run_async_repl` and passed into both the adapter and the agent event handler. All prompt redraws and agent prints acquire this mutex to serialize writes. This mirrors the current synchronous approach (which relies on Reedline's `ExternalPrinter`) and maintains compatibility with `render_agent_event` (`crates/shell/src/async_repl.rs:240-254`).
- **Prompt & History Coordination**:
  - History is still provided by `FileBackedHistory::with_file` using `~/.substrate_history` (`crates/shell/src/lib.rs:1742-1759`). The builder created in `stage5-shared-editor-core` will supply the history handle to the adapter.
  - The adapter caches the current buffer (`buffer_cache`) so agent events can restore partially typed lines. On redraw, it prints `CLEAR_LINE`, prompt text, and cached buffer, matching the semantics of `redraw_prompt` today.
- **Telemetry**: Keep existing telemetry hooks (`ReplSessionTelemetry::record_input_event` and `record_agent_event`) in the outer loop. Adapter exposes callbacks so we can inject additional telemetry later (e.g., `record_completion_popup`).
- **Fallback strategy**: Continue recommending `--legacy-repl` for production fallback. If the adapter introduces instability, we will introduce `--minimal-async-editor` (same async loop with line-editing stripped) behind a hidden flag and document it in `docs/USAGE.md`/`docs/CONFIGURATION.md` during `stage5-regression-validation`. Until then, no extra flag is planned.
- **Compatibility checks**: Confirm adapter honors platform-specific terminal modes:
  - macOS: ensure vsock/tty bridging (no changes, adapter remains host-side).
  - Windows: guard `stdout_lock` usage with `crate::platform::windows` newline handling to preserve CRLF semantics when Reedline painter writes to conhost.
- **Open edges for subsequent tasks**:
  - Evaluate whether Reedline's menu drawing (completion UI) requires additional locking beyond stdout (tracked in `stage5-history-completions`).
  - Consider exposing an adapter method `handle_resize` so we can reuse `resize` handling currently in `run_async_repl`.

### stage5-shared-editor-core
- **Implementation goals**: Factor the Reedline builder (prompt, keybindings, history, completer, menus) into a new helper module (`crates/shell/src/editor.rs`). The helper must return a configured `Reedline` instance and the associated metadata (history path, external printer) without altering the `run_interactive_shell` behavior.
- **Implemented structure**: `build_editor` now returns `EditorSetup { line_editor, printer }`, and `make_prompt` exposes the reusable `SubstratePrompt` for both sync and async REPL paths.
- **Regression guard**: Note in this plan (and in the task acceptance criteria) that `target/debug/substrate --legacy-repl --no-world` must still provide completion, history recall, and the same prompt styling as before refactor.
- **History setup reference**: Call out the existing file-backed history located at `~/.substrate_history` (see `crates/shell/src/lib.rs:1742-1759`) so the extraction keeps the path and file-creation semantics intact.

### stage5-history-completions
- **Async wiring checklist**:
  1. Async mode must load and sync the same `FileBackedHistory` instance used by the sync REPL, writing back on exit.
  2. Completion menu (`ColumnarMenu` registered as `completion_menu`) should display when the user presses Tab, just as in the legacy loop.
  3. Confirm history persistence by running the async REPL twice and inspecting `~/.substrate_history` for the new entries.
- **Implementation status**: `AsyncReedlineAdapter` now drives Reedline via `process_events` (see `crates/shell/src/async_repl.rs` and `third_party/reedline/src/engine.rs:752-805`), so history/completion behavior matches the sync REPL while keeping the async event loop.
- **Agent-output compatibility**: Document how the completion popup coexists with streaming agent events (e.g., the adapter defers redraw while menus are visible).
- **Testing guidance**: Encourage extending `scripts/dev/async_repl_prompt_checks.py` with a Stage 5 scenario that triggers history recall mid-stream and capture the transcript to `docs/project_management/now/stage5_prompt_checks_transcript.txt`.
- **Tooling**: `scripts/dev/async_repl_prompt_checks.py` now emits both Stage 4 and Stage 5 transcripts (use `--stage5-log` to capture history/completion runs).

### stage5-cursor-multiline
- **Event mapping**: Detail how crossterm key events (`CtKeyCode::Left`, `Right`, `Home`, `End`, `Enter` with modifiers) map to the corresponding `ReedlineEvent::Edit` commands. Reference `third_party/reedline/src/engine.rs:874-1010` for canonical handling.
- **Implementation status**: The async shell now proxies events through `Reedline::process_events` (`crates/shell/src/async_repl.rs`, `third_party/reedline/src/engine.rs:752-823`), so cursor motion, multi-line editing (`Shift+Enter`/`Alt+Enter`), and undo/redo reuse Reedline internals without bespoke mappings.
- **Line continuation**: Trailing `\` before Enter now keeps the command in the buffer with a newline, matching classic shell behavior.
- **Follow-up**: Residual CSI cursor reports (e.g., `^[[45;28R`) still appear intermittently after certain binaries (`python`, `sqlite3`). A Stage 6/Stage 5 follow-up needs to audit remaining manual redraw paths and ensure we let Reedline manage prompt rendering end-to-end.
- **Multiline caret**: Specify the redraw expectations—after receiving an agent event while the user edits a multi-line buffer, the caret should return to the correct logical column/row. ExternalPrinter flushing in `AsyncReedlineAdapter::flush_external_messages` ensures prompt restoration.
- **Manual drills**: Define a new scenario (e.g., multi-line shell function editing) to add to `async_repl_prompt_checks.py`, ensuring the log is saved alongside the Stage 4 transcript with a Stage 5 heading.
- **Undo/redo**: State whether we adopt Reedline's undo stack directly and how to verify it (e.g., `Ctrl+_` round trip).
- **Tooling**: Stage 5 prompt checks (`scripts/dev/async_repl_prompt_checks.py --stage5-log ...`) now cover completion + multi-line editing under streaming output and store transcripts in `docs/project_management/now/stage5_prompt_checks_transcript.txt`.

### stage5-regression-validation
- **Automation**: List the exact integration tests to add or extend, such as a new async REPL integration test under `crates/shell/tests/async_repl.rs` (create if missing) that asserts the prompt state after simulated events.
- **Metrics capture**: Reuse `scripts/dev/async_repl_metrics.py` (`--output docs/project_management/now/stage5_idle_metrics_<platform>.txt`) to record idle CPU samples per platform.
- **Manual logs**: Require storing prompt drill transcripts at `docs/project_management/now/stage5_prompt_checks_transcript.txt` and linking them in the session log.
- **Docs update checklist**: Update `docs/USAGE.md`, `docs/CONFIGURATION.md`, and `CHANGELOG.md` to note the restored editor parity and any new flags (e.g., debug escape hatch).
- **Status (2025-11-03)**: Added stdin drains around `begin_nonblocking_session` to consume Reedline's cursor-position replies, regenerated Stage 4/Stage 5 transcripts, and captured new idle metrics (`stage5_idle_metrics_linux.txt`). Prompt automation still relies on scripted CSI handshakes; follow-up is logged to harden the harness.

## Validation Strategy
- Automated: extend shell integration tests with scripted input sequences exercising history and cursor navigation, verifying stdout snapshots.
- Manual: reuse Stage 4 prompt drills plus new multiline scenarios; capture transcripts for each platform.
- Performance: rerun idle CPU measurements to confirm parity (<0.1% idle on representative hosts).

## Risks & Mitigations
- **API drift in Reedline fork** – Track upstream changes; vendor patch must expose the async-friendly hooks we need. Mitigation: add targeted fork patches with cargo tests.
- **Terminal inconsistencies on Windows** – Ensure adapter honors Windows newline semantics; test under both conhost and Windows Terminal.
- **Event ordering conflicts** – Guard against interleaving agent redraw with Reedline repaint by serializing stdout writes via a mutex-backed adapter.

## Open Questions
- Should completions be fully enabled in Stage 5 or deferred to Stage 6? (Default: enable minimal completion menu; advanced features can follow.)
- Do we need an escape hatch to fall back to minimal async editor for diagnosing Reedline issues? (Proposal: keep `--legacy-repl` and add `--minimal-async-editor` debug flag if needed.)
