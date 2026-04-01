---
pack_id: make-doctor-health-output-explain-why-seam-pack-v2-3
pack_version: v1
pack_status: extracted
source_ref: make-doctor-health-output-explain-why.zip :: docs/project_management/packs/draft/make-doctor-health-output-explain-why/
execution_horizon:
  active_seam: SEAM-1
  next_seam: SEAM-2
---

# Scope Brief - make-doctor-health-output-explain-why

- **Goal**: Make doctor and health output explain why world isolation is disabled without changing enablement precedence, exit semantics, or the existing disabled-status model.
- **Why now**: Current doctor and health output can misattribute disablement as `--no-world` even when the effective winner came from workspace config, global config, or the override environment variable. The source pack also identified overlapping queued work on disabled-status UX, JSON envelope shape, and provisioning guidance, so the attribution contract needs to be locked before more health/JSON surface changes land.
- **Primary user(s) + JTBD**: Operators, support engineers, and automation/CI tooling need a deterministic explanation of which layer disabled world isolation so they can change the correct layer immediately and avoid scraping ambiguous human text.
- **In-scope**:
  - `substrate host doctor`, `substrate world doctor`, and `substrate health` disable-attribution behavior
  - exact text message bodies for CLI, env, workspace, global, default, and `source unknown` cases
  - shared provenance-backed disable-attribution mapping that follows the same effective winner used for `world.enabled`
  - additive top-level JSON fields for doctor and health outputs when world is disabled
  - health text parity with doctor text, including nested doctor/shim paths that must preserve CLI flag attribution
  - redaction invariants for env tokens and tokenized config paths
  - Linux, macOS, and Windows parity where the corresponding doctor/health surface exists
  - manual validation, smoke parity expectations, and checkpoint-aware sequencing context
- **Out-of-scope**:
  - any change to `world.enabled` precedence, enable/disable semantics, or exit codes
  - world-agent readiness, provisioning behavior, or replay warnings beyond doctor/health attribution
  - new environment variables, new host<->agent protocol surfaces, new policy broker surfaces, or new telemetry schema fields
  - adding Windows host-doctor support
  - seam-local slices or `threaded-seams/.../review.md` artifacts in this extractor pack
- **Success criteria**:
  - doctor text names the effective disable source with the exact contract message bodies and emits nothing when world is enabled
  - attribution always matches the effective winner for `world.enabled`; if safe winner mapping cannot be proven, output degrades to `source unknown` rather than guessing
  - doctor and health JSON emit additive top-level `world_disable_reason` and `world_disable_source` only when world is disabled and omit both fields when it is enabled
  - health text uses the same attribution message bodies as doctor text and preserves CLI flag attribution through nested call paths
  - JSON and text never leak raw env values or absolute host paths; only tokenized display paths and the fixed safe env token appear
  - the pack preserves the source plan's critical path: stabilize doctor text attribution first, then expand to JSON + health parity
- **Constraints**:
  - exact contract strings and redaction tokens must remain stable once published
  - precedence order remains `CLI -> workspace patch -> env override when no workspace exists -> global patch -> default`
  - JSON changes are additive only; no rename or removal of existing fields
  - pack-level review surfaces are orientation only; active and next seams still require seam-local `review.md` later
  - extractor posture keeps the work one level above seam-local decomposition and reserves seam-exit intent without creating slices
- **External systems / dependencies**:
  - effective-config resolution and explain provenance for `world.enabled`
  - doctor platform entrypoints and platform-specific renderers for Linux, macOS, and Windows
  - health and shim-doctor reporting surfaces that must reuse the same attribution semantics
  - queued or overlapping work on disabled-status UX, JSON envelope shape, and provisioning health output
  - ADR background on disable attribution, disabled-status classification, replay-warning reuse, and policy/config terminology migration
- **Known unknowns / risks**:
  - misattribution is worse than omission, so provenance gaps must fall back to `source unknown`
  - nested health or shim paths could drift from the doctor contract if they do not carry CLI provenance cleanly end-to-end
  - JSON envelope or health-summary work in adjacent queues could collide with top-level field placement or omit/emit semantics
  - tokenized path or env redaction could regress if platform renderers bypass the shared helper
  - queue ordering matters: the feature assumes disabled-status semantics land first and JSON envelope work preserves these additive fields
- **Assumptions**:
  - the source pack's quality gate, session log, contract, schema spec, and accepted `DHO0 -> DHO1` order are current enough to seed extraction
  - `resolve_effective_config_with_explain(..., true)` or equivalent provenance-backed resolution remains available to downstream seam planning
  - the smallest coherent seam map is two seams, and cross-platform evidence plus queued-pack compatibility are better represented as threads and closeout concerns than as standalone extra seams
