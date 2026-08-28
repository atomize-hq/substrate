# ChatGPT Pro advisory review: runtime-refactor D2 reading-and-update-rules landing

- Date: 2026-08-24
- Source chat: https://chatgpt.com/c/6a8b9d81-dc24-83ea-9cbe-b10296e87bc9
- Model: exact model not surfaced in the visible ChatGPT UI; visible chat surface `Chat` with effort `Extra High`
- Effort setting: Extra High
- Task/spec: independent initial-range review of the bounded D2 reading-and-update-rules foundation landing only
- Current approach: review the returned three-file docs-only patch against the exact local baseline, without widening scope beyond this one D2 foundation landing

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Prompt sent

~~~text
Review mode: initial-range
Review context: independent fresh conversation
Review boundary: exact local baseline HEAD `3f4819b3c9323869fc7fc9a80d8d89d3e66ed4f5` / tree `7ab1f8ff3d6b0007f9ba861415b761499f077c4f`, candidate patch digest `sha256:dfdda821aca17511e41167412f0b15415522795d8cea31edd1918409cf36f585`
Review target: complete bounded docs-only D2 foundation landing for `Reading and update rules`, affecting only `llm-last-mile/runtime-refactor/00-README.md`, `llm-last-mile/runtime-refactor/foundations/reading-and-update-rules.md`, and `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`

Supported inputs and behavior: Markdown documentation only; unified Git patch against the stated local baseline; current local D1 and first D2 foundation landings already exist in the baseline and must remain intact
Supported platforms and dialects: repository Markdown only; no platform-specific runtime behavior; no code execution or schema migration outside the three named paths
In-scope invariants:
- path fence is exactly the three named paths and no others
- source heading `## Reading and update rules` and legacy anchor `#reading-and-update-rules` remain in `00-README.md`
- only the moved body is replaced in `00-README.md` with a direct compatibility pointer
- extracted substantive span from `00-README.md` lines 152–162 is preserved byte-for-byte in the destination file and still hashes to `fcefeb722469f836cacbca46839854facc11f578fbe181f76914a4af97ca7457`
- destination file follows the same small non-substantive metadata-header pattern as the existing `semantic-status-labels.md` and `authority-vocabulary.md` foundation files
- exactly one control-pack-map row is added for `foundations/reading-and-update-rules.md`
- exactly one extraction-ledger row is appended for this slice while existing D1 and D2 rows remain unchanged except as surrounding context
- rollback text is extraction-specific: it names the exact destination file, the exact control-pack-map row, the exact ledger entry, and says to delete the ledger file only if it would become empty
- no competing authority, scheduling prose, D3+ content, or unrelated wording cleanup is introduced
Required material consequence: a blocking finding must show that the patch would break canonical-authority ownership, corrupt the content-preserving extraction, misstate rollback/audit truth, violate the exact path fence, or otherwise leave this D2 landing incomplete or misleading in live use
Blocking threshold: report a blocking finding only for a reachable, material defect in this bounded patch; prefer conservative false negatives over speculative or stylistic findings
Accepted prior findings: none
Deferred or out-of-scope concerns: stylistic alternatives, broader roadmap/tracker questions, D3+ decomposition, whether other future D2 sections should use different row wording, and any repository content not included below
Project sources: No repo bundle, no Project source index, and no local files are attached to this fresh chat. Treat only the exact bounded facts and diff below as evidence. Do not infer unavailable repository state.
Validation evidence: local validation already passed `process_pro_return_patch.py validate` against the current bundle baseline; touched paths are exactly the three named files; apply stat is `3 files changed, 20 insertions(+), 8 deletions(-)`; local precheck confirmed the destination patch contains the exact extracted substantive span matching SHA-256 `fcefeb722469f836cacbca46839854facc11f578fbe181f76914a4af97ca7457`

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Return findings first by severity with file/function, supported-input reachability, violated invariant, material consequence, and delta causality. State explicitly when there are no qualifying findings.

Candidate patch:
```diff
diff --git a/llm-last-mile/runtime-refactor/00-README.md b/llm-last-mile/runtime-refactor/00-README.md
--- a/llm-last-mile/runtime-refactor/00-README.md
+++ b/llm-last-mile/runtime-refactor/00-README.md
@@ -82,6 +82,7 @@
 | [`06-review-finding-inventory.md`](06-review-finding-inventory.md) | classifying, retaining, deduplicating, or resolving non-blocking review findings | the single `P3`/`P4` review and process-debt inventory; never a `P1`/`P2` waiver |
 | [`foundations/semantic-status-labels.md`](foundations/semantic-status-labels.md) | classifying a seam or reviewing a promotion claim | semantic status labels and the promotion rule |
 | [`foundations/authority-vocabulary.md`](foundations/authority-vocabulary.md) | interpreting authority terms or reviewing a control-plane boundary | authority vocabulary |
+| [`foundations/reading-and-update-rules.md`](foundations/reading-and-update-rules.md) | reading or updating the control pack | reading and update rules |
 | [`review-control/`](review-control/) | opening, closing, or extending a review cycle | small standard-library cycle record, validator, example, and focused tests |
 
 Do not load the full historical design/debug stack by default. Start with the applicable crosswalk
@@ -152,12 +153,5 @@
 
 ## Reading and update rules
 
-1. Treat this pack as canonical for refactor intent, slice boundaries, contracts, and gates.
-2. Treat live code plus fresh runtime evidence as canonical for current artifact truth.
-3. If code truth changes, update the affected crosswalk row and regression row in the same implementation PR.
-4. Keep a seam below `ContractCorrectAndProven` until its actual production path is proven.
-5. Do not use helper/PID/socket liveness as authority in new contracts.
-6. Do not create one crate per named seam. A seam may be a module, facade, trait, type, or extracted function set.
-7. Keep slice boundaries hard. Adjacent sibling seams stay in context but are not implicit scope.
-8. Preserve resolved debug behavior while replacing the model that produced it.
+Canonical authority for these rules is now in [`foundations/reading-and-update-rules.md`](foundations/reading-and-update-rules.md).
 
diff --git a/llm-last-mile/runtime-refactor/foundations/reading-and-update-rules.md b/llm-last-mile/runtime-refactor/foundations/reading-and-update-rules.md
new file mode 100644
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/foundations/reading-and-update-rules.md
@@ -0,0 +1,17 @@
+**Kind:** foundation
+**Status:** canonical
+**Canonical for:** reading and update rules
+
+## Reading and update rules
+
+1. Treat this pack as canonical for refactor intent, slice boundaries, contracts, and gates.
+2. Treat live code plus fresh runtime evidence as canonical for current artifact truth.
+3. If code truth changes, update the affected crosswalk row and regression row in the same implementation PR.
+4. Keep a seam below `ContractCorrectAndProven` until its actual production path is proven.
+5. Do not use helper/PID/socket liveness as authority in new contracts.
+6. Do not create one crate per named seam. A seam may be a module, facade, trait, type, or extracted function set.
+7. Keep slice boundaries hard. Adjacent sibling seams stay in context but are not implicit scope.
+8. Preserve resolved debug behavior while replacing the model that produced it.
+
+**Source provenance:** extracted byte-for-byte from [`00-README.md#reading-and-update-rules`](../00-README.md#reading-and-update-rules), baseline lines 152–162
+**Baseline span SHA-256:** `fcefeb722469f836cacbca46839854facc11f578fbe181f76914a4af97ca7457`
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -6,3 +6,4 @@
 |---|---|---|---|---|---|---|---|---|---|---|
 | D1 | [`00-README.md`](../00-README.md) | Semantic status labels | [`semantic-status-labels`](../00-README.md#semantic-status-labels) | lines 143–156 | `d9179c4258ed8d17f4a2127729d5b537fad56c284ea05d6e0488eb9f66853736` | [`foundations/semantic-status-labels.md`](../foundations/semantic-status-labels.md) | foundation | canonical destination; source compatibility anchor | none | restore exact source span; remove destination, D1 control-pack-map row, and D1 ledger entry; delete ledger file if empty |
 | D2 | [`00-README.md`](../00-README.md) | Authority vocabulary | [`authority-vocabulary`](../00-README.md#authority-vocabulary) | lines 148–179 | `9a08c1887f75093c73f60fe40928387b27be340bb754620ff73ce0b2b659902c` | [`foundations/authority-vocabulary.md`](../foundations/authority-vocabulary.md) | foundation | canonical destination; source compatibility anchor | none | restore exact source span; remove D2 destination; remove D2 control-pack-map row; remove D2 ledger entry; delete ledger file if it would become empty |
+| D2 | [`00-README.md`](../00-README.md) | Reading and update rules | [`reading-and-update-rules`](../00-README.md#reading-and-update-rules) | lines 152–162 | `fcefeb722469f836cacbca46839854facc11f578fbe181f76914a4af97ca7457` | [`foundations/reading-and-update-rules.md`](../foundations/reading-and-update-rules.md) | foundation | canonical destination; source compatibility anchor | none | restore exact source span; remove `foundations/reading-and-update-rules.md`; remove `foundations/reading-and-update-rules.md` control-pack-map row; remove D2 `Reading and update rules` ledger entry; delete ledger file only if it would become empty |
```

~~~

## Answer received

~~~markdown
Findings
No qualifying blocking findings.
I do not see a reachable, material defect in the supplied bounded D2 patch under the stated blocking threshold.

Review disposition

Path fence: satisfied. The diff touches exactly:
- llm-last-mile/runtime-refactor/00-README.md
- llm-last-mile/runtime-refactor/foundations/reading-and-update-rules.md
- llm-last-mile/runtime-refactor/migration/extraction-ledger.md

Legacy source compatibility: satisfied. ## Reading and update rules remains in 00-README.md, preserving #reading-and-update-rules, and its moved body is replaced by a direct pointer to the canonical destination.

Content-preserving extraction: satisfied by the supplied diff plus validation evidence. The destination contains the eight substantive rules under the original heading, and the local precheck confirms the extracted span remains byte-identical at SHA-256 fcefeb722469f836cacbca46839854facc11f578fbe181f76914a4af97ca7457.

Canonical-authority ownership: coherent. The new foundation declares Status: canonical / Canonical for: reading and update rules; the legacy README surface redirects to it rather than retaining a competing copy.

Destination metadata: appropriately limited to the stated foundation metadata/provenance pattern; I see no new substantive authority or scheduling policy introduced around the extracted body.

Control-pack map: exactly one new row is added for foundations/reading-and-update-rules.md.

Extraction ledger: exactly one new row is appended for this extraction. The existing D1 and prior D2 rows shown in the patch remain unchanged.

Rollback/audit truth: sufficiently extraction-specific. It identifies:
- exact destination foundations/reading-and-update-rules.md;
- the foundations/reading-and-update-rules.md control-pack-map row;
- the D2 Reading and update rules ledger entry;
- and conditions ledger-file deletion on it becoming empty.

Scope discipline: no D3+ material, unrelated cleanup, competing authority, or additional scheduling prose appears in the candidate delta.

Conclusion
Approve from this initial-range review. On the supplied evidence, the candidate is a complete, content-preserving D2 foundation landing and does not meet the stated threshold for a blocking finding.

~~~
