# R3 macOS retirement planning-hardening ChatGPT Pro approach review

- **Date:** 2026-08-13
- **Task/spec:** Harden the bounded R3 macOS evidence-retirement and exact-orphan-recovery planning foundation before any implementation.
- **Review mode:** Approach review
- **Model:** GPT-5.6 Sol
- **Effort:** Pro
- **Conversation:** https://chatgpt.com/c/6a7e0717-eb6c-83ea-ae8b-68c5ec48d2ee
- **Landed checkpoint reviewed:** `879a8c680ba14cd93b79ba2c1a2ed598e9687c90` / tree `b201ce8f058b438ec6e8665c84b09abf0388404f`
- **Prompt SHA-256:** `0cefe2c45280fa408700722061ab99bf788b34aeaca446fb9b2642f6fe6b363f`
- **Answer SHA-256:** `ee6cbca8d7b85696cc9834575343c12d2fb53296435210211a9c4bbbdbcf125c`

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Local reconciliation

The response independently confirms the local `ADJUST`, not `PIVOT`, assessment. Candidate D remains conditionally sound for the prospective lane and Candidate C remains the required operational order, but the current planning foundation is not yet acceptable as implementation authority.

The following response additions are suitable for a bounded doc-only correction proposal and must remain unaccepted until the returned patch is locally validated and explicitly approved:

- make external receipt durability, externally durable acknowledgement, protected-CAS binding, and surviving-successor acceptance mandatory predecessors to every destructive transition;
- split both guest and host quiescence, receipt, acknowledgement, handoff, removal, and parity states;
- distinguish publisher-signed pre-removal receipts from post-removal parity proofs;
- make the finalizer a non-authoritative, dual-authorized executor of one frozen contiguous terminal host-removal suffix;
- require the finalizer to have no private signing key unless a later larger amendment supplies a complete retirement protocol for that key;
- close the fixed caller, mutual attestation, canonical FD3 framing, EOF, live-CAS verification, journal, replay, disconnect, and fixed-target rules;
- make delete-only noninteractive authority for the exact prospective finalizer identity a mandatory capability gate and the pivot trigger for only the terminal key-deletion mechanism;
- strengthen the separately authorized disposable Keychain experiment and preserve the prospective/recovery authority separation;
- preserve the hard no-effects state until the corrected planning and authoritative contract changes are explicitly accepted.

No production implementation, live recovery, Keychain mutation, evidence task, or MAC-CLOSEOUT is authorized by this advisory record.

## Prompt sent

~~~text
We are already working on this project task.

Task/spec: Harden the bounded R3 macOS evidence-retirement and exact-orphan-recovery planning foundation before any implementation. The desired result is a corrected, internally consistent, security-reviewable planning and authoritative-contract amendment—not production code. Candidate D is the proposed prospective structure: split/amend terminal authority through an external evidence finalizer. Candidate C is the proposed operational order: recover and prove parity for the preserved failed attempt before prospective implementation.

Current approach: Keep prospective retirement and orphan recovery separate in authority, schemas, executables, journals, and routes. For prospective retirement, introduce V2 types and a narrowly capable, root-owned, pre-baseline external evidence finalizer. For recovery, first run a separately authorized disposable Keychain surrogate experiment because SecurityAgent appeared despite query-level kSecUseAuthenticationUIFail. Then choose either a proven creator-authorized noninteractive route or, only with explicit user approval, one preplanned operator-authorized prompt. Repository-derived inference: there is no suitable landed post-Keychain finalizer, and the retained creator-bound helper is not currently callable for signer retirement.

Project sources: No ChatGPT Project source set is configured for this consultation. Treat the facts and exact contract excerpts below as the complete review boundary; do not infer excluded repository content.

Evidence, blockers, constraints, and validation: The current 970-line advisory planning foundation contains state tables, component/holdback ordering, schema inventory, kill matrix, deferred work, stop states, and serial task decomposition, but it is unaccepted and has not amended the frozen authoritative R3 contract. A local review found the following issues that must be independently challenged rather than assumed correct:

1. Authoritative invariant: the still-authorized publisher signs the external receipt; the external evidence harness persists and hashes it and durably acknowledges it; protected CAS binds that acknowledgement; only then may publisher evidence and nonterminal teardown state be deleted.

2. Proposed host contradiction: the current host state machine says HostNonTerminalPrepared -> HostNonTerminalEffects -> HostNonTerminalReceiptDurable -> HostNonTerminalAcknowledged, and the removal list deletes host nonterminal Keychain records before writing/acknowledging the publisher host receipt. This appears to violate the authoritative invariant by allowing destructive effects before externally durable authority.

3. Proposed guest contradiction: one state description makes GuestEffectsApplied include endpoint stop/disable and leaf/parent removal before GuestReceiptDurable and GuestAcknowledged. A separate removal-order section instead says stop/disable, persist receipt, durably acknowledge and CAS-bind, then remove components. The latter appears consistent with the contract. The state machine likely needs distinct GuestQuiesced, receipt-durable, acknowledgement-durable/CAS-bound, GuestRemoving, and parity-proven phases.

4. Proposed finalizer is underspecified. The contract must close: its fixed caller; privilege transition; FD3 peer and code-identity attestation before decoding; exact canonical FD3 frame versus fixed artifact reads; EOF rules; finalizer signing-key provisioning, opening, use, and retirement; journal location and CAS owner that remain available after all deletion effects; replay/idempotency; and retry if the invoking coordinator disappears.

5. Recovery remains blocked pending a separately approved disposable experiment. That experiment must compare query-level kSecUseAuthenticationUIFail with the legacy process-level SecKeychainSetUserInteractionAllowed(false), prove whether any SecurityAgent UI can still appear, avoid approving credentials, use disposable identities only, and restore exact baseline. Its result—not guesswork—must determine whether recovery can be noninteractive or requires one explicit, expected, operator-authorized prompt.

6. Non-negotiable separation: prospective retirement must not become a general orphan-recovery route; recovery authority must not be smuggled into the finalizer; the finalizer must not become a general shell, installer, uninstaller, upgrade mechanism, or broad Keychain mutation API. Broader product uninstall, upgrade/replacement, multi-install ownership, last-claim retirement, and general orphan recovery remain deferred.

7. No production implementation is authorized. The immediate requested output is a keep/adjust/pivot review and the smallest doc-only correction set needed to make the plan suitable for explicit acceptance and a later authoritative contract amendment. The review must identify any additional trust-transition, crash-window, acknowledgement, journal, identity, replay, or evidence-integrity defect. It should provide corrected host and guest state transitions, a closed finalizer invocation/attestation contract, experiment acceptance criteria, serial gates, stop states, and a recommendation on whether Candidate D plus Candidate C remains sound.

Validation expectation: every destructive transition must have a durable predecessor state and deterministic retry classification; externally durable receipt/acknowledgement must precede deletion; parity must prove the exact evidence scope returned to baseline; no UI prompt may be treated as expected unless separately and explicitly approved; the docs-only correction must not authorize implementation or live recovery.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; tradeoffs; what to keep/change first; inspections; validation; and pitfalls.
~~~

## ChatGPT Pro response

~~~text
Recommendation: ADJUST

Candidate D plus Candidate C remains the best direction, but the current planning foundation is not yet acceptable. The host ordering violates the stated authoritative invariant, the guest ordering is internally contradictory, and the proposed finalizer does not yet have a closed authority or recovery model.

The necessary adjustment is structural:

Prospective retirement remains completely separate from orphan recovery.

The external finalizer becomes a non-authoritative, dual-authorized executor, not a new source of retirement authority.

The finalizer must durably accept the complete authority chain before any host component needed for signing, retry, or handoff is deleted.

After acceptance, the finalizer exclusively owns one contiguous terminal host-removal suffix. It should not merely delete the last key after another component has already self-destructed.

The failed orphan is preserved, experimentally de-risked, recovered under separate one-time authority, and brought to exact parity before prospective production implementation begins.

No implementation, route installation, live recovery, or Keychain mutation is authorized by the corrected planning document.

Candidate D should be treated as conditionally sound: it depends on proving that the exact finalizer identity can delete a prospectively created publisher key without interaction and without acquiring broader signing authority. Failure to prove that is the specific pivot trigger; it is not a reason to merge orphan recovery into the finalizer.

What to keep, what to change, and the pivot trigger
Decision	Treatment
Separate prospective retirement and orphan recovery	Keep
V2 prospective schemas with strict domain separation from recovery	Keep
External, root-owned, pre-baseline finalizer	Keep, but narrow and fully specify
Experiment before touching the preserved orphan	Keep and strengthen
Recover and prove parity before prospective implementation	Keep
Host destructive effects before receipt durability and acknowledgement binding	Remove
Guest leaf/parent deletion before external acknowledgement	Remove
A finalizer that only deletes the last signer after the publisher has already removed its own retry machinery	Change to a contiguous finalizer-owned terminal suffix
A finalizer with an unspecified private signing key	Do not provision one unless a separately justified requirement emerges
General or reusable orphan-recovery behavior in the finalizer	Explicitly prohibit
Candidate D if delete-only noninteractive Keychain authority cannot be demonstrated	Pivot only the terminal key-deletion mechanism

The strongest part of the present approach is the recognition that prospective and retrospective authority are different. The preserved orphan cannot retroactively acquire a bootstrap precommit, and the prospective finalizer must not be weakened to accommodate that incident.

Corrected authority model

The normative prospective sequence should be:

precommit → quiesce → publisher-signed pre-removal receipt → external durable persistence → externally durable acknowledgement → protected-CAS acknowledgement binding → durable successor handoff → destructive effects → exact parity observation → external terminal evidence durability → terminal completion

Three invariants should be added at the beginning of the planning foundation:

1. External-durability invariant

No publisher evidence, protected teardown state, component parent, service registration, or lifecycle Keychain record may be deleted until:

the still-authorized publisher has signed the exact pre-removal receipt;

the harness has durably persisted and re-read the exact receipt;

the harness has signed and durably persisted its acknowledgement;

the surviving protected CAS has verified and bound that acknowledgement.

The acknowledgement itself must be durable before it is delivered for CAS binding. An in-memory acknowledgement of a durable receipt is insufficient.

2. Successor-acceptance invariant

An actor may not delete any state needed for its own retry, signing, journaling, or successor invocation until the successor has durably accepted all information needed to finish independently.

This applies twice:

Before guest self-removal, a surviving host-side journal/CAS must accept the guest capsule.

Before host self-removal, the external finalizer journal must accept the host capsule.

3. Receipt-chronology invariant

The publisher-signed artifact created before deletion must not claim to contain post-deletion observations.

The V2 model should distinguish:

a publisher-signed pre-removal receipt, containing authorization, before/quiesced observations, exact component identities, and the frozen removal plan; and

a terminal parity proof, produced after deletion and signed or acknowledged by the precommitted external harness.

Calling both artifacts simply “receipt” invites chronology errors. Suggested normative names are:

GuestPublisherTestRetirementPreRemovalReceiptV2

GuestPublisherTestRetirementReceiptDurabilityAcknowledgementV2

GuestPublisherTestRetirementParityProofV2

PublisherTestRetirementPreRemovalReceiptV2

PublisherTestRetirementReceiptDurabilityAcknowledgementV2

PublisherTestRetirementFinalizationRequestV2

PublisherTestRetirementParityProofV2

PublisherTestRetirementTerminalAcknowledgementV2

The exact names can follow repository conventions, but the semantic distinction must be normative.

Corrected guest state machine

The present GuestEffectsApplied concept is too broad. It mixes quiescence, deletion, receipt production, and parity.

A corrected guest sequence is:

State	Durable predicate and permitted behavior
GuestPrepared	Authorization, exact component DAG, baseline hash, component identities, counters, expected parent identity, and before-observations are CAS-bound. No effect has occurred.
GuestQuiescePrepared	The exact stop/disable operation and its expected observations are durably journaled before the operation.
GuestQuiesced	Ordinary dispatch is stopped or disabled, but the retirement signing capability remains available. No leaf, parent, protected state, or signer has been deleted.
GuestPreRemovalReceiptSigned	The still-authorized guest publisher signs a closed V2 receipt covering the exact authority, component DAG, before/quiesced observations, and frozen removal plan.
GuestReceiptExternallyDurable	The harness has verified canonical bytes/signature, atomically persisted the exact receipt, fsynced the file and parent directory, reopened it without following links, and rehashed it.
GuestAcknowledgementExternallyDurable	The harness has created, signed, and durably persisted an acknowledgement binding the receipt hash, scope, attempt, baseline, sequence, and retained evidence identity. Only then is it transmitted.
GuestAcknowledgementCASBound	The guest protected CAS verifies the harness signature and advances by expected generation to bind the exact receipt and acknowledgement hashes. Deletion is still prohibited until a surviving owner accepts the capsule.
GuestHandoffHostBound	The host-side protected CAS or another explicitly named surviving host journal independently verifies and durably copies the complete guest authority capsule, current guest-CAS head, exact effect plan, and retry state.
GuestRemoving	A host-surviving executor owns the destructive suffix. Each effect uses prepared → applied → observed records. Leaf/parent/component removal starts only here.
GuestRemoved	Every authorized guest component is observed absent. This is not yet parity.
GuestParityExternallyDurable	The harness compares the exact guest scope against the pre-bootstrap baseline using the same canonical observer, persists the result, and signs its terminal acknowledgement.
GuestParityHostBound	The surviving host CAS binds the guest parity proof and acknowledgement. Only this state permits host retirement to begin.

Two important constraints follow:

Quiesced must not mean retirement authority is gone. If stopping the endpoint destroys the only route capable of signing the pre-removal receipt, the order is still unsound. The plan must identify a retained in-process or already-open retirement channel, or move the signing operation before the destructive form of stop.

The guest endpoint being removed must not own the only recovery journal. A host-surviving actor must accept the capsule before guest parent or protected state deletion. If the guest and host CAS are actually the same surviving CAS, the document may collapse the two binding states, but it must prove that lifetime explicitly.

Corrected host state machine

The current host transition through HostNonTerminalEffects before external durability should be removed, not merely renamed.

The corrected host sequence should be:

State	Durable predicate and permitted behavior
HostPrepared	Guest parity is host-CAS-bound. Host authorization, baseline, component identities, signer identity, exact component DAG, finalizer identity, and before-observations are durably bound.
HostQuiescePrepared	The intended restriction of ordinary publisher activity is journaled before effect.
HostQuiesced	New ordinary lifecycle operations are rejected, but the exact retirement-signing/handoff capability remains available. No protected state, service registration, evidence record, or signer has been deleted.
HostPreRemovalReceiptSigned	The still-authorized publisher signs the exact V2 pre-removal receipt and finalization plan.
HostReceiptExternallyDurable	The harness atomically persists, fsyncs, reopens, and hashes the exact publisher receipt.
HostAcknowledgementExternallyDurable	The harness signs and durably persists its acknowledgement before delivering it.
HostAcknowledgementCASBound	The protected host CAS verifies and binds the receipt and acknowledgement. This is the earliest point at which deletion could become authorized, but the host still may not self-remove.
HostFinalizationRequestFrozen	One canonical request digest is frozen. It includes all authority-bearing bytes, exact component identities, signer identity, CAS head, guest terminal proof, baseline, finalizer identity, caller identity, and effect order.
HostFinalizerAccepted	The finalizer has attested the caller, independently read and verified the live protected CAS, copied all required bytes into its own surviving journal, claimed the scope/request digest, and fsynced the journal and directory. This is the irreversible commit point.
HostRemoving	The finalizer exclusively owns all remaining host destructive effects. The publisher coordinator performs no further mutation.
HostEffectsComplete	The exact service state, protected records, files, and signer specified by the request are absent. The signer is the final publisher-owned authority-bearing component removed.
HostParityExternallyDurable	The harness proves the exact publisher evidence scope matches baseline and durably persists/signs the terminal proof.
HostTerminalAcknowledgementBound	The finalizer’s surviving journal binds the terminal parity proof and harness acknowledgement.
HostComplete	The request is terminal and immutable. Replays return the same result and cannot execute a new deletion plan.

The key correction is that HostFinalizerAccepted precedes every host deletion capable of stranding the operation.

Allowing the publisher to remove service registration or protected records and only then asking the finalizer to delete the signer creates exactly the precommit-less orphan pattern the prospective design is meant to prevent.

The finalizer can remain narrow while owning the entire suffix: its target set is frozen, dual-authorized, live-CAS-verified, and limited to fixed component classes. It does not become a general uninstaller merely because it owns several exact deletions rather than one.

Closed finalizer invocation and attestation contract

The planning amendment should choose one exact invocation model. Given the need to retry after the publisher coordinator disappears, the cleanest model is a pre-baseline, root-owned, on-demand launchd finalizer endpoint that is external evidence infrastructure rather than publisher product state.

A direct child launched only by the publisher is insufficient unless another surviving mechanism can restart and rejoin it after both publisher and child fail.

Placement and privilege transition

The finalizer executable, launchd definition, endpoint, and immutable configuration are installed by the evidence setup before baseline and before publisher bootstrap.

The product bootstrap, product installer, and product uninstaller cannot install, modify, update, or remove it.

It runs as root only through that fixed on-demand launchd definition.

Its binary identity, code-signing requirement, executable hash, launchd-definition hash, protocol version, endpoint identity, and journal-root identity are included in the prospective bootstrap commitment.

Any change to those identities during an attempt is a hard stop.

It has no network route, shell facility, plugin loading, arbitrary subprocess route, arbitrary environment authority, or generic filesystem/Keychain API.

Fixed caller

The caller is the exact precommitted external evidence coordinator binary—not:

a shell script;

the product installer;

the publisher service;

an arbitrary root process;

a generic test harness;

an orphan-recovery executable.

The contract should bind at least:

effective UID and stable user identity;

executable vnode/path identity;

code-signing requirement and executable hash;

harness public key;

attempt/scope identifier;

finalizer endpoint and protocol version.

The endpoint’s filesystem permissions are only a preliminary filter. Cryptographic authorization, live-CAS verification, and peer identity remain mandatory.

Mutual peer attestation

Before decoding request bytes, the finalizer obtains the peer credentials and stable process identity from the accepted local connection and verifies:

exact UID/user identity;

PID and process start identity or audit token;

executable identity;

code-signing requirement;

committed executable hash.

It should revalidate peer identity after receiving the full frame and EOF but before durable acceptance, preventing an executable change or PID-reuse race.

The harness performs the symmetric check on the finalizer endpoint before sending authority-bearing bytes and before accepting the response.

The accepted connection may then be normalized to FD3, with unrelated inherited descriptors closed, a fixed working directory, sanitized environment, and restrictive umask.

Exact frame and EOF rules

The amendment should define one frame and prohibit mixed path-based ingestion.

A suitable closed framing rule is:

request-frame =
    uint64_be(canonical_document_byte_length)
    || canonical_finalization_request_v2_bytes

With:

a fixed normative maximum, for example 4,194,304 bytes;

one request frame only;

no BOM, newline terminator, second frame, or trailing bytes;

canonical serialization using the existing normative R3 serializer;

unknown and duplicate fields rejected;

caller write-half shutdown after the frame;

finalizer acceptance prohibited until request EOF is observed.

The response uses the same framing rule. The finalizer writes exactly one response and then shuts down its write half. The caller reads through EOF.

All authority-bearing documents are carried inline as exact canonical bytes. The finalizer does not follow caller-supplied artifact paths.

The only local reads permitted are:

its fixed root-owned journal;

the live protected state at fixed, typed identities;

exact effect targets derived from the signed component manifest and independently constrained by the finalizer’s built-in component allowlist.

No field may contain a command, glob, arbitrary Keychain query, unrestricted launchd label, or unrestricted path.

Acceptance checks

Before HostFinalizerAccepted, the finalizer independently verifies:

finalizer identity matches the original precommit;

caller identity matches the original precommit;

publisher receipt is canonical and correctly signed by the exact surviving publisher signer;

harness acknowledgement is canonical and correctly signed;

receipt and acknowledgement are durably sequenced;

live protected CAS contains the expected acknowledgement binding and generation;

guest terminal proof is host-CAS-bound;

machine/build/scope/attempt/component identities match;

no component identity has drifted;

no request for the scope was previously accepted under a different digest;

the authorization was unexpired at first acceptance;

exact targets remain within the disposable evidence scope;

no multi-install, replacement, or extra-claim condition exists.

The finalizer must read the live CAS itself. A caller-provided serialization claiming that CAS was bound is not enough.

Surviving journal and CAS ownership

At acceptance, the finalizer writes a root-owned journal entry containing:

the complete canonical request bytes, not only hashes;

the live protected-CAS checkpoint;

the request digest;

all authority artifact hashes and exact bytes;

the frozen component/effect plan;

exact target identities;

caller/finalizer attestation results;

the journal predecessor head and new generation.

The journal root must:

be one literal path named by the amendment;

be outside the publisher deletion scope;

be included in the pre-baseline evidence-infrastructure manifest;

reject symlinks and non-root ownership;

use exclusive per-scope locking;

use atomic replacement or append-only generations;

fsync the record and containing directory;

remain readable and resumable after every publisher component is gone.

The trust transition is therefore:

publisher protected CAS
    -> finalizer durable acceptance journal
    -> finalizer terminal journal

The finalizer must not delete the publisher CAS until the second state is durable.

Finalizer signing key

The recommended closure is: the finalizer has no mutable private signing key.

Its role is execution, not authorization.

Authority and portable evidence already come from:

the publisher’s pre-removal signature;

the harness’s durable acknowledgement;

the protected-CAS binding;

the harness’s terminal parity signature;

mutual code-identity attestation;

the finalizer’s root-owned hash-chained journal.

The harness signs the exact finalizer response and terminal journal head when it persists the terminal evidence. This avoids adding a third authority and avoids creating another “last signing key” that itself requires a retirement protocol.

The contract should explicitly state:

finalizer private signing-key provisioning: prohibited
finalizer private signing-key opening: prohibited
finalizer private signing-key use: prohibited
finalizer private signing-key retirement: not applicable

If an independent finalizer signature is later deemed mandatory, that is a larger amendment: it would require a precommitted per-attempt key, external durability and acknowledgement of its final signed record, CAS binding, key deletion, and a separate proof of key absence. It should not be slipped into the current design.

Keychain authority for deleting the publisher signer

Root ownership alone does not prove noninteractive Keychain authority.

Prospective bootstrap must bind an exact access-control posture under which the committed finalizer identity can delete the exact publisher signer without:

interactive UI;

signing with the key;

exporting it;

accessing another key;

changing its access control.

That delete-only property is a mandatory prospective capability gate. The finalizer must disable process-level interaction before any Keychain call and also apply query-level UI failure behavior.

If Security.framework cannot provide or demonstrate that narrow capability, Candidate D’s key-deletion mechanism must pivot. It must not silently grant the finalizer general use of the publisher signing key.

Replay, retries, and caller disappearance

The idempotency identity should be at least:

(scope_id, attempt_id, reservation_id, finalization_request_sha256)

Rules:

The first fully verified request may atomically claim the scope.

The same request digest rejoins and returns current or terminal status.

A different digest for the same scope is a permanent conflict.

Authorization expiry is checked at first acceptance; an accepted request may resume after expiry.

There is no cancellation operation after acceptance.

A caller disconnect before durable acceptance causes no destructive effect.

A caller disconnect after durable acceptance does not cancel execution.

If the finalizer crashes after acceptance, the same caller/request reactivates it and it resumes from the journal.

A status request is not an open-ended query; it is the same authenticated request digest.

Completed requests return the same terminal result and never inspect or delete newly created matching objects.

For each effect, the journal records:

EffectPrepared -> effect invocation -> EffectObserved

On restart:

target still present with the exact identity: retry the exact effect;

target absent after EffectPrepared: classify as effect-visible ambiguity, record that classification, and continue only when absence is a monotonic safe result;

target replaced, multiplied, or identity-drifted: stop;

absence with no durable prepared record: stop rather than infer that the finalizer performed it.

Recovery experiment: required corrections and acceptance criteria

Candidate C is sound only when “recover first” begins with forensic preservation and a rollback-safe experiment. The experiment must not risk creating a second unrecoverable System-Keychain orphan.

Safety prerequisites

Before any surrogate is created:

The preserved failed attempt’s evidence is externally persisted and hashed, including the surviving key’s public identity, exact tag/label attributes, machine and OS build, absent service/generic state, and the failed attempt artifacts.

The experiment occurs in a rollback-capable disposable native-macOS environment. Lima cannot reproduce System Keychain or SecurityAgent behavior.

The rollback mechanism is capable of restoring the relevant Keychain and host state exactly without approving a prompt.

No production scope ID, key tag, account, label, or artifact path is used.

The surrogate matches the live key in every material respect needed for transferability:

permanent software P-256 key;

explicit System Keychain;

same effective privilege;

same creation/access-control posture;

same creator or intended recovery code identity;

same query structure;

same GUI/login-session posture;

same macOS build.

Each arm executes in a fresh process because process-level interaction policy is process scoped.

SecKeychainSetUserInteractionAllowed(false) is invoked before opening the Keychain or making any Security-framework call in the process-level arm.

If rollback cannot be guaranteed, the System-Keychain portion of the experiment must not run.

Required matrix
Arm	Operation	Acceptance
E1 QueryOnly	Exact delete with query-level kSecUseAuthenticationUIFail, without process-level suppression	Record success or closed failure. Any SecurityAgent activation is a failed noninteractive result. Never interact.
E2 ProcessAndQuery	Fresh process sets SecKeychainSetUserInteractionAllowed(false) before any Security call, then performs the same exact delete with query-level failure	Exact surrogate is deleted, absence verified, and no UI is observed.
E3 WrongIdentity	Wrong code identity or deliberately invalid authorization attempts the exact operation with both suppression mechanisms	Must fail closed without UI and without changing the surrogate.
E4 RetryAbsent	The authorized identity repeats the delete after verified absence	Deterministic item-absent/idempotent classification, no UI.
E5 ProspectiveFinalizer	Separately labeled prospective test: a signer created with the planned finalizer access control is deleted by the exact finalizer identity	No UI; exact deletion only; finalizer cannot sign, export, or mutate access control. This result is not evidence for orphan recovery.

The test should instrument at least:

every Security-framework call and returned status;

SecurityAgent process activation;

visible SecurityAgent window creation;

relevant security/authentication event logs;

a bounded post-operation settle period for delayed UI;

exact before/after Keychain observations.

No credential, Touch ID, password, “Allow,” or “Always Allow” action is permitted during this experiment.

Noninteractive recovery acceptance

A creator-authorized noninteractive recovery route is proven only if:

E2 succeeds repeatedly from fresh processes;

zero SecurityAgent process/window/prompt evidence occurs;

the exact creator/recovery identity and access-control posture match the preserved key;

wrong identity fails without UI;

retry is deterministic;

every surrogate and experiment artifact is absent afterward;

the environment returns to its exact baseline;

there was no executable, OS-build, access-control, or privilege drift between experiment and planned live recovery.

A query-level-only failure does not itself block recovery if the process-level-plus-query arm succeeds. It does mean both controls become mandatory.

Prompt fallback

If process-level plus query-level suppression does not prove a noninteractive path, recovery remains stopped until a separate operator authorization explicitly approves one expected prompt.

That plan must bind:

exact surviving key identity;

exact executable/code identity causing the prompt;

exact operation;

exact machine/build;

one invocation;

one expected prompt;

expected prompt wording and code identity;

no persistent ACL change;

no “Always Allow”;

no retries after cancellation, mismatch, or second prompt;

no credential recording;

external evidence of the prompt event and result.

An unexpected prompt in a noninteractive route is never automatically converted into the approved prompt path.

The experiment result chooses between the two recovery branches. It does not itself authorize the live operation.

Serial gates

The cleanest governance structure is one non-authoritative umbrella planning item with separately accepted child tasks. It should not be one combined authority-bearing implementation task.

Gate	Requirement	What remains prohibited
G0 Freeze	Current planning foundation marked unaccepted; live orphan preserved	All code, routes, live Keychain mutation
G1 Planning correction	Corrected invariants, state machines, finalizer contract, experiment plan, and stop states explicitly accepted	Implementation and live recovery
G2 Surrogate experiment authorization	Separate bounded experiment authority approved	Live orphan mutation
G3 Experiment closure	Experiment completed, exact baseline restored, result externally persisted	Any recovery route not selected by evidence
G4 Recovery-specific authorization	Either exact creator-authorized noninteractive route or one prompt route is separately accepted	Prospective finalizer use; general recovery
G5 Preserved orphan recovery	One live recovery completes and exact host parity evidence is externally durable	Prospective production implementation
G6 Prospective authoritative amendment	Candidate D amendment is finalized with actual experiment conclusions and all literal identities/constants	Production implementation until explicit implementation task
G7 Implementation authorization	A separate implementation task is opened and accepted	Anything outside exact prospective V2 scope

The prospective amendment may be drafted and reviewed while recovery is proceeding, but no prospective executable work should begin before G5.

Required stop states

The planning document should use closed stop classifications rather than free-form failures.

SafePreAcceptanceStop

Used before a successor has durably accepted authority. No deletion has occurred. Exact retry is permitted after the cause is corrected.

Examples:

external receipt or acknowledgement not durable;

CAS generation conflict;

finalizer caller mismatch;

incomplete frame or missing EOF;

authorization expired before first acceptance;

finalizer journal unavailable.

RejoinAcceptedRequest

The successor journal contains the accepted request digest. No new authority is needed. Only the same digest may resume.

Examples:

caller disappeared after acceptance;

finalizer process crashed;

response was lost after signer deletion;

terminal acknowledgement has not yet been delivered.

IdentityOrAuthorityStop

No further destructive effects.

Examples:

different request digest for the scope;

signer SPKI/tag/label drift;

multiple matching Keychain items;

finalizer binary or launchd-definition drift;

wrong caller;

receipt/acknowledgement/CAS hash mismatch;

V1/V2 downgrade attempt;

unknown schema field.

InteractionStop

No further Keychain call in the process.

Examples:

SecurityAgent appears in an unapproved route;

unexpected authentication UI;

more than one prompt;

different prompt identity or wording;

persistent authorization option required.

TerminalParityFailure

All further cleanup is prohibited even if additional removal appears likely to make the machine “look clean.”

Examples:

an unlisted residual remains after signer deletion;

a preexisting baseline item is missing;

finalizer or evidence infrastructure changed unexpectedly;

exact absence cannot be proven.

This state preserves journals and evidence. The finalizer must not broaden its target set or invoke orphan recovery to chase parity.

ExperimentBaselineFailure

No live recovery conclusion may be drawn.

Examples:

surrogate remains;

SecurityAgent state cannot be accounted for;

rollback was incomplete;

experiment and live identity/build/access-control are not materially equivalent.

Smallest doc-only correction set

The current 970-line planning document does not need a wholesale rewrite. Replace or add these bounded sections:

Normative invariants and authority-role matrix

Publisher authorizes.

Harness persists, acknowledges, and proves parity.

Protected CAS binds.

Guest-surviving host coordinator accepts guest handoff.

External finalizer executes the host terminal suffix.

Recovery is an incident-specific authority lane.

Guest state table

Replace GuestEffectsApplied with separate quiescence, receipt, acknowledgement, surviving handoff, removal, and parity states.

Host state table

Remove destructive HostNonTerminalEffects before receipt/acknowledgement.

Add HostFinalizerAccepted.

Move all host destructive effects into one finalizer-owned suffix.

V2 schema inventory

Separate pre-removal receipts from terminal parity proofs.

Add finalization request, acceptance/journal record, parity proof, and terminal acknowledgement.

Give recovery schemas separate names, route magic, signature domains, journal roots, and validators.

Explicitly prohibit V1 fallback and cross-route parsing.

Closed finalizer contract

Fixed caller, privilege route, mutual attestation, exact FD3 frame, EOF rules, live-CAS verification, root journal, idempotency, disconnect behavior, and no signing key.

Keychain experiment and capability gates

Separate creator-recovery and prospective-finalizer conclusions.

Add rollback requirements and exact baseline restoration.

Kill matrix and stop-state table

Add failure points before/after every durability boundary and every destructive effect.

Non-authorization statement

State that the document does not authorize compiling, installing, registering, invoking, or testing the finalizer; modifying the live signer; or recovering the preserved orphan.

After explicit acceptance, the smallest authoritative amendment should affect only:

04-contracts-and-gates.md: normative authority, V2 contracts, finalizer boundary, separation, and no-implementation gate.

03-phase-slice-map.md: Candidate C ordering and the serial gates.

05-debug-regression-ledger.md: the observed SecurityAgent event, the preserved orphan, experiment results when available, and eventual parity receipt hash.

The ledger should record observations, not speculative conclusions.

Inspections and validation

Before the planning correction is accepted, inspect the following:

Inspection	Required pass condition
Destructive-edge audit	Every deletion edge crosses external receipt durability, durable acknowledgement, CAS binding, and a surviving successor acceptance.
Guest executor lifetime	The actor and journal responsible for guest deletion survive guest endpoint and parent removal.
Host executor lifetime	The finalizer route and journal survive every host deletion.
Quiescence semantics	Quiescence blocks ordinary work without destroying receipt-signing or handoff authority.
Acknowledgement durability	The acknowledgement file itself is fsynced before CAS binding, not merely the receipt.
CAS bridge	Guest CAS transfers to surviving host CAS; host CAS transfers to finalizer journal before either is deleted.
Finalizer authority	Requires publisher signature, harness signature, and live CAS; cannot originate authority.
Finalizer input surface	No arbitrary paths, commands, launchd labels, Keychain predicates, or environment-derived authority.
Keychain capability	Finalizer delete-only, noninteractive authority is an explicit proof gate; root is not assumed sufficient.
Replay	Same digest rejoins; conflicting digest stops; completed scope cannot be reused.
Baseline scope	Publisher evidence, retained harness artifacts, finalizer infrastructure, and finalizer journal are separately and explicitly accounted for.
Schema separation	Prospective and recovery types have different domains, parsers, executables, journals, and routes.
Terminal mismatch	Parity failure stops rather than widening cleanup.

The future fault-injection matrix should include at least:

harness death after receipt fsync but before acknowledgement creation;

harness death after acknowledgement fsync but before CAS delivery;

guest death immediately before and after host handoff;

host coordinator death immediately before and after finalizer acceptance;

caller disconnect at every frame and EOF boundary;

finalizer crash before an effect, after effect preparation, after effect invocation, and after observation;

finalizer crash after publisher signer deletion but before response;

finalizer restart after publisher CAS and service state are gone;

same-request replay and conflicting-request replay;

target replacement, multiple matches, symlink substitution, and identity drift;

parity mismatch after irreversible deletion;

every SecurityAgent/UI branch.

Principal tradeoffs

The adjusted Candidate D adds a new privileged TCB, an immutable pre-baseline service identity, another durable journal, and more state transitions. That is real cost.

It buys three critical properties:

the publisher never destroys its own only retry path;

terminal work remains recoverable after all publisher state is gone;

the finalizer cannot be repurposed to recover a precommit-less orphan.

Having the finalizer own the whole host destructive suffix makes it somewhat more capable than a signer-only deleter. The attack surface remains bounded because it has no discretion over scope or targets: it executes one frozen, dual-authorized, live-CAS-verified component plan.

Removing a finalizer signing key slightly reduces independent cryptographic attribution by the executor. Under the stated authority model, that is the correct trade: the publisher and external harness already provide the signatures, and the finalizer should not become a third authority with its own terminal key-retirement problem.

Candidate C delays prospective implementation, but it prevents implementation work from being validated on a contaminated host and forces the SecurityAgent assumption to be resolved using evidence rather than architecture-by-guesswork.

Pitfalls to explicitly prohibit

Assuming root can always delete a System-Keychain key without authentication.

Treating query-level kSecUseAuthenticationUIFail as sufficient without testing the process-level control.

Treating an acknowledgement generated in memory as durably acknowledged.

Allowing the guest or host to delete its only surviving journal before a successor accepts it.

Letting quiescence terminate the only receipt-signing route.

Giving the finalizer a private signing key without a full second retirement protocol.

Allowing the finalizer to read caller-selected artifact paths.

Allowing a different request digest to “repair” an accepted attempt.

Rechecking expiry on a retry after the request was already durably accepted.

Treating caller disconnect as cancellation.

Installing or updating the finalizer after bootstrap precommit.

Excluding finalizer journals or retained evidence from parity without having committed those exclusions before bootstrap.

Using successful creator recovery as proof that a different finalizer identity can delete a prospectively created key.

Accepting “Always Allow” or any persistent ACL change during an operator-prompt recovery.

Using parity failure as authority to delete additional unlisted state.

Reusing prospective V2 schemas, routes, or executables for the preserved orphan.

Bottom line

Adjust, do not pivot. Candidate D and Candidate C remain the right combined strategy when treated as two independently authorized lanes:

Recovery lane: preserve evidence → approve rollback-safe surrogate experiment → choose one exact recovery posture → separately authorize one live recovery → prove exact parity.

Prospective lane: correct the planning foundation → prove finalizer delete-only capability → amend the authoritative contract → only later authorize implementation.

The present plan should remain in a hard unaccepted/no-effects state until the host and guest chronology, surviving handoffs, finalizer route, Keychain capability gate, acknowledgement durability, replay model, and parity ownership are corrected as above.
~~~
