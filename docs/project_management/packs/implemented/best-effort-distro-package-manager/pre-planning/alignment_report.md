## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- None detected

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md — record the selected fallback PATH order and the selected wrapper exit-code pass-through rule. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L315`)
- docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md — record the selected alternate os-release input contract: `SUBSTRATE_INSTALL_OS_RELEASE_PATH`. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L316`)
- docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md — record that `smoke/linux-smoke.sh` is a thin wrapper over `tests/installers/pkg_manager_detection_smoke.sh`. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L317`)
- docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md — pin `tests/installers/pkg_manager_detection_smoke.sh` as the exact hermetic repo test path and align the selected `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contract. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L320`)
- docs/project_management/packs/implemented/best-effort-distro-package-manager/tasks.json — narrow the pack to Linux-only behavior and CI parity metadata, then add `BEDPM0`, `BEDPM1`, `BEDPM2`, and `BEDPM3` triads. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L321`)
- docs/project_management/packs/sequencing.json — add the feature entry and sequence ADR-0031 ahead of `persist-detected-linux-distro-pkg-manager`. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L322`)
- docs/project_management/adrs/draft/ADR-0031-detecting-badger.md — correct Related Docs and feature-directory path drift from `detecting-badger` to `best-effort-distro-package-manager`. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L325`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/ — remove any remaining ambiguity by inheriting `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, `<unknown>`, and `pkg_manager.source` exactly from this pack once ADR-0031 planning artifacts exist. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md#L326`)
- Reconcile `pre-planning/spec_manifest.md` to the accepted 4-slice order and the now-present `pre-planning/ci_checkpoint_plan.md`. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L247`)
- When full planning adds structured task and checkpoint metadata, set the end-of-checkpoint boundary to `BEDPM3` unless the accepted slice order changes first. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L248`)
- Re-open the `BEDPM2` boundary when lift recomputation reports `estimated_slices >= 5` after the missing structured fields are filled. (sources: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L249`)

