# Pro consultation response — 2026-10-19

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

Conversation: https://chatgpt.com/g/g-p-689bbd06a0d08191ae4d7e65be750533-substrate/c/6aa9dc25-5ee8-83ea-95d2-8b085b2d27d6

Recommendation: ADJUST—keep the Workspace sprint; make the next operation a version-bound UI preview and selection pass

Build the recognizable fixture-backed Workspace inside Collider’s existing Next.js/React/Tauri shell. Before implementation, show the actual candidate UI regions, choose the small set the sprint will use, and reconcile the active instructions that would otherwise send an agent back toward Vite migration or isolated-harness expansion.

My earlier isolated-host recommendation is superseded by your October 19, 2026 decision. The existing harness remains a source of fixtures, corrected-client behavior, and regression evidence—not the next product UI to expand.

The reviewed amendment already makes the intended result concrete: useful navigation, four distinct Composer routes, typed interactive Surfaces, coordinated views, reconstruction, background awareness, and all retained F1 context behaviors. It expressly rejects a collection of diagnostic controls as sufficient. 
bounded-evidence(1)

The next decision is not “which framework should Collider use?” in the abstract. It is “which actual components implement each region of this particular Workspace, and do you like the result enough to build with them?”

I reviewed the supplied evidence. I have not inspected the complete local component implementation, rendered its demos, or run its checks. An attempted retrieval of other Project conversations failed, so I cannot independently confirm a historical BlockNote decision from those conversations.

1. What we selected—and what we did not

Your recollection has an important basis: we deliberately pursued substantial Agent Native UI reuse, not just a headless controller experiment. But the recorded decisions did not approve every reference UI component or adopt the complete framework.

Point in the sequence	Decision or evidence	What it means now
August 19 AI Elements inventory	Records a curation/testing program and then-current component progress.	Existing assets, not a requirement to preserve their APIs or finish the registry.
Earlier direct clarification in this conversation	You explicitly allowed discarding AI Elements because the ds-skills work had already delivered its intended value.	No sunk-cost preference for the local component set.
Research and adjudication	Preferred candidate: AgentKit controller/protocol plus suitable React regions and selective Toolkit, with adoption deferred.	UI reuse was intentionally included. “Suitable” remained to be established per region.
Gate B and Packet A	Exercised the actual corrected client, React provider/hooks, coordinated views and Toolkit DataGrid. Packet A established a bounded local protocol extension and corrected lifecycle behavior.	Concrete reusable assets and evidence—not proof of stock AgentKitChat, every Composer feature, or a finished Workspace.
October 19 reviewed amendment	Replaces standalone F1 expansion with a product-shaped Workspace; retains F1-01–06 in full.	Build the selected journey, not another intermediate harness milestone.
October 19 newest direct decisions	Existing Next.js shell; reuse applicable fixtures; show components first; reduce optional design-system ceremony.	These control the next selection and implementation plan. No Vite migration or new product host.

The register still identifies substantial interaction and non-chat reuse as the proof target and keeps C-OQ-027 Open. Packet A likewise explicitly limits its result to fixture/client scope and says nothing was adopted. 
bounded-evidence
 
bounded-evidence(1)

Exact UI evidence boundary

Already exercised: the injected provider/hooks path, the selected shared-client views and placement behavior, and the Toolkit DataGrid region. These should be carried forward where applicable.

Declared/exported, but not thereby exercised: AgentKitChat, AgentKitComposer, AgentMessageView/AgentMessagePartView, activity/task groups, approval and connection cards, widget rendering, and error/failure regions. The supplied declarations establish that these candidates exist; they do not certify their behavior with the corrected profile or in the root application. 
bounded-evidence(1)

Not selected by a matching name: Toolkit’s app-shell, context-ui, editor, or chat-history entrypoints. Their actual public components, dependencies, appearance, and suitability still need inspection. 
bounded-evidence(1)

My recommendation is to preview AgentKit/Toolkit’s integrated regions first, then use lower-level pieces where there is a demonstrated mismatch. Do not automatically dismantle them into primitives and rebuild their useful behavior ourselves. Equally, do not make Collider a session-list chatbot simply because that is the reference composition.

2. Sprint-sized region and component matrix

The following is a proposed selection plan, not an adoption record. The requirements come from Foundations §§6–7 and the October 19 Workspace amendment. Candidate names come from the supplied installed exports, declarations, and local filenames. 
bounded-evidence
 
bounded-evidence(1)

Canonical requirement	User-visible region	Reuse candidate	Status	Why / constraints	Preview needed	Acceptance evidence
Workspace structure and useful navigation	Left Navigator, Workspace identity, fixture indicator; Home, Handbook, Lanes, Repositories, Activity	Toolkit app-shell candidates; existing local Radix-based controls; Collider destination composition	Host selected; components proposed	Reuse useful chrome without importing session-centric navigation, authentication, or a second app framework. Required destinations must work; optional destinations may be honest placeholders.	Navigator populated with the actual milestone destinations, selected state, nested Lane entries, and out-of-scope placeholder treatment	Stable destination identity; keyboard navigation; L1/L2 selection; required destinations open useful content; navigation causes no execution effect
Shared Control Plane/Lane Workstream and typed Blocks	One center interaction surface combining input, agent output, shell results, semantic and operation Surfaces	Preview AgentKitChat and individual message/activity/widget regions; local message/tool rendering only where useful; Collider mixed-activity composition	Controller proven in fixture scope; stock regions unqualified	A reference chat region may implement part of the Workstream. It must not make all activity a transcript or create a new client per view.	Integrated reference region versus selective public-region composition, using the same sample content	Same work in CP/L1; L2 exclusion; stable projected identity; typed non-chat Blocks; no duplicate command ownership
Unified Composer with four routes	One input environment with explicit route and destination, context selections, submit/commit controls	AgentKitComposer; Toolkit’s underlying PromptComposer/composer exports after inspection; local prompt-input as an alternative	Selection pending	act/plan is not Collider’s agent/shell/structured/presentation routing. Inspect whether the component permits truthful routing without a duplicate Composer or send path.	Actual Composer variants, route selection, required-context chips, missing destination, busy/disabled/refused states	Four distinct route payloads/effects; finite shell preserves command/stdout/stderr/exit; presentation route has zero owner effects; no transcript forwarding
Read-only Handbook-facing Surface and F1 context meaning	Semantic object/sections, R1/O1 selection, requiredness, revision, material preview, omissions/refusals and lineage	Toolkit context-ui or object-reference rendering; suitable Markdown/content rendering; local sources/citations primitives; Collider context composition	Domain behavior required; renderer proposed	Rendering is not resolution. Selection, material, resolution, acceptance and delivery remain distinct. A generic “context usage” widget does not satisfy provenance.	Exact-content preview; optional omission; required refusal; accepted-but-undelivered; failed/unknown delivery	Every retained F1 assertion, exact material/source/render joins, no required-to-optional downgrade, no display-only Injected, original work identity retained
Persistent execution awareness and explicit cancel	Workspace activity indicator, execution status Surface, request-versus-terminal display, reopen/cancel	Existing corrected-client consumers; suitable AgentKit activity/failure components; local badges/buttons; Collider execution presentation	Client behavior reusable; product composition to verify	Stock status/duration/action components must understand the local extension and must not infer running or terminality.	Accepted/not running, cancellation pending, completion winning, observation unavailable, all execution placements closed	Awareness and cancel remain reachable; ACK is not cancellation; close does not cancel; reopen uses the same work
Substantial non-chat dependency interaction	Requirement grid with detail, editable local declaration, explicit commit, revision/conflict/refusal/readback	Toolkit DataGrid 0.20.3, existing controlled fixture callbacks, Collider declaration mapping	Exercised; recommended reuse	Retain the proven region rather than commission another editor. Declaration is not installation or readiness.	Actual grid, selected-row detail, pending edit, inherited/local state, stale conflict and refusal	Independent mutation/result, expected revision, normalized proposal, explicit commit, same pending edit across placements, correct readback
Inline, Tab, Sticky and reconstruction	Center tabs, inline anchors, existing view-local Sticky, keyboard Open as tab, close/reopen	Existing placement proof assets; local or Toolkit tabs/menu primitives; Collider placement composition	Behavior exercised in isolation; app integration pending	A tab control does not supply Surface identity or reconstruction. Retain Sticky without building a new generalized Sticky system.	Same Surface inline and in a Tab, keyboard promotion, Sticky in one view, closed/reopened state	No implicit request/cancel/delivery; retained resource/edit identity; reconstruction from fixture records/read cut, not hidden mounted React state
Repository as a resource, not the Workspace	Useful read-only repository summary	Local file-tree where needed, cards and code/text primitives; a simpler summary where a tree adds no value	Proposed	The amendment asks for a useful resource summary, not a source editor or complete file manager.	Named repository with location/status/reference information and truthful unavailable fields	Correct resource identity; no editing, shell execution, or Workspace/Repository conflation
Accessible failure and deferred capability presentation	Source/renderer fallback, unavailable reasons, disabled capabilities and optional placeholders	AgentKitErrorBoundary, failure/connection regions; local alert/menu primitives; Collider capability mapping	Candidates declared; exact coverage to inspect	An error boundary’s declaration is not proof of containment. Preserve synthetic approval/queue negatives without enabling real capabilities.	Missing renderer versus missing source; unavailable/unknown state; placeholder with clear scope label	No success-by-fallback; use-time refusal; accessible recovery; no accidental queue, approval, upload, model or provider action
Where custom Collider work is necessary

The custom work is domain composition: destination identities, four-route dispatch, mixed Activity Projection, context lineage, Surface placement and reconstruction, and the joins between those views.

It should not automatically include another reconnect controller, message queue, run reducer, data grid, or copied primitive library.

Use the corrected client where it already owns the relevant client behavior. Use the existing fixture execution owner and transports behind it. Extend only the missing F1 material/refusal/delivery and Workspace behavior. Do not turn the fixture layer into a new generic backend.

BlockNote

Your recollection is unverified historical intent, not disproved. The local search described in the evidence covered only Markdown under docs/ and storybook/; it cannot establish that no decision exists. 
bounded-evidence(1)

I would record BlockNote as a recalled editor candidate to locate, not silently replace it with Toolkit’s editor and not install it as a prerequisite. BlockNote’s current official examples include read-only rendering, but that does not establish a selected Collider version or prior decision. 
BlockNote

For this sprint, semantic content is read-only. A rich-text editor is therefore not inherently required. Workstream Blocks are presentation units, not evidence that the Workstream must be implemented as a block editor.

3. How to show the actual candidates before building

The next deliverable should be a small visual selection packet built from real rendered components—not another prose-only architecture assessment, and not a screenshot of the diagnostic harness labeled “Agent Native UI.”

Show two meaningful implementation directions

A. Integrated AgentKit/Toolkit regions. Show the actual reference chat/message/activity/composer regions together where supported, plus the actual DataGrid and relevant shell controls.

B. The same candidate foundation with selective composition. Show the public message/activity/content pieces and a controllable Composer arrangement where the integrated reference imposes unsuitable behavior. Use local primitives only where they solve a concrete gap.

These are not two new application implementations. They are a comparison of existing region choices. A local AI Elements alternative is worth showing for the Composer or content region when materially useful, not for every button.

The supplied declarations give useful preview variants: the reference chat can place an empty Composer at the center or bottom, disable its Composer region, and control auto-scroll; the Composer exposes model/voice/queue-related options and act/plan mode. These are leads for inspection—not proof that its implementation supports all Collider routes. 
bounded-evidence(1)

Bind each preview to its source

Every screenshot or short interaction recording should identify:

Component/export · package version or local source hash · demo/example source · props/state · stock versus configured versus Collider-composed · unresolved behavior

Show at least the decision-relevant empty, populated, pending, failed and unavailable states. For the Composer, grid, and placement controls, a short interaction is more informative than a static screenshot.

Use existing installed examples and existing stories where available. Running an existing Storybook to inspect a component is useful; completing the Storybook/Figma curation program is not a prerequisite.

Where no runnable example exists, say so. A small preview-only render may be the next separately authorized development action, but it should use the existing app/development arrangement and exact candidate package—not scaffold another product host. It must not quietly become Workspace implementation before you have seen the options.

Public demos are orientation, not version proof

I checked the current official Agent Native documentation. Its React imports now appear under Toolkit paths that differ from the AgentKit 0.2.3 exports in your supplied evidence. That makes current website examples useful for discovery, but unsafe as an unqualified preview of the pinned package. 
Agent-Native
 
bounded-evidence(1)

The current Toolkit catalog also explicitly spans UI whose implementation may belong to Toolkit, Core, or templates. A catalog entry is not necessarily an independently reusable component in Toolkit 0.20.3. 
Agent-Native

I have read those pages, not visually exercised their demos. The local selection pass should supply the actual rendered evidence.

The bounded inventory needed to produce that packet

Inspect the installed package manifests, export maps, declaration barrels, relevant implementation, styles, and version-matched examples. For local components, inspect the actual files and stories, not inventory totals.

Return a complete list of the available public UI exports in the relevant package entries and local regions, with an in-scope shortlist mapped to the table above. Do not require a screenshot and review of every unrelated marketing, sharing, collaboration, or editor component.

The historical counts are different evidence sets: August 19’s inventory reports 48 registry components/32 completed loops, while the ownership note describes 33 AI Elements source items plus 23 primitive source items. Neither is a verified current UI catalog. 
bounded-evidence
 
bounded-evidence

The local pass also needs to check root Next compatibility: client-side boundaries, actual package resolution, required CSS/theme imports, and whether candidate regions create their own client or assume an unavailable backend. The root app must resolve the corrected package instance; an isolated proof’s patch registration does not by itself establish that.

4. Use a lightweight Workspace-sprint mode—not a design-system reset

My recommendation is to reduce new ceremony while retaining functioning infrastructure and essential checks.

Treatment	Scope for this sprint
Retain	Existing theme/tokens where usable; accessible controls; keyboard/focus behavior; safe content/URL handling; exact package/patch provenance; type/build checks; relevant controller and product-journey tests; required repository hooks and checks.
Reuse opportunistically	Existing stories as previews; existing visual tests; ds-skills checks that already run and help catch integration errors; curated components that are genuinely the best fit.
Defer as optional new work	Figma publication, per-component governance story expansion, complete upstream registry curation, full token redesign, design-system migration, exhaustive component documentation, and a general layout builder.
Require explicit approval to change	Enforced import/slot policies, root/nested agent rules, mandatory scripts/hooks/CI, accepted provenance snapshots, and checks that currently gate landing.

The evidence specifically says upstream baseline checks and consumer-contract checks are enforced through just check. Those are not optional ceremonies simply because their names contain design-system terminology. 
bounded-evidence

Before proposing a waiver, inspect what a failing check actually protects. A narrow import-policy amendment may be appropriate when introducing approved Toolkit UI. Deleting the validator, regenerating all baselines, or bypassing hooks is not the equivalent.

Retaining local primitives does not mean protecting AI Elements. Nor should adopting Toolkit mean duplicating every local primitive. Choose one practical primitive/theme integration for each region; leave unused assets intact for now.

The outcome should be a short, sprint-scoped statement of required checks and optional work—not a second design-system policy framework.

5. Minimal documentation and context hygiene
Amend current instructions where they conflict
Location	Minimal proposed treatment
Canonical glossary 00	No terminology rewrite is indicated. Next versus Vite and individual component choices are not new domain terms.
Foundations 01	Preserve the larger V1 requirements. Do not reduce them to this synthetic milestone or imply a read-only preview completes Handbook editing.
Register 02	Update the active target statements in C-OQ-027 and related intake text: this milestone uses existing Next.js/React/Tauri; no Vite migration prerequisite. Keep C-OQ-027 Open and distinguish experiment-specific component choices from final adoption.
Existing implementation/regression plan and acceptance matrix	Resolve the amendment’s now-stale “host/entrypoint unresolved” wording; record the selected existing application host, fixture reuse, component-preview checkpoint, and scoped checks. Keep every F1-01–06 sub-obligation.
Owner-boundary proof specification	Qualify active host-validation and migration assumptions for this milestone. Preserve historical Vite results as historical evidence; retain applicable owner/native/artifact gates.
Existing Substrate reconciliation pointer	Later make a narrow, separately authorized supersession note for its obsolete F1-only/isolated-host order. Its real-owner requirements remain valid.
Current repo entrypoint	Link directly to the current milestone, canonical requirements, acceptance matrix and component-selection record; identify superseded instructions as historical.

The reviewed amendment still leaves host choice unresolved, and explicitly records that Substrate’s older next-order text is not synchronized. Your newest message resolves the host choice; it does not automatically update those files. 
bounded-evidence(1)

Add one component-selection record in the existing docs/agent-native planning area, or a clearly bounded section of the existing plan. It should contain the region matrix, preview references, exact versions, recommended choices, your decisions, integration caveats, and applicable acceptance rows.

Use clear statuses such as candidate, previewed, selected for this milestone, and integrated/tested. None should mean “production adoption approved.”

Prefer navigation and supersession over an archival branch now

Do not make repository cleanup a prerequisite to seeing the UI. An archival branch preserves a historical revision, but it does not solve ambiguous active instructions unless current navigation and rules are corrected. It also is not a substitute for protecting the current uncommitted reviewed amendments.

Before any later archival or removal, classify the material:

Material	Protect or change how?
Imported components, tokens, styles, runtime/build configuration	Treat as active dependencies until an import/build audit establishes otherwise.
Hooks, CI, agent rules, validators, accepted provenance files	Treat as operative policy/tooling, not disposable historical notes.
Gate B/Packet A fixtures, controls, patches and evidence	Retain as regression and provenance assets; do not keep expanding their UI as the product.
Superseded prompts, old strategy recommendations, completed research	Keep retrievable, but remove them from default next-task reading and add precise successor pointers.
Current dirty amendments and untracked material	Inventory and preserve explicitly before any branch/archive operation.

The smallest active context set should be the newest decision summary, applicable canonical sections, the complete Workspace/F1 acceptance matrix, the current component choices, and the source/tests being changed. Load research and historical stop records only when their question arises.

Selective loading is not permission to ignore mandatory repository instructions. Correct conflicting rules through an approved scoped amendment; do not merely omit them from the prompt.

6. The next bounded operation

I recommend one repo-local “Workspace UI preview and selection” pass, followed by your component choices and then a separately authorized implementation sprint.

Concrete deliverable

The pass should return one compact selection packet containing:

Actual component previews for the relevant integrated AgentKit regions, the composable alternative where needed, the existing DataGrid, shell/navigation candidates, and semantic/content building blocks.
A complete sprint-region inventory mapping each requirement to its candidate, exact source/version, already-tested behavior, missing integration, and recommended choice.
A minimal proposed context/check amendment resolving Next.js, the product-host scope, optional design-system ceremony, and stale next-step pointers.

This is not permission to build two competing Workspaces, add a new backend, upgrade candidate packages, change branches, archive files, or waive checks.

Stop condition

Stop the selection pass when every required Workspace region has either an actual version-bound preview or an explicitly identified preview gap, and every human-facing choice has a recommendation. Hand that packet to you before Workspace implementation.

Escalate only a material issue: a candidate cannot use the corrected client/profile, forces automatic effects, needs unapproved backend authority, materially conflicts with the four-route model, or requires an unapproved package/tooling change. A missing optional demo should not trigger a broad framework investigation.

The human decisions at that checkpoint are visual and product-facing: preferred region composition, density, Composer arrangement, navigation treatment, and any deliberately proposed scope change. Export names, CSS dependencies, client boundaries, patch registration and fixture factoring are engineering discovery—not another architecture interview.

Evidence gaps that the pass must close

The attachment does not include the complete original F1 table or its six detailed Workspace test mappings, full current component bodies, all root rules/configuration, or a live rendered catalog. Therefore, I can endorse the selection approach but cannot certify full sub-obligation coverage or name every exact file to edit. The local pass should read those bounded sources rather than ask for another wholesale repository upload.

The amendment requires F1 retention in full and fresh evidence through product controls; it does not allow a simpler assertion or historical test total to stand in for that work. 
bounded-evidence(1)

Bottom line

Proceed toward a useful Workspace in the existing Next.js application—not another harness, another host, or another broad framework evaluation.

Substantial Agent Native UI reuse remains the intended candidate direction. The corrected client and DataGrid provide a strong starting point; stock UI regions still need actual preview and bounded fit checks. Necessary Collider composition should connect those pieces without rebuilding their behavioral machinery.

Keep the owner work separate: synthetic UI success does not close H1–H3, conditional H4, S1, B01–B05, real B04, Gate C, native readiness, or adoption. Those limits are already preserved by the reviewed amendment and need not block component selection. 
bounded-evidence(1)

The immediate milestone is a visible, source-bound component choice you can make—not another document that says the components might be useful.
