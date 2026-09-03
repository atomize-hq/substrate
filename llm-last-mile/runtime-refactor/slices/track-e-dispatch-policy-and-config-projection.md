**Kind:** track index
**Stable ID:** `track-e-dispatch-policy-and-config-projection`
**Canonical for:** Track E navigation only
**Status:** non-authoritative navigation
**Authority scope:** exact extracted Track E table plus current navigation correction; no schedule, dispatch, or implementation authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) lines 216–224
**Supersedes:** canonical ownership of the extracted Track E table
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# Track E — Dispatch-scoped policy narrowing and config projection

> **Authority boundary:** This file is a navigation index only. E1 is terminally complete at
> `18f719898ce2a48f65e95b3b23f3b2cfd685c4af`; E2 is terminally complete over implementation
> `96e102d9f5690e0d63957f6e9db56d632b7cdd17`, and its completed E2-RM authority correction remains
> separately specified/unadmitted. E3 now has a controlling documentation-only specification but is
> not admitted, dispatched, implemented, or green. E4 remains undispatched. Schedule authority stays
> with the controlling decision, packet, and gate owners.

The extracted E2 row below is pre-correction navigation. Its current authority is the
[E2 correction and closure](e2-policy-commitments-on-work-and-workers.md#terminal-closure):
E2 owns the independent immutable policy commitment/cap, not receipt or full-manifest construction.

The extracted E3 row is also historical navigation. Its current authority is the
[E3 controlling correction](e3-agent-config-projection-and-gateway-adoption.md#controlling-authority-correction):
one accepted-home projection registry, strict V2 carrier, descriptor-pinned Codex/wrapper/gateway,
deterministic Codex 0.125 rendering, and exact managed-gateway adoption. The row's `crates/codex`
path is absent; later implementation uses a new shared `crates/config-projection`. D1 owns later V3
and its envelope. E3 and E2-RM/B2.2 may coordinate integration windows but are not prerequisites for
one another.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| [**E1 — Restricted world_fs narrowing**](e1-restricted-world-fs-narrowing.md) | Accept a request-scoped restricted `PolicyPatch.world_fs`, validate path-containment monotonicity, and materialize a canonical narrowed snapshot. | `02` EffectivePolicyResolver + NarrowingPatch rows; `04` narrowing contract/rules; steering policy matrix capability section | WorldCommandExecutionBroker; receipt acceptance; agent inventory overlay logic | `crates/broker`; `execution/{policy_model,policy_snapshot,agent_inventory}.rs`; transport API policy types; resolver tests | No new filesystem policy model; no broadening dimensions; no receipt/manifests yet. | Gate=false rejects; gate=true accepts only narrowing; file-under-directory containment works; escapes/symlinks/broadening fail closed. | `RG-POLICY-01`, `RG-POLICY-02` |
| [**E2 — Policy commitments on work and workers**](e2-policy-commitments-on-work-and-workers.md) | Build immutable active-run snapshots and retained-worker caps linked to B1 acceptance records; recompute future turns as parent ∧ cap ∧ turn patch before B2.2/B3.2 expose final receipts. | `04` acceptance/receipt/manifest and immutable snapshot rules; `02` ReceiptRegistry/RetainedRuntime rows | B1 acceptance records; B3.1 event truth; B2.2/B3.2 consumers; E1 resolver; fork lifecycle | receipt/manifest persistence; `orchestrator_world_dispatch.rs`; retained lifecycle code; policy tests | No mutation of accepted snapshots; no automatic worker broadening; no config rendering. | Final receipts/manifests reference their B1 acceptance record and record immutable hash/ref/revision/reason; parent narrowing affects future turns; parent broadening does not widen worker; fork inherits cap. | `RG-POLICY-03`, `RG-RECEIPT-02`, `RG-CANCEL-01`; E2 policy clause of `RG-OBS-01` |
| [**E3 — AgentConfigProjectionService and gateway adoption**](e3-agent-config-projection-and-gateway-adoption.md) | Make logical/effective/native config projection first-class per retained worker and point world Codex at the existing in-world gateway secure-FD credential/session boundary. | `02` config projection + envelope + realization rows; `04` envelope projection, existing-carrier adoption note, and `LaunchTimeSecretHandoffV1`; existing gateway auth-handoff internals/verification docs; config projection, Codex mapping, and workspace overlay designs | Envelope; retained manifest; EffectivePolicyResolver; in-world gateway; RuntimeFamilyRealizationAdapter | `execution/agent_inventory.rs`; `crates/codex`; `world-service/member_runtime.rs`; existing in-world gateway endpoint/session integration points; bounded projection module; projection tests | No reimplementation of the landed FD carrier; no ambient runtime file as authority; no secret payload in projected files, manifests, traces, or UAA child FDs; no workspace-sync policy decision. | Sibling workers get isolated non-secret projections; `CODEX_HOME`, `config.toml`, and provider wiring can be rebuilt from Substrate truth plus accepted policy; Codex uses the exact managed gateway whose credentials arrived through the existing one-time FD carrier; copied auth/config runs only as named/logged/non-promotable compatibility; narrowed hints never substitute for enforcement. | `RG-CONFIG-01`, `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`; E3 credential-handoff clause of `RG-OBS-01` |
| [**E4 — Host-visible write/sync contract**](e4-host-visible-write-sync-contract.md) | Freeze and prove when world writes are immediately host-visible, isolated, or explicitly reconciled under narrowed policy. | `05` host-visible sync row; `04` monotonicity and snapshot rules; workspace overlay model sync boundary | E1–E3; world-service fs enforcement; broker | `world-service` fs/overlay execution; relevant `crates/world*`; workspace-sync code/tests/docs | No silent best-effort copying; no treating command success as host-visible proof; no policy broadening to make sync pass. | Matrix proof covers `host_visible=true`, full isolation, narrowed write allowlists, retained turns, denied writes, and explicit reconciliation semantics. | `RG-SYNC-01`, `RG-POLICY-02`, `RG-UAA-03` |
