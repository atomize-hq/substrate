# BEDPM1-spec - Lock override precedence, fallback order, and failure classes

## Behavior delta (single)
- Existing: Linux hosted installs do not expose one contract-bearing precedence chain for `--pkg-manager`, `PKG_MANAGER`, os-release mapping, and deterministic `PATH` fallback, so override failures and multi-manager `PATH` cases stay under-specified at the feature boundary.
- New: `scripts/substrate/install-substrate.sh` applies the fixed precedence chain `--pkg-manager -> PKG_MANAGER -> os_release -> path_probe`, validates explicit selections fail-closed, probes supported managers in one fixed order, and emits the contract-owned multi-manager warning before the stable decision line when the `PATH` branch detects several supported managers.
- Why: operators need deterministic override behavior, deterministic fallback selection, and explicit failure classes before wrapper and documentation propagation can reuse the contract without drift.

## Scope
- Constrain Linux hosted-installer behavior for `--pkg-manager`, `PKG_MANAGER`, supported-manager availability checks, the fixed ordered `PATH` probe, the multi-manager warning posture, and exits `2`, `3`, and `4`.
- Keep detection parsing, distro-family mapping, and the base decision-line template linked to `contract.md`.
- Keep wrapper exit-status pass-through and operator or env-doc propagation outside this slice.

## Inputs (authoritative)
- Operator-facing precedence, warning, exit-code, and remediation contract: `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`
- Accepted warning and pass-through decisions: `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md` (`DR-0002`, `DR-0004`)
- Slice boundary and required assertions: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- Implementation surfaces constrained by this slice:
  - `scripts/substrate/install-substrate.sh`

## Behavior (authoritative)
### Override precedence and fail-closed validation
- This slice inherits the exact supported manager vocabulary, exit-code table, and remediation content from `contract.md` and does not restate them as a second authority.
- `--pkg-manager` is the highest-precedence selector for Linux hosted installs.
- `PKG_MANAGER` is the next selector and applies only when `--pkg-manager` is absent.
- A valid explicit selector never falls through to os-release mapping or `PATH` probing.
- An invalid `--pkg-manager` value exits with code `2`.
- An invalid `PKG_MANAGER` value exits with code `2`.
- A valid manager chosen by `--pkg-manager` or `PKG_MANAGER` that is missing from `PATH` exits with code `3`.

### Ordered `PATH` probing and ambiguity handling
- When neither explicit selector makes a choice and the earlier os-release mapping stage produces no available manager, the installer probes supported managers in this fixed order: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- When exactly one supported manager is present in `PATH`, the installer selects that manager with `pkg_manager.source=path_probe` and emits no warning.
- When more than one supported manager is present in `PATH`, the installer selects the earliest detected manager from the same fixed order, sets `pkg_manager.source=path_probe`, emits the contract-owned warning line exactly once to stderr, and emits that warning before the stable decision line.
- When no supported manager is selected after os-release mapping and ordered `PATH` probing, the installer exits with code `4`.
- This slice does not redefine the stable decision-line template from `contract.md`; it constrains only the branches that reach or suppress that line.

## Acceptance criteria
- AC-BEDPM1-01: When both `--pkg-manager pacman` and `PKG_MANAGER=apt-get` are present for a Linux hosted install and `pacman` exists in `PATH`, the installer selects `pacman`, records `pkg_manager.source=flag`, and does not consult the lower-precedence env, os-release, or `PATH` branches.
- AC-BEDPM1-02: When `--pkg-manager` is absent, `PKG_MANAGER=dnf` is present, and `dnf` exists in `PATH`, the installer selects `dnf`, records `pkg_manager.source=env`, and does not consult os-release mapping or the ordered `PATH` probe.
- AC-BEDPM1-03: An invalid `--pkg-manager` value and an invalid `PKG_MANAGER` value each exit with code `2`, name the offending source, name the invalid value, and list the exact allowed manager spellings required by `contract.md`.
- AC-BEDPM1-04: A valid manager selected by `--pkg-manager` or `PKG_MANAGER` that is absent from `PATH` exits with code `3`, names the selected manager, names the selecting source, and tells the operator to install that manager or rerun with another allowed manager.
- AC-BEDPM1-05: When no supported manager is selected after the os-release stage and the ordered `PATH` probe finds none of `apt-get`, `dnf`, `yum`, `pacman`, or `zypper`, the installer exits with code `4` and emits the required remediation elements from `contract.md`, including the explicit override forms.
- AC-BEDPM1-06: When the ordered `PATH` probe detects more than one supported manager, the installer emits the exact warning line from `contract.md` once to stderr, lists detected managers in fixed probe-order order, selects the earliest detected manager from that same order, records `pkg_manager.source=path_probe`, and places the warning before the stable decision line.

## Out of scope
- Safe os-release parsing, normalized distro fields, `<unknown>` sentinel ownership, distro-family mapping, and the base decision-line timing contract remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`.
- Wrapper exit-status pass-through in `scripts/substrate/install.sh` and propagation into `docs/INSTALLATION.md` and `docs/reference/env/contract.md` remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`.
- Hermetic smoke coverage, the thin Linux smoke wrapper, and manual evidence capture remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`.
