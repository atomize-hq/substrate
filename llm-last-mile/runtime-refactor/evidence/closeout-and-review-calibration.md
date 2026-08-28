**Kind:** evidence/regression
**Status:** canonical
**Canonical for:** exact extracted `Closeout rule` and `Review-process calibration` bodies
**Authority scope:** documentation decomposition only; exact extracted evidence/regression source bodies only
**Source provenance:** extracted byte-for-byte from [`../05-debug-regression-ledger.md#closeout-rule`](../05-debug-regression-ledger.md#closeout-rule), baseline lines 193–201 inclusive (`347` bytes), and [`../05-debug-regression-ledger.md#review-process-calibration`](../05-debug-regression-ledger.md#review-process-calibration), baseline lines 332–353 inclusive (`1382` bytes); each exact source body is preserved independently between its matching boundary markers below; the bodies are noncontiguous in the source file and are not represented as one contiguous extraction
**Closeout rule source SHA-256:** `f81142a2bf6d0cbd6b31a27011260da25108cc518b582dd1be81268f94635460`
**Review-process calibration source SHA-256:** `284284ac507a9bdb3263f2c18bc615c05df3fd23ed33a7a6742f85e29fbe175e`
**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
**Superseded by:** none
**Relationship to D3 governance:** reference-only to [`../contracts/development-review-and-remediation-contract.md`](../contracts/development-review-and-remediation-contract.md); this owner does not reopen D3 governance
**Authority boundary:** this evidence/regression owner does not satisfy a gate, change a proof result, authorize implementation, promotion, remediation, or dispatch
**Projection consumers:** [`README.md`](README.md), [`../index/README.md`](../index/README.md)

# Closeout and review calibration

<!-- exact-extracted-body:closeout-rule:start -->
## Closeout rule

An implementation PR may mark a ledger row resolved only when:

1. its owning crosswalk seam has the correct owner and call path for that behavior;
2. the named permanent gate passes on the real path;
3. adjacent resolved baselines remain green; and
4. the evidence distinguishes durable success from transport/process success.

<!-- exact-extracted-body:closeout-rule:end -->

<!-- exact-extracted-body:review-process-calibration:start -->
## Review-process calibration

RP4 exposed a process failure without exposing a product or test regression: a blanket requirement
for four clean reviews allowed findings about agent-created cache-only orchestration to expand the
product-proof acceptance surface. Repeated fix/review attempts then improved bespoke evidence
tooling rather than the selected Substrate outcome. Human disposition correctly preserved the raw
`REQUEST_CHANGES` review while accepting the independently evaluable product proof.

The prospective correction is owned by the development-review contract in `04`:

- `P1`/`P2` block only on demonstrated impact to the selected contract, gate, scope, or completion
  claim;
- one discovery cycle, one consolidated remediation, one closure cycle, and at most two directly
  causal supplemental cycles bound automatic work;
- `CLEAN` is terminal, mechanical-only deltas do not create review cycles, and unrelated or expanded
  blockers stop for authority rather than widening scope; and
- valid non-blocking review/process debt is retained in `06`, separate from this product regression
  ledger.

The three RP4 persistence findings are registered as `RR-RF-0001` through `RR-RF-0003`. Their raw
review files, hashes, and original verdict remain unchanged. This calibration authorizes no
controller/supervisor remediation and changes no RP3/RP4/RP5 proof result.
<!-- exact-extracted-body:review-process-calibration:end -->
