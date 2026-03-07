# persist-detected-linux-distro-pkg-manager — decision register

This file records the contract decisions required to make ADR-0032 deterministic and testable.

## Inputs (non-authoritative links)

- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- Alignment report: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/alignment_report.md`
- Workstream triage: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/workstream_triage.md`
- Impact map: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- Spec manifest: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`

---

### DR-0001 — Persistence target (`install_state.json` vs separate metadata file)

**Decision owner(s):** `PDLDPM-PWS-contract`  
**Date:** 2026-03-07  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`  

**Problem / Context**

- ADR-0032 requires Linux distro and package-manager metadata to persist across installer runs.
- Pre-planning surfaced an unresolved choice between extending the existing install-state file and creating a second metadata file for Linux host details.

**Option A — Create a second metadata file dedicated to distro and package-manager state**

- **Pros:** isolates the new fields from the existing install-state document; reduces merge surface inside `install_state.json`.
- **Cons:** creates a second persistence location; forces future consumers to coordinate multiple files; breaks the existing operator expectation that installer state lives in `install_state.json`.

**Option B — Extend the existing canonical install-state file at `<effective prefix>/install_state.json`**

- **Pros:** keeps one persisted installer-state file; matches current installer behavior; keeps future-consumer lookup deterministic; preserves the operator-facing `$SUBSTRATE_HOME/install_state.json` language once it is tied to the effective prefix.
- **Cons:** requires additive merge rules inside the existing JSON document; makes atomic-update behavior part of the contract.

**Recommendation**

- **Selected:** Option B — extend the existing canonical install-state file at `<effective prefix>/install_state.json`.
- **Rationale (crisp):** one canonical file avoids path drift, avoids split-brain state, and matches the installer flows already in production.

**Follow-up tasks (explicit)**

- `contract.md`: define the effective-prefix to `$SUBSTRATE_HOME` equivalence rule and the protected-path invariant.
- `install-state-schema-spec.md`: define the additive merge and preservation rules for the existing install-state document.

---

### DR-0002 — Persisted field nesting (`host_state.platform.*` vs local pack-specific names)

**Decision owner(s):** `PDLDPM-PWS-contract`  
**Date:** 2026-03-07  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`  

**Problem / Context**

- Pre-planning identified unresolved naming and nesting for the persisted Linux metadata.
- The contract needs one durable location that can coexist with existing `host_state.group` and `host_state.linger` fields without a schema-version bump.

**Option A — Add pack-local names or top-level fields outside `host_state.platform.*`**

- **Pros:** can look simpler in the short term; avoids adding another nested object.
- **Cons:** creates a pack-local schema vocabulary; increases drift risk with existing host-state organization; makes future consumer logic less predictable.

**Option B — Persist only under `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*`**

- **Pros:** groups the Linux platform metadata under one stable boundary; preserves existing `host_state` organization; supports additive compatibility on `schema_version = 1`.
- **Cons:** requires downstream docs to reference the nested paths exactly; merge logic must preserve unrelated `host_state` siblings.

**Recommendation**

- **Selected:** Option B — persist only under `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*`.
- **Rationale (crisp):** one nested boundary keeps the schema additive, keeps older fields intact, and gives future consumers one predictable lookup path.

**Follow-up tasks (explicit)**

- `install-state-schema-spec.md`: define the exact field paths, types, and absence semantics for `id`, `id_like`, `selected`, and `source`.
- `PDLDPM0`: persist the fields only at those paths and preserve unrelated `host_state` content.

---

### DR-0003 — Vocabulary authority (duplicate locally vs import from best-effort detection contract)

**Decision owner(s):** `PDLDPM-PWS-contract`  
**Date:** 2026-03-07  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`  
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`  

**Problem / Context**

- The persisted package-manager fields need stable enum values.
- Pre-planning flagged a choice between re-stating the package-manager and `pkg_manager.source` vocabularies locally or inheriting them from the upstream detection contract.

**Option A — Duplicate the selected-manager and source vocabularies in this pack**

- **Pros:** local doc looks self-contained; downstream readers see all values without leaving the pack.
- **Cons:** duplicates authoritative strings; risks drift between detection and persistence; makes any future vocabulary change a multi-doc contract migration.

**Option B — Treat the best-effort detection contract as the only authority and persist its outputs verbatim**

- **Pros:** one source of truth for manager strings and source strings; no translation layer between detection and persistence; easier cross-pack testing.
- **Cons:** this pack is not self-contained; readers must follow a cross-pack reference for the exact vocabulary.

**Recommendation**

- **Selected:** Option B — treat `best-effort-distro-package-manager/contract.md` as the only vocabulary authority and persist its outputs verbatim.
- **Rationale (crisp):** persistence must store the detector output without local reinterpretation or duplicated enum definitions.

**Follow-up tasks (explicit)**

- `contract.md`: state that `pkg_manager.selected` and `pkg_manager.source` are copy-through values from the upstream detector contract.
- `PDLDPM0` and `PDLDPM2`: assert verbatim persistence rather than local vocabulary expansion.

---

### DR-0004 — Write-trigger scope (hosted-only vs hosted+dev, `--no-world`, and `--dry-run`)

**Decision owner(s):** `PDLDPM-PWS-contract`  
**Date:** 2026-03-07  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`  

**Problem / Context**

- Pre-planning identified missing branch definitions for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`.
- The pack needs one write matrix that keeps no-world diagnostics useful without making dry-run stateful.

**Option A — Persist metadata only for the hosted installer when world provisioning is enabled**

- **Pros:** smallest implementation surface; avoids touching the dev installer and the `--no-world` branch.
- **Cons:** breaks the documented hosted/dev shared producer contract; makes `--no-world` installs lose persisted metadata; leaves dev installs without the same diagnostics inputs.

**Option B — Persist metadata for both installers on successful non-dry-run Linux runs, including `--no-world`, and suppress all persistence on `--dry-run`**

- **Pros:** one producer contract across hosted and dev installers; keeps persisted metadata available when world provisioning is skipped; preserves dry-run as side-effect-free.
- **Cons:** wider implementation surface; requires the contract to define identical write rules for more than one installer script.

**Recommendation**

- **Selected:** Option B — persist metadata for both installers on successful non-dry-run Linux runs, including `--no-world`, and suppress all persistence on `--dry-run`.
- **Rationale (crisp):** the pack needs one deterministic producer contract, and `--dry-run` must remain side-effect-free.

**Follow-up tasks (explicit)**

- `contract.md`: define the exact write/no-write matrix and warning-only failure posture.
- `PDLDPM1`: implement the temp-file replacement rule and branch coverage for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`.

---

### DR-0005 — ADR feature-directory drift handling (`draft/stashing-ferret` vs resolved feature directory)

**Decision owner(s):** `PDLDPM-PWS-contract`  
**Date:** 2026-03-07  
**Status:** Accepted  
**Related docs:**  
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`  
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`  

**Problem / Context**

- `alignment_report.md` identified a hard gate: ADR-0032 still points at `docs/project_management/packs/draft/stashing-ferret/`, while the resolved feature directory is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
- This PWS cannot edit the ADR because the dispatcher allowlist restricts tracked writes to `contract.md` and `decision_register.md`.

**Option A — Leave the ADR drift unresolved and let this pack become a second authority**

- **Pros:** no external follow-up needed before continuing local pack work.
- **Cons:** creates dual-authority planning docs; makes cross-linking unstable; increases the chance that later work lands against the wrong feature directory.

**Option B — Treat ADR path alignment as a required external follow-up and keep this pack aligned to the resolved feature directory**

- **Pros:** preserves one resolved planning location for this pack; records the gate explicitly; avoids normalizing the drift inside local docs.
- **Cons:** requires another writer to update ADR-0032 before the broader planning set is considered fully reconciled.

**Recommendation**

- **Selected:** Option B — treat ADR path alignment as a required external follow-up and keep this pack aligned to the resolved feature directory.
- **Rationale (crisp):** this pack cannot resolve the ADR within its allowlist, but it can prevent further drift by naming the resolved feature directory as canonical for all local contract surfaces.

**Follow-up tasks (explicit)**

- External tracked update required: reconcile ADR-0032 Scope and Related Docs from `draft/stashing-ferret` to `draft/persist-detected-linux-distro-pkg-manager`.
- Keep all local contract, schema, slice, and task docs in this pack rooted at the resolved feature directory.
