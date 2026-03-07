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
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md — DR-0001: persistence location contract (`install_state.json` vs separate file) (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L235`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md — DR-0002: field naming and nesting under `host_state.platform.*` (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L236`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md — DR-0003: vocabulary ownership (`best-effort-distro-package-manager` contract vs local duplication) (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L237`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md — DR-0004: write-trigger scope across hosted install, hosted `--no-world`, dev install, and dev `--no-world` (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L238`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md — pin installer-scope selection, Linux-only guarantees, prefix-to-`$SUBSTRATE_HOME` equivalence, and no-world/dry-run/write-failure rules (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L241`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md — pin `schema_version` field name, `host_state.platform.*` schema, merge rules with `host_state.group` and `host_state.linger`, and JSON examples (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L242`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md — pin exact write/no-write branches and the temp-file replacement rule (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L243`)
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md — pin smoke assertions for no-event writes, missing `/etc/os-release`, and additive compatibility (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L244`)
- docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md — fix feature-directory drift and related-doc path drift (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L245`)
- docs/INSTALLATION.md — reconcile installer-scope language, `schema_version` field name, and effective metadata path wording (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L246`)
- scripts/substrate/uninstall-substrate.sh — review HOME-vs-prefix path handling as a separate follow-up outside the selected touch set (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L247`)
- ADR feature-path drift exists (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L297`)
- Prefix naming and `$SUBSTRATE_HOME` naming need one canonical rule (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L301`)
- Installer scope is not pinned (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L305`)
- Successful-install branches need exact write and no-write semantics (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L309`)
- Operator-doc schema naming drift already exists (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L313`)
- Uninstaller path compatibility needs explicit review (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md#L317`)
- Fix ADR-0032 feature-dir and related-doc drift from `draft/stashing-ferret` to `draft/persist-detected-linux-distro-pkg-manager`. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L350`)
- Pin the exact write/no-write matrix for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L351`)
- Pin the exact temp-file and replace rule for `install_state.json` updates. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L352`)
- Reconcile `docs/INSTALLATION.md` wording for `schema_version`, effective metadata path, and shared hosted/dev installer scope. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L353`)
- Keep hosted uninstaller HOME-vs-prefix alignment as a separate follow-up unless full planning explicitly expands scope. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L354`)

