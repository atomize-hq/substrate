---
pack_id: substrate-gateway-backend-adapter-contract
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md + docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/
execution_horizon:
  active_seam: null
  next_seam: null
---

# Scope Brief - Substrate gateway backend adapter contract

- **Goal**:
  - Turn ADR-0041 and its pre-planning packet into a governance-ready seam pack that can drive downstream seam-local planning for the gateway backend adapter contract.
- **Why now**:
  - ADR-0041 supersedes the architectural intent of ADR-0024, ADR-0040 fixed the boundary split, and the remaining work now needs explicit seam ownership before implementation or deeper planning touches runtime-adjacent surfaces.
- **Primary user(s) + JTBD**:
  - Substrate maintainers and gateway adapter implementers who need one stable backend-id contract, one fail-closed selection boundary, one adapter protocol/schema boundary, and one deterministic parity/validation proof path.
  - Operators are indirect beneficiaries because they rely on the same stable `<kind>:<name>` backend identity and fail-closed behavior without learning gateway-internal session mechanics.
- **In-scope**:
  - seam extraction for the adapter-selection boundary, adapter protocol/schema boundary, and parity/validation boundary
  - authoritative threading across backend-id selection, adapter-visible status publication, protocol/schema handoff, and parity proof
  - pack-level review surfaces for the product/service flow that should land
  - governance scaffolds and structured remediations for unresolved planning blockers
- **Out-of-scope**:
  - implementation slices, task graphs, or authoritative sub-slices
  - changing ADR-0027 config keys, file-family precedence, or allowlist storage
  - redefining ADR-0017 event envelopes or ADR-0028 trace vocabulary
  - introducing a new public remote or multi-tenant gateway
  - widening the operator CLI command family beyond `status`, `sync`, and `restart`
- **Success criteria**:
  - the forward planning window is explicit, including `null` active/next seams once all pack seams are closed
  - every meaningful cross-seam dependency is carried by a named thread
  - the pack stays above seam-local decomposition while preserving enough contract detail for downstream planning
  - unresolved questions are captured as governance remediations instead of hidden inside prose
- **Constraints**:
  - stable backend ids remain `<kind>:<name>`
  - `llm.allowed_backends` remains deny-by-default and gates selection before adapter dispatch
  - `cli:codex` remains the first required backend adapter
  - gateway-local session strategy, prompt shaping, provider quirks, and persistence do not become Substrate policy inputs
  - planning IDs remain inside planning and governance artifacts only
- **External systems / dependencies**:
  - `substrate-gateway`
  - ADR-0027, ADR-0040, ADR-0017, ADR-0028
  - `docs/contracts/gateway/operator-contract.md`
  - `docs/contracts/gateway/status-schema.md`
  - `docs/contracts/gateway/policy-evaluation.md`
  - `docs/contracts/gateway/runtime-parity.md`
  - Unified Agent API evidence from `unified-agent-api`
  - gateway boundary evidence from `kimi-claude-adapter`
- **Known unknowns / risks**:
  - ADR-0040 alignment remains evidence-only and future runtime-boundary drift would still require owner-led revalidation
  - future changes to platform guarantees, ADR-0024 supersession posture, ADR-0040 alignment posture, or protocol/schema wording will stale the closeout-backed parity and validation evidence
- **Assumptions**:
  - the existing feature directory `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/` is the correct pack root to reuse
  - `SEAM-1` has closed with `THR-01` published and `SEAM-2` has now closed with `THR-02` published
  - `SEAM-3` has now closed with `THR-01` and `THR-02` revalidated, so no active seam remains in the forward planning window
  - no additional seam remains in the forward planning window after the `SEAM-3` closeout
  - future durable contract publications for the adapter contract may land under descriptive `docs/contracts/` paths, but this pack still keeps those canonical docs outside the planning control plane
