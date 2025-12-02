# Task H1b-code (Health manager parity – UX & docs) – CODE

## Start Checklist (feat/p0-platform-stability)
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read `p0_platform_stability_plan.md`, `tasks.json`, `session_log.md`, H1a outputs, and this prompt.
3. Set `H1b-code` to `in_progress`, append START entry, commit doc update (`git commit -am "docs: start H1b-code"`).
4. Create branch/worktree:
   ```
   git checkout -b ps-h1b-healthux-code
   git worktree add wt/ps-h1b-healthux-code ps-h1b-healthux-code
   cd wt/ps-h1b-healthux-code
   ```

## Spec
- Update `substrate health` text + JSON output so severity labels convey host-only/world-only/both-missing manager states, with actionable next steps.
- Ensure doctor summary sections (and any aggregated outputs) highlight mismatches without flagging missing managers that the host lacks.
- Refresh docs (`docs/USAGE.md`, `docs/CONFIGURATION.md`, troubleshooting guides) with new examples (Linux/macOS/WSL, POSIX + PowerShell) describing the refined behavior.

## Scope & Guardrails
- Production CLI/doc changes only; tests handled in H1b-test.
- Keep JSON schemas compatible; add new fields under feature-flag or documented expansions.
- When touching docs, include both POSIX and Windows examples.

## Required Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell health
substrate health --json   # capture updated output
```

## End Checklist
1. Ensure fmt/clippy/tests/manual commands completed; document skips.
2. Commit changes (e.g., `feat: polish health manager UX`).
3. Merge `ps-h1b-healthux-code` into `feat/p0-platform-stability`.
4. Update `tasks.json` + `session_log.md` END entry summarizing results.
5. Confirm H1b-integ prompt remains accurate.
6. Commit doc/task/log updates (`git commit -am "docs: finish H1b-code"`), remove worktree, hand off.
