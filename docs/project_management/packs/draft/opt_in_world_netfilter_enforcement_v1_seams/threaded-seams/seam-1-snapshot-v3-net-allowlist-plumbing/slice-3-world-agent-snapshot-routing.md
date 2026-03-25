---
slice_id: S3
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to world-agent execute request shape that carries snapshot/worldspec"
    - "Any change to PTY/non-PTY execution path behavior"
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
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations:
  - REM-001
candidate_subslices: []
---
### S3 - world-agent consumes Snapshot V3 `net_allowed` and `WorldSpec` (remove broker coupling)

- **User/system value**: the world’s routing/enforcement inputs are explicit and replayable; no hidden dependency on in-guest broker state for allowlists.
- **Scope (in/out)**:
  - In:
    - Ensure both non-PTY and PTY paths consume Snapshot V3 `net_allowed` and `WorldSpec` fields.
    - Remove/avoid any call path that consults `substrate_broker::allowed_domains()` for execution allowlists.
  - Out:
    - Implementing the enforcement itself (owned by `SEAM-2`).
- **Acceptance criteria**:
  - world-agent forwards `isolate_network` and `allowed_domains` to the world backend consistently for all execute paths.
  - Snapshot V3 `net_allowed` is the sole source of truth for allowlists inside the world-agent request pipeline.
  - Diagnostics make it clear when isolation is requested and why invalid allowlist entries fail.
- **Dependencies**:
  - `S1` publishes `C-01` and resolves normalization posture (`REM-001`).
  - `S2` constructs `WorldSpec` consistently on the host.
- **Verification**:
  - Targeted tests for request parsing/handling (unit or integration, depending on existing harness) covering PTY and non-PTY surfaces.
- **Rollout/safety**:
  - Back-compat preserved as long as isolate requests remain gated off by default.
- **Review surface refs**:
  - `review.md` (PTY vs non-PTY divergence, back-compat defaults)

#### S3.T1 - Align PTY and non-PTY request handling

- **Outcome**: single, consistent route for consuming snapshot/worldspec regardless of execution mode.
- **Inputs/outputs**:
  - In: request payload (Snapshot V3 + WorldSpec)
  - Out: backend execute call with explicit `isolate_network` + `allowed_domains`
- **Thread/contract refs**: `THR-01`, `THR-02`, `C-01`, `C-02`, `C-03`
- **Implementation notes**:
  - Prefer a shared helper so `service.rs` and `pty.rs` cannot drift.
- **Acceptance criteria**:
  - No broker-derived allowlist lookups remain on the world-agent execution path.
  - PTY and non-PTY calls produce identical enforcement inputs for equivalent requests.
- **Test notes**:
  - Add at least one regression test that fails if broker allowlist code path reappears.

Checklist:
- Implement: shared helper for request → backend spec mapping
- Test: PTY + non-PTY coverage
- Validate: ensure diagnostics are actionable for invalid allowlist entries
- Cleanup: remove dead allowlist plumbing that bypasses Snapshot V3
