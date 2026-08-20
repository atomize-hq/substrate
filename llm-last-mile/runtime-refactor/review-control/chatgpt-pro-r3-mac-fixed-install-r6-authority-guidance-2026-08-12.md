# ChatGPT Pro consultation — R3 macOS fixed-install / R6 authority

- Date: 2026-08-12
- Conversation: https://chatgpt.com/c/6a7cca99-8810-83ea-b5b4-b69d2c7e012d
- Mode: GPT-5.6 Sol, Pro effort

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Prompt

```text
We are already working on this project task.

Task/spec: Complete the bounded macOS R3 fixed-install and R6 pairing-continuation corridor so the canonical installer can finish privileged guest provisioning and then expose one secure, explicit, non-auto-running pairing continuation for native evidence. A proposed authority amendment is pending this consultation: authorize the privileged macOS executor, only after a completed fixed installation, to CAS-create one signed, single-use R6 pairing predecessor bound to the current manifest, finalized platform mapping, observed guest identity, current protected-state anchor, and the manifest-bound guest Linux executor digest; keep host Mach-O helper identity distinct from guest ELF identity; emit a newly time-bounded continuation that does not auto-run pairing. Reject the macOS --world-netfilter option on this path until separately signed/authorized.

Current approach: Repository-derived inference: keep the existing same-user Lima, signed Stage-1, protected Keychain/CAS, privileged launchd/XPC publisher, literal fixed post-PM action plan, no-follow provenance checks, and independent guest-terminal pairing design. Adjust the fixed-install implementation rather than pivot architectures. The uncommitted implementation currently attempts: descriptor-bound adoption of the exact four externally built Linux artifacts; fixed sudo-only guest mutations; stdin-based Lima artifact transfer; canonical guest units and service startup; and a data-only R6 continuation. It must now be remediated against causal-review findings before landing.

Project sources: No external Project source bundle is configured for this consultation. Use only the bounded facts below; excluded repository source is unavailable and must not be inferred.

Evidence, blockers, constraints, and validation:
- Exact base before the uncommitted correction: 5dc5d98ad76ee5f9758f7bc139e453c47ce46d20. The previous observation-frame correction is landed and tested.
- The live native path now completes signed Stage-1, creates/starts the Lima VM, and reaches stable guest observation. The next code corridor is fixed install, then explicit pairing and evidence smoke.
- Causal review found that the original control pack authorizes consuming an independently produced protected Stage-1/R6 predecessor but names no producer and no renewal route after expiry. It also found a bad host-Mach-O versus guest-ELF digest conflation. This is the authority gap the proposed amendment addresses.
- The amendment must not authorize a caller selector, generic scheduler/action, raw authorization serialization, automatic pairing, ownership-separated Lima, VSock, Windows/WSL/Linux-host changes, or weaker admission/provenance/CAS/retry controls.
- Pairing must remain independently terminal-confirmed and single-use. A continuation may be emitted only after fixed install and must carry no pairing ticket or reusable secret.
- Other review findings that implementation must resolve: canonical R6 seed fields cannot be empty; guest managed home must be the observed guest home plus .substrate rather than the already-existing Lima account home; EffectStarted recovery needs temp-file verification/fsync/atomic publication instead of partial final files; staged artifact keys must admit exact replacement without cross-generation collision; socket/service activation must not create a Before-only retry wedge; volatile /run paths need systemd ownership or revalidation; readiness must prove exact units/socket and endpoint behavior; paths with spaces must use field-specific systemd escaping; root adoption needs transactional rollback and executable adversarial tests; fixed install must not silently ignore netfilter.
- A key timing issue: the existing five-minute Stage-1 authority may expire during a legitimate slow fixed install. The requested amendment should define whether the post-install pairing predecessor and continuation receive a fresh, narrowly bounded validity window derived by the privileged executor, without renewing or replaying the original Stage-1 effect authority.
- Required closure: focused unit/integration regressions must fail on the base; actual production validators must round-trip the continuation; tests must cover interruption/retry and exact replacement; Bash 3.2 syntax, locked/offline Rust checks/tests, four-role AArch64 Linux build, formatting/diff checks, strict six-path-or-explicitly-amended allowlist, then fresh independent authority/security, lifecycle/convergence, and evidence/allowlist causal review. Native installer/evidence remains a separate later run.
- No source is committed, pushed, or natively executed from the uncommitted correction.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; tradeoffs; what to keep/change first; inspections; validation; and pitfalls. Specifically:
1. Is the proposed authority amendment the minimal proper authority, or does it omit or over-authorize any trust transition?
2. Give exact normative constraints for producer identity, CAS predecessor creation, guest-ELF binding, fresh post-install continuation lifetime, single-use/replay behavior, and failure/restoration semantics.
3. Should the fresh continuation be treated as a new pairing-only authority derived after fixed-install completion, rather than renewal of Stage-1 effect authority?
4. Identify any remaining causal contradiction that would still make the canonical installer/pairing/evidence path impossible after these fixes.
5. Give the safest implementation and review order.

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.
```

## Response

```text
Worked for 18m 23s
Recommendation: ADJUST

Keep the architecture and the basic post-install authority model. Do not pivot away from same-user Lima, the signed Stage-1 path, protected Keychain/CAS state, the fixed launchd/XPC publisher, the literal guest-install plan, or separate guest-terminal pairing.

The proposed amendment selects the correct producer locus—the existing privileged macOS executor that owns the protected state and signer—but it is not yet a complete minimal authority amendment. As written, it has three gaps and one potentially overbroad phrase:

“The privileged macOS executor” is too generic unless the amendment names one exact closed operation on the already-authorized executor identity.

“Current protected-state anchor” is ambiguous because predecessor creation itself advances that anchor.

“One predecessor” still lacks an expired-unused replacement path, recreating the expiry wedge identified by the causal review.

Stage-1 expiry semantics must be separated from effect convergence; otherwise a fresh post-install R6 authority cannot help a fixed-install effect that becomes unauthorized midway through legitimate recovery.

Apple’s platform model supports retaining the existing architecture: XPC services are launchd-managed, and clients connect to a launchd-advertised Mach service; the Security framework supplies the data-signing primitive. Nothing in the platform mechanics requires another helper, a new transport, VSock, or an ownership-separated VM. 
Apple Developer
+2
Apple Developer
+2

This is necessarily a contract-level review. The repository at 5dc5d98ad76ee5f9758f7bc139e453c47ce46d20 and the uncommitted correction were not supplied, so the local call graph, schemas, exact six-path allowlist, and tests remain mandatory verification gates rather than facts I can claim to have inspected.

1. Is the amendment the minimal proper authority?

It becomes the minimal proper authority after four narrow corrections.

What the amendment should authorize

It should authorize exactly this transition:

Following durable completion and revalidation of the already-admitted fixed R3 install, the exact existing privileged macOS executor may perform one closed, domain-separated protected-state transition that creates or restores the current R6 pairing predecessor and emits its corresponding data-only pairing continuation.

That is the right trust boundary because the executor already owns or mediates:

Protected-state reads and CAS.

System Keychain signing.

The authoritative fixed-install completion receipt.

The finalized platform mapping and guest observation.

The manifest and installed guest-artifact provenance.

The canonical installer should only request the transition and display or publish its result. It should not derive authority-bearing fields, sign the result, serialize Stage-1 authorization material, or choose what privileged action the executor performs.

What is currently omitted

The amendment must additionally define:

The exact dedicated operation, not a generic executor permission.

The authoritative fixed-install completion witness from which the R6 seed is derived.

Pre-CAS and post-CAS anchor semantics.

At-most-one-live-generation replacement after an unused predecessor expires.

The consumption transition and effect-visible recovery rule, even if the existing R6 consumer already implements it.

Stage-1 admission versus convergence semantics.

What would over-authorize

The following forms would be too broad:

A request containing an action name, selector, command, schedule, arbitrary path, arbitrary payload, or raw authorization document.

Caller-selected manifest, mapping, guest identity, artifact digest, state anchor, validity interval, or pairing mode.

Permission for the executor to mint R6 before an authoritative R3 completion state.

Permission to reissue R6 while another predecessor is live or pairing is EffectStarted.

A continuation that also authorizes installation, repair, publisher bootstrap, networking changes, or any other Stage-1 effect.

Automatic timer-based renewal or automatic pairing execution.

Treating --world-netfilter as accepted-but-ignored.

A dedicated XPC method is appropriate; a generic method such as execute(action, payload) is not. If routing requires an existing opaque attempt/retry locator, that locator may identify the current attempt, but it must not select an operation or supply authority-bearing values.

2. Normative constraints to adopt
A. Producer identity and request admission

The contract should say:

Only the exact existing privileged macOS executor implementation may produce the predecessor and continuation.

It MUST satisfy the project’s existing live host-code identity, executable provenance, image binding, peer identity, and protected-key access checks.

The launchd/XPC publisher is the transport boundary; it MUST NOT become a generic authority broker.

The policy identity and the observed executable identity should both be retained if that is already the project convention: for example, stable signing requirement plus the observed image/CDHash.

The operation MUST have a closed method and closed request schema.

The caller MUST NOT provide an operation selector, scheduler instruction, shell command, target path, validity interval, manifest digest, platform mapping, guest identity, state anchor, guest executor digest, signing payload, or raw Stage-1 authorization.

All such fields MUST be loaded from protected state and the committed R3 receipt by the producer.

Existing authenticated retry locators may be accepted only as locators, never as authority-bearing content.

The requesting process MUST still pass the existing canonical-installer peer/provenance checks.

A valid caller does not make its assertions trustworthy.

The executor MUST independently derive and verify all R6 bindings.

The producer MUST NOT require Stage-1 to remain unexpired at R6 creation.

It MUST prove that Stage-1 was valid when the fixed-install effect was admitted.

It MUST prove that the exact admitted effect reached durable R3 completion.

R6 authority then comes from the completed protected-state transition, not from replaying Stage-1.

A new signing key is not required for minimal closure.

The existing protected P-256 signer can be reused.

The signatures MUST use distinct cryptographic domain tags and distinct closed schemas, such as conceptual domains:

R6_PAIRING_PREDECESSOR_V1

R6_PAIRING_CONTINUATION_V1

A Stage-1 signature MUST never validate as either R6 document.

B. Authoritative R3 completion precondition

R6 creation MUST fail without changing state unless all of the following are true:

The fixed-install effect is durably Completed, not merely EffectStarted.

Its admission receipt proves Stage-1 was valid at admission.

The current manifest exactly matches the completed-install manifest.

The finalized platform mapping exactly matches the completed-install mapping.

The stable observed guest identity exactly matches the completed-install guest.

The observed guest home is nonempty and canonical.

The managed home is exactly:

<observed guest home>/.substrate

All four adopted Linux artifact records are complete and generation-correct.

The installed guest Linux executor bytes match the manifest-bound guest-executor role and digest.

The exact unit files, socket, runtime ownership, and endpoint behavior pass readiness revalidation.

No invalidating protected-state transition occurred after R3 completion.

--world-netfilter was not accepted on the macOS fixed-install path.

A caller’s statement that installation completed is not sufficient.

C. Canonical predecessor and continuation bindings

Every authority-bearing field must be nonempty and covered by the appropriate signature. Conceptually, the signed predecessor needs at least:

kind
schema_version
signature_domain
scope = pairing-only

producer_host_role
producer_host_code_identity
signer_spki_digest

stage1_admission_id or digest
fixed_install_effect_id
fixed_install_receipt_digest

manifest_digest
platform_mapping_digest

stable_guest_identity
observed_guest_home
managed_guest_home

guest_executor_role
guest_executor_format = ELF
guest_executor_arch = AArch64
guest_executor_sha256
guest_executor_size

fixed_install_anchor
parent_state_generation

predecessor_id
predecessor_generation

issued_at
expires_at

Names should follow the repository’s conventions, but those semantic distinctions should be mandatory.

The continuation must bind to the same predecessor and repeat or cryptographically reference the same immutable values. At minimum it needs:

kind = R6 pairing continuation
schema_version
signature_domain
scope = pairing-only

predecessor_id
predecessor_generation
fixed_install_anchor
committed_predecessor_anchor or committed CAS receipt

manifest_digest
platform_mapping_digest
stable_guest_identity
guest_executor_sha256

issued_at
expires_at

The continuation MUST contain:

No Stage-1 raw authorization.

No pairing ticket.

No private key or reusable secret.

No generic action or command.

No networking or netfilter authority.

No automatic-execution instruction.

A public predecessor ID or nonce may exist for uniqueness and replay detection, but possession of it MUST NOT be sufficient to pair. The protected current state and independent terminal confirmation remain necessary.

Unknown fields, duplicate fields, missing fields, empty strings, malformed digests, noncanonical timestamps, and incorrect domain tags must fail closed.

D. Host Mach-O identity versus guest ELF identity

These must be separate typed domains, not two instances of a generic unqualified digest:

Producer identity: the live host Mach-O executor’s code-signing identity and, where already required, its observed image binding.

Guest executor identity: the content digest of the exact AArch64 ELF artifact assigned to the guest-executor manifest role.

Normatively:

The host code identity MUST never satisfy the guest artifact binding.

The guest SHA-256 MUST never satisfy the host code-signing predicate.

The two values MUST have distinct field names, parser types, comparison functions, diagnostics, and test fixtures.

The guest binding MUST include the manifest role, object format, architecture, digest algorithm, digest, and preferably exact size.

The digest used at R6 creation MUST originate from the descriptor-bound artifact adoption record, not from a caller string or a re-opened untrusted host pathname.

The final installed guest path MUST be reopened without following symlinks and verified against that exact digest before R3 completion and again before R6 creation or consumption according to the existing trust model.

This fixes the causal review’s Mach-O/ELF conflation without introducing a second helper identity or a different VM architecture.

E. CAS and anchor semantics

Do not use one field called simply “current anchor.” Define two states explicitly.

Let:

A3 be the protected-state anchor of the durably completed fixed install.

P6 be the signed predecessor whose body binds fixed_install_anchor = A3.

A6 be the protected-state anchor or commit receipt resulting from atomically committing P6.

Then:

The predecessor MUST bind to A3, the pre-CAS fixed-install anchor.

The CAS MUST atomically transition from the exact expected R3 state to a state containing the exact signed predecessor or a durable, verified locator for its exact bytes.

The continuation MUST bind to both:

The R3 source anchor A3.

The committed predecessor state A6, its generation, or an equivalent unambiguous CAS receipt.

The continuation MUST NOT be emitted before the CAS succeeds.

The consumer MUST NOT compare A3 directly with the post-creation current anchor and reject merely because creation advanced the state.

At consumption, the protected state must prove that P6 remains the current, unconsumed predecessor or a permitted descendant retaining that predecessor.

A CAS conflict requires re-reading protected state:

Exact same live predecessor: return the persisted continuation.

Different live predecessor or changed binding: fail closed.

EffectStarted or consumed state: enter recovery/status handling, not new creation.

This removes a potential self-invalidating transition.

F. Fresh validity window

Yes: give the predecessor and continuation a fresh pairing-only window.

For this bounded corridor, I recommend:

R6_PAIRING_VALIDITY = 300 seconds

The normative rules should be:

issued_at is sampled by the privileged host executor only after final R3 completion revalidation.

expires_at = issued_at + 300 seconds.

Validity is the half-open interval:

issued_at <= host_now < expires_at

The authoritative expiry check is performed by the trusted host consumer, not by trusting the guest wall clock.

The interval MUST NOT be caller-selectable.

The interval MUST NOT inherit the Stage-1 issuance time or expiration.

Re-emitting the same persisted continuation after response loss MUST NOT refresh the interval.

Re-signing the same predecessor with a later time is not a retry.

If an existing project-wide R6 maximum is shorter than 300 seconds, the shorter existing maximum controls.

Any longer window should require a separate explicit policy decision rather than appearing as an implementation constant.

Five minutes is the narrowest straightforward choice because it retains the already accepted duration while moving its start to the point at which pairing can actually occur. The explicit expired-unused restoration route below prevents that tight window from becoming a permanent wedge.

G. Single-use and replay semantics

The protected state should enforce this conceptual lifecycle:

Absent
  -> Available(generation N)
  -> PairingEffectStarted(attempt X)
  -> Consumed(receipt R)

An expired-unused predecessor may take this controlled route:

Available(N, expired, unconsumed)
  -> Superseded(N)
  -> Available(N+1)

Normatively:

There may be at most one live predecessor for the completed fixed-install generation.

The same canonical request retried while the predecessor remains live MUST return the same persisted continuation bytes.

Concurrent creation requests MUST converge on one committed predecessor.

Consumption MUST use a protected CAS.

Concurrent pairing attempts MUST produce at most one successful admission.

A replay after consumption MUST produce a deterministic already-consumed result or the original durable receipt, with no second pairing effect.

A superseded continuation MUST always be rejected.

A changed manifest, mapping, guest identity, fixed-install generation, guest-executor digest, or invalidating state anchor MUST prevent restoration.

A consumed predecessor MUST never be replaced by this corridor.

Replacement of an expired-unused predecessor must be:

Explicitly requested.

Performed only by the same exact producer.

Based on fresh revalidation.

A new generation with new issuance time and signature.

Non-overlapping with the old generation.

Recorded by CAS with the old generation permanently tombstoned.

Replacement must not be scheduled automatically.

The amendment should say “one live predecessor at a time,” not “only one issuance for the lifetime of the install.”

H. Pairing failure and restoration

Independent terminal confirmation must occur before pairing effect admission.

Recommended semantics:

User cancellation or validation failure before PairingEffectStarted leaves the predecessor available until expiry.

At terminal confirmation, CAS from Available to PairingEffectStarted, binding a unique pairing attempt and transcript/request digest.

After that transition, expiry blocks new pairing admission but does not abort recovery of the admitted pairing attempt.

If authoritative observation proves no pairing effect occurred, the exact same attempt may either resume or, under a narrowly defined CAS, return to Available while the original validity window remains open.

If any pairing effect may be externally visible, state MUST NOT be restored to Available merely because the caller disconnected or timed out.

Effect-visible ambiguity must be reconciled by observing the exact pairing state and then:

Finalizing Consumed with a durable receipt, or

Remaining failed closed in an explicit recovery state.

A lost success response returns the committed receipt; it never repeats pairing.

That preserves single use without turning one interrupted transport into a permanently lost pairing opportunity.

3. Should R6 be a new pairing-only authority rather than Stage-1 renewal?

Yes, unequivocally.

The intended authority chain should be:

valid Stage-1 admission
    ->
durable R3 fixed-install EffectStarted
    ->
R3 convergence and Completed receipt
    ->
new R6 pairing-only predecessor
    ->
explicit independent terminal confirmation
    ->
single-use pairing effect

R6 may cryptographically and evidentially bind to:

The Stage-1 admission identifier or digest.

The fixed-install effect identifier.

The fixed-install completion receipt.

The resulting manifest, mapping, guest identity, anchor, and guest-executor digest.

But it must not inherit:

Stage-1 scope.

Stage-1 raw serialization.

Stage-1 validity interval.

Stage-1 ability to mutate guest installation state.

Stage-1 ability to bootstrap services or change networking.

Stage-1 retry authority for unrelated effects.

Calling this a “renewal” would be dangerous because it suggests reopening the original privileged effect scope after its intended expiration. It is instead a new, narrower authority derived from the completed effect receipt.

The related Stage-1 rule must be:

Expiration blocks admission of a new fixed-install effect, but does not revoke deterministic convergence or recovery of an exact fixed-install effect that was durably admitted while Stage-1 was valid.

If the current implementation rechecks Stage-1 expiration before every guest mutation or requires an unexpired Stage-1 at R3 completion or R6 minting, that must be corrected. This is not an extension of Stage-1; it is the ordinary distinction between authority admission and recovery of an already-precommitted effect.

Also inspect when EffectStarted is durably written. If it happens only after a potentially slow VM boot/observation phase, Stage-1 can still expire before admission. The fixed-install effect must be precommitted while Stage-1 is valid and before the first corresponding privileged mutation. Moving the precommit after the slow work would leave the corridor causally broken.

4. Remaining causal contradictions

As presently worded, three contradictions remain.

1. Expired-unused predecessor with no successor route

A five-minute fresh continuation fixes install-time expiry only once. Without explicit CAS supersession of an expired-unused predecessor, a slow human response, lost publication, clock jump, or delayed native evidence permanently wedges pairing.

The narrow explicit replacement transition above is required.

2. Anchor self-invalidation

If the predecessor binds “the current anchor,” creation advances that anchor, and the consumer requires equality with the now-current value, the predecessor becomes stale at the instant it is created.

The contract must distinguish the R3 source anchor from the committed R6 anchor or CAS receipt.

3. Stage-1 checked at the wrong lifecycle point

If the implementation requires a live Stage-1:

Throughout fixed-install convergence,

At R3 completion,

Or at R6 predecessor creation,

then a legitimate slow install remains impossible. Stage-1 must govern admission; protected state and the fixed effect journal govern convergence.

Local-truth dependency: producer/consumer compatibility

The existing R6 consumer must be able to validate the newly produced document using the actual production schema, signature domain, expected SPKI, state lookup, and guest-executor binding. If it only accepts a predecessor created elsewhere, expects the host helper digest in the guest field, or requires the old anchor interpretation, adding a producer alone will still not complete the path.

Once those issues are resolved and fixed-install completion is defined by exact endpoint readiness rather than approximate service state, I see no remaining architecture-level contradiction requiring a pivot.

5. Fixed-install corrections to preserve

The current implementation direction is appropriate, but these corrections must land before R6 creation is enabled.

Descriptor-bound artifacts and staging generations

Keep descriptor-bound adoption of the four externally produced Linux artifacts, with these rules:

Open and validate the source once, without following symlinks.

Hash and transfer bytes from that same descriptor.

Verify exact byte count and digest after guest transfer.

Stage into a root-controlled, generation-specific temporary location.

Never stream directly into the final executable or unit path.

Within one install generation:

Repeating the same role with identical metadata is idempotent.

The same role with a different digest is a conflict.

Across generations:

The same role and final path may have a different digest.

Staging keys must include at least the install generation and artifact role.

Content-addressed blob deduplication, if present, must be separate from the generation-specific adoption record.

A safe conceptual key is:

(install_attempt, install_generation, artifact_role)

with digest and destination stored as checked values, rather than using destination alone as the identity.

Durable atomic publication

For each final file:

Create a unique temp file in the same parent directory or same filesystem.

Stream the exact descriptor-bound bytes.

Verify length, digest, regular-file type, ownership, and mode.

fsync the temp file.

Atomically rename it over the final path.

fsync the containing directory.

Reopen the final path without following symlinks and reverify it.

Only then record that publication step as completed.

On Linux, rename can atomically replace an existing destination, while durable recovery also requires syncing the file and then the containing directory. 
man7.org
+1

Because publication spans multiple artifacts and unit files, the overall install is a journaled transaction rather than one filesystem-atomic operation. On interruption it must either:

Deterministically finish the current generation, or

Restore the prior exact generation and verify that restoration.

If it can prove neither, leave an explicit recovery-required state and do not create R6.

Persist the exact signed predecessor and continuation bytes as well. Do not re-sign on response retry and assume byte identity; retain the exact capsule before exposing it.

Root adoption and rollback

Before changing final paths, record sufficient prior-generation information to restore:

Exact bytes or protected backup location.

Owner, group, mode, and relevant metadata.

Unit enablement and active state.

Socket state and endpoint identity.

Inject failures after every meaningful step. Tests should cover:

Symlink replacement races.

Parent-directory replacement attempts.

Preexisting hard-linked files.

Wrong owner or mode.

Truncated stdin transfer.

Digest mismatch after transfer.

Interruption after file fsync but before rename.

Interruption after rename but before directory fsync.

Failure after unit publication but before daemon reload.

Failure during activation and readiness.

Failure during rollback itself.

Exact same-generation retry.

Different-artifact replacement in a later generation.

systemd convergence and readiness

Before= and After= establish ordering; they do not independently cause another unit to be activated. Requirement/want dependencies and ordering are separate concepts. Therefore a “Before-only” relationship can absolutely produce the identified retry wedge. 
man7.org

The fixed plan must:

Publish the exact service and socket units.

Run daemon reload.

Explicitly converge enablement and activation of the intended units.

Clear only the exact relevant failed unit state when retrying.

Verify the exact socket unit is active and bound to the expected endpoint.

Trigger socket activation where applicable.

Verify the exact service associated with that socket.

Perform a protocol-level endpoint probe and validate the expected response/version.

Reject a stale socket, wrong unit basename, wrong service association, or unrelated process listening on the same path.

Do not treat systemctl is-active alone as readiness. If Type=notify is used, the actual service must send READY=1; otherwise use a service type consistent with the binary and make the endpoint probe authoritative. systemd explicitly treats Type=notify readiness as the service’s notification, not merely successful process execution. 
man7.org
+1

For volatile runtime paths, prefer systemd-owned lifecycle and ownership using the appropriate RuntimeDirectory=, socket ownership, group, and mode directives. An install-time mkdir /run/... is not durable across reboot. systemd’s runtime-directory mechanism is designed to create writable runtime directories with service-associated ownership and lifetime. 
man7.org

Field-specific escaping

Do not use one generic “shell escape” function for unit names, filesystem paths, ExecStart arguments, environment values, and displayed shell commands.

Use distinct typed encoders for at least:

Unit-name or template-instance escaping.

Filesystem-path-to-unit-name escaping.

ExecStart argv serialization.

Environment assignment values.

Human-display shell quoting.

systemd-escape --path has path-specific normalization and rejects remaining .. components; that behavior is not interchangeable with command-line quoting. 
man7.org

Test homes and destinations containing spaces, percent signs, backslashes, quotes, and non-ASCII characters. Run systemd-analyze verify against the generated units in the target Linux environment.

Netfilter rejection

Reject --world-netfilter twice:

In canonical macOS command validation, before VM or privileged effects.

In the privileged fixed-install request validator, as defense in depth.

The error must be explicit and stable. Tests must prove:

No fixed-install state transition.

No guest mutation.

No R6 predecessor.

No silent option dropping.

No recording of a misleading successful configuration.

6. Safest implementation and review order
Phase 1 — Freeze authority before landing implementation

Land or formally approve the narrow authority amendment first.

Include the exact producer operation, R3 precondition, signature domains, two-anchor semantics, 300-second window, expired-unused replacement, and consumption recovery.

Amend the six-path allowlist explicitly if the authority document or new production schema requires another path. Do not treat the proposed amendment as implicit permission to touch an extra file.

Phase 2 — Establish local project truth

Against the exact base:

Bash
BASE=5dc5d98ad76ee5f9758f7bc139e453c47ce46d20

git rev-parse HEAD
git status --short
git diff --name-status "$BASE"
git diff --stat "$BASE"
git diff --check "$BASE"

Trace, using the repository’s actual names:

Stage-1 admission and expiration checks.

The exact point at which R3 becomes EffectStarted.

R3 completion and recovery receipts.

Protected-state anchor derivation and CAS transitions.

Keychain signer and SPKI verification.

Existing R6 predecessor and continuation schemas.

Existing R6 production parser and validator.

Existing R6 consumption and replay state.

Retry locator/capsule persistence.

Host helper identity fields.

Guest artifact digest fields.

Manifest and platform mapping finalization.

Guest identity and home observation.

Artifact staging keys.

All direct writes to final paths.

Unit generation, escaping, activation, and readiness.

Every handling path for --world-netfilter.

All changed paths against the exact allowlist.

Most importantly, trace the entire causal chain in both directions:

canonical installer
  -> dedicated privileged method
  -> protected R3 receipt
  -> R6 creation CAS
  -> continuation serialization
  -> installer output/publication

and:

guest terminal invocation
  -> production continuation parser
  -> signature/state/binding validation
  -> terminal confirmation
  -> consumption CAS
  -> pairing effect
  -> durable result/evidence receipt
Phase 3 — Add focused regressions that fail on the base

The new tests should compile against the base where feasible and fail behaviorally. Prove this in a detached worktree with a test-only patch rather than relying on recollection:

Bash
git worktree add --detach ../r3-r6-base "$BASE"

The focused regression set should cover:

Authority and schema

R6 creation before R3 completion.

Stage-1 expired before R3 admission.

Stage-1 expired after R3 EffectStarted.

Caller-provided selector, action, expiry, digest, anchor, or raw authorization.

Empty canonical R6 fields, one field at a time.

Host Mach-O and guest ELF values swapped.

Wrong guest role, format, architecture, size, or digest.

Wrong signature domain.

Wrong signer SPKI.

Changed manifest, mapping, guest identity, or completed-install receipt.

Fresh R6 time derived from R6 issuance, not Stage-1.

R6 rejected as authority for any non-pairing operation.

macOS --world-netfilter rejection.

CAS, retry, expiry, and replay

Concurrent creation produces one predecessor.

Lost response after predecessor CAS returns the same persisted continuation.

Lost response after continuation persistence returns identical bytes.

Expired-unused predecessor is explicitly superseded.

Old continuation fails after supersession.

No two live generations.

EffectStarted prevents issuance of another predecessor.

Concurrent consumption produces one admitted pairing attempt.

Lost success response returns the original receipt.

Ambiguous effect state does not restore availability.

Clean pre-effect cancellation leaves the predecessor available.

Pairing admitted before expiry may converge after expiry.

Fixed install and lifecycle

Interrupted artifact transfer leaves final files unchanged.

Temp verification failure blocks publication.

Exact later-generation replacement does not collide with staging keys.

Same-generation changed digest is rejected.

Rollback restores exact prior bytes and metadata.

Managed home is exactly observed home plus .substrate.

Deleted /run state is correctly recreated on activation.

Before= without activation dependency fails the regression.

Wrong socket/service association fails readiness.

Active unit with nonfunctional endpoint fails readiness.

Paths with spaces and special characters round-trip through generated units.

Symlink and parent-directory adversarial cases fail safely.

Evidence and non-auto-run

Production serializer/parser/validator round-trips the exact continuation.

Mutation of every authority-bearing field invalidates it.

Unknown or duplicate fields fail.

Continuation contains no raw Stage-1, ticket, or secret.

Installer stops after emitting the continuation.

No pairing subprocess, Lima command, or guest executor invocation occurs from the installer.

Pairing requires a distinct terminal invocation and confirmation.

Phase 4 — Fix R3 before enabling R6

Implement in this order:

Generation-correct artifact staging keys.

Descriptor-bound transfer verification.

Temp-file publication, file fsync, atomic rename, directory fsync.

Transactional root adoption and rollback journal.

Observed-home-derived managed home.

Field-specific unit encoding.

systemd runtime-directory/socket ownership.

Exact activation convergence.

Endpoint-level readiness.

Explicit macOS netfilter rejection.

Do not let R6 creation compile into the production path until R3 completion means all of those checks passed.

Phase 5 — Add the R6 schema and producer

Introduce distinct typed host-code and guest-ELF identities.

Add closed predecessor and continuation schemas.

Add signature-domain separation.

Update actual production validators first.

Implement the dedicated executor operation.

Derive every seed field from protected state/R3 receipt.

Create and sign the predecessor.

Persist its exact bytes or durable capsule.

CAS-create the protected predecessor state.

Create, sign, and durably persist the continuation bound to the committed predecessor state.

Emit only after all persistence succeeds.

Add exact-response retry recovery.

Phase 6 — Add replacement and consumption convergence

Implement:

Idempotent return of the live continuation.

Explicit expired-unused supersession.

One-live-generation invariant.

Available -> PairingEffectStarted -> Consumed.

Effect-visible reconciliation.

Original-receipt recovery.

Permanent rejection of consumed and superseded continuations.

Phase 7 — Validate mechanically

At minimum:

Bash
git diff --check "$BASE"
git diff --name-only "$BASE" | sort -u

Verify exact set equality with the six-path allowlist or its explicitly approved amendment.

For host shell changes:

Bash
/bin/bash --version
git diff --name-only --diff-filter=ACMRT "$BASE" -- '*.sh' |
while IFS= read -r file; do
    [ -n "$file" ] || continue
    /bin/bash -n "$file" || exit 1
done

For Rust, use the repository’s focused packages first and then the required workspace scope:

Bash
cargo fmt --all -- --check
cargo check --locked --offline --workspace --all-targets
cargo test --locked --offline --workspace

Run clippy if it is already a required project gate:

Bash
cargo clippy --locked --offline --workspace --all-targets -- -D warnings

Use the repository’s canonical four-role build command for:

aarch64-unknown-linux-gnu

and verify the produced objects are actually AArch64 ELF files before packaging or manifest generation. Do not substitute a host build or merely trust output filenames.

Inside the compatible Linux validation environment, run:

Bash
systemd-analyze verify <all generated service and socket units>

Then execute the interruption, retry, replacement, endpoint, and adversarial integration suites.

Phase 8 — Independent causal reviews

After all automated validation, perform three fresh reviews in this order:

Authority/security

Exact producer identity.

Dedicated operation only.

Domain separation.

No caller authority fields.

Stage-1 admission semantics.

Two-anchor correctness.

Guest ELF binding.

TTL, supersession, and single use.

Explicit netfilter rejection.

Lifecycle/convergence

R3 completion witness.

Atomic publication and durability.

Generation replacement.

Rollback.

systemd activation and /run.

Endpoint readiness.

R6 create/consume interruption recovery.

Effect-visible ambiguity.

Evidence/allowlist

Production validator round-trip.

Sanitization.

No secret or raw authorization.

No automatic pairing.

Exact changed-path set.

Exact base, reviewed head, and diff fingerprints.

Only after those reviews should the separate native installer/pairing/evidence run occur.

Principal pitfalls to block

The highest-risk mistakes are:

Calling R6 a Stage-1 “renewal.”

Requiring Stage-1 to remain live after an effect was precommitted.

Using one ambiguous anchor field on both sides of a CAS.

Allowing a caller to supply any R6 binding or lifetime.

Authorizing a generic XPC action dispatcher.

Using one untyped digest structure for Mach-O and ELF identities.

Re-signing on retry instead of recovering exact persisted bytes.

Treating predecessor “single use” as “only one issuance ever.”

Restoring availability after an effect-visible ambiguous failure.

Checking service active state without proving the socket and endpoint.

Assuming Before= activates another unit.

Creating /run state only during installation.

Using shell quoting as systemd unit-name or ExecStart escaping.

Writing directly to final executable or unit paths.

Letting a staging destination key collide across generations.

Accepting and silently ignoring --world-netfilter.

Emitting a continuation and then invoking pairing from the same installer process.

With those corrections, the safest path is to keep the current architecture, narrowly amend authority, make R3 completion genuinely durable and observable, and then add a separate R6 pairing-only state transition with explicit expiry restoration and single-use consumption.
```
