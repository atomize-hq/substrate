**Kind:** architecture
**Stable ID:** `A1.1d-5R3-family`
**Canonical for:** R3 lifecycle ownership architecture bundle
**Status:** canonical
**Authority scope:** exact extracted 01 architecture source body only
**Source span:** [`01-target-architecture.md#a11d-5r3-lifecycle-ownership-architecture`](../01-target-architecture.md#a11d-5r3-lifecycle-ownership-architecture) lines 1294–1662
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R3 lifecycle ownership architecture

## A1.1d-5R3 lifecycle ownership architecture

This section is the controlling R3 architecture at source
`4ceecd50e20d822dda7cbd8f0e1bef4ccad65d8e` / tree
`8ed5dc7a354b731016a103b68091864b6a09223a`. It adds no runtime architecture now. It freezes the
only architecture later implementation may realize.

### Authority domains and action classes

Deletion authority is never derived from a selected path alone. `InstallBootstrapContextV1` and
the platform mapping select an authority domain, but a version, name, prefix, PID, timeout,
carrier, generated projection, apparently equal bytes, or `AlreadyExists` result is not deletion
provenance. R3 separates four action classes:

| Class | Sole authority | Required result |
|---|---|---|
| Safe rollback | exact live current-attempt object identity plus retained authority descriptors/handles | remove only the current attempt's unaccepted effect, or leave it unchanged |
| Uninstall | a durable exact managed-artifact entry in the same committed context and lifecycle state | remove or restore exactly that entry, idempotently |
| Replacement/migration | a durable entry for the old object plus a separately published entry for the replacement transaction | exact-join, finish, or restore without adopting an observed object |
| Shared-platform teardown | a durable manifest entry whose platform/principal/instance/transport scope exactly joins live observation | tear down only the recorded shared object; ambiguity requires human repair |

One implementation packet may own more than one class only where its path, symbol, test, and proof
fences are disjoint from every other packet. PI-012, PI-026, and PI-095 remain singularly owned by
the Unix integration packet; Linux and macOS packets own only the separately named provider action
fences in the crosswalk and may not edit those Unix orchestrator sections.

### Descriptor-bound private-home candidate rollback

A private-home candidate is rollback-eligible only during the exact synchronous attempt that
received `Created`, while the trusted parent descriptor is still live, after a no-follow child
open succeeds and the opened descriptor's physical identity is retained. Cleanup must rejoin the
same parent/name observation to that descriptor, prove directory type, exact owner and mode,
acceptable ACL state, unchanged physical identity, and emptiness, then perform one
descriptor-relative empty-directory operation. It never recurses and never follows a link.

`AlreadyExists`, pre-existing state, unknown creation provenance, a replacement, nonempty state,
wrong type/owner/mode/ACL, ambiguous lookup, unsupported observation, or a descriptor/path
mismatch disarms cleanup and leaves state unchanged. The current test
`signaled_creation_child_preserves_created_candidate_provenance` proves that an interrupted child
can leave an invalid candidate with unknown provenance. R3 therefore promises no automatic
crash-residue cleanup. A rerun remains fail-closed and unchanged. If product convergence requires
new durable pre-acceptance provenance, the packet stops as `BLOCKED_SCOPE_EXPANSION`.

### Managed-artifact manifest

The product owns one versioned canonical managed-artifact manifest contract; installers,
uninstallers, platform helpers, and runtime replacement paths are consumers, not alternate
authorities. A manifest commits at least:

- schema owner/version and canonical byte encoding;
- host install commitment, selected prefix, intended principal, platform kind, and any exact
  platform mapping commitment;
- object ID, object type, authority domain, logical role, canonical parent/name or platform
  handle, and physical identity where the platform exposes one;
- `pre_existing`, schema-reserved `adopted`, or `created` disposition; every R3 decoder rejects
  `adopted`, and only a later separately authorized protocol could define it;
- expected bytes or link target and exact mode, owner, group, ACL/security descriptor, unit
  content, service state, socket identity/ACL, process image/start identity, instance identity, or
  equivalent type-specific metadata;
- before-state, intended after-state, restoration record, dependency edges, and closed manifest
  subtree membership when recursive removal is ever proposed;
- lifecycle state, attempt ID, action receipt, last durable transition, and retry classification;
  and
- manifest temp identity, checksum, durable publication point, and parent-directory durability.

The canonical publication sequence is prepare complete bytes, write a new same-parent temporary
regular file without following links, sync it, exact-revalidate the destination/parent, publish by
one atomic non-replacing or explicitly state-transitioning rename, then sync the parent. Only the
post-parent-sync state is durable deletion authority. Missing, malformed, partial, duplicate,
tampered, stale, cross-principal, cross-prefix, cross-instance, or unsupported manifests fail
closed. The present `install_state.json`, carrier, generated file, PID file, project-head marker,
or byte similarity is consistency evidence only until migrated through this contract.

The digest supplies integrity only. Trust comes from a locator derived independently from the
already-validated IH/PM, the exhaustive role/domain/target/action table in `R3-MANIFEST-01`,
no-follow descriptor/handle traversal, exact parent and file identity, and a distinct
OS-protected publisher with a monotonic signed anchor. The manifest cannot supply its own locator
or turn an arbitrary path into a system target. A same-user capsule, coherent digest/head rewrite,
or replayed valid generation is rejected because it cannot advance the protected publisher
counter or reproduce the signed request/action record.

R3 adds no top-level shared state root. It uses only these exact authority-domain capsules:

| Domain | Exact capsule locator | Publisher and trust metadata |
|---|---|---|
| Prefix-local Unix/macOS host | `A/.substrate-lifecycle-v1/` | intended-principal cache under a trusted A descriptor; authoritative only with the Linux root or macOS System-Keychain publisher anchor |
| Prefix-local Windows | `A\.substrate-lifecycle-v1\` | committed-SID cache with reparse-free traversal; authoritative only with the LocalSystem/CNG/HKLM publisher anchor |
| Linux host or Lima/WSL guest system state | `/var/lib/substrate/.substrate-lifecycle-v1/` | root-owned no-follow chain; root publisher key/counter/current anchor in its `publisher/` child |
| macOS Lima host-shared state | `PM.host_platform_control_root/.substrate-lifecycle-v1/<mapping-commitment>/` | cache bound to exact PM identity; System Keychain service `com.substrate.lifecycle.v1` and LaunchDaemon `com.substrate.lifecycle.publisher.v1` are authoritative |
| Windows host-shared state | `PM.host_platform_control_root\.substrate-lifecycle-v1\` | cache bound to exact PM identity; LocalSystem service `SubstrateLifecyclePublisherV1`, LocalMachine CNG key, and SYSTEM-write-only HKLM counter are authoritative |

Each capsule contains immutable `manifest.<generation>.json`, exact immutable
`receipts/<generation>/receipt.<preallocated-id>.json` objects, append-only per-generation CAS
`action-receipts.<generation>.v1.json`, and
`head.v1.json`. The receipt index binds each manifest-planned receipt ID to its fixed filename,
external artifact SHA-256, retained file/parent identity, revision, and prior index digest. The head
binds installation or shared-scope ID, strictly increasing generation, manifest digest, lifecycle
state, prior head digest, and exact receipt-index revision/digest. Publication durably writes and
parent-fsyncs the receipt, externally hashes it, CAS-publishes and parent-fsyncs the index, CASes
the head, and only then advances the protected anchor. It compare-and-swaps the exact
opened head and the protected publisher counter, then retains a signed
`LifecyclePublisherAnchorV1` binding request, action, executor, and prior anchor. Linux trusts only
the root publisher; macOS trusts only the code-designated root LaunchDaemon plus System Keychain;
Windows trusts only the exact LocalSystem service plus LocalMachine CNG/HKLM anchor; Lima/WSL
requires the matching host and guest anchors. The installing principal is a client, never the
publisher. A valid old generation, coherently rehashed capsule, counter rollback, signature/key
substitution, parent replacement, locator substitution, unprivileged publication, or role scope
escape is rejected. There is no unsigned or user-keystore fallback. Prefix uninstall intentionally
retains only the closed lifecycle capsule with an
`Uninstalled` terminal head; reinstall advances that lineage. This bounded control residue is not
a product artifact or a general state root. Requiring complete A absence instead would require new
durable receipt authority and is `BLOCKED_SCOPE_EXPANSION`. Product lifecycle never removes the
publisher or last protected anchor. Exact-absence native bootstrap uses only an independently
authorized disposable evidence scope: before component removal the publisher signs a test-
retirement receipt into a descriptor-bound external harness store; the receipt contains no self
digest. The harness fsyncs it, externally hashes the canonical bytes, and returns a separately
signed/fsynced acknowledgement that the publisher verifies and durably records before teardown.
Only that receipt-plus-acknowledgement can resume reverse-order removal and prove baseline absence.
The ordinary evidence receipt is emitted
after retirement and parity; no product manifest or uninstaller receives retirement authority.
The harness first hashes the canonical bootstrap core with its optional retirement slot fixed to
null. Its key, external store descriptor identity, and exact core-bound test-retirement
authorization digest are generated and durably committed before bootstrap, then copied into the
final bootstrap authorization and generation-one anchor. This one-way construction has no digest
cycle. Teardown rejects any later-created, substituted, core-mismatched, or non-byte-identical
retirement authorization.

### AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

The R3 macOS publisher remains the fixed, code-designated root LaunchDaemon, but its signer is one
**software** P-256 private key stored in the explicitly opened legacy
`/Library/Keychains/System.keychain`. The publisher verifies the path returned for that exact
`SecKeychainRef`; uses `kSecUseKeychain` only to target adds and a one-element
`kSecMatchSearchList` for reads, updates, and deletes; and forces noninteractive Security.framework
operation. The canonical service is `com.substrate.lifecycle.v1`, and the exact scope-bound tag is
`<scope-id>:signing-key`. Zero matches may create once. One match must revalidate its tag, service
label, private P-256 type/size, permanent/signing attributes, and canonical SPKI. Duplicate,
substituted, missing-under-wrapper, or SPKI-mismatched state preserves the wrapper and stops.
Retirement refuses key deletion while the protected wrapper survives, deletes only the exact tag
from that exact Keychain, and verifies final absence. Protected-wrapper CAS takes the exact
`<scope-id>:current-anchor` durable lock before key/SPKI validation; retirement takes that same lock
across wrapper-absence validation, validated-item-reference deletion, and final absence. The
documented `kSecMatchItemList` selector prevents a post-validation tag-wide delete.

This is an explicit R3 threat-posture change, not a hardware claim: a sufficiently privileged root
process with System-Keychain access may export the private key. The security joins that remain
authoritative are the fixed LaunchDaemon, audit-token/designated-requirement admission, canonical
SPKI binding, P1363 low-S signatures, signed protected wrapper, monotonic CAS, receipt/retry joins,
and preserving-first failures. There is no default, ambient, user, file, environment, caller-
selected, unsigned, or software-file fallback. Secure Enclave/Data Protection Keychain and a
session-capable user LaunchAgent signer are deferred hardening only; this correction does not
authorize, claim, or dispatch either design.

Publisher bootstrap itself is not inferred from installation bytes. A direct-interactive hidden
control command verifies the published implementation receipt and executor, reads an exact terminal
confirmation, and constructs one expiring `PublisherBootstrapAuthorizationV1` bound to IH/PM,
principal, platform scope, source commit/tree/ref, executor digest/signature, exact component
manifest, and nonce. It immediately delivers those bytes over an authenticated OS elevation
channel: Unix socketpair plus `sudo`/`SO_PEERCRED`/retained executable identity, or a nonce-named
Windows pipe plus UAC, impersonated SID, client PID, retained executable handle/file identity,
exact native `ExecutorBuildEvidenceV1` SHA-256, and protected DACL. Authenticode is optional
release metadata and never bootstrap authority. Each native evidence harness builds its host
executor and isolated Linux guest executor from the exact remote-equal source checkpoint, records
source/lock/toolchain/target/artifact/code identity in `ExecutorBuildEvidenceV1`, and removes the
build scope before baseline capture. The privileged executor
copies itself first to a fixed protected service path; it never executes privileged lifecycle work
from A, PM, staging, or a build tree. Bootstrap advances component-by-component from durable intent
through executor, endpoint metadata, key, counter zero, active service, and generation-one anchor;
partial or pre-existing publisher identity only exact-joins the same nonce or stops. The global
greenfield flag applies to that publisher identity, while every enumerated parent/dependency role
separately records `ExactAbsentCreate` or an exact retained `ExactPreExistingDependency` before-
state. Both dispositions remain in the complete component set; only current-attempt-created
components enter the retirement DAG.

Lima and WSL guest bootstrap does not trust a key or child channel supplied in the request. The
protected host publisher signs a one-use ticket binding its public-key fingerprint/current anchor,
canonical P-256 SPKI DER bytes, challenge, IH/PM machine identity, source, and exact
`ExecutorBuildEvidenceV1` artifact, plus the canonical signed current-anchor public record. Its
attested control binary displays the complete fingerprint/challenge only on the host controlling
terminal. Before mutation, the exact root guest executor opens its own controlling TTY and requires
the operator to enter that full fingerprint/challenge plus the fixed pairing literal; automation,
stdin, argv, environment, file, or carrier confirmation is forbidden. Only then does the guest
hash the supplied SPKI, join it to the full pin, verify that key's signed current-anchor record,
and verify the fixed P1363 low-S ticket signature. macOS System Keychain or Windows HKLM holds the
host pairing record. Before the first guest effect, the guest exact-joins that protected host
record and receives its one-use signed mutation grant; the same CAS marks the preallocated
reservation consumed, so a merely issued ticket cannot race an unused-reservation retirement.
The guest intent is an atomic unnamed-file-to-exact-link publication under
the retained enumerated root-owned `/var/lib/substrate` state root, contains the root-confined
Ed25519 seed/nonce, and precedes publisher-
directory creation; the final inactive key is materialized from that seed only after the signed
transcript. Exact restart joins completed records, transcript, both anchors, and terminal ticket
consumption; pre-intent restart re-prompts, while an effect-visible/identity-record-not-durable
component-create gap preserves state and stops. The retained
`limactl shell`/`wsl -d` channel carries the hello and signed transcript after authentication; it is
not the trust source. A post-intent replacement channel may relay only the exact durable hello/
transcript and is recorded as observation. Ticket replay, child/channel-only authority, alternate-
child substitution, key drift, non-TTY confirmation, or any PM/source/machine/artifact/transcript
mismatch stops before guest publisher mutation.

Before disposable host bootstrap, the harness alone preallocates the sole MAC/WIN guest challenge,
host pairing-record component, and complete guest component set; Linux reserves none. Evidence
uses a separate null-slot guest ticket-core retirement authorization before the intent is linked. The commitment is copied
through host record, intent, hello, transcript, guest generation one, and later anchors. Externally
receipted guest teardown removes the exact guest DAG and seed intent before host teardown may
remove the terminal pairing record or the exact host DAG; protected evidence is retained until the
next layer's acknowledgement and baseline parity are durable. A rejection, expiry, cancellation,
EOF, or kill before the guest-mutation grant may advance `Reserved` or `TicketIssued` to
`ProvenUnused` only through a signed host proof that permanently revokes the ticket and exact-
matches every guest target to its precommitted before-state with zero guest-owned effect. The
external harness must durably acknowledge that proof before the host publisher may restore/remove
the precommitted pairing-record component and retire the reservation; liveness, timeout, PID, and
unacknowledged observations never suffice.

After bootstrap, Linux uses one root-owned `SOCK_SEQPACKET` service endpoint and peer executable
attestation; macOS uses the fixed LaunchDaemon XPC Mach service with audit-token plus designated-
requirement attestation; Windows uses the fixed LocalSystem named pipe with protected DACL,
impersonated SID, client PID, retained control-image file identity, exact build-evidence hash, and
DACL attestation. Each accepts one bounded canonical
request and response, durably deduplicates nonces, and advances signed prepared record, current
anchor, reconstructible response, and delivery states without a validator-to-executor gap. Exact
component paths, parameter grammars, symbols, dependencies, and crash transitions are frozen in
`03` and `04`.

Shared capsules additionally contain `shared-claims.v1.json`. For each closed shared role it
records the exact before-state/restoration obligation, compatible desired-state digest, and a
generation-CAS claimant set of `(installation_id, host_context_commitment, manifest_digest)`.
Install durably adds a claim before first shared mutation; a later compatible installation joins
without adopting the object. Uninstall marks its claim releasing, but does not remove it until its
prefix no longer consumes the object. Non-last release preserves shared state. The last claim
alone may restore/remove it, and only after the teardown receipt is durable may the claimant set
advance to empty. Creator-first uninstall transfers no authority between prefix manifests because
the before-state/restoration obligation is owned by the shared capsule from first creation.
Conflicting desired state or any CAS/claim ambiguity fails before mutation.

R3 has no automatic legacy adoption surface. The `adopted` disposition is schema-reserved, but
every R3 decoder rejects it and no packet in this index may create or accept
`ManagedAdoptionAuthorizationV1`. Every pre-R3/unmanifested collision,
including a legacy shim/version/unit with apparently matching bytes, is preserved and returns
`BLOCKED_SCOPE_EXPANSION` with an adoption/manual-repair handoff. Managed replacement/migration in
R3 therefore means only a prior R3 manifest generation.

Scripts do not validate and then destructively act on a pathname. A retained executor copy and
digest are entries in the lifecycle capsule and remain until the terminal head/receipt is durable.
The platform packet's typed executor derives the target from `ManagedArtifactRoleV1`, opens and
validates the manifest/head and object under the same descriptors/handles, performs the syscall or
platform operation without releasing them, observes the effect, and publishes the receipt.
Shell/PowerShell scripts only construct authenticated typed requests and sequence executors.
Linux, macOS, Windows, and Unix executors are distinct binaries with disjoint source fences.

### Lifecycle state machines and rollback order

Candidate rollback states are `Absent -> CreatedUnopened -> OpenedUnaccepted -> Accepted` with a
separate terminal `UnknownProvenance`; only `OpenedUnaccepted` may enter `RollbackArmed`, and only
an exact empty rejoin may enter `RolledBack`.

A managed action moves through `RecordedBefore -> ManifestPrepared -> ManifestDurable ->
ActionStarted -> ActionObserved -> ReceiptDurable -> Committed`. `ReceiptDurable` is only the
immutable receipt plus parent fsync; `Committed` additionally requires exact receipt-index, head,
and publisher-anchor CAS. A crash exact-joins the manifest-preallocated receipt filename and hash,
then finishes the missing CAS stages; enumeration never recovers authority. Replacement additionally binds
the old and new identities before either destructive step. Uninstall moves each entry through the
same states in reverse dependency order and records restoration before erasing the authority
needed to prove it. A crash may exact-join and finish, roll back only current-attempt effects, or
stop unchanged. Liveness, elapsed time, a PID, or an object name never chooses a transition.

Teardown order is dependents before providers: stop an exact managed process by retained handle or
fully joined process identity; preserve its exit/evidence; remove its exact socket/pipe endpoint;
quiesce and disable the exact unit/service; then remove only exact created objects in reverse
dependency order. Restoration reverses teardown direction and follows dependency order: restore
provider directories, files, links, unit/drop-in bytes and metadata, and groups first; restore
membership only after its group and ACLs only after each exact protected object; then restore
linger, run the exact platform reload, restore socket/service enabled and active state, restore
remaining dependents, remove only created empty directories, and durably record completion. The
typed dependency DAG controls; prose list order never overrides it.

Socket activation is one closed exception to a single-object effect, not a batch request. A
socket-service state role that explicitly names a fixed coupled endpoint precommits the exact
state and endpoint before-states in one protected prepared record. Start, stop, or restore then
observes both effects and commits one receipt. The endpoint component has no independently
requestable action, and a missing or mismatched endpoint precondition preserves both objects.
Every managed world socket unit omits `PartOf=` and every equivalent service-to-socket propagation
relationship, so stopping or restoring the service role cannot alter socket state or endpoint.
Publisher endpoint activation is confined to the protected bootstrap/retirement machine with the
same precommitted coupled-effect rule and is not an ordinary caller request.
An error is:

- retryable only while all authority evidence remains exact and no ambiguous effect occurred;
- terminal-preserving for mismatch, replacement, nonempty state, unsupported observation, or
  missing/tampered authority; or
- human-repair-required after a partially observed external/platform action whose result cannot
  be exact-joined.

No cleanup may erase the manifest, action receipt, before-state, child/process evidence, or
platform identity needed by a later step.

### Exact platform boundaries

Unix prefix-local ownership covers exact payload/version/bin/shim/cache/profile and managed link
objects under one selected prefix. Links and targets are distinct entries. No prefix match, glob,
ambient home, marker-only profile rewrite, or recursive tree removal is permitted without a
closed manifest subtree.

Linux privileged ownership separately covers exact helpers, gateways, units, drop-ins, sockets,
service/runtime/state directories, group and membership changes, ACL bridge entries, linger
state, and synthetic smoke-auth effects. Existing state requires exact before/after/restoration.
It does not own passive-health remediation.

macOS ownership joins the existing host context and finalized Lima platform mapping. A VM,
staging tree, unit, socket, child, known-hosts file, or forwarding endpoint is removable only by
its manifest identity. For an absent already-selected Lima instance, the one pre-PM path is a
macOS-publisher-signed `LimaStageOneAuthorizationV1` over IH, account-derived control root, fixed
instance/profile, source and attempt; it is durable before create and must attach the observed
machine identity and finalize PM before guest projection. It selects no alternate instance and
cannot authorize WSL. SSH-UDS activation may occur only after PI-101, PI-113, and PI-114 bind the
A-scoped socket unlink, `StreamLocalBindUnlink`, timeout kill/wait, handle-drop teardown, retry,
and convergence. No alternate prefix, principal, instance, endpoint, transport, or Lima root may
be selected. The existing in-guest build fallback, toolchain/package install, DNS-service restart,
and `/etc/resolv.conf` rewrite are tombstoned; they have no managed role because they are passive-
health remediation. A missing exact guest artifact matching the native evidence task's
`ExecutorBuildEvidenceV1` fails before guest mutation. Guest
group/membership/private home, binaries, units, enabled/active state, directories, layout sentinel,
A-local known-hosts entry, staging, sockets, and instance disposition each have separate roles and
restoration records.

Windows ownership distinguishes A-local version/bin/profile/shim/config/log state from
SID + registered-instance + guest-machine-ID + pipe scoped shared state. A wildcard, default
distro, process name, ambient PID file, stale PID, or pipe name alone never authorizes stop,
unregister, kill, or removal. Reparse points are never followed. The R2 PM is constructible only
for an already-registered exact distro with a machine ID, so R3 may start/stop and restore that
instance but may not import, remove its install tree, or unregister it. Missing-instance creation
is `BLOCKED_SCOPE_EXPANSION`. Shared forwarders may be stopped or restored only by their exact
manifest, shared claim, and protected publisher anchor. The same Windows packet owns the WSL
guest's exact `/run/substrate` directory, `substrate` group, PM-mapped guest membership, socket
unit, and root:`substrate` `0660` `/run/substrate.sock` roles. It records each before-state before
mutation and restores/removes only the exact manifest identity; an unlisted group, membership,
socket, ACL, or ambient WSL principal is preserving scope expansion.

Unix and Windows release-download roots exist before a manifest publisher is available, so they
use only `CurrentAttemptTempRollbackV1`: retained trusted parent/root descriptors or handles,
exact physical identity, a closed registered descendant list, reverse no-follow/handle-joined
removal, and an exact-empty root removal. Abrupt death, an unregistered child, replacement,
reparse/symlink, or lost handle preserves the residue; a name, age, prefix, or apparent content
never recovers rollback authority. Windows forwarder logs are not temporary: only the exact
PM-derived `A\forwarder\logs` role may be created, restored, or removed, with pre-existing state
preserved and ambient `LOCALAPPDATA` ignored.

Across every platform, first and repeat install, partial failure, exact retry, first and repeat
uninstall, uninstall after partial failure, and reinstall must converge while preserving every
unrelated or pre-existing file, link, unit, socket, process, group, membership, ACL, linger state,
VM/distro, platform-control object, and shared artifact.

### Publication and evidence architecture

Provider implementation and native evidence are separate authority layers. LINUX, MAC, and WIN
each land one reviewed implementation commit before a read-only native evidence task is dispatched
against that exact live-remote-equal commit/tree/ref. A following evidence-closeout packet may
commit only the validated evidence/receipt and bounded status/review files. UNIX lands only after
all three provider closeouts; it owns final executor distribution and prefix integration. The
three final native evidence tasks then independently bind the same published UNIX checkpoint, and
CLOSEOUT only ingests those receipts and records terminal R3 status. No evidence task changes the
repository, and no unpushed local commit is eligible native evidence.

This order also closes executor delivery. Each source file under `src/bin/substrate-lifecycle-*`
is a Cargo auto-discovered binary; provider tests build their exact binary directly and install an
exact retained copy before its first action. UNIX adds the five lifecycle names to the existing
cargo-dist binary list and owns the Unix dev/release staging integration; WIN owns its PowerShell
dev build/staging integration. Release archive tests require all five names, and every
wrong-platform binary exits before decoding a request or observing/mutating platform state.
**Source provenance:** extracted from [`../01-target-architecture.md#a11d-5r3-lifecycle-ownership-architecture`](../01-target-architecture.md#a11d-5r3-lifecycle-ownership-architecture), baseline lines 1294–1662
**Baseline span SHA-256:** `2bb3d2e346268734f0cde83d94371802336019480885b701a27acafb88c30d0e`
