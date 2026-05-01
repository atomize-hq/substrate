# LLM Last Mile SOW Packet

This directory contains the reviewed statement-of-work packet for the prerequisites that should be completed before the planned live registry step:

- planned registry target:
  - `~/.substrate/run/agent-sessions/<orchestration_session_id>.json`
  - registry wraps UAA handles with Substrate-owned metadata
  - `trace.jsonl` remains audit history, not live state authority

## Current Status

- `03-shared-world-ownership-linux-first` is the landed Linux-first interface/backend slice:
  shared-world requests now use explicit request/response proof shapes, and Linux owner-bound reuse lives in `crates/world`.
- `04-thread-world-binding-into-runtime-state` is the runtime-state bridge:
  authoritative projection of the active shared-world binding is the handoff that slice `05` invalidation semantics and slice `06` session-centric store work both consume.
- `05-restart-invalidation-semantics` is still pending:
  invalidation/replacement registry semantics for generation changes and restart handling remain a follow-on slice.
- `07-world-replacement-ordering-rollback-atomic-metadata` is the landed backend-hardening reference:
  shared-world replacement now has explicit ordering, rollback, cleanup, and atomic metadata expectations that later shell/runtime slices can rely on.
- `08` through `10` are the remaining follow-on SOWs after the replacement-ordering and replacement-member work:
  they cover event-emission authority cleanup, live-state authority/cutover cleanup, and the missing production member-launch seam.

## Reviewed Execution Order

1. [01-orchestration-session-identity.md](./01-orchestration-session-identity.md)
   - Replace the process-global orchestration identity with a real Substrate-owned session identity.
2. [02-session-participant-record.md](./02-session-participant-record.md)
   - Generalize runtime state from a host-orchestrator-only manifest to a participant record that can represent orchestrator and member sessions.
3. [03-shared-world-ownership-linux-first.md](./03-shared-world-ownership-linux-first.md)
   - Make shared-world ownership explicit on Linux and bind one active `world_id` to one `orchestration_session_id`.
4. [04-thread-world-binding-into-runtime-state.md](./04-thread-world-binding-into-runtime-state.md)
   - Persist authoritative `world_id` and `world_generation` in runtime state instead of relying on trace/event history.
   - Bridge role: this is the runtime-state handoff that slice `05` restart invalidation and slice `06` session-centric state store changes build on.
5. [05-restart-invalidation-semantics.md](./05-restart-invalidation-semantics.md)
   - Define the live-state rule that a generation change invalidates all prior-generation world-scoped member sessions.
   - Status: still pending after the slice `04` runtime-state bridge.
6. [06-session-centric-state-store.md](./06-session-centric-state-store.md)
   - Rework the runtime handle store APIs and layout so consumers resolve by orchestration session instead of by orchestrator agent heuristics.
7. [07-world-replacement-ordering-rollback-atomic-metadata.md](./07-world-replacement-ordering-rollback-atomic-metadata.md)
   - Harden shared-world replacement ordering, rollback, cleanup, and atomic Linux world metadata writes so exactly one recoverable active world remains authoritative.
8. [08-explicit-orchestration-authority-event-emission.md](./08-explicit-orchestration-authority-event-emission.md)
   - Remove ambient PID-based orchestration identity lookup from shell-owned event emission and require explicit runtime-owned context instead.
9. [09-live-state-authority-and-compatibility-cutover.md](./09-live-state-authority-and-compatibility-cutover.md)
   - Freeze canonical session-root records as live-state authority, bound compatibility writes/reads during cutover, and keep ambiguity/torn-root behavior fail-closed.
10. [10-member-runtime-launch-seam.md](./10-member-runtime-launch-seam.md)
   - Add the missing production world-scoped member runtime launch/lifecycle seam that consumes the existing authority and status machinery.

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
