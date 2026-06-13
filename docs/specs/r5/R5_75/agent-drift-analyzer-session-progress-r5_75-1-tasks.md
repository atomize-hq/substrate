# Tasks: Agent Drift Analyzer Session Progress R5.75-1

Status: draft task ledger created on 2026-06-12 after `R5.75-0` landed and promoted this packet
to the active pre-`R6` implementation queue.

Keep each task narrow, reviewable, and scoped to the objective-condensation seam. Do not widen
into fail-open behavior, delegated-parent stabilization, or adapted fixture-family expansion.

## R5.75-1: Objective Condensation And Target Extraction

- [ ] Task R5.75-1.1: Add regressions for giant pasted user prompts whose concrete ask appears
      inside the body.
  - Acceptance: `crates/agent-drift-analyzer/tests/checkpoints.rs` contains focused cases showing
    that long pasted prompt/scaffold bodies resolve to the concrete task ask rather than to the
    entire pasted body.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5.75-1.2: Add regressions proving preserved boilerplate-target requests still survive
      condensation.
  - Acceptance: regression coverage explicitly proves that real requests to analyze, compare,
    explain, or edit `AGENTS.md`, `<skill>`, `Available skills`, tooling scaffolds, or similar
    instruction surfaces remain the stored objective instead of being shortened into misleading
    fragments.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5.75-1.3: Refine objective-candidate ordering so longer same-priority bodies do not
      win by length.
  - Acceptance: `crates/agent-drift-analyzer/src/checkpoint/mod.rs` prefers `/goal`, short
    imperative asks, steer pivots, and concrete workspace/action-target phrases over longer
    same-priority prompt bodies, and it removes the current bias that can reward the longer text.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [ ] Task R5.75-1.4: Condense the chosen objective row down to the true concrete ask when
      possible.
  - Acceptance: when the winning row is still a large pasted user prompt, the stored checkpoint
    objective is reduced to the shortest concrete task ask that preserves the real target, rather
    than the full pasted body.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

- [x] Task R5.75-1.5: Run the packet’s automated validation gates.
  - Acceptance:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green
    - `cargo test -p agent-drift-analyzer -- --nocapture` is green
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - no additional implementation files; verification-only step
  - Closeout note (2026-06-12):
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - both commands passed on 2026-06-12
    - no implementation/code changes were required for this closeout rerun
    - docs-only packet-local closeout evidence was recorded in commit `47b3ab467`
      (`docs: record r5.75-1.5 closeout evidence`)

- [x] Task R5.75-1.6: Run the named native and adapted smoke sessions and inspect the first
      checkpoint objective manually.
  - Acceptance:
    - native sessions
      `019eb430-6f9a-7a03-9a63-cb451b654795`,
      `019eb47f-0118-7e90-8291-30a1fb93769e`,
      `019eb98e-3c16-7ba0-92f9-0085654b470c`
      all show the concrete task ask in the first checkpoint objective
    - adapted session `05a56cc51632982b` condenses to the workspace action request rather than
      the full pasted skill body
  - Verify:
    - run the native/adapted command blocks in
      `docs/specs/r5/R5_75/agent-drift-analyzer-session-progress-r5_75-1-spec.md`
    - inspect:
      - `sed -n '1,80p' "$ANALYZER_OUT/summary.md"`
      - `sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"`
  - Files:
    - no new implementation files; manual smoke evidence only
  - Closeout note (2026-06-12):
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
    - reran the packet smoke ladder for native sessions
      `019eb430-6f9a-7a03-9a63-cb451b654795`,
      `019eb47f-0118-7e90-8291-30a1fb93769e`,
      `019eb98e-3c16-7ba0-92f9-0085654b470c`,
      plus adapted session `05a56cc51632982b`
    - re-inspected `summary.md` and the first `checkpoints.jsonl` rows under
      `target/r5_75-smoke/r5_75-1/<session-id>/analyzer/`
    - first checkpoint objectives resolved to:
      - `019eb430-6f9a-7a03-9a63-cb451b654795` → /goal Review the already-landed Packet R5-7 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, tasks, fixtures, and validation protocol, and determine whether it is ready to keep.
      - `019eb47f-0118-7e90-8291-30a1fb93769e` → use the `$code-review-and-quality` skill to evaluate if what was implemented laned correctly and completely
      - `019eb98e-3c16-7ba0-92f9-0085654b470c` → review what landed in the codebase vs the planning docs and intended end state and validate/invalidate if it landed correctly and completely
      - `05a56cc51632982b` → add this skill to `@shared-cab-app`
    - packet-scoped analyzer fix retained scope in `checkpoint/mod.rs` plus
      `tests/checkpoints.rs`; no compactor/schema/sentinel widening was needed

## Hard Gate For R5.75-1

Do not mark this packet complete unless Task `R5.75-1.6` proves all of the following:

- native sessions
  `019eb430-6f9a-7a03-9a63-cb451b654795`,
  `019eb47f-0118-7e90-8291-30a1fb93769e`,
  `019eb98e-3c16-7ba0-92f9-0085654b470c`
  all show the concrete task ask in the first checkpoint objective,
- adapted session `05a56cc51632982b` condenses to the workspace action request rather than the
  full pasted skill body,
- preserved boilerplate-target requests still survive in regression coverage, and
- both automated analyzer test gates are green.

Automated green status alone does not satisfy this packet.

## Promotion Gate To R5.75-2

Do not promote to `R5.75-2` until every task above is complete and the smoke review proves the
first checkpoint objective resolves to the true concrete ask for all named repro sessions without
regressing deliberate boilerplate-target requests.
