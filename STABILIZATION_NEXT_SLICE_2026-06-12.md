# Stabilization Next Slice - 2026-06-12

## Purpose

This document captures the current stabilization context after Packet 4 of
[PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md](llm-last-mile/PLAN-54-inventory-selected-second-runtime-family-host-orchestrator-tool-surface-parity.md),
including:

- what already landed
- what was investigated
- what we now believe is real product risk vs harness noise
- what should be fixed before resuming broader roadmap work
- what is safe to defer

This is intended to be the single repo-root handoff doc for the current
"finish Packet 4, then take a bounded stabilization slice" decision.

## Required Skills And Working Mode

The next person taking this slice should explicitly use the same skills that
were required for this investigation/fix loop:

- `github:gh-fix-ci`
  Path:
  `/home/azureuser/.codex/plugins/cache/openai-curated/github/c6ea566d/skills/gh-fix-ci/SKILL.md`
- `gitnexus-debugging`
  Path:
  `/home/azureuser/.agents/skills/gitnexus-debugging/SKILL.md`
- `incremental-implementation`
  Path:
  `/home/azureuser/.agents/skills/incremental-implementation/SKILL.md`
- `code-review-and-quality`
  Path:
  `/home/azureuser/.agents/skills/code-review-and-quality/SKILL.md`

How to apply them here:

- use `github:gh-fix-ci` for any branch/PR/check/log inspection if GitHub
  Actions signal becomes available for the stabilization branch
- use `gitnexus-debugging` before and during root-cause work on runtime races
  and world-state publication behavior
- use `incremental-implementation` to keep this as a bounded slice with one
  small fix at a time, with verification after each increment
- use `code-review-and-quality` before treating any fix as done, including both
  the code change and its regression coverage

Additional repo-specific guardrails still apply:

- run GitNexus impact analysis before editing any function/class/method
- warn if impact comes back `HIGH` or `CRITICAL` before proceeding
- run GitNexus detect-changes before committing
- keep the slice narrow; do not mix runtime stabilization with unrelated new
  roadmap work

## Required Subagent Pattern

Do not do the next stabilization slice as a single-threaded pass.

Dispatch subagents deliberately in three roles:

1. Explorer subagents
   Use 2-4 explorers in parallel to narrow:
   - the retained startup race in
     [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
   - any still-relevant PTY harness amplifier in
     [crates/shell/tests/repl_world_first_routing_v1.rs](crates/shell/tests/repl_world_first_routing_v1.rs)
   - whether the world publication issue is actively blocking the current slice

2. Fix worker subagent
   After the main agent chooses the smallest justified increment, dispatch one
   worker to implement only that bounded fix. Give it explicit file ownership.
   For the expected next slice, that likely means one worker focused on
   [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
   and any directly-related regression test file.

3. Review subagent
   After the fix lands and verification passes, dispatch a fresh review agent
   using the `code-review-and-quality` lens. Findings should be reported first,
   ordered by severity, before merge/continuation.

Recommended orchestration shape:

- explore in parallel first
- synthesize locally
- run GitNexus impact on the specific symbol to edit
- implement one increment
- verify
- independently review

If the agent thread limit is reached, close stale explorers/reviewers before
spawning the next fix/review worker. Do not silently skip the explorer/fix/review
split just because old agents were left open.

## Current State

Packet 4 is unblocked.

Recent relevant commits:

- `1357259b` - `test: lock agent help selector contract`
- `8850a2ce` - `test: stabilize repl world routing checkpoint`

Packet 4 closeout context:

- `1357259b` kept the contract freeze in
  [crates/shell/tests/agent_public_control_surface_v1.rs](crates/shell/tests/agent_public_control_surface_v1.rs)
- `8850a2ce` widened two waits from 3s to 5s in
  [crates/shell/tests/repl_world_first_routing_v1.rs](crates/shell/tests/repl_world_first_routing_v1.rs)
- those changes were test-only
- no production symbols were changed in those two commits

At the time of investigation, the branch did not have authoritative GitHub
Actions branch/PR signal:

- `gh run list --branch feat/internal-host-orchestrator-world-dispatch-bootstrap`
  returned no runs
- the current workflow setup means `CI Testing` is not automatically giving a
  signal for this ref unless routed through the expected PR/manual path

So the best available signal came from local verification plus targeted
investigation.

## What We Already Fixed

One bounded harness fix has already been landed locally in
[crates/shell/tests/common.rs](crates/shell/tests/common.rs):

- `ensure_substrate_built()` now uses a cross-process Unix `flock` to serialize
  nested `cargo build -p substrate` calls across shell test processes
- the follow-up Clippy failure was fixed by making the lock file open intent
  explicit with `.truncate(false)`

Why this was worth doing:

- the original helper only used a process-local `OnceLock`
- multiple test binaries or parallel `cargo test` invocations could still all
  attempt nested `cargo build -p substrate`
- we reproduced Cargo package/build lock contention during investigation

Validation performed during this slice:

- `cargo clippy -p shell --tests -- -D warnings`
- targeted shell test coverage around the shared helper
- direct parallel test-binary runs showing one real nested build and a fast
  follower after the lock released

Conclusion:

- nested test-startup build contention was a real flake amplifier
- it has now been reduced enough that it should not block the next slice

## Investigated Findings

### 1. Retained startup race in `async_repl.rs`

Most likely real product issue.

Primary location:

- [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)

Main finding:

- retained startup readiness is driven by a one-shot startup verdict
- `Running` can win optimistically before startup is actually stable
- later failure can be dropped or normalized into a non-startup path

Why this looks real:

- three separate paths can race to signal startup state
- readiness can be inferred once a session handle is surfaced and retention
  flags are set
- completion can observe a terminal outcome after that optimistic ready edge
- once `Running` wins, the caller path does not do a sufficient revalidation
  before continuing

Practical effect:

- later slices may see misleading "startup succeeded" behavior when the durable
  ownership boundary was not actually stable
- failures can look intermittent and be misclassified as harness noise

Risk assessment from investigation:

- this is the highest-value remaining stabilization fix
- GitNexus impact on the startup path was high, but the likely fix is still
  local enough to take as a bounded slice

### 2. Non-atomic world/session/member publication

Likely real product issue, but broader than the retained startup race.

Primary locations:

- [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)

Main finding:

- observers can see torn state across parent session, world binding, and member
  participant records
- the largest known window is during world restart, after the new world binding
  is persisted and before stale world members are invalidated

What this means:

- a reader can momentarily observe:
  - parent session bound to generation `N+1`
  - stale world member records still authoritative-live at generation `N`

Why this matters:

- this is shared substrate behavior, not just a single test quirk
- it can distort control resolution, live-member selection, and world-restart
  truth if left unresolved

Important constraint:

- this does not look like a good "small local patch" candidate
- the durable fix likely wants a store-level batch/transactional publication
  primitive rather than another local REPL ordering tweak

### 3. PTY harness optimism and substring-based waits

Real harness issue, but secondary to the startup race.

Primary location:

- [crates/shell/tests/repl_world_first_routing_v1.rs](crates/shell/tests/repl_world_first_routing_v1.rs)

Main findings:

- some PTY waits poll for substrings without checking whether the child exited
- some tests still rely on short fixed waits or settle sleeps
- this makes the suite more load-sensitive than necessary

Important nuance:

- GitNexus blast radius for the broad `wait_for_output` helper in
  `repl_world_first_routing_v1.rs` was critical because it fans into most of
  that suite
- because of that, we deliberately did not do a broad helper rewrite during the
  first stabilization slice

Conclusion:

- this is worth tightening, but only in a narrow, evidence-driven way
- it should not expand into a generalized test-harness rewrite

### 4. Branch CI signal gap

Operational issue, not a product/runtime defect by itself.

Main finding:

- there was no authoritative branch-level CI signal for the current ref during
  this investigation
- that increased dependence on local targeted verification

Conclusion:

- this should be kept in mind while landing the next stabilization slice
- but it is not, by itself, the reason to stop product work

## Recommended Decision

Do not jump back into broader new work yet.

Take one more bounded stabilization slice first, then reassess.

Short version:

1. Fix the retained startup race next.
2. Do only minimal additional PTY hardening if flakes are still showing up.
3. Defer the store-level atomic publication work unless it becomes the active
   blocker.
4. After that bounded slice, return to the remaining roadmap work.

## What Must Be Fixed Before Continuing New Work

### Must-fix now: retained startup race

Target area:

- [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)

Goal:

- prevent an optimistic retained startup `Running` verdict from escaping before
  startup is truly stable enough to trust

Desired shape of the fix:

- keep the solution local to the retained startup path if possible
- avoid a large redesign of the retained runtime model
- either:
  - allow a later startup `Failed` verdict to override an earlier optimistic
    `Running`, or
  - keep the one-shot model but add a mandatory post-`Running` revalidation
    before returning a live retained runtime

Acceptance criteria:

- a surfaced early session handle is not enough, by itself, to let startup
  escape as healthy if completion immediately proves otherwise
- existing intentionally-supported success cases still work
- no stale live retained runtime is left behind on failed ownership

Recommended regression coverage:

- a low-level async REPL test near the retained-startup tests in
  [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
- a public-start style integration assertion in
  [crates/shell/tests/agent_public_control_surface_v1.rs](crates/shell/tests/agent_public_control_surface_v1.rs)

## Nice To Fix In The Same Bounded Slice

### Minimal PTY hardening only if still needed

This should only be taken if the suite is still showing actual noise after the
build-lock fix.

Good candidates:

- the hottest exit-aware wait/readiness cases
- the narrowest spots where a child exit should fail fast instead of timing out

Not recommended in this slice:

- broad rewrites of the shared `wait_for_output` helpers in the large routing
  suites
- generalized substring-matching cleanup across all PTY tests
- a full harness abstraction pass

Rationale:

- those changes have a much larger blast radius
- they are more likely to create new test churn than to give immediate product
  confidence

## Safe To Defer For Now

### Store-level atomic publication / transactional snapshot fix

Target area:

- [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)

Why defer:

- it still looks real
- but it is broader and more cross-cutting than the retained startup race
- the fix likely wants a new batch/transactional publication primitive

When to pull it forward:

- if world restart/linkage behavior actively blocks the next slice
- if later targeted verification still shows misleading world-generation truth
- if status/control surfaces remain unreliable after the retained startup fix

### General harness cleanup

Safe to defer:

- large PTY helper refactors
- broad settle-sleep cleanup
- generalized test-framework polishing

Unless:

- a concrete flake keeps reproducing after the build-lock fix and after any
  narrow readiness tweak

## Suggested Next Execution Order

1. Land the retained startup race fix in
   [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs).
2. Add only the smallest regression coverage needed to prove that fix.
3. Re-run the most relevant shell tests:
   - retained startup/owner-helper coverage
   - public control surface coverage
   - the targeted world-routing regression that previously needed the 3s to 5s
     wait bump
4. If flakes still show up, take one narrow PTY wait/readiness adjustment.
5. If that slice is green, go back to the broader remaining roadmap work in:
   - [llm-last-mile/REMAINING-overall-scope-2026-06-10.md](llm-last-mile/REMAINING-overall-scope-2026-06-10.md)
   - [AGENT_ORCHESTRATION_GAP_MATRIX.md](AGENT_ORCHESTRATION_GAP_MATRIX.md)

## Bottom Line

Do not treat the Packet 4 test-only fixes as proof that the deeper substrate
stability work is done.

The current best call is:

- nested test-startup build contention was real and has now been addressed
- the retained startup race still looks like the next fix to land
- non-atomic world publication still looks real, but is better handled as a
  later, broader store-level slice
- after one more bounded stabilization slice, it should be reasonable to resume
  broader new work
