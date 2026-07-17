# Contracts and Gates

## Normative conventions

- All V1 records are durable, schema-versioned, and reject unknown fields and identity/binding
  ambiguity.
- IDs and refs are opaque. Model-visible callers may receive task/worker handles but may not
  construct internal participant, resume, lease, policy-ref, or UAA-session truth.
- `root_revision`, `authority_revision`, and intent/receipt/manifest revisions increase
  monotonically under the exact atomic persistence rules owned by their contracts.
- Every A1 timestamp is `TimestampV1`; every A1 structured commitment is over the named immutable
  `CanonicalJsonV1` input, never a presentation record.
- A field marked `ref` is a complete `AuthorityObjectRefV1`. Its parent record owns the expected
  kind, schema version, and commitment and verifies all three before use.

### A1 canonical encoding, path identity, supporting types, and persistence

#### `CanonicalJsonV1` and timestamp encoding

`CanonicalJsonV1` is a repository-owned encoder/decoder with this closed algorithm:

1. Input and output are UTF-8 only; invalid UTF-8 and unpaired surrogate values are rejected.
2. Object keys are sorted lexicographically by their exact UTF-8 byte sequence.
3. No insignificant whitespace is emitted.
4. Arrays preserve their exact input/value order.
5. Strings are not Unicode-normalized.
6. Non-control Unicode scalar values are emitted directly as UTF-8.
7. Double quote and backslash are escaped as `\"` and `\\`.
8. U+0000 through U+001F use lowercase `\u00xx`; alternate short escapes such as `\n`, `\t`, or
   `\b` are forbidden.
9. Boolean and null literals are exactly `true`, `false`, and `null`.
10. A1 canonical objects permit only explicitly typed signed or unsigned integers within the
    range of the field declared by the V1 schema. Integers use minimal base-ten form: no leading
    plus, no leading zero except `0`, and no negative zero.
11. Floating-point, decimal, non-finite, and arbitrary JSON numeric values are rejected unless a
    later schema introduces a distinct canonical decimal type.
12. Unknown and duplicate fields are rejected for every hashed V1 object. Every declared field is
    present; an absent Rust `Option` is encoded as JSON `null` rather than by omitting the field.
13. Struct field names are the exact snake_case names shown by this contract. A unit enum variant
    is `{"kind":"VariantName"}`; a variant with fields is
    `{"kind":"VariantName","value":{...}}`. Maps have string keys only. Raw bytes are never
    coerced into JSON strings.
14. Encoding is implemented by the repository-owned `CanonicalJsonV1` path, not delegated to
    `serde_json`, display/debug formatting, or any presentation serializer.

```rust
struct TimestampV1(String);
```

`TimestampV1` encodes as one JSON string containing a valid UTC RFC3339 timestamp with exactly
nine fractional digits and uppercase `Z`, for example
`2026-07-11T12:49:11.000000000Z`. Offsets, omitted or shorter fractions, lowercase `z`, leap-second
`:60`, and non-calendar values are rejected. This is the only A1 timestamp serialization.

#### Exact directory identity

```rust
struct CanonicalDirectoryV1 {
    physical_path: String,
    physical_identity: DirectoryPhysicalIdentityV1,
}

enum DirectoryPhysicalIdentityV1 {
    Linux { device_id: u64, inode: u64 },
    MacOs {
        volume_uuid: String,
        file_id: u64,
        case_sensitive: bool,
    },
}

struct WorkspaceBindingV1 {
    workspace_root: CanonicalDirectoryV1,
    authority_store_root: CanonicalDirectoryV1,
    authority_store_id: String,
}

struct WorldBindingV1 {
    world_id: String,
    world_generation: u64,
}
```

Both input roots must be absolute UTF-8 paths naming existing directories. Raw input containing a
`.` or `..` component, a relative path, an empty path, a nonexistent directory, a file, an ambient
CWD fallback, or a failed-home fallback is rejected before issuance. Workspace-root bootstrap may
resolve symlinks under its existing canonical-identity rules, then opens the resulting directory,
derives the physical path and identity from that trusted handle, and persists them once.
`authority_store_root` is stricter because it is `SUBSTRATE_HOME`: bootstrap opens every component
from the filesystem root with no-follow semantics and rejects any symlink component or final
symlink rather than canonicalizing through it. Authority-store descendant access then remains
directory-relative with no-follow semantics; it never reinterprets the persisted string through
ambient CWD. Workspace content traversal remains governed by its existing execution/policy
contracts; this private-home rule does not narrow or expand it.

#### `PrivateSubstrateHomeV1` acceptance rules

`SUBSTRATE_HOME` is one user's private configuration, policy, dependency-inventory, runtime, and
authority root. On supported Unix hosts, the creator first opens and retains the intended physical
parent and validates from that descriptor that it has the expected type and owner, is not writable
by another principal, has no disallowed ACL grant, and retains the expected physical identity. The
creator then invokes `mkdirat` relative to that trusted parent or joins `AlreadyExists`. Successful
`mkdirat` establishes only a candidate name; it does not establish the accepted child identity.
Portable Linux/macOS APIs do not atomically create a directory and return an inode-bound directory
handle.

The accepted `PrivateSubstrateHomeV1` physical identity begins at the first successful no-follow
directory open of the candidate relative to that same parent descriptor. Descriptor-based
validation then proves all of the following before any descendant bootstrap:

1. the selected absolute UTF-8 physical path is the path represented by the opened handle;
2. the handle names a directory owned by the intended per-user owner;
3. permission and special bits are exactly `0700`;
4. Linux access/default ACL observations satisfy the closed R1 result contract below: any
   observable `Present` access or default ACL is rejected, while `NoData` is accepted only under
   this exact descriptor/owner/type/`0700`/identity/replacement-safety proof and is never treated as
   proof of physical ACL-xattr absence;
5. no path component or final entry was followed through a symlink or substituted with another
   type, and the accepted platform physical identity is sourced only from the opened descriptor;
   and
6. the candidate resides on the expected filesystem where the trusted-filesystem contract requires
   that constraint.

##### Linux ACL observation and effective-authority boundary

Linux R1 uses one closed descriptor-bound observation for each POSIX access/default ACL request:

```text
enum LinuxAclObservationV1 {
    Present(bytes),
    NoData,
    Failed(error_class),
}
```

`NoData` is exactly an `ENODATA` retrieval result. It means only **the kernel returned no ACL
data**. It never means ACL absent, ACL-free, xattr physically absent, or proof that no ACL metadata
exists. Linux documents `ENODATA` for either a nonexistent named attribute or a process that lacks
access to it; ACL/xattr reads can also be mediated by Linux security hooks. Therefore physical
absence is not R1 authority. See [`getxattr(2)`](https://man7.org/linux/man-pages/man2/getxattr.2.html)
and Linux [`security/security.c`](https://github.com/torvalds/linux/blob/master/security/security.c).

For an ancestor access ACL, `Present(bytes)` is accepted only after strict Linux POSIX-ACL parsing:
the version, complete length, canonical tag order, unique named IDs, exactly one owner/group/other
base entry, required single mask, permission bits, and supported tags must all be valid. Effective
permissions for the group object and every named user/group entry are their raw permissions
intersected with `ACL_MASK`; `ACL_OTHER` remains the other class. Effective non-writing access such
as `--x` or `r-x` is supported. Any effective write for another principal fails closed, while a raw
write bit fully removed by the mask is not effective write authority. Multiple named entries are
evaluated independently under the same mask.

For supported Linux POSIX access ACLs, `ACL_MASK` corresponds to the file group-class mode bits,
and `ACL_OTHER` corresponds to the other-class mode bits. Consequently a POSIX ACL cannot grant
effective named-principal write while the descriptor's authoritative group-write bit remains clear;
the other class cannot grant write while the authoritative world-write bit remains clear. This is
an intentionally narrow Linux POSIX-ACL proof, supported by [`acl(5)`](https://man7.org/linux/man-pages/man5/acl.5.html)
and Linux [`fs/posix_acl.c`](https://github.com/torvalds/linux/blob/master/fs/posix_acl.c). It is not
generalized to NFSv4 or any unknown/non-POSIX ACL model.

Ancestor `NoData` is accepted only after the retained descriptor proves the expected path role,
expected owner, directory type, stable descriptor identity, component-by-component no-follow
traversal, no replacement, and authoritative mode bits with neither group nor world write.
`Failed(error_class)` covers every distinguishable non-`ENODATA` result, including `ENOTSUP`,
unreadable/unavailable state, malformed/changed data, unsupported version/model/tag, and retrieval
uncertainty, and always fails closed. Any result outside valid `Present` or qualified `NoData` fails
closed.

Default ACLs govern the initial access ACL of created children; they do not govern access to the
directory carrying the default ACL. Accordingly, ancestor `Present` default ACL data is rejected
before candidate creation and final-root `Present` default ACL data is rejected. Default `NoData`
has the same limited meaning above. Exact `0700` on the final root prevents other-principal
traversal, and the existing exact owner-only descendant contracts—directories `0700`, files `0600`,
or any stricter per-object rule—prevent an inherited Linux POSIX access ACL from conferring
effective other-principal authority. See the access/default distinction and creation algorithm in
[`acl(5)`](https://man7.org/linux/man-pages/man5/acl.5.html). No ACL cleanup or repair is added.

For the final authority root, any observable `Present` access ACL and any observable `Present`
default ACL are rejected even when their entries would be ineffective. Qualified `NoData` is
accepted only with exact owner, directory type, exact `0700` independent of umask, stable
descriptor identity, no-follow traversal, and replacement rejection. Existing invalid roots and
their metadata/contents are never mutated. If a future architecture requires proof that an ACL
xattr is physically absent, it requires a separately approved privileged platform-attestation
boundary; R1 does not design or implement one. The selected host location continues to follow the
existing explicit-home/XDG placement rules; the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
does not supply ACL authority.

Creation uses a directory-relative sequence that distinguishes successful creation from
`AlreadyExists`. Legitimate concurrent Substrate creators are supported: one may create while
another observes `AlreadyExists`; each then no-follow opens and exactly validates the candidate,
and only the identity captured from its accepted descriptor proceeds. A newly created root is
never accepted from its requested mode alone. The creator must `fsync` the new directory and its
affected parent where the platform supports the A1 durable-filesystem contract. Ambient umask may
remove bits during creation, but the creator may set the newly opened candidate to exact `0700`
before acceptance; it may never broaden or otherwise repair a root that existed before the
attempt.

After the first accepted open, every access remains descriptor-relative. At required publication
and acceptance boundaries, a no-follow lookup beneath the retained parent must still join the
child name to that accepted descriptor identity and descriptor validation must still prove type,
owner, exact mode, ACL, and required filesystem posture. Rename, replacement, owner/mode/ACL drift,
or validation uncertainty fails closed. No path-based descendant operation is permitted after
binding.

The trusted-parent owner, mode, and ACL rules prevent a different-principal attacker from replacing
the child. The Linux R1 decision supports strictly parsed masked non-writing ancestor access ACLs
under the effective-authority proof above; every ancestor default ACL, effective write, unsupported
model, malformed/unreadable/unavailable state, and observable final-root access/default ACL remains
rejected. A1 V1 does not establish a boundary against
malicious root or malicious code already executing under the same UID, so substitution by either
before the first child descriptor is acquired is outside the threat model. A1.1d-5 makes no atomic
create-and-bind claim and requires no privileged creation broker, protected staging root, or
root-owned publication protocol. Such a broker is an explicit non-goal unless a later separately
approved architecture expands the threat model.

An existing root, including a custom `SUBSTRATE_HOME`, is accepted only if it already passes the
same rules. Wrong type, symlink, owner mismatch, `0755`, `0750`, any group/world permission,
setuid/setgid/sticky or other special bits, foreign or inherited ACL grants, changed identity, or
an indeterminate check fails before any descendant write. The current implementation emits this
legacy shape:

```text
substrate: unsupported SUBSTRATE_HOME '<path>': expected a private directory owned by intended uid <uid> with exact mode 0700 and no effective other-principal authority; found <reason>. Existing roots are never repaired; reset it manually and retry.
```

R1 replaces that ambiguous attribution with a structured/error-display contract containing the
requested final home, exact offending path, object role (`ancestor` or `final-root`), ACL kind
(`access`, `default`, or `unavailable` where applicable), effective authority/reason, and whether
the rejected candidate was created by the current attempt. It must not print ACL principals,
unrelated path contents, credentials, or authority/session data. An ancestor failure may not be
reported as if the final root carried the ACL, and a newly created candidate may not receive the
pre-existing-root repair instruction.

The bounded Linux ACL diagnostic class is one of `PresentAcceptedNoEffectiveWrite` (ancestor access
ACL only), `PresentRejected`, `NoDataAcceptedUnderModeAuthority`, or `FailedOrUnavailable`.
Accepted observations may be retained as bounded diagnostic/proof facts but never disclose ACL
principal identifiers. `NoDataAcceptedUnderModeAuthority` must not use “ACL absent”, “ACL-free”, or
equivalent physical-absence wording. `PresentRejected` and `FailedOrUnavailable` name the actual
offending path and role, never attribute an ancestor failure to the requested final root, never
expose authority payloads or secrets, and never imply that an existing final root was repaired.

When privileged execution cannot resolve an intended non-root account, no intended UID exists to
render in that shape. It fails before inspecting or creating the root with this separate exact
diagnostic:

```text
substrate: unsupported SUBSTRATE_HOME '<path>': cannot determine the intended per-user owner while running with effective uid 0; found owner-ambiguous. Set the supported explicit user input or run as the intended user; no home was created or modified.
```

The generic diagnostic's `<reason>` token is one of `missing-parent`, `wrong-type`, `symlink`,
`wrong-owner`, `wrong-mode`, `foreign-acl`, `replaced`, or `validation-unavailable`. For an
ordinary non-root process, the intended owner is its effective UID. An installer or provisioner
running as root resolves the intended account from its explicit supported user input first
(`SUBSTRATE_INSTALL_PRIMARY_USER` where applicable), then its verified invoking-user signal such
as `SUDO_USER`; macOS/Lima guest provisioning uses the discovered Lima VM user. It resolves that
account through the platform account database and uses the resulting UID. Root execution with no
unambiguous intended non-root user fails as `owner-ambiguous`; it does not silently create a
root-owned user state home. Descriptor-bound owner and exact-mode initialization is permitted only
for the candidate created by the current attempt; a candidate joined through `AlreadyExists` is a
pre-existing root. No product path chmods, chowns, removes ACLs from, deletes contents from,
migrates, adopts, converts, or repairs a pre-existing root or falls back to a shared home. Failure
occurs before config/runtime scaffolding and before any authority marker, key, root, or legacy state
mutation. Repeating creation against an unchanged valid root is idempotent.

#### `InstallBootstrapContextV1` propagation rules

For the current combined installation/state-root architecture, one selected context is fixed at
the public install or uninstall entry point:

```rust
struct InstallBootstrapContextV1 {
    selected_host_prefix: AbsoluteHostPath,
    host_substrate_home: AbsoluteHostPath,
    host_substrate_root: AbsoluteHostPath,
    intended_host_principal: PlatformPrincipalV1,
}

struct PlatformBootstrapMappingV1 {
    host_context_commitment: Digest,
    platform_instance: PlatformInstanceIdentityV1,
    realized_substrate_home: AbsolutePlatformPath,
    realized_principal: PlatformPrincipalV1,
}
```

V1 requires `selected_host_prefix == host_substrate_home == host_substrate_root`; this does not
introduce a separate host install root. `PlatformPrincipalV1` is platform-scoped (for example,
Unix account plus UID or Windows account plus SID), not a Unix UID imposed on every platform.
Every host child/helper and sudo boundary that can bootstrap, deploy/remove shims, run a doctor, or
write/read generated configuration, environment, or install-state data receives the same host
context explicitly and may not fall back to ambient `$HOME`/`USERPROFILE`.

A platform backend may realize that committed host selection at a distinct native path and
principal—for example, the Lima guest user's private home. It must produce and consume one explicit
`PlatformBootstrapMappingV1` bound to the host-context commitment and exact platform instance;
host and guest paths or principals are not required to be equal. World dependency
add/remove/sync/rollback, platform provision/enable/disable, service install/restart, and runtime
provisioning consume either the unchanged host context or that explicit mapping. Neither a backend
nor `RuntimeFamilyRealizationAdapter` may select an unrelated home. Generated projections must
distinguish encoded context, self-derived install location, and prefix-relative placement; their
existence is not proof that earlier children received the context. Install and uninstall select
the same host authority home from their declared prefix or recorded matching install context; an
undocumented outer `SUBSTRATE_HOME` override is never required for normal operation.

Partial failure records which current-attempt artifacts were created or managed. Retry exact-joins
a valid selected home and managed artifacts; it never repairs an invalid pre-existing home.
Candidate rollback is descriptor-bound: it may remove only an empty candidate created by the
current attempt after a no-follow lookup beneath the retained parent still joins the exact opened
candidate identity. It is never recursive; replacement, nonempty state, ambiguity, or an
`AlreadyExists`/pre-existing candidate fails closed without removal. Recorded prefix-local and
system-level managed installer artifacts are a separate cleanup class and may be removed only by
exact manifest identity; broad name-prefix, wildcard, or ambient-home deletion is forbidden.
Rollback/uninstall must not delete an ambient default home when a different prefix was selected.
Service cleanup must return recorded binaries, helpers, units/drop-ins, sockets, and runtime
directories plus installer-created group/membership/ACL-bridge/linger state to the proven
pre-install state without removing pre-existing account state. A normal install, repeat install, accepted-home/later-
stage partial failure, synchronous current-attempt rejection with completed exact safe rollback,
uninstall, and uninstall-followed-by-reinstall must converge on the same selected context. An abrupt
interruption leaving an unaccepted candidate is revalidated on rerun and accepted only if already
valid. If it is invalid, `AlreadyExists`/unknown provenance remains fail-closed and the candidate is
never automatically repaired or removed. Arbitrary invalid-candidate crash-window convergence is
an unresolved contract gap unless R3 proves the state unreachable after R1; otherwise it requires a
separately approved provenance/publication mechanism. These rules are the minimum R2/R3 contract;
they do not prescribe shell implementation mechanics or authorize service mutation in A1.1d-5I.

Exact `0700` applies to the `SUBSTRATE_HOME` root. Existing stricter authority-store descendant
contracts remain unchanged: authority directories remain owner-only `0700` and authority files
remain owner-only `0600`. Multiple operating-system users directly sharing/traversing one home are
unsupported in A1 V1. A future separate installation root does not weaken this state-root contract;
`SUBSTRATE_ROOT` separation is not implemented here.

Acceptance changes no policy or world semantics. Effective-policy snapshots, world requests,
network routing, filesystem enforcement plans, allow/deny lists, isolation and host-visibility
flags, and policy hashes must remain identical for identical inputs. World members receive
config/policy/dependency/credential material through existing Substrate-owned projection or
mediation. If an unprivileged world process requires direct home traversal, A1.1d-5 stops as a
capability-boundary change instead of broadening permissions.

`authority_store_root` is also the one normalized bootstrap home. Host bootstrap resolves that
home once, before StateStore construction or config, policy, and inventory resolution, and passes
the same opened `CanonicalDirectoryV1` identity to all four consumers. The value persisted in
`WorkspaceBindingV1.authority_store_root` must equal that bootstrap-home identity byte-for-byte and
by physical identity, and must equal the decoded current strict root's `bootstrap_home`—whether
`StateRootV1` or `StateRootV2`—before any authority record, intent, or object is accepted. No issuer, helper, retry, restart, reconciliation, config resolver,
policy resolver, or inventory resolver may later reread `SUBSTRATE_HOME`, fall back to `$HOME`, or
resolve the home relative to ambient CWD. A different home, even if valid and content-equivalent,
is a binding mismatch.

For effective-policy resolution, host bootstrap derives the global policy source from that accepted
home without rereading ambient environment state. The shell passes an explicit broker-appropriate
source input, not a shell authority type, to one bounded broker entry point. The broker remains the
sole owner and applies the same canonical defaults, global patch parsing, workspace discovery and
patch parsing, patch precedence, explanation provenance, validation, and finalization used by its
ambient API. The explicit-source and ambient APIs must return semantically identical finalized
policy, source paths/layers, explanation output, derived legacy and V3 filesystem fields,
network/backend/dispatch values, and validation errors for semantically identical inputs.

Conflicting ambient and explicit homes must prove that only the accepted explicit source affects the
explicit result; global environment mutation may not simulate this API. Malformed or unsafe global
or workspace input fails consistently with the ambient canonical path. Config validation preserves
its existing conditional policy-parsing behavior and must not read or require policy when the
canonical ambient path would not. A missing descriptor entry never bypasses exact store, workspace,
session, reference, policy, or snapshot identity validation. Shell projection may record or validate
the canonical broker result, but it may not duplicate policy parsing, layering, explanation,
validation, or finalization. This A1.1e rule does not authorize dispatch-scoped narrowing, worker-cap
composition, immutable active-work acceptance, policy schema/precedence/default changes, or policy
enforcement changes.

Issuer, helper validation/application, retry, restart, and reconciliation all use this
same resolver and comparison rule. None may substitute lexical normalization, ambient CWD, a
different case rule, or path-only equality.

On Linux, comparison requires byte-for-byte equality of the selected UTF-8 physical path and exact
`(device_id, inode)`. On macOS, the path comes from the opened directory's physical path with the
filesystem's actual component spelling; no case-folding or Unicode normalization is performed.
The exact volume UUID, file ID, and case-sensitivity flag must also match. Thus case aliases on a
case-insensitive volume converge only through the same opened physical directory, case-distinct
entries on a case-sensitive volume remain distinct, and volume replacement or rebinding fails
closed. Windows A1 authority initialization and transitions are unsupported and fail closed until
a later contract defines and proves drive-relative versus absolute paths, drive/UNC identity,
separator normalization, reparse-point/junction resolution, case and volume identity, owner-only
ACLs, locking, and atomic durable replacement. No Windows path is silently mapped to the Unix or
macOS model.

#### Typed immutable object references

```rust
enum AuthorityObjectKindV1 {
    AgentDescriptor,
    RetainedWorker,
    ResumeHandle,
    Policy,
    HostAttachContract,
    TransitionTransportPayload,
    TransitionInput,
    LeaseToken,
    ApplicationResult,
    InputAcceptance,
    StartupOwnershipResult,
    ObligationSnapshot,
    PostTurnProtocolEvent,
    PostTurnCompletion,
    TerminalHandoff,
}

enum AuthorityObjectCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

struct AuthorityObjectRefV1 {
    ref_id: String,
    object_kind: AuthorityObjectKindV1,
    schema_version: u32,
    commitment: AuthorityObjectCommitmentV1,
}

type ResumeHandleRefV1 = AuthorityObjectRefV1;
type PolicyRefV1 = AuthorityObjectRefV1;
```

An A1 `ref_id` is exactly `ao_` plus 32 lowercase hexadecimal characters encoding 128 OS-CSPRNG
bits, generated and collision-checked under the root lock. It is never supplied as a path or
derived from caller text. The final location is
`objects/<kind-slug>/v<schema_version>/<ref_id>.obj`, with minimal-decimal schema version and this
closed kind-slug mapping:

```text
AgentDescriptor=agent-descriptor
RetainedWorker=retained-worker
ResumeHandle=resume-handle
Policy=policy
HostAttachContract=host-attach-contract
TransitionTransportPayload=transition-transport-payload
TransitionInput=transition-input
LeaseToken=lease-token
ApplicationResult=application-result
InputAcceptance=input-acceptance
StartupOwnershipResult=startup-ownership-result
ObligationSnapshot=obligation-snapshot
PostTurnProtocolEvent=post-turn-protocol-event
PostTurnCompletion=post-turn-completion
TerminalHandoff=terminal-handoff
```

Publication has no-replace semantics. Exact retry may join an existing final object only after
verifying its complete ref and bytes.

The parent record carries the full `AuthorityObjectRefV1`; `object_index` metadata is routing and
lifecycle data only. Wrong kind, wrong schema version, unexpected commitment variant/domain/key,
or commitment mismatch fails closed. Replacing object bytes together with object-index metadata
cannot change the parent-owned commitment. Sensitive/raw objects use `StoreHmacSha256`.
Non-sensitive, closed structured objects may use `CanonicalSha256`. All digests are exactly 64
lowercase hexadecimal characters. A canonical object's file bytes are exactly its named
`CanonicalJsonV1` hash-input bytes; a sensitive object's file bytes are exactly the HMAC input's
`raw` field. Raw provider credentials are prohibited from every A1 object.

#### Named immutable hash inputs

```rust
struct DurableSessionAuthorityHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    authority_revision: u64,
    origin: DurableSessionAuthorityOriginV1,
    authoritative_participant_lineage: Vec<String>,
    active_authoritative_participant_id: Option<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    host_attach_contract_ref: Option<AuthorityObjectRefV1>,
    retained_worker_refs: Vec<AuthorityObjectRefV1>,
    internal_resume_handle_refs: Vec<AuthorityObjectRefV1>,
    lifecycle_posture: HostSessionPostureV1,
    current_policy_ref: Option<AuthorityObjectRefV1>,
    current_policy_revision: Option<String>,
}

struct AuthoritativeLineageHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    participant_ids: Vec<String>,
}

enum AgentExecutionScopeV1 {
    Host,
    World,
}

enum RuntimeBackendKindV1 {
    Codex,
    ClaudeCode,
}

struct AgentDescriptorV1 {
    schema_version: u32,
    agent_id: String,
    backend_id: String,
    backend_kind: RuntimeBackendKindV1,
    protocol: String,
    execution_scope: AgentExecutionScopeV1,
    binary_path: String,
}

struct HostAttachCapabilitiesV1 {
    session_resume: bool,
    session_fork: bool,
    session_stop: bool,
    status_snapshot: bool,
    event_stream: bool,
}

enum HostAttachExecutionClientStartV1 {
    StartNow,
    Defer,
}

enum HostAttachModePreferenceV1 {
    ContinuityRequired,
    ContinuityPreferred,
    FreshAllowed,
}

struct HostAttachLaunchKnobsV1 {
    requested_execution_scope: AgentExecutionScopeV1,
    host_execution_client_start: HostAttachExecutionClientStartV1,
    attach_mode_preference: HostAttachModePreferenceV1,
}

struct HostAttachContractV1 {
    schema_version: u32,
    backend_id: String,
    execution_scope: AgentExecutionScopeV1,
    protocol: String,
    descriptor_ref: AuthorityObjectRefV1,
    capabilities: HostAttachCapabilitiesV1,
    attach_launch_knobs: HostAttachLaunchKnobsV1,
    policy_ref: AuthorityObjectRefV1,
    continuity_resume_handle_ref: Option<AuthorityObjectRefV1>,
}

struct HostSessionTransitionPayloadHashInputV1 {
    schema_version: u32,
    intent_id: String,
    issuer_request_id: String,
    mode: HostSessionTransitionModeV1,
    authority_precondition: HostSessionAuthorityPreconditionV1,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    caller: HostSessionTransitionCallerV1,
    source_authoritative_participant_id: Option<String>,
    target_authoritative_participant_id: String,
    target_participant_lease_token_ref: AuthorityObjectRefV1,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AuthorityObjectRefV1,
    host_attach_contract_ref: AuthorityObjectRefV1,
    resume_handle_ref: Option<AuthorityObjectRefV1>,
    transition_input_ref: Option<AuthorityObjectRefV1>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
    transport_payload_ref: AuthorityObjectRefV1,
    issued_at: TimestampV1,
    expires_at: TimestampV1,
}

struct TransitionTransportPayloadObjectV1 {
    schema_version: u32,
    intent_id: String,
    mode: HostSessionTransitionModeV1,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    caller: HostSessionTransitionCallerV1,
    source_authoritative_participant_id: Option<String>,
    target_authoritative_participant_id: String,
    target_participant_lease_token_ref: AuthorityObjectRefV1,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AuthorityObjectRefV1,
    host_attach_contract_ref: AuthorityObjectRefV1,
    resume_handle_ref: Option<AuthorityObjectRefV1>,
    transition_input_ref: Option<AuthorityObjectRefV1>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
}

struct AgentDescriptorHashInputV1 {
    schema_version: u32,
    descriptor: AgentDescriptorV1,
}

struct HostAttachContractHashInputV1 {
    schema_version: u32,
    contract: HostAttachContractV1,
}

struct ResumeHandleHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    backend_id: String,
    protocol: String,
    internal_uaa_session_id: String,
}

struct PolicyObjectHashInputV1 {
    schema_version: u32,
    policy_revision: String,
    canonical_policy_snapshot_sha256: String,
}

struct RetainedWorkerObjectHashInputV1 {
    schema_version: u32,
    orchestration_session_id: String,
    participant_id: String,
    world_binding: WorldBindingV1,
    descriptor_ref: AuthorityObjectRefV1,
    resume_handle_ref: AuthorityObjectRefV1,
    policy_ref: AuthorityObjectRefV1,
}

enum ApplicationResultPhaseV1 {
    InitialTransition {
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        post_turn_pending_run_id: Option<String>,
    },
    PostTurn {
        completion_ref: AuthorityObjectRefV1,
        obligation_snapshot_ref: Option<AuthorityObjectRefV1>,
        authority_revision_before: u64,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
    },
}

struct ApplicationResultHashInputV1 {
    schema_version: u32,
    intent_id: String,
    mode: HostSessionTransitionModeV1,
    run_id: String,
    phase: ApplicationResultPhaseV1,
    applied_at: TimestampV1,
}

struct InputAcceptanceHashInputV1 {
    schema_version: u32,
    intent_id: String,
    run_id: String,
    input_ref: AuthorityObjectRefV1,
    accepting_participant_id: String,
    accepted_at: TimestampV1,
}

enum HostStartupTerminalReasonV1 {
    RuntimeCreationRejected,
    StartupFailedBeforeOwnership,
}

enum HostStartupOwnershipProtocolEventV1 {
    OwnershipAccepted {
        ownership_acknowledgement_id: String,
    },
    RuntimeCreationRejected {
        rejection_id: String,
    },
    StartupFailedBeforeOwnership {
        failure_id: String,
    },
}

enum HostStartupOwnershipProtocolActorV1 {
    TargetAuthoritativeParticipant {
        participant_id: String,
    },
    LaunchApplicationClaimant {
        claim_id: String,
        claimant_attempt_id: String,
    },
}

struct HostStartupOwnershipEvidenceV1 {
    schema_version: u32,
    evidence_id: String,
    authority_store_id: String,
    orchestration_session_id: String,
    intent_id: String,
    claim_id: String,
    claimant_attempt_id: String,
    run_id: String,
    application_result_ref: AuthorityObjectRefV1,
    expected_authority_revision: u64,
    active_authoritative_participant_id: String,
    protocol_actor: HostStartupOwnershipProtocolActorV1,
    protocol_event: HostStartupOwnershipProtocolEventV1,
    observed_at: TimestampV1,
}

enum StartupOwnershipOutcomeV1 {
    Accepted,
    TerminalReconciled {
        reason: HostStartupTerminalReasonV1,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
    },
}

struct StartupOwnershipResultHashInputV1 {
    schema_version: u32,
    evidence: HostStartupOwnershipEvidenceV1,
    outcome: StartupOwnershipOutcomeV1,
    resolved_at: TimestampV1,
}

enum ObligationSnapshotRecordStateV1 {
    UnresolvedAttention,
}

struct ObligationSnapshotRecordHashInputV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    source_journal_event: SupervisorJournalEventRefV1,
    obligation_id: String,
    obligation_revision: u64,
    state: ObligationSnapshotRecordStateV1,
}

struct SupervisorJournalEventRefV1 {
    schema_version: u32,          // exactly 1
    journal_entry_id: String,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    stream_id: String,
    frame_sequence: u64,
    event_id: String,
    event_sequence: u64,
    transport_event_commitment: AuthorityObjectCommitmentV1,
}

struct UnresolvedAttentionObligationSnapshotEntryV1 {
    obligation_id: String,
    obligation_revision: u64,
    canonical_record_commitment: AuthorityObjectCommitmentV1,
}

enum ObligationAttentionDispositionV1 {
    NoUnresolvedAttention,
    HasUnresolvedAttention,
}

struct ObligationMaterializationCutV1 {
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    session_ledger_revision: u64,
    terminal_event_id: String,
    terminal_event_sequence: u64,
    materialized_through_event_sequence: u64,
}

struct ObligationSnapshotHashInputV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: HostTransitionWorkCorrelationV1,
    transition_intent_id: String,
    transition_run_id: String,
    authority_revision_observed: u64,
    materialization_cut: ObligationMaterializationCutV1,
    materialized_journal_events: Vec<SupervisorJournalEventRefV1>,
    attention_disposition: ObligationAttentionDispositionV1,
    unresolved_attention_obligations: Vec<UnresolvedAttentionObligationSnapshotEntryV1>,
    captured_at: TimestampV1,
}

enum ObligationLedgerSnapshotReadV1 {
    Pending {
        authority_store_id: String,
        orchestration_session_id: String,
        authoritative_participant_id: String,
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        stream_id: String,
        accepted_work_identity: AcceptedWorldWorkIdentityV1,
        host_transition_correlation: HostTransitionWorkCorrelationV1,
        transition_intent_id: String,
        transition_run_id: String,
        authority_revision_observed: u64,
        observed_session_ledger_revision: u64,
        required_terminal_event_id: String,
        required_terminal_event_sequence: u64,
    },
    Complete {
        snapshot: ObligationSnapshotHashInputV1,
    },
}

enum PostTurnCompletionOutcomeV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure,
}

enum HostPostTurnTerminalReasonV1 {
    ResumeRuntimeCreationRejected,
    TargetFailedBeforeInputAcceptance,
    TargetFailedAfterInputAcceptance,
}

enum HostPostTurnProtocolEventKindV1 {
    ResumableClean,
    TerminalClean,
    TerminalFailure {
        reason: HostPostTurnTerminalReasonV1,
    },
}

enum HostPostTurnProtocolActorV1 {
    TargetAuthoritativeParticipant {
        participant_id: String,
    },
    LaunchApplicationClaimant {
        claim_id: String,
        claimant_attempt_id: String,
    },
}

struct PostTurnProtocolEventHashInputV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    intent_id: String,
    claim_id: String,
    claimant_attempt_id: String,
    run_id: String,
    authority_revision_observed: u64,
    active_authoritative_participant_id: String,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: HostTransitionWorkCorrelationV1,
    protocol_actor: HostPostTurnProtocolActorV1,
    event_id: String,
    event_sequence: u64,
    kind: HostPostTurnProtocolEventKindV1,
    emitted_at: TimestampV1,
}

struct PostTurnCompletionHashInputV1 {
    schema_version: u32,
    intent_id: String,
    run_id: String,
    authority_revision_observed: u64,
    acceptance_record_id: String,
    acceptance_record_revision: u64,
    stream_id: String,
    accepted_work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: HostTransitionWorkCorrelationV1,
    terminal_event_id: String,
    terminal_event_sequence: u64,
    protocol_event_ref: AuthorityObjectRefV1,
    outcome: PostTurnCompletionOutcomeV1,
    completed_at: TimestampV1,
}

enum TerminalHandoffStateV1 {
    Applied,
    Rejected { reason: HostSessionTransitionTerminalRejectionV1 },
    Expired,
}

struct TerminalHandoffHashInputV1 {
    schema_version: u32, // exactly 1
    intent_id: String,
    run_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
    terminal_state: TerminalHandoffStateV1,
    application_result_ref: Option<AuthorityObjectRefV1>,
    input_acceptance_ref: Option<AuthorityObjectRefV1>,
    post_turn_completion_ref: Option<AuthorityObjectRefV1>,
    post_turn_application_result_ref: Option<AuthorityObjectRefV1>,
    recorded_at: TimestampV1,
}

struct TerminalHandoffHashInputV2 {
    schema_version: u32, // exactly 2
    intent_id: String,
    run_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
    terminal_state: TerminalHandoffStateV1,
    application_result_ref: Option<AuthorityObjectRefV1>,
    input_acceptance_ref: Option<AuthorityObjectRefV1>,
    startup_ownership_result_ref: Option<AuthorityObjectRefV1>,
    post_turn_completion_ref: Option<AuthorityObjectRefV1>,
    post_turn_application_result_ref: Option<AuthorityObjectRefV1>,
    recorded_at: TimestampV1,
}
```

`SupervisorJournalEventRefV1.transport_event_commitment` is the B2.1-owned `CanonicalSha256` over
the complete canonical runtime event bytes. The supervisor treats those bytes as opaque transport
truth: it verifies only B0 identity/order, B1 scope, canonical-byte equality, and journal replay.
That generic contract lets B2.1 close without a B3.1 semantic dependency. Once B3.1 adopts an
accepted retained stream, every post-acknowledgement `ExecuteStreamFrame::Event` before the exact
terminal frame must decode exactly as the shared `WorldWorkerEventV1`; there is no producer-chosen
"C1-eligible" subset. B3.1 validates each event's source/target participants and backends, world,
accepted active run, thread, event class, attention bit, request/message causation, owner-supplied
host-transition correlation when present, payload, and emission time against the B0/B1 join. A
missing or invalid typed member makes the stream protocol-invalid and keeps the C1 cut
non-Complete; neither B3.1 nor C1 may drop it. C1 revalidates that each generic journal commitment
is the canonical commitment of its full envelope before classification, materialization, or
snapshot capture. The canonical obligation record commits this typed ref, so a Complete snapshot
cannot drop or substitute any retained event field.

`ObligationSnapshotHashInputV1.materialized_journal_events` contains every post-acknowledgement
retained `Event` journal ref in the exact acceptance/stream scope at or below the
materialized-through watermark, sorted by `event_sequence`, with unique journal entry, frame, and
event identities. It may be empty only when that closed retained scope contains no `Event` frame.
The vector must be an exhaustive join against B2.1's journal, not a set chosen by event class or
attention value. Every unresolved obligation entry's canonical source ref names an exact member of
this vector. The vector is part of the snapshot hash, so `NoUnresolvedAttention` still commits the
complete classified retained-event set and cannot prove completeness from a terminal watermark
alone.

For a transition-scoped obligation read, `accepted_work_identity` and
`host_transition_correlation` are copied from the exact B1 acceptance record, and
`acceptance_record_id`/revision plus `stream_id` match the B2.1 journal and materialization cut.
The top-level `transition_intent_id`, `transition_run_id`, and
`authority_revision_observed` exactly equal the same fields in `host_transition_correlation`; the
accepted task/active-run identity remains distinct. Missing or mismatched correlation cannot return
Complete and is never repaired from request IDs, active-run IDs, payloads, foreground state, or
A1.2 guesses.

Terminal handoff bytes use strict schema-version dispatch. Existing `schema_version = 1` bytes
decode only as `TerminalHandoffHashInputV1` and remain byte-for-byte unchanged; a missing V2 field
is never defaulted into V1. `schema_version = 2` bytes decode only as
`TerminalHandoffHashInputV2`. A V2 handoff is required exactly where the closed matrix carries a
startup-ownership result; all other rows remain V1. `Some` refs must have the exact named kind and
equal the corresponding intent substate/journal ref, and every unlisted ref is `None`:

| Terminal state and mode | Handoff schema | Initial application | Input acceptance | Startup ownership | Post-turn completion/result |
|---|---:|---|---|---|---|
| `Rejected` or `Expired`, every mode | V1 | none | none | none | none |
| `Applied Start` or `Applied Attach` | V2 | exact initial `ApplicationResult` | exact `InputAcceptance` iff a Start input is Accepted; none only for input-free mode or a terminally reconciled Start whose input is exactly `TerminalWithoutAcceptance` | exact `StartupOwnershipResult` in Accepted or TerminalReconciled | none |
| `Applied ResumeOneTurn`, every post-turn Applied outcome | V1 | exact initial `ApplicationResult` | exact `InputAcceptance` for Accepted input; none only for `TerminalFailure` with exact `TerminalWithoutAcceptance` | none | exact `PostTurnCompletion` and post-turn `ApplicationResult` pair; `ResumableClean` additionally carries the exact Complete obligation snapshot in the post-turn application, while `TerminalClean`/`TerminalFailure` carry none |

An applied Resume cannot bypass post-turn application. `ResumableClean` and `TerminalClean`
require exact Accepted input. `TerminalFailure` requires either Accepted input or, only for an
exact pre-acceptance terminal event, atomically changes Pending input to
`TerminalWithoutAcceptance` using that same terminal handoff. A terminally reconciled applied
Start likewise atomically terminalizes any Pending input; an ownership-Accepted Start with Pending
input remains nonterminal and retained. `AwaitingObligationCut` is nonterminal and cannot release
transport or have a terminal handoff. A startup result protocol-event/outcome mismatch, a
half-present post-turn pair, a forbidden cross-mode ref, or any ref not equal to parent/journal
truth is corruption and fails closed.

`CanonicalSha256` is exactly lowercase-hex
`SHA-256(CanonicalJsonV1(named_hash_input))`. The authority-record commitment uses
`DurableSessionAuthorityHashInputV1`; lineage uses `AuthoritativeLineageHashInputV1`; immutable
intent payload uses `HostSessionTransitionPayloadHashInputV1`; descriptor and attach-contract
objects use their named wrappers; resume handles, policy objects, and retained-worker reference
objects use `ResumeHandleHashInputV1`, `PolicyObjectHashInputV1`, and
`RetainedWorkerObjectHashInputV1`; both initial and post-turn application objects use
`ApplicationResultHashInputV1`; input acceptance, startup-ownership resolution, obligation
snapshot records/snapshots, post-turn protocol events/completions, and terminal-handoff objects use their corresponding named
wrappers. `HostStartupOwnershipEvidenceV1` is embedded in and committed by the startup result;
each obligation snapshot entry carries the exact `CanonicalSha256` commitment of its
`ObligationSnapshotRecordHashInputV1`.
`updated_at`, intent/claim/root revisions not explicitly present in a
wrapper, mutable state, presentation-only fields, plan/socket paths, and adjacent object metadata
are not accidentally swept into a hash. Authority-record, lineage, payload, descriptor,
attach-contract, resume-handle, policy, retained-worker, application, input acceptance,
startup-ownership resolution, obligation snapshot, post-turn protocol event/completion, and terminal-handoff commitments
must use `CanonicalSha256`; a `StoreHmacSha256` variant in those
fields fails closed.

`AgentDescriptorV1` and `HostAttachContractV1` are the closed typed projections above, not
arbitrary `serde_json::Value`. The policy object commits the exact already-resolved policy revision
and canonical snapshot digest; A1 preserves those accepted values without rereading or
reinterpreting inventory or policy. The resume handle binds the exact internal UAA continuity
identity to session, participant, backend, and protocol. Unknown projection fields fail closed.
Descriptor IDs/protocol/binary path and resume-handle identities are non-empty; the policy snapshot
digest is exactly 64 lowercase hexadecimal characters. A HostAttachContract descriptor ref must be
kind `AgentDescriptor`, its policy ref kind `Policy`, and its optional continuity ref kind
`ResumeHandle`; their backend, protocol, scope, session, and participant identities must agree with
the parent contract/intent. A retained-worker object must use kinds `AgentDescriptor`,
`ResumeHandle`, and `Policy` for its three refs and match the same session/world identity. Any
disagreement or cross-kind substitution fails before use.

Every `AuthorityObjectKindV1` has exactly one V1 bytes and commitment rule:

| Object kind | `schema_version` | Exact file bytes | Required commitment |
|---|---:|---|---|
| `AgentDescriptor` | 1 | `CanonicalJsonV1(AgentDescriptorHashInputV1)` | `CanonicalSha256` |
| `RetainedWorker` | 1 | `CanonicalJsonV1(RetainedWorkerObjectHashInputV1)` | `CanonicalSha256` |
| `ResumeHandle` | 1 | `CanonicalJsonV1(ResumeHandleHashInputV1)` | `CanonicalSha256` |
| `Policy` | 1 | `CanonicalJsonV1(PolicyObjectHashInputV1)` | `CanonicalSha256` |
| `HostAttachContract` | 1 | `CanonicalJsonV1(HostAttachContractHashInputV1)` | `CanonicalSha256` |
| `TransitionTransportPayload` | 1 | `CanonicalJsonV1(TransitionTransportPayloadObjectV1)`; those canonical bytes are the HMAC `raw` member | `StoreHmacSha256` with `substrate.a1.raw-transport-payload.v1` |
| `TransitionInput` | 1 | the exact input byte string, with no newline, JSON, or text coercion | `StoreHmacSha256` with `substrate.a1.transition-input.v1` |
| `LeaseToken` | 1 | the exact UTF-8 lease-token bytes, with no normalization | `StoreHmacSha256` with `substrate.a1.participant-lease-token.v1` |
| `ApplicationResult` | 1 | `CanonicalJsonV1(ApplicationResultHashInputV1)` | `CanonicalSha256` |
| `InputAcceptance` | 1 | `CanonicalJsonV1(InputAcceptanceHashInputV1)` | `CanonicalSha256` |
| `StartupOwnershipResult` | 1 | `CanonicalJsonV1(StartupOwnershipResultHashInputV1)` | `CanonicalSha256` |
| `ObligationSnapshot` | 1 | `CanonicalJsonV1(ObligationSnapshotHashInputV1)` | `CanonicalSha256` |
| `PostTurnProtocolEvent` | 1 | `CanonicalJsonV1(PostTurnProtocolEventHashInputV1)` | `CanonicalSha256` |
| `PostTurnCompletion` | 1 | `CanonicalJsonV1(PostTurnCompletionHashInputV1)` | `CanonicalSha256` |
| `TerminalHandoff` | 1 | `CanonicalJsonV1(TerminalHandoffHashInputV1)` | `CanonicalSha256` |
| `TerminalHandoff` | 2 | `CanonicalJsonV1(TerminalHandoffHashInputV2)` | `CanonicalSha256` |

`TransitionTransportPayloadObjectV1` is the closed replay/reprojection schema; it contains typed
refs to raw lease/input objects rather than copying their bytes. For all three sensitive kinds, the
named HMAC input below is the exact commitment wrapper and its `raw` member is exactly the file
bytes. Transition input is a length-preserved arbitrary byte string; lease token is a non-empty
UTF-8 byte string without normalization; neither is parsed as JSON. Kind, schema version, typed
path, parent-held ref, and the required commitment variant/domain are all checked before bytes are
read or used. A ref cannot be accepted under another kind even when file bytes happen to match, and
no adjacent index row can override the parent's kind, schema, or commitment.

Golden fixtures must commit exact `CanonicalJsonV1` bytes and SHA-256 digest, plus fixed test-key
HMAC values where sensitive refs occur. Start, Attach, and `ResumeOneTurn`-with-input fixtures use
`HostSessionTransitionPayloadHashInputV1`; authority, lineage, attach-contract, application-result,
input-acceptance, startup-ownership-result, obligation-snapshot-record, obligation-snapshot,
post-turn-protocol-event, post-turn-completion, terminal-handoff, descriptor, resume-handle, policy, and retained-worker
fixtures use their named wrappers; terminal-handoff fixtures cover strict V1 and V2 bytes under
their respective named wrappers, and the typed-ref fixture uses `AuthorityObjectRefV1` itself. The
fixture set covers every allowed `HostStartupOwnershipProtocolEventV1` actor/variant pairing and
terminal reason plus rejected cross-actor pairings, both
`ObligationAttentionDispositionV1` variants, every `HostPostTurnProtocolEventKindV1` variant and
terminal reason/actor combination, a wrong-claimant-attempt rejection, and every terminal-handoff
matrix row. Raw input,
lease-token, and transport fixtures commit both exact bytes and fixed-key HMAC outputs. Each sensitive-domain
fixture also rejects `run_present=0x00`, an empty run, and a different run from the parent intent.
Issuer, helper,
restart/reconciliation, and release tests consume the same fixture files rather than regenerating
expected bytes through the code under test. Test HMAC keys are fixture-only and can never be
selected by a production store.

#### Keyed commitments for sensitive payloads

```rust
struct AuthorityStoreCommitmentKeyV1 {
    schema_version: u32,
    authority_store_id: String,
    key_id: String,
    algorithm: AuthorityStoreCommitmentAlgorithmV1,
    created_at: TimestampV1,
    state: AuthorityStoreCommitmentKeyStateV1,
}

struct AuthorityStoreInitializationV1 {
    schema_version: u32,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    initial_key_id: String,
    created_at: TimestampV1,
}

enum AuthorityStoreCommitmentAlgorithmV1 {
    HmacSha256,
}

enum AuthorityStoreCommitmentKeyStateV1 {
    Active,
    VerificationOnly,
    Retired,
}
```

Store initialization generates a 256-bit HMAC key from the OS CSPRNG while holding the root lock.
`key_id` is exactly `ak_` plus 32 lowercase hexadecimal characters encoding a separate 128-bit
OS-CSPRNG identifier.
The lifecycle record is stored only in the current strict root's `commitment_key_registry`, for
either `StateRootV1` or `StateRootV2`; it contains no key bytes. Secret key bytes live outside the root at `authority-v1/keys/<key_id>.key`. Each key file is
an immutable binary `AuthorityStoreCommitmentKeyFileV1` envelope with this exact byte encoding:

```text
"substrate.a1.commitment-key-file.v1\0"
 || u32be(1)
 || len64(authority_store_id) || authority_store_id
 || len64(key_id) || key_id
 || 0x01
 || len64(created_at) || created_at
 || u32be(32) || 32 secret key bytes
```

Lengths are unsigned 64-bit big-endian; strings are exact UTF-8; `created_at` is the exact
`TimestampV1` string; `0x01` is the sole `HmacSha256` algorithm tag; the secret-key member is exactly
32 bytes; and the complete envelope has no trailing bytes. The file is written through
`tmp/key--<key_id>--<nonce>.tmp`, `fsync`ed, atomically renamed with no-replace semantics, and
followed by `fsync` of `keys/`. The opened complete envelope must be regular, owner-only, and
no-follow. With a valid root its filename, store ID, key ID, timestamp, and algorithm must match the
registry entry exactly. Lifecycle state exists only in the root registry: `Active` may create and
verify commitments, `VerificationOnly` may verify only, and `Retired` may do neither. During
`InitializationPending`, where no root/registry exists yet, it instead must match the strictly
decoded init marker's `authority_store_id`, `initial_key_id`, and `created_at`, use algorithm tag
`0x01`/`HmacSha256`, and is provisionally the sole `Active` key for the root that marker may create.
No other metadata source is accepted. Keys and raw values are never logged, traced, exported,
placed in helper plans, or included in diagnostics. Only local authority processes already
authorized for the store may open the key through the trusted store handle.

Exactly one registry entry is `Active`, and its key ID equals the decoded current strict root's
`active_commitment_key_id`. New sensitive records use only that key. Rotation is one
serialized protocol under the root lock:

1. Generate a new key ID, metadata record, and 32-byte key from the OS CSPRNG.
2. Publish and `fsync` the immutable key file and `keys/` directory.
3. Revalidate the still-locked root and commit one new root revision that inserts the new entry as
   `Active`, changes the former active entry to `VerificationOnly`, and changes
   `active_commitment_key_id` to the new key.
4. `fsync` the root parent before reporting rotation success.

A crash before step 3 leaves an unregistered key-file orphan. After a complete valid root and all
reachable refs are verified under the lock, an unregistered key ID has no commitment authority and
is deleted with a following `fsync(keys/)`; a ref naming an unregistered key is instead corruption.
A crash after step 3 is a committed rotation: both key files must exist, the new key is active, and
the previous key is verification-only. There is no mutable state field in a key file and therefore
no file/root disagreement to arbitrate.

The prior key remains `VerificationOnly` while any reachable live, terminal, retained, tombstoned,
journal, or application record references it. Retirement first performs a complete locked
reachability scan proving zero references, then commits a root revision changing that registry
entry to permanent `Retired`. Only after that root and parent directory are
`fsync`ed may the key file be deleted and `keys/` be `fsync`ed. A crash with `Retired` metadata and
the file still present resumes deletion; a retired file is never used. A missing active or
verification-only file, or an unreadable, wrong-store, wrong-key-ID, wrong-algorithm, malformed, or
mismatched envelope, is store corruption and fails closed. Retired registry entries are never
deleted or reused.

The required domains are exactly:

```text
substrate.a1.transition-input.v1
substrate.a1.participant-lease-token.v1
substrate.a1.raw-transport-payload.v1
```

For these three intent-bound domains the HMAC input is this byte sequence, where `len64` is an
unsigned 64-bit big-endian byte length, all strings are their exact UTF-8 bytes, and `raw` is not
JSON encoded:

```text
"substrate.a1.hmac-input.v1\0"
 || len64(domain) || domain
 || len64(authority_store_id) || authority_store_id
 || len64(intent_id) || intent_id
 || run_present
 || if run_present == 0x01 { len64(run_id) || run_id }
 || len64(raw) || raw
```

Transition input/prompt bytes, participant lease-token bytes, and raw transport-payload bytes use
their respective domain. All three V1 object kinds are run-bound: `run_present` is exactly `0x01`
and `run_id` is the non-empty exact run ID committed by the parent intent. `0x00`, an absent/empty
run ID, or a different run ID is invalid for every V1 domain and fails before object acceptance.
Delimiter concatenation is forbidden. The `StoreHmacSha256.digest_hex` is lowercase-hex
`HMAC-SHA-256(key, bytes_above)`. Sensitive bytes and these reusable commitments are authority
store data only; failed verification reports a redacted typed error without logging either.

#### Strict `StateRootV1`/`StateRootV2` and the session namespace

```rust
struct StateRootV1 {
    schema_version: u32,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    root_revision: u64,
    active_commitment_key_id: String,
    commitment_key_registry: BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1,
    session_namespace_map: BTreeMap<String, SessionNamespaceRecordV1>,
    transition_intent_map: BTreeMap<String, HostSessionTransitionIntentV1>,
    issuer_request_index: BTreeMap<String, IssuerRequestIndexEntryV1>,
    application_journal: BTreeMap<String, HostSessionTransitionApplicationJournalV1>,
    object_index: BTreeMap<String, AuthorityObjectIndexEntryV1>,
}

struct StateRootV2 {
    schema_version: u32, // exactly 2
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    root_revision: u64,
    active_commitment_key_id: String,
    commitment_key_registry: BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1,
    session_namespace_map: BTreeMap<String, SessionNamespaceRecordV1>,
    transition_intent_map: BTreeMap<String, HostSessionTransitionIntentV2>,
    issuer_request_index: BTreeMap<String, IssuerRequestIndexEntryV1>,
    application_journal: BTreeMap<String, HostSessionTransitionApplicationJournalV2>,
    retained_worker_registration_request_index: BTreeMap<String, RetainedWorkerAuthorityRegistrationRequestV1>,
    retained_worker_registration_journal: BTreeMap<String, RetainedWorkerAuthorityRegistrationV1>,
    object_index: BTreeMap<String, AuthorityObjectIndexEntryV1>,
}

struct GreenfieldNamespaceCertificateV1 {
    schema_version: u32,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
    certified_at: TimestampV1,
}

enum SessionNamespaceRecordV1 {
    Authority(DurableSessionAuthorityV1),
    StartReservation(SessionIdReservationV1),
    StartTombstone(SessionIdTombstoneV1),
}

struct SessionIdReservationV1 {
    schema_version: u32,
    orchestration_session_id: String,
    intent_id: String,
    issuer_request_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
    reserved_at: TimestampV1,
}

enum StartTombstoneStateV1 {
    Rejected { reason: HostSessionTransitionTerminalRejectionV1 },
    Expired,
}

struct SessionIdTombstoneV1 {
    schema_version: u32,
    orchestration_session_id: String,
    intent_id: String,
    issuer_request_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
    terminal_state: StartTombstoneStateV1,
    terminal_handoff_ref: AuthorityObjectRefV1,
    tombstoned_at: TimestampV1,
}

struct IssuerRequestIndexEntryV1 {
    schema_version: u32,
    issuer_request_id: String,
    orchestration_session_id: String,
    intent_id: String,
    payload_commitment: AuthorityObjectCommitmentV1,
}

struct InitialTransitionApplicationJournalV1 {
    authority_revision_before: Option<u64>,
    authority_revision_after: u64,
    authority_record_commitment: AuthorityObjectCommitmentV1,
    application_result_ref: AuthorityObjectRefV1,
    applied_at: TimestampV1,
}

struct PostTurnApplicationJournalV1 {
    completion_ref: AuthorityObjectRefV1,
    authority_revision_before: u64,
    authority_revision_after: u64,
    authority_record_commitment: AuthorityObjectCommitmentV1,
    application_result_ref: AuthorityObjectRefV1,
    applied_at: TimestampV1,
}

// A1.2b-only after C1; strict V1 is unchanged.
struct PostTurnApplicationJournalV2 {
    completion_ref: AuthorityObjectRefV1,
    obligation_snapshot_ref: Option<AuthorityObjectRefV1>,
    authority_revision_before: u64,
    authority_revision_after: u64,
    authority_record_commitment: AuthorityObjectCommitmentV1,
    application_result_ref: AuthorityObjectRefV1,
    applied_at: TimestampV1,
}

struct HostSessionTransitionApplicationJournalV1 {
    schema_version: u32,
    intent_id: String,
    initial_application: InitialTransitionApplicationJournalV1,
    post_turn_application: Option<PostTurnApplicationJournalV1>,
}

struct HostSessionTransitionApplicationJournalV2 {
    schema_version: u32, // exactly 2
    intent_id: String,
    initial_application: InitialTransitionApplicationJournalV1,
    startup_terminal_application: Option<StartupOwnershipTerminalApplicationJournalV1>,
    post_turn_application: Option<PostTurnApplicationJournalV1>,
}

struct StartupOwnershipTerminalApplicationJournalV1 {
    schema_version: u32,
    startup_ownership_result_ref: AuthorityObjectRefV1,
    evidence_id: String,
    authority_revision_before: u64,
    authority_record_commitment_before: AuthorityObjectCommitmentV1,
    authority_revision_after: u64,
    resulting_posture: HostSessionPostureV1,
    authority_record_commitment_after: AuthorityObjectCommitmentV1,
    applied_at: TimestampV1,
}

enum AuthorityObjectStorageStateV1 {
    Present,
    ReleaseEligible { terminal_handoff_ref: AuthorityObjectRefV1 },
    Released {
        terminal_handoff_ref: AuthorityObjectRefV1,
        released_at: TimestampV1,
    },
}

struct AuthorityObjectIndexEntryV1 {
    schema_version: u32,
    ref_id: String,
    object_kind: AuthorityObjectKindV1,
    object_schema_version: u32,
    byte_length: u64,
    storage_state: AuthorityObjectStorageStateV1,
}
```

Map ownership and keys are exact: `session_namespace_map` is keyed by
`orchestration_session_id`; `transition_intent_map` and `application_journal` by `intent_id`;
`issuer_request_index` by store-global `issuer_request_id`; `commitment_key_registry` by `key_id`;
and `object_index` by `ref_id`. V2 additionally keys
`retained_worker_registration_request_index` by store-global `issuer_request_id` and
`retained_worker_registration_journal` by store-global `registration_id`. Each value repeats and
must match its map key. A fresh A1.1 root is strict V1 and initializes its five semantic maps empty.
V1 never accepts, defaults, or ignores V2 fields. A1.2a must perform the closed V1-to-V2 upgrade
below before Start issuance. The resulting strict V2 root initializes both retained-registration
maps empty. Only R0 may add entries. Request fields are immutable and request state has the sole
`Reserved -> Applied` transition; journal entries are immutable; neither map permits deletion. An
issuer request ID or registration ID is semantic occupancy: exact retry joins only the same
complete record and any conflicting reuse fails before mutation. Initial V2 Start application
creates a V2 application journal with `startup_terminal_application = None` and
`post_turn_application = None`; only an exact terminal startup reconciliation may fill the startup
field once, while Accepted startup leaves it `None`. Later V3 Attach/Resume uses the separately
versioned application-journal member. Post-turn and startup-terminal phases occupy
distinct fields and cannot substitute for each other.
Every non-released
object-index entry is reachable from an exact parent record in the same root; an index row alone
cannot grant semantic authority. Only a
`TransitionTransportPayload` entry may be `ReleaseEligible` or `Released`; every other committed
object kind remains immutable `Present`. A session namespace key is never deleted from either
supported strict root after reservation, authority creation, or tombstoning; lifecycle changes
replace only the value variants explicitly authorized above.

The greenfield certificate is not a migration barrier. Its `schema_version` is exactly 1, its store
ID and bootstrap-home identity exactly equal the enclosing root, and its timestamp is the exact
timestamp fixed by `AuthorityStoreInitializationV1`. It can be created only by the `FreshAbsent`
initialization protocol below and is immutable for the life of the store. A missing, changed,
copied, synthesized, or independently created certificate is root corruption.

#### A1.2a strict greenfield root upgrade

A1.2a owns one version-correct schema evolution from the already-landed strict `StateRootV1` to
strict `StateRootV2`; this is not legacy authority conversion. The decoder first reads only the
closed schema-version discriminator and then decodes exactly the selected closed struct. It never
uses `serde(default)`, an optional V2 field on V1, unknown-field tolerance, or retry-by-parsing as a
different version. Unknown versions and malformed/mixed-version bytes fail closed.

The upgrade is allowed only while holding the retained opened physical-root transaction and only
when the V1 root, key registry/files, immutable greenfield certificate, bootstrap-home/store
identity, and exact root revision all validate; `session_namespace_map`, `transition_intent_map`,
`issuer_request_index`, `application_journal`, and `object_index` are all empty; both pre-A1 legacy
collections are safely absent/empty; and no object or semantic orphan exists. Any V1 semantic
entry, pre-A1 artifact, unsafe/unreadable state, or identity/revision mismatch returns
`UnsupportedNonGreenfieldRootV1` with zero mutation. No authority, reservation, intent, object, or
application result is created by the upgrade.

The one root transaction copies the store/home/certificate/key fields byte-for-byte, increments
`root_revision` once, sets `schema_version = 2`, keeps the existing five semantic maps empty, and
adds only empty `retained_worker_registration_request_index` and
`retained_worker_registration_journal` maps; V2 intent/application entries use
`HostSessionTransitionIntentV2` and
`HostSessionTransitionApplicationJournalV2`. The root temp file, file/directory `fsync`, rename,
lock, identity revalidation, and crash windows are exactly the existing root-publication protocol.
Crash before publication leaves the exact V1 root; crash after publication yields the exact V2
root. Exact retry joins only that byte-equivalent V2 conversion. Once V2 is published, no writer
may emit V1 again. A1.1e read-only support for an existing strict V1 root remains unchanged, while
A1.2a Start and all R0 mutations require strict V2. A1.2b mutations require the later strict V3.

Shared key lifecycle and object-reachability code must dispatch on that same closed root-version
type. Rotation or retirement preserves the decoded version and identical V1 behavior; it cannot
down-convert V2 or reinterpret one application-journal shape as the other. V2 reachability includes
every initial-application ref, every present startup-terminal or post-turn application ref, and,
once R0 is allowed, every Applied retained-registration descriptor/resume/worker/policy ref.
Reserved request IDs and commitments are plans, not object refs or reachability roots. A referenced
object missing from this exhaustive version-specific traversal is corruption; an object is never
made reachable merely by an object-index entry.

#### External-object write protocol and filesystem safety

The fixed layout below is entirely beneath the physically bound `authority_store_root`:

```text
authority-v1/state-root-v1.json
authority-v1/init-v1.json
authority-v1/lock/root.lock
authority-v1/tmp/
authority-v1/objects/<closed-kind>/v<schema_version>/<ref_id>.obj
authority-v1/keys/<key_id>.key
```

Every temp filename has one closed operation-bound grammar, where `nonce` is 32 lowercase
hexadecimal characters encoding 128 OS-CSPRNG bits:

```text
init--<authority_store_id>--<nonce>.tmp
key--<key_id>--<nonce>.tmp
object--<ref_id>--<nonce>.tmp
root--r<minimal-decimal-new-root_revision>--<nonce>.tmp
```

Temps exist only in `authority-v1/tmp/`, are regular owner-only `0600` files opened no-follow with
exclusive create, and never carry semantic authority. Before bootstrap classification and before
any transaction/reconciliation, the root lock holder enumerates `tmp/` and applies this exact rule:

1. A recognized safe temp is never promoted or treated as proof, whether empty, partial, complete,
   or `fsync`ed. Delete it directory-relatively and `fsync(tmp/)`.
2. If a valid committed root requires a key/object whose final file is absent, the store is corrupt;
   a matching temp cannot repair it. If a valid final file exists, its temp is still deleted.
3. With a valid pending init marker, deletion is followed by normal exact marker/key/root recovery.
   With no root or marker, deletion completes before evaluating `FreshAbsent`.
4. An unrecognized name, invalid embedded ID/revision/nonce, symlink, non-regular entry, wrong owner
   or mode, directory-enumeration error, or deletion/`fsync` failure is `CorruptOrUnsupported`.

Thus a crash before temp `fsync`, after temp `fsync`, or before/after rename has one result: an
unrenamed temp is safely removed; a completed rename is evaluated only at its final typed location;
and neither state can fabricate root, key, object, marker, or application success. Key retirement
uses final-file deletion and no temp.

Initialization creates/opens `authority-v1`, `lock`, `tmp`, `objects`, and `keys` relative to the
trusted root handle with no-follow/exclusive-create checks, then serializes competing initializers
on the exact `authority-v1/lock/root.lock` file. The lock file contains no PID authority and is
never broken by liveness heuristics. Linux and macOS use a whole-file exclusive `flock(LOCK_EX)`
held on the no-follow-opened lock-file descriptor for the entire transaction; a filesystem whose
`flock` is not cross-process and crash-safe is unsupported. Pre-lock directory creation is
idempotent only when post-open ownership, type, and mode validation succeeds; no root/key/object
publication occurs before the lock is held. The initializer `fsync`s every newly created directory,
the lock file, and their affected parent directories before classifying or publishing anything.

##### `LegacyStateStoreTransactionV1` retained-root contract

Every pre-A1 StateStore transaction that reads or mutates either guarded session/participant
authority collection is admitted through one bounded `LegacyStateStoreTransactionV1`-equivalent
capability. Admission retains all of the following as authority-bearing resources:

- the opened `TrustedAuthorityRoot` handle selected from the normalized bootstrap home;
- its exact `CanonicalDirectoryV1` physical path and platform physical identity;
- no-follow-opened collection and descendant directory handles, or the deepest safe opened parent
  plus exact missing suffix for a permitted create;
- exclusive ownership of the cross-process `authority-v1/lock/root.lock` guard; and
- the activation/classification observation made under that same root and lock.

The capability exposes only directory-relative/no-follow operations rooted in those retained
handles: bounded reads, exclusive temp creation and writes, file `fsync`, no-replace publication or
atomic rename, removals, and affected-directory `fsync`. It does not expose descendant `PathBuf`s,
absolute-path reconstruction, ambient-CWD lookup, or any API that rereads `SUBSTRATE_HOME`, `$HOME`,
or another lexical bootstrap path after admission. Every read used to decide a mutation, every
flat/canonical snapshot or lease write, every removal/rename, and every compatibility/read-repair or
parent-session persistence step that mutates a guarded collection remains inside that one retained
transaction.

Transaction completion occurs only after the final changed file and every affected directory have
been `fsync`ed while the root lock is still owned. The lock may be released only by consuming the
completed transaction or by fail-closed unwind; no successful outcome is returned after early lock
release. Rebind, rename, replacement, physical-identity mismatch or uncertainty, unsafe ACL or
ownership, unsupported no-follow/locking/atomic-publication/`fsync` semantics, or use of a handle
from another physical root fails closed. A lock retained on the former root never authorizes a
write to the lexical replacement: the replacement root must receive zero reads used for mutation,
writes, temps, renames, removals, or publication. The originally opened root may be reconciled only
through its retained handles and exact contract; otherwise the transaction returns failure without
fabricating success.

No production authority root or initialization marker may become reachable while any applicable
in-repository legacy writer can bypass this capability. A1 activation remains unavailable until
complete guarded-writer adoption and its cross-process proof pass.

A1 has no legacy-authority migration mode. Under the same trusted bootstrap-home handle and root
lock, classification safely enumerates both pre-A1 authority collections:
`run/agent-hub/sessions/` recursively and `run/agent-hub/participants/` non-recursively. A missing
collection is empty only on exact `ENOENT`; a present collection must be an owner-controlled,
no-follow-opened directory whose complete enumeration succeeds. An empty validated directory is
allowed. Any regular record, nested record/directory, symlink, hard link, device, socket, unknown
entry, or other artifact is pre-A1 authority state. Config, policy, and inventory paths outside
these two collections are not legacy authority artifacts and remain allowed.

Traversal is component-by-component and directory-relative from the already-open trusted
bootstrap-home descriptor; concatenated absolute paths and path re-resolution are forbidden. The
bootstrap home and every existing `run`, `agent-hub`, collection, and recursively visited sessions
component must be a no-follow-opened directory owned by the effective owner, with no group/world
write bit and no ACL granting another principal write, rename, delete, or traversal authority.
Each opened component's physical identity is captured using the platform-specific fields of
`DirectoryPhysicalIdentityV1`. A symlink/reparse point, non-directory ancestor, owner mismatch,
unsafe mode/ACL, cross-device rebinding, or identity-query failure is `CorruptOrUnsupported`.

The classifier retains the opened component descriptors and their physical identities for the
whole locked initialization attempt. If a suffix is absent, it retains the deepest existing safe
parent descriptor and the exact missing components. Immediately before root-temp creation and
again before root rename, it reopens each child directory-relative, verifies every existing
identity/owner/mode/ACL against the retained observation, verifies every missing suffix is still
exact `ENOENT`, and repeats the complete empty-collection enumeration. Any replacement, rename,
new entry, identity change, or validation uncertainty aborts without certificate/root publication.
Every later semantic transaction performs the same safe traversal and empty enumeration. A
same-owner unmodified or mixed-version process is outside the supported concurrency model and must
be quiesced; other filesystem principals are excluded by the required ownership/mode/ACL checks.

Classification precedence is exact: any authority-layout/root/marker/key/temp validation failure or
collection-enumeration uncertainty is `CorruptOrUnsupported`; otherwise any safely observed pre-A1
artifact is `UnsupportedLegacyState`; otherwise exactly one of `FreshAbsent`,
`InitializationPending`, or `ValidExisting` applies. Under that lock, bootstrap has exactly five
classifications:

1. `FreshAbsent`: after successful temp reconciliation and complete legacy-collection enumeration,
   `state-root-v1.json` and `init-v1.json` are both proven absent; `keys/`, `objects/`, `tmp/`, and
   both legacy authority collections contain no entries; and only the validated empty authority
   directory skeleton and `root.lock` exist. Only this state may begin initialization and mint the
   greenfield certificate.
2. `InitializationPending`: after successful temp reconciliation and complete empty
   legacy-collection enumeration, `state-root-v1.json` is proven absent and `init-v1.json` is a
   complete, strictly decoded `CanonicalJsonV1(AuthorityStoreInitializationV1)`. No objects or
   unrelated keys/temps exist. The one matching initial key file may be absent or complete.
   Recovery resumes only when the marker's persisted `bootstrap_home` equals the trusted handle's
   physical identity, then reuses the exact stored store ID, key ID, home, and timestamp; it never
   generates replacement identities.
3. `ValidExisting`: `state-root-v1.json` is a complete strictly decoded root whose store ID, bound
   `bootstrap_home`, greenfield certificate, key registry/files, maps, refs, permissions, and
   filesystem semantics all validate against the trusted handle and both legacy authority
   collections still enumerate empty. The object tree contains only closed kind slugs, minimal
   `v1` directories, registered immutable objects, safely empty known kind/version directories, and
   safely named unindexed orphan object files. An orphan must be a no-follow regular owner-only
   `0600` file at the exact typed path for a syntactically valid ref ID; it has no authority or
   reachability meaning. Unknown, unsafe, or malformed directory/object entries are corruption. A
   matching leftover init marker is removed only after its store ID, key ID, home, and timestamp
   match the root, certificate, and registry.
4. `UnsupportedLegacyState`: a complete safe enumeration finds any pre-A1 authority artifact.
   A1 does not parse, migrate, quarantine, rank, merge, delete, or select a winner from it; it
   creates no root/certificate and accepts no transition. The only recovery is an explicit
   operator/developer reset outside the A1 runtime contract.
5. `CorruptOrUnsupported`: every other state, including an unreadable or incompletely enumerable
   legacy collection, a present invalid/unreadable root, a root
   symlink or non-regular file, unsafe ownership/permissions, unsupported locking or fsync,
   root-without-required-key, key/object/temp artifacts without a valid root or init marker,
   mismatched init/key/root identities, or any malformed/partial file. This fails closed and can
   never trigger fresh-store creation or namespace replacement.

Missing/unreadable is never collapsed to absent: absence is accepted only from the no-follow,
directory-relative lookup result that distinguishes `ENOENT` from every other error. In
particular, decode failure, access denial, I/O failure, wrong type, symlink refusal, and unsupported
filesystem behavior are `CorruptOrUnsupported`.

A `FreshAbsent` initializer fixes the trusted handle's `CanonicalDirectoryV1` as `bootstrap_home`,
generates one immutable `authority_store_id` as `as_` plus 32 lowercase
hexadecimal characters encoding 128 OS-CSPRNG bits, one key ID, one 256-bit key, and one timestamp,
then performs this exact protocol:

1. Write `CanonicalJsonV1(AuthorityStoreInitializationV1)` to
   `tmp/init--<authority_store_id>--<nonce>.tmp`, `fsync` it, rename it
   with no-replace semantics to `init-v1.json`, and `fsync(authority-v1/)`.
2. Publish and `fsync` the matching immutable key envelope and `keys/` as specified above.
3. Re-enumerate both legacy authority collections under the still-held lock, then write the
   revision-1 root with that store ID, the identical persisted bootstrap home, a
   `GreenfieldNamespaceCertificateV1` carrying the same store ID/home/timestamp, one `Active` key
   registry entry, and empty maps to `tmp/root--r1--<nonce>.tmp`; `fsync` it, publish with
   no-replace semantics, and `fsync(authority-v1/)`. Any artifact or enumeration uncertainty aborts
   without publishing the root.
4. Delete `init-v1.json`, `fsync(authority-v1/)`, and only then report initialization success.

`InitializationPending` with no key repeats step 2 using a newly generated 256-bit secret under the
already-fixed key ID; with a complete key it validates that envelope only against the init marker
and fixed algorithm rule above, then reuses that exact file. Whether newly published or reused,
recovery reopens and revalidates the key file, `fsync`s the key file, and `fsync`s `keys/` before
performing steps 3–4. A crash after root publication is a committed initialization even if the
marker remains; a crash before root publication is never inferred from key presence alone. A
competitor always re-reads under the lock and joins the same pending or committed initialization.

Every object-creating or semantic root transaction uses this order:

1. Open `authority_store_root` through its trusted directory handle and revalidate its committed
   physical identity.
2. Reject unsafe/symlinked authority paths, wrong ownership, unsafe permissions, or an unsupported
   filesystem.
3. Acquire the cross-process exclusive lock on `authority-v1/lock/root.lock`.
4. Read the closed schema-version discriminator and strictly decode exactly the current
   `StateRootV1` or `StateRootV2`. A1.2a's upgrade is the only semantic operation permitted on V1;
   Start issuance/application and every R0 semantic mutation require strict V2. Before any A1.2b
   semantic mutation, its separately reviewed packet must extend this closed discriminator and
   transaction path for strict V3. Shared key
   lifecycle and validation preserve their version without defaulting, down-converting, or
   accepting mixed-version state.
5. Re-enumerate both legacy authority collections as empty and validate the greenfield certificate,
   `root_revision`, object state, namespace reservation, intent/application revisions, and all
   semantic preconditions. A pre-A1 artifact fails closed and cannot be imported or ignored.
6. Traverse `objects/<closed-kind-slug>/v<minimal-decimal-schema-version>` component-by-component
   from the no-follow-opened `objects/` descriptor. Existing components must be owner-only `0700`
   directories with the expected names. A missing closed kind or version directory is created
   under the lock with `mkdirat`-equivalent no-follow semantics and mode `0700`; each created
   directory and each affected parent is `fsync`ed before continuing. Unknown kind/version names,
   unsafe existing entries, or create/validation/`fsync` uncertainty fail closed. A crash may leave
   only safely empty known kind/version directories; `ValidExisting` accepts those as routing
   structure without granting object authority.
7. Write each new immutable object's bytes to
   `authority-v1/tmp/object--<ref_id>--<nonce>.tmp` on the same filesystem.
8. `fsync` each object temp file.
9. Atomically rename it with no-replace semantics into its final typed object location; an existing
   location is an exact-retry candidate only after complete verification.
10. `fsync` every affected object directory.
11. Re-read or otherwise revalidate the still-locked root revision before publication.
12. Write the new canonical root to
    `authority-v1/tmp/root--r<new-root-revision>--<nonce>.tmp`.
13. `fsync` the root temp file.
14. Atomically replace `authority-v1/state-root-v1.json`.
15. `fsync` the `authority-v1` parent directory.
16. Release the lock only after the transaction has a verified outcome.

Exact retry occupancy is semantic, not variant-name-based. A root is not authority-free merely
because `session_namespace_map` contains no `Authority` variant. Any `StartReservation`,
`StartTombstone`, transition-intent entry, issuer-request-index entry, application-journal entry, or
object-index entry is semantic authority state and participates in exact retry and conflict
validation. A retry may join only when every applicable store/session/request/intent identity,
payload commitment, root and authority revision, namespace record, object/index state, journal,
and immutable application field matches the already committed result exactly. Empty or partial
matching of one map can never authorize a join.

Root publication uses a post-reconciliation publication candidate. After temp, orphan, released
object, and key-file reconciliation, and immediately before publishing a replacement root under the
same retained trusted root and lock, rotation, retirement, CAS, and any other root publisher must
validate the complete candidate against:

- the current locked root and exact expected `root_revision` plus any expected authority revision;
- exact bootstrap-home and authority-store identity;
- the complete key registry and key files, including exactly one active key and all
  active/verification/retired constraints;
- every reachable typed object reference, object-index entry, and required object file/state;
- all namespace, reservation, tombstone, transition-intent, issuer-index, application-journal, and
  object-index invariants;
- complete safe enumeration proving both pre-A1 authority collections are empty; and
- the unchanged trusted physical-root identity and retained descendant identities.

Rotation and retirement may not rely solely on validation performed before reconciliation. A
stale, conflicting, missing, substituted, malformed, or post-reconciliation-invalid candidate
fails before root-temp creation/publication and without unauthorized cleanup or semantic mutation.

On Unix every file in this layout, including root, object, temp, key, and lock files, is `0600`;
every directory at or below `authority-v1`, including kind/version object directories, is `0700`.
The effective owner must own them, and group/world-writable components or ACLs granting another
principal access are rejected. Access is directory-relative/openat-style with `O_NOFOLLOW`
or the platform equivalent where available. A process-local mutex is insufficient. Kernel release
of the lock after process death is only an observation that permits the next process to acquire,
re-read, and reconcile; it does not prove, roll back, or complete a transaction.
Filesystems/platforms lacking trustworthy cross-process locks, no-follow access, same-filesystem
atomic no-replace/object publication, atomic root replacement, file and directory `fsync`, or
equivalent owner-only semantics fail closed. Windows remains fail-closed under the path rule above
until equivalent ACL, lock, and atomic-replace semantics are specified and proven.

Crash meaning is exact:

- Object bytes with no committed root/object-index parent are orphans. They have no authority
  meaning. A1's safe grace rule is retention: A1 authorizes no orphan deletion. A later
  schema/version may add garbage collection only with locked reconciliation plus an explicit
  time/evidence grace rule, so A1.1 does not invent one.
- A retained orphan may be adopted only during an exact retry that presents the same complete
  `AuthorityObjectRefV1`—ref ID, kind, schema, commitment variant/domain/key/digest—the exact parent
  intent/run/store context required by any HMAC wrapper, and exact final file bytes that verify the
  resulting commitment input. The locked retry then supplies the missing semantic root parent and
  object-index entry in its normal root transaction. Without that complete exact retry, the orphan
  remains retained and non-authoritative; its filename or bytes alone never reconstruct an intent,
  request, authority, or parent.
- `ObligationSnapshot` is stricter than the generic orphan rule. After a crash between snapshot
  publication and root commit, it may be adopted only if `ObligationLedger` reissues/revalidates
  the same Complete snapshot bytes under the still-current per-session ledger revision and
  materialization cut and every store/session/participant/intent/run/authority binding still
  matches. Otherwise the orphan remains non-authoritative and a new current snapshot/ref is
  published; an old empty or non-empty cut is never adopted merely because its bytes verify.
- A committed parent ref in a required-present state whose object is missing, wrong-kind,
  wrong-version, wrong-domain/key, or commitment-mismatched is store corruption. Fail closed and
  never infer success.
- `ReleaseEligible` with the transport object present may retry deletion idempotently. Deletion is
  directory-relative and the object directory is fsynced before the root records `Released`.
- `ReleaseEligible` with the transport object absent may become `Released` only after verifying the
  exact committed terminal-handoff ref and commitment in both intent and object index. Verified
  `ENOENT` is followed by `fsync` of that exact transport object directory before the root may
  record `Released`, matching the durability barrier used after successful deletion.
- `Released` is the sole intentional missing-object state. It requires the same exact terminal
  handoff and no remaining transport bytes. An unexpected surviving copy is removed
  directory-relative under the lock and the exact transport object directory is `fsync`ed before
  retry may join/report success; removal or durability uncertainty fails closed. The bytes never
  regain authority meaning.
- `Retained`/`Present` with a required object absent is store corruption and fails closed.
- If root replacement completed but the parent-directory `fsync` failed, durability is unproven:
  the operation must not report durable success. Recovery acquires the lock, accepts only a
  complete strictly decoded root and matching objects, and reconciles idempotently from the last
  verifiable root/application state. It never treats an orphan object as proof that the new root
  committed.

Production A1 timing is fixed:

```text
HOST_SESSION_TRANSITION_INTENT_DEFAULT_TTL = 300 seconds
HOST_SESSION_TRANSITION_INTENT_MAX_TTL     = 900 seconds
HOST_SESSION_TRANSITION_CLAIM_LEASE        = 30 seconds
HOST_SESSION_TRANSITION_CLAIM_LEASE_MAX    = 60 seconds
```

CLI, REPL, helper, and auto-attach producers use the default intent TTL and claim lease. Tests may
use a shorter injected clock/bound only; production environment variables or plan contents cannot
change these values in V1. Issuance rejects a non-positive TTL, a TTL above the maximum, or a claim
lease outside the positive maximum. Neither retry nor reclaim extends `expires_at`.

## 1. `DurableSessionAuthorityV1`

```rust
enum DurableSessionAuthorityOriginV1 {
    StartIntent {
        intent_id: String,
        issuer_request_id: String,
        payload_commitment: AuthorityObjectCommitmentV1,
    },
}

struct DurableSessionAuthorityV1 {
    schema_version: u32,                 // exactly 1
    orchestration_session_id: String,
    shell_trace_session_id: String,
    authority_revision: u64,
    origin: DurableSessionAuthorityOriginV1,
    authoritative_participant_lineage: Vec<String>,
    active_authoritative_participant_id: Option<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    host_attach_contract_ref: Option<AuthorityObjectRefV1>,
    retained_worker_refs: Vec<AuthorityObjectRefV1>,
    internal_resume_handle_refs: Vec<AuthorityObjectRefV1>,
    lifecycle_posture: HostSessionPostureV1,
    current_policy_ref: Option<AuthorityObjectRefV1>,
    current_policy_revision: Option<String>,
    updated_at: TimestampV1,
}

enum HostSessionPostureV1 {
    ActiveAttached,
    ParkedResumable,
    DetachedReconciled,
    AwaitingAttention,
    Terminal,
    StaleRecoverable,
    Invalid,
}
```

Acceptance rules:

1. Only `HostSessionAuthority` may create a newer `authority_revision` or change posture, lineage,
   binding, attach contract, refs, or authoritative participant.
2. Every authority is Start-created and retains the exact root-map `intent_id`, issuer request, and
   payload commitment that owned its former reservation. `StartIntent` is the only V1 origin and is
   immutable.
3. Every ref validates its closed kind: attach contract is `HostAttachContract`, retained workers
   are `RetainedWorker`, resume handles are `ResumeHandle`, and policy is `Policy`. Kind, schema,
   key/domain where applicable, and parent-owned commitment are checked before use.
4. `AwaitingAttention` is derived from unresolved attention-driving obligations; it is not set
   from a worker flag or inbox row alone.
5. `Terminal` is monotonic unless a separately versioned recovery protocol explicitly creates a
   successor session; it is never reversed in-place.
6. `HostSessionPostureV1::Invalid` is permitted only for an otherwise fully representable
   `DurableSessionAuthorityV1` that later becomes invalid or unroutable. It cannot supply missing
   identity, lineage, binding, contract, or ref fields.
7. World binding is the exact complete `(world_id, world_generation)` pair. Partial binding is
   unrepresentable, not an `Invalid` authority.
8. PID, socket, heartbeat, attached-client, helper, and plan data do not belong in this contract.
9. `shell_trace_session_id`, `workspace_binding.authority_store_root`,
   `workspace_binding.authority_store_id`, and the initial `workspace_binding.workspace_root` are
   immutable for one authority record. Attach and `ResumeOneTurn` match their complete physical
   identities; a different store or workspace requires a separately versioned explicit rebinding
   protocol outside A1.

### Runtime placement and durable session world binding

`AgentDescriptorV1.execution_scope` and
`HostAttachLaunchKnobsV1.requested_execution_scope` are runtime-placement truth.
`DurableSessionAuthorityV1.world_binding` is the durable parent orchestration session's exact
available world-substrate truth. These dimensions are independent and are not bijective. A
host-executing orchestrator may own a world-backed durable session; a world-executing runtime must
have an exact world binding.

Start issuance validation, application/persistence, and exact current-authority resolution freeze
this complete matrix:

| Descriptor and launch scope | Session world binding | Result |
|---|---|---|
| `Host` | `None` | accept |
| `Host` | `Some(exact binding)` | accept |
| `World` | `Some(exact binding)` | accept |
| `World` | `None` | reject |

In every accepted row, descriptor scope exactly equals requested launch scope. Every `Some` is the
exact complete `(world_id, world_generation)` pair supplied by session-authority truth; an empty or
malformed binding is rejected, and no compatibility input may synthesize one. Changing either
world ID or generation changes the canonical Start request, so it cannot exact-join an earlier
request. `Host + Some` does not mean the host runtime executes in the world. The host participant
descriptor and manifest stay host-scoped and receive no participant-level world placement fields;
the binding is persisted only as session authority.

A1.2a-WB changes validation semantics only. It adds no field or version, changes no canonical JSON
byte or golden vector, creates no V3/migration/compatibility bridge, and rewrites no persisted
object. World filesystem, network, caging, policy, capability, and enforcement semantics remain
unchanged.

The implementation boundary is exactly `transition.rs`, colocated `transition_tests.rs`, and
`facade.rs::HostSessionAuthority::resolve_current_exact`. The reader must return the exact persisted
binding unchanged and may not reject an authority already accepted and persisted under the matrix
through the obsolete `Host + Some` rule. All other facade behavior is outside scope. This
write/read boundary is review-clean through `275f9fa2`, and the dependent A1.2a-S adoption is
review-clean through `2f2fecb3`; B1/B2.1-R0 is review-clean through `bb3eefba`, and B3.2a plus
B3.2a-WA are review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.

## 1A. strict `HostSessionTransitionIntentV1`/`HostSessionTransitionIntentV2`

The landed `HostSessionTransitionIntentV1` byte shape remains strict and readable without defaults
or added fields. Production A1.2a Start uses the strict V2 shape after the greenfield root upgrade;
later Attach/Resume transition mutation uses the separately reviewed strict V3 shape. A
hidden-helper launch plan is only a private transport
projection of the selected record. Reading or deleting that plan does not consume, apply, reject,
or expire the intent.

```rust
struct HostSessionTransitionIntentV1 {
    schema_version: u32,                 // exactly 1
    intent_id: String,                   // globally unique in the authority store
    issuer_request_id: String,           // idempotency key for issuance
    intent_revision: u64,                // starts at 1; monotonic under CAS
    mode: HostSessionTransitionModeV1,
    authority_precondition: HostSessionAuthorityPreconditionV1,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    caller: HostSessionTransitionCallerV1,
    source_authoritative_participant_id: Option<String>,
    target_authoritative_participant_id: String,
    target_participant_lease_token_ref: AuthorityObjectRefV1,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AuthorityObjectRefV1,
    host_attach_contract_ref: AuthorityObjectRefV1,
    resume_handle_ref: Option<AuthorityObjectRefV1>,
    transition_input_ref: Option<AuthorityObjectRefV1>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
    transport_payload_ref: AuthorityObjectRefV1,
    payload_commitment: AuthorityObjectCommitmentV1,
    issued_at: TimestampV1,
    expires_at: TimestampV1,
    state: HostSessionTransitionIntentStateV1,
    input_handoff: HostSessionTransitionInputHandoffV1,
    transport_payload_state: HostSessionTransitionTransportPayloadStateV1,
    updated_at: TimestampV1,
}

struct HostSessionTransitionIntentV2 {
    schema_version: u32,                 // exactly 2
    intent_id: String,
    issuer_request_id: String,
    intent_revision: u64,
    mode: HostSessionTransitionModeV1,
    authority_precondition: HostSessionAuthorityPreconditionV1,
    orchestration_session_id: String,
    shell_trace_session_id: String,
    caller: HostSessionTransitionCallerV1,
    source_authoritative_participant_id: Option<String>,
    target_authoritative_participant_id: String,
    target_participant_lease_token_ref: AuthorityObjectRefV1,
    run_id: String,
    resulting_authoritative_lineage: Vec<String>,
    workspace_binding: WorkspaceBindingV1,
    world_binding: Option<WorldBindingV1>,
    descriptor_ref: AuthorityObjectRefV1,
    host_attach_contract_ref: AuthorityObjectRefV1,
    resume_handle_ref: Option<AuthorityObjectRefV1>,
    transition_input_ref: Option<AuthorityObjectRefV1>,
    post_turn_disposition: Option<HostPostTurnDispositionV1>,
    transport_payload_ref: AuthorityObjectRefV1,
    payload_commitment: AuthorityObjectCommitmentV1,
    issued_at: TimestampV1,
    expires_at: TimestampV1,
    state: HostSessionTransitionIntentStateV2,
    input_handoff: HostSessionTransitionInputHandoffV1,
    transport_payload_state: HostSessionTransitionTransportPayloadStateV1,
    updated_at: TimestampV1,
}
```

Modes and authority preconditions:

```rust
enum HostSessionTransitionModeV1 {
    Start,
    Attach,
    ResumeOneTurn,
}

struct HostSessionTransitionCallerV1 {
    kind: HostSessionTransitionCallerKindV1,
    caller_participant_id: Option<String>,
    auto_attach_obligation_id: Option<String>,
    auto_attach_claim_owner: Option<String>,
}

enum HostSessionTransitionCallerKindV1 {
    PublicCli,
    Repl,
    RouterAutoAttach,
}

enum HostSessionAuthorityPreconditionV1 {
    ExpectedAbsent,
    ExpectedRevision {
        authority_revision: u64,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        active_authoritative_participant_id: String,
        authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
        lifecycle_posture: HostSessionPostureV1,
    },
}

enum HostPostTurnDispositionV1 {
    ReconcileToAttentionParkOrTerminal,
}
```

Intent states:

```rust
enum HostSessionTransitionIntentStateV1 {
    Issued,
    Claimed {
        claim_id: String,
        claimant_attempt_id: String,
        claim_revision: u64,
        claimed_at: TimestampV1,
        claim_expires_at: TimestampV1,
    },
    Applied {
        claim_id: String,
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        application_result_ref: AuthorityObjectRefV1,
        post_turn: HostSessionPostTurnApplicationV1,
        applied_at: TimestampV1,
    },
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
        terminal_handoff_ref: AuthorityObjectRefV1,
        rejected_at: TimestampV1,
    },
    Expired {
        terminal_handoff_ref: AuthorityObjectRefV1,
        expired_at: TimestampV1,
    },
}

enum HostSessionTransitionIntentStateV2 {
    Issued,
    Claimed {
        claim_id: String,
        claimant_attempt_id: String,
        claim_revision: u64,
        claimed_at: TimestampV1,
        claim_expires_at: TimestampV1,
    },
    Applied {
        claim_id: String,
        claimant_attempt_id: String,
        authority_revision_before: Option<u64>,
        authority_revision_after: u64,
        active_authoritative_participant_id: String,
        resulting_posture: HostSessionPostureV1,
        authority_record_commitment: AuthorityObjectCommitmentV1,
        application_result_ref: AuthorityObjectRefV1,
        startup_ownership: HostSessionStartupOwnershipApplicationV1,
        post_turn: HostSessionPostTurnApplicationV1,
        applied_at: TimestampV1,
    },
    Rejected {
        reason: HostSessionTransitionTerminalRejectionV1,
        terminal_handoff_ref: AuthorityObjectRefV1,
        rejected_at: TimestampV1,
    },
    Expired {
        terminal_handoff_ref: AuthorityObjectRefV1,
        expired_at: TimestampV1,
    },
}

enum HostSessionPostTurnApplicationV1 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
    },
    Applied {
        completion_ref: AuthorityObjectRefV1,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        application_result_ref: AuthorityObjectRefV1,
        applied_at: TimestampV1,
    },
}

// A1.2b-only after B1/B2.1, B3.1, and C1; not compiled by A1.2a.
enum HostSessionPostTurnApplicationV2 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
    },
    AwaitingObligationCut {
        completion_ref: AuthorityObjectRefV1,
        expected_run_id: String,
        expected_authority_revision: u64,
        acceptance_record_id: String,
        acceptance_record_revision: u64,
        stream_id: String,
        accepted_work_identity: AcceptedWorldWorkIdentityV1,
        host_transition_correlation: HostTransitionWorkCorrelationV1,
        required_terminal_event_id: String,
        required_terminal_event_sequence: u64,
        recorded_at: TimestampV1,
    },
    Applied {
        completion_ref: AuthorityObjectRefV1,
        obligation_snapshot_ref: Option<AuthorityObjectRefV1>,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        application_result_ref: AuthorityObjectRefV1,
        applied_at: TimestampV1,
    },
}

enum HostSessionStartupOwnershipApplicationV1 {
    NotApplicable,
    Pending {
        expected_run_id: String,
        expected_authority_revision: u64,
        expected_active_authoritative_participant_id: String,
    },
    Accepted {
        evidence_id: String,
        result_ref: AuthorityObjectRefV1,
        authority_revision: u64,
        accepted_at: TimestampV1,
    },
    TerminalReconciled {
        evidence_id: String,
        result_ref: AuthorityObjectRefV1,
        authority_revision_before: u64,
        authority_revision_after: u64,
        resulting_posture: HostSessionPostureV1,
        reconciled_at: TimestampV1,
    },
}

enum HostSessionTransitionInputHandoffV1 {
    NotApplicable,
    Pending {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
    },
    Accepted {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
        acceptance_ref: AuthorityObjectRefV1,
        accepted_at: TimestampV1,
    },
    TerminalWithoutAcceptance {
        input_ref: AuthorityObjectRefV1,
        run_id: String,
        terminal_handoff_ref: AuthorityObjectRefV1,
        terminal_at: TimestampV1,
    },
}

enum HostSessionTransitionTerminalRejectionV1 {
    InvalidCommittedModePrecondition,
    AuthorityPreconditionNoLongerHolds,
    StaleAuthorityRevision,
    AuthorityRecordCommitmentMismatch,
}

enum HostSessionTransitionAttemptRejectionV1 {
    UnknownIntent,
    IntentOrRequestIdentityMismatch,
    IntentRevisionMismatch,
    ClaimRevisionMismatch,
    PayloadCommitmentMismatch,
    ObjectRefOrCommitmentMismatch,
    SessionIdentityMismatch,
    CallerIdentityMismatch,
    ParticipantOrLineageMismatch,
    WorkspaceBindingMismatch,
    WorldBindingMismatch,
    DescriptorProjectionMismatch,
    AttachContractProjectionMismatch,
    ResumeHandleProjectionMismatch,
    TransitionInputMismatch,
    SupersededClaim,
    ConflictingIntent,
}

enum HostSessionTransitionTransportPayloadStateV1 {
    Retained,
    ReleaseEligible {
        terminal_handoff_ref: AuthorityObjectRefV1,
    },
    Released {
        terminal_handoff_ref: AuthorityObjectRefV1,
        released_at: TimestampV1,
    },
}
```

V1 validation, reachability, retry, and canonical encoding use only the landed V1 fields; V1
`Applied` has neither `claimant_attempt_id` nor `startup_ownership`, and no decoder may default or
infer them. V2 validation requires the persisted claimant attempt and closed startup-ownership
substate, accepts only mode `Start`, and requires `post_turn=NotApplicable`; its reachable schema
contains no B1 accepted-work or correlation type. A V1 root containing a V2 intent, a V2 root
containing a V1 intent, or a state payload from the wrong intent version is mixed-version
corruption. Every exact retry and reachable-object walk dispatches from the root version through
the matching intent-state version.

A1.2b may not widen V2 in place. After the joint B1/B2.1 closeout, B3.1, and C1 make their shared
types available, A1.2b must first freeze and independently review a strict V3 root/intent/state
extension. That later root preserves each existing V2 Start intent as an unchanged closed V2
member, preserves both R0 maps and all authority/application proof bytes, and permits new
Attach/Resume intents only through a separate V3 member using
`HostSessionPostTurnApplicationV2`; its V3 application journal likewise uses the separate
`PostTurnApplicationJournalV2` rather than widening V1. Its V2-to-V3 publication, crash, exact-retry, key-lifecycle,
and reachability rules must be specified before A1.2b implementation. Neither the V3 type nor any
B1 accepted-work/correlation import is part of A1.2a.

All rules below that mention Attach, ResumeOneTurn, `AwaitingObligationCut`, a Complete ledger cut,
or B1 work correlation apply only to the later A1.2b V3 member. A1.2a implements only the closed
V2 Start subset and rejects those modes/states before persistence.

Attempt rejection and terminal intent rejection are separate outcomes:

| Validation outcome | Durable effect |
|---|---|
| Invalid request before a valid intent is issued | Reject the request; persist no intent, reservation, tombstone, object-index entry, or issuer-index entry. An unrenamed temp is removed by temp reconciliation; a final object published before the failed root commit is only an orphan. |
| Exact `intent_id`, `issuer_request_id`, and `payload_commitment` presented for an existing intent | Return/join its current claim or stored transition/input/post-turn/terminal result as allowed by state, even if the caller retained an older intent revision; do not reissue or reapply. |
| Unknown/substituted/mismatched attempt, stale intent/claim revision used for a claim/application/state mutation, superseded claim, or conflicting replay against a valid stored intent | Return and audit `HostSessionTransitionAttemptRejectionV1`; do not mutate, reject, expire, release, or otherwise strand the stored intent. |
| The stored intent was validly issued, the root/object/key set remains valid, but its committed authority precondition no longer holds before application | Commit `Rejected` plus one exact `TerminalHandoff` ref. For Start, atomically replace its exact reservation with an intent-linked permanent tombstone; no authority mutation is permitted. |
| The fixed intent expiry passes before application and no authority commit exists | Commit `Expired` plus one exact `TerminalHandoff` ref. For Start, atomically replace its exact reservation with an intent-linked permanent tombstone. |
| The exact stored intent and attempt validate | Continue the `Issued`/`Claimed`/`Applied` protocol. |

A substituted request using a stored `intent_id` or `issuer_request_id` with a different
`payload_commitment` is therefore attempt-rejected and audit-recorded without mutating or
terminalizing the valid stored intent. A missing/mismatched committed object or required HMAC key
is store corruption, not a terminal intent rejection; it fails closed without mutating the intent,
reservation, namespace, or release state.

### Immutable commitments and identity

1. `intent_id` identifies exactly one logical transition. `issuer_request_id` is store-global and
   indexes that intent. Exact retry requires the same session ID, intent ID, issuer request, and
   payload commitment; reuse with different content or identity fails closed.
2. `payload_commitment` must be `CanonicalSha256` over exactly
   `HostSessionTransitionPayloadHashInputV1`, including fixed issuance/expiry timestamps and every
   complete typed ref. It excludes only `intent_revision`, mutable state/claim/handoff/retention
   fields, `updated_at`, and terminal results. No presentation struct or adjacent metadata is
   hashed. The issuer fixes intent ID and both timestamps once before the first root commit. Retry
   first resolves the issuer index and reconstructs this wrapper with those stored timestamps;
   it never samples a new clock value to decide equality.
3. Descriptor, attach contract, resume handle, participant lease token, transition input, and
   transport payload are immutable objects. Their parent-held refs are verified before issuance
   commit where newly published, and again before claim, application, reprojection, or use. Plan
   JSON may not replace a ref or commitment.
4. The lease-token ref is kind `LeaseToken` with the participant-lease HMAC domain. The optional
   input ref is kind `TransitionInput` with the transition-input HMAC domain. The transport ref is
   kind `TransitionTransportPayload` with the raw-transport HMAC domain. Descriptor and attach
   contract are canonical structured refs; resume handle uses its closed kind-specific schema.
   Credentials are prohibited, and prompt/input/lease/transport bytes and reusable commitments
   never enter logs, traces, diagnostics, rejection text, compatibility snapshots, or object IDs.
5. `expires_at` is fixed at issuance, later than `issued_at`, and bounded by the V1 maximum.
   Ambient retries, plan rewrites, helper restarts, and reclaim do not extend it.
6. `intent_revision` increments on every durable intent-state, claim, input-handoff,
   startup-ownership, post-turn (including `AwaitingObligationCut`), or transport-payload-state
   change. A stale intent or claim revision cannot mutate the record; this does not block an exact
   read-only join.
7. The namespace and authority precondition permit at most one transition candidate for the exact
   reserved state. A different mode, target, issuer, intent, or commitment is a conflict, never a
   second candidate.
8. `PublicCli` and `Repl` callers have no auto-attach fields. `RouterAutoAttach` is valid only for
   `Attach` and commits the already-existing obligation ID and claim owner; A1 neither decides
   eligibility nor changes claim/settlement semantics. A present caller participant matches the
   exact authoritative caller. A new Start has none.
9. Pre-A1 `source_orchestration_session_id` plan data is rejected. Startup-prompt stream paths,
   helper PIDs, sockets, and plan paths are transport-only and cannot enter reservation or
   authority truth. Existing builders may reproject them only from verified retained objects.
10. `shell_trace_session_id` exactly matches the authority record for Attach/Resume and is committed
    by the Start payload and resulting authority. It cannot be recovered from ambient tracing
    state.

### A1 packet ownership boundary

A1.1 owns only the authority-store substrate: bootstrap classification and recovery, the normalized
bootstrap-home binding, cross-process locking, exact object publication/verification, key lifecycle,
greenfield namespace certification plus fail-closed pre-A1-state detection, exact authority
resolution, and generic root/authority revision-CAS. A1.1 does not accept production
`ExpectedAbsent`, create a `StartReservation`, issue or apply a transition intent, allocate a Start
participant, or create a Start-origin authority.

To make the same-home invariant implementable, A1.1 may add explicit-bootstrap-home parameters or
sibling entry points only to the existing common path, effective config, effective policy/policy
snapshot, and agent-inventory resolvers named by the A1 packet. Those entry points consume the
already-open `CanonicalDirectoryV1`/trusted handle and preserve all existing workspace/global
precedence, merge, parsing, and policy interpretation. A1.1 does not switch real callers. A1.3 may
thread that same handle only through its named CLI/REPL Start/Attach/Resume adoption paths; it may
not broaden resolver semantics or convert unrelated callers.

A1.2a is the first packet allowed to perform production Start semantics. It owns only greenfield
certificate validation, `ExpectedAbsent` acceptance, Start reservation plus intent issuance,
claim/application, initial Start-origin authority birth, crash reconciliation/exact retry, and the
typed current-authority read result required by B1/B2.1-0. Applied Start retains the exact claimant
attempt and records `HostSessionStartupOwnershipApplicationV1::Pending` for its run, authority
revision, and active participant; A1.2a does not resolve that substate. It does not own
startup-ownership/post-turn episode reconciliation, Attach/ResumeOneTurn, obligation-cut consumption, world-work correlation,
or public consumer adoption. R0 may later advance the authority only through the closed
non-transition registration chain defined below; the Pending substate continues to name the
original application revision.

A1.2b remains after the B1/B2.1 joint closeout, B3.1, and C1. Through the separately versioned V3
extension it owns successor
Attach/ResumeOneTurn, startup/post-turn reconciliation, ledger-snapshot consumption, correlation
supply, and release as the remainder of the aggregate A1.2 intent protocol. Startup reconciliation accepts
the original application revision only through the unique contiguous R0-registration ancestry
rule below; arbitrary stale authority remains rejected. Neither A1.2 packet owns runtime
event identity, receipt acceptance, durable observation, retained-event semantics, canonical
obligations, or their event/materialization cut. Until that corridor exists, A1.2b retains
`AwaitingObligationCut` and cannot claim aggregate A1.2 closure. A1.3 adopts the protocol on real
CLI/REPL consumers; A1.1 primitive tests are not evidence that a production Start path is adopted.

### Start namespace reservation

Start uses the namespace map and issuer index as one protocol:

1. **Pre-issuance:** A1.2a may evaluate a new `ExpectedAbsent` request only after the closed
   greenfield upgrade has published a complete strict `StateRootV2`, and while that root and the
   immutable `GreenfieldNamespaceCertificateV1` verify against the
   trusted bootstrap home and both pre-A1 authority collections re-enumerate empty. It then requires
   no `Authority`, `StartTombstone`, or foreign `StartReservation` for the session ID. A missing or
   unreadable individual record, absent compatibility projection, unverified/copy-created
   certificate, pre-A1 artifact, incomplete enumeration, or root error is never absence. A
   previously committed exact self reservation is handled only as the exact issuance retry below;
   it is not general absence.
2. **Issuance:** after preparing/verifying required immutable objects, one A1.2 root replacement creates
   the `Issued` intent, `issuer_request_index` entry, `object_index` entries, and
   `StartReservation` owned by the exact intent ID, issuer request ID, and payload commitment.
   External bytes published before this root have no issuance meaning.
3. **Exact issuance retry:** the same session ID, intent ID, issuer request ID, and payload
   commitment joins the indexed intent and its exact self reservation or terminal result. Any
   mismatch is an attempt rejection and cannot alter the stored intent/reservation/tombstone.
4. **Claim/application:** only the reservation whose three ownership fields exactly match the
   intent satisfies that intent's `ExpectedAbsent`. A foreign reservation, `Authority`,
   `StartTombstone`, missing reservation, or mismatched commitment fails closed. PID, socket,
   helper, plan, claim-owner liveness, and process-local state are irrelevant.
5. **Applied Start:** the same root transaction that records application journal and intent
   `Applied` atomically replaces `StartReservation` with `Authority`. The authority's
   `StartIntent` origin retains the exact intent, issuer, and payload commitment.
6. **Rejected or Expired Start:** the same root transaction that records the terminal intent,
   terminal handoff, and pending-input terminalization atomically replaces the reservation with a
   permanent `StartTombstone`. Its root-map `intent_id` remains the exact intent reference. The
   session ID is never reusable in A1; an exact retry joins the stored terminal result, while a
   different Start fails closed.
7. **Invalid before issuance:** invalid identity, path, schema, object, key, TTL, or mode input
   persists no intent, reservation, tombstone, issuer entry, or semantic root change.

### Exact mode preconditions

The posture domain is closed for V1:

| Mode | Required source posture | Initial resulting posture |
|---|---|---|
| `Start` | `ExpectedAbsent`, followed after issuance by the exact intent-owned self reservation only. | `ActiveAttached`, committed only when validated application is ready to own the exact target. |
| `Attach` | Exactly one of `ParkedResumable`, `DetachedReconciled`, `AwaitingAttention`, or an explicitly persisted `StaleRecoverable`; never a posture inferred from process loss during the attempt. | `ActiveAttached`. Existing obligations remain unchanged and canonical. |
| `ResumeOneTurn` | Exactly one of `ParkedResumable`, `DetachedReconciled`, `AwaitingAttention`, or an explicitly persisted `StaleRecoverable`, plus the exact resume handle. | `ActiveAttached` with `post_turn=Pending`. Exact accepted terminal evidence later yields `Terminal`; otherwise unresolved attention obligations yield `AwaitingAttention`, and a resumable clean result yields `ParkedResumable`. |

Every unlisted source posture or namespace variant fails closed. In particular, a new independent
Attach/Resume against `ActiveAttached`, `Terminal`, or `Invalid` is rejected; only an exact retry
of the already-Applied intent may join. No `StartTombstone` can satisfy any A1 transition. A1 does
not create, resolve, or change obligations when selecting the deterministic post-turn posture.

Parked-successor application sharpens these existing mode/precondition/application rules without
introducing another intent contract:

1. `Active` orchestration-session lifecycle with `ParkedResumable` posture is valid current durable
   authority, not session absence or a request to allocate a replacement session.
2. A successor cannot replace that authority with a caller-constructed `Allocating` snapshot.
3. `Attach` or `ResumeOneTurn` uses the exact expected authority revision and authority-record
   commitment/hash already required by `ExpectedRevision`.
4. Application atomically advances posture and authoritative lineage while preserving session
   identity, nonterminal lifecycle identity, and the exact world binding unless a separately
   authorized protocol changes that binding.
5. The target participant becomes the authority-approved successor only through transition
   application; StateStore persists that selected result and does not choose the transition.
6. Host execution episode construction and launch occur only after durable application.
7. `PID=0`, no active handle, helper/readiness state, a completed prior prompt, or a pending new
   prompt cannot satisfy, deny, or replace the authority precondition.
8. Prompt bytes may be committed through the immutable `TransitionInput` ref. Prompt delivery,
   stream, acceptance, pending, or completion status remains transport/application evidence, not
   authority.
9. Exact retry joins the committed intent, application, input-handoff, and post-turn result under
   the retry rules below; it never constructs another successor snapshot.
10. Failure after application reconciles from durable intent/application truth and must not restore
    a stale pre-transition snapshot or regress lifecycle to `Allocating`.

`Start` requires all of the following:

1. `authority_precondition` is `ExpectedAbsent`; A1.2 issuance first verifies the exact greenfield
   certificate, empty pre-A1 authority collections, and no namespace record. Later
   claim/application observes only the exact self reservation. Delete/recreate, compatibility
   hiding, a missing/copied certificate, an unreadable collection, any pre-A1 artifact, or any
   permanent namespace record does not satisfy it.
2. `source_authoritative_participant_id` and `resume_handle_ref` are absent.
3. The target participant, HMAC-bound `LeaseToken` ref, run ID, and shell-trace ID are new and
   exact. Resulting lineage is the valid initial lineage ending in that target. Caller is
   `PublicCli` or `Repl`, has no prior participant ID, and is bound to the unique issuer request.
4. The complete physical workspace/store binding and immutable store ID validate. World-scoped
   Start commits the full world pair; host-only Start commits none. Partial or ambient
   reconstruction fails closed.
5. Descriptor and attach-contract refs have the exact kinds/schemas/canonical commitments.
   Optional Start input is a `TransitionInput` HMAC ref; post-turn disposition is absent.
6. `input_handoff` is `Pending` with that exact input ref/run when input exists and
   `NotApplicable` otherwise.

`Attach` requires all of the following:

1. `ExpectedRevision` exactly matches current authority revision,
   `DurableSessionAuthorityHashInputV1` commitment, active participant,
   `AuthoritativeLineageHashInputV1` commitment, and one enumerated posture. PID, socket, helper,
   heartbeat, or attached-client observations cannot make it eligible.
2. Source is the current lineage tip. Target, HMAC-bound lease-token ref, and run are new and
   exact; target is a distinct valid successor and resulting lineage is the authority-approved
   append/replacement. Shell-trace identity is preserved exactly.
3. Physical workspace/store identity, world pair, descriptor ref, and attach-contract ref match
   the current authority and requested successor.
4. A `ResumeHandle` ref is present exactly when the committed attach contract requires it. Attach
   has no transition input and no post-turn disposition.
5. `input_handoff` is `NotApplicable`.

`ResumeOneTurn` requires all of the following:

1. The exact Attach revision/commitment, lineage, binding, descriptor, and attach-contract checks
   hold against one enumerated Resume posture.
2. The exact `ResumeHandle` ref is valid for current source participant and session.
3. A `TransitionInput` ref with the required HMAC key/domain/store/intent/run commitment is present
   and verifies over the exact one-turn input bytes.
4. `input_handoff` is `Pending` with that same input ref and run ID.
5. `post_turn_disposition` is `ReconcileToAttentionParkOrTerminal`. A verified
   `PostTurnCompletion` ref, claimed intent, current authority revision, and—only for a resumable
   outcome—a Complete `ObligationLedger` snapshot for the exact terminal event cut determine the
   revision-checked result. Pending materialization keeps reconciliation Pending; queue delivery,
   helper liveness, timeout, inbox rows/counts, and worker flags never determine posture.

### Lifecycle, retry, and fail-closed rules

1. Issuance verifies/publishes the immutable refs first, then commits `Issued`, issuer index,
   object index, and—only for Start—the exact reservation in one root revision before plan write or
   helper launch. Projection/launch failure leaves a fully reprojectable intent.
2. Claim CASes `Issued` to `Claimed` after revalidating payload commitment, key availability,
   expiry, namespace/authority precondition, identities, physical bindings, and every ref. Claim
   does not mutate authority or namespace.
3. Exact current-claim retry joins. A different claimant fails while the lease is current. After
   claim expiry, reclaim requires the current intent/root revisions and all preconditions; PID or
   socket liveness is never claim authority.
4. Application creates the immutable `ApplicationResult` object, then one root transaction commits
   authority mutation, application journal, intent `Applied`, result ref, and object-index entry.
   Start also replaces its reservation with Authority in that root; Attach/Resume replace the
   existing Authority value at the next authority revision. No split embedded/external or
   multi-root success model is permitted. Start/Attach record `startup_ownership=Pending` and
   `post_turn=NotApplicable`; Resume records `startup_ownership=NotApplicable` and exact post-turn
   run/revision `Pending`.
5. Core Applied result is immutable. Exact intent/issuer/payload retry joins it despite an older
   read revision and never reallocates target, appends lineage, rewrites binding, or repeats the
   authority revision. Current revisions remain mandatory for mutations.
6. A valid stored intent whose authority precondition becomes invalid before application, while
   the root/object/key set still verifies, commits one `TerminalHandoff`, `Rejected`, and any
   pending-input `TerminalWithoutAcceptance` in one root revision. Start additionally becomes its
   permanent tombstone. Rejected Attach/Resume may be followed only by a new intent against current
   authority; rejected Start permits only exact terminal join because its ID is burned. Missing or
   mismatched committed objects/keys instead take the non-mutating corruption path above.
7. Issued may expire at fixed `expires_at`; Claimed may expire only after journal, namespace, and
   exact resulting authority prove no application. Expiry atomically records terminal handoff and
   pending-input terminalization; Start also becomes its permanent tombstone. Any application
   marker/result requires reconciliation to Applied instead.
8. Stale revisions, substituted plan, wrong kind/schema/key/domain/commitment, wrong
   session/caller/source/target/lineage/binding, superseded claim, and conflicting replay fail
   closed before authority mutation. Exact read-only join is the only old-revision exception.
9. Plan removal, helper loss, EOF, timeout, or process death changes transport evidence only. It
   cannot alter reservation, intent, authority, retention, or an applied result.
10. Input acceptance CASes Pending to Accepted with an `InputAcceptanceHashInputV1` object bound to
    exact intent/run/input ref/participant. Exact duplicate joins. For an applied Resume, definitive
    terminal failure before acceptance must first publish and apply the exact `TerminalFailure`
    post-turn completion/result pair; only its resulting terminal handoff may set
    `TerminalWithoutAcceptance`. Missing/stale/reordered/mismatched evidence cannot change the
    substate.
11. Load/remove-before-use retries reproject from verified lease/input/transport refs through the
    existing builders. Applied retry may reproject only remaining handoff and never reapplies
    authority. Accepted or terminalized input is never redelivered.
12. Applied Start/Attach resolve startup ownership through one revision-CAS operation owned by
    `HostSessionAuthority`. The target authoritative participant's startup protocol produces
    `OwnershipAccepted` only after accepting ownership of the exact applied session/intent/run,
    application result, participant, and authority revision. The launch/application protocol
    produces `RuntimeCreationRejected` only as an exact typed rejection before participant
    ownership, and the target startup protocol produces `StartupFailedBeforeOwnership` only as an
    exact typed failure causally before any ownership acknowledgement. The protocol actor is
    closed: `OwnershipAccepted` and `StartupFailedBeforeOwnership` require
    `TargetAuthoritativeParticipant` with participant ID exactly equal to the evidence's active
    participant; `RuntimeCreationRejected` requires `LaunchApplicationClaimant` whose claim and
    claimant-attempt IDs exactly equal both the evidence and the persisted Applied intent. Every
    other actor/event pairing fails closed.
    `HostExecutionEpisode` owns observation reporting and A1.3's bounded real helper/REPL adapter
    transports that exact protocol event into one closed `HostStartupOwnershipEvidenceV1`; neither
    may relabel local observations into a protocol event. `HostSessionAuthority`, not the episode
    record, durably captures the submitted actor/event inside the committed
    `StartupOwnershipResultHashInputV1`. The plan and `SurfaceAdapter` only transport it. The evidence binds exact
    store, session, intent, claim, claimant attempt, run, application ref, expected authority
    revision, active participant, event identifier, and observation time. Its `evidence_id` must
    equal the identifier carried by its protocol-event variant and the stored startup substate's
    `evidence_id` must equal the result evidence ID. `OwnershipAccepted` publishes
    `StartupOwnershipResultHashInputV1` with unchanged current authority revision and mutates only
    the intent revision/substate. `RuntimeCreationRejected` and `StartupFailedBeforeOwnership`
    publish `TerminalReconciled` with the exactly corresponding reason; an event/outcome/reason
    mismatch fails closed. Running state, readiness, endpoint publication, PID/process/helper/socket/
    handle posture, timeout, EOF, local adapter error, plan loss, or ambiguity cannot construct any
    of these protocol events and leaves startup ownership Pending.
    A definitive terminal result atomically advances authority once without restoring a
    pre-application snapshot: Start deterministically becomes `Terminal`; Attach deterministically
    becomes `DetachedReconciled`. Both preserve session identity, workspace/store/world binding,
    lineage, policy, descriptor, and active successor identity. The same root revision terminalizes
    any Pending Start input to `TerminalWithoutAcceptance` and commits the exact terminal handoff;
    crash/retry joins all three results or none. Exact duplicate evidence joins; a
    conflicting evidence ID/protocol event, claimant, stale revision, substituted application, or
    second authority advance fails closed.
    The sole narrow old-revision exception is an A1.2a Start whose startup substate remains Pending
    while R0 registers retained targets before A1.2b runs. The evidence still names the exact
    original application revision. HostSessionAuthority must prove a unique contiguous sequence of
    `RetainedWorkerAuthorityRegistrationV1` records from that application hash/revision to the
    exact current authority hash/revision. Every link must preserve active caller, posture,
    workspace/store, world, policy, origin, attach ref, internal resume refs, and all existing
    lineage/retained refs, adding only its committed participant/ref. A gap, fork, ambiguity,
    non-registration authority mutation, reordered commitment, or mismatch fails as stale.
    `OwnershipAccepted` then mutates only the intent startup substate and leaves the current
    registration-descendant authority unchanged. `TerminalReconciled` CASes from that exact current
    descendant, preserves every registered lineage member/ref and supporting object, and changes
    only the already-specified terminal posture/revision fields; it never restores the original
    application snapshot. The same atomic root commit adds the one immutable
    `StartupOwnershipTerminalApplicationJournalV1` phase referencing the exact terminal
    `StartupOwnershipResult`, before/after revisions and commitments, posture, and evidence ID.
    That phase—not the evidence object by itself—is the current-authority proof for the terminal
    revision. Exact retry joins the same ancestry, phase, and result.
    This CAS increments `intent_revision` exactly once. A1.2 owns this internal decision protocol;
    A1.3 owns invocation by the real helper/REPL consumer, and A2 later generalizes episode
    observations without weakening these A1 commitments.
13. Transport stays `Retained`/object-index `Present` until one exact committed terminal handoff,
    using strict V1/V2 dispatch from the matrix above, proves release. Input-bearing modes require Accepted or
    TerminalWithoutAcceptance; input-free Start/Attach require NotApplicable. Applied Start/Attach
    with `startup_ownership=Pending` remain retained and cannot enter release. Before release they
    require `Accepted` or `TerminalReconciled`, and the V2 terminal handoff carries that exact
    startup-ownership result ref. Every applied Resume V1 terminal handoff requires
    post-turn Applied with the exact completion/result pair; `TerminalClean` and `TerminalFailure`
    carry no obligation snapshot, while `ResumableClean` carries the exact Complete ledger snapshot.
    Rejected/Expired require proof of no application.
    One root revision records `ReleaseEligible` in both parent and object index, deletion retries
    idempotently, and a later root records `Released`. A missing plan is never evidence. Exact
    issue/application/expiry retry verifies transport bytes only for Retained/Present; for
    ReleaseEligible or Released it verifies the exact matching parent/index retention state and
    terminal handoff without recursively requiring deleted bytes. A mismatched handoff/index or
    surviving Released bytes not durably removed as specified by the crash rules fails closed.
14. Resume completion first publishes/verifies one immutable
    `PostTurnProtocolEventHashInputV1` object and then a `PostTurnCompletionHashInputV1` whose exact
    `PostTurnProtocolEvent` ref, intent/claim/claimant-attempt/run/revision, B1 acceptance
    ID/revision and accepted work identity, B0 stream, the exact owner-supplied host-transition correlation,
    terminal event ID, and monotonically ordered run-local terminal event sequence all match that
    object and the persisted Applied intent. The protocol-event/completion `run_id` equals the
    correlation's `transition_run_id` and remains distinct from the accepted active-run ID.
    `ResumableClean` and `TerminalClean`
    require a `TargetAuthoritativeParticipant` actor equal to the active participant, the exactly
    matching event kind, and Accepted input. `TerminalFailure` with reason
    `ResumeRuntimeCreationRejected` requires the `LaunchApplicationClaimant` actor to equal the
    applied claim/claimant attempt and may occur only before input acceptance. `TerminalFailure`
    with reason `TargetFailedBeforeInputAcceptance` or `TargetFailedAfterInputAcceptance` requires
    the exact active participant actor and respectively Pending or Accepted input. The completion
    outcome must equal the protocol-event kind; actor,
    reason, ID, sequence, scope, or ref mismatch fails closed. PID/helper/socket/handle/process
    posture, readiness, timeout, EOF, stream loss, or a local adapter error is not a protocol event
    and cannot construct completion. Terminal clean or failure atomically advances authority to
    `Terminal`, applies the post-turn result, terminalizes Pending input only for the two exact
    pre-acceptance failure reasons, and commits terminal handoff without an obligation snapshot.
    `ResumableClean` requires Accepted input and instead commits `AwaitingObligationCut` with the
    completion ref, exact acceptance/stream/transition correlation, and required event cut but does
    not mutate authority. Exact retry joins the same event/completion/pending or terminal result,
    initial intent expiry cannot erase it, and transport remains Retained until its exact release
    gate.
15. `ObligationLedger` alone owns the semantic query and produces
    `ObligationLedgerSnapshotReadV1`; `HostSessionAuthority` is a consume-only client. A Complete
    snapshot binds exact authority store, session, active participant, B1 acceptance record
    ID/revision and accepted task/active-run identity, B0 stream ID, the unchanged verified
    transition intent/revision/payload commitment and distinct transition run, observed authority
    revision, per-session ledger revision, terminal event ID/sequence, and a
    `materialized_through_event_sequence` at least that terminal sequence. The materialization cut
    repeats the acceptance record and stream identity; all repeated fields must match exactly. The
    snapshot also carries every post-acknowledgement retained `Event` journal ref through that
    watermark as a typed B3.1 member, ordered by event sequence with unique journal/frame/event
    identities and full canonical event commitments. It is exhaustively joined to B2.1's journal;
    any untyped, omitted, substituted, or scope-mismatched retained event keeps the cut
    non-Complete even when the attention disposition would otherwise be empty. Revisions/sequences
    and all identities are nonzero/non-empty as
    applicable. Each unresolved entry binds obligation ID/revision and the exact `CanonicalSha256` commitment of
    `ObligationSnapshotRecordHashInputV1`; entries are non-empty unique IDs sorted by raw UTF-8 byte
    order. `ObligationLedger` sets the closed attention disposition and guarantees it matches the
    complete canonical record set: NoUnresolvedAttention requires an empty entry vector and
    HasUnresolvedAttention requires a non-empty vector. It derives only from canonical obligation
    records, never inbox rows, counts, worker flags, helper state, or compatibility projections.
16. A1.2 may publish the Complete snapshot bytes unchanged as `ObligationSnapshot` and validate
    only their closed schema, exact acceptance/stream/transition scope, cut/ref commitments, and
    equality with the current pending completion and authority. It cannot invent or repair the
    correlation, equate transition run with active run, or enumerate, classify, create, resolve,
    reinterpret, repair, or overwrite obligations. The ledger revalidates the same per-session
    revision and event cut immediately before the authority commit under the retained
    transaction/lock; the lock supplies physical serialization but not semantic ownership. A
    Pending ledger read leaves
    `AwaitingObligationCut` unchanged. A Complete `HasUnresolvedAttention` result selects
    `AwaitingAttention`; Complete `NoUnresolvedAttention` selects `ParkedResumable`. One root
    transaction advances post-turn to Applied, mutates authority at most once, and stores the
    completion/snapshot/result refs in intent state and `post_turn_application` journal. Exact
    duplicate joins; stale, reordered, mismatched, or conflicting completion/cut fails closed.
17. C1 owns the event-to-obligation materializer, canonical obligation revisions, and the complete
    per-session event cut required above. The current pre-C1 ledger cannot provide that semantic
    completeness proof: empty can mean either no unresolved obligation or not-yet-materialized
    events. C1 in turn requires B0 runtime identity/order, B1 accepted run identity, B2.1 durable
    observation/reconciliation, and B3.1 exact retained-event semantics. Therefore A1.2 can specify
    and persist the pending protocol but cannot close a ResumableClean post-turn or claim its full
    packet exit before that corridor lands. A1.2 does not implement any corridor owner or claim
    `RG-EVENT-01`, `RG-SUP-01`, `RG-SUP-02`, `RG-MSG-01`, `RG-OBL-01`, or `RG-OBL-02`.

### Crash reconciliation

On process restart or before retrying a nonterminal intent, `HostSessionAuthority` reconciles in this order:

1. Re-run the bootstrap classifier under the lock. Resume only an exact `InitializationPending`
   marker or a `ValidExisting` root; every invalid, partial, permission-invalid, unreadable, or
   unsupported root state fails closed and never initializes a replacement namespace.
2. Strictly decode the last verifiable root under lock and verify store ID and bound bootstrap home,
   the exactly-one-active key registry invariant, every required active/verification key envelope,
   the immutable greenfield certificate, empty pre-A1 authority collections, complete
   namespace/intent/index/journal invariants, and
   every reachable ref whose storage state requires bytes. A Released transport validates through
   its exact terminal handoff instead. Root uncertainty never falls back to external object or key
   presence.
3. `Issued` Start must have been created by A1.2a after verifying the greenfield certificate and
   empty pre-A1 authority collections and must own its exact reservation; Issued non-Start must
   retain its exact authority precondition. With no application marker it remains claimable until
   expiry. Missing/mismatched self reservation is corruption, not absence.
4. `Claimed` with unchanged precondition may resume under the exact claim or be reclaimed after
   lease expiry. No liveness observation substitutes.
5. Exact authority mutation plus journal/result ref commits Applied even if the caller saw failure;
   reconciliation recovers/joins the identical result. For Start, Authority origin must match the
   former reservation. A visible Authority without matching application proof fails closed.
6. Objects with no root parent are orphans and never prove issuance/application. A missing or
   mismatched object required by Retained/Present state is corruption. ReleaseEligible plus absent
   transport may advance only after exact terminal-handoff verification. An Applied V2 Start or V3 Attach
   whose startup ownership is `Pending` verifies only its exact expected run, original application
   revision, and active participant tuple and has no evidence/result ref. `Accepted` or
   `TerminalReconciled` additionally requires and verifies the exact evidence/result ref. Strict V1
   Applied remains read-only and does not acquire a synthesized startup substate. An AwaitingObligationCut Resume verifies its
   exact post-turn protocol-event ref, completion/event equality, acceptance/stream/transition
   correlation, and cut scope and remains pending until the semantic owner returns Complete;
   applied resumable post-turn Resume verifies the same event/completion chain, its exact Complete
   obligation snapshot ref, and current ledger revalidation.
7. Missing plan transport is reprojected only after revisioned input/application/startup-ownership/
   post-turn checks.
   Applied Resume with pending post-turn work accepts only the exact actor-bound protocol-event ref,
   matching completion/result, closed input state, and exact ledger cut when required for its
   committed run; ambiguous or incomplete evidence remains pending and diagnosable.
8. If root-directory fsync previously failed, report no inferred success. Reconcile whichever
   complete root revision and object set is verifiable, then use issuer index/application journal
   for exact retry; never choose the newest-looking orphan or temp file.
9. If neither prior precondition nor exact committed result can be proven—or any referenced HMAC
   key is lost—fail closed and retain diagnosable state without logging sensitive bytes or
   commitments.
10. Reconciliation, greenfield-certificate validation, key rotation/retirement, tombstoning, and
   payload cleanup are idempotent across repeated crashes and
   cannot increment initial or post-turn authority more than once per intent.

### Greenfield-only activation and unsupported pre-A1 state

A1 intentionally defines no legacy-authority migration or compatibility-adoption protocol. There
are no deployed users or valid pre-A1 authority records to preserve. Adding conversion,
quarantine, evidence, conflict arbitration, or dual-write semantics would create an unneeded
permanent authority surface. The generic StateStore migration/schema-evolution and
CompatibilityReadModel ownership in `01-target-architecture.md` remains available for other
schemas and read-only diagnostics; it does not authorize importing pre-A1 session/participant
records into A1 authority.

The closed rules are:

1. The bootstrap classifier enumerates the two exact pre-A1 authority collections defined above.
   A complete empty result is one prerequisite for `FreshAbsent`, `InitializationPending`,
   `ValidExisting`, and every semantic transaction.
2. Any safely observed artifact produces `UnsupportedLegacyState`. Multiple, contradictory,
   same-named, or differently shaped artifacts have the same single result; A1 does not parse them,
   create evidence, select a winner, or derive a session namespace.
3. Any unreadable, permission-invalid, symlinked, partially enumerable, or otherwise uncertain
   collection is `CorruptOrUnsupported`. A missing or unreadable individual record and an empty
   compatibility projection never imply namespace absence.
4. Once the greenfield certificate exists, every pre-A1 session/participant authority-write API is
   disabled for that store and must fail before touching either old or A1 state. Every A1 root
   transaction independently revalidates that the old collections remain empty, so an older or
   external writer cannot create state behind the certificate.
5. A1.1 routes every in-repository pre-A1 session/participant writer through the same root lock
   before activation. Such a writer may proceed only while neither `init-v1.json` nor
   `state-root-v1.json` exists; after marker publication it rejects without touching state. The
   initializer holds that lock from marker publication through the final empty-collection scan and
   root publication. Concurrent unmodified/mixed-version binaries are unsupported and must be
   quiesced; A1 never claims safety from process-liveness observation.
6. Pre-A1 helper plans without `intent_id` and `payload_commitment` cannot drive an A1 transition.
   Once a producer is switched, a mixed-version consumer fails closed and requires a newly issued
   valid intent; it never reconstructs authority from a plan or compatibility view.
7. A1 never deletes unsupported artifacts or offers a runtime reset flag. Because this is a
   greenfield contract, recovery is an explicit operator/developer cleanup outside the running A1
   authority protocol, followed by a new complete bootstrap classification. Diagnostics may report
   only bounded relative path and entry-type metadata; they never read or log artifact contents.

Helper endpoints/paths and auto-attach policy, eligibility, claim, and settlement remain unchanged.


## 2. `HostExecutionEpisodeV1`

```rust
struct HostExecutionEpisodeV1 {
    schema_version: u32,                 // exactly 1
    episode_id: String,
    kind: HostExecutionEpisodeKindV1,
    orchestration_session_id: String,
    observed_authority_revision: u64,
    backend_id: Option<String>,
    process_ref: Option<ProcessRefV1>,
    transport_status: HostExecutionEpisodeTransportStatusV1,
    started_at: Timestamp,
    last_heartbeat_at: Option<Timestamp>,
    ended_at: Option<Timestamp>,
    exit_observation: Option<EpisodeExitObservationV1>,
}
```

Episode kinds:

```text
ReplAttachedEpisode
HiddenOwnerHelperStartEpisode
HiddenOwnerHelperAttachEpisode
HiddenOwnerHelperResumeOneTurnEpisode
RuntimeToolboxEpisode
SyntheticOrRecoveredEpisode
```

Transport status:

```text
Available
UnavailableButDurableAuthorityExists
UnavailableAndNoAuthoritativeRoute
StaleOrOrphaned
```

Rules:

1. Episodes submit observations and requested transitions to `HostSessionAuthority`; they never write durable posture directly.
2. An observation whose `observed_authority_revision` is stale cannot mutate authority.
3. Episode exit does not delete session, worker, binding, receipt, or obligation truth.
4. Private transport success may accelerate delivery; it does not define durable success.
5. PID, process/helper presence, active handles, readiness, and prompt-stream state are episode or
   transport observations only; their absence does not erase `ParkedResumable` authority.
6. Episode construction and launch follow durable transition application and cannot reset a parked
   session to `Allocating` or authorize a successor participant.

## 2A. Runtime event identity and ordering carrier

B0 extends the existing runtime stream family; it does not create a parallel transport. The
runtime producer assigns identity before emission. Host decoders, receipt code, supervisors,
messaging code, and the ledger consume those fields unchanged.

```rust
struct RuntimeFrameIdentityV1 {
    schema_version: u32,          // exactly 1
    stream_id: String,            // stable for one accepted runtime stream
    frame_sequence: u64,          // starts at 1; strictly monotonic and gap-free
}

struct RuntimeEventIdentityV1 {
    event_id: String,             // stable for one logical semantic event
    event_sequence: u64,          // starts at 1; strictly monotonic within stream
}

struct RuntimeTerminalIdentityV1 {
    terminal_event_id: String,
    terminal_event_sequence: u64,
}
```

Every `ExecuteStreamFrame` carries `RuntimeFrameIdentityV1`. Semantic `Event` frames additionally
carry `RuntimeEventIdentityV1`; the terminal `Exit` frame carries both a semantic event identity
and `RuntimeTerminalIdentityV1` with equal ID/sequence. `Start`, stdout, stderr, event, and terminal
frames participate in the same frame sequence. `Error` is either a typed semantic terminal frame
with exact terminal identity or an observation/transport error that cannot prove run completion.

B0 rules:

1. `stream_id`, frame sequence, event ID/sequence, and terminal identity are runtime-produced;
   the host never synthesizes them from arrival order, timestamps, payload hashes, span IDs, EOF,
   or process state.
2. Re-emitting the same logical frame/event after retry or reconnect preserves every identity.
   The producer never originates a second meaning at the same sequence position.
3. Frame and event sequences are positive and strictly monotonic. An exact identity with identical
   canonical bytes is valid replay and B2.1 consumes it as a no-op. A gap, reorder, conflicting
   duplicate, or frame after terminal is protocol-invalid and B2.1 rejects it; that consumer-side
   enforcement is not B0 ownership.
4. The terminal frame is the final semantic event and names its exact event ID/sequence. Stream
   exhaustion without it is not completion.
5. B0 owns only the carrier. B1 owns acceptance, B2.1 owns durable observation/dedupe/restart,
   B3.1 owns retained-event semantics, and C1 owns obligation materialization/completeness.

### B2.1-3 exact producer replay transport

World-service may add one bounded process-memory registry and endpoint for exact B0 frame replay.
The registry is a producer transport cache, not durable state and not a supervisor. A replay request
must supply all of:

```rust
struct ExecuteStreamReplayRequestV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    stream_id: String,
    after_frame_sequence: u64,
}
```

The registry binds that request to the exact accepted producer stream. It returns only frames with
the same `stream_id` and `frame_sequence > after_frame_sequence`, in strict sequence, with the
original identity and canonical bytes, and may then continue that exact live stream. It cannot
enumerate streams or accept lookup by session, backend, process, endpoint, partial identity, or any
fuzzy match. The implementation must set explicit maximum retained streams, maximum frames per
stream, and maximum bytes per stream. Overflow, missing or expired retention, cursor-ahead state,
identity mismatch, corruption, reorder, conflict, and unavailable replay fail closed.

The replay registry is lost on world-service restart unless a later separately authorized durable
producer contract proves otherwise. It must not be described as durable across that restart. It
cannot create or modify B1 acceptance, a supervisor claim/cursor/journal, a terminal result,
cancellation, lifecycle state, an obligation or Complete cut, or success. ReceiptRegistry remains
immutable acceptance authority; `RuntimeEventTransport` transports producer facts only.

## 2B. Bounded retained worker event envelope

B3.1 supplies the minimum semantic envelope C1 needs without moving B3.2 early. It extends the
existing shared `AgentEvent` in `crates/common/src/agent_events.rs` with one optional, explicitly
typed top-level `worker_event` field. The same module defines the producer-construction-only
`NormalizedWorldWorkerEventFacetV1`. In `world-service/member_runtime.rs`, one named fail-closed
normalizer parses the existing provider `AgentWrapperEvent` kind/payload before `AgentEvent`
construction and produces this facet; it is the sole permitted provider-payload interpretation for
the C1 path. `ExecuteStreamFrame::Event { event: AgentEvent }` remains unchanged, so
`world-service` can produce the envelope and shell consumers can validate it without host-side
identity or semantic inference from `AgentEvent.data` or a second transport protocol.
Both the equality-only commitment representation and its host-transition correlation carrier live
in `crates/common/src/authority_commitment.rs` and are re-exported by
`crates/common/src/lib.rs`. They contain no verifier or semantic authority:

```rust
enum OpaqueAuthorityCommitmentV1 {
    CanonicalSha256 {
        digest_hex: String,
    },
    StoreHmacSha256 {
        key_id: String,
        domain: String,
        digest_hex: String,
    },
}

struct HostTransitionWorkCorrelationV1 {
    schema_version: u32,          // exactly 1
    authority_store_id: String,
    orchestration_session_id: String,
    authoritative_participant_id: String,
    transition_intent_id: String,
    transition_intent_revision_observed: u64,
    transition_run_id: String,
    transition_payload_commitment: OpaqueAuthorityCommitmentV1,
    authority_revision_observed: u64,
}

enum WorldWorkerEventClassV1 {
    Reply,
    ProgressUpdate,
    FollowUpQuestion,
    ApprovalRequest,
    Blocked,
    AttentionRequired,
    ForkRequest,
    ForkRecommendation,
    Result,
    Failure,
}

// Producer-construction type, not an independent wire envelope.
struct NormalizedWorldWorkerEventFacetV1 {
    schema_version: u32,          // exactly 1
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    causation_message_id: String,
    causation_request_id: String,
    payload: serde_json::Value,
}

struct WorldWorkerEventV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    stream_id: String,
    frame_sequence: u64,
    event_id: String,
    event_sequence: u64,
    request_id: String,
    active_run_id: String,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    causation_message_id: String,
    causation_request_id: String,
    orchestration_session_id: String,
    source_participant_id: String,
    target_participant_id: String,
    source_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    thread_id: String,
    event_class: WorldWorkerEventClassV1,
    attention_required: bool,
    payload: serde_json::Value,
    emitted_at: Timestamp,
}

struct AgentEvent {
    // All existing legacy fields remain unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    worker_event: Option<WorldWorkerEventV1>,
}
```

The exact B0 event identity is copied unchanged. `acceptance_record_id` and `active_run_id` join the
B1 accepted retained turn. When B1 carries `host_transition_correlation`, B3.1 copies it unchanged;
B3.1 rejects absence or mismatch rather than inferring intent/run/authority scope from request ID,
active-run ID, foreground state, or payload. Before `AgentEvent` construction, the producer
normalizer maps every provider wrapper event into exact thread/class/attention/payload semantics
and joins request/message causation from the retained B1 context. Explicit supported non-attention
wrapper shapes receive a typed non-attention class. Unknown, ambiguous, deferred, or malformed
shapes fail the retained stream and C1 cut closed; they cannot be dropped, left untyped, or
downgraded to progress or no-attention. The final source/target, world, run, thread, class,
attention, and causation fields are explicit and validated before ledger delivery; the resulting
B2.1 generic journal ref commits the exact canonical shared-envelope bytes without the supervisor
interpreting them.

After emission, neither host code nor C1 may infer a missing field from free text,
`AgentEvent.data`, optional nested runtime payloads, the most recent worker, or foreground request
state. Raw provider payload may remain as compatibility/debug data, but it is not semantic truth
for the C1 path. B3.1 does not require an upstream provider-wrapper taxonomy change and does not
complete host-to-worker messaging, retained lifecycle, park, cancel, stop, fork, or model-facing
early return; those remain B3.2/B4.

`worker_event` may be absent only for streams outside the B3.1-adopted accepted-retained scope. In
that scope, every post-acknowledgement `ExecuteStreamFrame::Event` must carry it through the exact
terminal cut; absence or invalidity is a protocol failure and makes a Complete C1 snapshot
unavailable. Every identity/correlation/semantic field comes from this typed member rather than
host parsing of `AgentEvent.data`. The normalizer may read provider `AgentWrapperEvent.data` only
before emission, under its explicit exhaustive provider-shape tests, to construct
`NormalizedWorldWorkerEventFacetV1`.
`OpaqueAuthorityCommitmentV1` preserves the exact variant and fields of the HostSessionAuthority
commitment supplied later by A1.2b, but B1/B3.1/C1 may only retain and compare it; they cannot mint,
verify, reinterpret, log as observability evidence, or use it as obligation semantics.

### B1/B2.1 bounded read-only dispatch-authority adapter

The implementability audit selected **Case B**. The adapter became implementation-authorized after
the independently review-clean docs-only control-pack correction and the review-clean A1.2a,
A1.2a-WB, A1.2a-S, B1/B2.1-R0, B3.2a, B3.2a-WA, B1 receipt-core, and B2.1 supervisor-core
prerequisites were present. Its implementation is part of B1/B2.1-0, not the later joint closeout.
A1.1e alone could read exact current authority but could not create it; A1.2a,
A1.2a-WB, and A1.2a-S now satisfy the bounded ordinary-internal-host creator/adopter portion of
that sequence, B1/B2.1-R0 is independently review-clean, and B3.2a plus B3.2a-WA are independently
review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The recovered B1/B2.1 cores
and B1/B2.1-0 implementation are review-clean through `6436289f`, `c519024b`, `de727091`,
`717579b0`, and `83101dcb`. The joint production closeout remains open. The
historical prepared type also combined
B-owned accepted/inspection routing with retained-worker admission data that has no canonical
live-state representation.

The authority/retained branch is **A1.2a → A1.2a-WB → A1.2a-S → B1/B2.1-R0 → B3.2a → B3.2a-WA**. The independently
recovered B1 receipt → B2.1 supervisor branch first joins it at **B1/B2.1-0**, after which the hard
order is **joint B1/B2.1 production closeout → B3.1 → C1 → A1.2b**. R0 consumes no receipt or
supervisor datum; neither B core is therefore a false prerequisite of R0.

`tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1` is a read-only adapter and
owns no durable state. For its active-task branch only, it consumes the existing bound authority
capability or the single authorized trusted open-and-bind conversion and resolves:

1. the exact current HostSessionAuthority;
2. the exact immutable `WorldWorkAcceptanceRecordV1`; and
3. the exact `WorldWorkExecutionSupervisor` claim, durable cursor, interruption/unresolved state,
   and immutable terminal closeout when present.

It accepts only one complete equality join across physical store ID and home, orchestration
session, caller participant and backend, world ID and generation, task/active-run identity,
acceptance record identity, and stream identity where applicable. It returns only a bounded
projection into the existing tool-invocation result and performs no mutation. Absent, stale,
ambiguous, cross-session, backend-mismatched, world-mismatched, task-reused, incomplete, or
conflicting truth fails closed. Unknown task, stale linkage, caller/backend mismatch, world-binding
mismatch, nonterminal active work, exact terminal work, and producer-replay-unavailable or otherwise
unresolved observation remain distinct existing outcomes; they cannot be collapsed into generic
not-found or stale linkage.

The adapter cannot create or mutate authority, acceptance, supervisor state, or any legacy active-
task record; treat receipt existence alone as active, running, or terminal truth; derive lifecycle
from PID, guard, waiter, helper, socket, timeout, EOF, readiness, process state, or caller presence;
reinterpret retained work as an ephemeral active task; or bypass the full dispatcher. Its retained-
worker branch, retained target selection, retained error categories, and retained Inspect/Cancel/
Stop, continue-fork, fork, and Spawn behavior remain unchanged. Its function signature and callers
remain unchanged. The existing shared pre-`match` legacy caller/world resolution may be
mechanically relocated into the retained-worker branch with identical inputs, order, errors, and
results so the active-task branch no longer executes it. That partitioning grants no retained
semantic change and no parallel resolver. The approved GitNexus HIGH impact is exactly 12 direct callers, four affected
execution processes, and 14 impacted symbols for this active-task branch. Any additional production
symbol, signature/caller change, retained-branch change, or new HIGH/CRITICAL process family is a
stop requiring a new control-pack correction.

The minimal prerequisite contract is:

1. **A1.2a — current-authority establishment/read prerequisite:** use the production
   HostSessionAuthority protocol to perform the strict greenfield-only V1-to-V2 root upgrade, then
   verify greenfield absence, issue/claim/apply one Start, create initial authority, and exact-join
   retry. Its read result joins the applied Start descriptor and accepted-home bound capability to
   exact session/caller/lineage/workspace/world/revision/policy truth. It does not implement a
   non-greenfield upgrade, Attach, ResumeOneTurn, startup/post-turn reconciliation, obligation-cut
   consumption, correlation supply to world work, or public consumer adoption.
2. **A1.2a-WB — Host/world-binding validation correction:** preserve exact descriptor/launch-scope
   equality, accept `Host + None`, `Host + Some(exact)`, and `World + Some(exact)`, and reject
   `World + None` or any malformed binding without mutation during Start issuance. Application
   persists the exact accepted binding, and `HostSessionAuthority::resolve_current_exact` enforces
   the identical matrix and returns that binding unchanged. Exact present binding is session
   authority, never host-participant placement. The packet changes no schema, canonical bytes,
   fixtures, migration, compatibility path, or already-persisted object; all other facade behavior
   remains outside scope.
3. **A1.2a-S — bounded internal Start adoption prerequisite:** make
   `prepare_host_orchestrator_runtime_from_resolved` construct only an unpersisted
   `GreenfieldHostStartProposalV1` carrying exact store/home/workspace/descriptor/policy/shell
   observations but no authoritative session/participant/run identity. The proposal is a distinct
   type, never a variant or partially initialized form of `PreparedAgentRuntime`. On the real
   dormant-host launch path, `dispatch_targeted_follow_up_turn` first obtains the exact optional
   world binding, then calls `apply_greenfield_host_start_from_authority`; that adapter derives the
   A1.2a issuer key from the greenfield store identity plus the complete canonical Start plan and,
   before transport or any legacy write, applies/exact-joins A1.2a Start. Only that result supplies
   identities and constructs one fully materialized existing-shape `PreparedAgentRuntime` plus its
   compatibility session/participant view. `PreparedAgentRuntime` itself, its fork/member
   constructors, and its remote-member consumers do not acquire a pending variant or change
   semantics. The new runtime carries the exact bound capability in an immutable shared
   `RuntimeAuthorityContext::Bound` inside `RuntimeOrchestrationContext`; no context exists before
   application. The hidden-owner constructor may initialize only the `Legacy` variant and receives
   no adoption semantics. The authority-managed branch of
   `start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx` performs no
   activated-store legacy session/participant/snapshot write; it leaves startup ownership Pending
   and treats launch, readiness, endpoint, process, prompt, and event state as observations. It does
   not adopt hidden-owner plans, public Start/Attach/Resume, startup-result reconciliation, or
   post-turn behavior.
4. **B1/B2.1-R0 — canonical retained-target protocol prerequisite:** add the smallest
   bounded registration handshake in which RetainedWorkerRuntime creates the immutable descriptor,
   participant-specific resume handle, and retained-worker object graph from a caller-fixed
   participant plan. HostSessionAuthority validates that plan and its exact commitment/scope and
   atomically appends exactly the retained participant to
   authoritative lineage, adds its typed worker ref, advances authority revision, and persists the
   non-transition `RetainedWorkerAuthorityRegistrationV1` proof consumed by `resolve_exact`.
   Exact retry joins; stale or conflicting revision fails closed. Registration cannot change
   active caller, posture, workspace, world, current policy, origin, or any unrelated ref. R0 has
   no production ingress caller and cannot itself count as full-dispatcher proof. It cannot use a
   test-only fixture or process-local map, and it adds no message, accepted-turn observation, active-turn,
   park/cancel/stop/fork, or live-admission semantics.
5. **B3.2a — retained creation/admission bridge prerequisite:** route both real Spawn adapters—the
   direct dispatcher path and the live internal-toolbox retained-runtime path—through the same
   authority-bound `SpawnWorldWorker` preparation before generic compatibility
   preparation, atomically count/reserve a durable participant slot across processes before R0,
   pass that fixed participant plan to R0, exact-join it before transport, commit the final proof
   plus the same fixed RetainedWorkerRuntime admission record, and
   make the remote retained-start path validate the exact
   session/request/participant/backend/protocol/world proof rather than invoke any activated-store
   legacy session/participant writer. The admission record is nonterminal before transport,
   routable only after exact Registered truth, and terminal only after exact B0 terminal truth.
   Every ambiguous interruption remains nonterminal and counted live, so existing spawn admission
   narrows without HSA-ref counting or compatibility liveness. B3.2a has no accepted-turn,
   message, park/cancel/stop/fork, abandonment protocol, or final lifecycle semantics. A queued
   `SlotReserved` record with no head is valid; head acquisition requires complete exact
   re-presentation of the earliest request, while current-head reconciliation/release never
   promotes another slot and persists no request/prompt/payload preimage.
6. **B3.2a-WA — exact bound-world ownership adoption prerequisite:** after authority-managed proof
   validation and before member creation, exact-join the HSA session/world ID/generation/policy,
   participant, project/world-spec identity, and current generic-world metadata. The runtime-family/
   Linux backend alone may durably publish `GenericExactBoundWorld ->
   SharedSessionOwnerExactBoundWorld`, preserving world ID and generation. Exact retry joins without
   rewrite; conflicts, missing/corrupt/ambiguous metadata, or partial publication fail closed with no
   alternate world. This changes no HSA, RetainedWorkerRuntime, transport-claim, routability, or
   terminal truth and proves no launch. Compatibility `None` stays unchanged. It adds no world-api
   field, schema version, side table, shell rebinding, request/prompt persistence, PID/liveness
   authority, recovery protocol, or non-Linux proof.
7. **B1/B2.1-0 — action-scoped dispatch preparation:** accept the A1.2a/A1.2a-WB/A1.2a-S,
   B1/B2.1-R0, B3.2a, and B3.2a-WA typed read results plus B1 receipt and B2.1 supervisor truth through a
   caller-supplied bound capability or one explicitly authorized trusted open-and-bind conversion;
   build a B-owned prepared view only for RunWorldTask, ordinary retained ContinueWorldWorker, and
   ephemeral accepted-task Inspect/Cancel/Wait; and leave `WorkerContinueForkCommand`, retained
   Inspect/Cancel/Stop and fork admission/lifecycle calculation on unchanged compatibility paths.
   It leaves the already-landed B3.2a spawn creation/admission bridge unchanged and does not alter spawn
   policy, steering, outcome, or lifecycle semantics. It neither uses
   nor replaces that legacy count and cannot claim it as authority.
8. After those prerequisites, validate the typed request, require session/caller/world agreement,
   exact-join receipt and supervisor truth for accepted work, and fail closed on any absent,
   incomplete, stale, or conflicting authority/binding/acceptance/claim.

R0's non-transition proof is a distinct HostSessionAuthority record, not an application-journal
entry and not a new transition mode:

```rust
enum RetainedWorkerAuthorityRegistrationRequestStateV1 {
    Reserved,
    Applied {
        authority_revision_after: u64,
        authority_record_commitment_after: AuthorityObjectCommitmentV1,
    },
}

struct RetainedWorkerAuthorityRegistrationRequestV1 {
    schema_version: u32, // exactly 1
    issuer_request_id: String,
    registration_id: String,
    orchestration_session_id: String,
    authority_revision_before: u64,
    authority_record_commitment_before: AuthorityObjectCommitmentV1,
    retained_participant_id: String,
    descriptor_ref_id: String,
    descriptor_commitment: AuthorityObjectCommitmentV1,
    resume_handle_ref_id: String,
    resume_handle_commitment: AuthorityObjectCommitmentV1,
    retained_worker_ref_id: String,
    retained_worker_commitment: AuthorityObjectCommitmentV1,
    current_policy_ref: AuthorityObjectRefV1,
    world_binding: WorldBindingV1,
    registered_at: TimestampV1,
    state: RetainedWorkerAuthorityRegistrationRequestStateV1,
}

struct RetainedWorkerAuthorityRegistrationV1 {
    schema_version: u32, // exactly 1
    issuer_request_id: String,
    registration_id: String,
    orchestration_session_id: String,
    authority_revision_before: u64,
    authority_record_commitment_before: AuthorityObjectCommitmentV1,
    authority_revision_after: u64,
    authority_record_commitment_after: AuthorityObjectCommitmentV1,
    retained_participant_id: String,
    authoritative_lineage_commitment_after: AuthorityObjectCommitmentV1,
    descriptor_ref: AuthorityObjectRefV1,
    resume_handle_ref: AuthorityObjectRefV1,
    retained_worker_ref: AuthorityObjectRefV1,
    current_policy_ref: AuthorityObjectRefV1,
    world_binding: WorldBindingV1,
    registered_at: TimestampV1,
}
```

The R0 caller must carry the existing validated retained-creation ingress request/idempotency ID.
The API domain-separates it with the closed `retained-worker-registration` operation tag before
using it as `issuer_request_id`; this key cannot be reused as a transition issuer key, and R0 may
not create a replacement after entering the operation. Its caller also supplies one exact
participant plan; the production caller is the B3.2a `SlotReserved` record. Before any object file
is published, one root CAS validates that participant plan, the complete proposed canonical object
bytes, and exact scope, then exact-joins or creates its request-index reservation. Under the root
lock it generates/collision-checks the registration and planned object IDs and fixes all three canonical
commitments, expected authority revision/commitment, current policy, world, and `registered_at`.
An exact retry with the same issuer ID and complete bytes joins; reuse with any changed field fails
before object publication. A crash before reservation publication leaves no object or semantic
change and may retry from the same ingress request. A crash after reservation publication rereads
these fixed identities rather than generating replacements.

Only after that reservation is durable may RetainedWorkerRuntime publish the exact descriptor,
resume-handle, and retained-worker bytes under the three reserved IDs. Their existence grants no
authority. The final root CAS verifies every byte/commitment and the unchanged reservation, adds
the three object-index entries, performs the authority lineage/ref mutation, inserts the immutable
registration journal, and advances the request state from `Reserved` to `Applied` atomically.
`StateRootV2.retained_worker_registration_journal` keys the proof by exact `registration_id`.
That final CAS must verify the pre-revision/commitment, current policy and world,
descriptor backend/protocol/world execution scope, participant-specific resume identity, and the
retained object's session/participant/world/descriptor/resume/policy refs; append the participant
exactly once to authoritative lineage; append the worker ref exactly once; increment authority
revision exactly once; and commit the new authority hash plus registration proof. Active caller,
posture, workspace/store, world, policy, origin, attach ref, internal resume refs, and existing
lineage/worker refs remain byte-for-byte unchanged. A reserved object file present before a
losing/crashed final CAS is verified and reused only through its exact Reserved request; without
that reservation it is an ordinary orphan subject to reconciliation. It never becomes authority by
existence alone.

`resolve_exact` and current-authority proof validation accept the unique highest matching proof
from exactly one of: an initial or post-turn transition application phase, a terminal startup
application phase, or this registration journal. An Accepted startup result does not advance
authority and supplies no authority proof; the already-current initial/R0 proof remains current.
For every proof source, before/after revisions and commitments must form the unique contiguous
history for that session, and the highest committed revision must equal the current authority.
Missing, ambiguous, behind, skipped, substituted, or conflicting proof fails closed. A retry after
the final CAS, including a lost response, joins the `Applied` request index to the identical journal
and current authority and returns the original result. Reuse with different bytes, duplicate
participant/ref under a different issuer or registration ID, or a stale expected revision fails.
This operation creates no live/terminal state and cannot satisfy retained admission
counting. When Start startup ownership is still Pending, each successful registration also becomes
one link in the only permitted application-revision-to-current-revision ancestry. It does not
rewrite the Pending expected revision or constitute startup acceptance; A1.2b must later validate
the complete contiguous chain under the startup rule above.

B3.2a adds a separate RetainedWorkerRuntime-owned creation/admission record; it is neither
HostSessionAuthority nor accepted-turn Supervisor truth:

```rust
enum RetainedWorkerAdmissionCommitmentAlgorithmV1 {
    HmacSha256,
}

struct RetainedWorkerAdmissionCommitmentV1 {
    schema_version: u32, // exactly 1
    algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1,
    key_id: String,
    digest_hex: String,
}

struct RetainedWorkerAdmissionRegistrationV1 {
    registration_id: String,
    retained_worker_ref: AuthorityObjectRefV1,
}

enum RetainedWorkerAdmissionStateV1 {
    SlotReserved {
        slot_sequence: u64,
        reserved_at: TimestampV1,
    },
    AuthorityRegistrationHead {
        authority_revision_expected: u64,
        authority_record_commitment_expected: AuthorityObjectCommitmentV1,
        head_acquired_at: TimestampV1,
    },
    PreTransportNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
    },
    TransportClaimedNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        transport_claim_id: String,
        claimed_at: TimestampV1,
    },
    Routable {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: String,
        registered_frame_sequence: u64,
        registered_event_id: String,
        registered_event_sequence: u64,
        registered_at: TimestampV1,
    },
    InterruptedNonterminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: Option<String>,
        last_frame_sequence: Option<u64>,
        interrupted_at: TimestampV1,
    },
    Terminal {
        registration: RetainedWorkerAdmissionRegistrationV1,
        stream_id: String,
        terminal_frame_sequence: u64,
        terminal_event_id: String,
        terminal_event_sequence: u64,
        exit_code: i32,
        terminal_at: TimestampV1,
    },
    RejectedBeforeRegistration {
        reason: String,
        rejected_at: TimestampV1,
    },
}

struct RetainedWorkerAdmissionRecordV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentV1,
    orchestration_session_id: String,
    admission_authority_revision: u64,
    admission_authority_record_commitment: AuthorityObjectCommitmentV1,
    retained_participant_id: String,
    bootstrap_run_id: String,
    backend_id: String,
    protocol: String,
    world_binding: WorldBindingV1,
    current_policy_ref: AuthorityObjectRefV1,
    current_policy_revision: String,
    max_live_retained_workers: u64,
    state: RetainedWorkerAdmissionStateV1,
    record_revision: u64,
}
```

The corrected queued-promotion contract uses this existing V1 schema and the existing
`SlotReserved` state. It adds no schema version, persisted request/preimage object, side table,
secret domain, or lifecycle state merely to encode "waiting for retry." Queued `SlotReserved`
records with no current `AuthorityRegistrationHead` are a valid registry state.

`canonical_spawn_fingerprint` is a RetainedWorkerRuntime-owned, domain-separated keyed HMAC. It
does not use, extend, pin, or participate in the HostSessionAuthority commitment-key registry. Its
exact HMAC input is the following byte sequence, where `len64` is unsigned 64-bit big-endian and
each `canonical_*` member is the exact `CanonicalJsonV1` encoding of the complete strict typed
value named by that member:

```text
"substrate.retained-worker.admission.hmac-input.v1\0"
 || len64("substrate.retained-worker.admission.spawn.v1")
 || "substrate.retained-worker.admission.spawn.v1"
 || len64(authority_store_id) || authority_store_id
 || len64(issuer_request_id) || issuer_request_id
 || len64(canonical_validated_spawn_request) || canonical_validated_spawn_request
 || len64(canonical_exact_current_authority) || canonical_exact_current_authority
 || len64(canonical_descriptor_and_runtime_plan) || canonical_descriptor_and_runtime_plan
 || len64(canonical_policy_and_admission_cap) || canonical_policy_and_admission_cap
 || len64(retained_participant_id) || retained_participant_id
 || len64(bootstrap_run_id) || bootstrap_run_id
```

The first canonical member is the complete validated `SpawnWorldWorker` request, including the
prompt/payload bytes in its typed canonical location. The second contains the exact caller,
session, revision, authority-record commitment, lineage, workspace, world, and current-policy
observation returned by the bound read. The third contains the complete descriptor and runtime
launch plan. The fourth contains the exact policy identity, current policy revision,
`max_live_retained_workers`, and admission inputs. Unknown or omitted fields, alternate JSON,
delimiter concatenation, presentation serialization, and digesting a prompt/payload separately are
forbidden. The stored digest is lowercase-hex `HMAC-SHA-256(admission_key, bytes_above)`.

RetainedWorkerRuntime owns one separate admission commitment-key envelope and registry header per
authority store through its opaque fixed-registry physical capability. The envelope binds exact
schema version, authority-store ID, key ID, creation timestamp, `HmacSha256`, and 32 OS-CSPRNG
bytes. First initialization under the root lock publishes the owner-only/no-follow key file through
a same-directory temp, `fsync`, no-replace rename, and directory `fsync`, then commits and `fsync`s
the registry header naming that key. A crash before the header commit leaves an unregistered key
orphan that locked recovery deletes only after proving the registry is absent; a committed header
with a missing, malformed, wrong-store, wrong-key, or unreadable envelope is corruption and fails
closed. Concurrent initialization exact-joins only the same committed header and key identity.

B3.2a performs no admission-key rotation, retirement, or deletion. Its single committed key remains
verification-capable for every live, interrupted, terminal, rejected, tombstoned, or otherwise
retained admission record that names it, so crash retry and long-lived exact joins cannot lose
verification. A later rotation/retention packet must add a locked complete reachability scan over
the separate admission registry and may retire a key only after zero records reference it. HSA key
rotation and reachability never inspect or control this registry. Raw key bytes, unredacted
canonical input bytes, and prompt/payload bytes are never persisted in the admission record, a side
table, a sealed-preimage object, logs, traces, diagnostics, transport proof, or any other durable
artifact; they exist only in the ordinary in-memory scope of the typed call that received them.
They are never reconstructed from adjacent state or inferred from the stored digest. Diagnostics
may report only key ID and equality/mismatch. Every exact join and every state advance that depends
on the Spawn fingerprint receives the complete typed canonical input, recomputes the framed HMAC
bytes above, and verifies the stored commitment. Digest equality without that complete input is
never promotion or state-advance authority. Issuer reuse
with changed prompt, payload, caller, authority, descriptor, policy, cap, participant, or run ID
conflicts before any additional mutation.

The fixed admission registry is persisted through an opaque physical capability bound to the same
opened root/store identity; RetainedWorkerRuntime alone validates and changes its semantic bytes.
It is keyed by exact `(orchestration_session_id, retained_participant_id)`, and each
`issuer_request_id` may name exactly one record. Each post-R0 registration ID may appear in exactly
one state value. The root lock orders registry publication with R0, but the files are not falsely
described as one cross-file atomic rename.

Before R0 reservation, B3.2a resolves the A1.2a-S bound exact authority and performs one locked registry
transaction that revalidates the current policy identity/cap, joins every post-R0 record to
current HSA registration truth, counts every `SlotReserved`, `AuthorityRegistrationHead`,
`PreTransportNonterminal`, `TransportClaimedNonterminal`, `Routable`, or
`InterruptedNonterminal` record as live, enforces the cap
including all existing slots, allocates/collision-checks participant and bootstrap-run IDs,
computes the complete canonical Spawn fingerprint, assigns the next monotonic session slot
sequence, and persists one immutable `SlotReserved` identity. `Terminal` and
`RejectedBeforeRegistration` do not count. This check and
slot creation are one cross-process admission CAS; the existing process-local bootstrap guard may
remain only as a same-process fast reservation. A missing, extra, stale, cross-session/world,
duplicate, or commitment-mismatched R0/registry join fails closed rather than being omitted from
the count.
`retained_worker_refs.len()`, legacy `authoritative_live`, PID, helper, socket, endpoint, and
process-local maps are never count inputs. Existing action/backend/session/world/policy checks and
the exact `max_live_retained_workers` comparison run against this count before a new reservation.

The registry also owns one durable per-session R0 head. At most one nonterminal record may be
`AuthorityRegistrationHead`, while any number permitted by the cap may remain `SlotReserved` when
no head exists. Head acquisition is request-driven. An exact retry first finds its existing
`SlotReserved` record, re-presents and verifies the complete typed Spawn request, prompt/payload,
admission-time authority observation, descriptor/runtime plan, policy/cap, retained participant,
bootstrap run, and stored HMAC, and proves that this record is the lowest-sequence queued slot. A
root-locked transaction may advance only that record to `AuthorityRegistrationHead` and fix the
current HSA revision/commitment only when no head exists. The retry also validates the complete
admission-to-current ancestry: every contiguous intervening proof must be an exact R0 registration
proof, while active caller, workspace, world, policy, origin, and every unrelated ref remain
unchanged. Any transition proof, gap, ambiguity, changed bound field, changed canonical byte, later
slot, or digest-only presentation conflicts without mutating either the requested or earlier slot.

Current-head reconciliation is a different durable transaction. The same head request must
re-present the complete canonical fingerprint input, exact-join its applied HSA proof, and may then
advance only its own record to `PreTransportNonterminal`, thereby releasing the head. It must not
promote any queued slot in that transaction. A future queued request acquires the released head only
when that earliest request is itself re-presented and passes the complete verification above. A
crash between the HSA CAS and current-record advancement leaves the same head in place; its exact
retry joins the proof and advances only that record. A crash after advancement may reopen with no
head and queued slots, which is valid and remains stable until an exact earliest retry.

A later request cannot overtake an earlier queued slot merely because it reappears first. PID,
caller presence or loss, helper/process liveness, timeout, EOF, socket/endpoint state, transport
state, and observer loss cannot acquire, replace, renew, or steal the head. An abandoned earliest
slot remains conservatively live and blocks every later slot until the same canonical request
retries or a later explicitly owned lifecycle/control protocol resolves it. B3.2a adds no such
abandonment/control protocol.

### Deferred retained-spawn admission recovery contract

B3.2a deliberately stops at conservative exact-retry behavior. It adds no cancellation,
abandonment, expiry, or liveness-based resolution protocol and adds no runtime enum, schema field,
or persisted request preimage for future recovery. The remaining B3.2 packet owns every durable
admission-state transition used to resolve an abandoned admission; `WorldDispatchControl` in B4
owns the user/tool-facing exact inspect/cancel verb and consumes the RetainedWorkerRuntime result
without writing admission state itself.

Any later resolution request must exact-join all of the following durable preconditions before a
state advance: issuer request identity, admission-record identity and revision, orchestration
session, current exact authority and the relevant admission-to-current ancestry, current policy
identity and authorization, retained participant, and the exact admission state being resolved.
The complete canonical Spawn request remains required wherever fingerprint verification or exact
retry semantics depend on it. PID, timeout, caller presence or disappearance, helper state, socket
state, endpoint state, EOF, observer loss, and process liveness supply no resolution authority.

Resolution is forward-only against durable protocol truth. It must not delete or roll back an
already-committed R0 lineage/ref/registration, classify R0 rejection as cancellation, or treat
stopping an already-created worker as cancelling a pending admission. Partial or already-applied
R0 registration is reconciled to its exact admission record. A transport claim whose launch or
registration result is ambiguous remains live and cannot free capacity until exact transport and
runtime truth is reconciled. Crashes before and after resolution converge on retry; repeated
resolution exact-joins the same terminal result; and the live admission count decreases only after
that terminal result is durably published. Only then may the next eligible queued request acquire
the registration head under the existing earliest-slot rule.

B4 may freeze and return these semantic outcome categories without adding them to the B3.2a
runtime schema: cancelled before registration; registered before transport; transport ambiguous
or cancellation pending; already routable or terminal; invalid target; ambiguous target; and
policy denied. Remaining B3.2 must first provide the durable, restart-safe resolution/reconciliation
primitive those outcomes consume.

The host-to-world launch carrier is typed and substitution-resistant:

```rust
struct RetainedWorkerAdmissionCommitmentCarrierV1 {
    schema_version: u32, // exactly 1
    algorithm: String,   // exactly "hmac-sha-256"
    key_id: String,
    digest_hex: String,
}

struct RetainedWorkerLaunchAuthorityProofV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1,
    registration_id: String,
    registration_commitment: AuthorityObjectCommitmentV1,
    authority_revision_after: u64,
    authority_record_commitment_after: AuthorityObjectCommitmentV1,
    orchestration_session_id: String,
    caller_participant_id: String,
    retained_participant_id: String,
    bootstrap_run_id: String,
    transport_claim_id: String,
    backend_id: String,
    protocol: String,
    world_binding: WorldBindingV1,
    current_policy_ref_id: String,
    current_policy_revision: String,
    retained_worker_ref_id: String,
    retained_worker_commitment: AuthorityObjectCommitmentV1,
}
```

Before either transport builder serializes it, the host exact-joins every field to the current HSA
registration proof, admission record/fingerprint, bound caller, descriptor, policy, and world. The
carrier is the exact field-for-field, transport-neutral equality projection of the internal
RetainedWorkerAdmissionCommitmentV1; it contains no secret key and grants no HSA or admission
mutation authority. The receiving world compares the closed carrier and dispatch fields only; it
does not verify or reinterpret the host-only HMAC input. The
optional carrier field on the V1 compatibility request may remain absent only on the explicitly
pre-activation legacy path; both A1.2a-S authority-managed Spawn producers require it with no
fallback. `transport-api-types` validates its closed shape. Production member Spawn enters
`Service::execute_stream`. For authority-managed `Some(exact proof)`, that member branch completes
the existing strict proof/dispatch equality validation, durably adopts the exact HSA-bound physical
world through B3.2a-WA, and only then calls `MemberRuntimeManager::launch`, which retains its own
validation before process creation. Compatibility `None` retains the existing direct path.
`world-api`, `Service::execute`, and `convert_member_dispatch_request` are not part of this route.
World-service does not mint or advance HSA or admission truth; the bound host verification supplies
authority authenticity, B3.2a-WA supplies physical ownership only, and the launch boundary supplies
exact carrier/request equality.

`#[serde(default)]` on the optional `MemberDispatchRequestV1` proof field preserves wire
compatibility by supplying `Default::default()` only when that field is absent during
deserialization. It does not initialize Rust struct literals: each literal must name the field (or
use explicit Rust struct update syntax, which is not authorized for these fixtures). Therefore the
B3.2a allowlist additionally permits edits only in
`crates/world-service/tests/member_runtime_world_placement_v1.rs`,
`crates/world-service/tests/streamed_execute_cancel_v1.rs`, and
`crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`, solely to set the field to
`None` in existing explicitly pre-activation or legacy literals and prove unchanged compatibility.
Every other fixture input, test name, assertion, expected outcome, and expected error remains
unchanged. Any existing test that claims the authority-managed B3.2a route must carry an exact
valid proof through the canonical production path instead; `None` is not valid there. These test
files may not introduce a production path, helper default, fixture-only authority, alternate
proof, alternate transport, side table, or weakened assertion. Authority-managed Spawn still
requires the exact proof. This mechanical authorization did not itself complete B3.2a; the
recorded B3.2a/B3.2a-WA result in `05` now does.

That Rust-literal requirement also authorizes exactly three production compatibility
initializers and nothing else: `build_run_world_task_transport_request` and
`build_fork_world_worker_transport_request` in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`, plus `convert_member_dispatch` in
`crates/world-mac-lima/src/lib.rs`. Each may only name the new optional proof field as explicit
`None`; every pre-existing backend, protocol, run, session, participant, lineage, world, prompt,
runtime, route-selection, policy, placement, lifecycle, error, and outcome field remains
byte-for-byte and behaviorally unchanged. `None` grants no retained-worker launch authority to
RunWorldTask, ForkWorldWorker, or the macOS world-api conversion. This is not macOS
authority-managed Spawn adoption and authorizes no edit to `crates/world-api/src/lib.rs`,
`Service::execute`, or `convert_member_dispatch_request`. No default constructor, side table,
environment carrier, alternate transport route, or hidden proof synthesis may replace these
explicit initializers. Both authority-managed B3.2a Spawn producers still require
`Some(exact RetainedWorkerLaunchAuthorityProofV1)` exact-joined to admission and R0 truth before
serialization, and missing, malformed, or mismatched proof fails before process creation. This
mechanical authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05`
is review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.

The same compiler-required widening authorizes only seven additional production-symbol edits. In
`crates/shell/src/execution/orchestrator_world_dispatch.rs`, `fork_world_worker` and
`continue_world_worker_fork_command_bootstrap_after_delivery` may only pass explicit `None` for the
optional retained-worker authority context to the widened stream helper. In
`crates/shell/src/repl/async_repl.rs`, `apply_greenfield_host_start_from_authority`,
`prepare_hidden_owner_helper_runtime`,
`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`,
`prepare_fork_child_runtime_startup_for_descriptor`, and
`prepare_member_runtime_startup_for_descriptor` may only initialize, destructure, or preserve the
new optional retained-worker launch-authority proof/admission fields as `None`. Those values grant
no retained-worker launch authority: fork and fork continuation remain compatibility paths; Host
Start retains only its A1.2a-S host-session authority; hidden-owner-helper behavior is unchanged;
and `prepare_member_runtime_startup_for_descriptor` remains legacy pre-activation preparation,
distinct from the B3.2a-only `prepare_member_runtime_startup_from_authority_registration` path.
Only that authority-registration preparer may construct the paired
`Some(exact RetainedWorkerLaunchAuthorityProofV1)` and exact admission context. An
authority-managed retained Spawn with either value missing fails closed and cannot fall back to the
legacy preparer or reinterpret generic `None` as compatibility. The seven exceptions change no
packet ownership, policy, lifecycle, transport, identity, lineage, error, outcome, host-runtime,
helper, legacy-member, or fork semantics and authorize no other structural change. This mechanical
authorization did not itself complete B3.2a; the recorded B3.2a/B3.2a-WA result in `05` is
review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.

### B3.2a-WA `ExactBoundWorldOwnershipAdoptionV1`

The first bounded Linux live Spawn proof reached durable admission, R0 registration, the unique transport
claim, and typed launch-proof validation, then failed closed before process creation because the
HSA-bound generic REPL world and the world-service-created shared-owner world had different IDs.
Creating another world is not a valid repair: HSA already owns the exact session binding and neither
world-service nor backend metadata may replace it. The docs-first correction therefore made the
internal operation contract B3.2a-WA, required before B3.2a closeout and B1/B2.1-0; the recorded
result in `05` now satisfies that prerequisite through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`:

```text
ExactBoundWorldOwnershipAdoptionV1

GenericExactBoundWorld
    -> SharedSessionOwnerExactBoundWorld
```

This is an internal service/backend call contract. It is not a new `world-api` field, request
variant, persisted wire-schema version, authority object, admission state, side table, or lifecycle
state. The existing generic/shared world metadata shape remains unchanged.

Ownership remains separated:

1. `HostSessionAuthority` alone owns the exact durable orchestration-session world ID and
   generation and the policy binding accepted with that authority. Adoption consumes this tuple as
   immutable evidence and cannot create, repair, revise, or clear it.
2. `RetainedWorkerRuntime` alone owns admission, R0-registration join, the unique transport claim,
   routability, interruption, and terminal truth. Adoption changes none of those bytes and cannot
   turn an exact joined claim into permission to resend.
3. World-service and the runtime-family/Linux world backend own only physical realization and
   durable physical ownership metadata. They may adopt the exact already-bound generic world; they
   may not choose a replacement world when that exact world exists.
4. `WorldDispatchControl`, the shell episode, helpers, and compatibility projections consume the
   result only. They do not authorize or perform adoption.

The closed transient input/evidence set is:

```rust
struct ExactBoundWorldOwnershipAdoptionV1 {
    orchestration_session_id: String,
    world_id: String,
    world_generation: u64,
    participant_id: String,
    authority_managed_launch_proof: RetainedWorkerLaunchAuthorityProofV1,
    policy_ref_id: String,
    policy_revision: String,
    policy_snapshot_hash: String,
    canonical_project_identity: String,
    canonical_world_spec_identity: String,
    target_shared_session_owner_id: String,
    adoption_correlation_id: String,
}
```

The sketch freezes semantic members, not a Rust/public/wire type requirement. The implementation may
use a narrower private borrowed type composed from existing values. The non-secret adoption
correlation is equality-only call-scope evidence and is not a prompt/request preimage or ownership
source. Raw prompt, request, payload, credentials, or secrets are excluded from the operation,
metadata, errors, logs, traces, and diagnostics.

The operation may run only when all preconditions hold under the backend's trusted shared-owner
lock:

1. The authority-managed carrier is `Some(exact proof)` and its complete existing strict validation
   has succeeded before adoption. Compatibility `None` cannot enter this operation and cannot be
   reinterpreted as a managed request.
2. Exact session, world ID, generation, participant, policy ref/revision/snapshot hash, project, and
   world-spec identity match the validated proof/request and the durable/current backend evidence.
3. The selected physical world is the exact HSA-bound world, has the exact supported generic owner
   shape, and has complete, contiguous, readable metadata. The implementation resolves by exact
   world identity, never by project/spec compatibility alone.
4. Generic metadata has no shared owner fields, foreign owner, conflicting generation, stale policy,
   substituted project/spec, or ambiguous/partially published ownership state.
5. If the exact world is already a shared owner, every owner/session/world/generation/policy/project/
   spec member must match and the operation joins without rewrite. Any mismatch fails closed.
6. Missing, corrupt, forked, discontinuous, substituted, unreadable, unsupported, or multiply
   matching evidence fails closed without metadata mutation or alternate-world creation.
7. Durable adoption completes before member process creation. Adoption does not prove transport
   submission, member creation, Registered, routability, terminal success, or any later lifecycle
   outcome.

First adoption changes only the existing world's ownership metadata from exact generic to exact
active shared-session owner while preserving its world ID, generation, root, project, cgroup,
network, overlay, filesystem mode, policy snapshot, and all other realization semantics. Publication
uses the existing trusted lock and recognized temporary-file, file-`fsync`, atomic rename, and parent
directory-`fsync` conventions. No success may be returned before directory durability. The exact
crash/reopen matrix is:

1. **Before temporary-file creation or persistence:** the durable final remains authoritative
   generic metadata. Reopen observes no adoption; an exact retry may begin first publication.
2. **After a temporary write but before its file `fsync`:** temporary-file presence is never
   semantic ownership. Under the trusted lock, an operation-bound temp with incomplete bytes is
   removed as non-semantic cleanup and the parent directory is re-`fsync`ed; a complete exact temp
   is fully revalidated and file-`fsync`ed before it can proceed. An unrecognized or conflicting
   temp fails closed without deletion or ownership mutation.
3. **After temporary-file `fsync`, including immediately before atomic rename:** the recognized
   candidate remains non-semantic. Reopen revalidates the authoritative generic final and the
   complete exact candidate under the trusted lock, then may atomically rename that candidate and
   parent-directory-`fsync` it. Presence or file durability alone never adopts the world.
4. **After atomic rename but before parent-directory `fsync`:** reopen may observe either the
   original exact generic final (the rename did not survive) or the exact adopted final (the rename
   survived without proven directory durability). Under the trusted lock, the exact generic final
   may restart publication from step 1, including step-2/3 handling of any recognized temp; the exact
   adopted final must be completely revalidated and its file plus parent directory re-`fsync`ed
   before join/success. Missing, conflicting, multiply matching, or otherwise ambiguous final/temp
   evidence fails closed without ownership mutation. Neither observed shape proves adoption before
   parent-directory durability.
5. **After parent-directory `fsync` but before the registry/root response:** the exact final tuple
   is durable adopted ownership. Exact retry joins it byte-for-byte without rewrite and without
   claiming member launch.

Exact retry is exercised after every state above. Recognized temp cleanup or republication never
changes HSA, RetainedWorkerRuntime, or semantic ownership before the exact final rename plus required
directory durability. Conflicting retry leaves the exact durable bytes unchanged.

Conflicting retry leaves the exact durable bytes unchanged. PID, timeout, caller disappearance,
helper state, socket or endpoint state, EOF, process liveness, prompt content, shell-local state, and
compatibility projections cannot acquire, steal, replace, renew, or clear ownership.

Compatibility and scope are closed:

- authority-managed retained Spawn uses adoption only after `Some(exact proof)` validation;
- existing compatibility `None` behavior and ordinary world execution remain unchanged;
- world filesystem, overlay, network, caging, capability, policy, discovery, and write-sync
  behavior remain unchanged;
- Linux proof makes no macOS or Windows claim;
- runtime implementation is authorized only in `crates/world-service/src/service.rs`,
  `crates/world/src/lib.rs`, and `crates/world/src/session.rs`, plus focused colocated or existing
  integration tests needed to prove this contract;
- shell/HSA rebinding, `crates/world-api/src/lib.rs`, schema versions, alternate side tables,
  unrelated world lifecycle refactors, resend/recovery, and abandonment/cancellation remain outside
  scope.

`RG-WORLD-ADOPT-01` was the blocking regression gate and is now satisfied for the bounded Linux
packet through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. Its permanent proof requires exact same-ID/generation adoption,
zero alternate-world creation, byte-stable exact retry, conflict-without-mutation coverage across
owner/session/policy/project/spec/generation, every named publication crash boundary, member creation
only after durable adoption, unchanged compatibility `None`, request/prompt-marker absence, and green
world-service/world-backend tests plus Linux doctor, ordinary world execution, and bounded
authority-managed live Spawn proof. The recorded result in `05` supplies that evidence. Passing
this gate does not promote any seam and makes no native macOS or Windows claim.

The production order is exact:

1. `dispatch_orchestrator_world_request` validates the raw request and routes only
   `SpawnWorldWorker` to the B3.2a authority-bound preparation before the still-legacy generic
   prepared-dispatch path. Concretely it calls `prepare_authority_bound_spawn_world_worker` and
   then `spawn_prepared_world_worker`; every other action remains on its prior path until
   B1/B2.1-0. Existing `spawn_world_worker` remains the legacy prepared-entry wrapper and delegates
   its already-prepared bootstrap to the same `spawn_prepared_world_worker` execution body.
2. Existing steering/session/world checks and the same-process fast concurrency reservation run
   through `WorldDispatchSteeringInput` and `WorldDispatchConcurrencyInput` passed to the same
   named functions used by legacy preparation. The locked admission CAS
   then exact-joins or creates the `SlotReserved` record from the
   domain-separated ingress request, complete canonical Spawn fingerprint, participant ID, and
   bootstrap run ID before R0. Concurrent processes
   therefore cannot both observe the same available slot. Exact retry returns the same slot;
   changed bytes conflict. The transport builder must use this participant ID and cannot allocate a
   retry-local UUID.
3. A request re-presentation for an existing `SlotReserved` may acquire the head only if it is the
   lowest-sequence queued slot, no head exists, the complete canonical fingerprint input and stored
   HMAC match, and the full admission-to-current ancestry is R0-only and exact. Changed bytes or a
   later-slot retry conflicts with zero mutation. Creating a slot, reconciling another head, caller
   presence, PID, helper, socket, endpoint, timeout, EOF, process state, or observer state never
   supplies promotion eligibility.
4. Only the durable per-session `AuthorityRegistrationHead` may call R0. Its exact retry
   re-presents the complete canonical input, validates the participant and bootstrap run rather
   than allocating either, then exact-joins or creates its HSA request reservation, publishes
   objects, and commits the final HSA CAS. A separate registry transaction exact-joins that proof,
   advances only the matching current record to `PreTransportNonterminal`, and releases the head;
   it does not promote another slot. No member transport opens before both durable commits. Crash
   after slot publication retries from that slot; crash after R0 commit but before admission
   advancement exact-joins R0 and advances only the matching slot. An unrelated request cannot
   adopt either half. A typed terminal R0 rejection may advance the slot to
   `RejectedBeforeRegistration` only after locked proof that no R0 request, registration, object
   parent, or authority mutation exists; ambiguous/local errors leave the same
   `AuthorityRegistrationHead` live. A proven rejection releases the head without promoting a
   queued slot.
5. Before either adapter opens transport, one registry CAS advances
   `PreTransportNonterminal` to `TransportClaimedNonterminal` and fixes a unique
   `transport_claim_id`; only that CAS winner may send. A concurrent exact join waits for Routable
   or returns the existing bounded in-progress/interrupted compatibility result and never sends a
   duplicate. Crash, caller loss, timeout, or process observation cannot steal or renew the claim;
   ambiguous claim state stays live/nonroutable for later B3.2 reconciliation.
6. B3.2a-WA is the intervening physical-ownership step. The direct dispatcher builder carries
   `RetainedWorkerLaunchAuthorityProofV1` through
   `MemberDispatchTransportRequest` and transport-api `MemberDispatchRequestV1`.
   `Service::execute_stream` first completes the existing strict authority-managed proof validation,
   then invokes `ExactBoundWorldOwnershipAdoptionV1` for the exact HSA-bound world. Only after durable
   same-ID/generation adoption may it pass the exact typed member-dispatch request to
   `MemberRuntimeManager::launch`, whose existing validation remains in force before creating the
   world member. The managed branch never calls generic `AttachOrCreate` first and never creates an
   alternate shared-owner world. The direct stream
   consumer accepts only the exact matching B0 Registered event, persists Routable, returns the
   unchanged Spawn outcome, and hands the remaining body to the registry observer.
7. The live `handle_internal_toolbox_world_dispatch_request` Spawn branch invokes the same
   authority-bound preparation instead of `prepare_orchestrator_world_dispatch`, and a new
   `prepare_member_runtime_startup_from_authority_registration` constructs `PreparedAgentRuntime`
   from the slot/R0 graph without calling `prepare_member_runtime_startup_for_descriptor` or
   allocating a participant, bootstrap run, lease, or descriptor identity. It preserves the
   retained runtime handle/map behavior. `build_member_dispatch_transport_request` carries the
   same typed proof.
8. `start_remote_member_runtime_with_prepared` resolves and validates the exact R0/admission tuple
   against its received session/request/participant/backend/protocol/world values before emitting
   Registered. On this activated path, it invokes none of its legacy
   `persist_participant`/`persist_runtime_snapshots` session-participant writers; their in-memory
   fields and existing emitted events remain episode observations. Missing or mismatched proof
   fails before Registered with no fallback. The pre-activation compatibility path is unchanged.
9. The host accepts Routable only from the exact matching B0 Registered event and persists its
   stream/frame/event identity before returning the existing Spawn receipt. The remaining response
   body is handed to an observer independent of the initiating caller. Only an exact B0 Exit and
   matching terminal identity may persist `Terminal`. Error, EOF, body drop, timeout, observer
   loss, process death, PID/helper/socket posture, or restart uncertainty remains or becomes
   `InterruptedNonterminal`; it stays counted live and cannot become routable by inference.

B3.2a proof must exercise a valid no-head/queued registry after reopen; exact earliest retry with
one and multiple queued slots; later retry and changed prompt, payload, caller, authority,
descriptor/runtime plan, policy/cap, participant, and bootstrap-run conflicts with zero slot
mutation; digest-only refusal; concurrent identical/conflicting earliest retries with at most one
head; independent heads for different sessions; and cap accounting that includes every queued
slot. Crash proof covers before, during, and after current-head advancement/release and queued-head
publication, with exact retry from each durable state. Security proof scans registry/object files,
logs, traces, diagnostics, and errors for request/prompt/payload bytes. The explicit caller/PID/
helper/socket/endpoint/timeout/EOF/process-liveness/observer-loss matrix proves that none can
acquire or steal admission, head, transport-claim, routability, or terminal authority.

B3.2a-WA proof separately exercises `RG-WORLD-ADOPT-01`: exact generic-to-shared adoption preserves
world ID/generation and creates no second world; exact retry joins without metadata rewrite; and
owner/session/policy/project/spec/generation conflicts fail without mutation. Publication tests
crash/reopen and exact retry (a) before temp creation/persistence, (b) after temp write, (c) after
temp-file `fsync`, (d) immediately before atomic rename, (e) after rename but before parent-directory
`fsync`, and (f) after parent-directory `fsync` but before response. At (e), tests prove both
permitted reopen shapes: an exact original generic final restarts publication, while an exact adopted
final is revalidated and re-`fsync`ed before join. They prove a temp is never semantic ownership by
presence alone, incomplete recognized temps receive only operation-bound cleanup, conflicting,
missing, or ambiguous temp/final evidence fails without mutation, and any exact final join
re-establishes required file and parent-directory durability before success. The member runtime
cannot be created before durable adoption. Compatibility `None` and ordinary world execution remain
unchanged, and prompt/request markers are absent from fixture storage, world metadata, service
storage, logs, traces, errors, and the bounded journal scan. Adoption alone is never accepted as
transport submission, Registered, routability, terminal success, or member launch proof.

An ordinary retained Continue may consume only the exact R0 target joined to a Routable B3.2a
record. `PreTransportNonterminal`, `InterruptedNonterminal`, and `Terminal` return distinct bounded
non-routable failures without changing authority. Remaining observer reconnect/replay, park,
cancel, stop, fork, accepted-turn lifecycle, and final worker semantics stay in the later B3.2/B4
packets.

The B-owned adapter never falls back to legacy active-task authority and cannot issue/apply a
transition, synthesize authority, change lifecycle, consume obligations, or fabricate correlation.
PID, helper, socket, prompt, foreground-guard, process-local-map, and `authoritative_live`
observations are excluded from every B-owned durable decision. A1.2b remains after B3.1/C1 for all
successor and obligation-dependent work.

Retained-worker Inspect/Cancel/Stop is outside B1/B2.1-0 because R0 deliberately supplies no
retained lifecycle, delivery, or terminal-closeout truth. Those historical cases must still enter
the unchanged full dispatcher for differential accounting, but at this closeout they may only
remain `FailToSameFailure` with an identical normalized signature. Replacing their dispatcher
entry with a lower-level resolver or transport call remains `RegressionMasked`.

The canonical source of every value used to construct or interpret
`PreparedOrchestratorWorldDispatch` is exactly:

| Prepared value or decision input | Canonical source classification | Binding rule |
|---|---|---|
| exact accepted bootstrap-home/store binding and derived `BoundAgentRuntimeStateStore` capability | `HostSessionAuthorityTruth` | A1.2a establishes and reads current authority; A1.2a-S adopts that applied Start on the ordinary internal host bootstrap and carries the exact accepted-home bound capability into production. Only the bound capability may access authority, receipt-registry, or supervisor namespaces. |
| legacy `AgentRuntimeStateStore` clone retained by the prepared compatibility shape | `CompatibilityProjectionValidatedAgainstAuthority` | It may perform only compatibility reads after its physical home/store identity validates against the bound capability; it cannot select or access durable authority namespaces. |
| complete validated request, including request/idempotency/action/mode/payload fields | `ValidatedRequestInput` | Validation proves shape only; every authority-bearing request field is checked against its named canonical source below. |
| orchestration session ID and authority revision | `HostSessionAuthorityTruth` | Must equal the exact resolved authority and root observation. |
| current prepared `OrchestrationSessionRecord` shape | `MissingCanonicalRepresentation` | A1 activation rejects legacy writers and A1.1e does not return this compatibility record. B1/B2.1-0 replaces it on its named B-owned actions with a narrower typed dispatch view sourced from A1.2a authority/application truth; it does not synthesize legacy lifecycle state. |
| active caller participant ID and authoritative lineage | `HostSessionAuthorityTruth` | Caller must be the exact active authoritative participant and a member of the exact lineage. |
| current prepared caller backend, role, and participant record shape | `MissingCanonicalRepresentation` | `DurableSessionAuthorityV1` contains IDs/lineage but not backend/role. A1.2a exposes the exact applied descriptor through a typed read result; B1/B2.1-0 narrows the named B-owned paths instead of synthesizing the legacy record shape. |
| workspace binding and authority-store identity | `HostSessionAuthorityTruth` | Use exact canonical workspace root, authority-store root, and store ID. |
| requested world ID and generation | `ValidatedRequestInput` | Must be present in the validated request where required and equal the exact authority world binding. |
| authoritative world binding and generation | `HostSessionAuthorityTruth` | For an action/runtime that requires a world, absence or mismatch fails closed; a host runtime may validly have no binding or may consume the exact session binding. Compatibility state cannot create or repair it. |
| current policy ref and policy revision | `HostSessionAuthorityTruth` | The ref must be a Policy object and both values must equal the exact current authority. |
| effective-policy/snapshot projection used by existing steering behavior | `CompatibilityProjectionValidatedAgainstAuthority` | Its ref/revision and canonical snapshot commitment must validate against exact current-policy identity before use. |
| `live_retained_worker_count` used by `WorkerContinueForkCommand`/spawn/fork steering | `MissingCanonicalRepresentation` | `retained_worker_refs` carry no live/terminal state, while legacy `authoritative_live` is forbidden authority. B3.2a has supplied the exact RetainedWorkerRuntime admission count for production Spawn, and B3.2a-WA has supplied the required exact physical same-world ownership prerequisite; both are review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. B1/B2.1-0 removes the value from only its RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait view. Continue-fork, retained Inspect/Cancel/Stop, and fork semantics remain unchanged and unpromoted for later RetainedWorkerRuntime/B4. |
| target participant ID named by the request | `ValidatedRequestInput` | It is a requested target only until exact authority/accepted-work validation succeeds. |
| current prepared retained target backend, role, and participant record shape | `MissingCanonicalRepresentation` | A1.1e does not expose the retained object/descriptor as this legacy record, and activated stores reject the legacy writer that current fixtures use. R0 replaces the immutable identity shape: RetainedWorkerRuntime creates the descriptor/resume/worker graph; HostSessionAuthority atomically appends its participant to lineage, binds its ref, and proves the new revision as `HostSessionAuthorityTruth`. B3.2a supplies separate routability/admission truth, B3.2a-WA exact-adopts physical ownership of the same HSA-bound world without changing its ID/generation, and B1/B2.1-0 consumes the exact join without fabricating broader lifecycle state. |
| immutable accepted task/active-run identity and acceptance revision | `ReceiptRegistryTruth` | Exact lookup is scoped by store, session, work identity, caller/backend, and world; unknown acceptance fails closed. |
| active claim, journal cursor, interruption state, routability, and immutable terminal closeout | `SupervisorTruth` | Exact claim must match the acceptance record; waiter/caller drop does not delete it and only exact terminal truth closes routability. |
| optional host-transition correlation before A1.2b adoption | `MissingCanonicalRepresentation` | It remains absent through A1.2a and the joint closeout; request ID, task/active-run ID, compatibility state, or the adapter may not synthesize it. |
| PID, helper, socket, prompt, foreground guard, process-local map, and legacy `authoritative_live` state | `EpisodeObservationOnly` | These values may support transport diagnostics only and never supply or override prepared authority, accepted work, routability, or terminal truth. |

The required current-authority creator/read view and the prepared session/caller/live-retained
fields make Case A unrealizable. A1.2a supplies only the prerequisite authority establishment/read
capability; A1.2a-S supplies only its bounded internal production adopter; B1/B2.1-R0 supplies only canonical retained-target registration/read; B3.2a supplies the
production creation/admission bridge and canonical live/routability state; B3.2a-WA supplies only
exact physical ownership adoption of the HSA-bound world; B1/B2.1-0 removes non-B fields from
the B-owned prepared view. A1.2b keeps all
successor/post-turn and obligation-dependent ownership after C1. Full
`dispatch_orchestrator_world_request`/`dispatch_prepared_orchestrator_world_request` entry is
required regression proof; tests that invoke only a lower-level receipt, supervisor, transport, or
resolver API cannot close this gate.

The recorded B1/B2.1-0 gate result is review-clean through `83101dcb`. The B-owned view requires
exact acceptance revision equality and a durable supervisor cursor at or beyond the acceptance
frame before projecting active truth; ordinary retained Continue journals the canonical B0 stream
without invoking legacy obligation or auto-attach writers. The active-task tool adapter preserves
distinct unknown, stale, backend/world mismatch, nonterminal, exact-terminal, and unresolved-replay
outcomes without mutation. Retained compatibility, Spawn, fork, and foreground blocking behavior
remain unchanged. This gate result does not close B1/B2.1 jointly or promote any seam.

Accordingly, the prior claim that `prepare_orchestrator_world_dispatch` itself belongs to A1.2 is
rejected. It remains a B-owned `WorldDispatchControl` consumer after the named prerequisites.
A1.2a alone issues/applies greenfield Start; A1.2a-S only adopts that result and carries its bound
capability; neither packet owns prepared world-work joins, receipts, supervision, or dispatch
lifecycle decisions.

## 2C. `WorldWorkAcceptanceRecordV1`

B1 persists one minimal accepted-work anchor for either lifecycle family without pulling E2 or
B3.2 into the early corridor:

```rust
struct WorldWorkAcceptanceContextV1 {
    schema_version: u32,          // exactly 1
    proposed_acceptance_record_id: String,
    request_id: String,
    message_id: Option<String>,
    caller_backend_id: String,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
}

enum AcceptedWorldWorkIdentityV1 {
    EphemeralTask {
        task_run_id: String,
    },
    RetainedTurn {
        active_run_id: String,
        message_id: String,
        target_participant_id: String,
    },
}

struct WorldWorkAcceptanceRecordV1 {
    schema_version: u32,          // exactly 1
    acceptance_record_id: String,
    request_id: String,
    authority_store_id: String,
    authority_revision_observed: u64,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    work_identity: AcceptedWorldWorkIdentityV1,
    host_transition_correlation: Option<HostTransitionWorkCorrelationV1>,
    current_policy_snapshot_ref: PolicySnapshotRefV1,
    current_policy_snapshot_hash: String,
    current_policy_revision: String,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    accepted_at: Timestamp,
    record_revision: u64,
}
```

B1 preallocates `proposed_acceptance_record_id` through ReceiptRegistry before submission, but that
candidate is not an accepted record and is not inspectable as accepted work until the runtime
acknowledgement arrives. `WorldWorkAcceptanceContextV1` lives in
`crates/transport-api-types/src/lib.rs` and imports the common correlation type. The existing
`ExecuteRequest` and `MemberTurnSubmitRequestV1` each gain one such typed context; `world-service`
retains the same request-scoped context in its stream/member context. On acknowledgement, B1
persists a record whose `acceptance_record_id`
exactly equals the proposal and whose `runtime_acceptance` joins the acknowledged B0 stream. An
exact submission retry reuses the same proposal; conflicting reuse fails closed. Subsequent B3.1
events copy the retained acceptance ID, request/message causation, caller backend, and optional
correlation into their typed `AgentEvent.worker_event` member; the existing member-turn request
supplies orchestration session, caller/worker participant, worker backend, active run, and world.
This is a typed
request-envelope extension only: it neither creates acceptance before acknowledgement nor changes
`ExecuteStreamFrame`.

The record owns accepted work identity, not final lifecycle, supervisor claim, host-transition
semantics, or model-facing receipt semantics. `host_transition_correlation` is absent for ordinary
world work and remains absent on every production path until A1.2b supplies it from an already
validated HostSessionAuthority transition. B1 defines and stores the optional carrier as part of
its canonical acceptance record, validates only exact equality with the record's
store/session/caller/authority fields, and retains it unchanged; it neither authenticates, issues,
applies, nor interprets the transition and never equates `transition_run_id` with the distinct
accepted task/active-run identity. The landed A1.1e facade is sufficient for the receipt core to
validate an already-existing authority without a new hash domain or verifier; it is not sufficient
to establish that authority or supply the complete joint-closeout prepared-dispatch view. A1.2a
supplies the missing current-authority establishment/read prerequisite but keeps correlation
absent. Production A1.2b later becomes the sole correlation source, first validating its own
intent/revision/payload commitment and then supplying the exact
correlation through the HostSessionAuthority-owned call boundary. B1 validates the exact current
policy identity used for submission but does not invent the E2 retained-worker capability cap. E2
creates the immutable run/cap commitments and the final active receipt references this exact
acceptance record. A conflicting retry cannot create another record for the same runtime work
identity. B2.1 creates its separate claim and journal only through this record.

### B1 receipt-core review versus production completion

B1-3a/B1-3b may become independently review-clean once proposal allocation, exact retry,
acknowledgement validation, activated-store persistence, immutable acceptance inspection, and
legacy-writer rejection are proven. That review point is the only prerequisite that B2.1 consumes;
it is not B1 production completion.

On both the ephemeral-task and retained-turn production paths, the exact acceptance transition is
the handoff boundary. After the immutable record commits, B2.1-1 must durably create or exact-join
the supervisor observation claim before the stream loop reads any subsequent frame. The ephemeral
path must not call `register_active_ephemeral_world_task`, and the retained path must not leave the
foreground loop as the observation owner. Any claim conflict or stale identity fails closed without
ignoring acceptance or falling back to a legacy/generic writer. B1 and B2.1 close only after one
joint production integration gate proves both paths; B3.1 cannot begin from receipt-core review
alone.

### B1 frozen proposal, persistence, and revision semantics

`WorldWorkReceiptRegistry` allocates `proposed_acceptance_record_id` as
`wwa_<lowercase UUIDv7>`. Allocation is a durable proposal reservation, not an acceptance record.
The reservation is stored before transport submission and survives caller drop and host restart so
an exact retry reuses the same proposal. B1 does not expire or garbage-collect reservations; a
later lifecycle packet may add an explicit terminal cleanup rule. Accepted-work inspection never
returns a reservation. Allocation occurs inside the registry transaction and checks the candidate
against every proposal and accepted record across all session states in the single store-wide
registry document; a generated collision is regenerated to a distinct UUIDv7 or fails closed before
publication.

The proposal fingerprint is the exact tuple of:

- authority store ID and observed authority revision;
- orchestration session, caller participant/backend, target backend, world ID/generation;
- request ID, optional retained message ID, and optional unchanged host-transition correlation;
- the complete validated dispatch request, including idempotency key, action, mode, exact typed
  payload, and every optional identity field;
- work family, plus the final typed transport-request commitment and every generated or resolved
  transport identity that must be reused, including the ephemeral task member participant or the
  retained active run/message/target; and
- current policy ref, canonical `PolicySnapshotV3` hash, and policy revision captured before
  submission.

The durable internal proposal and registry schemas are:

```rust
enum ProposedWorldWorkIdentityV1 {
    EphemeralTask,
    RetainedTurn {
        active_run_id: String,
        message_id: String,
        target_participant_id: String,
    },
}

enum WorldWorkSubmissionIdentityV1 {
    EphemeralTask {
        validated_dispatch_request: ValidatedWorldDispatchRequestV1,
        member_dispatch_request: MemberDispatchRequestV1,
        canonical_execute_request_sha256: String,
    },
    RetainedTurn {
        validated_dispatch_request: ValidatedWorldDispatchRequestV1,
        canonical_member_turn_submit_request_sha256: String,
    },
}

struct WorldWorkAcceptanceProposalV1 {
    schema_version: u32,          // exactly 1
    acceptance_context: WorldWorkAcceptanceContextV1,
    authority_store_id: String,
    authority_revision_observed: u64,
    orchestration_session_id: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    proposed_work: ProposedWorldWorkIdentityV1,
    submission_identity: WorldWorkSubmissionIdentityV1,
    current_policy_snapshot_ref: PolicySnapshotRefV1,
    current_policy_snapshot_hash: String,
    current_policy_revision: String,
    created_at: Timestamp,
}

struct WorldWorkReceiptRegistrySessionStateV1 {
    schema_version: u32,          // exactly 1
    proposals_by_request_id: BTreeMap<String, WorldWorkAcceptanceProposalV1>,
    records_by_acceptance_record_id: BTreeMap<String, WorldWorkAcceptanceRecordV1>,
}

struct WorldWorkReceiptRegistryStateV1 {
    schema_version: u32,          // exactly 1
    sessions_by_id: BTreeMap<String, WorldWorkReceiptRegistrySessionStateV1>,
}
```

Each transport-request digest is lowercase SHA-256 over canonical JSON of a B1-local tagged input
containing `domain = "substrate.b1.world-work-submission.v1"`, the exact
`ValidatedWorldDispatchRequestV1`, and the complete final typed `ExecuteRequest` or
`MemberTurnSubmitRequestV1` including its acceptance context. Canonical JSON recursively sorts
object keys and preserves array order. This registry-local equality commitment is not a
HostSessionAuthority commitment or hash domain, does not authenticate transition meaning, and is
never logged as observability evidence. The raw task environment does not need a second durable
copy in the proposal: the caller reconstructs the complete typed request, and exact retry proceeds
only when its canonical digest matches.

`PolicySnapshotRefV1` is not a new B1 policy-reference model. It is the existing exact
HostSessionAuthority `AuthorityObjectRefV1` current-policy reference, required to have object kind
`Policy`, retained unchanged under the B1-facing field name.

Within one authority store, the durable proposal key is
`(orchestration_session_id, request_id)`. An exact fingerprint match with an unaccepted proposal
returns that stored proposal for transport submission. If the proposal already has its immutable
accepted record, the exact retry returns that record without resubmitting transport or launching
duplicate runtime work. Any mismatch, including a different proposed ID, message, active run,
target, world, policy, or correlation, fails before transport submission or mutation. The current
production paths always supply `host_transition_correlation = None`. A retained proposal allocates one
`wwm_<lowercase UUIDv7>` message ID and reuses it on exact retry. The current V1 retained
`active_run_id` is the exact `MemberTurnSubmitRequestV1.run_id`; request and active-run fields remain
separately validated even when an existing caller supplies equal strings.

For an ephemeral task, the registry allocates the `awm_<lowercase UUIDv7>` member participant and
builds the complete `MemberDispatchRequestV1` and final `ExecuteRequest` before publishing the
proposal. An unaccepted exact retry reuses that stored member-dispatch request, including the same
participant, resolved runtime, prompt, world, run, backend, and lineage fields, reconstructs the
final ExecuteRequest, and requires the same submission digest before transport. For a retained
turn, the final `MemberTurnSubmitRequestV1` is built with the stored proposal/message/context before
publication and its exact digest is required on retry. A changed idempotency key, action, mode,
typed payload or rendered prompt, task member participant, runtime descriptor, command, CWD,
environment, network/filesystem request, retained target, or any other typed submission field is a
conflict before transport; B1 never silently allocates replacement transport identity for the same
proposal.

`WorldWorkReceiptRegistry` persists one store-wide versioned document at exactly
`run/agent-hub/world-work-receipt-registry-v1.json` beneath the already bound activated authority
store. This fixed path is deliberately outside the legacy `sessions` and `participants` collections:
placing a receipt below either collection would correctly make the next HostSessionAuthority
preflight reject the root as unsupported pre-A1 authority state. The document contains one
session-keyed state for each orchestration session; each session state contains proposal
reservations keyed by request ID and immutable acceptance records keyed by acceptance-record ID.
The outer session key must exactly equal every contained proposal/record session ID. Every mutation
loads and validates the complete store-wide document, checks acceptance-ID and retained-message
uniqueness across every session, checks the exact scoped work-identity secondary key, and publishes
one replacement document atomically. No separate index file, per-session file, process-local
registry, multi-file update, dual write, fallback write, or side table may participate in acceptance
truth.

Receipt semantics stay above the physical store. `WorldWorkReceiptRegistry` alone parses and
validates `WorldWorkReceiptRegistryStateV1`, allocates identities, decides exact retry versus
conflict, performs inspection, and supplies canonical JSON bytes. It uses the existing canonical
JSON rules and B1-local submission hashes unchanged; the physical layer treats registry bytes as
opaque and does not add a HostSessionAuthority commitment/hash domain or interpret a receipt.
Malformed canonical bytes, unknown fields/variants, invalid session keys, duplicate identities, or
semantic inconsistency fail before publication.

The HostSessionAuthority store exposes one additive physical capability, conceptually
`WorldWorkReceiptRegistryStorageV1`, bound to the exact `CanonicalDirectoryV1` physical root and
`authority_store_id` observed during exact resolution. Its public operation surface is restricted to
read the fixed registry document, atomically replace that document, and finish/revalidate the
transaction; it has no caller-selected collection/path, removal, authority-root CAS, typed-object,
key, intent, journal, or session-authority operation. `AgentRuntimeStateStore` and
`BoundAgentRuntimeStateStore` do not gain a generic post-activation writer. Root replacement/rebind,
wrong store identity, non-`ValidExisting` activation posture, unsafe/malformed receipt path state, or
an otherwise invalid retained capability fails before receipt mutation and leaves the observed tree
untouched. A later valid HostSessionAuthority root revision does not itself stale this store-bound
capability: the transaction re-reads the current root under lock and requires the same physical root
and store ID. B1 never changes either strict `StateRootV1` or strict `StateRootV2`,
`root_revision`, or `authority_revision`; proposal and
record fields retain the exact authority revision observed before submission. The only B1 accepted
record transition remains absence to immutable `record_revision = 1`; no additional physical or
registry revision counter is introduced.

Lock ordering is fixed. A receipt transaction opens and revalidates the trusted root against the
capability, acquires the existing cross-process `authority-v1/lock/root.lock` first, validates and
reconciles canonical authority-store temps, completes `ValidExisting` semantic preflight, verifies
the locked root's store ID, then opens/reconciles the fixed receipt file and runs the registry
operation. It never takes the process-local StateStore snapshot mutex and never nests another
authority transaction. The same root lock is held through registry validation, temp-file sync,
same-directory atomic replacement, directory/root sync, final root/file revalidation, and release.

Receipt crash reconciliation is namespace-specific and also runs under that root lock. Publication
writes canonical bytes to a same-directory temp named only
`world-work-receipt-registry-v1--<32 lowercase hex>.tmp`, syncs it, revalidates the retained root,
atomically replaces `world-work-receipt-registry-v1.json`, syncs the containing directories, reopens
and byte-validates the final file, and only then releases the lock. On entry, an entry using the
receipt-temp prefix but not the exact grammar, a non-regular exact-name entry, or a no-follow or
retained-identity revalidation failure is unsafe and fails closed without cleanup. Every safely
opened regular file with the exact receipt-temp grammar is non-authoritative interrupted publication
state and is removed without parsing or validating its bytes before the final registry document is
read; unrelated `run/agent-hub` entries remain untouched. A crash therefore leaves either the old
complete file or the new complete file plus at most an exact-name non-authoritative temp. The next
exact semantic retry reloads the survivor and either joins the committed proposal/record or reapplies
the missing mutation; it never infers acceptance from a temp. The existing legacy writer remains
rejected after initialization or activation and is neither called nor weakened by this capability.

The logical accepted-record primary key is
`(authority_store_id, acceptance_record_id)`. The exact work-identity uniqueness key is
`(authority_store_id, orchestration_session_id, AcceptedWorldWorkIdentityV1)`, including the enum
discriminant so task and retained identities cannot alias. An acceptance-record ID may name only
one complete record in the authority store, and one scoped runtime work identity may name only one
acceptance-record ID. Inspection by acceptance-record ID scans the exact bound store and succeeds
only for one record; zero or multiple matches fail closed. Inspection by work identity additionally
requires the exact orchestration-session scope. Proposal-only, missing, ambiguous, cross-store, or
stale-session lookups are not accepted work.

`record_revision` is created at exactly `1`. The B1 acceptance record is immutable: exact retry
returns the stored bytes and revision without rewriting timestamps or incrementing the revision;
conflict performs no write. Thus the only B1 revision transition is absence to revision 1.
Supervisor claims, journals, terminal state, and final active receipts use their own later revision
domains and cannot mutate this acceptance anchor.

The one-file creation transaction validates the proposal and every primary/secondary uniqueness
constraint before adding the record. Concurrent exact creations converge to the same stored record;
concurrent conflicting creations serialize to one winner and one fail-closed result. `accepted_at`
and `runtime_acceptance.observed_at` are fixed by the winning first acknowledgement and are ignored
only when deciding whether a later otherwise byte-identical acknowledgement is an exact retry; the
stored timestamps are always returned unchanged.

The live B1 acknowledgement choice is frozen to the first B0 `Start` frame for both current work
families; `SubmissionAccepted` and `RegisteredFrame` remain reserved enum values and are not B1
production sources:

- `run_world_task`: world-service has successfully created the member runtime control before
  emitting `Start`. The non-empty Start `span_id` is the exact `task_run_id`,
  `runtime_submission_id`, and accepted task identity. The unchanged typed request must satisfy
  `acceptance_context.request_id == MemberDispatchRequestV1.run_id == proposal.request_id`; any
  substituted request context fails before persistence.
- `continue_world_worker`: world-service has validated the exact retained target, reserved the turn
  slot, successfully created runtime run control, registered the submitted turn, and retained the
  unchanged request acceptance context before emitting `Start`. The Start `span_id` is the runtime
  submission ID; `active_run_id` is the request's exact `run_id`, and message ID/retained target come
  only from that retained typed context and typed request.

For either family, Start must be frame sequence 1 on the response stream holding the same typed
context. The record copies `stream_id` and `frame_sequence` from that frame and the proposed ID from
the retained context. Task Start sets only `task_run_id`; retained Start sets only
`active_run_id`, `message_id`, and `retained_participant_id`. A repeated Start, non-Start frame,
Error, EOF, stream exhaustion, terminal frame, local request write, or process-spawn observation
cannot create acceptance. Any mismatch in proposal, store/session/caller/backend/target/world,
policy, work identity, retained message/target, stream, or frame identity fails before persistence.

The policy ref and revision come from the exact pre-submission HostSessionAuthority observation.
The snapshot hash is exactly the existing `ResolvedPolicySnapshot.snapshot_hash`: lowercase
SHA-256 over the serialized canonical `PolicySnapshotV3` actually put on the task request or
resolved for the retained submission before the request is sent. B1 does not define or recompute a
second hash. It carries those three values unchanged through acknowledgement and never re-resolves
policy afterward. These fields describe current submission policy only; they are not the E2
immutable retained-worker cap or final receipt commitment.

## 3. `ActiveEphemeralTaskReceiptV1`

```rust
struct ActiveEphemeralTaskReceiptV1 {
    schema_version: u32,
    acceptance_record_id: String,
    task_run_id: String,
    request_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    policy_snapshot_ref: PolicySnapshotRefV1,
    policy_snapshot_hash: String,
    policy_revision: String,
    narrowing_reason: Option<String>,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    observation_claim: SupervisorObservationClaimV1,
    accepted_at: Timestamp,
    state_revision: u64,
    state: ActiveTaskStateV1,
    cancel_supported: bool,
    terminal: Option<WorldWorkTerminalV1>,
}
```

States:

```text
Accepted -> Running -> AttentionPending -> Running
Accepted|Running|AttentionPending -> Terminal|Failed|Cancelled|Invalidated
```

`Terminal`, `Failed`, `Cancelled`, and `Invalidated` are terminal and monotonic. `Parked` is not valid for an ephemeral task.

`WorldWorkTerminalV1` for an ephemeral task carries one explicit result class:

```text
Completed
Failed
Cancelled
NeedsRetainedFollowup
Invalidated
```

`NeedsRetainedFollowup` is a terminal ephemeral result, not a retained worker state. It promises no durable participant identity, creates no `continue_world_worker` route, and does not silently create a retained worker or durable conversational obligation. The host must make a new, explicit, policy-checked `spawn_world_worker` decision if ongoing work is warranted.

## 4. `ActiveRetainedTurnReceiptV1`

```rust
struct ActiveRetainedTurnReceiptV1 {
    schema_version: u32,
    acceptance_record_id: String,
    active_run_id: String,
    request_id: String,
    orchestration_session_id: String,
    orchestrator_participant_id: String,
    target_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    message_id: String,
    thread_id: Option<String>,
    worker_policy_cap_hash: String,
    turn_policy_snapshot_ref: PolicySnapshotRefV1,
    turn_policy_snapshot_hash: String,
    turn_policy_revision: String,
    narrowing_reason: Option<String>,
    runtime_acceptance: RuntimeAcceptanceEvidenceV1,
    observation_claim: SupervisorObservationClaimV1,
    accepted_at: Timestamp,
    state_revision: u64,
    state: ActiveRetainedTurnStateV1,
    cancel_supported: bool,
    terminal: Option<WorldWorkTerminalV1>,
}
```

States:

```text
Accepted -> Running
Running -> AttentionPending -> Running
Accepted|Running|AttentionPending -> Parked|Terminal|Failed|Cancelled|Stopped
```

Rules:

1. One retained worker may have at most one active cancelable turn unless a later version explicitly models concurrency.
2. `Parked` closes the active turn but preserves the retained worker manifest and resume handle.
3. Host posture is not copied from turn state. Obligations may cause host `AwaitingAttention`; worker `AttentionPending` remains worker truth.

## 5. Receipt acceptance source

The B1 acceptance record is an internal durable anchor, not the returnable active receipt described
below. It captures exact current-policy and B0 acknowledgement truth. E2 must create the final
immutable policy/cap commitment and B2.1 must persist its separate observation claim before B2.2
may return the linked active receipt.

A foreground call may return an accepted receipt only after all of the following are durable:

1. exact session, caller, backend, world, and task/worker target identity;
2. the accepted immutable `PolicySnapshotV3` ref/hash;
3. exact `task_run_id` or `active_run_id` plus retained `message_id` where applicable;
4. a supervisor observation cursor or resumable observation claim; and
5. runtime submission acknowledgement that is joined to the same identity.

The persisted acceptance evidence has this minimum shape:

```rust
struct RuntimeAcceptanceEvidenceV1 {
    acknowledgement_kind: RuntimeAcceptanceAcknowledgementKindV1,
    acceptance_record_id: String,
    stream_id: String,
    frame_sequence: u64,
    runtime_submission_id: Option<String>,
    task_run_id: Option<String>,
    active_run_id: Option<String>,
    message_id: Option<String>,
    retained_participant_id: Option<String>,
    observed_at: Timestamp,
}

enum RuntimeAcceptanceAcknowledgementKindV1 {
    SubmissionAccepted,
    StartFrame,
    RegisteredFrame,
}

struct SupervisorObservationClaimV1 {
    stream_id: String,
    last_durable_frame_sequence: u64,
    last_durable_event_sequence: Option<u64>,
    claim_revision: u64,
    lease_epoch: u64,
    resumable: bool,
}
```

V1 acceptance boundaries:

- `RuntimeAcceptanceEvidenceV1.acceptance_record_id` exactly equals the request's proposed ID, and
  the acknowledgement belongs to the runtime stream holding that same typed acceptance context;
  absence or mismatch fails before record persistence or B3.1 event emission.
- For `run_world_task`, acceptance may use the first non-terminal `Start` or `Registered` frame
  only when the protocol defines that frame as runtime acceptance and its B0 stream/frame identity
  includes or unambiguously joins to `task_run_id`.
- For `continue_world_worker`, acceptance requires an explicit retained-turn submission
  acknowledgement or first non-terminal B0 frame that includes or unambiguously joins to
  `active_run_id`, `message_id`, and the exact retained target.
- A socket write, HTTP request submission, process spawn attempt, or locally allocated ID is not runtime acceptance by itself.
- A terminal-only identity observation cannot be relabeled as pre-terminal acceptance.
- If the current runtime protocol cannot expose accepted identity before terminal exit, extend that protocol before changing the foreground tool to receipt-oriented early return.

B1 persists and inspects both task and retained-turn acceptance records while the existing
foreground call still waits. B2.1-1 then performs the no-gap durable supervisor handoff at the same
accepted production boundary, B2.1-2 makes that foreground call a waiter over journal truth, and
B2.1-3 proves restart reconciliation. Foreground early return is a separate B2.2 gate, and B1 is not
production-complete until the joint B1/B2.1 closeout passes.

## 6. `RetainedWorkerManifestV1`

```rust
struct RetainedWorkerManifestV1 {
    schema_version: u32,
    retained_participant_id: String,
    orchestration_session_id: String,
    orchestrator_participant_id: String,
    parent_retained_participant_id: Option<String>,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    runtime_family: String,
    resume_handle_ref: Option<ResumeHandleRefV1>,
    worker_policy_cap_ref: PolicySnapshotRefV1,
    worker_policy_cap_hash: String,
    worker_policy_cap_snapshot: PolicySnapshotV3,
    config_projection_identity: ConfigProjectionIdentityV1,
    config_projection_ref: ConfigProjectionRefV1,
    execution_envelope_ref: ExecutionEnvelopeRefV1,
    lifecycle_state: RetainedWorkerLifecycleStateV1,
    active_turn_ref: Option<ActiveRunRefV1>,
    manifest_revision: u64,
    created_at: Timestamp,
    updated_at: Timestamp,
}
```

Rules:

1. Spawn-time effective policy, including spawn narrowing, becomes the maximum worker cap.
2. Fork inherits the source cap by default and may narrow further; it cannot broaden.
3. Clean turn exit may park the worker. It must not delete retained identity or resume continuity.
4. World generation mismatch makes the worker unroutable/invalidated; it does not silently rebind.
5. `active_turn_ref` is a later RetainedWorkerRuntime lifecycle reference to the immutable B1
   accepted record. B1 does not create, update, or close it; the later retained-lifecycle packet
   owns that separate atomic lifecycle transition without mutating accepted-turn identity.

## 7. `DispatchPolicyNarrowingPatchV1`

```rust
struct DispatchPolicyNarrowingPatchV1 {
    schema_version: u32,
    request_id: String,
    orchestration_session_id: String,
    caller_participant_id: String,
    target_backend_id: String,
    target_world: WorldBindingRefV1,
    applies_to: DispatchCapabilitySubjectV1,
    parent_policy_ref: PolicyRefV1,
    parent_policy_revision: String,
    restricted_policy_patch: RestrictedPolicyPatchV1,
    reason: Option<String>,
}

enum DispatchCapabilitySubjectV1 {
    EphemeralTask,
    RetainedWorkerSpawn,
    RetainedWorkerTurn { retained_participant_id: String },
    RetainedWorkerFork { source_participant_id: String },
}
```

V1 `RestrictedPolicyPatchV1` may contain only `world_fs` fields. Unknown or non-`world_fs` fields are rejected, not ignored.

## 8. `WorldRuntimeAdapterExecutionEnvelopeV1`

```rust
struct WorldRuntimeAdapterExecutionEnvelopeV1 {
    schema_version: u32,
    envelope_id: String,
    kind: AdapterExecutionEnvelopeKindV1,
    runtime_family: String,
    guest_entrypoint: PathBuf,
    runtime_dependency_ref: RuntimeDependencyRefV1,
    projected_home: Option<PathBuf>,
    workspace_root: PathBuf,
    world_id: String,
    world_generation: u64,
    retained_participant_id: Option<String>,
    active_run_id: Option<String>,
    policy_snapshot_ref: PolicySnapshotRefV1,
    policy_snapshot_hash: String,
    env_projection_ref: EnvProjectionRefV1,
    config_projection_identity: ConfigProjectionIdentityV1,
    config_projection_ref: ConfigProjectionRefV1,
    in_world_gateway_ref: Option<InWorldGatewayRefV1>,
    credential_posture: AdapterCredentialPostureV1,
    mediation_posture: AdapterMediationPostureV1,
    command_broker_required: bool,
    allowed_side_effect_channels: Vec<BrokeredSideEffectChannelV1>,
}

enum AdapterMediationPostureV1 {
    BrokerRequired,
    CompatibilityUnproven { compatibility_mode_id: String },
}

enum AdapterCredentialPostureV1 {
    NoCredentialsRequired,
    SecureGatewayHandoff { secret_handoff_ref: SecretHandoffRefV1 },
    CompatibilityCopyBridge { compatibility_mode_id: String },
}
```

Envelope kinds are `HostOrchestrator` and `WorldMember`. A `WorldMember` envelope requires exact world binding, guest-realizable entrypoint, immutable policy snapshot, explicit credential posture, and `command_broker_required=true` for side-effect-capable UAA runtimes.

During D1-before-D2 staging, `CompatibilityUnproven` may preserve explicitly named and logged existing behavior, but it cannot claim Substrate policy mediation, cannot satisfy UAA caging/broker gates, and cannot promote this seam. `BrokerRequired` requires `command_broker_required=true` and fails closed when any declared side-effect channel lacks broker support.

Credential-posture invariants:

1. `SecureGatewayHandoff` requires an exact `in_world_gateway_ref` in the same world generation and a valid `LaunchTimeSecretHandoffV1` ref.
2. `NoCredentialsRequired` requires no secret-handoff ref and cannot later discover ambient host credentials.
3. `CompatibilityCopyBridge` requires a named/logged compatibility mode and cannot satisfy credential, projection, or UAA contract-promotion gates.
4. The UAA child receives the gateway endpoint/session contract, never the raw secret FD or host credential payload.

Allowed channel values describe broker support, not permission to bypass:

```text
ShellCommand
PatchOrApplyEdit
DirectFileWrite
McpOrToolCall
ProcessSpawn
NetworkOperation
ProviderNativeSideEffect
```

Any side-effect channel absent from the envelope is disabled in world scope.

## 9. `LaunchTimeSecretHandoffV1`

### Existing carrier versus remaining adoption work

The current repo already implements the secure carrier mechanics for the managed in-world gateway: `GatewayAuthBundleV1`, an inherited pipe prepared by `world-service`, the pointer environment variable `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, raw-secret env scrubbing, and gateway-side one-time read plus validation. Focused launcher and consumer integration tests make this a positive landed primitive that later slices must reuse and preserve.

`LaunchTimeSecretHandoffV1` adds the orchestration-facing identity, lifecycle, and non-secret evidence needed to join that carrier to an exact world generation, envelope, retained participant, and gateway receiver. The absence of this complete durable record does not mean the FD carrier itself is absent.

Remaining adoption work is to:

1. expose or persist the non-secret handoff reference/state required by the envelope without persisting secret payloads;
2. point direct world Codex/UAA provider traffic at the exact managed gateway that consumed the handoff;
3. construct per-worker runtime-native config from Substrate logical config plus accepted policy instead of copied host config/auth; and
4. prove the complete joined path with production-path smoke/e2e.

Do not replace the existing carrier merely to make its implementation names resemble this control-plane contract. Extend or adapt it only where one of the identity, evidence, fail-closed, or adoption requirements is genuinely missing.

```rust
struct LaunchTimeSecretHandoffV1 {
    schema_version: u32,                 // exactly 1
    handoff_id: String,
    orchestration_session_id: String,
    world_id: String,
    world_generation: u64,
    retained_participant_id: Option<String>,
    runtime_family: String,

    // Non-secret authority references only.
    credential_source_ref: CredentialSourceRefV1,
    receiving_gateway_ref: InWorldGatewayRefV1,
    delivery: SecretDeliveryMechanismV1,

    created_at: Timestamp,
    delivered_at: Option<Timestamp>,
    consumed_at: Option<Timestamp>,
    expires_at: Timestamp,
    state_revision: u64,
    state: SecretHandoffStateV1,
    failure_diagnostic_ref: Option<RedactedDiagnosticRefV1>,
}

enum SecretDeliveryMechanismV1 {
    SecureFd {
        fd_name: String,
        one_time: bool,
        gateway_receiver_only: bool,
        deny_child_inheritance: bool,
        close_after_consume: bool,
    },
}

enum SecretHandoffStateV1 {
    Prepared,
    Delivered,
    Consumed,
    Failed,
    Expired,
}
```

Allowed transitions:

```text
Prepared -> Delivered -> Consumed
Prepared|Delivered -> Failed|Expired
```

`Consumed`, `Failed`, and `Expired` are terminal. Reuse requires a new `handoff_id` and new descriptor.

Launch-time secret handoff rules:

1. Secret material is resolved by host credential authority and must not be persisted in Substrate records, runtime-native config, workspace overlays, manifests, traces, or logs.
2. `credential_source_ref` is an opaque host-authority reference, not a host filesystem path, credential-store locator exposed to the world, or digest of the secret payload. `fd_name` is a non-secret logical descriptor label.
3. Contract-correct world execution must not copy host credential files or secret-bearing host config into world-visible `CODEX_HOME`, `.codex`, `config.toml`, auth files, or equivalent runtime homes.
4. A bounded non-secret runtime config may be rendered from Substrate-owned logical inventory. Copying a host `config.toml` as authority is compatibility bridging, not projection authority.
5. V1 validation accepts `SecureFd` only when `one_time`, `gateway_receiver_only`, `deny_child_inheritance`, and `close_after_consume` are all `true`.
6. The secure FD is scoped to the exact `receiving_gateway_ref`, consumed by the in-world Substrate gateway at world launch, closed after consumption, and never inherited by the UAA adapter or its children.
7. The gateway—not Codex/UAA—owns credential application, gateway session material, and upstream provider forwarding. The UAA talks to the gateway through the envelope's endpoint/session contract.
8. Logs, receipts, traces, and manifests may contain handoff ID, non-secret refs, state, timestamps, and redacted diagnostics. They must not contain secret payloads, secret-bearing file paths, or reusable hashes/fingerprints derived from the secret payload.
9. Failure, expiry, receiver mismatch, world-generation mismatch, duplicate consumption, or descriptor inheritance risk fails closed for credential-requiring world adapters.
10. A compatibility copied-credential mode is temporary, explicitly named and logged, has retirement criteria, and cannot satisfy `ContractCorrectAndProven` or any secure-handoff acceptance gate.
11. Failed/expired handoffs close the descriptor and clear transient buffers before retry; retry creates a new handoff rather than reopening or replaying the old payload.

## 10. Cancel outcome categories

```rust
enum CancelWorldWorkOutcomeV1 {
    CancelledViaLiveTransport { active_run_id, terminal_ref },
    CancelAcceptedPendingCloseout { active_run_id, cancel_request_id },
    AlreadyTerminal { active_run_id, terminal_ref },
    NoActiveCancelableWork { worker_or_task_ref },
    OwnerUnreachable { active_run_id, durable_state, retryability },
    InvalidTarget { reason },
    WorldBindingMismatch { expected, actual },
    AmbiguousTarget { candidates },
    PolicyDenied { denial_code, explanation },
}
```

Rules:

1. Exact identity and world binding resolve before transport use.
2. `NoActiveCancelableWork` means valid routing context but no accepted non-terminal cancelable receipt.
3. `OwnerUnreachable` means a valid active receipt exists but the live cancellation route is unavailable and durable policy cannot yet declare closeout.
4. `CancelAcceptedPendingCloseout` is not terminal success; inspect remains able to observe eventual closeout.
5. Repeated cancel after terminal returns `AlreadyTerminal`; it never regresses the receipt.
6. Stop targets worker lifecycle. Cancel targets one active task/turn. They are not aliases.

## 11. Supervisor idempotency and restart rules

`WorldWorkExecutionSupervisor` alone owns one canonical receipt-scoped claim-and-journal state containing
active observation identity, lease/observer epoch, exact durable cursor, canonical B0 frame/event
entries, restart discovery, interruption/reconciliation state, unresolved producer-replay state,
and immutable terminal closeout. This state is not an
active-task side table and cannot be interpreted by StateStore or HostSessionAuthority. A dedicated
`WorldWorkExecutionSupervisorStorageV1`-shaped capability may provide opaque crash-safe physical
persistence in the activated authority store, but it has no caller-selected path/collection API and
does not create a generic activated-store writer. Legacy activated-store rejection remains
unchanged.

The claim identity binds the exact authority store, B1 acceptance-record ID and revision,
orchestration session, accepted task/active-run identity, B0 stream, world ID/generation, and claim
revision/observer epoch. An exact duplicate claim joins; a conflicting or stale claim fails closed.
Neither the receipt registry nor the physical store may mutate or infer supervisor state.

B2.1 is reviewed in three ordered subpackets:

1. **B2.1-1 — durable claim and no-gap handoff:** create or exact-join the claim at exact B1
   acceptance before reading the next frame; replace `register_active_ephemeral_world_task` on the
   accepted ephemeral path and transfer the accepted retained path from foreground observation
   ownership to that same claim.
2. **B2.1-2 — journal and blocking waiter:** persist canonical frame/event bytes and exact identity
   before compatibility delivery; exact replay is a no-op and every gap, reorder, conflict, stale
   observer, or post-terminal write fails closed. Foreground inspect/wait and the minimum cancel
   compatibility consume receipt/supervisor truth without owning it.
3. **B2.1-3 — restart and terminal reconciliation:** the supervisor reopens every nonterminal
   claim at its exact cursor, resumes only through exact producer replay/reconciliation, and keeps
   terminal closeout immutable/idempotent. A host/shell restart may resume when the process-memory
   world-service registry survives. A world-service restart or unavailable producer leaves the
   valid durable claim nonterminal and unresolved. Missing exact terminal truth remains
   nonterminal/interrupted.

After those reviews, the joint B1/B2.1 production closeout must prove ephemeral and retained
acceptance, legacy-writer exclusion, caller-drop/restart survival, blocking compatibility, and exact
terminal behavior before B3.1 becomes dependency-ready.

1. **Accepted anchor first:** B1 persists the acceptance record, exact current-policy identity, B0
   stream identity, and acknowledgement sequence before B2.1-1 creates an observation claim in the
   same production handoff without reading a subsequent frame.
2. **Single logical observer:** supervisors claim a lease with
   `(acceptance_record_id, record_revision, lease_epoch)`. A stale lease cannot write a newer
   revision.
3. **Durable observation journal:** at B1 acceptance, B2.1 installs the only post-acceptance
   observation path with no unjournaled handoff gap. It records canonical B0 frame bytes and the
   receipt-scoped frame/event cursor plus the exact B1 acceptance record ID/revision, accepted-work
   identity, and optional transition correlation before waiter delivery or derived receipt/terminal
   state.
4. **Restart discovery:** the canonical supervisor recovery operation enumerates nonterminal
   supervisor claims/cursors,
   exact-joins each to its immutable B1 acceptance record, and resumes only from that durable
   cursor or through exact runtime replay/reconciliation. A crash after acceptance publication but
   before claim publication leaves an accepted-but-unclaimed handoff, not active or terminal
   inference: the supervisor performs a bounded exact anti-join against immutable accepted records,
   validates the full claim identity, creates an interrupted claim at the acknowledgement cursor,
   and requires exact producer replay/reconciliation before advancing it. Missing claim state alone
   never proves running, completion, cancellation, or failure.
5. **Frame dedupe/order:** each frame is keyed by exact `(acceptance_record_id, stream_id,
   frame_sequence)`. An identical duplicate is a no-op; a gap, reorder, conflicting duplicate, or
   post-terminal frame fails closed.
6. **Event dedupe/order:** durable worker events use exact `(acceptance_record_id, event_id,
   event_sequence)` joined to their frame. Reprocessing cannot duplicate the journal entry. C1,
   not the supervisor, decides whether and how that event materializes an obligation.
   Once B3.1 lands, its semantic validator requires the event's acceptance ID, active run, and
   optional transition correlation to exactly equal the B1 record before generic journal handoff;
   absence, substitution, or mismatch fails closed without inference. B2.1's packet exit itself
   requires only opaque canonical-event commitment plus B0/B1 identity and order.
7. **Monotonic states:** terminal states never revert; stale observers cannot overwrite newer
   state; equal-revision conflicting writes fail closed.
8. **Interrupted observation:** EOF, timeout, caller/observer/process loss, endpoint or replay
   absence, or PID/helper/socket/readiness state without exact terminal event proof records an
   interruption, exact unresolved/replay-unavailable state, and retry metadata. It does not
   fabricate terminal success, cancellation, deletion, cursor advance, or a complete obligation
   cut.
9. **Reconciliation:** only exact producer replay/reconciliation that returns the missing B0 frames
   and exact terminal event may advance the cursor or close the run. A runtime known to have exited
   without that terminal event records an interruption/protocol failure with diagnostics, remains
   incomplete, and cannot produce a C1 Complete cut; ambiguous truth likewise retries/fails closed.
10. **Terminal ordering:** only the exact B0 terminal event ID/sequence closes the supervisor
    observation journal. Supervisor terminal closeout and worker active-turn clearing commit
    atomically or through replay-safe idempotent owner-approved steps; the immutable B1 acceptance
    record does not change, and StateStore supplies persistence only.
11. **Ledger handoff:** the supervisor invokes C1 with durable exact B3.1 events and the terminal
    cut, including exact acceptance-record ID/revision, stream ID, accepted-work identity, and the
    scope-equal owner-supplied transition correlation when a transition-scoped snapshot will be queried. It does not
    classify/materialize obligations. C1 independently commits canonical ledger revisions and
    completeness; coordinated storage never transfers semantic ownership.
12. **Blocking compatibility:** B2.1 may leave the foreground waiting on the durable receipt after
    handoff. Dropping that waiter or any foreground guard cannot delete the acceptance record,
    supervisor claim, journal, or work. Only B2.2 enables model-visible early return.
13. **Cancellation:** one durable cancel request ID is reused across retries; repeated transport
    delivery is safe.
14. **Diagnostics:** non-zero exit, stream error, reconciliation failure, and cancel failure retain
    exact active-run/session/world/policy/stream/event joins.
15. **Startup activation:** the current production `run_async_repl` hook may call exactly one
    canonical supervisor recovery entry point and retain the returned observation tasks. It cannot
    enumerate or interpret claims, duplicate reconciliation, or make lifecycle/terminal decisions.
    Store corruption, invalid claim identity, or impossible durable state is fatal and fails
    startup closed. An individual valid nonterminal claim with unavailable producer replay remains
    durably unresolved; ordinary shell startup cannot discard or terminalize it and need not
    pretend every accepted stream resumed. This bounded hook proves neither full ingress-surface
    neutrality nor any seam promotion.

## 11A. Differential-baseline transition gate

A broad-suite differential is monotonic only when exact test-name set comparison and the same
normalized failure-signature procedure prove every transition. Totals alone are not evidence. This
is the permanent `RG-DIFF-01` regression gate recorded in `05`.

Allowed transitions are limited to:

- historical pass to pass;
- historical failure to the same failure with the same normalized signature;
- historical failure to pass only with the exact causal-resolution proof below; and
- a new test that passes.

The following transitions or explanations are forbidden:

- a historical pass becomes a failure;
- a historical test is removed, hidden, filtered, renamed to evade comparison, or newly ignored;
- a historical failure has an unexplained changed signature;
- a historical failure passes because an assertion was deleted or weakened;
- success bypasses the intended production path through a direct resolver, transport, receipt,
  supervisor, or other lower-level substitute; or
- success disables behavior, swallows an error, adds a permissive fallback, or uses test-only
  branching.

For every historical failure that becomes a pass, the closeout evidence must:

1. preserve the exact historical test name;
2. record its historical failure message and normalized signature;
3. reproduce that historical failure at the exact baseline;
4. prove current success through the intended production path;
5. inspect the test-source changes;
6. show assertions are unchanged or strengthened, or explain and independently prove an
   intentional semantic rewrite;
7. map the transition to an exact slice behavior and changed symbol;
8. prove no unrelated feature or enforcement behavior was removed; and
9. obtain independent review.

For the B1/B2.1 inventory frozen in `05`, preserving a historical name while replacing its real
dispatcher, guard-drop, waiter-drop, or tool-to-dispatch route with a lower-level owner call is
still `RegressionMasked`. Renaming, replacing, newly ignoring, weakening assertions, or substituting
a direct resolver/transport/receipt/supervisor call cannot count as `FailToPass` proof.

The evidence must also publish the complete historical and current inventories, exact transition
matrix, retained-failure names and normalized signatures, failure-to-pass manifest, new-test
manifest, and hashes for those artifacts. Any uncertain transition is
`BaselineRegressionAmbiguous`; it cannot satisfy closeout or contract promotion.

## 12. Final-receipt immutable `PolicySnapshotV3` acceptance rules

B1's pre-E2 acceptance anchor records the exact current policy identity used by the runtime but is
not a final receipt and is not model-facing. E2 owns the immutable active-run snapshot and retained
worker cap below; B2.2 may expose a receipt only after those commitments and the B2.1 observation
claim are durable and linked to the B1 record.

A final active task/turn receipt may be exposed only when all are true:

1. exact session, caller, backend, and world binding are resolved;
2. steering policy allows the verb/mode/target;
3. current parent policy is resolved at a known revision;
4. retained worker cap is loaded and hash-verified when applicable;
5. optional narrowing is validated as monotonic;
6. the resulting `PolicySnapshotV3` canonicalizes and passes existing schema/enforcement validation;
7. snapshot bytes/ref/hash/revision are durable;
8. the execution envelope/world-service request carries the same verified snapshot;
9. runtime acceptance evidence joins the acknowledgement to the exact work identity;
10. the observation claim is durable and resumable; and
11. the receipt references the snapshot, acceptance evidence, and observation claim before the foreground caller is told the work was accepted.

After acceptance:

- the receipt's policy ref/hash is immutable;
- parent broadening or narrowing does not rewrite the active receipt;
- parent changes apply to future task acceptance, continue, fork, or worker turns;
- emergency revocation is an explicit audited cancel/revoke path, never silent snapshot mutation; and
- snapshot mismatch at broker/world-service fails closed.

## 13. Dispatch narrowing monotonicity rules

The resolver computes:

```text
ephemeral = current_parent AND dispatch_patch
worker_cap = current_parent_at_spawn AND spawn_patch
turn = current_parent_now AND worker_cap AND turn_patch
fork_cap = current_parent_now AND source_worker_cap AND fork_patch
```

V1 field rules:

| Field | Allowed narrowing | Rejected broadening |
|---|---|---|
| `world_fs.host_visible` | `true -> false` | `false -> true` |
| `world_fs.fail_closed.routing` | `false -> true` | `true -> false` |
| `world_fs.caged_required` | `false -> true` | `true -> false` |
| `world_fs.write.enabled` | `true -> false` | `false -> true` |
| `world_fs.deny_enforcement` | same or stronger rank | weaker rank or removal |
| discover/read/write `allow_list` | each requested path is contained by at least one parent path | root/directory/file widening |
| discover/read/write `deny_list` | add denials or retain parent denials | remove parent denial |

Path-containment rules:

1. Normalize relative to the authoritative world/project root.
2. `.` contains `src` and `src/parser.rs`; `src` contains `src/parser.rs`; a file contains only itself.
3. Reject absolute host paths, `..` escape, unsupported glob semantics, NUL, symlink escape, and paths outside the root.
4. Compare canonical policy paths without requiring the target file to already exist; runtime resolution must re-check symlink/ancestor escape at enforcement time.
5. `host_visible=true` with deny-list or unprovable isolation semantics fails closed unless the patch legally narrows to supported full isolation.
6. A patch supplied while `agents.world_dispatch.allow_capability_narrowing=false` is rejected, not ignored.
7. Narrowing may not enable a dispatch action, backend, mode, capability, network route, or side-effect channel forbidden by the parent.
8. Adapter config may receive policy hints, but only broker/world-service enforcement counts.

## 14. Contract promotion gates

A contract is not considered landed until tests prove:

1. serialization and validation;
2. atomic persistence and revision conflict handling;
3. the real ingress/dispatch/runtime path uses it;
4. restart/replay behavior where durable;
5. fail-closed negative cases;
6. every revision-bound host transition joins intent issuance, claim, authority application, and exact result on the real CLI and REPL path, including crash reconciliation and no-reapply exact retry;
7. at least one smoke/e2e path joins session, binding, policy, receipt, runtime event, and terminal/obligation truth;
8. credential-requiring world UAA proof joins the envelope to a consumed one-time in-world gateway handoff without copied secret files or inherited descriptors; and
9. no compatibility copy or `CompatibilityUnproven` evidence is used for contract promotion; and
10. B1/B2.1 production proof shows both accepted work families enter the durable supervisor without
    a legacy-writer attempt, caller/foreground drop does not erase truth, and B3.1 begins only after
    the joint closeout.
