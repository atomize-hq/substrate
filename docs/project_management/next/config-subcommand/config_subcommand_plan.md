# Config Subcommand Plan

## Context

The backlog’s top priority requests a first-class configuration UX: CLI commands
to scaffold, inspect, and edit `~/.substrate/config.toml` while retaining the
existing precedence stack (flags → directory config → global config → env).
Settings Stack work delivered the parsing/resolution logic, but today users must
edit TOML manually and the CLI only hints that the file is missing. This plan
builds the dedicated `substrate config` subcommand family so operators can:

- Run `substrate config init` to bootstrap or regenerate the config directory.
- Review defaults through `substrate config show` (humans or automation via
  `--json`).
- Update any key dynamically with `substrate config set key=value [...]`.

The workflow mirrors `docs/project_management/next/refactor` to keep agents in
lock-step on `feat/config-subcommand`.

## Goals

1. **CLI foundation** – Introduce a `config` subcommand group (`init`, `show`,
   `set`) wired into Clap, invoke-able before the shell/REPL starts, and guarded
   by the same platform policies as existing commands.
2. **Bootstrap & diagnostics** – `substrate config init` creates
   `~/.substrate/config.toml` (and parent directories) with default `[install]`
   and `[world]` tables, supports `--force` regeneration, and updates shell and
   installer error paths to mention the command when the file is missing.
3. **Readable output** – `substrate config show` prints the resolved global
   config (TOML by default, JSON with `--json`), redacts sensitive fields, and
   exits non-zero when the file is absent unless `init` is run.
4. **Dynamic setters** – `substrate config set key=value ...` accepts one or
   more dotted keys (e.g., `world.anchor_mode=follow-cwd`,
   `install.world_enabled=false`), validates values against supported schemas
   (mode enums, booleans, paths), preserves unknown keys, and writes atomically.
5. **Docs & automation** – `docs/CONFIGURATION.md` and `docs/USAGE.md` showcase
   the CLI flow on macOS/Linux **and** Windows (explicit `~/.substrate` vs
   `%USERPROFILE%\.substrate` paths, PowerShell usage). Integration tests cover
   `init`/`show`/`set` using `SUBSTRATE_HOME`/`USERPROFILE` overrides so both
   platforms are validated, with JSON output and multi-key edits documented.
   Installer scripts (`install-substrate.sh` and `.ps1`) include matching hints.

## Baseline Standards & References

- Repository guardrails: `AGENTS.md`.
- Backlog source: `docs/BACKLOG.md` (Global configuration UX section).
- Settings stack context: `docs/project_management/_archived/settings-stack/`.
- CLI implementation sites:
  - `crates/shell/src/execution/cli.rs`
  - `crates/shell/src/execution/invocation.rs`
  - `crates/shell/src/execution/settings.rs`
  - `crates/shell/tests`
- Installer scripts (`scripts/substrate/install-substrate.sh`,
  `scripts/substrate/uninstall.sh`) for config scaffolding references.
- Documentation touchpoints: `docs/CONFIGURATION.md`, `docs/USAGE.md`,
  `docs/INSTALLATION.md`.
- Tooling expectations: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  targeted `cargo test` suites and integration scripts per kickoff prompts.

## Guardrails & Workflow Expectations

Every agent works on `feat/config-subcommand`. Reuse the refactor program’s
discipline: doc-only commits on the main branch, production/test work in
dedicated branches + worktrees, and mirrored specs between code and test roles.

### Start Checklist (feat/config-subcommand)

1. `git checkout feat/config-subcommand && git pull --ff-only`
2. Read this plan, `tasks.json`, the latest `session_log.md`, and the kickoff
   prompt for your task.
3. Update `tasks.json` (set your task to `in_progress`) and append a START entry
   to the session log. Commit the doc-only change on `feat/config-subcommand`
   (`git commit -am "docs: start <task-id>"`).
4. Create a task-specific branch and worktree (never edit production/tests from
   the root checkout):
   ```
   git checkout -b <task-branch>
   git worktree add wt/<task-branch> <task-branch>
   cd wt/<task-branch>
   ```

### Active Work (worktree)

- Stay within your prompt’s scope. Code agents avoid test files; test agents
  avoid production code except permitted helpers; integration agents merge only.
- Capture commands/results you’ll document in the END log entry.
- Keep worktree commits focused and reference acceptance criteria.

### End Checklist

1. Ensure fmt/lint/tests per prompt have passed in the worktree.
2. Commit worktree changes with descriptive messages.
3. Merge/cherry-pick onto the task branch (if needed), then fast-forward merge
   into `feat/config-subcommand`.
4. Update `tasks.json` (mark `completed`), append an END session entry with
   commands/results, and author required kickoff prompts for the next role.
5. Commit the doc updates on `feat/config-subcommand`
   (`git commit -am "docs: finish <task-id>"`).
6. Remove finished worktrees (`git worktree remove wt/<task-branch>`) and push
   or hand off as instructed.

### Role Responsibilities

| Role        | Allowed work                                                                  | Forbidden work                               |
| ----------- | ----------------------------------------------------------------------------- | -------------------------------------------- |
| Code agent  | Production code, CLI plumbing, installer/documentation changes referenced.    | Editing tests beyond minimal helper stubs.   |
| Test agent  | Unit/integration tests, fixtures, harness scripts, kickoff prompts.           | Touching production logic.                   |
| Integration | Merge code/test branches, resolve conflicts, run fmt/clippy/tests, update docs/logs/tasks. | Adding features or net-new tests. |

Kickoff prompts must:
- Mirror the same specification between code and test tasks.
- Declare required commands/tests/scripts, guardrails, and success criteria.
- For integration tasks, restate that the job is to merge code/test branches,
  resolve differences, and ensure combined behavior matches the spec before
  updating docs/tasks/logs.

## Configuration CLI Tracks

1. **C1 – CLI foundation & init flow**  
   Add the `config` subcommand skeleton, implement `config init` with `--force`,
   and update shell diagnostics/installers to prompt users when `config.toml`
   is missing. Acceptance includes verifying macOS/Linux and Windows messages
   (PowerShell/cmd).
2. **C2 – Show command & serialization**  
   Implement `config show` with human-readable output and `--json`, integrate
   redaction safeguards, and document usage for both POSIX shells and PowerShell
   (quoting guidance, path formatting). Tests cover `SUBSTRATE_HOME`/`USERPROFILE`
   overrides.
3. **C3 – Dynamic setters & validation**  
   Deliver `config set key=value ...` with multi-key support, schema-aware
   validation, atomic writes, JSON-capable outputs, and explicit handling for
   Windows (CRLF writes, case-insensitive drives). Docs include Windows examples
   and PowerShell quoting advice.

Each track has paired code/test tasks plus an integration task to merge them.
See `tasks.json` for dependencies, acceptance criteria, and worktree names; use
the kickoff prompts in `kickoff_prompts/` for per-task execution details.
