# world-overlayfs-enumeration — plan

## Scope
- Feature directory: `docs/project_management/next/world-overlayfs-enumeration/`
- Orchestration branch: `feat/world-overlayfs-enumeration`
- ADR: `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`

## Goal
- Ensure Linux world overlay mounts support correct directory enumeration and a deterministic fallback strategy for overlayfs health failures.

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Task worktrees are retained until `FZ-feature-cleanup`.

## Execution gates
- `F0-exec-preflight` is completed before starting `WO0-code` / `WO0-test`.
- `WO0-closeout_report.md` is completed as part of `WO0-integ`.

## Triads
- WO0: overlayfs enumeration reliability (code/test/integ)

## Smoke
- Linux: `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world-overlayfs-enumeration/smoke/macos-smoke.sh` (skip; ADR scope is Linux-only)
- Windows: `docs/project_management/next/world-overlayfs-enumeration/smoke/windows-smoke.ps1` (skip; ADR scope is Linux-only)
