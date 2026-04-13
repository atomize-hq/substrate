---
seam_id: SEAM-3
seam_slug: anthropic-messages-gateway-surface
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-anthropic-messages-gateway-surface.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - "`docs/foundation/azure-kimi-c02-normalized-event-contract.md` changes tool/action/final semantics or `source_origin` guarantees in a way that changes public Anthropic block mapping"
    - client-surface code starts re-parsing raw Azure payload framing, hidden sentinel syntax, or provider chunk ordering instead of consuming `C-02`
    - public routes, docs, or configuration start exposing planner/executor role selection or other internal policy details
    - the client-surface boundary in `docs/foundation/claude-code-mux-extension-boundary.md` stops preserving a later thin Responses adapter
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
  planned_location: S3
  status: pending
open_remediations: []
---
# SEAM-3 - Anthropic Messages Gateway Surface

## Seam Brief (Restated)

- **Goal / value**: deliver the first Claude Code-compatible Anthropic Messages gateway surface on top of the landed normalized core while keeping the core client-agnostic for later adapter work.
- **Type**: `capability`
- **Scope**
  - **In**:
    - freeze the owned `C-03` Anthropic Messages-compatible contract tightly enough that implementation can proceed without inventing public semantics on the fly
    - map landed `C-02` `tool_intent`, `action`, and `final` semantics into Anthropic-compatible request, streaming, tool loop, and final-response behavior
    - define session continuation and tool-result loop rules that match Claude Code expectations without pushing Anthropic-only types into the core
    - record the exact thin-adapter boundary that keeps later OpenAI Responses support outside the provider-normalization seam
  - **Out**:
    - Azure provider parsing or raw Kimi normalization
    - planner/executor routing policy or model-role selection
    - external identity, deployment/auth boundary lock-in, or downstream Substrate event conformance
- **Touch surface**:
  - `gateway/src/server/mod.rs` and the `/v1/messages` ingress/session path
  - `gateway/src/server/openai_compat.rs` only where shared outer-surface translation remains appropriate
  - `gateway/src/providers/openai.rs` where normalized internal events are rendered into Anthropic-compatible blocks
  - session/tool-result loop handling, streaming adapters, and surface-level verification notes/tests
- **Verification**:
  - the owned `C-03` contract is concrete only if seam-local planning names one public mapping from normalized `tool_intent`, `action`, and `final` semantics into Anthropic `tool_use`, `thinking`, `text`, and stop/continue behavior
  - execution must prove `/v1/messages` remains a thin client surface over landed `C-02` truth rather than a second provider parser
  - tool-result continuation, streaming stop semantics, and final response shaping must be testable without inspecting raw Azure payloads
  - later OpenAI Responses work must remain a thin outer adapter over the same normalized core rather than a forked execution path
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**: `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`
  - **Required threads**: `THR-01`, `THR-02`, `THR-03`
  - **Stale triggers**:
    - `docs/foundation/azure-kimi-c02-normalized-event-contract.md` changes normalized event semantics or stable field guarantees in a way that changes the public surface mapping
    - the `C-01` client-surface boundary stops cleanly separating client ingress from provider parsing or internal policy anchors
    - Anthropic surface work starts depending on raw provider payload details, hidden sentinel syntax, or planner/executor role exposure
- **Threading constraints**
  - **Upstream blockers**: `THR-01` and `THR-02` were revalidated during this promotion run and no longer block active execution; `THR-03` remains the seam-owned publication thread for closeout
  - **Downstream blocked seams**: `SEAM-5`
  - **Contracts produced**: `C-03`
  - **Contracts consumed**: `C-01`, `C-02`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3`
- **Why this seam needs an explicit exit gate**: downstream seams cannot safely assume the first public surface is correct because `/v1/messages` exists; they need closeout-backed proof that the Anthropic contract stays thin over `C-02`, that `THR-03` is publishable, and that public behavior did not leak internal routing or provider details.
- **Expected contracts to publish**: `C-03`
- **Expected threads to publish / advance**: `THR-03` from `identified` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-4` if the public surface starts leaking planner/executor assumptions or changes the session/tool loop in a way that internal policy must honor
  - `SEAM-5` if public docs, config, or event rendering drift toward multiple backend identities or raw provider-shaped output
- **Expected closeout evidence**:
  - a canonical `C-03` contract note or equivalent public-surface source of truth
  - end-to-end Claude Code verification over `/v1/messages`
  - streaming and tool-result loop evidence that references normalized events rather than raw Azure payloads
  - a design note or closeout statement that keeps future Responses work on the same normalized core

## Slice index

- `S1` -> `slice-1-freeze-anthropic-messages-surface-contract.md`
- `S2` -> `slice-2-deliver-anthropic-session-and-streaming-loop.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
