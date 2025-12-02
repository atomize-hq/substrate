# Task S1c-integ (Socket activation – provisioning & docs) – INTEGRATION

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Confirm `S1c-code` and `S1c-test` are ready.
3. Set `S1c-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1c-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1c-provision-integ
   git worktree add wt/ps-s1c-provision-integ ps-s1c-provision-integ
   cd wt/ps-s1c-provision-integ
   ```

## Responsibilities
- Merge `ps-s1c-provision-code` + `ps-s1c-provision-test`, resolve conflicts, and fast-forward back to `feat/p0-platform-stability`.
- Re-run fmt/lint/tests plus representative provisioning dry-runs to validate combined changes.
- Update docs/tasks/session log and publish the R1a prompts.

## Required Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
./tests/installers/install_smoke.sh   # or record skip
scripts/linux/world-provision.sh --profile dev --dry-run
scripts/mac/lima-warm.sh --check-only   # skip ok w/notes
pwsh -File scripts/windows/wsl-warm.ps1 -WhatIf   # skip ok w/notes
```

## End Checklist
1. Confirm commands completed; note skips in log.
2. Merge integrated branch back into `feat/p0-platform-stability`.
3. Update `tasks.json` (`S1c-integ` → completed) and append END session log entry summarizing commands/results.
4. Ensure R1a-code/test prompts are present/accurate; edit if provisioning discoveries affect them.
5. Commit doc/task/log updates (`git commit -am "docs: finish S1c-integ"`), remove worktree, and hand off to replay work.
