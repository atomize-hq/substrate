# Kickoff Prompt – M2-integ (Installer parity – macOS)

## Scope
- Merge `M2-code` + `M2-test`, reconcile behavior to M2-spec, and gate with fmt/clippy/tests + `make preflight`. Integration owns final mac installer/uninstaller parity.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M2-spec.md, this prompt.
3. Set `M2-integ` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M2-integ`).
5. Create branch `mp-m2-installer-integ` and worktree `wt/mp-m2-installer-integ`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Merge code/test branches and ensure mac installer/uninstaller parity matches M2-spec (agent build/copy fallback, CLI shim stance, cleanup-state metadata).
- Run required commands (capture outputs in END log):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`  
  - Relevant tests (at minimum those added in M2-test; note mac-only skips)  
  - `make preflight`
- Confirm Linux/WSL installers remain unaffected.

## End Checklist
1. Run required commands above.
2. Commit integration changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m2-installer-integ`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M2-integ`).
5. Remove worktree `wt/mp-m2-installer-integ`.
