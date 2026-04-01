# WDRA2-spec — regression coverage and docs alignment

## Behavior delta (single)
- Existing: replay has partial tests and operator docs around world selection, but there is no locked end-to-end contract that covers effective-disable attribution, redacted path displays, telemetry omission rules, and cross-platform smoke/manual evidence.
- New: replay regression tests, operator docs, smoke wrappers, and manual playbook content lock the final attribution contract for override env, workspace config, global config, and replay-local opt-out cases across Linux, macOS, and Windows validation surfaces.
- Why: keep the shipped attribution contract stable, reviewable, and verifiable across supported platforms.

## Scope
- Extend `crates/shell/tests/replay_world.rs` for override env, workspace config, global config, unknown-source redaction, and replay-local opt-out omission rules.
- Align `docs/REPLAY.md`, `docs/TRACE.md`, and `docs/COMMANDS.md` with the final contract and telemetry fields.
- Wire manual playbook and smoke wrappers to the same expected test filters and assertions.

Likely touch surfaces (non-authoritative):
- `crates/shell/tests/replay_world.rs`
- `docs/REPLAY.md`
- `docs/TRACE.md`
- `docs/COMMANDS.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/manual_testing_playbook.md`

## Behavior (authoritative)

### Final contract lock-in
- Replay docs describe the final reason fragments and final telemetry field names.
- Replay tests assert highest-precedence attribution, tokenized path displays, and omission rules.
- Smoke wrappers run the planned replay tests for the selected slice id.
- Manual testing steps use the same expected reason fragments and field names as the docs and regression tests.

## Acceptance criteria
- AC-WDRA2-01: replay tests cover override env, workspace config, global config, and unknown-source redaction paths.
- AC-WDRA2-02: replay tests prove replay-local opt-out cases keep their existing fragments and omit `world_disable_source`.
- AC-WDRA2-03: `docs/REPLAY.md`, `docs/TRACE.md`, and `docs/COMMANDS.md` match `contract.md` and `telemetry-spec.md`.
- AC-WDRA2-04: manual playbook and Linux, macOS, and Windows smoke wrappers reference the same test filters and expected assertions.

## Out of scope
- new replay selection semantics
- new trace event types
- changes to ADR-0037 semantics
