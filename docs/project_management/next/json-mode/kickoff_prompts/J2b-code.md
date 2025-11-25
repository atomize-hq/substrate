# Task J2b-code (Structured output – world & shim) – CODE

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J2b-code` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2b-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j2b-coverage-code
   git worktree add wt/cs-j2b-coverage-code cs-j2b-coverage-code
   cd wt/cs-j2b-coverage-code
   ```

## Spec (shared with J2b-test)
- Commands in scope: `substrate world doctor/enable/deps status|install|sync`,
  `substrate shim doctor`, `substrate shim repair`, and `substrate health`.
- When `--json` is active:
  - All success outputs use envelopes with detailed payloads (manifest paths,
    doctor probes, repair hints, etc.).
  - Errors/ineligibility (world disabled, missing manifest, denied repair) emit envelopes.
- Docs updated with schemas for each command and platform-specific notes.

## Scope & Guardrails
- Production code/docs only. Avoid modifying graph or installer commands (J2c).
- Maintain existing text output.
- Keep platform differences (Linux/macOS/Windows) reflected in JSON payloads.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
cargo test -p substrate-shell world_enable
```

## End Checklist
1. Confirm commands/tests pass; capture outputs.
2. Commit worktree changes (e.g., `feat: add structured output for world/shim commands`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs
   (`git commit -am "docs: finish J2b-code"`). Reference J2b-test prompt.
5. Remove worktree and hand off.
