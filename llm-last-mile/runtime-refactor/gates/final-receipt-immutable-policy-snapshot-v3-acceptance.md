**Kind:** gate
**Status:** canonical
**Canonical for:** complete extracted B1 pre-E2 versus E2/B2.2 acceptance boundary, ordered conditions 1–11, immutable `PolicySnapshotV3` receipt exposure rule, post-acceptance immutable/future-only/revocation/fail-closed requirements, the current E2 completion-wall correction, and the `E2-RM` B2.2 admission prerequisite
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`](../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules), baseline lines 152–180; the exact 1675-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a`

## Current E2 completion-wall correction

The E2 completion unit is the independently valid
[`DispatchPolicyCommitmentV1`](../contracts/dispatch-policy-commitment-v1.md), not a final receipt
or retained manifest. E2 persists it and atomically links it to exact B1 plus source-owned B2.1
`WorldWorkExecutionClaimV1` truth before accepted work is reported. For fresh Spawn, the complete
E2 reservation object/ref is durable and `fsync`ed before B3.2a; B3.2a consumes its authenticated
preallocated identities; then E2 retains that ref, links the stable B3.2a source fields and exact
registration, and publishes the immutable commitment before success is reported. Mutable B3.2a
state/revision is not used as the durable link. Fork instead uses the exact E2-owned fork-dispatch link before child-launch success and
creates no B3.2a authority. B2.2/B3.2 later consume the E2 ref when constructing receipts. D1/E3
later compose their owned identities when the full retained manifest is produced.

Accordingly, extracted condition 8 is a downstream receipt/envelope composition wall, not an E2
prerequisite. The `resumable` requirement in extracted condition 10 is historical and
noncontrolling: B2.1's exact `WorldWorkExecutionClaimV1` has no such field, and resumability remains
with its existing HSA/retained-runtime owner. Condition 11 is satisfied by the receipt owner's exact
consumption of the already-durable E2 ref and the exact source-owned B2.1 link. No E2-only record may
masquerade as a final receipt or complete retained manifest.

The snapshot bytes in conditions 6–7 are exactly E1's
`serde_json::to_vec(PolicySnapshotV3)` bytes, and the hash is the existing E1/B1 SHA-256 over that
same sequence. E2 record/index/link hashes may use their own domain-separated deterministic
preimages, but no recursively key-sorted replacement is a policy snapshot hash.

## Current `E2-RM` / B2.2 admission correction

E2's completion satisfies persistence and immutable linkage, but does not by itself give B2.2 a
recoverable post-response-loss read path. The existing authenticated E2 operation requires an
already-known commitment ref. Therefore condition 11 also requires the separately bounded
[`E2-RM` authenticated accepted-work receipt-material
projection](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite)
before B2.2 can be freshly admitted.

`E2-RM` must resolve the exact committed request/subject index from authenticated authority plus
expected B1 acceptance, validate the immutable E2 record/ref/linkage hash, return the preserved
historic B2.1 claim preimage/hash and exact E1 snapshot bytes/ref/hash/revision/reason, and validate
the retained cap bytes/ref/hash where applicable. It never reads current B2.1 observer state or
current parent policy to reconstruct history. Missing legacy history returns typed
`UnsupportedLegacyState`; ambiguity, corruption, substitution, conflict, or hash failure fails
closed.

This is a read-only E2 prerequisite, not receipt construction or exposure. B1/B2.1 ownership is
unchanged, B2.2 still owns foreground receipt construction/return, B3.2 still owns remaining
receipt/manifest/messaging/lifecycle work, B4 still owns targeted control, and D1/E3 retain their
future envelope/projection fields. `E2-RM` and B2.2 each require later fresh admission and explicit
dispatch; this documentation correction admits neither and changes no completed E2 gate result.

## E3 projection/full-manifest clarification

E3's independently valid `ConfigProjectionIdentityV1` and `ConfigProjectionRefV1` remain later
full-manifest inputs, not final active task/turn receipt construction and not an E2-RM or B2.2
prerequisite. E3 validates the immutable E2 launch/fork cap it consumes without changing E2 or
reconstructing policy from current parent. D1 later owns the execution envelope on strict V3; B3.2
later owns complete retained-manifest construction. The E3 documentation specification admits or
implements none of those owners and does not change any E2 or E2-RM status.

<!-- exact-extracted-body:start -->
## 12. Final-receipt immutable `PolicySnapshotV3` acceptance rules

B1's pre-E2 acceptance anchor records the exact current policy identity used by the runtime but is
not a final receipt and is not model-facing. E2 owns the immutable active-run snapshot and retained
worker cap below; B2.2 may expose a receipt only after those commitments and the B2.1 observation
claim are durable and linked to the B1 record.

A final active task/turn receipt may be exposed only when all are true:

1. exact session, caller, backend, and world binding are resolved;
2. steering policy allows the verb/mode/target;
3. current parent policy is resolved at a known revision;
4. retained worker cap is loaded and hash-verified when applicable;
5. optional narrowing is validated as monotonic;
6. the resulting `PolicySnapshotV3` canonicalizes and passes existing schema/enforcement validation;
7. snapshot bytes/ref/hash/revision are durable;
8. the execution envelope/world-service request carries the same verified snapshot;
9. runtime acceptance evidence joins the acknowledgement to the exact work identity;
10. the observation claim is durable and resumable; and
11. the receipt references the snapshot, acceptance evidence, and observation claim before the foreground caller is told the work was accepted.

After acceptance:

- the receipt's policy ref/hash is immutable;
- parent broadening or narrowing does not rewrite the active receipt;
- parent changes apply to future task acceptance, continue, fork, or worker turns;
- emergency revocation is an explicit audited cancel/revoke path, never silent snapshot mutation; and
- snapshot mismatch at broker/world-service fails closed.

<!-- exact-extracted-body:end -->
