**Kind:** current-state projection
**Stable ID:** `B1-B2.1-family`
**Canonical for:** B1/B2.1 family corridor, prerequisite, recovery, and joint-closeout current-state projections
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** [`00-README.md#current-control-conclusion`](../00-README.md#current-control-conclusion), baseline lines 132–155, 172–182, and 395–452 (family-local sentences only where source lines are mixed)
**Supersedes:** canonical ownership of the extracted source bodies; source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# B1/B2.1 family current-state and closeout projection

## Current B1/B2.1 control conclusion

The current tree contains important constraints and footholds, but this pack does not classify any
required seam as `ContractCorrectAndProven`. That is intentional. A1.1e is landed and supplies
necessary exact-read primitives, but the production-ingress audit found that it is not sufficient
by itself for the B1/B2.1 joint closeout: no production path creates a current authority before
A1.2, and the shared prepared dispatch still requires noncanonical compatibility records plus a
legacy live-retained count. The corrected bounded corridor is:

```text
A1.1e -> B0 -> B1-3a/B1-3b receipt core -> B2.1-1/2/3 -------------------------+
       \-> A1.2a current-authority protocol -> A1.2a-WB binding correction       |
           -> A1.2a-S bounded Start adoption                                    |
           -> B1/B2.1-R0 canonical retained target protocol                     |
           -> B3.2a retained creation/admission bridge
           -> B3.2a-WA exact bound-world ownership adoption ---------------------+
                                                                                -> B1/B2.1-0
                                                                                -> joint closeout
                                                                                -> B3.1 -> C1 -> A1.2b
```

This corridor does not close A1.1d, bypass A2/A3 ownership, enable foreground early return, or
promote any seam. B0's runtime-owned identity carrier is landed with its producer clauses proven;
the B1 receipt and B2.1 supervisor cores are recovered and review-clean, and B1/B2.1-0 is
review-clean, but B1 and B2.1 remain below complete until their later joint production integration
closeout.

## B1/B2.1 completion projection

B1/B2.1-R0 is now landed and independently review-clean through
`bb3eefba`. B3.2a plus its B3.2a-WA prerequisite are independently review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The B1 receipt core is recovered through
`6436289fd9dd55ea516b96ef3299e4055d1ea718`; the B2.1 supervisor and replay/startup cores are
recovered through `c519024bd91b6ca6e332d0b8881f7d13ded940e0` and
`de727091a39c884044179a89135df3db5d566778`, with versioned authority-store binding corrected by
`717579b0744154d343985ad439fb8756158f376f`. B1/B2.1-0 is review-clean through
`83101dcbcc750e6e8fb8979bea19f1f777792188`. The later joint production integration closeout is now
recorded against the bound 2026-08-03 source snapshot without additional product/test edits; its
supported Linux doctor plus installed-product smoke preserves the existing host product boundary
rather than source-binding the installed binary.

## Retained-target and dispatch-prerequisite projection

B1/B2.1-R0 lets RetainedWorkerRuntime create the immutable retained object graph and requires
HostSessionAuthority first to reserve the ingress idempotency key, validate the exact participant
identity supplied by its caller, and fix the replay-stable registration/object identities before
object publication, then atomically append exactly its participant to lineage, add its validated
object ref, advance the authority revision, and persist a distinct non-transition registration
proof. It then exact-resolves the canonical target; a still-Pending Start keeps its original
expected revision and A1.2b later accepts only the unique contiguous registration-proof ancestry.
R0 remains a registration protocol and does not claim a production caller, messaging,
accepted-turn observation, park/cancel/stop/fork, or live-count semantics. B3.2a is the separate
RetainedWorkerRuntime-owned production bridge: before R0 it atomically checks the durable
admission count/cap and reserves one exact participant slot and full canonical request fingerprint
under its own crash-stable admission key (never an HSA commitment key) across processes. A durable
per-session registration head alone may then fix the current authority revision. A queued
`SlotReserved` record plus no current head is valid: after the current head reconciles R0 it
releases the head without automatically promoting another record. Only exact re-presentation of
the complete canonical request for the lowest-sequence queued slot may acquire the next head;
later requests cannot overtake it, and an abandoned earliest slot remains conservatively live.
Remaining B3.2 owns exact, restart-safe durable resolution and reconciliation of that abandoned
admission; B4 owns the user/tool-facing exact inspect/cancel verb and distinct outcomes. B3.2a
implements neither protocol.
The admission record stores only the keyed commitment and non-secret fixed fields, never the
request/prompt/payload preimage. The bridge passes that
slot-fixed participant to R0 instead of allocating a retry-local ID, exact-joins R0, commits the
proof before opening the member stream, and carries a transport-neutral typed equality proof through
both the direct dispatcher transport and live internal-toolbox Spawn adapter via the real
transport-api `Service::execute_stream` member branch to the world-service launch boundary. Activated-
store legacy session/participant writes are replaced by exact proof validation. Unknown or
interrupted state stays nonterminal and counted; only exact B0 terminal truth removes it from the
live count. Active caller, posture, workspace, world, policy, spawn steering/outcome, and transport
event behavior remain unchanged. The bounded Linux live Spawn proof then exposed one remaining
physical-realization prerequisite: an authority-managed request could exact-bind HSA and admission
truth to the already-running generic world while world-service `AttachOrCreate` created a different
shared-owner world before launch validation. **B3.2a-WA** therefore runs before B3.2a closeout and
B1/B2.1-0. It gives the runtime-family/world backend one internal, exact, durable adoption operation
for the already-HSA-bound generic world. Adoption preserves the HSA-owned world ID and generation,
changes no HSA or RetainedWorkerRuntime record, exact-joins retry, rejects conflicting ownership,
and completes durable ownership publication before member process creation. It is authorized only
for authority-managed `Some(exact proof)`; compatibility `None` and ordinary world execution keep
their current behavior. The operation adds no world-api field or persisted wire-schema version,
does not persist request/prompt bytes, and does not prove member launch, Registered, routability, or
terminal success. At authorization time no seam was promoted, and B3.2a remained incomplete until
this prerequisite and the full live proof were clean. That prerequisite and proof are now
review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`: the exact HSA-bound world was durably adopted with
unchanged ID/generation, the authority-managed member registered through the production toolbox,
and no alternate world or prompt persistence was observed. B1/B2.1-0 now partitions the shared
prepared state, review-clean through `83101dcbcc750e6e8fb8979bea19f1f777792188`,
for RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task
Inspect/Cancel/Wait. Those paths
do not require the missing live-retained lifecycle count. This is the first point at which the
independent B1/B2.1 receipt/supervisor branch joins the authority/retained branch. B1 and B2.1 share one production
integration closeout only after those prerequisites. A1.2b remains after B3.1/C1 and retains all
successor and obligation-dependent post-turn work; it begins by freezing the later strict V3
root/intent/state extension, so A1.2a's V2 Start schema imports no B1-owned accepted-work type.
Retained Inspect/Cancel/Stop remain on unchanged compatibility paths for B3.2/B4 and cannot count
as a joint-closeout failure-to-pass transition.
B2.2/B3.2 retain the deferred receipt-UX and broader retained-lifecycle work after A1 and the named
A2/A3 boundaries. Similarly named event, span, or payload fields are not closure evidence.
