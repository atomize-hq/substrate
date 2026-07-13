# Tasks: R7 Bounded Delegated-Session Support

Canonical path:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-tasks.md`

Status: **DRAFT / BLOCKED ON R6 CLOSURE DECISION**

All unchecked items are inactive until
`docs/specs/r6/FINDINGS-r6-scorer-context-cutover-closure.md` reaches `CLOSED`. The exact gate is:
the applicability audit is complete; every material scorer is complete, intentionally exempt, or
still open; broad R6 acceptance claims are behaviorally proven or narrowed honestly; named closure
controls are resolved; and the root/R6/R7 authority stack agrees.

## R7-0: Docs Lock And Evidence Matrix

- [x] **R7-0.1: Preserve the R7 spec/plan/tasks family as draft design work.**
  - Acceptance: R6 docs state the `dead_end_thrash` core landed but broad R6 closure remains partial;
    the R7 map, spec, plan, and tasks define reciprocal direct linkage, separate trajectories, and
    R8 boundaries without claiming implementation readiness.
  - Verify: `rg -n "R6-1|reciprocal|separate trajector|R8" docs/specs/r6/MAP.md docs/specs/r7`
  - Files: `docs/specs/r6/MAP.md`, `docs/specs/r7/{MAP.md,*-spec.md,*-plan.md,*-tasks.md}`
  - Dependencies: R6 closure finding for any implementation use
  - Scope: medium, docs only

- [ ] **R7-0.2: Add sanitized raw-link fixture matrix.**
  - Acceptance: fixtures cover reciprocal, parent-only, child-only, conflict, multi-child,
    nested-depth residue, and single-agent control without private prompt/tool content.
  - Verify: focused compactor fixture parser tests plus manual privacy review
  - Files: `crates/agent-session-compactor/tests/fixtures/delegation_links/**`, fixture README
  - Dependencies: R7-0.1
  - Scope: small

### Checkpoint R7-0

- [ ] Fixture shapes match current raw Codex parent result and child `session_meta` source fields.
- [ ] No raw private rollout is committed.
- [ ] Implementation symbols receive GitNexus impact analysis before editing.

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
