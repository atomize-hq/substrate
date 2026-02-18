# Task J1-code (Structured mode scaffold) – CODE

## Start Checklist (feat/json-mode)
1. `git checkout feat/json-mode && git pull --ff-only`
2. Read `json_mode_plan.md`, `tasks.json`, `session_log.md`, and this prompt.
3. Set `J1-code` to `in_progress`, log START entry, commit doc update
   (`git commit -am "docs: start J1-code"`).
4. Create branch/worktree:
   ```
   git checkout -b cs-j1-structured-code
   git worktree add wt/cs-j1-structured-code cs-j1-structured-code
   cd wt/cs-j1-structured-code
   ```

## Spec (shared with J1-test)
- Add root-level structured-mode flags:
  - `--json` (output) toggles JSON envelopes for non-interactive commands.
  - `--json-input <FILE|->` and `--json-payload <STRING>` provide optional input payloads (no behavior yet beyond parsing/validation).
- Define a `StructuredResponse` envelope (`status`, `message`, `data`) and ensure top-level success/errors use it when `--json` is active.
- Interactive REPL rejects `--json`/`--json-input` with a descriptive error; other execution modes pass the structured context through invocation/routing.
- Docs updated with flag descriptions, envelope schema, and REPL limitation.

## Scope & Guardrails
- Production code and docs only. Tests handled by J1-test.
- Keep default behavior unchanged when `--json` isn’t specified.
- Ensure flags behave consistently across subcommands; existing JSON-only flags remain functional (to be aliased later).

## Suggested Commands
```
cargo fmt
cargo clippy -p substrate-shell -- -D warnings
cargo test -p substrate-shell world_root
```

## End Checklist
1. Confirm commands succeed; capture outputs for log.
2. Commit worktree changes (e.g., `feat: add structured mode flags/envelope`).
3. Merge branch into `feat/json-mode` (fast-forward).
4. Update `tasks.json` + `session_log.md` (END entry) and commit docs
   (`git commit -am "docs: finish J1-code"`). Reference the J1-test prompt.
5. Remove worktree and hand off.
