---
pack_id: gateway-backend-selection-runtime-integration-seam-pack
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md + docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/pre-planning/
execution_horizon:
  active_seam: SEAM-1
  next_seam: SEAM-2
---

# Scope Brief - gateway-backend-selection-runtime-integration

- **Goal**:
  - Turn ADR-0046 plus the existing pre-planning pack into a governance-ready seam pack that preserves the repo’s current contract boundaries while exposing the critical path from backend selection through runtime realization and finally parity/rollout proof.
  - Replace the current hardcoded `cli:codex` integrated lifecycle assumption with a seam map that can later support inventory-backed, adapter-driven realization without widening adjacent contract surfaces in this extraction.
- **Why now**:
  - ADR-0046 and the pre-planning pack already define the shape of the feature, but the repo still has only Codex-specific integrated runtime code and several unresolved cross-seam decisions that would make downstream planning non-deterministic if they remain implicit.
  - Extracting seams now keeps the planning lane governed while avoiding premature seam-local docs or slice-level decisions.
- **Primary user(s) + JTBD**:
  - Substrate maintainers who need one clear planning control plane for moving from a Codex-only integrated lifecycle to an inventory-backed multi-adapter posture.
  - Shell, world-agent, gateway, and docs owners who need one agreed boundary between selection/policy truth, runtime realization, and parity/rollout proof.
  - Reviewers who need the unresolved items called out as explicit remediations instead of hidden assumptions.
- **In-scope**:
  - Backend selection and policy evaluation for the integrated gateway lifecycle, including backend allowlisting, trusted-input boundaries, auth-source precedence, and backend inventory lookup rules.
  - Runtime realization and artifacts for an integrated adapter path, including binding lookup, capability gating, auth handoff classification, config render, managed artifact semantics, launch, readiness, and restart ordering.
  - Parity, validation, and rollout proof for Linux, macOS, and Windows, including the `cli:codex` regression floor, explicit unsupported-backend behavior, and future multi-backend validation posture.
  - Governance scaffolds, authoritative threading, pack-level review surfaces, and explicit open remediations for unresolved cross-seam decisions.
- **Out-of-scope**:
  - Editing ADR-0040, ADR-0041, or any `docs/contracts/*` gateway doc in this extractor run.
  - ADR-0042 identity-tuple semantics and ADR-0043 tuple-policy keys.
  - Widening `substrate world gateway status --json`, widening tuple metadata, or widening tuple-policy surfaces.
  - Seam-local planning docs under `seam-planning/`, threaded seam `review.md` files, slice files, candidate subslices, or execution units.
- **Success criteria**:
  - Exactly three seams are extracted unless repo evidence forces a different count; that evidence did not force a change here.
  - The active seam, next seam, and future seam each have one clear purpose, explicit touch surfaces, and a credible verification path.
  - The six unresolved pre-planning items are represented as explicit open remediations and risks rather than silently normalized into contract truth.
  - Threading, review surfaces, and governance scaffolds make downstream seam-local planning possible without creating seam-local artifacts in this run.
  - The pack stays compatible with ADR-0046 while publishing seam-owned contract truth into durable canonical refs under `docs/contracts/` and treating feature-local ADR-0046 docs as supporting planning and implementation surfaces.
- **Constraints**:
  - Keep lifecycle state separate from basis freshness.
  - Seam briefs stay `status: proposed`; no seam is decomposed here.
  - `SEAM-1` is the only seam eligible for authoritative downstream planning by default.
  - `SEAM-2` may later receive seam-local review and only provisional deeper planning because unresolved upstream authority questions still affect runtime contract shape.
  - `SEAM-3` remains a future seam brief until upstream contracts and the first additional backend baseline are concrete.
  - Canonical contract publication targets live under `docs/contracts/`.
  - Feature-local ADR-0046 docs remain supporting planning and implementation surfaces, not canonical publication targets.
  - Planning IDs remain confined to planning/governance artifacts.
- **External systems / dependencies**:
  - `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`
  - `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/contracts/substrate-gateway-backend-adapter-selection.md`
  - `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
  - `docs/contracts/substrate-gateway-backend-adapter-schema.md`
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-policy-evaluation.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
  - `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/world-agent/src/gateway_runtime.rs`
  - `crates/world-agent/src/service.rs`
  - `crates/agent-api-types/src/lib.rs`
  - `crates/world-agent/tests/gateway_runtime_parity.rs`
  - `crates/shell/tests/world_gateway.rs`
- **Known unknowns / risks**:
  - The first supported non-`cli:codex` integrated backend id is not pinned by current repo evidence.
  - Missing integrated adapter binding classification is not yet fixed as contract truth.
  - Missing auth handoff material classification is not yet fixed as contract truth.
  - The integrated auth handoff delivery rule into the runtime is not yet fixed as `env-only`, `file-only`, or one fixed mixed model with explicit precedence.
  - Auth precedence between env material and host credential files is only evidenced by the current Codex-specific implementation path, not yet by a general integrated contract.
  - Backend inventory roots and filename rules are referenced as one-file-per-backend posture, but the exact discoverability roots and filename invariants for this feature are not published as integrated runtime contract truth.
- **Assumptions**:
  - The pre-planning pack is the authoritative planning basis for ADR-0046, but its `GBSRI-*` ids and seam-planning paths are lineage rather than required outputs.
  - Current integrated runtime code is decisive that only `cli:codex` is implemented today:
    - `crates/world-agent/src/gateway_runtime.rs` rejects any default backend other than `cli:codex`
    - `crates/shell/src/builtins/world_gateway.rs` only constructs `integrated_auth.cli_codex`
    - `crates/agent-api-types/src/lib.rs` only publishes a `cli_codex` integrated auth payload variant today
  - Those Codex-specific surfaces are evidence for present behavior, but not sufficient by themselves to close the six unresolved planning items for multi-backend integrated lifecycle work.
