# Task S1e-integ (Installer state tracking & cleanup) – INTEGRATION

## Start Checklist (feat/p0-platform-stability-follow-up)
1. `git checkout feat/p0-platform-stability-follow-up && git pull --ff-only`
2. Confirm `S1e-code` + `S1e-test` completed (review commits + session log).
3. Set `S1e-integ` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start S1e-integ"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-s1e-installer-integ
   git worktree add wt/ps-s1e-installer-integ ps-s1e-installer-integ
   cd wt/ps-s1e-installer-integ
   ```

## Spec
- Merge the installer metadata code and test branches, resolve conflicts, and fast-forward onto `feat/p0-platform-stability-follow-up`.
- Re-run formatting, shellcheck, and the installer harness/tests introduced in S1e-test; capture outputs or justified skips.
- Update docs/tasks/session log with the final status and tee up the interactive installer work.

## Scope & Guardrails
- Integration only: no new feature/test logic.
- Log every required command; if a harness can’t run on this host, record the skip reason in the session log.
- Remove the integration worktree after committing.

## Required Commands
```
cargo fmt
shellcheck scripts/substrate/dev-install-substrate.sh scripts/substrate/install-substrate.sh scripts/substrate/dev-uninstall-substrate.sh scripts/substrate/uninstall-substrate.sh
./tests/installers/install_state_smoke.sh
./tests/installers/install_smoke.sh --scenario dev
./tests/installers/install_smoke.sh --scenario prod
``` 

## End Checklist
1. Merge code/test branches and fast-forward `feat/p0-platform-stability-follow-up`.
2. Run fmt/shellcheck/tests; record outputs (or skips with justification).
3. Update `tasks.json` + `session_log.md` END entry (commands, results, next prompts).
4. Confirm downstream kickoff prompts (interactive installer work) or note TODOs.
5. Commit doc/task/log updates (`git commit -am "docs: finish S1e-integ"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
