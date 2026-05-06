# LLM Last Mile SOW Packet

This directory contains the reviewed statement-of-work packet for the prerequisites that led into the landed session-centric live registry step:

- landed registry shape:
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`
  - `~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`
  - registry wraps UAA handles with Substrate-owned orchestration metadata keyed by `orchestration_session_id` and `participant_id`
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
- `11` through `13` are addenda after the original packet:
  they cover the host↔world member transport cutover, the gateway auth-bundle last-mile security slice, and the final true in-world member-placement hardening pass.

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
   - Landed runtime output now writes canonical session roots under `agent-hub/sessions/<orchestration_session_id>/...` with per-session participant records keyed by `participant_id`.
7. [07-world-replacement-ordering-rollback-atomic-metadata.md](./07-world-replacement-ordering-rollback-atomic-metadata.md)
   - Harden shared-world replacement ordering, rollback, cleanup, and atomic Linux world metadata writes so exactly one recoverable active world remains authoritative.
8. [08-explicit-orchestration-authority-event-emission.md](./08-explicit-orchestration-authority-event-emission.md)
   - Remove ambient PID-based orchestration identity lookup from shell-owned event emission and require explicit runtime-owned context instead.
9. [09-live-state-authority-and-compatibility-cutover.md](./09-live-state-authority-and-compatibility-cutover.md)
   - Freeze canonical session-root records as live-state authority, bound compatibility writes/reads during cutover, and keep torn-root control-plane selection fail-closed while read-only status can degrade to warnings.
10. [10-member-runtime-launch-seam.md](./10-member-runtime-launch-seam.md)
   - Add the missing production world-scoped member runtime launch/lifecycle seam that consumes the existing authority and status machinery.

## Review Notes

- The packet is intentionally Linux-first for shared-world ownership.
- The packet does not require new inventory `uaa_kind`-style schema.
- The packet assumes the existing shell-owned host orchestrator runtime remains the starting point.
- The packet is ordered so that session identity and participant modeling land before world ownership, invalidation, and store reshaping rely on them.

## Follow-On Plans

The original packet stops before public control-plane productization. The follow-on planning documents for that work are:

- [PLAN-19.md](./PLAN-19.md)
  - Publicize `substrate agent start|resume|fork|stop` with exact session selectors and a Linux-first fail-closed posture.
- [ORCH_PLAN-19.md](./ORCH_PLAN-19.md)
  - Parent-frozen execution controller for the `PLAN-19` rollout on `feat/session-centric-state-store`.

## Non-Goals in This Packet

- Full `agent start/resume/fork/stop` CLI productization
- macOS and Windows shared-world parity
- Renaming the local `agent-api-*` crates
