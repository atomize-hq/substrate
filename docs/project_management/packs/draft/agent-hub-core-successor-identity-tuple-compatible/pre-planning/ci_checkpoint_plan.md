# agent-hub-core-successor-identity-tuple-compatible — CI checkpoint plan (pre-planning)

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/minimal_spec_draft.md`
- Slice specs: see `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`.

## Operator rules
- This plan is authoritative for **CI cadence** during pre-planning.
- If a mismatch appears between this plan and the accepted slice graph, platform scope, or contract surfaces, update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4 cross-platform automation packs, update `tasks.json` `meta.checkpoint_boundaries` to list the last slice in each checkpoint group after the slice task graph exists.
- Pre-planning note: `tasks.json` does not define slice tasks yet, so this plan uses the draft slice skeleton from `pre-planning/minimal_spec_draft.md`. Mechanical validation starts after full planning adds slice integration tasks and checkpoint task wiring.

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 1,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["AHCSITC0", "AHCSITC1", "AHCSITC2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Boundary after AHCSITC2 groups the operator contract, session protocol, fail-closed policy, and telemetry split into one high-risk stabilization seam before the final parity and compatibility closure slice."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["AHCSITC3"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "full"
      },
      "rationale": "Final checkpoint after AHCSITC3 closes the platform-parity, compatibility, and validation surfaces on the accepted operator contract."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`AHCSITC0` → `AHCSITC2`)

Why this boundary is code-grounded:
- `spec_manifest.md` defines an explicit checkpoint boundary after `AHCSITC2`.
- `AHCSITC0`, `AHCSITC1`, and `AHCSITC2` lock one contiguous chain of high-risk surfaces: operator contract, orchestrator and member session protocol, fail-closed control-plane gating, and the pure-agent versus nested-gateway telemetry split.
- `impact_map.md` ties those slices to shared shell, agent API, common event-envelope, and trace surfaces. Running parity before `AHCSITC3` prevents the final compatibility slice from absorbing unresolved protocol or telemetry drift.

What surfaces this checkpoint stabilizes:
- `contract.md` operator-facing list, status, and doctor semantics.
- `agent-hub-session-protocol-spec.md` capability discovery, session lifecycle, host-scoped orchestrator selection, and world-scoped member routing.
- `policy-spec.md` fail-closed deny posture and backend allowlist rules.
- `telemetry-spec.md` pure-agent and nested-gateway publication split, correlation keys, and redaction posture.

What risk is reduced by running cross-platform CI here:
- Compile parity catches cross-platform build and type drift across `crates/shell`, `crates/agent-api-*`, `crates/common`, and `crates/trace` before the final validation slice starts.
- `ci_testing = quick` exercises the high-risk contract and telemetry path without paying full-suite cost before the parity and compatibility closure slice lands.
- `feature_smoke = false` in this first pass because the pack defines no feature-local `smoke/` surface and `spec_manifest.md` assigns deterministic validation ownership to `manual_testing_playbook.md`.
- Windows remains part of the compile-parity checkpoint set only. The Windows WSL warm or smoke flow is not a CI checkpoint requirement for `CP1`.

Checkpoint-size justification:
- This group contains 3 slices, which stays within the machine-readable checkpoint range of 1 to 8 slices.
- The grouping remains code-grounded because `spec_manifest.md` requires the boundary after `AHCSITC2`, and `impact_map.md` shows that the first three slices carry the highest-risk contract, protocol, policy, and telemetry work.

### CP2 (`AHCSITC3`)

Why this boundary is code-grounded:
- `spec_manifest.md` defines the final checkpoint boundary after `AHCSITC3`.
- `AHCSITC3` owns the platform-parity, compatibility, and validation closure surfaces, so the last checkpoint belongs at the end of that slice.

What surfaces this checkpoint stabilizes:
- `platform-parity-spec.md` Linux, macOS, and Windows operator-visible parity guarantees.
- `compatibility-spec.md` ADR-0025 supersession and rollout closure.
- `manual_testing_playbook.md` deterministic validation evidence for list, status, doctor, and nested-record visibility.

What risk is reduced by running cross-platform CI here:
- Compile parity confirms the final compatibility and parity edits did not reopen cross-platform shell or trace regressions.
- `ci_testing = full` provides the last cross-platform confidence gate before planning promotes this pack into execution.
- `feature_smoke = false` remains correct in this first pass because the pack still has no feature-local smoke surface. Full planning updates this gate if a smoke workflow becomes part of the accepted validation contract.
- Windows remains part of the compile-parity checkpoint set only. The Windows WSL warm or smoke flow is not a CI checkpoint requirement for `CP2`.

Checkpoint-size justification:
- This group contains 1 slice, which stays within the machine-readable checkpoint range of 1 to 8 slices.
- The grouping remains code-grounded because the final slice is dedicated to parity and rollout closure, so the final checkpoint must run after that slice completes.

## Follow-ups

- Add slice triad tasks for `AHCSITC0` through `AHCSITC3` in `tasks.json`.
- Add checkpoint ops tasks `CP1-ci-checkpoint` and `CP2-ci-checkpoint` plus kickoff prompts.
- Set `tasks.json` `meta.checkpoint_boundaries = ["AHCSITC2", "AHCSITC3"]` after the slice task graph exists.
- Replace the draft slice list in this plan if full planning renames, splits, or merges any slice ids.
- Update the gate posture if full planning adds a feature-local smoke surface under `smoke/`.
- Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible"` after the slice task graph, checkpoint tasks, and checkpoint boundaries exist in `tasks.json`.
