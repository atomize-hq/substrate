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
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS0/ANS0-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS1/ANS1-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/ANS2/ANS2-spec.md`

### Edit
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
- `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/builtins/world_deps/inventory.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/src/builtins/world_deps/errors.rs`
- `crates/shell/src/builtins/health.rs`
- `crates/shell/tests/world_enable.rs`
- `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
- `crates/shell/tests/world_deps_apt_install_wdp5.rs`
- `crates/world-agent/src/service.rs`
- `scripts/substrate/world-enable.sh`
- `scripts/substrate/install-substrate.sh`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `docs/WORLD.md`
- `docs/INSTALLATION.md`
- `docs/USAGE.md`
- `docs/COMMANDS.md`
- `docs/cross-platform/wsl_world_troubleshooting.md`
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
- Change: `substrate world enable --provision-deps` provisions system-package world-deps via an in-world OS/manager probe (APT vs pacman) and fails deterministically on mismatches/unsupported backends.
  - Direct impact:
    - Operators can explicitly provision system packages required by the effective enabled world-deps set when the world OS is Arch-family (pacman) and the backend supports provisioning.
    - On unsupported worlds/backends, the command fails with actionable remediation that does not suggest the wrong manager and does not imply host OS mutation.
  - Cascading impact:
    - `world enable` must derive requirement sets from the same effective view as `world deps current ...` and must surface both the detected manager and the derived requirement set(s) (including `--dry-run` no-mutation guarantees).
    - The provisioning workflow must be manager-aware end-to-end: success/no-op messaging, mismatch errors, and `--verbose` output must not be “apt-shaped” when the world is pacman-based.
    - World-enable helper/installer flows that currently run `substrate world deps current sync` as part of provisioning must be reconciled so they do not produce misleading “next step: sync” guidance or create recursion/confusing failures when system-package items are enabled.
  - Contradiction risks:
    - Existing docs and scripts treat `world deps current sync` as the one-stop “apply deps” step; once system packages are provisioning-time, those steps become incomplete unless docs/scripts branch on install method and backend support.
    - Host installer package-manager selection (Linux prereqs via `install-substrate.sh`) uses `pacman` too; operator-facing messaging must keep “host pkg-manager” distinct from “world OS pkg-manager” to avoid wrong remediation.

- Change: Runtime `substrate world deps current sync|install` MUST NOT invoke OS package managers (`apt` or `pacman`) and must short-circuit with stable exit codes + remediation.
  - Direct impact:
    - Runtime world-deps becomes safe under hardened worlds and avoids implicit OS mutation.
    - When system-package items are in scope, operators receive a deterministic remediation that includes the exact provisioning command string.
  - Cascading impact:
    - The world-deps engine must represent “system-package required” as a stable operator-visible state (blocked vs missing vs present) and must attach a remediation string that is manager-aware and backend-aware.
    - `substrate health` / `substrate shim doctor` next-step guidance must be updated so it does not always recommend `world deps current sync` when the true next step is provisioning (or manual steps on unsupported backends).
    - Existing runtime APT implementation (`world_deps/surfaces.rs`) and its tests must be removed/repurposed so runtime never runs `apt-get` and the new behavior is enforced.
  - Contradiction risks:
    - If runtime short-circuiting is defined as “fail whenever any system-package items exist” (rather than “fail when system packages are missing/unprovisioned”), then `world enable` and installers that run `world deps current sync` can become permanently non-functional for mixed (system + script) enabled sets. Scope/trigger rules must be pinned to avoid dead-end workflows.

### Config / env vars / paths
- Change: World-deps inventory schema extends to `install.method=pacman` with `install.pacman: [<pkg>...]`.
  - Direct impact:
    - Inventory authors can express system-package requirements for Arch-family worlds without cross-distro package-name translation.
  - Cascading impact:
    - Shell inventory parsing/validation, JSON output fields, and schema docs must be updated to accept/emit `pacman` deterministically (ordered list + de-dup policy).
    - Operator docs must include “how to write” and “how it behaves” guidance that makes mismatch behavior explicit (e.g., enabling an APT-only item on an Arch world is a hard failure with remediation rather than an implicit translation attempt).
  - Contradiction risks:
    - Built-in inventory currently contains `install.method=apt` items; on Arch-family worlds these will become mismatch cases unless the operator overrides the inventory or disables built-ins. Guidance must avoid implying Substrate will translate package names across managers (non-goal).

- Change: Provisioning-time manager selection is derived from an in-world probe (not host PATH), using `/etc/os-release` plus manager presence checks.
  - Direct impact:
    - The selected manager reflects the world OS reality, reducing “apt-like” confusion on non-APT worlds.
  - Cascading impact:
    - The exact classification rules (Arch-family vs Debian/Ubuntu-family) and precedence between `/etc/os-release` and `command -v <manager>` must be decided and tested.
    - Canonical manager identifiers and “unknown” rendering should align with other planned host-detection/persistence work so operators see consistent vocabulary across installer output, persisted metadata, and in-world provisioning output.
  - Contradiction risks:
    - Multiple `/etc/os-release` parsers (host installer vs in-world probe vs doctor/capability surfaces) can drift and produce contradictory “OS identity” strings unless normalized and scoped explicitly.

### Policy / isolation / security posture
- Change: Provisioning-time OS mutation requires an execution isolation model that permits package-manager writes while preserving hardened runtime guarantees.
  - Direct impact:
    - Provisioning can succeed for system packages even when runtime executions are constrained to Substrate-managed writable surfaces.
  - Cascading impact:
    - The selection of “distinct world-agent request profile vs explicit guard rails” must be pinned and documented; if a request-profile is used, shell-side gating must ensure it is never used to mutate Linux host-native OS packages.
    - If any world-agent contract expectations change (even if only in documentation), `docs/WORLD.md` must reflect the provisioning execution model at the right level (without implying that runtime execution is less restricted).
  - Contradiction risks:
    - `SUBSTRATE_WORLD_REQUEST_PROFILE=world-deps-provision` exists as an advanced/testing escape hatch; provisioning-time behavior must not rely on operators setting this manually, and must not create a “hidden host mutation” path on Linux host-native.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `substrate world enable --provision-deps` entrypoint and base UX/exit-code taxonomy
    - runtime short-circuit posture for `world deps current sync|install`
    - provisioning execution isolation seam in `crates/world-agent/src/service.rs`
  - Conflict: yes
  - Resolution (explicit):
    - Sequencing boundary: ADR-0030 must land first (or concurrently) as the base provisioning workflow; ADR-0033 extends method coverage and MUST NOT redefine APT invocation details already owned by the ADR-0030 planning pack.
    - Shared contract authority must converge: this pack’s `contract.md` is manager-aware and therefore should become the single authoritative wording for the shared provisioning entrypoint + runtime invariants once pacman support lands; the ADR-0030 pack should defer to it for shared language and keep APT-specific details in its slice specs.

- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `/etc/os-release` parsing vocabulary and manager identifiers (`pacman`, `apt-get`, etc.)
    - `scripts/substrate/install-substrate.sh` (host prereq provisioning)
  - Conflict: yes (shared file + shared vocabulary)
  - Resolution (explicit):
    - Enforce a strict boundary: ADR-0031 is host-installer detection/override only; this pack is in-world probe + provisioning only.
    - Coordinate edits to `scripts/substrate/install-substrate.sh` so provisioning-related changes do not refactor the pkg-manager detection pipeline owned by ADR-0031.

- ADR: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - Overlap surfaces:
    - shared manager identifier vocabulary persisted to `install_state.json`
    - `/etc/os-release` canonicalization and “unknown” rendering expectations
  - Conflict: no (coordination-only), but merge-conflict risk exists in shared docs files
  - Resolution (explicit):
    - Keep “host platform metadata persistence” strictly out of this pack and align terminology in operator-facing docs so host vs world OS identity is not conflated.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh` helper staging/discovery expectations
    - `crates/shell/src/builtins/world_enable/runner/*` helper invocation UX
  - Conflict: yes (shared files)
  - Resolution (explicit):
    - Keep changes to helper invocation narrowly scoped (avoid refactoring discovery) and sequence after/with ADR-0034 to minimize merge conflicts; ensure `--dry-run` output remains stable across helper discovery changes.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh` and `scripts/substrate/install-substrate.sh` world-agent staging expectations
    - `crates/shell/src/builtins/world_enable/runner.rs` enable behavior and remediation
  - Conflict: yes (shared files)
  - Resolution (explicit):
    - Ensure “deps provisioning” additions do not obscure “world-agent artifact missing” early-remediation behavior; keep error classification deterministic (backend unavailable vs missing artifact vs provisioning unsupported).

- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` operator text and next-step guidance
  - Conflict: yes
  - Resolution (explicit):
    - When world is disabled, health/doctor must short-circuit and MUST NOT suggest provisioning-time deps actions; when world is enabled, health output must be method-aware (system packages may require provisioning, not runtime sync).

- ADR: `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
  - Overlap surfaces:
    - doctor/health messaging and its dependence on effective config provenance
  - Conflict: no (messaging-only), but ensure no contradictory “why disabled” attribution appears in provisioning guidance
  - Resolution (explicit):
    - Ensure any new remediation text added by this pack does not hardcode “(--no-world)” attributions and does not bypass the effective-config provenance model.

- ADR: `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
  - Overlap surfaces:
    - future Linux provisioning support surface and “no host OS mutation” invariant
  - Conflict: no (this pack keeps Linux host-native provisioning unsupported), but the capability seam must remain extensible
  - Resolution (explicit):
    - Keep Linux host-native provisioning fail-closed with explicit manual guidance; design the backend-capability check so a future guest-rootfs backend can opt into provisioning without changing operator-visible semantics.

- ADR: `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`
  - Overlap surfaces:
    - potential doctor/capability reporting of `supports_system_packages_provisioning` and in-world OS identity
  - Conflict: no
  - Resolution (explicit):
    - If this feature adds any new “capability gates” or doctor-visible fields, align naming and exit-code posture (`4` for unsupported) with ADR-0010’s contract proposal.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - shared provisioning entrypoint (`world enable --provision-deps`) and its `--dry-run`/`--verbose` UX
    - shared runtime short-circuit posture for system-package methods
    - `crates/world-agent/src/service.rs` provisioning execution isolation seam
    - shared operator-doc update targets and scripts (`docs/reference/world/deps/README.md`, `docs/internals/world/deps.md`, `scripts/substrate/world-enable.sh`, `scripts/substrate/install-substrate.sh`)
  - Conflict: yes
  - Resolution (explicit):
    - Dependency: this pack assumes the APT provisioning baseline and MUST NOT re-spec APT invocation details (owned by `world-deps-apt-provisioning` slice specs).
    - Contract ownership selection:
      - Option A: This pack’s `contract.md` is authoritative for the shared provisioning UX (manager-aware) and runtime system-package prohibition; the APT pack defers for shared text and remains authoritative for APT-only details.
      - Option B: The APT pack’s `contract.md` remains authoritative for the shared provisioning UX; this pack adds a second “overlay contract” for pacman and manager-detection, risking drift.
      - Selected: Option A (ADR-0033 makes the shared surface manager-aware; duplicating the base contract across two packs would create dual authority and high drift risk).

- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
  - Conflict: yes (shared file)
  - Resolution (explicit):
    - Keep this feature’s changes to `install-substrate.sh` limited to world-deps provisioning workflow integration/messaging and avoid modifying the pkg-manager detection pipeline owned by the ADR-0031 pack; sequence to minimize conflicts.

- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `docs/INSTALLATION.md`
  - Conflict: yes (shared files)
  - Resolution (explicit):
    - Keep host-metadata persistence changes isolated to their pack and avoid overlapping refactors; align terminology in `docs/INSTALLATION.md` so host vs world OS identity remains distinct.

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - `crates/world-agent/src/service.rs` request/response invariants and execution semantics
  - Conflict: yes (shared file; security-sensitive)
  - Resolution (explicit):
    - Coordinate changes to provisioning profiles/guard rails so they do not break process-capture plumbing; keep service hardening rules compatible with provisioning-time behavior without broadening runtime write surfaces.

- Planning Pack (archived reference): `docs/project_management/_archived/world_deps_selection_layer/`
  - Overlap surfaces:
    - legacy provisioning entrypoint (`world deps provision`) and “runtime blocked until provision” semantics
  - Conflict: yes
  - Resolution (explicit):
    - Treat ADR-0030/ADR-0033 as superseding the archived provisioning entrypoint; do not reintroduce `world deps provision` unless explicitly documented as an alias with a compat policy.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Inventory schema approach (explicit `install.method=pacman` vs abstract mapping) and its implications for built-ins.
  - DR-0002 — Deterministic in-world OS-family + manager probe rules (inputs, precedence, and disagreement handling).
  - DR-0003 — Deterministic pacman invocation contract (non-interactive flags, idempotency, ordering/de-dup, error mapping).
  - DR-0004 — Deterministic mismatch policy (mixed `apt`+`pacman` enabled sets; fail-closed vs partial provisioning; required remediation content).
  - DR-0005 — Provisioning execution isolation model (request profile vs guard rails; whether any `docs/WORLD.md` protocol notes are required).

- Spec updates required (if any):
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md` — keep operator-doc update targets and cross-pack contract ownership notes aligned to the selected resolution above.
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` — extend schema to include `pacman` and reconcile runtime semantics to remove the “runtime apt install” contract conflict (defer to the provisioning-time contract where appropriate).
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` — reconcile feature-directory link drift so there is exactly one authoritative planning-pack directory for ADR-0033.
  - `docs/reference/world/deps/README.md` — update headings/sections that currently imply runtime `current sync` provisions everything so they link to `contract.md` and describe provisioning-time system packages (manager-aware).
  - `docs/internals/world/deps.md` — update “APT installs vs hardening” and high-level flow sections to reflect provisioning-time system packages and add pacman-aware notes.

- Operator-doc update targets (exact paths + headings; must link to `contract.md` and avoid restating contract tables):
  - `docs/reference/world/deps/README.md`:
    - “Commands you will use”
    - “APT packages (current limitation in hardened worlds)” (must become manager-aware and reflect provisioning-time system packages)
  - `docs/internals/world/deps.md`:
    - “High-level flow” (remove runtime “apt first” execution description; describe provisioning-time system packages)
    - “APT installs vs hardening” (update to provisioning-time; add pacman notes)
  - `docs/USAGE.md`:
    - “Health” example output and next-step guidance (currently always suggests `world deps current sync`)
  - `docs/INSTALLATION.md`:
    - “Installer Options Reference” (`--sync-deps` semantics)
    - “Troubleshooting Highlights” (“World deps not present (Linux/macOS)” guidance)
  - `docs/COMMANDS.md`:
    - `substrate world enable` (flag surface and semantics for `--provision-deps`)
    - `substrate world deps current sync` and `substrate world deps current install` (runtime prohibition + remediation semantics)
  - `docs/cross-platform/wsl_world_troubleshooting.md`:
    - “World deps quick check” next-step guidance
  - `docs/WORLD.md`:
    - World-deps command list + any provisioning execution isolation notes (if added as part of DR-0005 selection)
