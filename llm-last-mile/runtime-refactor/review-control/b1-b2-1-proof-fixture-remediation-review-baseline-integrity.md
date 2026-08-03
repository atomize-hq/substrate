# B1/B2.1 baseline integrity review

Subject fingerprint:
`sha256:c4027fca095cfb6004ef15341f9eb565d6cbe3e527f75254f099f1cb8b0f673e`

Evidence reviewed:

- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-proof-fixture-remediation-final/20260803T171725Z/proof-summary.json`
- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-proof-fixture-remediation-final/20260803T171725Z/parallel/failure-names.txt`
- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-proof-fixture-remediation-final/20260803T171725Z/parallel/normalized-signatures.txt`
- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-proof-fixture-remediation-final/20260803T171725Z/serial/failure-names.txt`
- `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-proof-fixture-remediation-final/20260803T171725Z/serial/normalized-signatures.txt`

Read-only findings:

- The final parallel wall reproduced `1322 discovered / 1274 passed / 48 failed / 0 ignored` with failure-name SHA-256 `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`.
- The final serial wall reproduced the exact same counts and hashes.
- Both proofs preserved the three separately classified world-deps/report failures and reported no missing known-extra names.
- No Rust file changed, so there is no fixture assertion weakening, ignored-test expansion, or production-semantic drift in scope for this increment.

Findings:

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Verdict: `CLEAN`.
