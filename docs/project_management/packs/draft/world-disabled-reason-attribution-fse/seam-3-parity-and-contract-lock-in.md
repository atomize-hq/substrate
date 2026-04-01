---
seam_id: SEAM-3
seam_slug: parity-and-contract-lock-in
type: conformance
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - published runtime fragments change after docs or smoke expectations are drafted
    - telemetry field names or omission rules change after trace docs and tests are updated
    - required platform list or allowed platform divergences change
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

# SEAM-3 - Parity and contract lock-in

- **Goal / value**:
  - Prevent drift after runtime publication by locking replay tests, docs, smoke wrappers, manual validation, and Linux/macOS/Windows parity to the same contract.
  - Turn the runtime behavior into stable review evidence and durable regression coverage.
- **Scope**
  - In:
    - replay regression coverage for override env, workspace config, global config, unknown-source fallback, and replay-local opt-out omission rules
    - alignment of `docs/REPLAY.md`, `docs/TRACE.md`, and `docs/COMMANDS.md` with the final runtime contracts
    - Linux/macOS/Windows smoke-wrapper expectations
    - manual testing playbook alignment with the same test filters and expected assertions
  - Out:
    - new replay selection semantics
    - new trace event types
    - changes to ADR-0037 semantics
    - foundation helper redesign or new runtime behavior publication
- **Primary interfaces**
  - Inputs:
    - published provenance/redaction contract `C-02`
    - published replay copy contract `C-03`
    - published telemetry contract `C-04`
    - required platform list and permitted divergences from the source parity spec
  - Outputs:
    - regression lock-in in `crates/shell/tests/replay_world.rs`
    - docs examples aligned to final fragments and field names
    - smoke wrappers and manual playbook aligned to the same assertions
    - parity evidence for Linux, macOS, and Windows
- **Key invariants / rules**:
  - tests must cover override env, workspace config, global config, and unknown-source redaction paths
  - replay-local opt-out cases must keep their existing fragments and omit `world_disable_source`
  - docs must match `contract.md` and `telemetry-spec.md`
  - smoke wrappers and the manual playbook must reference the same expected filters and assertions
  - allowed platform divergence stays limited to non-fragment transport or backend details
- **Dependencies**
  - Direct blockers:
    - `SEAM-2`
    - `THR-03`
    - `THR-04`
  - Transitive blockers:
    - `SEAM-1`
    - `THR-02`
  - Direct consumers:
    - none inside this feature's runtime path
  - Derived consumers:
    - future replay maintainers
    - release reviewers
    - downstream packs that rely on replay docs or trace examples
- **Touch surface**:
  - `crates/shell/tests/replay_world.rs`
  - `docs/REPLAY.md`
  - `docs/TRACE.md`
  - `docs/COMMANDS.md`
  - `manual_testing_playbook.md`
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
- **Verification**:
  - This seam consumes upstream contracts, so verification may depend on accepted upstream evidence for `C-02`, `C-03`, and `C-04`.
  - Later seam-local verification should prove:
    - the same reason fragments and telemetry fields are asserted in tests, docs, smoke wrappers, and the manual playbook
    - Linux, macOS, and Windows emit the same `origin_reason_code` values and tokenized displays for equivalent cases
    - replay-local opt-out cases keep their historical fragments and omit `world_disable_source`
- **Risks / unknowns**:
  - Risk:
    - docs and regression tests may drift from the final runtime publication if they are drafted too early
  - De-risk plan:
    - keep this seam future until `SEAM-2` publishes and then revalidate against recorded upstream closeouts
  - Risk:
    - platform-local backend behavior may add noise that obscures the actual attribution assertions
  - De-risk plan:
    - keep smoke assertions feature-local and backend-agnostic outside the required reason fragments and telemetry values
  - Risk:
    - parity evidence may be incomplete if smoke wrappers and the manual playbook diverge
  - De-risk plan:
    - lock the same test filters and expected outputs across all validation artifacts
- **Rollout / safety**:
  - This seam should not introduce new runtime semantics.
  - Its safety role is to close drift loops, preserve redaction guarantees, and make the feature reviewable across platforms.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`
    - `next` because it now sits immediately behind the active replay-runtime seam and should wait on published `C-03` / `C-04` truth before becoming active.
  - Which threads matter most
    - `THR-03`
    - `THR-04`
    - plus upstream provenance contract thread `THR-02`
  - What the first seam-local review should focus on
    - coverage completeness for all four attribution cases plus replay-local opt-out omission rules
    - docs parity against final fragments and field names
    - smoke-wrapper/platform scope
    - closeout evidence shape for parity and redaction
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none net-new; the main work is revalidation and lock-in of published runtime contracts
  - Threads likely to advance:
    - `THR-02` from `defined` or `published` to `revalidated`
    - `THR-03` from `published` to `revalidated` or `closed`
    - `THR-04` from `published` to `revalidated` or `closed`
  - Review-surface areas likely to shift after landing:
    - docs examples
    - smoke evidence matrix
    - manual validation narrative
  - Downstream seams most likely to require revalidation:
    - future replay or diagnostics work that consumes replay stderr or `replay_strategy` examples
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
- **Source pack anchors**:
  - `platform-parity-spec.md`
  - `manual_testing_playbook.md`
  - `pre-planning/ci_checkpoint_plan.md`
  - `slices/WDRA2/WDRA2-spec.md`
  - `quality_gate_report.md`
