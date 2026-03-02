# best-effort-distro-package-manager — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- Spec manifest:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/best-effort-distro-package-manager"` (strict packs only).

### Create
- `tests/installers/pkg_manager_detection_test.sh`

### Edit
- `scripts/substrate/install-substrate.sh`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: Add `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` (Linux-only) and print a stable “distro + pkg-manager decision” one-liner before installing prerequisites.
  - Direct impact:
    - Operators can deterministically force the Linux prereq install path to use a specific manager, reducing brittle PATH-based selection.
    - Installer output gains a stable, support-friendly one-liner that makes the detection inputs and selected manager explicit.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` help/usage output and error messages must document the new flag and the legacy `PKG_MANAGER` override, including allowed values.
    - “Exactly one one-liner” requirement implies the installer must avoid emitting the line multiple times across internal prereq checks and must define whether the line is emitted when no prereq installation is needed.
    - The hermetic test must assert both precedence and one-liner content/stability (source enum + selected manager) to prevent drift.
  - Contradiction risks:
    - Multiple-manager PATH ambiguity: the ADR requires deterministic selection plus a warning listing alternates + override guidance, but the fixed precedence order is not yet pinned (must be decided and recorded; see DR-0002).
    - Output stability vs reuse: `scripts/substrate/world-enable.sh` sources `install-substrate.sh` and may call the prereq installer helpers; changes that print the decision line unconditionally can alter `world enable` helper UX (and can violate “exactly once” if invoked in multiple contexts) unless carefully scoped.

- Change: Enforce exit-code taxonomy for pkg-manager selection/override failures.
  - Direct impact:
    - Invalid override values fail with exit `2`; missing forced manager fails with exit `3`; inability to select any supported manager fails with exit `4` (per ADR-0031).
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` currently uses `exit 1` for many failures; implementing the ADR’s pkg-manager exit codes requires refactoring error handling so only the intended failure modes map to `2|3|4` without unintentionally reclassifying unrelated failures.
    - The hermetic test should assert exit codes for the override/selection failure modes (not only the selected manager) to keep the contract enforceable.
  - Contradiction risks:
    - Platform “no behavior change” vs exit codes: today the script exits `2` on Windows (“not yet implemented” warning). If the exit-code meanings are treated as global (not Linux-only), Windows should likely be `4` (“not supported”) per taxonomy. Specs must explicitly scope which exit-code meanings are Linux-only vs global to avoid hidden cross-platform behavior changes.

### Config / env vars / paths
- Change: Legacy env override `PKG_MANAGER=...` becomes an explicit part of the contract with deterministic precedence under `--pkg-manager`.
  - Direct impact:
    - Operators can continue to use `PKG_MANAGER` while gaining a higher-precedence flag for explicit override.
  - Cascading impact:
    - Precedence pipeline becomes a stable contract surface that must be consistent across all error messages, one-liner `source` values, and tests.
  - Contradiction risks:
    - Value vocabulary: existing users may set `PKG_MANAGER` to values not in `{apt-get,dnf,yum,pacman,zypper}` (e.g., `apt`), which will become a hard error (exit `2`) under the ADR contract.

- Change: Read `/etc/os-release` best-effort (safe parse; no `source`) and use `ID`/`ID_LIKE` for mapping + diagnostics.
  - Direct impact:
    - On Linux, default manager selection becomes more predictable (distro-family mapping) and diagnostics include the os-release inputs even when they are missing (`<unknown>` rendering).
  - Cascading impact:
    - Safe parsing rules must be explicit (quotes/whitespace/comments/duplicates/case-sensitivity) and must be shared across slices and tests to avoid drift.
    - Hermetic testing requires a deterministic mechanism to inject a fake os-release input (DR-0003); whichever test seam is chosen becomes a contract-bearing surface that must not weaken production safety posture.
  - Contradiction risks:
    - “Mapping matched but manager missing” semantics are underspecified (specs must choose: fall back to PATH probe vs fail closed, and which `pkg_manager_source`/exit code applies).

### Policy / isolation / security posture
- Change: Best-effort detection + override is fail-closed on explicit override but non-fatal for missing/unreadable `/etc/os-release`.
  - Direct impact:
    - Forced manager selection never silently falls back, reducing “installed the wrong thing” risk.
    - `/etc/os-release` read failures do not by themselves block installs; the installer can continue via other selection steps.
  - Cascading impact:
    - Error/warning outputs must avoid dumping full `/etc/os-release` contents (no accidental sensitive-data/log spam) and must not copy the `source /etc/os-release` pattern from container smoke checks.
  - Contradiction risks:
    - If selection falls through to PATH probing, the resulting behavior can still be surprising on mixed systems unless the warning + override guidance is consistent and test-asserted.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `/etc/os-release` parsing + canonicalization rules (`ID`, `ID_LIKE`)
    - pkg-manager identifier + `source` vocabulary (`flag|env|os_release|path_probe`)
  - Conflict: yes
  - Resolution (explicit):
    - Sequence + non-overlap boundary: `best-effort-distro-package-manager` owns detection/selection outputs and MUST remain the authoritative contract for parsing + selection; `ADR-0032` work persists those outputs to `install_state.json` and MUST NOT redefine the detection pipeline.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - `/etc/os-release` parsing and OS-family classification (world OS probe; Arch-family logic)
    - manager identifier vocabulary (`pacman`, etc.)
  - Conflict: no
  - Resolution (explicit):
    - Keep “host installer manager selection” distinct from “world OS probe” while reusing the same safe parsing/canonicalization rules where applicable to prevent drift between host detection, in-world detection, and persisted metadata.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (staged under `$SUBSTRATE_HOME/scripts/substrate/…` for dev-install helper discovery)
  - Conflict: no
  - Resolution (explicit):
    - Minimize refactors in `install-substrate.sh` that would complicate helper staging; treat the installer script as a stable artifact that other flows stage/copy, and keep behavior changes localized to pkg-manager selection logic + output.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` and `scripts/substrate/world-enable.sh` provisioning flows (Linux dev “enable later” ergonomics)
  - Conflict: no
  - Resolution (explicit):
    - Ensure pkg-manager detection changes do not alter the expectations around provisioning artifacts (`world-agent`) and do not introduce new side effects when the installer helpers are sourced by `world-enable.sh`.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - Installer responsibilities for writing `$SUBSTRATE_HOME/env.sh` and related env-script invariants
  - Conflict: no
  - Resolution (explicit):
    - This feature introduces no changes to `SUBSTRATE_*` env var semantics or env-script generation; keep the scope limited to Linux package-manager selection and `/etc/os-release` read-only detection.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - shared detection output fields/semantics (`distro_id`, `id_like`, `pkg_manager.*`, `source`)
  - Conflict: yes
  - Resolution (explicit):
    - Enforce a strict non-overlap boundary:
      - `best-effort-distro-package-manager` defines detection + selection outputs and explicitly MUST NOT persist `install_state.json`.
      - `persist-detected-linux-distro-pkg-manager` persists those outputs and MUST treat this pack’s `contract.md` as authoritative for parsing/selection semantics.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `/etc/os-release` parsing and canonicalization; distro-family classification vocabulary
  - Conflict: no
  - Resolution (explicit):
    - Align parsing/canonicalization rules and the Arch-family classification vocabulary to avoid contradictory behavior between host installer detection, in-world provisioning probes, and any persisted host metadata.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - shared exit-code taxonomy usage (`2|3|4`) and “no silent OS mutation” posture (in a different execution context)
  - Conflict: no
  - Resolution (explicit):
    - Keep concerns separate: this pack’s manager selection is host-installer-only; provisioning-time system package work owns the `substrate world enable --provision-deps` contract and must not introduce host PATH-based manager selection semantics.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — `/etc/os-release` parsing + canonicalization rules (no `source`; quotes/whitespace/duplicates/case-sensitivity).
  - DR-0002 — PATH probe ambiguity policy + fixed precedence order (warn vs fail; exact ordering; required warning content).
  - DR-0003 — Hermetic test seam for fake os-release input (test-only env var/arg vs harness approach; ensure no production safety weakening).
- Spec updates required (if any):
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` — pin the exact prereq command set to name in remediation guidance; pin “mapping matched but binary missing” behavior; pin when the one-liner is emitted (including “no prereqs needed” behavior).
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` — explicitly scope exit-code meanings as Linux-only vs global (to avoid accidental Windows/macOS behavior changes).
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/C2/C2-spec.md` — assert exit codes + one-liner content + precedence in the hermetic harness; forbid host mutation.
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md` — reconcile “Related Docs” link drift (`detecting-badger/*` vs `best-effort-distro-package-manager/*`) during planning.
