---
seam_id: SEAM-3
seam_slug: host-config-opt-in-and-parity-env-plumbing
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-host-config-opt-in-and-parity-env-plumbing.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-03
  stale_triggers:
    - "Any change to config schema merge/patch behavior"
    - "Any change to workspace detection for overrides"
    - "Any change to host routing semantics for when isolate_network is requested"
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
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-3 - Config opt-in `world.net.filter` + CLI patching + env parity

## Seam Brief (Restated)

- **Goal / value**: Publish the host-side opt-in gate that preserves back-compat by default and gives downstream seams one authoritative answer for when egress enforcement may be requested.
- **Type**: capability
- **Scope**
  - In:
    - Add `world.net.filter: bool` under `WorldConfig` with patch/merge/explain plumbing.
    - Add no-workspace override env `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER`.
    - Export parity/debug state `SUBSTRATE_WORLD_NET_FILTER=1|0`.
    - Update operator docs and examples for the three-way gate alignment.
  - Out:
    - Snapshot `net_allowed` schema/canonicalization (`SEAM-1`).
    - Runtime netfilter enforcement implementation (`SEAM-2`).
- **Touch surface**:
  - `crates/shell/src/execution/config_model.rs`
  - `crates/shell/src/execution/config_cmd.rs`
  - `crates/shell/src/execution/env_scripts.rs`
  - `docs/reference/config/world.md`
  - `docs/CONFIGURATION.md`
- **Verification**:
  - Config mutation + explain tests
  - No-workspace override env tests
  - Export parity tests
  - Downstream routing handoff checks for `SEAM-1`
- **Basis posture**:
  - Currentness: `current`; this seam has no upstream contract producers and is now the earliest safe active seam on the critical path.
  - Upstream closeouts assumed: none
  - Required threads: `THR-03`
  - Stale triggers: see `basis.stale_triggers`
- **Threading constraints**
  - Downstream consumers:
    - `SEAM-1` consumes `C-04` to decide whether the host may request `isolate_network`.
    - `SEAM-5` consumes `C-05` / `C-06` for override/export coverage.
  - Contracts produced (per `../../threading.md`):
    - `C-04` (`world.net.filter` host-side opt-in gate)
    - `C-05` (`SUBSTRATE_OVERRIDE_WORLD_NET_FILTER`)
    - `C-06` (`SUBSTRATE_WORLD_NET_FILTER`)
  - Current downstream posture:
    - The host-gate contract is now published in code, env parity, and operator docs; downstream seams consume it as a
      landed handoff and revalidate only when those semantics change.

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- `../../review_surfaces.md` is pack-level orientation only.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: it publishes the back-compat control-plane contract that all later routing, enforcement, and conformance work must trust.
- **Expected contracts to publish**: `C-04`, `C-05`, `C-06`
- **Expected threads to publish / advance**:
  - `THR-03`: `identified` -> `defined`/`published`
- **Likely downstream stale triggers**:
  - Any change to config precedence or explain output
  - Any change to override parsing or workspace detection semantics
  - Any change to exported env names or meanings
- **Expected closeout evidence**:
  - Config model and CLI patch surfaces landed for `world.net.filter`
  - No-workspace override and export parity evidence landed
  - Operator docs/examples landed and aligned to the three-way gate semantics

## Slice index

- `S1` -> `slice-1-publish-world-net-filter-config-contract.md`
- `S2` -> `slice-2-override-and-parity-env-plumbing.md`
- `S3` -> `slice-3-operator-docs-and-routing-handoff.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
