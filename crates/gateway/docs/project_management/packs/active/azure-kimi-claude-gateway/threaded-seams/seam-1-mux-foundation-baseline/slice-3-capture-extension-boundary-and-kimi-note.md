---
slice_id: S3
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the final extension-boundary note still leaves `SEAM-2` guessing where Azure normalization attaches
    - the `5a372fb` note does not distinguish upstream Kimi support from Azure Foundry hidden-tool behavior
    - baseline deviations discovered in `S2` change the client-agnostic or Substrate-safe posture of the core
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
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S3 - Capture The Extension Boundary And `5a372fb` Truth Record

- **User/system value**: downstream seams inherit a trustworthy boundary map and a written statement of what the upstream Kimi fix covers, rather than folklore or optimistic assumptions.
- **Scope (in/out)**:
  - In: publish the extension-boundary note at `docs/foundation/claude-code-mux-extension-boundary.md`, capture the `5a372fb` verification result against Azure-specific evidence at `docs/foundation/claude-code-mux-5a372fb-validation.md`, and state what remains unresolved for `SEAM-2`.
  - Out: implementing Azure normalization logic itself or shipping the external Anthropic surface.
- **Acceptance criteria**:
  - the extension-boundary note points to one provider-normalization hook, one client-surface hook, and one internal-policy hook in the adopted baseline
  - the `5a372fb` note states what was checked, what Azure evidence was used, and what remains unresolved
  - any downstream stale triggers implied by the note are named explicitly for seam exit
- **Dependencies**: `S1`, `S2`, `THR-01`, ADR 0002, and the Azure Kimi handoff evidence chain referenced by the pack README
- **Verification**:
  - a reviewer can trace how `SEAM-2` starts from the published boundary without adding new contract language
  - the note contains explicit pass/fail reasoning for `5a372fb`, not just a link or vague confidence statement
  - pass condition: `SEAM-2` can consume `C-01` with a current basis, subject only to future execution-time revalidation
- **Rollout/safety**: keep the note factual; do not overstate Azure coverage just to unblock downstream planning.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`R2`)

#### S3.T1 - Publish The Extension-Boundary Note

- **Outcome**: the adopted baseline has one documented map for provider work, public API adapters, and internal policy layers.
- **Inputs/outputs**: inputs are the implemented baseline from `S2`; output is `docs/foundation/claude-code-mux-extension-boundary.md`, which downstream seams can cite directly.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: boundary language must stay client-agnostic at the core even though Anthropic Messages remains the first external target.

#### S3.T2 - Validate `5a372fb` Against Azure Evidence

- **Outcome**: the repo has a written truth record for whether upstream commit `5a372fb` addresses any part of the Azure Kimi failure mode.
- **Inputs/outputs**: inputs are the upstream commit, ADR 0002, and the handoff evidence chain; output is `docs/foundation/claude-code-mux-5a372fb-validation.md` with explicit covered versus uncovered behavior.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: separate native or generic Kimi behavior from Azure Foundry hidden-tool behavior inside `reasoning_content`.

#### S3.T3 - Name Downstream Basis Implications

- **Outcome**: the seam records exactly what `SEAM-2` through `SEAM-5` should treat as current, provisional, or stale after this seam lands.
- **Inputs/outputs**: inputs are the boundary note and `5a372fb` result; output is a concise stale-trigger list prepared for the seam-exit slice and mirrored in the `docs/foundation/` note set where needed.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: if the note changes the assumed provider hook or core boundary shape, say so explicitly rather than forcing downstream seams to infer it.
