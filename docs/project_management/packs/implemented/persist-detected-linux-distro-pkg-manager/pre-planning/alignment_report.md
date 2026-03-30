## Misalignment / follow-ups (wrapper-detected)
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/stashing-ferret/` but pack dir is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json#L1`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/spec-manifest/handoff.md#L26`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L73`)

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/stashing-ferret/` but pack dir is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json#L1`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/spec-manifest/handoff.md#L26`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L73`)

### Decision Register required
- None detected

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md — trace the accepted hosted-plus-dev Linux write matrix, `--dry-run` no-write rule, idempotency rule, and temp-file replacement invariant from `contract.md` (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L237`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md — trace smoke assertions for no-event writes, missing `/etc/os-release`, and additive compatibility against `install-state-schema-spec.md` (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L238`)
- docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md — fix feature-directory drift, related-doc path drift, canonical install-state path wording, and hosted-plus-dev installer scope wording (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L241`)
- docs/INSTALLATION.md — reconcile installer-scope language, `schema_version` field name, effective metadata path wording, and the accepted `host_state.platform.*` field set (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L242`)
- scripts/substrate/uninstall-substrate.sh — review HOME-vs-prefix path handling as a separate follow-up outside the selected touch set (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L243`)
- ADR feature-path drift exists (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L305`)
- Operator wording still needs to reconcile to the accepted canonical path rule (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L309`)
- Installer-scope wording still needs to reconcile to the accepted shared-producer contract (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L313`)
- Slice acceptance still needs to trace the accepted write matrix and atomic update rule (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L317`)
- Operator-doc schema naming drift already exists (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L321`)
- Hosted uninstaller path mismatch remains an explicit follow-up boundary (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L325`)
- Fix ADR-0032 feature-dir and related-doc drift from `draft/stashing-ferret` to `draft/persist-detected-linux-distro-pkg-manager`. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L350`)
- Pin the exact write/no-write matrix for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L351`)
- Pin the exact temp-file and replace rule for `install_state.json` updates. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L352`)
- Reconcile `docs/INSTALLATION.md` wording for `schema_version`, effective metadata path, and shared hosted/dev installer scope. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L353`)
- Keep hosted uninstaller HOME-vs-prefix alignment as a separate follow-up unless full planning explicitly expands scope. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L354`)

