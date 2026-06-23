# Tasks: Agent Drift Analyzer Zero-Verifier Anti-Flap Gate (R5.75-4)

Status: draft task ledger created on 2026-06-23 after `R5.75-3` was promoted and the live
`docs/specs/r5/R5_75/MAP.md` named `R5.75-4` as the active seam.

Keep each task narrow, reviewable, and scoped to zero-verifier anti-flap gating. Do not widen into
adapted committed fixture-family work (`R5.75-5`), structured-goal consumer wiring (`R5.75-6`), or
scorer / `dead_end_thrash` redesign (`R6`).

Packet prerequisite rule: this packet names `R5.75-3` as landed. Verify that in live code/tests before
editing. If the prerequisite is missing, stop and report it instead of compensating inside `R5.75-4`.

## R5.75-4.0: Docs Lock

- [x] Task R5.75-4.0.1: Commit the SPEC/PLAN/TASKS family for `R5.75-4`.
  - Acceptance: `docs/specs/r5/R5_75/R5_75-4/` contains the spec, plan, and this tasks ledger, and
    they record:
    - the analyzer-local packet boundary,
    - the explicit adapted smoke witnesses (`097d97e914ca220f`, `da59436e63915185`),
    - the rule that structured `primary_intent` stays observational-only here,
    - the prohibition on widening into `R5.75-5`, `R5.75-6`, or `R6`, and
    - the fact that packet prompts are out of scope for this docs pass.
  - Verify: manual review against the `R5.75-4` packet in `docs/specs/r5/R5_75/MAP.md` and the live
    analyzer seams in `crates/agent-drift-analyzer/src/checkpoint/progress.rs`,
    `crates/agent-drift-analyzer/tests/checkpoints.rs`, and
    `crates/agent-drift-analyzer/tests/progress_acceptance.rs`.
  - Files:
    - `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-spec.md`
    - `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-plan.md`
    - `docs/specs/r5/R5_75/R5_75-4/agent-drift-analyzer-zero-verifier-anti-flap-gate-tasks.md`
  - Closeout note (2026-06-23): the docs-only SPEC/PLAN/TASKS authority lock is committed for
    `R5.75-4`, with the analyzer-local boundary, explicit adapted smoke witnesses, observational-only
    structured `primary_intent` rule, packet-boundary prohibitions, and packet-prompt exclusion all
    preserved exactly for the pre-implementation pass.

## R5.75-4.1: Characterize The Canonical Zero-Verifier And Mixed Delegated/Exploratory Adapted Repros

- [x] Task R5.75-4.1.1: Run the adapted smoke witnesses and record exact packet-local expectations
      before changing code.
  - Acceptance: this ledger records the current/expected lane, progress status, confidence band, and
    decisive/counter-evidence expectations for:
    - `097d97e914ca220f`
    - `da59436e63915185`
    and explicitly distinguishes:
    - `097d97e914ca220f` as the packet's canonical long exploratory zero-verifier witness, and
    - `da59436e63915185` as the mixed delegated/exploratory witness that must preserve the conservative
      `R5.75-3` parent-visible bar while losing any unrelated stronger overclaim.
  - Verify: run the two adapted smoke commands from the companion spec and inspect `summary.md` plus the
    relevant `checkpoints.jsonl` rows; record exact checkpoint ordinals if the overclaim localizes to a
    specific interval range.
  - Files:
    - no committed implementation files required; evidence recorded in this ledger and under
      `target/r5_75-smoke/R5.75-4/<session-id>/`
  - Closeout note (2026-06-23): prerequisite `R5.75-4.0` was confirmed landed first in live history
    (`15724b12a`, `c0f2df9c7`, `79dedb3cd`) before the adapted smoke reruns. The packet-local
    expectations recorded from the live smoke are:
    - `097d97e914ca220f` (canonical long exploratory zero-verifier witness):
      - observed current behavior: `summary.md` reports `verification_density=0.00`, `write=0`,
        `progress dimension distribution: troubleshooting_frontier=2, planning_convergence=6`, and
        the overclaim localizes to checkpoints `0007-0008`.
      - decisive checkpoints:
        - `0005-0006` are the honest conservative baseline: lane `planning_convergence`, status
          `stalled`, confidence `medium`, with decisive support
          `earlier planning working set was more focused` +
          `planning working set expanded instead of narrowing`, and decisive counter-evidence
          `repeated broad planning scan without convergence artifact`.
        - `0007-0008` are the current packet-owned overclaim: lane `troubleshooting_frontier`, status
          `insufficient_evidence`, confidence `low`, with troubleshooting archetype confidence
          `high -> low` driven by repeated failure evidence / renewed inspection despite zero verifier
          density and no source edits.
      - packet-local expectation to preserve in `R5.75-4`: keep this witness on the conservative
        planning lane (`planning_convergence`, low-to-medium confidence, `insufficient_evidence` or
        `stalled`) unless future checkpoints show decisive verifier-backed, concrete source-edit, or
        explicit failure proof stronger than the current broad-scan counter-evidence.
    - `da59436e63915185` (mixed delegated/exploratory witness):
      - observed current behavior: `summary.md` reports `verification_density=0.00`,
        `progress dimension distribution: troubleshooting_frontier=7, planning_convergence=15,
        implementation_verification_wall=1, parent_visible_orchestration=5`, and the packet-owned
        overclaim localizes to checkpoints `0017-0023` before the late delegated-parent-visible band.
      - decisive checkpoints:
        - `0017-0023` are the exploratory overclaim to remove: lane `troubleshooting_frontier`,
          status `insufficient_evidence`, confidence `low`, with dead-end-thrash history
          `30 -> 20 historical_only` driven by repeated failure evidence despite zero verifier density
          and no direct child-proof progress.
        - `0024-0028` are the `R5.75-3` bar to preserve: lane `parent_visible_orchestration`,
          confidence `low`, statuses `insufficient_evidence` (`0024`, `0025`, `0027`) and
          `stalled` (`0026`, `0028`). The decisive support/counter-evidence here is the existing
          child-opaque delegation language:
          `only parent-visible orchestration evidence was available for progress assessment` versus
          `child-opaque delegation limited direct child progress claims` and
          `delegation visibility limited progress confidence`.
      - packet-local expectation to preserve in `R5.75-4`: remove the earlier exploratory
        troubleshooting-frontier stretch, but keep the late delegated interval on the conservative
        `parent_visible_orchestration` lane with low confidence and the same child-opaque delegation
        limiting evidence rather than upgrading it to a stronger troubleshooting or implementation
        posture.

## R5.75-4.2: Land The Analyzer-Local Anti-Flap Guard

- [ ] Task R5.75-4.2.1: Add the minimal analyzer-local rule that keeps zero-verifier exploratory
      sessions conservative when decisive evidence is absent.
  - Acceptance: `crates/agent-drift-analyzer/src/checkpoint/progress.rs` prefers a conservative
    `planning_convergence` / `insufficient_evidence` posture for long browse/read/tool-output-heavy
    intervals when verifier attempts, concrete source-edit progress, and explicit failure evidence are
    absent. The change:
    - remains additive and packet-local,
    - does not wire structured `primary_intent` into the decision,
    - does not retune scorer / `dead_end_thrash` logic,
    - does not fabricate stronger progress claims for `097d97e914ca220f`, and
    - preserves conservative `parent_visible_orchestration` behavior for `da59436e63915185` where
      delegation evidence still justifies it.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - targeted rerun of the two adapted smoke witnesses
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.75-4.3: Fast Checkpoint Regressions

- [ ] Task R5.75-4.3.1: Add or refresh fast regressions that prove both conservatism and non-regression.
  - Acceptance: `crates/agent-drift-analyzer/tests/checkpoints.rs` proves all of the following:
    - zero-verifier broad-scan / read-output exploratory sessions stay low-confidence and conservative,
    - real verifier-backed or explicit-failure-backed troubleshooting still advances when it should,
    - the packet does not erase delegated parent-visible semantics that `R5.75-3` already landed, and
    - the anti-flap rule explains the missing proof through limiting/counter-evidence instead of merely
      muting output.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`

## R5.75-4.4: Bounded Semantic Acceptance Proof (Only If Needed)

- [ ] Task R5.75-4.4.1: Add a bounded `progress_acceptance` proof only if fast regressions alone would
      leave the packet without an honest committed semantic wall.
  - Acceptance: if needed, the packet adds or refines one bounded semantic proof in
    `crates/agent-drift-analyzer/tests/progress_acceptance.rs` while keeping the corpus honest:
    - no adapted committed fixture-family expansion lands here,
    - corpus counts/exclusions remain truthful,
    - the proof stays packet-local to zero-verifier anti-flap behavior, and
    - the packet does not silently start `R5.75-5`.
    If no such update is needed, close this task with an explicit note explaining why the packet's proof
    wall is already sufficient without it.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - one existing bounded progress-acceptance fixture directory or README entry only if the packet
      truly needs it

## R5.75-4.5: Automated Validation Gates

- [ ] Task R5.75-4.5.1: Run the packet's automated validation wall.
  - Acceptance:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` is green
    - `cargo test -p agent-drift-analyzer -- --nocapture` is green
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - no additional implementation files; verification-only step

## R5.75-4.6: Adapted Smoke Review

- [ ] Task R5.75-4.6.1: Re-run canonical zero-verifier witness `097d97e914ca220f` and mixed delegated/exploratory witness `da59436e63915185`, then confirm the packet-owned
      conservative bar holds at live smoke.
  - Acceptance:
    - `097d97e914ca220f` stays boring/conservative and does not surface troubleshooting-frontier or
      equivalent strong failure posture without decisive evidence,
    - `da59436e63915185` preserves the conservative delegated parent-visible bar from `R5.75-3` where
      delegation evidence exists, while no longer flapping into unrelated stronger failure posture, and
    - the packet documents any remaining non-packet-owned issue honestly instead of widening scope.
  - Verify: rerun the adapted smoke harness from the companion spec for both sessions and inspect
    `summary.md` plus the relevant `checkpoints.jsonl` rows, not just command exit status.
  - Files:
    - no committed implementation files required; evidence recorded under
      `target/r5_75-smoke/R5.75-4/<session-id>/`

## R5.75-4.7: MAP Promotion And Routing Update

- [ ] Task R5.75-4.7.1: Update the main `R5.75` routing hub only after the packet has actually closed.
  - Acceptance: `docs/specs/r5/R5_75/MAP.md` is updated only after Tasks `R5.75-4.1` through
    `R5.75-4.6` are complete and records all of the following honestly:
    - `R5.75-4` is promoted history,
    - canonical zero-verifier exploratory witness `097d97e914ca220f` and mixed delegated/exploratory witness `da59436e63915185` used for promotion,
    - the packet-local anti-flap result,
    - any bounded `progress_acceptance` proof that landed, and
    - `R5.75-5` is now the active next seam.
  - Verify: manual review of `docs/specs/r5/R5_75/MAP.md` against the green automated gates and the
    saved smoke outputs under `target/r5_75-smoke/R5.75-4/`.
  - Files:
    - `docs/specs/r5/R5_75/MAP.md`
