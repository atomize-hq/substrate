# best-effort-distro-package-manager — contract surface

This file is the single authoritative operator-facing contract for Linux host package-manager selection during install, per `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`.

Decision inputs:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` (DR-0001/2/3)
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md` (surface ownership)
- `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md` (User Contract)

External authoritative inputs (this feature does not redefine these surfaces):
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## In scope

- Script: `scripts/substrate/install-substrate.sh`
- Platforms:
  - Linux: behavior defined by this contract.
  - macOS/Windows: no behavior change introduced by this feature.

## Locked vocabularies

- Supported package-manager set (exact): `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- Decision one-liner `source` enum (exact): `flag`, `env`, `os_release`, `path_probe`.

## Inputs (operator-controlled)

### CLI flag override (highest precedence)

- Flag: `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`
- Meaning: force the package manager used to install Linux host prerequisites.
- Validation:
  - If the provided value is not in the supported set, the installer MUST exit `2`.
  - If the provided value is in the supported set but the binary is not found in `PATH`, the installer MUST exit `3`.
- Fallback behavior:
  - When `--pkg-manager` is provided, the installer MUST NOT fall back to any other selection method.

### Env override (second precedence; legacy)

- Env var: `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`
- Meaning: force the package manager used to install Linux host prerequisites.
- Validation:
  - If the provided value is not in the supported set, the installer MUST exit `2`.
  - If the provided value is in the supported set but the binary is not found in `PATH`, the installer MUST exit `3`.
- Fallback behavior:
  - When `PKG_MANAGER` is used, the installer MUST NOT fall back to autodetection.

### Hermetic test seam (os-release path override)

- Env var: `SUBSTRATE_INSTALL_OS_RELEASE_PATH=<path>`
- Default: unset (reads `/etc/os-release`).
- Meaning: override the path read for os-release parsing.
- Safety invariants:
  - The installer MUST read the configured path as plain text.
  - The installer MUST NOT `source`/eval/execute file content.

## Precedence pipeline (Linux only)

The installer MUST select a supported package manager using this total order:

1) `--pkg-manager …` (source=`flag`)
2) `PKG_MANAGER=…` (source=`env`)
3) `/etc/os-release` mapping (or `SUBSTRATE_INSTALL_OS_RELEASE_PATH`) (source=`os_release`)
4) Deterministic `PATH` probe fallback (source=`path_probe`)

If no supported manager can be selected, the installer MUST exit `4` and print remediation guidance.

## `/etc/os-release` parsing contract (DR-0001)

### File selection

- The installer MUST read `/etc/os-release` unless `SUBSTRATE_INSTALL_OS_RELEASE_PATH` is set.
- If the selected file path is missing or unreadable:
  - `distro_id` MUST be `<unknown>`.
  - `distro_id_like` MUST be `<unknown>`.

### Safe parsing posture

- The installer MUST treat the file as plain text (no shell evaluation).
- The installer MUST ignore:
  - empty lines, and
  - comment lines (after trimming leading whitespace) that start with `#`.

### Key/value extraction (deterministic)

- Only the keys `ID` and `ID_LIKE` are in scope.
- Key matching MUST be case-insensitive (normalize keys to uppercase before comparing).
- If a key occurs multiple times, the last occurrence MUST win.
- Values MUST be normalized as follows:
  - Trim leading/trailing ASCII whitespace.
  - If the value starts and ends with the same quote character (`"` or `'`), strip exactly one leading and one trailing quote.
  - Convert to lowercase for matching and rendering.
  - Collapse internal ASCII whitespace runs to a single space.
- Missing keys:
  - If `ID` is missing, `distro_id` MUST be `<unknown>`.
  - If `ID_LIKE` is missing, `distro_id_like` MUST be `<unknown>`.

### `ID_LIKE` tokenization

- `ID_LIKE` MUST be tokenized on ASCII whitespace into an ordered list of non-empty tokens.
- Token matching MUST be exact on tokens (not substring matching) unless the ADR explicitly requires substring matching for a family rule.

## Mapping table (Linux only)

Mapping runs only when at least one of `distro_id` or `distro_id_like` is not `<unknown>`.

### Family match rules (ADR-0031)

- Debian/Ubuntu family → `apt-get`
  - Match if `ID` in `{debian, ubuntu, linuxmint, pop}` OR `ID_LIKE` contains token `debian` or `ubuntu`.
- Fedora/RHEL family → prefer `dnf` (fallback `yum` if `dnf` missing)
  - Match if `ID` in `{fedora, rhel, centos, rocky, almalinux, ol, amzn}` OR `ID_LIKE` contains token `fedora` or `rhel`.
- Arch family → `pacman`
  - Match if `ID` in `{arch, manjaro, endeavouros, arcolinux, artix, garuda}` OR `ID_LIKE` contains token `arch`.
- SUSE family → `zypper`
  - Match if `ID` contains substring `suse` or `opensuse` OR `ID_LIKE` contains token `suse` or `opensuse`.

### Match precedence (tie-breaker)

- If `ID` matches one or more families, the installer MUST select the first matching family in the mapping-table order above, without consulting `ID_LIKE`.
- Otherwise, the installer MUST select the first matching family in the mapping-table order above using `ID_LIKE` token matches.

### Mapping availability checks + fallback semantics

- For Debian/Ubuntu, Arch, and SUSE families:
  - If the mapped manager binary is found in `PATH`, selection MUST succeed with source=`os_release`.
  - If the mapped manager binary is not found in `PATH`, the installer MUST fall back to the deterministic `PATH` probe (source=`path_probe`) and MUST emit a warning that includes:
    - the mapped manager name,
    - a statement that the binary is not present in `PATH`,
    - a statement that `PATH` probe fallback is being used, and
    - override guidance (`--pkg-manager …` and `PKG_MANAGER=…`).
- For Fedora/RHEL family:
  - If `dnf` is found in `PATH`, selection MUST use `dnf` with source=`os_release`.
  - Otherwise, if `yum` is found in `PATH`, selection MUST use `yum` with source=`os_release`.
  - Otherwise, selection MUST fall back to the deterministic `PATH` probe (source=`path_probe`) and MUST emit the warning described above (mapped manager name MUST be `dnf`).

## Deterministic `PATH` probe fallback (DR-0002)

### Probe set

- The probe set is exactly the supported package-manager set: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- A manager is considered “found” when `command -v <manager>` succeeds.

### Selection order

When `PATH` probe fallback runs, selection MUST use this fixed precedence order:

1) `apt-get`
2) `dnf`
3) `yum`
4) `pacman`
5) `zypper`

### Ambiguity handling

- If exactly one supported manager is found, it MUST be selected with source=`path_probe`.
- If multiple supported managers are found:
  - Selection MUST pick the first manager in the fixed precedence order above.
  - The installer MUST emit a warning that includes:
    - the chosen manager,
    - the list of the other found supported managers (names only), and
    - override guidance (`--pkg-manager …` and `PKG_MANAGER=…`).
- The warning MUST NOT print the full `PATH` value.

### No supported manager

If zero supported managers are found, the installer MUST exit `4` and MUST print remediation guidance (see below).

## Required installer output (Linux only)

### Decision one-liner (exact string; stderr; exactly once)

When a supported manager has been selected and validated as available in `PATH`, and before invoking any package-manager install commands, the installer MUST print exactly one line to stderr with this exact string template (no prefixes):

```text
Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)
```

Rendering rules:
- If the os-release file is missing/unreadable, `<id>` and `<id_like>` MUST be `<unknown>`.
- If `ID` is missing, `<id>` MUST be `<unknown>`.
- If `ID_LIKE` is missing, `<id_like>` MUST be `<unknown>`.
- `<pkg_manager>` MUST be one of the supported package-manager set.
- `<source>` MUST be one of the `source` enum values.

## Remediation guidance content (failure cases)

All remediation guidance MUST be printed to stderr.

### Exit `2` (invalid override value)

The error output MUST include:
- the invalid value,
- the allowed set, and
- override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`).

### Exit `3` (forced manager missing from `PATH`)

The error output MUST include:
- the forced manager name,
- a statement that no fallback is performed for explicit overrides, and
- override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`).

### Exit `4` (no supported manager selectable)

The error output MUST include:
- override usage guidance (`--pkg-manager …` and `PKG_MANAGER=…`), and
- a manual prerequisite command list (see next section).

## Linux prerequisite command universe (used in remediation)

The manual prerequisite command list printed for exit `4` MUST be derived from the installer’s prerequisite probes for the current invocation:

- Always-required commands (Linux): `curl`, `tar`, `jq`, and one of `sha256sum` or `shasum`.
- When world provisioning is enabled (default; equivalent to not passing `--no-world`): `systemctl`, `fuse-overlayfs`, `nft`, `ip`.

The guidance MUST list command names only and MUST NOT print the full environment or the full `PATH`.

## Exit codes (Linux installer flows only)

- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

Mapping for the Linux installer flow:
- `0`: success (including no-op by contract)
- `2`: invalid CLI usage or invalid override value
- `3`: required dependency unavailable (forced package manager missing from `PATH`)
- `4`: not supported / missing prerequisites (no supported package manager can be selected)

## Safety invariants (non-negotiable)

- Distro detection MUST perform no network calls.
- Distro detection MUST perform no writes.
- `/etc/os-release` (or the overridden path) MUST be parsed as plain text (no `source`/eval/execute).
- Outputs introduced by this feature MUST NOT print the full environment or the full `PATH`.

## Cross-pack authority boundary

- This pack owns distro detection, package-manager selection, and `source` semantics for Linux host installer behavior.
- This pack MUST NOT persist host metadata.
