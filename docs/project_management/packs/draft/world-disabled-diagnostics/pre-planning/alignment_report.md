## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- DR-0001 — Decide JSON field paths + enum spellings for world/world-deps status fields (avoid collisions with existing `world_doctor` status vocabulary). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L206`)
- DR-0002 — Decide legacy `error`/`ok` field behavior when disabled/skipped (strictly additive; no ambiguous “ok=false” that reads as failure when disabled). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L207`)
- DR-0003 — Decide deterministic copy constraints (stable phrases/substrings) for disabled/skipped messaging across `substrate health` and `substrate shim doctor`. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L208`)

### CI/checkpoint wiring gaps
- None

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json — populate schema v4 triad tasks + kickoff prompt paths; add `meta.checkpoint_boundaries`. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L210`)
- docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md — define checkpoint groups covering `WDD0..WDD2`; ensure alignment with `tasks.json`. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L211`)
- docs/project_management/packs/sequencing.json — add the WDD sequencing entry referenced by ADR-0036 and `plan.md`. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L212`)
- docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md — inventory and lock the full existing JSON shapes for both commands before specifying additive status fields. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L213`)
- docs/USAGE.md — document the disabled/skipped states and the preferred machine-detectable status fields (keep `summary.world_ok` semantics coherent when disabled). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L214`)
- Decision Register (required; blocks implementation): (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L105`)
- Additive-schema grounding (required): (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L110`)
- Exit semantics confirmation (required): (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L113`)
- Pre-planning doc integrity (required): (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L120`)

