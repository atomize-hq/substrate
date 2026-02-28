# add-non-apt-system-package-provisioning-support — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

## Required spec documents (authoritative)

List the exact spec documents that MUST exist under `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`.

Each entry includes:
- Owns (authoritative): the surfaces it is the single source of truth for.
- Must define (deterministic items): the exact items it MUST pin down with singular, testable statements.
- Links (non-authoritative): upstream context docs it may reference but MUST NOT contradict.

Spec templates:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
  - Owns (authoritative): required doc set; surface→doc ownership; follow-ups (this file).
  - Must define (deterministic items): the exact required-doc list; a surface-complete coverage matrix; the determinism checklist gate for each selected doc.
  - Links (non-authoritative): ADR-0033; planning/spec standards.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`
  - Owns (authoritative): touch set + cascading implications + cross-queue conflicts for slices `ANS0`/`ANS1`/`ANS2`.
  - Must define (deterministic items):
    - Exact create/edit touch allowlists (by path) for each slice triad.
    - Exact operator-doc update targets (by path) required by ADR-0033 (and the “link-to-contract.md; do not restate contract tables” rule).
    - Cross-pack dependency/conflict notes, including `world-deps-packages-bundles-contract` (inventory schema) and `world-deps-apt-provisioning` (APT provisioning baseline).
  - Links (non-authoritative): all docs selected by this manifest.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
  - Owns (authoritative): execution runbook + sequencing notes for slices `ANS0`/`ANS1`/`ANS2` (including required validation commands).
  - Must define (deterministic items):
    - Slice sequencing: one explicit ordering for `ANS0`/`ANS1`/`ANS2` and rationale.
    - Exact validation commands required for unit/integration/manual/smoke validation, including required platform coverage.
    - Explicit gates and operator-facing expected outputs for these distinct classes:
      - world backend unavailable (exit `3`)
      - provisioning unsupported (exit `4`)
      - world OS manager mismatch / unsupported manager (exit `4`)
  - Links (non-authoritative): `contract.md`; slice specs; `tasks.json`; `manual_testing_playbook.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json` (already exists)
  - Owns (authoritative): triad task graph + slice references for slices `ANS0`/`ANS1`/`ANS2`.
  - Must define (deterministic items):
    - Task IDs and deps for each slice triad (`ANS{0,1,2}-{code,test,integ-*}`) consistent with automation schema.
    - Explicit references to the slice spec paths for each slice.
    - Acceptance-criteria traceability (`ac_ids`) if strict mode is enabled for this pack.
  - Links (non-authoritative): slice specs; `plan.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
  - Owns (authoritative): operator-facing contract introduced/changed by ADR-0033, including:
    - CLI provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]` (manager-aware; includes `pacman` support).
    - CLI runtime invariant: `substrate world deps current sync|install` MUST NOT invoke OS package managers (`apt` or `pacman`).
    - Exit code mapping for provisioning/runtime flows (taxonomy subset used by ADR-0033: `0/2/3/4/5`).
    - Operator-visible remediation invariants, including the required exact command string `substrate world enable --provision-deps`.
    - Platform/backends support matrix (Linux host-native vs macOS Lima vs Windows WSL assumptions) for `--provision-deps` and for runtime remediation behavior.
    - Protected paths / OS-mutation invariants referenced by ADR-0033 (no host OS mutation; hardened runtime remains fail-closed).
    - Explicit “no new env vars and no new config keys introduced by ADR-0033” statement.
  - Must define (deterministic items):
    - Success/no-op semantics for provisioning when the effective enabled set contains zero system-package items (`install.method=apt` and `install.method=pacman`).
    - The exact exit-code mapping per command and per failure class (invalid inventory schema vs world backend unavailable vs unsupported provisioning vs manager mismatch).
    - The minimum guaranteed remediation content for every unsupported/unmet-prereq path (including “no host OS mutation” messaging on Linux host-native).
    - The exact definition of provisioning `--dry-run` as “no mutation” (no OS package-manager execution and no persistent state changes).
    - The minimum guaranteed `--dry-run` output content (requirement sets + detected/selected manager + “no mutation” statement) and the stream(s) used.
    - The minimum guaranteed `--verbose` output invariants (what extra is guaranteed vs prohibited from being required).
    - The exact behavior of runtime `deps current sync|install --dry-run` when system-package items are in scope (exit code + output/remediation invariants).
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (world-deps inventory + enabled-resolution schema; `install.method=*` item shapes)
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` (APT provisioning baseline; referenced by ADR-0033)
    - `docs/WORLD.md` (world-agent `/v1/execute` and `/v1/stream` baselines; transport context only)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
  - Owns (authoritative): decision records referenced by ADR-0033 and any additional DRs required to remove ambiguity, including at minimum:
    - DR-0001: schema approach (explicit `install.method=pacman` vs abstract mapping).
    - DR-0002: probe strategy (exact rules and precedence for `/etc/os-release` vs manager presence).
    - DR-0003: pacman invocation and idempotency strategy.
    - DR-0004: mismatch policy (fail vs partial provision; exact selection).
    - DR-0005 (if needed to avoid implied protocol changes): provisioning execution isolation model (distinct world-agent request profile vs explicit guard rails; exactly one selection).
  - Must define (deterministic items): exactly two options (A/B) per DR; one selection per DR; the constraints each selection imposes; and the doc paths/sections that MUST be updated after selecting.
  - Links (non-authoritative): `contract.md` (final operator contract wording); slice specs (behavior constrained by each DR).

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS0/ANS0-spec.md` — slice spec (ADR-0033 C0: world OS package-manager probe)
  - Owns (authoritative): acceptance criteria for the in-world probe that derives OS family + available OS package manager for provisioning-time gating.
  - Must define (deterministic items):
    - Probe inputs: exact file paths read (at minimum `/etc/os-release`) and exact command(s) executed for manager presence checks.
    - Probe precedence: one total ordering for how `/etc/os-release` and manager presence are combined into a single “detected manager/family” result.
    - Classification rules: the exact Arch-family vs Debian/Ubuntu-family mapping rules (including behavior when keys are missing/unreadable).
    - Determinism rule: manager selection MUST be derived from in-world probe only and MUST NOT consult the host PATH.
    - Failure posture: exact behavior when the probe cannot determine a supported manager (exit code class + required remediation elements, via linkage to `contract.md`).
  - Links (non-authoritative): `contract.md`, `decision_register.md`.

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS1/ANS1-spec.md` — slice spec (ADR-0033 C1: pacman provisioning path)
  - Owns (authoritative): acceptance criteria for provisioning-time derivation and execution of `install.method=pacman` system-package requirements inside Arch-family worlds.
  - Must define (deterministic items):
    - Requirement derivation: enabled-set boundary, `install.method=pacman` filter, ordering + de-dup rules across multiple enabled items, and absence semantics.
    - Execution contract: exact non-interactive `pacman` invocation(s), idempotency definition, and the error→exit-code mapping (via linkage to `contract.md`).
    - `--dry-run` acceptance: asserts no `pacman` execution occurs when dry-run is set, and the planned actions printed match `contract.md` invariants.
    - Backend unsupported posture: exact behavior when provisioning is unsupported (exit code + required remediation elements, via linkage to `contract.md`).
  - Links (non-authoritative): `contract.md`, `decision_register.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` (APT provisioning analog for comparison only).

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS2/ANS2-spec.md` — slice spec (ADR-0033 C2: validation + operator docs updates)
  - Owns (authoritative): acceptance criteria for:
    - runtime fail-early behavior when system-package items are in scope (`apt` or `pacman`), and
    - deterministic updates to operator-facing docs/errors required by ADR-0033 (“manager-aware remediation; no apt-like guidance on non-APT worlds”).
  - Must define (deterministic items):
    - Runtime scope rule: exactly one trigger definition for the fail-early path across `deps current sync` and `deps current install` variants (enabled-set vs explicit args vs union).
    - Runtime outputs: required error/remediation content elements (must include the exact command string `substrate world enable --provision-deps`) and required exit-code class (via linkage to `contract.md`).
    - Test/fixture requirements: the minimum required unit + integration assertions implied by ADR-0033 Validation Plan (schema validation, requirement derivation, fail-early remediation).
    - Operator doc update acceptance: exact doc paths/headings to update (from `impact_map.md`) and the “link-to-contract.md; do not restate” rule.
  - Links (non-authoritative): `contract.md`, `decision_register.md`, `impact_map.md`.

- Validation artifacts (authoritative; required by ADR-0033):
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
    - Owns (authoritative): deterministic manual test cases and expected outputs/exit codes for provisioning + runtime fail-early across required platforms.
    - Must define (deterministic items): preconditions, exact commands, expected key output lines, expected exit codes, and explicit “no host OS mutation” checks where applicable.
    - Links (non-authoritative): `contract.md` (source of truth for expected behavior).
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
    - Owns (authoritative): automated validation steps per platform and pass/fail expectations aligned to `manual_testing_playbook.md`.

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0033 touches MUST appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | flags, defaults, success/no-op semantics, exit codes, remediation invariants, examples |
| Provisioning OS/manager probe (in-world; not host PATH-based) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS0/ANS0-spec.md` | exact probe inputs (`/etc/os-release`, manager presence checks), precedence, classification rules, behavior when ambiguous/missing |
| Provisioning requirement derivation (pacman packages from effective enabled world-deps set) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS1/ANS1-spec.md` | enabled-set boundary, `install.method=pacman` filter, de-dup rules, ordering, absence semantics |
| Pacman provisioning execution semantics (idempotency, mutation boundary, `pacman` invocation) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS1/ANS1-spec.md` | exact non-interactive invocation(s), idempotency definition, error→exit mapping, unsupported-backend posture |
| Provisioning `--dry-run` and `--verbose` contract (no-mutation semantics + output invariants) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact “no mutation” definition; minimum guaranteed content; stream requirements; stability posture |
| Provisioning APT invocation contract (when `install.method=apt` applies and provisioning is supported) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` | exact APT invocation contract, de-dup/ordering, error→exit mapping, `--dry-run` definition (APT path) |
| Runtime invariant: `substrate world deps current sync|install` MUST NOT invoke OS package managers (`apt` or `pacman`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | prohibited side effects; operator-visible rationale; linkage to provisioning command |
| Runtime fail-early operational semantics (scope rules + side-effect prohibitions) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS2/ANS2-spec.md` | exact trigger definition; enabled-set vs explicit-args handling; partial-apply rules (if any) |
| Runtime `--dry-run` and `--verbose` contract under fail-early posture | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | whether dry-run bypasses fail-early; minimum guaranteed output/remediation elements; exit code mapping |
| Runtime remediation message invariants (exact command string + unsupported-backend messaging) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | required exact command string `substrate world enable --provision-deps`; required “no host OS mutation” messaging on Linux host-native; stream requirements |
| Exit code meanings (`0/2/3/4/5`) for provisioning + runtime fail-early | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | mapping to taxonomy; per-command mapping; reserved meaning for exit `5` in this feature |
| Platform/backends support matrix (Linux host-native vs macOS Lima vs Windows WSL) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | supported/unsupported rules; guarantees about host OS mutation; Windows assumption posture |
| Protected paths / OS-mutation invariants | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | explicit “no host OS mutation” invariant; hardened-runtime write constraints; any explicitly permitted writable surfaces |
| Inventory schema (including `install.method=pacman` and `install.pacman` list shape) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | full schema update; constraints; canonical ordering rules (if any); explicit absence semantics |
| Effective enabled world-deps set resolution (inputs to provisioning/runtime checks) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | merge order, enabled-precedence, bundle-expansion boundary, determinism guarantees |
| World-agent execute/stream protocol baselines | `docs/WORLD.md` | `/v1/execute` and `/v1/stream` request/response shapes; transport notes; any explicit provisioning-execution isolation contract if introduced |
| Decision records DR-0001/DR-0002/DR-0003/DR-0004 (and DR-0005 if required) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` | A/B options; chosen selection; constraints; pointers to constrained spec sections |
| Manual validation | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md` | deterministic preconditions, exact commands, expected key output lines, expected exit codes |
| Smoke validation | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/*` | automated validation commands per platform; pass/fail expectations aligned to manual playbook |
| Slice acceptance (ANS0) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS0/ANS0-spec.md` | per-slice scope + acceptance criteria IDs for the probe surface |
| Slice acceptance (ANS1) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS1/ANS1-spec.md` | per-slice scope + acceptance criteria IDs for pacman provisioning |
| Slice acceptance (ANS2) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS2/ANS2-spec.md` | per-slice scope + acceptance criteria IDs for runtime fail-early + operator-doc updates |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes, error-message invariants where applicable) and failure posture
- Ordering/atomicity/concurrency rules (if any)
- Security/redaction invariants (if any)
- Platform guarantees (Linux/macOS/Windows/WSL as applicable)

## Follow-ups

Record missing/ambiguous ADR intent here (do not patch ADRs from this step).

1) ADR scope/links drift vs this feature directory
   - Issue: ADR-0033 declares feature dir `docs/project_management/packs/draft/world-deps-pacman-provisioning/` and lists related planning-pack docs under that path, but the actual feature dir for this run is `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`.
   - Required fix: during planning, reconcile to exactly one canonical feature directory and update all planning-pack doc links to match (without creating dual-authority contract/spec text across two dirs).

2) World OS family detection rules are underspecified
   - Issue: ADR-0033 requires using `/etc/os-release` (`ID`/`ID_LIKE`) plus manager presence checks, but does not define the exact mapping rules, precedence, or conflict resolution when inputs disagree.
   - Required fix: in `decision_register.md` (DR-0002) and `slices/ANS0/ANS0-spec.md`, define one deterministic algorithm, including:
     - exact Arch-family and Debian/Ubuntu-family allowlists (and whether `ID_LIKE` can override `ID`),
     - behavior when `/etc/os-release` is missing/unreadable,
     - behavior when `/etc/os-release` implies one family but `command -v <manager>` implies another.

3) Pacman invocation contract is underspecified
   - Issue: ADR-0033 requires provisioning via `pacman` but does not specify the exact non-interactive invocation, idempotency rules, or the mapping of `pacman` failures to exit codes.
   - Required fix: in `decision_register.md` (DR-0003) and `slices/ANS1/ANS1-spec.md`, define:
     - exact `pacman` command(s) and flags,
     - whether any db refresh step is required and its failure posture,
     - package-list ordering/de-dup rules and how they affect determinism,
     - error→exit mapping aligned to `contract.md`.

4) Provisioning mismatch policy needs a single deterministic rule
   - Issue: ADR-0033 requires failing when enabled system-package items do not match the detected manager, but does not define whether partial provisioning is ever allowed, nor how mixed `apt`+`pacman` enabled sets are handled.
   - Required fix: in `decision_register.md` (DR-0004), `contract.md`, and the relevant slice specs, define exactly one posture (fail-closed vs partial) and the exact remediation content.

5) Runtime fail-early “scope” is underspecified for `deps current install`
   - Issue: ADR-0033 states the runtime short-circuit triggers when the “effective enabled set contains `install.method=apt` or `install.method=pacman` items”, but the existing CLI includes `substrate world deps current install <ITEM...>` which can target explicit items.
   - Required fix: in `slices/ANS2/ANS2-spec.md` and `contract.md`, define exactly one scope rule (enabled-set vs explicit args vs union) and require tests to enforce it.

6) Provisioning execution isolation model is implied but not pinned
   - Issue: ADR-0033 architecture requires provisioning execution “without weakening hardened runtime execution” and mentions “distinct request profile or explicit guard rails”, but ADR-0033 Decision Summary does not record a selection.
   - Required fix: either:
     - add DR-0005 and record the selection (and update `docs/WORLD.md` if any wire contract changes), or
     - explicitly state (in `decision_register.md`) that no protocol change is required and that guard rails are implementable entirely within existing request shapes.

7) `--verbose` output is underspecified
   - Issue: ADR-0033 includes `--verbose` but does not define what additional information is guaranteed and on which stream(s).
   - Required fix: in `contract.md` define minimum guaranteed verbose elements (and any explicitly prohibited requirements), and in slice specs define where/when those elements are emitted.

8) Operator-doc update targets are not enumerated by exact path/headings
   - Issue: ADR-0033 requires updating operator reference and error text under `docs/reference/world/deps/…` but does not enumerate the exact file(s)/headings that must change.
   - Required fix: in `impact_map.md` and `slices/ANS2/ANS2-spec.md`, list the exact doc paths and headings, and require those docs to link to `contract.md` rather than restating the contract.

9) Cross-document contract ownership conflicts already exist and must be reconciled
   - Issue: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` currently specifies runtime `deps current sync|install` applying apt (“world image installs first (apt)”), which conflicts with ADR-0033 (“runtime MUST NOT invoke OS package managers (APT or pacman)”).
   - Required fix: during planning/implementation, reconcile the world-deps contract so there is exactly one authoritative truth for runtime behavior; it MUST either:
     - incorporate the provisioning-time-only system-package contract, or
     - explicitly defer to the provisioning feature’s `contract.md` for all system-package runtime/provisioning rules.
