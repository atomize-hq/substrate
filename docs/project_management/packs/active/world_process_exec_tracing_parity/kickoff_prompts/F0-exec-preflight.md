# Kickoff: F0-exec-preflight (ops)

## Scope

- Validate Planning Pack mechanical invariants before starting WPEP0 execution triads.
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist

Do not edit planning docs inside the worktree.

1. Read: `docs/project_management/packs/active/world_process_exec_tracing_parity/plan.md`, `docs/project_management/packs/active/world_process_exec_tracing_parity/spec_manifest.md`, `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json`.
2. Confirm the feature directory is present in `docs/project_management/packs/sequencing.json`.

## Required commands

- `make planning-lint FEATURE_DIR="docs/project_management/packs/active/world_process_exec_tracing_parity"`
- `bash -n docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
- `bash -n docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/macos-smoke.sh`

## End Checklist

1. Record command outputs in `docs/project_management/packs/active/world_process_exec_tracing_parity/session_log.md`.
2. Confirm no hard-ban or ambiguity-word violations remain.
