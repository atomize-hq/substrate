**Kind:** foundation
**Status:** canonical
**Canonical for:** per-slice context assembly protocol

## Per-slice context assembly protocol

Treat context assembly as part of every implementation and review slice. Do not hand an agent the entire directive, design family, debug history, control pack, and source tree at once.

Assemble four bounded packets:

1. **Authority packet — what must be true:** the exact `03` slice row, affected `02` seam rows, applicable `01` invariants, exact `04` contract sections, and only the design sections named by the slice.
2. **Repo-truth packet — what is true now:** the current production call path, files allowed by the slice, related types and tests, one relevant precedent when available, and fresh call-graph/impact evidence. Separate artifact existence, semantic correctness, real-path adoption, and runtime proof.
3. **Proof packet — how completion is judged:** exact `05` gate rows, targeted tests, negative/fail-closed cases, required smoke/e2e evidence, and the classification change permitted by that evidence.
4. **Review packet — how change is challenged without expanding scope:** the selected integrated
   outcome, exact subject fingerprint, `04` priority rubric and cycle budget, fresh reviewer lenses,
   cycle-record path, mechanical prechecks, and any applicable `06` inventory entries.

Target fewer than 2,000 focused lines per implementation task. Historical debug documents are regression provenance, not current implementation authority. Conversation history and prior summaries are discovery hints only until revalidated against the current tree.

Use this capsule at slice start:

```text
SLICE / OBJECTIVE:
SELECTED INTEGRATED OUTCOME / COMPLETION CLAIM:
TARGET AUTHORITY BOUNDARY:
CURRENT PRODUCTION PATH / SEMANTIC STATUS:
MUST-READ SECTIONS:
LIVE SOURCE / TESTS / PRECEDENT:
SIBLING SEAMS IN CONTEXT:
ALLOWED CODE AREAS / EXPLICIT NON-GOALS:
APPLICABLE CONTRACTS / REGRESSION GATES:
KNOWN CORRECTIONS OR CONFLICTS:
EXIT PROOF / STOP CONDITIONS:
REVIEW BUDGET / CYCLE RECORD:
```

If target docs, live code, tests, or fresh runtime evidence conflict, record the conflict in `KNOWN CORRECTIONS OR CONFLICTS` and resolve it before implementation. Never silently select the source that makes the slice appear easiest or most complete.

For B2.1-3, keep three facts separate in every capsule and review: the durable supervisor claim
and cursor, the process-memory world-service producer replay registry, and the shell startup hook
that invokes the canonical supervisor recovery operation. Only `WorldWorkExecutionSupervisor`
interprets durable claims or performs restart discovery and reconciliation. Producer replay only
retains and transports exact B0 frames, and a startup surface only activates the canonical owner.

**Source provenance:** extracted byte-for-byte from [`00-README.md#per-slice-context-assembly-protocol`](../00-README.md#per-slice-context-assembly-protocol), baseline lines 93–132
**Baseline span SHA-256:** `d5f91b758a8d0d3ae774c09d4c668010a22a10ccac8dd4beb3082763467f1094`
