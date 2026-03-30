# Decision Register — persist-detected-linux-distro-pkg-manager

Template standard:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

## DR-0001 — Persistence location contract

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`

**Problem / context**

ADR-0032 requires Linux distro and package-manager metadata to survive beyond the installer process. The planning pack must choose whether that metadata extends the existing installer state file or creates a second file.

**Option A — Extend `install_state.json`**

- **Pros:** keeps one canonical installer metadata file; preserves one operator lookup path; aligns with existing uninstall-state storage.
- **Cons:** requires additive schema work inside an existing document; reliable-write semantics must protect existing cleanup metadata.
- **Cascading implications:** `install-state-schema-spec.md` must define additive fields and merge rules; `PDLDPM1` must guarantee file creation on successful Linux installs.
- **Risks:** weak merge rules would damage `host_state.group` or `host_state.linger`; operator docs would drift if they still describe event-only writes.
- **Unlocks:** later guidance surfaces can read one file without cross-file reconciliation; smoke coverage can validate one canonical path.
- **Quick wins / low-hanging fruit:** reuse the existing `install_state.json` path, naming, and cleanup compatibility story.

**Option B — Write a separate host-platform metadata file**

- **Pros:** isolates new metadata from the existing cleanup state shape; avoids modifying the current file contract.
- **Cons:** creates two installer metadata files; doubles operator documentation burden; forces consumers to resolve precedence across files.
- **Cascading implications:** docs and tests must explain two canonical locations; uninstall and support surfaces would need file-selection rules.
- **Risks:** dual files create drift and partial-write failure modes; later consumers would face conflicting state across files.
- **Unlocks:** narrow local schema for the new metadata only.
- **Quick wins / low-hanging fruit:** none, because every consumer and doc still needs a second-path story.

**Recommendation**

- **Selected:** Option A — Extend `install_state.json`
- **Rationale (crisp):** one canonical installer metadata file is simpler, stays additive under `schema_version = 1`, matches the existing cleanup surface, and avoids fragmented state.

**Follow-up tasks (explicit)**

- `install-state-schema-spec.md` must define the additive `host_state.platform.*` fields under `schema_version = 1`.
- `slices/PDLDPM0/PDLDPM0-spec.md` must trace field persistence into the existing document.
- `slices/PDLDPM1/PDLDPM1-spec.md` must require reliable creation and update of `install_state.json` on successful Linux installs.
- `docs/INSTALLATION.md` must describe only the canonical `install_state.json` path for this feature.

## DR-0002 — Field naming and nesting under `host_state.platform.*`

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`

**Problem / context**

The planning pack must choose one stable JSON shape for persisted platform metadata. The selected shape must remain additive, must preserve the existing host-state structure, and must support later reads without schema ambiguity.

**Option A — Nest new fields under `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*`**

- **Pros:** keeps all host-derived state under `host_state`; separates distro identity from package-manager selection cleanly; leaves existing `host_state.group` and `host_state.linger` untouched.
- **Cons:** requires nested-object merge rules; adds one more object layer than a flatter shape.
- **Cascading implications:** `install-state-schema-spec.md` must define exact field paths and absence semantics; smoke coverage must assert nested keys.
- **Risks:** implementation drift would occur if scripts flatten or rename keys; operator docs must use exact field names.
- **Unlocks:** later consumers get a stable namespace for platform guidance data; future additive fields can extend `host_state.platform` without top-level sprawl.
- **Quick wins / low-hanging fruit:** maps directly to ADR-0032 field names and to the slice boundary for persisted platform metadata.

**Option B — Flatten the new values into top-level keys or ad hoc `host_state` names**

- **Pros:** fewer nested objects in the short term.
- **Cons:** weak namespace discipline; higher collision risk with existing or future host-state fields; harder to explain as one coherent platform block.
- **Cascading implications:** docs and tests must carry custom field-by-field naming rules; later additions would need more one-off keys.
- **Risks:** field drift between scripts, docs, and later consumers; ambiguous ownership of package-manager metadata.
- **Unlocks:** shorter JSON paths only.
- **Quick wins / low-hanging fruit:** none, because path brevity does not offset drift risk.

**Recommendation**

- **Selected:** Option A — Nest new fields under `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*`
- **Rationale (crisp):** one namespaced platform block is additive, preserves the existing host-state structure, keeps the field contract testable, and leaves package-manager vocabulary ownership external.

**Follow-up tasks (explicit)**

- `install-state-schema-spec.md` must define the exact nested paths, types, examples, and merge rules.
- `slices/PDLDPM0/PDLDPM0-spec.md` must require writes to the exact nested paths only.
- `docs/INSTALLATION.md` must use the accepted field names `schema_version`, `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`.

## DR-0003 — Selected-manager and source-vocabulary ownership

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`

**Problem / context**

This pack persists `pkg_manager.selected` and `pkg_manager.source`, but the planning pack must choose whether it owns those vocabularies directly or inherits them from the Linux detection contract.

**Option A — Duplicate the vocabulary locally**

- **Pros:** local document would be self-contained.
- **Cons:** duplicates authoritative detection semantics; creates two places that must stay identical; raises drift risk for manager spellings and `source` values.
- **Cascading implications:** every detection-contract update would require mirrored edits here; slice specs would need cross-pack reconciliation rules.
- **Risks:** persisted values would diverge from emitted installer values; tests would target conflicting authorities.
- **Unlocks:** single-file reading for this pack only.
- **Quick wins / low-hanging fruit:** none, because duplicate vocabulary increases long-term cost immediately.

**Option B — Treat `best-effort-distro-package-manager` as the single source of truth and copy emitted strings verbatim**

- **Pros:** preserves one authority for manager selection and `source`; keeps this pack focused on persistence only; removes vocabulary drift risk.
- **Cons:** readers must follow one cross-pack link.
- **Cascading implications:** this pack must not restate supported-manager spellings or `source` enums; slice specs must trace verbatim copying rather than local parsing.
- **Risks:** weak link discipline would tempt later duplication; docs must preserve the authority boundary clearly.
- **Unlocks:** ADR-0031 and ADR-0032 can land independently while sharing one vocabulary contract; later consumers can trust persisted strings to match installer output exactly.
- **Quick wins / low-hanging fruit:** reuse the accepted detection one-liner vocabulary without inventing new local rules.

**Recommendation**

- **Selected:** Option B — Treat `best-effort-distro-package-manager` as the single source of truth and copy emitted strings verbatim
- **Rationale (crisp):** persistence must record the detection contract's output, not create a second vocabulary authority.

**Follow-up tasks (explicit)**

- `contract.md` must state that `pkg_manager.selected` and `pkg_manager.source` are copied verbatim from `best-effort-distro-package-manager`.
- `install-state-schema-spec.md` must reference the upstream vocabulary owner instead of redefining it and must keep the schema additive under `schema_version = 1`.
- `slices/PDLDPM0/PDLDPM0-spec.md` must require tests that prove no local re-derivation of `pkg_manager.selected` or `pkg_manager.source`.

## DR-0004 — Successful-install write-trigger scope

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`

**Problem / context**

The planning pack must choose which successful Linux installer flows create or update `install_state.json`. The current event-only write behavior is not sufficient for this ADR's persistence goal.

**Option A — Keep persistence scoped to the hosted installer or to world-enabled branches only**

- **Pros:** smaller implementation delta in the short term.
- **Cons:** dev install remains on a different metadata contract; `--no-world` success would still skip persistence; operator docs would need split rules.
- **Cascading implications:** tests must branch by installer flavor and world-enabled state; consumers cannot rely on a successful Linux install leaving a canonical metadata file behind.
- **Risks:** drift between hosted and dev installers; support guidance would be wrong after successful `--no-world` or dev installs.
- **Unlocks:** fewer immediate script edits only.
- **Quick wins / low-hanging fruit:** none, because split write rules undermine the feature's core value.

**Option B — All successful Linux producer flows share one write contract**

- **Pros:** one rule covers hosted install, hosted `--no-world`, dev install, and dev `--no-world`; later consumers can rely on one persisted file after successful Linux installs; docs and smoke coverage stay unified.
- **Cons:** both installer scripts require aligned updates.
- **Cascading implications:** `PDLDPM1` must pin the same write matrix across both installers; smoke coverage must cover no-event success and no-world success.
- **Risks:** implementation drift if one script keeps event-only behavior; documentation drift if `docs/INSTALLATION.md` still implies hosted-only writes.
- **Unlocks:** stable producer semantics independent of world provisioning; future guidance surfaces can trust the file on successful Linux installs.
- **Quick wins / low-hanging fruit:** reuse one shared rule for both installers and for both world-enabled states.

**Recommendation**

- **Selected:** Option B — All successful Linux producer flows share one write contract
- **Rationale (crisp):** the feature only solves operator confusion when every successful Linux producer flow leaves the same canonical metadata surface.

**Follow-up tasks (explicit)**

- `contract.md` must publish the exact write matrix: hosted install, hosted `--no-world`, dev install, and dev `--no-world` write on success; hosted `--dry-run` does not write.
- `slices/PDLDPM1/PDLDPM1-spec.md` must define idempotent create/update behavior and the exact temp-file replacement rule.
- `tests/installers/install_state_smoke.sh` must add assertions for no-event Linux success and additive compatibility under `schema_version = 1`.
- `docs/INSTALLATION.md` must describe the shared hosted-plus-dev metadata-producer contract.

## DR-0005 — Canonical feature-directory authority for this planning pack

**Decision owner(s):** Installer / host-provisioning maintainers  
**Date (UTC):** 2026-03-07  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/alignment_report.md`

**Problem / context**

ADR-0032 still points at `docs/project_management/packs/draft/stashing-ferret/`, while the resolved feature directory is `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`. Full planning needs one canonical path immediately so contract and schema documents have a single authority boundary.

**Option A — Keep the ADR path authoritative until the ADR file is edited**

- **Pros:** defers a naming decision inside the planning pack.
- **Cons:** preserves dual-authority paths; breaks discoverability for this pack's documents; leaves triage and alignment outputs pointing at a different directory than the ADR.
- **Cascading implications:** every new planning document would need duplicate path caveats; reviewers would have to resolve path drift manually.
- **Risks:** docs would fork across two feature directories; downstream slices would link to the wrong pack path.
- **Unlocks:** no immediate ADR wording choice only.
- **Quick wins / low-hanging fruit:** none, because the drift remains active.

**Option B — Treat the resolved feature directory as canonical now and record ADR reconciliation as an explicit follow-up**

- **Pros:** gives full planning one stable authority path; keeps all new pack documents in the same directory; removes ambiguity for slice specs and task references.
- **Cons:** requires an external tracked ADR edit after this PWS completes.
- **Cascading implications:** ADR-0032 related-doc links and scope text must be updated to the resolved pack path; docs-validation work must preserve that authority boundary.
- **Risks:** the ADR remains temporarily stale until the external edit lands; reviewers must treat the pack docs as authoritative during that interval.
- **Unlocks:** contract, schema, and slice docs can proceed without duplicate path language; later lint and orchestration can target one feature directory.
- **Quick wins / low-hanging fruit:** the planning pack can publish stable links immediately under the resolved feature directory.

**Recommendation**

- **Selected:** Option B — Treat the resolved feature directory as canonical now and record ADR reconciliation as an explicit follow-up
- **Rationale (crisp):** full planning needs one authority path immediately, and the resolved feature directory already exists as the execution target.

**Follow-up tasks (explicit)**

- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` must replace `docs/project_management/packs/draft/stashing-ferret/` references with `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
- `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` must update Related Docs links for `plan.md`, `tasks.json`, `spec_manifest.md`, `decision_register.md`, and `impact_map.md`.
- `contract.md` must state that stale ADR path references do not override the resolved feature directory.
