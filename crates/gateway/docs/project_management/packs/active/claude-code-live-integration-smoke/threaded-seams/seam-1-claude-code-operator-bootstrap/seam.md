---
seam_id: SEAM-1
seam_slug: claude-code-operator-bootstrap
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-claude-code-operator-bootstrap.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-1-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md
  required_threads: []
  stale_triggers:
    - docs/foundation/azure-foundry-c07-runtime-transport-contract.md changes Azure provider setup, model-mapping, or startup assumptions that the bootstrap path depends on
    - docs/foundation/azure-foundry-c08-operator-verification-contract.md changes the minimum pre-smoke evidence or redaction posture that C-09 must require
    - gateway/README.md, gateway/config/default.example.toml, gateway/config/models.example.toml, or the startup/statusline/tracing runtime anchors drift enough that the recorded operator sequence is no longer true
    - Claude Code attachment rules change materially enough that ANTHROPIC_BASE_URL and placeholder API key guidance must be rewritten
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

## Seam Brief (Restated)

- **Goal / value**: freeze one canonical operator bootstrap path that starts with Azure prerequisites, carries through gateway config and startup validation, attaches Claude Code to the Anthropic-compatible gateway path, and enables the minimum statusline or trace hooks that later live smoke work depends on.
- **Type**: `integration`
- **Scope**
  - In:
    - define the owned `C-09` bootstrap contract and the single landing artifact that will hold it
    - declare the Azure credential, deployment, and config prerequisites an operator must satisfy before a live session is meaningful
    - make the gateway startup, validation, Claude Code attachment, statusline, and trace-hook steps reproducible without reading runtime code
    - align docs, examples, and any bounded helper surfaces with the capability-oriented boundary inherited from `C-05`
  - Out:
    - running or proving the normal, think, and tool-loop smoke scenarios themselves
    - redesigning the router policy, `/v1/messages` semantics, or Azure transport behavior
    - broad troubleshooting ownership or support-runbook work beyond the bootstrap surface
- **Touch surface**:
  - `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`
  - `gateway/README.md`
  - `gateway/config/default.example.toml`
  - `gateway/config/models.example.toml`
  - `gateway/src/cli/mod.rs`
  - `gateway/src/main.rs`
  - `gateway/src/server/mod.rs`
  - any bounded operator checklist or launch-helper surface introduced for Claude Code bootstrap
- **Verification**:
  - because this seam produces owned `C-09`, pre-exec verification is about making the bootstrap contract concrete enough that implementation can proceed without guessing about config shape, startup order, or evidence hooks
  - a reviewer can state the canonical operator sequence from Azure prerequisites through `claude` launch without reading code
  - a reviewer can identify which statusline and trace surfaces count as required pre-smoke evidence and which ones are optional convenience
  - the bootstrap story stays capability-oriented and does not make Azure deployment names, planner/executor roles, or loopback convenience look like the public product identity
  - current repo anchors already support execution planning: `gateway/README.md` names the Claude Code env flow, `install-statusline`, and tracing posture, the example config encodes the Azure Kimi mapping, and the runtime still writes `last_routing.json` plus `trace.jsonl` through the documented surfaces
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**:
    - `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-1-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-4-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-5-closeout.md`
  - **Required threads**: none; this seam owns `THR-08`
  - **Stale triggers**:
    - `C-07` changes Azure setup or mapping semantics
    - `C-08` changes pre-smoke evidence or redaction requirements
    - README, config examples, startup validation, or statusline/tracing runtime anchors drift from the recorded operator sequence
    - Claude Code environment attachment changes enough that the bootstrap command posture is no longer accurate
- **Threading constraints**
  - **Upstream blockers**: none; the pack basis is closeout-backed and current
  - **Downstream blocked seams**: `SEAM-2` and `SEAM-3` both depend on `THR-08` and cannot safely execute until `C-09` lands and publishes bootstrap truth
  - **Contracts produced**: `C-09`
  - **Contracts consumed**: `C-03`, `C-04`, `C-05`, `C-07`, `C-08`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3`
- **Why this seam needs an explicit exit gate**: downstream smoke and troubleshooting seams should only consume bootstrap truth after the landed contract, startup assets, and evidence-hook surfaces agree with runtime reality and are recorded in closeout-backed form
- **Expected contracts to publish**: `C-09`
- **Expected threads to publish / advance**: `THR-08`
- **Likely downstream stale triggers**:
  - the gateway config or startup flow changes enough to invalidate the bootstrap sequence
  - the statusline or trace-hook story changes enough to alter what `SEAM-2` counts as required evidence
  - Claude Code attachment or routed-model evidence changes enough to require downstream revalidation
- **Expected closeout evidence**:
  - the landed `C-09` contract artifact
  - operator-facing bootstrap docs or checklist surfaces aligned with runtime behavior
  - statusline and trace-hook instructions aligned with the actual CLI and server surfaces
  - closeout accounting for any planned-versus-landed delta that affects `SEAM-2` or `SEAM-3`

## Slice index

- `S1` -> `slice-1-freeze-claude-code-bootstrap-contract.md`
- `S2` -> `slice-2-deliver-bootstrap-assets-and-evidence-hooks.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
