# Remediation Log - World Disabled Diagnostics

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: post_exec
source_gate: closeout
related_seam: SEAM-4
related_slice: S3
related_thread: THR-05
related_contract: C-05
related_artifact: scripts/windows/wsl-smoke.ps1
severity: blocking
status: open
owner_seam: SEAM-4
blocked_targets:
  - seam: SEAM-4
    field: promotion_readiness
    value: ready
summary: Native Windows execution for the SEAM-4 disabled-diagnostics smoke path was not available in this environment, so the cross-platform closeout cannot be marked promotion-ready yet.
required_fix: Run the Windows disabled-diagnostics smoke path on native Windows, capture the observed proof bundle for `scripts/windows/wsl-smoke.ps1`, and refresh `governance/seam-4-closeout.md` with that evidence before promotion readiness is published.
resolution_evidence: []
```

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked transition also applies.

## Resolved remediations

- None.
