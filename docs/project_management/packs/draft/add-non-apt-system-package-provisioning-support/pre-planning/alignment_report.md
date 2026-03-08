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
- DR-0004 — pacman runnable-wrapper and present-semantics scope. (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md#L245`)
- docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md — explicitly state whether runnable pacman-backed packages are in scope in v1. (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md#L251`)
- docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md — pin the single-authority handoff from the APT pack and define the exact mixed-manager failure rule. (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md#L252`)
- docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md — lock the exact Windows posture instead of leaving it assumption-only. (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md#L253`)
- Shared contract ownership is still split across existing docs (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L326`)
- ADR-0033 still points at stale spec paths (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L330`)
- Windows posture is still assumption-only (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L334`)
- Mixed-manager behavior is not pinned yet (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L338`)
- Runtime `current install <ITEM...>` scope remains ambiguous (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L342`)
- Probe tie-break behavior is still implied (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L346`)
- Pacman invocation details are not pinned yet (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L350`)
- Built-in inventory strategy is still open (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L354`)
- Validation substrate for real Arch-family success is not enumerated by exact path (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md#L358`)
- If full planning accepts the five-slice model, update `pre-planning/spec_manifest.md` and `pre-planning/ci_checkpoint_plan.md` before writing slice tasks and kickoff prompts. (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/workstream_triage.md#L428`)
- If full planning rejects the five-slice model, record the rejection explicitly and justify how the original three-slice plan will manage the `split_required` lift signal and the 41-file touch set. (sources: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/workstream_triage.md#L429`)

