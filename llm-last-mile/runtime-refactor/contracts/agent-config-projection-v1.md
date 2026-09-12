**Kind:** contract
**Stable ID:** `agent-config-projection-v1`
**Status:** canonical specification; E3-A has a preserved unlanded candidate under bounded Linux proof recovery; E3-B through E3-F are not admitted, dispatched, or implemented
**Canonical for:** E3 config-projection identity, record, reference, versioned member-dispatch carrier, persistence, validation, Codex 0.125 native rendering, compatibility, and D1 handoff boundaries
**Correction source baseline:** commit `81cfd33d4c5d16c31c837eeddff769995c566570`, tree `d40f6663a2aa01996ce8b4957f4aae0d30830961`, parent `138864a26dbc4721366c6cc8934d464d1929a189`
**Landed prerequisite fact:** `E2-RM` is landed at `2012bb8b5562a73ed0ee45238c19252c16b25065`; that landing permits this documentation correction but is not an E3 runtime dependency
**Supersedes:** only E3 planning statements that leave projection schema, persistence, versioning, Codex ambient-config closure, or the absent local `crates/codex` unresolved
**Superseded by:** none
**Projection consumers:** [`../slices/e3-agent-config-projection-and-gateway-adoption.md`](../slices/e3-agent-config-projection-and-gateway-adoption.md), [`managed-gateway-adoption-v1.md`](managed-gateway-adoption-v1.md), [`world-runtime-adapter-execution-envelope-v1.md`](world-runtime-adapter-execution-envelope-v1.md), [`retained-worker-manifest-v1.md`](retained-worker-manifest-v1.md)

# `AgentConfigProjectionRecordV1`

> **Authority boundary:** E3 publishes one independently valid, non-secret config projection for an
> exact retained worker before D1 exists. The projection consumes the already-immutable E2 launch or
> fork cap without mutating, reconstructing, or re-owning E2. D1 later consumes the opaque published
> capability and owns `WorldRuntimeAdapterExecutionEnvelopeV1`; E3 does not construct that envelope,
> receipts, retained manifests, broker policy, or E4 workspace synchronization.

The preserved E3-A wire candidate is not complete or landed. This documentation correction
authorizes only a later bounded Linux proof-recovery dispatch; it changes no product/test bytes,
executes no missing proof, and grants no E3-B admission or successor dispatch.

## Source-grounded correction

At the authority baseline:

1. `crates/shell/src/execution/agent_inventory.rs` supplies placement-aware inventory and policy-
   overlay inputs, but no durable projection authority.
2. `crates/world-service/src/member_runtime.rs::prepare_codex_runtime_env` creates an isolated
   launch directory, copies host auth through `codex::CodexHomeLayout::seed_auth_from`, and renders a
   bounded host-derived `config.toml`. That code names itself a compatibility bridge.
3. There is no repository path `crates/codex`. The `codex::CodexHomeLayout` helper is the external,
   exactly pinned Cargo package `unified-agent-api-codex = 0.3.7`, declared by `crates/shell` and
   `crates/world-service`. E3 must add the shared `crates/config-projection` authority crate; it must
   not fabricate a local `crates/codex`.
4. The world dependency inventory provisions the pinned official OpenAI Codex `0.125.0`
   `x86_64-unknown-linux-musl` release archive and verifies SHA-256
   `4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001`; it does not source-build
   Codex. This is distinct from the helper crate above. Codex 0.125 discovers user config from
   `${CODEX_HOME}/config.toml`, also has
   system, managed, project, and session-override layers, and accepts custom `model_providers` with
   `base_url`, `wire_api = "responses"`, and `requires_openai_auth = false`. E3 therefore closes all
   ambient layers rather than assuming `CODEX_HOME` alone is sufficient. See the exact upstream tag:
   [config loader](https://github.com/openai/codex/blob/rust-v0.125.0/codex-rs/core/src/config_loader/mod.rs),
   [loader model](https://github.com/openai/codex/blob/rust-v0.125.0/codex-rs/core/src/config_loader/README.md),
   [provider schema](https://github.com/openai/codex/blob/rust-v0.125.0/codex-rs/model-provider-info/src/lib.rs),
   [MCP schema](https://github.com/openai/codex/blob/rust-v0.125.0/codex-rs/config/src/mcp_types.rs), and
   [configuration reference](https://github.com/openai/codex/blob/rust-v0.125.0/docs/config.md).
5. `MemberDispatchRequestV1` is strict: its private deserialization form denies unknown fields and
   validation requires `schema_version == 1`. E3 adds V2 beside it; it does not add an optional field
   to V1 or weaken V1 parsing.
6. The completed E2 baseline already pins the immutable launch/fork cap before member registration.
   E3 consumes that exact ref and revisions. `E2-RM` is already landed at
   `2012bb8b5562a73ed0ee45238c19252c16b25065`; its landed historic accepted-work read behavior made
   shared HSA root-lock serialization a prerequisite for this authority correction, but it is not
   an E3 runtime dependency, E3 prerequisite, or E3 write surface.
7. `ExecuteRequest.member_dispatch`, `WorldService::execute_stream`,
   `resolve_authoritative_member_placement_context`, `requested_shared_world_owner_spec`,
   `exact_bound_world_ownership_adoption`, `convert_member_dispatch_request`, and
   `validate_member_dispatch_binding` are all V1-typed at the baseline. A transport-only V2
   declaration would therefore be unreachable; E3's exact fence includes the minimum version-aware
   path through each symbol.
8. The baseline world Codex inventory is V2 and explicitly has `mcp_client: false`; its placement
   schema has no model/MCP/feature fields. The landed gateway binding for `cli:codex-world` routes the
   public model `codex` to the existing provider target `codex-mini-latest`. E3 preserves that mapping
   and adds a strict V3 input instead of inventing values.
9. Local source for pinned `unified-agent-api-codex` 0.3.7 allocates an output path through
   `std::env::temp_dir()` when none is supplied and executes `Command::new(binary_path)`; the pinned
   high-level adapter supplies no output path. E3 cannot claim descriptor execution or native-root
   closure through that launch path and therefore adds an E3-only local adapter while freezing V1.
10. The installed Linux world-service unit has no `User=` and explicitly receives ambient
    `CAP_SYS_ADMIN`, `CAP_NET_ADMIN`, `CAP_DAC_OVERRIDE`, and `CAP_SYS_PTRACE`; baseline gateway and
    member spawns do not descend credentials. E3 therefore cannot release either credential-bearing
    gateway or Codex through those baseline spawn semantics.
11. Baseline `ActiveMemberRuntime` retains a binary path, environment, E2 activation, and worker-cap
    identity but no E3 capability/native/gateway state; both submitted-turn paths reconstruct a
    prompt bridge from that path. E3 must add exact retained state and may not treat successful first
    launch as a capability.
12. The E2-authorized continue/fork-command path reaches
    `continue_world_worker_fork_command_bootstrap_after_delivery` and
    `build_continue_world_worker_fork_command_transport_request` before the common fork builder.
    Both are therefore in the exact V2 carrier fence.
13. `crates/world-service/src/main.rs` uses `#[tokio::main(flavor = "multi_thread")]` at the
    baseline. Tokio creates worker threads before the async body, while Linux capability sets are
    per-thread; E3 transition capabilities cannot be parked correctly from that async body.
14. The landed E2 Landlock plan in `internal_exec.rs` grants its fixed system roots and E2
    allowlists, but it contains neither the E3 accepted-home native source nor either E3 `/run`
    realization. Landlock stacks by intersection, so an unchanged applied E2 layer cannot later be
    broadened with those support roots.
15. The public transport names are `DispatchPolicyCommitmentRefCarrierV1` and `PolicyRefV1`.
    Shell's similarly named `DispatchPolicyCommitmentRefV1` is private, and no public Rust
    `PolicySnapshotRefV1` type exists at the baseline.
16. Baseline submitted-turn serialization uses `ActiveMemberRuntime.active_turn_span_id`, while the
    initial E3 draft used a separate lifecycle mutex. Separate validation and reservation cannot
    linearize a resumed submit against terminal unregister/restart.
17. Baseline gateway readiness opens its socket from a world-service blocking thread, not a child in
    a dedicated readiness cgroup. It cannot satisfy a socket-cgroup-v2 readiness rule.
18. `CanonicalDirectoryV1` and `DirectoryPhysicalIdentityV1` exist only as shell-private
    host-session-authority types. A shared projection crate cannot import them, and transport cannot
    depend on a projection crate that itself depends on transport.
19. Startup, periodic, and manual GC reach `gc::sweep`, whose `list_netns`, `netns_pids`,
    `delete_nft_table`, and `delete_netns` helpers spawn privileged `ip`/`timeout` processes. They
    must participate in the same process-wide exclusion as every other child launch.
20. `ExecuteRequest` and `MemberDispatchRequestV1` contain no integrated-auth field. At baseline the
    secret-bearing `GatewayIntegratedAuthPayloadV1` reaches world-service only through
    `GatewayLifecycleRequestV1` on `/v1/gateway/sync` or `/v1/gateway/restart`; shell constructs it in
    `world_gateway::resolve_integrated_auth_payload`. Consequently world-service cannot publish an
    E3 `Dormant` record bound to a process-local credential source before a V2 execute merely by
    extending the execute carrier. E3 requires the bounded preparation route specified below.
21. Baseline gateway `codex_auth_context.rs` retains the access token in process-lifetime static
    state, while the Linux service unit has no core-dump exclusion. A process-scoped user-namespace
    boundary does not prevent a dumpable process from writing a core. Both world-service before E3
    body acceptance and the gateway before secret delivery therefore require the independent
    non-dumpable/zero-core barriers below.
22. The landed E2 launch validator requires `retained_worker_launch_authority = Some(exact proof)`
    only for `FreshSpawn` and requires it to be `None` for `Fork`. The E3 prepare request and sealed
    equality join must preserve that exact nullable value.
23. The landed carrier's public constant and gateway consumer both use the exact pointer name
    `SUBSTRATE_LLM_AUTH_BUNDLE_FD`; the generic preserved V1 handoff schema accepts any non-secret
    logical label. E3 must bind the exact landed name rather than treating another label as an
    equivalent secure-FD carrier.
24. The shell authors the accepted-home manifest as the invoking, intended non-root account, while
    both E3 installer source stores are root-published. Their lock and authority-object modes must
    therefore permit read/shared-lock access to the existing `substrate` group without granting any
    group write authority.
25. `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`,
    specifically `StoreLayout::validate_closed_layout`, currently recognizes only `lock`, `tmp`,
    `objects`, `keys`, `retained-worker-admission-v1`, and `dispatch-policy-commitment-v1` as
    top-level directories in the closed `authority-v1` layout. The landed E2-RM read path at commit
    `2012bb8b5562a73ed0ee45238c19252c16b25065` independently repeats that exact manifest in
    `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`,
    specifically `validate_e2_rm_authority_manifest`, and returns `UnsafeNamespaceEntry` for any
    other entry. The E3 registry is therefore an intentional new namespace member whose later E3-B
    storage admission
    must bind both validators for the same one literal accepted directory name; leaving either
    unchanged would make ordinary HSA or landed E2-RM reads reject the registry.

The three draft design documents under `llm-last-mile/` remain reviewed inputs, not authority. This
numbered contract is the controlling E3 specification.

## Additive V1 schema

The Rust-like schema fixes the serialized field set and enum tags. All structs deny unknown fields.
All optional fields are present as JSON `null` in canonical hash preimages even when wire serializers
may omit them outside a preimage.

```rust
// Public E3 type owned by the new config-projection crate. Linux is the only E3 V1 platform.
#[serde(transparent)]
struct Timestamp(String);

struct CanonicalDirectoryV1 {
    physical_path: String,
    physical_identity: DirectoryPhysicalIdentityV1,
}

enum DirectoryPhysicalIdentityV1 {
    Linux { device_id: u64, inode: u64 },
}

struct ConfigProjectionIdentityV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    accepted_home: CanonicalDirectoryV1,
    workspace_root: CanonicalDirectoryV1,
    orchestration_session_id: String,
    retained_participant_id: String,
    bootstrap_run_id: String,
    backend_id: String,
    runtime_family: String, // exactly "codex" for the E3 implementation
    world_id: String,
    world_generation: u64,
    immutable_launch_cap: ConfigProjectionE2CapLinkV1,
    runtime_artifacts: ConfigProjectionRuntimeArtifactsV1,
    identity_hash: String,
}

struct ConfigProjectionRefV1 {
    authority_store_id: String,
    series_id: String,
    record_id: String,
    revision: u64,
    record_hash: String,
}

struct AgentConfigProjectionRecordV1 {
    schema_version: u32, // exactly 1
    identity: ConfigProjectionIdentityV1,
    record_id: String,
    revision: u64,
    predecessor_ref: Option<ConfigProjectionRefV1>,
    logical: LogicalAgentConfigProjectionV1,
    effective: EffectiveAgentConfigProjectionV1,
    native: NativeAgentConfigProjectionV1,
    managed_gateway: ManagedGatewayProjectionV1,
    nonsecret_handoff: NonsecretHandoffProjectionV1,
    activation: ConfigProjectionActivationV1,
    created_at: Timestamp,
    record_hash: String,
}

struct ConfigProjectionE2CapLinkV1 {
    e2_activation_id: String,
    e2_launch_kind: E2MemberLaunchKindV1,
    commitment_ref: DispatchPolicyCommitmentRefCarrierV1,
    commitment_subject: ConfigProjectionE2SubjectV1,
    immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    immutable_worker_cap_created_revision: u64,
    immutable_worker_cap_application_revision: u64,
    policy_snapshot_ref: PolicyRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    request_id: String,
    idempotency_key: String,
    caller_participant_id: String,
    caller_backend_id: String,
    target_backend_id: String,
    target_world: WorldBindingRefV1,
    registry_publication_revision: u64,
}

enum ConfigProjectionE2SubjectV1 {
    RetainedWorkerLaunch {
        retained_participant_id: String,
        bootstrap_run_id: String,
    },
    RetainedWorkerFork {
        source_participant_id: String,
        child_participant_id: String,
        bootstrap_run_id: String,
    },
}

struct ConfigProjectionRuntimeArtifactsV1 {
    codex: DescriptorPinnedArtifactV1,
    world_entry_wrapper: DescriptorPinnedArtifactV1,
    managed_gateway: DescriptorPinnedArtifactV1,
}

struct DescriptorPinnedArtifactV1 {
    role: ConfigProjectionArtifactRoleV1,
    configured_absolute_path: String,
    device_id: u64,
    inode: u64,
    file_type: String, // exactly "regular"
    mode: u32,
    owner_uid: u64,
    byte_length: u64,
    sha256: String,
    authority_ref: RuntimeArtifactAuthorityRefV1,
    provenance: RuntimeArtifactProvenanceV1,
    runtime_support: E3RuntimeSupportManifestV1,
}

struct E3RuntimeSupportManifestV1 {
    schema_version: u32, // exactly 1
    support_policy_version: u32, // exactly 1
    elf_execution_model: E3ElfExecutionModelV1,
    elf_interpreter: Option<RuntimeSupportFileV1>,
    dynamic_loader_cache: Option<RuntimeSupportFileV1>,
    ordered_elf_dependencies: Vec<RuntimeSupportFileV1>,
    ordered_present_common_files: Vec<RuntimeSupportFileV1>,
    system_config_mount_target: RuntimeSupportDirectoryV1,
    manifest_hash: String,
}

enum E3ElfExecutionModelV1 {
    StaticExec,
    StaticPie(E3StaticPieRelocationMetadataV1),
}

struct E3StaticPieRelocationMetadataV1 {
    dynamic_segment_file_offset: u64,
    dynamic_segment_byte_length: u64,
    rela_virtual_address: u64,
    rela_byte_length: u64,
    rela_entry_byte_length: u64, // exactly 24 for ELF64 x86-64
    relative_relocation_count: u64,
    ordered_dynamic_entries_sha256: String,
}

struct RuntimeSupportFileV1 {
    absolute_path: String,
    device_id: u64,
    inode: u64,
    mode: u32,
    byte_length: u64,
    sha256: String,
}

For the sole CA entry, `absolute_path` remains the logical
`/etc/ssl/certs/ca-certificates.crt` name while `device_id`, `inode`, `mode`, `byte_length`, and
`sha256` describe the opened final regular file after the bounded resolution below. The existing
struct and `runtime-support` canonical hash already bind both facts; this correction adds no field,
schema version, manifest family, alternate trust-source name, or registry object.

struct RuntimeSupportDirectoryV1 {
    absolute_path: String, // exactly "/etc/codex"
    device_id: u64,
    inode: u64,
    mode: u32, // exactly 0o755
    owner_uid: u64, // exactly 0
    owner_gid: u64, // exactly 0
    ordered_entry_names: Vec<String>, // exactly []
}

enum ConfigProjectionArtifactRoleV1 {
    Codex0125,
    WorldEntryWrapper,
    ManagedGateway,
}

struct RuntimeArtifactAuthorityRefV1 {
    authority_store_id: String,
    manifest_id: String,
    manifest_revision: u64,
    manifest_entry_id: String,
    manifest_hash: String,
    entry_hash: String,
}

enum RuntimeArtifactProvenanceV1 {
    OfficialCodexRelease {
        version: String, // exactly "0.125.0"
        target_triple: String, // exactly "x86_64-unknown-linux-musl"
        archive_name: String,
        archive_url: String,
        archive_sha256: String,
        archive_entry_path: String,
        extracted_executable_sha256: String,
    },
    SubstrateSourceBuild {
        component: String, // "substrate-gateway" or "substrate-world-entry"
        source_commit: String,
        source_tree: String,
        cargo_lock_sha256: String,
        target_triple: String, // exactly "x86_64-unknown-linux-musl" for E3 V1
        profile: String,
        executable_sha256: String,
    },
}
```

This `CanonicalDirectoryV1` is a new public type owned by `config-projection`, not shell's
same-named private host-session-authority type. `physical_path` is the absolute, symlink-free path
resolved from an already-open `O_PATH|O_DIRECTORY|O_CLOEXEC|O_NOFOLLOW` descriptor; device and inode
are captured by `fstat` on that same descriptor. Capture rejects a non-directory, deleted object,
path/descriptor mismatch, mount crossing beneath an accepted root, zero device/inode, or a non-Linux
variant. Revalidation reopens descriptor-relatively beneath the held trusted parent and requires the
same path/device/inode. No lexical path alone is a substitute, and no crate imports, exports, or
converts shell's private type. `authority_store_id` is `cpa_<lowercase UUIDv7>`, `series_id` is
`cps_<lowercase UUIDv7>`, `record_id` is `cpr_<lowercase UUIDv7>`, and a projection consumer ID is
`cpc_<lowercase UUIDv7>`. Artifact manifest and entry IDs are `ram_<lowercase UUIDv7>` and
`rae_<lowercase UUIDv7>`. Revision begins at 1. IDs are allocated once by the accepted-home authority
store and reused only by exact retry.
Installer source-store and source-record IDs are `ias_<lowercase UUIDv7>` and
`iar_<lowercase UUIDv7>`; each installer stream starts at revision 1 and advances only by exact
prior revision plus one.

The sole Codex V1 provenance tuple is fixed by the baseline
`resolve_codex_runtime_install_spec_for_target_v1` and its rendered installer: version `0.125.0`,
target triple `x86_64-unknown-linux-musl`, archive name
`codex-x86_64-unknown-linux-musl.tar.gz`, URL
`https://github.com/openai/codex/releases/download/rust-v0.125.0/codex-x86_64-unknown-linux-musl.tar.gz`,
archive SHA-256 `4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001`,
archive entry path `codex-x86_64-unknown-linux-musl`, and extracted executable SHA-256
`86dc42ac5823f25233d6dc4ec5ff34693afd8c32ff2d17b54c5eb0d15bc7d902`. The entry-path and
executable digests are derived by listing and hashing the sole regular-file entry only after the
archive digest succeeds; they are stored in the installer source record and exact-matched through
accepted artifact authority and the projection. The baseline installer's fallback `find` behavior
is not admitted for E3 authority: a missing, additional, nested, renamed, non-regular, or differently
hashed archive member is `UnsupportedRuntimeVersion`.

`ConfigProjectionE2CapLinkV1` is an immutable, exact-value join to the already accepted
`E2MemberLaunchActivationCarrierV1`. Its activation and launch kind, commitment/cap references,
created/application revisions, snapshot reference/hash/revision, request and idempotency identities,
caller and target bindings, target world, and registry publication revision must be byte/value-equal
to that carrier. `commitment_subject` is its normalized launch/fork subject and must agree with the
carrier's retained child, bootstrap run, and optional source participant. E3 obtains these fields only
through E2's authenticated read capability; it neither copies E2 snapshot bytes into E3 authority nor
recomputes an E2 cap from current parent policy.

`DescriptorPinnedArtifactV1` is produced from an already-open, no-follow descriptor after checking
regular-file type, non-group/other-writable mode, expected owner, content length/hash, and a trusted
manifest entry that existed before projection validation. Owner execute must be set; setuid, setgid,
and sticky bits, POSIX ACLs beyond the mode
bits, `security.capability`, and every unrepresented executable-affecting xattr must be absent. Linux execution
uses that same open file description with
`execveat(fd, "", argv, envp, AT_EMPTY_PATH)`; re-opening `configured_absolute_path` after validation
is a TOCTOU violation. E3's wrapper is an ELF executable; scripts are unsupported on V2. Gateway,
wrapper, and Codex descriptors and every unrelated descriptor remain close-on-exec except during
their own immediate `execveat`.

The expected bytes do not come from running or hashing a caller-selected executable during
projection construction. E3-C implements the admitted installer source-store schemas, strict
validation and import, publication/recovery helpers, and rendering/native-source behavior. E3-D
consumes that landed machinery: its Linux provisioning integration builds and installs the actual
`substrate-world-entry` helper and E3-eligible `substrate-gateway`, descriptor-readback-validates both,
and only then invokes the Substrate publication helper to create the root-owned source-build record
and head. That record includes source commit, tree, `Cargo.lock` hash, exact
`x86_64-unknown-linux-musl` target, profile, installed-path digest, and physical file identity.
Source-build record creation requires a clean tracked tree whose computed commit/tree and
`Cargo.lock` digest equal the recorded build inputs; an uncommitted product build is not E3-eligible.
The world-deps Codex installer publishes a separate root-owned entry only after verifying
the official archive, extracting the uniquely named executable, hashing that extracted executable,
installing it, fsyncing file and directory, and readback-matching the installed descriptor. E3 imports
those entries into one accepted-home `TrustedRuntimeArtifactManifestV1` by authenticated descriptor,
never by path or self-reported `--version`; import is first-writer/CAS and exact retry only. The
provisioner uses the exact in-process static-ELF validation below against each installed artifact and
publishes its complete `E3RuntimeSupportManifestV1` inside the same root-owned/no-replace installer
source entry. Import copies that exact manifest into the accepted-home artifact entry, and the
descriptor-pinned projection copies it once more. The source-entry `entry_hash`, accepted-entry
`entry_hash`, and their enclosing record/manifest hashes therefore authenticate the support bytes;
every later import, projection, and wrapper recomputes the manifest from the same held executable and
support descriptors and requires exact canonical-byte equality at all three locations. The manifest
is described under persistence below. The baseline contains neither manifest publication
path, so both are explicit E3 implementation surfaces rather than claimed existing protection.

For `Codex0125`, provenance fixes `codex-cli 0.125.0`, target
`x86_64-unknown-linux-musl`, the pinned official archive SHA-256 above, its unique archive entry, and
the independently recorded extracted executable SHA-256. Archive equality alone cannot validate the
executable. `ManagedGateway` and `WorldEntryWrapper` must equal their manifest-pinned installed source
builds. E3's wrapper is a new small ELF helper built in the existing `world-service` package and
installed as `/usr/local/lib/substrate/e3/substrate-world-entry`; it is not the baseline generated
`world-entry.sh`. V1 keeps that generated compatibility wrapper unchanged. A different version,
target, archive entry, executable byte sequence, or source build requires a new series.

## Authoritative projection inputs and inventory V3

The source tree at the authority baseline has strict `AgentFileV2`, `AgentConfigV2`,
`AgentPlacementConfigV2`, and `PlacementProjectedInventoryEntryV2`, but a placement contains only
enablement, CLI/API settings, and `AgentCapabilitiesV1`. It contains no model, MCP-server, or feature
values. E3 therefore adds strict inventory V3 as the sole source of those values; it does not infer
them from Codex defaults, an ambient home, a provider response, or process environment.

```rust
struct AgentFileV3 {
    version: u32, // exactly 3
    id: String,
    config: AgentConfigV3,
    policy_overlay: Option<PolicyPatch>,
}

struct AgentConfigV3 {
    enabled: bool,
    kind: AgentConfigKind,
    protocol: Option<String>,
    placements: AgentPlacementsV3,
}

struct AgentPlacementsV3 {
    host: Option<AgentPlacementConfigV3>,
    world: Option<AgentPlacementConfigV3>,
}

struct AgentPlacementConfigV3 {
    enabled: bool,
    cli: Option<AgentCliConfigV1>,
    api: Option<AgentApiConfigV1>,
    capabilities: AgentCapabilitiesV1,
    runtime_projection: Option<AgentRuntimeProjectionInputV1>,
}

struct AgentRuntimeProjectionInputV1 {
    model: String,
    mcp_servers: Vec<LogicalMcpServerV1>,
    features: Vec<LogicalFeatureV1>,
}

struct PlacementProjectedInventoryEntryV3 {
    // Every PlacementProjectedInventoryEntryV2 field, unchanged.
    runtime_projection: Option<AgentRuntimeProjectionInputV1>,
    source: AgentInventorySourceMaterialV1,
}

struct AgentInventorySourceMaterialV1 {
    inventory_scope: String, // exactly "global" or "workspace"
    accepted_root: CanonicalDirectoryV1,
    relative_path: String,
    file_device_id: u64,
    file_inode: u64,
    byte_length: u64,
    raw_bytes_sha256: String,
    source_revision: String,
    source_hash: String,
}
```

Every V3 struct denies unknown fields. YAML duplicate mapping keys, aliases, anchors, merge keys,
non-UTF-8 input, multiple documents, tags, and non-string mapping keys are rejected before typed
deserialization. Existing V1 and V2 decoders, compatibility materialization, and non-E3 behavior
remain byte/behavior-frozen. An enabled `kind: cli`, `placement: world`, `runtime_family: codex`
candidate is E3-eligible only in V3, with `cli.mode`, `cli.binary`, and `runtime_projection` explicit.
For E3's fixed `cli:codex-world` backend, `model` is exactly `"codex"`, the landed
`GatewayBackendBinding.routed_model`; the gateway's existing provider mapping to
`codex-mini-latest` remains gateway policy and is not re-owned by E3. A future different routed model
requires new authority. `mcp_servers` and
`features` are required even when empty. Server IDs and feature names are unique and bytewise sorted;
all nested named-value lists are unique and bytewise sorted. No default exists for any of these three
projection fields. A V1/V2 candidate remains valid only on the named compatibility path and resolves
as `UnsupportedLegacyState` when asked to create E3 authority.

For the E3 Codex world placement, `cli.binary` is exactly
`/var/lib/substrate/world-deps/codex-runtime/bin/codex`, the manifest-recorded installed regular file,
not the baseline convenience symlink `/var/lib/substrate/world-deps/bin/codex`. The wrapper and
gateway configured paths are exactly `/usr/local/lib/substrate/e3/substrate-world-entry` and
`/usr/local/lib/substrate/e3/substrate-gateway`. No PATH lookup or symlink resolution participates in E3 artifact
selection.

Inventory discovery preserves the landed order: the authenticated accepted-home `agents/` directory
is the global root and `<workspace-root>/.substrate/agents/` is the workspace root; files within each
root are processed by raw filename bytes in ascending order. Exactly one file per logical agent ID is
permitted in a root. A workspace V3 file shadows the whole same-ID global V3 value before placement
projection, matching the whole-logical-agent shadow posture already used by V2; partial field merge is
forbidden. The selected world placement realizes `<logical-id>-world` and derives the backend ID by
the existing `derive_agent_backend_id` algorithm. For E3's exact Codex descriptor this is
`codex-world` / `cli:codex-world`. `AgentCapabilitiesV1` becomes the bytewise-sorted list of field
names whose values are `true`; false fields are absent. The baseline `config/agents/codex.yaml` is V2,
so later E3 implementation must atomically migrate that one descriptor to V3 and supply explicit
`model: codex` plus explicit empty MCP/feature lists unless the later admission authorizes literal
nonempty values. E3 does not infer a changing upstream model name.

E3 V1 authenticates and persists provenance only for the selected contributing inventory source. A
valid workspace descriptor that shadows a global descriptor replaces it before projection
construction, so the shadowed global descriptor is not a contributor. Its bytes are not copied into
the projection record, any hash preimage, the selected source revision or identity, or the retention
set, and V1 makes no durable shadow-history reconstruction promise. Discovery still validates all
encountered source material before selection: malformed or ambiguous global or workspace input fails
closed even if a later valid descriptor would otherwise shadow it. “Not retained after valid shadow
selection” is never permission to ignore invalid input. Any future durable shadow-history feature
requires separate versioned authority and cannot be inferred into V1; no shadow-provenance field or
schema is part of this contract.

The parser opens each root through its already authenticated directory descriptor and opens each
relative single-component `*.yaml` name with no-follow semantics. `raw_bytes_sha256` is ordinary
SHA-256 of the exact file bytes. `source_revision` is exactly
`aisr1_<raw_bytes_sha256>`. `source_hash` is the canonical domain hash of
`{"domain":"substrate.e3.agent-inventory-source.v1","source":<source material with source_hash omitted>}`.
The logical `AgentInventory` source copies that revision and hash and uses the projection store ID as
`authority_store_id`; resolution must still readback-match the complete `AgentInventorySourceMaterialV1`.

The new shell-private `resolve_e3_effective_config_source_v1` performs one descriptor-pinned
resolution transaction before projection construction. It does not call the landed resolver and
then reread named paths. It reuses the landed parse, merge, precedence, explain, environment, and
protected-exclude semantics over one retained input snapshot: accepted-home `config.yaml` is opened
once through new `OpenedBootstrapHomeV1::open_e3_config_source`; the returned sealed
`OpenedBootstrapConfigSourceV1` retains the exact trusted root and regular-file descriptors together
with the sole byte vector read from that file. That same borrowed byte slice is both parsed and
hashed. An absent `config.yaml` yields the sealed absent posture tied to the still-held trusted
root and supplies the default empty global patch. The workspace root selected by the landed
root-selection order must equal the candidate projection's
already captured physical workspace root. E3 opens that root and its `.substrate` directory with
component-wise no-follow descriptor traversal, proves `workspace.yaml` is one regular file and that
`workspace.disabled` and legacy `settings.yaml` are absent, then reads `workspace.yaml` once through
the held regular-file descriptor. The exact returned workspace byte vector is likewise both parsed
and hashed. The transaction snapshots environment overrides exactly once and supplies an exactly
empty `CliConfigOverrides`.

The transaction passes the patches parsed from those same byte vectors to the existing
`resolve_effective_from_layers` once with explain enabled. It then constructs each patch-origin
location solely from the retained source that supplied that returned value; an explain pathname is
an equality check against that source, never permission to reopen it. Descriptor/root revalidation,
unexpected entry kind, selected-root mismatch, invalid UTF-8/YAML, read failure, or explain/source
mismatch fails before publication. Concurrent replacement after a descriptor is acquired cannot
substitute bytes: one in-memory byte vector is the input to both parsing and SHA-256. There is no
second semantic resolution, pathname readback, or A-to-B-to-A comparison window.

`OpenedBootstrapConfigSourceV1` is non-cloneable, non-serializable, and constructible only by that
facade method. Its private fields make the presence posture non-forgeable: a non-null byte vector is
paired with the exact retained `TrustedFile` used for its one read, while two nulls retain the trusted
root that proved absence. Any other pair is impossible. Its only callable operations are
`physical_root_path`, `source_bytes`, `validate_e3_public_root_identity`, and `revalidate`.
Config-model opens the reported already-physical root path as a no-follow directory, captures the
new public config-projection `CanonicalDirectoryV1` from that descriptor, and passes its
path/device/inode tuple to `validate_e3_public_root_identity`. The original trusted-root descriptor
remains held throughout, so a replacement cannot make the two identities equal. No raw descriptor,
`TrustedFile`, shell-private `CanonicalDirectoryV1`, or conversion of that private type crosses the
facade. Drop closes the retained file; no API can reread it or detach its bytes from its descriptor.

E3 extracts only the eight nonsecret values in `E3EffectiveConfigInputV1` and their exact ordered
explain origins from that returned snapshot; it never persists the private raw vectors, an unrelated
config field, or a secret. That strict object is `EffectiveSubstrateConfigSourceV1`; the source
revision is `ecsr1_<SHA-256 of the canonical object with source_revision and source_hash omitted>`
and its source hash uses domain `substrate.e3.effective-substrate-config-source.v1`. This source
contributes only existing enablement, placement, CLI-mode, and managed-gateway selection values; it
cannot supply or default model, MCP, or feature values. Both immutable source objects are persisted
before the subject binding and are exact-readback dependencies of every revision.

## Exact projection content

```rust
struct LogicalAgentConfigProjectionV1 {
    projection_hash: String,
    sources: Vec<LogicalConfigSourceRefV1>,
    logical_agent_id: String,
    placement: String, // exactly "world"
    realized_agent_id: String,
    backend_id: String,
    kind: String, // exactly "cli"
    protocol: String,
    execution_scope: String, // exactly "world"
    cli_mode: String,
    runtime_family: String, // exactly "codex"
    capabilities: Vec<String>,
    requested_model: String,
    requested_mcp_servers: Vec<LogicalMcpServerV1>,
    requested_features: Vec<LogicalFeatureV1>,
}

enum LogicalConfigSourceRefV1 {
    EffectiveSubstrateConfig {
        authority_store_id: String,
        source_revision: String,
        source_hash: String,
    },
    AgentInventory {
        authority_store_id: String,
        inventory_scope: String, // "global" or "workspace"
        source_revision: String,
        source_hash: String,
    },
}

struct LogicalMcpServerV1 {
    server_id: String,
    transport: LogicalMcpTransportV1,
    enabled: bool,
}

enum LogicalMcpTransportV1 {
    Stdio { command: String, args: Vec<String>, nonsecret_env: Vec<NamedValueV1> },
    StreamableHttp { url: String, nonsecret_headers: Vec<NamedValueV1> },
}

struct LogicalFeatureV1 { name: String, enabled: bool }
struct NamedValueV1 { name: String, value: String }

struct EffectiveAgentConfigProjectionV1 {
    projection_hash: String,
    logical_projection_hash: String,
    accepted_policy: ConfigProjectionE2CapLinkV1,
    capabilities: Vec<String>,
    model: String,
    provider: EffectiveManagedProviderV1,
    mcp_servers: Vec<EffectiveMcpServerV1>,
    features: Vec<LogicalFeatureV1>,
    environment: EffectiveEnvironmentV1,
    workspace_overlay: WorkspaceOverlayPostureV1,
}

struct EffectiveManagedProviderV1 {
    provider_id: String, // exactly "substrate-managed-gateway"
    wire_api: String, // exactly "responses"
    requires_openai_auth: bool, // exactly false
    supports_websockets: bool, // exactly false in E3
    gateway_intent_ref: ManagedGatewayActivationIntentRefV1,
}

struct EffectiveMcpServerV1 {
    server_id: String,
    transport: LogicalMcpTransportV1,
    enabled: bool,
}

struct EffectiveEnvironmentV1 {
    inherited_names: Vec<String>,
    set: Vec<NamedValueV1>,
    remove: Vec<String>,
}

enum WorkspaceOverlayPostureV1 {
    Disabled,
}
```

The logical projection contains exactly the listed agent/config inventory values. It contains no
host file path, native Codex table, credential locator, environment-secret name, policy object, or
gateway endpoint. `sources` is nonempty and ordered lowest-to-highest precedence; duplicate source
kind/scope pairs conflict. Capabilities, MCP servers, features, named values, inherited names, and
removals are sorted bytewise by their identifier and contain no duplicates.

For E3, `sources` is exactly the one `EffectiveSubstrateConfig` ref followed by exactly one
`AgentInventory` ref for the selected world placement: `workspace` when a same-ID workspace V3 file
shadows global, otherwise `global`. A shadowed global file is not a contributor and is not hashed into
the logical projection, record, revision, source identity, or retention set. All discovered material
must nevertheless pass the strict ambiguity and malformed-input checks before this selected-only
provenance decision. Missing either required source, or two candidates in the selected scope, is
`Missing` or `Conflict` before subject publication.

The effective projection is deterministic and is not a new policy-composition surface.
`accepted_policy` must be byte-equal to `identity.immutable_launch_cap`, but the landed E2 snapshot
contains only network and world-filesystem policy and E2 narrowing is limited to its named
`RestrictedWorldFs` patch. It therefore supplies no authority to filter agent capabilities, features,
or MCP servers. For V1, `effective.capabilities` is byte-for-byte equal to
`logical.capabilities`, `effective.model` equals `logical.requested_model`, `effective.features`
equals `logical.requested_features`, and `effective.mcp_servers` is the field-for-field mapping of
`logical.requested_mcp_servers` with identical order, transport, and `enabled` values. No
`policy_decision_ref` exists. Any implementation that removes, disables, adds, or rewrites one of
those values is hash-invalid. The E2 link remains an exact immutable launch-capability join and never
substitutes for E1/E2 enforcement.

The controlling E3 V1 admission is narrower still: both logical and effective MCP/feature lists must
be exactly `[]`. A nonempty list is `UnsupportedPolicySurface`, even when
`capabilities.mcp_client = true`; admitting nonempty MCP or feature projection requires later explicit
authority that names its policy source. Secret-bearing MCP material is always malformed.
`workspace_overlay` is exactly `Disabled`; E4 owns any later host-visible workspace synchronization or
reconciliation.

For the controlling E3 Codex path, `logical.requested_model` and `effective.model` are both exactly
`"codex"`; `effective.provider.provider_id` is `"substrate-managed-gateway"`; and the intent's backend
is exactly `cli:codex-world`. E3 cannot rewrite the model to the gateway's existing
`codex-mini-latest` provider target, mutate that mapping, or accept an arbitrary model that merely
elicits a successful response. The effective MCP/features lists are exactly equal to their logical
lists and, for the controlling V1 admission, are exactly `[]`. The baseline Codex world placement also
has `capabilities.mcp_client = false`; its E3 migration therefore uses exact empty MCP input and
output.

`EffectiveEnvironmentV1.inherited_names` is exactly `[]`. E3 builds a clean environment. Its sorted
`set` contains only `HOME`, `CODEX_HOME`, `CODEX_SQLITE_HOME`, `TMPDIR`, `PATH`, `LANG`, `LC_ALL`, and
`RUST_LOG`,
with exactly these values, where `<root>` is `native.root.guest_absolute_path`:

```text
CODEX_HOME=<root>/codex-home
CODEX_SQLITE_HOME=<root>/state/sqlite
HOME=<root>/home
LANG=C.UTF-8
LC_ALL=C.UTF-8
PATH=/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
RUST_LOG=error
TMPDIR=<root>/tmp
```

`TERM` may be added only for a PTY carrier, which E3's non-PTY member dispatch does not use. `remove`
is exactly the bytewise-sorted vector
`[ANTHROPIC_API_KEY,CODEX_API_KEY,CODEX_BINARY,CODEX_OSS_BASE_URL,CODEX_OSS_PORT,OPENAI_ACCESS_TOKEN,
OPENAI_API_KEY,OPENAI_BASE_URL,OPENAI_ORGANIZATION,OPENAI_PROJECT,
SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD,SUBSTRATE_E3_NATIVE_REALIZATION_FD,
SUBSTRATE_E3_NATIVE_SOURCE_FD,SUBSTRATE_E3_SYSTEM_EMPTY_FD,SUBSTRATE_E3_WORLD_FS_INPUT_FD,
SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME,SUBSTRATE_LLM_AUTH_BUNDLE_FD,
SUBSTRATE_WORLD_ENTRY_BINARY,SUBSTRATE_WORLD_ENTRY_BINARY_FD,
SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_FD,SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH,
SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD,SUBSTRATE_WORLD_ENTRY_REQUIRE_CGROUP_ATTACH,
SUBSTRATE_WORLD_ENTRY_ROLE,SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD,SUBSTRATE_WORLD_ENTRY_USERNS_FD,
SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD]`. The FD-named keys are numeric nonsecret descriptor pointers
that exist only in the wrapper launch environment described below.
Unknown inherited environment is absent because inheritance is empty, not because the denylist is
assumed exhaustive.

```rust
struct NativeAgentConfigProjectionV1 {
    projection_hash: String,
    renderer: NativeRendererIdentityV1,
    root: NativeProjectionRootV1,
    files: Vec<NativeProjectedFileV1>,
    directories: Vec<NativeProjectedDirectoryV1>,
    environment: EffectiveEnvironmentV1,
    invocation: CodexNativeInvocationV1,
    cwd: CanonicalDirectoryV1,
    ambient_closure: CodexAmbientConfigClosureV1,
}

struct NativeRendererIdentityV1 {
    renderer_id: String, // exactly "substrate.codex.config-renderer"
    renderer_schema_version: u32, // exactly 1
    codex_version: String, // exactly "0.125.0"
}

struct NativeProjectionRootV1 {
    root_id: String,
    authority_relative_path: String,
    guest_absolute_path: String,
    owner_uid: u64,
    owner_gid: u64,
    directory_mode: u32, // exactly 0o700
}

struct CodexNativeInvocationV1 {
    wrapper_argv: Vec<String>,
    initial_codex_argv: Vec<String>,
    resume_codex_argv_prefix: Vec<String>,
    prompt_delivery: String, // exactly "stdin-lf-eof"
    output_last_message_directory: String,
    output_last_message_name_domain: String,
    forbidden_arguments: Vec<String>,
}

// Strict, bounded, nonsecret input read by the descriptor-pinned wrapper.
struct E3CodexLaunchPlanV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    fence_id: String,
    closed_projection_ref: ConfigProjectionRefV1,
    projection_identity_hash: String,
    native_projection_hash: String,
    turn: E3CodexLaunchTurnV1,
    codex_argv: Vec<String>,
    final_environment: EffectiveEnvironmentV1,
    required_absent_codex_home_entries: Vec<String>,
    output_last_message: E3CodexOutputFileV1,
    workspace: CanonicalDirectoryV1,
    native_source: CanonicalDirectoryV1,
    native_realization: CanonicalDirectoryV1,
    system_empty: CanonicalDirectoryV1,
    stdin_fd: u32, // exactly 0
    stdout_fd: u32, // exactly 1
    stderr_fd: u32, // exactly 2
    plan_hash: String,
}

enum E3CodexLaunchTurnV1 {
    Initial { bootstrap_run_id: String },
    Resume { turn_id: String, codex_session_id: String },
}

struct E3CodexOutputFileV1 {
    relative_path: String,
    guest_absolute_path: String,
    device_id: u64,
    inode: u64,
    mode: u32, // exactly 0o600
    initial_byte_length: u64, // exactly 0
    initial_sha256: String, // SHA-256 of empty bytes
}

struct NativeProjectedDirectoryV1 { relative_path: String, mode: u32 }

struct NativeProjectedFileV1 {
    role: NativeProjectedFileRoleV1,
    relative_path: String,
    mode: u32, // exactly 0o600
    bytes_base64: String,
    byte_length: u64,
    sha256: String,
}

enum NativeProjectedFileRoleV1 { CodexConfigToml }

struct CodexAmbientConfigClosureV1 {
    loader_source: CodexLoaderSourceIdentityV1,
    allowed_enabled_layers: Vec<String>,
    inputs: Vec<CodexLoaderInputAttestationV1>,
    forbidden_cli_overrides: Vec<String>,
    validated_loader_input_fingerprint: String,
}

struct CodexLoaderSourceIdentityV1 {
    codex_version: String, // exactly "0.125.0"
    upstream_tag: String, // exactly "rust-v0.125.0"
    config_loader_source_sha256: String,
    layer_io_source_sha256: String,
    loader_model_source_sha256: String,
    exec_source_sha256: String,
    cloud_requirements_source_sha256: String,
    auth_storage_source_sha256: String,
    config_types_source_sha256: String,
    validator_schema_version: u32, // exactly 1
}

struct CodexLoaderInputAttestationV1 {
    layer: String,
    locator: String,
    disposition: CodexLoaderInputDispositionV1,
    directory: CanonicalDirectoryV1,
    relative_path: String,
    device_id: Option<u64>,
    inode: Option<u64>,
    byte_length: Option<u64>,
    sha256: Option<String>,
}

enum CodexLoaderInputDispositionV1 {
    Projected,
    ApprovedMandatory,
    ProvenAbsent,
    DisabledByTrust,
    DisabledByEphemeralCredentialStoreAndProvenAbsent,
    DisabledByEmptyMcpSetAndProvenAbsent,
    DisabledByNoEphemeralAuthAndProvenAbsent,
}

// Process-local, non-persistent setup evidence returned by the pinned wrapper.
struct CodexSetupReadyAttestationV1 {
    schema_version: u32, // exactly 1
    series_id: String,
    fence_id: String,
    wrapper_pid: u32,
    wrapper_pid_start_time_ticks: u64,
    mount_namespace_inode: u64,
    masked_system_directory: CanonicalDirectoryV1,
    native_root: CanonicalDirectoryV1,
    codex_launch_plan_hash: String,
    validated_loader_input_fingerprint: String,
    child_security: E3ChildSecurityAttestationV1,
    attestation_hash: String,
}

// Strict, bounded, nonsecret pipe input. It is not durable authority.
struct E3WorldFsEnforcementInputV1 {
    schema_version: u32, // exactly 1
    child_role: E3IsolatedChildRoleV1,
    projection_identity_hash: String,
    policy_authority: E3PolicyAuthoritySourceV1,
    immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1,
    policy_snapshot_bytes_base64: String,
    policy_snapshot_byte_length: u64,
    policy_snapshot_ref: PolicyRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    expected_process_cgroup: CanonicalCgroupIdentityV1,
    kernel_boot_id: String,
    user_namespace_requirement: E3UserNamespaceRequirementV1,
    target_uid: u64,
    target_gid: u64,
    immutable_config_source: Option<CanonicalDirectoryV1>,
    private_realization: Option<CanonicalDirectoryV1>,
    codex_launch_plan_hash: Option<String>, // non-null exactly for Codex
    executable_artifact: DescriptorPinnedArtifactV1,
    denied_control_probe_targets: Vec<E3DeniedControlProbeTargetV1>,
    support_policy_version: u32, // exactly 1
    enforcement_input_hash: String,
}

enum E3IsolatedChildRoleV1 { Codex, ManagedGateway, ManagedGatewayReadinessProbe }

struct E3LinuxIdMapExtentV1 {
    inside_id: u64,
    outside_id: u64,
    length: u64, // exactly 1
}

struct E3UserNamespaceRequirementV1 {
    trusted_service_uid: u64, // exactly the installed primary service UID, currently 0
    parent_namespace_device_id: u64,
    parent_namespace_inode: u64,
    uid_map: E3LinuxIdMapExtentV1,
    gid_map: E3LinuxIdMapExtentV1,
}

struct E3UserNamespaceAttestationV1 {
    namespace_device_id: u64,
    namespace_inode: u64,
    owner_uid: u64,
    parent_namespace_device_id: u64,
    parent_namespace_inode: u64,
    uid_map: E3LinuxIdMapExtentV1,
    gid_map: E3LinuxIdMapExtentV1,
}

enum E3PolicyAuthoritySourceV1 {
    InitialLaunch { e2_activation_id: String, commitment_ref: DispatchPolicyCommitmentRefCarrierV1 },
    RetainedTurn { subject: RetainedTurnPolicyCommitmentSubjectV1 },
}

struct E3DeniedControlProbeTargetV1 {
    binding: E3DeniedControlProbeTargetBindingV1,
    target_hash: String,
}

enum E3DeniedControlProbeTargetBindingV1 {
    AcceptedHomeRegistry { directory: CanonicalDirectoryV1 },
    SiblingNativeRealization { directory: CanonicalDirectoryV1 },
    CgroupControl {
        cgroup: CanonicalCgroupIdentityV1,
        control_file: String, // exactly "cgroup.procs"
    },
    NftablesControl { network_namespace_inode: u64 },
    WorldServiceState { directory: CanonicalDirectoryV1 },
    OtherRolePrivateRoot { directory: CanonicalDirectoryV1 },
    OtherRoleProcessState {
        pid: u32,
        pid_start_time_ticks: u64,
        procfs_mount_device_id: u64,
        procfs_mount_inode: u64,
    },
}

// Transient exact plan derived by the trusted wrapper; it is never E2 authority.
struct E3DerivedWorldFsEnforcementPlanV1 {
    e2_plan_hash: String,
    e2_discover_paths: Vec<String>,
    e2_execute_paths: Vec<String>,
    e2_read_paths: Vec<String>,
    e2_write_paths: Vec<String>,
    e3_support_discover_paths: Vec<String>,
    e3_support_execute_paths: Vec<String>,
    e3_support_read_paths: Vec<String>,
    e3_support_write_paths: Vec<String>,
    derived_support_ruleset_hash: String,
    role_narrowing_ruleset_hash: String,
    effective_landlock_hash: String,
}

struct E3ChildSecurityAttestationV1 {
    schema_version: u32, // exactly 1
    child_role: E3IsolatedChildRoleV1,
    projection_identity_hash: String,
    pid: u32,
    pid_start_time_ticks: u64,
    real_uid: u64,
    effective_uid: u64,
    saved_uid: u64,
    filesystem_uid: u64,
    real_gid: u64,
    effective_gid: u64,
    saved_gid: u64,
    filesystem_gid: u64,
    supplementary_group_count: u32, // exactly 0
    cap_inheritable: String, // exactly "0000000000000000"
    cap_permitted: String, // exactly "0000000000000000"
    cap_effective: String, // exactly "0000000000000000"
    cap_bounding: String, // exactly "0000000000000000"
    cap_ambient: String, // exactly "0000000000000000"
    cap_last_cap: u32, // must be at most 63 and every value through it was dropped
    no_new_privs: bool, // exactly true
    dumpable: u32, // exactly 0 in the pre-final-exec wrapper/probe
    tracer_pid: u32, // 0 for an untraced role; otherwise the exact trusted service tracer PID
    kernel_boot_id: String,
    user_namespace: E3UserNamespaceAttestationV1,
    seccomp_mode: u32, // exactly 2, filter
    landlock_abi: u32,
    e2_enforcement_plan_hash: String,
    derived_support_ruleset_hash: String,
    role_narrowing_ruleset_hash: String,
    effective_landlock_hash: String,
    policy_snapshot_ref: PolicyRefV1,
    policy_snapshot_hash: String,
    policy_snapshot_revision: String,
    enforcement_input_hash: String,
    denied_control_probe_hash: String,
    attestation_hash: String,
}

struct E3DeniedControlProbeV1 {
    target: E3DeniedControlProbeTargetV1,
    operation: String,
    result_errno: i32, // exactly EACCES or EPERM as fixed per target below
}

struct NativeProjectionSourceManifestV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    fence_id: String,
    source_root: CanonicalDirectoryV1,
    native_projection_hash: String,
    ordered_file_hashes: Vec<String>,
    created_at: Timestamp,
    manifest_hash: String,
}

struct NativeProjectionRealizationManifestV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    fence_id: String,
    source_manifest_hash: String,
    realization_root: CanonicalDirectoryV1,
    owner_uid: u64,
    owner_gid: u64,
    immutable_config_mount_read_only: bool, // exactly true
    created_at: Timestamp,
    manifest_hash: String,
}
```

`DispatchPolicyCommitmentRefCarrierV1`, `PolicyRefV1`,
`RetainedTurnPolicyCommitmentSubjectV1`, `E2MemberLaunchKindV1`, and `WorldBindingRefV1` above are
the exact public types from `transport-api-types` at the authority baseline. E3 imports and embeds
those types directly. It does not name shell's private `DispatchPolicyCommitmentRefV1`, invent a
`PolicySnapshotRefV1`, mirror either shape, or convert through a hash-only surrogate. Consequently
the persisted projection link, the V2 carrier, the initial activation, and the transient resumed-turn
input compare the imported values with derived `PartialEq` field equality and their existing strict
validation before any E3 hash is accepted.

Crate ownership is acyclic. `transport-api-types` remains the lower leaf and owns only the E3 wire
types `ConfigProjectionRefV1`, `ConfigProjectionAuthoringInputRefV1`, `InWorldGatewayRefV1`,
`ManagedGatewayActivationIntentRefV1`, `ConfigProjectionActivationCarrierV1`,
`E3ConfigProjectionPrepareRequestV1`, `E3ConfigProjectionPrepareResponseV1`,
`E3ConfigProjectionCancelRequestV1`, `E3ConfigProjectionCancelResponseV1`,
`MemberDispatchRequestV2`, and their strict private decode forms. Those types contain strings,
integers, and other `transport-api-types` values only; in particular the activation carrier does not
embed `ConfigProjectionIdentityV1`, an ACK, a directory identity, or any operational capability.
The new `config-projection` crate depends downward on `transport-api-types`, owns
`CanonicalDirectoryV1`, `ConfigProjectionIdentityV1`, `AgentConfigProjectionRecordV1`, all remaining
projection/gateway persistence schemas and operational services, and consumes the wire refs. Shell,
world-service, and gateway depend on both crates as needed. `transport-api-types` never depends on or
re-exports `config-projection`, so no Cargo edge points back upward.
For the E3-B HSA transaction bridge specifically, `config-projection` also owns the object-safe
`ConfigProjectionHsaAuthorityV1` consumer trait described below, shell depends on and implements that
trait, and world-service later depends on shell only to construct the implementation from the sealed
installed accepted-home authority. `config-projection` does not depend on shell. Thus the normal
edges are `config-projection -> transport-api-types`, `shell -> config-projection`, and
`world-service -> {shell, config-projection}`; gateway's later edge remains
`gateway -> config-projection`. The existing shell-to-world-service test-only dev dependency remains
test-only and creates no reverse normal edge. The service's operational methods accept and validate
only their own typed nonsecret observations and this one narrow capability; they publish/resolve
projection authority but never instantiate a world-service manager, spawn a process, read a
transport body, or own a secret/listener capability. World-service composes those downward calls
with its gateway and child lifecycle.

`root_id` is `cnr_<lowercase UUIDv7>`. `authority_relative_path` is exactly
`authority-v1/agent-config-projection-v1/native-sources/<series-id>/<fence-id>` beneath accepted
home, and `guest_absolute_path` is exactly
`/run/substrate/member-config/<series-id>/<fence-id>`. Neither path is caller-selectable.
`owner_uid` and `owner_gid` are exactly the intended UID and primary GID authenticated by the
installed accepted-home bootstrap record; name-service lookup during request handling is forbidden.

The native file set for Codex 0.125 is exactly one `codex-home/config.toml`; there is no `auth.json`,
`.credentials.json`, profile file, copied `.codex`, `.mcp.json`, secret-bearing environment file, or
workspace file. The directory set is exactly `.`, `system-empty`, `home`, `codex-home`, `state`,
`state/sqlite`, `state/log`, `tmp`, and `tmp/output-last-message`, all mode `0700`. Runtime-created state stays beneath this
worker-local root and cannot become projection authority; the three forbidden credential/cloud names
below must also remain absent at every per-exec and post-turn wall. `directories` uses exactly that order;
`files` contains its sole entry; no filesystem enumeration order is hashed. `cwd` equals
`identity.workspace_root`.

The record's embedded native bytes are the semantic authority. Before the dormant record is
published, the registry also realizes those bytes once into the immutable accepted-home source
directory named by `authority_relative_path`. That directory contains exactly
`source-manifest.json`, `codex-home/config.toml`, and an empty `system-empty/`; it contains no
`home`, `state`, `tmp`, log, socket, or process-created file. The source directory and its children
are descriptor-created beneath the accepted-home root, owned by the authenticated UID/GID, mode
`0700` for directories and `0600` for the config, and source-manifest/readback hashes must equal the
record before the projection head can advance.

Publication constructs a same-parent
`.e3-native-source-tmp.<lowercase-uuidv7>` directory, creates every component with `mkdirat`, writes
the config and canonical source manifest with create-exclusive/no-follow semantics, `fsync`s both
files and every directory bottom-up, then installs the complete directory with
`renameat2(RENAME_NOREPLACE)` and `fsync`s its parent. An existing final directory is accepted only
when every descriptor identity, owner/mode, entry name, file byte, native hash, and manifest byte is
exactly equal. A truncated tree, unrecognized entry, symlink, unequal retry, or multiple temporary
contender is `PartialPublication` or `Conflict`; recovery under the projection root lock either
finishes one complete equal no-replace rename or removes a temporary whose exact final already
exists, with the same readback and fsync wall. It never repairs individual files in place.

The `/run` path is a separate rebuildable per-fence realization, never authority. World-service
creates `.e3-native-realization-tmp.<lowercase-uuidv7>` beneath the fixed root-owned mode-`0711`
`/run/substrate/member-config`, then descriptor-relatively creates the exact projected directory set,
an `immutable/codex-home/config.toml` copy, a `codex-home/config.toml` mount target, and
`realization-manifest.json`. All fence descendants are the authenticated UID/GID and mode `0700`,
except the two config regular files and manifest are mode `0600`. It copies only from the held
accepted-home source descriptor, verifies source and destination bytes/hashes, fsyncs files and every
directory, installs the complete tree by no-replace rename, fsyncs the fixed parent, and exact-reads
back `NativeProjectionRealizationManifestV1`. Mutable `home`, `codex-home`, `state`, `state/sqlite`,
`state/log`, `tmp`, and `tmp/output-last-message` are empty at first publication and never copied
back to accepted-home authority.

Linux provisioning creates the permanent mount target `/etc/codex` before any E3 manifest or service
startup. It opens `/etc` as a root-owned, non-group/other-writable directory, creates only the single
`codex` component with descriptor-relative `mkdirat(0755)` when absent, reopens it with
`O_PATH|O_DIRECTORY|O_CLOEXEC|O_NOFOLLOW`, and requires root:root ownership, mode `0755`, the same
filesystem/mount relation expected beneath `/etc`, and the exact empty entry set. It then `fsync`s
the new directory and `/etc`. A pre-existing exact empty directory is accepted after the same
descriptor validation; a symlink, mount, non-directory, nonempty directory, wrong metadata, or
identity change is `UnsupportedSecurityPosture` without repair. The source installer entry,
accepted artifact entry, and each projection copy the resulting
`RuntimeSupportDirectoryV1` byte-for-byte, so the target's device/inode and empty-set claim are
authenticated before the service starts. It is retained permanently and never removed by E3 cleanup.
This is the one bounded shared-root provisioning effect; per-child namespace setup never creates or
removes a shared `/etc` object.

Inside each private child mount namespace, before privilege descent, the pinned wrapper bind-mounts
the realization's `immutable/codex-home/config.toml` onto `codex-home/config.toml` and remounts that
single file read-only; it likewise revalidates the manifest-pinned permanent `/etc/codex` target,
then uses the exact empty `system-empty` directory for that target's mask. The parent validates both
mounts through `/proc/<pinned-pid>/root` against its held source, target, and
realization descriptors. Same-UID sibling access is denied by the exact per-child Landlock rules
below, not by mode bits alone. An existing realization can be adopted only by the retained
`ActiveE3ConfigProjectionRuntimeV1` that owns the matching held descriptor and live consumer lease.
Before any child release, exact immutable bytes/mounts and mutable directory identities/modes are
revalidated; mutable contents are not authority and cannot change a projection hash. A crash before
any child could own the root may complete or discard the temporary deterministically. Once a child
could have owned it, ambiguous recovery kills the bound member cgroup, revokes the gateway boundary,
and never reuses that root for a new fence. Cleanup may delete only the descriptor-revalidated `/run`
realization after terminal-child proof, boundary revocation, and lease release; accepted-home source
bytes remain retained.

For every Codex child, `invocation.wrapper_argv` is exactly `["substrate-world-entry"]`; the
descriptor-pinned wrapper receives no flag or positional argument, and role/FD selection remains the
strict environment ABI above. For the initial retained-member turn, `initial_codex_argv` is exactly
`["codex","--dangerously-bypass-approvals-and-sandbox","exec","--color","never","--skip-git-repo-check",
"--json","--output-last-message","<initial-output-path>"]` after replacing the angle-bracketed
metavariable with the one computed path below; the stored vector contains the path, not placeholder
text. `resume_codex_argv_prefix` is exactly
`["codex","--dangerously-bypass-approvals-and-sandbox","exec","--color","never",
"--skip-git-repo-check","--json","--output-last-message"]`, before appending the computed per-turn
output path, `"resume"`, the exact recorded session ID, and `"-"`. In both complete arrays `argv[0]`
is the literal display string `codex`; neither the wrapper path nor the configured Codex path appears
in Codex argv. The first option after `argv[0]` is exactly
`"--dangerously-bypass-approvals-and-sandbox"`. The external-sandbox flag is the Codex 0.125
external-sandbox posture: it delegates process/filesystem/network enforcement to the already-
validated Substrate world-entry wrapper and does not bypass E1/E2 or broker/world-service policy.
The pinned [Codex 0.125 exec CLI](https://github.com/openai/codex/blob/rust-v0.125.0/codex-rs/exec/src/cli.rs)
defines an omitted initial prompt or literal `-` as stdin input. E3 deliberately uses the omitted
prompt for initial exec and the explicit final `-` for resume; no argv element is implicit beyond
those two source-defined cases.
`output_last_message_directory` is exactly `<root>/tmp/output-last-message`, created mode `0700`,
and `output_last_message_name_domain` is exactly
`substrate.e3.codex-output-last-message-name.v1`. For a validated turn ID, the adapter computes
`olm_<lowercase SHA-256 of {"domain":<domain>,"fence_id":<fence>,"series_id":<series>,"turn_id":<id>}>.txt`,
opens that single component under the held directory descriptor with create-exclusive/no-follow mode
`0600`, and substitutes its fixed guest path for `<initial-output-path>`. The initial turn ID is the
E2-bound bootstrap run ID; later turn IDs must already be exact-bound to the retained participant.
The path is never accepted from a caller, ambient `TMPDIR`, or UAA. Exact retry may reopen the
same empty regular file only after proving no child ever owned the turn. Once a child could have
written it, recovery consumes the existing turn evidence and never re-executes that turn; nonempty,
wrong-metadata, or ambiguously owned files conflict. The prompt is written as its exact UTF-8 bytes followed by
one LF, then stdin is closed. The sorted `forbidden_arguments` is exactly
`[--config,--ignore-rules,--ignore-user-config,--model,--oss,--profile,-c]`; no provider, model, feature, MCP,
header, credential, host path, or extra config override may enter argv. Resumed turns use the same
fixed flags through an output path computed by the same algorithm, followed by `resume`, exactly one
validated recorded session ID, then `-` for the stdin prompt; `resume_codex_argv_prefix` stores the fixed
flags through `--output-last-message` and is renderer-hash input. Any argv change requires a new
renderer version and projection revision before use.

This is an E3-only local launch seam, not a capability attributed to UAA 0.3.7. External
`unified-agent-api-codex` 0.3.7 remains unchanged and is not claimed to provide descriptor-pinned
execution or confined output-path construction. Exact source shows
that `unified-agent-api-codex` 0.3.7 calls `std::env::temp_dir()` when
`ExecStreamRequest.output_last_message` is absent and uses `Command::new(binary_path)`; the higher
level `unified-agent-api` 0.3.7 Codex adapter supplies `output_last_message: None`. Those behaviors
cannot satisfy native-root confinement or descriptor-pinned execution. E3 therefore adds
`E3Codex0125LaunchAdapterV1` in world-service. For V2 only, the strict launch admission calls
`PromptFulfillmentBridge::for_e3_codex` with the sealed published projection capability; V1 and
non-Codex dispatch continue through the existing `PromptFulfillmentBridge::for_member_backend` and
`GatewayAdapterRuntime`/UAA path unchanged. The local adapter preserves the existing
`AgentWrapperRunControl` event/cancel surface and Codex JSONL normalization but owns argv assembly,
preopened output-file injection, stdin/stdout/stderr pipes, pidfd, cancellation, and exec.
E3 neither creates nor patches a local `crates/codex` replacement; this division is clarification of
existing ownership, not a new product primitive.

The adapter uses a two-stage wrapper protocol plus a pre-exec fork gate. Before `fork`, the parent
constructs immutable `argv`/`envp` storage from the empty environment, creates every pipe, prepares all
descriptor numbers and close-on-exec states, and opens the pinned wrapper, Codex, workspace,
native source, native realization, and `system-empty` directories. It also serializes exactly one
bounded `E3WorldFsEnforcementInputV1` from the already authenticated E2 launch activation or
retained-turn carrier and exactly one bounded `E3CodexLaunchPlanV1` containing the complete per-turn
argv, final clean environment, output-file identity, turn binding, and all four directory identities.
The plan hash is copied into the enforcement input before either pipe is finalized. The post-fork child performs only raw
async-signal-safe `read`, `close`, `capset`, `prctl`, `execveat`, and `_exit` syscalls over that
prebuilt memory; `capset`/`prctl` may only raise the three parked transition capabilities into the E3
wrapper child's effective/ambient sets before wrapper exec. It does
not allocate, lock, log, construct Rust strings/maps, inspect ambient environment, or run Rust
destructors. It waits on a private wrapper-exec gate while the parent places its PID in the exact
member cgroup through a control descriptor retained only by world-service. One gate byte permits only
the descriptor-pinned wrapper `execveat(AT_EMPTY_PATH)`. No cgroup, nftables, accepted-home registry,
installer-store, or other enforcement-control descriptor is inherited by the child.

For that wrapper exec, the already-built environment contains exactly ten pairwise-distinct decimal
descriptor pointers: `SUBSTRATE_E3_CODEX_LAUNCH_PLAN_FD`,
`SUBSTRATE_E3_NATIVE_REALIZATION_FD`, `SUBSTRATE_E3_NATIVE_SOURCE_FD`,
`SUBSTRATE_E3_SYSTEM_EMPTY_FD`, `SUBSTRATE_E3_WORLD_FS_INPUT_FD`,
`SUBSTRATE_WORLD_ENTRY_BINARY_FD`, `SUBSTRATE_WORLD_ENTRY_FINAL_EXEC_FD`,
`SUBSTRATE_WORLD_ENTRY_SETUP_READY_FD`, `SUBSTRATE_WORLD_ENTRY_USERNS_FD`, and
`SUBSTRATE_WORLD_ENTRY_WORKING_DIR_FD`, plus the literal
child role `SUBSTRATE_WORLD_ENTRY_ROLE=codex`. They name the bounded launch-plan reader, native
realization/source/system-empty directories, bounded enforcement-input reader, exact manifest-pinned
Codex ELF, final-exec reader, setup-ready writer, user-namespace setup socket, and pinned workspace
directory. File descriptors
0, 1, and 2 are respectively the already-created prompt reader, event stdout writer, and stderr
writer; the plan fixes those numbers and the wrapper validates their pipe types and directions. The
wrapper-exec gate and its parent end are closed before wrapper exec. `CODEX_BINARY`, every cgroup
control pointer, pathname-valued V1 wrapper key, and every other environment entry are absent.

The descriptor-pinned ELF wrapper strictly parses and removes those eleven variables, verifies all
descriptor types and non-aliasing, reads the launch plan and enforcement input once each to separate
64-KiB bounded EOF, closes both pipe FDs, strictly decodes/re-encodes them, reproduces both hashes,
requires the plan hash to equal the enforcement input, exact-matches every plan directory to its held
descriptor, validates the plan's internal argv/environment/output/turn rules, reproduces the E2
snapshot bytes/hash, and verifies its PID is
already in `expected_process_cgroup`. It completes the protected-user-namespace setup handshake, then
creates its private mount namespace, bind-mounts the accepted
immutable config and `system-empty` mounts as specified above, applies the exact authenticated E2
filesystem policy plus only the role-specific execution-support paths below, and performs the
complete source-matched loader-input validation inside that namespace. It then completes the
one-way security transition below and writes exactly one bounded canonical
`CodexSetupReadyAttestationV1` to the setup-ready pipe, closes that pipe, and blocks on the
close-on-exec final-exec pipe. The parent exact-validates the attestation, pinned PID/cgroup, native
root, `/proc/<pinned-pid>/ns/mnt`, and `/proc/<pinned-pid>/root/etc/codex` against its retained
descriptors, then completes the final authority and access-boundary CAS. No setup-ready bytes or a
closed pipe authorize execution.
The parent independently exact-matches the decoded plan and setup-ready plan hash to the configured
accepted-home projection and current E2 launch/turn authority before either gate release; the wrapper
has no registry descriptor and does not pretend to resolve authority itself.

`E3CodexLaunchPlanV1` is process-local and nonpersistent. `codex_argv` is the complete initial array
or the complete resume prefix plus computed output path, `resume`, session ID, and `-` specified
above. `final_environment.inherited_names` is empty and its set/remove vectors are byte-equal to the
native projection. `required_absent_codex_home_entries` is exactly the bytewise-sorted vector
`[.credentials.json,auth.json,cloud-requirements-cache.json]`. While holding the runtime control
mutex and after proving every prior Codex turn cgroup empty, the parent descriptor-relative
`fstatat(AT_SYMLINK_NOFOLLOW)` checks those three single-component names beneath the exact held
`codex-home` and requires `ENOENT` before creating the next wrapper child. It never opens, reads,
hashes, unlinks, truncates, or repairs a present entry; any regular file, symlink, directory, special
file, mount, or observation error is `Conflict`, revokes the retained runtime, and forbids resume.
The wrapper repeats the same absence checks after its private mounts and immediately before
setup-ready; the parent repeats them through the pinned wrapper root immediately before final-exec
release. The output file is descriptor-relatively reopened beneath the held realization;
its relative and guest paths, owner/mode, zero length, inode/device, and empty-byte SHA-256
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` must all match. Initial and
resume variants reject the other's fields. The plan hash is the canonical JSON hash of
`{"domain":"substrate.e3.codex-launch-plan.v1","plan":<plan with plan_hash omitted>}`. Equal plan
bytes for another turn, fence, projection, output inode, or descriptor set never substitute.

`E3WorldFsEnforcementInputV1` does not create or persist E2 authority. For an initial child it copies
the exact launch snapshot bytes/ref/hash/revision from the already exact-validated
`E2MemberLaunchActivationCarrierV1`; for a resumed child it copies them from the
`DispatchPolicySnapshotCarrierV1` only after the landed E2 authenticated-copy, immutable-worker-cap,
current-parent, acceptance, and durable join checks succeed. The existing
`PolicySnapshotV3` decoder/canonical-byte check and world-service `resolve_policy_inputs` path derive
the filesystem enforcement plan. E3 neither rewrites the snapshot nor computes a replacement
commitment. E3 supports only authenticated `world_fs.isolation=full` with available Landlock and,
when the E2 plan requires strict deny mounts, the landed strict deny-mask/seccomp primitives;
anything weaker or unavailable is `UnsupportedSecurityPosture` before a gateway secret is writable or
a child gate can release.
The controlling direct-PID wrapper path additionally requires the landed deny-mask preparation to
report `cleanup_created_dirs = []` and `cleanup_readonly_mount_targets = []`; E3 V1 does not fork the
landed cleanup supervisor because that would replace the pinned PID/barrier identity. A plan needing
synthetic mountpoint cleanup is `UnsupportedSecurityPosture` before wrapper exec, not silently
weakened or run under a privileged residual parent.

For role `Codex`, `immutable_config_source` is the exact held accepted-home native source and
`private_realization` is `Some` with the exact `/run/substrate/member-config/...` root. For role
`ManagedGateway`, `immutable_config_source` is `null` because its config authority is already the
persisted launch-input/projection bytes, and `private_realization` is `Some` with the exact
`/run/substrate/e3-gateway/...` root. For `ManagedGatewayReadinessProbe`, both are `null` and the
executable artifact is the same exact world-entry wrapper artifact that is already bound into the
projection identity. Any other nullability or role/root combination is malformed.

Landlock cannot broaden an already-enforced ruleset. The wrapper therefore does not install the
baseline E2 ruleset and then attempt to add E3 paths. It first strict-decodes the authenticated E2
snapshot and calls the extracted, byte-behavior-preserving E2 resolver to obtain the exact canonical
E2 discover/execute/read/write rules and deny-mount/seccomp plan. It hashes those ordered E2 rules as
`e2_plan_hash`; that value and every E2-governed rule remain unchanged. It then constructs one
`E3DerivedWorldFsEnforcementPlanV1` whose first Landlock layer is the set union of those exact E2
rules and only the descriptor-recomputed E3 support rules produced by `support_policy_version = 1`.
The union is a child execution plan, not a mutation, replacement, widening, or new persistence of
the E2 snapshot/cap/commitment. No E3 support rule may overlap an E2 workspace-policy subject except
with rights already present in the E2 rule; overlap with a narrower E2 rule is intersected to that
narrower right set. A support object outside the fixed role roots below is malformed authority.

Support policy V1 has no caller- or config-supplied allowlist. All three E3 executables are exactly
`x86_64-unknown-linux-musl` static ELFs. The installer and wrapper parse each held ELF and admit only
either `ET_EXEC` with no `PT_DYNAMIC` (`StaticExec`) or the constrained `ET_DYN` static-PIE form
(`StaticPie`) below. Both forms require no `PT_INTERP`, no `DT_NEEDED`, and no runtime loader or
shared-library dependency; the runtime-support manifest therefore always has
`elf_interpreter = null`, `dynamic_loader_cache = null`, and `ordered_elf_dependencies = []`.

The `StaticPie` form permits one `PT_DYNAMIC` solely for self-relocation. Its entries, in ELF order,
may contain only `DT_INIT`, `DT_FINI`, paired `DT_INIT_ARRAY`/`DT_INIT_ARRAYSZ`, paired
`DT_FINI_ARRAY`/`DT_FINI_ARRAYSZ`, `DT_HASH`, `DT_GNU_HASH`, `DT_STRTAB`, `DT_SYMTAB`, `DT_STRSZ`,
`DT_SYMENT`, `DT_DEBUG`, `DT_PLTGOT`, `DT_RELA`, `DT_RELASZ`, `DT_RELAENT`, `DT_FLAGS`,
`DT_FLAGS_1`, `DT_RELACOUNT`, and one terminal `DT_NULL`. Unknown tags and duplicate singleton tags
are rejected. `DT_SYMENT` and `DT_RELAENT` are exactly 24; the dynamic string table is exactly the
single NUL byte; the dynamic symbol table contains only its mandatory all-zero local undefined entry;
there is at least one of `DT_HASH`/`DT_GNU_HASH`; every address/extent lies wholly in the held ELF's
declared load segments; `DT_FLAGS` has exactly `DF_BIND_NOW`; `DT_FLAGS_1` has exactly
`DF_1_NOW|DF_1_PIE`; and the sole relocation table is `DT_RELA`, containing only
`R_X86_64_RELATIVE` entries with symbol index zero. `DT_REL`, `DT_JMPREL`, `DT_PLTREL`,
`DT_PLTRELSZ`, `DT_NEEDED`, `DT_RPATH`, `DT_RUNPATH`, `DT_SONAME`, `DT_TEXTREL`, a nonempty dynamic
symbol/string payload, lazy binding, or any other runtime resolution input is forbidden. The
metadata struct records the exact dynamic segment and RELA values; `relative_relocation_count`
equals `DT_RELACOUNT` and `DT_RELASZ / DT_RELAENT`, and
`ordered_dynamic_entries_sha256` is ordinary lowercase SHA-256 over the exact file-order dynamic
entry bytes including the terminal `DT_NULL`. This form matches the pinned Codex 0.125 static-PIE
artifact without treating its relocation metadata as a loader dependency.

Any dynamically linked, malformed, or architecture-mismatched artifact is
`UnsupportedRuntimeVersion` before publication or child creation. Except for the exact CA rule
below, the wrapper resolves every remaining support object component-by-component from held `/` and
realization descriptors, rejects every symbolic or magic link and escape, opens the final object
with `O_NOFOLLOW`, and computes this exact closure:

1. `validate_e3_static_elf_v1` reproduces the static-ELF checks above on the same target descriptor
   used for final execution. No library root, loader, loader cache, or shared object enters either
   Landlock layer.
2. The fixed Codex/gateway regular/device set is exactly `/dev/null` (character device major/minor `1:3`,
   read/write), `/dev/urandom` (character device `1:9`, read),
   `/etc/hosts`, `/etc/nsswitch.conf`, `/etc/passwd`, `/etc/group`, `/etc/resolv.conf`, and
   `/etc/ssl/certs/ca-certificates.crt` (read). Every named regular file is mandatory, and its
   device/inode/SHA-256 is in the trusted installed artifact support manifest and projection
   identity. Ordinary direct regular-file installation of the logical CA path remains valid. As the
   sole symlink exception in support policy V1, that exact logical CA path may instead be one
   root-owned symbolic link whose raw `readlinkat` bytes are exactly
   `../../ca-certificates/extracted/tls-ca-bundle.pem` and whose final object remains beneath the held
   `/etc` resolution boundary. The publisher, importer, and wrapper each open and validate the same way:
   they descriptor-open `/etc`, require it and every traversed directory to be root-owned and not
   group/other-writable, walk without directory enumeration, require the logical CA leaf to be the
   only encountered symlink, and compare its owner and raw target before expansion. `..` in that
   target pops one already-held
   descendant descriptor and is valid only while the stack remains at or below the held `/etc`;
   an absolute target or a pop above that boundary is an escape. A different relative target is not
   accepted even if it would remain beneath `/etc`. Resolution rejects a mount crossing, magic link,
   second symlink or cycle, NUL or malformed component, and any substituted, untrusted-owned, or
   group/other-writable component.

   The final open is `O_RDONLY|O_CLOEXEC|O_NOFOLLOW` from the resolved parent descriptor. Its object
   must be a root-owned, non-group/other-writable regular file with link count one and the existing
   support-file size bound. Metadata is captured by `fstat` on that descriptor; SHA-256 is read from
   the same descriptor; a second `fstat` must reproduce device/inode/type/mode/owner/link-count/length.
   Re-resolving the exact logical name before publication/import completion or private realization
   must select the same final device/inode and bytes. The manifest continues to store the logical CA
   pathname plus that final object's identity and digest, so link or endpoint substitution, endpoint
   drift, a nonregular endpoint, absence, or hash mismatch is `UnsupportedSecurityPosture`.

   Before either artifact's Landlock layers are installed, `substrate-world-entry` performs the final
   re-resolution while unrestricted setup authority exists, retains the validated endpoint
   descriptor, and uses only its already-private mount namespace to mount an empty
   `nodev,nosuid,noexec` filesystem over `/etc/ssl/certs`, create its single non-writable
   `ca-certificates.crt` mountpoint, and bind/remount the held endpoint there read-only. It then
   reopens the logical path without following links and requires the mounted file's
   device/inode/type/mode/length/digest to equal the manifest and held descriptor before applying the
   existing exact-file Landlock rule. The host symlink and endpoint are never changed. No Landlock
   rule or runtime access is added for `/etc`, `/etc/ssl`, `/etc/ssl/certs`, the resolved endpoint
   pathname, or another trust source; directory enumeration remains denied, and the descriptor is
   closed after the rule is installed. All other support-file symlinks remain forbidden. No other
   `/etc`, `/usr`, `/lib*`, `/dev`, or CA object is admitted.
   The separately represented `/etc/codex` mount target is the sole additional `/etc` directory: it
   is descriptor-resolved and covered by the private bind mount before Landlock restriction, receives
   no E3 rule for `/`, `/etc`, or another ancestor, and grants no read or enumeration of the underlying
   shared directory.
3. `Codex` adds execute/read for the held Codex ELF, read/discover for the descriptor-validated
   numeric `/proc/<wrapper-pid>` subtree only, read for the
   exact read-only mounted `codex-home/config.toml`, read/write for the exact `home`, `codex-home`,
   `state`, and `tmp` descendants of its held realization, and every E2 path/right unchanged. Its
   accepted-home source descriptor is consumed for validation/mounting before Landlock and is not in
   either applied layer. `ManagedGateway` adds execute/read for the held gateway ELF,
   read/discover for its descriptor-validated numeric `/proc/<wrapper-pid>` subtree only, and
   read/write for its one held gateway realization; it adds
   no E2 workspace/project rule. `ManagedGatewayReadinessProbe` executes no second artifact and adds
   only its descriptor-validated numeric `/proc/<probe-pid>` subtree read/discover plus its already-
   open setup/start/result descriptors; it receives
   none of the common filesystem set except loaded dependencies already mapped before Landlock.
4. No E3 Landlock rule is emitted solely to make an ancestor traversable. The wrapper opens and
   validates every positive rule target while unrestricted setup authority still exists; ordinary DAC search
   permission provides later pathname traversal without `LANDLOCK_ACCESS_FS_READ_DIR` on an
   ancestor. `e3_support_discover_paths` therefore contains only the exact numeric self-proc subtree
   for the applicable role; enumeration of each private realization subtree is already an explicit
   consequence of that exact subtree's read/write rule. It never contains `/`, `/etc`, `/usr`,
   `/lib*`, `/dev`, an accepted-home ancestor, a realization parent, or an ancestor of an exact file
   or executable merely because the leaf is admitted. E2 discover paths remain byte-identical E2
   authority and are not synthesized, removed, or broadened by E3. Execute, read, and write vectors
   contain only the exact objects above. All four vectors are bytewise sorted and duplicate-free.
   The wrapper recomputes them from held descriptors and the trusted artifact support manifest; the
   pipe supplies only the literal policy version. Any required object that cannot be represented by
   the running Landlock ABI rejects E3 rather than broadening a directory rule.

The wrapper next stacks a role-narrowing Landlock layer containing exactly the applicable V1 closure
above, so kernel intersection produces the final rights. Thus E2 remains exact on E2-governed paths,
Codex receives only named E3-owned support objects, and the gateway/probe receives no workspace
rights. Neither layer contains a rule for the accepted-home registry/source path, another
series/fence realization, `/sys/fs/cgroup`, nftables/netlink control, `/proc/<other-pid>`, the other
role's private root, installer stores, or world-service state. The attestation separately binds the
exact E2 plan hash, dependency/support closure, derived-support layer hash, role-narrowing layer hash,
and effective intersection hash.

The numeric proc subject is not resolved through the `/proc/self` magic link. Before applying
Landlock, the child opens a trusted procfs root descriptor, formats `getpid()` as the shortest decimal
component, resolves that one component with `openat2(RESOLVE_BENEATH|RESOLVE_NO_MAGICLINKS|
RESOLVE_NO_SYMLINKS|RESOLVE_NO_XDEV)`, and verifies procfs mount identity plus the directory's
`stat`/`status` PID, namespace PID, and start-time tuple against itself. The held numeric descriptor
defines the Landlock subject and remains the same PID across final exec. Failure, a magic/symbolic
link, a non-procfs mount, a PID/start-time mismatch, another numeric PID, or any requested proc parent
or sibling is `UnsupportedSecurityPosture`; `/proc/self` is never an admitted or opened E3 support
path.

E3 V1 does not use host-wide Yama policy as authority. Substrate must not write or temporarily toggle
`/proc/sys/kernel/yama/ptrace_scope`, require an administrator to change it, or encode any observed
value as an enforcement input, attestation, or canonical hash. The existing host value may be read
only as diagnostic and acceptance evidence that it stayed byte-identical before, during, and after
E3 operation; no particular value, including the feasibility probe's observed `1`, is required. If
the unchanged host policy prevents an operation whose existing Substrate semantics require trusted
tracing, that exact operation fails with its real compatibility error before release; E3 may not
silently disable tracing or claim a weaker execution result. The boot ID remains required because it
binds process and namespace identities across PID/inode reuse, not because it attests Yama.

The primary world-service process remains in the inherited host user namespace for its complete
lifetime. Before registering E3 routes it descriptor-opens its own user namespace and binds that
descriptor's device/inode plus the installed trusted service UID and boot-bound service PID/start
identity. It never calls `unshare(CLONE_NEWUSER)` or enters a child user namespace. Each Codex,
managed-gateway, and managed-gateway-readiness wrapper instead creates a fresh user namespace for
that one descriptor-pinned wrapper/final-exec process tree. The namespace must be distinct from the
service namespace and every sibling or other-role namespace; descendants may inherit only their
exact parent's namespace. A shared user namespace is not isolation among members, so no gateway,
separate participant, sibling, readiness probe, non-E3 child, or unclassified helper may join that
unit. The existing cgroup, role-specific Landlock, seccomp, gateway non-dumpability, and process-wide
exclusion requirements remain independently mandatory.

World-service creates one parent-owned `SOCK_SEQPACKET|SOCK_CLOEXEC` user-namespace setup socketpair
for the exact child and clears close-on-exec only on the child endpoint for the single pinned wrapper
exec. Only that endpoint and the nonsecret decimal pointer
`SUBSTRATE_WORLD_ENTRY_USERNS_FD` cross the pinned wrapper exec. After strictly parsing its trusted
launch descriptors, and before a mount, policy effect, target executable, secret byte, listener
allowance, or prompt can be exposed, `prepare_private_child_namespace` calls
`unshare(CLONE_NEWUSER)`, sends the sole one-byte namespace-created message `0x01`, and blocks. The trusted wrapper
exec before this call is intentionally inside the parent namespace but is descriptor-pinned, receives
no secret, and cannot pass either release gate; the gateway and authorized agent executable never run
there. World-service binds the notification to the held pidfd/PID/start tuple, opens
`/proc/<pid>/ns/user` itself, and validates `NS_GET_NSTYPE`, `NS_GET_OWNER_UID`, `NS_GET_PARENT`, and
descriptor identity. The owner is exactly the installed trusted service UID, currently host UID 0;
it is never the ordinary developer UID. The parent is byte-identical to the service's held host-user-
namespace descriptor. The parent accepts only one `0x01` packet and no ancillary descriptor.

While the wrapper remains blocked, the same synchronous service thread raises only its already-
parked `CAP_SETUID` and `CAP_SETGID` into its effective set, writes one UID-map extent and one GID-map
extent, and immediately clears and reads back both effective bits before sending the fixed mapped
release `0x02`. There is no await, callback, fork, or unrelated work while those bits are raised. Each extent
is exactly `<target-id> <same-host-target-id> 1`, where the nonzero UID and GID come from the installed
bootstrap identity; no inside or outside host-root mapping, subordinate range, supplementary identity,
caller-selected map, new account, or additional host capability is allowed. The parent rereads both
map files exactly, validates the held namespace descriptor again, and retains that descriptor and its
bound pidfd/process identity until the exact child cgroup is empty. The wrapper independently verifies
its actual namespace membership and exact maps after receiving the sole one-byte mapped release
`0x02`, then closes the setup socket. EOF, duplicate/trailing bytes, or a substituted
notification, descriptor, owner, parent, map, member PID, or reused namespace identity fails closed.
Every error or unwind after a parent capability is raised must clear and read it back; inability to do
so aborts the service while the child remains unreleased.

This per-child direct-creation lifecycle uses no later user-namespace `setns`: precreating one shared
namespace and entering it later is neither required nor authorized. Any already-required network-
namespace entry happens in the existing trusted post-fork setup before wrapper exec and before the
user-namespace boundary using the already-effective service `CAP_SYS_ADMIN`. The trusted wrapper uses
that same existing bit only for direct user-namespace creation; afterward its setup capabilities are
scoped to the new namespace and cannot administer the parent namespace. After mapped release it uses
namespace-local `CAP_SYS_ADMIN` to create its private mount namespace, fixes its filesystem UID/GID to
the configured identity, and performs the existing descriptor-rooted mount and Landlock setup. It
then clears supplementary groups, uses only namespace-local `CAP_SETGID`/`CAP_SETUID` for final ID
descent and `CAP_SETPCAP` for the securebits/bounding-set transition, descends all
real/effective/saved/filesystem IDs to the configured UID/GID, and removes every capability. It then
sets `PR_SET_NO_NEW_PRIVS=1`, establishes `PTRACE_TRACEME` before filtering only
when the existing execution path requires the trusted service parent to trace it, applies the role's
dumpability/core posture, and installs the existing seccomp filter returning `EPERM` for `ptrace`,
`process_vm_readv`, `process_vm_writev`, `kcmp`, `pidfd_getfd`, `bpf`, `perf_event_open`, all
mount-family calls, and namespace creation/setns. Final exec cannot change namespace membership.
Failure of namespace creation/mapping, filesystem setup, ID descent, capability removal, required
trusted tracing, or any readback aborts before secret delivery or final-exec/prompt release. The
forked-child feasibility probe did not exercise this wrapper/world-service integration, a precreated
namespace, or later `setns`; E3-D must prove the selected ordering against the actual static wrapper.

Provisioning adds those three transition capabilities to both `CapabilityBoundingSet` and
`AmbientCapabilities` so primary service exec initially places them in
bounding/permitted/inheritable/effective sets, and sets `SecureBits=noroot-locked`. The installed
binary's synchronous `main` checks the internal-exec argument before changing capability state, so
the existing V1 internal helper path remains byte-behavior unchanged. On the primary daemon path,
while it is still the sole thread and before constructing Tokio or any other thread, `main` lowers
exactly those three bits from ambient and effective, retains them in permitted/inheritable, reads all
five sets and securebits back, and aborts on mismatch. Only then may it build the multi-thread runtime
and call `run_world_service`. Every runtime worker is therefore created from the parked state; an
`on_thread_start` readback aborts the process if any worker differs, and later blocking/helper threads
inherit from an already-verified thread. Existing baseline service capabilities remain explicitly
ambient/effective as at the baseline. A non-E3 fork never raises the three parked transition bits,
and locked `NOROOT` prevents UID-0 exec semantics from regaining them. Only the E3 post-fork syscall
stub raises all three for the manifest-pinned wrapper, and only the bounded parent map-installation
scope above raises `CAP_SETUID`/`CAP_SETGID` on its one service thread. They are parked again before
mapped release and absent from every released E3 child exec.

The baseline capabilities that remain effective in trusted world-service include
`CAP_SYS_PTRACE`, `CAP_SYS_ADMIN`, `CAP_NET_ADMIN`, and `CAP_DAC_OVERRIDE`. Baseline V1/UAA,
ordinary execute, PTY, and compatibility-gateway children do not all descend those bits, so
they may not coexist with a credential-bearing E3 epoch anywhere in the same service process. The
process-wide `E3PrivilegedChildExclusionV1` below is therefore a mandatory security boundary, not an
optimization. Before any E3 gateway wrapper can fork or the first E3 gateway secret can be written,
it must hold the exclusive mode,
prove the service-wide count of live non-E3/unclassified child processes is zero, and prove from
`/proc`, its child registry, and every service-owned world/command/member/PTY/gateway cgroup that
there is no unclassified descendant, including a reparented or daemonized process. While any E3 epoch lease is
live, every ordinary execute/stream/PTY, strict-V1 member launch or turn, compatibility
gateway start/restart, manual/periodic GC helper spawn, and other non-E3 or privileged-helper spawn is
rejected before fork, exec, secret access,
or other child side effect. Additional E3 participants may join only the same exact world ID and
generation; two such siblings retain separate roots, cgroups, gateways, and leases. The final E3
revocation kills all E3 children and gateways and proves their cgroups empty before returning the
exclusion to idle. A service restart begins `Recovering`, kills/revokes classified E3 remnants,
exact-validates and registers any adoptable compatibility-gateway remnant under a pending non-E3
lease, kills every other classified remnant, and cannot enter either legacy-shared or E3-exclusive
mode until no unclassified descendant exists.

Outside an E3-exclusive epoch, V1 schemas, validation, launch behavior, and wire bytes are unchanged.
During an E3-exclusive epoch, a valid V1 or ordinary execution request receives the new typed
`UnsupportedSecurityPosture` admission failure; it is never reinterpreted as V2. This is the exact
mixed-version rule: old servers reject V2, new servers continue to run V1 when the exclusion is idle,
and new servers fail closed rather than co-running privileged V1 code beside E3 secrets.

Before serializing an enforcement input, world-service constructs
`denied_control_probe_targets` only from its already-held accepted-home, realization, cgroup,
network-namespace, world-service-state, and pidfd/process identities. Each `target_hash` is recomputed,
and the complete list is ordered by the binding enum order above, then by canonical path bytes or
PID/start tuple within a repeated variant, with no duplicate target hash. Directory variants carry
the exact `CanonicalDirectoryV1`; `CgroupControl` carries the exact cgroup identity and literal
`cgroup.procs`; `NftablesControl` carries the child's exact network-namespace inode; and
`OtherRoleProcessState` carries the exact live PID/start tuple and descriptor-validated procfs mount
identity. Gateway setup may omit only a sibling or other-role process/root that does not yet exist;
Codex and readiness setup include every then-existing gateway, sibling, and other-role target.
World-service revalidates each held identity immediately before wrapper-exec release. No target or
control descriptor is inherited: the wrapper derives the one probe pathname or namespace operation
solely from this authenticated, hash-bound input, and a caller or ambient path cannot select it.

The trusted wrapper reads back its own UID/GID tuples, zero supplementary groups, all five zero
capability masks, `NoNewPrivs: 1`, seccomp-filter mode, role-appropriate dumpability and tracer
posture, Landlock ABI and ruleset hash, the exact user-namespace owner/parent/maps/membership, and
exact E2/input hashes into `E3ChildSecurityAttestationV1`. A kernel
`cap_last_cap > 63` is unsupported by V1 rather than truncated. It also performs negative
probes in that exact target order: operation `open_read_directory` returns `EACCES` for the registry,
sibling root, world-service state, and other-role private root; `open_write_cgroup_procs` returns
`EACCES`; `send_noop_nft_batch` returns `EPERM` in the exact named network namespace; and
`open_read_proc_status` returns `EACCES` for `/proc/<pid>/status` under the exact other-role
PID/start tuple and procfs identity. No other operation string or errno is valid. The netfilter
probe contains no rule/table operation and cannot mutate state even if a kernel unexpectedly accepts
the empty batch. Each emitted result repeats byte-for-byte the input target and the exact fixed
operation string; missing, extra, reordered, substituted, or rehashed targets are malformed. Absence
is omission, never a successful probe; before a later sibling's first child release, world-service
runs reciprocal same-UID probes against both existing runtimes. The canonical ordered results form
`denied_control_probe_hash`. The parent treats the wrapper-emitted pipe bytes only
as evidence from the manifest-pinned trusted wrapper, independently checks `/proc/<pinned-pid>/status`,
UID/GID maps, groups, cgroup, seccomp, user/mount namespace membership, open-FD inventory, and the
descriptor identities bound into the wrapper's negative target set, revalidating every held target
after the results return. The namespace fields in the enforcement input and child attestation are
covered by their existing canonical hashes; a matching hash never substitutes for the parent-held
namespace descriptor and live membership readback.

The same-UID denial canary must isolate the user-namespace effect from ambient host policy. A fresh
same-target-UID, zero-capability inspector outside the target namespace first exercises the identical
operations against a `Dumpable=1` same-host-user-namespace control that explicitly authorizes that
inspector; ptrace attach, `process_vm_readv`, `/proc/<pid>/mem`, and `pidfd_getfd` must all succeed.
Using the same inspector and authorization shape against the post-exec protected target, ptrace and
`process_vm_readv` must fail with `EPERM`, `/proc/<pid>/mem` must fail with `EACCES` or `EPERM`, and
`pidfd_open` may succeed but `pidfd_getfd` must fail with `EPERM`. The inspector receives no namespace
descriptor. A failed positive control is inconclusive and cannot be reported as namespace protection;
the exact compatibility/security failure is surfaced without changing host policy. The privileged
parent's own `CAP_SYS_PTRACE` result cannot substitute. Every exact identity/value/hash and conclusive
canary is required before release. Separate fresh namespaces, the syscall filter, Landlock's
inaccessible `/proc/<gateway-pid>` rule, and the gateway's non-dumpability together keep same-host-UID
Codex and sibling/other-role processes from inspecting the credential-bearing gateway.

Gateway exec is the one stricter post-exec case because that process retains the credential. Before
world-service writes the auth bundle, the pinned gateway must complete the managed-gateway
secret-ready barrier proving `PR_GET_DUMPABLE=0`, both `RLIMIT_CORE` values zero, and `TracerPid=0`;
those values remain zero for the gateway lifetime. A non-secret Codex exec may have `Dumpable=1`
only because it receives no gateway credential, while its exact protected-user-namespace membership
and the conclusive cross-process denial checks still gate its prompt release. When an existing
Substrate path requires trusted tracing, `TracerPid` instead must identify the boot-bound trusted
service tracer and the expected exec/exit stops and readbacks must succeed; an untraced role still
requires `TracerPid=0`.

This boundary protects process memory and file descriptors across its validated namespace edge. It
does not prevent same-UID signals or modification of intentionally shared user files, and it does not
make every same-UID process trusted. Descriptor-pinned executables, installed configuration, policy
inputs, accepted-home/native-source authority, and control objects retain all existing integrity,
Landlock, mode, ownership, and hash requirements. E3 claims no resistance to a hostile host
administrator, kernel compromise, or another process already admitted inside the same protected
process tree.

Only after the `Active/Released` record and allowed boundary are durable and exact-revalidated does
the parent write one final-exec byte for this initial or resumed turn. The wrapper consumes it,
immediately revalidates its namespace, security posture,
masked directory, native root, its in-wrapper authenticated launch plan, final argv/environment/
output identity, and all held descriptors, `fchdir`s to the workspace, closes the native-source,
native-realization, system-empty, working-directory, setup, and final descriptors, marks the Codex descriptor close-on-exec, and
executes that same open file description with `execveat(AT_EMPTY_PATH)`. Its own executable
descriptor closes on entry. Every unrelated descriptor is closed before final exec; successful Codex
exec closes the Codex descriptor atomically. Thus Codex receives no launcher pointer, launcher FD, or
executable pathname and its environment is byte-equal to `effective.environment.set`. Neither
adapter nor wrapper may recover a value from ambient environment. A crash, EOF, duplicate byte,
malformed attestation, or identity change at either stage kills the unreleased cgroup and revokes the
boundary; it never advances to Codex exec. `Active/Released` releases the retained projection/gateway
capability, not an unlimited process launcher and not evidence that any particular Codex process ran.
The initial E2 launch identity or the resumed turn's already durable `CallEntered` join state supplies
the per-turn exact-once authority. A crash after a final-gate write is handled by the landed E2
launch/join uncertainty rules and is never retried merely from the E3 record.

The parent retains the prompt pipe write end until the pinned PID's `/proc/<pid>/exe` matches the
no-setid/no-file-capability Codex artifact and post-exec checks reproduce target UID/GID, zero
capability masks, no-new-privileges, `Dumpable=1`, role-appropriate trusted-tracer identity, the same
boot ID, exact membership in the still-held protected user namespace, seccomp mode, Landlock/cgroup
identity, and absence of every wrapper/gateway/secret/control descriptor. Only then does it write the prompt plus LF and close the
pipe. Failure kills the cgroup and follows the same E2 uncertainty path; a Codex event or session ID
cannot substitute for this check.

The generated TOML is UTF-8 without BOM, LF-only, and ends in exactly one LF. Fixed keys are bare;
every data-derived table component and inline-table key is a basic quoted key. Basic strings and
quoted keys encode `"`, reverse solidus, backspace, tab, LF, form feed, and CR as `\"`, `\\`, `\b`,
`\t`, `\n`, `\f`, and `\r`; every other C0 control and DEL uses lowercase four-digit `\u00xx`.
Solidus is never escaped and every other Unicode scalar remains its original UTF-8. Arrays are
`["a", "b"]` with comma plus one space and are `[]` when empty. Nonempty inline tables are
`{ "A" = "a", "B" = "b" }`, keys bytewise sorted; empty optional tables are omitted. Assignment
uses one ASCII space around `=`, Booleans are lowercase, and a single empty line precedes each table
header with no other empty lines. Tables and keys occur only in the order below plus the exact MCP and
feature tables described immediately after it:

```toml
model = "<effective.model>"
model_provider = "substrate-managed-gateway"
check_for_update_on_startup = false
cli_auth_credentials_store = "ephemeral"
log_dir = "<root>/state/log"

[model_providers.substrate-managed-gateway]
name = "Substrate managed gateway"
base_url = "<managed_gateway.codex_base_url>"
wire_api = "responses"
requires_openai_auth = false
supports_websockets = false
http_headers = { "X-Substrate-Orchestration-Session" = "<session>", "X-Substrate-Participant" = "<participant>", "X-Substrate-Projection" = "<identity_hash>" }

[projects."<workspace-root-escaped>"]
trust_level = "untrusted"
```

For each bytewise-sorted `effective.mcp_servers` entry, the renderer emits one empty line then
`[mcp_servers."<server-id-escaped>"]`, then `enabled`, then exactly one transport shape. Stdio emits
`command`, `args` as an inline string array in original order including `args = []` when empty, and
`env` as a name-sorted inline table
only when nonempty; it omits `url`, `cwd`, `env_vars`, `http_headers`, `env_http_headers`, and bearer
fields. Streamable HTTP emits `url` and `http_headers` as a name-sorted inline table only when
nonempty; it omits `command`, `args`, `env`, `env_vars`, `cwd`, `bearer_token`,
`bearer_token_env_var`, and `env_http_headers`. URL fragments, userinfo, non-loopback redirects,
secret values, environment-derived values, and duplicate/invalid TOML identifiers fail closed.
These names and mutually exclusive shapes are the Codex 0.125 `RawMcpServerConfig` surface; E3 does
not invent a second `.mcp.json` authority. If `effective.features` is nonempty, one final `[features]`
table emits each Codex-0.125-recognized feature key in bytewise order with its Boolean value; an
unknown feature is `Malformed`, not an ignored extension.

This renderer grammar has the following mandatory golden fragment, including its final LF:

```toml
[mcp_servers."mcp.é"]
enabled = true
command = "/bin/mcp"
args = ["a", "line\n", "é"]
env = { "A" = "\u0000", "Z" = "v" }
```

Its exact UTF-8 SHA-256 is
`ae31989bc6eee464225d8f80fc68bce879f0b4d2d86b476aad51f01130a8a898`. Renderer tests also hash the
complete minimal document (empty MCP/features) and complete nonempty document; a parser-equivalent
but byte-different rendering is a failure.

The three `X-Substrate-*` values are non-secret correlation metadata;
`X-Substrate-Projection` is the stable `identity.identity_hash`. They are not bearer material, are not
authorization, and a matching header cannot satisfy the gateway access boundary.

Codex 0.125 loads more than `${CODEX_HOME}/config.toml`; it exposes no CLI origin-attestation report
that E3 can trust. `ambient_closure` is therefore computed by a source-matched host-side validator,
not by a nonexistent Codex callback. The fixed Linux source identity is tag `rust-v0.125.0`,
`config_loader/mod.rs` SHA-256
`f8e2eff1db4d4cc004dff849682e81e224acca742b5d38d94e2db4a713b560bc`,
`config_loader/layer_io.rs` SHA-256
`5b8aa76b0776370cc6db86b679efbb6bab361d2d474dc59768724f8a4c80a52a`, and its loader-model
`README.md` SHA-256
`8c6573bc83f53396a31bae92371790997c30521e46d29c0662c44d58d570ea81`. The same identity also pins
`exec/src/lib.rs` SHA-256
`b113fd23d8a0d264556b234fb067a4233ab8c3d5491dd393c0bc39b53c3170bd`,
`cloud-requirements/src/lib.rs` SHA-256
`f43176f2889c54d19b65b0a669e98e22effc52712ca38710b9265ee22c804c77`,
`login/src/auth/storage.rs` SHA-256
`31c05505d2ed91f852a225e154929db3083ae61ef8a662fb6aad09442c33650c`, and
`config/src/types.rs` SHA-256
`927c2a72b29136a5d8f627450a1f85a91173bc76e863351fd7233664b5ee32e4`.
Any upstream-source mismatch is `UnsupportedRuntimeVersion`, not best-effort validation.

`allowed_enabled_layers` is exactly `["System","User"]`, in Codex precedence order. Before record
publication, the authority creates and descriptor-validates the empty owner-only `system-empty`
directory beneath the native root. Before each initial or resumed exec, the wrapper creates a private
mount namespace and bind-mounts that exact directory read-only over `/etc/codex`; the validator descriptor-opens that mount and
records `ProvenAbsent` attestations for `config.toml`, `managed_config.toml`, `requirements.toml`,
`rules/`, and `skills/`. The `System` layer consequently exists only as Codex's required empty layer.
The `User` attestation is the exact projected `codex-home/config.toml`; `codex-home/rules/`,
`skills/`, `requirements.toml`, `managed_config.toml`, and every profile-named file are
`ProvenAbsent`. The immutable projection records the expected source-directory and loader-input
closure. The wrapper's setup-ready attestation proves the per-exec mount realization; both wrapper and
parent revalidate the mount and native-root descriptors after namespace setup and immediately before
the final-exec barrier release.

The validator implements the pinned loader's default project-root-marker and trust-key algorithms.
With `cwd` exactly the recorded workspace root and no CLI overrides, it enumerates and attests the
exact project-directory chain that pinned `load_project_layers` forms, in increasing precedence,
from the resolved project root through `cwd`. For each directory in that chain it observes the
`.codex` entry exactly as the pinned loader does and attests only that directory's
`.codex/config.toml` plus folder-derived project rules/skills; existing objects are opened no-follow
and hashed and absent objects are proven absent relative to pinned directories. Every such entry must
be `DisabledByTrust` by the exact projected User-layer trust decision. A trusted, ambiguous,
differently rooted, or source-divergent decision fails before exec. Project bytes may be recorded as
disabled evidence but never enter any projection value.

Pinned Codex 0.125 does not load `<cwd>/config.toml`, and git-root discovery contributes only trust
lookup keys; it does not inject a separate `<git-root>/.codex/config.toml` layer. Neither locator may
appear in `CodexAmbientConfigClosureV1.inputs` or in source-matched differential golden vectors. A
git root that is also one of the ordinary project-root-to-cwd ancestors is covered exactly once by
that ancestor's normal `.codex/config.toml` attestation. The upstream module comments that describe
standalone cwd/repo layers are not executable authority; the pinned `load_project_layers` call graph
and its source hash control.

Linux has no MDM layer. The pinned exec source calls
`cloud_requirements_loader_for_storage` on every invocation and uses
`cli_auth_credentials_store.unwrap_or_default()`; the pinned default is `File`, so omission is not
closed. E3 therefore renders the literal `cli_auth_credentials_store = "ephemeral"` above and
rejects any other effective value. The pinned ephemeral backend is process-memory-only and is empty
in each newly execed Codex process. Because E3 passes no auth descriptor, auth environment value, or
login operation, the auth manager returns no auth without reading `${CODEX_HOME}/auth.json`; the
cloud-requirements loader then returns `None` before cache lookup, cache write, refresh-task creation,
or network access. A managed-gateway URL or response is not ChatGPT auth and cannot alter that branch.

The closure contains three exact per-exec pseudo-layer inputs after `User`: locator
`${CODEX_HOME}/.credentials.json` uses `DisabledByEmptyMcpSetAndProvenAbsent` because the controlling
V1 MCP set is exactly empty; locator `${CODEX_HOME}/auth.json` uses
`DisabledByEphemeralCredentialStoreAndProvenAbsent`; locator
`${CODEX_HOME}/cloud-requirements-cache.json` uses
`DisabledByNoEphemeralAuthAndProvenAbsent`. For each, `directory` is the held private `codex-home`,
`relative_path` is the literal single-component basename, and device, inode, byte length, and SHA-256
are all JSON `null`. Those dispositions are valid only when the plan contains the exact three-name
absence vector, parent/wrapper checks observe `ENOENT` at every boundary specified above, and the
effective credential-store literal plus all four pinned source hashes match. A present or unreadable
name never becomes a hashed input and is not ignored; it conflicts before exec. After every Codex
child becomes terminal, the parent repeats the same no-follow absence proof before accepting the
runtime as resumable. Appearance of any name revokes the boundary/runtime and permanently rejects
that fence for another turn without reading or deleting the entry. Thus retained mutable state may
carry Codex session data but cannot carry authentication or cloud-requirements input between turns.

Thread-config layers must be empty and session/CLI overrides are exactly empty; the forbidden
argument check is repeated on the final argv. `validated_loader_input_fingerprint` is the canonical domain hash of
`{"domain":"substrate.e3.codex-0.125-loader-inputs.v1","inputs":<ordered complete attestations>,"loader_source":<loader source>}`.
The input list is ordered by Codex precedence, then locator bytes. Activation recomputes the entire
list after all mounts and requires byte-equal attestations and fingerprint. Missing evidence, an
extra locator, a newly enabled layer, an unrepresented cloud/thread/runtime input, or any effective
value difference fails closed. Differential fixtures execute the pinned official binary against
each layer permutation and must match the validator's enabled/disabled and effective-value result;
merely setting `CODEX_HOME` or writing `trust_level = "untrusted"` is not proof of closure.

## Managed gateway, handoff, and activation content

```rust
struct ManagedGatewayProjectionV1 {
    projection_hash: String,
    activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    expected_gateway_ref: InWorldGatewayRefV1,
    codex_base_url: String,
    access_boundary_ref: GatewayAccessBoundaryRefV1,
    activation_ack_ref: Option<ManagedGatewayActivationAckRefV1>,
    posture: ManagedGatewayProjectionPostureV1,
}

enum ManagedGatewayProjectionPostureV1 { Dormant, ReadyClosed, Active }

struct NonsecretHandoffProjectionV1 {
    projection_hash: String,
    secret_handoff_ref: SecretHandoffRefV1,
    credential_source_ref: CredentialSourceRefV1,
    receiving_gateway_ref: InWorldGatewayRefV1,
    delivery: SecretDeliveryMechanismV1,
    observed_state: SecretHandoffStateV1,
    activation_ack_ref: Option<ManagedGatewayActivationAckRefV1>,
}

struct ConfigProjectionActivationV1 {
    activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    publication_fence: ConfigProjectionPublicationFenceV1,
    gateway_activation_ack_ref: Option<ManagedGatewayActivationAckRefV1>,
    released_at: Option<Timestamp>,
}

enum ConfigProjectionPublicationFenceV1 {
    ZeroLiveClosed { fence_id: String },
    Released {
        fence_id: String,
        closed_record_ref: ConfigProjectionRefV1,
        activation_ack_ref: ManagedGatewayActivationAckRefV1,
        release_hash: String,
    },
}
```

For every E3 record, `delivery` is exactly
`SecureFd { fd_name: "SUBSTRATE_LLM_AUTH_BUNDLE_FD", one_time: true,
gateway_receiver_only: true, deny_child_inheritance: true, close_after_consume: true }`. Validation
requires byte equality with the embedded `LaunchTimeSecretHandoffV1.delivery` at every Dormant,
ReadyClosed, and Active resolution. A different logical FD label remains representable by the
preserved generic V1 schema but is `WrongBinding` for E3 and cannot produce a projection, intent,
ACK, or activation.

The exact gateway and ACK/ref rules are owned by
[`managed-gateway-adoption-v1.md`](managed-gateway-adoption-v1.md). A newly created activation epoch must first
publish `Dormant`, `observed_state = Prepared`, and `ZeroLiveClosed`; no Codex/UAA
process for the series may yet exist. After exact gateway readiness and one-time handoff consumption,
a later record may publish `ReadyClosed` plus the ACK while keeping the same closed fence. Only a
final record may publish `Active` and `Released`. Release is invalid unless its predecessor is that
same series' `ReadyClosed` head, its ACK matches, and no turn barrier has released before the retained
projection/gateway capability becomes active. `Released` names release of that retained capability;
it is not a per-process execution counter.

Stable identity fields agree exactly; handoff and boundary refs advance only by the unique successor
rules in the gateway contract:

| Record posture | handoff ref/state | access-boundary ref | ACK | fence |
|---|---|---|---|---|
| `Dormant` | intent's exact `Prepared` ref / `Prepared` | intent's exact `DenyAllDormant` ref | `null` | `ZeroLiveClosed` |
| `ReadyClosed` | ACK's unique `Prepared -> Delivered -> Consumed` terminal ref / `Consumed` | byte-equal to ACK and intent `DenyAllDormant` ref | one exact ACK ref | same `ZeroLiveClosed` fence |
| `Active` | same ACK `Consumed` ref / `Consumed` | unique `AllowExactMember` successor of ACK's Deny ref | same ACK ref | `Released` from the `ReadyClosed` ref |

Prepared, Delivered, and Consumed refs, or Deny and Allow refs, are never required or permitted to be equal.

## Immutable series and monotonic revision behavior

The series lookup key is the canonical identity with `series_id` and `identity_hash` omitted. A
first-writer CAS allocates one series ID. These identity fields never change for that series:

- authority store and accepted physical root;
- workspace physical identity;
- orchestration session, retained participant, and bootstrap run;
- backend and runtime family;
- world ID and generation;
- descriptor-pinned Codex, wrapper, and gateway artifacts; and
- exact immutable E2 launch/fork cap, its created/application revisions, and its snapshot.

A change to any listed field requires a new series with a newly closed zero-live fence. In
particular, world-generation rollover, participant reuse, binary replacement, or cap substitution is
not a revision.

Revisions may change only projection-owned logical inputs, the exactly equal V1 effective content, deterministic
rendering, gateway-session identity, non-secret handoff/ACK evidence, and adoption/fence evidence;
any logical/effective/native or gateway-session content change starts a fresh closed fence/root epoch.
Every revision is an immutable complete record. Revision is predecessor revision plus one; its
`predecessor_ref` must be the prior current series head. An exact retry republishes or resolves the
byte-equal record and does not allocate a record ID, increment a revision, or repeat a side effect.
The only permitted posture edges are:

```text
Dormant/ZeroLiveClosed -> ReadyClosed/ZeroLiveClosed
ReadyClosed/ZeroLiveClosed -> Active/Released
Dormant|ReadyClosed/ZeroLiveClosed(fence N) -> Dormant/ZeroLiveClosed(fence N+1)
Active/Released(fence N) -> Dormant/ZeroLiveClosed(fence N+1)
```

`Active/Released` is frozen: it has no same-fence successor and E3 never live-reconfigures a child.
It may support the initial turn and later sequential resumed-turn child processes only through the
same retained sealed capability and live lease; it does not authorize concurrent turns or a second
gateway.
Abandoning a `Dormant` or `ReadyClosed` epoch requires proof that any unreleased child was killed and
the old access boundary is `Revoked`. Following an active epoch requires no live turn child plus
terminal retained-runtime evidence and
a verified revoked old boundary. Either transition to fence N+1 allocates a fresh root,
gateway-session identity, access boundary, handoff, intent, readiness nonce, ACK, and barrier; it
never returns fence N to closed or erases its evidence. Retirement is separate from record revision
and has no successor.

## Canonical encoding, hashes, IDs, and exact validation

Canonical JSON is UTF-8 without BOM or insignificant whitespace. Object keys are recursively sorted
by UTF-8 byte sequence; array order is preserved; integer JSON numbers use their shortest decimal
form; floating-point values are forbidden; and absent optionals are `null`. Duplicate object keys are
rejected before hashing. Struct fields use the exact snake_case names shown. Locally defined enums
use Serde's standard externally tagged representation with the exact Rust-like variant names shown—
unit variants are that exact JSON string and data-bearing variants are a one-key object. No
`rename_all`, alias, or alternate tag/content shape is accepted. Imported canonical types retain
their already-defined encoding; in particular `E2MemberLaunchKindV1` is the existing snake_case
string `"fresh_spawn"` or `"fork"`, not a renamed E3 value.

Strings are not Unicode-normalized. JSON emits quotation mark and reverse solidus as `\"` and `\\`;
backspace, tab, LF, form feed, and CR as `\b`, `\t`, `\n`, `\f`, and `\r`; and every remaining
U+0000–U+001F control as lowercase `\u00xx`. Solidus is never escaped. Every other Unicode scalar is
its original UTF-8, and invalid/unpaired surrogate input is rejected. The canonical golden bytes
`{"a":"\b\t\n\f\r\u0000","é":"雪"}` have SHA-256
`f56970062a492ea5987431971823cbce9374f4aa9f75ebe2e0d78645d2eb6867`.
`Timestamp` is exactly UTC RFC 3339 `YYYY-MM-DDTHH:MM:SS.ffffffZ` with six fractional
digits. UUIDv7 text is lowercase canonical hyphenated form. `bytes_base64` uses RFC 4648 padded base64
with the standard alphabet and no whitespace.

Lowercase SHA-256 domains and preimages are:

| Value | Canonical preimage |
|---|---|
| `identity_hash` | `{"domain":"substrate.e3.config-projection-identity.v1","identity":<identity with identity_hash omitted>}` |
| `subject_hash` | `{"domain":"substrate.e3.config-projection-subject.v1","identity":<identity with series_id and identity_hash omitted>}` |
| `logical.projection_hash` | `{"domain":"substrate.e3.logical-config-projection.v1","projection":<logical with projection_hash omitted>}` |
| `effective.projection_hash` | `{"domain":"substrate.e3.effective-config-projection.v1","projection":<effective with projection_hash omitted>}` |
| `native.projection_hash` | `{"domain":"substrate.e3.native-config-projection.v1","projection":<native with projection_hash omitted>}` |
| Codex launch-plan `plan_hash` | `{"domain":"substrate.e3.codex-launch-plan.v1","plan":<plan with plan_hash omitted>}` |
| `managed_gateway.projection_hash` | `{"domain":"substrate.e3.managed-gateway-projection.v1","projection":<managed_gateway with projection_hash omitted>}` |
| `nonsecret_handoff.projection_hash` | `{"domain":"substrate.e3.nonsecret-handoff-projection.v1","projection":<nonsecret_handoff with projection_hash omitted>}` |
| runtime-support `manifest_hash` | `{"domain":"substrate.e3.runtime-support-manifest.v1","manifest":<runtime support manifest with manifest_hash omitted>}` |
| enforcement-input `enforcement_input_hash` | `{"domain":"substrate.e3.world-fs-enforcement-input.v1","input":<complete input, including user_namespace_requirement, with enforcement_input_hash omitted>}` |
| `e2_enforcement_plan_hash` | `{"discover":<exact ordered E2 paths>,"domain":"substrate.e3.e2-enforcement-plan.v1","execute":<exact ordered E2 paths>,"policy_snapshot_hash":<hash>,"read":<exact ordered E2 paths>,"write":<exact ordered E2 paths>}` |
| `derived_support_ruleset_hash` | `{"domain":"substrate.e3.derived-support-landlock-layer.v1","e2_enforcement_plan_hash":<hash>,"support_discover":<ordered explicitly enumerable E3 subtree paths; no synthetic ancestors>,"support_execute":<ordered paths>,"support_read":<ordered paths>,"support_write":<ordered paths>}` |
| `role_narrowing_ruleset_hash` | `{"child_role":<role>,"discover":<ordered final explicitly enumerable subtree paths; no synthetic ancestors>,"domain":"substrate.e3.role-narrowing-landlock-layer.v1","execute":<ordered final role paths>,"read":<ordered final role paths>,"write":<ordered final role paths>}` |
| `effective_landlock_hash` | `{"derived_support_ruleset_hash":<hash>,"domain":"substrate.e3.effective-landlock-intersection.v1","role_narrowing_ruleset_hash":<hash>}` |
| denied-control target `target_hash` | `{"domain":"substrate.e3.denied-control-probe-target.v1","target":<target with target_hash omitted>}` |
| `denied_control_probe_hash` | `{"domain":"substrate.e3.denied-control-probes.v1","probes":<the ordered complete E3DeniedControlProbeV1 array>}` |
| child-security `attestation_hash` | `{"attestation":<complete child security attestation, including user_namespace, with attestation_hash omitted>,"domain":"substrate.e3.child-security-attestation.v1"}` |
| setup-ready `attestation_hash` | `{"attestation":<setup attestation with attestation_hash omitted>,"domain":"substrate.e3.codex-setup-ready.v1"}` |
| native-source `manifest_hash` | `{"domain":"substrate.e3.native-projection-source-manifest.v1","manifest":<manifest with manifest_hash omitted>}` |
| native-realization `manifest_hash` | `{"domain":"substrate.e3.native-projection-realization-manifest.v1","manifest":<manifest with manifest_hash omitted>}` |
| `record_hash` | `{"domain":"substrate.e3.agent-config-projection-record.v1","record":<record with record_hash omitted>}` |
| `release_hash` | `{"activation_ack_ref":<ack ref>,"closed_record_ref":<ref>,"domain":"substrate.e3.config-projection-release.v1","fence_id":<id>}` |

Native-file and `RuntimeSupportFileV1.sha256` values are ordinary lowercase SHA-256 over the exact
file bytes, not canonical JSON hashes. `fence_id` is `cpf_<lowercase UUIDv7>`. A reference is valid only when the configured
accepted-home store resolves exactly one immutable record whose store/series/record/revision fields
equal the ref and whose recomputed `record_hash` equals the ref. A caller-supplied filesystem path is
never part of reference resolution.

Validation recomputes all nested hashes, decodes every native file, checks length/hash/mode/path,
re-renders native bytes from the effective/gateway values and requires byte equality, validates the
E2 record/ref/cap/snapshot through E2's existing authenticated read capability, and compares every
overlapping session/participant/bootstrap/backend/world/generation/policy field. It does not write,
repair, migrate, or synthesize E2 state. Equal hashes do not permit object substitution: every ID,
revision, enum, ordered list, directory identity, artifact identity, E2 ref, gateway ref, handoff ref,
and byte sequence is equality-compared. Lexically equal paths, equal content in another series,
current parent policy, a newer E2 commitment, a different world generation, or an equivalent gateway
is not a substitute.

## Accepted-home persistence and recovery

The registry objects are strict and deny unknown fields:

```rust
struct ConfigProjectionStoreV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    accepted_home: CanonicalDirectoryV1,
    created_at: Timestamp,
    store_hash: String,
}

struct InstalledAcceptedHomeBootstrapRecordV1 {
    schema_version: u32, // exactly 1
    install_bootstrap_carrier: String,
    host_context_commitment: String,
    intended_account: String,
    intended_uid: u64,
    intended_gid: u64,
    accepted_home: CanonicalDirectoryV1,
    installed_at: Timestamp,
    record_hash: String,
}

struct InstalledAcceptedHomeBootstrapHeadV1 {
    schema_version: u32, // exactly 1
    head_record_hash: String,
    predecessor_head_hash: Option<String>,
    updated_at: Timestamp,
    head_hash: String,
}

// Sealed, process-local, non-serializable and non-cloneable.
struct ConfiguredAcceptedHomeAuthorityV1 {
    installed_record: InstalledAcceptedHomeBootstrapRecordV1,
    accepted_home_descriptor: HeldDirectoryDescriptor,
}

struct EffectiveSubstrateConfigSourceV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    accepted_home: CanonicalDirectoryV1,
    workspace_root: CanonicalDirectoryV1,
    values: E3EffectiveConfigInputV1,
    ordered_explain_origins: Vec<E3ConfigExplainOriginV1>,
    source_revision: String,
    source_hash: String,
}

struct E3EffectiveConfigInputV1 {
    llm_enabled: bool,
    agents_enabled: bool,
    world_enabled: bool,
    default_execution_scope: String,
    default_cli_mode: String,
    managed_gateway_enabled: bool,
    managed_gateway_mode: String, // exactly "in_world"
    default_backend_id: String,
}

struct E3ConfigExplainOriginV1 {
    key: String,
    source_kind: E3ConfigExplainOriginKindV1,
    source_location: Option<E3ConfigExplainFileV1>,
}

enum E3ConfigExplainOriginKindV1 {
    Default,
    GlobalPatch,
    WorkspacePatch,
    OverrideEnv,
    CliFlag,
}

struct E3ConfigExplainFileV1 {
    source_root: CanonicalDirectoryV1,
    source_relative_path: String,
    source_bytes_sha256: String,
}

// Shell-private host-session-authority facade type; fields are inaccessible outside the facade.
struct OpenedBootstrapConfigSourceV1<'authority> {
    trusted_root: &'authority TrustedAuthorityRoot,
    trusted_file: Option<TrustedFile>,
    source_bytes: Option<Vec<u8>>,
}

// Shell-private, non-serializable transaction inputs; raw bytes are never persisted.
enum E3PinnedConfigPatchSourceV1<'authority> {
    Global {
        opened: OpenedBootstrapConfigSourceV1<'authority>,
        source_root: CanonicalDirectoryV1,
        source_relative_path: String, // exactly "config.yaml"
        source_bytes_sha256: Option<String>, // null iff opened has absent posture
    },
    Workspace {
        source_root: CanonicalDirectoryV1,
        held_source_root: HeldDirectoryDescriptor,
        source_relative_path: String, // exactly ".substrate/workspace.yaml"
        held_file: HeldRegularFileDescriptor,
        file_device_id: u64,
        file_inode: u64,
        file_byte_length: u64,
        source_bytes: Vec<u8>,
        source_bytes_sha256: String,
    },
}

// Shell-private and consumed before immutable source publication.
struct E3EffectiveConfigResolutionSnapshotV1<'authority> {
    effective: SubstrateConfig,
    explain: ConfigExplainV1,
    global_patch: E3PinnedConfigPatchSourceV1<'authority>,
    workspace_patch: E3PinnedConfigPatchSourceV1<'authority>,
}

struct TrustedRuntimeArtifactManifestV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    manifest_id: String,
    revision: u64, // exactly 1
    entries: Vec<RuntimeArtifactManifestEntryV1>,
    created_at: Timestamp,
    manifest_hash: String,
}

struct RuntimeArtifactManifestEntryV1 {
    manifest_entry_id: String,
    authority_role: RuntimeArtifactAuthorityRoleV1,
    configured_absolute_path: String,
    device_id: u64,
    inode: u64,
    mode: u32,
    owner_uid: u64,
    byte_length: u64,
    sha256: String,
    installer_source_ref: InstallerArtifactSourceRefV1,
    provenance: RuntimeArtifactProvenanceV1,
    runtime_support: E3RuntimeSupportManifestV1,
    entry_hash: String,
}

enum RuntimeArtifactAuthorityRoleV1 {
    Codex0125,
    ManagedGateway,
    WorldEntryWrapper,
}

struct InstallerArtifactSourceRefV1 {
    source_store_id: String,
    source_record_id: String,
    revision: u64,
    record_hash: String,
}

struct InstallerArtifactSourceStoreV1 {
    schema_version: u32, // exactly 1
    source_store_id: String,
    root: CanonicalDirectoryV1,
    created_at: Timestamp,
    store_hash: String,
}

struct InstallerArtifactSourceRecordV1 {
    schema_version: u32, // exactly 1
    source_store_id: String,
    source_record_id: String,
    source_stream: InstallerArtifactSourceStreamV1,
    revision: u64,
    predecessor_ref: Option<InstallerArtifactSourceRefV1>,
    build_input: InstallerArtifactBuildInputV1,
    entries: Vec<InstallerArtifactSourceEntryV1>,
    created_at: Timestamp,
    record_hash: String,
}

enum InstallerArtifactSourceStreamV1 { SubstrateSourceBuild, Codex0125OfficialArchive }

enum InstallerArtifactBuildInputV1 {
    SubstrateSourceBuild {
        source_commit: String,
        source_tree: String,
        cargo_lock_sha256: String,
        rustc_version: String,
        target_triple: String,
        profile: String,
    },
    CodexOfficialArchive {
        version: String, // exactly "0.125.0"
        target_triple: String, // exactly "x86_64-unknown-linux-musl"
        archive_name: String,
        archive_url: String,
        archive_sha256: String,
        archive_entry_path: String,
        extracted_executable_sha256: String,
    },
}

struct InstallerArtifactSourceEntryV1 {
    component: String,
    installed_absolute_path: String,
    device_id: u64,
    inode: u64,
    file_type: String, // exactly "regular"
    mode: u32,
    owner_uid: u64,
    byte_length: u64,
    sha256: String,
    runtime_support: E3RuntimeSupportManifestV1,
    entry_hash: String,
}

struct InstallerArtifactSourceHeadV1 {
    schema_version: u32, // exactly 1
    source_store_id: String,
    source_stream: InstallerArtifactSourceStreamV1,
    head_ref: InstallerArtifactSourceRefV1,
    head_revision: u64,
    predecessor_head_hash: Option<String>,
    updated_at: Timestamp,
    head_hash: String,
}

struct ConfigProjectionSubjectBindingV1 {
    schema_version: u32, // exactly 1
    subject_hash: String,
    identity: ConfigProjectionIdentityV1,
    series_id: String,
    created_at: Timestamp,
    binding_hash: String,
}

struct ConfigProjectionHeadV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    head_ref: ConfigProjectionRefV1,
    head_revision: u64,
    predecessor_head_hash: Option<String>,
    updated_at: Timestamp,
    head_hash: String,
}

enum ConfigProjectionConsumerKindV1 {
    MemberDispatchV2,
    WorldRuntimeAdapterV3,
}

enum ConfigProjectionConsumerLeasePostureV1 { Held, Released }

struct ConfigProjectionConsumerLeaseV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    consumer_id: String,
    consumer_kind: ConfigProjectionConsumerKindV1,
    revision: u64,
    predecessor_lease_hash: Option<String>,
    acquired_projection_ref: ConfigProjectionRefV1,
    posture: ConfigProjectionConsumerLeasePostureV1,
    acquired_at: Timestamp,
    released_at: Option<Timestamp>,
    lease_hash: String,
}

enum ConfigProjectionEvidenceKindV1 { SecretHandoff, ConsumerLease }

struct ConfigProjectionEvidenceHeadV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    evidence_kind: ConfigProjectionEvidenceKindV1,
    series_id: Option<String>,
    evidence_owner_id: String,
    head_revision: u64,
    head_object_hash: String,
    predecessor_head_hash: Option<String>,
    updated_at: Timestamp,
    evidence_head_hash: String,
}

struct E3TerminalChildQuiescenceEvidenceRefV1 {
    authority_store_id: String,
    series_id: String,
    evidence_id: String,
    evidence_hash: String,
}

struct E3TerminalChildQuiescenceEvidenceV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    evidence_id: String,
    final_projection_ref: ConfigProjectionRefV1,
    world_id: String,
    world_generation: u64,
    ordered_terminal_processes: Vec<E3TerminalProcessObservationV1>,
    ordered_empty_cgroups: Vec<E3TerminalCgroupQuiescenceV1>,
    observed_at: Timestamp,
    evidence_hash: String,
}

struct E3KernelEffectIntentRefV1 {
    authority_store_id: String,
    effect_intent_id: String,
    intent_hash: String,
}

struct E3KernelEffectIntentV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    effect_intent_id: String,
    preparation_id: String,
    fence_id: String,
    effect: E3KernelEffectKindV1,
    created_at: Timestamp,
    intent_hash: String,
}

enum E3KernelEffectKindV1 {
    CreateChildCgroup {
        cgroup_registration_id: String,
        role: E3TerminalProcessRoleV1,
        parent_cgroup: CanonicalCgroupIdentityV1,
        child_component: String,
        expected_relative_path: String,
    },
    InstallGatewayBoundary {
        access_boundary_id: String,
        network_namespace_inode: u64,
        table_name: String,
        chain_name: String,
    },
}

struct E3KernelEffectResolutionV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    resolution_id: String,
    effect_intent_ref: E3KernelEffectIntentRefV1,
    disposition: E3KernelEffectResolutionDispositionV1,
    observed_cgroup: Option<CanonicalCgroupIdentityV1>,
    observed_nftables_table_handle: Option<u64>,
    resolved_at: Timestamp,
    resolution_hash: String,
}

enum E3KernelEffectResolutionDispositionV1 { NoEffectObserved, RevertedAndQuiescent }

struct E3ChildProcessRegistrationV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    registration_id: String,
    cgroup_registration_id: String,
    cgroup_registration_hash: String,
    fence_id: String,
    role: E3TerminalProcessRoleV1,
    pid: u32,
    pid_start_time_ticks: u64,
    process_cgroup: CanonicalCgroupIdentityV1,
    kernel_boot_id: String,
    parent_service_instance_id: String,
    registered_at: Timestamp,
    registration_hash: String,
}

struct E3ChildCgroupRegistrationV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    cgroup_registration_id: String,
    kernel_effect_intent_ref: E3KernelEffectIntentRefV1,
    fence_id: String,
    turn_id: Option<String>,
    role: E3TerminalProcessRoleV1,
    cgroup: CanonicalCgroupIdentityV1,
    kernel_boot_id: String,
    registered_at: Timestamp,
    cgroup_registration_hash: String,
}

struct E3TerminalProcessObservationV1 {
    registration_id: String,
    registration_hash: String,
    role: E3TerminalProcessRoleV1,
    pid: u32,
    pid_start_time_ticks: u64,
    observation: E3TerminalProcessObservationKindV1,
}

enum E3TerminalProcessObservationKindV1 {
    ParentWaitid {
        terminal_wait_status: i32,
    },
    RecoveryObservedTerminal {
        original_service_instance_id: String,
        recovery_service_instance_id: String,
        pidfd_open_errno: Option<i32>, // exactly ESRCH when present
        pidfd_kill_errno: Option<i32>, // null for success/reuse; exactly ESRCH for a terminal race
        pidfd_became_readable: Option<bool>, // exactly true when present
        waitid_errno: Option<i32>, // exactly ECHILD when present
        proc_identity: E3RecoveryProcIdentityV1,
    },
}

enum E3TerminalProcessRoleV1 { ManagedGateway, Codex, ReadinessProbe }

enum E3RecoveryProcIdentityV1 {
    Absent,
    PidReused { observed_pid_start_time_ticks: u64 },
}

struct E3TerminalCgroupQuiescenceV1 {
    cgroup_registration_id: String,
    cgroup_registration_hash: String,
    role: E3TerminalProcessRoleV1,
    cgroup: CanonicalCgroupIdentityV1,
    cgroup_events_sha256: String,
    cgroup_procs_sha256: String,
    populated: bool, // exactly false
    ordered_live_pids: Vec<u32>, // exactly []
}

struct ConfigProjectionRetirementV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    series_id: String,
    final_head_ref: ConfigProjectionRefV1,
    terminal_child_evidence_ref: E3TerminalChildQuiescenceEvidenceRefV1,
    revoked_boundary_ref: GatewayAccessBoundaryRefV1,
    consumer_lease_count: u64, // exactly 0
    retired_at: Timestamp,
    retirement_hash: String,
}
```

The eight keys occur exactly once in `ordered_explain_origins`, in this order:
`llm.enabled`, `llm.gateway.enabled`, `llm.gateway.mode`,
`llm.routing.default_backend`, `agents.enabled`, `agents.defaults.execution.scope`,
`agents.defaults.cli.mode`, and `world.enabled`. Every corresponding baseline explain entry must have
merge strategy `replace` and exactly one source. Baseline layer strings map one-to-one as
`default -> Default`, `global_patch -> GlobalPatch`, `workspace_patch -> WorkspacePatch`,
`override_env -> OverrideEnv`, and `cli_flag -> CliFlag`; any other spelling, missing key, repeated
key/source, or extra source is `Malformed`.

`Default`, `OverrideEnv`, and `CliFlag` require `source_location = null`, matching the landed
path-less explain form. `GlobalPatch` requires the exact `config.yaml` byte vector borrowed from a
sealed `OpenedBootstrapConfigSourceV1` in present posture; `WorkspacePatch` requires
the exact `.substrate/workspace.yaml` byte vector retained beneath the selected workspace descriptor.
Those two variants require `source_location` to equal the retained source root and normalized
relative path and `source_bytes_sha256` to be ordinary SHA-256 of the same byte vector passed to
`parse_config_patch_yaml`. The explain path must resolve to that expected display path but is never
opened. File device/inode/length and held descriptors are transaction evidence and are not serialized
into the immutable source object. Conversely a patch origin with null location, a non-patch origin
with a location, an origin naming an absent retained patch, or two retained patches claiming the same
origin is malformed. Because E3 supplies an empty `CliConfigOverrides`, `CliFlag` is always malformed
for this object. An `OverrideEnv` result is representable and hash-stable as location-null and must
come from the transaction's one `parse_env_overrides` snapshot; E3 never rereads or records the
environment variable name or ambient value separately.

Values are copied only from the same returned `SubstrateConfig`: `llm_enabled` from `llm.enabled`,
`managed_gateway_enabled` from `llm.gateway.enabled`, `managed_gateway_mode` from
`llm.gateway.mode`, `default_backend_id` from `llm.routing.default_backend`, `agents_enabled` from
`agents.enabled`, `default_execution_scope` from `agents.defaults.execution.scope`,
`default_cli_mode` from `agents.defaults.cli.mode`, and `world_enabled` from `world.enabled`, using
the exact snake-case enum spellings. E3 activation is eligible only when `llm_enabled`,
`managed_gateway_enabled`, `agents_enabled`, and `world_enabled` are all `true`; gateway mode is
`in_world`; default execution scope and the selected V3 placement are both `world`; default CLI mode
and the selected placement CLI mode are both `persistent`; and the default backend plus selected
backend are both exactly `cli:codex-world`. Any false bit or unequal selector is typed
`UnsupportedConfiguration` before publication or gateway side effects. No other config key can
enable, disable, or repair this predicate.

The subject hash is recomputed from `binding.identity` after omitting `series_id` and `identity_hash`;
`binding.series_id` must equal `binding.identity.series_id`. A subject binding cannot carry an
extensible or caller-supplied identity map. A held lease has `released_at = null`; its sole successor
is one `Released` revision with the same consumer and acquired ref plus non-null `released_at`. A
lease never expires by clock or startup; crash recovery may publish its release only from exact
durable evidence that the carrier was never made observable or that the bound consumer is terminal.
Evidence heads are mutable CAS pointers only: `SecretHandoff` names a handoff ID and its immutable
`ConfigProjectionSecretHandoffRevisionV1.revision_hash` with `series_id = null`, while
`ConsumerLease` names the exact non-null series plus `consumer_id` and its immutable lease-revision
hash. The kind, series, owner, and path must agree exactly. Handoff history is audited through each
retained immutable revision wrapper's exact `predecessor_ref`; an overwritten head's former bytes or
`predecessor_head_hash` is never required to reconstruct Prepared -> Delivered -> Consumed. The head
predecessor hash exists only to validate the mutable-pointer CAS history while its expected predecessor
is current. The additional lowercase SHA-256 preimages are:

| Value | Canonical preimage |
|---|---|
| `store_hash` | `{"domain":"substrate.e3.config-projection-store.v1","store":<store with store_hash omitted>}` |
| installed-home `record_hash` | `{"domain":"substrate.e3.installed-accepted-home-bootstrap.v1","record":<installed-home record with record_hash omitted>}` |
| installed-home `head_hash` | `{"domain":"substrate.e3.installed-accepted-home-bootstrap-head.v1","head":<installed-home head with head_hash omitted>}` |
| effective-config `source_hash` | `{"domain":"substrate.e3.effective-substrate-config-source.v1","source":<source with source_hash omitted>}` |
| inventory `source_hash` | `{"domain":"substrate.e3.agent-inventory-source.v1","source":<source with source_hash omitted>}` |
| artifact `entry_hash` | `{"domain":"substrate.e3.runtime-artifact-entry.v1","entry":<entry with entry_hash omitted>}` |
| artifact `manifest_hash` | `{"domain":"substrate.e3.runtime-artifact-manifest.v1","manifest":<manifest with manifest_hash omitted>}` |
| installer `store_hash` | `{"domain":"substrate.e3.installer-artifact-source-store.v1","store":<installer store with store_hash omitted>}` |
| installer entry `entry_hash` | `{"domain":"substrate.e3.installer-artifact-source-entry.v1","entry":<installer entry with entry_hash omitted>}` |
| installer record `record_hash` | `{"domain":"substrate.e3.installer-artifact-source-record.v1","record":<installer record with record_hash omitted>}` |
| installer head `head_hash` | `{"domain":"substrate.e3.installer-artifact-source-head.v1","head":<installer head with head_hash omitted>}` |
| `binding_hash` | `{"binding":<binding with binding_hash omitted>,"domain":"substrate.e3.config-projection-subject-binding.v1"}` |
| `head_hash` | `{"domain":"substrate.e3.config-projection-head.v1","head":<head with head_hash omitted>}` |
| `lease_hash` | `{"domain":"substrate.e3.config-projection-consumer-lease.v1","lease":<lease with lease_hash omitted>}` |
| `evidence_head_hash` | `{"domain":"substrate.e3.config-projection-evidence-head.v1","head":<evidence head with evidence_head_hash omitted>}` |
| kernel-effect `intent_hash` | `{"domain":"substrate.e3.kernel-effect-intent.v1","intent":<intent with intent_hash omitted>}` |
| kernel-effect `resolution_hash` | `{"domain":"substrate.e3.kernel-effect-resolution.v1","resolution":<resolution with resolution_hash omitted>}` |
| child-cgroup `cgroup_registration_hash` | `{"cgroup_registration":<registration with cgroup_registration_hash omitted>,"domain":"substrate.e3.child-cgroup-registration.v1"}` |
| child `registration_hash` | `{"domain":"substrate.e3.child-process-registration.v1","registration":<registration with registration_hash omitted>}` |
| terminal-child `evidence_hash` | `{"domain":"substrate.e3.terminal-child-quiescence.v1","evidence":<terminal-child evidence with evidence_hash omitted>}` |
| `retirement_hash` | `{"domain":"substrate.e3.config-projection-retirement.v1","retirement":<retirement with retirement_hash omitted>}` |

### HSA projection namespace and cross-process lock protocol

The projection registry remains at
`<accepted-home>/authority-v1/agent-config-projection-v1/`. This is an intentional new member of the
existing closed `authority-v1` namespace, not a second authority hierarchy. A later E3-B storage
implementation must extend exactly
`crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`,
specifically `StoreLayout::validate_closed_layout`, to recognize only the literal top-level directory
`agent-config-projection-v1`. It must preserve rejection of every other unknown entry: weakening the
closed-layout check, accepting arbitrary entries, or adding a wildcard is forbidden. That product
path is not changed or admitted here. The same fresh E3-B admission must also bind only
`crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`,
specifically `validate_e2_rm_authority_manifest`, for the identical literal recognition so the
landed read-only snapshot remains compatible with the expanded closed HSA namespace. This is HSA
namespace-layout compatibility, not ownership of or permission to change E2-RM authority data, read
semantics, history, reconciliation, schema, or namespace. Every other product path required by this
corrected ownership must likewise be bound before implementation begins.

Every E3 read, publication, CAS, recovery, retirement, and GC transaction uses this mandatory
cross-process order:

1. Open and validate the accepted HSA authority root through the existing trusted-root capability.
2. Acquire the existing exclusive `authority-v1/lock/root.lock`.
3. While holding that parent lock, revalidate the accepted HSA root identity and the exact closed
   `authority-v1` layout.
4. Only then open and exclusively acquire the child registry lock at
   `authority-v1/agent-config-projection-v1/lock`.
5. Perform the complete E3 transaction while retaining both locks and the trusted-root capability.
6. Before release, revalidate the HSA root, the exact closed layout, all applicable namespace
   manifests, and every affected E3 object; then release the child lock before the parent lock.

The order is always parent root lock before E3 child lock. Reverse acquisition, child-only mutation,
lock upgrading, and releasing the parent lock during an E3 transaction are forbidden. E2-RM reads
and ordinary HSA operations that acquire the existing parent root lock are thereby serialized
against E3 publication and recovery. E3 does not modify E2, E2-RM, B1, B2.1, retained-worker, or any
other namespace authority, and reconciliation may never mutate another owner's namespace. Process
death releases kernel locks but cannot make incomplete E3 state authoritative; the next E3
transaction must acquire both locks in this order and either recover its own namespace completely or
fail closed. Later E3-B proof must include every publication/recovery crash boundary and concurrent
HSA/E2-RM/E3 transactions, including successful landed E2-RM reads after E3 namespace creation plus
deadlock, partial-state, unknown-entry, and cross-owner mutation negatives.

The exact E3-B cross-crate bridge is fixed rather than left to implementation choice:

- On `#[cfg(target_os = "linux")]`, `crates/config-projection/src/registry.rs` owns public trait
  `ConfigProjectionHsaAuthorityV1: Send + Sync`. Its sole operation is object-safe
  `with_locked_parent`, taking one
  `&mut dyn for<'fd> FnMut(BorrowedFd<'fd>) -> Result<(), ConfigProjectionFailureV1>` and returning
  `Result<(), ConfigProjectionFailureV1>`. It must invoke that callback exactly once while the shell
  parent transaction is live. `ConfigProjectionRegistryV1::open` accepts an
  `Arc<dyn ConfigProjectionHsaAuthorityV1>` and retains no other HSA access. A registry operation
  that returns a value captures that value in its own caller frame; the callback itself remains the
  object-safe unit-returning boundary.
- On that same Linux gate,
  `crates/shell/src/execution/agent_runtime/host_session_authority/facade.rs` owns the public,
  non-serializable `OpenedConfigProjectionHsaAuthorityV1`, its sole constructor
  `OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home`, and its implementation of
  `ConfigProjectionHsaAuthorityV1`. The constructor accepts only a sealed
  `&ConfiguredAcceptedHomeAuthorityV1`; it accepts no pathname, environment value, request field, or
  raw descriptor. It calls `ConfiguredAcceptedHomeAuthorityV1::revalidate`, reads only that type's
  exact `accepted_home` and `intended_uid` accessors, opens the named root through existing
  `TrustedAuthorityRoot::open_for_owner` after a checked `u64`-to-`uid_t` conversion, exact-compares
  physical path/device/inode, revalidates the
  still-held configured descriptor, and then retains the `TrustedAuthorityRoot` for the bridge
  lifetime. Private fields prevent any other production construction.
- `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`
  owns private `ConfigProjectionHsaParentTransactionV1` with only `begin`, `authority_fd`, and
  `finish`, plus `with_opened_config_projection_hsa_parent`; `store.rs` owns only the corresponding
  `pub(super)` `with_config_projection_hsa_parent` forwarding operation. `begin` composes the current
  trusted-directory entry guards,
  `StoreLayout`, strict versioned-root decoding, matching-marker checks, and
  `TrustedOwnedFileLock`: it opens the fixed `authority-v1/lock/root.lock`, acquires it exclusively,
  revalidates the accepted root and named authority/lock objects, runs
  `StoreLayout::validate_closed_layout` and `validate_e2_rm_authority_manifest`, rejects HSA temp or
  legacy state rather than reconciling it, and snapshots the exact root plus every non-E3 authority
  entry. `finish` repeats those checks and requires the root and every non-E3 entry to be unchanged;
  the only permitted top-level transition is absent-to-one-directory for the literal E3 child, or
  retention of that child's exact directory identity. It never calls the landed E2-RM read wrapper,
  whose deliberately read-only final metadata equality would reject a real E3 mutation, and it does
  not alter that wrapper's semantics.
- Linux-only `TrustedDirectory::borrow_fd` is the only new `trusted_fs.rs` allowance and remains
  `pub(crate)`. The sole value crossing into `config-projection` is the callback-bounded
  `BorrowedFd<'fd>` for the already-open exact `authority-v1` directory. No `TrustedAuthorityRoot`,
  `TrustedDirectory`, `TrustedFile`, parent-lock descriptor, shell-private directory identity, owned
  descriptor, or path crosses the boundary. The registry must not duplicate, persist, return, or
  expose the borrowed descriptor; its lifetime prevents a safe retained reference after the
  callback.

Inside that callback, `crates/config-projection/src/registry.rs` alone opens or creates the literal
`agent-config-projection-v1` child descriptor-relatively, opens or creates and validates its literal
`lock`, acquires the child `flock(LOCK_EX)`, and constructs private
`ConfigProjectionChildTransactionV1`. Every registry operation runs through that type's private
`begin`/`finish` wrapper. For both `Ok` and ordinary `Err`, the wrapper preserves the operation
result, performs the required child namespace/object/readback revalidation, and drops the child lock
before returning from the callback; the shell wrapper then performs its root/layout/non-E3
revalidation and only afterward drops the parent lock. A verification failure overrides an otherwise
successful or domain-error result and fails closed. During panic unwinding, the child transaction's
RAII guard is in the inner config-projection callback frame and is therefore dropped before the
shell parent transaction's RAII guard in the outer frame; neither destructor publishes or recovers
state. The next transaction may recover only recognized E3-owned state while holding both locks.
This supplies safe child-before-parent release during unwinding without pretending that fallible
post-operation revalidation can execute after a panic.

The later `WorldService::new_linux` consumer constructs and retains one
`Arc<ConfiguredAcceptedHomeAuthorityV1>`, calls
`OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home` with that sealed value, and
passes the resulting `Arc<dyn ConfigProjectionHsaAuthorityV1>` together with the same configured
authority into `AgentConfigProjectionServiceV1::new`. No request handler, registry method, gateway,
or member caller may construct or replace the bridge.

The only E3 authority root is:

```text
<accepted-home>/authority-v1/agent-config-projection-v1/
  store.json
  lock
  inputs/effective-config/<source-hash>.json
  inputs/agent-inventory/<source-hash>.json
  runtime-artifacts/<manifest-id>/<revision-20d>.json
  native-sources/<series-id>/<fence-id>/source-manifest.json
  native-sources/<series-id>/<fence-id>/codex-home/config.toml
  native-sources/<series-id>/<fence-id>/system-empty/
  subjects/<subject-hash>.json
  series/<series-id>/head.json
  series/<series-id>/records/<revision-20d>-<record-id>.json
  gateways/<gateway-instance-id>.json
  gateway-boundaries/<access-boundary-id>/<revision-20d>.json
  gateway-intents/<intent-id>.json
  gateway-launch-inputs/<launch-input-id>.json
  gateway-acks/<ack-id>.json
  handoffs/<handoff-id>/head.json
  handoffs/<handoff-id>/revisions/<state-revision-20d>.json
  leases/<series-id>/<consumer-id>/head.json
  leases/<series-id>/<consumer-id>/revisions/<revision-20d>.json
  diagnostics/<diagnostic-id>.json
  kernel-effects/intents/<effect-intent-id>.json
  kernel-effects/resolutions/<resolution-id>.json
  child-cgroups/<series-id>/<cgroup-registration-id>.json
  child-processes/<series-id>/<registration-id>.json
  terminal-child-evidence/<series-id>/<evidence-id>.json
  retirement/<series-id>.json
```

The configured capability has exactly one bootstrap source:

```text
/var/lib/substrate/install-bootstrap-authority-v1/
  lock
  records/<record-hash>.json
  active.json
```

During provisioning, `world-provision.sh` already holds the canonical
`InstallBootstrapContextCarrierV1` and intended account/UID. E3 resolves that exact account once
through the local passwd database, requires its UID to equal the carrier's intended UID, records its
primary GID, and rejects a missing, duplicate, or changed account/UID/GID result. The E3 lifecycle installation opens the
carrier's `host_substrate_home` with no-follow component traversal, validates the existing private
home posture, records its `CanonicalDirectoryV1`, strictly decodes/re-encodes and validates the
carrier, and requires the carrier commitment/account/UID plus resolved primary GID to equal its explicit fields. Under the
root-owned mode-`0600` lock it first-writer-publishes the canonical mode-`0640`
`root:substrate` record at its content-addressed `records/<record-hash>.json`, file- and
directory-fsyncs it, and expected-bytes-CASes `active.json`; the head stores the exact prior head hash.
Recognized temporaries are only `.e3-bootstrap-tmp.<lowercase-uuidv7>.<record|head>` in the matching
directory and use the same no-replace/CAS, readback, recovery, and conflict rules as the installer
stores. Records are immutable and retained. A changed accepted home creates a new record/head and
cannot retarget an existing projection series.

`ConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority` is the only production
constructor. At `WorldService::new` it opens those fixed root-owned files descriptor-relatively,
strictly validates the active head/record/hash and canonical carrier, exact-compares the carrier's
home, commitment, account, and UID plus the current passwd primary GID to the record, opens the recorded accepted home no-follow, and
requires physical equality with `accepted_home`. It ignores `ExecuteRequest`, member carrier,
`SUBSTRATE_HOME`, `HOME`, cwd, CLI, and caller paths. The resulting sealed held descriptor is threaded
through `WorldService::new_linux`, `AgentConfigProjectionServiceV1`, and member/gateway activation;
failure leaves non-E3 service paths available but makes every V2/E3 request
`UnsupportedConfiguration`. Test construction exists only as
`ConfiguredAcceptedHomeAuthorityV1::from_record_for_test` and must pass the same record and physical
checks.

The projection registry root is reached only from that configured accepted-home bound capability and
only after the parent HSA root lock has been acquired and the closed layout revalidated as specified
above. Its already-open `CanonicalDirectoryV1` is revalidated, then every descendant component is
opened descriptor-relatively with Linux `openat2` resolution flags `RESOLVE_BENEATH |
RESOLVE_NO_MAGICLINKS | RESOLVE_NO_SYMLINKS | RESOLVE_NO_XDEV`; final opens also use `O_NOFOLLOW`.
Registry directories are owner UID, mode `0700`; regular authority files and the child `lock` are
owner UID, mode `0600`, link count one, with no unexpected ACL or xattr. The held root descriptor's
device/inode must remain equal to `accepted_home`. The regular child `lock` file is opened from that
descriptor and held with `flock(LOCK_EX)` while the already-held parent root lock remains held for the
complete read/validate/recovery/publication transaction. A caller cannot select or override a home,
store, series directory, or active-head path. Root replacement, descriptor rebind, unexpected
metadata, mount crossing, symlink, non-regular file, link-count anomaly, or physical identity change
fails before a read or write consumes the replacement.

The accepted-home artifact manifest may import only immutable records from these two installer-owned
source stores:

```text
/var/lib/substrate/runtime-artifacts-v1/
  source-store.json
  lock
  records/substrate-source-build/<revision-20d>-<source-record-id>.json
  heads/substrate-source-build.json

/var/lib/substrate/world-deps/runtime-artifacts-v1/
  source-store.json
  lock
  records/codex-0.125.0-x86_64-unknown-linux-musl/<revision-20d>-<source-record-id>.json
  heads/codex-0.125.0-x86_64-unknown-linux-musl.json
```

`source-store.json`, source records, and heads use the strict schemas, canonical encoding, domains,
IDs, and hashes above. The Substrate stream record has the
`SubstrateSourceBuild` input and exactly two entries, ordered
`[substrate-gateway,substrate-world-entry]`, for
`/usr/local/lib/substrate/e3/substrate-gateway` and
`/usr/local/lib/substrate/e3/substrate-world-entry`. The Codex stream record has the `CodexOfficialArchive` input
and exactly one `codex` entry for
`/var/lib/substrate/world-deps/codex-runtime/bin/codex`. Every build-input field, component, installed
path, physical identity, mode/owner/length, executable SHA-256, entry hash, predecessor ref, record
hash, source-store binding, and complete descriptor-derived `runtime_support` are mandatory. Unknown components, duplicate entries, missing physical
identity, an archive digest substituted for the extracted executable digest, or a build/source field
that disagrees with provenance is malformed or wrong-binding. The source entry's `entry_hash`
authenticates its `runtime_support` because that nested value is present while only `entry_hash` is
omitted from the entry-hash preimage.

In E3-D, `install_linux_managed_state` invokes E3-C's landed helper to publish a new immutable
Substrate record only after building, installing, and descriptor-readback-validating both real
entries against their exact build provenance. No Substrate artifact record or head may be published
or promoted before both entries pass those checks. A V1 artifact, placeholder ELF, invented
provenance, partial manifest, or publisher success that silently omits either required artifact is
never a substitute. The rendered Codex installer does the same only after official-archive
verification, unique-entry extraction, install, and installed-descriptor readback. A source record's
`predecessor_ref` is null at revision 1 and thereafter equals the exact prior head ref; its revision
is prior plus one. The stream head advances by expected-bytes CAS and records its own predecessor-head
hash. Exact retry is byte-equal. A different valid source build or installed byte sequence creates a
new immutable record and head revision, and any accepted projection using it creates a new projection
series; no fixed immutable filename blocks an upgrade and no old record is rewritten. A later gateway
replacement must establish its matching valid source record before the replacement bytes become
E3-eligible; a stale record cannot authorize different bytes.

Before E3-D's real Substrate artifact producers and integration exist, the missing installed
artifacts and absent valid Substrate source head have the explicit disposition `Unavailable`.
Ordinary non-E3 provisioning may remain available, but it cannot report successful E3 artifact
publication or E3 readiness. Any invocation that actually requires E3 Substrate artifact publication
must fail closed. E3-C exercises its schemas, validators, import path, and publication/recovery
helpers with valid bounded test inputs; those inputs are never production artifact authority or
installed-runtime readiness evidence, and E3-C completion does not require executing or installing
the E3-D-owned wrapper.

Both source directory chains are `root:substrate` mode `0750`, non-group/other-writable, and opened
no-follow. `source-store.json`, records, heads, and `lock` are `root:substrate` mode `0640`, link count
one; records are immutable first-writer objects. A root publisher opens `lock` read/write and holds
exclusive `flock(LOCK_EX)`; publication writes only
`.e3-artifact-tmp.<lowercase-uuidv7>.<store|record|head>` mode `0640` in the target directory,
file-fsyncs, uses no-replace for immutable objects or exact expected-bytes CAS for a head,
directory-fsyncs leaf-to-root, and exact-readback-validates. Recovery may complete only one valid
recognized temp with its still-current predecessor; unequal final/temp, two contenders, an unknown
temporary, stale predecessor/inode, or ambiguous install state fails without deletion or import.
The non-root shell opens that same no-follow lock `O_RDONLY|O_CLOEXEC`, validates the exact
root/group/mode/link-count identity, and holds `flock(LOCK_SH)` while selecting and validating the
current head and immutable chain. Group membership grants neither source-store writes nor an
exclusive publication role.

`--skip-build` is acceptable only when the current Substrate head and its immutable record plus both
installed descriptors exact-match. A world-deps early return is acceptable only when the current
Codex head, immutable record, archive/extracted hashes, and installed descriptor exact-match. Version
output is diagnostic and never a byte-authority substitute.

An `InstallerArtifactSourceRefV1` resolves only under the fixed source store selected by the entry's
role. The strict store object must match the held root descriptor and `source_store_id`; the ref's
record ID/revision/hash must equal exactly one immutable record filename and recomputed body; the
record stream/build input/predecessor chain and every entry hash/physical identity must validate.
Head lookup may select a ref only while the corresponding shared installer lock is held; after
import, resolution follows the pinned immutable ref and never substitutes the current head. Equal executable bytes under another
record/store, a later head, a copied JSON object, or a self-reported version is `WrongBinding`.

Import holds the accepted-home root lock, then obtains read-only shared locks for the Substrate and
Codex source stores in that fixed order, opens each current head by its fixed installer root,
validates the complete head/record chain, and pins the two resulting immutable
`InstallerArtifactSourceRefV1` values. It then opens the three named executables, validates
ownership/metadata/provenance/digests, retains executable descriptors, and publishes the accepted
manifest with no-replace semantics. Each accepted entry stores the exact immutable installer source
ref and an exact canonical-byte copy of that source entry's `runtime_support`; later movement of an
installer head cannot change an accepted manifest. The accepted entry hash and enclosing manifest
hash authenticate that copy. Entry order is
`[Codex0125,ManagedGateway,WorldEntryWrapper]`. The wrapper entry describes the installed
`substrate-world-entry` ELF executable and fixes component `substrate-world-entry`.
`RuntimeArtifactAuthorityRefV1` must resolve this complete accepted manifest revision and exact entry,
whose installer source ref must in turn resolve the one retained immutable source record and entry;
the source-entry, accepted-entry, descriptor-pinned projection, and descriptor recomputation must
have byte-equal `runtime_support` including execution model, relocation metadata, fixed common-file
identities, and `manifest_hash`. Equal content copied into another manifest, a current path hash, or self-reported build identity is
wrong-binding.

Bootstrap creates each missing fixed directory component with descriptor-relative `mkdirat(0700)`,
reopens and validates it, and `fsync`s its parent before continuing; an already-existing component
must pass the same physical/metadata checks. The lock is created with no-follow exclusive-create or
opened no-follow after an `EEXIST`, then revalidated before `flock`. No registry object is inspected or
published until that lock is held.

Publication uses first-writer/no-replace for `store.json`, subject bindings, records, gateway
identities, input-source objects, runtime-artifact manifests, boundary revisions, intents, launch
inputs, ACKs, handoff revisions, lease revisions, redacted diagnostics, kernel-effect intents and
resolutions, child-cgroup registrations, child-process registrations, terminal-child evidence, and
retirement.
Each immutable object is written as canonical bytes to a same-directory `0600` recognized temporary,
file-`fsync`ed, and installed with descriptor-relative `renameat2(RENAME_NOREPLACE)`. An existing final
file succeeds only for an exact byte-equal retry after strict parse/hash validation; unequal bytes are
`Conflict`.

Series, handoff, and lease heads use locked expected-bytes CAS. The writer strictly validates and
byte-compares the current head with its expected predecessor, writes and file-`fsync`s the successor
temporary, rechecks the expected final name while still holding the lock, then atomically renames the
temporary over that one head. Initial heads use `RENAME_NOREPLACE`. After any install, every changed
directory is descriptor-`fsync`ed from leaf through the registry root. The committed final and head
are reopened descriptor-relatively and must read back byte-for-byte, parse strictly, and reproduce
every hash before success, capability publication, gateway activation, or final-exec release is
observable. A record becomes active only when its separately committed head is the final successful
CAS; partial earlier objects confer no authority.

The only recognized regular-file temporary grammar is
`.e3-tmp.<lowercase-uuidv7>.<store|input|artifact-manifest|subject|record|head|gateway|boundary|intent|launch-input|ack|handoff|lease|diagnostic|kernel-effect-intent|kernel-effect-resolution|child-cgroup|child-process|terminal-child|retirement>`.
A temporary is recognized only in the directory valid for its suffix, as an owner-only `0600`
regular file with link count one. Recovery runs under the same root lock. If its final object exists
byte-equal, recovery removes the temp and `fsync`s the directory. If the final object is absent and
the temp is a complete, valid immutable object, recovery performs its pending no-replace rename. For
a head, recovery additionally requires the embedded expected predecessor to remain byte-equal before
performing the CAS. Every recovered install receives exact readback plus leaf-to-root directory
`fsync`. A final mismatch, stale predecessor, two valid contenders, unknown temp, truncated bytes,
wrong directory, or unverifiable target is `PartialPublication` or `Conflict`; it is never guessed,
merged, promoted, or deleted as harmless.

The only recognized directory temporaries are the exact native-source and `/run` realization names
defined above, in their fixed parents. Their recovery validates a complete closed entry set and
manifest before no-replace rename; the regular-file temporary grammar never authorizes a partial
native tree.

All immutable input sources, artifact manifests, projection records, subject bindings, gateway
identities, boundary revisions, intents, launch inputs, ACKs, handoff state revisions,
consumer-lease revisions, redacted diagnostics, kernel-effect intents/resolutions, child-cgroup registrations, child-process
registrations, terminal-child evidence,
and immutable accepted-home native sources
are retained indefinitely in V1. A
monotonic retirement object may retire a series only after its child is terminal, its access boundary
is verified `Revoked`, its fence cannot release, and the latest revision of every known consumer lease
is `Released`; acquiring a new lease after retirement is forbidden and release evidence remains
retained. Retirement permits best-effort deletion only of the exactly revalidated, rebuildable `/run`
native realization. It never deletes accepted-home source bytes or rewrites authority/evidence objects and makes no secure-
erasure claim. There is no age-, count-, or startup-based pruning.

`effect_intent_id` is `eki_<lowercase UUIDv7>`, `resolution_id` is
`ekr_<lowercase UUIDv7>`, `cgroup_registration_id` is `ecg_<lowercase UUIDv7>`, `registration_id` is
`ecp_<lowercase UUIDv7>`, `evidence_id` is `tce_<lowercase UUIDv7>`, and a world-service process
instance ID is `wsi_<lowercase UUIDv7>`. After bootstrap authority and transition-capability parking
are validated but before startup recovery can publish or adopt any E3 object, world-service allocates
exactly one process-instance ID from its CSPRNG. It remains constant for that process, is copied into
every child registration or recovery observation it authors, is never caller-selected or adopted
from durable state, and is never reused after restart. A recovery observation preserves the original
registration's service-instance ID and separately records the current recovery service-instance ID;
equality between the two does not substitute for PID/start/boot/cgroup validation.

Every E3 cgroup creation or nftables installation is intent-first. While holding the projection root
lock and the live E3-exclusive lease, world-service publishes, file/directory-fsyncs, and
exact-readback-resolves the immutable kernel-effect intent before its first kernel mutation. A child
cgroup component is exactly `substrate-e3-` plus the first 24 lowercase hex characters of SHA-256
over the UTF-8 cgroup-registration ID; its expected relative path is the intent's exact canonical
parent path plus that single component. The gateway boundary intent repeats the already fixed
`substrate_e3_<24-hex>` table name, `gateway_output` chain, access-boundary ID, and network-namespace
inode. A successful child-cgroup registration or `GatewayAccessBoundaryV1` must reference that exact
intent and exact-match the observed created identity/handles. No unrecorded name or kernel object can
be adopted.

Startup and request recovery, still under `Recovering`, enumerates every kernel-effect intent before
allowing prepare, V2, legacy spawn, or generic GC. An intent with a complete exact effect record
continues through that record's normal recovery. For an unresolved cgroup intent, recovery
descriptor-looks up only the fixed component beneath the recorded parent. Absence publishes
`NoEffectObserved`; presence must be the exact non-mounted cgroup directory, after which recovery
kills every discovered descendant, proves `populated 0` and twice-empty complete-tree state, removes
the directory, and publishes `RevertedAndQuiescent` with its observed identity. For an unresolved
boundary intent, recovery enters only the recorded namespace and exact table name. Absence publishes
`NoEffectObserved`; presence must match the table/chain identity, is atomically replaced with reject-
all before deletion, is re-enumerated absent, and publishes `RevertedAndQuiescent` with the observed
table handle. Resolution optionals are null for `NoEffectObserved`; exactly the role-relevant
observed field is non-null for `RevertedAndQuiescent`. Unknown objects, name collisions, multiple
resolutions, mismatched handles/identity, failed kill/revoke/delete/rescan, or an intent with neither
provable absence nor safe reversal keeps E3 and legacy child admission closed. Generic GC never
matches or repairs these objects; immutable intents/resolutions are retained indefinitely.

Before clone/fork, world-service then first-writer-publishes and exact-readback-validates one
child-cgroup registration for the already created empty cgroup and exact optional turn ID. After it receives the child PID/pidfd
but before any gateway, readiness-probe, or initial/resumed Codex child can pass its first parent
gate, it publishes the process registration containing the observed PID/start tuple, exact equal
cgroup registration/ref, boot ID, and current service-instance ID. A crash before process
registration leaves the child behind an EOF-closing gate; it can never exec, and the durable
child-cgroup registration still makes that cgroup part of complete-tree recovery. Neither
registration can be synthesized after release, rewritten for PID reuse, or substituted by a process
scan.

Terminal processes are duplicate-free and sorted by the fixed role rank `ManagedGateway = 0`,
`Codex = 1`, `ReadinessProbe = 2`, then `pid`, `pid_start_time_ticks`, and registration-ID UTF-8
bytes. Empty cgroups use the same role rank, then
`(cgroup_v2_mount_device_id, cgroup_v2_mount_inode, cgroup_directory_inode,
cgroup_relative_path UTF-8 bytes)`, then cgroup-registration-ID bytes. Every terminal cgroup entry
exact-resolves its immutable cgroup registration and repeats the same role/cgroup. The list contains
every gateway, readiness-probe, and per-turn Codex cgroup preallocated or registered by the retained
E3 runtime.

When the registering service remains the parent, `ParentWaitid` requires exactly one terminal
`waitid(P_PIDFD, ...)` result and records its raw terminal status. On restart the new service cannot
invent that parent-only status. It instead uses `RecoveryObservedTerminal`: it exact-resolves the
durable registration, boot ID, cgroup, and PID/start tuple; opens a pidfd when that exact process is
still live; exact-validates its start time again through the opened identity; kills it through the
pidfd; and waits for pidfd readability. That exact-live case has `pidfd_open_errno = null`,
`pidfd_kill_errno = null` on successful delivery or `ESRCH` only for a terminal race,
`pidfd_became_readable = true`, `waitid_errno = ECHILD`, and then requires
`proc_identity = Absent` or a provably different start time. If the descriptor opens but the PID has
already been reused, recovery closes it without signaling; the only valid shape has both errno
fields null, both pidfd-readable/waitid fields null, and `PidReused` with a different nonzero start
time. If `pidfd_open` first returns `ESRCH`, the only valid shape has `pidfd_open_errno = ESRCH` and
all kill/pidfd/waitid fields null, followed by the same absent-or-reused `/proc` proof. Any other
errno/Boolean combination, same live PID/start after the terminal poll, signaling a reused PID, or
attempt to claim an exit code is `Malformed` or
`UnsupportedSecurityPosture`. Recovery-observed termination proves absence, not how or why the old
child exited and not provider/receipt success.

World-service may publish the evidence only after every child registration has exactly one of those
terminal observations, every preallocated cgroup is included, each exact cgroup's `cgroup.events`
parses as `populated 0`, its `cgroup.procs` is byte-empty on two reads separated by a complete
cgroup-tree rescan, and no registered, unregistered, or discovered descendant remains. A child that
died before its registration was publishable contributes no fabricated process observation; its
preallocated cgroup and the complete descendant scan supply the terminal proof. The ordinary SHA-256
fields bind the exact bytes read; the evidence hash binds the complete typed record. Its ref must
resolve under the configured
accepted-home root to exactly one immutable record with equal store/series/ID/hash, whose final
projection/world/generation/cgroup identities equal the retirement and active runtime. This is E3
quiescence evidence only: it is not a D1 envelope, work receipt, retained manifest, provider-success
record, or E2 authority, and it cannot manufacture any of them.

Retirement publication holds the root lock and re-resolves the final projection, quiescence evidence,
latest boundary, and every consumer-lease head. It requires the evidence's complete cgroup set, a
unique `Revoked` boundary successor bound to the final head, zero Held leases, and no live in-process
capability. `retirement/<series-id>.json` is immutable first-writer state; an equal retry joins and an
unequal object conflicts. After it exists, subject/head lookup checks retirement before constructing
a capability, no new lease or successor record may publish, and audit-only record reads remain
available without executable/artifact descriptors or a consumer capability.

## Typed resolution, compatibility, and failure behavior

```rust
enum ConfigProjectionResolutionV1 {
    Current {
        identity: ConfigProjectionIdentityV1,
        projection_ref: ConfigProjectionRefV1,
        record: AgentConfigProjectionRecordV1,
        capability: PublishedConfigProjectionCapabilityV1,
    },
    Retired {
        final_projection_ref: ConfigProjectionRefV1,
        retirement: ConfigProjectionRetirementV1,
    },
    Missing,
    UnsupportedLegacyState { diagnostic_ref: RedactedDiagnosticRefV1 },
    UnsupportedNewerSchema { observed_schema_version: u32 },
}

enum ConfigProjectionFailureV1 {
    Malformed,
    MissingPreparation,
    ExpiredPreparation,
    CancelledPreparation,
    WrongBinding,
    HashInvalid,
    StaleRevision,
    PartialPublication,
    Conflict,
    UnsupportedConfiguration,
    UnsupportedPolicySurface,
    UnsupportedRuntimeVersion,
    UnsupportedPlatform,
    UnsupportedSecurityPosture,
    RetiredSeries,
}
```

`Missing` means the trusted registry and exact subject were safely absent. `UnsupportedLegacyState`
means a pre-E3 worker lacks canonical projection authority; E3 never imports its copied home or
reconstructs a projection. `UnsupportedNewerSchema` is returned only after safely reading a complete
object with a top-level unsigned `schema_version` greater than 1 through a bounded discriminator
scan; its remaining fields are not interpreted. A missing, negative, noninteger, duplicated, or V1
schema value proceeds only through the strict V1 decoder. Unknown fields in V1 are `Malformed`.
Missing required fields, invalid JSON/TOML/base64/UTF-8, duplicate keys, unsafe paths/modes, or unknown
enum variants are `Malformed`. Store/root/session/participant/bootstrap/backend/runtime/artifact/
world/E2/gateway/handoff mismatches are `WrongBinding`. Any failed file or domain hash is
`HashInvalid`. A requested ref behind or ahead of the exact active head is `StaleRevision`; caller-
selected older revisions are not active. Interrupted object/head state is `PartialPublication`.
Multiple writers or unequal retry material is `Conflict`. All failures occur before gateway
activation or final-exec release. A Codex/version/source identity outside the fixed 0.125 loader is
`UnsupportedRuntimeVersion`; inability to prove the required Linux descriptor, mount, cgroup, or
nftables primitive is `UnsupportedPlatform`. A false/mismatched eight-key activation predicate or
missing installed accepted-home authority is `UnsupportedConfiguration`; a nonempty V1 MCP/feature
surface is `UnsupportedPolicySurface`. Missing full-isolation Landlock, unauthenticated E2
enforcement input, nonzero child capability state, privilege-transition failure, invalid or
substituted protected-user-namespace identity/membership, unavailable required trusted tracing,
dumpable credential-process exposure, or any failed control-path probe is
`UnsupportedSecurityPosture`. None degrades to
compatibility automatically.

`MissingPreparation` means the V2 carrier has no live sealed preparation in this process even though
its durable refs may still resolve. `ExpiredPreparation` and `CancelledPreparation` mean the exact
attempt reached that terminal state. They are distinct from `Missing` projection authority and from
`StaleRevision`; all three preparation failures reject before gateway spawn and require the fresh-
auth preparation behavior above. A preparation route request whose input authority is missing still
returns the ordinary `Missing` resolution, while an unknown preparation ID on V2 is
`MissingPreparation`.

`Retired` is returned deterministically whenever a valid series retirement exists, even when its
final projection head and all retained immutable records also remain readable. It carries no
`PublishedConfigProjectionCapabilityV1`. A V2 carrier, activation, resumed turn, lease acquisition,
gateway recovery, or D1 opaque-capability request naming that series fails `RetiredSeries`; it cannot
select the retained final head as `Current`, revive the fence, or create a new revision. A malformed,
hash-invalid, wrong-bound, or partial retirement is the corresponding typed failure, never ignored in
favor of the old head.

`PublishedConfigProjectionCapabilityV1` is a sealed, non-serializable in-process capability. It holds
the trusted accepted-home/root descriptors, exact store/identity/ref, a held consumer lease, and the
validated artifact descriptors required by its consumer. Only the authority service can construct it
after complete resolution; public fields, wire deserialization, cloning into a path-only token, or
reconstruction from `ConfigProjectionRefV1` are forbidden. `MemberDispatchRequestV2` carries the
serializable ref/equality material and must re-resolve it; D1 later receives this capability opaquely
and cannot inspect or rebuild its E2 authority.

World-service retains that capability explicitly rather than reducing it to `binary_path`:

```rust
struct MemberRuntimeLaunchAdmissionV2 {
    dispatch: MemberDispatchRequestV2,
    acceptance_context: Option<WorldWorkAcceptanceContextV1>,
    policy_snapshot: PolicySnapshotV3,
    preparation: SealedE3ConfigProjectionPreparationV1,
}

// Sealed, process-local state owned by exactly one ActiveMemberRuntime.
struct ActiveE3ConfigProjectionRuntimeV1 {
    identity: ConfigProjectionIdentityV1,
    closed_ref: ConfigProjectionRefV1,
    active_ref: ConfigProjectionRefV1,
    activation_ack_ref: ManagedGatewayActivationAckRefV1,
    consumer_id: String,
    consumer_lease: HeldConfigProjectionConsumerLeaseV1,
    projection_capability: PublishedConfigProjectionCapabilityV1,
    codex_adapter: E3Codex0125LaunchAdapterV1,
    native_source: HeldDirectoryDescriptor,
    native_realization: HeldDirectoryDescriptor,
    gateway_runtime: HeldE3GatewayRuntimeV1,
    immutable_worker_cap_identity: RetainedTurnWorkerCapIdentity,
    privileged_child_exclusion_lease: HeldE3PrivilegedChildExclusionLeaseV1,
    control: Mutex<ActiveE3ProjectionControlV1>,
}

struct ActiveE3ProjectionControlV1 {
    lifecycle: ActiveE3ProjectionLifecycleV1,
    turn: E3TurnControlV1,
}

enum ActiveE3ProjectionLifecycleV1 { Active, Revoking, Revoked }

enum E3TurnControlV1 {
    Idle,
    Preparing { span_id: String, turn_id: String },
    Prepared { span_id: String, turn_id: String, child_pid: u32 },
    Released { span_id: String, turn_id: String, child_pid: u32 },
}

struct E3PrivilegedChildExclusionV1 {
    state: Mutex<E3PrivilegedChildExclusionStateV1>,
}

struct RecoveredNonE3ChildIdentityV1 {
    pid: u32,
    pid_start_time_ticks: u64,
    cgroup: CanonicalCgroupIdentityV1,
}

enum E3PrivilegedChildExclusionStateV1 {
    Recovering { recovered_non_e3: BTreeSet<RecoveredNonE3ChildIdentityV1> },
    LegacyShared { live_non_e3_children: u64 },
    E3Exclusive { world_id: String, world_generation: u64, live_e3_leases: u64 },
}

enum E3GcSweepAdmissionV1 {
    StartupRecovery,
    NonE3Lease(HeldNonE3PrivilegedChildLeaseV1),
}
```

`RecoveredNonE3ChildIdentityV1` ordering is exactly `(pid, pid_start_time_ticks,
cgroup_v2_mount_device_id, cgroup_v2_mount_inode, cgroup_directory_inode,
cgroup_relative_path UTF-8 bytes)`; no implicit enum discriminant, display string, path hash, or
filesystem enumeration order participates.

The primary `run_world_service` constructs one process-wide
`Arc<E3PrivilegedChildExclusionV1>` in `Recovering` before its initial GC sweep, passes a clone into
`WorldService::new_with_privileged_child_exclusion`, completes descendant/cgroup recovery, and calls
`finish_recovery` before registering listeners. `WorldService::new_linux` constructs one
`Arc<AgentConfigProjectionServiceV1>` from the installed accepted-home authority and injects it plus
that exact exclusion instance
into `MemberRuntimeManager`, the ordinary execution/replay paths, PTY handling, and gateway runtime;
no default/path-only manager can accept V2. Every non-E3 spawn obtains a counted RAII shared lease
before its first child side effect and holds it until the exact command/member/PTY/gateway cgroup is
proven empty, including descendants; reaping only the direct child is insufficient. A missing cgroup,
lost process identity, lease leak, or failed emptiness proof keeps the non-E3 count nonzero and makes
E3 unavailable rather than guessing that the child exited.

The helper-spawning observation routes are inside that same rule. `doctor_world` acquires one
non-E3 lease before `select_strategy` or `run_enumeration_probe` and retains it through every kernel/
FUSE probe, `ls`, cleanup, kill, and wait. `pending_diff`, `pending_diff_clear`,
`pending_diff_reconcile`, and `world_fs_read` acquire one before `ensure_session` or any overlay
operation and hold it through the response-producing operation. Failure to acquire returns typed
`UnsupportedSecurityPosture` before world lookup, mount, probe-file creation, command spawn, or
filesystem mutation; a request is not queued for the end of the E3 epoch. Ordinary execute/stream
and PTY paths use the same pre-`ensure_session` rule already specified above.

The route lease cannot be dropped while a helper spawned by that request remains live. A transient
kernel/FUSE probe or `ls` is reaped before release. A `fuse-overlayfs` child or mount that persists
beyond the request is live non-E3 or unclassified state for the already mandatory descendant,
cgroup, and mount recovery scan; E3 acquisition rejects until both the helper process and its mount
are proved absent. A route that cannot prove its helper reaped and mount absent before dropping the
lease leaves or returns the exclusion to `Recovering`; it does not invent a persistent-helper
registry, adopt the helper, or guess from a missing PID. Startup recovery kills and unmounts such
state and proves both absent before `finish_recovery`, or remains `Recovering`. While E3 is exclusive
all five named observation routes are rejected at their first admission check, so neither transient
`ls` nor persistent `fuse-overlayfs` can appear.

The same injected instance is a required constructor argument of the production
`GatewayRuntimeManager`; it is never reconstructed per request. Compatibility gateway discovery and
spawn are linearized under the existing per-world lifecycle lock. An already registered
`ManagedGatewayRuntime` owns one `HeldNonE3PrivilegedChildLeaseV1` for its entire cgroup lifetime.
When no runtime is registered, `sync`, `sync_with_timeout`, `sync_with_timeout_locked`, and `restart`
must obtain a new shared lease before `recover_runtime`, manifest adoption, `start_runtime`, or any
other gateway child side effect, and move that lease into the resulting runtime. A rejected lease
returns `UnsupportedSecurityPosture` without opening a handoff, adopting a process, or spawning.
Stopping/restarting removes the old runtime only after its cgroup is proven empty and only then drops
its lease; a restart obtains a distinct lease for its replacement. `status` may observe a registered
runtime without another lease, but any manifest recovery it triggers follows the same pre-adoption
rule. `runtime_for_world_or_manifest` cannot return a recovered runtime that lacks the lease.

Startup recovery is explicit rather than a constructor loophole. While the process gate is
`Recovering`, `GatewayRuntimeManager::recover_runtime` may classify a manifest-pinned live
compatibility gateway only by exact pidfd/start-time/cgroup/executable/manifest validation and call
`register_recovered_non_e3_child` before storing it. That call creates the pending non-cloneable
lease; `finish_recovery` atomically converts all such registered pending leases into the initial
`LegacyShared { live_non_e3_children }` count. An invalid, unregistered, duplicate, or unclassified
gateway descendant is killed and its cgroup proven empty or recovery remains closed. Thus neither a
service restart nor the baseline manifest-adoption path can place a compatibility gateway outside
the exclusion count.

If the service cannot enumerate a complete cgroup forest for every earlier untrusted launch, the
process-wide exclusion remains `Recovering` and E3 is `UnsupportedSecurityPosture`.
The one startup GC sweep runs only with `StartupRecovery` while the exclusion state is still
`Recovering`, finishes and reaps every `ip`/`timeout` helper before `finish_recovery`, and cannot
overlap any E3 authority. Each later periodic or manual sweep must first acquire
`NonE3Lease`; `gc::sweep` rejects a missing/wrong admission before its first `Command`. The single
lease covers `list_netns`, `netns_pids`, `delete_nft_table`, and `delete_netns` until all helper
processes are reaped. A periodic tick during E3-exclusive mode is skipped and logged; a manual sweep
returns typed `UnsupportedSecurityPosture`. Neither path queues a sweep to run after an E3 lease is
released.
Before listener reservation, cgroup creation, nftables mutation, kernel-effect intent publication,
Dormant publication, gateway-wrapper fork, or any other preparation/activation side effect, the
first accepted E3 preparation CASes `LegacyShared { live_non_e3_children: 0 }` to `E3Exclusive`;
later E3 preparations in the exact world/generation join it and increment a non-cloneable lease
count. The preparation retains that lease through V2 consumption and the active runtime. No request may
wait while holding a spawn-side lock: a conflicting mode fails immediately and safely.
`MemberRuntimeManager::launch_v2` consumes
`MemberRuntimeLaunchAdmissionV2`, resolves the carried Dormant head, performs the gateway activation
and ReadyClosed publication, obtains the exact Active ref,
and moves the non-cloneable capability, lease, artifact/native descriptors, adapter, and live gateway
handle plus its exclusion lease into the new `ActiveMemberRuntime.e3_projection`. Registration failure calls the single
revoke-and-release path before discarding the runtime. The V1 `launch` constructor and V1 active
state never synthesize this field; while the process-wide exclusion is idle their behavior remains
frozen, and while it is E3-exclusive they fail at the common pre-spawn security admission.

The bootstrap child uses the same `control` mutex and `Preparing` → `Prepared` → `Released`
transitions with its bootstrap run/span identity. The runtime is not registered as resumable until
that exact `Released` state and all E2 launch evidence are durable. Any registration or terminal
failure enters `Revoking` under the same mutex, so the initial final-exec gate has the same
`lifecycle == Active` requirement as every later turn.

`MemberTurnSubmitRequestV1` remains schema/byte unchanged; a resumed request acquires no E3 carrier
from its caller. Both existing submit branches first inspect the retained active runtime. When
`e3_projection` is present they call
`ActiveE3ConfigProjectionRuntimeV1::admit_and_reserve_resumed_turn`. The existing E2 durable join may
first reserve/identify its turn and elect a leader because that operation cannot launch a child; the
active-member registry lock is then released before the E3 control lock is acquired. Under the one
`ActiveE3ConfigProjectionRuntimeV1.control` mutex, admission requires lifecycle `Active` and turn
`Idle`, installs `Preparing { span_id, turn_id }`, and thereby reserves the sole E3 turn slot. That
state change is the submit-vs-terminal linearization point. The legacy
`ActiveMemberRuntime.active_turn_span_id` mutex is not consulted for E3 and can never be acquired
while holding the E3 control mutex.

Only the submit that installed `Preparing` may perform child preparation. It revalidates the held
lease, process-wide exclusive lease, current `Active/Released` head,
identity/ACK/boundary/gateway process, native source and realization, artifact descriptors,
immutable worker-cap identity, world generation, and exact request session/participant/backend
binding. The E2 branch must additionally complete its existing authenticated carrier/current-parent/
acceptance/durable-join validation and pass that exact turn snapshot into the transient enforcement
input. A non-E2 submit against an E3 runtime is `UnsupportedLegacyState`; neither submit path may
fall back to `active.binary_path`, UAA, or a new projection resolution. After descriptor, output,
wrapper, and cgroup preparation, the submit reacquires the same mutex and may change its exact
matching `Preparing` to `Prepared`; any lifecycle other than `Active` or any turn mismatch kills the
unreleased cgroup and publishes the E2 prelaunch failure or uncertainty required by the already-
reached join state.

Immediately before the one final-exec byte, `release_resumed_turn_final_exec` holds that same mutex
and requires lifecycle still `Active`, the exact matching `Prepared` PID/turn/span, all final E3/E2
revalidation, and a live process-wide exclusive lease. The gate write and transition to `Released`
occur while the mutex remains held; it is the only point at which that child can run Codex.
Completion or a proven pre-release failure changes the matching turn back to `Idle` under the same
mutex. The active projection and gateway stay unchanged across turns; every child receives a fresh
pidfd, member-cgroup placement proof, E2-authenticated enforcement input, security attestation, and
one-shot final-exec gate. The existing durable E2
`CallEntered`/`LaunchIndeterminate`/Started/Completed join is the retry authority; E3 never replays an
ambiguous final-gate write. On completion the per-turn descriptors close but the active E3 state,
consumer lease, and exclusion lease remain held for the next sequential turn.

`unregister_member`, terminal bootstrap without a resumable Codex session, world-generation loss,
cap mismatch, cancellation that terminates the retained runtime, or service recovery invokes exactly
`ActiveE3ConfigProjectionRuntimeV1::revoke_and_release`: under the same control mutex change
`Active` to `Revoking` before any teardown and capture the exact `Preparing`, `Prepared`, or
`Released` turn identity. If terminal revocation wins that lock before submit, no later child
preparation is permitted; if submit already installed `Preparing`, revocation cancels that owned
preparation and no final release can pass the required `Active` check. It then kills any live or
preparing turn cgroup,
CAS the gateway boundary to `Revoked`, prove denial, stop the gateway, close all held descriptors,
publish the lease's sole `Released` successor, and only then permit `/run` realization cleanup.
It changes `Revoking` to `Revoked` and drops the process-wide E3 lease only after both member and
gateway cgroups are empty; the exclusion returns to legacy-shared mode only when the last same-world
E3 lease is gone. `unregister_turn` never drops the retained capability. Restart cannot reconstruct active process
handles from a path or ref; it first revokes/kills any exact recorded remnants, then a fresh dispatch
must create a new fence/gateway/lease. A poisoned lock or partial cleanup fails closed and retains
authority evidence.

## `MemberDispatchRequestV2` carrier and D1's later V3

```rust
// Lower-leaf, nonsecret pointer to inputs already published in accepted-home authority.
struct ConfigProjectionAuthoringInputRefV1 {
    authority_store_id: String,
    effective_config_source_hash: String,
    agent_inventory_source_hash: String,
    runtime_artifact_manifest_id: String,
    runtime_artifact_manifest_revision: u64,
    runtime_artifact_manifest_hash: String,
    input_ref_hash: String,
}

// Strict request for POST /v1/e3/config-projection/prepare.
struct E3ConfigProjectionPrepareRequestV1 {
    schema_version: u32, // exactly 1
    preparation_id: String,
    preparation_idempotency_key: String,
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    e2_launch_activation: E2MemberLaunchActivationCarrierV1,
    authoring_input_ref: ConfigProjectionAuthoringInputRefV1,
    integrated_auth: GatewayIntegratedAuthPayloadV1, // transient secret-bearing field
}

struct E3ConfigProjectionPrepareResponseV1 {
    schema_version: u32, // exactly 1
    preparation_id: String,
    preparation_idempotency_key: String,
    config_projection: ConfigProjectionActivationCarrierV1,
    prepared_at: String, // canonical RFC 3339 UTC timestamp
    expires_at: String, // canonical RFC 3339 UTC timestamp
    response_hash: String,
}

struct E3ConfigProjectionCancelRequestV1 {
    schema_version: u32, // exactly 1
    preparation_id: String,
    preparation_idempotency_key: String,
    config_projection: ConfigProjectionActivationCarrierV1,
}

struct E3ConfigProjectionCancelResponseV1 {
    schema_version: u32, // exactly 1
    preparation_id: String,
    disposition: E3ConfigProjectionCancelDispositionV1,
    cancelled_dormant_projection_ref: ConfigProjectionRefV1,
    response_hash: String,
}

enum E3ConfigProjectionCancelDispositionV1 { Cancelled, AlreadyTerminal }

// Private, non-serializable, non-cloneable world-service state.
struct SealedE3ConfigProjectionPreparationV1 {
    preparation_id: String,
    preparation_idempotency_key: String,
    response: E3ConfigProjectionPrepareResponseV1,
    projection: PublishedConfigProjectionCapabilityV1,
    credential_source: SealedCredentialSourceCapabilityV1,
    gateway_authority: E3GatewayRuntimeAuthorityV1,
    expires_deadline: ClockBoottimeDeadline,
}

struct ConfigProjectionActivationCarrierV1 {
    authority_store_id: String,
    series_id: String,
    dormant_projection_ref: ConfigProjectionRefV1,
    activation_intent_ref: ManagedGatewayActivationIntentRefV1,
    expected_gateway_ref: InWorldGatewayRefV1,
    fence_id: String,
    consumer_id: String,
    consumer_lease_revision: u64, // exactly 1 / Held
    consumer_lease_hash: String,
}

struct MemberDispatchRequestV2 {
    schema_version: u32, // exactly 2
    // Every V1 field, unchanged in name, type, and meaning:
    orchestration_session_id: String,
    participant_id: String,
    orchestrator_participant_id: String,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: String,
    run_id: String,
    world_id: String,
    world_generation: u64,
    initial_prompt: Option<String>,
    resolved_runtime: ResolvedMemberRuntimeDescriptorV1,
    retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    e2_launch_activation: Option<E2MemberLaunchActivationCarrierV1>,
    config_projection: ConfigProjectionActivationCarrierV1,
}

// Wire representation remains the untagged member_dispatch object.
enum MemberDispatchRequest {
    V1(MemberDispatchRequestV1),
    V2(MemberDispatchRequestV2),
    // D1 later adds V3(MemberDispatchRequestV3).
}
```

`SealedE3ConfigProjectionPreparationV1`, `SealedCredentialSourceCapabilityV1`, and
`ClockBoottimeDeadline` are private world-service implementation types, not wire or
`config-projection` exports. The world-service preparation manager owns the sealed map and composes
the downward `AgentConfigProjectionServiceV1` persistence API with the sibling gateway-runtime and
child-exclusion APIs. The shared crate neither imports world-service nor owns listener/process/secret
capabilities, preserving the stated acyclic Cargo graph.

The E3 local route additionally uses these private, non-serializable world-service observations:

```rust
struct E3AuthenticatedLinuxUdsListenerV1 {
    source: String, // exactly "systemd-socket-activation"
    path: String, // exactly "/run/substrate.sock"
    descriptor_device_id: u64,
    descriptor_inode: u64,
    filesystem_device_id: u64,
    filesystem_inode: u64,
    filesystem_owner_uid: u64, // exactly 0
    filesystem_owner_gid: u64, // exactly the service effective GID
    filesystem_mode: u32, // exactly 0o660
    filesystem_link_count: u64, // exactly 1
    listen_accepting: bool, // exactly true
    expected_peer_uid: u64,
    expected_peer_account: String,
    kernel_boot_id: String,
}

struct E3AuthenticatedLinuxUdsPeerV1 {
    peer_pid: u32,
    peer_pid_start_time_ticks: u64,
    peer_uid: u64,
    peer_gid: u64,
    kernel_boot_id: String,
    listener_descriptor_device_id: u64,
    listener_descriptor_inode: u64,
}
```

These preparation/cancellation types and their strict private decode forms are owned by
`transport-api-types`; they contain only lower-leaf transport types, strings, and integers. Every
struct denies unknown fields and validates its exact schema version before use.
`preparation_id` is `e3p_<lowercase UUIDv7>` and is unique per attempt.
`preparation_idempotency_key` is `e3pik_<lowercase SHA-256>` over the canonical JSON preimage
`{"credential_shape":{"backend_id":<integrated_auth.backend_id>,"ordered_field_names":<the exact
credential-source ordered_field_names>},"domain":"substrate.e3.config-projection-prepare-idempotency.v1",
"request":<prepare request with preparation_idempotency_key and integrated_auth omitted>}`. It
contains the request's exact nullable `retained_worker_launch_authority` value and therefore binds
the launch kind without containing any secret value, secret-derived digest, account value, or
reusable verifier.
`input_ref_hash` is the canonical JSON hash of
`{"domain":"substrate.e3.config-projection-authoring-input-ref.v1","input_ref":<ref with input_ref_hash omitted>}`.
It resolves only beneath the configured accepted-home registry: store ID must equal `store.json`,
both source hashes must name strict readback-valid input objects, and the artifact manifest
ID/revision/hash must name one exact accepted manifest. Equal input bytes in another store, under
another source hash, or in another manifest revision do not substitute.
The prepare response hash is the canonical JSON hash of
`{"domain":"substrate.e3.config-projection-prepare-response.v1","response":<response with response_hash omitted>}`;
the cancel response uses the parallel domain
`substrate.e3.config-projection-cancel-response.v1`. These response hashes are exact transport
integrity checks, not persistent authority or substitutes for resolving their refs.

The shell first publishes and exact-readback-validates only the immutable effective-config and
inventory source objects named by `authoring_input_ref`, imports the trusted artifact manifest, and
resolves one fresh `GatewayIntegratedAuthPayloadV1` by a bounded extraction of the baseline
`world_gateway::resolve_integrated_auth_payload`. It then calls exactly
`POST /v1/e3/config-projection/prepare`; it does not call the compatibility gateway sync/restart
routes, construct a gateway identity, publish a projection record, or put the secret in
`ExecuteRequest`. The request's common launch fields and E2 values must pass the same strict checks
as V2. World-service re-resolves every accepted-home input ref and the E2 authority and requires
field-for-field equality before accepting the integrated auth. In exact agreement with the landed E2
validator, a `FreshSpawn` activation requires `retained_worker_launch_authority = Some(<the exact
validated proof>)`, `parent_participant_id = null`, and `resumed_from_participant_id = null`; a
`Fork` activation requires `retained_worker_launch_authority = null`,
`parent_participant_id = e2_launch_activation.source_participant_id`, and
`resumed_from_participant_id = null`. Any other nullability or lineage is wrong-binding before the
secret-bearing field is accepted.

The prepare and cancel routes are registered only on one independently built E3 Linux UDS router,
never on the baseline router cloned to TCP or on a direct-bind UDS. E3 route registration first
requires exactly one `InheritedUnixListener` accepted by the landed socket-activation collector for
the current `LISTEN_PID`; zero or multiple inherited UDS descriptors disables both routes. On that
descriptor, `AF_UNIX`, `SOCK_STREAM`, `SO_ACCEPTCONN=1`, and `getsockname` pathname exactly
`/run/substrate.sock` are mandatory. Descriptor `fstat` and no-follow filesystem `lstat` must be the
respective nonzero descriptor-socket and pathname-entry identities and both must be socket types;
their distinct kernel-socket and filesystem inode domains are never compared as equal. The
filesystem entry must have link count one, owner UID 0, owner GID equal to the already running
service's effective GID, and mode exactly `0660`. The
service does no group-name or caller-selected path lookup. The expected peer UID/account come only
from the already validated `ConfiguredAcceptedHomeAuthorityV1` installed record and its current-
principal binding. Any socket-activation, descriptor/path, owner/mode, accepted-home, or boot-ID
mismatch leaves the ordinary V1 router available but does not register E3 routes and is typed
`UnsupportedConfiguration` to the E3 client.

Before registering the E3 router or allowing its inherited listener to accept a byte,
`run_world_service` calls `lock_e3_service_secret_memory_v1`: set soft and hard `RLIMIT_CORE` to
zero, set `PR_SET_DUMPABLE=0`, and require exact `getrlimit` plus `PR_GET_DUMPABLE` readback. The
process never restores either value. Failure leaves E3 routes unregistered, so a secret-bearing
prepare body cannot enter a dumpable primary service. This E3 service-process hardening neither
changes an E2 object nor supplies child confinement.

Before Hyper/Axum receives a byte from an accepted E3 connection, the dedicated acceptor obtains
Linux `SO_PEERCRED`, requires its UID to equal the installed expected peer UID, captures PID/GID,
opens and validates `/proc/<pid>/stat` start time under the current boot ID, and binds the resulting
`E3AuthenticatedLinuxUdsPeerV1` to that connection. Failure closes the stream without parsing an
HTTP request. Each prepare/cancel handler requires that exact connection extension and repeats its
UID, boot, listener device/inode, and installed accepted-home account equality before reading the
body. Peer GID is evidence, not a group-membership substitute. TCP, inherited TCP, direct-bind UDS,
VSock, Lima host forwarding, WSL, and non-Linux dispatch use routers with no E3 route; their 404 is
mapped by the E3 client to `UnsupportedConfiguration` without retrying the secret body on another
transport. Prepare has an exact 64-KiB body limit, never records the body or serde/debug
representation, and maps parse failure to a fixed redacted diagnostic. After strict decoding,
secret strings move immediately into non-cloneable zeroizing buffers; every reject, retry, cancel,
expiry, successful move to the one-time pipe, and unwind closes descriptors and zeroizes those
buffers. Traces contain only preparation ID, nonsecret idempotency key, backend binding, typed
disposition, and redacted diagnostic ref.

For one accepted preparation, world-service acquires or joins the process-wide E3-exclusive lease,
moves the integrated-auth payload into a sealed process-local credential-source capability, binds and
retains the sole listener without listening, installs/readback-validates the initial deny-all
boundary, calls `listen(16)`, and proves the accepting queue empty before it publishes in dependency
order the nonsecret credential-source ref, `Prepared` handoff, gateway identity, activation intent,
native source, `Dormant/ZeroLiveClosed` projection, and revision-1 Held consumer lease. The Dormant
head and lease are committed before the response is returned. No gateway, readiness probe, Codex,
UAA, provider, or other child is spawned by either preparation route. The sealed preparation retains
the credential capability, listener descriptor, artifact descriptors, cgroup/boundary handles,
consumer lease, and E3-exclusive admission, keyed by preparation ID and exact carrier, until one
matching V2 launch consumes it or cancellation/expiry/recovery terminates it. Thus the returned
projection is independently valid before D1 while remaining launch-inhibiting.

An exact retry with the same preparation ID, idempotency key, and byte-equal nonsecret request joins
the live sealed attempt and returns the identical response. Because reusable secret equality is
forbidden, world-service does not compare the retried secret values: it immediately closes and
scrubs that duplicate payload without changing the original capability. A reused preparation ID or
key with any nonsecret difference is `Conflict`. A preparation expires after exactly 120 seconds by
the service's `CLOCK_BOOTTIME` deadline; `expires_at` is the corresponding informational RFC 3339
UTC projection and is not the timer authority. Cancel and expiry kill any unreleased child, revoke
and verify the boundary, terminate the handoff as `Failed` or `Expired`, make the old carrier
terminal/stale, release the consumer and exclusion leases, and retain every authority object. The
next accepted fresh-auth preparation may publish a legal new `Dormant/ZeroLiveClosed` successor
fence when the series remains eligible. Cancel is equality/idempotency guarded and can never cancel
a different or already Active attempt.

After a service restart no process-local credential capability or held listener/preparation
capability is reconstructible from durable refs. Recovery therefore revokes the recorded boundary,
kills and proves quiescent any bound remnant, terminates the old handoff, and makes the old carrier
stale. A caller must submit fresh auth under a fresh preparation ID; if the immutable subject is
unchanged, the service publishes a new fence/root/gateway/handoff/lease revision in the same series.
A fresh series is permitted only when an immutable subject field changed. No recovery replays a
secret, listener, launch input, barrier, or ACK.

Once the shell exact-validates the response and resolves its Dormant record, it copies the returned
carrier unchanged into `MemberDispatchRequestV2` and sends `/v1/execute` or
`/v1/execute/stream`. World-service accepts that V2 only when its carrier joins the still-live sealed
preparation with equal preparation/lease/intent/gateway/fence bindings and has not expired or been
cancelled. The V2 common launch fields, `retained_worker_launch_authority` including exact nullability,
and `e2_launch_activation` must be field-for-field equal to the canonical nonsecret prepare request
retained in that sealed preparation. The execute path consumes the sealed preparation exactly once and performs activation;
it never obtains integrated auth from the V2 body. If the shell cannot submit V2 it calls exactly
`POST /v1/e3/config-projection/cancel`; loss of that cancellation still converges through the fixed
expiry and recovery rules.

`ExecuteRequest.member_dispatch` adopts the version wrapper without changing its wire key. A bounded
top-level `schema_version` discriminator routes to exactly one strict decoder before fields are
interpreted: absent or 1 selects V1, 2 selects V2, and any other unsigned value is unsupported. The V1
type, strict `deny_unknown_fields`, default schema 1, validation, and wire bytes remain unchanged. V2
has its own strict definition, requires schema 2 and the non-optional projection carrier, and repeats
every V1 validation. The wrapper does not probe permissive decoders, ignore unknown fields, or coerce
schema versions; a duplicate, negative, noninteger, or otherwise malformed discriminator is rejected
as malformed before either decoder. An old server rejects V2 before launch; a new server may accept
V1 only on the existing named compatibility path and cannot claim E3. Missing E3 material never falls
back from V2 to V1. `MemberDispatchRequest::as_v1` and `as_v1_mut` return the corresponding borrowed
V1 value only for the V1 variant and `None` for V2; they never down-convert, synthesize, or strip E3
fields. Existing V1-only fixtures must match or call one of those accessors before their unchanged
field assertions or mutations.

V2 is valid only for an E2-authorized retained-worker launch or retained-worker fork. Its
`retained_worker_launch_authority`/`e2_launch_activation` combination must be the exact combination
permitted by the E2 launch kind. Resolution of `config_projection.dormant_projection_ref` through
the configured accepted-home authority must yield the exact Dormant record, and that record's
`identity.immutable_launch_cap` must be the field-for-field normalized join to the same activation;
the carrier itself never embeds or substitutes an identity. Ephemeral task dispatch, legacy fork,
ordinary non-E2 member startup, and every compatibility route continue to emit V1 and cannot claim
E3. This bounded rule is why the later implementation fence names only the exact retained Spawn/Fork
constructors; it does not silently upgrade every `MemberDispatchRequestV1` producer.

`ExecuteRequest.member_dispatch` is `Option<MemberDispatchRequest>`. The wrapper provides a private
borrowed common-fields view for the fields literally shared by V1/V2 and version-specific accessors
for E2 and E3 carriers; it does not deserialize through V1 or allocate a lossy common wire object.
`execute_stream`, `requested_shared_world_owner_spec`,
`resolve_authoritative_member_placement_context`, `exact_bound_world_ownership_adoption`, and
`validate_member_dispatch_binding` consume that borrowed view for their existing equality checks.
The member-runtime admission remains versioned through launch: V1 selects the exact legacy
`PromptFulfillmentBridge` branch, while V2 must retain and resolve its projection carrier before any
gateway or child side effect. `convert_member_dispatch_request` likewise returns a strict internal
V1/V2 enum and may not erase V2's carrier. No V2 value may be down-converted to V1 to reach launch.
Fixtures must demonstrate that serialized V1 requests, V1 error classification, shared-world
selection, E2 validation, and legacy UAA invocation are byte/behavior-identical to the baseline.

The baseline Lima bridge accepts only `world_api::MemberDispatchRequestV1`, while the transport-side
field now names the wrapper. Its bounded adapter therefore constructs the same transport V1 value
field-for-field and returns `MemberDispatchRequest::V1(value)`. Because the wrapper is untagged, the
JSON member-dispatch object is byte-identical to the baseline. Lima has no V2 input type or projection
service in E3, must never emit V2, and rejects any future attempt to route E3 through this adapter;
this compatibility conversion is not non-Linux E3 support.

The carrier is deliberately pre-activation. Its `dormant_projection_ref` names the exact
`Dormant/ZeroLiveClosed` head whose ACK field is `null`; the carrier's store/series, intent, gateway,
and fence fields must equal that record. The preparation route acquires a durable
`MemberDispatchV2` consumer lease with the carried consumer/lease IDs before the carrier becomes
observable; neither route activates the gateway or fabricates an ACK. This bounded route is the only
way the first valid V2 request can receive both durable Dormant authority and a live, nonpersistent
credential capability.
The carrier's top-level store/series must equal its dormant ref and resolved identity; its consumer
ID, lease revision, and lease hash must resolve the unique revision-1 `Held` `MemberDispatchV2`
lease for that same acquired ref/fence; and all
request session/participant/bootstrap/backend/world/E2 values must equal the dormant record. Equal
bytes under another record, lease, intent, gateway, or series never substitute.

`WorldService::execute_stream` re-resolves that dormant head and lease from configured accepted-home
authority and joins the sealed preparation, then `MemberRuntimeManager::launch_v2` owns the single
two-phase advance inside the same request: consume the already-held process-wide E3 exclusion;
activate the managed gateway; publish the ACK and
unique `ReadyClosed` successor; prepare the initial Codex child; publish exactly one
`Active/Released` successor whose `closed_record_ref` is that service-created ReadyClosed ref; and
release the child only after final validation. The shell's carrier remains bound to the dormant
predecessor and is never rewritten to point at ACK, ReadyClosed, or Active. Every successor exact-
validates its unique predecessor plus the original carrier fields. Failure releases the consumer
lease only after killing every unreleased child, revoking the boundary, and recording the legal
handoff terminal.

On first admission the carried dormant ref must be the current head. After world-service wins the
first successor CAS, only its retained non-serializable capability may use that immutable dormant ref
as a lineage anchor while resolving the unique current ReadyClosed/Active successor. A new caller
cannot use the now-historical ref as active authority. An exact transport retry with the same request,
idempotency key, consumer, lease hash, and dormant ref may subscribe to the already durable launch
outcome under the existing E2 replay/join rules; it never reruns gateway activation or writes another
gate. Any other historical carrier is `StaleRevision` or `Conflict`.

Immediately before gateway activation, the E3 authority service resolves the dormant active head
from configured accepted-home authority—not a request path—and exact-validates the carrier, record,
E2 link, activation intent, zero-live fence, descriptor-pinned artifacts, prepared handoff, and
dormant gateway identity, including that no ACK exists. Immediately before final-exec release it
resolves the service-created Active head and additionally exact-validates ReadyClosed, ACK, consumed
handoff, allowed boundary, and live gateway process identity. WorldService separately
revalidates the local open descriptors, exact gateway process/listener/access-boundary identity, and
carrier equality before release. Any change between checks aborts and leaves the child unlaunched.

D1 later owns an additive strict `MemberDispatchRequestV3` containing the exact V2 fields plus an
opaque `WorldRuntimeAdapterExecutionEnvelopeV1` carrier. D1 does not amend V2 and E3 does not prebuild,
partially populate, or claim the D1 envelope. Thus E3-before-D1 ordering is real: E3's published
capability validates and can be adopted independently; D1 later consumes it opaquely.

## Compatibility and nonownership wall

The current copied host auth/config path may remain only as
`CompatibilityCopyBridgeV1 { compatibility_mode_id }`: separately policy-granted per launch,
prominently logged without secret paths/values, absent from projection authority, excluded from all
E3 proof, and non-promotable. It cannot create a record/ref/ACK, satisfy a missing or invalid V2
carrier, or be upgraded by observing successful provider traffic. Retirement requires contract-
correct managed-gateway proof for every credential-requiring Codex path and a later explicitly
authorized removal; E3 does not silently delete it.

E3 does not own or modify:

- `WorldRuntimeAdapterExecutionEnvelopeV1` or D1 `MemberDispatchRequestV3`;
- B1/B2.1 acceptance, observation, journal, replay, or terminal truth;
- B2.2/B3.2 receipts or `RetainedWorkerManifestV1` construction;
- E1/E2 policy composition, commitment, cap, persistence, or `E2-RM` history lookup;
- broker policy or per-operation D2 enforcement;
- gateway provider policy or upstream credential authority;
- E4 host-visible workspace projection, sync, import, export, or reconciliation; or
- migration/backfill of legacy workers or projection state.

## Serial E3 decomposition

The controlling slice's E3-A through E3-F labels are internal work-packet headings, not new stable
IDs or independent authority documents. Their binding order and owners are:

| Packet | Bounded owner | Direct predecessor |
|---|---|---|
| `E3-A — strict V2 wire carrier and V1 compatibility` | strict transport/version-wrapper reachability and byte-identical V1 compatibility | pushed and live-verified E3-AC1 correction |
| `E3-B — projection codec, registry, CAS, recovery, and retirement` | config-projection codec/registry plus the one literal HSA closed-layout addition | proof-clean landed E3-A |
| `E3-C — authenticated authoring, V3 inventory, artifacts, and Codex rendering` | descriptor-pinned source-store schemas, validation/import, publication/recovery helpers, selected-only V3 provenance, native source, and deterministic Codex rendering; no E3-D artifact production | proof-clean landed E3-B |
| `E3-D — Linux child security, capability parking, and process-wide exclusion` | Linux privilege descent, enforcement, actual wrapper implementation, static Substrate build/install and publication integration, and universal child/helper exclusion | proof-clean landed E3-C |
| `E3-E — dormant managed-gateway preparation and adoption` | authenticated preparation, Dormant/no-ACK publication, listener/boundary/readiness, one-time secret delivery, adoption, and revocation | proof-clean landed E3-D |
| `E3-F — retained V2 Codex launch/resume adoption and integrated proof` | retained local-adapter/gateway capability across initial and resumed V2 turns and integrated proof | proof-clean landed E3-E |

The slice specifies each packet's consumed inputs, explicit nonownership, and minimum successor proof.
Every packet requires a later fresh admission and explicit dispatch. Passing one packet only makes
its direct successor eligible to seek admission; it never dispatches that successor. One combined
E3-A-through-E3-F implementation candidate is forbidden.

## Packet-proportional executable Linux readiness

No statement in this contract means that the current host automatically satisfies an E3 packet's
acceptance wall. Each later fresh admission must bind that packet's exact outcome and nonownership,
source revision and toolchain identities, actual proof commands, required host facilities, and
realistic resource needs. Its retained receipt must make those commands and inputs reproducible and
must establish the environment needed by the bounded packet; it does not certify unfinished behavior
owned by a successor.

E3-A requires Linux `x86_64`, the repository-required Rust toolchain (at this baseline, channel
`1.89.0` and MSRV `1.89`), and the other environment needed for its strict wire-transport and
V1-compatibility tests. A musl target or linker, privileged namespace/cgroup/nftables facilities,
gateway or Codex execution, and full E3 security attestation are not E3-A prerequisites merely
because a later packet needs them. If an actual retained E3-A invocation requires any such facility,
however, that invocation must satisfy its real prerequisite before the dependent operation; a false
green, silent skip, or otherwise invalid invocation is not product proof. The same rule applies
packet by packet: missing or drifting required facilities are environmental stops for the work that
depends on them, while security behavior implemented by a candidate is established by the owning
packet's proof rather than presumed before that code exists.

E3-C's bounded proof uses valid test inputs to exercise its source-store schemas, strict validators,
import and publication/recovery helpers, and native-source/rendering behavior. It neither executes nor
installs the E3-D-owned `substrate-world-entry` implementation, and its inputs cannot be promoted to
production artifact authority or installed-runtime readiness evidence. E3-D must consume the landed
E3-C machinery, implement and statically build/install the real wrapper and gateway, readback-validate
their descriptors and exact build provenance, and establish the valid Substrate source record/head
before any dependent E3 runtime operation. Until then the E3 artifact capability is unavailable and
an invocation requiring its publication fails closed; unrelated non-E3 provisioning may continue
without claiming E3 publication or readiness.

All product-security predicates in this contract remain unchanged and fail closed. The owning packet
must establish the required existing `CAP_SYS_ADMIN`, `CAP_NET_ADMIN`, `CAP_DAC_OVERRIDE`, and
`CAP_SYS_PTRACE` service-authority posture, capability parking and readback, privilege descent,
trusted-service-owned per-child user-namespace identity/membership, required tracing,
namespace/cgroup/nftables/Landlock/seccomp confinement, credential
handling, gateway activation and revocation, and control-path denial before the operations they
protect and before claiming that packet complete. In particular, adding `CAP_SETUID`, `CAP_SETGID`,
or `CAP_SETPCAP` to an otherwise unmodified service is not a substitute for E3-D implementing and
proving the specified synchronous transition-capability parking and bounded parent map-installation
behavior; no fallback or substitute privilege model is authorized. Host-wide Yama policy remains
untouched and is not an E3 authority input.

Admissions require reproducible commands and sufficient retained evidence, not a bespoke proof
wrapper or six newly created roots for every packet. Existing valid build targets, roots, and evidence
may be reused when their causal inputs are unchanged. Filesystem use must not collide with repository
or accepted authority state; must use short paths when Unix sockets are involved; must preserve every
proof-required device relationship; and must keep evidence persistent while placing neither heavy
builds nor durable state on tmpfs. Separate roots or other isolation are required where the proof
needs them, including between baseline and candidate when shared state could affect the result, but
incidental harness topology is not itself a universal admission predicate.

Applicable differential proof begins with the minimum valid matched baseline-and-candidate commands
that can reach the changed behavior and compares canonical semantic outcomes rather than scheduling
or emission order. Unchanged commands, candidate bytes, causal inputs, and retained invocation
evidence may be reused; candidate-only semantic failures still reject the candidate, and an invalid,
resource-killed, incomplete, or provenance-ambiguous invocation never counts as product proof. The
mandated E3-A comparison baseline remains
`2b2fc6c50b40046dbeaeb5b316562fbd96480a2a`; the pushed E3-AC1 documentation landing is authority,
not a substitute comparison commit.
Documentation validation in this correction is not E3 implementation evidence, and this correction
grants no packet admission, dispatch, implementation, green E3 gate, or security exception.

### E3-A Linux-first proof-recovery authority

The controlling platform schedule is Linux first, completely. E3-A is a Linux implementation and
landing packet. Native macOS testing, implementation, provisioning, service work, Lima work, and the
previously separate macOS parity lane are deferred until the Linux runtime-refactor program is
complete. `E3A-P2-003` is therefore deferred and nonblocking for E3-A's Linux completion: the
historical baseline and candidate Lima logs remain unavailable/zero-test evidence and must remain
recorded honestly, but the macOS-only exact Lima test is not a Linux proof gate. A macOS-gated
crate's empty or zero-test Linux execution establishes no macOS compatibility, correctness, or
completion claim. Required Linux checks must exercise their intended Linux code or tests nonzero.
No rerun is required solely to remove a non-discriminating macOS package from an already-retained
invocation. The existing byte-preserving Lima V1-wrapper conversion in the preserved candidate is
frozen mechanical compatibility only; it authorizes no macOS runner, VM, installer, service,
implementation, parity work, or proof claim.

The preserved recovery subject is product base
`5e972add652224b97323f6abd496e94440d72242`, tree
`ddbceef5f723a190cea70c310b462b3276fee82d`, with product fingerprint
`4c4e1b4ef940621553e3a9d6472d69748117857080849cf4e246aca69d64f81a` and full-index binary
product-delta SHA-256 `139b28a1eeb8d5e498b20723d17a37ad170be26e01585d0e6136c7a33aefbe94` against exact
comparison baseline `2b2fc6c50b40046dbeaeb5b316562fbd96480a2a`. The semantic differential remains governed by
retained authority commit `5b703b26431f9dee5e9e0686e280536a6fb030c8`. Existing valid E3-A
proof is reusable only while its candidate bytes, commands, and causal inputs remain unchanged; this
recovery does not restart admission or the full proof wall.

`E3A-P2-004` retains its original failed parity fixture and result. Recovery may add only a direct
colocated prepared-runtime builder fixture under the existing test authority, proving that
`backend_id`, `protocol`, `binary_path`, and `backend_kind` remain frozen after descriptor drift and
that `config_projection == None`. It must not relabel the old fixture as passing, change unrelated
HSA semantics, or edit production behavior solely to manufacture the witness.

For `E3A-P2-005`, a later recovery dispatch has exactly this harness-only ownership:

1. In `crates/shell/tests/repl_world_first_routing_v1.rs`, change only the `PtyRepl::spawn`
   argument lists inside
   `c3_first_targeted_world_turn_uses_initial_prompt_in_member_dispatch` and
   `c3_internal_toolbox_fork_command_reuses_retained_fork_bootstrap_with_explicit_lineage`, solely
   to pass the global `--install-prefix PATH` selector followed by that test's existing UTF-8
   `substrate_home`. The selector is the supported global CLI input: its Unix path is normalized and
   selected as the invocation's install-bootstrap prefix. No other call site, helper, fixture,
   assertion, timeout, or test behavior may change.
2. In the actual helper `crates/shell/tests/common.rs::ensure_substrate_built`—not
   `tests/support/common.rs`—change only the nested build program/argv so the existing build is
   exactly `rustup run 1.89.0 cargo build -p substrate --bin substrate --locked`. Do not change the
   helper's locking, binary selection, messages, or callers.

Forged witnesses, disabled validation, unrelated installed binaries, broader fixture rewrites,
increased timeouts, and changed assertions remain forbidden. The same minimal harness-only overlay
is permitted for those two matched integrations on each side. The baseline commit remains immutable:
the baseline proof subject must be labeled as exact
`2b2fc6c50b40046dbeaeb5b316562fbd96480a2a` plus a separately fingerprinted baseline harness
overlay, while the candidate subject is the preserved product fingerprint above plus a separately
fingerprinted candidate harness overlay. Each overlay must contain only the three approved hunks in
the two named files and retain its exact full-index binary patch digest plus pre/post file SHA-256
values. It grants no baseline product repair, assertion change, or alternate comparison commit.
Each side's receipt must bind its actual source identity, Rust 1.89.0 toolchain, built executable
path/hash, and isolated mutable HOME, `SUBSTRATE_HOME`, target, socket, runtime, and temporary state.
Use the existing side-specific checkouts and harness; do not create a full source copy or a bespoke
proof framework.

For `E3A-P2-006`, the only permitted E3-A inherited-product Clippy difference is:

- file `crates/shell/src/execution/agent_runtime/retained_worker_runtime.rs`;
- symbol `reject_before_registration_in_registry`;
- diagnostic `clippy::too_many_arguments`, severity `error` under `-D warnings`, message
  `this function has too many arguments (9/7)`;
- owning product base `5e972add652224b97323f6abd496e94440d72242`, file blob
  `ea701ea539a1dc85c72a1a2c4b3c843696c3570b`, and file SHA-256
  `91e2bddab7a6c0f3d13d412c8ca86396973deed16dec118eeea878114a55482c`; and
- retained command targets `transport-api-types`, `transport-api-client`, `world-mac-lima`, `shell`,
  and `world-service` with `--lib --tests --locked -- -D warnings`. The baseline raw log contains
  zero copies of the exact message and the candidate raw log contains one. The directly resulting
  terminal summaries are baseline 12 versus candidate 13 errors for `shell` lib and baseline 11
  versus candidate 12 errors for `shell` lib test: exactly one additional error in each summary.

This is an explicit E3-A-only allowance over retained red Clippy logs, not an absolutely green
Clippy result and not permission to ignore every inter-lineage difference. The named file and symbol
must remain byte-identical to the owning product base, the raw diagnostic and terminal summaries
must be retained and reported, and no lint suppression or unrelated source repair is authorized.
Every other diagnostic identity, severity, multiplicity, terminal outcome, and semantic difference
remains subject to the existing differential rules; any E3-A-added diagnostic or expansion of this
single content-bound difference rejects completion. The comparison commit remains
`2b2fc6c50b40046dbeaeb5b316562fbd96480a2a`, not the product base.

The preserved review record at
`/home/spenser/__Active_code/review-evidence/e3-a-strict-v2-wire-5e972add/review/review-cycle-record.json`
remains unchanged at `bounded_stop`. `E3A-P2-001` and `E3A-P2-002` are fixed with reusable retained
proof; `E3A-P2-003` is deferred and nonblocking for Linux only by the controlling decision above;
`E3A-P2-004` and `E3A-P2-005` require only their bounded Linux recovery; `E3A-P2-006` requires the
exact allowance disposition above; and `E3A-P2-007` is mechanically corrected in preserved
evidence. Those are carried dispositions, not findings caused by the last remediation.

After the bounded recovery and proof—but not during this documentation correction—one new process-
evidence record may start a separate schema-version-1 review sequence solely for this scope-stop
re-entry. It must use the existing closed fields and validator without a new kind or validator
change. Its linked prompt/report must cite the preserved stopped record and bind one complete
subject fingerprint covering the bounded recovery delta and both overlay fingerprints, disposition
of `E3A-P2-003` through `E3A-P2-006`, relevant retained proof, and the final aggregate candidate
identity. The initial schema kind is `discovery`, but the reviewer receives one fresh focused
assessment of that recovery subject, not rediscovery of unchanged E3-A implementation. Do not copy
historical IDs into the new record, rewrite raw reviews, or append a fictitious
`supplemental_causal` cycle to the stopped record. `CLEAN` is terminal. If the focused assessment
finds a new valid `P1` or `P2` in its bounded subject, assign a new ID, perform one consolidated
in-fence remediation, and use a different fresh delta closure under the canonical procedure; no
open-ended review is authorized.

The final Linux completion decision must account explicitly for every carried `E3A-P2-001` through
`E3A-P2-007` disposition, the macOS deferral, the exact lint allowance, all relevant retained and
recovered proof, and the final candidate identity. Linux strict V1/V2 compatibility, fail-closed
behavior, and every other applicable Linux safety requirement remain mandatory. This platform
scheduling and proof-applicability correction is not a Linux safety waiver, does not itself complete
or land E3-A, and does not admit or dispatch E3-B.

## Admission fence and required proof

This specification is documentation authority only. The following is an ownership catalog partitioned
by the serial E3-A through E3-F work packets in the controlling slice; it is not one combined
implementation candidate. Every packet requires its own later fresh admission, exact source commit,
readiness receipt, impact analysis on each named symbol, explicit dispatch, and packet-bounded proof.
Success in one packet makes only its direct successor eligible to seek admission and never dispatches
that successor automatically.

For E3-B only, the later storage admission must additionally bind
`crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`,
specifically `StoreLayout::validate_closed_layout`, for the sole additive literal
`agent-config-projection-v1` directory match described above. No other function in that product path
and no weakening of the closed-layout validator is owned. It must also bind
`crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`,
specifically `validate_e2_rm_authority_manifest` for the identical sole literal match and the exact
`ConfigProjectionHsaParentTransactionV1::{begin,authority_fd,finish}` plus
`with_opened_config_projection_hsa_parent` bridge named above; no E2-RM authority data, read
semantics, history, reconciliation, schema, or namespace change is owned.
Subject to those corrections, later packet admissions may select only their applicable symbols from
this catalog:

1. create exactly `crates/config-projection/{Cargo.toml,src/lib.rs,src/codec.rs,src/registry.rs,
   src/service.rs,src/codex_0125.rs,src/linux_artifacts.rs}` and add only its dependency/export entries
   to workspace `Cargo.toml`, `crates/shell/Cargo.toml`, `crates/world-service/Cargo.toml`, and
   `crates/gateway/Cargo.toml`. `Cargo.lock` may change only the new `config-projection` package entry
   and the exact dependency-list edges caused by those named manifest edits; every unrelated package,
   version, source, checksum, feature, and dependency edge is byte-frozen. The new crate's manifest
   depends downward on `transport-api-types`.
   `transport-api-types` receives no dependency on the new crate. The new crate may expose only the
   data types named in this contract and `managed-gateway-adoption-v1` other than the explicitly
   transport-owned wire types above, plus these operational symbols. E3-D alone owns the narrow
   security-field correction in `src/lib.rs`: add the embedded `E3LinuxIdMapExtentV1`,
   `E3UserNamespaceRequirementV1`, and `E3UserNamespaceAttestationV1` types; replace only the Yama
   field in `E3WorldFsEnforcementInputV1` and `E3ChildSecurityAttestationV1`; and add the exact child
   namespace device/inode fields to `E3GatewaySecretReadyAttestationV1`. It owns no other projection,
   gateway, record, receipt, or codec-schema change. The admitted operational symbols are:
   `ConfigProjectionCodecV1::{encode_canonical_json,decode_canonical_json,domain_sha256}`,
   `ConfigProjectionRegistryV1::{open,recover,resolve,publish_dormant,publish_ready_closed,
   publish_active,publish_native_source,realize_native_root,recover_native_realization,
   acquire_consumer_lease,release_consumer_lease,publish_redacted_diagnostic,
   publish_kernel_effect_intent,publish_kernel_effect_resolution,
   publish_child_cgroup_registration,publish_child_process_registration,
   publish_terminal_child_evidence,retire,
   import_runtime_artifacts}`,
   `AgentConfigProjectionServiceV1::{new,publish_retained_launch_inputs,
   publish_retained_fork_inputs,publish_prepared_retained_launch,
   publish_prepared_retained_fork,publish_preparation_abandonment,
   resolve_activation_carrier,
   activate_managed_gateway,prepare_codex_launch,
   validate_resumed_turn,release_codex_launch,revoke_retained_runtime}`,
   `ConfiguredAcceptedHomeAuthorityV1::{from_installed_bootstrap_authority,from_record_for_test,
   revalidate,accepted_home,intended_uid}`,
   `ConfigProjectionHsaAuthorityV1::with_locked_parent`,
   `CanonicalDirectoryV1::{capture_linux_from_fd,revalidate_linux_from_fd}`,
   `Codex0125ProjectionV1::{render,validate_loader_inputs,validate_setup_ready}`, and
   `LinuxArtifactSourceV1::{validate_store,resolve_ref,import_manifest,
   validate_e3_static_elf_v1,validate_system_config_mount_target_v1}`. The obsolete
   `validate_host_ptrace_posture_v1` name is deleted rather than renamed or left with artifact-source
   ownership; host process security belongs to E3-D's world-service child-security path below.
   For the bounded installed-CA corrective delta only, E3-D may add private
   `resolve_exact_installed_ca_bundle_v1`, call it only from the otherwise-frozen support-file opening
   inside `LinuxArtifactSourceV1::import_manifest`, and change that file's colocated `mod tests` in
   `crates/config-projection/src/linux_artifacts.rs`, solely to implement the exact logical-CA
   resolver and final-descriptor validation above. The exception adds no public symbol, schema,
   registry behavior, manifest field, alternate pathname, or resolution rule for another support
   object.
   Colocated `mod tests` blocks
   are allowed. No other `pub`/`pub(crate)` surface is admitted. Within only the E3-B-owned
   `src/codec.rs` and `src/registry.rs`, ordinary private free functions, private inherent methods,
   and private implementation types necessary to implement the admitted codec/registry operations
   are allowed without a separate authority amendment; they may not expand the public API, product
   behavior, file ownership, or later-packet scope. Private helpers in any other new-crate file remain
   unadmitted;
2. in `crates/shell/src/execution/config_model.rs`, only new private
   `E3PinnedConfigPatchSourceV1`, `E3EffectiveConfigResolutionSnapshotV1`,
   `open_e3_workspace_config_source_v1`, `resolve_e3_effective_config_source_v1`, and
   `extract_e3_effective_config_source_v1`. They may call the existing
   `parse_config_patch_yaml`, `parse_env_overrides`, and `resolve_effective_from_layers` without
   modifying them, may call existing `workspace::find_workspace_root` only to select the candidate
   whose physical identity is then descriptor-validated. In
   `crates/shell/src/execution/agent_runtime/host_session_authority/facade.rs`, only new sealed
   `OpenedBootstrapConfigSourceV1`, new `OpenedBootstrapHomeV1::open_e3_config_source`, and
   `OpenedBootstrapConfigSourceV1::{physical_root_path,source_bytes,
   validate_e3_public_root_identity,revalidate}`. They may compose existing `TrustedAuthorityRoot`,
   `TrustedDirectory::{entry_kind,open_file}`, and `TrustedFile::read_all` without changing
   `trusted_fs.rs`; they expose no private authority type or descriptor and leave existing
   `OpenedBootstrapHomeV1::read_config_yaml` unchanged. Existing
   `resolve_effective_config_with_explain_for_bootstrap_home`, cached/pathname loaders,
   `ConfigExplainV1`, `ConfigExplainKey`, `ConfigExplainSource`, and all non-E3 config resolution
   remain byte/behavior-frozen. For E3-B in that same facade file, only public
   `OpenedConfigProjectionHsaAuthorityV1`, its
   `from_configured_accepted_home` constructor, and its `ConfigProjectionHsaAuthorityV1`
   implementation are additionally admitted. The constructor and implementation may call only the
   exact configured-authority accessors and shell parent-transaction bridge named above. E3-B may
   also add only: `with_config_projection_hsa_parent` as the one `pub(super)` forwarding operation in
   `host_session_authority/store.rs`; `TrustedDirectory::borrow_fd` as `pub(crate)` in
   `host_session_authority/trusted_fs.rs`; and one direct public re-export of
   `OpenedConfigProjectionHsaAuthorityV1` from `crates/shell/src/lib.rs`, without making any
   intermediate HSA module public. No other HSA type, descriptor, operation, or facade visibility is
   admitted. All of these bridge additions and the re-export are Linux-gated; other platforms retain
   their existing unsupported E3 posture. In
   `crates/shell/src/execution/agent_inventory.rs`, only additive `AgentFileV3`, `AgentConfigV3`,
   `AgentPlacementsV3`, `AgentPlacementConfigV3`, `AgentRuntimeProjectionInputV1`,
   `PlacementProjectedInventoryEntryV3`, and `AgentInventorySourceMaterialV1`, the V3 variants/arms in
   existing `ParsedAgentInventoryFile`, `parse_and_validate_agent_file_raw`, `validate_agent_file`,
   `merge_inventory_root`,
   `load_effective_agent_inventory`, `load_effective_agent_inventory_for_bootstrap_home`, and new
   `validate_agent_schema_v3` and `project_inventory_v3_entry`. The legacy
   `validate_agent_file -> AgentFileV1` entry point adds only an exhaustive V3 arm returning
   `user_error("unsupported agent schema_version 3 in legacy validate_agent_file")`; the E3 caller
   maps that disposition to `UnsupportedLegacyState`, and no path down-converts V3. V1/V2 behavior
   is frozen;
3. in `crates/transport-api-types/src/lib.rs`, only new `ConfigProjectionRefV1`,
   `ConfigProjectionAuthoringInputRefV1`,
   `InWorldGatewayRefV1`, `ManagedGatewayActivationIntentRefV1`,
   `ConfigProjectionActivationCarrierV1`, their strict private decode forms,
   `E3ConfigProjectionPrepareRequestV1`, `E3ConfigProjectionPrepareResponseV1`,
   `E3ConfigProjectionCancelRequestV1`, `E3ConfigProjectionCancelResponseV1`,
   `E3ConfigProjectionCancelDispositionV1`, their strict private decode forms and validation,
   `MemberDispatchRequestV2`, `MemberDispatchRequestV2Def`, `MemberDispatchRequest`, and
   `MemberDispatchCommonFieldsV1`; their
   `validate`, strict `Deserialize`/`TryFrom`, `common`, `as_v1`, `as_v1_mut`,
   `e2_launch_activation`, and
   `config_projection` implementations; and the type/strict-codec arms for existing
   `ExecuteRequest`, `ExecuteRequestDef`, `ExecuteRequest::validate`, and
   `TryFrom<ExecuteRequestDef> for ExecuteRequest`. Existing `MemberDispatchRequestV1`,
   `MemberDispatchRequestDef`, and V1 fixtures are byte/behavior-frozen. In
   `crates/world-mac-lima/src/lib.rs`, the preserved E3-A candidate already contains only the
   existing `convert_member_dispatch` return-type/body adaptation and its call in
   `convert_exec_request`: it converts the existing `world_api::MemberDispatchRequestV1` exactly as
   at the baseline and wraps that value as `transport_api_types::MemberDispatchRequest::V1`. That
   mechanical conversion remains frozen during Linux proof recovery. It cannot construct, accept,
   down-convert, or claim V2/E3, and the untagged wrapper must leave the serialized V1 object byte-
   identical. Its presence grants no native macOS proof or implementation authority;
   in `crates/transport-api-client/src/lib.rs`, only new strict
   `e3_config_projection_prepare` and `e3_config_projection_cancel` methods posting to the two fixed
   routes above; existing gateway and execute client methods are frozen;
4. in `crates/shell/src/execution/routing/dispatch/world_ops.rs`, only existing
   `MemberDispatchTransportRequest`, `ExecuteRequestInput`, `build_execute_request`,
   `build_member_dispatch_payload`, `build_agent_client_and_member_dispatch_request`,
   `build_agent_client_and_member_dispatch_request_for_cwd`, and the Linux
   `build_agent_client_and_member_dispatch_request_impl`; in
   `crates/shell/src/execution/orchestrator_world_dispatch.rs`, only existing
   `prepare_orchestrator_world_dispatch`, both cfg-specific `fork_world_worker` and
   `spawn_world_worker` definitions, `prepare_spawn_world_worker_bootstrap`,
   `prepare_authority_bound_spawn_world_worker`, `spawn_prepared_world_worker`,
   `prepare_fork_policy_commitment`, `member_dispatch_transport_request_from_typed`,
   `build_spawn_world_worker_transport_request`, `build_fork_world_worker_transport_request`,
   `continue_world_worker_fork_command_bootstrap_after_delivery`,
   `build_continue_world_worker_fork_command_transport_request`, and
   `execute_spawn_world_worker_stream`, plus existing `prepare_task_acceptance_submission` and
   `validate_task_submission_against_proposal` solely to borrow the exact V1 variant before storing
   or comparing the unchanged `WorldWorkSubmissionIdentityV1::EphemeralTask.member_dispatch_request`,
   and new `author_e3_projection_for_spawn` and
   `author_e3_projection_for_fork`; and in `crates/shell/src/repl/async_repl.rs`, only existing
   `PreparedAgentRuntime`, `prepare_member_runtime_startup_from_authority_registration`,
   `build_member_dispatch_transport_request`, `start_internal_dispatch_member_runtime`,
   `start_remote_member_runtime_with_prepared`, `start_remote_member_runtime`, and
   `start_remote_member_runtime_with_binding_retry_using`. V2 is emitted only for exact E2 retained
   launch/fork authority; task and non-E2 compatibility construction remains V1. In
   `crates/shell/src/builtins/world_gateway.rs`, only existing `resolve_integrated_auth_payload`,
   `resolve_cli_codex_integrated_auth`, and `codex_auth_state_path`, plus new
   `resolve_e3_integrated_auth_payload`, may be extracted/reused solely to produce the transient
   prepare-request field; compatibility gateway context/sync/restart behavior is frozen;
   `crates/shell/src/execution/agent_runtime/state_store.rs` and its durable
   `WorldWorkSubmissionIdentityV1::EphemeralTask.member_dispatch_request:
   MemberDispatchRequestV1` remain completely unchanged; the two named B1 call sites must reject a
   non-V1 wrapper rather than widen, rewrite, or migrate that authority;
5. in `crates/world-service/src/main.rs`, only existing `main` plus new
   `park_e3_child_transition_capabilities_before_runtime`,
   `verify_e3_transition_capabilities_parked`, and
   `build_primary_runtime_after_e3_capability_parking`. `main` may change only from the
   `#[tokio::main]` form to the specified synchronous internal-exec-first/capability-parking/manual-
   runtime sequence; its existing internal-exec and `run_world_service` outcomes remain unchanged. In
   `crates/world-service/src/lib.rs`, only the `e3_config_projection_prepare`, `e3_codex_launch`,
   `e3_child_security` and `e3_local_transport` module declarations;
   existing `run_world_service`, `run_uds_server`, `build_router`, and `spawn_periodic_gc`; and new
   `lock_e3_service_secret_memory_v1` plus new
   `build_e3_authenticated_uds_router`, solely to keep the two fixed E3 preparation routes off the
   baseline TCP/direct-bind router, register them only on the exactly validated inherited Linux UDS,
   and pass the startup-recovery or non-E3 GC admission described above. The existing
   `crates/world-service/src/socket_activation.rs` collector is consumed without modification;
   accepted socket names, `LISTEN_PID`/`LISTEN_FDS` semantics, TCP inheritance, and all non-E3
   behavior are frozen. In
   `crates/world-service/src/service.rs`, only additive
   `WorldService.config_projection_service`, `WorldService.e3_projection_preparations`,
   `WorldService.accepted_home_authority`, and
   `WorldService.privileged_child_exclusion` fields plus additive
   `WorldService::new`, `WorldService::new_linux`,
   `WorldService::new_with_privileged_child_exclusion`,
   `WorldService::new_with_member_turn_state_root_for_test`, `WorldService::execute`,
   `WorldService::execute_stream`, `WorldService::pending_diff`,
   `WorldService::pending_diff_clear`, `WorldService::pending_diff_reconcile`,
   `WorldService::world_fs_read`,
   new `WorldService::new_with_member_turn_and_e3_authority_for_test`,
   new `WorldService::e3_config_projection_prepare`,
   new `WorldService::e3_config_projection_cancel`,
   new `WorldService::acquire_non_e3_helper_operation`,
   new `WorldService::ensure_non_e3_world_session`,
   `WorldService::resolve_authoritative_member_placement_context`,
   `requested_shared_world_owner_spec`, `exact_bound_world_ownership_adoption`,
   `convert_member_dispatch_request`, and `validate_member_dispatch_binding`. In
   `crates/world-service/src/member_runtime.rs`, only new `MemberRuntimeLaunchAdmissionV2`,
   `ActiveE3ConfigProjectionRuntimeV1`, `ActiveE3ProjectionLifecycleV1`, the additive
   `ActiveMemberRuntime.e3_projection` field, `MemberRuntimeManager.e3_projection_service`,
   `MemberRuntimeManager.privileged_child_exclusion`,
   `MemberRuntimeManager::with_replay_and_e3_projection_service`,
   `MemberRuntimeManager::launch_v2`,
   `ActiveE3ConfigProjectionRuntimeV1::{admit_and_reserve_resumed_turn,
   mark_resumed_turn_prepared,release_resumed_turn_final_exec,clear_resumed_turn,
   revoke_and_release}`, and existing `MemberRuntimeManager::launch`,
   `MemberRuntimeManager::submit_turn`, `MemberRuntimeManager::submit_e2_turn`,
   `MemberRuntimeManager::register_member`, `MemberRuntimeManager::unregister_member`,
   `MemberRuntimeManager::finish_bootstrap`, `MemberRuntimeManager::unregister_turn`,
   `validate_retained_worker_launch_authority_proof`, `validate_e2_member_launch_activation`,
   `validate_member_runtime_binary`, `prepare_runtime_env_for_member_backend`,
   `prepare_codex_runtime_env`, and `prepare_member_runtime_launcher`. Existing
   `MemberRuntimeManager::{new,with_replay_registry}` may initialize no E3 service and must reject V2;
   After the common process-wide pre-spawn security admission, V1 construction, submission,
   registration, and cleanup behavior is frozen. In
   `crates/world-service/src/prompt_fulfillment.rs`, only `PromptFulfillmentBridge`, new
   `PromptFulfillmentBridge::for_e3_codex`, and `PromptFulfillmentBridge::run_control`; existing
   `PromptFulfillmentBridge::for_member_backend` and the V1 `GatewayAdapterRuntime`/UAA branch are
   frozen after the common process-wide child-exclusion admission. In
   `crates/world-service/src/pty.rs`, only existing `handle_ws_pty`, `handle_legacy_start`,
   `handle_persistent_session`, `spawn_legacy_ws_exec`, and `spawn_persistent_exec`, solely to
   acquire/hold/release the same non-E3 child lease around a spawned PTY lifetime; PTY protocol,
   rendering, tracing, and execution behavior are otherwise frozen. In
   `crates/world-service/src/handlers.rs`, only existing `doctor_world`, `pending_diff`,
   `pending_diff_clear`, `pending_diff_reconcile`, `world_fs_read`, and `gc` plus new
   `e3_config_projection_prepare` and `e3_config_projection_cancel`; the five observation handlers
   change only to obtain/hold the exact helper-operation admission above, and `gc` changes solely to
   obtain the service's non-E3 GC admission. In `crates/world-service/src/gc.rs`, only existing `sweep`, `list_netns`,
   `netns_pids`, `delete_nft_table`, and `delete_netns`, solely to require and retain the exact
   `E3GcSweepAdmissionV1` across every `Command`; GC selection and deletion semantics are frozen;
6. create exactly `crates/world-service/src/e3_config_projection_prepare.rs` with only
   `E3ConfigProjectionPreparationManagerV1::{new,prepare,cancel,take_for_v2,recover_expired}` and
   its private `SealedE3ConfigProjectionPreparationV1`, `SealedCredentialSourceCapabilityV1`, and
   `ClockBoottimeDeadline`; create exactly `crates/world-service/src/e3_codex_launch.rs` with
   `E3Codex0125LaunchAdapterV1::{new,prepare,spawn_setup_wrapper,validate_setup_ready,
   release_final_exec,cancel}` and `crates/world-service/src/e3_child_security.rs` with only
   `bind_e3_service_user_namespace_v1`,
   `build_authenticated_world_fs_enforcement_input_v1`,
   `create_e3_child_user_namespace_channel_v1`,
   `install_and_validate_e3_child_user_namespace_v1`,
   `validate_child_security_attestation_v1`, `probe_child_control_path_denials_v1`, the private
   `HeldE3ServiceUserNamespaceV1`, private `HeldE3ChildUserNamespaceV1`, and
   `E3PrivilegedChildExclusionV1::{new_recovering,register_recovered_non_e3_child,
   finish_recovery,acquire_non_e3_child,
   acquire_e3_exclusive,release_non_e3_child,release_e3_exclusive,poison_recovering}`. In
   `crates/world-service/src/e3_local_transport.rs`, create only
   `E3AuthenticatedLinuxUdsListenerV1::from_inherited`,
   `E3AuthenticatedLinuxUdsListenerV1::accept_peer`, and the private authenticated-listener/peer
   types specified above. In
   `crates/world-service/src/internal_exec.rs`, only extraction of the landed policy path into
   `resolve_authenticated_world_fs_enforcement_plan_v1` and
   `apply_authenticated_world_fs_enforcement_plan_v1`; `run_landlock_exec` must call those extracted
   functions with byte-identical V1 behavior. The E3 wrapper may call the resolver but must apply the
   separately hashed derived-support and role-narrowing layers specified above. Create
   `crates/world-service/src/bin/substrate-world-entry.rs` with only `main`,
   `parse_launch_descriptors`, `prepare_private_child_namespace`,
   `apply_authenticated_world_fs_enforcement`, `drop_child_privileges_and_caps`,
   `install_child_seccomp`, `write_setup_ready_attestation`, `await_final_exec`, and
   `exec_pinned_child`, plus `run_managed_gateway_readiness_probe` for the exact no-final-exec probe
   role. `parse_launch_descriptors` owns the exact `SUBSTRATE_WORLD_ENTRY_USERNS_FD` startup pointer,
   and `prepare_private_child_namespace` owns only the child-side direct-unshare/setup handshake and
   subsequent private mount-namespace setup; neither may accept a caller namespace or use `setns`
   for user-namespace entry. For the bounded installed-CA corrective delta only,
   `validate_runtime_support_manifest` may add private `resolve_exact_e3_ca_bundle_v1`, and
   `prepare_private_child_namespace` may add and call private
   `install_exact_e3_ca_bundle_mount_v1`, solely for the exact descriptor validation and private
   single-file realization above before `apply_authenticated_world_fs_enforcement`. The existing
   Landlock implementation and path-policy model are unchanged; no generic resolver, directory rule,
   or symlink-capable Landlock surface is owned. In
   `crates/world-service/src/gateway_runtime.rs`, only existing `GatewayRuntimeManager`,
   `GatewayRuntimeManager::{new,status,sync,sync_with_timeout,sync_with_timeout_locked,restart}`,
   `start_runtime`, `stop_runtime`, `recover_runtime`, `runtime_for_world_or_manifest`,
   `resolve_gateway_binary`,
   `prepare_gateway_auth_bundle_handoff`,
   `create_inherited_auth_bundle_pipe`, plus new `create_e3_gateway_secret_ready_pipe` and
   `E3GatewayRuntimeAuthorityV1::{new,bind_dormant_listener,
   install_dormant_boundary,listen_after_dormant_boundary,
   spawn_descriptor_pinned,validate_child_security,
   validate_listener_inventory,validate_gateway_secret_ready,
   deliver_secret_after_privilege_drop,spawn_readiness_probe,
   validate_readiness_probe,probe_readiness,
   allow_exact_member,revoke}` plus only the additive
   `GatewayRuntimeManager.privileged_child_exclusion:
   Arc<E3PrivilegedChildExclusionV1>` and
   `ManagedGatewayRuntime.privileged_child_exclusion_lease:
   Arc<HeldNonE3PrivilegedChildLeaseV1>` field held for a non-E3 gateway lifetime.
   Existing `prepare_linux_world_entry_launcher` and
   `render_linux_world_entry_wrapper`, `gateway_health_ready`, and
   `gateway_health_ready_blocking` remain V1-only and frozen;
7. in `crates/gateway/src/launch.rs`, only existing `GatewayLaunchContract`,
   `GatewayLaunchContract::resolve`, and new
   `E3GatewayLaunchContractV1::{from_environment,consume_launch_input,adopt_listener,
   lock_and_attest_secret_ready}`; in
   `crates/gateway/src/main.rs`, only existing `resolve_launch_and_config` and `start_foreground`; and
   in `crates/gateway/src/server/mod.rs`, only existing
   `IntegratedGatewayAuthContext::from_auth_bundle_env`, `take_auth_bundle_fd_env`,
   `read_gateway_auth_bundle_from_env`, `AppState`, and `start_server`, plus new
   `build_e3_in_world_app`, `serve_e3_inherited_listener`, `e3_validate_member_identity`, and
   `e3_health_check`. Existing `build_app`, `health_check`, every OAuth/token/messages/chat route,
   and the non-E3 main/port-1455 bind behavior are frozen and unreachable in E3 mode;
8. E3-C owns the following publication machinery without owning production of the E3-D wrapper: in
   `crates/shell/src/builtins/world_deps/inventory.rs`, only existing
   `codex_runtime_install_script_template_v1` plus new
   `render_codex_installer_artifact_source_v1`; in
   `crates/shell/src/builtins/world_deps/surfaces.rs`, only existing
   `CodexRuntimeInstallSpecV1`, `resolve_codex_runtime_install_spec_for_target_v1`, and
   `render_codex_runtime_install_script_v1`. The version remains exactly `0.125.0`. In
   `scripts/linux/world-lifecycle.sh`, only existing `record_linux_managed_state`,
   `install_linux_managed_state`, and `restore_linux_managed_state` for the E3-C-owned system-config,
   installed-home-bootstrap, and source-store lifecycle changes, plus new shell functions
   `provision_e3_system_config_mount_target_v1`, `publish_installed_home_bootstrap_v1`, and
   `publish_substrate_artifact_source_v1`. E3-C's production lifecycle integration may call the
   system-config and installed-home-bootstrap helpers, but it must not invoke the Substrate artifact
   publisher as a production success path. It tests the Substrate helper with valid bounded inputs
   that are neither production artifact authority nor installed-runtime readiness evidence. In
   `scripts/linux/world-provision.sh`, E3-C owns only existing `resolve_install_bootstrap_context` and
   the exact carrier/account/UID/primary-GID values passed into `install_linux_managed_state` for its
   owned bootstrap behavior.

   E3-D owns the corresponding Substrate production integration: only the later additive changes to
   existing `install_linux_managed_state` needed to build/install/readback-validate the two real
   Substrate artifacts and invoke the landed Substrate publication helper without a missing-artifact
   success path; and in `scripts/linux/world-provision.sh`, only the `cargo build` target list,
   `SERVICE_UNIT_CONTENT` (including explicit `CAP_SETUID`, `CAP_SETGID`, and `CAP_SETPCAP` solely for
   the verified one-way child transition plus `SecureBits=noroot-locked`), and the exact source-build
   values passed into `install_linux_managed_state`. The new ELF target is added to the existing
   world-service package; E3-D provisioning builds
   and installs `substrate-world-entry` and the E3-eligible `substrate-gateway` for exact
   `x86_64-unknown-linux-musl`, rejects either ELF unless it is exact `StaticExec` or constrained
   relocation-only `StaticPie` under support policy V1, and does not replace the baseline host-
   target world-service daemon or V1 gateway artifact. It must publish and read back the complete
   valid Substrate record/head before dependent E3 runtime use; no V1 fallback, placeholder ELF,
   invented provenance, partial manifest, silent missing-artifact success, or stale record for
   replacement bytes is admitted. For the bounded installed-CA corrective delta only, E3-D may also
   change the private `file_support` logic inside
   `scripts/linux/world-lifecycle.sh::publish_substrate_artifact_source_v1` and the `e3c_tests`
   module in `crates/shell/src/builtins/world_deps/inventory.rs`, solely for the direct-file and exact
   relative-symlink cases above. Lifecycle order, artifact-production requirements, other support
   objects, and every other E3-C publication/import surface remain frozen; and
9. only the exact V2-to-V3 Codex world placement change in `config/agents/codex.yaml`, setting the
   fixed installed Codex path, `model: codex`, and empty MCP/feature lists. No host placement meaning
   changes; and
10. create only `crates/config-projection/tests/agent_config_projection_v1.rs` and
    `crates/world-service/tests/e3_config_projection_activation_v1.rs` for the E3 acceptance walls.
    Existing V1 fixtures may receive only imports, mechanical
    `Some(MemberDispatchRequest::V1(MemberDispatchRequestV1 { ... }))` wrapping, and assertions of
    byte/behavior identity in
    `crates/transport-api-types/src/lib.rs::execute_request_member_dispatch_round_trip`,
    `crates/shell/src/execution/orchestrator_world_dispatch.rs::{sample_task_acceptance_submission,
    make_member_dispatch_execute_request}`,
    `crates/shell/tests/support/repl_world_service.rs::{assert_member_dispatch_capture,
    ReplWorldAgentStub::start_with_overrides}` solely to require `as_v1` before the existing V1
    capture/response script,
    `crates/shell/tests/repl_world_first_routing_v1.rs::persist_test_fork_child_participant_manifest`
    and that file's existing typed `ExecuteRequest.member_dispatch` field-access sites solely to
    require `as_v1` before the unchanged V1 persistence/assertion logic,
    `crates/world-service/tests/member_runtime_world_placement_v1.rs::make_member_dispatch_request`,
    `crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs::{make_member_dispatch_request,
    attach_exact_retained_launch_authority}`, with the latter using `as_v1_mut` before its unchanged
    V1 authority attachment,
    `crates/world-service/tests/member_turn_idempotency_v1.rs::launch_harness`, and
    `crates/world-service/tests/streamed_execute_cancel_v1.rs::make_member_dispatch_request_with_backend`.
    Colocated test helpers/call sites in already named product files may make only that same wrapper
    adaptation. No baseline fixture value, V1 assertion, test skip condition, or product behavior may
    change to accommodate E3.

No file-wide authority is granted. D1 envelope code, receipt/manifest/StateStore/E2/E2-RM/B2.2,
broker, D2, E4, unrelated gateway providers, public UX, and all new non-Linux product changes are
excluded. The already-present byte-preserving Lima V1 wrapper conversion named in item 3 remains
frozen compatibility material rather than proof-recovery ownership.
The implementation must stop for a new authority decision if correct closure requires a symbol or
file outside this fence.

The later admission must require proof using candidate Substrate binaries built from source on Linux
and the descriptor-pinned official Codex archive above for:

- canonical codec/hash/reference negatives and every typed resolution class;
- E3-owned Linux `CanonicalDirectoryV1` capture/reopen/device/inode/path negatives and proof that no
  config-projection or transport dependency reaches shell's private host-session authority types;
- strict V3 inventory duplicate/alias/merge/unknown/newer/legacy cases, exact global/workspace
  shadowing, explicit `model: codex`, empty/nonempty list semantics, source revisions/hashes, and proof
  that V1/V2 cannot source E3 model/MCP/feature values;
- first-writer/CAS, two-process conflict, every temp/rename/file-fsync/directory-fsync/readback crash
  boundary, exact retry, reopen, retention, and retirement;
- immutable accepted-home native-source publication and separate `/run` realization at every
  directory-create/copy/manifest/fsync/rename/readback crash boundary, including wrong UID/GID,
  config mutation, read-only bind failure, mutable-state non-aliasing, safe pre-ownership recovery,
  and kill/revoke-before-cleanup after possible ownership;
- every cgroup and nftables boundary crash at intent-before-effect, effect-before-effect-record,
  effect-record publication, recovery-resolution publication, and resolution-readback boundaries,
  proving fixed-name absence or exact safe reversal, no unrecorded/adopted/orphaned kernel object,
  and closed E3/V1 admission until every intent has either its complete effect record or one durable
  exact-readback recovery resolution;
- zero-live publication and crashes before/after intent, FD delivery/consume, ACK, fence release, exec,
  and first provider request, with no double child, gateway, handoff, or release;
- inherited-E3-transport fixtures proving route registration only on one systemd-activated
  `/run/substrate.sock` descriptor with `AF_UNIX`/`SOCK_STREAM`/`SO_ACCEPTCONN=1`, distinct valid
  descriptor and pathname socket identities, pathname owner `root`, service-effective GID, mode
  `0660`, and link count one; rejection before body read for zero/multiple inherited UDS listeners,
  inherited TCP, direct-bind UDS, baseline TCP, wrong path/type/owner/GID/mode/link count,
  accepted-home account drift, wrong `SO_PEERCRED` UID, invalid peer PID/start time, and boot-ID
  drift; and proof that a transport-level 404 never retries the secret body elsewhere;
- bound-before-listen ordering and race fixtures proving `SO_ACCEPTCONN=0` after the unique
  `127.0.0.1:0` bind, connection refusal with no queued bytes before the deny boundary is effective,
  exactly one `listen(16)` only after boundary publication/readback, immediate `LISTEN` plus empty
  zero-time-`ppoll` proof, and denial of every unauthorized connect racing the post-listen check;
  crashes or observation failures at bind, boundary intent/effect/readback, listen, and empty-queue
  proof must close the descriptor, revoke/remove the boundary, and publish no Dormant head;
- preparation-plus-V2 reachability proving the shell publishes only immutable nonsecret authoring
  inputs, sends secret material only to the strict prepare route, receives and carries only
  Dormant/no-ACK authority, and world-service advances that exact predecessor through
  ACK/ReadyClosed/Active within `launch_v2`, with no secret in ExecuteRequest, caller-authored ACK,
  rewritten carrier, or cyclic reference;
- prepare response-loss/exact-retry, conflicting retry, cancel, fixed expiry, service-restart, and
  prepare-versus-V2 races proving the credential/listener/exclusion capability is held once, the
  route spawns no child, retried secret bytes are scrubbed without comparison, an old carrier cannot
  activate, and fresh credentials create a new fence in the same unchanged-subject series;
- the prebuilt post-fork syscall-only gate, wrapper setup-ready attestation, parent namespace/mount
  readback, final-exec gate, and crashes/EOF/duplicate bytes at every two-stage boundary;
- the synchronous `main` internal-exec-first branch, sole-thread transition-cap parking/readback,
  runtime construction only afterward, and every Tokio worker-start mismatch abort, including proof
  that no thread exists before parking and late helper threads inherit the verified parked state;
- source-type/codec fixtures proving both E3 E2-link schemas embed the exact public
  `DispatchPolicyCommitmentRefCarrierV1` and `PolicyRefV1` bytes and validation, with no shell-private
  type, undefined mirror, lossy conversion, or dependency cycle;
- support-policy V1 fixtures proving each artifact is static musl with null/empty loader/dependency
  fields; acceptance of both exact `StaticExec` and constrained relocation-only `StaticPie`, including
  the descriptor-pinned Codex 0.125 `ET_DYN` artifact; rejection of every interpreter, `DT_NEEDED`,
  forbidden dynamic tag, non-relative relocation, nonempty dynamic symbol/string payload,
  loader/dependency/architecture mismatch; direct-regular-file CA success; success for the exact
  logical CA path resolving through the observed trusted relative
  `../../ca-certificates/extracted/tls-ca-bundle.pem` layout beneath `/etc`; publisher/importer/wrapper
  agreement on the logical pathname and consumed final device/inode/mode/length/hash; and rejection
  of a boundary escape, absolute or magic link, cycle or second traversal, different otherwise-safe
  relative target, untrusted or writable component, nonregular endpoint, link/endpoint substitution,
  or final identity/hash drift. The
  privately realized logical CA path must reopen as the same regular object and remain readable under
  the existing exact-file Landlock rule without a CA-directory/discovery rule. Existing direct-file
  support-manifest fixtures and every non-CA support-file symlink rejection remain compatible;
  every fixed device/regular file; exact explicitly enumerable subtree closure; rejection of
  every E3-synthesized ancestor discovery rule, including `/`, `/etc`, `/usr`, `/lib*`, `/dev`, an
  accepted-home ancestor, or a realization parent; and proof that no directory-wide CA or ambient
  path can enter a self-consistent hash; numeric self-proc descriptor success plus rejection of
  `/proc/self`, another PID, PID reuse,
  a non-procfs mount, or any magic/symbolic-link resolution; first-provision/exact-existing
  `/etc/codex` target success plus nonempty/symlink/mount/metadata/identity/crash failure, with no
  per-child shared-root creation or cleanup;
- for every gateway, readiness probe, and initial/resumed Codex child, proof that the primary service
  remained in its bound host user namespace while the wrapper created a fresh per-process-tree user
  namespace before exposure; exact trusted-service owner, held parent descriptor, one-extent
  identity UID/GID maps, actual child membership, and distinct sibling/other-role namespace identities;
  rejection of wrong/substituted owner, parent, map, descriptor, member, reused identity, shared role
  namespace, host-root mapping, caller namespace, or premature setup release; and exact target UID/GID
  with zero supplementary groups and zero ambient/effective/permitted/inheritable/bounding
  capabilities, `NoNewPrivs=1`, wrapper/probe setup `Dumpable=0`, gateway post-exec secret-ready
  `Dumpable=0` plus soft/hard `RLIMIT_CORE=0` for its credential lifetime, non-secret Codex post-exec
  `Dumpable=1`, role-appropriate `TracerPid`, seccomp-filter mode, exact E2 plan hash plus derived-support,
  role-narrowing, and effective-intersection Landlock hash equality, including positive access to
  each required E3 `/run`/mounted-config object and denied access outside the union; denied
  registry/sibling/cgroup/nftables/world-service/other-role probes whose ordered concrete targets are
  byte-equal across the enforcement input, per-target hashes, emitted results, and parent-held
  identity revalidation, and failure before secret write or final release for every
  missing/extra/reordered/substituted target or transition/readback error, plus proof that the three
  parked transition capabilities are never effective/ambient in world-service except the exact
  synchronous `CAP_SETUID`/`CAP_SETGID` map-write scope, are parked and read back before child release,
  and are never acquired by any V1 or non-E3 child;
- a same-host-UID, zero-capability denial canary whose explicitly authorized, `Dumpable=1`, same-host-
  namespace positive control succeeds for ptrace, `process_vm_readv`, `/proc/<pid>/mem`, and
  `pidfd_getfd` before the identical inspector is denied against the post-exec protected target with
  exact `EPERM` for ptrace/process-vm, `EACCES|EPERM` for proc-mem, and successful `pidfd_open` allowed
  only when `pidfd_getfd` then returns `EPERM`; an unsuccessful control is inconclusive, not a pass;
  trusted service tracing across the protected boundary must retain its required initial exec stop,
  exit stop, cwd/environment readback, and applicable PTY `FIONREAD`/drain marker, while unrelated
  same-user host tracing retains the same result and errno before, during, and after E3, the distinct
  canary positive control succeeds, and the global Yama bytes are unchanged;
  invalid namespace identity or unavailable required tracing fails closed without changing host
  policy; and a target-UID mode-`0600` file retains ordinary target-user ownership and read/write
  access;
- service/gateway crash-dump barriers proving service `RLIMIT_CORE=0`/`PR_GET_DUMPABLE=0` readback
  precedes E3 route registration, an injected service barrier failure accepts no E3 body,
  and a canonical gateway secret-ready attestation precedes every secret delivery; missing, malformed,
  noncanonical, extra, wrong-bound, nonzero-limit, dumpable, tracer, early-EOF, or wrong-PID/start
  evidence closes the auth pipe unwritten, while neither process restores its zero posture;
- exactly one inherited gateway listener, no `127.0.0.1:1455` or other auxiliary bind, exact
  `GET /health` probe behavior, exact identity-gated `POST /v1/responses`, and 404/405 with no handler
  side effect for every other baseline method/path;
- readiness-probe cgroup enforcement proving the probe is the descriptor-pinned wrapper child rather
  than a world-service thread, receives and exact-validates the complete nested enforcement input and
  self-artifact descriptor through the eight-FD/three-barrier ABI, has the exact zero-capability/support/seccomp
  posture, cannot access secrets/config/workspace/control objects, is denied from the wrong cgroup,
  permits only the enumerated syscall/argument filter, proves the exact socket remains connected and
  in the readiness cgroup while the parent withholds the request-release byte, transmits its canonical
  connected attestation over the dedicated bounded pipe, and completes exactly one CRLF/framed,
  content-length/deadline-bounded request/result before exit; missing/truncated/extra objects, wrong
  socket type/tuple/inode, PID/start drift, or cgroup/boundary mismatch fails before secret delivery,
  ACK, or member release;
- two same-backend sibling retained workers in one world proving different roots/series/gateway access
  grants, denied cross-root reads, denied cross-gateway requests, and no mutable-state aliasing;
- process-wide exclusion races proving E3 cannot activate beside any live/unclassified ordinary,
  PTY, V1/UAA, compatibility-gateway, or GC helper child; every such request fails before spawn while E3 is
  exclusive; same-world-generation E3 siblings may join; secrets remain unreachable; the last E3
  teardown proves empty cgroups before restoring V1 admission; and recovery fails closed on an
  unclassified descendant; compatibility-gateway `sync`/timeout/restart/start and manifest-recovery
  paths all transfer one shared lease into the runtime before spawn/adoption, retain it through every
  runtime handle until cgroup-empty stop, and startup recovery seeds the exact legacy count; E3
  rejects boot-ID drift or any protected-user-namespace identity/membership drift before the relevant
  barrier; a non-secret Codex exec's `Dumpable=1` remains protected by its held user namespace and the
  positive-controlled same-UID memory/FD denial canary, while the credential-bearing gateway must
  remain in its separate held namespace, non-dumpable, and at both core limits zero;
- doctor/pending-diff/clear/reconcile/world-fs-read races proving each route holds one non-E3 lease
  before strategy selection, `ensure_session`, probe-file mutation, or helper spawn and returns typed
  `UnsupportedSecurityPosture` with no side effect during E3-exclusive mode; kernel-overlay, transient
  FUSE-probe, `ls` success/failure/timeout, and cleanup paths must retain the lease through complete
  child reap; a persistent `fuse-overlayfs` child or mount is detected by the mandatory pre-E3 and
  startup-recovery scans, never adopted, and prevents E3 until the process and mount are both proved
  absent; an incomplete reap/unmount proof leaves or returns the exclusion to `Recovering`, while
  idle V1 route bytes and behavior remain unchanged;
- startup, periodic, and manual GC races proving startup helpers finish under `Recovering`, a
  pre-existing sweep lease prevents E3 entry until every `ip`/`timeout` child is reaped, periodic
  ticks are skipped and manual GC is typed-failed while E3-exclusive, and no GC `Command` can start
  without the exact admission token;
- secret canaries absent from projection records/native bytes, argv, inherited environment,
  `/proc/<codex>/environ`, `/proc/<codex>/cmdline`, Codex/UAA open FDs, logs, traces, receipts,
  manifests, crash files, and gateway ACKs, including forced world-service crashes after prepare-body
  acceptance and forced gateway crashes after secret consumption under permissive host core policy;
  both must retain zero dumpability/core limits and create no credential-bearing dump, while the
  gateway alone consumes the one-time secret FD
  and Codex receives none of the gateway, wrapper, or readiness-channel descriptors; the readiness
  probe and parent close the probe socket and every pipe endpoint on success, rejection, timeout,
  crash, and unwind without ever carrying secret bytes; an arbitrary `SecureFd.fd_name`, any false
  delivery Boolean, or delivery mismatch between handoff and projection must fail before intent,
  secret write, ACK, or activation;
- installer/source-manifest/accepted-manifest missing, tamper, stale, `--skip-build`, early-return,
  archive-versus-extracted-digest, replacement/symlink/rename/inode/hash attacks for Codex, wrapper,
  and gateway, including exact nested runtime-support equality from source entry through accepted
  entry and projection plus rejection of a self-consistent outer rehash after any support mutation;
  root publishers must exclude a concurrent non-root import through exclusive versus shared installer
  locking, the intended `substrate`-group shell must import through a read-only `0640` lock and
  `0750` directory chain, and group write/exclusive-publication attempts must fail;
- redacted diagnostic, pre-spawn child-cgroup registration, pre-release child-process registration,
  and terminal-child evidence
  missing/malformed/hash/substitution/partial cases, exact parent `waitid` status, service-restart
  `ESRCH`/pidfd-readable-plus-`ECHILD` recovery observations without fabricated exit status, plus a
  twice-empty complete cgroup forest, immutable retirement publication,
  audit-only retired reads, `Retired` resolution precedence, and `RetiredSeries` rejection for every
  new carrier/lease/revision/activation/recovery attempt;
- installed accepted-home bootstrap record/head missing, forged carrier/projection, wrong account/UID,
  caller-selected home, stale head, replacement, and service-restart cases;
- exact eight-key effective-config extraction, path-less Default/OverrideEnv origins, one-snapshot
  environment parsing, descriptor-pinned patch origins, and proof that the byte vector passed to YAML
  parsing is the same vector whose SHA-256 enters every patch origin; concurrent truncate/replace/
  rename/symlink and A-to-B-to-A races before open, during read, after read, and before publication
  must yield one self-consistent consumed-byte snapshot or a typed failure, never values parsed from
  one byte sequence with another sequence's digest; present/absent accepted-home facade postures,
  root-identity replacement while the original descriptor remains held, attempts to clone/serialize/
  detach/reopen the sealed source, and drop-time descriptor closure are explicit walls; plus
  `llm.enabled` false and every activation-predicate mismatch;
- deterministic Codex 0.125 reconstruction, byte-identical TOML on retry/restart, exact effective
  model/provider/MCP values, deterministic confined output-last-message paths, descriptor-only
  wrapper/Codex exec, source-matched loader differential fixtures, and rejection of
  system/managed/project/profile/thread/CLI/env ambient overrides; project-layer vectors must match
  only the project-root-to-cwd `.codex/config.toml` chain and must prove that neither a standalone
  `<cwd>/config.toml` nor a separately injected git-root layer enters the loader input list; exact
  ephemeral credential-store
  rendering and source fixtures proving no `auth.json` load and no cloud-cache read/write/network
  branch, plus initial/resumed/pre-release/post-turn absence walls for `.credentials.json`,
  `auth.json`, and `cloud-requirements-cache.json`, including prior-turn creation, symlink, hardlink,
  special-file, mount, and observation-error failures without content read or deletion;
- V1 byte-identical compatibility while exclusion is idle, typed pre-spawn
  `UnsupportedSecurityPosture` while it is E3-exclusive, V2 success, old-server/new-client and
  new-server/old-client behavior, missing legacy state, and unknown/newer V3 rejection before D1
  exists. Native Lima conversion proof is deferred until Linux completion under the E3-A recovery
  authority above and is not part of this Linux acceptance wall;
- exact E2 launch and fork cap linkage without any E2/E2-RM mutation or current-parent
  reconstruction, including `Some(exact proof)` for `FreshSpawn`, `None` for `Fork`, and byte-equal
  nullable proof values across prepare/idempotency/sealed preparation/V2; and
- V2 emission and equality fixtures through both
  `continue_world_worker_fork_command_bootstrap_after_delivery` and
  `build_continue_world_worker_fork_command_transport_request`, plus every direct Spawn/Fork path;
- at least two sequential resumed E2 turns proving the retained active projection/lease/gateway and
  descriptor-pinned adapter are revalidated, each child receives its own current-turn E2 enforcement
  input/security attestation/gate, ambiguous release never re-executes, `unregister_turn` preserves
  the capability, and terminal unregister/restart revokes then releases it; plus deterministic
  submit-vs-unregister, submit-vs-restart, prepare-vs-revoke, and final-release-vs-revoke barriers
  proving the one control mutex's winner, no child side effect after `Revoking`, and no final gate
  write unless lifecycle is still `Active`; and
- an exact-baseline differential wall against
  `2b2fc6c50b40046dbeaeb5b316562fbd96480a2a` showing no unallowed candidate-only failure across
  the packet-authorized crates and all existing applicable Linux world-service/gateway/member-
  dispatch tests.

`E2-RM` is already landed at `2012bb8b5562a73ed0ee45238c19252c16b25065`; that fact satisfied the
prerequisite for the E3-AC1 documentation authority and is not an E3 runtime dependency. The
preserved E3-A candidate remains incomplete and unlanded, and no E3 gate becomes green here. No D1,
D2, D3, E4, compatibility-promotion, or non-Linux work is admitted. B4 remains separate and is
neither modified nor adjudicated by this correction. E3-A Linux proof recovery requires a next
explicit dispatch after this correction is pushed and its live origin identity is verified; E3-B
through E3-F remain separately admission- and dispatch-gated by their direct predecessor.
