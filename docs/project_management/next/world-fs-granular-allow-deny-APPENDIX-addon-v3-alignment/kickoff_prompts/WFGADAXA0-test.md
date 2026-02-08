# Kickoff: WFGADAXA0-test (test)

## Scope
- Tests only (plus minimal test-only helpers if needed); no production code.
- Spec: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA0-spec.md`
- Authoritative output contract: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` (§1.3 / Appendix A.6)
- Execution standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start checklist
Do not edit planning docs inside the worktree.

1. Verify worktree: `wt/world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa0-test` on branch `world-fs-granular-allow-deny-appendix-addon-v3-wfgadaxa0-test` with `.taskmeta.json`.
2. Read: plan/tasks/session_log/spec + this prompt.

## Requirements
- Add deterministic tests that fail if `substrate policy show` output regresses to V2 shape or omits empty deny lists.
- Cover both YAML and `--json` output.
- Run:
  - `cargo fmt`
  - the targeted tests you add/touch

## End checklist
1. Run required commands; capture outputs for handoff.
2. `make triad-task-finish TASK_ID="WFGADAXA0-test"`

