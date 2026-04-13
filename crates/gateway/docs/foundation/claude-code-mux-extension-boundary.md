# Claude Code Mux Extension Boundary

## Purpose

This note defines the seam-local extension boundary for `SEAM-1` so downstream seams can target a single, explicit contract instead of inferring provider or API adapter hooks.
It is the boundary-map portion of `C-01` for `THR-01`.

The boundary remains intentionally narrow and architectural:

- it does not implement Azure Kimi parsing
- it does not implement the public Anthropic Messages surface
- it does not implement planner/executor orchestration

## Landed Tree And Identity

The landed foundation is the repo-local `gateway/` tree, built as `substrate-gateway`.

The repo-local identity is already visible in the landed tree:

- crate and binary identity: `gateway/Cargo.toml` sets `package.name = "substrate-gateway"` and the `substrate-gateway` binary
- CLI identity and help text: `gateway/src/main.rs` and `gateway/src/cli/mod.rs`
- runtime/config identity: `gateway/src/cli/mod.rs` uses `~/.substrate-gateway/` paths
- provider boundary anchor: `gateway/src/providers/mod.rs` and `gateway/src/providers/openai.rs`
- client-surface anchor: `gateway/src/server/mod.rs` and `gateway/src/server/openai_compat.rs`
- internal-policy anchor: `gateway/src/router/mod.rs`

These are contract anchors, not instructions to rewrite the landed implementation here.

## Boundary Map

The foundation is divided into three contract zones:

1. Provider normalization boundary
2. Client-surface boundary
3. Internal policy boundary

### Provider Normalization Boundary

This is the hook where Azure normalization will attach later.

Required properties:

- Azure-specific parsing stays behind this boundary
- hidden tool markers remain implementation details inside the provider seam
- downstream code does not depend on sentinel syntax or raw provider chunk shape
- the boundary must stay compatible with the provider trait and registry model already present in `gateway/src/providers/mod.rs`
- the provider seam must remain separate from router policy decisions in `gateway/src/router/mod.rs`

### Client-Surface Boundary

This is the hook where Anthropic Messages remains the first ingress contract.

Required properties:

- Anthropic Messages is the first external delivery shape
- the gateway core does not become Anthropic-only at its internal boundary
- future OpenAI Responses support remains a thin outer adapter seam, not a second engine
- the surface should map to the compatibility server ingress family already visible in `gateway/src/server/mod.rs` and `gateway/src/server/openai_compat.rs`
- the client surface must remain distinct from provider parsing and internal routing policy

### Internal Policy Boundary

This is the hook where planner/executor routing stays above provider normalization.

Required properties:

- orchestration policy remains internal
- model-role selection is not exposed as a public backend identity
- provider parsing must not make policy decisions
- the policy seam is anchored in `gateway/src/router/mod.rs`
- configuration and runtime selection surfaces are allowed only as internal inputs, not as external backend ids

## Contract Rules

The extension boundary must preserve these invariants:

- one logical backend identity for the gateway
- local transport is a replaceable development convenience, not the architectural contract
- normalized events are the internal handoff format, not raw provider streams
- Azure normalization and planner/executor policy remain internal implementation concerns
- Azure provider parsing must normalize explicit `tool_calls` and hidden `reasoning_content` markers without leaking sentinel syntax downstream
- Anthropic Messages stays the first external surface, while OpenAI Responses remains a later thin adapter seam
- no public config or API surface may require Substrate to choose between internal planner, executor, or provider roles
- the core engine must remain separable from loopback-only or host-credential assumptions
- shell, REPL, and downstream integrations must consume normalized structured events rather than raw provider frames

## Downstream Stale Triggers

Any of the following should force downstream revalidation instead of silent reuse:

- `SEAM-2` if the provider hook is moved, renamed, or described in a way that stops Azure normalization from attaching cleanly behind `gateway/src/providers/mod.rs`
- `SEAM-3` if this note starts implying Anthropic-only core types or raw provider-frame coupling at the client surface
- `SEAM-4` if this note exposes planner/executor selection, model-role labels, or other internal policy decisions as public backend identity
- `SEAM-5` if this note turns localhost, host credential access, or multiple externally selectable gateway identities into architectural assumptions

## Downstream Use

This note is the source of truth for later seam planning. `SEAM-2` should be able to point to the provider boundary here, `SEAM-3` to the client-surface boundary, and `SEAM-4` to the internal policy boundary without inventing new terminology.
`SEAM-5` should inherit only the generic deployment and identity constraints, not a provider- or policy-specific contract.
