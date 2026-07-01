# R6-2.4 Semantic Goal Drift Acceptance Fixtures

This directory freezes the kickoff-anchored semantic-goal-drift acceptance corpus for Packet `R6-2.4`.

Included cases:

- `synthetic-kickoff-anchor-unauthorized-pivot`

Maintenance rules:

- Each committed case must contain exactly `raw.json` and `expected.json`.
- Tests must fail closed if a checked-in case is missing; they must not read from `target/`, `.codex`, or
  runtime-generated analyzer output at test time.
- `raw.json` carries the compact rollout rows, session id, and notes needed to rebuild a
  contract-valid bundle and drive the live `analyze_bundle -> checkpoint_analyses -> score_session`
  path at test time.
- `expected.json` is the packet-local contract for the kickoff target, final pivoted target, final
  `semantic_goal_drift` posture, and the required kickoff-anchor/current-goal evidence prefixes.
- Corpus changes require explicit packet authority; this directory is intentionally bounded to the one
  kickoff-anchored unauthorized-pivot case admitted by `R6-2.4`.
