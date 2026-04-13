---
slice_id: S1
seam_id: SEAM-5
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-03` changes public session continuation or thin-adapter assumptions in a way that alters external boundary expectations"
    - "`C-04` changes internal policy or state-handoff assumptions in a way that alters public identity rules"
    - public config or docs start exposing separate planner, executor, or provider identities
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
  - THR-04
  - THR-05
contracts_produced:
  - C-05
contracts_consumed:
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze Public Identity And Deployment Boundary

- **User/system value**: downstream integration can treat the gateway as one stable capability instead of a bundle of internal roles or a localhost-only demo path.
- **Scope (in/out)**:
  - In: define the owned `C-05` external identity and deployment-boundary contract, including one logical backend identity, replaceable auth/transport, and the non-negotiable public naming rules.
  - Out: downstream event schema details, provider parsing, and any direct Substrate implementation.
- **Acceptance criteria**:
  - one canonical `C-05` contract artifact path is named for landing: `docs/foundation/substrate-boundary-c05-contract.md`
  - the contract states that public capability naming stays singular and does not expose planner, executor, or provider identities
  - the contract states that loopback-local transport is a convenience, not the architecture contract
  - the contract gives downstream work enough detail to implement without reverse-engineering runtime identity choices
  - the contract names which existing runtime/config surfaces already express the external identity and which ones must remain replaceable outer-layer concerns
- **Landed outputs**:
  - contract note target: `docs/foundation/substrate-boundary-c05-contract.md`
  - public identity and runtime anchors: `gateway/src/main.rs`, `gateway/src/cli/mod.rs`, `gateway/src/server/mod.rs`, `gateway/README.md`, and `gateway/config/default.example.toml`
  - constraint sources carried into closeout: `docs/foundation/claude-code-mux-extension-boundary.md`, ADR 0005, and ADR 0006
- **Dependencies**: `../../threading.md`, `../../governance/seam-4-closeout.md`, `docs/foundation/claude-code-mux-extension-boundary.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/foundation/planner-executor-c04-policy-contract.md`, `docs/adr/0005-present-a-single-backend-identity-to-substrate.md`, `docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md`, `gateway/src/main.rs`, `gateway/src/cli/mod.rs`, `gateway/src/server/mod.rs`, `gateway/README.md`, and `gateway/config/default.example.toml`
- **Verification**:
  - a reviewer can explain the public identity and deployment boundary without reading runtime code
  - public docs and config examples stay capability-oriented
  - no separate planner/executor/provider backend ids appear in the public surface
  - pass condition: execution can land `C-05` by editing the named contract/doc/config surfaces without inventing backend identity or deployment rules during implementation
  - failure conditions are explicit: public docs/config expose planner/executor/provider identities, localhost becomes the only supported boundary, or host-only credential assumptions move into the core request path
- **Rollout/safety**: keep the boundary replaceable; do not let convenience defaults hard-code localhost or internal role names as the contract.
- **Review surface refs**: `../../review_surfaces.md` (`R2`, `R3`) and `review.md`

#### S1.T1 - Freeze The External Identity Contract

- **Outcome**: one owned `C-05` artifact names the stable external backend identity, the allowed public capability labels, and the identities that must remain internal-only.
- **Inputs/outputs**: inputs are ADR 0005, the landed `C-03` surface note, the landed `C-04` policy note, and the existing repo-local identity anchors; output is `docs/foundation/substrate-boundary-c05-contract.md` plus any seam-local cross-links needed for closeout.
- **Thread/contract refs**: `THR-05`, `C-05`, `C-03`, `C-04`
- **Implementation notes**: external naming must stay singular and capability-oriented even if runtime policy continues to route across multiple internal roles.

#### S1.T2 - Trace Deployment And Auth Boundary Anchors

- **Outcome**: the seam names the exact code/config/doc surfaces that already express the boundary and the places that must remain replaceable beyond localhost.
- **Inputs/outputs**: inputs are ADR 0006 and the current CLI, server, runtime-path, and config-example anchors; output is a landing checklist for `gateway/src/main.rs`, `gateway/src/cli/mod.rs`, `gateway/src/server/mod.rs`, `gateway/README.md`, and `gateway/config/default.example.toml`.
- **Thread/contract refs**: `THR-05`, `C-05`
- **Implementation notes**: local loopback can remain a developer convenience, but the contract must keep transport, auth, and secret delivery outside the core engine assumptions.

#### S1.T3 - Add Public-Boundary Drift Guards

- **Outcome**: closeout and later promotion can fail on concrete identity drift instead of relying on informal review.
- **Inputs/outputs**: inputs are the planned `C-05` note and the public-facing docs/config surfaces; output is explicit drift-guard language for closeout and future revalidation.
- **Thread/contract refs**: `THR-05`, `C-05`, `C-04`
- **Implementation notes**: guards should flag planner/executor/provider naming leakage, localhost-only language, and host-only credential assumptions that escape the outer deployment layer.
