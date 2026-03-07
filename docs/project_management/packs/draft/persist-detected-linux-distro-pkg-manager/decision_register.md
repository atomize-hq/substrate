# Decision Register — persist-detected-linux-distro-pkg-manager

Template standard:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

Authority note:
- The canonical planning-pack directory for this feature is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
- ADR-0032 still references `docs/project_management/packs/draft/stashing-ferret/`. Those path references are stale and must be reconciled before quality gate, but they do not override the pack-local decisions below.

## DR-0001 — Persistence behavior when Linux platform inputs are incomplete

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`

**Problem / context**

ADR-0032 requires Linux installs to persist `host_state.platform.*`, but it does not pin the exact write behavior when `/etc/os-release` inputs are missing or only partially available. The contract needs one deterministic rule so stored JSON does not drift from the stderr-rendering rules owned by `best-effort-distro-package-manager`.

**Option A — Persist available fields and omit unavailable os-release keys**

- Persist `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` whenever the installer reaches a successful Linux install.
- Persist `host_state.platform.os_release.id` only when the normalized `ID` input is available.
- Persist `host_state.platform.os_release.id_like` only when the normalized `ID_LIKE` input is available.
- Omit unavailable `host_state.platform.os_release.*` keys from JSON.
- Do not persist placeholder strings or sentinel values for missing os-release inputs.

**Option B — Persist placeholder values for missing os-release inputs**

- Persist the full `host_state.platform.os_release` object on every successful Linux install.
- When `ID` or `ID_LIKE` is unavailable, write a placeholder value such as `<unknown>` or `null`.
- Keep `pkg_manager.*` persistence the same as Option A.

**Recommendation**

- **Selected:** Option A — Persist available fields and omit unavailable os-release keys
- **Rationale (crisp):** it keeps stored JSON additive and clean while preserving the stderr-only `<unknown>` rendering contract in the upstream detection pack.

**Downstream doc updates required by this decision**

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - State that incomplete os-release inputs do not block Linux install-state persistence.
  - State that unavailable `host_state.platform.os_release.*` keys are omitted instead of stored as placeholders.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - Define exact omission rules and sample payloads for partial os-release input cases.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
  - Add acceptance criteria for persistence with readable and unreadable `/etc/os-release` inputs.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM3/PDLDPM3-spec.md`
  - Add Linux smoke assertions that verify omission semantics instead of placeholder strings.

## DR-0002 — Dry-run semantics for install-state persistence

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`

**Problem / context**

ADR-0032 requires reliable Linux install-state persistence after successful installs, but it does not state whether `scripts/substrate/install-substrate.sh --dry-run` writes the file. The contract needs one exact rule so dry-run remains deterministic and non-contradictory.

**Option A — Dry-run skips install-state writes**

- `scripts/substrate/install-substrate.sh --dry-run` does not create, replace, or update `<resolved SUBSTRATE_HOME>/install_state.json`.
- Dry-run logs that install-state persistence was skipped because the invocation was non-mutating.
- The successful-install write guarantee applies only to non-dry-run Linux installs.

**Option B — Dry-run writes install-state metadata**

- `scripts/substrate/install-substrate.sh --dry-run` writes or updates `<resolved SUBSTRATE_HOME>/install_state.json` with the Linux platform payload.
- Dry-run remains non-provisioning, but it is allowed to mutate the install-state file so operators can inspect the planned metadata payload.

**Recommendation**

- **Selected:** Option A — Dry-run skips install-state writes
- **Rationale (crisp):** it preserves the installer’s non-mutating dry-run meaning and matches the current host-state metadata posture in `scripts/substrate/install-substrate.sh`.

**Downstream doc updates required by this decision**

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - State the exact non-mutating dry-run rule and the logging requirement.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
  - Add acceptance criteria that dry-run does not leave an install-state artifact behind.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM3/PDLDPM3-spec.md`
  - Add validation coverage for the selected dry-run rule in Linux smoke.

## DR-0003 — Installer-entrypoint scope for the shared install-state contract

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`

**Problem / context**

ADR-0032 names `scripts/substrate/install-substrate.sh`, while the repository already exposes the same `install_state.json` surface through `scripts/substrate/dev-install-substrate.sh`. Planning needs one exact scope decision so the same on-disk file does not gain two incompatible meanings.

Deterministic slice-order note:
- The accepted four-slice execution order is `PDLDPM0 -> PDLDPM1 -> PDLDPM2 -> PDLDPM3`.
- `PDLDPM2` is the dev-installer parity slice.
- `PDLDPM3` is the final validation and checkpoint slice.

**Option A — Both installer entry points share one install-state contract**

- `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` both participate in the persisted Linux install-state contract for this feature.
- Linux `--no-world` runs still persist the selected install-state payload through both entry points.
- `tests/installers/install_state_smoke.sh` remains the Linux persistence harness for production install behavior.
- `tests/installers/install_smoke.sh` becomes the required validation harness for dev-installer parity.

**Option B — Only the production installer adopts ADR-0032**

- `scripts/substrate/install-substrate.sh` writes the new Linux `host_state.platform.*` payload.
- `scripts/substrate/dev-install-substrate.sh` remains on the pre-existing event-only metadata behavior.
- Operator docs must split `install_state.json` semantics by entry point.

**Recommendation**

- **Selected:** Option A — Both installer entry points share one install-state contract
- **Rationale (crisp):** the repository already exposes one `install_state.json` path through both installers, so a production-only expansion would create silent contract drift on the same file.

**Downstream doc updates required by this decision**

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - State that both installer entry points are in scope for the Linux persistence contract.
  - State that `--no-world` does not suppress Linux install-state persistence.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`
  - Add exact acceptance criteria for dev-installer parity on the shared install-state file.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
  - Include `tests/installers/install_smoke.sh` as the required validation surface for dev-installer parity.
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md`
  - Keep Linux as the only behavior-delta platform while recording that both Linux installer entry points share the same persisted metadata meaning.
