# SFR-RB-100 / P7 Planning Review Dispatch

- Date: 2026-07-29
- Conversation: https://chatgpt.com/c/6a6a70ea-cd2c-83ea-bac6-bd0e41aab113
- ChatGPT setting: Extra High
- Mode: bounded specification, plan, and task-ledger review
- Implementation authority: none

## Reviewed artifacts

1. `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-spec.md`
   - SHA-256: `0cdcd5cd8638f3f661cd931f62bf73408df9d0a42a9da24051d9a45f51f16b13`
2. `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-plan.md`
   - SHA-256: `52d39f3794610797727d912827bf9bad465d3556e7aa87e7e532b36e5c83f835`
3. `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-tasks.md`
   - SHA-256: `d7a1015c12e9aeecd317bb70713c63d18f32f262835a306d7e3af37d3249ab60`

## Exact review prompt

```text
We are continuing the same SFR-RB-100 / P7 consultation. The user explicitly requested continuity in this conversation for a bounded review of the now-written planning family.

Review mode: bounded specification/plan/task-ledger review; review only, no code or patch generation.

Review boundary: exactly these three attached Markdown files, in this order:
1. SFR-RB-100-current-native-recall-corpus-spec.md
   SHA-256 0cdcd5cd8638f3f661cd931f62bf73408df9d0a42a9da24051d9a45f51f16b13
2. SFR-RB-100-current-native-recall-corpus-plan.md
   SHA-256 52d39f3794610797727d912827bf9bad465d3556e7aa87e7e532b36e5c83f835
3. SFR-RB-100-current-native-recall-corpus-tasks.md
   SHA-256 d7a1015c12e9aeecd317bb70713c63d18f32f262835a306d7e3af37d3249ab60

Task/spec: P7 builds a sanitized, deterministic cross-layer recall wall for raw Codex streams routed by this repository as RolloutFormat::CurrentNativeV2, plus a private deterministic real-session sampling lane. It preserves the compactor/analyzer bundle schema v0.2, the direct bounded-closure contract, existing R6/R7/R8 walls, privacy, and the production-defect stop rule.

Important verified terminology correction since your earlier consultation: multi_agent_version: "v2" selects the repository's CurrentNativeV2 adapter. This does not establish that the complete Codex rollout schema is globally named v2. Compactor export and analyzer input remain bundle schema v0.2. Evaluate the attached documents using that corrected terminology.

Current approach: The confirmed spec is implemented as a nineteen-case contract-family × pipeline-boundary matrix owned by a dedicated Sentinel integration target. Each case is a vertical diagnostic slice with an owning stage. The private lane freezes and hashes its inventory before deterministic overlapping-quota set-cover selection. Existing historical corpora remain frozen. Planning contains explicit checkpoints, machine privacy/provenance/determinism gates, and a three-way fixture-vs-production-vs-new-behavior stop rule. No implementation is authorized yet.

Continuity context: You may use the repository bundle and your earlier answer already present in this conversation only to understand the repository facts behind these three documents. They do not broaden the review boundary. Do not reopen P0-P6 implementation, historical R6/R7/R8 behavior, or conduct a general codebase/parser audit.

In-scope review questions:
- Do the three documents contradict one another or the exact P7 authority established in this conversation?
- Is any required P7 contract family, exact outcome, privacy/provenance/determinism rule, or stopping condition missing?
- Does dependency order permit false proof, premature implementation, or loss of fail-fast behavior?
- Are any tasks too broad, ambiguous, untestable, or outside the spec's file/behavior fence?
- Does the nineteen-case allocation duplicate focused tests unnecessarily or leave a composition boundary unproved?
- Does the private sampler/receipt design expose private data, overclaim heuristic strata, or fail reproducibility?

Blocking threshold: report High or Medium only when the attached planning family would materially permit a required P7 invariant to go unproved, authorize out-of-scope behavior, make the implementation non-reproducible/non-private, or make a task impossible to validate. Cite the exact attached file and section/task, the violated invariant, material consequence, and minimum document-only correction. Put Low/editorial suggestions in a separate non-blocking section. Do not enumerate adjacent event variants, new parser classes, unsupported platforms, or broader future-corpus ideas unless they prove a stated P7 requirement is missing.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return:
1. APPROVE or ADJUST.
2. Findings first, ordered by severity.
3. For every finding: exact file/section or task, violated invariant, material consequence, and minimum bounded document correction.
4. A short coverage-table assessment of the nineteen planned cases.
5. Explicitly state when there are no qualifying findings.

Review only. Do not rewrite the files and do not provide implementation code.
```

## Initial review result

The completed response matched all three supplied attachment hashes and returned `ADJUST` with
three Medium findings and no High findings.

### Exact captured answer

```text
ADJUST

All three attachments match the supplied SHA-256 hashes. I found three Medium findings and no High findings.

Findings
Medium — Raw typed delegation is not required to cross the analyzer boundary

Location:

Spec: Objective, Required Coverage Matrix, and Success Criteria

Plan: Planned Case Set, P7-10

Tasks: P7-3.2 — P7-10 typed delegation linkage

The specification requires current-native typed identities and delegation linkage to survive composition across compactor export, analyzer semantics, and the public-live boundary. It separately identifies typed linkage, metadata-only acceptance, and missing-child rejection as required delegation coverage.





P7-10, however, terminates at ingest/closure/export, and its task acceptance asserts only the direct delegation link and file-registry projection through ingest/export.





 P7-11 reaches analyzer input, but it is not required to originate from raw CurrentNativeV2 typed spawn/result and child-origin records. P7-12 is deliberately malformed analyzer input.

Violated invariant: At least one structurally current-native typed delegation witness must prove the composition seam from raw typed linkage through bundle-v0.2 into analyzer-owned delegation semantics. The historical R7 bundle matrix and focused adapter tests cannot substitute for that cross-layer composition proof.

Material consequence: An implementation could pass all nineteen planned cases while:

correctly parsing typed delegation;

correctly exporting a delegation link;

correctly analyzing separately constructed delegation bundles;

yet fail when the raw-derived exported link is actually loaded and interpreted by the analyzer. P7 would then miss the exact adapter/export/analyzer seam it exists to validate.

Minimum bounded document correction: Extend P7-10 so its production-generated bundle is loaded by the analyzer and exact parent/child topology, visibility, linkage evidence, and applicable ownership semantics are asserted. Reaching a checked public-live checkpoint may be included in P7-10 or explicitly shared with another case, but raw typed linkage must at minimum reach analyzer semantics without reconstructing the bundle independently.

Medium — Varied-run determinism is scoped only to P7-19, not the committed wall

Location:

Spec: Testing Strategy §5 — Determinism

Plan: Phase 4 — Determinism and private sampler

Tasks: P7-5.1 — P7-19 typed-output determinism stress

The specification says the committed wall runs at least twice under different temporary roots, reversed creation or discovery order, and warm/cold closure state where applicable; canonical projections must match.


The task ledger applies those varied configurations only to “the case,” P7-19. Its acceptance proves typed-output determinism for that one fixture, while P7-5.2 checks matrix inventory rather than rerunning the other eighteen semantic projections.


 The plan’s Phase 4 wording is less explicit and does not cure the narrower executable task.


Violated invariant: All committed case outcomes—not only the dedicated stress fixture—must be deterministic under the relevant environmental variations.

Material consequence: Semantic, closure, delegation, malformed, parity, or public-live cases could depend on discovery order, temporary paths, registry allocation, cache state, or unordered collections while P7-19 remains stable. The packet could therefore claim corpus determinism without testing most corpus outcomes for determinism.

Minimum bounded document correction: Make P7-5.1 or P7-5.2 rerun all nineteen matrix cases in two canonicalized wall executions. Apply different roots and stable ordering variation to every applicable case, and warm/cold closure variation to cases that traverse closure. Retain P7-19 as the stronger typed-segment stress witness rather than the sole varied-run determinism proof.

Medium — The private lane can close vacuously and lacks an explicit defect-triage gate

Location:

Spec: Assumptions To Confirm §5, Private Real-Session Lane, Failure Classification, and Success Criteria §13

Plan: Overview, AD-6, and Phase 5

Tasks: P7-7.2 — Run the private batch and record the sanitized receipt

The specification says private underfill blocks only when it prevents proof of a required P7 contract family.


 But the committed matrix is independently required to prove every P7 contract family. Consequently, the current condition permits every private stratum—including the entire private sample—to be underfilled without blocking closeout, because committed cases can still supply the contract proof.

Task P7-7.2 repeats that criterion and requires only a receipt containing digests, quotas, counts, underfill, and conclusions. It does not require a non-empty or minimally representative selected set, nor does it explicitly require concrete production contradictions surfaced by the private run to be classified under the packet’s three-way stop rule.


The documents otherwise describe the private lane as one of two actual evidence lanes, based on frozen inventory and deterministic overlapping-quota selection—not merely an optional empty receipt.


Violated invariants:

P7 must include a meaningful deterministic real-session sampling lane.

A production defect exposed by the P7 corpus must stop P7 for separate remediation.

Material consequence: P7 could close with:

zero selected real sessions;

wholly unrepresented meaningful strata;

an underfilled sample caused by a sampler defect rather than inventory scarcity; or

a private batch that surfaces a concrete P1–P6 contradiction that is merely summarized rather than classified and stopped.

That would make the private lane formally reproducible but substantively vacuous.

Minimum bounded document correction: Define private-lane acceptance separately from committed contract-family proof:

freeze an exact quota configuration and identify the required private strata before selection;

require a non-empty selected set;

treat underfill in a sufficiently populated bucket as a sampler/harness failure;

allow inventory-scarcity underfill only as an explicit limitation under a defined closeout rule; and

require every concrete contract-relevant contradiction surfaced by the run to be classified as fixture/harness error, production defect, or new behavior before the receipt is accepted.

This does not require broad semantic auditing of every ordinary outcome; it requires triage only where the batch presents evidence of a stated P7/P1–P6 contract violation.

Nineteen-case coverage assessment
Coverage area	Planned cases	Assessment
Native identity and typed values	P7-01, P7-10, P7-19	Good allocation. P7-19 adds useful segment-order stress. Raw typed delegation currently stops too early.
Semantic positive and parity	P7-01, P7-02	Complete. P7-02 is justified by cross-format projection parity.
Conservative semantic behavior	P7-03, P7-04, P7-05, P7-17	Complete across alignment, progression/narrowing, sanctioned replan, and negative parity.
Zero-test behavior	P7-06, P7-18	Complete and appropriately pins both current-native behavior and cross-format parity.
Path authority and identity	P7-07, P7-08	Complete; separates authority laundering from collision-based false suppression.
Session isolation and closure	P7-09, P7-13, P7-14	Strong allocation. Unrelated evidence and malformed selected/unselected behavior remain independently diagnosable.
Delegation and registry integrity	P7-10, P7-11, P7-12	Correct three-way division, subject to extending P7-10 through analyzer interpretation.
Analyzer/public malformed boundaries	P7-15, P7-16	Complete and appropriately terminate at their owning boundaries.
Determinism	P7-19 plus matrix gates	The stress case is useful, but it does not satisfy the spec’s whole-wall varied-run requirement by itself.
Historical compatibility floor	Existing gates, no cloned case	Correct. Historical walls are referenced and executed rather than rewritten.

The nineteen-case count is reasonable and remains within the approved size. It does not broadly clone focused P1–P6 tests. The three parity cases partly repeat the current-native semantic shapes of P7-01, P7-03, and P7-06, but each adds a legitimate cross-format assertion; their fixture data can reference or share the existing current-native half rather than copying it unnecessarily.

Non-blocking clarifications

The planned-case table uses composite owning-stage labels such as scoring/public-live, progress/scoring, and export/analyzer input, although the specification requires an owning failure stage. Consider distinguishing a case’s terminal boundary from the per-assertion or first-failing stage used in diagnostics.


P7-B is slightly ambiguous about whether all nineteen fixture directories already exist while substantive cases have not yet been implemented. Clarify that incomplete-inventory failure is exercised through an internal negative mutation or skeleton fixture, rather than leaving the real P7 target intentionally failing at the checkpoint.


The specification already requires scanning every committed P7 file. Task P7-7.2 should explicitly state that this includes the newly committed validation receipt, rather than relying on “recursive privacy scan” to imply it.
```

## Local reconciliation

All three Medium findings were confirmed against the local planning family and accepted:

1. **Typed delegation composition** — P7-10 now requires the raw typed delegation witness to pass
   through the production-generated bundle-v0.2 into analyzer-owned parent/child topology,
   visibility, linkage evidence, and applicable ownership semantics without reconstructing the
   bundle independently.
2. **Whole-wall determinism** — all nineteen cases now run in two canonicalized executions with
   applicable temporary-root, stable-ordering, and warm/cold closure variation. P7-19 remains the
   stronger typed-output stress witness rather than the sole determinism proof.
3. **Non-vacuous private lane** — exact quota authority is frozen before selection; the selected set
   must be non-empty; sufficiently populated underfill is a harness failure; only explicit
   inventory-scarcity limitations are permitted; and every concrete contract-relevant
   contradiction must receive three-way triage before receipt acceptance.

The three non-blocking clarifications were also incorporated: the plan now separates terminal
boundary from diagnostic owner, P7-B uses internal inventory mutations instead of leaving the
tracked target intentionally red, and the committed validation receipt is explicitly included in
the recursive privacy scan.

Post-reconciliation SHA-256 values:

1. `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-spec.md`
   - `d330828e477f3453165d47de667591c210cad72c09a946c6924c929eca64bf92`
2. `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-plan.md`
   - `cfe3b37410469d039583a0356a18e2c578bdd672915f2b0effd4c46a839549b8`
3. `docs/specs/sfr/SFR-RB-100-current-native-recall-corpus-tasks.md`
   - `ad63f4cf48e1b22b9c606350089c1a4b528ac913a780f122f3e903f5fda123f8`

Local consistency checks passed: exact P7-01 through P7-19 ordering in both plan and task ledger, 33
unique task IDs, all three corrective invariants present in every document, and whitespace/diff
checks clean.

## Remediation follow-up dispatch

The post-reconciliation three-file family was uploaded to the same conversation and sent for a
semantically bounded remediation-follow-up review.

### Exact follow-up prompt

```text
Remediation-follow-up planning review only. Continue the exact SFR-RB-100 / P7 three-document review already in this conversation.

Review boundary: only the document delta from the three previously reviewed attachments to the three newly attached current files. Do not reopen the original nineteen-case allocation, P0-P6 implementation, historical R6/R7/R8 behavior, repository code, parser variants, unsupported platforms, or broader corpus ideas.

Previously reviewed hashes:
- spec: 0cdcd5cd8638f3f661cd931f62bf73408df9d0a42a9da24051d9a45f51f16b13
- plan: 52d39f3794610797727d912827bf9bad465d3556e7aa87e7e532b36e5c83f835
- tasks: d7a1015c12e9aeecd317bb70713c63d18f32f262835a306d7e3af37d3249ab60

Current attached hashes:
- spec: d330828e477f3453165d47de667591c210cad72c09a946c6924c929eca64bf92
- plan: cfe3b37410469d039583a0356a18e2c578bdd672915f2b0effd4c46a839549b8
- tasks: ad63f4cf48e1b22b9c606350089c1a4b528ac913a780f122f3e903f5fda123f8

Accepted findings repaired:
1. P7-10 now requires raw repository-CurrentNativeV2 typed delegation to reach analyzer-owned parent/child topology, visibility, linkage evidence, and applicable ownership semantics through the production-generated bundle-v0.2 without reconstructing delegation metadata.
2. All nineteen committed cases now run in two canonicalized wall executions with applicable temporary-root, stable ordering, and warm/cold closure variation. P7-19 remains the stronger typed-segment stress case rather than the sole determinism proof.
3. The private lane now freezes its exact quota configuration and required named strata before selection, requires a non-empty set, fails underfill in sufficiently populated buckets, permits only explicit inventory-scarcity limitations, and requires three-way classification of every concrete contract-relevant contradiction before receipt acceptance.

The prior non-blocking clarifications were also incorporated: terminal boundary is distinct from diagnostic owner; P7-B uses internal negative inventory mutation rather than leaving the tracked target intentionally red; and the committed validation receipt is explicitly included in the recursive privacy scan.

Blocking threshold: report High or Medium only if one of the three accepted root causes remains unfixed in the current attachments, the correction creates a cross-document contradiction or impossible task, or the delta materially weakens privacy, determinism, fail-fast behavior, or the production-defect stop rule. Do not enumerate new adjacent concerns. Low/editorial observations must remain non-blocking.

Do not reduce the task or replace it with an easier alternative. Review only; do not rewrite files and do not provide implementation code.

Return:
1. APPROVE or ADJUST.
2. Findings first with exact file/section or task and delta causality.
3. For each accepted finding, state FIXED or NOT FIXED.
4. Explicitly state when there are no qualifying findings.
```

## Follow-up review result

The remediation follow-up returned **APPROVE**. It confirmed that the three current attachments
match the supplied post-reconciliation hashes, all three accepted findings are **FIXED**, and no
qualifying High or Medium findings remain. It also confirmed that the three prior non-blocking
clarifications were incorporated consistently.

### Exact captured follow-up answer

```text
APPROVE
Findings

No qualifying High or Medium findings. The reviewed delta repairs all three accepted root causes without creating a cross-document contradiction, impossible dependency, scope expansion, or weakening of privacy, determinism, fail-fast behavior, or the production-defect stop rule.

The current attachments match the three supplied current SHA-256 hashes.

Accepted-finding status
1. Raw typed delegation composition — FIXED

Delta causality: The revised specification now requires raw typed linkage to survive production bundle-v0.2 export and reach analyzer-owned topology, visibility, linkage evidence, and ownership semantics. It also distinguishes a case’s terminal boundary from the diagnostic owner of each assertion.


The plan explicitly makes P7-10 the composition exception: it cannot terminate after export and must reach analyzer delegation semantics through the production-generated bundle. Its case table now terminates P7-10 at analyzer delegation semantics.





Task P7-3.2 now requires:

loading the production-generated bundle-v0.2 through the analyzer;

no independent reconstruction of delegation metadata; and

exact parent/child topology, visibility, linkage evidence, and applicable ownership semantics.


The repaired requirement is also present in the specification’s closeout criteria.


2. Whole-wall varied-run determinism — FIXED

Delta causality: The specification now says all nineteen cases run in at least two canonicalized wall executions, with applicable temporary-root, stable ordering, and warm/cold closure variation. It explicitly states that P7-19 is not the sole determinism proof.


The plan sequences the full nineteen-case rerun separately from P7-19’s stronger typed-output stress role.


Task P7-5.2 makes this executable by requiring:

two complete canonicalized wall executions;

applicable root and ordering variation;

warm/cold state for every closure-traversing case; and

exact projection comparison for every case.


The checkpoint and final success criteria consistently require whole-wall determinism rather than P7-19-only stability.





3. Non-vacuous private lane and contradiction triage — FIXED

Delta causality: The specification now makes the private lane explicitly required and non-vacuous. Before selection it freezes the quota configuration, named strata, and mandatory population thresholds. It requires a non-empty result, fails sufficiently populated underfill, and permits scarcity only under exact documented conditions.


The plan carries the same contract into its architecture and closeout phases: quota authority is frozen first, the selected set must be non-empty, sufficiently populated underfill blocks, and unresolved contract-relevant contradictions prevent receipt acceptance.





The task ledger now provides testable work for:

immutable quota authority;

deterministic set cover;

non-empty selection;

populated-bucket underfill failure;

permitted scarcity handling; and

privacy-safe eligible/selected counts.


Task P7-7.2 also requires every concrete contract-relevant contradiction to receive one of the three classifications before receipt acceptance, and explicitly includes the committed validation receipt in the recursive privacy scan.


The production-defect stop rule remains intact and now expressly applies to contradictions surfaced by the private run.


Prior non-blocking clarifications

All three were incorporated consistently:

Terminal boundary versus diagnostic owner: now separate matrix fields and plan columns.


P7-B remains green: missing/duplicate/extra inventory is exercised through internal mutation or a synthetic skeleton rather than an intentionally red tracked target.


Validation-receipt privacy: explicitly required in the receipt task and specification.


No qualifying findings remain in the reviewed remediation delta.
```

Planning-review state: **REVIEW CLEAN** for the exact post-reconciliation hashes recorded above.

Implementation authority remains **NONE** pending explicit user approval of the reviewed planning
family.

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.
