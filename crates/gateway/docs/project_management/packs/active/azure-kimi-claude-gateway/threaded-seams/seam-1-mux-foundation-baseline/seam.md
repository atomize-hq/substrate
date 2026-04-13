---
seam_id: SEAM-1
seam_slug: mux-foundation-baseline
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-1-mux-foundation-baseline.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - the chosen `claude-code-mux` import/adoption topology changes away from `gateway/` or the crate name changes away from `substrate-gateway` after `C-01` is frozen
    - the baseline-stabilization step is skipped and identity renames or feature changes begin before the adopted code proves out near baseline behavior
    - the local baseline requires Anthropic-only, loopback-only, or host-credential assumptions in the core request path
    - the post-rename verification path no longer runs through the manifest-path build and smoke commands defined in `S2`
    - the `5a372fb` verification note shows Azure hidden-tool coverage differs from the plan assumptions carried into `SEAM-2`
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-1 - Mux Foundation And Baseline Verification

## Seam Brief (Restated)

- **Goal / value**: establish one real `claude-code-mux`-based gateway baseline in this repository so every downstream seam plans against a concrete runtime, explicit extension boundary, and a written truth record for what upstream commit `5a372fb` does or does not solve for Azure Kimi.
- **Type**: `platform`
- **Scope**
  - **In**:
    - adopt the downloaded archived `claude-code-mux` codebase as the repo's primary starting point under `gateway/`
    - prove the adopted baseline builds and reaches a minimal runnable or smoke-tested baseline state before identity changes or feature work begin
    - rename the Rust crate identity in `gateway/Cargo.toml` to `substrate-gateway` and perform the related repo-local naming pass after baseline stabilization
    - make the baseline manifest-path build/smoke verification path concrete enough to execute from the repo root
    - name the extension boundary that `SEAM-2` through `SEAM-5` will consume through `docs/foundation/claude-code-mux-extension-boundary.md`
    - capture the verification note boundary at `docs/foundation/claude-code-mux-5a372fb-validation.md`
  - **Out**:
    - Azure hidden-tool parsing or normalized event semantics
    - Anthropic Messages gateway behavior beyond confirming the baseline can host it later
    - planner/executor routing policy implementation
    - Substrate-facing boundary lock-in beyond preserving its constraints
- **Touch surface**:
  - adopted gateway baseline path at `gateway/`
  - crate manifest identity at `gateway/Cargo.toml` with `package.name = "substrate-gateway"`
  - other project-identity rename surfaces such as binary names, config names, and user-facing gateway labels
  - post-rename repo-root Cargo invocation surface for `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`
  - post-rename repo-root smoke invocation surface for `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`
  - documentation outputs under `docs/foundation/` for adoption, extension boundaries, and `5a372fb` validation
- **Verification**:
  - `C-01` is concrete enough only if `SEAM-2` can point to one provider-extension boundary note and one baseline verification source of truth before execution begins
  - the plan must leave exactly one baseline build path and one startup or smoke path with named evidence locations rather than generic "baseline works" language
  - the plan must explicitly sequence baseline stabilization before the project-identity rename pass and before downstream feature modifications
  - the `5a372fb` note must explicitly separate upstream Kimi fixes from Azure Foundry hidden-tool behavior that remains unproven
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**: none
  - **Required threads**: `THR-01`
  - **Stale triggers**:
    - the import/adoption topology changes after slice `S1`
    - a buildable baseline still depends on loopback-only or host-only assumptions that violate `IMPORTANT_SUBSTRATE_ALIGNMENT.md`
    - the `5a372fb` verification note forces a different provider-boundary plan for `SEAM-2`
- **Threading constraints**
  - **Upstream blockers**: none; this seam is the producer for `THR-01`
  - **Downstream blocked seams**: `SEAM-2`, `SEAM-3`, `SEAM-4`, `SEAM-5`
  - **Contracts produced**: `C-01`
  - **Contracts consumed**: none

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4`
- **Why this seam needs an explicit exit gate**: downstream seams cannot legally promote on "we imported something mux-like"; they need closeout-backed proof that the foundation, extension boundary, and `5a372fb` truth record actually landed.
- **Expected contracts to publish**: `C-01`
- **Expected threads to publish / advance**: `THR-01` from `defined` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-2` if the published extension boundary differs from the provider-hook assumptions in its seam brief
  - `SEAM-3` if the adopted baseline is more Anthropic-coupled than the pack review surfaces assume
  - `SEAM-4` and `SEAM-5` if the baseline hard-codes loopback or backend-identity assumptions into the core
- **Expected closeout evidence**:
  - landed adopted baseline path at `gateway/`
  - evidence that the baseline was stable before the repo-local identity rename pass
  - evidence that the repo-local identity pass renamed the gateway away from the old project naming without destabilizing the baseline
  - build evidence from `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`
  - smoke evidence from `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`
  - published `docs/foundation/claude-code-mux-extension-boundary.md`
  - written `docs/foundation/claude-code-mux-5a372fb-validation.md` with explicit unresolved Azure behavior

## Slice index

- `S1` -> `slice-1-freeze-foundation-contract.md`
- `S2` -> `slice-2-adopt-baseline-runtime.md`
- `S3` -> `slice-3-capture-extension-boundary-and-kimi-note.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
