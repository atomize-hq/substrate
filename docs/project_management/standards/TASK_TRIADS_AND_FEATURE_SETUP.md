# Task Triads & Feature Setup Standard

This document explains, step by step, how to create a new feature directory, define triads (code/test/integration), and produce all required files with zero ambiguity.

## Principles
- Every slice of work ships as a triad: code, test, integration.
- Code agent: production code only. No tests. Runs `cargo fmt` and `cargo clippy --workspace --all-targets -- -D warnings`. Optional targeted/manual sanity checks are allowed but not required. No unit/integration suite requirement.
- Test agent: tests only (plus minimal test-only helpers if absolutely needed). No production code. Runs `cargo fmt` and the targeted tests they add/touch; not responsible for full suite.
- Integration agent: merges code+tests, resolves drift to the spec, ensures behavior matches the spec, runs `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, all relevant tests, and finishes with `make preflight` (required). They own the final green state.
- Docs/tasks/session log edits happen **only** on the orchestration branch (never in worktrees).
- Specs are the single source of truth; integration reconciles code/tests to the spec.

## Creating a New Feature Directory (from scratch)
1. Choose orchestration branch name (e.g., `feat/<feature>`). Create/pull it.
2. Create directory: `docs/project_management/next/<feature>/`.
3. Add files:
   - `plan.md` (runbook/guardrails/triad overview).
   - `tasks.json` (all tasks with ids, worktrees, deps, prompts).
   - `session_log.md` (START/END entries only).
   - Specs: `C0-spec.md`, `C1-spec.md`, ... (one per triad).
   - `kickoff_prompts/` directory with `<triad>-code.md`, `<triad>-test.md`, `<triad>-integ.md`.
   - Optional user-facing drafts (e.g., `DRAFT_*.md`).
4. Update `plan.md` triad overview to list all triads.
5. Commit the scaffolding on the orchestration branch.

## Required Content (no ambiguity)
### plan.md
- Principles/guardrails, start/end checklists, triad overview, role rules.

### tasks.json (fields)
- Required fields per task:
  - `id`, `name`, `type` (code/test/integration), `phase`, `status`, `description`
  - `references` (array of files/docs to read)
  - `acceptance_criteria` (array of concrete outcomes)
  - `start_checklist` (array of steps)
  - `end_checklist` (array of steps)
  - `worktree`, `integration_task`, `kickoff_prompt`
  - `depends_on` (list), `concurrent_with` (list)
- Example entry:
```json
{
  "id": "C2-code",
  "name": "Manual sync (non-PTY)",
  "type": "code",
  "phase": "World Sync",
  "status": "pending",
  "description": "Implement manual world→host sync per C2-spec.",
  "references": ["docs/project_management/next/world-sync/C2-spec.md"],
  "acceptance_criteria": [
    "Sync applies world→host per conflict policy and filters",
    "Protected paths are skipped",
    "Size guard enforced"
  ],
  "start_checklist": [
    "Checkout feat/world-sync, pull ff-only",
    "Set status to in_progress, log START, commit docs",
    "Create branch ws-c2-sync-code and worktree wt/ws-c2-sync-code"
  ],
  "end_checklist": [
    "Run fmt/clippy",
    "Commit worktree changes",
    "Merge back ff-only to feat/world-sync",
    "Update tasks/session log, commit docs, remove worktree"
  ],
  "worktree": "wt/ws-c2-sync-code",
  "integration_task": "C2-integ",
  "kickoff_prompt": "docs/project_management/next/world-sync/kickoff_prompts/C2-code.md",
  "depends_on": ["C1-integ"],
  "concurrent_with": ["C2-test"]
}
```

### Specs (`C*-spec.md`)
Must include:
- Scope (explicit behaviors, defaults, error handling, platform guards, protected paths).
- Acceptance (observable outcomes).
- Out of scope (to avoid scope creep).

### Kickoff prompts
Each prompt must include:
- Scope and explicit role boundaries (“prod code only, no tests” for code; “tests only” for test; integration owns aligning to spec).
- Start checklist (always): checkout orchestration branch, pull ff-only, read plan/tasks/session_log/spec/prompt, set task status to `in_progress` in tasks.json, add START entry to session_log.md, commit docs (`docs: start <task-id>`), create task branch and worktree, no docs/tasks/log edits in worktree.
- Requirements: what to build/test, protected paths/safety, required commands (code: fmt/clippy only; test: fmt + targeted tests; integration: fmt/clippy/tests + `make preflight`), sanity-check expectations.
- End checklist: run required commands; commit worktree; merge back to orchestration branch (ff-only); update tasks.json status; add END entry (commands/results/blockers); create downstream prompts if missing (mandatory when absent); commit docs (`docs: finish <task-id>`); remove worktree.

## Branch/Worktree Naming
- Branch: `<feature-prefix>-<triad>-<short-scope>` (e.g., `ws-c3-autosync-code` for world-sync, `ss-s2-settings-code` for settings-stack style). Use a consistent prefix per feature.
- Worktree: `wt/<branch>` or `wt/<feature-prefix>-<triad>-<short-scope>`.
- Integration worktrees may be created from a dedicated integration branch or directly from the orchestration branch; ensure tasks.json/kickoff prompts specify the expected workflow.

## Start/End Checklists (copy/paste)
Start (all tasks):
1. `git checkout <orchestration-branch> && git pull --ff-only`
2. Read plan/tasks/session_log/spec/prompt.
3. Set task status to `in_progress` in tasks.json.
4. Add START entry to session_log.md; commit docs (`docs: start <task-id>`).
5. Create task branch; create worktree: `git worktree add wt/<worktree> <branch>`.
6. Do not edit docs/tasks/logs inside worktree.

End (code/test):
1. Run required commands (code: fmt/clippy; test: fmt + targeted tests). Capture outputs.
2. Commit worktree changes to the task branch (from inside the worktree). Do **not** touch the orchestration branch.
3. From the task branch (outside the worktree), fast-forward/merge the worktree commit into the task branch if needed.
4. Switch to the orchestration branch; update tasks.json status and add the END entry (commands/results/blockers) to session_log.md; create downstream prompts if missing; commit docs (`docs: finish <task-id>`).
5. Remove worktree: `git worktree remove wt/<worktree>`. Leave the task branch intact for integration.

End (integration):
1. Merge code+test task branches into the integration worktree; resolve drift to spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`. Capture outputs.
3. Commit integration changes to the integration branch.
4. Fast-forward merge the integration branch into the orchestration branch; update tasks.json/session_log.md with END entry; commit docs (`docs: finish <task-id>`).
5. Remove worktree.

## Role Command Requirements
- Code: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`; optional targeted/manual sanity checks allowed but not required; no unit/integration suite requirement.
- Test: `cargo fmt`; targeted `cargo test ...` for tests added/modified; no production code; no responsibility for full suite.
- Integration: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`; run relevant tests (at least new/affected suites) and finish with `make preflight` (required full-suite gate). Integration must reconcile code/tests to the spec.

## Context Budget & Triad Sizing
- Agents typically have a 272k token context window. Size each task so a single agent needs no more than ~40–50% of that window (roughly 110–150k tokens) to hold the spec, plan, code/tests, and recent history.
- If a task risks breaching that budget (large migration, many platforms, or broad refactors), split into additional triads or narrower phases before kickoff.
- Use specs to keep scope crisp; avoid “grab bag” triads. Aim for small, testable chunks with clear acceptance criteria.

## Protected Paths
If relevant to the feature (e.g., sync/FS operations), explicitly list in specs/prompts: `.git`, `.substrate-git`, `.substrate`, sockets, device nodes, and any feature-specific exclusions.

## Typical Triad Ordering (example: world-sync)
- C0: Init & gating
- C1: Config/CLI surface
- C2: Manual path A
- C3: Auto path A
- C4: Additional path (e.g., PTY)
- C5: Opposite direction
- C6: Internal system (host)
- C7: Rollback/CLI
- C8: Internal system (world/bridge)
- C9: UX/migration polish
Adjust counts to keep each triad ≤ ~40–50% of a 272k context window (~110–140k tokens).

## Session Log Usage
- Only START/END entries. Include: timestamp (UTC), agent role, task ID, commands run (fmt/clippy/tests/scripts), results (pass/fail, temp roots if applicable), worktree/commits touched, prompts created/verified, blockers/next steps.
- Use a consistent template (can copy the settings-stack template) and do not edit from worktrees.

## Adding New Triads (step-by-step)
1. Create spec file (`C*-spec.md`) with scope/acceptance/out-of-scope.
2. Add tasks (code/test/integ) to tasks.json with worktrees/branches/deps/prompts.
3. Create kickoff prompts for code/test/integ.
4. Update plan.md triad overview.
5. Commit docs on orchestration branch.
