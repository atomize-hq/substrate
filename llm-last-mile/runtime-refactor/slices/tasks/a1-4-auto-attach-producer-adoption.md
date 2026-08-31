**Kind:** task row
**Stable ID:** `a1-4-auto-attach-producer-adoption`
**Canonical for:** extracted A1.4 task/checkpoint row only
**Status:** terminally complete; enclosing A1 terminally complete
**Authority scope:** exact extracted A1 bounded packet-decomposition table header and A1.4 row, plus the bounded corrective fence below; no independent slice or successor-slice authority
**Source span:** [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md) line 145 within `### A1 bounded packet decomposition`
**Supersedes:** canonical ownership of the extracted `A1.4 — bounded auto-attach producer adoption and regression closure` row
**Superseded by:** none
**Projection consumers:** [`../a1-host-session-authority.md`](../a1-host-session-authority.md), [`../../03-phase-slice-map.md`](../../03-phase-slice-map.md)

# A1.4 — bounded auto-attach producer adoption and regression closure

> **Authority boundary:** A1.4 remains a substep/checkpoint inside A1 rather than an independent slice. It is terminally complete under the exact implementation and review identities below. The earlier bounded-stop, correction, and final successor-review records remain immutable; this closure neither widens the completed implementation fence nor authorizes a successor.

The final executable wall also required the bounded public `run_stop`/HSA adoption previously
allocated to A2. That exact Stop intent, bound active-owner delivery, and durable terminal closeout
were pulled forward solely to satisfy the A1 final wall. That adoption is complete and must not be
repeated in A2; the remainder of A2 is not admitted, dispatched, implemented, or completed here,
and legacy-only Stop compatibility remains unchanged.

The bounded row below preserves the admitted implementation fence and exit criteria; the terminal
closure following it records that those criteria passed.

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, preserve ObligationLedger ownership, and remediate the accepted post-HSA settlement and fresh-Start proof blockers without changing semantics. | `agent_runtime/auto_attach.rs`; typed post-HSA claim/settlement and directly necessary persistence plumbing in `agent_runtime/{obligation_ledger,state_store}.rs` and the existing trusted store transaction module; fresh-Start continuity; the bounded HSA Stop contract and exact public/helper/REPL integration; focused colocated/public-control tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, idempotent applied result, exact non-empty settlement result, trusted-root/revision revalidation, validated greenfield V1 retry lookup, and one exact HSA Stop intent/delivery/terminal result. | No auto-attach policy, eligibility, claim meaning, or settlement meaning redesign; no legacy-writer relaxation or reactivation; no router responsibility expansion; no new ledger/coordinator/authority store; no compatibility Stop change; no remainder-of-A2 dispatch; no subsequent-slice work. | Prove exact claim/join/conflict handling and exact authenticated non-empty settlement on activated HSA without the legacy writer; prove fresh public Start upgrades from validated V1 while retaining no-replay/resume semantics; prove manual and auto-attach cannot substitute or double-apply an intent and retry/restart converges; prove public Stop reaches durable HSA Terminal without legacy authority writes or transport inference; then run every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` on real CLI and REPL paths. Only then may A1 close. | No A1 gate remains claimable until this packet's complete Linux wall and successor review pass; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows and the remainder of A2 remain unresolved, and later sibling seams remain out of scope. |

## Terminal closure

The completed implementation is bound to:

- commit `9cb9fc78b8f2efdc7a8e90a456f6abaf13dbf851`, with direct parent
  `9d1b84d62e213450acbb28de7c9a98a91811c2b5`;
- tree `947e00f1cc3df101f37c1b663351f7db453dc331`;
- reviewed subject fingerprint
  `sha256:fc8b5a2cdda50a3dee44089b6a8f62f047feafbcfc202b05cc7822e693dba800`;
- full committed-diff fingerprint
  `sha256:6f9990bc03b20a107e396156d1a6f0fd01089e83208b8575a8881217987e0247`;
  and
- final independent verdict `CLEAN`, recorded in the preserved
  [final successor review](../../review-control/a1-4-auto-attach-producer-adoption-final-successor-review.txt).

The verified final wall recorded:

- HSA `207/207`, auto-attach `18/18`, retained-worker runtime `84/84`, and typed prompt
  fulfillment `7/7`;
- a real installed-witness `Start` → retained `Turn`/reattach → terminal cut → `Stop`;
- a green installed-witness world doctor and a successful real world command;
- `cargo check -p shell --lib`, `cargo build -p substrate --bin substrate`, and
  `cargo build -p substrate --bin substrate-shim`;
- `cargo fmt --all -- --check` and `git diff --check`; and
- documentation-only closure validation leaving every product and test path byte-identical to the
  implementation commit.

`RG-BASE-01` and `RG-BASE-02` are green. The A1-scoped clauses of `RG-AUTH-01` and
`RG-AUTH-02`, and all of `RG-AUTH-03`, are green; the ledger-wide remainder of
`RG-AUTH-01`/`RG-AUTH-02` stays unresolved for A2/A3 and other named owners. A1.1/A1.2 and
A1.3-P1 were already complete, the Linux A1.1d closeout is landed, and the controlling
[Linux-first decision](../../linux-first-runtime-resumption/DECISION.md#macos-lane) makes native
macOS a separate non-predecessor lane. No other A1 condition remains: A1.4 and the enclosing A1
slice are terminally complete.
