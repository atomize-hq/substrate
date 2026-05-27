---
seam_id: SEAM-2
seam_slug: live-session-smoke-verification
type: conformance
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/claude-code-live-integration-smoke/governance/seam-1-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
  required_threads:
    - THR-08
  stale_triggers:
    - `C-09` changes the canonical bootstrap path, evidence hooks, or Claude Code attachment rules
    - the landed `/v1/messages` or route-selection behavior changes the observable normal, think, or continuation path
    - Claude Code session behavior reveals new client-side constraints that invalidate the planned smoke scenarios or evidence expectations
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

# SEAM-2 - Live Session Smoke Verification

- **Goal / value**: turn the canonical bootstrap path into concrete Claude Code proof for normal execution, think/planner, and tool-loop continuation behavior so the live path is verified through the real client operators use, not only through gateway-local smoke calls.
- **Scope**
  - In:
    - normal Claude Code live session smoke through the gateway
    - think/planner smoke that proves the intended `Kimi-K2-Thinking` routing path
    - tool-loop continuation or follow-up smoke that proves the intended handoff behavior after tool results
    - redacted evidence expectations for transcripts, statusline, tracing, and route-visible artifacts
    - operator-visible pass/fail criteria tied back to the landed gateway and transport contracts
  - Out:
    - redefining bootstrap assets owned by `SEAM-1`
    - redefining the public `/v1/messages` contract or internal route policy
    - broad support ownership or escalation guidance beyond what is needed to interpret the smoke evidence
- **Primary interfaces**
  - Inputs:
    - published `C-09` bootstrap contract from `SEAM-1`
    - landed `C-03`, `C-04`, `C-07`, and `C-08`
  - Outputs:
    - `C-10` live Claude Code smoke verification contract
    - redacted proof surfaces for normal, think, and continuation flows
- **Key invariants / rules**:
  - the smoke path must use real Claude Code sessions rather than provider-only or curl-only substitutes
  - the proof must remain capability-oriented even when route evidence mentions internal labels like `Kimi-K2-Thinking` and `Kimi-K2.5`
  - tool-loop continuation proof must validate the intended handoff behavior without turning internal continuation hints into public contract
  - evidence must be minimal, redacted, and sufficient for later troubleshooting work
- **Dependencies**
  - Direct blockers:
    - none; `SEAM-1` has now published `C-09`
  - Transitive blockers:
    - any stale trigger on `C-03`, `C-04`, `C-07`, or `C-08` that changes observable live behavior
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - future operator runbooks and regression-style live verification work
- **Touch surface**:
  - Claude Code setup and session guidance in `gateway/README.md`
  - any smoke procedure or evidence manifest surfaces introduced for live Claude Code verification
  - statusline, trace, or bounded log surfaces under the gateway runtime if they are required to explain the route evidence
  - any redacted transcript or checklist artifacts added for operator smoke proof
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Verify three live branches exist and are concrete: normal execution, think/planner, and tool-loop continuation.
  - Verify each branch names the expected operator-visible evidence and redaction rules.
  - Verify the smoke proof stays grounded in Claude Code behavior above the landed `/v1/messages` and routing contracts instead of redefining them.
- **Risks / unknowns**:
  - Risk: a gateway-backed smoke recipe may not expose the same client-visible behavior Claude Code does during continuation or plan-mode turns.
  - De-risk plan: require client-real evidence and name each scenario explicitly in `C-10`.
  - Risk: evidence capture may become too invasive or too opaque.
  - De-risk plan: freeze a minimum redacted evidence set tied to pass/fail interpretation.
  - Risk: route confirmation may drift into exposing internal identities as public truth.
  - De-risk plan: keep route evidence in an operator-support role and preserve the public capability boundary language.
- **Rollout / safety**:
  - keep any live evidence redacted and bounded
  - prefer reproducible operator steps over ad hoc local knowledge or one-off shell history
  - avoid making Claude Code smoke success depend on undocumented local environment quirks
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it is `active` because `SEAM-1` has now published the canonical bootstrap path and live Claude Code smoke proof is the next critical-path execution unit
  - Which threads matter most: `THR-08`, `THR-09`
  - What the first seam-local review should focus on: scenario coverage, evidence sufficiency, and whether the smoke proof remains truly client-real instead of slipping back to gateway-only checks
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-10`
  - Threads likely to advance: `THR-09`
  - Review-surface areas likely to shift after landing: `R2`, `R3`, `R4`
  - Downstream seams most likely to require revalidation: `SEAM-3`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
