---
seam_id: SEAM-3
seam_slug: health-disabled-aware-summary
type: capability
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - THR-02 or THR-03 change after `SEAM-2` closeout
    - provisioning-related packs change enabled-mode world-deps remediation text in health.rs
    - docs/USAGE.md drifts from shipped JSON/text behavior
    - summary aggregation continues to consume legacy error strings rather than published status contracts
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
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-3 - Health disabled-aware summary

- **Goal / value**:
  Make `substrate health` a trustworthy operator signal for host-only-by-choice installs by treating disabled/skipped as non-error, suppressing enabled-world guidance when disabled, and keeping enabled-world failures visible.
- **Scope**
  - In:
    - Consume published shim status contracts rather than inferring from legacy fields
    - Render deterministic disabled/skipped text for `substrate health`
    - Set `summary.world_ok = null` and omit summary error fields when disabled
    - Emit empty `world_deps_missing` / `world_deps_blocked` arrays when disabled
    - Suppress enabled-world `substrate world deps current` guidance when disabled
    - Align `docs/USAGE.md` to the published machine-readable contract
  - Out:
    - Shim-doctor reporting internals
    - Resolver/plumbing foundation
    - Cross-platform checkpoint orchestration beyond seam-local proof and docs alignment
- **Primary interfaces**
  - Inputs:
    - embedded shim payload carrying `.world.status` and `.world_deps.status`
    - health summary aggregation logic
    - operator docs/examples surface
  - Outputs:
    - disabled-mode health summary contract and operator copy (`C-05`)
    - updated docs/examples consistent with landed behavior
- **Key invariants / rules**:
  - Disabled mode is non-error and must not create world/world-deps failures solely because probes were skipped
  - Enabled mode remains fail-visible and may still report overall attention required
  - Disabled/skipped enum values must not appear when world is enabled
  - `docs/USAGE.md` must describe the new status enums as canonical machine-readable fields
- **Dependencies**
  - Direct blockers:
    - `SEAM-1` / `THR-01`
    - `SEAM-2` / `THR-02`, `THR-03`
  - Transitive blockers:
    - future provisioning guidance changes in health-related surfaces
    - json-envelope work that could obscure the embedded shim payload
  - Direct consumers:
    - `SEAM-4`
  - Derived consumers:
    - operators and downstream packs that rely on `substrate health` as the summary signal
- **Touch surface**:
  `crates/shell/src/builtins/health.rs`, `crates/shell/tests/shim_health.rs`, `docs/USAGE.md`.
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - For this seam specifically: seam-local review should prove disabled summary null/omission rules, guidance suppression, enabled-but-broken attention-required behavior, and docs/examples that match the landed contract exactly.
- **Risks / unknowns**:
  - Risk: summary aggregation may still key off legacy `error` strings and reintroduce false-negative failures even after shim contracts publish.
  - De-risk plan: drive summary from status enums first, with explicit disabled-mode branches and negative assertions on `summary.failures`.
  - Risk: docs or examples drift from exact shipped behavior and confuse operators/tooling.
  - De-risk plan: derive examples from test fixtures or smoke assertions instead of hand-authored prose only.
- **Rollout / safety**:
  This seam should reduce misleading attention-required signals without hiding real failures. It must preserve enabled-mode remediation guidance and error visibility when `world.enabled=true`.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now the active seam because `SEAM-2` published `THR-02` and `THR-03` through a passed seam-exit gate, so the health summary can consume landed shim status contracts instead of provisional assumptions.
  - Which threads matter most: `THR-01`, `THR-02`, `THR-03`, `THR-05`.
  - What the first seam-local review should focus on: status-driven summary aggregation, guidance suppression, and documentation parity.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-05`
  - Threads likely to advance: `THR-05`
  - Review-surface areas likely to shift after landing: overall status line, summary JSON, guidance lines, and docs/examples
  - Downstream seams most likely to require revalidation: `SEAM-4` and future health/provisioning/json-envelope packs
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
