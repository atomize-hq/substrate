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

## Next Resume Slice

Next, continue with:

1. repoint live ADR-to-ADR prerequisite links that should follow the curated implemented paths
2. classify and promote the next ADR cluster
3. keep provisioning ADRs `ADR-0030` and `ADR-0033` separate until that narrower cluster is
   explicitly curated
