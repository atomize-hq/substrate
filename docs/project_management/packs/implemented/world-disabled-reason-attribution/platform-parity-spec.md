# world-disabled-reason-attribution — platform parity spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope
- This spec is authoritative for platform guarantees and permitted divergences for ADR-0038.

## Required platforms
- Behavior platforms (smoke required): `linux`, `macos`, `windows`
- CI parity platforms (parity required): `linux`, `macos`, `windows`
- WSL required: `false`
- WSL task mode: `bundled`

## Guarantees
- Replay origin summaries and host warnings use the same reason fragments on Linux, macOS, and Windows.
- `replay_strategy` uses the same field names, enum values, and redaction rules on Linux, macOS, and Windows.
- Workspace and global config origins use the same tokenized display paths on Linux, macOS, and Windows.

## Permitted divergences
- Backend-specific transport warnings outside the reason fragment may differ by platform.
- Agent-socket details outside the effective-disable fragment may differ by platform.

## Known platform hazards
- Linux can exercise world-agent transport paths that do not exist on macOS or Windows.
- macOS and Windows may hit local-backend replay paths more often during smoke.
- The validation seam uses replay tests and feature-local smoke wrappers so attribution assertions stay backend-agnostic.

## Validation evidence
- Smoke scripts:
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/windows-smoke.ps1`
- CI parity gates:
  - compile parity at `CP1-ci-checkpoint`
  - feature smoke at `CP1-ci-checkpoint`
  - full CI testing at `CP1-ci-checkpoint`
- Manual playbook sections:
  - replay-local baseline
  - effective override env attribution
  - workspace and global config attribution
  - telemetry redaction

## Acceptance criteria
- Linux, macOS, and Windows pass the same smoke assertions for the selected slice id.
- Linux, macOS, and Windows emit the same `origin_reason_code` values for the same attribution case.
- Linux, macOS, and Windows emit tokenized `path_display` values only.
