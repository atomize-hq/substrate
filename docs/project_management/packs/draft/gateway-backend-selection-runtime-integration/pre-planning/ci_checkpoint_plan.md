# gateway-backend-selection-runtime-integration — CI checkpoint plan

This file defines where later multi-platform verification is expected to happen for this feature.

Standard:
- `docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/minimal_spec_draft.md`

## Operator rules
- This plan is authoritative for checkpoint intent during pre-planning.
- If reality diverges from this plan, update this plan first.
- This document remains advisory until downstream FSE planning or decomposition turns the checkpoint cadence into concrete execution behavior.

## Machine-readable plan

```json
{
  "version": 1,
  "defaults": {
    "min_draft_seams_per_checkpoint": 2,
    "max_draft_seams_per_checkpoint": 6
  },
  "platform_scope": {
    "ci_parity_platforms": ["linux", "macos", "windows"],
    "behavior_validation_platforms": ["linux", "macos", "windows"],
    "wsl": "follow_up_required"
  },
  "checkpoints": [
    {
      "checkpoint_id": "CP1",
      "draft_seam_ids": ["GBSRI-01", "GBSRI-02"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "targeted",
        "targeted_platform_validation": []
      },
      "stabilized_surfaces": [
        "selected-backend resolution through config, policy, and inventory",
        "backend-aware auth handoff sourcing and fail-closed classification",
        "integrated adapter binding lookup and capability gating",
        "runtime config rendering, launch, readiness, and restart wiring"
      ],
      "rationale": "GBSRI-01 and GBSRI-02 form one implementation path from backend selection through integrated runtime realization. The first checkpoint lands after the selection, policy, auth, adapter-binding, and runtime-launch surfaces line up end to end, which makes compile parity, feature smoke, and targeted CI evidence meaningful."
    },
    {
      "checkpoint_id": "CP2",
      "draft_seam_ids": ["GBSRI-03"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "deeper",
        "targeted_platform_validation": ["linux", "macos", "windows"]
      },
      "stabilized_surfaces": [
        "cross-platform parity guarantees for Linux, macOS, and Windows",
        "cli:codex regression coverage plus one additional integrated backend proof",
        "manual testing playbook scope and smoke script assertions",
        "compatibility and rollout evidence for explicit unsupported-backend handling"
      ],
      "rationale": "GBSRI-03 is the parity-validation and rollout seam. It owns the heaviest Linux, macOS, and Windows confirmation work in this pack, so it stands alone as a high-risk checkpoint exception to the default minimum seam count."
    }
  ]
}
```

## Human-readable rationale

### Applicability

Checkpoint planning applies to this feature.

Reasons:
- ADR-0046 declares Linux, macOS, and Windows platform guarantees for the same selected-backend realization contract.
- `impact_map.md` ties the feature to shell, world-agent, gateway auth, gateway providers, parity tests, and per-platform smoke scripts.
- `spec_manifest.md` reserves one explicit parity-validation surface and one explicit CI checkpoint planning artifact.
- The feature is security-sensitive and fail-closed, which raises the cost of late cross-platform surprises.

### CP1

Code-grounded boundary:
- `GBSRI-01` and `GBSRI-02` are the first end-to-end realization path in the draft seam skeleton.
- `impact_map.md` treats backend selection, policy gating, auth sourcing, adapter binding, config rendering, launch, readiness, and restart as one ordered chain.
- Splitting those seams before the first checkpoint would separate the core contract from the first runtime consumer that proves it.

Surfaces stabilized:
- selected-backend resolution through config, policy, and inventory
- fail-closed handling for disallowed backends, blocked auth reads, missing inventory, and malformed selection
- integrated adapter binding lookup and capability gating
- runtime config rendering, launch, readiness, and restart wiring

Risk reduced:
- catches cross-platform compile or type-shape drift across shell, world-agent, gateway, and shared types before the parity-proof seam starts
- catches end-to-end lifecycle regressions before the pack invests in the heavier rollout and smoke-script evidence pass

Downstream confirmation still required:
- exact downstream execution labels for the targeted CI gate
- exact first additional integrated backend used in fixtures, smoke assertions, and manual validation
- exact WSL handling rule if downstream planning expands the platform matrix

### CP2

Code-grounded boundary:
- `GBSRI-03` owns `platform-parity-spec.md`, `compatibility-spec.md`, `manual_testing_playbook.md`, and the Linux/macOS/Windows smoke scripts.
- `impact_map.md` makes parity evidence, explicit unsupported-backend handling, and rollout proof part of the final seam rather than the earlier realization path.
- This checkpoint intentionally carries one draft seam because the seam itself is the cross-platform proof boundary and contains the expensive validation surfaces.

Surfaces stabilized:
- Linux, macOS, and Windows parity guarantees
- `cli:codex` regression baseline plus first additional integrated backend proof
- smoke script coverage for invalid backend, blocked backend, missing adapter binding, and missing auth
- rollout and compatibility evidence

Risk reduced:
- concentrates the expensive multi-platform pass at the point where the realization path is already stable
- prevents repeated Linux/macOS/Windows churn while the earlier realization seams are still moving
- provides the final proof that explicit unsupported-backend failure posture holds across platforms

Downstream confirmation still required:
- exact verification cadence for the deeper CI gate
- whether WSL stays out of scope or joins the final platform-validation pass
- exact evidence format recorded for platform smoke and manual validation

## Follow-ups

- Replace `GBSRI-01`, `GBSRI-02`, and `GBSRI-03` with the final downstream identifiers once they exist.
- Confirm the exact platform scope and verification cadence once downstream planning stabilizes the touched surfaces.
- Convert this checkpoint intent into concrete execution wiring only in the downstream subsystem that owns execution.
- Pin the first supported integrated backend beyond `cli:codex`; CP2 depends on that backend for parity and compatibility proof.
- Decide whether WSL remains outside this checkpoint plan or joins the final platform-validation scope.
