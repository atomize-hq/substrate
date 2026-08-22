# ChatGPT Pro advisory review: runtime-refactor decomposition D0 baseline

- Date: 2026-08-22
- Source chat: https://chatgpt.com/c/6a890462-cf44-83ea-9faf-bbcb0891e21e
- Model: GPT-5.6 Sol
- Effort setting: Pro
- Task/spec: independent advisory review of the completed D0 read-only baseline only
- Current approach: review the completed D0 baseline for completeness and scope discipline; do not authorize D1, propose source-document edits, or alter the ZIP

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Prompt sent

~~~text
We are already working on this project task.

Task/spec: Complete D0 only: freeze and record the read-only baseline required before D1 of the runtime-refactor control-plane decomposition. Review the completed D0 baseline for completeness and scope discipline only.

Current approach: Direct local repository evidence on branch feat/runtime-refactor-decomposition-d0-baseline at HEAD/upstream 60f34a7064d21a6755308e94abf564af6a554a6e shows a docs-only D0 baseline record plus one machine-readable JSON companion under docs/guidance. The baseline records the source-root file inventory, byte hashes, heading tree, Markdown links/anchors, links leaving the source root, explicit authority assignments, explicit supersession edges, current status-token surfaces, path-frozen review subjects, the exact D1 source span, and a bounded comparison proving the current source root still matches commit 6ab6032fe6f0f917be68982adba416d2ded707e3. The ZIP SHA-256 34d7c3c4f40867af7d71963a310c809eaf84e87ec77bcdd108b3ec133b77454e is unchanged. The bundle-manifest head_sha 7fdacfd12590baec3c258e909b65caadee4212f8 is treated as observed, unclassified, and non-authoritative. Those are direct local facts, not inference.

Project sources: No repo bundle, no Project source index, and no local files are attached to this fresh chat. Treat only the bounded facts below as evidence. Do not infer unavailable repository state.

Evidence, blockers, constraints, and validation:
- Bound repo state: branch feat/runtime-refactor-decomposition-d0-baseline; HEAD 60f34a7064d21a6755308e94abf564af6a554a6e; tree 31020fdd8b5311521a2a35d0c10a39c99a2843d1; upstream origin/feat/runtime-refactor-decomposition-tracker at the same SHA; clean worktree before edits.
- Source root: llm-last-mile/runtime-refactor with 148 files and 2698308 bytes.
- Inventory manifest SHA-256: 61aea8276c3d18e028c4deefc776c38218cfb7b6b7e295eec96d83a501ab8acd.
- Heading tree SHA-256: e554fee1fbf24f11db49d307ad173ae1bef6a1ac38cbd96bdef09c0a295d8688.
- Link graph SHA-256: c6376d59b1ec4f3db055db9cf77ee319b0a3b6483f2242ba503bc6d61ee43e2f.
- Markdown link results: 0 invalid in-root file/anchor links; 31 external links; 4 out-of-root relative links.
- Curated authority assignments captured: 8 explicit entries, including the canonical 00 control-pack surface, the single durable 06 P3/P4 inventory, the active A1.3-P1 Linux-first packet, the macOS lane decision/gate, the authoritative R3 planning subject path list, the R2-4 frozen review subject boundary, and the authoritative R3 evidence-retirement planning constraint.
- Curated supersession edges captured: 5 explicit entries, including the macOS decision superseding the former R3 macOS/recovery/retirement/finalizer/Windows predecessor sequence for active scheduling only, and the historical R3_RESUME replacement being superseded as the global schedule by AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY.
- Status-token surfaces captured: 5 semantic status labels; review-inventory statuses open/accepted/scheduled/resolved/superseded; 8 document status lines; review JSON statuses EVIDENCE_CLEAN/LANDED_CLEAN/bounded_stop/complete; cycle kinds closure/discovery/supplemental_causal; cycle verdicts clean/findings; 18 AUTHORITY_REQUIRED tokens.
- Frozen review-subject surfaces captured: 3 .sha256 subject manifests, 29 review-cycle JSON records with subject fingerprints, and 9 explicit subject-path sections.
- Exact D1 source span: 00-README.md, heading Semantic status labels, legacy anchor semantic-status-labels, lines 143-156, SHA-256 d9179c4258ed8d17f4a2127729d5b537fad56c284ea05d6e0488eb9f66853736.
- Bounded comparison to commit 6ab6032fe6f0f917be68982adba416d2ded707e3: 148 head files, 148 base files, 0 head-only, 0 base-only, 0 blob mismatches; all files still match.
- Strict exclusions: do not authorize D1, do not propose any edit under llm-last-mile/runtime-refactor, do not propose D1 patches or successor work, do not change runtime/test/platform lanes, and do not modify or rebuild the ZIP.
- Desired use of your answer: advisory review only. If you recommend adjust, limit it to D0 baseline/tracker completeness or scope-discipline issues; do not convert the review into D1 design advice.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; tradeoffs; what to keep/change first; inspections; validation; and pitfalls. Do not authorize D1 and do not propose source-document edits.
~~~

## Answer received

~~~markdown
# Recommendation: **KEEP**

The current approach is the smallest safe and substantively complete way to finish D0. Based only on the bounded evidence, there is no reason to pivot and no demonstrated need for a substantive adjustment.

The baseline establishes all of the information needed for a read-only freeze:

* Exact repository identity: branch, HEAD, tree, upstream, and source-root identity.
* Exact content identity: 148 files, 2,698,308 bytes, per-file byte hashes, and aggregate manifest hashes.
* Structural identity: heading tree, anchors, Markdown link graph, and links crossing the source-root boundary.
* Semantic-control identity: explicit authority assignments, explicit supersession edges, status-token surfaces, and frozen review subjects.
* A narrowly bounded D1 locator: exact file, heading, legacy anchor, line range, and span hash—without designing or authorizing D1.
* Provenance containment: exact equality with source commit `6ab6032fe6f0f917be68982adba416d2ded707e3`, unchanged ZIP identity, and no elevation of the unclassified `bundle-manifest.json` `head_sha`.

D0 therefore appears complete as a baseline record and disciplined in scope. This review does **not** authorize D1.

## Why the approach is sound

### 1. Content identity is established independently of the ambiguous manifest field

The strongest evidence is not the generic bundle manifest. It is the bounded comparison showing:

* 148 files at HEAD and at `6ab6032…`
* no files present on only one side
* no blob mismatches

That directly establishes that the current `llm-last-mile/runtime-refactor` source root still has the content of `6ab6032…`.

Treating `7fdacfd12590baec3c258e909b65caadee4212f8` as observed, unclassified, and non-authoritative is therefore correct. D0 neither ignores the value nor invents a provenance meaning for it.

### 2. The baseline freezes more than bytes

A byte inventory alone would have been insufficient for later content-preserving decomposition. The additional inventories correctly preserve the control-plane semantics most likely to be damaged accidentally:

| Surface                  | Why it belongs in D0                                                                   |
| ------------------------ | -------------------------------------------------------------------------------------- |
| Heading tree and anchors | Detects structural movement and anchor loss                                            |
| Link graph               | Preserves navigation and cross-document dependencies                                   |
| Authority assignments    | Prevents projections or historical records from becoming canonical accidentally        |
| Supersession edges       | Preserves the difference between historical existence and current scheduling authority |
| Status vocabularies      | Prevents normalization from silently changing process meaning                          |
| Frozen review subjects   | Protects evidence and review identities from path or content drift                     |
| Exact D1 span            | Defines a later extraction boundary without carrying out the extraction                |

This is the correct division between machine-derived inventories and curated semantic records. Mechanical facts can be generated; authority and supersession must be explicitly traced rather than inferred.

### 3. Scope discipline is strong

The described D0 work is limited to a human-readable baseline and a machine-readable companion under `docs/guidance`. The source root remains byte-identical to the reference commit, and the ZIP remains unchanged.

The record also appears descriptive rather than prescriptive: it captures existing authority, existing supersession, existing statuses, and an existing source span. It does not establish new scheduling authority, rewrite historical material, or make decomposition decisions.

## Tradeoffs

### Curated semantic inventories versus exhaustive interpretation

The eight authority assignments and five supersession edges are necessarily curated. That creates a risk of omission, but trying to infer implicit authority would be more dangerous and would exceed D0. The correct standard is traceability to explicit source language—not an attempt to produce a newly synthesized global authority model.

### Exact line and span hashes versus flexibility

The D1 span is deliberately brittle. Line numbers or bytes changing should invalidate the recorded span. That brittleness is desirable for a frozen baseline because silent drift would be worse than an explicit validation failure.

### Human-readable record plus JSON companion

The pair improves auditability and machine checking, but creates a representation-divergence risk. The two should not be allowed to become independent competing authorities. Their field ownership and parity should be explicit or mechanically verifiable.

### Recorded external links versus external-link validation

Recording 31 external links and four out-of-root relative links is appropriate. D0 does not need to prove that external destinations remain live or correct, and attempting to repair or reinterpret them would move beyond baseline capture.

## What to keep

Keep the following without substantive modification:

1. The separation between the frozen source root and the D0 records under `docs/guidance`.
2. The exact source-root inventory, byte counts, per-file hashes, and aggregate hashes.
3. The bounded comparison against `6ab6032…`.
4. The explicit qualification that the macOS decision supersedes its predecessor sequence for **active scheduling only**, rather than erasing the historical record.
5. The separation between current authority, historical status, review status, cycle kind, cycle verdict, and `AUTHORITY_REQUIRED` surfaces.
6. The frozen subject manifests, review-cycle fingerprints, and explicit subject-path sections.
7. The exact D1 locator and span hash, with no extraction or migration guidance.
8. The observed-but-unclassified treatment of the generic manifest SHA.
9. The unchanged ZIP hash:
   `34d7c3c4f40867af7d71963a310c809eaf84e87ec77bcdd108b3ec133b77454e`.

## What to change first

No source-document or D1 change is warranted.

The first action should be a final, read-only D0 closure inspection. Only a failed inspection should cause a D0-only correction.

One detail deserves particular attention: the supplied evidence explicitly says the worktree was clean **before edits**, but does not separately state that the final checkout at `60f34a7064d21a6755308e94abf564af6a554a6e` is clean. If the committed baseline already records final cleanliness, nothing should change. If it records only pre-edit cleanliness, the smallest safe adjustment is to add a final-state attestation to the D0 record or tracker:

* HEAD `60f34a7064d21a6755308e94abf564af6a554a6e`
* tree `31020fdd8b5311521a2a35d0c10a39c99a2843d1`
* upstream at the same commit
* empty final worktree/index status
* unchanged ZIP SHA-256

That would be baseline bookkeeping only; it would not touch the runtime-refactor source root or authorize anything downstream.

The second conditional adjustment is representation parity. If the record does not already define the relationship between the prose baseline and the JSON companion, clarify within D0 which one is the canonical explanatory record and that the JSON is its machine-readable projection—or define exact field ownership where they differ. Do not create a second semantic authority.

## Required closure inspections

### Repository and changed-path boundary

Confirm at the final accepted state:

* Local HEAD is exactly `60f34a7064d21a6755308e94abf564af6a554a6e`.
* The commit tree is exactly `31020fdd8b5311521a2a35d0c10a39c99a2843d1`.
* The named upstream resolves to the same commit.
* The index and worktree are empty.
* The D0 commit-level diff contains only the intended D0 baseline, JSON companion, and any intended D0 tracker record.
* No runtime, test, installer, platform, or `llm-last-mile/runtime-refactor` source path changed.

The differently named local branch and upstream branch are not inherently problematic, but the exact pair should remain recorded rather than silently normalized.

### Mechanical baseline reproduction

From the accepted tree, reproduce and compare:

| Check                          | Required result                                                    |
| ------------------------------ | ------------------------------------------------------------------ |
| Source-root file count         | `148`                                                              |
| Source-root byte count         | `2698308`                                                          |
| Inventory manifest SHA-256     | `61aea8276c3d18e028c4deefc776c38218cfb7b6b7e295eec96d83a501ab8acd` |
| Heading tree SHA-256           | `e554fee1fbf24f11db49d307ad173ae1bef6a1ac38cbd96bdef09c0a295d8688` |
| Link graph SHA-256             | `c6376d59b1ec4f3db055db9cf77ee319b0a3b6483f2242ba503bc6d61ee43e2f` |
| Invalid in-root Markdown links | `0`                                                                |
| External links                 | `31`                                                               |
| Out-of-root relative links     | `4`                                                                |

The reproduction rules should make path ordering, byte treatment, line numbering, anchor derivation, and treatment of non-regular entries unambiguous. This is an inspection target, not a reason to redesign the baseline generator.

### Human/JSON parity

Verify that both representations agree on at least:

* Repository and source identities.
* All counts and aggregate hashes.
* The eight authority assignments.
* The five supersession edges and their scope qualifiers.
* Every status vocabulary and the count of 18 `AUTHORITY_REQUIRED` tokens.
* The three subject manifests, 29 review-cycle records, and nine subject-path sections.
* The D1 file, heading, anchor, lines, and span hash.
* The base-comparison result.
* The ZIP hash and unclassified manifest SHA treatment.

Any mismatch should be corrected in D0 only. It should not be resolved by altering the source documents.

### Semantic traceability

For every curated authority assignment and supersession edge, inspect that the D0 entry:

* Points to an explicit source location.
* Separates quoted or directly observed semantics from reviewer explanation.
* Preserves qualifiers such as “for active scheduling only.”
* Does not convert a historical statement into present authority.
* Does not turn an `AUTHORITY_REQUIRED` token into an implied decision.
* Does not treat a review verdict or cycle status as document authority.

For the status inventory, confirm that identically spelled tokens from different schemas remain classified by their original surface rather than merged into a single normalized enum.

### Frozen-subject integrity

Recalculate the three `.sha256` subject manifests and the fingerprints referenced by the 29 review-cycle JSON records. Confirm that all nine explicit subject-path sections resolve to the recorded paths and that no D0 record “repairs” or substitutes a subject path.

A missing or stale subject should produce a D0 validation failure, not an inferred replacement.

### D1 locator integrity

Read-only extraction of `00-README.md` lines 143–156 should produce:

`d9179c4258ed8d17f4a2127729d5b537fad56c284ea05d6e0488eb9f66853736`

Also confirm:

* The enclosing heading is exactly `Semantic status labels`.
* The legacy anchor is exactly `semantic-status-labels`.
* The heading/anchor locator is unique enough to detect line-number drift.
* The D0 record contains no proposed destination path, rewritten wording, extraction patch, or successor sequencing.

### ZIP and provenance containment

Recalculate the ZIP SHA-256 and confirm it remains:

`34d7c3c4f40867af7d71963a310c809eaf84e87ec77bcdd108b3ec133b77454e`

The generic bundle-manifest SHA should remain recorded as an observation only. It must not be silently relabeled as:

* the source commit,
* the bundle construction commit,
* the D0 baseline commit,
* or evidence that contradicts the verified source-root comparison.

## Pitfalls to avoid

1. **Equating the unclassified manifest SHA with source provenance.** The exact blob comparison is the authoritative evidence available here.

2. **Treating D0 acceptance as D1 authorization.** D0 can be accepted as a completed baseline while all downstream authority remains separately gated.

3. **Repairing external or out-of-root links.** D0 records those edges; it does not adjudicate or edit their targets.

4. **Normalizing status vocabularies.** `accepted`, `complete`, `clean`, `LANDED_CLEAN`, and similar values may belong to different schemas and must retain their original meanings.

5. **Collapsing supersession into deletion.** A scheduling predecessor can be superseded while remaining historically authoritative for what it recorded at the time.

6. **Allowing prose and JSON to drift.** A machine-readable companion is useful only if it remains a faithful projection or has clearly bounded field ownership.

7. **Regenerating the ZIP.** Rebuilding it would change the evidentiary object even if its extracted source files happened to remain the same.

8. **Using the D1 span to begin decomposition design.** The span is a frozen review subject, not permission to select destinations, wording, navigation, or migration order.

9. **Expanding the curated authority map through inference.** Any unresolved or ambiguous semantic relationship should remain unresolved or explicitly unclassified rather than being “completed” by reviewer judgment.

## Final disposition

Accept and retain the current D0 approach. No pivot is justified, and no substantive baseline rewrite is supported by the bounded facts.

The only possible D0-only adjustments are mechanical closure corrections discovered by the inspections above—most notably a missing final clean-state attestation or an undefined human/JSON parity relationship. Otherwise, preserve the committed baseline unchanged and stop at D0.

~~~
