# world-deps-apt-provisioning — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`

## Required spec documents (authoritative)

List the exact spec documents that must exist under `docs/project_management/packs/draft/world-deps-apt-provisioning/`.

Each entry includes:
- what surfaces it owns (authoritative), and
- what it links to (non-authoritative).

Spec templates:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md`
  - Owns (authoritative): required doc set; surface→doc ownership; follow-ups (this file).
  - Must define (deterministic items): the exact required-doc list; a surface-complete coverage matrix; the determinism checklist gate for each selected doc.
  - Links (non-authoritative): ADR-0030; planning/spec standards.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/impact_map.md`
  - Owns (authoritative): touch set + cascading implications + cross-queue conflicts for slices `WDAP0`/`WDAP1`.
  - Must define (deterministic items): create/edit touch allowlists; cross-pack dependency/conflict notes (including `world-deps-packages-bundles-contract`); validation evidence requirements implied by touched contract surfaces.
  - Links (non-authoritative): all docs selected by this manifest.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`
  - Owns (authoritative): execution runbook + sequencing notes for slices `WDAP0`/`WDAP1` (including required validation commands).
  - Must define (deterministic items): sequencing overview; exact commands required for unit/integration/manual/smoke validation; required platform coverage; explicit gates for “world backend unavailable” vs “provisioning unsupported”.
  - Links (non-authoritative): `contract.md`; slice specs; `tasks.json`; `manual_testing_playbook.md`.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`
  - Owns (authoritative): triad task graph + `ac_ids` traceability for slices `WDAP0`/`WDAP1`.
  - Must define (deterministic items): task IDs and deps for each slice triad; references to the slice spec paths; acceptance criteria traceability (`ac_ids`) for strict mode.
  - Links (non-authoritative): slice specs; `plan.md`.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - Owns (authoritative): operator-facing contract introduced/changed by ADR-0030, including:
    - CLI provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]`.
    - CLI runtime behavior change: `substrate world deps current sync|install` MUST NOT invoke APT/dpkg; when APT-backed items are in scope the command exits non-zero with remediation.
    - Exit code mapping for provisioning/runtime flows (ADR-0030 subset of taxonomy: 0/3/4/5).
    - Operator-visible remediation invariants, including the required exact command string `substrate world enable --provision-deps`.
    - Platform/backends support matrix (Linux host-native vs macOS Lima vs Windows WSL) for `--provision-deps` and for runtime remediation behavior.
    - Protected paths/invariants referenced by ADR-0030 (no host OS mutation; hardened runtime remains fail-closed).
  - Must define (deterministic items):
    - Success/no-op semantics for provisioning when the effective enabled set contains zero `install.method=apt` items.
    - The exact exit-code mapping per command and per failure class (world backend unavailable vs unsupported provisioning vs unmet prerequisites).
    - The minimum guaranteed remediation content for every unsupported/unmet-prereq path (including “no host OS mutation” messaging on Linux host-native).
    - The exact definition of “APT-backed world-deps are provisioning-time” as an operator-visible invariant (what runtime actions are prohibited).
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (world-deps inventory + enabled-resolution schema; `install.method=apt` item shape)
    - `docs/WORLD.md` (world-agent `/v1/execute` and `/v1/stream` baselines; transport context only)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
  - Owns (authoritative): the decision records referenced by ADR-0030:
    - DR-0001: Option A vs Option B selection (record ADR selection as the single decision outcome).
    - DR-0002: provisioned-state tracking (A: probe-only; B: persisted state file).
    - DR-0003: provisioning execution profile isolation model (exact A/B split; one selection).
  - Must define (deterministic items): exactly two options (A/B) per DR; one selection per DR; the constraints each selection imposes; and the doc paths/sections that must be updated after selecting.
  - Links (non-authoritative): `contract.md` (final operator contract wording); slice specs (the behavior constrained by each DR).

- Slice specs (canonical; feature-derived slice IDs)
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` — provisioning surface for APT requirements (ADR-0030 C0)
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` — runtime fail-early + remediation for APT items (ADR-0030 C1)

- Validation artifacts (authoritative; required by ADR-0030):
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/windows-smoke.ps1`

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0030 touches MUST appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]` | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | flags, defaults, success/no-op semantics, exit codes, remediation guarantees, examples |
| Provisioning requirement derivation (APT packages from effective enabled world-deps set) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` | bundle expansion boundary, `install.method=apt` filter, de-dup rules, ordering, absence semantics |
| Provisioning execution semantics (idempotency, mutation boundary, APT invocation, `--dry-run`, `--verbose`) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` | exact “no mutation” definition for `--dry-run`; idempotency definition; exact APT invocation contract; error→exit mapping; unsupported-backend posture |
| Runtime invariant: `substrate world deps current sync|install` MUST NOT invoke APT/dpkg | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | prohibited side effects; operator-visible rationale; linkage to provisioning command |
| Runtime fail-early operational semantics (scope rules + side-effect prohibitions) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` | “fail early” definition; whether scope is enabled-set vs explicit items; behavior for `--all`; whether non-APT items are skipped vs partially applied |
| Runtime remediation message invariants (exact command string + unsupported-backend messaging) | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | required exact command string `substrate world enable --provision-deps`; required “no host OS mutation” wording on Linux host-native; stream requirements |
| Exit code meanings (0/3/4/5) for provisioning + runtime short-circuit | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | mapping to taxonomy; per-command mapping; “reserved” meaning for exit `5` in this feature |
| Platform/backends support matrix (Linux host-native vs macOS Lima vs Windows WSL) | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | supported/unsupported rules; guarantees about host OS mutation; Windows assumption posture |
| Protected paths / OS-mutation invariants | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | explicit “no host OS mutation” invariant; hardened-runtime write constraints; any explicitly permitted writable surfaces |
| Decision records DR-0001/DR-0002/DR-0003 | `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md` | A/B options; chosen selection; constraints; pointers to constrained spec sections |
| World-deps inventory schema + enabled-resolution model (inputs) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | inventory file schema (including `install.method=apt`); bundle expansion semantics; enabled resolution precedence |
| World-agent execute/stream protocol baselines (dependency) | `docs/WORLD.md` | `/v1/execute` and `/v1/stream` request/response shapes; transport notes; capability/doctor endpoints |
| Manual validation | `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md` | deterministic preconditions, exact commands, expected key output lines, expected exit codes |
| Smoke validation | `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/*` | automated validation commands per platform; pass/fail expectations aligned to manual playbook |
| Slice acceptance (WDAP0) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md` | per-slice scope + AC-WDAP0-* acceptance criteria |
| Slice acceptance (WDAP1) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md` | per-slice scope + AC-WDAP1-* acceptance criteria |

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

1) ADR “Related Docs” path drift for the world-deps contract
   - Issue: ADR-0030 links `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`, but the contract currently lives at `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`.
   - Required fix: update ADR-0030 links (and any planning-pack docs) to the correct path and ensure there is exactly one authoritative contract source for world-deps schema.

2) Runtime fail-early “scope” is underspecified for `deps current install`
   - Issue: ADR-0030 states the runtime short-circuit triggers when the “effective enabled set contains `install.method=apt` items”, but the existing CLI includes `substrate world deps current install <ITEM...>` which can target explicit items.
   - Required fix: in `slices/WDAP1/WDAP1-spec.md`, define exactly one scope rule for fail-early (enabled-set vs explicit args vs union) and require tests to enforce it.

3) Provisioning APT invocation contract is underspecified
   - Issue: ADR-0030 requires “install/ensure APT packages” but does not define the exact non-interactive invocation (update/install flags), ordering, de-duplication, or the mapping of APT failures to exit codes.
   - Required fix: decide any open pieces in `decision_register.md` (DR-0002/DR-0003 as needed) and define the full invocation and error mapping in `slices/WDAP0/WDAP0-spec.md` and `contract.md`.

4) `--verbose` behavior is underspecified
   - Issue: ADR-0030 lists `--verbose` on provisioning, and runtime surfaces already accept `--verbose`, but the contract does not define what additional information is emitted and on which stream(s).
   - Required fix: define `--verbose` output invariants (what is guaranteed vs best-effort) in `contract.md` and the per-command behavior details in the relevant slice spec(s).

5) Provisioned-state tracking (probe-only vs state file) implies a potential new file-format surface
   - Issue: ADR-0030 includes DR-0002 but does not define where a state file would live, whether it is host-side or world-side, nor its schema/compat posture.
   - Required fix: in `decision_register.md` DR-0002, select exactly one approach; if a state file is selected, the chosen path and schema MUST be defined in `contract.md` (and referenced by the slice spec) to avoid an implied file-format surface.

6) Linux host-native “unsupported by default” is ambiguous
   - Issue: ADR-0030 states provisioning is “unsupported by default” on Linux host-native but does not define whether v1 permits any override/escape hatch.
   - Required fix: in `contract.md`, state a single deterministic v1 posture: either “no override exists” or “override exists and is defined” (name/type/default/guardrails).

7) Operator-doc update targets are not enumerated by exact path
   - Issue: ADR-0030 requires updating `docs/reference/world/deps/…` but does not specify the exact file(s) and headings to update.
   - Required fix: in `impact_map.md` and `plan.md`, list the exact doc paths to update and ensure they link back to `contract.md` rather than redefining the contract text.

8) Cross-document contract ownership must be reconciled for runtime `world deps` APT behavior
   - Issue: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` currently describes runtime `deps current sync|install` applying “world image installs first (apt)”, which conflicts with ADR-0030’s contract change (“runtime MUST NOT invoke APT/dpkg”).
   - Required fix: during planning/implementation, update the world-deps contract doc so it does not define conflicting runtime APT behavior; it MUST either (a) fully incorporate the new provisioning-time contract, or (b) explicitly defer to `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` for the APT-related runtime/provisioning rules.
