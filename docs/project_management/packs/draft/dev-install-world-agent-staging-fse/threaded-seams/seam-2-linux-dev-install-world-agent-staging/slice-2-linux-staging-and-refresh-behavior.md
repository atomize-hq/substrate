---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - THR-01 changes accepted staged path rules or disabled-world ordering
    - selected-profile mapping changes
    - ln -sfn refresh semantics change
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-03
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-03
open_remediations: []
candidate_subslices: []
---
### S2 - Implement Linux staging, refresh, and regression proof

- **User/system value**: Linux dev install with `--no-world` becomes a reliable “stage now, enable later” workflow that satisfies the published runtime contract from `SEAM-1`.
- **Scope (in/out)**:
  - In:
    - stage both accepted `world-agent` links from the selected profile
    - refresh stale links deterministically with `ln -sfn`
    - preserve `world.enabled: false` and skip provisioning/systemd mutation
    - keep installer smoke aligned to the owned staging behavior without broadening production-installer scope
  - Out:
    - runtime preflight or missing-artifact UX changes
    - changes to macOS or Windows behavior
    - production-installer behavior changes beyond regression evidence
- **Acceptance criteria**:
  - debug-profile installs stage both links to `target/debug/world-agent`
  - release-profile installs stage both links to `target/release/world-agent`
  - reruns refresh both links deterministically
  - `world.enabled` remains `false` during `--no-world`
  - no provisioning or systemd mutation occurs during `--no-world`
- **Verification**:
  - targeted installer smoke coverage
  - staged-link evidence showing both paths and selected-profile mapping
  - proof that a staged artifact produced here satisfies the `SEAM-1` runtime preflight contract

#### S2.T1 - Stage both accepted `world-agent` links

- **Outcome**: `dev-install-substrate.sh --no-world` publishes both accepted bridge paths for the selected profile.
- **Implementation notes**:
  - keep the owned touch surface narrow to `scripts/substrate/dev-install-substrate.sh` and installer-smoke evidence
  - preserve the shared runtime path order from `SEAM-1`
- **Acceptance criteria**: both links exist and target the selected profile output

#### S2.T2 - Preserve disabled-world and no-provisioning semantics

- **Outcome**: the staged bridge is refreshed without enabling the world or mutating provisioning/systemd state.
- **Implementation notes**:
  - use the `SEAM-1` closeout-backed `world.enabled` ordering as the invariant
  - keep production installer behavior reference-only
- **Acceptance criteria**: `world.enabled` stays `false`; no provisioning/systemd mutation is introduced

Checklist:
- Implement:
- Test:
- Validate:
- Cleanup:
