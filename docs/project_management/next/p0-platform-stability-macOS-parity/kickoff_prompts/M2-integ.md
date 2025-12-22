# Kickoff Prompt – M2-integ (Installer parity – macOS)

## Scope
- Merge `M2-code` + `M2-test`, reconcile behavior to M2-spec, and gate with fmt/clippy/tests + `make preflight`. Integration owns final mac installer/uninstaller parity.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M2-spec.md, this prompt.
3. Set `M2-integ` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M2-integ`).
5. Create branch `mp-m2-installer-integ` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/mp-m2-installer-integ mp-m2-installer-integ`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Merge code/test branches and ensure mac installer/uninstaller parity matches M2-spec (prod copy-first with build fallback when bundle invalid/missing; dev build-in-guest path; CLI shim stance; cleanup-state metadata).
- Run required commands (capture outputs in END log):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`  
  - Relevant tests (at minimum those added in M2-test; note mac-only skips)  
  - `make preflight`
- Confirm Linux/WSL installers remain unaffected.

## End Checklist
1. Merge the upstream code/test branches for M2 into `mp-m2-installer-integ` inside `wt/mp-m2-installer-integ` and reconcile behavior to the spec.
2. Run the required commands (cargo fmt; cargo clippy --workspace --all-targets -- -D warnings; relevant tests; make preflight) and capture outputs.
3. Commit integration changes on branch `mp-m2-installer-integ`.
4. Fast-forward merge `mp-m2-installer-integ` into `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; commit docs (`docs: finish M2-integ`).
5. Remove worktree `wt/mp-m2-installer-integ`.
