# BEDPM0-spec — Detect distro + emit installer decision one-liner

## Behavior delta (single)
- Existing: `scripts/substrate/install-substrate.sh` selects a Linux host package manager by probing `PATH` (and optionally using `PKG_MANAGER`), but does not report the distro inputs or a stable “what will we use” decision line.
- New: On Linux, the installer reads `/etc/os-release` (best-effort; plain-text parse) to derive normalized `distro_id` / `distro_id_like`, and prints exactly one stable decision one-liner to stderr before running any package-manager install commands. Selection semantics are unchanged in this slice (no `/etc/os-release`-based mapping yet).
- Why: Make the distro + package-manager decision observable and deterministic without expanding the selection surface area yet.

## Scope
- Implement best-effort os-release read + deterministic parsing for `ID` and `ID_LIKE` (no shell evaluation).
- Add a hermetic input seam `SUBSTRATE_INSTALL_OS_RELEASE_PATH` used only to select the file path read for parsing.
- Introduce normalized values:
  - `distro_id` (string; `<unknown>` allowed)
  - `distro_id_like` (string; `<unknown>` allowed)
- Emit the decision one-liner (exact template from `contract.md`) to stderr:
  - exactly once per installer invocation,
  - only after a supported package manager has been selected and verified present in `PATH`,
  - before any package-manager install commands would execute (including in `--dry-run`).
- Add coverage for parsing normalization and one-liner rendering via hermetic fixtures (no host mutation).

## Inputs (authoritative)
- Operator contract (one-liner template, rendering rules, safe parsing posture): `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- DR-0001 (normalization + duplicates + tokenization) and DR-0003 (os-release path seam): `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
- Feature intent + locked vocabularies: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## Behavior (authoritative)

### Definitions
- **os-release path**: the file path used for distro detection input.
  - If `SUBSTRATE_INSTALL_OS_RELEASE_PATH` is set and non-empty, that path is used.
  - Otherwise, the path is `/etc/os-release`.
- **Normalized distro fields**:
  - `distro_id`: normalized `ID` value, or `<unknown>`.
  - `distro_id_like`: normalized `ID_LIKE` value, or `<unknown>`.

### Best-effort read + safe parsing posture
- The installer MUST read the os-release path as plain text.
- The installer MUST NOT `source`, eval, or execute os-release contents.
- The installer MUST ignore empty lines and comment lines (after trimming leading whitespace) that start with `#`.
- The installer MUST perform no network calls and no writes as part of distro detection.

### Key/value extraction (deterministic; DR-0001)
- Only keys `ID` and `ID_LIKE` are in scope.
- Key matching MUST be case-insensitive.
- If a key occurs multiple times, the last occurrence MUST win.

Value normalization for both keys:
- Trim leading/trailing ASCII whitespace.
- If the value starts and ends with the same quote character (`"` or `'`), strip exactly one leading and one trailing quote.
- Convert to lowercase for matching and rendering.
- Collapse internal ASCII whitespace runs to a single space.

Missing/invalid inputs:
- If the os-release path is missing or unreadable, both `distro_id` and `distro_id_like` MUST be `<unknown>`.
- If `ID` is missing, `distro_id` MUST be `<unknown>`.
- If `ID_LIKE` is missing, `distro_id_like` MUST be `<unknown>`.

### `ID_LIKE` tokenization (for later slices)
- `ID_LIKE` MUST be tokenized on ASCII whitespace into an ordered list of non-empty tokens.
- Token matching semantics are owned by the mapping logic (BEDPM1); this slice only owns tokenization determinism.

### Decision one-liner (stderr; exactly once; contract template)
When a supported package manager has been selected and verified present in `PATH`, and before any package-manager install commands would run, the installer MUST print exactly one line to stderr:

```text
Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)
```

Rendering rules (from `contract.md`):
- `<id>` is `distro_id`; `<id_like>` is `distro_id_like`.
- If the os-release path is missing/unreadable, both render as `<unknown>`.
- `<pkg_manager>` is the selected manager name (must be in the supported set).
- `<source>` is the selection source enum; this slice introduces only:
  - `env` when selection is derived from `PKG_MANAGER`,
  - `path_probe` when selection is derived from `PATH` probing.

## Acceptance criteria
- AC-BEDPM0-01: When `SUBSTRATE_INSTALL_OS_RELEASE_PATH` points to a fixture containing `ID=ubuntu` and `ID_LIKE="debian"`, the installer renders `distro_id=ubuntu` and `distro_id_like=debian` in the decision one-liner (lowercased; quotes stripped).
- AC-BEDPM0-02: When the os-release input is missing or unreadable, the decision one-liner renders `Detected distro: <unknown> (like: <unknown>)` (exact tokens, including angle brackets).
- AC-BEDPM0-03: Parsing ignores comments and empty lines: given a fixture where `ID` appears only in commented lines, `distro_id` renders as `<unknown>`.
- AC-BEDPM0-04: Duplicate key handling: given a fixture with two `ID` lines, the last `ID` occurrence wins in rendered `distro_id`.
- AC-BEDPM0-05: Normalization: given fixtures with (a) mixed-case keys (`Id`, `id_like`), (b) surrounding single quotes, and (c) internal whitespace runs, the rendered `distro_id` / `distro_id_like` are normalized per DR-0001 (lowercase; one space between tokens).
- AC-BEDPM0-06: The decision one-liner is emitted to stderr exactly once per installer invocation and matches the exact template in `contract.md` (no prefixes and no additional formatting).
- AC-BEDPM0-07: Emission timing: the decision one-liner is emitted before the first package-manager install command would execute (including in `--dry-run` mode).
- AC-BEDPM0-08: Safety posture: distro detection uses plain-text reading (no `source`/eval) and performs no writes; the decision one-liner and any new stderr text introduced by this slice do not print the full `PATH` or the full environment.

## Out of scope
- `--pkg-manager` flag parsing/validation and exit code semantics for invalid/forced managers (BEDPM1).
- `/etc/os-release` family mapping and deterministic `PATH`-probe warnings and ambiguity policy (BEDPM1).
- Full hermetic end-to-end detection harness covering precedence, mapping, warnings, and exit codes (BEDPM2).
