# Task J2c-code (Structured output – graph & advanced) – CODE

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J2c-code` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J2c-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j2c-coverage-code
   git worktree add wt/cs-j2c-coverage-code cs-j2c-coverage-code
   cd wt/cs-j2c-coverage-code
   ```

## Spec (shared with J2c-test)
- Commands in scope: `substrate graph ingest/status/what-changed`, `world deps install/sync`,
  `shim repair` (structured output mode), installer/helper commands, and any emerging config commands.
- For each, ensure success + failure responses emit envelopes with schema-aligned payloads (files, limits, manifest results, repair hints).
- Document payload schemas/examples in `docs/USAGE.md`.

## Scope & Guardrails
- Production code/docs only. Leave JSON input adoption for J3 phases.
- Maintain text output.
- Coordinate with config-subcommand work so new commands can hook into envelopes.

## Suggested Commands
```
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p substrate-shell world_root
```

## End Checklist
1. Ensure commands/tests pass; capture outputs.
2. Commit worktree changes (e.g., `feat: add structured output for graph/advanced commands`).
3. Merge branch into `feat/json-mode`.
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs
   (`git commit -am "docs: finish J2c-code"`). Reference J2c-test prompt.
5. Remove worktree and hand off.
