# WDRA1-spec — replay stderr copy and telemetry wiring

## Behavior delta (single)
- Existing: replay host-origin summaries and `replay_strategy` trace output expose replay-local opt-out reasons, but effective-disable fallback cases do not yet share a fully aligned attribution contract for stderr copy and telemetry.
- New: replay origin summaries, host warnings, and additive `replay_strategy` fields consume the WDRA0 attribution seam so effective-disable fallback reports the exact contract fragments and source metadata defined by this pack.
- Why: make replay host fallback intelligible to operators and machine readers without changing replay selection precedence.

## Scope
- Wire effective-disable attribution into `[replay] origin: ...` output.
- Wire the same attribution into `[replay] warn: running on host (...)` output.
- Extend `replay_strategy` emission with the additive fields defined in `telemetry-spec.md`.
- Preserve existing replay-local opt-out fragments and omission rules.

Likely touch surfaces (non-authoritative):
- `crates/shell/src/execution/routing/replay.rs`
- `crates/replay/src/replay/executor.rs`
- `crates/shell/tests/replay_world.rs`
- `docs/TRACE.md`

## Behavior (authoritative)

### Replay-facing attribution output
- Replay-local opt-out fragments remain unchanged.
- Effective-disable attribution uses the exact fragments from `contract.md`.
- `origin_reason_code` extends with the `world_disabled_*` values from `telemetry-spec.md`.
- `world_disable_source` emits only for effective-disable attribution and is omitted for replay-local opt-out cases.
- The recorded-host case prints `host (recorded; <reason>)` exactly.

## Acceptance criteria
- AC-WDRA1-01: replay origin summaries use the exact effective-disable fragments from `contract.md`.
- AC-WDRA1-02: the recorded-host case prints `host (recorded; <reason>)` exactly.
- AC-WDRA1-03: replay host warnings use the same effective-disable fragment as the origin summary.
- AC-WDRA1-04: `replay_strategy` emits `origin_reason_code` and `world_disable_source` per `telemetry-spec.md`.
- AC-WDRA1-05: replay-local opt-out cases do not emit `world_disable_source`.

## Out of scope
- helper placement changes beyond the WDRA0 seam
- replay docs outside trace-field alignment
- smoke-script authoring
