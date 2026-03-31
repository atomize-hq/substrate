---
seam_id: SEAM-3
seam_slug: cross-platform-proof-drift-guards
type: conformance
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - Linux or macOS behavior scope changes
    - Windows compile-parity-only posture changes
    - smoke command set or manual playbook cases change
    - checkpoint boundary or checkpoint evidence requirements change
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
  planned_location: S3
  status: passed
open_remediations: []
---

# SEAM-3 - Cross-platform proof + drift guards

This seam is landed. Its authoritative planning and exit-gate record live in `threaded-seams/seam-3-cross-platform-proof-drift-guards/` and `governance/seam-3-closeout.md`.

- **Goal / value**: Keep the landed feature contract provable and non-drifting across Linux and macOS behavior validation, Windows compile parity, and checkpoint evidence after the first two seams land.
- **Scope**
  - In:
    - manual playbook coverage for bundle staging, helper discovery, and managed cleanup
    - Linux and macOS smoke wrappers as authoritative behavior evidence
    - Windows smoke as compile-parity evidence only
    - checkpoint-boundary proof after the cleanup seam lands
  - Out:
    - new runtime behavior not already owned by `SEAM-1` or `SEAM-2`
    - enabling supported `substrate world enable` behavior on Windows
    - broadening the staged bundle beyond the upstream seams
- **Primary interfaces**
  - Inputs:
    - published helper-order, bundle-layout, and managed-marker contracts from `SEAM-1`
    - published cleanup and refusal contract from `SEAM-2`
    - `manual_testing_playbook.md`
    - `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`
    - checkpoint-boundary intent and quality-gate evidence surfaces
  - Outputs:
    - manual and smoke evidence aligned to the landed contracts
    - stable platform-parity posture for Linux, macOS, and Windows compile parity
    - recorded stale triggers when upstream or platform assumptions drift
- **Key invariants / rules**:
  - Linux and macOS behavior smoke remain required before the checkpoint closes.
  - Windows remains compile parity only in this pack.
  - The checkpoint boundary stays after the cleanup seam, not halfway through contract publication.
  - Validation claims must not overstate macOS provisioning parity beyond the actual staged release-root assets.
- **Dependencies**
  - Direct blockers:
    - `THR-01`, `THR-02`, and `THR-03` must have closeout-backed truth
  - Transitive blockers:
    - checkpoint wiring drift
    - platform-scope drift in ADR or contract surfaces
  - Direct consumers:
    - pack closeout and any future staging follow-on that depends on stable evidence
  - Derived consumers:
    - future Windows enablement or broader macOS parity work that will need a trusted evidence baseline
- **Touch surface**:
  - `manual_testing_playbook.md`
  - `platform-parity-spec.md`
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
  - checkpoint and closeout evidence surfaces
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Expected proof points:
    - Linux smoke proves prefix bundle staging, helper-order behavior, and cleanup assertions
    - macOS smoke proves helper discovery and cleanup within the narrowed macOS scope
    - Windows smoke stays compile parity only and clearly skips behavior claims
    - manual playbook cases and checkpoint wording match the landed upstream contracts
- **Risks / unknowns**:
  - Risk: evidence can drift from actual landed behavior when upstream seams adjust path lists, messages, or refusal semantics.
  - De-risk plan: require seam-local revalidation through `THR-01`, `THR-02`, and `THR-03` before this seam promotes to `exec-ready`.
  - Risk: Windows or macOS evidence can overclaim support if unsupported surfaces are described loosely.
  - De-risk plan: keep platform claims explicit and narrow in smoke output, parity docs, and closeout.
- **Rollout / safety**:
  - Use the checkpoint only after the full feature contract is present.
  - Preserve deterministic skip behavior on non-target hosts.
  - Record stale triggers rather than silently carrying evidence forward when upstream reality changes.
- **Downstream decomposition context**:
  - This seam was the terminal active seam after `SEAM-2` closed with a passed seam-exit gate and published `THR-03`.
  - Which threads matter most: `THR-01`, `THR-02`, and `THR-03`.
  - The landed seam-local work focused on evidence-to-contract alignment, platform claim boundaries, checkpoint proof, and stale-trigger handling.
- **Landed seam-exit posture**:
  - `REM-002` is resolved by the landed claim-boundary and smoke-surface work recorded in `governance/seam-3-closeout.md`.
  - The seam owned evidence alignment only and did not reopen upstream helper-bundle or cleanup behavior.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: minimal or none beyond finalized evidence mapping.
  - Threads likely to advance: `THR-01`, `THR-02`, and `THR-03` toward `revalidated` or `closed`.
  - Review-surface areas likely to shift after landing: smoke assertions, platform wording, and checkpoint evidence summaries.
  - Downstream seams most likely to require revalidation: any future macOS parity expansion, Windows enablement, or helper-bundle surface growth.
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
