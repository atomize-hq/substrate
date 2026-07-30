# SFR-RB-100 TASKS — Current-Native Recall Corpus

Status: **APPROVED 2026-07-29 — EXECUTION ACTIVE**

Specification:
`docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-spec.md`.

Plan:
`docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-plan.md`.

Implementation authority: **P7 ONLY**. Execute this ledger autonomously in dependency order with
task-scoped verification and commits. Production Rust remains outside the default fence.

Task sizing counts the shared harness, matrix, one self-contained case directory, and any exact owner
test as logical paths. A case directory may contain multiple JSON/JSONL files but remains one bounded
fixture surface. No task authorizes a production source edit.

## P7-0 — Planning review and baseline lock

### Task P7-0.1 — Complete the bounded planning review

- [x] Send the exact spec, plan, and task ledger to the existing GPT Pro consultation.
- [x] Receive a bounded review covering only contradictions, missing coverage, dependency order,
      task sizing, ambiguous acceptance, privacy, and scope leakage.
- [x] Reconcile actionable findings without reopening P0–P6 or historical R6–R8 behavior.
- **Acceptance:** review output is saved locally; every in-boundary finding is accepted, rejected with
  repository evidence, or converted into an explicit question; the three docs stay mutually
  consistent.
- **Verification:** record the reviewed attachment hashes, post-reconciliation hashes, bounded
  document delta, and consultation URL.
- **Dependencies:** confirmed specification.
- **Files likely touched:**
  - `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-spec.md`
  - `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-plan.md`
  - `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-tasks.md`
- **Estimated scope:** M — 3 documents.

### Task P7-0.2 — Lock the implementation baseline and protected walls

- [x] Obtain user approval of the reviewed planning family.
- [x] Verify branch, HEAD, ancestry, worktree/index state, and exact P7 path fence.
- [x] Capture the current focused-test inventory and confirm frozen corpus counts.
- **Acceptance:** implementation begins from one exact clean commit descended from
  `133b55249f88492e16f80f98a63368911d733c7e`; semantic 18-case, progress 16-case, and delegated
  10-case walls are recorded as protected.
- **Verification:**
  - `git status --short --branch`
  - `git rev-parse HEAD`
  - `git merge-base --is-ancestor 133b55249f88492e16f80f98a63368911d733c7e HEAD`
  - focused test listing and fixture-directory counts
- **Dependencies:** P7-0.1.
- **Files likely touched:** none; optionally a bounded run note.
- **Estimated scope:** XS.

### Task P7-0.3 — Run implementation preflight and GitNexus scope analysis

- [x] Refresh the GitNexus index if stale.
- [x] Query the existing cross-layer flow and record the intended test-only blast radius.
- [x] Confirm no production Rust symbol is authorized.
- **Acceptance:** preflight records the existing Sentinel/compactor/analyzer seam and a
  test/fixture/script/doc-only default fence.
- **Verification:** GitNexus query/context output plus `git diff --check`.
- **Dependencies:** P7-0.2.
- **Files likely touched:** none.
- **Estimated scope:** XS.

### Checkpoint P7-A — Planning lock

- [x] GPT Pro planning review reconciled.
- [x] User approved the reviewed family.
- [x] Exact clean baseline and protected walls recorded.
- [x] No implementation or production edit occurred before this checkpoint.

## P7-1 — Harness, matrix, privacy, and provenance

### Task P7-1.1 — Create the P7 matrix and harness skeleton

- [x] Add the dedicated Sentinel integration target and fixture root.
- [x] Define matrix parsing, case IDs, adapter class, parity mode, terminal boundary, diagnostic
      owners, exact expectations, and historical references.
- [x] Fail closed on missing, duplicate, extra, or unknown cases.
- **Acceptance:** the harness compiles; all nineteen planned case IDs are declared; an internal
  negative mutation or synthetic skeleton inventory proves missing, duplicate, extra, and unknown
  inventory produces a case-specific failure without leaving the tracked P7 target intentionally
  red while substantive fixtures are built.
- **Verification:**
  - `cargo test -p agent-drift-sentinel --test current_native_recall -- --nocapture`
  - `git diff --check`
- **Dependencies:** P7-A.
- **Files likely touched:**
  - `crates/agent-drift-sentinel/tests/current_native_recall.rs`
  - `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/matrix.json`
  - `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/README.md`
- **Estimated scope:** M — 3 files.

### Task P7-1.2 — Enforce recursive privacy and provenance

- [x] Define synthetic grammars for session, turn, call, event, agent, repository, and path values.
- [x] Recursively scan every P7 fixture and metadata file for forbidden private markers.
- [x] Verify declared adapter class and event variants against actual raw records.
- **Acceptance:** privacy/provenance violations fail with exact `case_id` and file; a test mutation
  proves each rule can fail; no local absolute path or real identifier is committed.
- **Verification:** focused P7 test plus targeted negative controls inside the harness.
- **Dependencies:** P7-1.1.
- **Files likely touched:**
  - `crates/agent-drift-sentinel/tests/current_native_recall.rs`
  - `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/matrix.json`
  - `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/README.md`
- **Estimated scope:** M — 3 files.

### Checkpoint P7-B — Foundation

- [x] Matrix schema self-tests pass; internal negative mutations prove inventory checks fail closed.
- [x] Privacy and provenance checks pass and fail closed under mutation.
- [x] Harness failures name case, terminal boundary, and diagnostic owner.
- [x] The tracked P7 target is not intentionally red while later case directories are incomplete.
- [x] No production source or frozen corpus changed.

## P7-2 — Semantic positive and conservative slices

Unless a task says otherwise, each case touches:

- `crates/agent-drift-sentinel/tests/current_native_recall.rs`;
- `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/matrix.json`; and
- one case directory under
  `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/<case-id>/`.

Each task runs the P7 target. After every third case, also run the frozen semantic and progress walls.

### Task P7-2.1 — P7-01 typed semantic-goal-drift positive

- [x] Preserve a raw stream routed as `CurrentNativeV2`, typed tool input/output ordering, turn
      identity, and a true abandoned-goal pivot.
- [x] Traverse direct closure, bundle-v0.2 export, analyzer, and public-live delivery.
- **Acceptance:** exact declared semantic signal, score/state/progress, evidence, and public-live
  result match; no legacy-shaped approximation is used.
- **Verification:** P7 target plus frozen semantic wall.
- **Dependencies:** P7-B.
- **Estimated scope:** M — shared harness, matrix, one case directory.

### Task P7-2.2 — P7-02 positive Legacy/CurrentNativeV2 parity

- [ ] Express equivalent legacy and `CurrentNativeV2` raw streams for one true pivot.
- [ ] Compare only canonical semantic projections.
- **Acceptance:** both paths produce the same declared semantic projection; unstable manifest fields
  are excluded explicitly.
- **Verification:** P7 target.
- **Dependencies:** P7-2.1.
- **Estimated scope:** M.

### Task P7-2.3 — P7-03 semantic alignment control

- [ ] Add a current-native aligned-objective case.
- **Acceptance:** exact conservative signal/state/`NoClaim` outcome is pinned; absence of a panic is
  insufficient.
- **Verification:** P7 target plus frozen semantic/progress walls.
- **Dependencies:** P7-2.2.
- **Estimated scope:** M.

### Task P7-2.4 — P7-04 path narrowing/progression control

- [ ] Add a legitimate directory/file or artifact-family narrowing/progression witness.
- **Acceptance:** the exact narrowing/progression result stays non-drift and preserves expected
  progress semantics.
- **Verification:** P7 target.
- **Dependencies:** P7-2.3.
- **Estimated scope:** M.

### Task P7-2.5 — P7-05 sanctioned replan control

- [ ] Add a structurally explicit sanctioned replan or the repository’s exact equivalent
      conservative control.
- **Acceptance:** the case is suppressed for the declared reason, not because its objective or
  evidence disappeared.
- **Verification:** P7 target.
- **Dependencies:** P7-2.4.
- **Estimated scope:** M.

### Task P7-2.6 — P7-06 zero-test execution control

- [ ] Preserve a zero-test execution result through typed output normalization.
- **Acceptance:** exact progress dimension/status and `NoClaim` behavior are pinned; the case forbids
  `verification_clean`, clean recovery, and false frontier advancement.
- **Verification:** P7 target plus frozen progress wall.
- **Dependencies:** P7-2.5.
- **Estimated scope:** M.

### Task P7-2.7 — P7-07 directive path non-authority control

- [ ] Preserve directive/path syntax in a raw current-native task surface.
- **Acceptance:** the directive cannot grant path authority or produce false semantic alignment; exact
  working-set/authority projection is asserted.
- **Verification:** P7 target.
- **Dependencies:** P7-2.6.
- **Estimated scope:** M.

### Task P7-2.8 — P7-08 lexical collision positive

- [ ] Add a prefix, hyphen, stem, or sibling collision where the new target is genuinely unrelated.
- **Acceptance:** lexical similarity cannot suppress the real pivot; exact signal and path identities
  are pinned.
- **Verification:** P7 target plus frozen semantic wall.
- **Dependencies:** P7-2.7.
- **Estimated scope:** M.

### Checkpoint P7-C — Semantic wall

- [ ] P7-01 through P7-08 pass.
- [ ] Frozen semantic 18-case and progress 16-case walls pass unchanged.
- [ ] Positive recall traverses the full production composition.
- [ ] Conservative cases fail closed for exact declared reasons.

## P7-3 — Session isolation, delegation, and closure

### Task P7-3.1 — P7-09 unrelated-session isolation

- [ ] Place semantically tempting evidence in an unrelated rollout outside selected closure.
- **Acceptance:** unrelated evidence cannot affect selected bundle, score, or public observation; the
  manifest contains only selected session IDs.
- **Verification:** P7 target plus `real_session_live`.
- **Dependencies:** P7-C.
- **Estimated scope:** M.

### Task P7-3.2 — P7-10 typed delegation linkage

- [ ] Preserve typed parent spawn/result, child origin, turn identity, and agent message/activity.
- [ ] Load the production-generated bundle-v0.2 through the analyzer without independently
      reconstructing delegation metadata.
- **Acceptance:** exact direct delegation link and file-registry projection survive ingest/export;
  analyzer interpretation retains exact parent/child topology, child-work visibility, linkage
  evidence, and applicable ownership semantics; no transitive discovery occurs.
- **Verification:** P7 target plus `current_native_adapter`, `bounded_closure`, `export_bundle`, and
  `delegated_acceptance`.
- **Dependencies:** P7-3.1.
- **Estimated scope:** M.

### Task P7-3.3 — P7-11 registered metadata-only child

- [ ] Include a verified child in the registry with no compact rows.
- **Acceptance:** export and analyzer accept the child as metadata-only and retain exact linkage
  state.
- **Verification:** P7 target plus `export_bundle` and `delegated_acceptance`.
- **Dependencies:** P7-3.2.
- **Estimated scope:** M.

### Task P7-3.4 — P7-12 missing verified child

- [ ] Remove the required child from the registry while preserving the verified link.
- **Acceptance:** analyzer rejects the bundle with the selected exact missing-session/input error;
  it is not treated as metadata-only.
- **Verification:** P7 target plus `input_contract`.
- **Dependencies:** P7-3.3.
- **Estimated scope:** M.

### Task P7-3.5 — P7-13 malformed unrelated source

- [ ] Add a malformed rollout outside selected direct closure.
- **Acceptance:** selection excludes it and the selected pipeline succeeds; no deeper residue is
  decoded.
- **Verification:** P7 target plus `bounded_closure`.
- **Dependencies:** P7-3.4.
- **Estimated scope:** M.

### Task P7-3.6 — P7-14 malformed selected source

- [ ] Place equivalent malformed structure inside selected direct closure.
- **Acceptance:** closure verification returns the exact declared error and no analyzer/public-live
  success is emitted.
- **Verification:** P7 target plus `bounded_closure`.
- **Dependencies:** P7-3.5.
- **Estimated scope:** M.

### Checkpoint P7-D1 — Closure wall

- [ ] P7-09 through P7-14 pass.
- [ ] Raw P7-10 typed delegation reaches analyzer-owned semantics through its production-generated
      bundle-v0.2.
- [ ] Registered metadata-only and missing-child cases remain distinct.
- [ ] Malformed unrelated and malformed selected cases remain distinct.
- [ ] Direct closure did not expand transitively.

## P7-4 — Analyzer and public-live boundaries

### Task P7-4.1 — P7-15 incomplete bundle-v0.2

- [ ] Construct the smallest intentionally incomplete analyzer-facing bundle.
- **Acceptance:** `load_bundle` returns the selected exact `InputError`; the case is called
  bundle-v0.2, never a v2 rollout fixture.
- **Verification:** P7 target plus `input_contract`.
- **Dependencies:** P7-D1.
- **Estimated scope:** M.

### Task P7-4.2 — P7-16 invalid public-live event

- [ ] Provide one invalid public event shape at the checked live boundary.
- [ ] Snapshot permitted runtime state before and after rejection.
- **Acceptance:** exact rejection category is pinned and no forbidden state mutation occurs.
- **Verification:** P7 target plus `live_event_shape` and `live_checkpoint_compatibility`.
- **Dependencies:** P7-4.1.
- **Estimated scope:** M.

### Task P7-4.3 — P7-17 aligned Legacy/CurrentNativeV2 parity

- [ ] Express equivalent aligned legacy and `CurrentNativeV2` raw streams.
- **Acceptance:** canonical semantic projections match exactly and remain conservative.
- **Verification:** P7 target.
- **Dependencies:** P7-4.2.
- **Estimated scope:** M.

### Task P7-4.4 — P7-18 zero-test Legacy/CurrentNativeV2 parity

- [ ] Express equivalent zero-test evidence in both supported adapter classes.
- **Acceptance:** canonical progress/scoring projections match and neither claims clean verification.
- **Verification:** P7 target plus frozen progress wall.
- **Dependencies:** P7-4.3.
- **Estimated scope:** M.

### Checkpoint P7-D2 — Boundary and compatibility wall

- [ ] P7-15 through P7-18 pass.
- [ ] Exact analyzer/public-live failures are stage-owned.
- [ ] All three declared parity pairs compare canonical projections only.
- [ ] Relevant existing compactor/analyzer/Sentinel owner tests pass.

## P7-5 — Determinism

### Task P7-5.1 — P7-19 typed-output determinism stress

- [ ] Add multiple typed output segments whose type/order/turn/call identity must survive.
- [ ] Run the case under different temporary roots, reversed source creation/discovery order, and
      warm/cold closure state.
- **Acceptance:** canonical projections are byte-identical across runs; the raw unstable fields are
  explicitly excluded; typed sequence remains exact.
- **Verification:** run the P7 target twice in the same test invocation and once as a separate
  process.
- **Dependencies:** P7-D2.
- **Estimated scope:** M.

### Task P7-5.2 — Enforce whole-wall determinism, completeness, and bounded count

- [ ] Run all nineteen cases in two canonicalized wall executions.
- [ ] Vary temporary roots and stable source/discovery ordering for every applicable case.
- [ ] Exercise warm and cold closure state for every closure-traversing case.
- [ ] Compare exact canonical projections for every case and retain P7-19 as the stronger
      typed-segment order/identity stress witness.
- [ ] Assert all required contract families and planned cases are present.
- [ ] Assert case count remains within 15–25 and explain any count above 22.
- [ ] Assert every historical reference resolves to a tracked frozen artifact.
- **Acceptance:** every case has byte-identical canonical projections across its applicable varied
  runs; deleting, renaming, or adding an undeclared case fails deterministically.
- **Verification:** P7 target runs the complete varied wall twice in one invocation and once as a
  separate process, with internal inventory negative controls.
- **Dependencies:** P7-5.1.
- **Files likely touched:**
  - `crates/agent-drift-sentinel/tests/current_native_recall.rs`
  - `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/matrix.json`
  - `crates/agent-drift-sentinel/tests/fixtures/current_native_recall/README.md`
- **Estimated scope:** M — 3 files.

### Checkpoint P7-E1 — Committed corpus complete

- [ ] All nineteen cases pass.
- [ ] Matrix, privacy, provenance, whole-wall varied-run determinism, and bounded-count gates pass.
- [ ] Frozen historical walls remain unchanged.

## P7-6 — Private inventory and overlapping-quota sampler

### Task P7-6.1 — Freeze and digest the candidate inventory

- [ ] Filter candidates by the repository’s exact `CurrentNativeV2` route.
- [ ] Sort a privacy-sensitive local inventory deterministically at an explicit as-of cutoff.
- [ ] Compute inventory digest without emitting raw private content to committed outputs.
- [ ] Freeze the exact quota configuration, required named strata, and the minimum candidate
      population that makes each quota mandatory before selection begins.
- **Acceptance:** identical synthetic inventory produces identical digest regardless filesystem
  enumeration order; different inventory changes the digest; quota authority cannot change after
  selection begins.
- **Verification:** `python3 scripts/dev/drift-batch-scan/test_sample_sessions.py`.
- **Dependencies:** P7-B; may be researched before P7-E1 but must not mutate shared corpus files.
- **Files likely touched:**
  - `scripts/dev/drift-batch-scan/sample_sessions.py`
  - `scripts/dev/drift-batch-scan/test_sample_sessions.py`
  - `scripts/dev/drift-batch-scan/README.md`
- **Estimated scope:** M — 3 files.

### Task P7-6.2 — Implement deterministic overlapping-quota set cover

- [ ] Assign each candidate all observable language/repository, workflow, tooling, and delegation
      labels.
- [ ] Select greedily against 3-session ordinary quotas and up-to-5 high-risk/sparse quotas.
- [ ] Use seed only as a stable tie-breaker and emit selected-set digest plus underfill.
- [ ] Require a non-empty selected set.
- [ ] Fail underfill in any bucket whose eligible candidate population meets or exceeds its quota.
- [ ] Permit inventory-scarcity underfill only when population is below quota and all eligible
      candidates are selected or a documented privacy/safety exclusion explains otherwise.
- **Acceptance:** multi-label sessions receive all genuine credits; unknown never substitutes for a
  named bucket; every populated stratum family has a non-`unknown` selected representative unless
  the scarcity rule applies; identical inventory/config is reproducible; only permitted scarcity
  underfill is accepted.
- **Verification:** synthetic unit tests for overlap, tie-breaking, unknown, non-empty selection,
  sufficiently populated underfill failure, sparse inventory, and permitted scarcity limitations.
- **Dependencies:** P7-6.1.
- **Files likely touched:**
  - `scripts/dev/drift-batch-scan/sample_sessions.py`
  - `scripts/dev/drift-batch-scan/test_sample_sessions.py`
  - `scripts/dev/drift-batch-scan/README.md`
- **Estimated scope:** M — 3 files.

### Task P7-6.3 — Produce privacy-safe batch summaries

- [ ] Thread inventory/selection metadata through the local batch invocation.
- [ ] Extend tabulation only with observable, non-private coverage fields.
- [ ] Prevent scorer-internal claims from heuristic strata.
- **Acceptance:** committed tests can validate receipt shape without private sessions; generated local
  report contains digests, frozen quota configuration, required strata, eligible/selected counts,
  and permitted underfill but no raw paths, IDs, repos, or messages.
- **Verification:**
  - `python3 scripts/dev/drift-batch-scan/test_sample_sessions.py`
  - `python3 scripts/dev/drift-batch-scan/test_tabulate.py`
- **Dependencies:** P7-6.2.
- **Files likely touched:**
  - `scripts/dev/drift-batch-scan/run_batch.py`
  - `scripts/dev/drift-batch-scan/tabulate.py`
  - `scripts/dev/drift-batch-scan/test_tabulate.py`
  - `scripts/dev/drift-batch-scan/README.md`
- **Estimated scope:** M — 4 files.

### Checkpoint P7-E2 — Private lane ready

- [ ] Candidate inventory and selected set are digest-reproducible.
- [ ] Selected set is non-empty.
- [ ] Overlapping quotas, mandatory population floors, and permitted scarcity underfill are exact.
- [ ] Sufficiently populated underfill fails the synthetic harness.
- [ ] Synthetic sampler/report tests pass without private data.
- [ ] No private artifact is tracked.

## P7-7 — Validation, private receipt, review, and closeout

### Task P7-7.1 — Run focused and full release gates

- [ ] Run the P7 target and every focused owner.
- [ ] Run full compactor, analyzer, Sentinel, and workspace suites.
- [ ] Run formatting, clippy with `-D warnings`, Python tests, and diff checks.
- **Acceptance:** every command in Checkpoint P7-F of the plan exits zero; failures are classified
  before fixes.
- **Verification:** saved command/result ledger.
- **Dependencies:** P7-E1 and P7-E2.
- **Files likely touched:** none unless a fixture/harness defect is confirmed.
- **Estimated scope:** S.

### Task P7-7.2 — Run the private batch and record the sanitized receipt

- [ ] Freeze the live local inventory and execute the selected batch outside Git.
- [ ] Record digests, frozen quota configuration, required named strata, per-bucket eligible and
      selected counts, underfill, exact command versions, and pass/fail conclusions.
- [ ] Require a non-empty selected set and block underfill in every sufficiently populated bucket.
- [ ] Record inventory-scarcity underfill only when it satisfies the explicit selection limitation.
- [ ] Classify every concrete contract-relevant contradiction surfaced by the run as fixture/harness
      error, production contract defect, or new behavior request before accepting the receipt.
- **Acceptance:** receipt is sufficient for release signoff, proves the private lane was non-vacuous,
  contains no unresolved contract-relevant contradiction, and contains no raw private data.
- **Verification:** recursive privacy scan explicitly including the committed validation receipt,
  plus manual comparison with the local generated report and three-way triage ledger.
- **Dependencies:** P7-7.1.
- **Files likely touched:**
  - `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-validation.md`
- **Estimated scope:** S — 1 file.

### Task P7-7.3 — Run GitNexus change detection and bounded implementation review

- [ ] Run GitNexus detection over all uncommitted P7 changes.
- [ ] Verify affected symbols/flows stay within the approved fence.
- [ ] Send the exact P7 implementation delta, matrix, and sanitized receipt to a fresh independent
      reviewer.
- **Acceptance:** review is semantically bounded to supported P7 inputs and explicit invariants;
  findings are classified and no historical lexical audit is reopened.
- **Verification:** saved review prompt, response, hashes, and reconciliation.
- **Dependencies:** P7-7.2.
- **Files likely touched:** review guidance/receipt documents only.
- **Estimated scope:** S.

### Task P7-7.4 — Reconcile review findings

- [ ] Fix only confirmed fixture/harness defects within P7.
- [ ] Stop for a separately authorized production remediation if any production contract defect is
      confirmed.
- [ ] Rerun the smallest affected gates, then the full release wall.
- **Acceptance:** exact final delta is review clean; no unsupported behavior or threshold change was
  introduced.
- **Verification:** final bounded review response plus full gate ledger.
- **Dependencies:** P7-7.3.
- **Files likely touched:** only files named by confirmed in-boundary findings.
- **Estimated scope:** variable; each remediation must be separately bounded to S/M.

### Task P7-7.5 — Commit and establish P8 baseline

- [ ] Reconfirm hashes, path boundary, validation ledger, review result, and GitNexus change report.
- [ ] Stage exactly approved P7 files and create a scoped Conventional Commit.
- [ ] Verify the worktree is clean and record the commit as P8 baseline.
- **Acceptance:** one review-clean P7 commit contains no private data or unrelated changes; push occurs
  only on explicit user direction.
- **Verification:**
  - `git diff --cached --check`
  - `git status --short`
  - `git show --stat --oneline HEAD`
- **Dependencies:** P7-7.4.
- **Files likely touched:** no new content beyond closeout status.
- **Estimated scope:** S.

### Checkpoint P7-F — Complete

- [ ] All specification success criteria are proven.
- [ ] All focused/full/static/private gates are green.
- [ ] Privacy and whole-wall determinism are machine-proven.
- [ ] Private selection is non-empty; sufficiently populated quotas are filled; every surfaced
      contract-relevant contradiction is classified.
- [ ] Exact implementation delta is independently review clean.
- [ ] P7 is committed and the worktree is clean.
- [ ] P8 baseline is recorded; P8 itself has not started.
