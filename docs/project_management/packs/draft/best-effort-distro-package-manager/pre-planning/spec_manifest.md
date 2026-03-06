# best-effort-distro-package-manager — spec manifest (pre-planning)

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## Slice IDs (canonical)

Slice IDs MUST be feature-derived and stable per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `BEDPM` (derived from `best-effort-distro-package-manager`)
- `BEDPM0` — Linux installer distro + package-manager discovery (override + mapping + deterministic PATH fallback) + hermetic tests

## Required spec documents (authoritative)

List the exact spec documents that MUST exist under `docs/project_management/packs/draft/best-effort-distro-package-manager/`.

Each entry includes:
- Owns (authoritative): the surfaces it is the single source of truth for.
- Must define (deterministic items): the exact items it MUST pin down with singular, testable statements.
- Links (non-authoritative): upstream context docs it may reference but MUST NOT contradict.

Spec templates:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md` (this file)
  - Owns (authoritative): required doc set; surface→doc ownership; follow-ups.
  - Must define (deterministic items): the exact required-doc list; a surface-complete coverage matrix; the determinism checklist gate for each selected doc.
  - Links (non-authoritative): `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
  - Owns (authoritative): touch set + cascading implications + cross-pack conflict notes for slice `BEDPM0`.
  - Must define (deterministic items): the explicit create/edit touch allowlists (by path) implied by ADR-0031, including installer + test paths.
  - Links (non-authoritative): all docs selected by this manifest.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
  - Owns (authoritative): execution runbook + sequencing notes for slice `BEDPM0` (including required validation commands).
  - Must define (deterministic items):
    - Slice sequencing: `BEDPM0` as a single-slice plan, and the triad workflow boundaries.
    - Validation commands to run, including the hermetic installer detection test required by ADR-0031.
    - Whether `tests/installers/pkg_manager_container_smoke.sh` is required; if not required, it MUST be explicitly labeled “optional; not CI gating”.
  - Links (non-authoritative): `tasks.json`; `contract.md`; `decision_register.md`; `manual_testing_playbook.md`; slice specs under `slices/`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` (already exists)
  - Owns (authoritative): triad task graph + slice references for slice `BEDPM0`.
  - Must define (deterministic items):
    - Task IDs and deps for the `BEDPM0` triad (`BEDPM0-code`, `BEDPM0-test`, `BEDPM0-integ`).
    - Explicit references to `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`.
    - The orchestration branch MUST remain `feat/best-effort-distro-package-manager` (matches current `meta.automation.orchestration_branch`).
  - Links (non-authoritative): `plan.md`; slice specs.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Owns (authoritative): the operator-facing contract introduced/changed by ADR-0031 for Linux installer package-manager selection, including:
    - Installer entrypoint: `scripts/substrate/install-substrate.sh`.
    - CLI override: `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` (highest precedence).
    - Env override: `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>` (second precedence).
    - Default selection algorithm (Linux only): flag → env → `/etc/os-release` mapping → deterministic `PATH` probe fallback.
    - Mapping table and match rules exactly as specified by ADR-0031.
    - Required stable stderr one-liner (exact string; printed exactly once; printed before prerequisite installation begins).
    - Exit codes for invalid override / forced manager unavailable / no supported manager available (taxonomy subset used by ADR-0031: `0/2/3/4`).
    - Required warning/error output content elements for ambiguity and failure remediation.
    - Security/safety posture for detection (`/etc/os-release` safe parsing; no shell execution; no network calls; no detection writes).
    - Platform guarantees (Linux behavior contract applies; macOS/Windows no behavior change for this feature).
  - Must define (deterministic items):
    - Supported package-manager set: `{apt-get, dnf, yum, pacman, zypper}`.
    - Input precedence:
      - `--pkg-manager` wins over `PKG_MANAGER`.
      - `PKG_MANAGER` wins over mapping and PATH probe.
    - Validation + fail-closed rules:
      - Invalid `--pkg-manager` value exits `2`.
      - Invalid `PKG_MANAGER` value exits `2`.
      - Forced manager missing from `PATH` exits `3`.
    - `/etc/os-release` handling:
      - Canonical path is `/etc/os-release`.
      - Best-effort read; missing/unreadable results in `<unknown>` rendering.
      - Parsing MUST NOT execute shell code; the contract MUST state the exact parsing/normalization rules and matching rules (with DR-0001 as the rationale record).
    - Mapping behavior:
      - Exact match rules and the resulting preferred manager per family.
      - Fedora/RHEL family fallback rule (`dnf` preferred; fallback to `yum` when `dnf` is unavailable).
      - Behavior when a mapped manager is unavailable in `PATH` (must be singular and testable).
      - Behavior when `/etc/os-release` is readable but matches no mapping rule (must be singular and testable).
    - PATH probe behavior:
      - Exact probe set (the supported manager binaries).
      - Exact multi-manager precedence order.
      - Required warning content elements when multiple supported managers are present.
    - Required stderr one-liner (exact string):
      - `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)`
      - `<id>` and `<id_like>` rendering rules (including `<unknown>` cases).
      - `source` enum values and the exact conditions for each.
    - Failure guidance content elements:
      - For “no supported manager can be selected” failures: the output MUST include override guidance (`--pkg-manager` and `PKG_MANAGER`) and MUST include the missing prerequisite command list that triggered the failure.
      - The contract MUST define the full prerequisite-command universe used by `install-substrate.sh` Linux prereq checks: `{curl, tar, jq, sha256sum, shasum, systemctl, fuse-overlayfs, nft, ip}` and MUST define the conditionality for `{sha256sum, shasum}` and for the `--no-world` path.
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - Owns (authoritative): decision records required to remove ambiguity in ADR-0031, including at minimum:
    - DR-0001: `/etc/os-release` parsing + normalization + matching rules.
    - DR-0002: multi-manager PATH probe ambiguity policy + fixed precedence order.
    - DR-0003: hermetic test hook for supplying fake os-release input.
  - Must define (deterministic items): exactly two options (A/B) per DR; one selection per DR; and the exact docs/sections that MUST be updated to reflect the selection.
  - Links (non-authoritative): `contract.md` (final operator contract wording); slice specs (acceptance criteria constrained by selected DRs).

- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
  - Owns (authoritative): deterministic manual validation cases and expected outputs for ADR-0031 (Linux behavior).
  - Must define (deterministic items): at minimum the cases required by ADR-0031:
    - Default Debian/Ubuntu-family selection via `/etc/os-release` mapping.
    - Default Arch-family selection via `/etc/os-release` mapping.
    - Forced selection via `--pkg-manager`.
    - Forced selection via `PKG_MANAGER`.
    - Failure: invalid override value.
    - Failure: forced manager not in `PATH`.
    - Failure: no supported manager can be selected (must verify exit code `4` and required guidance elements).
  - Links (non-authoritative): `contract.md` (source of truth for exact output/exit-code expectations).

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
  - Owns (authoritative): acceptance criteria for implementing ADR-0031 in `scripts/substrate/install-substrate.sh` and for adding the hermetic installer detection test under `tests/installers/`.
  - Must define (deterministic items):
    - Scope: the exact installer behavior deltas from ADR-0031 that `BEDPM0` ships.
    - Acceptance criteria for precedence: `--pkg-manager` → `PKG_MANAGER` → os-release mapping → deterministic PATH probe.
    - Acceptance criteria for `/etc/os-release` parsing posture (no `source`) and for `<unknown>` rendering.
    - Acceptance criteria for the stable stderr one-liner:
      - emitted exactly once,
      - emitted before prerequisite installation begins,
      - matches the exact `contract.md` template.
    - Acceptance criteria for failure modes and exit codes `2/3/4` as defined in `contract.md`.
    - Hermetic test harness requirements:
      - Controlled `PATH` containing stub `apt-get|dnf|yum|pacman|zypper` binaries.
      - Fake os-release input supply mechanism (per DR-0003).
      - Assertions for precedence + mapping + PATH ambiguity warning + stable one-liner content (including `source`).
      - Tests MUST NOT require containers and MUST NOT mutate the host OS.
  - Links (non-authoritative): `contract.md`; `decision_register.md`.

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0031 touches MUST appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Feature scope | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | Linux-only behavior delta; explicit “no behavior change” statement for macOS/Windows |
| Installer entrypoint | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | script path: `scripts/substrate/install-substrate.sh` |
| CLI override: `--pkg-manager` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | syntax, allowed values, precedence, validation, exit codes, examples |
| Env override: `PKG_MANAGER` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | allowed values, precedence, validation, exit codes, examples |
| Supported manager set | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact allowed enum: `apt-get|dnf|yum|pacman|zypper` |
| Selection precedence pipeline | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | total order: flag → env → os-release mapping → PATH probe |
| `/etc/os-release` read | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | path, best-effort readability rules, read-only posture |
| `/etc/os-release` parsing posture | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | safe parsing rules (no `source`); exact `ID` / `ID_LIKE` extraction + normalization |
| Distro family mapping rules | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact match rules for Debian/Ubuntu, Fedora/RHEL, Arch, SUSE families |
| Mapping table (distro → preferred manager) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact locked mapping table from ADR-0031 |
| Fedora/RHEL fallback rule | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact fallback trigger and the resulting manager selection |
| Mapping failure semantics | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | behavior when mapped manager is unavailable in `PATH` |
| PATH probe fallback | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | probe set, determinism rules, and selection behavior |
| Multi-manager PATH ambiguity warning | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | fixed precedence order; required warning content elements; override guidance elements |
| Stable decision one-liner (stderr) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact string; printed exactly once; printed before prerequisite installation begins |
| `<unknown>` rendering | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact rendering for missing/unreadable `ID` and `ID_LIKE` |
| `source` enum and semantics | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | allowed enum: `flag|env|os_release|path_probe` and exact trigger conditions |
| Exit codes | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exit codes `0/2/3/4` meanings + taxonomy reference |
| Failure guidance content elements | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | required remediation elements for invalid override, forced manager missing, and “no manager” |
| Linux prerequisite command universe | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact command set `{curl, tar, jq, sha256sum, shasum, systemctl, fuse-overlayfs, nft, ip}` + conditionality rules |
| Hermetic detection test harness | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md` | controlled PATH stubs; fake os-release input hook; required assertions; non-mutating guarantee |
| Manual validation cases | `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` | deterministic manual cases and expected outputs |
| A/B decision records | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | DR-0001/2/3: options A/B; one selection per DR; required downstream updates |
| Plan/sequence/validation | `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` | slice order; required validation commands; optional-vs-required status for container smoke |
| Touch set and conflicts | `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md` | explicit touched paths + cross-pack conflict notes |
| Task graph | `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` | `BEDPM0` triad tasks + references to the slice spec |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Error model (exit codes and failure posture)
- Security/safety invariants (where applicable)
- Platform guarantees (Linux/macOS/Windows as applicable)

### `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`

MUST define:
- The exact required-doc set under `docs/project_management/packs/draft/best-effort-distro-package-manager/`.
- A coverage matrix that assigns every ADR-0031-touched surface to exactly one authoritative doc.
- A follow-ups list that is sufficient to remove all ADR-0031 ambiguity before quality gate.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

MUST define (Linux installer contract):
- CLI:
  - Flag: `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`.
  - Precedence and validation rules.
- Environment:
  - Env var: `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`.
  - Precedence and validation rules.
- Detection inputs and precedence (Linux only):
  - Total order: flag → env → os-release mapping → PATH probe.
  - Definition of “available” (binary found in `PATH`).
- `/etc/os-release` contract:
  - Canonical path: `/etc/os-release`.
  - Safe parsing posture (no `source`);
  - Normalization + matching rules (DR-0001 records the choice).
  - `<unknown>` rendering rules.
- Mapping table and fallback:
  - Exact mapping rules from ADR-0031.
  - Fedora/RHEL `dnf`→`yum` fallback rule.
  - Behavior when mapped manager is unavailable in `PATH`.
- PATH probe determinism:
  - Exact probe set.
  - Fixed precedence order for selection.
  - Required warning content elements when multiple managers are present.
- Output:
  - Exact stderr one-liner string template.
  - Exact `source` values and their trigger conditions.
- Exit codes:
  - Taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
  - `0/2/3/4` meanings and trigger conditions.
- Failure guidance content elements:
  - Override guidance elements.
  - Missing prerequisite command list inclusion rule.
- Prerequisites:
  - Full Linux prerequisite command universe and conditionality rules.
- Platform guarantees:
  - Linux behavior contract applies.
  - macOS/Windows: explicit “no behavior change” statement.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

MUST define:
- DR-0001 (`/etc/os-release` parsing/matching rules): exactly two options (A/B) and one selection.
- DR-0002 (PATH ambiguity policy + precedence order): exactly two options (A/B) and one selection.
- DR-0003 (hermetic test hook for fake os-release): exactly two options (A/B) and one selection.
- For each DR: the exact downstream doc updates required in `contract.md` and `slices/BEDPM0/BEDPM0-spec.md`.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`

MUST define:
- Acceptance criteria that fully cover every contract surface owned by `contract.md` (no gaps).
- Hermetic test harness contract:
  - Controlled PATH stub setup.
  - Fake os-release input hook (DR-0003 selection).
  - Required assertions for:
    - precedence ordering,
    - mapping selection,
    - deterministic PATH fallback + warning behavior,
    - stable stderr one-liner content and `source` value.
  - Non-mutating guarantee (no host OS mutation; no container dependency).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`

MUST define:
- Deterministic manual cases with exact commands and expected outputs for:
  - Debian/Ubuntu-family mapping default.
  - Arch-family mapping default.
  - `--pkg-manager` override.
  - `PKG_MANAGER` override.
  - Invalid override value (`exit 2`).
  - Forced manager missing from PATH (`exit 3`).
  - No supported manager selectable (`exit 4`).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`

MUST define:
- Slice sequencing for `BEDPM0` and triad workflow notes.
- Validation commands:
  - Hermetic detection test command (ADR-0031 requirement).
  - Manual playbook execution requirements.
- Status of `tests/installers/pkg_manager_container_smoke.sh` (required vs optional) and the non-CI-gating statement when it is optional.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`

MUST define:
- Exact touch set (create/edit) implied by ADR-0031.
- Cross-pack conflict notes for shared installer/tests surfaces.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`

MUST define:
- A task graph that includes the `BEDPM0` triad tasks and required dependencies.
- For each task: explicit references to `slices/BEDPM0/BEDPM0-spec.md` and the authoritative contract (`contract.md`).

## Follow-ups

Missing or ambiguous ADR-0031 intent that MUST be resolved in the selected specs before quality gate:

- ADR-0031 “Related Docs” links point at `docs/project_management/packs/draft/detecting-badger/*` while ADR-0031 scope declares feature dir `docs/project_management/packs/draft/best-effort-distro-package-manager/`; planning MUST reconcile and update the ADR links.
- DR-0001 is required: ADR-0031 does not fully specify `/etc/os-release` parsing/normalization rules (quotes/whitespace, duplicate keys, missing keys, case-sensitivity for matches).
- DR-0002 is required: ADR-0031 requires a fixed multi-manager PATH probe precedence order but does not specify the exact ordering.
- DR-0003 is required: ADR-0031 requires a hermetic test harness and requires fake os-release input, but does not specify the injection/hook mechanism.
- Mapping failure semantics are underspecified beyond the Fedora/RHEL `dnf`→`yum` rule:
  - When os-release mapping selects a manager that is not available in `PATH`, specs MUST define a single behavior (fallback to PATH probe vs fail-closed) and MUST define the resulting exit code and `source` value.
