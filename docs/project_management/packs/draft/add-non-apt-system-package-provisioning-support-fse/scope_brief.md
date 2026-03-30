---
pack_id: add-non-apt-system-package-provisioning-support-seam-pack-v2-3
pack_version: v1
pack_status: extracted
source_ref: add-non-apt-system-package-provisioning-support.zip
execution_horizon:
  active_seam: SEAM-3
  next_seam: SEAM-4
---

# Scope Brief - Add non-APT system-package provisioning support

- **Goal**:
  - Make world-deps system-package provisioning manager-aware so pacman-backed items can be provisioned on supported worlds while runtime system-package mutation stays prohibited.
- **Why now**:
  - The source planning pack already found that the original three-slice model was too coarse. It split the work into five accepted slices and a prerequisite contract workstream because probe routing, schema support, provisioning execution, runtime fail-early behavior, and validation/doc reconciliation each carry distinct churn and verification risk.
- **Primary user(s) + JTBD**:
  - Operators running `substrate world enable --provision-deps` who need one deterministic way to provision APT-backed or pacman-backed prerequisites inside supported worlds.
  - Inventory authors who need an additive way to describe pacman-backed system packages without a translation layer.
  - Maintainers and support engineers who need Linux/macOS/Windows behavior, exit codes, and remediation wording to remain explicit and fail-closed.
- **In-scope**:
  - Shared manager-aware CLI/runtime contract for `substrate world enable --provision-deps` and runtime `substrate world deps current sync|install`
  - In-world world-manager probe and support gate using `/etc/os-release` plus in-world package-manager availability
  - Additive `install.method=pacman` plus `install.pacman` schema support and inventory-view rendering
  - Provisioning-time requirement derivation, mixed-manager rejection, request-profile boundary, and exact pacman command shape
  - Runtime read-only presence probes, explicit-item scope rules, and manager-aware fail-early remediation
  - Platform parity posture, smoke/manual evidence, and reconciliation targets for overlapping ADR and documentation surfaces
- **Out-of-scope**:
  - Any new config key, environment variable, structured log field, trace field, protocol field, or agent API request field
  - Host OS mutation on Linux host-native or Windows backends
  - Manager fallback, distro-translation layers, AUR helpers, lock-file recovery, or pacman retries
  - Runtime `apt` or `pacman` mutation in `deps current sync|install`
  - Pacman runnable-wrapper generation or widening pacman-backed packages beyond non-runnable prerequisites in v1
  - New built-in pacman inventory catalog entries
- **Success criteria**:
  - One authoritative manager-aware contract exists for provisioning and runtime fail-early behavior
  - One deterministic in-world probe and support gate chooses `apt`, chooses `pacman`, or fails closed
  - `install.method=pacman` and `install.pacman` are additive, validated, and visible in list/show/JSON/YAML views
  - Provisioning executes exactly one supported manager path with stable ordering, no partial mixed-manager behavior, and no host mutation
  - Runtime system-package handling stays read-only and emits deterministic remediation that points back to `substrate world enable --provision-deps`
  - Linux, macOS, and Windows validation evidence plus shared-doc reconciliation targets are explicit and aligned
- **Constraints**:
  - Detect the world package manager in-world only; do not route from host PATH, host installer detection, or host package-manager state
  - Keep Linux host-native and Windows provisioning unsupported and fail-closed with exit `4`
  - Preserve additive inventory compatibility on `version: 1`; do not bump inventory schema version
  - Preserve the existing enabled-set, bundle-expansion, and merge semantics owned by the upstream world-deps bundles contract
  - Keep macOS default smoke aligned to the repo-default Ubuntu-based Lima guest while treating Arch-family pacman-success evidence as manual-only in this pack
- **External systems / dependencies**:
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - Shared code surfaces in `crates/shell`, `crates/world-agent`, and `scripts/substrate/world-enable.sh`
- **Known unknowns / risks**:
  - Overlapping ADR and planning-pack docs can still present a second truth until reconciliation lands
  - Shared-file overlap with adjacent staging/tracing work can stale the provisioning-wiring basis before seam-local planning begins
  - Arch-family pacman-success evidence on macOS is manual-only and depends on a non-default Arch Lima fixture
  - Older runtime and doc surfaces still encode APT-only assumptions that must not leak back in during implementation
- **Assumptions**:
  - The source pack's accepted five-slice model is a trustworthy decomposition signal for delivery seams
  - The source pack's prerequisite contract/decision workstream is substantive enough to stand as its own seam
  - The extractor should preserve deep-research detail while removing slice files from the output contract
  - Active and next seam policy stays at one `active` seam and one `next` seam by default
