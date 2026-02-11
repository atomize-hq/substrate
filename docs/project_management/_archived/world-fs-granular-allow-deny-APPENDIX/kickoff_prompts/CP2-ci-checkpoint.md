# Kickoff: CP2-ci-checkpoint (ops)

## Scope
- Cross-platform CI checkpoint dispatch and evidence capture.
- Checkpoint plan: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm `WFGADAX3-integ-core` is completed.
2. Set:
   - `export FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX"`
   - `export ORCH_REF="$(git branch --show-current)"`
   - `export SMOKE_SLICE_ID="WFGADAX3"`
3. Pin the exact commit under test (`checkout_ref`) to the boundary slice’s core integration HEAD:
   - `CORE_BRANCH="$(jq -r --arg id "${SMOKE_SLICE_ID}-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "$FEATURE_DIR/tasks.json")"`
   - `CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"`
4. Run advisory CI audit (exit `0`) and record the `RECOMMENDATION:` line(s) in `session_log.md`:
   - `scripts/ci-audit/ci_audit.sh --ledger-path "$FEATURE_DIR/logs/$SMOKE_SLICE_ID/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "$ORCH_REF"`
   - `scripts/ci-audit/ci_audit.sh --ledger-path "$FEATURE_DIR/logs/$SMOKE_SLICE_ID/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "$ORCH_REF" --feature-dir "$FEATURE_DIR"`
5. If ci-audit recommends RUN, dispatch the planned gates (each dispatch exit `0`):
   - `make ci-compile-parity CI_WORKFLOW_REF="$ORCH_REF" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
   - `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=behavior SMOKE_SLICE_ID="$SMOKE_SLICE_ID" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_REF" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## If smoke fails
Start only failing platform-fix tasks from the orchestration checkout:
- Single multi-platform smoke run (`PLATFORM=behavior`):
  - `make triad-task-start-platform-fixes-from-smoke FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SMOKE_SLICE_ID" SMOKE_RUN_ID="<run-id>" LAUNCH_CODEX=1`
- Per-platform smoke runs:
  - `make triad-task-start-platform-fixes FEATURE_DIR="$FEATURE_DIR" SLICE_ID="$SMOKE_SLICE_ID" PLATFORMS="<csv>" LAUNCH_CODEX=1`

## End Checklist
1. Record ORCH_REF, CHECKOUT_SHA, ci-audit output lines, and dispatched run URLs/ids (or explicit SKIP) in `session_log.md`.
2. Mark the task as completed in `tasks.json`.
