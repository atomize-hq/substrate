# Kickoff Prompt – M1-code (Lima migration & socket parity)

## Scope
- Production code/scripts only; no tests. Implement the M1-spec: migrate existing Lima VMs to the socket-activated world-agent layout, install/refresh the agent binary, align socket perms/group access, and make warm/provision idempotent with clear diagnostics.
- Keep changes mac-specific; avoid regressions on other platforms.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M1-spec.md, this prompt.
3. Set `M1-code` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M1-code`).
5. Create branch `mp-m1-migration-code` and worktree `wt/mp-m1-migration-code`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Implement migration + idempotent warm/provisioning per M1-spec; surface actionable errors when prerequisites (agent, units, toolchain, Lima) are missing.
- Align socket ownership/permissions and user access model with documented mac behavior (root/substrate or documented equivalent).
- Ensure the Lima provisioning path (including any reuse of `scripts/linux/world-provision.sh` in-guest) sets `SocketGroup=substrate`, adds the SSH user to the `substrate` group, and emits linger guidance so socket activation survives logout.
- Keep changes confined to mac flows (Lima profiles, warm/provision scripts, mac doctor hooks as needed).
- Protected paths: do not touch `.git`, device nodes, or unrelated host paths.
- Required commands (before handoff):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Optional sanity checks allowed (e.g., script dry-runs), but no required tests.

## End Checklist
1. Run required commands above.
2. Commit worktree changes.
3. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only mp-m1-migration-code`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish M1-code`).
5. Remove worktree `wt/mp-m1-migration-code`.
