# Remediation Log - gateway-backend-selection-runtime-integration

This pack is extracted at seam-brief depth, but ADR-0046 and the pre-planning pack leave several cross-seam decisions unresolved. They are recorded here as explicit open remediations so downstream planning does not silently normalize them into contract truth.

Future remediation entries must use the canonical fields from the extractor governance model:

- `remediation_id`
- `origin_phase`
- `source_gate`
- `related_seam`
- `related_slice`
- `related_thread`
- `related_contract`
- `related_artifact`
- `severity`
- `status`
- `owner_seam`
- `blocked_targets`
- `summary`
- `required_fix`
- `resolution_evidence`

## Open remediations

```yaml
- remediation_id: REM-001
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-1
  related_slice: S00
  related_thread: THR-01
  related_contract: C-02
  related_artifact: docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/env-vars-spec.md
  severity: blocking
  status: open
  owner_seam: SEAM-1
  blocked_targets:
    - seam: SEAM-1
      field: status
      value: exec-ready
  summary: auth precedence between env material and host credential files is not yet published in the feature-local ADR-0046 delta docs as integrated lifecycle contract truth
  required_fix: publish one explicit precedence rule in the feature-local ADR-0046 docs for authorized env material versus authorized host credential file reads and align shell/runtime consumers to it
  resolution_evidence: []
```

```yaml
- remediation_id: REM-002
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-1
  related_slice: S00
  related_thread: THR-01
  related_contract: C-01
  related_artifact: docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/policy-spec.md
  severity: blocking
  status: open
  owner_seam: SEAM-1
  blocked_targets:
    - seam: SEAM-1
      field: status
      value: exec-ready
  summary: backend inventory roots and filename rules are not yet fixed as integrated realization contract truth
  required_fix: publish one explicit inventory-root and filename/id rule set in the feature-local ADR-0046 docs before downstream runtime planning relies on filesystem semantics
  resolution_evidence: []
```

```yaml
- remediation_id: REM-003
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-2
  related_slice: S00
  related_thread: THR-02
  related_contract: C-03
  related_artifact: docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-protocol-spec.md
  severity: blocking
  status: open
  owner_seam: SEAM-2
  blocked_targets:
    - seam: SEAM-2
      field: status
      value: exec-ready
  summary: missing integrated adapter binding classification is unresolved for the integrated runtime protocol
  required_fix: publish one explicit classification in the feature-local ADR-0046 docs for missing or unsupported integrated adapter bindings and align runtime failure handling to it
  resolution_evidence: []
```

```yaml
- remediation_id: REM-004
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-2
  related_slice: S00
  related_thread: THR-02
  related_contract: C-04
  related_artifact: docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/gateway-runtime-adapter-schema-spec.md
  severity: blocking
  status: open
  owner_seam: SEAM-2
  blocked_targets:
    - seam: SEAM-2
      field: status
      value: exec-ready
  summary: missing auth handoff material classification is unresolved after policy permits the read path
  required_fix: publish one explicit classification in the feature-local ADR-0046 docs for missing integrated auth handoff material and align shell and world-agent handling to it
  resolution_evidence: []
```

```yaml
- remediation_id: REM-005
  origin_phase: pre_exec
  source_gate: revalidation
  related_seam: SEAM-3
  related_slice: S00
  related_thread: THR-03
  related_contract: C-05
  related_artifact: docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/compatibility-spec.md
  severity: blocking
  status: open
  owner_seam: SEAM-3
  blocked_targets:
    - seam: SEAM-3
      field: status
      value: exec-ready
  summary: the first supported non-cli:codex integrated backend id is not yet pinned for parity, compatibility, and rollout proof
  required_fix: name the first supported additional integrated backend id in the feature-local ADR-0046 docs and use it consistently in parity tests, manual validation, smoke scripts, and compatibility evidence
  resolution_evidence: []
```

## Resolved remediations

- None.
