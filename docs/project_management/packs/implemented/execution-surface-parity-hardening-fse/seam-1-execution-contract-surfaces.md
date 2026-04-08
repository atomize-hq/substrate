---
seam_id: SEAM-1
seam_slug: execution-contract-surfaces
type: integration
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - world-network routing semantics change in the shell or world-agent path
    - replay request construction changes without updating the four-case routing matrix
    - SUBSTRATE_ENABLE_PREEXEC forwarding, builtin telemetry generation, or canonical trace omission rules change
    - the active world_process_exec_tracing_parity playbook changes Case B expectations outside this pack
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
  planned_location: S99
  status: passed
open_remediations: []
---

# SEAM-1 - Execution contract surfaces

- **Goal / value**:
  Publish the authoritative replay-routing contract and tracing-validation behavior matrix so later work stops depending on local heuristics or ambiguous playbook assertions.
- **Scope**
  - In:
    - align replay request construction with the canonical world-network contract for both local and agent-backed replay
    - decide and publish the mode-by-platform behavior matrix for `world_process_*`, `builtin_command`, and `SUBSTRATE_ENABLE_PREEXEC`
    - update the authoritative validation guidance so WPEP Case B asserts the chosen semantics rather than a misleading proxy
    - identify any minimal shared helper extraction needed to keep replay thin and contract-driven
  - Out:
    - interactive REPL terminal-loss runtime handling
    - world-agent or shim architectural changes
    - broad trace-schema redesign beyond the surfaces needed to publish the contract
- **Primary interfaces**
  - Inputs:
    - effective `world.net.filter` and canonical `policy_snapshot.net_allowed`
    - replay request-construction code paths in shell and replay crates
    - `SUBSTRATE_ENABLE_PREEXEC`, builtin handling, and current WPEP smoke / manual validation semantics
  - Outputs:
    - published four-case replay-routing contract
    - explicit execution-mode behavior matrix and Case B validation contract
- **Key invariants / rules**:
  - replay must not derive routing from replay-only heuristics when a canonical policy-snapshot contract already exists
  - canonical trace remains safe-by-default and must not persist raw builtin or preexec command bodies
  - the chosen behavior matrix must name whether `SUBSTRATE_ENABLE_PREEXEC` is supported, best-effort, or intentionally ignored in each mode
  - docs and playbooks must validate the same contract that runtime code publishes
- **Dependencies**
  - Direct blockers:
    - existing policy-snapshot and world-network semantics in the normal shell/world execution path
    - authoritative trace-safety posture already documented in the repo
  - Transitive blockers:
    - downstream smoke and doc surfaces that currently encode the wrong or incomplete behavior
    - any later ADR decision if the team elects to add a first-class preexec control lever
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - replay-related follow-on packs
    - the active world-process tracing parity pack
    - operator-facing replay and trace documentation
- **Touch surface**:
  `crates/replay/src/replay/executor.rs`, `crates/shell/src/execution/policy_snapshot.rs`, `crates/shell/src/scripts/bash_preexec.rs`, `crates/shell/src/execution/manager.rs`, `crates/shell/src/execution/routing/dispatch/exec.rs`, `docs/REPLAY.md`, `docs/TRACE.md`, `docs/internals/env/inventory.md`, and `docs/project_management/packs/active/world_process_exec_tracing_parity/`.
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - For this seam specifically: seam-local review should prove the replay four-case matrix is shared across local and agent-backed execution, the behavior matrix is explicit by mode and platform, and the updated Case B assertion validates the chosen trace contract without leaking command bodies.
- **Risks / unknowns**:
  - Risk: the seam couples a straightforward replay fix to a broader preexec decision and becomes too large.
  - De-risk plan: keep the seam contract-first, publish the minimal decision record needed for preexec semantics, and reserve any larger config-surface change for a later follow-on if evidence demands it.
  - Risk: helper extraction crosses ownership boundaries between shell and replay crates.
  - De-risk plan: prefer the smallest shared helper that preserves one authoritative source of routing truth.
  - Risk: the active WPEP pack keeps asserting the old behavior while runtime code changes underneath it.
  - De-risk plan: treat the playbook and smoke assertion update as part of the same owned contract publication, not optional cleanup.
- **Rollout / safety**:
  No new public CLI or config surface is assumed here. The seam should reduce security-sensitive routing drift and validation ambiguity while preserving the safe-by-default trace posture.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `future` only because it has landed and left the forward planning window after publishing `C-01`, `C-02`, and `THR-01`.
  - Which threads matter most: `THR-01`
  - What the first seam-local review should focus on: whether the routing contract and behavior matrix are scoped tightly enough to decompose cleanly, and whether the chosen Case B assertion actually tests the intended safety posture
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`, `C-02`
  - Threads likely to advance: `THR-01`
  - Review-surface areas likely to shift after landing: replay routing flow, tracing behavior matrix, and WPEP validation surfaces
  - Downstream seams most likely to require revalidation: `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
