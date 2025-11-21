# Settings Stack Plan

## Context

We need to replace the ad hoc install metadata (`~/.substrate/config.json`) with a
human-friendly TOML file and add a configurable "world root" policy so humans
and agents can explicitly control how far Substrate roams the filesystem while
still defaulting to the secure project-only overlay.

## Goals

1. Migrate install metadata to TOML (`~/.substrate/config.toml`) and keep the
   same semantics for `world_enabled` plus room for new keys.
2. Introduce a settings stack for the world root scope with clear precedence:
   CLI flag → directory config → global config → env vars → built-in default.
3. Provide documentation + kickoff prompts so future agents can scale the work
   using the code/test/integration workflow (with strict guardrails).

## Guardrails & Workflow Expectations

All agents (code, test, integration) **must** follow the same session flow on
`feat/settings-stack`. Treat the steps below as a mandatory runbook.

### Start Checklist (feat/settings-stack)
1. `git checkout feat/settings-stack && git pull --ff-only`
2. Read this plan, `tasks.json`, the latest `session_log.md`, and the kickoff
   prompt for your task.
3. Update `tasks.json` (set your task to `in_progress`) and append a START entry
   to the session log. Commit the doc-only change on `feat/settings-stack`
   (`git commit -am "docs: start <task-id>"`).
4. Create a **dedicated task branch** from `feat/settings-stack`, named for the
   task (e.g., `ss-s1-config-code`):
   ```
   git checkout -b ss-s1-config-code
   ```
5. Create the worktree from that task branch (from repo root):
   ```
   git worktree add wt/ss-s1-config-code ss-s1-config-code
   cd wt/ss-s1-config-code
   ```
   Never edit docs/tasks/session log inside a worktree.

### Active Work (worktree)
- Stay inside the scope defined by your kickoff prompt. Production changes go in
  code tasks, test updates in test tasks.
- Document outputs/commands you will need for the END log entry.
- Commit worktree changes once they meet the acceptance criteria.

### End Checklist
1. Ensure fmt/lint/tests (if required by the kickoff prompt) pass in the
   worktree.
2. Commit your worktree changes with a descriptive message.
3. Return to the task branch in repo root and merge/cherry-pick from the
   worktree (if the worktree is already on the task branch, skip merge).
4. Merge the task branch back into `feat/settings-stack`:
   ```
   git checkout feat/settings-stack
   git pull --ff-only
   git merge --ff-only ss-s1-config-code   # or appropriate task branch
   ```
4. Update `tasks.json` (e.g., set to `completed`), append an END entry to the
   session log (include commands run, test results, blockers), and create the
   next kickoff prompt(s) you are responsible for.
5. Commit those doc updates on `feat/settings-stack`
   (`git commit -am "docs: finish <task-id>"`).
6. Remove the worktree if finished (`git worktree remove wt/...`) and push or
   hand off per instructions.

### Role Responsibilities

| Role        | Allowed work                                                                 | Forbidden work                                 |
|-------------|------------------------------------------------------------------------------|------------------------------------------------|
| Code agent  | Production code, installer scripts, documentation supporting the feature.    | Creating/updating tests; touching harnesses.   |
| Test agent  | Test files, fixtures, harness scripts, kickoff prompts for future tasks.     | Changing production code except tiny test-only helpers. |
| Integration | Merge code/test worktrees, resolve conflicts, run fmt/clippy/tests, update docs/logs. | Adding new features or expanding test coverage. |

Additional rules:
- **Kickoff prompts** must be authored by the agent finishing a session. Code
  agents produce the paired test prompt; test agents produce the integration
  prompt and the next code/test prompts so those tasks can start while
  integration runs.
- Documentation/tasks/session log updates are committed **only** on
  `feat/settings-stack` with descriptive messages.
- Each session’s START/END entries must list commands executed, artifacts
  generated, and references to the kickoff prompts created.

## World Root Settings Stack

We are introducing two new concepts:

- **Global config**: `~/.substrate/config.toml`
- **Directory config**: `<repo>/.substrate/settings.toml`

Precedence (highest wins):

1. CLI flag `--world-root-mode` (and optional `--world-root-path`).
2. Directory config (`.substrate/settings.toml` in the repo/home the shell is
   started from).
3. Global config (`~/.substrate/config.toml`).
4. Environment variables `SUBSTRATE_WORLD_ROOT_MODE` and
   `SUBSTRATE_WORLD_ROOT_PATH`.
5. Built-in default: `project` (locks world to the directory where the shell
   started).

### Modes

| Mode          | Description |
| ------------- | ----------- |
| `project`     | Anchor the world overlay to the initial project root
| `follow-cwd`  | Recalculate root on each `cd` so commands run relative to the
|               | current working directory (within explicit allowlist). |
| `custom`      | Use `--world-root-path` / config value to point at an explicit
|               | directory tree. |

### File Layout

`~/.substrate/config.toml` example:

```toml
[install]
world_enabled = true

[world]
root_mode = "project"
root_path = ""
```

Directory config inherits the same schema but omits `[install]`.

## Task Buckets

We are delivering this in two sequential buckets, each with code/test/integration tasks:

1. **S0 – Manifest Bundling Fix**
   - Update release packaging/installer scripts so `config/manager_hooks.yaml`
     (and related manifests) ship inside every bundle, ensuring `substrate health`
     works immediately after install.
2. **S1 – TOML Install Config**
   - Convert install metadata to TOML (`config.toml`), update installer/CLI,
     and refresh docs.
3. **S2 – Settings Stack & World Root**
   - Implement the layered settings stack, new CLI flag/env vars, config file
     parsing, and documentation (depends on S1 being merged).

S0-code/test run in parallel, followed by S0-integ. After S0-integ merges, S1
code/test may start. Likewise, **do not** start S2 code/test until S1-integ has
merged into `feat/settings-stack`; each stage builds on the previous one.
Integration tasks always depend on the corresponding code + test tasks.

See `tasks.json` for detailed entries, worktree names, and dependencies.
