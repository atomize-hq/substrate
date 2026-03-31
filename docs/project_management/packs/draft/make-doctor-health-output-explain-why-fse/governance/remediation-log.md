# Remediation Log - make-doctor-health-output-explain-why

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: post_exec
source_gate: closeout
related_seam: SEAM-1
related_slice: S4
related_thread: THR-02
related_contract: C-01
related_artifact: crates/shell/tests/doctor_scopes_ds0.rs
severity: blocking
status: open
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: promotion_readiness
    value: ready
summary: macOS and Windows doctor text parity proof was not executed in the current Linux environment, so the SEAM-1 exit-gate publication remains blocked from promotion readiness.
required_fix: Execute the macOS and Windows doctor text parity proof surfaces, capture the observed output for the disabled-attribution text contract, and refresh `governance/seam-1-closeout.md` with that evidence before promotion readiness can be published.
resolution_evidence: []
```

## Resolved remediations

- None.
