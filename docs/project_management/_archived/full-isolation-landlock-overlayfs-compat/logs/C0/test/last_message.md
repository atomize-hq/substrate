- Added fail-closed integration coverage for the Landlock exec wrapper in `crates/world-agent/tests/landlock_exec_fail_closed.rs:1` (expects exit code `4` + no inner command execution when mountinfo derivation can’t satisfy the C0 prerequisites).
- Tightened the existing full-isolation allowlist integration test to only run when Landlock is supported in `crates/world-agent/tests/full_isolation_nonpty.rs:1`.
- Added fixture-based mountinfo parsing tests + fixtures in `crates/world/tests/mountinfo_parsing.rs:1` and `crates/world/tests/fixtures/mountinfo/overlay_ok_with_escapes.mountinfo:1`.

- Ran `cargo fmt` (ok).
- Ran `cargo test -p world --test mountinfo_parsing -- --nocapture` (ok).
- Ran `cargo test -p world-agent --test landlock_exec_fail_closed -- --nocapture` (fails as intended pre-fix: wrapper exits `0`/runs inner cmd; expected fail-closed `4`).
- Ran `cargo test -p world-agent --test full_isolation_nonpty -- --nocapture` (ok; skipped due to missing root/overlay).

- Finished task: `make triad-task-finish TASK_ID="C0-test"` (commit `5fee99a14e0061671df25502491929fca1f65ad7`).