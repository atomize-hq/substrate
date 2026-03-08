# add-non-apt-system-package-provisioning-support — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- Spec manifest:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`

## Touch set (explicit)

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None`.
- The Touch Set must include at least one non-None entry total across all sections.

### Create
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/session_log.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/quality_gate_report.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`

### Edit
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/builtins/world_deps/inventory.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
- `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/world-agent/src/service.rs`
- `crates/shell/tests/world_deps_inventory_validation_wdp0.rs`
- `crates/shell/tests/world_deps_inventory_views.rs`
- `crates/shell/tests/world_deps_current_dry_run_wdp3.rs`
- `crates/shell/tests/world_deps_apt_install_wdp5.rs`
- `crates/shell/tests/world_enable.rs`
- `scripts/substrate/world-enable.sh`
- `scripts/substrate/install-substrate.sh`
- `docs/reference/world/deps/README.md`
- `docs/internals/world/deps.md`
- `docs/CONFIGURATION.md`
- `docs/INSTALLATION.md`
- `docs/COMMANDS.md`
- `docs/WORLD.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior/UX)

### CLI / UX
- Change: `substrate world enable --provision-deps` stops being effectively APT-only and becomes manager-aware for system-package items.
  - Direct impact:
    - Arch-family worlds can provision `install.method=pacman` items through the same explicit provisioning entrypoint.
    - Unsupported backends and manager mismatches must fail with one deterministic exit-`4` path that names the world-side problem and does not imply host mutation.
  - Cascading impact:
    - `crates/shell/src/execution/cli.rs`, `crates/shell/src/builtins/world_enable/runner.rs`, `crates/shell/src/builtins/world_enable/runner/helper_script.rs`, `crates/shell/src/builtins/world_enable/runner/log_ops.rs`, and `scripts/substrate/world-enable.sh` must all agree on the new flag flow, dry-run output, and verbose output.
    - `scripts/substrate/install-substrate.sh` and `docs/INSTALLATION.md` must stop presenting post-provision `world deps current sync` as sufficient when enabled system-package items still require provisioning-time routing.
    - `docs/COMMANDS.md` must stop advertising drifted `--prefix` wording for `substrate world enable` while adding `--provision-deps`.
  - Contradiction risks:
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` currently owns the shared provisioning entrypoint for APT-backed items only.
    - `docs/COMMANDS.md` still documents `substrate world enable` with `--prefix`, while the actual CLI uses `--home`.

- Change: runtime `substrate world deps current sync|install` must treat `pacman` the same way ADR-0030 already treats `apt`: provisioning-time only, never runtime OS mutation.
  - Direct impact:
    - Runtime commands must fail early for in-scope `install.method=pacman` items with the same exact remediation command string: `substrate world enable --provision-deps`.
    - `current install <ITEM...>` still needs an explicit-item scope rule so pacman-backed enabled items do not poison unrelated explicit installs.
  - Cascading impact:
    - `crates/shell/src/builtins/world_deps/surfaces.rs` must stop encoding “APT first, script second” as the only system-package runtime path and instead treat both `apt` and `pacman` as provisioning-only methods.
    - `crates/shell/tests/world_deps_current_dry_run_wdp3.rs` and `crates/shell/tests/world_deps_apt_install_wdp5.rs` must be repurposed from “runtime apt install happens” to “runtime system-package install never happens.”
    - `docs/internals/world/deps.md`, `docs/reference/world/deps/README.md`, and `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` must stop describing runtime APT mutation as the normative behavior.
  - Contradiction risks:
    - `docs/internals/world/deps.md` still says the install plan is `apt` first, `script` second.
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` still models `install.method=apt` as a runtime install method, which becomes incomplete once `pacman` exists but runtime system-package mutation is prohibited.

- Change: world-deps inventory gains a second system-package method.
  - Direct impact:
    - authors can declare `install.method=pacman` plus `install.pacman`.
    - list/show/JSON/YAML output must surface `pacman` without collapsing it into `apt` or `manual`.
  - Cascading impact:
    - `crates/shell/src/builtins/world_deps/inventory.rs` and `crates/shell/tests/world_deps_inventory_validation_wdp0.rs` must define the exact schema and validation failures.
    - `crates/shell/tests/world_deps_inventory_views.rs` must cover the new method string in inventory views.
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` must either defer schema authority for `pacman` to the new spec or be updated so it no longer states `apt | script | manual` as the only valid methods.
  - Contradiction risks:
    - the upstream implemented contract still hard-codes `apt | script | manual`.
    - the feature pack still has an unresolved built-in-inventory scope, so adding schema support without deciding built-in examples leaves authoring expectations ambiguous.

### Config / env vars / paths
- Change: no new config key and no new environment variable are introduced; the enabled-set and the existing provisioning request-profile plumbing stay in place.
  - Direct impact:
    - operators still drive the workflow through inventory plus enabled-set plus `substrate world enable --provision-deps`.
    - operators must not be told to set `SUBSTRATE_WORLD_REQUEST_PROFILE` manually.
  - Cascading impact:
    - `docs/CONFIGURATION.md` must explicitly position `SUBSTRATE_WORLD_REQUEST_PROFILE=world-deps-provision` as an advanced/internal execution hook, not the operator-facing contract.
    - `crates/shell/src/execution/routing/dispatch/world_ops.rs` must keep profile scoping narrow so pacman provisioning is selected from the in-world probe result, not from host env or host PATH.
    - `docs/WORLD.md` must remain additive-only if it mentions the request `profile` field; no new protocol or request field can be implied by this feature.
  - Contradiction risks:
    - `scripts/substrate/install-substrate.sh` also uses `pacman`, but only for host prerequisite installation; that host-side vocabulary cannot become the source of truth for in-world provisioning routing.

- Change: the world OS probe becomes a contract-bearing path boundary.
  - Direct impact:
    - manager selection must use in-world `/etc/os-release` plus in-world `command -v pacman`, not host detection.
  - Cascading impact:
    - `crates/shell/src/builtins/world_enable/runner.rs`, `scripts/substrate/world-enable.sh`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`, and `crates/world-agent/src/service.rs` must all preserve the “probe inside the world” invariant.
    - platform smoke and manual validation must cover at least one supported Arch-family path and one mismatch or unsupported path.
  - Contradiction risks:
    - `tests/installers/pkg_manager_container_smoke.sh` and ADR-0031 validate host installer detection by reading host/container `/etc/os-release`; that is similar vocabulary but a different contract surface.

### Policy / isolation / security posture
- Change: provisioning-time mutation broadens from “APT/dpkg exception” to “explicit system-package provisioning exception,” but only for the named provisioning flow.
  - Direct impact:
    - `world-deps-provision` remains the only request profile allowed to escape the default always-isolated runtime path for OS mutation.
    - Linux host-native still must not mutate the host OS.
  - Cascading impact:
    - `crates/world-agent/src/service.rs` must generalize the existing profile commentary and guard rails from APT-only language to manager-agnostic system-package provisioning language.
    - `crates/shell/tests/world_enable.rs` must keep exercising the “no writes on dry-run / unsupported path” guarantees after the new routing is added.
    - `platform-parity-spec.md`, `manual_testing_playbook.md`, and the three smoke scripts must make supported-vs-unsupported platform behavior explicit.
  - Contradiction risks:
    - active tracing work also touches `crates/world-agent/src/service.rs`, so provisioning-profile changes can silently regress trace coverage or request semantics if they are not kept orthogonal.
    - ADR-0010 expects capability divergence to be surfaced clearly; silent fallback from pacman to apt or host mutation would violate that posture.

## Cross-queue scan (ADRs + Planning Packs)

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - shared `substrate world enable --provision-deps` entrypoint
    - shared runtime “no OS package manager at runtime” invariant
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
  - Conflict: yes
  - Resolution (explicit):
    - Option A: keep `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` as the single authority for the shared provisioning UX and make this feature own only pacman-specific deltas.
    - Option B: promote this feature’s `contract.md` to the single authority for the now manager-aware shared provisioning UX, and reduce the APT pack’s contract to APT-specific rules plus deferral.
    - Selected: Option B.
    - Why selected: `ADR-0033` is the first source that spans more than one system-package manager, and `pre-planning/spec_manifest.md` already assigns the manager-aware shared contract to this feature’s `contract.md`.

- ADR: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - `/etc/os-release` vocabulary
    - `pacman` naming and source selection wording
  - Conflict: yes
  - Resolution (explicit):
    - Option A: reuse host installer package-manager detection for guest provisioning decisions.
    - Option B: keep guest provisioning detection fully in-world and never use host installer detection, `PKG_MANAGER`, or host PATH to route guest provisioning.
    - Selected: Option B.
    - Why selected: `ADR-0033` explicitly requires an in-world probe and forbids host-PATH-based routing.

- ADR: `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
  - Overlap surfaces:
    - Linux no-host-mutation posture
    - future guest-world Linux support for provisioning-time system packages
  - Conflict: no
  - Resolution (explicit):
    - Keep current Linux host-native behavior unsupported for provisioning and preserve exit `4`.
    - Allow future Linux guest-rootfs work to reuse the same manager-aware provisioning entrypoint without changing the CLI or exit-code contract.

- ADR: `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`
  - Overlap surfaces:
    - backend capability reporting expectations
    - supported-vs-unsupported provisioning behavior
  - Conflict: no
  - Resolution (explicit):
    - No new doctor or trace surface is introduced in this ADR.
    - The feature must still keep unsupported backends explicit and scriptable, not silent or heuristic.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
    - `crates/world-agent/src/service.rs`
  - Conflict: yes
  - Resolution (explicit):
    - Option A: leave the APT pack as the shared contract owner and make this pack APT-aware by reference only.
    - Option B: move shared manager-aware contract ownership to this pack and make the APT pack defer on shared CLI/runtime semantics.
    - Selected: Option B.
    - Sequencing boundary: the APT pack still owns APT-specific derivation and tests; this pack owns the multi-manager operator contract.

- Planning Pack: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh`
    - manager identifier vocabulary
    - `/etc/os-release` terminology
  - Conflict: no
  - Resolution (explicit):
    - Keep host install-state persistence and in-world provisioning routing separate.
    - Shared files can rebase cleanly if this feature avoids touching installer-state persistence helpers.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/tests/world_enable.rs`
  - Conflict: yes
  - Resolution (explicit):
    - Keep `--provision-deps` flag plumbing and world-agent-staging preflight orthogonal.
    - Sequence helper/staging changes first when possible; otherwise rebase this feature as a narrow flag-and-routing change rather than a helper-discovery refactor.

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - `crates/world-agent/src/service.rs`
  - Conflict: yes
  - Resolution (explicit):
    - Preserve existing request/response and trace assumptions while broadening the provisioning-profile commentary from APT-only to system-package-manager-agnostic language.

### Relevant Archived Work
- Archived pack/ADR context: `docs/project_management/_archived/next/linux_guest_rootfs_backend/`
  - Overlap surfaces:
    - Linux future guest provisioning posture
  - Conflict: no
  - Resolution (explicit):
    - Treat it as future enablement only; this ADR does not reopen host-native Linux mutation.

## Follow-ups (explicit)

- Decision Register entries required:
  - `DR-0004` — pacman runnable-wrapper and present-semantics scope.
  - Option A: `install.method=pacman` supports runnable packages in v1, so wrapper/present semantics from host-visible hardening must be extended beyond APT.
  - Option B: v1 pacman support is constrained so runnable-wrapper parity is not implied yet.
  - Reason this is needed: the current spec set selects a pacman schema doc and provisioning docs, but the upstream runnable-wrapper contract is still APT-specific.

- Spec updates required:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md` — explicitly state whether runnable pacman-backed packages are in scope in v1.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` — pin the single-authority handoff from the APT pack and define the exact mixed-manager failure rule.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md` — lock the exact Windows posture instead of leaving it assumption-only.

- Additional ownership gaps to close before quality gate:
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` still points at flat pack paths instead of the current `pre-planning/` layout; reconcile those links during pack authoring.
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` and `docs/internals/world/deps.md` currently describe runtime APT behavior that conflicts with the runtime no-system-package-mutation posture; one authoritative truth must remain after reconciliation.
