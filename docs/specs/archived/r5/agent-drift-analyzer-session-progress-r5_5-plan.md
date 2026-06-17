# Plan: Agent Drift Analyzer Session Progress R5.5

Status: draft plan created on 2026-06-11 from the post-landing planning-input inventory. The
original `R5.5` packet family is now historically landed on this worktree, but follow-on
validation kept `R6` closed and moved the remaining pre-`R6` hardening into the active `R5.75`
family on 2026-06-12.

## Objective

Land a bounded post-`R5` hardening family that fixes the highest-confidence analyzer-semantic
issues, deepens the real acceptance wall where it is still thin, and cleans up the most misleading
residual hygiene debt without reopening the full `R5` design space.

## Landed Status Reconciliation (2026-06-12)

Historical `R5.5` landings preserved by this plan:

- `R5.5-0` docs lock and root authority update
- `R5.5-1` troubleshooting repeated-failure hardening
- `R5.5-2` objective extraction hardening
- `R5.5-3` JS/TS verifier-role hardening
- `R5.5-4` real-rollout acceptance corpus deepening
- `R5.5-5` parent-visible progress normalization consistency
- `R5.5-6` delegation limiting evidence promotion
- `R5.5-7` cleanup and doc hygiene

Remaining pre-`R6` work is no longer tracked as open `R5.5` packet debt. The active follow-on
family is now `R5.75`, which carries the narrower post-validation gaps:

- `R5.75-1` objective condensation / target extraction
- `R5.75-2` sparse readable session fail-open
- `R5.75-3` delegated parent-visible stabilization
- `R5.75-4` zero-verifier anti-flap gating for long exploratory sessions
- `R5.75-5` adapted external robustness fixture family

## Planning Decisions Locked For This Draft

1. `R5.5` is an explicit post-landing follow-up family. It does not rewrite the historical fact
   that the original `R5` landing plan ended at `R5-7`.
2. The family stays analyzer-owned by default. Supporting doc and fixture updates are in scope;
   broad sentinel/runtime/scorer changes are not.
3. `R5.5-FU-001` is the first landing gate because it is the only clearly identified P0 issue that
   can make troubleshooting progress unsafe for later scorer consumption.
4. `R5.5-FU-002`, `R5.5-FU-003`, and `R5.5-FU-004` are the core trust-building follow-ons and
   should land before lower-priority cleanup packets.
5. `R5.5-FU-005` and `R5.5-FU-006` stay separate from the P0/P1 non-delegated core so
   delegated-progress normalization can be reviewed independently.
6. `R5.5-FU-007` is deferred by default. If it becomes necessary, it should be treated as an
   ask-first design-deepening packet rather than silently folded into semantic bug fixes.

## Recommended Answers To The Planning-Input Entry Questions

### 1. Which items are true correctness blockers vs. acceptance-depth gaps?

- **True correctness / honesty blockers:**
  - `R5.5-FU-001` repeated-failure troubleshooting overclaim
  - `R5.5-FU-002` objective pollution when boilerplate wins over the real goal
  - `R5.5-FU-004` incomplete JS/TS verifier-attempt coverage when it suppresses real verifier
    evidence
  - `R5.5-FU-005` parent-visible early return bypassing final normalization
  - `R5.5-FU-006` missing counter-evidence promotion when delegation caps suppress stronger claims
- **Acceptance-depth gap:**
  - `R5.5-FU-003` real-rollout corpus thinness
- **Cleanup / design follow-up:**
  - `R5.5-FU-007`
  - `R5.5-FU-008`
  - `R5.5-FU-009`

### 2. Does `R5.5` stay analyzer-only?

Yes, by default. The planned code changes are analyzer-local. Supporting fixture/docs updates are
part of the analyzer acceptance wall. Sentinel code should only change if new replay/operator
fixtures prove a concrete compatibility or presentation gap.

### 3. Which real-rollout cases should become committed progress fixtures?

At minimum:

1. one real implementation-progress case expected to hit `implementation_verification_wall`,
2. one real closeout/review case expected to hit `verification_closeout_narrowing` or a deliberate
   conservative closeout status,
3. one real review-findings/reopen/re-verify case expected to prove honest reopen behavior.

### 4. How narrow should the objective-pollution fix stay?

Keep it narrow to objective-row filtering, row priority, and first-checkpoint objective selection.
Do not rewrite general phase segmentation or checkpoint construction unless live tests show the
narrow fix cannot hold.

### 5. Should delegation consistency land together with P0/P1 fixes?

No. Keep delegation normalization as separate reviewable packets after the core trust-building work
lands.

## Implementation Principles

1. Fix the highest-risk false positive before expanding acceptance claims.
2. Prefer analyzer-local rule hardening over public contract or schema changes.
3. Keep each packet reviewable and bounded to a narrow seam.
4. Grow the real acceptance wall only after the underlying semantic or classifier rule is stable.
5. Treat delegated-progress honesty as important but separable from the core non-delegated trust
   fixes.
6. Defer design-deepening refactors unless the correctness work proves they are necessary.

## Dependency Graph

```text
R5.5 docs lock
  -> troubleshooting overclaim fix
  -> objective extraction hardening
  -> JS/TS verifier-role hardening

troubleshooting overclaim fix
objective extraction hardening
JS/TS verifier-role hardening
  -> real-rollout corpus deepening

parent-visible normalization
  -> delegation limiting-evidence promotion

cleanup / doc hygiene after core packets are stable

Deferred by default:
  progress-window named seam
```

Notes:

- `R5.5-4` depends on `R5.5-1`, `R5.5-2`, and `R5.5-3` producing stable semantics worth freezing in
  committed fixtures.
- `R5.5-5` and `R5.5-6` can be developed after the core analyzer semantics are understood; they do
  not need to block real implementation/review corpus growth unless a delegated case is chosen for
  the added real fixture set.

## Historical Packet Split

The packet descriptions below preserve what `R5.5` landed. They are no longer the active
pre-`R6` implementation queue; later remaining work now routes through `docs/specs/r5/R5_75/`.

## R5.5-0: Docs Lock And Root Authority Update

### Scope

- Add `R5.5` spec, plan, and tasks docs.
- Update the repo-root landing-order authority so it records `R5` as landed and `R5.5` as the
  pre-`R6` hardening family.
- Explicitly record that this is a post-landing follow-up family, not a rewrite of the original R5
  packet history.

### Files

```text
docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md
```

### Verification

Manual review only.

## R5.5-1: Troubleshooting Repeated-Failure Overclaim Fix

### Scope

- Harden troubleshooting progress so repeated exact/strong-fuzzy failures after overlapping edits do
  not produce `advancing` by `FailingScopeEdited` alone.
- Define the direct advancement signal set explicitly so supporting/contextual signals cannot
  authorize `advancing` on their own.
- Add no-overcorrection guardrails for reduced-failure-count and later-stage-failure advancement.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/progress.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R5.5-2: Objective Extraction Hardening

### Scope

- Tighten objective-row filtering and precedence so `/goal`, explicit user requests, and thread-goal
  text outrank developer/system boilerplate.
- Use an explicit candidate-priority model rather than only accumulating more exclusion strings.
- Preserve user-requested boilerplate targets when the user is actually asking to edit or analyze
  those instruction blocks.
- Keep the change narrow to task-frame inference.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R5.5-3: JS/TS Verifier-Role Hardening

### Scope

- Harden command-role and attempt-role classification for npm/pnpm/yarn/vitest-style verifier
  commands.
- Keep the command-role and attempt-role matrices aligned across the planned JS/TS verifier set,
  including the low-cost `bun test` surface.
- Prove the improved classifier changes checkpoint-level progress evidence, not just low-level role
  enums.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/src/checkpoint/attempt.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R5.5-4: Real-Rollout Acceptance Corpus Deepening

### Scope

- Add bounded committed real fixtures for implementation advancement, closeout/review narrowing,
  and reopen/re-verify honesty.
- Add a fixture annotation rubric so each committed real case carries explicit expected-status,
  expected-dimension, signal, and evidence requirements.
- Update fixture authority docs so the new corpus is explicit and reviewable.

### Files

```text
crates/agent-drift-analyzer/tests/progress_acceptance.rs
crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**
docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md
```

### Verification

```bash
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Notes

If the new fixture shape crosses replay/operator assumptions, add the minimal sentinel spot-checks,
but do not preemptively widen into sentinel code changes.

## R5.5-5: Parent-Visible Progress Normalization Consistency

### Scope

- Route `parent_visible_orchestration_progress(...)` through `finalize_progress(...)`.
- Keep the normalization order explicit: build candidate -> apply delegation caps and limiting
  counter-evidence -> finalize once.
- Preserve conservative delegated-parent status while gaining shared normalization behavior.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/progress.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R5.5-6: Delegation Limiting Evidence Promotion

### Scope

- When delegation caps suppress a stronger claim or lower confidence, promote the limiting evidence
  into `counter_evidence` as well as limiting signals.
- Preserve that limiting counter-evidence through finalization.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/progress.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R5.5-7: Cleanup And Doc Hygiene

### Scope

- Remove or explicitly justify `#![allow(dead_code)]` in `diagnostics.rs`.
- Clean the stale unchecked legacy checklist text from the historical R5 tasks doc.

### Files

```text
crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs
docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md
docs/specs/r5/agent-drift-analyzer-session-progress-r5-follow-ups.md
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

### Notes

Doc cleanup should happen only after the core packets are stable so the history stays honest while
implementation is still in motion.

Repo state note on 2026-06-11: this cleanup packet is already landed via commit
`323600c22 chore: clean r5.5 diagnostics and task doc hygiene`.

## R6 Readiness Gate

Do not open `R6` scorer work until:

- the landed `R5.5` hardening baseline remains green and historically intact
- `R5.75` closes the remaining pre-`R6` follow-on gaps
- `cargo test -p agent-drift-analyzer -- --nocapture` is green
- any touched sentinel spot-checks are green
- the root landing-order authority names `R6` as next only after `R5.75`

## Deferred / Ask-First Packet: ProgressWindow Named Seam

### Scope

- Reify the currently implicit progress window into a named internal seam only if the core `R5.5`
  fixes become too hard to express cleanly with the current structure.

### Why Deferred

- The planning-input doc explicitly classifies this as non-blocking design/debuggability/perf work.
- Pulling it into the default `R5.5` path would widen scope before the P0-P2 semantic and corpus
  gaps are closed.
