# Decision Register — best-effort-distro-package-manager

Template standard:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

## DR-0001 — `/etc/os-release` parsing + matching rules (normalization, duplicates, case, `ID_LIKE` tokenization)

**Decision owner(s):** Installer / shell maintainers  
**Date (UTC):** 2026-03-06  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`

**Problem / context**

ADR-0031 requires safe `/etc/os-release` parsing and a locked mapping table, but it does not pin the deterministic handling for:
- quoted values and whitespace normalization,
- duplicate keys,
- case-sensitivity,
- `ID_LIKE` tokenization, and
- tie-break behavior when multiple mapping rules match.

The parsing/matching rules MUST be deterministic and MUST be testable by the hermetic detection harness.

**Option A — Minimal parser (lower complexity; more corner-case drift)**

- Key matching is case-sensitive (`ID`, `ID_LIKE` only).
- Duplicate keys: first occurrence wins.
- Value normalization: trim leading/trailing whitespace; do not strip surrounding quotes.
- `ID_LIKE` matching: substring search on the raw `ID_LIKE` value.
- Mapping tie-break: apply mapping-table family rules in order and allow both `ID` and `ID_LIKE` to participate; first match wins.

**Option B — Normalized parser (deterministic; robust; locked semantics)**

- Key matching is case-insensitive (normalize keys to uppercase before comparing).
- Duplicate keys: last occurrence wins.
- Value normalization:
  - trim leading/trailing ASCII whitespace,
  - strip one matching leading/trailing quote pair (`"` or `'`),
  - lowercase for matching and rendering,
  - collapse internal ASCII whitespace runs to a single space.
- `ID_LIKE` tokenization: split on ASCII whitespace into ordered, non-empty tokens; token matching is exact (not substring).
- Mapping tie-break:
  - if `ID` matches one or more families, select the first match in mapping-table order without consulting `ID_LIKE`,
  - otherwise select the first match in mapping-table order using `ID_LIKE` token matches.

**Recommendation**

- **Selected:** Option B — Normalized parser (deterministic; robust; locked semantics)
- **Rationale (crisp):** it produces a single stable matching model across distros and prevents hidden behavior changes caused by quoting/whitespace/duplicates.

**Downstream doc updates required by this decision**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Add a deterministic `/etc/os-release` parsing contract with the Option B rules.
  - Add explicit mapping tie-break rules so a single family is always selected deterministically.
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
  - Add acceptance criteria and hermetic tests that cover: quoted values, whitespace normalization, duplicate keys, case variants, and `ID_LIKE` token behavior.

## DR-0002 — Deterministic PATH-probe precedence order + multi-manager ambiguity policy

**Decision owner(s):** Installer / shell maintainers  
**Date (UTC):** 2026-03-06  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`

**Problem / context**

ADR-0031 requires the installer to behave deterministically when multiple supported package managers are present in `PATH`, and to emit a warning that lists the other detected managers and how to override.

The contract MUST pin:
- the fixed `PATH` probe precedence order, and
- the ambiguity policy (warn-and-continue vs fail-closed).

**Option A — Warn and deterministically select (ADR-aligned; compatibility-preserving)**

- Probe set: `apt-get`, `dnf`, `yum`, `pacman`, `zypper`.
- “Found” definition: `command -v <manager>` succeeds.
- Fixed selection precedence order:
  1) `apt-get`
  2) `dnf`
  3) `yum`
  4) `pacman`
  5) `zypper`
- If multiple supported managers are found:
  - select the first by precedence,
  - emit a warning that includes the other found managers (names only) and override guidance (`--pkg-manager …`, `PKG_MANAGER=…`),
  - do not print the full `PATH`.

**Option B — Fail on ambiguity (fail-closed; requires explicit override)**

- If multiple supported managers are found in `PATH`, exit `2` and require `--pkg-manager …` or `PKG_MANAGER=…` to proceed.
- Emit an error message that lists detected managers and the override mechanisms.

**Recommendation**

- **Selected:** Option A — Warn and deterministically select (ADR-aligned; compatibility-preserving)
- **Rationale (crisp):** it preserves the current “pick one” posture while making the choice deterministic and observable.

**Downstream doc updates required by this decision**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Pin the fixed precedence order and the required warning content elements.
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
  - Add hermetic tests that cover: single-manager PATH, multi-manager PATH warning content elements, and deterministic selection by precedence.

## DR-0003 — Hermetic-test os-release injection seam (fake input without host mutation)

**Decision owner(s):** Installer / shell maintainers  
**Date (UTC):** 2026-03-06  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`

**Problem / context**

ADR-0031 requires a hermetic detection test harness that supplies a fake os-release file without requiring containers and without mutating the host OS. The production installer reads `/etc/os-release`, so the contract must define a deterministic injection seam.

**Option A — Env-var os-release path override (execute-as-script compatible)**

- Add env var: `SUBSTRATE_INSTALL_OS_RELEASE_PATH=<path>`.
- Default behavior: when unset, read `/etc/os-release`.
- When set, read the configured path instead of `/etc/os-release`.
- Safety posture:
  - read as plain text,
  - no `source`/eval/execute of file content.

**Option B — Source-only seam (no new env-var surface)**

- Do not add a new env var.
- Require tests to `source scripts/substrate/install-substrate.sh` and override an internal variable/function that supplies os-release contents.

**Recommendation**

- **Selected:** Option A — Env-var os-release path override (execute-as-script compatible)
- **Rationale (crisp):** it allows hermetic tests to run the installer as an executable script while keeping the parsing safety posture unchanged.

**Downstream doc updates required by this decision**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Define `SUBSTRATE_INSTALL_OS_RELEASE_PATH` and its safety invariants.
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
  - Require the hermetic harness to set `SUBSTRATE_INSTALL_OS_RELEASE_PATH` to a fake os-release fixture path.
