**Kind:** slice/task
**Stable ID:** `A1.1d-5R3-family`
**Canonical for:** R3 authoritative implementation index
**Status:** canonical
**Authority scope:** exact extracted 03 implementation-index source body only
**Source span:** [`03-phase-slice-map.md#a11d-5r3-authoritative-implementation-index`](../03-phase-slice-map.md#a11d-5r3-authoritative-implementation-index) lines 2228–3096
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R3 authoritative implementation index

## A1.1d-5R3 authoritative implementation index

This entire R3 implementation index is archived preserved planning evidence, not the current next
dispatch. It is superseded for active scheduling by
[`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md). Planning was complete at
`19c40d41679e843e3e524f64fb9827959849d33e` / `d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with
planning fingerprint `sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`.
R3 implementation is `PARKED_BY_USER`, no R3 implementation task has been dispatched, and the
previous authority wall `AUTHORITY_REQUIRED:B1_B2_1_JOINT_CLOSEOUT` is now closed. B3.1, C1, and
the bounded internal A1.2b packet are complete on the bound Tuesday, August 4, 2026 candidate,
and the then-current historical gate was `AUTHORITY_REQUIRED:R3_RESUME`. The former edge below is
archived and does not govern current scheduling. At that historical checkpoint, only after fresh
user/meta authority revalidated live repository truth could it reopen the preserved future edge
`AUTHORITY_REQUIRED:R3_IMPLEMENTATION -> A1.1d-5R3-HOME`; R3 must then complete before A1.3, A1.4,
or A1 closeout. Every later edge remains an explicit authority gate.
No task is pre-created.

```text
AUTHORITY_REQUIRED:R3_IMPLEMENTATION
  -> HOME -> MANIFEST
  -> LINUX -> EVIDENCE:R3-LINUX-IMP-01 -> LINUX-CLOSEOUT
  -> MAC -> EVIDENCE:R3-MAC-IMP-01 -> MAC-CLOSEOUT
  -> WIN -> EVIDENCE:R3-WIN-IMP-01 -> WIN-CLOSEOUT
  -> UNIX
  -> EVIDENCE:R3-NATIVE-LINUX-01
  -> EVIDENCE:R3-NATIVE-MAC-01
  -> EVIDENCE:R3-NATIVE-WIN-01
  -> CLOSEOUT
```

The three final native evidence nodes all bind the same published `UNIX` commit/tree/ref; the
linear drawing is dispatch order, not a change of checkpoint. Evidence nodes are read-only
platform tasks and publish no repository bytes. Each validates a native
`codex.top-level-evidence-receipt.v1` with the orchestration skill validator; that validator binds
the receipt and artifact digest but has no successor field. The separate native evidence artifact
is independently validated with the exact expected gated successor under `04`. The user/meta
orchestrator must separately authorize each edge.

Every future evidence dispatch is correlated to product project
`2ccb802f-301c-4af4-9bd5-51d22808f0a2` and discovers the repository by exact origin plus target
ref. Its dispatch binds a fresh nonce, source implementation task thread/host, evidence task
thread/host, return meta thread/host, evidence ID, source commit/tree/ref, artifact paths, and
gated successor. Those exact correlation values appear in artifact and receipt; the evidence task
sends the validated native JSON to the bound return meta task with `send_message_to_thread` as its
final tool action. Planning-task IDs or a different project/host are never inferred or reused.

Each packet starts from the exact landed predecessor, permits only the commit count named in its
contract and one normal fast-forward push of that linear range to the bound target ref after every
gate, and ends with a validated `codex.top-level-task-receipt.v1`. Merge, rebase, cherry-pick,
reset, amend/rewrite, force push, staged publication, or an orchestration-branch push is forbidden.
A packet maps every terminal condition through the closed status table in `04`; no invented status
is permitted.

Every implementation packet must run fresh GitNexus query/context exploration, impact every
existing function/method before edit, manually close script/cfg/generated/platform callers, and
run `gitnexus_detect_changes()` before commit. Any HIGH/CRITICAL result, new caller outside the
fence, or graph/manual disagreement requires explicit risk review; a new authority domain,
selector, execution family, or destructive primitive is `BLOCKED_SCOPE_EXPANSION`.

In the packet contracts below, canonical documentation set `R3-DOCS` means exactly
`llm-last-mile/runtime-refactor/00-README.md`,
`02-seam-crosswalk.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, and
`05-debug-regression-ledger.md`, with the same directory prefix on the last four paths.
`01-target-architecture.md` is frozen through implementation and evidence packets. Only
`A1.1d-5R3-CLOSEOUT` may append its bounded terminal R3 status in that file; changing any frozen
architecture or earlier text is `BLOCKED_SCOPE_EXPANSION`. Each named review set means exactly one
`review-control/r3-<slug>-review-cycle-record.json` plus
`review-control/r3-<slug>-review-authority-security.md`,
`review-control/r3-<slug>-review-lifecycle-convergence.md`, and
`review-control/r3-<slug>-review-allowlist-evidence.md`; angle-bracket substitution is descriptive
notation here, and each packet's literal slug is its lower-case suffix.
`R3-DOCS` is a sequential status/receipt surface: each packet may append only its own landed
result and may not rewrite another packet's row, proof, or architecture.
Every provider/final evidence task and closeout validates the native artifact with the
MANIFEST-owned `scripts/ci/validate_r3_native_evidence.py` and exact expected evidence ID,
source commit/tree/ref, and artifact `gated_successor`; it then validates the separate
`codex.top-level-evidence-receipt.v1` with the skill validator's single positional receipt
argument and exact-joins receipt field `evidence.artifact_sha256` to the validated artifact digest.
The skill validator is never passed or
credited with a successor check.

### `A1.1d-5R3-HOME`

**Completion claim.** Implement only synchronous exact current-attempt private-home candidate
rollback. Unknown-provenance crash residue remains unchanged/fail-closed. Predecessor:
`A1.1d-5R3-PLAN`; successor:
`AUTHORITY_REQUIRED:A1.1d-5R3-MANIFEST`.

- Ownership: `A1D5I-HOME-04`, `A1D5I-HOME-05`, the implementation clause of `RG-HOME-01`.
- Production allowlist:
  `crates/shell/src/execution/agent_runtime/host_session_authority/trusted_fs.rs` symbols
  `ensure_private_substrate_home`, `ensure_private_substrate_home_with`,
  `PrivateHomeCandidateProvenance`, `PrivateHomeError`, and new
  `PrivateHomeCandidateRollback`/`rollback_created_private_home_candidate`; and
  `crates/shell/src/execution/home_bootstrap.rs` symbols
  `ensure_substrate_home_deps_scaffold_at`, `HomeBootstrapError`, and
  `map_trusted_scaffold_error`.
- Test allowlist: embedded tests in those two files only. Required negatives cover
  `AlreadyExists`, pre-existing, replaced, nonempty, wrong type/owner/mode/ACL, unsupported or
  ambiguous lookup, symlink, descriptor/name/identity mismatch, and `Unknown`; kill points bracket
  create, first open, validation, cleanup, and acceptance.
- Documentation/control surface: exact `R3-DOCS` plus the exact `home` review set.
- Checks: focused private-home/home-bootstrap tests, `cargo test -p shell --lib`, formatting,
  clippy for `shell`, `git diff --check`, allowlist, secret review, change detection, and a
  non-privileged secure-fixture bootstrap smoke. GitNexus currently reports the wrapper HIGH
  (16 graph-visible direct callers; manual production chain reaches `run_shell_with_cli`) and the
  inner helper MEDIUM; the packet must preserve that caller set or stop.
- Platform/privilege: Unix implementation; Linux ACL/identity behavioral proof, no sudo. One
  commit/push. `BLOCKED_SCOPE_EXPANSION` if durable provenance is required;
  `BLOCKED_CONTRADICTION` for any attempted recursive/path-only cleanup.
- Non-goals/freeze: all installers, manifests, shims, services, Lima, Windows, passive health, and
  pre-existing-invalid-home remediation.

### `A1.1d-5R3-MANIFEST`

**Completion claim.** Land a non-destructive, canonical managed-artifact manifest/parser/
publication/state-transition core and hidden authenticated CLI surface. It validates and records
authority but performs no artifact deletion, replacement, stop, kill, unregister, or restoration.
Predecessor: `HOME`; successor: `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX`.

- Ownership: prerequisite for `RG-INSTALL-01`, `R3-LIFE-01`, and `R3-WIN-01`; no PI row and no
  destructive finding closure. It cites as consumers all 22 current R3 rows and findings
  INSTALL-02/03/06/07/08/10 while their sole owners remain exactly those in `02`.
- Production allowlist:
  new `crates/common/src/managed_artifact.rs` with
  `ManagedArtifactManifestV1`, `ManagedArtifactEntryV1`, `ManagedArtifactIdentityV1`,
  `ManagedArtifactRoleV1`, `ManagedArtifactDispositionV1`, `ManagedLifecycleStateV1`,
  `ManagedActionPreparedRecordV1`, `ManagedActionReceiptV1`, `ManagedActionReceiptIndexV1`,
  `ManagedActionReceiptIndexEntryV1`, `ManagedManifestHeadV1`, `ManagedSharedClaimsV1`,
  `ManagedSharedClaimV1`, `ManagedExecutorIdentityV1`,
  `LifecycleSignatureV1`, `LifecyclePublisherAnchorV1`,
  `LifecyclePublisherProtectedStateV1`, `ManagedLifecyclePublisherRequestV1`,
  `PublisherBootstrapAuthorizationV1`, `PublisherBootstrapComponentV1`,
  `PublisherBootstrapComponentRoleV1`,
  `PublisherTestRetirementCommitmentV1`,
  `PublisherTestRetirementAuthorizationV1`, `PublisherTestRetirementReceiptV1`,
  `PublisherTestRetirementAcknowledgementV1`,
  `GuestPublisherRetirementReservationV1`,
  `GuestPublisherReservationUnusedProofV1`,
  `GuestPublisherReservationUnusedAcknowledgementV1`,
  `GuestPublisherTestRetirementCommitmentV1`,
  `GuestPublisherTestRetirementAuthorizationV1`,
  `GuestPublisherTestRetirementReceiptV1`,
  `GuestPublisherTestRetirementAcknowledgementV1`, `PublisherTransportFrameV1`,
  `GuestPublisherPairingChallengeV1`, `GuestPublisherPairingTicketV1`,
  `GuestPublisherPairingHostRecordV1`, `GuestPublisherPairingGuestIntentV1`,
  `GuestPublisherBootstrapHelloV1`,
  `GuestPublisherBootstrapTranscriptV1`, `ExecutorBuildEvidenceV1`,
  `LimaStageOneAuthorizationV1`, `ManagedActionV1`,
  `CanonicalManifestBytesV1`, `parse_and_validate_manifest_v1`,
  `canonical_manifest_bytes_v1`, `canonical_publisher_bootstrap_authorization_v1`,
  `canonical_publisher_bootstrap_core_v1`,
  `canonical_managed_action_prepared_record_v1`,
  `canonical_lifecycle_publisher_protected_state_v1`,
  `validate_lifecycle_publisher_protected_state_v1`,
  `canonical_action_receipt_bytes_v1`, `canonical_managed_action_receipt_signature_payload_v1`,
  `validate_managed_action_receipt_signature_v1`, `action_receipt_artifact_sha256_v1`,
  `canonical_action_receipt_index_bytes_v1`,
  `compare_and_swap_action_receipt_index_v1`,
  `commit_action_receipt_index_to_head_v1`,
  `retirement_receipt_artifact_sha256_v1`,
  `validate_publisher_test_retirement_acknowledgement_v1`,
  `canonical_guest_publisher_test_retirement_ticket_core_v1`,
  `canonical_guest_publisher_reservation_unused_proof_v1`,
  `validate_guest_publisher_reservation_unused_proof_v1`,
  `guest_publisher_reservation_unused_proof_artifact_sha256_v1`,
  `canonical_guest_publisher_reservation_unused_acknowledgement_v1`,
  `validate_guest_publisher_reservation_unused_acknowledgement_v1`,
  `guest_publisher_reservation_unused_acknowledgement_artifact_sha256_v1`,
  `validate_guest_publisher_test_retirement_authorization_v1`,
  `validate_guest_publisher_test_retirement_acknowledgement_v1`,
  `canonical_guest_publisher_pairing_ticket_v1`,
  `validate_guest_publisher_pairing_ticket_v1`,
  `canonical_lifecycle_signature_payload_v1`, `verify_lifecycle_signature_v1`,
  `parse_p256_spki_der_v1`, `verify_p256_p1363_low_s_v1`,
  `verify_ed25519_fixed_v1`,
  `parse_publisher_bootstrap_authorization_v1`, and
  `validate_publisher_bootstrap_authorization_v1`,
  `derive_publisher_bootstrap_component_target_v1`,
  `validate_complete_publisher_bootstrap_component_set_v1`,
  `canonical_lima_stage_one_authorization_v1`, and
  `validate_lima_stage_one_authorization_v1`; `crates/common/src/lib.rs` module/export only;
  new
  `crates/shell/src/execution/managed_lifecycle.rs` with
  `ManagedLifecycleControlRequestV1`, `publish_manifest_v1`, `load_manifest_v1`,
  `derive_lifecycle_capsule_locator_v1`, `open_trusted_lifecycle_capsule_v1`,
  `compare_and_swap_head_v1`, `transition_manifest_v1`, `update_shared_claims_v1`,
  `publish_action_receipt_index_v1`, `resume_action_receipt_commit_v1`,
  `issue_publisher_bootstrap_authorization_v1`, `open_publisher_bootstrap_channel_v1`, and
  `validate_publisher_response_v1`, plus trait `LifecyclePublisherClientV1` with closed
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, and
  `issue_guest_publisher_pairing_ticket_v1` methods; new platform-client
  stub files `crates/shell/src/execution/managed_lifecycle/linux_client.rs`,
  `macos_client.rs`, and `windows_client.rs`, each exposing only
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, and
  `issue_guest_publisher_pairing_ticket_v1`, returning a typed provider-unavailable error until
  its sole platform packet replaces those stub bodies;
  `crates/shell/src/execution/mod.rs` module declaration/re-exports only;
  `crates/shell/src/lib.rs` re-exports only; new non-destructive control binary
  `src/bin/substrate-lifecycle-control.rs` symbols `main`,
  `publisher_bootstrap_direct_interactive_v1`, `read_exact_bootstrap_confirmation_v1`, and
  `guest_publisher_pairing_direct_interactive_v1`, `display_guest_pairing_challenge_v1`, and
  `submit_managed_lifecycle_request_v1`; new evidence validator
  `scripts/ci/validate_r3_native_evidence.py` and focused test
  `scripts/ci/test_validate_r3_native_evidence.py`; `Cargo.toml` only to add direct root-package
  dependencies `substrate-common = { version = "0.2.8", path = "crates/common" }`, `serde` with
  derive, `serde_json = { workspace = true }`, `ed25519-dalek = { workspace = true }`,
  `rand_core = { workspace = true }`, and
  `libc = 0.2`; its `[workspace.dependencies]` may add `base64 = 0.22`,
  `ed25519-dalek = 2.1` with `rand_core` and `pkcs8`,
  `p256 = { version = "=0.13.2", default-features = false, features = ["ecdsa", "pkcs8", "std"] }`,
  and `rand_core = 0.6` with `getrandom`; target-Windows root dependencies may add
  `windows-sys = 0.52` with exactly
  `Win32_Foundation`, `Win32_Security`, `Win32_Security_Cryptography`,
  `Win32_Storage_FileSystem`, `Win32_System_IO`,
  `Win32_System_Pipes`, `Win32_System_Registry`, `Win32_System_Services`, and
  `Win32_System_Threading`; `crates/common/Cargo.toml` only to add workspace `serde_json`, `sha2`,
  `uuid`, `base64`, `ed25519-dalek`, `p256`, and `rand_core`; and `Cargo.lock` only for the resulting
  dependency graph. No other
  manifest key, version, feature, package, checksum, or lock entry is mutable.
- Mutable test allowlist: embedded tests in
  `crates/common/src/managed_artifact.rs` and
  `crates/shell/src/execution/managed_lifecycle.rs`, plus new
  `crates/shell/tests/managed_lifecycle_v1.rs` and the exact evidence-validator test above.
  Required cases cover canonical/golden encoding,
  unknown/duplicate/missing/wrong-type fields, invalid Unicode/NUL, path normalization, owner/
  version/context/platform/principal/object/type/disposition/metadata joins, missing/malformed/
  tampered/stale/cross-principal/cross-instance/partial manifests, and kill points before/after
  temp write, temp sync, publication rename, parent sync, head CAS, shared-claim CAS, transition,
  and receipt. Add coherent forgery, valid-old replay, parent replacement, locator substitution,
  unprivileged publisher, role scope escape, publisher bootstrap field/confirmation/peer/framing/
  replay negatives, evidence artifact source/successor/correlation joins,
  bootstrap-core/retirement-precommit joins with no digest cycle,
  publisher-signed and externally hashed action receipts plus externally hashed retirement
  receipts and exact acknowledgement joins with no self or future digest field, manifest-
  preallocated per-generation receipt names, unsigned/wrong-key/wrong-algorithm/wrong-counter/
  wrong-prepared-record/action-observation and same-principal receipt substitution negatives,
  full `LifecyclePublisherProtectedStateV1` bytes at each exact platform locator, prepared-slot/
  counter/revision/final-anchor CAS and terminal retention, with detached-digest, alternate-locator,
  torn/rollback/replay, and clear-before-final-anchor negatives; extra-file and wrong-generation
  preservation, kill-before/after receipt signature and durable
  receipt publication, action-receipt signature/index/head/anchor CAS crash joins, and rejection
  of a final-anchor digest embedded back into its index, Ed25519 and
  P-256 SPKI/P1363/
  low-S/domain-separation golden vectors, host-publisher pairing ticket canonical/signature/scope/
  expiry/consumption vectors, guest-TTY pinning, and ticket/transcript replay negatives,
  including malformed/substituted SPKI, wrong curve/algorithm/encoding, SPKI/fingerprint and
  SPKI/anchor-key mismatch, DER/high-S/malleable P-256 signatures, and fixed Ed25519 harness-
  authorization/acknowledgement key/signature encodings; exhaustive host/guest test-retirement
  DAGs, null-slot guest ticket-core construction, commitment copying through ticket/host record/
  intent/hello/transcript/generation one, guest receipt/acknowledgement before reverse-order guest-
  then-host removal, kill points at every removal/parity join, and missing/cross-ticket/late
  commitment, early seed-intent/pairing-record/protected-state removal, omitted-role, and non-parity
  negatives; harness-only pre-host-bootstrap reservation of exactly zero Linux or one MAC/WIN
  challenge/pairing-record/guest-component tuple, its reserved/issued/consumed/unused/retired
  transitions, exact `Reserved` and `TicketIssued` unused proofs against each target's precommitted
  before-state, signed external acknowledgement before pairing-record restoration/removal, and
  expiry/EOF/cancellation/kill boundaries; unreserved/extra/reused/reordered/omitted/cross-
  bootstrap/second-ticket/nonterminal-host-teardown, liveness-only unused proof, missing
  acknowledgement, pre-existing-as-absent, guest-effect-before-consumption, wrong proof or
  acknowledgement key/algorithm/hash/file/parent/fsync/reservation/ticket/pairing-record/dispatch/
  scope/nonce, replay, cross-scope, unknown-field, resulting/future CAS-or-anchor field, and self/
  future-digest negatives; golden construction order is prior state -> proof fsync/hash ->
  acknowledgement fsync/hash -> `ProvenUnused` CAS -> `Retired` CAS;
  exhaustive `PublisherBootstrapComponentRoleV1` platform/
  guest variants, derived targets/types/metadata/dependencies/durability, complete-set golden
  vectors, authorization-level greenfield publisher absence, per-role `ExactAbsentCreate` versus
  `ExactPreExistingDependency` golden vectors, complete sets containing both dispositions,
  created-only retirement membership, and forbidden pre-existing publisher identity, unsupported
  pre-existing dependency, disposition/observation mismatch, omitted/unlisted-parent/container/
  role-target preserving negatives,
  creator-first/two-install last-claim, and legacy
  adoption rejection. Every role variant, parameter type, domain, target, object type, dependency,
  and action in the exhaustive `R3-MANIFEST-01` table has a positive golden vector; unknown roles,
  unlisted actions, caller paths, and every role/target mismatch are preserving negatives.
- Documentation/control surface: exact `R3-DOCS` plus the exact `manifest` review set.
- Checks: common and shell focused tests, `cargo test -p substrate-common`, the new shell
  integration test, shell library regression, format/clippy, schema golden vectors, diff/
  allowlist/secret/change detection. Impact `Cli`, `run_shell_with_cli`, and every edited existing
  symbol; the ordinary `Cli` and `run_shell_with_cli` are frozen, and any new public selector or change to
  `InstallBootstrapContextCarrierV1::validate` is a hard stop.
- Platform/privilege: portable, no privilege or platform mutation. One commit/push. Parser or
  publication uncertainty is terminal-preserving; durability unavailable on a supported
  filesystem uses `BLOCKED_SCOPE_EXPANSION` when new durability authority is required or
  `BLOCKED_CONTRADICTION` when the supported filesystem violates the frozen contract.
- Non-goals/freeze: no consumer integration, recursive subtree, deletion, adoption of observed
  state, IH/PM redesign, or new top-level shared root. Dependency edits are limited to the exact
  root-package keys and lock closure above; every other dependency change is a stop. Pre-R3 collisions
  preserve state and terminate `BLOCKED_SCOPE_EXPANSION`.

### `A1.1d-5R3-LINUX`

**Completion claim.** Implement the Linux managed-system provider and exact pre-state restoration
without editing Unix row-owner orchestrators. Predecessor: `MANIFEST`; successor:
`EVIDENCE:R3-LINUX-IMP-01`.

- Ownership: PI-031, PI-103, `A1D5I-INSTALL-06`, `A1D5I-ENV-01`,
  `R3-ACT-LNX-012`, `R3-ACT-LNX-026`, `R3-ACT-LNX-095`, `R3-ACT-080-LNX`, and
  `R3-ACT-089-LNX`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-linux.rs` symbols `main`,
  `LinuxManagedArtifactExecutorV1`, `open_linux_lifecycle_capsule_v1`,
  `join_linux_role_identity_v1`, `execute_linux_managed_action_v1`,
  `restore_linux_managed_role_v1`, `publish_linux_action_receipt_v1`,
  `open_linux_publisher_protected_state_v1`, and
  `compare_and_swap_linux_publisher_protected_state_v1`; new
  executor symbols `bootstrap_linux_publisher_v1`, `resume_linux_publisher_bootstrap_v1`,
  `run_linux_publisher_v1`, `accept_linux_publisher_connection_v1`,
  `attest_linux_publisher_peer_v1`, `relay_linux_publisher_request_v1`,
  `handle_linux_publisher_request_v1`, `begin_guest_publisher_bootstrap_v1`,
  `open_guest_controlling_tty_v1`, `read_guest_publisher_pairing_confirmation_v1`,
  `verify_guest_publisher_pairing_ticket_v1`, `emit_guest_publisher_bootstrap_hello_v1`,
  `publish_guest_pairing_intent_v1`, `open_guest_pairing_intent_parent_v1`,
  `publish_guest_pairing_intent_otmpfile_v1`, `open_or_join_guest_signing_key_v1`,
  `resume_guest_publisher_pairing_v1`, `verify_guest_publisher_bootstrap_transcript_v1`,
  `join_guest_host_consumption_v1`,
  `commit_guest_publisher_bootstrap_v1`, and
  `retire_linux_test_publisher_v1`; its unsafe/FFI fence is exactly the small wrappers
  `linux_seqpacket_socket_v1`, `linux_peer_credentials_v1`, `linux_openat2_nofollow_v1`,
  `linux_otmpfile_linkat_v1`, and `linux_fsync_parent_v1`, while Ed25519 use is confined to
  `load_or_create_linux_signing_key_v1` and `sign_linux_anchor_v1`; new fixed units
  `scripts/linux/substrate-lifecycle-publisher-v1.service` and
  `scripts/linux/substrate-lifecycle-publisher-v1.socket`; new
  `crates/shell/src/execution/managed_lifecycle/linux_client.rs` replacement symbols
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, `open_linux_seqpacket_channel_v1`, and
  `attest_linux_publisher_response_v1`; `submit_publisher_request_v1` must use the fixed sudo relay
  and never open the root-only publisher socket directly. No other control/core symbol may change;
  new
  `scripts/linux/world-lifecycle.sh` functions `record_linux_managed_state`,
  `install_linux_managed_state`, `restore_linux_managed_state`,
  `invoke_linux_lifecycle_executor`, and its `main` dispatcher; and
  `scripts/linux/world-provision.sh` existing functions
  `prepare_gateway_smoke_auth`, `cleanup_gateway_smoke_auth`,
  `run_gateway_lifecycle_proof`, `ensure_substrate_group_exists`, `ensure_user_in_group`,
  `install_unit`, `verify_socket_acl_bridge`, and `verify_world_deps_acl_bridge`; the exact build
  argv at the current `cargo build -p substrate --bin substrate` sites, extended only with
  `--bin substrate-lifecycle-linux`; `LIFECYCLE_EXECUTOR_BIN_PATH` derivation and executable
  check; plus the exact
  top-level creation/legacy-cleanup range from the first
  `ensure_substrate_group_exists` call through
  `sudo_cmd systemctl start substrate-world-service.service`. That
  range may only be replaced by one call to `world-lifecycle.sh`; the exact
  `SOCKET_UNIT_CONTENT` heredoc removes only `PartOf=substrate-world-service.service` and admits
  no replacement propagation relationship; no installer file beneath
  `scripts/substrate` may change.
- Mutable test/fixture allowlist:
  `tests/installers/world_provision_context_r2_2.sh`,
  `tests/installers/world_provision_smoke.sh`,
  `tests/installers/linux_lifecycle_r3.sh`. Negative cases cover created and
  pre-existing-preserved state plus preserving rejection of schema-reserved `adopted`,
  units, drop-ins, helpers, gateway, sockets, dirs, group/memberships, ACLs, linger, exact
  synthetic-auth A/B mismatch, partial install, retry, repeat restore, and unrelated siblings.
  They prove one protected world and disposable-publisher socket-state/coupled-endpoint prepared
  transaction and receipt,
  endpoint non-requestability, missing/replaced/ambiguous endpoint zero-change, kill/retry at each
  activation/endpoint-observation/receipt boundary, exact absence of service-to-socket propagation,
  and service stop/restart/restore with socket state and endpoint identity unchanged.
  Guest-bootstrap cases use a test host key and TTY harness to prove full fingerprint/challenge/
  literal confirmation, ticket signature/scope/expiry/one-use state, absent/non-TTY rejection,
  malformed/substituted SPKI, wrong curve/algorithm/encoding, SPKI/fingerprint and anchor-key
  mismatch, DER/high-S signatures, request-key/child/channel/nonce/transcript substitution, replay,
  expiry-before-intent rejection versus exact completed-state post-intent resume, root-only
  external unnamed-file/link/fsync intent publication, seed-confined intent/signed-hello/
  transcript/final-key joins, publisher-directory and final-key create/identity-CAS gaps with the
  required terminal-preserving result, and every pairing kill point;
  no automated or stdin confirmation path is accepted.
  Run-only regression paths are `tests/installers/install_state_smoke.sh` and
  `tests/installers/prefix_propagation_r2_2.sh`; their bytes must not change.
- Documentation/control surface: exact `R3-DOCS` and the exact `linux` review set. Evidence bytes
  are not created or committed in this implementation packet.
- Checks: shell syntax/static checks, every listed fixture, non-privileged model/fixture baseline-
  restoration assertions, diff/allowlist/secret/change detection. The fixture directly builds
  `substrate-lifecycle-linux`; world-provision's
  amended build argv produces it, and the first exact-absence bootstrap installs its retained
  root-owned copy before any other role. Native privileged lifecycle, sudo invalidation,
  restoration parity, and the product behavior wall for service/socket/world/gateway plus existing
  filesystem/network/policy/config/dependency/runtime/shim/replay/trace/diagnostic and PTY/
  non-PTY behavior are exclusively `EVIDENCE:R3-LINUX-IMP-01`.
- Platform/privilege: Linux implementation and fixture validation; no native lifecycle action in
  this packet. Exactly one product/test/review commit and one normal fast-forward push. The
  receipt successor is exactly `EVIDENCE:R3-LINUX-IMP-01`; the implementation may not claim
  native evidence or authorize MAC.
- Non-goals/freeze: PI-012/026/095 orchestrator bodies, prefix payload/shims/profiles, macOS,
  Windows, passive health, new account policy, and broad process kill.

### `EVIDENCE:R3-LINUX-IMP-01` and `A1.1d-5R3-LINUX-CLOSEOUT`

The evidence task is read-only with respect to the repository. It discovers the product project
by origin plus exact target ref, requires the live remote, checkout HEAD, commit, and tree all
equal the published `LINUX` receipt, and runs only the Linux IMP actions in `05` on a dedicated
supported sudo host. It emits and validates `codex.top-level-evidence-receipt.v1`; source
`live_remote` must equal source `commit`. The separately validated evidence artifact's exact
`gated_successor` is
`AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT`. Platform absence returns
`BLOCKED_PLATFORM_HANDOFF_REQUIRED`; an available failed/restoration-inexact run returns
`BLOCKED_NATIVE_EVIDENCE`.

`LINUX-CLOSEOUT` starts only from the same published Linux commit after the clean evidence receipt.
Production/test allowlists are empty. It may add exactly
`review-control/r3-linux-imp-01-evidence.json`,
`review-control/r3-linux-imp-01-receipt.json`, the exact `linux-closeout` review set, and append its
status to `R3-DOCS`; it validates the evidence receipt again with the orchestration skill. One
documentation/evidence commit and one fast-forward push are allowed. Its receipt successor is
exactly `AUTHORITY_REQUIRED:A1.1d-5R3-MAC`.

### `A1.1d-5R3-MAC`

**Completion claim.** Implement manifest-bound Lima staging/teardown and activate only the
already-fixed PM-bound SSH-UDS path after exact unlink/timeout/drop/retry ownership. Predecessor:
`LINUX-CLOSEOUT`; successor: `EVIDENCE:R3-MAC-IMP-01`.

- Ownership: PI-051, PI-098, PI-099, PI-100, PI-101, PI-113, PI-114,
  `A1D5I-INSTALL-08`, `R3-ACT-MAC-012`, and `R3-ACT-MAC-026`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-macos.rs` symbols `main`,
  `MacManagedArtifactExecutorV1`, `open_mac_lifecycle_capsule_v1`,
  `join_mac_role_identity_v1`, `execute_mac_managed_action_v1`,
  `restore_mac_managed_role_v1`, `publish_mac_action_receipt_v1`,
  `MacLifecyclePublisherServiceV1`, `open_system_keychain_protected_state_v1`, and
  `compare_and_swap_mac_publisher_protected_state_v1`, `bootstrap_mac_publisher_v1`,
  `export_mac_p256_spki_der_v1`, `normalize_mac_p256_signature_p1363_low_s_v1`,
  `resume_mac_publisher_bootstrap_v1`, `run_mac_xpc_publisher_v1`,
  `accept_mac_xpc_connection_v1`, `attest_mac_xpc_audit_token_v1`,
  `verify_mac_control_designated_requirement_v1`, `handle_mac_publisher_request_v1`, and
  `issue_lima_guest_pairing_ticket_v1`, `consume_lima_guest_pairing_ticket_v1`,
  `open_mac_guest_pairing_record_v1`, `compare_and_swap_mac_guest_pairing_record_v1`,
  `prove_lima_guest_reservation_unused_v1`,
  `commit_lima_guest_reservation_unused_acknowledgement_v1`,
  `retire_mac_test_publisher_v1`, `publish_lima_stage_one_intent_v1`,
  `attach_lima_stage_one_machine_identity_v1`, and `close_lima_stage_one_intent_v1`; its
  unsafe/FFI fence is exactly
  `mac_xpc_listener_ffi_v1`, `mac_audit_token_ffi_v1`, `mac_security_key_ffi_v1`,
  `mac_keychain_anchor_ffi_v1`, and `mac_atomic_file_ffi_v1`, with link declarations only for
  `Security`, `CoreFoundation`, and the system XPC library; new
  `crates/shell/src/execution/managed_lifecycle/macos_client.rs` replacement symbols
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, `open_mac_xpc_channel_v1`, and
  `attest_mac_publisher_response_v1`, `issue_guest_publisher_pairing_ticket_v1`; no other
  control/core symbol may change; new fixed plist
  `scripts/mac/com.substrate.lifecycle.publisher.v1.plist`; new
  `scripts/mac/lima-lifecycle.sh` functions `load_mapped_lifecycle_v1`,
  `invoke_mac_lifecycle_executor`, `install_mapped_lima_state_v1`,
  `restore_mapped_lima_state_v1`, `install_mac_publisher_v1`,
  `bootstrap_lima_guest_publisher_v1`, `retain_lima_guest_bootstrap_channel_v1`,
  `display_lima_guest_pairing_challenge_v1`,
  `join_lima_guest_bootstrap_transcript_v1`,
  `record_lima_guest_pre_state_v1`, `install_exact_guest_artifacts_v1`,
  `persist_lima_guest_unused_proof_v1`, `acknowledge_lima_guest_unused_proof_v1`,
  `restore_lima_guest_state_v1`, `retire_lima_guest_test_publisher_v1`, and `main`; existing `scripts/mac/lima-stop.sh` parameter/intake
  surface may add required prefix, `InstallBootstrapContextCarrierV1`, and
  `PlatformBootstrapMappingV1` arguments plus `resolve_lima_stop_authority_v1` and
  `invoke_mapped_lima_stop_v1`; its complete executable body after `set -euo pipefail`, including
  the first `limactl list substrate` through final `fi`, is replaced by one exact mapped-executor
  request. No hard-coded/default instance remains on the destructive path;
  `scripts/mac/lima-warm.sh` functions
  `destroy_vm`, `ensure_vm_ready`, `ensure_substrate_group`, `stage_workspace`,
  `install_agent_from_host`, `install_cli_from_host`, `install_gateway_from_host`,
  `build_missing_components_inside_vm`, `install_guest_binaries`,
  `bootstrap_guest_private_home`, `write_systemd_units`, `enable_socket_activation`,
  `write_layout_sentinel`, and `configure_guest`; the exact `configure_guest` mutation range from
  `ensure_substrate_group` through `write_layout_sentinel` is replaced by one
  `lima-lifecycle.sh` call. `build_missing_components_inside_vm` and its nested `fix_dns`, package,
  Rustup, Cargo-build, `/etc/resolv.conf`, and DNS-service mutation branches are tombstoned; absence
  of an exact `ExecutorBuildEvidenceV1`-joined guest artifact stops before guest mutation. Existing mapping observation,
  check-only/status, socket summary, and linger guidance remain read-only and frozen;
  `scripts/mac/lima/units/substrate-world-service.socket` removes exactly
  `PartOf=substrate-world-service.service` and admits no replacement propagation relationship;
  `crates/world-mac-lima/src/forwarding.rs` symbols
  `ForwardingHandle`, `ForwardingHandle::drop`, `ForwardingKind::SshUds`,
  `create_ssh_uds_forwarding`, and new
  `MappedSshUdsAttemptV1`, `create_mapped_ssh_uds_forwarding_v1`,
  `record_mapped_known_hosts_entry_v1`, and `restore_mapped_known_hosts_entry_v1`; and
  `crates/world-mac-lima/src/lib.rs` symbols `MacLimaBackend::ensure_forwarding`,
  `ensure_session_setup`, `get_agent_endpoint`, and removal of the exact
  `r3_forwarding_activation_required` gate. `new_with_mapping`, mapping validation,
  `auto_select`, VSock, TCP, and ambient selectors are frozen.
- Mutable test/fixture allowlist: embedded tests in
  `crates/world-mac-lima/src/forwarding.rs` and
  `crates/world-mac-lima/src/lib.rs`,
  `tests/mac/installer_parity_fixture.sh`, and new `tests/mac/lifecycle_r3.sh`. Run-only regression
  paths are `tests/mac/prefix_mapping_r2_3.sh` and `tests/mac/lima_doctor_fixture.sh`; their bytes
  must not change. Required negatives
  cover exact socket/replacement/symlink identity, `StreamLocalBindUnlink`, timeout kill/wait,
  Drop teardown, handle loss, retry, partial staging/unit/socket state, pre-existing VM,
  protected host-publisher plus guest world/publisher service-state/coupled-endpoint prepared
  transactions and receipts, endpoint
  non-requestability, missing/replaced/ambiguous endpoint zero-change, kill/retry at each activation/
  endpoint-observation/receipt boundary, exact absence of service-to-socket propagation, and
  service stop/restart/restore with socket state and endpoint identity unchanged,
  absent-instance stage-one authorization/create/finalize-PM kill points and preserving ambiguous
  create/finalize failure,
  group/membership/private-home/layout-sentinel/service-state and A-local known-hosts restoration,
  missing or build-evidence-mismatched artifact with zero mutation, prohibited DNS/toolchain/package remediation,
  host-key fingerprint/challenge confirmation only through independent host and guest controlling
  terminals; signed ticket PM/machine/artifact scope, expiry, one-use consumption, exact retry,
  wrong/automated/non-TTY confirmation, malformed/substituted SPKI, wrong curve/encoding,
  SPKI/fingerprint/anchor mismatch, high-S signature, alternate child/key/channel/transcript,
  durable host-record/guest-intent restart, pairing kill points, exact pre-existing versus created
  Lima guest state-root/lifecycle-container/publisher-directory/executor-parent roles, root create/
  identity-CAS preserving gaps, reserved challenge/component tuple consumption; both `Reserved`
  and `TicketIssued` unused revocation with exact pre-existing/absent target-before-state,
  protected-host P-256 proof, external file/parent fsync/hash, harness-Ed25519 acknowledgement,
  Keychain pairing-record restoration/removal, and kill points; liveness/timeout/pathname proof,
  missing/wrong acknowledgement, cross-reservation/replay/file/parent/fsync/digest negatives;
  reverse created-empty parent retirement after guest acknowledgement, and final guest parity,
  preserving rejection of schema-reserved `adopted`,
  unrelated Lima instance/state, and no alternate transport.
- Documentation/control surface: exact `R3-DOCS` and the exact `mac` review set. Evidence bytes
  are not created or committed in this implementation packet.
- Checks: focused crate tests, mac fixture scripts, non-native lifecycle-model/restoration
  assertions, available target static-check only, diff/allowlist/secret/change detection. IMP
  fixtures use a non-executing mock artifact identity; only `EVIDENCE:R3-MAC-IMP-01` may natively
  build/code-sign the host executor and build the Linux guest executor from the exact published
  source under `ExecutorBuildEvidenceV1` before baseline. It supplies those exact digests to the
  disposable exact-absence publisher bootstrap; guest bootstrap additionally requires a live
  operator to pin the full host-key fingerprint/challenge between independent host and guest
  controlling TTYs. Absence of that operator/TTY boundary is
  `BLOCKED_PLATFORM_HANDOFF_REQUIRED`. Final release/dev
  archive staging remains UNIX-owned. Native supported-Lima publisher/lifecycle/product/
  restoration proof is exclusively `EVIDENCE:R3-MAC-IMP-01`. Treat the graph-visible LOW results
  for private/cfg/Drop symbols as manual HIGH review surfaces; any edit to HIGH
  `new_with_mapping` or any new selector/caller stops.
- Platform/privilege: implementation and non-native fixtures only; publisher installation and
  Lima lifecycle are deferred to the evidence task. Exactly one product/test/review commit and one
  normal fast-forward push. Receipt successor is exactly `EVIDENCE:R3-MAC-IMP-01`.
- Non-goals/freeze: Unix uninstall bodies, Linux, Windows, ambient compatibility activation,
  alternate/newly selected VM, endpoint, transport, principal, or prefix, and unowned VM deletion.
  The sole absent-instance create path is the already-selected R2 Stage-1 identity under
  `LimaStageOneAuthorizationV1`.

### `EVIDENCE:R3-MAC-IMP-01` and `A1.1d-5R3-MAC-CLOSEOUT`

The evidence task discovers a native supported macOS checkout whose HEAD/tree/ref and live remote
equal the published `MAC` receipt, then runs only the MAC IMP actions in `05`, including exact
publisher bootstrap, human-pinned signed guest pairing, and externally receipted test-retirement
back to LaunchDaemon/System-Keychain
baseline. It changes no repository byte and validates a
`codex.top-level-evidence-receipt.v1` whose source remote equals its source commit. The separately
validated artifact's exact `gated_successor` is
`AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT`; platform absence and failed native proof map to the
same two platform statuses as Linux.

`MAC-CLOSEOUT` has empty production/test allowlists. It may add exactly
`review-control/r3-mac-imp-01-evidence.json`,
`review-control/r3-mac-imp-01-receipt.json`, the exact `mac-closeout` review set, and append its
status to `R3-DOCS`. It revalidates the evidence receipt, makes one documentation/evidence commit,
and performs one fast-forward push. Its receipt successor is exactly
`AUTHORITY_REQUIRED:A1.1d-5R3-WIN`.

### `A1.1d-5R3-WIN`

**Completion claim.** Implement exact A-local versus SID+instance+machine-ID+pipe shared Windows
lifecycle and convergence. Predecessor: `MAC-CLOSEOUT`; successor:
`EVIDENCE:R3-WIN-IMP-01`.

- Ownership: PI-047, PI-053, PI-096, PI-097, PI-102,
  `A1D5I-INSTALL-07`, `A1D5I-INSTALL-10`, `R3-ACT-049-WIN`, and `R3-ACT-070-WIN`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-windows.rs` symbols `main`,
  `WindowsManagedArtifactExecutorV1`, `open_windows_lifecycle_capsule_v1`,
  `join_windows_role_identity_v1`, `execute_windows_managed_action_v1`,
  `restore_windows_managed_role_v1`, `publish_windows_action_receipt_v1`,
  `WindowsLifecyclePublisherServiceV1`, `open_machine_protected_state_v1`, and
  `compare_and_swap_windows_publisher_protected_state_v1`, `bootstrap_windows_publisher_v1`,
  `export_windows_p256_spki_der_v1`, `normalize_windows_p256_signature_p1363_low_s_v1`,
  `resume_windows_publisher_bootstrap_v1`, `run_windows_named_pipe_publisher_v1`,
  `accept_windows_publisher_client_v1`, `impersonate_windows_publisher_client_v1`,
  `attest_windows_publisher_client_image_v1`, `handle_windows_publisher_request_v1`,
  `issue_wsl_guest_pairing_ticket_v1`, `consume_wsl_guest_pairing_ticket_v1`,
  `open_windows_guest_pairing_record_v1`,
  `compare_and_swap_windows_guest_pairing_record_v1`,
  `prove_wsl_guest_reservation_unused_v1`,
  `commit_wsl_guest_reservation_unused_acknowledgement_v1`,
  `issue_wsl_guest_bootstrap_v1`, `join_wsl_guest_anchor_v1`, and
  `retain_wsl_guest_bootstrap_channel_v1`, `join_wsl_guest_bootstrap_transcript_v1`, and
  `retire_windows_test_publisher_v1`, `retire_wsl_guest_test_publisher_v1`; its unsafe/Win32 fence is exactly
  `windows_named_pipe_ffi_v1`, `windows_client_attestation_ffi_v1`,
  `windows_cng_key_ffi_v1`, `windows_registry_anchor_ffi_v1`,
  `windows_service_control_ffi_v1`, and `windows_flush_file_ffi_v1`;
  `crates/shell/src/execution/managed_lifecycle/windows_client.rs` replacement symbols
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`,
  `open_windows_publisher_pipe_v1`, `attest_windows_publisher_response_v1`, and
  `issue_guest_publisher_pairing_ticket_v1`; no other
  control/core symbol may change;
  `scripts/windows/install-substrate.ps1` new
  `Open-CurrentAttemptTempRootV1`, `Register-CurrentAttemptTempMemberV1`,
  `Expand-CurrentAttemptTempArchiveV1`, `Rollback-CurrentAttemptTempRootV1`, and
  `Install-ManagedWindowsPrefixV1`, replacing exactly the top-level range from
  `$tempRoot = Join-Path ([System.IO.Path]::GetTempPath())` through the closing brace of the
  existing `finally` block. The replacement acquires and retains the temp parent/root handles,
  registers the checksum, payload, bundle, and every extracted descendant before bytes are
  published, and invokes `CurrentAttemptTempRollbackV1` on every handled-error/finally exit; only
  after that safe-rollback fence may it route version/bin/profile/shim/WSL provisioning and doctor
  mutations through the typed publisher. No path-only `Remove-Item -Recurse` remains;
  `scripts/windows/dev-install-substrate.ps1` new
  `Install-ManagedWindowsDevPrefixV1` replacing exactly the top-level range from the
  `Get-Command cargo` prerequisite through `Write-Log 'Substrate dev install complete.'`,
  including build argv, shim deployment, and profile-helper creation;
  `scripts/windows/uninstall-substrate.ps1` new `Invoke-ManagedWindowsUninstallV1` replacing the
  top-level range from `$forwarderPidPath =` through the final WSL unregister block; the replacement
  removes the unregister action entirely and returns a preserving scope-expansion result if a
  caller requests instance/install-tree deletion;
  `scripts/windows/dev-uninstall-substrate.ps1` new `Invoke-ManagedWindowsDevUninstallV1`
  replacing the top-level range from `$repoRoot =` through the final dev-uninstall status block,
  including the existing `--shim-remove` invocation;
  `scripts/windows/wsl-stop.ps1` parameter/intake surface adding mandatory `InstallPrefix`,
  `InstallBootstrapContextV1`, `PlatformBootstrapMappingV1`, and committed `PipePath`, plus new
  `Resolve-ManagedWindowsStopAuthorityV1`, `Assert-ManagedWindowsStopAuthorityV1`, and
  `Stop-ManagedWindowsPlatformV1`; the complete destructive body from the first status line through
  final pipe observation is replaced by one request. The broad process-name scan is tombstoned and
  distro termination is allowed only for the exact already-registered PM identity; and
  `scripts/windows/wsl-warm.ps1` new `Start-ManagedWarmForwarderV1` replacing the top-level range
  from `$logDir = Join-Path $env:LOCALAPPDATA 'Substrate\\logs'` through the final forwarder
  capabilities probe. The replacement derives both log directory and PID record only from the
  already-validated PM (`MappingState.ForwarderLogDir` and `MappingState.SharedForwarderPidPath`),
  manifest-records the exact log-directory before-state before creation, and includes stale-PID
  handling and PID-record publication; new `Save-WslGuestUnusedProofV1` and
  `Confirm-WslGuestUnusedProofV1` persist only to the precommitted external harness parent and
  accept only the exact LocalSystem-signed proof/HKLM reservation CAS plus the precommitted
  harness acknowledgement; new
  `Enter-ManagedExistingWslLifecycleV1` replacing only the unconditional fail-closed call
  immediately after the already-landed mapping assertion; it requires that mapping's exact
  already-registered distro and machine ID. `Reject-UnmanagedWslCreationV1` replaces the complete
  absent-distro import branch and returns `BLOCKED_SCOPE_EXPANSION` without download, directory
  creation, import, or cleanup. New `Provision-ManagedExistingWslV1` replaces the complete
  provisioning range from the provisioning-script lookup through both the unhealthy/force branch
  and the healthy binary-refresh branch's closing brace; it is syntactically closed through the
  current final `Install-GuestWorldBinaries` call and may act only through joined host/guest
  publisher requests. The mapping resolver/assertion and every selector are frozen. New
  `scripts/wsl/units/substrate-world-service.service.tmpl` and
  `scripts/wsl/units/substrate-world-service.socket` contain only the byte-exact canonical
  templates frozen in `04`; no other unit generator/template path is allowed. New
  `scripts/wsl/provision.sh` bounded replacement of its unconditional live guard/body with
  `render_managed_wsl_world_service_unit_v1` and
  `invoke_managed_wsl_guest_lifecycle_v1` is allowed only after the Windows publisher has issued a
  single-use host-signed pairing ticket, the exact staged Linux executor has been verified
  against the source receipt, an operator has pinned the host-key fingerprint/challenge through
  the guest controlling TTY, and the guest root publisher has joined that ticket/transcript; no ambient
  guest selection or package/passive-health remediation is allowed; and
  `scripts/windows/start-forwarder.ps1` new `Start-ManagedForwarderV1` replacing the range from
  `$logDir = $mappingState.ForwarderLogDir` through both readiness and wait-mode terminal
  branches. It records or joins `windows.prefix.forwarder-log-directory` before creation and then
  starts/stops only the exact manifest-bound child; all R2 mapping construction, validation,
  assertion, and selection logic before this range remains frozen.
- Mutable test/fixture allowlist:
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`,
  `crates/shell/tests/installer_env_wcu4.rs`, new
  `scripts/windows/dev-lifecycle-r3.Tests.ps1`, and new
  `scripts/windows/lifecycle-r3.Tests.ps1`. Required cases cover default/custom A under hostile
  B, two prefixes, unrelated `.substrate*`, reparse points, missing/malformed/tampered/stale/
  cross-SID/cross-instance/partial manifests, version/bin/profile/shim identity, PID/image/start
  identity, pipe scope, timeout/stop/uninstall order, partial install, repeat lifecycle, other
  distros, exact existing-instance start/stop/restoration, preserving rejection of WSL import,
  install-tree deletion, or unregister; exact A-local forwarder-log before-state/restoration under
  hostile `LOCALAPPDATA`; pre-existing/replaced/reparse log directories; and
  `CurrentAttemptTempRollbackV1` kill points at root create/open, every checksum/bundle/payload/
  extracted-child registration and write, reverse removal, and empty-root proof without reparse
  traversal; plus pairing ticket PM/machine/artifact scope, full fingerprint/challenge/literal
  confirmation on independent host/guest terminals, one-use consumption/exact retry, non-TTY or
  automated confirmation rejection, malformed/substituted SPKI, wrong curve/encoding,
  SPKI/fingerprint/anchor mismatch, high-S signature, alternate child/key/channel/transcript,
  durable host-record/guest-intent restart, and every pairing kill point; exact pre-existing versus
  created WSL `/run/substrate`, `/var/lib/substrate`, lifecycle-container, publisher-directory, and
  executor-parent roles; exact `substrate` group, PM-mapped guest membership, and
  root:`substrate` `0660` `/run/substrate.sock` create/readback/restore order; distinct
  service-template-source digest, exact PM render inputs, rendered-service digest, socket-source
  digest, and unchanged installed-socket digest; PM-bound template substitution/escaping,
  installed-template-marker and rendered/modified-socket rejection, hostile ambient HOME/context/mapping,
  root:root `0644` unit metadata, absence of implicit directory-management directives and proof
  that activation does not mutate either root/lifecycle subtree, wrong path/content/owner/mode
  preservation, daemon-reload-before-state ordering; one protected prepared transaction binding
  the socket service-state and its fixed coupled endpoint before-states before activation, exact
  endpoint identity/ACL observation in the same receipt, no independently requestable endpoint
  action, kill/retry at every activation/observation/receipt boundary, exact absence of
  service-to-socket propagation, and service stop/restart/restore with socket state and endpoint
  identity unchanged; the LocalSystem publisher and WSL guest publisher prove the same protected
  service-state/coupled-endpoint rule inside bootstrap/retirement; root pending/create/identity-CAS
  kill gaps; replacement/nonempty/wrong owner/mode/group/membership/socket identity or
  ACL preservation; both `Reserved` and `TicketIssued` challenge/component tuple unused
  revocation, the LocalSystem-signed proof and its externally fsynced hash, the precommitted
  harness acknowledgement and its externally fsynced hash, the exact HKLM reservation CAS, and
  pairing-record before-state restoration/removal, including replay, cross-reservation,
  missing-acknowledgement, and future/resulting-CAS-or-anchor-digest rejection; guest receipt/
  acknowledgement before reverse created-empty parent removal; and final WSL baseline parity.
- Documentation/control surface: exact `R3-DOCS` and the exact `win` review set. Evidence bytes
  are not created or committed in this implementation packet.
- Checks: Pester/static tests, shell installer static integration, non-native lifecycle-model and
  restoration assertions, diff/allowlist/secret/change detection. IMP fixtures use non-executing
  mock artifact identities. Only `EVIDENCE:R3-WIN-IMP-01` may natively build the Windows executor
  and build the Linux guest executor from the exact published source under
  `ExecutorBuildEvidenceV1`; publisher bootstrap verifies and copies that exact hash to the
  LocalSystem service identity before any other role. Guest bootstrap additionally requires a
  live operator to pin the full host-key fingerprint/challenge between independent host and guest
  controlling TTYs; absence is `BLOCKED_PLATFORM_HANDOFF_REQUIRED`. Native Windows/WSL lifecycle/product/
  restoration proof is
  exclusively `EVIDENCE:R3-WIN-IMP-01`. Final cargo-dist archive wiring remains UNIX-owned.
  PowerShell/script actions
  require manual caller closure because GitNexus under-resolves them.
- Platform/privilege: implementation and static/portable fixtures only; LocalSystem publisher and
  WSL lifecycle are deferred to the evidence task. Exactly one product/test/review commit and one
  normal fast-forward push. Receipt successor is exactly `EVIDENCE:R3-WIN-IMP-01`.
- Non-goals/freeze: any change to the existing R2 WSL mapping resolver/assertion/selector,
  missing-distro import or unregister, unmanifested WSL adoption, wildcard/default/ambient
  selection, process-name or stale-PID authority,
  Unix/Linux/macOS, policy redesign.

### `EVIDENCE:R3-WIN-IMP-01` and `A1.1d-5R3-WIN-CLOSEOUT`

The evidence task discovers a native supported Windows/WSL checkout whose HEAD/tree/ref and live
remote equal the published `WIN` receipt. It runs only the WIN IMP actions in `05`, including
LocalSystem/CNG/HKLM publisher bootstrap, human-pinned signed guest pairing, and externally
receipted test-retirement to baseline,
changes no repository byte, and validates
`codex.top-level-evidence-receipt.v1`. The separately validated artifact's exact
`gated_successor` is
`AUTHORITY_REQUIRED:A1.1d-5R3-WIN-CLOSEOUT`; absence and failure use the closed platform statuses.

`WIN-CLOSEOUT` has empty production/test allowlists. It may add exactly
`review-control/r3-win-imp-01-evidence.json`,
`review-control/r3-win-imp-01-receipt.json`, the exact `win-closeout` review set, and append its
status to `R3-DOCS`. It revalidates the native receipt, makes one documentation/evidence commit,
and performs one fast-forward push. Its receipt successor is exactly
`AUTHORITY_REQUIRED:A1.1d-5R3-UNIX`.

### `A1.1d-5R3-UNIX`

**Completion claim.** Convert Unix prefix/shim/payload/profile install, replacement, and uninstall
to the manifest core and integrate only the already-landed Linux/macOS providers. This packet owns
the row-level convergence of bundled PI-012/026/095. Predecessor: `WIN-CLOSEOUT`; successor:
`EVIDENCE:R3-NATIVE-LINUX-01`.

- Ownership: PI-012, PI-026, PI-064, PI-074, PI-092, PI-093, PI-094, PI-095,
  `A1D5I-INSTALL-02`, `A1D5I-INSTALL-03`, `R3-ACT-007-RB`, and `R3-ACT-019-RB`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-unix.rs` symbols `main`,
  `UnixManagedArtifactExecutorV1`, `open_unix_lifecycle_capsule_v1`,
  `join_unix_role_identity_v1`, `execute_unix_managed_action_v1`,
  `restore_unix_managed_role_v1`, and `publish_unix_action_receipt_v1`;
  `Cargo.toml` only the exact `[package.metadata.dist.binaries]."*"` value, adding
  `substrate-lifecycle-control`, `substrate-lifecycle-linux`, `substrate-lifecycle-macos`,
  `substrate-lifecycle-windows`, and `substrate-lifecycle-unix` to the existing two names;
  `scripts/substrate/dev-install-substrate.sh` functions
  `write_host_state_metadata`, `stage_managed_bundle_symlink`,
  `stage_managed_linux_binary_copy`, `clear_managed_prefix_linux_binary_cache`, and
  `cleanup_legacy_world_enable_helper_bridge`, plus new
  `append_platform_lifecycle_build_flags_v1`, `stage_lifecycle_executors_v1`, and
  `run_managed_dev_install_v1`; the exact top-level range from
  `TARGET_DIR="${PROFILE}"` through the final `write_host_state_metadata` call is replaced by the
  one orchestrator call, so build argv, config/payload/bin/shim/cache/profile creation and both
  provider integrations cannot bypass the manifest;
  `scripts/substrate/dev-uninstall-substrate.sh` existing
  `kill_live_dev_owner_helpers`, `remove_managed_symlink`,
  `remove_managed_prefix_linux_binary_copies`, `load_host_state_metadata`,
  `perform_auto_cleanup`, and new `run_managed_dev_uninstall_v1`, which replaces exactly the
  top-level range from `kill_live_dev_owner_helpers` through the final
  `perform_auto_cleanup "${cleanup_user}"` call; `kill_live_dev_owner_helpers` and the public
  `--kill-live-processes` branch are tombstoned into a preserving
  `BLOCKED_SCOPE_EXPANSION` result before enumeration or signal. Hidden owner-helper launch/runtime
  authority belongs to the excluded retained-worker/session program and is not widened here; R3
  never infers or kills that process. Uninstall proceeds only when the separately observed exact
  lifecycle prerequisite says no live owner helper consumes the prefix;
  `scripts/substrate/install-substrate.sh` functions
  `prepare_tmpdir`, `cleanup`, `write_host_state_metadata`, `prepare_bundle_payload`,
  `link_binaries`, `deploy_shims`,
  `harden_shim_symlinks`, `provision_linux_world`, and new
  `install_managed_release_prefix_v1`; in `install_macos` and `install_linux`, the exact range from
  each `prepare_bundle_payload` call through its final `write_host_state_metadata` call is replaced
  by one orchestrator call, including payload/version/bin/executor/shim/profile/provider staging;
  `provision_linux_world` removes exactly `PartOf=substrate-world-service.service` from its socket
  source and adds no replacement propagation relationship; `prepare_tmpdir`, every
  `prepare_bundle_payload` child-creation/extraction call, and `cleanup`
  jointly implement `CurrentAttemptTempRollbackV1`: retain parent/root descriptors and physical
  identities before the first download, register/rejoin the closed descendant set, and perform
  reverse descriptor-relative removal. Pre-existing, replaced, symlinked, unregistered, non-
  joined, or ambient `TMPDIR` is never recursively removed;
  `scripts/substrate/uninstall-substrate.sh` existing `remove_path_snippet`,
  `remove_shell_path_snippets`, `load_host_state_metadata`, `perform_auto_cleanup`, and new
  `run_managed_release_uninstall_v1`, which replaces exactly the top-level range from
  `log "Stopping substrate processes (if any)..."` through the final
  `perform_auto_cleanup "${cleanup_user}"` call;
  `scripts/substrate/dev-shim-bootstrap.sh::{install_shims,write_env_file,uninstall_shims,
  remove_env_file}`;
  `crates/shell/src/execution/shim_deploy.rs::{ShimDeployer::with_context,ensure_deployed,
  deploy_shims,migrate_old_shims}`; and
  `crates/shell/src/execution/invocation/plan.rs` new
  `remove_managed_shims_v1` plus only the `cli.shim_remove` callsite inside
  `ShellConfig::from_cli`.
- Mutable test/fixture allowlist:
  `crates/shell/tests/shim_deployment.rs`,
  `tests/installers/dev_shim_bootstrap_context_r2_1.sh`,
  `tests/installers/install_state_smoke.sh`,
  `tests/installers/prefix_propagation_r2_1.sh`,
  `tests/installers/prefix_propagation_r2_2.sh`,
  `tests/installers/install_smoke.sh`, and new
  `tests/installers/unix_lifecycle_r3.sh`. Required cases cover first/repeat/partial/retry install,
  first/repeat uninstall, uninstall after failure, reinstall, hostile B/two-home/two-prefix,
  link/target and profile bytes/metadata, closed subtree, unrelated sibling, and every manifest
  rejection class, plus preserving rejection of `--kill-live-processes` before process
  enumeration/signal and `CurrentAttemptTempRollbackV1` kill points before/after root creation/open,
  every child registration/write/extract, reverse removal, and empty-root proof; generated Linux
  socket bytes omit service-to-socket propagation, and service stop/restart/restore leaves the
  independently owned socket state and endpoint identity unchanged.
  `tests/mac/installer_parity_fixture.sh`
  is a run-only regression path owned for
  mutation by `MAC`; its bytes must not change here.
- Documentation/control surface: exact `R3-DOCS` plus the exact `unix` review set.
- Checks: all listed fixtures, focused and broad shell tests, format/clippy, non-privileged install
  matrix, provider-integration dry fixtures, `cargo dist plan` and archive-content checks proving
  every target contains the five lifecycle binaries (wrong-platform binaries exit before parsing
  or mutation), exact dev build/staging assertions,
  diff/allowlist/secret/change detection. GitNexus
  currently reports `ShellConfig::from_cli` HIGH (8 direct/10 total graph-visible) and
  `ShimDeployer::ensure_deployed` at least MEDIUM under full caller closure; any edit outside the
  shim action block or new production caller is a stop.
- Platform/privilege: Unix implementation, no native platform mutation in non-native checks.
  Native effects are deferred to the final three evidence tasks. One commit/push. The receipt
  successor is `EVIDENCE:R3-NATIVE-LINUX-01`; the two subsequent evidence tasks are separately
  authorized and all three bind this same published UNIX checkpoint.
- Non-goals/freeze: internals of the Linux/macOS providers, Windows, private-home rollback,
  recursive unmanifested removal, broad kill, ambient-home cleanup, passive health.

### Final native evidence task sequence

`EVIDENCE:R3-NATIVE-LINUX-01`, `EVIDENCE:R3-NATIVE-MAC-01`, and
`EVIDENCE:R3-NATIVE-WIN-01` are three read-only repository tasks with the platform/privilege,
action, proof, restoration, and artifact contracts in `05`. Each independently discovers a clean
checkout whose HEAD/tree/ref and live remote equal the exact published `UNIX` receipt; none may
use the preceding evidence task as a changed source checkpoint. Each validates a native
`codex.top-level-evidence-receipt.v1` with the orchestration skill's evidence validator. Their
successors are respectively `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-MAC-01`,
`AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-WIN-01`, and
`AUTHORITY_REQUIRED:A1.1d-5R3-CLOSEOUT`. They change no repository byte and create no commit or
push. Platform absence returns `BLOCKED_PLATFORM_HANDOFF_REQUIRED` with the full handoff; a run
failure or restoration mismatch returns `BLOCKED_NATIVE_EVIDENCE`.

### `A1.1d-5R3-CLOSEOUT`

**Completion claim.** Ingest and revalidate the three clean remote-equal native evidence receipts
for the exact landed `UNIX` checkpoint, run only non-mutating cross-platform regression/review
walls, close all R3 findings/gates, and update documentation/review control. No production or test
behavior may change. Predecessors: all three final native evidence tasks; successor:
`AUTHORITY_REQUIRED:A1.1d-CLOSEOUT`.

- Ownership: `A1D5I-REG-01`; final joins for `RG-HOME-01`, `RG-INSTALL-01`, `R3-LIFE-01`,
  `R3-WIN-01`; verification only for every R3 PI/finding.
- Production/test allowlist: empty. Tests and native smokes may run but no source/test/fixture
  byte may change.
- Documentation/control allowlist: exact `R3-DOCS`,
  `llm-last-mile/runtime-refactor/01-target-architecture.md`,
  `review-control/r3-closeout-review-cycle-record.json`,
  `review-control/r3-closeout-review-authority-security.md`,
  `review-control/r3-closeout-review-lifecycle-convergence.md`,
  `review-control/r3-closeout-review-allowlist-evidence.md`,
  `review-control/r3-native-linux-01-evidence.json`,
  `review-control/r3-native-linux-01-receipt.json`,
  `review-control/r3-native-mac-01-evidence.json`,
  `review-control/r3-native-mac-01-receipt.json`,
  `review-control/r3-native-win-01-evidence.json`, and
  `review-control/r3-native-win-01-receipt.json`.
- Gates: `R3-NATIVE-LINUX-01`, `R3-NATIVE-MAC-01`, and `R3-NATIVE-WIN-01` as specified in `05`;
  broad filesystem/network/policy/config/dependency/gateway/runtime/shim/replay/trace/diagnostic
  and PTY/non-PTY regression; installer/uninstaller symmetry; exact restoration; independent
  bounded review CLEAN with zero unresolved P1-P4.
- Publication: one docs/evidence commit and one normal fast-forward push only after live target
  revalidation. Missing platform or evidence yields `BLOCKED_PLATFORM_HANDOFF_REQUIRED`;
  mismatch, incomplete restoration, or open finding blocks publication. No weakened/static
  substitute is accepted.
- Non-goals/freeze: every production symbol, test, fixture, script, dependency, schema, generated
  product artifact, platform selection, passive health, and later runtime-refactor packet.
**Source provenance:** extracted from [`../03-phase-slice-map.md#a11d-5r3-authoritative-implementation-index`](../03-phase-slice-map.md#a11d-5r3-authoritative-implementation-index), baseline lines 2228–3096
**Relocation note:** repository-relative Markdown targets were rebased as needed to preserve their original repository destinations after relocation.
**Baseline span SHA-256:** `11d3d8e34f66e62637f6507d661e81dff04f93769c48bdffec3321838448b5f2`
