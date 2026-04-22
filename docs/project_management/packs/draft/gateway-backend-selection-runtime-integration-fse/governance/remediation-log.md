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
  related_artifact: docs/contracts/substrate-gateway-policy-evaluation.md
  severity: blocking
  status: open
  owner_seam: SEAM-1
  blocked_targets:
    - seam: SEAM-1
      field: seam_exit_gate.status
      value: passed
  summary: the canonical policy-evaluation contract now pins env-primary auth precedence for integrated lifecycle truth; the remaining open work is landing aligned ADR-0046 policy/env-var docs and consumer adoption behind that published rule
  required_fix: keep the published env-primary, file-fallback-only, no-mixed-source rule aligned in supporting ADR-0046 policy/env-var docs plus shell and runtime consumers before `SEAM-1` closes and publishes `THR-01`
  resolution_evidence:
    - docs/contracts/substrate-gateway-policy-evaluation.md now states that complete allowlisted env auth material is primary, host credential files are fallback-only when env auth is absent, and partial env auth fails closed as invalid integration
    - threaded-seams/seam-1-backend-selection-and-policy-surface/slice-00-c-01-c-02-contract-definition.md now records the owner execution checklist and verification plan for the remaining landing work
```

```yaml
- remediation_id: REM-002
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-1
  related_slice: S00
  related_thread: THR-01
  related_contract: C-01
  related_artifact: docs/contracts/substrate-gateway-backend-adapter-selection.md
  severity: blocking
  status: open
  owner_seam: SEAM-1
  blocked_targets:
    - seam: SEAM-1
      field: status
      value: exec-ready
  summary: backend inventory roots and filename rules are not yet fixed as integrated realization contract truth
  required_fix: clarify one explicit inventory-root and filename/id rule set in `docs/contracts/substrate-gateway-backend-adapter-selection.md` first, then align supporting ADR-0046 policy and filesystem docs before downstream runtime planning relies on it
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
  related_artifact: docs/contracts/substrate-gateway-backend-adapter-protocol.md
  severity: blocking
  status: open
  owner_seam: SEAM-2
  blocked_targets:
    - seam: SEAM-2
      field: status
      value: exec-ready
  summary: missing integrated adapter binding classification is unresolved for the integrated runtime protocol
  required_fix: clarify one explicit classification in `docs/contracts/substrate-gateway-backend-adapter-protocol.md` first, then align supporting ADR-0046 protocol docs and runtime failure handling to it
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
  related_artifact: docs/contracts/substrate-gateway-backend-adapter-schema.md
  severity: blocking
  status: open
  owner_seam: SEAM-2
  blocked_targets:
    - seam: SEAM-2
      field: status
      value: exec-ready
  summary: missing auth handoff material classification is unresolved after policy permits the read path
  required_fix: clarify one explicit classification in `docs/contracts/substrate-gateway-backend-adapter-schema.md` first, then align supporting ADR-0046 schema docs plus shell and world-agent handling to it
  resolution_evidence: []
```

```yaml
- remediation_id: REM-006
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-2
  related_slice: S00
  related_thread: THR-02
  related_contract: C-04
  related_artifact: docs/contracts/substrate-gateway-backend-adapter-schema.md
  severity: blocking
  status: open
  owner_seam: SEAM-2
  blocked_targets:
    - seam: SEAM-2
      field: status
      value: exec-ready
  summary: the integrated auth handoff delivery rule into the runtime is still unresolved between env-only, file-only, or one fixed mixed model with explicit precedence
  required_fix: clarify one explicit integrated auth handoff delivery model in `docs/contracts/substrate-gateway-backend-adapter-schema.md` first, then align supporting ADR-0046 protocol/schema docs and runtime wiring to it
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
  related_artifact: docs/contracts/substrate-gateway-integrated-runtime-compatibility.md
  severity: blocking
  status: open
  owner_seam: SEAM-3
  blocked_targets:
    - seam: SEAM-3
      field: status
      value: exec-ready
  summary: the first supported non-cli:codex integrated backend id is not yet pinned for parity, compatibility, and rollout proof
  required_fix: publish the first supported additional integrated backend baseline in `docs/contracts/substrate-gateway-integrated-runtime-compatibility.md` first, then align supporting ADR-0046 compatibility, parity, playbook, and smoke evidence to it
  resolution_evidence: []
```

## Resolved remediations

- None.
