## Misalignment / follow-ups (wrapper-detected)
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` but pack dir is `docs/project_management/packs/draft/persist-macos-host-os-install-state/` (hard gate: reconcile to avoid dual-authority docs). (note: missing from spec_manifest.md follow-ups; ensure it stays tracked in later FSE planning) (handoff-only critical) (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L123`)
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/dev-install-helper-discovery/` but pack dir is `docs/project_management/packs/draft/persist-macos-host-os-install-state/` (hard gate: reconcile to avoid dual-authority docs). (note: missing from spec_manifest.md follow-ups; ensure it stays tracked in later FSE planning) (handoff-only critical) (sources: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md#L9`, `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L136`)

## Consolidated FSE pre-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` but pack dir is `docs/project_management/packs/draft/persist-macos-host-os-install-state/` (hard gate: reconcile to avoid dual-authority docs). (note: missing from spec_manifest.md follow-ups; ensure it stays tracked in later FSE planning) (handoff-only critical) (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L123`)
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/dev-install-helper-discovery/` but pack dir is `docs/project_management/packs/draft/persist-macos-host-os-install-state/` (hard gate: reconcile to avoid dual-authority docs). (note: missing from spec_manifest.md follow-ups; ensure it stays tracked in later FSE planning) (handoff-only critical) (sources: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md#L9`, `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L136`)

### Decision Register required
- None detected

### Checkpoint intent follow-ups
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md — pin exact stored values and exact leaf absence semantics when `sw_vers -productVersion`, `sw_vers -buildVersion`, or `uname -m` fails during an otherwise successful hosted install. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L220`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md — state that hosted install and hosted `--no-world` are in scope, and `scripts/substrate/dev-install-substrate.sh` remains out of scope and unchanged. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L221`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md — add explicit no-change assertions for `scripts/substrate/uninstall-substrate.sh` and `scripts/substrate/dev-uninstall-substrate.sh` plus the Linux-only cleanup-state guidance on mac hosts. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L222`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md — pin the temp-file cleanup rule and the hosted dry-run no-write rule for macOS branches. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L223`)
- Lock the exact `host_state.os.*` partial-capture semantics in downstream surface-lock planning. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L181`)
- Lock the exact timestamp rewrite and rebuild rules in downstream surface-lock planning. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L182`)
- Lock the final assertion split for the shared smoke harness and the macOS fixture harness in downstream validation planning. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L183`)

