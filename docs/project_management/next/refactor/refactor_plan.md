# Crate Refactor Plan

## Context

This plan operationalizes the findings in
`docs/project_management/analysis/CRATE_REFACTORING_ANALYSIS.md` (standard:
`docs/project_management/standards/rustStandards.md`). We will remove panic
paths in shared libraries, decompose the `shell` god module, and tighten global
state/binary boundaries across the workspace while keeping Substrate usable and
traceable through every handoff.

## Goals

1. Eliminate `.unwrap()` panics in library code (broker, world, telemetry-lib,
   forwarder) and backstop with panic-focused tests.
2. Decompose `crates/shell` into clear modules plus channel-based PTY handling,
   with tests moved out of the monolith.
3. Remove global mutable state and enforce thin-binary patterns (broker,
   trace, world-agent, host-proxy) with doc coverage.
4. Split remaining single-file crates (trace, world-windows-wsl), improve
   replay/common docs, and keep CHANGELOG/tests/benchmarks green.
5. Decompose oversized shell execution modules (execution/mod.rs, pty_exec.rs,
   settings, manager initialization) into focused units without altering CLI
   behavior.
6. Split bootstrap/builtin monoliths (manager_manifest, shim exec, shell
   builtins) into testable modules while preserving outputs and logging.
7. Slim down service modules (host-proxy/lib.rs, world/overlayfs.rs,
   replay/replay.rs) with thin public surfaces and stable APIs.
8. Finish shell execution file slicing (routing, pty/io, invocation, settings,
   platform, manager_init) to keep modules small and maintainable without
   changing behavior.

## Baseline Standards & References

- AGENTS: `AGENTS.md` (repo policies, platform guardrails).
- Rust standards: `docs/project_management/standards/rustStandards.md`.
- Analysis source: `docs/project_management/analysis/CRATE_REFACTORING_ANALYSIS.md`.
- Workspace docs: `docs/TRACE.md`, `docs/REPLAY.md`, `docs/CONFIGURATION.md`,
  `docs/INSTALLATION.md`, `docs/TELEMETRY.md`.
- Build/test workflow: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  targeted `cargo test --workspace -- --nocapture` as prompted.
- Logging/redaction: `crates/common/src/log_schema.rs` + `redact_sensitive`.
- Platform boundaries: keep cfg-gates aligned with `crates/shell/src/platform_world.rs`
  and world backends.

## Guardrails & Workflow Expectations

All agents (code, test, integration) **must** follow the session flow on
`feat/crate-refactor`. Treat the steps below as a mandatory runbook.

### Start Checklist (feat/crate-refactor)

1. `git checkout feat/crate-refactor && git pull --ff-only`
2. Read this plan, `tasks.json`, the latest `session_log.md`, the kickoff prompt
   for your task, and `CRATE_REFACTORING_ANALYSIS.md`.
3. Update `tasks.json` (set your task to `in_progress`) and append a START entry
   to the session log. Commit the doc-only change on `feat/crate-refactor`
   (`git commit -am "docs: start <task-id>"`).
4. Create a **dedicated task branch** from `feat/crate-refactor`, named for the
   task (e.g., `cr-r1-panics-code`):
   ```
   git checkout -b <task-branch>
   git worktree add wt/<task-branch> <task-branch>
   cd wt/<task-branch>
   ```
   Never edit docs/tasks/session log inside a worktree.

### Active Work (worktree)

- Stay inside the scope defined by your kickoff prompt. Production changes go in
  code tasks, test updates in test tasks.
- Document outputs/commands you will need for the END log entry.
- Commit worktree changes once they meet the acceptance criteria.

### End Checklist

1. Ensure fmt/lint/tests (per kickoff prompt) pass in the worktree.
2. Commit your worktree changes with a descriptive message.
3. Return to the task branch in repo root and merge/cherry-pick from the
   worktree (if the worktree is already on the task branch, skip merge).
4. Merge the task branch back into `feat/crate-refactor`:
   ```
   git checkout feat/crate-refactor
   git pull --ff-only
   git merge --ff-only <task-branch>
   ```
5. Update `tasks.json` (e.g., set to `completed`), append an END entry to the
   session log (include commands run, test results, blockers), and create the
   next kickoff prompt(s) per role:
   - Code agent: create the paired test prompt for the same spec.
   - Test agent: create the integration prompt for this task.
   - Integration agent: create the next code/test prompts after merging.
6. Commit those doc updates on `feat/crate-refactor`
   (`git commit -am "docs: finish <task-id>"`).
7. Remove the worktree if finished (`git worktree remove wt/...`) and push or
   hand off per instructions.

### Role Responsibilities

| Role        | Allowed work                                                                          | Forbidden work                                          |
| ----------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Code agent  | Production code, installer/binary plumbing, documentation supporting the refactor.    | Creating/updating tests; touching harnesses.            |
| Test agent  | Test files, fixtures, harness scripts, kickoff prompts for future tasks.              | Changing production code except tiny test-only helpers. |
| Integration | Merge code/test worktrees, resolve conflicts, run fmt/clippy/tests, update docs/logs. | Adding new features or expanding test coverage.         |

Additional rules:

- **Kickoff prompts** must be authored by the agent finishing a session:
  - Code agent: produce the paired test prompt for the same spec.
  - Test agent: produce the integration prompt for the task you just finished.
  - Integration agent: produce the next code/test prompts after merging so
    subsequent tasks can start on the merged baseline.
- Documentation/tasks/session log updates are committed **only** on
  `feat/crate-refactor` with descriptive messages.
- Each session’s START/END entries must list commands executed, artifacts
  generated, and references to the kickoff prompts created.
- Preserve API compatibility wherever possible; document any intentional
  breakage and update `CHANGELOG.md`.
- Code and test tasks run concurrently: the **same spec must be mirrored in both
  kickoff prompts**, code agents must **not** add tests, and test agents author
  tests from the spec without relying on visibility into code branches.
- Integration agents merge both code and test branches into their integration
  branch/worktree, resolve any misalignments, and ensure the combined result
  meets the intended spec before updating docs/tasks/logs on
  `feat/crate-refactor`.

## Refactor Tracks

1. **R1 – Critical Panic Remediation**  
   Remove `.unwrap()` panics from `broker`, `world`, `telemetry-lib`,
   `forwarder`; add panic-focused tests and ensure error contexts are rich.
2. **R2 – Shell Decomposition & Concurrency Fix**  
   Break `crates/shell/src/lib.rs` into execution/repl/builtins/scripts modules,
   extract tests to `tests/`, and replace nested `Arc<Mutex>` PTY handling with
   channel-based orchestration.
3. **R3 – State & Binary Boundaries**  
   Eliminate global singletons (broker, trace), enforce thin binary patterns
   (world-agent, host-proxy), and document lifecycle/config impacts.
4. **R4 – Polish & Documentation**  
   Split remaining single-file crates (trace, world-windows-wsl), add replay and
   common prelude docs, refresh examples, and keep benches/tests stable.
5. **R5 – Shell Execution Decomposition**  
   Break `crates/shell/src/execution/mod.rs` (~7.5k lines) into routing,
   invocation planning, and platform adapters; split `pty_exec.rs`, settings,
   and manager initialization into focused modules with unchanged CLI behavior.
6. **R6 – Bootstrap & Builtins Decomposition**  
   Split `common/manager_manifest.rs`, `shim/src/exec.rs`, and the large shell
   builtins (`shim_doctor`, `world_enable`, `world_deps`) into schema/resolver/
   validator modules and per-command helpers with preserved outputs.
7. **R7 – Service Module Slimming**  
   Reduce `host-proxy/src/lib.rs`, `world/src/overlayfs.rs`, and
   `replay/src/replay.rs` into thin public surfaces plus internal modules; keep
   performance and APIs stable.
8. **R8 – Shell Execution File Slicing**  
   Split remaining large shell execution files (`pty/io.rs`, `invocation.rs`,
   `settings.rs`, `platform.rs`, `manager_init.rs`) into focused modules with
   preserved CLI/PTY semantics and platform gates.

Task details, dependencies, and worktree names live in `tasks.json`.

## Outstanding Hotspots (post-R7)

- `crates/shell/src/execution/routing.rs` ~5,287 LOC
- `crates/shell/src/execution/pty/io.rs` ~1,328 LOC
- `crates/shell/src/execution/invocation.rs` ~1,080 LOC
- `crates/shell/src/execution/settings.rs` ~763 LOC
- `crates/shell/src/execution/platform.rs` ~721 LOC
- `crates/shell/src/execution/manager_init.rs` ~668 LOC
- (Large fixtures accepted: `shim/tests/integration.rs` ~959 LOC,
  `shell/tests/integration.rs` ~745 LOC)
