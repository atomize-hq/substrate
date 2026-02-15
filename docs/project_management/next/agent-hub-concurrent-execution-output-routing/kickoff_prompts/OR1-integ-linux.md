# Kickoff: OR1-integ-linux (integration platform-fix — linux)

## Scope
- Ensure the OR1 slice is green on linux.
- This task may change production code and tests as needed to fix linux failures.
- This task must not merge back to the orchestration branch; OR1-integ final performs the merge once all platforms are green.
- Spec: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/OR1-spec.md`
- Execution workflow standard: `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Run this task on a linux machine.
2. Verify you are in the task worktree `wt/agent-hub-concurrent-execution-output-routing-or1-integ-linux` on branch `agent-hub-concurrent-execution-output-routing-or1-integ-linux` and that `.taskmeta.json` exists at the worktree root.
3. Read: `plan.md`, `tasks.json`, `session_log.md`, `OR1-spec.md`, and this prompt.
4. If `.taskmeta.json` is missing or mismatched, stop and ask the operator to run:
   - `make triad-task-start FEATURE_DIR="docs/project_management/next/agent-hub-concurrent-execution-output-routing" TASK_ID="OR1-integ-linux" TASK_PLATFORM=linux`

## Requirements
- Merge the slice core integration branch into this worktree:
  - `CORE_BRANCH="$(jq -r --arg id "OR1-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "docs/project_management/next/agent-hub-concurrent-execution-output-routing/tasks.json")"`
  - `git merge "$CORE_BRANCH"`
- Run local quality gates:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Run advisory audit and record outputs:
  - `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/next/agent-hub-concurrent-execution-output-routing/logs/OR1/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "feat/agent-hub-concurrent-execution-output-routing" --required-platforms linux`
- Dispatch linux smoke via CI and record the run id/URL in session_log.md:
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/agent-hub-concurrent-execution-output-routing" PLATFORM=linux SMOKE_SLICE_ID="OR1" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/agent-hub-concurrent-execution-output-routing" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- If smoke fails: apply a minimal fix, then re-run smoke until green.

## End Checklist
1. Ensure linux smoke is green and capture the run id/URL.
2. From inside the worktree, run: `make triad-task-finish TASK_ID="OR1-integ-linux"`
3. Hand off run id/URL and linux-specific notes to the operator (do not edit planning docs inside the worktree).

