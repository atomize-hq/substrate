# Tasks: Agent Drift Analyzer Session Progress R5.75-0

Status: draft task ledger created on 2026-06-12 for the first sequential `R5.75` landing.

This packet is docs-only and exists to reconcile authority before the later analyzer-semantic
packets start landing. Keep each task reviewable and avoid widening into code changes.

## R5.75-0: R5.5 Status Reconciliation And Authority Cleanup

- [ ] Task R5.75-0.1: Add landed-status reconciliation language to the `R5.5` plan.
  - Acceptance: `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md` explicitly
    distinguishes landed `R5.5` work from remaining pre-`R6` work and routes the remaining work
    into `R5.75`.
  - Verify:
    - Manual review against `docs/specs/r5/R5_75/MAP.md`
    - `rg -n "R5\\.5|R5\\.75|R6" docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`

- [ ] Task R5.75-0.2: Reconcile the `R5.5` task ledger so landed work is not still presented as
      open implementation debt.
  - Acceptance: `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md` no longer
    reads like already-landed `R5.5` work is still waiting to be implemented, and remaining work
    points at `R5.75` where appropriate.
  - Verify:
    - Manual review against `docs/specs/r5/R5_75/MAP.md`
    - `rg -n "R5\\.5|R5\\.75|R6" docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`

- [ ] Task R5.75-0.3: Update root landing-order authority to name `R5.75` as the active pre-`R6`
      family.
  - Acceptance: `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md` identifies `R5.75` as the
    next/active family before `R6`, and any touched companion root-order doc matches that wording.
  - Verify:
    - Manual review against `docs/specs/r5/R5_75/MAP.md`
    - `rg -n "R5\\.75|R6" HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md docs/specs/hybrid-drift-sentinel-implementation-order.md`
  - Files:
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md` (only if touched)

- [ ] Task R5.75-0.4: Run a cross-doc authority audit after the edits.
  - Acceptance: the touched docs agree on:
    - what landed under `R5.5`
    - what remains under `R5.75`
    - why `R6` is still gated
  - Verify:
    - `rg -n "R5\\.5|R5\\.75|R6" docs/specs/r5/R5_75/MAP.md docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md`
    - `HYBRID_DRIFT_REMAINING_GAPS_AND_LANDING_ORDER.md`
    - `docs/specs/hybrid-drift-sentinel-implementation-order.md` (only if touched)

- [ ] Task R5.75-0.5: Run baseline validation and optional control smoke before promotion.
  - Acceptance:
    - `cargo test -p agent-drift-analyzer -- --nocapture` is green
    - optional native control smoke on `019eb430-6f9a-7a03-9a63-cb451b654795` is either green or
      explicitly recorded as intentionally skipped
  - Verify:
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - optional manual smoke using the command block in
      `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-0-spec.md`
  - Files:
    - no additional implementation files; verification-only closeout step

## Promotion Gate To R5.75-1

Do not advance to `R5.75-1` until all unchecked tasks above are complete and the authority audit
proves no touched doc still presents already-landed `R5.5` work as open or `R6` as currently
ready to open.
