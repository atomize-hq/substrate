## Misalignment / follow-ups (wrapper-detected)
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/stashing-ferret/` but pack dir is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json#L1`)

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/stashing-ferret/` but pack dir is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json#L1`)

### Decision Register required
- DR-0001 — Pin exact persistence behavior when platform metadata inputs are incomplete. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L278`)
- DR-0002 — Pin exact dry-run semantics for install-state persistence. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L279`)
- DR-0003 — Record the installer-entrypoint scope decision selected in this impact map (`install-substrate.sh` + `dev-install-substrate.sh`). (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L280`)

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md — define the Linux-only file-presence guarantee, the selected dual-installer scope, the `--no-world` rule, the dry-run rule, the no-fail write posture, and the downstream read fallback contract. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L283`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md — define exact `host_state.platform.*` nesting, omission rules for unavailable `os_release.*` fields, and the preservation rule for existing group/linger content while linking externally to the detection contract. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L284`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md — define merge/reset behavior for corrupt or wrong-schema files and restate the `schema_version=1` invariant. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L285`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md — define Linux as the only behavior delta and capture explicit no-delta evidence expectations for macOS and Windows. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L286`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md — include exact validation commands for `tests/installers/install_state_smoke.sh` and `tests/installers/install_smoke.sh`, plus the sequencing boundary against `best-effort-distro-package-manager`. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L287`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json — populate `PDLDPM0`, `PDLDPM1`, `PDLDPM3`, and `PDLDPM2`, and reconcile `meta.behavior_platforms_required` with Linux-only behavior plus macOS/Windows no-delta evidence. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L288`)
- docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md — reconcile the feature-directory and Related Docs drift from `stashing-ferret` to `persist-detected-linux-distro-pkg-manager` before the quality gate. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L289`)
- ADR feature-directory paths conflict with the dispatcher-selected feature directory. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L292`)
- Dry-run persistence semantics are not defined by ADR-0032. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L296`)
- Installer-entrypoint scope is not pinned. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L300`)
- Dependency-owned detection semantics must stay external. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L304`)
- When full planning accepts or rejects `PDLDPM3`, update `pre-planning/ci_checkpoint_plan.md` and `tasks.json` together. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L303`)
- If full planning rejects the split, treat dev-installer parity as the highest-risk sub-seam inside `PDLDPM1` and avoid mixing unrelated helper-staging changes into that slice. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L304`)

