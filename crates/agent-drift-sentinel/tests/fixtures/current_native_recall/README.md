# P7 current-native recall fixtures

This directory is the committed, sanitized authority for SFR-RB-100.

`matrix.json` declares exactly nineteen planned cases. A case directory is added atomically with the
task that implements that case, and the corresponding matrix entry changes `implemented` from
`false` to `true`. The harness rejects missing, duplicate, unknown, extra, premature, and undeclared
case inventory.

Every failure identifies:

- the `case_id`;
- the terminal production boundary reached by the case; and
- the diagnostic owner responsible for the assertion.

The adapter classes are distinct:

- `Legacy` and `CurrentNativeV2` describe repository raw-rollout adapters;
- `BundleV0_2` describes the compactor-to-analyzer schema; and
- `PublicLive` describes a checked Sentinel event boundary.

No file in this tree may contain a real session, turn, call, event, agent, repository, username,
home-directory path, or credential. P7-1.2 adds executable recursive privacy and provenance checks
before substantive case fixtures are committed.

All nineteen cases also participate in one canonical whole-wall determinism test. The first wall
uses fresh temporary roots, forward source creation order, and cold closure state. The second uses
different temporary roots, reverse source creation order, and warm closure state for every
raw-rollout case. Bundle-v0.2 and public-live cases rerun their exact checked input boundaries.

The compared projection deliberately excludes environmental values that the P7 contract does not
own: generated timestamps, temporary roots, absolute source/output paths, and numeric source-file or
turn-reference allocation. Stable session and turn identities, row order/content/hashes, typed
dedupe identity, delegation state/evidence locations, analyzer semantics, and exact boundary errors
remain in the byte-for-byte comparison. P7-19 separately retains the stronger typed-segment
type/order/call-identity stress assertion.
