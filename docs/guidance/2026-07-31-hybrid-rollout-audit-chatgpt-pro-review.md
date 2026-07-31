# ChatGPT Pro review: historical rollout compatibility audit

Date: 2026-07-31

## Task/spec

Independently adjudicate the sanitized historical Codex rollout compatibility audit findings F1-F5 and proposed remediation packets A-F, without widening into a general parser audit or requesting the raw private corpus.

## Current approach

Separate the work into bounded discovery, logical-source segmentation, wire-shape decoding, missing-turn authority policy, row-gated operator success, and a continuously maintained regression matrix.

## Review transport

- Conversation: https://chatgpt.com/c/6a6ca148-afc4-83ea-b2a1-0d0d0a4f518b
- Conversation title: Project Task Review
- Model selection as displayed: Pro intelligence mode. The menu listed `Instant 5.5`, `Medium`, `High`, `Extra High`, and `Pro`, with `Pro` checked; no more precise build label was exposed.
- Sanitized packet SHA-256: dc1c9abd642ce5290dc707e2d22c5a5f9ad89cbce2d1ed225b62146a47a5abeb
- Repository revision reviewed: a37be732d59595002b46e8ad1df64784dbb3a4bd

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Exact prompt

We are already working on this project task.

Task/spec: Independently adjudicate the sanitized historical Codex rollout compatibility audit findings F1-F5 and the proposed bounded remediation packets A-F for the Substrate rollout-compaction/drift pipeline. Use the attached packet as the exact review source. No code patch is requested.

Current approach: Keep the problems separated into root-first bounded discovery, a specified logical-epoch model for multi-identity physical files, wire-shape-aware legacy tool-output decoding, a security-preserving missing-turn user-message policy, row-gated smoke/batch success, and a bounded regression matrix. Do not assume the proposed ordering is correct; recommend KEEP, ADJUST, or PIVOT and give the safest dependency order.

Project sources: The attached sanitized Markdown packet contains the audit scope/integrity, aggregate structural findings, the 85-sample/19-kind execution matrix, narrowly selected Rust source excerpts, the six proposed remediation packets, explicit stop boundaries, and the requested answer format. The raw private corpus is intentionally unavailable and should not be requested.

Evidence, blockers, constraints, and validation: The audit covered 12,180 2026 rollout files and 3,754,763 records through two independent frozen structural buckets plus a fresh frozen execution matrix. It found global-discovery poisoning, common multi-identity/marker-transition files, legacy typed-list decoder failures independent of subagents, current-native user messages without turn_id, and 18 exit-zero semantic failures. Treat aggregate counts and code excerpts as direct evidence; treat architectural causality and remediation claims as inferences to adjudicate. Preserve fail-closed authority semantics. Do not enumerate new format or lexical variants, widen into a general parser redesign, or reduce the task to smoke-script containment.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; tradeoffs; what to keep/change first; inspections; validation; and pitfalls. Follow the attached packet's exact seven-part requested answer format. For every F1-F5 finding, return Confirmed, Overstated, or Insufficient evidence with calibrated severity and operational priority. Explicitly check the five possible conflations named in the packet and state which packets require a formal spec/plan/task-ledger process.

## ChatGPT Pro response

1. Direct recommendation: ADJUST

The six-way decomposition is fundamentally correct and should be preserved. Do not pivot to a general parser redesign, a format-variant inventory, or a smoke-script-only workaround. The packet is expressly bounded to architectural adjudication of F1–F5 and Packets A–F, with the aggregate evidence and excerpts serving as the review authority.

2026-07-31-rollout-compatibilit…

The current approach needs four adjustments:

Make Packet F a continuous validation spine, not the last remediation packet, and land Packet E at the beginning. Existing exit-zero behavior cannot be trusted to validate A–D.

Put a formal logical-source contract in front of Packet A and Packet B. Packet A’s notion of “relevant source” cannot silently retain the assumption that a physical file is one logical session.

Implement Packet C before Packet B’s implementation. C is an independently demonstrated, narrow decoder incompatibility. Fixing it first removes a confounder from the segmentation work.

Make Packet D’s default outcome explicitly turnless and non-authoritative. A stronger association rule is allowed only after a deterministic invariant is specified and tested.

At the packet level:

Packet	Recommendation
A	ADJUST — use relevance-gated strict indexing; global directory enumeration or a tolerant locator may still be necessary.
B	ADJUST — specification and proof gate first; only deterministic segment patterns become accepted.
C	KEEP — narrow field-shape compatibility, before B implementation.
D	KEEP — with untrusted/no-claim as the mandatory fallback.
E	KEEP — wrapper-local and first in the implementation sequence.
F	ADJUST — establish before changes, expand after each packet, and close the program.

The tradeoff is more contract work before the broadest compatibility recovery. That delays Packet B, but it prevents three materially worse outcomes: inventing epoch boundaries, manufacturing user-message authority, and accepting false-green remediation results.

2. Finding table
Finding	Verdict	Severity and operational priority	Evidence quality	Reason
F1 — Global discovery poisoning	Confirmed	High availability/operational severity. P0 behavioral fix, immediately after the validation gate.	High. Direct control-flow evidence plus a real-home reproduction.	prepare discovers and strictly indexes all sources before root selection, and an index_source failure propagates before the requested root is chosen. The reproduced unrelated pre-envelope source establishes reachability of the failure, not merely a theoretical concern. The remediation inference is also sound: strict validation should apply to the root and sources capable of entering its closure, not to demonstrably unreachable inventory.

2026-07-31-rollout-compatibilit…


F2 — File-wide identity/format rejection	Confirmed, bounded to the rejection defect and need for a segment model	High compatibility exposure. P1 specification; implementation after A and C.	High for the current rejection mechanism and population prevalence; moderate for the exact logical-epoch interpretation.	The code imposes one identity and one format across an entire physical file. Meanwhile, 1,610 files have multiple identities, 1,543 end with a different identity, and all unmarked→v1/v2 transition files are multi-identity. All 39 sampled multi-identity files failed. This is enough to reject the physical-file-equals-logical-source assumption and to warrant Packet B. It is not enough to declare every session_meta record a boundary or every affected file recoverable.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…


F3 — String-only legacy output decoder	Confirmed	High compatibility severity. P1, before B implementation.	High. Direct schema mismatch, aggregate wire-shape evidence, and isolated execution failures agree.	Current-native parsing accepts a string or typed array, while the legacy dependency fixes output as Option<String>. Typed output appears in recent unmarked files without collaboration or child-lineage evidence, disproving marker/subagent status as a reliable wire-shape proxy. The 219 additional single-identity legacy files establish that this defect exists independently of F2.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…


F4 — Missing-turn current-native user messages	Confirmed	Medium compatibility severity; High authority risk if remediated incorrectly. P2.	High for the deterministic parse failure; insufficient for any proposed implicit turn association.	All 78 observed records lacked a string turn_id, affected 63 consistent-v2 files, and the adapter requires the field. That confirms the compatibility problem. Nothing in the packet proves that previous-turn, next-turn, or ambient segment state is authoritative. The only generally justified fallback is retained turnless evidence with no authority claim and an explicit diagnostic.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…


F5 — Exit zero is not semantic success	Confirmed	High operator-safety severity. P0 validation fix.	Very high. The two semantic classifiers agreed in all 85 samples; only process status disagreed.	There were 45 exit-zero runs but only 27 semantic-clean runs, leaving 18 false successes. Packet E is therefore necessary before later packets can be credibly validated. The evidence supports a developer smoke/batch gate; it does not establish that a public CLI contract should change.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…

F2-specific adjudication

The logical-epoch hypothesis is strong enough to warrant formal specification and bounded implementation work, rather than preserving file-wide rejection indefinitely. It is not strong enough to authorize an unconditional rule such as “each session_meta starts a new epoch.”

Packet B should therefore use a proof gate:

Transitions satisfying the specified boundary contract may be segmented.

Conflicts within a segment remain fatal.

Files with ambiguous or unsupported transition structure remain file-wide rejected until their structure is covered by an authorized contract.

“Logical source segment” is the safer implementation term; “epoch” should not imply more runtime semantics than the evidence establishes.

3. Conflation check
Named conflation	Adjudication
Physical file vs logical segment	Present and material. Current indexing and ingestion both assign one identity and one format to the entire file, while ordered identity/marker transitions are common. Packet B correctly separates the concepts, but the audit does not itself supply the segment-boundary algorithm. Packet A must also avoid creating a new API that permanently equates a discovered file with a closure node.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…


Collaboration/version marker vs tool-output wire shape	Directly disproven. Structured output occurs in unmarked sources without serialized collaboration or child-lineage evidence. Route selection may continue to use the supported format marker, but the output field must be decoded from its actual supported JSON shape. Packet C must not reroute the whole legacy record through the current-native adapter.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…


Fail-closed security vs compatibility	Must remain separated. Current rejection is safe but over-broad in F1 and likely over-broad at physical-file granularity in F2. The fixes must narrow the unit to which strict validation applies, not weaken strictness: reachable sources still fail closed; conflicts within a defined segment still fail; unsupported tool-output shapes still fail; missing-turn user messages gain no turn authority.
Isolated-source execution vs real-home discovery	Distinct evidence domains. The 85-sample execution matrix deliberately used one-source homes and therefore cannot validate or refute F1. F1 is supported by its separate real-home control. Both test forms must remain: isolated homes for decoder/segment causality and a mixed real home for discovery poisoning.

2026-07-31-rollout-compatibilit…


Process exit code vs semantic success	Directly confirmed. Exit zero cannot stand in for successful selected-source compaction, analyzer completion, and Sentinel replay. Packet E should reuse the established semantic-clean/completion contract rather than creating a third classifier based on log strings or shell heuristics.

2026-07-31-rollout-compatibilit…

Packet A’s fail-closed condition

Packet A preserves fail-closed semantics only if closure membership is root-generated under the existing authority contract:

Start with the requested root.

Expand through authoritative claims made by already selected sources.

Resolve candidates exactly.

Treat duplicate or ambiguous candidates as failure.

Require reciprocal lineage wherever the existing contract requires it.

Strictly index and decode every source or logical segment that becomes reachable.

An apparently unrelated source can still become relevant if the current closure contract supports any of the following: inbound-only child self-nomination, alias-based candidate resolution, duplicate identities, or independently addressable later segments in a shared physical file. The packet does not include enough closure code to rule those mechanisms in or out.

Accordingly, Packet A should be framed as relevance-gated strict indexing, not necessarily “never enumerate the rest of the home.” A non-fatal filename/metadata catalog may be required to locate exact candidates. The defect is global fatal body indexing before relevance, not directory enumeration by itself.

4. Revised packet order

The safest dependency structure is:

F₀ → E → B₀ contract → A → C → B implementation → D → F final

Packet C is independent enough to proceed in parallel with the B₀/A work after E, but it should land before B implementation. The packet definitions and stop boundaries are directionally sound.

2026-07-31-rollout-compatibilit…

1. F₀ — Bootstrap the validation spine

Before changing behavior, freeze a minimal sanitized baseline containing:

At least one existing semantic-clean control.

One exact reproducer for each of F2, F3, F4, and F5.

The separate mixed-home F1 reproducer.

Record the current expected failure category for each. This is not the full 19-kind corpus yet; it is the minimum needed to prove that each later packet changes only its intended predicate.

Stop boundary: tests and receipts only; no parser or closure behavior changes.

2. E — Row-gated smoke/batch success

Make developer smoke/batch success require the structured semantic completion condition:

No selected-run error rows.

Expected fresh artifacts exist.

Required analyzer checkpoints are valid.

Sentinel replay completed.

No timeout or incomplete state occurred.

The underlying subprocess also exited successfully.

Use the existing semantic-clean interpretation that agreed with strict isolated-source results, rather than duplicating it as shell text matching.

Stop boundary: wrappers and wrapper tests. No production/public CLI exit change.

3. B₀ — Formal logical-source and closure contract, without implementation

Before A or B introduces durable interfaces, specify:

Physical artifact versus logical source segment.

Stable record/byte range and physical provenance.

Root addressability: filename identity, first identity, and whether later segments can ever be independently selected.

Exact segment-start and segment-end predicates.

How identity and format are assigned to a segment.

Which closure edge directions are authoritative.

Candidate ambiguity and reciprocal-lineage behavior.

How claims at segment boundaries are attributed.

This contract may conclude that some observed transition patterns remain unsupported and rejected.

Stop boundary: no temporal-proximity linkage and no “every session_meta is a segment” default.

4. A — Root-first, relevance-gated discovery

Implement selected-root lookup and graph expansion under the B₀ definitions.

A tolerant catalog may enumerate potential source locations, but a malformed body outside the proven closure must not abort the selected request. Once a candidate is selected or can affect exact candidate resolution, it is validated strictly.

Stop boundary: no general discovery rewrite, no skipping failures in reachable sources, and no relaxed lineage or ambiguity checks.

5. C — Wire-shape-aware legacy tool output

Decode only the known supported scalar and typed-list shapes at the legacy adapter boundary, reusing current-native content semantics only where they are actually equivalent.

C should precede B implementation because:

It repairs the 219 additional single-identity cases independently.

It prevents typed-output failures from obscuring whether a B fixture was segmented correctly.

Partial compatibility is not misleading once E and F report exact per-kind outcomes.

Stop boundary: do not reroute legacy sources wholesale, add generic JSON permissiveness, or fork the dependency broadly unless the narrow adapter is demonstrated impossible.

6. B — Logical segment implementation

Implement only the segment patterns authorized by B₀:

Index physical files into stable logical ranges.

Assign one identity and one format per segment.

Route and decode each selected segment independently.

Keep intra-segment conflicts fatal.

Preserve root first-identity behavior without allowing later segments to impersonate the root.

Preserve physical-file and range provenance through normalization and replay.

Stop boundary: ambiguous transitions continue to fail; no temporal linkage and no implicit cross-segment authority inheritance.

7. D — Missing-turn user-message policy

After segment provenance exists, evaluate whether a deterministic association invariant is available. If none is proven, the accepted behavior should be:

Preserve source, segment, record position, and message content as compatibility evidence.

Leave authoritative turn_id absent.

Mark the event explicitly as untrusted/no-claim.

Emit a structured non-authoritative diagnostic.

Exclude it from task, path, objective, lineage, dedupe, and other turn-derived authority.

A parse representation that merely makes turn_id: Option<_> and lets downstream code inherit ambient state is not acceptable.

Stop boundary: no previous-turn, next-turn, nearest-turn, or same-segment inheritance without a separately specified invariant.

8. F — Continuous expansion and final closure

Expand the bounded fixture matrix after each behavior packet, then run:

All sanitized 3–5-per-kind controls available for the 19 frozen operational kinds.

The frozen 85-sample local execution matrix.

The mixed-home F1 control.

Packet-specific negative authority controls.

Exact route, identity/segment range, normalized state/error rows, checkpoints, Sentinel completion, and process-result assertions.

F is therefore both the first and last packet operationally: baseline first, incremental additions throughout, full closure last.

5. Required inspections/tests
Packet A

Inspect only:

The supported root-locator behavior of discover_session_artifacts.

Every currently accepted way a child enters bounded closure.

Whether inbound-only lineage or self-nomination is supported.

Duplicate identity and exact-candidate ambiguity handling.

Reciprocal-lineage requirements.

Cache keys that currently use physical file identity.

Required tests:

Valid selected root plus unrelated pre-envelope source: succeeds.

The same malformed source explicitly selected or authoritatively claimed: fails closed.

Claimed child with no exact or reciprocal match: fails.

Two exact-looking candidates for one claim: fails as ambiguous.

Unrelated well-formed and malformed inventory does not alter the selected closure.

A multi-identity selected physical file remains rejected until B authorizes its segmentation; A must not silently reinterpret it.

Packet B

Inspect only:

All consumers of file-wide session_id and format.

Full-file routing in ingest_rollout_file.

Ordered session_meta handling.

Closure claims, turn state, and normalization state at proposed boundaries.

Provenance structures capable of carrying physical path plus record/byte range.

Required tests:

A single logical segment remains byte-for-byte semantically unchanged.

A supported identity/marker transition produces two stable ranges with exact provenance.

Repeated metadata that does not satisfy the segment-start contract does not create an accidental segment.

Conflicting identity or format inside one segment remains fatal.

An ambiguous transition remains rejected.

A later segment cannot satisfy or impersonate a request for the filename/first identity.

Child claims and turn-derived authority do not leak across segment boundaries.

Closure selection and full ingestion choose the same segment range and format.

Success should not be defined as “all 1,610 files now pass.” Only files satisfying the formal segment contract should change outcome.

Packet C

Inspect only:

The legacy dependency deserialization seam for output.

The current-native ToolOutput content policy.

Normalization equivalence for scalar and segmented output.

Whether images are retained, represented conservatively, or already handled by the shared content model.

Required tests using only the packet’s observed shapes:

Existing scalar string output is unchanged.

Typed input_text list preserves order and text.

Typed input_image follows the existing supported current-native tool-output policy.

Mixed image/text preserves order and the same content semantics.

Unsupported element kinds and malformed arrays still fail closed.

Source route remains legacy; only the field decoder changes.

Packet D

Inspect only:

Current-native event ordering and explicit turn lifecycle records.

Every downstream consumer that treats a user message as task, objective, path, lineage, or turn authority.

Whether an explicit neighboring record supplies an invariant stronger than adjacency.

Diagnostic row/status semantics under Packet E.

Required tests:

Existing explicit-turn_id user messages are unchanged.

Missing-turn_id messages are retained with source/segment/record provenance.

They have no authoritative turn association.

They cannot establish task, path, objective, child-lineage, or dedupe authority.

Previous and next explicit turns remain unaffected.

The diagnostic is machine-readable and does not masquerade as an authoritative message.

If a verified association rule is later adopted, positive and negative controls prove its exact boundary.

Packets E and F

The frozen execution results already establish that process status and semantic outcome diverge and that isolated homes cannot exercise global-discovery poisoning.

2026-07-31-rollout-compatibilit…

Required validation:

Exit zero plus any selected-source error row: wrapper failure.

Missing analyzer checkpoint: failure.

Missing or incomplete Sentinel replay: failure.

Timeout or incomplete artifact: failure.

Fully clean run: success.

Underlying nonzero exit: failure regardless of partial artifacts.

Results are reported per operational kind, not only as one aggregate pass rate.

The real-home control is always run separately from isolated-source fixtures.

After each packet, only its intended failure category should change. Remaining known failures must stay visibly red.

The local audit owner should rerun the frozen private matrix; the review does not need and should not request the raw corpus.

6. Spec decision
Packet	Required process
A	Formal security contract, implementation plan, and task ledger required. It changes which sources are allowed to affect closure preparation. The proof of irrelevance and exact closure-edge authority must be written before implementation.
B	Full formal specification, implementation plan, and task ledger mandatory. It introduces a new logical-source unit, range provenance, routing granularity, and root-identity behavior across discovery, ingestion, normalization, and replay.
C	Focused implementation task and regression plan; no full architecture spec initially. Escalate to a formal plan only if the narrow adapter is impossible and a dependency fork or broader shared decoder becomes necessary.
D	Formal authority-policy specification, implementation plan, and task ledger mandatory. The accepted representation must state exactly what provenance is retained and which authority claims are forbidden.
E	Explicit wrapper acceptance contract and focused task plan; no full architecture spec while wrapper-local. Any public CLI exit-contract change is a separate authorized task requiring its own specification.
F	Formal validation plan and task ledger required, but not a separate behavioral architecture spec. It should track fixture coverage, expected outcome transitions, frozen-matrix reruns, and completion gates across A–E.

A and B should share one vocabulary and interface contract for physical artifacts, logical segments, root locators, and closure nodes, but their implementation tasks should remain separate. D should remain a distinct authority-policy specification rather than being buried inside B’s parsing work.

7. Pitfalls

Implementing A as “ignore parse failures.” The safe rule is “do not strictly parse proven-unreachable bodies.” Every selected or potentially selectable source remains fail-closed.

Assuming root-first means no global enumeration. A tolerant directory or filename catalog may be necessary. Global fatal body indexing is the defect.

Baking filename equals logical identity into A. The audit observation that filenames matched first identities is useful compatibility evidence, not by itself a complete future addressability contract.

Treating every session_meta as a boundary. Packet B needs exact start/end predicates and an ambiguous-file fallback.

Using time proximity to connect epochs or turns. Neither B nor D may infer authority from chronology alone.

Solving F3 by rerouting legacy records as current-native-v2. Only the supported output field shape should be generalized.

Making turn_id optional without auditing consumers. Optional syntax plus ambient downstream inheritance would preserve the parse while introducing an authority vulnerability.

Representing D’s accepted diagnostic as a semantic error without deciding the consequence. If it remains kind:error, Packet E will correctly continue to fail the run; that would be preservation, not compatibility remediation.

Changing the public CLI under Packet E. The evidence establishes wrapper false-greens, not the intended public exit contract.

Landing F only after implementation. That would forfeit exact before/after causality and allow C or A improvements to conceal remaining B/D failures.

Interpreting C’s partial success as evidence that B is correct. C and B have independent rejection predicates and must retain separate fixtures.

Treating 1,892/12,180 as the overall failure rate. It is explicitly the population matching three known deterministic predicates under the audited behavior, not an estimate of total pipeline failure.

2026-07-31-rollout-compatibilit…

Expanding the work into a combinatorial parser audit. The regression scope should remain the frozen 19 operational kinds, the packet’s observed wire shapes, the specified authority negatives, and the single real-home poisoning control.


## Qualification follow-up

### Exact follow-up prompt

Bounded qualification follow-up on your completed review. Please validate or correct only the following two qualifications before we adopt the remediation sequence.

Qualification 1 — Packet B must remain evidence-gated:
We should not assume that every session_meta record starts a logical segment or that all 1,610 multi-identity physical files become recoverable. Packet B should accept only transition patterns authorized by the formal B₀ contract. Unsupported or ambiguous transitions must remain rejected, with intra-segment identity/format conflicts still fatal. Success must be measured by exact authorized patterns and preserved negative controls, not by making the full multi-identity population pass.

Qualification 2 — Packet D must define an accepted non-authoritative outcome:
If a missing-turn user message is merely retained as another kind:error row, Packet E will correctly keep the run red, so D would preserve evidence but would not remediate compatibility. The D specification must explicitly decide whether a turnless/no-claim user message is:
(a) an accepted degraded but semantic-clean, non-authoritative evidence state, with a machine-readable non-error diagnostic; or
(b) a hard semantic failure.
If (a), it must remain unable to establish task, objective, path, lineage, dedupe, or any turn-derived authority. If (b), we must not claim that D recovers those 63 files.

Please answer in exactly four sections:
1. Qualification 1: AGREE, ADJUST, or REJECT, with the exact invariant and acceptance metric.
2. Qualification 2: AGREE, ADJUST, or REJECT, including whether option (a), option (b), or a narrower rule is justified by the supplied evidence.
3. Contract impact: any required change to B₀, D, E, F, or the sequence F₀ → E → B₀ → A → C → B → D → F.
4. Final adoption language: a concise paragraph we can place into the formal plan.

Do not reopen F1-F5, request raw corpus evidence, enumerate new format variants, or widen beyond these two qualifications.

### ChatGPT Pro response

1. Qualification 1: AGREE

Invariant: Packet B may partition a physical rollout only when an ordered transition satisfies a deterministic boundary rule explicitly authorized by B₀. Every accepted logical segment must have exactly one session identity, exactly one parser format, a stable physical-file plus record/byte range, and unambiguous provenance. Any identity or format change not consumed by an authorized boundary, any ambiguous boundary, and any identity/format conflict within a segment remains fatal. Later segments must not impersonate the filename/first-identity root. The aggregate evidence supports the logical-segment hypothesis but expressly does not prove a universal segmentation rule.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…

Acceptance metric: For every transition pattern authorized by B₀, all positive fixtures must produce the exact expected segment count, ranges, identities, formats, provenance, route, and downstream result. All unsupported-transition, ambiguous-boundary, and intra-segment-conflict controls must remain rejected, and existing single-segment behavior must remain unchanged. The number or percentage of the 1,610 multi-identity files that pass is an observation, not an acceptance target.

2. Qualification 2: AGREE

The supplied evidence justifies a narrower form of option (a).

A syntactically otherwise valid current-native user_message whose only compatibility defect is the absence of turn_id may become a distinct accepted-degraded, turnless/no-claim evidence state. It must:

Preserve message content and exact source, segment, and record provenance.

Carry no inferred or ambient turn association.

Emit a machine-readable non-error degradation diagnostic.

Be structurally incapable of establishing task, objective, path, lineage, dedupe, or any other turn-derived authority.

Remain distinguishable from a fully associated user message throughout normalization, analysis, and replay.

It should count as successful pipeline completion under Packet E when no independent failure exists, but it should be reported as accepted-degraded, not collapsed into an undifferentiated full-fidelity “clean” result.

This rule applies only after tests prove that every authority-producing consumer rejects the turnless state. Until those gates are established, option (b) remains the safe behavior, and the affected file cannot be claimed as recovered. Malformed events or events with additional unsupported defects remain hard failures. The evidence establishes the missing-field failure and rules out guessed previous/next-turn attachment; it supports no broader association rule.

2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…



2026-07-31-rollout-compatibilit…

3. Contract impact

B₀: Add a closed boundary-decision contract. Each observed transition must resolve to one of: authorized segment boundary, permitted repetition within the same segment, or unsupported/ambiguous hard rejection. B₀ must define exact range ownership, root addressability, and provenance; it must not contain a population-recovery objective.

D: Define the turnless state as a separate normalized variant or equally strong typed representation—not an ordinary user message with an optional turn_id. Specify the allowed provenance, the non-error degradation code, the complete denial of authority effects, and the precise condition under which a file may be called recovered.

E: Distinguish at least three outcomes: fully clean, accepted-degraded, and failed. Both successful states require complete compaction, analyzer checkpoints, and Sentinel replay; any kind:error, timeout, incomplete artifact, or failed authority gate remains failure. Wrappers may succeed for an accepted-degraded run, but they must surface the degradation rather than report an unqualified clean result.

2026-07-31-rollout-compatibilit…

F: Assert exact B₀-authorized positive segment patterns and preserved rejection controls. For D, assert the accepted-degraded status, the machine-readable diagnostic, unchanged surrounding turns, and the complete absence of task, objective, path, lineage, dedupe, or other turn-derived authority artifacts. F must not use the number of recovered multi-identity or missing-turn files as its primary gate.

2026-07-31-rollout-compatibilit…

Sequence: No reorder is required. Retain:

F₀ → E → B₀ → A → C → B → D → F

F₀ records the current missing-turn case as a hard failure; E establishes trustworthy status handling; B₀ gates B; D follows B so it can retain stable segment provenance; and F verifies the exact outcome transition from failure to accepted-degraded. F remains an incremental validation spine throughout, with final closure at the end.

4. Final adoption language

Packet B is evidence-gated: only deterministic transition patterns expressly authorized by B₀ may create logical segments, each with one identity, one parser format, stable range provenance, and exact root-addressability semantics; unsupported or ambiguous transitions and all intra-segment identity or format conflicts remain fatal, and population-wide recovery is not an acceptance criterion. Packet D may recover a missing-turn file only by producing a distinct accepted-degraded, turnless/no-claim evidence state with a machine-readable non-error diagnostic and verified inability to establish task, objective, path, lineage, dedupe, or any turn-derived authority; otherwise the event remains a hard failure and the file is not counted as recovered. Packet E must distinguish fully clean, accepted-degraded, and failed completion, Packet F must prove the authorized positives and preserved negative controls exactly, and the sequence remains F₀ → E → B₀ → A → C → B → D → F.

## Local adoption and closeout

The two qualifications above are adopted as binding planning constraints. No remediation code was
implemented during this audit. The provisional operator-smoke drafts were not landed because the
audit proved that process exit alone can produce false-green results; their exact bytes and patch
were preserved in ignored repo-local normalization artifacts for the future Packet E work. Private
corpus evidence remains local and ignored. The next activity is planning from the validated sequence,
not widening this completed audit.
