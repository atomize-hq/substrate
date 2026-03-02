## Misalignment / follow-ups (wrapper-detected)
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/stashing-ferret/` but pack dir is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json#L1`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/spec-manifest/handoff.md#L28`)

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- ADR feature-dir drift: ADR declares `docs/project_management/packs/draft/stashing-ferret/` but pack dir is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` (hard gate: reconcile to avoid dual-authority docs). (sources: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md#L9`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json#L1`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/spec-manifest/handoff.md#L28`)

### Decision Register required
- DR-0001 — Reconcile “install_state.json must exist after successful Linux install” with best-effort write posture (writer dependencies, fallback strategy, required warning output). (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L168`)
- DR-0002 — Define `pkg_manager.source` authoritative vocabulary for persistence (explicitly defer to ADR-0031 contract vs define locally). (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L169`)
- DR-0003 — Scope: whether `scripts/substrate/dev-install-substrate.sh` is in-scope for persisting the new `host_state.platform.*` keys (A: prod installer only; B: prod + dev installers). (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L170`)
- DR-0004 — Overwrite policy on re-run: preserve existing `host_state.platform.*` vs overwrite with newly detected values (and what happens when inputs are missing on subsequent runs). (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L171`)

### CI/checkpoint wiring gaps
- Confirm slice ids (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md#L68`)
- Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform) (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md#L72`)
- Add checkpoint task + kickoff prompt + deps (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md#L75`)
- If additional checkpoints are added later, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md#L81`)

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md — update “Installer entrypoint in scope” row if dev-install is selected as in-scope; add explicit dependency link to the upstream detection pack path if required. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L173`)
- docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md — reconcile feature-dir link drift (`stashing-ferret` vs `persist-detected-linux-distro-pkg-manager`) and dependency naming drift (`detecting_badger` vs the actual upstream pack directory). (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md#L174`)
- Decision Register entries (from `impact_map.md`) (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L90`)
- `$SUBSTRATE_HOME` resolution (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L96`)
- “Successful install” definition for the file-exists guarantee (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L99`)
- Absence semantics (schema + persistence) (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L103`)
- Documentation alignment (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L109`)
- Naming/path drift cleanup (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md#L112`)
- Populate `tasks.json` with slice triads + CP1 wiring (per `ci_checkpoint_plan.md` Follow-ups), then validate mechanically: (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L146`)
- ADR link drift follow-up (do not fix in this pack): ADR-0032 references `stashing-ferret` feature dir; reconcile to `persist-detected-linux-distro-pkg-manager` during planning. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L148`)
- Dependency naming follow-up: ADR intake references `detecting_badger`; reconcile to the upstream pack path `best-effort-distro-package-manager` and encode sequencing in `docs/project_management/packs/sequencing.json`. (sources: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md#L149`)

