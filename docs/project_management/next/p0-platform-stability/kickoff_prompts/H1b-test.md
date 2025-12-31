# Task H1b-test (Health manager parity – UX & docs) – TEST

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, H1b-code scope, and this prompt.
3. Set `H1b-test` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start H1b-test"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-h1b-healthux-test
   git worktree add wt/ps-h1b-healthux-test ps-h1b-healthux-test
   cd wt/ps-h1b-healthux-test
   ```

## Spec
- Update CLI/integration tests verifying `substrate health` text + JSON outputs for host-only/world-only/both-missing scenarios, ensuring severities align with the new rules.
- Cover the `manager_states`, `attention_required_managers`, and `world_only_managers` fields in JSON fixtures so telemetry regressions are caught.
- Cover PowerShell/macOS fixtures if they include sample outputs.
- Validate doctor summary tests pick up the new phrasing and do not regress older behavior.

## Scope & Guardrails
- Tests/fixtures only; keep them hermetic via mocked manager states.
- Document any manual `substrate health --json` runs executed to compare outputs.
- Avoid touching production docs except for helper references.

## Required Commands
```
cargo fmt
cargo test -p substrate-shell health
substrate health --json   # via harness or real CLI; log skip if unavailable
```

## End Checklist
1. Ensure fmt/tests/manual commands complete; document skips.
2. Commit changes (e.g., `test: verify refined health manager UX`).
3. Merge `ps-h1b-healthux-test` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm H1b-integ prompt still matches.
6. Commit doc/task/log updates (`git commit -am "docs: finish H1b-test"`), remove worktree, hand off.


Do not edit planning docs inside the worktree.
