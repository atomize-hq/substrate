# ADR-0038 — Replay Attributes Why World Is Disabled in Warnings

## Status

- Status: Implemented
- Original date (UTC): 2026-02-21
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Replay must use the same disabled-world attribution contract as doctor and health whenever replay
falls back to host due to world isolation being disabled.

The stable decision is:

- replay warning and origin messaging reuses the shared disabled-world attribution model
- replay preserves existing selection and exit behavior; only attribution becomes more accurate
- replay-facing structured reason fields stay aligned with doctor/health precedence and phrasing
- attribution remains redaction-safe and avoids leaking raw paths or secret values

## Stable Owned Surface

This ADR owns the stable replay disable-attribution contract documented in:

- `docs/REPLAY.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/shell/src/execution/routing/replay.rs`
- `crates/replay/src/replay/executor.rs`
- `crates/shell/src/execution/config_model.rs`
- `crates/shell/tests/replay_world.rs`
- `crates/shell/src/execution/routing/dispatch/tests/host_replay.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0037-doctor-health-attribute-why-world-is-disabled.md`

## Historical Note

The original ADR captured the consistency work needed to make replay warnings match other
world-disabled surfaces. The stable replay attribution contract now lives here and in the replay
docs.
