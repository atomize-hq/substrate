**Kind:** slice row
**Stable ID:** `b2-2-foreground-receipt-return`
**Canonical for:** extracted B2.2 slice row, its preserved blocked history, and the current second `E2-RM` admission correction
**Status:** B2.2 terminally complete by present META acceptance on 2026-10-27; historical Linux subject only
**Authority scope:** extracted row, preserved admission history, and the present bounded terminal acceptance below; no successor dispatch authority
**Source span:** [`../03-phase-slice-map.md`](../03-phase-slice-map.md) line 177
**Supersedes:** canonical ownership of the extracted `B2.2 — Foreground receipt return` row
**Superseded by:** none
**Projection consumers:** [`track-b-world-dispatch-receipts-supervision-and-cancel.md`](track-b-world-dispatch-receipts-supervision-and-cancel.md), [`../03-phase-slice-map.md`](../03-phase-slice-map.md)

# B2.2 — Foreground receipt return

> **Authority boundary:** This file owns only the extracted B2.2 Track B row and its current
> admission correction and present terminal acceptance. It preserves the exact historical row,
> does not reopen B1/B2.1 or E2, and grants no new implementation or successor dispatch.

## Terminal closure — 2026-10-27

Under the user's explicit instruction to make the B2.2 closure decision and update the control
plane, META `01a17a4d-1468-7bc3-9f77-5bb0cfce0b07` **accepts B2.2 as terminally complete** for
bounded Linux foreground receipt return and authenticated immutable retry at:

- Commit: `0e16a88b2c833c8176ac0e40c6eef33388a5b7df`.
- Tree: `ebe0ff7c56f571a23eb5be71db470fe9c0cfa023`.
- Parent / E2-RM baseline: `2012bb8b5562a73ed0ee45238c19252c16b25065`.
- Reviewed landed patch SHA-256: `e3c73cb85906f40b0afe5d11c4c343b478005d69ada89d728834e11b8b71efac`.

This is a present acceptance of that exact historical subject, not a recovered historical
approval or qualification of the current combined-tree runtime. TASK06's accepted evidence
recovery and both independent observations establish the source/review correspondence. Fresh
META authority advice found no additional substantive proof blocker for this precise claim.

### Accepted fence and authority disposition

META expressly adopts the exact five-file landed delta: `execution/orchestrator_world_dispatch.rs`,
`execution/agent_runtime/{dispatch_contract.rs,state_store.rs,tool_invocation_contract.rs}`, and
`repl/async_repl.rs`, all under `crates/shell/src/`. The `state_store.rs` addition is limited to
the authenticated request-indexed B1 semantic reader, typed outcomes/errors, reused validators
and colocated tests. It transfers no physical storage, reconciliation or lifecycle ownership.

G-ADM (original admission/five-file expansion) and G-ACCEPT (original accepting/publication
authority) remain unrecovered historical facts. Today's decision supplies present acceptance;
it does not assert those old decisions occurred. G-PROOF-TRACE remains incomplete broad-command
invocation detail, limiting the claims below. E2-RM's separate terminal closure remains unverified;
no additional B2.2 prerequisite is inferred. The preserved preimplementation restrictions below
remain chronology and behavioral constraints, not a requirement to redispatch landed work.

### Seven closeout records

| Required item | Accepted record |
|---|---|
| Crosswalk rows | WorldDispatchControl, RuntimeToolInvocationAdapter and InternalToolboxTransport record this bounded foreground-return increment, consuming WorldWorkReceiptRegistry and WorldWorkExecutionSupervisor truth. No seam-wide classification is promoted. |
| Authority moved | Foreground callers stop owning a terminal wait for accepted ephemeral and ordinary retained work. B1 owns acceptance, B2.1 observation/journal/replay/terminal truth, and E2 immutable policy/cap truth. |
| Production path | Four dispatcher entrypoints enter task/ordinary retained dispatch; durable Start/B1 acceptance and B2.1 handoff plus E2 publication/authenticated resolution precede receipt return through toolbox normalization and REPL consumption. Retry resolves immutable history before current authority preparation. |
| Enforcement | Request-indexed authenticated B1 snapshot and E2-RM exact joins, typed fail-closed outcomes, and durable-before-return ordering; no new policy or physical-store authority. |
| Exact proof | Final R/119 has 18 focused passes; R/147 final CLEAN matches R/139 patch and R/149/164 landing. Qualified differential and diagnostic evidence is accepted only as detailed below. |
| Permanent regressions | Landed colocated `b2_2_` tests satisfy B2.2's RG-RECEIPT-01/02/03 clauses: before-terminal return, exact immutable retry, concurrency winner notification, and caller-drop survival. Existing B1/B2.1 and named blocking compatibility are preserved. |
| Sibling boundaries | No E2-RM resolver/storage rewrite, B1 persistence/schema change, B2.1 journal/replay ownership change, obligation change, B3.2/B4/C2/C3/D1/E3 completion, migration, or platform-parity claim. |

### Accepted proof and retained limitations

Historical Linux Rust/Cargo 1.89 proof is bound to the final patch over the E2-RM baseline.
`cargo test --locked --offline -p shell --lib b2_2_ -- --nocapture --test-threads=1`
passed 18 tests; the root-lock regression passed separately. Final formatting and diff checks
passed. Final shell comparison was candidate 1433 passed / 100 failed / 1 filtered versus
baseline 1417 / 100 / 1, both exit 101, with identical named failures. Clippy retained 16
matching inherited diagnostics. Earlier workspace build/clippy/test failures were not rerun on
the final corrections; they establish no final whole-workspace success. Source-built CLI and
world doctor succeeded; shim/health failed on isolated missing `manager_hooks.yaml` and did not
produce successful JSON. Some broad-check full argv remains unrecovered. META accepts these
explicitly qualified results for this bounded historical slice, not as unconditional green checks.

Original HIGH retry/current-authority and mutating-probe findings were confirmed resolved by
R/129; its started-ID notification confirmation and the final root-lock correction/R/147 CLEAN
complete the historical finding chain. Controlled stream tests exercise the production dispatcher;
there is no live-provider, macOS/Windows, installed-current-runtime or whole-seam qualification.
B2.2-scoped receipt clauses close; remaining B3.2 receipt/manifest/lifecycle and B4 control clauses,
C2/C3 eligibility, E3/F14 and other ledger-wide obligations remain separately owned and open.

Retained evidence: [TASK06 final return](</home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration/.codex/orchestration/e3f-retained-next-20261022/evidence/task06/final/final-return.md>)
contains the exact command/result matrix, original finding dispositions, all seven minima and
source pointers. Its [manifest](</home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration/.codex/orchestration/e3f-retained-next-20261022/evidence/task06/producer/reference-manifest.json>)
binds 187 references. Historical R/* originals live in
`/home/spenser/__Active_code/.evidence/substrate-b2-2-foreground-receipt-return-ac4702a5/remediation-five-file-20260905/`.
These local evidence pointers do not claim remote archival availability.

## Current admission disposition

The 2026-10-27 terminal acceptance above supersedes the closure-unknown status in the dated
history below; it preserves its ownership and immutable-material requirements.


**Current status (2026-09-17):** B2.2 foreground receipt return and authenticated retry landed at
`0e16a88b2c833c8176ac0e40c6eef33388a5b7df`. The [2026-09-17 control-plane audit](</home/spenser/__Active_code/review-evidence/runtime-refactor-control-plane-audit-20260917T152119Z.md>)
matched its landed patch to retained final `CLEAN` review, with focused 18-pass proof and inherited
failures preserved; this is not an absolutely green workspace claim. Separate canonical terminal
closure was not located. [`E2-RM` owner](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite) separately records its implemented/review-clean/landed status
and closure uncertainty. Neither landing is a new closure or successor admission here.

**Historical admission correction:** The following diagnosis and fresh-admission requirements
record the earlier preimplementation disposition. Its immutable-material and ownership rules remain
controlling; its unimplemented-prerequisite claim is superseded by the landing facts above.

B2.2 is not admissible from the completed E2 closure alone. The existing authenticated E2 lookup
requires an already-known `DispatchPolicyCommitmentRefV1` and does not expose the complete stored
material needed to reproduce an original foreground receipt after response loss, restart, B2.1
observer/claim revision advancement, or parent-policy drift.

The new prerequisite is [`E2-RM — authenticated accepted-work receipt-material
projection`](../contracts/dispatch-policy-commitment-v1.md#e2-rm--authenticated-accepted-work-receipt-material-projection-prerequisite).
It is an E2-owned, read-only, behavior-neutral lookup/projection and is not B2.2 behavior. This
documentation correction specifies but does not admit, dispatch, implement, or complete it.

B1 continues to own acceptance and runtime-acknowledgement truth; B2.1 continues to own observation,
journal, replay, and terminal truth; E2 continues to own immutable policy commitments and retained
caps. B2.2 alone later owns foreground receipt construction and return from the authenticated
`E2-RM` result. B3.2 retains all remaining receipt, messaging, manifest, and lifecycle work; B4
retains receipt-targeted cancel/inspect/stop; and D1/E3 retain their later execution-envelope and
config-projection fields. C2/C3 and all other authority remain outside this correction.

B2.2 requires a fresh admission and explicit dispatch only after `E2-RM` has its own fresh
admission, bounded implementation, Linux acceptance evidence, independent final `CLEAN` review,
commit, and exact commit/tree identity. The B2.2 re-admission must consume
`AuthenticatedAcceptedWorkReceiptMaterialV1` without mutation or reinterpretation, prove that the
same original material is used after response loss/restart/B2.1 advancement/parent drift, and
re-establish B2.2's own file and behavior fence against the then-live baseline. Landing `E2-RM`
does not itself admit B2.2.

### Second `E2-RM` prerequisite correction (2026-09-03; controlling)

**Integration status qualification (2026-09-17):** The second correction below remains
controlling for read semantics, ownership, and the bounded implementation/proof fence. Its
“future”, “unadmitted”, “unimplemented”, and B2.2-blocked statements record 2026-09-03
scheduling and proof status, not current blockers. E2-RM and B2.2 are now
implemented/review-clean/landed as recorded above; separate canonical terminal closures
were not located. E3-A–D landed; [E3-E is closed](e3-agent-config-projection-and-gateway-adoption.md#e3-e-terminal-closure-2026-09-17);
enclosing E3 remains incomplete, and E3-F remains unadmitted and undispatched. This
qualification creates no proof result, closure, admission, or new implementation authority.

The first prerequisite record above remains the reason B2.2 is blocked, but E2-RM's authority is
now corrected and bounded as a specification: it must capture the existing E2 commitment store
and B1 receipt registry under one existing HSA root lock through a new non-reconciling,
descriptor-relative no-follow read transaction. The aggregate snapshot retains exact HSA
state-root bytes and the store-wide root revision as transaction-stability evidence, not receipt
material; it selects or consults no per-session current authority revision. A cleanly absent E2
directory is absence; an existing incomplete E2 layout is an error. Validated HSA/E2/B1
temporaries fail closed and remain untouched. The read creates, reconciles, cleans, writes,
publishes, repairs, migrates, backfills, rotates, renames, unlinks, and `fsync`s nothing.

The B1 lookup is keyed only by `(authority_store_id, acceptance_record_id)` and yields an opaque
witness directly on success, constructed solely from a fully validated canonical durable registry
snapshot. An absent registry, no matching record, violated uniqueness, or a store mismatch returns
`RegistryAbsent`, `AcceptanceRecordMissing`, `AcceptanceRecordAmbiguous`, or
`AuthorityStoreMismatch`; unsafe, partial, corrupt, malformed, or unsupported material returns its
typed physical/authentication error. Caller-provided B1 material is only a lookup selector and
complete equality expectation; it supplies no returned field. Every independently persisted B1
field has a typed mismatch classification, including distinct `AuthorityRevisionObserved` and
`AcceptedAt` results. `accepted_at` and `runtime_acceptance.observed_at` remain independent
persisted values. Historical B1 `authority_revision_observed` is never compared to current HSA
authority. E2 provides the preserved historical claim, policy snapshot, and retained-cap material;
the resolver exact-joins all store/request/subject/session/caller/backend/world/work/correlation/
acceptance/policy bindings and never supplements history from current B2.1, retained-worker,
current HSA authority, or parent-policy state. Missing B1 is never a successful result, E2 legacy
compatibility, synthesized evidence, or a reason to consult current B1/B2.1 state.

B2.2 remains blocked and unadmitted until this corrected `E2-RM` contract receives a separate
fresh admission and explicit dispatch, is implemented inside its exact six-file fence, passes its
future proof wall, and lands review-clean with exact commit/tree identity. This documentation
landing passes none of those implementation tests. A later B2.2 admission may consume only the
opaque authenticated projection and must separately authorize receipt construction/return. E3 is
independently specified at `81cfd33d4c5d16c31c837eeddff769995c566570` and remains unadmitted,
undispatched, and unimplemented.

## Preserved pre-correction row (chronology only)

The current disposition above supersedes only this row's admission and prerequisite interpretation;
the extracted header and row remain byte-identical chronology.

| Packet | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B2.2 — Foreground receipt return** | Switch long-running ingress surfaces from a blocking compatibility wait to returning the already-durable, E2-complete receipt after supervisor handoff. | WorldDispatchControl for the ingress contract, consuming ReceiptRegistry and Supervisor truth unchanged. | `02` WorldDispatchControl/ReceiptRegistry/Supervisor rows; `04` receipt acceptance; `05` blocking receipt rows; tool-invocation async model. | RuntimeToolInvocationAdapter; InternalToolboxTransport; B3.2 retained lifecycle; B4 cancel. | `crates/shell/src/execution/orchestrator_world_dispatch.rs`; `crates/shell/src/execution/agent_runtime/{dispatch_contract.rs,tool_invocation_contract.rs}`; receipt-response consumption only in `crates/shell/src/repl/async_repl.rs`; focused early-return tests colocated in those exact files. | No new receipt identity or policy commitment, no journal/reconciliation change, no obligation semantics, no A2 episode demotion, and no retained lifecycle rewrite. | `run_world_task` and `continue_world_worker` return the exact accepted receipt only after durable B1 acceptance, E2 policy completion, and B2.1 handoff and before terminal exit; caller drop does not stop observation; blocking compatibility may remain only on explicitly named non-core UX. | `RG-RECEIPT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`. | A1, A2/A3, E2, B2.1, and C1. | B3.2 lifecycle/park completion, B4 cancel outcomes, C2/C3, and unrelated surfaces. |
