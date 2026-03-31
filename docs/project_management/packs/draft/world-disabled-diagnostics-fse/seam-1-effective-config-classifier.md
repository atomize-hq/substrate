---
seam_id: SEAM-1
seam_slug: effective-config-classifier
type: integration
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - docs/reference/env/contract.md changes effective-config precedence or the workspace override-ignore rule
    - resolve_effective_config semantics or signature change in crates/shell/src/execution/config_model.rs
    - diagnostics routing or exit-code handling changes for user/config failures after `THR-01` publication
    - adjacent diagnostics packs modify health or shim-doctor call paths before downstream seams revalidate the published handoff
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
  planned_location: S3
  status: passed
open_remediations: []
---

# SEAM-1 - Effective config classifier

- **Goal / value**:
  Produce one canonical diagnostics-side answer to "is the world enabled?" so every downstream seam can distinguish disabled-by-choice from enabled-but-broken without duplicating precedence logic.
- **Scope**
  - In:
    - Map CLI overrides `--world` / `--no-world` into the shared effective-config resolver
    - Resolve `world.enabled` for both `substrate shim doctor` and `substrate health`
    - Fail fast with stderr + exit `2` on config-resolution errors before any probe or output happens
    - Make the shared classifier concrete enough that downstream seams can consume one contract instead of local heuristics
  - Out:
    - Disabled/skipped text copy and JSON status publication
    - Health summary aggregation rules
    - Cross-platform smoke evidence beyond proving the classifier and config-error posture
- **Primary interfaces**
  - Inputs:
    - current working directory
    - CLI overrides `--world` / `--no-world`
    - workspace/global/default config layers and `SUBSTRATE_OVERRIDE_WORLD` when allowed
    - `resolve_effective_config` and diagnostics routing
  - Outputs:
    - shared `effective_world_enabled` outcome for diagnostics
    - deterministic config-resolution failure posture (stderr + exit `2`, no diagnostic probing)
- **Key invariants / rules**:
  - No ad-hoc precedence outside the existing resolver
  - Enabled workspace ignores `SUBSTRATE_OVERRIDE_*`
  - `--world` wins over disablement and `--no-world` wins over enablement
  - Resolver failure is terminal for diagnostics classification and must not degrade into guesswork
- **Dependencies**
  - Direct blockers:
    - External authoritative precedence contract in `docs/reference/env/contract.md`
    - Existing resolver API and diagnostics routing behavior in `crates/shell/src/execution/*`
  - Transitive blockers:
    - Exit-code taxonomy assumptions used by downstream reporting seams
    - Cross-queue shared-file churn in diagnostics entrypoints
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - `SEAM-4`
    - future attribution / json-envelope / provisioning-related diagnostics packs
- **Touch surface**:
  `crates/shell/src/execution/config_model.rs`, `crates/shell/src/execution/routing.rs`, `crates/shell/src/builtins/shim_doctor/report.rs`, `crates/shell/src/builtins/health.rs`, and resolver precedence/error tests.
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - For this seam specifically: seam-local review should prove invalid YAML/unreadable config exits `2` before any probe/output, outside-workspace override behavior is honored, and enabled-workspace override-ignore behavior is preserved.
- **Risks / unknowns**:
  - Risk: diagnostics call sites could reintroduce local precedence logic while still appearing to use the shared resolver.
  - De-risk plan: insist on one helper or one directly shared call path and add precedence fixtures for both commands.
  - Risk: routing divergence could make one command exit `2` and the other exit `0` on the same config error.
  - De-risk plan: verify both entrypoints through the same failure-path assertions.
- **Rollout / safety**:
  No new config keys or environment variables are introduced. This seam only changes how diagnostics classify the existing config state and should reduce misleading output rather than broaden runtime behavior.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is the earliest publishable foundation and blocks every downstream user-facing seam.
  - Which threads matter most: `THR-01`.
  - What the first seam-local review should focus on: resolver authority, workspace override-ignore semantics, and proof that config errors stop all probe/output paths.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`
  - Threads likely to advance: `THR-01`
  - Review-surface areas likely to shift after landing: command decision flow and pre-probe branching in both diagnostics entrypoints
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
- **Closeout posture**:
  - This seam has left the forward window after `governance/seam-1-closeout.md` recorded `THR-01` as published with `promotion_readiness: ready`.
  - The owned contract surface is now authoritative through `governance/seam-1-closeout.md` and the landed resolver-backed helper path.
  - Downstream seams that consume this handoff are `SEAM-2` and `SEAM-3`.
