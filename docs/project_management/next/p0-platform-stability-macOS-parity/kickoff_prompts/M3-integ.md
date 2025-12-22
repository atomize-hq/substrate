# Kickoff Prompt – M3-integ (Backend & doctor parity)

## Scope
- Merge `M3-code` + `M3-test`, reconcile to M3-spec, and gate with fmt/clippy/tests + `make preflight`. Integration owns final mac backend/doctor parity.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M3-spec.md, this prompt.
3. Set `M3-integ` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M3-integ`).
5. Create branch `mp-m3-backend-integ` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/mp-m3-backend-integ mp-m3-backend-integ`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Merge code/test branches, ensure mac fs_mode propagation, forwarding order, doctor/docs outputs, and shim-status/health parity match M3-spec.
- Run required commands (capture outputs in END log):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`  
  - Relevant tests (at minimum those added in M3-test; note mac-only skips)  
  - `make preflight`
- Confirm non-mac platforms remain green.

## End Checklist
1. Merge the upstream code/test branches for M3 into `mp-m3-backend-integ` inside `wt/mp-m3-backend-integ` and reconcile behavior to the spec.
2. Run the required commands (cargo fmt; cargo clippy --workspace --all-targets -- -D warnings; relevant tests; make preflight) and capture outputs.
3. Commit integration changes on branch `mp-m3-backend-integ`.
4. Fast-forward merge `mp-m3-backend-integ` into `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; commit docs (`docs: finish M3-integ`).
5. Remove worktree `wt/mp-m3-backend-integ`.
