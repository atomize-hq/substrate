**Kind:** packet
**Stable ID:** `B01-B03-P1`
**Status:** ADMISSION CONTRACT BOUND — proposed bounded implementation; source dispatch NOT AUTHORIZED; runtime unproved
**Canonical for:** initial same-host Substrate Workspace open/load/initialize boundary and exact-destination no-send preflight contract, proposed source fence, proof gates and admission assessment only
**Authority scope:** documentation/control-plane preparation; no source implementation dispatch, setup operation, runtime qualification or B1 acceptance
**Projection consumers:** [`current`](../../index/current.md), [`by-packet`](../../index/by-packet.md), [`by-id`](../../index/by-id.md), [`by-kind`](../../index/by-kind.md)

# B01-B03-P1 — authenticated exact-destination no-send preflight

## Admission disposition

The human product clarification on 2026-10-21 supersedes **B01-B03-P1-B01**, the earlier
setup-issuer prerequisite. Initially Collider opens Substrate Workspaces on the same host.
An existing configured Workspace loads through Substrate with its current known configuration,
policy, Lanes and destinations. There is no separate Connect Collider, enrollment, pairing,
client-key registration or issuer-approval ceremony. An absent Workspace offers Substrate
initialization/onboarding; that is where the user chooses and approves initial settings once.
Folder selection is discovery input, never Workspace identity or general operation permission.

**Admission contract bound** means the proposed owner entrypoints, minimal source fence, schemas
and required proof below delimit future implementation work. It does not mean source dispatched,
APIs implemented, owner signoff obtained or runtime ready. Final source-admission assessment is
separate from independent documentation review and TASK/META acceptance. No implementation,
initialization, installed IPC or consumer send is authorized by this documentation task.

The prior packet was independently reviewed with **no findings** and accepted as documentation
with NOT ADMITTED status under its earlier external-client setup assumption. The human correction
supersedes that product assumption and its blocker; it is not dismissal of a reviewer P2 and does
not rewrite prior acceptance records. Later external/third-party ingress may need a separate
protocol; it is outside this initial native path, not a prerequisite to ordinary Workspace open.

## Integrated outcome and authority set

For the initial native same-host Collider adapter, Substrate discovers and validates the selected
location, loads an existing Workspace, or returns an explicit initialization offer for genuinely
absent state. It supplies current authorized configuration/policy and owner-backed Lanes and
known destinations with capability/availability qualifications. The separate no-send operation
checks one **existing** exact HSA session/caller/backend/World and genuine Workspace-owned
Lane/World association. Its eligibility observation describes a checked read cut; it is never
an execution capability, reservation, accepted request or permission for a later send.

Load only the [selective-load protocol](../../foundations/per-slice-context-assembly-protocol.md),
[authority vocabulary](../../foundations/authority-vocabulary.md),
[reading/update rules](../../foundations/reading-and-update-rules.md),
[semantic status rules](../../foundations/semantic-status-labels.md),
[authority amendment](../../architecture/authority-map.md#proposed-b01b03-consumer-admission-responsibility-amendment--2026-10-21),
[composition boundary](../../seams/runtime-event-receipt-supervision-and-retained-runtime.md#proposed-b01b03-composition-boundary--2026-10-21),
[B1/B2.1 extension boundary](../../b1-b2-1/contracts-and-gates.md#proposed-b01b03-extension-boundary--2026-10-21),
[B01 §§3.2–3.4](../../../../docs/contracts/draft/collider-b01-b05-handoff.md#32-immutable-request-identity-and-the-admission-boundary)
and [B03 §§4.6–4.9](../../../../docs/contracts/draft/lane-world-binding.md#46-recommended-model-authorized-selection-not-session-rebinding).
Those accepted design responsibilities are target extensions, not current APIs or component-owner
signoff. This packet adds a distinct **preflight** operation before Submit/Reconcile; it does not
reuse their request/disposition family or redefine the completed B1/B2.1 acceptance contract.

**Hard exclusions:** no client request key, consumer correlation, proposal identity allocation,
association reservation/hold, historical acceptance publication, context resolution/material seal
or injection, `MaySend`, runtime admission gate/consume/fence, execute, task launch, delivery,
provider traffic or credentials. Initialization is a separately approved Workspace operation, never an implicit preflight effect.
No retained route, E3-F continuation, scheduler, Lane move,
custody transfer, filesystem/merge/backing selection, plural HSA binding, Mac/Windows source work,
Collider or Handbook changes. Lanes remain durable Workspace entities with multiple Worlds and
one current association per associated World; a session, path or branch is never a Lane identity.

## Source correspondence and current status

Authoring baseline: Mac branch `docs/substrate-collider-contracts`, HEAD
`aa5ef73ae087aa82d1f952ba6dd5c92361b8e24f`. The preceding five dirty documentation changes
are accepted prior-phase bytes, separate from this packet's delta. The present correction changes only their directly affected opening/ingress paragraphs;
request, full-send, retention and recovery rules remain intact. The [author observation](../../../../.codex/orchestration/2026-10-21-b01-b03-preflight-packet/evidence/author-preedit-observation.json)
records the protected baseline, narrow fence, owner release and attributed fresh Linux source
correspondence. The separately retained [source correction](../../../../.codex/orchestration/2026-10-21-b01-b03-preflight-packet/evidence/author-source-correction.json)
supersedes the original snapshot-integration inference using actual production callers. These
are evidence, not product authority or runtime results.

Fresh read-only Linux relay `/root/bind_linux_source` binds D
`/home/spenser/__Active_code/substrate` and W
`/home/spenser/__Active_code/.worktrees/substrate-runtime-refactor-product-authority-integration`
to HEAD `088f62d81c182b92cac404983b8bf512becc829c`, tree
`b40da914c1e5d73fc79f13c0946f6af9c97455d1`. D is clean; W retains exactly its eight unstaged
files. W binding `cd89226b984def497b8a67f92a329e430f9d2899f16cfd743d4f7db07c55f427` and diff
`efcd97c433e33bc8a5182d7fc4a8f7db4a1946625dfeed3ab252291e6785c8c5` match the retained stop.
This retained correspondence resolves the earlier source-location question only; it is not a new
Linux visit or IPC/runtime proof. Current narrow local Workspace source inspection is recorded in
[the author observation](../../../../.codex/orchestration/2026-10-21-b01-b03-workspace-preflight-correction/evidence/author-preedit-observation.json).

| Existing source owner / locator | Actual foothold and constraint |
| --- | --- |
| HSA [`facade.rs::resolve_current_exact`](../../../../crates/shell/src/execution/agent_runtime/host_session_authority/facade.rs), local line 1310 | Supplies strict exact session/caller/root/store/policy facts, but calls `store::read_opened_root_v2_or_v3` and semantic preflight. That path reconciles temps under exclusive root lock (`store.rs:1653`, `store/platform/transaction.rs:3682–3684`); it is **not** a nonreconciling read. |
| HSA [`store.rs::read_existing_accepted_work_authority_snapshot`](../../../../crates/shell/src/execution/agent_runtime/host_session_authority/store.rs), lines 83 and 862–877 | Linux-only raw HSA/E2/B1 snapshot via `with_opened_existing_versioned_read_only_snapshot` (`store/platform/transaction.rs:547+`), reused by the current accepted-work projection path (`orchestrator_world_dispatch.rs:1740`, `dispatch_policy_commitment.rs:2031`). The old dead-code annotation describing future integration is stale. This existing production use does not supply complete Workspace/current-authority or custody validation; B2.2 is not reopened. |
| [`StateStore::resolve_world_work_registry_authority`](../../../../crates/shell/src/execution/agent_runtime/state_store.rs), line 5321 | Resolves HSA then binds receipt registry and supervisor; preflight cannot call it and claim zero persistent effects. |
| Backend [`session.rs`](../../../../crates/world/src/session.rs), `ownership_from_metadata:672–717`, `read_canonical_metadata:1146–1158` | Existing private custody decoding is the narrow reader foothold. `find_compatible_session` recovery reaches `ensure_cgroup_attach_target:575–578`; it is not a pure custody lookup. |
| [`world-service/service.rs::exact_bound_world_ownership_adoption`](../../../../crates/world-service/src/service.rs), local line 3264 | Constructs ownership adoption used on execute; adoption publishes ownership. Neither execute nor adoption can bootstrap a read-only eligibility check. |
| HSA [`stop.rs::issue_stop` / `validate_issue_request`](../../../../crates/shell/src/execution/agent_runtime/host_session_authority/stop.rs), lines 108 / 392–419; broker [`api.rs::evaluate`](../../../../crates/broker/src/api.rs), line 40 | Exact PublicCli/current HSA/lineage plus explicit `session_stop` capability is an operation-specific authorization precedent. Command evaluation and that Stop capability grant no Workspace initialization or arbitrary execution permission. |

Line numbers are source locators, not cross-host byte identity claims. The Linux relay and Mac
baseline are separate source states; a later implementation must bind exact files/symbols at its
actual checkout, include fresh impact analysis, and retain current raw command/source evidence.
No seam is promoted to `ContractCorrectAndProven` by this packet.

## Substrate Workspace open, load and initialize

These are **proposed native owner adapters**, not existing exported APIs or a new authority
framework. Collider invokes Substrate; it does not parse/write `.substrate` or synthesize policy,
Workspace identity, Lanes or destinations. Authentication uses the existing same-host native
Substrate caller context; caller identity and trusted root/store context are constructed inside
Substrate, not accepted from browser/request fields. No custom enrolled-client/key handshake is
required for this initial route. A future cross-process binding must preserve that context and
privacy; generic World execute and the E3 preparation socket are not substitutes.

| Existing owner / actual source entrypoint | Supported foothold and minimum proposed change |
| --- | --- |
| [`execution/workspace.rs`](../../../../crates/shell/src/execution/workspace.rs), `find_workspace_root`, `find_workspace_root_any`, marker/policy/disabled path helpers | Existing discovery looks for `.substrate/workspace.yaml`, canonicalizes with fallback and searches ancestors; enabled lookup skips disabled roots. New `execution/workspace_open.rs::WorkspaceOpenAdapter::inspect_existing` must inspect the selected boundary first, preserve actual root identity and distinguish disabled, missing-marker, malformed, incomplete and absent states before ancestor/default fallback. A present but invalid `.substrate` cannot become an unrelated parent Workspace or an initialization offer. An ordinary subdirectory may resolve its genuine enclosing Workspace only when no intervening invalid/disabled boundary exists. |
| [`workspace_cmd.rs`](../../../../crates/shell/src/execution/workspace_cmd.rs), `handle_workspace_command`, private `run_workspace_init`; [`cli.rs`](../../../../crates/shell/src/execution/cli.rs), `WorkspaceAction::Init`, `WorkspaceInitArgs` | Current `substrate workspace init [PATH]` creates directories/internal Git storage, fills missing YAML and edits `.gitignore`, even without `--force`. It does not implement this product onboarding contract or create the complete Lane model. Extract only the minimum initializer writer for `WorkspaceOpenAdapter::initialize_absent`; require strict absence plus the user's exact approved settings, expose partial outcomes, never invoke repair/enable/force during open. |
| [`config_model.rs`](../../../../crates/shell/src/execution/config_model.rs), `parse_config_patch_yaml`, `resolve_effective_config_with_explain` / explicit-bootstrap-home counterpart | Existing sparse config parsing, precedence, validation and provenance are reused. New `WorkspaceOpenAdapter::load_existing` reads fresh validated owner inputs; stat-key cache hits are not a coherent preflight cut. Preserve existing config semantics and reject unsupported legacy state. |
| [`policy_model.rs`](../../../../crates/shell/src/execution/policy_model.rs), `resolve_effective_policy_with_explain_for_bootstrap_home`; [`broker/effective_policy.rs`](../../../../crates/broker/src/effective_policy.rs), `resolve_effective_policy_with_explain_from_global_source` | Reuse the selected trusted bootstrap home's global policy plus current Workspace patch and existing precedence. Existing `policy_cmd.rs::run_current_show` is a command-level precedent; a machine adapter returns typed authorized provenance/denial rather than parsing printed output. Do not change broker precedence or treat an `Allow` as general permission. |
| New `agent_runtime/workspace_admission_authority.rs`, `read_workspace_identity`, `read_current_lanes_and_destinations`, `read_current_world_association`, `resolve_current_read_access` | Proposed Workspace-owned identity/association read adapter alongside HSA; **absent as a complete supported source path**. Return genuine owner records and explicit partial/unsupported availability. Existing YAML marker proves discovery only; never derive a Workspace/Lane ID from path/session or seed fixture records. Missing genuine state is diagnosed; it cannot be silently initialized, migrated or regranted. |

Proposed native versioned semantics (names/layout are future source work, not current wire APIs):

```text
OpenWorkspaceV1 = { version, selected_location }
WorkspaceOpenReplyV1 = Loaded(current_owner_view, component_statuses)
                    | InitializationOffered(selected_location, proposed_settings)
                    | Denied | Disabled | Invalid | Incomplete | Unsupported | Unavailable
InitializeWorkspaceV1 = { version, selected_location, approved_initial_settings }
WorkspaceReadViewV1 = { workspace_ref, root_store_binding, config_provenance,
                        policy_provenance, lanes, destinations, owner_revision_vector }
```

`selected_location` is an untrusted discovery hint. The owner constructs root/store binding,
initiator and every returned reference. `Loaded` is useful current owner-backed state with
explicit component coverage, not destination eligibility; unavailable Lane/destination support
cannot be filled by UI fixtures. An invalid/incomplete required identity or configuration fails
load; an unsupported optional component remains visibly unsupported. Policy may prohibit reads;
unauthorized callers receive no target existence, private path, identity or configuration data.

- **Existing valid state:** load current known settings and owner-backed objects without another
  setup approval. Reopen repeats current validation and disclosure checks; it never replays setup
  or extends permission. Read-only open performs no hidden recovery or runtime launch.
- **Genuinely absent state:** offer initial settings and effects for one user approval. Denial or
  cancellation writes nothing. Substrate's initializer alone applies the approved settings; it
  does not obtain provider credentials, start execution or treat onboarding as a blanket grant.
  Any initial durable Workspace/Lane records must use their genuine Substrate owner schema;
  initialization need not manufacture a World or an execution destination.
- **Failed/interrupted initialization:** report effects known to have occurred and incomplete
  state. Reopen diagnoses that state; it does not call init again, fill missing files, infer
  success, regrant permission or silently migrate. Any later completion/repair is a separate
  explicit Substrate operation outside ordinary open/preflight and outside this packet's proof.
- **Existing invalid/disabled/incompatible state:** diagnose and refuse the affected load/read;
  no enabling, `--force`, parent/default-policy fallback, regenerated identity or automatic
  onboarding. Missing marker within existing `.substrate`, bad/unknown schema, unsupported legacy
  files, substituted/symlinked root, unreadable policy and partial owner state stay distinct.

## Current-read schema and stable cut

The proposed Workspace adapter returns an opaque owner-qualified `WorkspaceReadCutV1` containing
root/store incarnation, config/policy source revisions and protected content commitments, genuine
Workspace/association revisions, HSA snapshot identity and backend custody incarnation/revision.
This is internal evidence, not a bearer capability or exposed secret-derived digest. Config and
policy provenance includes the exact defaults/global/Workspace/environment/CLI inputs actually
used; the adapter neither invents new precedence nor assumes UI state is current authority.

The proposed strict `WorkspaceStateV1` record is Workspace-owned, separate from HSA: version,
opaque stable `workspace_ref`, canonical root/store binding, monotonic state/config/policy
revisions, Lane refs and exact World/generation→Lane association entries with monotonic association
revision and backend-custody ref. Proposed persistence is `.substrate/workspace-state.v1.json`;
this is a **new schema**, not an existing file or enrollment/grant directory. Initialization may
publish its genuinely new empty Lane/association collections with the approved initial settings;
it does not create an existing World or permission. Existing owner records need a supported exact
reader; absent/unknown/conflicting state is incomplete/unsupported, never synthesized or silently
migrated on open. Strict decoding rejects unknown/duplicate fields and unsupported versions.
The new Workspace owner serializes revision publication and config/policy updates with its
per-Workspace lock and recoverable atomic replacement; crash/partial state refuses eligibility.
Global-policy/config revisions remain with those existing owners, not copied into a second grant
store. No lock spans backend IPC. This specifies the minimum future record responsibility; actual
production support and interruption behavior require the PF proof below.

Use a bounded two-collect protocol: read fresh validated config/policy/Workspace/HSA snapshots,
read existing custody, then collect the complete owner revision/incarnation vector and current
read authorization again. Return eligibility only when every relevant invalidator is covered by
its owner's monotonic revision and both collects join unchanged. Config/policy owner writers
must advance the corresponding cut revision under their own persistence lock; new minimum
`config_cmd.rs` / `policy_cmd.rs` writer hooks are confined to that bookkeeping. Existing stat
caches, timestamps or equal final YAML bytes cannot prove no intervening A→B→A. Unmediated edits,
missing revision coverage or unsupported owner snapshot protection must produce changed,
unsupported or integrity failure; never repair metadata during a read. Exact revision storage
must stay in the Workspace owner's namespace, not add arbitrary files to HSA's closed layout.
A stable observation can become stale immediately afterward; later use reauthorizes independently.

## Proposed no-send operation and failure semantics

Proposed `WorkspaceOpenAdapter::preflight_exact_destination` consumes the native Substrate caller
context and an owner-qualified Workspace reference from loading, plus an exact expected Lane,
World/generation/association revision and existing session/caller/backend expectation. All client
selectors remain lookup hints. Reject unknown/duplicate fields, unsupported versions, empty
identities and execution/context/payload/request-key fields. The caller cannot supply authoritative
principal, root/store identity or grant. The initial route has no enrolled-client challenge state.

| Responsible owner / proposed read interface | No-send responsibility |
| --- | --- |
| Native Substrate caller boundary and Workspace `read_workspace_identity` | Bind actual native initiator and trusted root/store; validate genuine Workspace identity. No client enrollment or path-derived identity. |
| Workspace `resolve_current_read_access` with existing policy owners | Check current operation/resource and disclosure authority, exact initiator and HSA actor. Existing execution/history permissions remain distinct; loading does not mint delegation. |
| HSA `read_current_exact_nonreconciling` (new adapter) | Apply existing strict current exact session/caller/backend/World/root/store and policy validation to one verified read-only snapshot. No semantic-preflight reconciliation, registry/supervisor bind, bootstrap or repair. |
| Workspace `read_current_world_association` | Authenticate genuine association and monotonic revision/incarnation; compare expected Lane/World and revision without reserving or holding it. A→B→A cannot recreate an old revision. Siblings remain independent. |
| Existing backend owner `read_existing_custody_nonreconciling` (new export) | Read/authenticate exact physical World/generation, owning session/backend and custody revision/incarnation from existing protected metadata; no recovery/attach/adopt/create/cleanup. Missing/conflicting custody fails. |

Read current authorized owner snapshots, validate all exact joins, then revalidate the complete
relevant owner revision/incarnation vector and current read authorization. A positive result
requires a supported stable read cut: every relevant invalidator must advance its owner's
monotonic revision, and a bounded double collect must establish unchanged revisions across the
checked interval, or an equivalent owner-supported coherent snapshot must do so. No global lock
crosses backend IPC. Drift, unresolved temps/transactions, unsupported currentness coverage or
unreadable authority cannot yield eligibility. Return changed/unavailable/unsupported rather
than reconciling or repeatedly retrying until a desired answer appears. Changes after the cut
remain possible; observations carry owner revisions, never frozen permission.

Proposed semantic replies (not B01 request dispositions):

| Reply | Meaning and allowed disclosure |
| --- | --- |
| `EligibleObservation` | Authorized exact owner-qualified destination/actor and read-cut revision vector checked now. No bearer token, request/work identity, hold or permission for later use. |
| `Denied` / `InvalidRequest` | Privacy-filtered authentication/authorization or strict request failure. Unauthorized callers learn no target existence, principal match, association, refs or material. |
| `IneligibleObservation` / `ChangedDuringRead` | Authorized caller receives bounded stale destination/association, denied current delegation, incompatible posture or changed-cut observation where disclosure itself is authorized. |
| `Unsupported` / `Unavailable` / `IntegrityFailure` | Unsupported profile/owner evidence, unreadability/transport failure, or authenticated inconsistency. None means absent destination, safe resubmit, no past work or successful setup. |

No durable observation lookup, request reconciliation, background recovery, audit-triggered task
allocation or automatic retry follows a reply. Adapter/service loss requires a fresh current
read; it cannot execute. Bounded nonsecret diagnostics may name authorized opaque refs/revisions
but exclude private roots, prompts/material, private authentication material and raw
credential-derived hashes. Later Submit must authenticate and authorize again, allocate
its own original identity, reserve through owners and satisfy every then-current effect fence.

## Proposed later implementation fence

The following is a bounded source-admission contract, **not a source dispatch now**. New symbols,
records and test launchers are proposed and absent; existing entrypoints are identified above.

| Source owner / exact proposed entrypoints | Minimum work and later proof launcher |
| --- | --- |
| New `crates/shell/src/execution/workspace_open.rs::WorkspaceOpenAdapter::{inspect_existing,load_existing,initialize_absent,preflight_exact_destination}`; minimum registration in `execution/mod.rs` | Native typed open/init/read binding above, no external issuer/client-key framework. Proposed `cargo test -p shell --lib workspace_open_ -- --nocapture` must discover positive/reopen/denied/malformed/incomplete cases. |
| Existing `execution/workspace.rs` discovery/path helpers; `workspace_cmd.rs::{handle_workspace_command,run_workspace_init}`; `cli.rs::{WorkspaceAction,WorkspaceInitArgs}` | Strict selected-boundary classification and minimum approved-absent initializer extraction; preserve unrelated CLI/sync/checkpoint/rollback behavior. No implicit repair, World creation or provisioning. Proposed `workspace_open_` tests include partial init and cancelled/failed/reopen outcomes. |
| Existing `config_model.rs` fresh parse/effective resolvers; `policy_model.rs` trusted-home resolver; minimum `config_cmd.rs::{run_global_init,run_global_set,run_global_reset,run_workspace_set,run_workspace_reset}` and `policy_cmd.rs` corresponding init/set/reset writers | Reuse exact precedence/validation, provide cut-bound fresh inputs and minimum monotonic config/policy revision bookkeeping only. Existing broker resolver is consumed, not rewritten. Missing/unmediated revision coverage refuses eligibility. Proposed `cargo test -p shell --lib workspace_read_cut_ -- --nocapture` covers same-bytes ABA, cache invalidation, current-read denial and concurrent writes. |
| New `agent_runtime/workspace_admission_authority.rs::{read_workspace_identity,read_current_lanes_and_destinations,read_current_world_association,resolve_current_read_access,collect_read_cut}`; minimum parent registration | Typed existing genuine owner reads, no scheduler, grant issuer or duplicate HSA authority. Implement strict `WorkspaceStateV1` / `WorkspaceReadCutV1` and the named Workspace-owned revision storage; bind exact bytes at source start; no broad HSA-layout exception. Missing production state stays incomplete/unsupported. Proposed `workspace_read_cut_` tests plus real-owner IPC proof below. |
| HSA `facade.rs::read_current_exact_nonreconciling` (new), `store.rs` / `store/platform/transaction.rs` minimum snapshot access | Reuse production-adopted `read_existing_accepted_work_authority_snapshot` and `with_opened_existing_versioned_read_only_snapshot`; apply existing exact validation without temp reconciliation, B1/Supervisor bind, bootstrap or repair. Preserve transitions/codecs/root layout. Proposed `cargo test -p shell --lib preflight_ -- --nocapture`. |
| `crates/world/src/session.rs::read_existing_custody_nonreconciling` (new) and minimum `world/src/lib.rs` export | Export authenticated existing canonical metadata/ownership read under owner-supported snapshot protection; never call `find_compatible_session`, attach/create cgroup, adopt or cleanup. Proposed `cargo test -p world --lib custody_read -- --nocapture`. |
| Only if actual existing backend IPC requires it: `world-service` router/service plus versioned agent API model/client | Freeze one typed custody-read route and transport symbols before touching them; consume the export above with the same native caller/owner binding. No generic execute fallback/service gate/ownership transfer. Proposed real Linux test target below must bind this route if selected; wider access/layout changes return as exact source-scope conflicts. |

`B01-B03-P1-B02` remains **implementation and proof work**, not missing product authority: a
complete nonreconciling joined Workspace/current-authority/custody path does not exist yet.
These concrete adapters and exact read-cut coverage are what implementation must supply. They
do not revive B01 under an owner-TBD, renamed issuer or general management-framework prerequisite.

## Dependencies and required proof

| Item | Current disposition | Required implementation evidence |
| --- | --- | --- |
| B01-B03-P1-B01 separate setup issuer | **SUPERSEDED by human product clarification**, not a reviewer finding disposition. | Initial same-host configured open works without new enrollment/approval; absent initialization obtains settings approval once. |
| B01-B03-P1-B02 owner adapters/readers | **BOUNDED PROPOSED IMPLEMENTATION / PROOF REQUIRED.** No source dispatch or runtime readiness. | Actual Workspace identity/Lane support, strict root/config/policy joins, complete monotonic read-cut coverage, nonreconciling HSA/custody export and PF gates below. |
| Linux source correspondence | Retained resolved documentation evidence; no new host audit. | Rebind exact checkout/source/allowlist before later symbol edits; reuse valid evidence and obtain actual IPC proof. |

At a later authorized source start, bind exact candidate/schema/owner revision layout, actual
backend custody transport and test launchers to this fence; run required symbol impact analysis.
A concrete unsupported owner/export/layout conflict returns its path/symbol/required change for
assessment. API absence is assigned implementation work. No implementation follows this docs
correction automatically, and no fixture is accepted as genuine production Workspace/Lane truth.

| Gate | Required positive and negative proof on the exact source candidate |
| --- | --- |
| PF-OPEN | Real configured Workspace opens/reopens through Substrate with current known config/policy/Lanes/destinations and explicit component availability, with no Connect/enroll/pair/issuer approval or hidden repair. Nested/displaced/disabled/missing-marker/legacy/malformed/incompatible/unreadable states diagnose/refuse; no wrong-parent/default fallback or caller/path-derived identity. |
| PF-INIT | Truly absent state offers initial settings approval once. Denied/cancelled setup writes nothing; successful approved init returns genuine owner state and reopen does not repeat approval. Failed/interrupted/partial init reports known effects, remains incomplete on reopen and never auto-repairs/regrants. No runtime/World creation or arbitrary `.substrate` client write. |
| PF-AUTH | Native initiator/root/store binding is owner-constructed; spoofed principal/Workspace/context, wrong root/store and current policy/read denial fail without existence/material disclosure. Folder access does not grant execution, history, mutation or delegation. |
| PF-EXACT | Existing session/caller/backend/World/root/store/custody and genuine association joins succeed; mismatch, ambiguity, missing ownership and current operation/resource denial refuse eligibility. No session-as-Lane or fixture identity. |
| PF-REV | Genuine selected-World revision succeeds; stale A→B→A, config/policy/current-read/HSA/custody invalidation and incomplete state refuse eligibility. Prove bounded double collect plus every owner invalidator; unchanged sibling remains independent. Equal bytes/cache stats alone cannot pass. |
| PF-ZERO | Snapshot authority/config/policy/association/B1/B2.1/E2/backend metadata, directory entries/modes and process/cgroup/runtime effects before/after positive and every failed open/preflight. Zero durable lookup repair, setup/reconcile, identity/key/proposal allocation, reservation, acceptance, `MaySend`, execute, delivery, provider traffic, adoption/custody/cgroup change. Separately measure approved initialization's listed Workspace writes; they are not counted as read-only success. |
| PF-LATER | Change/revoke destination or permission after eligibility; observation grants no later send. No accepted-work/request ID or capability is returned; future submission reauthenticates/authorizes and satisfies all full-send fences. |

All following commands are **proposed later proof, not execution in this documentation task**.
The new filters above must discover nonzero intended cases. New integration target/launcher
`cargo test -p shell --test consumer_admission_preflight_linux -- --nocapture` does not exist yet;
it must exercise real Substrate-owned state and the actual native adapter/backend IPC route.
The designated ignored real-profile case is also absent:

```sh
SUBSTRATE_PREFLIGHT_PROOF_PROFILE=/absolute/protected/no-send-profile.json cargo test -p shell --test consumer_admission_preflight_linux test_real_workspace_open_init_and_existing_world_no_send -- --ignored --exact --nocapture
```

This test-only proposed profile selects selected location, trusted native adapter context, genuine
owner-issued Workspace/Lane/association references and an existing HSA session/caller/backend/World.
It contains no provider credentials, enrollment/issuer grant or client signing key. Bind its actual
parser, fixture distinction and installed launcher at source admission. Missing real owner state,
profile or selected IPC route must fail visibly, never skip or seed a production fixture. Exercise
PF-OPEN–PF-LATER with nonzero case counts and measured zero read-side effects. Initialization is
separately approved/source-bound and its partial writes are reported; no provisioning is assumed.

Retain command, cwd, baseline/candidate hashes, platform, actual exit and raw output for these
checks, `cargo fmt --all -- --check`, `cargo build -p substrate --bin substrate` and, if service IPC
changes, `cargo build -p world-service --bin world-service` plus `cargo test -p world-service`.
Before later source publication include applicable workspace build/clippy/tests and Linux
`substrate world doctor --json`, `substrate shim doctor --json`, `substrate health --json`.
Doctor output cannot replace PF gates; missing installed/platform evidence remains explicit.

**Later full-send dependencies:** preserve H1–H3, conditional H4/S1 context/material/use/delivery;
B1/B2.1/E2/B2.2 composition, B03 reservations/history and runtime consume/close/fence before real
submission. Applicable security/profile and D1–D3 requirements remain; E3-F/I1/I2 still govern
retained routes. These are not automatic predecessors to normal open/no-send reads. Existing
later external-client enrolled ingress remains outside this initial route. Preserve close-before-
effective-commit revocation and resolved B01B03-D01 original-material/unresolved-evidence custody
permission until positive resolution; opening/onboarding grants neither execution nor later use.

## Stop, review and publication endpoint

Stop for a newly demonstrated authority/layout/ownership conflict, drift in frozen source/protected
bytes, required API scope beyond the proposed fence, or inability to prove genuine owner state,
approved absent-only initialization and current exact read with zero read-side effects. Report
the exact owner, unsupported input/event, consequence and minimum choice/evidence; do not widen scope or weaken a
gate to manufacture eligibility.

The authorized present endpoint is independently reviewed **working-tree documentation** with
admitted-or-blocked disposition, exact delta/source correspondence and link/anchor/table/status/
fence/whitespace/protected-byte checks. Apply the
[development review contract](../../contracts/development-review-and-remediation-contract.md)
through the task's governing causal protocol; original findings and owner assessment remain
separate. No source dispatch, runtime test, installation, component-owner signoff, commit,
publication to a remote, integration, installed proof or Gate C follows from documentation
completion. TASK assessment and META acceptance remain separate decisions.

The [dated E3 correction](../e3-agent-config-projection-and-gateway-adoption.md#current-e3-f-status-correction--2026-10-21)
supersedes inherited “undispatched” current projections. E3-E stays closed; E3-F is paused after
dispatch, its W8 result unaccepted/unclosed, I1/I2 unresolved and `c381` published-unintegrated.
Older extracted/as-of status text remains chronology. Inherited navigation wording in
`slices/README.md`, Track E and E3 contract headers is nonblocking projection drift under that
canonical dated correction; it grants no continuation and is not rewritten by this packet.
