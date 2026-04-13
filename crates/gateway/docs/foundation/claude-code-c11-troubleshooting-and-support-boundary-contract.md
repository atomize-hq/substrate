# Claude Code Troubleshooting And Support Boundary `C-11` Contract

## Purpose

This note is the canonical landing artifact for `C-11`.
It freezes the operator troubleshooting and support-boundary contract for the Claude Code live integration smoke path while keeping the public gateway identity capability-oriented and keeping provider, planner, executor, and deployment details internal.

This contract is intentionally narrow:

- it defines the troubleshooting ownership matrix for the live integration path
- it defines the evidence review order operators use before escalating a failure
- it defines the minimum redacted evidence posture for support-oriented triage
- it defines one stable landing path for this troubleshooting truth

It does not define:

- runtime transport construction
- public `/v1/messages` semantics beyond the landed public surface
- planner/executor routing policy
- operator support guide delivery, closeout publication, or broader incident-response workflow

## Canonical Sources

This contract is grounded in the landed basis and seam-local planning below:

- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-3-troubleshooting-and-support-boundary/seam.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-3-troubleshooting-and-support-boundary/review.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-3-troubleshooting-and-support-boundary/slice-1-freeze-troubleshooting-boundary-contract-and-taxonomy.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threading.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/governance/seam-1-closeout.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/governance/seam-2-closeout.md`
- `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`
- `docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md`
- `docs/foundation/claude-code-c10-live-session-smoke-procedure.md`
- `docs/foundation/azure-foundry-c08-operator-verification-contract.md`
- `docs/foundation/substrate-boundary-c05-contract.md`

Repo-grounded runtime and documentation anchors:

- `gateway/README.md`
- `gateway/src/router/mod.rs`
- `gateway/src/server/mod.rs`

If this note and those anchors disagree, the note or the upstream evidence must be revalidated before downstream implementation proceeds.

## Public Boundary

`C-11` must preserve the `C-05` boundary.

Required behavior:

- public docs and examples stay capability-oriented
- public docs and examples do not require operators to choose planner, executor, or provider identities
- the gateway may remain one logical backend capability to the outside world
- Azure deployment names remain internal mapping data, not public product identity
- `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets used to explain verification and troubleshooting coverage, not public backend identities or public model labels

Boundary statements:

- localhost and `127.0.0.1` are valid default development conveniences
- loopback is not the architectural contract
- the public troubleshooting story must not force operators to learn provider-specific transport mechanics before they can classify the failure

## Troubleshooting Ownership Matrix

The canonical troubleshooting boundary separates failures into four operator-facing ownership branches.

### Claude Code Setup

Use when the failure is in the local Claude Code bootstrap path, environment attachment, or statusline/setup posture before the live smoke branch can be exercised.

Typical signals:

- `ANTHROPIC_BASE_URL` is missing, malformed, or not pointed at the gateway
- `ANTHROPIC_API_KEY` placeholder posture is not present for the local launch path
- the statusline script is not installed or is not reading `~/.substrate-gateway/last_routing.json`
- the operator cannot start a real Claude Code session against the public gateway ingress

### Gateway Runtime / Config

Use when the failure is in gateway startup, config loading, model mapping, route selection, or evidence-hook behavior inside the gateway boundary.

Typical signals:

- the gateway does not start from the documented config path
- routing decisions do not match the landed `C-09` and `C-10` truth
- `last_routing.json` does not reflect the branch the operator exercised
- the handler returns a route-selection or transport-drift error that maps to gateway behavior rather than upstream auth or deployment rejection

### Azure Transport

Use when the failure is in Azure authentication, Azure endpoint targeting, deployment mapping, or provider-backed request execution.

Typical signals:

- credentials are missing or rejected
- the base URL or provider endpoint is wrong
- the routed model does not resolve to the intended internal deployment target
- the upstream provider returns an auth, URL, or deployment failure that is external to the gateway’s own route-selection logic

### Broader Drift

Use when the observed behavior no longer matches the landed bootstrap, smoke, or evidence contract even though the immediate failure is not clearly attributable to setup, gateway config, or Azure transport.

Typical signals:

- the operator-visible evidence order no longer matches the contract
- the redaction posture leaks or hides too much information
- the live behavior changes enough that the failure can no longer be classified using the landed ownership matrix
- runtime anchors drift in a way that invalidates the current review order

### Decision Rule

When a failure crosses more than one branch, classify it by the first operator-visible break in the evidence chain:

1. Claude Code setup
2. Gateway runtime / config
3. Azure transport
4. Broader drift

This order keeps troubleshooting aligned to the first broken control point rather than collapsing distinct responsibilities into one support bucket.

## Evidence Review Order

The canonical review order is:

1. Reconfirm the bootstrap contract in `C-09` and the bootstrap checklist in `gateway/README.md`.
2. Reconfirm the live smoke contract in `C-10` and the live-session procedure.
3. Inspect the routing-history and statusline evidence surfaces: the installed Claude Code statusline output and `~/.substrate-gateway/last_routing.json`.
4. Inspect the branch identity exercised and the redacted session outcome for the failing run.
5. Inspect `~/.substrate-gateway/trace.jsonl` only when the required evidence does not explain the failure class.

Required evidence:

- the bootstrap posture that led to the live run
- the branch identity exercised: normal execution, think / planner execution, or tool-loop continuation
- the installed Claude Code statusline output
- `~/.substrate-gateway/last_routing.json`
- the redacted session outcome for the run

Optional evidence:

- `~/.substrate-gateway/trace.jsonl` when additional redacted debugging evidence is needed
- extra redacted request or response context when it materially improves classification without exposing internal identity

Review guidance:

- statusline and routing-history evidence are the primary support surface
- tracing is secondary and should not become the default troubleshooting prerequisite
- the review order must stay consistent with the landed bootstrap and smoke contracts

## Redaction Rules

The redaction model is capability-oriented, not forensic.

Required rules:

- do not publish credentials
- do not publish bearer tokens or API keys
- do not expose planner/executor role selection as public identity
- do not expose Azure deployment mechanics as public backend names
- do not expand redaction so far that the operator can no longer classify the failure

Allowed evidence includes redacted mentions of:

- the public `ANTHROPIC_BASE_URL` bootstrap path
- the public `/v1/messages` usage path
- the branch identity exercised
- the failure class and the minimal surrounding context needed to diagnose it

## Failure-Class Taxonomy

Troubleshooting failures should be classified using the operator-facing classes aligned with `C-08`, `C-09`, and the server error surface:

### Auth

Use when Azure credentials are missing, malformed, or rejected.

### URL

Use when the base URL, host variant, or request target is malformed or points at the wrong endpoint.

### Deployment

Use when the think/default mapping does not resolve to the intended internal Azure deployment target.

### Route

Use when the gateway selected the wrong internal route for think, default, or continuation traffic even though the transport itself is otherwise valid.

### Transport Drift

Use when the observed behavior no longer matches the landed transport, bootstrap, or evidence contract, including request-target assumptions, evidence-hook behavior, or redaction posture drift that no longer matches the operator path.

Required boundary statement:

- these categories are operator-facing diagnostics, not public backend identities
- the taxonomy must stay specific enough to guide remediation without leaking internal implementation detail

## Artifact Path And Verification Checklist

The single canonical landing path for this contract is:

- `docs/foundation/claude-code-c11-troubleshooting-and-support-boundary-contract.md`

`C-11` is complete only if a reviewer can answer yes to all of the following without reading runtime code:

- can the reviewer state the troubleshooting ownership matrix from the contract alone
- can the reviewer explain the evidence review order from bootstrap through optional tracing
- can the reviewer identify the minimum redacted evidence required before escalation
- can the reviewer classify failures into auth, URL, deployment, route, or transport-drift classes
- can the reviewer confirm the contract preserves the `C-05` boundary and does not expose planner/executor identity as public truth
- can the reviewer confirm the contract stays grounded in the landed `C-09` and `C-10` bootstrap and live-smoke truth

## Drift Guards

Downstream revalidation is required if any of the following changes:

- the bootstrap sequence changes enough that the operator path no longer matches `C-09`
- the live smoke contract or procedure changes enough that the evidence review order is no longer accurate
- routing, continuation, or evidence-hook behavior changes enough that `last_routing.json`, the statusline script, or optional tracing no longer match the operator path
- `C-08`, `C-09`, `C-10`, or `C-05` change in a way that affects troubleshooting truth
- runtime anchors in `gateway/README.md`, `gateway/src/router/mod.rs`, or `gateway/src/server/mod.rs` drift enough that the current ownership matrix no longer matches real operator behavior

## Compatibility Notes

- This contract is compatible with the landed `C-03` public surface and does not redefine `/v1/messages`.
- This contract is compatible with the landed `C-04` policy boundary and does not expose planner/executor roles.
- This contract is compatible with the landed `C-05` boundary and does not leak provider or deployment identity as public truth.
- This contract is compatible with the landed `C-08`, `C-09`, and `C-10` evidence posture and remains capability-oriented rather than support-ops specific.
- This note is intentionally narrow and must not be read as a public declaration of provider identity or deployment topology.
