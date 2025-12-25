# P0 Agent Hub Isolation Hardening Plan

## Context

Substrate is becoming an agent hub: multiple frontier-model CLIs (and humans) execute through Substrate as
the centralized policy and isolation boundary. That requires:
- Security guarantees that are **actually enforced** (no “false sense of security”).
- **Fail-closed** behavior when a policy requires isolation and the host cannot provide it.
- A clear, strict, versioned policy schema for filesystem restrictions and “caging” mode.

This track is focused on Linux isolation semantics (with macOS Lima/Windows WSL parity in mind) and is
recorded in the ADR:
- `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`

## Guardrails

- Orchestration branch: `feat/p0-agent-hub-isolation-hardening`
- All docs/tasks/session log edits happen **only** on the orchestration branch (never in worktrees).
- Each slice ships as a triad: **code**, **test**, **integration**.
- Specs are the single source of truth; integration reconciles code/tests to the spec.
- Follow `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`.

### Role boundaries

- Code agent: production code + implementation-tied docs only. No tests.
  - Required: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`
- Test agent: tests/fixtures/harness only (plus minimal test-only helpers). No production code.
  - Required: `cargo fmt`; targeted `cargo test ...` for touched tests
- Integration agent: merge code/test, reconcile to spec, run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant `cargo test ...`
  - `make preflight` (required)

## Triads Overview

Triads are numbered `I0..I5` (Isolation track). Each triad has a `*-spec.md` and code/test/integ tasks.

1) **I0 — Strict policy schema (world_fs v1)**
2) **I1 — Fail-closed semantics (no host fallback when required)**
3) **I2 — Full cage (non-PTY): mount namespace + pivot_root**
4) **I3 — Full cage (PTY): parity with non-PTY**
5) **I4 — Landlock (additive hardening)**
6) **I5 — Docs alignment + verification tooling**

## Exit codes

Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

This track uses the canonical exit code meanings. Each `I*-spec.md` and the manual playbook define the exact
per-command mappings and messages.

## Start Checklist (all tasks)

1. `git checkout feat/p0-agent-hub-isolation-hardening && git pull --ff-only`
2. Read `plan.md`, `tasks.json`, `session_log.md`, your `I*-spec.md`, and your kickoff prompt.
3. Set task status → `in_progress` in `tasks.json`.
4. Add START entry to `session_log.md`; commit docs (`docs: start <task-id>`).
5. Create task branch and worktree: `git worktree add wt/<worktree> <branch>`.
6. Do not edit docs/tasks/logs in worktrees.

## End Checklist (code/test)

1. Run required commands (code: fmt/clippy; test: fmt + targeted tests). Capture outputs.
2. Commit worktree changes to the task branch.
3. Switch back to orchestration branch; update `tasks.json` + add END entry to `session_log.md`; commit docs
   (`docs: finish <task-id>`).
4. Remove worktree.

## End Checklist (integration)

1. Merge code+test branches into the integration worktree; reconcile to the spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`.
3. Commit integration changes; fast-forward into orchestration branch.
4. Update `tasks.json` + `session_log.md`; commit docs; remove worktree.
