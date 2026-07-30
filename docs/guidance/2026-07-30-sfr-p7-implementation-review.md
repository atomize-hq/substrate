# SFR P7 Fresh Independent Implementation Review

Date: 2026-07-30

Task/spec: `SFR-RB-100` / P7 current-native recall corpus

Review mode: `initial-range`

Review context: fresh independent ChatGPT Pro conversation

Conversation: https://chatgpt.com/c/6a6ad2cc-2504-83ea-9437-98947f375f97

Committed range: `133b55249f88492e16f80f98a63368911d733c7e..36db5c4b1912040970ce6d013d1fb9d2b7ea45ca`

GitNexus compare summary: 66 changed files, 505 changed symbols, 4 affected private-tooling
flows, `medium` aggregate risk, and no production-Rust change.

Bundle artifacts:

- `repo-review-bundle-index(16).md` — `e5fb49fd0e219825723941ce21a3149a857236398e13e0d8844fcf8055e13a66`
- `repo-review-diff-001(16).md` — `b4cc3cb6abbbec2e03c36620f4a5cc885d71ffc93cecf7c8131d0d6fa9b9ee8a`
- `repo-review-source-001(16).md` — `9e844c25d6b7fbb366f33953e6d4b6ac027dc5b7b29e11ba027cf2238b9d2177`
- Included source files: 85
- Intentionally excluded guidance paths: 3

Prompt SHA-256: `1a8915d81fdb2f2f6786b5faa6db2cbd0a2a5bb0614ed5a2e7f958fe4faf7ed7`

Response SHA-256: `692d602b3c8df000aeb708e25d404f01c1e68ac6121514cc981c6d29d69d014c`

Advisory only; verify against local project truth and authoritative docs; do not reduce scope
without user approval.

## Prompt

~~~~text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: 133b55249f88492e16f80f98a63368911d733c7e..36db5c4b1912040970ce6d013d1fb9d2b7ea45ca
Review target: the complete bounded P7 current-native recall-corpus implementation range. This is a test/fixture/script/documentation packet; production Rust is intentionally unchanged.

Supported inputs and behavior: the exact committed synthetic raw Codex rollout fixtures declared as Legacy or repository CurrentNativeV2; analyzer-facing bundle schema v0.2 fixtures; the exact valid/invalid public live-checkpoint fixtures; the P7 Rust harness; the deterministic Python private-inventory, overlapping-quota, batch, and aggregate-receipt tools; and the sanitized private validation receipt. Evaluate only the supported routes and behaviors explicitly declared by the P7 spec, matrix, and tests.

Supported platforms and dialects: Rust 1.89 and Python 3.11 on the validated macOS host; JSON/JSONL Codex rollout events, bundle v0.2 JSON/JSONL, and the committed public-live JSON shape. No shell-dialect audit, no Windows/Linux platform expansion, and no general Markdown or natural-language lexical audit is in scope.

In-scope invariants:
1. Exactly nineteen matrix-addressable cases cover the required P7 families with a strict matrix/fixture bijection and exact diagnostic ownership.
2. At least one CurrentNativeV2 positive traverses compactor -> bundle v0.2 -> analyzer -> public live output with the exact semantic-goal-drift result; required aligned, narrowing, zero-test, directive/path, collision, unrelated-session, delegation, malformed, incomplete-bundle, public-live, Legacy-parity, and typed-output outcomes are non-vacuously asserted.
3. Current-native identity, typed input/output ordering, and typed delegation linkage survive the production path where the matrix says they must.
4. Privacy and provenance gates fail closed on the committed corpus and the committed validation receipt.
5. Whole-wall canonical execution is byte-deterministic across cold/forward and warm/reverse variation, while deliberately unstable fields are excluded only where the spec permits.
6. The private selector freezes a deterministic CurrentNativeV2 inventory, applies immutable overlapping quotas with seed used only as a tie-breaker, rejects sufficiently populated underfill, permits scarcity only when every eligible candidate is selected, and emits only privacy-safe aggregate receipts.
7. The sanitized receipt accurately records a non-empty six-session private run, exact quota coverage, zero batch failures, limitations, tool identity, and three-way triage without raw private data.
8. P7 does not change production Rust, frozen R6/R7/R8 fixtures, scorer thresholds, or public schemas.

Required material consequence: a blocking finding must demonstrate, using a supported input or reachable P7 execution path, a false passing/clean result, a false authority or privacy claim, nondeterministic canonical evidence, a missing/vacuous required proof, incorrect routing or exact outcome, invalid quota/signoff behavior, or a contradiction that makes the committed validation receipt materially unreliable.

Blocking threshold: report only High or Medium findings that are reachable through supported behavior, violate an explicit in-scope invariant, have the required material consequence, and are caused by this committed P7 range. Low-severity improvements, style issues, theoretical hardening, conservative heuristic false negatives, and unsupported-input variants are non-blocking and should be listed only as deferred observations if essential.

Accepted prior findings: none; this is the first independent implementation review of this exact P7 range.

Deferred or out-of-scope concerns: reopening P0-P6 production parser behavior; enumerating adjacent lexical, Markdown, shell, malformed-command, platform, or dialect variants; production scorer retuning; expanding the private sample or quota taxonomy; new public schemas; P8 external-review work; unrelated cleanup or redesign. Unchanged supporting production code may be used only to establish reachability or prove that a P7 assertion is vacuous; it is not itself a review target.

Project sources: read repo-review-bundle-index(16).md first, then repo-review-diff-001(16).md, then repo-review-source-001(16).md. Their SHA-256 digests are respectively e5fb49fd0e219825723941ce21a3149a857236398e13e0d8844fcf8055e13a66, b4cc3cb6abbbec2e03c36620f4a5cc885d71ffc93cecf7c8131d0d6fa9b9ee8a, and 9e844c25d6b7fbb366f33953e6d4b6ac027dc5b7b29e11ba027cf2238b9d2177. Three docs/guidance paths are intentionally excluded and unavailable; do not infer their contents.

Validation evidence: the P7 target passes 26 tests; focused owner targets pass (7 current-native adapter, 10 bounded closure, 5 export bundle, 2 semantic-goal-drift, 4 progress, 2 delegated, 17 input-contract, 12 live-event-shape, 38 real-session-live); full compactor, analyzer, Sentinel, and workspace suites pass; cargo fmt --check, workspace clippy -D warnings, 12 sampler tests, 9 tabulator tests, and git diff --check pass. The private frozen inventory contained 706 eligible CurrentNativeV2 candidates, selected 6, filled every sufficiently populated quota, recorded only two zero-population scarcity buckets, and completed 6/6 compactor/analyzer runs.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions. Review only; do not generate a patch.

Return findings first by severity. For each qualifying finding give the exact file/function or fixture, supported-input reachability, violated invariant, material consequence, and why it is caused by this P7 range. Then give a concise gate-assessment and state explicitly whether there are no qualifying findings. Stop after the bounded determination; do not continue into adjacent audits.
~~~~

## Response

~~~~text
Findings
High — The private receipt chain can certify a different or stale run, and batch failures still exit successfully

Files/functions:
scripts/dev/drift-batch-scan/run_batch.py::{load_selection_receipt, main}
scripts/dev/drift-batch-scan/tabulate.py::{main, build_sanitized_receipt}

Supported-input reachability: The selection receipt carries a selected-set digest and quota coverage, but run_batch.py does not recompute that digest from selected_sessions.jsonl; it verifies only that the number of JSONL records equals selected_count. A different same-sized selection is therefore accepted under the original receipt’s inventory, quota, and selection digests.

repo-review-source-001(16)

Per-session compactor or analyzer errors are caught, recorded in status, and then followed by normal receipt generation. main() never returns a failure status when failed_count > 0.

repo-review-source-001(16)

The checkpoint directory is created with exist_ok=True and is not cleared. The documented workflow likewise reuses a named scratch directory via mkdir -p.

repo-review-source-001(16)



repo-review-source-001(16)

 tabulate.py then consumes every *.jsonl already present, grouping them by the embedded _session_file_id; it never reconciles those IDs, their count, or their provenance with the batch receipt.

repo-review-source-001(16)

 build_sanitized_receipt simply copies the selection and batch sections beside the independently calculated coverage values and applies only the privacy-key/path validator.

repo-review-source-001(16)

I reproduced both reachable failure modes using the committed scripts:

A one-session run whose synthetic compactor exited nonzero caused run_batch.py to exit 0 and write failed_count: 1.

A batch receipt declaring one attempted/succeeded session, paired with one current and one stale checkpoint file, caused tabulate.py to exit 0 and write sessions_analyzed: 2.

A failed rerun can be worse: because a failure occurs before the checkpoint output is reopened, an older file for the same session can survive and be counted by the tabulator.

Violated invariants: 6 and 7. The selector’s specification requires the selected-set digest and exact quota result to describe the selected candidates, and the receipt is supposed to establish the exact private run and zero failures.

repo-review-source-001(16)

Material consequence: A generated aggregate receipt can claim quota-selected set A while running same-sized set B, can include checkpoint results from a previous run, and can be emitted after a current batch failure. The committed validation receipt claims six attempted, six succeeded, zero failed, and says structural validation proved those facts, but the P7 tools do not establish the necessary linkage between selection receipt, executed sessions, batch status, and aggregated checkpoint files.

repo-review-source-001(16)



repo-review-source-001(16)

 This is invalid signoff behavior and makes the committed private receipt materially unreliable as proof of the claimed run, even though its recorded numbers may in fact have come from a clean manual execution.

Range causality: The defect is wholly in the P7-owned batch and receipt scripts introduced or modified by this range.

Medium — The matrix’s exact outcomes and diagnostic ownership are metadata-only, not enforced authority

Files/functions:
crates/agent-drift-sentinel/tests/current_native_recall.rs::{validate_case_metadata, p7_01…p7_19}
crates/agent-drift-sentinel/tests/fixtures/current_native_recall/matrix.json

The approved design states that matrix.json owns the canonical expected projection, exact result or error, terminal boundary, and per-assertion diagnostic owner; every vertical case is also supposed to terminate at its declared boundary with separately owned assertions.

repo-review-source-001(16)

 The specification’s illustrative harness directly compares each canonical result to case.expected and requires failure diagnostics to name case, boundary, and owner.

repo-review-source-001(16)

The implemented validator only establishes that case.expected is a non-empty JSON object. It does not validate its fields or compare it to execution output.

repo-review-source-001(16)

 All nineteen behavioral tests instead load a separate fixture-local expected.json; the matrix’s expected object is not used by those assertions.

repo-review-source-001(16)

There are also concrete ownership contradictions in the committed matrix:

P7-07 declares terminal boundary analyzer scoring and expects an exact active wrong_plan_branch score, but its only diagnostic owner is objective-path-extraction.

repo-review-source-001(16)

 The test actually asserts the exact scorer state, flag, and raw score.

repo-review-source-001(16)

P7-09 declares a public-live checkpoint terminal boundary and asserts public_trigger, but its declared owners are only direct-closure and scoring, omitting public-live conversion/runtime.

repo-review-source-001(16)

Supported-input reachability: Changing a committed matrix expected score, result, or diagnostic owner while leaving the separate expected.json untouched would leave the behavioral P7 tests green, provided the matrix value remained a non-empty object. The current P7-07 and P7-09 ownership mismatches already pass that gate.

Violated invariant: 1 — strict matrix/fixture authority and exact diagnostic ownership.

Material consequence: The wall can falsely pass with an incorrect authoritative matrix and can route a failure to the wrong component. This is a missing/vacuous required proof, not a formatting issue: release reviewers cannot treat the matrix as the claimed source of truth.

Range causality: Both the matrix and its harness validation are P7 additions in this range.

Medium — P7-03, P7-04, and P7-05 do not non-vacuously prove their declared conservative reason

Files/functions and fixtures:
p7_03_semantic_alignment_is_conservative / p7-03/raw/root.jsonl
p7_04_path_narrowing_preserves_progress / p7-04/raw/root.jsonl
p7_05_sanctioned_replan_is_suppressed_for_the_declared_reason / p7-05/raw/root.jsonl

For P7-03, the raw fixture has two separate turns with the same grounded objective, but the test inspects only the final checkpoint’s score, target, and empty evidence. It never asserts that the earlier eligible objective survived production ingest or that two objectives were compared.

repo-review-source-001(16)



repo-review-source-001(16)

 Dropping the first objective would still leave the asserted final target and NoClaim.

P7-04’s fixture changes from docs/specs/p7 to docs/specs/p7/narrowed.md, but the test likewise inspects only the final target, final progress, and score. Its only “narrowing” check compares a literal field from expected.json to the literal string "narrowing"; it does not assert the earlier broad target or the component relationship between the two targets.

repo-review-source-001(16)



repo-review-source-001(16)

 A current-native ingest regression that dropped the first goal would therefore pass this case as a single-goal NoClaim.

P7-05 is more directly non-causal. The fixture begins with the scoring-module goal and later contains a steer to two documentation paths.

repo-review-source-001(16)

 Nevertheless, the expected and asserted final structured target remains the original scoring-module path; the replan paths are asserted only as working-set residue.

repo-review-source-001(16)

 The test verifies that a Replan: row was typed as steer, then establishes the alleged suppression reason only by checking the literal fixture field expected["suppression"] == "sanctioned_replan".

repo-review-source-001(16)

That is non-causal because the production scorer returns the same NoClaim both when analysis.sanctioned_replan is true and when neither kickoff nor rolling divergence exists.

repo-review-source-001(16)

 The objective ranking heavily favors the original prompt and its /goal marker, while the later steer receives no role bonus, explaining why the replan never becomes the compared structured objective.

repo-review-source-001(16)

 Removing or breaking sanctioned-replan detection would therefore leave this P7 case green.

Violated invariant: 2 — aligned, narrowing, and sanctioned-replan outcomes must be asserted non-vacuously and for the declared reason. The task ledger explicitly requires P7-05 to be suppressed because of the replan rather than because the objective/evidence path failed to create a claim.

repo-review-source-001(16)

Material consequence: Regressions that lose the first aligned/narrowing objective or stop recognizing sanctioned replans can still produce a fully passing P7 target. The required conservative semantic proof is therefore missing.

Range causality: The fixtures and assertions are P7-owned additions; no production-parser reopening is required to establish the vacuity.

Medium — The varied-run whole wall stops before the declared public-live terminal for P7-01 and P7-09

File/functions:
current_native_recall.rs::{run_canonical_recall_wall, run_canonical_raw_case, canonical_successful_root}

The determinism specification requires all nineteen cases to run in at least two canonicalized wall executions under different roots, reversed applicable source ordering, and warm/cold closure state.

repo-review-source-001(16)

 The case design separately requires each vertical slice to end at its declared terminal boundary.

repo-review-source-001(16)

The whole-wall dispatcher chooses its terminal runner from adapter_classes: only a case explicitly carrying BundleV0_2 uses the bundle runner, and only one carrying PublicLive uses the public-live runner. Every other case uses the raw runner.

repo-review-source-001(16)

 A successful raw run canonicalizes the bundle and invokes analyze_bundle, then returns only bundle and analyzer projections; it never reaches LiveSessionCoordinator, LiveRuntime, or a public checkpoint event.

repo-review-source-001(16)

P7-01 and P7-09 both declare only CurrentNativeV2 as their adapter class while declaring terminal boundary public-live checkpoint.

repo-review-source-001(16)



repo-review-source-001(16)

 Consequently, their cold/forward and warm/reverse whole-wall executions compare only compactor/analyzer output. Their Sentinel event projection, public trigger, conversion, and runtime state are outside the byte-determinism comparison.

Violated invariant: 5 — whole-wall canonical execution must be byte-deterministic across the declared variations, with exclusions only for permitted unstable fields.

Material consequence: A deterministic compactor/analyzer result combined with order-, cache-, or run-sensitive public-live conversion would still pass the canonical whole wall. P7-01 has a one-off public-live behavioral test, and P7-09 has an isolation comparison, but neither supplies the required cold/warm and forward/reverse terminal-boundary comparison for both public-live cases. The committed claim that all nineteen complete vertical cases participate in the varied whole wall is therefore not proven.

Range causality: The route selection and canonical wall are P7 harness code introduced by this range.

Gate assessment

Blocked: one High and three Medium qualifying findings remain.

The supplied file digests matched the declaration, the range respects the production-Rust modification fence, and the extracted Python suites pass all 12 sampler and 9 tabulator tests. Those gates do not exercise the receipt-linkage failure, conservative-case causality, matrix authority, or raw-to-public-live determinism gaps above.

There are qualifying findings; this bounded P7 determination is not clean.
~~~~

## Local reconciliation

Status: **four qualifying P7-owned findings accepted for focused remediation**.

1. **High — receipt-chain integrity:** accepted. Selection digest binding, fresh checkpoint output,
   nonzero batch failure, and tabulator-to-batch reconciliation require executable proof.
2. **Medium — matrix authority:** accepted. Matrix expected projections and diagnostic ownership must
   be machine-bound to fixture expectations and executed assertions.
3. **Medium — conservative reason non-vacuity:** accepted. P7-03, P7-04, and P7-05 must prove the
   earlier objective/replan comparison and the exact suppression cause rather than merely observe a
   final `NoClaim`.
4. **Medium — public-live whole-wall boundary:** accepted. P7-01 and P7-09 must reach their declared
   public-live terminal in both varied canonical wall runs.

All four are fixture/harness/script defects within P7. No production contract defect is confirmed,
and no production source edit is authorized.
