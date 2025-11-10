# Reedline Prompt Regression Investigation (November 2025)

## Summary
- **Symptom**: After running interactive/PTY-heavy programs (e.g., `node`, `sqlite3`, `codex-cli`), the Substrate prompt does not repaint until an extra key press is sent. Non-PTY commands (e.g., `pwd`, `npm --version`) return immediately without the delay.
- **Status**: The regression persists on `testing` and the latest installer tarballs (`substrate 0.2.10`). Prompt corruption (literal `^[[row;colR`) is fixed, but the prompt hang remains.
- **Impact**: Users experience a blank line after exiting REPL-like tools and must send an additional key (Enter/Space/etc.) to recover the Substrate prompt.

## Environment & Reproduction
1. Install Substrate via the updated installer (`curl .../install-substrate.sh | bash`) so the world-agent/systemd flow is active.
2. Launch the async shell (`substrate`).
3. Run a sequence of commands. Sample transcript:
   ```text
   substrate> pwd
   /var/lib/substrate/overlay/wld_019a6e77-fe79-7b91-9784-7b048b65c61b/merged
   substrate> node --version
   v24.8.0

   substrate> npm --version
   11.6.0
   [human] stdout: 11.6.0
   substrate> sqlite3
   SQLite version 3.50.4 2025-07-30 19:33:53
   ...
   sqlite> .exit

   substrate>
   ```
   Commands that hand the TTY back (Node REPL, sqlite shell, codex-cli, etc.) leave a blank line and require another key press before the Substrate prompt is visible.

## Historical Context & Commits
We *did* ship a working solution after the Stage 5 async editor landed:
- `19480711b86752b5ddabe96a411258c90275e4c7` – final async REPL work before any upstream Reedline sync; no extra key press was required at this point.
- `c2a260acb85e9368336e07687464725b95541cbc` – “chore: align with reedline upstream” (Substrate changes + vendored Reedline submodule update). At the time, the submodule still pointed at our `minimal-suspend-api` branch, so behavior remained stable.

Regression window on the Reedline fork:
- `de034c5` – local reapplication of our async APIs on top of upstream `nushell/reedline`.
- `f405847` – merge of `upstream/main` into our `upstream-sync` branch.
- `7cdaf3e` – PR merge for `minimal-suspend-api`.
- `2bedbe2` – “fix: remove duplicate suspend helpers.”
Since these merges, the prompt now waits for the next input event after each PTY run.

## Work Completed in This Session
1. **Reedline fork updates** (`third_party/reedline`, branch `atomize/prompt-fixes`, merged into upstream fork `main` @ `4a6aef2`):
   - `fast_cursor_position` drains any stray `CSI 6n` replies to prevent visible `^[[row;colR` artifacts.
   - `SuspendGuard::drop` no longer clears `suspended_state`; `begin_nonblocking_session` handles it so the painter state survives PTY suspensions.
2. **Async REPL changes** (`crates/shell/src/async_repl.rs`):
   - Added `AsyncReedlineAdapter::resume_after_command()` (calls `begin_nonblocking_session`, `process_events`, `flush_external_messages`, `force_repaint`).
   - PTY completion flow now uses the helper immediately after each command.
3. **Testing**:
   - `cargo test -p substrate-shell linux_world_tests -- --nocapture`
   - Manual prompt drills (still reproduces hang).

## Remaining Issues / Unknowns
- Reedline still paints the prompt but blocks until another `Event` arrives. Even with `process_events(Vec::new())`, no repaint occurs until the next keypress.
- Need to diff `origin/minimal-suspend-api` vs. `main` to see what changed around `begin_nonblocking_session`, `process_events`, and painter flushing.
- Possibility: upstream adds buffering in `process_events` or `flush_external_messages` that we aren’t triggering. Minimal branch may have forced a repaint directly after `suspend_guard`.

## Next Steps for Follow-up Agent
1. Review docs and historical plans:
   - `docs/project_management/_archived/pty_spike/REEDLINE_PTY_ISSUE.md`
   - `docs/project_management/_archived/pty_spike/REEDLINE_MIGRATION_PLAN.md`
   - `docs/project_management/_archived/pty_spike/REEDLINE_PR_READY.md`
   - `docs/project_management/now/stage5_editor_parity_plan.md`
2. Compare Reedline commits: `minimal-suspend-api` vs. current `main` (focus on `engine.rs` and `painter.rs`). Identify what changed between `c2a260a` and now.
3. Instrument the async adapter to log Reedline signals after PTY commands. Confirm whether Reedline delivers any events on resume.
4. Explore forcing a synthetic `Event::Resize` or `Event::FocusGained` immediately after we drop the guard, to unblock the painter.
5. Re-run `scripts/dev/async_repl_prompt_checks.py` once a candidate fix is implemented.

