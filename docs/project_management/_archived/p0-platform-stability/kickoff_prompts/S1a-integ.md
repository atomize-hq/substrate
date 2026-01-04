# Task S1a-integ (Socket activation – agent plumbing) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Confirm `S1a-code` and `S1a-test` are merged/ready.
3. Set `S1a-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1a-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1a-agent-integ
   git worktree add wt/ps-s1a-agent-integ ps-s1a-agent-integ
   cd wt/ps-s1a-agent-integ
   ```

## Responsibilities
- Merge `ps-s1a-agent-code` + `ps-s1a-agent-test`, resolve conflicts, and fast-forward back onto `feat/p0-platform-stability` after validation.
- Re-run fmt/lint/tests to ensure combined changes are green.
- Update docs/tasks/session log and ready the S1b prompts.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p world-agent
```
Document any supplemental smoke commands or skips.

## End Checklist
1. Confirm commands succeeded (note skips if any) and include logs in the session entry.
2. Fast-forward merge the integration branch into `feat/p0-platform-stability`.
3. Update `tasks.json` (`S1a-integ` → completed) and append END session log entry summarizing commands/results.
4. Ensure S1b-code/test kickoff prompts reference the now-available agent behavior; edit if required.
5. Commit doc/task/log updates (`git commit -am "docs: finish S1a-integ"`), remove worktree, hand off to S1b.


Do not edit planning docs inside the worktree.
