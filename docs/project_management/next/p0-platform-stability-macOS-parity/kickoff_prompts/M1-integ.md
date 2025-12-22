# Kickoff Prompt – M1-integ (Lima socket parity)

## Scope
- Merge `M1-code` + `M1-test`, resolve drift against M1-spec, and ensure the socket parity replacement flow is green. Integration owns fmt/clippy/tests + `make preflight`.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M1-spec.md, this prompt.
3. Set `M1-integ` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M1-integ`).
5. Create branch `mp-m1-sockets-integ` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/mp-m1-sockets-integ mp-m1-sockets-integ`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Merge code/test branches, reconcile behavior to M1-spec (socket parity replacement, group perms, idempotent warm/provision).
- Run required commands (capture outputs in END log):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`  
  - Relevant tests (at minimum the suites added in M1-test; document any mac-only skips)  
  - `make preflight`
- Ensure doctor/log guidance matches spec and that no regressions appear on non-mac platforms.

## End Checklist
1. Merge the upstream code/test branches for M1 into `mp-m1-sockets-integ` inside `wt/mp-m1-sockets-integ` and reconcile behavior to the spec.
2. Run the required commands (cargo fmt; cargo clippy --workspace --all-targets -- -D warnings; relevant tests; make preflight) and capture outputs.
3. Commit integration changes on branch `mp-m1-sockets-integ`.
4. Fast-forward merge `mp-m1-sockets-integ` into `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; commit docs (`docs: finish M1-integ`).
5. Remove worktree `wt/mp-m1-sockets-integ`.
