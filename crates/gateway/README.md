# Substrate Gateway

Substrate Gateway is the stable public runtime boundary for model-backed agent traffic in this repo.

It presents one logical backend capability to external consumers while keeping provider normalization, routing policy, deployment details, and backend-specific transport concerns behind the boundary.

## What It Is

The gateway sits between client-surface adapters and backend adapters:

```text
client -> client-surface adapter -> substrate-gateway core -> backend adapter
```

In practice, that means:

- external consumers talk to one gateway identity
- multiple ingress shapes can terminate at the same core
- public model names stay capability-oriented
- backend adapters own provider-specific transport and parsing details
- the core owns routing, normalization, failover, and boundary enforcement
- downstream integrations consume normalized semantics rather than raw provider streams

## Boundary And Ownership

The intended shape is already captured in the repo’s boundary contracts.

The gateway owns:

- one stable external backend identity
- public client-surface contracts
- capability-oriented routing and model-label policy
- normalized internal semantics for downstream consumers
- keeping deployment and auth concerns as replaceable outer layers

Backend adapters own:

- provider-specific request construction
- provider auth/header mechanics
- transport quirks and response parsing
- translating raw provider behavior into normalized gateway semantics

Client surfaces own:

- public request/response compatibility for supported API shapes
- thin adaptation over the same internal gateway core
- preserving the gateway boundary instead of becoming separate engines

Non-negotiable rules:

1. Substrate should see one logical backend capability, not planner/executor/provider identities.
2. `127.0.0.1` is a development convenience, not the architecture contract.
3. Backend-specific parsing stays behind the adapter boundary.
4. Shell, REPL, and downstream consumers should rely on normalized structured events, not raw provider frames.

## Contract Anchors

These docs are the clearest source of truth for the README’s framing:

- [IMPORTANT_SUBSTRATE_ALIGNMENT.md](docs/IMPORTANT_SUBSTRATE_ALIGNMENT.md)
- [Claude Code Mux Extension Boundary](docs/foundation/claude-code-mux-extension-boundary.md)
- [Substrate Boundary `C-05`](docs/foundation/substrate-boundary-c05-contract.md)
- [Substrate Structured Events `C-06`](docs/foundation/substrate-structured-events-c06-contract.md)
- [ADR 0005: Single backend identity](docs/adr/0005-present-a-single-backend-identity-to-substrate.md)
- [ADR 0006: In-world-compatible deployment boundary](docs/adr/0006-preserve-an-in-world-compatible-deployment-boundary.md)
- [ADR 0007: Normalized structured events](docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md)

## Public Surfaces

The gateway currently exposes these public client-surface adapters:

- `POST /v1/messages`
- `POST /v1/messages/count_tokens`
- `POST /v1/chat/completions`
- `POST /v1/responses`

Current boundary posture:

- `POST /v1/messages` is the primary Claude Code-facing adapter
- `POST /v1/chat/completions` and `POST /v1/responses` are parallel public adapters over the same core
- `POST /v1/messages/count_tokens` is a utility surface on the same gateway boundary
- public routes should not expose backend-role truth or provider-specific transport behavior

More detail:

- [OpenAI compatibility notes](docs/openai-compatibility.md)

## Runtime Model

The runtime is best understood as three zones:

1. Provider normalization boundary
2. Client-surface boundary
3. Internal policy boundary

### Provider Normalization Boundary

This is where backend adapters translate provider-specific transport into gateway semantics.

Required properties:

- provider quirks remain internal
- raw chunk shapes remain internal
- hidden marker syntax remains internal
- backend parsing does not become public contract truth

### Client-Surface Boundary

This is where public API compatibility is delivered.

Required properties:

- client-surface adapters stay thin over the same core
- the core does not become locked to one external API family
- additional public surfaces remain adapters, not independent engines

### Internal Policy Boundary

This is where routing and orchestration policy live.

Required properties:

- route selection stays internal
- planner/executor distinctions do not become public backend ids
- backend adapters do not make public policy decisions

## Structured Events

The gateway’s downstream contract is normalized semantics, not transport bytes.

Downstream consumers should be able to depend on stable structured meanings such as:

- tool intent
- action/progress
- final outcome

They should not need to reason about:

- raw SSE framing
- provider chunk ordering
- hidden marker syntax
- which backend-specific stream artifact produced a normalized event

The contract note for that boundary is:

- [Substrate Structured Events `C-06`](docs/foundation/substrate-structured-events-c06-contract.md)

## Quick Start

This is the generic local operator bootstrap for the gateway itself.

### 1. Build

From the repo root:

```bash
cargo build -p substrate-gateway --release
```

Binary output:

```text
target/release/substrate-gateway
```

### 2. Create config

Copy the example into the default runtime location:

```bash
mkdir -p ~/.substrate-gateway
cp crates/gateway/config/default.example.toml ~/.substrate-gateway/config.toml
```

Notes:

- `~/.substrate-gateway/config.toml` is the default local config path
- the checked-in example is a local bootstrap artifact, not the final deployment contract
- public model labels should stay capability-oriented even when backend mappings are concrete

### 3. Start the gateway

```bash
./target/release/substrate-gateway start
```

Default local address:

```text
http://127.0.0.1:13456
```

That loopback host is a development default only.

### 4. Install the Claude Code statusline

```bash
./target/release/substrate-gateway install-statusline
```

This installs `~/.substrate-gateway/statusline.sh`, which reads `~/.substrate-gateway/last_routing.json`.

### 5. Optional: enable tracing

Enable tracing only when you need redacted debugging evidence:

```toml
[server.tracing]
enabled = true
path = "~/.substrate-gateway/trace.jsonl"
omit_system_prompt = true
```

### 6. Point Claude Code at the gateway

```bash
export ANTHROPIC_BASE_URL="http://127.0.0.1:13456"
export ANTHROPIC_API_KEY="any-string"
claude
```

The placeholder API key is acceptable for the local bootstrap path because real backend auth lives behind the gateway boundary.

That Claude Code step exercises the Anthropic-shaped client-surface adapter. Other clients can enter through the OpenAI-shaped adapters while still hitting the same gateway core.

## Routing And Model Labels

Routing is internal gateway policy.

The README should describe the external posture like this:

- callers address one gateway capability
- public model names are gateway-facing labels
- backend-specific deployment names stay internal mapping data
- routing may choose different internal paths for different turn types without exposing those paths as public backend identities

Current routing behavior includes:

- tool-aware routing
- background-task routing
- subagent-model override support
- prompt-rule routing
- plan-mode routing
- tool-result continuation handoff
- default routing with backend fallback

Those are runtime behaviors of the gateway, not separate products or public backends.

## Evidence Surfaces

The operator-visible evidence surfaces for local verification are:

- `~/.substrate-gateway/last_routing.json`
- `~/.substrate-gateway/statusline.sh`
- `~/.substrate-gateway/trace.jsonl` when tracing is enabled

These exist to support debugging and smoke verification without turning internal backend details into public truth.

Related docs:

- [Claude Code bootstrap contract (`C-09`)](docs/foundation/claude-code-c09-operator-bootstrap-contract.md)
- [Claude Code live smoke contract (`C-10`)](docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md)
- [Claude Code troubleshooting guide](docs/foundation/claude-code-c11-operator-troubleshooting-guide.md)

## Development Conveniences

Useful local conveniences:

- local admin UI at `http://127.0.0.1:13456`
- local config under `~/.substrate-gateway/`
- statusline and routing-history files under `~/.substrate-gateway/`

These are convenience surfaces for development and smoke, not architectural commitments.

## Additional Docs

- [OpenAI compatibility](docs/openai-compatibility.md)
- [OAuth setup](docs/OAUTH_SETUP.md)
- [OAuth testing](docs/OAUTH_TESTING.md)
- [Architecture decisions](docs/adr)
- [Foundation contracts](docs/foundation)

## Changelog

- [CHANGELOG.md](CHANGELOG.md)

## License

MIT
