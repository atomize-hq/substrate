# Kickoff – M5a-test (World deps inventory & layering)

## Role
Test agent: tests/fixtures only. No production code. Do not edit docs/tasks/session logs from the worktree.

## Goal
Add tests/fixtures validating inventory alignment and manifest layering per M5a-spec.

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M5a-spec.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M5a-test` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M5a-test`).
4. Create branch `mp-m5a-world-deps-inventory-test`, then worktree: `git worktree add wt/mp-m5a-world-deps-inventory-test mp-m5a-world-deps-inventory-test`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Only touch tests/fixtures/harnesses.
- Prefer deterministic tests that do not require Lima/agent availability.

## Required commands (record output in END entry)
- `cargo fmt`
- Targeted `cargo test ...` for the suites you add/touch

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M5a-test`).
4. Remove worktree `wt/mp-m5a-world-deps-inventory-test`.

