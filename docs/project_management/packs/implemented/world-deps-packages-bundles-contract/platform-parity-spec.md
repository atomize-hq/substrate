# world-deps-packages-bundles-contract — platform parity spec

Owner standard:

- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

This spec is authoritative for platform guarantees and permitted divergences for `substrate world deps` (ADR-0011).

## Required platforms

- Behavior platforms (smoke required): `linux, macos`
- CI parity platforms (parity required): `linux, macos`
- WSL required: `true` (bundled into Linux smoke via `RUN_WSL=1`)

## Guarantees (explicit)

- The CLI contract in `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md` is identical across platforms.
- World-backed operations (`current list applied`, `current show --explain`, `current install`, `current sync`) fail closed with exit `3` when the world backend is unavailable.
- Inventory/enabled operations (`current list available`, `current list enabled`, `global/workspace list available|enabled`, `global/workspace add|remove|reset`) do not require world backend access and remain available when world is disabled.

## Permitted divergences (explicit)

- Backend remediation steps in error messages may be platform-specific:
  - macOS remediation references Lima warm/doctor scripts.
  - WSL remediation may reference WSL warm/doctor scripts where applicable.

## Validation evidence requirements (authoritative)

- Smoke scripts under `docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/` encode these platform guarantees.
- For checkpoint-boundary slices (`WDP2`, `WDP5`), the smoke workflow sets:
  - `SUBSTRATE_SMOKE_SLICE_ID=WDP2` or `WDP5`
- Each smoke script:
  - exits `0` only when the slice’s required behaviors are satisfied for that platform, and
  - asserts exit codes and one or more key message substrings for unsupported/error paths.
