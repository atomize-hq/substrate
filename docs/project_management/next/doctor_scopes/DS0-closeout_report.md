# Slice Closeout Gate Report — doctor_scopes / DS0

Date (UTC): 2026-01-08

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:
- `docs/project_management/next/doctor_scopes/`

Slice spec:
- `docs/project_management/next/doctor_scopes/DS0-spec.md`

## Status

NOT RUN.

This file MUST be completed as part of the `DS0-integ` task. Leaving it in this “NOT RUN” state after `DS0-integ` is a hard gate failure.

## Behavior Delta (Existing → New → Why)

- Existing behavior: `substrate world doctor` is host-oriented on macOS and mixes host/world facts on Linux without an explicit scope split.
- New behavior: introduce `substrate host doctor`; redefine `substrate world doctor` to include `host` + `world` blocks with world facts sourced from the world-agent endpoint `GET /v1/doctor/world`.
- Why: operators need a single, authoritative answer for “is isolation enforceable right now?” across Linux and macOS, including guest-kernel facts on macOS.
- Links:
  - `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`
  - `docs/project_management/next/doctor_scopes/DS0-spec.md`

## Spec Parity (No Drift)

- Acceptance criteria satisfied: `NOT YET VERIFIED`
- Spec changes during the slice: `NONE EXPECTED (spec is authoritative; drift must be justified in writing if it occurs)`

## Checks Run (Evidence)

- `cargo fmt`: `NOT RUN`
- `cargo clippy --workspace --all-targets -- -D warnings`: `NOT RUN`
- Relevant tests: `NOT RUN`
- `make integ-checks`: `NOT RUN`

## Cross-Platform Smoke

Record run ids/URLs for required platforms:
- CI compile parity (linux/macos/windows): `NOT RUN`
- Linux smoke: `NOT RUN`
- macOS smoke: `NOT RUN`

Platform-fix work summary (must be explicit; use `NONE` if nothing was needed):
- What failed: `NOT RUN`
- What was changed: `NOT RUN`
- Why the change is safe (guards, cfg, feature flags): `NOT RUN`

## Smoke ↔ Manual Parity

- Smoke scripts mirror manual playbook: `NOT YET VERIFIED`
- Smoke scripts validate exit codes + key output: `NOT YET VERIFIED`
