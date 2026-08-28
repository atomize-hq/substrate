**Kind:** architecture
**Stable ID:** `B3.1-C1-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** B3.1/C1 obligation architecture invariant
**Source span:** [`01-target-architecture.md#6-obligations-are-event-derived-canonical-truth`](../01-target-architecture.md#6-obligations-are-event-derived-canonical-truth), baseline lines 171–183

# B3.1/C1 family architecture

### 6. Obligations are event-derived canonical truth

Attention-driving runtime events are persisted by the supervisor and materialized by
`ObligationLedger` into idempotent obligations as they arrive, before terminal exit when applicable.
For an accepted retained stream, every post-acknowledgement semantic `Event` frame is normalized
into the typed messaging envelope and included in the ledger's ordered classified-event set; an
unknown, untyped, or omitted event keeps the materialization cut non-Complete.
The ledger alone advances the per-session revision and materialized-through watermark and declares
a Complete cut for an exact runtime-generated terminal event ID/sequence. Stream exhaustion, EOF,
timeout, PID/helper/socket state, inbox rows, pending counts, worker flags, and compatibility
projections cannot establish event completion or obligation completeness. Host `awaiting_attention`
derives from unresolved obligations. Worker `attention_pending` is a separate lifecycle state.
