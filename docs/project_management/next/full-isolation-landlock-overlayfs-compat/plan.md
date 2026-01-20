# full-isolation-landlock-overlayfs-compat — plan

## Scope
- Feature directory: `docs/project_management/next/full-isolation-landlock-overlayfs-compat/`
- Orchestration branch: `feat/full-isolation-landlock-overlayfs-compat`
- ADR: `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

## Goal
- Restore `world_fs.write_allowlist` correctness in `world_fs.isolation=full` + `world_fs.mode=writable` on Linux when Landlock is supported and overlayfs is the active filesystem strategy.

## Non-Goals
- Any policy snapshot schema changes or policy snapshot hash behavior changes.
- Any new user-facing allowlist syntaxes or matching semantics.
- Any behavior changes for macOS or Windows world backends.

## Platform scope
- Behavior platforms (smoke required): Linux, macOS.
- CI parity platforms (compile parity required): Linux, macOS, Windows.
- WSL coverage: not required.

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Execution triads do not begin unless:
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/execution_preflight_report.md` contains `RECOMMENDATION: ACCEPT`.
  - `docs/project_management/next/full-isolation-landlock-overlayfs-compat/quality_gate_report.md` exists and contains `RECOMMENDATION: ACCEPT`.

## Triads
- C0: Derive overlay backing dirs from procfs mountinfo and extend the full-isolation Landlock write allowlist at runtime; add fixture tests; validate via Linux smoke + CI parity.
