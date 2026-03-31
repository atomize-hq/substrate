## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- DR-0001 — JSON field paths + enum spellings for world/world-deps status (including the health JSON surface). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L204`)
- DR-0002 — Legacy error-field behavior when disabled/skipped applies (must not encode skip purely as an error string). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L205`)
- DR-0003 — Operator-facing copy standardization for disabled/skipped across `substrate health` and `substrate shim doctor` (deterministic + testable). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L206`)

### CI/checkpoint wiring gaps
- Confirm slice ids and ordering (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md#L81`)
- Add `tasks.json` checkpoint boundary metadata (schema v4 cross-platform) (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md#L85`)
- Add checkpoint task + kickoff prompt + deps (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md#L88`)
- If additional checkpoints are added later, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md#L95`)

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md — define checkpoint boundaries and ensure `tasks.json` `meta.checkpoint_boundaries` matches. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L208`)
- docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json — populate `WDD0-code`/`WDD0-test`/`WDD0-integ` with AC references and automation metadata. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L209`)
- docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md — lock additive field placement, enums, emission/absence rules, and examples for disabled/healthy/needs-attention cases. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L210`)
- docs/project_management/packs/draft/world-disabled-diagnostics/contract.md — lock deterministic copy, exit-code mapping, and cross-platform parity statements. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L211`)
- docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md — define the operational “skip probes” boundary and AC matrix (disabled vs enabled-but-broken). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L212`)
- docs/project_management/packs/sequencing.json — add the sequencing entry and dependency edges (at minimum: WDD before ADR-0037 attribution work that touches the same surfaces). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md#L213`)
- Decide DR-0001: final JSON field paths + enum spellings for world/world-deps statuses for both `health --json` and `shim doctor --json`. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L98`)
- Decide DR-0002: deterministic legacy JSON error-field behavior when disabled/skipped applies (omit vs null vs still populated). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L99`)
- Decide DR-0003: deterministic operator-facing copy contract (exact templates, or exact required substrings + ordering rules) for disabled/skipped across both commands. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L100`)
- Define “skip probes” operational boundary (singular + testable): list forbidden operations when disabled and how tests assert “no probes”. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L101`)
- Pin behavior for effective-config resolution errors (invalid YAML, unreadable config) and the corresponding exit-code mapping for `health`/`shim doctor`. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L102`)
- Verify current exit-code behavior for “needs attention” vs hard failures and reconcile ADR-0036 assumptions in `contract.md`. (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L103`)
- Update operator docs (`docs/USAGE.md`) to match shipped disabled/skipped semantics and remediation hints (impact map touch set). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L104`)
- Add sequencing entry and dependency edges for WDD in `docs/project_management/packs/sequencing.json` (impact map follow-up; at minimum WDD before ADR-0037 attribution work). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/minimal_spec_draft.md#L105`)
- Repo hygiene: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` is referenced as canonical by multiple standards, but does not exist at repo root in this checkout (only found at `docs/project_management/_archived/misc/WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`). (sources: `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/workstream_triage.md#L195`)

