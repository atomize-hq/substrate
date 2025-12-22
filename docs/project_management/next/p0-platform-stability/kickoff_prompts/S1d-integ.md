# Task S1d-integ (Installer socket-activation parity) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm `S1d-code` and `S1d-test` branches are ready/merged into feat/p0-platform-stability-follow-up.
3. Set `S1d-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1d-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1d-devinstall-integ
   git worktree add wt/ps-s1d-devinstall-integ ps-s1d-devinstall-integ
   cd wt/ps-s1d-devinstall-integ
   ```

## Responsibilities
- Merge `ps-s1d-devinstall-code` and `ps-s1d-devinstall-test`, resolve conflicts, and fast-forward back to `feat/p0-platform-stability-follow-up` once all checks pass.
- Re-run `cargo fmt`, `shellcheck` across the installer scripts, and both dev/prod installer smoke scenarios, logging results and justified skips.
- Update docs/tasks/session log with the integrated installer instructions, including substrate group creation steps and lingering guidance for both dev and production paths.

## Required Commands
```
cargo fmt
shellcheck scripts/substrate/dev-install-substrate.sh scripts/substrate/dev-uninstall-substrate.sh scripts/substrate/install-substrate.sh scripts/substrate/uninstall-substrate.sh
./tests/installers/install_smoke.sh --scenario dev
./tests/installers/install_smoke.sh --scenario prod
```

## End Checklist
1. Confirm all commands succeeded; document skips with justification.
2. Merge the integration branch back into `feat/p0-platform-stability-follow-up` (fast-forward).
3. Update `tasks.json` (`S1d-*` statuses) and append an END session log entry summarizing smoke results plus lingering/group notes.
4. Document hand-off/next steps (e.g., Windows WhatIf task) in the session log.
5. Commit doc/task/log updates (`git commit -am "docs: finish S1d-integ"`), remove the worktree, and notify stakeholders that dev installer parity is in place on `feat/p0-platform-stability-follow-up`.
