# Kickoff: CP1-ci-checkpoint (ops)

## Scope
- Dispatch feature smoke gates after the LACP1 core merge boundary.
- Plan: `docs/project_management/next/llm_and_agent_config_policy_surface/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Confirm orchestration branch: `feat/llm-and-agent-config-policy-surface`.
2. Confirm `LACP1-integ-core` is integrated and pushed.

## Required commands

Suggested env setup (copy/paste):
```bash
export FEATURE_DIR="docs/project_management/next/llm_and_agent_config_policy_surface"
export ORCH_REF="feat/llm-and-agent-config-policy-surface"
export CHECKOUT_SHA="$(git rev-parse HEAD)"
```

Run advisory audit:
- `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "$ORCH_REF" --feature-dir "$FEATURE_DIR" --ledger-path "$FEATURE_DIR/logs/CP1/ci-audit/ledger.jsonl"`

Dispatch smoke:
- `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=linux SMOKE_SLICE_ID="LACP1" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_REF" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
- `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=macos SMOKE_SLICE_ID="LACP1" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_REF" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`

## End Checklist
1. Record run URLs and outcomes in `docs/project_management/next/llm_and_agent_config_policy_surface/session_log.md`.

