# SFR-RB-100 PLAN — Current-Native Recall Corpus

Status: **REVIEW-RECONCILED CANDIDATE — AWAITING USER APPROVAL**

Companion specification:
`docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-spec.md`.

Companion task ledger:
`docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-tasks.md`.

Baseline: post-P6 commit `133b55249f88492e16f80f98a63368911d733c7e`.

Implementation authority: **NONE**. This plan sequences P7 but authorizes no fixture, harness,
script, production, commit, or push work until the user approves the review-reconciled document
family.

## Overview

P7 adds a dedicated, sanitized, deterministic cross-layer recall wall for raw Codex streams that the
repository routes through `RolloutFormat::CurrentNativeV2`. The wall will prove composition through
direct bounded closure, bundle schema v0.2 export, analyzer semantics, and the Sentinel public-live
boundary. It will not clone every historical corpus group or modify production behavior.

The implementation has two evidence lanes:

1. **Committed lane** — nineteen self-contained, synthetic, CI-runnable cases selected against the
   P7 contract-family × pipeline-boundary matrix.
2. **Private lane** — a deterministic inventory and overlapping-quota sample of real local sessions,
   with a frozen quota contract, a required non-empty selected set, explicit scarcity rules, and
   only a sanitized digest/coverage/triage receipt committed.

Existing R6/R7/R8 fixtures and receipts remain unchanged regression walls. A valid witness that
contradicts production behavior stops P7 for a separate remediation.

## Verified Starting Truth

- The repository selects `RolloutFormat::CurrentNativeV2` only when
  `session_meta.payload.multi_agent_version == "v2"`.
- This does not establish a globally versioned Codex “rollout schema v2.”
- Compactor output and analyzer input remain bundle schema `v0.2`.
- The Sentinel crate already depends on both compactor and analyzer and its
  `real_session_live` integration exercises bounded closure, export, analyzer, live delivery, and
  replay equivalence.
- Existing frozen walls contain:
  - 18 semantic-goal-drift acceptance cases;
  - 16 progress acceptance cases; and
  - 10 delegated-acceptance matrix cases.
- The private batch tooling currently samples by month/repository diversity and does not yet freeze
  the candidate inventory, filter explicitly to `CurrentNativeV2`, or perform overlapping-quota set
  cover.
- Focused terminology verification is green:
  - `current_native_adapter`: 7 passed;
  - `export_bundle`: 5 passed.

## Architecture Decisions

### AD-1: Sentinel owns the cross-layer wall

Create a P7-owned integration target under `crates/agent-drift-sentinel/tests/`. Sentinel already
links compactor and analyzer and reaches the public-live seam, so this avoids a new crate or a
test-only orchestration framework.

### AD-2: Matrix is authority; count is a guardrail

`matrix.json` is the committed corpus manifest. It owns case identity, required contract family,
raw adapter class, event variants, direct closure, parity mode, historical references, canonical
expected projection, exact error/result, terminal boundary, per-assertion diagnostic owner, and
provenance.

The planned nineteen cases sit inside the approved 15–25 envelope and below the 22-case soft cap.
Any implementation-time change to this list must retain complete matrix coverage and receive user
approval if it crosses the agreed bounds.

### AD-3: Each case is a vertical diagnostic slice

Every case starts at the earliest boundary needed for its contract and ends at its declared terminal
boundary. Each assertion or first failure separately names the component that diagnostically owns
that result:

- end-to-end cases start as raw JSONL and reach Sentinel;
- closure cases terminate at exact closure selection/error;
- analyzer malformed cases start with a deliberately invalid bundle-v0.2 surface and terminate at
  exact `InputError`;
- public-live malformed cases terminate at exact rejection and state invariance.

Cases do not traverse later stages merely to appear more end-to-end. P7-10 is the required
composition exception: its raw typed delegation records must reach analyzer-owned delegation
semantics through the production-generated bundle-v0.2 rather than ending after export.

### AD-4: Canonical projections, never raw byte equality

Parity and determinism compare stable semantic projections. Temporary roots, generated timestamps,
absolute paths, registry-number allocation that is not semantically owned, and other documented
environmental values are excluded.

### AD-5: Privacy and provenance are executable contracts

The harness recursively scans the P7 fixture tree. It validates synthetic identifier grammars,
forbidden private markers, declared raw format/event variants, fixture-to-matrix completeness, and
historical-reference syntax.

### AD-6: Private selection is inventory-reproducible

The private sampler first freezes and digests a stable candidate inventory, then applies deterministic
greedy set cover across a pre-frozen exact quota configuration and required named strata. Seed is
only a deterministic tie-breaker. The selected set must be non-empty. A sufficiently populated
bucket must meet quota or fail as a sampler/harness defect; inventory-scarcity underfill is accepted
only under the specification's explicit limitation rule. Every concrete contract-relevant
contradiction is classified before the receipt is accepted. Raw paths, messages, repositories, and
session IDs never enter the committed receipt.

### AD-7: Existing walls remain frozen

P7 references but does not copy or edit the semantic, progress, delegated, R6, R7, or R8 fixture
families. Their existing commands are required regression gates.

### AD-8: Production contradictions stop the packet

No production Rust symbol is in the default file fence. Before any indexed Rust helper edit,
GitNexus impact analysis is mandatory. If a required valid case needs production behavior to pass,
implementation stops and a separate remediation is proposed.

## Dependency Graph

```text
reviewed SPEC / PLAN / TASKS
              |
              v
baseline + matrix schema + harness inventory
              |
              +----------------------+
              |                      |
              v                      v
privacy/provenance gates      private inventory/sampler tests
              |
              v
semantic positive + parity slices
              |
              v
semantic conservative/path/verification slices
              |
              v
delegation + direct-closure slices
              |
              v
analyzer malformed + public-live slices
              |
              v
determinism stress + matrix completeness
              |
              +----------------------+
              |
              v
focused gates -> three-crate gates -> workspace gates
              |
              v
private batch receipt + bounded implementation review
              |
              v
commit/closeout -> separate P8
```

The committed harness foundation precedes all cases because every slice updates the same matrix and
uses the same diagnostic runner. Case implementation is sequential to preserve fail-fast stopping
and avoid concurrent edits to shared authority.

## Planned Case Set

| ID | Case | Terminal boundary | Diagnostic owner | Parity |
|---|---|---|---|---|
| P7-01 | typed semantic-goal-drift positive | public-live checkpoint | scoring and public-live | current-native only |
| P7-02 | positive Legacy/CurrentNativeV2 semantic parity | canonical projection | parity comparison | equivalent |
| P7-03 | semantic alignment control | analyzer scoring | scoring | current-native only |
| P7-04 | path narrowing/progression control | analyzer scoring | progress and scoring | current-native only |
| P7-05 | sanctioned replan conservative control | analyzer scoring | scoring | current-native only |
| P7-06 | zero-test execution is not clean proof | analyzer scoring | progress and scoring | current-native only |
| P7-07 | directive path syntax cannot grant authority | analyzer scoring | objective/path extraction | current-native only |
| P7-08 | lexical prefix/hyphen/sibling collision preserves pivot | analyzer scoring | path identity and scoring | current-native only |
| P7-09 | unrelated-session evidence isolation | public-live checkpoint | direct closure and scoring | current-native only |
| P7-10 | typed delegation linkage survives composition | analyzer delegation semantics | ingest, closure, export, and delegation semantics | current-native only |
| P7-11 | registered metadata-only child is accepted | analyzer input | export and analyzer input | current-native only |
| P7-12 | missing verified child is rejected | analyzer input | analyzer input | current-native only |
| P7-13 | malformed unrelated source is excluded | closure selection | closure selection | current-native only |
| P7-14 | malformed selected source is rejected | closure verification | closure verification | current-native only |
| P7-15 | intentionally incomplete bundle-v0.2 is rejected | analyzer input | analyzer input | not applicable |
| P7-16 | invalid public-live event causes no forbidden mutation | public-live validation | public-live validation | not applicable |
| P7-17 | aligned Legacy/CurrentNativeV2 semantic parity | canonical projection | parity comparison | equivalent |
| P7-18 | zero-test Legacy/CurrentNativeV2 parity | canonical projection | parity comparison | equivalent |
| P7-19 | typed-output order and two-run determinism stress | canonical projection | ingest and canonical projection | current-native only |

If a case cannot own one clear failure stage, split it only within the approved count envelope. Do not
combine unrelated malformed cases to preserve the nominal count.

## Implementation Phases

### Phase 0 — Planning authority and baseline

1. Receive bounded review of the exact spec/plan/task files in the existing GPT Pro consultation.
2. Reconcile only findings inside the three-document boundary.
3. Obtain user approval of the reviewed family.
4. Reconfirm exact baseline, branch, worktree status, test names, and protected historical walls.
5. Run or refresh GitNexus and record that the default implementation fence is test/fixture/script/doc
   only.

**Checkpoint P7-A:** reviewed and approved planning family; exact clean baseline; no production edits.

### Phase 1 — Harness, matrix, privacy, and provenance foundation

1. Add `current_native_recall.rs`, fixture README, and `matrix.json`.
2. Implement matrix parsing, case-directory equality, required-family completeness, event-variant
   declarations, parity-mode validation, terminal-boundary validation, and diagnostic ownership.
3. Add recursive privacy and provenance validation before adding substantive fixtures.
4. Use an internal negative mutation or synthetic skeleton inventory to prove missing, duplicate,
   extra, and unknown cases fail closed. Do not leave the tracked P7 target intentionally failing
   while substantive case directories are added in later phases.

**Checkpoint P7-B:** harness/schema self-tests compile and pass; internal inventory mutations prove
bijection failures are closed; privacy/provenance tests fail closed; the tracked target is not
intentionally red; no production behavior is touched.

### Phase 2 — Semantic and authority vertical slices

Implement P7-01 through P7-08 one case at a time. Run the P7 target after every case and the frozen
semantic/progress walls after every two or three cases.

Order:

1. positive recall;
2. positive parity;
3. alignment;
4. narrowing/progression;
5. sanctioned replan;
6. zero-test;
7. directive authority; and
8. lexical collision.

The positive case is first to fail fast on the packet’s central recall claim.

**Checkpoint P7-C:** positive and conservative semantic families are exact; typed raw input survives
composition; frozen semantic and progress walls remain green.

### Phase 3 — Delegation, closure, malformed, and public-boundary slices

Implement P7-09 through P7-18 sequentially:

1. unrelated-session isolation;
2. typed delegation from raw current-native records through the production-generated bundle-v0.2
   into analyzer-owned parent/child topology, visibility, linkage, and applicable ownership
   semantics;
3. metadata-only child;
4. missing child;
5. malformed unrelated source;
6. malformed selected source;
7. incomplete bundle;
8. invalid public-live event;
9. aligned parity; and
10. zero-test parity.

Run the relevant compactor/analyzer/Sentinel focused owner after every slice.

**Checkpoint P7-D:** direct closure, raw-to-analyzer delegation composition, analyzer failure, and
public-live rejection outcomes are exact with declared terminal boundaries and diagnostic owners;
existing delegated and Sentinel walls remain green.

### Phase 4 — Determinism and private sampler

1. Implement P7-19 as the dedicated typed-output order/identity stress case.
2. Run all nineteen cases in two canonicalized wall executions. Vary temporary roots and stable
   source/discovery order for every applicable case, and warm/cold closure state for every
   closure-traversing case. Compare exact canonical projections.
3. Extend the private sampler to:
   - identify candidates routed as `CurrentNativeV2`;
   - freeze a sorted as-of inventory;
   - hash the inventory;
   - freeze an exact quota configuration, required named strata, and mandatory population floors;
   - assign overlapping labels;
   - apply deterministic greedy set cover; and
   - emit a non-empty selected-set digest, per-bucket eligible counts, quota results, and only
     explicitly permitted scarcity underfill.
4. Add synthetic unit tests for overlap, tie-breaking, non-empty selection, sufficiently populated
   underfill failure, and permitted inventory-scarcity limitations so CI never needs private
   sessions.
5. Extend batch reporting only as needed to produce a privacy-safe signoff and triage receipt.

**Checkpoint P7-E:** every committed case is deterministic; private selection is non-empty and
reproducible from its inventory/quota contract; sufficiently populated buckets meet quota; no raw
private content is committed.

### Phase 5 — Full validation, review, and closeout

1. Run the P7 target, all focused owners, all three crate suites, formatting, clippy, workspace tests,
   Python tests, and diff checks.
2. Run the private batch locally and write the sanitized receipt. Block on an empty selected set,
   underfill in a sufficiently populated bucket, a sampler/harness defect, or any concrete
   contract-relevant contradiction that has not been classified. Record inventory-scarcity
   underfill only when it satisfies the specification's explicit limitation rule.
3. Run GitNexus change detection before staging.
4. Send the exact bounded implementation diff plus matrix/receipt to a fresh independent reviewer.
5. Fix only review-confirmed P7 defects through a separately bounded remediation delta.
6. After review clean, stage only approved P7 paths, commit, verify clean, and establish that commit as
   P8’s baseline.

**Checkpoint P7-F:** all gates green, privacy clean, review clean, committed, pushed only upon explicit
user direction, and ready for separate P8.

## Verification Checkpoints

### P7-A — Planning lock

- Exact three-document review completed.
- Findings reconciled without broadening P7.
- User approves reviewed plan/task family.

### P7-B — Foundation

```bash
cargo test -p agent-drift-sentinel --test current_native_recall -- --nocapture
git diff --check
```

Expected: harness/schema checks pass; internal negative mutations prove incomplete, duplicate, extra,
or unknown inventory fails without leaving the tracked P7 target intentionally red.

### P7-C — Semantic wall

```bash
cargo test -p agent-drift-sentinel --test current_native_recall -- --nocapture
cargo test -p agent-drift-analyzer --test semantic_goal_drift_acceptance -- --nocapture
cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture
```

### P7-D — Closure and boundary wall

```bash
cargo test -p agent-session-compactor --test current_native_adapter -- --nocapture
cargo test -p agent-session-compactor --test bounded_closure -- --nocapture
cargo test -p agent-session-compactor --test export_bundle -- --nocapture
cargo test -p agent-drift-analyzer --test delegated_acceptance -- --nocapture
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
cargo test -p agent-drift-sentinel --test live_event_shape -- --nocapture
cargo test -p agent-drift-sentinel --test real_session_live -- --nocapture
```

### P7-E — Determinism and sampling

```bash
cargo test -p agent-drift-sentinel --test current_native_recall -- --nocapture
python3 scripts/dev/drift-batch-scan/test_sample_sessions.py
python3 scripts/dev/drift-batch-scan/test_tabulate.py
```

### P7-F — Release wall

```bash
cargo test -p agent-session-compactor -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
python3 scripts/dev/drift-batch-scan/test_sample_sessions.py
python3 scripts/dev/drift-batch-scan/test_tabulate.py
git diff --check
```

## Review Boundaries

### Planning review now

Review only:

- `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-spec.md`
- `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-plan.md`
- `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-tasks.md`

Review for contradictions, missing contract coverage, dependency-order errors, task sizing, ambiguous
acceptance, privacy gaps, and scope leakage. Do not review implementation or reopen P0–P6/R6–R8.

### Implementation review later

Use a fresh conversation and the exact P7 implementation delta. Report a blocking finding only when
supported input can violate an explicit P7 invariant, create false authority/false clean recovery,
break privacy/determinism, or invalidate a required exact outcome.

## Risks And Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| “v2” terminology conflates multi-agent marker with rollout or bundle schema | High | Use repository `CurrentNativeV2` and bundle-v0.2 names everywhere; matrix validates declared adapter class. |
| Nineteen cases duplicate focused P1–P6 tests | Medium | Require each case to own a cross-layer matrix cell or exact boundary composition not already proven. |
| One shared harness becomes monolithic | Medium | One case per task; shared runner stays data-driven; failures name terminal boundary and diagnostic owner. |
| Sanitization changes away the structural witness | High | Validate actual event variants and adapter route from fixture bytes; retain raw envelopes to first parser. |
| Private session store grows between runs | High | Freeze sorted candidate inventory and digest it before selection. |
| Private lane closes with an empty or vacuous sample | High | Freeze required quotas first; require a non-empty set; fail sufficiently populated underfill; permit only explicit inventory-scarcity limitations. |
| Set-cover labels overclaim scorer semantics | High | Limit private labels to observable heuristics; prove scorer internals only in committed wall. |
| Malformed unrelated evidence poisons selected closure | High | Separate exact P7-13 and P7-14 cases with different expected outcomes. |
| Metadata-only child collapses into missing-child behavior | High | Separate exact P7-11 and P7-12 cases. |
| Determinism test compares unstable bytes | Medium | Compare documented canonical projections, not raw manifests. |
| A fixture exposes a real production defect | High | Freeze P7 and open a separate remediation; never repair production inside a fixture task. |
| Review thread reopens broad historical semantics | Medium | Same consultation only for these three docs; explicit file/semantic fence and blocking rubric. |

## Parallelization

Implementation should be sequential because every case updates the shared matrix/harness and because
the production-defect stop rule must be evaluated before later fixtures are built. Read-only fixture
research or private-inventory analysis may occur in parallel only after matrix authority is frozen,
but no parallel agent may mutate shared P7 files.

## Exit Conditions

Planning exits only after:

1. the exact three files receive the requested bounded GPT Pro review;
2. all actionable in-boundary findings are reconciled;
3. the reviewed documents remain mutually consistent; and
4. the user explicitly approves implementation.

P7 implementation exits only through Checkpoint P7-F. P8 is not part of this plan.
