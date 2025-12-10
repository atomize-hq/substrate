# Kickoff Prompt – M1-integ (Lima migration & socket parity)

## Scope
- Merge `M1-code` + `M1-test`, resolve drift against M1-spec, and ensure migration/socket parity is green. Integration owns fmt/clippy/tests + `make preflight`.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M1-spec.md, this prompt.
3. Set `M1-integ` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M1-integ`).
5. Create branch `mp-m1-migration-integ` and worktree `wt/mp-m1-migration-integ`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Merge code/test branches, reconcile behavior to M1-spec (migration, socket perms, idempotent warm/provision).
- Run required commands (capture outputs in END log):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`  
  - Relevant tests (at minimum the suites added in M1-test; document any mac-only skips)  
  - `make preflight`
- Ensure doctor/log guidance matches spec and that no regressions appear on non-mac platforms.

## End Checklist
1. Run required commands above.
2. Commit integration changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m1-migration-integ`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M1-integ`).
5. Remove worktree `wt/mp-m1-migration-integ`.
