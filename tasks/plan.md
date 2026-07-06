# R6-3.6 execution plan

Authoritative packet docs:
- /Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-spec.md
- /Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-plan.md
- /Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r6/R6-3.6/agent-drift-analyzer-semantic-goal-drift-positive-controls-and-corpus-revalidation-tasks.md

Execution mode: slash-build-auto + incremental-implementation + test-driven-development.

## Dependency order

1. Baseline capture and review gate confirmation
   - Record git status/log
   - Re-run focused semantic-goal-drift tests
   - Confirm Claude `--model opus` lane

2. Acceptance harness expansion
   - Convert the hardcoded 3-case invariant into a curated bounded corpus invariant
   - Add assertions for full eligibility (`unknowns.is_empty()`) and stable-anchor proof

3. Positive-control fixtures
   - Add at least five true-positive pivot fixtures covering repo / crate / file / work-item / verification pivots
   - Land with focused acceptance + scorer verification

4. Negative-control fixtures and boundary guardrails
   - Add at least five legitimate non-pivot fixtures
   - Pin non-containment boundaries, with at least one acceptance-path case

5. Funnel reporting
   - Inspect existing checkpoint exports
   - Add the smallest honest reporting deltas possible without widening analyzer export surface

6. Verification wall and corpus rerun
   - Focused tests
   - Full agent-drift-analyzer tests
   - Workspace clippy + fmt
   - Batch harness rerun or explicit blocker

7. Docs closeout
   - Update FINDINGS / MAP / R6-3 task ledger with counts, funnel results, delegation split, residue, and next-step decision

8. Final Claude code review
   - Run Claude on the landed packet diff with `--model opus`
   - Address any findings before final closeout claim

## Commit discipline

- One commit per implementation task or tightly-coupled slice
- Never stage unrelated files
- Each slice must include its own task-status update when relevant
