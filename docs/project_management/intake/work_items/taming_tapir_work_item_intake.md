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
- The user-facing contract change is narrow: when the interactive TTY disappears, Substrate should stop running interactively instead of busy-spinning.
- No new CLI/config surface is required.

## 4. Task definition (bounded)

### 4.1 Problem statement (observed)

On macOS, Substrate interactive REPL processes can peg a CPU core (~100%) indefinitely when their controlling TTY is revoked while Reedline/crossterm is active. In practice this shows up as orphaned `substrate --async-repl` processes reparented to `launchd` and consuming sustained CPU (Activity Monitor), even though no work is happening.

Evidence (example stack signature):
- `substrate_shell::repl::async_repl::run_prompt_worker`
  - `reedline::engine::Reedline::read_line`
    - `crossterm::event::poll`

A reliable repro exists by spawning Substrate under a PTY, allowing the prompt to render, then invoking `revoke(2)` on the PTY slave device path (e.g., `/dev/ttys154`). After revoke, CPU ramps to ~100% and remains pinned until the process is terminated.

### 4.2 Intended behavior (target)

When Substrate is running interactively and the controlling TTY becomes invalid (revoked / disconnected):
- Substrate must stop the interactive prompt loop and exit promptly (no busy-spin).
- Best-effort print a single diagnostic line to stderr describing the terminal disconnect.
- Prefer exiting with a non-zero runtime failure code (not a “user error” code) unless we intentionally treat this as a normal hangup path.

Non-goals:
- Do not attempt to keep the REPL running “headless” after terminal loss.
- Do not add new CLI flags/config keys for this behavior.

### 4.3 Constraints (must hold)

- Maintain existing REPL behavior under normal interactive use.
- Keep the fix bounded to shell REPL input handling (no world-agent or shim changes).
- Ensure test harnesses cannot leave behind CPU-pinning orphan processes when a test is interrupted/aborted.

## 5. Done means (<= 8 outcomes)

- When the controlling TTY is revoked/disconnects while the async REPL is idle, Substrate exits promptly instead of spinning.
- The fix covers the common macOS failure mode (revoked PTY) and does not regress normal interactive behavior.
- A targeted macOS integration test reproduces the revoke/disconnect scenario and asserts the process exits (no indefinite hang).
- The test suite does not leave behind orphaned `substrate --async-repl` processes on failure paths.
- A short note is added to user-facing docs describing the expected behavior (terminal disconnect implies exit) if needed for operator clarity.
- No new CLI flags/config keys are introduced.

## 6. Likely touch paths

- REPL prompt worker / error handling:
  - `crates/shell/src/repl/async_repl.rs`
  - `crates/shell/src/repl/editor.rs` (only if we need to adjust Reedline wiring)
- New regression test (macOS-only or unix-gated):
  - `crates/shell/tests/`
- Optional docs clarification:
  - `docs/USAGE.md`

## 7. Dependencies (ADR/WI)

- depends_on_adrs: []
- depends_on_work_items: []
- blocks: []

## 8. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 1,
    "edit_files": 2,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 1,
    "boundary_crossings": 2
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
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
    "unknowns_high": 1
  },
  "notes": "Estimate assumes a bounded shell-only fix (prompt worker termination on revoked/disconnected TTY), one new macOS regression test, and an optional short docs note."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (from `make pm-lift-intake`)

```text
Lift Score (v1): 29
Estimated slices: 3
Confidence: high
Triggers:
- likely_split:lift_score>24
```

## 9. Open questions

- Exit code: should a revoked/disconnected TTY map to a clean hangup exit (`0`) or a runtime failure exit (`1`)?
- Scope: should the fix be macOS-specific (revoke-driven) or should we generalize to “TTY invalid” detection across Unix platforms?
- Detection: do we treat specific error strings/codes from crossterm/reedline (e.g., `ENOTTY` / “Inappropriate ioctl for device”) as terminal-loss signals, or do we implement explicit TTY health checks on a backoff?
