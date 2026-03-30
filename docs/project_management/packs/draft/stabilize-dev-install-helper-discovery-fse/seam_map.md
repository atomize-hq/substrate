# Seam Map - stabilize-dev-install-helper-discovery

This map converts the deep-planned source pack into governance-ready seam briefs without carrying forward slice-level execution units.

## Extraction axis and horizon

- **Primary seam axis**: workflow-first capability seams with one conformance seam.
- **Active seam**: `SEAM-1` because the fixed bundle layout, helper lookup order, and managed-marker shape are the upstream basis for every later seam.
- **Next seam**: `SEAM-2` because cleanup can stay bounded once `SEAM-1` publishes the real staged surface and managed-marker contract.
- **Future seam**: `SEAM-3` because cross-platform proof should consume landed truth from the first two seams rather than invent it early.

## Seam inventory

| Seam | Name | Type | Horizon | Source-pack absorption | Value boundary | Primary verification path |
| --- | --- | --- | --- | --- | --- | --- |
| `SEAM-1` | Durable helper-bundle staging + discovery | capability | active | Absorbs source slice `SDIHD0` plus the staging/discovery parts of `contract.md`, `decision_register.md`, and the narrow code touch set | After dev-install, the durable helper bundle exists under `$SUBSTRATE_HOME` and `substrate world enable` prefers it before falling back to the inferred version-dir helper | Prefix bundle existence checks, helper-order tests, `cargo clean` fallback proof, CLI flag rejection proof |
| `SEAM-2` | Managed cleanup + protected-path guard | capability | next | Absorbs source slice `SDIHD1` plus the managed-only cleanup and refusal parts of `contract.md` and the selected ownership decisions | Dev-uninstall removes only repo-managed assets, never deletes user-managed paths, and reports protected-path refusal deterministically | Managed-symlink cleanup proof, manifest-tracked copy cleanup proof, refusal cases for user-managed regular files and non-managed symlinks |
| `SEAM-3` | Cross-platform proof + drift guards | conformance | future | Absorbs `platform-parity-spec.md`, `manual_testing_playbook.md`, `smoke/*`, checkpoint wiring intent, and quality-gate evidence surfaces | The feature contract stays provable and non-drifting across Linux/macOS behavior validation and Windows compile parity | Smoke wrappers, manual playbook alignment, checkpoint boundary proof, stale-trigger capture after upstream seams land |

## Why these seams were kept

- `SEAM-1` is a real user-facing capability seam, not a raw “script seam”: it changes the lived install/enable workflow and can be verified independently.
- `SEAM-2` is a separate capability seam because cleanup safety has its own operator-facing contract, its own failure class, and a different verification path than staging and discovery.
- `SEAM-3` is a justified conformance seam because the remaining meaningful work is cross-seam hardening: smoke evidence, checkpoint proof, and drift guards that should not be hidden as cleanup.

## Candidates that were pruned

- **Standalone contract-definition seam**: pruned because the helper-order, bundle-layout, and cleanup-guard contracts are inseparable from the seams that produce them. Keeping a separate “contracts” seam here would mostly mirror the source planning workstreams and create an org-chart seam.
- **Standalone task/checkpoint orchestration seam**: pruned because `tasks.json`, kickoff prompts, and checkpoint wiring are governance scaffolds, not product or system value boundaries.
- **Standalone macOS provisioning seam**: pruned because the source pack explicitly narrows macOS scope to helper discovery and validation rather than full provisioning parity.
- **Standalone Windows seam**: pruned because Windows remains compile-parity only and does not add a new user-facing behavior surface in this feature.

## Source-pack mapping notes

- Source slice `SDIHD0` maps cleanly to `SEAM-1`.
- Source slice `SDIHD1` maps cleanly to `SEAM-2`.
- Source checkpoint `CP1-ci-checkpoint`, `platform-parity-spec.md`, `manual_testing_playbook.md`, and `smoke/*` map to `SEAM-3`.
- Source pack documents such as `plan.md`, `contract.md`, `decision_register.md`, `impact_map.md`, and `spec_manifest.md` act as pack-wide authorities feeding all seams; they are not emitted here as standalone seams.

## Decomposition posture

- `SEAM-1` is ready for downstream authoritative seam-local planning because it owns the first publishable contracts and the narrowest critical-path landing.
- `SEAM-2` should stay only provisionally deeper-planned until `SEAM-1` publishes the exact landed bundle layout and managed-marker reality.
- `SEAM-3` remains seam-brief depth only; it must consume closeout-backed truth from `SEAM-1` and `SEAM-2` rather than guess it in advance.
