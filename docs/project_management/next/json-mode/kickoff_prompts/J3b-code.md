# Task J3b-code (JSON input – complex workflows) – CODE

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J3b-code` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J3b-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j3b-input-code
   git worktree add wt/cs-j3b-input-code cs-j3b-input-code
   cd wt/cs-j3b-input-code
   ```

## Spec (shared with J3b-test)
- Extend JSON payload support to complex commands:
  - `substrate world deps install/sync` (multiple tools/states).
  - Installer/helper commands (e.g., world provisioning) as needed.
  - Config commands introduced by the config-subcommand plan.
- CLI flags must override payload values deterministically; document precedence.
- Validators ensure invalid payloads produce structured errors without mutations.
- Docs updated with payload schemas/examples.

## Scope & Guardrails
- Production code/docs only. Reuse plumbing from J3a.
- Keep CLI-only workflows fully supported.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Ensure commands/tests pass; capture outputs.
2. Commit worktree changes (e.g., `feat: add json payloads for world deps/installers`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs
   (`git commit -am "docs: finish J3b-code"`). Reference J3b-test prompt.
5. Remove worktree and hand off.
