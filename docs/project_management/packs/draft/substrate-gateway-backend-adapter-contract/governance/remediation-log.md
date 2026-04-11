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
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: v1 gateway status publication remains limited to the existing `status` plus `client_wiring.*` schema, and any future additive adapter-visible field family must be introduced by the status-schema owner before runtime models widen.
required_fix: keep `docs/contracts/substrate-gateway-status-schema.md` as the owner for the machine-readable status field list and require an explicit schema-owner update before any additive adapter-visible status field family ships.
resolution_evidence:
  - docs/contracts/substrate-gateway-status-schema.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-1-adapter-selection-boundary/slice-00-c-01-c-02-contract-definition.md
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
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: ADR-0041 now points at the implemented `packs/implemented/llm_and_agent_config_policy_surface/*` paths, resolving the authority-path drift that previously confused downstream planning.
required_fix: replaced the stale `packs/active/...` references with the current `packs/implemented/...` paths and recorded the exit-gate evidence.
resolution_evidence:
  - docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/governance/seam-1-closeout.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threading.md
```

```yaml
remediation_id: REM-006
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: S00
related_thread: THR-01
related_contract: C-01
related_artifact: docs/contracts/substrate-gateway-backend-adapter-selection.md
severity: blocking
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: the canonical backend-selection baseline now exists and the seam-local execution checklist points downstream work at that durable contract instead of at planning-pack-only prose.
required_fix: keep `docs/contracts/substrate-gateway-backend-adapter-selection.md` aligned with the pack-local `contract.md` and `policy-spec.md` as the implementation checklist executes.
resolution_evidence:
  - docs/contracts/substrate-gateway-backend-adapter-selection.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-1-adapter-selection-boundary/slice-00-c-01-c-02-contract-definition.md
```

## Resolved remediations

- `REM-001`
- `REM-005`
- `REM-006`
