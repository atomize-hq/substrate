---
seam_id: SEAM-3
review_phase: pre_exec
execution_horizon: active
basis_ref: seam.md#basis
---
# Review Bundle - SEAM-3 Cross-platform proof + drift guards

This artifact feeds `gates.pre_exec.review`.
`../../review_surfaces.md` is pack orientation only.

## Falsification questions

- Can the smoke wrappers or manual playbook overclaim macOS provisioning parity beyond the staged helper bundle and managed-cleanup scope published by upstream closeouts?
- Can the conformance surfaces drift away from the landed `C-01`..`C-04` contracts so checkpoint proof passes while helper discovery or protected-path refusal behavior has already changed?
- Can Windows compile-parity-only validation be reported as behavior support instead of an explicit compile-only posture?

## R1 - Upstream truth to conformance surface flow

```mermaid
flowchart LR
  S1["SEAM-1 closeout / THR-01 + THR-02"] --> Review["SEAM-3 review + planning"]
  S2["SEAM-2 closeout / THR-03"] --> Review
  Review --> Playbook["manual_testing_playbook.md"]
  Review --> Smoke["smoke/linux-smoke.sh + smoke/macos-smoke.sh + smoke/windows-smoke.ps1"]
  Review --> Checkpoint["checkpoint evidence surfaces"]
```

## R2 - Claim-boundary guard

```mermaid
flowchart TB
  Contracts["C-01 / C-02 / C-03 / C-04"] --> Claims{"Platform claim valid?"}
  Claims -->|"Linux + macOS behavior"| Smoke["Smoke + manual evidence"]
  Claims -->|"Windows compile only"| Windows["Explicit compile-parity wording"]
  Claims -->|"No"| Refuse["Open remediation / block exec-ready"]
```

## Likely mismatch hotspots

- macOS validation text can still overclaim provisioning parity if helper discovery correctness is mistaken for full release-root support
- checkpoint wording can drift from the landed upstream contract boundaries and silently stop testing protected-path refusal or helper-order behavior
- Windows evidence can drift from compile-parity-only wording into implied feature support

## Pre-exec findings

- `REM-002` remains open and must be handled explicitly during seam-local review so playbook, parity, and smoke wording make the macOS support boundary explicit against the landed upstream contracts.
- Upstream basis freshness is now good enough for active seam-local planning: `THR-01`, `THR-02`, and `THR-03` all have closeout-backed truth.

## Pre-exec gate disposition

- **Review gate**: pending
- **Contract gate**: pending
- **Revalidation gate**: pending
- **Opened remediations**:
  - `REM-002`

## Planned seam-exit gate focus

- What must be true before downstream closeout is legal:
  - smoke, playbook, and checkpoint surfaces align to landed `C-01`..`C-04`
  - macOS wording stays within helper discovery, validation, and managed-cleanup scope
  - Windows remains compile parity only
- Which outbound threads matter most:
  - `THR-01`
  - `THR-02`
  - `THR-03`
- Which review-surface deltas would force downstream revalidation:
  - helper-order wording drift
  - protected-path refusal wording drift
  - checkpoint boundary drift
  - platform claim drift
