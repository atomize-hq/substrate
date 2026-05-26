# Azure Foundry Runtime Transport `C-07` Contract

## Purpose

This note is the canonical landing artifact for `C-07`.
It defines the Azure Foundry runtime transport contract for the gateway while keeping Azure-specific request construction below the landed public and normalized contracts.

This contract is intentionally narrow:

- it defines the primary GA v1 transport truth for Azure OpenAI-compatible requests
- it defines how Azure auth, base URL composition, and deployment selection work for the gateway
- it defines the provider-mode/helper boundary that keeps Azure-specific behavior isolated
- it keeps `Kimi-K2-Thinking` and `Kimi-K2.5` as internal routing targets, not public backend identities

It does not define:

- the public `/v1/messages` surface
- normalized event semantics
- planner/executor policy
- live operator smoke procedure

## Canonical Sources

This contract is grounded in the seam-local evidence and the landed upstream basis below, with current Microsoft Learn v1 documentation used to confirm Azure GA transport truth:

Repo-grounded anchors:

- `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/threaded-seams/seam-1-azure-foundry-runtime-transport/seam.md`
- `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/threaded-seams/seam-1-azure-foundry-runtime-transport/review.md`
- `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/threading.md`
- `crates/gateway/docs/project_management/packs/active/azure-foundry-provider-transport/scope_brief.md`
- `docs/foundation/anthropic-messages-c03-contract.md`
- `docs/foundation/planner-executor-c04-policy-contract.md`
- `docs/foundation/substrate-boundary-c05-contract.md`
- `docs/foundation/substrate-structured-events-c06-contract.md`

Current external v1 evidence used by this contract:

- API key auth uses the `api-key` header
- auth token / Entra token auth uses the `authorization` header
- the GA v1 base URL is under `https://{resource}.openai.azure.com/openai/v1/`
- current docs also accept `https://{resource}.services.ai.azure.com/openai/v1/` as an equivalent host variant
- `chat/completions` and `responses` are called relative to that base URL
- the deployed Azure model or deployment name is passed in the request `model` field
- the GA v1 path does not require a dated `api-version` query parameter

If this note and the current Microsoft Learn v1 documentation disagree, the note must be revalidated before downstream implementation proceeds.

## Primary GA Transport Truth

### Base URL

The primary Azure base URL for this gateway is:

- `https://{resource}.openai.azure.com/openai/v1/`

The following host variant is acceptable as an equivalent GA v1 base URL if the repo needs to support it:

- `https://{resource}.services.ai.azure.com/openai/v1/`

Required behavior:

- request paths are appended relative to the GA v1 base URL
- `chat/completions` and `responses` are treated as relative resource paths under that base URL
- the contract must not freeze `/openai/deployments/{deployment-id}/...` as the primary transport shape

### Authentication

Required behavior:

- API key auth uses the `api-key` header
- Entra or OAuth auth uses the `authorization` header with a bearer token
- the gateway must not assume generic bearer auth is the only Azure path
- the gateway must not require Azure API key auth to flow through `Authorization: Bearer ...`

### Deployment Selection

Required behavior:

- the Azure deployment or model selection is carried in the request `model` field
- the request body selects `Kimi-K2-Thinking` or `Kimi-K2.5` through internal model mapping, not by exposing a public backend identity
- the transport contract must not require the deployment name to appear in the URL path for the primary GA v1 path

### `api-version`

Required behavior:

- the GA v1 path must not require a dated `api-version` parameter
- if a compatibility path includes `api-version`, it must be treated as secondary compatibility behavior only
- the primary contract must remain valid when the query parameter is omitted

### Provider Boundary

Required behavior:

- Azure-specific auth and request-target construction stay behind an explicit provider-mode or helper boundary
- non-Azure OpenAI-compatible providers keep their existing behavior unless intentionally routed through the Azure path
- the generic OpenAI-compatible path must not pick up Azure-specific assumptions by default

## Runtime And Config Anchors

These are the repo surfaces governed by this contract for later slices:

- `gateway/src/providers/openai.rs`
- `gateway/src/providers/registry.rs`
- `gateway/src/providers/mod.rs`
- `gateway/src/cli/mod.rs`
- `gateway/config/default.example.toml`
- `gateway/config/models.example.toml`
- `gateway/README.md`

Required behavior:

- the Azure transport contract must be expressible through these surfaces without inventing a second request schema
- config and examples must be able to express Azure provider mode, base URL, auth posture, and think/default deployment mapping
- config and examples must not require ad hoc header hacks to represent Azure transport truth
- the landed `C-03` and `C-04` behavior remains above the provider seam and is not redefined by Azure transport fields
- Azure-specific semantics remain limited to auth, base URL, deployment selection, and the helper boundary that implements them

## Secondary Compatibility Notes

This section is intentionally non-normative.

If the repo must support preview or legacy deployment-path semantics, they may be documented only as compatibility behavior:

- `/openai/deployments/{deployment-id}/...`
- dated `api-version` query parameters

Rules for any compatibility path:

- it must remain secondary to the GA v1 contract above
- it must not become the default implementation path
- it must not replace the `model`-field deployment selection rule for GA v1
- it must not force non-Azure providers to adopt Azure-specific transport behavior

## Internal Routing Rules

Required behavior:

- `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets
- those names are mapped to Azure deployment names through config/model mapping surfaces
- the contract must not expose planner/executor roles or Azure deployment mechanics as public backend identities
- the contract must not rename the public gateway surface to make Azure deployment details externally visible

## Failure Modes

The contract is incomplete if any of the following remain true:

- the primary path still depends on `/openai/deployments/{deployment-id}/...` rather than GA v1 base URL plus `model`
- API key auth still requires `Authorization: Bearer ...` on the primary Azure path
- the primary path still requires a dated `api-version`
- Azure-specific branching leaks into the generic OpenAI-compatible provider path
- model-to-deployment mapping is still implicit enough that implementation has to guess which Azure deployment a routed model should use
- preview semantics have become the default truth instead of a documented compatibility note
- a deployment identifier is missing and the contract does not say how the routed model resolves to an Azure deployment name
- auth mode is ambiguous enough that API key and Entra/OAuth requests can be built incorrectly from the same config shape
- config or example surfaces cannot represent Azure provider mode, base URL, auth posture, and think/default mapping without a free-form header workaround
- request-body shape would need to change above the provider seam instead of preserving the landed OpenAI-compatible body contract
- a transport change would force public-surface or `C-03`/`C-04` behavior changes above the provider seam
- later slices would have to rediscover whether Azure semantics belong in request body, headers, config, or the public gateway surface

## Verification Checklist

`C-07` is complete only if a reviewer can answer yes to all of the following without reading runtime code:

- can the primary GA v1 base URL be stated as `https://{resource}.openai.azure.com/openai/v1/` or the documented `.services.ai.azure.com` variant
- can `chat/completions` and `responses` be explained as relative paths under that base URL
- can API key auth and bearer-token auth be distinguished by header, not by URL shape
- can deployment selection be explained through the request `model` field
- can the reviewer state that a dated `api-version` is not required for the primary GA path
- can the reviewer explain how `Kimi-K2-Thinking` and `Kimi-K2.5` stay internal while mapping to Azure deployment names
- can the reviewer explain how Azure-specific behavior remains behind a provider-mode/helper boundary
- can the reviewer identify any preview/deployment-path behavior as secondary compatibility only
- can the reviewer confirm Azure keeps the existing OpenAI-compatible request-body shape and does not introduce a second request schema above the provider seam
- can the reviewer confirm config and example surfaces can express provider mode, base URL, auth posture, and think/default mapping without ad hoc header hacks
- can the reviewer confirm request-body invariance is preserved while Azure semantics remain limited to auth, base URL, and deployment selection

## Drift Guards

Downstream revalidation is required if any of the following changes:

- the Azure request body stops matching the landed OpenAI-compatible request shape
- config or example surfaces can no longer represent the Azure provider mode or auth posture cleanly
- a later change pushes Azure-specific semantics into the public surface instead of the provider boundary
- preview or legacy deployment-path semantics start displacing the GA v1 path as primary truth
- a mapping change forces the contract to redefine `C-03` or `C-04` behavior above the provider seam
- the Azure deployment selection rule can no longer be expressed as internal mapping into the request `model` field

## Compatibility Notes

- This contract is compatible with the landed `C-03` public surface and does not redefine it.
- This contract is compatible with the landed `C-04` policy boundary and does not expose planner/executor roles.
- This contract is compatible with the landed `C-05` and `C-06` boundary contracts and does not move Azure transport semantics above the provider seam.
- This note is intentionally capability-oriented and must not be read as a public declaration of provider identity or public deployment topology.
