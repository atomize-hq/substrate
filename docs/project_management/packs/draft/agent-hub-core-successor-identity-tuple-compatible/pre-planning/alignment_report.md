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
- DR-AHCSITC-01 — lock the canonical CLI namespace on `substrate agent ...` and define the compatibility posture for `substrate agents validate`. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L234`)
- DR-AHCSITC-02 — lock successor implementation placement on existing `crates/shell`, `crates/common`, and `crates/agent-api-*` surfaces for this feature boundary. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L235`)
- DR-AHCSITC-03 — lock nested LLM publication on a separate correlated record instead of mutating the base pure-agent record. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L236`)
- docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md — replace the unresolved `crates/agent-hub` owner wording with the selected existing-crate owner set and note that no new crate lands in this feature boundary. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L239`)
- docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md — align the command namespace, remove ambiguous modal wording, and replace the unresolved `crates/agent-hub` implementation path with the selected existing-crate owner set. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L240`)
- docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md — align toolbox dependency wording with the selected successor implementation placement and the canonical `substrate agent ...` namespace. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L241`)
- docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md — pin the exact publication path for `world_generation` before execution work changes the envelope or trace flattening. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L242`)
- docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md — tighten any additional new test-file paths if execution work introduces dedicated command or session-protocol tests beyond the existing test files listed in the touch set. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md#L243`)
- Repair the ADR-0044 Lift Vector field type for `risk.unknowns_high`, then rerun `pm-lift-intake`. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md#L295`)
- Mirror the existing-crate implementation placement in ADR-0044 before full planning promotes slice specs. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md#L296`)
- Pin the machine-readable list and status output contract inside `contract.md` during `AHCSITC-PWS-contract`. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md#L297`)
- Pin the exact publication path for `world_generation` inside `telemetry-spec.md` during `AHCSITC-PWS-runtime_fail_early`. (sources: `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/workstream_triage.md#L298`)

