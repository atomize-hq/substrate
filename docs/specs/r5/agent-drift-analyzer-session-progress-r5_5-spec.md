# Spec: Agent Drift Analyzer Session Progress R5.5

Status: draft spec created on 2026-06-11 from
`docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md`.

## Assumptions I'm Making

1. The planning-input doc created on 2026-06-11 is the source inventory for this follow-up
   family; this spec becomes the implementation authority once accepted, while the inventory still
   records the grounded post-landing review inputs it came from.
2. `R5` itself is landed in code and docs, including the formatter cleanup commit
   `5073ef445 chore: run fmt --all`; `R5.5` is hardening and acceptance deepening, not unfinished
   `R5` surface-area work.
3. `R5.5` should stay analyzer-owned by default. Doc, fixture, and analyzer-test updates are in
   scope; scorer retuning, scheduler changes, and broad sentinel policy changes are out of scope
   unless new repo evidence forces escalation.
4. The highest-risk issue is the troubleshooting overclaim described in `R5.5-FU-001`; packet
   ordering should treat that as the first landing gate before corpus deepening or cleanup.
5. Objective extraction should be fixed narrowly by improving row filtering and row-priority
   selection, not by rewriting the full checkpoint phase model.
6. Real-rollout acceptance growth should remain bounded and curated. `R5.5` should add a small
   committed corpus of annotated real cases rather than importing a broad uncontrolled fixture set.
7. The implicit progress-window representation (`R5.5-FU-007`) is a valid design follow-up, but it
   is not required for the first `R5.5` landing unless the P0-P2 fixes prove impossible to make
   clearly within the current seam.

If any of these assumptions drift, update this spec before implementation.

## Objective

Harden the landed `R5` session-progress analyzer so its checkpoint-level progress claims are safer
for future scorer consumption and more trustworthy on real packet work.

Primary users:

1. maintainers reading analyzer checkpoints and summary output,
2. operators using replay/manual smoke to understand real packet progress,
3. future `R6` scorer work that must not consume false `advancing` signals,
4. reviewers validating whether implementation, troubleshooting, and closeout progress are being
   classified honestly.

`R5.5` succeeds when the analyzer is materially more conservative and better grounded in real
session evidence, specifically by:

1. removing the known repeated-failure troubleshooting overclaim,
2. preferring the real `/goal` or user objective over system/developer boilerplate,
3. recognizing common JS/TS verifier attempts as first-class progress evidence,
4. expanding the committed real-rollout acceptance wall for implementation/review flows,
5. normalizing delegated/parent-visible progress consistently,
6. cleaning up the most misleading residual hygiene debt from the landed `R5` docs/code.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Supporting docs: `docs/specs/r5/*.md`
- Existing checkpoint schema under test: `v0.6`
- Existing progress seams already landed in `R5`:
  - `checkpoint/mod.rs`
  - `checkpoint/attempt.rs`
  - `checkpoint/diagnostics.rs`
  - `checkpoint/progress.rs`
- Acceptance harnesses already present:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`

No new external dependency is required by default. Ask first before adding one.

## Commands

Formatting and lint gates:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Analyzer-focused validation:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Optional sentinel parity spot-check when fixture shape or replay presentation is touched:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
```

Optional bounded manual wall when a real compactor bundle is available:

```bash
cargo run -p agent-drift-analyzer -- \
  --input-dir target/hybrid-drift-evals/<case>/compactor \
  --output-dir target/hybrid-drift-evals/<case>/analyzer-r5_5

cargo run -p agent-drift-sentinel -- \
  --mode replay \
  --checkpoint-dir target/hybrid-drift-evals/<case>/analyzer-r5_5
```

## Project Structure

```text
crates/agent-drift-analyzer/src/checkpoint/progress.rs
  Primary R5.5 semantics work: troubleshooting overclaim fix, parent-visible normalization,
  delegation limiting evidence, and any narrow progress-window comments left intentionally deferred.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Objective-row detection, phase/objective selection, and checkpoint assembly.

crates/agent-drift-analyzer/src/checkpoint/attempt.rs
  Attempt-role classification and JS/TS verifier command handling.

crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs
  Diagnostic seam hygiene; remove or justify dead code staging.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Narrow deterministic regressions for objective extraction, classifier coverage, delegated
  normalization, and troubleshooting semantics.

crates/agent-drift-analyzer/tests/progress_acceptance.rs
  Dedicated real/synthetic progress corpus assertions.

crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**
  Annotated committed rollout fixtures for implementation, closeout, and reopen/re-verify flows.

docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
  Source inventory for this family.

docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md
  Fixture authority that must be updated when the committed corpus grows.

docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md
  Validation protocol that should reflect any changed acceptance wall expectations.

docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md
  Historical R5 task ledger that needs cleanup once the legacy unchecked text is no longer useful.
```

## Code Style

Prefer narrow, conservative rule changes over broad rewrites. New logic should make the stronger
claim only when the evidence is explicit.

```rust
fn troubleshooting_progress_after_repeat(
    repeated_signature: bool,
    edited_failing_scope: bool,
    has_real_verifier_improvement: bool,
) -> ProgressStatus {
    if repeated_signature && edited_failing_scope && !has_real_verifier_improvement {
        return ProgressStatus::Mixed;
    }

    ProgressStatus::Advancing
}
```

Conventions:

- Keep helper names explicit and domain-specific.
- Prefer fail-closed progress classification when evidence conflicts.
- Add focused tests beside every new decision branch.
- Reuse existing `ProgressSignalCode` / `EvidenceRef` vocabulary instead of inventing parallel
  reporting types.
- Avoid widening public schema or replay contracts unless the follow-up truly needs it.

## Troubleshooting Advancement Rule

For troubleshooting, `advancing` requires at least one **direct advancement signal**:

- `VerificationClean`
- `FailureFrontierAdvanced`
- `FailureCountReduced`
- `TargetExercised` after a prior `BlockedBeforeTarget`
- a comparable later-stage failure on the same target

The following are **supporting/contextual signals only** and must not produce `advancing` by
themselves:

- `FailingScopeEdited`
- `WorkingSetConcentrated`
- `VerificationScopeNarrowed`
- commentary alignment
- repeated command cadence

In particular, repeated exact or strong-fuzzy failure signatures with overlapping edits and no
direct advancement signal must resolve conservatively to `mixed` or `stalled`, not `advancing`.

## Objective Candidate Ranking

Objective extraction should prefer ranked candidates over ad hoc string filtering alone.

```text
ObjectiveCandidate {
  row,
  text,
  source,
  boilerplate_class,
  priority,
}

ObjectiveSource =
  LiteralGoalCommand
  | ExplicitUserRequest
  | ThreadGoalText
  | AssistantRestatedGoal
  | DeveloperOrSystemInstruction
  | Unknown

BoilerplateClass =
  PermissionBlock
  | SkillsBlock
  | AgentInstructionBlock
  | MemoryOrProfileBlock
  | ToolingCapabilityBlock
  | SafetyOrPolicyBlock
  | GenericScaffold
```

Priority rule:

```text
literal /goal row
  > explicit user request
  > thread-goal / goal-creation text
  > assistant restated goal that references the user objective
  > non-boilerplate unknown
  > developer/system boilerplate (normally excluded)
```

Important caveat:

- do **not** filter quoted boilerplate when the user’s actual request is to edit or analyze that
  boilerplate itself, such as `AGENTS.md` instructions or a skill block

## JS/TS Verifier Command Matrix

Command-role and attempt-role expectations should stay aligned across the common JS/TS verifier
surface:

```text
npm test              -> verifier/test
npm run test          -> verifier/test
npm run lint          -> verifier/lint
npm run typecheck     -> verifier/typecheck-or-build
pnpm test             -> verifier/test
pnpm run test         -> verifier/test
pnpm run lint         -> verifier/lint
pnpm exec vitest      -> verifier/test
yarn test             -> verifier/test
yarn run test         -> verifier/test
yarn lint             -> verifier/lint
yarn run lint         -> verifier/lint
npx vitest            -> verifier/test
vitest                -> verifier/test
bun test              -> verifier/test
```

`R5.5` does not need to become “all JS package managers,” but this matrix should remain the
minimum aligned surface for the packet family.

## Progress Normalization Order

`SessionProgress` normalization should happen in one shared order:

1. Build the archetype-native or parent-visible progress candidate.
2. Apply delegation caps and add limiting signals plus limiting `counter_evidence`.
3. Run shared finalization exactly once.
4. Export the finalized `SessionProgress`.

If implementation pressure ever requires a different order, the resulting path must still preserve
stable sorting, dedupe, and evidence limits for both supporting and counter-evidence.

## Real-Rollout Fixture Annotation Rubric

Each new committed real-rollout fixture should carry a small manifest or equivalent structured
annotation with at least:

```text
case_id
source_rollout_id
screening:
  delegated
  child_visibility
expected_checkpoints:
  - ordinal
    expected_status
    expected_dimension
    confidence_floor
    confidence_ceiling
    required_signal_codes
    forbidden_signal_codes
    required_supporting_evidence_count
    required_counter_evidence_count
    decisive_evidence
    counter_evidence
    why_not_other_dimensions
notes
```

For reopen / re-verify cases, the expected dimension may honestly be
`verification_closeout_narrowing`, `implementation_verification_wall`, or
`troubleshooting_frontier`, but the annotation must justify the transition and prove the analyzer
does not overclaim clean closeout after work has reopened.

## Testing Strategy

1. **Focused checkpoint regressions first**
   - Add deterministic tests for each semantic bug or classifier gap in
     `crates/agent-drift-analyzer/tests/checkpoints.rs`.
2. **Acceptance-wall growth second**
   - Extend `progress_acceptance.rs` and fixture directories only after the underlying semantic or
     classifier rule is stable.
3. **Analyzer crate safety net**
   - Run the full `agent-drift-analyzer` test suite before claiming a packet done.
4. **Sentinel spot-check only when touched**
   - Replay/operator tests are required only if fixture or presentation changes cross the analyzer
     boundary.
5. **Manual unseen-session smoke as final confidence step**
   - Use a bounded unseen real-session sample to confirm the fixes improve live classification
     without widening scope beyond the committed acceptance corpus.

Coverage expectations:

- `R5.5-FU-001` must land with a regression that proves repeated signature + overlapping edit is
  not `advancing` by itself.
- `R5.5-FU-001` must also land with no-overcorrection guardrails:
  - same failure -> overlapping edit -> fewer failing tests => `advancing`
  - compile failure -> overlapping edit -> focused test failure on the same target => `advancing`
- `R5.5-FU-002` must land with objective-selection coverage where boilerplate appears before the
  true `/goal`.
- `R5.5-FU-002` must also preserve user-requested boilerplate targets when the actual task is to
  analyze or edit that boilerplate.
- `R5.5-FU-003` must land with at least three additional committed real-rollout cases:
  implementation advancement, closeout/review narrowing, and reopen/re-verify.
- `R5.5-FU-004` must land with command-role, attempt-role, and checkpoint-level proof for JS/TS
  verifier evidence.
- `R5.5-FU-005` and `R5.5-FU-006` must land with delegated-parent tests that assert conservative
  normalization, visible limiting evidence, and preserved counter-evidence after finalization.

## Boundaries

- **Always:**
  - Keep `R5.5` scoped to analyzer semantics, analyzer tests, and the minimal supporting docs.
  - Add a regression or acceptance assertion for every semantic change.
  - Prefer conservative (`mixed`, `stalled`, `insufficient_evidence`) outcomes over optimistic ones
    when evidence is incomplete.
  - Re-check real code anchors before changing packet scope.

- **Ask first:**
  - Adding new dependencies.
  - Widening into scorer logic, scheduler policy, or broad sentinel behavior.
  - Introducing a named internal `ProgressWindow` seam (`R5.5-FU-007`) instead of the planned
    narrow hardening path.
  - Changing committed real-rollout fixture sourcing rules beyond the bounded corpus described here.

- **Never:**
  - Retune `R6` scorer behavior as part of `R5.5`.
  - Treat stale legacy checklist text as open `R5` implementation work.
  - Silently widen into compactor rewrites, parent/child semantic linkage, or unrelated sentinel
    policy changes.
  - Remove acceptance coverage to make a packet appear green.

## Success Criteria

1. The troubleshooting repeated-failure path no longer reports `advancing` when the same failure
   repeats after an overlapping edit unless separate verifier-improvement evidence is present.
2. First-checkpoint objective extraction prefers the real `/goal` or equivalent user goal over
   developer/system boilerplate in the bounded repro cases.
3. JS/TS verifier commands such as `npm run lint`, `pnpm run lint`, `yarn lint`, `npm test`,
   `pnpm test`, and `vitest ...` classify deterministically as verifier attempts.
4. The committed acceptance corpus includes additional real rollout fixtures for implementation
   advancement, closeout/review narrowing, and reopen/re-verify honesty.
5. Parent-visible delegated progress flows through final normalization and exposes limiting evidence
   honestly when stronger claims are capped.
6. `diagnostics.rs` no longer relies on an unexplained top-level `#![allow(dead_code)]`, and the
   stale unchecked R5 legacy checklist text is cleaned up or explicitly collapsed into history.
7. The resulting spec/plan/tasks remain small-packet oriented and reviewable.

## R6 Readiness Gate

Do not open `R6` scorer cutover until all of the following are true:

- `R5.5-1` repeated-failure overclaim regressions are green
- objective extraction repros prefer the true `/goal` over boilerplate
- JS/TS verifier commands produce checkpoint-level progress evidence, not just low-level enums
- the committed real-rollout corpus includes implementation, closeout/review, and reopen/re-verify
  cases
- delegated parent-visible progress is normalized and limiting evidence is surfaced
- `cargo test -p agent-drift-analyzer -- --nocapture` is green
- any sentinel spot-checks touched by fixture or presentation changes are green
- the root landing-order authority identifies `R6` as next only after `R5.5` is closed

## Open Questions

1. Should `R5.5` include the low-priority `ProgressWindow` named-seam refactor, or should that stay
   deferred until after the correctness and acceptance packets are green?
2. Are the best new real-rollout fixtures already present in the local eval artifacts, or will the
   next planning pass need to curate them from fresh bounded manual smoke?
3. Does any real-rollout corpus addition require sentinel-facing fixture or presentation updates, or
   can `R5.5` stay fully analyzer-owned in code?
