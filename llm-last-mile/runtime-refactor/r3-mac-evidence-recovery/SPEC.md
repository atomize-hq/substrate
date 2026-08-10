# SPEC — AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN

## Status and authority

- **Packet:** `AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN`
- **Orchestration:** `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- **Dispatch nonce:** `c1f964d03d31a82ad5bf8b15a61b1a90cb618e20199b83b5674c6552a38a1798`
- **Planning base:** `d8a65fc8890dd37584aaeac2984c906188e5f06e` / `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`
- **Publication for this packet:** one documentation/review commit only; no product behavior, installation, Keychain/XPC/Lima action, native evidence, or successor dispatch.
- **Only possible next increment:** `AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-IMPLEMENTATION`.

This specification replaces neither the frozen architecture nor the historical `A1.1d-5R3-MAC`
review epoch. It supplies the exact recovery authority needed to construct a *new*, small,
exact-base source subject for a future official `EVIDENCE:R3-MAC-IMP-01` attempt.

## Frozen assumptions and evidence inputs

1. The target ref is remote-authoritative and must remain at the planning base until this planning
   commit is published. The local target ref is not publication authority.
2. The preserved donor is read-only at
   `/Users/spensermcconnell/.codex/worktrees/60cc/substrate`, has the same base/tree, and its
   tracked binary diff is exactly
   `39682fd4415a00ae4099580135a687d3e11858f62f84e9ef6ff3771bafd4b54c`.
3. The donor has 26 tracked paths, two untracked regression paths, and `+8821/-681` tracked
   lines. It is a hunk source, never a branch, patch, publishable subject, or implementation
   baseline.
4. Amendment `0008-r3-mac-source-correction-scope-freeze` and its independent scope audit are
   controlling historical evidence. Their two unresolved P1s are real: the one-stdio pairing model
   cannot provide the required independent guest TTY, and `lima-lifecycle.sh` sends `lima-action`
   to an executor that rejects it.
5. The existing control pack requires the fixed root LaunchDaemon to use one software P-256 signer
   in the explicitly opened legacy `/Library/Keychains/System.keychain`; a sufficiently privileged
   root process may export its private material. Canonical SPKI DER/hash and P1363 low-S
   verification, code-designated/audit-token-before-decode peer admission, signed wrapper and
   generation-CAS joins, Stage-1 joins, a PM/machine/source/artifact-bound one-use ticket, and
   independent host/guest terminal confirmation remain mandatory and preserving-first. It does
   **not** authorize raw helper bypasses, a user-keychain, file-key, default/ambient store,
   unsigned or alternate fallback, or prove that one child stdio stream is exclusive authority.
   R3 supports Apple Silicon macOS only. Secure Enclave/Data Protection Keychain and a
   user-LaunchAgent signer are deferred hardening, not current authority.

## Selected outcome

Publish a review-clean planning packet that allows a later, separately authorized implementation to
recover only the source corrections needed for a fresh official MAC evidence attempt. The later
implementation must select audited donor hunks into a new exact-base worktree; it must never stage,
commit, or publish the preserved donor wholesale.

Success is a set of ordered recovery packets with exact file/symbol/test fences, a durable decision
for the data/TTY and mapped-lifecycle joins, a salvage/discard matrix for all 28 donor paths, and
proof gates that distinguish non-native implementation proof from official native evidence.

## Required recovered behavior

### A. Retained independent corrections

The recovery must retain, with their focused regressions:

1. the Bash-3.2-safe installer context reader that does not allocate/close a caller-owned file
   descriptor;
2. macOS cfg/compile admissions and the installer compile surface for the two lifecycle binaries;
   ordinary Linux host behavior remains byte/behavior compatible; and
3. caller-supplied, trusted `--expected-product-project-id` validation, including the historical
   Linux fixture's explicit historical project ID.

These are independent of the abandoned pairing prototype and may be reviewed and landed before it.

### B. Trusted mapped lifecycle submit/bootstrap bridge

The later implementation must replace raw `lima-action` with the closed
`submit-mapped-lifecycle-v1` control operation. `scripts/mac/lima-lifecycle.sh` may invoke only the
fixed installed `substrate-lifecycle-control`; it cannot invoke the protected helper, construct an
XPC request, select an operation, or pass a ticket/key/state record.

`submit-mapped-lifecycle-v1` is a tagged union with three disjoint, closed branches; no branch
accepts a generic operation string, caller path, caller ticket/record/key, or raw `lima-action`.

| Branch | Sole admission and effect | Mandatory rejection |
|---|---|---|
| `stage_one_absent_instance_create` | carries `InstallBootstrapContextCarrierV1`, the pre-PM mapping, `LimaStageOneAuthorizationV1`, `ExecutorBuildEvidenceV1`, and exactly `mac.lima.instance` / `Create`; the executor rederives the fixed selected instance, proves absence, anchors Stage-1, creates, observes machine ID, finalizes PM, and durably publishes the full instance manifest before any guest projection. | every other role/action, pre-existing/ambiguous instance, post-PM forwarding, ticket issue, guest projection, reuse after any result, or an unjoined create receipt. |
| `post_pm_managed_action` | carries the complete canonical `ManagedLifecyclePublisherRequestV1` plus carrier/mapping/build joins; after fixed XPC admission the executor recomputes the request, PM, role/action, locator, and peer audit-token/designated-requirement before the one allowed observed transition and durable receipt. | any `LimaStageOneAuthorizationV1`, bootstrap role, unknown field/pair, stale anchor/manifest/nonce, or caller-selected target. |
| `publisher_bootstrap_direct_interactive` | is reachable only from hidden direct-interactive `substrate-lifecycle-control publisher-bootstrap`; its independently issued `PublisherBootstrapAuthorizationV1` is delivered immediately over the retained bootstrap channel and joined to the controlling terminal, image, build, peer, and exact bootstrap record. | script/wrapper issuance or replay, serialization to a request/file/environment/output, Stage-1 substitution, absent controlling terminal, or a normal `submit-mapped-lifecycle-v1` request without that independent authorization. |

The R5 wrapper exposes only the first two tags; it cannot invoke the third. The protected executor
rejects every publisher-bootstrap role unless the separate direct-interactive authorization is present
and its retained-channel/terminal joins still verify. `LimaStageOneAuthorizationV1` is create-only
and is never carried, accepted, or replayed for forwarding or any post-PM role.

Ordering is fixed:

`request-tag validation -> fixed XPC admission/audit token before decode -> branch-specific
recomputation -> (Stage-1 absent create and PM finalization **or** complete post-PM request join,
or retained direct bootstrap authorization) -> one mapped action -> durable receipt -> caller
observation`.

No caller can retry an ambiguous action. A killed, missing, mismatched, or unreceipted state stops
preserving-first; retry must exact-join the protected record or return the closed blocked status.

### C. Two logical sessions on one PM-bound SSH transport

The recovery explicitly rejects the donor's `lima-stdio-v1` exclusivity premise. The compliant
replacement has exactly two logical sessions, both created from the same already-accepted
`PlatformBootstrapMappingV1` and the same PM-bound SSH transport identity:

| Session | Purpose | Forbidden data |
|---|---|---|
| `GuestPublisherPairingDataSessionV1` | Delivers the signed ticket and exchanges typed `Hello`, signed transcript, and guest-anchor frames. | Operator confirmation, arbitrary commands, a caller-supplied ticket/record/key, or a second transport selector. |
| `GuestPublisherPairingOperatorTtySessionV1` | Opens an independently controlling guest TTY, displays scope/fingerprint/challenge, and receives the manual exact fingerprint, challenge, and literal. | Ticket/transcript frames, stdin aliasing, argv/env/file/pipe confirmation injection, or a guest mutation. |

Both sessions bind the same `scope_id`, mapping commitment, guest machine identity, source
commit/tree/ref, staged guest executor digest, Stage-1 record digest, ticket challenge ID, host
record generation, and one fresh `pairing_session_nonce`. Their transport observations are evidence
only; the signature/key/ticket/record joins remain authority. The host record records two distinct
session identities and exact start/end/failure observations, then accepts a transition only if all
shared fields and the nonce match.

The attested host control is the **only** display source for the full host-key fingerprint, challenge
ID, and challenge; it prints those values only on its retained controlling terminal. Neither a
ticket, data frame, transcript, receipt, nor guest display supplies a confirmation value. The guest
opens the independent controlling TTY, displays the ticket scope and fingerprint, and requires the
operator to manually enter the full fingerprint, challenge, and literal `PAIR EXACT SUBSTRATE GUEST
PUBLISHER` obtained from that host terminal. The guest binds only a non-reusable confirmation
commitment to the protected generation-CAS record before intent publication; the reusable input is
never persisted, logged, projected, or returned.

The data session cannot cause a confirmation. If the host terminal is absent, the guest TTY is
absent/non-controlling, the values originate from ticket/data/argv/environment/file/pipe/automation,
or either session is mismatched, expires, ends early, replays, or cannot rejoin the current record,
the record remains preserved and no guest component mutates. Native evidence must prove this exact
capability; it may return `BLOCKED_PLATFORM_HANDOFF_REQUIRED` only when the necessary platform or
privilege is unavailable before action, never by substituting a one-stream or alternate transport
design.

### D. Minimum necessary protected surface

Retain only donor hunks that implement all of the following control-pack requirements:

- explicitly opened legacy `/Library/Keychains/System.keychain` with service
  `com.substrate.lifecycle.v1` for the exact software P-256 signer, current-anchor,
  bootstrap-intent, and challenge-keyed pairing-record identities; sufficiently privileged root
  may export the private material, but product signing never exports it;
- canonical SPKI DER export/hash and canonical P1363 low-S verification without a second wire
  representation, plus signed-wrapper, generation-CAS, and preserving-first joins;
- fixed root LaunchDaemon and Mach service admission: the actual peer audit token is read and
  admitted before any request byte is decoded; the exact code-designated requirement and fixed
  control image are checked;
- no user-keychain, file-key, default/ambient store, unsigned, software-file, or alternate fallback;
  Apple Silicon macOS only, with Secure Enclave/Data Protection Keychain and a user-LaunchAgent
  signer remaining deferred hardening rather than current authority;
- durable Stage-1 record/open/validation before ticket issue or mapped mutation;
- canonical ticket issue, expiry, PM/machine/source/artifact/component binding, host-record
  generation-CAS, one-use consume/replay rejection, and evidence-only unused reservation/retirement
  flow required for restoration proof; and
- descriptor/no-follow preservation of known-host/socket predecessor state where the existing MAC
  forwarding contract requires it.

Every other donor hunk is discarded unless a later recovery packet names it in its exact allowlist
and proves it necessary for one item above. In particular there is no retained raw `lima-action`,
`GuestPublisherPairingGuestChannelV1`, `GuestPublisherPairingStdioPhaseV1`,
`GuestPublisherPairingStdioFrameV1`, one-child-channel exclusivity rule, Windows provider behavior,
or ordinary Linux host pairing command.

## Boundaries

### Always

- Begin every recovery packet from an independently verified remote-equal predecessor and recreate
  selected donor hunks manually into a new exact-base worktree.
- Run GitNexus/manual impact before changing any existing source symbol; treat private/cfg/Drop,
  shell, FFI, and platform callers as manual high-risk surfaces when graph search is degraded.
- Require exact allowlist, secret scan, deterministic checks, a full subject fingerprint, causal
  review record validation after every cycle, and a fresh closure reviewer.
- Preserve the current R3 MAC review records and completed Linux evidence history.

### Ask/stop

- Stop `BASE_DRIFT` if the remote target differs from the named predecessor.
- Stop `BLOCKED_SCOPE_EXPANSION` for a new transport, selector, endpoint, principal, generic
  session broker, one-stdio substitute, raw helper fallback, Windows runtime behavior, Linux host
  pairing behavior, unlisted donor hunk, or native action outside the evidence task.
- Stop `BLOCKED_PLATFORM_HANDOFF_REQUIRED` when a native macOS host cannot prove the independent
  TTY/data-session capability or required protected primitives.

### Never

- Never publish/copy/reset/clean/checkout the donor, use it as a source tree, or reuse its old
  review-clean claim.
- Never run installation, publisher bootstrap, Keychain, XPC, Lima, code-signing, or official
  evidence in this planning packet.
- Never use confirmation through argv, environment, files, pipes, generated projections, or the
  data session; never treat a child channel as authentication.
- Never change `01-target-architecture.md`, Windows behavior, or frozen run-only paths.

## Acceptance criteria for this planning packet

- [ ] The three required planning documents and only the allowed current-status/control updates
  define all recovery packets, dependencies, stops, proof, and successors.
- [ ] The full 28-path donor hunk/symbol matrix identifies retention, recreation, or discard.
- [ ] The data/TTY and mapped-lifecycle decisions are closed enough that a later implementation
  does not choose an authority model.
- [ ] Every future packet has exact production/test/run-only/doc fences and non-native/native proof
  separation.
- [ ] The planning subject passes deterministic Markdown/reference, allowlist, secret, and diff
  checks; three fresh disjoint reviewers and a fresh closure reviewer end with zero unresolved P1/P2
  and this stricter packet with zero unresolved P1-P4.
- [ ] This packet is one remote documentation/review commit whose sole next value is
  `AUTHORITY_REQUIRED:AUX-R3-MAC-EVIDENCE-RECOVERY-IMPLEMENTATION`.
