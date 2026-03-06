# best-effort-distro-package-manager — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## Slice IDs (canonical)

Slice IDs are stable identifiers used by:
- slice specs under `slices/`,
- triad tasks (`tasks.json`), and
- CI checkpoint wiring.

Canonical slice IDs:
- Slice prefix: `BEDPM`
- `BEDPM0` — Detect distro + emit decision one-liner
- `BEDPM1` — Deterministic pkg-manager selection + failure posture
- `BEDPM2` — Hermetic detection test harness

## Required spec documents (authoritative)

Each spec doc below MUST exist under `docs/project_management/packs/draft/best-effort-distro-package-manager/` and MUST NOT contradict the other authoritative docs in this directory.

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md` (this file)
  - Owns (authoritative): required doc set; surface→doc ownership; determinism checklist.
  - Links (non-authoritative): `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`
  - Owns (authoritative): touch set; cascading implications; cross-queue overlaps; contradiction risks.
  - Links (non-authoritative): all specs in this manifest.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Owns (authoritative): operator-facing Linux installer pkg-manager selection contract (flags/env vars/precedence/output/exit codes/safety posture).
  - Links (non-authoritative): exit code taxonomy; ADR-0031 context.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - Owns (authoritative): DR-0001/2/3 decisions and downstream required updates.
  - Links (non-authoritative): `contract.md`; slice specs.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
  - Owns (authoritative): slice sequencing; execution runbook; required validation commands.
  - Links (non-authoritative): `tasks.json`; `contract.md`; `decision_register.md`; `manual_testing_playbook.md`; slice specs.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
  - Owns (authoritative): deterministic human-run cases (commands + expected outputs/exit codes).
  - Links (non-authoritative): `contract.md`; slice specs; smoke scripts.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`
  - Owns (authoritative): triad task graph; slice references; checkpoint boundaries (when populated).
  - Links (non-authoritative): `plan.md`; slice specs.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
  - Owns (authoritative): BEDPM0 slice acceptance criteria and test strategy.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
  - Owns (authoritative): BEDPM1 slice acceptance criteria and test strategy.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`
  - Owns (authoritative): BEDPM2 slice acceptance criteria and test strategy (including the hermetic harness contract for `tests/installers/pkg_manager_detection_test.sh`).

## Coverage matrix (surface → authoritative doc)

Every ADR-0031-touched surface MUST appear exactly once in this table.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Installer entrypoint | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | `scripts/substrate/install-substrate.sh` scope; Linux-only behavior; no behavior change on macOS/Windows |
| Supported pkg-manager set | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact allowed values: `apt-get|dnf|yum|pacman|zypper` |
| CLI override | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | `--pkg-manager`; precedence; validation; exit `2/3`; no fallback |
| Env override | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | `PKG_MANAGER`; precedence; validation; exit `2/3`; no fallback |
| Hermetic os-release seam | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | `SUBSTRATE_INSTALL_OS_RELEASE_PATH`; default; safety invariants |
| `/etc/os-release` parsing | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | safe parsing; normalization; duplicates; case; `ID_LIKE` tokenization |
| Mapping rules | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | family match rules; tie-break; Fedora/RHEL fallback; mapped-missing behavior |
| PATH probe fallback | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | probe set; fixed precedence order; ambiguity warning content; no full PATH printing |
| Decision one-liner | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact string; stderr; exactly once; emission timing; `<unknown>` rendering; `source` enum |
| Exit codes | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | `0/2/3/4` meanings scoped to Linux installer flows; taxonomy reference |
| Remediation guidance | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | required stderr content elements for exit `2/3/4`; prerequisite command list rules |
| Decision records | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | DR-0001/2/3; selected options; required downstream updates |
| Touch set + conflicts | `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md` | explicit create/edit lists; contradiction risks; cross-queue scan + explicit resolutions |
| BEDPM0 acceptance criteria | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md` | os-release best-effort read; `<unknown>` rules; decision one-liner exactness + timing |
| BEDPM1 acceptance criteria | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md` | precedence pipeline; override failures; mapping/PATH probe selection; warning/error elements |
| BEDPM2 acceptance criteria | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md` | hermetic harness: stubs, fixtures, assertions, non-mutating guarantee |
| Hermetic harness script | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md` | `tests/installers/pkg_manager_detection_test.sh` contract: inputs, assertions, exit codes |
| Plan/sequence/validation | `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` | slice ordering; required validation commands; optional container smoke status (not CI gating) |
| Manual validation | `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` | deterministic cases; exact commands; expected exit codes; expected one-liner string |
| Task graph | `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` | triad task IDs and deps; slice-spec references; checkpoint boundary wiring |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Error model (exit codes and failure posture)
- Security/safety invariants (where applicable)
- Platform guarantees (Linux/macOS/Windows as applicable)

### `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

MUST define (Linux installer contract):
- CLI: `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` (precedence; validation; exit codes).
- Environment: `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>` (precedence; validation; exit codes).
- Hermetic seam: `SUBSTRATE_INSTALL_OS_RELEASE_PATH=<path>` (default; safety invariants).
- Precedence pipeline: flag → env → os-release mapping → PATH probe.
- `/etc/os-release` parsing: safe text parsing; deterministic normalization; deterministic duplicate-key behavior; deterministic `ID_LIKE` tokenization.
- Mapping: locked family rules; Fedora/RHEL `dnf`→`yum` fallback; mapped-missing behavior.
- PATH probe: probe set; fixed precedence order; ambiguity warning content elements.
- Output: exact decision one-liner string template; emission timing; exactly-once invariant; `source` enum.
- Exit codes (Linux flows): `0/2/3/4` meanings + taxonomy reference.
- Failure guidance content elements: required stderr elements for exit `2/3/4`, including the manual prerequisite command list rule.
- Platform guarantees: Linux applies; macOS/Windows no behavior change for this feature.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

MUST define:
- DR-0001: `/etc/os-release` parsing + matching rules (two options; one selected; downstream updates).
- DR-0002: PATH probe precedence order + ambiguity policy (two options; one selected; downstream updates).
- DR-0003: hermetic os-release injection seam (two options; one selected; downstream updates).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`

MUST define:
- Exact touch set (create/edit) implied by ADR-0031.
- Cascading implications and contradiction risks (Linux-only semantics; Windows exit code overlap risk).
- Cross-queue scan for shared installer surfaces.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`

MUST define:
- Slice sequencing: `BEDPM0` → `BEDPM1` → `BEDPM2`.
- Validation commands:
  - hermetic detection test (`tests/installers/pkg_manager_detection_test.sh`),
  - manual playbook execution requirements.
- Status of `tests/installers/pkg_manager_container_smoke.sh` as optional local validation (not CI gating).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`

MUST define:
- Deterministic manual cases with exact commands and expected outputs for:
  - Debian/Ubuntu-family mapping default.
  - Arch-family mapping default.
  - `--pkg-manager` override.
  - `PKG_MANAGER` override.
  - Invalid override value (exit `2`).
  - Forced manager missing from `PATH` (exit `3`).
  - No supported manager selectable (exit `4`).

### `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`

MUST define:
- Acceptance criteria for os-release best-effort read posture and `<unknown>` rendering.
- Acceptance criteria for the decision one-liner exactness + emission timing + exactly-once invariant.
- Bedrock test cases that cover os-release parsing normalization rules from DR-0001.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`

MUST define:
- Acceptance criteria for the precedence pipeline, override failures, and selection behavior.
- Acceptance criteria for mapped-manager missing behavior and deterministic PATH probe fallback.
- Acceptance criteria for warning/error remediation content elements and exit-code mapping.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`

MUST define:
- Acceptance criteria for the hermetic harness contract:
  - controlled PATH stubs,
  - fake os-release input hook (`SUBSTRATE_INSTALL_OS_RELEASE_PATH`),
  - assertions for precedence, mapping, PATH probe fallback warnings, decision one-liner content, and exit codes,
  - non-mutating guarantee.

### `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`

MUST define (when populated):
- The `BEDPM0/BEDPM1/BEDPM2` triad tasks and dependencies.
- Explicit references to each slice spec and to the authoritative contract (`contract.md`).
- Orchestration branch: `feat/best-effort-distro-package-manager` (matches `meta.automation.orchestration_branch`).

## Follow-ups

Work remaining before execution triads begin:
- Create the slice specs (`slices/BEDPM0/`, `slices/BEDPM1/`, `slices/BEDPM2/`) and ensure they conform to Slice Spec v2 invariants.
- Populate `tasks.json` with the triad task graph and checkpoint wiring.
- Implement the hermetic installer detection harness at `tests/installers/pkg_manager_detection_test.sh`.
- Update the touch-set edits from `impact_map.md` (`scripts/substrate/install-substrate.sh`, `docs/INSTALLATION.md`, `docs/project_management/packs/sequencing.json`, and ADR-0031 related-doc links).
