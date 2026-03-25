---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to host config gating semantics for requesting isolation (C-04)"
    - "Any change to WorldSpec.isolate_network/allowed_domains semantics (C-02/C-03)"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
  - THR-03
contracts_produced:
  - C-02
  - C-03
contracts_consumed:
  - C-01
  - C-04
open_remediations:
  - REM-001
candidate_subslices: []
---
### S2 - Host snapshot builder populates `net_allowed` and constructs `WorldSpec` (C-02/C-03)

- **User/system value**: the host sends a single, explicit request describing both the allowlist and whether enforcement is required, so the world can enforce-or-fail deterministically.
- **Scope (in/out)**:
  - In:
    - Populate `PolicySnapshotV3.net_allowed` from effective policy.
    - Construct `WorldSpec.isolate_network` and `WorldSpec.allowed_domains` with semantics from `../../threading.md`.
    - Wire opt-in gating from `world.net.filter` (`C-04`) into whether `isolate_network=true` is ever requested.
  - Out:
    - Adding new config fields / CLI / docs for `world.net.filter` itself (owned by `SEAM-3`).
- **Acceptance criteria**:
  - Snapshot builder emits canonicalized `net_allowed` (via `C-01` helper) for every execution request.
  - `WorldSpec.isolate_network=true` is only requested when the opt-in gate is true (per `C-04`), and defaults preserve back-compat (no unexpected isolation requests).
  - `WorldSpec.allowed_domains` is derived from canonicalized `net_allowed` and is only meaningful when `isolate_network=true`.
  - Unsupported wildcard forms are rejected (fail with diagnostic) when `isolate_network=true`.
- **Dependencies**:
  - Upstream: `SEAM-3` publishes `C-04` (`world.net.filter`) so the gating source is stable.
  - Contracts/threads: `C-01`/`C-02`/`C-03` and `THR-01`/`THR-02`/`THR-03`
- **Verification**:
  - Unit/integration tests at the shell layer asserting:
    - snapshot contains canonicalized `net_allowed`
    - constructed `WorldSpec` values match the snapshot + gating posture
- **Rollout/safety**:
  - Default opt-in remains off; isolate requests are not emitted unless explicitly enabled.
- **Review surface refs**:
  - `review.md` (PTY vs non-PTY divergence, back-compat defaults)

#### S2.T1 - Build Snapshot V3 `net_allowed` from effective policy

- **Outcome**: snapshot carries canonicalized allowlist for every request.
- **Inputs/outputs**:
  - In: effective policy output from broker evaluation
  - Out: `PolicySnapshotV3.net_allowed` populated
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - Ensure snapshot build path calls the single canonicalization/validation helper.
- **Acceptance criteria**:
  - Snapshot is always populated (field present via serde default, but also explicitly set during build).
  - Errors for invalid entries are surfaced with actionable diagnostics when isolation is requested.
- **Test notes**:
  - Add tests for: empty policy list, duplicates/whitespace, `"*"` collapse.

Checklist:
- Implement: snapshot build uses shared helper
- Test: shell-level tests for snapshot contents
- Validate: ensure failures are diagnostic and do not silently fall back
- Cleanup: remove any old broker-based allowlist plumbing at this layer
