**Kind:** packet index
**Stable ID:** `A1.3-family`
**Canonical for:** no authority; Linux-first A1.3 family navigation only
**Status:** non-authoritative navigation
**Authority scope:** none; follow the linked canonical owners and shared gate records
**Source span:** D6 assembly only
**Supersedes:** none
**Superseded by:** none
**Projection consumers:** [`../index/README.md`](../index/README.md)

# A1.3 Linux-first packet family index

> **Authority boundary:** This file only assembles links for terminally complete `A1.3-P1`,
> terminally complete A1.4/A1, A2, A3, Track A, E1, and E2, and the inseparable held predecessors
> `A1.3-P0` and `A1.3`. B2.2 and E3 are separate future-admission candidates, neither admitted nor
> dispatched. This index does not dispatch implementation, reopen the closed reentry gate, broaden
> macOS or Windows scope, or replace the linked canonical owners.

| Component | Canonical owner |
|---|---|
| Controlling decision | [`DECISION.md`](DECISION.md) |
| Completed packet | [`A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md`](A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md) |
| Completed A1.4/A1 closure | [`../slices/tasks/a1-4-auto-attach-producer-adoption.md`](../slices/tasks/a1-4-auto-attach-producer-adoption.md) |
| Held packet | [`A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md) |
| Held packet | [`A1.3-LINUX-FIRST-PACKET.md`](A1.3-LINUX-FIRST-PACKET.md) |
| Extracted slice/task family rows | [`slice-and-task.md`](slice-and-task.md) |
| Closed global selection gate | [`../gates/authority-required-runtime-refactor-reentry.md`](../gates/authority-required-runtime-refactor-reentry.md) |
| Current non-authoritative state projection | [`../index/current.md`](../index/current.md) |

A1.3-P1 and A1.4 are terminally complete under their exact commit, tree, reviewed fingerprint,
CLEAN review, and verification-evidence identities recorded in their owners; the enclosing A1
slice, A2, A3, and Track A are terminally complete. This family index does not admit or dispatch
E2.

Historical predecessors stay linked in their preserved packet records. The macOS developer-parity
lane remains separate under [`../macos-dev-parity/DECISION.md`](../macos-dev-parity/DECISION.md),
and Windows remains deferred.

No uniquely canonical A1.3-family span remains in `../01-target-architecture.md` or
`../04-contracts-and-gates.md`, so those root compatibility and shared-contract surfaces remain
unchanged by this D6 landing.
