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
  - Publicize the narrow `substrate agent start|reattach|fork|stop` control plane with exact session selectors, `reattach` as exact attached-owner recovery for the named durable session, and `stop` as the canonical closeout path for attached or parked durable sessions.
- [ORCH_PLAN-19.md](./ORCH_PLAN-19.md)
  - Parent-frozen execution controller for the `PLAN-19` rollout on `feat/session-centric-state-store`.
- [PLAN-20.md](./PLAN-20.md)
  - Add the public prompt-taking caller surface with exact `start` / `turn` verbs, canonical `reattach`, backend-aware turn resolution, and helper-owned streaming.
- [ORCH_PLAN-20.md](./ORCH_PLAN-20.md)
  - Parent-frozen execution controller for the `PLAN-20` rollout on `feat/session-centric-state-store`.
- [PLAN-21.md](./PLAN-21.md)
  - Extend the shared-owner/member-runtime contract onto the supported macOS/Lima forwarded path without changing the Linux source-of-truth placement model.
- [ORCH_PLAN-21.md](./ORCH_PLAN-21.md)
  - Parent-frozen execution controller for the `PLAN-21` rollout on the shared-owner/member-runtime parity branch.
- [PLAN-22.md](./PLAN-22.md)
  - Harden the public `turn` surface around the already-landed `start|turn|reattach|fork|stop` contract, prove Linux world-member follow-up through the typed submit boundary, and pin explicit fail-closed coverage.
- [ORCH_PLAN-22.md](./ORCH_PLAN-22.md)
  - Parent-frozen execution controller for the `PLAN-22` rollout on the authoritative shared-owner/member-runtime parity branch.
- [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](./23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
  - Landed lifecycle correction: durable host orchestration sessions now survive clean attached-client exit, retain session-local inbox work under `sessions/<session>/inbox/`, let `status` surface `parked_resumable` and `awaiting_attention`, treat successful `reattach` as durable attached truth for that exact session, keep `stop` as the canonical closeout path for attached or parked durable sessions, and reserve non-routable meaning for `terminal` only.
- [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](./24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)
  - Hardening follow-on: host bootstrap and detach parking now preserve the exact durable-session fields that later `turn`, `reattach`, `stop`, and `status --json` rely on.
- [25-host-durable-session-closeout-and-qa-hardening.md](./25-host-durable-session-closeout-and-qa-hardening.md)
  - Truth-convergence and QA-hardening follow-on: freeze `turn` as prompt-taking follow-up on the same durable session, `reattach` as attached-owner recovery only, `stop` as durable closeout for attached and parked host sessions, `status --json` as the authoritative parked-session read surface, detached-world follow-up as fail-closed until `reattach`, and inbox behavior as narrow retained state rather than a public inbox workflow.
- [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](./29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
  - Landed shared dispatch-contract truth: one internal dispatch contract now resolves inventory-backed and persisted-attach-backed baselines, persists generalized `HostAttachContract` truth, and keeps human plus orchestrator-controlled dispatch on the same semantics.
- [30-public-world-scoped-agent-start-and-capability-flags.md](./30-public-world-scoped-agent-start-and-capability-flags.md)
  - Draft follow-on: public world-scoped `agent start` depends on the slice 29 contract and keeps durable authority host-rooted.
- [31-lazy-host-attach-for-host-rooted-world-start.md](./31-lazy-host-attach-for-host-rooted-world-start.md)
  - Draft follow-on: later attach must reuse the persisted slice 29 host-attach truth for explicit continuity or fresh attach modes.

## Non-Goals in This Packet

- Full agent caller-surface parity beyond the landed narrow `start|turn|reattach|fork|stop` contract
- macOS and Windows shared-world parity
- Renaming the local `agent-api-*` crates
