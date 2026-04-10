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
- DR-0001 — fresh macOS file scaffolding — mirror the selected Option A: fresh macOS files write top-level metadata plus `host_state.os.*` and do not seed empty Linux cleanup containers. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L225`)
- DR-0003 — primary macOS automated validation vehicle — mirror the selected Option A: `tests/mac/installer_parity_fixture.sh` owns macOS execution-path assertions and `tests/installers/install_state_smoke.sh` owns shared JSON and filesystem assertions. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L227`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md — lock the hosted-installer-only scope, the hosted `--no-world` write rule, the hosted `--dry-run` no-write rule, and the Linux-only cleanup boundary. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L230`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md — lock the exact `host_state.os.*` field set, fresh-file shape, merge behavior, and partial-emission rule. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L231`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md — lock temp-file naming, same-directory replace ordering, prior-file preservation, and warning-only failure semantics. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L232`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md — lock macOS hosted and hosted `--no-world` guarantees, Linux no-delta guarantees, and Windows no-delta guarantees. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L233`)
- docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md — lock preservation of unknown keys, `host_state.group`, `host_state.linger`, and `host_state.platform`. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md#L234`)
- Select fresh-file scaffolding behavior for new macOS metadata files. ADR-0039 defines `host_state.os.*` and compatibility preservation but does not select whether a brand-new macOS file also seeds empty legacy `host_state.group` and `host_state.linger` containers. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md#L360`)
- Select partial-emission behavior for `host_state.os`. ADR-0039 names best-effort collection and future-consumer tolerance but does not select whether the writer omits only failed child fields or skips the entire `host_state.os` block when any required source command fails. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md#L361`)
- Select the primary macOS automated validation harness. ADR-0039 requires smoke coverage and manual validation but does not select the canonical harness path for the new assertions. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md#L362`)
- Align the next full-planning pass to the accepted 4-slice order before kickoff prompts and triad tasks land. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L303`)
- Update the checkpoint boundary references to `PMHOIS3` when full planning rewires `tasks.json`. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L304`)
- Record the intake-lift gap as backlog debt on ADR-0039 or its intake doc. (sources: `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md#L305`)

