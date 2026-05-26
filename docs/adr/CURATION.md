# ADR Curation Ledger

This ledger records the curation rules and the current classification state for ADRs that may move
from `docs/project_management/adrs/**` into the stable `docs/adr/**` tree.

## Classification Axes

Every ADR under review is classified on two axes:

1. Curation disposition
   - `stable_keeper`: belongs in `docs/adr/**`
   - `historical_only`: keep only as history or audit trail
   - `superseded`: retained only as a replaced decision record
2. Implementation posture
   - `implemented`
   - `draft_but_implemented`
   - `still_draft`

`draft_but_implemented` is the key normalization bucket for this repo because multiple ADRs still
say `Status: Draft` while stable docs, code, and tests already rely on their decisions.

## Keep Criteria

Promote an ADR into `docs/adr/**` when one or more of these conditions hold:

- it defines a current operator-facing contract that stable docs already expose
- it defines a runtime boundary or data model that current code already implements
- stable docs, tests, or contract references still use the ADR number as an authority anchor
- later ADRs still depend on it as an active prerequisite rather than only historical context

Do not promote an ADR yet when it is primarily:

- a planning-pack wrapper around already-extracted stable docs
- a feature-local execution plan rather than a durable architectural decision
- a superseded proposal whose current value is only historical context

## Migration Policy

- Preferred strategy: `restate + supersede`
- Stable target tree:
  - `docs/adr/implemented/`
  - `docs/adr/draft/`
  - `docs/adr/historical/`
- Promotion order:
  1. curate stable keepers with current operator/runtime contract weight
  2. repoint stable references to `docs/adr/**`
  3. leave compatibility stubs in `docs/project_management/adrs/**` only where older planning or
     archive material still needs a breadcrumb

## First Cluster: Policy and Gateway Contract ADRs

This is the first high-value cluster called out by the retirement tracker.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0027 | `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md` | `stable_keeper` | `draft_but_implemented` | Stable policy contract/schema docs already exist under `docs/reference/policy/**`, broker and shell config/policy models implement the surface, and multiple later ADRs still treat ADR-0027 as the root contract. |
| ADR-0040 | `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md` | `stable_keeper` | `draft_but_implemented` | The gateway operator contract is already published under `docs/contracts/gateway/operator-contract.md`, and shell/world-service lifecycle code plus tests implement the named gateway command family and ownership boundary. |
| ADR-0041 | `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` | `stable_keeper` | `draft_but_implemented` | Stable gateway backend-selection docs exist, `llm.routing.default_backend` is implemented in config models, and gateway runtime code already carries adapter-style backend bindings beyond a planning-only statement. |
| ADR-0042 | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | `stable_keeper` | `draft_but_implemented` | The tuple and placement-posture surfaces are already reflected in gateway lifecycle/status code and tests, and later docs treat ADR-0042 as the semantic owner of tuple meaning. |
| ADR-0043 | `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` | `stable_keeper` | `draft_but_implemented` | Stable tuple-constraint docs exist under `docs/reference/policy/tuple_constraints.md`, and broker/shell policy models already parse and validate `llm.constraints.*`. |
| ADR-0046 | `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md` | `stable_keeper` | `draft_but_implemented` | Although written as a thin implementation follow-on, the gateway runtime and shell lifecycle code already implement inventory-backed default-backend handling, integrated auth shaping, and multi-backend runtime wiring. |

## First-Cluster Decision

The first cluster belongs in `docs/adr/implemented/`, not in a long-lived draft bucket.

Rationale:

- Each ADR still defines active product/runtime truth.
- Each ADR already has stable downstream references, code touchpoints, or both.
- Leaving them under `docs/project_management/adrs/draft/**` preserves an inaccurate signal about
  implementation maturity and keeps stable references pointed at a namespace scheduled for
  retirement.

## Promoted In This Slice

The following curated ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md`

## Explicit Non-Decisions In This Slice

- This ledger does not yet classify the full ADR registry.
- The legacy project-management ADR files remain in place with relocation notes; this slice does
  not yet archive or delete those historical source files.
- This ledger does not yet decide the final disposition of provisioning ADRs `ADR-0030` and
  `ADR-0033`; they remain separate because their pack-owned contract references may still be
  intentional until that narrower provisioning cluster is curated.

## Second Cluster: Superseded Gateway and Agent-Hub Predecessors

This is the next bounded cluster after live-consumer prerequisite repointing because these ADRs
still carry legacy backlinks but are no longer the active architectural truth.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0023 | `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md` | `superseded` | `still_draft` | Its gateway-capability intent is preserved historically, but the current runtime ownership split now lives in ADR-0040 and the adapter boundary in ADR-0041. |
| ADR-0024 | `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md` | `superseded` | `still_draft` | The stable backend-id and allowlisting goals survived, but the Substrate-local engine framing was replaced by the gateway-owned adapter contract in ADR-0041. |
| ADR-0025 | `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md` | `superseded` | `still_draft` | The core Agent Hub direction remains relevant, but the newer successor ADRs changed the identity and backend semantics enough that this draft is historical, not current truth. |

## Promoted In This Slice

The following curated historical ADRs now exist under `docs/adr/historical/`:

- `docs/adr/historical/ADR-0023-in-world-llm-gateway-front-door.md`
- `docs/adr/historical/ADR-0024-cli-backend-provider-engine.md`
- `docs/adr/historical/ADR-0025-agent-hub-core-role-swappable.md`

## Queued Draft: ADR-0026

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0026 | `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md` | `stable_keeper` | `still_draft` | The toolbox concept is still queued work rather than settled history. It has not been landed, and when implementation is ready it should be rewritten against the later orchestration, identity, and trace semantics rather than treated as an already-closed predecessor. |

Curated queued draft ADR:

- `docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md`

## Third Cluster: Orchestration and Workflow ADRs

This is the next current ADR cluster after the first contract-heavy promotion and predecessor
cleanup. It groups the accepted execution/trace contracts that are already implemented plus the
active queued orchestration/workflow follow-ons that still need restatement before landing.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0017 | `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` | `stable_keeper` | `implemented` | Accepted and materially implemented in REPL/output-routing behavior; still used as a live foundation for orchestration and trace-related work. |
| ADR-0028 | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` | `stable_keeper` | `implemented` | Accepted and implemented through the canonical tracing stack and stable trace internals docs. |
| ADR-0047 | `docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md` | `stable_keeper` | `draft_but_implemented` | Its durable host-session and terminal-delivery posture is already treated as current runtime truth and backed by runtime/test anchors. |
| ADR-0021 | `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md` | `stable_keeper` | `still_draft` | Still queued architectural input for a future workflow runtime; not implemented and should remain draft. |
| ADR-0022 | `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md` | `stable_keeper` | `still_draft` | Still queued as a workflow-node derivative of the broader workflow-engine direction. |
| ADR-0026 | `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md` | `stable_keeper` | `still_draft` | Queued toolbox work that still needs a rewrite before implementation; active draft, not history. |
| ADR-0029 | `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md` | `stable_keeper` | `still_draft` | Queued host-router/service direction that remains active architectural input but not landed behavior. |
| ADR-0044 | `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md` | `stable_keeper` | `still_draft` | Queued successor Agent Hub contract that should remain draft until the orchestration/session stack is restated and landed. |
| ADR-0045 | `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md` | `stable_keeper` | `still_draft` | Queued toolbox successor contract that depends on the surrounding orchestration and identity work. |

## Promoted In This Slice

The following curated implemented ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`

The following curated draft ADRs now exist under `docs/adr/draft/`:

- `docs/adr/draft/ADR-0021-substrate-workflow-engine.md`
- `docs/adr/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`
- `docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md`
- `docs/adr/draft/ADR-0029-host-event-bus-and-router-daemon.md`
- `docs/adr/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

## Fourth Cluster: Remaining Current ADR Tail

This tail slice captures the remaining current non-provisioning ADRs that still matter after the
contract-heavy, predecessor, and orchestration/workflow clusters were promoted.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0016 | `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md` | `stable_keeper` | `draft_but_implemented` | World-first REPL semantics are already implemented and still anchor later output-routing and tracing work, so this belongs in the implemented tree despite the legacy draft label. |
| ADR-0019 | `docs/project_management/adrs/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md` | `stable_keeper` | `still_draft` | Still queued UX work around config visibility and scope messaging; active input, not landed behavior. |
| ADR-0020 | `docs/project_management/adrs/draft/ADR-0020-profiles-config-policy-snapshots.md` | `stable_keeper` | `still_draft` | Still queued architecture for full profile snapshots and surface scoping; not implemented and should remain draft. |

## Promoted In This Slice

The following curated implemented ADR now exists under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0016-world-first-repl-persistent-pty.md`

The following curated draft ADRs now exist under `docs/adr/draft/`:

- `docs/adr/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`
- `docs/adr/draft/ADR-0020-profiles-config-policy-snapshots.md`

## Next Resume Slice

The remaining current ADR tail is now classified and promoted. Next, continue with:

1. keep provisioning ADRs `ADR-0030` and `ADR-0033` as the next explicit curation slice
2. after that, narrow any remaining `docs/project_management/**` dependency surface that still
   points stable readers at retiring namespaces
3. do not reopen already-curated ADR clusters unless new stable references are discovered
