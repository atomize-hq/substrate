# add-non-apt-system-package-provisioning-support — spec manifest

This file enumerates every contract/protocol/schema surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

## Required spec documents (authoritative)

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
  - Owns (authoritative): required doc set; surface→doc ownership; follow-ups (this file).
  - Must define (deterministic items): the exact required-doc list; a surface-complete coverage matrix; the determinism checklist gate for each selected doc.
  - Links (non-authoritative): ADR(s); planning/spec standards.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`
  - Owns (authoritative): touch set + cascading implications + cross-queue conflicts for slices `C0`/`C1`/`C2`.
  - Must define (deterministic items): create/edit touch allowlists; cross-pack dependency/conflict notes; validation evidence requirements implied by touched contract surfaces.
  - Links (non-authoritative): all docs selected by this manifest.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
  - Owns (authoritative): execution runbook + sequencing notes for slices `C0`/`C1`/`C2` (including required validation commands).
  - Must define (deterministic items): sequencing gates (including the `provisioning_otter` prerequisite); exact validation commands for unit/integration/manual/smoke coverage; an explicit pointer to `contract.md` for operator-visible contract wording.
  - Links (non-authoritative): `contract.md`; slice specs; `tasks.json`; `manual_testing_playbook.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`
  - Owns (authoritative): triad task graph + `ac_ids` traceability for slices `C0`/`C1`/`C2`.
  - Must define (deterministic items): task IDs and deps for each slice triad; references to the slice spec paths; acceptance criteria traceability (`ac_ids`) for strict mode.
  - Links (non-authoritative): slice specs; `plan.md`; `contract.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
  - Owns (authoritative): user-facing contract introduced/changed by ADR-0033, including:
    - CLI: `substrate world enable --provision-deps [--dry-run] [--verbose]` manager-aware behavior for `install.method=pacman` on Arch-family world OSes.
    - CLI: manager mismatch failure behavior for provisioning (enabled set `install.method` vs detected world OS manager).
    - CLI: runtime `substrate world deps current sync|install` fail-early + remediation when the effective enabled set contains system-package methods (`apt|pacman`) (no OS manager execution at runtime).
    - Exit code mapping for provisioning/runtime flows (and any explicit taxonomy overrides).
    - Operator-visible error/remediation message invariants (including required exact command strings and “no host mutation” messaging when provisioning is unsupported).
    - Platform support matrix (Linux host-native vs macOS Lima vs Windows WSL) for `--provision-deps` and runtime behavior.
    - Protected paths/invariants referenced by ADR-0033 (read-only probe paths; writable-path constraints; no host OS mutation).
  - Must define (deterministic items):
    - the exact provisioning entrypoint behavior when the enabled set contains: (a) only `pacman` items, (b) only `apt` items, (c) both `apt` and `pacman` items
    - the exact failure posture and required remediation elements for: (a) provisioning unsupported, (b) world OS manager unsupported/unknown, (c) manager mismatch
    - `--dry-run` output requirements (streams, ordering, and what is guaranteed to be printed)
    - `--verbose` behavior for the provisioning and runtime error paths
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (world-deps inventory schema + effective enabled resolution)
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md` (prerequisite planning pack for APT provisioning)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
  - Owns (authoritative): the decision records referenced by ADR-0033:
    - DR-0001: inventory schema approach (A: explicit `install.method=pacman`; B: abstract system-packages method with per-distro mapping).
    - DR-0002: world OS package-manager probe strategy (A/B; must specify exact inputs and precedence).
    - DR-0003: pacman invocation + idempotency strategy (A/B; must specify exact command shape and non-interactive posture).
    - DR-0004: provisioning mismatch policy (A/B; fail vs partial provision; must specify exact behavior and exit codes).
  - Must define (deterministic items): exactly two options (A/B) per DR; one selection; the contract consequences of each selection (which doc sections are constrained) with no TBDs.
  - Links (non-authoritative): `contract.md`; slice specs; protocol spec (if applicable).

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-provisioning-protocol-spec.md`
  - Owns (authoritative): host↔world-agent protocol contract required by ADR-0033 for provisioning-time probing + pacman provisioning execution, including:
    - how provisioning-time execution is requested (including the exact `ExecuteRequest.profile` value, if used)
    - guard rails that ensure provisioning execution does not weaken hardened runtime execution
    - error model + timeouts/retries relevant to probe + pacman execution
  - Must define (deterministic items): request/response schema references; timeout budgets; mapping points that distinguish exit code `3` (backend unavailable) vs `4` (unsupported/prereq missing); and any required allowlist/denylist rules for provisioning execution.
  - Links (non-authoritative): `docs/WORLD.md`; `docs/CONFIGURATION.md` (only if existing env vars/profiles are reused).

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
  - Owns (authoritative): manual validation procedures + expected observable outcomes (stdout/stderr + exit codes) for:
    - Arch-family world provisioning success via `pacman`
    - provisioning mismatch remediation (world OS vs enabled `install.method`)
    - runtime fail-early behavior for system-package methods (`apt|pacman`)
    - Linux host-native unsupported behavior (explicit “no host mutation” messaging)
  - Must define (deterministic items): preconditions; exact commands to run; expected key output lines; expected exit code per case; and cleanup/restore steps (if any).
  - Links (non-authoritative): `contract.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
  - Owns (authoritative): Linux smoke procedure for this feature.
  - Must define (deterministic items): preconditions; commands; expected exit codes; required signals for provisioning unsupported behavior on Linux host-native.
  - Links (non-authoritative): `manual_testing_playbook.md`; `contract.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
  - Owns (authoritative): macOS smoke procedure for this feature (Lima backend).
  - Must define (deterministic items): preconditions; commands; expected exit codes; required signals for Arch-family world provisioning when applicable.
  - Links (non-authoritative): `manual_testing_playbook.md`; `contract.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
  - Owns (authoritative): Windows smoke procedure for this feature (WSL backend).
  - Must define (deterministic items): preconditions; commands; expected exit codes; explicit behavior when provisioning is unsupported on Windows/WSL in v1 (must match `contract.md`).
  - Links (non-authoritative): `manual_testing_playbook.md`; `contract.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md`
  - Owns (authoritative): slice `C0` behavior (world OS package-manager probe for provisioning-time):
    - exact in-world probe inputs (`/etc/os-release` keys; manager presence checks)
    - OS-family classification rules sufficient to gate `pacman` provisioning for Arch-family worlds
    - backend capability detection and mapping to exit codes/messages for the probe stage
  - Must define (deterministic items): probe command sequence; parsing/canonicalization rules; absence semantics; and the exact derived “manager” enum used by provisioning selection.
  - Links (non-authoritative): `contract.md`; `decision_register.md` (DR-0002); `world-deps-pacman-provisioning-protocol-spec.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C1/C1-spec.md`
  - Owns (authoritative): slice `C1` behavior (pacman provisioning path for world-deps system packages):
    - derivation of the required `pacman` package set from the effective enabled world-deps set
    - provisioning execution steps (including idempotency rules and `--dry-run` output requirements)
    - failure posture and deterministic error mapping for unsupported backends and missing prerequisites
  - Must define (deterministic items): de-duplication/ordering rules; the exact `pacman` command invocation shape selected (per DR-0003); and the mapping from execution failures to exit codes and required remediation elements.
  - Links (non-authoritative): `contract.md`; `decision_register.md` (DR-0003/DR-0004); `world-deps-pacman-provisioning-protocol-spec.md`; `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C2/C2-spec.md`
  - Owns (authoritative): slice `C2` behavior (validation + operator docs updates):
    - required unit/integration test coverage for schema validation, probe selection, and pacman command construction
    - required runtime short-circuit coverage for system-package methods (`apt|pacman`) with exit `4` + remediation invariants
    - operator docs update requirements to avoid “apt-like” remediation on non-APT worlds
  - Must define (deterministic items): the exact assertions required by ADR-0033 “Validation Plan”; test harness assumptions; and the exact docs pages/messages that must be updated (by path).
  - Links (non-authoritative): `contract.md`; `manual_testing_playbook.md`.

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0033 touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI provisioning entrypoint: `substrate world enable --provision-deps` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | flags, defaults, success/no-op semantics, exit codes, and worked examples |
| `--dry-run` behavior for provisioning | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | “no mutation” invariant; required printed content; output stream(s) and ordering |
| `--verbose` behavior for provisioning/runtime errors | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | what additional info is emitted; stable phrasing/fields required by tests/playbook |
| Provisioning manager selection (probe result → `apt` vs `pacman`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md` | probe-derived manager enum; selection rules; behavior when unknown/unsupported |
| World OS probe input: `/etc/os-release` (`ID` / `ID_LIKE`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md` | parsing/canonicalization rules; absence semantics; classification mapping rules |
| World OS probe input: manager presence check (`command -v pacman`, etc.) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md` | exact commands; precedence vs `/etc/os-release`; behavior on execution failure |
| Provisioning mismatch behavior (enabled methods vs detected manager) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact failure posture; required remediation elements; exit code mapping |
| Provisioning-time APT execution semantics (Debian/Ubuntu-family worlds) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C0/C0-spec.md` | requirement derivation + idempotency; `--dry-run` semantics; mapping from failures to exit codes |
| Host↔world-agent provisioning execution protocol (APT path) | `docs/project_management/packs/draft/world-deps-apt-provisioning/world-deps-apt-provisioning-protocol-spec.md` | request/response schema; request profile semantics; guard rails; timeouts; error mapping |
| Pacman provisioning derivation (enabled set → package list) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C1/C1-spec.md` | derivation algorithm; ordering/de-dup rules; interaction with bundles |
| Pacman provisioning execution semantics | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C1/C1-spec.md` | exact command shape; idempotency behavior; non-interactive posture |
| Runtime fail-early + remediation for system-package methods (`apt|pacman`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | “no OS manager execution at runtime” invariant; required exact command string `substrate world enable --provision-deps`; unsupported-backend messaging |
| Runtime preflight detection rules for system-package methods | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C2/C2-spec.md` | detection inputs (effective enabled set vs explicit args); side effects prohibited; required error assertions |
| Exit codes (`0`,`2`,`3`,`4`,`5`) for provisioning/runtime flows | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | mapping to `EXIT_CODE_TAXONOMY.md` meanings; feature-specific semantics per failure mode |
| World-deps inventory schema for APT items (`install.method=apt`, `install.apt[]`) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | schema, constraints, defaults/absence semantics, and version semantics |
| World-deps inventory schema for pacman items (`install.method=pacman`, `install.pacman[]`) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | schema, constraints, defaults/absence semantics, and ordering semantics |
| Effective enabled world-deps resolution (inventory sources + enabled patches) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | source paths, precedence/merge rules, and “effective enabled set” definition used by provisioning/runtime checks |
| Host↔world-agent provisioning execution protocol (probe + pacman) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-provisioning-protocol-spec.md` | request/response schema; request profile semantics; guard rails; timeouts; error mapping |
| Filesystem/path invariants referenced (`/etc/os-release`, `/var/lib/substrate/world-deps`, `/tmp`, “no host OS mutation”) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | invariants, failure posture, and platform-specific notes |
| Platform support matrix (Linux host-native vs macOS Lima vs Windows WSL) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | deterministic per-platform behavior; explicit unsupported behavior; required messaging |
| Validation assertions required by ADR-0033 | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C2/C2-spec.md` | exact unit/integration assertions; required fixtures/harness hooks; required doc-update assertions |
| Manual validation procedures | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md` | command sequences; expected key output lines; expected exit codes; cleanup steps |
| Smoke procedure (Linux) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh` | minimal pass/fail command set; expected exit codes; required signals |
| Smoke procedure (macOS) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh` | minimal pass/fail command set; expected exit codes; required signals |
| Smoke procedure (Windows) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1` | minimal pass/fail command set; expected exit codes; required signals |
| Decision records DR-0001/DR-0002/DR-0003/DR-0004 | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` | A/B options; chosen selection; rationale; pointers to the constrained spec sections |

## Determinism checklist (must be satisfied before quality gate)

### `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- MUST define all affected commands/flags/defaults with at least one worked example per platform/backend class.
- MUST define the exact remediation message invariants, including the exact command string `substrate world enable --provision-deps`.
- MUST define a deterministic support matrix for provisioning and the exact behavior when unsupported.
- MUST define the mismatch policy behavior (including the `apt`+`pacman` mixed-enabled-set case).
- MUST define exit codes for every failure mode described in ADR-0033, referencing `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.

### `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-provisioning-protocol-spec.md`
- MUST define the provisioning execution request contract (including whether `ExecuteRequest.profile` is used and its exact value, if used).
- MUST define guard rails that keep hardened runtime execution fail-closed and prevent provisioning behavior from leaking into runtime execution paths.
- MUST define timeouts/budgets and the error model exposed to the CLI surfaces (including mapping points to exit code `3` vs `4`).
- MUST define any pacman/probe-specific allowlist/denylist rules required by `world-agent` (if applicable).

### `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md`
- MUST define the probe algorithm (command sequence + parsing rules) with no host-PATH dependency.
- MUST define the derived manager classification enum and the exact mapping rules from probe outputs to that enum.
- MUST define failure posture and deterministic exit-code mapping when the probe cannot run or yields an unsupported result.

### `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C1/C1-spec.md`
- MUST define the `pacman` requirement derivation algorithm from the effective enabled set (including bundle expansion, de-duplication, and ordering).
- MUST define provisioning execution semantics (idempotency, `--dry-run` output requirements, and what constitutes “no mutation”).
- MUST define the exact `pacman` invocation contract (per DR-0003) and the mapping from failures to exit codes and remediation requirements.

### `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C2/C2-spec.md`
- MUST define “fail early” precisely (what it means operationally and what side effects are prohibited).
- MUST define how runtime `sync` and runtime `install <ITEM...>` behave when system-package methods (`apt|pacman`) exist in scope (including whether scope is “effective enabled set” vs “explicit args only”).
- MUST define the exact unit/integration assertions required by ADR-0033 “Validation Plan” (schema validation, probe selection, pacman command construction, unsupported-backend messaging).
- MUST define the exact operator-doc paths that must be updated (by path) to avoid “apt-like” remediation on non-APT worlds.

### `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
- MUST contain DR-0001/DR-0002/DR-0003/DR-0004 as A/B decisions with one explicit selection each.
- MUST specify the contract consequences of each selected decision (which spec(s) it constrains) with no TBDs.

### `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- MUST define preconditions, exact commands, expected key output lines, and expected exit codes for each required scenario from ADR-0033.
- MUST include at least one mismatch case and one unsupported-backend case with the required “no host mutation” messaging checks.

## Follow-ups

Record missing/ambiguous ADR intent here (do not patch ADRs from this step).

1) ADR “Scope” / “Related Docs” directory drift
   - Issue: ADR-0033 points to `docs/project_management/packs/draft/world-deps-pacman-provisioning/…`, but the resolved feature directory is `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`.
   - Required fix: update ADR-0033 links to the correct feature directory paths after this manifest is accepted.

2) World OS probe algorithm details are underspecified
   - Issue: ADR-0033 lists probe inputs (`/etc/os-release`, manager presence checks) but does not define exact parsing/canonicalization rules, precedence, or the exact Arch-family classification mapping.
   - Required fix: decide and lock probe strategy in `decision_register.md` (DR-0002) and specify the exact algorithm in `slices/C0/C0-spec.md`.

3) Pacman invocation contract is underspecified
   - Issue: ADR-0033 requires “pacman provisioning” but does not specify the exact non-interactive invocation flags, idempotency behavior, or ordering/de-dup rules.
   - Required fix: decide and lock the invocation/idempotency strategy in `decision_register.md` (DR-0003) and specify the exact command contract in `slices/C1/C1-spec.md`.

4) Mismatch policy needs a deterministic rule for mixed enabled sets
   - Issue: ADR-0033 requires mismatch failure, but does not define the exact behavior when the enabled set contains both `install.method=apt` and `install.method=pacman` items, nor whether partial provisioning is ever allowed.
   - Required fix: decide and lock a deterministic mismatch policy in `decision_register.md` (DR-0004) and define the exact operator-facing behavior in `contract.md`.

5) Runtime “scope” for system-package short-circuit is ambiguous
   - Issue: ADR-0033 requires runtime fail-early for `apt|pacman` items, but does not specify whether `deps current install <ITEM...>` evaluates system-package presence over (a) the effective enabled set or (b) only explicitly requested items.
   - Required fix: select exactly one scope rule in `slices/C2/C2-spec.md` and ensure tests enforce it.

6) Cross-pack contract ownership boundary must be reconciled
   - Issue: ADR-0033 extends system-package provisioning beyond APT (ADR-0030). There MUST be exactly one authoritative contract wording for `substrate world enable --provision-deps` and runtime `world deps current sync|install` behavior across methods.
   - Required fix: in `impact_map.md`, explicitly list all existing docs that currently define these surfaces (including `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md` and `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`) and choose a single authoritative `contract.md` location for the unified wording.
