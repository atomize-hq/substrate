**Kind:** slice row
**Stable ID:** `a3-persistence-and-compatibility-split`
**Canonical for:** extracted A3 slice row only
**Status:** canonical slice row record
**Authority scope:** exact extracted source table header and row plus a non-canonical projection of the bounded A3-scoped `RG-BASE-03` disposition; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 132
**Supersedes:** canonical ownership of the extracted `A3 — Persistence and compatibility split` row
**Superseded by:** none
**Projection consumers:** [`track-a-authority-and-surface-neutrality.md`](track-a-authority-and-surface-neutrality.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# A3 — Persistence and compatibility split

> **Authority boundary:** This file owns only the extracted A3 Track A row. The `RG-BASE-03` note below projects the canonical evidence disposition and adds no authority here. This file preserves the exact row text, does not authorize persistence/compatibility implementation, and does not dispatch A3.

## RG-BASE-03 completion-wall projection

**Projection only:** the
[canonical disposition](../evidence/baseline-behaviors-and-smoke-scenarios.md#a3-scoped-rg-base-03-disposition)
keeps `RG-BASE-03` open, blocking, unwaived, and unchanged as a stable gate; the failed full
targeted-host-turn/UAA continuity witness is not green. A3 consumes only the bounded differential
disposition recorded there:
unchanged canonical/compatibility behavior, usable unprefixed `ls`/`pwd`, the required `cd ../` cage
denial, zero compatibility-read mutation, and no regression relative to the exact baseline. Honest
baseline-equivalent failure at the later lifecycle boundary permits A3 verification to continue,
but any candidate-specific persistence or compatibility defect still blocks A3.

The full later targeted-turn continuity proof remains with its existing downstream owner,
[C2](c2-inbox-and-auto-attach-projections.md#rg-base-03-continuity-proof-owner-projection), after C2's declared
prerequisites. A3 may not weaken the activated-store legacy-writer guard or add HSA,
obligation-ledger, receipt, supervisor, or lifecycle semantics to make that witness pass. This
correction does not admit or dispatch C2 and does not mark A3 complete.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A3 — Persistence and compatibility split** | Reduce StateStore to atomic persistence/schema evolution and isolate diagnostic compatibility reads from new authority writes. | `02` StateStore + CompatibilityReadModel rows; `01` authority map; `04` durable revision rules | HostSessionAuthority; InboxProjection; receipt persistence | `agent_runtime/state_store.rs`; a bounded new facade/module under `agent_runtime/`; persistence/read-model tests | No wholesale database rewrite; no conversion of pre-A1 authority artifacts into A1 authority; no behavior changes outside moved ownership. | New authority writes bypass compatibility projection; torn-root/unsupported-state diagnostics remain read-only; newer revisions always win. | `RG-AUTH-02`, `RG-BASE-01`, `RG-BASE-03` |
