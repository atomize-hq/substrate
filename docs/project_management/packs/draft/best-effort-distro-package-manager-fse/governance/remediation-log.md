# Remediation Log - Best-Effort Distro Package Manager

## Open remediations

No open remediations at seam extraction time. All seams are in `proposed` status, and extraction has not yet identified blocking issues.

Remediation schema for future additions:

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-<n>
related_slice: null
related_thread: THR-<nn>
related_contract: C-<nn>
related_artifact: <repo-relative-path>
severity: blocking | material | follow_up
status: open | in_progress | resolved | accepted_risk | carried_forward
owner_seam: SEAM-<n>
blocked_targets:
  - seam: SEAM-<n>
    field: status | execution_horizon
    value: decomposed | exec-ready | in_flight | landed | closed
summary: <one-sentence machine-readable finding summary>
required_fix: <one-sentence explicit fix>
resolution_evidence: []
```

## Resolved remediations

None yet. Remediations will be moved here once resolved, with `status: resolved` and populated `resolution_evidence`.
