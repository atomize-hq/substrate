# Task R6-code (Bootstrap & builtins decomposition) – CODE

## Start Checklist (feat/crate-refactor)
1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read `refactor_plan.md`, `tasks.json`, the latest `session_log.md`,
   `CRATE_REFACTORING_ANALYSIS.md`, and this prompt.
3. Set `R6-code` to `in_progress` in `tasks.json`, add a START entry to
   `session_log.md`, and commit the docs update (`git commit -am "docs: start R6-code"`).
4. Create the task branch and worktree:
   ```
   git checkout -b cr-r6-bootstrap-code
   git worktree add wt/cr-r6-bootstrap-code cr-r6-bootstrap-code
   cd wt/cr-r6-bootstrap-code
   ```
   Do not edit docs/tasks/session logs from the worktree.

## Spec (shared with R6-test)
- Crates: `common`, `shim`, `shell` builtins.
- Split `common/src/manager_manifest.rs` into schema/resolver/validator modules
  while preserving manifest semantics and serde behavior.
- Split `shim/src/exec.rs` into bootstrap/logging/policy pieces with a thin
  entry surface; keep PATH interception/depth tracking intact.
- Decompose builtins (`shim_doctor`, `world_enable`, `world_deps`) into smaller
  modules per subcommand with shared helpers; preserve outputs/logging.

## Scope & Guardrails
- Production code and docs only; no new features or CLI/config changes.
- Maintain logging/redaction, cfg-gates, and compatibility of manifest formats
  and builtin output.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-common --all-targets
cargo test -p substrate-shim
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure fmt/clippy/tests above are green; capture outputs for the END log
   entry (note platform skips if any).
2. Commit worktree changes with a descriptive message (e.g., `refactor: split manifest/shim builtins`).
3. Return to repo root, merge the worktree branch:
   ```
   git checkout cr-r6-bootstrap-code
   git merge --ff-only wt/cr-r6-bootstrap-code   # if needed
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only cr-r6-bootstrap-code
   ```
4. Update `tasks.json` (status → `completed`), append an END entry to
   `session_log.md` (commands/results/blockers), and ensure the paired test
   prompt (`docs/project_management/next/refactor/kickoff_prompts/R6-test.md`)
   is referenced.
5. Commit docs/log updates on `feat/crate-refactor`
   (`git commit -am "docs: finish R6-code"`).
6. Remove the worktree (`git worktree remove wt/cr-r6-bootstrap-code`) if done
   and hand off per instructions.
