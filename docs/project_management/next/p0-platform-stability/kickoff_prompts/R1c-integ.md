# Task R1c-integ (Replay world coverage) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Ensure `R1c-code` and `R1c-test` are merged/ready.
3. Set `R1c-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start R1c-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-r1c-coverage-integ
   git worktree add wt/ps-r1c-coverage-integ ps-r1c-coverage-integ
   cd wt/ps-r1c-coverage-integ
   ```

## Responsibilities
- Merge `ps-r1c-coverage-code` + `ps-r1c-coverage-test`, resolve conflicts, fast-forward to `feat/p0-platform-stability` after checks pass.
- Re-run fmt/lint/tests plus targeted `substrate --replay` smoke commands covering world-on/off cases.
- Update docs/tasks/session log, then publish the H1a prompts.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-replay -- --nocapture
cargo test -p substrate-shell replay_world
substrate --replay --replay-verbose --no-world --span <fixture>   # capture/skip rationale
```

## End Checklist
1. Confirm commands succeeded (document skips).
2. Merge integration branch into `feat/p0-platform-stability`.
3. Update `tasks.json` (`R1c-integ` → completed) + session log END entry summarizing results.
4. Ensure H1a-code/test prompts incorporate relevant assumptions from R1.
5. Commit doc/task/log updates (`git commit -am "docs: finish R1c-integ"`), remove worktree, and hand off to H1.


Do not edit planning docs inside the worktree.
