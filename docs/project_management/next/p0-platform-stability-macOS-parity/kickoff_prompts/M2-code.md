# Kickoff Prompt – M2-code (Installer parity – macOS)

## Scope
- Production code/scripts only; no tests. Implement M2-spec: dev/prod installer parity on mac (agent build/copy fallback in Lima, optional CLI shim, metadata/cleanup-state parity, uninstall cleanup). Keep Linux/WSL behaviors intact.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M2-spec.md, this prompt.
3. Set `M2-code` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M2-code`).
5. Create branch `mp-m2-installer-code` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/mp-m2-installer-code mp-m2-installer-code`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Update mac dev/prod installers to prefer copying the bundled Linux agent into Lima for prod; build in-guest only when the bundle is missing/invalid (dev may build in-guest by design); fail loudly with guidance otherwise.
- Keep CLI shim behavior consistent across dev/prod if required for diagnostics; ensure uninstall removes guest/host artifacts and honors cleanup-state metadata.
- Preserve/align installer metadata and logging with Linux P0 behavior; log whether copy vs build path was taken.
- Required commands:  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Optional sanity checks allowed (installer dry-runs), but no required tests.

## End Checklist
1. Run the required commands above and capture their outputs.
2. Inside `wt/mp-m2-installer-code`, commit M2-code changes to branch `mp-m2-installer-code` (no docs/tasks/session_log.md edits).
3. From outside the worktree, ensure branch `mp-m2-installer-code` contains the worktree commit (fast-forward if needed); do **not** merge into `feat/p0-platform-stability-macOS-parity`.
4. Checkout `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; create downstream prompts if missing; commit docs (`docs: finish M2-code`).
5. Remove worktree `wt/mp-m2-installer-code`.
