**Warning: Pre-Planning Only. Delete or retire this draft during full planning.**

# persist-macos-host-os-install-state minimal spec draft

## Scope + authority

This draft defines the pack-level alignment backbone only:
- cross-cutting defaults
- precedence
- invariants
- slice seams that every later spec must honor

This draft does not define:
- slice-local acceptance details
- full JSON schemas or examples
- implementation tasks
- command transcripts
- manual test procedures

Authority boundaries:
- `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md` defines the feature intent and user contract basis.
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` defines the required doc set, slice IDs, and authoritative doc ownership.
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md` defines the touch set and overlap boundaries.
- Full planning moves final user-facing contract text into `contract.md` and topic-specific spec docs named in `spec_manifest.md`.

## Defaults + precedence

- Feature-specific precedence resolves from installer invocation state first: the invocation decides `effective_prefix`, `--dry-run`, and `--no-world`, then the canonical metadata path resolves to `<effective_prefix>/install_state.json`.
- This draft adds no new CLI flag, config key, or environment variable.
- Canonical file name: `install_state.json`.
- Canonical file path: `<effective_prefix>/install_state.json`.
- Default hosted macOS path: `~/.substrate/install_state.json`.
- Source-of-truth alignment for later planning docs:
  - `contract.md` owns final user-facing path and write semantics.
  - `install-state-schema-spec.md` owns `host_state.os.*` field definitions.
  - `filesystem-semantics-spec.md` owns temp-file and replace ordering rules.
  - `platform-parity-spec.md` owns macOS/Linux/Windows guarantee boundaries.
  - `compatibility-spec.md` owns additive-only preservation rules.

## Failure posture + invariants

- Metadata persistence is fail-open relative to an otherwise successful hosted macOS install. Read, parse, collect, write, and replace failures emit warning-only diagnostics and do not convert install success into install failure.
- Sensitive-data scope is fail-closed. Persist only `host_state.os.family`, `host_state.os.product_version`, `host_state.os.build_version`, and `host_state.os.arch`.
- Hosted macOS install success writes or updates the canonical metadata file.
- Hosted macOS install success with `--no-world` writes or updates the same canonical metadata file.
- Hosted macOS `--dry-run` writes no metadata file and no temp file.
- `schema_version` remains integer `1`.
- Rewrites preserve unknown keys plus existing `host_state.group`, `host_state.linger`, and `host_state.platform` content when present.
- The writer uses same-directory temp-file plus replace ordering. In-place truncation stays forbidden.
- This pack stays inside the hosted installer path. `scripts/substrate/dev-install-substrate.sh` stays out of scope.
- This pack adds no new telemetry field, no new log field, no new redaction surface, and no new policy surface.

## Exit-code posture

- Canonical taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
- This feature requires no new exit code.
- Warning-only metadata degradation does not change success semantics for an otherwise successful install.
- Existing installer failure codes outside this metadata path remain unchanged.

## Cross-cutting seams / constraints

- The pack owns hosted macOS persistence only. Linux distro/package-manager persistence remains with prior Linux work. Windows gains no new persisted `host_state.os.*` surface in this pack.
- The field list stays fixed across later specs: `family`, `product_version`, `build_version`, `arch`.
- The source-command mapping stays aligned across later specs:
  - `family` = `macos`
  - `product_version` from `sw_vers -productVersion`
  - `build_version` from `sw_vers -buildVersion`
  - `arch` from `uname -m`
- The canonical temp path stays `<effective_prefix>/install_state.json.tmp`.
- The same shared writer path in `scripts/substrate/install-substrate.sh` remains the implementation seam. This pack does not introduce a second metadata file or a second hosted-installer writer.
- `docs/INSTALLATION.md` must describe macOS metadata persistence as diagnostic state only and keep Linux cleanup semantics separate.
- Later planning docs must preserve the manifest slice order: `PMHOIS0` -> `PMHOIS1` -> `PMHOIS2`.

## Follow-ups for full planning

- Close the partial-emission decision for `host_state.os` when one collection command fails, then align `contract.md`, `install-state-schema-spec.md`, and validation ownership to that result.
- Confirm whether the impact-map defaults for fresh-file scaffolding and primary validation harness move into `decision_register.md` as accepted decisions or remain open until the quality gate.
- Lock the exact validation ownership split between `tests/mac/installer_parity_fixture.sh` and `tests/installers/install_state_smoke.sh`.
- Replace the stale macOS sentence in `docs/INSTALLATION.md` and keep the Linux-only cleanup boundary explicit.
- Retire this document after full planning lands the authoritative contract and slice specs.

## Draft slice skeleton (pre-planning only)

Draft disclaimer: draft; split/merge remains allowed during full planning; do not wire `tasks.json` yet.

Slice prefix (draft): `PMHOIS`

Downstream note:
- CI-checkpoint uses this slice list first when populating `pre-planning/ci_checkpoint_plan.md`.
- Workstream triage records recommended edits to `pre-planning/workstream_triage.md` and does not edit this file.

### `PMHOIS0`

- `slice_id`: `PMHOIS0`
- `name`: Freeze contract backbone
- `intent`: Lock the cross-cutting contract boundary, file identity, compatibility posture, failure posture, and open decisions that every later spec reuses.
- `likely touch surfaces`:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md`

### `PMHOIS1`

- `slice_id`: `PMHOIS1`
- `name`: Implement hosted macOS persistence
- `intent`: Stabilize the hosted installer write/update path so successful macOS installs persist additive `host_state.os.*` metadata with warning-only degradation and atomic replace semantics.
- `likely touch surfaces`:
  - `scripts/substrate/install-substrate.sh`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS1/PMHOIS1-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`

### `PMHOIS2`

- `slice_id`: `PMHOIS2`
- `name`: Close validation and operator parity
- `intent`: Lock the automated evidence, manual evidence, and operator-doc conformance path for hosted macOS normal installs and hosted macOS `--no-world`.
- `likely touch surfaces`:
  - `tests/mac/installer_parity_fixture.sh`
  - `tests/installers/install_state_smoke.sh`
  - `docs/INSTALLATION.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/plan.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/PMHOIS2/PMHOIS2-spec.md`
