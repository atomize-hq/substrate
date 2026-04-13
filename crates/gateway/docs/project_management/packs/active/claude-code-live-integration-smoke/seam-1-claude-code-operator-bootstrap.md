---
seam_id: SEAM-1
seam_slug: claude-code-operator-bootstrap
type: integration
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-1-closeout.md
    - docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md
  required_threads: []
  stale_triggers:
    - `docs/foundation/azure-foundry-c07-runtime-transport-contract.md` changes the required Azure provider or model-mapping setup surfaces
    - `docs/foundation/azure-foundry-c08-operator-verification-contract.md` changes the minimum smoke/evidence posture in a way that alters bootstrap prerequisites
    - Claude Code attachment rules or gateway startup/config anchors change materially enough that the canonical bootstrap sequence must be rewritten
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

# SEAM-1 - Claude Code Operator Bootstrap

- **Goal / value**: freeze one reproducible operator bootstrap path from Azure prerequisites through gateway configuration, gateway startup, Claude Code attachment, and evidence-hook enablement so later live smoke work starts from current truth instead of ad hoc local knowledge.
- **Scope**
  - In:
    - prerequisite declaration for Azure credentials, deployment identifiers, and the intended `Kimi-K2-Thinking` / `Kimi-K2.5` mappings
    - canonical gateway config and startup sequence, including validation of the chosen config surfaces
    - Claude Code environment and launch path for routing through the gateway
    - operator-visible evidence hooks needed before live smoke begins, such as statusline, tracing, or bounded logs
    - docs/examples/scripts/checklists that make the bootstrap path reproducible
  - Out:
    - proving the live smoke scenarios themselves
    - redesigning provider transport, router policy, or public gateway semantics
    - broad troubleshooting ownership or escalation logic beyond the bootstrap surface
- **Primary interfaces**
  - Inputs:
    - landed `C-03`, `C-04`, `C-05`, `C-07`, and `C-08`
    - current operator anchors in `gateway/README.md`, `gateway/config/*.toml`, `gateway/src/cli/mod.rs`, and `gateway/src/main.rs`
  - Outputs:
    - `C-09` canonical operator bootstrap contract
    - reproducible setup assets for gateway config, startup, Claude Code attachment, and evidence hooks
- **Key invariants / rules**:
  - the bootstrap path must remain capability-oriented and must not expose Azure deployment details or planner/executor roles as public product identity
  - the canonical client path remains Claude Code through `ANTHROPIC_BASE_URL` into the landed `/v1/messages` surface
  - loopback is allowed as a practical first smoke route, but the seam must preserve the replaceable transport/deployment boundary
  - bootstrap guidance must consume `C-07` and `C-08` rather than redefine transport or gateway-only smoke behavior
- **Dependencies**
  - Direct blockers:
    - none; the upstream transport and gateway contracts are already landed and current
  - Transitive blockers:
    - any stale trigger on `C-03`, `C-04`, `C-05`, `C-07`, or `C-08` that changes the operator path or evidence posture
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - future operator onboarding and support work outside this pack
- **Touch surface**:
  - `gateway/README.md`
  - `gateway/config/default.example.toml`
  - `gateway/config/models.example.toml`
  - `gateway/src/cli/mod.rs`
  - `gateway/src/main.rs`
  - any bounded operator scripts, checklist templates, or launch helpers introduced for Claude Code bootstrap
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify a reviewer can state the bootstrap order from Azure prerequisites through Claude Code launch without reading code.
  - Verify the intended config surfaces can express the Azure provider mapping and Claude Code attachment cleanly.
  - Verify the evidence hooks that later smoke work depends on are explicit, bounded, and redaction-safe.
- **Risks / unknowns**:
  - Risk: the current repo guidance may still privilege gateway-only testing over the real Claude Code path.
  - De-risk plan: make the Claude Code attachment and evidence-hook steps explicit seam outputs rather than optional notes.
  - Risk: bootstrap guidance may accidentally imply loopback-only architecture.
  - De-risk plan: preserve the `C-05` boundary language while still naming the simplest first smoke route.
  - Risk: operator evidence hooks may leak too much provider or deployment detail.
  - De-risk plan: define the minimum evidence surface and redaction posture as part of `C-09`.
- **Rollout / safety**:
  - bootstrap assets should prefer redacted or placeholder-safe examples
  - no step should require operators to expose secrets in docs or shared evidence
  - the seam should improve determinism of later smoke work rather than broaden the runtime surface
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because the immediate blocker is not transport anymore; it is the reproducible operator path that later live smoke depends on
  - Which threads matter most: `THR-08`
  - What the first seam-local review should focus on: whether the bootstrap sequence is complete, whether the operator-visible evidence hooks are sufficient, and whether boundary language still hides internal identities correctly
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-09`
  - Threads likely to advance: `THR-08`
  - Review-surface areas likely to shift after landing: `R1`, `R4`
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
