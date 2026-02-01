# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run the cross-platform CI gates defined by `docs/project_management/next/world-fs-granular-allow-deny/ci_checkpoint_plan.md`.
- This task runs on the orchestration checkout (no worktree).
- Do not edit planning docs inside any worktree.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Ensure you are on the orchestration branch `feat/world-fs-granular-allow-deny` (or the orchestration worktree).
2. Read: `docs/project_management/next/world-fs-granular-allow-deny/ci_checkpoint_plan.md`, `docs/project_management/next/world-fs-granular-allow-deny/tasks.json`, `docs/project_management/next/world-fs-granular-allow-deny/session_log.md`, `docs/project_management/next/world-fs-granular-allow-deny/impact_map.md`.
3. Confirm this checkpoint validates slice `WFGAD1` and its core integration task `WFGAD1-integ-core`.
4. Determine the exact commit this checkpoint validates:
   - `CORE_BRANCH="$(jq -r --arg id \"WFGAD1-integ-core\" '.tasks[] | select(.id==$id) | .git_branch' \"docs/project_management/next/world-fs-granular-allow-deny/tasks.json\")"`
   - `CHECKOUT_SHA="$(git rev-parse \"$CORE_BRANCH\")"`

## Required gates (dispatch from orchestration checkout)

1) Cross-platform compile parity (GitHub-hosted):
- `make ci-compile-parity CI_WORKFLOW_REF="feat/world-fs-granular-allow-deny" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`

2) CI Testing (quick):
- `scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/world-fs-granular-allow-deny" --remote origin --cleanup --mode quick --checkout-ref "$CHECKOUT_SHA"`

Feature Smoke:
- Not required by `ci_checkpoint_plan.md` for this checkpoint.

## End Checklist
1. Record run ids/URLs (compile parity + CI Testing) in `docs/project_management/next/world-fs-granular-allow-deny/session_log.md`.
2. Mark this task `completed` in `docs/project_management/next/world-fs-granular-allow-deny/tasks.json` and add an END entry.
