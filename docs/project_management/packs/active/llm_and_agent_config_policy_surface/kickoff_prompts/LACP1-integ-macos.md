# Kickoff: LACP1-integ-macos (integration platform fix)

## Scope
- Fix macOS-specific smoke failures for LACP1 and re-run macOS smoke to green.
- Spec: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/LACP1-spec.md`
- Smoke: `docs/project_management/packs/active/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify you are in worktree `wt/llm-and-agent-config-policy-surface-lacp1-integ-macos` on branch `llm-and-agent-config-policy-surface-lacp1-integ-macos` and `.taskmeta.json` exists at the worktree root.
2. Ensure CP1 smoke for macOS exists and is failing for this slice/commit.

## Required commands
```bash
export FEATURE_DIR="docs/project_management/packs/active/llm_and_agent_config_policy_surface"
export ORCH_REF="feat/llm-and-agent-config-policy-surface"
make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=macos SMOKE_SLICE_ID="LACP1" SMOKE_CHECKOUT_REF="$(git rev-parse HEAD)" RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_REF" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0
```

## End Checklist
1. Ensure macOS smoke is green and record the run URL in `session_log.md`.
2. Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
3. From inside the worktree, run: `make triad-task-finish TASK_ID="LACP1-integ-macos"`

