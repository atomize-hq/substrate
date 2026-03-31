# stabilize-dev-install-helper-discovery — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/spec_manifest.md`
- Slice specs: see `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/spec_manifest.md` (slice specs live under `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/slices/<SLICE_ID>/<SLICE_ID>-spec.md`).

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: once slice tasks exist, update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted).

## Machine-readable plan (linted)

Pre-planning note:
- `tasks.json` does not yet define slice tasks (`*-integ*`) or checkpoint tasks, so mechanical validation is expected to fail until full planning wires the task graph.

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
      "slices": ["SDIHD0", "SDIHD1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single checkpoint for the full feature (2 slices < min_triads_per_checkpoint=4). Boundary after SDIHD1 ensures the helper staging + ownership-guarded cleanup contract is complete before running cross-platform gates."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (SDIHD0 → SDIHD1)

Why this boundary is code-grounded:
- **Contract completion seam:** the feature’s user-visible contract is “staged helpers exist under `$SUBSTRATE_HOME/scripts/substrate/...` and can be safely removed only when dev-managed”. That contract is only complete after both:
  - `SDIHD0` stages helpers and stabilizes helper discovery, and
  - `SDIHD1` defines and implements ownership-guarded cleanup + refusal posture.

What surfaces this checkpoint stabilizes (from `spec_manifest.md`):
- CLI helper discovery contract subset for `substrate world enable` (including fail-closed behavior when helper is missing).
- Dev-install helper staging contract under `$SUBSTRATE_HOME/scripts/substrate/...` (paths + overwrite policy).
- Dev-uninstall cleanup contract (ownership guard + protected-path invariants).
- Exit-code posture aligned to taxonomy (no new exit codes by default).
- Platform guarantees: Linux/macOS supported; Windows “unsupported/no-change” for behavior (but still included in CI parity).

What risk is reduced by running cross-platform CI here (from `impact_map.md`):
- Validates cross-platform parity for shell + scripts changes (`dev-install-substrate.sh`, `dev-uninstall-substrate.sh`, `paths.rs`, `world_enable.rs`).
- Validates the “`cargo clean` removed `<repo>/target/scripts/...` but helper is still resolved from `$SUBSTRATE_HOME/scripts/substrate/...`” behavior.
- Validates deterministic ownership-guarded cleanup behavior, reducing the chance of platform-specific path/permission edge cases.

## Follow-ups

- Add slice tasks to `tasks.json` (including `*-integ` / `*-integ-core`) and add the `CP1-ci-checkpoint` ops task wired per the CI checkpoint standard.
- For schema v4 cross-platform packs, set `tasks.json` `meta.checkpoint_boundaries = ["SDIHD1"]` to match this plan once slice tasks exist.
- Once the above wiring exists, run (must pass): `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/stabilize-dev-install-helper-discovery"`.
