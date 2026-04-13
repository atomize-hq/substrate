---
seam_id: SEAM-2
seam_slug: azure-kimi-normalization
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-azure-kimi-normalization.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - "the published `C-01` provider-extension boundary changes in a way that moves or narrows where Azure normalization attaches"
    - "new Azure Foundry evidence shows hidden-tool markers or empty-content behavior that the `C-02` contract does not cover"
    - normalization starts encoding planner/executor routing or public-surface semantics instead of staying a provider-boundary contract
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-2 - Azure Kimi Provider Normalization

## Seam Brief (Restated)

- **Goal / value**: isolate Azure Foundry Kimi behavior behind one provider-boundary normalization contract so every downstream seam consumes stable internal tool/action/final events instead of raw Azure `tool_calls`, `reasoning_content`, or sentinel syntax.
- **Type**: `integration`
- **Scope**
  - **In**:
    - define the owned `C-02` normalized event contract concretely enough for downstream seam planning and later producer-seam contract approval
    - capture Azure Foundry evidence and fixture coverage for explicit `tool_calls`, hidden tool intent in `reasoning_content`, mixed cases, and no-tool cases
    - implement or plan the provider-boundary parser/adapter behavior that converts Azure responses into one internal event model
    - record what parts of the adopted `claude-code-mux` provider transform are reused versus bypassed for Azure Kimi behavior
  - **Out**:
    - Anthropic Messages gateway delivery
    - planner/executor routing policy or model-role selection
    - public backend identity, deployment transport, or Substrate-facing boundary freeze
- **Touch surface**:
  - adopted gateway provider adapter and provider-mode selection boundary from `C-01`
  - reasoning parser and normalization helpers for Azure chat-completions responses
  - normalized event model definitions consumed by later seams
  - Azure probe artifacts, fixture corpus, and regression tests
  - seam-local notes that record parser invariants and reuse-versus-bypass decisions
- **Verification**:
  - `C-02` is concrete enough only if the seam names one normalized event vocabulary, one provenance/debug posture, and one verification checklist that covers explicit tool calls, hidden markers, mixed evidence, and no-tool responses.
  - the seam must prove downstream consumers can rely on normalized semantics only and never re-parse Azure sentinel text.
  - the seam must keep provider parsing independent from planner/executor policy so later seams can choose routing without reopening Azure normalization.
  - the seam must carry enough raw evidence and fixture coverage to decide whether `Kimi-K2.5` shares or diverges from the currently observed `Kimi-K2-Thinking` behavior.
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**: `../../governance/seam-1-closeout.md`
  - **Required threads**: `THR-01`, `THR-02`
  - **Stale triggers**:
    - `docs/foundation/claude-code-mux-extension-boundary.md` changes the provider hook or reuse/bypass expectations inherited from `SEAM-1`
    - Azure probes show new hidden-tool variants, malformed sentinel ordering, or empty-content behavior that the frozen `C-02` contract does not cover
    - downstream delivery work starts depending on raw provider payload shape instead of the normalized event contract
- **Threading constraints**
  - **Upstream blockers**: `THR-01` is already `published`; no other upstream seam blocks decomposition
  - **Downstream blocked seams**: `SEAM-3`, `SEAM-4`, `SEAM-5`
  - **Contracts produced**: `C-02`
  - **Contracts consumed**: `C-01`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S4`
- **Why this seam needs an explicit exit gate**: downstream seams cannot safely promote on "Azure parsing exists"; they need closeout-backed proof that `C-02` is concrete, that explicit and hidden tool intent land in one normalized event model, and that outbound stale triggers are recorded.
- **Expected contracts to publish**: `C-02`
- **Expected threads to publish / advance**: `THR-02` from `defined` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-3` if normalized event semantics change the ordering or shape of tool/action/final events consumed by the Anthropic surface
  - `SEAM-4` if parser work accidentally absorbs routing policy or model-role assumptions
  - `SEAM-5` if raw Azure payload details leak into the structured-event boundary
- **Expected closeout evidence**:
  - a concrete `C-02` contract definition with invariants and verification checklist
  - Azure probe evidence and fixture coverage for explicit, hidden, mixed, and no-tool cases
  - regression coverage proving explicit and hidden Azure tool intent normalize into one internal event model
  - a written note or code-local evidence stating what upstream provider behavior was reused versus bypassed
  - closeout accounting for any newly observed hidden-tool variants and their downstream stale-trigger impact

## Slice index

- `S1` -> `slice-1-freeze-normalized-event-contract.md`
- `S2` -> `slice-2-capture-azure-evidence-and-fixtures.md`
- `S3` -> `slice-3-implement-provider-normalization-boundary.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
