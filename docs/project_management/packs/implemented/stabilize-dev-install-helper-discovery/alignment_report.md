## Misalignment / follow-ups (wrapper-detected)
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/dev-install-helper-discovery/` but pack dir is `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md#L9`, `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json#L1`)

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/dev-install-helper-discovery/` but pack dir is `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md#L9`, `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json#L1`)

### Decision Register required
- DR-0001 — Decide helper staging mechanism (copy vs symlink) as a user-visible contract. (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L172`)
- DR-0002 — Decide uninstall ownership-guard algorithm (how “managed by dev-install” is determined). (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L173`)
- DR-0003 — Decide overwrite policy when `$SUBSTRATE_HOME/scripts/substrate/*` already exists (including behavior when destination is not dev-managed). (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L174`)

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/adrs/draft/ADR-0034-staging-beaver.md — reconcile feature directory path drift (`.../draft/dev-install-helper-discovery/` vs `.../draft/stabilize-dev-install-helper-discovery/`). (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L176`)
- docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/plan.md — include an explicit mapping from ADR slice labels (`C0`, `C1`) to the pack’s canonical slice IDs (`SDIHD0`, `SDIHD1`). (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L177`)
- docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md — complete DR-0001..DR-0003 with explicit A/B options and a selection (ADR currently enumerates only). (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L178`)
- docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md — explicitly scope macOS “enable later” expectations if additional staging of `${RELEASE_ROOT}/scripts/mac/…` is not part of this ADR. (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L179`)
- docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json — reconcile required Windows behavior evidence with “world enable unsupported on Windows” (define deterministic expectation or mark validation as N/A for this feature). (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L180`)
- docs/COMMANDS.md — update `substrate world enable` flag documentation drift (`--home` vs `--prefix`) to match ADR-0003 + current CLI behavior. (sources: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/impact_map.md#L181`)

