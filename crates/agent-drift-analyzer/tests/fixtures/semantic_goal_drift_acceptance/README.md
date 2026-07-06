# R6-3.6 Semantic Goal Drift Acceptance Fixtures

This directory freezes the bounded semantic-goal-drift acceptance corpus for Packets `R6-2.4`, `R6-3.3`,
`R6-3.X.2`, and `R6-3.6`.

Included cases:

## Positive controls
- `synthetic-kickoff-anchor-unauthorized-pivot`
- `synthetic-kickoff-hyphen-collision-pivot`
- `synthetic-kickoff-objective-vs-objective-rs-pivot`
- `synthetic-rolling-mid-session-pivot`
- `synthetic-rolling-sibling-stem-pivot`

## Negative controls
- `synthetic-kickoff-line-suffix-same-file`
- `synthetic-kickoff-narrowing-into-anchored-subtree`
- `synthetic-rolling-directory-to-file-narrowing`
- `synthetic-rolling-file-to-containing-directory-broadening`
- `synthetic-rolling-map-review-to-map-verify-progression`

Maintenance rules:

- Each committed case must contain exactly `raw.json` and `expected.json`.
- Tests must fail closed if a checked-in case is missing; they must not read from `target/`, `.codex`, or
  runtime-generated analyzer output at test time.
- `raw.json` carries the compact rollout rows, session id, and notes needed to rebuild a contract-valid bundle
  and drive the live `analyze_bundle -> checkpoint_analyses -> score_session` path at test time.
- `expected.json` is the packet-local contract for the kickoff target, the final pivoted target, any
  penultimate rolling anchor that must stay stable, the final `semantic_goal_drift` posture, and the
  required/forbidden evidence prefixes.
- The corpus is intentionally curated and bounded; update the allowlist in
  `crates/agent-drift-analyzer/tests/semantic_goal_drift_acceptance.rs` in lockstep with directory changes.
