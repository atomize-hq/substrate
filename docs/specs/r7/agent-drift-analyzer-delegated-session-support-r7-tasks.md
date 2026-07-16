# Tasks: R7 Bounded Delegated-Session Support

Canonical path:
`docs/specs/r7/agent-drift-analyzer-delegated-session-support-r7-tasks.md`

Status: **IMPLEMENTATION-READY / R7-PROMOTE AND R7-0..R7-3 COMPLETE / R7-4 SOLE ACTIVE
PHASE — CHECKPOINT-DOC RECEIPT GATE / ACTIVE PACKET NONE / R7-4.1 COMMIT/FIX SERIES `ebcb052b9` +
`e7b65523f` AND R7-4.2 DOCS DECISION COMMIT `8a0790a3d` FRESH INDEPENDENT BUILT-IN `default`
`REVIEW CLEAN` / R7-4.1, R7-4.2, AND BEHAVIOR/STATIC CHECKPOINT COMPLETE / CHECKPOINT-DOC RECEIPT
PENDING FRESH INDEPENDENT REVIEW / R7-4 EXIT GATE AND `CTX-R7-05` BLOCKED/PENDING / R7-5.1
UNSTARTED / R7-5..R7-6 AND R8 BLOCKED / NO NEXT-PHASE SELECTORS PREPARED OR INVOKED**

Promotion series `455d0ed90` + `876ac55de` completed the content/gate audit and received fresh
independent built-in `default` `REVIEW CLEAN`, so `R7-PROMOTE` is complete. Transition series
`6bf0ac6ad` + `4a887ee0c` + `e83ebb430` received fresh independent built-in `default` `REVIEW
CLEAN`. `R7-0.1` is complete after the exact contract `rg` passed; series `a9e75f149` +
`55bea5fa5` + `faff68ac6` is fresh independent built-in `default` `REVIEW CLEAN`. Fixture-only
`R7-0.2` commit `fa85cd4b8` also received fresh independent built-in `default` `REVIEW CLEAN`, so
`R7-0` is complete. Transition/fix series `339744dff` + `d20cac6a9` also received fresh independent
built-in `default` `REVIEW CLEAN`. `R7-1.1` series `e65127720` + `685cf843b`, `R7-1.2` commit
`4d122cd9f`, and `R7-1.3` commit `e865eee13` each received fresh independent built-in `default`
`REVIEW CLEAN`. Checkpoint-doc commit `1cae7d693` received fresh independent built-in `default`
`REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task commits `c60d05f77` and
`9403c8a24`, plus R7-2.3 series `7af2ae517` + `75a353e46`, are fresh independent built-in `default`
`REVIEW CLEAN`; the fix reconciled the summary-vs-checkpoint blocker. R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc
commit `78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the
R7-2 exit gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series
`e077de489` + `3dd5ba943` remains fresh independent built-in `default` `REVIEW CLEAN`. R7-4.1
commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs decision commit `8a0790a3d` each
received fresh independent built-in `default` `REVIEW CLEAN`; R7-4.1, R7-4.2, and the behavior/
static checkpoint are complete. R7-4 remains the sole active phase with packet `none` while this
checkpoint-doc receipt still requires fresh independent review. The R7-4 exit gate and `CTX-R7-05`
remain blocked/pending until the receipt itself is fresh-review-clean. `R7-5..R7-6` and R8 remain
blocked; `R7-5.1` is unstarted; no next-phase selectors are prepared or invoked.

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
`fa85cd4b8`; `R7-0` is complete. Separate transition/fix series `339744dff` + `d20cac6a9` received
fresh independent built-in `default` `REVIEW CLEAN`. At that historical R7-0 boundary, Prompt 1 for
R7-1 was prepared but had not started.

## R7-1: Compactor Linkage And Direct-Child Closure

- [x] **R7-1.1: Preserve parent spawn-result and child-origin metadata.**
  - Acceptance: ingestion exposes parent child ids, child parent id/depth, and provenance without
    turning metadata into ordinary message text.
  - Verify: `cargo test -p agent-session-compactor ingest -- --nocapture`
  - Files: `crates/agent-session-compactor/src/ingest/codex_rollout.rs`, focused tests
  - Dependencies: R7-0.2
  - Receipt: implementation/docs series `e65127720` + `685cf843b` received fresh independent
    built-in `default` `REVIEW CLEAN`. Structured parent spawn-result and child-origin metadata,
    including row provenance, remains outside ordinary normalized message text.
  - Scope: small

- [x] **R7-1.2: Add additive delegation link contract and reciprocal validation.**
  - Acceptance: only exact reciprocal links become `Verified`; one-sided/conflicting/self/duplicate
    cases remain typed non-semantic states; legacy manifests deserialize with no links.
  - Verify: `cargo test -p agent-session-compactor delegation_link -- --nocapture`
  - Files: `crates/agent-session-compactor/src/export/mod.rs`, `src/export/files.rs`, focused tests
  - Dependencies: R7-1.1
  - Receipt: after operator decision `R7-1-HIGH-IMPACT-COMPACTOR-CONTRACT-01: A` accepted the bounded
    additive contract impact, commit `4d122cd9f` received fresh independent built-in `default`
    `REVIEW CLEAN`. Focused delegation-link proof passes `6 / 6`; exact reciprocal depth-1 links are
    verified, non-reciprocal/conflicting/self/duplicate/malformed cases remain typed non-semantic
    states, legacy v0.2 manifests default to no links, and link ordering is deterministic.
  - Scope: medium

- [x] **R7-1.3: Add explicit direct linked-child discovery.**
  - Acceptance: opt-in compaction includes verified direct children only, records deeper residue,
    and leaves ordinary `--session-id` behavior unchanged.
  - Verify: `cargo test -p agent-session-compactor --test end_to_end -- --nocapture`
  - Files: `crates/agent-session-compactor/src/discovery.rs`, `src/cli.rs`, `src/lib.rs`, end-to-end tests
  - Dependencies: R7-1.2
  - Receipt: commit `e865eee13` received fresh independent built-in `default` `REVIEW CLEAN`.
    End-to-end proof passes `6 / 6` and CLI proof passes `2 / 2`: the opt-in includes only exact
    verified direct children, records deeper residue without recursive import, fails closed on
    ambiguous/non-verified candidates, and preserves ordinary discovery when the option is absent.
  - Scope: medium

### Checkpoint R7-1

- [x] `cargo test -p agent-session-compactor -- --nocapture`
- [x] Manifest links and session/file ordering are deterministic.
- [x] `gitnexus detect-changes -r 97a0-substrate` shows only expected compactor flows.

Checkpoint receipt: all R7-1 task commits are fresh independent built-in `default` `REVIEW CLEAN`.
The full compactor wall passes `36` unit/integration tests plus `3` doctests; focused R7-1.2 proof
passes `6 / 6`; R7-1.3 end-to-end and CLI proof pass `6 / 6` and `2 / 2`. Formatting, clippy, diff,
and staged GitNexus gates are green. Deterministic link/session/file ordering is proven, and no raw
private rollout data was added. Checkpoint-doc commit `1cae7d693` received fresh independent
built-in `default` `REVIEW CLEAN`, satisfying the R7-1 exit gate. R7-1 is complete. R7-2 task
commits `c60d05f77` and `9403c8a24`, plus R7-2.3 series `7af2ae517` + `75a353e46`, are fresh
independent built-in `default` `REVIEW CLEAN`; `75a353e46` fixed the summary-vs-checkpoint blocker.
R7-2.1, R7-2.2, R7-2.3, and the behavior/static checkpoint are complete. Checkpoint-doc commit
`78a168c09` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-2 exit
gate and proving `CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series
`e077de489` + `3dd5ba943` remains fresh independent built-in `default` `REVIEW CLEAN`. R7-4.1
commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs decision commit `8a0790a3d` each
received fresh independent built-in `default` `REVIEW CLEAN`; R7-4.1, R7-4.2, and the behavior/
static checkpoint are complete. R7-4 remains the sole active phase with packet `none` while this
checkpoint-doc receipt still requires fresh independent review. The R7-4 exit gate and `CTX-R7-05`
remain blocked/pending until the receipt itself is fresh-review-clean. `R7-5..R7-6` and R8 remain
blocked; `R7-5.1` is unstarted; no next-phase selectors are prepared or invoked.

## R7-2: Analyzer Link Graph And Checkpoint v0.8

- [x] **R7-2.1: Load and validate the typed direct link graph.**
  - Acceptance: verified links reference included sessions; missing/conflicting links remain bounded;
    v0.2 manifests without links still load.
  - Verify: `cargo test -p agent-drift-analyzer input -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/input.rs`, input tests
  - Dependencies: R7-1
  - Receipt: commit `c60d05f77` received fresh independent built-in `default` `REVIEW CLEAN`.
    Verified links load only when their sessions are included, missing/conflicting states remain
    bounded, legacy v0.2 manifests without links still load, and the input filter passes `16 / 16`.
    Its staged GitNexus gate reported LOW risk with `0` affected processes.
  - Scope: medium

- [x] **R7-2.2: Promote delegation context into checkpoint schema v0.8.**
  - Acceptance: v0.8 requires topology, parent/child ids, visibility, confidence, and evidence;
    v0.7 remains readable; `ChildWorkVisibility::Linked` is explicit.
  - Verify: schema round-trip and legacy compatibility tests
  - Files: `crates/agent-drift-analyzer/src/checkpoint/schema.rs`, schema/export tests
  - Dependencies: R7-2.1
  - Receipt: operator decision `R7-2-HIGH-IMPACT-ANALYZER-CONTRACT-01: A` authorized the bounded
    high-impact analyzer seam. Commit `9403c8a24` received fresh independent built-in `default`
    `REVIEW CLEAN`. Public checkpoint v0.8 carries required topology, role ids, visibility,
    confidence, and deterministic `RowRef` evidence; v0.7 remains readable; `Linked` is explicit.
    Its staged GitNexus gate reported MEDIUM risk with `1` affected process.
  - Scope: medium

- [x] **R7-2.3: Derive parent/child roles from graph truth.**
  - Acceptance: verified parent/child sessions become `DelegatingParent`/`DelegatedChild`; heuristic
    markers remain fallback-only; mixed closures become `Partial` or `MixedOrAmbiguous`.
  - Verify: `cargo test -p agent-drift-analyzer delegation -- --nocapture`
  - Files: `crates/agent-drift-analyzer/src/checkpoint/mod.rs`, `src/inference/mod.rs`, tests
  - Dependencies: R7-2.2
  - Receipt: implementation `7af2ae517` plus fix `75a353e46` received fresh independent built-in
    `default` `REVIEW CLEAN` after the fix aligned JSON summary delegation with checkpoint truth.
    Verified graph roles and parent/child ids are authoritative; mixed visibility becomes `Partial`;
    conflicts fail closed; heuristic markers remain fallback-only; JSON-summary parity and separate
    trajectories are proven. Staged GitNexus reported HIGH / `9` affected processes for the
    authorized implementation and MEDIUM / `2` for the fix.
  - Scope: medium

### Checkpoint R7-2

- [x] `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
- [x] Legacy R3.75 delegated and ordinary single-agent controls remain stable.
- [x] Public field naming and compatibility receive interface review.

Checkpoint receipt: at implementation HEAD `75a353e46`, the input filter passes `16 / 16`;
delegation matches pass `39` total (`25` library + `4` checkpoint + `8` delegation-context + `2`
export); checkpoint matches pass `172` total (`36` library + `134` checkpoints + `1` export + `1`
truth-grounding); and the full analyzer passes `417 / 417`. `cargo fmt --all -- --check`, `cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and `git diff --check` are green. The public v0.8/v0.7 compatibility,
verified graph roles and ids, `Linked`/`Partial` visibility, fail-closed conflicts, deterministic
`RowRef` evidence, JSON-summary parity, and separate trajectories are proven. No R7-3, R7-4,
sentinel, or R8 work leaked into the series. Checkpoint-doc commit `78a168c09` received fresh
independent built-in `default` `REVIEW CLEAN`, satisfying the R7-2 exit gate and proving
`CTX-R7-03`. R7-2 is complete. Transition/fix series `e27d82580` + `305e40bf2` and entry-authority
repair `9fd9d9972` received fresh independent built-in `default` `REVIEW CLEAN`. R7-3.1 test-only
commit `f8dd04549` and R7-3.2 test-only commit `c7c6f35b8` each received fresh independent built-in
`default` `REVIEW CLEAN`; R7-3.1, R7-3.2, and the behavior/static checkpoint are complete.
Checkpoint-doc commit `931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`,
satisfying the R7-3 exit gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series
`e077de489` + `3dd5ba943` remains fresh independent built-in `default` `REVIEW CLEAN`. R7-4.1
commit/fix series `ebcb052b9` + `e7b65523f` and R7-4.2 docs decision commit `8a0790a3d` each
received fresh independent built-in `default` `REVIEW CLEAN`; R7-4.1, R7-4.2, and the behavior/
static checkpoint are complete. R7-4 remains the sole active phase with packet `none` while this
checkpoint-doc receipt still requires fresh independent review. The R7-4 exit gate and `CTX-R7-05`
remain blocked/pending until the receipt itself is fresh-review-clean. `R7-5..R7-6` and R8 remain
blocked; `R7-5.1` is unstarted; no next-phase selectors are prepared or invoked.

## R7-3: Separate Parent And Child Progress

- [x] **R7-3.1: Analyze linked children through their own progress pipeline.**
  - Acceptance: each child checkpoint owns its archetype/progress; parent checkpoints remain
    parent-visible and reference child ids without copying child status.
  - Verify: `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture`
  - Files: `crates/agent-drift-analyzer/tests/progress_acceptance.rs`
  - Dependencies: R7-2
  - Receipt: test-only commit `f8dd04549` received fresh independent built-in `default` `REVIEW
    CLEAN`. Its placeholder acceptance scaffold intentionally produced test-scaffold RED `0 / 1`;
    after replacement with the real acceptance assertion, its exact target passed `1 / 1`, and full
    `progress_acceptance` passed `4 / 4`. Parent progress remains
    `ParentVisibleOrchestration / InsufficientEvidence`; the child retains its own
    verification-closeout progress and session-local evidence. Staged GitNexus reported LOW risk
    with `0` affected processes.
  - Scope: medium

- [x] **R7-3.2: Pin cross-trajectory non-inference guardrails.**
  - Acceptance: parent wait/child advance, parent clean/child stall, and missing-child cases preserve
    separate statuses and evidence.
  - Verify: `cargo test -p agent-drift-analyzer checkpoints -- --nocapture`
  - Files: `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - Dependencies: R7-3.1
  - Receipt: test-only commit `c7c6f35b8` received fresh independent built-in `default` `REVIEW
    CLEAN`. Its placeholder acceptance scaffolds intentionally produced test-scaffold RED `3 / 3`;
    after replacement with the real acceptance assertions, exact `3 / 3` passed, and the checkpoint
    filter passed `175` matched tests across targets. Parent wait does not absorb
    child advancement, clean parent orchestration does not inherit child stall, and a missing child
    leaves the parent `Opaque / InsufficientEvidence` without fabricating a child trajectory.
    Staged GitNexus reported LOW risk with `0` affected processes.
  - Scope: medium

### Checkpoint R7-3

- [x] No parent checkpoint contains copied child `SessionProgress`.
- [x] Parent-only opacity still yields insufficient evidence for child progress.
- [x] Full analyzer progress wall passes.

Checkpoint receipt: at implementation HEAD `c7c6f35b8`, R7-3.1's placeholder acceptance scaffold
intentionally produced test-scaffold RED `0 / 1`; after replacement with the real acceptance
assertion, exact `1 / 1` and full `progress_acceptance` `4 / 4` passed. R7-3.2's placeholder
acceptance scaffolds intentionally produced test-scaffold RED `3 / 3`; after replacement with the
real acceptance assertions, exact `3 / 3` passed. The
`cargo test -p agent-drift-analyzer checkpoints -- --nocapture` filter passed `175` matched tests
across targets. `cargo test -p agent-drift-analyzer -- --nocapture` passes `421 / 421` aggregate.
`cargo fmt --all -- --check`,
`cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and `git diff --check` are green.
Both staged GitNexus gates were LOW / `0` affected processes.
`cargo clippy --workspace --all-targets -- -D warnings` remains RED only in already-planned R7-6.1
sentinel v0.8 compatibility: test constructors at `crates/agent-drift-sentinel/tests/support/mod.rs:81`,
`crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs:48`, and
`crates/agent-drift-sentinel/tests/replay_input.rs:79` are missing `Checkpoint.delegation`. Do not
fix that R7-6-owned witness in R7-3 and do not claim workspace clippy green. Checkpoint-doc commit
`931c2701c` received fresh independent built-in `default` `REVIEW CLEAN`, satisfying the R7-3 exit
gate and proving `CTX-R7-04`. R7-3 is complete. Transition/fix series `e077de489` + `3dd5ba943`
remains fresh independent built-in `default` `REVIEW CLEAN`.

## R7-4: Delegated Scorer Guardrails

- [x] **R7-4.1: Keep drift scores trajectory-local.**
  - Acceptance: parent wait/orchestration cannot become child `dead_end_thrash`; child scores remain
    on child checkpoints; semantic-goal-drift opaque-parent guards remain intact.
  - Verify: focused `dead_end_thrash` and `semantic_goal_drift` tests
  - Files: `crates/agent-drift-analyzer/tests/dead_end_thrash.rs`
  - Dependencies: R7-3
  - Receipt: commit `ebcb052b9` added the production-shaped linked parent/child scorer witness.
    The first fresh reviewer returned `CHANGES_REQUIRED` with one P2/Important contract failure:
    the fixture hand-built `Verified` linkage with untruthful provenance (no matching spawn-result
    child identity, child evidence pointed at the goal instead of child-origin `session_meta`, and
    `discovered_file_count` was `1` for two files). Fix `e7b65523f` routes the witness through
    production `export_bundle` using a matching spawn call/result, child-origin metadata at event
    `0`, and discovery count `2`; fresh independent built-in `default` re-review returned `CLEAN`
    with no actionable findings.
  - Scope: medium, split by scorer if more than five files are required

- [x] **R7-4.2: Record the delegated drift taxonomy decision.**
  - Acceptance: either existing classes plus delegation context are sufficient, or a separate
    evidence-backed follow-on packet is opened; no opportunistic variant is added.
  - Decision: existing classes plus typed delegation context are sufficient for the production-shaped
    R7-4.1 evidence: parent waits remain cleared, child-local repetition remains active
    `dead_end_thrash`, and topology/visibility preserve ownership. No new variant was added; any
    future distinct, unexpressible delegated failure mode requires a separate evidence-backed
    reviewed packet before schema or compatibility edits.
  - Verify: manual review against acceptance evidence and `DriftClass` impact analysis
  - Files: R7 docs and, only if separately approved, schema/compatibility files
  - Dependencies: R7-4.1
  - Receipt: docs decision commit `8a0790a3d` received fresh independent built-in `default` `CLEAN`
    with no actionable findings. `DriftClass` remained unchanged.
  - Scope: small docs decision

### Checkpoint R7-4

- [x] `cargo test -p agent-drift-analyzer --test dead_end_thrash -- --nocapture`
- [x] `cargo test -p agent-drift-analyzer semantic_goal_drift -- --nocapture`
- [x] `cargo test -p agent-drift-analyzer -- --nocapture`

Checkpoint receipt candidate at implementation/docs HEAD `8a0790a3d`: `dead_end_thrash` passes
`19 / 19`; the `semantic_goal_drift` filter passes `58 / 58` aggregate (`56` library plus `2`
acceptance); and the full analyzer passes `422 / 422`. `cargo fmt --all -- --check`,
`cargo clippy -p agent-drift-analyzer --all-targets -- -D warnings`, and `git diff --check` are
green. The known workspace-clippy RED remains owned by already-planned R7-6.1 and was neither
rerun nor fixed in this checkpoint. R7-4.1, R7-4.2, and the behavior/static checkpoint are complete,
but R7-4 remains the sole active phase with packet `none` until this checkpoint-doc receipt receives
fresh independent review. The R7-4 exit gate and `CTX-R7-05` remain blocked/pending; `R7-5.1` is
unstarted, `R7-5..R7-6` and R8 remain blocked, and no next-phase selectors are prepared or invoked.

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
