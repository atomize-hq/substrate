## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- DR-0001 — APT requirement derivation (de-dup + ordering + version-pin conflict policy) (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L289`)
- DR-0002 — Provisioned-state tracking (probe-only vs state file) and its impact on runtime fail-early/no-op behavior (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L290`)
- DR-0003 — Provisioning execution isolation model (request `profile` value(s), guard rails, and explicit relationship to `SUBSTRATE_WORLD_REQUEST_PROFILE`) (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L291`)

### CI/checkpoint wiring gaps
- Confirm slice ids and ordering (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md#L113`)
- Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform) (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md#L117`)
- Add checkpoint task(s) + kickoff prompt(s) + deps (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md#L120`)
- Wire gating between checkpoints (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/ci_checkpoint_plan.md#L131`)

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md — update if the Touch Set or operator-doc targets expand beyond the surfaces listed here (single-authority enforcement). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L294`)
- docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md — resolve spec-manifest follow-ups #3–#6 (ordering of enable vs provision; runtime scope rules; dry-run semantics; Windows posture). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L295`)
- docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md — pin deterministic derivation + APT invocation + backend capability gate and produce testable acceptance criteria. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L296`)
- docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md — pin deterministic fail-early triggers + remediation invariants + dry-run behavior and produce testable acceptance criteria. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/impact_map.md#L297`)
- Pin deterministic APT requirement derivation conflict policy (version pins, de-dup, ordering) via `decision_register.md` + `WDAP0-spec.md`. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L85`)
- Pin provisioning-time execution isolation model and host-mutation guard rails, including request `profile` value(s) and relationship to `SUBSTRATE_WORLD_REQUEST_PROFILE`, via `decision_register.md` + `WDAP0-spec.md`. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L86`)
- Pin `world enable --provision-deps` operational scope and ordering relative to baseline `world enable` behavior via `contract.md` + `WDAP0-spec.md`. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L87`)
- Pin runtime fail-early scope rule for `deps current install <ITEM...>` via `contract.md` + `WDAP1-spec.md`. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L88`)
- Pin runtime `--dry-run` and `--verbose` behavior under the fail-early posture via `contract.md` + `WDAP1-spec.md`. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L89`)
- Pin Windows posture for this feature (supported vs unsupported) via `contract.md` + playbooks/smoke. (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L90`)
- Reconcile upstream contract/doc contradictions that currently imply runtime APT mutation (single authoritative truth). (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/minimal_spec_draft.md#L91`)
- APT requirement derivation is underspecified (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L278`)
- Provisioning execution isolation model is implied but not pinned (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L284`)
- `world enable --provision-deps` operational scope is underspecified (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L291`)
- Runtime fail-early “scope” is underspecified for `deps current install` (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L298`)
- Runtime `--dry-run`/`--verbose` behavior under fail-early is underspecified (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L302`)
- Windows provisioning posture is ambiguous (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L306`)
- Operator-doc update targets are not enumerated by exact path/headings (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L312`)
- Cross-document contract ownership conflicts must be reconciled (sources: `docs/project_management/packs/draft/world-deps-apt-provisioning/pre-planning/spec_manifest.md#L316`)

