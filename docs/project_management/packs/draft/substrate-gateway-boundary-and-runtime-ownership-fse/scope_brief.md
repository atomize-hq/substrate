---
pack_id: substrate-gateway-boundary-and-runtime-ownership-seam-pack
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/
execution_horizon:
  active_seam: SEAM-3
  next_seam: SEAM-4
---

# Scope Brief - substrate-gateway-boundary-and-runtime-ownership

- **Goal**:
  - Make the Substrate-owned gateway boundary explicit and durable across CLI commands, machine-readable status, policy evaluation, typed runtime ownership, and cross-platform validation.
  - Prevent gateway-local runtime/admin/config surfaces from becoming a second control plane for integrated Substrate operation.
- **Why now**:
  - ADR-0040 supersedes ADR-0023 intent, but the repo still carries stale references, archived command spellings, and unresolved planning ambiguity about which document owns which surface.
  - The pre-planning pack already narrowed the work into a stable authority set; extracting seams now gives downstream planners a governed critical path without forcing slice-level detail into future seams.
- **Primary user(s) + JTBD**:
  - Substrate operators who need one stable command family and one authoritative wiring/status surface for gateway availability and lifecycle actions.
  - Maintainers of `crates/shell`, `crates/world-agent`, shared agent API crates, and operator docs who need one ownership boundary before implementing typed runtime behavior.
  - Documentation and quality-gate maintainers who need deterministic one-owner-per-surface validation across ADR-0040, ADR-0027, ADR-0017, ADR-0028, ADR-0041, and ADR-0042.
- **In-scope**:
  - The operator-facing contract for `substrate world gateway sync`, `status`, and `restart`, including absent-state behavior and exit-code boundaries.
  - The `status --json` envelope and `client_wiring.*` field family as the authoritative machine-readable wiring surface.
  - Gateway-integration policy evaluation over existing ADR-0027 inputs, including fail-closed placement and host-to-world secret-delivery trust boundaries.
  - The typed world-agent lifecycle/status direction selected in pre-planning and the Linux/macOS/Windows parity guarantees it must honor.
  - Manual validation, cross-doc alignment, task/checkpoint wiring, and quality-gate evidence for one-owner-per-surface coverage.
- **Out-of-scope**:
  - New config families, new policy file families, or gateway-local config/admin/persistence as trusted Substrate inputs.
  - Redefining ADR-0027 schema ownership, ADR-0017 event routing, ADR-0028 trace vocabulary, ADR-0041 runtime internals, or ADR-0042 additive identity-tuple metadata outside `client_wiring.*`.
  - Provisioning-script changes, backend warm-flow changes, or a public remote or multi-tenant gateway design.
  - Slice files, threaded seam reviews, or authoritative sub-slices for future seams.
- **Success criteria**:
  - One authoritative operator boundary exists for the command family, ownership split, stable wiring env semantics, and exit-code mapping.
  - `status --json` stays the single Substrate-owned machine-readable wiring authority, with a locked `client_wiring.*` family and explicit absence semantics.
  - Policy evaluation is defined as reuse of existing ADR-0027 inputs with fail-closed no-host-fallback behavior and no trust in gateway-local config/admin/persistence.
  - Typed world-agent lifecycle/status ownership is specified without letting shell probing or platform-specific quirks become user-facing contract.
  - Linux, macOS, and Windows parity requirements and validation evidence are explicit.
  - Manual playbook, docs, plan/task wiring, and quality-gate evidence prove one owner per surface.
- **Constraints**:
  - Keep lifecycle state separate from basis freshness in the extracted seam pack.
  - Keep `status --json` authoritative for machine-readable gateway wiring; human-readable output may abbreviate but must not redefine the machine surface.
  - Stable non-secret env outputs remain `SUBSTRATE_LLM_OPENAI_BASE_URL` and `SUBSTRATE_LLM_ANTHROPIC_BASE_URL`, and they target Substrate-managed endpoints rather than upstream providers.
  - Fail closed when policy requires in-world execution; do not authorize host-level gateway fallback in that state.
  - Keep canonical trace authority with Substrate; gateway-local trace remains implementation-local.
  - Future seams stay at seam-brief depth only.
- **External systems / dependencies**:
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `crates/shell`, `crates/world-agent`, `crates/agent-api-types`, `crates/agent-api-client`
  - `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, `docs/TRACE.md`
- **Known unknowns / risks**:
  - The exact typed world-agent lifecycle/status endpoint surface is still only bounded by pre-planning selection of Option A and needs seam-local review later.
  - The exact additive-field boundary between `client_wiring.*` and ADR-0042 metadata must stay explicit or later docs will drift.
  - Provisioning remains deliberately out of scope for this pack; later runtime work must consume this pack’s contract before changing platform scripts.
  - ADR-0040 and adjacent docs still carry stale `packs/active/...` references that can reintroduce ambiguity during downstream planning.
  - The ADR-0040 lift vector still has an invalid `risk.unknowns_high` field type, so intake-derived sizing remains unavailable until corrected.
- **Assumptions**:
  - The pre-planning pack is authoritative for current scope, ownership, and accepted planning-lane boundaries.
  - The accepted five-slice spine in `pre-planning/workstream_triage.md` is the best proxy for the feature’s critical path, even though this seam pack intentionally stays one level above slice planning.
  - Linux, macOS, and Windows remain the required parity platforms.
  - No hidden requirements require a separate provisioning seam inside this pack.
  - Canonical contract references can be reserved under `docs/contracts/` even though this extraction does not create or populate those descriptive documents.
