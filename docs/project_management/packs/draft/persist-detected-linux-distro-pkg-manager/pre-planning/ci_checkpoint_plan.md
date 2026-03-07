# persist-detected-linux-distro-pkg-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md` (draft slice skeleton + invariants)
- Slice specs: see `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`.
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice tasks yet, so this plan is not mechanically validated yet; the slice list below is derived from the canonical draft slice inventory in `pre-planning/minimal_spec_draft.md`.

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
      "slices": ["PDLDPM0", "PDLDPM1", "PDLDPM3", "PDLDPM2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single checkpoint for the whole feature (4 slices fits the default checkpoint window). Boundary after PDLDPM2 is the first contract-complete seam: the additive host_state.platform persistence contract, Linux successful-install write guarantee, dev-installer parity for the shared install-state contract, and Linux smoke evidence all exist together, so compile parity plus Linux smoke can validate the feature without splitting one coherent installer contract across multiple checkpoints."
    }
  ]
}
```

## Human-readable rationale (required)

For each checkpoint, explain:
- Why the boundary is code-grounded (subsystem seam / contract completion / enabling refactor / UX seam).
- What surfaces are stabilized by this checkpoint (from `spec_manifest.md`).
- What risk is reduced by running cross-platform CI here (from `impact_map.md`).

### CP1 (`PDLDPM0`, `PDLDPM1`, `PDLDPM3`, `PDLDPM2`) — install-state persistence contract completion seam

- Code-grounded boundary:
  - Contract completion seam: `PDLDPM0` establishes the additive `host_state.platform.*` persistence surface, `PDLDPM1` completes the Linux successful-install file-presence and compatibility-preserving merge rules, `PDLDPM3` brings the dev installer onto the same install-state contract, and `PDLDPM2` adds the Linux smoke assertions that make the contract observable end-to-end.
  - UX seam: after `PDLDPM2`, the operator-visible claim that successful Linux installs leave a stable `install_state.json` artifact becomes testable rather than speculative.
- Stabilized surfaces (from `spec_manifest.md` ownership):
  - `contract.md` for the resolved `install_state.json` path, Linux-only write guarantee, no-fail posture, dry-run and `--no-world` rules, and downstream read fallback.
  - `install-state-schema-spec.md` for additive `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` nesting and omission semantics.
  - `compatibility-spec.md` for `schema_version=1` preservation and `host_state.group` / `host_state.linger` merge rules.
  - `platform-parity-spec.md` for Linux behavior delta and explicit macOS/Windows no-delta statements.
  - `slices/PDLDPM0/PDLDPM0-spec.md`, `slices/PDLDPM1/PDLDPM1-spec.md`, `slices/PDLDPM3/PDLDPM3-spec.md`, and `slices/PDLDPM2/PDLDPM2-spec.md` for schema capture, reliable write behavior, dev-installer parity, and smoke validation coverage.
- Risk reduced (from `impact_map.md`):
  - Detects cross-platform regressions from shared installer-script changes in `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` without paying full multi-OS cost after every slice.
  - Verifies that Linux behavior changed where intended while macOS and Windows remain parity-only no-delta platforms.
  - Catches contract drift where install-state writes remain tied to group/linger events or `--no-world` instead of the new successful-install guarantee.
  - Validates the doc and smoke alignment needed because `docs/INSTALLATION.md`, `tests/installers/install_state_smoke.sh`, and `tests/installers/install_smoke.sh` all become part of the externally visible contract.

## Follow-ups

- Mechanical validity (when slice tasks exist in `tasks.json`):
  - Ensure the real slice ids in `tasks.json` match `PDLDPM0`, `PDLDPM1`, `PDLDPM3`, and `PDLDPM2`, or update this plan and the accepted slice inventory together.
  - Replace any remaining draft or placeholder slice ids in this plan with the accepted final ids.
  - Add `CP1-ci-checkpoint` to `tasks.json` and wire it after `PDLDPM2-integ-core`, with the next slice group depending on it if new slices are later introduced.
  - Set `tasks.json` `meta.checkpoint_boundaries` to `["PDLDPM2"]`.
  - Run (must pass): `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"`
- Planning gaps to confirm during full planning:
  - Confirm whether `tests/installers/install_smoke.sh` needs explicit Linux assertions at this checkpoint or whether `tests/installers/install_state_smoke.sh` is the sole behavior-smoke owner.
  - Re-check the touch-set entry for `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` against the dispatcher constraint that ADR files are not edited in this pre-planning pass.
