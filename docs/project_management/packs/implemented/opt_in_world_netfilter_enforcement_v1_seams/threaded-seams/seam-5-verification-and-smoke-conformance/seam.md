---
seam_id: SEAM-5
seam_slug: verification-and-smoke-conformance
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-5-verification-and-smoke-conformance.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
    - ../../governance/seam-4-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - "Any change to net_allowed canonicalization/validation rules or world_network routing semantics"
    - "Any change to world.net.filter precedence, override applicability, or exported parity env semantics"
    - "Any change to runtime failure taxonomy, attach-or-fail behavior, or deny-all DNS semantics"
    - "Any change to doctor endpoint schema, field naming, or shell-side rendering/passthrough"
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S3
  status: pending
open_remediations: []
---
# SEAM-5 - Verification and smoke conformance

## Seam Brief (Restated)

- **Goal / value**: turn the landed config, routing, runtime, and doctor contracts into one conformance bundle that prevents silent drift and gives operators a repeatable smoke path.
- **Type**: conformance
- **Scope**
  - In:
    - extend or add regression coverage that spans Snapshot V3 `net_allowed`, host routing, doctor JSON passthrough, and runtime failure semantics
    - refresh privileged Linux verification posture for requested isolation and deny-all behavior
    - publish a repeatable macOS Lima smoke flow for allow-all versus deny-all/restrictive policy postures
    - close the terminal seam with a concrete closeout and thread/accounting pass
  - Out:
    - new enforcement behavior or contract ownership changes in `SEAM-1` through `SEAM-4`
- **Touch surface**:
  - `crates/transport-api-types/src/lib.rs`
  - `crates/shell/tests/world_request_net_allowed_snapshot.rs`
  - `crates/shell/tests/doctor_scopes_ds0.rs`
  - `crates/shell/tests/shim_doctor.rs`
  - `crates/world/src/netfilter.rs`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/smoke.sh`
  - `docs/reference/config/world.md`
  - `docs/reference/world/platforms/macos-lima-setup.md`
  - `docs/reference/world/verification/netfilter_enforcement.md`
- **Verification**:
  - cross-seam regression coverage for config/routing/doctor invariants
  - ignored privileged Linux validation for requested isolation and deny-all behavior
  - operator-facing macOS Lima smoke steps that exercise both deny-all and allow-all postures
  - seam-exit evidence in `../../governance/seam-5-closeout.md`
- **Basis posture**:
  - Currentness: `current`; the seam now plans against landed `SEAM-1` through `SEAM-4` closeouts instead of provisional upstream intent.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
    - `../../governance/seam-3-closeout.md`
    - `../../governance/seam-4-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
    - `THR-04`
    - `THR-05`
  - Stale triggers:
    - see `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers:
    - none; the prior active seam `SEAM-4` closeout records `seam_exit_gate.status: passed`, `promotion_readiness: ready`, and `gates.post_exec.landing: passed`, while `THR-05` is now published and revalidated for this seam.
  - Downstream blocked seams:
    - none; this is the terminal seam in the pack.
  - Contracts produced:
    - none; this seam consumes previously published contracts and turns them into conformance evidence.
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
    - `C-05`
    - `C-06`
    - `C-07`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: this is the terminal seam, so the pack needs one closeout-backed record that the regression, privileged, and smoke surfaces all landed and that the consumed threads were revalidated without downstream carry.
- **Expected contracts to publish**:
  - none
- **Expected threads to publish / advance**:
  - `THR-05`: already revalidated during promotion; closeout should account for that handoff explicitly
  - `THR-01` through `THR-04`: remain revalidated and may be closed only if the seam-exit evidence shows no further pack work depends on them
- **Likely downstream stale triggers**:
  - none inside this pack; future drift would reopen conformance work outside this pack
- **Expected closeout evidence**:
  - landed regression coverage references
  - landed privileged Linux verification references
  - landed macOS Lima smoke guidance and/or automation references
  - explicit terminal remediation disposition

## Slice index

- `S1` -> `slice-1-regression-matrix-for-routing-and-doctor-contracts.md`
- `S2` -> `slice-2-privileged-and-macos-smoke-conformance.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-5-closeout.md`
