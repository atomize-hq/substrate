# LLM Last Mile SOW Packet

This directory contains the reviewed statement-of-work packet for the prerequisites that should be completed before the planned live registry step:

- planned registry target:
  - `~/.substrate/run/agent-sessions/<orchestration_session_id>.json`
  - registry wraps UAA handles with Substrate-owned metadata
  - `trace.jsonl` remains audit history, not live state authority

## Reviewed Execution Order

1. [01-orchestration-session-identity.md](./01-orchestration-session-identity.md)
   - Replace the process-global orchestration identity with a real Substrate-owned session identity.
2. [02-session-participant-record.md](./02-session-participant-record.md)
   - Generalize runtime state from a host-orchestrator-only manifest to a participant record that can represent orchestrator and member sessions.
3. [03-shared-world-ownership-linux-first.md](./03-shared-world-ownership-linux-first.md)
   - Make shared-world ownership explicit on Linux and bind one active `world_id` to one `orchestration_session_id`.
4. [04-thread-world-binding-into-runtime-state.md](./04-thread-world-binding-into-runtime-state.md)
   - Persist authoritative `world_id` and `world_generation` in runtime state instead of relying on trace/event history.
5. [05-restart-invalidation-semantics.md](./05-restart-invalidation-semantics.md)
   - Define the live-state rule that a generation change invalidates all prior-generation world-scoped member sessions.
6. [06-session-centric-state-store.md](./06-session-centric-state-store.md)
   - Rework the runtime handle store APIs and layout so consumers resolve by orchestration session instead of by orchestrator agent heuristics.

## Review Notes

- The packet is intentionally Linux-first for shared-world ownership.
- The packet does not require new inventory `uaa_kind`-style schema.
- The packet assumes the existing shell-owned host orchestrator runtime remains the starting point.
- The packet is ordered so that session identity and participant modeling land before world ownership, invalidation, and store reshaping rely on them.

## Non-Goals in This Packet

- Full `agent start/resume/fork/stop` CLI productization
- Auth-bundle / `SUBSTRATE_LLM_AUTH_BUNDLE_FD` work
- macOS and Windows shared-world parity
- Renaming the local `agent-api-*` crates
