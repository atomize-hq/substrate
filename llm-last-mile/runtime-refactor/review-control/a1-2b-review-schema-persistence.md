# A1.2b schema, persistence, and lineage review

Terminal subject fingerprint:
`sha256:72ac97adbbadd1692409782789daea7f1c64cdd34996563865b4d5959afc4b0b`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard/default speed inspected
the final A1.2b persistence surface without authoring subject bytes. The terminal focus was strict
V2-to-V3 preservation, no-successor runtime reconstruction, applied-successor lineage
reconstruction, object-index/reachability validation, and exact reopen/retry behavior.

## Recorded causal lineage

The discovery subject was
`sha256:ecc3fe2f83745ec07507f4ef21d1758a020068e2755c624eeaed3e7a01c58df3`.
It produced one accepted persistence finding:

- `A1-2B-P2-002`: V3 validation still accepted a no-successor current authority that drifted away
  from the exact reconstructed V2 runtime state.

The closure subject was
`sha256:be2b7f4352153c6bfd206fd223eab81c2b13510e824ccd1aa5087e7bf75564c8`.
That changed subject closed `A1-2B-P2-002` but still exposed one accepted lineage finding:

- `A1-2B-P1-002`: applied successor reopen still accepted a stored
  `resulting_authoritative_lineage` that was not exactly `precondition lineage + [target]`,
  because validation only required containment and reconstruction copied the stored lineage
  verbatim.

## Supplemental causal evidence

The terminal subject closes `A1-2B-P1-002` and preserves the prior no-successor fix:

- `crates/shell/src/execution/agent_runtime/host_session_authority/store_schema.rs` now compares
  every no-successor current authority to the exact reconstructed V2 runtime state and rejects
  successor intents whose lineage differs from the exact precondition-lineage extension or contains
  duplicates.
- `reconstruct_successor_initial_state` now derives the applied successor lineage from the
  reconstructed prior authority plus the exact target instead of trusting stored successor bytes.
- Focused tests
  `strict_v3_validation_rejects_no_successor_runtime_authority_drift`,
  `strict_v3_upgrade_preserves_v2_maps_and_joins_exact_retry_only`, and
  `strict_v3_reopen_rejects_successor_lineage_drift` pass on the terminal subject.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
