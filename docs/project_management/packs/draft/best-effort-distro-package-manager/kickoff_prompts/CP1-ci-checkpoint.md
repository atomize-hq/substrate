# Kickoff: CP1-ci-checkpoint (CI checkpoint)

## Scope
- Run compile parity and CI Testing quick for the `BEDPM2` checkpoint boundary.
- Stay on the orchestration checkout for `feat/best-effort-distro-package-manager`.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Read `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`, `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`, and this prompt.
2. Determine `CHECKOUT_SHA` from the `BEDPM2-integ-core` branch:

```bash
CORE_BRANCH="$(jq -r --arg id "BEDPM2-integ-core" '.tasks[] | select(.id==$id) | .git_branch' "docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json")"
CHECKOUT_SHA="$(git rev-parse "$CORE_BRANCH")"
```

## Requirements
- Use the CI audit ledger path `docs/project_management/packs/draft/best-effort-distro-package-manager/logs/BEDPM2/ci-audit/ledger.jsonl`.
- Run `scripts/ci-audit/ci_audit.sh --ledger-path "docs/project_management/packs/draft/best-effort-distro-package-manager/logs/BEDPM2/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "feat/best-effort-distro-package-manager"`.
- Run `make ci-compile-parity CI_WORKFLOW_REF="feat/best-effort-distro-package-manager" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`.
- Run `scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/best-effort-distro-package-manager" --remote origin --cleanup --mode quick --checkout-ref "$CHECKOUT_SHA"`.
- `pre-planning/ci_checkpoint_plan.md` keeps `feature_smoke=false` for CP1, so do not dispatch feature smoke from this checkpoint.

## End Checklist
1. Record the audit output, `CHECKOUT_SHA`, run ids, and run URLs in `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`.
2. Set task status to `completed` in `tasks.json`, add an END entry, and commit docs on the orchestration branch.
