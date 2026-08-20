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

## Development review and remediation contract

This contract governs implementation, documentation, proof, remediation, and closeout work driven
by this control pack. It does not define a Substrate product-runtime review engine.

### Required workflow skills and selected outcome

Every packet begins by loading `using-agent-skills` and then the skills applicable to its phase.
Multi-file implementation or documentation uses `incremental-implementation`; every independent
review uses `code-review-and-quality`. Other skills remain conditional on the work rather than
becoming automatic acceptance requirements.

Before implementation, the packet freezes:

1. the selected integrated outcome and the exact completion claim;
2. the controlling contracts, acceptance criteria, and required proof gates;
3. the allowed files/symbols and explicit non-goals;
4. the subject-fingerprint method and review lenses; and
5. the review budget and stop conditions below.

A reviewer may discover a defect but may not create a new acceptance requirement. A property of
agent-created proof, controller, supervisor, dispatch, or reporting tooling blocks the selected
packet only when the controlling authority explicitly requires that property or the property is
demonstrably necessary for a named proof claim. Otherwise the concern is classified against its
actual effect on the selected integrated outcome and, when valid, retained as `P3` or `P4` in
[`06-review-finding-inventory.md`](06-review-finding-inventory.md).

### Review priority

| Priority | Reviewer label | Required evidence and effect |
|---|---|---|
| `P1` | Critical | Demonstrated severe safety, security, data-loss, destructive-mutation, or authority-integrity failure. Blocks completion. |
| `P2` | Required | Demonstrated failure of the selected contract, acceptance criterion, required gate, authorized scope boundary, or completion claim. Blocks completion. |
| `P3` | Optional / Consider | Useful hardening or improvement without demonstrated failure of the selected integrated outcome. Non-blocking and inventoried when unfixed. |
| `P4` | Nit | Minor polish, naming, formatting, or consistency issue with no correctness effect. Non-blocking and inventoried when unfixed. |

`CLEAN` means no unresolved valid `P1` or `P2`; it may include `P3` or `P4` advisories. A findings
verdict contains at least one valid `P1` or `P2`. Reviewer wording does not set priority by itself:
the parent validates each finding against current authority and live truth, records any evidence-
based reclassification, and preserves the raw review unchanged. Uncertainty alone does not elevate
defense-in-depth or speculative robustness to `P2`; missing evidence is blocking only when that
evidence is required for the selected completion claim.

Priority follows the selected integrated outcome, not the most severe isolated component
observation. An independently observable wrapper or attestation weakness is `P2` only when it makes
the selected proof unable to support its claimed status. A concern that leaves the required proof
independently evaluable is normally `P3`, even when hardening the wrapper would be worthwhile.

### Bounded review cycles

The default automatic budget is:

1. one complete-subject discovery review or same-fingerprint review burst;
2. one consolidated remediation covering every validated `P1` and `P2` from that cycle;
3. one different-fresh, delta-focused closure review; and
4. at most two supplemental causal remediation/closure cycles for new `P1` or `P2` findings
   demonstrated to have been directly caused or unmasked by the immediately preceding remediation.

A review burst uses disjoint lenses over the same subject fingerprint and is consolidated before
one remediation pass. A closure review verifies the remediation, affected contracts/call paths,
invalidated proof, aggregate subject identity, and absence of remediation-caused regression; it
does not restart open-ended discovery. A new observation outside that boundary is `P3` unless the
parent demonstrates its `P1` or `P2` effect on the selected integrated outcome.

Every supplemental cycle remains within the frozen scope, authority, and risk ceiling, cites the
immediately preceding `P1`/`P2` IDs, and records evidence for the causal claim. An unrelated
blocker, material scope/risk expansion, or exhausted two-cycle allowance produces a bounded non-
completed stop for explicit authority. Budget exhaustion never waives a valid `P1` or `P2`, and a
`CLEAN` cycle is terminal: no further review or remediation cycle may be launched.

Reviewers are read-only and fresh after material remediation. Give them the exact authority,
subject/delta, gates, raw verification, unavailable proof, and non-goals. Do not provide
implementation reasoning, remediation discussion, prior reviewer conclusions, or a success-
asserting summary. The parent retains cycle/finding lineage separately from the reviewer's isolated
context.

### Machine-auditable cycle record

For every new runtime-refactor review sequence, the parent owns one JSON record shaped like
[`review-control/review-cycle-record.example.json`](review-control/review-cycle-record.example.json).
Each consolidated cycle records its kind (`discovery`, `closure`, or `supplemental_causal`), stable
ID, exact subject fingerprint, review evidence refs, verdict, findings, and immediate causal
lineage. The record is process evidence only; it does not replace the packet's contracts, tests,
raw reviews, or proof artifacts.

After every returned review cycle, validate the updated record:

```bash
python llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py <record.json>
```

Before launching a closure or supplemental cycle, keep the record `in_progress` and require the
candidate next kind to pass:

```bash
python llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py \
  <record.json> --next-cycle closure
python llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py \
  <record.json> --next-cycle supplemental_causal \
  --causal-evidence-ref <evidence-ref>
```

The standard-library validator rejects invalid order, a cycle after `CLEAN`, inexact triggering
`P1`/`P2` IDs, unchanged post-remediation subject identity, missing supplemental causal evidence,
a third supplemental cycle, and false `complete`/`bounded_stop` status. It intentionally does not
collect evidence, hash live files, judge whether a causal claim is true, launch an agent, or mutate
a checkout. Those remain parent/reviewer responsibilities under the frozen packet authority; do
not add a bespoke supervisor to satisfy this contract.

### Mechanical changes and completion

Before the discovery fingerprint, run formatting, `git diff --check`, the packet allowlist/scope
check, and applicable focused verification so deterministic cleanup does not create a late review
round. A mechanical-only delta is limited to deterministically proved whitespace/formatting,
generated fingerprint or ledger bytes, or exact `P3`/`P4` inventory transcription. Record the diff
and deterministic checks without another reviewer; any semantic uncertainty makes the delta
material. A packet that explicitly requires exact reviewed bytes remains stricter and must perform
the mechanical work before review or follow its named re-review rule.

A packet completes only when its final material cycle is `CLEAN`, all required proof gates pass,
and every valid unfixed `P3`/`P4` is added to or deduplicated against `06`. Historical review
verdicts and packet-specific stricter gates remain immutable evidence, but no future packet inherits
a blanket reviewer-count or all-findings-block rule unless its authority states that requirement
explicitly.

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

R1 implements a structured/error-display contract containing the
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

This bounded Linux contract and diagnostic implementation is review-clean through
`4d0acff68e20d86b97fe5367b8a4617554f33ef4`. It does not close the later R2/R3 or product-wall
proof, promote a seam, or extend authority beyond the two-file R1 allowlist.

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

For the current combined installation/state-root architecture, one selected context is constructed
exactly once at each public install, uninstall, installed world-enable, standalone
install-context-sensitive product-CLI entry, and physical shim entry point, before any filesystem
mutation, build, download, shim operation, doctor, generated-file read/write, sudo crossing,
service action, runtime provisioning, or platform entry. `clap` parse failure, help, `--version`,
`--version-json`, and other explicitly non-mutating syntax exits print and return without private-home/dependency
scaffolding; a successful route that can mutate or consume an install projection constructs the
context before calling the explicit-context home bootstrap:

```rust
struct InstallBootstrapContextV1 {
    selected_host_prefix: AbsoluteHostPath,
    host_substrate_home: AbsoluteHostPath,
    host_substrate_root: AbsoluteHostPath,
    intended_host_principal: PlatformPrincipalV1,
}

enum PlatformPrincipalV1 {
    Unix { account: String, uid: u32 },
    Windows { account: String, sid: String },
}

struct PlatformBootstrapMappingV1 {
    host_context_commitment: Digest,
    platform_instance: PlatformInstanceIdentityV1,
    host_platform_control_root: AbsoluteHostPath,
    realized_substrate_home: AbsolutePlatformPath,
    realized_principal: PlatformPrincipalV1,
    realized_transport: PlatformTransportIdentityV1,
}

enum PlatformInstanceIdentityV1 {
    Lima { vm_name: String, guest_machine_id: String },
    Wsl { distro_name: String, guest_machine_id: String },
}

enum PlatformTransportIdentityV1 {
    Lima { host_socket: AbsoluteHostPath, guest_socket: AbsolutePlatformPath },
    Wsl { pipe_path: NormalizedWindowsPipePath, guest_socket: AbsolutePlatformPath },
}

struct InstallBootstrapContextCarrierV1 {
    context: InstallBootstrapContextV1,
    host_context_commitment: Digest,
}
```

The one Rust wire-model/strict-framing implementation is owned by
`transport-api-types::{InstallBootstrapContextV1, InstallBootstrapContextCarrierV1,
PlatformBootstrapMappingV1}`. Host OS principal/path construction is owned only by
`crates/shell/src/execution/install_bootstrap.rs`, the equivalent public shell/PowerShell entry
sections, and only the physical-shim witness/principal observation functions in
`crates/shim/src/context.rs`. The shim is a distinct public process entry that constructs one IH
from its exact installed invocation witness and current OS identity; it reuses the shared wire
model/normalization validation and is not a second precedence rule or local context type.
`world-backend-factory`, shim, replay, and platform backends consume those shared wire
types or a factory-owned typed projection derived from them; they never define a second prefix
constructor. Cross-language scripts implement the same fixed framing below and must match the Rust
golden vectors byte-for-byte. This is one shared contract model, not a new seam, persistence table,
or backend-selection authority.

The exact dependency realization is part of that single-model rule. `transport-api-types` adds
only `base64 = "0.22"` and workspace `sha2` for this framing/commitment.
`world-backend-factory`, `substrate-forwarder`, and `substrate-shim` add only the existing internal
`transport-api-types` 0.2.8 path dependency so none reimplements parsing or defines a local IH.
For physical-shim identity observation only, its existing Unix `nix` 0.29 dependency adds exactly
feature `user`, and a target-Windows `windows-sys` 0.52 dependency adds exactly
`Win32_Foundation`, `Win32_Security`, `Win32_Storage_FileSystem`, and
`Win32_System_Threading`, `Win32_System_Com`, and `Win32_UI_Shell`. The final two expose
`SHGetKnownFolderPath(FOLDERID_LocalAppData)` and `CoTaskMemFree` for the token-bound control-root
projection. These are feature/package-list edges only; no new
package/version/checksum, parallel codec/identity crate, raw local FFI, or other dependency is
allowed.
These bootstrap/mapping records are in-memory/transport contracts; the A1 durable-object convention
above does not create persistence or lifecycle semantics for them. An install-state copy is only the
generated consistency projection defined below.

V1 requires byte-for-byte equality after normalization:

```text
selected_host_prefix == host_substrate_home == host_substrate_root
```

No R2 implementation may introduce a separate installation root, state root, path-selection side
table, second context constructor, or parallel prefix-precedence function.

##### Host-path normalization and validation

The public entry point normalizes its one selected path before populating any of the three fields.
Normalization is lexical because the selected root may not yet exist; `PrivateSubstrateHomeV1`
later performs the existing no-follow physical acceptance. Normalization never expands `~`, an
environment variable, or CWD; never follows a symlink; never case-folds or Unicode-normalizes; and
rejects invalid UTF-8/unpaired surrogates so the result has one UTF-8 wire encoding.

On Unix hosts, the raw value must begin with exactly one `/`. Empty input, `/`, `//...`, a NUL,
and `.` or `..` components are rejected. One or more trailing `/` characters are removed, and each
remaining non-leading run of `/` characters is collapsed to one. The resulting absolute UTF-8
string is the normalized path. The lexical contract does not invent one cross-filesystem
`NAME_MAX`; component/path length failures remain exact errors from the existing descriptor-bound
filesystem operation before product scaffolding. This intentionally rejects POSIX's
implementation-defined double-leading-slash namespace and does not call `realpath`.

On Windows hosts, `[System.IO.Path]::IsPathFullyQualified` must be true and the raw value must be
either a drive-rooted path (`X:\...` or `X:/...`) or a UNC path with nonempty server and share.
Drive-relative (`X:foo`), root-relative (`\foo`), device/extended namespaces (`\\.\...`,
`\\?\...`), a bare drive root, a bare UNC share root, empty input, NUL/unpaired surrogates,
alternate data-stream colons after the drive, and `.` or `..` components are rejected. Every
component also rejects U+0001–U+001F; `<`, `>`, `"`, `|`, `?`, or `*`; a trailing dot or space;
and, case-insensitively before the first dot, `CON`, `PRN`, `AUX`, `NUL`, `COM1`–`COM9`,
`COM¹`–`COM³`, `LPT1`–`LPT9`, or `LPT¹`–`LPT³`. UNC server/share components obey the same rules. `/` becomes `\`; repeated
separators after the drive root or UNC introducer collapse; a drive letter is uppercased; and
trailing separators are removed. All other component spelling and Unicode scalar values are
preserved.

V1 does not case-fold a not-yet-existing Windows path. It instead rejects physical aliases before
product scaffolding: walk every existing ancestor using a directory handle opened with
`FILE_FLAG_OPEN_REPARSE_POINT`, reject every reparse point, obtain the normalized DOS path with
`GetFinalPathNameByHandleW`, strip only the documented `\\?\` or `\\?\UNC\` presentation prefix,
and require exact component spelling against the normalized selected path. After creating or
joining the final root, repeat that handle check and require exact normalized path plus stable
volume serial number and file ID for the duration of the operation. A short-name, case variant,
junction, mount/reparse alias, or spelling that resolves to the same physical prefix under a
different commitment fails closed; it is never silently joined. These installer-path rules do not
claim Windows A1 authority-store initialization, which remains unsupported by the later
authority-transition contract.

The same normalizer is used for a default and a declared prefix. If Unix receives no declared
prefix, the default is the intended Unix principal's account-database home plus `/.substrate`; it
is not `$HOME`. If Windows receives no `-Prefix`, the default is the intended Windows principal's
OS Known Folder `LocalApplicationData` plus `\Substrate`; it is not `$USERPROFILE`, the
`LOCALAPPDATA` environment variable, or another ambient value. This preserves the current product
default location while removing its environment authority. Install and uninstall use the identical
rule.

These restrictions are grounded in POSIX pathname resolution (relative input depends on CWD and
exactly two leading slashes are implementation-defined), Microsoft `Path.IsPathFullyQualified`
(drive-relative/rooted is not necessarily fully qualified), and the existing no-follow private-home
contract. See [POSIX.1-2024 pathname resolution](https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap04.html)
[Microsoft `Path.IsPathFullyQualified`](https://learn.microsoft.com/en-us/dotnet/api/system.io.path.ispathfullyqualified),
[Microsoft file/path naming rules](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file),
and [`GetFinalPathNameByHandleW`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew).

##### Intended-principal construction

On Unix, the representation is exactly `Unix { account, uid }`. For a non-root public process,
`uid` is the effective UID and `account` is the canonical account name returned by `getpwuid(uid)`;
`$USER`, `$LOGNAME`, and `SUDO_USER` are ignored. For effective UID 0, an explicit intended-account
parameter has first priority; otherwise both `SUDO_USER` and decimal `SUDO_UID` must be present,
nonempty, mutually consistent with `getpwnam`/`getpwuid`, and non-root. A direct effective-UID-0
public invocation may instead require an explicit intended-account parameter, but that account must
resolve and round-trip to a nonzero UID. UID 0 is never an accepted intended principal, even when
explicitly named. Missing, root-inferred, nonexistent, mismatched, root, or non-round-tripping
identity fails before mutation. UID is a canonical unsigned decimal `u32`, never a string. This
preserves the review-clean R1 requirement for one unambiguous intended non-root owner.

On Windows, the representation is exactly `Windows { account, sid }`. The public entry reads
`WindowsIdentity.GetCurrent()` once, stores its canonical logon `Name` and `User.Value` SID, and
rejects anonymous, missing, or untranslatable identity. An explicitly declared account must
translate to that same SID; R2 does not install on behalf of a different Windows principal.
The SID string uses the OS canonical `S-1-...` form. No Unix UID field exists on Windows. See
[Microsoft `WindowsIdentity.GetCurrent`](https://learn.microsoft.com/en-us/dotnet/api/system.security.principal.windowsidentity.getcurrent)
and [`WindowsIdentity.User`](https://learn.microsoft.com/en-us/dotnet/api/system.security.principal.windowsidentity).

##### Exact host-context commitment and carrier encoding

The commitment input is this closed object and no other field:

```rust
struct InstallBootstrapContextCommitmentInputV1 {
    domain: String, // exactly "substrate.install_bootstrap_context"
    version: u32,   // exactly 1
    selected_host_prefix: String,
    host_substrate_home: String,
    host_substrate_root: String,
    intended_host_principal: PlatformPrincipalV1,
}
```

For this install contract only, cross-crate and cross-language source truth requires a fixed
line-framed encoding rather than the shell-private A1 authority-store `CanonicalJsonV1` codec. This
does not change or duplicate that A1 codec. Define `B64(s)` as unpadded RFC 4648 base64url of the
UTF-8 bytes of `s`; its canonical spelling contains only `[A-Za-z0-9_-]`, and decoding must
re-encode to the identical spelling. Define `U32(n)` as unsigned base-10 with no sign and no
leading zero except `0`. The commitment input is the exact ASCII/UTF-8 byte sequence below, with
the shown key order, one `=` per line, LF (`0x0a`) separators, and a required final LF:

```text
domain=substrate.install_bootstrap_context
version=1
selected_host_prefix=<B64(normalized selected_host_prefix)>
host_substrate_home=<B64(normalized host_substrate_home)>
host_substrate_root=<B64(normalized host_substrate_root)>
principal_kind=unix
principal_account=<B64(canonical account)>
principal_uid=<U32(uid)>
```

For Windows, the last three lines are exactly:

```text
principal_kind=windows
principal_account=<B64(canonical account)>
principal_sid=<B64(canonical SID)>
```

`host_context_commitment` is lowercase 64-character hex of SHA-256 over those exact bytes. A parser
accepts exactly eight lines, the exact platform-specific keys/order, LF-only termination, and no
empty encoded path/account/SID; it rejects CR, NUL, whitespace, an unknown/duplicate/reordered key,
noncanonical inner base64url, noncanonical UID, the wrong principal variant, or any extra byte.
Thus field framing is unambiguous without delimiter-sensitive raw values. The normalized path
values must already satisfy the V1 equality invariant.

The carrier payload is the same eight commitment-input lines followed by exactly
`host_context_commitment=<64-lowercase-hex>\n`; the entire nine-line ASCII record is then encoded
once as unpadded base64url. Decoding is strict: outer decode/re-encode equality, exact record
grammar, recomputed digest, path equality, principal round trip, and any declared-prefix match are
all required before child mutation. It is passed as
`--install-bootstrap-context-v1 <carrier>`. Presentation JSON is never a hash or carrier input.
See [RFC 4648 sections 3.2 and 5](https://www.rfc-editor.org/rfc/rfc4648#section-5). The carrier is
not persistence, an artifact-ownership manifest, or deletion provenance.

##### Boundary carriers and selection precedence

Authority is fixed as follows:

| Boundary | Authoritative input | Mandatory matching projections | Forbidden substitution |
|---|---|---|---|
| Public install/uninstall entry | Declared normalized prefix, or the principal-derived default when no parameter exists; principal is OS-resolved once | In-memory `InstallBootstrapContextV1` and commitment | Outer context carrier/env, `$HOME`, `$USERPROFILE`, CWD, outer `SUBSTRATE_HOME`/`SUBSTRATE_ROOT` |
| Dual-mode installer/uninstaller script | With internal carrier argv: validate child mode, carrier, and current-principal binding and forbid construction. Without it: public mode constructs from declared prefix/default; direct Unix uninstall gains a declared `--prefix` | Exactly one mode and one context | Treating an outer carrier as public authority or falling from invalid child mode into public mode |
| Installed `substrate world enable` entry | Before dispatch, reconcile declared normalized global `--install-prefix` and nested `world enable --home`: either one selects A, equal normalized values select A once, and unequal values fail before context construction or mutation; with neither, A is self-derived only from a verified installed-product invocation witness under the exact release/dev/Windows rules below, so a direct repository binary without a selector or witness fails before mutation | In-memory `InstallBootstrapContextV1` and commitment matching any installed projection; an internal argv carrier remains authoritative after strict validation and every declared selector must match it | Environment/home/profile/CWD, generated record contents, runner-local state, or a different executable ancestor |
| Standalone `substrate` shim/status/doctor/world/deps/config/policy/gateway entry | Declared normalized global `--install-prefix`; otherwise A self-derived from a verified installed-product invocation witness below; if neither works, fail before mutation | In-memory context supplied to the selected leaf | Ambient home/root/profile, generated record contents, or silently accepting a direct dev/repo binary without an installed witness |
| Physical `substrate-shim` command entry | A self-derived from the unique no-follow-validated invocation witness at `A/shims/<command>` under the exact pathname/PATH rules below; current host principal; exact default platform selector where platform telemetry is requested | Recomputed `IH`; any inherited carrier/commitment/PM must match; verified factory projection | PATH order/precedence, home/profile/carrier selecting A, an ambient instance/pipe override, or a contextless backend factory |
| Direct Lima/WSL/forwarder/pipe helper | Declared normalized host prefix or principal-derived host default constructs `IH`; declared instance, fixed Lima V1 transport target or declared/default Windows pipe, and OS-resolved host platform-control root construct `PM`, except the bounded pre-PM Lima Stage 1 below | Both carriers after PM exists; internal-child mode validates and never reconstructs; child `HOME`/`LIMA_HOME` and Windows projection paths are exact overwritten projections | Platform selector without host context, guest/default home, ambient `HOME`/`LIMA_HOME`/`LOCALAPPDATA`/pipe env, or falling from bad child mode into public mode |
| Same-process function | Explicit context/carrier parameter | Function-local derived paths | Rereading globals or environment to select a different prefix |
| Context-aware script child | `--install-bootstrap-context-v1` argv value plus current OS/sudo-origin principal equality | Declared `--prefix`/`--home`/`-Prefix` must match | Ambient context carrier or home/root variables, or a self-consistent carrier for another principal |
| Hidden installer home-bootstrap action | Exact `--install-bootstrap-context-v1 <carrier>` argv plus `--install-bootstrap-home-v1`; strict commitment/current-principal/checked-projection validation precedes mutation | Calls only the explicit-context private-home/dependency bootstrap and returns; exact retry is idempotent | Public feature discovery, environment-only selection, normal dispatch, world-deps production processing, shim/runtime/world/policy/lifecycle action, or carrier disclosure |
| Installer-managed leaf product CLI (`substrate` shim/status/doctor/world-deps/config/policy/gateway) | Parent supplies `--install-bootstrap-context-v1 <carrier>` as the authoritative hidden argv child discriminator; leaf decodes/authenticates before dispatch | Leaf overwrites H/R/principal/commitment environment projections from the validated context, passes typed IH to the selected leaf, and requires any inherited tuple to match | Environment carrier/tuple as authority, leaf fallback to home/profile/CWD, or falling from invalid child mode into public construction |
| `sudo` to a Substrate-owned helper | Full carrier and intended-principal input on the helper's explicit argv; helper authenticates sudo origin as specified below | Helper reconstructs the exact environment tuple after validation | `sudo -E`, `--preserve-env`, root's home/account, a caller-chosen unverified UID, or fresh principal reinterpretation |
| Linux ACL bridge fixed system leaf | The owning provisioner validates IH/principal before installing or invoking the helper; the root-owned systemd drop-in or provisioner supplies one exact closed mode/target/group tuple | Only the fixed ACL effect and independent enumeration of current `substrate` group members | Supplying or reconstructing A/H/R/account/UID, accepting any other target/group/mode, treating group membership as intended-principal selection, or accepting an ambient context |
| `sudo` to an arbitrary system tool | The still-owning installer validates IH/principal before `sudo --` and derives the tool's complete target/principal argv from that context | Only the minimum boundary-specific argv (`install` target, account, UID, unit path/name, or service operation); the arbitrary tool never parses a Substrate carrier | Appending a carrier as meaningless tool argv, preserved env, ambient root/user/home paths, shell re-evaluation, or target reconstruction in the tool |
| Generated file or install record | No selection authority | Encoded carrier, digest, and/or self-derived A as specified below | Selecting B from ambient values or treating file location/existence as proof |
| Platform adapter | Validated host carrier plus adapter's public instance selector and OS-resolved host platform-control root | `PlatformBootstrapMappingV1` carrier and matching service/socket/forwarder projection | Host-path string conversion, guest `$HOME`, ambient Lima/Windows control root, backend default account/home, or unrelated instance |

At a public entry, an outer `SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1` is ignored as authority. It is
accepted only when the process was explicitly invoked as an internal child by argv, validates
against the argv carrier, and passes the current-principal binding below. A declared public
`--prefix`, `--home`, `--install-prefix`, or `-Prefix` fixes the context; a conflicting outer
home/root/context is a negative diagnostic input, not a competing precedence branch. For world
enable, global `--install-prefix` and nested `--home` are peer public selectors: the
pre-dispatch shell route normalizes both, joins equal values into one selection, and rejects a
collision before context construction, projection, bootstrap, or mutation. The same collision rule
applies when either selector accompanies an internal argv carrier: the carrier remains authoritative
only after strict decode, commitment, and current-principal validation, and the normalized declared
selector must match it. An outer override may be used only by an explicitly named diagnostic test
and cannot satisfy normal product proof.

For PI-027 the same-process carrier chain is exactly `run_shell_with_cli` →
`ShellConfig::from_cli` → `handle_world_command` → `run_enable`. Every hop receives typed IH by
explicit argument. For PI-108–PI-110 the same rule extends only through the existing Unix
Host/Health/Config/Policy branches to their named consumers. The hidden Agent owner-helper branch
may forward the context only to derive PI-105's immutable `PlatformPrincipalV1`; public Agent and
all other branches remain behavior-equivalent. H/R/carrier environment values remain checked
projections and cannot recover a missing typed argument. No `ShellConfig` mode,
constructor/validator, process-global, or side-table behavior participates in these chains.

PI-071 owns the following process-crossing continuation and no authority reconstruction:

```text
same-process typed IH
  → canonical authenticated carrier encoding
  → explicit child argv
  → child-mode discrimination
  → carrier authentication
  → current-principal/sudo-origin binding
  → declared path equality
  → checked environment projection
  → leaf dispatch
```

`run_enable` and its existing provision-deps path are the only callers; `run_helper_script`
transports the exact encoding and retains normalized `--home` only as a required matching
projection. The helper neither interprets nor logs authority. Carrier-option argv presence is the
sole internal-child discriminator in `world-enable.sh`: missing, malformed, duplicate, tampered,
reordered, forged-principal, or conflicting carrier/home input fails before bootstrap or child
action and cannot fall through to public mode. Environment-only carrier/H/R, ambient `HOME`, CWD,
root home, and generated records cannot select A. Carrier bytes are absent from normal output,
errors, traces, and logs; existing child ordering, stdout/stderr, exit propagation, dependency and
runtime actions, and platform behavior remain unchanged.

`run_enable`, including its existing provision-deps branch, derives A directly from that typed IH
and supplies explicit A to `update_manager_env_exports`. The manager-environment helper cannot use
environment state to select or recover A; its existing config read, world-enabled update, rendering,
output, and error semantics remain unchanged. Non-enable world actions receive no new behavior, and
`run_sync_after_provisioning` remains textually and semantically unchanged. The bounded GitNexus
CRITICAL aggregate for the exact six-file carrier closure, the manager-environment HIGH limited to
explicit-A transport, and the `run_helper_script` HIGH limited to exact carrier-argv transport
authorize only their named argument/signature propagation across existing process families. None
of these results authorizes a new process family, capability, fallback, cleanup, deletion,
rollback, migration, convergence, R2-3, or R3 behavior.

##### Remaining R2-2 same-process carrier closure

The four remaining routes are closed only by explicit arguments:

```text
validated typed IH
  ├─ Host / Health / Config / Policy → named typed consumer
  ├─ intended_host_principal: PlatformPrincipalV1
  │    → REPL or hidden owner-helper runtime
  │    → prepared member dispatch
  │    → Unix account+UID database round-trip
  │    → account home/.codex
  ├─ host shell doctor → optional host diagnostic projection
  ├─ Health
  │    → health.rs
  │    → crate-private shim_doctor::collect_report_for_context name exposure
  │    → existing report::collect_report_for_context
  └─ direct Unix shim doctor
       → unchanged shim_doctor::run_doctor
       → existing report::collect_report_for_context
       → build_report
       ├─ [World enabled] gather_world_doctor_snapshot
       │    → run_json_subcommand
       │    → authenticated hidden passive world doctor child
       ├─ [World disabled] disabled_world_doctor_snapshot
       │    → no child or fixture lookup
       └─ [World enabled] A/health/world_deps.json, or gather_world_deps_section
            → collect_doctor_snapshot_v1
            → A-derived config/dependency paths
```

The principal projection is immutable and process/request-scoped. It is provenance-bound to the
validated IH, travels as a separate explicit argument alongside rather than inside prepared world
dispatch values, and is not independently authoritative or persisted into `HostSessionAuthority`,
orchestration/session records, receipts, supervisor state, retained-worker identity, policy
snapshots, or any other durable state, and never reconstructed from environment or process globals.
The Codex seed resolver requires `PlatformPrincipalV1::Unix`, calls unchanged read-only
`crates/shell/src/execution/install_bootstrap.rs::unix_account_home_for_principal` for the exact
account and UID round-trip through the account database, and derives only that account's home plus
`/.codex`. The terminal helper remains outside the editable allowlist and may not be duplicated. A,
`dirs::home_dir`, `HOME`, `USERPROFILE`, root home, CWD, PID, and helper/session state are forbidden
credential-source selectors. Policy still decides whether the projection is permitted, and no
prompt, credential, auth payload, or secret enters logs, traces, errors, receipts, or new state.

The principal-less compatibility entrypoint
`dispatch_run_world_task_request_with_started_task_run_id_tx` remains intentionally uncalled by the
typed route and must fail closed before any allowlisted Codex seed injection. It may receive only an
item-level `dead_code` allowance whose reason names principal-less compatibility, intentional
non-use, and temporary R2-3 ownership. That annotation retains the frozen surface only; it changes
no visibility, signature, body, cfg, caller, output, error, or runtime behavior, and no second lint
allowance is permitted.

Route B containment is patch-bound to preserved ordinary/binary SHA-256
`f8f845ef6aa6688fdf60be6ed983bb119971d65895a0f38c25fc1dad7643a77f`, preservation commit
`a6e29a10ea3dc9cb673a22912002efb83658e7b2`, and the exact six-file PI-105 manifest. The refreshed
CRITICAL result attributes 17 symbols and 25 existing labels to only three semantic roots:
`run_shell_with_cli`, `handle_agent_command`, and
`build_agent_client_and_member_dispatch_request_impl`; the complete label-by-label mapping is in
`03-phase-slice-map.md`. Final containment requires the same six files, the preserved production
hunks plus only the item-level annotation, no new module or execution-family root, and fresh
semantic review. Raw future symbol-count movement is diagnostic rather than authority, but any
seventh file, other hunk, unmappable label, new owner/family, compatibility-body change, or Route
C/Route D/R2-3 semantic import is an `ImpactDecisionRequired` stop.

The exact post-lint, pre-format Route B candidate is preserved at ordinary/binary SHA-256
`e9da85eb206be522645120dfee459ee79ce48793bc61161487d4b5c4d2fda243`, commit
`07f3117aa0d2f3d57279d20fec204757bba8383a`, with the same six-file manifest. A single mechanical
successor may be produced only by repository-default `cargo fmt --all`. Relative to that candidate,
rustfmt may change layout only in `execution/agents_cmd.rs`,
`execution/orchestrator_world_dispatch.rs`, `execution/routing/dispatch/world_ops.rs`, and
`repl/async_repl.rs`; `execution/invocation/plan.rs` and `execution/routing.rs` remain byte-identical.
No identifier, expression, type, import membership, cfg, comment/string content, test, lint,
signature, caller, or control-flow change is authorized. The successor receives a new exact patch
hash and fingerprints only after an exact hunk audit, zero semantic diff with whitespace ignored,
format-check proof, fresh GitNexus label-to-root mapping, and fresh independent runtime review.
GitNexus count/label movement is formatting attribution drift only while the same six files, module
owners, semantic Route B roots, and execution families remain fixed. Any broader change stops under
the categories in `03-phase-slice-map.md`; this formatting exception cannot authorize Route C,
Route D, R2-3, capability, cleanup, or lifecycle behavior.

The reviewed formatted Route B candidate is preserved at ordinary/binary SHA-256
`ea9cf3e582650007083812ad70e0bf3198405e73d0cde0fb8ecfbedeb49884a2`, commit
`57e13d291abf1239aacee0020ac444ec05e11d56`, with the same exact six-file manifest. Its sole
security-remediation successor preserves that manifest and may change only three contracts plus
their focused colocated tests:

1. `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` is a reserved Substrate-owned output key.
   `maybe_inject_codex_auth_seed_home_for_policy` removes it before any backend, allowlist, or
   principal branch and before every early return. Non-Codex, non-allowlisted, missing-principal,
   and account/UID-resolution failure all leave the key absent. Only successful allowlisted Codex
   resolution from the exact typed principal inserts `account-home/.codex`, overwriting poison.
   Policy eligibility is unchanged, and no process-global environment mutation or ambient override
   is created.
2. Spawn keeps principal truth as a separate request-scoped argument. Compatibility direct and
   prepared entrypoints pass `None`; principal-aware direct and prepared entrypoints pass
   `Some(exact principal)` through `spawn_world_worker` and `spawn_prepared_world_worker` into the
   existing `execute_spawn_world_worker_stream` parameter. `PreparedSpawnWorldWorkerBootstrap`,
   HSA, admission, receipts, supervision, retained workers, policy snapshots, manifests, and wire
   schemas do not acquire the projection. Compatibility remains valid without credential
   projection and fails closed before allowlisted Codex injection when principal truth is absent.
3. The Unix account resolver and policy helper use compile-time
   `cfg(any(target_os = "linux", all(test, unix)))`, with only mechanically matching constant,
   import, and test-item cfg. Linux production remains enabled; Linux/macOS Unix tests may compile
   the helper; Windows tests do not compile Unix account calls. The Linux-only member-dispatch
   injector and `install_bootstrap.rs` ownership remain unchanged.

The bounded source/impact audit is:

| Return/error path | Current poisoned-key result | Required result |
|---|---|---|
| non-Codex backend | preexisting reserved value survives | key absent |
| Codex backend not exactly allowlisted | preexisting reserved value survives | key absent |
| allowlisted Codex with no typed principal | error with preexisting value retained | error with key absent |
| allowlisted Codex with invalid account/UID | error with preexisting value retained | error with key absent |
| allowlisted Codex with exact typed principal | resolved value overwrites poison | exact resolved value; poison absent |

| Function | Principal available? | Currently forwarded? | Required argument |
|---|---:|---:|---|
| `dispatch_orchestrator_world_request` direct Spawn | no | no | `None` |
| `dispatch_orchestrator_world_request_for_principal` direct Spawn | yes | no | `Some(&intended_host_principal)` |
| `dispatch_prepared_orchestrator_world_request` prepared Spawn | no | no | `None` |
| `dispatch_prepared_orchestrator_world_request_for_principal` prepared Spawn | yes | no | `Some(&intended_host_principal)` |
| Linux `spawn_world_worker` | caller-dependent | no | forward the separate option unchanged |
| `spawn_prepared_world_worker` | caller-dependent | no | forward the separate option unchanged |
| `execute_spawn_world_worker_stream` | option already accepted | yes for Fork/Continue-fork, `None` for Spawn | receive the exact Spawn option; body semantics unchanged |

| Symbol/test surface | Linux production | Unix test | Windows test |
|---|---:|---:|---:|
| `resolve_host_codex_seed_home` | compiled | compiled | excluded |
| `maybe_inject_codex_auth_seed_home_for_policy` | compiled | compiled | excluded |
| `maybe_inject_codex_auth_seed_home_for_member_dispatch` | compiled | not newly broadened | excluded |
| colocated Unix account/seed tests and imports | not applicable | compiled | excluded item by item |

GitNexus resolves the policy injector as LOW with four direct callers, one existing
`build_agent_client_and_member_dispatch_request_impl` process root (eleven generated labels), and
the Dispatch module; it resolves the account-home resolver as LOW with one direct caller and the
same process/module. It under-resolves the large Spawn functions. Manual source closure therefore
binds every Spawn call. Compatibility/principal-aware direct dispatch first calls unchanged
`prepare_authority_bound_spawn_world_worker`, then carries separate `None`/`Some` alongside its
result into `spawn_prepared_world_worker`. Compatibility/principal-aware prepared dispatch reaches
`spawn_world_worker` with `None`/`Some`; that function calls unchanged
`prepare_spawn_world_worker_bootstrap`, whose Linux arm calls unchanged
`prepare_authority_bound_spawn_world_worker`, then carries the separate option alongside the
prepared result into `spawn_prepared_world_worker`. Neither preparation function receives or stores
principal truth, and `PreparedSpawnWorldWorkerBootstrap` stays unchanged. The final hop forwards
the option into the already-principal-aware stream function. Fork and continue-fork already use
that final parameter and remain unchanged. A new exact successor patch, fingerprints, hunk
authorization map, fresh GitNexus output, and independent semantic review are required. A seventh
file, additional semantic hunk, durable principal representation, new module or execution-family
root, Route C/D, R2-3, capability, cleanup, or lifecycle change is not authorized.

##### Route C authenticated Host/World doctor projection

Route B remains complete, security-review-clean, Spawn-review-clean, cfg-review-clean, and
semantically unchanged at commit `6cee990f0370013c8b05a5495301db7aea642cd5`. Its exact
ordinary/binary patch from its parent is
`28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674`, and its manifest remains
the reviewed six files. Route C starts only from preservation commit
`41b82327e23798719ed9a0b4cae1f557fb593670`, whose parent is the exact Route B commit/tree. The
starting diff is 182 insertions and five deletions in exactly five files. Its canonical
`git diff --full-index --binary` SHA-256 is
`5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277`; its abbreviated-index
ordinary/binary rendering is
`7c5a908a6760244d9ad6922dfcf7e944cfce3c723d994e279a3aeaded7a8ffa0`. Canonical per-file
full-index binary fingerprints are:

| Route C file | SHA-256 |
|---|---|
| `crates/shell/src/execution/platform/linux.rs` | `d1c53712aef41e01cb44408e42735511af75b8432b8de651cf072a4d13be80a1` |
| `crates/shell/src/execution/platform/mod.rs` | `08340fc50773063113b032422a0a63742aee50d04c8cd8b4edd6c9c4ada13f4c` |
| `crates/shell/tests/doctor_scopes_ds0.rs` | `7f13f02e382dbbe55b53fc15a32183ab889d73a08fe8661b67169161bbe90ba7` |
| `crates/transport-api-types/src/lib.rs` | `82e510bd76fefec4c256c93e271761b1f9ba3047a003a03329979862db631457` |
| `crates/world-service/src/handlers.rs` | `cd24633ffd3049b4dcd30ad3da6d769302483061dcf0a2c473d4d38927bb6018` |

Those bytes authorize a starting candidate, not immutable completion bytes. Within the same five
files, review-driven remediation may correct only Route C tests or diagnostic projection while
remaining inside this semantic envelope; every such correction establishes a new exact patch hash
and per-file fingerprint manifest and reruns impact, focused/broad proof, differential, and fresh
review. A sixth file or a broader authority owner requires `CrossDocumentChangeRequired`.

The authority and disclosure contract is closed:

1. The already-authenticated typed `InstallBootstrapContextCarrierV1` is the only source for A and
   its commitment. `handle_host_command` and `handle_world_command` only forward that typed value;
   `host_doctor_main` and `world_doctor_main` only project it. No second resolver, ambient recovery,
   generated-record authority, process-global state, or side table is permitted.
2. `WorldDoctorReportV1` host fields are optional/defaulted and omit when absent. They report
   `selected_host_prefix` and the non-secret host-context commitment only; they cannot establish,
   validate, replace, or mutate the authenticated context. The world-service `doctor_world`
   producer sets them to `None` and reads no host carrier, HOME/root/prefix, principal, or authority
   state. The legacy adapter may populate them only from explicitly supplied typed IH.
3. Diagnostics return or log no hidden carrier bytes, credential or secret bytes, request bytes, or
   sensitive principal material. The change cannot mutate installation, service, world, policy,
   capabilities, filesystem/network enforcement, placement, caging, credentials, receipts,
   supervision, retained workers, cleanup, or lifecycle/execution behavior.
4. Every source-level process attribution must map to exactly the existing diagnostic roots
   `handle_host_command`, `host_doctor_main`, `handle_world_command`, or `world_doctor_main`.
   Representation/test symbols, the legacy helper, and the mechanical world-service `None`
   producer create no new process root. Route D and R2-3 ownership remain untouched; no seam is
   promoted.

The preservation-time GitNexus result was CRITICAL with 25 attributed symbols, 32 process labels,
and the exact five files. A pre-authorization refresh over byte-identical files reported CRITICAL
19/32/five. Both results are retained as observed evidence. A raw count or generated-label change
is attribution-only only when the patch/fingerprints and five-file manifest remain exact, every
source-level label still maps to the same four roots, no module owner/execution family appears, and
a fresh independent reviewer confirms semantic containment. An actual new call path, sixth file,
module owner, execution family, or authority source is `ImpactDecisionRequired`; stale-index
pinning is forbidden.

Route C proof requires transport API tests; focused and full world-service library tests; installed-
witness Host projection; World doctor typed-A JSON; conflicting ambient-B negatives; malformed or
missing typed-context fail-closed cases where applicable; warnings-denied Clippy for touched crates;
shell all-target compilation; workspace all-target check; formatting; and diff checks. The inherited
disabled-world doctor failure is first reproduced at the clean replayed Route B baseline and then
compared exactly; Route C may not fix, suppress, rename, weaken, or reinterpret it. The broad shell
differential requires `PassToFail = 0`, `NewFail = 0`, no removed/renamed/substituted/weakened tests,
unchanged normalized signatures for retained baseline failures, and a causal audit of every
`FailToPass` that excludes test bypass. Completion additionally requires fresh independent
authority/security, call-path/impact, and cross-platform/regression reviews. Route C remains
uncommittable until all are CLEAN.

`WorldDoctorReportV1` remains a world-service/world-enforcement report with additive optional,
defaulted, omit-when-absent host prefix and commitment fields. The in-world `doctor_world` producer
sets them to `None` and reads no host IH, carrier, HOME, prefix, principal, or authority projection.
Only the host shell may enrich final host-visible doctor output from typed IH, and it does so before
either JSON serialization or text rendering; the legacy Linux fallback may populate the same fields
only when typed IH is explicitly supplied. Old JSON without
the fields remains valid, and world enforcement, Landlock, netfilter, filesystem strategy, policy,
health, and service behavior remain unchanged.

For shell/shim doctor, `shim_doctor/mod.rs` performs name visibility only: on Unix it exposes the
existing `report::collect_report_for_context` with crate-private visibility so sibling `health.rs`
can continue the typed route. It does not make the report module public, add a wrapper or resolver,
or alter either collector. The existing crate-private `collect_report` compatibility re-export may
receive only an item-level Unix `unused_imports` allowance with a reason naming temporary R2-3
compatibility ownership. The `report.rs::collect_report` function may receive only an item-level
Unix `dead_code` allowance with the same temporary ownership. These annotations retain code only:
they do not change visibility, signatures, bodies, cfg branches, callers, outputs, environment
behavior, or non-Unix behavior, and no file-/module-level or other lint suppression is permitted.
`gather_world_deps_section`, `try_load_health_fixture`, `health_fixture_path`, and
`collect_doctor_snapshot_v1` retain the Route D/F5-owned A-rooted world-deps fixture/collector
closure. For nested World Doctor, the later historical F5-PD contract, whose boundary the
completed-F gate record retains, superseded the historical fixture-or-public-child behavior:
whenever Linux World-enabled production reaches
`gather_world_doctor_snapshot`, it invokes the authenticated hidden passive child, and
`A/health/world_doctor.json` is `cfg(test)` evidence only on that path. The World-disabled
short-circuit remains unchanged and spawns no child. Non-Linux compatibility remains
behavior-frozen and cannot satisfy proof.
The child receives the same canonical hidden argv carrier and context-derived checked projections;
no contextless repository-binary invocation may satisfy the path. The parent environment is not
changed. The child transport never formats the carrier into a command string, output, error, log,
trace, snapshot, or fixture, and no credential, request byte, non-public commitment, or sensitive
principal datum is added to diagnostics. World-deps selected-A fixture payload/source semantics
and non-Unix cfg behavior remain unchanged.

Unix `collect_report` is the explicitly named compatibility path: it retains its checked-projection
behavior unchanged, cannot be selected by typed Health, and cannot satisfy R2-2 product proof. Its
body, visibility, cfg, caller, output, and crate-private re-export remain frozen apart from the
already-authorized item-level lint retention. Physical-shim, global-trace, replay, and
platform-native compatibility migration remains R2-3. No diagnostic reconstructs a default home,
creates a second context constructor/resolver, or changes world, policy, capability, installation,
cleanup, service, credential, receipt, supervisor, retained-worker, or lifecycle semantics.

The Route D source-closure impact observations are LOW: `gather_world_doctor_snapshot` has one
direct/five transitive impacts, `try_load_health_fixture` two/four, `health_fixture_path` one/four,
and `run_json_subcommand` one/three. All resolve to the existing Health/shim-doctor family. These
counts are evidence rather than an exact ceiling; final containment is decided by the approved
source paths and symbols, unchanged owner/family set, exact manifest/patch, mapped GitNexus output,
and fresh read-only semantic review. A new execution family, authority owner, module, resolver,
schema, or R2-3 behavior requires a cross-document stop.

#### R2-2 source closure, E closeout, F0/F0a/F0b prerequisites, and remaining F contract

The failed Routes A–D integration closeout established that those four routes were individually
review-clean but did not close every authenticated-context consumer. R2-2E has since implemented and
proved the gateway projection without changing Routes A–D; all five routes are individually
review-clean. F0/F0a/F0b/F0-HC are now implemented, proof-complete, review-clean, canonically
closed out, and preserved. R2-2 stays incomplete pending F and the renewed integration closeout. The
following table preserves E's reviewed historical source-closure boundary
and freezes the completed F0/F0a/F0b boundaries plus F's remaining boundary;
brace groups are exact symbol sets, not file wildcards.

| Increment | File | Symbol | Pre-increment authority | Required carrier | Callers | Process family | Platform cfg | GitNexus risk | Inventory ID | Test | Reviewed allowlist disposition |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R2-2E | `crates/shell/src/execution/platform/mod.rs` | `handle_world_command` Gateway arm | typed carrier arrives but is dropped | already-authenticated A/request context | `ShellConfig::from_cli` | shell World dispatch | Unix authenticated; non-Unix fails before ambient selection | LOW; manual arm closure | PI-111 | `world_gateway.rs`; `agent_successor_contract_ahcsitc0.rs` | EDIT arm only |
| R2-2E | `crates/shell/src/builtins/world_gateway.rs` | `GatewayLifecycleRequestContext`; `run`; `run_inner`; `run_typed_action_with_status_args`; `run_typed_action`; `call_gateway_action`; `build_gateway_request_context` | ambient config/policy/network/inventory and pre-validation disabled routing | one validated A-derived gateway context | World Gateway status/sync/restart | gateway lifecycle | all; cfg clients below | LOW; struct one direct/five total | PI-111 | colocated; `world_gateway.rs` | EDIT |
| R2-2E | same | `validate_gateway_backend_selection`; `resolve_integrated_auth_payload`; `resolve_cli_codex_integrated_auth`; `codex_auth_state_path` | ambient inventory and `dirs`/HOME/USERPROFILE Codex home | explicit inventory plus committed-principal account home | `build_gateway_request_context` and auth helpers | gateway projection/credential handoff | Unix account leaf; cfg-coherent others | LOW | PI-111 | colocated; `agent_successor_contract_ahcsitc0.rs` | EDIT; credential content/lifecycle frozen |
| R2-2E | same | `world_routing_disabled` | ambient toggles can bypass failed context | validated A-derived effective config before classification | two typed action functions | gateway status/action | all | LOW; two direct/four total | PI-111 | colocated; `world_gateway.rs` | EDIT only after A validation |
| R2-2E | same | `synthesized_unavailable_response_without_context` | contextless unavailable response can bypass authentication | no contextless route | two typed action functions | gateway status/action | all | LOW; two direct/four total | PI-111 | colocated; `world_gateway.rs` | DELETE only |
| R2-2E | same | `build_gateway_client` Linux cfg | ambient `SUBSTRATE_WORLD_SOCKET` | fixed authenticated-route `/run/substrate.sock` | `call_gateway_action` non-macOS | gateway transport selection | Linux | LOW/graph under-resolved | PI-111 | colocated; `world_gateway.rs` | EDIT Linux cfg only |
| R2-2E | same | `build_macos_gateway_client`; `resolve_macos_gateway_client_endpoint`; `resolve_macos_host_gateway_socket`; `macos_default_world_socket_path` | ambient socket/home/VM and lower `auto_select`; endpoint helper is production-compiled dead/test support | no carrier: E fails in `call_gateway_action` before these symbols | `call_gateway_action` plus four test-only endpoint-helper callers | gateway transport selection | macOS | LOW locally; endpoint helper four test callers; lower adapter risk frozen | PI-111 | unchanged colocated cfg tests plus new pre-client rejection proof | SOURCE-CLOSURE INSPECTED, FROZEN R2-3; no edit/delete/call |
| R2-2E | `crates/shell/src/execution/policy_snapshot.rs` | `resolve_policy_snapshot_for_bootstrap_home`; additive explicit-bootstrap-home world-network resolver | snapshot resolver is explicit; network resolver re-enters ambient config | A bootstrap home and explicit config | gateway context; later F authenticated builder | policy/network projection | all | LOW | PI-111 | colocated | REUSE plus ADD; sole owner |
| R2-2E | `crates/shell/src/execution/agent_inventory.rs` | `resolve_gateway_backend_inventory_entry`; additive bootstrap-home gateway resolver; `load_effective_agent_inventory_for_bootstrap_home` | gateway resolver loads ambient inventory | A bootstrap home | gateway backend validation | runtime-family inventory | all | LOW | PI-111 | colocated | EDIT/ADD/REUSE |
| R2-2F0 | `crates/shell/src/execution/mod.rs` | existing `WORLD_ENV_LOCK`/`world_env_guard`; additive `WorldSocketTestGuard` and focused tests | shared reentrant lock exists but socket mutation/restoration is decentralized | exact `OsString`/absence plus same shared lock | shell library unit tests only | test harness | `cfg(test)` | HIGH historical 35 direct/70 total; fresh 13/19 | none; proof prerequisite | colocated prior/absence/panic/recovery/nesting/concurrency | ADD test-only helper/tests; existing helper body and production exports frozen |
| R2-2F0 | `crates/shell/src/execution/orchestrator_world_dispatch.rs` | 49 socket `EnvVarGuard::set_path` call sites and two manual restore blocks | local guards do not all acquire shared lock; manual blocks are not unwind-safe or exact non-Unicode restoration | `WorldSocketTestGuard` | named unit tests only | test harness | Linux `cfg(test)` module | graph-under-resolved; source closure binding | none; proof prerequisite | exact pair plus neighboring socket tests | EDIT test-only call sites; names/assertions and production symbols frozen |
| R2-2F0 | `crates/shell/src/execution/platform/macos.rs` | four test `with_env_var` socket call sites/readers | `#[serial]` and unlocked closure restoration | shared test guard around each complete closure | macOS unit tests | test harness | macOS tests | source reviewed | none; proof prerequisite | existing four tests plus guard proof | EDIT test helper/calls only; production platform adapter frozen |
| R2-2F0 | `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs` | two macOS tests with direct remove/set/remove | separate standard test lock; prior socket state not restored exactly | shared test guard retained through server/client cleanup | two unit tests | test harness | macOS tests | source reviewed | none; proof prerequisite | existing tests | EDIT test calls only; production persistent-session code frozen |
| R2-2F0 | `crates/shell/src/execution/routing/world.rs` | two direct socket mutations | shared lock is held, but mutation is not RAII-restored | shared test guard | two unit tests | test harness | macOS/Windows tests | source reviewed | none; proof prerequisite | existing tests | EDIT test calls only; production routing frozen |
| R2-2F0 | `crates/shell/src/builtins/world_enable/runner/paths.rs` | `resolve_world_socket_path_normalizes_relative_components` manual set/restore | `#[serial]` only; restoration is not unwind-safe | shared test guard | one unit test | test harness | tests | source reviewed | none; proof prerequisite | existing test | EDIT test call only; production path resolver frozen |
| R2-2F0 | `crates/shell/src/builtins/world_gateway.rs` | `classification_tests::with_env_var`; one Linux socket call | shared lock releases after panic without restoration because the closure helper restores only after normal return | shared test guard for the socket branch through the complete closure | one unit-test call; four separate RAII world-gateway calls unchanged | test harness | `cfg(test)` classification module | source reviewed | none; proof prerequisite | existing Linux fixed-socket test plus guard proof | EDIT test helper/call only; production gateway and four compliant RAII sites frozen |
| R2-2F0a | `crates/shell/src/execution/mod.rs` | generalize/reuse test-only `WORLD_ENV_LOCK` boundary for HOME plus socket; combined focused tests | socket guard candidate exists only on preserved branch; clean source has shared reentrant lock but decentralized HOME restoration | exact `OsString`/absence under one authority-environment lock | all same-process HOME/socket mutators and stable authority readers | test harness | `cfg(test)` | CRITICAL 35 direct/70 total | none; proof prerequisite | prior/absence/non-Unicode/panic/poison/nesting/concurrency/mixed-variable | EDIT/ADD test-only; no production export or behavior |
| R2-2F0a | `crates/shell/src/execution/agent_runtime/{auto_attach.rs,control.rs,state_store.rs,tool_invocation_contract.rs}`; `crates/shell/src/execution/host_inbox_materialization.rs`; `crates/shell/src/execution/agents_cmd.rs` | five `with_store` families (223 dependent tests) plus two agents-command helpers (five tests) | manual set/remove or local string guard; four callers are unannotated; restoration is not uniformly exact or unwind-safe | shared authority-environment guard held across complete helper callback | 228 dependent tests | test harness | shell library tests | tool-contract helper HIGH 12 direct; others MEDIUM/LOW or graph-under-resolved | none | exact HOME pair and all helper dependents | EDIT test helpers/calls only; names/assertions frozen |
| R2-2F0a | `crates/shell/src/builtins/{shim_doctor/report.rs,world_deps/mod.rs,world_enable/runner/manager_env.rs,world_gateway.rs}`; `crates/shell/src/execution/{agent_inventory.rs,config_model.rs,env_scripts.rs,invocation/tests.rs,orchestrator_world_dispatch.rs,routing/builtin/tests.rs,settings/tests.rs}`; `crates/shell/src/execution/agent_runtime/host_session_authority/store_tests.rs`; `crates/shell/src/repl/async_repl.rs` | remaining direct/local-guard/manual HOME mutators in the 435-test shell-library inventory | `#[serial]` and unrelated local guards do not exclude unannotated peers; some restore `String` rather than exact `OsString` | same authority-environment guard; dependent async/process lifetime ends before restore/unlock | remaining same-process mutating tests and stable readers | test harness | test modules only | source reviewed; large modules graph-under-resolved | none | combined HOME/socket neighbor and restoration wall | EDIT test-only sites/helpers; production symbols frozen |
| R2-2F0a | `crates/shell/tests/{shim_deployment.rs,agent_successor_contract_ahcsitc0.rs,support/mod.rs}` | parent-process HOME mutation in three independent integration source families | separate binaries; restoration is manual and not uniformly panic-safe/exact | equivalent per-binary exact-restoration boundary in every inventoried parent-mutating helper; no cross-process lock | nine serialized shim callers, one successor caller, one ignored one-test helper binary, one serialized support caller | integration test harnesses | existing cfgs | source reviewed | none | per-helper prior/absence/panic plus child inheritance | EDIT all three inventoried integration helper/call families; child-only `Command::env` files frozen |
| R2-2F0 | `crates/shell/src/execution/orchestrator_world_dispatch.rs` | nine named socket-owning tests with `server.abort()` and no awaited termination | task cancellation may remain pending while guard restores/unlocks and temp socket/root cleanup begins; one scheduler yield is insufficient | abort → await confirmed termination → complete/confirm fixture cleanup → restore environment → unlock | nine named tests in `03`/`05` | test harness | Linux test module | test symbols graph-under-resolved; source closure exact | none | termination-before-restore and zero task/socket/root leaks | EDIT nine test bodies only; no production server/readiness lifecycle change |
| R2-2F0b | `crates/shell/src/execution/agent_runtime/control.rs` | private Unix-only `PublicPromptRenderer::render`; ADD private explicit-writer core/adapter; DELETE `capture_stdout_once` and `capture_stderr_once`; two exact fallback tests | production render selects real stdout/stderr directly; tests replace process fd 1/2 with `dup2`, so parallel libtest reporter output can enter the private pipe | private explicit stdout/stderr writers for the core; production `render` delegates through real selected stream; tests own memory buffers | source-exact production callers `run_hidden_owner_helper_startup_prompt_stream_with_projection` and `run_public_prompt_command` remain frozen; two test callers migrate | production renderer plus test harness | Unix; helpers/tests under `cfg(test)` | `render` MEDIUM 4 direct/39 total/one process; generic-`new` HIGH is source-proven over-attribution and frozen; helpers/tests LOW | none; proof prerequisite | exact stdout/stderr bytes, stream selection, forced reporter overlap, private-buffer stress | EDIT `render` delegation and two tests; ADD private core/adapter; DELETE only two capture helpers; no caller/public API/production contract change |
| R2-2F | `crates/shell/src/execution/platform/mod.rs` | `handle_world_command` Doctor/Deps arms; `handle_host_command` Doctor arm | World doctor/deps resolve ambient config/policy or drop A; Host policy remains ambient | `AuthenticatedWorldDepsContextV1`-equivalent plus A-derived doctor inputs | `ShellConfig::from_cli` | World/Host dispatch | authenticated Unix/Linux; non-Unix compatibility unproven/no A claim | LOW; manual arm closure | PI-106/PI-107 | `doctor_scopes_ds0.rs`; world-deps suites | EDIT arms only |
| R2-2F | `crates/shell/src/execution/platform/linux.rs` | `host_doctor_main`; `world_doctor_main` | `detect_profile` plus global `world_fs_policy` | explicit A-derived world-fs policy and identity | platform handlers | Linux Host/World doctor | Linux | LOW | PI-106/PI-107 | `doctor_scopes_ds0.rs` | EDIT identity/policy projection only; public Doctor's active readiness/socket/service/endpoint/probe behavior stays unchanged |
| R2-2F | `crates/shell/src/builtins/world_deps/mod.rs` | new authenticated context/binder; `WorldDepsDoctorSnapshotV1`; `collect_doctor_snapshot_v1`; `resolve_effective_enabled_provisioning_requirements_v1` | diagnostic Unix path partly explicit; provisioning and non-Unix ambient; snapshot lacks identity | one validated A context with root/config/policy/deps/CWD/runtime projection | shim doctor; world-enable; surfaces | dependency resolution/diagnostics | Unix/Linux validation; non-Unix unavailable or explicit R2-3 ambient compatibility only | existing symbols LOW; new symbols N/A | PI-106/PI-107 | colocated; inventory/enabled/provision suites | ADD/EDIT |
| R2-2F | `crates/shell/src/builtins/world_deps/surfaces.rs` | `run`; `run_current`; `run_global`; `run_workspace`; `run_current_list`; `run_current_show`; `run_current_install`; `run_current_sync`; `run_global_list`; `run_global_add`; `run_global_remove`; `run_global_reset`; `run_workspace_list`; `run_workspace_add`; `run_workspace_remove`; `run_workspace_reset`; `resolve_global_available_inventory_view`; `build_current_show_explain_v1` | ambient H/config/deps and lower `current_dir` | shared context; explicit workspace CWD | world dispatch and post-provision sync | normal dependency CLI | symbols compile across existing cfgs; authenticated guarantee Unix/Linux only, other adapters R2-3/unproven or fail closed | LOW per entry/action | PI-106/PI-107 | all exact world-deps suites in `03` | EDIT exact symbols |
| R2-2F | same | `run_current_list_applied`; `compute_current_applied_items_v1`; `preflight_runtime_system_requirements_v1`; `probe_world_apt_requirements_v1`; `probe_world_pacman_requirements_v1`; `apply_install_plan_v1`; `reconcile_world_deps_bin_v1`; `apply_apt_entrypoint_wrappers_v1`; `apply_script_package_v1`; `resolve_script_body_for_package_v1`; `current_codex_runtime_target_triple_v1`; `run_world_command_output_for_deps`; `run_world_command_output_for_deps_with_profile`; `run_world_command_checked_for_deps`; `query_world_package_presence`; `query_world_package_entrypoint_presence`; `run_world_presence_check_v1`; `ensure_world_backend_available`; `run_world_command_for_deps`; `run_world_command_for_deps_at` | A inventory can feed ambient B request builder | shared context and authenticated request projection | current applied/show/install/sync and doctor | dependency runtime execution | symbols compile across existing cfgs; authenticated guarantee Unix/Linux only; profile-specific behavior frozen | LOW per symbol; frozen `resolve_current_inventory_view` HIGH | PI-106/PI-107 | applied/present/apt/dry-run/script/install suites | EDIT exact symbols; frozen reuse helper untouched |
| R2-2F | `crates/shell/src/builtins/world_enable/runner.rs`; `runner/provision_deps.rs` | `run_enable_with_provision_deps`; `run_sync_after_provisioning`; `probe_world_manager`; `probe_requirements`; `provision_apt_requirements`; `provision_pacman_requirements`; `execute_with_profile` | A reaches runner then is dropped before probes/install/sync | same authenticated world-deps context | `run_enable` and provisioning helpers | world-enable provisioning | Unix carrier; existing non-Unix restrictions | LOW | PI-106/PI-107 | `world_enable.rs`; `world_enable_provision_deps_wdap0.rs` | EDIT |
| R2-2F | `crates/shell/src/execution/routing/dispatch/world_ops.rs`; `routing/dispatch/prelude.rs`; `routing.rs` | one new authenticated builder/private cfg implementations and export-only re-exports | existing shared builder resolves config/policy/network/socket ambient | prevalidated F context reusing E projection | only `run_world_command_for_deps_at` and `execute_with_profile` | world execution request construction | cfg implementations | new N/A; frozen ambient builder HIGH, trace variant CRITICAL | PI-106/PI-107 | colocated world-ops plus world-deps/provision suites | ADD plus export-only; ambient builders FROZEN |
| R2-2F | `crates/shell/src/builtins/shim_doctor/report.rs` | `gather_world_doctor_snapshot`; `gather_world_deps_section`; `status_for_world_deps_report`; `snapshot_from_value`; `snapshot_from_command`; shared identity validator | fixtures/children can omit/mismatch A; missing `ok` may become healthy | expected non-secret A prefix/commitment | `build_report` | Health/shim-doctor composition | Unix typed path; non-Unix R2-3 compatibility/unproven and cannot claim A | LOW | PI-106/PI-107; PI-108 mechanics preserved | `common.rs`; `shim_doctor.rs`; `shim_health.rs`; doctor suites | EDIT coherence only |

R2-2E's binding contract is exact:

1. The gateway consumes an already-authenticated `InstallBootstrapContextV1`; missing, malformed,
   tampered, mismatched, wrong-principal, or unavailable A fails before classification or effects.
2. Existing explicit-home config and policy resolvers select A. `policy_snapshot.rs` owns the only
   policy snapshot and network-policy projection; duplication inside `world_gateway.rs` is forbidden.
3. `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, CWD, account-home variables, XDG, manager projections,
   `CODEX_HOME`, `.codex`, `config.toml`, and platform-control roots never replace A. CWD may be an
   explicit workspace-scope input only. Runtime-native files remain projections.
4. Network allow/deny semantics and gateway lifecycle behavior are unchanged; only selection
   authority becomes exact. Credentials remain launch-time in-world handoff under the existing
   contract, and no durable host credential authority is created.
5. Linux uses the fixed socket. E has no authenticated macOS endpoint source, so macOS fails before
   client construction or ambient forwarding; Windows/other contextless entries fail before any
   ambient selector. Existing non-Linux clients remain frozen/unreachable from E. No
   `world-mac-lima` edit or native macOS/Windows claim is authorized.
6. Output may include only intended non-secret references/commitments. Carrier, prompt/request,
   credential/token, and sensitive-principal bytes are forbidden everywhere observable.

Its required tests cover custom A with conflicting B; A-only config, policy, network policy,
inventory/runtime-family, and Codex projection; environment-only/malformed/tampered input;
fail-before-mutation/forwarding/launch; disabled-routing validation; disclosure scans; unchanged
network allow/deny behavior; Linux fixed socket; macOS/Windows/other fail-before-ambient selection;
and non-Unix build/static cfg preservation without an A-bound/native proof claim.

R2-2E is now **implemented, proof-complete, review-clean, committed, and preserved** with this exact
evidence:

- commit `7e8e83802885c0ece93efcaacccc26503eeb6715`; tree
  `02a1a2f6b7e4be47ed9ec38537c805ae348c96b6`; ordinary patch SHA-256
  `fb7b65b02cdac46857750b64ebd5ceab651e8e99910c05240b187c3e729444ff`; full-index patch SHA-256
  `c712f467ac92efb0de7b524272479741ac6b318201cf45dbd994116ec0e8b862`; preservation branch
  `feat/preserve-a1-1d-5r2-2e-c712f467`;
- exact manifest: `crates/shell/src/execution/platform/mod.rs`,
  `crates/shell/src/builtins/world_gateway.rs`,
  `crates/shell/src/execution/agent_inventory.rs`,
  `crates/shell/src/execution/policy_snapshot.rs`, and `crates/shell/tests/world_gateway.rs`;
- changed ownership is confined to the Gateway arm's authenticated binding, the existing gateway
  request/projection call chain, additive explicit-bootstrap-home inventory and network projection,
  and its tests. The sole deleted symbol is
  `synthesized_unavailable_response_without_context`; canonical config/policy resolver ownership,
  network allow/deny meaning, and Routes A–D bytes are unchanged;
- focused results: gateway classification 13/13; config resolution 21/21; effective policy 14/14;
  policy snapshot/network 10/10; inventory 17/17; install-bootstrap context 8/8; explicit
  `HostSessionAuthority` composition 1/1; new integration negatives 3/3; managed auth bundle 7/7;
  world-service gateway runtime 32/32; gateway receiver/server 18/18;
- the **clean Route D comparison baseline** is `1101 passed / 149 failed / 0 ignored`; final E
  produced the genuine **post-E pre-F0 success observation**, `1114 passed / 149 failed / 0
  ignored`.
  `PassToFail=0`, `NewFail=0`, `FailToChangedFailure=0`, and `FailToPass=0`; the 149 failure-name set
  is identical, with only the recorded nondeterministic orchestration identifiers normalizing. The
  same runtime can instead produce the known interference observation `1113 passed / 150 failed /
  0 ignored`; neither pre-F0 result is an F comparison baseline;
- final GitNexus output is semantically contained to the approved owners and existing process
  families. Its aggregate CRITICAL adjacency is diagnostic over-attribution; each exact edited
  existing symbol was LOW, and no new process family, resolver owner, schema, or capability was
  introduced;
- final isolated read-only reviewers `e_final_gateway_authority`, `e_final_policy_network`,
  `e_final_credential_boundary`, and `e_final_platform_regression_replacement` returned CLEAN. The
  original platform reviewer is excluded because it violated the required read-only process;
- Linux fixed-socket and regression proof is complete. macOS fails before ambient client/forwarding
  selection, while Windows/other entries fail before ambient selection; static cfg preservation is
  recorded, but no native macOS/Windows or privileged R2-4 product proof is claimed.

PI-111's **R2-2E implementation/proof clause only** is complete. This does not close the full
gateway credential/config architecture: the managed-gateway secure-FD producer, receiver, bundle
schema, and lifecycle are landed, regression-proven, and unchanged by E, while direct-member
Codex/UAA gateway adoption remains unresolved, transitional compatibility, non-promotable, and
owned by E3/D1/D3. `RG-CONFIG-02`, `RG-CONFIG-04`, `RG-UAA-02`, and `RG-UAA-03` remain open and
unchanged. E changes no user/world capability, policy meaning, credential transport, or service
lifecycle and promotes no seam.

**A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation** joins R2-2F0 under this exact binding contract:

At this historical authorization checkpoint, F0/F0a were incomplete and test-harness-only. Their
exact pre- and post-fork-remediation candidates were preserved rather than landed. Focused proof was green (socket pair 100/100 parallel and 20/20 serial;
four-test neighbor matrix 20/20; prior absence, non-Unicode value, panic/reacquisition, nesting, and
concurrent exclusion all pass), but its final broad runs were `1118 passed / 150 failed` and `1119
passed / 149 failed`. The extra failure is
`dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`, normalized as
`resolve exact B-owned acceptance authority: open activated versioned authority layout`. It passes
alone. F0a remains an authorized part of the combined candidate, not a waiver of F0 proof. The
later exact combined candidate has now passed restoration, corrected differential proof, fresh
containment review, canonical closeout, and runtime preservation.

1. The exact HOME minimal pair is the target above plus
   `prompt_submit_continuity_prefers_persisted_session_contract`. In canonical same-process
   `--exact --test-threads=2` runs it failed 20/20. The stable same-process
   `--test-threads=1` harness controlled competitor-then-target and failed 0/10; separate-process
   sequential runs passed both directions. Stable libtest could not force reverse same-process
   order, so no reverse same-process result is claimed. Non-source tracing captured the competitor setting a private HOME,
   the target installing its own private HOME, then the competitor removing HOME before the target
   opens its authority layout. Three other unannotated HOME mutators independently reproduce 20/20.
   Commit `f5a150f94d585b1f55ec0067845cd5d715773c78` first adds the selected competitor.
   `83101dcbcc750e6e8fb8979bea19f1f777792188` later adds the target and its HOME-mutating fixture
   and is the first source commit where the exact pair coexists. This remains
   `TestIsolationDefectConfirmed`, not a production defect.
2. One `cfg(test)` RAII authority-environment guard acquires the existing shared reentrant lock and
   jointly protects HOME and world socket. It captures exact prior `OsString` or absence, installs
   the test value or absence, spans dependent socket/server/helper/child/process work and cleanup,
   restores on normal return and panic unwind, and releases only after restoration. At least 88
   tests depend jointly on HOME/socket, while existing helpers acquire them in opposite orders;
   separate locks are forbidden because they admit mixed snapshots and lock-order inversion.
3. Same-thread nesting must restore in stack order without deadlock. Poison/non-poison behavior and
   later reacquisition are explicit; no panic may strand a value or silently bypass the lock.
   `#[serial]` is supplemental only. Stable readers participate whenever source evidence requires
   one authority snapshot. Child environment inheritance is intentional and bounded; integration
   binaries use their own process-local disposition and no cross-process lock.
4. Source closure identifies 435 shell-library mutating test functions across 19 files. Five
   `with_store` families cover 223 dependent tests; two agents-command helpers cover five. The four
   unannotated mutators are the two control continuity and two agents-command toolbox-status tests.
   The table above is the complete test-only allowlist. Production-only non-Unix world-enable HOME
   mutation remains frozen; any test invoking it must hold the test boundary externally.
5. Nine named F0 orchestrator fixtures own an async server and guarded socket but abort without
   awaiting termination. They must execute `abort` → await confirmed task termination → complete
   and confirm fixture-owned socket/task cleanup → restore environment → release lock. Reverse
   declaration/drop order and `yield_now` do not prove termination. No production server lifecycle,
   readiness semantic, retry, or world-service behavior changes.
6. Combined focused proof reruns the F0 socket pair at least 100 parallel/20 serial and the F0a HOME
   pair at least 100 parallel/20 serial, then a mixed HOME/socket neighbor matrix. It proves exact
   prior value/absence/non-Unicode restoration, panic recovery, poison behavior, nesting, concurrent
   exclusion, bounded child inheritance, async cleanup ordering, and zero leaked task/socket/helper/
   temp-root state. The deliberate retained-registration conflicting child remains internal while
   its parent passes; retry validation is never weakened.
7. Combined F0/F0a/F0b broad proof is at least three independent exact final-candidate default-parallel shell
   walls and one canonical single-thread shell wall. Failure names and normalized signatures are
   identical; no test is removed, renamed, substituted, weakened, ignored, or made
   environment-authoritative; no unexplained count variance is accepted. Added passing tests have
   an explicit count delta. Only combined F0/F0a/F0b closeout records the deterministic F
   comparison baseline.
8. Production `std::env` readers, HOME/world-socket resolution, readiness, retries, policies,
   capabilities, world-service behavior, retained-worker state, credential transport, and the
   managed secure-FD path are frozen. F0/F0a alter no production byte. A required production change
   stops as `CrossDocumentChangeRequired`.

**A1.1d-5R2-2F0b — deterministic renderer-output test isolation** is bound by this exact
test-infrastructure contract:

1. Primary classification is `TestIsolationDefectConfirmed`. The exact post-fork parallel walls
   were `1134 passed / 146 failed / 0 ignored`, `1134 passed / 146 failed / 0 ignored`, and
   `1133 passed / 147 failed / 0 ignored`. Only wall 3 added
   `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails`, so canonical
   closeout is invalid. The helper redirects process fd 1 with `dup2`; libtest's parallel reporter
   wrote through the same descriptor. Exact captured bytes were
   `".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`.
2. Causal stress is binding: forced same-process parallel passed 376 and failed 124 of 500 with one
   normalized signature; isolated, identical-neighbor same-process serial, and separate-process
   controls each passed 100/100; parallel pretty reporting passed 99/100. Candidate introduction
   is unnecessary because the capture helper and target test are byte-identical to clean E. This
   is neither random flakiness nor a production renderer defect.
3. Future implementation may edit only
   `crates/shell/src/execution/agent_runtime/control.rs`. It may mechanically delegate
   `PublicPromptRenderer::render` to a new private Unix-only explicit-writer core or equivalent
   private sink adapter, delete `capture_stdout_once` and `capture_stderr_once`, and migrate only
   `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails` and
   `public_prompt_renderer_renders_bounded_structured_stderr_fallback_when_decode_fails`. No other file or symbol
   is authorized.
4. `PublicPromptRenderer::render` remains the production entry point with its signature frozen.
   `PublicPromptRenderer::new` and the bodies of
   `run_hidden_owner_helper_startup_prompt_stream_with_projection` and
   `run_public_prompt_command` remain frozen. Production delegates through real stdout/stderr,
   chooses and locks only the selected stream at the same point, and preserves JSON-envelope
   stdout, completed/normal-event stdout, warning/failure/stderr-event stderr, byte ordering,
   newlines, flushes, serialization/write error propagation or intentional suppression, redaction,
   and bounded fallback exactly.
5. Tests supply distinct in-memory stdout and stderr writers and assert the complete exact bytes
   emitted to the selected buffer plus an empty nonselected buffer. They may not strip, search
   around, or tolerate unrelated prefixes. The same-owner stderr helper is structurally unsafe and
   is migrated even though stdout alone appeared in the broad wall. Raw fd replacement remains
   forbidden as the final mechanism.
6. No public output API or transport/schema, process-global output lock, writer registry, side
   table, environment-selected sink, eager dual-stream lock, reporter suppression, sleep, retry,
   larger timeout, thread reduction, whole-suite serialization, ignore, removal, rename,
   substitution, or assertion weakening may satisfy F0b. `#[serial]` may remain supplemental but
   cannot exclude libtest reporter writes.
7. GitNexus reports `PublicPromptRenderer::render` MEDIUM (four direct, 39 total, one
   `handle_agent_command` process family; Agent_runtime direct and Execution indirect).
   `PublicPromptRenderer::new` reports HIGH (19 direct, 43 total, two process labels, three
   modules), but exact source closure proves generic-`new` over-attribution and freezes its body.
   The renderer type, capture helpers, and tests report LOW/zero-process. No CRITICAL impact, new
   production process family, or changed production process semantics are authorized.
8. Required proof includes both exact tests isolated; at least 100/100 exact renderer-pair parallel
   and 20/20 serial; forced reporter overlap; exact private stdout/stderr bytes; unchanged
   JSON/completed/warning/failure/event selection, order, newline, flush, and error behavior; then
   every combined F0/F0a/F0b focused, compile, lint, format, and four-wall gate. Production,
   user-facing, world, policy, credential, secure-FD, gateway, receipt, supervisor, worker,
   placement, caging, lifecycle, and capability behavior remains unchanged. No seam is promoted.

The following is the historical pre-F5-PD R2-2F binding contract; the later historical F5-PD
contract, whose boundary the completed-F gate record retains, superseded its immediate-next and
nested-doctor side-effect statements:

At that checkpoint F was the **exact next increment** after the then-complete corrected
differential proof, fresh containment review, and harness closeout, but it was not active or
implemented. It starts from the post-closeout replayed runtime commit and tree recorded by the
dedicated preservation ref and completion checkpoint. Routes A–E and F0/F0a/F0b are immutable prior
evidence. Its clean comparison baseline is the deterministic value recorded by combined closeout,
not either genuine but nondeterministic post-E observation (`1114 passed / 149 failed / 0 ignored`
or `1113 passed / 150 failed / 0 ignored`), either F0-candidate broad result (`1118/150` or
`1119/149`), any post-fork candidate wall (`1134/146`, `1134/146`, or `1133/147`), the
**R2-2 historical starting baseline** (`1089 passed / 149 failed`), or the
**clean Route D comparison baseline** (`1101 passed / 149 failed / 0 ignored`). A renewed
production-fix-free Routes A–F
integration closeout follows F; R2-3, R2-4, and R3 remain after that and are unstarted.

1. Normal current, global, workspace, runtime, provision-deps, and post-sync paths receive the one
   shared authenticated context. A selects global config/inventory/dependency roots; explicit launch
   CWD selects workspace scope only. No normal product path re-enters ambient resolution.
2. The additive authenticated request builder is consumed by exactly two leaves and reuses E's
   projection. The existing HIGH/CRITICAL ambient builders, `resolve_current_inventory_view`,
   resolver bodies, schemas, services, lower adapters, and compatibility callers remain frozen.
3. World-deps mutation/readiness/execution behavior is unchanged except for selecting the correct
   A root. Validation precedes every read that can influence mutation and every effect. Only the
   F5-PD nested World Doctor child is guaranteed non-mutating.
4. On the authenticated Unix/Linux route, doctor may identify A only when configuration,
   policy/world-fs policy, inventory, dependency,
   runtime observation, fixture, and child evidence all match A. Missing `ok` or identity, mixed
   A/B, or mismatch is unavailable/incoherent with `ok=false`, never success. Rejected reports are
   not retained as healthy payloads.
5. macOS, Windows, fallback, and other non-Unix world-deps/Health/doctor routes remain explicit
   R2-3 compatibility/unproven paths. They preserve labeled ambient compatibility or report
   unavailable/fail closed, but never claim A-bound success or consume the F context.
6. Missing/malformed/tampered/mismatched context fails closed. No carrier, credential, request,
   prompt, token, or sensitive-principal material appears in output, errors, fixtures, logs, or
   traces. Filesystem, network, caging, placement, service, receipt, supervisor, retained-worker,
   and lifecycle capabilities do not change.

Its required tests cover conflicting A/B for current/global/workspace/runtime; mutation and
nonmutation; current applied/show probes; apt/pacman/runtime installs; manager probes and
post-provision sync; direct no-witness and tamper failures before mutation; exact fixture/child
identity; mixed-source rejection; truthful unavailable output; existing A-rooted snapshot behavior;
explicit compatibility; disclosure scans; and non-Unix build/static preservation with no A-bound
success claim. The broad comparison is against the deterministic F baseline recorded by combined
F0/F0a/F0b closeout and requires
zero pass-to-fail, new-fail, changed-failure, removed, renamed, substituted, weakened, or newly
ignored tests plus the identical retained 149-name failure set.

The binding stop conditions for E, F0, F0a, F0b, and F are: any file/symbol outside `03`'s exact allowlist;
any new resolver or side table; any edit to a frozen HIGH/CRITICAL builder; any change to another
shared caller, network meaning, request schema, service, lower platform adapter, capability,
credential lifecycle, cleanup, or deletion; any mutation before validation; any mixed-authority
healthy diagnostic; any non-Linux compatibility path presented as E/F-authenticated; or any
privileged/native platform proof claim without evidence. Encountering
one stops as `ImpactDecisionRequired` or `CrossDocumentChangeRequired`; it never silently widens an
increment.

For this exact Route A closure, the immutable production baseline is fixed at patch SHA-256
`ea4abf43043013994e698d54f21c04bba58833b3bb3247bf6d5d3b454f51b943`. Its manifest and
file SHA-256 fingerprints are:

| File | SHA-256 |
|---|---|
| `crates/shell/src/builtins/health.rs` | `997f138b72dce898cc707c128bf726e0f882da7f3a0c1cdcc3ff71e9b94e7405` |
| `crates/shell/src/builtins/shim_doctor/mod.rs` | `3035df6e3bca693c5e8c10e5f5cfee3fd1e1a6ad57d40c443979d25282bda748` |
| `crates/shell/src/builtins/shim_doctor/report.rs` | `0337379b8c4f1ce5169a60eb099b4d66cf7e9dfc32cc9f314ca4194ddd6f0b60` |
| `crates/shell/src/execution/config_cmd.rs` | `ef102e015759a784c8601a9cc4a8d2c220c467679f3eac111bb87f527eca12c6` |
| `crates/shell/src/execution/config_model.rs` | `42b7e851f7c3a63f3c739295bd12bacca1e51f4ecf9fb7037e919a3ea1985a07` |
| `crates/shell/src/execution/invocation/plan.rs` | `cc8a9d52db3bad5d571f64012ca37ec2b6219757b47d1802e91cae804c730c01` |
| `crates/shell/src/execution/platform/mod.rs` | `b93601f17f120bc3c28b544eb569b671750224b741d893ed0eb2d02252f04274` |
| `crates/shell/src/execution/policy_cmd.rs` | `a9f90acf883b264d0b76bbda4ca5dc9b5fdcf42f0d3119c49843981086a25d7b` |
| `crates/shell/src/execution/policy_model.rs` | `a04940b54c68d284dcc1ed65245180676b2822dbe596386e6bbba544507f2931` |
| `crates/shell/tests/config_show.rs` | `7a9819b02b507c35ef289a492cbd9c9b3e75ba462644e02914b1959b4de0f444` |
| `crates/shell/tests/policy_discovery.rs` | `7bad42c95925c1fb8f74c277e37710c8c8d69d2e866f7d4eb4f34a65df6226d1` |

The review-remediation successor preserves every production hunk above byte-for-byte. It may differ
from the base only by the following test delta:

| File/symbol | Sole successor-only change | Review finding |
|---|---|---|
| `config_model.rs::tests::explicit_bootstrap_home_explain_uses_selected_global_config_path` | Add item-level `#[cfg(unix)]` to the test and its sole test-only `HostSessionAuthority` import; preserve the name, body, assertions, fixtures, every other import, module cfg, and all production bytes. | Unix-only `HostSessionAuthority::open` must not make the test fail on unsupported non-Unix authority-store implementations, and the gated import must not trigger warnings-denied unused-import failure. |
| `policy_model.rs::tests::explicit_bootstrap_home_explain_uses_selected_global_policy_path` | Add item-level `#[cfg(unix)]` to the test, its sole test-only `HostSessionAuthority` import, and the test module's sole `tempfile::TempDir` import; preserve the name, body, assertions, fixtures, every other import, module cfg, and all production bytes. | Same cross-platform cfg/import correction for the policy test, including the warnings-denied import made unused by the item-level test gate. |
| `crates/shell/tests/shim_health.rs` focused Route A test | Add one focused regression or bounded strengthening that selects A through typed IH while H/R/HOME name conflicting B, proves A-derived Health state, uses syscall/path interception that fails on any B operation, compares a complete B-tree snapshot before/after, preserves classification/output assertions, and cannot be satisfied by `collect_report`. | Existing Health proof did not conflict ambient B or independently exclude the compatibility collector; mode bits and selected-file assertions alone do not prove absence of metadata probes or writes elsewhere under B. |
| `crates/shell/tests/doctor_scopes_ds0.rs` focused Host test | Add one focused regression or bounded strengthening using the canonical R2-1 installed-product invocation witness for A, no explicit selector, conflicting ambient B, A-derived Host/config output and expected dependency scaffold, absence of B dependency artifacts, complete B-tree preservation across the installed-witness dispatch, and complete B-tree preservation across the direct-repository no-witness fail-before-mutation case. | Existing Host test omitted both selector and witness and therefore never reached the changed typed-IH route; selected config bytes alone do not prove dependency routing or complete fail-before-mutation behavior. |
| `config_show.rs::config_current_show_uses_declared_prefix_under_conflicting_ambient_home` | Replace only the `/run/user/<effective uid>` fallback used to parent its unique selected-A root: use nonempty `XDG_RUNTIME_DIR`; otherwise resolve the current account's nonempty home and use `.cache` or a repository-established fixture subdirectory beneath that same account home; create the parent as needed; keep the selected root `0700`; and fail setup explicitly if neither source exists. The account home is current-account-derived, not the command's ambient/conflicting HOME. Preserve the name, cfg, command, A/B fixtures, config bytes, assertions, output, and production route. | The existing Unix test-parent fallback is Linux-specific and fails on macOS when `XDG_RUNTIME_DIR` is absent; a portable owner-controlled parent is required without weakening conflicting-B authority proof. |
| `policy_discovery.rs::policy_current_show_uses_declared_prefix_under_conflicting_ambient_home` | Apply the identical bounded secure-parent replacement while preserving the name, cfg, command, A/B fixtures, policy bytes, assertions, output, and production route. | The same Linux-only fallback makes the policy proof nonportable even though its Route A authority assertions are otherwise valid. |

The preserved current successor before the final test-parent correction is exact patch SHA-256
`0a53e8b60326e21b4237392bfa99671fa8c19c0cd45eb4c573cae5140e808720` at preservation commit
`ad139789ef317b978e96c36ee8170dd503bff8a4`, with exactly thirteen files. The two portable-parent
tests already belong to that manifest, so the corrected candidate remains exactly thirteen files;
all production fingerprints and all other test fingerprints remain unchanged. Its new patch hash is
computed only after those two authorized hunks are applied. Before each existing test symbol is
edited, run fresh upstream impact analysis; the recorded impacts for both parent-selection tests are
LOW with zero callers, processes, or modules. After implementation, create a deterministic
base-to-successor delta containing the new ordinary/binary patch SHA-256, exact manifest and
fingerprints, and a hunk table mapping every successor-only line to one row above. Existing tests
cannot be removed, renamed, substituted, ignored, or weakened; the two tests remain active on Unix,
and only their intentional non-Unix cfg exclusion is authorized. Any changed production byte, changed
base hunk outside the item-level cfg attributes and the two exact portable-parent setup hunks, or
successor-only hunk that does not map exactly to one of the six authorized delta rows is
`SuccessorPatchScopeMismatch`.

For both final parent-selection hunks, empty environment values are unavailable rather than paths.
The allowed precedence is nonempty `XDG_RUNTIME_DIR`, then the current account's nonempty home under
`.cache` or a repository-established fixture subdirectory beneath that same account home. The latter
is not a third source, and the command's ambient/conflicting HOME does not select it. The test creates
that parent if needed and creates its unique selected-A root beneath it with owner-only `0700`. It
fails with an explicit setup error when neither source is usable. It never falls back to `/tmp`, `/var/tmp`,
`/run/user/<uid>`, CWD, ambient `SUBSTRATE_HOME`, or B. This changes fixture placement only: explicit
A still wins, B remains conflicting and unread, and the typed Config/Policy production route and all
assertions remain identical.

GitNexus observations over the identical or mechanically extended Route A bytes are diagnostic
provenance: 33 symbols/18 process labels; 11/18 in the reverse committed comparison; 37/18; 39/18;
and, after replay and a current-index refresh, CRITICAL with 24 attributed symbols and 31 labels.
The incompatible raw counts and generated label sets are not stable semantic authority for this
exact patch. Do not pin or restore a stale index.

Base containment is determined in this order: exact patch bytes; the exact manifest and fingerprints
above; the exact source-level production functions and permitted tests. Successor containment then
requires byte-identical production hunks plus the deterministic six-row test delta above; manual call-path mapping to
the approved `ShellConfig::from_cli` Host/Health/Config/Policy/context-aware shim-doctor roots; no
new source module, authority source, call path, execution-family root, side table, environment
fallback, or lifecycle behavior; and fresh independent semantic-diff review. The exact production
diff is limited to `ShellConfig::from_cli`, `handle_host_command`, `handle_health_command`,
`health::run`, the crate-private shim-doctor exports, the item-level `collect_report` annotation,
Config's handler/current-show and two explicit-bootstrap-home resolver functions, and Policy's
handler/current-show and two explicit-bootstrap-home resolver functions. Permitted test changes are
the mechanical Config test carrier, the two explicit-home explain tests, the two focused integration
tests, and the secure-parent setup only in the named `config_show.rs` and `policy_discovery.rs` tests
already in the manifest. No other production function changes behavior.

Every fresh GitNexus result must be retained and every label mapped to one of
`AuthorizedRouteA`, `LineOrHunkAttributionOnly`, `TestColocationOnly`, or
`UnexpectedSemanticPath`. Count/label drift alone does not reopen the docs when the base production
bytes, successor test delta, fingerprints, source-level diff, mapping, module set, and semantic roots
remain exact. The six review-remediation edits restore ordinary pre-edit impact and fresh
change-detection requirements but need no new raw-count ceiling. Any `UnexpectedSemanticPath`,
production-byte difference, unclassified test difference, new module or semantic execution root,
authority source, or behavior expansion is an `ImpactDecisionRequired` stop. This rule is specific
to the exact Route A base plus its bounded test-only successor and cannot
apply to Route B–D, R2-3, another HIGH/CRITICAL increment, environment/global authority,
physical-shim migration, product capability, output, repair, cleanup, rollback, or lifecycle
behavior.

The current refreshed 31-label attribution audit is:

| Current label | Root symbol/module | Route A family | Source diff touches behavior? | Classification |
|---|---|---|---|---|
| `Handle_host_command → AuthorityFacadeError` | `platform::handle_host_command` | Host | Yes; typed IH selects the opened global-config root and this trace follows the new explicit-home resolver. | `AuthorizedRouteA` |
| `Handle_host_command → New` | `platform::handle_host_command` | Host | Yes; typed IH selects the opened global-config root and this trace follows the new explicit-home resolver. | `AuthorizedRouteA` |
| `Run → Lookup_unix_account_by_uid` | `health::run` | Health/shim-doctor | Yes; Unix Health now calls the typed context-aware collector and validates the committed principal. | `AuthorizedRouteA` |
| `Run → As_path` | `health::run` | Health/shim-doctor | Yes at the typed collector root; the reported configuration leaf is unchanged. | `AuthorizedRouteA` |
| `Run → WorldDisableAttribution` | `health::run` | Health/shim-doctor | Yes at the typed collector root; diagnostic classification is unchanged. | `AuthorizedRouteA` |
| `Run → WorldDisableSource` | `health::run` | Health/shim-doctor | Yes at the typed collector root; diagnostic classification is unchanged. | `AuthorizedRouteA` |
| `Run → As_str` | `health::run` | Health/shim-doctor | Yes at the typed collector root; the reported formatting leaf is unchanged. | `AuthorizedRouteA` |
| `Run → Validate` | `health::run` | Health/shim-doctor | No; GitNexus joins the Unix typed Health root to the cfg-incompatible compatibility collector. | `LineOrHunkAttributionOnly` |
| `Run → Current_dir` | `health::run` | Health/shim-doctor | No; the generated trace enters the compatibility collector before the typed collector, which typed Unix Health cannot do. | `LineOrHunkAttributionOnly` |
| `Run → CliConfigOverrides` | `health::run` | Health/shim-doctor | No; the generated trace enters the compatibility collector before the typed collector, which typed Unix Health cannot do. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → ActionableError` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged workspace-set leaf attributed through the handler signature hunk. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → New` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged workspace-set parse-error leaf attributed through the handler signature hunk. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Parent` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Write_all` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → As_str` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → Workspace_legacy_settings_path` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → For_path` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → Resolve_replace` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → ConfigExplainKey` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → I64ClampInfo` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → ConfigExplainSource` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Exists` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-init existence leaf. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Print_patch` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-show leaf, not current-show. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Parent` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Write_all` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → As_bytes` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-init hashing/write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Exists` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged global-init existence leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Invalidate_policy_snapshot_cache` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged global-init cache invalidation leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Print_patch` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged global-show leaf, not current-show. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Current_dir` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-show leaf, not current-show. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → As_path` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |

The three labels recorded in the prior 18-label output but absent after refresh are also mapped:

| Historical label now absent | Root/family | Source-level account | Classification |
|---|---|---|---|
| `Run → Encode` | `health::run`; Health/shim-doctor | No Route A source hunk encodes a carrier; this was generated attribution breadth. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → WorldDisableAttribution` | `platform::handle_host_command`; Host | Typed IH changes selected config input; the diagnostic classification leaf remains unchanged. | `AuthorizedRouteA` |
| `Handle_host_command → WorldDisableSource` | `platform::handle_host_command`; Host | Typed IH changes selected config input; the diagnostic classification leaf remains unchanged. | `AuthorizedRouteA` |

##### Hidden installer bootstrap action

`--install-bootstrap-home-v1` is a hidden internal action, not a public user feature. It is accepted
only when the same invocation includes
`--install-bootstrap-context-v1 <exact-carrier>`, whose argv presence remains the sole
internal-child discriminator. An environment carrier or matching H/R tuple alone cannot select the
action or internal mode. Before mutation, the root dispatcher strictly decodes and authenticates the
V1 carrier and commitment, binds its Unix account+UID to the current principal, and requires checked
H/R/principal/commitment projections to match typed IH. Missing, malformed, reordered, tampered,
conflicting, or forged context fails before any scaffold or other write.

After validation, the action calls only the existing explicit-context private-home/dependency
scaffold bootstrap, preserves all R1 owner/mode/ACL/no-follow/stable-identity/no-repair semantics,
and returns without entering the REPL or command dispatcher. Exact retry is idempotent. The action
does not deploy shims, provision runtimes, enter a world, evaluate policy, clean up, repair, delete,
roll back, migrate, replace, or converge anything, and it emits no secret or carrier payload in
normal output, errors, logs, or traces. Installers use this action instead of `--version`; focused R1
private-home tests may use it only under the narrow R2-1 test authorization. World-deps command
processing remains owned by R2-2.

Verified installed-product self-derivation first recovers the actual product invocation witness
with the same explicit-path or unique absolute-PATH-candidate algorithm used for shims below, but
with the exact command name `substrate[.exe]`. Zero or multiple matching candidates fail; PATH order
does not choose one. It accepts exactly these source-grounded shapes:

1. an exact no-follow witness `A/bin/substrate` on Unix that resolves to the running executable;
   the target may be the release `A/versions/<nonempty-version>/bin/substrate` or the current dev
   installer symlink target outside A;
2. an exact physical Windows witness `A\bin\substrate.exe` whose file identity is the running
   executable; the release install's copied `A\versions\<version>\bin\substrate.exe` is a payload
   projection and need not have the same file identity; or
3. a physical Unix release executable at
   `A/versions/<nonempty-version>/bin/substrate` for which `A/bin/substrate` exists and resolves to
   that exact physical file.

For every shape, A and `A/bin` (and the version ancestors in shape 3) pass the host no-follow
owner/path checks before A is normalized/accepted. The invocation witness selects A; an install
record, version file, profile, H/R, CWD, or executable target parent can only be checked afterward
and cannot supply A. A direct repository build with no exact A/bin witness, an arbitrary witness
outside these shapes, a missing/mismatched public link, or an ambiguous PATH candidate requires the
declared public prefix. Windows normal product proof invokes the copied A/bin executable; direct
execution from the Windows version payload requires an explicit prefix. Tests cover Unix release,
Unix dev symlink, Windows release copy, A under conflicting ambient B, zero/multiple witnesses, and
direct-repository rejection. This is one invocation-witness constructor, not a generated-file or
executable-parent precedence table.

Verified shim-layout self-derivation must remain implementable with the current Unix symlink and
Windows-copy deployment shape; R2 does not change shim replacement/deployment ownership. If
`argv[0]` is an absolute pathname, or contains a separator and therefore names an explicit relative
invocation pathname, resolve only that pathname (using CWD solely to resolve its explicit relative
spelling) and validate it. If `argv[0]` is a bare command, enumerate only absolute, nonempty PATH
entries and collect every `entry/<command>` whose no-follow metadata is an installed shim link/copy
and whose resolved file identity is the running `substrate-shim` target. Exactly one candidate must
exist. Zero or multiple candidates fail closed; PATH order is never a prefix-precedence rule and a
different earlier executable is not a candidate merely because the shell could have selected it.
The accepted witness must be exactly `A/shims/<nonempty-command>` and its A/`shims` ancestors must
pass the applicable no-follow owner/path checks. CWD, PATH, inherited H/R, or a carrier cannot supply
a fallback A after witness failure. This uses PATH/CWD only to recover the invocation witness that
the current symlink form otherwise loses; it creates no general prefix selector.

The current principal must reproduce the resulting host context. After that check, shell manager
initialization and `ManagerHintEngine::new` receive typed IH. Normal product loading consumes exactly the
generated base `A/manager_hooks.yaml` and optional user overlay `A/manager_hooks.local.yaml`; it
never calls the common ambient home resolver and never falls back to the compile-time repository.
`SUBSTRATE_MANAGER_MANIFEST` is not normal shell or physical-shim authority. A separately named diagnostic
input may be exercised and labeled `diagnostic_override`, but cannot satisfy install or physical-
shim product proof. The dev/release installer copies the already-selected version manifest to the
exact base path only after IH fixes A; Windows dev uses its already-known repository manifest only
as copy data after IH fixes A. The copy's location/content is a projection, cannot select A, and
grants no deletion, replacement-convergence, or artifact-ownership authority.

Because shim commands expose no
instance selector, direct platform telemetry uses only the exact product default Lima/WSL instance
and default normalized pipe; inherited platform carriers are consistency projections. A shim child
launched by an already-validated shell may consume that parent's explicit factory projection, but
the backend factory still receives typed fields and cannot reread ambient home/socket/pipe values.

An effective-UID-0 context-aware Substrate-owned internal helper reached through sudo accepts a carrier only when the sudo-set
`SUDO_UID` and `SUDO_USER` are both present, name/UID round-trip through the account database,
identify a non-root principal, and exactly match the already-committed principal. The helper does
not use them to reselect A. Missing or conflicting sudo origin fails. A direct-root public entry has
no internal-child mode: it must declare a non-root intended account at the public boundary, builds
the context there, and may then perform its already-authorized privileged operations in-process.
An unkeyed SHA-256 commitment proves consistency only; these OS identity checks provide the
principal authorization.

An arbitrary privileged executable such as a package manager, `groupadd`, `usermod`, `install`, or
`systemctl` is not a context validator and must not receive the encoded carrier as an invented
option. Immediately before each such crossing, the owning installer revalidates its in-memory
context and derives an exact, shell-free argv projection from A and the committed account/UID. It
invokes `sudo -- <program> <derived argv>` without preserved environment; the child receives no
home/root selection input and has no permission to reconstruct one. A Substrate script/CLI that
does need H/R remains the first sudo case and must validate the full carrier itself. This reviewed
split is the exact carrier authority at every current sudo boundary.

`substrate-apply-socket-acl` is the sole frozen non-context Substrate system leaf in that split. It
runs both directly under sudo and later from root-owned systemd `ExecStartPost`, so it receives no
fictional intended-account or sudo-origin carrier. R2-2 closes its parser over exactly these tuples:
`--socket /run/substrate.sock substrate`,
`--directory-traverse /var/lib/substrate substrate`, and
`--tree-readonly /var/lib/substrate/world-deps substrate`. Any omitted/extra argument, other mode,
target, or group fails before ACL mutation. The provisioner validates IH/principal before writing
the fixed drop-in or invoking the fixed leaf. The helper's existing account-database enumeration of
all current `substrate` group members remains its ACL subject-set rule, not intended-principal or
prefix authority; its ACL application, degraded-warning, and subject-enumeration semantics are
otherwise frozen. This exception cannot be generalized to a helper that reads or reconstructs
H/R/principal context.

For every internal child, including a product-CLI child and the hidden installer home-bootstrap
action, presence of the hidden/child argv carrier
is the child discriminator; the environment variable carrier alone never selects child mode. The
script intake or root dispatcher decodes and authenticates argv, then binds the committed principal
to the current OS identity before dispatch.
On Unix, a non-root effective UID and canonical `getpwuid` account must exactly equal the carrier's
UID/account; effective root is accepted only through the already-frozen exact SUDO_UID/SUDO_USER
round trip. On Windows, `WindowsIdentity.GetCurrent()` account and SID must exactly equal the
carrier. A validly re-encoded/rehashed carrier for another principal is rejected even if H/R and
the prefix match. Only then does the dispatcher snapshot inherited H/R for conflict diagnostics,
overwrite the mandatory H/R/principal/commitment environment projections from IH, and pass typed IH to the inventoried
leaf before config, policy, dependency, gateway, doctor, or shim resolution. The same ordering
applies after a standalone public constructor. PI-106–PI-111 name the legacy inner consumers:
where a downstream resolver still reads the process projection, its owning leaf must first assert
that projection equals typed IH. That read is a checked projection consumption, never a second
constructor or mutable side table. No unlisted same-process consumer gains this exception.

The carrier golden/negative suite includes a canonical but forged-principal carrier for each OS,
Unix account/UID mismatch in both directions, missing/mismatched sudo origin, and Windows
account/SID mismatch. Every case fails before bootstrap, projection read, leaf dispatch, or
mutation. The unkeyed commitment supplies consistency, never principal authentication.

Every shim deploy/remove/status/doctor/repair, automatic CLI shim deployment, trace projection,
world/host/health doctor, dependency global add/remove, current sync/list, config/policy proof,
gateway lifecycle proof, runtime-family/Codex provision step, generated-file writer/reader,
service renderer/restart, and uninstall child receives the applicable tuple. Existing rollback or
remove calls receive the same context in R2, but R2 may change only their carrier—not their decision,
deletion, or convergence semantics.

##### `PlatformBootstrapMappingV1` construction and verification

The host adapter constructs the mapping only after validating the host carrier. It recomputes the
host commitment, resolves the exact platform instance, queries the guest account database, and
normalizes the platform-native absolute home with the Unix rule above. Lima identity is
`Lima { vm_name, guest_machine_id }`; WSL identity is
`Wsl { distro_name, guest_machine_id }`. `vm_name` is the exact name supplied to `limactl`; WSL
matches the declared name against `wsl --list --quiet` case-insensitively but stores the exact
registered spelling. `guest_machine_id` is the lowercase 32-hex content of `/etc/machine-id` read
inside that exact running guest; empty, missing, malformed, or changed identity fails closed. This
OS instance observation is not a Substrate-generated install projection and does not select host or
guest home authority.

The guest principal is always the Unix account plus UID returned by `id -un`/`id -u` and verified
by `getpwnam`/`getpwuid` round trip in that guest. Its account-database home must exist as an
absolute normalized Unix path; `/home/<name>`, guest `$HOME`, or a host-mounted home is not a
fallback. `realized_substrate_home` is that guest account home plus `/.substrate`. The resulting
`realized_principal` is the guest Unix principal, even when the host principal is Windows. Thus no
host/guest path equality, account equality, or host UID-to-guest UID equality is asserted.

`host_platform_control_root` is a typed host projection committed by PM; it is not a second install
root, state-root selector, side table, or guest path. For Lima, resolve the already-committed Unix
host account+UID through the host account database, require the same account/UID round trip used by
IH, normalize its absolute account home, and append `/.lima`. That exact path selects the Lima
instance store before Stage 1. The public parent discards outer `HOME` and `LIMA_HOME`, passes the
typed path, and launches every `limactl`/SSH-config child with `HOME=<account-database-home>` and
`LIMA_HOME=<host_platform_control_root>` overwritten. An internal child rejects a nonempty inherited
value that does not equal those projections; `lima_home_dir`/`lima_ssh_config_path` receive the
typed path and never read process environment. Thus an outer home B can neither select another
store nor block the parent product path merely by being present: it is scrubbed before the internal
boundary, while a directly injected conflicting child projection fails closed.

For WSL, resolve `FOLDERID_LocalAppData` for the current Windows token after its account+SID equals
IH; never read `LOCALAPPDATA` or `USERPROFILE`. Define `WindowsForwarderScopeV1` as SHA-256 over
exactly these six LF-terminated ASCII lines, with the same strict base64url and lowercase-hex rules:

```text
domain=substrate.windows_forwarder_scope
version=1
windows_sid=<B64(canonical SID)>
distro_name=<B64(exact registered distro spelling)>
guest_machine_id=<32-lowercase-hex>
pipe_path=<B64(normalized pipe path)>
```

The 64-lowercase-hex digest is a deterministic path component, not a prefix commitment, ownership
manifest, or deletion authority. WSL `host_platform_control_root` is exactly
`<KnownFolderLocalApplicationData>\Substrate\forwarder\<scope-digest>`, normalized by the Windows
rule. The prefix-scoped forwarder config and logs are instead exactly
`A\forwarder\forwarder.toml` and `A\forwarder\logs`; the shared PID projection is exactly
`<host_platform_control_root>\forwarder.pid`. The pipe itself remains the PM transport identity.
Every producer/consumer receives these paths explicitly, records the active host-context
commitment where a shared projection exists, and rejects another commitment. Location derivation
authorizes R2 only to generate the A-scoped config and create the selected log directory after IH
validation, and to transport the future shared PID target without moving the guarded WSL path.
Replacement, rollback, PID removal, process termination, ownership manifests, and deletion remain
R3.

Lima has an exact two-stage construction order. Stage 1 validates `IH` and the declared/default
normalized `vm_name`, may read instance status, render the existing fixed host profile, create an
absent declared instance or start that same stopped instance, and wait for it to run. Those are the
only pre-mapping instance-realization actions; they may not select or project a guest home,
principal, service, unit, socket, forwarded socket, or staged workspace. Once the exact instance is
running, Stage 2 reads machine ID and account-database identity/home and constructs/revalidates
`PM` before any R2-owned guest-home/service/socket/platform child propagation. Lima's fixed base
image provisioning during first creation is part of instance realization and may not receive A or
invent a guest home. `destroy_vm`, layout-mismatch delete/rebuild, staged-tree replacement,
legacy-unit/socket deletion, and forwarded-socket unlink are PI-098–PI-101 R3 actions;
active-handle teardown and timeout kill are PI-113–PI-114 R3 actions. R2-3 does not edit or
exercise them as mapping proof. If an existing instance requires one of those actions,
R2-3 stops with the R3 prerequisite unmet rather than treating delete/rebuild as Stage 1.

`PlatformBootstrapMappingV1` uses the same strict line-framing rules and outer unpadded base64url.
Its exact platform-specific record is:

```text
domain=substrate.platform_bootstrap_mapping
version=1
host_context_commitment=<64-lowercase-hex>
platform_kind=<lima-or-wsl>
instance_name=<B64(exact vm_name or registered distro spelling)>
guest_machine_id=<32-lowercase-hex>
host_platform_control_root=<B64(normalized host absolute path)>
realized_substrate_home=<B64(normalized guest absolute path)>
realized_principal_account=<B64(guest account)>
realized_principal_uid=<U32(guest uid)>
transport_kind=<same lima-or-wsl>
transport_host=<B64(normalized host socket or normalized pipe path)>
transport_guest_socket=<B64(normalized guest socket)>
```

It is exactly thirteen LF-terminated lines in that order. `platform_kind` and `transport_kind` must
match; all base64url, decimal, digest, path, instance, control-root, and transport rules re-encode canonically.
It has no second home selector and no independent host commitment: its
`host_context_commitment` must equal the recomputed host-carrier commitment. Each platform-adapter
boundary carries that mapping and revalidates all six semantic fields against the host carrier and
live instance before any post-realization R2 mutation. After that validation,
a leaf service/forwarder process may receive only the exact required typed/argv fields plus the host
commitment; those values are authoritative for that child only because the validated parent passed
them, and the child must reject a conflicting carrier/projection rather than reselect. Generated
units, socket paths, and forwarder files may echo the mapping/digest only as projections. The default Lima VM or WSL distro
has one instance-scoped guest service/socket; if it is already projected for a different valid host
commitment, a second prefix fails closed or uses a separately declared instance. No mapping side
table or backend-selected home is permitted.

`world-backend-factory::factory` accepts an explicit typed factory projection derived from the
verified host/mapping records on macOS/Windows; the contextless platform overload is removed or
fails closed. Shell platform adapters, shim telemetry, and replay are all explicit callers. Replay
only threads this projection through its public config/shell entry and planner/executor call chain;
recorded command, environment reconstruction, origin, policy, timeout, strategy, and execution
semantics are unchanged. A direct replay-library world call on macOS/Windows without explicit
bootstrap input fails before backend construction. Linux may use the platform-independent factory
variant because it selects no home/socket/guest mapping.

On macOS, the host-side forwarded socket is always
`<selected_host_prefix>/sock/agent.sock`; the guest socket remains `/run/substrate.sock`; those
exact paths populate `PlatformTransportIdentityV1::Lima`. The SSH `UserKnownHostsFile` is exactly
`<selected_host_prefix>/lima_known_hosts`, passed from verified IH into forwarding as a generated
helper projection; its file existence or contents cannot select A, the VM, or PM. Explicit socket
unlink, SSH `StreamLocalBindUnlink`, `ForwardingHandle::drop` child/socket teardown, and the SSH
timeout kill remain byte-frozen R3 lifecycle actions in R2-3. On Windows, shims and
`substrate-profile.ps1` are prefix-scoped. The first public Windows mapping entry selects the pipe
from an explicit `-PipePath`/equivalent parameter or the exact default
`\\.\pipe\substrate-agent`; no later child selects it. A normalized pipe is exactly
`\\.\pipe\<name>`, where `<name>` is 1–128 ASCII characters from `[A-Za-z0-9._-]`; the prefix is
canonicalized as shown and the name is lowercased so Windows case aliases have one commitment.
Slash variants, nested names, whitespace/control characters, and any other spelling are rejected.
Warm, forwarder, backend, pipe-status, and doctor consume this mapping field. A public parameter in
a downstream child must match it; `SUBSTRATE_FORWARDER_PIPE`, a hard-coded default, or an ambient
value is only a consistency/diagnostic input and cannot select another pipe.
This deliberate subset follows the documented local pipe form and case-insensitive name behavior;
see [Microsoft Pipe Names](https://learn.microsoft.com/en-us/windows/win32/ipc/pipe-names).

For V1, `PlatformTransportIdentityV1::Lima` has exactly one future normal-product target: SSH UDS
from `<selected_host_prefix>/sock/agent.sock` to `/run/substrate.sock`. PM fixes those paths without
launching a forwarding child; SSH or `vsock-proxy` availability is never a selector.
`Transport::auto_select`, `forwarding::auto_select`, the TCP loopback endpoint, and ambient endpoint
state may not choose or replace the validated target. Because the current SSH constructor performs
pre-launch socket deletion, sets `StreamLocalBindUnlink`, kills/waits on timeout, and its handle drop
kills the child and removes the socket, R2-3 must stop the validated normal-product path before any
forwarder launch with an explicit R3 lifecycle prerequisite. It may pass the future A-scoped socket
and known-hosts parameters into typed construction surfaces, but it may not call that constructor,
exercise its lifecycle branches as proof, or claim macOS product transport. R3 alone may activate
the exact PM-bound target after PI-101, PI-113, and PI-114 own unlink, timeout, teardown, retry, and
convergence semantics.

Existing VSock, SSH UDS, and SSH-TCP construction may remain only for direct diagnostic/test call
sites explicitly labeled non-product; R2 adds no public or ambient transport selector. Those paths
cannot satisfy PM, R2-MAP-MAC-01 normal-product proof, any install/world product gate, or R2 native
mapping proof. A native R2 mapping-only run may read an existing Lima instance and construct/verify
PM, but it starts no forwarder and creates, unlinks, or removes no socket. Admitting another normal
Lima transport requires a separate reviewed contract extension with an unambiguous committed
identity; it is not an ambient fallback.

The Windows guest socket remains `/run/substrate.sock`. The named pipe, scoped PID root, and WSL
service are shared only within the exact `(Windows SID, WSL platform-instance, normalized
PipePath)` scope. Forwarder config/logs are prefix-scoped under A. Shared projections must
carry/reject conflicting host commitments; prefix projections must equal IH. That classification
transports identity only; it is not a managed-artifact deletion manifest. Lima's documented host mount and
independent guest home make path equality specifically invalid; WSL supports multiple named
distributions, so the declared distro is part of the instance identity. See
[Lima usage](https://lima-vm.io/docs/usage/) and
[Microsoft WSL commands](https://learn.microsoft.com/en-us/windows/wsl/basic-commands).

The public or installer-managed `start-forwarder.ps1` boundary constructs or validates IH and PM,
binds the current Windows account+SID, and starts `substrate-forwarder` in internal-child mode with
both carriers. It passes explicit `--config A\forwarder\forwarder.toml` and
`--log-dir A\forwarder\logs`; the internal Rust CLI requires those exact arguments and
`ForwarderConfig::load`/`Cli::resolve_log_dir` never use an environment-derived default.
`ForwarderConfig` authenticates the carriers and requires its distro and pipe to equal PM's
registered distro and normalized pipe. For normal product UDS mode, `BridgeTarget::Uds` must equal
PM's exact guest socket. Existing explicitly selected TCP compatibility mode remains a labeled
transport diagnostic after PM validation; it cannot select the distro, pipe, host commitment,
guest identity/home, or satisfy R2-MAP-WIN-01 product proof. A config file,
`LOCALAPPDATA`, `USERPROFILE`, `SUBSTRATE_FORWARDER_TARGET`, pre-existing
`SUBSTRATE_FORWARDER_TARGET_*`, or inherited `WSLENV`
is match-only diagnostic input and cannot replace the verified mapping.

At PI-115, `spawn_bridge` receives that typed verified forwarder configuration and passes only the
exact registered distro to `wsl -d`, plus the derived target fields and 64-lowercase-hex host
commitment to the embedded guest bridge. `wsl::spawn` overwrites the complete `WSLENV` list and
each `SUBSTRATE_FORWARDER_TARGET_*`/commitment value; it never inherits one as selection authority.
This is the reviewed leaf projection permitted after PM verification, not a new guest mapping
constructor. Ambient-conflict tests must prove the `wsl` argv/environment use PM-derived values or
fail before spawn. Process wait/stream shutdown behavior remains operational; timeout kill, stop,
unregister, PID deletion, and convergence remain R3.

At this starting commit, `scripts/windows/wsl-warm.ps1` intentionally fails closed before WSL
mutation and `scripts/wsl/provision.sh` is an unconditional exit-4 guard. R2-3 may add only carrier
parameter validation before the warm guard and may construct/verify `PlatformBootstrapMappingV1`
against a separately existing named WSL instance through the backend/doctor paths. It must preserve
both guards byte-for-byte, may not execute the currently unreachable provisioning body, and may not
claim Windows world provisioning proof. Moving either guard is
`ArchitecturalBoundaryDecisionRequired`, not prefix propagation.

##### Generated projections, install/uninstall symmetry, and diagnostics

`env.sh` and Windows `substrate-profile.ps1` encode both `SUBSTRATE_HOME=A` and
`SUBSTRATE_ROOT=A`, the host commitment, and the encoded carrier. Unix `manager_env.sh` and the
runtime `MANAGER_ENV_SCRIPT` derive A from their own file directory before reading any ambient
value, then source only `A/env.sh`; Windows profiles use their own parent directory. The dev shim
helper encodes the same tuple. Configuration, version, dependency inventory, and binaries are
prefix-relative projections and do not need to repeat the path merely to look authoritative.
The shell and physical-shim manager-hint base is likewise the exact generated `A/manager_hooks.yaml`; its
optional overlay is `A/manager_hooks.local.yaml`. Installers select the source version/repository
asset only after IH fixes A, and the shim receives typed IH before it reads either file. Neither
file, ambient H, an environment override, nor a compiled repository path may select A.

The generated Bash preexec script and its writer receive A explicitly. The dispatcher places the
helper at an A-derived location and passes that path into `ShellConfig`; the script derives A from
its own no-follow-validated file location, validates/consumes only `A/manager_env.sh`, and rejects a
conflicting outer H/R/carrier rather than using `$HOME` or `$USERPROFILE`. `BASH_ENV` and the user's
interactive `.bashrc` remain user shell inputs but cannot select Substrate's manager, trace, or
prefix projections. No new per-user helper location outside A is authority.

Every generated Linux service unit carries `SUBSTRATE_HOME=A`, `SUBSTRATE_ROOT=A`,
`SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=<hex>`,
`SUBSTRATE_INSTALL_PRIMARY_USER=<account>`,
`SUBSTRATE_INSTALL_PRIMARY_UID=<decimal UID>`, and the encoded carrier; every `ReadWritePaths` entry for the
selected home derives from A. An inherited/root `$HOME` is never a service allowlist input. The
fixed `/run/substrate.sock` may be removed only as the existing operational step immediately before
same-attempt service restart/socket recreation; this is not general cleanup authority. Legacy-unit,
uninstall, wildcard, recursive, or managed-artifact removal remains R3.

Unix PATH-snippet upsert and its wrapper diagnostic derive the target shell files from the intended
principal's account-database home, not the invoking/root `$HOME`. The snippet is a projection of
`A/bin`. Removing snippets, choosing ownership for removal, and preserving unrelated profile text
remain R3 lifecycle work.

Every Linux Codex host-credential projection resolves the committed Unix account+UID through the
account database and derives that account's home plus `/.codex`; `dirs::home_dir`, invoking/root
HOME, USERPROFILE, and A are not substitutes. This applies both to gateway integrated-auth reads
and member-dispatch seed-home injection. It changes neither policy authorization nor credential
contents and grants no cleanup authority; synthetic-auth removal remains R3.

The existing `install_state.json` may record the canonical carrier and commitment as a consistency
projection. R2 adds no deletion inventory, ownership manifest, rollback journal, or cleanup
provenance. Uninstall constructs A by the same public rule as install, then may validate a matching
record under A; the record can confirm or reject A but cannot select a different prefix. Missing or
malformed records never authorize ambient fallback. A direct installed uninstaller must receive
`--prefix A` from its wrapper or self-derive A from its installed location; the Unix wrapper may no
longer locate by one root and let its child select another home.

Under conflicting ambient `SUBSTRATE_HOME=B`, `SUBSTRATE_ROOT=B`, `$HOME=B`, or
`$USERPROFILE=B`, an installed projection at A consumes only A or fails closed. Repeat install and
install/uninstall use the same normalized context and commitment. No product proof may pre-export A
as an outer workaround; tests must include A versus B and prove zero reads/writes/deletes under B.

Shim, trace, world, and health doctors invoked by an installer consume the same validated tuple and
project A, the commitment, and the carrier source (`install_context`). A direct diagnostic may
accept an explicitly named diagnostic home/trace override and labels it `diagnostic_override`; it
cannot be reported as install-context proof. `SHIM_TRACE_LOG` is a diagnostic trace-path override,
not prefix authority. A normal selected-context doctor derives `A/trace.jsonl`, `A/shims`, A's
configuration/dependency state, the prefix-scoped macOS host socket, and the applicable platform
mapping without `dirs::home_dir()`/`$HOME` fallback.

The same rule applies to live trace production, not only doctor output, but migration is staged.
R2-1 adds a closed explicit product-bound `TraceContext` posture equivalent to
`ExplicitProduct { trace_output: A/trace.jsonl, policy_git_directory: A }`. Only after shell IH and
current-principal validation may the shell construct/register it. Both paths are immutable explicit
inputs: the posture cannot read `SUBSTRATE_HOME`, `$HOME`, `SHIM_TRACE_LOG`, CWD, or
`dirs::home_dir()` to choose them. Its `init_trace(None)` initializes or reuses the already-bound A
target; an explicit path must equal the bound target. Additive `get_policy_git_hash_at(A)` reads
only A and returns no commit for missing `.git`, unreadable/invalid metadata, or failed
`git rev-parse`, with no retry under B, ambient home, repository root, CWD, or `/tmp`.

`set_global_trace_context` remains a neutral registration primitive with its exact existing
signature and set-once/global-registration semantics. It gains no IH validation, path selection,
caller discrimination, or rejection of existing default callers. Pre-existing unmigrated callers
retain their byte- and behavior-equivalent starting operation under the named temporary posture
`LegacyAmbientCompatibility`. That posture is not `ContractCorrectAndProven`, cannot satisfy R2-1
shell trace proof, and is unreachable from migrated shell product paths. No side table or inferred
caller identity is allowed in the trace crate. R2-3 owns physical-shim explicit binding under
PI-118, the already-frozen replay/platform caller migration, compatibility removal/unreachability,
and the final global rule that unbound `init_trace(None)` fails. R2-1 may claim only shell A output,
shell A policy lookup, shell repeated-A reuse, shell A/B zero access, and unchanged explicitly
unpromoted compatibility callers. Policy mode/content, span/replay schema, environment hashing,
writer flush, rotation, retention, rename, and removal bodies do not change in R2.

R2-1 satisfies the shell/common portion of this gate through
`2653c2ef20ae2e119a444811e6fb46e86d1a6ec6`. Golden and tamper tests prove the frozen IH
domain/version/line framing, base64url fields, SHA-256 commitment, duplicate/unknown/reordered-field
rejection, checked H/R equality, and current Unix account+UID binding. Hidden argv alone selects the
internal bootstrap action; forged/missing/malformed context rejects before mutation; version/help
and parse exits remain non-mutating. Product-bound shell trace and policy tests prove A-only output,
repeat reuse, conflicting-path rejection, missing-metadata no-fallback, and no B access. The setter,
legacy compatibility posture, physical shim, replay, writer/serialization/rotation/retention, and
R3 lifecycle bodies remain unchanged and unresolved exactly where assigned.

##### R3-exclusive lifecycle authority

R2 selects and transports the exact context that later lifecycle operations consume. It does not
define or exercise their deletion authority. A1.1d-5R3 exclusively owns: deleting partial
candidates; current-attempt rollback; removing managed gateway/helper/unit/socket artifacts;
managed-artifact ownership manifests used for deletion; removing recursive or wildcard deletion;
uninstall convergence; restoration of installer-created group/membership/ACL/linger state;
crash-window cleanup; uninstall-to-reinstall convergence; and preservation of unrelated or
pre-existing artifacts. Shared Lima/WSL stop action, recursive dev-shim fallback removal,
shell-profile snippet removal, and legacy ambient-home `.substrate_preexec` removal are cleanup
actions under that same R3 authority. So are shim-tree
replacement/migration/removal; dev/release/Windows payload, version, bin, profile, cache, and legacy
helper/unit replacement; Lima destroy/rebuild, staged-tree/temp/unit/socket cleanup, forwarded
socket unlink, SSH-side bind unlink, active forwarding-handle teardown, and SSH timeout kill;
Windows forwarder timeout process kill; and synthetic Codex-auth deletion. Any R2
change that adds, broadens, or claims one of those behaviors stops as
an ownership violation and moves to R3. The existing R1 no-repair/private-home contract remains
unchanged.

R3 also owns the first activation of the R2-fixed PM-bound Lima SSH-UDS target. It may make that
target reachable only in the same reviewed change that gives PI-101/PI-113/PI-114 exact ownership,
failure, teardown, retry, and convergence semantics. Activation consumes the existing IH/PM and
cannot select a different prefix, home, principal, VM, transport kind, host socket, guest socket,
or known-hosts projection. This activation assignment is lifecycle ownership, not a new mapping
seam or permission for R2 to exercise the current constructor.

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
obligations, or their event/materialization cut. On the bound Tuesday, August 4, 2026 source
candidate, the bounded internal A1.2b durable protocol is complete and retains
`AwaitingObligationCut` only as the exact incomplete-cut durable state. Aggregate A1.2 remains
open because A1.3 adopts the protocol on real CLI/REPL consumers; A1.1 primitive tests are not
evidence that a production Start path is adopted.

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
    `HostExecutionEpisode` owns observation reporting and A1.3-P0's bounded real helper/REPL
    transport adapter carries that exact protocol event into one closed
    `HostStartupOwnershipEvidenceV1`; neither may relabel local observations into a protocol event.
    `HostSessionAuthority`, not the episode
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
    A1.3-P0 first owns invocation by the real helper/REPL public transport consumer, A1.3 later
    consumes that landed split for public adoption and closeout, and A2 later generalizes episode
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
`717579b0`, and `83101dcb`. The later joint production closeout is complete on the frozen
2026-08-03 source without additional product bytes, and B3.1, C1, and the bounded internal A1.2b
packet are complete on the bound Tuesday, August 4, 2026 candidate. The then-current historical
gate was `AUTHORITY_REQUIRED:R3_RESUME`; its former macOS-parity replacement is
superseded as the global schedule by `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY`. The historical
prepared type also combined
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

The completed joint B1/B2.1 production closeout recorded at the bound 2026-08-03 source snapshot
proves ephemeral acceptance, retained production handoff, legacy-writer exclusion, blocking
compatibility, restart survival, and exact terminal behavior. Actual caller/waiter/guard drop is
proven on the RunWorldTask and ephemeral accepted-task production routes; retained
foreground-waiter drop remains separately proved at the accepted-stream boundary. B3.1 is now
complete on the bound Tuesday, August 4, 2026 candidate, and every later packet still requires
fresh authority and its own review/proof gates.

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

## A1.1d-5R2-2F0-HC corrected complete process-resource ledger

This is the durable closure inventory for the process running the shell-library wall under the
[canonical broad-wall invocation contract](#canonical-shell-library-broad-wall-invocation-contract). It was
formed by two independent methods: lexical scanning of all `crates/shell/src` test modules found
1,303 source test functions, while the same-file helper/call parser closed 5,070 functions and
initially recognized 1,302 tests before the independent scan restored the multiline-attribute
`execution/lock.rs::test_concurrent_lock_attempts`; and symbol, caller, reader, and execution-flow
inspection used refreshed GitNexus plus source closure where the graph under-resolved `cfg(test)`
code. The first parser found 506 environment-dependent tests in 31 files; the published audit then
manually added twelve disjoint dynamic-wrapper tests to claim 518 in 35 files. Complete lexical and
resolved-call closure instead finds 534 parent-mutating tests in the same 35 files: eighteen real
mutators were absent, while two B1 acknowledgement tests were false positives from ambiguous
same-file bare-name fanout. The exact additions and removals are enumerated in `02` and `05`. A2 is
therefore 129 direct tests in 22 files—116 additive plus 13 F0a—not the earlier 113 or intermediate
116. The Linux test binary discovered 1,263 runnable tests. Known integration parent-mutators were
inspected only for helper and inherited-child contracts.

The corrected environment closure found 86 exact names (some are only projected into a child or
read without parent mutation):
`ANTHROPIC_API_KEY`, `API_TOKEN`, `BASH_ENV`, `CODEX_HOME`, `COLUMNS`, `EXPORT_COMPLEX`, `HOME`,
`LIMA_VM_NAME`, `LINES`, `OLDPWD`, `OPENAI_API_KEY`, `PATH`, `PLAIN_VALUE`, `PWD`, `SHIM_ACTIVE`,
`SHIM_ORIGINAL_PATH`, `SHIM_PARENT_SPAN`, `SHIM_SESSION_ID`, `SHIM_TRACE_LOG`,
`SUBSTRATE_A1_REPLACEMENT_CHILD_ROOT`, `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`,
`SUBSTRATE_AGENT_TOOLBOX_VERSION`, `SUBSTRATE_ANCHOR_MODE`, `SUBSTRATE_ANCHOR_PATH`,
`SUBSTRATE_CAGED`, `SUBSTRATE_COMMAND_SUCCESS_EVENTS`, `SUBSTRATE_DISABLE_PTY`,
`SUBSTRATE_ENABLE_PREEXEC`, `SUBSTRATE_FORCE_PTY`, `SUBSTRATE_HOME`,
`SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1`, `SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT`,
`SUBSTRATE_INSTALL_PRIMARY_UID`, `SUBSTRATE_INSTALL_PRIMARY_USER`,
`SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME`,
`SUBSTRATE_LIMA_VM_NAME`,
`SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`,
`SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`, `SUBSTRATE_MANAGER_ENV`,
`SUBSTRATE_MANAGER_ENV_ACTIVE`, `SUBSTRATE_MANAGER_INIT`, `SUBSTRATE_MANAGER_INIT_DEBUG`,
`SUBSTRATE_MANAGER_INIT_POWERSHELL`, `SUBSTRATE_MANAGER_MANIFEST`, `SUBSTRATE_NO_SHIMS`,
`SUBSTRATE_ORIGINAL_BASH_ENV`, `SUBSTRATE_OVERRIDE_ANCHOR_MODE`,
`SUBSTRATE_OVERRIDE_ANCHOR_PATH`, `SUBSTRATE_OVERRIDE_CAGED`, `SUBSTRATE_OVERRIDE_WORLD`,
`SUBSTRATE_PARENT_SPAN_ID`,
`SUBSTRATE_POLICY_MODE`, `SUBSTRATE_PTY_PIPELINE_LAST`,
`SUBSTRATE_R0_RETAINED_SUBPROCESS_VARIANT`, `SUBSTRATE_REPLAY_USE_WORLD`,
`SUBSTRATE_REPLAY_VERBOSE`, `SUBSTRATE_ROOT`, `SUBSTRATE_SHELL`, `SUBSTRATE_SKIP_MANAGER_INIT`,
`SUBSTRATE_SKIP_MANAGER_INIT_LIST`, `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE`,
`SUBSTRATE_SYSTEMCTL_TIMEOUT_MS`, `SUBSTRATE_TEST_LOCAL_WORLD_ID`,
`SUBSTRATE_TEST_SHARED_WORLD_METADATA_ROOT`, `SUBSTRATE_WORLD`,
`SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR`, `SUBSTRATE_WORLD_DEPS_SKIP_APT`,
`SUBSTRATE_WORLD_DEPS_SKIP_PACMAN`, `SUBSTRATE_WORLD_ENABLED`,
`SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING`, `SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64`,
`SUBSTRATE_WORLD_FS_ISOLATION`, `SUBSTRATE_WORLD_FS_MODE`, `SUBSTRATE_WORLD_ID`,
`SUBSTRATE_WORLD_NET_FILTER`, `SUBSTRATE_WORLD_PROJECT_DIR`,
`SUBSTRATE_WORLD_REQUEST_PROFILE`, `SUBSTRATE_WORLD_REQUIRE_WORLD`,
`SUBSTRATE_WORLD_SOCKET`, `TEST_ENV_KEY`, `TEST_MODE`, `UNSET_ME`, `USERPROFILE`,
`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, and `XDG_STATE_HOME`.

Exactly 74 of those names are parent-mutated. The 12 names that are child-only projections or
read-only in this bounded source are `ANTHROPIC_API_KEY`, `BASH_ENV`,
`SUBSTRATE_A1_REPLACEMENT_CHILD_ROOT`, `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`,
`SUBSTRATE_AGENT_TOOLBOX_VERSION`, `SUBSTRATE_ENABLE_PREEXEC`,
`SUBSTRATE_MANAGER_ENV_ACTIVE`, `SUBSTRATE_ORIGINAL_BASH_ENV`,
`SUBSTRATE_PARENT_SPAN_ID`, `SUBSTRATE_R0_RETAINED_SUBPROCESS_VARIANT`,
`SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64`, and `SUBSTRATE_WORLD_PROJECT_DIR`. The other 74,
including all three XDG names and `SUBSTRATE_SHELL`, are parent mutations. `SUBSTRATE_ROOT` is mutated by the
wrapper/F0a families enumerated in `02` and belongs to A2. Environment values, credentials, and
complete paths were not emitted by diagnostic probes.

The prior 80-name claim is withdrawn. Its test-reachability pass recognized wrapper bodies, but
its exact-name phase scanned direct primitive arguments and did not resolve every wrapper
argument/callsite. The manual dynamic-wrapper repair kept the two settings tests without extracting
their `SUBSTRATE_OVERRIDE_*` arguments and omitted the gateway `AmbientSelectionGuard` call. This
was not a child-only XDG projection and was not stale-source drift: immutable source invokes
`std::env::{set_var,remove_var}` in the parent for all three XDG entries. The corrected audit
reconciles 395 direct primitive invocations, including 73 dynamic-key invocations, with 43 dynamic
mutation-sink owners, 1,005 resolved mutation callsites, and 534 exact mutating tests, and reports
zero unclassified dynamic rows. Independent correction review rejected both the earlier 405-row
and later 604-row evidence because they omitted cross-file or duplicate-name calls through
`with_store`, `ProjectionEnvGuard::capture`, `install_bootstrap_projections`,
`apply_world_root_env`, `export_runtime_config_env`, `update_world_env`, and the macOS `restore`
family. Its prevention rule is binding: any absent
wrapper callsite, unresolved literal/constant/table/parameterized name, silently dropped dynamic
name, parent-as-child classification, or mutating test missing from the migration manifest fails
the gate. Full callsite closure additionally proves `SUBSTRATE_SHELL` is mutated by
`routing/test_utils.rs::{set_env,restore_env}` in
`async_repl_host_commands_record_replay_context`; `crates/trace/src/span.rs::SpanBuilder::new` is
the overlapping stable reader. Three early returns precede the manual restore, and `set_env`
captures `Option<String>`, so current restoration is neither early-return/unwind safe nor exact for
non-Unicode values. The test is already in the A2 116-test manifest and 35-file union. The two
macOS platform tests restore only `SUBSTRATE_WORLD` and `SUBSTRATE_WORLD_ENABLED` as
`String`/absence even though `update_world_env` changes six names; their future test-only helper
must restore all six as exact `OsString`/absence under the same coordinator.

The evidence validator freezes the normalized 395-call primitive manifest at SHA-256
`df0254488f37d4f53f05c81bbbd47c05bbbe410496d465827daade80922e0f4a`, the 1,005-call resolved
mutation manifest at `350ccd1876ecc07cced81df89b523e3d24f9674f5dc6a5e33f052d65f4489772`, and the 534-test
manifest at `de7e2bc43c0fcab74096dc272340726c919aeb72004fd0208ab5b4dffb802f4a`. It also freezes 43
dynamic mutation-sink owners at SHA-256
`133e28a495e5be69016fb7f6483018cfb406b44c8ca5e2d8779fdf82ded0c66c`, source-parses fixed
projection and local restore-key tables, and runs negative perturbation checks. The rejected
604-row intermediate remains historical evidence only. The validator parses the canonical
35-file, 116-additive-test, and 13-F0a-test lists, checks 129 unique source symbols, and compares
their exact manifest hashes. A duplicate removal, changed argument, missing callsite, swapped path,
or missing test therefore fails even when the set of environment names would remain unchanged.

### Identity and use closure

“Direct tests” counts the source-closed tests that directly use, or reach a same-file helper using,
the resource. Counts overlap between rows and are not test totals. `All` means the single
shell-library process; `child` means an intentionally inherited helper process.

| ID | Category and exact resource | Mutating symbols | Reading symbols / helper family | Direct tests | Scope / cfg | Current guard or isolation | Known evidence / competing tests | GitNexus impact |
|---|---|---|---|---:|---|---|---|---|
| A1 | Environment: `HOME`, `SUBSTRATE_HOME`, `SUBSTRATE_WORLD_SOCKET` | `std::env::{set_var,remove_var}`, `EnvVarGuard`, `with_store`, `with_env_var`, manual save/restore | authority/store/socket/path resolvers; F0/F0a helper families | 456 / 20 files | All; Unix plus cfg-specific readers | reentrant `WORLD_ENV_LOCK`, local guards, `#[serial]` islands | HOME and socket races proven; 11 unannotated dependents; mixed snapshots possible | `with_test_mode` HIGH; many test symbols under-resolved |
| A2 | Other ambient selectors, including `SUBSTRATE_ROOT`, `SUBSTRATE_SHELL`, the three XDG roots, and three `SUBSTRATE_OVERRIDE_*` selectors: the remaining directly or dynamically mutated names in the 86-name closure | same APIs plus `set_env`/`restore_env`, `ProcessStateGuard::set`, `EnvGuard::apply`, `ProjectionEnvGuard`, `AmbientSelectionGuard::{set,drop}`, `platform_tests::{snapshot,restore}`, and file-local wrappers | PTY, world, shim, manager, trace, policy, profile, install, terminal-size, gateway authority, settings, platform/world-policy, and `SpanBuilder::new` readers | 129 / 22 files: 116 additive plus 13 existing-source F0a | All; cfg varies | local guards and `#[serial]`; 27 additive unannotated | `SUBSTRATE_FORCE_PTY` stable-reader race proven; XDG/settings callsites, the early-return-unsafe host-replay mutation, and incomplete six-name macOS restoration now source-closed; any concurrent ambient mutator competes | `world_env_guard` CRITICAL; `EnvGuard::new` MEDIUM; macOS snapshot/restore LOW with two direct test callers and zero processes; gateway/routing test helpers LOW/under-resolved; production readers frozen |
| A3 | The read-only subset of the 12 non-parent names in the corrected closure | none in shell-library tests | corresponding config/home/root readers | 0 mutators | All / cfg varies | immutable during this binary | source closure found no parent writer; cannot form a same-process pair | source-only; no future edit |
| A4 | Child-only `Command::{env,env_remove,env_clear}` projections among the 12 non-parent names | command builders in test helpers | child bootstrap/probe decoders | 14 child spawners plus helper callers | child / cfg varies | per-`Command` environment | parent process is not mutated; exact child inheritance is intentional | LOW/source-closed |
| B1 | fd 1/fd 2 replacement: `dup`, `dup2`, `pipe`, `close` in capture helpers | `capture_stdout_once`, `capture_stderr_once` | two `PublicPromptRenderer` fallback tests; libtest reporter is competing writer | 2 | All / Unix | manual descriptor save/restore | stdout contamination proven; stderr structurally identical | renderer MEDIUM; capture helpers LOW |
| B2 | fd 0 `O_NONBLOCK` flag and terminal/console input mode | manual `fcntl(F_SETFL)` probes plus `minimal_terminal_guard_handles_creation` via `MinimalTerminalGuard::new` | libtest/input consumers and terminal/console state readers | 2 | All / Unix flag plus Unix/Windows terminal cfg | manual exact flag/mode restore | fixed descriptor is shared despite restoration; inherited child fd 0 would share the same open-file description | test symbols under-resolved |
| B3 | Child/local descriptors and inherited listener descriptors | pipe/listener/child builders | same fixture owner | 103 listener-dependent; 14 child-spawn-dependent | test/child / cfg varies | OS ownership plus joined fixture | no fixed-number replacement in parent; ownership remains local | LOW/source-closed |
| C1 | Process CWD | `set_current_dir`, `DirGuard`, direct/manual wrappers | `WorldRootSettings::effective_root`, config/workspace/root readers | 100 / 10 files | All / all platforms | `cwd_lock` in routing helpers, `#[serial]`, local guards | forced stable-reader race proven; one unannotated helper-closed test | `effective_root` HIGH, 2 direct / 1 process |
| C2 | umask | self-spawned sentinel test child | child filesystem creation | 22 helper-closed; 1 direct | child / Unix | child saves and restores exact mode | parent umask never changes; child exits after restore | LOW/source-closed |
| C3 | locale, timezone, process title, current-user/account overrides | none found | ordinary OS/account readers | 0 | All / cfg varies | immutable in this test process | exhaustive lexical categories found no mutator | no future impact |
| D1 | Global trace context/output | test `set_global_trace_context` + `init_trace(Some(...))` in routing/manager helpers | `append_to_trace`, telemetry and trace writers | 13 direct helper callers | All / all platforms | reset then global initialize; no ownership guard | forced output-owner replacement proven | `TraceContext::init_trace` MEDIUM; helper graph under-resolved |
| D2 | Private stop retry callback slot | `PrivateStopTransportRetryHookGuard::install` / Drop-clear | `request_private_stop*` hook read | 9 | All / Unix tests | mutex protects slot access, not owner lifetime | forced hook replacement proven | private/test graph under-resolved |
| D3 | Production signal/Ctrl-C handlers, including `initialize_global_sigwinch_handler_impl` | production initialization only | runtime signal consumers and the SIGWINCH background thread | 0 test installers/callers | product process / cfg varies | production lifecycle owner | source call closure proves no shell-library unit test calls `execute_with_pty`, so this handler/thread is never installed by this binary | production PTY/signal lifecycle owner |
| D4 | Panic hooks, tracing subscribers, global logger, allocator/runtime hooks | none found in shell-library tests | default libtest/runtime infrastructure | 0 | All | immutable/default | lexical and helper closure found no installer | no future impact |
| E1 | Existing environment coordination topology | `WORLD_ENV_LOCK`, `world_env_guard`, file-local env mutexes | all A1/A2 writers and stable readers | 534 / 35 files | All | one partial reentrant lock plus lock islands | opposite helper order and incomplete participation; `#[serial]` insufficient | `world_env_guard` CRITICAL: 70 impacted, 35 direct, five processes, nine modules; future test-only boundary only |
| E2 | `AGENT_EVENT_SENDER` / `EVENT_TEST_GUARD` | `init_event_channel`, `clear_agent_event_sender`; guard accessor `acquire_event_test_guard` | `publish_agent_event`, `agent_event_sender`, and publication readers | 10 guarded tests | All | two mutexes; poison `expect`/manual clear | competing stable readers are not excluded by file-local guard | private registry, source-closed |
| E3 | Non-keyed `REPORT_CACHE` | `refresh_socket_activation_report` | `socket_activation_report` | 1 mutating focused test | All / Linux behavior | mutex, no key or per-test reset | cache survives exact PATH/timeout restoration; proven | refresh HIGH, 3 direct / 3 process families |
| E4 | `CONFIG_PATCH_CACHE`, `POLICY_SNAPSHOT_CACHE` | keyed cache insert/update | exact path/file-stat keyed readers | multiple temp-root tests | All | mutex and exact workspace/global path keys | different fixtures cannot alias without same exact key/stat | production-adjacent but no edit |
| E5 | `WorldDispatchConcurrencyTracker` global maps | acquire/release test paths | cap/session lookup | 4 direct tracker tests plus fixture callers | All | mutex; key is caller session ID | forced reused-ID cap consumption proven | production tracker adjacency; private injection only |
| E6 | `PTY_ACTIVE` atomic | `active_guard_resets_flag_on_drop` | PTY routing/load checks | 1 direct mutator | All | atomic and RAII for product work | test can overwrite a real concurrent PTY state | production readers frozen |
| E7 | `world::SessionWorld` shared test-root override | two shell tests set/reset override | world metadata root readers | 2 | All / tests | currently held under world environment guard | safe only while unified guard participation remains complete | graph source-closed |
| E8 | Platform `GLOBAL_CTX` / Windows `CONTEXT` non-resettable initialization | no Linux-wall test mutation | platform context readers | 0 in Linux wall | platform cfg | once-only immutable initialization | cannot change after initialization and absent from active cfg | production platform owner frozen |
| E9 | Immutable `OnceLock` values, regexes, instance IDs, monotonic atomics | one-time initialization / atomic increments | same thread-safe primitives | many readers | All | `OnceLock`, immutable values, atomics | values do not depend on mutable env/CWD/user/policy paths | LOW/source-closed |
| E10 | `ACTIVE_PTY` registry and Windows `WIN_PTY_INPUT_GATE` | `ActivePtyGuard::register` / Drop-clear; Windows guard toggles the input gate | `active_pty_control` and PTY control dispatch readers | 1 direct test | All; additional gate on Windows | mutex/RAII, but one process-global owner slot | forced overlap proved a second registration replaces the first test's control | `ActivePtyGuard::register` LOW, one direct caller |
| E11 | `substrate_broker::GLOBAL_BROKER` mutable broker singleton | `with_test_mode`, `set_policy_mode`, policy initialization/reload paths | `policy_mode`, broker evaluation and selector readers | 23 exact tests | All / cfg varies | `OnceLock<BrokerHandle>` containing mutable `RwLock<PolicyBroker>`; no owner-scoped reset | forced overlap proved a stable reader observes another test's policy mutation | `set_policy_mode` LOW/under-resolved; source closure exact |
| F1 | Predictable private stop/cancel/prompt/startup/toolbox socket paths, especially `/tmp/substrate-agent-hub-stop/sessdispatch-*.sock` | exact 62 test bodies and their test-owned short root/home/store construction in `02` | corresponding stop/cancel/prompt/startup/toolbox clients | 62 exact tests: 36 orchestrator plus 26 async-REPL | host filesystem / Unix | some TempDir roots; some path-length fallbacks to predictable `/tmp` names | stale `sessdispatch-ashmember.sock` caused `AddrInUse`; six new stale stop nodes observed | direct path plus registration-helper closure exact in `02` |
| F2 | Unique TempDir Unix/TCP listeners | `UnixListener::bind`, `TcpListener::bind` on fixture-owned roots/port 0 | paired client/tasks | 103 helper-closed; 68 direct | fixture / cfg varies | unique temp root or kernel-assigned port; awaited owner | no cross-test alias when teardown is awaited | LOW/source-closed |
| F3 | Fixed TCP ports and inherited service endpoints | no fixed test bind found; child endpoint projection only | test client fixtures | 0 fixed-port mutators | child/fixture | kernel port 0 or explicit unique endpoint | no competing fixed port in shell lib process | no future impact |
| F4 | Cross-process lock files and private authority/temp roots | child/store fixture writers | exact-root store readers | 14 child spawners plus store suites | filesystem/child | exact private root, durable lock/CAS | proven safe when each test owns its root; parent ambient roots handled by A1 | store symbols production-adjacent, frozen |
| G1 | Wall-clock sleeps used to create ordering | 19 exact direct sleep tests in seven allowlisted files | competing task/thread readiness | 19 | test runtime | elapsed sleep only | scheduler load can invert intended order | source-closed tests; exact manifest in `02` |
| G2 | Bounded protocol timeouts/retry waits after readiness | `timeout`, retry loops, remaining helper sleeps | ready-published servers/clients | 94 timeout-dependent; remaining sleep closure | test runtime | explicit listener/state readiness plus deadline | timeout tests are safe where readiness precedes the clock | production timeout bodies frozen |
| G3 | Paused/advanced Tokio clocks or global clock override | none found | Tokio test-local time | 0 | runtime-local | none needed | no shared/global clock manipulation | no future impact |
| H1 | Abort without awaited termination | ten named orchestrator test servers/tasks | socket/root/env teardown | 10 | async test runtime / Unix | abort and sometimes one yield; no join | tenth case found; stale task/socket can outlive restore/unlock | test bodies under-resolved |
| H2 | Abort followed by awaited termination | remaining abort paths | same task owner | 28 helper-closed | async test runtime | `abort`; await `JoinHandle` | cancellation completion is observed before fixture drop | source-proven safe |
| H3 | Joined threads/children and runtime-scoped background tasks | 127 Tokio-spawn-, 48 thread-spawn-, 14 child-spawn-dependent tests | corresponding join/wait/Drop owners | overlapping aggregate | runtime/child | join/wait or runtime completion | no unjoined owner found outside H1; child death tests wait exact exit | source-proven safe |
| I1 | `#[serial]` groups | 550 source annotations | serial_test scheduler | 550 | libtest | named/default serial groups | excludes only participating tests, not unannotated readers or reporter | test-only; no graph node |
| I2 | Custom lock islands and order | env, CWD, event, per-helper mutexes | nested helpers | 4 lock families plus call closure | All | partial/reentrant and non-reentrant mutexes | mixed ENV/CWD order and poison semantics require one reviewed topology | helper graph under-resolved |
| I3 | libtest reporter and thread-count assumptions | reporter writes fds; no resource mutation by tests | all 1,263 discovered Linux tests | 1,263 | All | libtest | reporter is safe until a test captures fixed fd; no test may rely on thread count | runner external; B1 owns defect |

### Semantics, ownership, and primary dispositions

The compact fields below are binding. “Exact” means prior value or absence is restored without
Unicode conversion; “local” means no parent-process restoration exists because the resource is
owned by a child/fixture. Every row has exactly one primary disposition.

| ID | Restoration / panic / nesting | Async lifetime and subprocess inheritance | Potential competitors / proof | Primary disposition | Future owner packet and allowlist |
|---|---|---|---|---|---|
| A1 | exact `OsString`/absence; unwind-safe; stack-safe nesting or reject before mutation | retain through awaited cleanup; intentional bounded inheritance | all HOME/socket mutators and stable authority readers; proven races | `UnifiedProcessStateLock` | combined Harness; existing F0/F0a allowlists unchanged |
| A2 | exact `OsString`/absence; same panic/poison/nesting contract | retain across dependent child construction/termination | every ambient mutator versus stable selector reader; forced pair proven; XDG/settings corrections source-closed | `UnifiedProcessStateLock` | combined Harness; exact test-only files/symbols/tests in `02`; no XDG-specific lock |
| A3 | no mutation/restoration | immutable read | no parent writer in bounded process | `ProvenConcurrencySafe` | frozen; no future allowlist |
| A4 | `Command`-local bytes; parent unchanged | child receives explicit/inherited bounded projection and is waited | other processes cannot mutate parent | `ProvenConcurrencySafe` | frozen except already-authorized parent guard migration |
| B1 | exact descriptor restoration is insufficient; panic could strand redirection | reporter writes concurrently; child isolation not needed | libtest reporter; stdout failure and stderr structural proof | `ExplicitDependencyInjection` | F0b; exact one-file contract unchanged |
| B2 | exact fd flags/mode currently restored, but panic window remains | flag child owns null/pipe fd 0; mode child owns a PTY slave/console; neither inherits parent fd 0 | any stdin/terminal consumer in process; mode mutation must actually execute | `SubprocessIsolation` | Harness; exact two tests in `02` |
| B3 | OS closes local owned handles; normal RAII/unwind | join/await before fixture drop | unique owned endpoints only | `ProvenConcurrencySafe` | frozen |
| C1 | exact `PathBuf`; unwind-safe; stack-safe/rejected nesting | retain through dependent async/child work | all CWD mutators and stable root readers; forced race proven | `UnifiedProcessStateLock` | Harness; ten exact files in `02` |
| C2 | child restores exact mode; child exit is final containment | parent waits child | no parent-process competitor | `ProvenConcurrencySafe` | frozen |
| C3 | no mutation/restoration | none | no pair exists | `ProvenConcurrencySafe` | frozen |
| D1 | global API has no owner-scoped exact restore; reset is not composable | writers can outlive helper return | routing versus manager trace owner; forced race proven | `SubprocessIsolation` | Harness; exact helper callers in `02` |
| D2 | explicit callback lifetime replaces global install/clear; unwind-safe | callback belongs to one test server episode | nine hook installers; forced replacement proven | `ExplicitDependencyInjection` | Harness; private test sections in orchestrator file |
| D3 | production lifecycle restoration not a test concern | product-owned signal task/thread | no `execute_with_pty` caller or installer in this unit-test binary | `SeparatelyOwnedDeferred` | production PTY/signal lifecycle; cannot perturb this wall because source closure proves it is never installed here |
| D4 | no mutation | none | no pair exists | `ProvenConcurrencySafe` | frozen |
| E1 | coordinator owns exact restore, panic/poison, stack nesting | same as A1/A2 | every lock island; topology defect source-proven | `UnifiedProcessStateLock` | combined Harness; `execution/mod.rs` test-only coordinator |
| E2 | no parent restoration: bounded child exit discards its registry after owned event work ends | wait/reap child before parent test returns | channel initializers versus unguarded parent readers; child containment removes the pair | `SubprocessIsolation` | Harness; exact ten test entry points in four files in `02`; resource and production symbols frozen |
| E3 | parent cache is intentionally non-resettable | helper process exit discards cache | mutated env test versus later report reader; persistence proven | `SubprocessIsolation` | Harness; exact socket-activation test only |
| E4 | keyed replacement under mutex; panic-safe lock semantics | values scoped by exact path/stat key | different TempDirs cannot share key | `ProvenConcurrencySafe` | frozen |
| E5 | no global reset; unique fixture key removes collision | task releases production guard normally | repeated literal session IDs; cap theft proven | `ExplicitDependencyInjection` | Harness; test ID constructors only |
| E6 | atomic swap/restore is not owner-scoped | product PTY work may overlap | sole mutator versus routing readers | `SubprocessIsolation` | Harness; exact one test only |
| E7 | exact test override restore under environment coordinator | await world fixture work before restore | two writers/readers share same lane | `UnifiedProcessStateLock` | combined Harness; existing test sites |
| E8 | non-resettable initialization is immutable after set | platform-local | inactive cfg in Linux wall; no mutation | `ProvenConcurrencySafe` | frozen/platform owner |
| E9 | immutable/atomic semantics; no restoration | thread-safe | no environment/path-dependent mutable payload | `ProvenConcurrencySafe` | frozen |
| E10 | no composable exact restore for overlapping owners; RAII only clears its own current slot | bounded child owns all control/task lifetime and is awaited | two `ActivePtyGuard` owners; forced replacement proven | `SubprocessIsolation` | Harness; exact one test in `02` |
| E11 | no owner-scoped broker reset; process exit discards the test mutation | bounded child is waited/reaped before parent returns | broker mutator versus stable `policy_mode` reader; forced pair proven | `SubprocessIsolation` | Harness; exact 23 tests in `02` |
| F1 | fixture-owned path removed only after server termination; unwind cleanup | confirm task exit and socket unlink | predictable same path across tests/processes; stale failure proven | `ExplicitDependencyInjection` | Harness; exact 62 test bodies and test-owned root construction in two files in `02` |
| F2 | TempDir/socket RAII after awaited owner | await/join confirmed | unique roots/port 0 | `ProvenConcurrencySafe` | frozen |
| F3 | local endpoint ownership | child/listener waited | no fixed port found | `ProvenConcurrencySafe` | frozen |
| F4 | exact private path/CAS semantics | wait child before root drop | distinct private roots; shared ambient roots fall under A1 | `ProvenConcurrencySafe` | frozen |
| G1 | no state to restore; remove scheduling assumption | barrier/readiness wait completes deterministically | scheduler load; ordering intent source-classified | `DeterministicSynchronization` | Harness; exact 19 test waits in seven files |
| G2 | no global state; deadline starts after readiness | owner awaits/join | shared load cannot change established ordering contract | `ProvenConcurrencySafe` | frozen unless a focused test is listed in G1 |
| G3 | no override | runtime-local | none | `ProvenConcurrencySafe` | frozen |
| H1 | teardown order is binding and unwind-safe | abort/stop → await/join → verify cleanup → restore → unlock | all ten named tests; stale socket/task evidence | `TerminationConfirmedTeardown` | combined Harness; ten exact test bodies |
| H2 | await observes termination before Drop | confirmed | none after join | `ProvenConcurrencySafe` | frozen |
| H3 | join/wait/runtime completion precedes return | confirmed or runtime-scoped | no owner crosses return outside H1 | `ProvenConcurrencySafe` | frozen |
| I1 | supplemental only; no restoration authority | cannot cover reporter/unannotated work | 550 annotations versus 24 unannotated env dependents in broad closure | `UnifiedProcessStateLock` | Harness; participation migration only, no “serial and stop” |
| I2 | ENV → CWD; restore each before reverse unlock; explicit poison/nesting | retain all held lanes through owned cleanup | existing reverse/partial acquisition; event locks remain child-local | `UnifiedProcessStateLock` | Harness; test-only coordination helpers |
| I3 | no test-owned state | reporter lifetime equals process | reporter only conflicts with B1 descriptor capture | `ProvenConcurrencySafe` | runner frozen; no suppression/thread-count gate |

Disposition totals reconcile exactly: 7 `UnifiedProcessStateLock`, 4
`ExplicitDependencyInjection`, 1 `TerminationConfirmedTeardown`, 1
`DeterministicSynchronization`, 7 `SubprocessIsolation`, 17 `ProvenConcurrencySafe`, and 1
`SeparatelyOwnedDeferred` = 38. Category totals reconcile exactly: A 4 + B 3 + C 3 + D 4 + E
11 + F 4 + G 3 + H 3 + I 3 = 38. No broad-wall-capable row is deferred.

### Frozen combined implementation gate

The preserved combined F0/F0a/F0b/Harness implementation remained within the
corrected consolidated allowlists and additive file/symbol table in `02`; no new tracked file was
added. The restored candidate's direct-primitive and wrapper/callsite
inventories reconcile to zero unclassified rows. All production callers
and behavior are frozen except F0b's private writer delegation with byte-identical production
output. Focused proof covers all eleven proven interference families, exact restoration,
non-Unicode values, panic, poison, nesting, child inheritance, unique ID/path ownership, descriptor
bytes, confirmed async termination, and deterministic readiness. Every helper child uses a
recursion-proof sentinel and bounded timeout, propagates nonzero/signal status, kill-then-waits and
reaps on timeout, and reports no secret-bearing state. The B2 flag child owns a fresh null/pipe
stdin open-file description; the mode child owns a PTY slave/console and proves the guarded
terminal/console mutation actually ran. Neither inherits parent fd 0, and both prove the parent's
fd-0 flags/mode unchanged after ordinary and panic/abort child outcomes. Final proof is at least three
independent default-parallel shell-library walls and one canonical serial wall with identical
counts, failure names, and normalized signatures. That proof is complete and establishes F's clean
comparison baseline as `1280 discovered / 1235 passed / 45 failed / 0 ignored`.

F0-HC by itself authorized no implementation beyond its consolidated harness contract. The exact
candidate now marks F0, F0a, F0b, F0-HC, and the combined closeout complete. Its environment
inventory is corrected; the implementation changes no runtime/user behavior and promotes no seam.
At that historical F0-HC checkpoint, F remained unstarted and was the next packet. The completed-F
gate record below supersedes that next-task status.

### Corrected historical differential evidence-authority contract

This contract supersedes only the impossible assignment of transition-matrix authority to the
incomplete historical parallel output. It waives no regression gate.
The bounded provenance result is `HistoricalParallelArtifactUnavailable`.

1. **Historical serial semantic authority.** The authenticated complete serial wall has 1,263
   discovered, 1,202 passed, 61 failed, and 0 ignored tests, with complete names and normalized
   signatures. Its exact candidate transition matrix is 1,202 `PassToPass`, zero `PassToFail`, 45
   `FailToSameFailure`, zero `FailToChangedFailure`, 16 `FailToPass`, zero `Removed`, zero
   `RenamedOrSubstituted`, 17 `NewPass`, zero `NewFail`, and zero `NewIgnored`. Every one of the 16
   `FailToPass` rows requires an exact causal audit; every one of the 17 `NewPass` rows must be an
   added authorized test. Deterministic listings at the serial baseline and candidate must prove
   zero removed, renamed, or substituted tests.
2. **Final concurrency authority.** Three independent default-parallel walls and one canonical
   serial wall of the exact candidate must each report 1,280 discovered, 1,235 passed, 45 failed,
   and 0 ignored, with identical failure-name sets and normalized signatures. Any disagreement is
   a hard stop. This proves the repaired harness has no parallel-versus-serial semantic outcome
   change.
3. **Historical parallel diagnostic evidence.** The authenticated aggregate remains 1,263
   discovered, 1,113 passed, 150 failed, and 0 ignored. Panic headers and the final-summary tail
   from the same transcript yield all 150 names, but retained panic output is complete for only 37
   names. Because the artifact lacks complete normalized signatures, it is diagnostic only. It
   cannot prove a parallel
   `PassToFail`, a historical parallel signature comparison, membership of the final failures, or
   exactly 105 named historical failures becoming passes. Rerunning the nondeterministic historical
   code creates a new run and cannot recover this historical artifact.
4. **Source and inventory continuity.** Completion still requires the exact 45-file candidate
   manifest and per-file fingerprints, deterministic endpoint listings, the exact 17 additions in
   `02`, zero removed/renamed/substituted tests, no changed production execution flow, the exact
   serial matrix, and final four-wall identity. The manifest SHA-256 is
   `b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; the fingerprint aggregate
   is `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; the ordinary patch is
   `7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; and the full-index patch is
   `adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`.

The original exact candidate remains preserved at commit
`86ed6f5620787121b1c2e5b033ee8d6f9ff369d3` and tree
`f6480f3986d43bb41e5387fa1ba5b68ae53f598b`. Its byte-identical patch is committed on the corrected
source base as `770a6a9de9f537f7bc179c75421abbc3fff05b8d` with tree
`61fdd2e9476f1ce3e041720ce106f7c3427895be`. Environment closure, renderer/injection, and
lifecycle/subprocess implementation reviews are CLEAN. The old containment/differential review is
not reused; fresh reviewer `/root/final_containment_corrected_authority`
(`019f8681-7957-7cc3-88fa-37ab3ad2fc87`) returned CLEAN under this contract.

`PassToFail=0`, `FailToChangedFailure=0`, `Removed=0`, `RenamedOrSubstituted=0`, `NewFail=0`, and
`NewIgnored=0` are satisfied hard requirements. At that historical harness checkpoint no product
behavior changed, F0/F0a/F0b/F0-HC were complete, F remained unstarted, and no seam was promoted.
The completed-F gate record below supersedes that packet status.

### Canonical harness gate result

| Gate | Canonical result |
|---|---|
| Candidate identity | 45 files, 5,630 insertions, 2,104 deletions; commit `770a6a9de9f537f7bc179c75421abbc3fff05b8d`; tree `61fdd2e9476f1ce3e041720ce106f7c3427895be`; manifest `b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; fingerprints `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; ordinary patch `7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; full-index patch `adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4` |
| Corrected inventory | 86 names = 74 parent-mutated + 12 child-only/read-only; 395 primitives; 73 dynamic calls; 43 sinks; 1,005 resolved callsites; 534 tests across 35 files; 38 resource rows; zero unresolved dynamic rows; all 38 dispositions implemented or retained |
| Focused and quality proof | All eleven interference families plus HOME/socket overlap, renderer isolation, negative projection, environment/CWD restoration, fork publication, async termination, subprocess isolation, exact absence/non-Unicode, panic/poison/nesting/reacquisition, and child status propagation; shell/workspace all-target checks, Clippy `-D warnings`, format, and diff checks pass |
| Final concurrency authority | Three parallel walls plus one serial wall each `1280/1235/45/0`; common failure-name hash `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; common normalized-signature hash `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |
| Serial semantic authority | `1202 PassToPass`; `0 PassToFail`; `45 FailToSameFailure`; `0 FailToChangedFailure`; `16 FailToPass`; `0 Removed`; `0 RenamedOrSubstituted`; `17 NewPass`; `0 NewFail`; `0 NewIgnored` |
| Transition audit | The 16 `FailToPass` rows are causally tied to seven detached-availability, two fork-publication, six stop-dispatch, and one hidden-owner harness fixes. The 17 `NewPass` rows are exactly the authorized tests listed in `02`. |
| Historical parallel limitation | `1263/1113/150/0` is diagnostic only; all 150 names are recovered, only 37 complete panic bodies remain, and 113 normalized signatures are unavailable. No historical parallel transition matrix or 105-transition claim exists. |
| Containment and review | GitNexus's one MEDIUM process label resolves to test-only `AuthorityEnvTestTempDir::new`; no production flow changes. The only production hunk is authorized mechanical F0b delegation with byte-identical output. Three implementation reviews and the fresh containment review are CLEAN. |
| Historical status and sequence | F0/F0a/F0b/F0-HC complete; no production or user-facing behavior change; no seam promotion; `Routes A–E → F0/F0a/F0b/F0-HC complete → F → renewed R2-2 closeout → R2-3 → R2-4 → R3`; F was unstarted at that checkpoint. The completed-F gate record below supersedes this status. |

## A1.1d-5R2-2F readiness and outbound-environment correction

This contract supersedes the earlier F builder/readiness boundary. F1 and F2 are complete local
commits; F3/F4 are incomplete, blocked, and preserved; F itself is not complete.

### Exact outbound command environment

The authenticated builder constructs this entire `ExecuteRequest.env` map from immutable guest
constants. Values shown here are generated values, not forwarded parent `HOME`, XDG, PATH, or other
environment state.

| Exact name | Exact value | Classification and reason |
|---|---|---|
| `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` | `/var/lib/substrate/world-deps/bin` | Required non-secret guest runtime projection; ambient override forbidden. |
| `PATH` | `/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin` | Required deterministic guest command discovery; not inherited. |
| `HOME` | `/root` | Fixed guest execution home required by the existing world command envelope; host HOME is never read or copied. |
| `XDG_CONFIG_HOME` | `/root/.config` | Fixed guest runtime path; ambient XDG input is never read or copied. |
| `XDG_DATA_HOME` | `/root/.local/share` | Fixed guest runtime path; ambient XDG input is never read or copied. |
| `XDG_CACHE_HOME` | `/root/.cache` | Fixed guest runtime path; ambient XDG input is never read or copied. |
| `TERM` | `xterm-256color` | Fixed non-secret command-runtime posture; ambient TERM is not inherited. |

No other entry is allowed. Locale, timezone, and color variables are unnecessary because source
closure found no authenticated-path requirement for them. CWD, request profile, policy snapshot,
world-network policy, and filesystem mode remain exact structured `ExecuteRequest` fields.
World-service already derives any internal enforcement environment from those structured fields;
the shell must not duplicate it in the outbound command environment. No schema or side channel is
added.

### Forbidden input and forwarding table

| Class | Exact examples or pattern | Required disposition |
|---|---|---|
| Wildcard/prefix forwarding | all ambient `SUBSTRATE_*`, all ambient `WORLD_*`, all `LC_*`, or the complete parent environment | FORBIDDEN. An allowlist must enumerate all seven names above and generate every value. |
| Authority selectors | `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, `SUBSTRATE_WORLD_SOCKET`, `HOME`, all ambient XDG variables, `CODEX_HOME`, anchor/project/workspace selectors | FORBIDDEN. They cannot accompany A as command authority. The fixed guest values above are constants, not selectors. |
| Service/backend selectors | `SUBSTRATE_WORLD_AGENT_BIN`, socket-activation overrides/timeouts, world backend/socket/request-profile selectors, guest-bin-dir override | FORBIDDEN in request environment and forbidden as explicit-F readiness input. |
| Policy/config selectors | policy-mode, world network/fs/caging selectors, enforcement-plan payloads, config roots, generated manager/bootstrap selectors | FORBIDDEN. Structured authenticated request fields remain authoritative. |
| Hidden authority carriers | bootstrap carrier, install context/home, commitment preimage, selected user/UID, shim caller/stack/depth state | FORBIDDEN. No hidden transport can reconstruct or supplement A. |
| Secrets and credentials | any credential, session secret, API key, access key, private key, authorization value, or provider token | FORBIDDEN regardless of name or prefix. |
| Prompt/request material | prompt text, request preimage, serialized carrier/request bytes, raw policy/config payloads | FORBIDDEN. |
| Compatibility-only or unnecessary state | ambient locale, timezone, color, terminal, user/shell/temp/editor variables | EXCLUDED. No downstream precedence rule may be used as a security boundary. |

The builder may retain an existing non-environment request metadata field only if its current
non-authority semantics are source-proven and unchanged; that field cannot select A, readiness,
policy, backend, or credentials. This is not permission to add a transport field.

### Explicit readiness contract

1. `ensure_world_service_ready()` keeps its current signature and compatibility behavior.
2. One private Linux core receives the exact socket path plus the minimum private service posture.
3. The core never resolves the target socket from environment, home/root state, CWD, activation
   report path, or a global side table.
4. Compatibility delegates with legacy socket/binary inputs and retains every existing caller.
5. F delegates with `/run/substrate.sock` and immutable installed-product binary
   `/usr/local/bin/substrate-world-service`; environment selection is unavailable.
6. Probe, activation wait, stale-socket safety, spawn, readiness polling, timeouts, and error
   classification exist only in that owner.
7. Authenticated validation precedes readiness; readiness precedes request construction.
8. Existing capability, policy, systemd/service, service unit, socket, lifecycle, non-Linux,
   world-service, transport, receipt, supervisor, and retained-worker behavior is frozen.

### Required future F3/F4 proof

1. Under conflicting ambient `SUBSTRATE_WORLD_SOCKET`, explicit readiness probes and connects only
   to the authenticated fixed socket; environment-only socket selection cannot affect F.
2. Existing no-argument readiness callers retain their current behavior, including compatibility
   override behavior.
3. Capability probe, activation detection/wait, stale-socket handling, compatibility spawn fallback,
   polling intervals, timeout values, and error classification/text remain equivalent.
4. Source inspection and tests prove no readiness code is duplicated in the F builder or either
   consumer.
5. The serialized runtime request contains exactly the seven environment entries above, with exact
   values, and no parent-environment entry.
6. Poisoned authority, backend, policy, HOME/XDG, shim/bootstrap, locale, and unknown-prefix values
   do not appear in the request or alter its target.
7. Synthetic credential and request markers do not appear in the request environment, serialized
   transport, errors, logs, traces, doctor output, or stored fixtures.
8. A under conflicting B executes against A. No downstream precedence or overwrite is accepted as
   proof.
9. Missing, malformed, tampered, wrong-principal, or mismatched authority fails before readiness,
   request construction, mutation, or launch.
10. Both exact consumers use the additive builder; no third consumer is redirected.
11. Existing world execution, network allow/deny, filesystem, caging, capability, lifecycle,
    receipt, supervisor, and retained-worker behavior remains unchanged.
12. Non-Linux behavior remains frozen under R2-3 ownership.

Any credential/request marker crossing the boundary, any need for a transport/world-service edit,
or any inability to preserve compatibility callers is a hard stop. Passing these tests closes only
the corrected F3/F4 work; it does not start F5 or the final F walls and does not promote a seam.

## Historical F5-PD non-mutating nested Doctor contract

This was the controlling contract at the post-F3/F4 checkpoint. Earlier text that called F3/F4
incomplete or placed F5 immediately after them was historical and superseded there. F3/F4 were
complete, F5-PD was unimplemented, and F5/later nodes were unstarted. The completed-F gate record
below supersedes this historical packet status while preserving the contract's boundary decisions.

### Compatibility invariant

Normal public `substrate world doctor --json` is behaviorally frozen. Its readiness behavior,
socket-activation observation and connection, service probing, world-service calls, filesystem and
capability probes, `/v1/execute` fallback, output, exit codes, errors, and platform behavior are not
changed or represented as side-effect-free. The Host Doctor paths and all public callers remain
unchanged as well.

### Authenticated internal mode

Whenever the existing authenticated Linux F5 composition has World enabled and reaches
`gather_world_doctor_snapshot`, the parent must still create a real product-CLI child. The frozen
World-disabled `build_report` branch continues to return `disabled_world_doctor_snapshot` without
spawning a child or consulting a fixture. The enabled parent adds a hidden argv
discriminator named `--internal-passive-world-doctor-v1` inside the existing
`WorldAction::Doctor` grammar beside the canonical encoded `InstallBootstrapContextCarrierV1` and
exactly `world doctor --json`. The child must:

1. require the hidden carrier on argv; environment state cannot select or repair the mode;
2. require an exclusive World Doctor JSON action and reject every other flag/action combination;
3. decode and validate canonical carrier bytes, commitment, selected prefix, current Unix
   account+UID, optional declared selector, and conflicting checked projections;
4. branch immediately after `decode_and_bind_unix_install_bootstrap_context` and before
   `install_bootstrap_projections`;
5. avoid home scaffold, trace setup, config/policy/platform/socket/service selection, and ordinary
   routing;
6. emit only the existing bounded diagnostic envelope with `ok=false`, exact non-secret A
   diagnostic identity, platform label, and unavailable status, then return the existing
   diagnostic failure class; and
7. never fabricate coherent success.

The initial child JSON shape is exact and private to this authenticated mode:

```json
{
  "schema_version": 1,
  "platform": "<compile-time target>",
  "ok": false,
  "host": {
    "platform": "<compile-time target>",
    "ok": false,
    "selected_host_prefix": "<validated A>",
    "host_context_commitment": "<validated non-secret commitment>"
  },
  "world": {
    "status": "unavailable",
    "ok": false,
    "selected_host_prefix": "<validated A>",
    "host_context_commitment": "<validated non-secret commitment>"
  }
}
```

No additional field is permitted. Both identity copies come from the same validated typed carrier;
they are diagnostic proof for the parent validator, not independent authority or health evidence.
The child exits 4 after emitting the JSON. F5 composition may map that bounded failure through its
existing `NeedsAttention`/unavailable path but cannot retain it as healthy.

Whenever the Linux World-enabled branch reaches the gather function, the parent spawns this child
and does not inspect `A/health/world_doctor.json`. At the raw-stdout boundary,
`run_json_subcommand` must decode through exact private
`#[serde(deny_unknown_fields)]` wire structs before constructing a `serde_json::Value`; this rejects
duplicate or unknown fields before they can be collapsed. The Linux command decoder then receives
expected A prefix/commitment and accepts only the exact object above, exit 4, and empty stderr. It
discards the raw result and constructs only `WorldDoctorStatus::NeedsAttention`,
`ok=false`, platform from the compile-time label, source `command`, exit 4, empty stderr/details,
and error `passive world doctor unavailable`. Any absent/additional/duplicate/malformed/mixed/tampered field,
wrong platform/identity/exit, nonempty stderr, or secret/request-bearing key constructs the same
bounded closed shape with error `passive world doctor incoherent`; rejected bytes are never echoed.
The Linux value decoder becomes `cfg(test)`-only and direct fixtures pass through the same
validator. Non-Linux gather/decoder/fixture/public-child compatibility remains unchanged and is
barred from authenticated/native proof.
The validator recursively normalizes key case and separators and rejects credential, token,
API/private-key, authorization, password, secret, prompt, request/body/bytes/input,
carrier/auth-bundle, parent/full-environment, and commitment-preimage keys. It does not scan by
printing values and does not include a rejected key or value in any error.

The mode must not call service readiness; start/restart world-service; stat/connect/create/remove
an activation socket; invoke `socket_activation_report`; call `/v1/capabilities`,
`/v1/doctor/world`, `/v1/execute`, or a WebSocket endpoint; construct a Tokio/runtime transport;
create a world/overlay/probe; read active capability/network/filesystem state; install, provision,
sync, migrate, lock-promote, touch, truncate, rewrite, repair, or clean anything; recover authority
from environment, HOME/XDG, CWD, account defaults, repository paths, or global state; or expose
carrier/credential/request/prompt/preimage bytes.

### Evidence and truthful classification

The authenticated carrier is authoritative only for A identity. Compile-time platform is a
diagnostic label. Neither is runtime-health proof. No existing production artifact is presently
authorized as durable coherent World Doctor evidence. Static socket/unit/process/filesystem state,
even when read-only, is insufficient because it cannot prove endpoint/world/probe coherence;
inactive, absent, stale, or unprovable state yields unavailable and must not trigger startup.

The existing `WorldDoctorSnapshot`, `WorldDoctorStatus`, `WorldDoctorReportV1`, human renderer, and
public transport JSON remain frozen. The existing `NeedsAttention` plus bounded error
representation is the contract-owned fail-closed class for passive unavailable/incoherent state;
`details` and `stderr` are `None` for both outcomes.
The child exits through the existing failure class so human output names the unavailable reason;
JSON carries `ok=false` and the same class. A malformed, mixed, stale, conflicting, secret-bearing,
or tampered result is incoherent/closed and is not retained as a healthy payload. This is honest
without adding an enum variant or changing a wire field.

If implementation discovers that this representation cannot express the same truthful human and
JSON outcome, it stops as `CrossDocumentChangeRequired`; F5-PD does not authorize a schema change.
Likewise, finding a candidate durable evidence source does not authorize its use: source-close its
authority, freshness, nonmutation, identity, platform behavior, and impact in a separate decision
first. Do not add a side table, endpoint, daemon, broker, or persistence format.

### Exact implementation allowlist

| File | Existing symbol authorization | Additive private authorization | Frozen boundary |
|---|---|---|---|
| `crates/shell/src/execution/cli.rs` | EDIT `WorldAction::Doctor` only to add hidden bool `internal_passive_world_doctor_v1` and its colocated parser tests | None | Top-level `Cli`, every other field/variant, normal `world doctor [--json]` grammar/help/behavior |
| `crates/shell/src/execution/routing.rs` | EDIT `run_shell_with_cli` only for Linux exhaustive hidden-action validation/early return and non-Linux hidden-bit rejection | ADD one Linux-only private `passive_world_doctor_action_is_exclusive` predicate | `run_shell`, normal/no-bit action ordering, authentication owner, projection/scaffold/config/trace behavior for every other action |
| `crates/shell/src/execution/platform/mod.rs` | EDIT `handle_world_command` only to reject a leaked true hidden bit before the Doctor arm performs work | ADD one Linux-only, at-most-`pub(super)` `emit_authenticated_passive_world_doctor_v1` accepting `&InstallBootstrapContextCarrierV1` and returning the existing CLI result/exit form; colocated tests only | False-bit public Doctor body, `handle_host_command`, `resolve_doctor_world_disable_attribution`, platform adapters and all public Doctor semantics |
| `crates/shell/src/builtins/shim_doctor/report.rs` | EDIT only Linux cfg implementations of `gather_world_doctor_snapshot` when reached from the frozen enabled branch to remove production World Doctor fixture selection and add the hidden child discriminator; `run_json_subcommand` only at its Linux raw-stdout parse expression for strict private-wire decode; `snapshot_from_command` for expected-A/exact-schema/exit/redaction validation and bounded construction; and `snapshot_from_value` to become Linux-test-only and use the same validator | ADD exactly Linux-private `PassiveWorldDoctorChildV1`, `PassiveWorldDoctorHostV1`, and `PassiveWorldDoctorWorldV1` structs with `deny_unknown_fields`, plus `decode_passive_world_doctor_child_v1` and `validate_passive_world_doctor_child_v1` helpers | `build_report`, `disabled_world_doctor_snapshot`, `JsonCommandOutput`, all non-Linux implementations, every other statement in `run_json_subcommand`, `WorldDoctorSnapshot`, `WorldDoctorStatus`, report/wire schemas, renderer, world-deps fixture/composition, and broader F5 composition |

Permitted integration test files are exactly `crates/shell/tests/doctor_scopes_ds0.rs`,
`crates/shell/tests/shim_doctor.rs`, and `crates/shell/tests/shim_health.rs`. Colocated tests may be
edited only in the four production files above. No new file, re-export, public signature,
directory/module-wide authority, or third-party dependency is permitted.

The sole public Rust type-layout change is the additive hidden bool inside
`WorldAction::Doctor`; top-level `Cli` and `auto_sync.rs::cli_for_auto_sync` remain byte-for-byte
unchanged. Linux-private `snapshot_from_command` and Linux-test-only value decoding may add expected
A prefix/commitment parameters through cfg-specific implementations. All non-Linux private
signatures/bodies and all other existing signatures/visibility remain byte-for-byte unchanged. The
emitter is not exported outside the `execution` parent. Every non-Linux target rejects the hidden
mode and retains its ordinary compatibility behavior without a native/A-bound claim.

### Exact impact authorization

The existing edited symbols are LOW: `WorldAction` has 0 direct/0 total graph dependents;
`run_shell_with_cli` has one direct/two total in Execution and no attributed process;
`handle_world_command` has 0/0; `gather_world_doctor_snapshot` and `run_json_subcommand` each have
0/0; and
`snapshot_from_command`/`snapshot_from_value` each have one direct/one total caller in Shim-doctor
and no attributed process. Frozen `build_report` has one direct/five total, one process/one module;
frozen `disabled_world_doctor_snapshot` has one/six, one/two. The seven additive private symbols
have no pre-edit node. Those are the complete impact authorizations.

The following observations are explicit non-authorizations: `install_bootstrap_projections` HIGH
(4 direct/11 total, one process, three modules); `ShellConfig::from_cli` HIGH (7/8, one/three);
`WorldDoctorSnapshot` HIGH (4/10, one/three); `WorldDoctorReportV1` HIGH (2/7, one/four);
`ensure_world_service_ready` CRITICAL (4 direct/10 total, four/five); and
`socket_activation_report` CRITICAL (6 direct/12 total, four/five). Their bodies, signatures,
callers, process families, and module ownership are
frozen. The passive branch may occur before them; it may not edit, wrap, redirect, or call them.
The two CRITICAL lifecycle values are the test-inclusive `maxDepth=5` GitNexus results; the
test-excluded view is narrower and is not used as authorization evidence.

### Required proof

Implementation exit requires all twenty exact regressions:

1. Missing or malformed authenticated carrier rejects before observation.
2. Environment-only authority rejects.
3. Conflicting ambient B cannot affect the passive child.
4. Inactive service remains inactive; the frozen World-disabled branch returns its current
   disabled snapshot without spawning a child.
5. No activation socket connection occurs.
6. No service process starts.
7. No `/v1/doctor/world` request occurs.
8. No `/v1/execute` request occurs.
9. No world or filesystem probe is created.
10. No config, policy, metadata, socket, fixture, or service state mutates.
11. Existing passive evidence for exact A can compose truthfully. In the initial implementation,
    the existing permitted evidence proves only authenticated A identity, so truthful composition
    is unavailable, not coherent health.
12. Missing evidence returns unavailable.
13. Mixed A/B evidence returns incoherent or the exact contract-owned fail-closed class.
14. Malformed, tampered, duplicate-key, or unknown-field evidence fails closed before lossy value
    construction; include conflicting top-level and nested duplicates and B-then-A identity keys.
15. JSON and human classifications agree.
16. No credential, prompt, request, carrier, preimage, unselected host-path, or synthetic marker
    leaks; the already-public selected-prefix/commitment diagnostic fields remain bounded.
17. Normal public `world doctor --json` compatibility tests remain unchanged.
18. F3/F4 exact environment and readiness tests remain unchanged.
19. Non-Linux compatibility remains frozen and no native proof is fabricated.
20. No test is removed, renamed, ignored, substituted, or weakened.

The service/socket proof must observe accepts, requests, process identity, and before/after state
without invoking product lifecycle actions. The no-mutation proof snapshots exact relevant paths,
metadata, content hashes, socket identity, and service state before and after. Marker scans cover
stdout, stderr, JSON, human output, logs/traces created by the test harness, and retained fixture
payloads. Test fixtures never become production evidence.

### Mandatory stops and exit sequence

Stop on `RepositoryStateMismatch`; `ControlPackStateMismatch`; unapproved HIGH/CRITICAL impact;
new module/process-family ownership; transport/report-schema need; world-service or lifecycle
behavior need; inability to avoid socket activation; inability to represent unavailable/
incoherent truthfully; credential/request/carrier/preimage or unselected-path leakage; public or
product behavior regression; non-deterministic differential; or review infrastructure failure.
No test result waives a stop condition.

After implementation, focused proof and three fresh isolated read-only reviews must be CLEAN before
F5 can resume. Then the only legal sequence is `F5-PD → F5 → final F walls → F closeout → renewed
R2-2 integration closeout`. R2-3, R2-4, and R3 remain later and unstarted.

## A1.1d-5R2-2F completed gate record

The required sequence above completed through F closeout. It did not enter renewed R2-2
integration closeout. The following record is binding for the completed F boundary.

### Final composition contract

1. Request-scoped authenticated A is the sole identity owner. Selected prefix, non-secret
   commitment, CWD/workspace scope, config, inventory, dependency scope, passive child, and every
   retained diagnostic constituent must join to that exact A.
2. Linux `collect_doctor_snapshot_v1` validates A and reads A-derived config/global inventory plus
   request-scoped launch-CWD/workspace inventory through the shared authenticated context. It
   performs no applied/runtime-health probe and reports the bounded unavailable reason.
3. Linux no-fixture World-enabled composition uses the frozen F5-PD authenticated passive child.
   Carrier identity is not runtime-health authority. Missing evidence is unavailable, never success.
4. `A/health/world_deps.json` remains compatibility/test evidence only. Its physical canonical path,
   identity, commitment, CWD, enums, item shape, and duplicates are checked. Runtime claims are
   rejected, and accepted fixture bytes are not retained as production truth.
5. Duplicate, unknown, malformed, partial, stale, mixed A/B, tampered, conflicting, or
   secret-bearing evidence is incoherent/fail-closed. Rejected raw values are not echoed to JSON,
   human output, errors, logs, traces, snapshots, or retained fixtures/state.
6. World-disabled composition returns the pre-existing disabled snapshot before any child, fixture,
   or runtime evidence selection.
7. Coherent success still requires all contract-mandated adequate runtime evidence. Because no such
   passive evidence exists, current Linux authenticated composition truthfully remains unavailable.
8. Public `world doctor --json`, report/transport schemas, readiness, exact seven-entry generated
   environment, socket/service lifecycle, world-service, execute, probes, policy/network/world-fs,
   caging/capability, secure-FD, receipt/supervisor, cleanup/rollback, and non-Linux compatibility
   remain frozen.

### Final regression authority

| Gate | Final result |
|---|---|
| Shell library, parallel wall 1 | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Shell library, parallel wall 2 | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Shell library, parallel wall 3 | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Shell library, serial wall | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Failure names | 45; SHA-256 `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` |
| Normalized signatures | 45; SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |
| Clean F5-PD → F5 transition | two library `NewPass`, three focused integration `NewPass`; every adverse/removal/weakening transition zero |
| F5 doctor/Health suites | shim-doctor 20/20; shim-health 8/8; world-enable 21/21 |
| Auth/request/readiness/passive/policy focused library suites | 97/97 across the exact eight recorded filters |
| Managed secure-FD regressions | common 46/46; world-service launcher 18/18; gateway receiver 18/18; bounded gateway runtime 21/21 |
| Policy/network/capability regressions | shell policy snapshot 10/10; policy model 14/14; broker in-process 67/67 with eight frozen pre-witness shell-spawn failures; world-service routing 6/6 plus four exact authority/netfilter tests |
| Compilation and formatting | shell all-target, workspace all-target, `cargo fmt --all -- --check`, and `git diff --check` pass |
| Clippy | raw `-D warnings` retains exactly 20 inherited `needless_borrow` findings and no other warning; differential allows only that lint and passes |
| GitNexus | LOW; four F5 files; 27 mapped symbols; zero affected flows; no new process family |

The auxiliary full world-service unit run completed 131 tests without failure before two existing
FUSE doctor tests stalled on environmental unmount cleanup; it was terminated and is not claimed as
a full pass, privileged smoke, or a substitute for the bounded gates above.

Fresh whole-F read-only reviews `/root/f_final_authority_security`,
`/root/f_final_lifecycle_regression`, and `/root/f_final_source_platform` are CLEAN. F5 preservation
is `feat/preserve-a1-f5-runtime-20260722`; review-clean complete-F preservation is
`feat/preserve-a1-f-complete-runtime-20260722`. The F5-PD and blocked-donor preservation refs remain
unchanged, and the blocked donor was neither merged, cherry-picked, nor restored.

At that completed-F checkpoint, the exact historical next gate was **A1.1d-5R2-2 renewed
production-fix-free integration closeout**. It was a new proof node, not part of that F closeout,
and could not repair production code. The remediation-planning gate below supersedes only that
next-gate disposition. R2-3, R2-4, R3, privileged product smoke, and direct-member Codex/UAA
gateway adoption remain outside the later closeout.

## Canonical shell-library broad-wall invocation contract

This is the single normative command contract for every live reference in the six-file control
pack to a broad shell wall, parallel wall, serial wall, canonical or differential baseline wall,
final change-detection wall, renewed closeout wall, or F/Harness wall. Historical transcripts may
retain the command they actually ran only when they are explicitly labeled historical and
non-normative. No other live command form may establish shell-library broad-wall authority.

### Fresh private root and preflight

Every broad wall creates a fresh disposable root `R`. Before the test process starts, no-follow
descriptor validation must prove all of the following for `R` and retain enough descriptor
identity to prove the same object remained bound through the wall:

1. `R` is an absolute path to a directory, not a symlink, owned by the current user, with exact
   mode `0700` and stable filesystem identity.
2. No group or world write authority exists, and no access or default ACL grants effective write
   authority to another principal. Unsupported, malformed, unreadable, or otherwise uncertain ACL
   state fails closed.
3. The complete ancestor chain is no-follow safe, identity-stable, and free of foreign effective
   write authority. `R` is created beneath a trusted, non-world-writable ancestor and is never
   beneath `/tmp`, `/var/tmp`, or another shared sticky directory.
4. `R` is not the product's real `SUBSTRATE_HOME`, is not reused by another concurrent wall, and
   cannot select or modify product/user authority state.
5. Fresh `R/tmp` and `R/xdg-runtime` exist before Cargo starts. Each is a current-user-owned,
   exact-`0700`, non-symlink directory whose descriptor identity remains stable through the wall.

The exact test environment is:

```text
TMPDIR="$R/tmp"
XDG_RUNTIME_DIR="$R/xdg-runtime"
```

Both values must be exported or prefixed on the exact Cargo invocation before the test process
starts. Ambient `TMPDIR` or `XDG_RUNTIME_DIR` is never authority. Omission or invalidity of either
value makes the wall ineligible; it does not establish a regression or authorize a baseline
transition.

### Canonical command forms

Default-parallel shell-library wall:

```bash
TMPDIR="$R/tmp" \
XDG_RUNTIME_DIR="$R/xdg-runtime" \
cargo test -p shell --lib -- --nocapture
```

One-thread serial shell-library wall:

```bash
TMPDIR="$R/tmp" \
XDG_RUNTIME_DIR="$R/xdg-runtime" \
cargo test -p shell --lib -- --nocapture --test-threads=1
```

P1 implements—not supersedes—these command forms. Inside its Stage-B containment, Cargo sees the
same lexical CWD, exact argv, and exact inherited environment E0 with only the two prefix
insertions/replacements above. P1 adds no Cargo flag or environment key, does not set
`CARGO_HOME`, `CARGO_TARGET_DIR`, `CARGO_NET_OFFLINE`, `CARGO`, `RUSTC`, `PATH`, or `HOME`, and
does not consume the repository's host `target`. Its private authenticated snapshots are mounted
at the same lexical repository/rustup-home/Cargo-home paths and its fresh writable target is
mounted at the canonical default `<repository>/target`, making containment invisible to command
semantics. The copied Cargo home preserves the exact `cargo -> rustup` and `rustc -> rustup`
symlinks, while the copied rustup home preserves the selected 1.89 toolchain and settings; the
verified repository `rust-toolchain.toml` remains the selection authority. Any
argv/CWD/environment or resolution-chain difference is `BaselineCommandMismatchConfirmed` and
the wall is ineligible.

If a live Rust/libtest version requires a different argument order, evidence must record the exact
verified equivalent while preserving these semantics: the same shell library target, `--nocapture`,
default libtest parallelism for the parallel form, exactly one libtest thread for the serial form,
and both validated private-root variables installed before process start.

### Reproducible creation and cleanup template

The following is a normative semantic template, not a repository script. The selected parent must
pass the trusted-ancestor preflight before `mktemp` runs. The `require_*`, `wait_*`, `preserve_*`,
and `remove_*` names denote mandatory harness operations backed by descriptor-based, no-follow
validation; they are not optional comments or ambient `PATH` commands. A harness must implement
them, retain the validated descriptors, and stop nonzero if any operation is absent, fails, or is
uncertain.

```bash
trusted_parent="${SUBSTRATE_TEST_TRUSTED_PARENT:-$HOME}"

# Before creation: validate the exact absolute trusted parent and its complete
# ancestor chain for owner, directory type, no symlink, mode, identity, and
# effective ACL safety. Reject shared sticky parents such as /tmp and /var/tmp.
require_trusted_wall_parent "$trusted_parent" || exit 1
wall_root="$(mktemp -d "${trusted_parent%/}/substrate-r2-wall.XXXXXX")" || exit 1
[ -n "$wall_root" ] || exit 1

chmod 0700 "$wall_root" || exit 1
mkdir -m 0700 "$wall_root/tmp" "$wall_root/xdg-runtime" || exit 1

# Before Cargo: no-follow open and validate exact owner, directory type, mode,
# identity, trusted ancestor chain, and effective ACL safety for wall_root and
# both children. Retain their identities through the wall.
require_validated_wall_root "$wall_root" "$wall_root/tmp" "$wall_root/xdg-runtime" || exit 1

if TMPDIR="$wall_root/tmp" \
   XDG_RUNTIME_DIR="$wall_root/xdg-runtime" \
   cargo test -p shell --lib -- --nocapture; then
    status=0
else
    status=$?
fi

# Preserve the complete log, counts, failure names, normalized signatures, and
# required provenance before cleanup. Only after Cargo and every child exit:
# revalidate the exact wall_root path, owner, type, and retained identity, then
# remove only that exact disposable root. Never remove a glob or parent.
wait_for_wall_children "$wall_root" || exit 1
preserve_wall_evidence "$wall_root" "$status" || exit 1
remove_revalidated_wall_root "$wall_root" || exit 1

exit "$status"
```

`SUBSTRATE_TEST_TRUSTED_PARENT`, when present, is test-harness placement input only. It cannot
weaken any owner/mode/type/identity/ancestor/ACL check, cannot select product authority, and cannot
name a shared sticky directory. `mktemp -d` without this explicit trusted template is forbidden
because its ambient default may be `/tmp`. Cleanup occurs only after every Cargo child has
terminated, revalidates the exact root before removal, and never recursively targets a glob or the
trusted parent. Failure logs and hash artifacts are preserved before cleanup. Actual product and
user state remain untouched.

### Required provenance and eligibility

A wall result is canonical only when its retained evidence contains:

- absolute `R`, current owner UID, exact mode, directory/no-symlink proof, retained filesystem
  identity, and the complete ancestor/ACL safety result;
- exact `TMPDIR` and `XDG_RUNTIME_DIR` values;
- full Cargo command, repository working directory, Rust version, and Cargo version;
- exit status and discovered/passed/failed/ignored counts; and
- the complete failure-name hash and normalized-signature hash.

Validate this invocation provenance before comparing any result with a baseline. A failed root
preflight, missing evidence field, changed target, omitted variable, or invalid root classifies the
wall as ineligible. Correct the invocation, not trusted-root validation, tests, fixtures,
production code, or the canonical baseline, then rerun the complete required wall. Only a
provenance-valid wall may trigger `PassToFail`, `NewFail`, changed-failure, or other baseline
classification.

### Frozen baseline and command-mismatch evidence

The provenance-valid canonical shell-library baseline remains:

| Evidence | Canonical value |
|---|---|
| Counts | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Failure-name SHA-256 | `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` |
| Normalized-signature SHA-256 | `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |

The renewed-closeout invocation that omitted the private roots is retained separately as
ineligible evidence:

| Evidence | Ineligible value |
|---|---|
| Command | `cargo test -p shell --lib` |
| Counts | 1,309 discovered; 1,205 passed; 104 failed; 0 ignored |
| Failure-name SHA-256 | `7d6564bbcfaaed1fdb784eb4610683a02e4464b362215761d728be3e7361d8e5` |
| Normalized-signature SHA-256 | `ae67f87fbdc155575e13467506405afbd4568fc08b9058cb4159daa539549579` |
| Additional failures | 59, all caused by `TempDir` placement beneath normal mode-`1777` `/tmp` and correct trusted-root rejection |
| Classification | `BaselineCommandMismatchConfirmed` |

The causally correct historical control used `/home/spenser/t` for both variables and restored the
canonical counts and hashes. That path is historical evidence only and is not a canonical
machine-specific path. Serial and isolated tests failed identically without the private `TMPDIR`;
the exact private-root invocation restored the 45-failure wall. Runtime source and tests were
unchanged. The 59 failures are not added to the baseline, waived, treated as product failures or
environment drift, used to weaken trusted-root validation, or used to justify serial-only proof.

## R2-3 closeout status

The canonical shell-wall runner remains proof infrastructure, not an R2-3 completion gate.
`R2-3ZP3` is permanently deferred from the blocking R2-3 path. Its unpublished commits
`2cb796ffef68c2b049376984a90ce0382e5f3980`,
`9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`, `50948dbeb921582515a34bb6b7be21c46f29d008`, and
`868994efaf132bb04c6cdd7c82da9433333c94e5` are diagnostic evidence only; they are not accepted
product source and must not be cherry-picked, pushed, or represented as landed.

The recurring canonical-runner defect is now explicit. Authenticated wall execution can materialize
honest shell results yet still finalize ineligible with `evidence_write_failed` and
`mount_teardown_failed`, because the current runner does not yet produce authenticated Stage-A
completion/teardown proof or prove hidden backing-path removal. R2-3 closeout therefore makes no
claim of eligible canonical-runner provenance; this defect remains open proof-infrastructure work
outside the R2-3 completion gate.

The accepted closeout suffix landed as `R2-3ZT1 -> R2-3ZH1 -> R2-3ZM5 -> fresh native macOS and
Windows evidence -> R2-3Z`, with landed commits `56e0a8582d562bd7e60e8f4348b4d596e1b2b36e`,
`610db8a9350c9b52496954f5c93232d885f439d9`, and
`c583c5f293644fab75d8d42bd3bcad63f114d4fe`.

The durable machine closeout evidence is the byte-identical tracked
[terminal receipt](review-control/r2-3z-terminal-receipt.json), SHA-256
`681700be6b4c483a78896573f8c982591c6f026fae566fe8c45b5b829f82e58a`, and the byte-identical
tracked [review-cycle record](review-control/r2-3z-review-cycle-record.json), SHA-256
`a3236d6ec901906d1ef84851d45995b2d595820be93d20cb40e3af1bb9db73ef`. The receipt binds terminal
subject fingerprint `sha256:2478c015df2851e05b767f81a8cc889cd1b936be38fbc11c33e816f2ff607a5a`
and terminal review verdict `CLEAN`. The tracked JSON retains its original external review paths;
the raw external review Markdown and subject-fingerprint file were not copied into this repository.
[`review-control/README.md`](review-control/README.md) records the source paths and retention limit.

- The accepted direct shell-library proof at final source is
  `1322 discovered / 1274 passed / 48 failed / 0 ignored`.
- The direct failure-name SHA-256 is
  `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`.
- The direct normalized-signature SHA-256 is
  `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`.
- Count-only equivalence remains insufficient. The historical 45-failure inventory must remain
  present, and the only additional failures may be the separately classified non-R2-3 expectations
  `builtins::shim_doctor::report::tests::world_deps_fixture_cannot_establish_runtime_health_or_cross_a`,
  `builtins::shim_doctor::report::tests::world_deps_section_forwards_authenticated_a_under_conflicting_ambient_b`,
  and `builtins::world_deps::tests::doctor_snapshot_uses_authenticated_a_under_conflicting_ambient_b_without_mutation`.
- The sole current broad shell-wall entrypoints are `make shell-lib-wall` and
  `make shell-lib-wall-serial`.
- Each Make target is Linux-only and repo-root only, validates a trusted current-user-owned
  mode-`0700` parent plus compact private `TMPDIR` and `XDG_RUNTIME_DIR`, runs exact Cargo argv,
  streams Cargo output unchanged, preserves the Cargo recipe exit status through exact-root cleanup,
  and leaves GNU Make's standard public zero/nonzero mapping intact.
- The tracked Python runner and its self-tests are retired from live repository authority. Their
  authenticated `1322 / 1277 / 45 / 0` result remains historical diagnostic evidence only.
- `R2-3ZH1` owns only the host-inbox trusted-root test helper.
- `R2-3ZM5` owns only the macOS contextless-constructor removal and typed pre-R3 smoke contract.
- The refreshed source-bound native receipts validated clean at
  `sha256:3b44f6387070b7aaea4306ae58d7f280b1cee3e163ee59219d4900ce3f53dfaf` / artifact
  `sha256:64a726d45b8bb6724fe43f566903d7b5e5ed6ef16119f14d30cd5a03edcc7e50` for
  `R2-DIAG-01` and `R2-MAP-MAC-01`, and
  `sha256:4ee942690655c1fac185244438d14e2561df52c306dea7e5428d556b530fd28c` / artifact
  `sha256:2c36a8dae9f3bcfa1c240c62a1e44f93aee0798702ae78a0076d43778b93d46e` for
  `R2-DIAG-01` and `R2-MAP-WIN-01`.
- R2-3 closes only the explicitly listed PI rows while explicitly preserving the canonical-runner
  provenance limitation, the R2-3-owned exceptions PI-059 harness-only status and
  PI-077/PI-078 byte-frozen fail-closed guards,
  PI-050 as the R2-4 guardrail, PI-080 as satisfied by earlier R2-2 Linux restart-scope work and
  not reopened here, and every R3 lifecycle/forwarding/provisioning/cleanup/rollback/convergence
  row as open.

## Normative renewed R2-2 publication contract

The selected model is **Docs-on-top → one fast-forward publication**. This section is the normative
publication contract for the renewed production-fix-free integration closeout. It becomes
operative from exact publication-authority commit
`928f94e7b4c498273b40385f7bffea9e4f949700`, whose local/remote parity at
`feat/preserve-a1-1d-5r2-2-publication-authority-20260722` must remain unchanged. That activation is
monotonic for exact descendants. The B1 invocation correction becomes controlling authority only
when its exact successor commit has local/remote parity at
`feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722` and remains an unchanged descendant of
`928f94e7`; it is not bound to the publication-authority branch. Neither activation marks the
closeout or R2-2 complete, and neither is lost when final or authorized append-only remediation
documentation follows.

### Mandatory sequence

1. Preserve the exact unchanged linear 17-commit runtime range from F docs closeout
   `2f6f1f69b3519dafff01ef543e7d260da2c37700` through
   `e5fbd2d4441d248e137d52c44e493fb0abe158f8`.
2. Retain publication-authority commit `928f94e7b4c498273b40385f7bffea9e4f949700`
   unchanged above that range and preserve it at
   `feat/preserve-a1-1d-5r2-2-publication-authority-20260722`.
3. Retain the six-file broad-wall invocation correction unchanged on exact `928f94e7` and its
   preservation ref
   `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722`.
4. Commit/review exactly this six-file remediation-planning authority on `f7ded83`, preserve
   only its dedicated planning ref, and leave the source remote unchanged.
5. Add one bounded R1 implementation commit using only its exact two-file/function/test allowlist;
   run its focused proof and obtain a fresh authority/security review CLEAN.
6. Add one bounded P1 implementation commit using only its exact two new test-harness files and
   helper/test allowlist; run all authenticated self-tests and obtain a fresh provenance/security
   review CLEAN.
7. Establish a fresh canonical baseline only through P1: three parallel walls and one serial
   wall, each with its own fresh validated private root and complete eligible evidence. Prior
   ineligible walls cannot contribute.
8. Run every focused and broad gate in the renewed production-fix-free integration closeout from
   that exact R1/P1/baseline state. During this closeout phase permit no production, test, fixture,
   script, schema, dependency, generated, configuration, or policy-implementation change.
9. If any planning, remediation, baseline, closeout, or review gate fails or stops, do not push
   the source branch. Classify the stop and obtain any new implementation authority docs-first;
   never rewrite a historical result as passing.
10. Under the preserved pre-disposition publication gate, only when integration is fully
    proof-clean and all four independent integration reviews are CLEAN, create the
    integration-clean preservation required by the closeout packet, draft the final six-file
    integration-closeout documentation on top, validate and review it to CLEAN, commit exactly
    those reviewed bytes, and rerun final post-commit documentation validation/reviews. A purely
    documentary post-commit finding permits only an append-only six-file remediation-doc successor
    followed by renewed documentation validation/reviews. A proof/topology-invalidating finding
    reruns affected proof/review. No finding authorizes rewriting or source publication while
    unresolved.
11. Immediately before publication, freshly read the source remote and require exact old OID
    `2f6f1f69b3519dafff01ef543e7d260da2c37700`. Bind an explicit expected-old-OID CAS/lease to that
    exact OID and independently prove the proposed update is a fast-forward from that OID to final
    local HEAD. The CAS/lease is only a race guard; it never authorizes a forced or non-fast-forward
    update. Publish the complete linear source branch exactly once. Any intervening ref change is
    a hard stop, even when the changed ref is an ancestor of local HEAD.
12. Require final local HEAD, upstream, and source remote to match exactly at zero ahead/zero
    behind.
13. Preserve every existing runtime commit byte-for-byte and identity-for-identity.
14. Keep every preservation-only and blocked-donor commit outside source ancestry.

Replay, rebase, commit rewriting, cherry-pick, merge commit, force push, and any multi-step source
publication are forbidden. The 17 commits already sit on the latest canonical F documentation;
F's earlier replay fulfilled its packet-specific docs-first requirement. Historical Route D's
docs-first replay was likewise Route D-specific. Another replay would add identity churn without
architectural, semantic, or evidentiary value. The renewed integration closeout certifies the
already assembled range as a whole, and its final closeout document therefore follows the range it
certifies.

The docs-on-top -> one ordinary fast-forward publication model above remains controlling. Its RP0
sequencing and the step-10 pre-disposition four-review-CLEAN gate are historical only. RP3 is
complete. RP4 product proof on exact integration commit/tree
`8c46135c861a468dea316cf9fd7d6c6bb15bddac` /
`5358497a8baec6f36e15aaef58415a759e64977d` is accepted clean by human disposition, while the raw
persistence review remains `REQUEST_CHANGES` for three cache-only orchestration/attestation
findings preserved as non-blocking process-audit debt outside RP4 product-proof scope. This exact
six-file change was the bounded RP5 closeout packet; source publication had not yet occurred in
that run and was the exact next step after the reviewed/committed RP5 docs before later completing
without rewrite.

### Required topology

At the historical B1 checkpoint, before source publication:

```text
source remote: 2f6f1f69b3519dafff01ef543e7d260da2c37700 (F docs closeout)
  ↓
17 existing runtime commits, unchanged, ending e5fbd2d4441d248e137d52c44e493fb0abe158f8
  ↓
publication-authority docs commit 928f94e7, local source only and dedicated-preservation remote
  ↓
broad-wall invocation docs commit, local source only and dedicated-preservation remote
```

At that checkpoint the source remote remained at
`2f6f1f69b3519dafff01ef543e7d260da2c37700`, the local source was 19 commits ahead after the B1
correction, and the source worktree/index/untracked set was clean. The remediation-planning
topology below supersedes only that B1 next-node state.

After a future successful closeout:

```text
source remote F docs closeout
  ↓
17 existing runtime commits, unchanged
  ↓
publication-authority docs commit
  ↓
broad-wall invocation docs commit
  ↓
remediation-planning docs commit
  ↓
bounded R1 implementation commit and focused security review
  ↓
bounded P1 implementation commit and adversarial provenance review
  ↓
fresh P1 canonical baseline
  ↓
renewed production-fix-free integration wall and reviews, with no implementation/test commit
  ↓
final integration-closeout docs commit
  ↓
optional append-only six-file remediation-doc successor(s), only for post-commit documentary findings
  ↓
one fast-forward source push; local HEAD = upstream = remote
```

The final source contains the original 17 runtime commits plus the publication-authority,
broad-wall invocation, remediation-planning, bounded R1, bounded P1, and final-closeout commits and
only any review-required append-only remediation-doc successors, with no changed commit identity,
merge commit, or force push. The ordinary CLEAN path has four closeout-related documentation
commits—publication authority, B1 invocation authority, remediation planning, and final
closeout—plus the two bounded implementation commits. Immediately before that single publication,
the source remote must still be exact
`2f6f1f69b3519dafff01ef543e7d260da2c37700`; an explicit expected-old-OID CAS/lease binds that OID,
while a separate ancestry proof and server result require a normal fast-forward update. The lease
cannot authorize a forced update, and a concurrent source-ref change must fail rather than be
silently incorporated.

### Failure and no-push posture

If any renewed integration or final documentation gate fails or stops: do not modify runtime or
tests; do not publish the source branch; retain the exact local state and preservation refs;
classify the stop; and authorize any future remediation docs-first. A purely documentary finding
may be corrected docs-only before the relevant commit and reviewed by a fresh replacement. A
purely documentary post-commit finding permits only an append-only six-file remediation-doc
successor and renewed documentation validation/review. A finding that invalidates the proof state
or topology requires the affected proof/review to rerun from the corrected preserved state. A
commit containing a post-commit finding is never rewritten by amend, rebase, or force push. The
local publication-authority commit may remain preserved, but it is authority only and must not
imply that renewed proof ran,
R2-2 passed, product smoke passed, or any seam was promoted. At that checkpoint, R2-3 remained
blocked until the successful one-fast-forward publication completed; that publication later
completed, after which the accepted R2-3 suffix and refreshed native evidence closed R2-3. R2-4
and R3 remain later.

## A1.1d-5R2-2 closeout-remediation contracts (historical RP1/RP2 record)

These preserved contracts record the closed source facts and gates that governed the completed
bounded R1 and P1 increments. This exact six-document packet makes no implementation/test change
and does not reopen those contracts.

### R1 — release dry-run authenticated-carrier non-disclosure

#### Closed source facts

At `f7ded83ef147b748678ba6b028eea959870a04fe`,
`scripts/substrate/install-substrate.sh::run_cmd` has exactly four direct calls: three in
`run_with_sudo` and one in `deploy_shims`. Only `deploy_shims` carries
`--install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}"`. Dry-run `run_cmd` renders
verbatim `$*`; the renewed-closeout sentinel therefore reported
`carrier_marker_disclosed=true`. Live execution already uses exact `"$@"`.

`run_with_sudo` was closed across its current callers and error paths. Bare tools are resolved
through a fixed privileged PATH but are not checked against a named allowlist. The finite current
set is `groupadd`, `usermod`, package managers, `install`, `systemctl`, and `rm`, plus the one
permitted absolute ACL helper. Values include account/group names, packages, artifact/TMP and
destination paths, modes/owners, unit/socket names, and ACL actions/paths/groups. No current call
supplies an authenticated carrier, credential, authorization value, commitment preimage,
prompt/request material, secret token, or poisoned ambient selector. Other installer carrier
routes use fixed dry-run placeholders and xtrace suppression. The uninstaller has no `run_cmd`
route and remains frozen.

Existing redaction helpers are not reusable authority for this contract.
`scripts/linux/world-provision.sh::show_install_context_cmd` structurally displays with `%q` and
redacts the separate carrier and environment-assignment forms, but it is display-only, omits the
accepted equals form and the missing/duplicate/malformed/end-of-options contract, and is not
coupled to byte-exact execution argv. `crates/common::{redact_sensitive, redact_process_argv}` are
Rust-only logging helpers that apply heuristic sensitive-name/value matching; `redact_sensitive`
also has raw logging behavior. Importing or extending those helpers would broaden R1 beyond its
two-file allowlist. Both files and all three helpers remain unchanged.

#### Exact implementation allowlist

Only the following future edits are authorized:

1. `scripts/substrate/install-substrate.sh`
   - add exactly one adjacent helper named
     `run_cmd_with_redacted_install_bootstrap_carrier`;
   - change only `deploy_shims` to call that helper;
   - do not edit `run_cmd`, `run_with_sudo`, `resolve_install_bootstrap_context`, carrier
     construction/validation, another installer function, or the uninstaller.
2. `tests/installers/prefix_propagation_r2_2.sh`
   - add exactly `assert_shim_dry_run_carrier_non_disclosure`, reusing existing file-local
     assertion helpers without editing them;
   - add its exact top-level test-matrix invocation;
   - do not weaken or replace an existing assertion.

No other production or test file is authorized. `tests/installers/install_state_smoke.sh` and
applicable uninstaller tests are required unchanged compatibility commands, not editable surfaces.
GitNexus does not index these shell symbols, so the caller inventory above is the authoritative
manual impact result. Security impact is **CRITICAL** because the value is authenticated authority;
the implementation blast radius is deliberately bounded to one caller and one focused test file.

#### Two-channel command contract

`run_cmd_with_redacted_install_bootstrap_carrier` shall:

1. retain the original argv array as the only live-execution input and invoke exact `"$@"`;
2. structurally scan the original array before either display or execution, without copying a
   candidate value into diagnostics;
3. build a separate display argv only when `DRY_RUN=1`;
4. walk argv by array index and recognize both `--install-bootstrap-context-v1 VALUE` and
   `--install-bootstrap-context-v1=VALUE`; consume the separate following element or the equals
   suffix without rendering it, substitute exactly
   `<redacted-authenticated-bootstrap-carrier-v1>`, and preserve the input flag form in display;
5. boundary-safely render each remaining nonsensitive array element without `eval` or command
   reconstruction;
6. scan the complete argv so reordering cannot bypass the boundary;
7. fail with a fixed nonsecret error before display or execution on a missing/empty carrier or any
   duplicate, including one separate-form plus one equals-form occurrence; the exact stderr bytes
   for this and every structural rejection in items 8–10 are
   `[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n`, with no argv
   or ambient `INSTALLER_NAME` bytes, and the exact helper return status is `2`;
8. treat an element beginning with the exact bytes `--install-bootstrap-context-v1` but not
   matching the separate flag or a first-`=` split as a protected malformed near-match; reject
   before rendering any argv or invoking the CLI, while an equals value beginning with another
   `=` is still a redacted value and is left byte-exact for live CLI rejection;
9. track an exact standalone `--` end-of-options element; any later protected separate form,
   equals form, or near-match is not a carrier option and must be rejected with the same fixed
   nonsecret error before any display or execution, so the live Clap parser cannot echo it as an
   unrecognized subcommand;
10. reject a separate-form candidate whose first byte is `-` with the same fixed nonsecret error
   before display or execution, because Clap otherwise classifies and renders it as an unknown
   argument; an attached equals-form value may begin `-` or `=` and remains redacted while its
   original argv is passed byte-exactly for generic invalid-carrier rejection;
11. never emit the carrier, digest, prefix, suffix, length, fragment, commitment preimage, or a
   regex-derived approximation.

The release installer parser and production call construct `--flag value`; the invoked Clap-based
Substrate CLI also accepts `--flag=value`. R1 adds neither form and must safely display both. It
must not remove or transform the actual carrier, weaken duplicate/malformed validation, or change
install context, prefix selection, shim deployment, exit status, or xtrace behavior.
`deploy_shims` must continue disabling inherited xtrace before the helper and restoring its exact
prior state after success or failure. After a structural rejection it restores that state and
returns exact status `2`; after a structurally valid live invocation it propagates the child's
exact status unchanged. Generic `run_cmd` output remains byte-compatible for every nonsensitive
caller.

#### R1 proof gate

The focused test must prove:

- dry-run output uses the exact fixed placeholder and contains no carrier marker;
- normal execution receives byte-exact original argv and exact argument boundaries;
- spaces, quotes, newlines, Unicode, leading dashes, and reordered arguments cannot break
  redaction;
- separate and equals forms redact without changing their live execution argv; missing, empty,
  duplicate, mixed-form duplicate, malformed near-match, separate leading-dash, and invalid
  carrier inputs, plus every protected form/near-match after `--`, fail without rendering the
  candidate value and preserve existing parser/contract behavior; every helper structural
  rejection emits only exact stderr
  `[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n`, returns exact
  status `2`, and a valid child status is propagated unchanged;
- inherited xtrace cannot disclose the value and is restored exactly;
- sentinel values representing credentials, tokens, authorization, commitment preimages,
  prompts/requests, and unselected private-path markers that exist only inside the carrier do not
  enter stdout, stderr, logs, traces, or generated files; intentionally public selected-prefix and
  executable-path dry-run displays remain compatible;
- install behavior, authenticated context validation, account/UID binding, prefix selection, and
  shim deployment remain unchanged;
- the unchanged full installer dry-run and installer/uninstaller compatibility suites remain
  CLEAN.

Any extra live disclosure is a fresh security finding and requires new docs-first authorization;
it must not silently broaden R1.

### P1 — canonical broad-wall provenance runner

#### Source-closure decision

No existing tracked test helper provides all required properties. The failed external
`run_private_wall.py` used `subprocess.run`, waited only for Cargo, unconditionally asserted child
exit, closed validation FDs, and then called pathname `shutil.rmtree`. Its wrapper made a PID
namespace but left shell/chroot/Python layers without a verified subreaper-to-eligibility
handshake. Therefore:

1. Cargo exit did not prove every descendant exited;
2. a descendant could remain alive or retain a handle;
3. closing descriptors lost object-identity continuity;
4. pathname-only removal admitted replacement ambiguity;
5. matching counts/hashes could not establish provenance.

Installer/test fixture scripts do not solve descendant containment. Production `trusted_fs`,
world cgroups, service scripts, and platform provisioning are forbidden product surfaces. One
new bounded tracked runner is therefore the minimum correction. The source-closed environment
permits exact nested unprivileged Bubblewrap user/mount/PID namespaces with the invoking UID/GID
mapped one-to-one, so a concrete bounded mechanism exists and
`BroadWallContainmentDecisionRequired` is not triggered.

The source-closure candidate inventory is exact:

| Candidate | Useful capability | Rejection or selection |
|---|---|---|
| External failed `~/.cache/substrate-a1-r2-2-renewed-closeout-20260722/run_private_wall.py` | root validation, Cargo launch, logs/counts/hashes | Rejected: parent-only `subprocess.run`, no descendant authority, closes FDs, pathname `shutil.rmtree` |
| `scripts/ci/dispatch_ci_testing.sh` embedded Python launcher | bounded child wait/timeout | Rejected: one process only; no private-root, reparenting, descriptor, ACL, deletion, or evidence contract |
| `scripts/ci/check_self_hosted_runners.sh` | CI host prerequisite inspection | Rejected: no wall launch/containment/root/evidence lifecycle |
| `tests/installers/{prefix_propagation_r2_1,prefix_propagation_r2_2,dev_shim_bootstrap_context_r2_1,world_provision_context_r2_2,standalone_cli_prefix_r2_1}.sh` | safe-parent fixture roots and trap cleanup | Rejected: fixture-specific `mktemp`/trap; no descendants, stable descriptors, mount identity, or structured evidence |
| `crates/shell/src/execution/routing/test_utils.rs` and colocated `TempDir` helpers | Rust test fixture creation | Rejected: test-local path lifetime only |
| `crates/shell/src/execution/agent_runtime/host_session_authority/trusted_fs.rs` | production owner/mode/ACL/no-follow identity | Rejected: private production authority, not a standalone wall/deletion/supervision helper |
| `crates/world/src/cgroups.rs` and world isolation modules | cgroup attachment and product-world teardown | Rejected: product/privileged state and lifecycle semantics; no P1 edit/import |
| `/usr/bin/unshare` and `/usr/bin/systemd-run --user` | namespace or user-cgroup launch | Rejected: neither combines authenticated runner bytes, immutable inputs, descendant/result handshake, continuous descriptor deletion, and evidence; systemd-run mutates user-manager state |
| `/usr/bin/bwrap` 0.11.0-1 | unprivileged nested user/mount/PID namespaces, private tmpfs, read-only projections, die-with-parent, JSON child status, default namespace reaper or explicit `--as-pid-1` command | **Selected external platform-TCB primitive** after exact identity/hash and clean nested probe. A detached-descendant probe returned the initial child status immediately under the default reaper, so default Bubblewrap wait/PID 1 is fail-safe teardown only. Stage A may use that fail-safe; Stage B must use `--as-pid-1` so the authenticated worker itself is the reported namespace PID 1 and its verified normal reap-to-`ECHILD` record is mandatory |

No other repository Python, shell, Rust test utility, cgroup, subreaper, checkpoint, or cleanup
helper supplies the combined contract. Bubblewrap is not extended or edited.

#### Historical runner allowlist (retired diagnostic-only)

The following Python-runner design is preserved as historical diagnostic evidence only. It is not
the current broad shell-wall authority, and B1_B2_1 retires the tracked runner files instead of
extending them.

The historical P1 design authorized only these new files:

1. `scripts/ci/canonical_shell_wall_runner.py`, Linux-only, Python standard library only, with:
   exact constants `BOOTSTRAP_V2_SOURCE`, `PLATFORM_STARTUP_TCB_V1`,
   `HOST_INVOCATION_ARGV_TEMPLATE_V1`, `BWRAP_STAGE_A_ARGV_TEMPLATE_V1`,
   `BWRAP_STAGE_B_ARGV_TEMPLATE_V1`, and
   `CANONICAL_STDLIB_MANIFEST_V1`; `main`, `parse_args`, `inline_sha1`, `inline_sha256`,
   `_resolved_commit_oid`,
   `_resolved_authority_commit_oid`, `_resolved_reviewed_authority_commit_oid`,
   `validate_platform_startup_tcb`, `exec_held_executable`,
   `verify_expected_head_object_chain`, `read_expected_head_blob`,
   `parse_stdlib_manifest`, `preload_validated_modules`, `load_authenticated_module`,
   `run_authenticated_self_tests`, `construct_repository_snapshot`,
   `construct_rustup_home_snapshot`, `construct_cargo_home_seed`,
   `construct_cargo_home_runtime`, `validate_cargo_home_runtime_writes`,
   `reconstruct_locked_vendor`,
   `seal_snapshot_mounts`, `create_sealed_runner_memfd`,
   `_initialize_staged_artifact_authorities`, `launch_stage_a_bwrap`,
   `launch_stage_b_bwrap`, `parse_bwrap_status`, `create_worker_control_listener`,
   `accept_authenticated_worker`, `collect_stage_b_output`,
   `write_preopened_provenance`, `_publish_evidence_directory`,
   `host_main`, `stage_a_main`, `stage_b_worker_main`,
   `set_child_subreaper`, `wait_for_containment_empty`, `terminate_contained_children`,
   `prepare_backing_root`, `verify_namespace_mount_teardown`,
   `open_validated_directory`, `validate_safe_ancestor_chain`, `read_effective_acl`,
   `statx_identity`, `revalidate_named_identity`, `verify_clean_repository`,
   `_verify_clean_repository_with_gitdir`,
   `validate_python_runtime`, `resolve_validated_executable`, `validate_rustup_resolution`,
   `hash_open_file`, `remove_tree_at`,
   `unlink_validated_entry_at`, `preserve_evidence`, `summarize_test_log`,
   `_validate_provenance_record`, `_verify_runtime_argv`,
   `_validate_stage_b_projection_authority`, `_default_provenance`,
   `_read_authenticated_staged_provenance`, `_finalize_ineligible`,
   `finalize_after_stage_a_exit`, `_run_bounded_fixture`, and `write_provenance`.
2. `scripts/ci/test_canonical_shell_wall_runner.py`, containing one
   `CanonicalShellWallRunnerTests` suite; exact fixture/helper symbols `setUp`, `tearDown`,
   `make_safe_parent`, `make_disposable_repo`, `fixture_git_environment`,
   `make_fake_rustup_home`,
   `make_fake_cargo_home_seed`, `make_registry_archives`, `invoke_bootstrap`,
   `invoke_self_test`,
   `run_bounded_fixture`, `read_final_provenance`,
   `_assert_primary_command_uses_private_umask_without_mutating_parent`,
   `_assert_stage_b_rejects_runtime_cwd_drift_from_projection_authority`, and
   `assert_ineligible`; and only the
   self-tests named below.

Fixture-only descendant repositories and any stubbed inner self-test payloads are unit coverage
for ancestry/bootstrap wiring only. They do not satisfy the authenticated proof requirement; the
required proof remains the later disposable full non-worktree checkout described in this contract.

No repository/Python dependency, generated file, configuration, production test, runtime
shell/world/policy/service, lifecycle, capability, secure-FD, receipt/supervisor, cleanup, or
placement symbol may change. The fixed interpreter is `/usr/bin/python3.13`; the runner uses
standard-library `os`/`ctypes` bindings for `prctl`, `statx`, xattr, pidfd, signal, wait,
`renameat2`, and descriptor operations. Bubblewrap alone performs namespace, UID/GID-map, tmpfs,
bind, `/proc`, and mount teardown operations under the exact reviewed argv. The runner resolves no
namespace helper or executable through ambient `PATH`, requires exact current-account
`/etc/subuid` and `/etc/subgid` entries plus exact held `/usr/bin/newuidmap` and
`/usr/bin/newgidmap`, and requires no privileged service, daemon, or product-state mutation. If an
exact Bubblewrap feature
or required Linux syscall is unavailable, it emits an ineligible/environment result and stops;
no weaker fallback is permitted. Exact isolated interpreter flags also exclude ambient Python
startup state.

#### Historical runner interface (retired diagnostic-only)

The following interface is preserved only to explain the retired diagnostic runner. It must never
be treated as the current broad shell-wall entrypoint.

The concrete host-side controller is the authenticated in-memory runner's `host_main`, executed
directly by exact root-owned `/usr/bin/python3.13` with no enclosing Bubblewrap. Platform
containment authority begins only when that controller, after authenticating the live
expected-head runner/test blobs against a reviewed authority commit and held platform inputs,
directly launches exact installed Bubblewrap
`/usr/bin/bwrap`, package `bubblewrap 0.11.0-1`, version stdout `bubblewrap 0.11.0\n`, SHA-256
`11e350f9154f8c8e30482fb087f4c3d033907452b7a5d82d027b6662d07f3ab6`, under the root-owned,
mode-`0755`, no-foreign-write-ACL platform TCB. Its exact ELF closure also includes
`/usr/lib/libcap.so.2` SHA-256
`925dd48b2062a4f434981ad351e433dd68167de69e1ed39a1ff2e69ca5d4ecb2`,
`/usr/lib/libgcc_s.so.1`
`847b4db3641fb57c5b8d5ec7ff501d77ee9e0e0a98ab3facfdfbdf57a789446e`,
`/usr/lib/libc.so.6`
`6fa19a84c327820e76051b86a8f42c1863e946519f9d3185f27db8937203780d`, and
`/usr/lib64/ld-linux-x86-64.so.2`
`2da5672efa5bba637eb7e01d485227790255088778435f1a6ea5c5db485a4a0e`.
The SHA-256 of bytewise path-sorted exact `PATH SP SHA256 LF` records for Bubblewrap plus those
four files is
`4c954d6217bbd1c42b51486388120e47f10cd834624002aa34861711f4c1f40b`. Each is root-owned,
no-foreign-write-ACL, and identity/hash pinned. Bubblewrap is the source-closed concrete
unprivileged namespace authority; a nested user/PID-namespace probe completed cleanly on this
checkpoint. `/usr/bin/python3.13` plus exact `REVIEWED_BOOTSTRAP_V2_BYTES` and authenticated
`host_main` are the concrete host-controller authority. No shell, tracked launcher pathname,
unnamed supervisor, or Stage-A self-observation is part of the success chain.

```text
executable = /usr/bin/python3.13 from the immutable platform TCB
envp       = canonical inherited wall environment E0, with every loader/Python injection key absent
argv       = /usr/bin/python3.13 -I -S -B -c REVIEWED_BOOTSTRAP_V2_BYTES
             host --mode MODE --label LABEL --timeout-seconds SECONDS
             --evidence-parent ABSOLUTE_SAFE_DIRECTORY
             --expected-head FORTY_LOWERCASE_HEX
             --authority-commit-oid FORTY_LOWERCASE_HEX
             --reviewed-authority-commit-oid FORTY_LOWERCASE_HEX
```

The displayed vector is the structural rendering of
`HOST_INVOCATION_ARGV_TEMPLATE_V1`, not one invariant runtime hash. The template is the exact
NUL-terminated concatenation of its argv elements, with seven whole-element ASCII slot tokens:
`{MODE}`, `{LABEL}`, `{TIMEOUT_DECIMAL}`, `{EVIDENCE_PARENT}`, `{EXPECTED_HEAD}`,
`{AUTHORITY_COMMIT_OID}`, and `{REVIEWED_AUTHORITY_COMMIT_OID}`. No slot may appear inside an
option name or a partial element. Runtime construction validates each value by the CLI contract,
substitutes each slot exactly once,
forbids NUL, and hashes the resulting NUL-terminated argv bytes as that wall's
`host_argv_sha256`.

Stage-A and Stage-B constants use the same framing. Their closed whole-element slot sets are:

- Stage A: `{MODE}`, `{LABEL}`, `{TIMEOUT_DECIMAL}`, `{CURRENT_UID_DECIMAL}`,
  `{CURRENT_GID_DECIMAL}`, `{STAGE_A_USERNS_FD_DECIMAL}`, `{STAGE_B_USERNS_SYNC_FD_DECIMAL}`,
  `{RUNNER_DATA_FD_DECIMAL}`, `{STATUS_FD_DECIMAL}`, `{EVIDENCE_PARTIAL}`, `{BACKING_PATH}`,
  `{REPOSITORY_SOURCE}`, `{RUSTUP_SOURCE}`, `{CARGO_SOURCE}`, `{EXPECTED_HEAD}`,
  `{AUTHORITY_COMMIT_OID}`, and `{REVIEWED_AUTHORITY_COMMIT_OID}`;
- Stage B: `{MODE}`, `{LABEL}`, `{TIMEOUT_DECIMAL}`, `{CURRENT_UID_DECIMAL}`,
  `{CURRENT_GID_DECIMAL}`, `{RUNNER_DATA_FD_DECIMAL}`, `{STATUS_FD_DECIMAL}`, `{ROOT}`,
  `{REPOSITORY_SNAPSHOT}`, `{RUSTUP_SNAPSHOT}`, `{CARGO_HOME_RUNTIME}`,
  `{CONTROL_DIRECTORY}`, `{REPOSITORY_CWD}`, `{EXPECTED_HEAD}`, `{AUTHORITY_COMMIT_OID}`, and
  `{REVIEWED_AUTHORITY_COMMIT_OID}`.

Every path slot is the already descriptor-validated exact absolute pathname for that invocation;
FD slots name only the exact explicitly passed Bubblewrap-consumed descriptors below. The host
controller propagates its already validated mode, label, and timeout only through those exact
whole-element Stage-A slots; Stage A repeats their validation and propagates the same byte-exact
values only through the corresponding Stage-B slots. They never use the status socket or
environment as authority. Unknown, missing, duplicate, partial, or unused slots reject. Actual
Stage-A/Stage-B argv hashes are
computed after substitution and recorded per wall; they are never commit trailers.

Invocation authority is not an external pathname or replaceable side artifact. The one bounded P1
implementation commit must carry, in its immutable commit message, exact unique trailers
`P1-Bootstrap-Bytes`, `P1-Bootstrap-SHA256`, `P1-Host-Argv-Template-SHA256`,
`P1-Stage-A-Argv-Template-SHA256`, and `P1-Stage-B-Argv-Template-SHA256`. The expected-HEAD runner
blob named by the reviewed authority commit contains exact `BOOTSTRAP_V2_SOURCE` and the three
template constants. Because the templates contain literal `{EXPECTED_HEAD}`,
`{AUTHORITY_COMMIT_OID}`, and `{REVIEWED_AUTHORITY_COMMIT_OID}` rather than resulting commit OIDs,
their hashes have no self-reference. The fresh P1 provenance/security reviewer independently
recomputes the byte length and four
constant/template hashes from that reviewed blob, requires exact trailer equality, records the
resulting exact P1 commit OID, and returns CLEAN only for that one commit. This commit object plus
its independently verified runner blob is
`P1InvocationAuthorityV1`; Git object framing supplies immutable identity, so no artifact file
location, owner, mode, or mutable producer exists.

Before each wall, the closeout initiator reads `BOOTSTRAP_V2_SOURCE` from that exact reviewed Git
blob—not the worktree—substitutes the host template with `{EXPECTED_HEAD}` equal to the live
descendant commit under test, `{AUTHORITY_COMMIT_OID}` equal to the reviewer-recorded P1 OID, and
`{REVIEWED_AUTHORITY_COMMIT_OID}` equal to that same reviewer-recorded P1 OID, then submits the
resulting vector through a direct array-based `execve(2)`/spawn primitive with the bootstrap as
one `-c` element. This is the only permitted control-plane operation outside the tracked runner;
it performs no shell rendering, command substitution, environment interpolation, temporary
launcher creation, or tracked-path execution. If the closeout environment cannot supply that
direct-array primitive, stop as `BroadWallContainmentDecisionRequired`; an ambient/ad hoc launcher
is not a fallback. At its first instruction V2 binds `/proc/self/cmdline` to the exact validated
runtime instantiation of `HOST_INVOCATION_ARGV_TEMPLATE_V1`, authenticates the live expected-head
commit and reviewed authority commit through retained Git plus independent object framing, proves
that the authority commit is equal to or an ancestor of the live head, requires
`--reviewed-authority-commit-oid` to exactly equal `--authority-commit-oid`, requires exact
authenticated runner/test blob continuity across those two commits, recomputes the same
constant/template trailers from the reviewed authority blob, reconstructs the exact runtime vector
from the validated slot values, and compiles `host_main`. Live `HEAD`, CLI `--expected-head`, CLI
`--authority-commit-oid`, CLI `--reviewed-authority-commit-oid`, the reviewed P1 OID,
authenticated commit objects, runner/test blobs, V2 bytes, template hashes, and per-wall actual
argv hashes must agree. Stage A and Stage B repeat the corresponding template/substitution check,
and the Stage-B authority record carries both reviewed-authority OIDs plus the projected
repository CWD. Stage B additionally binds that projected repository CWD as authenticated
projection authority. The independent evidence gate repeats every comparison against the
reviewer's recorded P1 OID and the live descendant head; `reviewed_oid_matches` is derived from
the exact equality check between the two reviewed-authority inputs rather than asserted by the
caller. Once a trusted external control plane supplies that reviewed authority OID independently
of the live descendant, caller/artifact/bootstrap co-variation cannot turn a later descendant into
the reviewed authority. The runner alone does not prove independent review for a same-head
invocation where `expected_head == authority_commit_oid == reviewed_authority_commit_oid`.

`E0` is the exact environment inherited by the canonical Cargo command at the immutable
checkpoint. P1 control is carried only in validated argv/FDs, never environment. The launcher
rejects any `LD_*`, `GLIBC_TUNABLES`, `MALLOC_*`, `PYTHON*`, locale-path, dynamic-loader, or
Python-startup injection key; all other E0 entries remain byte-identical and unreported.
`host_main` validates and retains Bubblewrap and every source descriptor, creates the evidence
partial directory and empty backing mountpoint, seals the authenticated runner blob in a memfd,
and launches Stage A as its retained direct child. Stage A uses only fixed
`--die-with-parent`, a new user/mount/PID namespace, private propagation, fixed root-owned OS-TCB
read-only projections, read-only input projections, `--ro-bind-data` of that sealed runner memfd
at one fixed private path, one private tmpfs snapshot root, `/proc`, `/dev`, and the exact
host-created evidence partial directory writable-bound at fixed
`/run/substrate-wall-evidence` plus the one exact empty backing directory hidden beneath `M1`.
The evidence parent and its siblings are not visible in Stage A. Its exact ordered argv is frozen
by `BWRAP_STAGE_A_ARGV_TEMPLATE_V1`; no optional flag, ambient path, child-surviving arbitrary
non-stdio FD, or shell expansion is accepted. The only Stage-A Bubblewrap-process exceptions are
the exact sealed runner memfd named by `--ro-bind-data` and the exact status-pipe writer named by
`--json-status-fd`. `host_main` clears close-on-exec only for those two numeric descriptors in the
forked Bubblewrap child and retains their authoritative counterparts. Bubblewrap consumes and
closes the runner-data FD before the Stage-A command starts; its monitor alone retains the status
writer through the final `exit-code` record and then closes it. The Stage-A payload inherits
neither numeric descriptor, verifies that absence in its FD table, and verifies the read-only
runner blob hash before compiling it as `stage_a_main`.
If exact Bubblewrap identity/features or nested unprivileged launch is unavailable, stop as
`BroadWallContainmentDecisionRequired`; post-start self-hashing is not a substitute.

For human review only, the structural rendering of the host-controller argv is:

```bash
/usr/bin/python3.13 -I -S -B -c 'REVIEWED_BOOTSTRAP_V2_BYTES' host \
  --mode parallel \
  --label LABEL \
  --timeout-seconds SECONDS \
  --evidence-parent ABSOLUTE_SAFE_DIRECTORY \
  --expected-head FORTY_LOWERCASE_HEX \
  --authority-commit-oid FORTY_LOWERCASE_HEX \
  --reviewed-authority-commit-oid FORTY_LOWERCASE_HEX
```

That rendering is never eligibility evidence and must not be executed through an ambient shell;
the exact direct host vector and its nested Stage-A/Stage-B vectors are mandatory.

`REVIEWED_BOOTSTRAP_V2_BYTES` is the exact LF-terminated UTF-8 bootstrap literal frozen by the P1
implementation commit's authenticated runner blob and commit trailers, passed as the single `-c`
argument. The fresh reviewer records the exact P1 commit OID and independently reproduces the
bytes, length, SHA-256, and three argv-template hashes; the bytes are never read from a worktree pathname,
environment variable, shell substitution, temporary file, or mutable external artifact. The
implementation is ineligible until that exact commit receives fresh CLEAN review. No code that
has already begun executing is claimed to authenticate its own pre-execution origin.

The following V1 prototype is retained only as rejected planning evidence. It is **not**
authorized implementation and must never be invoked: it imported pathname-backed `hashlib` and
`subprocess` before trust validation, closed the validated Git FD, then executed mutable
`/usr/bin/git` by pathname.

```python
import sys
if not (sys.flags.isolated == 1 and sys.flags.no_site == 1 and sys.flags.no_user_site == 1 and sys.flags.ignore_environment == 1 and sys.flags.dont_write_bytecode == 1 and sys.flags.safe_path):
    raise SystemExit(66)
if sys.path != ["/usr/lib/python313.zip", "/usr/lib/python3.13", "/usr/lib/python3.13/lib-dynload"]:
    raise SystemExit(66)
import hashlib
import os
import stat
import subprocess
sys.excepthook = lambda *_: os._exit(65)
PYTHON = "/usr/bin/python3.13"
GIT = "/usr/bin/git"
RUNNER = "scripts/ci/canonical_shell_wall_runner.py"
PYTHON_SHA256 = "44c4dbc2292ba0f2f1b6eba5328ceefd41261e962b129c7eb9a670b1640a1acf"
GIT_SHA256 = "1a16ab6ec08c8e22b27360c1e12bfc426aa676cac113a14a42a8530baecdf060"
def read_regular(path):
    fd = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        st = os.fstat(fd)
        if not stat.S_ISREG(st.st_mode):
            raise SystemExit(65)
        data = bytearray()
        while True:
            part = os.read(fd, 1048576)
            if not part:
                return bytes(data), st
            data.extend(part)
    finally:
        os.close(fd)
python_bytes, python_st = read_regular(PYTHON)
if hashlib.sha256(python_bytes).hexdigest() != PYTHON_SHA256:
    raise SystemExit(65)
self_st = os.stat("/proc/self/exe")
if (self_st.st_dev, self_st.st_ino) != (python_st.st_dev, python_st.st_ino):
    raise SystemExit(65)
git_bytes, _ = read_regular(GIT)
if hashlib.sha256(git_bytes).hexdigest() != GIT_SHA256:
    raise SystemExit(65)
args = sys.argv[1:]
positions = [i for i, value in enumerate(args) if value == "--expected-head"]
if not args or args[0] != "run" or len(positions) != 1 or positions[0] + 1 >= len(args):
    raise SystemExit(64)
expected = args[positions[0] + 1]
if len(expected) != 40 or any(c not in "0123456789abcdef" for c in expected):
    raise SystemExit(64)
root = os.getcwd()
if os.path.realpath(root) != root:
    raise SystemExit(65)
gitdir = os.open(os.path.join(root, ".git"), os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | os.O_NOFOLLOW)
def read_git_regular(relative):
    parts = relative.split("/")
    if any(not part or part in (".", "..") for part in parts):
        raise SystemExit(65)
    current = os.dup(gitdir)
    try:
        for part in parts[:-1]:
            following = os.open(part, os.O_RDONLY | os.O_CLOEXEC | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=current)
            os.close(current)
            current = following
        fd = os.open(parts[-1], os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW, dir_fd=current)
        try:
            st = os.fstat(fd)
            if not stat.S_ISREG(st.st_mode):
                raise SystemExit(65)
            data = bytearray()
            while True:
                part = os.read(fd, 4096)
                if not part:
                    return bytes(data)
                data.extend(part)
        finally:
            os.close(fd)
    finally:
        os.close(current)
head = read_git_regular("HEAD")
if not head.startswith(b"ref: refs/heads/") or not head.endswith(b"\n"):
    raise SystemExit(65)
ref = head[5:-1].decode("ascii", "strict")
if read_git_regular(ref) != (expected + "\n").encode("ascii"):
    raise SystemExit(65)
for forbidden in ("objects/info/alternates", "info/grafts", "shallow"):
    try:
        read_git_regular(forbidden)
    except FileNotFoundError:
        pass
    else:
        raise SystemExit(65)
env = {"LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "PATH": "/usr/bin:/bin", "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_OPTIONAL_LOCKS": "0", "GIT_NO_REPLACE_OBJECTS": "1"}
result = subprocess.run([GIT, "--git-dir=/proc/self/fd/%d" % gitdir, "--work-tree=" + root, "--no-optional-locks", "--no-replace-objects", "cat-file", "blob", expected + ":" + RUNNER], stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env, close_fds=True, pass_fds=(gitdir,), check=False)
os.close(gitdir)
if result.returncode != 0 or result.stderr:
    raise SystemExit(65)
source = result.stdout.decode("utf-8", "strict")
cmdline = open("/proc/self/cmdline", "rb").read().split(b"\0")
if len(cmdline) < 6 or cmdline[:5] != [PYTHON.encode("ascii"), b"-I", b"-S", b"-B", b"-c"]:
    raise SystemExit(65)
bootstrap = cmdline[5]
scope = {"__name__": "__main__", "__file__": "<git-blob:" + expected + ":" + RUNNER + ">", "__package__": None, "__CANONICAL_BOOTSTRAP_SHA256": hashlib.sha256(bootstrap).hexdigest(), "__CANONICAL_RUNNER_SHA256": hashlib.sha256(result.stdout).hexdigest()}
exec(compile(source, scope["__file__"], "exec", dont_inherit=True), scope, scope)
```

The rejected V1 bootstrap block is 4,598 bytes including its final LF and has SHA-256
`0c3bc02f4c83ffda76a3b901151923400905a80333449d590aed7ed331a069e1`. It executes no tracked
pathname only as a historical prototype claim; that claim is superseded and cannot establish
authority.

V2 has this closed algorithm:

1. The external platform TCB is explicit, not self-attested. On this Linux/x86-64 packet it is the
   kernel/ELF loader, exact Bubblewrap and its complete libcap/libgcc/libc/loader closure above,
   and root-owned, non-foreign-writable Python/glibc startup closure for Manjaro
   packages `python 3.13.7-1` and `glibc 2.42+r17+gd7274d718e6f-1`. It includes exact
   `/usr/bin/python3.13` hash
   `44c4dbc2292ba0f2f1b6eba5328ceefd41261e962b129c7eb9a670b1640a1acf`,
   `/usr/lib/libpython3.13.so.1.0`
   `e1c00674c629637066ac7195ce3da29bdcabc7459e575aedb10b253984882549`,
   `/usr/lib/libc.so.6`
   `6fa19a84c327820e76051b86a8f42c1863e946519f9d3185f27db8937203780d`,
   `/usr/lib/libm.so.6`
   `6c34914125e318409f1fc28503245e854aa773d6c1cee82d6824fef6fb03d0b4`,
   `/usr/lib64/ld-linux-x86-64.so.2`
   `2da5672efa5bba637eb7e01d485227790255088778435f1a6ea5c5db485a4a0e`,
   and the only pathname-backed modules loaded before the first `-c` statement:
   `encodings/{__init__,aliases,utf_8}.py` with hashes
   `78c4744d407690f321565488710b5aaf6486b5afa8d185637aa1e7633ab59cd8`,
   `cac92d68c7ea5bc0f05b448b9144e3bdf236d0b7d27ab66112e96d43aad15b3f`,
   `ba0cac060269583523ca9506473a755203037c57d466a11aa89a30a5f6756f3d`,
   and `linecache.py`
   `0939f472e54b4aadbf261d3c45a92d5aafe16e749c199917df9f4e47ce4cc4da`.
   The implementation review must confirm this exact startup set under the exact invocation,
   package versions, hashes, root ownership, modes, ancestor state, and absence of foreign
   effective write ACLs. Drift is `python_startup_tcb_invalid` and no P1 wall may start. This TCB
   is a stated platform prerequisite; the runner must not claim that code can authenticate bytes
   which necessarily executed before its first instruction.
2. Before any runner-controlled pathname import, V2 uses only `sys`, built-in/frozen
   `posix`/`os`/`stat`, builtins, and an inline audited pure-Python SHA-256 implementation. It
   revalidates and retains the TCB files, validates exact isolated flags/path and the complete
   preloaded-module set, then no-follow opens and retains exact root-owned safe `/usr/bin/git`.
   It never imports `hashlib`, `subprocess`, `ctypes`, or another pathname-backed module in this
   phase.
3. Git is SHA-validated through that retained FD. V2 requires `os.execve in os.supports_fd`,
   forks with bounded memfd/pipe capture, and calls `os.execve(git_fd, argv, fixed_env)` on the
   exact open object. The retained gitdir FD is passed only for
   `--git-dir=/proc/self/fd/N`; the Git pathname is never executed. After every Git operation V2
   re-fstats and rehashes the held FD, verifies the pathname still names it, and keeps it open
   through final runner/self-test blob authentication. Replacement makes the invocation
   ineligible but never executes the replacement. Git output is never object authority by itself:
   inline SHA-1 independently verifies `type + SP + decimal-size + NUL + content` for the expected
   commit, each traversed tree, and both final blobs; a strict parser traverses raw path components
   from verified commit tree to `scripts/ci/{canonical_shell_wall_runner.py,test_canonical_shell_wall_runner.py}`.
   Any OID/content mismatch, malformed/duplicate/ambiguous tree entry, wrong mode/type, replace,
   alternate, or path substitution rejects before compile. The same verified commit/tree/blob
   chain is the sole expected-tree authority for repository snapshot construction.
4. The held Git object reads the expected-HEAD blobs only after that independent object-chain
   proof. Every runner and self-test blob begins with
   one strict, sorted `canonical-stdlib-manifest-v1` comment header naming every built-in, frozen,
   source, and extension module it may load and every file-origin hash. V2 parses that header as
   bytes, no-follow opens and retains every file origin, validates root owner/mode/ancestor/ACL
   safety and hashes before importing it, preloads the complete union, verifies every resulting
   `__spec__.origin`, then sets `sys.path=[]` and installs a closed importer that permits only the
   already loaded manifest. No top-level runner/test import executes before this prevalidation.
   Every module FD remains open and is revalidated/rehashed after wall or self-test cleanup.
5. `host` compiles only the authenticated runner blob with synthetic
   `<git-blob:EXPECTED_HEAD:scripts/ci/canonical_shell_wall_runner.py>` and injects the V2,
   runner, TCB, and module-manifest hashes/FD identities. It rejects unless
   `/proc/self/cmdline` binds the exact validated runtime instantiation of
   `HOST_INVOCATION_ARGV_TEMPLATE_V1`. Authenticated `host_main` remains
   outside Stage A, retains the Stage-A pidfd/status and backing/evidence authority, and alone may
   finalize `eligible=true` after Stage A exits and its namespace is gone.
6. `host_main` supplies the exact authenticated runner bytes to Stage A by sealed memfd consumed
   only through Bubblewrap `--ro-bind-data`; no child depends on inherited arbitrary FDs. Stage-A
   V2 reauthenticates those bytes, then `stage_a_main` constructs three byte-owned read-only seeds
   under private tmpfs: repository, selected rustup/toolchain, and Cargo-home seed. The Cargo seed
   contains the exact `cargo`/`rustc` symlinks, copied rustup executable, and checksum-selected
   registry inputs. Stage A makes a separate writable byte-copy, `cargo-home-runtime`, before
   Stage B. Stage B read-only projects repository and rustup snapshots, but projects only that
   private Cargo runtime writable at the canonical Cargo-home path. The runtime begins as an exact
   seed copy with none of Cargo's three lock/cache metadata paths; Cargo 1.89 may create or mutate
   exactly regular files `.package-cache`, `.package-cache-mutate`, and `.global-cache`. Any other
   runtime write, created entry, removal, rename, symlink, or type/mode/owner drift is ineligible.
   The seed remains unchanged, the runtime is post-validated and descriptor-removed, and neither
   is a host alias.
7. Stage A creates current-UID mode-`0600` `AF_UNIX SOCK_SEQPACKET` listener
   `R/control/worker.sock` before nested Bubblewrap. Stage B bind-projects `R/control` at fixed
   `/run/substrate-wall/control`, authenticates the read-only runner blob supplied by its own
   sealed-memfd/`--ro-bind-data` input, and `stage_b_worker_main` connects to that fixed socket.
   Stage A accepts only `SO_PEERCRED` matching Bubblewrap's retained `child-pid`/pidfd. Both
   endpoints are close-on-exec; the listener pathname is unlinked and its absence proved before
   START. Cargo receives the original stdin unchanged and inherits no control descriptor.
   Repository/rustup read-only mounts, the private writable Cargo runtime, and namespace
   isolation make transient host/source replace-and-restore irrelevant. Stage A retains every
   in-namespace snapshot/runtime descriptor through validation and cleanup; `host_main` retains
   independent host-side teardown/finalization authority.
8. `self-test` uses the same held Git FD to authenticate both expected-HEAD blobs, registers the
   runner only as an in-memory module, compiles the test blob with its synthetic Git filename, and
   invokes exactly `CanonicalShellWallRunnerTests`; it performs no discovery, worktree import, or
   tracked-path execution. Self-test output is explicitly non-eligible evidence.

The sole self-test request uses the same host Python/TCB/E0/bootstrap direct-exec boundary and
exact argv; individual mechanics tests may launch only their bounded dummy Bubblewrap fixtures:

```text
["/usr/bin/python3.13","-I","-S","-B","-c",REVIEWED_BOOTSTRAP_V2_BYTES,
 "self-test","--evidence-parent",ABSOLUTE_SAFE_DIRECTORY,
 "--expected-head",FORTY_LOWERCASE_HEX,
 "--authority-commit-oid",FORTY_LOWERCASE_HEX,
 "--reviewed-authority-commit-oid",FORTY_LOWERCASE_HEX]
```

The authenticated loader constructs one ordered suite containing every allowlisted method below
exactly once and rejects any extra `test_*` method. It performs no `unittest` discovery. Success
requires all `101` methods and emits exact compact sorted-key stdout
`{"failed":0,"run":101,"schema":"substrate.canonical_shell_wall_runner_selftest.v1","status":"clean"}\n`,
zero stderr, and exit `0`. Loader/authority rejection exits `65`; invalid self-test CLI exits
`64`; a completed test failure emits the same four-key schema with actual integer `failed` and
`run`, status `failed`, exits `68`, and can never create wall eligibility or a canonical baseline.

`--mode serial` is the only other mode value. `host` requires seven public options:
`--mode`, `--label`, `--timeout-seconds`, `--evidence-parent`, `--expected-head`,
`--authority-commit-oid`, and `--reviewed-authority-commit-oid`. `self-test` requires four public
options: `--evidence-parent`, `--expected-head`, `--authority-commit-oid`, and
`--reviewed-authority-commit-oid`. `LABEL` must match `[a-z0-9][a-z0-9-]{0,63}`; `SECONDS` is an
integer in `1..=21600`; the evidence parent must already exist and pass the current-UID,
directory/no-follow, mode, ACL, and safe-ancestor checks; `--expected-head` must equal live
`HEAD`; `--authority-commit-oid` must equal or be an ancestor of that live head; and
`--reviewed-authority-commit-oid` must exactly equal `--authority-commit-oid`. The invocation CWD
must equal the no-symlink repository root. `host` and `self-test` are the only public roles.
`stage-a` and `stage-b-worker` are internal-only roles accepted solely with the exact sealed-blob
hash, fixed namespace/mount identity, and per-invocation private control/backing identities
created by authenticated `host_main`; a public attempt fails before setup. There is no tracked-path
execution, re-exec override, arbitrary command, root,
`TMPDIR`, `XDG_RUNTIME_DIR`, environment, shell, CWD, executable, hash, count, or baseline
override. `host_main` launches Stage A and retains host-side teardown/finalization authority;
`stage_a_main` constructs the private seeds/runtime and launches exact nested Bubblewrap Stage B
as its retained direct child; Bubblewrap status plus the fixed private credential socket
authenticate the Stage B worker. Immutable validated configuration crosses Bubblewrap only by
read-only bind/data projections and fixed argv, never assumed non-stdio FD inheritance.
Self-tests use only the authenticated in-memory loader above and may not add a public command
escape.

Before namespace setup the runner requires:

1. exact Bubblewrap identity/version/hash and nested unprivileged feature probe above, repeated
   through the same platform-TCB path before any wall;
2. exact `/usr/bin/python3.13` as its `/proc/self/exe`, SHA-256
   `44c4dbc2292ba0f2f1b6eba5328ceefd41261e962b129c7eb9a670b1640a1acf`, whose
   `["/usr/bin/python3.13", "--version"]` stdout is exact `Python 3.13.7\n`;
3. exact root-owned `/usr/bin/git`, SHA-256
   `1a16ab6ec08c8e22b27360c1e12bfc426aa676cac113a14a42a8530baecdf060`, whose
   `["/usr/bin/git", "--version"]` stdout is exact `git version 2.51.0\n`;
4. exact current-account regular executable `~/.cargo/bin/rustup`, SHA-256
   `20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c`;
   exact current-account symlinks `~/.cargo/bin/cargo -> rustup` and
   `~/.cargo/bin/rustc -> rustup`; and exact `rustup --version` stdout
   `rustup 1.28.2 (e4f3ad6f8 2025-04-28)\n` plus stderr
   ``info: This is the version for the rustup toolchain manager, not the rustc compiler.\ninfo: The currently active `rustc` version is `rustc 1.89.0 (29483883e 2025-08-04)`\n``;
5. exact account-database-home `~/.rustup/settings.toml`, SHA-256
   `46d451a6b8ea2bed375971c34fee742235e1cd143e34368c6f9211802126de45`,
   and exact verified repository `rust-toolchain.toml`, SHA-256
   `8aa4ad836dc3e6479a7677e869fd3ad6a9314e5624d76b5e7f666fe7820b30c8`,
   selecting the held no-follow directory
   `~/.rustup/toolchains/1.89.0-x86_64-unknown-linux-gnu`; within it, `bin/cargo` and `bin/rustc`
   have SHA-256
   `5b32bd53b8a08d8e206daed51f3fce40cdc0729038ff96bd0ca845596a2c1019` and
   `c6ac0142c05a60b0f6c15e1a118bd5ac3bd04924cd6dc0c328f501c5f86e4142`, with exact versions
   `cargo 1.89.0 (c24e10642 2025-06-23)\n` and
   `rustc 1.89.0 (29483883e 2025-08-04)\n`.

Each executable's version probe is exact argv `[canonical-argv-zero, "--version"]`, executes the
retained FD with `execve(fd, argv, env)`, uses empty stdin, exact environment
`LANG=C.UTF-8` and `LC_ALL=C.UTF-8`, captures stdout/stderr separately, and accepts no whitespace
normalization. Bubblewrap, Python, Git, pinned Cargo, and pinned rustc require the exact single
stdout line above and zero stderr bytes; rustup requires its exact two stderr lines above. Two
additional proxy probes execute the retained rustup FD with canonical argv zero equal to the
`cargo` and `rustc` shim path respectively; they require the exact Cargo/rustc stdout lines above
and zero stderr. This proves the ordinary shim selection without executing a mutable pathname.

The complete held toolchain root has exactly 26 directories, 213 regular files, zero symlinks,
and 1,071,859,799 regular-file bytes. Its canonical binary manifest is sorted by raw relative-path
bytes. Each record is `type-byte` (`D`, `F`, or `L`), 8-byte big-endian path length, raw path, and
4-byte big-endian permission mode; a regular-file record then has 8-byte big-endian size and raw
32-byte SHA-256, while a symlink record instead has 8-byte big-endian target length and raw target.
The SHA-256 of the concatenated records is exact
`62de669575b22b82124eacb7caf100fd1e1041e42bd66b2a148f2ff6d14540c0`.
Any unsupported type, source submount, count/byte/hash/mode drift, or pre/post manifest difference
is `executable_identity_invalid`/`executable_identity_drift`.

The immutable platform TCB—not Python self-authentication—covers the code that necessarily runs
before the first bootstrap instruction. At that first instruction V2 verifies flags
`isolated=1`, `no_site=1`, `no_user_site=1`, `ignore_environment=1`,
`dont_write_bytecode=1`, and `safe_path=true`; exact `sys.path`
`["/usr/lib/python313.zip", "/usr/lib/python3.13", "/usr/lib/python3.13/lib-dynload"]`; the exact
initial module and `/proc/self/maps` manifests; absent `/etc/ld.so.preload`; and the exact
forbidden-key-free E0 environment contract. Before compile or execution of a runner/test blob, V2
validates and retains
the authenticated blob's complete standard-library manifest and preloads it. Every origin is
interpreter-defined `built-in`/`frozen` or a no-follow-opened root-owned regular file beneath one
validated `sys.path` entry with safe ancestors, no foreign effective write ACL, and exact hash.
Only after every origin is loaded and checked does V2 set `sys.path=[]` and close imports to the
fixed snapshot. Symlink, user-owned origin, `.pth`, `sitecustomize`, `usercustomize`,
cwd/script-directory origin, user site, ambient loader/Python influence, late import, module-file
drift, or TCB/bootstrap/runner/test hash mismatch is
`python_startup_tcb_invalid`/`executable_identity_invalid`/`executable_identity_drift`. The runner
never claims that already-executed platform bytes self-authenticate, and neither runner nor test
worktree pathname is executed or imported. Stage B Bubblewrap, its worker-as-PID-1 command, and
Cargo inherit none of host's retained TCB/module-origin FDs.

Each real executable is no-follow opened, regular, executable, owner/ancestor/ACL safe,
identity- and SHA-validated, retained by FD through the wall, and rehashed/revalidated after all
descendants. The two shim names are separately no-follow `lstat`-validated as current-account
symlinks with the exact relative target `rustup`; their path identities are retained and
revalidated. Stage A byte-copies the complete held toolchain plus exact rustup settings into its
private rustup-home snapshot and copies the held rustup bytes, two exact symlinks, and verified
registry inputs into its immutable Cargo-home seed. It then makes one separate private writable
byte-copy of that seed as the Cargo-home runtime. Stage B read-only projects the rustup snapshot
and writable-projects only the Cargo runtime at the original canonical paths, so E0 resolves
`cargo`/`rustc` through copied rustup exactly as the ordinary invocation did while no host source
alias can mutate executed bytes. The seed must remain byte-identical. The runtime initially
contains none of the three Cargo metadata entries; runtime changes are limited to creation and
content/locking state of exact root regular files `.package-cache`,
`.package-cache-mutate`, and `.global-cache`; any other difference fails closed. Exact private
rustc `--print sysroot` must emit that canonical toolchain path plus LF, and, inside the verified
Stage B worker containment, a
bounded pre-Cargo probe must compile and execute a
minimal no-dependency Rust test in exact `R/tmp/toolchain-probe`, reap it, and descriptor-remove
that probe directory before Cargo so the fresh target remains empty. The byte-identical E0
`PATH` must resolve `cargo`/`rustc` to the projected Cargo-home symlinks, which must resolve to
the projected rustup bytes and select the projected pinned toolchain. Wrappers, aliases, extra
symlinks, or resolution elsewhere are ineligible. P1 does not add `CARGO`, `RUSTC`, `CARGO_HOME`,
`CARGO_TARGET_DIR`, `RUSTUP_HOME`, `PATH`, `HOME`, or another environment selector.

Held `/usr/bin/git` runs only with explicit `--git-dir=<repo>/.git --work-tree=<repo>`,
`--no-optional-locks`, `--no-replace-objects`, `GIT_CONFIG_NOSYSTEM=1`,
`GIT_CONFIG_GLOBAL=/dev/null`, `GIT_OPTIONAL_LOCKS=0`, `GIT_NO_REPLACE_OBJECTS=1`, fixed C locale,
fixed `/usr/bin:/bin` PATH, and no other `GIT_*` input. The runner rejects object alternates,
replace refs, grafts, a shallow repository, a non-directory/symlinked gitdir, merge state,
worktree config, `info/attributes`,
and configured fsmonitor/sparse-index/untracked-cache authority. It no-follow opens and retains
`.git/config` and `.git/info/exclude`; the latter is hashed evidence only and is never consulted
for cleanliness. Fixed `git config --local --null --list --no-includes` must find only exact
`core.repositoryformatversion=0`, `core.filemode=true`, `core.bare=false`,
`core.logallrefupdates=true`, plus keys matching only
`remote.<name>.{url,fetch}` or `branch.<name>.{remote,merge,vscode-merge-base}`. Includes,
`core.excludesFile`, `core.worktree`, any other `core.*`, `extensions.*`, status/diff/filter/hook
configuration, or any unrecognized key is rejected. `.git/config.worktree` and
`.git/info/attributes` must be absent before and after the wall.

Cleanliness does not use `git status` or mutable local/global exclude authority. The independently
verified expected commit/tree/blob chain is authority. Fixed `git ls-tree -rz` is accepted only
when its complete raw result equals the runner's strict tree parser; Git output alone is never
authority. The runner obtains exact stage-zero index
path/mode/blob entries and flags with fixed `git ls-files --stage -v -z`, and requires the two sets be
byte-for-byte equal after both are normalized only to `mode<NUL>blob-id<NUL>path<NUL>`, with no
unmerged, sparse, intent-to-add, skip-worktree, assume-unchanged,
submodule, or non-stage-zero entry. It also parses the retained raw index header/extensions and
rejects split-index `link`, sparse-directory `sdir`, untracked-cache `UNTR`, and fsmonitor `FSMN`
extensions. Independently it no-follow opens every tracked worktree path,
requires regular/symlink type and executable mode match the tree entry, hashes Git blob framing
plus exact file or symlink-target bytes in Python, and requires the resulting blob ID equal both
tree and index. It records a bytewise path/mode/blob/SHA-256 manifest plus the index and HEAD/ref
descriptor identities/hashes and repeats every check after all descendants.

Untracked classification uses only tracked, byte-verified `.gitignore` files named by that exact
HEAD tree. The runner rejects an untracked `.gitignore` unless its own containing subtree was
already excluded by a tracked ancestor `.gitignore` without consulting that file; such a nested
file can affect only an already-ignored subtree and is excluded from authority. It then invokes fixed
`git ls-files --others --exclude-per-directory=.gitignore -z` without `--exclude-standard`; any
output is dirty. It separately hashes the ignored-output list from
`git ls-files --others --ignored --exclude-per-directory=.gitignore -z` for evidence only.
Therefore `.git/info/exclude`, global excludes, and local config cannot hide an untracked path,
while build/agent artifacts ignored by reviewed tracked policy remain allowed. Pre/post config,
exclude, ignore-source, tree, index, tracked-worktree, untracked, and ignored-list identities and
hashes must match except that bytes below tracked-policy-ignored paths may change. The runner
derives the logical exact command from `--mode`, creates the root itself, and embeds the canonical
`1309/1264/45/0` plus name/signature hash expectations.

Cargo receives exact E0, byte-for-byte, with only `TMPDIR=R/tmp` and
`XDG_RUNTIME_DIR=R/xdg-runtime` inserted/replaced. This is exactly the canonical shell-prefix
semantics; P1 adds, removes, or rewrites no other environment key. It rejects loader/Python
injection keys before Stage A and never emits E0 values. The evidence records only the sorted key
names hash, entry count, the two public overrides, and booleans proving every other entry was
byte-identical and no protected value was serialized. Parallelism is selected only by omission of
`--test-threads`; serial mode is selected only by the exact command suffix.

Stage B binds a fresh writable private target directory at exact lexical
`<repository>/target`, so unmodified Cargo default target selection sees a clean target without a
`CARGO_TARGET_DIR` override. The private writable Cargo-home runtime is projected at the exact path
E0 would normally select. Its immutable seed contains only the verified rustup regular file, exact
`cargo`/`rustc` symlinks, and Cargo.lock-selected checksum-verified registry subset; it contains
no config, credential, Git cache, lock/cache metadata, wrapper, unrelated executable, or extra
package. Stage A proves the runtime was an exact seed copy before Stage B and, after containment,
permits creation/content change only for the three named root regular metadata files. Any other
new path, deletion, rename, symlink/type/mode/owner change, or mutation is
`snapshot_identity_invalid`. The private rustup home contains only exact settings and the
selected complete pinned toolchain. Repository target and host Cargo/rustup-home bytes are never
Cargo inputs. Cargo-home runtime, target, temp, control, and XDG directories are
descriptor-removed only after containment emptiness.
Before and after the wall, the runner no-follow proves `config` and `config.toml` absent from every
Cargo search location from the repository `.cargo` ancestors through the account `.cargo`, and
proves account credential files are not mounted into the fresh home. Appearance, replacement, or
read ambiguity is `repository_identity_mismatch`/`executable_identity_drift` and invalidates the
wall.

The runner prints only `wall_gate=VALUE invocation_id=INVOCATION_ID` to stdout; it never prints the
evidence parent, repository/root paths, command argv, environment, or raw log. Fixed error
classifications go to stderr. Exit codes are:

| Code | Meaning |
|---|---|
| `0` | provenance eligible and canonical result match (`wall_gate=clean`) |
| `64` | invalid public invocation |
| `65` | repository/CWD/HEAD/tree/cleanliness or executable identity mismatch |
| `66` | Linux namespace/current-ID-map/kernel prerequisite unavailable; no fallback |
| `67` | safe-parent/root/descriptor/ACL/mount setup rejection |
| `68` | Stage B/worker handshake, containment, subreaper, timeout, survivor, or forced-teardown rejection |
| `69` | provenance eligible but counts/names/signatures differ (`wall_gate=regression`) |
| `70` | descriptor identity or exact cleanup/absence rejection |
| `71` | bounded output/evidence capture or finalization failure |
| `72` | fail-closed internal invariant failure |

#### Descendant-containment contract

At `host_main` entry the authenticated host controller completes TCB/repository/input validation,
captures fixed monotonic `snapshot_deadline = start + 900 seconds`, opens the evidence/backing
authority, and launches retained Stage A. Stage A authenticates the sealed runner projection,
constructs the three immutable seeds plus writable Cargo runtime under that deadline, captures
`stage_b_setup_deadline = now + 60 seconds`, creates `R/control/worker.sock` as a
current-UID mode-`0600` `AF_UNIX SOCK_SEQPACKET|SOCK_CLOEXEC` listener, and launches exact nested
Bubblewrap Stage B as its direct child. Stage A retains a pidfd for that exact Bubblewrap process
and the read end of Bubblewrap's fixed `--json-status-fd`. Stage B uses new user, mount, and PID
namespaces, one-to-one current UID/GID mapping, `--die-with-parent`, private propagation, and
exact `--as-pid-1`. The authenticated Python worker is therefore the namespace PID 1 and the
`child-pid` reported to Stage A, rather than a child hidden behind Bubblewrap's default reaper.

Before Stage B launch, Stage A also opens exact partial-evidence `cargo.log` by its retained
directory FD and creates one `pipe2(O_CLOEXEC)` stream. In the forked Bubblewrap child only, it
`dup2`s that single write endpoint onto both stdout and stderr, leaves original stdin byte-exact,
closes every other pipe endpoint, and executes retained Bubblewrap. Stage A closes its writer
immediately after successful spawn and continuously drains the sole read endpoint through
`collect_stage_b_output` into the already-open `cargo.log`; neither Stage B nor Cargo can see the
partial-evidence mount. Pinned Bubblewrap and the authenticated worker must emit zero
successful-path stdout/stderr of their own, so the single pipe is the exact combined Cargo byte stream in
kernel delivery order. Cargo inherits only original fd 0 and those exact fd 1/2 duplicates; it
inherits neither the read end, an evidence FD, nor a control FD.

The collector begins before Stage B can write, uses bounded reads/writes and continuous
backpressure, and retains its pipe/file identities through EOF, `fsync`, byte count, and SHA-256.
EOF is mandatory only after authenticated `ECHILD`, worker exit, and exact Stage-B Bubblewrap
reap; early EOF, a surviving writer, short write, collector failure, missing EOF, or descriptor
identity drift is ineligible. Up to 4 GiB the eligible `cargo.log` is byte-exact. On the first byte
beyond the cap, Stage A irreversibly marks `evidence_limit_exceeded`, continues draining to avoid
deadlock, preserves only the bounded diagnostic prefix, performs no result analysis, and can never
make that wall eligible. The fixed private control protocol remains numeric/status-only and never
carries log bytes or `SCM_RIGHTS`.

Bubblewrap's status stream must provide exactly two LF-terminated JSON objects with duplicate and
unknown keys rejected for pinned Bubblewrap 0.11.0: first exact keys `child-pid`,
`mnt-namespace`, and `pid-namespace`, naming the Stage B Python worker in the Stage-A PID namespace
and its namespace identities; then the sole key `exit-code` after completion. Stage A opens a
pidfd for the reported worker and verifies the namespace/mount identities. The Stage B worker
connects to fixed `/run/substrate-wall/control/worker.sock`; Stage A accepts exactly one peer and
requires `SO_PEERCRED` to identify that retained worker pidfd. The worker's first instructions set
and read back `PR_SET_CHILD_SUBREAPER`, verify its fixed projected control path and namespace
identity, and send `READY`. Stage A unlinks `R/control/worker.sock`, proves its name absent, and
sends the one-byte `START` record only after the Stage B pidfd/status record, worker pidfd,
credentials, namespace identities, mount manifest, listener absence, and setup deadline all
validate. Both accepted endpoints are close-on-exec; the worker keeps its own endpoint while its
Cargo child loses the inherited duplicate on exec. Cargo's stdin is the byte-identical original
stdin. Before `START`, the worker mounts nothing, forks nothing, and cannot start Cargo.

The source-closure detached-descendant probe used Bubblewrap's default namespace reaper and
returned the initial child status in 3 ms while a detached two-second child remained, proving that
default parent-only Bubblewrap wait can coincide with forced namespace teardown. Stage B therefore
uses `--as-pid-1`: the authenticated worker is both the JSON-reported child and namespace adoption
root, sets/reads back `PR_SET_CHILD_SUBREAPER`, and must normally reap to `ECHILD` before it exits.
The worker's PID-1 role does not make a Bubblewrap exit, JSON exit status, Cargo exit, process
group, `pgrep -P`, sleep, or one-time snapshot sufficient by itself.
Missing/spoofed/duplicate/out-of-order status/control records, early EOF, credential mismatch,
nonzero Stage B runner status, premature Bubblewrap/worker exit, or setup expiry makes the wall
ineligible before any cleanup result is considered.

After `START`, the worker captures
`wall_deadline = now + timeout_seconds`, starts exact Cargo as its child, waits for Cargo, and
then continues `waitid`/`waitpid` over every child until the kernel reports `ECHILD`.
Because the worker became a verified subreaper before Cargo, reparented, double-forked,
new-session, process-group-changing, or nested-PID-namespace descendants remain its waitable
children. A descendant that outlives Cargo is waited normally within the same wall deadline.
Only after `ECHILD` may the worker send one fixed authenticated `CONTAINMENT_EMPTY` record
containing bounded numeric Cargo status/reap counts and exit zero. Stage A requires that record
from the retained worker identity, then requires the worker and exact Stage B Bubblewrap process
to exit normally and the status FD to report exit code zero. This authenticated
subreaper-to-`ECHILD` chain is the sole descendant-completion authority.

On wall expiry or a surviving descendant, the worker first irreversibly records
`eligible=false` and bounded numeric diagnostics. It enumerates only processes visible inside
its private `/proc`, verifies the exact PID-namespace inode, excludes only itself, opens each
remaining target with `pidfd_open`, sends `SIGTERM`, and reaps for at most five
seconds; verified survivors receive `SIGKILL` through the same pidfds and are reaped for at
most five more seconds. Any signal, timeout, survivor, or Bubblewrap-forced teardown remains
ineligible. No numeric-PID-only signal, host-namespace enumeration, or unrelated target is
permitted.

On Stage B setup expiry or premature worker/Bubblewrap loss, Stage A marks the wall ineligible,
signals only the retained Stage B Bubblewrap pidfd with TERM, waits two seconds, then KILL if
needed, and reaps that exact direct child. Exact `--die-with-parent` supplies bounded namespace
teardown when Stage A disappears; it is never a success signal. Stage A never enumerates or
signals unrelated host processes. Stage B must be fully reaped before in-namespace root deletion
begins.

When normal emptiness or invalid-wall teardown finishes, Stage A captures
`cleanup_deadline = now + 120 seconds` covering log analysis, post-identity/snapshot manifests,
descriptor cleanup, and staged evidence hashing. It writes an authenticated bounded Stage-A
record into the host-created partial evidence directory and exits; it never claims that its own
enclosing mount namespace is gone. `host_main` then requires exact Stage-A Bubblewrap status,
pidfd/process disappearance, Stage-A mount-namespace disappearance, retained backing identity,
empty underlying mountpoint, and descriptor-relative backing removal before
`finalize_after_stage_a_exit` may atomically publish `eligible=true`. Any stuck Stage A or Stage B,
residual namespace/mount, deadline, forced termination, or missing authenticated emptiness record
remains ineligible even if diagnostic cleanup later succeeds.

Before Cargo, the runner must:

1. validate a current-UID-owned `0700` backing/evidence parent outside `/tmp`, `/var/tmp`,
   and every shared sticky or world-writable ancestor;
2. establish exact nested unprivileged Bubblewrap user/mount/PID containment and map only the
   invoking current UID/GID one-to-one;
3. create one fresh private root distinct from the real product `SUBSTRATE_HOME` and every
   other wall;
4. authenticate the Stage B worker from Bubblewrap status plus accepted fixed-socket
   `SO_PEERCRED`, set/read back
   subreaping, and complete the START handshake;
5. place Cargo, every test binary, and every descendant below that worker in the PID namespace.

Parent-only `wait`, `pgrep -P`, sleeps, one-time process snapshots, process-group membership,
or Bubblewrap's cleanup reaper cannot establish containment. A timeout or surviving descendant
makes the wall ineligible; diagnostic teardown never repairs it. The runner must never kill an
unrelated process.

#### Exact namespace and mount lifecycle

The authenticated host controller and two source-closed Bubblewrap stages perform this exact
sequence:

1. `host_main` opens and retains the safe evidence/backing parent, creates one empty exact
   backing mountpoint, validates sources/TCB, seals the authenticated runner blob, then launches
   Stage A as its exact direct Bubblewrap child. It retains Stage-A pidfd/status, namespace IDs,
   backing-parent/mountpoint descriptors, and the evidence partial-directory descriptor.
2. Stage A starts before any snapshot or Cargo process with fixed `--die-with-parent`, a new user,
   mount, and PID namespace, recursively private propagation, fixed root-owned OS-TCB projections,
   read-only no-follow source projections, the runner supplied by `--ro-bind-data`, writable bind
   of only the host-created partial evidence directory at fixed
   `/run/substrate-wall-evidence`, the exact empty backing path mounted at a fixed location, and
   one private `tmpfs` `M1` over that location. The evidence parent/siblings remain hidden. Exact
   UID/GID mappings bind only the invoking IDs. Any
   option/order/map/mount/blob drift rejects.
3. Authenticated `stage_a_main` records `M1`, creates current-UID `0700` `R`, `R/tmp`,
   `R/xdg-runtime`, `R/control`, writable private `target`, repository/rustup-home/Cargo-home-seed
   trees, and a separate writable `cargo-home-runtime`. Every entry is descriptor-relative and
   stays on `M1`.
4. From held no-follow inputs it constructs and revalidates the three authenticated immutable
   seeds defined above. Repository bytes/modes must match the independently verified expected
   tree; rustup settings/toolchain bytes must match their pinned complete manifests; the
   Cargo-home seed must contain exact rustup/shim/registry bytes and no Cargo lock/cache metadata
   file. The runtime is an exact byte-copy of that seed before Stage B.
   Unknown executable/package/config/credential/Git-cache content rejects.
5. Stage A creates the fixed private listener and launches Stage B with the exact validated
   runtime instantiation of `BWRAP_STAGE_B_ARGV_TEMPLATE_V1`: new user/mount/PID namespace,
   `--die-with-parent`, private
   propagation, a new `/proc`, read-only binds of repository/rustup-home snapshots, writable bind
   of the private Cargo runtime at canonical Cargo home, writable binds only for target, `R/tmp`,
   `R/xdg-runtime`, and the fixed control directory, plus the exact projected Stage-B repository
   CWD (`/home/spenser/__Active_code/substrate`).
   Stage-B runner bytes arrive through a separate sealed-memfd/`--ro-bind-data` projection.
   Stage A clears close-on-exec only for that exact data FD and the exact
   `--json-status-fd` writer in the forked Bubblewrap child. Bubblewrap consumes and closes the
   runner-data FD before starting the worker; its monitor alone retains the status writer through
   the final `exit-code` record and then closes it. The worker inherits neither numeric descriptor
   and proves that absence.
   Exact `--as-pid-1` makes the Python worker both namespace PID 1 and Bubblewrap's reported
   application; it establishes the verified subreaper/fixed-socket handshake before Cargo.
   Stage-B fd 1 and
   fd 2 are the same Stage-A-owned capture-pipe writer; fd 0 is original stdin; no evidence path
   or other capture descriptor is projected.
6. Cargo executes only after Stage B proves each projected target is on `M1`, has the expected
   seed/runtime manifest and mount flags, exposes no host source inode, the control pathname is
   absent, the worker peer matches the retained Bubblewrap child, stdin is unchanged, and Cargo
   will inherit no control FD. A concurrent host source mutation can affect neither copied bytes
   nor a Stage-B mount.
7. After Cargo and every descendant exit, the Stage B worker proves `ECHILD`, revalidates its
   snapshot/runtime/target/root views, sends authenticated completion, and exits zero. Exact
   Bubblewrap then tears down its private `/proc` and binds without lazy/forced unmount; Stage A
   requires its zero status and independently verifies that every Stage B mount is gone.
8. Stage A first requires capture-pipe EOF after Stage-B reap, fsyncs and hashes exact bounded
   `cargo.log`, then validates that the Cargo seed is unchanged and the runtime changed only in
   the three authorized metadata files. It preserves all remaining evidence outside `R`, removes
   target/tmp/XDG/control, runtime, and every private seed tree descriptor-relatively, retaining
   each in-`M1` parent/entry FD through that exact name-absence proof. It closes each removed
   entry FD, removes `R`, proves `R` absent, closes remaining in-`M1` FDs, writes the authenticated
   Stage-A completion record, and exits. It does not remove its own enclosing host backing path
   or claim its enclosing namespace unmounted.
9. Stage-A Bubblewrap tears down `M1` by normal namespace exit. Only authenticated `host_main`,
   still holding the separate underlying backing-parent/mountpoint authority, proves the exact
   Stage-A process and mount namespace gone, revalidates the now-empty underlying mountpoint,
   removes it descriptor-relatively, proves name absence, and closes that authority. It also
   requires host repository/rustup/toolchain/Cargo-home/cache pre/post manifests unchanged, then
   and only then finalizes the evidence record.

Lazy/forced unmount, busy/residual mount, changed mount ID/source/type/flags/propagation, unknown
snapshot content, a new Cargo config/child, or any Stage-A/Stage-B protocol drift is ineligible.
The runner touches no evidence-parent entry except its exact partial/final evidence directory; the
self-test-owned unrelated sibling sentinel is outside runner-managed paths.

#### Root, descriptor, and deletion contract

For the evidence parent, disposable backing mountpoint/private mounted parent, root `R`, `R/tmp`,
`R/xdg-runtime`, `R/control`, private target, three immutable seed trees, and writable Cargo
runtime, the runner shall:

1. open no-follow descriptors before use and record owner UID, exact `0700` mode, directory type,
   device/inode, `statx` mount ID, ancestor safety, and effective ACL state;
2. keep each authoritative parent/root/child descriptor open through setup, Cargo, all
   descendants, log preservation, final validation, that entry's unlink, and that entry's
   pathname-absence proof;
3. reject replacement, symlink substitution, owner/mode/ACL drift, descriptor/path identity
   mismatch, unexpected mount, unexpected entry type, shared/reused roots, and sticky or
   world-writable ancestry;
4. after containment emptiness, preserve output outside `R`, revalidate each descriptor and
   pathname, and recursively remove entries with no-follow descriptor-relative
   `openat`/`fstatat`/`statx`/`unlinkat` operations;
5. retain each authoritative parent/entry FD until the exact name is removed and absence is
   proved, then close that removed in-mount FD before Stage A exits; retain the separate
   host-owned underlying backing-parent/mountpoint authority in `host_main` across Stage-A exit
   and `M1` teardown, remove only that exact empty mountpoint after process/namespace/mount
   disappearance is proved, prove its pathname absent, and only then close the underlying
   authority;
6. reject any attempt to use a glob, ambient path, recursive parent removal, real product home, or
   pathname-only `shutil.rmtree`; the external self-test creates and verifies the unrelated
   sentinel before and after the run.

ACL inspection uses the standard-library xattr interface and parses the Linux
`system.posix_acl_access` and `system.posix_acl_default` values under the effective mask. A missing
ACL is accepted only when the mode check is exact; a malformed/unsupported ACL result, named user
or group with effective write, group-class effective write, other-class write, or ACL drift fails
closed.

Deletion before containment emptiness or closure of all authority before absence proof is an
ineligible provenance result.

#### Evidence layout, bounds, and atomic finalization

`host_main` creates a random 128-bit lowercase-hex `INVOCATION_ID` and a current-UID `0700`
directory `<evidence-parent>/.<LABEL>-<INVOCATION_ID>.partial` before Stage A. Stage A may write
only its fixed staged artifact basenames there; `host_main` retains the directory authority,
revalidates every staged file after Stage-A exit, adds the host teardown/finalization fields, and
alone performs the final rename. Every file is `0600`. The maximal result-complete final layout,
in its fixed artifact order, is:

```text
<evidence-parent>/<LABEL>-<INVOCATION_ID>/
  cargo.log
  containment.jsonl
  failure-names.txt
  normalized-signatures.txt
  summary.json
  provenance.json
  manifest.sha256
```

Every finalized evidence directory contains `provenance.json` and `manifest.sha256`. Of the five
result artifacts, `containment.jsonl` is present iff at least one containment event was committed,
`cargo.log` is present iff Cargo started, `summary.json` is present iff `result` is nonnull, and
`failure-names.txt` plus `normalized-signatures.txt` are both present iff their derivation
succeeded. A result-complete record contains all five. An invalid-analysis record may omit only
the two derived text files. A pre-Cargo staged failure therefore has zero or one of the five
artifacts according to whether containment events began; it does not synthesize an empty Cargo
log or summary. No other basename is allowed in a final directory.

For an evidence-eligible wall, `cargo.log` is the exact single-pipe combination of Cargo
stdout/stderr bytes in kernel delivery order and is never echoed to the runner console. It is
capped at 4 GiB. Overflow makes the wall ineligible, retains only the bounded diagnostic prefix,
drains the remaining pipe without analysis, and records no exact-log claim; `containment.jsonl`
is capped at 65,536 fixed-schema
events, 4,096 survivor PID records, 512 bytes per line, and 32 MiB total;
`failure-names.txt` is capped at 4,096 records and 16 MiB;
`normalized-signatures.txt` is capped at 4,096 records and 256 MiB; every derived line is capped at
1 MiB; each JSON document is capped at 8 MiB; and `manifest.sha256` is capped at 1 MiB. Any overflow is
`evidence_limit_exceeded`, makes provenance ineligible, and is never silently truncated as valid.
The runner receives no carrier/credential input and adds no ambient environment to evidence.
Product-emitted raw Cargo output remains restricted evidence; its directory/file modes and hashes
are retained. `summary.json` is retained in that restricted evidence directory; the console emits
only the fixed wall-gate/invocation-ID line defined above.

All files are first written in the partial directory, flushed, and `fsync`ed. Evidence needed from
`R` is copied before cleanup, but `provenance.json` cannot contain `eligible=true` until the
authenticated worker has reported `ECHILD`, Stage B has exited and unmounted cleanly, every root
descriptor/path revalidation passes, the descriptor-relative root removal succeeds, pathname
absence is proved, the in-`M1` descriptors close, Stage A exits cleanly, `host_main` proves the
Stage-A process and namespace gone, and the retained host backing authority removes and proves
absence of the exact underlying mountpoint. Only `host_main` may then write the final
`eligible=true` provenance and close the evidence/backing authority. `manifest.sha256` contains
lowercase SHA-256 hex, two spaces, the exact
basename, and LF for every other final file in bytewise basename order. After every file and the
partial directory are `fsync`ed, `renameat2(RENAME_NOREPLACE)` atomically gives the whole directory
its final basename and the evidence parent is `fsync`ed. An unavailable no-replace syscall,
preexisting final name, or replacement is `evidence_write_failed`; no overwrite-capable rename
fallback is allowed. A `.partial` directory is diagnostic debris only and never a provenance
record.

#### Exact provenance schema

`provenance.json` is UTF-8 JSON with sorted keys, no duplicate keys, LF termination, schema
`substrate.canonical_shell_wall_provenance.v1`, and exactly these keys. Invalid CLI (exit `64`) and
an unsafe/unwritable evidence parent (the corresponding exit `67`) produce no final evidence
directory. Evidence-finalization failure (exit `71`) may leave only `.partial` diagnostic debris.
Every other exit except provenance-eligible result mismatch/invalid analysis (`69`) must finalize
an ineligible staged record if the already validated evidence parent remains writable. Exit `69`
must instead finalize `eligible=true`, `wall_gate=regression`, and `record_stage=complete`.
“Nullable” means JSON `null` is permitted only as stated; no unknown key is permitted.

| JSON path | Type / nullability | Exact contract |
|---|---|---|
| `schema` | string, nonnull | Exact schema string above |
| `record_state` | string, nonnull | Exact `final` |
| `record_stage` | enum, nonnull | Last completed stage: `preflight`, `setup`, `containment`, `cleanup`, or `complete` |
| `invocation_id` | string, nonnull | 32 lowercase hex |
| `label` | string, nonnull | Validated label |
| `mode` | enum, nonnull | `parallel` or `serial` |
| `eligible` | boolean, nonnull | May be true only at `record_stage=complete` after all containment, post-identity, cleanup, unmount, and absence gates |
| `wall_gate` | enum, nonnull | `clean`, `regression`, or `ineligible` |
| `ineligibility_reasons` | array of enum, nonnull | Empty iff eligible; sorted unique values from the closed enum below |
| `runner_exit_code` | integer, nonnull | One exit code from the table above and precedence rule below |
| `started_at_utc`, `finalized_at_utc` | RFC 3339 UTC string; `finalized_at_utc` nullable until finalization | Invocation start and final-record times |
| `request` | object, nonnull | Exact keys `launcher`, `command` (string array), `environment_contract`, `tmpdir`, `xdg_runtime_dir`, `target_dir` (absolute strings or null until root setup), `snapshot_timeout_seconds` (exact `900`), `setup_timeout_seconds` (exact `60`), `wall_timeout_seconds` (requested integer). `launcher` has only `trust_model` (exact `immutable-root-platform-plus-reviewed-git-object`), `controller` (exact `python-v2-host-main`), `authority_commit_oid` (40 lowercase hex), `reviewed_authority_commit_oid` (40 lowercase hex that must exactly equal `authority_commit_oid`), `python_path`, `bootstrap_sha256`, `runner_blob_sha256`, `bubblewrap_path`, `bubblewrap_version`, `bubblewrap_sha256`, `bubblewrap_elf_closure_manifest_sha256`, `host_argv_template_sha256`, `stage_a_argv_template_sha256`, `stage_b_argv_template_sha256` (always 64 lowercase hex), `host_argv_sha256` (64 lowercase hex, nullable until the host argv has been recorded), `stage_a_argv_sha256`, `stage_b_argv_sha256` (64 lowercase hex, nullable only while `mounts.stage_a` / `mounts.stage_b` are null; once the corresponding stage record exists each value must be non-null and equal that stage record's `argv_sha256`), `authority_commit_matches_live_head`, `authority_commit_trailers_verified`, `reviewed_oid_matches`, `direct_array_spawn`, `sealed_stage_a_runner`, `sealed_stage_b_runner`, `nested_probe_clean` (booleans). `reviewed_oid_matches` must be exact `true`, and the validator recomputes the `authority_commit_oid == reviewed_authority_commit_oid` equality even while `repository` is null before preflight completes. `environment_contract` has only `inherited_entry_count` (integer), `inherited_key_names_sha256` (64 lowercase hex), `forbidden_startup_keys_absent`, `tmpdir_overridden`, `xdg_runtime_dir_overridden`, `all_other_entries_byte_equal`, `protected_values_not_recorded` (booleans) |
| `repository` | object or null | Null only before successful repository preflight; otherwise exact keys `branch`, `head`, `tree`, `cwd` (strings), `clean`, `alternates_absent`, `replace_refs_absent`, `grafts_absent`, `shallow_absent`, `local_config_allowlist_valid`, `worktree_config_absent`, `info_attributes_absent`, `tree_index_equal`, `ignore_sources_tracked`, `untracked_absent` (booleans), `gitdir_identity` (directory identity), `index_pre`, `index_post`, `head_file_pre`, `head_file_post`, `branch_ref_pre`, `branch_ref_post`, `local_config_pre`, `local_config_post`, `info_exclude_pre`, `info_exclude_post` (regular-file identities; each post value nullable only if failure prevents post-check), `tree_manifest_sha256`, `index_manifest_pre_sha256`, `index_manifest_post_sha256`, `ignore_sources_manifest_pre_sha256`, `ignore_sources_manifest_post_sha256`, `tracked_manifest_pre_sha256`, `tracked_manifest_post_sha256`, `untracked_paths_pre_sha256`, `untracked_paths_post_sha256`, `ignored_paths_pre_sha256`, `ignored_paths_post_sha256` (64 lowercase hex; post values nullable only if failure prevents post-check), `tracked_count` (integer) |
| `toolchain` | object or null | Null only before executable preflight; exact keys `platform`, `machine` (strings), `executables`, `python_runtime`, `rustup_resolution`, and `rust_toolchain_root` |
| `toolchain.executables` | array of eight objects | Ordered `bubblewrap`, `python`, `git`, `newuidmap`, `newgidmap`, `rustup`, `toolchain-cargo`, `toolchain-rustc`; exact keys `role`, `path`, `version`, `pre`, `post`. `pre`/`post` each use the executable identity schema below; `post` is null only if failure prevents post-check |
| `toolchain.python_runtime` | object | Exact keys `startup_tcb_trust_model` (exact `immutable-root-platform`), `startup_tcb_manifest_sha256`, `initial_modules_sha256`, `proc_maps_sha256` (64 lowercase hex), `ld_so_preload_absent`, `host_controller_verified`, `bubblewrap_launcher_verified`, `canonical_environment_preserved`, `git_executed_from_fd`, `git_object_chain_verified`, `late_imports_absent` (booleans), `flags` (map containing only the six exact flag/value pairs above), `sys_path` (the exact ordered three-string array above), `bootstrap_bytes` (positive integer fixed by the reviewed V2 artifact), `bootstrap_sha256` (that artifact's 64-lowercase-hex hash), `runner_blob_git_oid` (40 lowercase hex), `runner_blob_sha256`, `stdlib_manifest_sha256` (64 lowercase hex), `runner_file` (exact synthetic string), `runner_blob_matches_manifest` (boolean), `self_test_blob_git_oid`, `self_test_blob_sha256`, `self_test_file` (null for `host`; exact authenticated values for `self-test`), and `modules` (module-origin array sorted by module name) |
| `toolchain.rustup_resolution` | object | Exact keys `cargo_shim_pre`, `cargo_shim_post`, `rustc_shim_pre`, `rustc_shim_post` (symlink identities), `settings_pre`, `settings_post`, `repo_toolchain_file_pre`, `repo_toolchain_file_post` (regular-file identities), `rustup_version_stdout_sha256`, `rustup_version_stderr_sha256`, `cargo_proxy_version_stdout_sha256`, `rustc_proxy_version_stdout_sha256` (64 lowercase hex), `proxy_stderr_empty`, `canonical_selection` (booleans). Post identities are nullable only if failure prevents post-check |
| `toolchain.rust_toolchain_root` | object | Exact keys `source_path`, `source_pre`, `source_post` (directory identities; post nullable only if unavailable), `directory_count` (exact `26`), `file_count` (exact `213`), `symlink_count` (exact `0`), `regular_file_bytes` (exact `1071859799`), `manifest_pre_sha256`, `manifest_post_sha256` (exact pinned 64-lowercase-hex hash; post nullable only if unavailable) |
| module-origin entry | object | Exact keys `name` (string), `origin_kind` (`built-in`, `frozen`, or `file`), `path` (absolute string only for `file`, otherwise null), `pre`, `post` (regular-file identities only for `file`, otherwise null; `post` may be null only if failure prevents post-check) |
| executable identity | object, nonnull when present | Exact keys `dev`, `ino`, `uid`, `mode` (integers), `kind` (exact `regular`), `sha256` (64 lowercase hex), `path_matches_fd`, `owner_ancestor_acl_safe`, `executable` (booleans) |
| symlink identity | object, nonnull when present | Exact keys `dev`, `ino`, `uid`, `mode` (integers), `kind` (exact `symlink`), `target` (exact `rustup`), `path_matches_lstat`, `owner_ancestor_acl_safe` (booleans) |
| regular-file identity | object, nonnull when present | Exact keys `dev`, `ino`, `uid`, `mode` (integers), `kind` (exact `regular`), `sha256` (64 lowercase hex), `path_matches_fd`, `owner_ancestor_acl_safe` (booleans) |
| `namespace` | object, nonnull | Always present; exact keys `host_uid`, `wall_uid`, `host_pid`, `stage_a_bwrap_pid`, `stage_b_bwrap_pid`, `worker_stage_a_pid`, `host_start_time_ticks`, `stage_a_bwrap_start_time_ticks`, `stage_b_bwrap_start_time_ticks`, `worker_start_time_ticks`, `stage_a_pid_namespace_inode`, `stage_a_mount_namespace_inode`, `stage_b_pid_namespace_inode`, `stage_b_mount_namespace_inode` (integers or null until that stage), `uid_map`, `gid_map` (exact strings or null until captured), `stage_a_bwrap_pidfd_opened`, `stage_a_status_received`, `stage_b_bwrap_pidfd_opened`, `stage_b_status_received`, `worker_pidfd_opened`, `worker_peer_credentials_verified`, `worker_ready_received`, `control_path_absence_proved`, `worker_start_sent`, `worker_subreaper_verified`, `worker_echild_received`, `stage_b_exit_received`, `stage_a_exit_received`, `stage_a_namespace_absent` (booleans) |
| `mounts` | object, nonnull | Always present; exact keys `propagation_private` (boolean), `evidence_parent`, `backing_mountpoint`, `tmpfs`, `stage_a`, `stage_b`, `snapshots`, `cargo_home_runtime`, `proc` |
| `mounts.evidence_parent` | object | Exact `pre`, `post` directory identities; `post` is nullable only before cleanup |
| `mounts.backing_mountpoint` | object | Exact keys `path`, `underlying_pre`, `mounted`, `underlying_post` (directory identities; later values nullable by stage), `removed`, `name_absence_proved`, `descriptor_closed_after_absence` (booleans or null until cleanup) |
| `mounts.tmpfs`, `mounts.proc` | object or null | Null until that mount exists; otherwise exact keys `source`, `fstype`, `flags` (strings), `mount_id` (integer), `unmounted` (boolean or null until teardown) |
| `mounts.stage_a`, `mounts.stage_b` | object or null | Null until that stage mount exists; otherwise exact keys `argv_template_sha256`, `argv_sha256` (64 lowercase hex and required to match the corresponding `request.launcher.stage_{a,b}_argv_template_sha256` / `request.launcher.stage_{a,b}_argv_sha256` values), `user_namespace`, `mount_namespace`, `pid_namespace`, `json_status_fd`, `as_pid_1`, `builtin_pid1_fail_safe_only` (booleans; Stage A has `as_pid_1=false`/`builtin_pid1_fail_safe_only=true`, Stage B has `as_pid_1=true`/`builtin_pid1_fail_safe_only=false`; the other four are true for both), `uid_map`, `gid_map` (strings), `private_propagation`, `die_with_parent`, `completed`, `unmounted` (booleans) |
| `mounts.snapshots` | array of zero to three objects | Ordered prefix of `repository`, `rustup-home`, `cargo-home-seed`; exact keys `role`, `source_manifest_sha256`, `snapshot_manifest_sha256` (64 lowercase hex), `source_entry_count`, `snapshot_entry_count` (integers), `private_tmpfs`, `no_host_alias`, `stage_b_read_only`, `post_manifest_equal`, `removed` (booleans or null by stage). Repository and rustup-home have `stage_b_read_only=true`; Cargo-home seed is not mounted into Stage B and therefore has `stage_b_read_only=false`. Rustup-home hashes bind exact settings plus the pinned toolchain manifest; repository hash derives from the independently verified expected tree; Cargo-home seed binds exact rustup bytes, two shim symlinks, and Cargo.lock/checksum-selected registry subset while proving all three authorized runtime metadata paths absent |
| `mounts.cargo_home_runtime` | object or null | Null until copied; otherwise exact keys `seed_manifest_sha256`, `pre_manifest_sha256`, `post_manifest_sha256` (64 lowercase hex), `seed_entry_count`, `pre_entry_count`, `post_entry_count` (integers), `private_tmpfs`, `no_host_alias`, `stage_b_writable`, `seed_copy_equal`, `only_authorized_metadata_changed`, `removed` (booleans or null by stage), `authorized_mutable_paths` (exact ordered array `[".global-cache",".package-cache",".package-cache-mutate"]`), and `changed` (sorted unique subset of that ordered allowlist) |
| directory identity | object, nonnull when present | Exact keys `dev`, `ino`, `mount_id`, `uid`, `mode` (integers), `kind` (exact `directory`), `acl_sha256` (64 lowercase hex), `foreign_effective_write`, `path_matches_fd` (booleans) |
| `containment` | object, nonnull | Always present; exact keys `cargo_pid`, `cargo_exit`, `stage_b_exit`, `stage_a_exit` (integers or null if never started/not yet exited), `reaped_descendants`, `event_count`, `output_pipe_dev`, `output_pipe_ino`, `output_bytes_preserved` (integers or null until observed), `stage_b_setup_timed_out`, `wall_timed_out`, `premature_stage_b_exit`, `echld_observed`, `authenticated_empty_record`, `forced_teardown`, `event_overflow`, `output_pipe_opened_before_stage_b`, `single_pipe_for_stdout_stderr`, `original_stdin_preserved`, `cargo_control_fd_absent`, `output_eof_after_stage_b_reap`, `output_overflow`, `stage_a_namespace_absent` (booleans), `events_file` (string or null before creation) |
| `roots` | array of zero to nine objects | Present roles in fixed order `root`, `tmp`, `xdg-runtime`, `control`, `target`, `repository-snapshot`, `rustup-home-snapshot`, `cargo-home-seed`, `cargo-home-runtime`; each has exact keys `role`, `path`, `pre`, `post`, `removed`, `name_absence_proved`, `descriptor_closed_after_required_absence`; `post` and the three booleans are null until that phase |
| `roots[].pre`, `roots[].post` | directory identity or null | `pre` nonnull for every listed role. Child/snapshot post is immediately before its removal; root post is after all child/snapshot absence and immediately before root removal |
| `evidence` | array of zero to five objects | Present artifacts in fixed order `cargo.log`, `containment.jsonl`, `failure-names.txt`, `normalized-signatures.txt`, `summary.json`; exact keys `name`, `bytes`, `sha256`. Presence follows the stage rules above. A result-complete record has all five; invalid analysis has either both derived text files after successful derivation or neither after failed derivation, never only one. `manifest.sha256` binds every present artifact plus `provenance.json` |
| `result` | object or null | Null before Cargo-log analysis; otherwise exact keys `analysis_status`, `analysis_reasons`, `cargo_exit_code`, `final_status`, `discovered`, `passed`, `failed`, `ignored`, `failure_name_count`, `failure_names_sha256`, `signature_count`, `normalized_signatures_sha256`, `raw_log_sha256`, `canonical_match`, `transitions` |
| `result.analysis_status` | enum | `complete` or `invalid`; invalid forces `canonical_match=false`, `wall_gate=regression`, and exit `69` when provenance is otherwise eligible |
| `result.analysis_reasons` | sorted unique enum array | Values from `missing_final_result`, `invalid_result_grammar`, `cargo_exit_status_mismatch`, `failure_count_mismatch`, `duplicate_failure_name`, `missing_panic_block`, `duplicate_panic_block`, `invalid_panic_header`, `invalid_encoding`, `nul_byte` |
| `result.cargo_exit_code`, `result.final_status` | integer / nullable enum | Cargo exit is always the exact collected integer. Final status is `FAILED` or `ok` after a valid final-result selection and is null iff no valid final-result line was selected. `FAILED` is valid only with code `101`, and `ok` only with code `0`; any other nonnull combination is invalid analysis |
| `result` counts/hashes | integer/string or null | Nonnegative integers and 64-lowercase-hex hashes when analysis is complete; otherwise nullable exactly for the unavailable artifact. Whenever counts are nonnull, `discovered` is defined exactly as `passed + failed + ignored` |
| `result.canonical_match` | boolean | True only when analysis is complete, Cargo exit is exact `101`, final status is exact `FAILED`, counts are exact `1309/1264/45/0`, failure-name and signature counts are each `45`, and both artifact hashes are canonical |
| `result.transitions` | object | Exact keys `pass_to_fail`, `new_fail`, `fail_to_changed_failure`, `fail_to_pass`, `removed`, `renamed_or_substituted`, `newly_ignored`, `weakened`; every value is null because the runner accepts no historical comparison artifact. The renewed closeout computes these only against its separately authenticated baseline inventory |

The closed ineligibility-reason enum is:
`environment_unavailable`, `invocation_authority_invalid`, `repository_identity_mismatch`,
`repository_dirty`,
`python_startup_tcb_invalid`, `executable_identity_invalid`, `executable_identity_drift`,
`safe_parent_rejected`, `snapshot_identity_invalid`, `snapshot_timeout`,
`cargo_home_runtime_drift`, `root_validation_failed`, `namespace_setup_failed`,
`stage_a_liveness_failed`, `stage_b_setup_timeout`, `stage_b_liveness_failed`,
`premature_stage_b_exit`, `subreaper_unverified`, `control_channel_invalid`,
`containment_timeout`, `surviving_descendant`, `worker_handshake_failed`,
`output_capture_failed`, `diagnostic_scope_violation`, `evidence_limit_exceeded`,
`root_identity_drift`, `unexpected_mount`, `acl_drift`, `mount_teardown_failed`,
`cleanup_identity_mismatch`, `cleanup_failed`, `evidence_write_failed`, and
`internal_invariant_failed`. Result mismatch is not an ineligibility reason: a provenance-valid
mismatch or invalid result analysis produces `eligible=true`, `wall_gate=regression`, and exit
`69`.

If multiple failures occur, exit precedence is `72 > 71 > 70 > 68 > 67 > 66 > 65 > 69 > 0`;
all applicable ineligibility reasons remain in the sorted array. Invalid result analysis is
recorded only in `result.analysis_reasons` and maps to `69`; output capture/evidence bounds map to `71`;
cleanup/mount teardown map to `70`; setup/root/mount construction map to `67`; repository or
executable preflight maps to `65`.

`summary.json` is the exact `result` object above. `containment.jsonl` has one sorted-key UTF-8
object per LF-terminated line with exact keys `seq`, `monotonic_ns` (nonnegative integers),
`pid_namespace_inode` (nonnegative integer or null before PID-namespace creation), `kind` (one of
`stage_a_started`, `stage_a_status_received`, `output_pipe_opened`, `stage_b_started`,
`stage_b_status_received`,
`worker_pidfd_opened`, `worker_peer_credentials_verified`, `worker_ready_received`,
`control_path_absence_proved`, `worker_subreaper_verified`, `worker_start_sent`, `cargo_started`,
`cargo_exited`, `child_reaped`, `stage_b_setup_timeout`, `wall_timeout`, `sigterm_sent`,
`sigkill_sent`, `echild_observed`, `containment_empty_received`, `premature_stage_b_exit`,
`stage_b_exited`, `output_pipe_eof`, `stage_a_exited`, `stage_a_namespace_absent`, `teardown_started`,
`teardown_finished`), and `pid`, `ppid` (integer or null), `state` (one kernel state byte or
null). It contains no other string.

#### Exact log-summary compatibility contract

`summarize_test_log` reproduces the prior canonical analyzer inside the tracked runner; it may not
invoke or import the untracked analyzer. The self-test file embeds one fixed UTF-8 Cargo-log byte
fixture and the exact expected `failure-names.txt`, `normalized-signatures.txt`, `summary.json`,
and three SHA-256 byte strings as literals; changing any parser byte requires docs-first
reauthorization. The implementation follows this complete grammar:

1. input is LF-delimited strict UTF-8; a NUL, undecodable sequence, or CR-only record makes
   analysis invalid and exit `69` when provenance is otherwise eligible. A line/artifact bound
   overflow or failure to write the required derived LF is `evidence_limit_exceeded`/
   `evidence_write_failed`, provenance-ineligible, and exit `71`;
2. a result line must match from byte 0 through LF:
   `^test result: (FAILED|ok)\. (0|[1-9][0-9]{0,9}) passed; (0|[1-9][0-9]{0,9}) failed; (0|[1-9][0-9]{0,9}) ignored; (0|[1-9][0-9]{0,9}) measured; (0|[1-9][0-9]{0,9}) filtered out;.*\n$`;
   every integer must be `<= 2147483647`, `ok` requires failed count zero, and `FAILED` requires a
   positive failed count. Exactly as the prior `tail -n 1` analyzer, the last matching result line
   is selected and earlier matching result lines are ignored. The selected status must agree with
   the collected Cargo exit: `FAILED` requires exact exit `101` and `ok` requires exact exit `0`;
   disagreement is `cargo_exit_status_mismatch`, invalid analysis, and exit `69` when provenance
   is otherwise eligible. `discovered` is computed only as the checked integer sum
   `passed + failed + ignored`; measured and filtered-out counts never contribute;
3. an exact `failures:\n` resets the candidate-name list and makes it active; until another exact
   reset or the selected result, any line matching `^    .*::.*\n$` contributes the bytes after
   the first four spaces. The active list immediately preceding the selected result is used,
   must contain exactly the selected failed count, and must contain no empty or duplicate name;
   an `ok` result produces exact zero-byte names/signatures files without requiring a failure
   section;
4. bytewise-sort the unique names and write each exact UTF-8 name plus LF, with no header or blank
   trailer. On a canonical log this is byte-identical to the prior analyzer;
5. a panic header must match the entire line
   `^thread '([^'\n]+)' panicked at ([^\n]+):\n$`. For every wanted name there must be exactly one
   header; a missing/duplicate wanted header or malformed would-be wanted header is invalid.
   Every valid header first flushes the active capture, then starts capture only for a wanted,
   unseen name; unwanted headers are otherwise ignored;
6. capture nonblank body lines after a wanted header until a line starts exactly `test `,
   `[codex-`, `running `, `test result:`, or `error:`, or equals `FAILED\n` or `failures:\n`.
   A line beginning exactly `note: run with` is discarded. EOF flushes the active block;
7. remove each input LF, join captured body lines with one ASCII space, replace
   `aos_[0-9a-f]+` with `aos_<id>`, replace `.tmp[A-Za-z0-9]+` with `.tmp<id>`, collapse only
   ASCII `[ \t\r\n\f\v]+` to one space, then remove at most the one collapsed leading/trailing
   space;
8. emit exact UTF-8 `NAME<TAB>WHERE<TAB>NORMALIZED_BODY<LF>`, bytewise-sort the complete unique
   records, require signature count equal failure-name count, and write no header/blank trailer;
9. compute SHA-256 over exact raw bytes of `cargo.log`, `failure-names.txt`, and
   `normalized-signatures.txt`, encoded as lowercase hex.

The embedded golden byte literals are exactly:

```python
GOLDEN_LOG = b"running 2 tests\ntest alpha::case ... FAILED\ntest zeta::case ... FAILED\n\nfailures:\n\n---- alpha::case stdout ----\nthread 'alpha::case' panicked at crates/shell/src/a.rs:10:2:\nleft  aos_deadbeef\n right: .tmpABC\nnote: run with `RUST_BACKTRACE=1` environment variable to display a backtrace\nFAILED\n---- zeta::case stdout ----\nthread 'zeta::case' panicked at crates/shell/src/z.rs:20:4:\n spaced    body\nsecond\tline\nfailures:\n    zeta::case\n    alpha::case\n\ntest result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 1307 filtered out; finished in 0.01s\n"
GOLDEN_NAMES = b"alpha::case\nzeta::case\n"
GOLDEN_SIGNATURES = b"alpha::case\tcrates/shell/src/a.rs:10:2\tleft aos_<id> right: .tmp<id>\nzeta::case\tcrates/shell/src/z.rs:20:4\tspaced body second line\n"
```

Their byte lengths/SHA-256 values are respectively `552` /
`80e7c12e6f3711bc85d8b6871e7d8e68442634f6883b04257ae14d9618256b57`, `23` /
`375dafeeb68560e1316881e277d0242b20d3bfa50585a8ae4279de03f748e91f`, and `131` /
`df36b0aca2ed2d7e5d7efdc37bb4577960dcf643d77d120fa0365d6b8779f634`.
The exact sorted-key compact `summary.json` is:

```json
{"analysis_reasons":[],"analysis_status":"complete","canonical_match":false,"cargo_exit_code":101,"discovered":2,"failed":2,"failure_name_count":2,"failure_names_sha256":"375dafeeb68560e1316881e277d0242b20d3bfa50585a8ae4279de03f748e91f","final_status":"FAILED","ignored":0,"normalized_signatures_sha256":"df36b0aca2ed2d7e5d7efdc37bb4577960dcf643d77d120fa0365d6b8779f634","passed":0,"raw_log_sha256":"80e7c12e6f3711bc85d8b6871e7d8e68442634f6883b04257ae14d9618256b57","signature_count":2,"transitions":{"fail_to_changed_failure":null,"fail_to_pass":null,"new_fail":null,"newly_ignored":null,"pass_to_fail":null,"removed":null,"renamed_or_substituted":null,"weakened":null}}
```

It is `672` bytes including its LF and has SHA-256
`b63d9a162a47026329f8b018398418632b60e05366e74f080da46673b7cd461f`.

The canonical expectations embedded in P1 are failure-name count `45`, SHA-256
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`, signature count `45`,
and SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.

The only canonical wall commands are:

```bash
TMPDIR="$R/tmp" XDG_RUNTIME_DIR="$R/xdg-runtime" \
  cargo test -p shell --lib -- --nocapture
```

and:

```bash
TMPDIR="$R/tmp" XDG_RUNTIME_DIR="$R/xdg-runtime" \
  cargo test -p shell --lib -- --nocapture --test-threads=1
```

#### P1 self-test gate

`CanonicalShellWallRunnerTests` shall contain:

- `test_rejects_cargo_or_bwrap_exit_with_live_descendant`
- `test_reaps_reparented_descendant`
- `test_process_group_escape_remains_contained`
- `test_timeout_is_ineligible`
- `test_snapshot_timeout_is_ineligible`
- `test_copy_tree_no_follow_preserves_modes_despite_umask`
- `test_rejects_root_path_replacement`
- `test_rejects_root_symlink_substitution`
- `test_rejects_child_entry_replacement`
- `test_rejects_owner_drift`
- `test_rejects_mode_drift`
- `test_rejects_acl_drift`
- `test_rejects_descriptor_path_inode_mismatch`
- `test_rejects_premature_descriptor_closure`
- `test_rejects_deletion_before_containment_empty`
- `test_preserves_unrelated_sentinel`
- `test_rejects_shared_or_reused_root`
- `test_rejects_sticky_world_writable_ancestor`
- `test_rejects_unexpected_mount_id`
- `test_rejects_unexpected_entry_type`
- `test_rejects_subreaper_readback_failure`
- `test_rejects_bwrap_status_or_worker_ready_handshake_failure`
- `test_rejects_namespace_prerequisite_fallback`
- `test_teardown_never_targets_unrelated_process`
- `test_stage_b_setup_timeout_signals_only_retained_bwrap_pidfd`
- `test_stage_b_timeout_before_status_starts_no_cargo_and_reaps_exact_bwrap`
- `test_stage_b_bwrap_death_before_worker_ready_is_ineligible`
- `test_stage_a_maps_vanished_post_status_worker_to_premature_exit`
- `test_stage_a_parent_loss_before_stage_b_start_starts_no_wall`
- `test_stage_b_bwrap_loss_after_ready_before_start_is_ineligible`
- `test_stage_a_start_requires_bwrap_status_worker_pidfd_peer_credentials_and_ready`
- `test_stage_a_loss_before_stage_b_launch_starts_no_wall`
- `test_stage_a_loss_before_worker_ready_terminates_bwrap_and_worker`
- `test_stage_a_loss_after_ready_before_start_terminates_bwrap_and_worker`
- `test_stage_a_loss_after_start_invalidates_and_terminates_wall`
- `test_premature_stage_b_exit_after_start_is_ineligible`
- `test_stage_b_complete_requires_authenticated_echild_record_and_zero_exit`
- `test_fixed_private_socket_peer_status_cloexec_unchanged_stdin_and_no_cargo_control_fd`
- `test_sigterm_ignoring_descendant_requires_ineligible_sigkill`
- `test_stuck_stage_b_hits_bounded_stage_a_deadline`
- `test_rejects_forbidden_glob_ambient_parent_and_product_home_targets`
- `test_rejects_pathname_only_cleanup_api`
- `test_diagnostics_omit_comm_cmdline_environment_and_markers`
- `test_cli_rejects_arbitrary_command_root_and_environment_overrides`
- `test_public_request_cannot_select_internal_host_stage_worker_roles_or_control_path`
- `test_cli_rejects_invalid_label_timeout_cwd_and_head`
- `test_rejects_dirty_index_worktree_and_nonignored_untracked_state`
- `test_ignores_only_git_ignored_build_outputs`
- `test_rejects_git_environment_config_alternates_replace_and_fsmonitor`
- `test_descendant_authority_commit_must_be_ancestor_with_exact_runner_and_test_blobs`
- `test_descendant_authority_rejects_non_ancestor_and_authenticated_blob_drift`
- `test_rejects_local_git_config_include_mode_exclude_worktree_and_unknown_keys`
- `test_info_exclude_is_held_but_never_cleanliness_authority`
- `test_rejects_info_attributes_worktree_config_and_index_authority_extensions`
- `test_rejects_tree_index_stage_mode_flag_and_worktree_byte_drift`
- `test_tracked_gitignore_is_only_untracked_exclusion_authority`
- `test_rejects_untracked_gitignore_and_every_nonignored_untracked_path`
- `test_rejects_path_shim_and_executable_identity_drift`
- `test_executes_and_revalidates_held_bwrap_python_git_rustup_cargo_and_rustc`
- `test_trusted_bwrap_launcher_rejects_ambient_loader_and_startup_tcb_drift`
- `test_stage_b_as_pid1_matches_json_child_pid_peer_credentials_and_echild`
- `test_held_git_fd_path_replacement_never_executes_replacement`
- `test_isolated_python_startup_rejects_environment_site_path_and_module_poison`
- `test_python_runtime_rejects_origin_and_late_import_drift`
- `test_host_controller_bootstrap_commit_templates_and_slots_avoid_self_reference`
- `test_bootstrap_rejects_path_execution_cmdline_drift_and_runner_blob_mismatch`
- `test_self_test_loader_authenticates_exact_runner_and_test_blobs`
- `test_self_test_loader_rejects_path_import_discovery_and_blob_mismatch`
- `test_private_rustup_toolchain_snapshot_matches_pinned_complete_manifest`
- `test_mounted_rustc_reports_private_sysroot_and_compiles_minimal_test`
- `test_vendor_snapshot_reconstructs_only_lock_checksum_selected_archives`
- `test_constructed_cargo_home_resolves_locked_vendor_offline_without_drift`
- `test_private_snapshots_ignore_transient_source_modify_replace_and_restore`
- `test_cargo_environment_preserves_e0_with_only_tmpdir_xdg_overrides`
- `test_environment_values_and_protected_markers_are_never_serialized`
- `test_fresh_private_target_never_reads_repository_target`
- `test_private_target_is_descriptor_removed_after_containment`
- `test_cargo_home_seed_runtime_confines_lock_metadata_and_is_removed`
- `test_rejects_project_ancestor_or_account_cargo_config_appearance`
- `test_evidence_bounds_fail_closed`
- `test_schema_rejects_unknown_missing_and_wrong_type_fields`
- `test_schema_encodes_preflight_setup_containment_cleanup_failures`
- `test_atomic_finalization_hides_partial_eligibility`
- `test_final_evidence_rename_is_noreplace`
- `test_summarizer_golden_failure_names_and_signatures`
- `test_summarizer_normalization_sort_and_newline_bytes`
- `test_summarizer_last_result_and_failure_section_grammar`
- `test_summarizer_accepts_valid_all_pass_as_eligible_regression`
- `test_summarizer_rejects_cargo_exit_status_mismatch`
- `test_summarizer_rejects_integer_duplicate_missing_and_panic_grammar`
- `test_summarizer_rejects_invalid_encoding_nul_and_overflow`
- `test_mount_source_flags_ids_propagation_and_order_are_exact`
- `test_projected_root_ids_include_mapped_stage_identity`
- `test_cache_and_proc_mounts_unmount_before_tmpfs`
- `test_backing_mountpoint_removed_and_evidence_parent_preserved`
- `test_parallel_command_and_fresh_root`
- `test_serial_command_and_fresh_root`
- `test_stage_a_output_pipe_preserves_combined_order_eof_backpressure_bounds_and_hash`
- `test_success_removes_root_under_continuous_descriptor_authority`
- `test_success_preserves_unrelated_files`
- `test_host_finalizes_eligible_only_after_stage_a_exit_namespace_and_backing_absence`

The fixed-socket and output-pipe tests use real pinned Bubblewrap mechanics with bounded dummy
children. They prove one pipe write description is duplicated only to Stage-B fd 1/2, original
stdin survives, the worker control endpoint is close-on-exec and absent in Cargo, alternating and
larger-than-`PIPE_BUF` stdout/stderr bytes reproduce the exact captured stream/hash, a deliberately
retained writer prevents EOF and eligibility, collector backpressure does not deadlock, and the
first byte over the cap irreversibly selects bounded ineligible diagnostic handling.

Mechanics tests use bounded dummy subprocesses, not the product wall. After every self-test and
fresh P1 security/provenance review is CLEAN, a separate baseline phase must run three parallel
walls and one serial wall with four distinct roots. The old four walls are never inputs.

### RP5 publication and sequencing gate

This exact six-file docs packet may edit only these six control-pack Markdown files. RP3 is
complete. RP4 product proof on exact integration commit/tree
`8c46135c861a468dea316cf9fd7d6c6bb15bddac` /
`5358497a8baec6f36e15aaef58415a759e64977d` is accepted clean by human disposition; the raw
persistence review remains `REQUEST_CHANGES` for three cache-only process-audit findings preserved
as non-blocking debt outside RP4 product-proof scope. This exact six-file change is the bounded
RP5 closeout packet.

At that preserved RP5 checkpoint, the docs-on-top -> one ordinary fast-forward publication model
was unchanged. Source publication had not yet occurred in that run, and no local/upstream/remote
parity was claimed there. After the reviewed/committed RP5 docs, the exact next step was one
ordinary fast-forward source publication. That publication later completed, after which the R2-3
suffix and refreshed native evidence closed R2-3; R2-4 and R3 remain later. No historical result
is rewritten as passing.

## A1.1d-5R2-4 terminal gate disposition

The R2-4 gate closes from immutable bounded evidence plus the landed correction; it adds no
production/test allowlist and reruns no broad product or Cargo wall. The controlling record is
[`review-control/r2-4-closeout-evidence.md`](review-control/r2-4-closeout-evidence.md).

- source authority is the exact linear chain
  `9e7b4b48e92864be6970ad373c35cb8bf18593b0` ->
  `316ee5c6cf12c060388c9d9376e0a79537f2094a` ->
  `d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32` on the same target ref, without reset, rebase,
  merge, clean, or history rewrite;
- the corrected normal first/repeat proof has `SUBSTRATE_INSTALL_NO_PATH` unset, identical
  commitment `3c57e0459af3e9e09bef46773e832e57bd3a0e5d14cac5ea51d81395c7bc8cbd`,
  byte-stable profiles with one selected-A block, hostile-B parity, and public-wrapper child
  authentication of the same custom A;
- exact baseline restoration passed, including the single-object `0650` continuation baseline
  constrained by SHA-256, type/no-symlink, `root:substrate`, ACL, and no group/other-write gates;
- the earlier `0640` archive observation does not authorize a general mode relaxation or an
  upstream artifact guarantee;
- quick world-command and Codex CLI reachability close only the narrow R2 join, not authenticated
  Codex execution or direct-member architecture; and
- passive diagnostic failures remain in their existing lane while uninstall leftovers and all
  installer/uninstaller cleanup, rollback, managed-artifact manifest, and convergence actions
  remain R3-owned.

The predecessor's immutable receipt is legacy plain text, not V1 JSON. Its later exact propagation
and the parent's independent Git/hash verification are recorded transparently; no replacement
JSON protocol claim is made. R3 implementation is `PARKED_BY_USER` and no R3 implementation task
has been dispatched. The just-closed corridor head was the B1/B2.1 joint closeout, B3.1, C1, and
the bounded internal A1.2b packet are complete on the bound Tuesday, August 4, 2026 candidate,
the then-current historical gate was `AUTHORITY_REQUIRED:R3_RESUME`. Its protected R3 predecessor
sequence remains outside the active schedule under `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY`.
Authenticated Codex,
retained workers/tasks, authoritative-session repair/refresh,
orchestrator packet-3 lifecycle/routing, gateway adoption, direct-member architecture, A1.1d/A1,
and the whole runtime refactor remain open.

## A1.1d-5R3 lifecycle contracts and gates

These contracts are archived normative planning evidence for the former packets indexed in `03`.
They are superseded for active scheduling by
[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md). They
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

## R3 implementation status append

This append preserves historical landed/status facts only. Its protected MAC, evidence, recovery,
retirement/finalizer, E03, and Windows gates are not current parity prerequisites and do not
authorize successor work.

### `A1.1d-5R3-MANIFEST`

`A1.1d-5R3-MANIFEST` now has a landed implementation for the non-destructive portions of
`R3-MANIFEST-01`: canonical manifest parsing, canonical bytes/digests, manifest/index/head/shared
claim publication helpers, receipt and pairing/ticket verification helpers, hidden direct-
interactive control entrypoints, and the separate native-evidence artifact validator required by
later evidence tasks and closeout. Its provider channels remain preserving `provider_unavailable`
stubs until LINUX/MAC/WIN replace them, and the closed terminal status mapping above remains
unchanged. Successor authority remains `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX`.

### `A1.1d-5R3-LINUX`

`A1.1d-5R3-LINUX` now has a landed implementation for the Linux portions of the managed-system
lifecycle contract: the exact `substrate-lifecycle-linux` executor and attested sudo relay,
protected/disposable publisher prepared-transition and receipt publication paths, fixed
`substrate-lifecycle-publisher-v1` service/socket units, and the exact pre-state snapshot/restore
handoff used by `world-provision.sh`. The packet preserves the closed product wall for Unix
orchestrator bodies, native evidence, and other platforms, and leaves successor authority at
`EVIDENCE:R3-LINUX-IMP-01`.

### `A1.1d-5R3-LINUX-CLOSEOUT`

`A1.1d-5R3-LINUX-CLOSEOUT` now has a landed docs/evidence closeout for the published Linux
provider checkpoint. It materializes the validated external evidence artifact and the
`codex.top-level-evidence-receipt.v1` under `review-control`, revalidates the artifact via
`validate_r3_native_evidence.py` with exact evidence/source/gated-successor joins, revalidates the
receipt with the orchestration skill validator, and records the bounded closeout review set. The
packet changes no production or test bytes, performs no Linux repair or MAC dispatch, and for this
authoritative orchestration the terminal successor is `COMPLETE`.

### `A1.1d-5R3-MAC`

The MAC implementation request is closed over the selected prefix, install bootstrap carrier,
PlatformBootstrapMappingV1, and ExecutorBuildEvidenceV1. The implementation package models and
validates those joins with non-executing fixtures; native build, Keychain/XPC/Lima lifecycle
exercise, operator-TTY pairing, receipt collection, and restoration evidence are deferred without
substitution to `EVIDENCE:R3-MAC-IMP-01`.

## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Under `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d` amendment
`0003-fresh-mac-review-epoch.json`, this new nonce-bound epoch reconstructs the verified
19-path attempt-3 baseline solely to remediate the six mandatory P1/P2 findings. It binds the
actual XPC peer audit token before request decoding, repeats canonical carrier/mapping/role and
Stage-1 joins before any mapped mutation, removes the standalone retire operation, and refuses
pre-spawn SSH-UDS replacement by disabling SSH-side unlink. The added checks are non-native only:
no Lima, launchd, Keychain, code-signing, publisher installation, or evidence artifact is run or
created here. Fresh review is recorded only in the exact MAC review-control set; the sole
successor remains `EVIDENCE:R3-MAC-IMP-01`.

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN recovery decision (2026-08-07)

For MAC recovery, a signed guest pairing uses two distinct logical sessions bound to the same
accepted PM mapping/SSH transport identity and a single fresh pairing-session nonce: a typed data
session carries ticket/hello/transcript/anchor frames, while an independently controlling guest TTY
session displays scope and receives the human-entered fingerprint/challenge/literal. Transport
observations are evidence only; P-256 SPKI/anchor/ticket/record joins remain authority. Confirmation
must not pass through the data session, argv, environment, file, pipe, or automation. Absence,
non-TTY, mismatched PM/machine/source/artifact/session, expiry, replay, or ambiguous generation-CAS
preserves state and blocks. A raw `lima-action` is replaced by a fixed control-binary typed
`submit-mapped-lifecycle-v1` request whose carrier/mapping/role/Stage-1 joins are repeated by the
admitted XPC executor before effect. This is source-planning authority only; native proof remains
exclusively `EVIDENCE:R3-MAC-IMP-01`.

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current contract correction (2026-08-07)

For this planning packet only, the authoritative planning subject is the exact eight-file list in
`r3-mac-evidence-recovery/TASKS.md` (five current-status/control documents plus `SPEC.md`, `PLAN.md`,
and `TASKS.md`), not any earlier six-file planning-subject shorthand. Its hash is computed only by
`for p in <eight literal paths>; do shasum -a 256 -- "$p"; done | LC_ALL=C sort | shasum -a 256`;
review-control files are excluded and all discovery/closure entries bind the resulting value.

The recovery-era evidence command additionally requires
`--expected-product-project-id <dispatch-bound-project-id>` between source-ref and gated-successor;
a missing/mismatched argument invalidates evidence. The historical Linux validation keeps
`2ccb802f-301c-4af4-9bd5-51d22808f0a2` as its explicitly passed historical binding.

`LimaStageOneAuthorizationV1` is limited to the absent fixed `mac.lima.instance/Create` branch
before PM finalization. It never substitutes for `PublisherBootstrapAuthorizationV1`, never authorizes
forwarding/post-PM roles, and never makes a script bootstrap-capable. The attested host control alone
prints full fingerprint/challenge values on its retained terminal; a separate guest controlling TTY
requires manual values from that terminal and binds only a confirmation commitment to the protected
record. The recovery route supersedes older present-tense attempt-4-to-evidence wording: only a
future reviewed remote-equal R1–R6 receipt may precede a fresh evidence dispatch.


## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id <dispatch-bound-project-id> --expected-gated-successor <value>`

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

The R3 macOS signer store is exactly the legacy System Keychain opened by public
`SecKeychainOpen("/Library/Keychains/System.keychain")` and identity-checked with
`SecKeychainGetPath`. `kSecUseKeychain` is add destination only. Every read, update, and deletion
uses a one-element `kSecMatchSearchList`; every applicable item operation sets the public
noninteractive UI-fail policy. Private `kSecUseSystemKeychain`, a default or ambient Keychain,
user/file/environment/caller-selected stores, Data Protection Keychain, software-file fallback,
and unsigned fallback are contract violations.

The key is one permanent, sign-capable software P-256 private key with service label
`com.substrate.lifecycle.v1` and exact application tag `<scope-id>:signing-key`. Zero exact-tag
matches may create. Exactly one must validate its tag, label, private class, EC P-256 type and size,
permanence, signing capability, canonical public point/SPKI, and equality with the SPKI recorded in
the signed wrapper. More than one match, a wrong attribute, a missing key under an existing wrapper,
an orphan under a new scope, create/reopen drift, or SPKI mismatch stops without regeneration.
Product signing never exports private bytes, but the control pack explicitly accepts that
sufficiently privileged root may export this software key.

Retirement first requires the protected wrapper to be absent. It then reopens and revalidates the
one exact key, deletes only the validated item reference through documented `kSecMatchItemList`
within the exact one-keychain search list, and re-queries for final absence. Protected-wrapper CAS
acquires the scope's current-anchor durable lock before key/SPKI validation and holds it through
readback; retirement holds the same lock across wrapper absence, key validation, exact deletion,
and final absence. Any lookup, identity, delete, or final-absence uncertainty preserves remaining
state and stops. The fixed code-designated root LaunchDaemon, audit-token/peer admission, canonical P1363
low-S signatures, signed protected wrapper, monotonic CAS, receipt/retry, and preserving-first
joins are unchanged. Secure Enclave/Data Protection Keychain and a session-capable user
LaunchAgent signer are explicitly deferred hardening and are not evidence or successor authority.
Intel/T2 is not an R3 support target.

## AUX-R3-MAC-EVIDENCE-RETIREMENT-V2 planning contract amendment (2026-08-13; docs-only)

This section is the authoritative R3 planning constraint for prospective macOS evidence retirement and the one preserved precommit-less orphan. It supersedes earlier retirement wording only where that wording permits destructive work before external durability/successor acceptance, conflates a pre-removal receipt with post-removal parity, allows the publisher to strand its own retry authority, or allows prospective authority to reach the old orphan. It authorizes no source, test, configuration, build, install, launchd, Keychain experiment/query/mutation, recovery, cleanup, evidence, mirror refresh, or `MAC-CLOSEOUT` effect.

### Normative authority and chronology

Candidate D is conditionally sound only for a newly precommitted prospective V2 lane. Candidate C is the mandatory serial order: preserve and experimentally characterize the old state, separately authorize exact recovery, prove exact parity, then close the prospective contract before any later implementation authorization.

The prospective and recovery lanes are permanently disjoint in authority, schemas, owner/version values, routes, executables, journals, parsers, signature domains, idempotency keys, and target decoders. No conversion or fallback is permitted in either direction. The old attempt's null/precommit-less state can never be upgraded into V2 authority.

The prospective V2 lane obeys all of the following:

1. **External-durability invariant:** no protected state, evidence record, registration, Keychain record, signer, file, component parent, or retry/handoff state is deleted until the still-authorized publisher has signed the exact canonical pre-removal receipt; the harness has durably persisted, reopened no-follow, byte-compared, and hashed that receipt; the harness has signed and independently made its acknowledgement durable; and surviving protected CAS has verified and bound both hashes.
2. **Successor-acceptance invariant:** a current owner cannot delete any state required for its own signing, retry, journaling, or successor invocation until the next owner independently verifies and durably accepts the complete authority capsule, current CAS head, frozen effect plan, and retry state. Guest removal requires surviving-host acceptance. Host removal capable of stranding the publisher requires finalizer acceptance.
3. **Receipt-chronology invariant:** publisher-signed pre-removal receipts describe only authorization, identities, counters, before/quiesced observations, and frozen plans. They never assert future absence or parity. Post-removal absence is recorded only in separately typed harness parity proofs. A parity proof cannot retroactively authorize an earlier effect.
4. **Contiguous terminal ownership invariant:** after finalizer acceptance, the publisher makes no mutation and the finalizer exclusively owns one frozen host terminal-removal suffix. Harness-owned residual cleanup is a later separate phase and never enters the finalizer API.
5. **No-originating-authority invariant:** the finalizer can verify and execute authority but cannot create, countersign, repair, widen, or substitute it.

### Closed prospective transition order

The guest transition is:

`Prepared -> QuiescePrepared -> Quiesced -> PreRemovalReceiptSigned -> ReceiptExternallyDurable -> AcknowledgementExternallyDurable -> AcknowledgementCASBound -> HandoffHostBound -> Removing -> Removed -> ParityExternallyDurable -> ParityHostBound`.

`Quiesced` stops ordinary service/dispatch while preserving the exact retirement signer, protected journal/CAS, handoff channel, and retry executor. It removes no component. `HandoffHostBound` requires the surviving host to independently verify and durably copy the complete canonical guest capsule, guest-CAS head, target/effect plan, and retry state. Only that surviving host owner may then execute removal. `Removed` is an effect observation, not parity. Host retirement cannot begin until `ParityHostBound`.

An unused reservation follows the same authority order: publisher-signed exact pre-effect proof; external proof durability; independent external acknowledgement durability; surviving protected-CAS binding; then exact pairing-record restoration/removal. Timeout, process death, pathname state, or memory-only acknowledgement is insufficient.

The host transition is:

`Prepared -> QuiescePrepared -> Quiesced -> PreRemovalReceiptSigned -> ReceiptExternallyDurable -> AcknowledgementExternallyDurable -> AcknowledgementCASBound -> FinalizationRequestFrozen -> FinalizerAccepted -> Removing -> EffectsComplete -> HarnessResidualRemoving -> ParityExternallyDurable -> TerminalAcknowledgementBound -> Complete`.

`Quiesced` preserves the publisher's signing, protected CAS/lock/journal, latch, finalizer handoff, and retry capabilities and performs no removal. `FinalizationRequestFrozen` binds one canonical request digest, the current live-CAS precondition, exact authority bytes, identities, targets, and effect order while publisher authority survives. `FinalizerAccepted` exists only after the finalizer has mutually attested its peer, independently verified live CAS and every authority join, claimed the request digest, and durably copied the complete capsule and predecessor head into its surviving root journal. It is the irreversible commit point. No host removal occurs before it.

`Removing` is exclusively finalizer-owned and uses a prepared/invoked/observed substate for each effect. `EffectsComplete` is an immutable finalizer response, not parity. The harness may then remove only its separately precommitted attempt-created artifacts under its own durable journal. It subsequently produces the externally durable parity proof and terminal acknowledgement. `Complete` exists only after the finalizer journal binds those artifacts to the exact response/journal head.

### Receipt and parity artifact wall

Prospective V2 must define separate closed canonical types and signature domains for:

- guest and host publisher-signed pre-removal receipts;
- harness receipt-durability acknowledgements;
- the finalization request, which transports rather than originates authority;
- guest and host harness parity proofs;
- the host terminal acknowledgement;
- finalizer journal generations and immutable `EffectsComplete` response.

Each type requires strict owner/version checking, unknown/duplicate-field rejection, canonical bytes, named hash inputs, exact predecessor joins, and golden vectors. No pre-removal type may contain a claimed after-state. No post-removal type may authorize an already-performed deletion. V1 signed bytes remain unchanged and historical-only; product V1/V2 state keeps the retirement commitment null and cannot enter these routes.

### Fixed external-finalizer boundary

The prospective finalizer is root-owned, pre-baseline external evidence infrastructure and not an installed product component. It is a non-authoritative, dual-authorized executor of exactly one frozen contiguous host suffix. Before G6 contract closure, the accepted contract must bind literal values for its executable, root launchd service/route, local endpoint, immutable configuration if any, root journal, exact parent/file physical and code identities, source/build identity, capability digest, endpoint ownership/mode, intended-principal coordinator identity, framing limit, target roles, and fixed effect implementations. This amendment creates no placeholder executable or ambient route.

The only privilege transition is the precommitted launchd route starting the precommitted executable as root. No sudo, setuid, caller-selected executable, mutable configuration, direct publisher call, product install/update/remove route, recovery route, or network route is allowed. The only caller is the exact precommitted external coordinator as the intended principal.

Before parsing authority-bearing bytes, finalizer and coordinator mutually verify the fixed endpoint and exact audit/peer credentials, effective UID and canonical account, PID plus process-start identity, executable physical identity, code identity/designated requirement, and image digest. They repeat peer verification after complete framing and request EOF and before acceptance. Socket ownership/mode alone is never authority.

The accepted connection is normalized to FD3. It contains one bounded length-prefixed canonical document and then request EOF; the response has the same one-frame-plus-EOF property. The later G6 contract freezes the exact maximum and encoding. Missing/trailing bytes, duplicate/unknown fields, a second frame, absent EOF, wrong owner/version, peer drift, extra argv/stdin, ambient environment, cwd input, or alternate framing stops before acceptance with no effect.

Authority-bearing records are carried inline. No caller-supplied path is followed. The finalizer accepts no arbitrary command, action, path, launchd label, Keychain service/account/tag/predicate, environment-derived target, dynamic/plugin load, product uninstall, publisher upgrade, or recovery input. It recomputes exact target identities and allowed operations solely from closed component roles, the precommitted exhaustive evidence ledger, and its compiled allowlist.

Before `FinalizerAccepted`, the finalizer independently verifies the publisher signature, exact durable harness acknowledgement, receipt/ack file and parent identity claims, live signed protected CAS and generation/counter, CAS-bound request digest, terminal latch and current lock identity, guest parity binding, authorization window at first acceptance, caller/finalizer/code/build/evidence/scope identities, exact target membership, and absence of a prior different request claim. It acquires the exact current-anchor lock before reading the live CAS and holds it through the acceptance generation.

The finalizer durably claims one request/idempotency digest and transfers successor CAS ownership into one fixed root-owned, no-follow, per-scope locked, hash-chained, generation-CAS journal. Each generation and its directory are fsynced and reopened/verified; the journal survives all publisher deletions. After a crash, the exact lock must be reacquired and the journal/live target state revalidated before continuing.

For each effect, `EffectPrepared` is durable before invocation, `EffectInvoked` is durable immediately before the system call, and `EffectObserved` is durable only after exact final-state observation. A restart after `Prepared` may invoke. A restart after `Invoked` must observe before any reinvocation. Mismatch or ambiguity preserves state. Caller disconnect before acceptance has no effect; disconnect after acceptance leaves the finalizer responsible for convergence. A byte-identical committed caller may rejoin the same digest; an alternate digest is rejected. Expiry applies at first acceptance only. Same-digest terminal replay returns the immutable response without inspecting new targets.

The frozen suffix may contain only precommitted host-publisher roles: fixed publisher service/endpoint quiesce or bootout; exact pairing/lifecycle records; protected wrapper/current anchor; the one exact P-256 signer; the exact durable lock if attempt-created and no longer needed; fixed helper/plist/provenance/bootstrap artifacts; created-empty publisher parents; and the latch last. Detailed order is determined by the exhaustive resource DAG and separate `required_until` holdbacks. Harness-owned selected-prefix/build/coupled artifacts are prohibited finalizer targets.

The component DAG and holdbacks may refine but cannot invert these terminal edges: host quiescence and `FinalizerAccepted` precede all host-publisher removal; protected-wrapper/current-anchor final absence precedes signer deletion; the exact current-anchor lock and latch remain reacquirable through signer final-absence observation; helper/plist/provenance removal follows signer absence; the latch is the last publisher target; and the external finalizer, journal, coordinator, and receipt infrastructure survive through terminal acknowledgement and `Complete` as measured pre-baseline evidence infrastructure.

The finalizer has **no private signing key** and cannot sign receipts or originate authority. A finalizer key is prohibited unless a later larger amendment defines its necessity, provisioning/access control, every signature domain, and its own retirement protocol. The finalizer's immutable response is authenticated by the committed journal/CAS chain and then bound by the harness-signed parity proof and terminal acknowledgement.

The protocol accepts only the same-digest finalization request/rejoin and, after `EffectsComplete`, the exact harness terminal acknowledgement for that request. There is no caller-selected action. Before acceptance the only response is a no-effect stop. After acceptance, responses are limited to same-digest in-progress/rejoin or immutable `EffectsComplete` with the request digest and journal head. Only after the journal binds exact parity and terminal acknowledgement may it return immutable `Complete`; effects completion alone is never terminal success.

### Keychain capability and experiment gate

The exact prospective finalizer identity's noninteractive delete-only System-Keychain capability is mandatory evidence before G6. In a separately authorized rollback-safe disposable native-macOS experiment, the exact identity and proposed access-control posture must prove that it can delete the one exact prospective signer and deterministically re-observe absence, while sign, export, ACL/trusted-application mutation, other-key deletion, and target broadening all fail without UI or mutation. Root is not assumed sufficient. SecurityAgent activation/window/prompt, ambiguous status, identity/access-control drift, or residual surrogate fails the gate.

A failed gate pivots only the terminal signer-deletion mechanism. It cannot broaden the finalizer, merge recovery, authorize a prompt, or create product-uninstall/general-recovery power.

The same separately authorized experiment packet may include creator-route arms for the old orphan only when conclusions remain separate. It must compare query-level UI failure with a fresh process whose first Security-framework call successfully disables process interaction before the same query-level UI-fail operation, plus wrong-identity and already-absent retries. Every effectful arm uses a fresh surrogate, precommitted repetition count, exact logs/settle window, and closed rollback in a rollback-capable native-macOS environment. It uses no live orphan or product identity. The experiment grants no live authority.

No password, credential, Touch ID, `Allow`, `Always Allow`, ACL/trusted-application edit, or persistent authorization change is permitted. Any UI, duplicate/mismatch, unaccounted SecurityAgent state, suppression failure before query, rollback failure, or baseline drift is an experiment failure and no conclusion. A later one-prompt live recovery fallback requires separate exact user authorization binding the one old key, executor/code identity, machine/build/session, operation, invocation, expected prompt text and SecurityAgent/code identity, and exactly one prompt. Cancellation, mismatch, a second prompt, `Always Allow`, persistent ACL/trust change, or UI on a no-UI route terminates that authorization.

### Closed stops, review matrix, and authority posture

Before `FinalizerAccepted`, any authority, identity, peer, code, framing, durability, CAS, target, lock, capability, or baseline mismatch is a no-effect preserving stop. After acceptance, the same-request journal is the only rejoin authority. Alternate input, target drift, unexplained state, ambiguous effect, UI, lock loss, journal corruption, or parity failure stops without widened cleanup or substituted authority.

The only stop/rejoin classifications are `SafePreAcceptanceStop`, `RejoinAcceptedRequest`, `IdentityOrAuthorityStop`, `InteractionStop`, `AmbiguousEffectStop`, and `TerminalParityFailure`. They never grant new authority or success.

| Injected edge | Mandatory result |
|---|---|
| Before guest receipt/acknowledgement durability or CAS binding | No guest deletion; same-state retry or preserving stop. |
| After guest CAS binding but before surviving-host acceptance | No guest deletion; signing/handoff remains callable. |
| Before host receipt/acknowledgement durability, CAS binding, or finalizer acceptance | No host deletion; same-state retry or `SafePreAcceptanceStop`. |
| After finalizer acceptance and before effect invocation | Surviving finalizer journal retains ownership and resumes the same digest. |
| After `EffectInvoked` and before `EffectObserved` | Observe before any reinvocation. |
| Caller disconnect after acceptance | No rollback or authority return; finalizer continues or same-digest rejoin occurs. |
| Key deletion/absence UI or ambiguity | `InteractionStop` or `AmbiguousEffectStop`; no approval or widened query. |
| `EffectsComplete` before parity/terminal acknowledgement | Immutable effects response only; never terminal success. |
| Parity/terminal acknowledgement mismatch | `TerminalParityFailure`; no evidence, mirror movement, or closeout. |
| Same-digest retry after `Complete` | Byte-identical terminal response; no new target inspection. |

A later implementation review must inject failure before and after every receipt/acknowledgement write, file fsync, parent fsync, reopen/hash, signature, CAS, peer re-attestation, EOF, finalizer claim, journal generation, lock acquisition/reacquisition, effect prepared/invoked/return/observed boundary, caller disconnect, service bootout/absence, Keychain delete/final absence, unlink/parent-fsync, response, harness residual effect, parity proof, and terminal acknowledgement. It must prove no destructive-before-ack path; no host deletion before finalizer acceptance; observe-before-reinvoke after ambiguous returns; same-digest deterministic replay; peer/path/identity/target substitution rejection; unknown-field and alternate-digest rejection; cross-lane decoder rejection; and every SecurityAgent/UI stop.

This amendment remains planning authority only. Literal V2 schema/domain/route constants, implementation file/symbol fences, experiment execution, live recovery, landing, evidence, mirror refresh, and `MAC-CLOSEOUT` each require their later serial authorities from the phase map.

## `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` contract (2026-08-19; macOS lane)

This gate supersedes the archived R3 macOS/recovery/retirement/finalizer/E03 and Windows-predecessor
contracts above **for macOS-lane scheduling only**. It is not the global product-work predecessor,
is not self-authorizing, and Phase 1 grants no Phase 2 implementation or native-operation authority.

### Admission

A later parity task must freshly bind live Git/source truth, exact edit and symbol fences, current
installer/runtime behavior, the selected user prefix, selected Lima instance, forwarding endpoint,
Linux-preservation obligations for shared scripts, and native action/restoration rules. Before any
effect it must complete an exact read-only overlap check, without a Keychain query, showing that
those declared developer resources are disjoint from Attempt 4. Any actual path/resource collision
requires a separately authorized disposition and leaves this gate open.

### Allowed completion claim

The gate may close only after the current product proves user-owned-prefix install/uninstall,
current shims/configuration/binary staging, current Lima/`world-service` provisioning, typed
selected-prefix and Lima-instance mapping, safe host-to-guest forwarding, and native install →
exercise world → uninstall → verify → reinstall behavior. Exact intended removal and preservation
of unrelated/pre-existing state are part of acceptance. Static checks alone are insufficient.

### Prohibited ownership and actions

The parity corridor owns no System-Keychain record, protected publisher, macOS lifecycle
LaunchDaemon/privileged host helper, terminal-retirement/finalizer state, E03/freeze/identity-
rotation/assurance evidence, or Attempt 4 artifact. It must not inspect or mutate Attempt 4
Keychain records; retire, migrate, overwrite, adopt, or clean its fixed privileged artifacts; use
unfinished lifecycle machinery as cleanup; or revive the archived architecture.

Landed Linux R3 facts remain historical facts and receive no new implementation/evidence claim;
later shared-script changes must preserve Linux behavior. Windows remains untouched and incomplete
where applicable, outside this lane, and deferred until a separately authorized post-runtime-refactor
scheduling decision.

### Exit and continuation

Any ambiguity, overlap, protected-lifecycle dependency, unrelated-state change, native failure, or
inexact uninstall/restoration keeps `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` open. Its closure neither
blocks nor authorizes the Linux-first runtime-refactor sequence; the global reentry gate separately
requires a fresh live-repository bind and exact packet selection. This contract does not dispatch
A1.3, A1.4, Windows, E03, or any successor. Protected machinery can return only through a separate
production threat-model decision.

The complete decision record is
[`macos-dev-parity/DECISION.md`](macos-dev-parity/DECISION.md).


## `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` contract (2026-08-20; closed selection record)

This was the global documentation/control-plane rebind-and-selection gate. It superseded the former
use of `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` as the global product-work predecessor, while leaving
the macOS lane contract above unchanged in scope. It is now closed only as the selection of the
historical [A1.3 Linux-first implementation packet](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md),
later amended so the active implementation authority is
[`linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md).

### Admission

The reentry task must bind the live product branch, ancestry, worktree/index, current source and
control-pack state, the specific historical evidence it relies on, and Linux-first adoption posture.
It must name exactly one subsequent runtime-refactor packet with its owner, path and symbol fences,
dependencies, acceptance criteria, verification, and macOS/Windows exclusions.

### Allowed completion claim

The gate may close only as a documentation/control-plane rebind that selects that one later packet.
It may not claim implementation, Linux proof, cross-platform completion, or promotion of a runtime
seam.

### Prohibited ownership and actions

This gate owns no runtime, installer, Lima, macOS, Windows, archive, Keychain, Attempt 4, or native
operation. It does not reopen Linux R3, make macOS parity a predecessor, or dispatch the selected
packet without separate authority.

The complete global scheduling decision and closed selection record are
[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md). The
separate, active A1.3-P0 implementation authority is
[`linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md`](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md),
and the held-later A1.3 packet remains
[`linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md`](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md).
