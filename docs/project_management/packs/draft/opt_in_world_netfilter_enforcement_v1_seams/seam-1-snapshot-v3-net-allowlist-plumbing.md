---
seam_id: SEAM-1
seam_slug: snapshot-v3-net-allowlist-plumbing
type: integration
status: decomposed
execution_horizon: next
plan_version: v2
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - "Any change to policy schema for net egress (net_allowed semantics)"
    - "Any world-agent request/response contract changes"
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

- **Goal / value**: Ensure world-agent enforcement is driven by an explicit, canonicalized `net_allowed` allowlist in Policy Snapshot V3 (no in-guest broker state), and that the host-to-world routing is unambiguous about when to request isolation vs allow-all.
- **Scope**
  - In:
    - Add `net_allowed` to Snapshot V3 with `#[serde(default)]`.
    - Canonicalize/validate `net_allowed` (trim, drop empties, dedupe, collapse `"*"` to `["*"]`, reject unsupported wildcards/URLs/ports/paths when enforcement is enabled).
    - Host snapshot builder populates `net_allowed` from effective policy.
    - world-agent uses Snapshot V3 `net_allowed` instead of `substrate_broker::allowed_domains()` and routes requests to the world backend via `WorldSpec`.
  - Out:
    - Implementing `world.net.filter` config and CLI (owned by `SEAM-3`).
    - Making nftables enforcement fail-closed at runtime (owned by `SEAM-2`).
- **Primary interfaces**
  - Inputs:
    - Effective policy `net_allowed` (host-side).
    - Host config gate `world.net.filter` (producer is `SEAM-3`; consumed here).
  - Outputs:
    - `PolicySnapshotV3.net_allowed` delivered to world-agent.
    - `WorldSpec.isolate_network` and `allowed_domains` request shape passed to world backend.
- **Key invariants / rules**:
  - Snapshot is the sole source of truth for `net_allowed` in the world; no in-world broker lookups for allowlists.
  - `"*"` is the only supported wildcard and canonicalizes to exactly `["*"]`.
  - When enforcement is requested, unsupported wildcards must fail with a diagnostic (no pretend enforcement).
- **Dependencies**
  - Direct blockers:
    - `SEAM-3` for the opt-in config value definition (to decide when to request isolation).
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-2` (world enforcement consumes isolate/allowed_domains)
    - `SEAM-4` (doctor/diagnostics reflect requested vs enabled)
  - Derived consumers:
    - `SEAM-5` (tests/smoke)
- **Touch surface**:
  - `crates/agent-api-types` snapshot schema
  - `crates/shell/src/execution/policy_snapshot.rs`
  - `crates/world-agent/src/service.rs` + `crates/world-agent/src/pty.rs`
- **Verification**:
  - Unit tests for canonicalization/validation in `agent-api-types`.
  - Tests asserting world-agent routes allowlists from snapshot (not broker).
- **Current blocker posture**:
  - `S2` still consumes `C-04` / `THR-03`, so this seam now sits in `next` with a provisional basis until the active `SEAM-3` publishes the config gate and parity surfaces.
- **Risks / unknowns**:
  - Risk: hostname normalization/IDNA behavior could introduce false denies or unexpected allows.
  - De-risk plan: keep the normalization posture explicit and test-locked (decision is recorded in `SEAM-1/S1.T1`).
- **Rollout / safety**:
  - This seam must be back-compat additive; no behavior changes unless opt-in gate later requests isolation.
- **Downstream decomposition context**:
  - Why this seam is `next`: its consumer-side routing work is still real, but it should not be the active execution seam until the upstream opt-in gate (`SEAM-3`) is published.
  - Which threads matter most: `THR-01`, `THR-02`, `THR-03`.
  - What the next seam-local review should focus on: canonicalization rules, failure diagnostics, end-to-end data flow correctness across non-PTY and PTY execute paths, and whether the published `C-04` semantics from active `SEAM-3` are consumed consistently.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-01`, `C-02`, `C-03`
  - Threads likely to advance:
    - `THR-01` to `defined`/`published`
    - `THR-02` to `defined`
  - Review-surface areas likely to shift after landing:
    - control-plane/data-plane separation diagram details
  - Downstream seams most likely to require revalidation:
    - `SEAM-2` if enforcement inputs/semantics change
    - `SEAM-4` if doctor schema changes
