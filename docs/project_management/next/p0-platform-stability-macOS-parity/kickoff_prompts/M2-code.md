# Kickoff Prompt – M2-code (Installer parity – macOS)

## Scope
- Production code/scripts only; no tests. Implement M2-spec: dev/prod installer parity on mac (agent build/copy fallback in Lima, optional CLI shim, metadata/cleanup-state parity, uninstall cleanup). Keep Linux/WSL behaviors intact.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M2-spec.md, this prompt.
3. Set `M2-code` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M2-code`).
5. Create branch `mp-m2-installer-code` and worktree `wt/mp-m2-installer-code`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Update mac dev/prod installers to build/install the Linux agent inside Lima when not bundled; copy when present; fail loudly with guidance otherwise.
- Keep CLI shim behavior consistent across dev/prod if required for diagnostics; ensure uninstall removes guest/host artifacts and honors cleanup-state metadata.
- Preserve/align installer metadata and logging with Linux P0 behavior.
- Required commands:  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Optional sanity checks allowed (installer dry-runs), but no required tests.

## End Checklist
1. Run required commands above.
2. Commit worktree changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m2-installer-code`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M2-code`).
5. Remove worktree `wt/mp-m2-installer-code`.
