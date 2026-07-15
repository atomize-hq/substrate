# Tasks: R7 Bounded Delegated-Session Support

Canonical path:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-tasks.md`

Status: **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0 COMPLETE / R7-0.1 SERIES AND R7-0.2 COMMIT
`fa85cd4b8` FRESH INDEPENDENT REVIEW CLEAN / R7-1 ACTIVE AT ENTRY ONLY / ACTIVE PACKET NONE /
R7-0 -> R7-1 TRANSITION COMMIT PENDING FRESH REVIEW / R7-1.1 NEXT AND UNSTARTED / R7-2..R7-6 AND
R8 BLOCKED / PRODUCTION IMPLEMENTATION NOT STARTED**

Promotion series `455d0ed90` + `876ac55de` completed the content/gate audit and received fresh
independent built-in `default` `REVIEW CLEAN`, so `R7-PROMOTE` is complete. Transition series
`6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in `default` `REVIEW
CLEAN`. `R7-0.1` is complete after the exact contract `rg` passed; series `a9e75f149` +
`55bea5fa5` + `faff68ac6` is fresh independent built-in `default` `REVIEW CLEAN`. Fixture-only
`R7-0.2` commit `fa85cd4b8` also received fresh independent built-in `default` `REVIEW CLEAN`, so
`R7-0` is complete. `R7-1` is active at entry only with packet `none`; the narrow transition commit
awaits fresh review. `R7-1.1` is next and unstarted. Production implementation remains unstarted;
`R7-2..R7-6` plus R8 remain blocked.

## R7-PROMOTE: Implementation-Readiness Audit

- [x] **R7-PROMOTE.1: Reconcile the R7 authority family to implementation-ready content.**
  - Acceptance: MAP/SPEC/PLAN/TASKS lock reciprocal structured direct links, compactor-only raw
    parsing, analyzer-owned typed topology, separate parent/child trajectories, direct children
    only, no new drift class by default, minimal sentinel compatibility, and the R8 exclusion.
  - Verify: focused status/contract `rg`, complete four-file diff inspection, and `git diff --check`
  - Files: `docs/specs/r7/{MAP.md,*-spec.md,*-plan.md,*-tasks.md}`
  - Dependencies: review-clean R6 `CLOSED` authority
  - Receipt: promotion series `455d0ed90` + `876ac55de` fresh independent built-in `default`
    `REVIEW CLEAN`; transition series `6bf0ac6ad` + `4a887ee0c` + `e83ebb430` fresh independent
    built-in `default` `REVIEW CLEAN`; at that transition boundary `R7-0` was active at entry only,
    `R7-0.1` was next, and no `R7-0` item had started
  - Scope: medium, docs only

## R7-0: Docs Lock And Evidence Matrix

- [x] **R7-0.1: Freeze the implementation-ready R7 family at phase entry.**
  - Acceptance: the landed family records the `dead_end_thrash` core plus reciprocal direct
    linkage, separate trajectories, and R8 boundaries, and the review-clean transition identifies
    `R7-0` as the sole active phase before fixture work begins.
  - Verify: `rg -n "R6-1|reciprocal|separate trajector|R8" docs/specs/r6/MAP.md docs/specs/r7`
  - Files: `docs/specs/r6/MAP.md`, `docs/specs/r7/{MAP.md,*-spec.md,*-plan.md,*-tasks.md}`
  - Dependencies: review-clean `R7-PROMOTE` phase transition
  - Receipt: docs-only landing series `a9e75f149` + `55bea5fa5` + `faff68ac6` is fresh independent
    built-in `default` `REVIEW CLEAN`; the exact verification command passed and confirmed the
    `R6-1` core, reciprocal direct linkage, separate trajectories, direct-child-first boundary,
    no-new-drift-class-by-default posture, and R8 exclusion. No fixture, product behavior, or
    implementation symbol changed in that docs-only series.
  - Scope: medium, docs only

- [x] **R7-0.2: Add sanitized raw-link fixture matrix.**
  - Acceptance: fixtures cover reciprocal, parent-only, child-only, conflict, multi-child,
    nested-depth residue, and single-agent control without private prompt/tool content.
  - Verify: focused compactor fixture parser tests plus manual privacy review
  - Files: `crates/agent-session-compactor/tests/fixtures/delegation_links/**`, fixture README,
    `crates/agent-session-compactor/tests/delegation_link_fixtures.rs`
  - Dependencies: R7-0.1
  - Receipt: fixture-only commit `fa85cd4b8` received fresh independent built-in `default` `REVIEW
    CLEAN` with no actionable findings. Twelve JSONL files / `24` rows cover all seven accepted
    cases. The focused parser/privacy target passes `2 / 2`; full `agent-session-compactor` passes
    `25 / 25`, including end-to-end `2 / 2`; JSON parsing, formatting, and manual private-marker /
    raw-UUID scans pass with zero matches. No production symbol changed, and no raw private rollout
    was copied.
  - Scope: small

### Checkpoint R7-0

- [x] Fixture shapes match current raw Codex parent result and child `session_meta` source fields.
- [x] No raw private rollout is committed.
- [x] No implementation symbol was edited; therefore no pre-edit symbol impact analysis was
  applicable to `R7-0.2`.

Checkpoint receipt: all items above are complete and fresh-review-clean at fixture commit
`fa85cd4b8`; `R7-0` is complete. The narrow phase-transition commit is a separate review boundary.

## R7-1: Compactor Linkage And Direct-Child Closure

- [ ] **R7-1.1: Preserve parent spawn-result and child-origin metadata.**
  - Acceptance: ingestion exposes parent child ids, child parent id/depth, and provenance without
    turning metadata into ordinary message text.
  - Verify: `cargo test -p agent-session-compactor ingest -- --nocapture`
  - Files: `crates/agent-session-compactor/src/ingest/codex_rollout.rs`, focused tests
  - Dependencies: R7-0.2
  - Scope: small

- [ ] **R7-1.2: Add additive delegation link contract and reciprocal validation.**
  - Acceptance: only exact reciprocal links become `Verified`; one-sided/conflicting/self/duplicate
    cases remain typed non-semantic states; legacy manifests deserialize with no links.
  - Verify: `cargo test -p agent-session-compactor delegation_link -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/mod.rs`, `src/export/files.rs`, focused tests
  - Dependencies: R7-1.1
  - Scope: medium

- [ ] **R7-1.3: Add explicit direct linked-child discovery.**
  - Acceptance: opt-in compaction includes verified direct children only, records deeper residue,
    and leaves ordinary `--session-id` behavior unchanged.
  - Verify: `cargo test -p agent-session-compactor --test end_to_end -- --nocapture`
  - Files: `crates/agent-session-compactor/src/discovery.rs`, `src/cli.rs`, `src/lib.rs`, end-to-end tests
  - Dependencies: R7-1.2
  - Scope: medium

### Checkpoint R7-1

- [ ] `cargo test -p agent-session-compactor -- --nocapture`
- [ ] Manifest links and session/file ordering are deterministic.
- [ ] `gitnexus detect-changes -r 97a0-substrate` shows only expected compactor flows.

## R7-2: Analyzer Link Graph And Checkpoint v0.8

- [ ] **R7-2.1: Load and validate the typed direct link graph.**
  - Acceptance: verified links reference included sessions; missing/conflicting links remain bounded;
    v0.2 manifests without links still load.
  - Verify: `cargo test -p agent-drift-analyzer input -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/input.rs`, input tests
  - Dependencies: R7-1
  - Scope: medium

- [ ] **R7-2.2: Promote delegation context into checkpoint schema v0.8.**
  - Acceptance: v0.8 requires topology, parent/child ids, visibility, confidence, and evidence;
    v0.7 remains readable; `ChildWorkVisibility::Linked` is explicit.
  - Verify: schema round-trip and legacy compatibility tests
  - Files: `crates/agent-drift-analyzer/src/checkpoint/schema.rs`, schema/export tests
  - Dependencies: R7-2.1
  - Scope: medium

- [ ] **R7-2.3: Derive parent/child roles from graph truth.**
  - Acceptance: verified parent/child sessions become `DelegatingParent`/`DelegatedChild`; heuristic
    markers remain fallback-only; mixed closures become `Partial` or `MixedOrAmbiguous`.
  - Verify: `cargo test -p agent-drift-analyzer delegation -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/mod.rs`, `src/inference/mod.rs`, tests
  - Dependencies: R7-2.2
  - Scope: medium

### Checkpoint R7-2

- [ ] `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- [ ] Legacy R3.75 delegated and ordinary single-agent controls remain stable.
- [ ] Public field naming and compatibility receive interface review.

## R7-3: Separate Parent And Child Progress

- [ ] **R7-3.1: Analyze linked children through their own progress pipeline.**
  - Acceptance: each child checkpoint owns its archetype/progress; parent checkpoints remain
    parent-visible and reference child ids without copying child status.
  - Verify: `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/progress.rs`, progress acceptance fixtures/tests
  - Dependencies: R7-2
  - Scope: medium

- [ ] **R7-3.2: Pin cross-trajectory non-inference guardrails.**
  - Acceptance: parent wait/child advance, parent clean/child stall, and missing-child cases preserve
    separate statuses and evidence.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files: `crates/agent-drift-analyzer/tests/checkpoints.rs`, support/fixture files
  - Dependencies: R7-3.1
  - Scope: medium

### Checkpoint R7-3

- [ ] No parent checkpoint contains copied child `SessionProgress`.
- [ ] Parent-only opacity still yields insufficient evidence for child progress.
- [ ] Full analyzer progress wall passes.

## R7-4: Delegated Scorer Guardrails

- [ ] **R7-4.1: Keep drift scores trajectory-local.**
  - Acceptance: parent wait/orchestration cannot become child `dead_end_thrash`; child scores remain
    on child checkpoints; semantic-goal-drift opaque-parent guards remain intact.
  - Verify: focused `dead_end_thrash` and `semantic_goal_drift` tests
  - Files: `crates/agent-drift-analyzer/src/scoring/*.rs`, focused tests
  - Dependencies: R7-3
  - Scope: medium, split by scorer if more than five files are required

- [ ] **R7-4.2: Record the delegated drift taxonomy decision.**
  - Acceptance: either existing classes plus delegation context are sufficient, or a separate
    evidence-backed follow-on packet is opened; no opportunistic variant is added.
  - Verify: manual review against acceptance evidence and `DriftClass` impact analysis
  - Files: R7 docs and, only if separately approved, schema/compatibility files
  - Dependencies: R7-4.1
  - Scope: small docs decision

### Checkpoint R7-4

- [ ] `cargo test -p agent-drift-analyzer --test dead_end_thrash -- --nocapture`
- [ ] `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- [ ] `cargo test -p agent-drift-analyzer -- --nocapture`

## R7-5: Delegated Acceptance And Real-Corpus Proof

- [ ] **R7-5.1: Land the bounded delegated acceptance matrix.**
  - Acceptance: every spec matrix case asserts link state, topology, visibility, progress ownership,
    and scorer ownership.
  - Verify: dedicated delegated acceptance test
  - Files: analyzer acceptance test plus fixture directory/README
  - Dependencies: R7-4
  - Scope: medium

- [ ] **R7-5.2: Re-run named real delegated witnesses.**
  - Acceptance: sanitized parent/child proof and earlier R3.75/R5.75 witnesses are manually audited;
    results are stratified by topology and no raw rollout is committed.
  - Verify: documented compactor -> analyzer commands and report artifact
  - Files: `docs/specs/r7/FINDINGS-r7-delegated-session-validation.md`, scripts only if necessary
  - Dependencies: R7-5.1
  - Scope: medium

### Checkpoint R7-5

- [ ] Analyzer linkage/progress/scoring is review-clean before sentinel live changes.
- [ ] Single-agent controls remain unchanged.
- [ ] All missing/conflicting cases fail closed.

## R7-6: Minimal Sentinel Compatibility

- [ ] **R7-6.1: Add checkpoint v0.8 input and compact presentation.**
  - Acceptance: sentinel accepts v0.8, preserves legacy versions, and renders analyzer-owned
    topology/role/visibility without raw inference.
  - Verify: `cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/input.rs`, `src/operator_surface.rs`, focused tests
  - Dependencies: R7-5
  - Scope: medium

- [ ] **R7-6.2: Support verified linked closure in real-session live mode.**
  - Acceptance: root plus verified direct children are accepted; cursors remain monotonic per
    session; unexpected sessions still fail closed; scheduling policy stays unchanged.
  - Verify: `cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture`
  - Files: `crates/agent-drift-sentinel/src/real_session_live.rs`, live runtime/state tests
  - Dependencies: R7-6.1
  - Scope: medium

### Final Checkpoint

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test -p agent-session-compactor -- --nocapture`
- [ ] `cargo test -p agent-drift-analyzer -- --nocapture`
- [ ] `cargo test -p agent-drift-sentinel -- --nocapture`
- [ ] `cargo test --workspace -- --nocapture`
- [ ] `git diff --check`
- [ ] `npx gitnexus detect-changes -r 97a0-substrate`
- [ ] Fresh review confirms R8 consolidation, recursive graphs, and R6 reopenings did not leak in.
