# best-effort-distro-package-manager — spec manifest

This file enumerates every contract surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## Required spec documents (authoritative)

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` — execution runbook + sequencing overview
- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` — triad task graph + acceptance criteria

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` — authoritative user-facing contract for Linux installer package-manager selection:
  - Owns (authoritative): installer CLI (`--pkg-manager`), env override (`PKG_MANAGER`), default selection algorithm, mapping table, PATH probe determinism rule, required stderr one-liner, exit codes, and required remediation guidance elements.
  - Links to (non-authoritative): slice specs for implementation acceptance + tests.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` — A/B decisions and selections required by the ADR:
  - Owns (authoritative): DR-0001, DR-0002, DR-0003 selections (and their exact chosen contracts).
  - Links to (non-authoritative): slice specs and `contract.md` that implement the selected decisions.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` — manual validation (authoritative):
  - Owns (authoritative): deterministic manual test cases and expected outputs for the new installer behavior (Linux).
  - Links to (non-authoritative): `contract.md` (source of truth for expected behavior).

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C0/C0-spec.md` — slice spec (distro detection + reporting):
  - Owns (authoritative): acceptance criteria for `/etc/os-release` best-effort read + safe parsing posture, `<unknown>` rendering, and emitting the required decision one-liner exactly once.
  - Links to (non-authoritative): `contract.md` (selection algorithm + mapping table).

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C1/C1-spec.md` — slice spec (override + precedence + failure posture):
  - Owns (authoritative): acceptance criteria for override precedence (`--pkg-manager` over `PKG_MANAGER`), validation/fail-closed rules, PATH probe fallback determinism + warnings, and exit-code mapping enforcement.
  - Links to (non-authoritative): `contract.md`, `decision_register.md`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C2/C2-spec.md` — slice spec (hermetic detection tests):
  - Owns (authoritative): hermetic test harness requirements (stub PATH + fake os-release input), required test cases, and what outputs must be asserted.
  - Links to (non-authoritative): `contract.md`, `decision_register.md`.

## Coverage matrix (surface → authoritative doc)

Every surface that the ADR touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Feature scope (Linux-only behavior change; macOS/Windows unchanged) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact platform applicability and “no-op on macOS/Windows” statement |
| Installer entrypoint | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | script path: `scripts/substrate/install-substrate.sh` |
| CLI flag: `--pkg-manager` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | name, args, allowed values, precedence, examples |
| Env var override: `PKG_MANAGER` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | allowed values, precedence vs flag, examples |
| Config files / persistent config | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | explicit “no new config files; no new persistent config surface” statement |
| Input precedence pipeline | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | total ordering: `--pkg-manager` → `PKG_MANAGER` → `/etc/os-release` mapping → PATH probe |
| Supported package-manager set | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact locked set: `apt-get`, `dnf`, `yum`, `pacman`, `zypper` |
| `/etc/os-release` read | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | path, readability check, “read-only; no writes”, “no network calls for detection” |
| `/etc/os-release` parsing posture | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C0/C0-spec.md` | safe parsing requirement: do not `source`; extraction rules for `ID` and `ID_LIKE` |
| Distro fields used | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | which keys influence behavior: `ID`, `ID_LIKE` |
| Distro rendering when unknown | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C0/C0-spec.md` | when to render `<unknown>` for `id` / `id_like` |
| Mapping table (distro → package manager) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact locked mapping table + match rules stated in ADR |
| Fedora/RHEL fallback rule (`dnf` preferred; fallback `yum`) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact fallback trigger and output expectations |
| PATH probe fallback | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | when PATH probing is used and what it searches for |
| Multi-manager PATH ambiguity behavior | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | deterministic precedence order + warning requirement + override guidance content |
| `PATH` environment dependence | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | definition of “available”: package-manager binary must be found in `PATH` |
| Decision one-liner (stderr) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact string template, stderr-only, printed exactly once, printed before installing prerequisites |
| `pkg_manager_source` values in output | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact allowed set: `flag`, `env`, `os_release`, `path_probe` and when each applies |
| Exit code meanings | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exit-code mapping and taxonomy reference (`docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`) |
| Override validation errors | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | what is considered “invalid value” and required remediation guidance elements |
| Fail-closed rules for explicit overrides | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | when to fail vs fall back (override invalid or unavailable must fail) |
| Failure when no manager can be selected | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exit code `4` + required guidance elements including prerequisite list |
| Linux prerequisite set | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact prerequisite command set the installer ensures (and the exact list printed in failure guidance) |
| Slice acceptance (C0) | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C0/C0-spec.md` | acceptance criteria aligned to `contract.md` for detection/reporting |
| Slice acceptance (C1) | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C1/C1-spec.md` | acceptance criteria aligned to `contract.md` for overrides/failures |
| Validation (automation; hermetic) | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C2/C2-spec.md` | deterministic test harness contract and assertions (no host mutation; no containers required) |
| Validation (manual) | `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` | deterministic manual cases and expected outputs (Linux) |
| A/B decision ownership | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | DR-0001, DR-0002, DR-0003 options + one explicit selection each |

## Determinism checklist (per document; must be satisfied before quality gate)

### `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`

Must define:

- The exact required-doc set under `docs/project_management/packs/draft/best-effort-distro-package-manager/`.
- A coverage matrix that assigns every ADR-touched surface to exactly one authoritative doc.
- A per-doc determinism checklist that is sufficient for a third-party reviewer to verify “no implied surfaces”.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

Must define, with singular/testable statements:

- CLI:
  - Script: `scripts/substrate/install-substrate.sh`.
  - `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`: meaning, precedence, validation rules, and examples.
- Environment:
  - `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`: meaning, precedence vs `--pkg-manager`, validation rules, and examples.
- Config:
  - Explicit “no new config files; no new persistent config surface” statement for this feature.
- Defaults and precedence:
  - End-to-end selection precedence pipeline (flag → env → os-release mapping → PATH probe).
  - PATH probe deterministic precedence order when multiple supported managers are present.
- Mapping table:
  - Exact locked distro-family matching rules and resulting default manager.
  - Fedora/RHEL rule: prefer `dnf`, fallback to `yum` when applicable.
- Outputs:
  - Exact stderr one-liner template (including `<unknown>` rendering and `source` field values).
  - Required warning/error message content elements for ambiguity and failure cases (the content elements are normative; exact prose is non-normative, except for the required decision one-liner which is exact).
- Prerequisites:
  - Exact prerequisite command set the installer must ensure (Linux) and the exact list printed in “no supported package manager” guidance.
- Exit codes:
  - Exit code taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
  - Exact meanings for exit codes `0`, `2`, `3`, `4` for this script.
- Security/safety invariants (Linux):
  - Detection reads `/etc/os-release` only; no writes for detection.
  - No network calls for detection.
  - Explicit overrides fail-closed (no silent fallback).
- Platform guarantees:
  - Linux behavior contract applies.
  - macOS/Windows: no behavior change for this feature.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

Must define, as exactly-two-option A/B decisions with one explicit selection each:

- DR-0001: `/etc/os-release` parsing details
  - Must decide: normalization rules (quotes/whitespace), duplicate-key handling, missing-key handling, and case-sensitivity for matches.
- DR-0002: multi-manager PATH probe ambiguity policy
  - Must decide: warning vs failure posture (ADR implies “warn + deterministic precedence”) and the exact precedence order.
- DR-0003: hermetic test hook approach
  - Must decide: how tests inject a fake os-release input (e.g., test-only env var/arg vs harness approach) without weakening production safety posture.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C0/C0-spec.md`

Must define:

- Inputs:
  - The os-release input surface used for this slice (`/etc/os-release` as defined by `contract.md`).
- Parsing acceptance:
  - Explicitly prohibits `source`/shell execution of os-release contents.
  - Extraction behavior for `ID` and `ID_LIKE` and how `<unknown>` rendering is determined.
- Output acceptance:
  - Decision one-liner is emitted exactly once to stderr before prerequisite installation begins (as defined by `contract.md`).
- Failure posture:
  - Behavior when `/etc/os-release` is unreadable or missing is non-fatal to detection (falls through to other steps per `contract.md`).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C1/C1-spec.md`

Must define:

- Override acceptance:
  - `--pkg-manager` takes precedence over `PKG_MANAGER` and autodetection.
  - Invalid override values fail with exit code `2`.
  - Unavailable forced manager fails with exit code `3` (no fallback).
- Autodetection acceptance:
  - When mapping does not yield a usable manager, behavior deterministically proceeds per `contract.md` (no silent/implicit branches).
  - PATH probe ambiguity behavior: deterministic precedence + required warning elements + override guidance.
- Error text acceptance:
  - Failure outputs include the required guidance elements defined in `contract.md`.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C2/C2-spec.md`

Must define:

- Hermetic test harness contract:
  - How the test provides a controlled `PATH` (stub binaries for the supported manager set).
  - How the test provides a fake os-release input (as selected by DR-0003).
  - What outputs must be asserted (one-liner content and `source` values; warning/error content elements where applicable).
- Non-mutating guarantee:
  - Tests must not modify the host system and must not require containers.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`

Must define deterministic cases with expected outputs (Linux), including at minimum:

- Default Ubuntu/Debian-family selection via os-release mapping.
- Default Arch-family selection via os-release mapping.
- Forced selection via `--pkg-manager`.
- Forced selection via `PKG_MANAGER`.
- Failure case: invalid override value.
- Failure case: forced manager not in `PATH`.
- Failure case: no supported manager can be selected (and required remediation guidance).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`

Must define:

- Slice sequencing: a single explicit ordering for C0/C1/C2 and rationale.
- Files touched (by path) and high-level implementation plan consistent with the ADR.
- Validation commands to run (hermetic test + manual playbook).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`

Must define:

- Exact touch set (files/dirs) implied by the ADR.
- Any cascading implications (docs/tests/scripts) and cross-pack conflict scan results.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`

Must define:

- A task graph that includes at least one execution task per slice (C0, C1, C2).
- For each slice task: an explicit reference to its slice spec path and the acceptance criteria source of truth (`contract.md`).

## Follow-ups

Record missing/ambiguous ADR intent that must be resolved before the Planning Pack can pass quality gate:

- ADR “Related Docs” links reference `docs/project_management/packs/draft/detecting-badger/*`, but this ADR’s scope declares feature dir `docs/project_management/packs/draft/best-effort-distro-package-manager/`; reconcile and update the ADR links during planning.
- PATH probe “fixed precedence order” is required by the ADR but not specified; `contract.md` must define the exact ordering and `decision_register.md` DR-0002 must record the selection.
- `/etc/os-release` parsing details are underspecified (quotes/whitespace, missing keys, duplicate keys, case-sensitivity); `decision_register.md` DR-0001 must make these rules explicit and slice specs must align.
- Mapping step failure semantics are underspecified:
  - If os-release mapping matches but the mapped manager binary is missing from `PATH`, specs must state whether to fall back to PATH probe or fail (and which exit code/message applies).
  - If os-release is readable but does not match any mapping rule, specs must state whether to proceed to PATH probe and what `source` value is printed.
- Failure guidance is required to list “which prerequisite commands must be installed manually”, but the ADR only gives examples; `contract.md` must define the exact prerequisite command set the installer requires and prints on failure.
