---
seam_id: SEAM-1
seam_slug: effective-disable-attribution-foundation
type: integration
status: proposed
execution_horizon: active
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - ADR-0037 winning-layer precedence changes for world.enabled=false
    - tokenized display rules change for workspace or global config paths
    - allowlisted env token display rules change for SUBSTRATE_OVERRIDE_WORLD
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-1 - Effective disable attribution foundation

- **Goal / value**:
  - Produce one replay-safe classifier seam for effective `world.enabled=false` attribution so later runtime copy and telemetry can consume a single winning-layer contract.
  - Eliminate precedence drift and redaction drift between replay and the source pack's already-researched semantics.
- **Scope**
  - In:
    - one helper or helper-adjacent seam that resolves exactly one outcome: `override_env`, `workspace_patch`, `global_patch`, or `unknown`
    - tokenized display output for `<workspace>/.substrate/workspace.yaml` and `$SUBSTRATE_HOME/config.yaml`
    - allowlisted env-token handling for `SUBSTRATE_OVERRIDE_WORLD`
    - deterministic precedence and redaction tests for override env, workspace config, global config, and unknown-source fallback
  - Out:
    - replay origin-summary copy changes
    - replay host-warning copy changes
    - `replay_strategy` field additions
    - docs, smoke wrappers, and manual playbook lock-in
- **Primary interfaces**
  - Inputs:
    - effective config evaluation for `world.enabled=false`
    - workspace-presence signal
    - `SUBSTRATE_OVERRIDE_WORLD`
    - tokenized display policy from the source contract
  - Outputs:
    - one normalized classifier result consumable by replay runtime paths
    - winning-layer metadata: layer, env token or path display, `value_display=false`, and unknown-source fallback
    - replay-safe provenance contract with no raw secrets
- **Key invariants / rules**:
  - workspace config beats `SUBSTRATE_OVERRIDE_WORLD` when a workspace exists
  - absolute host paths must never appear
  - raw env values must never appear outside fixed allowlisted tokens
  - replay selection precedence stays unchanged: `--world`, `--no-world`, `SUBSTRATE_REPLAY_USE_WORLD`, recorded origin plus `--flip-world`
  - replay backend selection, timeout behavior, and exit codes stay unchanged
- **Dependencies**
  - Direct blockers:
    - none inside this extracted pack
    - external semantic anchors from the source pack remain assumed-stable until revalidated
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-2`
  - Derived consumers:
    - `SEAM-3`
- **Touch surface**:
  - `crates/shell/src/execution/config_model.rs`
  - replay routing call site in `crates/shell/src/execution/routing/replay.rs`
  - deterministic precedence/redaction coverage in `crates/shell/tests/replay_world.rs`
- **Verification**:
  - This seam produces owned contracts `C-01` and `C-02`. At seam-brief depth, verification is the contract becoming concrete enough for seam-local planning and implementation: named layer outcomes, tokenized display rules, workspace-versus-env precedence, and deterministic tests.
  - Later seam-local verification should prove:
    - replay can query one shared helper without duplicating precedence logic
    - workspace config beats override env when a workspace exists
    - only tokenized paths and allowlisted env tokens are emitted
    - unknown-source fallback stays explicit and deterministic
- **Risks / unknowns**:
  - Risk:
    - helper extraction can accidentally couple replay to unrelated doctor/health rendering logic
  - De-risk plan:
    - keep the seam replay-safe and data-shaped; return normalized attribution metadata rather than rendered prose
  - Risk:
    - unknown-source fallback may be under-specified if provenance cannot be trusted
  - De-risk plan:
    - keep the fallback explicit in the contract and cover it with exact tests
  - Risk:
    - redaction regressions can leak absolute paths or raw env values
  - De-risk plan:
    - add exact-string and negative-string tests at the classifier seam before runtime adoption
- **Rollout / safety**:
  - This seam is internal foundation work only. It should not publish new user-visible behavior on its own.
  - Safety comes from keeping the result shape narrow and from proving that redaction and precedence are deterministic before runtime wiring begins.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `active` because the source pack's `WDRA0` work is the first critical-path dependency for every later runtime and conformance surface.
  - Which threads matter most
    - `THR-01` and `THR-02`
  - What the first seam-local review should focus on
    - API narrowness
    - workspace-versus-env precedence
    - tokenized display semantics
    - exact negative coverage for raw-path and raw-env leaks
    - determinism of the temp-dir based tests
  - Later next-seam planning posture
    - `SEAM-2` should stay provisional until this seam publishes its helper/result contract and provenance semantics.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-01`
    - `C-02`
  - Threads likely to advance:
    - `THR-01` from `defined` to `published`
    - `THR-02` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - provenance resolution path
    - classifier-to-runtime wiring surface
  - Downstream seams most likely to require revalidation:
    - `SEAM-2`
    - `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
- **Source pack anchors**:
  - `pre-planning/minimal_spec_draft.md`
  - `contract.md`
  - `decision_register.md` DR-0001
  - `slices/WDRA0/WDRA0-spec.md`
