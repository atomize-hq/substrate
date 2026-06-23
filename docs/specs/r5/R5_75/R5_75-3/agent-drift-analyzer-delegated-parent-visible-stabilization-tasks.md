# Tasks: Agent Drift Analyzer Delegated Parent-Visible Stabilization (R5.75-3)

Status: draft task ledger created on 2026-06-22 after `R5.75-2` was promoted and `R5.75-3` became the
active seam in `docs/specs/r5/R5_75/MAP.md`.

Keep each task narrow, reviewable, and scoped to delegated parent-visible stabilization. Do not widen
into generalized fingerprint redesign, adapted committed fixture-family work, or structured-state
comparability migration.

Packet prerequisite rule: this packet names `R5.75-2` as landed. Verify that in live code/tests before
editing. If the prerequisite is missing, stop and report it instead of compensating inside `R5.75-3`.

## R5.75-3.0: Docs Lock

- [ ] Task R5.75-3.0.1: Commit the SPEC/PLAN/TASKS family for `R5.75-3`.
  - Acceptance: `docs/specs/r5/R5_75/R5_75-3/` contains the spec, plan, and this tasks ledger, and
    they record:
    - the packet's analyzer-local scope,
    - the explicit per-session acceptance bar,
    - the first delegated real-rollout fixture choice (`019eb970-3543-7ab1-a5d6-2a62c00c7185` by
      default),
    - the smoke-only status of `da59436e63915185`, and
    - the narrow conditional comparability fallback rule.
  - Verify: manual review against the `R5.75-3` packet in `docs/specs/r5/R5_75/MAP.md`, the delegated
    corpus contract in
    `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`, and the live
    `progress.rs` seam.
  - Files:
    - `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-spec.md`
    - `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-plan.md`
    - `docs/specs/r5/R5_75/R5_75-3/agent-drift-analyzer-delegated-parent-visible-stabilization-tasks.md`

## R5.75-3.1: Characterize The Named Delegated Repros

- [x] Task R5.75-3.1.1: Run the named native/adapted delegated smoke sessions and record exact packet
      expectations before changing code.
  - Acceptance: this ledger records the current/expected lane, progress status, confidence band, and
    limiting/supporting evidence expectations for:
    - `019eb907-95c4-73e1-843e-e337d1e93cb9`
    - `019eb917-9531-74e0-897d-ad8d362138ec`
    - `019eb970-3543-7ab1-a5d6-2a62c00c7185`
    - `da59436e63915185`
    and explicitly distinguishes:
    - `019eb970-3543-7ab1-a5d6-2a62c00c7185` as the packet's positive-proof case,
    - `019eb907-95c4-73e1-843e-e337d1e93cb9` and
      `019eb917-9531-74e0-897d-ad8d362138ec` as conservative delegated-parent stability witnesses, and
    - `da59436e63915185` as the adapted smoke witness whose anti-flap behavior beyond this parent-visible
      stability bar remains owned by `R5.75-4`.
  - Observed packet-local expectations from the 2026-06-22 smoke rerun:
    - `019eb907-95c4-73e1-843e-e337d1e93cb9`
      - current smoke: all four checkpoints stayed in `planning_convergence`
        (`stalled` once, then `insufficient_evidence`; confidence `low`) even though the summary records
        `delegating_parent` / `opaque` delegation markers.
      - packet-local expectation: this remains a conservative delegated-parent witness, so closeout smoke
        should move the delegated block into `parent_visible_orchestration` with only `low` confidence and
        either `insufficient_evidence` or `stalled` status, not a stronger child-progress claim.
      - supporting evidence to preserve: visible `spawn_agent` / `wait_agent` / `close_agent` markers and
        the summary's "delegation topology kept orchestration evidence in the visible parent prefix"
        support.
      - limiting evidence to preserve: opaque child visibility and repeated "explicit delegation markers
        lacked child visibility/context evidence across command families: git, sed, rg", which must keep
        the lane conservative.
    - `019eb917-9531-74e0-897d-ad8d362138ec`
      - current smoke: checkpoint 1 stayed `single_agent`, then checkpoints 2-3 became
        `delegating_parent` / `opaque` but still held `planning_convergence` with `stalled` status
        (confidence `medium` then `low`).
      - packet-local expectation: once delegation starts, this remains a conservative delegated-parent
        witness and should hold a `parent_visible_orchestration` lane at `low` confidence with a
        `stalled` posture rather than collapsing back to generic planning-only noise.
      - supporting evidence to preserve: visible `spawn_agent` / `wait_agent` markers on the delegated
        portion of the session.
      - limiting evidence to preserve: opaque child visibility plus repeated "explicit delegation markers
        lacked child visibility/context evidence across command families: npx, sed, git" and the existing
        "repeated broad planning scan without convergence artifact" counter-evidence.
    - `019eb970-3543-7ab1-a5d6-2a62c00c7185`
      - current smoke: checkpoints 2-5 already hold `mixed / parent_visible_orchestration / medium`
        after an initial `insufficient_evidence / troubleshooting_frontier / low` checkpoint and before a
        final `insufficient_evidence / planning_convergence / low` cooldown checkpoint.
      - packet-local expectation: this is the packet's positive-proof case and must keep a stable
        `parent_visible_orchestration` cluster with `mixed` status and `medium` confidence; it is the
        default first real-rollout fixture candidate for committed `progress_acceptance`.
      - supporting evidence to preserve: `delegating_parent` / `partial` delegation, visible
        `spawn_agent` / `wait_agent` markers, and summary evidence that the parent session linked child
        rollout surfaces.
      - limiting evidence to preserve: partial child visibility and the summary's
        "delegation visibility limited progress confidence" counter-evidence, so the packet does not
        overclaim direct child execution progress.
    - `da59436e63915185`
      - current smoke: the run stayed mostly `insufficient_evidence` across planning/troubleshooting
        lanes, but checkpoints 25-26 reached `parent_visible_orchestration` (`insufficient_evidence`
        then `stalled`, confidence `low`) under `delegating_parent` / `opaque`.
      - packet-local expectation: this adapted witness should hold only a conservative
        `parent_visible_orchestration` lane with `low` confidence and `insufficient_evidence`-or-`stalled`
        status; anti-flap tightening beyond that remains explicitly deferred to `R5.75-4`.
      - supporting evidence to preserve: dense late-session `spawn_agent` / `wait_agent` markers that
        surface parent-visible delegation.
      - limiting evidence to preserve: opaque child visibility, repeated lack of child context across
        `Get-Content`, `Get-ChildItem`, and `.\.venv\Scripts\python`, and the existing prohibition on
        fabricating positive opaque-child progress.
  - Verify: run the native/adapted smoke commands from the companion spec and inspect
    `summary.md` + the first `checkpoints.jsonl` rows for each session.
  - Files:
    - no committed implementation files required; evidence recorded in this ledger and under
      `target/r5_75-smoke/R5.75-3/<session-id>/`

## R5.75-3.2: Stop Dropping Parent-Visible Progress On Planning Artifact Edits

- [ ] Task R5.75-3.2.1: Stabilize the parent-visible path when the parent edits plan/spec/handoff
      artifacts.
  - Acceptance: `crates/agent-drift-analyzer/src/checkpoint/progress.rs` no longer returns to generic
    planning-only handling solely because `plan_artifact_edits(analysis)` is non-empty when delegation
    markers and parent-visible synthesis/orchestration evidence are otherwise strong. Child-visibility
    limits remain explicit; the packet must not fabricate positive opaque-child progress.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - targeted smoke rerun for the named delegated sessions
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.75-3.3: Fast Delegated-Parent Regressions

- [ ] Task R5.75-3.3.1: Add/refresh fast checkpoint regressions for delegated parent-visible stability.
  - Acceptance: `crates/agent-drift-analyzer/tests/checkpoints.rs` proves all of the following:
    - planning/spec/handoff edits do not erase the delegated parent-visible lane when delegation
      evidence is otherwise strong,
    - limited child visibility remains visible as limiting/counter evidence,
    - the packet preserves conservative parent-visible interpretation instead of overclaiming child
      execution progress, and
    - any later comparability tweak stays narrow and additive.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`

## R5.75-3.4: Promote The First Native Delegated Real-Rollout Fixture

- [ ] Task R5.75-3.4.1: Admit the first native delegated real-rollout proof into the committed
      `progress_acceptance` corpus.
  - Acceptance: by default,
    `019eb970-3543-7ab1-a5d6-2a62c00c7185` becomes a committed `annotated_real_rollout` delegated case
    under `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/`, and the surrounding corpus
    contract is updated coherently:
    - `README.md` includes the new case and explains its delegated-parent role,
    - `progress_acceptance.rs` corpus-count / excluded-case assertions are updated honestly,
    - delegated cases remain guardrail-only in `R5`,
    - the synthetic delegated guardrail proof is retained unless there is an explicitly documented
      reason to replace it.
  - Verify:
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files:
    - `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/README.md`
    - `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/019eb970-3543-7ab1-a5d6-2a62c00c7185/**`

## R5.75-3.5: Conditional Comparability Follow-On (Only If Needed)

- [ ] Task R5.75-3.5.1: Apply one narrow parent-visible comparability-reset tweak only if the named
      repros still fail for that reason after Task `R5.75-3.2`.
  - Acceptance: only if the post-stabilization rerun shows that the remaining failure is specifically
    over-broad parent-visible comparability reset behavior, apply one additive tweak to
    `parent_visible_comparability_fingerprint(...)` in
    `crates/agent-drift-analyzer/src/checkpoint/progress.rs`, keeping the logic on the legacy
    `task_frame.objective` / delegated-objective-surface / working-set inputs. If the needed change is
    broader than that, stop and open a new packet instead of widening this one silently.
  - Verify:
    - `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
    - `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
    - targeted smoke rerun on the named delegated sessions
  - Files:
    - `crates/agent-drift-analyzer/src/checkpoint/progress.rs`
    - `crates/agent-drift-analyzer/tests/checkpoints.rs`

## R5.75-3.6: Automated Validation Gates

- [ ] Task R5.75-3.6.1: Run the packet's automated validation wall.
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

## R5.75-3.7: Named Native Smoke Review

- [ ] Task R5.75-3.7.1: Re-run the three named native delegated sessions and inspect the packet-owned
      outputs manually.
  - Acceptance:
    - `019eb970-3543-7ab1-a5d6-2a62c00c7185` remains the positive proof that the parent-visible path
      works and matches the committed delegated real-rollout expectation,
    - `019eb907-95c4-73e1-843e-e337d1e93cb9` and
      `019eb917-9531-74e0-897d-ad8d362138ec` stay in the explicit conservative delegated-parent lane
      recorded by Task `R5.75-3.1` instead of collapsing into generic planning-only noise,
    - limiting child-visibility evidence remains visible in the exported output.
  - Verify:
    - run the native smoke command block from the companion spec for each session
    - inspect:
      - `sed -n '1,80p' "$ANALYZER_OUT/summary.md"`
      - `sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"`
  - Files:
    - no new implementation files; manual smoke evidence only

## R5.75-3.8: Named Adapted Smoke Review And Packet-Boundary Audit

- [ ] Task R5.75-3.8.1: Re-run adapted delegated witness `da59436e63915185` and confirm the packet
      boundary stays honest.
  - Acceptance:
    - `da59436e63915185` holds the explicit parent-visible stability bar recorded by Task `R5.75-3.1`,
    - the packet does not overclaim child progress,
    - any residual anti-flap / zero-verifier concerns beyond that stability bar are explicitly left to
      `R5.75-4` instead of being silently absorbed here.
  - Verify:
    - run the adapted smoke command block from the companion spec
    - inspect:
      - `sed -n '1,80p' "$ANALYZER_OUT/summary.md"`
      - `sed -n '1,5p' "$ANALYZER_OUT/checkpoints.jsonl"`
  - Files:
    - no new implementation files; manual smoke evidence only

## Hard Gate For R5.75-3

Do not mark this packet complete unless:

- Task `R5.75-3.1` has locked explicit session-by-session expectations,
- planning/spec/handoff edits no longer erase the parent-visible lane,
- `019eb970-3543-7ab1-a5d6-2a62c00c7185` is admitted into committed `progress_acceptance` coverage,
- `019eb907-95c4-73e1-843e-e337d1e93cb9` and
  `019eb917-9531-74e0-897d-ad8d362138ec` no longer collapse into generic planning-only noise,
- adapted witness `da59436e63915185` holds the packet-owned parent-visible bar,
- all three analyzer gates are green, and
- the packet stayed narrow: no generalized fingerprint redesign, no adapted committed fixture-family
  expansion, and no structured-state comparability migration.

Automated green status alone does not satisfy this packet.

## Promotion Gate To R5.75-4

Do not promote to `R5.75-4` until every task above is complete and the native/adapted smoke review
proves the named delegated sessions all hold a conservative but stable parent-visible interpretation
that matches the characterization recorded for this packet.
