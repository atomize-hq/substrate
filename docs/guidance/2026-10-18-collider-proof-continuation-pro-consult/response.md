# Pro approach-review response — 2026-10-18

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

Conversation: https://chatgpt.com/g/g-p-689bbd06a0d08191ae4d7e65be750533-substrate/c/6aa9dc25-5ee8-83ea-95d2-8b085b2d27d6

Recommendation: KEEP the existing staged proof; adjust the resumption scope and coverage accounting

The proposed direction is correct: resume the existing C-OQ-027 proof, reuse Packet A’s corrected client and demonstrated UI pieces, identify the remaining coordinated-workflow delta, and implement only that delta after authorization. This is not a new fixture-first strategy, a restart of Packet A, or a substitute for real owner integration.

The newer findings sharpen my previous advice in two ways:

Collider needs more than adapter wiring, but less than a fresh UI implementation. Its main application is a shell; its isolated proof already contains tested controller, coordinated-view, placement, and DataGrid behavior.

B04 now points toward specific owner-composition and admission work, not merely finding a convenient existing SDK call. That affects the real-owner path. It does not make that work a prerequisite for every explicitly synthetic UI test.

I read the supplied C-OQ-027 and proof specification first, then Packet A and the reconciliation proposal, and used the full glossary/Foundations to check product intent. This review is bounded to the supplied Collider a491f987…, Substrate aa5ef73a…, and Handbook 379af4ed… material; I performed no repository refresh or runtime tests. 
bounded-evidence

1. Resuming the staged proof is the correct reading

The sequence is already explicit in the supplied documents:

Document section	Existing direction
C-OQ-027 — Required proof evidence	Separate contract, actual-client fixture, owner-integrated, and deployment/artifact/economic evidence.
Open register §5.1	Run the separately authorized staged proof; retain separation between its results; permit stable Shell design alongside it.
Proof specification §4	Defines Gates A–D and what each can establish.
Proof specification §§5–6	Requires supported owner inventory and exact mappings before claiming owner-integrated results—not completion of every owner implementation before any fixture work.
Proof specification §§9–11	Actual-client fixtures, then the same core scenarios through real owners, followed by applicable desktop/artifact evidence.

These are existing proof directions, not implementation authority granted by the documents themselves. 
bounded-evidence
 
bounded-evidence
 
bounded-evidence

The contradiction is in §15.5’s framing, not in the staged architecture

Reconciliation §15.5 still asks whether the next milestone should be “fixture UI or real-owner execution first” and recommends fixture-first as a strategic choice. The attachment’s introductory author correction explicitly disclaims that interpretation. The canonical proof already supplies the sequence. 
bounded-evidence
 
bounded-evidence

The question to settle now is therefore:

Which requirements already have applicable proof, which remaining requirements need an incremental fixture/UI change, and what exact change is authorized next?

That is an implementation-scope decision, not a renewed frontend strategy decision.

Also, Packet A is not Gate A. Packet A corrected the candidate client within fixture scope. Gate A is the supported-owner contract/inventory gate. Packet A’s success did not finish that owner gate or certify all Gate B product-experience coverage. 
bounded-evidence
 
bounded-evidence

2. What Packet A established—and what remains

The proper baseline is neither “Collider already has the intended UI” nor “there is no UI worth retaining.”

Area	Established by the supplied Packet A record	Reuse and remaining limit
Corrected actual client	Exact-version AgentKit 0.2.3 patch; unobserved/accepted local extension; cancellation acknowledgement separated from terminality; generation handling, merge guards, and event-admission checks.	Reuse the corrected candidate. Do not redo the original correction. It is not unchanged upstream compatibility or Substrate behavior.
Artifact reproducibility	Tracked patch, isolated package configuration, clean installation, full installed inventories, and missing/tampered-artifact controls are reported.	Preserve that verification path. A future delta must establish it still uses the same corrected package, not silently resolve an unpatched copy.
Coordinated presentation	Actual injected AgentKitProvider/useAgentRun consumers drive Control Plane/Lane/Tab views from one same-work client; Sticky is view-local; close/reopen does not dispatch or cancel; admission precedes filtering.	These are real reusable proof assets. They do not establish the entire intended user journey, full reconstruction architecture, or main-app integration.
Non-chat interaction	Toolkit DataGrid is exercised; the consumer plan explicitly says to retain its selection, editing, keyboard/focus, provenance, and placement behavior.	Extend the same selected requirement-editing region where needed. Do not introduce a new editor domain merely to demonstrate non-chat reuse again.
Synthetic approvals, queues, and context separation	The implementation record says enabled synthetic approval/queue cases are distinct from the disabled selected-owner profile; invocation input remains current intent plus explicit context.	Preserve applicable tests. The supplied record is not a test-by-test demonstration of every required negative or UI path.
Main application and full UI	The supplied Home source renders a background and “Collider.” Packet A expressly excludes certification of stock full UI and unselected package surfaces.	Main product integration remains work. A coherent fixture workflow may require additional composition, controls, and state presentation even though constituent views already exist.
Real owners and native deployment	Not established by Packet A.	Gate C, native Tauri, distribution, and adoption remain separate.

The implementation record supports the lifecycle and coordinated-view claims, including its express limit on full UI certification. 
bounded-evidence
 Its reproducibility claim is separately recorded, and the actual Home source confirms the minimal main-app presentation. 
bounded-evidence
 
bounded-evidence

Do not infer missing coverage from missing test files in this attachment

The proof README identifies the actual-client tests, unchanged upstream profile, separate accepted-first incompatibility check, shared invariants, type/build checks, and browser entrypoint. But the complete test bodies and current assertion inventory are not supplied. 
bounded-evidence

The bounded inventory needed next is a crosswalk:

Requirement → existing test/assertion → exercised implementation/UI → exact profile → retained evidence → remaining delta.

For example, “coordinated views passed” may establish shared state and detachment, without establishing submission through a unified Composer, a semantic preview, and post-acceptance delivery failure in the same walkthrough. Conversely, those behaviors may already have tests; the agent must inspect before rebuilding them.

Test counts are not that crosswalk. Nor should the inventory expand into another general audit.

3. Is reconciliation §15 a valid incremental delta?

Yes, provided it remains an incremental delta beneath the existing proof specification. Section 15.3 already says its minimum cases do not replace mandatory proof coverage and requires reused-versus-missing test mapping before implementation. That qualification is essential. 
bounded-evidence

I would retain its reference-selection/context-status emphasis and its preference for the isolated host. I would make the following advisory corrections.

Topic	Keep	Necessary correction or coverage check
Readiness estimate	Main app and isolated proof are assessed separately.	Avoid interpreting “must be built” as “none exists.” Qualify each workflow as retained, extended, or not yet demonstrated.
Unified Composer	Current intent plus explicit selections; no implicit transcript forwarding.	A natural-language-only walkthrough does not establish unified Composer with distinct routing, required by proof §9.1. Retain existing routing tests or add the missing representative command/structured route through the same active Composer.
Sticky and Tabs	“No new Sticky region” can bound new construction.	It cannot remove existing Sticky behavior or its required evidence. Preserve view-local Sticky, chronological anchor, same-resource placements, and keyboard-accessible promotion.
Approvals and queues	Real approvals and queue promotion remain excluded from the selected owner scenario.	Their synthetic compatibility/negative tests remain required coverage. An upstream profile with these features disabled is not evidence that their enabled synthetic paths and refusal cases passed.
Missing provider/renderer	Basic unavailable-content fallback.	Preserve the distinction between an unavailable source, stale reference, missing renderer, and unknown capability. Reuse existing cases; add only missing ones.
Context and acceptance	Required/optional references, incomplete acceptance, resolved-versus-delivered states, and failure after acceptance.	Keep these as distinct facts, not one optimistic progress bar. A preview is not admission; resolution is not injection; delivery failure does not erase acceptance.
B05	Existing requirement grid, revisions, stale rejection, and declaration-versus-realization.	Preserve the already mapped no-op, original mutation result versus current value, lost-ACK reconciliation, mixed local/inherited editability, and unknown outcomes. Do not replace that with a fresh boolean-only demo.

The mandatory fixture table explicitly includes unified routing, approvals, queue authorization, capability distinctions, replay, missing-provider behavior, and all three placements. 
bounded-evidence
 The current consumer plan already contains the richer B05 interpretation, including mixed local/inherited rows and original-operation reconciliation. 
bounded-evidence

Excluding a real PTY is not the same as omitting routing

The bounded fixture delta need not add actual shell execution or a full command parser. It can exercise a documented synthetic command/structured-operation route and a natural-language route, with distinct binding and dispatch assertions. Raw-shell capability may remain explicitly unavailable in this slice.

That demonstrates the scoped routing behavior, not the full raw-shell semantics of Foundations §§6.3–6.4 and §17. The latter remain product obligations rather than prerequisites to this entire prototype. 
bounded-evidence
 
bounded-evidence

No new broad scope is needed

The proposed read-only semantic preview and context-status presentation fit the existing context and proof requirements. They do not require a Handbook browser, semantic editor, general resolver, full Inspector, or new execution manager.

Likewise, preserving Sticky coverage does not require finishing its global persistence, compaction, or customization design.

An incremental packet may legitimately finish less than all remaining Gate B work. Its closeout must list what remains. “This delta passed” must not become “Gate B is complete” without the cumulative requirement-to-evidence crosswalk.

4. The smallest next Collider packet

I recommend a coverage-reconciled coordinated-workflow delta within the existing owner-boundary proof, not a replacement plan.

Its initial inventory should be the first bounded activity inside that packet—not another standalone planning program.

Entry conditions

Before implementation is separately authorized, the packet should:

Bind the current local source and exact corrected package/profile; distinguish current target documents from historical proof records.
Map proof §§7–9 and the applicable newer B01–B05 meanings to existing assertions.
Identify only the missing or newly affected product-experience cases.
Record the synthetic reference kind, outcomes, and mapping assumptions needed for those cases. Unselected owner semantics remain labeled proposals.
State the file-level change boundary and preserve the original Gate B control and Packet A evidence.

No complete SDK endpoint, full V1 UI, or closure of all 27 questions is required for that fixture authorization. The actual-owner inventory and exact mappings remain prerequisites before calling the result owner-integrated. 
bounded-evidence

Areas to inspect—not assumed files to rewrite

The relevant source/test set is narrow:

proofs/agentkit-controller-correction/: actual browser entrypoint, provider/client construction, fixture adapters, selectors, projection/placement code, DataGrid callbacks, and tests resolved by its existing scripts.
Its artifact-identity.json, pnpm-workspace.yaml, lockfile, patch, and artifact verifier.
proofs/agentkit-gate-b/: retained unpatched comparison and inherited characterization coverage.
The linked controller-correction regression/provenance/decision records.
Existing proof and consumer lifecycle/boundary mappings.

The README establishes those package and test entrypoints; I do not have the component filenames or assertion bodies needed to specify a truthful exact edit list here. 
bounded-evidence

Construction target

The useful delta is one inspectable workflow linking whichever pieces are not already connected:

Select exact synthetic context and destination → enter current intent → observe semantic/preparation outcomes → submit without losing identity → follow the same synthetic work through Control Plane and Lane views → inspect result and context lineage → manipulate placements without changing work → perform the existing revision-aware dependency edit.

Use the actual corrected client. A synthetic owner harness may independently control acknowledgements, events, failures, and revisions so the test is meaningful; it must not become a second client controller or a production authority implementation.

The proposal already selects this representative journey and explicitly permits deterministic simulated execution. 
bounded-evidence

Exit conditions and evidence

The packet should demonstrate:

The walkthrough and selected negative cases operate through the actual patched client and exercised UI, not only transport tests or manually toggled display state.
Every new case has a source/target reference, a concrete assertion, and an explicit fixture-evidence classification.
Existing applicable controller, type/validator, coordinated-view, placement, approval/queue, and DataGrid regressions remain intact.
Synthetic receipts, authority results, and delivery events are visibly and structurally distinguished from real owner records. Injected is not displayed merely because submission or resolution succeeded.
Reproducible artifact, build, and relevant browser checks use the intended patched package.
The result records cumulative coverage and remaining gaps without claiming Gate C, native Tauri, full B04, V1 completion, or adoption.

Use an incremental section in the existing proof plan/test matrix and append new result/evidence entries. Do not overwrite historical results or create another competing directory of governance templates.

Keep the isolated host for this stage

Yes—the existing isolated proof host is the right place initially. It already provides the corrected package boundary, actual-client consumers, DataGrid, and validation commands. Moving into the main app now would combine workflow learning with the Next-to-Vite migration and product integration. 
bounded-evidence

The tradeoff is that the isolated host postpones discovery of some main-app/native integration costs. That is acceptable only because the proof retains explicit desktop and artifact gates. It is not a reason to defer those gates indefinitely or claim Playwright WebKit establishes native Tauri behavior. 
bounded-evidence

5. Separate synthetic UI work, real owner work, and human decisions
A. Collider fixture work can proceed without implementing Handbook or Substrate

The fixture can supply a supported/proposed reference kind, exact revision, required/optional disposition, a synthetic material identity, and separate resolution/delivery outcomes. Collider then demonstrates correct interaction and presentation of those inputs.

That does not authorize Collider to implement semantic eligibility, derive real permissions, invent authentic Receipts, or decide how production admission works. Reconciliation §15.4 explicitly says no owner runtime change is required merely to run this fixture experiment. 
bounded-evidence

An unresolved Handbook authority choice blocks the corresponding real authorized path, not neutral presentation of synthetic authorized/denied/stale outcomes. If a fixture exercises a specifically unselected authority lifecycle, label that lifecycle as proposed rather than silently approving it.

B. Real B04 requires more than packet generation

The newer reconciliation proposes:

H1: supported composition of exact references, revisions, purpose/action, Scope/Resolution, and requiredness.
H2: owner admission joined to the actual read/render source closure.
H3: complete result/provenance/fidelity/refusal and current-use validation.
S1: Substrate’s trusted binding, retained original material, admission joins, and actual execution delivery.

Those are meaningful refinements of the previous “exact adapter mapping remains open” finding. They suggest that closing B04 likely requires supported owner extensions, not only translating arguments into existing generate_packet calls. But the supplied section is explicitly an uncommitted investigation/proposal; it does not independently prove every candidate change is necessary or approve its implementation. 
bounded-evidence

The minimum real path must preserve full required content or return the required refusal, bind resolved source and rendered bytes accurately, and distinguish historical resolution from authority to use it now. A successful resolver call, matching digest alone, or frontend mutex is not the complete admission/delivery guarantee described in H2/H3.

SDK-first remains conditional on satisfying B04. It does not authorize smuggling missing inputs into packet IDs, rewriting project configuration to simulate a request, bypassing through private/test-support APIs, or forwarding a raw prompt instead.

C. The real human choice is narrower than another product interview

The genuine unresolved choice identified here is the authority population of the real Handbook scenario:

Does it promise continuing local-project use, or is an explicitly narrower initial-only demonstration acceptable?

The reconciliation recommends continuing-project support and H4’s successor-aware route, but labels H4’s scope and authority direction as proposed. SDK-first approval does not independently approve root-binding replacement permissions. An initial-only demonstration must not be silently substituted to avoid H4. 
bounded-evidence

That choice is distinct from provider-native Session Continuity. A continuing Handbook project may still supply freshly reconstructed context for each Invocation.

The following are engineering/owner-contract work, not questions to make the user answer merely because they are unfamiliar: SDK operation placement, exact reference encoding, error representation, admission carrier, retention mechanism, supported caller binding, and adapter file structure.

No evidence here requires reopening Lane/World topology, selecting a storage engine, or deciding the entire Workspace UI before this delta. The file-native project stays separate.

6. Recommended order and safe parallel work

I would use this sequence, preserving the existing packet lineage.

First, reconcile coverage and scope in Collider. Update the interpretation of §15.5, map existing proof assertions, and define the missing incremental cases. This can incorporate the newer target meanings without declaring those targets implemented or owner-approved.

Second, separately authorize the bounded Collider delta. Build only missing composition and tests in the isolated host. Retain passed coverage and the exact client extension. There is no reason to redo the controller correction merely because context presentation has become more precise.

In parallel where separately authorized, close the real owner contracts.

Repository	Safe independent work	Necessary coordination
Handbook	H1–H3 supported composition, semantic admission/read-render integrity, outcomes, and current-use contract; H4 only for the selected authority scope.	Design H3 with Substrate’s S1 consumption/admission protocol so a resolved result cannot become an unsafe reusable permission.
Substrate	Concrete B01/B03 authority, request/acceptance/recovery, then B02 observation/control; B05 writer coordination can proceed independently of semantic resolution.	Preserve exact original material and accepted binding; join Handbook result/current-use validation to actual runtime delivery.
Collider	Fixture workflow, projection/mapping, failure presentation, placement and shared-state behavior.	Treat proposed owner schemas as provisional; update the mapping when owners settle them rather than demanding that owners imitate the fixture.

This parallelism is a dependency observation, not an instruction to dispatch agents. It also remains compatible with carrying out the work serially.

Third, qualify the selected real paths independently. Handbook needs a supported external SDK consumer path with its selected negative tests. Substrate needs the applicable authenticated acceptance/recovery, observation/control, context delivery, and revision-aware mutation implementation and proof. Reuse existing foundations and work already owned by their active packets.

Your reported missing frozen macOS production manifest means that attempted Handbook test run did not execute. Resolve the relevant supported-build prerequisite when that platform’s evidence is needed; do not turn the blocked run into either a pass or proof that Handbook is absent. The manifest and test record are not included here for independent assessment.

Fourth, replace fixture adapters with supported owner mappings and run the selected Gate C scenario. Preserve the user-visible semantics where the real contracts agree, but permit explicit mapping corrections. Compare the same scenarios, not necessarily byte-identical fixture DTOs.

The complete selected scenario remains one supported semantic selection and Linux ephemeral task in coordinated views, plus B05 declaration read/add/remove. A real B05-only or B01–B03 subset may provide useful separately scoped evidence; it does not replace complete Gate C, and new task dispatch cannot bypass mandatory context. 
bounded-evidence

Finally, complete applicable native/artifact/distribution/economic gates and make the adoption decision. Some desktop checks can be brought forward when useful; they do not require waiting mechanically for every owner result. Their evidence must still remain separate, and C-OQ-027 stays Open until an explicit decision. 
bounded-evidence

7. What remains unknown—and the narrowly actionable corrections

The attachment is sufficient for this approach judgment, but not for a complete implementation estimate or test-coverage verdict.

I do not have:

The current fixture/browser test bodies, full regression matrix, or exact composition source. Consequently, I cannot determine which proposed selector, Composer, fallback, or context-state cases are already implemented.
The complete newer reconciliation, including referenced §14.5 negative/race cases and its underlying source evidence. H1–H4/S1 are assessed here as supplied proposals, not independently re-audited source conclusions.
The current patched lifecycle mapping in full, emitted package bytes, or test traces. The implementation record is credible bounded evidence of prior execution, not a fresh run.
Live Substrate/Handbook implementation or supported publication/native status beyond the supplied pins and qualifications.
The main-app package/layout sources. Home’s minimal rendering is directly visible; the Next build description comes from the supplied reconciliation and your qualification.

The resulting corrections are small and actionable:

Correct §15.5’s strategic framing. Replace the reopened fixture-first choice with approval of the specific remaining delta under the existing staged proof.

Qualify §15.1’s construction language. The main UI is a shell, but coordinated views, placements, and the DataGrid are not all missing.

Make §15.3’s cumulative coverage rule operational. Map retained assertions and missing cases before coding; especially preserve unified routing, Sticky, synthetic approval/queue negatives, capability distinctions, fallback, and existing B05 reconciliation.

Keep owner changes explicit and separate. H1–H3/S1 and the remaining Substrate contracts gate the real path; H4 is conditional; storage-engine work and full V1 are not added prerequisites.

Bottom line

The newer findings do not overturn my previous recommendation. They identify the remaining work more accurately.

Collider can continue the originally planned staged experiment, using a corrected and partially demonstrated frontend foundation. It should neither wait for every owner capability before testing interaction nor pretend the remaining work is just connecting two SDK calls.

The next useful result is a narrowly inventoried, inspectable coordinated-workflow delta with retained regressions and explicit synthetic boundaries. The following real-owner stage must still demonstrate the full selected context, acceptance, execution, observation, and dependency-mutation path.

No new fixture-first strategy decision, controller rewrite, provider shortcut, owner implementation, or storage prerequisite is justified by this review. Implementation of the bounded delta still needs its own authorization.
