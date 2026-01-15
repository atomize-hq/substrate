# Workspace Config/Policy Unification (ADR-0008) — Plan

## Context (why this exists)
This Planning Pack implements the operator-facing contract defined by:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

This body of work also **must** implement the cross-cutting refinement defined by:
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`

Non-negotiable additional gating for this pack:
- `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`

## Execution model (triads + automation)
- Standards:
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
  - `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md` (because `tasks.json` enables execution gates)
- Orchestration branch: `feat/workspace-config-policy-unification`
- Docs discipline:
  - planning docs (`plan.md`, `tasks.json`, `session_log.md`, specs, prompts) are edited only on the orchestration branch
  - do not edit planning docs inside worktrees

## Hard requirements (operator contract)
- Patch files are sparse YAML mappings; omitted keys mean “inherit”.
- Scopes are explicit and symmetric for config and policy:
  - `current` (effective/merged)
  - `global` (patch view)
  - `workspace` (patch view)
- Workspace root discovery uses `<workspace_root>/.substrate/workspace.yaml` and respects `<workspace_root>/.substrate/workspace.disabled`.
- `current show --explain` is deterministic and (per ADR-0012) supports **multi-source** keys where effective values are derived from multiple layers.

## Cross-cutting dependency (must ship in this pack)
Phase A/B from ADR-0012 must be completed in this body of work (not deferred):
- Phase A: per-key merge strategies + multi-source provenance
- Phase B: config editor supports `world.deps.enabled` (list merge key)

See:
- `docs/project_management/next/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`

## Triad slices (high-level)
This plan follows the sequencing spine entry:
- `docs/project_management/next/sequencing.json` → `workspace_config_policy_unification`

Slices:
- WCU1: Workspace directory + internal git unification
- WCU2: Patch parsing + merge (includes ADR-0012 Phase A)
- WCU3: CLI scopes + reset semantics + explain (includes ADR-0012 Phase B)
- WCU4: Installer/dev env cleanup (stop exporting `SUBSTRATE_OVERRIDE_*` by default)
- WCU5: Docs + smoke + playbook alignment (including Phase A/B validation coverage)

## Validation artifacts (required)
- Manual playbook:
  - `docs/project_management/next/workspace-config-policy-unification/manual_testing_playbook.md`
- Smoke scripts (must mirror manual playbook; required for behavior platforms):
  - `docs/project_management/next/workspace-config-policy-unification/smoke/linux-smoke.sh`
  - `docs/project_management/next/workspace-config-policy-unification/smoke/macos-smoke.sh`
  - `docs/project_management/next/workspace-config-policy-unification/smoke/windows-smoke.ps1`

## Execution gates (enabled)
- Feature start gate:
  - Task: `F0-exec-preflight` in `docs/project_management/next/workspace-config-policy-unification/tasks.json`
  - Report: `docs/project_management/next/workspace-config-policy-unification/execution_preflight_report.md`
- Per-slice closeout gates (one per slice):
  - `docs/project_management/next/workspace-config-policy-unification/WCU1-closeout_report.md`
  - `docs/project_management/next/workspace-config-policy-unification/WCU2-closeout_report.md`
  - `docs/project_management/next/workspace-config-policy-unification/WCU3-closeout_report.md`
  - `docs/project_management/next/workspace-config-policy-unification/WCU4-closeout_report.md`
  - `docs/project_management/next/workspace-config-policy-unification/WCU5-closeout_report.md`
