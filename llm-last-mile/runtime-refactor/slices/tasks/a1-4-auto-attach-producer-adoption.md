**Kind:** task row
**Stable ID:** `a1-4-auto-attach-producer-adoption`
**Canonical for:** extracted A1.4 task/checkpoint row only
**Status:** canonical task row record
**Authority scope:** exact extracted A1 bounded packet-decomposition table header and A1.4 row only; no slice, schedule, dispatch, or implementation authority
**Source span:** [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md) line 145 within `### A1 bounded packet decomposition`
**Supersedes:** canonical ownership of the extracted `A1.4 — bounded auto-attach producer adoption and regression closure` row
**Superseded by:** none
**Projection consumers:** [`../a1-host-session-authority.md`](../a1-host-session-authority.md), [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md)

# A1.4 — bounded auto-attach producer adoption and regression closure

> **Authority boundary:** This file owns only the extracted A1.4 row below. A1.4 is the named implementation successor to terminally complete A1.3-P1, but it still requires fresh admission and explicit dispatch as a substep/checkpoint inside A1. It is not an independent slice and is not authorized, admitted, dispatched, implemented, scheduled, or complete here.

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, then close A1 without changing projection policy or settlement ownership. | Only `build_auto_attach_launch_plan` and directly required plan integration in `agent_runtime/auto_attach.rs`; focused auto-attach producer/consumer tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, and idempotent applied result. | No auto-attach policy, eligibility, claim, or settlement redesign; no router responsibility expansion; no endpoint/path redesign; no A2 demotion; no subsequent-slice work. | Prove manual and auto-attach cannot substitute or double-apply an intent, retry/restart converges, every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` pass on real CLI and REPL paths; only then may A1 close. | No A1 gate remains claimable until this packet's final wall passes; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows remain unresolved for A2/A3, and later sibling-seam gates remain out of scope. |
