**Kind:** architecture
**Stable ID:** `B1-B2.1-family`
**Canonical for:** B1/B2.1 family control-state and B2.1 replay/restart architecture
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** composite of [`01-target-architecture.md#current-b1b21-control-state`](../01-target-architecture.md#current-b1b21-control-state) baseline lines 58–69 and [`01-target-architecture.md#b21-3-restart-and-producer-replay-boundary`](../01-target-architecture.md#b21-3-restart-and-producer-replay-boundary) baseline lines 171–205
**Supersedes:** canonical ownership of the extracted source bodies; source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# B1/B2.1 family architecture

> **Historical wording note:** The phrase `C1 is next` below is preserved verbatim from the source projection as provenance. It is not current scheduling truth: B3.1, C1, A1.3-P1, A1.4, the enclosing A1 slice, and A2 are recorded complete elsewhere, while A3 awaits fresh admission and explicit dispatch.

### Current B1/B2.1 control state

The immutable receipt core is recovered and review-clean through `6436289f`; the durable
supervisor core through `c519024b`; exact replay/startup activation through `de727091`; and the
versioned authority-store correction through `717579b0`. The action-scoped B-owned dispatch view,
including the read-only active-task tool adapter, is review-clean through `83101dcb`. This records
the B1/B2.1-0 prerequisite plus its later joint production integration closeout on frozen
2026-08-03 source `f37943eb917285a044c5e12a05b481572c8d0a09` /
`54ae7b2a2d467a568b575665b99dcb94ef2893a2`: B1 and B2.1 are now production-complete for the named
accepted families, foreground behavior remains blocking until B2.2, B3.1 is complete on the bound
Tuesday, August 4, 2026 source candidate, C1 is next, and no seam classification is promoted.


#### B2.1-3 restart and producer-replay boundary

`WorldWorkExecutionSupervisor` alone discovers durable nonterminal observations, interprets claims
and journals, validates the exact acceptance-record/stream/cursor join, starts reconciliation, and
decides whether exact B0 truth may advance or terminalize an observation. A startup surface may
invoke exactly one canonical recovery operation and retain the returned observation tasks. The
current `run_async_repl` call is only that bounded production activation hook: it cannot enumerate
or interpret supervisor records, reproduce recovery logic, or make lifecycle decisions. This hook
does not establish complete ingress-surface neutrality and cannot promote `SurfaceAdapter`,
`HostExecutionEpisode`, or `WorldWorkExecutionSupervisor`.

World-service may keep a bounded process-memory producer registry of exact B0 frames and expose
replay only for a supplied exact acceptance-record ID, exact stream ID, and exact frame cursor.
Frame identity, canonical bytes, and ordering remain unchanged. The endpoint cannot enumerate
streams or accept a fuzzy, backend-only, session-only, or otherwise partial lookup. Explicit stream,
per-stream frame, and per-stream byte limits are mandatory. Missing, mismatched, expired,
unavailable, corrupt, reordered, or conflicting replay fails closed. Producer replay cannot create
acceptance, an observation claim, terminal truth, an obligation, lifecycle truth, or success.

The supported restart domains are exact:

1. After a host/shell restart while the world-service registry survives, the durable supervisor
   discovers its nonterminal claim, reconnects by exact acceptance/stream/cursor identity, replays
   missing frames, and resumes live observation without duplicate journal transitions.
2. After a world-service restart, or whenever producer replay is unavailable, the durable claim
   remains nonterminal and unresolved. No terminal result, success, cancellation, deletion, cursor
   advance, or Complete obligation cut is fabricated. The shell may report the exact unresolved or
   replay-unavailable state, but PID, helper, socket, process, readiness, timeout, caller presence,
   endpoint absence, EOF, and stream exhaustion cannot resolve it.

Ordinary shell startup must distinguish fatal recovery-initialization corruption from a valid
nonterminal claim whose producer is unavailable. Corrupt storage, invalid claim identity, or an
impossible durable state fails closed. An individually unavailable producer stays durably
unresolved and does not require startup to pretend the stream resumed successfully.
