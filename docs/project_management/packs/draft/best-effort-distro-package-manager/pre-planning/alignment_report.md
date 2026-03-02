## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- DR-0001 — `/etc/os-release` parsing + canonicalization rules (no `source`; quotes/whitespace/duplicates/case-sensitivity). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L166`)
- DR-0002 — PATH probe ambiguity policy + fixed precedence order (warn vs fail; exact ordering; required warning content). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L167`)
- DR-0003 — Hermetic test seam for fake os-release input (test-only env var/arg vs harness approach; ensure no production safety weakening). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L168`)

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md — pin the exact prereq command set to name in remediation guidance; pin “mapping matched but binary missing” behavior; pin when the one-liner is emitted (including “no prereqs needed” behavior). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L170`)
- docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md — explicitly scope exit-code meanings as Linux-only vs global (to avoid accidental Windows/macOS behavior changes). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L171`)
- docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C2/C2-spec.md — assert exit codes + one-liner content + precedence in the hermetic harness; forbid host mutation. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L172`)
- docs/project_management/adrs/draft/ADR-0031-detecting-badger.md — reconcile “Related Docs” link drift (`detecting-badger/*` vs `best-effort-distro-package-manager/*`) during planning. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L173`)
- Populate `tasks.json` with slice `*-integ` tasks + CP1 wiring, then validate mechanically: (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L118`)
- Re-evaluate `tasks.json` `meta.behavior_platforms_required` (currently includes `linux|macos|windows`) against the “Linux-only behavior change” contract. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L120`)
- Align slice naming across artifacts (spec-manifest references `slices/C0/C1/C2` while minimal spec draft uses `BEDPM0/1/2`): pick one scheme during full planning and update non-authoritative references. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L121`)

