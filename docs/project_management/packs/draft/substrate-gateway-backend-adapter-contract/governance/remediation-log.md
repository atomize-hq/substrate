# Remediation Log - Substrate gateway backend adapter contract

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-02
related_artifact: docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/workstream_triage.md
severity: blocking
status: open
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: status
    value: exec-ready
summary: the exact owner line for any additive adapter-visible `status --json` subset is unresolved, so selection-boundary planning cannot publish a stable contract handoff yet.
required_fix: assign the exact owning document and field-family boundary for adapter-visible gateway status data before `SEAM-1` publishes `C-02`.
resolution_evidence: []
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-2
related_slice: null
related_thread: THR-02
related_contract: C-04
related_artifact: docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md
severity: blocking
status: open
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: status
    value: exec-ready
summary: the adopted Universal Agent API subset is unresolved for capability ids, extension keys, session-handle facets, and bounded adapter error detail.
required_fix: pin the exact adopted Universal Agent API subset in the seam-local protocol/schema contract-definition bundle before `SEAM-2` can become `exec-ready`.
resolution_evidence: []
```

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-2
related_slice: null
related_thread: THR-02
related_contract: C-03
related_artifact: docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/alignment_report.md
severity: blocking
status: open
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: status
    value: exec-ready
summary: the local-to-external owner line for adapter translation versus ADR-0017 and ADR-0028 remains unresolved.
required_fix: record the exact handoff boundary between local adapter translation and the externally owned event-envelope and trace semantics before `SEAM-2` publishes `C-03`.
resolution_evidence: []
```

```yaml
remediation_id: REM-004
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-3
related_slice: null
related_thread: null
related_contract: null
related_artifact: docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md
severity: blocking
status: open
owner_seam: SEAM-3
blocked_targets:
  - seam: SEAM-3
    field: status
    value: exec-ready
summary: parity and compatibility proof cannot finalize until ADR-0040 alignment is explicitly confirmed as evidence-only or promoted into the downstream touch set.
required_fix: decide whether ADR-0040 planning-pack docs remain evidence-only or require direct alignment edits, then record that decision in the seam-local parity and compatibility review.
resolution_evidence: []
```

```yaml
remediation_id: REM-005
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: null
related_contract: null
related_artifact: docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md
severity: material
status: open
owner_seam: SEAM-1
blocked_targets: []
summary: ADR-0041 still points at `packs/active/llm_and_agent_config_policy_surface/*` even though this checkout uses `packs/implemented/...`, which creates authority-path drift during downstream planning.
required_fix: replace the stale `packs/active/...` references with the current `packs/implemented/...` paths or record an explicit evidence-only reason for leaving the links unchanged.
resolution_evidence: []
```

## Resolved remediations

- None.
