---
seam_id: SEAM-1
seam_slug: snapshot-v3-net-allowlist-plumbing
status: decomposed
execution_horizon: active
plan_version: v2
basis:
  currentness: provisional
  source_seam_brief: ../../seam-1-snapshot-v3-net-allowlist-plumbing.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - "Any change to PolicySnapshotV3 schema or net_allowed canonicalization/validation rules"
    - "Any change to world-agent execute request shape that carries snapshot/worldspec"
    - "Any change to WorldSpec.isolate_network/allowed_domains semantics"
    - "Any change to host config gating semantics for world.net.filter (C-04)"
gates:
  pre_exec:
    review: passed
    contract: failed
    revalidation: failed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations:
  - REM-004
---
# SEAM-1 - Snapshot V3 `net_allowed` contract + host→world-agent plumbing

## Seam Brief (Restated)

- **Goal / value**: Ensure world-agent enforcement is driven by an explicit, canonicalized `net_allowed` allowlist carried in Policy Snapshot V3 (no in-world broker state), and that the host-to-world request shape is unambiguous about when to request isolation vs allow-all.
- **Type**: integration
- **Scope**
  - In:
    - Add `net_allowed` to Snapshot V3 with `#[serde(default)]`.
    - Canonicalize/validate `net_allowed` (trim, drop empties, dedupe, collapse `"*"` to `["*"]`, reject unsupported wildcards/URLs/ports/paths when enforcement is requested).
    - Host snapshot builder populates `net_allowed` from effective policy and constructs `WorldSpec.isolate_network` + `WorldSpec.allowed_domains`.
    - world-agent uses Snapshot V3 `net_allowed` / `WorldSpec` fields instead of `substrate_broker::allowed_domains()`.
  - Out:
    - Define/ship the host config opt-in lever `world.net.filter` (owned by `SEAM-3`).
    - Make nftables enforcement fail-closed at runtime (owned by `SEAM-2`).
- **Touch surface**:
  - `crates/agent-api-types` (Snapshot V3 schema + canonicalization/validation helpers)
  - `crates/shell/src/execution/policy_snapshot.rs` (host snapshot builder + WorldSpec construction)
  - `crates/world-agent/src/service.rs` and `crates/world-agent/src/pty.rs` (execute routing / allowlist plumbing)
- **Verification**:
  - Unit tests for canonicalization/validation in `agent-api-types`.
  - Tests asserting world-agent routes allowlists from snapshot (not broker), across non-PTY and PTY execute paths.
- **Basis posture**:
  - Currentness: `provisional` because `S2` still consumes `C-04` / `THR-03` from future `SEAM-3`; promotion to `exec-ready` stays blocked until that dependency is published or the sequencing/ownership is rewritten.
  - Upstream closeouts assumed: none
  - Required threads: `THR-01`, `THR-02`, `THR-03`
  - Stale triggers: see `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers:
    - `SEAM-3` publishes `C-04` (`world.net.filter`) which gates whether the host requests `WorldSpec.isolate_network=true`.
  - Downstream blocked seams:
    - `SEAM-2` consumes `C-02`/`C-03` to enforce fail-closed netfilter when isolation is requested.
    - `SEAM-4` and `SEAM-5` derive diagnostics and tests from these contracts/threads.
  - Contracts produced (per `../../threading.md`):
    - `C-01` (`PolicySnapshotV3.net_allowed`)
    - `C-02` (`WorldSpec.isolate_network` semantics)
    - `C-03` (`WorldSpec.allowed_domains` semantics)
  - Contracts consumed (per `../../threading.md`):
    - `C-04` (`world.net.filter` host-side opt-in gate)
  - Current blocker:
    - `REM-004` blocks `SEAM-1 -> exec-ready` because the authoritative control plane still assigns `C-04` / `THR-03` to future `SEAM-3`.

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- `../../review_surfaces.md` is pack-level orientation only.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: it publishes cross-seam boundary contracts (`C-01`..`C-03`) that downstream enforcement, diagnostics, and conformance work must be able to rely on without re-inventing implicit semantics.
- **Expected contracts to publish**: `C-01`, `C-02`, `C-03`
- **Expected threads to publish / advance**:
  - `THR-01`: `identified` → `defined`/`published`
  - `THR-02`: `identified` → `defined`/`published`
- **Likely downstream stale triggers**:
  - Any change to `net_allowed` canonicalization/validation rules
  - Any change to `WorldSpec` fields that represent enforcement request / allowlist shape
  - Any change to how PTY vs non-PTY paths source the allowlist
- **Expected closeout evidence**:
  - Pointers to the schema + tests landing for `C-01`
  - Evidence that world-agent consumes Snapshot V3 `net_allowed` and does not consult in-world broker state for allowlists

## Slice index

- `S1` -> `slice-1-publish-net-allowed-contract.md`
- `S2` -> `slice-2-host-snapshot-and-worldspec-plumbing.md`
- `S3` -> `slice-3-world-agent-snapshot-routing.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
