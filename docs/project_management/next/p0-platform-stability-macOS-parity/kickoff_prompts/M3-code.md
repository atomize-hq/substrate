# Kickoff Prompt – M3-code (Backend & doctor parity)

## Scope
- Production code/docs only; no tests. Implement M3-spec: propagate policy fs_mode to mac backend, fix readiness/forwarding ordering, align socket/group expectations, and update doctor/manual flows accordingly. Keep other platforms stable.

## Start Checklist
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, M3-spec.md, this prompt.
3. Set `M3-code` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start M3-code`).
5. Create branch `mp-m3-backend-code` from `feat/p0-platform-stability-macOS-parity`; run `git worktree add wt/mp-m3-backend-code mp-m3-backend-code`.
6. Do **not** edit docs/tasks/session_log.md from the worktree.

## Requirements
- Honor `WorldSpec.fs_mode` across mac exec/replay (PTY/non-PTY); keep env overrides compatible.
- Ensure forwarding is established before agent probes; avoid pre-forward UDS failures.
- Align/document socket ownership/group model and reflect this in doctor/shim/health output/manuals; update mac portions of docs/manual testing playbook as needed.
- Ensure `substrate --shim-status[(-json)]` and `substrate health[ --json]` on mac surface socket activation state and manager parity in line with Linux P0 (document any mac-only caveats).
- Required commands:  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Optional sanity checks allowed; no required tests.

## End Checklist
1. Run the required commands above and capture their outputs.
2. Inside `wt/mp-m3-backend-code`, commit M3-code changes to branch `mp-m3-backend-code` (no docs/tasks/session_log.md edits).
3. From outside the worktree, ensure branch `mp-m3-backend-code` contains the worktree commit (fast-forward if needed); do **not** merge into `feat/p0-platform-stability-macOS-parity`.
4. Checkout `feat/p0-platform-stability-macOS-parity`; update tasks.json to completed; add an END entry to session_log.md with commands/results/blockers; create downstream prompts if missing; commit docs (`docs: finish M3-code`).
5. Remove worktree `wt/mp-m3-backend-code`.
