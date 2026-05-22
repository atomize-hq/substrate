# substrate-gateway-backend-adapter-contract — CI checkpoint plan

This file defines where later multi-platform verification is expected to happen for this feature.

Standard:
- `docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/minimal_spec_draft.md`

## Operator rules
- This plan is authoritative for checkpoint intent during pre-planning.
- If a mismatch appears between this plan and the pack inputs, update this plan first.
- This document remains advisory until downstream FSE planning or decomposition turns the checkpoint cadence into concrete execution behavior.

## Applicability

Checkpoint planning applies to this feature because the authoritative inputs define:
- cross-platform parity guarantees for Linux, macOS, and Windows,
- fail-closed policy and capability validation behavior,
- operator-facing status and contract boundaries that cross shell, gateway, world-service, and trace-adjacent surfaces,
- a dedicated parity-and-validation draft seam (`SGBA-03`) that exists to prove platform and compatibility behavior.

The checkpoint cadence stays lightweight during pre-planning:
- `CP1` stabilizes the contract, policy, protocol, and schema seams before the feature enters parity proof.
- `CP2` validates the final parity, compatibility, and manual-validation seam after the cross-platform guarantees are fully defined.

## Machine-readable plan

```json
{
  "version": 1,
  "platform_scope": {
    "ci_parity_platforms": ["linux", "macos", "windows"],
    "behavior_platforms": ["linux", "macos", "windows"],
    "wsl_scope": "bundled_with_windows_hidden_transport_mechanics"
  },
  "defaults": {
    "min_draft_seams_per_checkpoint": 2,
    "max_draft_seams_per_checkpoint": 6
  },
  "checkpoints": [
    {
      "checkpoint_id": "CP1",
      "draft_seam_ids": ["SGBA-01", "SGBA-02"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "targeted",
        "targeted_platform_validation": []
      },
      "stabilized_surfaces": [
        "stable backend-id selection and allowlist evaluation",
        "adapter lookup and dispatch ordering",
        "capability validation and extension-key failure posture",
        "session-handle and event-translation contract boundaries"
      ],
      "rationale": "This boundary keeps the selection seam attached to the first protocol and schema consumer seam. Verification at this point confirms the fail-closed contract before parity-proof and compatibility evidence fan out across Linux, macOS, and Windows."
    },
    {
      "checkpoint_id": "CP2",
      "draft_seam_ids": ["SGBA-03"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "deeper",
        "targeted_platform_validation": ["linux", "macos", "windows"]
      },
      "stabilized_surfaces": [
        "compile-parity intent for the final parity and validation seam",
        "feature-smoke intent for the final parity and validation seam",
        "Linux, macOS, and Windows validation intent for the final parity and validation seam",
        "ADR-0024 compatibility and supersession proof",
        "manual validation evidence for operator/status, policy, event, and trace ownership"
      ],
      "rationale": "This boundary aligns with the dedicated parity-and-validation seam in the draft skeleton. Verification here confirms the compile-parity, feature-smoke, and platform-validation intent after all contract and protocol semantics are fixed and ready for cross-platform evidence collection."
    }
  ]
}
```

## Human-readable rationale

### CP1

`CP1` covers `SGBA-01` and `SGBA-02`.

Why this boundary is code-grounded:
- `impact_map.md` ties the first two seams to backend-id selection, policy gating, adapter dispatch, capability validation, session handles, and event or trace boundary lines.
- `minimal_spec_draft.md` keeps those surfaces in one contract family and keeps them ahead of the parity-and-validation seam.
- `spec_manifest.md` assigns the first two seams to the documents that define contract, policy, protocol, and schema truth.

What this checkpoint stabilizes:
- stable `<kind>:<name>` selection semantics,
- deny-by-default allowlist evaluation,
- adapter registry lookup and dispatch order,
- fail-closed capability and extension-key handling,
- the boundary between local adapter semantics and ADR-0017 or ADR-0028 owners.

Risk reduced at this checkpoint:
- prevents parity work from starting before the contract error classes and adapter lifecycle rules are fixed,
- prevents schema drift between selection semantics and the first request or response consumers,
- catches compile and smoke regressions before the work spreads into platform-proof surfaces.

Downstream confirmation still required:
- exact smoke coverage depth for the first checkpoint once downstream planning locks the touched crates and smoke script surfaces,
- exact compile-parity mode selection once downstream planning fixes the implementation blast radius.

### CP2

`CP2` covers `SGBA-03`.

Why this boundary is code-grounded:
- `minimal_spec_draft.md` reserves `SGBA-03` for parity, compatibility, validation evidence, and checkpoint intent.
- `spec_manifest.md` assigns Linux, macOS, and Windows parity guarantees plus compatibility proof to `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md`.
- `impact_map.md` identifies cross-platform parity evidence and runtime-parity alignment as explicit downstream implications.

What this checkpoint stabilizes:
- compile-parity intent for the final parity and validation seam,
- feature-smoke intent for the final parity and validation seam,
- Linux, macOS, and Windows validation intent for adapter-backed execution,
- compatibility proof that ADR-0024 remains historical evidence and no second Substrate control plane exists,
- document-validation evidence against ADR-0040, ADR-0027, ADR-0017, and ADR-0028,
- the final advisory checkpoint boundary that downstream execution wiring can consume.

Risk reduced at this checkpoint:
- catches platform-specific divergence after the contract and schema surfaces are already fixed,
- prevents release-safe validation from depending on unresolved compatibility or ownership questions,
- creates one final cross-platform verification point instead of repeated heavyweight validation after every seam.

Downstream confirmation still required:
- exact runner selection and workflow mode for compile parity, feature smoke, and deeper CI testing,
- how the Windows validation runner should exercise WSL-backed hidden transport mechanics while keeping them out of the operator-facing contract.

## Follow-ups

- Replace provisional draft seam identifiers with the final downstream identifiers once they exist.
- Confirm the exact platform scope and verification cadence once downstream planning stabilizes the touched surfaces.
- Convert checkpoint intent into concrete execution wiring only in the downstream subsystem that owns execution.
- Confirm the exact compile-parity workflow mode and the exact feature-smoke slice scope for `CP1` and `CP2` after downstream seam planning fixes the implementation scope.
- Confirm the Windows validation runner details for WSL-backed hidden transport mechanics once downstream seam planning fixes the implementation scope.
