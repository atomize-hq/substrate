---
seam_id: SEAM-1
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - "Any change to PolicySnapshotV3 schema or net_allowed canonicalization/validation rules requires SEAM-2, SEAM-4, and SEAM-5 revalidation."
    - "Any change to host routing semantics for world.net.filter or world_network payload construction requires SEAM-2 and SEAM-5 revalidation."
    - "Any change to world-service request routing across PTY/non-PTY paths requires SEAM-2, SEAM-4, and SEAM-5 revalidation."
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-1 Snapshot V3 `net_allowed` contract + host→world-service plumbing

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/slice-4-seam-exit-gate.md`
- **Landed evidence**:
  - `crates/transport-api-types/src/lib.rs` publishes `PolicySnapshotV3.net_allowed` plus the canonicalization and enforcement-validation helpers that define `C-01`.
  - `crates/shell/src/execution/policy_snapshot.rs` canonicalizes `net_allowed`, consumes the published `world.net.filter` gate from `SEAM-3`, and emits `world_network` routing that encodes `C-02` and `C-03`.
  - `crates/world-service/src/request_routing.rs`, `crates/world-service/src/service.rs`, and `crates/world-service/src/pty.rs` consume Snapshot V3 plus `world_network` across non-PTY and PTY paths without broker-derived allowlist lookups on the `SEAM-1` request path.
  - Verification landed in `crates/transport-api-types/src/lib.rs`, `crates/world-service/src/request_routing.rs`, `crates/shell/tests/world_request_net_allowed_snapshot.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/config_show.rs`, `crates/shell/tests/config_set.rs`, and `crates/shell/tests/ev0_override_split.rs`.
- **Contracts published or changed**:
  - `C-01`: `PolicySnapshotV3.net_allowed` is now the canonical host-to-world allowlist contract with additive serde-default back-compat behavior.
  - `C-02`: `WorldSpec.isolate_network` is requested only when the effective `world.net.filter` gate is enabled and canonicalized `net_allowed` is restrictive.
  - `C-03`: `WorldSpec.allowed_domains` is derived from canonicalized `net_allowed` and is only meaningful when `isolate_network=true`.
- **Threads published / advanced**:
  - `THR-01` is published: the host snapshot now carries canonicalized `net_allowed`, and downstream consumers rely on the snapshot instead of broker state.
  - `THR-02` is published: host and world-service now share one explicit routing contract for `isolate_network` and `allowed_domains`, including allow-all and deny-all postures.
- **Review-surface delta**:
  - Snapshot V3 now owns the allowlist truth.
  - Host routing now owns the opt-in gate semantics and request shape.
  - world-service request routing now enforces PTY/non-PTY parity through `request_routing.rs` instead of per-path broker lookups.
- **Planned-vs-landed delta**:
  - The landed implementation uses the additive `world_network` request payload to make `C-02` and `C-03` explicit across service boundaries; this is tighter than the original planning text but aligned with the seam intent.
  - No remaining seam-local delta exists for `C-01` to `C-03`; downstream seams now consume a published handoff.
- **Downstream stale triggers raised**:
  - Any change to `net_allowed` canonicalization or validation rules requires `SEAM-2`, `SEAM-4`, and `SEAM-5` revalidation.
  - Any change to `world.net.filter` routing semantics or `world_network` payload construction requires `SEAM-2` and `SEAM-5` revalidation.
  - Any change to PTY/non-PTY request routing semantics requires `SEAM-2`, `SEAM-4`, and `SEAM-5` revalidation.
- **Remediation disposition**:
  - `REM-001` remains resolved by the recorded normalization contract and the landed canonicalization tests.
  - `REM-004` remains resolved because `SEAM-1` now consumes the published `SEAM-3` host gate in code and request routing tests.
  - No new seam-local remediations were opened during closeout.
- **Promotion blockers**:
  - none at the `SEAM-1` boundary; downstream enforcement, diagnostics, and conformance seams still own their execution work.
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - none
- **Carried-forward remediations**:
  - none; remaining work belongs to downstream seams rather than an unresolved `SEAM-1` owner gap.
