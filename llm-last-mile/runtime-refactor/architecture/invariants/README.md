**Kind:** architecture index
**Stable ID:** `shared-invariants`
**Canonical for:** no authority; shared invariant navigation only
**Status:** non-authoritative navigation
**Authority scope:** none; follow the linked canonical invariant owners
**Source span:** D7 assembly only
**Supersedes:** none
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md)

# Non-negotiable invariants

> **Authority boundary:** This file only indexes the extracted shared invariants. It does not renumber invariants, alter packet-family-local canonical ownership, or replace the linked canonical invariant owners.

| Invariant | Canonical owner |
|---|---|
| 1. Durable session truth is process-independent | [`01-durable-session-truth-is-process-independent.md`](01-durable-session-truth-is-process-independent.md) |
| 2. Private transports are fast paths | [`02-private-transports-are-fast-paths.md`](02-private-transports-are-fast-paths.md) |
| 3. Routing is exact and fail-closed | [`03-routing-is-exact-and-fail-closed.md`](03-routing-is-exact-and-fail-closed.md) |
| 4. Long-lived work accepts before it completes | [`04-long-lived-work-accepts-before-it-completes.md`](04-long-lived-work-accepts-before-it-completes.md) |
| 5. Cancel targets active work | [`05-cancel-targets-active-work.md`](05-cancel-targets-active-work.md) |
| 6. Obligations are event-derived canonical truth | [`06-obligations-are-event-derived-canonical-truth.md`](06-obligations-are-event-derived-canonical-truth.md) |
| 7. Auto-attach restores ownership only | [`07-auto-attach-restores-ownership-only.md`](07-auto-attach-restores-ownership-only.md) |
| 8. World placement and policy mediation are separate proofs | [`08-world-placement-and-policy-mediation-are-separate-proofs.md`](08-world-placement-and-policy-mediation-are-separate-proofs.md) |
| 9. `external_sandbox` assigns responsibility | [`09-external-sandbox-assigns-responsibility.md`](09-external-sandbox-assigns-responsibility.md) |
| 10. Dispatch policy only narrows | [`10-dispatch-policy-only-narrows.md`](10-dispatch-policy-only-narrows.md) |
| 11. Existing `world_fs` enforcement is the execution path | [`11-existing-world-fs-enforcement-is-the-execution-path.md`](11-existing-world-fs-enforcement-is-the-execution-path.md) |
| 12. Runtime-native configuration is projection | [`12-runtime-native-configuration-is-projection.md`](12-runtime-native-configuration-is-projection.md) |
| 13. Credentials are launch-time gateway handoff, not projected files | [`13-credentials-are-launch-time-gateway-handoff-not-projected-files.md`](13-credentials-are-launch-time-gateway-handoff-not-projected-files.md) |
| 14. `SUBSTRATE_HOME` is private per-user authority state | [`14-substrate-home-is-private-per-user-authority-state.md`](14-substrate-home-is-private-per-user-authority-state.md) |

The packet-family-local B1/B2.1 replay/restart boundary and B3.1/C1 obligation material remain preserved through the existing family-local canonical records linked from invariants 4 and 6.
