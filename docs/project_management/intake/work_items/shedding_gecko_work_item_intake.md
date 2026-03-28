Final chosen path: `docs/project_management/intake/work_items/shedding_gecko_work_item_intake.md`

---
codename: shedding_gecko
created: "2026-03-12T15:11:33Z"
status: draft
depends_on:
  - ADR-0003
  - ADR-0011
  - ADR-0007
---

# Work Item Intake Sheet

## 1. Codename + date + status

- Codename: `shedding_gecko`
- Created: 2026-03-12T15:11:33Z
- Status: draft

## 2. Title (imperative)

Remove `manager_hooks.yaml` as a required Substrate input.

## 3. Why not ADR

- This is implementation and cleanup work that follows already-accepted direction rather than introducing a new A/B architecture choice.
- `ADR-0011` already says `world deps` must stop reading legacy manifest paths, and the current gap is that several host/runtime/install surfaces still lag that contract.
- If the team later wants to choose between materially different replacement strategies for manager-init and shim-doctor metadata, that follow-on choice can be raised as a separate ADR.

## 4. Task definition (bounded)

- Remove `manager_hooks.yaml` as a required install-time, bundle-time, and runtime dependency across Substrate.
- Replace or relax all remaining consumers that currently assume the manifest exists, including installer checks, release bundle assembly, startup manager-init generation, shim diagnostics/hinting, and affected tests/docs.
- Preserve existing user-visible behavior where it is still required by accepted ADRs, but stop requiring an on-disk `config/manager_hooks.yaml` to achieve it.
- Treat the restored repo copy of `config/manager_hooks.yaml` as a temporary compatibility bridge only; the end state is that Substrate functions correctly without requiring that file.

## 5. Done means (<= 8 outcomes)

- `scripts/substrate/dev-install-substrate.sh` no longer fails solely because `config/manager_hooks.yaml` is absent.
- Production bundle assembly and install validation no longer require `manager_hooks.yaml` as a bundled file.
- Normal `substrate` startup, `substrate shim doctor`, and shim hint logging no longer depend on loading `manager_hooks.yaml` from disk.
- `substrate world deps` remains uninfluenced by legacy manifest paths, consistent with `ADR-0011`.
- Tests that currently rely on `config/manager_hooks.yaml` are updated to validate the manifest-free path or a new replacement source.
- User-facing docs no longer describe `manager_hooks.yaml` as a required shipped/runtime artifact unless explicitly called out as temporary compatibility only.
- The temporary restored manifest file is either no longer needed or is clearly marked as transitional with follow-on removal tracked.

## 6. Likely touch paths

- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/install-substrate.sh`
- `dist/scripts/assemble-release-bundles.sh`
- `crates/shell/src/execution/manager.rs`
- `crates/shell/src/builtins/shim_doctor/`
- `crates/shim/src/exec/logging.rs`
- `crates/common/src/manager_manifest/`
- `crates/shell/tests/`
- `tests/installers/`
- `README.md`
- `docs/INSTALLATION.md`
- `docs/CONFIGURATION.md`
- `config/manager_hooks.yaml`

## 7. Dependencies (ADR/WI)

- depends_on_adrs:
  - `ADR-0011-world-deps-packages-bundles-contract`
  - `ADR-0003-policy-and-config-mental-model-simplification`
  - `ADR-0007-host-and-world-doctor-scopes`
- depends_on_work_items: []
- blocks: []

## 8. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "model_version": 1,
  "touch": {
    "create_files": 0,
    "edit_files": 12,
    "delete_files": 1,
    "deprecate_files": 0,
    "crates_touched": 3,
    "boundary_crossings": 5
  },
  "contract": {
    "cli_flags": 0,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": {
    "new_test_files": 0,
    "new_test_cases": 6
  },
  "docs": {
    "new_docs_files": 0
  },
  "ops": {
    "new_smoke_steps": 1,
    "ci_changes": 0
  },
  "risk": {
    "cross_platform": true,
    "security_sensitive": false,
    "concurrency_or_ordering": false,
    "migration_or_backfill": true,
    "unknowns_high": 2
  },
  "notes": "Estimate covers installers, bundle assembly, startup manager wiring, shim doctor/hinting, docs, and tests. One delete assumes the temporary restored manifest can be removed by the end state."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (from `make pm-lift-intake`)

```text
Lift Score (v1): 92
Estimated slices: 8
Confidence: high
Triggers:
- likely_split:crates_touched>2
- likely_split:lift_score>24
- likely_split:touch_files_sum>12
- split_required:estimated_slices>3
Missing inputs:
- None.
```

## 9. Open questions

- What replaces the remaining manager metadata consumers: compiled-in defaults, a different checked-in artifact, or narrower feature removal?
- Should `substrate shim doctor` keep manager-specific hint/report behavior, or should that surface shrink now that `world deps` no longer uses the manifest model?
- Is `config/manager_hooks.yaml` expected to disappear entirely at the end of this WI, or should a temporary compatibility phase remain after requirement removal lands?
