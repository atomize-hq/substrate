---
seam_id: SEAM-3
seam_slug: host-config-opt-in-and-parity-env-plumbing
type: capability
status: landed
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-03
  stale_triggers:
    - "Any change to config schema merge/patch behavior; revalidate SEAM-1 routing consumption and SEAM-5 conformance coverage"
    - "Any change to workspace detection for overrides; revalidate SEAM-5 override coverage and operator docs"
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S4
  status: passed
open_remediations: []
---

# SEAM-3 - Config opt-in `world.net.filter` + CLI patching + env parity

- **Goal / value**: Provide an operator-controlled opt-in lever that determines whether the host requests netfilter enforcement at all, preserving back-compat by default.
- **Scope**
  - In:
    - Add `world.net.filter: bool` under `WorldConfig` (default `false`) with patch/merge/explain plumbing.
    - CLI patch application:
      - `substrate config set world.net.filter=true|false`
      - `substrate config reset world.net.filter`
    - Override env (no workspace): `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER=1|0|true|false`.
    - Export env script parity: `SUBSTRATE_WORLD_NET_FILTER=1|0`.
    - Documentation updates: `docs/reference/config/world.md` and `docs/CONFIGURATION.md`.
  - Out:
    - Implementing world enforcement details (`SEAM-2`).
    - Snapshot schema/canonicalization (`SEAM-1`).
- **Primary interfaces**
  - Inputs:
    - Workspace config and CLI patches.
    - Override env input for non-workspace contexts.
  - Outputs:
    - Host-side routing decision: whether to ever request isolate_network based on `world.net.filter`.
    - Exported env var for parity/debugging.
- **Key invariants / rules**:
  - Default is `false` (must preserve existing behavior for restrictive `net_allowed` policies).
  - Overrides apply only when there is no workspace (must not silently override workspace config).
- **Dependencies**
  - Direct blockers:
    - none
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-1` consumes the config decision to decide whether to request isolation.
  - Derived consumers:
    - `SEAM-5` tests for config round-trip and override behavior.
- **Touch surface**:
  - `crates/shell/src/execution/config_model.rs`
  - `crates/shell/src/execution/env_scripts.rs`
  - `docs/reference/config/world.md`
  - `docs/CONFIGURATION.md`
- **Verification**:
  - Config round-trip tests (`config current show`).
  - Override env tests when no workspace exists.
  - User-facing docs/examples now publish the three-way gate alignment.
- **Risks / unknowns**:
  - Risk: inconsistent merge/patch behavior could make the lever confusing across workspace/global config layers.
  - De-risk plan: model patches similar to existing `world.env.*`/`world.deps.*` patterns and include explain output.
- **Rollout / safety**:
  - This is the primary back-compat gate; must land before enabling enforcement for broader users.
- **Downstream decomposition context**:
  - Why this seam is `active`: it is the earliest safe upstream seam on the critical path and publishes the host-side gate that `SEAM-1` and later seams cannot safely invent downstream.
  - Which threads matter most: `THR-03`.
  - What the first seam-local review should focus on: config patch semantics, override precedence, documentation clarity, and the exact handoff `SEAM-1` will consume.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-04`, `C-05`, `C-06`
  - Threads likely to advance:
    - `THR-03` to `published`
  - Review-surface areas likely to shift after landing:
    - config/export visibility in operator workflows
  - Downstream seams most likely to require revalidation:
    - `SEAM-1` for routing adoption of the published host gate
    - `SEAM-5` for smoke expectations
