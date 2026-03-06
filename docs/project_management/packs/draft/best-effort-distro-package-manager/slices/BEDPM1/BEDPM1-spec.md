# BEDPM1-spec — Deterministic package-manager selection + precedence + failures

## Behavior delta (single)
- Existing: `scripts/substrate/install-substrate.sh` selects a Linux host package manager via legacy override (`PKG_MANAGER`) and/or `PATH` probing; BEDPM0 adds distro parsing + a decision one-liner, but does not add `/etc/os-release` family mapping, `--pkg-manager` flag override, or contract-pinned precedence/warnings/exit codes.
- New: On Linux, the installer implements the full contract-defined precedence pipeline: `--pkg-manager` (source=`flag`) → `PKG_MANAGER` (source=`env`) → `/etc/os-release` mapping (source=`os_release`) → deterministic `PATH` probe fallback (source=`path_probe`), with fail-closed override validation (exit `2|3`), deterministic ambiguity handling, and exit-`4` remediation guidance.
- Why: Make manager selection deterministic, operator-controllable, and diagnosable, with explicit failures and guidance.

## Scope
- Add `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` flag parsing and validation.
- Implement the Linux-only precedence pipeline and ensure the decision one-liner `source` enum value is correct for every selection path.
- Implement `/etc/os-release` family mapping, including match rules and tie-break behavior.
- Implement deterministic `PATH` probe fallback with fixed precedence order, ambiguity warnings, and redaction invariants (no full `PATH` output).
- Implement failure posture for invalid override values (exit `2`), forced-manager missing in `PATH` (exit `3`), and no selectable supported manager (exit `4`), including required stderr content elements.

## Inputs (authoritative)
- Operator-facing contract (precedence, mapping table, warnings, one-liner template, exit codes): `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Deterministic parsing + PATH-probe policy decisions (DR-0001/DR-0002): `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
- Feature intent + locked vocabularies: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## Behavior (authoritative)

### Precedence pipeline (Linux only)
- The installer MUST select a supported package manager using this total order (from `contract.md`):
  1) `--pkg-manager …` (source=`flag`)
  2) `PKG_MANAGER=…` (source=`env`)
  3) `/etc/os-release` mapping (source=`os_release`)
  4) Deterministic `PATH` probe fallback (source=`path_probe`)
- When `--pkg-manager` or `PKG_MANAGER` is present, the installer MUST NOT fall back to later steps.

### Override validation + failures (exit `2` / exit `3`)
- Supported package-manager set is exact: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- If `--pkg-manager` or `PKG_MANAGER` is not in the supported set, the installer MUST:
  - exit `2`, and
  - print stderr that includes the invalid value, the allowed set, and override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`).
- If `--pkg-manager` or `PKG_MANAGER` forces a supported manager but the binary is not found in `PATH`, the installer MUST:
  - exit `3`, and
  - print stderr that includes the forced manager name, a statement that no fallback is performed for explicit overrides, and override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`).

### `/etc/os-release` mapping (Linux only)
- Mapping evaluation MUST follow the family match rules and tie-break rules in `contract.md`:
  - If `ID` matches one or more families, select the first matching family in mapping-table order and do not consult `ID_LIKE`.
  - Otherwise, select the first matching family in mapping-table order using `ID_LIKE` token matches.
- Mapping availability checks and fallback semantics are contract-owned:
  - Debian/Ubuntu, Arch, SUSE: if the mapped manager binary is missing from `PATH`, emit a warning and fall back to deterministic `PATH` probe selection (source=`path_probe`).
  - Fedora/RHEL: prefer `dnf`, fall back to `yum` when applicable; if neither is present in `PATH`, emit the mapping warning (mapped manager name `dnf`) and fall back to deterministic `PATH` probe selection (source=`path_probe`).
- Mapping fallback warnings MUST include the required content elements from `contract.md` and MUST NOT print the full `PATH` value.

### Deterministic `PATH` probe fallback (Linux only)
- Probe set is exact: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- “Found” definition: `command -v <manager>` succeeds.
- Selection MUST use this fixed precedence order:
  1) `apt-get`
  2) `dnf`
  3) `yum`
  4) `pacman`
  5) `zypper`
- If multiple supported managers are found, the installer MUST:
  - select the first by fixed precedence, and
  - emit a warning that includes the chosen manager, the other found supported managers (names only), and override guidance (`--pkg-manager …` and `PKG_MANAGER=…`),
  - without printing the full `PATH` value.
- If zero supported managers are found, the installer MUST exit `4` and print remediation guidance per `contract.md`.

### Required output invariants
- The decision one-liner shape and timing is contract-owned (`contract.md`) and MUST remain exact.
- On exit `2`, exit `3`, or exit `4` paths, the installer MUST NOT emit the decision one-liner, because no supported manager has been selected and validated as available in `PATH`.

## Acceptance criteria
- AC-BEDPM1-01: Override precedence: with `--pkg-manager yum` and `PKG_MANAGER=apt-get` and both binaries present in `PATH`, the installer selects `yum`, emits the decision one-liner with `source: flag`, and does not consult env, mapping, or `PATH` probe for selection; with no `--pkg-manager` and `PKG_MANAGER=pacman` and `pacman` present in `PATH`, the installer selects `pacman` and emits the decision one-liner with `source: env`.
- AC-BEDPM1-02: Invalid override value: when `--pkg-manager` or `PKG_MANAGER` is set to a value not in the supported set, the installer exits `2`, prints stderr that includes the invalid value, the allowed set, and override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`), performs no fallback selection, and does not emit the decision one-liner.
- AC-BEDPM1-03: Forced manager missing: when `--pkg-manager <supported>` or `PKG_MANAGER=<supported>` forces a supported manager that is absent from `PATH`, the installer exits `3`, prints stderr that includes the forced manager name, a statement that no fallback is performed for explicit overrides, and override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`), and does not emit the decision one-liner.
- AC-BEDPM1-04: Mapping match tie-break: given an os-release fixture `ID=arch` and `ID_LIKE=\"debian\"` with `pacman` present in `PATH`, the installer selects `pacman` with `source: os_release` (ID match wins); given a fixture with no `ID` line and `ID_LIKE=\"rhel debian\"` with `apt-get` present in `PATH`, the installer selects `apt-get` with `source: os_release` (mapping-table order takes precedence over `ID_LIKE` token order).
- AC-BEDPM1-05: Fedora/RHEL mapping: given an os-release fixture `ID=fedora` with `dnf` present in `PATH`, the installer selects `dnf` with `source: os_release`; given the same fixture with `dnf` absent and `yum` present, the installer selects `yum` with `source: os_release`; given the same fixture with both `dnf` and `yum` absent, the installer emits the mapping warning (mapped manager name `dnf`) and falls back to deterministic `PATH` probe selection with `source: path_probe`.
- AC-BEDPM1-06: Mapping binary missing (non-Fedora families): given an os-release fixture `ID=ubuntu` with `apt-get` absent from `PATH` and `dnf` present, the installer emits a mapping warning that includes `apt-get`, a statement that the binary is not present in `PATH`, a statement that `PATH` probe fallback is being used, and override guidance (`--pkg-manager …` and `PKG_MANAGER=…`); selection uses deterministic `PATH` probe and the decision one-liner uses `source: path_probe` (warning does not print the full `PATH` value).
- AC-BEDPM1-07: Deterministic `PATH` probe ambiguity policy: given no overrides, an unreadable os-release input (so mapping is skipped), and `PATH` containing both `yum` and `dnf`, the installer selects `dnf` with `source: path_probe` and emits a warning that includes the chosen manager `dnf`, the other found manager `yum` (names only), and override guidance (`--pkg-manager …` and `PKG_MANAGER=…`), without printing the full `PATH` value.
- AC-BEDPM1-08: No supported manager: given no overrides, an unreadable os-release input, and zero supported managers found in `PATH`, the installer exits `4` and prints remediation guidance to stderr that includes override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`) and a manual prerequisite command list (command names only; no full `PATH` value and no full environment), and it does not emit the decision one-liner.

## Out of scope
- Changing `/etc/os-release` parsing and normalization rules (BEDPM0 owns parsing/normalization; DR-0001).
- Adding the hermetic end-to-end detection harness that asserts the full matrix (BEDPM2).
- Expanding the supported package-manager set, mapping table, or decision one-liner template beyond `contract.md`.
- Persisting host distro or manager metadata (cross-pack boundary).
