# Kickoff: WFGADAXA1-test (test)

## Scope
- Tests only; no production code.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA1-spec.md`
- Authoritative: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` (§2) and `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa1-test` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa1-test` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Update/add tests so world-agent rejects:
  - `schema_version != 3`
  - unknown fields (deny_unknown_fields)
  - missing `policy_snapshot` (for both HTTP execute and WS start_session)
- Run:
  - `cargo fmt`
  - targeted tests you add/touch

## End checklist
1. Run required commands; capture outputs for handoff.
2. `make triad-task-finish TASK_ID="WFGADAXA1-test"`

