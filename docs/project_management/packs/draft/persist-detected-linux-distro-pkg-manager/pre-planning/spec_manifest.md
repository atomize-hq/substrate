# persist-detected-linux-distro-pkg-manager — spec manifest

This file enumerates every contract surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

## Required spec documents (authoritative)

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md` — execution runbook + sequencing overview
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` — triad task graph + acceptance criteria (already exists)

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` — authoritative operator-facing contract for persisted host detection metadata:
  - Owns (authoritative): file location and lifecycle guarantees for `$SUBSTRATE_HOME/install_state.json`, platform scope (Linux-only), exit-code posture (no change; taxonomy reference), safety posture (best-effort metadata persistence), and consumer read-precedence guidance (persisted → runtime detection fallback).
  - Links to (non-authoritative): `install-state-schema-spec.md` (schema), `slices/PDL0/PDL0-spec.md` (acceptance + tests), `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (default exit codes), upstream detection contract (see “Coverage matrix”).

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` — authoritative schema spec for `$SUBSTRATE_HOME/install_state.json` (`schema_version=1`) including `host_state.platform.*`:
  - Owns (authoritative): full JSON schema (types, required/optional, constraints), additive-compatibility policy, unknown-field handling, canonicalization expectations (if any), and explicit absence semantics for each `host_state.platform.*` field.
  - Links to (non-authoritative): `contract.md` (file location + behavioral guarantees), upstream detection contract (source-of-truth for how `distro_id`/`distro_like` and `pkg_manager.*` values are derived).

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` — A/B decisions required to remove ambiguity in ADR-0032 for this feature directory:
  - Owns (authoritative): any remaining A/B decisions needed to make the persistence contract deterministic (examples in “Follow-ups”).
  - Links to (non-authoritative): `contract.md`, `install-state-schema-spec.md`, and slice specs that implement the selected decisions.

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDL0/PDL0-spec.md` — slice spec (persist detected platform metadata + assert via installer smoke):
  - Owns (authoritative): acceptance criteria for writing/updating `install_state.json` on successful Linux installs (including idempotency/atomicity posture), persisting the new `host_state.platform.*` keys, and extending installer smoke assertions to cover the new keys.
  - Links to (non-authoritative): `contract.md`, `install-state-schema-spec.md`, upstream detection contract, and the existing installer smoke script path (`tests/installers/install_state_smoke.sh`).

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0032 touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Feature scope (Linux-only behavior delta; macOS/Windows unchanged) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | exact platform applicability, and explicit “no-op on macOS/Windows” statement |
| Installer entrypoint in scope | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | script path(s) this feature’s contract applies to (at minimum `scripts/substrate/install-substrate.sh`) |
| CLI commands/flags/defaults | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | explicit “no new CLI commands or flags introduced by this feature” statement |
| File location + precedence: `$SUBSTRATE_HOME/install_state.json` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | how `$SUBSTRATE_HOME` is determined for installer runs (default + override), and the exact canonical path |
| Install success guarantee: `install_state.json` presence after successful Linux install | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | what “successful install” means for this guarantee (incl. `--dry-run` exclusion if applicable) and the failure posture when metadata cannot be persisted |
| Exit codes | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | reference to taxonomy + explicit “no exit code behavior changes introduced by this feature” |
| Data schema / file format: `install_state.json` (`schema_version=1`) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | full schema, additive policy, unknown-field handling, and explicit absence semantics |
| New persisted keys: `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type/constraints, when present vs absent, and the required linkage to `/etc/os-release` fields (`ID`, `ID_LIKE`) |
| New persisted keys: `host_state.platform.pkg_manager.selected`, `host_state.platform.pkg_manager.source` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type/constraints, when present vs absent, and explicit rule for “missing `/etc/os-release` still persists `pkg_manager.*` with fallback source” |
| Source-of-truth for detection inputs/outputs (`/etc/os-release` parsing and pkg-manager selection that produces `distro_id`/`distro_like` and `pkg_manager.*`) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact detection pipeline + precedence + allowed `pkg_manager.source` values; this feature MUST NOT redefine detection rules |
| Persistence filesystem semantics (write/update of `$SUBSTRATE_HOME/install_state.json`) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDL0/PDL0-spec.md` | write trigger, atomic-update posture, idempotency expectations, and failure posture alignment to `contract.md` |
| Safety / privacy invariants | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | explicit allowlist of persisted fields (no hostnames/env dumps); constraints ensuring only `/etc/os-release` `ID`/`ID_LIKE` and selected pkg-manager metadata are persisted under `host_state.platform.*` |
| Consumer read semantics (future consumers) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | “prefer persisted metadata for guidance strings; fall back to runtime detection when missing/unreadable” rule and non-authoritative linkage to future consuming surfaces |
| Acceptance criteria (persistence behavior + smoke assertions) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDL0/PDL0-spec.md` | deterministic acceptance criteria (including the exact smoke assertions that must be added/updated) |

## Determinism checklist (must be satisfied before quality gate)

For the docs above, confirm they explicitly define:
- Inputs and precedence (prefix/`$SUBSTRATE_HOME`, persisted metadata vs runtime fallback guidance).
- Defaults and absence semantics (missing/unreadable `/etc/os-release`; missing/unreadable `install_state.json`; optional new keys).
- Data model (schema) for every serialized boundary (`install_state.json`).
- Error model and failure posture (metadata write failures do not block install; exit code posture unchanged).
- Ordering/atomicity/idempotency rules (when writes occur; overwrite vs preserve; atomic update expectations).
- Security/redaction invariants (persisted field allowlist; no sensitive host info).
- Platform guarantees (Linux-only behavior delta; macOS/Windows unchanged).

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md`

Must define:
- The exact spec-doc set required by ADR-0032 for this feature directory.
- A coverage matrix that assigns every ADR-0032 contract surface to exactly one authoritative doc.
- Follow-ups required to remove ambiguity before quality gate.

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`

Must define:
- Exact touch set (files/dirs) implied by ADR-0032 (including installers and the relevant test script(s)).
- Cascading implications and cross-pack conflict scan results, including:
  - dependency on upstream detection work, and
  - any other packs that touch `install_state.json`.

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`

Must define:
- Slice sequencing (single explicit ordering) and rationale.
- The intended implementation touch points (by file path) consistent with ADR-0032.
- Validation commands to run (must include the `tests/installers/install_state_smoke.sh` run that asserts `install_state.json` presence + new keys).

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`

Must define:
- A task graph that includes at least one execution triad for `PDL0` (`PDL0-code`, `PDL0-test`, and `PDL0-integ-*`).
- For each task: explicit references to `slices/PDL0/PDL0-spec.md` and the authoritative contract docs (`contract.md`, `install-state-schema-spec.md`).

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

Must define:
- File contract:
  - Canonical location: `$SUBSTRATE_HOME/install_state.json` and how `$SUBSTRATE_HOME` is resolved for installer runs.
  - Guarantee: after a successful Linux install, the file exists (and when it is updated vs created).
  - Explicit statement for `--dry-run` (whether it is excluded from the “successful install” guarantee).
- Platform guarantees:
  - Linux: required behavior.
  - macOS/Windows: explicit “no change; no writes required” statement.
- Exit codes:
  - Exit code taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
  - Explicit statement that this feature introduces no new installer exit code behavior.
- Failure posture:
  - Deterministic rule for what happens when metadata cannot be read or written (and whether any operator-visible warning output is required).
- Consumer guidance contract:
  - A single deterministic precedence rule for future consumers: “prefer persisted metadata for guidance strings; fall back to runtime detection when missing/unreadable”.

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`

Must define:
- Format:
  - Format: JSON.
  - Canonical file name/location(s): `install_state.json` at `$SUBSTRATE_HOME/`.
- Compatibility:
  - `schema_version` remains `1`.
  - Additive semantics: older consumers MUST ignore unknown keys.
  - Unknown-field handling policy for readers (if any validation exists).
- Schema:
  - Full schema for `install_state.json` as used by installers/uninstallers (including the existing `host_state.{group,linger}` blocks and the new `host_state.platform.*` blocks).
  - Exact field types, constraints, and required/optional status for:
    - `host_state.platform.os_release.id`
    - `host_state.platform.os_release.id_like`
    - `host_state.platform.pkg_manager.selected`
    - `host_state.platform.pkg_manager.source`
  - `host_state.platform.pkg_manager.source` constraint MUST defer to upstream detection contract (`docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`) for the allowed enum values and their meaning; this schema spec MUST NOT redefine the enum set.
  - Absence semantics for each new field:
    - When `/etc/os-release` is missing/unreadable, `os_release.*` behavior is explicit and testable.
    - When package-manager detection is missing/unavailable, `pkg_manager.*` behavior is explicit and testable.
- Security / privacy:
  - Explicit allowlist of what may be persisted under `host_state.platform.*` (and an explicit denylist of common sensitive host details).
- Examples:
  - Minimal valid payload (including how “no platform metadata available” is represented).
  - Full valid payload (with platform metadata present).

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`

Must define (only if required to remove ambiguity; otherwise explicitly state “no decisions required”):
- Any A/B decisions needed to reconcile:
  - The “file must exist after successful Linux install” guarantee vs the “metadata persistence is best-effort and must not hard-fail” posture.
  - Any ambiguity in how `pkg_manager.source` values are defined/validated for persistence (including how this feature aligns with the upstream detection contract).
  - Any ambiguity in overwrite behavior when re-running the installer (preserve existing metadata vs overwrite with newly detected values).

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDL0/PDL0-spec.md`

Must define:
- Inputs:
  - Installer prefix/`$SUBSTRATE_HOME`.
  - Detection outputs to persist (and the authoritative source for how those outputs are derived).
- Persistence acceptance:
  - When the installer writes/updates `install_state.json` (must include the “no host-state events occurred” success case from ADR-0032).
  - Idempotency expectations (repeat runs do not corrupt or multiply state).
  - Atomicity posture (explicitly defined; e.g., write-then-rename).
  - Required behavior when `/etc/os-release` is missing/unreadable:
    - Detection behavior is owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
    - Persistence behavior in this slice MUST ensure `pkg_manager.*` is still persisted, and `pkg_manager.source` MUST match the upstream detection contract’s fallback path (e.g., `path_probe`).
- Test acceptance:
  - Update `tests/installers/install_state_smoke.sh` to assert the ADR-0032 acceptance criteria.
  - Exact smoke assertions required by ADR-0032:
    - `install_state.json` exists after successful Linux install.
    - New keys are present when `/etc/os-release` is available.
    - Missing `/etc/os-release` does not cause install failure and still records `pkg_manager.*` with an explicit fallback source.

## Follow-ups

Record missing/ambiguous ADR intent that must be resolved before the Planning Pack can pass quality gate:

- ADR-0032 “Scope” and “Related Docs” reference feature dir `docs/project_management/packs/draft/stashing-ferret/`, but this Planning Pack is running under `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`; reconcile naming/paths (and update ADR links during planning).
- ADR-0032 references dependency `detecting_badger`; the existing upstream detection Planning Pack is `docs/project_management/packs/draft/best-effort-distro-package-manager/` (ADR-0031). Planning must reconcile whether these are the same dependency and ensure this feature defers detection contracts to the upstream pack.
- ADR-0032 simultaneously requires (a) `install_state.json` exists after successful Linux install and (b) best-effort persistence that must not hard-fail solely due to metadata persistence; planning must make this non-contradictory and testable (likely via `decision_register.md` + explicit prerequisite/implementation strategy).
- ADR-0032 does not state whether the platform metadata must be written by `scripts/substrate/dev-install-substrate.sh` in addition to `scripts/substrate/install-substrate.sh`; planning must either scope it out explicitly or include it in the touched-script list in `contract.md` and `impact_map.md`.
