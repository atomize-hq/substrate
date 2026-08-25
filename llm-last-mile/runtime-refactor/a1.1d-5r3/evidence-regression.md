**Kind:** evidence/regression
**Stable ID:** `A1.1d-5R3-family`
**Canonical for:** R3 planned proof and regression ledger
**Status:** canonical
**Authority scope:** exact extracted 05 evidence/regression source body only
**Source span:** [`05-debug-regression-ledger.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling`](../05-debug-regression-ledger.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling) lines 3194–3561
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R3 planned proof and regression ledger

## A1.1d-5R3 planned proof and regression ledger (archived for active scheduling)

Nothing in this section is executed evidence. It preserves former proof obligations as archived
engineering history and is superseded for active scheduling by
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md). It froze obligations at planning
source `4ceecd50e20d822dda7cbd8f0e1bef4ccad65d8e` / tree
`8ed5dc7a354b731016a103b68091864b6a09223a`. Planning is complete at
`19c40d41679e843e3e524f64fb9827959849d33e` / `d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with
planning fingerprint `sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`.
R3 implementation is `PARKED_BY_USER`, no implementation task has been dispatched from this plan,
and the just-closed corridor head was
`AUTHORITY_REQUIRED:B1_B2_1_JOINT_CLOSEOUT`; B3.1, C1, and the bounded internal A1.2b packet are
complete on the bound Tuesday, August 4, 2026 candidate, and the then-current historical gate was
`AUTHORITY_REQUIRED:R3_RESUME`. That former gate and proof program are not current prerequisites;
the later reentry was first selected as
[A1.3 Linux-first packet](../linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md), later
narrowed by the held
[A1.3-P0 Linux-first preparatory packet](../linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md),
and is now corrected to the active
[A1.3-P1 Linux-first atomic public-adoption packet](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).
No `cargo test --workspace` expected-failure inventory is frozen here, and approximate
workspace-failure counts are not authority.

### Source-closure and risk ledger

GitNexus was refreshed to the exact planning base. Query/context/impact still under-resolved shell,
PowerShell, cfg-specific paths, generated projections, and implicit `Drop`, so manual source
closure is binding:

- `ensure_private_substrate_home` is HIGH (16 graph-visible direct callers; the manual production
  chain reaches shell routing and scaffold intake);
- `ShellConfig::from_cli` is HIGH (8 direct/10 total graph-visible) and the edit fence is only its
  shim action block;
- `ShimDeployer::ensure_deployed` has a production chain to `run_shell_with_cli` and must be
  treated at least MEDIUM despite variable graph counts for private helpers;
- `create_ssh_uds_forwarding`, `ForwardingHandle::drop`, and
  `MacLimaBackend::ensure_forwarding` appear LOW or partial in the graph, but cfg reachability,
  implicit destruction, and activation make them manual HIGH-review action boundaries;
- `MacLimaBackend::new_with_mapping` and existing IH/PM validation are frozen; any need to edit
  them stops the macOS packet; and
- every Bash/PowerShell deletion, installer/uninstaller child, generated projection, service/
  platform call, and native action must be manually caller-closed again at its implementation
  base.

The planning-base manual closure specifically found and fenced Linux service enable/stop/socket/
ACL/start actions after the old daemon-reload boundary; the complete `lima-stop.sh` body (which
has no `VM_NAME=` assignment); Windows release/dev creator and destructor ranges, WSL warm stale
PID plus the unconditional live guard and both provisioning health branches; and Unix dev/release
creation through final state writes, the live owner-helper kill tombstone, and release temp-root
creation/registration/extraction/cleanup. Windows closure also begins before the two current log-
directory creations and before release `$tempRoot` creation. The WSL mapping
resolver already requires an exact registered distro machine ID, so the import branch cannot be
made authoritative and is a preserving tombstone. The Lima path also reaches group/membership,
guest binary, private-home, unit/service, layout, and in-guest DNS/toolchain mutations; the latter
are excluded passive-health/build fallback and are tombstoned. It also found that Cargo dist and
dev build/staging omitted all lifecycle executors. The exact
replacement/delivery owners are in `03` and `02`; a later implementation that leaves any legacy
action reachable or any executor unbuilt/unstaged fails its allowlist gate.

Any new HIGH/CRITICAL impact, consumer outside the frozen fence, destructive call, selector, or
authority source is a blocking plan contradiction until explicitly re-authorized.

### Candidate and manifest matrices

| Matrix | Required positive cases | Required preserving negative cases |
|---|---|---|
| Private-home rollback | current attempt created; retained parent; no-follow child open; exact identity rejoin; exact owner/`0700`/ACL; empty; one descriptor-relative removal; clean rerun | `AlreadyExists`, pre-existing, unknown provenance, replaced, nonempty, wrong type/owner/mode/ACL, inaccessible or ambiguous lookup, symlink, descriptor/path mismatch, unsupported identity; bytes/metadata unchanged |
| Manifest parse/join | exact schema/version/independent ID/digest/context/principal/platform/instance/closed role/object/type/disposition/metadata; exact locator/head/current generation and first/repeat read | missing, malformed, duplicate, unknown, tampered, coherent forgery, valid-old replay, partial, unsupported, cross-prefix/principal/platform/instance, locator substitution, unprivileged publisher, parent replacement, role scope escape, wrong object/bytes/target/mode/owner/ACL/security descriptor |
| Protected publisher | direct-terminal exact authorization; verified source receipt plus `ExecutorBuildEvidenceV1`; Unix socketpair or Windows UAC-pipe peer attestation; exhaustive `PublisherBootstrapComponentRoleV1` identities including every creatable parent/container and per-role exact absent-create versus exact pre-existing-dependency disposition; complete `LifecyclePublisherProtectedStateV1` wrapper CAS; null-slot bootstrap-core digest followed by precommitted Ed25519 harness key/external-store descriptor/test-retirement digest and harness-only pre-host reservation of zero Linux or one MAC/WIN guest tuple in final bootstrap and generation one; durable intent/executor/endpoint/key/counter-zero/active/generation-one sequence; fixed Linux `SOCK_SEQPACKET`, macOS XPC, or Windows named-pipe service; one framed request/response; nonce replay join; protected-host P-256-signed one-use guest pairing ticket carrying canonical SPKI; full SPKI hash/challenge/literal pinned through independent host and guest controlling terminals before ticket verification; host-protected one-use guest-mutation grant before the first guest effect; durable host record plus atomic root-only guest intent inside the enumerated state root with confined seed/nonce, signed hello/transcript, later final-key materialization, exact host+guest anchor/consumption join, and null-slot precommitted exhaustive guest-before-host test retirement; exact reserved/ticket-issued unused revocation proving every target's precommitted before-state and zero guest effect, with protected-host-P-256 proof, externally hashed/fsynced proof identity, precommitted-harness-Ed25519 acknowledgement, externally hashed/fsynced acknowledgement identity, and both hashes in protected CAS before pairing-record restoration/removal | missing/wrong confirmation, file/env/script carrier, wrong/expired source receipt, wrong build-evidence source/lock/toolchain/target/hash/file or code identity, staging-path privileged execution, pre-existing publisher identity, absent/pre-existing dependency disposition mismatch, partial/mismatched/omitted/unlisted-parent component, wrong component role/target/dependency, wrong peer PID/SID/uid/image/macOS designated requirement/Windows file identity/hash/DACL/framing, extra/oversize/truncated frame, concurrent/conflicting nonce, same-client coherent rewrite, valid-old anchor, detached prepared digest, protected-wrapper/counter rollback, clear-before-final-anchor, user keystore/DPAPI fallback, caller path/action, batch/unknown role, cross-build/instance; unreserved/extra/reused/omitted guest tuple, second evidence ticket, nonterminal reservation at host teardown, liveness/PID/timeout/pathname unused proof, guest effect without signed consumption grant, pre-existing target treated as absent, wrong unused-proof/acknowledgement key, algorithm, hash, file/parent identity, fsync, reservation/ticket/pairing record, dispatch/scope/nonce, replay, unknown field, self/future digest, missing acknowledgement, early pairing-record removal, final-bootstrap or guest-ticket digest cycle, core/final non-commitment-field mismatch, later-created/substituted retirement authorization, harness key/algorithm/encoding, omitted retirement role, early seed-intent/protected-state removal, or external directory; malformed/substituted SPKI, wrong curve/algorithm/encoding, SPKI/fingerprint or anchor-key mismatch, DER/high-S/malleable signature, request-supplied key, unsigned/wrong/expired/replayed ticket, non-TTY or automated pairing, channel-only authority, non-byte-identical child/channel retry, substituted nonce/transcript/intent/key, named-temp/rename intent publication, effect-visible/identity-record-not-durable component gap, ambiguous restart; zero state change |
| Closed role oracle | every serialized variant in `R3-MANIFEST-01` derives exactly its table domain/target/type/dependencies/actions and rejects every other action | arbitrary absolute/relative path, unknown binary/profile/unit/service/instance/pipe, wildcard, legacy name, alternate transport, wrong domain/object/action, role/manifest target mismatch |
| Publication | temp write, temp fsync, exact destination/generation/head join, atomic rename, parent fsync, head CAS, durable retry | temp symlink/nonregular, parent replacement, destination race, generation mismatch, head rollback/CAS conflict, short write, any sync/rename failure; no deletion authority |
| Action/retry | exact current action finishes or restores; manifest-preallocated receipt ID/name; protected prepared-record/counter; publisher-signed receipt without self/future digest; receipt write/fsync/parent-fsync; verified external hash; append-only `ManagedActionReceiptIndexV1` CAS; head CAS; publisher-anchor CAS; crash/retry at every boundary; committed repeat is a no-op | missing/unplanned/extra/unsigned receipt, filename/hash/file-parent identity mismatch, same-principal substitution, wrong signature key/algorithm/counter/prepared record/action observation, receipt self digest, index/head/anchor revision mismatch or rollback, replacement, ambiguous effect, stale PID, liveness-only join, nonempty state, unowned shared object; preserve and classify |
| Shared claims | first claimant records before-state; compatible second claimant joins; creator-first and creator-last uninstall; last-claim restoration/teardown | conflicting desired state, missing claimant, stale generation, claim removal before consumer release or teardown receipt, one-prefix shared removal, inferred transfer/adoption |
| Legacy collision | none: R3 has no adoption producer or acceptor | every `adopted` value and every unmanifested/pre-R3 collision, even matching bytes, remains unchanged and yields `BLOCKED_SCOPE_EXPANSION` plus manual-adoption handoff |

Candidate kill points are immediately before and after create, first no-follow open, identity/
metadata/ACL validation, cleanup arm, empty proof, removal, and acceptance. Manifest/action kill
points are before and after temp create, write, temp fsync, publication rename, parent fsync,
`ActionStarted`, every destructive action, effect observation, restoration, receipt temp write,
receipt signature, receipt fsync, receipt rename, receipt parent fsync, signature verification,
external receipt hash, receipt-index temp/
fsync/rename/parent-fsync CAS, head CAS, publisher-anchor CAS, and final commit. Every kill point runs exact
retry and verifies the permitted join/finish/current-attempt rollback/preserving-stop branch.

### Lifecycle convergence and preservation matrices

Every platform runs default A and custom A under hostile ambient B, two-prefix A1/A2, and unrelated
sibling sentinels. The matrix includes:

- first install, repeat install, partial failure at every action boundary, exact retry, first and
  repeat uninstall, uninstall after partial failure, uninstall/reinstall, and inverse-order
  two-prefix uninstall;
- exact link and target as separate objects; payload/version/bin/cache/profile file bytes, target,
  mode, owner, ACL/security descriptor, flags, and timestamps where authoritative;
- unit/drop-in bytes, enabled/active state, service identity, socket identity/ACL, runtime/state
  trees, helper/gateway/shim identities, group existence, membership, ACL bridge, linger, process/
  forwarder identity, pipe, VM/distro identity, and platform-control state;
- created and pre-existing-preserved dispositions, plus preserving rejection of every
  schema-reserved `adopted` input with `BLOCKED_SCOPE_EXPANSION`; and
- prohibited wildcard/prefix deletion, ambient-home selection, recursive candidate cleanup,
  follow-links/reparse behavior, broad process kill, stale-PID/timeout authority, unowned VM/WSL
  teardown, and shared-parent cleanup.

Mac cases separately cover guest group/membership, exact private home, all four guest binaries,
unit bytes, service/socket enabled and active state, runtime/state directories, fixed
`/etc/substrate-lima-layout`, A-local `lima_known_hosts` line/file before-state, rendered-unit
current-attempt temp tree, staged tree, and forwarding sockets/process. A missing exact evidence-
built binary
must produce zero guest change; any attempt to build in guest, install a toolchain/package, rewrite
`/etc/resolv.conf`, or restart DNS services is a failing prohibited-action oracle. Mac absent-
instance cases additionally kill before/after stage-one anchor, create, first identity
observation, second identity observation, PM finalization, manifest publication, and stage-one
close; wrong profile/control root, pre-existing instance, identity drift, partial create, or
unfinalizable PM preserves state. Windows cases
begin with an already-registered exact PM-bound distro and prove both unhealthy and healthy guest-
refresh branches; absent distro, download/import, WSL install-tree deletion, and unregister are
preserving scope-expansion oracles. Unix and Windows release cases kill at current-attempt temp-tree
parent/root open, creation, cleanup arm, each checksum/bundle/payload/extracted-child registration
and write, each reverse handle/descriptor-relative removal, empty proof, and root removal; abrupt
death residue is preserved. Windows additionally proves exact `A\forwarder\logs` before-state,
creation/restoration under hostile `LOCALAPPDATA`, and preserving pre-existing, replaced, or
reparse directories for both forwarder entry points. Unix `--kill-live-processes` cases require
`BLOCKED_SCOPE_EXPANSION` before enumeration or signal, preserve the hidden owner helper, and reject
name/PID/executable inference; a separately observed live consumer blocks uninstall.

Success requires an exact pre/post parity manifest for every unrelated or pre-existing object and
no unreceipted managed change. Product uninstall retains only the exact
`Uninstalled` lifecycle capsule/head/executor/receipts named in `04`; repeat uninstall and
reinstall must join it. Disposable publisher proof requires an external descriptor-bound
`PublisherTestRetirementAuthorizationV1` before bootstrap. The harness first creates its key and
descriptor-bound external directory, durably signs the authorization, and commits their exact
identities/digest into bootstrap and the generation-one anchor. The publisher accepts only those
byte-identical precommitted values and signs the exact retirement receipt without an in-record self
digest; the harness fsyncs its canonical bytes, computes the external artifact SHA-256, then signs
and fsyncs `PublisherTestRetirementAcknowledgementV1` binding that hash before reverse-order
component removal. Every kill
point before/after retirement receipt serialization/write/fsync/external hash, acknowledgement
sign/write/fsync/return/verification, publisher acknowledgement record, stop,
endpoint/registration/current-anchor/key/
executor/created-empty-parent removal exact-joins that receipt, and baseline parity precedes the ordinary
evidence receipt. Without that authority, test teardown is forbidden.

### Baseline and product behavior wall

Before a native run, the harness builds the exact platform host executor and, where Lima/WSL is in
scope, the Linux guest executor from the remote-equal source and lock state in an isolated build-
only scope. It records source/lock/toolchain/target/artifact SHA-256/file identity and platform code
identity in `ExecutorBuildEvidenceV1`, removes that scope, and proves build-state absence. It then
captures the clean checkout commit/tree/ref/status/divergence, OS/architecture,
tool versions, current principal, platform mapping, mounted/filesystem/security features, services,
units, sockets/pipes, processes, groups/memberships, ACL/security descriptors, linger, relevant
prefix and platform-control trees, registered VM/distro identities, and unrelated sentinels. The
capture is canonicalized, hashed, and stored before mutation.

An exact-absence publisher-bootstrap subcase additionally requires a disposable dedicated scope,
an absent-component baseline, and an external retained-descriptor retirement directory created by
the evidence harness before product bootstrap. The harness durably publishes the test-retirement
authorization first and treats its signed retirement receipt as the only authority for teardown;
ordinary product lifecycle never exercises retirement. A native host that cannot provide this
isolated scope reports the full platform handoff rather than weakening the bootstrap gate.

Every evidence task must discover product project
`2ccb802f-301c-4af4-9bd5-51d22808f0a2` by exact origin plus target ref and join its freshly bound
dispatch nonce, source task thread/host, evidence task thread/host, return meta thread/host,
evidence ID, and source commit/tree/ref. Those fields are part of the artifact and receipt. After
both validators pass, the task sends the native JSON to the exact return meta task with
`send_message_to_thread` as its final tool action. A missing/mismatched correlation field is invalid
evidence, not a hand-waved routing detail.

The product smoke must preserve:

- filesystem isolation/diff and network behavior;
- policy/configuration/dependency resolution;
- gateway and world-service transport;
- runtime, shim, replay, trace, and diagnostic behavior;
- PTY and non-PTY execution; and
- exact selected-context behavior under hostile ambient B.

It does not import passive health/world-deps remediation, authenticated Codex execution, retained
worker/task/session work, gateway adoption, or direct-member architecture. After failure or
success, execute the recorded restoration plan, invalidate sudo credentials on Linux, prove every
owned test object absent or restored, prove every unrelated/pre-existing sentinel exact, recapture
the baseline, and require canonical digest parity. Static review is never native evidence.

### `R3-NATIVE-LINUX-01`

- **Gates:** `A1.1d-5R3-CLOSEOUT`; also required by `R3-LIFE-01` and
  `A1D5I-ENV-01`.
- **Source checkpoint:** exact landed `A1.1d-5R3-UNIX` commit/tree on
  `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`, with live remote equality,
  required predecessor ancestry, clean checkout/index/untracked set, and `0/0` divergence.
- **Discovery/prerequisites:** dedicated supported Linux host; exact product repository discovered
  by git top-level plus origin/ref/commit/tree checks; systemd, required filesystem/ACL features,
  supported Rust/toolchain, authorized interactive sudo, and read-only discovery plus a restored
  same-filesystem disposable proof that the enumerated `/var/lib/substrate` state root supports
  root `O_TMPFILE`, `linkat(AT_EMPTY_PATH)`, file/parent fsync, and exact unlink; baseline proves no concurrent
  installer/lifecycle process.
- **Allowed actions:** only manifest-bound install/repeat/partial/retry/uninstall/reinstall,
  service/socket/gateway/world product smoke, exact test-created group/membership/ACL/linger and
  synthetic-auth scenarios, the exact harness-owned atomic-intent capability probe, exact
  disposable publisher bootstrap plus externally receipted test retirement, and recorded restoration.
- **Prohibited:** passive-health remediation, unrelated service/account/network changes, broad
  kill, ambient-home or unmanifested deletion, reuse of a nondedicated host, retained sudo
  credential, or restoration by reset/clean.
- **Evidence:** main artifact
  `review-control/r3-native-linux-01-evidence.json`, receipt
  `review-control/r3-native-linux-01-receipt.json`, schema
  `substrate.r3-native-evidence` v1, canonical SHA-256 digests, ordered command/output digests,
  baseline/restoration digests, product behavior results, unrelated sentinels, exact absence of
  service-to-socket propagation, protected world and disposable-publisher
  service-state/coupled-endpoint prepared transactions and receipts, endpoint non-requestability,
  missing/replaced/ambiguous endpoint zero-change,
  activation/observation/receipt kill-retry joins, service stop/restart/restore socket and endpoint
  nonmutation, and terminal `EVIDENCE_CLEAN`, including non-null
  `intent_publication_capability` with probe cleanup/parity.
  The receipt is native `codex.top-level-evidence-receipt.v1`, validated by
  `validate_evidence_receipt.py`; the evidence artifact's gated successor is exactly
  `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-MAC-01`, and receipt source live remote equals its source
  commit/tree/ref.
- **Restoration:** exact unit/drop-in/service/socket/runtime/state/helper/gateway/group/
  membership/ACL/linger/prefix/home parity; `sudo -k` and verification; clean repository and host
lifecycle state. Any mismatch blocks. Unavailable host yields the complete
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` handoff.

### Packet-local provider evidence gates

These gates prevent a later platform packet from starting until the preceding provider has been
published, exercised on its native platform, and closed with evidence-only repository bytes. The
implementation packet first makes one reviewed product/test commit and one normal fast-forward
push. A separate evidence task then discovers a clean checkout at that exact published
commit/tree/ref and requires `source.live_remote == source.commit`. It changes no repository byte,
creates no commit, and validates the native `codex.top-level-evidence-receipt.v1` with
`orchestrate-top-level-tasks/scripts/validate_evidence_receipt.py <receipt>`. That skill validator
has no successor flag. The evidence artifact separately binds the exact gated successor and is
validated with `scripts/ci/validate_r3_native_evidence.py` plus the exact expected evidence ID,
source commit/tree/ref, and gated successor; the receipt's evidence digest must equal the validated
artifact digest through exact receipt field `evidence.artifact_sha256`. Only a later platform-
closeout packet may commit the evidence artifact and receipt.

All three use `substrate.r3-native-evidence` v1, the baseline/action/artifact/restoration/digest
schema in `04`, the same platform discovery and allowed/prohibited clauses as the corresponding
final gate, terminal `EVIDENCE_CLEAN`, and exact checkout/host restoration. A provider gate
exercises only its packet fences plus run-only regressions; it does not satisfy the later post-UNIX
final gate.

| Evidence task | Exact published source | Evidence/receipt paths committed only by closeout | Artifact gated successor | Closeout task-receipt successor |
|---|---|---|---|---|
| `EVIDENCE:R3-LINUX-IMP-01` | landed LINUX commit/tree/ref, live remote equal | `review-control/r3-linux-imp-01-evidence.json`; `review-control/r3-linux-imp-01-receipt.json` | `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT` | `AUTHORITY_REQUIRED:A1.1d-5R3-MAC` |
| `EVIDENCE:R3-MAC-IMP-01` | landed MAC commit/tree/ref, live remote equal | `review-control/r3-mac-imp-01-evidence.json`; `review-control/r3-mac-imp-01-receipt.json` | `AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT` | `AUTHORITY_REQUIRED:A1.1d-5R3-WIN` |
| `EVIDENCE:R3-WIN-IMP-01` | landed WIN commit/tree/ref, live remote equal | `review-control/r3-win-imp-01-evidence.json`; `review-control/r3-win-imp-01-receipt.json` | `AUTHORITY_REQUIRED:A1.1d-5R3-WIN-CLOSEOUT` | `AUTHORITY_REQUIRED:A1.1d-5R3-UNIX` |

If a platform or privilege is unavailable, the evidence task returns
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` with the published OID/tree/ref, checkout discovery,
prerequisites, allowed/prohibited actions, evidence schema/digests, restoration commands, and
gated successor. An available run that fails or does not restore returns
`BLOCKED_NATIVE_EVIDENCE`. The closeout packet and every later implementation packet are forbidden
in either case.

### `R3-NATIVE-MAC-01`

- **Gates:** `A1.1d-5R3-CLOSEOUT`, `R3-LIFE-01`, and the macOS portion of
  `A1D5I-INSTALL-08`.
- **Source checkpoint:** the same exact landed `UNIX` commit/tree/ref and clean remote-equal
  checkout.
- **Discovery/prerequisites:** native supported macOS/architecture, supported Lima and SSH,
  repository identity checks, exact intended principal, existing or test-created Lima instance
  whose disposition and machine identity are recorded, exact IH/PM, clean platform-control root,
  unrelated instance/state sentinels, and a live human operator with independent host and guest
  controlling TTYs for the full pairing fingerprint/challenge/literal confirmation; read-only
  Lima guest filesystem discovery plus a restored same-filesystem disposable proof requires the
  enumerated guest `/var/lib/substrate` root to support root `O_TMPFILE`,
  `linkat(AT_EMPTY_PATH)`, file/parent fsync, and exact unlink.
- **Allowed actions:** manifest-bound candidate/home parity, Lima stage/unit/socket lifecycle,
  exact owned instance stop/delete only when disposition authorizes it, A-scoped SSH-UDS
  activation/unlink, `StreamLocalBindUnlink`, timeout kill/wait, handle-drop teardown, retry,
  disposable LaunchDaemon/System-Keychain bootstrap, signed one-use guest pairing ticket plus
  human-pinned guest bootstrap, the exact harness-owned guest atomic-intent capability probe,
  both reserved and ticket-issued unused revocation with protected-host proof, externally hashed/
  fsynced proof identity, harness acknowledgement, protected CAS, and exact Keychain pairing-record
  restoration/removal, and externally receipted
  test retirement, product transport, and recorded restoration.
- **Prohibited:** ambient `auto_select`, alternate endpoint/transport/instance/principal/prefix,
  unowned instance destroy, unrelated known-hosts/control-state mutation, or path-existence-only
  unlink; automated/stdin/file/environment pairing confirmation is also prohibited.
- **Evidence:** `review-control/r3-native-mac-01-evidence.json` and
  `r3-native-mac-01-receipt.json`, canonical v1 schema/digests, mapping and instance identities,
  socket/process/action receipts, product outputs, baseline/restoration parity, unrelated
  sentinels, non-null guest `intent_publication_capability` with probe cleanup/parity, unused-
  reservation proof/acknowledgement artifact hashes and Keychain CAS/removal observations, exact
  absence of service-to-socket propagation, protected host-publisher plus guest world/publisher
  service-state/coupled-endpoint prepared transactions and receipts, endpoint non-requestability,
  missing/replaced/ambiguous
  endpoint zero-change, activation/observation/receipt kill-retry joins, service stop/restart/
  restore socket and endpoint nonmutation, terminal `EVIDENCE_CLEAN`. The receipt is native
  `codex.top-level-evidence-receipt.v1`, validated by `validate_evidence_receipt.py`; the evidence
  artifact's gated successor is exactly `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-WIN-01`, and receipt
  source live remote equals the same UNIX source commit/tree/ref.
- **Restoration:** staged/current trees, units, enabled/active state, runtime/host sockets,
  forwarding child, known-hosts, instance/control root, prefix/home, unused or consumed pairing
  record, every reserved guest target, and unrelated instance state exactly restore. Both
  `Reserved` and `TicketIssued` unused branches prove exact baseline parity. Unavailable platform
  yields the full handoff and never a static substitute.

### `R3-NATIVE-WIN-01`

- **Gates:** `A1.1d-5R3-CLOSEOUT`, `R3-WIN-01`,
  `A1D5I-INSTALL-07`, and `A1D5I-INSTALL-10`.
- **Source checkpoint:** the same exact landed `UNIX` commit/tree/ref and clean remote-equal
  checkout.
- **Discovery/prerequisites:** native supported Windows and PowerShell 7; supported WSL; repository
  identity checks; exact current SID, Known Folder observation, registered distro name and
  machine-ID, pipe scope, clean two-prefix fixture, unrelated `.substrate*` and other-distro
  sentinels, and a live human operator with independent host and guest controlling TTYs for the
  full pairing fingerprint/challenge/literal confirmation; read-only WSL guest filesystem
  discovery plus a restored same-filesystem disposable proof requires the enumerated guest
  `/var/lib/substrate` root to support root `O_TMPFILE`, `linkat(AT_EMPTY_PATH)`, file/parent fsync,
  and exact unlink.
- **Allowed actions:** manifest-bound two-prefix install/repeat/partial/retry/uninstall/reinstall;
  profile/shim/bin/version cleanup; exact forwarder child/PID/image/start/pipe timeout and stop;
  exact already-registered PM-bound WSL instance start/stop/restoration and guest service
  lifecycle, including exact `/run/substrate`, group `substrate`, PM-mapped guest membership,
  exact PM-bound `SUBSTRATE_HOME`/host-commitment/mapping service-unit rendering, byte-exact
  root:root `0644` service/socket units with no implicit directory management,
  daemon-reload-before-state ordering, and
  root:`substrate` `0660` `/run/substrate.sock` creation/readback/restoration;
  disposable LocalSystem/CNG/HKLM bootstrap, signed one-use guest pairing ticket plus
  human-pinned guest publisher bootstrap, the exact harness-owned guest atomic-intent capability
  probe, both `Reserved` and `TicketIssued` unused revocation with exact LocalSystem-signed proof,
  externally fsynced proof hash, precommitted-harness acknowledgement and hash, protected HKLM
  reservation CAS, pairing-record restoration/removal, and externally
  receipted test retirement; product transport; restoration. Instance import, install-tree deletion, and
  unregister are never allowed actions.
- **Prohibited:** wildcard/profile sibling deletion, ambient/default prefix or distro selection,
  process-name kill, stale-PID authority, shared-state removal from one prefix alone, any import or
  unregister, unowned terminate, reparse following, unrelated WSL/platform state mutation, or
  automated/stdin/file/environment pairing confirmation.
- **Evidence:** `review-control/r3-native-win-01-evidence.json` and
  `r3-native-win-01-receipt.json`, canonical v1 schema/digests, SID/instance/machine-ID/pipe and
  object receipts, product outputs, baseline/restoration parity, sentinels, non-null guest
  `intent_publication_capability` with probe cleanup/parity, distinct service-template-source,
  PM-render-input, rendered-service, socket-source, and unchanged-installed-socket digests,
  rejection of installed template markers or a rendered/modified socket, hostile-ambient rejection,
  protected LocalSystem-publisher plus WSL world/publisher service-state/coupled-endpoint
  transactions and receipts, endpoint non-requestability,
  missing/replaced/ambiguous endpoint zero-change, activation/observation/receipt kill-retry joins,
  exact absence of service-to-socket propagation, service stop/restart/restore socket and endpoint
  nonmutation, unchanged state/runtime/lifecycle directory
  identities/metadata, unit metadata and daemon-reload/enabled/active observations, both unused
  branch proof/acknowledgement artifact hashes and protected HKLM CAS/removal observations, terminal
  `EVIDENCE_CLEAN`. The receipt is native `codex.top-level-evidence-receipt.v1`, validated by
  `validate_evidence_receipt.py`; the evidence artifact's gated successor is exactly
  `AUTHORITY_REQUIRED:A1.1d-5R3-CLOSEOUT`, and receipt source live remote equals the same UNIX
  source commit/tree/ref.
- **Restoration:** both prefixes, profiles, shims, bin/version/config/log state, forwarder/PID/
  pipe, WSL service/instance, `/run/substrate`, `/run/substrate.sock`, group/membership and socket
  owner/mode/ACL state, exact service/socket unit bytes/metadata and enabled/active state, Known
  Folder control state, both `Reserved` and `TicketIssued` pairing-record before-state and
  baseline parity, unrelated `.substrate*`, and other distros
  exactly match baseline. Missing native authority yields the full platform handoff.

### Final decision rule

`A1.1d-5R3-CLOSEOUT` may close only when all three evidence IDs bind the same landed `UNIX`
checkpoint, validate independently, report exact restoration and `EVIDENCE_CLEAN`, and the
cross-platform regression/review walls are clean with zero unresolved P1-P4. Any missing,
stale, cross-checkpoint, digest-mismatched, partially restored, or statically substituted evidence
leaves the corresponding gate open. R3 closeout does not authorize the later A1.1d/A1 program.
**Source provenance:** extracted from [`../05-debug-regression-ledger.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling`](../05-debug-regression-ledger.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling), baseline lines 3194–3561
**Relocation note:** repository-relative Markdown targets were rebased as needed to preserve their original repository destinations after relocation.
**Baseline span SHA-256:** `220f645a336d933b86bd6b350acd022ef66914607b9184aead88440033957364`
