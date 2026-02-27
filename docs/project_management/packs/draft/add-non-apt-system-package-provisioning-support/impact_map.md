# add-non-apt-system-package-provisioning-support — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- Spec manifest:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/"` (strict packs only).

### Create
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-provisioning-protocol-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C1/C1-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C2/C2-spec.md`

### Edit
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/execution/platform/mod.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_deps/inventory.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/tests/world_enable.rs`
- `crates/shell/tests/world_deps_apt_install_wdp5.rs`
- `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`
- `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

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
- Change: Extend provisioning-time system package support to `pacman` on Arch-family world OSes via `substrate world enable --provision-deps`.
  - Direct impact:
    - On supported guest backends, provisioning can install/ensure system packages using the in-world package manager (`pacman` for Arch-family worlds; APT for Debian/Ubuntu-family worlds per ADR-0030).
    - On unsupported backends (notably Linux host-native), the command fails with explicit “provisioning unsupported / no host OS mutation” guidance.
    - `--dry-run` becomes manager-aware: prints derived requirement set(s) and intended actions without mutation.
  - Cascading impact:
    - `world enable` must derive requirements from the **effective enabled** world-deps set (bundle expansion + de-dup rules) and emit stable, testable stdout/stderr guidance.
    - `world enable` must use an explicit provisioning request profile (expected: `world-deps-provision`) so world-agent can relax isolation only for this opt-in workflow.
    - `world enable` must map failures to the exit-code taxonomy (not “unhandled error → exit 1”) and preserve coherent messages across macOS (Lima) vs Windows (WSL) vs Linux host-native.
  - Contradiction risks:
    - Mixed enabled sets (`install.method=apt` + `install.method=pacman`) require a single deterministic mismatch policy (fail-closed vs partial provision), or operators will see non-repeatable behavior across environments.
    - `world enable` already has a `--profile` flag for helper provisioning; its semantics must not be confused with the world-agent request `profile` used for deps provisioning.

- Change: Runtime `substrate world deps current sync|install` must fail early for system-package methods (`apt|pacman`) and never invoke OS package managers.
  - Direct impact:
    - Operators attempting runtime `sync|install` with system-package methods in scope get a friendly error + remediation pointing to `substrate world enable --provision-deps` (or manual guidance when provisioning is unsupported).
  - Cascading impact:
    - Existing tests that assert runtime APT execution must be updated/repurposed.
    - Operator docs must stop implying that runtime sync/install can install system packages in hardened worlds.
  - Contradiction risks:
    - The “scope” of detection must be explicit (effective enabled set vs explicitly requested items only), or `current install <ITEM...>` will behave inconsistently across projects.

### Config / env vars / paths
- Change: Extend world-deps inventory schema to allow `install.method=pacman` with `install.pacman[]`.
  - Direct impact:
    - Inventory authors can explicitly declare pacman system packages (no cross-distro name translation implied).
  - Cascading impact:
    - Schema validation must be updated (deny unknown fields; require `install.pacman` when method is pacman).
    - Contract docs that enumerate the install-method set must be updated to include `pacman` and to clarify provisioning-time vs runtime behavior.
  - Contradiction risks:
    - Existing contract text in `world-deps-packages-bundles-contract/contract.md` currently enumerates only `apt|script|manual` and includes runtime APT notes; it must be reconciled with provisioning-time posture to avoid two incompatible truths.

- Change: Add an in-world package-manager probe for provisioning (`/etc/os-release` + manager presence checks).
  - Direct impact:
    - Provisioning selects `apt` vs `pacman` from in-world evidence and fails with manager-aware remediation when unsupported/unknown.
  - Cascading impact:
    - Probe parsing/canonicalization rules must be safe and deterministic and should align with other `/etc/os-release` parsing work (ADR-0031 / ADR-0032) to avoid drift.
  - Contradiction risks:
    - If multiple “os-release parsing” implementations exist with different normalization rules, “Arch-family” classification can diverge between installer detection vs world probe vs persisted host-state metadata.

### Policy / isolation / security posture
- Change: Provisioning-time OS mutation is allowed only when explicitly invoked and only inside supported guest worlds.
  - Direct impact:
    - Runtime hardening remains fail-closed; provisioning is the only path that requests relaxed isolation for system package managers.
  - Cascading impact:
    - The provisioning request profile must be confined to the provisioning code path (not a global operator toggle).
    - Guard rails must prevent accidental host env leakage and must ensure the package-manager selection cannot be influenced by host PATH state.
  - Contradiction risks:
    - Existing scripts/tests that export `SUBSTRATE_WORLD_REQUEST_PROFILE=world-deps-provision` globally can inadvertently weaken runtime isolation if treated as an operator-facing knob; documentation and implementation should ensure the provisioning profile is internal-by-default.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `substrate world enable --provision-deps` contract surface
    - runtime `substrate world deps current sync|install` fail-early posture for system packages
    - world-agent provisioning request profile seam
  - Conflict: no
  - Resolution (explicit):
    - Sequence: ADR-0030 is a prerequisite baseline (APT provisioning + runtime fail-early for APT); ADR-0033 extends to pacman + probe + mismatch semantics.

- ADR: `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
  - Overlap surfaces:
    - Linux posture for system package provisioning (host-native unsupported; guest backend may allow)
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0009 currently references a legacy `substrate world deps provision` surface; it must be reconciled to the chosen provisioning entrypoint (`substrate world enable --provision-deps`) or explicitly scoped as historical/legacy.

- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `/etc/os-release` parsing + distro-family mapping + manager identifier set (host installer)
  - Conflict: no
  - Resolution (explicit):
    - Treat host package-manager detection and in-world provisioning probes as distinct contracts; share parsing/canonicalization rules (or a shared implementation) to avoid drift in family classification and manager identifiers.

- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - `/etc/os-release` parsing and canonicalization; “selected manager” strings (host persistence)
  - Conflict: no
  - Resolution (explicit):
    - Ensure the world OS probe does not reuse the host persistence schema; align only on safe parsing/canonicalization rules and manager identifier vocabulary where appropriate.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - `substrate world enable` CLI surface + invariants
  - Conflict: yes
  - Resolution (explicit):
    - ADR-0003 must be updated/extended to include the `--provision-deps` flag and to confirm it does not violate the “no host mutation” posture or `--home` semantics requirements.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `substrate world enable` helper discovery + staging behavior
  - Conflict: no
  - Resolution (explicit):
    - Coordinate sequencing to minimize merge conflicts in `crates/shell/src/builtins/world_enable/`; avoid coupling helper staging changes with deps provisioning logic in a single PR.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `substrate world enable` behavior around ensuring `world-agent` artifacts exist
  - Conflict: no
  - Resolution (explicit):
    - Keep “world enable robustness” changes orthogonal to “deps provisioning” changes; ensure both ADRs agree on failure/exit-code mapping for world enable.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - Same provisioning entrypoint + runtime fail-early posture
    - Same request-profile seam and guard-rail requirements
  - Conflict: yes
  - Resolution (explicit):
    - A/B options for “single authoritative contract wording”:
      - Option A: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` is authoritative for the shared CLI/exit-code/remediation contract across `apt|pacman`; `world-deps-apt-provisioning` owns only APT-specific protocol + execution details and links to the shared contract.
      - Option B: `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` remains authoritative for shared CLI surfaces and is extended to cover pacman; this pack becomes method-only and/or is merged/renamed.
    - Selected: Option A (minimizes duplicated shared CLI contract text and forces one place to define mixed-method and mismatch behavior).

- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `/etc/os-release` parsing rules; manager identifier vocabulary (`apt-get`, `pacman`, etc.)
  - Conflict: no
  - Resolution (explicit):
    - Reuse the safe parsing and canonicalization rules for the in-world probe where applicable; keep “host installer manager” vs “world OS manager” concepts distinct.

- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `/etc/os-release` parsing and canonicalization vocabulary (host persistence)
  - Conflict: no
  - Resolution (explicit):
    - Ensure parsing/canonicalization rules match the upstream contract to avoid conflicting persisted values vs probe-derived values.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Inventory schema approach: explicit `install.method=pacman` vs abstract system-packages mapping
  - DR-0002 — World OS probe strategy + parsing/canonicalization + Arch-family classification rules
  - DR-0003 — Pacman invocation + idempotency strategy (exact non-interactive command shape)
  - DR-0004 — Mismatch policy (mixed enabled sets; fail-closed vs partial provision)
  - DR-0005 — Cross-pack contract ownership boundary (confirm selected “Option A” resolution above and list required downstream doc updates)
- Spec updates required (if any):
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md` — update surface ownership to defer shared CLI contract to the selected authoritative `contract.md`
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` — update install-method set + remove contradictory runtime APT semantics; add pacman schema notes
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C0/C0-spec.md` — lock probe algorithm and mismatch classification rules
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C1/C1-spec.md` — lock pacman derivation + command contract + failure mapping
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/C2/C2-spec.md` — lock runtime fail-early scope + exact doc-update targets
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-provisioning-protocol-spec.md` — lock provisioning request profile semantics + guard rails (env filtering, isolation boundaries, timeouts)
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` — fix feature-dir link drift to this pack path after the pack is accepted

