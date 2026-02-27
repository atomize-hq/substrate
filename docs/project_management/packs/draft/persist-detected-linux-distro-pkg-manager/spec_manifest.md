# persist-detected-linux-distro-pkg-manager — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

## Required spec documents (authoritative)

List the exact spec documents that must exist under `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.

Each entry includes:
- what surfaces it owns (authoritative), and
- what it links to (non-authoritative).

Spec templates:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/spec_manifest.md` — spec selection + ownership map (this file)
  - Owns (authoritative): required doc set; surface→doc ownership; follow-ups.
  - Must define (deterministic items): the exact required-doc list; a surface-complete coverage matrix; the determinism checklist gate for each selected doc.
  - Links (non-authoritative): ADR(s); planning/spec standards.

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/impact_map.md`
  - Owns (authoritative): touch set allowlists + derived risk notes for installer + test harness touches for slices `C0`/`C1`/`C2`.
  - Must define (deterministic items): create/edit touch allowlists; cross-pack dependency/conflict notes; validation evidence requirements implied by touched contract surfaces.
  - Links (non-authoritative): all spec docs selected by this manifest.

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
  - Owns (authoritative): execution runbook + sequencing notes for slices `C0`/`C1`/`C2` (including required validation commands).
  - Must define (deterministic items): sequencing gates (including the `detecting_badger` prerequisite); exact validation commands for Linux installer smoke coverage; a pointer to `contract.md` for the operator-visible contract delta.
  - Links (non-authoritative): `contract.md`; `install-state-schema-spec.md`; slice specs; `tasks.json`.

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`
  - Owns (authoritative): triad task graph + `ac_ids` traceability for slices `C0`/`C1`/`C2`.
  - Must define (deterministic items): task IDs and deps for each slice triad; references to the slice spec paths; acceptance criteria traceability (`ac_ids`) for strict mode.
  - Links (non-authoritative): slice specs; `plan.md`.

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
  - Owns (authoritative): the decision records referenced by ADR-0032:
    - DR-0001: metadata persistence location (A: extend `$SUBSTRATE_HOME/install_state.json`; B: separate `$SUBSTRATE_HOME/host_platform.json`).
    - DR-0002: field naming + nesting for the persisted platform keys (confirm/lock the JSON path set).
    - DR-0003: `host_state.platform.pkg_manager.source` enum set + semantics (exact allowed values; mapping to detection inputs).
  - Must define (deterministic items): exactly two options (A/B) per DR; one selection; the constraints each selection imposes; and the doc paths/sections that must be updated after selecting.
  - Links (non-authoritative): `contract.md` (final contract wording); `install-state-schema-spec.md` (schema + compatibility truth).

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - Owns (authoritative): the operator-facing installer contract (Linux only) for persisting detected distro/pkg-manager metadata to `install_state.json`, including:
    - the persisted file path: `$SUBSTRATE_HOME/install_state.json` (with `SUBSTRATE_HOME` resolution linked, not re-defined)
    - success-path presence: “after a successful Linux install, `install_state.json` exists”
    - failure posture: inability to read `/etc/os-release` and/or write `install_state.json` MUST NOT make the install fail solely due to metadata persistence
    - the write/update rule at a contract level: “write at least once per successful install; idempotent updates”
    - platform guarantees: Linux-only write of `host_state.platform.*`; macOS/Windows MUST NOT gain these fields from this work
    - protected-path invariants: writes MUST remain scoped to `$SUBSTRATE_HOME`; must not write outside that directory
    - exit codes: taxonomy reference; explicit statement that this feature introduces no new exit code meanings
  - Must define (deterministic items):
    - the exact “successful install” boundary that triggers the required write (success-only vs also on partial failure)
    - the exact update semantics when `install_state.json` already exists (merge vs overwrite for `host_state.platform.*`; treatment of unrelated keys)
    - the exact operator-facing warning posture (if any) when metadata write fails (stderr content constraints + “warnings do not change exit code”)
    - a normative link to `install-state-schema-spec.md` as the authoritative schema for `host_state.platform.*`
  - Links (non-authoritative): `install-state-schema-spec.md`; `docs/reference/env/contract.md` (`SUBSTRATE_HOME`); `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - Owns (authoritative): the on-disk schema and interpretation rules for the `install_state.json` extension introduced by ADR-0032:
    - `schema_version` invariant (`1`)
    - `host_state.platform.os_release.{id,id_like}` value rules (types, optionality, canonicalization)
    - `host_state.platform.pkg_manager.{selected,source}` value rules (types, optionality, canonicalization)
    - backward/forward compatibility policy for additive keys and unknown-field handling for older uninstallers/consumers
    - consumer read semantics for these fields (“prefer persisted metadata for guidance strings; fallback to runtime detection when missing/unreadable”)
  - Must define (deterministic items):
    - canonical file name + canonical location(s)
    - the exact JSON schema for `host_state.platform.*` (types; required/optional; absence semantics)
    - the exact allowed value set for `pkg_manager.source` (must be an explicit finite set; no “e.g.” lists)
    - the mapping from `pkg_manager.source` values to detection inputs (flag/env/os_release/path probe)
    - canonicalization for `os_release.id_like` (explicitly: raw string semantics; whitespace/quoting handling)
    - examples: minimal valid and full valid payloads containing the new keys
    - security constraints: explicit non-inclusion of sensitive host fields (hostnames, env dumps) and logging/redaction posture for these keys
  - Links (non-authoritative): `docs/project_management/system/templates/spec/schema-spec.md.tmpl`; `decision_register.md`; `docs/INSTALLATION.md` (overview; not authoritative once this spec exists).

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C0/C0-spec.md`
  - Owns (authoritative): slice `C0` acceptance for persisting platform detection fields.
  - Must define (deterministic items): how `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` are populated on Linux success paths; absence semantics when `/etc/os-release` is missing/unreadable; AC-C0-* acceptance criteria IDs.
  - Links (non-authoritative): `contract.md`; `install-state-schema-spec.md`; upstream pkg-manager detection contract (dependency).

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C1/C1-spec.md`
  - Owns (authoritative): slice `C1` acceptance for ensuring `install_state.json` is reliably present post-install.
  - Must define (deterministic items): the required post-success guarantee that `install_state.json` exists after a successful Linux install even when no group/linger events occurred; idempotency expectations across repeated runs; AC-C1-* acceptance criteria IDs.
  - Links (non-authoritative): `contract.md`; `install-state-schema-spec.md`.

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C2/C2-spec.md`
  - Owns (authoritative): slice `C2` acceptance for installer smoke assertions covering the new keys.
  - Must define (deterministic items): the test(s) to extend (e.g., `tests/installers/install_state_smoke.sh`); required assertions and skip conditions; negative-case behavior for missing `/etc/os-release`; AC-C2-* acceptance criteria IDs.
  - Links (non-authoritative): `contract.md`; `install-state-schema-spec.md`; `plan.md`.

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0032 touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI commands/flags/defaults (no changes) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | explicit statement of “no new CLI surface”; ensure no implied flags are introduced by tests/docs |
| Exit code meanings (no changes) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | taxonomy reference + explicit statement that this feature introduces no new exit code meanings |
| Env var `SUBSTRATE_HOME` resolution | `docs/reference/env/contract.md` | name, type, default (`~/.substrate`), precedence; empty treated as unset |
| Installer metadata file path | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | canonical location `$SUBSTRATE_HOME/install_state.json`; Linux-only write guarantee |
| `install_state.json` schema version invariant | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | `schema_version` MUST remain `1`; unknown keys MUST be safe to ignore for older consumers |
| `host_state.platform.os_release.id` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type, optionality, canonicalization, absence semantics, source input (`/etc/os-release` `ID`) |
| `host_state.platform.os_release.id_like` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type, optionality, canonicalization, absence semantics, source input (`/etc/os-release` `ID_LIKE`, raw-string contract) |
| `host_state.platform.pkg_manager.selected` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type, optionality, absence semantics; stability rule: MUST match the pkg-manager identifier contract used by the installer |
| `host_state.platform.pkg_manager.source` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | exact finite enum set; mapping to detection inputs; fallback source when `/etc/os-release` is missing/unreadable |
| `/etc/os-release` parsing rules (dependency) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | safe parsing posture (must not `source`); normalization for `ID`/`ID_LIKE`; absence semantics |
| Pkg-manager selection rules (dependency) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | allowed manager identifiers; selection precedence; PATH-probe tie-breakers; “no new manager support” invariant |
| Required write timing | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | “write at least once per successful Linux install” and what qualifies as “successful” |
| Update/idempotency semantics | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | merge/overwrite rules for `host_state.platform.*`; behavior on repeated installs |
| Failure posture for metadata persistence | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | best-effort write; warnings posture; install success MUST NOT be blocked solely by persistence failures |
| Sensitive-data restrictions (persisted + logged) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | explicit allowlist of persisted keys; explicit non-inclusion of sensitive host fields; logging/redaction rules |
| Platform parity/divergence | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | Linux-only behavior; macOS/Windows “no behavior delta” guarantees; required validation evidence |
| Slice acceptance (C0) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C0/C0-spec.md` | per-slice scope + AC-C0-* acceptance criteria |
| Slice acceptance (C1) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C1/C1-spec.md` | per-slice scope + AC-C1-* acceptance criteria |
| Slice acceptance (C2) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/C2/C2-spec.md` | per-slice scope + AC-C2-* acceptance criteria |
| Decision records DR-0001/DR-0002/DR-0003 | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` | A/B options; chosen selection; rationale; pointers to the spec sections constrained by the selection |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes, warning posture) and failure posture
- Ordering/atomicity/concurrency rules (if any)
- Security/redaction invariants (if any)
- Platform guarantees (Linux/macOS/Windows/WSL as applicable)

## Follow-ups

Record missing/ambiguous ADR intent here (do not patch ADRs from this step).

1) ADR “Related Docs” directory drift
   - Issue: ADR-0032 “Scope” and “Related Docs” point to `docs/project_management/packs/draft/stashing-ferret/...`, but the resolved feature directory is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
   - Required fix: update ADR-0032 links to the correct feature directory paths after this manifest is accepted.

2) `pkg_manager.source` enum is underspecified
   - Issue: ADR-0032 lists example values (`flag|env|os_release|path_probe`) but does not define the exact finite set, nor the exact mapping rules.
   - Required fix: decide and lock the exact enum set + semantics in `decision_register.md` (DR-0003) and define it in `install-state-schema-spec.md`.

3) Value canonicalization rules for `os_release.id_like` must be pinned
   - Issue: ADR-0032 calls `ID_LIKE` a “raw string”, but does not define canonicalization (quote stripping, whitespace normalization, comment handling) for persisted values.
   - Required fix: define exact canonicalization rules in `install-state-schema-spec.md` and ensure they match the upstream `/etc/os-release` parsing contract.

4) Write/update semantics for `install_state.json` need an explicit merge model
   - Issue: ADR-0032 requires idempotent updates and “write at least once per successful install”, but does not specify overwrite vs merge rules for `host_state.platform.*` vs unrelated keys, nor the posture for corrupted pre-existing JSON.
   - Required fix: define the merge model + corrupted-file posture (warn/overwrite/backup) in `contract.md` and align it with `docs/INSTALLATION.md` warning-only posture.

5) Upstream dependency contract must be treated as authoritative
   - Issue: ADR-0032 depends on `detecting_badger` landing first, but does not specify which document is the final contract for pkg-manager selection and `/etc/os-release` parsing.
   - Required fix: in `plan.md`, explicitly gate this work on the upstream contract doc (expected: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`) and treat that document as authoritative for selection/parsing surfaces.

