# Tasks: Agent Drift Analyzer Session Progress R5.5

Status: draft task ledger created on 2026-06-11 from the post-landing planning-input inventory.
The original `R5.5` packet family is now historically landed on this worktree; later validation on
2026-06-12 kept `R6` closed and moved the remaining pre-`R6` work into the active `R5.75` family.

This ledger remains as the historical record of what `R5.5` landed. It should not be read as the
active next-packet queue; the remaining pre-`R6` work now belongs to `docs/specs/r5/R5_75/`.

Keep each task as close as possible to five touched files or fewer. Do not advance from one packet
to the next until the packet verification commands are green or the failure is explicitly
captured in the packet notes.

## Landed Status Reconciliation (2026-06-12)

Landed under `R5.5` on this worktree:

- `R5.5-0` docs lock and root authority update
- `R5.5-1` troubleshooting repeated-failure hardening
- `R5.5-2` objective extraction hardening
- `R5.5-3` JS/TS verifier-role hardening
- `R5.5-4` real-rollout acceptance corpus deepening
- `R5.5-5` parent-visible progress normalization consistency
- `R5.5-6` delegation limiting evidence promotion
- `R5.5-7` cleanup and doc hygiene

Active remaining pre-`R6` work moved to `R5.75`:

- `R5.75-1` objective condensation / target extraction
- `R5.75-2` sparse readable session fail-open
- `R5.75-3` delegated parent-visible stabilization
- `R5.75-4` zero-verifier anti-flap gating for long exploratory sessions
- `R5.75-5` adapted external robustness fixture family

## R5.5-0: Docs Lock And Root Authority Update

- [x] Task R5.5-0.1: Add the `R5.5` spec.
  - Acceptance: `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md` exists and
    covers objective, commands, project structure, code style, testing strategy, boundaries,
    success criteria, and open questions.
  - Verify: Manual review against
    `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md` and `AGENTS.md`.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md`

- [x] Task R5.5-0.2: Add the `R5.5` plan and task ledger.
  - Acceptance: `plan` and `tasks` docs exist, answer the planning-entry questions, and packetize
    the work into narrow reviewable steps.
  - Verify: Manual review against the planning-input inventory and current R5 doc stack.
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`

- [x] Task R5.5-0.3: Update root landing-order authority for `R5.5`.
  - Acceptance: `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` records `R5` as landed,
    identifies `R5.5` as the pre-`R6` hardening family, and changes the immediate next action
    from `R5` to `R5.5`.
  - Verify: Manual review against the landed `R5` state and
    `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md`.
  - Files:
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`

## R5.5-1: Troubleshooting Repeated-Failure Overclaim Fix

- [x] Task R5.5-1.1: Add a regression for repeated signature plus overlapping edit.
  - Acceptance: a checkpoint regression proves the sequence “same failure -> overlapping edit ->
    same failure” is not classified as `advancing` by itself.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-1.2: Prevent `FailingScopeEdited` from independently authorizing troubleshooting
      advancement.
  - Acceptance: the troubleshooting progress builder still emits `FailingScopeEdited` as
    supporting evidence, but repeated exact or strong-fuzzy signatures with overlapping edits and
    no direct advancement signal resolve conservatively to `mixed` or `stalled`, not `advancing`.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-1.3: Add no-overcorrection guardrail regressions.
  - Acceptance: focused checkpoint regressions prove that:
    - same failure -> overlapping edit -> fewer failing tests => `advancing`
    - compile failure -> overlapping edit -> focused test failure on the same target => `advancing`
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.5-2: Objective Extraction Hardening

- [x] Task R5.5-2.1: Tighten objective-row filtering for boilerplate classes.
  - Acceptance: objective selection filters out more developer/system boilerplate categories than
    the current `AGENTS.md instructions`, `<skill>`, and `Available skills` exclusions while still
    preserving real task objectives.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-2.2: Prefer `/goal`, explicit user requests, and thread-goal text.
  - Acceptance: when boilerplate appears before the true objective, first-checkpoint objective
    selection resolves to the real task goal and comparability stays anchored to that goal.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-2.3: Preserve user-requested boilerplate targets.
  - Acceptance: when the user’s actual task is to edit or analyze an `AGENTS.md` block, skill
    block, or similar instruction scaffold, objective selection preserves that requested target
    instead of filtering it away as boilerplate.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.5-3: JS/TS Verifier-Role Hardening

- [x] Task R5.5-3.1: Expand top-level command-role coverage for npm-like verifier commands.
  - Acceptance: the checkpoint command-role layer classifies the agreed JS/TS verifier matrix
    deterministically, including `npm run lint`, `pnpm run lint`, `yarn lint`, `npm test`,
    `pnpm test`, `vitest ...`, `npx vitest`, and `bun test`.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-3.2: Expand attempt-role coverage for npm-like verifier commands.
  - Acceptance: the attempt classifier treats the same agreed JS/TS verifier matrix as verifier
    attempts instead of generic shell noise.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-3.3: Prove JS/TS verifier attempts affect checkpoint progress.
  - Acceptance: at least one checkpoint-level progress regression shows the improved JS/TS
    classifier contributes comparable verifier evidence rather than leaving the progress result
    underfit.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.5-4: Real-Rollout Acceptance Corpus Deepening

- [x] Task R5.5-4.1: Add the real-fixture annotation rubric.
  - Acceptance: new committed real-rollout cases carry a structured annotation or manifest that
    names expected status, expected dimension, confidence bounds, required and forbidden signals,
    required evidence counts, decisive evidence, counter-evidence, and why alternative dimensions
    were not chosen.
  - Verify: `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
    - `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`

- [x] Task R5.5-4.2: Add one real implementation-progress acceptance case.
  - Acceptance: the committed corpus includes one annotated real rollout fixture expected to hit
    `implementation_verification_wall`.
  - Verify: `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<implementation-case>/**`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

- [x] Task R5.5-4.3: Add one real closeout/review acceptance case.
  - Acceptance: the committed corpus includes one annotated real rollout fixture for
    `verification_closeout_narrowing` or a deliberately conservative closeout status.
  - Verify: `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<closeout-case>/**`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

- [x] Task R5.5-4.4: Add one real reopen/re-verify acceptance case.
  - Acceptance: the committed corpus includes one annotated real rollout fixture that proves
    review findings can reopen work honestly without being forced to remain in closeout; the
    expected dimension may be `verification_closeout_narrowing`,
    `implementation_verification_wall`, or `troubleshooting_frontier`, but the annotation must
    justify the transition and prove the analyzer does not overclaim clean closeout once source or
    verifier work has reopened.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/<reopen-case>/**`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
    - `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`

## R5.5-5: Parent-Visible Progress Normalization Consistency

- [x] Task R5.5-5.1: Route parent-visible progress through the shared normalization order.
  - Acceptance: the `parent_visible_orchestration_progress(...)` path no longer returns a raw
    pre-finalized result, and the shared order is: build candidate -> apply delegation caps and
    limiting counter-evidence -> finalize exactly once.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-5.2: Prove delegated-parent normalization stays conservative.
  - Acceptance: delegated-parent tests show status conservatism is preserved while signal sorting,
    dedupe, and normalized shape match the non-parent-visible hygiene rules.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.5-6: Delegation Limiting Evidence Promotion

- [x] Task R5.5-6.1: Promote limiting evidence into `counter_evidence` when caps suppress claims.
  - Acceptance: when `apply_delegation_caps(...)` lowers confidence or denies a stronger status,
    the same limiting context becomes visible as `counter_evidence` in addition to limiting signals.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-6.2: Add delegated-partial and delegated-opaque regressions.
  - Acceptance: tests assert capped confidence, visible limiting signal, and visible limiting
    evidence in `counter_evidence` for delegated partial/opaque cases, and the finalizer does not
    remove that counter-evidence.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.5-7: Cleanup And Doc Hygiene

- [x] Task R5.5-7.1: Remove or justify `#![allow(dead_code)]` in `diagnostics.rs`.
  - Acceptance: the attribute is removed after wiring/deleting dead code, or the file/documentation
    explains the remaining intentional staging clearly enough that the allow is no longer a silent
    trust gap.
  - Verify:
    - Manual code review of `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
    - Commit `323600c22 chore: clean r5.5 diagnostics and task doc hygiene`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.5-7.2: Clean the stale unchecked legacy checklist text from the historical R5 task
      doc.
  - Acceptance: `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` no longer reads
    like unfinished R5 implementation work while still preserving honest history.
  - Verify:
    - Manual review against current landed R5 state and this R5.5 plan
    - Commit `323600c22 chore: clean r5.5 diagnostics and task doc hygiene`
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-follow-ups.md`

## Deferred / Ask-First (Historical, Not Active Pre-`R6` Debt)

- [ ] Task R5.5-X.1: Introduce a named `ProgressWindow` seam only if the primary packets cannot be
      expressed cleanly in the current structure.
  - Acceptance: a separate ask-first approval explicitly widens `R5.5` into design-deepening work,
    and the new seam improves explainability without reopening scorer/sentinel scope.
  - Verify: to be defined only if this packet is approved.
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - any new helper module approved during the ask-first review

## R6 Readiness Gate

Do not open `R6` scorer cutover until:

- the landed `R5.5` hardening baseline remains green and historically intact
- `R5.75` closes the remaining pre-`R6` follow-on gaps
- `cargo test -p agent-drift-analyzer -- --nocapture` is green
- any touched sentinel spot-checks are green
- `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` identifies `R6` as next only after `R5.75`
