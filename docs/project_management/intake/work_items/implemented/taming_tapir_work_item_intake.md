---
codename: taming_tapir
created: "2026-04-06T20:52:04Z"
status: draft
depends_on: []
---

# Work Item Intake Sheet

## 1. Codename + date + status

- Codename: `taming_tapir`
- Created: 2026-04-06T20:52:04Z
- Status: draft

## 2. Title (imperative)

Prevent macOS async REPL CPU spin when the controlling TTY is revoked/disconnects (exit cleanly; no orphaned REPLs).

## 3. Why not ADR

- This is a bug fix / resilience improvement in the interactive shell loop, not an A/B architectural choice.
- The core architectural shape stays the same: Substrate still uses the async REPL + prompt worker model, and no new CLI/config surface is required.
- The only contract question is how abnormal terminal loss should be surfaced (exit code + messaging). That is a narrow follow-on contract clarification, not a net-new architecture decision.

## 4. Task definition (bounded)

### 4.1 Problem statement (observed)

On macOS, Substrate interactive REPL processes can peg a CPU core (~100%) indefinitely when their controlling TTY is revoked while Reedline/crossterm is active. In practice this shows up as orphaned `substrate --async-repl` processes reparented to `launchd` and consuming sustained CPU (Activity Monitor), even though no work is happening.

Evidence (example stack signature):
- `substrate_shell::repl::async_repl::run_prompt_worker`
  - `reedline::engine::Reedline::read_line`
    - `crossterm::event::poll`

A reliable repro exists by spawning Substrate under a PTY, allowing the prompt to render, then invoking `revoke(2)` on the PTY slave device path (for example `/dev/ttys154`). After revoke, CPU ramps to ~100% and remains pinned until the process is terminated.

This is a macOS-proven failure in shared REPL prompt code, not a macOS-only code path.

### 4.2 Repo-verified current behavior

- The async REPL prompt path is implemented in `crates/shell/src/repl/async_repl.rs`.
- `PromptWorker::spawn()` uses Reedline only when stdin is a TTY and the child process does not see `CI` / `GITHUB_ACTIONS`; otherwise it falls back to the plain stdio prompt worker.
- The prompt worker thread calls `Reedline::read_line(&prompt)` directly inside `run_prompt_worker()`, then sends one of:
  - `PromptWorkerResponse::Line`
  - `PromptWorkerResponse::CtrlC`
  - `PromptWorkerResponse::CtrlD`
  - `PromptWorkerResponse::Error`
- The main async loop already has one special prompt-error downgrade path:
  - cursor-position timeout errors trigger a fallback from Reedline to the stdio prompt worker.
- All other prompt errors currently print `prompt error: ...` and exit the REPL loop.
- Important nuance: today that path still returns the final `auto_sync_exit_code`, which is typically `0`; prompt failure after startup does not currently force a non-zero process exit.
- `PromptWorker::shutdown()` sends a shutdown command and then `join()`s the prompt thread. Any fix that detects terminal loss out-of-band must also ensure the worker unwinds out of `read_line()`; flipping a boolean in the async loop is not sufficient.
- The same file already contains platform/TTY plumbing that may be reusable for detection:
  - stdin termios handling in the stdio worker
  - resize handling via `get_terminal_size()`
  - signal tasks for SIGINT / SIGWINCH

### 4.3 Exit-code decision (resolved)

- `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md` currently says:
  - REPL process exit code remains `0` on normal `exit` / `quit`
  - REPL process is non-zero only on startup/config errors that prevent starting the REPL
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` reserves exit code `1` for unexpected internal/runtime failures.
- `docs/reference/env/contract.md` also states that shell entrypoint errors map to exit code `1`.
- A revoked controlling TTY after startup sits between those statements:
  - it is not a normal user exit
  - but it also is not a startup/config failure
- Resolution:
  - abnormal controlling-TTY loss during an interactive REPL session maps to exit code `1`
  - this follows `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`, where `1` is the canonical unexpected runtime/internal failure bucket
  - this is not a `0` success/no-op path, not a `2` user/config error, not a `3` dependency-unavailable path, not a `4` unsupported-prereq path, and not a `5` policy/safety path
- Required contract wording after this decision:
  - `0` on normal operator exit (`exit` / `quit`)
  - `1` on abnormal runtime termination such as terminal disconnect/revoke after startup

### 4.4 Intended behavior (target)

When Substrate is running interactively and the controlling TTY becomes invalid (revoked / disconnected):
- Substrate must stop the interactive prompt loop and exit promptly (no busy-spin).
- Substrate must not leave behind an orphaned CPU-pinning REPL process.
- Best-effort print a single diagnostic line to stderr describing the terminal disconnect.
- Exit with code `1` for abnormal terminal loss.

Non-goals:
- Do not attempt to keep the REPL running “headless” after terminal loss.
- Do not add new CLI flags/config keys for this behavior.
- Do not widen scope into world-service, shim, or backend transport changes.

### 4.5 Recommended implementation shape

Recommended default approach:

1. Treat this as a shell-local prompt-worker failure mode, not a world/session failure.
   - The minimal reproduction can run with world disabled and `--shim-skip`; the bug sits in the prompt/input layer.

2. Add explicit terminal-loss classification in the REPL code.
   - Immediate `read_line()` failures that look like terminal invalidation should map to one dedicated REPL error path rather than the generic `prompt error: ...` branch.
   - Candidate signals include TTY-invalid / I/O-invalid failures (`ENOTTY`, `EIO`, `EBADF`, EOF/disconnect behavior as observed in practice).

3. Ensure the disconnect path can unwind the prompt worker itself.
   - The observed macOS failure mode is that Reedline/crossterm may spin inside `read_line()` and never surface an error to the outer loop.
   - Therefore the implementation likely needs more than response classification alone; it needs either:
     - a shell-local liveness/disconnect detector that can make `read_line()` return, or
     - a narrower prompt-backend change that avoids the spin under revoked TTYs.

4. Track an explicit REPL session exit code.
   - The current loop returns the post-session auto-sync exit code, which means late prompt failures collapse to `0`.
   - The implementation should separate:
     - REPL session termination cause
     - optional auto-sync exit code on clean exit

5. Keep the fix bounded and host-only in tests.
   - The regression test should disable world and shims so it validates only prompt-worker behavior.

### 4.6 Two implementation options to evaluate up front

#### Option A — Error classification only after `read_line()` returns

- Pros:
  - Smallest code change.
  - Keeps Reedline as the prompt backend with minimal additional machinery.
- Cons:
  - Likely insufficient for the observed macOS revoke path, because the worker appears to spin inside `crossterm::event::poll` and may never return an error.
  - Does not by itself solve orphan cleanup if the worker thread never unwinds.

#### Option B — Add explicit terminal-loss detection plus a worker-unwind path

- Pros:
  - Matches the actual failure signature seen in the repo and the repro steps.
  - Solves both user-facing failure handling and the thread-join/orphan cleanup problem.
- Cons:
  - Slightly more invasive in `async_repl.rs`.
  - Requires careful validation to avoid false positives during normal TTY use.

Recommended default: Option B.

## 5. Done means (<= 8 outcomes)

- When the controlling TTY is revoked/disconnects while the async REPL is idle, Substrate exits promptly instead of spinning.
- The fix covers the common macOS failure mode (revoked PTY) and does not regress normal interactive behavior.
- The implementation returns exit code `1` for terminal-loss termination, and that status is documented anywhere the REPL exit contract is stated.
- A targeted macOS regression test reproduces the revoke/disconnect scenario and asserts the process exits within a bounded timeout.
- The test harness does not leave behind orphaned `substrate --async-repl` processes on failure or timeout paths.
- The regression test exercises the Reedline path specifically (not the stdio fallback path).
- No new CLI flags/config keys are introduced.
- Any user-facing/operator-facing wording about abnormal terminal disconnect is updated in the single authoritative place(s) rather than ad hoc.

## 6. Likely touch paths

- REPL prompt worker / termination handling:
  - `crates/shell/src/repl/async_repl.rs`
- Optional prompt wiring only if detection needs builder-level changes:
  - `crates/shell/src/repl/editor.rs`
- New macOS regression test:
  - `crates/shell/tests/repl_tty_disconnect_macos.rs`
- Optional shared PTY helper extraction if the test needs reusable cleanup/revoke utilities:
  - `crates/shell/tests/support/mod.rs`
  - or `crates/shell/tests/support/repl_pty.rs`
- Contract/docs if exit semantics change:
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - `docs/reference/env/contract.md`
  - `docs/USAGE.md`

## 7. Dependencies (ADR/WI)

- depends_on_adrs: []
- depends_on_work_items: []
- blocks: []

## 8. Implementation considerations discovered during fact finding

### 8.1 Test-path constraints

- The regression test must not run the child process with `CI=...` or `GITHUB_ACTIONS=...` in its environment, because `PromptWorker::spawn()` would otherwise choose the stdio worker and miss the Reedline path entirely.
- The regression test should launch `substrate --async-repl --shim-skip` with world disabled to isolate the prompt subsystem.
- Existing PTY-heavy integration tests already use `portable_pty`, non-blocking reads, and `#[serial]`; those patterns are the best local starting point.
- Existing PTY harness implementations are duplicated across several tests. This WI should stay bounded:
  - prefer a local helper inside the new test first
  - extract to `tests/support/` only if revoke/cleanup logic is clearly reusable

### 8.2 Cleanup / orphan-risk constraints

- A simple timeout assertion is not enough; the test must actively reap the child on failure.
- Several existing PTY tests kill the child in `shutdown()` / `Drop`, but not all harnesses centralize this. The new test should use a drop-safe cleanup path so aborted assertions do not strand a child.
- Because the reported production failure involves reparenting to `launchd`, the test cleanup path should not assume the original PTY master still being open is sufficient to terminate the child.

### 8.3 Scope constraints

- The reproduction is macOS-specific because it relies on `revoke(2)`, so the regression proof should be `#[cfg(target_os = "macos")]`.
- The runtime classification logic should still be written as generic Unix TTY-invalid handling unless implementation evidence forces a macOS-only branch; the prompt-worker codepath itself is not macOS-specific today.

## 9. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 4,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 1,
    "boundary_crossings": 3
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 1,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 1, "new_test_cases": 1 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": true,
    "migration_or_backfill": false,
    "unknowns_high": 2
  },
  "notes": "Refined estimate assumes a shell-only fix in async_repl.rs, one new macOS regression test, and one or more contract/doc edits if abnormal terminal-loss exit semantics are made non-zero."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (estimated pending `make pm-lift-intake`)

```text
Lift Score (v1): ~43
Estimated slices: 4
Confidence: medium
Likely split triggers:
- implementation requires both prompt-worker recovery logic and macOS-specific revoke regression coverage
- exit-code/contract wording may need a narrow docs update alongside code
```

## 10. Open questions

- What is the narrowest shell-local mechanism that reliably makes the spinning Reedline worker unwind after TTY revoke on macOS?
  - This is the main remaining technical unknown.
- Can the disconnect be classified generically enough to share logic across Unix while still proving the bug with a macOS-only test?
  - Recommended default: yes.
- Are there any other authoritative REPL contract surfaces beyond ADR-0016, `docs/reference/env/contract.md`, and `docs/USAGE.md` that restate REPL process exit semantics and therefore need the same wording adjustment?
  - Recommended default: treat those three as the minimum required touch set and update any additional mirrors only if they state the contract normatively.
