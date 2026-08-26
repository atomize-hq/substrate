**Kind:** gate
**Status:** canonical
**Canonical for:** complete extracted tests-prove-before-landed promotion gate and ordered proof requirements 1–10 covering serialization, persistence, real-path usage, restart and replay, smoke and e2e joins, one-time gateway handoff proof, compatibility exclusion, and B1/B2.1-to-B3.1 sequencing
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#14-contract-promotion-gates`](../04-contracts-and-gates.md#14-contract-promotion-gates), baseline lines 215–231; the exact 1106-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43`

<!-- exact-extracted-body:start -->
## 14. Contract promotion gates

A contract is not considered landed until tests prove:

1. serialization and validation;
2. atomic persistence and revision conflict handling;
3. the real ingress/dispatch/runtime path uses it;
4. restart/replay behavior where durable;
5. fail-closed negative cases;
6. every revision-bound host transition joins intent issuance, claim, authority application, and exact result on the real CLI and REPL path, including crash reconciliation and no-reapply exact retry;
7. at least one smoke/e2e path joins session, binding, policy, receipt, runtime event, and terminal/obligation truth;
8. credential-requiring world UAA proof joins the envelope to a consumed one-time in-world gateway handoff without copied secret files or inherited descriptors; and
9. no compatibility copy or `CompatibilityUnproven` evidence is used for contract promotion; and
10. B1/B2.1 production proof shows both accepted work families enter the durable supervisor without
    a legacy-writer attempt, caller/foreground drop does not erase truth, and B3.1 begins only after
    the joint closeout.

<!-- exact-extracted-body:end -->
