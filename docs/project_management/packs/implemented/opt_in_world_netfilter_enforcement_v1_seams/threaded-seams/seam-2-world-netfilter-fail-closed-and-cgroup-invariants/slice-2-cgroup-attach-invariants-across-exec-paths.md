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
    - "Any new process-spawn path that bypasses cgroup attach or weakens attach-or-fail behavior under isolate_network"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-02
open_remediations: []
candidate_subslices: []
---
### S2 - Make cgroup attach invariants unavoidable across exec paths

- **User/system value**: the nftables scope cannot be bypassed by spawning through a path that never joins the world cgroup.
- **Scope (in/out)**:
  - In:
    - inventory and harden the overlay bind-mount path, world-deps fallback path, and direct-exec path
    - convert cgroup attach failures from warnings to hard failures when `isolate_network=true`
    - refuse or re-route direct exec under isolation unless attach is guaranteed
  - Out:
    - downstream diagnostics JSON for last failure reason (`SEAM-4`)
    - terminal conformance coverage (`SEAM-5`)
- **Acceptance criteria**:
  - every spawn path reachable under requested isolation either writes the child PID to `cgroup.procs` successfully or aborts before the command runs
  - the direct-exec branch no longer bypasses cgroup attach under `isolate_network=true`
  - attach failures surface actionable diagnostics that `SEAM-4` can later publish
- **Dependencies**:
  - `S1` establishes enforce-or-fail posture for requested isolation
  - `crates/world/src/exec.rs` and `crates/world/src/session.rs` remain the authoritative spawn-path inventory
- **Verification**:
  - focused tests around attach failure behavior for bind-mount and fallback helpers
  - targeted regression proving direct exec cannot run unscoped under requested isolation
- **Rollout/safety**:
  - attach hard-fail semantics are limited to requested isolation; non-isolated execution keeps current behavior
- **Review surface refs**:
  - `review.md`

#### S2.T1 - Harden helper-based exec paths to attach-or-fail

- **Outcome**: helper paths stop treating cgroup attach as best effort during isolated runs.
- **Files**:
  - `crates/world/src/exec.rs`
- **Thread/contract refs**:
  - `THR-04`
  - `C-02`
- **Acceptance criteria**:
  - bind-mount and world-deps fallback helpers return errors instead of warnings when attach fails under isolation
  - diagnostics identify which helper path failed
- **Test notes**:
  - add focused failure-path tests or seams for attach refusal behavior.

Checklist:
- Implement: attach failure becomes fatal for isolated helper paths
- Test: bind-mount and fallback attach failure coverage
- Validate: helper-path warnings no longer allow isolated commands to proceed

#### S2.T2 - Remove the direct-exec cgroup bypass

- **Outcome**: forced direct exec can no longer escape cgroup scoping when isolation is requested.
- **Files**:
  - `crates/world/src/session.rs`
  - any helper introduced to preserve attach semantics
- **Thread/contract refs**:
  - `THR-04`
  - `C-02`
- **Acceptance criteria**:
  - `SUBSTRATE_WORLD_EXEC_FORCE_DIRECT` cannot yield an unscoped isolated process
  - either the direct path attaches correctly or isolated runs reject that path with a clear error
- **Test notes**:
  - add regression coverage for forced direct exec under `isolate_network=true`.

Checklist:
- Implement: attach-preserving or rejecting direct-exec behavior
- Test: forced direct-exec regression
- Validate: no isolated process can spawn outside the world cgroup
