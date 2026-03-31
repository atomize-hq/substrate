---
pack_id: world-disabled-diagnostics-seam-extraction
pack_version: v1
pack_status: extracted
source_ref: /mnt/data/world-disabled-diagnostics.zip
execution_horizon:
  active_seam: SEAM-2
  next_seam: SEAM-3
---

# Scope Brief - World Disabled Diagnostics

- **Goal**:
  Normalize the deep-researched `world-disabled-diagnostics` pack into governance-ready feature seams that preserve the source pack's contract fidelity, cross-platform validation posture, and sequencing logic without carrying over authoritative slice decomposition.
- **Why now**:
  The source pack already proves that ADR-0036 has multiple coupled execution surfaces: shared effective-config classification, shim-doctor disabled-mode behavior, health summary behavior, and cross-platform parity. Extracting seams now makes the pack safer to hand off into downstream seam-local planning without losing the source pack's depth.
- **Primary user(s) + JTBD**:
  - Operators running host-only-by-choice installs who need diagnostics to distinguish **disabled** from **broken**.
  - CLI/tooling maintainers who need one stable machine-readable contract for JSON outputs and exact disabled-mode text lines.
  - Future diagnostics-related packs (attribution, json envelope work, provisioning guidance) that need explicit handoff points instead of implicit behavior coupling.
- **In-scope**:
  - Shared `effective_world_enabled` resolution and config-error posture for `substrate shim doctor` and `substrate health`
  - Disabled/skipped no-probe behavior for shim doctor
  - Health summary derivation and guidance suppression when world is disabled
  - Additive JSON status contracts, exact disabled-mode operator copy, and cross-platform validation evidence
  - Threading to adjacent queued work called out in the source pack (`ADR-0037`, `json-mode`, provisioning-related packs)
- **Out-of-scope**:
  - Seam-local slice creation, kickoff prompts, or authoritative sub-slices
  - New config keys or environment variables
  - "Why disabled" attribution fields or telemetry/log schema changes
  - JSON envelope reshaping outside the additive status fields already defined in the source pack
  - Changes to enabled-mode provisioning semantics beyond preserving fail-visible behavior
- **Success criteria**:
  - Exactly one active seam and one next seam are named from the real critical path
  - Every extracted seam keeps the source pack's behavior invariants, risks, and verification depth
  - Contract ownership and dependency directionality are explicit in `threading.md`
  - Pack-level review surfaces show the actual CLI/product behavior expected to land
  - Governance scaffolds reserve seam-exit intent and post-exec closeout without creating slices
- **Constraints**:
  - External authoritative inputs stay external-authoritative: `docs/reference/env/contract.md`, `docs/CONFIGURATION.md`, and `EXIT_CODE_TAXONOMY`
  - Disabled mode is explicit and non-error, with no world backend or world-deps probes
  - Enabled-but-broken remains fail-visible
  - JSON posture is additive-only; new fields are canonical and legacy error fields are omitted only where the source contract says so
  - Linux/macOS/Windows semantics must remain aligned
- **External systems / dependencies**:
  - `resolve_effective_config` and diagnostics routing inside `crates/shell/src/execution/*`
  - World backend probe surfaces (world-agent socket / forwarder pipe / `substrate world doctor --json` subprocess)
  - CLI text/JSON renderers in `shim_doctor` and `health`
  - `docs/USAGE.md`, manual playbook, smoke scripts, and CP1 cross-platform checkpoint expectations from the source pack
  - Adjacent in-flight surfaces named by the source pack: `ADR-0037`, `ADR-0030`, `ADR-0033`, `json-mode`, provisioning packs
- **Known unknowns / risks**:
  - Hidden probe paths could survive disabled-mode short-circuiting unless seam-local review makes the boundary operationally explicit
  - Health summary aggregation could still key off legacy error strings instead of status enums
  - Shared-file churn (`health.rs`, `shim_doctor/report.rs`) raises revalidation risk with adjacent packs
  - Platform-specific path/pipe behavior can drift even when Linux is green
  - The source pack flags a sequencing follow-up for adjacent work; basis should be treated as provisional until that cross-pack ordering is revalidated
- **Assumptions**:
  - The attached source pack available for transformation is `world-disabled-diagnostics.zip`
  - The prompt's requested source archive `stabilize-dev-install-helper-discovery.zip` was not attached in this session
  - The source pack's contract docs, slice specs, manual playbook, smoke scripts, and checkpoint plan are trustworthy basis inputs, but cross-queue overlap keeps basis freshness provisional until downstream revalidation
