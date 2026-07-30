# SFR P7 ChatGPT Pro consultation

- Date: 2026-07-29
- Task/spec: P7 / SFR-RB-100 sanitized current-native recall corpus planning review
- Repository: `/Users/spensermcconnell/.codex/worktrees/sfr-p4-authority-provenance/substrate`
- Reviewed baseline: `61da5bd5313e73898122c3e48d0599d27abd6b83..133b55249f88492e16f80f98a63368911d733c7e`
- ChatGPT conversation: https://chatgpt.com/c/6a6a70ea-cd2c-83ea-bac6-bd0e41aab113
- Setting: ChatGPT Pro, Chat surface, Intelligence Extra High
- Packet: sanitized repository-grounded review bundle containing the P0–P6 change surface plus current corpus, harness, batch-scan, and selected historical specification context
- Current approach: preserve existing historical regression corpora; add a small committed sanitized current-native corpus plus a larger private local real-session validation lane; gradually deepen current-native coverage without replacing the legacy floor prematurely.

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Prompt sent

~~~text
We are already working on this project task.

Task/spec: Plan P7 / SFR-RB-100, the sanitized structurally current-native semantic-goal-drift recall corpus and deterministic end-to-end validation, on exact clean post-P6 commit 133b55249f88492e16f80f98a63368911d733c7e. P7 must preserve historical R6/R7/R8 receipts and bounded legacy fixtures; exercise the restored P1-P6 current-native contracts end to end; avoid private data and scorer tuning; and stop for a separately bounded remediation if the corpus exposes a production defect.

Current approach: Repository-derived inference: “current-native” means the current Codex v2 raw rollout/event schema (session_meta multi_agent_version v2, turn_context turn IDs, response_item input, typed outputs, typed delegation/agent messages), not merely a new analyzer-row fixture. The repository already has a private regenerable seed-42 batch harness over roughly 110 real sessions, stratified by language/repo type, workflow, tooling, and delegation, plus committed bounded legacy/analyzer-level acceptance groups (including the 18-case semantic-goal-drift wall and 16-case progress wall). We currently propose two lanes:

1. Committed deterministic P7 lane: add a bounded sanitized v2 end-to-end corpus, likely about 15-25 cases rather than cloning the full 110-session store. Map each important v2 case to an existing semantic family or historical control where equivalence is intended. Cover genuine drift, alignment/narrowing, progression/replan, zero-test behavior, directive/path authority, delegation and metadata-only children, bounded closure malformed-inside/outside behavior, intentionally incomplete bundles, and public-live validation. Preserve v1 fixtures; add paired v1/v2 equivalence only where the semantics should truly match; allow v2-only cases for typed/current-only behavior.

2. Local real-session lane: retain real private sessions and derived outputs outside Git, but add or use quota-based v2-only/v2-stratified batch validation, targeting roughly 3-5 sessions per meaningful stratum without taking a Cartesian product. New regression coverage should default to v2 over time, while v1 remains the bounded compatibility floor.

Repository-derived inference: P7 should reuse existing compactor/analyzer/Sentinel helpers and normally touch only fixtures, tests/harness code, and a sanitization/provenance note. It should not add production behavior or a large new corpus framework unless a concrete missing contract requires a separate packet.

Project sources: Read repo-review-bundle-index(15).md first, then repo-review-diff-001(15).md, then repo-review-source-001(15).md. The bundle is exact committed range 61da5bd5313e73898122c3e48d0599d27abd6b83..133b55249f88492e16f80f98a63368911d733c7e plus narrowly selected current corpus harnesses, fixtures, source, and R6 corpus documentation. Excluded source is intentionally unavailable and must not be inferred. The exact P7 authority is also quoted below because the consultation/backlog source is not tracked at the reviewed head.

Exact P7 authority:
- Build the positive witness from actual current-native event variants, typed inputs/outputs, session/turn identity, delegation linkage, bundle registry, bounded linked-live closure, and public live-event boundary. Replace all real content, identifiers, repository names, messages, paths not required by the structural witness, and other private data. Do not substitute a legacy-shaped synthetic approximation.
- Include at least: one semantic-goal-drift true positive; controls for semantic alignment, unrelated-session evidence, zero-test execution, directive path syntax, lexical path-prefix collision, verified metadata-only child, intentionally incomplete analyzer-rejected bundle, malformed unrelated linked-live evidence, malformed selected linked-live evidence, public live-event shape rejection, and bounded legacy regressions. Define exact deterministic scorer, state, error, and NoClaim outcomes.
- Normally limit P7 to fixtures, tests, corpus harness changes, and a sanitization/provenance note. Do not tune thresholds or add fixture-specific production logic. If a new implementation defect appears, stop P7, create a narrow release-blocking remediation, and regenerate P7 from the repaired committed baseline.
- Run native witness and stratified corpus tests first, then complete compactor, analyzer, and Sentinel suites, cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, and git diff --check. Acceptance requires native positive recall, exact conservative negative/malformed outcomes, green legacy controls, deterministic results, and no private data.

Evidence, blockers, constraints, and validation: P0-P6 are committed and P6 is clean/review-clean. The existing 110-session batch is sampled from a growing private ~/.codex/sessions store and is explicitly not committed. Existing committed acceptance fixtures often begin at compacted rows or analyzer bundles, so they do not collectively prove raw v2 ingress through compactor export/closure, analyzer, and public-live boundaries. We need a repository-grounded decision on whether the proposed two-lane plan is the right size and shape, whether 15-25 committed cases and 3-5 local real sessions per meaningful stratum are justified, and how to avoid both undercoverage and unnecessary duplication. Privacy is strict: no raw private sessions, local absolute paths, real repository names, messages, or identifiers may be uploaded or committed. Historical R6/R7/R8 contracts must not be rewritten or reopened. P8 remains a separate external additive successor review receipt after P7 is applied, validated, reviewed, and committed.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; tradeoffs; what to keep/change first; inspections; validation; and pitfalls. Be repository-grounded and specifically answer:

1. Is the two-lane committed-v2 plus private-real-v2 strategy correct, and what should change?
2. Should P7 map v2 cases across every existing corpus group, only the P7 contract families, or another explicit matrix?
3. Is 15-25 committed v2 cases appropriate, too large, or too small? Is “3-5 per stratum” a sound local batch target, and how should overlapping strata be sampled?
4. Which existing harnesses/files should be extended versus kept frozen? Identify the minimum likely file/area set.
5. What exact v1/v2 equivalence, v2-only, malformed, privacy, determinism, and provenance assertions are missing from our plan?
6. What should be P7 acceptance criteria versus post-P8 maintenance/deferred expansion?
7. Identify any important repo-grounded concern, dependency, coverage hole, or stopping rule we have forgotten.

~~~

## ChatGPT Pro response

Recommendation: ADJUST

The two-lane direction is correct and does not need a pivot. The committed corpus should, however, become a dedicated P7 cross-layer proof wall, not a v2 copy of every historical acceptance group. The private lane should become a deterministic overlapping-quota sampler, not “3–5 independently selected sessions for every stratum.”

The most important architectural distinction to preserve is:

Current-native v2 describes the raw Codex rollout/event schema: session_meta, turn_context, event_msg, response_item, typed tool calls and outputs, and typed delegation activity.

repo-review-source-001(15)

The compactor-to-analyzer bundle remains schema v0.2. P7 should not introduce a “v2 analyzer fixture” concept or rename the exported bundle contract.

repo-review-source-001(15)

That distinction should be written into the P7 matrix and provenance note because it is the easiest place for the implementation to drift into a legacy-shaped approximation.

The supplied review material is a bounded projection of the exact committed range, and its sanitization changed private literals without changing current-native structure, public schema, outcomes, or P0–P6 logic. No raw private sessions or generated batch output are present.

repo-review-bundle-index



SANITIZATION

What to change first

Freeze the P7 coverage matrix before writing fixtures. Each case must identify the raw variants exercised, pipeline stages traversed, exact expected result, parity class, and any historical reference.

Choose one production end-to-end spine. The closest existing spine is the Sentinel real-session-live path: bounded closure → compactor export → analyzer → checked public live event, with replay equivalence. The existing test already proves selected-closure isolation and that live observations are the same checkpoints consumed by replay.

repo-review-source-001(15)

Define canonical determinism and privacy checks up front. Do not add them after the semantic fixtures are complete.

Make the local sampler explicitly v2-aware and overlap-aware. The existing seed is reproducible only relative to a fixed store; the private store grows, so seed 42 alone does not reproduce the same session set.

repo-review-source-001(15)

1. Two-lane strategy

Keep the two lanes, with these adjustments.

Committed lane

Make it the authoritative, CI-runnable P7 release gate:

raw current-native v2 JSONL
→ bounded linked closure
→ compactor normalization/export v0.2
→ analyzer bundle load and scoring
→ Sentinel checked public-live boundary
→ replay/live comparison where applicable.

This lane must use actual event variants. The current adapter already distinguishes typed custom_tool_call.input, ordinary function-call arguments, typed output segments, agent messages, sub-agent activity, and malformed or unsupported records.

repo-review-source-001(15)

 Its focused tests already pin segment order, type, turn identity, call identity, delegation activity, agent messages, and explicit malformed/unsupported handling.

repo-review-source-001(15)

 P7 should traverse those contracts, not duplicate their parser-level test inventories.

Private lane

Keep it outside Git as discovery and release-signoff evidence:

v2-only candidate inventory;

deterministic selection from a frozen local snapshot;

overlapping quota coverage;

no raw sessions or derived checkpoint contents committed;

production defect discovery triggers the P7 stop rule.

It should not be required for ordinary CI because another checkout cannot possess the private store. A P7 signoff can require that it was run locally, while the committed lane remains the independently reproducible proof.

Tradeoffs

This is slightly more work than merely converting existing compact-row fixtures to raw JSONL, but it gives materially stronger evidence:

raw schema recognition;

closure selection;

export registry correctness;

analyzer input rejection;

scorer behavior;

public-live validation;

cross-format compatibility.

It also avoids turning the existing 18-case R6 wall, 10-case R7 matrix, and progress wall into duplicate maintenance surfaces. The R6 wall is explicitly frozen at 18 allowlisted compact-row cases, while the R7 matrix is a separate synthetic v0.2 analyzer-level ownership wall.

repo-review-source-001(15)



repo-review-source-001(15)

2. Mapping strategy

Do not map v2 cases across every existing corpus group.

Use a P7 contract-family × pipeline-boundary matrix, with historical cases referenced rather than cloned.

P7 contract family	Required boundary proof
Native identity and typed shape	session_meta, thread/session identity, turn_context, typed call input, typed output ordering
Semantic positive	Raw v2 reaches a true semantic-goal-drift result
Semantic conservative controls	Alignment, narrowing/progression, sanctioned replan or equivalent NoClaim behavior
Verification execution	Zero-test output does not become clean proof
Path authority	Directive syntax does not launder authority
Lexical path identity	Prefix/hyphen/sibling collision does not suppress a real pivot
Session isolation	Unrelated session evidence cannot affect the selected result
Delegation	Typed linkage and metadata-only child registry behavior
Closure malformed handling	Unrelated malformed source excluded; selected malformed source rejected
Analyzer malformed handling	Intentionally incomplete bundle rejected with an exact InputError
Public-live boundary	Invalid public event rejected without mutating runtime state
Compatibility	A small number of declared v1/v2 semantic-equivalence pairs
Legacy floor	Existing R6/R7/R8 tests executed unchanged

Each matrix row should carry at least:

case_id

raw source format and event variants

logical source files and selected closure

semantic family

parity_mode: equivalent, v2_only, or not_applicable

historical case references

expected compactor projection

expected analyzer score/state/progress or exact error

expected public-live result or exact rejection

privacy and provenance assertions.

The historical references are especially important: they show intended equivalence without rewriting historical receipts. The R6 wall begins from compact rows and rebuilds an analyzer-valid bundle; it therefore cannot, by itself, prove raw v2 ingress.

repo-review-source-001(15)

3. Case counts and local sampling
Committed corpus

15–25 is the right envelope. Target 18–20; use 22 as a soft upper bound.

A reasonable initial budget is:

7 semantic/progress/path cases;

5 closure and delegation cases;

3 analyzer/public-boundary cases;

2 explicit v1/v2 parity pairs;

1 typed-output/determinism stress case.

That produces approximately 18 cases. Some cases may fill multiple matrix cells, but only when the combined witness remains easy to diagnose. For example, the positive semantic witness can also prove typed command input and output identity, but it should not simultaneously own every malformed condition.

Fewer than about 15 would likely force unrelated contracts into overloaded fixtures. More than about 22 probably means P7 is cloning focused P1–P6 tests rather than proving their composition.

Local “3–5 per stratum”

This is sound as a quota, not as a multiplication rule.

Use deterministic greedy set cover:

Filter to structurally recognized CurrentNativeV2.

Assign every session all applicable labels:

language/repository type;

workflow;

tooling;

delegation;

typed-output/delegation characteristics;

month or age band.

Select the session that satisfies the largest number of remaining quotas.

Break ties deterministically using the frozen inventory order and seed.

Credit a selected session to every stratum it genuinely covers.

Continue until quotas are satisfied or reported as underfilled.

Recommended quota policy:

3 distinct sessions for each sufficiently populated ordinary bucket;

5 only for high-risk or structurally sparse buckets such as delegated v2, typed multi-segment outputs, or whichever buckets the inventory shows are rare;

where available, at least two normalized repositories and two different time bands per bucket;

cap domination by one repository;

treat unknown as a diagnostic bucket, not proof that a named stratum was covered;

never fill a missing bucket with a “close enough” session.

Do not set a total unique-session target first. Let the overlapping quotas determine the minimum set and report both unique sessions and per-cell credits.

This matters because the existing language, workflow, tooling, and delegation classifications are explicitly best-effort reporting heuristics rather than analyzer facts, and several scorer decisions cannot be reconstructed from the current export.

repo-review-source-001(15)

For one release run, freeze locally:

an as-of cutoff;

the sorted v2 candidate inventory;

its local digest;

sampler configuration;

selected-set digest.

Those local identifiers and digests should remain outside Git unless separately privacy-reviewed.

4. Minimum likely file and harness set
New P7-owned area

The cleanest organization is a new Sentinel-level proof wall, for example:

crates/agent-drift-sentinel/tests/current_native_recall.rs

crates/agent-drift-sentinel/tests/fixtures/current_native_recall/matrix.json

crates/agent-drift-sentinel/tests/fixtures/current_native_recall/README.md

per-case raw .jsonl sources and expected.json

one new P7 sanitization/provenance note under the packet’s SFR documentation area.

Sentinel is the right owner for the cross-crate wall because the existing real_session_live test already exercises closure, compactor, analyzer, live delivery, and replay equivalence. The public-live tests separately own shape validation and expose rejection categories including missing or unexpected checkpoint payloads, invalid cursor identity, checkpoint/cursor mismatch, and invalid synthetic cursor sequencing.

repo-review-source-001(15)

Existing areas to extend narrowly

Likely script changes:

scripts/dev/drift-batch-scan/sample_sessions.py

scripts/dev/drift-batch-scan/run_batch.py

scripts/dev/drift-batch-scan/tabulate.py

scripts/dev/drift-batch-scan/test_tabulate.py

scripts/dev/drift-batch-scan/README.md

The current batch pipeline already isolates each session in its own temporary Codex home and runs compactor → analyzer.

repo-review-source-001(15)

 Extend it with:

explicit v2 eligibility;

frozen-inventory input;

multi-label quota selection;

underfill reporting;

selection determinism tests;

aggregate privacy-safe output.

Conditional focused-test changes, only when the new P7 harness cannot express the assertion cleanly:

agent-session-compactor/tests/current_native_adapter.rs

agent-session-compactor/tests/bounded_closure.rs

agent-session-compactor/tests/export_bundle.rs

agent-drift-analyzer/tests/input_contract.rs

agent-drift-sentinel/tests/live_event_shape.rs

agent-drift-sentinel/tests/real_session_live.rs

These are already the focused owners identified in the bundle.

repo-review-bundle-index

Keep frozen

Do not modify or reinterpret:

semantic_goal_drift_acceptance/**

progress_acceptance/**

delegated_acceptance/**

historical R6/R7/R8 receipts;

scorer thresholds or eligibility rules;

production parser, closure, analyzer, or Sentinel behavior.

The R8 receipt internals were not included in the bounded source selection, so P7 should not infer their contents. Their role is an unchanged legacy validation target.

5. Missing exact assertions
v1/v2 equivalence

Add only two or three pairs where semantic equivalence is intentional:

a true positive;

an aligned or legitimate-narrowing negative;

optionally zero-test behavior if it is useful and naturally equivalent.

Compare a canonical semantic projection, not raw files or whole manifest bytes:

logical selected sources;

session and turn partition;

ordered normalized row kind, role, text, row ordinal, and call linkage;

file-registry and delegation-link state;

structured objective and progress;

relevant score tuple;

public event cursor, warning fingerprint, and presentation.

Do not require byte equality. The manifest intentionally includes generated_at, codex_home, output_dir, source paths, and registry IDs, so raw comparison across temporary roots is the wrong determinism test.

repo-review-source-001(15)

v2-only assertions

Pin these without a legacy parity requirement:

session_meta.id/thread identity and optional session metadata;

turn_context.turn_id;

metadata-passthrough turn identity;

custom_tool_call.input versus function_call.arguments;

typed output segment order, type, index, and call ID;

typed agent-message author and recipient;

collab spawn begin/end call linkage;

sub-agent activity;

reciprocal child-origin metadata;

explicit preservation of unsupported native variants;

exact malformed-event parse failures.

The adapter already demonstrates these individual shapes; P7 must prove they survive composition.

repo-review-source-001(15)

Metadata-only child and incomplete bundle

Make these an intentional pair:

Accepted: the compactor registers a verified child source with no rows or turns, and the analyzer admits the verified link because the child exists in the registry.

Rejected: the same verified relation references a child absent from the bundle registry, producing exact InputError::VerifiedDelegationSessionMissing fields.

The accepted metadata-only behavior already exists at export level.

repo-review-source-001(15)

 The analyzer’s error taxonomy already distinguishes missing artifacts, malformed JSON/JSONL, schema problems, unknown registry references, insufficient contract, ordering failure, and missing verified sessions.

repo-review-source-001(15)

Malformed closure behavior

Pin all of the following:

malformed unrelated source is not decoded;

it is absent from selected sources, exported manifest, analyzer sessions, and live observations;

malformed selected source returns exact SelectedPayloadMalformed;

no published partial bundle or analyzer output remains;

the error names the selected logical source;

no unrelated malformed content appears in warnings or evidence.

The focused closure tests already establish the distinction between unrelated and selected malformed bodies.

repo-review-source-001(15)

Public-live rejection

Choose one strong rejection, preferably checkpoint/cursor mismatch, and assert:

exact surfaced error variant and identities;

no event-count increment;

no latest-cursor movement;

no warning or presentation emitted;

a subsequent valid event is still accepted;

valid replay and public-live interpretations remain identical.

Valid replay/public-live visible equivalence already has a focused witness.

repo-review-source-001(15)

Scorer and NoClaim outcomes

Do not assert merely “flagged” or “not flagged.”

For the positive witness, pin the final checkpoint tuple used by the existing wall: state=active, flagged=true, raw_score=80, plus confidence and required evidence prefixes.

repo-review-source-001(15)

For semantic NoClaim, pin:

class SemanticGoalDrift;

state Cleared;

raw score 0;

expected confidence;

flagged=false;

empty evidence.

repo-review-source-001(15)

Zero-test cases must additionally pin the progress dimension/status and explicitly forbid verification_clean, failure-frontier advancement, and other proof claims. The existing zero-verifier control follows that conservative pattern.

repo-review-source-001(15)

Privacy

A prose note is insufficient. Add a recursive test over every committed P7 file:

all session, turn, call, event, and agent IDs match a P7 synthetic grammar;

all absolute paths fall under an allowlist such as /repo, /fixtures, or C:/repo;

reject /Users/, /home/, ~/.codex/sessions, email addresses, credential-shaped values, and secret markers;

reject undeclared UUID-like or repository identifiers;

scan expected.json, matrix files, and documentation as well as raw JSONL;

ensure no source hash or value creates a reversible mapping to a private session;

ensure typed images/audio use synthetic payloads only.

Determinism

Run the committed corpus at least twice:

under different temporary roots;

with reversed source creation/discovery order;

with a warm and cold closure cache;

with the same fixed timestamps and synthetic IDs.

Compare canonicalized:

closure snapshots;

exported registry/rows/links;

analyzer checkpoint outputs;

live compatibility and presentation.

Also assert:

no duplicate or orphan matrix IDs;

every required authority family is covered;

every fixture is referenced exactly once;

no unapproved files appear in the corpus root;

expected outputs are stable across both runs.

Provenance

Each case should declare:

source_format: current_native_v2;

exact event/item variants retained;

whether it is structural sanitization, hand-authored from a structural template, or a parity projection;

transformations applied;

semantic family;

pipeline stages required;

historical reference case, where applicable;

parity mode;

why each retained field is structurally necessary;

a statement that no raw source content or identifier remains.

A test should verify that the declared variant inventory matches the actual JSONL, preventing provenance notes from becoming stale.

6. P7 acceptance versus post-P8 work
P7 acceptance

P7 is complete only when:

Every exact P7 authority family is represented in the matrix.

At least one raw current-native v2 positive traverses the complete production path and reaches the exact semantic-goal-drift result.

Alignment, narrowing/progression, zero-test, directive/path, and unrelated-session controls produce exact conservative outcomes.

Metadata-only registered child acceptance and missing-child rejection are both pinned.

Malformed unrelated and malformed selected closure behavior are exact.

The intentionally incomplete analyzer bundle returns the selected exact InputError.

Public-live shape rejection leaves runtime state unchanged.

Declared v1/v2 parity pairs match on their canonical semantic projections.

v2-only typed identities and event variants are preserved.

Two-run determinism and corpus completeness checks pass.

The recursive privacy scan passes.

Existing R6/R7/R8 walls remain unchanged and green.

The local v2 batch satisfies declared quotas, or P7 is blocked with an explicit underfill report.

The specified focused tests, complete three-crate suites, format, clippy, and git diff --check all pass.

Post-P8 maintenance or deferred expansion

Defer these until after P7 is applied, validated, reviewed, committed, and followed by P8:

additional raw event variants discovered later;

larger or rotating private-session samples;

raising quota floors;

broad language/tooling expansion;

performance or large-corpus benchmarks;

fuzzing/property-based malformed input;

widening checkpoint exports to replace heuristic strata;

changing scorer thresholds or eligibility;

removing the bounded v1 compatibility floor;

continuous private-corpus automation.

P8 should be an additive external review receipt. It should not silently update P7 expectations. A P8 finding that changes behavior should produce another bounded remediation.

7. Additional concerns and stopping rules
Important concerns

Schema-name collision. “v2” must mean raw Codex current-native input; “v0.2” remains the compactor/analyzer bundle.

Direct-closure boundary. Existing closure code is explicitly organized around direct linked closure selection. P7 should pin the currently authorized boundary rather than accidentally introducing transitive graph expansion or decoding deeper residue.

Stage ownership. Every failure must name its owning stage: native parser, closure selection, export, analyzer input, scoring, or public-live conversion. A generic “case failed” test would make production-defect triage too ambiguous.

Metadata-only is not missing. A child registered in the file registry with no rows is valid; a verified child absent from the registry is invalid. P7 must prevent those cases from collapsing into one.

Seed reproducibility is not inventory reproducibility. The private store’s growth means a seed is only a tie-breaker. A frozen local candidate inventory is necessary.

Heuristic local strata cannot prove scorer internals. The current batch cannot report structural-containment, scorer-true stable-target hygiene, or sanctioned-replan suppression without duplicating Rust logic or widening export. Do not claim those from batch tables; prove them in the committed wall.

repo-review-source-001(15)

Positive witness provenance. “Sanitized” must not mean rewriting a real case into compact rows. At least one positive must retain actual current-native envelopes and typed variants all the way to the first production parser.

Three-way stopping rule

Classify every failure before changing anything:

Fixture or harness error: malformed synthetic JSON, wrong expectation, stale matrix, or privacy violation with production behavior still conforming. Fix within P7.

Production contract defect: a structurally valid required witness violates a P1–P6 contract, an unrelated malformed source poisons the selected closure, a negative fires, a registered child is rejected, public-live accepts an invalid event, or canonical output is nondeterministic.
Freeze P7, open a narrowly bounded release-blocking remediation, repair and commit it, then regenerate P7 from the new clean baseline.

New behavior request: coverage reveals a desirable semantic expansion not required by P1–P7. Record and defer it; do not tune the scorer or add fixture-specific production logic.

Immediate production-remediation triggers should include:

needing a threshold or eligibility change to make a required fixture pass;

adding a case-specific parser/scorer branch;

any private-data dependency in a committed test;

nondeterministic canonical output;

exact accepted malformed/negative behavior differing from the restored P1–P6 contract;

partial outputs published after a selected malformed source;

replay and public-live interpretations diverging.

Bottom line

Keep the architecture; adjust the corpus design and sampling mechanics.

The best P7 is approximately 18–20 dedicated raw-v2 cross-layer cases, organized by a P7 matrix, with two or three explicit v1/v2 parity pairs, exact malformed and public-boundary outcomes, a machine-enforced privacy/provenance contract, and unchanged historical walls. The private lane should use 3–5 overlapping quotas per meaningful stratum through deterministic set cover, not independent per-cell sampling. Any production behavior change needed to satisfy that wall is a separate remediation, not part of P7.

## Local reconciliation

The recommendation is repo-grounded and should adjust, not replace, P7.

Verified local facts:

- Raw Codex rollout format is explicitly named `RolloutFormat::CurrentNativeV2` in `crates/agent-session-compactor/src/ingest/codex_rollout.rs`.
- The analyzer-facing compactor bundle remains schema `v0.2` (enforced in `crates/agent-drift-analyzer/src/input.rs` and emitted by `crates/agent-session-compactor/src/export/files.rs`). Therefore, raw fixtures may be called current-native v2, but analyzer bundle fixtures must not be called v2.
- `crates/agent-drift-sentinel/tests/real_session_live.rs` already protects the production cross-layer seam and is the correct place to assess a P7-owned end-to-end proof wall before inventing another pipeline.
- Existing semantic, progress, delegated, closure, adapter, batch-scan, replay, and public-live tests are substantial and should remain frozen regression walls rather than being copied wholesale into a parallel corpus.

Adopted adjustments for the P7 plan:

1. Freeze a P7 coverage matrix before selecting fixtures. Organize it by required P7 contract family and pipeline boundary; link historical cases as inherited proof instead of duplicating every old corpus grouping.
2. Keep two evidence lanes:
   - a committed, sanitized, structurally current-native CI corpus;
   - a private local batch over real sessions for recall and distribution evidence.
3. Target roughly 18–20 committed P7 cases (15–25 acceptable; 22 soft cap), but make matrix completeness—not a case count—the acceptance rule.
4. Include only 2–3 explicit v1/current-native-v2 equivalence pairs and compare canonical semantic projections, not raw bundle bytes.
5. Select private sessions with a deterministic overlapping set-cover sampler. Use 3 distinct sessions for ordinary populated strata and up to 5 only for high-risk or sparse strata; record honest underfill rather than substituting unrelated sessions.
6. Add machine-enforced provenance, recursive privacy, and determinism gates. Determinism must cover inventory digest and selection as well as seed, run outputs under varied temp roots/order/cache state, and canonicalized comparisons.
7. Keep production parser, closure, analyzer, Sentinel behavior, thresholds, eligibility, and existing acceptance corpora frozen. A fixture/harness error may be fixed within P7; a production contract defect stops P7 and becomes a separately authorized remediation; a new behavior request is deferred.
8. Preserve direct-link closure semantics. P7 must not accidentally turn the existing bounded direct-closure contract into transitive graph discovery.

Calibration against the roadmap:

- The committed CI corpus is the normative P7 acceptance wall.
- The private batch is release-signoff evidence and must never become an ordinary CI dependency or expose raw sessions.
- A private bucket underfill blocks P7 only when it prevents proof of a required P7 contract family. Otherwise it is recorded as an explicit coverage limitation rather than allowing private-store availability to redefine the roadmap.
- No production change or P7 implementation is authorized by this consultation alone.
