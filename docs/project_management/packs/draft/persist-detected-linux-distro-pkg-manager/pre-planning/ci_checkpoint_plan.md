# persist-detected-linux-distro-pkg-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md` (draft slice skeleton + cross-cutting invariants)
- Slice specs: see `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice triads (`*-integ`) yet, so this plan is not mechanically validated yet; the slice list below is derived from the canonical slice inventory in `pre-planning/spec_manifest.md` and the draft slice skeleton in `pre-planning/minimal_spec_draft.md`.

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
      "slices": ["PDLDPM0", "PDLDPM1", "PDLDPM2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single end-of-feature checkpoint after PDLDPM2. Total slices (3) is below defaults.min=4, so one checkpoint is allowed; CP1 waits until Linux metadata persistence, reliable install_state creation/update semantics, and smoke/operator evidence form one coherent installer contract before cross-platform parity and Linux behavior smoke."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (`PDLDPM0..PDLDPM2`) — metadata persistence + reliable writes + smoke evidence seam

- Code-grounded boundary:
  - This feature reaches its first full contract-completion seam only after `PDLDPM2`.
  - `PDLDPM0` stabilizes the persisted `host_state.platform.*` field set.
  - `PDLDPM1` stabilizes successful Linux write triggers, idempotent updates, and the no-event file-creation rule.
  - `PDLDPM2` stabilizes smoke coverage and operator-facing wording for the same producer contract.
  - A single checkpoint is valid because the draft slice count is 3, which is below the default minimum of 4 triads per checkpoint.
- Surfaces stabilized at CP1 (from `pre-planning/spec_manifest.md`, `pre-planning/minimal_spec_draft.md`, and `pre-planning/impact_map.md`):
  - `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` share one install-state persistence contract.
  - `tests/installers/install_state_smoke.sh` and `docs/INSTALLATION.md` align to the same field names, effective-path wording, and write-trigger rules.
  - Full-planning contract surfaces are ready to lock coherently after this seam:
    - `contract.md`
    - `install-state-schema-spec.md`
    - `decision_register.md`
    - `slices/PDLDPM0/PDLDPM0-spec.md`
    - `slices/PDLDPM1/PDLDPM1-spec.md`
    - `slices/PDLDPM2/PDLDPM2-spec.md`
- Risk reduced by running cross-platform CI here:
  - Compile parity catches shared-script and shared-doc regressions across `linux`, `macos`, and `windows` without claiming new behavior guarantees on non-Linux platforms.
  - Linux behavior smoke validates the Linux-only contract named by the pack inputs:
    - successful installs write `install_state.json` even with no group or linger event,
    - missing `/etc/os-release` does not fail an otherwise successful install,
    - persisted manager metadata stays additive and keeps `schema_version = 1`.
  - The checkpoint catches drift between hosted install and dev install before later planning wires triad tasks.
  - The checkpoint also catches vocabulary drift against `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, which remains authoritative for selected-manager and `pkg_manager.source` values.
- Gate choice:
  - `feature_smoke = true` because the impact map selects `tests/installers/install_state_smoke.sh` as a direct touch surface and `PDLDPM2` exists specifically to lock smoke coverage and evidence.
  - `ci_testing = "quick"` because the canonical impact map excludes `crates/`, `src/`, `crates/world*`, `crates/shim`, `crates/shell`, and `crates/world-agent` from the touch set. If full planning expands the touch set into those areas, update this plan first and then raise the checkpoint gate to `full`.

## Follow-ups

This plan is not mechanically validated yet because `tasks.json` does not currently define slice integration tasks (`*-integ`) or checkpoint ops tasks.

Before running:
`python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`

...complete these wiring steps:

1. Confirm slice ids and ordering
   - Ensure the final slice ids in `tasks.json` match the accepted slice ids `PDLDPM0`, `PDLDPM1`, and `PDLDPM2` unless full planning explicitly splits or merges slices.
   - Ensure this plan's JSON `slices` list matches the deterministic slice order from `tasks.json`.

2. Add `tasks.json` checkpoint boundary metadata
   - Set `meta.checkpoint_boundaries = ["PDLDPM2"]` to match the checkpoint group boundary.
   - Keep `meta.behavior_platforms_required = ["linux"]`.
   - Keep `meta.ci_parity_platforms_required = ["linux", "macos", "windows"]`.

3. Add the checkpoint task and wiring
   - Add an ops task `CP1-ci-checkpoint` with:
     - `type: "ops"`
     - `depends_on: ["PDLDPM2-integ-core"]`
     - `kickoff_prompt: docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/kickoff_prompts/CP1-ci-checkpoint.md`

4. Wire future checkpoint gating
   - If later planning adds another checkpoint, make the first slice in the next checkpoint group depend on the prior checkpoint task.

5. Re-run mechanical validation
   - Run:
     - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`
