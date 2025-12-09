# Task C2-code (Config show command) – CODE

## Start Checklist (feat/config-subcommand)
1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Read `config_subcommand_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `C2-code` to `in_progress`, log START entry, and commit doc update
   (`git commit -am "docs: start C2-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-c2-show-code
   git worktree add wt/cs-c2-show-code cs-c2-show-code
   cd wt/cs-c2-show-code
   ```

## Spec (shared with C2-test)
- Implement `substrate config show`:
  - Default output: pretty TOML representation of `~/.substrate/config.toml`.
  - `--json` flag prints machine-readable JSON.
  - Missing file surfaces hint to run `substrate config init`.
  - Redaction hook exists for potential sensitive fields (even if none today).
- Command exits 0 on success, non-zero on errors; no side effects.
- Docs updated (`docs/CONFIGURATION.md`, `docs/USAGE.md`) with human/JSON examples.

## Scope & Guardrails
- Production code + docs only. Tests belong to C2-test.
- Reuse existing parsing logic; avoid reimplementing `resolve_world_root`.
- Keep behavior cross-platform; ensure SUBSTRATE_HOME override respected.

## Suggested Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
```

## End Checklist
1. Confirm fmt/clippy/tests succeed; note outputs.
2. Commit worktree changes (e.g., `feat: add substrate config show command`).
3. Merge branch back to `feat/config-subcommand` (fast-forward).
4. Update `tasks.json` + `session_log.md` (END entry) and commit
   (`git commit -am "docs: finish C2-code"`). Mention doc updates.
5. Remove worktree and hand off.
