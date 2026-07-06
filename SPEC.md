# Spec: R6 Semantic Goal Drift Positive Controls And Corpus Revalidation

## Assumptions I'm Making
1. This packet is the next bounded line of work after `R6-3` and `R6-3.5`, not a restart of the earlier semantic-goal-drift design.
2. The main deliverable is validation truth: prove recall on known true pivots while preserving the recent precision fixes, before any weighted or graduated semantic distance work begins.
3. The work should stay analyzer-local unless the smallest necessary test/support/reporting touch needs to reach nearby analyzer test helpers.
4. The repo's existing `semantic_goal_drift` acceptance path in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs` is the authority for end-to-end fixture shape.
5. Claude review is a required gate for later planning docs and landed code, but the local Claude CLI is currently not authenticated, so that gate cannot run until login is restored.

## Objective
Land a bounded validation packet that answers one question before `R6-3.X.2B` weighted or graduated semantic distance begins:

Can `semantic_goal_drift` still fire on known true abandoned-goal pivots while staying quiet on legitimate narrowing and progression?

The user is the maintainer of the `agent-drift-analyzer` line and the downstream consumers of analyzer evidence. Success means the repo has a trustworthy validation wall for recall and precision, not just green unit tests. The packet must add explicit positive controls, explicit negative controls, containment false-negative guardrails, and corpus-report funnel visibility so the next decision is evidence-backed.

## Tech Stack
- Rust workspace on Rust 2021 / MSRV policy from `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/Cargo.toml`
- Analyzer code in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/crates/agent-drift-analyzer`
- Acceptance fixtures as committed JSON under `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance`
- Batch tooling in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/scripts/dev/drift-batch-scan`
- Planning and packet authority docs under `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r6`

## Commands
Baseline capture:
```bash
git status --short
git log --oneline -n 20
```

Focused validation:
```bash
cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

Corpus tooling inspection and rerun lane:
```bash
find scripts/dev/drift-batch-scan -maxdepth 2 -type f | sort
python3 scripts/dev/drift-batch-scan/sample_sessions.py --help
python3 scripts/dev/drift-batch-scan/run_batch.py --help
python3 scripts/dev/drift-batch-scan/tabulate.py --help
python3 scripts/dev/drift-batch-scan/inspect_targets.py --help
python3 scripts/dev/drift-batch-scan/filter_junk.py --help
```

Review gates:
```bash
# planning docs review gate, after plan/tasks docs are drafted
claude -p --model opus --output-format json --disable-slash-commands --tools ""

# landed diff review gate, after implementation is complete
claude -p --model opus --output-format json --disable-slash-commands --tools ""
```

Note: as of 2026-07-05, local Claude CLI exists at `/Users/spensermcconnell/.local/bin/claude` but authentication is missing, so the review commands are specified but blocked until `claude` login or `ANTHROPIC_API_KEY` is restored.

## Project Structure
```text
/Users/spensermcconnell/.codex/worktrees/97a0/substrate/
├── SPEC.md                                                # this packet spec
├── docs/specs/r6/MAP.md                                   # packet-family authority and closeout routing
├── docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md
├── docs/specs/r6/R6-3/agent-drift-analyzer-rolling-semantic-goal-drift-tasks.md
├── docs/specs/r6/R6-3.5/agent-drift-analyzer-objective-target-hygiene-tasks.md
├── crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs
├── crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs
├── crates/agent-drift-analyzer/tests/fixtures/semantic_goal_drift_acceptance/
├── crates/agent-drift-analyzer/tests/checkpoints.rs                    # secondary scope only if needed
├── crates/agent-drift-analyzer/tests/support/mod.rs                    # secondary scope only if needed
└── scripts/dev/drift-batch-scan/                                       # corpus rerun + funnel reporting
```

## Code Style
Keep the packet narrow, additive, and analyzer-local. Prefer new acceptance fixtures and explicit reporting fields over broad scorer rewrites. Follow existing Rust patterns: helper extraction when a rule is reusable, `Result<T, anyhow::Error>` for fallible tooling paths, and evidence strings that explain why a claim fired or was suppressed.

Example style:
```rust
fn positive_control_case_ids() -> &'static [&'static str] {
    &[
        "synthetic-repo-target-pivot",
        "synthetic-crate-target-pivot",
    ]
}
```

Conventions:
- Preserve existing fixture contract: one committed case directory per scenario with `raw.json` and `expected.json`.
- Prefer scorer-path assertions over helper-only assertions when the packet is proving operational behavior.
- Add the smallest possible reporting expansion to `scripts/dev/drift-batch-scan/**` when a funnel metric is missing.
- Keep evidence and suppression labels auditable. A quiet negative control should be quiet for a named reason, not by disappearing into unobservable behavior.

## Testing Strategy
### Test levels
1. **Unit / scorer tests** in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs`
   - containment false-negative guardrails
   - suppression behavior
   - scorer-local positive or negative edge cases when the acceptance path would be too heavy
2. **Acceptance fixtures** in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs`
   - at least five true-positive pivot cases
   - at least five legitimate non-pivot cases
   - at least one Codex-discovered containment guardrail exercised through the live analyzer path
3. **Batch tooling validation** in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/scripts/dev/drift-batch-scan`
   - verify the funnel reports eligibility, suppressions, disjoint residue, fires, and delegation topology splits where available
4. **Full analyzer regression wall**
   - the focused semantic-goal-drift commands
   - the full `agent-drift-analyzer` test suite
   - workspace clippy and fmt gates

### Coverage expectations
- Positive controls must prove recall without loosening the current eligibility bar.
- Negative controls must pin the recent precision fixes so the packet cannot regress into obvious narrowing or plan-progress false positives.
- Corpus rerun output must explain zero fires or non-zero fires through explicit funnel counts, not just a summary headline.

## Boundaries
### Always do
- Start from live repo truth in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, not stale task assumptions.
- Record the pre-change baseline: git state, recent history, focused semantic-goal-drift tests, full analyzer wall.
- Keep the work bounded to analyzer validation, acceptance fixtures, corpus tooling, and R6 findings or routing docs.
- Preserve the same live analyzer checkpoint path for new fixtures that the existing acceptance corpus uses.
- Update findings, MAP, and the relevant R6 task ledger with an honest proceed or stop decision for `R6-3.X.2B`.
- Run Claude review on final planning docs and on the landed code diff once Claude authentication is restored.

### Ask first about
- Any change outside the declared primary scope files.
- Any need to touch sentinel behavior, compactor normalization, or production input contracts.
- Any proposal to change the semantic-goal-drift eligibility bar instead of validating under the current bar.
- Any packet-scope expansion beyond this validation lane, including starting weighted or graduated distance work.
- Any need to commit or preserve non-private corpus artifacts beyond scripts, fixtures, and docs.

### Never do
- Do not implement weighted or graduated semantic distance in this packet.
- Do not loosen semantic-goal-drift eligibility in this packet.
- Do not change sentinel behavior.
- Do not change compactor normalization.
- Do not implement full delegated or child-session semantics.
- Do not migrate `TaskFrame`, `working_set`, `checkpoint/progress`, or downstream consumers.
- Do not broaden objective extraction unless a positive-control or containment-guardrail fixture exposes a direct scorer-input bug that this packet must pin honestly.
- Do not claim success from green tests alone if the funnel and corpus evidence still leave recall or precision ambiguous.

## Success Criteria
This packet is complete only when all of the following are true:

1. At least five known true `semantic_goal_drift` positive controls fire through the analyzer or scorer path the repo treats as authoritative.
2. At least five legitimate narrowing or progression negative controls stay quiet.
3. The Codex-discovered containment false-negative guardrails are pinned, with at least one exercised through the acceptance path.
4. The batch or reporting tooling explains the eligibility and suppression funnel for why candidates did or did not fire.
5. The 110-session corpus rerun completes with comparable settings, or the exact blocker is captured with an actionable rerun command.
6. The corpus summary includes delegation stratification when the current tooling can surface it.
7. No production input contracts are weakened.
8. No eligibility-bar loosening is introduced.
9. No weighted or graduated distance implementation is introduced.
10. Focused semantic-goal-drift tests, full analyzer tests, workspace clippy, and workspace fmt all pass.
11. The updated docs state clearly whether to proceed to `R6-3.X.2B` next, or to stop and repair recall or precision first.
12. Final planning docs receive a Claude review pass, and landed code receives a Claude diff review pass, both intended to use the local Claude CLI `--model opus` lane once authentication is available.

## Open Questions
1. Should the follow-on planning artifact stay as a generic root `PLAN.md` or live under a packet-specific R6 path such as `/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/r6/R6-3.X.2A-or-next/` to match the repo's existing packet-family layout?
2. Is restoring local Claude authentication part of this workstream now, or should the packet proceed with planning and implementation while marking the Claude review gate blocked until login is restored?
3. If the 110-session corpus sample command used by `R6-3.5` differs from the current scripts' canonical README examples, which one should be treated as the official rerun command in the final plan?
