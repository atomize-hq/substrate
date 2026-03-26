---
seam_id: SEAM-4
seam_slug: world-doctor-netfilter-status-observability
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-4-world-doctor-netfilter-status-observability.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - "Any change to net_allowed canonicalization, world_network routing semantics, or PTY/non-PTY request parity"
    - "Any change to runtime failure taxonomy, WORLD_NETFILTER_ENABLE propagation, or attach-failure wording"
    - "Any change to doctor endpoint schema or shell-side world doctor JSON rendering"
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
# SEAM-4 - Observability: doctor output makes enforcement status obvious

## Seam Brief (Restated)

- **Goal / value**: turn the landed config/routing/runtime handoffs into one operator-readable doctor contract that makes requested netfilter state debuggable without reading code or tracing logs.
- **Type**: integration
- **Scope**
  - In:
    - publish an additive doctor JSON netfilter block for requested vs enabled status, `WORLD_NETFILTER_ENABLE` presence, and last failure reason
    - keep the doctor response aligned across world-agent output and shell-side `substrate world doctor --json` rendering
    - preserve the upstream fail-closed/runtime taxonomy as actionable diagnostics instead of collapsing to generic readiness
  - Out:
    - new runtime enforcement behavior (`SEAM-2`)
    - host gate policy/config ownership (`SEAM-3`)
    - terminal smoke and conformance coverage (`SEAM-5`)
- **Touch surface**:
  - `crates/agent-api-types/src/lib.rs`
  - `crates/world-agent/src/handlers.rs`
  - `crates/shell/src/execution/platform/linux.rs`
  - `crates/shell/src/execution/platform/macos.rs`
  - `crates/shell/src/execution/platform/windows.rs`
  - focused world-doctor tests in `crates/shell/tests`
- **Verification**:
  - additive JSON shape tests for the world doctor surface
  - focused rendering assertions for shell-side doctor output
  - downstream manual smoke consumption in `SEAM-5`
- **Basis posture**:
  - Currentness: `current`; the seam now plans against landed `SEAM-1`, `SEAM-2`, and `SEAM-3` closeouts instead of provisional observability intent.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
    - `../../governance/seam-3-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
    - `THR-04`
  - Stale triggers:
    - see `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers:
    - none; the prior active seam `SEAM-2` closeout records `seam_exit_gate.status: passed`, `promotion_readiness: ready`, and `gates.post_exec.landing: passed`.
  - Downstream blocked seams:
    - `SEAM-5` depends on this seam to publish `C-07` / `THR-05` before its conformance and smoke plan can fully revalidate the doctor surface.
  - Contracts produced (per `../../threading.md`):
    - `C-07`
  - Contracts consumed (per `../../threading.md`):
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`
  - Current upstream handoff:
    - `SEAM-1` now publishes the requested-isolation derivation and request-shape truth this seam must surface as `requested`.
    - `SEAM-3` now publishes the host gate semantics this seam must distinguish from runtime enablement.
    - `SEAM-2` now publishes the fail-closed runtime taxonomy and guard semantics this seam must expose as actionable diagnostics.

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- `../../review_surfaces.md` is pack-level orientation only.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: downstream conformance work needs one closeout-backed statement of which doctor fields landed, which failure reasons are intentionally surfaced, and when `THR-05` is actually published.
- **Expected contracts to publish**:
  - `C-07`
- **Expected threads to publish / advance**:
  - `THR-05`: `identified` -> `published`
- **Likely downstream stale triggers**:
  - any change to doctor endpoint schema or shell-side world doctor rendering
  - any change to runtime failure taxonomy or `WORLD_NETFILTER_ENABLE` wording surfaced from `SEAM-2`
  - any change to request derivation semantics inherited from `SEAM-1` / `SEAM-3`
- **Expected closeout evidence**:
  - landed world-agent doctor fields and schema updates
  - landed shell-side world doctor JSON passthrough/rendering updates
  - focused tests pinning additive field shape and actionable failure-reason output

## Slice index

- `S1` -> `slice-1-publish-netfilter-status-contract.md`
- `S2` -> `slice-2-thread-runtime-failure-state-into-doctor-surfaces.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-4-closeout.md`
