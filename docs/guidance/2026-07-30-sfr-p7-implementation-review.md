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

## Remediation Follow-Up Review 1

Review mode: `remediation-follow-up`

Review context: continuity-preserving follow-up in the same bounded conversation

Committed range: `36db5c4b1912040970ce6d013d1fb9d2b7ea45ca..d9978860d2f9f6b613d02340cf0c197c4a04d403`

Bundle artifacts:

- `repo-review-bundle-index(17).md` — `e79e190650cfc2fba04919e2547e48f3d454b899f267df414703c6d09cdf03b6`
- `repo-review-diff-001(17).md` — `286992378d5a8d48ef8448d3432bc232df38bf0974f8a7ed2bf2089ab6706c49`
- `repo-review-source-001(17).md` — `00f010b7b98eee47362a554a38a7ab680a753871d2b8af4774a7307d9052a1eb`
- Included source files: 16
- Intentionally excluded guidance paths: 1

Prompt SHA-256: `131ad8d91693b0e6c0555c84127d0ac75118370d367a60ca45214b666ff45e93`

Raw browser-response SHA-256: `07564431031d7c33a666c32367bd730a1e69f0a10b04b6676735c314261a59f3`

Result: **one High and one Medium qualifying finding accepted**. The High finding proved that
receipt-bound metadata still did not bind the source bytes at the selected path. The Medium finding
proved that P7-03, P7-04, and the sanctioned side of P7-05 still lacked an explicit scorer-eligibility
witness. Both remained within the P7 private-tool/test-harness fence.

Advisory only; verify against local project truth and authoritative docs; do not reduce scope
without user approval.

### Prompt

~~~~text
Review mode: remediation-follow-up
Review context: continuity-preserving follow-up in the same conversation that produced the initial P7 review
Review boundary: 36db5c4b1912040970ce6d013d1fb9d2b7ea45ca..d9978860d2f9f6b613d02340cf0c197c4a04d403
Review target: only the committed remediation delta for the four accepted P7 fixture/harness findings from your prior response, plus regressions caused by that delta. Do not reopen the original implementation range or search for new unrelated P7 issues.

Supported inputs and behavior: the exact committed synthetic Legacy and repository CurrentNativeV2 rollout fixtures changed by this delta; analyzer-facing bundle schema v0.2; the P7 Rust test harness; the deterministic Python private batch and privacy-safe receipt scripts; and the committed sanitized remediation receipt. Evaluate only these supported routes and the four accepted remediation objectives below.

Supported platforms and dialects: Rust 1.89 and Python 3.11 on the validated macOS host; JSON/JSONL Codex rollout events, bundle v0.2 JSON/JSONL, and the repository public-live checkpoint path. No shell-dialect audit, no Windows/Linux expansion, and no general Markdown, natural-language, malformed-command, or lexical-variant audit.

In-scope invariants:
1. The private receipt chain recomputes the selected-set digest, requires a fresh batch directory, writes checkpoint files atomically, returns nonzero after any failure, and allows aggregation only when the successful batch receipt exactly binds the recomputed checkpoint set and session/checkpoint counts.
2. Every matrix expected object exactly equals its fixture-local expected.json, mutation fails closed, and each declared terminal boundary has a compatible diagnostic owner. P7-07 includes scoring ownership and P7-09 includes public-live ownership.
3. P7-03 asserts a repeated eligible aligned trajectory; P7-04 asserts the earlier directory target and contained-file narrowing trajectory; P7-05 asserts a disjoint eligible pivot and proves causality by changing only the Replan: sanction marker, with the sanctioned fixture producing NoClaim and the otherwise identical counterfactual producing active semantic_goal_drift with raw score 80.
4. The varied whole-wall execution carries P7-01 and P7-09 through LiveSessionCoordinator and includes checkpoint-ready events, scheduler decisions, runtime snapshots, bundle output, and analyzer output in the byte-deterministic projection.
5. The remediation remains inside P7 test/fixture/script/documentation surfaces and changes no production Rust or scorer threshold.

Required material consequence: a blocking finding must show that one of the four accepted root causes remains reachable and unfixed, that the remediation delta can still produce a materially false clean/private signoff or vacuous required proof, or that the delta introduced a reachable material regression in a supported P7 path.

Blocking threshold: High or Medium only, with concrete supported-input reachability, a violated invariant or accepted remediation objective, the required material consequence, and clear causality to this remediation delta. Conservative false negatives, style, theoretical hardening, and unchanged pre-existing behavior are non-blocking.

Accepted prior findings:
1. High — the private receipt chain was not bound to the exact selected manifest/checkpoint set, failures exited successfully, and stale checkpoints could aggregate.
2. Medium — matrix expected projections and diagnostic ownership were metadata-only; P7-07 and P7-09 had owner contradictions.
3. Medium — P7-03, P7-04, and P7-05 did not non-vacuously prove alignment, narrowing, and sanctioned-replan causality.
4. Medium — whole-wall determinism stopped at analyzer output for P7-01 and P7-09 rather than their declared public-live boundary.

Deferred or out-of-scope concerns: all previously deferred concerns remain deferred, including reopening P0-P6 production parser behavior; additional lexical, Markdown, shell, malformed-command, platform, or dialect variants; production scorer retuning; expanding sample size or quota taxonomy; new schemas; P8; unrelated cleanup or redesign. Do not report a new issue merely because unchanged supporting production code is visible.

Project sources: read repo-review-bundle-index.md first, then repo-review-diff-001.md, then repo-review-source-001.md. Their SHA-256 digests are e79e190650cfc2fba04919e2547e48f3d454b899f267df414703c6d09cdf03b6, 286992378d5a8d48ef8448d3432bc232df38bf0974f8a7ed2bf2089ab6706c49, and 00f010b7b98eee47362a554a38a7ab680a753871d2b8af4774a7307d9052a1eb. One docs/guidance path is intentionally excluded and unavailable; do not infer its contents.

Validation evidence: the P7 target passes 26 tests; focused owner suites pass (7 current-native adapter, 10 bounded closure, 5 export bundle, 2 semantic-goal-drift, 4 progress, 2 delegated, 17 input-contract, 12 live-event-shape, 38 real-session-live); full compactor, analyzer, Sentinel, and workspace suites pass; cargo fmt --check and workspace clippy -D warnings pass; 12 sampler and 14 tabulator tests pass; py_compile and git diff --check pass. The strengthened private lane revalidated the frozen selected-set digest, ran a fresh six-file checkpoint set with 6/6 sessions and 13 checkpoints, recorded zero failures, and recomputed the exact checkpoint-set digest before aggregate receipt generation.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

For this remediation-follow-up, report a blocking finding only when it is reachable through supported behavior, violates an in-scope invariant or accepted remediation objective, meets the material-consequence threshold, and was introduced by the delta, is a regression caused by it, or proves that an accepted root cause remains unfixed. You may inspect unchanged supporting code only to establish causality. Unchanged pre-existing behavior is not an in-scope finding merely because you observed it.

After the four accepted findings are fixed and no qualifying delta regression remains, stop. Do not enumerate adjacent variants or additional audit ideas. Return findings first by severity with file/function, supported-input reachability, violated invariant, material consequence, and delta causality. State explicitly when there are no qualifying findings.
~~~~

### Response

~~~~text
Findings
High — The selected-set receipt still does not bind the rollout bytes that the batch runner executes

File/function: scripts/dev/drift-batch-scan/run_batch.py::{_CANDIDATE_DIGEST_FIELDS, selected_manifest_digest, validate_selected_manifest, main}; missed by scripts/dev/drift-batch-scan/test_tabulate.py::test_same_sized_selected_manifest_must_match_receipt_digest.

The remediation hashes the manifest’s claimed candidate metadata, including source_digest, but excludes the local path. Excluding an unstable private path from a portable digest is reasonable only if the runner verifies that the file found at that path actually has the declared size and source_digest. validate_selected_manifest does not open or hash the source file; it only recomputes a digest over the JSON fields. main subsequently trusts s["path"] and copies those bytes into the Codex home for execution.

repo-review-source-001(17)



repo-review-source-001(17)



repo-review-source-001(17)

Supported-input reachability: Start with a valid selected record for rollout A, including A’s source_digest, and change only the private path field to a different valid CurrentNativeV2 rollout B. All fields covered by selected_manifest_digest remain unchanged, so validation succeeds, but the compactor processes B. The downstream batch and checkpoint-set digests then correctly bind B’s outputs—while the embedded selection receipt still claims A.

I reproduced this with two same-length, different rollout files: the path-only substitution passed validate_selected_manifest; the committed batch script completed with one success and zero failures; the committed tabulator completed successfully and emitted an aggregate receipt. The newly added test does not cover this route: it mutates the recorded source_digest, rather than changing the executed path while leaving that claimed digest untouched.

repo-review-diff-001(17)

Violated invariant: In-scope invariant 1 and accepted prior finding 1.

Material consequence: The strengthened chain can still produce a materially false clean signoff in which quota selection, inventory provenance, and selected-set digest describe candidate A while compaction and analysis were actually performed on candidate B. The committed validation receipt specifically claims that the selected-set digest matched before execution and that the strengthened chain binds the exact private run; the current implementation does not establish that source-byte link.

repo-review-source-001(17)

Delta causality: The incomplete digest validation is remediation code introduced by this delta. The delta added digest recomputation but omitted verification of path against the already-recorded source_digest and size.

repo-review-diff-001(17)



repo-review-diff-001(17)

Medium — P7-03, P7-04, and the sanctioned side of P7-05 still do not prove scorer eligibility

File/functions:
crates/agent-drift-sentinel/tests/current_native_recall.rs::{canonical_semantic_projection, p7_03_semantic_alignment_is_conservative, p7_04_path_narrowing_preserves_progress, p7_05_sanctioned_replan_is_suppressed_for_the_declared_reason}

The strengthened trajectory projection carries only target_display, working-set paths, score state/flag/raw score, evidence reasons, and progress. It discards the structured-objective fields that distinguish an eligible scorer input from an ineligible one: objective_class, confidence, unknowns, and any stable-term/comparison-key eligibility result.

repo-review-source-001(17)

Using unchanged scorer code only to establish reachability, an eligible current goal requires a confident TaskStatement, no unknowns, a nonempty comparison key, and a stable distinguishing term. A target display by itself does not establish those conditions.

repo-review-source-001(17)



repo-review-source-001(17)

Supported-input reachability:

P7-03 now checks three repeated target strings, but not that those three checkpoints were eligible for semantic comparison. A regression on the exact fixture that preserves the structured target while introducing an unknown, lowering confidence, or losing the comparison key would still produce the expected target trajectory and NoClaim.

repo-review-source-001(17)

P7-04 verifies the directory and contained-file strings, but not that the earlier and later goals both entered the scorer’s eligibility bar. The case can therefore remain green without exercising the scorer’s narrowing/containment comparison.

repo-review-source-001(17)

P7-05 proves that removing Replan: yields an active score of 80, but it does not establish that the sanctioned fixture retains the same eligible structured goal. The scorer performs current-goal eligibility before checking analysis.sanctioned_replan; therefore, the sanctioned run could still return NoClaim at the earlier eligibility gate while the markerless counterfactual remains active.

repo-review-source-001(17)



repo-review-source-001(17)

Violated invariant: In-scope invariant 3 and accepted prior finding 3.

Material consequence: The P7 target can still pass without proving that aligned goals were compared, that directory-to-file narrowing reached the containment logic, or that P7-05’s NoClaim came from the sanctioned-replan branch. These are missing or potentially vacuous required proofs, rather than conservative behavior differences.

Delta causality: This delta added target-display trajectory assertions and the P7-05 counterfactual, but did not add an eligibility witness for the sanctioned or conservative trajectories. The accepted root cause therefore remains partially unfixed.

repo-review-diff-001(17)



repo-review-diff-001(17)

Gate assessment

The matrix-authority remediation is otherwise present: matrix expectations are compared exactly with fixture-local expected.json, terminal boundaries require compatible owners, and the committed P7-07 and P7-09 entries now include scoring and public-live, respectively.

repo-review-source-001(17)



repo-review-source-001(17)



repo-review-source-001(17)

The public-live determinism remediation also reaches the requested terminal path. The whole wall explicitly checks P7-01 and P7-09 for measured coordinator execution and checkpoint-ready observations, and its canonical projection contains bundle output, analyzer output, public observations, scheduler decisions, per-observation snapshots, and final runtime state.

repo-review-source-001(17)



repo-review-source-001(17)

Fresh-directory enforcement, atomic checkpoint publication, nonzero batch-failure exit, and downstream checkpoint/session/count reconciliation are implemented. Those protections do not repair the upstream source-byte binding gap.

repo-review-source-001(17)



repo-review-source-001(17)



repo-review-source-001(17)

The changed-path inventory remains within P7 test, fixture, script, and documentation surfaces; production Rust appears only as unchanged supporting context.

repo-review-bundle-index

Blocked: one High and one Medium qualifying finding remain. There are qualifying findings; this remediation delta is not clean.
~~~~

## Final Remediation Follow-Up Review

Review mode: `remediation-follow-up`

Review context: continuity-preserving final follow-up in the same bounded conversation

Committed range: `d9978860d2f9f6b613d02340cf0c197c4a04d403..729e0f61ccf1b42a780df615e965b86ecf69d2a2`

Bundle artifacts:

- `repo-review-bundle-index(18).md` — `758fb13062c02bbc79e4a534573ced443786b37a05864766345c0cfba25f6e29`
- `repo-review-diff-001(18).md` — `71d3eb5c4fe97933c21ae8829f5595c4916feae987a727f065c0893420da7581`
- `repo-review-source-001(18).md` — `88e9d1ce9624e188fbe74d36a1afe255a6045612863097b0ac785a0d50baccc4`
- Included source files: 6, exactly 3 changed and 3 narrowly supporting
- Intentionally excluded paths: 0

Prompt SHA-256: `86ff33375bc7af95309a4e059d9f30d02369e92ad2c8f309ab018c1878ce2824`

Raw browser-response SHA-256: `0a1c127e043e63457f9ba4cf48ba0d283ac079b84e9d5c76216bf2714b3ebcf1`

Result: **review clean**. GPT Pro found no qualifying High or Medium findings, confirmed both accepted
root causes fixed, confirmed the exact three-path delta changed no production Rust, and found no
reachable material regression in the supported private-batch or P7 fixture paths.

Advisory only; verify against local project truth and authoritative docs; do not reduce scope
without user approval.

### Prompt

~~~~text
Review mode: remediation-follow-up
Review context: continuity-preserving follow-up in the same bounded P7 review conversation; do not use earlier conversation breadth to widen this contract.
Review boundary: exact committed range d9978860d2f9f6b613d02340cf0c197c4a04d403..729e0f61ccf1b42a780df615e965b86ecf69d2a2. The base is the exact head you just reviewed; the head contains only the two commits repairing your one High and one Medium finding.
Review target: only whether the two accepted findings below are fully fixed and whether this three-path remediation delta introduced a reachable material regression.

Supported inputs and behavior:
1. `sample_sessions.py`-produced `p7-private-selection-v1` receipt plus its private `selected_sessions.jsonl`, whose records name a readable local CurrentNativeV2 rollout path and declare exact byte size and SHA-256 `source_digest`; `run_batch.py` validates and executes those records through the existing compactor/analyzer binaries.
2. The tracked P7 CurrentNativeV2 fixtures exercised by `current_native_recall.rs` through the existing compactor/analyzer pipeline. For this review, scorer eligibility means the unchanged scorer reached its eligible-current-goal path: a `task_statement`, objective confidence at least medium, no unknowns, a nonempty comparison key, and a stable distinguishing term. The unchanged scorer returns low-confidence NoClaim before that eligibility path; therefore medium/high semantic-score confidence is the behavioral witness that the gate was passed.

Supported platforms and dialects: Python 3.11 local file execution on macOS/Linux and Rust workspace tests on supported workspace platforms. No shell-language, Markdown-parser, Windows-cmd, PowerShell, or other dialect audit is in scope.

In-scope invariants:
1. Before compaction, the batch runner must prove that each selected path resolves to bytes matching the receipt-bound record's declared size and SHA-256 digest, and it must recheck the copied bytes actually placed in the isolated Codex home. A same-size path-only substitution must fail rather than produce a clean batch/aggregate receipt.
2. Every target-bearing P7-03 aligned checkpoint and P7-04 broad/narrow checkpoint must carry an explicit public structured-objective witness for task-statement class, medium-or-higher confidence, zero unknowns, and medium-or-higher semantic-score confidence. P7-05's sanctioned final checkpoint must carry the same eligibility witness as the marker-only counterfactual, so its cleared result cannot have occurred at the earlier ineligibility return.
3. The remediation remains confined to P7 test/private-validation surfaces; production Rust behavior is unchanged.

Required material consequence: a blocking finding must permit a materially false clean private-batch signoff, leave one of the named P7-03/P7-04/P7-05 required proofs vacuous, or introduce a reachable regression in these supported paths.

Blocking threshold: High or Medium only, with concrete supported-input reachability and the required material consequence. Low hardening, style, theoretical TOCTOU outside the rechecked copied bytes, conservative false negatives, and unsupported inputs are non-blocking.

Accepted prior findings:
1. High: the selected-set receipt did not bind the rollout bytes actually executed because a path-only substitution could retain the declared metadata while executing another file.
2. Medium: P7-03, P7-04, and the sanctioned side of P7-05 carried target strings but did not prove they passed scorer eligibility.

Deferred or out-of-scope concerns:
- Do not reopen the two earlier remediation findings you explicitly affirmed fixed: matrix/expected authority with compatible terminal owners, and public-live whole-wall execution/projection.
- Do not audit new parser classes, additional corpus scenarios, production objective extraction, public schema expansion, shell dialects, broader platform behavior, or general private-corpus representativeness.
- Do not require exporting private raw paths/content or changing production Rust.
- Unchanged pre-existing behavior is out of scope unless required solely to prove one of the two accepted root causes remains unfixed.

Project sources: read `repo-review-bundle-index.md` first, then `repo-review-diff-001.md`, then `repo-review-source-001.md`. The index declares exactly three changed paths and three narrowly necessary unchanged supporting files. Excluded source is unavailable and must not be inferred.

Validation evidence:
- `python3 -m unittest test_sample_sessions.py`: PASS, 12 tests.
- `python3 -m unittest test_tabulate.py`: PASS, 15 tests, including a same-sized path-only source substitution regression.
- Python compile: PASS.
- `cargo fmt --all -- --check`: PASS.
- `cargo check --workspace --all-targets`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace -- --nocapture`: PASS.
- P7 target: PASS, 26 tests.
- Fresh private rerun against the exact frozen six-record selection: selected-set receipt SHA-256 `98ad10a7ae164fff6d2fb622aa469ae74756bdc893de37e689f450faeb1e2801`; all six source paths matched declared bytes; 6 attempted, 6 succeeded, 0 failed; 6 checkpoint files and 13 checkpoints; checkpoint-set digest `2fa97f8cb47c04d040bb9b36543007eff533ce527e73ac3a0d4a1bec2f5aa98d`; batch receipt SHA-256 `47e0d6dd137c370cd95a6adaf1b18e5f2c58400fe646a5b50f771fe2efa8f4f2`; aggregate receipt SHA-256 `ddb5f11cb08ebe0c3c8ef930445726e3d281332dd4a001972a254aaf310a7fa9`.
- `git diff --check`: PASS.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

For this remediation-follow-up, report a blocking finding only when it is reachable through supported behavior, violates an in-scope invariant or accepted remediation objective, meets the material-consequence threshold, and was introduced by this delta, is a regression caused by it, or proves that one of the two accepted root causes remains unfixed. You may inspect unchanged supporting code only to establish causality or reachability; unchanged pre-existing behavior is not an in-scope finding merely because you observed it.

After the two accepted findings are fixed and no qualifying delta regression remains, stop. Do not enumerate adjacent variants, unsupported dialects, new parser classes, or broader platform behavior. Conservative false negatives are non-blocking unless introduced by this delta. Label other observations as deferred, not blocking.

The same causal mechanism means the same validation omission, provenance error, eligibility-proof gap, or equivalent trust transition remains operative. Superficial textual similarity is insufficient.

Return findings first by severity with file/function, supported-input reachability, violated invariant, material consequence, and delta causality. State explicitly when there are no qualifying findings.
~~~~

### Response

~~~~text
Findings

No qualifying High or Medium findings.

Gate assessment

Accepted finding 1 — exact source-byte binding: fixed. The batch runner now recomputes each selected path’s byte count and SHA-256 against the receipt-bound size and source_digest, after first confirming the selected-manifest digest. Every selected record passes this byte validation before the batch directory is created.

repo-review-source-001(18)



repo-review-source-001(18)

The bytes actually placed in the isolated Codex home are independently rehashed after shutil.copy and before the compactor starts. Thus, changing only the private path to a same-sized but byte-different rollout fails either the initial source check or the copied-byte check and cannot reach a clean compactor/analyzer execution.

repo-review-source-001(18)

 The focused regression constructs two same-sized, different files, retains the receipt-bound metadata, substitutes only the path, and requires validation to reject the mismatch.

repo-review-source-001(18)

 This closes the exact false-signoff mechanism from the prior High finding.

Accepted finding 2 — scorer-eligibility proof: fixed. The canonical checkpoint projection now exposes the public structured objective’s class, confidence, and unknown count together with the semantic score confidence. The shared assertion requires task_statement, objective confidence of at least medium, zero unknowns, and semantic-score confidence of at least medium.

repo-review-source-001(18)

P7-03 and P7-04 apply that assertion to every checkpoint carrying a structured target, while retaining their exact repeated-alignment and directory-to-contained-file trajectories.

repo-review-source-001(18)



repo-review-source-001(18)

 P7-05 applies the same assertion to both the sanctioned final checkpoint and the marker-only counterfactual, then requires the complete eligibility witness to be identical between them before asserting the counterfactual active score.

repo-review-source-001(18)



repo-review-source-001(18)

This is a causal eligibility witness under the declared unchanged scorer contract: failure to establish an eligible current goal returns a low-confidence NoClaim, while the public score preserves the confidence supplied by that branch. A medium/high semantic-score confidence therefore rules out the earlier ineligibility return for these fixtures.

repo-review-source-001(18)



repo-review-source-001(18)

Scope and regression gate: clean. The range changes exactly three paths: the P7 Rust harness, run_batch.py, and its focused Python tests. The production analyzer schema and scorer, plus the sampler, are included only as unchanged supporting sources.

repo-review-bundle-index(18)

 I found no reachable material regression in the supported private-batch or P7 fixture paths caused by this delta.

Both accepted findings are fully fixed, and there are no qualifying delta regressions.
~~~~

## Final Local Reconciliation

Status: **P7 REVIEW CLEAN**.

- Source-byte binding is independently confirmed: selected paths and the post-copy execution bytes
  must match receipt-bound size and SHA-256 metadata.
- P7-03 and P7-04 prove every target-bearing checkpoint reached semantic scorer eligibility; P7-05
  proves the sanctioned and marker-only counterfactual retain the same eligibility witness.
- The full workspace release wall and strengthened frozen six-session private lane passed after the
  final remediation.
- No production Rust changed, no production contract defect was surfaced, and no new behavior
  request was opened.
