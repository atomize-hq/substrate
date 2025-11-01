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
Populate this section during `stage5-design-async-reedline`. Summarize the chosen adapter architecture, rejected alternatives, and any follow-up items.

### stage5-design-async-reedline
#### Async Reedline Adapter
- **Deliverables**: An expanded "Async Reedline Adapter" section in this plan plus a short ADR-style note inside `docs/project_management/now/stage5_editor_parity_plan.md#design-decision` (create the anchor) that records why we chose an adapter rather than a blocking Reedline loop. Include the rejection reasoning for the alternative.
- **Adapter outline**: Describe the Rust surface you expect to implement (`AsyncReedlineAdapter` struct with methods such as `process_event`, `handle_agent_event`, `pause_for_command`, `resume_after_command`). Reference the existing synchronous editor at `crates/shell/src/lib.rs:1736-1928` for the history/completion setup and note which pieces will be shared.
- **Stdout / locking**: Document how stdout writes stay serialized—e.g., via a shared `Mutex<Stdout>` or by reusing Reedline`s painter APIs—and confirm the approach against the `render_agent_event` helper (`crates/shell/src/async_repl.rs:240-254`).
- **Fallback guidance**: Specify how we expose a debug escape hatch (e.g., `--minimal-async-editor`) if the adapter misbehaves and where that flag should be documented.

### stage5-shared-editor-core
- **Implementation goals**: Factor the Reedline builder (prompt, keybindings, history, completer, menus) into a new helper module (`crates/shell/src/editor.rs`). The helper must return a configured `Reedline` instance and the associated metadata (history path, external printer) without altering the `run_interactive_shell` behavior.
- **Regression guard**: Note in this plan (and in the task acceptance criteria) that `target/debug/substrate --legacy-repl --no-world` must still provide completion, history recall, and the same prompt styling as before refactor.
- **History setup reference**: Call out the existing file-backed history located at `~/.substrate_history` (see `crates/shell/src/lib.rs:1742-1759`) so the extraction keeps the path and file-creation semantics intact.

### stage5-history-completions
- **Async wiring checklist**:
  1. Async mode must load and sync the same `FileBackedHistory` instance used by the sync REPL, writing back on exit.
  2. Completion menu (`ColumnarMenu` registered as `completion_menu`) should display when the user presses Tab, just as in the legacy loop.
  3. Confirm history persistence by running the async REPL twice and inspecting `~/.substrate_history` for the new entries.
- **Agent-output compatibility**: Document how the completion popup coexists with streaming agent events (e.g., the adapter defers redraw while menus are visible).
- **Testing guidance**: Encourage extending `scripts/dev/async_repl_prompt_checks.py` with a Stage 5 scenario that triggers history recall mid-stream and capture the transcript to `docs/project_management/now/stage5_prompt_checks_transcript.txt`.

### stage5-cursor-multiline
- **Event mapping**: Detail how crossterm key events (`CtKeyCode::Left`, `Right`, `Home`, `End`, `Enter` with modifiers) map to the corresponding `ReedlineEvent::Edit` commands. Reference `third_party/reedline/src/engine.rs:874-1010` for canonical handling.
- **Multiline caret**: Specify the redraw expectations—after receiving an agent event while the user edits a multi-line buffer, the caret should return to the correct logical column/row.
- **Manual drills**: Define a new scenario (e.g., multi-line shell function editing) to add to `async_repl_prompt_checks.py`, ensuring the log is saved alongside the Stage 4 transcript with a Stage 5 heading.
- **Undo/redo**: State whether we adopt Reedline's undo stack directly and how to verify it (e.g., `Ctrl+_` round trip).

### stage5-regression-validation
- **Automation**: List the exact integration tests to add or extend, such as a new async REPL integration test under `crates/shell/tests/async_repl.rs` (create if missing) that asserts the prompt state after simulated events.
- **Metrics capture**: Reuse `scripts/dev/async_repl_metrics.py` (`--output docs/project_management/now/stage5_idle_metrics_<platform>.txt`) to record idle CPU samples per platform.
- **Manual logs**: Require storing prompt drill transcripts at `docs/project_management/now/stage5_prompt_checks_transcript.txt` and linking them in the session log.
- **Docs update checklist**: Update `docs/USAGE.md`, `docs/CONFIGURATION.md`, and `CHANGELOG.md` to note the restored editor parity and any new flags (e.g., debug escape hatch).

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
