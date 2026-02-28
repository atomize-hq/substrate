# stabilize-dev-install-helper-discovery — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
- Spec manifest:
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/"` (strict packs only).

### Create
- None

### Edit
- `scripts/substrate/dev-install-substrate.sh`
- `scripts/substrate/dev-uninstall-substrate.sh`
- `crates/shell/src/builtins/world_enable/runner/paths.rs`
- `crates/shell/tests/world_enable.rs`

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
- Change: Dev installs stage `world-enable.sh` + `install-substrate.sh` under `$SUBSTRATE_HOME/scripts/substrate/…` so `substrate world enable` can still locate the helper after `cargo clean` removes `<repo>/target/scripts/…`.
  - Direct impact:
    - After `scripts/substrate/dev-install-substrate.sh`, running `substrate world enable` no longer fails solely because `<repo>/target/scripts/substrate/world-enable.sh` was deleted by `cargo clean`.
    - “Enable later” becomes less brittle for multi-repo workflows and frequent clean/rebuild cycles.
  - Cascading impact:
    - `dev-install-substrate.sh` output/help needs to reflect the additional staged helper location and the selected overwrite policy (Decision Register).
    - Validation surfaces must explicitly prove the helper is *resolved* from `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` when the version-dir location is absent (unit/integration coverage + manual playbook).
  - Contradiction risks:
    - Multi-checkout dev installs into the same `$SUBSTRATE_HOME` can compete for the same helper paths; the overwrite policy must be deterministic and safe when the destination already exists.
    - If staging uses symlinks, `$SUBSTRATE_HOME/scripts/substrate/*` may become dangling if the checkout is moved/removed; the failure mode should remain clear and fail-closed.

- Change: Dev uninstall removes only dev-managed helpers under `$SUBSTRATE_HOME/scripts/substrate/…` (ownership-guarded cleanup).
  - Direct impact:
    - `scripts/substrate/dev-uninstall-substrate.sh` can clean up the additional staged helper paths without risking accidental deletion of user-managed scripts.
  - Cascading impact:
    - The ownership-guard algorithm and refusal behavior must be deterministic (including exit-code class and operator-facing messaging) and must be covered by tests/playbook.
  - Contradiction risks:
    - Operators may expect dev-uninstall to remove “everything under `$SUBSTRATE_HOME`”; the contract must explicitly constrain deletion to dev-managed files only, and the script must warn when it refuses to delete a non-dev-managed destination.

### Config / env vars / paths
- Change: `substrate world enable` helper discovery order remains `(<version_dir>/scripts/substrate/world-enable.sh) → ($SUBSTRATE_HOME/scripts/substrate/world-enable.sh)`; dev installs guarantee the `$SUBSTRATE_HOME` fallback exists post-install.
  - Direct impact:
    - `cargo clean` removing `<repo>/target/scripts/…` no longer breaks helper discovery for dev installs.
  - Cascading impact:
    - The feature’s `contract.md` must explicitly treat `<repo>/target/scripts/substrate/*` as “may be absent after `cargo clean`” while requiring the `$SUBSTRATE_HOME/scripts/substrate/*` fallback to exist post dev-install.
    - Test coverage should include both behaviors: preference for version-dir when present and fallback to `$SUBSTRATE_HOME` when absent.
  - Contradiction risks:
    - The current `paths.rs` error messaging is optimized for “reinstall Substrate” (production). If dev installs can still end up without either helper path, remediation text may be misleading unless explicitly scoped in the contract/playbook.

- Change: Cross-platform behavior is affected differently by the helper’s location because `scripts/substrate/world-enable.sh` derives `RELEASE_ROOT` from its own path.
  - Direct impact:
    - Linux: helper can provision via staged artifacts under the inferred version dir and is primarily sensitive to helper *discovery* and `world-agent` availability.
    - macOS: the helper’s macOS provisioning path expects `${RELEASE_ROOT}/scripts/mac/lima-warm.sh`; staging the helper under `$SUBSTRATE_HOME/scripts/substrate/…` does not imply `${RELEASE_ROOT}/scripts/mac/…` exists for dev installs.
  - Cascading impact:
    - The contract and playbook must be explicit about macOS expectations for “enable later” in dev installs (e.g., whether the helper is expected to succeed beyond `--dry-run`, or whether additional staging/follow-up work is required).
  - Contradiction risks:
    - The ADR’s manual validation includes `substrate world enable --dry-run`, which can pass on macOS even when real provisioning would fail due to missing `${RELEASE_ROOT}/scripts/mac/lima-warm.sh`. If the intent is “enable later actually provisions”, the touch set may need to expand (follow-up ADR) to stage additional macOS provisioning scripts or change helper release-root resolution.
    - Resolution options (A/B):
      - Option A: This feature guarantees helper discovery (and `--dry-run` plan resolution) on macOS dev installs; successful macOS provisioning is out of scope and handled by a follow-up.
      - Option B: Expand this feature to make macOS provisioning succeed after dev-install by staging the required `${RELEASE_ROOT}/scripts/mac/…` surface (or changing `RELEASE_ROOT` resolution) in addition to the helper scripts.
      - Selected: Option A (aligns with ADR-0034 scope/validation focus and avoids “bundle parity” expansion).

- Change: Windows remains unsupported for `substrate world enable` (no behavior change), but this pack’s task metadata requires Windows behavior evidence.
  - Direct impact:
    - Windows users should observe “unsupported” behavior (no new staging semantics beyond what dev-install/dev-uninstall already do on Windows hosts).
  - Cascading impact:
    - Planning artifacts must define the deterministic Windows expectation and how it is validated (or mark it as N/A for this feature) to satisfy `tasks.json` platform evidence requirements.
  - Contradiction risks:
    - If automation requires Windows validation for a feature that is explicitly unsupported on Windows, CI/triad gating can deadlock unless the platform-evidence contract is reconciled.
    - Resolution options (A/B):
      - Option A: Treat Windows as “no change / unsupported” for this feature and explicitly mark Windows validation as N/A (or “expected unsupported” evidence) in planning artifacts.
      - Option B: Expand scope to introduce Windows-specific staging/validation behavior for helper discovery even though `world enable` remains unsupported.
      - Selected: Option A (explicit no-change/unsupported contract; keep scope aligned with ADR-0034).

### Policy / isolation / security posture
- Change: Uninstall behavior becomes explicitly ownership-guarded for `$SUBSTRATE_HOME/scripts/substrate/…`.
  - Direct impact:
    - Reduces risk of destructive deletion under a user-owned prefix when multiple installation mechanisms (prod installer, dev installer, user scripts) coexist.
  - Cascading impact:
    - The Decision Register must pick a concrete ownership-guard algorithm (and staging mechanism) that is robust to symlinks/copies and does not follow attacker-controlled paths.
  - Contradiction risks:
    - Naive cleanup approaches (e.g., recursively deleting `$SUBSTRATE_HOME/scripts/…`) violate the ADR’s protected-path invariant and create a security footgun.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `scripts/substrate/world-enable.sh`
    - `scripts/substrate/install-substrate.sh`
  - Conflict: yes (shared script touchpoints + same “enable later” workflow)
  - Resolution (explicit):
    - Sequencing boundary: land helper-staging stability (this ADR) before or concurrently with “stage world-agent on `--no-world`” so the enable-later path has both (a) resolvable helper scripts and (b) required provisioning artifacts.
    - Non-overlap boundary: this ADR owns helper staging under `$SUBSTRATE_HOME/scripts/substrate/…`; ADR-0035 owns `world-agent` artifact staging and missing-artifact remediation. Both must share/align on dev-install overwrite/ownership guard decisions to avoid divergent behavior.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `crates/shell/src/builtins/world_enable/`
    - `scripts/substrate/world-enable.sh`
    - `scripts/substrate/install-substrate.sh`
  - Conflict: no (orthogonal goals), but shared `world enable` surface
  - Resolution (explicit):
    - Keep helper discovery/staging refactors orthogonal to provisioning/deps logic; ensure `substrate world enable --dry-run` remains stable and continues to locate the correct helper path while new provisioning flags are added.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - `substrate world enable` “home” semantics and helper invocation invariants (`--home`, `SUBSTRATE_HOME`)
    - Installer/dev-installer responsibilities for what is written under `$SUBSTRATE_HOME`
  - Conflict: no (this ADR does not change CLI/config semantics), but must remain aligned
  - Resolution (explicit):
    - Ensure helper staging and cleanup do not introduce new env/config precedence behavior and do not reintroduce removed legacy knobs (`SUBSTRATE_PREFIX`, `--prefix` on `world enable`).
    - Ensure any new path ownership under `$SUBSTRATE_HOME/scripts/substrate/…` is documented as an installer-managed surface in the feature’s `contract.md` (without contradicting ADR-0003 path invariants).

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/dev-install-substrate.sh` (potentially; this pack has an explicit DR about whether dev-install is in-scope)
  - Conflict: no (unless dev-install is explicitly brought into scope for that pack)
  - Resolution (explicit):
    - If that pack decides to touch dev-install, sequence its dev-installer edits after this feature to reduce merge conflict risk and keep helper-staging contract changes coherent.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh`
    - `scripts/substrate/install-substrate.sh`
  - Conflict: no (this feature stages helpers; that feature changes provisioning behavior)
  - Resolution (explicit):
    - Coordinate so helper-staging changes do not break `world enable --dry-run` output expectations used by provisioning/deps packs, and so helper-script interface changes are reflected in what dev-install stages under `$SUBSTRATE_HOME/scripts/substrate/…`.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh`
    - `scripts/substrate/install-substrate.sh`
  - Conflict: no (shared helper scripts; different feature intent)
  - Resolution (explicit):
    - Keep helper-script interface changes coordinated so dev-install staging always provides the current `world-enable.sh` + `install-substrate.sh` pair under `$SUBSTRATE_HOME/scripts/substrate/…`.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Decide helper staging mechanism (copy vs symlink) as a user-visible contract.
  - DR-0002 — Decide uninstall ownership-guard algorithm (how “managed by dev-install” is determined).
  - DR-0003 — Decide overwrite policy when `$SUBSTRATE_HOME/scripts/substrate/*` already exists (including behavior when destination is not dev-managed).
- Spec updates required (if any):
  - `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md` — reconcile feature directory path drift (`.../draft/dev-install-helper-discovery/` vs `.../draft/stabilize-dev-install-helper-discovery/`).
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/plan.md` — include an explicit mapping from ADR slice labels (`C0`, `C1`) to the pack’s canonical slice IDs (`SDIHD0`, `SDIHD1`).
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/decision_register.md` — complete DR-0001..DR-0003 with explicit A/B options and a selection (ADR currently enumerates only).
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/contract.md` — explicitly scope macOS “enable later” expectations if additional staging of `${RELEASE_ROOT}/scripts/mac/…` is not part of this ADR.
  - `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery/tasks.json` — reconcile required Windows behavior evidence with “world enable unsupported on Windows” (define deterministic expectation or mark validation as N/A for this feature).
  - `docs/COMMANDS.md` — update `substrate world enable` flag documentation drift (`--home` vs `--prefix`) to match ADR-0003 + current CLI behavior.
