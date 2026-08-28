**Kind:** contract and gate record
**Stable ID:** `A1.2-earlier-histories-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** A1 canonical encoding/persistence, DurableSessionAuthorityV1, and strict transition-intent contracts only
**Source span:** composite of the three preserved root compatibility spans listed in the extraction ledger

# A1.2 and earlier packet-histories contracts and gates

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

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#remaining-r2-2-same-process-carrier-closure`](../a1.1d-5r2-2f/contracts-and-gates.md#remaining-r2-2-same-process-carrier-closure).

##### Route C authenticated Host/World doctor projection

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#route-c-authenticated-hostworld-doctor-projection`](../a1.1d-5r2-2f/contracts-and-gates.md#route-c-authenticated-hostworld-doctor-projection).

#### R2-2 source closure, E closeout, F0/F0a/F0b prerequisites, and remaining F contract

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/contracts-and-gates.md#r2-2-source-closure-e-closeout-f0f0af0b-prerequisites-and-remaining-f-contract`](../a1.1d-5r2-2f/contracts-and-gates.md#r2-2-source-closure-e-closeout-f0f0af0b-prerequisites-and-remaining-f-contract).
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

Compatibility anchor only; canonical content: [`a1.1d-5r2-3/contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification`](../a1.1d-5r2-3/contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification).
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
    `HostExecutionEpisode` owns observation reporting and the bounded A1.3-P1 real helper/REPL
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
    the later A1.3-P1 packet first owns the real helper/REPL public transport consumer, exact
    public actor-event resolution, and the mechanical B1 acceptance-context projection on that same
    path only through the exact retained-turn `orchestrator_world_dispatch.rs`
    acceptance-context construction/projection seam; A2 later generalizes episode observations
    without weakening these A1 commitments.
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
