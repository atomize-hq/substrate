**Kind:** slice index
**Stable ID:** `shared-slice-map`
**Canonical for:** shared sequencing/dependency rules, slice-closeout minimum, and non-authoritative Track A–E navigation only
**Status:** canonical shared slice-map record
**Authority scope:** exact extracted shared sequencing/dependency and slice-closeout source bodies plus non-authoritative track navigation only; no schedule, dispatch, or implementation authority
**Source span:** D9 shared sequencing/dependency and slice-closeout extraction from [`../03-phase-slice-map.md`](../03-phase-slice-map.md)
**Supersedes:** canonical ownership of the extracted `Sequencing rules` and `Slice closeout minimum` root spans; track and slice/task projections remain separately qualified below
**Superseded by:** none
**Projection consumers:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md), [`../index/README.md`](../index/README.md), [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md), [`track-b-world-dispatch-receipts-supervision-and-cancel.md`](track-b-world-dispatch-receipts-supervision-and-cancel.md), [`track-c-obligations-inbox-auto-attach-and-router-attach.md`](track-c-obligations-inbox-auto-attach-and-router-attach.md), [`track-d-uaa-execution-envelope-and-side-effect-mediation.md`](track-d-uaa-execution-envelope-and-side-effect-mediation.md), [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md)

# Shared slice map index

> **Authority boundary:** This file owns only the extracted shared sequencing/dependency and slice-closeout bodies plus non-authoritative Track A–E navigation. Schedule and dispatch authority remain with the controlling decision, packet, gate, and review owners together with [`../index/current.md`](../index/current.md); this file creates none.

## Sequencing rules

Each slice is independently reviewable. A0 is a diagnostic inventory; every later implementation slice must move one authority boundary plus proof. A slice may read sibling context without editing sibling ownership.

### Bounded review sequence

Every new slice follows the development-review contract in `04`. Freeze the selected integrated
outcome, scope, proof gates, subject-fingerprint method, and review budget before implementation.
Run one discovery review or same-subject burst, consolidate valid `P1`/`P2` remediation, and use one
different-fresh delta-focused closure review. Up to two supplemental causal cycles are permitted
only for `P1`/`P2` findings directly caused or unmasked by the immediately preceding remediation
inside unchanged authority and risk. `CLEAN` ends the loop; unrelated blockers, expansion, or budget
exhaustion stop non-completed. Unfixed `P3`/`P4` findings go to `06` and do not create remediation or
review cycles.

The parent validates the review-cycle record after each returned cycle and runs the `--next-cycle`
preflight before launching any closure or supplemental review. Historical packet-specific reviewer
counts remain evidence of those packets, not an automatic requirement inherited by later slices.

Hard dependency spine:

```text
A0 -> A1.1e -> B0 -> B1-3a/B1-3b receipt core -> B2.1-1/2/3 ------------------+
              \-> A1.2a current-authority protocol -> A1.2a-WB correction     |
                  -> A1.2a-S bounded Start                                    |
                  -> B1/B2.1-R0 -> B3.2a -> B3.2a-WA ------------------------+
                                                                               -> B1/B2.1-0
                                                                               -> joint closeout
                                                                               -> B3.1 -> C1 -> A1.2b
                                                                               -> A1.3-P1 -> A1.4 -> A1
A1.1d integrated Linux/native-macOS closeout -> A1
A1 -> A2/A3 -> E2 -> B2.2 -> remaining B3.2 -> B4 -> C2 -> C3
B1/B2.1 joint production closeout + E1 -> E2
D1 -> D2 -> D3
E1 -> E2 -> E3 -> E4
B4 + C1 + D2 + E2 + E3 -> D3 `RG-OBS-01` integration closure
D2 and E1 must agree on PolicySnapshotV3, but may land in either order behind fail-closed gates.
```

Current Track A closure disposition: the controlling
[Linux-first decision](../linux-first-runtime-resumption/DECISION.md#macos-lane) supersedes only
the native-macOS predecessor edge shown in the preserved spine by making that lane separate and
non-blocking. The landed Linux A1.1d closeout, completed A1.1/A1.2 and A1.3-P1 predecessors, and
the terminal [A1.4 closure](tasks/a1-4-auto-attach-producer-adoption.md#terminal-closure) leave no
other A1 condition. A1 is terminally complete. [A2](a2-host-execution-episode-demotion.md#terminal-closure)
and [A3](a3-persistence-and-compatibility-split.md#terminal-closure) are terminally complete under
their separate closure identities. A0's committed inventory already satisfies its diagnostic exit,
so the four-row Track A is terminally complete. The dependency spine names
[E2](e2-policy-commitments-on-work-and-workers.md) as the canonical successor, while E1 remains a
separate unresolved prerequisite; E2 awaits fresh admission and explicit dispatch.

This graph is acyclic. `B1-3a/B1-3b receipt core` is a review state, not a claim that B1 is
production-complete: it supplies the exact proposal, acknowledgement, activated-store acceptance
record, and immutable anchor that B2.1 consumes. B2.1 may begin only after that core is review-
clean. B1 and B2.1 then receive one joint production closeout, and B3.1 has no edge from either
packet separately.

Cross-packet integration hold: a lower-level A1 persistence packet may expose a preexisting
production bypass that only a later canonical owner packet can close. The production-ingress audit
found that A1.1e is a necessary reader but not a sufficient joint-closeout foundation: it cannot
create the current authority its reader requires, does not return the applied caller descriptor,
and does not represent the shared dispatcher's live-retained count. The corrected bounded corridor
keeps the already-preserved B0 → B1 receipt core → B2.1 branch independent while
A1.1e → A1.2a current-authority protocol → A1.2a-WB Host/world-binding correction →
A1.2a-S bounded internal Start adoption →
B1/B2.1-R0 → B3.2a → B3.2a-WA builds the authority/retained branch. Those branches first join at
B1/B2.1-0 action-scoped preparation, then proceed through the joint
closeout → B3.1 → C1 → A1.2b even while A1.1d
integrated Linux and native-macOS closeout remain open. The corridor may not issue or apply host
transitions except the exact greenfield Start subset assigned to A1.2a, demote episodes, extract
general persistence, enable foreground early return, or perform retained-worker lifecycle beyond
B3.2a's exact creation/admission state and fail-closed live count. A1.2b resumes only after C1 can return the complete semantic cut from B2.1's durable exact
event truth and B3.1's typed retained-event semantics. This
hold does not permit A1.3 consumer adoption. The failing regression remains explicit, stale
lifecycle/world-binding behavior remains rejected, the feature branch remains non-landable, and
A1.2b/A1.3 must close their assigned gates before A1 can complete. This exception creates no
A1.1d-6, waives no gate, bypasses no A2/A3 ownership, and does not apply to another blocker.

The corridor is minimal for these reasons:

1. B0 owns runtime-generated stream/frame/event/terminal identity and ordering; B2.1 may not invent
   those identities after observation.
2. B1-3a/B1-3b precede B2.1 because a durable observer claim and journal require one accepted task
   or active-run identity. The review-clean core preallocates the proposed acceptance ID and typed
   request context that world-service retains before B3.1 can emit an exact accepted-run envelope;
   the proposal is not accepted truth until the B0 acknowledgement is durably recorded. B1 itself
   does not close at that review point because the production path must still hand the accepted
   stream to B2.1 without attempting a legacy writer.
3. B2.1 owns durable observation, duplicate/reorder rejection, terminal ordering, and restart
   reconciliation. The existing foreground may block as a receipt waiter until B2.2.
4. B3.1 is required because C1 needs exact retained target/run/thread/class/attention/causation
   semantics that neither a generic transport carrier nor the receipt registry owns. B3.1 moves
   the existing provider-payload classification to a fail-closed producer-side normalizer before
   `AgentEvent` construction; the emitted typed member, not host parsing of `AgentEvent.data`, is
   then the semantic source.
5. C1 alone classifies and materializes obligations and declares the complete cut. A1.2b consumes
   that result unchanged.

The typed host-transition correlation does not add a reverse dependency on A1.2b. B1 places both
the carrier and its equality-only opaque commitment representation in `substrate-common`, adds it
optionally to the typed request context, and keeps it absent on production paths until
HostSessionAuthority supplies it; no new A1.1e hash domain or verifier is required. B3.1 copies it,
B2.1 journals the bytes without interpreting them, B3.1 validates equality with B1, and C1 is fully functional for exact inputs while
refusing a transition-scoped Complete result when the correlation is absent or mismatched. A1.2b
later becomes the only production source, validates its own intent/revision/payload commitment,
supplies its exact intent/transition-run correlation through the HostSessionAuthority-owned call
boundary, and consumes the C1 result unchanged. Its real-path adoption is therefore the downstream
integration gate, not a C1 prerequisite.

No A2 prerequisite is pulled forward because process/PID/socket demotion is unnecessary to build
the producer-to-ledger truth path. No A3 prerequisite is pulled forward because StateStore may
provide bounded atomic persistence without deciding receipt, observation, messaging, or obligation
semantics.

The B1/B2.1 production-ingress ownership audit selects **Case B**. A1.2a now provides the
current-authority protocol and exact read; A1.2a-WB has corrected its Host/world-binding validation;
and A1.2a-S has adopted greenfield Start on the ordinary internal host bootstrap and carries the
bound capability into the live toolbox. B1/B2.1-R0 now provides the review-clean minimal
RetainedWorkerRuntime object/read plus HostSessionAuthority-owned lineage/ref registration proof for a canonical retained target;
B3.2a has now supplied the real Spawn integration plus canonical admission/routability truth that
R0 deliberately lacks, and B3.2a-WA has exact-adopted the HSA-bound world for shared-session
physical ownership without changing that binding; both are review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`. B1/B2.1-0 now partitions
`prepare_orchestrator_world_dispatch` by action and payload so RunWorldTask, ordinary retained
ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait do not consume the missing
legacy `live_retained_worker_count`; `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop,
and fork admission/lifecycle behaviors remain unchanged, unpromoted, and outside the bridge. Spawn
keeps its existing steering/outcome but obtains its live count and registration from B3.2a. This
makes the prerequisite review-clean through `83101dcbcc750e6e8fb8979bea19f1f777792188` and
moves no Attach/Resume, post-turn, obligation, correlation, public-adoption, accepted-turn, or
broader retained-lifecycle work early. A1.2b remains after C1.

All code areas below are allowlists for planning, not permission to edit every listed file. Future implementation must run live impact analysis before symbol changes.


## Track navigation

> **Navigation only:** Each linked Track file preserves extracted D9 table projections without authorizing implementation, altering packet/gate status, or dispatching a slice.

| Track | Canonical owner | Extracted navigation scope |
|---|---|---|
| Track A — Authority and surface neutrality | [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md) | Terminally complete navigation for the extracted A0/A1/A2/A3 rows while keeping A1 packet decomposition inside the A1 owner and preserving held `A1.3-P0`/`A1.3` plus the completed `A1.3-P1` owner. |
| Track B — World-dispatch receipts, supervision, and cancel | [`track-b-world-dispatch-receipts-supervision-and-cancel.md`](track-b-world-dispatch-receipts-supervision-and-cancel.md) | Navigation for the extracted B2.2/B3.2/B4 rows while preserving the recorded B0 through joint-closeout and B3.1 family owners at their existing D6 locations. |
| Track C — Obligations, inbox, auto-attach, and router attach | [`track-c-obligations-inbox-auto-attach-and-router-attach.md`](track-c-obligations-inbox-auto-attach-and-router-attach.md) | Navigation for the extracted C2/C3 rows while preserving the completed C1 family owner at its existing D6 location. |
| Track D — UAA execution envelope and side-effect mediation | [`track-d-uaa-execution-envelope-and-side-effect-mediation.md`](track-d-uaa-execution-envelope-and-side-effect-mediation.md) | Navigation for the extracted D1/D2/D3 rows plus the preserved fail-closed staging rule. |
| Track E — Dispatch-scoped policy narrowing and config projection | [`track-e-dispatch-policy-and-config-projection.md`](track-e-dispatch-policy-and-config-projection.md) | Navigation for the extracted E1/E2/E3/E4 rows only. |

## Slice closeout minimum

A0 closeout records inventory coverage, classification evidence, proposed owners, and first migration target in `02-seam-crosswalk.md`.

Every implementation slice closeout records:

1. the crosswalk row(s) changed;
2. the authority decision moved;
3. the production call path now using it;
4. the enforcement point, or `not applicable` with justification;
5. exact unit/integration/smoke evidence;
6. the permanent regression gate added or preserved; and
7. why sibling seams were not accidentally widened.

Do not promote a crosswalk row merely because its slice landed. Promote only after the seam-level four-part landing rule is satisfied.

## Navigation authority reminder

Schedule and dispatch authority remain with the controlling decision, packet, gate, and [`../index/current.md`](../index/current.md) owners.
