# Kickoff: WFGADAXA1-code (code)

## Scope
- Production code only; no new tests.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA1-spec.md`
- Authoritative: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` (§2) and `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa1-code` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa1-code` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Implement PolicySnapshotV3 end-to-end (types + shell emit + world-agent validate/require).
- Enforce V3-only: reject legacy snapshot schema versions and missing snapshots on the protocol surfaces.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Do not add tests.

## End checklist
1. Run required commands; capture outputs for handoff.
2. `make triad-task-finish TASK_ID="WFGADAXA1-code"`

