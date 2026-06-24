# Tasks: Agent Drift Analyzer Adapted External Robustness Fixture Family (R5.75-5)

Status: draft task ledger created on 2026-06-23 after `R5.75-4` was promoted and the live
`docs/specs/r5/R5_75/MAP.md` named `R5.75-5` as the active seam.

Keep each task narrow, reviewable, and scoped to the adapted external robustness fixture family. Do not
widen into new analyzer semantic retuning, `R5.75-6`, or `R6`.

Packet prerequisite rule: this packet names `R5.75-4` as landed. Verify that in live docs/tests before
editing. If the prerequisite is missing, stop and report it instead of compensating inside `R5.75-5`.

## R5.75-5.0: Docs Lock

- [x] Task R5.75-5.0.1: Commit the SPEC/PLAN/TASKS family for `R5.75-5`.
  - Acceptance: `docs/specs/r5/R5_75/R5_75-5/` contains the spec, plan, and this tasks ledger, and
    they record:
    - the explicit native-primary / adapted-secondary authority boundary,
    - the adopted adapted sample ids and their shape-routing rule,
    - the rule that objective `stretch-external/` expands only if it adds net-new signal,
    - the restoration of the missing fixture-manifest doc as packet-owned work, and
    - the fact that packet prompts are out of scope for this docs pass.
  - Verify: manual review against the `R5.75-5` packet in `docs/specs/r5/R5_75/MAP.md`, the current
    `progress_acceptance` / `objective_acceptance` harnesses, and the live adapted corpus under
    `target/ranga-validation/`.
  - Files:
    - `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-spec.md`
    - `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-plan.md`
    - `docs/specs/r5/R5_75/R5_75-5/agent-drift-analyzer-adapted-external-robustness-fixture-family-tasks.md`
  - Closeout note (2026-06-23): the docs-only SPEC/PLAN/TASKS authority lock is committed for
    `R5.75-5`, preserving the primary-vs-secondary authority boundary, shape-routing rules, missing
    fixture-manifest restoration requirement, and packet-prompt exclusion for the pre-implementation
    pass.

## R5.75-5.1: Audit And Route The Adopted Adapted Sessions

- [ ] Task R5.75-5.1.1: Re-read the adopted adapted sample ids and record their correct homes before
      committing any fixture.
  - Acceptance: this ledger records the routing outcome for:
    - `05a56cc51632982b`
    - `f47b81f39f2495dd`
    - `097d97e914ca220f`
    - `da59436e63915185`
    and explicitly states:
    - which are progress-shaped,
    - whether `05a56cc51632982b` adds objective signal beyond the existing locked corpus, and
    - which packet-owned expectation each committed adapted case must encode.
  - Verify:
    - inspect `docs/specs/r5/R5_75/MAP.md`
    - inspect `target/ranga-validation/runs/<session-id>/compactor/` and any saved `target/r5_75-smoke/`
      outputs already cited by `R5.75-1` through `R5.75-4`
    - re-read the objective locked corpus and `stretch-external/` placeholder contract
  - Files:
    - this tasks ledger
  - Notes to record during closeout:
    - `f47b81f39f2495dd` should map to sparse-readable progress robustness (`R5.75-2`)
    - `097d97e914ca220f` should map to zero-verifier anti-flap progress robustness (`R5.75-4`)
    - `da59436e63915185` should map to delegated parent-visible progress robustness (`R5.75-3`)
    - `05a56cc51632982b` should only enter `stretch-external/` if it adds signal beyond the existing
      `R5.75-1` locked objective cases

## R5.75-5.2: Restore The Fixture Authority Contract

- [ ] Task R5.75-5.2.1: Restore/create the live fixture-manifest authority doc and update the README
      contract for the new secondary lane.
  - Acceptance:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md` exists in the live docs tree,
      not only under `docs/specs/archived/`
    - it documents the current native annotated real-rollout corpus, synthetic support cases, and the
      bounded adapted secondary lane honestly
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md` clearly distinguishes
      native primary authority from adapted secondary robustness
    - if `stretch-external/` remains placeholder-only, its README wording stays truthful
  - Verify:
    - manual review against `docs/specs/design-arch/DESIGN-r5-validation-and-rollout-protocol.md`
    - manual review against the live progress/objective harness contracts
  - Files:
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/README.md`

## R5.75-5.3: Commit The Adapted Progress Secondary Lane

- [ ] Task R5.75-5.3.1: Add the explicit adapted-external secondary lane to the progress acceptance
      harness.
  - Acceptance:
    - `progress_acceptance.rs` distinguishes:
      - native annotated real-rollout authority cases,
      - adapted external secondary cases, and
      - synthetic support cases
    - the harness stays deterministic and bounded
    - at least one native annotated real-rollout anchor remains required
    - adapted cases cannot be mistaken for native primary authority in code or test messages
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`

- [ ] Task R5.75-5.3.2: Commit the sparse-readable adapted progress case.
  - Acceptance: one adapted fixture directory and expectation file are committed for
    `f47b81f39f2495dd`, and they encode the packet-owned `R5.75-2` expectation:
    - analyzer does not abort,
    - the selected checkpoint stays conservative,
    - `ProgressStatus::InsufficientEvidence` / weak structured fields remain honest,
    - the case is explicitly secondary robustness, not primary authority.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/adapted-sparse-readable-f47b81f39f2495dd/**`
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

- [ ] Task R5.75-5.3.3: Commit the zero-verifier exploratory adapted progress case.
  - Acceptance: one adapted fixture directory and expectation file are committed for
    `097d97e914ca220f`, and they encode the packet-owned `R5.75-4` expectation:
    - the selected checkpoint stays on conservative planning posture,
    - troubleshooting-frontier overclaim does not return,
    - the case is explicitly secondary robustness, not primary authority.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/adapted-zero-verifier-097d97e914ca220f/**`
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

- [ ] Task R5.75-5.3.4: Commit the delegated parent-visible adapted progress case.
  - Acceptance: one adapted fixture directory and expectation file are committed for
    `da59436e63915185`, and they encode the packet-owned `R5.75-3` / `R5.75-4` expectation:
    - the exploratory overclaim stays removed,
    - the late delegated interval remains conservative `parent_visible_orchestration`,
    - child-visibility limits remain explicit limiting/counter-evidence,
    - the case is explicitly secondary robustness, not primary authority.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/adapted-parent-visible-da59436e63915185/**`
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`

## R5.75-5.4: Objective Stretch-External Decision

- [ ] Task R5.75-5.4.1: Decide whether `05a56cc51632982b` earns one bounded `stretch-external`
      objective case.
  - Acceptance: one of the following is true, and the ledger/doc wording makes the choice explicit:
    - **Add one case:** `05a56cc51632982b` contributes net-new objective robustness signal beyond
      `R5.75-1`'s locked corpus, so one bounded `stretch-external` fixture lands with exact rationale, or
    - **No-op honestly:** `05a56cc51632982b` adds no net-new signal, so `stretch-external/` remains
      placeholder-only and the packet records the explicit cross-reference/no-op rationale.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - manual comparison against the locked objective cases:
      - `orchestration-evaluate-ask-anchor`
      - `orchestration-marker-free-boilerplate-exclusion`
  - Files:
    - `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/**`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md`
    - `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/README.md`
    - this tasks ledger

## R5.75-5.5: Acceptance And Analyzer Validation Wall

- [ ] Task R5.75-5.5.1: Run the packet's focused acceptance and full analyzer gates.
  - Acceptance:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture` is green
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` is green
    - `cargo test -p agent-drift-analyzer -- --nocapture` is green
  - Verify:
    - `cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
    - `cargo test -p agent-drift-analyzer -- --nocapture`
  - Files:
    - no additional implementation files required; verification-only step

## R5.75-5.6: Full Native + Adapted Smoke Review

- [ ] Task R5.75-5.6.1: Re-run the full named smoke set and confirm all earlier packet expectations
      still hold together after the fixture-family landing.
  - Acceptance:
    - all six named native smoke sessions still hold the packet expectations from `R5.75-1` through
      `R5.75-4`
    - all four named adapted smoke sessions still hold the expectations the committed secondary lane now
      claims
    - adapted sessions are clearly recorded as secondary robustness proof, not native authority
      replacement
    - no adopted repro class is left guarded only by `target/` artifacts or memory
  - Verify:
    - rerun the full native + adapted smoke commands from the companion spec
    - inspect `summary.md` and relevant `checkpoints.jsonl` rows for every session, not just exit codes
  - Files:
    - no committed implementation files required; evidence recorded under
      `target/r5_75-smoke/R5.75-5/<session-id>/`
    - this tasks ledger

## R5.75-5.7: MAP Promotion And Routing Update

- [ ] Task R5.75-5.7.1: Update the main `R5.75` routing hub only after the packet has actually closed.
  - Acceptance:
    - `docs/specs/r5/R5_75/MAP.md` is updated only after Tasks `R5.75-5.1` through `R5.75-5.6` are
      honestly complete
    - `R5.75-5` is recorded as promoted history with the committed adapted fixture-family outcome
    - `R5.75-6` becomes the next active seam only after the promotion gate is truly met
  - Verify:
    - manual review against the completed tasks ledger
    - manual review against the saved smoke outputs and green acceptance walls
  - Files:
    - `docs/specs/r5/R5_75/MAP.md`
