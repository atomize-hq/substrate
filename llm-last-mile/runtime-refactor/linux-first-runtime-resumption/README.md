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
> `A1.3-P0` and `A1.3`. B2.2 remains a separate future-admission candidate, not admitted or
> dispatched. E3-A through E3-D are landed; E3-E is an admitted, incomplete and unlanded
> implementation candidate; E3-F is not dispatched. This index does not dispatch implementation,
> reopen the closed reentry gate, broaden
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
| E3-E Prepared/Held gateway startup/readiness; Consumed → ACK → ReadyClosed; cancellation/cleanup, exact same-live-attempt retry and fresh authorization after service restart | [Activation publication and ownership seam](../contracts/managed-gateway-adoption-v1.md#e3-e-activation-publication-and-ownership-seam) |
| E3-E ACK evidence and original Held-lease inputs; one-time handoff | [Activation callable boundaries](../contracts/agent-config-projection-v1.md#e3-e-activation-callable-boundaries), [E3 handoff clarification](../contracts/launch-time-secret-handoff-v1.md#e3-adoption-clarification-2026-09-02-outside-preserved-body) |
| E3-E versus E3-F ownership and unchanged successor requirements | [E3-E](../slices/e3-agent-config-projection-and-gateway-adoption.md#e3-e--dormant-managed-gateway-preparation-and-adoption), [E3-F](../slices/e3-agent-config-projection-and-gateway-adoption.md#e3-f--retained-v2-codex-launchresume-adoption-and-integrated-proof) |

The E3-E gateway activation seam correction landed as documentation at
`d3e82fa2d486be95ab5a2326c7c8ccd304e520f6`. Whole-E3-E and installed acceptance remain incomplete;
neither the narrow connected-recovery CLEAN nor documentation CLEAN approves whole-E3-E. These
links project status and navigation only; the contracts and E3 slice remain the normative owners.

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
