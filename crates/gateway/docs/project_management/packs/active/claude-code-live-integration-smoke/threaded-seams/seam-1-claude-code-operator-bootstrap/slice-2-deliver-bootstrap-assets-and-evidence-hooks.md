---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the bootstrap docs or examples drift from the runtime startup, statusline, or trace behavior before implementation begins
    - the Claude Code attachment or routed-model evidence surfaces change in a way that invalidates the planned operator checklist
    - C-09 lands with different artifact boundaries than the delivery work assumes
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
threads:
  - THR-08
contracts_produced:
  - C-09
contracts_consumed:
  - C-03
  - C-04
  - C-05
  - C-07
  - C-08
open_remediations: []
candidate_subslices: []
---
### S2 - Deliver Bootstrap Assets And Evidence Hooks

- **User/system value**: a real operator can follow one reproducible bootstrap flow, enable the right evidence surfaces, and arrive at a ready-to-smoke Claude Code session without reverse-engineering repo internals.
- **Scope (in/out)**:
  - In: land the bootstrap docs, examples, helper notes, and any bounded runtime-facing clarifications required to make `C-09` executable in practice.
  - Out: proving normal, think, or tool-loop continuation scenarios, or writing the later troubleshooting ownership matrix.
- **Acceptance criteria**:
  - operator-facing surfaces make the bootstrap order explicit from config through `claude` launch
  - README, config examples, statusline installation, and trace-hook guidance align with the frozen `C-09` contract
  - the bootstrap path names what an operator checks before smoke work starts, including startup success and minimum evidence surfaces
  - no bootstrap step requires exposing secrets or teaches loopback convenience as the architecture contract
- **Dependencies**: `S1`, `gateway/README.md`, `gateway/config/default.example.toml`, `gateway/config/models.example.toml`, `gateway/src/cli/mod.rs`, `gateway/src/main.rs`, `gateway/src/server/mod.rs`
- **Verification**:
  - pass condition: an operator can launch the gateway, install or configure the statusline, optionally enable tracing, export the Claude Code env vars, and explain what proof exists before smoke execution
  - pass condition: a reviewer can map every documented step to an existing file or runtime anchor without consulting provider code
- **Rollout/safety**: prefer redacted or placeholder-safe examples and keep the routed model names in internal verification context only.
- **Review surface refs**: `review.md#r1---bootstrap-workflow-that-should-land`, `review.md#r2---evidence-chain-the-bootstrap-seam-must-make-explicit`

#### S2.T1 - Align Operator Docs And Config Examples

- **Outcome**: the operator-facing docs and example config surfaces tell one canonical bootstrap story.
- **Inputs/outputs**: inputs from `C-09`, README, and example TOML files; outputs are aligned operator docs and example snippets
- **Thread/contract refs**: `THR-08`, `C-09`, `C-07`
- **Implementation notes**: reconcile the default config path, Azure provider mapping examples, and Claude Code environment section into one sequence
- **Acceptance criteria**: operators no longer need to infer which file or step comes first
- **Test notes**: compare the final docs against current CLI-generated defaults and example files
- **Risk/rollback notes**: if one surface cannot align without code change, capture that seam-owned delta instead of letting docs drift

Checklist:
- Implement: update docs and examples to follow `C-09`
- Test: compare against runtime defaults and example TOML files
- Validate: verify the path from config to startup to `claude` launch is unambiguous
- Cleanup: remove duplicated or contradictory bootstrap instructions

#### S2.T2 - Surface Bounded Startup And Evidence-Hook Guidance

- **Outcome**: the bootstrap assets tell operators exactly how to confirm startup success and what statusline or trace artifacts count as pre-smoke proof.
- **Inputs/outputs**: inputs from `gateway/src/main.rs`, `gateway/src/server/mod.rs`, `gateway/src/cli/mod.rs`, and README sections; outputs are aligned startup and evidence-hook guidance
- **Thread/contract refs**: `THR-08`, `C-09`, `C-08`
- **Implementation notes**: keep statusline and tracing bounded; do not imply a broad observability program
- **Acceptance criteria**: operators can tell what is required, what is optional, and where each artifact appears on disk
- **Test notes**: validate the documented artifact locations against the runtime code paths
- **Risk/rollback notes**: if an evidence surface leaks too much detail, tighten the redaction posture rather than expanding public identity

Checklist:
- Implement: document startup checks and evidence hooks
- Test: verify artifact names and file paths against runtime anchors
- Validate: ensure the guidance is compatible with `C-08` redaction rules
- Cleanup: remove any implicit “just inspect the code” assumption

#### S2.T3 - Capture Bootstrap Checklist Inputs For Closeout

- **Outcome**: the seam records exactly which landed artifacts and operator-visible proofs `S3` must later publish in closeout.
- **Inputs/outputs**: inputs from `C-09`, the aligned bootstrap assets, and review hotspots; output is a closeout-ready checklist for bootstrap evidence
- **Thread/contract refs**: `THR-08`, `C-09`
- **Implementation notes**: this is still delivery work, not closeout; it should prepare the evidence chain, not realize it
- **Acceptance criteria**: `S3` can enumerate landed bootstrap proof without reopening planning questions
- **Test notes**: confirm each checklist item maps to a real artifact or runtime output named in `S1` or `S2`
- **Risk/rollback notes**: if an expected artifact does not exist yet, keep promotion blocked until the missing proof is resolved

Checklist:
- Implement: define the closeout input checklist
- Test: trace each item back to a planned artifact
- Validate: ensure the checklist covers docs, startup proof, and evidence-hook proof
- Cleanup: keep troubleshooting and live-smoke proof out of this slice
