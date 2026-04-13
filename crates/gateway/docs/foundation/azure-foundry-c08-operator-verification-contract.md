# Azure Foundry Operator Verification `C-08` Contract

## Purpose

This note is the canonical landing artifact for `C-08`.
It defines the operator verification contract for Azure Foundry live smoke and troubleshooting while keeping the public gateway surface capability-oriented and keeping provider, planner, executor, and deployment details internal.

This contract is intentionally narrow:

- it defines the live `/v1/messages` smoke path operators must use for Azure Kimi verification
- it defines what success looks like for the `Kimi-K2-Thinking` and `Kimi-K2.5` routes
- it defines the minimum redacted evidence an operator must capture
- it defines a troubleshooting taxonomy that separates auth, URL, deployment, route, and transport drift failures

It does not define:

- runtime transport construction
- public API semantics beyond the landed Anthropic surface
- planner/executor routing policy
- deployment or provider identity as public backend truth

## Canonical Sources

This contract is grounded in the landed basis and seam-local planning below:

- `docs/foundation/azure-foundry-c07-runtime-transport-contract.md`
- `docs/foundation/anthropic-messages-c03-contract.md`
- `docs/foundation/planner-executor-c04-policy-contract.md`
- `docs/foundation/substrate-boundary-c05-contract.md`
- `docs/project_management/packs/active/azure-foundry-provider-transport/threading.md`
- `docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-1-closeout.md`
- `docs/project_management/packs/active/azure-foundry-provider-transport/threaded-seams/seam-2-azure-live-smoke-operator-readiness/seam.md`
- `docs/project_management/packs/active/azure-foundry-provider-transport/threaded-seams/seam-2-azure-live-smoke-operator-readiness/review.md`

If this note and the landed basis disagree, this note must be revalidated before downstream operator work proceeds.

## Canonical Smoke Path

The canonical operator smoke path is the public Anthropic-compatible `/v1/messages` surface, exercised against the gateway and routed internally through the Azure Kimi mapping for:

- `Kimi-K2-Thinking`
- `Kimi-K2.5`

Required behavior:

- the operator verifies the public gateway path, not a provider-only side channel
- the operator uses the same public `/v1/messages` ingress that downstream clients consume
- `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets, not public backend identities
- Azure deployment names remain internal mapping data under `C-07` and are not promoted into the public verification identity

## Success Signals

The smoke is successful only when the operator can confirm all of the following from the public gateway interaction and redacted evidence:

- the gateway accepts the request on `/v1/messages`
- the routed think path reaches the intended Azure-backed internal target
- the routed default path reaches the intended Azure-backed internal target
- the response path is consistent with the landed Anthropic-compatible behavior
- no secret material or internal deployment identity is exposed in the captured evidence

Success is intentionally defined at the capability boundary:

- operators verify that the gateway capability works
- operators do not need to prove or expose provider internals as public contract truth

## Required Evidence

An operator run is not complete unless the evidence set includes:

- the request intent or invocation summary for the `/v1/messages` smoke
- the routed model name used for the think path
- the routed model name used for the default path
- the public success signal returned by the gateway
- redacted failure evidence when a run does not pass
- enough context to distinguish whether the failure was auth, URL, deployment mapping, route selection, `api-version` compatibility, or transport drift

Evidence must remain minimal and redacted:

- redact API keys, bearer tokens, and any other credential material
- redact raw deployment names if they would expose provider internals beyond the contract boundary
- redact hostnames or config fragments when they are only useful as secrets or environment identifiers
- keep the evidence legible enough for a reviewer to determine the failure class without reading runtime code

## Redaction Rules

The redaction model is capability-oriented, not forensic.

Required rules:

- do not publish credentials
- do not publish provider-specific secret material
- do not expose planner/executor role selection as a public identity
- do not expose Azure deployment mechanics as a public backend name
- do not expand redaction so far that the operator can no longer classify the failure

Allowed evidence includes redacted mentions of:

- public `/v1/messages` usage
- the internal routed model label needed to explain which verification branch was exercised
- the failure class and the minimal surrounding context needed to diagnose it

## Troubleshooting Taxonomy

Troubleshooting must be grouped into these operator-facing classes:

### Auth

Use when the gateway cannot authenticate to Azure or the credential posture is wrong.

### URL

Use when the base URL, host variant, request target, or secondary `api-version` compatibility path is malformed or points at the wrong Azure endpoint.

### Deployment

Use when the think/default mapping does not resolve to the intended Azure deployment target.

### Route

Use when the gateway selects the wrong internal route for think or default traffic, even though the transport itself is otherwise valid.

### Transport Drift

Use when the observed behavior no longer matches `C-07`, including request-target or request-body assumptions that should stay below the provider seam, or when a compatibility-only `api-version` path starts behaving like primary transport truth.

Required boundary statement:

- these categories are operator-facing diagnostics, not public backend identities
- the taxonomy must stay specific enough to guide remediation without leaking internal implementation detail

## Boundary And Identity Rules

`C-08` must preserve the `C-05` public boundary.

Required behavior:

- the gateway presents one logical backend capability to the outside world
- public docs and examples remain capability-oriented
- operators do not need to choose between planner, executor, or provider identities to use the verification contract
- Azure-specific transport semantics remain below the provider seam and below the public gateway identity
- `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets used to explain verification coverage, not public product identities

Non-goals for this note:

- redefining the public gateway identity
- exposing deployment topology as a public contract
- moving planner/executor policy into the operator verification surface
- requiring a second public request schema for Azure verification

## Verification Checklist

`C-08` is complete only if a reviewer can answer yes to all of the following without reading runtime code:

- can the reviewer state that the canonical smoke path uses `/v1/messages`
- can the reviewer explain that both `Kimi-K2-Thinking` and `Kimi-K2.5` are covered as internal routing targets
- can the reviewer describe the success signals without naming provider internals as public identity
- can the reviewer identify the minimum redacted evidence that must be captured for pass and fail cases
- can the reviewer classify failures into auth, URL, deployment, route, or transport drift
- can the reviewer confirm the note stays capability-oriented and preserves the `C-05` boundary
- can the reviewer confirm the note does not require reading runtime code to understand the operator verification contract

## Compatibility Notes

- This contract is compatible with the landed `C-03` public surface and does not redefine `/v1/messages`.
- This contract is compatible with the landed `C-04` policy boundary and does not expose planner/executor selection.
- This contract is compatible with the landed `C-05` boundary and does not leak provider or deployment identity as public truth.
- This contract is intentionally capability-oriented and must not be read as a public declaration of provider identity or deployment topology.
