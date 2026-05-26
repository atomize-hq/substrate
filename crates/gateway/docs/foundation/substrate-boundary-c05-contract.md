# Substrate Boundary `C-05` Contract

## Purpose

This note is the canonical landing artifact for `C-05`.
It defines the public gateway identity and deployment boundary for the Substrate-facing seam while keeping planner/executor/provider details internal.

This contract is intentionally narrow:

- it defines one logical backend identity for the gateway
- it defines the public naming rules that must stay capability-oriented
- it defines the deployment/auth boundary as a replaceable outer-layer concern, not a core contract
- it preserves localhost loopback as a development convenience only

It does not define:

- provider parsing
- downstream structured-event semantics
- planner/executor routing policy
- any Substrate integration behavior beyond the public boundary language

## Canonical Source Of Truth

This contract is grounded in the seam-local boundary note and upstream contracts:

- `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/threaded-seams/seam-5-substrate-compatible-boundary/seam.md`
- `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/threaded-seams/seam-5-substrate-compatible-boundary/review.md`
- `docs/foundation/claude-code-mux-extension-boundary.md`
- `docs/foundation/anthropic-messages-c03-contract.md`
- `docs/foundation/planner-executor-c04-policy-contract.md`
- `docs/adr/0005-present-a-single-backend-identity-to-substrate.md`
- `docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md`

If this note and those anchors disagree, the note or the upstream seam evidence must be revalidated before downstream use.

## Public Boundary

The gateway must present one stable external backend identity.

Required behavior:

- public docs and examples stay capability-oriented
- public docs and examples do not require Substrate to choose between planner, executor, or provider roles
- the binary/package identity may remain `substrate-gateway`
- internal routing and provider details stay behind the public boundary

Boundary statements:

- localhost and `127.0.0.1` are valid default development conveniences
- loopback is not the architectural contract
- auth, transport, and credential delivery must remain factored so a later in-world boundary can wrap the gateway without renaming the public capability
- public docs must not imply that host-local access is the only supported deployment posture

## Runtime And Config Anchors

These are the repo surfaces that currently express the boundary and therefore need to remain consistent with this contract:

- `gateway/src/main.rs`
- `gateway/src/cli/mod.rs`
- `gateway/src/server/mod.rs`
- `gateway/README.md`
- `gateway/config/default.example.toml`

This note is the source of truth for how those anchors should be read.

## Drift Guards

Downstream revalidation is required if any of the following changes:

- planner/executor/provider naming leaks into the public backend identity
- localhost-only wording starts describing the architectural boundary instead of a default dev posture
- host-only credential or transport assumptions move into the core contract
- public examples require readers to reason about internal roles before they can use the gateway

## Verification Checklist

`C-05` is complete only if a reviewer can answer yes to all of the following without reading runtime code:

- can the public identity be described as one logical backend capability
- can the deployment boundary be explained as replaceable and not loopback-only
- do public docs and config examples avoid planner/executor/provider identity leakage
- are localhost and `127.0.0.1` clearly framed as development conveniences
- do the drift guards make identity and deployment regressions explicit
