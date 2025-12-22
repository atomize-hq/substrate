# Kickoff – M5c-test (First-run UX wiring)

## Role
Test agent: tests/fixtures only. No production code. Do not edit docs/tasks/session logs from the worktree.

## Goal
Add tests/fixtures validating first-run wiring and coherent recommendations across installer/health/doctor per M5c-spec.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M5c-spec.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M5c-test` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M5c-test`).
4. Create branch `mp-m5c-world-deps-first-run-test`, then worktree: `git worktree add wt/mp-m5c-world-deps-first-run-test mp-m5c-world-deps-first-run-test`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Only touch tests/fixtures/harnesses.
- Prefer deterministic tests; avoid requiring Lima.

## Required commands (record output in END entry)
- `cargo fmt`
- Targeted `cargo test ...` for the suites you add/touch

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M5c-test`).
4. Remove worktree `wt/mp-m5c-world-deps-first-run-test`.

