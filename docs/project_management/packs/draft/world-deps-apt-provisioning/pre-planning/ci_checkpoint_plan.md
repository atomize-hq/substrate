# world-deps-apt-provisioning — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/impact_map.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice triads (`*-integ`) yet, so this plan’s slice list is based on the draft slice skeleton in `minimal_spec_draft.md` and is not mechanically validated yet.

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 4,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["WDAP0", "WDAP1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single end-to-end checkpoint after the provisioning-time APT workflow (WDAP0) and the runtime fail-early + remediation posture (WDAP1) compose a coherent operator experience. Total slices (2) is below defaults.min=4, so one checkpoint is acceptable; add an intermediate checkpoint only if planning splits slices or a standalone WDAP0 risk seam is identified."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`WDAP0..WDAP1`) — end-to-end provisioning + runtime fail-early stabilization

- Code-grounded boundary (contract completion seam):
  - By the end of `WDAP1`, the feature’s core operator contract should be coherent end-to-end:
    - provisioning-time APT install is explicit (`substrate world enable --provision-deps`), and
    - runtime `world deps current sync|install` is fail-closed for APT-backed items (no APT/dpkg execution; deterministic exit + remediation).
- Surfaces stabilized at this checkpoint (from `spec_manifest.md` ownership + `minimal_spec_draft.md` slice skeleton):
  - Operator contract + remediation/exit-code invariants: `contract.md`
  - Cross-cutting invariants (fail-closed, no host OS mutation, remediation command invariant): `minimal_spec_draft.md`
  - Slice specs:
    - `slices/WDAP0/WDAP0-spec.md` (provisioning-time APT derivation/execution; `--dry-run`; idempotency)
    - `slices/WDAP1/WDAP1-spec.md` (runtime fail-early scope; remediation invariants; no APT)
  - Cross-platform behavior smoke procedures:
    - `smoke/linux-smoke.sh` (Linux host-native unsupported provisioning posture + remediation)
    - `smoke/macos-smoke.sh` (macOS Lima guest provisioning posture)
    - `smoke/windows-smoke.ps1` (Windows/WSL guest provisioning posture; finalize support contract during planning)
- Risk reduced by running cross-platform CI here (from `impact_map.md`):
  - Backend capability matrix divergence (Linux host-native vs macOS Lima vs Windows WSL) is a frequent source of “primary dev platform is green but other platforms fail” drift; compile parity + smoke at this seam catches it early.
  - Provisioning is a safety-sensitive mutation surface; smoke across behavior platforms validates the “no host OS mutation” guard rails and the runtime prohibition on APT/dpkg.
  - This feature touches `crates/world-agent/src/service.rs` and multiple operator-facing docs/scripts; full CI testing mode provides broader regression signal than compile-only parity.

## Follow-ups

This plan cannot be mechanically validated yet because `tasks.json` does not currently define slice integration tasks (`*-integ`) or checkpoint ops tasks.

Before running:
`python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/world-deps-apt-provisioning"`

…complete these wiring steps:

1) Add slice triads to `tasks.json`
   - Create `WDAP0-*` and `WDAP1-*` code/test/integration tasks per `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`.

2) Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform)
   - Set `meta.checkpoint_boundaries = ["WDAP1"]` to match the checkpoint group boundary (the last slice in each checkpoint group).

3) Add checkpoint task + kickoff prompt + deps
   - Add an ops task `CP1-ci-checkpoint` with:
     - `type: "ops"`
     - `depends_on: ["WDAP1-integ-core"]`
     - `kickoff_prompt: docs/project_management/packs/draft/world-deps-apt-provisioning/kickoff_prompts/CP1-ci-checkpoint.md`
   - If additional checkpoints are introduced (e.g., a WDAP0-only risk seam), wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task.

