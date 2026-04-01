## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- Gate 1 — contract truth must land before slice specs are treated as final: (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/workstream_triage.md#L141`)
- Gate 2 — JSON contract must land before DHO1 AC IDs and tests: (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/workstream_triage.md#L144`)
- Gate 3 — task graph must follow checkpoint plan: (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/workstream_triage.md#L146`)

### Decision Register required
- DR-0001 — Resolve ADR-0037’s contradiction: provenance-based attribution vs heuristic attribution; selection MUST preserve “attribution matches effective winner” invariant. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md#L178`)
- DR-0001 (decision_register): resolve ADR-0037’s Option A (provenance) vs Option B (heuristic) contradiction. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/minimal_spec_draft.md#L85`)
- DR-0002 — Lock the JSON contract: field placement for health JSON, enum value set (including whether `default` is real/emittable), and redaction rules for `path_display`/`env`/`flag`. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md#L179`)
- DR-0002 + schema spec: lock the JSON contract for doctor + health. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/minimal_spec_draft.md#L88`)

### CI/checkpoint wiring gaps
- Ensure slice ids in `tasks.json` match `DHO0`, `DHO1` (or update this plan to match the accepted ids). (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md#L90`)
- Add triad tasks for each slice (`DHO*-code`, `DHO*-test`, `DHO*-integ-core`, `DHO*-integ`) and their kickoff prompts. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md#L91`)
- Add ops checkpoint tasks `CP1-ci-checkpoint` and `CP2-ci-checkpoint`: (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md#L92`)
- Gate wiring: (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md#L95`)
- Keep `tasks.json` `meta.checkpoint_boundaries` equal to the checkpoint boundaries in this plan (currently `["DHO0","DHO1"]`). (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md#L97`)
- Then run (must pass): (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md#L98`)

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json — populate triad tasks (`DHO0-*`, `DHO1-*`), deps, kickoff prompt paths, and checkpoint boundaries. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md#L181`)
- docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md — define cross-platform checkpoint groups aligned to `tasks.json`. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md#L182`)
- docs/project_management/packs/sequencing.json — add the sequencing entry referenced by ADR-0037. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md#L183`)
- docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md — reconcile Recommendation vs Decision Summary and update Related Docs links once pack artifacts exist. (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md#L184`)
- Integration boundary check (schema/copy): verify collision-free compatibility with queued packs listed in `pre-planning/impact_map.md` (world-disabled-diagnostics, json-mode, provisioning packs). (sources: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/minimal_spec_draft.md#L94`)

