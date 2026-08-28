**Kind:** contract/gate
**Stable ID:** `A1.1d-5R3-family`
**Canonical for:** R3 lifecycle contracts and gates
**Status:** canonical
**Authority scope:** exact extracted 04 lifecycle-contract source body only
**Source span:** [`04-contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates`](../04-contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates) lines 8677–9999
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R3 lifecycle contracts and gates

## A1.1d-5R3 lifecycle contracts and gates

These contracts are archived normative planning evidence for the former packets indexed in `03`.
They are superseded for active scheduling by
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md). They
authorize no action during `A1.1d-5R3-PLAN`, they are not the current next implementation line,
and no R3 implementation task has been dispatched. Planning is complete at
`19c40d41679e843e3e524f64fb9827959849d33e` / `d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with
planning fingerprint `sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`.
At that historical checkpoint, R3 implementation was `PARKED_BY_USER` and resume would have
required fresh explicit authority plus live revalidation. That resume sequence is no longer an
active prerequisite.

### `R3-CANDIDATE-01` — exact synchronous candidate rollback

The only permitted private-home removal is equivalent to one
`unlinkat(parent_fd, child_name, AT_REMOVEDIR)` under all of these simultaneously true facts:

1. this process attempt received `PrivateHomeCandidateProvenance::Created` from its own successful
   create operation;
2. the trusted parent descriptor and child name used for that create remain live and unchanged;
3. a no-follow open of the created child succeeded and retained its exact device/inode or stronger
   supported physical identity;
4. the opened child validates as the intended principal's directory with exact `0700`, accepted
   ACL state, no replacement, and no link/reparse traversal;
5. an immediate descriptor-relative parent/name observation exact-joins the opened identity;
6. descriptor-relative enumeration proves the directory empty; and
7. acceptance has not published and cleanup has not been disarmed.

Failure of any fact leaves state unchanged and returns the original validation failure plus a
non-secret rollback classification. Cleanup is never recursive, never path-only, and never
best-effort success. `AlreadyExists`, unknown provenance, pre-existing state, replacement,
nonempty state, wrong type/owner/mode/ACL, ambiguous lookup, unsupported observation, or identity
mismatch is preserving terminal failure. A failed exact empty-directory removal is retryable only
within the same live descriptor corridor; after that corridor closes it is preserving terminal.
No rerun infers provenance. Durable crash-residue recovery is out of scope and requires
`BLOCKED_SCOPE_EXPANSION`.

### `R3-MANIFEST-01` — canonical managed-artifact authority

The schema owner is `substrate.managed-artifact-manifest`; version 1 is the only accepted version.
The persisted representation is RFC 8785 canonical JSON with no duplicate keys, no floating-point
values, no insignificant whitespace, and no trailing bytes. Native path/identity byte sequences
that are not guaranteed Unicode are unpadded base64url fields; display paths are non-authoritative.
`installation_id` is a UUIDv7 assigned once at first install. `manifest_id` is the independent
ASCII identifier `m1:<installation_id>:<decimal-generation>`. `manifest_sha256` is lower-case
SHA-256 of the RFC 8785 object after removing only the top-level `manifest_sha256` member; because
`manifest_id` is independently derived, the digest is not self-referential.

The mandatory digest-rule golden vector has preimage bytes exactly
`{"installation_id":"018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90","manifest_generation":7,"manifest_id":"m1:018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90:7","schema_owner":"substrate.managed-artifact-manifest","schema_version":1}`
with no newline and SHA-256
`971e968f3a35a201b04fd55650b43b74c81fd1f7aba0bfca4a2eee4fcf8ad55f`.

Top-level required fields are:

- `schema_owner`, `schema_version`, `manifest_id`, `manifest_sha256`;
- `host_context_commitment`, `selected_host_prefix`, `intended_principal`;
- `platform_kind`, optional exact `platform_mapping_commitment`, and `authority_domain`;
- `installation_id`, `attempt_id`, `manifest_generation`, `created_at_unix_ns`;
- `lifecycle_state`, `previous_manifest_sha256`, `entries`, and `planned_action_receipts`. Each
  planned item preallocates exactly one UUIDv7 `receipt_id` and binds entry ID, action, attempt,
  and fixed relative receipt filename; it contains no future artifact hash.

The manifest digest is integrity evidence, not the trust anchor. The exact locator is derived
before parsing from validated IH/PM and the authority domain:

| Domain | Manifest capsule and head | Required publisher/security |
|---|---|---|
| Unix/macOS A-local | `A/.substrate-lifecycle-v1/`; `head.v1.json`; `manifest.<generation>.json` | intended principal may write the cache; deletion authority additionally requires the platform publisher anchor below |
| Windows A-local | `A\.substrate-lifecycle-v1\` with the same filenames | committed current SID may write the cache; reparse-free handles; deletion authority additionally requires the Windows publisher anchor |
| Linux host and Lima/WSL guest system | `/var/lib/substrate/.substrate-lifecycle-v1/` | root:root; directory `0700`, files `0600`; root-owned no-follow chain and Linux publisher anchor |
| macOS Lima host-shared | `PM.host_platform_control_root/.substrate-lifecycle-v1/<platform_mapping_commitment>/` | intended-principal cache under the exact PM control-root identity plus the macOS publisher anchor |
| Windows host-shared | `PM.host_platform_control_root\.substrate-lifecycle-v1\` | committed-SID cache under the exact PM control-root identity plus the Windows publisher anchor |

Every capsule also has exact no-follow child directory `receipts/<decimal-generation>/` and exact
per-generation mutable CAS file `action-receipts.<decimal-generation>.v1.json`. A receipt filename
is only `receipts/<decimal-generation>/receipt.<preallocated-receipt-id>.json`; neither caller nor
receipt supplies a path, and old generations/indices remain immutable after head advancement.
`ManagedActionReceiptIndexV1` has owner/version `substrate.managed-action-receipt-index`/1 and
binds authority domain, scope/installation ID, manifest generation/digest, monotonically increasing
index revision, previous index digest, and an ordered append-only list of
`ManagedActionReceiptIndexEntryV1`. Each entry contains the planned receipt ID, entry ID, action,
attempt, fixed relative filename, canonical byte length, external receipt-artifact SHA-256,
retained receipt file/parent physical identities, pre-action prepared-record digest, allocated
publisher counter, and durable observation. It never contains the later anchor digest that will
bind the index. Duplicate/reordered receipt IDs, a receipt absent from the manifest plan, hash/name/
identity mismatch, index rollback, replacement, or non-append update rejects.

`ManagedManifestHeadV1` binds installation/scope ID, manifest generation/digest, lifecycle state,
prior head digest, `action_receipt_index_revision`, and `action_receipt_index_sha256`. Receipt
index revision 0 is a canonical empty list published and parent-fsynced with a new manifest before
the initial head/anchor CAS; a generation never has an absent index. Receipt
commit order is exact. Before `ActionStarted`, the protected publisher durably allocates one
counter in `ManagedActionPreparedRecordV1` and binds the manifest-preallocated receipt ID/name.
That record has owner/version `substrate.managed-action-prepared-record`/1 and binds the authority
domain, scope/installation, manifest generation/digest, receipt ID/name, entry/action/attempt,
request digest, exact before observation, executor identity, allocated counter, prior protected
record/anchor digest, state `Prepared`, and its own `LifecycleSignatureV1` by the protected
publisher key. `LifecyclePublisherProtectedStateV1`, owner/version
`substrate.lifecycle-publisher-protected-state`/1, is the exact stored wrapper: it contains the
complete signed current `LifecyclePublisherAnchorV1`, counter, optional complete signed
`ManagedActionPreparedRecordV1` bytes, prior protected-state digest, and state revision. Its sole
locators are Linux
`/var/lib/substrate/.substrate-lifecycle-v1/publisher/current-anchor.v1.json`, macOS System-Keychain
service `com.substrate.lifecycle.v1` account `<scope-id>:current-anchor`, and Windows protected
HKLM value `SOFTWARE\Substrate\LifecycleV1\Anchors\<scope-id>`. The wrapper, not a detached digest,
is atomically CASed and restart-readable; no second prepared-record file/item/value exists.

Before the executor may mutate, the publisher verifies the prior wrapper, increments its revision/
counter, and CASes the full prepared record into the optional slot; one scope has at most one
uncommitted prepared record. Final anchor publication is one wrapper CAS that installs the new
signed current anchor and clears the optional prepared slot. A preserving terminal result instead
CASes that full prepared record to its terminal state and retains it. Kill points bracket wrapper
read, prepared signature, CAS, action, final-anchor/clear CAS, and response; missing full bytes,
detached digest-only state, alternate locator, torn wrapper, rollback, clear-before-final-anchor,
or replay preserves state. After
the exact action observations are available, that same publisher signs the canonical
`ManagedActionReceiptV1` payload with `LifecycleSignatureV1`. The signed payload binds authority
domain, installation/scope, manifest generation/digest, preallocated receipt ID/name, entry/action/
attempt, prepared-record digest, allocated counter, pre/effect/post observations, restoration and
error classification, and executor identity. It contains no receipt-artifact, index, head, anchor,
or self digest. Publish and parent-fsync those fixed immutable signed bytes; compute their external
hash through the retained file; append and temp-fsync/rename/parent-fsync the index by exact prior-
digest CAS; CAS the head to that exact index revision/hash; then advance the protected publisher
anchor over manifest, index, and head. `ReceiptDurable` means the signed receipt publication step;
`Committed` requires all four publication steps.

A fresh process before index publication opens only the manifest-preallocated filename, validates
the receipt signature against the exact protected publisher key named by the prepared record,
requires the exact allocated counter and prepared-record digest, reopens and reobserves the
prepared action, and only then externally hashes the canonical signed bytes. It never trusts a
rediscovered file identity/hash merely because the name or bytes match. Wrong key, algorithm,
counter, prepared record, action observation, filename, signature, or same-principal substitution
preserves state. A crash before the receipt signature exists resumes from the protected prepared
record and exact effect observation or returns the existing preserving ambiguity result; a crash
after signed-receipt durability finishes index/head/anchor CAS. Any extra file or ambiguity
preserves state and requires human repair; directory enumeration never invents a receipt.

No manifest field, environment value, generated projection, or caller path may replace these
locators. A same-principal capsule, digest chain, file owner, or mutable head is never sufficient
deletion provenance. Every accepted generation must also join `LifecyclePublisherAnchorV1`, whose
signed record binds authority domain, IH/PM commitment, scope ID, generation, manifest digest,
receipt-index revision/digest, head digest, previous anchor digest, closed role/action request
digest, requester principal, attempt nonce, and executor build identity. The publisher atomically compare-and-swaps an OS-protected counter/head
before returning the signed record. The verifier rejects any missing record, counter rollback,
valid-old replay, coherent manifest+head rewrite, signature/key substitution, caller-supplied key,
or request/manifest/action mismatch.

The publisher mechanisms are closed and no software fallback is permitted:

| Platform domain | Protected publisher and monotonic anchor | Trusted boundary and failure rule |
|---|---|---|
| Linux host and Linux guest | the root-owned typed Linux executor; Ed25519 key plus the complete `LifecyclePublisherProtectedStateV1` wrapper at fixed `current-anchor.v1.json` under `/var/lib/substrate/.substrate-lifecycle-v1/publisher/`, directory `0700`, files `0600`, opened no-follow from the root-owned chain | the invoking user is not the publisher; root, kernel, and the exact installed executor build are TCB. Missing/replaced key or wrapper, detached prepared digest, counter/revision rollback, non-root publisher, or unavailable durable fsync stops unchanged |
| macOS host | root LaunchDaemon label `com.substrate.lifecycle.publisher.v1`; software P-256 signing key under exact tag `<scope-id>:signing-key` plus the complete protected-state wrapper in the explicitly opened legacy `/Library/Keychains/System.keychain`, service `com.substrate.lifecycle.v1`, account `<scope-id>:current-anchor`; only the exact code-designated lifecycle executor may request product signing | intended principal is only a client. A sufficiently privileged root process with System-Keychain access may export the private key. Missing code signature/requirement, exact Keychain identity, key, wrapper, daemon, counter/revision, or durable update stops unchanged; there is no default/user/file/ambient, detached prepared record, software-file, or unsigned fallback |
| Windows host | LocalSystem service name `SubstrateLifecyclePublisherV1`; non-exportable P-256 key in the LocalMachine CNG key `SubstrateLifecyclePublisherV1`; complete protected-state wrapper at `HKLM\SOFTWARE\Substrate\LifecycleV1\Anchors\<scope-id>` with a protected SYSTEM-only write DACL | committed SID is only a client. Missing service/key/wrapper, wrong service image, DACL inheritance, detached prepared record, counter/revision rollback, or unavailable durable registry flush stops unchanged; there is no current-user/DPAPI/path fallback |
| Lima or WSL guest | the Linux root publisher above, plus a host publisher record binding the exact PM mapping, guest machine identity, and guest anchor digest | both signatures/counters must join; either side missing, stale, or cross-instance stops unchanged |

`LifecycleSignatureV1` is the only signature envelope. Its closed algorithms are
`ed25519-v1` and `ecdsa-p256-sha256-p1363-low-s-v1`. Ed25519 public keys are exactly 32 raw bytes
and signatures exactly 64 raw bytes. P-256 public keys are canonical DER RFC 5480 SubjectPublicKeyInfo
with `id-ecPublicKey` plus `prime256v1`, one uncompressed SEC1 point, and no trailing bytes;
signatures are exactly 64-byte IEEE P1363 `r || s` with low-S required. All byte fields use
unpadded base64url. The unsigned payload is the record's RFC 8785 object with only its top-level
`signature` member removed, prefixed by bytes `SUBSTRATE-LIFECYCLE-SIGNATURE-V1\0`, the exact
record owner, and another NUL. Ed25519 signs those bytes directly; P-256 signs their SHA-256.
Unknown algorithms/encodings, malformed or noncanonical SPKI, wrong curve/point, DER signature,
short/long component, zero/out-of-range `r` or `s`, high-S/malleable signature, wrong payload owner,
or trailing bytes reject before state change. `p256` is the sole portable guest verifier; platform
Security/CNG APIs only sign or expose the public SPKI and never define a second wire encoding.

Linux publisher and disposable retirement-harness signatures use `ed25519-v1`; the harness
commitment carries the exact raw public key. macOS and Windows host publisher tickets,
transcripts, anchors, and publisher-signed retirement receipts use the P-256 variant and carry the
exported public SPKI. Windows retains its non-exportable CNG contract; the R3 macOS software key
does not claim non-exportability from sufficiently privileged root. Every SPKI SHA-256 field is
recomputed from those exact DER bytes. The retirement authorization and acknowledgement use the
same precommitted harness Ed25519 key and fixed encoding; wrong key/algorithm/encoding or a valid
signature over a different record kind is rejected.

The root/System publisher residue and macOS System-Keychain anchor are lifecycle authority, not a
new general installation root. They persist through terminal uninstall so restart can reject a
replayed older generation. Removing the last protected anchor would erase deletion/restoration
proof and therefore requires a later, separately authorized retirement protocol; R3 returns
`BLOCKED_SCOPE_EXPANSION` instead. Product install/uninstall never retires a publisher. Native
evidence may exercise exact-absence bootstrap only in a disposable, evidence-harness-owned scope
that first publishes `PublisherTestRetirementAuthorizationV1` outside every product capsule and
protected publisher. That authorization binds the baseline-absence digest, evidence ID, source
commit/tree/ref, platform scope, exact `bootstrap_core_digest`, complete component IDs,
maximum counter, external receipt-directory identity, one retirement nonce, and the closed future-
guest reservation set below. Before deleting
any publisher evidence, the publisher signs `PublisherTestRetirementReceiptV1`; the independent
harness writes and fsyncs its canonical bytes under its retained private-root descriptor, computes
the external receipt-artifact SHA-256, then signs and fsyncs a separate
`PublisherTestRetirementAcknowledgementV1` over that hash. Only then may the same authorized
retirement attempt return the acknowledgement over the still-attested control channel. The
publisher verifies the harness signature, exact precommitted key, receipt hash/file/parent
identity, and fsync observation, durably records the acknowledgement hash, and only then may it
stop the publisher and remove the
exact test-created endpoint, registration metadata, current anchor, signing key, executable, and
created-empty parents in that dependency-reverse order. Those words are only a summary: the
authorization's exhaustive component DAG must name every exact role/parameter and component UUID,
and teardown may touch nothing outside that DAG. The external receipt carries the public
key and final anchor/record bytes needed to verify every later step. A crash exact-joins from the
external receipt and component observations; mismatch
stops. Baseline parity and the ordinary evidence receipt are recorded only after retirement is
complete. No product caller, installer, uninstaller, manifest, or path observation can issue or
consume this test-only authority.

This is a one-way precommit, not a circular reference. Before bootstrap, the harness generates an ephemeral signing
key and the final bootstrap ID/nonce/component set. For disposable Linux evidence it reserves zero
guest tuples; for disposable macOS/Lima and Windows/WSL evidence it reserves exactly one
`GuestPublisherRetirementReservationV1`. That reservation preallocates the UUIDv7 challenge ID,
host pairing-record component UUID, every guest component UUID/role/target, expected-before state,
and state `Reserved`, all before the host bootstrap authorization is finalized or retirement
authorization is signed. Both canonical records bind the complete ordered reservation bytes.
Product bootstrap has no test-retirement commitment and no reservation set: both fields are
canonical null/empty and product code never constructs or signs retirement authority. The
evidence harness alone canonicalizes the bootstrap core with the optional retirement-commitment
slot fixed to null, signs the canonical test-retirement authorization that binds that core digest,
and supplies its public key, authorization digest, retained external-directory physical identity,
and ordered zero-or-one guest reservation set as the final bootstrap authorization's exact
optional `test_retirement_commitment`.
Generation one and every later anchor copy that immutable commitment. Retirement accepts only the
byte-identical precommitted authorization, recomputes the same null-slot bootstrap core, verifies
the harness signature/key and external parent identity, and rejects a record minted or changed
after bootstrap. The publisher's signed retirement
receipt also binds the same generation-one commitment. No precommit means no retirement path.

For a disposable Linux-host scope, the exact host DAG includes every test-created
`linux.publisher.{bootstrap-intent,executor,service-unit,socket-unit,endpoint,signing-key,current-anchor}`
role, both parameterized publisher service-state roles,
`linux.publisher.{executor-parent,lifecycle-container,state-directory}`, and the exact disposition
of `linux.host.directory(/var/lib/substrate)`. For disposable macOS it includes every test-created
`mac.publisher.{bootstrap-intent,executor,plist,signing-key,current-anchor,mach-service,service-state}`
role and every preallocated `mac.publisher.guest-pairing-record(challenge_id)`; creating any Apple
platform parent is forbidden because all such containers are required pre-existing. For
disposable Windows it includes every test-created
`windows.publisher.{bootstrap-intent,executable,service-registration,signing-key,current-anchor,endpoint,service-state}`
role, `windows.publisher.{product-directory,install-directory}`, every created parameterized
`windows.publisher.registry-container(kind)`, every preallocated
`windows.publisher.guest-pairing-record(challenge_id)`, and no unlisted service/registry parent or
WSL instance deletion. The current-anchor component
means the full `LifecyclePublisherProtectedStateV1`, including counter/revision and optional
prepared record. A guest DAG includes the exact parameterized MAC or WIN guest publisher executor,
service unit, socket unit, endpoint, signing key, current-anchor wrapper, both service-state roles,
external pairing-intent role, exact `mac.lima.publisher-state-directory` or
`windows.wsl.publisher-state-directory`, plus the exact platform's parameterized guest-directory
roles for `/var/lib/substrate`, `/var/lib/substrate/.substrate-lifecycle-v1`, and
`/usr/libexec/substrate` whose disposition proves the evidence attempt created them empty.
Pre-existing state-root, Lima,
or WSL objects are restoration targets, never retirement targets.

Guest retirement is separately precommitted before `PairingIntentDurable` and is acyclic.
`GuestPublisherTestRetirementCommitmentV1` has owner/version
`substrate.guest-publisher-test-retirement-commitment`/1 and is canonical null for product pairing.
For evidence, the harness alone has already preallocated the challenge ID and every host-pairing/
guest component UUID in the immutable host-bootstrap retirement reservation. The host publisher
may issue no unreserved evidence challenge. It canonicalizes `GuestPublisherPairingTicketV1` with its optional
`guest_test_retirement_commitment` slot forced to null, and call that digest the ticket-core digest.
The harness signs `GuestPublisherTestRetirementAuthorizationV1`, owner/version
`substrate.guest-publisher-test-retirement-authorization`/1, binding that core digest, exact ticket/
challenge/source/build/PM/machine scope, ordered guest DAG and host pairing-record component,
expected before/after observations, maximum guest and host counters, harness key, retained external
guest-receipt parent identity, evidence dispatch, nonce, and expiry. The final host-signed ticket
may differ from its core only by carrying the harness algorithm/key, authorization digest, and
external-parent identity as the commitment. The host pairing record, guest intent, hello,
transcript, guest generation one, and every later guest anchor copy that byte-identical commitment.
Changing another ticket-core field, adding the commitment after intent, using a different
challenge/component ID, or a missing/product-null commitment in evidence rejects before guest
mutation.

Reservation state is `Reserved -> TicketIssued -> GuestConsumed -> Retired`, with signed protected-
record CAS at each transition. `GuestConsumed` is the one-use guest-mutation grant: after TTY and
ticket verification but before `GuestStateRootDurable` or any other guest effect, the guest exact-
joins the host protected record and the host atomically advances the reservation from
`TicketIssued` to `GuestConsumed`; a guest must present that signed transition before its first
mutation. Thus an exact `TicketIssued -> ProvenUnused` CAS permanently revokes an unused ticket
before any guest effect can begin. A failed evidence run may use `Reserved -> ProvenUnused` or
`TicketIssued -> ProvenUnused` only after the host publisher signs
`GuestPublisherReservationUnusedProofV1`, binding the bootstrap/reservation/ticket/host-record
identities (canonical null ticket/record fields when issuance never occurred), terminal rejection/
expiry/cancellation reason, protected counter, and exact observation
that every reserved guest target still equals its precommitted before-state with zero guest-owned
effect. "Before-state" is exact absence only for an absent target; a pre-existing target must match
its retained physical identity, bytes, metadata, and service/platform state. The external harness
durably stores and hashes that proof, signs and fsyncs
`GuestPublisherReservationUnusedAcknowledgementV1`, and the host publisher verifies and records
that acknowledgement before proving the never-created pairing record still matches before-state or
restoring/removing the exact created precommitted pairing-record component, and before advancing
`ProvenUnused -> Retired`. No liveness, PID, timeout, pathname, or unacknowledged host
observation proves unused state. A clean MAC/WIN evidence result requires the sole tuple to reach
`GuestConsumed` and later retirement. An extra, unreserved, reused, reordered, omitted, or cross-
bootstrap tuple, a second evidence ticket, host teardown while a reservation is nonterminal, a
guest effect without the signed `GuestConsumed` grant, or deletion without exact
`GuestConsumed`/`ProvenUnused` proof preserves state.

The unused branch has a closed, non-self-referential wire contract. The host proof owner/version is
`substrate.guest-publisher-reservation-unused-proof`/1. Its exact canonical RFC 8785 fields are:
owner/version; bootstrap ID and authorization digest; complete reservation bytes and digest;
challenge ID; ticket and host pairing-record bytes/digests/physical identity or canonical nulls
when never issued; source commit/tree/ref and evidence dispatch correlation; IH/PM/principal/
platform/guest-machine scope; prior protected-host state/anchor digest, expected next counter and
revision, transition intent `ProveUnused`, and CAS nonce; closed terminal reason `RejectedBeforeGrant`,
`ExpiredBeforeGrant`, `CancelledBeforeGrant`, `EofBeforeGrant`, or `InterruptedBeforeGrant`; ordered
reserved-target expected-before and exact current observations; `guest_effect_count=0`; observation
time; and one `LifecycleSignatureV1`. It is signed only by the exact protected host publisher P-256
key/SPKI using canonical P1363 low-S encoding and contains no proof-artifact, acknowledgement,
resulting `ProvenUnused`/`Retired` CAS or anchor digest, future-anchor, or self digest.

The harness publishes those canonical proof bytes under the precommitted external parent, retains
the proof file and parent physical identities, fsyncs file and parent, and only then computes
`guest_publisher_reservation_unused_proof_artifact_sha256_v1`. The acknowledgement owner/version is
`substrate.guest-publisher-reservation-unused-acknowledgement`/1. Its exact canonical fields are:
owner/version; that external proof SHA-256; exact proof file and parent physical identities; file
and parent fsync observations; bootstrap/reservation/challenge/ticket/pairing-record identities and
digests with the same canonical-null rule; evidence dispatch correlation; IH/PM/principal/platform/
guest-machine scope; acknowledgement time; 256-bit nonce; and one `LifecycleSignatureV1` using only
the already-precommitted harness `ed25519-v1` key, raw 32-byte public key, and 64-byte signature. It
contains no acknowledgement-artifact or self digest. After acknowledgement file/parent fsync, the
publisher independently hashes those bytes as
`guest_publisher_reservation_unused_acknowledgement_artifact_sha256_v1`, verifies every identity,
fsync observation, digest, scope, dispatch, nonce, key, and signature, and only then CASes from the
exact prior protected state to `ProvenUnused` using the precommitted expected counter/revision and
nonce while binding both external artifact hashes. The later `Retired` CAS binds the resulting
`ProvenUnused` CAS digest plus those same external hashes. Neither external record contains a
resulting or future CAS/anchor digest. Wrong key or
algorithm, proof/ack hash, file/parent identity, missing fsync, reservation/ticket/pairing record,
dispatch/scope, replayed nonce, cross-scope record, unknown field, or self/future digest preserves
state before pairing-record mutation.

Guest teardown precedes host teardown. The guest publisher first quiesces its endpoint/service,
then signs `GuestPublisherTestRetirementReceiptV1` over the guest authorization/commitment, final
guest anchor/protected-state bytes and counter, complete guest DAG identities/observations, and
attempt. The harness durably stores and externally hashes that receipt, then signs and fsyncs
`GuestPublisherTestRetirementAcknowledgementV1` over the hash and retained file/parent identities.
After the guest verifies and durably records that acknowledgement, the still-attested bootstrap
executor removes only DAG members in reverse dependency order: service state, endpoint, units,
executor, current-anchor wrapper/key, publisher directory/created-empty parents, and finally the
external seed intent. The host pairing record remains until the host verifies the guest receipt,
acknowledgement, removal observations, and exact guest baseline parity. The host then includes that
guest receipt hash/acknowledgement and pairing-record identity in its own publisher retirement
receipt; only after the host acknowledgement is durable may it remove the pairing record, host
endpoint/service state/registration, protected-state wrapper/key, bootstrap intent, executable,
and exact created-empty parents in reverse order. Host or guest bootstrap intent/protected state is
never erased before all evidence it authenticates is external and acknowledged. Kill points bracket
every quiesce, receipt/signature/fsync/hash/acknowledgement, guest removal, guest parity join, host-
record removal, host removal, and final parity observation. Wrong/missing/cross-ticket commitment,
early intent/pairing-record/protected-state removal, role omission, counter drift, or non-parity
preserves state and returns `BLOCKED_NATIVE_EVIDENCE`.

The test-retirement authorization owner/version is
`substrate.publisher-test-retirement-authorization`/1, the receipt owner/version is
`substrate.publisher-test-retirement-receipt`/1, and the acknowledgement owner/version is
`substrate.publisher-test-retirement-acknowledgement`/1. The authorization binds exact harness
account/UID or SID, retained external parent physical identity, ordered component identities,
expected before/after observations, bootstrap-core digest, maximum counter, issued/expiry time,
source evidence dispatch correlation, and one `LifecycleSignatureV1` using only `ed25519-v1`, the
precommitted 32-byte raw harness public key, and 64-byte signature; it contains no future receipt
or acknowledgement hash.
The publisher-signed receipt binds the authorization/commitment digest, final bootstrap-
authorization digest, last-anchor digest, counter, component observations, and retirement attempt,
but contains no digest of its own canonical bytes. Only after canonical serialization and fsync
does the harness compute `retirement_receipt_artifact_sha256`. Its signed acknowledgement binds
that hash, the exact external receipt file identity/parent identity, fsync observation, evidence
dispatch, acknowledgement time, and another `ed25519-v1` signature by that exact same harness key;
it likewise contains no self digest. Unknown fields, a
product scope, an unretained external directory, missing or mismatched acknowledgement, or counter
above the authorized maximum rejects before teardown.

Greenfield publisher bootstrap is a separate closed transition,
`PublisherBootstrapAuthorizationV1`. Its canonical fields are exactly schema owner/version,
UUIDv7 `bootstrap_id`, platform/domain/scope ID, IH commitment, optional required PM commitment,
requester OS account plus UID or SID, exact source commit/tree/ref, reviewed implementation receipt
digest, `ExecutorBuildEvidenceV1` digest, executor artifact SHA-256 and platform code identity when
the native platform supplies one, ordered
`PublisherBootstrapComponentV1` records, `publisher_expected_absent=true`, issued/expiry Unix nanoseconds,
256-bit nonce, exact optional `test_retirement_commitment` (canonical null for product; otherwise
harness algorithm `ed25519-v1`, raw 32-byte public key, authorization digest, and external parent
physical identity plus the ordered zero-Linux/one-MAC-or-WIN
`GuestPublisherRetirementReservationV1` set), and confirmation
string `CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER`. Unknown,
duplicate, empty, expired, cross-source, cross-principal, cross-IH/PM, or reordered fields reject.
The literal owner is `substrate.publisher-bootstrap-authorization`, version is integer 1, and the
canonical owner/version for its component records is
`substrate.publisher-bootstrap-component`/1. Each component record contains exactly UUIDv7
`component_id`, closed component role, independently derived target identity, expected object type,
closed `expected_before` disposition, source artifact digest/signature when applicable, exact
intended owner/group/mode/DACL/code requirement, ordered dependency component IDs, and durability
method. The authorization-level `publisher_expected_absent=true` means the publisher identity itself is
greenfield: endpoint, signing key, protected state, service registration/units, and executable may
not pre-exist. It does not assert that enumerated parent/dependency roles are absent.
`expected_before` is exactly `ExactAbsentCreate` or
`ExactPreExistingDependency { physical_identity, object_type, bytes_or_target, owner, group, mode,
acl_or_security, service_or_platform_state }`. Only state-root, lifecycle/product/install/
executor-parent, or registry-container roles whose table entry explicitly admits pre-existence may
use the latter; every publisher-identity role requires `ExactAbsentCreate`. Both dispositions stay
in the complete component set. A pre-existing dependency is validated and retained but never
created, replaced, retired, or converted into deletion authority; a created dependency is included
in the current-attempt retirement DAG. The
authorization validator recomputes every target and dependency from platform plus scope and rejects
a caller-supplied path, identity, metadata relaxation, dependency cycle, duplicate target, or
incomplete platform component set.

`PublisherBootstrapComponentRoleV1` is the closed component-role universe. Each variant derives
the exact target below and then exact-matches the same target/type/metadata in the exhaustive
managed-role table; `platform` for a guest is only `mac_lima` or `windows_wsl`, and `kind` accepts
only the literal values shown. Filesystem directory creation retains the parent/child descriptors,
records the physical identity in the protected bootstrap/pairing state immediately after create,
and fsyncs file/directory plus parent. Keychain creation uses one exact `SecItemAdd`/readback;
registry creation uses exact DACL/readback plus `RegFlushKey`; service/LaunchDaemon registration
uses exact readback plus its protected-state CAS. An effect-visible/identity-CAS gap is the already-
defined terminal-preserving result.

| Component role | Exact derived target and object | Dependencies and durability |
|---|---|---|
| `LinuxStateRoot` | `/var/lib/substrate`, root:root directory; pre-existing exact non-group/world-writable metadata is recorded, created mode is `0755` | retained `/var/lib`; directory/parent fsync |
| `LinuxLifecycleContainer` | `/var/lib/substrate/.substrate-lifecycle-v1`, root:root `0700` directory | `LinuxStateRoot`; directory/parent fsync |
| `LinuxPublisherDirectory` | `/var/lib/substrate/.substrate-lifecycle-v1/publisher`, root:root `0700` directory | `LinuxLifecycleContainer`; directory/parent fsync |
| `LinuxExecutorParent` | `/usr/libexec/substrate`, root:root `0755` directory | pre-existing retained `/usr/libexec`; directory/parent fsync |
| `LinuxBootstrapIntent` | exact state-root intent name derived by the frozen scope hash | `LinuxStateRoot`; unnamed-file/link/parent-fsync protocol |
| `LinuxExecutor` | `/usr/libexec/substrate/substrate-lifecycle-linux`, root:root `0755` regular file | `LinuxExecutorParent`; source identity/hash, file/parent fsync |
| `LinuxServiceUnit(kind)` | exact lifecycle publisher unit under `/etc/systemd/system`, `kind`=`service` or `socket`, root:root `0644` regular file | `LinuxExecutor`; pre-existing `/etc/systemd/system` required, file/parent fsync |
| `LinuxEndpoint` | `/run/substrate-lifecycle-publisher-v1.sock`, root:root `0600` seqpacket socket | socket unit; exact socket identity after activation |
| `LinuxSigningKey` | fixed publisher `signing-key.v1`, root:root `0600` regular file | `LinuxPublisherDirectory`; file/parent fsync |
| `LinuxProtectedState` | fixed publisher `current-anchor.v1.json`, root:root `0600` `LifecyclePublisherProtectedStateV1` | signing key; wrapper/parent fsync CAS |
| `LinuxServiceState(kind)` | lifecycle publisher `kind`=`service` or `socket` enabled/active state | corresponding unit, protected before/after state and exact manager query |
| `MacExecutor` / `MacPlist` / `MacMachService` / `MacSigningKey` / `MacProtectedState` / `MacBootstrapIntent` / `MacServiceState` | exactly the corresponding `mac.publisher.*` target/object in the role table | fixed existing Apple parent/container, designated requirement, Keychain/XPC/launchd readback and protected CAS as applicable |
| `MacGuestPairingRecord(challenge_id)` | exact System-Keychain account `<scope-id>:guest-pairing:<challenge-id>` | reserved challenge tuple; Keychain add/readback and generation CAS |
| `WindowsProductDirectory` | `%ProgramFiles%\Substrate`, protected directory with the frozen publisher DACL | pre-existing `%ProgramFiles%`; handle identity and directory/parent flush |
| `WindowsInstallDirectory` | `%ProgramFiles%\Substrate\Lifecycle`, same protected DACL | `WindowsProductDirectory`; handle identity and directory/parent flush |
| `WindowsExecutable` / `WindowsServiceRegistration` / `WindowsSigningKey` / `WindowsProtectedState` / `WindowsBootstrapIntent` / `WindowsEndpoint` / `WindowsServiceState` | exactly the corresponding `windows.publisher.*` target/object in the role table | exact directory/registry/service/key/pipe dependency and flush/readback defined by that role |
| `WindowsRegistryContainer(kind)` | exact HKLM key for `kind`=`substrate`, `lifecycle-v1`, `anchors`, `bootstrap-intents`, `pairings`, or `pairing-scope` | parent kind in listed order, then protected DACL readback and `RegFlushKey`; `pairing-scope` additionally binds scope ID |
| `WindowsGuestPairingRecord(challenge_id)` | exact protected pairing value for the reserved challenge | `pairing-scope`; value write/readback/`RegFlushKey` and generation CAS |
| `GuestStateRoot(platform)` | guest `/var/lib/substrate`, root:root directory; same pre-existing/created rule as `LinuxStateRoot` | protected host pairing-record pending/identity CAS, then directory/parent fsync |
| `GuestLifecycleContainer(platform)` | guest `/var/lib/substrate/.substrate-lifecycle-v1`, root:root `0700` directory | `GuestStateRoot`; directory/parent fsync |
| `GuestPublisherDirectory(platform)` | guest `/var/lib/substrate/.substrate-lifecycle-v1/publisher`, root:root `0700` directory | `GuestLifecycleContainer`; directory/parent fsync |
| `GuestExecutorParent(platform)` | guest `/usr/libexec/substrate`, root:root `0755` directory | retained pre-existing `/usr/libexec`; directory/parent fsync |
| `GuestBootstrapIntent(platform,challenge_id)` | exact state-root pairing-intent name derived by the frozen scope/challenge hash | `GuestStateRoot` plus reserved tuple; unnamed-file/link/parent-fsync protocol |
| `GuestExecutor(platform)` / `GuestServiceUnit(platform,kind)` / `GuestEndpoint(platform)` / `GuestSigningKey(platform)` / `GuestProtectedState(platform)` / `GuestServiceState(platform,kind)` | exact corresponding `mac.lima.publisher-*` or `windows.wsl.publisher-*` target/object; unit/state `kind`=`service` or `socket` | guest executor-parent/lifecycle-container and corresponding unit/key dependencies; exact file/parent fsync, manager readback, or wrapper CAS |

The grouped dependency edges are also closed. macOS uses `MacExecutor -> MacPlist ->
MacMachService -> MacServiceState`, `MacSigningKey -> MacProtectedState`, and an independent
Keychain `MacBootstrapIntent`; the pairing record depends on the protected state and reserved
tuple. Windows uses `WindowsProductDirectory -> WindowsInstallDirectory -> WindowsExecutable ->
WindowsServiceRegistration -> WindowsEndpoint/WindowsServiceState`, CNG
`WindowsSigningKey -> WindowsProtectedState` with the `anchors` registry container, and
`WindowsBootstrapIntent` with `bootstrap-intents`; the pairing record depends on protected state,
`pairings`/`pairing-scope`, and the reserved tuple. Each guest uses `GuestExecutorParent ->
GuestExecutor -> GuestServiceUnit(kind) -> GuestEndpoint/GuestServiceState(kind)`, and
`GuestStateRoot -> GuestLifecycleContainer -> GuestPublisherDirectory -> GuestSigningKey ->
GuestProtectedState`; the pairing intent depends directly on guest state root and its reserved
tuple. No grouped slash denotes optional ownership or an alternate dependency.

`/var/lib`, `/usr/libexec`, `/etc/systemd/system`, `/run`, `/Library/PrivilegedHelperTools`,
`/Library/LaunchDaemons`, `%ProgramFiles%`, HKLM `SOFTWARE`, the service manager, CNG store,
Keychain, and pipe namespace are required platform containers and are never created or removed by
R3. Every potentially created intermediate below them is listed above and in the managed-role table.
An absent required platform container, unlisted intermediate, target mismatch, incomplete variant
set, or effect outside a precommitted component UUID stops before mutation.

`canonical_publisher_bootstrap_core_v1` is the same canonical bootstrap authorization with
`test_retirement_commitment` forced to null; its digest is computable before the retirement
authorization. For a disposable evidence scope, validation requires the final bootstrap's every
other byte to equal that core, then verifies the signed retirement authorization over the core
digest before accepting its commitment. This one-way construction is the only permitted ordering;
using the final bootstrap digest inside the precommitment, changing any core field afterward, or
nesting either digest into itself is a preserving decoder error.

`ExecutorBuildEvidenceV1` has owner/version `substrate.executor-build-evidence`/1 and binds the
published source commit/tree/ref, Cargo.lock digest, target triple, host OS/architecture, exact
Rust/Cargo/linker/signing-tool identities, ordered argv/environment allowlist, output artifact
SHA-256, optional platform code identity, and evidence-task correlation. Provider implementation
fixtures may build for static/model checks but publish no binary. Each provider evidence task builds
its host executor natively and its Linux guest executor in an isolated build scope from the exact
live-remote-equal source, cleans that scope before platform baseline capture, and records this
evidence; this is the sole pre-UNIX artifact source. MAC/WIN provider packets therefore make no
production-distribution claim. UNIX later owns cargo-dist archive delivery for ordinary product
use. Missing build evidence, different source/lock/toolchain/target/output, or an artifact supplied
from an earlier packet/task is rejected.
Only the hidden direct-interactive command
`substrate-lifecycle-control publisher-bootstrap` may issue the canonical bytes; it reads the
confirmation from its controlling terminal, verifies the published implementation receipt and
artifact, and immediately delivers the authorization on the bootstrap channel below. It never
writes the authorization to a file, environment variable, generated projection, script output,
or reusable carrier. Scripts and ordinary product commands cannot issue or replay it.

Bootstrap elevation and first delivery are closed. Linux control creates an `AF_UNIX`
`SOCK_SEQPACKET|SOCK_CLOEXEC` socketpair. macOS control creates an `AF_UNIX SOCK_STREAM`
socketpair and sets `FD_CLOEXEC` plus `SO_NOSIGPIPE` on both endpoints before spawn. Each retains
the client end, places only the peer on descriptor 3, and invokes the exact verified platform
executor through `/usr/bin/sudo -C 4 --` with an empty supplemental environment and
`--publisher-bootstrap-fd 3`; the macOS executor immediately re-arms `FD_CLOEXEC` on inherited
FD3 before socket, peer, image, terminal, codesign, or decode work. The elevated executor
joins `SO_PEERCRED`, the retained requester PID/executable identity, the executor file identity/
digest, and the invoking terminal session. Windows control creates a nonce-named bootstrap pipe
with a DACL containing only SYSTEM and the committed requester SID, then uses `ShellExecuteExW`
with `runas` for the exact verified executor and the pipe name plus nonce; the elevated executor
impersonates the pipe client, joins SID and `GetNamedPipeClientProcessId`, and verifies both images
through retained handles, `GetFileInformationByHandleEx`, SHA-256/build-evidence equality, and
exact DACL. Authenticode may be recorded when a later release supplies it but is not bootstrap
authority. One canonical JSON request and response, each nonempty and at most 1 MiB, is exchanged.
Linux preserves message boundaries, macOS uses bounded EOF framing with `shutdown(SHUT_WR)` only
after each complete document, and Windows uses an unsigned little-endian 32-bit byte length.
Empty, premature/disconnected EOF, timeout/trickle, trailing/concatenated JSON, a second frame,
truncation, unknown peer, executable replacement, or consent cancellation leaves all state
unchanged; an ambiguous partial exchange is never automatically resent.

The first host-bootstrap durable record is never placed under a publisher directory that the same
bootstrap has not yet created. Linux uses only the already-enumerated `/var/lib/substrate` state
root. Its lowercase `scope-sha256` is SHA-256 over bytes
`SUBSTRATE-LIFECYCLE-BOOTSTRAP-SCOPE-V1\0` followed by the RFC 8785 canonical JSON array
`[platform_kind,authority_domain,scope_id,ih_commitment,pm_commitment_or_null]`; invalid UTF-8,
alternate order/encoding/case, or a collision with any existing role rejects. From a retained,
validated root:root, non-group/world-writable `/var/lib/substrate` descriptor, Linux atomically
publishes the canonical intent from a root:root `0600` unnamed `O_TMPFILE` through
`linkat(AT_EMPTY_PATH)` to exact absent
`/var/lib/substrate/.substrate-lifecycle-bootstrap-intent-v1.<scope-sha256>.json` and then fsyncs
that parent. If the enumerated state root was absent, its exact bootstrap component must reach a
durable identity record first; a crash after root visibility but before that record is the explicit
terminal-preserving create gap and is never adopted on retry. macOS atomically creates the System
Keychain item service `com.substrate.lifecycle.v1`, account `<scope-id>:bootstrap-intent`; Windows
atomically creates and flushes the protected SYSTEM-write-only HKLM value
`SOFTWARE\Substrate\LifecycleV1\BootstrapIntents\<scope-id>`. Each intent binds the accepted
authorization digest/nonce, source/executor evidence, ordered component set, exact per-role
expected-before observations, and `CreatePending`/identity-CAS state. Collision or an unavailable durable primitive
stops before a component effect. If the state root was already joined, a crash before successful
initial intent publication leaves no named bootstrap record or new component. The explicitly
enumerated state-root visibility/identity-record gap remains terminal-preserving as stated above;
after intent publication, restart exact-joins that protected record.

The authorization is accepted only when every publisher-identity role is
`ExactAbsentCreate` and every admitted dependency role exactly matches its
`ExactPreExistingDependency` expected-before identity and metadata. The exhaustive component set
contains both dispositions; no pre-existing executor, service, endpoint, signing key, protected
state, counter, anchor, or other publisher identity is adoptable. The component manifest is atomic
and explicit: Linux uses
executor `/usr/libexec/substrate/substrate-lifecycle-linux`, service/socket units
`/etc/systemd/system/substrate-lifecycle-publisher-v1.{service,socket}`, endpoint
`/run/substrate-lifecycle-publisher-v1.sock`, fixed publisher directory/key/current-anchor
children, and the independently located bootstrap-intent role inside the enumerated
`/var/lib/substrate` state root; macOS uses executable
`/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1`, plist
`/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist`, Mach service
`com.substrate.lifecycle.publisher.v1`, and distinct System-Keychain key/current-anchor items;
Windows uses executable `%ProgramFiles%\Substrate\Lifecycle\substrate-lifecycle-windows.exe`,
service `SubstrateLifecyclePublisherV1`, pipe `\\.\pipe\SubstrateLifecyclePublisherV1`, CNG key,
and the fixed HKLM current-anchor value. User-writable executors under A, PM, a download/staging
directory, or a build tree are never privileged service images: bootstrap first no-follow opens,
hashes and verifies the source artifact, copies it to the fixed protected destination, fsyncs or
flushes it and its parent, reopens and rejoins identity, then registers the endpoint/service.
Lima/WSL guest bootstrap reuses the exact Linux component paths inside the PM-bound guest and does
not treat child-channel continuity as authentication. The already-protected host publisher first
durably issues one `GuestPublisherPairingTicketV1`, signed by its current anchor key and bound to a
UUIDv7 challenge ID, 256-bit challenge, expiry, host publisher public-key SPKI SHA-256/current-
anchor digest, canonical host public-key SPKI DER bytes and canonical signed current-anchor public
record bytes, exact IH/PM and guest machine identity,
source commit/tree/ref,
`ExecutorBuildEvidenceV1` digest, staged executor SHA-256, and the complete guest component
commitment with publisher-identity roles `ExactAbsentCreate` and admitted dependencies bound to
their exact `ExactPreExistingDependency` expected-before state.
The attested host control binary prints the full host-key fingerprint, challenge ID, and challenge
only on its retained controlling terminal. The ticket may cross the retained child channel, but it
is not trusted merely because it arrived there.

The challenge owner/version is `substrate.guest-publisher-pairing-challenge`/1 and the ticket
owner/version is `substrate.guest-publisher-pairing-ticket`/1. The challenge contains exactly its
ID, random value, expiry, host-key fingerprint/current-anchor digest, PM/machine/source/build-
evidence scope, and expected guest component commitment. The ticket contains those same fields,
the canonical signer SPKI DER bytes, exact `LifecyclePublisherAnchorV1` public record and digest,
signature algorithm/encoding, challenge digest, host publisher generation/counter, and host
signature, plus exact optional `guest_test_retirement_commitment` (canonical null for product).
The signer SPKI hash must equal the complete challenge/terminal fingerprint; the guest
first verifies the anchor record with that pinned key and exact digest, then requires the ticket
key/generation/counter to equal the anchor before verifying the ticket. Unknown, duplicate,
empty, reordered, cross-scope, expired, already-consumed, or noncanonical fields reject.

The host packet copies only the verified executor into a current-attempt guest staging file and
launches that exact digest as root through the mapped `limactl shell` or `wsl -d` child. Before any
guest mutation, the guest executor observes its machine ID and own artifact identity, opens its
guest controlling TTY independently of stdin/stdout, displays the complete ticket scope and host-
key fingerprint, and requires the operator to enter the exact full fingerprint, challenge, and
literal `PAIR EXACT SUBSTRATE GUEST PUBLISHER` obtained from the host terminal. Neither control nor
the harness may copy that confirmation through argv, environment, file, pipe, stdin, generated
projection, or automation. Only after this out-of-band pin does the guest verify the ticket
by parsing the supplied SPKI, hashing its exact DER bytes, exact-matching the fully entered
fingerprint, verifying the carried current-anchor public record/key/digest, and applying the closed
P-256 verifier. It then generates its
durable intent, key, nonce, and `GuestPublisherBootstrapHelloV1` through the sequence below. Thus a request-supplied key,
an independently launched child, or an attacker-signed ticket cannot become authority.

Pairing state is not process-local. The host record owner/version is
`substrate.guest-publisher-pairing-host-record`/1. macOS stores it as a System-Keychain item service
`com.substrate.lifecycle.v1`, account `<scope-id>:guest-pairing:<challenge-id>`; Windows stores it
at protected HKLM value
`SOFTWARE\Substrate\LifecycleV1\Pairings\<scope-id>\<challenge-id>`. The record binds the ticket,
counter/current-anchor, an ordered list of child/channel transport observations, hello, signed
transcript, guest anchor, state, and prior-record digest. Transport observations are evidence only,
never authority or signed pairing identity. The host publisher advances the record by signed
generation-CAS and retains
the terminal consumed record for replay rejection.

The guest intent owner/version is `substrate.guest-publisher-pairing-guest-intent`/1. Its first
durable locator is outside the not-yet-created publisher directory but inside the already-
enumerated guest state root: exact root-owned regular file
`/var/lib/substrate/.substrate-lifecycle-pairing-intent-v1.<scope-challenge-sha256>.json`. The
lowercase suffix is SHA-256 over bytes `SUBSTRATE-GUEST-PAIRING-INTENT-V1\0` followed by the RFC
8785 canonical JSON array `[scope_id,challenge_id]`; alternate encoding/order/case or a collision
with another role rejects. Before the first guest filesystem effect, the protected host pairing record either binds
the exact pre-existing `/var/lib/substrate` physical identity/before-state or records
`GuestStateRootCreatePending`, then the guest reports the created identity and the host publisher
CASes `GuestStateRootDurable`. A kill after root visibility but before that host CAS preserves the
root and returns `BLOCKED_SCOPE_EXPANSION`; it is never adopted. The executor retains and validates
the joined `/var/lib/substrate` descriptor as root:root, non-group/world-writable, and
no-follow. It creates an unnamed `O_TMPFILE` mode `0600` in that filesystem, writes and fsyncs the
complete canonical intent, atomically `linkat(AT_EMPTY_PATH)` publishes only the exact absent
derived name, and fsyncs `/var/lib/substrate`; named temporary files, rename fallback, another parent, and
filesystems without those semantics return `BLOCKED_SCOPE_EXPANSION` before publisher mutation.
Before the link succeeds there is no guest publisher directory or named intent. If no state-root
effect occurred, retry repeats the independent TTY confirmation; the enumerated root-identity gap
uses only the terminal result above. After a complete link is visible, restart accepts it only after
root ownership/mode/type, ticket, scope, confirmation commitment, machine/artifact, canonical
bytes, and its Ed25519 self-signature all verify; absence repeats confirmation, and any other
observation preserves state.

The intent contains a root-confined 32-byte Ed25519 seed, the derived raw public key/digest, and one
fixed nonce, in addition to the ticket and confirmation/machine/artifact commitments. The seed is
never returned to the host, capsule, log, receipt, or evidence artifact; the root-only external
intent remains retained bootstrap authority. It lets every retry derive the byte-identical hello
without adopting a final-path key. Only after the host-signed transcript is durable does the guest
commit the fixed publisher components in the common bootstrap order, materializing the exact same
seed at final `signing-key.v1` only at `KeyDurable`. No alternate or regenerated seed/key/nonce may
join. The intent also copies the ticket's exact guest-test-retirement commitment; evidence pairing
rejects a missing/mismatched commitment before linking the intent. No publisher endpoint accepts a request until generation one and host consumption are
joined.

`GuestPublisherBootstrapHelloV1` binds the exact ticket and durable intent digests, byte-identical
guest-test-retirement commitment, guest machine/
artifact, intent publication time/fsync observation, nonce, guest raw Ed25519 public key, and an
`ed25519-v1` signature by that key over the hello. This self-signature is not independent authority;
it proves that every retry presents the same private key whose public identity is joined into the
host-signed transcript. A ticket past expiry is accepted only to finish an intent whose exact
signed hello proves the already-durable intent publication time was within the ticket window.

The host publisher exact-joins the still-unconsumed host record, PM, guest machine identity,
staged digest, durable hello, guest nonce, and guest public-key digest,
then signs and durably records `GuestPublisherBootstrapTranscriptV1`, which copies the same
guest-test-retirement commitment. The guest verifies that
signature with the pinned key, durably records the exact transcript in its intent, and atomically
commits the remaining fixed publisher components and generation-one anchor. The host joins that
anchor and durably consumes
the ticket; the guest records the consumed host proof before any guest role. Pairing states are:

```text
host:  TicketIssued -> GuestMutationGranted -> HelloJoined -> TranscriptDurable
         -> GuestGenerationOneJoined -> TicketConsumed
guest: [ephemeral] HumanPinned -> [durable] GuestStateRootDurable -> PairingIntentDurable -> HelloDurable
         -> TranscriptDurable -> PublisherDirectoryDurable -> ExecutorDurable
         -> EndpointMetadataDurable -> KeyDurable -> CounterZeroDurable
         -> PublisherActive -> GenerationOneDurable -> HostConsumptionJoined
```

Expiry and the host-protected `GuestMutationGranted`/`GuestConsumed` join are checked before
`GuestStateRootDurable`; an exact durable intent plus its exact signed hello may finish after process restart
without re-prompting, but cannot change ticket, key, nonce, machine, artifact, or confirmation
commitment. EOF/process loss may open a new transport attempt, but it may relay only the byte-
identical durable hello/transcript and is appended as non-authorizing observation; channel identity
never changes the pairing record. Replacement/ambiguity of either host record, guest intent, signing key, or
anchor is preserving `BLOCKED_SCOPE_EXPANSION`. Exact retry joins the same state. Wrong/absent TTY
confirmation, expired/duplicate ticket, fingerprint/challenge/PM/machine/artifact drift,
malformed/wrong-curve/substituted SPKI, SPKI/fingerprint or SPKI/anchor-key mismatch,
noncanonical/high-S signature, channel-only authority, alternate-child substitution,
substituted key/nonce/transcript,
replay on another instance, or
carrier/staged-file-only input leaves the guest unchanged. Kill tests bracket ticket issue/fsync,
host display, guest TTY open/confirmation, ticket verification, the one-use guest-mutation grant,
each `Reserved`/`TicketIssued -> ProvenUnused` proof and acknowledgement durability boundary,
pairing-record restoration/removal, guest-state-root pending/create/
identity CAS, unnamed-intent create/write/fsync, atomic link, `/var/lib/substrate` fsync, hello publication, host-record CAS, transcript
signature/publication/verification, publisher-directory create/identity-record CAS, signing-key
atomic publication/identity-record CAS, and each remaining guest component
transition, both anchor joins, host ticket consumption, and guest consumption join. Each kill
restarts a fresh process and proves re-prompt-before-intent, exact durable resume from a completed
state record, or the closed terminal-preserving result for an effect-visible/identity-record-not-
durable creation gap.

Host bootstrap durable states are exactly `Absent -> IntentDurable -> ExecutorDurable ->
EndpointMetadataDurable -> KeyDurable -> CounterZeroDurable -> PublisherActive ->
GenerationOneAnchored`; guest bootstrap uses the expanded pairing sequence above. Every state
records the authorization digest, component ID and physical
identity, dependency IDs, before/after observation, and action receipt. Kill points bracket every
copy, metadata change, registration, key creation, counter-zero publication, activation, and first
anchor update. Partial bootstrap exact-joins only that recorded authorization/nonce and next state;
any pre-existing, partial, mismatched, or unverifiable state is never adopted or overwritten and
returns `BLOCKED_SCOPE_EXPANSION`. In particular, every named component create has a protected
`CreatePending` record before the effect and an identity CAS immediately after. A kill after the
object becomes visible but before its physical identity is durable never adopts it from path or
bytes: it preserves the object and returns `BLOCKED_SCOPE_EXPANSION` with the pending record as
human-repair evidence. Only a completed identity CAS is restart-joinable. Package signatures,
checksums, commit/tree, sudo/UAC, a name, or apparent matching bytes alone authorize nothing.

Lima instance realization has one pre-PM exception because the R2 two-stage mapping cannot know a
guest machine identity before an absent fixed instance is created. `LimaStageOneAuthorizationV1`
has schema owner/version `substrate.lima-stage-one-authorization`/1 and binds the already-validated
IH, account-database-derived Lima control-root identity, already-selected instance name, fixed
profile bytes/digest, expected absence, source commit/tree/ref and executor receipt, requester,
attempt UUIDv7, nonce, and expiry. The macOS protected publisher durably anchors it before
`limactl create`; no A/PM capsule or path supplies this authority. After start, the packet observes
and reobserves guest machine ID/account/UID/home, finalizes PM, and publishes the full instance
manifest entry joined to the stage-one anchor before any guest projection. Pre-existing instance,
selector/control-root mismatch, profile mismatch, ambiguous create result, or inability to finalize
PM preserves the instance. A crash may resume only the same anchored attempt; it may remove the
instance only when the stage-one record proves this attempt created it, the exact machine identity
has been durably attached, no guest projection or unrelated state exists, and the reverse action
receipt is externally durable. This exception selects no new instance or control root, authorizes
no WSL creation, and is not reusable for forwarding or other roles.

Scripts submit `ManagedLifecyclePublisherRequestV1`; they never sign or authorize it. The request
binds the exact IH/PM commitment, scope ID, current anchor counter/digest, manifest generation and
digest, one role and action from the table below, object identity, requester principal, attempt
nonce, and expected executor build. The publisher re-derives the locator and target, opens the
object itself, validates requester entitlement to that exact IH/PM, and signs only the observed
transition. Arbitrary bytes, batch roles, caller absolute paths, unknown fields, or a request made
through a different executor build are rejected. The corresponding negative oracle mutates each
field independently and proves zero manifest, anchor, object, or receipt change.

A table row expressly marked as a fixed coupled socket-endpoint transition remains one role and
one action, not a caller-supplied batch. The publisher deterministically derives its non-requestable
endpoint component, validates both exact before-states, and binds both into the same protected
prepared record before `Start`, `Stop`, or `Restore`. It then observes the state and endpoint
effects and binds both to one receipt. No other row may induce an unlisted object effect; missing,
replaced, or ambiguous endpoint authority leaves both objects unchanged.

Post-bootstrap publisher transport is fixed and authenticated. Linux listens only on the
root-owned `AF_UNIX SOCK_SEQPACKET` endpoint
`/run/substrate-lifecycle-publisher-v1.sock` with mode `0600`, owner/group root. The unprivileged
control never opens it directly. For each request it creates a fresh `SOCK_SEQPACKET` socketpair
and invokes the fixed installed Linux executor through `/usr/bin/sudo -C 4 --` as a one-request
relay; the relay joins the original control peer through `SO_PEERCRED` and retained
`/proc/<pid>/exe`, while the publisher separately joins the root relay PID and the same fixed
executor file identity. A relay cannot batch, rewrite, or persist a request. macOS accepts only XPC connections to
Mach service `com.substrate.lifecycle.publisher.v1`, with an XPC dictionary containing exactly
`protocol_version=1` and canonical `request_bytes`; it joins the audit-token euid and the control
binary's one closed code requirement: exactly `cdhash H"<40 lowercase hex>"` for an ad-hoc
development/evidence image, or exactly `anchor apple generic and identifier
"com.substrate.lifecycle.publisher.v1" and cdhash H"<40 lowercase hex>"` for a properly signed
production image. In either form the embedded CDHash must exact-join the separately measured
control image CodeDirectory identity, artifact SHA-256, physical identity, and retained
source/provenance record before `SecCodeCheckValidity` admits the audited XPC or retained FD3 peer; no
generic identifier-only, caller-supplied, or extra-clause requirement is authority. Windows accepts only
`\\.\pipe\SubstrateLifecyclePublisherV1` with a protected SYSTEM-plus-committed-SID DACL, one
little-endian-u32-framed canonical JSON request/response of at most 1 MiB, and joins impersonated
client SID, `GetNamedPipeClientProcessId`, retained control-image handle/file identity, exact
build-evidence SHA-256, and DACL for the fixed control binary.
Every transport has a durable nonce replay table: an identical completed nonce returns the exact
prior response; a reused nonce with different bytes, concurrent duplicate, extra frame, oversized
message, unauthenticated peer, or disconnect before response fails closed. Publisher request states
are exactly `RequestAccepted -> SignedAnchorPrepared -> PreparedRecordDurable ->
CurrentAnchorAdvancedDurable -> ResponseReconstructible -> Delivered`; the signed record and
all inputs needed to reproduce the response are durable before advancing the current anchor; after
that CAS the exact response cache is durably published before delivery. Every boundary has
before/after kill tests.

Role parameters are closed. `version` matches `[A-Za-z0-9][A-Za-z0-9._-]{0,63}`, is neither `.`
nor `..`, contains no slash, backslash, colon, NUL, Windows trailing dot/space, or non-round-tripping
single component. The host-binary set is exactly `substrate`, `substrate-shim`,
`substrate-forwarder`, `host-proxy`, `world-service`, `substrate-world-service`,
`substrate-gateway`, `substrate-lifecycle-control`, `substrate-lifecycle-linux`,
`substrate-lifecycle-macos`, `substrate-lifecycle-windows`, and
`substrate-lifecycle-unix`. Unix projections are exactly `config.yaml`, `env.sh`,
`manager_init.sh`, `manager_env.sh`, and `manager_hooks.yaml`; Windows projections are exactly
`A\config.yaml` and `A\forwarder\forwarder.toml`; unit kinds are exact `service`, `socket`, and
`socket-drop-in` where the row permits them. Synthetic auth is exactly `H/.codex/auth.json`, where
H is the canonical intended-principal home from IH. No free-form kind, path, unit, service,
binary, version, profile, VM/distro, process, endpoint, or target survives decoding.

`ManagedArtifactRoleV1` is exactly the following closed enum. `A`, canonical principal home `H`,
IH, PM, SID, instance/machine identity, and version are validated typed inputs named in the row;
no variant accepts an arbitrary path, command, unit, service, process name, PID, or object kind.
`Create` means create only after durable before-state; `Replace` means replace only the exact prior
R3 generation; `Remove` means remove only an exact R3-created object; `Restore` means reproduce the
recorded before-state; `Enable`/`Disable` and `Start`/`Stop` mean only the exact independently
recorded service or process state transition. Only a row explicitly marked as a fixed coupled
socket-endpoint transition may include its deterministically derived endpoint before/after state
in the same prepared transaction and receipt; that endpoint exposes no separate action. Every
other implicit file/object mutation is forbidden. An action not listed is a decoder error.

| Serialized role variant and typed parameter | Domain and exact derived target | Object / dependencies | Allowed actions | Sole packet |
|---|---|---|---|---|
| `unix.prefix.payload-version(version)` | A-local: `A/versions/<version>` | closed directory subtree; before all prefix links | Create, Replace, Remove | UNIX |
| `unix.prefix.bin-entry(binary)` | A-local: `A/bin/<binary>`; `binary` is one of `substrate`, `substrate-shim`, `substrate-forwarder`, `host-proxy`, `world-service`, `substrate-world-service`, `substrate-gateway`, or the five `substrate-lifecycle-*` names | file/link; depends on payload version | Create, Replace, Remove, Restore | UNIX |
| `unix.prefix.shim-tree` | A-local: `A/shims` | closed no-follow tree; depends on installed shim executor and payload | Create, Replace, Remove | UNIX |
| `unix.prefix.linux-guest-cache(binary)` | A-local: `A/bin/linux/<binary>`; binary is `substrate`, `world-service`, or `substrate-gateway` | regular file; depends on exact mapped Lima source | Create, Replace, Remove | UNIX |
| `unix.prefix.dev-env` | A-local: `A/dev-shim-env.sh` | regular projection; depends on bin/shims | Create, Replace, Remove, Restore | UNIX |
| `unix.prefix.projection(kind)` | A-local kind exactly `config.yaml`, `env.sh`, `manager_init.sh`, `manager_env.sh`, or `manager_hooks.yaml` | regular file; exact bytes/metadata or recorded before-state; user-modified content fails closed | Create, Replace, Remove, Restore | UNIX |
| `unix.prefix.runtime-script(kind)` | A-local kind exactly `scripts/substrate/world-enable.sh`, `scripts/substrate/install-substrate.sh`, `scripts/substrate/world-deps.yaml`, `scripts/mac/lima-warm.sh`, `scripts/mac/lima/substrate.yaml`, or `scripts/mac/lima/substrate-dev.yaml` | regular file/link; depends on payload version | Create, Replace, Remove, Restore | UNIX |
| `unix.prefix.run-directory` | A-local: `A/run` | directory; only exact created-empty removal | Create, Remove, Restore | UNIX |
| `unix.principal.profile-snippet(kind)` | H-relative kind exactly `.bashrc`, `.bash_profile`, `.profile`, `.zshrc`, `.zprofile`, or `.config/fish/config.fish` | bounded marker block inside regular file; H comes from IH, never ambient HOME | Create, Replace, Remove, Restore | UNIX |
| `windows.prefix.payload-version(version)` | A-local: `A\versions\<version>` | closed reparse-free tree | Create, Replace, Remove | WIN |
| `windows.prefix.bin-entry(binary)` | A-local: `A\bin\<binary>.exe`; binary is the Windows member of the same closed host-binary set | regular file; depends on payload version | Create, Replace, Remove, Restore | WIN |
| `windows.prefix.shim-tree` | A-local: `A\shims` | closed reparse-free tree | Create, Replace, Remove | WIN |
| `windows.prefix.profile-helper(kind)` | A-local: `A\substrate-profile.ps1` or `A\dev-substrate-profile.ps1` | regular file | Create, Replace, Remove, Restore | WIN |
| `windows.prefix.projection(kind)` | A-local kind exactly `config.yaml` or an R3-generated forwarder configuration committed by PM | regular file; user modification is preserving mismatch | Create, Replace, Remove, Restore | WIN |
| `windows.prefix.forwarder-log-directory` | exact A-local `A\forwarder\logs`, equal to `MappingState.ForwarderLogDir` | reparse-free directory with exact before identity/DACL; only an R3-created empty directory may be removed | Create, Remove, Restore | WIN |
| `windows.principal.profile-snippet(kind)` | exact `PROFILE.CurrentUserAllHosts` or `PROFILE.CurrentUserCurrentHost` captured under committed SID | bounded marker block; target identity and before bytes must be recorded before mutation | Create, Replace, Remove, Restore | WIN |
| `linux.host.binary(kind)` | fixed `/usr/local/bin/substrate-world-service` or `/usr/local/bin/substrate-gateway` | regular file | Create, Replace, Remove, Restore | LINUX |
| `linux.host.acl-helper` | fixed `/usr/libexec/substrate/substrate-apply-socket-acl` | regular file | Create, Replace, Remove, Restore | LINUX |
| `linux.host.unit(kind)` | fixed service, socket, or socket-drop-in path under `/etc/systemd/system` for `substrate-world-service` | regular file only; reload precedes separate state action | Create, Replace, Remove, Restore | LINUX |
| `linux.host.service-state(kind)` | exact `substrate-world-service.service` or `substrate-world-service.socket` | enabled/active state; depends on corresponding unit; `kind=socket` is the fixed coupled endpoint transition and precommits the exact `linux.host.socket` before/after state in the same prepared record and receipt | Enable, Disable, Start, Stop, Restore | LINUX |
| `linux.host.directory(kind)` | fixed `/run/substrate`, `/var/lib/substrate`, `/var/lib/substrate/world-deps`, or `/var/lib/substrate/world-deps/bin` | directory; only created-empty removal | Create, Remove, Restore | LINUX |
| `linux.host.socket` | fixed `/run/substrate.sock` | non-requestable observed Unix-socket component; depends on socket unit and group; exact identity/owner/group/mode/ACL before/after is owned only by `linux.host.service-state(socket)` | none; coupled observation only | LINUX |
| `linux.host.group` | exact group `substrate` | group database entry | Create, Remove, Restore | LINUX |
| `linux.host.membership(principal)` | exact IH principal in group `substrate` | membership; depends on group | Create, Remove, Restore | LINUX |
| `linux.host.linger(principal)` | exact IH principal linger state | login-manager state | Create, Remove, Restore | LINUX |
| `linux.host.acl-bridge(kind,principal)` | exact socket, state-tree traversal, or world-deps read-only ACL for IH principal | ACL entry; depends on its exact object | Create, Remove, Restore | LINUX |
| `linux.host.synthetic-auth` | exact `H/.codex/auth.json`, H from IH, for the closed smoke role | regular file; never ambient invoking HOME | Create, Remove, Restore | LINUX |
| `linux.publisher.executor` | fixed `/usr/libexec/substrate/substrate-lifecycle-linux` | root:root `0755` verified retained executable | Create, Replace, Restore | LINUX |
| `linux.publisher.executor-parent` | fixed `/usr/libexec/substrate` | root:root `0755`; only a recorded test-created empty directory may retire | Create, Remove | LINUX |
| `linux.publisher.lifecycle-container` | fixed `/var/lib/substrate/.substrate-lifecycle-v1` | root:root `0700`; only a recorded test-created empty directory may retire | Create, Remove | LINUX |
| `linux.publisher.state-directory` | fixed `/var/lib/substrate/.substrate-lifecycle-v1/publisher` | root:root `0700`; only a recorded test-created empty directory may retire | Create, Remove | LINUX |
| `linux.publisher.service-unit` | fixed `/etc/systemd/system/substrate-lifecycle-publisher-v1.service` | root:root `0644`; depends on executor | Create, Restore | LINUX |
| `linux.publisher.socket-unit` | fixed `/etc/systemd/system/substrate-lifecycle-publisher-v1.socket` | root:root `0644`; depends on executor | Create, Restore | LINUX |
| `linux.publisher.endpoint` | fixed `/run/substrate-lifecycle-publisher-v1.sock` | non-requestable observed root:root `0600` Unix `SOCK_SEQPACKET`; depends on socket unit; owned only by protected publisher bootstrap/retirement socket-state transition; unprivileged callers use the attested sudo relay | none; bootstrap/retirement coupled observation only | LINUX |
| `linux.publisher.signing-key` | fixed `/var/lib/substrate/.substrate-lifecycle-v1/publisher/signing-key.v1` | root:root `0600`; non-replaceable Ed25519 terminal key | Create | LINUX |
| `linux.publisher.current-anchor` | fixed `/var/lib/substrate/.substrate-lifecycle-v1/publisher/current-anchor.v1.json` | root:root `0600` complete `LifecyclePublisherProtectedStateV1`; signed current anchor/counter plus optional full signed prepared record in one monotonic CAS | Create, Replace | LINUX |
| `linux.publisher.bootstrap-intent` | fixed `/var/lib/substrate/.substrate-lifecycle-bootstrap-intent-v1.<scope-sha256>.json`, atomically linked from unnamed root-only storage under retained enumerated state-root descriptor | root:root `0600`; exact host-bootstrap crash join, never capsule-derived or a new state root | Create, Replace | LINUX |
| `linux.publisher.service-state(kind)` | exact publisher service or socket unit | internal protected bootstrap/retirement state only; `kind=socket` precommits and receipts the fixed coupled `linux.publisher.endpoint`; service state cannot propagate to socket state | Enable, Disable, Start, Stop, Restore; not an ordinary caller request | LINUX |
| `mac.publisher.executor` | fixed `/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1` | root:wheel `0755`, exact designated requirement | Create, Replace, Restore | MAC |
| `mac.publisher.plist` | fixed `/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist` | root:wheel `0644`, fixed Mach-service declaration | Create, Restore | MAC |
| `mac.publisher.signing-key` | explicitly opened `/Library/Keychains/System.keychain`; service label `com.substrate.lifecycle.v1`, exact application tag `<scope-id>:signing-key` | software P-256 private key; sufficiently privileged root may export it; canonical SPKI remains signed-state identity | Create | MAC |
| `mac.publisher.current-anchor` | System-Keychain service `com.substrate.lifecycle.v1`, account `<scope-id>:current-anchor` | complete `LifecyclePublisherProtectedStateV1`; signed current anchor/counter plus optional full signed prepared record in one monotonic CAS | Create, Replace | MAC |
| `mac.publisher.bootstrap-intent` | System-Keychain service `com.substrate.lifecycle.v1`, account `<scope-id>:bootstrap-intent` | protected exact host-bootstrap intent/state CAS | Create, Replace | MAC |
| `mac.publisher.guest-pairing-record(challenge_id)` | System-Keychain service `com.substrate.lifecycle.v1`, account `<scope-id>:guest-pairing:<challenge-id>` | signed generation-CAS host record; terminal consumed record is retained | Create, Replace | MAC |
| `mac.publisher.mach-service` | fixed `com.substrate.lifecycle.publisher.v1` | non-requestable observed XPC endpoint from fixed plist; owned only by protected publisher bootstrap/retirement service-state transition | none; bootstrap/retirement coupled observation only | MAC |
| `mac.publisher.service-state` | exact LaunchDaemon `com.substrate.lifecycle.publisher.v1` | internal protected bootstrap/retirement state precommitting and receipting the fixed coupled `mac.publisher.mach-service` | Enable, Disable, Start, Stop, Restore; not an ordinary caller request | MAC |

The macOS installer realizes `mac.publisher.service-state` only after the retained-FD3 direct
publisher bootstrap has completed and before the first mapped-lifecycle XPC request. The control
binary sends no serialized request: it passes an EOF-only retained stream through the existing
fixed `sudo -C 4` executor boundary, and the helper admits the kernel-observed control peer against
the installed provenance before selecting one of two compiled operations. Installation accepts
only the exact root:wheel helper, plist, and provenance identities, exact compiled plist bytes,
completed bootstrap intent, signed initial anchor, and issued Stage-1 capsule. It CAS-precommits the
fixed System-Keychain service-state record, executes exactly `/bin/launchctl bootstrap system
/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist`, observes the definite presence
of `system/com.substrate.lifecycle.publisher.v1`, remeasures all files, and only then commits the
installed receipt. `launchctl print` is used only as a success/non-success presence observation;
its body is never parsed as structured service identity. A registered service with only a
precommit is an effect-visible/CAS gap and is preserved rather than adopted. The idempotent state
is registered plus an exact committed receipt plus the exact receipted files; absence with a
committed installed receipt, an indeterminate observation, or any foreign/partial identity stops.

Retirement CAS-precommits an intent against that exact receipt, executes exactly `/bin/launchctl
bootout system/com.substrate.lifecycle.publisher.v1`, proves definite absence, deletes only the
three exact receipted fixed files, proves service and files absent, and finally commits `retired`.
An interrupted precommit may retry the fixed bootout or continue exact deletion only from the
corresponding observed state. Foreign, mismatched, ambiguous, or unreceipted state is never
removed. Neither transition uses `kickstart`, a caller-supplied label/domain/path/action, or a
generic service-manager API.
| `mac.lima.instance` | fixed R2-selected instance plus finalized PM machine identity; Create additionally requires its exact protected `LimaStageOneAuthorizationV1` | Lima instance; pre-existing disposition is never removed | Create, Start, Stop, Remove, Restore | MAC |
| `mac.lima.staged-workspace` | guest fixed `/var/lib/substrate/staged-workspace/current` | closed guest tree; depends on exact PM instance | Create, Replace, Remove, Restore | MAC |
| `mac.lima.guest-binary(kind)` | guest fixed `/usr/local/bin/substrate-world-service`, `/usr/local/bin/substrate-gateway`, or `/usr/local/bin/substrate` | regular file; depends on mapped instance | Create, Replace, Remove, Restore | MAC |
| `mac.lima.guest-unit(kind)` | guest fixed service or socket unit under `/etc/systemd/system` | regular file; reload precedes separate state action | Create, Replace, Remove, Restore | MAC |
| `mac.lima.guest-service-state(kind)` | exact `substrate-world-service.service` or `substrate-world-service.socket` | enabled/active state; `kind=socket` is the fixed coupled endpoint transition and precommits the exact `mac.lima.guest-socket` before/after state in the same prepared record and receipt; service state cannot propagate to socket state | Enable, Disable, Start, Stop, Restore | MAC |
| `mac.lima.guest-directory(kind)` | fixed `/run/substrate`, `/run/substrate/substrate-gateway-runtime`, `/var/lib/substrate`, `/var/lib/substrate/.substrate-lifecycle-v1`, `/var/lib/substrate/staged-workspace`, or `/usr/libexec/substrate` | exact directory; only created-empty removal | Create, Remove, Restore | MAC |
| `mac.lima.guest-group` | exact guest group `substrate` | group database entry | Create, Remove, Restore | MAC |
| `mac.lima.guest-membership(principal)` | PM guest account in exact group `substrate` | membership; depends on guest group | Create, Remove, Restore | MAC |
| `mac.lima.guest-private-home` | PM exact guest private home | HOME packet rules; exact created-empty descriptor-relative rollback only | Create, Remove, Restore | MAC |
| `mac.lima.layout-sentinel` | fixed `/etc/substrate-lima-layout` | exact regular file bytes/version plus before state | Create, Replace, Remove, Restore | MAC |
| `mac.lima.publisher-executor` | guest fixed `/usr/libexec/substrate/substrate-lifecycle-linux` | root:root `0755`, source-receipt digest; one object | Create, Replace, Restore | MAC |
| `mac.lima.publisher-state-directory` | guest fixed `/var/lib/substrate/.substrate-lifecycle-v1/publisher` | root:root `0700`; only a recorded evidence-created empty directory may retire | Create, Remove | MAC |
| `mac.lima.publisher-service-unit` | guest fixed `/etc/systemd/system/substrate-lifecycle-publisher-v1.service` | root:root `0644`; one object | Create, Restore | MAC |
| `mac.lima.publisher-socket-unit` | guest fixed `/etc/systemd/system/substrate-lifecycle-publisher-v1.socket` | root:root `0644`; one object | Create, Restore | MAC |
| `mac.lima.publisher-endpoint` | guest fixed `/run/substrate-lifecycle-publisher-v1.sock` | non-requestable observed root:root `0600` `SOCK_SEQPACKET`; owned only by protected guest bootstrap/retirement socket-state transition; host-signed guest requests use the exact root relay | none; bootstrap/retirement coupled observation only | MAC |
| `mac.lima.publisher-signing-key` | guest fixed publisher `signing-key.v1` | root:root `0600`, non-replaceable Ed25519 key and inactive until paired generation one; one object | Create | MAC |
| `mac.lima.publisher-current-anchor` | guest fixed publisher `current-anchor.v1.json` | complete `LifecyclePublisherProtectedStateV1`; signed anchor/counter plus optional full signed prepared record; one object | Create, Replace | MAC |
| `mac.lima.publisher-bootstrap-intent` | guest exact `/var/lib/substrate/.substrate-lifecycle-pairing-intent-v1.<scope-challenge-sha256>.json`, atomically linked from unnamed root-only storage | exact retained `GuestPublisherPairingGuestIntentV1` with private seed confined to root; inside enumerated state root, never inside the absent publisher directory | Create, Replace | MAC |
| `mac.lima.publisher-service-state(kind)` | guest exact publisher service or socket | internal protected guest bootstrap/retirement state only; `kind=socket` precommits and receipts the fixed coupled `mac.lima.publisher-endpoint`; service state cannot propagate to socket state | Enable, Disable, Start, Stop, Restore; not an ordinary caller request | MAC |
| `mac.lima.guest-socket` | guest fixed `/run/substrate.sock` | non-requestable observed Unix-socket component; depends on guest socket unit and group; exact identity/owner/group/mode/ACL before/after is owned only by `mac.lima.guest-service-state(socket)` | none; coupled observation only | MAC |
| `mac.host.known-hosts-entry` | exact A-local `A/lima_known_hosts`, one PM instance host-key entry derived from `limactl show-ssh` | bounded exact line/file before state; no global known-hosts mutation | Create, Replace, Remove, Restore | MAC |
| `mac.attempt.rendered-units-tree(attempt_id)` | retained current-attempt `mktemp -d` identity used by unit render | closed no-follow tree, never a pre-existing path | Create, Remove | MAC |
| `mac.host.forward-socket` | PM exact A-scoped transport host socket | Unix socket; depends on exact mapping and forwarder | Create, Remove, Restore | MAC |
| `mac.host.ssh-forwarder` | exact child handle, executable identity, argv commitment, start identity, and PM mapping | process; depends on Lima instance and both socket identities | Start, Stop | MAC |
| `windows.publisher.executable` | fixed `%ProgramFiles%\Substrate\Lifecycle\substrate-lifecycle-windows.exe` | SYSTEM/TrustedInstaller writable, Administrators read/execute, exact retained file identity/build-evidence SHA-256/DACL; Authenticode is optional metadata, not authority | Create, Replace, Restore | WIN |
| `windows.publisher.product-directory` | fixed `%ProgramFiles%\Substrate` | protected frozen publisher DACL; only a recorded evidence-created empty directory may retire | Create, Remove | WIN |
| `windows.publisher.install-directory` | fixed `%ProgramFiles%\Substrate\Lifecycle` | protected DACL; only a recorded evidence-created empty directory may retire | Create, Remove | WIN |
| `windows.publisher.service-registration` | fixed service `SubstrateLifecyclePublisherV1` | exact fixed image path and protected service DACL | Create, Restore | WIN |
| `windows.publisher.signing-key` | LocalMachine CNG key `SubstrateLifecyclePublisherV1` | non-exportable P-256 key | Create | WIN |
| `windows.publisher.current-anchor` | fixed `HKLM\SOFTWARE\Substrate\LifecycleV1\Anchors\<scope-id>` value | SYSTEM-write-only complete `LifecyclePublisherProtectedStateV1`; signed current anchor/counter plus optional full signed prepared record in one monotonic CAS | Create, Replace | WIN |
| `windows.publisher.bootstrap-intent` | fixed protected HKLM value `SOFTWARE\Substrate\LifecycleV1\BootstrapIntents\<scope-id>` | SYSTEM-write-only exact host-bootstrap intent/state CAS with registry flush | Create, Replace | WIN |
| `windows.publisher.registry-container(kind)` | fixed HKLM key `SOFTWARE\Substrate`, `SOFTWARE\Substrate\LifecycleV1`, or its exact `Anchors`, `BootstrapIntents`, `Pairings`, or `Pairings\<scope-id>` child | protected frozen DACL; exact parent-ordered creation and only recorded evidence-created empty-key retirement | Create, Remove | WIN |
| `windows.publisher.guest-pairing-record(challenge_id)` | fixed protected HKLM value `SOFTWARE\Substrate\LifecycleV1\Pairings\<scope-id>\<challenge-id>` | signed generation-CAS host record; terminal consumed record is retained | Create, Replace | WIN |
| `windows.publisher.endpoint` | fixed `\\.\pipe\SubstrateLifecyclePublisherV1` | non-requestable observed pipe with SYSTEM-plus-committed-SID DACL; owned only by protected publisher bootstrap/retirement service-state transition | none; bootstrap/retirement coupled observation only | WIN |
| `windows.publisher.service-state` | exact service `SubstrateLifecyclePublisherV1` | internal protected bootstrap/retirement state precommitting and receipting the fixed coupled `windows.publisher.endpoint` | Enable, Disable, Start, Stop, Restore; not an ordinary caller request | WIN |
| `windows.host.forwarder-process` | PM exact executable, SID, instance, machine ID, pipe, argv, and start identity | process | Start, Stop | WIN |
| `windows.host.forwarder-pid-record` | PM fixed forwarder PID-record path | regular file that is evidence only, never standalone kill authority | Create, Replace, Remove | WIN |
| `windows.host.forwarder-pipe` | PM exact `\\.\pipe\<committed-name>` | named pipe; depends on exact forwarder process | Create, Remove, Restore | WIN |
| `windows.wsl.instance` | PM exact already-registered distro plus guest machine ID | pre-existing platform object; absence/import/unregister are preserving scope-expansion stops | Start, Stop, Restore | WIN |
| `windows.wsl.guest-binary(kind)` | guest fixed world-service or gateway path under `/usr/local/bin` | regular file; depends on exact WSL instance | Create, Replace, Remove, Restore | WIN |
| `windows.wsl.guest-unit(kind)` | `kind` is exactly `service` or `socket`; guest fixed `/etc/systemd/system/substrate-world-service.service` or `/etc/systemd/system/substrate-world-service.socket` from the matching `scripts/wsl/units/` source below; service bytes are the deterministic PM-bound render | root:root `0644` regular file; service manifest entry binds template-source digest, exact PM render inputs, and rendered-service digest; socket entry binds source digest and equal unchanged installed digest; service depends on world-service binary, group, and pre-created state/runtime directories; socket depends on group and service unit; file/parent fsync precedes daemon reload | Create, Replace, Remove, Restore | WIN |
| `windows.wsl.guest-service-state(kind)` | `kind` is exactly `service` or `socket`; exact `substrate-world-service.service` or `substrate-world-service.socket` | enabled/active state; daemon reload after both unit bytes are durable and before any enable/start/restore; `kind=socket` is the fixed coupled endpoint transition and precommits the exact `windows.wsl.guest-socket` before/after state in the same prepared record and receipt | Enable, Disable, Start, Stop, Restore | WIN |
| `windows.wsl.guest-directory(kind)` | guest fixed `/run/substrate`, `/var/lib/substrate`, `/var/lib/substrate/.substrate-lifecycle-v1`, or `/usr/libexec/substrate` | root:root exact directory; pre-existing disposition restores, only recorded created-empty removal | Create, Remove, Restore | WIN |
| `windows.wsl.guest-group` | exact guest group `substrate` with precommitted before-state | group database entry; required before socket activation | Create, Remove, Restore | WIN |
| `windows.wsl.guest-membership(principal)` | exact PM-mapped guest principal in group `substrate` | membership; depends on exact guest group and mapped SID/instance/machine identity | Create, Remove, Restore | WIN |
| `windows.wsl.guest-socket` | guest fixed `/run/substrate.sock` | non-requestable observed root:`substrate` `0660` Unix-socket component; depends on exact socket unit, guest group, and `/run/substrate`; exact identity/ACL before/after is owned only by `windows.wsl.guest-service-state(socket)` | none; coupled observation only | WIN |
| `windows.wsl.publisher-executor` | guest fixed `/usr/libexec/substrate/substrate-lifecycle-linux` | root:root `0755`, source-receipt digest; one object | Create, Replace, Restore | WIN |
| `windows.wsl.publisher-state-directory` | guest fixed `/var/lib/substrate/.substrate-lifecycle-v1/publisher` | root:root `0700`; only a recorded evidence-created empty directory may retire | Create, Remove | WIN |
| `windows.wsl.publisher-service-unit` | guest fixed `/etc/systemd/system/substrate-lifecycle-publisher-v1.service` | root:root `0644`; one object | Create, Restore | WIN |
| `windows.wsl.publisher-socket-unit` | guest fixed `/etc/systemd/system/substrate-lifecycle-publisher-v1.socket` | root:root `0644`; one object | Create, Restore | WIN |
| `windows.wsl.publisher-endpoint` | guest fixed `/run/substrate-lifecycle-publisher-v1.sock` | non-requestable observed root:root `0600` `SOCK_SEQPACKET`; owned only by protected guest bootstrap/retirement socket-state transition; host-signed guest requests use the exact root relay | none; bootstrap/retirement coupled observation only | WIN |
| `windows.wsl.publisher-signing-key` | guest fixed publisher `signing-key.v1` | root:root `0600`, non-replaceable Ed25519 key and inactive until paired generation one; one object | Create | WIN |
| `windows.wsl.publisher-current-anchor` | guest fixed publisher `current-anchor.v1.json` | complete `LifecyclePublisherProtectedStateV1`; signed anchor/counter plus optional full signed prepared record; one object | Create, Replace | WIN |
| `windows.wsl.publisher-bootstrap-intent` | guest exact `/var/lib/substrate/.substrate-lifecycle-pairing-intent-v1.<scope-challenge-sha256>.json`, atomically linked from unnamed root-only storage | exact retained `GuestPublisherPairingGuestIntentV1` with private seed confined to root; inside enumerated state root, never inside the absent publisher directory | Create, Replace | WIN |
| `windows.wsl.publisher-service-state(kind)` | guest exact publisher service or socket | internal protected guest bootstrap/retirement state only; `kind=socket` precommits and receipts the fixed coupled `windows.wsl.publisher-endpoint`; service state cannot propagate to socket state | Enable, Disable, Start, Stop, Restore; not an ordinary caller request | WIN |

For Windows/WSL, the newly allowlisted service template and socket file contain exactly the
following UTF-8 bytes with LF line endings and one final LF. No alternate executable, drop-in, or
generator is allowed. The service template is:

```ini
[Unit]
Description=Substrate World Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/substrate-world-service
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock
Environment=SUBSTRATE_HOME=${R3_WSL_SUBSTRATE_HOME}
Environment=SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${R3_WSL_HOST_CONTEXT_COMMITMENT}
Environment=SUBSTRATE_PLATFORM_BOOTSTRAP_MAPPING_V1=${R3_WSL_PLATFORM_BOOTSTRAP_MAPPING_V1}
UMask=0027
WorkingDirectory=/var/lib/substrate
StandardOutput=journal
StandardError=journal
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE

[Install]
WantedBy=multi-user.target
```

The socket file is:

```ini
[Unit]
Description=Substrate World Service Socket

[Socket]
ListenStream=/run/substrate.sock
SocketMode=0660
SocketUser=root
SocketGroup=substrate
DirectoryMode=0750
RemoveOnStop=yes
Service=substrate-world-service.service

[Install]
WantedBy=sockets.target
```

The service unit deliberately has no `Group=`, `RuntimeDirectory=`, `StateDirectory=`,
`DirectoryMode=`, `PermissionsStartOnly=`, or other implicit directory-creation/metadata directive.
The manifest executor creates or exact-joins `/run/substrate` and `/var/lib/substrate` first and
proves their identities/metadata; service activation may not chown, chmod, create, or recurse into
either root or the protected lifecycle subtree.

Every R3-managed world socket source is equally exact: the Linux
`SOCKET_UNIT_CONTENT` in `scripts/linux/world-provision.sh`, the release Linux socket source in
`scripts/substrate/install-substrate.sh::provision_linux_world`, the tracked Lima socket file, and
the new WSL socket file omit `PartOf=`, `BindsTo=`, `Requires=`, and every equivalent propagation
edge from service state to socket state. A service-role stop/restart/restore must leave socket
enabled/active state and endpoint identity unchanged. Socket activation/removal is owned only by
the corresponding socket-service-state coupled transaction. Publisher service/socket/endpoint
roles are not ordinary requests: their protected bootstrap/retirement intent precommits all fixed
components, and their socket-state transition and endpoint observation share one protected state
transition and receipt. No publisher unit may introduce a service-to-socket propagation edge.

`render_managed_wsl_world_service_unit_v1` takes only the already-validated PM's exact
`realized_substrate_home`, 64-lowercase-hex host-context commitment, and canonical base64url
`PlatformBootstrapMappingV1`. It never reads process environment, HOME, account databases, a
default distro, or a caller path. For each UTF-8 input byte it emits the byte unchanged only for
`A-Z`, `a-z`, `0-9`, `/`, `.`, `_`, `:`, or `-`; it emits `%` as `%%`; every other byte is emitted
as lowercase `\xhh`. It replaces each of the three literal `${R3_WSL_*}` markers exactly once,
rejects a missing/duplicate/unresolved marker, and otherwise preserves the template bytes exactly.
The rendered digest and all three source PM values are manifest-bound. Hostile ambient
`SUBSTRATE_HOME` or mapping variables, substitution metacharacters, a different guest account/
home, or any render/digest mismatch stops before unit or service mutation.

The WIN executor separately binds the service-template source digest, the three exact PM render
inputs, and the rendered-service digest, then writes and reads back only the rendered service bytes.
It binds the socket source digest and requires the installed socket digest to equal it, then writes
and reads back only those unchanged socket bytes. Template bytes or any unresolved marker at the
installed service target, a rendered or otherwise modified socket target, and any source/rendered
digest conflation reject. Through retained target handles it reads back bytes plus root:root `0644`,
fsyncs both files and `/etc/systemd/system`, runs one exact `systemctl daemon-reload`, and only then
restores/enables/starts the exact service/socket states. Wrong path, bytes, digest, owner, mode,
symlink/type, parent identity, reload result, or state order preserves the old units and returns the
closed failure status.

The serialized variant/parameter pair, authority domain, derived target, observed object type,
dependency set, and requested action must match one row exactly. The manifest parent/name fields
are observations only. Legacy unit names, wildcard `.substrate*`, arbitrary profile paths,
alternate Lima/WSL instances, alternate pipes/transports, and unlisted binaries are not variants
and therefore cannot become deletion authority.

There is deliberately no role for rewriting guest `/etc/resolv.conf`, restarting DNS providers,
installing toolchains/packages, importing a missing WSL distro, removing a WSL installation tree,
unregistering a WSL distro, or enumerating/signaling the hidden Unix owner-helper process. The MAC
packet tombstones the current in-guest build/DNS fallback; missing exact
`ExecutorBuildEvidenceV1`-joined guest artifacts stop before mutation. The WIN packet replaces the live WSL
guard only with an existing-instance gate after PM validation; absent-instance/import code remains
unreachable and preserving. Reaching any of these omitted actions is `BLOCKED_SCOPE_EXPANSION`.
The fixed Linux ACL-helper file remains a distinct managed artifact; it never supplies authority
over the excluded retained-worker/session process. Unix `--kill-live-processes` stops before
enumeration or signal, and uninstall may continue only after a separate non-authorizing
observation proves no live owner helper consumes the prefix.

Every entry requires:

- `object_id`, `logical_role`, `object_type`, `scope`, `parent_identity`, `name_identity`, and
  platform-specific physical identity;
- disposition exactly one of `created`, `pre_existing_preserved`, or `adopted`;
- expected bytes hash or link/redirect target plus exact mode, owner, group, ACL/security
  descriptor, flags, service/unit state, socket/pipe identity, process image/start identity,
  VM/distro machine identity, or other type-specific metadata;
- canonical before-state, intended after-state, restoration state, dependency object IDs, and an
  explicit closed-subtree member list when a recursively represented installation tree is used;
- entry lifecycle state, last durable transition, and terminal/retryable/human-repair error class.

An observation cannot create an `adopted` entry. The value remains schema-reserved, but every R3
decoder rejects it and no R3 packet may create or accept `ManagedAdoptionAuthorizationV1`.
Pre-R3/unmanifested collisions are preserved and the task returns `BLOCKED_SCOPE_EXPANSION` with a
manual-adoption handoff. R3 tests exercise `adopted` only as a preserving rejection;
`pre_existing_preserved` entries are never removed. A carrier, generated
file, present `install_state.json`, path/name/version, PID, timeout, current bytes, prefix match,
or manifest carrier location is not authority.

Every lifecycle capsule retains an exact executor copy recorded by role, bytes digest, file
identity, owner/mode/DACL, build/source identity, and supported action set. The executor remains
until terminal head and receipts are durable. The per-platform typed executor opens and retains
the manifest/head and object descriptors/handles, derives the target only from
`ManagedArtifactRoleV1`, performs the destructive syscall/platform operation, observes it, and
publishes the receipt without a validator-to-script gap. Scripts may only pass the authenticated
context, manifest generation, closed role, and requested transition.

Prefix uninstall terminates as `Uninstalled` while retaining only
`A/.substrate-lifecycle-v1/` (or the Windows separator equivalent), its executor, immutable
generation, head, and receipts. Reinstall advances the lineage; repeat uninstall exact-joins the
terminal head and is a non-mutating success. Requiring no A-local control capsule is
`BLOCKED_SCOPE_EXPANSION`, because deleting the last receipt would reintroduce an unprovable crash
gap. A disposable test scope may remove its exact capsule only as a component of the external
test-retirement protocol above, after the signed retirement receipt is durable outside that
capsule; ordinary evidence publication alone supplies no removal authority.

Shared capsules contain `shared-claims.v1.json`. For each shared role it stores the exact
before-state/restoration obligation, compatible desired-state digest, generation/head, and a
claimant set of installation ID, context commitment, and manifest digest. Install CAS-adds its
claim before mutation; a compatible later claimant joins without adoption. Uninstall marks its
claim releasing; non-last release cannot change the object. The last claim alone may restore or
remove it, and the set becomes empty only after its action receipt is durable. First-creator
before-state lives in this shared record, so creator-first uninstall cannot erase the restoration
obligation. Conflicts or CAS ambiguity fail before action.

Publication is:

```text
complete canonical bytes
  -> no-follow create unique same-parent temp regular file
  -> write all bytes and fsync temp
  -> exact-revalidate trusted parent and destination absence/expected generation
  -> atomic rename with no unintended replacement
  -> fsync parent
  -> ManifestDurable
```

Any failure before parent fsync is not durable deletion authority. Retry must exact-join the temp,
destination generation, bytes, and parent; otherwise it preserves all state and requires repair.
Missing, malformed, duplicate, tampered, stale, partial, unsupported, cross-prefix, cross-
principal, cross-platform, or cross-instance manifests fail before an artifact action.

### `R3-TEMP-ROLLBACK-01` — pre-manifest current-attempt temporary trees

Unix release download and Windows release download create their temporary roots before any
lifecycle executor/publisher/manifest is available. They therefore do not pretend to be
`ManagedArtifactRoleV1`. `CurrentAttemptTempRollbackV1` is an in-process safe-rollback witness only:
attempt UUIDv7, retained trusted parent descriptor/handle and physical identity, random leaf name,
`created_by_this_attempt=true`, retained no-follow/reparse-free child descriptor/handle and physical
identity, a closed ordered member list with each member's relative components/type/physical
identity, and `cleanup_armed` state. The installer creates the root through the retained parent,
opens it immediately without following links, records identity before any download, registers each
created file/directory before publishing bytes into it, and extracts only through a bounded helper
that records and rejoins every closed descendant. A new/unrecorded child, hard link, symlink/reparse
point, replacement, cross-device member, nonempty unclosed entry, or lost descriptor/handle makes
cleanup preserving and reports the residue.

The states are exactly `Absent -> RootCreated -> RootOpened -> CleanupArmed -> MembersClosed ->
RollbackStarted -> MembersRemoved -> RootRemoved`. Cleanup retains the root handle, removes exact
recorded leaves in reverse order with descriptor-relative Unix operations or handle/file-ID joined
Windows operations, and removes the root only when exact and empty. It never calls path-only
`rm -rf`/`Remove-Item -Recurse`. Ordinary handled-error and EXIT/finally paths may finish this
synchronous rollback. Abrupt process death leaves unknown residue unchanged on rerun; name/prefix/
age/apparent content never recovers authority. Kill tests bracket root create/open, cleanup arm,
every member registration/write/extract, every leaf removal, root empty proof, and root removal.

### `R3-CLASS-01` — mutually explicit lifecycle classes

The action class is canonical input and cannot be inferred from a verb or target:

| Class | Exact meaning | Packets that may own it |
|---|---|---|
| safe rollback | reverse only a rejected effect created by this still-synchronous attempt while retained descriptors/handles and exact identity remain; includes HOME candidate, `CurrentAttemptTempRollbackV1`, and manifest-recorded current-attempt staging effects | HOME; packet-local current-attempt fences in MAC, WIN, and UNIX |
| uninstall | reverse dependency-ordered removal/restoration of exact R3-created accepted objects under a durable manifest and receipts; terminal protected capsule/publisher persists | LINUX, MAC, WIN, UNIX for only their table roles |
| replacement/migration | replace only an exact prior R3 generation after old/new identities and restoration state are durable; pre-R3 or schema-reserved adoption is never this class | LINUX, MAC, WIN, UNIX for only their table roles |
| shared-platform teardown | last compatible shared claimant stops/removes/restores the exact shared process/service/socket/VM role after consumer release and durable receipts; pre-existing state restores, and WSL import/install-tree deletion/unregister are excluded | LINUX host shared roles, MAC exact owned Lima/forwarder roles, WIN forwarder/pipe and guest-service roles |

MANIFEST defines and validates the enum but performs none of the four. A packet may own multiple
classes only for its disjoint role rows and exact executor/action symbols above. Class mismatch,
same target claimed by two packets, uninstall presented as rollback, legacy replacement presented as
migration, or one-prefix request for shared teardown is a decoder/review stop before mutation.

### `R3-ACTION-01` — mutation and crash/retry protocol

The allowed durable states are:

```text
RecordedBefore
  -> ManifestPrepared
  -> ManifestDurable
  -> ActionStarted
  -> ActionObserved
  -> ReceiptDurable
  -> Committed
```

Replacement binds both old and new identities before `ActionStarted`. Uninstall walks reverse
dependency order. An action receipt binds manifest digest/generation, entry ID, action, attempt,
pre-observation, exact effect observation, post-observation, restoration status, error class,
preallocated receipt ID/name, protected prepared-record digest, and allocated publisher counter.
The protected publisher signs that canonical payload with `LifecycleSignatureV1` before capsule
publication. It never contains a digest of its own bytes or of a future index/head/anchor. After
canonical serialization of the signed record the publisher computes
`action_receipt_artifact_sha256`; the exact capsule `ManagedActionReceiptIndexV1`, head, and
publisher-anchor CAS defined in `R3-MANIFEST-01` bind that external hash. The receipt uses the same
temp-fsync/rename/parent-fsync publication rule, and the hash cannot authorize an action until
those bytes are durable and the full index/head/anchor commit completes.

Kill points bracket prepared-record/counter allocation, receipt payload completion, signature,
receipt temp create/write/fsync/rename/parent-fsync, retained-file signature verification and
external hash, receipt-index temp/write/fsync/rename/parent-fsync CAS, head CAS, publisher-anchor
CAS, and final response. A fresh process at each boundary must exact-join only the protected
prepared record, manifest-preallocated signed receipt, and current index/head/anchor revisions or
stop preserving. Same-principal receipt replacement, valid signature with a wrong counter or
prepared-record/action observation, wrong key/algorithm, and an unsigned receipt are preserving
negatives.

At every kill point, retry does exactly one of: join the already-complete intended state and
publish the missing receipt; finish the same recorded action; restore only current-attempt changes
from the durable before-state; or stop unchanged. PID existence, liveness, elapsed time, path
existence, name, version, apparent byte equality, or timeout cannot select a branch.

Rollback preserves evidence until all dependent restoration is proven. Teardown stops exact
processes before endpoints and quiesces/disables services before provider removal. Restoration
follows the typed dependency DAG: it restores provider directories, files, unit/drop-in bytes and
metadata, and groups first; membership follows its group, and ACL follows its exact object; linger
and other independent providers follow their recorded dependencies. It then runs the exact platform
reload and restores socket/service enabled/active state and remaining dependents. Created empty
parents are removed only after dependents, and the terminal capsule,
head, executor, and receipts are retained. Failure is:

- `retryable` only with exact authority/effect identity still available;
- `terminal_preserving` for mismatch, ambiguity, replacement, nonempty state, unsupported
  observation, or authority failure; or
- `human_repair_required` when an external/platform action may have occurred but cannot be
  exact-joined.

No error is converted to success merely because the target is absent. Repeat install/uninstall
must join the exact committed state and emit no broadened action.

### `R3-PRESERVE-01` — exact removal and restoration

Prohibited actions include prefix/wildcard/glob deletion, ambient-home selection, recursive
private-home candidate cleanup, following links/reparse points, broad process-name or substring
kills, stale-PID authority, default/ambient VM or distro selection, unowned WSL/Lima teardown,
shared-parent cleanup, and recursive deletion without an exact closed manifest subtree.

Links and targets are different entries. Pre-existing files, links, units, sockets, processes,
groups, memberships, ACLs, linger state, VMs/distros, platform-control state, and shared artifacts
are preserved or restored to exact before-state. Schema-reserved `adopted` input is rejected
unchanged with `BLOCKED_SCOPE_EXPANSION`. Any installer change to
existing bytes, target, mode, owner, ACL/security descriptor, enabled/active state, process,
instance, or membership requires before/after/restoration proof; ambiguity rejects the change.

Installer/uninstaller symmetry includes first install, repeat install, partial install, exact
retry, first uninstall, repeat uninstall, uninstall after partial failure, and reinstall. A
successful terminal state has no unreceipted manifest entry and no changed unrelated object.

### Platform clauses

- `R3-LINUX-01` covers exact helper, gateway, shim/payload projection, unit/drop-in, service/socket,
  runtime/state directory, group/membership, ACL bridge, linger, and synthetic-auth lifecycle with
  full restoration. It cannot claim passive-health remediation.
- `R3-MAC-01` consumes only the existing IH/PM and exact A-scoped host socket. VM destroy,
  staged-tree/unit/socket replacement, SSH `StreamLocalBindUnlink`, timeout kill/wait, handle-drop
  teardown, and retry require manifest/receipt identity. `auto_select`, alternate endpoints,
  VSock/TCP fallback, and new instance/principal/prefix selection are forbidden.
- `R3-WIN-01` separates A-local objects from SID + registered-distro + machine-ID + pipe shared
  objects. Stop/kill/pipe/PID/terminate actions require the full joined identity and disposition.
  Guest provisioning separately manifest-binds `/run/substrate`, group `substrate`, the exact PM-
  mapped guest membership, socket unit, and root:`substrate` `0660` `/run/substrate.sock`, with
  dependency-ordered before-state capture, readback, and restoration.
  The R2 PM can exist only for an already-registered exact distro, so R3 may start/stop and restore
  that instance but may neither import nor unregister it; absence is a scope-expansion stop.
  Wildcards, defaults, ambient Known Folder state, process names, and stale PIDs are forbidden.

### Review, proof, and publication wall

Every implementation subject is reviewed by fresh read-only authority/security,
lifecycle/convergence, and allowlist/evidence lenses. The V1 bounded review validator is necessary
but not sufficient. The causal review protocol itself reaches terminal `CLEAN` when the record is
`complete`, its last verdict is `clean`, and unresolved P1/P2 are zero; P3/P4 neither trigger nor
extend that cascade. Separately, this R3 planning/publication contract is stricter: its final
subject and reports must have zero unresolved P1-P4 before publication. An edit changes the subject
fingerprint and reruns every affected lens. Reviewers do not author the subject they review.

Every packet additionally requires exact path/symbol/test allowlist comparison, applicable
format/schema/link checks, `git diff --check`, staged secret/credential inspection,
`gitnexus_detect_changes()`, clean index/worktree/untracked state after commit, and live
remote/base/ancestry verification immediately before its one normal fast-forward push.

No `cargo test --workspace` expected-failure inventory is frozen by this control pack. Approximate
claims about how many workspace failures are "expected" are not authority. Any later task that
requires a full-workspace run must either pass that gate or first land a separately reviewed exact
baseline contract naming the command, environment, discovered/pass/fail/ignored counts, exact
failure identities and stable signatures, provenance, and allowed transition rules.

Terminal receipt status mapping is closed:

| Condition | Required status |
|---|---|
| target/base/tree/ancestry drift | `BASE_DRIFT` |
| dirty protected checkout, prohibited action, repository contradiction, or nonrecoverable allowlist violation | `BLOCKED_CONTRADICTION` |
| new authority root/domain/selector, legacy adoption, or out-of-scope behavior is required | `BLOCKED_SCOPE_EXPANSION` |
| any unresolved P1-P4 or invalid review lineage | `BLOCKED_REVIEW` |
| an available native run fails, evidence is invalid, or restoration parity fails | `BLOCKED_NATIVE_EVIDENCE` |
| required native platform/privilege is unavailable | `BLOCKED_PLATFORM_HANDOFF_REQUIRED` |
| a required delegated task has not reached terminal state | `BLOCKED_TASK_NOT_TERMINAL` |
| implementation or successor authority has not been granted | `AUTHORITY_REQUIRED` |
| all gates landed, remote-equal, clean, and `0/0` | `LANDED_CLEAN` |

No other receipt status, including `BLOCKED_ENVIRONMENT`, is permitted.

Read-only platform evidence tasks use the separate closed
`codex.top-level-evidence-receipt.v1` protocol and
`orchestrate-top-level-tasks/scripts/validate_evidence_receipt.py`. Their only clean status is
`EVIDENCE_CLEAN`; their blocked statuses must be accepted by that validator, including
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` and `BLOCKED_NATIVE_EVIDENCE`. Every clean evidence receipt
binds an exact published source with `source.live_remote == source.commit`, the exact tree/ref and
artifact digest. That skill validator has no successor field or `--expected-next` option and must
never be described or invoked as validating one. The separate evidence artifact binds restoration
and `gated_successor`; every evidence task and closeout independently validate that field with the
MANIFEST-owned `scripts/ci/validate_r3_native_evidence.py <artifact>
--expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree>
--expected-source-ref <ref> --expected-gated-successor <value>`, then validate the receipt with
the skill validator's single positional receipt argument and join its `evidence.artifact_sha256` to the
artifact digest. Evidence tasks never use `LANDED_CLEAN`, create a commit, or push.

The final native evidence record has schema owner `substrate.r3-native-evidence`, version 1, and
binds evidence ID, source commit/tree/ref, platform/OS/architecture/tool versions, product project
discovery, clean-host prerequisites, baseline manifest/digest, allowed and prohibited actions,
ordered commands with exit/status/output digests, artifact manifest/digest, restoration manifest/
digest, sudo-credential invalidation where applicable, unrelated-state sentinels, guest-pairing
ticket/canonical SPKI/fingerprint/signature, host-record and guest-intent/key/transcript state
digests, TTY identities, `intent_publication_capability`, and operator-confirmation commitment when
applicable (never the reusable confirmation input), result, and
gated successor. Its receipt independently binds the evidence digest and exact post-test parity.
Static review cannot satisfy a native gate.

`intent_publication_capability` is null only for a platform gate that has no Linux publisher
bootstrap. Otherwise it binds the exact host or Lima/WSL guest machine identity, enumerated
`/var/lib/substrate` state-root physical identity, filesystem/mount/device identity, same-filesystem
result, kernel/API versions, `O_TMPFILE` mode support, `linkat(AT_EMPTY_PATH)` support, effective
root capability, ordered preflight argv/status/output digests, disposable probe file/parent
identities, unlink/parent-fsync observations, and post-probe baseline parity. Discovery first uses
read-only filesystem/kernel queries; the capability operation itself runs only in an exact
harness-owned disposable directory on the same filesystem, after baseline capture, and removes/
fsyncs it before any publisher action. It must prove unnamed create/write/fsync, one exact absent-
name atomic link, retained-file verification, unlink, and no residual entry. A different mount,
named-temp/rename substitute, unsupported flag/API/filesystem, missing privilege, leftover probe,
or inability to prove parity returns `BLOCKED_PLATFORM_HANDOFF_REQUIRED` before bootstrap, with
the full source/platform/filesystem/privilege/probe/handoff facts; it never weakens publication.

Every evidence dispatch binds product project ID
`2ccb802f-301c-4af4-9bd5-51d22808f0a2`, discovery by exact origin plus target ref, a fresh dispatch
nonce, source implementation task thread/host, evidence task thread/host, return meta thread/host,
evidence ID, source commit/tree/ref, and gated successor. The evidence task rejects any mismatch,
records those correlation fields in the artifact and receipt, and sends the validated native JSON
to the exact bound return task with `send_message_to_thread` as its final tool action. No current
planning-task thread/host value is reusable by a future dispatch; all future task IDs and nonces
must be freshly bound by that dispatch.

If a required platform is unavailable, the only result is
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` with source checkpoint commit/tree/ref, project discovery
criteria, prerequisites, exact commands/actions, prohibited actions, expected artifacts/digests,
restoration steps, evidence schema, gated successor, and continuation prompt. A gate is never
weakened.

For this planning increment, the reviewed subject is the ordered six-file set `00` through `05`.
The SHA-256 subject fingerprint is the SHA-256 of the byte-for-byte POSIX `sha256sum` output for
those six paths in lexical order: each line is the lowercase 64-hex content digest, two spaces, the
repository-relative path, and LF. Review reports and the cycle record are excluded so they can
attest to that fixed subject. Publication remains one planning commit and one ordinary fast-forward
push.
**Source provenance:** extracted from [`../04-contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates`](../04-contracts-and-gates.md#a11d-5r3-lifecycle-contracts-and-gates), baseline lines 8677–9999
**Relocation note:** repository-relative Markdown targets were rebased as needed to preserve their original repository destinations after relocation.
**Baseline span SHA-256:** `94f7ebf518682b306989dc52117e39926c9e33975a5ac14c030f9286adb7103f`
