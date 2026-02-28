# world-deps-apt-provisioning — impact map

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- Spec manifest:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/world-deps-apt-provisioning/"` (strict packs only).

### Create
- `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/manual_testing_playbook.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`

### Edit
- `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`
- `crates/shell/src/execution/cli.rs`
- `crates/shell/src/execution/platform/mod.rs`
- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/builtins/world_enable/runner.rs`
- `crates/shell/src/builtins/world_enable/runner/log_ops.rs`
- `crates/shell/src/builtins/world_deps/surfaces.rs`
- `crates/shell/src/builtins/health.rs`
- `crates/shell/src/execution/home_bootstrap.rs`
- `crates/shell/tests/world_enable.rs`
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
- Change: Add provisioning-time workflow for APT-backed world-deps.
  - Direct impact:
    - Operators gain an explicit, auditable provisioning path for system packages on supported guest backends.
    - Runtime world-deps no longer attempts OS mutation for APT items under hardened execution paths.
  - Cascading impact:
    - `substrate world enable` must grow a new flag surface and a deterministic “derive → (dry-run|execute) → report” UX, including stable remediation text and exit-code mapping.
    - `world enable` must clearly distinguish “world backend provisioning” vs “deps provisioning” when both are invoked together (especially under `--dry-run` and `--verbose`).
  - Contradiction risks:
    - Existing operator docs and helper scripts currently treat `world deps current sync` as “apply everything”; if APT-backed items now require provisioning, “next step: run sync” messaging becomes misleading unless it branches on method/backends.

- Change: Runtime `substrate world deps current sync|install` fail-early posture for APT items.
  - Direct impact:
    - When APT-backed items are in scope, the command exits non-zero with actionable remediation (must include the exact command string) instead of attempting `apt-get` and failing with hardening errors.
  - Cascading impact:
    - Exit code `4` becomes the stable operator signal for “unsupported/unmet prerequisites” in these paths; `5` becomes reserved for genuine safety violations (wrapper collisions, deny enforcement, etc.).
    - Tests and any automated scripts relying on runtime APT installs must be updated to expect the new short-circuit behavior.
  - Contradiction risks:
    - The existing `world-deps-packages-bundles-contract` contract text currently states runtime `current sync|install` applies APT first; it must be updated or it will directly contradict the new behavior.

- Change: Platform-specific support matrix for provisioning.
  - Direct impact:
    - Linux host-native: provisioning-time APT is unsupported by default; runtime guidance must explicitly state “no host OS mutation”.
    - macOS Lima guest: provisioning-time APT is supported (guest OS mutation).
    - Windows WSL: ADR assumption is “supported when world enable exists”; until then, remediation must be explicit about unsupported surfaces.
  - Cascading impact:
    - `substrate world enable` must produce consistent exit codes and messaging across OSes (including when the world backend is unavailable vs when provisioning is unsupported by policy/posture).
  - Contradiction risks:
    - `substrate world enable` is currently unsupported on Windows, but ADR-0030’s operator contract includes Windows behavior assumptions; specs must make the interim posture deterministic.

### Config / env vars / paths
- Change: Provisioning derives APT requirements from the effective enabled world-deps set; no new config keys.
  - Direct impact:
    - Operators do not need to learn a new selection surface; provisioning uses the same inventory + enabled resolution as `world deps current ...`.
  - Cascading impact:
    - Provisioning must reuse/align with the existing bundle expansion + de-dup semantics; drift here creates “enabled list looks right but provisioning installs the wrong set” failures.
  - Contradiction risks:
    - Runtime short-circuit scope is currently ambiguous for `deps current install <ITEM...>` vs enabled-set-based sync; the slice spec must pick exactly one rule or UX will differ across commands.

- Change: Provisioned-state tracking may introduce a new file-format surface (DR-0002).
  - Direct impact:
    - If a persisted state file is chosen, it becomes a new operator-visible (and testable) artifact with path + schema + absence semantics.
  - Cascading impact:
    - Docs must name the exact path(s) and define host-side vs world-side ownership to avoid silently writing state into protected locations in hardened worlds.
  - Contradiction risks:
    - A probe-only strategy can be correct but may be slower/noisier; a state-file strategy can drift unless its update/invalidations are explicitly defined and tested.

### Policy / isolation / security posture
- Change: Provisioning-time OS mutation must be allowed only under explicit guard rails.
  - Direct impact:
    - Hardened runtime execution remains fail-closed; provisioning is the only sanctioned path for APT/dpkg mutation.
  - Cascading impact:
    - The world-agent request-profile seam used for provisioning must not become a general-purpose “escape hatch”; guard rails must be enforceable and auditable.
  - Contradiction risks:
    - Existing `world-deps-provision` profile behavior (and any shell env hooks enabling it) can conflict with “no host OS mutation” unless the shell blocks the provisioning profile on Linux host-native and the agent enforces profile constraints.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - `substrate world enable` flag surface and `$SUBSTRATE_HOME` semantics (`--home`, dry-run, verbose)
    - `crates/world-agent/src/service.rs` (explicitly listed by ADR-0003 as a world env plumbing hot spot)
  - Conflict: yes
  - Resolution (explicit):
    - `--provision-deps` must be defined as an additive `world enable` flag that fully honors ADR-0003’s `--home` contract and does not write disallowed env exports; if ADR-0003 is treated as authoritative for the base `world enable` surface, this feature must be sequenced after (or implemented concurrently with) ADR-0003 updates to the `world enable` CLI contract.

- ADR: `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
  - Overlap surfaces:
    - Linux system package provisioning posture (no host OS mutation; guest-only provisioning)
    - CLI naming for provisioning (`world deps provision` vs `world enable --provision-deps`)
  - Conflict: yes
  - Resolution (explicit):
    - Align on ADR-0030’s selected CLI surface: treat `substrate world enable --provision-deps` as the canonical provisioning entrypoint; ADR-0009 must be updated to either (a) use the same entrypoint for guest-rootfs provisioning, or (b) explicitly define a stable alias/compat surface if `world deps provision` is retained.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - Same provisioning entrypoint and runtime fail-early posture; same request-profile seam and guard-rail requirements
  - Conflict: no
  - Resolution (explicit):
    - Sequencing boundary: ADR-0030 must land first (or concurrently) to define the provisioning-time system-packages workflow and runtime prohibition; ADR-0033 extends method coverage and must not redefine shared exit-code/remediation guarantees.

- ADR: `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md`
  - Overlap surfaces:
    - `crates/shell/src/builtins/world_enable/runner/paths.rs` helper discovery and `$SUBSTRATE_HOME/scripts/substrate/...` staging expectations
  - Conflict: no
  - Resolution (explicit):
    - Keep helper-discovery/staging refactors orthogonal to deps provisioning logic; ensure `world enable --dry-run` output remains stable even when helper discovery changes.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - `crates/shell/src/builtins/world_enable/runner.rs` enable runner behavior and its reliance on helper scripts/artifacts
    - `scripts/substrate/world-enable.sh` and `scripts/substrate/install-substrate.sh` (enable helper + artifact staging)
  - Conflict: no
  - Resolution (explicit):
    - Ensure “deps provisioning” changes do not obscure/override “missing world-agent artifact” early-remediation behavior; keep failure classification deterministic (backend unavailable vs missing artifact vs unsupported provisioning).

- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` operator text and “next steps” guidance
  - Conflict: yes
  - Resolution (explicit):
    - When world is disabled, diagnostics must short-circuit and avoid suggesting provisioning-time deps actions; when world is enabled, health output must remain actionable and method-aware (system packages may require provisioning rather than runtime sync).

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - `crates/world-agent/src/service.rs` behavior and request/response invariants
    - world-agent service configuration surfaces referenced by that pack (systemd unit changes, capability bounding)
  - Conflict: yes
  - Resolution (explicit):
    - Coordinate changes in `crates/world-agent/src/service.rs` so “provisioning profile” guard rails do not break process-capture plumbing; keep service hardening rules and added capabilities compatible with provisioning-time behavior (no accidental broadening of runtime write surfaces).

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - Same provisioning entrypoint + runtime fail-early posture
    - Same request-profile seam and guard-rail requirements
    - Shared operator-doc update targets (`docs/reference/world/deps/README.md`, `docs/internals/world/deps.md`)
  - Conflict: yes
  - Resolution (explicit):
    - A/B options for “single authoritative contract wording”:
      - Option A: `add-non-apt-system-package-provisioning-support/contract.md` is authoritative for the shared CLI/exit-code/remediation contract across system-package methods; this pack owns only APT-specific derivation/execution details and defers shared wording.
      - Option B: `world-deps-apt-provisioning/contract.md` is authoritative for the shared CLI surface introduced by ADR-0030; non-APT packs extend it (or link to it) and own only method-specific additions.
    - Selected: Option B (ADR-0030 introduces the shared CLI surface; avoid duplicating the base contract and forcing downstream edits to retarget authoritative ownership).

- Planning Pack: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
  - Overlap surfaces:
    - `scripts/substrate/install-substrate.sh` (sourced by `world-enable.sh`)
  - Conflict: yes
  - Resolution (explicit):
    - Sequence or isolate edits: keep world-deps provisioning-related changes narrowly scoped (avoid refactoring installer pkg-manager selection logic in this feature) to reduce merge conflicts with ADR-0031/ADR-0032 packs.

- Planning Pack (archived reference): `docs/project_management/_archived/world_deps_selection_layer/`
  - Overlap surfaces:
    - Legacy provisioning UX (`world deps provision`) and “sync blocked until provision” semantics
  - Conflict: yes
  - Resolution (explicit):
    - Treat ADR-0030 as superseding the archived contract for system-package provisioning entrypoints; ensure current operator docs and contracts do not reintroduce the archived `provision` subcommand unless it is explicitly retained as an alias with a documented compat posture.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Confirm provisioning entrypoint selection (record ADR-0030 Option A vs B selection as the single decision outcome).
  - DR-0002 — Provisioned-state tracking strategy (probe-only vs persisted state file; if file: path + schema + ownership).
  - DR-0003 — Provisioning execution profile isolation model (how the provisioning request profile is selected, what it relaxes, and what guard rails prevent misuse).
- Spec updates required (if any):
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` — remove/replace contradictory runtime APT “apply apt first” semantics; defer to this feature’s contract for APT provisioning/runtime prohibition.
  - `docs/reference/world/deps/README.md` — update APT section to the new provisioning-time workflow and align runtime sync/install guidance.
  - `docs/internals/world/deps.md` — update internal flow notes (APT execution no longer occurs in runtime sync/install) and point to the provisioning-time model.
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md` — if cross-pack contract ownership boundaries change (e.g., non-APT packs attempting to own shared CLI surfaces), update ownership to preserve “exactly one authoritative doc per surface”.

