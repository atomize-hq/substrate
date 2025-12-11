# Kickoff Prompt – LP1-integ (Linux world provision parity fix)

## Scope
- Merge `LP1-code` + `LP1-test`, reconcile to LP1-spec, and gate with fmt/clippy/tests + `make preflight`. Integration owns final provisioner parity.

## Start Checklist
1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read: plan.md, tasks.json, session_log.md, LP1-spec.md, this prompt.
3. Set `LP1-integ` status to `in_progress` in tasks.json (orchestration branch only).
4. Add START entry to session_log.md; commit docs (`docs: start LP1-integ`).
5. Create branch `ps-lp1-provision-integ` and worktree `wt/ps-lp1-provision-integ`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## Requirements
- Merge code/test branches; ensure `scripts/linux/world-provision.sh` matches spec (SocketGroup=substrate, group add, linger guidance) and docs/helpers reflect the behavior.
- Run required commands (capture outputs in END log):  
  - `cargo fmt`  
  - `cargo clippy --workspace --all-targets -- -D warnings`  
  - Relevant tests/harnesses added in LP1-test (note privileged skips)  
  - `make preflight`

## End Checklist
1. Run required commands above.
2. Commit integration changes.
3. `git checkout feat/p0-platform-stability && git pull --ff-only && git merge --ff-only ps-lp1-provision-integ`
4. Update tasks.json status to completed; add END entry to session_log.md with commands/results; commit docs (`docs: finish LP1-integ`).
5. Remove worktree `wt/ps-lp1-provision-integ`.
