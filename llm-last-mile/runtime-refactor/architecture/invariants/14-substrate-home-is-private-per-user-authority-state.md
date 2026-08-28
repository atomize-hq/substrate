**Kind:** architecture
**Stable ID:** `shared-invariant-14`
**Canonical for:** invariant 14 only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted invariant source body only; no implementation authority
**Source span:** [`../../01-target-architecture.md#14-substrate_home-is-private-per-user-authority-state`](../../01-target-architecture.md#14-substrate_home-is-private-per-user-authority-state) lines 238–380
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../README.md`](../README.md), [`../../01-target-architecture.md`](../../01-target-architecture.md)

# Invariant 14: SUBSTRATE_HOME is private per-user authority state

### 14. `SUBSTRATE_HOME` is private per-user authority state

`SUBSTRATE_HOME` contains one operating-system user's configuration, policy, dependency inventory,
runtime, and authority state. Creating and accepting that root is part of authority bootstrap: the
physical directory is owned by the intended per-user owner, has exact owner-only mode `0700`
independent of ambient umask, has no effective other-principal authority, rejects every observable
POSIX access or default ACL, and is opened and revalidated no-follow before any descendant
bootstrap. On Linux, `ENODATA` means only that the kernel returned no ACL data; it does not prove
physical xattr absence and is accepted only with exact owner/type/`0700` mode, stable descriptor
identity, no-follow traversal, and replacement-safety proof. Existing nonconforming roots fail
closed without chmod, chown, ACL removal, migration, adoption, deletion, or other automatic repair.
Custom homes remain valid only when they satisfy the same contract. Multiple operating-system
principals directly sharing one `SUBSTRATE_HOME` are unsupported in A1 V1.

Directory creation and identity acceptance are distinct. `mkdirat` success establishes only a
candidate name under an already-opened trusted parent; portable Linux/macOS APIs do not atomically
create a directory and return its inode-bound handle. The accepted `PrivateSubstrateHomeV1`
physical identity begins at the first successful no-follow directory open followed by
descriptor-based owner, type, exact-mode, ACL, filesystem, and physical-identity validation. The
parent remains descriptor-bound across candidate creation and opening and must already have the
expected type and owner and no create/delete/rename/replacement authority for another principal.
Ancestor access ACLs and default ACLs are different security surfaces. A named access entry is
evaluated after the ACL mask: effective read/search without effective write does not itself grant
replacement authority, while any effective write bit remains rejected, including write-only
entries. Default ACLs can be inherited into a new child and are rejected on every ancestor before
candidate creation in V1; supporting one requires a later separately approved contract. Malformed,
unreadable, unsupported, or ambiguous ACL state also fails closed. Under supported Linux POSIX
access-ACL semantics, `ACL_MASK` is the file group-class mode bits, so a named user/group entry
cannot retain effective write while the descriptor's authoritative group-write bit is clear.
Unknown ACL models are outside that proof and fail closed. These ancestor distinctions never weaken
the final root's exact owner, exact `0700`, observable-ACL rejection, or owner-only descendant modes.
Legitimate concurrent Substrate creators converge when one creates and another observes
`AlreadyExists`: each no-follow opens and validates the candidate, and only its exact accepted
descriptor identity may proceed.

After that first accepted open, all descendant access remains descriptor-relative and the child
name is rejoined to the accepted descriptor identity at required publication or acceptance
boundaries. Rename, replacement, owner/mode/ACL drift, or validation uncertainty fails closed.
Preventing malicious root or malicious code already executing under the same UID from substituting
the child before the first descriptor acquisition is outside the A1 V1 threat model. A1.1d-5
therefore requires neither an impossible atomic create-and-bind claim nor a privileged creation
broker; adding such a broker is a separately approved architecture change.

World members do not gain direct traversal authority over the host user's private home. They
consume configuration, policy, dependency, and credential material through Substrate-owned
projection and mediation boundaries. A privileged Substrate service may access the private root
only as the currently landed service boundary requires; that access does not convert the root into
shared state. If an unprivileged world process is found to depend on direct traversal, work stops
for an explicit projection/broker boundary change rather than broadening permissions.

Private-home enforcement does not alter effective policy or narrow world capabilities. Existing
filesystem read/discovery/write rules, host visibility, isolation and copy/overlay strategy,
network modes and DNS enforcement, PTY/non-PTY execution, dependency synchronization, gateway
handoff, runtime availability, shims, replay, traces, diagnostics, and lifecycle/binding behavior
remain governed by their existing contracts. A separate shared installation root may be designed
later; `SUBSTRATE_ROOT`/installation-root separation is not part of A1.1d-5.

#### Installation prefix authority and platform realization

The public installer or uninstaller selects exactly one normalized `InstallBootstrapContextV1`.
For V1, `selected_host_prefix`, `host_substrate_home`, and `host_substrate_root` are the same host
path and are bound to one platform-scoped intended principal. A declared `--prefix`, `--home`,
`-Prefix`, or the wrapper's self-derived installed prefix fixes that context before any child,
generated projection, sudo crossing, service renderer, doctor, runtime provisioner, or platform
adapter runs. A child may validate the transported context but may not reconstruct it from
`$HOME`, `$USERPROFILE`, CWD, an outer `SUBSTRATE_HOME`, or an independently selected root.

Generated `env.sh`, manager/profile/preexec helpers, configuration, version, dependency,
install-state, and Lima known-hosts files are projections of that decision. They never become
prefix-selection authority. When an
installed helper at A runs under conflicting ambient home B, it must self-derive A, validate any
encoded context/commitment against A, export both `SUBSTRATE_HOME=A` and `SUBSTRATE_ROOT=A`, and
consume only A. A missing, malformed, or conflicting projection fails closed; B is not a fallback.
The normal install/uninstall product gate must pass without an outer override. An explicitly named
diagnostic override may exercise a negative test, but it cannot satisfy product proof.

The shared Rust wire model lives in `transport-api-types`; OS path/principal construction remains
at the public shell/script entry. Its fixed line-framed, base64url field encoding has one
cross-language golden-vector suite and does not reuse, move, or duplicate the shell-private A1
authority-store canonical codec. Successful install-sensitive CLI routes construct this context
before private-home/dependency scaffolding; parse failures, help, `--version`, and `--version-json`
exit without creating private-home, dependency, shim, manager, trace, or generated-install state.
Installer-managed or focused private-home regression callers that need the existing scaffold use the
hidden `--install-bootstrap-home-v1` action only with
`--install-bootstrap-context-v1 <carrier>`. The root dispatcher strictly authenticates the carrier,
binds its Unix account+UID to the current principal, validates checked H/R projections, calls only the
existing explicit-context private-home/dependency bootstrap, and returns. Environment values alone
cannot select the action or internal-child mode, and normal output/errors/logs/traces never disclose
the carrier. Automatic
shim deploy, shim doctor/repair, physical shim telemetry, and shell/replay backend-factory callers
consume explicit context or mapping projections rather than common ambient path resolution.
Installer-managed product-CLI children use the hidden argv carrier as their child discriminator;
an environment carrier never selects internal mode. A context-aware Substrate-owned sudo child validates that
same argv carrier, while an arbitrary system tool receives only minimal context-derived argv from
the still-owning installer and never receives a meaningless carrier or preserved ambient home.
Before any internal-child dispatch, the current Unix account+UID or Windows account+SID must equal
the committed principal; a self-consistent carrier for another principal fails before projection
or mutation. Standalone self-derivation accepts the live Unix release link, Unix dev A/bin symlink,
and Windows release A/bin copy shapes through one exact invocation-witness algorithm; a direct
repository binary or ambiguous PATH witness still requires an explicit prefix.
World-deps/doctor/config/policy/gateway leaves receive typed context or a checked projection, and
host Codex paths derive from the committed Unix principal's account-database home.

Route C's Host/World doctor identity fields are a non-authoritative projection of that same
authenticated context. The public doctor execution is not classified as read-only: on Linux, the
existing `handle_host_command` → `host_doctor_main` and
`handle_world_command` → `world_doctor_main` paths carry typed IH to the final host-visible JSON or
text report. Optional `WorldDoctorReportV1` host-prefix and commitment fields remain data only: old
wire JSON may omit them, the in-world `doctor_world` producer emits them absent and reads no host
carrier, environment, prefix, principal, or context, and only the host shell may populate them from
typed IH. The legacy report adapter may project them only when its existing caller explicitly
supplies typed IH. A conflicting ambient home/root or generated projection cannot replace A.
Diagnostics never decode or disclose hidden carrier bytes, credentials, request bytes, secrets, or
sensitive principal material. The projected identity fields cannot install, repair, restart,
clean up, or otherwise change world, policy, capability, filesystem/network enforcement,
placement, caging, receipt, supervisor, retained-worker, credential, or lifecycle state. The public
Doctor compatibility path retains its existing active readiness, socket, service, endpoint, and
probe behavior. This is not a new authority seam, module owner, or execution family.

At the Route D checkpoint, the intended rule applied to the complete authenticated
shell/shim-doctor snapshot carrier.
The typed Unix report path passes one already-validated `InstallBootstrapContextV1` through
`collect_report_for_context` and `build_report` to both its embedded world-doctor and world-deps
branches. Health fixtures are diagnostic projections beneath `A/health` only; an ambient
`B/health/world_doctor.json` or `B/health/world_deps.json` cannot replace, supplement, redirect, or
appear in the A-bound report. When the embedded world-doctor branch invokes the existing product
CLI, the child receives the same canonical hidden argv carrier and only context-derived checked
environment projections. Those projections are child transport, never authority reconstruction.
Later source closure proved that the ordinary child could activate infrastructure and probe the
world, so the claim that the nested diagnostic was non-mutating was incorrect and was superseded by
the later historical F5-PD section, whose boundary the completed-F record retains. A missing,
malformed, tampered, wrong-principal, or contextless
Unix witness fails closed. Carrier bytes, credentials, request bytes, commitments not
already intended for public diagnostics, and sensitive principal data never enter report JSON,
text, fixtures, errors, logs, traces, or snapshots.

This closure changes no report schema, world-doctor meaning, fixture payload meaning, dependency
classification, world enforcement, policy, capability, service, installation, cleanup, receipt,
supervisor, retained-worker, credential, or lifecycle behavior. Non-Unix behavior stays on its
existing cfg path. The Unix checked-projection `collect_report` entrypoint and its crate-private
re-export remain temporary behavior-frozen R2-3 compatibility; Route D neither calls that path from
typed Health nor migrates the physical shim, replay, global trace compatibility, or platform-native
adapters.
