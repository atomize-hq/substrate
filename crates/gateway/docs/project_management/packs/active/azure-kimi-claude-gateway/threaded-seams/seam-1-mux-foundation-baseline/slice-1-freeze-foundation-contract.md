---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the repo-local import/adoption topology changes after this slice freezes `C-01`
    - the chosen extension boundary still requires downstream seams to infer provider or API adapter hooks
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze The Foundation Contract

- **User/system value**: downstream seams stop planning against "import `claude-code-mux` somehow" and instead inherit one explicit foundation contract with a named extension boundary.
- **Scope (in/out)**:
  - In: freeze the repo-local adoption topology at `gateway/`, require baseline stabilization before identity renames, rename the crate to `substrate-gateway`, name the baseline build/start entrypoint, and define where provider normalization, public API adapters, and internal policy layers attach in the `docs/foundation/` note set.
  - Out: importing code, parsing Azure hidden-tool markers, or delivering the public Anthropic surface itself.
- **Acceptance criteria**:
  - `C-01` names one repo-local adoption topology for the `claude-code-mux` baseline: adopt the archived source as the primary codebase under `gateway/`.
  - `C-01` names one execution order for that baseline: baseline stabilization first, identity renames second, feature modifications third.
  - `C-01` names one crate identity for that baseline: `package.name = "substrate-gateway"` in `gateway/Cargo.toml`.
  - `C-01` names one extension-boundary map at `docs/foundation/claude-code-mux-extension-boundary.md` covering provider normalization, external API surfaces, and internal routing policy.
  - `C-01` includes a verification checklist with concrete pass/fail conditions for `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`, `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`, and `docs/foundation/claude-code-mux-5a372fb-validation.md`.
- **Dependencies**: `scope_brief.md`, `threading.md` (`C-01`, `THR-01`), ADR 0001, and `IMPORTANT_SUBSTRATE_ALIGNMENT.md`
- **Verification**:
  - A reviewer can identify one adoption topology, one extension-boundary map, and one verification-note location without unresolved placeholders.
  - Edge cases are named up front: why the seam uses a downloaded archived baseline instead of an upstream-sync plan, which naming surfaces must change during the identity pass, and baseline startup paths that require loopback-only topology.
  - Pass condition: `SEAM-2` can cite one foundation contract source of truth without inventing any new boundary language.
- **Rollout/safety**: keep the adoption topology reversible until the local baseline proves out; do not freeze external client or deployment behavior in this slice.
- **Review surface refs**: `../../review_surfaces.md` (`R2`, `R3`) and `review.md` (`R1`, `R2`)

#### S1.T1 - Define The Repo-Local Adoption Topology

- **Outcome**: one documented import/adoption shape for `claude-code-mux` exists and is referenced by future implementation work.
- **Inputs/outputs**: inputs are ADR 0001 and the current docs-only repo layout; output is the chosen baseline location `gateway/`, the renamed crate identity `substrate-gateway`, and the repo-root manifest-path build/smoke expectations recorded in `docs/foundation/claude-code-mux-adoption.md`.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: because the upstream repo is archived, this seam adopts the code as the primary starting codebase, proves it out close to baseline behavior, and only then performs the repo-local identity pass.

#### S1.T2 - Define The Extension-Boundary Map

- **Outcome**: provider, API-surface, and internal-policy extension points are explicit enough for downstream seams to target them.
- **Inputs/outputs**: inputs are `review_surfaces.md`, ADR 0002, ADR 0004, and ADR 0005 through ADR 0007; output is a single boundary map recorded in `docs/foundation/claude-code-mux-extension-boundary.md`.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: the boundary must keep Azure normalization and planner/executor policy internal while preserving a future Anthropic-first outer surface and a later Responses adapter as a thin outer seam.

#### S1.T3 - Freeze The Verification Checklist

- **Outcome**: the contract-definition slice includes the narrow verification plan needed for later pre-exec contract approval.
- **Inputs/outputs**: inputs are ADR deliverable boundaries and current pack risks; output is a checklist naming the manifest-path build/smoke proof for `substrate-gateway`, `5a372fb` evidence at `docs/foundation/claude-code-mux-5a372fb-validation.md`, edge cases, and pass/fail conditions.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: the checklist is about making the producer seam executable; it must distinguish baseline proof, identity-renaming proof, and later feature work rather than blending them into one step.
