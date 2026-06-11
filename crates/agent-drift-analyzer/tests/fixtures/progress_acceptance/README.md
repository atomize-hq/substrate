# Packet R5 Progress Acceptance Fixtures

This directory is the dedicated Packet `R5-7` semantic wall for `session_progress`.
It is separate from the frozen Packet `R2` acceptance wall under
`crates/agent-drift-analyzer/tests/fixtures/acceptance/**`.

Included cases:

- `019e899c-453f-71f2-a99d-155848c7b081` — annotated real-rollout troubleshooting advancement.
- `019e940c-a91b-7fe0-a967-b0bdd595b581` — annotated real-rollout planning meander/stall.
- `019e8b42-42bd-7b10-baae-3265edb65f4b` — annotated real-rollout verification closeout narrowing.
- `synthetic-planning-advancing` — committed bundle-shaped planning narrowing proof.
- `synthetic-implementation-advancing` — committed bundle-shaped implementation verification-wall proof.
- `synthetic-parent-visible-opaque` — committed delegated guardrail proof that limits progress claims to `parent_visible_orchestration`.

Excluded cases:

- `019e93f8-a5e9-7490-ac1a-955b74c92ad0` — delegated orchestration live fixture stayed outside this corpus because the current analyzer output did not deterministically surface `parent_visible_orchestration` on the copied real bundle.
- `019e9406-6736-79a2-946b-8a603e557422` — delegated orchestration bundle likewise stayed out for the same reason; R5 keeps delegated proof bounded to the explicit guardrail case instead of overclaiming child progress.
- `019e9401-9d69-7190-a43e-9ee3be08b369` — review-only bundle remained outside because it duplicated the planning/closeout shape already covered without adding a new core dimension.

Maintenance rules:

- Every case directory must contain exactly `manifest.json`, `rows.archival.jsonl`, `rows.compact.jsonl`, `dedupe-audit.jsonl`, and `expected.json`.
- Tests must fail closed if a committed case is missing; they must not read from `target/` or `~/.codex` at test time.
- The legacy R2 acceptance wall remains frozen and is verified separately by `acceptance_fixtures.rs`.
- Delegated cases are guardrail-only in R5: they may prove `parent_visible_orchestration` with limited confidence, but they must not claim positive opaque-child progress before R7.
- At least one case in this corpus must remain an `annotated_real_rollout` before R5 claims bounded semantic acceptance.
