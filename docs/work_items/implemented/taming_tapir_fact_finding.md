# Taming Tapir Fact Finding

This note captures the repo facts and pre-implementation decisions for `taming_tapir`.

Primary intake:
- `docs/project_management/intake/work_items/taming_tapir_work_item_intake.md`

## 1. Scope summary

Goal:
- prevent the async REPL from busy-spinning on macOS when its controlling TTY is revoked/disconnected
- exit cleanly
- avoid orphaned REPL processes

Strictly out of scope:
- world-service changes
- shim changes
- new CLI/config knobs
- broad REPL redesign

## 2. Repo-verified facts

### 2.1 Prompt-worker architecture

- `crates/shell/src/repl/async_repl.rs`
  - `PromptWorker::spawn()` chooses Reedline only when stdin is a TTY and the child process is not in CI.
  - `run_prompt_worker()` is a dedicated thread that blocks inside `line_editor.read_line(&prompt)`.
  - The async REPL loop consumes `PromptWorkerResponse` messages over a Tokio channel.

- `crates/shell/src/repl/editor.rs`
  - Substrate enables Reedline `external_printer`.
  - This means the active editor path depends on Reedline/crossterm polling while the prompt is idle.

### 2.2 Why the current generic error branch is not enough

- The current async loop already handles `PromptWorkerResponse::Error(err)`.
- For non-cursor-timeout prompt errors it prints `prompt error: ...` and exits the loop.
- That is not enough for the reported macOS bug because the failure signature says the worker can spin inside `Reedline::read_line()` and never return an error message to the outer loop.

Implication:
- fixing only the outer `PromptWorkerResponse::Error` handling is unlikely to solve the revoke path.

### 2.3 Exit-code reality today

- `run_async_repl()` returns the post-loop `auto_sync_exit_code`.
- In normal host-only runs that is typically `0`.
- Therefore prompt failure after startup currently collapses to a successful process exit unless auto-sync independently fails.

Implication:
- if terminal loss should be non-zero, the implementation needs an explicit session termination code path.

### 2.4 Shutdown hazard

- `PromptWorker::shutdown()` sends `Shutdown` over the worker command channel and then `join()`s the worker thread.
- If the worker is still stuck inside Reedline and never returns to `blocking_recv()`, this join path can hang.

Implication:
- any out-of-band disconnect detector must also make the prompt worker unwind, not just mark the REPL loop as done.

### 2.5 Test harness facts

- Existing REPL PTY tests already use:
  - `portable_pty`
  - non-blocking master reads via `fcntl(..., O_NONBLOCK)`
  - timeout-based waits
  - `#[serial]`
- Existing PTY harness code is duplicated across multiple files:
  - `crates/shell/tests/repl_config_max_pty_buffered_lines.rs`
  - `crates/shell/tests/repl_output_routing.rs`
  - `crates/shell/tests/wfgadax3_repl_exit_transparency.rs`
  - others

Important test-path constraint:
- `PromptWorker::spawn()` falls back to stdio if the child sees `CI` or `GITHUB_ACTIONS`.
- A revoke regression test must explicitly remove those env vars from the spawned child or it will miss the Reedline path.

### 2.6 Contract docs implicated by a non-zero disconnect exit

- `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - currently frames non-zero REPL exits as startup/config failures
- `docs/reference/env/contract.md`
  - shell entrypoint errors map to exit code `1`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - code `1` is the canonical unexpected runtime/internal failure code

Implication:
- if terminal loss is treated as exit `1`, ADR-0016 should be narrowed from:
  - "non-zero only on startup/config errors"
  to:
  - "0 on normal operator exit; non-zero on abnormal runtime termination"

## 3. Recommended decisions before coding

### DR-TT-01 — Exit status on terminal loss

Option A:
- Exit `0` to preserve the broadest possible reading of the current REPL contract.

Option B:
- Exit `1` because revoked/disconnected TTY is not a normal operator exit and should not look successful to automation.

Decision:
- Option B.

Rationale:
- `0` hides an abnormal termination.
- the canonical taxonomy already has `1` for runtime failure
- the contract change needed is narrow and easy to document

Locked result:
- normal `exit` / `quit`: process exit `0`
- revoked/disconnected controlling TTY after startup: process exit `1`

### DR-TT-02 — Scope of detection logic

Option A:
- macOS-only detection and handling

Option B:
- generic Unix TTY-invalid classification with a macOS-only regression proof

Recommendation:
- Option B.

Rationale:
- the codepath is shared across Unix
- only the reproduction primitive is macOS-specific
- this avoids baking `target_os = "macos"` into error handling unless strictly necessary

### DR-TT-03 — Recovery strategy

Option A:
- classify prompt errors only after `read_line()` returns

Option B:
- combine prompt-error classification with an explicit worker-unwind strategy for the revoked/disconnected TTY case

Recommendation:
- Option B.

Rationale:
- the observed bug is specifically that the worker may never return from Reedline/crossterm on revoke
- classification after the fact does not solve the thread-join/orphan problem

## 4. Proposed implementation file plan

Code:
- `crates/shell/src/repl/async_repl.rs`
  - terminal-loss classification
  - explicit REPL session exit code
  - worker-unwind / disconnect shutdown path

Optional code:
- `crates/shell/src/repl/editor.rs`
  - only if builder-level hooks or prompt-backend setup need to change

Tests:
- `crates/shell/tests/repl_tty_disconnect_macos.rs`
  - new macOS-only regression proof
- `crates/shell/tests/support/mod.rs`
  - touch only if revoke/cleanup helper extraction is clearly worthwhile

Docs:
- `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/reference/env/contract.md`
- `docs/USAGE.md`

## 5. Test design recommendation

Recommended child command:
- `substrate --async-repl --shim-skip`

Recommended child env:
- world disabled
- `CI` removed
- `GITHUB_ACTIONS` removed

Recommended test flow:
1. open PTY
2. spawn REPL child on the slave side
3. wait for prompt/banner
4. resolve the PTY slave path
5. call `revoke(2)` on the slave path
6. assert child exits within a bounded timeout
7. assert output contains one disconnect diagnostic line
8. always reap child on timeout/failure

Preferred isolation:
- host-only path; no world stub

Why:
- the bug is in prompt handling, not world/session execution

## 6. Remaining technical unknown

Main unknown:
- what shell-local mechanism most cleanly forces the Reedline worker to unwind once the TTY is known-bad

This should be resolved during implementation spike work before broader refactors are considered.

## 7. Things that should not expand scope

- do not add a global macOS fallback to stdio unless a bounded shell-local fix fails
- do not turn this into a broader "replace Reedline" effort
- do not mix this WI with unrelated async agent-output or PTY rendering cleanup
