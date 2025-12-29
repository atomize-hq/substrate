# Task S1b-integ (Socket activation – shell readiness) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Ensure `S1b-code` and `S1b-test` are ready.
3. Set `S1b-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1b-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1b-shell-integ
   git worktree add wt/ps-s1b-shell-integ ps-s1b-shell-integ
   cd wt/ps-s1b-shell-integ
   ```

## Responsibilities
- Merge `ps-s1b-shell-code` + `ps-s1b-shell-test`, resolve conflicts, and fast-forward `feat/p0-platform-stability` after validation.
- Re-run fmt/lint/tests plus a representative `substrate world doctor --json` (or documented skip) to validate combined behavior.
- Update docs/tasks/session log and hand off to the provisioning phase (S1c).

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_enable
substrate world doctor --json   # capture output/skip rationale
```

## End Checklist
1. Confirm commands (and doctor run) completed; log skips.
2. Merge the integration branch back into `feat/p0-platform-stability`.
3. Update `tasks.json` (`S1b-integ` → completed) and append END session entry summarizing commands/results.
4. Ensure S1c-code/test prompts reference any new assumptions; edit if required.
5. Commit doc/task/log updates (`git commit -am "docs: finish S1b-integ"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
