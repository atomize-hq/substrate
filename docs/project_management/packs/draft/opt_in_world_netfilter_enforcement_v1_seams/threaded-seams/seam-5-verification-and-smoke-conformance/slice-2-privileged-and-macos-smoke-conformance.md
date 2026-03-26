---
slice_id: S2
seam_id: SEAM-5
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to WORLD_NETFILTER_ENABLE propagation or Lima systemd env setup"
    - "Any change to fail-closed runtime wording, deny-all DNS semantics, or attach-or-fail behavior"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
  - THR-04
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-04
  - C-05
  - C-06
  - C-07
open_remediations: []
candidate_subslices: []
---
### S2 - Privileged and macOS smoke conformance

- **User/system value**: maintainers get one trusted path to prove the fail-closed runtime is real on Linux and one repeatable operator smoke workflow for macOS Lima.
- **Scope (in/out)**:
  - In:
    - refresh the privileged Linux verification surface and document when to run it
    - exercise macOS Lima warm/smoke flows against allow-all versus deny-all or restrictive policy postures
    - publish an operator-facing manual verification playbook for the netfilter feature
  - Out:
    - new runtime enforcement behavior or new doctor fields
- **Acceptance criteria**:
  - the privileged Linux verification surface is explicit about prerequisites, execution command, and expected pass/fail evidence for requested isolation.
  - the macOS Lima smoke path validates guest env propagation plus at least one restrictive posture and one allow-all posture.
  - operator-facing documentation points to the same three-way gate contract already published in `docs/reference/config/world.md`.
- **Dependencies**:
  - landed closeouts `../../governance/seam-2-closeout.md`, `../../governance/seam-3-closeout.md`, and `../../governance/seam-4-closeout.md`
- **Verification**:
  - `cargo test -p world -- --ignored`
  - targeted `scripts/mac/lima-warm.sh` run with `SUBSTRATE_WORLD_NETFILTER_ENABLE=1`
  - targeted `scripts/mac/smoke.sh` run or an equivalent scripted smoke sequence that exercises the netfilter postures
- **Rollout/safety**:
  - the Linux validation remains opt-in/privileged and the macOS smoke remains operator-controlled
- **Review surface refs**:
  - `review.md`

#### S2.T1 - Make privileged Linux verification evidence explicit

- **Outcome**: the seam closeout can cite one concrete privileged verification surface instead of relying on source-only ignored tests.
- **Files**:
  - `crates/world/src/netfilter.rs`
  - `docs/manual_verification/netfilter_enforcement.md`
- **Thread/contract refs**:
  - `THR-04`
  - `C-02`
- **Acceptance criteria**:
  - the ignored Linux test path and its prerequisites are documented in one operator-facing location
  - closeout evidence can cite an actual command and observed result for requested isolation behavior
- **Test notes**:
  - keep the privileged surface bounded to existing Linux-only nftables coverage unless a new focused test is clearly required

Checklist:
- Implement: privileged verification doc and any focused test harness adjustments
- Test: privileged Linux run or equivalent captured evidence
- Validate: documented expectations match the fail-closed runtime taxonomy

#### S2.T2 - Publish the macOS Lima smoke flow for netfilter conformance

- **Outcome**: macOS operators can verify that guest env propagation and doctor/smoke results match the published contract.
- **Files**:
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/smoke.sh`
  - `docs/cross-platform/mac_world_setup.md`
  - `docs/manual_verification/netfilter_enforcement.md`
- **Thread/contract refs**:
  - `THR-03`
  - `THR-05`
  - `C-04`
  - `C-07`
- **Acceptance criteria**:
  - the smoke flow makes `SUBSTRATE_WORLD_NETFILTER_ENABLE` / `WORLD_NETFILTER_ENABLE` propagation explicit
  - the documented steps distinguish allow-all from deny-all or restrictive allowlist behavior
  - the smoke flow tells operators which doctor/shim fields to inspect when a requested run fails
- **Test notes**:
  - prefer extending the existing Lima warm/smoke scripts instead of inventing a second macOS smoke entrypoint

Checklist:
- Implement: macOS smoke/doc updates
- Test: warm + smoke sequence with explicit posture checks
- Validate: operator-facing steps match the published three-way gate contract
