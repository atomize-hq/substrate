# Bounded local evidence — 2026-10-19

Advisory context only. Current human decisions in prompt supersede older Vite/isolated-host recommendations. This is selected current working-tree evidence, not full repo access or a complete component inventory. Collider HEAD a491f98724e539a3d7a23aa0661b6b99af79c657. Plans contain uncommitted reviewed amendments. No Project source refresh. Historical test results are not fresh passes.


## Collider/docs/agent-native/canonical/01-collider-v1-foundations.md:248–405
SHA256 full file: 6a32874cd28bbec5347fbdb79b9f63ea4b3f8206357eee4fa4b446e39309b54a

248: ## 6. Primary Interaction Model
249: 
250: ### 6.1 Projection-oriented Workstreams
251: 
252: The primary center interaction model is a Workstream Surface rendering an ordered Activity Projection as Blocks.
253: 
254: A Block MAY represent:
255: 
256: - user input;
257: - shell output;
258: - an agent response;
259: - a Handbook Projection;
260: - a repository browser;
261: - an editor;
262: - a plan;
263: - a diff;
264: - test or gate results;
265: - execution status;
266: - a rich interactive Surface, including one supplied by a Collider-native, Substrate, Handbook, Extension, or MCP App Surface Provider.
267: 
268: When a Block hosts a contributed or externally provided Surface, the Block remains the inline presentation placement, the Surface remains the typed application or view, and the Extension, MCP App, or other Surface Provider remains the attributable provider. Rendering that Surface does not transfer provider-owned state, semantic authority, execution authority, or unrelated host capabilities to the Block or Collider.
269: 
270: The Workstream MAY look conversational, but it is more structured than a conventional chat transcript. Its visible sequence may combine stable Activity Items backed by several authoritative source systems, and one Activity Item may fold several correlated Source Records or revisions.
271: 
272: The Workstream renderer SHOULD receive final Activity Items or bounded Surface models from the Activity Projection pipeline. It SHOULD NOT rediscover business identity, correlation, or fold state by repeatedly scanning raw heterogeneous histories.
273: 
274: A Workstream MUST NOT be assumed to own one independent event history merely because it presents an ordered view. The underlying history, assembler placement, indexing, and persistence architecture remains open.
275: 
276: ### 6.2 Shared Control Plane and Lane infrastructure
277: 
278: The Control Plane and Lane views SHOULD use the same core:
279: 
280: - Workstream Surface;
281: - Activity Projection pipeline;
282: - Block renderer;
283: - Surface host;
284: - Composer;
285: - selection and branching primitives;
286: - Sticky Stack behavior;
287: - Tab Promotion behavior.
288: 
289: Their primary difference is projection and binding posture:
290: 
291: ```text
292: Control Plane
293:   Lane filter: none
294:   Default Lane: none unless explicitly established
295: 
296: Lane view
297:   Lane filter: selected Lane
298:   Default Lane: selected Lane
299: ```
300: 
301: This filtering may remain an internal implementation detail. The user does not need to manipulate raw filter expressions.
302: 
303: `Lane filter: none` means lane-unfiltered, not “display all raw internal telemetry.” The projection may still collapse, summarize, or progressively disclose low-level tool, provider, or internal execution detail.
304: 
305: Control Plane and Lane projections MAY reuse Source Records and Projection Definitions while producing view-specific Activity Items, ordering, visibility, or projected locations. Exact Activity Item identity sharing across those views remains an open decision.
306: 
307: ### 6.3 Unified Composer
308: 
309: The active Workstream SHOULD provide a unified Composer capable of accepting:
310: 
311: - raw shell input;
312: - Collider or Substrate commands;
313: - Handbook-oriented requests;
314: - natural-language agent instructions;
315: - structured actions from an existing Surface.
316: 
317: A user SHOULD NOT need to switch between separate terminal and chat applications merely because one Invocation is shell-shaped and the next is natural language.
318: 
319: The routing and execution semantics of different input dialects remain distinct even when they share one Composer.
320: 
321: ### 6.4 Terminal capability without terminal confinement
322: 
323: Collider MAY provide Terminal Surfaces and raw PTY behavior, but the Control Plane and Workstream MUST NOT be architecturally reduced to a single PTY or shell process.
324: 
325: Normal shell semantics MUST be preserved when the user is operating in raw shell or PTY mode. Rich presentation MAY enhance a command result, but MUST NOT silently falsify the command, output, exit status, Receipt, or execution record.
326: 
327: The desired principle is:
328: 
329: > **Commands may produce applications; applications are not all terminals.**
330: 
331: ### 6.5 Interactive Surfaces inline
332: 
333: Commands, agent responses, owner-system operations, and authorized Surface
334: Providers MAY cause rich typed Surfaces to appear directly inside the active
335: Workstream.
336: 
337: Such a Surface MAY be Collider-native, Substrate-provided, Handbook-provided,
338: or supplied by an authorized Extension or through an MCP App adapter. Its
339: provider path does not change the applicable Surface, placement, owner-authority,
340: trust, or capability boundaries.
341: 
342: A user should be able to:
343: 
344: 1. request or discover an object;
345: 2. receive a useful interactive Surface inline;
346: 3. interact with it without leaving the Workstream;
347: 4. keep it visible in the Workstream's Sticky Stack where supported;
348: 5. expand or focus it where supported;
349: 6. use **Open as tab** when it deserves durable visual attention.
350: 
351: An Extension- or MCP App-supplied Surface participates in the same presentation
352: model as other Surfaces, but only for the placement modes it declares and the
353: host authorizes.
354: 
355: Opening a richer representation MUST NOT require moving the application into the optional right sidebar.
356: 
357: ## 7. Control Plane and Lane Workstream Foundation
358: 
359: Each Workspace SHOULD expose one canonical **Control Plane** as its default opening Workstream Surface.
360: 
361: The Control Plane is the broad Workspace-orchestration entry point. It is not itself a Lane and has no implicit execution environment merely because it is visible.
362: 
363: ### 7.1 V1 presentation
364: 
365: The first V1 Control Plane SHOULD be intentionally minimal:
366: 
367: - a block-oriented Workstream Surface;
368: - a terminal-like Composer;
369: - the ability to submit shell-shaped and natural-language Invocations;
370: - the ability to render rich inline Surfaces;
371: - access to Workspace-level navigation and execution awareness.
372: 
373: A dashboard is not required for the initial version.
374: 
375: Later versions MAY add persistent Workspace-status, active-work, gate, or orchestration applications to the Control Plane without replacing its Workstream model.
376: 
377: ### 7.2 Lane-unfiltered Control Plane projection
378: 
379: The Control Plane SHOULD present a Workspace-relevant Activity Projection with no Lane filter. This allows one coherent view to include eligible Workspace-level activity and eligible activity associated with individual Lanes.
380: 
381: This is a presentation contract, not a decision that all source systems must write into one canonical history.
382: 
383: The Control Plane projection MUST remain eligible for summarization, collapsing, progressive disclosure, permission filtering, and exclusion of low-level internal machinery from the default view.
384: 
385: ### 7.3 Lane-filtered Workstream views
386: 
387: A Lane Workstream Surface SHOULD reuse the same core components as the Control Plane while applying an internal filter for the selected Lane and using that Lane as the normal default binding for new Invocations.
388: 
389: A Lane view is a projection onto the Lane, not the Lane itself. It may be closed while the Lane and its execution continue in the background.
390: 
391: The exact inclusion rules remain unresolved for:
392: 
393: - a Workspace-level Invocation that later dispatches into a Lane;
394: - cross-Lane orchestration;
395: - Activity Items associated with several Lanes or scopes;
396: - activity that begins in one projection and completes in another;
397: - historical Lane rebinding.
398: 
399: These rules belong to the open architecture register and later execution-and-binding contract.
400: 
401: ### 7.4 Resolution posture
402: 
403: The Control Plane begins with a broad Workspace-orchestration Resolution Posture. That posture is a default orientation, not a static context package.
404: 
405: Each Invocation MUST be eligible for new Handbook Resolution appropriate to its actual purpose and target. A narrow request issued from the Control Plane may resolve narrowly, and an implementation request may need to dispatch into a specific Lane.


## Collider/docs/agent-native/canonical/01-collider-v1-foundations.md:745–899
SHA256 full file: 6a32874cd28bbec5347fbdb79b9f63ea4b3f8206357eee4fa4b446e39309b54a

745: ## 15. V1 Product Scope
746: 
747: ### 15.1 Required first coherent vertical slice
748: 
749: The smallest coherent Collider V1 SHOULD include:
750: 
751: 1. opening a Substrate Workspace;
752: 2. a persistent Workspace Navigator with identity-bearing Navigator Items and the highest-level-only default naming rule;
753: 3. Handbook, Lanes, and Repository navigation;
754: 4. a canonical Control Plane Workstream Surface;
755: 5. a unified terminal-like Composer;
756: 6. shell-shaped and natural-language Invocations in the same center environment;
757: 7. typed Blocks and a protocol-neutral Surface host;
758: 8. a Workstream and Activity Projection boundary that produces stable Activity Items through at least one recognized, built-in Projection Definition without requiring one canonical event store or a dynamic projection-registration system;
759: 9. at least one first-party Handbook Surface;
760: 10. at least one first-party Lane or execution-status Surface;
761: 11. inline rendering of eligible Surfaces;
762: 12. opening an eligible Surface as a center Tab;
763: 13. closing and reopening Surface placements without destroying owner-system state;
764: 14. background execution independent of visible Tabs;
765: 15. minimum persistent execution awareness and explicit stop or cancel access;
766: 16. stable Source Record references and Activity Item Business Identities sufficient to incrementally update, reconnect, trace, and reconstruct projected Surfaces;
767: 17. Handbook mutations routed through Handbook-owned capabilities.
768: 
769: ### 15.2 Secondary V1 work
770: 
771: Basic Navigator Layout customization under Section 8.4 is a V1 requirement, although it need not block the first coherent vertical slice. Its delivery must support group creation and renaming, membership and ordering changes, ungrouped items, hiding/restoration, and recovery without altering owner-system state.
772: 
773: Additional secondary work includes:
774: 
775: - first-party configuration and policy, Agent Profile, Agent Harness, environment/dependency, and capability/integration Surfaces through supported owner interfaces;
776: - Inbox, credential-binding status, and broader diagnostics Surfaces;
777: - Sticky Placements and a bounded Sticky Stack;
778: - richer Handbook structured editing;
779: - a rich Repository browser;
780: - source editing Blocks;
781: - saved Tab and presentation layouts across application restarts;
782: - richer Lane status summaries;
783: - a populated `Work` section in the Workspace Navigator;
784: - additional first-party Surface types;
785: - user-facing Workstream branching and reprocessing;
786: - deeper activity and execution-history navigation;
787: - explicit Qualified Session Continuity and the dedicated Sessions destination, including user-chosen prominent or ungrouped access.
788: 
789: ### 15.3 Deferred beyond the first shell
790: 
791: The following are explicitly deferred:
792: 
793: - a full Control Plane dashboard;
794: - pipelines and visual workflow construction;
795: - arbitrary pane docking or general-purpose layout composition beyond basic Navigator Layout customization;
796: - detailed public third-party extension security and isolation implementation;
797: - broad external MCP App support;
798: - production-grade dynamic-extension loading, hot replacement, and composability guarantees;
799: - a full context X-ray or agent debugger UI;
800: - a complete internal execution-tree visualization;
801: - detailed right-side Inspector behavior;
802: - Storybook or Open Design integration;
803: - comprehensive design-system management;
804: - user-authored UI layout commands;
805: - multi-Workspace Project orchestration.
806: 
807: These deferrals do not include basic Navigator Layout customization or the dedicated Sessions destination. Their staged V1 scope remains in Section 15.2; default organization remains non-Session-centric while user-selected Session navigation is supported.
808: 
809: Deferred features MUST layer onto the foundational model rather than redefine the ownership and lifecycle boundaries established here.
810: 
811: ## 16. Foundational Invariants
812: 
813: The following are the primary Collider V1 foundations:
814: 
815: 1. **Workspace is the canonical top-level Substrate configuration and policy boundary.**
816: 2. **Workspace is not a synonym for Repository or Project.**
817: 3. **Handbook owns semantic truth; Substrate owns execution truth; Collider owns projection, presentation, and interaction.**
818: 4. **Collider MUST NOT become a competing Handbook, Substrate, MCP App, shell, or provider authority.**
819: 5. **A Lane is independent of every Tab or Surface that displays it.**
820: 6. **A Workstream is an interaction context and ordered Activity Projection, not an execution environment or assumed independent event log.**
821: 7. **The Control Plane is a Workspace-level Workstream Surface, not a Lane or a single PTY.**
822: 8. **The Control Plane SHOULD be Lane-unfiltered, while Lane views SHOULD reuse the same core Workstream components with an internal Lane filter.**
823: 9. **Lane-unfiltered does not mean raw or unrestricted internal telemetry.**
824: 10. **The center Workstream is the primary place where terminal, agent, Handbook, and application interaction converge.**
825: 11. **Commands MAY produce interactive applications rather than only text.**
826: 12. **A durable or reconstructable Block SHOULD be traceable to sufficient Source Records, references, or owner-system objects without becoming their authority.**
827: 13. **The UX may present one coherent activity flow without requiring one canonical Workspace event store.**
828: 14. **A Surface may have inline, Sticky, and Tab placements.**
829: 15. **A Sticky Placement is local presentation state and preserves the chronological Block.**
830: 16. **Open as tab creates focused presentation; it is distinct from making an item Sticky.**
831: 17. **Closing a Tab, removing a Sticky Placement, or unmounting presentation MUST NOT implicitly stop work or delete durable state.**
832: 18. **Background execution requires persistent user-visible awareness.**
833: 19. **Each Invocation may be resolved anew; a Workstream Activity Projection is not the sole context authority.**
834: 20. **The default interaction model is resolution-reconstructed; Qualified Session Continuity is explicit and optional, and remaining in one Workstream MUST NOT imply it.**
835: 21. **Projection is not automatically canonical or editable Artifact source.**
836: 22. **Handbook mutation MUST use Handbook-owned semantics.**
837: 23. **Collider's Surface model is protocol-neutral; MCP is one possible provider mechanism.**
838: 24. **In-process composition and capability mediation are not security isolation; untrusted executable extensions require an actual Isolation Boundary.**
839: 25. **Permission to render or contribute UI does not imply host execution authority or unrelated host capabilities.**
840: 26. **Automatic cleanup and reversion claims are limited to Mediated Effects; a Compensating Action is not exact rollback.**
841: 27. **Crash and non-graceful failure safety MUST NOT depend solely on graceful extension cleanup.**
842: 28. **Raw terminal and shell semantics remain available and truthful.**
843: 29. **The right-side Inspector is additive, hideable, and nonessential to the first coherent V1.**
844: 30. **Execution-instance and agent-topology vocabulary remains provisional until separately resolved.**
845: 31. **Durable meaning and execution history survive independently from the UI currently displaying them.**
846: 32. **Dynamically reconfigurable Extensions SHOULD aim for Spatiotemporal Composability, but the current architecture does not claim that guarantee.**
847: 33. **Temporal Composability claims are limited to mediated and attributable contributions with valid cleanup, recovery, or inverse behavior evaluated under a declared recovery criterion; External Effects and Compensating Actions do not imply exact reversion.**
848: 34. **Spatial Composability is driven by declared requirements and provisions, reactive Extension Lifecycle behavior, and orderly dependent teardown before provider withdrawal where required.**
849: 35. **Extension access to host functionality SHOULD use owner-mediated and attributable Capabilities where practical rather than undocumented ambient globals.**
850: 36. **Required and Provided Capability declarations SHOULD inform Extension eligibility and lifecycle while providers retain authority for the state and services they expose.**
851: 37. **An Extension Context and Extension Context Mediation MUST NOT become a cross-system source of truth, replace Handbook Resolution or Substrate Context Packs, or substitute for an Isolation Boundary.**
852: 38. **Cross-process Capabilities MUST represent asynchronous, availability, failure, cancellation, timeout, retry, and provider-loss semantics honestly where they apply.**
853: 39. **Collider-owned layout declares valid Extension Points; authorized Contribution Providers contribute through scoped host-approved interfaces.**
854: 40. **Extension Points compose Surfaces and controls without replacing the Surface abstraction or inline, Sticky, Tab, overlay, and Inspector presentation semantics.**
855: 41. **UI Contributions remain provider-attributable, lifecycle-bound, and owner-system-bound; permission to contribute UI does not grant runtime or semantic authority.**
856: 42. **Host accessibility, fallback, and recovery behavior MUST remain possible when a UI Contribution is absent, removed, replaced, or fails.**
857: 43. **Workstream renderers SHOULD consume final Activity Items or bounded Surface models rather than repeatedly scan raw heterogeneous source histories.**
858: 44. **Activity Items MUST have stable Business Identity independent of visual list position; that identity may be owner-supplied or deterministically projection-derived, and an Activity Item MAY fold multiple correlated Source Records or revisions while retaining inspectable provenance.**
859: 45. **Projection assembly SHOULD be deterministic and replayable where source semantics permit, and incremental append or older-history loading SHOULD preserve unaffected projected identity and state where possible.**
860: 46. **Projection Definitions, Activity Assemblers, and Projection Indexes own view-specific assembly rules and derived state, not semantic, execution, or Source Record authority.**
861: 47. **Different Workstream projections MAY assemble the same Source Records differently; deterministic presentation order does not establish one authoritative total Source Record order, and this projection architecture does not select one canonical Workspace event store or require shared Activity Item identity across views.**
862: 48. **A Workstream may span zero, one, or several Qualified Sessions; Session History may source Activity Projections without becoming the Workspace's canonical history or Workstream identity. Default navigation is not Session-centric, but users may prioritize Qualified Sessions in their own Navigator Layout.**
863: 49. **Resuming a Qualified Session MUST NOT silently carry forward stale Handbook Scope or Resolution, Lane binding, Substrate policy, Capability Grants, permissions, provider availability, or model identity and configuration without applicable revalidation.**
864: 50. **Purely local Collider presentation changes are Presentation Actions, not Invocations; a Surface action becomes an Invocation only when it requests owner-system behavior.**
865: 51. **The one-word naming rule applies only to Collider-shipped labels at the highest level of the default navigator; descendant labels, user names, user-promoted items, and canonical architectural terminology are not constrained by it.**
866: 52. **Users control supported Navigator Group names, membership, ordering, visibility, and ungrouped placement; stable Navigator Item identity and owner-system authority do not change with that arrangement.**
867: 53. **Collider declares eligible navigation contributions and safe composition rules; providers MUST respect user layout choices, and customization MUST preserve required execution awareness, cancellation, and recovery.**
868: 54. **Removing an item from navigation is not disabling a capability, deleting an object, changing policy, resuming a Session, or stopping execution.**
869: 55. **Environment Dependency configuration feeds Substrate's World dependency tooling; Repository/package dependencies and Extension capability dependencies remain distinct.**
870: 56. **A unified configuration Surface does not merge native harness, Substrate integration/policy, Agent Profile, or Collider presentation authority, and visible or enabled capability does not imply authorization or readiness.**
871: 57. **Inbox read or dismissal state does not approve requests, satisfy gates, or resolve owner-system obligations; authoritative actions use the relevant owner boundary.**
872: 
873: ## 17. V1 Acceptance Scenario
874: 
875: The foundational direction is proven when a user can complete the following flow:
876: 
877: 1. open a Substrate Workspace in Collider;
878: 2. see its durable Handbook, Lane, and Repository structure in the Workspace Navigator;
879: 3. begin in the canonical Control Plane Workstream Surface;
880: 4. submit both a shell-shaped command and a natural-language request through the same Composer;
881: 5. see a coherent Lane-unfiltered activity view without requiring every provider to share one canonical event store;
882: 6. request a Handbook Artifact and receive a typed inline Surface rather than being forced into its backing file;
883: 7. use **Open as tab** on that Surface through drag, menu, or keyboard interaction;
884: 8. open a Lane Workstream Surface that reuses the same core Workstream components with a Lane-filtered projection;
885: 9. initiate Lane-bound work;
886: 10. close the Lane-related Tab while the underlying execution continues;
887: 11. retain visible Workspace-level awareness that background work remains active or needs attention;
888: 12. reopen the Lane or execution Surface and reconstruct its current state from durable owner-system records or references;
889: 13. explicitly stop or cancel work without confusing that action with closing presentation;
890: 14. make any Handbook change through a Handbook-owned mutation path;
891: 15. complete the flow without depending on a right-side Inspector, a dashboard, a giant chat-history list, or file tabs as the primary information architecture.
892: 
893: Before the V1 navigator capability is complete, a secondary acceptance path MUST demonstrate creating and renaming a group, moving an eligible item between groups and to the ungrouped root, reordering, hiding and restoring it, and restoring defaults. A multiword descendant or user-promoted item must keep its valid display name, and its identity, target state, permissions, and execution behavior must remain unchanged. Required execution awareness and controls must remain reachable.
894: 
895: As expanded destinations are delivered, acceptance must verify that environment edits use Substrate World dependency tooling, harness/profile and enablement changes reach the correct owner, Inbox dismissal does not resolve authoritative work, and users may prioritize Sessions without implicitly resuming one.
896: 
897: A later secondary slice may additionally prove that a Block can be made Sticky in one Workstream projection without becoming Sticky in another projection or being removed from chronological history.
898: 
899: If this flow is coherent, the initial Collider shell has established the intended product foundation.


## Collider/docs/agent-native/canonical/02-collider-v1-open-architecture-questions.md:364–446
SHA256 full file: dfac1ea399908053e40ad647042067060d8585cc05d0f9f9f2f314e9b24a9cff

364: ### C-OQ-027 — Frontend foundation and reuse strategy
365: 
366: **Status:** Open\
367: **Decision:** Determine whether Collider should adopt, compose, adapt, selectively own, fork, or independently implement its frontend foundation, including presentation components and interaction controllers, over the existing Substrate/Handbook/UAA owner boundaries.
368: 
369: **Adjudication disposition:** **ADJUST — retain the shortlist; defer adoption.** The corrected report establishes a credible frontend-reuse opportunity, but documentation and source inspection do not establish integrated lifecycle truth, a supported Substrate GUI boundary, deployment compatibility, published-artifact provenance, or measured economics.
370: 
371: **Provisional proof shortlist:**
372: 
373: 1. public AgentKit controller/protocol plus suitable AgentKit React regions, selective Toolkit, one owner-routed bridge, and Collider-defined Workstream/Control Plane/Lane behavior;
374: 2. headless-first AgentKit when lifecycle mapping succeeds but the reference React composition does not;
375: 3. substantial Toolkit reuse with another single controller when AgentKit lifecycle translation costs exceed demonstrated value;
376: 4. a bounded coherent frontend fork when explicit controller/composition ownership is cheaper and clearer than a translation-heavy public adapter;
377: 5. a broader owner-preserving framework/fork remains eligible but unselected pending retained-value, distribution, and maintenance evidence;
378: 6. Collider-native implementation remains a fair comparator rather than the automatic fallback after any package mismatch.
379: 
380: **Must evaluate:** all previously listed adoption, composition, ejection, fork, native, and hybrid options against the same desired product; the actual public and private package seams; controller/protocol behavior; published artifacts and notices; Tauri/Vite/WebView deployment; and the retained value of one substantial agent-interaction region plus one non-chat region. In addition, prove the actual controller semantics for pending versus owner-accepted work, accepted/queued/running representation, lost acknowledgement, exact request/receipt identity, detach versus cancel, cancellation acknowledgement versus authoritative terminal effect, completion/cancel races, subscription interruption versus terminal completion, approval protocol segmentation and exact owner approval identity, automatic queue promotion and current authorization, repeated identical content with distinct identities, transcript versus reconstructed model context, capability-state honesty, source-cursor to projection-sequence mapping, coordinated Control Plane/Lane/inline/Sticky/Tab views under one behavioral owner, renderer fallback, and restart/replay behavior. Assess sidecars, private APIs, and fork size as deployment/economic tradeoffs unless they require forbidden authority or privilege changes.
381: 
382: **Current constraints:** AI Elements curation is disposable, with no preservation preference or sunk-cost penalty. Tauri + React + TypeScript + Vite is the target. Substrate and Handbook retain their existing authority; UAA/provider execution is not to be recreated in a parallel TypeScript backend. Framework code or owned source may implement Collider-defined roles; accepted semantics do not mandate bespoke implementations. AgentKit thread/run/controller identities do not become Workstream, Qualified Session, Invocation, Receipt, Source Record, Activity Item, or owner terminal truth by name. Default Agent-Native Core authority over database, auth, execution, persistence, or deployment is not selected. A sidecar or fork is not rejected merely for existing, but any authority-displacing design, false acceptance/terminal reporting, or UI-owned execution lifetime is a hard semantic failure. No Agent Native, AgentKit, Toolkit, BB, Cordis, controller, datastore, sidecar, fork, or dependency is adopted by this question.
383: 
384: **Required proof evidence:**
385: 
386: - **Contract gate:** exact supported owner entrypoint and production call path; versioned schemas; authority and policy checks; field-by-field command/receipt/work/cursor/approval/cancel/session mapping; unknown fields remain unknown.
387: - **Fixture/client proof:** the actual public AgentKit client/controller, not only a transport-shaped double; full relevant transport conformance plus Collider-specific lifecycle, identity, queue, approval, sequence, coordinated-view, and substantial non-chat Surface cases. Fixture success remains fixture evidence.
388: - **Owner-integrated proof:** the same scenario through the supported owner boundary and declared real execution path, preserving a traceable join from user intent through owner acceptance, source observations, projected events, and authoritative terminal/approval records; demonstrate view/window release and restart without implicit provider cancellation or duplicate dispatch.
389: - **Deployment/artifact/economics proof:** exact package artifacts, notices, provenance and applicable rights; Vite/Tauri/CSP/accessibility behavior; production dependency graph versus emitted assets; startup, memory, event-burst, replay, focus/keyboard, cleanup, maintenance, sidecar, source-divergence and upgrade costs.
390: 
391: **Decision boundary:** this adjudication selects a proof direction, not implementation. Only a separately authorized, passing exact-path proof plus applicable artifact/licensing evidence can support a later adoption or dependency decision. Research, adjudication, proof, and canonical amendment remain separate steps.
392: 
393: ## 4. Research Clusters and Preliminary Dependencies
394: 
395: The following grouping is non-normative research-ordering metadata. It does not resolve an item, create implementation authority, or claim that every dependency is complete. Per-question `Depends on` and `Blocks` fields SHOULD be added only when a bounded research packet establishes them with sufficient confidence.
396: 
397: | Cluster | Questions | Preliminary dependency or gating relationship |
398: |---|---|---|
399: | **History and projection** | `C-OQ-001`–`C-OQ-007`, `C-OQ-022` | Establishes source, identity, ordering, assembly, replay, and reconstruction boundaries needed by later Workstream persistence and history-sensitive UX contracts. |
400: | **Qualified Session continuity** | `C-OQ-008` | Depends partly on owner-history and Activity Projection findings and coordinates destination behavior with `C-OQ-025`; it MUST NOT resolve Workspace history or make navigation imply continuation. |
401: | **Presentation placements** | `C-OQ-009–010` | Can proceed against stable Surface and Activity Item identities; persistence and restoration details may depend on history/projection findings. |
402: | **Execution topology and context lineage** | `C-OQ-011–012` | Requires current Substrate implementation truth and later gates the detailed Execution and Binding contract. |
403: | **Reference-architecture evaluation** | `C-OQ-013` | May inform experiments, but does not gate canonical Collider terminology or select a dependency. |
404: | **Extension trust, effects, and lifecycle** | `C-OQ-014–017` | Trust and isolation boundaries constrain every later dynamic-extension mechanism; recovery criteria gate truthful composability claims. |
405: | **Capability composition** | `C-OQ-018–019` | Depends on owner and trust boundaries from `C-OQ-014` and dependency-lifecycle findings from `C-OQ-017`. |
406: | **UI extension composition** | `C-OQ-020–021` | Depends on trust and capability constraints from `C-OQ-014`, `C-OQ-018`, and `C-OQ-019`; it does not replace Surface or placement semantics. |
407: | **Environment realization** | `C-OQ-023` | Starts from existing Substrate World dependency tooling; coordinates Lane/environment changes with the execution contract rather than creating a Collider resolver. |
408: | **Workspace execution setup** | `C-OQ-024` | Requires owner-grounded configuration truth and coordinates environment, capability, policy, and Session behavior with `C-OQ-023`, `C-OQ-014`, `C-OQ-018`, and `C-OQ-008`. |
409: | **Navigator customization** | `C-OQ-025` | Builds on Presentation Actions and UI-contribution boundaries; informs the Workspace Shell contract without requiring every destination editor first. |
410: | **Inbox and notifications** | `C-OQ-026` | Depends on the source owners' acknowledgment and resolution semantics; coordinates discoverability with `C-OQ-025` without selecting a universal history. |
411: | **Frontend foundation and reuse** | `C-OQ-027` | Research and adjudication are complete enough to shortlist proof candidates, but adoption remains Open. The next gate is the non-authorizing owner-boundary/proof specification and separately authorized staged proof; coordinates with history/projection, continuity, execution, capability, UI, setup, and Inbox questions without requiring them all to close first. |
412: 
413: ## 5. Recommended First Research Packet
414: 
415: The first research packet should address **C-OQ-001 through C-OQ-007 together with C-OQ-022**, because they share one architectural boundary.
416: 
417: It should inspect current implementation truth for:
418: 
419: - Substrate invocation, JSONL, Receipt, process, tool, Lane, and execution records;
420: - Handbook Artifact history, Projection, Snapshot Memory, deltas, Evidence, and SDK boundaries;
421: - MCP App object and lifecycle semantics used or contemplated by Collider;
422: - shell and PTY capture already available in Substrate;
423: - Collider and Tauri persistence, restoration, and Surface-host constraints.
424: 
425: The packet should return:
426: 
427: 1. a source-of-truth inventory;
428: 2. required user-visible queries, Projection Definitions, and Activity Item identity requirements;
429: 3. payload-copy versus reference rules;
430: 4. candidate Activity Assembler and Projection Index placements with failure modes;
431: 5. append, prepend, pagination, replay, resynchronization, gap-repair, snapshot, and compaction requirements;
432: 6. candidate history architectures with failure modes;
433: 7. a recommended option or a reason to preserve a bounded hybrid;
434: 8. migration and incremental-delivery implications;
435: 9. the exact glossary, foundations, workspace-shell, and execution-contract edits required by the decision.
436: 
437: ### 5.1 Current frontend-evaluation sequence
438: 
439: The corrected Agent Native evaluation and bounded adjudication are complete as advisory evidence. `C-OQ-027` remains Open.
440: 
441: 1. Preserve the corrected report, adjudication decision coverage, adjudication check record, exact pins, and missing-original-ledger limitation as research provenance. Do not reconstruct the missing ledger or promote the new check record to that status.
442: 2. Use `agentkit-owner-boundary-proof-spec.md` to ground the exact supported Substrate/Handbook/UAA owner surface and field-by-field mapping. Contract readiness is not runtime acceptance evidence.
443: 3. After separate authorization, run the staged proof: actual-client fixture proof, owner-integrated proof, and deployment/artifact/economics proof. Keep their results separate.
444: 4. Draft stable implementation-neutral Workspace Shell behavior in parallel where sufficiently settled. Do not silently choose controller identity, history storage, or execution truth while doing so.
445: 5. Use the owner inventory and exact-path evidence to complete supported Execution and Binding contracts. Where a real owner path is unavailable, record the capability as unavailable or unproved rather than creating a candidate-shaped workaround.
446: 6. After the proof, explicitly decide dependency/composition/source-ownership/fork/native strategy. Amend glossary or Foundations only when an accepted product-semantic decision actually changes.


## Collider/docs/agent-native/controller-ownership-plan/implementation-and-regression-plan.md:407–590
SHA256 full file: 9958d52648a443ef05f4e14a46d7d2f31bf08feedc888192cfc027b01d7cb3fc

407: ## 2026-10-19 amendment — minimal fixture-backed Workspace
408: 
409: **Current next milestone: a recognizable, product-shaped minimal Workspace,
410: backed by explicit fixtures.** Newer human direction supersedes implementing F1
411: as a separate extension of the isolated diagnostic harness. F1 is carried into
412: this milestone's acceptance tests, not a prerequisite packet to finish first.
413: Completion of all Gate B is also not an entry prerequisite. This is a documentation
414: amendment and proposed implementation contract, with no implementation authorized
415: or performed by this task.
416: 
417: The product basis is the [glossary](../canonical/00-collider-v1-glossary.md),
418: [Foundations §§6–7, 15 and 17](../canonical/01-collider-v1-foundations.md#6-primary-interaction-model)
419: and [open register §5.1](../canonical/02-collider-v1-open-architecture-questions.md#51-current-frontend-evaluation-sequence).
420: They permit settled, implementation-neutral Shell behavior alongside staged proof.
421: The bounded claim is **a user can navigate a synthetic Workspace, direct distinct
422: kinds of work through one Composer, use typed interactive Surfaces, observe the
423: same fixture work in CP/Lane/tab, close and reconstruct placements, retain
424: background awareness, and explicitly request cancellation while all retained F1
425: context distinctions remain truthful.** A collection of diagnostic controls,
426: status labels or chat messages alone does not meet that claim.
427: 
428: ### Workspace journey and bounded construction
429: 
430: Open one named fixture Workspace with a persistent left Navigator and a center
431: Control Plane Workstream. A visible, persistent fixture-mode label identifies
432: simulated execution and synthetic semantic/context evidence. The Navigator has
433: identity-bearing destinations for Home (the canonical Control Plane), Handbook,
434: Lanes, Repositories and Activity; root default labels are one word. Handbook opens
435: semantic objects by fixture owner-qualified identity rather than forcing backing
436: file navigation. One Repository entry opens a useful read-only resource summary;
437: it is not an editor or the Workspace itself. Lanes opens L1, with L2 as a filtering
438: control. Activity and a Workspace-wide indicator remain reachable when execution
439: placements are closed. These are minimum useful destinations, not empty menus.
440: 
441: The center shares Workstream/Block/Surface/Composer behavior between the
442: Lane-unfiltered Control Plane (initial default Lane absent) and L1 (filter L1,
443: new-request default L1). Typed Blocks distinguish current input, shell result,
444: agent result, semantic Surface, execution status and non-chat operation/result;
445: inspectable fixture source references and stable view-local Business Identities
446: are required. At least one bounded built-in Projection Definition has an explicit
447: identity/version and deterministic fold/reconstruction rules for these fixture
448: records. This does not select a canonical Workspace log, storage engine, dynamic
449: registration model, global controller or cross-view Activity Item identity policy.
450: 
451: A user requests the fixture Handbook object through the same Composer, receives
452: an inline semantic Surface, selects exact required R1 and optional O1 and previews
453: known material and provenance. The Surface supports useful interactions: inspect
454: sections/source revision, select/deselect supported references and requiredness,
455: and inspect permitted omission or refusal reasons. It stays read-only as semantic
456: content; arbitrary edited Projection text never becomes Handbook source. The user
457: then enters a natural-language task, chooses or inherits an explicit valid L1
458: fixture destination, submits, observes acceptance independently of running and
459: context delivery, and inspects the same joined work in CP, L1 and an execution Tab.
460: The F1 tests include all refusal, omission, failed/unknown delivery and edit/default
461: variants, not only this successful walkthrough.
462: 
463: The same Composer also accepts a bounded shell-shaped fixture command and a typed
464: Workspace requirement action from a substantial interactive non-chat Surface.
465: The latter provides row selection/detail, a proposed editable declaration, explicit
466: commit and fixture readback with revision/conflict/refusal behavior. A local
467: Presentation Action opens that Surface as a Tab with no owner effect. The
468: [acceptance matrix](../packet-2/proof-scenario-and-evidence-matrix.md#2026-10-19-minimal-workspace-acceptance-specifications)
469: defines exact route payloads, results, barriers, effects and negatives; clicking a
470: non-agent route that merely changes placement is not completion of owner routing.
471: 
472: Open semantic and execution/non-chat Surfaces as center Tabs using an action and
473: keyboard path. Retain inline chronological anchors and bound resource/pending edit
474: identity; preserve any existing view-local Sticky behavior without a new Sticky
475: system. Close all execution placements and switch away from L1 while fixture work
476: continues behind an independent gate. Persistent Workspace awareness still shows
477: that exact work and a path to reopen/cancel. Reopen from Navigator/Activity and
478: reconstruct the declared fields from retained fixture records/read cut, not saved
479: React component state. Explicit Cancel then shows request/ACK separately from
480: terminal evidence; completion may win. No close, navigation, preview, placement
481: or reconstruction operation submits or cancels work implicitly.
482: 
483: ### Unified Composer contract for this milestone
484: 
485: One input environment may use explicit dialect selection; this plan does not pick
486: a production parser or infer a route from ambiguous text. Tests must show what
487: route is selected before any effect and must reject unsupported/ambiguous inputs.
488: The fixture contract has four distinct routes:
489: 
490: | Route | Required bounded behavior | Limit retained for later work |
491: | --- | --- | --- |
492: | Natural-language agent | Current intent and frozen explicit selections/purpose/posture/destination cross the preparation and actual-client send boundary; independent acceptance, running, delivery and result evidence. No prior transcript or implicit Qualified Session continuation. | No real agent, model call, supported owner API or full B04 claim. |
493: | Shell-shaped fixture command | Preserve literal command, arguments, destination, stdout, stderr and exit status from independent fixture records, including nonzero exit. Result is a typed shell Block, not a paraphrased agent answer. | Clearly labeled finite simulated dialect. Arbitrary raw shell/PTY, pipelines, interactive input and real process effects are unavailable; full canonical raw-shell/PTY capability remains open. |
494: | Structured owner action | Typed fixture semantic-object read with exact query/result identity, or one-item Workspace declaration proposal from the non-chat Surface, version/revision/key and desired value; explicit commit yields an independent fixture operation/result and authoritative-within-fixture readback. | No real Substrate write, install/provision or full B05 transaction/recovery claim. |
495: | Local Presentation Action | Open/focus/close a bound Surface or change a local placement; observable placement/focus change, zero begin/accept/agent/shell/mutation/cancel/delivery effects. | No Invocation, fabricated receipt or owner authorization from presentation. |
496: 
497: Control Plane read-only semantic/Workspace inspection may remain Lane-neutral.
498: Environment-touching agent or shell work must have an explicit selected/established
499: fixture Lane/World before effect; absent/ambiguous/wrong binding refuses. L1 defaults
500: new eligible work to its declared L1/World reference only. Structured Workspace
501: configuration targets Workspace scope explicitly, not an arbitrary worktree.
502: Defaults and view filters are not permissions. Current support, authorization and
503: readiness remain separate; test the fixture's use-time refusal after an eligible
504: control was rendered. Historical work retains original destination after defaults
505: change. This subset does not assert genuine B03 association fencing.
506: 
507: ### F1 disposition and required proof
508: 
509: **F1-01, F1-02, F1-03, F1-04, F1-05 and F1-06: RETAINED in full. No behavioral
510: sub-obligation is deferred.** Their original table above remains the source of
511: retained requirements. The [six detailed Workspace test mappings](../packet-2/proof-scenario-and-evidence-matrix.md#retained-f1-workspace-test-mappings)
512: separate every assertion into stimuli, independent expected fixture data,
513: barriers/oracles, visible joins/effect counts, negatives and evidence classes.
514: The changed host shape/order is the superseded part. Any future proposed semantic
515: reduction must return through the task owner to META for disposition; implementing
516: an easier assertion or marking an untested requirement covered is not permitted.
517: 
518: Planned proof requires both actual public corrected-client assertions and browser
519: interaction through the product controls. Retain one behavioral owner per exact
520: work across placements, correct source admission before filtering, original input
521: capture, explicit missing/absent/unavailable states and source-cursor versus local
522: sequence. Material/source/render identity, selection, preparation/resolution,
523: execution acceptance and delivery evidence are distinct. Fixture labels must remain
524: visible on Tabs, lineage, shell output and result views, not only a landing page.
525: 
526: Preserve the corrected Packet A client/profile, original diagnostic harness source,
527: all historical results, original Gate B characterization and corrected-invariant
528: negative controls. Their preservation does not decide how to reuse or integrate
529: them in the Workspace. That step-3 evaluation is outside this amendment. Fresh
530: implementation proof must bind its actual exercised client/artifact, selected host
531: and affected regressions; historical test totals and source presence are not new
532: passes. Do not alter old tests/assertions to manufacture coverage.
533: 
534: ### Remaining scope and implementation-entry choices
535: 
536: This milestone is a synthetic subset of Foundations §15.1 and the §17 journey,
537: not completion of the first coherent V1. Workspace/Lane/Repository/semantic data,
538: read cuts, acceptance, delivery, cancellation and shell/requirement effects are
539: fixtures. Reconstruction is within a declared retained fixture lifetime/cut;
540: no disk, frontend-process, native-window or owner-service restart guarantee follows.
541: The [matrix's canonical scope account](../packet-2/proof-scenario-and-evidence-matrix.md#canonical-scope-and-cumulative-requirements-left-open)
542: keeps the unproved canonical and mandatory proof requirements explicit.
543: 
544: The later implementation assignment must choose its host/entrypoint, exact edit
545: fence, fixture retention mechanism, deterministic test binding and applicable
546: checks on freshly frozen bytes. A product-shaped fixture mode can be specified
547: now without selecting main-app migration, Next-to-Vite work, production adoption,
548: final controller/history architecture, or concrete Handbook/Substrate APIs. These
549: entry choices are unresolved and do not block completion of this documentation
550: amendment. The historical nine-file F1 fence is not authority for the larger
551: Workspace milestone. No new implementation path or reuse/integration decision is
552: selected here, and no Step 3 work is begun.
553: 
554: Real owner composition remains H1–H3/S1 in
555: [Substrate reconciliation §13](../../../../substrate/docs/guidance/2026-10-17-handbook-b04-reconciliation.md#13-supported-adapterowner-binding-qualification--2026-10-18):
556: supported exact semantic input/materialization, authorized coherent read/render
557: closure, complete results/current-use admission, original-material retention,
558: B01/B03 acceptance and actual runtime delivery/recovery. Conditional H4 remains
559: required for a selected continuing-local-project population; initial-only cannot
560: silently replace it. File-native storage remains separate. B01/B02/B03/B05,
561: installed Linux security/runtime readiness and owner acceptance remain actual work.
562: 
563: **External alignment outstanding:** the read-only Substrate reconciliation
564: [§§15.2–15.4](../../../../substrate/docs/guidance/2026-10-17-handbook-b04-reconciliation.md#152-smallest-next-separately-authorizable-delta)
565: still prescribes F1-only extension in the isolated host (and §15.5 repeats it).
566: That older recommendation and external consultation advice do not override newer
567: human direction for this Collider amendment. The next-packet order is superseded
568: here only; Substrate was not edited and cross-repository agreement is not claimed.
569: Its real-owner requirements and proof ceilings remain valid. Obtain separately
570: scoped alignment before treating the external next-order text as synchronized.
571: 
572: ### Milestone exit and documentation endpoint
573: 
574: Future milestone exit requires the complete Workspace journey, every retained F1
575: assertion, honest dialect/routing tests, exact identity/effect-count evidence and
576: applicable regressions, followed by independent review. Record each remaining
577: cumulative P01–P22/proof-spec gap as open unless fresh applicable assertions and
578: executions close it. Approval/queue synthetic obligations survive real-feature
579: deferral. A bounded Workspace success is not full Gate B, B04, Gate C, native Tauri,
580: coherent V1, Gate D or frontend adoption; none of those is a new prerequisite to
581: learning from this milestone. Separately authorized owner work may proceed under
582: its own authority without changing that ceiling.
583: 
584: This task's endpoint is only independently reviewed documentation in these two
585: existing files, with task delta against starting dirty bytes and combined HEAD
586: diff preserved separately. Documentation whitespace, links/anchors, table shape,
587: F1 sub-obligation retention, source correspondence and scope checks apply. No
588: runtime tests, install, preflight or sweep are asserted by this amendment; later
589: publication retains the repository's required gates. Independent review and owner
590: acceptance are separate from authorship and checks.


## Collider/docs/agent-native/controller-correction/controller-correction-decision.md:1–65
SHA256 full file: 2fcb292b3c0e24e5c2605f8bcbfd83915103909bc74289c39defb884b630f965

1: # Packet A correction decision
2: 
3: ## Current resumed disposition
4: 
5: **CONTROLLER_CORRECTION_VERIFIED — Packet A fixture/client scope only.**
6: 
7: The user's cancellation clarification resolves the historical stop below. On the
8: existing `spike/agentkit-controller-correction`, based exactly on
9: `de2882ee120b0e89d3d3b25b53b18ffea287144e`, the maintained exact-version AgentKit
10: 0.2.3 patch corrects false running and false terminal cancellation in the actual
11: client. Toolkit remains 0.20.3 unchanged. Required ancestry is verified; no new
12: branch, base rewrite or historical proof rewrite.
13: 
14: The patch remains five emitted runtime/type files plus their five removed maps.
15: Validator changes are only for approved run statuses, not a new error-code enum.
16: Local cancelRequest accepted/refused/failed, generation guards, observation and
17: terminal races follow the user's clarified contract. There is no new refusal
18: stream event, durable owner state, owner facade or parallel controller.
19: 
20: ## Evidence and independent review
21: 
22: - [Implementation and preserved stop history](implementation-record.md)
23: - [Regression results and exclusions](regression-results.md)
24: - [Exact patch provenance and staged clean-install receipt](patch-provenance.json)
25: 
26: Original 45 characterization tests pass unchanged; two shared corrected-invariant
27: controls fail against unpatched bytes for the expected assertions and pass against
28: the patched candidate. Corrected suite 80/80; upstream original full profile 14
29: checks (approval/queue off); strict types; 14 browser tests; Vite/baseline and clean
30: frozen installation; check/loc/test-all all pass. Accepted-first local streams
31: remain intentionally outside upstream's first-run.started conformance assumption.
32: 
33: Independent built-in reviewers, both `gpt-5.6-terra` / `high`:
34: `/root/resumed_contract_review` confirmed contract closure before patching;
35: `/root/patch_review` reviewed exact patch, tests, mechanics and revised race/identity
36: handling, CLEAN with no unresolved P1/P2. Its original P2 normalized-message-loss
37: finding was fixed and independently rechecked. Final evidence review: **CLEAN, no
38: P1/P2**. The reviewer independently checked all 45 staged paths, 40 clean-export
39: input hashes, command receipts/fail-closed controls and evidence separation. No
40: wording correction was required. No owner/native certification is implied.
41: 
42: ## Preservation, limits and next authorization
43: 
44: All changes are new files under the isolated correction proof and correction
45: evidence directories. Root production dependencies, original Gate B, six plans,
46: canonical 00/01/02 and prior result documents remain byte-identical. Initial branch
47: refs and unrelated ZIP are preserved. Substrate/UAA were not accessed or modified
48: by the task; no refreshed owner HEAD/status claim. Nothing is merged, pushed,
49: published, opened as a PR or adopted. C-OQ-027 remains Open.
50: 
51: Still **unproven/not authorized**: real owner/provider integration, native
52: Tauri/WebView, distribution/notice clearance and production adoption. Reported
53: SLSA/source correspondence is not signature validation or source-build equivalence.
54: No Packet B or Gate C work was performed. The maintained patch needs future
55: security/update review and a revalidated exact-version replacement/removal decision.
56: 
57: The exact next separate authorization to request is **Packet B focused Substrate
58: contract design for the committed B01–B05 prerequisites only**. Do not implement
59: that owner boundary or enter Gate C without later authorization. This result does
60: not close those owner prerequisites.
61: 
62: The correction is eligible for one bounded local commit after final evidence
63: review, formatting/integrity checks and normal hooks. The final commit hash is
64: reported externally, never embedded in its own contributing evidence files.
65: 


## Collider/docs/ai-elements-inventory.md:1–65
SHA256 full file: cb0e89191d73019e66733d836c5371787037167e7815de85b8ba234e67aecaa9

1: # AI Elements Inventory
2: 
3: Local reference for every component in the [ai-elements registry](https://elements.ai-sdk.dev), tracked against
4: our Stage-2 loop progress. Refresh this file whenever a loop lands or the registry churns; do not treat
5: memory notes or old snapshots as canonical.
6: 
7: - **Registry total:** 48 components
8: - **Fresh as of:** 2026-08-19
9: - **Progress:** 32/48 done via Stage-2 (all 32 loops committed — coding-agent cluster complete)
10: - **Source of truth:** <https://elements.ai-sdk.dev/api/registry/registry.json>
11: - **Per-component source:** `https://elements.ai-sdk.dev/api/registry/<name>.json`
12: - **Refresh:** fetch the two URLs above, diff against the alphabetical index at the bottom, update statuses,
13:   update the header dates and totals
14: 
15: ## Legend
16: 
17: | Marker    | Meaning                                                                    |
18: | --------- | -------------------------------------------------------------------------- |
19: | ✅ done   | Stage-2 loop complete — code + stories + governance + Figma seed committed |
20: | 🧩 folded | Present in the repo via another loop (no dedicated loop needed)            |
21: | 🎯 queue  | Planned for the next Stage-2 loops, ordered by intended sequence           |
22: | 🌉 bridge | Stage-3 assembly primitive, not a Stage-2 loop target                      |
23: | ⏸️ parked | Out of the current transcript + coding-agent scope                         |
24: 
25: ## ✅ Done (32) — Tier A + Tier B + composer + coding-agent (13/13)
26: 
27: Ordered by loop number.
28: 
29: | #   | Component             | Cluster      | Commit  | Notes                                                                              |
30: | --- | --------------------- | ------------ | ------- | ---------------------------------------------------------------------------------- |
31: | 1   | message               | Tier A       | 1d6445b | First component; established the loop shape                                        |
32: | 2   | reasoning             | Tier A       | 17cf091 | Introduced shimmer                                                                 |
33: | 3   | code-block            | Tier A       | a395f0c | shiki-backed; split into 4 files                                                   |
34: | 4   | tool                  | Tier A       | 3759598 | Interactive tier baseline                                                          |
35: | 5   | sources               | Tier A       | 744ca96 | First primitive tier                                                               |
36: | 6   | task                  | Tier A       | f118a9b | asChild-button reconcile                                                           |
37: | 7   | chain-of-thought      | Tier A       | 39cd15d | Compound flow steps                                                                |
38: | 8   | suggestion            | Tier A       | 80379f9 | Chip strip                                                                         |
39: | 9   | inline-citation       | Tier A       | 2b4c5b4 | Hover-card + carousel deps                                                         |
40: | 10  | prompt-input          | Tier B       | b138a6d | 1464→8 files; 6 new ui prims                                                       |
41: | 11  | context               | Tier B       | ff3a259 | tokenlens + progress                                                               |
42: | 12  | snippet               | Tier B       | 1729d41 | Monospace copy pill                                                                |
43: | 13  | image                 | Tier B       | 059a6ab | 17 LOC wrapper                                                                     |
44: | 14  | open-in-chat          | Tier B       | 5310d51 | Dropdown with 6 providers                                                          |
45: | 15  | artifact              | Tier B       | 3583c43 | Bordered file card                                                                 |
46: | 16  | web-preview           | Tier B       | 65437e9 | LOC-split; Figma rebuilt to v3 (d384303)                                           |
47: | 17  | confirmation          | Tier B       | b5e16bc | New alert ui prim                                                                  |
48: | 18  | plan                  | Tier B       | fd21718 | New card ui prim; closed Tier B                                                    |
49: | 19  | attachments           | Composer     | 0f9e385 | 3 variants (grid/inline/list); LOC-split into 2 files                              |
50: | 20  | agent                 | Coding-agent | a6f9012 | New accordion ui prim; 7 memo'd exports; composes CodeBlock                        |
51: | 21  | queue                 | Coding-agent | 8afecba | 15 exports, LOC-split (queue + queue-item); zero new deps                          |
52: | 22  | checkpoint            | Coding-agent | 713f580 | Primitive tier, 62 LOC, 3 exports; zero new deps                                   |
53: | 23  | package-info          | Coding-agent | b5348ea | 9 exports LOC-split; 5 change-type variants; zero new deps                         |
54: | 24  | environment-variables | Coding-agent | d673bc1 | Visibility toggle, masked values, copy affordance; new switch prim                 |
55: | 25  | test-results          | Coding-agent | 890da56 | Collapsible suite results, progress, status and error states; no new deps          |
56: | 26  | file-tree             | Coding-agent | 77f560b | Controlled/uncontrolled hierarchy, selection, keyboard and workflow proof          |
57: | 27  | schema-display        | Coding-agent | 486f813 | HTTP endpoint card, 5 method colors, 3-file split (471→162/90/133 LOC)             |
58: | 28  | commit                | Coding-agent | fa86aa2 | Git commit card w/ file diff; new avatar prim; asChild-div a11y fix                |
59: | 29  | stack-trace           | Coding-agent | 1af9d7f | Parses JS stack traces; internal-frames muted; 3-file split (464→151/156/107)      |
60: | 30  | jsx-preview           | Coding-agent | 717cdcf | react-jsx-parser wrapper; derived-state refactor for stricter react-hooks lint     |
61: | 31  | terminal              | Coding-agent | 82df12f | ansi-to-react ANSI output card; 2-file split (273→116/171); streaming caret        |
62: | 32  | sandbox               | Coding-agent | 9f0bdb8 | Collapsible+Tabs code/output card; new shadcn tabs prim; composes Tool + CodeBlock |
63: 
64: ## 🎯 Queue (0) — Stage-2 complete
65: 


## Collider/src/components/upstream-sources.md:1–48
SHA256 full file: 2972f758e1a06b4a3308558986e01a9f663ea48c2b4e5883980e7f27ac1106e5

1: # Collider's upstream evidence and owned components
2: 
3: Collider owns its copied components and local splits. Captured upstream sources are
4: comparison/curation inputs, not instructions to overwrite those components.
5: 
6: - `upstream-registries.json` explicitly selects the registry endpoints, items and
7:   license documents. Version labels describe an observation, not verified npm versions.
8: - `upstream-evidence.json` retains exact payload, index and license bytes plus hashes.
9:   Its SHA-256 pin is in the root `ds-skills.project.json`. Do not format this file.
10: - `upstream-ownership.json` maps actual copied files and the 30 explicit local splits
11:   to source items. `migration-target` still means a comparison target: the primitive
12:   sources are not claimed to have completed the newer-style migration. `current-source`
13:   identifies the acquisition lineage, not continuous upstream ownership or parity.
14: - `upstream-policy.json` is Collider's chosen invariants/deviations/API obligations.
15:   The installed product evaluates those rules and the configured source import/slot
16:   contract. Application typechecks and browser tests remain separate obligations.
17: 
18: All 23 primitive source items and 33 AI Elements source items are captured. Their
19: 56 payload digests match the former baseline; the richer snapshot now preserves the
20: bytes themselves, full file paths, dependency declarations, licenses and observed
21: index membership. This is not a three-component demonstration or a new adoption.
22: 
23: ## Explicit refresh
24: 
25: 1. Review changes to selections, endpoints and any owned-file provenance.
26: 2. `pnpm baseline:upstream` explicitly captures a candidate under
27:    `.codex-artifacts/upstream/candidate.json`. It does not accept or install it.
28: 3. `pnpm baseline:upstream:diff` compares that candidate against accepted evidence
29:    without another network request. Review the sources, metadata, licenses and diff.
30: 4. After approval, copy exact candidate bytes to `upstream-evidence.json` and update
31:    its SHA-256 in `ds-skills.project.json` together. Run
32:    `pnpm baseline:upstream:check`, the source checks and applicable app tests.
33: 
34: `pnpm baseline:upstream:check` is offline and required by `just check` and CI. It
35: verifies the accepted pin and current selection, **not current remote freshness**.
36: Capture failure or missing index membership does not establish upstream deletion.
37: No source scripts, dependency installers, implicit registry traversal or application
38: rewrites occur. `--dry-run` is not supported: capture already writes only a candidate;
39: use the separate diff command to inspect it.
40: 
41: `pnpm validate:consumer-contract --json` reports actual selected imports and literal
42: slot ownership. `just check-contract` is now enforced in `just check`. The Select
43: trigger slot has been repaired, with dark/light browser tests proving the ButtonGroup
44: trailing corners; no other Select migration or unrelated component redesign is implied.
45: These checks establish neither recipe eligibility nor Figma publication/readiness.
46: 
47: Product command semantics, supported source syntax and acquisition limits belong to
48: ds-skills. This document records only Collider's inputs and invocation policy.


## Installed proof package agentkit 0.2.3: exported entrypoints only, not selected/qualified UI
.
./protocol
./http
./conformance
./react
./react/root
./react/chat
./react/components
./react/context
./react/headless
./react/streaming-text
./react/styles.css


## Installed proof package toolkit 0.20.3: exported entrypoints only, not selected/qualified UI
.
./app-shell
./app-shell/*
./agentkit
./collab-ui
./collab-ui/*
./dashboard
./dashboard/*
./data-grid
./data-grid/*
./chat-history
./chat-history/*
./canvas-annotations
./canvas-annotations/*
./canvas-interactions
./canvas-interactions/*
./clipboard
./composer
./composer/*
./conformance
./conformance/*
./context-ui
./design-tweaks
./design-system
./design-system/*
./editor
./editor/*
./hooks
./hooks/*
./marketing
./marketing/*
./onboarding
./onboarding/*
./provider
./sharing
./sharing/*
./ui
./ui/*
./utils
./streaming-text-smoothing
./markdown-block-split
./agent-native.eject.json
./migration-manifest.json
./styles.css
./chat-history.css
./editor.css


## AgentKit public reference React declarations (candidate surfaces, not all exercised)
import { type PromptComposerProps } from "@agent-native/toolkit/agentkit";
import { Component, type ErrorInfo, type ReactNode } from "react";
import { type AgentActivity, type AgentApprovalRequest, type AgentConnectionRequest, type AgentInteraction, type AgentMessage, type AgentMessagePart, type AgentObjectReference, type AgentParticipant, type AgentTask, type AgentWidget, type RunId } from "../protocol/index.js";
import { type AgentConnectionErrorRenderProps, type AgentKitRenderProps, type AgentRunFailureRenderProps } from "./context.js";
export interface AgentKitErrorBoundaryProps {
    children: ReactNode;
    resetKey?: string | number;
    fallback: ReactNode | ((error: Error) => ReactNode);
    onError?: (error: Error, info: ErrorInfo) => void;
}
interface AgentKitErrorBoundaryState {
    error?: Error;
    resetKey?: string | number;
}
/** Isolates custom renderers so one extension cannot take down the chat. */
export declare class AgentKitErrorBoundary extends Component<AgentKitErrorBoundaryProps, AgentKitErrorBoundaryState> {
    state: AgentKitErrorBoundaryState;
    static getDerivedStateFromError(error: Error): Partial<AgentKitErrorBoundaryState>;
    static getDerivedStateFromProps(props: AgentKitErrorBoundaryProps, state: AgentKitErrorBoundaryState): Partial<AgentKitErrorBoundaryState> | null;
    componentDidCatch(error: Error, info: ErrorInfo): void;
    render(): ReactNode;
}
/** Rejects executable and embedded-data URLs before they reach a default anchor. */
export declare function safeAgentHref(href?: string): string | undefined;
/** Keeps participant avatars on image-capable, non-executable URL schemes. */
export declare function safeAgentImageSrc(src?: string): string | undefined;
export declare function AgentObjectReferenceView({ value: object, }: AgentKitRenderProps<AgentObjectReference>): import("react").JSX.Element;
export declare function AgentParticipantView({ value: agent, }: AgentKitRenderProps<AgentParticipant>): import("react").JSX.Element;
export declare function AgentInteractionItem({ value: interaction, threadId, }: AgentKitRenderProps<AgentInteraction>): import("react").JSX.Element;
export declare function AgentActivityItem({ value: activity, threadId, }: AgentKitRenderProps<AgentActivity>): import("react").JSX.Element;
interface AgentEventRange {
    runId?: RunId;
    afterSequence?: number;
    throughSequence?: number;
}
export declare function formatAgentKitDuration(ms: number, units?: {
    hour?: string;
    minute?: string;
    second?: string;
}): string;
export declare function AgentActivityGroup({ runId, afterSequence, throughSequence, excludeAgentActivities, }: {
    runId?: RunId;
    afterSequence?: number;
    throughSequence?: number;
    excludeAgentActivities?: boolean;
}): import("react").JSX.Element | null;
export declare function AgentCollaborationFeed({ runId, afterSequence, throughSequence, }: AgentEventRange): import("react").JSX.Element | null;
export declare function AgentTaskItem({ value: task, threadId, }: AgentKitRenderProps<AgentTask>): import("react").JSX.Element;
export declare function AgentTaskGroup({ runId, afterSequence, throughSequence, }: AgentEventRange): import("react").JSX.Element | null;
export declare function AgentApprovalPrompt({ request, runId, }: {
    request: AgentApprovalRequest;
    runId: string;
}): import("react").JSX.Element;
export declare function AgentConnectionRequestCard({ request, runId, }: {
    request: AgentConnectionRequest;
    runId: string;
}): import("react").JSX.Element;
export declare function AgentWidgetView({ value: widget, threadId, }: AgentKitRenderProps<AgentWidget>): import("react").JSX.Element | null;
export interface AgentMessagePartViewProps extends AgentKitRenderProps<AgentMessagePart> {
    active?: boolean;
    resetKey?: string;
}
export declare function AgentMessagePartView({ value: part, threadId, active, resetKey, }: AgentMessagePartViewProps): import("react").JSX.Element | null;
export declare function AgentMessageActions({ value: message, threadId, }: AgentKitRenderProps<AgentMessage>): import("react").JSX.Element;
export declare function AgentMessageView({ value: message, threadId, }: AgentKitRenderProps<AgentMessage>): import("react").JSX.Element;
export declare function AgentRunFailure({ error, runId, threadId, }: AgentRunFailureRenderProps): import("react").JSX.Element;
export declare function AgentConnectionErrorView({ error, threadId, recover, recovering, recoveryError, }: AgentConnectionErrorRenderProps): import("react").JSX.Element;
export interface AgentKitComposerProps extends Pick<PromptComposerProps, "slashCommands" | "slashSkills" | "includeDefaultSlashCommands" | "includeDefaultSlashSkills" | "onSlashCommand" | "plusMenuMode" | "voiceEnabled" | "autoFocus"> {
    threadId?: string;
    className?: string;
    queueWhileRunning?: boolean;
    showModelSelector?: boolean;
    /** Controlled execution mode for agent-native act/plan workflows. */
    mode?: "act" | "plan";
    /** Initial execution mode when the composer is uncontrolled. */
    defaultMode?: "act" | "plan";
    /** Called when the execution mode changes. */
    onModeChange?: (mode: "act" | "plan") => void;
}
export declare function AgentKitComposer({ threadId: requestedThreadId, className, queueWhileRunning, showModelSelector, slashCommands, slashSkills, includeDefaultSlashCommands, includeDefaultSlashSkills, onSlashCommand, plusMenuMode, voiceEnabled, autoFocus, mode, defaultMode, onModeChange, }: AgentKitComposerProps): import("react").JSX.Element;
export interface AgentKitChatProps {
    title?: ReactNode;
    toolbar?: ReactNode;
    composer?: boolean;
    /** Configures the reference composer without replacing its slot. */
    composerProps?: Omit<AgentKitComposerProps, "threadId">;
    /** New conversations center the composer; embedded panels can anchor it. */
    emptyComposerPlacement?: "center" | "bottom";
    /** Follow new output until the user deliberately scrolls away. */
    autoScroll?: boolean;
    className?: string;
}
export declare function isAgentKitTranscriptNearBottom(element: Pick<HTMLElement, "clientHeight" | "scrollHeight" | "scrollTop">, threshold?: number): boolean;
export declare function AgentKitChat({ title, toolbar, composer, composerProps, emptyComposerPlacement, autoScroll, className, }: AgentKitChatProps): import("react").JSX.Element;
export {};
//# sourceMappingURL=components.d.ts.map

## Local component filenames (presence only, not readiness/adoption)

src/components/ai-elements
src/components/ai-elements/agent.stories.tsx
src/components/ai-elements/agent.tsx
src/components/ai-elements/artifact.stories.tsx
src/components/ai-elements/artifact.tsx
src/components/ai-elements/attachment-parts.tsx
src/components/ai-elements/attachments.stories.tsx
src/components/ai-elements/attachments.tsx
src/components/ai-elements/chain-of-thought.stories.tsx
src/components/ai-elements/chain-of-thought.tsx
src/components/ai-elements/checkpoint.stories.tsx
src/components/ai-elements/checkpoint.tsx
src/components/ai-elements/code-block-body.tsx
src/components/ai-elements/code-block-highlight.ts
src/components/ai-elements/code-block-parts.tsx
src/components/ai-elements/code-block-theme.test.ts
src/components/ai-elements/code-block-theme.ts
src/components/ai-elements/code-block.stories.tsx
src/components/ai-elements/code-block.tsx
src/components/ai-elements/commit-files.tsx
src/components/ai-elements/commit.stories.tsx
src/components/ai-elements/commit.tsx
src/components/ai-elements/confirmation.stories.tsx
src/components/ai-elements/confirmation.tsx
src/components/ai-elements/context-usage.tsx
src/components/ai-elements/context.stories.tsx
src/components/ai-elements/context.tsx
src/components/ai-elements/environment-variables-parts.tsx
src/components/ai-elements/environment-variables.stories.tsx
src/components/ai-elements/environment-variables.tsx
src/components/ai-elements/file-tree-items.tsx
src/components/ai-elements/file-tree.stories.tsx
src/components/ai-elements/file-tree.tsx
src/components/ai-elements/image.stories.tsx
src/components/ai-elements/image.tsx
src/components/ai-elements/inline-citation-carousel.tsx
src/components/ai-elements/inline-citation.stories.tsx
src/components/ai-elements/inline-citation.tsx
src/components/ai-elements/jsx-preview-parts.tsx
src/components/ai-elements/jsx-preview.stories.tsx
src/components/ai-elements/jsx-preview.tsx
src/components/ai-elements/message-branch.tsx
src/components/ai-elements/message.stories.tsx
src/components/ai-elements/message.tsx
src/components/ai-elements/open-in-chat-providers.tsx
src/components/ai-elements/open-in-chat.stories.tsx
src/components/ai-elements/open-in-chat.tsx
src/components/ai-elements/package-info-parts.tsx
src/components/ai-elements/package-info.stories.tsx
src/components/ai-elements/package-info.tsx
src/components/ai-elements/plan.stories.tsx
src/components/ai-elements/plan.tsx
src/components/ai-elements/prompt-input-actions.tsx
src/components/ai-elements/prompt-input-context.tsx
src/components/ai-elements/prompt-input-controls.tsx
src/components/ai-elements/prompt-input-helpers.ts
src/components/ai-elements/prompt-input-hooks.ts
src/components/ai-elements/prompt-input-parts.tsx
src/components/ai-elements/prompt-input-select.tsx
src/components/ai-elements/prompt-input.stories.tsx
src/components/ai-elements/prompt-input.tsx
src/components/ai-elements/queue-item.tsx
src/components/ai-elements/queue.stories.tsx
src/components/ai-elements/queue.tsx
src/components/ai-elements/reasoning.stories.tsx
src/components/ai-elements/reasoning.tsx
src/components/ai-elements/sandbox.stories.tsx
src/components/ai-elements/sandbox.tsx
src/components/ai-elements/schema-display-parts.tsx
src/components/ai-elements/schema-display-property.tsx
src/components/ai-elements/schema-display.stories.tsx
src/components/ai-elements/schema-display.tsx
src/components/ai-elements/shimmer.tsx
src/components/ai-elements/snippet.stories.tsx
src/components/ai-elements/snippet.tsx
src/components/ai-elements/sources.stories.tsx
src/components/ai-elements/sources.tsx
src/components/ai-elements/stack-trace-frames.tsx
src/components/ai-elements/stack-trace-header.tsx
src/components/ai-elements/stack-trace.stories.tsx
src/components/ai-elements/stack-trace.tsx
src/components/ai-elements/suggestion.stories.tsx
src/components/ai-elements/suggestion.tsx
src/components/ai-elements/task.stories.tsx
src/components/ai-elements/task.tsx
src/components/ai-elements/terminal-parts.tsx
src/components/ai-elements/terminal.stories.tsx
src/components/ai-elements/terminal.tsx
src/components/ai-elements/test-results-suite.tsx
src/components/ai-elements/test-results-test.tsx
src/components/ai-elements/test-results.stories.tsx
src/components/ai-elements/test-results.tsx
src/components/ai-elements/tool.stories.tsx
src/components/ai-elements/tool.tsx
src/components/ai-elements/web-preview-console.tsx
src/components/ai-elements/web-preview.stories.tsx
src/components/ai-elements/web-preview.tsx

src/components/ui
src/components/ui/accordion-governance.stories.tsx
src/components/ui/accordion.stories.tsx
src/components/ui/accordion.tsx
src/components/ui/alert.stories.tsx
src/components/ui/alert.tsx
src/components/ui/avatar.stories.tsx
src/components/ui/avatar.tsx
src/components/ui/badge.stories.tsx
src/components/ui/badge.tsx
src/components/ui/button-governance.stories.tsx
src/components/ui/button-group-governance.stories.tsx
src/components/ui/button-group.stories.tsx
src/components/ui/button-group.tsx
src/components/ui/button.stories.tsx
src/components/ui/button.tsx
src/components/ui/card.stories.tsx
src/components/ui/card.tsx
src/components/ui/carousel-context.ts
src/components/ui/carousel-governance.stories.tsx
src/components/ui/carousel.stories.tsx
src/components/ui/carousel.tsx
src/components/ui/collapsible-governance.stories.tsx
src/components/ui/collapsible.stories.tsx
src/components/ui/collapsible.tsx
src/components/ui/command-governance.stories.tsx
src/components/ui/command.stories.tsx
src/components/ui/command.tsx
src/components/ui/dropdown-menu-governance.stories.tsx
src/components/ui/dropdown-menu-parts.tsx
src/components/ui/dropdown-menu.stories.tsx
src/components/ui/dropdown-menu.tsx
src/components/ui/hover-card-governance.stories.tsx
src/components/ui/hover-card.stories.tsx
src/components/ui/hover-card.tsx
src/components/ui/input-governance.stories.tsx
src/components/ui/input-group-governance.stories.tsx
src/components/ui/input-group.stories.tsx
src/components/ui/input-group.tsx
src/components/ui/input.stories.tsx
src/components/ui/input.tsx
src/components/ui/progress.stories.tsx
src/components/ui/progress.tsx
src/components/ui/scroll-area-governance.stories.tsx
src/components/ui/scroll-area.stories.tsx
src/components/ui/scroll-area.tsx
src/components/ui/select-governance.stories.tsx
src/components/ui/select.stories.tsx
src/components/ui/select.tsx
src/components/ui/separator.stories.tsx
src/components/ui/separator.tsx
src/components/ui/spinner.stories.tsx
src/components/ui/spinner.tsx
src/components/ui/switch-governance.stories.tsx
src/components/ui/switch.stories.tsx
src/components/ui/switch.tsx
src/components/ui/tabs-governance.stories.tsx
src/components/ui/tabs.stories.tsx
src/components/ui/tabs.tsx
src/components/ui/textarea-governance.stories.tsx
src/components/ui/textarea.stories.tsx
src/components/ui/textarea.tsx
src/components/ui/tooltip-governance.stories.tsx
src/components/ui/tooltip.stories.tsx
src/components/ui/tooltip.tsx


## Discovery limits
Targeted case-insensitive BlockNote search under Collider docs/ and storybook/ Markdown returned no hits. That is not proof no historical decision exists; ask history and broader discovery, do not invent adoption. Full design-system configuration, agent rules and enforcement not supplied. Do not prescribe disabling gates without local policy reconciliation.
