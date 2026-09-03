**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted existing-carrier-versus-remaining-adoption boundary, `LaunchTimeSecretHandoffV1` schema, delivery and state enums, allowed transitions, terminal/reuse rules, and handoff rules 1–11 covering non-secret refs, fail-closed exclusions, compatibility copied-credential status, and retry semantics
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#9-launchtimesecrethandoffv1`](../04-contracts-and-gates.md#9-launchtimesecrethandoffv1), baseline lines 176–258; the exact 4968-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `0a5a13799969f165c3acca200dfd5f28bf2e74ccbfeb71356176caecec78b0cd`

<!-- exact-extracted-body:start -->
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

<!-- exact-extracted-body:end -->

## E3 adoption clarification (2026-09-02; outside preserved body)

The preserved V1 body intentionally permits a producer-defined non-secret logical `fd_name`. E3 is
narrower: its `delivery` must be exactly
`SecureFd { fd_name: "SUBSTRATE_LLM_AUTH_BUNDLE_FD", one_time: true,
gateway_receiver_only: true, deny_child_inheritance: true, close_after_consume: true }`, matching the
landed public constant and gateway consumer. E3 exact-validates this value in every Prepared,
Delivered, and Consumed revision and against `NonsecretHandoffProjectionV1`; an arbitrary label or
false Boolean cannot satisfy E3 even if it remains representable for another generic V1 producer.

For E3 only, the two historically named but previously shape-free references above have these exact
strict V1 wire definitions in the new shared `config-projection` crate:

```rust
struct CredentialSourceRefV1 {
    schema_version: u32, // exactly 1
    credential_source_id: String,
    preparation_id: String,
    selected_backend_id: String, // exactly "cli:codex-world"
    bundle_backend_id: String, // exactly "cli:codex"
    ordered_field_names: Vec<String>,
    optional_account_id_present: bool,
    issued_at: Timestamp,
    expires_at: Timestamp,
    ref_hash: String,
}

struct RedactedDiagnosticRefV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    diagnostic_id: String,
    diagnostic_hash: String,
}

struct RedactedDiagnosticRecordV1 {
    schema_version: u32, // exactly 1
    authority_store_id: String,
    diagnostic_id: String,
    code: RedactedDiagnosticCodeV1,
    phase: RedactedDiagnosticPhaseV1,
    occurred_at: Timestamp,
    diagnostic_hash: String,
}

enum RedactedDiagnosticCodeV1 {
    Missing,
    Malformed,
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

enum RedactedDiagnosticPhaseV1 {
    AuthorityResolution,
    ProjectionPublication,
    GatewayPreparation,
    SecretDelivery,
    Readiness,
    MemberActivation,
    Retirement,
}
```

`credential_source_id` is `crs_<lowercase UUIDv7>` allocated when world-service accepts one exact
in-memory `GatewayIntegratedAuthPayloadV1`; it names a sealed, process-local, non-cloneable
credential-source capability and never a filesystem or credential-store locator. Field names are
bytewise sorted and duplicate-free and must be exactly
`["SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN"]` or that value followed by
`"SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID"` in bytewise order as applicable; the optional
account ID value is never persisted. `ref_hash` is the canonical JSON hash of
`{"credential_source":<ref with ref_hash omitted>,"domain":"substrate.e3.credential-source-ref.v1"}`.
For the E3 preparation route, `issued_at` equals the prepare response's `prepared_at` and both this
ref and `LaunchTimeSecretHandoffV1` use the response's exact `expires_at`, corresponding to the
fixed 120-second `CLOCK_BOOTTIME` deadline; wall-clock timestamps never extend that deadline.
Resolution exact-matches every nonsecret field, preparation, issuance/expiry, and the capability's selected and
bundle backend identities, then moves the live secret capability into the one-time handoff. It never
hashes, logs, serializes, or equality-compares a reusable derivative of secret bytes. After service
restart, expiry, or consumption the durable ref is unresolvable, so activation fails closed and a
fresh credential source, handoff, gateway identity, fence/root, and consumer lease are required.
Those form a new monotonic revision in the same projection series when the immutable subject is
unchanged; only an immutable subject change permits a fresh series.

`diagnostic_id` is `rdg_<lowercase UUIDv7>`. An E3 diagnostic ref resolves only beneath the configured
accepted-home authority at `diagnostics/<diagnostic-id>.json`, whose immutable, owner-only,
first-writer canonical record exact-matches store/ID/hash. `diagnostic_hash` is the canonical JSON
hash of `{"diagnostic":<record with diagnostic_hash omitted>,"domain":"substrate.e3.redacted-diagnostic.v1"}`.
The record has no message, path, argv, environment, identity header, account value, provider body, or
free-form field. Unknown/newer fields or enums are rejected, and a missing record cannot substitute
for a typed failure. Its publication follows E3's same root lock, recognized temporary, file/directory
fsync, exact-readback, and immutable-retention rules.

E3 does not replace or reopen the landed FD mechanics above. The controlling E3 adoption contract is
[`managed-gateway-adoption-v1.md`](managed-gateway-adoption-v1.md): it preallocates the exact gateway
identity, persists only non-secret handoff/intent/ACK references, preserves one fresh FD for one exact
gateway `exec`, and closes every secret-bearing buffer and descriptor after the attempt. It also
persists the exact nonsecret gateway launch-input object/ref. The secret-FD pointer and E3's two
nonsecret launch/listener FD pointers plus its nonsecret secret-ready-attestation FD pointer exist
only in the gateway launch environment and are removed by the gateway. World-service has already
set/read back zero core limits and non-dumpability before accepting the E3 secret body; the gateway
sets, reads back, and attests the same posture after exec and before the secret write. Codex, the E3
local adapter, UAA compatibility wrappers, MCP/tool children, and provider children inherit neither
pipe end, any pointer variable, nor raw credentials.

The E3 ACK is equality-only evidence that the exact gateway consumed the exact handoff, answered a
fresh non-secret readiness challenge, and remains behind the exact dormant access boundary. It is not
a digest of secret material, a reusable credential, provider-success evidence, or authorization.
Gateway request identity headers are non-secret metadata; the Linux cgroup/nftables boundary, not a
header, authorizes the exact member process tree. Copying host auth/config remains separately granted,
named, logged, non-promotable compatibility and cannot satisfy a missing or invalid E3 record.

This clarification does not change the byte-preserved V1 body, admit E3, dispatch implementation,
alter gateway provider authority, or make E2-RM/B2.2 an E3 prerequisite.
