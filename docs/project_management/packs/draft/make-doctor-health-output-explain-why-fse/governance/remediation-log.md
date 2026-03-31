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
resolution_evidence:
  - "Resequencing decision: `SEAM-2` may advance to active/decomposition because `governance/seam-1-closeout.md` already publishes landed `C-01`/`C-02` and the current `THR-01`/`THR-02` handoff; this remediation remains scoped to `SEAM-1` promotion readiness only: `README.md`, `seam_map.md`, `threading.md`, `seam-2-json-health-disable-attribution.md`."
  - "`SEAM-2` seam-local decomposition now consumes that published handoff directly and does not treat `REM-001` as an activation blocker: `threaded-seams/seam-2-json-health-disable-attribution/seam.md`, `threaded-seams/seam-2-json-health-disable-attribution/review.md`."
```

## Resolved remediations

- None.
