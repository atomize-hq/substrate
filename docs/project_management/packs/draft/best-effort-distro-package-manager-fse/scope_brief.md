---
pack_id: best-effort-distro-package-manager-fse
pack_version: v2
pack_status: extracted
source_ref: docs/project_management/packs/draft/best-effort-distro-package-manager/
execution_horizon:
  active_seam: SEAM-07
  next_seam: null
---

# Scope Brief - Best-Effort Distro Package Manager

- **Goal**: turn ADR-0031 into a fully implementable seam pack that preserves the deeply researched hosted-installer contract: safe Linux distro detection, deterministic package-manager selection, explicit overrides, stable reporting, wrapper parity, Linux-direct and macOS-Lima-backed validation evidence, and downstream handoff.
- **Why now**: the source plan already resolved the contract, task graph, validation topology, and downstream boundaries; the remaining failure mode was that the first extractor pass collapsed several independently reviewable contracts into seams that were too coarse for reliable downstream decomposition.
- **Primary user(s) + JTBD**:
  - Linux operators need to see which package manager was selected, why it was selected, and how to override or remediate failures.
  - maintainers need a seam pack whose decomposition can preserve contract ownership across implementation, tests, docs, and checkpoint evidence without rediscovering planning truth.
- **In-scope**:
  - safe `/etc/os-release` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH` input handling
  - normalized distro field and `<unknown>` sentinel contract
  - distro-family mapping and decision-line reporting
  - explicit override selectors, precedence, and fail-closed posture
  - deterministic PATH probing, multi-manager warning, and no-manager remediation
  - wrapper exit-status pass-through and operator/env-doc propagation
  - hermetic repo harness, thin smoke wrapper, manual evidence playbook, and macOS-hosted Lima-backed verification
  - single checkpoint boundary with compile parity, quick CI testing, Linux behavior smoke, and macOS-hosted behavior evidence
  - downstream handoff to `persist-detected-linux-distro-pkg-manager`
- **Out-of-scope**:
  - direct macOS-native package-manager-selection logic outside the Lima-backed hosted-install path
  - Windows behavior changes
  - runtime crate, world backend, or telemetry-schema changes
  - persistence into `install_state.json`
  - new config files or persistent config keys
  - network calls during detection
  - `scripts/substrate/dev-install-substrate.sh` behavior changes
- **Success criteria**:
  - safe parser and alternate-input contract are published without ambiguity
  - mapping/reporting, override, fallback, warning, and remediation semantics are decomposable into bounded seam-local work
  - wrapper/docs and validation topology are not treated as cleanup buckets; each has explicit ownership
  - macOS-hosted runs that traverse Lima-backed Linux installation are explicitly covered by validation and checkpoint evidence
  - checkpoint and downstream handoff are represented as real conformance work rather than implicit tail work
  - downstream seam/slice/subslice planning can proceed without inventing new contracts or redistributing ownership
- **Constraints**:
  - package-manager decision logic remains Linux-scoped, but macOS-hosted installs that reach that logic through Lima require behavior coverage
  - preserve source-time invariants in `scripts/substrate/install-substrate.sh`
  - keep contract vocabulary exact: manager spellings, `pkg_manager.source`, `<unknown>`, warning line, decision line, exit taxonomy
  - preserve the source pack's single checkpoint boundary at the end of the feature
  - keep extractor output at seam-brief depth only
- **External systems / dependencies**:
  - `/etc/os-release` semantics on Linux hosts
  - host `PATH` and supported package-manager binaries
  - macOS Lima VM bootstrap and forwarding path used by hosted installs
  - shared exit-code taxonomy
  - downstream pack `persist-detected-linux-distro-pkg-manager`
  - related ADR boundaries with ADR-0030, ADR-0032, ADR-0035
  - CI infrastructure for compile parity, quick test, feature smoke, and macOS-hosted validation
- **Known unknowns / risks**:
  - seam-local decomposition could over-couple parser/input and mapping/reporting if the handoff contracts are not kept crisp
  - override and fallback branches share one script surface; decomposition must preserve one decision pipeline
  - validation work can drift into duplicate authorities unless the repo harness remains authoritative and the smoke wrapper stays thin
  - macOS-host and Linux-guest behavior can drift if the Lima-backed path is treated as parity-only instead of behavior coverage
  - checkpoint/handoff work can be under-planned unless its evidence and downstream stale-trigger responsibilities are explicit
- **Assumptions**:
  - the source planning pack is approved input and does not carry an open extraction-blocking remediation
  - the accepted BEDPM0-3 slice research is authoritative evidence for seam extraction
  - the four canonical distro families and fixed manager vocabulary remain the v1 contract target
