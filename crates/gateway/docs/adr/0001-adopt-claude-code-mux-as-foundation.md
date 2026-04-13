# ADR 0001: Adopt `claude-code-mux` as the Gateway Foundation

- Status: Proposed
- Date: 2026-03-27

## Context

This project needs a new home for a local proxy/adapter/gateway that makes Azure-hosted Kimi models usable with Claude Code and later with additional clients.

The earlier local Node adapter under `~/.codex/kimi-chat-adapter` proved the Azure connectivity path, but it is the wrong foundation for the next phase:

- It is a thin request/response shim rather than a stateful gateway.
- It is optimized for OpenAI Responses rather than Anthropic Messages.
- It does not preserve multi-turn reasoning state.
- It does not provide a clean Rust path for a future general-purpose router.

The selected candidate foundation is the Rust project [`elidickinson/claude-code-mux`](https://github.com/elidickinson/claude-code-mux), which is positioned as an Anthropic-compatible proxy for Claude Code with routing, translation, streaming, and provider abstractions. A referenced upstream commit, [`5a372fb`](https://github.com/9j/claude-code-mux/commit/5a372fbff301ddc6d2fb8b942f672ad0346acbb3), claims a Kimi tool-call fix, but that claim still requires validation against the Azure Foundry Kimi behavior observed in prior debugging.

## Decision

Use `claude-code-mux` as the implementation foundation for this repository.

This repository becomes the primary home for the new Rust gateway. The old Node adapter remains reference material only and is not the main implementation path.

The adoption sequence is:

1. Download the archived `claude-code-mux` codebase into this repository as the primary starting codebase.
2. Build, run, and stabilize it close to its baseline behavior so the repo proves the foundation actually works before deeper changes begin.
3. Rename project-identity surfaces needed to disconnect from the old project naming, including the crate/package identity, binary/config naming, and repo-local documentation labels.
4. Start Azure Kimi normalization, gateway-surface, and downstream integration changes on top of that stabilized renamed baseline.

Because the source repository is archived, this is not a fork with an expected upstream sync path. We should keep the code structurally close to the adopted baseline during the stabilization step, then perform targeted identity renames before feature modifications expand.

## Consequences

Positive:

- We start from a codebase that already targets Claude Code’s Anthropic-facing contract.
- We move directly onto the Rust foundation desired for future router work.
- We inherit existing routing, SSE, and provider abstraction patterns instead of rebuilding them from scratch.

Negative:

- We take on the complexity and design assumptions of an external codebase.
- We must verify the claimed Kimi fixes ourselves, especially for Azure Foundry behavior rather than only native Moonshot behavior.
- We may still need substantial Azure-specific provider work after adoption.
- We must be disciplined about sequence so identity renames and feature work do not get mixed into the initial baseline-stabilization pass.

## Deliverable Boundary

This ADR is complete when:

1. The archived `claude-code-mux` codebase is downloaded and established as the local foundation in this repository.
2. The adopted baseline builds and reaches a minimal runnable or smoke-tested baseline state before project-identity renames begin.
3. The repo-local identity pass renames the adopted codebase to the new project naming, including the crate/package identity (`substrate-gateway`) and repo-local gateway/config labels.
4. A short verification note records whether commit `5a372fb` addresses any part of the observed Kimi failure mode and what remains unresolved.
