# Kickoff Prompt – LP1-code (Linux world provision parity fix)

## Scope
- Production code/scripts/docs only; no tests. Implement LP1-spec: fix `scripts/linux/world-provision.sh` so standalone provisioning matches installer behavior (SocketGroup=substrate, group add, linger guidance) and update referenced docs/helpers.
- Keep changes Linux-specific; avoid regressions on other platforms.

## Start Checklist
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, LP1-spec.md, this prompt.
3. Set `LP1-code` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start LP1-code`).
5. Create branch `ps-lp1-provision-code` and worktree `wt/ps-lp1-provision-code`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Update `scripts/linux/world-provision.sh` to create/add the `substrate` group, set `SocketGroup=substrate`, recreate `/run/substrate.sock` as `root:substrate 0660`, and emit linger guidance; keep `--skip-build`/profile handling and non-root invocation (sudo escalation) intact.
- Update references (e.g., WORLD/INSTALLATION manuals, world-socket-verify) to reflect the corrected behavior; keep installers stable.
- Required commands:  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Optional: shellcheck/dry-run logs as sanity checks (record in END entry if run).

## End Checklist
1. Run required commands above.
2. Commit worktree changes.
3. `git checkout feat/p0-platform-stability && git pull --ff-only && git merge --ff-only ps-lp1-provision-code`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish LP1-code`).
5. Remove worktree `wt/ps-lp1-provision-code`.
