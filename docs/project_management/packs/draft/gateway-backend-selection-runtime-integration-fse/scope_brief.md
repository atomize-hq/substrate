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
  - Execute ADR-0046 as an implementation pack that turns the current hardcoded `cli:codex` integrated lifecycle into an inventory-backed backend realization path.
  - Finish the minimum remaining `SEAM-1` contract alignment needed to make backend selection and policy inputs deterministic in implementation, then use that handoff to drive runtime realization and later parity proof.
- **Why now**:
  - ADR-0046 already defines the intended implementation seam, and the repo already has canonical selection, policy, protocol, and runtime-parity contracts.
  - The actual gap is execution: `world-agent` and shell still realize only a Codex-specific runtime path, so the pack needs to drive implementation rather than produce more contract/governance scaffolding.
- **Primary user(s) + JTBD**:
  - Substrate maintainers who need one execution spine for moving from a Codex-only integrated lifecycle to an inventory-backed multi-adapter posture.
  - Shell, world-agent, broker, and docs owners who need one agreed handoff from selection/policy truth into runtime realization and then into validation.
  - Reviewers who need the pack to distinguish the small amount of remaining contract-alignment work from the much larger implementation and rollout work.
- **In-scope**:
  - `SEAM-1`: backend selection and policy evaluation for the integrated gateway lifecycle, including selected-backend source of truth, deny-by-default allowlisting, trusted-input boundary, auth-source precedence, and the remaining inventory-root / filename-id alignment needed for implementation surfaces.
  - `SEAM-1`: consumer alignment in broker, shell, config/policy surfaces, and supporting ADR-0046 docs so downstream runtime work consumes one fixed handoff.
  - `SEAM-2`: runtime realization and artifacts for an integrated adapter path, including binding lookup, capability gating, auth handoff validation, config render, managed artifact semantics, launch, readiness, and restart ordering.
  - `SEAM-3`: parity, validation, and rollout proof for Linux, macOS, and Windows, including the `cli:codex` regression floor, explicit unsupported-backend behavior, and a future first-additional-backend baseline.
  - Execution-oriented threading, review surfaces, and remediation tracking that make the active seam executable instead of re-opening already-published contract ownership.
- **Out-of-scope**:
  - Reopening ADR-0040, ADR-0041, ADR-0042, or ADR-0043 ownership.
  - Widening `substrate world gateway status --json`, tuple metadata, or tuple-policy surfaces.
  - Secret-channel redesign beyond the current policy-owned auth precedence and host-to-world delivery boundary.
  - Creating new top-level operator command families.
  - Treating future additional-backend rollout compatibility publication as a prerequisite for the current implementation pack.
- **Success criteria**:
  - `SEAM-1` fixes the remaining selection/policy ambiguity and lands consumer alignment evidence in the repo surfaces that implement it.
  - `SEAM-2` can proceed without inventing upstream truth about selection order, failure classes, trusted inputs, or auth precedence.
  - The pack no longer treats missing adapter binding classification, auth carrier choice, or first-additional-backend rollout baseline as pack-level contract blockers.
  - Review surfaces and threading describe one implementation critical path from selection through runtime realization to later parity proof.
  - The pack stays compatible with ADR-0046 while treating the existing canonical contracts as upstream constraints rather than placeholders for more governance publication.
- **Constraints**:
  - Keep lifecycle state separate from basis freshness.
  - `SEAM-1` is the active execution target and may begin with a narrow `S00` contract-alignment slice before implementation slices.
  - `SEAM-2` is implementation work gated by `SEAM-1`, not another contract-publication seam.
  - `SEAM-3` is a later validation/rollout seam that begins only after runtime realization exists and a named additional backend baseline is chosen.
  - Canonical contract artifacts live under `docs/contracts/`, but this pack does not assume broad new contract publication work.
  - Feature-local ADR-0046 docs remain supporting implementation and verification surfaces.
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
  - `crates/shell/src/execution/config_model.rs`
  - `crates/shell/src/execution/policy_model.rs`
  - `crates/broker/src/policy.rs`
  - `crates/world-agent/src/gateway_runtime.rs`
  - `crates/world-agent/src/service.rs`
  - `crates/agent-api-types/src/lib.rs`
  - `crates/world-agent/tests/gateway_runtime_parity.rs`
  - `crates/shell/tests/world_gateway.rs`
- **Known unknowns / risks**:
  - The first supported non-`cli:codex` integrated backend id is still a later rollout decision, not part of the current execution target.
  - Shared payload surfaces still publish only `cli_codex` integrated auth today, so `SEAM-2` will have to decide how much schema change is needed to support more than one integrated backend.
  - Current runtime launch still exports auth material through env-based child-process injection, which is compatible with current policy rules but not the stronger deferred secret-channel direction.
  - macOS remains guest-managed for runtime lifecycle even though the host control path works; parity proof still has to account for that posture.
- **Assumptions**:
  - Current integrated runtime code is decisive that only `cli:codex` is implemented today:
    - `crates/world-agent/src/gateway_runtime.rs` rejects any default backend other than `cli:codex`
    - `crates/shell/src/builtins/world_gateway.rs` only constructs `integrated_auth.cli_codex`
    - `crates/agent-api-types/src/lib.rs` only publishes a `cli_codex` integrated auth payload variant today
  - The selection and policy contracts already own most of the normative truth this feature needs; the remaining contract work is narrow alignment, not a fresh publication phase.
  - The pre-planning pack remains valid lineage, but this pack is now execution-oriented rather than extraction-oriented.
