# Claude Code Operator Bootstrap `C-09` Contract

## Purpose

This note is the canonical landing artifact for `C-09`.
It freezes the operator bootstrap contract for the Claude Code live integration smoke path while keeping the public gateway identity capability-oriented and keeping provider, planner, executor, and deployment details internal.

This contract is intentionally narrow:

- it defines the canonical bootstrap sequence from Azure prerequisites through gateway config, startup validation, evidence hooks, and Claude Code attachment/launch
- it defines the minimum pre-smoke evidence posture that downstream smoke work can rely on
- it defines the redaction and failure-class rules operators use before live smoke begins
- it defines one stable landing path for this bootstrap truth

It does not define:

- runtime transport construction
- public `/v1/messages` semantics beyond the landed public surface
- planner/executor routing policy
- live smoke execution or troubleshooting ownership beyond the bootstrap boundary

## Canonical Sources

This contract is grounded in the landed basis and seam-local planning below:

- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-1-claude-code-operator-bootstrap/seam.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-1-claude-code-operator-bootstrap/review.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-1-claude-code-operator-bootstrap/slice-1-freeze-claude-code-bootstrap-contract.md`
- `docs/project_management/packs/active/claude-code-live-integration-smoke/threading.md`
- `docs/foundation/azure-foundry-c07-runtime-transport-contract.md`
- `docs/foundation/azure-foundry-c08-operator-verification-contract.md`
- `docs/foundation/anthropic-messages-c03-contract.md`
- `docs/foundation/planner-executor-c04-policy-contract.md`
- `docs/foundation/substrate-boundary-c05-contract.md`

Repo-grounded runtime and documentation anchors:

- `gateway/README.md`
- `gateway/config/default.example.toml`
- `gateway/config/models.example.toml`
- `gateway/src/cli/mod.rs`
- `gateway/src/main.rs`
- `gateway/src/server/mod.rs`

If this note and those anchors disagree, the note or the upstream evidence must be revalidated before downstream implementation proceeds.

## Public Boundary

`C-09` must preserve the `C-05` boundary.

Required behavior:

- public docs and examples stay capability-oriented
- public docs and examples do not require operators to choose planner, executor, or provider identities
- the gateway may remain one logical backend capability to the outside world
- Azure deployment names remain internal mapping data, not public product identity
- `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets used to explain verification and bootstrap coverage, not public backend identities or public model labels

Boundary statements:

- localhost and `127.0.0.1` are valid default development conveniences
- loopback is not the architectural contract
- the public bootstrap story must not force operators to learn provider-specific transport mechanics before they can launch the gateway

## Canonical Bootstrap Sequence

The canonical operator sequence is:

1. satisfy Azure prerequisites
2. prepare gateway config and model mappings
3. start the gateway and validate startup
4. enable or inspect statusline and tracing evidence hooks
5. attach and launch Claude Code against the gateway

### 1. Azure Prerequisites

Required posture:

- Azure credentials are available to the gateway in the configured provider mode
- the Azure Foundry / Azure OpenAI v1 base URL is configured on the provider surface
- the routed model names resolve to the intended internal Azure deployment names through config mapping
- the operator can distinguish the think path from the default path without exposing provider identity as public truth

What this section must not do:

- redefine `C-07`
- turn deployment names into public backend names
- require provider parsing knowledge from the operator

### 2. Gateway Config

Required posture:

- the config file path is the default gateway config location or an explicitly supplied equivalent
- the operator can express provider mode, base URL, auth posture, and model mappings through config and examples
- `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets used in smoke-oriented operator guidance
- `actual_model` remains the internal Azure deployment name carried behind the provider boundary

Evidence anchors:

- `gateway/config/default.example.toml` shows the default Azure Kimi bootstrap posture
- `gateway/config/models.example.toml` shows the internal mapping from public model names to Azure deployment names
- `gateway/src/cli/mod.rs` and `gateway/src/main.rs` show the default config path and config loading behavior

### 3. Startup Validation

Required posture:

- the gateway starts from the resolved config and validates the operator path before a live session
- the operator can launch the server with the documented CLI flow
- startup output and config loading make it clear that the loopback host is a development default, not architectural identity

Evidence anchors:

- `gateway/src/main.rs` defines the `start`, `restart`, `status`, `model`, and `install-statusline` CLI surfaces
- `gateway/README.md` documents `substrate-gateway start` and the default config location
- `gateway/src/cli/mod.rs` shows the default config creation and loopback-default phrasing

### 4. Statusline And Tracing Evidence Hooks

Required posture:

- `install-statusline` is the canonical helper surface for the Claude Code statusline script
- `last_routing.json` is the operator-visible routing summary used by the statusline script
- `trace.jsonl` is the optional message-tracing path for redacted debugging evidence
- statusline and tracing are evidence hooks for smoke preparation, not replacements for the public `/v1/messages` contract

Evidence anchors:

- `gateway/src/main.rs` installs the Claude Code statusline script
- `gateway/src/server/mod.rs` writes `~/.substrate-gateway/last_routing.json`
- `gateway/src/cli/mod.rs` and `gateway/README.md` define the default tracing path `~/.substrate-gateway/trace.jsonl`

### 5. Claude Code Attachment

Required posture:

- Claude Code attaches through `ANTHROPIC_BASE_URL="http://127.0.0.1:13456"`
- a placeholder API key is acceptable for local bootstrap posture
- the operator launches Claude Code against the gateway path, not a provider-only side channel

Evidence anchors:

- `gateway/README.md` documents the Claude Code environment flow
- `docs/foundation/anthropic-messages-c03-contract.md` defines the public `/v1/messages` surface that Claude Code consumes
- `docs/foundation/substrate-boundary-c05-contract.md` keeps loopback and host-local access as a development convenience, not the architectural boundary

## Minimum Pre-Smoke Evidence Posture

An operator run is not ready for live smoke unless the bootstrap evidence set can show:

- the gateway was configured with the expected Azure provider and model mapping posture
- Claude Code was pointed at the gateway via `ANTHROPIC_BASE_URL`
- statusline output or routing history is available through `last_routing.json`
- tracing is available when enabled through `trace.jsonl`
- the operator can explain the chosen think and default paths without reading runtime code

This contract treats the evidence as minimum viable operator proof, not as a request for exhaustive forensics.

## Redaction Rules

The redaction model is capability-oriented, not forensic.

Required rules:

- do not publish credentials
- do not publish bearer tokens, API keys, or other secret material
- do not expose planner/executor role selection as public identity
- do not expose Azure deployment mechanics as public backend names
- do not expand redaction so far that the operator can no longer classify the failure

Allowed evidence includes redacted mentions of:

- the public `ANTHROPIC_BASE_URL` bootstrap path
- the public `/v1/messages` usage path
- the routed model label needed to explain which bootstrap branch was exercised
- the failure class and the minimal surrounding context needed to diagnose it

## Failure-Class Taxonomy

Bootstrap failures should be classified using the operator-facing classes aligned with `C-08`:

### Auth

Use when Azure credentials are missing, malformed, or rejected.

### URL

Use when the base URL, host variant, or request target is malformed or points at the wrong endpoint.

### Deployment

Use when the think/default model mapping does not resolve to the intended internal Azure deployment target.

### Route

Use when the gateway selected the wrong internal route for think or default traffic even though the transport is otherwise valid.

### Transport Drift

Use when the observed behavior no longer matches the landed transport or bootstrap contract, including request-target assumptions, redaction posture drift, or evidence-hook behavior that no longer matches the operator path.

Required boundary statement:

- these categories are operator-facing diagnostics, not public backend identities
- the taxonomy must stay specific enough to guide remediation without leaking internal implementation detail

## Artifact Path And Verification Checklist

The single canonical landing path for this contract is:

- `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`

`C-09` is complete only if a reviewer can answer yes to all of the following without reading runtime code:

- can the reviewer state the bootstrap sequence from Azure prerequisites through statusline/tracing evidence hooks
- can the reviewer explain which config and model-mapping surfaces are required before launching Claude Code
- can the reviewer explain how `Kimi-K2-Thinking` and `Kimi-K2.5` remain internal routing targets while the operator path stays capability-oriented
- can the reviewer identify the minimum redacted evidence required before live smoke begins
- can the reviewer classify bootstrap failures into auth, URL, deployment, route, or transport-drift classes
- can the reviewer confirm the contract preserves the `C-05` boundary and does not expose planner/executor identity as public truth

## Drift Guards

Downstream revalidation is required if any of the following changes:

- the bootstrap sequence changes enough that the operator path no longer matches this contract
- config or example surfaces can no longer express the Azure provider posture and internal model mapping cleanly
- statusline, routing history, or tracing evidence surfaces change enough that the minimum pre-smoke evidence posture is no longer accurate
- `C-07`, `C-08`, `C-03`, `C-04`, or `C-05` change in a way that affects bootstrap truth
- loopback or placeholder-auth language starts describing architectural truth instead of development convenience

## Compatibility Notes

- This contract is compatible with the landed `C-03` public surface and does not redefine `/v1/messages`.
- This contract is compatible with the landed `C-04` policy boundary and does not expose planner/executor roles.
- This contract is compatible with the landed `C-05` boundary and does not leak provider or deployment identity as public truth.
- This note is intentionally capability-oriented and must not be read as a public declaration of provider identity or deployment topology.
