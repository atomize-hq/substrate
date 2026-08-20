# ChatGPT Pro consultation package: macOS XPC `connection-invalid`

- Date: 2026-08-11
- Task/spec: Independent approach review of the first real mapped-lifecycle XPC failure and the still-unexecuted remainder of R3 macOS native evidence
- Exact product source: commit `987e51431cce1159454f8d5b2d3fbff76495c650`, tree `52370189687a7d04c9557cab73bf31bcaa641583`
- Consultation mode: approach review with external research

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## Goal

Determine whether the confirmed absence of a bootstrapped system LaunchDaemon fully explains the observed `XPC_ERROR_CONNECTION_INVALID`, define the smallest proper correction that remains aligned with the runtime-refactor control pack, and audit the unexecuted PM/Lima/pairing/smoke/retirement corridor for likely adjacent blockers before another expensive native evidence run.

## Confirmed evidence

1. The canonical installer ran once from a real local Visual Studio Code controlling terminal on Apple Silicon macOS.
2. All bounded source gates and the isolated AArch64 Linux guest build passed.
3. The direct privileged FD3 bootstrap completed and returned a signed Stage-1 result.
4. The next mapped-lifecycle XPC call classified the reply object as exactly `connection-invalid`.
5. The installer stopped before PM derivation, Lima creation, guest pairing, product smoke, retirement, or MAC-CLOSEOUT.
6. At the failure point, both fixed files existed:
   - `/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1`
   - `/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist`
7. The historical pre-restoration query returned:

~~~text
Bad request.
Could not find service "com.substrate.lifecycle.publisher.v1" in domain for system
launchctl_status=113
~~~

8. Restoration subsequently removed task-created publisher state. Current launchd state is baseline-only and cannot establish more about the historical failure.

## Read-only local diagnosis

The installer copies the helper and plist but never invokes `launchctl` and never bootstraps the plist into the system launchd domain. Therefore the fixed Mach service was not registered for demand activation when the XPC client connected.

This is stronger than a generic hypothesis:

- installed files existed;
- `launchctl print system/com.substrate.lifecycle.publisher.v1` reported the label absent;
- the exact installer source contains no `launchctl` reference;
- the XPC client uses the same fixed service name and correctly distinguished `connection-invalid` from interruption, peer code-signing rejection, unexpected type, and malformed protocol response.

No evidence currently points to a malformed reply, Blocks lifetime fault, or peer code-signing rejection. Those remain possible only after a service is actually registered and invoked.

## Exact source excerpts

### Installer copies files but does not register the service

`scripts/substrate/dev-install-substrate.sh`:

~~~sh
install -d -o root -g wheel -m 0755 /Library/PrivilegedHelperTools
install -d -o root -g wheel -m 0755 /Library/LaunchDaemons
install -d -o root -g wheel -m 0755 "/Library/Application Support/Substrate"
install -d -o root -g wheel -m 0755 "/Library/Application Support/Substrate/lifecycle"
install -o root -g wheel -m 0755 "$executor_src" "$executor_path"
install -o root -g wheel -m 0644 "$plist_src" "$plist_path"
~~~

The full exact installer at this commit has zero `launchctl` references.

### Fixed LaunchDaemon plist

`scripts/mac/com.substrate.lifecycle.publisher.v1.plist`:

~~~xml
<dict>
  <key>Label</key>
  <string>com.substrate.lifecycle.publisher.v1</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1</string>
    <string>run-publisher</string>
  </array>
  <key>MachServices</key>
  <dict>
    <key>com.substrate.lifecycle.publisher.v1</key>
    <true/>
  </dict>
  <key>RunAtLoad</key>
  <false/>
  <key>KeepAlive</key>
  <false/>
</dict>
~~~

### Fixed XPC client

`crates/shell/src/execution/managed_lifecycle/macos_client.rs` creates a privileged Mach-service connection to `com.substrate.lifecycle.publisher.v1`, activates it, performs a synchronous request, then classifies XPC error objects. The relevant closed classification labels are:

~~~text
connection-invalid
connection-interrupted
peer-code-signing-requirement-rejected
unexpected-xpc-type
dictionary-missing-response
dictionary-empty-response
dictionary-oversized-response
dictionary-response
~~~

The native result was exactly `connection-invalid`.

## Control-pack requirements

`llm-last-mile/runtime-refactor/04-contracts-and-gates.md` already requires:

- service/LaunchDaemon registration to use exact readback plus protected-state CAS;
- `mac.publisher.plist` at the fixed LaunchDaemon path with fixed Mach-service declaration;
- `mac.publisher.mach-service` as the non-requestable observed endpoint owned only by the protected publisher bootstrap/retirement service-state transition;
- `mac.publisher.service-state` as an internal protected transition that precommits and receipts the fixed coupled Mach service;
- fail-closed behavior for effect-visible/identity-CAS gaps.

The correction should implement this declared transition, not add a client fallback or relax XPC admission.

## Proposed minimal proper correction

1. After exact helper/plist copy and identity validation, bootstrap the fixed plist into the `system` launchd domain under the existing privileged installer boundary.
2. Read back and exact-verify the label, program path/arguments, Mach-service declaration, and service state before considering `mac.publisher.service-state` installed.
3. Fail closed on any pre-existing, foreign, mismatched, or ambiguous service state; do not adopt or overwrite it silently.
4. Couple the service-state mutation to the protected bootstrap transaction/receipt so an effect-visible/CAS gap preserves first and stops.
5. On rollback/retirement of task-created state, boot out the exact label before deleting its plist/helper, then verify absence.
6. Keep fixed service name, code requirement, audit-token admission, System Keychain signer, direct FD3 bootstrap, and XPC schema unchanged.
7. Prove real demand activation with a native disposable regression; static plist checks are insufficient.

Open design question: whether installation should execute `launchctl bootstrap system <plist>` directly, whether an already-bootstrapped exact matching label is a valid retry state, whether `kickstart` is unnecessary or harmful for a demand-activated Mach service, and what exact `launchctl print` fields are stable enough for authoritative readback across supported Apple Silicon macOS releases.

## Still-unexecuted evidence corridor

After a healthy XPC request, the executor is expected to:

1. validate the retained provenance and signed Stage-1 capsule;
2. execute the closed mapped lifecycle `Create` effect for the canonical same-user Lima instance;
3. observe the guest machine identity and finalize PM;
4. execute the fixed post-PM install sequence;
5. create required guest directories/group/membership/private home/layout sentinel;
6. install four already-built AArch64 roles and guest publisher/service units;
7. enable/start the guest publisher socket and world-service socket/service;
8. perform the R6 direct guest pairing confirmation on an independent guest TTY;
9. run product/lifecycle smoke;
10. retire evidence-created publisher and platform state according to exact receipts.

Static local review found no new contradiction in that remaining path, but none of it has been natively exercised in this sequence. Same-user Lima is intentional for this slice. Ownership-separated Lima and VSock remain future hardening and must not be pulled into this correction.

## Required external research

Use current primary Apple sources wherever possible, and cite URLs. Research:

1. Does copying a plist into `/Library/LaunchDaemons` make a service immediately available, or must an installer explicitly call `launchctl bootstrap system <plist>` on current macOS?
2. What are the demand-activation semantics for a `MachServices` entry when `RunAtLoad=false` and `KeepAlive=false`?
3. Is `kickstart` needed, unnecessary, or counterproductive for a correctly bootstrapped Mach-service LaunchDaemon?
4. What is the safest documented bootstrap/readback/bootout ordering for a root LaunchDaemon installed by a non-package developer installer?
5. Which `launchctl print` or related readback fields are documented/stable enough to validate exact label, executable, domain, and Mach-service registration?
6. Is `XPC_ERROR_CONNECTION_INVALID` consistent with connecting to an unregistered privileged Mach service, and what other documented conditions can produce the same classification?
7. What race or transaction hazards exist between file installation, launchd bootstrap, protected-state CAS, XPC demand activation, bootout, and file removal?
8. Are there Apple-supported alternatives (for example SMAppService or SMJobBless) that are mandatory or strongly preferred for this deployment shape, or would adopting them be an unjustified architecture expansion for this evidence slice?

When Apple public documentation is incomplete, distinguish Apple man pages, Apple Open Source sources, and carefully corroborated secondary material. Do not present community inference as documented behavior.

## Questions for the consultation

1. Is the local diagnosis sufficient to treat missing launchd bootstrap as the root cause of this attempt?
2. Keep, adjust, or pivot the proposed correction?
3. What exact retry/idempotency and rollback state machine is needed for absent, exact-running, exact-loaded-but-idle, mismatched, and ambiguous service states?
4. What minimum native tests prove registration, demand activation, peer admission, crash/exit handling, bootout, and restoration without weakening current controls?
5. Which unexecuted downstream seams deserve a cheap preflight now, and which should remain deferred until this blocker is fixed and real evidence reaches them?
6. What is the smallest reviewed implementation fence consistent with the control pack?


## Consultation surface

- ChatGPT account tier: Pro
- Surface: Chat with Deep research enabled
- Reasoning control: Extra High
- Named backend model: not exposed by the UI
- Conversation: https://chatgpt.com/c/6a7b1e4a-41f4-83ea-9b17-6db96eb61bd3

## Consultation prompt sent

~~~text
We are already working on this project task.

Task/spec: Independently review the confirmed Apple Silicon macOS mapped-lifecycle XPC blocker and the still-unexecuted remainder of the native evidence corridor. Determine whether the missing system-domain LaunchDaemon bootstrap fully explains the observed connection-invalid result, define the smallest proper correction aligned with the supplied control-pack contract, and identify cheap preflights for likely downstream blockers without pulling future ownership-separated Lima or VSock architecture into this slice.

Current approach: Repository-derived inference: the canonical installer copied the fixed root helper and LaunchDaemon plist but never invoked launchctl bootstrap. At the historical failure point both files existed, but launchctl print system/com.substrate.lifecycle.publisher.v1 returned service not found/status 113. The fixed XPC client connected to that exact privileged Mach service and classified the error object as connection-invalid. The proposed correction is to implement the already-declared mac.publisher.service-state transition: privileged system-domain bootstrap after exact file/image checks, exact readback, fail-closed retry/idempotency for pre-existing state, protected receipt/CAS coupling, bootout before file removal, final absence verification, and a native demand-activation regression. Do not add client fallback, weaken XPC admission, or introduce a new transport.

Project sources: Read the attached bounded package chatgpt-pro-xpc-launchd-connection-invalid-package-2026-08-11.md first. It contains redacted evidence facts, exact source excerpts, control-pack requirements, proposed correction, remaining native corridor, and research questions. No excluded repository behavior should be inferred.

Evidence, blockers, constraints, and validation: Signed Stage-1 completed. The next real mapped-lifecycle call returned exactly connection-invalid. PM derivation, Lima creation, guest pairing, product/lifecycle smoke, retirement, and closeout were not reached. The service label was absent from the system launchd domain before restoration. Current host state is baseline-only after exact task-owned restoration and should not be used to invent historical facts. Supported target for this slice is Apple Silicon macOS. Same-user Lima is intentional. Ownership-separated Lima, VSock, Intel support, admission weakening, generic schedulers, caller selectors, client fallbacks, and automatic evidence retries are out of scope. Preserve exact helper/plist/service identity, code requirement, audit-token admission, System Keychain signer, signed wrapper/CAS, preserving-first recovery, and exact rollback. Native validation must prove real registration, readback, Mach-service demand activation, peer admission, restart/crash behavior, retry/idempotency, bootout, and restoration.

Conduct external web research as part of this answer. Prefer current primary Apple documentation and cite URLs. Where Apple public documentation is incomplete, clearly distinguish Apple man pages, Apple Open Source material, and corroborated secondary sources from inference. Specifically research current launchctl bootstrap/print/bootout semantics for system LaunchDaemons; MachServices demand activation with RunAtLoad=false and KeepAlive=false; whether kickstart is necessary or undesirable; documented causes of XPC_ERROR_CONNECTION_INVALID; safe installer transaction ordering; and whether SMAppService or SMJobBless is mandatory/strongly preferred versus unjustified expansion here.

Do not reduce the task or replace it with an easier alternative. Preserve the requested scope and project conventions.

Is the current approach the best way forward? Give a direct keep, adjust, or pivot recommendation; root-cause confidence and alternatives; tradeoffs; exact service-state/retry/rollback state machine; smallest implementation fence; native tests; research citations; downstream preflights; what to defer; and pitfalls.
~~~

## ChatGPT Pro answer

Worked for 10m 30s
Recommendation: ADJUST, not pivot

The core approach is correct. I would make one material adjustment to the transaction boundary and one adjustment to what counts as authoritative “readback,” but I would not change the transport, XPC client, admission policy, service identity, plist semantics, or pull SMAppService/SMJobBless into this remediation.

My root-cause confidence is very high — roughly 98% for the proximate cause of this specific failed attempt.

The package establishes that Stage-1 succeeded, the very next mapped-lifecycle XPC operation returned exactly connection-invalid, both fixed files existed, and launchctl print system/com.substrate.lifecycle.publisher.v1 said that the service did not exist in the system domain. Nothing after that point—PM, Lima, pairing, smoke, retirement—ran. 

chatgpt-pro-xpc-launchd-connect…

 The source review also establishes that the installer copied the files but contained no launchctl registration step, while the client connected to that same fixed privileged Mach-service name. 

chatgpt-pro-xpc-launchd-connect…

That evidence lines up unusually closely with Apple's XPC semantics. Current Apple Developer documentation says a Mach-service connection requires the service name to exist in an accessible Mach bootstrap and to be advertised by a launchd.plist; creation can appear to succeed because the Mach namespace isn't actually queried until the connection is resumed. Apple also documents XPC_CONNECTION_MACH_SERVICE_PRIVILEGED as targeting the privileged Mach bootstrap. 
Apple Developer
+1
 Apple's XPC_ERROR_CONNECTION_INVALID documentation says that error is delivered when a named service cannot be found in the XPC namespace. 
Apple Developer
 Even more specifically, the Apple-shipped XPC man-page text says the privileged Mach-service flag yields XPC_ERROR_CONNECTION_INVALID when the name isn't found in the root Mach bootstrap. 
Keith's GitHub Pages

So I would record the conclusion as:

Missing system-domain registration fully explains the observed connection-invalid result. It does not establish that the publisher has no later defects once launchd actually invokes it.

That distinction matters. The bootstrap blocker is sufficiently established to fix; peer admission, helper launch, protocol handling, crash behavior, and the downstream corridor remain genuinely untested.

1. Why explicit bootstrap is the right correction

Simply placing the plist in /Library/LaunchDaemons is not a sound same-boot installation mechanism. Apple's current Terminal documentation says launchctl is the interface used to load and unload launchd jobs, while its archived daemon guide describes /Library/LaunchDaemons being scanned during system startup. Inference: copying the file may make it eligible at a future startup, but an installer that needs the service available now must explicitly register it. 
Apple Support
+1

The launchctl man page gives the modern primitive directly:

launchctl bootstrap system /Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist

The system domain is the root Mach bootstrap and requires root to modify; bootstrap adds a service definition to that domain and bootout removes it. 
Keith's GitHub Pages
+1

Your plist is also already shaped correctly for launch-on-demand operation: fixed MachServices, RunAtLoad=false, and KeepAlive=false. 

chatgpt-pro-xpc-launchd-connect…

 The launchd man page says KeepAlive=false leaves invocation under demand control, RunAtLoad=false means don't speculatively launch when loaded, and explicitly gives a MachServices plist as an example of a daemon that launches only when a Mach message arrives. 
Keith's GitHub Pages
+3
Keith's GitHub Pages
+3
Keith's GitHub Pages
+3

Therefore: do not add kickstart. kickstart explicitly runs a service immediately regardless of configured launch conditions. Using it in the canonical installation/test path would destroy the thing you actually need the regression to prove: that registration followed by the existing XPC request causes real Mach-service demand activation. 
Keith's GitHub Pages

2. The two adjustments I would make
Adjustment A — collision detection must happen before file overwrite

The proposed sequence says “after exact helper/plist copy and identity validation, bootstrap.” That's right as far as the bootstrap position goes, but it needs an earlier read-only gate.

Before changing either fixed path, inspect:

protected mac.publisher.service-state receipt/CAS;

system/com.substrate.lifecycle.publisher.v1 registration presence;

the existing helper path;

the existing plist path.

If a label or fixed-path artifact already exists without a matching protected transaction/receipt that proves this installation owns it, stop without altering it.

Otherwise an existing foreign/stale service could be loaded from those fixed paths and the installer would overwrite its executable/plist before discovering the collision. That's precisely the kind of preserving-first violation your existing contract is trying to prevent.

Adjustment B — don't parse launchctl print as an authoritative schema

This is the biggest research qualification.

launchctl print is perfectly useful to establish that the service target exists in the system domain and for human/native-test diagnostics. But its own man page says its output is not an API at all and may change without warning. 
Keith's GitHub Pages

So I would implement “exact readback” as three complementary proofs:

Exact configuration/image readback: reread and parse the actual installed plist; verify the fixed Label, exact ProgramArguments, exact MachServices, RunAtLoad=false, KeepAlive=false, ownership/mode, helper digest/code identity, etc.

Registration-domain proof: launchctl print system/com.substrate.lifecycle.publisher.v1 succeeds. Use this as a presence assertion, not a parser for undocumented fields.

Endpoint proof: a real request through the existing privileged XPC client successfully demand-activates the helper and passes the existing code-requirement/audit-token admission.

That actually gives you stronger evidence than parsing a human-readable launchctl dump.

If the existing control-pack phrase “exact readback” was intended literally to mean parsing launchd's internal ProgramArguments and MachServices fields from print, I would clarify the control pack rather than build a security boundary on explicitly unstable output. The package already requires exact readback plus protected-state CAS, but doesn't require that exact configuration truth come from undocumented print fields. 

chatgpt-pro-xpc-launchd-connect…

3. Exact service-state / retry state machine

I would make receipt ownership authoritative, not “the bytes happen to match.”

Observed state	Required behavior
Service absent; fixed paths absent; no receipt	Valid fresh-install starting state. Precommit install intent, then proceed.
Service absent; matching install-intent exists; paths absent	Safe retry before any effect: continue installation.
Service absent; matching install-intent; exact task-owned files exist	Safe pre-bootstrap retry: reverify files, then bootstrap.
Service absent; no receipt but one/both fixed paths already exist, even if bytes match	Ambiguous/pre-existing. Fail closed; do not adopt or overwrite.
Service registered; committed matching installed receipt; files exact; process idle	Idempotent success. Do not kickstart.
Service registered; committed matching installed receipt; files exact; process running	Idempotent success. Do not restart simply to normalize state.
Service registered; only install-intent exists	Effect-visible/CAS gap. Preserve state and stop. No silent adoption.
Service registered; no matching receipt	Foreign/ambiguous. Preserve and fail.
Service registered; receipt/files/code identity mismatch	Preserve and fail.
Service absent; committed installed receipt says it should exist	Unexpected drift. Fail closed; do not silently re-bootstrap.
launchctl observation is neither definite presence nor definite absence	Indeterminate. Preserve and fail.

The important subtlety is that “exact loaded but idle” and “exact running” are both healthy registered states. Process liveness is not service identity. A demand-managed Mach service can legitimately have a registered endpoint while its backing process is not running. 
Keith's GitHub Pages
+1

And because print is not machine-stable, I would not make production retry logic depend on parsing its textual state = running or PID fields. Native diagnostics can record those fields; the protected state machine shouldn't.

4. Install transaction I would implement

This is the smallest transaction consistent with the package's preserving-first/CAS requirements:

A. Pre-mutation observation. Read receipt/CAS, query the exact system service target, and inspect both fixed paths. Reject any unowned or ambiguous pre-existing state before touching files.

B. Precommit. CAS mac.publisher.service-state into the existing install-intent/precommitted form, binding whatever exact service/plist/helper identity the control pack already requires. Don't invent a second ownership mechanism.

C. Install files. Install the fixed helper and fixed plist using the current privileged boundary.

D. Exact file/image readback. Parse/read back the plist and validate the helper image/code requirement, path, owner/mode, digests, etc. Re-read protected state.

E. Close the pre-bootstrap race. Query system/com.substrate.lifecycle.publisher.v1 again. It must still be absent. If somebody registered that name between A and E, preserve and stop.

F. Bootstrap exactly once:

launchctl bootstrap system /Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist

No kickstart, no legacy load, no client fallback. bootstrap is the documented modern domain operation. 
Keith's GitHub Pages

G. Observe the effect. launchctl print system/com.substrate.lifecycle.publisher.v1 must now succeed. Re-read the plist/helper exactly again. Do not parse undocumented print body fields as the identity source.

H. Commit the protected installed state. CAS from the exact precommitted intent into installed/receipted state.

I. Only then continue to the real XPC demand request.

The error handling around F–H is important:

bootstrap fails and service is definitely absent → no live service effect occurred; rollback only the exact files created by this transaction, then close the intent according to existing rollback semantics.

bootstrap fails but the service is now present → ambiguous effect; preserve and stop.

bootstrap succeeds but subsequent registration observation is indeterminate → preserve and stop.

service becomes visible but final protected CAS fails → preserve and stop. This is exactly the effect-visible/identity-CAS gap the control pack already says must fail closed. 

chatgpt-pro-xpc-launchd-connect…

Do not turn the latter into “well, we know it was probably ours, so clean it up.” That weakens the preserving-first rule.

5. Retirement / rollback state machine

Retirement should be the mirror image, but bootout precedes file deletion:

Require the committed protected receipt and exact current task-owned helper/plist.

CAS to retirement intent.

Execute:

launchctl bootout system/com.substrate.lifecycle.publisher.v1

Verify the service target is absent.

Reverify exact task-owned file identities.

Remove plist/helper.

Verify service absent and files absent.

Commit retired/restored protected state.

bootout explicitly removes the service definition from the domain. 
Keith's GitHub Pages

Useful retry rules are:

retirement-intent + service absent + exact task-owned files still present → safe continuation after a crash between bootout and deletion;

retirement-intent + service absent + files absent → complete the protected closeout/CAS;

installed receipt + unexpectedly absent service, with no retirement intent → drift, not an implied retry; stop;

bootout reports failure but subsequent observation is indeterminate → preserve;

any file changed from the receipted identity → do not delete it.

That gives you useful idempotency without turning “retry” into unbounded state adoption.

6. Native validation corridor for this change

I would require these before calling the service-state fix proven:

Registration without launch. Begin from a disposable, proven-absent task service. Install/bootstrap it. Establish registration, but don't use kickstart. This proves the installer can register the endpoint without disguising demand behavior.

Real Mach-service demand activation. The first thing that actually starts the publisher should be the existing mapped-lifecycle XPC request. A successful response proves substantially more than a process check: Mach lookup, launch, protocol setup, and endpoint plumbing. Apple describes named XPC connections as virtual; sending to a registered but non-running endpoint causes on-demand launch. 
Keith's GitHub Pages

Peer admission positive and negative. The legitimate existing client must pass the fixed code requirement/audit-token checks. Retain a negative client/admission control. Do not “fix” an admission failure by changing the requirement.

Normal exit/re-demand. Let the publisher exit in the supported idle path, then send another request. It should demand-launch again.

Crash/restart behavior. After one healthy request, terminate/crash the disposable publisher. XPC documents a remote crash/close on a named service as XPC_ERROR_CONNECTION_INTERRUPTED, with the connection remaining recoverable. A subsequent demand should relaunch the service. 
Keith's GitHub Pages
 Do not require KeepAlive=false to make launchd immediately respawn it without demand.

Retry matrix. Exercise fresh absent, committed idle, committed running, present/no receipt, present/install-intent gap, mismatch, and indeterminate observation.

Bootout while connected. Boot out the registered service. Apple's XPC man page says unloading a Mach-service job can produce interruption followed by invalidation; after removal a fresh privileged connection should no longer find the service. 
Keith's GitHub Pages

Exact restoration. Remove only receipted task-owned artifacts after bootout, verify service and fixed files absent, and restore the protected state exactly. Current baseline state can be used for this prospective test, but—as the package correctly warns—not as retrospective evidence about the failed run. 

chatgpt-pro-xpc-launchd-connect…

I would also add failure injection around the narrow transactional seams: after precommit, after file copy, around bootstrap, after visible registration but before final CAS, after retirement intent, after bootout, and after file deletion. The state-machine unit tests are cheap; only a small subset needs real native launchd/XPC execution.

7. Other documented causes of connection-invalid

Missing registration is not the only theoretical path to INVALID.

Apple also documents cancellation as producing XPC_ERROR_CONNECTION_INVALID; creator-side cancellation can propagate INVALID to the peer. 
Keith's GitHub Pages
 A service can also be malformed with respect to how its Mach endpoint is registered, and unloading can eventually invalidate connections. Meanwhile, a remote process crash normally produces XPC_ERROR_CONNECTION_INTERRUPTED, not the initial missing-name signature seen here. 
Keith's GitHub Pages

Those alternatives matter for the next native run, but none is a better explanation for the historical failure than the exact missing root-bootstrap name. The package also explicitly says there was no evidence at that point for malformed reply, Blocks lifetime, or peer signing rejection. 

chatgpt-pro-xpc-launchd-connect…

So I would not spend implementation effort “defensively” changing those paths before registration is fixed.

8. Smallest implementation fence

The reviewed implementation fence should be extremely narrow:

In scope: the existing privileged installer/service-state code necessary to observe, precommit, bootstrap, read back, receipt, bootout, and restore mac.publisher.service-state; focused state-machine tests; disposable Apple-Silicon native launchd/XPC tests; and, only if necessary, a tiny control-pack clarification that launchctl print body text is diagnostic rather than an authoritative structured API.

Out of scope: changes to macos_client.rs behavior, service name, plist service semantics, XPC schema, client fallback, peer requirement, audit-token admission, FD3 Stage-1, signer/keychain routing, PM derivation, Lima ownership model, guest transport, VSock, Intel, generic launch manager abstraction, or automated evidence retry.

In particular, I see no reason to change the fixed plist itself based on the package. The declared MachServices/RunAtLoad=false/KeepAlive=false combination is the right architecture for this use. 

chatgpt-pro-xpc-launchd-connect…

9. Cheap downstream preflights worth doing now

The rest of the native corridor has not run at all. 

chatgpt-pro-xpc-launchd-connect…

 I would spend a little effort on non-mutating or artifact-only checks, but not start fixing hypothetical downstream behavior.

Lima name/template collision: run the existing/pinned limactl, validate the canonical template, and query existing instances in machine-readable form before Create. Current Lima exposes limactl validate for template validation and limactl list --format json; use whatever exact version/flags your repository pins rather than silently upgrading to current Lima. 
Lima
+1
 A pre-existing canonical instance without the appropriate project receipt should be treated as a prospective collision, not adopted.

Four AArch64 payloads: cheaply prove all four already-built guest roles exist, have the expected AArch64 executable format, and match whatever digests/manifests are already authoritative. Do not rebuild opportunistically during the evidence run.

Guest unit/socket payload: statically verify that the fixed publisher/world service/socket unit names and referenced binary paths correspond to the already-defined post-PM install sequence. This can catch a typo before paying for Lima creation without asserting that systemd will actually start them.

Same-user template inputs: statically verify that the canonical template still expresses the already-approved same-user expectation. Don't turn a mismatch into ownership-separated Lima work; simply fail the preflight.

Independent guest-TTY harness: confirm the planned test harness has a way to open the required independent guest terminal for R6 after creation. Don't pair yet and don't introduce a new transport.

System Keychain prerequisites: recheck, read-only, that the already-required signer/key material and code identities needed by the post-Stage-1 path remain available. Do not rotate or repair them automatically.

Host launchd/BTM policy diagnostic: on the actual evidence host, a read-only Background Task Management check is now worthwhile. Apple says current macOS background-task management covers LaunchDaemons and provides sfltool dumpbtm and relevant logging for diagnostics. On macOS 26, background-task behavior also has additional user/management policy considerations. 
Apple Support
 Treat this solely as a prospective environment preflight, never as a retrospective explanation for the historical failure.

And immediately before the next native run, verify the exact fixed service/path baseline and protected receipt state once more. Again, that's a run precondition, not historical evidence.

10. SMAppService / SMJobBless: do not pull either into this correction

This deserves a qualification rather than a hand-wave.

For modern product architecture, SMAppService is clearly Apple's preferred direction for an app that owns ongoing privileged background service installation. Apple's current documentation exposes SMAppService for registering LaunchDaemons, and app-backed daemon registration expects the plist inside the calling app's Contents/Library/LaunchDaemons. 
Apple Developer
+1
 Apple DTS guidance is even clearer: if an installer package is appropriate, use one; for ongoing privileges without a package, use SMAppService; SMJobBless is the older path and is deprecated as of macOS 13. 
Apple Developer

But that does not make SMAppService mandatory for fixing this bounded evidence slice.

Migrating now would change:

packaging/layout ownership;

how registration is authorized/approved;

the fixed /Library/LaunchDaemons installation contract;

helper lifecycle semantics;

probably the installer boundary and receipts.

That is architecture work, not remediation of the proven missing bootstrap transition. Your control pack already owns mac.publisher.service-state and explicitly expects the fixed LaunchDaemon/Mach-service pair. 

chatgpt-pro-xpc-launchd-connect…

So:

Direct launchctl bootstrap now. SMAppService/package architecture as a separately reviewed productization decision later. Do not pivot to SMJobBless.

I also found no need to rely on Apple Open Source implementation details for this conclusion. The public XPC API docs, Apple documentation, and Apple-shipped launchctl/launchd man-page contracts are sufficient; using launchd implementation source would give you more internal detail but not a better stable contract.

11. Things to explicitly defer

Do not widen this blocker into PM/Lima behavior before evidence reaches it. Specifically defer ownership-separated Lima, VSock, Intel support, alternate transports, client fallbacks, admission changes, general service managers, automatic retry/reconciliation, and speculative fixes to guest pairing or smoke. The package explicitly marks same-user Lima as intentional and those architectural changes as future work. 

chatgpt-pro-xpc-launchd-connect…

Likewise, don't “pre-prove” PM derivation, actual Lima Create, guest service startup, R6 pairing, product smoke, or retirement from static inspection. Cheaply validate their inputs; let the corrected native run tell you which seam actually fails next.

Main pitfalls I would guard against

The dangerous mistakes are: overwriting fixed paths before checking for an already-loaded foreign label; treating matching bytes without a protected receipt as ownership; parsing launchctl print as a durable API; hard-coding historical status 113 as the only possible absent-service signal across macOS releases; using kickstart and accidentally producing a false-positive activation test; treating “process not running” as “service not registered”; automatically re-bootstraping a committed service that unexpectedly disappeared; deleting files before bootout; deleting changed/unreceipted files during rollback; expecting KeepAlive=false to crash-loop rather than re-launch on demand; and treating the restored current host as evidence about the historical failure.

Bottom line

ADJUST. Implement the already-designed mac.publisher.service-state transition with explicit privileged bootstrap system, but add a pre-mutation collision gate and change “exact launchctl readback” into exact disk/image readback + stable service-presence observation + real XPC demand proof. Preserve the existing identity/admission/receipt architecture. Use no kickstart. Treat an already-loaded service as retry-success only when the protected receipt/CAS proves that it is your exact committed installation. Boot out before removal. Preserve on any effect-visible/CAS ambiguity.

That is the smallest correction that both explains the historical failure and gives the next native run a legitimate chance to reach the still-untested PM/Lima corridor, without borrowing any future ownership-separated or VSock architecture.

Sources
Do you like this personality?

## Research URLs captured from the answer

- [Apple Developer+1](https://developer.apple.com/documentation/xpc/xpc_connection_mach_service_privileged?changes=_11&language=objc)
- [Apple Developer](https://developer.apple.com/documentation/xpc/xpc_error_connection_invalid-c.macro?language=objc)
- [Keith's GitHub Pages](https://keith.github.io/xcode-man-pages/xpc_connection_set_event_handler.3.html)
- [Apple Support+1](https://support.apple.com/en-mide/guide/terminal/apdc6c1077b-5d5d-4d35-9c19-60f2397b2369/mac)
- [Keith's GitHub Pages](https://keith.github.io/xcode-man-pages/launchctl.1.html)
- [Keith's GitHub Pages+3Keith's GitHub Pages+3Keith's GitHub Pages+3](https://keith.github.io/xcode-man-pages/launchd.plist.5.html)
- [Lima+1](https://lima-vm.io/docs/reference/limactl_list/)
- [Apple Support](https://support.apple.com/guide/deployment/manage-login-items-background-tasks-mac-depdca572563/web)
- [Apple Developer+1](https://developer.apple.com/documentation/servicemanagement/smappservice?language=objc)
- [Apple Developer](https://developer.apple.com/forums/tags/servicemanagement/?page=2&sortBy=oldest)

## Follow-up prompt sent

~~~text
Given your answer, what is the fastest way to execute or continue while fully complying with the original task and constraints? Do not reduce scope. Provide the exact order of operations, minimum files or areas to touch, commands, validation, and what to defer until after the next working slice.

Resolve one important bootstrapping-locus question explicitly. The direct retained-FD3 privileged helper already runs and produces the signed Stage-1 result, then exits. The next operation is a mapped lifecycle Create sent through the fixed XPC service, but that XPC service is unavailable until its LaunchDaemon has been bootstrapped. The control pack describes mac.publisher.service-state as an internal protected transition with CAS/receipts. Explain which component should perform the first launchctl bootstrap without creating a circular dependency: (a) the canonical installer shell under its existing privileged boundary after signed Stage-1, (b) the direct FD3 privileged executor before it returns Stage-1, (c) a narrowly authorized local control/executor action after Stage-1 but before XPC, or (d) another existing mechanism. Specify how that choice preserves precommit, protected CAS, retry, effect-visible ambiguity, and retirement. Do not invent a generic service manager or weaken XPC admission.
~~~

## ChatGPT Pro follow-up answer

Worked for 3m 12s
Recommendation: choose (c)

Use a narrowly authorized local protected-executor action after signed Stage-1 and before the first XPC Create.

The canonical installer remains the orchestrator, but does not itself own launchctl bootstrap as a bare shell side effect. The same local privileged executor mechanism that already exists independently of XPC should execute one fixed mac.publisher.service-state transition. That transition owns precommit → bootstrap → observation → protected CAS/receipt. This stays within the bounded source at commit 987e514... and does not infer excluded repository behavior. 

chatgpt-pro-xpc-launchd-connect…

Conceptually:

installer
   │
   ├── install/verify fixed helper + plist
   │
   ├── retained-FD3 Stage-1 authorization
   │       └── signed Stage-1 returned; executor exits
   │
   ├── local protected service-state action      <-- ADD THIS
   │       ├── verify Stage-1/protected attempt
   │       ├── precommit CAS
   │       ├── launchctl bootstrap system ...
   │       ├── verify registration
   │       └── commit receipt CAS
   │
   └── existing XPC mapped-lifecycle Create
           └── MachServices demand-launches publisher

This is the fastest route that preserves the control-pack architecture.

Why not the other loci?

(a) Bare installer-shell launchctl bootstrap: reject as the owner. The installer can sequence the operation, but a raw shell mutation between Stage-1 and XPC would separate the effect from the protected precommit/receipt machinery. The package explicitly says mac.publisher.service-state must be a protected, precommitted, receipted transition and must fail closed across effect-visible/CAS gaps. 

chatgpt-pro-xpc-launchd-connect…

(b) Stage-1 executor before returning Stage-1: don't do this unless the repository proves there is already no semantic separation between those transitions. It would make obtaining an authorization artifact mutate launchd, broaden the one-shot Stage-1 operation, entangle its retry/timeout semantics with service installation, and make “Stage-1 succeeded” mean something different from today. There is no need for that coupling.

(c) is the clean seam. It reuses a local privileged route that does not depend on the unavailable XPC Mach service, while implementing exactly the already-declared internal transition rather than introducing a generic service manager.

(d) only supersedes (c) if repository inspection reveals an existing post-Stage-1 protected effect executor whose contract already includes mac.publisher.service-state. If so, use that existing mechanism rather than adding another entrypoint. The supplied package doesn't establish one, so it would be wrong to assume it exists.

Apple's architecture supports this sequencing: a launchd service is a virtual service that can remain idle until an advertised endpoint receives demand; MachServices registers that endpoint, and KeepAlive=false leaves launching demand-controlled. 
Keith's GitHub Pages
+1
 The existing privileged XPC connection specifically expects its service in the privileged Mach bootstrap. 
Apple Developer

Exact fastest execution order
0. Freeze the implementation boundary first

From the repository root:

Bash
git status --short
git rev-parse HEAD
git rev-parse HEAD^{tree}

rg -n \
  'mac\.publisher\.service-state|mac\.publisher\.mach-service|mac\.publisher\.plist' \
  llm-last-mile crates scripts

rg -n \
  'Stage-1|stage.?1|FD3|fd3|run-publisher|PrivilegedHelperTools' \
  crates scripts

rg -n \
  'launchctl|bootstrap|bootout|MachServices' \
  crates scripts llm-last-mile

Expected starting identity from the bounded package:

HEAD = 987e51431cce1159454f8d5b2d3fbff76495c650
tree = 52370189687a7d04c9557cab73bf31bcaa641583

The package establishes that dev-install-substrate.sh currently installs the helper/plist but has no launchctl operation. 

chatgpt-pro-xpc-launchd-connect…

The first search determines whether (c) can use an already-existing protected action dispatcher. Do not create a second executor mechanism if one already exists.

1. Minimum areas/files to touch
Definitely inspect, and probably modify

1. scripts/substrate/dev-install-substrate.sh

Only add orchestration of the protected service-state transition at the exact seam:

signed Stage-1 accepted
        ↓
mac.publisher.service-state install
        ↓
mapped lifecycle Create over XPC

The shell should not contain the security-sensitive state machine itself and should not blindly execute launchctl.

2. The existing privileged executor/action implementation that owns protected state/CAS.

The package does not expose its exact path, so I would not invent one. The rg commands above should identify it.

Add exactly one closed operation representing the existing contract concept:

mac.publisher.service-state

with only its fixed install/retire semantics.

It must not accept:

arbitrary service labels;

arbitrary plist paths;

arbitrary helper paths;

arbitrary domain names;

caller-selected commands.

The constants remain:

label:
com.substrate.lifecycle.publisher.v1

plist:
/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist

helper:
/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1

domain:
system

Those identities are already fixed by the supplied source. 

chatgpt-pro-xpc-launchd-connect…

3. Focused tests adjacent to that executor/state-machine implementation.

Prefer the existing test location rather than creating a new testing framework.

Probably do not modify
scripts/mac/com.substrate.lifecycle.publisher.v1.plist
crates/shell/src/execution/managed_lifecycle/macos_client.rs

There is currently no evidence requiring changes to either. The plist already advertises the correct fixed Mach service with RunAtLoad=false and KeepAlive=false, and the XPC client correctly distinguished the failure. 

chatgpt-pro-xpc-launchd-connect…

Control-pack documentation

Only touch:

llm-last-mile/runtime-refactor/04-contracts-and-gates.md

if implementation reveals ambiguity about the meaning of “exact readback.”

I would clarify, if needed, that authoritative verification is:

exact installed plist/helper/image readback
+ definite system-domain registration observation
+ native XPC demand proof

rather than parsing undocumented launchctl print fields.

The Apple-shipped launchctl(1) man page (here via the current Xcode man-page mirror, not an Apple web API specification) describes print as an inspection command but warns elsewhere in that interface that its output shouldn't be treated as a stable API. Use success/failure and native diagnostics, not a parser coupled to internal formatting. 
Keith's GitHub Pages

2. Put a cheap collision preflight before overwrite

Keep this separate from the new mutation.

Before replacing either fixed file, the installer should establish:

protected receipt state is compatible with fresh/retry install
AND
fixed LaunchDaemon service is not foreign/ambiguous
AND
fixed helper/plist paths are absent or provably task-owned

For manual diagnosis:

Bash
LABEL='com.substrate.lifecycle.publisher.v1'
PLIST='/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist'
HELPER='/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1'

sudo /bin/launchctl print system
sudo /bin/launchctl print "system/$LABEL"

launchctl print system should first prove that querying the target domain itself works. Then the service-target query establishes whether that fixed target is registered.

Do not build production semantics around historical status 113. The package's historical 113 is valuable evidence for that run, but the stable invariant you need is “target registration definitely present / definitely absent / observation failed.” 

chatgpt-pro-xpc-launchd-connect…

If either installed path already exists, inspect rather than overwrite:

Bash
sudo /usr/bin/stat -f '%N %Su:%Sg %Sp' "$HELPER" "$PLIST" 2>/dev/null || true
sudo /usr/bin/codesign --verify --strict --verbose=4 "$HELPER"
sudo /usr/bin/codesign -d -r- "$HELPER"
sudo /usr/bin/plutil -lint "$PLIST"

These are useful operator diagnostics. Existing repository code for exact code/image/plist validation should remain authoritative if it already exists.

3. Leave Stage-1 itself alone

Run the existing retained-FD3 flow unchanged.

It should still perform exactly its current authorization transaction and return the signed Stage-1 result.

Only after the caller has accepted/verifed that result does it advance to:

mac.publisher.service-state/install

This is important operationally because the package proves Stage-1 itself already worked. Changing it unnecessarily would enlarge the regression surface. 

chatgpt-pro-xpc-launchd-connect…

It also keeps the recently established Stage-1 response-production timing model independent of launchd registration.

4. Implement service-state/install as one closed protected transaction

Inside the privileged action, the order should be exact.

Phase I — observe

Read:

current protected service-state receipt/CAS
fixed installed plist identity
fixed helper/image identity
system/<fixed-label> registration state

Classify into the closed state machine below.

Phase II — protected precommit

Before calling launchctl:

CAS:
expected previous service-state
    ->
install-intent / precommitted

Bind this to the same existing attempt/authorization identity machinery already used by the control pack. Do not introduce caller-selected ownership tokens.

If the CAS fails:

STOP
NO launchctl
NO cleanup of unowned state
Phase III — final pre-effect verification

Re-read:

plist
helper
service registration state
protected precommit

This closes the obvious check→mutation race.

Phase IV — actual launchd mutation

The one fixed command is:

Bash
/bin/launchctl bootstrap \
  system \
  /Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist

The Apple-shipped launchctl(1) interface defines bootstrap as adding service definitions to a domain. 
Keith's GitHub Pages
 Apple's own Terminal documentation likewise says launchctl is the interface for loading/unloading launchd jobs rather than merely copying files. 
Apple Support

Do not run:

Bash
launchctl kickstart ...

A successfully bootstrapped MachServices job is supposed to be launchable in response to endpoint demand; KeepAlive=false means demand controls invocation and RunAtLoad=false avoids speculative launch. 
Keith's GitHub Pages
+1

Phase V — observe effect

Immediately after bootstrap:

Bash
/bin/launchctl print \
  system/com.substrate.lifecycle.publisher.v1

This must succeed.

Then exact-reverify the installed plist and helper through the existing validators.

Phase VI — protected commit

Only then:

CAS:
install-intent
    ->
installed receipt

The receipt records the existing fixed identities/effect evidence required by the control pack.

Critical rule

If registration is visible but this final CAS fails:

DO NOT bootout
DO NOT delete files
DO NOT silently adopt
RETURN effect-visible/CAS-gap failure
PRESERVE FIRST

That behavior is directly required by the control-pack evidence supplied in the package. 

chatgpt-pro-xpc-launchd-connect…

5. Use this exact retry classification
Protected state	launchd	Exact files	Result
fresh/absent	absent	expected fresh/task-owned	precommit → bootstrap
install-intent	absent	exact	safe continuation → bootstrap
install-intent	present	exact	effect-visible ambiguity; preserve/fail
installed	present	exact	idempotent success
installed	absent	exact	drift; fail closed
no ownership receipt	present	any	foreign/ambiguous; preserve/fail
no ownership receipt	absent	pre-existing paths	foreign/ambiguous; preserve/fail
any state	present	mismatch	preserve/fail
any state	observation indeterminate	any	preserve/fail

I would deliberately not make:

install-intent + service present

automatically become success, even if the files happen to match.

That is precisely the crash window where launchd may have accepted the effect but the protected final CAS never happened. Silently completing ownership from matching ambient state weakens the control pack's effect-visible gap rule.

6. Immediately prove the native activation seam

Once unit/state-machine tests pass, do one disposable Apple-Silicon native test before retrying the larger evidence corridor.

Starting from proven absence:

Bash
sudo /bin/launchctl print \
  system/com.substrate.lifecycle.publisher.v1

Expected: absent.

Run the normal installer path through Stage-1 and the new protected service-state action.

Then:

Bash
sudo /bin/launchctl print \
  system/com.substrate.lifecycle.publisher.v1

Expected: registered.

Do nothing else to start it.

No:

Bash
launchctl kickstart

Then invoke the existing fixed XPC mapped-lifecycle request.

That request must be what crosses the demand boundary. Apple's launchd contract describes services/endpoints exactly this way: a service can be available for spawning and a message to its advertised endpoint causes launch on demand; MachServices advertises those names in the bootstrap namespace. 
Keith's GitHub Pages
+1

The client continues using its privileged Mach-service connection unchanged. Apple's current XPC documentation says that flag specifically requires the advertised service to live in the privileged Mach bootstrap. 
Apple Developer

The validation sequence should prove:

registration succeeds
→ no kickstart
→ existing XPC request demand-activates publisher
→ existing peer code/audit-token admission succeeds
→ valid closed response returns

That is the next working slice.

Do not require Lima Create to succeed before accepting this launchd/XPC slice as fixed; but after it succeeds, continue the original native corridor immediately rather than stopping the overall task.

7. Add crash/re-demand validation before the larger run

The minimum native regression should then exercise:

1. bootstrap
2. first demand activation
3. valid peer admission/request
4. publisher normal exit
5. second demand activation
6. forced publisher crash/termination
7. subsequent demand activation
8. retry/idempotent install invocation
9. retirement/bootout
10. final absence/restoration

Do not change KeepAlive to make crash handling easier. With KeepAlive=false, demand—not speculative permanent residency—is intentionally controlling invocation. 
Keith's GitHub Pages

XPC_ERROR_CONNECTION_INVALID can also result from cancellation or a connection becoming invalid, so successful registration doesn't imply every future INVALID is impossible; that's why the native admission/request test still matters. Apple explicitly documents INVALID for cancellation and for messages whose connection became invalid. 
Apple Developer
+1

8. Implement retirement through the same narrow action

Do not have the installer simply rm the two files.

The same protected mac.publisher.service-state implementation gets a fixed retirement path:

verify committed installed receipt
        ↓
verify exact service/files
        ↓
CAS installed → retirement-intent
        ↓
launchctl bootout
        ↓
verify service absent
        ↓
remove only exact task-owned fixed files
        ↓
verify files absent + service absent
        ↓
CAS retirement-intent → retired/restored

The actual removal command inside the narrow privileged transition is:

Bash
/bin/launchctl bootout \
  system/com.substrate.lifecycle.publisher.v1

bootout is the corresponding launchctl operation for removing the service from its domain. 
Keith's GitHub Pages

Only after definite absence should the two fixed task-owned files be removed:

/Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist
/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1

If their identities no longer match the receipt, do not delete them.

For a crash during retirement:

retirement-intent + service absent + exact files present

is safely resumable at file deletion.

retirement-intent + service absent + files absent

is safely resumable at final CAS.

Unknown/mismatching state preserves first.

9. Then rerun the actual corridor, in its original order

Once the native launchd/XPC slice is proven, continue without redesigning anything:

signed Stage-1
↓
protected publisher service-state bootstrap
↓
real mapped-lifecycle XPC Create
↓
retained provenance + signed Stage-1 validation
↓
canonical same-user Lima Create
↓
observe guest machine identity
↓
finalize PM
↓
fixed post-PM installation
↓
guest dirs/group/membership/private-home/layout sentinel
↓
four AArch64 roles
↓
guest publisher/service units
↓
guest publisher + world sockets/services
↓
R6 independent guest-TTY pairing confirmation
↓
product/lifecycle smoke
↓
exact publisher/platform retirement
↓
closeout/restoration

That is the original unexecuted corridor; none of it was reached in the failed attempt. 

chatgpt-pro-xpc-launchd-connect…

10. Cheap preflights before spending another native run

Do these after the XPC slice passes but before triggering the full Create corridor:

Bash
# Host architecture
uname -m
# expected: arm64

# Lima exists; record actual version
command -v limactl
limactl --version

# Observe existing machines before touching them
limactl list

# Verify built guest payload architectures
file <each-of-the-four-already-built-AArch64-role-binaries>

# Validate the fixed plist
sudo /usr/bin/plutil -lint \
  /Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist

# Verify helper signing/image
sudo /usr/bin/codesign --verify --strict --verbose=4 \
  /Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1

Also statically resolve every guest install path and every guest service/socket unit's referenced executable before creating the VM.

If the repository already pins or wraps a Lima template validator, run that existing command. The bounded package does not provide its exact syntax, so I would not substitute an invented current-Lima command and silently widen the supported tool version.

Likewise, confirm that the R6 independent guest-TTY mechanism required by the existing plan can actually be opened, but do not start inventing alternate pairing transports.

What to defer until after the next working slice

Do not work on any of these now:

ownership-separated Lima;

VSock;

Intel macOS;

SMAppService migration;

SMJobBless;

generic launchd/service-management abstraction;

client fallback transport;

XPC admission weakening;

changing the fixed service name;

changing RunAtLoad or KeepAlive;

arbitrary scheduler/control verbs;

caller-selectable plist/service/path arguments;

speculative PM/Lima fixes;

automatic evidence retries.

Apple currently provides SMAppService for app-owned LaunchDaemon registration on macOS 13+, but moving this fixed /Library/LaunchDaemons installer/control-pack architecture to it changes the packaging and registration model. It is therefore a separate architecture/productization decision, not the fastest compliant repair of this proven blocker. 
Apple Developer
+1

The smallest change in one sentence

Keep Stage-1 unchanged; have the canonical installer, immediately after verifying its signed Stage-1 result, invoke one fixed existing-local-executor mac.publisher.service-state/install action that performs protected precommit → exact revalidation → /bin/launchctl bootstrap system <fixed-plist> → definite registration observation → protected receipt CAS, then proceed unchanged into the existing XPC Create.

That resolves the circularity without putting service registration inside XPC, without making authorization itself install the daemon, and without letting the shell create an unreceipted privileged launchd side effect.

Sources

## Additional research URLs captured from the follow-up

- [Keith's GitHub Pages](https://keith.github.io/xcode-man-pages/launchd.plist.5.html?utm_source=chatgpt.com)
- [Apple Developer](https://developer.apple.com/documentation/xpc/xpc_connection_mach_service_privileged?language=objc&utm_source=chatgpt.com)
- [Keith's GitHub Pages](https://keith.github.io/xcode-man-pages/launchctl.1.html?utm_source=chatgpt.com)
- [Apple Support](https://support.apple.com/guide/terminal/script-management-with-launchd-apdc6c1077b-5d5d-4d35-9c19-60f2397b2369/mac?utm_source=chatgpt.com)
- [Apple Developer+1](https://developer.apple.com/documentation/xpc/xpc_connection_cancel%28_%3A%29?language=objc&utm_source=chatgpt.com)
- [Apple Developer+1](https://developer.apple.com/documentation/servicemanagement/smappservice?utm_source=chatgpt.com)
