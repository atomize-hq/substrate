# warn-config-global-show-workspace-overrides — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`
- Spec manifest:
  - `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/spec_manifest.md`

## Touch set (explicit)

### Create
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/spec_manifest.md` — required spec ownership map (planning v4)
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/impact_map.md` — impact map (planning v4)
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/ci_checkpoint_plan.md` — bounded CI cadence for cross-platform automation packs
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/kickoff_prompts/CP1-ci-checkpoint.md` — checkpoint kickoff prompt
- `crates/shell/tests/config_global_show_workspace_override_note.rs` — focused integration test for stderr note + stdout invariants

### Edit
- `crates/shell/src/execution/config_cmd.rs` — emit stderr note for config global show when workspace override is active
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/tasks.json` — schema v4 + checkpoint wiring
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/plan.md` — update artifact index and v4 references
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/manual_testing_playbook.md` — ensure smoke linkage and explicit cases
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/` — keep smoke aligned with manual playbook and contract invariants
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/execution_preflight_report.md` — update required artifact list and checkpoint references

### Deprecate
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/integration_map.md` — replaced by impact_map.md

### Delete
- None

## Cascading implications (behavior/UX)

### CLI / UX
- Change: `substrate config global show` emits a single stderr note when workspace overrides are active.
  - Direct impact: operators get an immediate hint that the shown global patch is not the effective config for the current directory.
  - Cascading impact: operator workflows route to `substrate config show --explain` for effective config inspection.
  - Contradiction risks: tooling that asserts stderr is empty for `config global show` inside a workspace changes behavior; stdout and exit codes remain stable.

### Config / env vars / paths
- Change: note includes `<workspace_root>/.substrate/workspace.yaml` in platform-native display form.
  - Direct impact: the note points at the precise workspace config file location.
  - Cascading impact: documentation and tests must accept platform-native path formatting.
  - Contradiction risks: workspace config parse failures trigger the note without failing the command; this is intentional and is pinned by `contract.md`.

### Policy / isolation / security posture
- Change: none.
  - Direct impact: none.
  - Cascading impact: none.
  - Contradiction risks: none.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - Overlap surfaces: workspace enablement + workspace config file location.
  - Conflict: no.
  - Resolution (explicit): `contract.md` uses the workspace marker path `<workspace_root>/.substrate/workspace.yaml` and defers enablement semantics to `workspace::find_workspace_root`.
- ADR: `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md`
  - Overlap surfaces: config precedence explanation and operator expectations.
  - Conflict: no.
  - Resolution (explicit): this feature emits a note routing operators to the effective config view and does not change precedence.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/`
  - Overlap surfaces: none (this is the pack for the feature).
  - Conflict: no.
  - Resolution (explicit): N/A.

## Follow-ups (explicit)

- Decision Register entries required:
  - None.
- Spec updates required (if any):
  - None.
