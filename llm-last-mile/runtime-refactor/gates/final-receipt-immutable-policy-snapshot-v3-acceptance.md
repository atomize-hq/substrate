**Kind:** gate
**Status:** canonical
**Canonical for:** complete extracted B1 pre-E2 versus E2/B2.2 acceptance boundary, ordered conditions 1–11, immutable `PolicySnapshotV3` receipt exposure rule, post-acceptance immutable/future-only/revocation/fail-closed requirements, and the current E2 completion-wall correction
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules`](../04-contracts-and-gates.md#12-final-receipt-immutable-policysnapshotv3-acceptance-rules), baseline lines 152–180; the exact 1675-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `672c9ba3db1e792a7afb145a4081450deafe6a1123262ae7c104c6f1b4b0b24a`

## Current E2 completion-wall correction

The E2 completion unit is the independently valid
[`DispatchPolicyCommitmentV1`](../contracts/dispatch-policy-commitment-v1.md), not a final receipt
or retained manifest. E2 persists it and atomically links it to exact B1/B2.1 truth before accepted
work is reported, or to exact B3.2a admission before fresh-Spawn success is reported. Fork instead
uses the exact E2-owned fork-dispatch link before child-launch success and creates no B3.2a
authority. B2.2/B3.2 later consume the E2 ref when constructing receipts. D1/E3 later compose their
owned identities when the full retained manifest is produced.

Accordingly, extracted condition 8 is a downstream receipt/envelope composition wall, not an E2
prerequisite, and condition 11 is satisfied by the receipt owner's exact consumption of the
already-durable E2 ref. No E2-only record may masquerade as a final receipt or complete retained
manifest.

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
