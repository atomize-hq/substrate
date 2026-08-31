**Kind:** task row
**Stable ID:** `a1-4-auto-attach-producer-adoption`
**Canonical for:** extracted A1.4 task/checkpoint row only
**Status:** corrective remediation authorized and under proof; not terminally closed
**Authority scope:** exact extracted A1 bounded packet-decomposition table header and A1.4 row, plus the bounded corrective fence below; no independent slice or successor-slice authority
**Source span:** [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md) line 145 within `### A1 bounded packet decomposition`
**Supersedes:** canonical ownership of the extracted `A1.4 — bounded auto-attach producer adoption and regression closure` row
**Superseded by:** none
**Projection consumers:** [`../a1-host-session-authority.md`](../a1-host-session-authority.md), [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md)

# A1.4 — bounded auto-attach producer adoption and regression closure

> **Authority boundary:** A1.4 remains a substep/checkpoint inside A1 rather than an independent slice. Corrective remediation is authorized only for accepted findings `A14-DISC-P1-001` and `A14-DISC-P2-001`: mechanically migrate the existing auto-attach claim/settlement semantics to a narrow ObligationLedger-owned post-HSA persistence seam, require exact non-empty settlement, and correct the fresh validated V1 retryable-Start lookup. The earlier bounded-stop review record remains immutable. A1.4 and the A1/RG gates remain open until the complete Linux wall and successor review are clean.

The final executable wall also required the bounded public `run_stop`/HSA adoption previously
allocated to A2. That exact Stop intent, bound active-owner delivery, and durable terminal closeout
are pulled forward only as an A1.4 closure prerequisite; this does not dispatch or complete the
remainder of A2, and legacy-only Stop compatibility remains unchanged.

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, preserve ObligationLedger ownership, and remediate the accepted post-HSA settlement and fresh-Start proof blockers without changing semantics. | `agent_runtime/auto_attach.rs`; typed post-HSA claim/settlement and directly necessary persistence plumbing in `agent_runtime/{obligation_ledger,state_store}.rs` and the existing trusted store transaction module; fresh-Start continuity; the bounded HSA Stop contract and exact public/helper/REPL integration; focused colocated/public-control tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, idempotent applied result, exact non-empty settlement result, trusted-root/revision revalidation, validated greenfield V1 retry lookup, and one exact HSA Stop intent/delivery/terminal result. | No auto-attach policy, eligibility, claim meaning, or settlement meaning redesign; no legacy-writer relaxation or reactivation; no router responsibility expansion; no new ledger/coordinator/authority store; no compatibility Stop change; no remainder-of-A2 dispatch; no subsequent-slice work. | Prove exact claim/join/conflict handling and exact authenticated non-empty settlement on activated HSA without the legacy writer; prove fresh public Start upgrades from validated V1 while retaining no-replay/resume semantics; prove manual and auto-attach cannot substitute or double-apply an intent and retry/restart converges; prove public Stop reaches durable HSA Terminal without legacy authority writes or transport inference; then run every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` on real CLI and REPL paths. Only then may A1 close. | No A1 gate remains claimable until this packet's complete Linux wall and successor review pass; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows and the remainder of A2 remain unresolved, and later sibling seams remain out of scope. |
