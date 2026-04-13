---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the imported baseline cannot build or start from the repo-root workflow defined by `S1`
    - the adopted runtime requires loopback-only or host-only assumptions in the core request path
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
### S2 - Adopt The Baseline Runtime

- **User/system value**: the repository gains a real gateway baseline that downstream seams can build on instead of only documentation and ADR intent.
- **Scope (in/out)**:
  - In: materialize the downloaded import/adoption topology in `gateway/`, prove it works near baseline behavior, then execute the repo-local identity pass including the `substrate-gateway` crate rename, and capture the first baseline verification evidence.
  - Out: Azure-specific parser work, public Anthropic behavior changes beyond baseline parity, and planner/executor policy implementation.
- **Acceptance criteria**:
  - the adopted `claude-code-mux` baseline is present at `gateway/`
  - the baseline is verified close to its adopted starting behavior before identity renames or feature modifications begin
  - `gateway/Cargo.toml` names the crate `substrate-gateway` after the identity pass
  - the post-rename repo-root verification path is `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`
  - the post-rename repo-root smoke path is `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`
  - any deviations from upstream baseline behavior that matter to downstream seams are captured immediately rather than deferred in `docs/foundation/claude-code-mux-adoption.md`
- **Dependencies**: `S1`, `THR-01`, and the verification checklist frozen in `C-01`
- **Verification**:
  - a reviewer can run or inspect one repo-root build path and one startup or smoke path without guessing where the baseline lives
  - failure conditions are explicit: missing workspace wiring, startup path coupled to host-only credentials, or runtime assumptions that prevent future Azure/provider extension work
  - pass condition: the repo now contains a buildable or at least startable baseline that can host downstream seam work
- **Rollout/safety**: keep imported baseline changes isolated so they remain replaceable if the verification note later proves the upstream shape is wrong for Azure.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`R1`)

#### S2.T1 - Materialize The Adopted Foundation

- **Outcome**: the chosen `claude-code-mux` adoption topology is present in the repository and linked into repo-root workflows.
- **Inputs/outputs**: inputs are the topology frozen in `S1`; output is the downloaded code tree at `gateway/`, the renamed crate identity `substrate-gateway`, and the repo-root invocation notes needed to run manifest-path build/smoke checks.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: preserve the adopted codebase close to baseline behavior during the first proof step; do not fold provider-specific work into this task before the identity pass is complete.

#### S2.T2 - Wire Build And Startup Proof

- **Outcome**: one local build path and one startup or smoke path exist for the adopted baseline.
- **Inputs/outputs**: inputs are the imported baseline and verification checklist; output is the documented commands `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway` and `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help` plus the first recorded evidence or blockers.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: if the baseline cannot fully run yet, capture the exact blocker rather than masking it behind generic "integration pending" language; do not start renaming or feature work until that blocker posture is explicit.

#### S2.T3 - Record Baseline Deviations Early

- **Outcome**: any gap between upstream expectations and local baseline reality is captured before downstream seams inherit stale assumptions.
- **Inputs/outputs**: inputs are build/start results and substrate constraints; output is a short deviation record tied back to `C-01` in `docs/foundation/claude-code-mux-adoption.md`.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**: deviations that would change downstream planning should become stale-trigger candidates for seam exit, not hidden TODOs; the same note should distinguish baseline-stabilization deviations from rename-pass decisions.
