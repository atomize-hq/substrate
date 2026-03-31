---
seam_id: SEAM-1
seam_slug: doctor-text-disable-attribution
type: capability
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - effective-config precedence for world.enabled changes
    - exact doctor disable-attribution message bodies change
    - disabled-status UX work changes doctor framing or status semantics
    - platform doctor renderers bypass tokenized path or env redaction
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

# SEAM-1 - Doctor text disable attribution

- **Source pack mapping**: corresponds to `DHO0` in the deep-researched source plan.
- **Goal / value**: make `substrate host doctor` and `substrate world doctor` state the real disable source immediately so operators stop debugging the wrong layer.
- **Scope**
  - In:
    - doctor text output for host and world doctor
    - shared provenance-backed disable-attribution mapping from effective `world.enabled` winner to exact message body
    - CLI/env/workspace/global/default/fallback wording and redaction invariants
    - Linux/macOS parity for host doctor and Linux/macOS/Windows parity for world doctor
  - Out:
    - additive JSON fields
    - health text or health JSON
    - replay-warning reuse, provisioning guidance, or exit-code changes
    - adding Windows host-doctor support
- **Primary interfaces**
  - Inputs:
    - CLI flags `--world` / `--no-world`
    - `<workspace>/.substrate/workspace.yaml`
    - `SUBSTRATE_OVERRIDE_WORLD`
    - `$SUBSTRATE_HOME/config.yaml`
    - default config layer
    - effective-config explain provenance for `world.enabled`
  - Outputs:
    - exact doctor message bodies in `C-01`
    - shared disable-attribution model and precedence truth in `C-02`
    - stable fallback path `source unknown` when safe winner attribution is unavailable
- **Key invariants / rules**:
  - attribution must use the same effective winner that resolves `world.enabled`
  - when world is disabled, doctor text emits exactly one contract message body; when enabled, it emits none
  - workspace patch beats the override env because env applies only when no workspace exists
  - raw env values and absolute host paths never appear; only the fixed safe env token and tokenized display paths may be rendered
  - misattribution is worse than omission; fallback is mandatory when winner proof is unsafe
- **Dependencies**
  - Direct blockers:
    - effective-config explain provenance must be available at doctor entrypoints
    - concurrent disabled-status UX work must not redefine this seam's contract while it is being planned
  - Transitive blockers:
    - future policy/config terminology migration may rename layers without changing the actual precedence
    - future JSON envelope work could indirectly change doctor framing and force string-set revalidation
  - Direct consumers:
    - `SEAM-2`
    - operators and support staff reading doctor output
  - Derived consumers:
    - health output through `SEAM-2`
    - future replay-warning reuse of the same attribution vocabulary
- **Touch surface**:
  - `crates/shell/src/execution/world_disable_attribution.rs`
  - `crates/shell/tests/world_disable_attribution.rs`
  - `crates/shell/src/execution/platform/mod.rs`
  - `crates/shell/src/execution/platform/linux.rs`
  - `crates/shell/src/execution/platform/macos.rs`
  - `crates/shell/src/execution/platform/windows.rs`
  - source-plan contract, decision register, `DHO0` spec, manual playbook, and smoke evidence surfaces
- **Verification**:
  - This seam **produces** the doctor-facing UX contract and shared disable-attribution model.
  - At seam-brief depth, readiness is that the exact string set, precedence truth, fallback behavior, and redaction tokens are concrete enough for seam-local planning and implementation.
  - Downstream seam-local review should verify winner-to-message mapping tests, enabled-case omission, CLI/env/workspace/global/default/fallback cases, and platform parity where the doctor surface exists.
- **Risks / unknowns**:
  - Risk: a doctor entrypoint cannot surface provenance cleanly and falls back to a local heuristic.
    - De-risk plan: centralize mapping in a shared helper and require `source unknown` instead of any guessed winner.
  - Risk: platform-specific doctor renderers diverge in wording or leak absolute paths.
    - De-risk plan: bind all renderers to the same string table and run Linux/macOS/Windows smoke parity plus redaction assertions.
  - Risk: external disabled-status UX work edits surrounding framing and obscures the exact message body.
    - De-risk plan: treat message bodies as exact contract text and revalidate `THR-02` whenever framing changes.
- **Rollout / safety**:
  - additive messaging only; no change to enablement semantics or exit codes
  - Windows host doctor remains unsupported and must stay unsupported
  - any failure to prove winner attribution degrades to explicit `source unknown` rather than silently lying
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because the source plan sequences it first and all later health/JSON work depends on its exact contract truth
  - Which threads matter most: `THR-01` carries the shared attribution model; `THR-02` protects message-body parity into health
  - What the first seam-local review should focus on: provenance plumbing at doctor entrypoints, exact string set, fallback safety, and cross-platform framing differences
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`, `C-02`
  - Threads likely to advance: `THR-01` and `THR-02` should move from `defined` toward `published`
  - Review-surface areas likely to shift after landing: doctor render framing, tokenized path display, and where the attribution line sits relative to disabled-status summary text
  - Downstream seams most likely to require revalidation: `SEAM-2` first, then any later health/JSON or replay-warning consumer that reuses this contract
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
