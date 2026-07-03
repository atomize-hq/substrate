# R6-3.3 Semantic Goal Drift Acceptance Fixtures

This directory freezes the bounded semantic-goal-drift acceptance corpus for Packets `R6-2.4` and `R6-3.3`.

Included cases:

- `synthetic-kickoff-anchor-unauthorized-pivot`
- `synthetic-rolling-mid-session-pivot`

Maintenance rules:

- Each committed case must contain exactly `raw.json` and `expected.json`.
- Tests must fail closed if a checked-in case is missing; they must not read from `target/`, `.codex`, or
  runtime-generated analyzer output at test time.
- `raw.json` carries the compact rollout rows, session id, and notes needed to rebuild a
  contract-valid bundle and drive the live `analyze_bundle -> checkpoint_analyses -> score_session`
  path at test time.
- `expected.json` is the packet-local contract for the kickoff target, the final pivoted target,
  any penultimate rolling anchor that must stay stable, the final `semantic_goal_drift` posture,
  and the required/forbidden evidence prefixes.
- Corpus changes require explicit packet authority; this directory is intentionally bounded to the
  existing kickoff-anchored witness plus the one abrupt mid-session rolling pivot admitted by
  `R6-3.3`.
