# Kickoff Prompt – M1-code (Lima socket parity)

## Scope
- Production code/scripts only; no tests. Implement the M1-spec: rebuild/replace the Lima environment with the socket-activated world-agent layout (no backwards compatibility or user carry-over), install/refresh the agent binary, align socket perms/group access, and make warm/provision idempotent with clear diagnostics.
- Keep changes mac-specific; avoid regressions on other platforms.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M1-spec.md, this prompt.
3. Set `M1-code` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M1-code`).
5. Create branch `mp-m1-sockets-code` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/mp-m1-sockets-code mp-m1-sockets-code`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Implement replacement + idempotent warm/provisioning per M1-spec; surface actionable errors when prerequisites (agent, units, toolchain, Lima) are missing.
- Align socket ownership/permissions and user access model with documented mac behavior (root/substrate or documented equivalent).
- Ensure the Lima provisioning path (including any reuse of `scripts/linux/world-provision.sh` in-guest) sets `SocketGroup=substrate`, adds the SSH user to the `substrate` group, and emits linger guidance so socket activation survives logout.
- Keep changes confined to mac flows (Lima profiles, warm/provision scripts, mac doctor hooks as needed).
- Protected paths: do not touch `.git`, device nodes, or unrelated host paths.
- Required commands (before handoff):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Optional sanity checks allowed (e.g., script dry-runs), but no required tests.

## End Checklist
1. Run the required commands above and capture their outputs.
2. Inside `wt/mp-m1-sockets-code`, commit M1-code changes to branch `mp-m1-sockets-code` (no docs/tasks/session_log.md edits).
3. From outside the worktree, ensure branch `mp-m1-sockets-code` contains the worktree commit (fast-forward if needed); do **not** merge into `feat/p0-platform-stability-macOS-parity`.
4. Checkout `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; create downstream prompts if missing; commit docs (`docs: finish M1-code`).
5. Remove worktree `wt/mp-m1-sockets-code`.
