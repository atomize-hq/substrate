# best-effort-distro-package-manager — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## Required spec documents (authoritative)

List the exact spec documents that must exist under `docs/project_management/packs/draft/best-effort-distro-package-manager/`.

Each entry includes:
- what surfaces it owns (authoritative), and
- what it links to (non-authoritative).

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md` — spec selection + ownership map (this file)
  - Owns (authoritative): required doc set; surface→doc ownership; follow-ups.
  - Must define (deterministic items): the exact required-doc list; a surface-complete coverage matrix; the determinism checklist gate for each selected doc.
  - Links (non-authoritative): ADR(s); planning/spec standards.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`
  - Owns (authoritative): touch set allowlists + derived risk notes for the installer + test harness touch set.
  - Must define (deterministic items): create/edit touch allowlists; cross-pack dependency/conflict notes; validation evidence requirements implied by touched surfaces.
  - Links (non-authoritative): all spec docs selected by this manifest.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
  - Owns (authoritative): execution runbook + sequencing notes for slices `BEM0`/`BEM1`/`BEM2` (including required validation commands).
  - Must define (deterministic items): execution sequencing and required commands for validation (hermetic tests + any required shellcheck/lint if adopted); exact operator-visible “what changed” summary pointer to `contract.md`.
  - Links (non-authoritative): `contract.md`; slice specs; `tasks.json`; `manual_testing_playbook.md`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`
  - Owns (authoritative): triad task graph + `ac_ids` traceability for slices `BEM0`/`BEM1`/`BEM2`.
  - Must define (deterministic items): task IDs and deps for each slice triad; references to the slice spec paths; acceptance criteria traceability (`ac_ids`) for strict mode.
  - Links (non-authoritative): slice specs; `plan.md`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - Owns (authoritative): the decision records referenced by ADR-0031:
    - DR-0001: `/etc/os-release` parser approach (must not `source`; must specify quoting/whitespace handling deterministically).
    - DR-0002: deterministic policy for multi-manager PATH probe (warn vs fail) and the exact precedence order when multiple managers are present.
    - DR-0003: hermetic test hook for supplying a fake os-release file (mechanism + posture; must not become a persistent operator-facing surface).
  - Must define (deterministic items): exactly two options (A/B) per DR; one selection; the contract-level constraints each selection imposes; and the spec/doc paths that must be updated after selecting.
  - Links (non-authoritative): `contract.md` (final contract wording); slice specs.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Owns (authoritative): the operator-facing installer contract (Linux only) for `scripts/substrate/install-substrate.sh`, including:
    - CLI flag: `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`
    - env var: `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`
    - selection precedence: flag → env → `/etc/os-release` mapping → PATH probe
    - distro detection input surface: `/etc/os-release` (read-only) and keys `ID` / `ID_LIKE`
    - mapping table (initial locked set) + any fallback rules (e.g., `dnf`→`yum` for Fedora/RHEL family)
    - deterministic PATH-probe behavior when multiple managers exist (including required warning content)
    - required stderr one-liner format and `<unknown>` rendering rules
    - exit codes (taxonomy reference + feature semantics)
    - invariants (no network calls for detection; do not `source` os-release; do not persist install-state)
  - Must define (deterministic items):
    - allowed manager identifiers and rejection rules (exact set; no synonyms)
    - all precedence rules, including tie-breakers for PATH probe and mapped-manager availability behavior
    - absence semantics for `/etc/os-release`, `ID`, and `ID_LIKE`
    - remediation guidance requirements for exit codes `3` and `4` (required elements)
  - Links (non-authoritative): `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
  - Owns (authoritative): manual validation procedures + expected observable outcomes (stderr one-liner fields + exit codes) for:
    - default Ubuntu-family selection
    - default Arch-family selection
    - `--pkg-manager` override
    - `PKG_MANAGER` override (and precedence loss to `--pkg-manager`)
    - failure: invalid override value (exit `2`)
    - failure: forced manager missing from `PATH` (exit `3`)
    - failure: no manager selectable (exit `4`) + required remediation elements
  - Must define (deterministic items): preconditions; exact commands to run; expected stderr decision line (by fields); expected exit code per case; and cleanup/restore steps (if any).
  - Links (non-authoritative): `contract.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEM0/BEM0-spec.md` — ADR slice C0: distro detection + reporting
  - Owns (authoritative): slice `BEM0` acceptance criteria and out-of-scope constraints (must not expand supported manager set).
  - Must define (deterministic items): AC IDs for every `BEM0` acceptance criterion; required test signals consumed by `BEM2`.
  - Links (non-authoritative): `contract.md`; `decision_register.md` (DR-0001).

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEM1/BEM1-spec.md` — ADR slice C1: explicit override + precedence pipeline
  - Owns (authoritative): slice `BEM1` acceptance criteria (arg parsing + precedence + fail-closed behavior).
  - Must define (deterministic items): AC IDs for every `BEM1` acceptance criterion; error/exit-code assertions by meaning; multi-manager PATH-probe warning requirements.
  - Links (non-authoritative): `contract.md`; `decision_register.md` (DR-0002).

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEM2/BEM2-spec.md` — ADR slice C2: hermetic detection tests
  - Owns (authoritative): slice `BEM2` acceptance criteria AND the developer/CI-facing contract for the hermetic test harness under `tests/installers/`:
    - test script path(s) and invocation contract
    - fixture layout (stub package-manager binaries; fake os-release file)
    - mechanism for supplying fake os-release data (per DR-0003)
    - required assertions (precedence; mapping selection; stable stderr decision line fields)
    - zero host mutation guarantees
  - Must define (deterministic items): setup/teardown; how stderr/exit code are captured; exact assertion list required by ADR “Validation Plan”.
  - Links (non-authoritative): `contract.md`; `decision_register.md` (DR-0003).

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0031 touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Installer entrypoint (Linux): `scripts/substrate/install-substrate.sh` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | platform gating; scope of this feature’s contract within the installer |
| CLI flag `--pkg-manager` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | grammar; allowed values; precedence; invalid-value exit `2`; missing-binary exit `3` |
| Env var override `PKG_MANAGER` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | allowed values; precedence below flag; invalid-value exit `2` |
| Supported manager identifier set | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact locked set `{apt-get,dnf,yum,pacman,zypper}`; explicit non-support for others (e.g., `apk`) |
| Selection precedence pipeline | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact precedence order; `pkg_manager_source` allowed values `{flag,env,os_release,path_probe}`; per-stage availability checks |
| `/etc/os-release` read path | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | path used; read-only invariant; behavior when missing/unreadable |
| `/etc/os-release` keys `ID` / `ID_LIKE` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | parsing rules; absence semantics; `<unknown>` rendering rules |
| Distro-family mapping table | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact match sets for `ID`/`ID_LIKE`; Fedora/RHEL `dnf` preference and `yum` fallback behavior |
| PATH probe fallback | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | supported-manager detection method; deterministic precedence order when multiple managers are present; required warning fields + override instructions |
| Required stderr decision line | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact one-liner format; stream=stderr; emission timing (before prereq install); `<unknown>` substitution |
| Remediation guidance (failure paths) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | required elements for exit `3` and `4` guidance: override instructions + manual prereq list |
| Exit codes (`0`,`2`,`3`,`4`) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | mapping to taxonomy meanings; feature-specific success/no-op semantics; no additional exit codes |
| Detection locality | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | invariant: “detection” performs no network calls and only reads local files (`/etc/os-release`) |
| Safe parsing posture | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | invariant: must not `source` `/etc/os-release` or execute arbitrary shell code from it |
| No install-state persistence | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | invariant: must not write `$SUBSTRATE_HOME/install_state.json` (explicit ADR non-goal) |
| Decision records DR-0001/DR-0002/DR-0003 | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | A/B options; chosen selection; rationale; pointers to the contract sections that the selection constrains |
| Hermetic detection test harness (`tests/installers/...`) | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEM2/BEM2-spec.md` | test script interface; fixtures; fake os-release injection mechanism; required assertions |
| Optional container smoke check (`tests/installers/pkg_manager_container_smoke.sh`) | `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` | optional usage notes; non-CI gating posture; expected signals |
| Manual operator validation steps | `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` | command sequences; expected stderr decision line fields; expected exit codes |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes, error messages where applicable) and failure posture
- Ordering/atomicity/concurrency rules (if any)
- Security/redaction invariants (if any)
- Platform guarantees (Linux/macOS/Windows/WSL as applicable)

## Follow-ups

Record missing/ambiguous ADR intent here (do not patch ADRs from this step).

1) ADR “Related Docs” path drift
   - Issue: ADR-0031 “Related Docs” points to `docs/project_management/packs/draft/detecting-badger/...`, but the feature directory is `docs/project_management/packs/draft/best-effort-distro-package-manager/`.
   - Required fix: update ADR-0031 links to the correct feature directory paths after this manifest is accepted.

2) PATH-probe precedence order is underspecified
   - Issue: ADR-0031 requires a “fixed precedence order” when multiple managers are found in `PATH`, but does not specify the exact order.
   - Required fix: select and record an exact precedence order in `decision_register.md` (DR-0002) and reflect it in `contract.md`.

3) `/etc/os-release` parsing rules must be pinned
   - Issue: ADR-0031 requires safe parsing (no `source`) but does not define the exact parsing rules (quoting, whitespace, comment handling, case normalization, `ID_LIKE` tokenization).
   - Required fix: select and record deterministic parsing rules in `decision_register.md` (DR-0001) and reflect them in `contract.md`.

4) Mapped-manager availability semantics (non-Fedora families)
   - Issue: Fedora/RHEL family specifies `dnf`→`yum` fallback, but other families do not specify what happens when the mapped manager binary is missing from `PATH`.
   - Required fix: define the exact fallback/exit behavior in `contract.md` (and ensure it remains consistent with exit codes `3`/`4`).

5) Hermetic test os-release injection mechanism
   - Issue: ADR-0031 requires a fake os-release file for hermetic tests but does not specify the injection mechanism (env var, flag, wrapper, etc.).
   - Required fix: decide in `decision_register.md` (DR-0003) and specify the test contract in `slices/BEM2/BEM2-spec.md`.

6) Canonical “manual prereq” list for exit `4` remediation
   - Issue: ADR-0031 requires remediation guidance listing “which prerequisite commands must be installed manually” but does not define the exact list.
   - Required fix: define the canonical prerequisite-command list and its rendering in `contract.md` (and reference it from the manual testing playbook).
