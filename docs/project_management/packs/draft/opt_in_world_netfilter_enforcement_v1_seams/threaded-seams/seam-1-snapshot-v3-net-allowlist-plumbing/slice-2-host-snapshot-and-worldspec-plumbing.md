---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: next
status: decomposed
plan_version: v2
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
  - REM-004
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
  - Upstream: active `SEAM-3` publishes `C-04` (`world.net.filter`) so the gating source is stable.
  - Contracts/threads: `C-01`/`C-02`/`C-03` and `THR-01`/`THR-02`/`THR-03`
- **Current blocker posture**:
  - `REM-004`: this slice remains provisional because the required `C-04` / `THR-03` publication now belongs to active `SEAM-3` owner slices that are not yet landed.
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

#### S2.T2 - Publish the host-side gating contract consumed from `SEAM-3`

- **Outcome**: `SEAM-1` consumes a concrete `C-04` / `THR-03` contract instead of relying on unpublished future-seam intent.
- **Inputs/outputs**:
  - In: `../../threading.md` (`C-04`, `C-05`, `C-06`, `THR-03`), `../../scope_brief.md`, and existing config/env precedence patterns in `crates/shell/src/execution/config_model.rs` and `crates/shell/src/execution/env_scripts.rs`
  - Out: an implementation-ready contract for when the host requests `WorldSpec.isolate_network` and how operators inspect/override that decision
- **Thread/contract refs**: `THR-03`, `C-04`
- **Contract decision (C-04 / THR-03): host-side netfilter opt-in posture**
  - `world.net.filter: bool` remains the authoritative host-side opt-in key under `WorldConfig`; default `false` preserves current behavior for existing restrictive policies.
  - The effective value follows the existing config replace stack for world booleans:
    - workspace config wins when `<workspace_root>/.substrate/workspace.yaml` exists
    - `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER=1|0|true|false|yes|no|on|off` applies only when no workspace exists
    - global config is the last persisted layer before the built-in default
  - User-facing mutation surface is `substrate config set world.net.filter=true|false` and `substrate config reset world.net.filter`; the key must participate in `substrate config current show --explain`.
  - `SUBSTRATE_WORLD_NET_FILTER=1|0` is exported output-only parity/debug state; it mirrors the resolved effective value and is not an override input.
  - `SEAM-1` host routing still publishes canonicalized `PolicySnapshotV3.net_allowed` on every request, but it only requests `WorldSpec.isolate_network=true` when both are true:
    - the effective `world.net.filter` gate is `true`
    - policy `net_allowed` is restrictive after canonicalization (anything other than the allow-all singleton `["*"]`)
  - When the gate is `false`, or the canonicalized policy posture is `["*"]`, the host does not request enforcement; `WorldSpec.allowed_domains` is semantically ignored because `C-03` only applies when `isolate_network=true`.
  - `WORLD_NETFILTER_ENABLE=1` is not part of `C-04`; it is the downstream world/service safety gate from `SEAM-2`. Missing that env must never force host routing on or off; it only determines whether a requested isolation run can execute successfully.
  - Operator-facing docs for this contract must describe the three-way gate alignment explicitly:
    - `world.net.filter` answers whether the host may request enforcement
    - `WORLD_NETFILTER_ENABLE=1` answers whether the world backend may apply enforcement
    - policy `net_allowed` answers what the allowlist is once enforcement is requested
- **Verification plan (planning-only)**:
  - Add config mutation coverage in `crates/shell/tests/config_set.rs`:
    - `config_set_updates_world_net_filter_and_supports_reset`
  - Add precedence / explain coverage in `crates/shell/tests/config_show.rs`:
    - `config_show_reports_world_net_filter_precedence_and_explain_sources`
  - Add no-workspace override coverage in `crates/shell/tests/ev0_override_split.rs`:
    - `world_net_filter_override_applies_only_without_workspace`
  - Add export parity coverage in `crates/shell/src/builtins/world_enable/runner/manager_env.rs` or `crates/shell/tests/world_enable.rs`:
    - assert `SUBSTRATE_WORLD_NET_FILTER=1|0` is written alongside the other derived `SUBSTRATE_*` exports
  - Add host-routing coverage in a shell/world request test surface (`crates/shell/tests/repl_world_first_routing_v1.rs` or a new focused routing test):
    - canonicalized `net_allowed` is still present on every request
    - `isolate_network=true` only when `world.net.filter=true` and policy is restrictive
    - `net_allowed=["*"]` keeps routing in allow-all posture even when the config gate is enabled
    - `net_allowed=[]` with `world.net.filter=true` requests deny-all enforcement
    - missing `WORLD_NETFILTER_ENABLE` does not alter host request construction; the failure belongs to downstream execution/runtime checks
- **Risk/rollback notes**:
  - This contract remains an open blocker until `SEAM-3` lands the config key, parity env export, and docs/tests, and until `SEAM-1` switches routing to consume the published gate instead of broker-only state.
