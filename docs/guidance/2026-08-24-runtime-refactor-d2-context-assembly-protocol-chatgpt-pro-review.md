# ChatGPT Pro advisory review: runtime-refactor D2 per-slice-context-assembly-protocol landing

- Date: 2026-08-24
- Source chat: https://chatgpt.com/c/6a8babc0-0dc4-83ea-b77a-777f0bf54255
- Model: exact model not surfaced in the visible ChatGPT UI; visible chat surface `Chat` with effort `Extra High`
- Effort setting: Extra High
- Task/spec: independent initial-range review of the bounded D2 per-slice-context-assembly-protocol foundation landing only
- Current approach: review the corrected three-file docs-only patch against the exact local baseline, without widening scope beyond this one D2 foundation landing

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Prompt sent

~~~text
Review exactly one bounded docs-only D2 foundation landing patch.

Scope and authority:
- This is D2 only. Do not reopen D1. Do not start D3 or later.
- Review only for qualifying blocking issues within the patch itself.
- Ignore style preferences and non-blocking nits.

Patch identity and local validation already completed:
- Patch digest: `sha256:47e048e02b663a78998f3a6c0f363be1f03ae6d07e48b2e2678a6bfb93e8cf9c`
- Path fence validated: `00-README.md`, `foundations/per-slice-context-assembly-protocol.md`, `migration/extraction-ledger.md` only
- `git apply --check` passed on the current local baseline
- Extracted destination substantive span revalidated byte-for-byte against current local `00-README.md#per-slice-context-assembly-protocol`, baseline lines `93–132`, SHA-256 `d5f91b758a8d0d3ae774c09d4c668010a22a10ccac8dd4beb3082763467f1094`

Review questions:
1. Does the patch preserve the exact source heading/legacy anchor in `00-README.md` while moving only the body to the canonical foundation file?
2. Does it preserve the extracted substantive span byte-for-byte in the destination foundation file, with only the expected small non-substantive metadata header and provenance footer added?
3. Does it add exactly one correct control-pack-map row and exactly one correct extraction-ledger row, with extraction-specific rollback text and no competing authority?
4. Does it avoid pulling in D3+ governance, scheduling prose, or unrelated cleanup?

Return format:
- If there is no qualifying blocking issue, respond exactly with `APPROVED` and then a short justification.
- If there is a qualifying blocking issue, respond with a short findings list containing only blocking issues, each with the exact file/path and why it blocks approval.

Patch under review:
```diff
diff --git a/llm-last-mile/runtime-refactor/00-README.md b/llm-last-mile/runtime-refactor/00-README.md
--- a/llm-last-mile/runtime-refactor/00-README.md
+++ b/llm-last-mile/runtime-refactor/00-README.md
@@ -82,6 +82,7 @@
 | [`foundations/semantic-status-labels.md`](foundations/semantic-status-labels.md) | classifying a seam or reviewing a promotion claim | semantic status labels and the promotion rule |
 | [`foundations/authority-vocabulary.md`](foundations/authority-vocabulary.md) | interpreting authority terms or reviewing a control-plane boundary | authority vocabulary |
 | [`foundations/reading-and-update-rules.md`](foundations/reading-and-update-rules.md) | reading or updating the control pack | reading and update rules |
+| [`foundations/per-slice-context-assembly-protocol.md`](foundations/per-slice-context-assembly-protocol.md) | assembling implementation or review context for a slice | per-slice context assembly protocol |
 | [`review-control/`](review-control/) | opening, closing, or extending a review cycle | small standard-library cycle record, validator, example, and focused tests |
 
 Do not load the full historical design/debug stack by default. Start with the applicable crosswalk
@@ -92,41 +93,5 @@
 
 ## Per-slice context assembly protocol
 
-Treat context assembly as part of every implementation and review slice. Do not hand an agent the entire directive, design family, debug history, control pack, and source tree at once.
-
-Assemble four bounded packets:
-
-1. **Authority packet — what must be true:** the exact `03` slice row, affected `02` seam rows, applicable `01` invariants, exact `04` contract sections, and only the design sections named by the slice.
-2. **Repo-truth packet — what is true now:** the current production call path, files allowed by the slice, related types and tests, one relevant precedent when available, and fresh call-graph/impact evidence. Separate artifact existence, semantic correctness, real-path adoption, and runtime proof.
-3. **Proof packet — how completion is judged:** exact `05` gate rows, targeted tests, negative/fail-closed cases, required smoke/e2e evidence, and the classification change permitted by that evidence.
-4. **Review packet — how change is challenged without expanding scope:** the selected integrated
-   outcome, exact subject fingerprint, `04` priority rubric and cycle budget, fresh reviewer lenses,
-   cycle-record path, mechanical prechecks, and any applicable `06` inventory entries.
-
-Target fewer than 2,000 focused lines per implementation task. Historical debug documents are regression provenance, not current implementation authority. Conversation history and prior summaries are discovery hints only until revalidated against the current tree.
-
-Use this capsule at slice start:
-
-```text
-SLICE / OBJECTIVE:
-SELECTED INTEGRATED OUTCOME / COMPLETION CLAIM:
-TARGET AUTHORITY BOUNDARY:
-CURRENT PRODUCTION PATH / SEMANTIC STATUS:
-MUST-READ SECTIONS:
-LIVE SOURCE / TESTS / PRECEDENT:
-SIBLING SEAMS IN CONTEXT:
-ALLOWED CODE AREAS / EXPLICIT NON-GOALS:
-APPLICABLE CONTRACTS / REGRESSION GATES:
-KNOWN CORRECTIONS OR CONFLICTS:
-EXIT PROOF / STOP CONDITIONS:
-REVIEW BUDGET / CYCLE RECORD:
-```
-
-If target docs, live code, tests, or fresh runtime evidence conflict, record the conflict in `KNOWN CORRECTIONS OR CONFLICTS` and resolve it before implementation. Never silently select the source that makes the slice appear easiest or most complete.
-
-For B2.1-3, keep three facts separate in every capsule and review: the durable supervisor claim
-and cursor, the process-memory world-service producer replay registry, and the shell startup hook
-that invokes the canonical supervisor recovery operation. Only `WorldWorkExecutionSupervisor`
-interprets durable claims or performs restart discovery and reconciliation. Producer replay only
-retains and transports exact B0 frames, and a startup surface only activates the canonical owner.
+Canonical content is maintained in [`foundations/per-slice-context-assembly-protocol.md`](foundations/per-slice-context-assembly-protocol.md).
 
diff --git a/llm-last-mile/runtime-refactor/foundations/per-slice-context-assembly-protocol.md b/llm-last-mile/runtime-refactor/foundations/per-slice-context-assembly-protocol.md
new file mode 100644
--- /dev/null
+++ b/llm-last-mile/runtime-refactor/foundations/per-slice-context-assembly-protocol.md
@@ -0,0 +1,46 @@
+**Kind:** foundation
+**Status:** canonical
+**Canonical for:** per-slice context assembly protocol
+
+## Per-slice context assembly protocol
+
+Treat context assembly as part of every implementation and review slice. Do not hand an agent the entire directive, design family, debug history, control pack, and source tree at once.
+
+Assemble four bounded packets:
+
+1. **Authority packet — what must be true:** the exact `03` slice row, affected `02` seam rows, applicable `01` invariants, exact `04` contract sections, and only the design sections named by the slice.
+2. **Repo-truth packet — what is true now:** the current production call path, files allowed by the slice, related types and tests, one relevant precedent when available, and fresh call-graph/impact evidence. Separate artifact existence, semantic correctness, real-path adoption, and runtime proof.
+3. **Proof packet — how completion is judged:** exact `05` gate rows, targeted tests, negative/fail-closed cases, required smoke/e2e evidence, and the classification change permitted by that evidence.
+4. **Review packet — how change is challenged without expanding scope:** the selected integrated
+   outcome, exact subject fingerprint, `04` priority rubric and cycle budget, fresh reviewer lenses,
+   cycle-record path, mechanical prechecks, and any applicable `06` inventory entries.
+
+Target fewer than 2,000 focused lines per implementation task. Historical debug documents are regression provenance, not current implementation authority. Conversation history and prior summaries are discovery hints only until revalidated against the current tree.
+
+Use this capsule at slice start:
+
+```text
+SLICE / OBJECTIVE:
+SELECTED INTEGRATED OUTCOME / COMPLETION CLAIM:
+TARGET AUTHORITY BOUNDARY:
+CURRENT PRODUCTION PATH / SEMANTIC STATUS:
+MUST-READ SECTIONS:
+LIVE SOURCE / TESTS / PRECEDENT:
+SIBLING SEAMS IN CONTEXT:
+ALLOWED CODE AREAS / EXPLICIT NON-GOALS:
+APPLICABLE CONTRACTS / REGRESSION GATES:
+KNOWN CORRECTIONS OR CONFLICTS:
+EXIT PROOF / STOP CONDITIONS:
+REVIEW BUDGET / CYCLE RECORD:
+```
+
+If target docs, live code, tests, or fresh runtime evidence conflict, record the conflict in `KNOWN CORRECTIONS OR CONFLICTS` and resolve it before implementation. Never silently select the source that makes the slice appear easiest or most complete.
+
+For B2.1-3, keep three facts separate in every capsule and review: the durable supervisor claim
+and cursor, the process-memory world-service producer replay registry, and the shell startup hook
+that invokes the canonical supervisor recovery operation. Only `WorldWorkExecutionSupervisor`
+interprets durable claims or performs restart discovery and reconciliation. Producer replay only
+retains and transports exact B0 frames, and a startup surface only activates the canonical owner.
+
+**Source provenance:** extracted byte-for-byte from [`00-README.md#per-slice-context-assembly-protocol`](../00-README.md#per-slice-context-assembly-protocol), baseline lines 93–132
+**Baseline span SHA-256:** `d5f91b758a8d0d3ae774c09d4c668010a22a10ccac8dd4beb3082763467f1094`
diff --git a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
--- a/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
+++ b/llm-last-mile/runtime-refactor/migration/extraction-ledger.md
@@ -6,3 +6,4 @@
 | D1 | [`00-README.md`](../00-README.md) | Semantic status labels | [`semantic-status-labels`](../00-README.md#semantic-status-labels) | lines 143–156 | `d9179c4258ed8d17f4a2127729d5b537fad56c284ea05d6e0488eb9f66853736` | [`foundations/semantic-status-labels.md`](../foundations/semantic-status-labels.md) | foundation | canonical destination; source compatibility anchor | none | restore exact source span; remove destination, D1 control-pack-map row, and D1 ledger entry; delete ledger file if empty |
 | D2 | [`00-README.md`](../00-README.md) | Authority vocabulary | [`authority-vocabulary`](../00-README.md#authority-vocabulary) | lines 148–179 | `9a08c1887f75093c73f60fe40928387b27be340bb754620ff73ce0b2b659902c` | [`foundations/authority-vocabulary.md`](../foundations/authority-vocabulary.md) | foundation | canonical destination; source compatibility anchor | none | restore exact source span; remove D2 destination; remove D2 control-pack-map row; remove D2 ledger entry; delete ledger file if it would become empty |
 | D2 | [`00-README.md`](../00-README.md) | Reading and update rules | [`reading-and-update-rules`](../00-README.md#reading-and-update-rules) | lines 152–162 | `fcefeb722469f836cacbca46839854facc11f578fbe181f76914a4af97ca7457` | [`foundations/reading-and-update-rules.md`](../foundations/reading-and-update-rules.md) | foundation | canonical destination; source compatibility anchor | none | restore exact source span; remove `foundations/reading-and-update-rules.md`; remove `foundations/reading-and-update-rules.md` control-pack-map row; remove D2 `Reading and update rules` ledger entry; delete ledger file only if it would become empty |
+| D2 | [`00-README.md`](../00-README.md) | Per-slice context assembly protocol | [`per-slice-context-assembly-protocol`](../00-README.md#per-slice-context-assembly-protocol) | lines 93–132 | `d5f91b758a8d0d3ae774c09d4c668010a22a10ccac8dd4beb3082763467f1094` | [`foundations/per-slice-context-assembly-protocol.md`](../foundations/per-slice-context-assembly-protocol.md) | foundation | canonical destination; source compatibility anchor | none | restore exact source span; remove `foundations/per-slice-context-assembly-protocol.md`; remove the `foundations/per-slice-context-assembly-protocol.md` control-pack-map row; remove D2 `Per-slice context assembly protocol` ledger entry; delete ledger file only if it would become empty |
```
~~~

## Answer received

~~~markdown
APPROVED

The patch preserves the legacy `## Per-slice context assembly protocol` heading/anchor, moves only its body to the canonical foundation, preserves the validated substantive span byte-for-byte, adds exactly one control-pack-map row and one extraction-ledger row with appropriate rollback instructions, establishes no competing authority, and introduces no D3+ or unrelated changes.
~~~
