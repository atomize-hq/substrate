# Unified Agent API

Agent-agnostic facade and backend registry for the `unified-agent-api` workspace.

- crates.io package: `unified-agent-api`
- Rust library crate: `agent_api`

This crate presents one Rust-facing surface across the repo's Codex and Claude
Code backends while keeping backend-specific types out of the public API.

## Feature flags

- `codex`: enable the Codex backend integration.
- `claude_code`: enable the Claude Code backend integration.

## Quickstart

```rust,no_run
use agent_api::{AgentWrapperGateway, AgentWrapperKind};

let gateway = AgentWrapperGateway::new();
let codex = AgentWrapperKind::new("codex").unwrap();
assert!(gateway.backend(&codex).is_none());
```

See the repository contracts under `docs/specs/unified-agent-api/` for the
canonical behavior and compatibility expectations.
