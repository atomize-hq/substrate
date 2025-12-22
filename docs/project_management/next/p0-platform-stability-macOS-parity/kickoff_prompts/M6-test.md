# Kickoff – M6-test (World deps failure safety)

## Role
Test agent: tests/fixtures only. No production code. Do not edit docs/tasks/session logs from the worktree.

## Goal
Add tests/fixtures validating that macOS world deps install/sync fail when the backend is unavailable (and do not claim success via host fallback), per M6-spec.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M6-spec.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M6-test` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M6-test`).
4. Create branch `mp-m6-world-deps-safety-test`, then worktree: `git worktree add wt/mp-m6-world-deps-safety-test mp-m6-world-deps-safety-test`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Only touch tests/fixtures/harnesses.
- Prefer deterministic tests that simulate backend unavailability without requiring Lima.

## Required commands (record output in END entry)
- `cargo fmt`
- Targeted `cargo test ...` for the suites you add/touch

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M6-test`).
4. Remove worktree `wt/mp-m6-world-deps-safety-test`.

