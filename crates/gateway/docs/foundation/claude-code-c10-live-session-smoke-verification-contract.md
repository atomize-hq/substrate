# Claude Code Live Session Smoke Verification `C-10` Contract

## Purpose

This note is the canonical landing artifact for `C-10`.
It freezes the live Claude Code smoke verification contract for the `SEAM-2` path while keeping the public gateway identity capability-oriented and keeping provider, planner, executor, and deployment details internal.

This contract is intentionally narrow:

- it defines the three required live branches: normal execution, think/planner, and tool-loop continuation
- it defines the minimum redacted evidence posture operators must capture for each branch
- it defines the canonical landing path for this live smoke truth
- it defines the reviewer checklist that lets downstream work execute without rereading runtime code

It does not define:

- runtime transport construction
- public `/v1/messages` semantics beyond the landed public surface
- planner/executor routing policy
- smoke-procedure delivery, closeout accounting, or downstream troubleshooting ownership

## Canonical Sources

This contract is grounded in the landed basis and seam-local planning below:

- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-2-live-session-smoke-verification/seam.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-2-live-session-smoke-verification/review.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-2-live-session-smoke-verification/slice-1-freeze-live-session-smoke-contract-and-coverage.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threading.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/governance/seam-1-closeout.md`
- `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`
- `docs/foundation/azure-foundry-c08-operator-verification-contract.md`
- `docs/foundation/anthropic-messages-c03-contract.md`
- `docs/foundation/planner-executor-c04-policy-contract.md`
- `docs/foundation/substrate-boundary-c05-contract.md`

Repo-grounded runtime and documentation anchors:

- `gateway/README.md`
- `gateway/src/router/mod.rs`
- `gateway/src/server/mod.rs`

If this note and those anchors disagree, the note or the upstream evidence must be revalidated before downstream implementation proceeds.

## Live Smoke Contract

The canonical live smoke proof is a real Claude Code session exercised through the public gateway path, not a provider-only probe and not a gateway-only synthetic route check.

Required live branches:

1. Normal execution
2. Think / planner execution
3. Tool-loop continuation after tool results

Required behavior:

- the operator verifies the public gateway path, not a provider-only side channel
- the operator uses the same public `/v1/messages` ingress that downstream clients consume
- tool-loop continuation remains a distinct branch, separate from ordinary think/planner execution
- route labels, provider names, and deployment names remain internal or support-facing evidence, not public truth
- the smoke story stays capability-oriented above `C-05` and does not force operators to learn provider identity as a prerequisite for proving behavior

## Branch Coverage

### 1. Normal Execution

This branch proves the gateway can carry a standard Claude Code turn through the public `/v1/messages` path and preserve the expected routing and response behavior.

Evidence anchors:

- `gateway/src/server/mod.rs` routes `/v1/messages`
- `gateway/README.md` documents the Claude Code bootstrap and statusline flow
- `gateway/src/server/mod.rs` writes routing state for later operator inspection

### 2. Think / Planner Execution

This branch proves a turn with thinking enabled reaches the think path and remains distinguishable from the default execution path.

Evidence anchors:

- `gateway/src/router/mod.rs` checks `thinking` and routes to the think model
- `gateway/src/server/mod.rs` preserves the routed model and emits routing evidence
- `gateway/README.md` describes the operator-facing statusline and routing-history surfaces

### 3. Tool-Loop Continuation

This branch proves a plan-mode turn that resumes after tool results is treated as a continuation branch and does not collapse into the ordinary think branch.

Evidence anchors:

- `gateway/src/router/mod.rs` documents tool-result continuation handoff ahead of think routing
- `gateway/src/router/mod.rs` detects tool-result-only continuations in plan mode
- `gateway/src/server/mod.rs` injects continuation prompts for qualifying tool-result follow-up turns

## Minimum Redacted Evidence Posture

An operator run is not ready for live smoke unless the evidence set can show:

- the public `/v1/messages` path was used against Claude Code
- the branch exercised was normal execution, think/planner, or tool-loop continuation
- the routing state or statusline evidence is available through `~/.substrate-gateway/last_routing.json`
- tracing is available only when enabled through `~/.substrate-gateway/trace.jsonl`
- the operator can explain the branch outcome without exposing provider or deployment identity as public truth

This contract treats the evidence as minimum viable operator proof, not as a request for exhaustive forensics.

Required evidence:

- request intent or invocation summary for the `/v1/messages` smoke
- routed model label for the branch exercised
- statusline or routing-history evidence from `last_routing.json`
- redacted pass or fail signal from the session outcome

Optional evidence:

- `trace.jsonl` when redacted debugging context is needed
- additional redacted request or response context when it helps classify the failure

Redaction rules:

- redact API keys, bearer tokens, and other credentials
- redact raw deployment names if they would expose provider internals beyond the contract boundary
- redact hostnames or config fragments when they are only useful as secrets or environment identifiers
- keep the evidence legible enough for a reviewer to determine the failure class without reading runtime code

## Failure-Class Taxonomy

Live smoke failures should be classified using the operator-facing classes aligned with `C-08` and `C-09`:

### Auth

Use when Azure credentials are missing, malformed, or rejected.

### URL

Use when the base URL, host variant, or request target is malformed or points at the wrong endpoint.

### Deployment

Use when the think/default mapping does not resolve to the intended internal Azure deployment target.

### Route

Use when the gateway selected the wrong internal route for think, default, or continuation traffic even though the transport itself is otherwise valid.

### Transport Drift

Use when the observed behavior no longer matches the landed transport or bootstrap contract, including request-target assumptions, evidence-hook behavior, or redaction posture drift that no longer matches the operator path.

Required boundary statement:

- these categories are operator-facing diagnostics, not public backend identities
- the taxonomy must stay specific enough to guide remediation without leaking internal implementation detail

## Artifact Path And Verification Checklist

The single canonical landing path for this contract is:

- `docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md`

`C-10` is complete only if a reviewer can answer yes to all of the following without reading runtime code:

- can the reviewer state the three required live branches
- can the reviewer explain which branch is tool-loop continuation and why it is distinct from normal think/planner execution
- can the reviewer identify the minimum redacted evidence required for pass and fail cases
- can the reviewer explain which evidence is required and which is optional
- can the reviewer confirm the contract preserves the `C-05` boundary and does not expose planner/executor identity as public truth
- can the reviewer confirm the contract stays grounded in real Claude Code sessions above the landed `C-09` bootstrap path

## Drift Guards

Downstream revalidation is required if any of the following changes:

- the live smoke branch set changes enough that the three-branch story is no longer accurate
- routing, continuation, or evidence-hook behavior changes enough that the minimum redacted evidence posture is no longer accurate
- `C-09`, `C-08`, `C-04`, or `C-05` change in a way that affects live-smoke truth
- route labels, statusline semantics, or trace-hook behavior change enough that downstream troubleshooting would need to rediscover proof from runtime code

## Compatibility Notes

- This contract is compatible with the landed `C-03` public surface and does not redefine `/v1/messages`.
- This contract is compatible with the landed `C-04` policy boundary and does not expose planner/executor roles.
- This contract is compatible with the landed `C-05` boundary and does not leak provider or deployment identity as public truth.
- This note is intentionally capability-oriented and must not be read as a public declaration of provider identity or deployment topology.
