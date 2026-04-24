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
- DR-ITPS-01 — lock tuple-policy telemetry to the existing `identity_tuple` and `placement_posture` field family and reject a parallel trace-only tuple schema. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md#L150`)
- DR-ITPS-02 — lock the operator inspection surface so `substrate policy current show --explain` is the authoritative merged view for `llm.constraints.*`, while `substrate config` remains the config-root inspection surface. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md#L151`)
- docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md — remove the row that assigns tuple-policy visibility to `substrate config show --explain`, and align the manual testing surface with the actual policy-view contract. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md#L153`)
- docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md — pin the exact rendered deny text, exit-code mapping, and explain-surface wording for tuple-axis mismatches. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md#L154`)
- docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md — pin the exact allow and deny field set, deny-reason projection, and redaction rules using the selected tuple field family. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md#L155`)
- docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md — replace stale “new surface” framing, remove config and policy conflation, and replace ambiguous modal language with direct contract text. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md#L156`)
- docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md — remove stale statements that describe tuple-axis policy as unimplemented. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md#L157`)
- Repair the `ADR-0043` lift vector so `pm-lift-intake` emits valid JSON during the next refinement pass. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/workstream_triage.md#L178`)
- Update the checkpoint plan’s machine-readable slice list during full planning to match the accepted slice order. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/workstream_triage.md#L179`)
- Record the accepted slice split in `plan.md` and `tasks.json` exactly as `ITPS0`, `ITPS1`, `ITPS2`, `ITPS3`. (sources: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/workstream_triage.md#L180`)

