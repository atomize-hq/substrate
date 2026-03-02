## Misalignment / follow-ups (wrapper-detected)
- Cross-pack contract authority conflict: `docs/project_management/packs/draft/world-deps-apt-provisioning/` selects Option B but `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/` selects Option A (hard decision: converge on one authoritative contract doc). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L200`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md#L220`)

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- Cross-pack contract authority conflict: `docs/project_management/packs/draft/world-deps-apt-provisioning/` selects Option B but `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/` selects Option A (hard decision: converge on one authoritative contract doc). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L200`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md#L220`)

### Decision Register required
- DR-0001 — Confirm provisioning entrypoint selection (record ADR-0030 Option A vs B selection as the single decision outcome). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L219`)
- DR-0002 — Provisioned-state tracking strategy (probe-only vs persisted state file; if file: path + schema + ownership). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L220`)
- DR-0003 — Provisioning execution profile isolation model (how the provisioning request profile is selected, what it relaxes, and what guard rails prevent misuse). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L221`)

### CI/checkpoint wiring gaps
- Add slice triads to `tasks.json` (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md#L81`)
- Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform) (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md#L84`)
- Add checkpoint task + kickoff prompt + deps (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md#L87`)

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md — remove/replace contradictory runtime APT “apply apt first” semantics; defer to this feature’s contract for APT provisioning/runtime prohibition. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L223`)
- docs/reference/world/deps/README.md — update APT section to the new provisioning-time workflow and align runtime sync/install guidance. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L224`)
- docs/internals/world/deps.md — update internal flow notes (APT execution no longer occurs in runtime sync/install) and point to the provisioning-time model. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L225`)
- docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md — if cross-pack contract ownership boundaries change (e.g., non-APT packs attempting to own shared CLI surfaces), update ownership to preserve “exactly one authoritative doc per surface”. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L226`)
- Define runtime fail-early scope (single rule; testable): (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L72`)
- Define provisioning APT invocation contract (single deterministic contract): (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L75`)
- Define `--verbose` output invariants: (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L78`)
- Decide provisioned-state tracking strategy (DR-0002): (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L81`)
- Make Linux host-native “unsupported by default” deterministic: (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L84`)
- Make Windows posture deterministic: (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L87`)
- Enumerate operator-doc update targets by exact path: (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L90`)
- Reconcile ADR-0030 “Related Docs” path drift for world-deps schema contract: (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L93`)
- Define provisioning execution profile isolation model (DR-0003): (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L96`)
- If adopting `WDAP2..WDAP5`, update `spec_manifest.md`, `impact_map.md`, and `ci_checkpoint_plan.md` to include the additional slice specs and any new validation gates. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/workstream_triage.md#L107`)
- Add explicit coordination note for ADR-0036 overlap (`crates/shell/src/builtins/health.rs`) so diagnostics never suggest contradictory “next steps”. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/workstream_triage.md#L108`)

