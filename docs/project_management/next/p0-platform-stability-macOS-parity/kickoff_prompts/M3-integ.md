# Kickoff Prompt – M3-integ (Backend & doctor parity)

## Scope
- Merge `M3-code` + `M3-test`, reconcile to M3-spec, and gate with fmt/clippy/tests + `make preflight`. Integration owns final mac backend/doctor parity.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M3-spec.md, this prompt.
3. Set `M3-integ` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M3-integ`).
5. Create branch `mp-m3-backend-integ` and worktree `wt/mp-m3-backend-integ`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Merge code/test branches, ensure mac fs_mode propagation, forwarding order, and doctor/docs outputs match M3-spec.
- Run required commands (capture outputs in END log):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`  
  - Relevant tests (at minimum those added in M3-test; note mac-only skips)  
  - `make preflight`
- Confirm non-mac platforms remain green.

## End Checklist
1. Run required commands above.
2. Commit integration changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m3-backend-integ`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M3-integ`).
5. Remove worktree `wt/mp-m3-backend-integ`.
