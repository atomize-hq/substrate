# Task R1a-integ (Replay isolation fallback) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Verify `R1a-code` and `R1a-test` are complete.
3. Set `R1a-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1a-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1a-isolation-integ
   git worktree add wt/ps-r1a-isolation-integ ps-r1a-isolation-integ
   cd wt/ps-r1a-isolation-integ
   ```

## Responsibilities
- Merge `ps-r1a-isolation-code` + `ps-r1a-isolation-test`, resolve conflicts, fast-forward back to `feat/p0-platform-stability` once checks pass.
- Re-run fmt/lint/tests and capture any manual cleanup scripts or replay commands executed.
- Update docs/tasks/session log and tee up the R1b prompts.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
```
Add any manual `substrate --replay ...` runs or cleanup helper invocations to the log.

## End Checklist
1. Confirm commands succeeded (document skips as needed).
2. Merge integration branch back into `feat/p0-platform-stability`.
3. Update `tasks.json` (`R1a-integ` → completed) and append END session log entry summarizing results.
4. Ensure R1b-code/test prompts reflect any changes exposed during integration.
5. Commit doc/task/log updates (`git commit -am "docs: finish R1a-integ"`), remove worktree, hand off to R1b.
