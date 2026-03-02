## Misalignment / follow-ups (wrapper-detected)
- None detected

## Consolidated full-planning follow-ups (wrapper-compiled)
### Gates / hard decisions
- None

### Decision Register required
- DR-0001 — Shared helper reuse vs replay-local duplication (ADR-0038 selection is A; record rationale + test strategy). (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L177`)
- DR-0002 — Replay trace contract: emit explicit `world_disable_reason` / `world_disable_source` on `replay_strategy` vs overloading `origin_reason(_code)` (must remain additive with explicit absence semantics). (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L178`)

### CI/checkpoint wiring gaps
- Confirm slice ids and ordering (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md#L81`)
- Confirm `tasks.json` checkpoint boundary metadata (schema v4 cross-platform) (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md#L85`)
- Add checkpoint task + kickoff prompt + deps (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md#L89`)
- If additional checkpoints are added later, wire gating so the next checkpoint group’s first slice code/test tasks depend on the prior checkpoint task. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/ci_checkpoint_plan.md#L95`)

### Risks + unknowns
- None

### Other follow-ups
- docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json — populate WDRA0 triad tasks (code/test/integ) with acceptance criteria IDs from `slices/WDRA0/WDRA0-spec.md`. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L180`)
- docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md — lock replay stderr line templates, gating/absence semantics, and redaction rules; link ADR-0037 strings without redefining them. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L181`)
- docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md — lock `replay_strategy` field placement and redaction rules; define additivity + absence semantics. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L182`)
- docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md — define AC matrix (CLI/env/workspace/global winners + redaction + “no behavior change” assertions) and the boundary between world-disablement vs replay-only opt-out. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L183`)
- docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md — reconcile precedence text with ADR-0037 + env contract (workspace overrides env); update Related Docs paths to point at the canonical pre-planning artifacts. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L184`)
- docs/project_management/packs/sequencing.json — add the sequencing entry and gate this work on ADR-0037 integration landing first. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md#L185`)
- Reconcile ADR-0038 precedence text with ADR-0037 + `docs/reference/env/contract.md` and update ADR-0038 so it does not contradict “workspace overrides env override”. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md#L77`)
- Define the exact condition under which replay is “host due to world-disablement” vs: (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md#L78`)
- Decision Register DR-0002: choose the replay trace contract strategy (explicit `world_disable_reason` / `world_disable_source` on `replay_strategy` vs overloading existing fields) and lock one option in `telemetry-spec.md` with explicit absence semantics. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md#L82`)
- Inventory the current replay stderr surfaces and exact string templates in `crates/shell/src/execution/routing/replay.rs` to pin the minimal substitution points and ensure redaction invariants. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md#L83`)
- Confirm replay verbose gating contract (flag/env precedence and absence semantics) in `docs/reference/env/contract.md` and reflect it consistently in `contract.md` and the slice spec. (sources: `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md#L84`)

