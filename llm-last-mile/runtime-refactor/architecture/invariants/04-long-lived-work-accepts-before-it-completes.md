**Kind:** architecture
**Stable ID:** `shared-invariant-04`
**Canonical for:** invariant 4 only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted invariant source body only; no implementation authority
**Source span:** [`../../01-target-architecture.md#4-long-lived-work-accepts-before-it-completes`](../../01-target-architecture.md#4-long-lived-work-accepts-before-it-completes) lines 153–165
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md)

# Invariant 4: Long-lived work accepts before it completes

### 4. Long-lived work accepts before it completes

`run_world_task` and `continue_world_worker` persist accepted receipts and return durable handles before terminal exit. A blocking UX may wait on the receipt; it may not redefine the core contract. The supervisor—not the foreground tool call—owns the terminal-framed stream.

Durable observation ownership may land before model-visible early return: the foreground may remain
a compatibility waiter over the receipt while the supervisor alone ingests and closes the stream.
At the exact acceptance transition, the supervisor claim must become durable before any subsequent
frame can be consumed outside its journal. Dropping a foreground waiter or guard cannot delete an
accepted record, supervisor claim, journal entry, or supervised work.

#### B2.1-3 restart and producer-replay boundary

Canonical content: [`b1-b2-1/architecture.md#b21-3-restart-and-producer-replay-boundary`](../../b1-b2-1/architecture.md#b21-3-restart-and-producer-replay-boundary).
