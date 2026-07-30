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
