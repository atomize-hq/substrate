# BEDPM3-spec — Hermetic validation, thin smoke alignment, and manual evidence

## Behavior delta (single)
- Existing: the validation lane for ADR-0031 has no authored slice that binds the repo harness path, the feature-local smoke wrapper, and the operator rerun workflow to one contract.
- New: `tests/installers/pkg_manager_detection_smoke.sh` is the authoritative hermetic harness for Linux package-manager detection and override behavior, `smoke/linux-smoke.sh` remains a thin wrapper over that harness, and `manual_testing_playbook.md` captures the same precedence, warning, remediation, and wrapper-pass-through evidence.
- Why: one validation authority plus one thin wrapper keeps the feature contract testable without allowing smoke or playbook drift.

## Scope
- Constrain repo-level validation to `tests/installers/pkg_manager_detection_smoke.sh`.
- Constrain feature-local Linux smoke execution to `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`.
- Constrain operator rerun evidence to `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`.
- Prove precedence, source vocabulary, `<unknown>` fallback, remediation branches, and wrapper pass-through against `scripts/substrate/install-substrate.sh` and `scripts/substrate/install.sh`.
- Keep installer implementation changes and downstream doc propagation outside this slice.

## Inputs (authoritative)
- Operator-facing detection, warning, decision-line, remediation, and wrapper contract: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Accepted parser, ambiguity, alternate-input, wrapper, and smoke-topology decisions: `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` (`DR-0001`, `DR-0002`, `DR-0003`, `DR-0004`, `DR-0005`)
- Required validation ownership map: `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- Implementation and validation surfaces constrained by this slice:
  - `tests/installers/pkg_manager_detection_smoke.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/install.sh`

## Behavior (authoritative)
### Authoritative harness topology
- `tests/installers/pkg_manager_detection_smoke.sh` is the exact repo test path for this slice.
- The repo harness owns fake `PATH` construction with stub binaries for `apt-get`, `dnf`, `yum`, `pacman`, and `zypper`.
- The repo harness owns fake os-release input through `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.
- The repo harness captures stderr and exit status from the direct installer path and the wrapper path for the feature-specific branches covered by ADR-0031.
- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh` invokes that repo harness and returns the same pass or fail result instead of adding a second assertion set.
- Any local container smoke or ad hoc wrapper beyond `smoke/linux-smoke.sh` is non-gating for this feature pack.

### Hermetic assertions
- The repo harness proves the precedence order `--pkg-manager -> PKG_MANAGER -> os_release -> path_probe`.
- The repo harness asserts the fixed `pkg_manager.source` vocabulary `flag`, `env`, `os_release`, and `path_probe`.
- The repo harness asserts the stable decision line reuses the selected manager and source vocabulary from `contract.md`.
- The repo harness asserts that missing or invalid alternate os-release input yields `<unknown>` distro fields and still reaches the remaining selection stages.
- The repo harness asserts the exact multi-manager warning line from `contract.md` when more than one supported manager is present in `PATH` and no earlier stage selects a manager.
- The repo harness asserts exit `2` for invalid explicit overrides, exit `3` for explicit managers absent from `PATH`, and exit `4` when no supported manager is selected after os-release mapping and `PATH` probing.
- For exits `3` and `4`, the repo harness checks the required remediation elements from `contract.md`.
- The repo harness asserts that `scripts/substrate/install.sh` preserves direct-installer exits `0`, `2`, `3`, and `4` for the feature-specific branches covered by ADR-0031.

### Manual evidence workflow
- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` uses deterministic temp directories, stubbed manager availability, and alternate os-release files to rerun the validation contract without mutating `/etc/os-release`.
- The manual playbook defines exact commands and expected stderr for Debian-family selection, Arch-family selection, `--pkg-manager` override, `PKG_MANAGER` override, and one actionable failure case on the documented wrapper path.
- The manual playbook names `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh` as the non-interactive Linux rerun entrypoint.
- The manual playbook records macOS and Windows as explicit no-change platforms under ADR-0031.

## Acceptance criteria
- AC-BEDPM3-01: `tests/installers/pkg_manager_detection_smoke.sh` is the only authoritative hermetic harness path for this slice and it drives Linux detection cases through stub `PATH` binaries plus `SUBSTRATE_INSTALL_OS_RELEASE_PATH` without mutating `/etc/os-release`.
- AC-BEDPM3-02: The repo harness proves the precedence chain `--pkg-manager -> PKG_MANAGER -> os_release -> path_probe` by covering at least one case for each stage and asserting the decision line selects the expected manager and `pkg_manager.source` value from `contract.md`.
- AC-BEDPM3-03: The repo harness proves the missing-os-release and invalid-alternate-path branches by forcing `<unknown>` distro fields and verifying fallback selection still proceeds through the fixed `PATH` probe order when no earlier stage selects a manager.
- AC-BEDPM3-04: The repo harness proves the selected multi-manager posture by stubbing more than one supported manager in `PATH`, asserting the exact warning line from `contract.md`, and asserting the earliest detected manager in the fixed order is selected with `pkg_manager.source=path_probe`.
- AC-BEDPM3-05: The repo harness proves the explicit failure branches by asserting exit `2` for invalid `--pkg-manager` or invalid `PKG_MANAGER`, exit `3` when an explicit manager is absent from `PATH`, exit `4` when no supported manager is selected, and the required remediation elements for exits `3` and `4`.
- AC-BEDPM3-06: The repo harness exercises both `scripts/substrate/install-substrate.sh` and `scripts/substrate/install.sh` and proves the wrapper preserves the same feature-specific exit classes `0`, `2`, `3`, and `4`.
- AC-BEDPM3-07: `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh` remains a thin wrapper that invokes `tests/installers/pkg_manager_detection_smoke.sh`, adds no second behavior contract, and returns the harness exit status.
- AC-BEDPM3-08: `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` defines exact Linux commands, expected key stderr text, and expected exit codes for default Debian-family selection, default Arch-family selection, `--pkg-manager` override, `PKG_MANAGER` override, and one actionable failure case, and it states that macOS and Windows have no behavior delta under ADR-0031.

## Out of scope
- Distro parsing, family mapping, `<unknown>` sentinel ownership, and decision-line timing remain owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`.
- Explicit override precedence, fixed `PATH` probing order, multi-manager warnings, and exits `2`, `3`, and `4` remain owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`.
- Wrapper implementation changes and propagation into `docs/INSTALLATION.md` and `docs/reference/env/contract.md` remain owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`.
