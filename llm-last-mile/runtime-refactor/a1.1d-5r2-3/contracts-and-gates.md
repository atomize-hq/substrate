**Kind:** contract/gate
**Stable ID:** `A1.1d-5R2-3-family`
**Canonical for:** A1.1d-5R2-3 contracts and gates
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** composite of the preserved root compatibility spans listed in Source provenance below
**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-3 contracts and gates

##### `PlatformBootstrapMappingV1` construction and verification

The host adapter constructs the mapping only after validating the host carrier. It recomputes the
host commitment, resolves the exact platform instance, queries the guest account database, and
normalizes the platform-native absolute home with the
[Unix host-path normalization rule](../04-contracts-and-gates.md#host-path-normalization-and-validation).
Lima identity is
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
[terminal receipt](../review-control/r2-3z-terminal-receipt.json), SHA-256
`681700be6b4c483a78896573f8c982591c6f026fae566fe8c45b5b829f82e58a`, and the byte-identical
tracked [review-cycle record](../review-control/r2-3z-review-cycle-record.json), SHA-256
`a3236d6ec901906d1ef84851d45995b2d595820be93d20cb40e3af1bb9db73ef`. The receipt binds terminal
subject fingerprint `sha256:2478c015df2851e05b767f81a8cc889cd1b936be38fbc11c33e816f2ff607a5a`
and terminal review verdict `CLEAN`. The tracked JSON retains its original external review paths;
the raw external review Markdown and subject-fingerprint file were not copied into this repository.
[`review-control/README.md`](../review-control/README.md) records the source paths and retention limit.

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

**Source provenance:**
- extracted from [`04-contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification`](../04-contracts-and-gates.md#platformbootstrapmappingv1-construction-and-verification), lines 1417–1611; baseline span SHA-256 `11cce17bba267d0aa42b56ba29045bbdecd40d226462f80c340f83e40669582c`
- extracted from [`04-contracts-and-gates.md#r2-3-closeout-status`](../04-contracts-and-gates.md#r2-3-closeout-status), lines 7019–7084; baseline span SHA-256 `89892cf1df9dbf4c3eb148fda6942f3f88e97256c85a9fdd518fcc62ad1f6dea`
