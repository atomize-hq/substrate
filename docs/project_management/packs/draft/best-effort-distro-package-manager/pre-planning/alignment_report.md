## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- DR-0001 — Pin `/etc/os-release` parsing + matching rules (normalization; duplicate keys; case-sensitivity; `ID_LIKE` tokenization). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L191`)
- DR-0002 — Pin deterministic PATH-probe precedence order and multi-manager ambiguity policy (warn vs fail; required warning content elements). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L192`)
- DR-0003 — Pin hermetic-test os-release injection seam (fake input without weakening production safety posture). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L193`)

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md — pin mapping failure semantics (“mapped manager missing” posture), PATH-probe precedence order, and one-liner emission timing (including the “no prereqs needed” case and reuse via `scripts/substrate/world-enable.sh`). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L195`)
- docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md — pin remediation guidance content elements and the exact prerequisite command list included in the “no supported manager” guidance. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L196`)
- docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md — define the hermetic harness contract and assert precedence, one-liner content, warning/error elements, and exit codes. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L197`)
- docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md — include the exact command(s) to run the hermetic detection test and explicitly label the container smoke as optional (if it is not CI gating). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L198`)
- docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json — populate the `BEDPM0` triad tasks and reference the slice spec + contract explicitly. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L199`)
- docs/project_management/adrs/draft/ADR-0031-detecting-badger.md — reconcile Related Docs link drift (paths under `docs/project_management/packs/draft/detecting-badger/` vs `docs/project_management/packs/draft/best-effort-distro-package-manager/`) so downstream planning artifacts are discoverable. (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md#L200`)
- Populate `tasks.json` with slice triad tasks + CP1 wiring, then validate mechanically: (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L259`)
- Reconcile ADR “Related Docs” links (`detecting-badger/*` vs `best-effort-distro-package-manager/*`) as part of full planning (Touch Set already includes editing the ADR). (sources: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/workstream_triage.md#L261`)

