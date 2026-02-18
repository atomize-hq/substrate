# JSON Mode Plan

## Context

Upcoming UI work embeds Substrate as the execution backend for a Tauri/React,
block-based “terminal.” That frontend needs structured request/response semantics:
commands should accept JSON payloads (when present) and emit JSON envelopes with
consistent status fields. Today only a handful of reporting commands provide a
`--json` toggle, and none accept JSON input. This plan introduces a workspace-wide
“structured mode” so every CLI command (except the interactive REPL) can participate
in machine-controlled sessions.

## Goals

1. **Global structured mode** – Add root-level flags (`--json` for output,
   `--json-input <FILE|->` or `--json-payload <STRING>`) recognized by every
   command/subcommand except the interactive REPL, plus a shared JSON envelope
   (status/message/data) for responses.
2. **Command coverage** – Extend all existing commands (graph, shim, world,
   health, installer helpers, trace/replay, etc.) so they honor structured output,
   including error paths, and document schemas **for macOS/Linux and Windows**
   (PowerShell/cmd). To keep scope manageable, this work is split into waves:
   core/global commands (J2a), world/shim flows (J2b), and graph/installer/advanced
   commands (J2c).
3. **JSON input plumbing** – Allow commands that accept complex arguments to
   consume structured payloads (e.g., sequences of actions, multi-tool installs)
   while preserving traditional CLI options. This is delivered in two phases:
   foundational plumbing/simple commands (J3a) and complex workflows (J3b),
   with explicit consideration for Windows stdin/encoding (PowerShell CRLF/BOM).

## Standards & References

- Repository guardrails: `AGENTS.md`.
- CLI definitions: `crates/shell/src/execution/cli.rs`,
  `crates/shell/src/execution/invocation.rs`, `crates/shell/src/execution/routing.rs`,
  and subcommand handlers in `crates/shell/src/builtins`.
- Documentation: `docs/USAGE.md`, `docs/CONFIGURATION.md`, `docs/TRACE.md`
  with cross-platform examples (POSIX shells + PowerShell/cmd).
- Existing JSON-producing code paths for reference (world doctor, shim doctor,
  health).
- Tooling expectations: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  targeted `cargo test` suites, installer smoke scripts.

## Guardrails & Workflow

Mirror the execution rigor used in `refactor/` and `config-subcommand/`:

- Work happens on branch `feat/json-mode`.
- Doc/task/session log updates happen on the branch root; production/test edits
  live in dedicated task branches + worktrees.
- Code vs test roles share identical specs; integration role merges them.
- Start/end checklists, required commands, and session log format follow the same
  template as other project-management tracks.

### Start Checklist (feat/json-mode)

1. `git checkout feat/json-mode && git pull --ff-only`
2. Read this plan, `tasks.json`, the latest `session_log.md`, and your kickoff prompt.
3. Update `tasks.json` (set task → `in_progress`) and append a START entry;
   commit doc-only change (`git commit -am "docs: start <task-id>"`).
4. Create the task branch and worktree:
   ```
   git checkout -b <task-branch>
   git worktree add wt/<task-branch> <task-branch>
   cd wt/<task-branch>
   ```

### End Checklist

1. Run required fmt/lint/tests per prompt and capture outputs.
2. Commit worktree changes with descriptive messages.
3. Merge branch into `feat/json-mode` (fast-forward).
4. Update `tasks.json` (mark `completed`), append END entry with commands/results,
   create downstream kickoff prompts, and commit docs (`git commit -am "docs: finish <task-id>"`).
5. Remove worktree and hand off per workflow.

### Role Responsibilities

| Role | Allowed work | Forbidden work |
| --- | --- | --- |
| Code | Production code, CLI plumbing, doc updates tied to implementation. | Editing tests beyond trivial helpers. |
| Test | Tests/fixtures/scripts, kickoff prompts for integration. | Touching production code (except helpers). |
| Integration | Merge code/test branches, resolve conflicts, run fmt/clippy/tests, update docs/logs. | Adding new features/tests beyond reconciliation. |

Kickoff prompts must echo the same spec for paired code/test tasks and spell out
commands + guardrails. Integration prompts focus solely on merging/validation.

## Tracks

1. **J1 – Structured mode scaffolding**
   - Add global `--json` and `--json-input/--json-payload` flags.
   - Implement JSON envelope type shared across commands (consistent keys across platforms; Windows doctor stubs must match Linux/mac fields or mark them null).
   - Update root execution/invocation to respect structured mode, except the
     interactive REPL (still text-only).
2. **J2 – Command coverage (multi-phase)**
   - **J2a**: Core/root commands (version, shim status, trace/replay, health, global errors) with identical envelope schemas on macOS/Linux and Windows (PowerShell/cmd).
   - **J2b**: World/shim flows (world doctor/enable/deps, shim doctor/repair, health aggregator) ensuring Linux/macOS/Windows outputs share schema.
   - **J2c**: Graph CLI, world-deps install/sync, shim repair/install helpers, future config commands, including `.ps1` installer flows.
   - Each phase documents schemas in `docs/USAGE.md` with matching fixtures/tests.
3. **J3 – JSON input handlers (multi-phase)**
   - **J3a**: Implement JSON input loader/merging and adopt it in simple payload
     consumers (graph ingest, shim repair, other single-action commands), with guidance/tests for PowerShell stdin feeds and CRLF handling.
   - **J3b**: Extend JSON input to complex workflows (world deps install/sync,
     installer/config commands) with full validation/backward compatibility, plus Windows installer smoke coverage.

Each phase (J1, J2a–J2c, J3a–J3b) contains code/test/integration tasks described in `tasks.json`.
