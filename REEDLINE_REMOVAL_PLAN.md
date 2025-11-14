# Reedline Removal Plan

## Goal
Replace the custom vendored Reedline fork with the official crates.io release by isolating our prompt handling behind a worker thread. This restores compatibility with upstream Reedline and prepares us for future block‑style UX (e.g., Tauri) without Reedline internals.

## Current Issues
- Async REPL depends on fork-only APIs (`SuspendGuard`, `begin_nonblocking_session`, `process_events`, `force_repaint`, etc.).
- Sync REPL calls `suspend_guard` to pause Reedline during PTY commands.
- Completer references non-existent fields (`match_indices`).
- Because of those patches, Cargo had to override Reedline to point at `third_party/reedline` (legacy fork removed in v0.2.12).

## Plan Overview
1. **Abstraction**
   - Introduce a `PromptWorker` (or similar) that runs in its own thread, owns a `Reedline` instance, and communicates via channels.
   - Worker receives commands (`StartPrompt`, `Shutdown`) and responds with signals (`Submit(String)`, `CtrlC`, `CtrlD`).
   - The worker is the only place we touch Reedline APIs.

2. **Async REPL Changes**
   - Remove `AsyncReedlineAdapter` and related cursor-drain helpers.
   - Main loop sends `StartPrompt` when ready and waits on the worker response.
   - After executing a command, send `StartPrompt` again to redraw.
   - Agent/stream output continues to flow via Reedline’s `ExternalPrinter` handle.

3. **Sync REPL Changes**
   - Stop using `suspend_guard`. Instead, only call `read_line` when no PTY is active; commands run outside the prompt loop.
   - Prompt resumes automatically when the worker starts the next `read_line` call.

4. **Completer Adjustments**
   - Remove `match_indices` assignments and stay within the public `Suggestion` fields.

5. **Infrastructure**
   - Delete the `[patch.crates-io]` entry in `Cargo.toml` once everything builds against upstream Reedline.
   - Remove the legacy `third_party/reedline` submodule when the migration is finished.

## Validation
- `cargo build --workspace --all-targets`
- `cargo test -p substrate-shell -- --nocapture`
- Manual prompt drill (run `node`, `sqlite3`, etc.) to ensure prompt returns immediately and agent output still appears.

## Future Flexibility
- With prompt handling isolated behind a worker, we can swap Reedline for any other editor or a Tauri-based block UI by reimplementing the worker.
- Remaining code paths (command execution, agent events, tracing) remain unchanged and compatible with `substrate -c` workflows.
