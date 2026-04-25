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
- DR-AHCSITC-01 — lock the canonical CLI namespace on `substrate agent ...` and define the compatibility posture for `substrate agents validate`. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L260`)
- DR-AHCSITC-02 — lock successor implementation placement on existing `crates/shell`, `crates/common`, and `crates/agent-api-*` surfaces for this feature boundary. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L261`)
- DR-AHCSITC-03 — lock nested LLM publication on a separate correlated record instead of mutating the base pure-agent record. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L262`)
- docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md — pin the exact publication path for `world_generation` before execution work changes the envelope or trace flattening. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L266`)
- docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md — tighten any additional new test-file paths if execution work introduces dedicated command or session-protocol tests beyond the existing test files listed in the touch set. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L267`)
- Pin the machine-readable list and status output contract inside `contract.md` during `AHCSITC-PWS-contract`. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md#L299`)
- Pin the exact publication path for `world_generation` inside `telemetry-spec.md` during `AHCSITC-PWS-runtime_fail_early`. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md#L300`)
- Create or land the ADR-0045 successor toolbox pack after ADR-0044 planning closes so toolbox status/env/tool-call surfaces consume the now-locked `substrate agent ...` namespace and existing-crate hub identity/session owner set. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L251`)
- Continue `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/` after ADR-0044 planning closes so the integrated gateway realizes inventory-backed multi-adapter runtime behavior instead of the current `cli:codex` proof path. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L253`)
