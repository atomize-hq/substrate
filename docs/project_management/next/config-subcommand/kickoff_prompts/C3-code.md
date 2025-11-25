# Task C3-code (Config set command) – CODE

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `C3-code` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start C3-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-c3-set-code
   git worktree add wt/cs-c3-set-code cs-c3-set-code
   cd wt/cs-c3-set-code
   ```

## Spec (shared with C3-test)
- Implement `substrate config set key=value [...]`:
  - Accept one or more `key=value` arguments using dotted keys (e.g.,
    `world.anchor_mode=follow-cwd`, `install.world_enabled=false`,
    `world.caged=true`).
  - Validate values: anchor modes limited to supported enum, booleans for toggles,
    strings/paths for others.
  - Apply all updates atomically (write temp file + rename).
  - Expose `--json` to emit a summary of applied changes.
  - Preserve unknown keys and emit clear errors without mutation on invalid input.
- Docs updated with examples and multi-key guidance.

## Scope & Guardrails
- Production code + docs. Tests belong to C3-test.
- Reuse existing config parsing helpers; consider a generic setter utility.
- Ensure precedence stack unchanged: CLI/env overrides still win at runtime.

## Suggested Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Confirm commands above succeed; log outputs.
2. Commit worktree changes (e.g., `feat: add substrate config set command`).
3. Merge branch into `feat/config-subcommand`.
4. Update `tasks.json` + `session_log.md` (END entry), ensure C3-test prompt is
   referenced, and commit docs (`git commit -am "docs: finish C3-code"`).
5. Remove worktree and hand off.
