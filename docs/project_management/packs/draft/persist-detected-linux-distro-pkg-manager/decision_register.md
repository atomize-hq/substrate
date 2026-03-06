# persist-detected-linux-distro-pkg-manager — decision register

This file records the A/B decisions required to make the Linux install-state persistence contract deterministic and testable.

## Inputs (non-authoritative links)
- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- Minimal spec draft (pre-planning): `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md`
- Impact map (pre-planning): `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- Alignment report (required input): `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/alignment_report.md`

## DR-0001 — Persistence behavior when platform metadata inputs are incomplete

**Decision owner(s):** PDLDPM-PWS-contract (contract)  
**Date (UTC):** 2026-03-06  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`

**Problem / context**

ADR-0032 requires Linux installs to persist `host_state.platform.*` additively, but the inputs are best-effort:
- `/etc/os-release` can be missing or unreadable,
- `ID` or `ID_LIKE` can be absent independently, and
- the installer can still have a valid selected package manager from the dependency-owned detection pipeline.

The contract must choose one exact JSON behavior for partially available inputs.

**Option A — Persist available metadata and omit unavailable `os_release.*` keys**

- Persist `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` whenever the upstream detection contract resolves them.
- Persist `host_state.platform.os_release.id` only when its input is available.
- Persist `host_state.platform.os_release.id_like` only when its input is available.
- Omit unavailable `os_release.*` keys entirely.
- Do not write sentinel strings such as `<unknown>`.
- Do not write `null` for unavailable `os_release.*` keys.

**Option B — Persist a shape-complete object with sentinel placeholders**

- Always write `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like`.
- Represent unavailable values with sentinel strings such as `<unknown>` or explicit `null`.
- Keep `pkg_manager.*` persistence the same as Option A.

**Recommendation**

- **Selected:** Option A — persist available metadata and omit unavailable `os_release.*` keys
- **Rationale (crisp):** Stored JSON must reflect only authoritative available inputs. Sentinel rendering belongs to operator output, not to persisted state.

**Downstream doc updates required by this decision**

- `contract.md`
  - State that partial Linux metadata persistence is valid and that unavailable `os_release.*` keys are omitted.
- `install-state-schema-spec.md`
  - Define exact omission rules and sample payloads for partial-input and unreadable-`/etc/os-release` cases.
- `slices/PDLDPM0/PDLDPM0-spec.md`
  - Add acceptance criteria that assert omission, not sentinels or `null`.

## DR-0002 — Dry-run semantics for install-state persistence

**Decision owner(s):** PDLDPM-PWS-contract (contract)  
**Date (UTC):** 2026-03-06  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

**Problem / context**

ADR-0032 defines a Linux successful-install persistence guarantee but does not state whether a dry-run mutates the install-state file. The contract must select one exact rule so tests and operator expectations are stable.

**Option A — Dry-run is non-mutating**

- A dry-run invocation does not create or modify `<resolved SUBSTRATE_HOME>/install_state.json`.
- The Linux write guarantee applies only to successful non-dry-run installs.
- Dry-run output may describe the would-be metadata, but no on-disk state changes occur.

**Option B — Dry-run writes install-state metadata**

- A dry-run invocation creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` with the same metadata that a real install would persist.
- The Linux write guarantee applies to both real installs and dry-runs.

**Recommendation**

- **Selected:** Option A — dry-run is non-mutating
- **Rationale (crisp):** Dry-run must remain simulation-only. Persisting install-state during dry-run would create a misleading success artifact without an actual install.

**Downstream doc updates required by this decision**

- `contract.md`
  - State that dry-run must not create or modify `install_state.json`.
- `plan.md`
  - Include validation coverage that confirms dry-run leaves the install-state file unchanged.
- `slices/PDLDPM1/PDLDPM1-spec.md`
  - Add acceptance criteria for no-write dry-run behavior.
- `slices/PDLDPM2/PDLDPM2-spec.md`
  - Add smoke assertions that dry-run does not mutate persisted install-state.

## DR-0003 — Installer entrypoint scope for the shared install-state contract

**Decision owner(s):** PDLDPM-PWS-contract (contract)  
**Date (UTC):** 2026-03-06  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`

**Problem / context**

ADR-0032 names `scripts/substrate/install-substrate.sh`, but repository docs and current behavior already expose one `install_state.json` surface across both installer entrypoints. The feature must select whether the persistence contract applies to one script or both.

**Option A — Shared contract across both installer entrypoints**

- `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` both create or update the same Linux install-state contract.
- The same dry-run rule, `--no-world` rule, path invariant, and failure posture apply to both entrypoints.
- Validation must cover both the production installer and the dev installer harness that already exercises the same file surface.

**Option B — Scope the feature to `install-substrate.sh` only**

- Only `scripts/substrate/install-substrate.sh` receives the new persistence guarantee.
- `scripts/substrate/dev-install-substrate.sh` remains on its prior behavior, even though it writes the same file path.
- Docs must describe distinct semantics for the same on-disk file depending on entrypoint.

**Recommendation**

- **Selected:** Option A — shared contract across both installer entrypoints
- **Rationale (crisp):** One file path must have one meaning. Splitting semantics by entrypoint would create silent contract drift for `install_state.json`.

**Downstream doc updates required by this decision**

- `contract.md`
  - State that both installer entrypoints share one Linux install-state contract.
- `plan.md`
  - Sequence validation across both installer paths and record the dev-installer parity seam explicitly.
- `slices/PDLDPM1/PDLDPM1-spec.md`
  - Add acceptance criteria for shared entrypoint behavior.
- `pre-planning/impact_map.md`
  - Keep the selected dual-installer touch set authoritative until the ADR path drift is reconciled separately.
