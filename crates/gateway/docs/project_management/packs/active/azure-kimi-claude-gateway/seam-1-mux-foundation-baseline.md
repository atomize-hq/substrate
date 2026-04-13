---
seam_id: SEAM-1
seam_slug: mux-foundation-baseline
type: platform
status: landed
execution_horizon: future
plan_version: v3
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - the downloaded baseline moves away from `gateway/` or the crate identity changes away from `substrate-gateway` before `C-01` is landed and published
    - the post-rename verification path no longer runs through `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway` and `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`
    - the `5a372fb` validation note changes the provider-extension assumptions that `SEAM-2` inherits
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S4
  status: passed
open_remediations: []
---

# SEAM-1 - Mux Foundation And Baseline Verification

- **Goal / value**: establish the real `claude-code-mux` foundation inside this repo, prove it builds or runs as the gateway baseline, and record exactly what the claimed Kimi-related upstream fix does or does not solve for Azure Kimi behavior.
- **Scope**
  - In:
    - bring the archived `claude-code-mux` codebase into the repo as the implementation foundation
    - establish local baseline build and startup verification before project-identity renames or feature modifications
    - perform the repo-local identity pass needed to disconnect from the old project naming once baseline behavior is stable
    - identify the extension seams for provider normalization, public API surfaces, and internal policy layers
    - record a short verification note for upstream commit `5a372fb`
  - Out:
    - solving Azure hidden-tool normalization itself
    - final Anthropic Messages surface behavior
    - planner/executor policy behavior
    - Substrate-facing deployment or event conformance lock-in
- **Primary interfaces**
  - Inputs:
    - upstream `claude-code-mux` codebase and commit history
    - repo ADR constraints and Substrate memo
    - Azure Kimi behavior summarized in the handoffs
  - Outputs:
    - adopted baseline runtime rooted at `gateway/` with crate name `substrate-gateway`
    - named extension-boundary notes at `docs/foundation/claude-code-mux-adoption.md` and `docs/foundation/claude-code-mux-extension-boundary.md`
    - verification note at `docs/foundation/claude-code-mux-5a372fb-validation.md`
- **Key invariants / rules**:
  - the foundation must preserve Anthropic-first gateway delivery rather than re-centering on Responses-first behavior
  - foundation adoption must not hard-code loopback-only transport or host-only credential assumptions into the core
  - downstream seams must be able to consume normalized events and internal policy boundaries rather than raw provider streams
  - the output of this seam is a usable foundation and truth record, not a speculative architectural rewrite
- **Dependencies**
  - Direct blockers:
    - none beyond the authoritative source material already loaded
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Derived consumers:
    - future OpenAI Responses adapter work
- **Touch surface**:
  - adopted baseline codebase at `gateway/`
  - project-identity rename surfaces including `gateway/Cargo.toml`, binary names, config names, and repo-local gateway labels
  - root-level Cargo entry surfaces needed to run manifest-path build and smoke checks from this repo
  - provider, surface, and policy boundary notes under `docs/foundation/`
  - local verification note for upstream commit `5a372fb`
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify the downloaded archived baseline is established in `gateway/` with `package.name = "substrate-gateway"` in `gateway/Cargo.toml`.
  - Verify the baseline first proves out near upstream behavior before the rename pass changes project identity surfaces.
  - Verify the post-rename repo-root executable checks are `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway` and `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`.
  - Verify `docs/foundation/claude-code-mux-extension-boundary.md` names one provider hook, one client-surface hook, and one internal-policy hook without unresolved placeholders.
  - Verify `docs/foundation/claude-code-mux-5a372fb-validation.md` states what part of the observed Azure failure mode is covered and what remains unresolved.
- **Risks / unknowns**:
  - Risk: `claude-code-mux` may hide the required Azure adaptation points behind abstractions that are too narrow.
  - De-risk plan: inspect and document the concrete provider and streaming extension points during baseline adoption rather than assuming they exist.
  - Risk: upstream `5a372fb` may fix native or non-Azure Kimi behavior while leaving Azure hidden-tool behavior unresolved.
  - De-risk plan: validate against the Azure-specific evidence path named in the handoffs.
- **Rollout / safety**:
  - baseline adoption should stay close to upstream behavior until the verification note confirms the repo is extending the right upstream shape
  - project-identity renames should be targeted and sequenced after baseline stabilization, not mixed into the initial baseline proof
  - no public contract should be frozen by this seam beyond confirming the repo foundation
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is now `future` because the foundation has already landed and left the forward planning window; downstream seams consume it through closeout-backed `THR-01` truth rather than renewed active planning
  - Which threads matter most: `THR-01`
  - What the first seam-local review should focus on: the `gateway/` adoption boundary, the `substrate-gateway` crate identity, the concrete `docs/foundation/` note outputs, and whether the claimed Kimi fix meaningfully changes Azure planning assumptions
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`
  - Threads likely to advance: `THR-01`
  - Review-surface areas likely to shift after landing: `R2`, `R3`
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
