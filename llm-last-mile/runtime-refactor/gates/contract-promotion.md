**Kind:** gate
**Status:** canonical
**Canonical for:** complete extracted tests-prove-before-landed promotion gate and ordered proof requirements 1–10 covering serialization, persistence, real-path usage, restart and replay, smoke and e2e joins, one-time gateway handoff proof, compatibility exclusion, and B1/B2.1-to-B3.1 sequencing
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#14-contract-promotion-gates`](../04-contracts-and-gates.md#14-contract-promotion-gates), baseline lines 215–231; the exact 1106-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `0fa77777b7f8ae85bc4e06082454b243677d89d070ff4c510bd9e148b61b2d43`

## Current E2 promotion disposition

This authority correction does not land or promote E2. Any later fresh E2 dispatch must prove the
commitment's record/index/link serialization; exact E1/B1
`serde_json::to_vec(PolicySnapshotV3)` bytes/hash without a second canonicalization; strict real
tool-to-`WorldDispatchRequestV1` translation; real `ExecuteRequest`/retained-turn carrier use; exact
B1 plus source-owned B2.1 `WorldWorkExecutionClaimV1` linkage; retained current-parent drift with
immutable cap preservation; E2 fork-dispatch linkage; typed mixed-version failure; and the mandatory
`RG-DIFF-01` differential wall.

Fresh Spawn additionally proves first-writer immutable E2 reservation-object/ref publication with a
keyed commitment over the complete validated request and no prompt/payload preimage, plus
file/directory `fsync` before B3.2a; private E2 recomputation/equality of that commitment followed by
injection of only an opaque authenticated reservation capability/ref and the reservation's
preallocated identities through the bounded admission input; the exact
`allow_capability_narrowing` validation exception gated by a distinct opaque attestation that only
authenticated nonempty `RestrictedWorldFs` E1 narrowing can obtain, never `UnchangedParent` or an
empty patch; use of the
unchanged B3.2a plan/fingerprint/schema; a stable
source-field/registration admission link that survives mutable B3.2a revisions; retained
reservation ref and exact full-request commitment in the committed index and final record;
post-admission E2 CAS publication;
identical retry joins; changed-request/material pre-admission conflicts; and restart at every
reservation/admission/publication boundary. An unreserved production B3.2a admission is unsupported
after activation. D1/E3 receipt/manifest
composition is later-owner proof, not an E2 prerequisite.

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
