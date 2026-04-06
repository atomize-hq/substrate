**Warning: Pre-Planning Only. This draft exists only to align downstream planning for `best-effort-distro-package-manager` and will be deleted or retired during full planning.**

# best-effort-distro-package-manager — minimal spec draft

## Scope + authority

This draft defines only pack-level defaults, precedence rules, naming, and cross-slice invariants for ADR-0031 planning.

This draft does not define slice-specific acceptance criteria, detailed schemas, implementation task wiring, or full user-facing contract tables.

Until full planning creates `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`, downstream pre-planning artifacts for this pack must treat these inputs as the only authoritative inputs:
- `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md`

During full planning, authority splits as follows:
- `contract.md` owns CLI, env-var, exit-code, path, and platform contract text.
- `slices/BEDPM*/BEDPM*-spec.md` owns slice-local behavior and acceptance.
- `tasks.json` owns execution wiring only after slice specs exist.

## Defaults + precedence

- Installer entrypoint: `scripts/substrate/install-substrate.sh`
- Wrapper boundary: `scripts/substrate/install.sh` preserves the upstream installer exit status for this feature.
- Selection precedence is fixed:
  1. `--pkg-manager`
  2. `PKG_MANAGER`
  3. `/etc/os-release` mapping
  4. `PATH` probe in this fixed order: `apt-get -> dnf -> yum -> pacman -> zypper`
- No config file or persistent config key exists in scope for this feature.
- Detection reads `/etc/os-release` only from the installer path and only for `ID` and `ID_LIKE`.
- Hermetic alternate os-release input uses one installer-local env var: `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.

## Failure posture + invariants

- Source selection is fail-open only between detection stages: unreadable or missing `/etc/os-release` renders `<unknown>` fields and falls through to `PATH` probing.
- Explicit override validation is fail-closed:
  - invalid `--pkg-manager` exits `2`
  - invalid `PKG_MANAGER` exits `2`
  - forced manager missing from `PATH` exits `3`
  - no supported manager selected exits `4`
- Detection is Linux-only, local-only, and performs no network call.
- `/etc/os-release` parsing uses safe line parsing and does not execute shell code.
- Operator-visible reporting stays limited to the stable decision line and actionable remediation text. This draft introduces no raw env dump, no raw os-release dump, and no new telemetry surface.

## Exit-code posture

- Exit-code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work does not require new exit codes.
- The required feature-specific classes remain `0`, `2`, `3`, and `4`.
- `1` and `5` retain the shared taxonomy meanings.
- `scripts/substrate/install.sh` and `scripts/substrate/install-substrate.sh` must expose the same failure class for this feature path.

## Cross-cutting seams / constraints

- Supported package-manager vocabulary is fixed: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- `pkg_manager.source` vocabulary is fixed: `flag`, `env`, `os_release`, `path_probe`.
- The stable stderr line is fixed:
  - `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)`
- The missing-data sentinel is fixed: `<unknown>`.
- Linux is the only behavior-changing platform in scope. macOS and Windows remain no-change platforms.
- Implementation touch stays inside installer shell scripts, installer docs, repo tests, and planning artifacts. No implementation change belongs under `crates/`, `src/`, `crates/world/`, `crates/world-mac-lima/`, `crates/world-windows-wsl/`, `crates/shim/`, `crates/shell/`, or `crates/world-agent/`.
- The exact repo test path for hermetic validation is fixed: `tests/installers/pkg_manager_detection_smoke.sh`.
- The feature-local smoke script `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh` is a thin wrapper over the repo test path above and does not define a second contract.
- `docs/INSTALLATION.md` and `docs/reference/env/contract.md` must reuse the same precedence chain, vocabularies, stderr line shape, exit classes, and remediation posture.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` consumes the detection vocabulary, `<unknown>` sentinel, and `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contract from this pack. It does not redefine them.

## Follow-ups for full planning

- Select DR-0001 and define the exact safe parser rule for extracting `ID` and `ID_LIKE`.
- Define the full `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contract in `contract.md` and `decision_register.md`: allowed values, path validation, precedence against real `/etc/os-release`, absence semantics, and failure behavior.
- Write the exact remediation text for exit `3` and exit `4` and reuse it across `contract.md`, `manual_testing_playbook.md`, `docs/INSTALLATION.md`, and `smoke/linux-smoke.sh`.
- Define the exact warning text for multi-manager `PATH` detection so docs, repo tests, and smoke evidence assert one string shape.
- Record the slice-local ownership split for `docs/INSTALLATION.md`, `docs/reference/env/contract.md`, and `scripts/substrate/install.sh` in the final slice specs.

## Draft slice skeleton (pre-planning only)

Slice prefix (draft): `BEDPM`

Draft; may split/merge; do not wire `tasks.json` yet.

CI-checkpoint must use this slice list as the default input when populating the machine-readable slices list in `pre-planning/ci_checkpoint_plan.md`. Mechanical validation stays disabled until slice tasks exist in `tasks.json`.

Workstream triage may propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md`. Workstream triage does not edit this file.

### `BEDPM0`

- `slice_id`: `BEDPM0`
- `name`: `Stabilize distro detection and reporting`
- `intent`: Stabilize best-effort distro detection, distro-family mapping, and the single stderr decision line. Lock the shared vocabulary and `<unknown>` sentinel used by all downstream docs and slices.
- `likely touch surfaces`: `scripts/substrate/install-substrate.sh`; `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`; `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`; `docs/INSTALLATION.md`

### `BEDPM1`

- `slice_id`: `BEDPM1`
- `name`: `Lock override precedence and failure classes`
- `intent`: Stabilize explicit override precedence, deterministic fallback order, wrapper exit-code pass-through, and the actionable failure contract for unsupported or unavailable managers.
- `likely touch surfaces`: `scripts/substrate/install-substrate.sh`; `scripts/substrate/install.sh`; `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md`; `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`; `docs/reference/env/contract.md`

### `BEDPM2`

- `slice_id`: `BEDPM2`
- `name`: `Prove hermetic detection behavior`
- `intent`: Stabilize the repo test path, feature-local smoke wrapper, and manual evidence path for precedence, mapping, warning, and remediation behavior without host mutation.
- `likely touch surfaces`: `tests/installers/pkg_manager_detection_smoke.sh`; `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh`; `docs/project_management/packs/implemented/best-effort-distro-package-manager/manual_testing_playbook.md`; `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`
