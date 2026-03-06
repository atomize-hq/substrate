# persist-detected-linux-distro-pkg-manager — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- Spec manifest:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager"` (strict packs only).

### Create
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

### Edit
- `docs/INSTALLATION.md`
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/install-substrate.sh`
- `tests/installers/install_smoke.sh`
- `tests/installers/install_state_smoke.sh`

### Deprecate
- None

### Delete
- None

## Surface scope note

No `crates/`, `src/`, `crates/world*`, `crates/shim`, `crates/shell`, or `crates/world-agent` path is directly implied by ADR-0032 itself. The behavior delta is installer-script scoped; later consumer features may read the persisted metadata, but that work is explicitly out of scope for this ADR.

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: Successful Linux installs gain a stable post-install artifact at `<resolved SUBSTRATE_HOME>/install_state.json` even when no group/linger event occurred.
  - Direct impact:
    - Operators and support now have a deterministic file to inspect after a successful Linux install instead of an event-only artifact that may be absent.
    - The successful-install contract changes for `--no-world` runs too; a Linux install that skips world provisioning still needs the same metadata surface.
  - Cascading impact:
    - `scripts/substrate/install-substrate.sh` must stop tying metadata writes to `world_enabled` or to the presence of group/linger event payloads.
    - `tests/installers/install_state_smoke.sh` must assert file creation when no host-state event occurred and when `--no-world` is used.
    - `docs/INSTALLATION.md` must stop describing `install_state.json` as only a group/linger record and must state the Linux-only file-presence guarantee.
  - Contradiction risks:
    - `scripts/substrate/install-substrate.sh` currently calls `write_host_state_metadata "${world_enabled}"`, which skips metadata writes on `--no-world`.
    - The current writer also exits early when no event payload exists, which contradicts ADR-0032’s “reliably present after successful install” contract.

- Change: The `install_state.json` contract stays shared across the production and dev installers.
  - Direct impact:
    - Operators who use `scripts/substrate/dev-install-substrate.sh` and operators who use `scripts/substrate/install-substrate.sh` keep seeing one meaning for the same file path instead of diverging metadata shapes.
  - Cascading impact:
    - Dev-install validation must join the scope: `tests/installers/install_smoke.sh` is the existing harness that exercises the dev installer and therefore becomes the required validation surface for this scope selection.
    - `docs/INSTALLATION.md` must continue to describe one install-state surface for both installers, while still stating that the new `host_state.platform.*` write contract is Linux-only.
  - Contradiction risks:
    - `docs/INSTALLATION.md` already states that both installers record host-state details in `install_state.json`.
    - `scripts/substrate/dev-install-substrate.sh` already contains a parallel host-state writer, so a production-only expansion would create two incompatible meanings for the same file.
  - Scope resolution options:
    - Option A: edit both `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` so the install-state contract stays singular.
    - Option B: limit ADR-0032 to `scripts/substrate/install-substrate.sh` and narrow the operator docs so dev installs remain event-only.
    - Selected: Option A. The current product contract already exposes `install_state.json` through both installers, so leaving dev-install divergent would create silent contract drift for the same on-disk file.

### Config / env vars / paths
- Change: ADR-0032 adds `host_state.platform.*` under the existing `schema_version=1` file instead of creating a new metadata file.
  - Direct impact:
    - The resolved metadata path stays `<resolved SUBSTRATE_HOME>/install_state.json`; operators do not learn a second file location.
    - Existing cleanup-state readers continue using the same file path and only need additive compatibility.
  - Cascading impact:
    - The feature-local `install-state-schema-spec.md` must define the exact JSON nesting and preservation rules for `host_state.group` and `host_state.linger`.
    - The feature-local `compatibility-spec.md` must define wrong-schema and corrupt-file handling so older uninstall flows keep working.
    - `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` must preserve existing group/linger content when writing platform fields.
  - Contradiction risks:
    - Introducing a second file such as `host_platform.json` would contradict the ADR selection and fragment the operator-facing metadata surface.
    - Any schema bump above `1` would break the explicit compatibility invariant carried forward from the existing uninstall readers.

- Change: Persisted `pkg_manager.selected` and `pkg_manager.source` are storage outputs owned by ADR-0032, but the allowed values and derivation rules remain external.
  - Direct impact:
    - The persisted values must exactly mirror the manager names and `source` enum already defined by `best-effort-distro-package-manager`.
  - Cascading impact:
    - `contract.md` and `install-state-schema-spec.md` must link to `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` instead of restating the detection algorithm.
    - `scripts/substrate/install-substrate.sh` must capture the final selected manager/source from the detection pipeline without redefining that pipeline inside ADR-0032.
  - Contradiction risks:
    - A persistence-specific enum or renamed manager vocabulary would make installer output, persisted metadata, and later in-world provisioning docs disagree.
    - Re-parsing `/etc/os-release` differently inside ADR-0032 would split “selection semantics” from “persistence semantics” even though both live in the same script.

- Change: Missing `/etc/os-release` input remains a best-effort case; JSON persistence uses omission semantics rather than sentinel strings.
  - Direct impact:
    - When `/etc/os-release` is missing or unreadable, the install still succeeds and can still persist `pkg_manager.*`; the unavailable `os_release.*` keys do not need placeholder strings in the JSON payload.
  - Cascading impact:
    - `install-state-schema-spec.md` must define exact omission rules for `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like`.
    - `tests/installers/install_state_smoke.sh` must assert the selected omission rule explicitly.
  - Contradiction risks:
    - Persisting literal `<unknown>` strings would conflate ADR-0031’s stderr rendering contract with ADR-0032’s stored JSON contract.
    - Persisting `null` or placeholder values without a written contract would create drift across future consumers.
  - Serialization resolution options:
    - Option A: omit unavailable `host_state.platform.os_release.*` keys while still persisting the available `pkg_manager.*` fields.
    - Option B: persist sentinel values such as `<unknown>` or `null` for unavailable `os_release.*` keys.
    - Selected: Option A. ADR-0032 defines these fields as additive and required only when inputs are available; the exact `<unknown>` rendering rule belongs to operator output, not to stored JSON.

### Policy / isolation / security posture
- Change: Metadata persistence stays best-effort, Linux-only, and confined to the resolved `SUBSTRATE_HOME`.
  - Direct impact:
    - A metadata read/merge/write failure must not turn a successful install into a failed install.
    - macOS and Windows remain explicit no-delta platforms for the new `host_state.platform.*` contract.
  - Cascading impact:
    - The feature-local `contract.md` must pin one exact no-fail posture for read/merge/write errors and one exact dry-run rule.
    - The feature-local `platform-parity-spec.md` must state Linux as the only behavior delta and require explicit no-delta evidence for macOS and Windows.
    - The implementation must continue writing only under `<resolved SUBSTRATE_HOME>` and must not add broader host inspection or any sensitive payload beyond `/etc/os-release` fields and selected manager identifiers.
  - Contradiction risks:
    - A hard-fail write posture would contradict both the ADR and the current cleanup-state metadata behavior.
    - Provisioning packs that also edit `scripts/substrate/install-substrate.sh` would couple metadata persistence to helper-sourcing or world-enable behavior if the write path stops being install-only and side-effect free when sourced.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - persisted values for `host_state.platform.pkg_manager.selected`
    - persisted values for `host_state.platform.pkg_manager.source`
    - `/etc/os-release` input vocabulary
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0031 owns the detection algorithm, canonical manager names, and `source` enum semantics.
    - ADR-0032 persists the final outputs and MUST NOT redefine detection/parsing rules.
    - Sequencing boundary: land or adopt the ADR-0031 contract first so ADR-0032 stores a single authoritative vocabulary.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes
  - Resolution (explicit):
    - Keep host install-state persistence isolated from world-deps provisioning semantics.
    - If ADR-0030 edits installer messaging in the shared script, it must not refactor the detection/persistence path owned by ADR-0031 and ADR-0032.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - shared manager identifiers (`apt-get`, `dnf`, `yum`, `pacman`, `zypper`)
    - `/etc/os-release` family vocabulary used in operator-facing guidance
  - Conflict: no
  - Resolution (explicit):
    - Keep host-install metadata and in-world provisioning metadata as separate concepts.
    - Reuse the same canonical manager identifiers so host guidance and world guidance do not drift.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0034 owns helper staging/discovery.
    - ADR-0032 owns install-state metadata parity for the dev installer.
    - Sequence or rebase so ADR-0032 makes the smallest possible metadata-only edits after helper-staging changes land.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `tests/installers/install_smoke.sh`
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0035 owns world-agent staging and “enable later” readiness.
    - ADR-0032 owns install-state persistence parity.
    - Keep the dev-installer edits orthogonal: no change to staged world-agent semantics, only metadata persistence contract updates and validation.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - `SUBSTRATE_HOME` path semantics for the install-state file
  - Conflict: no
  - Resolution (explicit):
    - Continue treating `SUBSTRATE_HOME` meaning and default path resolution as externally authoritative via `docs/reference/env/contract.md`.
    - ADR-0032 introduces no new env/config precedence surface.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - detection outputs consumed by `host_state.platform.pkg_manager.*`
  - Conflict: yes
  - Resolution (explicit):
    - Single-authority rule:
      - `best-effort-distro-package-manager` owns detection/selection/parsing/`source` semantics.
      - `persist-detected-linux-distro-pkg-manager` persists those outputs and MUST treat the other pack’s `contract.md` as authoritative.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes
  - Resolution (explicit):
    - Keep host metadata persistence and schema evolution isolated to this pack.
    - Provisioning-pack changes in the shared script must not refactor install-state write behavior or detection-state capture.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - manager identifier vocabulary
    - `docs/INSTALLATION.md` terminology about host vs world package managers
  - Conflict: no
  - Resolution (explicit):
    - Use one manager vocabulary across packs, but keep “host installer package manager” and “world OS package manager” explicitly distinct in docs.

- Planning Pack: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
  - Conflict: yes
  - Resolution (explicit):
    - Do not couple ADR-0032 to helper-staging path changes.
    - If the packs overlap in time, keep ADR-0032 to metadata-only edits and let helper-discovery remain the path/discovery authority.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `tests/installers/install_smoke.sh`
  - Conflict: yes
  - Resolution (explicit):
    - Keep the shared-script merge surface narrow: install-state persistence parity only.
    - Avoid changing the meaning of `--no-world` beyond the metadata contract selected here.

- Planning Pack: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
  - Overlap surfaces:
    - later consumers that may read the persisted install-state metadata for operator guidance
  - Conflict: no
  - Resolution (explicit):
    - Non-overlap boundary: ADR-0032 only persists the metadata contract.
    - Later doctor/health work may consume it, but must preserve the fallback-to-runtime-detection rule when the file is missing or unreadable.

- Archived reference: `docs/project_management/_archived/p0-platform-stability/`
  - Overlap surfaces:
    - original `install_state.json` contract
    - `schema_version=1`
    - `host_state.group` / `host_state.linger`
    - `tests/installers/install_state_smoke.sh`
  - Conflict: no
  - Resolution (explicit):
    - Treat the archived pack as provenance for the existing compatibility contract.
    - Extend the file additively only; do not reopen cleanup-state semantics or schema-version policy.

- Archived reference: `docs/project_management/_archived/p0-platform-stability-macOS-parity/`
  - Overlap surfaces:
    - macOS installer parity documentation around install-state metadata
  - Conflict: no
  - Resolution (explicit):
    - Keep ADR-0032 Linux-only.
    - Preserve macOS as an explicit no-delta platform for `host_state.platform.*` writes unless a separate ADR changes that contract.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Pin exact persistence behavior when platform metadata inputs are incomplete.
  - DR-0002 — Pin exact dry-run semantics for install-state persistence.
  - DR-0003 — Record the installer-entrypoint scope decision selected in this impact map (`install-substrate.sh` + `dev-install-substrate.sh`).

- Spec updates required:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` — define the Linux-only file-presence guarantee, the selected dual-installer scope, the `--no-world` rule, the dry-run rule, the no-fail write posture, and the downstream read fallback contract.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` — define exact `host_state.platform.*` nesting, omission rules for unavailable `os_release.*` fields, and the preservation rule for existing group/linger content while linking externally to the detection contract.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md` — define merge/reset behavior for corrupt or wrong-schema files and restate the `schema_version=1` invariant.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md` — define Linux as the only behavior delta and capture explicit no-delta evidence expectations for macOS and Windows.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md` — include exact validation commands for `tests/installers/install_state_smoke.sh` and `tests/installers/install_smoke.sh`, plus the sequencing boundary against `best-effort-distro-package-manager`.
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` — populate `PDLDPM0` through `PDLDPM2` and reconcile `meta.behavior_platforms_required` with Linux-only behavior plus macOS/Windows no-delta evidence.
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` — reconcile the feature-directory and Related Docs drift from `stashing-ferret` to `persist-detected-linux-distro-pkg-manager` before the quality gate.
