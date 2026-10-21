# Bounded approach-review evidence — 2026-10-18

Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

Collider branch spike/agentkit-controller-correction at a491f98724e539a3d7a23aa0661b6b99af79c657. Substrate docs/substrate-collider-contracts at aa5ef73ae087aa82d1f952ba6dd5c92361b8e24f. Handbook reviewed source at 379af4eda9b95ee82be9d3afcfdcdca2d6d132f9. No new runtime tests were performed. These are bounded local source copies, not evidence you have live GitHub access.

Read canonical C-OQ-027 and the proof specification first, then Packet A evidence, then the reconciliation proposal. Full glossary/foundations are supplied to preserve product intent, not to make full V1 a prerequisite.

## Included Collider sources
- docs/agent-native/canonical/00-collider-v1-glossary.md
- docs/agent-native/canonical/01-collider-v1-foundations.md
- docs/agent-native/canonical/02-collider-v1-open-architecture-questions.md
- docs/agent-native/proof/agentkit-owner-boundary-proof-spec.md
- docs/agent-native/controller-correction/implementation-record.md
- docs/agent-native/controller-ownership-plan/substrate-accepted-work-boundary-plan.md
- proofs/agentkit-controller-correction/README.md
- src/app/page.tsx


# SOURCE: Collider/docs/agent-native/canonical/00-collider-v1-glossary.md

Source SHA-256: 6ea4d8651176f901bb6a77e3f964e1125bbd3681ca7bf8282908be4e59e42852

# Collider V1 Glossary and Concept Model

**Status:** Proposed canonical terminology for Collider V1\
**Document role:** Terminology and relationship authority\
**Applies to:** Collider V1 product, UX, architecture, protocol, and implementation documents\
**Implementation authorization:** None; this document defines vocabulary and conceptual boundaries only\
**Provisional sections:** 6.7–6.14 — execution-instance and agent-topology vocabulary\
**Open architecture register:** [`02-collider-v1-open-architecture-questions.md`](./02-collider-v1-open-architecture-questions.md)

## 1. Purpose and Authority

This document defines the canonical terms used by the Collider V1 program. Its purpose is to prevent the product, UI, Substrate runtime, and Handbook integration from assigning different meanings to the same words.

All later Collider V1 documents SHOULD reference this glossary rather than redefine its terms. When a later document requires a narrower rule, it MAY refine a term for that bounded context, but it MUST NOT contradict this document without explicitly updating or superseding the affected definition here.

External research, reference architectures, and implementation studies are non-normative inputs unless an authorized Collider document explicitly adopts a result. Reusing an external architectural pattern does not automatically import the source project's terminology, schemas, APIs, lifecycle model, package boundaries, or authority assumptions. A term originating outside Collider becomes canonical only through an explicit change to this glossary that defines its Collider meaning and reconciles it with existing terms and owner-system boundaries.

Capitalized terms such as **Workspace**, **Lane**, **Workstream**, **Invocation**, **Run**, **Surface**, and **Tab** refer to the definitions in this document.

Any section or term explicitly marked **Provisional Draft** is excluded from this document's canonical authority. It records current design thinking only. Normative keywords inside a provisional section express candidate constraints rather than settled protocol, schema, persistence, compatibility, or implementation requirements. References elsewhere in this document to a provisional term inherit that provisional status unless the surrounding text explicitly identifies a terminology-independent invariant.

The key words **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative:

- **MUST / MUST NOT** identify required invariants.
- **SHOULD / SHOULD NOT** identify strong defaults that require an explicit reason to violate.
- **MAY** identifies permitted but optional behavior.

Where this glossary summarizes a term whose canonical semantic authority belongs to Handbook, Handbook's own definition remains authoritative. Collider MUST consume and present that meaning; it MUST NOT create a competing definition.

## 2. Compact Concept Model

The shortest useful distinction among the primary terms is:

| Primitive | Question it answers | Primary authority |
|---|---|---|
| **Workspace** | Within what Substrate configuration and policy boundary are we operating? | Substrate |
| **Scope** | Semantically, what does this work concern? | Handbook |
| **Work Item** | What durable outcome or obligation is being pursued? | Handbook |
| **Lane** | Where and under what execution conditions does work run? | Substrate |
| **Workstream** | Through which interaction context and ordered activity projection is work directed, observed, and revisited? | Collider for projection and presentation; source systems retain record authority |
| **Qualified Session** | Which owner-specific provider or runtime continuity resource may be explicitly resumed? | The relevant provider or runtime |
| **Session History** | Which owner-scoped records support reconstruction, resumption, audit, or explanation of a Qualified Session? | The relevant provider or runtime |
| **Presentation Action** | What local Collider presentation-state change was requested? | Collider |
| **Invocation** | What owner-system action was accepted through Collider? | Substrate |
| **Run** *(provisional)* | What execution occurred? | Substrate |
| **Source Record** | What durable or durably addressable owner-system record supports reconstruction or explanation? | The producing source system |
| **Projection Definition** | By what host-recognized rules are eligible Source Records correlated and assembled for a target view? | Collider projection machinery; source systems retain record authority |
| **Activity Projection** | Which stable projected activity is assembled for a particular Workstream view? | Collider projection over owner-system records and references |
| **Activity Item** | What stable projected business object may be rendered in the Workstream? | Collider projection metadata; backing authority remains with source systems |
| **Block** | How is an Activity Item, response, or application currently rendered inline? | Collider |
| **Surface** | What typed application or view is being rendered? | Collider, backed by owner-system state |
| **Sticky Placement** | What Workstream-local placement remains visible while chronological activity scrolls? | Collider presentation state |
| **Tab** | Where is a Surface opened for focused center-workspace access? | Collider |

A simplified presentation model is:

```text
Workspace owner systems and providers
├── Handbook Artifacts, snapshots, and semantic records
├── Substrate Invocations, Receipts, and execution records
├── MCP App objects, revisions, or references
├── shell / PTY command and output records
└── Collider Presentation Actions and presentation references
                         │
                         │ host-recognized Projection Definitions
                         ▼
                 Activity Assembler
                  + Projection Index
                         │
                         ▼
                 Activity Projection
                         │
          Activity Items with stable Business Identity
                         │
                rendered as Blocks
                         │
                   may host Surfaces
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
       inline         sticky           Tab
      placement      placement       placement
```

The Control Plane and Lane views SHOULD be capable of using the same core Workstream, Block, Surface, and Composer components:

```text
Workspace-relevant Activity Projection
├── Control Plane Workstream Surface
│   └── no Lane filter
└── Lane Workstream Surface
    └── internal filter = selected Lane
```

`No Lane filter` means **lane-unfiltered**, not “show every raw byte, internal tool event, provider callback, or telemetry record.” Projection eligibility, progressive disclosure, and internal-detail policy still apply.

This glossary intentionally does **not** select a canonical Workspace event-store architecture. The visible Workstream may ultimately be backed by one Workspace history, federated owner-system histories plus an activity index, query-time federation, or a hybrid. That decision is tracked in the open architecture register.

The durable invariant is narrower:

> **A Block is presentation derived from one or more traceable Source Records, durable references, or reconstructable owner-system objects; the Block does not become their authority.**

## 3. System Authorities

### 3.1 Handbook

**Handbook** is the semantic-authority system integrated into the broader Substrate product stack.

Handbook owns the canonical meaning and lifecycle of structured semantic artifacts, including applicable schema and artifact-kind semantics, Scope and Resolution, Projection, contracts, admitted Evidence, verdicts, gates, and related semantic relationships.

Collider MAY render, inspect, and initiate owner-approved mutations of Handbook state. Substrate MAY resolve and consume Handbook semantics during execution. Neither system becomes a second Handbook authority.

### 3.2 Substrate

**Substrate** is the workspace, policy, runtime, execution, agent, tool, process, and orchestration engine beneath Collider.

Substrate owns Workspace identity, Lane and provisional Run execution state, tool and process authority, accepted Invocation records, Substrate execution records and Receipts, raw runtime telemetry, and the runtime assembly and delivery of context.

Substrate consumes Handbook Resolution and Projection results. It does not redefine their semantic meaning.

### 3.3 Collider

**Collider** is the human-facing application and interaction layer over Substrate and integrated Handbook capabilities.

Collider owns the presentation and interaction model, including Presentation Actions, the Workspace Navigator, Control Plane presentation, Workstream and Activity Projection behavior, Activity Items, Blocks, Surfaces, Surface Placements, Tabs, Sticky Placements, and optional Inspector behavior.

Collider MUST NOT own the authoritative lifecycle of a Lane, provisional Run, Handbook Artifact, Contract, Work Item, Source Record, provider object, or other owner-system state merely because it displays that state.

### 3.4 Authority

**Authority** means the system that determines the canonical meaning, accepted state transitions, and durable identity of an object.

A system that renders, caches, indexes, or requests a mutation of an object is not necessarily that object's authority.

### 3.5 Trusted Extension

A **Trusted Extension** is executable extension code explicitly authorized by the host's deployment and policy model to run within a designated trusted process or runtime.

`Trusted` is an execution-trust classification. It does not grant Handbook semantic authority, Substrate execution authority, unrestricted host access, or permission to bypass Capability Grants and mediation. Permission to contribute UI does not by itself grant filesystem, process, network, credential, clipboard, browser, host-object, or semantic-mutation capability.

### 3.6 Untrusted Extension

An **Untrusted Extension** is executable extension code that is not authorized to execute with the ambient authority of a trusted host process or runtime.

Untrusted Extensions MUST remain behind an Isolation Boundary before they execute or receive any mediated host capability. They MUST NOT receive trusted-process authority merely because their requested capabilities appear limited.

### 3.7 Isolation Boundary

An **Isolation Boundary** is an enforceable process, runtime, operating-system, or equivalent containment boundary intended to limit unauthorized access, fault propagation, and ambient capability use by executable code.

An Isolation Boundary is distinct from a context object, registry, API wrapper, or Capability Grant. Capability mediation may narrow what isolated code can request, but it does not itself establish isolation. This glossary does not select a specific isolation technology.

### 3.8 Capability Grant

A **Capability Grant** is an explicit, scoped authorization allowing an identified extension or provider to request or use a particular host capability under defined constraints.

A Capability Grant MAY be issued and enforced only by the authority governing the relevant capability and policy seam, or by an explicitly delegated authority. A consumer cannot self-authorize. A provider may issue or enforce a grant only when it is itself the governing authority or has been explicitly delegated that role. A Capability Grant MAY be attenuated, revoked, and audited according to owner policy. It does not create an Isolation Boundary and does not transfer owner-system authority. A grant to render or contribute UI does not imply a grant to execute host code or access unrelated host resources.

### 3.9 Mediated Effect

A **Mediated Effect** is an effect initiated through a host-controlled interface that allows the host to observe it, attribute it to an owner or lifetime, and control the portion of its lifecycle that remains within that interface's boundary.

Only Mediated Effects may support claims of automatic host tracking, cleanup, or reversion, and only to the extent actually controlled by the mediation boundary.

### 3.10 External Effect

An **External Effect** is an effect that crosses the host-controlled mediation boundary or becomes observably committed in another system, process, filesystem, network destination, credential store, or other external state.

An External Effect is not automatically reversible. Its contract may require transactionality, copy-on-write, idempotent cleanup, a Compensating Action, or explicit classification as non-revertible.

### 3.11 Compensating Action

A **Compensating Action** is a later action intended to offset or mitigate the consequences of an earlier effect when an exact inverse is unavailable or inappropriate.

Compensation is not exact rollback. It may fail, may create additional observable history, and may not restore the prior state precisely.

### 3.12 Extension

An **Extension** is a bounded unit of optional or dynamically reconfigurable functionality that contributes behavior or integration through host-recognized seams.

An Extension MAY be first-party or third-party. Its permitted execution placement depends on its trust classification and host policy: a Trusted Extension may be authorized to run in a trusted process, while an Untrusted Extension remains subject to Section 3.6 and MUST execute behind an Isolation Boundary. Being an Extension does not grant Handbook semantic authority, Substrate execution authority, ambient host access, a Capability Grant, or any particular trust classification. Its loading mechanism, implementation language, package structure, and isolation mechanism are separate concerns.

### 3.13 Extension Contribution

An **Extension Contribution** is an attributable capability, service, UI contribution, renderer, tool, command, configuration contribution, subscription, or other bounded host-visible behavior made available by an Extension.

An Extension Contribution remains subject to the authority and capability boundaries of the systems it interacts with. Permission to contribute UI does not imply permission to execute host code or use unrelated runtime capabilities.

### 3.14 Extension Lifecycle

An **Extension Lifecycle** is the host-observed progression through which an Extension becomes eligible, activates, remains active or inactive, deactivates, is replaced or updated, fails, and may recover.

The final lifecycle state machine is not defined by this glossary. A lifecycle claim is meaningful only to the extent that the host can observe and control the relevant Extension Contributions and effects. Graceful deactivation MUST NOT be assumed to run after every failure or crash.

### 3.15 Revertible Contribution

A **Revertible Contribution** is an Extension Contribution for which the host has a valid, attributable, host-tracked inverse or restoration path that restores the relevant observable host state under a declared recovery criterion, within a declared mediated boundary and under stated preconditions.

Best-effort cleanup, resource abandonment, retry, and a Compensating Action do not make a contribution revertible. A Revertible Contribution does not imply that related External Effects can be exactly reversed.

### 3.16 Temporal Composability

**Temporal Composability** is a design objective for dynamically reconfigurable Extensions: within the actual mediated and attributable system boundary, removing, deactivating, replacing, or recovering an Extension should remove or restore its attributable contributions without restarting unrelated host functionality, disturbing unrelated host state, or leaving attributable residual state behind.

Temporal Composability is not an automatic guarantee. It is bounded by the host's ability to track contributions, execute valid inverse or recovery behavior, contain failures, and distinguish External Effects or non-revertible behavior.

### 3.17 Spatial Composability

**Spatial Composability** is a design objective under which Extensions declare the capabilities or services they require and provide, and their Extension Lifecycles react to dependency availability and identity.

The objective favors activation only when declared requirements are satisfied; orderly inactivity or deactivation when requirements disappear or materially change; reactivation when requirements become satisfied again; and dependent teardown before provider withdrawal when cleanup requires the provider. This definition does not select a concrete dependency-resolution API or implementation.

### 3.18 Spatiotemporal Composability

**Spatiotemporal Composability** is the combined design objective of Temporal Composability and Spatial Composability for a bounded class of dynamically reconfigurable Extensions.

It MUST NOT be presented as a formal or system-wide guarantee without evidence for the specific extension class, mediated boundary, lifecycle behavior, dependency discipline, confinement, valid recovery or inversion behavior, and any required independence, commutativity, or ordering conditions. The current Collider V1 documents adopt the objective; they do not claim that it has been proved or fully implemented.

### 3.19 Capability

A **Capability** is an owner-mediated interface through which an identified consumer may request bounded functionality or service behavior from an identified provider.

A Capability exposes access without transferring the provider's authority or copying its canonical state into the consumer. The provider remains authoritative for the state, behavior, lifecycle, and constraints it exposes. A Capability may be local or remote, and its contract MUST represent synchronous, asynchronous, failure, cancellation, and availability semantics honestly where they apply. Capability access remains subject to trust classification, Capability Grants, policy, and any required Isolation Boundary.

### 3.20 Required Capability

A **Required Capability** is a declared dependency that an Extension instance needs in order to become or remain eligible for a particular Extension Lifecycle state.

A Required Capability describes the needed interface, identity, and compatibility constraints rather than granting ambient access or transferring authority. Its absence, withdrawal, or material provider change may affect activation, inactivity, deactivation, or reactivation under the Spatial Composability objective.

### 3.21 Provided Capability

A **Provided Capability** is a Capability made available by an identified provider within a declared lifecycle and compatibility boundary.

The provider remains authoritative for the underlying state and service. Provision does not copy that authority into an Extension Context or consumer. Provider identity, availability, replacement, and withdrawal may affect dependent Extension Lifecycles.

### 3.22 Dependency Declaration

A **Dependency Declaration** is an inspectable statement of the Required Capabilities and Provided Capabilities associated with an Extension or provider, together with any declared identity or compatibility constraints.

A Dependency Declaration supports eligibility, dependency-topology inspection, and reactive lifecycle behavior. It is not automatically a Capability Grant, Handbook Contract, Substrate policy decision, package format, or concrete capability-resolution protocol.

### 3.23 Extension Context

An **Extension Context** is a bounded extension-facing capability environment associated with one Extension instance or another explicitly scoped composition environment.

An Extension Context resolves access to owner-approved Provided Capabilities subject to Dependency Declarations, Capability Grants, policy, trust classification, and provider availability. It MAY be derived or specialized for a Workspace, Lane, Workstream, Qualified Session, policy, audit, instrumentation, or trust condition, but it does not copy provider state or become a new cross-system source of truth.

An Extension Context is not Handbook Scope or Resolution, a Handbook Projection, a Substrate Context Pack, Injected Context, an Activity Projection, a Workstream, a React context, or a Workspace or Lane execution environment. It is also not an Isolation Boundary.

### 3.24 Extension Context Mediation

**Extension Context Mediation** is the routing of Extension capability access through an Extension Context and owner-approved provider interfaces so that access can be selected, attributed, constrained, observed, revoked, or audited.

Extension Context Mediation preserves the authority of the provider and the owner systems involved. It does not create semantic authority, copy canonical state, or establish security isolation.

### 3.25 Capability Interception

**Capability Interception** is host- or owner-approved augmentation of capability access to apply policy, audit, telemetry, instrumentation, attenuation, or other declared constraints without changing which system remains authoritative for the capability's state and service.

Interception MUST preserve the capability contract and MUST NOT disguise a remote or failure-prone operation as an infallible synchronous local call. Capability Interception is not an Isolation Boundary.

### 3.26 Contribution Provider

A **Contribution Provider** is an identified Extension, Surface Provider, or host-owned module authorized to register one or more UI Contributions through declared Extension Points.

A Contribution Provider remains attributable for the contributions it registers. Authorization to contribute UI does not make it authoritative for the owner-system state displayed by that UI and does not grant unrelated runtime Capabilities.

### 3.27 Extension Point

An **Extension Point** is a named, scoped, host-declared UI composition location through which authorized Contribution Providers may register UI Contributions.

Collider retains authority over the surrounding layout, the accepted contribution contract, and the policy governing the point. An Extension Point does not replace a Surface and is not itself an inline, Sticky, Tab, overlay, or Inspector placement. An Extension Point MAY be nested within another host-recognized Surface, but its declaring owner and lifecycle MUST remain explicit.

### 3.28 Extension Point Declaration

An **Extension Point Declaration** is the host-owned contract that identifies an Extension Point and describes which UI Contributions it may accept.

A declaration MAY constrain accepted contribution kinds, UI Contribution Scope, narrow props or handles, supported presentation modes, multiplicity, ordering, fallback, election, replacement, and conflict behavior. The concrete declaration schema is not defined by this glossary.

### 3.29 UI Contribution

A **UI Contribution** is an attributable, lifecycle-bound registration of a Surface, control, renderer, action, or related UI behavior into an authorized Extension Point.

A UI Contribution is not the Extension Point itself. A contributed Surface retains its Contribution Provider identity and all owner-system bindings. Cleanup follows the applicable Extension or provider lifecycle within the mediated boundary, and UI contribution permission does not imply filesystem, process, network, credential, semantic, or other host execution authority.

### 3.30 UI Contribution Scope

A **UI Contribution Scope** is the bounded set of host locations or composition conditions in which a UI Contribution is eligible to appear.

It MAY constrain a contribution by Workspace, Lane, Workstream, Qualified Session, Surface, provider, policy, trust condition, or another explicitly declared UI-composition dimension. UI Contribution Scope is not Handbook Scope and does not change semantic authority, execution placement, or Capability Grants.

### 3.31 Extension Point Policy

An **Extension Point Policy** is the host-defined policy governing admission and composition of UI Contributions at an Extension Point.

It MAY permit zero, one, or several contributions and MAY define keyed identity, ordering, priority, fallback, election, replacement, or conflict behavior. The policy governs presentation composition only; it does not transfer owner-system state or runtime authority to the selected Contribution Provider.

## 4. Workspace and Structural Terms

### 4.1 Workspace

A **Workspace** is a directory or root governed by its own Substrate configuration and policy boundary.

**Authority:** Substrate.

A Workspace MAY contain or reference:

- one or more repositories;
- Handbook state;
- Work Items and Scopes;
- Lanes and worktrees;
- Workstream identities, projection definitions, or reconstructable interaction contexts;
- provisional Runs and agents;
- tools and execution environments;
- Substrate-owned environment dependency configuration and Agent Profiles;
- Agent Harness configuration and Workspace-enabled skills, tools, apps, Extensions, and connections through their respective owners;
- Collider presentation state associated with that Workspace, including Navigator Layout preferences.

A Workspace is the canonical top-level entity that Collider opens in V1.

A Workspace is not defined by an open window, tab, terminal process, chat transcript, or repository. Closing Collider does not destroy the Workspace.

### 4.2 Project

**Project** is a reserved term and MUST NOT be used as a Collider synonym for Workspace.

A future system MAY define Project as:

- a Handbook semantic entity;
- a product-level grouping spanning multiple Workspaces;
- another explicitly owned higher-order concept.

Until such a definition exists, Collider V1 documentation and UI SHOULD use **Workspace** for the Substrate configuration and policy boundary.

### 4.3 Repository

A **Repository** is a version-controlled resource contained in or referenced by a Workspace.

A Workspace MAY contain zero, one, or multiple Repositories. Repository and Workspace are therefore not interchangeable.

A Repository MAY expose branches, commits, diffs, status, and tracked resources. Execution against a Repository still occurs under the containing Workspace and, when applicable, a Lane.

### 4.4 Worktree

A **Worktree** is a concrete checkout or working-tree resource used by a Lane.

A Worktree is one possible part of a Lane's execution binding. A Lane MAY use another isolation mechanism, and the Lane remains the canonical Collider/Substrate execution concept.

A Worktree does not own Workstreams, Runs, or Tabs.

### 4.5 Agent Harness

An **Agent Harness** is an agent-execution backend or runtime with which Substrate integrates. The term identifies an integration role, not an Agent Profile, model provider, Lane, Qualified Session, or the provisional Agent execution identity.

Native harness behavior and configuration remain governed by that harness. Substrate owns its adaptation, launch, execution-policy, and integration settings; Collider owns its presentation settings. One configuration Surface may expose these settings together without merging their authorities. **Harnesses** is a possible navigator label, not a new execution owner.

The exact configuration schemas, supported controls, compatibility model, and relationships among harnesses, profiles, models, Lanes, and Qualified Sessions require Substrate-grounded design under C-OQ-024. This definition does not establish a new harness protocol or settle the provisional execution topology.

### 4.6 Agent Profile

An **Agent Profile** is a reusable Substrate-owned configuration for an agent role or purpose. It may select or reference harness, model, instruction, skill, tool, environment, and execution defaults where the owner model supports them.

An Agent Profile is not an Agent Harness or a Qualified Session. It does not grant capabilities, override Substrate policy, or define Handbook Scope or Resolution merely by declaring defaults. Handbook profiles and native harness configuration retain their own meanings and owners.

The profile schema, inheritance, configuration precedence, compatibility, and application to active work remain owner-defined or open under C-OQ-024; this term does not canonize the provisional Agent model.

### 4.7 Environment Dependencies

**Environment Dependencies** are the execution-environment requirements consumed by Substrate's World dependency tooling, such as required toolchains, binaries, libraries, or services where that tooling supports them.

Collider may expose configuration and status for these requirements, but preparation, validation, and realization remain Substrate-owned. A declaration is not proof that a dependency has been realized or is ready. The existing World tooling's terminology and contracts control; Collider MUST NOT introduce a competing environment dependency engine.

Environment Dependencies are distinct from Repository/package dependencies and Extension Required Capabilities or Dependency Declarations. Relationships may be surfaced without merging their authority. Exact configuration sources, effective requirements, Lane relationships, realization, and readiness behavior remain open under C-OQ-023.

## 5. Handbook Semantic Terms

### 5.1 Handbook Artifact

A **Handbook Artifact** is a canonical structured semantic object governed by Handbook.

A Handbook Artifact MAY have one or more backing files or serialized representations, but it is not semantically equivalent to any one physical file.

Collider MUST interact with a Handbook Artifact through Handbook-owned identity, validation, and mutation semantics whenever those are available.

### 5.2 Scope

A **Scope** identifies the semantic subject, concern, or location to which work, context, or an artifact applies.

**Authority:** Handbook.

Examples may include a Workspace-wide concern, architectural area, feature, design area, contract family, Work Item, or task-level concern. The exact hierarchy and allowed dimensions are Handbook-owned.

Scope answers:

> What does this concern semantically?

Scope does not answer where execution occurs. That is the role of a Lane.

### 5.3 Resolution

**Resolution** is the Handbook-owned process that determines what semantic material is applicable for a given subject, purpose, posture, and bounded request.

Resolution does not mean “load the entire Workspace.” It may narrow, expand, omit, or project material according to Handbook rules.

A Resolution result is not automatically the exact content delivered to a model. Substrate may subsequently assemble a bounded runtime Context Pack from the applicable Handbook output and other authorized runtime material.

### 5.4 Resolution Posture

A **Resolution Posture** is a default semantic orientation associated with a Workstream, Surface, or Invocation.

Examples may include workspace orchestration, implementation, review, design, or contract remediation.

A Resolution Posture guides a new Resolution request. It is not a frozen context package and does not cause every Invocation in a Workstream to receive identical material.

### 5.5 Projection

A **Projection** is a Handbook-produced representation of canonical semantic state for a particular purpose, audience, Resolution, or interface.

A Projection MAY be filtered, composed, normalized, condensed, or otherwise purpose-specific.

Therefore:

> **Projection is not the canonical Artifact.**

Displayability does not imply editability. Collider MUST NOT assume that arbitrary edits to rendered Projection text can be round-tripped into canonical Handbook state.

A Projection is editable only when Handbook exposes an appropriate structured mutation, candidate, patch, or promotion path.

### 5.6 Work Item

A **Work Item** is a durable semantic object representing an outcome, obligation, task, packet, goal, remediation, or other unit of work.

**Primary authority:** Handbook, or another explicitly designated semantic owner surfaced through Handbook.

Specific Work Item kinds and lifecycle states are outside this glossary and SHOULD be defined by their owner.

A Work Item answers:

> What durable outcome is being pursued?

A Work Item does not define an execution environment and is not an interaction transcript.

One Work Item MAY be associated with multiple Workstreams and multiple Runs.

### 5.7 Work

**Work** is an aggregate UI or documentation label for Work Items and related status. It is not, by itself, a separate canonical entity.

A `Work` section in the Workspace Navigator SHOULD be a projection over owner-defined Work Items and relevant execution state. Collider MUST NOT create a competing informal task system merely to populate that section. The architecture reserves this section, but a populated dedicated `Work` Navigator section is secondary to the first coherent V1 slice until its Work Item kinds and projections are sufficiently defined.

### 5.8 Contract

A **Contract** uses Handbook's canonical contract-membrane meaning.

Collider may render Contract definitions, applicability, claims, status, associated Work Items, Evidence, verdicts, and gates. Collider does not define or evaluate Contract semantics independently.

### 5.9 Evidence

**Evidence** is proof admitted or normalized under Handbook's evidence semantics.

Raw logs, tool outputs, diffs, test output, and execution receipts are not automatically admitted Evidence. They may be candidates or inputs from which Handbook accepts or derives Evidence.

### 5.10 Verdict

A **Verdict** is a Handbook-owned evaluation result produced under the applicable Contract, claim, Evidence, and gate semantics.

Collider may display a Verdict but MUST NOT infer one from visual status or raw execution output.

### 5.11 Gate

A **Gate** is a Handbook-owned enforcement or promotion condition evaluated from authoritative semantic state.

A failed test, blocked execution, or warning in Collider is not automatically a Handbook Gate unless Handbook identifies it as one.

### 5.12 Receipt

A **Receipt** is a durable record that an operation, tool call, process, mutation, or other execution event occurred.

**Primary authority:** Substrate for execution receipts.

A Receipt may become input to Handbook Evidence admission, but:

> **Receipt is not automatically admitted Evidence.**

## 6. Lane, Workstream, and Execution Terms

### 6.1 Lane

A **Lane** is a durable, Substrate-owned execution partition within a Workspace.

A Lane determines where and under what operational conditions environment-touching work executes.

A Lane MAY define or reference:

- a purpose;
- a Worktree, branch, checkout, or repository binding;
- a working directory;
- an execution environment;
- runtime and tool configuration;
- a policy or capability projection;
- active and historical execution;
- relevant Scope or Work Item associations.

A Lane MAY exist and continue operating with no open Tab, Surface, or Workstream view.

A Lane does not own:

- an interaction transcript or Activity Projection;
- a Workstream;
- a Tab;
- a Block;
- a Surface;
- Handbook semantic authority.

A Lane answers:

> Where and under what conditions does execution happen?

### 6.2 Workstream

A **Workstream** is a durable or reconstructable interaction context through which users and applications direct, observe, and revisit work. Its visible sequence is an ordered **Activity Projection** rendered through a shared Workstream Surface.

A Workstream MAY have or reference:

- a stable identity or reconstruction key;
- a Resolution Posture;
- one or more associated Work Items or Scopes;
- an optional mutable default Lane binding;
- an internal Lane, Scope, Work Item, Qualified Session association, purpose, or other projection filter;
- Invocations and Activity Items relevant to the projection;
- Surfaces used to present or interact with projected activity.

A Workstream does **not** imply ownership of one independent event log. Its visible activity may be derived from records and references owned by Substrate, Handbook, MCP Apps, shell or PTY facilities, Collider, or other authorized providers.

Whether a Workstream is persisted as an independent entity, a projection definition, a materialized index, or a hybrid is unresolved. That persistence choice MUST NOT change its product-level role.

A Workstream does not own:

- a Worktree;
- an execution environment;
- an agent or process lifecycle;
- provisional Run state;
- a Lane;
- the canonical Source Records it projects;
- canonical Handbook state.

A Workstream answers:

> Through which interaction context and activity projection is this work being directed, observed, and revisited?

The Control Plane is the canonical Lane-unfiltered Workstream Surface. A Lane view SHOULD use the same core Workstream implementation with an internal filter and default binding for the selected Lane.

### 6.3 Lane versus Workstream

The operational difference is:

> **A Lane is where execution happens. A Workstream is where interaction and projected activity unfold.**

| Dimension | Lane | Workstream |
|---|---|---|
| Primary purpose | Establish execution conditions | Provide interaction continuity and an ordered Activity Projection |
| Primary authority | Substrate | Collider for projection and presentation; source systems retain record authority |
| Defines or references | Worktree, branch, working directory, environment, policy/capability projection, execution | Projection posture and filters, Invocations, Activity Items, Surfaces, Work Items, Scopes, and execution references |
| Executes work | Hosts environment-touching execution | Dispatches, observes, or links to execution |
| Owns a transcript | No | Presents a coherent ordered view; does not necessarily own the underlying source histories |
| Requires a visible Tab | No | No |
| May exist without the other | Yes | Yes |
| Typical example | `auth-refactor` execution partition | Control Plane or `Authentication` Lane-filtered interaction view |

The relationship is not one-to-one:

- one Lane MAY appear in multiple Workstream projections;
- one Workstream MAY observe or orchestrate activity associated with multiple Lanes;
- the Control Plane MAY show Lane-associated and Workspace-level Activity Items together;
- an individual environment-touching execution MUST still bind to exactly one Lane under the eventual execution model.

The exact association rules that determine when a Workspace-level Invocation and its later Lane-bound activity appear in one or more Workstream projections remain open.

### 6.4 Lane Binding

A **Lane Binding** associates an Invocation or execution instance with a specific Lane.

Three bindings MUST be distinguished conceptually:

1. **Workstream default Lane** — a mutable convenience used when new Invocations do not specify another target.
2. **Invocation target Lane** — the Lane resolved or selected for a particular accepted action.
3. **Execution Lane Binding** — the immutable execution record identifying where environment-touching work actually executed.

Changing a Workstream's default Lane MUST NOT rewrite historical Invocation or execution bindings.

An execution that reads from or mutates an execution environment MUST have exactly one Lane Binding before that environment access occurs. The final execution-instance vocabulary and orchestration topology remain provisional.

### 6.5 Invocation

An **Invocation** is one accepted request initiated through a Workstream or Surface and admitted by Substrate for routing to semantic, runtime, repository, provider, or other owner-system behavior.

Examples include:

- a shell command;
- a natural-language agent request;
- a Collider or Substrate command that requests owner-system behavior;
- a Handbook query or mutation request;
- an action initiated from a Surface that requests owner-system behavior.

An Invocation SHOULD record or reference:

- stable identity;
- origin Workstream or Surface;
- user or application input;
- time and actor;
- applicable Scope and Resolution request/reference;
- selected or resolved Lane Binding, if any;
- resulting Source Records, Activity Items, outputs, or references;
- an optional execution reference using the eventual execution vocabulary;
- provenance when derived from an earlier selection or Invocation.

An Invocation MAY produce no provisional Run or other execution instance, but it MUST cross an owner-system boundary. Purely local UI changes are Presentation Actions rather than Invocations. Editing or reissuing an earlier Invocation MUST create a new derived Invocation rather than rewrite the historical one.

### 6.6 Command

A **Command** is an Invocation payload expressed in a recognized command dialect.

Possible dialects include raw shell syntax, Collider commands, Substrate commands, or Handbook-oriented commands.

Collider MAY enhance the presentation of a Command's result, but it MUST preserve the command's actual execution semantics, output, exit status, and Receipt when the Command is executed as shell or Substrate work.

### Provisional status of Sections 6.7–6.14

> **Exploratory draft — non-canonical.**
>
> Sections 6.7–6.14 record the current working model for execution instances, execution hierarchy, and agent participation. The names, boundaries, cardinalities, ownership, nesting, and lifecycle relationships among **Run**, **Root Run**, **Child Run**, **Run Node**, **Run Tree**, **Agent**, **Subagent**, and **Subtask** are not frozen. They MUST NOT yet be treated as settled schemas, protocol types, persistence entities, compatibility commitments, or implementation authority. Later design work may rename, merge, split, redefine, or replace these concepts.
>
> Surrounding Collider invariants—such as presentation not owning execution lifetime and closing UI not implicitly cancelling background work—may remain valid independently. This draft cluster does not settle the execution model or vocabulary that will ultimately realize those invariants.

### 6.7 Run *(Provisional Draft)*

A **Run** is a durable execution instance managed or observed by Substrate.

A Run MAY execute an agent, workflow, process, tool sequence, gate evaluation, or orchestration operation.

A Run has a lifecycle independent of any Collider component displaying it. Unmounting a React component, closing a Tab, closing a Surface, or leaving a Workstream MUST NOT implicitly stop the Run.

A Run MAY produce:

- Child Runs or other Run Nodes;
- tool and process activity;
- file or artifact changes;
- logs and Receipts;
- candidate Evidence;
- results and errors;
- Source Records or references eligible for one or more Activity Projections.

A Run is not a chat session and is not an Agent identity.

### 6.8 Root Run *(Provisional Draft)*

A **Root Run** is the top execution node associated with one Invocation.

A Root Run may directly execute work or may orchestrate Child Runs. A Root Run that is not Lane-bound MUST NOT directly access an execution environment.

### 6.9 Child Run *(Provisional Draft)*

A **Child Run** is a Run whose parent is another Run.

Child Runs support delegation, parallelism, review, verification, or other bounded execution beneath a user-meaningful Root Run.

Child Runs SHOULD remain nested under their parent in default Collider views unless they become independently meaningful to the user.

### 6.10 Run Node *(Provisional Draft)*

A **Run Node** is any addressable node in a Run Tree.

A Run Node may represent:

- an agent execution;
- a Child Run;
- a tool operation;
- a process;
- a review step;
- a gate evaluation;
- another typed execution activity.

Not every Run Node needs to appear as a top-level user-facing agent or Workspace object.

### 6.11 Run Tree *(Provisional Draft)*

A **Run Tree** is the parent/child execution topology rooted at a Root Run.

A Run Tree preserves the relationship among a user-meaningful execution and its internal agents, subtasks, tools, reviews, and verification steps.

Default Collider views SHOULD emphasize the Root Run and progressively disclose internal Run Nodes.

### 6.12 Agent *(Provisional Draft)*

An **Agent** is an execution actor or harness capable of participating in one or more Runs.

An Agent is not itself a Run, Workstream, Lane, Tab, or Work Item.

A single Agent identity MAY participate in multiple Runs. A Run Tree MAY contain multiple Agent-backed nodes.

### 6.13 Subagent *(Provisional Draft)*

A **Subagent** is an Agent-backed Child Run or Run Node delegated by another Run.

Subagents SHOULD normally remain nested under the user-meaningful parent Run. Collider MUST NOT create a new top-level Tab, Lane, or Workstream for every internal Subagent by default.

### 6.14 Subtask *(Provisional Draft)*

**Subtask** SHOULD be reserved for a semantic child Work Item when the governing work model defines one.

Ephemeral internal execution should instead use **Child Run**, **Run Node**, **operation**, or another precise runtime term. This prevents semantic work decomposition from being confused with internal agent machinery.

### 6.15 Source Record

A **Source Record** is a durable record, durable object revision, or durably addressable reference produced and governed by an authorized source system.

Examples may include:

- a Substrate Invocation, Receipt, execution record, or structured result;
- a Handbook Artifact, snapshot, delta, verdict, or Evidence reference;
- an MCP App object identity or revision reference;
- a shell or PTY command record, output reference, or structured summary;
- a Collider Presentation Action or presentation-state reference.

Source Records do not need to share one schema or storage engine. Their meaning, lifecycle, and mutation authority remain with their producing systems.

A Source Record SHOULD expose stable owner-system identity, revision, ordering, or causal metadata where its source model provides those concepts. The same Source Record or durable reference MAY participate in more than one Projection Definition or Activity Projection without transferring authority to the projection machinery.

Not every transient terminal byte, UI callback, or internal telemetry item must become an individually durable Source Record. The capture and retention boundary remains open.

### 6.16 Activity Projection

An **Activity Projection** is a user-facing view with deterministic presentation order, produced by applying one or more host-recognized Projection Definitions to eligible Source Records, durable references, or reconstructable owner-system objects for a particular target view.

Its presentation order MUST be sufficient for deterministic rendering and navigation for a given projection state and Projection Definition version. That order does not imply that the underlying Source Records possess one authoritative total order; source ordering and causality may remain partial, concurrent, late-arriving, or owner-specific.

An Activity Projection MAY be shaped by:

- Workspace;
- Lane association or Lane filter;
- Scope or Work Item;
- purpose or Resolution Posture;
- Qualified Session association;
- user visibility and progressive-disclosure rules;
- another declared projection dimension.

The Control Plane uses a Workspace-relevant, Lane-unfiltered Activity Projection. A Lane Workstream Surface uses the same general model with an internal Lane filter and default Lane binding.

Different Activity Projections MAY consume some of the same Source Records while applying different Projection Definitions, filters, correlation rules, visibility rules, Business Identities, or projected locations. Exact Activity Item identity sharing across projections remains unresolved.

Projection assembly SHOULD be deterministic and replayable to the extent that the contributing source systems provide stable identity, revision, ordering, and reconstruction semantics. Incremental append and older-history loading SHOULD preserve unaffected projected identity and fold state where possible. Pagination, replay, resynchronization, and gap repair are explicit projection operations rather than renderer-side rescans of raw heterogeneous history.

An Activity Projection does not become the authority for the Source Records it selects. It also does not imply one canonical Workspace event log.

The persistence, indexing, ordering, federation, retention, compaction, and invalidation model for Activity Projections is unresolved.

### 6.17 Activity Item

An **Activity Item** is an addressable projected business object within an Activity Projection.

An Activity Item MUST have or derive stable Business Identity within its applicable Projection Definition and target view. That identity is independent of visual list position, page boundaries, current renderer, and the order in which records happen to arrive. An object without stable Business Identity is not an Activity Item; intentionally ephemeral output may render directly as a Block without becoming one.

An Activity Item MAY be backed by:

- one Source Record;
- several correlated Source Records or revisions folded into one projected object;
- a stable reference plus a reconstruction recipe;
- a provider-owned object and revision;
- a structured summary derived from a larger output stream.

An Activity Item may carry projection metadata such as Workspace, Lane, Scope, Work Item, Qualified Session association, causal relationship, ordering information, visibility, display hints, Projection Definition identity or version, and inspectable references to its contributing Source Records.

Where the source and Projection Definition semantics permit it, an Activity Item SHOULD preserve its Business Identity through incremental append, older-page loading, pagination, replay, and renderer changes. A Block is the current visual rendering of an Activity Item. Changing the Block renderer does not change the Activity Item's Business Identity or the underlying Source Records.

Whether Activity Items themselves are materialized, indexed, or computed on demand, and whether equivalent items share identity across different Activity Projections, remain open.

### 6.18 Chat, Conversation, Qualified Session, and Session History

**Chat** and **Conversation** describe interaction styles, not the canonical durable architecture.

A **Qualified Session** is an owner-specific continuity resource whose type is identified by the relevant provider or runtime, such as a `model-provider session`, `agent-harness session`, `shell session`, or `PTY session`. Its identity, lifecycle, resumability, and continuity guarantees belong to that provider or runtime.

**Session** MUST NOT be used unqualified as a synonym for Workstream, Lane, Invocation, Activity Projection, or provisional Run. A Qualified Session does not become authority for Handbook semantic truth, Lane execution placement, Substrate policy, or Collider Workstream identity.

A **Session History** is an owner-scoped record or reconstructable history sufficient to support the continuity guarantees of a Qualified Session. A Session History MAY use event sourcing when the owning provider or runtime supplies those semantics, and MAY contribute Source Records to one or more Activity Projections where policy permits. It is one possible source history, not the canonical Workspace history, and it does not decide the Workspace-history architecture.

Collider's default interaction model is resolution-reconstructed: each Invocation may receive newly resolved and assembled context rather than automatically continuing a prior Qualified Session. Qualified Session association SHOULD remain traceable on relevant Source Records, Activity Items, and Context Records where the owning system and policy permit it.

### 6.19 Session Continuity

**Session Continuity** is an explicit, user-visible request to bind one or more Invocations to a Qualified Session by creating, resuming, reusing, or continuing it.

Remaining in the same Workstream, Lane view, Tab, or adjacent message sequence MUST NOT by itself imply Session Continuity. A Workstream may project activity associated with zero, one, or several Qualified Sessions over time, and one Qualified Session's activity MAY appear in several Workstream projections where policy and visibility rules permit.

Session Continuity may preserve provider- or runtime-owned state only to the extent promised by the Qualified Session. It MUST NOT silently preserve or bypass current validation of Handbook Scope or Resolution, Lane binding, Substrate policy, Capability Grants, permissions, provider availability, or model identity and configuration.

Collider SHOULD preserve room for one-Invocation continuation and a future session-oriented continuity mode while keeping resolution-reconstructed behavior as the default. A dedicated **Sessions** navigator destination is part of the product direction; users MAY prioritize it in their own Navigator Layout. Its entries MUST identify the relevant provider or runtime, and opening or moving that destination MUST NOT itself resume a Qualified Session. The exact continuation commands, UI states, provider guarantees, resume handles, rebinding rules, failure behavior, and delivery slice remain open.

### 6.20 Projection Definition

A **Projection Definition** is a host-recognized, version-addressable declaration describing how eligible Source Records or references become stable projected business objects for a target Activity Projection.

A Projection Definition MAY be built into the host or dynamically registered through an authorized mechanism. Dynamic registration authority, lifecycle, compatibility, and migration remain open.

A Projection Definition MAY declare:

- source eligibility and matching;
- correlation and Business Identity formation;
- deterministic fold or update behavior;
- target-view visibility and materialization rules;
- ordering or projected-location metadata;
- Activity Item kind and renderer-facing output shape;
- source-reference and fold-provenance requirements.

A Projection Definition does not own the Source Records it consumes and does not become semantic or execution authority. Its registration authority, compatibility rules, versioning, and invalidation behavior remain open.

### 6.21 Business Identity

**Business Identity** is the stable domain identity used by a Projection Definition to correlate records or revisions that represent the same projected business object.

Business Identity MUST be independent of visual list position. It MAY be supplied by an owner system or deterministically derived from stable source fields. It is projection identity, not owner-system authority, and it does not by itself imply that two Activity Projections share one Activity Item identity.

### 6.22 Activity Assembler

An **Activity Assembler** is the technology-neutral projection mechanism that applies host-recognized Projection Definitions to eligible Source Records or references and produces or updates Activity Items.

An Activity Assembler MAY maintain projection-specific fold state and Projection Indexes for initial reconstruction, incremental append, older-history loading, pagination, replay, resynchronization, and gap repair. It does not own source-system meaning or lifecycle, and its process, language, placement, and persistence topology remain unresolved.

### 6.23 Projection Index

A **Projection Index** is derived, projection-specific state used to correlate Source Records, locate existing Activity Items, maintain fold state, preserve predecessor or ordering relationships, and update only affected projected objects.

A Projection Index SHOULD remain rebuildable or otherwise reconcilable from its declared source and snapshot inputs to the extent permitted by the chosen source and persistence architecture. It is not a source of semantic or execution authority. Its materialization, storage, snapshot, compaction, privacy, and retention behavior remain open.

## 7. Presentation and Interaction Terms

### 7.1 Block

A **Block** is an inline visual unit rendered within a Workstream Surface.

A Block MAY present:

- user input;
- command or shell output;
- an agent response;
- an execution update;
- a file or diff;
- a static summary;
- a rich interactive Surface.

A Block is a presentation object. It does not own the Activity Item, Artifact, Lane, provisional Run, Work Item, Source Record, provider object, or other durable state it displays.

Intentionally ephemeral visual output MAY render directly as a Block without becoming an Activity Item. A durable or reconstructable projected object MUST be represented as an Activity Item with stable Business Identity before it is rendered as a Block. A durable or reconstructable Block SHOULD be traceable to one or more Activity Items, Source Records, durable references, or owner-system objects sufficient to explain or reconstruct it. The relationship need not be one Block to one Source Record, and one Activity Item may have different Block renderings or placements over time.

A Workstream Block renderer SHOULD consume a final Activity Item or bounded Surface view model. It SHOULD NOT repeatedly scan arbitrary raw heterogeneous source collections to rediscover correlation and fold state that belongs to a Projection Definition and Activity Assembler.

### 7.2 Surface

A **Surface** is a typed, addressable application or view rendered by Collider.

A Surface SHOULD have or reference:

- a stable identity;
- a provider or application kind;
- its Workspace binding;
- optional Scope, Work Item, Lane, Workstream, Invocation, provisional Run, Artifact, Repository, Qualified Session, or Activity Item bindings;
- a durable state reference or reconstruction recipe;
- supported presentation modes and actions;
- declared capabilities.

Examples include:

- a Handbook Artifact viewer;
- a Contract browser;
- a Lane overview;
- a Workstream view;
- a repository browser;
- a source editor;
- a plan;
- test results;
- an active-execution view;
- a terminal or PTY.

A Surface renders owner-system state but MUST NOT become the owner of that state merely because it is interactive.

### 7.3 Surface Provider

A **Surface Provider** is the implementation that creates or renders a type of Surface.

A Surface Provider may be:

- Collider-native;
- Substrate-provided;
- Handbook-provided;
- adapted from an MCP App or another extension protocol.

Collider's core Surface model MUST remain protocol-neutral. MCP is one possible provider or transport mechanism, not the definition of a Surface.

### 7.4 Surface Placement

A **Surface Placement** is one presentation of a Surface in a particular UI host.

The principal placement types are:

- an inline placement in the chronological Workstream flow;
- a Sticky Placement in the Workstream's Sticky Stack;
- a focused placement in a center-workspace Tab.

A Surface MAY have more than one placement. Creating or removing a placement does not create, duplicate, or destroy the owner-system state represented by the Surface.

### 7.5 Tab

A **Tab** is a center-workspace host for a focused Surface Placement.

A Tab is not a Lane, Workstream, file, process, Agent, provisional Run, or state owner.

Closing a Tab MUST remove only that placement. It MUST NOT implicitly:

- stop execution;
- retire a Lane;
- archive a Workstream;
- delete an Artifact;
- remove a Work Item;
- terminate a worktree or process;
- erase Source Records or interaction history.

A Tab MAY display a Workstream Surface, Lane Surface, Handbook Surface, result Surface, or another application type.

### 7.6 Tab Promotion / Open as tab

**Tab Promotion** is the Presentation Action that creates a Tab placement for an eligible Surface currently shown inline or discovered elsewhere.

The preferred user-facing label is **Open as tab**. Dragging a Surface into the Tab bar MAY invoke the same action, but drag MUST NOT be the only available mechanism.

Tab Promotion is not ownership transfer and need not physically remove the originating Block. The originating Block SHOULD remain as a compact chronological anchor or reference.

The Tab placement SHOULD refer to the same Surface identity or to a provider-declared canonical expanded Surface. The exact identity and state-sharing contract remains open.

### 7.7 Sticky Placement

A **Sticky Placement** is a Workstream-local placement that keeps an eligible Block or Surface visible while the chronological activity flow scrolls beneath it.

Creating a Sticky Placement MUST NOT remove or relocate the original chronological Block. It creates an additional presentation reference.

Stickiness is local to the relevant Workstream Surface or projection placement. If the same Activity Item appears in both the Control Plane and a Lane-filtered Workstream, it may be sticky in one and not the other.

A Sticky Placement is not a Tab placement and does not change Source Record, Lane, execution, Work Item, or Handbook state.

### 7.8 Sticky Stack

A **Sticky Stack** is the small ordered collection of Sticky Placements attached to a Workstream Surface.

Collider SHOULD keep the Sticky Stack deliberately bounded and SHOULD compact it when its contents would consume excessive center-workspace space. A likely design range is two or three fully visible items, but the exact capacity, compaction behavior, persistence, and user controls are not frozen.

### 7.9 Workstream Surface

A **Workstream Surface** is the shared Collider Surface that renders an Activity Projection as Blocks and provides a Composer for new Invocations.

The Control Plane and Lane views SHOULD use the same core Workstream Surface implementation, Block renderer, Surface host, selection behavior, Sticky Stack, and Composer infrastructure. Their principal difference is projection and binding posture rather than a separate application architecture.

### 7.10 Control Plane

The **Control Plane** is the canonical Workspace-level Workstream Surface opened by default in Collider V1.

The Control Plane:

- begins with a broad Workspace-orchestration Resolution Posture;
- uses a Workspace-relevant Activity Projection with no Lane filter;
- has no implicit default Lane unless one is explicitly selected or established;
- may inspect, coordinate, and dispatch work across Lanes;
- may host terminal, chat-like, Handbook, execution, and other rich Blocks;
- is not itself a Lane;
- is not required to be a dashboard;
- is not architecturally equivalent to one shell or PTY session.

`No Lane filter` does not require exposing every low-level or internal record. Default projection rules MAY collapse or omit internal machinery while preserving inspectability.

For V1, the Control Plane SHOULD initially appear as a minimal block-oriented Workstream Surface with a terminal-like Composer.

### 7.11 Lane Workstream Surface

A **Lane Workstream Surface** is a Workstream Surface whose Activity Projection is internally filtered to a selected Lane and whose Composer normally uses that Lane as its default binding.

It SHOULD reuse the same core components as the Control Plane. The Lane filter does not need to be exposed as public query syntax.

A Lane Workstream Surface is not the Lane itself. Closing it does not stop or retire the Lane or its execution.

The exact inclusion rules for Workspace-level Invocations that later dispatch into the Lane, cross-Lane orchestration, shared Activity Items, and historical rebinding remain open.

### 7.12 Composer

The **Composer** is the input surface through which a user submits a new Invocation to the active Workstream.

The Composer MAY accept shell commands, Collider or Substrate commands, natural-language requests, structured actions, or application-generated input.

The Composer's routing behavior does not change the definitions of Invocation, Lane Binding, Activity Projection, or provisional Run.

### 7.13 Terminal Surface

A **Terminal Surface** is a Surface that provides raw or enhanced shell or PTY interaction.

A Terminal Surface is one application type inside Collider; it is not the architectural definition of the Control Plane or Workstream.

When operating in raw shell or PTY mode, normal shell semantics MUST be preserved.

### 7.14 Workspace Navigator

The **Workspace Navigator** is the persistent left-side Collider Surface for discovering Workspace destinations and arranging access to them through Navigator Items and Navigator Groups.

Its conceptual destinations include the Control Plane, Inbox, Handbook, Work, Lanes, execution activity, Qualified Sessions, Agent Harnesses, environments and Environment Dependencies, Repositories, Agent Profiles, Workspace configuration and policy, capabilities and integrations, credential bindings, and diagnostics. This is destination coverage, not a fixed root-level tree or a requirement to deliver every destination in the first slice; Foundations Section 8 governs product scope and defaults.

Collider defines eligible items and safe composition rules. Users control supported grouping, group labels, item ordering and visibility, and whether an item appears ungrouped. These Presentation Actions MUST NOT change underlying identity, ownership, permissions, capability enablement, execution binding, or lifecycle.

The default organization SHOULD foreground durable Workspace structure rather than chronological chat titles. Users MAY prominently expose the Sessions destination, including at the root, without making a Qualified Session a Workstream or changing the resolution-reconstructed default. The highest-level default naming rule is defined in Section 12; it does not constrain every descendant or user-customized label.

### 7.15 Inspector

The **Inspector** is an optional, selection-driven contextual surface, typically rendered in the right sidebar.

It may inspect objects, provisional Runs, Context Records, Source Records, or activity. It is not the primary owner or canonical list of active execution, and V1 essential behavior MUST NOT depend on it.

### 7.16 Active Execution Indicator

An **Active Execution Indicator** is a persistent, lightweight Collider presentation of Workspace-wide execution status.

It may summarize running, queued, blocked, failed, or attention-requiring user-meaningful execution and provide access to a richer execution Surface.

The indicator displays Substrate state; it does not own that state. The final execution terminology remains provisional.

### 7.17 Reopen and Reconstruct

To **Reopen** a Surface means to create a new placement for an existing or reconstructable Surface identity.

To **Reconstruct** a Surface means to render its current useful view from owner-system state, Source Records or durable references, Activity Projection metadata, and saved presentation references.

Reconstruction does not require restoring the exact in-memory component tree, copying every provider payload into Collider, or treating an old chat transcript as the sole source of context.

### 7.18 Presentation Action

A **Presentation Action** is a Collider-owned interaction that changes only Collider presentation state and does not request semantic, runtime, repository, provider, or other owner-system behavior.

Examples include making or removing a Sticky Placement, using **Open as tab**, collapsing or expanding a Block, changing focus, changing a local selection, and creating or renaming Navigator Groups or regrouping, reordering, showing, or hiding Navigator Items. These changes affect Collider presentation only. Enabling a Workspace integration, changing an Agent Profile or Substrate policy, realizing Environment Dependencies, or resuming a Qualified Session requests owner-system behavior and therefore requires an Invocation rather than a mere layout change.

A Presentation Action MAY be retained as a Collider-owned Source Record when restoration, audit, provenance, or durable user history requires it. Recording the action does not transfer its presentation-state authority to Substrate or another source system.

### 7.19 Navigator Item

A **Navigator Item** is an identity-bearing destination or reference offered through the Workspace Navigator. It may open an appropriate Surface or browse owner-defined resources. Its identity MUST remain independent of its display label, group membership, ordering, and visibility.

An item may appear in a Navigator Group or ungrouped at the root. Moving or hiding its presentation does not rename, disable, delete, authorize, or rebind the target object. **Remove from navigator** removes or hides the presentation reference; it is not an owner-system deletion or capability-disable operation. Eligible hidden destinations MUST remain recoverable through a discoverable host path.

A Navigator Item is not itself the target Surface or owner-system object. Whether multiple shortcuts share one item identity or use separate references remains an implementation question under C-OQ-025.

### 7.20 Navigator Group

A **Navigator Group** is a user-visible presentation container for organizing Navigator Items. It is not a Handbook Scope, execution partition, permission boundary, or configuration owner.

Collider supplies default groups; users MUST be able to create and rename groups and change their displayed membership within supported navigation rules. Removing a group MUST NOT delete or disable the destinations it previously displayed. User-defined names are not subject to the one-word default naming convention.

### 7.21 Navigator Layout

A **Navigator Layout** is the Collider-owned presentation arrangement of Navigator Groups, grouped and ungrouped Navigator Items, display labels, ordering, visibility, and related local view preferences.

Users MUST be able to customize that arrangement and recover hidden eligible items or restore a default arrangement without modifying represented owner-system state. Layout changes MUST NOT silently change another user's arrangement or be written as Substrate operational configuration or policy. Preference persistence, app/user/Workspace scoping, synchronization, and explicitly shared defaults remain open under C-OQ-025.

Host-declared Extension Points still govern eligible contributions and safe composition. Contribution Providers MUST NOT overwrite user organization or restore hidden items merely because a contribution reloads. Navigation customization does not grant capabilities and MUST preserve access to required execution awareness, cancellation, and recovery.

### 7.22 Inbox

**Inbox** is the preferred default destination label for notifications and actionable items drawn from their owner systems. It is a presentation and access point, not a new authority for approvals, obligations, Work Items, gates, or execution status. Users may choose a display label such as **Notifications**.

Collider-local read or dismissal state is presentation state and MUST NOT implicitly approve a request, satisfy a Gate, unblock execution, or resolve an owner-system obligation. Any acknowledgment or resolution that changes owner-system state must use that owner's authorized operation. Source selection, delivery, grouping, retention, and exact read/dismiss semantics remain open under C-OQ-026.

## 8. Context and Inspection Terms

### 8.1 Resolved Set

A **Resolved Set** is the semantic material Handbook identifies as applicable to a bounded Resolution request.

The Resolved Set may be broader than the material ultimately delivered to a model or agent.

### 8.2 Context Pack

A **Context Pack** is the Substrate-assembled runtime payload prepared for a specific model or agent invocation.

A Context Pack may contain:

- Handbook Projections or references;
- current Work Item and Scope state;
- Lane and repository state;
- authorized runtime instructions;
- selected prior results, decisions, or Receipts;
- other bounded execution material.

A Context Pack is specific to a particular execution step. It is not the permanent memory of a Workstream.

### 8.3 Injected Context

**Injected Context** is the exact content and metadata made model-visible for a particular model invocation.

Injected Context is a subset or transformation of the available Resolved Set and runtime material.

### 8.4 Acquired Context

**Acquired Context** is material observed after execution begins through authorized tools, file reads, queries, browser operations, agent messages, or other runtime interactions.

Acquired Context MUST be distinguished from Injected Context because it entered execution through a different provenance path.

### 8.5 Produced Material

**Produced Material** is output created by execution, including responses, patches, files, decisions, test results, Receipts, candidate Evidence, or other artifacts.

Produced Material does not become canonical Handbook state or admitted Evidence merely because execution created it.

### 8.6 Context Record

A **Context Record** is the durable linkage needed to explain what an execution step or model invocation was resolved against, what was injected, what was acquired, and what it produced. It SHOULD be traceable to the relevant Source Records and Activity Items without requiring Collider to own their canonical payloads.

The detailed Collider Inspector for Context Records may be deferred, but the underlying identities and references SHOULD be captured from the first V1 implementation.

The canonical distinctions are:

```text
Resolved   what Handbook determined was applicable
Injected   what Substrate actually delivered to the model
Acquired   what execution later observed
Produced   what execution created
```

## 9. Lifecycle Verbs

The following verbs MUST remain distinct.

### 9.1 Open

**Open** creates or focuses a presentation of an existing entity or Surface.

Opening does not create a new Lane, Workstream, or execution instance unless the invoked action explicitly requires one.

### 9.2 Close

**Close** removes or hides a UI placement such as a Tab, drawer, Sticky Placement, or Surface placement.

Close does not stop execution or delete durable state.

### 9.3 Stop or Cancel

**Stop** or **Cancel** requests that Substrate end active execution according to its runtime cancellation semantics.

Closing its UI does not imply cancellation. The exact parent/child cancellation topology remains provisional.

### 9.4 Archive

**Archive** removes a durable or reconstructable interaction context such as a Workstream from normal active navigation while retaining required identity, history, references, or reconstruction metadata.

Archiving a Workstream does not delete its historical Invocations or owner-system Source Records.

### 9.5 Retire

**Retire** transitions a Lane out of normal availability for new execution while preserving its identity and historical bindings.

The exact treatment of its Worktree and other resources is an execution-policy concern and MUST be explicit.

### 9.6 Delete

**Delete** is an explicit destructive operation against an owner-system entity or presentation record.

Delete MUST NOT be inferred from Close, Archive, Retire, removal from the Sticky Stack, Tab closure, or navigation changes.

### 9.7 Make Sticky

**Make Sticky** is a Presentation Action that creates a Sticky Placement for an eligible Block or Surface in the current Workstream Surface.

It does not remove the chronological placement or create a Tab.

### 9.8 Remove Sticky

**Remove Sticky** is a Presentation Action that removes only the Sticky Placement from the current Workstream Surface.

It does not delete the original Activity Item, Block history, Surface, or owner-system state.

### 9.9 Open as tab

**Open as tab** is a Presentation Action that creates or focuses a Tab placement for an eligible Surface.

It does not make the Surface sticky in a Workstream and does not transfer ownership of the underlying state.

## 10. Cardinality and Relationship Rules

> Relationships in this section that depend on the execution-instance or agent-topology vocabulary from Sections 6.7–6.14 are also **Provisional Draft**. Their presence here does not promote those terms or relationships to canonical status.
>
> This section also does not select a canonical Workspace event-history architecture or a Workstream persistence model.

Collider V1 uses the following conceptual relationships:

1. A Workspace has zero or more Lanes.
2. A Workspace exposes one canonical Control Plane Workstream Surface.
3. A Workspace may expose zero or more Lane Workstream Surfaces and other Workstream projections.
4. A Lane may appear in zero or more Workstream projections.
5. A Workstream may observe or orchestrate activity associated with zero or more Lanes.
6. A Workstream has zero or more Invocations, may retain zero or more Presentation Actions, and projects zero or more Activity Items.
7. An Activity Projection may select from one or more authorized source systems or provider histories through one or more Projection Definitions.
8. The same Source Record may contribute to multiple Activity Projections without transferring authority to any Activity Assembler or Projection Index.
9. An Activity Item has stable Business Identity within its Projection Definition and target view.
10. An Activity Item may fold or correlate one or more Source Records, revisions, durable references, or reconstructable owner-system objects.
11. An Activity Item may render as one or more Blocks or Surface placements over time.
12. A Block MAY host a Surface.
13. A Surface may have zero or more inline, Sticky, or Tab placements.
14. A Sticky Placement belongs to one Workstream Surface placement and does not automatically apply to another projection of the same Activity Item.
15. A Tab hosts one primary focused Surface Placement.
16. Removing every presentation placement MUST NOT destroy the underlying Lane, execution, Workstream identity or reconstruction data, Activity Item, Artifact, Work Item, Source Record, or provider object.
17. Every environment-touching execution MUST bind to exactly one Lane under the final execution model.
18. A Workstream may project activity associated with zero, one, or several Qualified Sessions over time.
19. One Qualified Session's activity may appear in more than one Workstream projection where policy and visibility rules permit.
20. Session Continuity MAY associate multiple Invocations with one Qualified Session, but remaining in the same Workstream does not imply continuity.
21. Session History may serve as one Source Record family or owner history for Activity Projection, but it is not the canonical Workspace history.

Whether Activity Items and Workstreams are materialized, indexed, or computed on demand remains open.

## 11. Foundational Non-Equivalences

The following distinctions are normative except where a term is explicitly provisional:

```text
Workspace  ≠ Project
Workspace  ≠ Repository
Scope      ≠ Lane
Work Item  ≠ Workstream
Lane       ≠ Workstream
Lane       ≠ Tab
Workstream ≠ Tab
Workstream ≠ Canonical Workspace Event Store
Presentation Action ≠ Invocation
Navigator Item ≠ target Surface or owner-system object
Navigator Group ≠ Handbook Scope or Lane
Navigator Layout ≠ Substrate Workspace configuration or policy
Remove from navigator ≠ disable or Delete
Inbox dismissal ≠ owner-system resolution
Agent Harness ≠ Agent Profile
Environment Dependencies ≠ Repository/package dependencies or Extension Dependency Declarations
Invocation ≠ Run
Agent      ≠ Run
Run        ≠ Chat Session
Qualified Session ≠ Workstream
Qualified Session ≠ Lane
Qualified Session ≠ Invocation
Qualified Session ≠ Activity Projection
Qualified Session ≠ Run
Session History ≠ Canonical Workspace Event Store
Same Workstream ≠ implicit Session Continuity
Session Continuity ≠ automatic Scope, Lane, policy, capability, permission, or model inheritance
Source Record ≠ Activity Item
Projection Definition ≠ Source Authority
Activity Assembler ≠ Source Authority
Projection Index ≠ Source Authority
Activity Projection ≠ Source Authority
Activity Projection presentation order ≠ authoritative total Source Record order
Business Identity ≠ Visual List Position
Activity Item identity ≠ Block identity
Activity Item ≠ Block
Block      ≠ Surface
Surface    ≠ State Owner
Extension ≠ Semantic or Execution Authority
Trusted Extension ≠ Semantic Authority
UI contribution permission ≠ host runtime authority
Capability Grant ≠ Isolation Boundary
Capability mediation ≠ Isolation Boundary
Extension Context Mediation ≠ Isolation Boundary
Capability Interception ≠ Isolation Boundary
Extension Point ≠ Surface
Extension Point ≠ Surface Placement
UI Contribution ≠ Extension Point
UI Contribution Scope ≠ Handbook Scope
Contribution Provider ≠ Owner-System Authority
UI contribution permission ≠ Capability Grant or runtime authority
Extension Context ≠ Handbook Scope or Resolution
Extension Context ≠ Context Pack or Injected Context
Extension Context ≠ Activity Projection or Workstream
Extension Context Mediation ≠ Authority Transfer
Required Capability ≠ Capability Grant
Provided Capability ≠ copied owner state
Spatiotemporal Composability ≠ Isolation Boundary
Mediated Effect ≠ guaranteed exact reversion
Revertible Contribution ≠ Compensating Action
Compensating Action ≠ Exact Rollback
Temporal Composability objective ≠ complete unload guarantee
Spatiotemporal Composability objective ≠ formal proof
Sticky Placement ≠ Tab
Open as tab ≠ Make Sticky
Projection ≠ Canonical Artifact
Receipt    ≠ Admitted Evidence
Resolved Set ≠ Injected Context
Close      ≠ Stop
Remove Sticky ≠ Delete
```

## 12. Naming and Usage Rules

1. Collider documents MUST use **Workspace**, not Project, for the Substrate configuration and policy boundary.
2. **Lane** MUST refer to execution placement and conditions, not merely a work category or visible terminal.
3. **Workstream** MUST refer to an interaction context and ordered Activity Projection, not a parallel execution partition or an assumed independent event log.
4. UI copy MAY shorten Workstream to **Stream** where a visible label is useful, but architecture and protocol documents SHOULD use Workstream.
5. A phrase such as `Lane tab` means “a Tab displaying a Lane Workstream Surface”; it does not mean the Tab is the Lane.
6. **Qualified Session** MUST identify its provider or runtime type. Unqualified **Session** MUST NOT replace Workstream, Lane, Invocation, Activity Projection, or provisional Run.
7. **App** SHOULD refer to a provider or implementation. **Surface** refers to the typed rendered instance exposed through Collider.
8. **MCP App** MUST refer only to an app delivered through the MCP App mechanism. Core Collider and Handbook Surfaces are not automatically MCP Apps.
9. The preferred user-facing action for creating a Tab placement is **Open as tab**. Architecture documents MAY use **Tab Promotion**.
10. **Sticky** means keep a Block or Surface visible within a Workstream Surface while activity scrolls. It MUST NOT be used as a synonym for creating a Tab.
11. **Pin** SHOULD NOT be used unqualified in normative V1 documents because it is ambiguous between Sticky Placement and Tab Promotion.
12. **Dock** SHOULD NOT be used unqualified for Collider UI placement because Handbook's contract membrane may use Dock as a semantic term.
13. Handbook-owned terms MUST retain Handbook's canonical capitalization and meaning where available.
14. `Lane-unfiltered` describes the absence of a Lane predicate in an Activity Projection; it does not mean raw, unrestricted, or uncollapsed telemetry.
15. Unless a bounded implementation has supplied explicit evidence, assumptions, and a system boundary, Collider documents MUST describe Temporal, Spatial, and Spatiotemporal Composability as design objectives rather than proven guarantees.
16. **Extension Context** MUST remain qualified in architecture and protocol documents. Unqualified `Context` MUST NOT collapse Extension Context, Handbook Scope or Resolution, Handbook Projection, Substrate Context Pack, Injected Context, Activity Projection, Workstream, or React context into one concept.
17. **Required Capability**, **Provided Capability**, and **Dependency Declaration** are extension-composition terms. They do not by themselves create a Capability Grant, Handbook Contract, Substrate policy decision, or owner-system authority.
18. Cross-process or remote Capabilities MUST expose asynchronous, availability, failure, cancellation, timeout, and retry semantics honestly where those semantics apply; they MUST NOT masquerade as infallible synchronous local access.
19. **Extension Point** is the canonical public architecture term for a host-declared UI composition location. `Slot` remains a non-canonical implementation or reference-architecture term unless an explicit future glossary decision adopts it.
20. **UI Contribution Scope** MUST remain qualified and MUST NOT be shortened to `Scope` where it could be confused with Handbook Scope.
21. Inline, Sticky, Tab, overlay, and Inspector are presentation locations or modes; none is automatically an Extension Point merely because it can display contributed UI.
22. A Contribution Provider's right to register UI MUST NOT be interpreted as a Capability Grant, trust elevation, or owner-system authority.
23. A Projection Definition that emits an Activity Item MUST preserve owner-supplied Business Identity or deterministically derive stable Business Identity from domain or source semantics; visual list position, current page, and renderer order MUST NOT serve as business identity.
24. An Activity Projection's deterministic presentation order MUST NOT be interpreted as one authoritative total order over its underlying Source Records.
25. Workstream renderers SHOULD consume final Activity Items or bounded Surface models rather than repeatedly scan raw heterogeneous source collections.
26. Projection Definitions, Activity Assemblers, and Projection Indexes are projection machinery; none becomes semantic, execution, or Source Record authority.
27. A purely local Collider presentation-state change is a **Presentation Action**, not an Invocation. A Surface action becomes an Invocation only when it requests owner-system behavior.
28. **Session Continuity** MUST be explicit and visible; remaining in the same Workstream MUST NOT be interpreted as continuation of a Qualified Session.
29. Resuming a Qualified Session MUST NOT bypass current validation of Handbook Scope or Resolution, Lane binding, Substrate policy, Capability Grants, permissions, provider availability, or model identity and configuration.

30. Collider-shipped labels at the **highest level** of the default Workspace Navigator MUST be one word. This applies to default root-level group headings and default ungrouped root-level destinations only. Descendant labels, user-defined names, and user-promoted items are not subject to this rule; moving an item to the root MUST NOT automatically rename it.
31. Navigator display labels are not canonical terminology or identity. Architectural terms such as **Control Plane**, **Qualified Session**, and **Agent Profile** retain their names. A root display alias such as **Home** for Control Plane MAY be used without renaming that architectural concept; a nested entry may retain the full name.
32. **Sessions** is permitted as a navigator label for Qualified Session resources. It does not make Sessions the default Workspace organization, imply Session Continuity, or remove provider/runtime qualification from the represented objects.
33. **Inbox** is the preferred default destination label rather than Attention. **Notifications** remains an acceptable user label; local read/dismiss state is distinct from owner-system acknowledgment and resolution.

## 13. Deferred or Reserved Concepts

The following concepts are intentionally not fully defined by this V1 glossary and are tracked in [`02-collider-v1-open-architecture-questions.md`](./02-collider-v1-open-architecture-questions.md):

- canonical Workspace event history versus federated histories or a hybrid;
- Workstream persistence and identity;
- Projection Definition registration, ownership, versioning, compatibility, and invalidation;
- Activity Assembler placement, incremental update, pagination, replay, resynchronization, and gap-repair mechanics;
- Projection Index materialization, snapshots, compaction, privacy, retention, and reconstruction;
- Activity Projection indexing, ordering, retention, and compaction;
- Source Record normalization, Business Identity formation, cross-source identity, correlation, and cross-projection identity sharing;
- shell and PTY capture boundaries;
- MCP App object, revision, and reconstruction behavior;
- Qualified Session creation, explicit resume handles, one-Invocation continuation, persistent continuity mode, provider guarantees, Session History retention, expiry and recovery, branching or cloning, rebinding and revalidation, and session-oriented views;
- exact Control Plane and Lane projection-association rules;
- Sticky Stack capacity, persistence, and compaction;
- Tab Promotion identity and state-sharing behavior;
- final execution-instance and agent-topology vocabulary;
- a multi-Workspace Project grouping;
- pipelines and visual workflow composition;
- detailed public third-party Surface extension trust, isolation, capability, provenance, and effect-cleanup mechanisms;
- Capability identity, typed keys or schemas, compatibility and versioning, provider identity, multiple-provider selection or multiplexing, failover, and Dependency Declaration format;
- Extension Context derivation and scoping, Extension Context Mediation and Capability Interception mechanics, relationship to Substrate policy and Handbook semantics, and local versus cross-process capability bridges;
- Extension Contribution granularity, Extension Lifecycle state transitions, declared recovery criteria, exact equality versus observational equivalence, inverse or disposer composition, asynchronous teardown, partial-activation recovery, configuration reconciliation, dependency cycles, hot replacement, update rollback, and evidence required for composability claims;
- Extension Point declaration schemas, identity and naming, `Slot` as a possible implementation term, contribution kinds and scopes, multiplicity, ordering, fallback, election and replacement, nested-point lifetimes, parent replacement, rendering boundaries, styling isolation, accessibility, failure recovery, and contributed-Surface restoration;
- detailed Inspector and context-debugger UX;
- Navigator Item identity and shortcuts, default grouping, preference persistence and sharing, contribution restoration, and customization interactions under C-OQ-025;
- Environment Dependency configuration, effective requirements, World realization, readiness, and Lane relationships under C-OQ-023;
- Agent Harness and Agent Profile schemas, configuration sources and precedence, compatibility, and application to active work under C-OQ-024;
- Inbox source ownership, read/dismiss behavior, owner-system acknowledgment, delivery, and retention under C-OQ-026;
- design-system and Storybook or Open Design artifact semantics.

Later documents MAY define these concepts, but they MUST preserve the ownership and lifecycle boundaries established here.

## 14. Canonical Summary

The Collider V1 model can be summarized as:

```text
Workspace          the Substrate config/policy boundary
Agent Harness      an integrated agent-execution backend or runtime
Agent Profile      reusable Substrate-owned agent configuration
Environment Dependencies  requirements consumed by Substrate World dependency tooling
Navigator Item     an identity-bearing navigation destination or reference
Navigator Group    a presentation-only grouping of navigation items
Navigator Layout   the user's supported arrangement of navigation presentation
Extension          bounded optional or dynamically reconfigurable functionality
Extension Contribution   an attributable bounded contribution made by an Extension
Extension Point    a host-declared location for authorized UI composition
UI Contribution    a lifecycle-bound provider contribution to an Extension Point
Scope              what the work means
Work Item          what durable outcome is pursued
Lane               where execution occurs
Workstream         the interaction context and activity projection
Qualified Session  an owner-specific provider or runtime continuity resource
Session History    the owner-scoped history supporting Qualified Session continuity
Presentation Action a local Collider presentation-state change
Invocation         the accepted request for owner-system behavior
Run                what executed (provisional vocabulary)
Source Record      the owner-system record or durable reference
Projection Definition the host-recognized rules for assembling projected business objects
Business Identity  the stable projected domain identity independent of list position
Activity Assembler the mechanism that applies Projection Definitions
Projection Index   derived projection-specific correlation and fold state
Activity Projection the deterministically ordered eligible view across source records
Activity Item      one stable projected business object
Block              the current rendering of an Activity Item or app inline
Surface            the typed app/view being rendered
Sticky Placement   a Workstream-local always-visible placement
Tab                a focused center-workspace Surface placement
```

The central lifecycle rule is:

> **Collider presents and controls durable Handbook and Substrate state; its Activity Projections, Blocks, Surfaces, Sticky Placements, and Tabs do not become the authority for that state.**

The central Lane/Workstream rule is:

> **A Lane is where execution happens. A Workstream is where interaction and projected activity around the work unfold.**

The central history rule is:

> **Collider should provide one coherent interaction experience without presupposing one canonical underlying event store or duplicating owner-system authority.**

The central continuity rule is:

> **Resolution-reconstructed behavior is the default; Qualified Session Continuity is explicit, owner-specific, and does not make Session History the Workspace's canonical history or bypass current Scope, Lane, policy, capability, permission, or model validation.**

The central dynamic-extension rule is:

> **Dynamically reconfigurable Extensions should aim for Spatiotemporal Composability, but every composability claim remains bounded to a specific mediated, attributable, and validated extension class and does not replace authority, trust, or isolation boundaries.**



# SOURCE: Collider/docs/agent-native/canonical/01-collider-v1-foundations.md

Source SHA-256: 6a32874cd28bbec5347fbdb79b9f63ea4b3f8206357eee4fa4b446e39309b54a

# Collider V1 Foundations — Product and Architecture Contract

**Status:** Draft — proposed foundational contract for landing\
**Document role:** Product direction, authority boundaries, foundational invariants, and V1 scope\
**Applies to:** Collider V1 product, UX, architecture, protocol, and implementation planning\
**Terminology dependency:** [`00-collider-v1-glossary.md`](./00-collider-v1-glossary.md)\
**Open architecture register:** [`02-collider-v1-open-architecture-questions.md`](./02-collider-v1-open-architecture-questions.md)\
**Implementation authorization:** None; this document establishes direction and constraints but does not authorize implementation\
**Provisional dependency:** The execution-instance and agent-topology vocabulary in glossary Sections 6.7–6.14 remains exploratory and non-canonical

## 1. Purpose and Authority

This document defines the foundational product and architecture contract for Collider V1.

It establishes:

- what Collider is;
- how Collider relates to Substrate and Handbook;
- the top-level Workspace interaction model;
- the role of the Control Plane, Workstreams, Blocks, Surfaces, and Tabs;
- the separation between durable owner-system state and temporary UI presentation;
- the context and continuity model;
- the minimum coherent V1 scope;
- the principles that later UX and execution contracts MUST preserve.

This document does not define detailed visual layout, application schemas, command routing, Lane dispatch, persistence formats, execution topology, a canonical Workspace event-history architecture, detailed extension-security mechanisms, trust-class schemas, isolation technologies, enforcement protocols, or package-provenance implementation. Those concerns belong to later bounded documents or the open architecture register.

Capitalized terms use the meanings defined by the glossary. Where this document and the glossary differ, the glossary controls terminology and this document controls the broader Collider product direction. Handbook and Substrate remain authoritative for the state and semantics they own.

### 1.1 Provisional execution vocabulary

The glossary currently marks **Run**, **Root Run**, **Child Run**, **Run Node**, **Run Tree**, **Agent**, **Subagent**, and **Subtask** as provisional.

This document intentionally does not freeze that execution ontology. It establishes only terminology-independent requirements such as:

- execution may continue without a visible UI Surface;
- Collider presentation MUST NOT own execution lifetime;
- background execution requires persistent user-visible awareness;
- stopping execution MUST be explicit rather than inferred from closing presentation;
- context and execution provenance SHOULD be retained for future inspection.

Later execution-design work may rename, merge, split, or replace the provisional concepts while preserving these foundational requirements.

## 2. Product Thesis

Collider is the human-facing Workspace application over Substrate and its integrated Handbook capabilities.

Collider V1 combines:

- terminal and process interaction;
- natural-language and agent interaction;
- Handbook semantic artifacts and projections;
- execution and Lane awareness;
- rich first-party applications;
- durable interaction history;
- focused working Surfaces with inline, Sticky, and Tab placements.

Collider SHOULD NOT be designed as:

- a conventional IDE with chat added to it;
- a chatbot with IDE panels attached to it;
- a terminal emulator whose architecture assumes every application is a PTY;
- a chronological collection of chat sessions as its imposed default organization;
- an independent semantic or execution authority.

The intended model is:

> **A Workspace-scoped, block-oriented control environment in which commands, natural-language requests, and application actions can produce interactive Surfaces inline, keep selected items visible within a Workstream, and open important Surfaces as Tabs without owning the durable state they display.**

The center of Collider is therefore not exclusively a terminal, editor, chat transcript, or dashboard. It is a Workstream-oriented interaction environment capable of hosting all of those forms.

## 3. System Authority Boundaries

Collider depends on a strict separation among semantic authority, execution authority, and presentation authority.

| System        | Primary authority                                                                                                                                                                                                                                 |
| ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Handbook**  | Canonical semantic artifacts, artifact kinds and schemas, Scope, Resolution, Projection, contracts, admitted Evidence, verdicts, gates, and owner-approved semantic mutation paths                                                                |
| **Substrate** | Workspace identity and policy, Lanes and execution environments, agents and processes, tools, accepted Invocations, execution lifecycle, runtime telemetry and Receipts, and runtime context assembly and delivery                                |
| **Collider**  | Human interaction, Presentation Actions, Workspace navigation, Control Plane presentation, Workstream and Activity Projection behavior, Activity Items, Blocks, Surfaces, Surface Placements, Sticky Placements, Tabs, and optional inspection UX |

### 3.1 No competing authority

Collider MUST NOT become a second source of truth for Handbook or Substrate state.

Specifically:

- rendering a Handbook Artifact does not make Collider its semantic owner;
- displaying execution activity does not make a React component, Tab, or Surface its lifecycle owner;
- caching state does not grant authority to mutate that state independently;
- a backing file is not automatically the canonical user-facing meaning of a Handbook Artifact;
- a Workstream transcript is not the canonical source of Workspace semantics or execution truth.

Collider MAY initiate actions and mutations, but owner-system state changes MUST pass through the appropriate Handbook or Substrate boundary.

### 3.2 Process topology does not redefine authority

Handbook may be integrated into Substrate through a process bridge, a direct Rust boundary, or another implementation topology. That physical arrangement MUST NOT blur the logical authority boundary.

Substrate consumes Handbook semantics. Collider consumes and presents both systems. Neither integration strategy authorizes duplicate interpretation of Handbook meaning inside Substrate or Collider.

### 3.3 Reference architectures and extension neutrality

Collider MAY learn from and selectively emulate external reference architectures. External influence does not change Handbook, Substrate, or Collider authority, automatically import source terminology, schemas, APIs, lifecycle models, or package topology, or select a product dependency. Any adopted result must be stated in Collider's own canonical language and reconciled with existing owner-system boundaries.

Collider's protocol-neutral Surface and extension direction permits multiple implementation strategies. It does not commit Collider to MCP, Cordis, or any single extension or composition framework. The fact that higher-level capabilities may be extensible also does not require the core responsibilities of Handbook, Substrate, or Collider to become dynamically loaded plugins.

### 3.4 Extension trust, isolation, and bounded cleanup

Dynamic in-process composition is not a security boundary. A context object, registry, API façade, or Capability Grant may mediate and attenuate host access, but none of those mechanisms alone isolates executable code from ambient APIs, host objects, shared mutable state, faults, or unmediated capabilities.

Untrusted executable extensions MUST remain behind an actual Isolation Boundary. Capability mediation MAY supplement that boundary by constraining, revoking, and auditing what isolated code can request; it MUST NOT replace process, runtime, operating-system, or equivalently justified isolation.

Surface-provider identity, permission to render UI, and runtime execution authority are distinct. A provider allowed to create or contribute a Surface does not thereby receive filesystem, process, network, credential, clipboard, browser, host-object, Handbook mutation, or other host capability.

Automatic tracking, cleanup, or reversion claims MUST be limited to Mediated Effects and to the portion of those effects that the host can observe, attribute, and control. Unload, update, deactivation, or graceful disposal MUST NOT claim complete cleanup when untracked or External Effects may remain. External Effects require an explicit contract such as transactionality, copy-on-write, idempotent cleanup, a Compensating Action, or classification as non-revertible; compensation MUST NOT be represented as exact rollback.

Failure containment MUST NOT depend solely on extension-authored graceful cleanup. Crash and non-graceful termination paths require owner-system containment and recovery appropriate to the actual trust and isolation boundary. Every security, unloadability, cleanup, and reversibility claim MUST remain bounded by the system interfaces through which the relevant capabilities, dependencies, and effects are actually mediated.

### 3.5 Dynamic-extension composability objective

Collider adopts **Spatiotemporal Composability** as a design objective for dynamically loadable or reconfigurable Extensions. This direction does not claim that the current architecture, or any current extension class, already provides a formal or production-grade composability guarantee.

For Temporal Composability, Extension Contribution registration and cleanup SHOULD remain local, attributable, and associated with an Extension Lifecycle within the actual mediated boundary. The host SHOULD track mediated contributions and their valid cleanup, inverse, or recovery behavior under a declared recovery criterion. Partial activation and failed update paths SHOULD recover already-installed mediated contributions rather than knowingly leave half-installed host state, and deactivation or replacement SHOULD preserve unaffected functionality and state where feasible. Exact reversion claims require Revertible Contributions evaluated against their declared recovery criteria; External Effects, compensation, crashes, and non-graceful termination remain subject to Section 3.4.

For Spatial Composability, Extensions SHOULD declare the capabilities or services they require and provide. Activation SHOULD occur only while declared requirements are satisfied. Requirement disappearance or material provider change SHOULD cause orderly inactivity or deactivation, and restored requirements MAY permit reactivation. Where cleanup depends on a provider, dependent teardown SHOULD occur before provider withdrawal. Dependency topology SHOULD be inspectable, and configuration change SHOULD reconcile affected Extensions rather than require a blind restart of unrelated host functionality where feasible.

This objective remains bounded by actual mediation, trust, Isolation Boundaries, confinement, valid inverse or recovery behavior, dependency discipline, External Effects, and any required independence, commutativity, conflict, or ordering conditions. These documents do not select a concrete dependency-resolution interface, hot-replacement implementation, or proof method.

### 3.6 Mediated extension capability composition

Extension access to host functionality SHOULD use owner-mediated Capabilities where practical. When an owner-approved capability seam exists, Extensions SHOULD NOT depend on undocumented ambient globals, direct access to another owner's mutable state, or untracked host objects. Capability use and Extension Contributions SHOULD remain attributable to both the provider and the consuming Extension instance.

Extensions and providers SHOULD declare Required Capabilities and Provided Capabilities where feasible. Those Dependency Declarations SHOULD inform extension eligibility and reactive Extension Lifecycle behavior, and the resulting dependency topology SHOULD remain inspectable. A provider remains authoritative for the state, service, and lifecycle it exposes; neither a consumer nor an Extension Context becomes a copied source of truth.

Each Extension instance MAY receive a bounded Extension Context through which owner-approved Capabilities are resolved. Extension Context Mediation and Capability Interception MAY specialize access for a Workspace, Lane, Workstream, Qualified Session, policy, audit, instrumentation, or trust condition while preserving Handbook, Substrate, and Collider authority boundaries. An Extension Context MUST remain distinct from Handbook Scope, Resolution, and Projection; Substrate Context Packs and model-visible context; Activity Projections; Workstreams; and execution environments. It is not an Isolation Boundary, and capability mediation does not replace the trust and isolation requirements in Section 3.4.

Local and cross-process Capabilities MAY use different implementation mechanisms. Cross-process contracts MUST expose asynchronous execution, availability, failure, cancellation, timeout, retry, and provider-loss semantics honestly rather than pretending remote access has infallible synchronous local behavior. This document does not select capability keys, schemas, provider-selection rules, bridge protocols, or a concrete Extension Context implementation.

### 3.7 Host-declared UI extension composition

Collider SHOULD use one general, scoped UI contribution model where practical rather than create unrelated ad hoc registries for every extensible feature. Collider-owned layout declares valid Extension Points and retains authority over their location, accepted contribution contract, composition policy, accessibility requirements, and fallback behavior.

Host ownership of these rules does not require a fixed Navigator Layout. Users control supported grouping, group names, item order, visibility, and ungrouped placement of eligible Navigator Items. Contribution Providers MUST respect those preferences rather than reorganize the user's navigator or restore hidden items merely on reload. Eligibility, permissions, accessibility, and required recovery remain host-controlled; user arrangement does not change an Extension Point's capability or authority boundary.

Authorized Contribution Providers MAY register identity-bearing UI Contributions such as Surfaces, controls, renderers, actions, or related UI behavior through those Extension Points. Contributions SHOULD remain attributable, UI Contribution Scope-bounded, ordered or selected under an explicit Extension Point Policy, and tied to the applicable Extension or provider lifecycle. Zero, one, or several contributions may be valid depending on the host declaration.

Nested Extension Points MAY support compositional UI, but the declaring owner and lifetime of every nested point MUST remain explicit. Replacement, failure, or removal of a parent Surface must account for descendant point declarations and contributions rather than leave unowned UI state. The exact descendant-lifecycle and replacement mechanics remain open.

Contributed UI SHOULD receive only narrow owner-approved props, handles, or references and SHOULD retrieve authoritative data through owner-approved Capabilities or owner-system interfaces. State remains with the nearest authoritative owner. Surface identity, Contribution Provider identity, rendering permission, and runtime execution authority remain distinct.

The Surface model remains the protocol-neutral typed application/view abstraction. MCP Apps are one possible Contribution Provider or Surface Provider path, not the complete extension architecture. Extension Points do not redefine inline, Sticky, Tab, overlay, or Inspector presentation behavior, and the absence or failure of a contribution MUST NOT prevent host recovery, required accessibility behavior, or an owner-defined fallback where one is necessary.

## 4. Workspace-First Product Model

The canonical top-level entity opened by Collider V1 is a **Workspace**.

A Workspace is the Substrate configuration and policy boundary. It is not synonymous with a Repository, open window, terminal session, Workstream, Lane, or chat.

Collider V1 MUST use **Workspace** rather than **Project** for this entity. `Project` remains reserved for a future, explicitly distinct concept or for Handbook-owned semantics.

Opening a Workspace SHOULD make the following categories discoverable:

- Handbook semantic state;
- Work Items and related status through owner-defined Surfaces, even when a dedicated `Work` Navigator section is not present;
- Lanes;
- Repositories or other physical resources;
- active and historical execution activity;
- the canonical Control Plane;
- reconstructable Workstreams and Surfaces;
- Workspace configuration and policy, Agent Profiles, and Agent Harness configuration;
- environment requirements and readiness through Substrate's existing World dependency tooling;
- Workspace-enabled skills, tools, apps, Extensions, providers, integrations, and credential-binding status;
- a dedicated Qualified Session destination for explicit continuity and user-chosen Session navigation;
- Inbox items and diagnostics backed by their respective owners.

These are conceptual access requirements, not a requirement for a fixed top-level row or a fully populated editor for every destination in the first coherent slice; Section 15 preserves staged delivery.

The Workspace remains durable when Collider closes. Collider presentation is a view onto the Workspace, not the Workspace itself.

## 5. Durable Meaning, Federated Sources, and Reconstructed Presentation

Collider MUST distinguish durable owner-system state from the UI currently presenting it.

Durable or durably addressable source state may include:

- Handbook Artifacts, snapshots, Contracts, Work Items, decisions, and semantic relationships;
- Workspace configuration and policy;
- Lanes, worktrees, repositories, and execution environments;
- Substrate Invocations, Receipts, execution records, file changes, results, and errors;
- MCP App objects, revisions, or durable provider references;
- shell or PTY command records, output references, and structured summaries;
- Collider-owned Presentation Action records or presentation-state references when restoration, audit, provenance, or durable history requires them;
- Context Records, Evidence, verdicts, gates, and provenance relationships;
- Workstream identity, projection, indexing, or reconstruction metadata where the selected architecture requires it.

Presentation state may include:

- open Tabs;
- inline and Sticky Surface Placements;
- the Sticky Stack attached to a Workstream Surface;
- selected objects;
- collapsed or expanded Blocks;
- scroll position;
- local layout and display preferences;
- Navigator Layout group names, membership, ordering, visibility, and ungrouped item preferences;
- Collider-local Inbox read or dismissal markers, where those markers are not owner-system acknowledgments.

The foundational rule is:

> **Persist or durably reference meaning and execution history; reconstruct the useful view.**

Collider MUST NOT assume that all durable activity is copied into one Collider-owned or Substrate-owned Workspace event store. Handbook, Substrate, MCP Apps, shell or PTY facilities, and Collider may each retain authoritative records appropriate to their domains.

Collider MAY create projection metadata, indexes, stable references, summaries, or caches needed to provide a coherent Workstream experience, but those mechanisms MUST NOT silently become competing semantic or execution authority.

The exact history architecture remains unresolved. Candidate shapes include:

- one canonical Workspace history;
- federated source histories plus a Workspace activity index;
- query-time federation;
- a hybrid of references, selected snapshots, and materialized projection items.

This document freezes the UX and ownership requirement, not the storage choice:

> **A durable or reconstructable Block must be traceable to sufficient Source Records, durable references, or owner-system objects to explain or reconstruct it, but the relationship need not be one Block to one event or one shared store.**

Closing a Tab or removing a Sticky Placement MUST NOT implicitly delete, retire, archive, stop, or mutate the underlying owner-system state.

Collider SHOULD be capable of reconstructing a current useful Surface from stable identities, owner-system records or references, Activity Projection metadata, and durable presentation references rather than requiring an old in-memory component tree to remain alive.

### 5.1 Projection assembly and stable activity identity

Host-recognized Projection Definitions SHOULD transform eligible Source Records or references into stable Activity Items for a target Activity Projection. A Projection Definition may be built in or dynamically registered; the registration mechanism remains open. Workstream renderers SHOULD consume final Activity Items or bounded Surface models rather than repeatedly scan raw heterogeneous source collections.

Each Activity Item MUST have stable Business Identity independent of visual list position, page boundaries, arrival order, and renderer implementation. That identity may be supplied by an authoritative owner system or deterministically derived by the applicable Projection Definition from stable source fields. Intentionally ephemeral visual output may render as a Block without becoming an Activity Item. One Activity Item MAY correlate or fold several Source Records or revisions while retaining inspectable source references and projection-definition provenance.

An Activity Projection MUST provide deterministic presentation order sufficient for rendering and navigation for a given projection state and Projection Definition version. That presentation order does not imply that the underlying Source Records possess one authoritative total order; source causality may remain partial, concurrent, late-arriving, or owner-specific.

An Activity Assembler MAY own projection-specific fold state and Projection Indexes needed for efficient correlation, predecessor or ordering lookup, and incremental maintenance. That machinery MUST NOT become semantic, execution, or Source Record authority.

Projection assembly SHOULD be deterministic and replayable to the extent permitted by the participating source systems. Incremental append and older-history loading SHOULD update only affected projected objects and preserve unaffected Business Identity and fold state where possible. Pagination and replay are ordinary explicit operations; gap repair, projection invalidation, and full resynchronization SHOULD be explicit exceptional paths rather than hidden renderer behavior.

Different Workstream views MAY assemble or locate the same Source Records differently. Control Plane and Lane projections may share Source Record identity while producing view-specific Activity Items, ordering, visibility, or location. Whether they also share Activity Item identity remains open.

This projection architecture strengthens both federated and indexed history options but does not select one canonical Workspace event store, assembler process, database, language, or persistence topology.

## 6. Primary Interaction Model

### 6.1 Projection-oriented Workstreams

The primary center interaction model is a Workstream Surface rendering an ordered Activity Projection as Blocks.

A Block MAY represent:

- user input;
- shell output;
- an agent response;
- a Handbook Projection;
- a repository browser;
- an editor;
- a plan;
- a diff;
- test or gate results;
- execution status;
- a rich interactive Surface, including one supplied by a Collider-native, Substrate, Handbook, Extension, or MCP App Surface Provider.

When a Block hosts a contributed or externally provided Surface, the Block remains the inline presentation placement, the Surface remains the typed application or view, and the Extension, MCP App, or other Surface Provider remains the attributable provider. Rendering that Surface does not transfer provider-owned state, semantic authority, execution authority, or unrelated host capabilities to the Block or Collider.

The Workstream MAY look conversational, but it is more structured than a conventional chat transcript. Its visible sequence may combine stable Activity Items backed by several authoritative source systems, and one Activity Item may fold several correlated Source Records or revisions.

The Workstream renderer SHOULD receive final Activity Items or bounded Surface models from the Activity Projection pipeline. It SHOULD NOT rediscover business identity, correlation, or fold state by repeatedly scanning raw heterogeneous histories.

A Workstream MUST NOT be assumed to own one independent event history merely because it presents an ordered view. The underlying history, assembler placement, indexing, and persistence architecture remains open.

### 6.2 Shared Control Plane and Lane infrastructure

The Control Plane and Lane views SHOULD use the same core:

- Workstream Surface;
- Activity Projection pipeline;
- Block renderer;
- Surface host;
- Composer;
- selection and branching primitives;
- Sticky Stack behavior;
- Tab Promotion behavior.

Their primary difference is projection and binding posture:

```text
Control Plane
  Lane filter: none
  Default Lane: none unless explicitly established

Lane view
  Lane filter: selected Lane
  Default Lane: selected Lane
```

This filtering may remain an internal implementation detail. The user does not need to manipulate raw filter expressions.

`Lane filter: none` means lane-unfiltered, not “display all raw internal telemetry.” The projection may still collapse, summarize, or progressively disclose low-level tool, provider, or internal execution detail.

Control Plane and Lane projections MAY reuse Source Records and Projection Definitions while producing view-specific Activity Items, ordering, visibility, or projected locations. Exact Activity Item identity sharing across those views remains an open decision.

### 6.3 Unified Composer

The active Workstream SHOULD provide a unified Composer capable of accepting:

- raw shell input;
- Collider or Substrate commands;
- Handbook-oriented requests;
- natural-language agent instructions;
- structured actions from an existing Surface.

A user SHOULD NOT need to switch between separate terminal and chat applications merely because one Invocation is shell-shaped and the next is natural language.

The routing and execution semantics of different input dialects remain distinct even when they share one Composer.

### 6.4 Terminal capability without terminal confinement

Collider MAY provide Terminal Surfaces and raw PTY behavior, but the Control Plane and Workstream MUST NOT be architecturally reduced to a single PTY or shell process.

Normal shell semantics MUST be preserved when the user is operating in raw shell or PTY mode. Rich presentation MAY enhance a command result, but MUST NOT silently falsify the command, output, exit status, Receipt, or execution record.

The desired principle is:

> **Commands may produce applications; applications are not all terminals.**

### 6.5 Interactive Surfaces inline

Commands, agent responses, owner-system operations, and authorized Surface
Providers MAY cause rich typed Surfaces to appear directly inside the active
Workstream.

Such a Surface MAY be Collider-native, Substrate-provided, Handbook-provided,
or supplied by an authorized Extension or through an MCP App adapter. Its
provider path does not change the applicable Surface, placement, owner-authority,
trust, or capability boundaries.

A user should be able to:

1. request or discover an object;
2. receive a useful interactive Surface inline;
3. interact with it without leaving the Workstream;
4. keep it visible in the Workstream's Sticky Stack where supported;
5. expand or focus it where supported;
6. use **Open as tab** when it deserves durable visual attention.

An Extension- or MCP App-supplied Surface participates in the same presentation
model as other Surfaces, but only for the placement modes it declares and the
host authorizes.

Opening a richer representation MUST NOT require moving the application into the optional right sidebar.

## 7. Control Plane and Lane Workstream Foundation

Each Workspace SHOULD expose one canonical **Control Plane** as its default opening Workstream Surface.

The Control Plane is the broad Workspace-orchestration entry point. It is not itself a Lane and has no implicit execution environment merely because it is visible.

### 7.1 V1 presentation

The first V1 Control Plane SHOULD be intentionally minimal:

- a block-oriented Workstream Surface;
- a terminal-like Composer;
- the ability to submit shell-shaped and natural-language Invocations;
- the ability to render rich inline Surfaces;
- access to Workspace-level navigation and execution awareness.

A dashboard is not required for the initial version.

Later versions MAY add persistent Workspace-status, active-work, gate, or orchestration applications to the Control Plane without replacing its Workstream model.

### 7.2 Lane-unfiltered Control Plane projection

The Control Plane SHOULD present a Workspace-relevant Activity Projection with no Lane filter. This allows one coherent view to include eligible Workspace-level activity and eligible activity associated with individual Lanes.

This is a presentation contract, not a decision that all source systems must write into one canonical history.

The Control Plane projection MUST remain eligible for summarization, collapsing, progressive disclosure, permission filtering, and exclusion of low-level internal machinery from the default view.

### 7.3 Lane-filtered Workstream views

A Lane Workstream Surface SHOULD reuse the same core components as the Control Plane while applying an internal filter for the selected Lane and using that Lane as the normal default binding for new Invocations.

A Lane view is a projection onto the Lane, not the Lane itself. It may be closed while the Lane and its execution continue in the background.

The exact inclusion rules remain unresolved for:

- a Workspace-level Invocation that later dispatches into a Lane;
- cross-Lane orchestration;
- Activity Items associated with several Lanes or scopes;
- activity that begins in one projection and completes in another;
- historical Lane rebinding.

These rules belong to the open architecture register and later execution-and-binding contract.

### 7.4 Resolution posture

The Control Plane begins with a broad Workspace-orchestration Resolution Posture. That posture is a default orientation, not a static context package.

Each Invocation MUST be eligible for new Handbook Resolution appropriate to its actual purpose and target. A narrow request issued from the Control Plane may resolve narrowly, and an implementation request may need to dispatch into a specific Lane.

The Control Plane MUST NOT achieve apparent Workspace awareness by treating one ever-growing transcript as the sole source of context.

### 7.5 Lane neutrality

The Control Plane SHOULD remain Lane-neutral until an Invocation is explicitly or deterministically bound to a Lane.

Read-only Workspace inspection may be possible without a task-specific Lane. Environment-touching, mutating, or task-specific work MUST NOT silently execute in an arbitrary worktree or repository context.

The detailed binding and dispatch rules belong to the later execution-and-binding contract.

## 8. Workspace Navigator Foundation

Collider V1 MUST provide a persistent left-side **Workspace Navigator**. It exposes durable Workspace destinations through Navigator Items, presentation-only Navigator Groups, and a user-customizable Navigator Layout.

The default organization foregrounds Workspace structure rather than open windows or chronological chat titles. That default is not an imposed hierarchy: users may arrange eligible destinations differently, including prominent access to Qualified Sessions, without changing underlying identity or authority.

### 8.1 Conceptual destinations

The long-term navigator should provide the following destinations. This is a coverage map, not a fixed root-level hierarchy, final label set, or requirement to ship every populated Surface in the first slice.

| Destination | Purpose and boundary |
|---|---|
| **Control Plane** | Return to the canonical Lane-unfiltered Workstream home. A default root shortcut may use **Home** without renaming Control Plane. |
| **Inbox** | Notifications and actionable owner-system items, including questions, approvals, blocked work, and failures where available. It is not a new obligation or task authority. |
| **Handbook** | Semantic Artifacts and owner-defined projections; curated entries may include Charter, Architecture, Design, Contracts, Decisions, or Evidence & Gates without freezing Handbook's registry. |
| **Work** | Owner-defined Work Items and related status, not an independent Collider task database. |
| **Lanes** | Durable execution partitions and lightweight activity state, opening Lane-filtered Workstream Surfaces independently of Tab lifetime. |
| **Execution activity** | Active, queued, interrupted, attention-requiring, and historical Substrate activity as supported by its owner model. **Execution** may be a group heading and **Activity** a destination label; this does not settle execution topology. |
| **Sessions** | A dedicated destination for discovering and explicitly continuing actual Qualified Sessions. Users may place it in a group or at the root; opening the destination does not itself resume a Session. |
| **Harnesses** | Configured Agent Harnesses and their native, Substrate-integration, and Collider-presentation settings, with compatibility and health information where available. Root versus nested placement is not fixed. |
| **Environments** | Environment Dependency configuration feeding Substrate's World dependency tooling, plus effective requirements, realized resources, readiness, and drift where supported. |
| **Repositories** | Physical source, filesystem, and version-control resources; a Workspace may reference zero, one, or several Repositories. |
| **Agent Profiles** | Reusable Substrate-owned role and execution configuration, distinct from harnesses, model providers, and Qualified Sessions. |
| **Configuration & Policy** | Workspace configuration, effective values and their sources, overrides, operational policy, and validation through Substrate-owned interfaces. |
| **Capabilities & Integrations** | Workspace availability and enablement of skills, tools, apps, Extensions, providers, and connections. Their distinct types and owner systems remain visible. |
| **Credential bindings** | Authorized views of credential references, scope, availability, and connection status, not general-purpose disclosure of secret values. |
| **Diagnostics** | Owner-reported Workspace, Handbook, Substrate, environment, harness, provider, and projection health, with authorized recovery paths. |

Related destinations may be nested together or exposed ungrouped. A destination being discoverable does not grant permission to use its operations, and an unavailable capability MUST NOT be presented as ready merely because its entry is visible.

### 8.2 Configuration, environment, and enablement boundaries

Workspace configuration and policy Surfaces SHOULD show declared and effective values, their sources and overrides, validation, and the effect of proposed changes where the owner exposes that information. Substrate operational policy remains distinct from Handbook Charter, posture, Contracts, or other semantic governance. Collider application preferences remain presentation settings rather than Substrate Workspace policy.

Environment configuration MUST feed Substrate's existing World dependency tooling through owner-approved interfaces rather than a new Collider dependency engine. The UI SHOULD distinguish requirements from resolved/effective configuration and from actual realization or observed readiness where the source model supports those distinctions; these are explanatory views, not newly mandated lifecycle states. Environment Dependencies, Repository/package dependencies, and Extension capability dependencies MUST remain distinct.

A Harness Surface MAY bring native harness settings, Substrate adaptation and execution settings, and Collider presentation settings together, but MUST identify their owners and use the appropriate mutation boundary. Agent Profiles remain separate reusable configuration. Configuration precedence, profile/harness relationships, supported controls, and whether a change applies immediately, on the next Invocation, or after restart require owner-grounded design; no new schema or runtime capability is implied here.

Capability and integration Surfaces SHOULD distinguish availability or installation, Workspace enablement, authorization, and operational readiness where applicable. Enabling a skill, tool, app, Extension, or connection is not a navigator visibility change and MUST NOT implicitly issue a Capability Grant. Credential views SHOULD show authorized bindings and status rather than raw secrets. Diagnostics and Inbox views MUST preserve the authority of the systems whose state they report.

### 8.3 Default naming at the highest level only

**Collider-shipped labels at the highest level of the default navigator MUST be one word.** This covers root-level group headings and default ungrouped root-level destinations; it does not constrain every level of the tree.

Possible root headings include **Planning**, **Execution**, **Resources**, and **Setup**. These illustrate the naming rule, not a frozen hierarchy. Nested entries may use names such as **Agent Profiles**, **Configuration & Policy**, **Capabilities & Integrations**, or **Models & Providers**. **Inbox** is the preferred default destination label; users may choose **Notifications** or another name.

Canonical terminology is unchanged: **Control Plane**, **Qualified Session**, and **Agent Profile** remain those architectural concepts. A root display alias such as **Home** MAY refer to Control Plane. User-created group names and user-promoted items are not bound by the default naming rule, and promoting a multiword entry to the root MUST NOT automatically rename it.

### 8.4 User customization and host composition

Collider V1 MUST support basic Navigator Layout customization. Users MUST be able to:

- create and rename Navigator Groups;
- add or remove displayed eligible Navigator Items within groups;
- move and reorder groups and items, including displaying an item ungrouped at the root;
- recover hidden eligible destinations and restore a default arrangement.

These are Presentation Actions. Labels, membership, order, and visibility MUST NOT change underlying object identity, Handbook Scope, Lane binding, policy, permissions, capability enablement, or execution lifetime. **Remove from navigator** means hide or remove a presentation reference, not delete an object, retire a Lane, uninstall an app, or disable a harness. Removing a group MUST NOT remove the underlying destinations.

Collider retains authority over eligible built-in and contributed items, valid Extension Points, permissions, accessibility, and recovery. Providers may contribute through approved interfaces but MUST NOT overwrite the user's arrangement or reinsert hidden entries merely on reload. A Navigator Group is not itself a semantic scope or a new permission boundary.

Collider SHOULD retain user layout preferences. Such changes MUST NOT silently rewrite Substrate operational configuration or policy, or another user's arrangement. Preference storage, application/user/Workspace scoping, synchronization, explicit shared defaults, aliases, and provider-unavailability behavior remain open under C-OQ-025. Basic navigator customization is distinct from the deferred general-purpose pane/layout builder.

### 8.5 Discoverability and staged delivery

The following rules remain foundational:

1. a Lane remains discoverable when no Tab displays it, even when its navigator shortcut is hidden;
2. Workstreams and execution activity need not be represented as conventional chat titles, while users may prioritize the dedicated Sessions destination;
3. Handbook navigation exposes semantic objects and registered projections rather than requiring backing-file navigation;
4. Work remains a projection over owner-defined work state rather than a separate Collider task authority;
5. Repositories are resources of a Workspace, not replacements for it;
6. selecting a Navigator Item SHOULD open the appropriate center Surface without requiring another application mode;
7. customization MUST preserve a discoverable path to hidden eligible destinations, required execution awareness, cancellation, and recovery.

A populated dedicated **Work** section remains secondary to the first coherent slice until its Work Item kinds and projections are sufficiently defined. Work Items SHOULD remain discoverable through Handbook, the Control Plane, Lane or Artifact Surfaces, search, or other owner-defined paths; Collider SHOULD NOT render empty placeholder sections merely to satisfy this coverage map.

The expanded configuration, harness, environment, enablement, Inbox, and Sessions Surfaces are accepted product destinations with staged delivery under Section 15. The exact default hierarchy, interaction design, and first implementation of each belong to the Workspace Shell contract; operational behavior belongs to the appropriate owner and Execution and Binding work. Neither those details nor a concrete persistence mechanism are selected here.

## 9. Surfaces, Blocks, Sticky Placements, and Tabs

### 9.1 Protocol-neutral Surface model

Collider's core application model MUST be based on typed Surfaces rather than on one transport protocol.

A Surface may be:

- Collider-native;
- Substrate-provided;
- Handbook-provided;
- adapted from an MCP App or another extension mechanism.

MCP Apps are a possible provider mechanism. They are not the definition of all Collider applications.

### 9.2 Three presentation placements

Collider's Surface model SHOULD distinguish three principal placements:

```text
inline placement
    the Surface appears in chronological Workstream flow

Sticky Placement
    an additional Workstream-local placement remains visible while flow scrolls

Tab placement
    the Surface is opened in the center Tab bar for focused access
```

These placements may refer to the same Surface or to a provider-declared related expanded Surface. None of them owns the represented Handbook, Substrate, MCP App, repository, or shell state.

### 9.3 Sticky Stack

A Workstream Surface SHOULD support a small bounded Sticky Stack so that important items such as a plan, Contract, design reference, test status, or execution status can remain visible while new Blocks continue below.

Creating a Sticky Placement MUST:

- preserve the original chronological Block;
- create only additional presentation state;
- remain local to the relevant Workstream Surface or projection placement;
- avoid making the item sticky automatically in every other projection where it appears.

Collider SHOULD compact the Sticky Stack when it would consume excessive space. Two or three fully visible items is a plausible direction, but the exact capacity, persistence, compaction, ordering, and controls remain unresolved.

Sticky behavior MAY be deferred from the first vertical implementation slice, but the Surface and placement model SHOULD avoid making it impossible.

### 9.4 Open as tab

The preferred user-facing action for creating a Tab placement is **Open as tab**. Architecture documents may call this **Tab Promotion**.

The action may be initiated through:

- drag into the Tab bar;
- an `Open as tab` action;
- a context menu;
- a keyboard-accessible command.

Drag MUST NOT be the only available mechanism.

The originating Block SHOULD remain as a compact chronological anchor. The exact rule for whether the Tab shares the same Surface identity or opens a provider-declared canonical expanded Surface remains unresolved.

### 9.5 Tabs are apertures, not owners

A Tab is a center-workspace host for a focused Surface Placement.

Tabs SHOULD represent meaningful working Surfaces such as:

- the Control Plane;
- a Lane Workstream Surface;
- a Handbook Artifact or Contract browser;
- a design Surface;
- execution or test results.

Collider SHOULD avoid recreating file-tab overload as its primary navigation model. Files may open inside editor or repository Surfaces without automatically becoming top-level Workspace Tabs.

Closing a Tab removes a placement only. The underlying Lane, Workstream, execution, Artifact, Work Item, provider object, or repository state continues according to its owner.

## 10. Background Execution and Ambient Awareness

Collider is designed for execution that may continue while its associated Surface or Tab is not visible.

This requires two separate guarantees.

### 10.1 Presentation-independent lifetime

No Collider component may implicitly own the lifetime of Substrate execution merely because it displays that execution.

Unmounting a component, closing a Tab, switching Workstreams, or closing a lane-related Surface MUST NOT implicitly stop background work.

Stop or cancel behavior MUST be an explicit action routed through Substrate.

### 10.2 Persistent awareness

Because background work can outlive visible Surfaces, Collider V1 MUST provide minimum persistent awareness of ongoing execution and execution that requires user attention. A concrete implementation MAY distinguish states such as active, queued, blocked, failed, or completed; the detailed status vocabulary is not frozen here.

The first version does not require a full execution debugger. It does require:

- a lightweight Workspace-wide activity indicator;
- discoverable Lane-level activity state where applicable;
- a path to open a richer execution-status Surface;
- a clear explicit stop/cancel action for user-controllable work;
- the ability to reconnect a Surface to ongoing or completed execution state.

Internal execution detail SHOULD be progressively disclosed beneath user-meaningful activity rather than flooding the top-level UI. The exact execution hierarchy and vocabulary remain provisional.

Navigator customization MUST NOT remove all awareness of active or attention-requiring execution or all access to cancellation and recovery. No particular navigator row must be permanently locked merely to satisfy this rule; a persistent indicator and another discoverable route may provide access.

Inbox read or dismissal state MUST NOT be treated as approval, Gate satisfaction, unblocking, cancellation, or resolution of an owner-system obligation. An action that changes authoritative acknowledgment or work state must use the relevant owner-system boundary rather than masquerade as a local Presentation Action.

## 11. Context and Continuity Model

Collider SHOULD be described as **resolution-reconstructed** or **session-independent by default**, not stateless.

The system is highly stateful. Its durable state exists in Handbook, Substrate, source systems, Workstream projection metadata, repositories, execution records, and Collider presentation preferences. What Collider avoids is making one long chat transcript the sole memory and authority for future work.

### 11.1 Per-Invocation reconstruction

Each meaningful Invocation SHOULD be resolved and assembled against its current:

- Workspace;
- Scope and Resolution Posture;
- Work Item or target concern;
- Lane and repository state where applicable;
- current owner-system state;
- purpose and requested action.

A later Invocation may receive a newly reconstructed Context Pack rather than inheriting an unbounded transcript.

### 11.2 Transcript and projection as history, not sole memory

A Workstream preserves or reconstructs useful interaction and activity continuity, but its visible Activity Projection is not automatically injected wholesale into future model context.

Prior interaction may be:

- selected through Handbook Resolution;
- distilled into durable Artifacts, decisions, Work Items, or state;
- referenced through Context Records, Source Records, execution records, or Receipts;
- omitted when irrelevant to the current Invocation.

This enables continuity without requiring conventional chat-session organization or one canonical Workspace event log.

### 11.3 Explicit Qualified Session Continuity

Collider's default behavior SHOULD remain resolution-reconstructed. Each Invocation may receive newly resolved and assembled context appropriate to its current purpose even when it occurs in the same Workstream as a prior Invocation.

The architecture MUST nevertheless preserve room for an explicit request to bind to, resume, reuse, or continue a Qualified Session owned by a model provider, runtime, shell, PTY, or agent harness. Continuation MUST be intentional and visible; remaining in the same Workstream, Lane view, Tab, or adjacent message sequence MUST NOT silently continue a Qualified Session.

A Qualified Session is not a Workstream. One Workstream may project activity associated with zero, one, or several Qualified Sessions over time, and one Qualified Session's activity may appear in several Workstream projections where policy permits. A dedicated **Sessions** destination MUST be available when the Session Continuity capability is delivered. Its Surface may present another Activity Projection without replacing Workspace or Workstream identity.

Collider's default organization MUST NOT be Session-centric, but users MAY choose prominent or primarily Session-oriented navigation in their own Navigator Layout, including an ungrouped Sessions item. That user preference does not change the resolution-reconstructed default. Opening, regrouping, or highlighting the Sessions destination MUST NOT itself imply continuation. Actual Session entries MUST identify their provider or runtime, and continuation remains intentional and visible.

Session History may provide Source Records for Activity Projection and may support owner-scoped continuity, reconstruction, pagination, or resume. It is not the Workspace's canonical history and does not settle whether Workspace activity uses one history, federation, an index, query-time composition, or a hybrid.

Resuming a Qualified Session may preserve only the provider- or runtime-owned state that the Qualified Session explicitly promises. Before a new Invocation uses that continuity, current Handbook Scope or Resolution, Lane binding, Substrate policy, Capability Grants, permissions, and provider/model compatibility MUST be revalidated where applicable; stale values MUST NOT be silently inherited merely because a Qualified Session resumes.

A future capability may support one-Invocation continuation, a persistent session-oriented continuity mode, and an explicit return to resolution-reconstructed behavior. The exact commands, UI state, resume handles, provider guarantees, expiry, recovery, disconnection behavior, branching, lifecycle, and V1 delivery timing remain open.

### 11.4 Inspectable context lineage

The following distinctions SHOULD be retained in the underlying system even if their detailed UI is deferred:

```text
Resolved   what Handbook determined was applicable
Injected   what Substrate actually delivered
Acquired   what execution later observed
Produced   what execution created
```

Stable references among these stages and the relevant Source Records or Activity Items are important for future context inspection, debugging, Evidence workflows, and provenance.

Collider V1 does not require the complete right-side context X-ray, but implementation work SHOULD avoid discarding the records required to build it later.

## 12. Handbook Experience in Collider

Handbook should feel like an integrated semantic system, not a folder of configuration files.

Collider SHOULD be capable of presenting polished, purpose-specific Handbook Surfaces for concepts such as:

- Charter;
- architecture;
- Contracts;
- Work Items;
- decisions;
- future design artifacts;
- other registered Artifact kinds.

### 12.1 Semantic navigation before files

The Workspace Navigator SHOULD expose Handbook objects through owner-defined identity and Projection rather than requiring users to locate and edit their serialized backing files.

Physical files remain inspectable through Repository or filesystem Surfaces when useful, but they are not the primary UX contract for Handbook.

### 12.2 Owner-approved editing

A displayed Projection is not automatically a lossless editable source.

Collider MUST only present an Artifact as directly editable when Handbook exposes a valid mutation path. Depending on Artifact capabilities, editing may use:

- structured field mutation;
- a candidate or patch workflow;
- validation and promotion;
- a source-backed lossless editor;
- a read-only Projection.

Chat-based editing MUST also use Handbook-owned mutation semantics rather than directly rewriting arbitrary projected text or backing files.

### 12.3 Registry-driven growth

Collider SHOULD consume Handbook's registered Artifact kinds, instances, capabilities, and Projections. It MUST NOT hardcode the assumption that the currently named Charter, Contract, architecture, or design categories are the complete future Handbook model.

Curated shortcuts are allowed. Semantic ownership remains Handbook-driven.

## 13. Right-Side Inspector

The optional right-side Inspector is lower priority than:

1. the Workspace Navigator;
2. the center Control Plane and Workstream model;
3. inline rich Surfaces;
4. inline, Sticky, and Tab Surface placements;
5. minimum persistent execution awareness.

V1 essential work MUST remain possible with the Inspector hidden or absent.

When later implemented, the Inspector MAY provide selection-driven views for:

- an Artifact, file, Contract, Work Item, or design object;
- execution activity;
- context lineage;
- relationships, history, tests, Evidence, or provenance.

The Inspector SHOULD NOT become the mandatory destination for expanded applications, and this document does not freeze its idle-state behavior.

## 14. Accessibility and Progressive Disclosure

Collider is a developer tool, but its semantic and conversational interaction model should make it usable by people who are not expert terminal or IDE users.

Collider V1 SHOULD provide:

- natural-language access to the same Workspace capabilities exposed through commands;
- semantic Handbook views rather than requiring direct file manipulation;
- progressive disclosure of advanced runtime and execution detail;
- keyboard-accessible alternatives to drag interactions;
- explicit raw-output and execution details for users who need them;
- clear distinction between view changes and destructive or execution-stopping actions.

Accessibility MUST NOT be achieved by hiding execution truth or silently changing shell behavior. Simplicity should come from structured presentation and good defaults, not from removing inspectability.

## 15. V1 Product Scope

### 15.1 Required first coherent vertical slice

The smallest coherent Collider V1 SHOULD include:

1. opening a Substrate Workspace;
2. a persistent Workspace Navigator with identity-bearing Navigator Items and the highest-level-only default naming rule;
3. Handbook, Lanes, and Repository navigation;
4. a canonical Control Plane Workstream Surface;
5. a unified terminal-like Composer;
6. shell-shaped and natural-language Invocations in the same center environment;
7. typed Blocks and a protocol-neutral Surface host;
8. a Workstream and Activity Projection boundary that produces stable Activity Items through at least one recognized, built-in Projection Definition without requiring one canonical event store or a dynamic projection-registration system;
9. at least one first-party Handbook Surface;
10. at least one first-party Lane or execution-status Surface;
11. inline rendering of eligible Surfaces;
12. opening an eligible Surface as a center Tab;
13. closing and reopening Surface placements without destroying owner-system state;
14. background execution independent of visible Tabs;
15. minimum persistent execution awareness and explicit stop or cancel access;
16. stable Source Record references and Activity Item Business Identities sufficient to incrementally update, reconnect, trace, and reconstruct projected Surfaces;
17. Handbook mutations routed through Handbook-owned capabilities.

### 15.2 Secondary V1 work

Basic Navigator Layout customization under Section 8.4 is a V1 requirement, although it need not block the first coherent vertical slice. Its delivery must support group creation and renaming, membership and ordering changes, ungrouped items, hiding/restoration, and recovery without altering owner-system state.

Additional secondary work includes:

- first-party configuration and policy, Agent Profile, Agent Harness, environment/dependency, and capability/integration Surfaces through supported owner interfaces;
- Inbox, credential-binding status, and broader diagnostics Surfaces;
- Sticky Placements and a bounded Sticky Stack;
- richer Handbook structured editing;
- a rich Repository browser;
- source editing Blocks;
- saved Tab and presentation layouts across application restarts;
- richer Lane status summaries;
- a populated `Work` section in the Workspace Navigator;
- additional first-party Surface types;
- user-facing Workstream branching and reprocessing;
- deeper activity and execution-history navigation;
- explicit Qualified Session Continuity and the dedicated Sessions destination, including user-chosen prominent or ungrouped access.

### 15.3 Deferred beyond the first shell

The following are explicitly deferred:

- a full Control Plane dashboard;
- pipelines and visual workflow construction;
- arbitrary pane docking or general-purpose layout composition beyond basic Navigator Layout customization;
- detailed public third-party extension security and isolation implementation;
- broad external MCP App support;
- production-grade dynamic-extension loading, hot replacement, and composability guarantees;
- a full context X-ray or agent debugger UI;
- a complete internal execution-tree visualization;
- detailed right-side Inspector behavior;
- Storybook or Open Design integration;
- comprehensive design-system management;
- user-authored UI layout commands;
- multi-Workspace Project orchestration.

These deferrals do not include basic Navigator Layout customization or the dedicated Sessions destination. Their staged V1 scope remains in Section 15.2; default organization remains non-Session-centric while user-selected Session navigation is supported.

Deferred features MUST layer onto the foundational model rather than redefine the ownership and lifecycle boundaries established here.

## 16. Foundational Invariants

The following are the primary Collider V1 foundations:

1. **Workspace is the canonical top-level Substrate configuration and policy boundary.**
2. **Workspace is not a synonym for Repository or Project.**
3. **Handbook owns semantic truth; Substrate owns execution truth; Collider owns projection, presentation, and interaction.**
4. **Collider MUST NOT become a competing Handbook, Substrate, MCP App, shell, or provider authority.**
5. **A Lane is independent of every Tab or Surface that displays it.**
6. **A Workstream is an interaction context and ordered Activity Projection, not an execution environment or assumed independent event log.**
7. **The Control Plane is a Workspace-level Workstream Surface, not a Lane or a single PTY.**
8. **The Control Plane SHOULD be Lane-unfiltered, while Lane views SHOULD reuse the same core Workstream components with an internal Lane filter.**
9. **Lane-unfiltered does not mean raw or unrestricted internal telemetry.**
10. **The center Workstream is the primary place where terminal, agent, Handbook, and application interaction converge.**
11. **Commands MAY produce interactive applications rather than only text.**
12. **A durable or reconstructable Block SHOULD be traceable to sufficient Source Records, references, or owner-system objects without becoming their authority.**
13. **The UX may present one coherent activity flow without requiring one canonical Workspace event store.**
14. **A Surface may have inline, Sticky, and Tab placements.**
15. **A Sticky Placement is local presentation state and preserves the chronological Block.**
16. **Open as tab creates focused presentation; it is distinct from making an item Sticky.**
17. **Closing a Tab, removing a Sticky Placement, or unmounting presentation MUST NOT implicitly stop work or delete durable state.**
18. **Background execution requires persistent user-visible awareness.**
19. **Each Invocation may be resolved anew; a Workstream Activity Projection is not the sole context authority.**
20. **The default interaction model is resolution-reconstructed; Qualified Session Continuity is explicit and optional, and remaining in one Workstream MUST NOT imply it.**
21. **Projection is not automatically canonical or editable Artifact source.**
22. **Handbook mutation MUST use Handbook-owned semantics.**
23. **Collider's Surface model is protocol-neutral; MCP is one possible provider mechanism.**
24. **In-process composition and capability mediation are not security isolation; untrusted executable extensions require an actual Isolation Boundary.**
25. **Permission to render or contribute UI does not imply host execution authority or unrelated host capabilities.**
26. **Automatic cleanup and reversion claims are limited to Mediated Effects; a Compensating Action is not exact rollback.**
27. **Crash and non-graceful failure safety MUST NOT depend solely on graceful extension cleanup.**
28. **Raw terminal and shell semantics remain available and truthful.**
29. **The right-side Inspector is additive, hideable, and nonessential to the first coherent V1.**
30. **Execution-instance and agent-topology vocabulary remains provisional until separately resolved.**
31. **Durable meaning and execution history survive independently from the UI currently displaying them.**
32. **Dynamically reconfigurable Extensions SHOULD aim for Spatiotemporal Composability, but the current architecture does not claim that guarantee.**
33. **Temporal Composability claims are limited to mediated and attributable contributions with valid cleanup, recovery, or inverse behavior evaluated under a declared recovery criterion; External Effects and Compensating Actions do not imply exact reversion.**
34. **Spatial Composability is driven by declared requirements and provisions, reactive Extension Lifecycle behavior, and orderly dependent teardown before provider withdrawal where required.**
35. **Extension access to host functionality SHOULD use owner-mediated and attributable Capabilities where practical rather than undocumented ambient globals.**
36. **Required and Provided Capability declarations SHOULD inform Extension eligibility and lifecycle while providers retain authority for the state and services they expose.**
37. **An Extension Context and Extension Context Mediation MUST NOT become a cross-system source of truth, replace Handbook Resolution or Substrate Context Packs, or substitute for an Isolation Boundary.**
38. **Cross-process Capabilities MUST represent asynchronous, availability, failure, cancellation, timeout, retry, and provider-loss semantics honestly where they apply.**
39. **Collider-owned layout declares valid Extension Points; authorized Contribution Providers contribute through scoped host-approved interfaces.**
40. **Extension Points compose Surfaces and controls without replacing the Surface abstraction or inline, Sticky, Tab, overlay, and Inspector presentation semantics.**
41. **UI Contributions remain provider-attributable, lifecycle-bound, and owner-system-bound; permission to contribute UI does not grant runtime or semantic authority.**
42. **Host accessibility, fallback, and recovery behavior MUST remain possible when a UI Contribution is absent, removed, replaced, or fails.**
43. **Workstream renderers SHOULD consume final Activity Items or bounded Surface models rather than repeatedly scan raw heterogeneous source histories.**
44. **Activity Items MUST have stable Business Identity independent of visual list position; that identity may be owner-supplied or deterministically projection-derived, and an Activity Item MAY fold multiple correlated Source Records or revisions while retaining inspectable provenance.**
45. **Projection assembly SHOULD be deterministic and replayable where source semantics permit, and incremental append or older-history loading SHOULD preserve unaffected projected identity and state where possible.**
46. **Projection Definitions, Activity Assemblers, and Projection Indexes own view-specific assembly rules and derived state, not semantic, execution, or Source Record authority.**
47. **Different Workstream projections MAY assemble the same Source Records differently; deterministic presentation order does not establish one authoritative total Source Record order, and this projection architecture does not select one canonical Workspace event store or require shared Activity Item identity across views.**
48. **A Workstream may span zero, one, or several Qualified Sessions; Session History may source Activity Projections without becoming the Workspace's canonical history or Workstream identity. Default navigation is not Session-centric, but users may prioritize Qualified Sessions in their own Navigator Layout.**
49. **Resuming a Qualified Session MUST NOT silently carry forward stale Handbook Scope or Resolution, Lane binding, Substrate policy, Capability Grants, permissions, provider availability, or model identity and configuration without applicable revalidation.**
50. **Purely local Collider presentation changes are Presentation Actions, not Invocations; a Surface action becomes an Invocation only when it requests owner-system behavior.**
51. **The one-word naming rule applies only to Collider-shipped labels at the highest level of the default navigator; descendant labels, user names, user-promoted items, and canonical architectural terminology are not constrained by it.**
52. **Users control supported Navigator Group names, membership, ordering, visibility, and ungrouped placement; stable Navigator Item identity and owner-system authority do not change with that arrangement.**
53. **Collider declares eligible navigation contributions and safe composition rules; providers MUST respect user layout choices, and customization MUST preserve required execution awareness, cancellation, and recovery.**
54. **Removing an item from navigation is not disabling a capability, deleting an object, changing policy, resuming a Session, or stopping execution.**
55. **Environment Dependency configuration feeds Substrate's World dependency tooling; Repository/package dependencies and Extension capability dependencies remain distinct.**
56. **A unified configuration Surface does not merge native harness, Substrate integration/policy, Agent Profile, or Collider presentation authority, and visible or enabled capability does not imply authorization or readiness.**
57. **Inbox read or dismissal state does not approve requests, satisfy gates, or resolve owner-system obligations; authoritative actions use the relevant owner boundary.**

## 17. V1 Acceptance Scenario

The foundational direction is proven when a user can complete the following flow:

1. open a Substrate Workspace in Collider;
2. see its durable Handbook, Lane, and Repository structure in the Workspace Navigator;
3. begin in the canonical Control Plane Workstream Surface;
4. submit both a shell-shaped command and a natural-language request through the same Composer;
5. see a coherent Lane-unfiltered activity view without requiring every provider to share one canonical event store;
6. request a Handbook Artifact and receive a typed inline Surface rather than being forced into its backing file;
7. use **Open as tab** on that Surface through drag, menu, or keyboard interaction;
8. open a Lane Workstream Surface that reuses the same core Workstream components with a Lane-filtered projection;
9. initiate Lane-bound work;
10. close the Lane-related Tab while the underlying execution continues;
11. retain visible Workspace-level awareness that background work remains active or needs attention;
12. reopen the Lane or execution Surface and reconstruct its current state from durable owner-system records or references;
13. explicitly stop or cancel work without confusing that action with closing presentation;
14. make any Handbook change through a Handbook-owned mutation path;
15. complete the flow without depending on a right-side Inspector, a dashboard, a giant chat-history list, or file tabs as the primary information architecture.

Before the V1 navigator capability is complete, a secondary acceptance path MUST demonstrate creating and renaming a group, moving an eligible item between groups and to the ungrouped root, reordering, hiding and restoring it, and restoring defaults. A multiword descendant or user-promoted item must keep its valid display name, and its identity, target state, permissions, and execution behavior must remain unchanged. Required execution awareness and controls must remain reachable.

As expanded destinations are delivered, acceptance must verify that environment edits use Substrate World dependency tooling, harness/profile and enablement changes reach the correct owner, Inbox dismissal does not resolve authoritative work, and users may prioritize Sessions without implicitly resuming one.

A later secondary slice may additionally prove that a Block can be made Sticky in one Workstream projection without becoming Sticky in another projection or being removed from chronological history.

If this flow is coherent, the initial Collider shell has established the intended product foundation.

## 18. Matters This Document Does Not Freeze

This document does not settle:

- whether the Workspace has one canonical event history, federated source histories plus an activity index, query-time federation, or a hybrid;
- whether Workstreams are stored entities, projection definitions, materialized indexes, or hybrids;
- Projection Definition registration authority, identity, versioning, compatibility, and invalidation;
- Activity Assembler location, process boundaries, incremental maintenance, replay, resynchronization, and gap repair;
- Projection Index persistence, snapshots, compaction, privacy, retention, and reconstruction;
- Business Identity formation, correlation, cross-source identity, causal relationships, deterministic presentation ordering, tie-breaking, paging, and cross-projection Activity Item identity sharing;
- the boundary between Source Records, Activity Items, Blocks, and large shell or PTY output streams;
- MCP App object, revision, snapshot, and provider-unavailability behavior;
- exact Control Plane and Lane Activity Projection inclusion rules;
- Qualified Session creation and identity, explicit resume handles, one-Invocation continuation versus persistent continuity mode, provider guarantees, Session History retention and pagination, expiry and recovery, disconnection and reconnection, branching or cloning, Session-to-Workstream cardinality, Lane-spanning behavior, revalidation, fallback, and future Session-oriented Surface behavior;
- Sticky Stack capacity, ordering, persistence, and compaction;
- whether Tab Promotion shares one Surface identity or opens a provider-declared expanded Surface;
- the final execution-instance or agent-topology vocabulary;
- exact execution-tree cardinalities;
- protocol or persistence schemas;
- command parsing and dispatch algorithms;
- Lane creation, rebinding, retirement, or worktree lifecycle details;
- the exact Work Item kind system and the populated dedicated `Work` Navigator projection;
- final navigator labels and grouping below the highest-level default naming constraint, item/shortcut identity, customization gestures, preference storage and sharing, migration, and provider-restoration behavior;
- Environment Dependency configuration sources, effective requirements, realization, readiness, drift, Lane relationships, and the concrete interface to existing Substrate World tooling;
- native harness, Substrate integration, Agent Profile, and Collider presentation configuration schemas, precedence, compatibility, and effects on active execution or Qualified Sessions;
- workspace capability/integration enablement mechanics, credential-binding views, and diagnostics contracts;
- Inbox aggregation, notification/read/dismiss semantics, owner-system acknowledgments, delivery, and retention;
- detailed status vocabularies;
- exact visual dimensions, styling, or component design;
- right-side Inspector modes or idle behavior;
- detailed extension trust classifications, isolation technology, capability-security mechanisms, and External Effect cleanup contracts;
- capability keys, identity, schemas, compatibility, versioning, provider identity and selection, multiple-provider behavior, failover, and Dependency Declaration format;
- Extension Context hierarchy and derivation, Workspace/Lane/Workstream/qualified-Session scoping, Extension Context Mediation and Capability Interception mechanics, relationship to Substrate policy and Handbook semantics, and local versus cross-process capability bridges;
- Extension Contribution atomicity and granularity, lifecycle state transitions, declared recovery criteria, exact equality versus observational equivalence, disposer or inverse composition and ordering, asynchronous teardown, partial-activation recovery, configuration reconciliation, dependency cycles, hot replacement, update rollback, and evidence required for composability claims;
- Extension Point declaration schemas and identity, `Slot` as a possible implementation term, UI Contribution Scope, multiplicity, ordering, fallback, election, replacement and conflict policy, nested-point and descendant lifetimes, React and Tauri rendering boundaries, styling isolation, accessibility enforcement, provider failure handling, client/host pairing, and contributed-Surface serialization or restoration;
- multi-Workspace Project semantics.

These matters are tracked in [`02-collider-v1-open-architecture-questions.md`](./02-collider-v1-open-architecture-questions.md). Later documents may define them, but MUST preserve the foundational ownership, continuity, traceability, and presentation-independence rules established here unless this document is explicitly amended.

## 19. Downstream Document Boundaries

This foundations document is accompanied by the following decision register and should be followed by two bounded contracts.

### `02-collider-v1-open-architecture-questions.md`

Tracks unresolved architectural decisions, research requirements, candidate shapes, and closure criteria. It is non-normative and MUST NOT be treated as implementation authority.

### `03-collider-v1-workspace-shell.md`

Owns the concrete user-facing shell, including:

- overall layout;
- Workspace Navigator destination delivery, default hierarchy and labels, grouping, visibility, keyboard/drag interaction, and restore-default behavior;
- Navigator Layout preferences, contributed-item presentation, and user-selected Sessions prominence;
- Inbox, configuration, harness, profile, environment, enablement, credential-status, and diagnostics presentation;
- Control Plane and Lane Workstream presentation;
- Workstream and Composer interaction;
- local Presentation Actions and their boundary from Invocations;
- Block and Surface presentation modes;
- Sticky Stack behavior when included;
- **Open as tab** behavior;
- minimum execution-awareness placement;
- optional Inspector placement.

### `04-collider-v1-execution-and-binding.md`

Owns the operational path from Invocation to execution, including:

- command and request routing;
- Workspace-bound versus Lane-bound operations;
- Lane selection and binding;
- owner-approved application of configuration and policy changes, Environment Dependencies through Substrate World tooling, Agent Harness and Agent Profile settings, and workspace capability enablement;
- Qualified Session resumption and revalidation, authoritative Inbox actions, and the effects of operational changes on active work;
- background execution and reconnection;
- stop and cancellation behavior;
- execution hierarchy once its vocabulary is resolved;
- Context Record and provenance requirements;
- source-record and Activity Projection synchronization boundaries selected through the open-question process.

Neither downstream document should redefine the canonical terms in the glossary or weaken the foundations established here without an explicit coordinated revision.



# SOURCE: Collider/docs/agent-native/canonical/02-collider-v1-open-architecture-questions.md

Source SHA-256: dfac1ea399908053e40ad647042067060d8585cc05d0f9f9f2f314e9b24a9cff

# Collider V1 Open Architecture Questions

**Status:** Living non-normative decision and research register\
**Document role:** Track unresolved architecture, research needs, candidate shapes, and decision closure\
**Depends on:** [`00-collider-v1-glossary.md`](./00-collider-v1-glossary.md), [`01-collider-v1-foundations.md`](./01-collider-v1-foundations.md)\
**Implementation authorization:** None\
**Normative authority:** None; candidate options in this document are not decisions

## 1. Purpose

This register prevents attractive but unproven implementation shapes from becoming accidental Collider V1 architecture.

An item remains open until a bounded research or design packet:

1. identifies the affected owner systems and existing implementation truth;
2. compares viable options against explicit criteria;
3. records a decision and rejected alternatives;
4. updates the glossary and foundations when their language is affected;
5. places implementation authority in a separate bounded specification or plan.

The settled baseline is narrower than the open questions. It is a non-authoritative summary of constraints established by the glossary and Foundations; if this summary conflicts with either normative document, the glossary and Foundations control:

- Collider provides a coherent Workstream experience without becoming a competing source authority.
- A durable or reconstructable Block is traceable to sufficient Source Records, durable references, or owner-system objects.
- The Control Plane and Lane views should reuse the same core Workstream components.
- The Control Plane is Lane-unfiltered; a Lane view applies an internal Lane filter and default Lane binding.
- `Lane-unfiltered` does not mean raw, unrestricted internal telemetry.
- Sticky Placement and **Open as tab** are distinct presentation behaviors.
- Collider presentation does not own Lane, execution, Handbook, MCP App, shell, or provider state.
- Resolution-reconstructed interaction is the default.
- Qualified Session Continuity remains possible, but it must be explicit and visible rather than inferred from remaining in one Workstream.
- Session History may contribute to Activity Projection and owner-scoped reconstruction, but it is not the Workspace-wide canonical history. Default navigation is not Session-centric; users may prioritize a dedicated Sessions destination without making navigation imply continuation.
- Resuming a Qualified Session does not silently preserve Handbook Scope or Resolution, Lane binding, Substrate policy, Capability Grants, permissions, provider availability, or model identity and configuration without current validation.
- Context and capability mediation are not Isolation Boundaries.
- Untrusted executable extension code requires genuine isolation before it can execute.
- Cleanup and reversion claims are bounded to effects the host can observe, attribute, and control; a Revertible Contribution requires a declared recovery criterion, and compensation is not exact rollback.
- Spatiotemporal Composability is a dynamic-extension design objective, not a guarantee already provided by Collider.
- Temporal Composability is bounded to mediated and attributable Extension Contributions; Spatial Composability relies on declared requirements and provisions plus reactive lifecycle behavior.
- Extension capability access should use owner-mediated and attributable Capabilities where practical, without creating a cross-system source of truth.
- Extension Context is distinct from Handbook Scope, Resolution, and Projection; Substrate Context Packs and model-visible context; Activity Projections; Workstreams; and security Isolation Boundaries.
- Cross-process Capabilities must represent asynchronous and failure semantics honestly rather than pretending to be infallible synchronous local calls.
- Collider-owned layout declares valid Extension Points, while Surfaces remain the protocol-neutral typed application/view abstraction.
- UI Contributions remain distinct from inline, Sticky, Tab, overlay, and Inspector presentation behavior, and UI contribution permission does not imply runtime or semantic authority.
- `Extension Point` is the canonical public architecture term; `Slot` remains a non-canonical implementation or reference term unless separately adopted.
- Host-recognized Projection Definitions assemble eligible Source Records into stable Activity Items; one Activity Item may fold several correlated records or revisions.
- Activity Item Business Identity is mandatory and independent of visual list position, while exact identity sharing across Workstream projections remains open.
- Projection assembly should be deterministic and replayable where source systems permit it; projection-local presentation order does not imply one authoritative total Source Record order; and projection-specific indexes do not become source authority.
- Purely local Collider presentation changes are Presentation Actions rather than Substrate-owned Invocations.
- Work Items remain discoverable, while a populated dedicated `Work` Navigator section is secondary to the first coherent V1 slice.
- Collider's one-word naming rule applies only to its highest-level default navigator labels, not descendant labels, user-customized names, user-promoted entries, or canonical architectural terms.
- Navigator Items, Groups, and Layout are presentation concepts; users may rename groups, change displayed membership and ordering, and show eligible items ungrouped without changing owner identity, permissions, capability enablement, or execution.
- The expanded destination map includes Inbox, configuration/policy, Agent Profiles, Agent Harnesses, Environments, workspace capability/integration enablement, credential bindings, diagnostics, and actual Qualified Sessions; it is not a fixed top-level tree or an all-at-once first-slice requirement.
- Environment Dependency configuration feeds existing Substrate World dependency tooling; it is not a new Collider dependency engine or a synonym for package or Extension capability dependencies.
- Native harness, Substrate integration, and Collider presentation settings may share a Surface without sharing authority.
- Basic Navigator Layout customization is required within V1, distinct from a deferred general-purpose layout builder; hiding destinations must not remove all execution awareness or access to cancellation and recovery.
- Inbox read/dismiss state does not implicitly resolve owner-system obligations, approvals, gates, or execution state.

### 1.1 Research intake — BB and frontend-foundation evaluation (2026-09-16)

This intake extends research scope only. It does not adopt a dependency, revise the glossary or Foundations, select a Workspace-history architecture, or authorize an implementation spike. The preceding baseline and existing question decisions remain controlling within their stated authority.

**Evaluation constraints supplied by the product owner:** the target frontend is Tauri + React + TypeScript + Vite. Substrate, Handbook, and the Unified Agent API are the existing operational and semantic foundation to inspect, not missing infrastructure to recreate for a library. Existing AI Elements components are provisional and may be discarded entirely; their earlier use to refine ds-skills has already delivered value. Do not treat preservation of those components, their APIs, or their past curation cost as a decision criterion. Evaluate useful tooling and all remaining integration work prospectively.

A reusable framework or controller may implement an accepted Collider frontend role. Do not require a second custom controller, projection store, queue, or approval owner solely because the role has a Collider name. Equally, do not silently replace owner-system authority to accommodate a vendor. Any proposed semantic amendment requires separate adjudication.

### 1.2 Research sources and evidence status

The following exact Library names identify supporting inputs. Names identify documents, not new canonical terms. Existing research is non-normative; source inspection, documentation claims, upstream-reported tests, and locally executed validation must remain distinguishable. Retrieve the named inputs in the evaluation session rather than assume a historical chat download link grants access.

| Input | Exact Library name or source | Evidence status and permitted use |
|---|---|---|
| Accepted Collider terms and direction | `00-collider-v1-glossary.md`; `01-collider-v1-foundations.md` | Use their latest verified Library versions, not older `(1)` copies or stale mounted copies. The current register adds questions, not new authority. |
| DeepSeek/Cordis research | `deep-seek-harness-deep-research-report.md` | Supporting composition and architecture research; not a dependency, terminology, or proof authority. |
| BB research | `bb-deepseek-collider-architecture-report.md` | September 15, 2026 report pinned to `41e7132e817306e43f68f91bde0a6b9f79af99de`. Provides hypotheses and failure cases; no build, test suite, or live provider workflow was executed by that review. |
| BB evidence ledger | `research-source-ledger.json` | The ledger identifies `get-bb/bb` and the same revision. Verify that identity rather than selecting an unrelated file with a generic ledger name. |
| Earlier Agent Native evaluation | `Agent-Native Architectural Evaluation for Collider and Substrate` | Native Deep Research report, pinned to `92e6aec8b26b68e870384009af2a4d40cc68e41d`. Preserve source observations; reassess fit/adoption conclusions against actual Substrate/UAA and the disposable-UI clarification. This is a report title, not an assumed Markdown filename. |
| Corrected evaluation instructions | `agent-native-collider-research-prompt-v2.md` | Active research brief. Supersedes earlier `agent-native-collider-grounded-research-prompt.md` and earlier reuse-prompt variants for this task. Includes the implementation-grounded context and disposable-UI instruction; not a completed evaluation. |
| Corrected Agent Native report | `Corrected Agent Native Evaluation for Collider` | Completed Deep Research report; independently hashed during adjudication as SHA-256 `2e23d0d698e5023483292828648dbfda7a08d408f88b5cc06de8bf6697f180f8`. Its original machine-readable evidence ledger is unavailable and its claimed ledger digest was not verified; decision-relevant source links were selectively rechecked in the adjudication. |
| Adjudication decision coverage | `adjudication-decision-coverage.md` | Advisory extraction of 38 material recommendations and 14 stop/reversal conditions. Selects a proof direction, not adoption or implementation. |
| Adjudication check record | `adjudication-check-record.md` | New verification record for the bounded adjudication. Re-establishes selected pinned source links and bundle integrity; does not reconstruct or replace the missing original research ledger. |
| Owner-boundary/proof specification | `agentkit-owner-boundary-proof-spec.md` | Non-normative proof-planning contract produced after adjudication. Defines evidence gates and mapping obligations; does not authorize implementation or select a dependency. |

**Live implementation entry points:** [Substrate implementation branch](https://github.com/atomize-hq/substrate/tree/feat/runtime-refactor-e3-ac1-projection-authority-correction), its [runtime-refactor control pack](https://github.com/atomize-hq/substrate/tree/feat/runtime-refactor-e3-ac1-projection-authority-correction/llm-last-mile/runtime-refactor), [Unified Agent API staging](https://github.com/atomize-hq/unified-agent-api/tree/staging), and [Collider](https://github.com/atomize-hq/collider). Follow current owner contracts and code; resolve branch heads once per evaluation and pin decisive reads. Distinguish canonical target documents, landed implementation, compatibility paths, non-authoritative indexes, and unexecuted proof claims.

**Candidate and comparison entry points:** [Agent Native repository](https://github.com/BuilderIO/agent-native), [website](https://www.agent-native.com/), [documentation](https://www.agent-native.com/docs/), and [BB repository](https://github.com/get-bb/bb). Repo source, published artifacts, and generated repository summaries may differ. Verify important claims against pinned primary sources; neither an attractive catalog nor a prior favorable recommendation selects the outcome.

Separate changes caused by corrected project understanding, upstream revision changes, and new execution evidence. The earlier implementation-grounded reassessment was conversation guidance; the active v2 brief carries its scope correction without inventing a separately saved report.

### 1.3 Adjudication intake — Agent Native frontend proof direction (2026-09-16)

The corrected evaluation and bounded adjudication are complete as advisory evidence. They do **not** close `C-OQ-027`, authorize a dependency, authorize a fork, authorize a proof branch, or amend the glossary or Foundations. The accepted adjudication disposition is **ADJUST — retain the shortlist; defer adoption**.

The provisional shortlist for a separately authorized proof is:

1. public AgentKit controller/protocol plus suitable React regions, selective Toolkit, and one owner-routed bridge while preserving Collider-defined Workstream/Control Plane/Lane semantics;
2. a headless-first AgentKit variant when controller/protocol reuse proves sound but reference React composition does not;
3. substantial Toolkit reuse with another single controller when AgentKit lifecycle translation costs exceed demonstrated value;
4. a bounded coherent frontend fork when a small number of owned controller/composition changes preserve more useful behavior than a translation-heavy public adapter;
5. a broader owner-preserving framework/fork remains eligible but lower priority until retained-value and maintenance evidence justifies it.

The adjudication rejects **authority-displacing** Core adoption and a duplicate TypeScript CLI-agent backend. It does **not** reject every Core-labelled import, sidecar, fork, or source-owned path by name. Accepted Collider semantics constrain outcomes, but they do not require bespoke Collider implementations.

The most consequential unproved mappings are: pending command versus owner-accepted receipt; accepted/queued/running state; cancellation request versus authoritative terminal effect; client subscription completion versus owner terminal truth; approval protocol segments versus exact owner operations; library thread/run identities versus Workstream, Qualified Session, Invocation, Receipt, Source Record, and Activity Item identities; transcript state versus reconstructed model context; automatic queue promotion versus owner authorization; and AgentKit sequence/checkpoint state versus owner cursors and projection sequences.

Evidence must remain staged:

- **Contract gate:** establish exact supported owner entrypoints, call paths, schemas, policy checks, and field-by-field identity/cursor/approval/cancel mappings.
- **Fixture/client proof:** exercise the actual public AgentKit controller with controlled fixtures and coordinated Control Plane/Lane/placement views. This proves candidate expressiveness and client behavior only.
- **Owner-integrated proof:** repeat the same product scenario through the supported owner boundary and real declared execution path, preserving a traceable join from user intent through owner acceptance and terminal/approval truth.
- **Deployment/artifact/economics proof:** verify exact package artifacts and rights/notices, Tauri/Vite/CSP/a11y behavior, dependency and emitted bundle costs, startup/memory/replay/cleanup behavior, and measured maintenance tradeoffs.

The original corrected-evaluation machine-readable ledger remains unavailable. Its absence is an evidence limitation, not a reason to reconstruct a substitute or to erase the corrected report. The adjudication check record is new evidence and MUST NOT be represented as the recovered ledger.

## 2. Status Vocabulary

| Status | Meaning |
|---|---|
| **Open** | The problem and decision boundary are known; research or comparison is still required. |
| **Researching** | A bounded research packet is actively gathering implementation and ecosystem truth. |
| **Decision-ready** | Viable options and consequences are sufficiently documented for an explicit decision. |
| **Resolved** | An accepted decision exists and affected authority documents have been updated. |
| **Deferred** | The decision is intentionally postponed and the current architecture preserves more than one viable path. |

## 3. Open Decision Register

### C-OQ-001 — Workspace history and Workstream projection architecture

**Status:** Open\
**Decision:** Determine whether coherent Workspace activity is backed by:

- one canonical Workspace event history;
- federated owner-system histories plus a Workspace activity index;
- query-time federation;
- a hybrid using references, selected snapshots, summaries, and materialized projection items.

**Must evaluate:** source authority, duplication, cross-source ordering, causal relationships, Activity Assembler placement, projection-definition registration boundaries, materialized Activity Items versus indexes versus query-time computation, paging, replay, resynchronization, gap repair, reconstruction, offline behavior, snapshots, retention, compaction, performance, privacy, provider availability, and migration cost. Distinguish authoritative owner histories from view caches and retained audit evidence; compare immutable snapshots with read-boundary tokens over mutable history, and evaluate the recovery contract of each source rather than assuming one universal event log; owner-to-projection cursor mapping, projection version, snapshot cut, source completeness, and the distinction between complete source admission and view filtering; bounded library thread snapshots must not become a universal Workspace-history decision.

**Current constraint:** Neither the glossary nor foundations assumes one canonical event store. Projection machinery may own derived indexes and fold state but does not become source authority. Qualified Session Histories may participate as owner-scoped source histories without resolving this Workspace-wide choice.

### C-OQ-002 — Workstream identity and persistence

**Status:** Open\
**Decision:** Determine whether a Workstream is persisted as an independent entity, a durable projection definition, a materialized index, a reconstructable interaction context, or a hybrid.

**Must evaluate:** stable Workstream identity; whether Projection Definitions are part of Workstream identity or separately registered; branching; archival; reopening; Lane rebinding; association with zero, one, or several Qualified Sessions over time; whether one Qualified Session may appear through several Workstream projections; which Presentation Actions are retained as Collider Source Records, restored as presentation state, or intentionally omitted; selection provenance; deletion semantics; definition-version changes; and whether multiple UI placements share one Workstream identity. Evaluate whether a library controller can implement the client-side Workstream projection role without creating duplicate message, queue, approval, or optimistic-state ownership; keep library thread identity distinct from durable Substrate orchestration authority and provider-native continuity.

### C-OQ-003 — Source Record and Activity Item identity, ordering, and causality

**Status:** Open\
**Decision:** Define how records from Substrate, Handbook, MCP Apps, shell or PTY facilities, and Collider receive stable references; correlate into Activity Item Business Identity; and are ordered, located, or causally related inside an Activity Projection.

**Must evaluate:** owner-supplied versus derived Business Identity; correlation and fold rules; canonical owner-system and cross-source causal relationships where knowable; total versus partial source ordering; projection-local deterministic presentation ordering; tie-breaking; whether a chosen linearization is durable or recomputed; whether different projections may validly linearize the same concurrent records differently; predecessor or projected-location indexes; clock skew; concurrent Lane activity; parent/child relationships; deduplication; retries; revisions; tombstones; late-arriving records; cross-source identity collisions; and whether Control Plane, Lane, or other projections share Activity Item identity or only Source Record identity. Locate native harness event normalization in the existing Substrate/UAA integration versus target-view Activity Assembly; examine duplicate terminal events, reused provider identifiers, late child activity, and preservation of unknown, absent, or incomparable provider metrics.

### C-OQ-004 — Block backing and reconstruction contract

**Status:** Open\
**Decision:** Define the minimum durable information required to explain, reopen, or reconstruct an Activity Item, Block, or Surface.

**Must evaluate:** one-to-one versus composite backing; inspectable fold provenance; Projection Definition identity and version; materialized, indexed, or computed Activity Items; live provider objects versus snapshots; renderer- and definition-version changes; projection invalidation; unavailable providers or source objects; partial reconstruction; stale references; Presentation Action retention and provenance; selected text or projected-node provenance; and which Blocks may remain intentionally ephemeral. Evaluate whether rich Activity Items need bounded declarative fallback presentation when an executable renderer or source provider disappears; distinguish retained explanation from authoritative source state and from a fully reconstructable interactive Surface.

### C-OQ-005 — Shell and PTY capture boundary

**Status:** Open\
**Decision:** Determine which portions of shell and PTY activity become durable Source Records or structured summaries.

**Must evaluate:** command identity, working directory, environment, process and exit status, raw byte streams, ANSI state, interactive applications, `tail` or long-running output, storage volume, privacy and redaction, replay, pagination, compaction, and how large streams become stable summaries, references, or structured test and build Activity Items without treating every byte as a business object. Test how interrupted, buffered, and long-running streams retain truthful output and status through pagination or reconnect without causing replay of the underlying shell operation; library thread/controller identity versus Workstream identity and durable Substrate orchestration authority; one accepted work item must not acquire duplicate command, queue, approval, or optimistic-state owners merely because it appears in several views; exact user-command-to-owner-receipt correlation; lost-acknowledgement reconciliation; repeated identical content with different request identities; source-cursor to projection-sequence mapping; and filtering after complete source admission rather than fabricating sequence gaps; portable fallback or reconstruction behavior when a richer renderer is missing or incompatible, without promoting retained presentation metadata into source authority.

**Current constraint:** Normal raw shell and PTY semantics must remain truthful; every terminal byte need not become an individual Workspace activity item.

### C-OQ-006 — MCP App state, revision, and lifecycle integration

**Status:** Open\
**Decision:** Determine how Collider projects MCP App objects and events without copying or competing with provider-owned state.

**Must evaluate:** object identity; revision correlation and folding into Activity Items; snapshots; refresh; provider disappearance or unavailable source objects; partial reconstruction; permissions; credentials; caching; offline reconstruction; retention; app and Projection Definition upgrades; and whether an app supplies compact, inline, Sticky, and Tab-specific Surface representations. Evaluate portable identity, revision-aware fallback, and the separation of widget presentation from validated owner actions when an MCP App or its renderer is unavailable.

### C-OQ-007 — Control Plane and Lane projection-association rules

**Status:** Open\
**Decision:** Define which Activity Items appear in the Lane-unfiltered Control Plane and each Lane-filtered Workstream.

**Must evaluate:** Workspace-level Invocations later dispatched to a Lane; cross-Lane orchestration; activity associated with several Lanes or Scopes; shared Handbook or MCP App Source Records; view-specific correlation, folding, ordering, visibility, and projected location; historical Lane rebinding; summaries; internal-detail disclosure; and whether Control Plane and Lane views share Activity Item Business Identity or only Source Record identity. Compare coordinated Control Plane and Lane views under a shared client controller or another projection arrangement without assuming one library thread equals one Lane, Workstream, or Qualified Session; the same accepted owner work appearing coherently in Control Plane and Lane projections under one behavioral owner; complete per-run event admission before Lane filtering or an explicit projection-sequence remap; and coordinated inline, Sticky, and Tab placements without duplicated command or approval ownership.

**Current direction:** Control Plane and Lane views reuse the same core Workstream Surface and differ primarily by internal projection and binding posture. They may assemble some of the same Source Records differently.

### C-OQ-008 — Explicit Qualified Session Continuity

**Status:** Open\
**Decision:** Define how users intentionally create, resume, reuse, or continue an owner-specific model-provider, runtime, agent-harness, shell, or PTY Qualified Session while resolution-reconstructed behavior remains the default.

**Must evaluate:** one-Invocation continuation versus persistent continuity mode; Qualified Session creation; explicit resume handles; provider-specific identity and guarantees; expiration, recovery, disconnection, and reconnection; context compaction; provider or model changes; Handbook Scope or Resolution changes; Lane rebinding; Substrate policy, Capability Grant, permission, trust, and compatibility revalidation; termination; branching or cloning; Qualified-Session-to-Workstream cardinality; whether a Qualified Session may span several Lanes; how Qualified-Session-associated activity appears in the Lane-unfiltered Control Plane and Lane-filtered views; private, temporary, or hidden Qualified Sessions; Session History pagination, retention, privacy, redaction, deletion, and reconstruction; interaction with Projection Definitions, Activity Assemblers, Activity Items, and Context Records; the dedicated Sessions destination and its Session-oriented Surface; grouped, conditional, prominent, or user-selected root placement; the distinction between browsing Session resources and explicitly invoking continuation; coordination with Navigator Layout preferences under C-OQ-025; visible indication that continuity is active; fallback when a provider cannot resume; and the exact state promised by `continue`. Map library threads, provider-native resume resources, and Substrate durable orchestration sessions separately; inspect the actual UAA and Substrate semantics for detach versus cancellation and reconnect versus resubmission before assessing a candidate transport; library thread/run identities and protocol continuation segments versus provider Qualified Session identity and Substrate durable orchestration identity; approval continuation must not silently create false owner Invocations, Receipts, or Session continuation.

**Current constraints:** a dedicated Sessions destination is accepted product direction, while its delivery details remain open; users may prioritize it without changing the resolution-reconstructed default or automatically resuming a Session; remaining in one Workstream does not imply Session Continuity; a Workstream may span zero, one, or several Qualified Sessions; one Qualified Session's activity may appear in several Workstream projections where policy permits; Session History is one owner-scoped source history rather than the Workspace-wide canonical history; and resumption does not make a Qualified Session authoritative for Handbook semantics, Lane placement, Substrate policy, Collider Workstream identity, or unvalidated policy, capability, permission, provider, or model state. No universal cross-provider Session protocol, provider-specific resume API, automatic continuation rule, final Session UI, exact persistence schema, Activity Projection storage choice, or compaction algorithm is selected.

### C-OQ-009 — Sticky Stack behavior

**Status:** Open\
**Decision:** Define Sticky Stack capacity, ordering, persistence, compaction, and controls.

**Must evaluate:** likely two-or-three-item visible limit, compact summaries, manual ordering, per-Workstream versus per-placement state, restoration after restart, Surface updates, duplicate Activity Items, accessibility, and narrow-window behavior.

**Current constraint:** Making an item Sticky preserves its chronological placement and does not make it Sticky in every other projection.

### C-OQ-010 — Tab Promotion and Surface identity

**Status:** Open\
**Decision:** Determine whether `Open as tab` creates another placement of the same Surface identity or opens a provider-declared canonical expanded Surface.

**Must evaluate:** state sharing, concurrent interaction in inline and Tab placements, focus, close behavior, restoration, origin anchors, navigation, provider capabilities, and stale or incompatible views.

### C-OQ-011 — Execution-instance and agent-topology model

**Status:** Open\
**Decision:** Resolve or replace the provisional glossary vocabulary for Run, Root Run, Child Run, Run Node, Run Tree, Agent, Subagent, and Subtask.

**Must evaluate:** actual Substrate and harness models, orchestration-only versus environment-touching execution, Lane binding, nested and parallel work, cancellation, retries, user-meaningful versus internal activity, persistence, and cross-provider normalization. Start from current Substrate authority, dispatch, supervision, receipt, and runtime contracts plus the UAA version and features Substrate actually consumes; distinguish accepted work, process exit, event-stream finality, and owner-authoritative terminal closeout without freezing the provisional execution vocabulary; same-state multi-placement under one behavioral owner, including whether a shared controller is sufficient without forcing Surface, Workstream, or Activity Item identity equivalence; pending request, owner-accepted receipt, accepted-but-queued, actually running, cancellation-requested, authoritative terminal, and protocol-segment identities; library run completion or local timestamps must not define owner execution truth.

**Current constraint:** UI presentation never owns execution lifetime, and internal machinery should be progressively disclosed beneath user-meaningful activity.

### C-OQ-012 — Context lineage and provenance retention

**Status:** Open\
**Decision:** Define the stable identities and retention needed to support future inspection of Resolved, Injected, Acquired, and Produced material.

**Must evaluate:** model-visible exposure, file and line references, artifact snapshots, tool reads, context compaction, redaction, privacy, Evidence admission, Source Record references, and storage cost. Trace selected references through owner Resolution, admitted revisions, assembled Context Packs, delivered context, and provider acceptance; evaluate whether reusable context UI can display these distinctions without inventing missing telemetry or semantic authority; exact joins from user intent to owner acceptance, Source Records, projected events, Context Records, approval records, and terminal outcomes; transcript arrays and library metadata remain projection inputs rather than model-context or provenance authority.

**Current constraint:** Deferring the Inspector must not require discarding the records needed to build truthful context inspection later.

### C-OQ-013 — Cordis feasibility and reference monitoring

**Status:** Open\
**Decision:** Determine whether Cordis merits a bounded prototype or integration spike, should remain a reference-only implementation study, or should be rejected for Collider or Substrate use.

**Must evaluate:** TypeScript and runtime fit with Rust, Tauri, Substrate, Handbook, and Collider; whether any plausible use is limited to trusted client-side composition; whether a Rust-native analogue would fit the architecture better; dependency footprint, upgrade burden, maturity, maintenance, coupling, and long-term compatibility risk; the evidence and criteria that would justify embedding Cordis later; explicit rejection criteria; and how to monitor DeepSeek Harness while it remains immature or rapidly changing without treating it as canonical authority.

**Current constraint:** Cordis is not a selected dependency or the canonical Collider or Substrate extension substrate. Any prototype or integration spike requires separate bounded authorization and MUST preserve the existing Handbook, Substrate, and Collider authority boundaries and Collider's protocol-neutral Surface model.

### C-OQ-014 — Extension trust classes, isolation, and capability security

**Status:** Open\

**Decision:** Define the executable-extension trust classifications, determine which classes may run in process, and establish the Isolation Boundary and capability-security requirements for every other class.

**Must evaluate:** first-party, third-party, locally authored, user-installed, and remotely sourced code, including which classes may qualify as trusted; separate-process, WASM, sandboxed-runtime, and other isolation candidates without assuming one outcome; filesystem, network, process, credential, clipboard, browser, webview, Tauri, and host-object mediation; which authority may issue, enforce, delegate, attenuate, revoke, and audit each Capability Grant; prevention of consumer or provider self-authorization outside an owner or delegated authority role; secret injection, use, redaction, and lifetime; provider identity, package provenance, signing, update channels, rollback, and supply-chain trust; host/client split extensions and the trust boundary between their halves; crash containment and recovery after non-graceful termination; and whether UI-only contribution can remain structurally separate from host runtime execution. Evaluate the actual privileges exercised by candidate browser, Tauri, sidecar, host-process, and agent-authored UI paths; distinguish admitted trusted code from untrusted code and provider-package identity from authority, rather than relying on sandbox or lifecycle labels.

**Current constraint:** Context or capability mediation is not a sandbox. Untrusted executable extensions are not authorized to execute with the ambient authority of a trusted Collider, Substrate, or Handbook process by default. Surface rendering permission does not imply filesystem, process, network, credential, semantic, or other host authority. No concrete isolation technology is selected.

### C-OQ-015 — External effects, cleanup guarantees, and reversibility claims

**Status:** Open\

**Decision:** Define effect classes and the truthful cleanup, unloadability, recovery, and reversibility claims allowed for each extension trust class and host boundary.

**Must evaluate:** Mediated Effects versus External Effects; which registrations and resources can be observed, attributed, revoked, or unwound by the host; the declared recovery criterion for each claimable contribution class; exact equality versus declared observational equivalence; which observations and invariants are authoritative; which owner or policy authority defines the criterion; whether unrelated state must remain unchanged; how the criterion is evaluated after interleaved or ordering-sensitive contributions; what evidence proves it; what happens when it is untestable or unverifiable; transaction and copy-on-write candidates; Compensating Actions; idempotent cleanup and retry; irreversible or already-observed emissions; crash-time cleanup when graceful disposal does not run; orphaned resources and partial failure; audit evidence; and the exact user- and developer-facing claims permitted for unload, update, deactivation, rollback, compensation, and recovery. Compare guarantees by failure phase and component class; distinguish rollback of a registration set from reversal of candidate storage writes or external effects, and distinguish releasing a subscriber from cancelling owner-managed work.

**Current constraint:** Automatic tracking, cleanup, or reversion may be claimed only for effects within an observed and controlled mediation boundary. A Compensating Action is not an exact inverse, and the architecture MUST NOT claim complete unload or rollback when untracked External Effects may remain.

### C-OQ-016 — Extension contribution lifecycle and temporal composability

**Status:** Open\

**Decision:** Define the atomic unit and lifecycle semantics of an Extension Contribution, and determine what implementation evidence would justify a bounded Temporal Composability claim for a specific extension class.

**Must evaluate:** Extension and contribution granularity; local registration ownership; host-tracked disposers, inverses, and recovery operations; declared recovery criteria and their owner; exact equality versus observational equivalence; composition and ordering of cleanup; evaluation after interleaved contributions; preservation of unrelated state; asynchronous teardown; partial activation failure; failed update and update rollback; crash recovery when graceful cleanup does not run; configuration reconciliation; preservation of unaffected Extensions and host state; hot replacement across Rust, Tauri, React, Substrate, Handbook, and provider processes; External Effect and non-revertible-effect classification; idempotency and retry; conflicts or ordering-sensitive contributions; untestable or unverifiable criteria; and the tests, receipts, fault injection, or other evidence needed to substantiate a Temporal Composability claim. Investigate generation ownership, stale handles, candidate activation failure, bounded asynchronous teardown, and non-graceful failure; require contribution-class evidence rather than infer transactional hot replacement from a successful reload demonstration.

**Current constraint:** Temporal Composability is a design objective only. Any claim is limited to mediated, attributable contributions within an explicit system boundary and valid cleanup, recovery, or inverse behavior. A Compensating Action is not exact rollback, and composability does not replace trust or Isolation Boundaries.

### C-OQ-017 — Extension dependency topology and spatial composability

**Status:** Open\

**Decision:** Define the abstract required/provided dependency model and reactive lifecycle behavior needed to evaluate Spatial Composability without selecting a concrete dependency-resolution or capability-resolution interface.

**Must evaluate:** dependency and provider identity; activation only when requirements are satisfied; provider disappearance or material change; deactivation, inactivity, and reactivation behavior; dependent teardown before provider withdrawal when cleanup requires the provider; inspectable dependency topology; configuration reconciliation; extension and component granularity; dependency cycles and mutually dependent Extensions; cycle-breaking architecture; provider conflicts; independence, commutativity, and ordering-sensitive contributions; compatibility evolution; and the evidence or validation required to claim Spatial or Spatiotemporal Composability for a particular extension class and bounded system. Evaluate provider withdrawal, dependent teardown, capability compatibility changes, and mixed frontend/host generations under the existing bounded composability objective; a library lifecycle mechanism is not proof of the objective.

**Current constraint:** No concrete dependency-resolution interface, package format, hot-replacement mechanism, or formal guarantee is selected. Spatial Composability remains subject to the existing authority, trust, isolation, mediation, and External Effect constraints.

### C-OQ-018 — Extension Context and capability mediation model

**Status:** Open\

**Decision:** Define the bounded Extension Context and capability-composition model without creating a cross-system authority object or selecting a concrete runtime or protocol.

**Must evaluate:** typed capability keys, interface schemas, namespacing, compatibility, and versioning; capability identity and provider identity; Required Capability, Provided Capability, and Dependency Declaration representation; which owner or delegated authority issues and enforces Capability Grants for each capability seam; prevention of consumer or provider self-authorization outside that role; multiple providers, selection, multiplexing, replacement, and failover; hierarchical or derived Extension Contexts; Workspace, Lane, Workstream, Qualified Session, policy, trust, audit, instrumentation, and extension-instance specialization; Extension Context Mediation, Capability Interception, attenuation, approval, revocation, attribution, and audit metadata; inspectable dependency topology; relationship to Substrate policy without redefining it; relationship to Handbook Contracts, Scope, Resolution, and Projection without claiming equivalence; whether the Cordis evaluation in C-OQ-013 has value only for trusted client-side composition; and whether a Rust-native capability/context substrate is required for owner-aligned composition across Substrate and Handbook. Distinguish declared support, negotiated support, policy authorization, operational readiness, and unknown capability state; identify which owner supplies each fact and whether candidate adapters preserve it.

**Current constraint:** An Extension Context is an extension-facing capability environment, not Handbook Resolution, a Context Pack, an Activity Projection, a Workstream, an execution environment, a security sandbox, or a new source of truth. Required and Provided Capabilities remain owner-mediated, and providers retain authority for the state and services they expose. No concrete capability schema, key system, provider-selection algorithm, Cordis dependency, or Rust-native implementation is selected.

### C-OQ-019 — Local and cross-process capability contract semantics

**Status:** Open\

**Decision:** Define truthful local and cross-process capability contracts and bridge behavior across Rust, Tauri, React, Substrate, Handbook, Collider, and provider processes.

**Must evaluate:** which Capabilities may be synchronous and local; which MUST be explicitly asynchronous; serialization and compatibility boundaries; provider discovery and loss; error, cancellation, deadline, timeout, retry, idempotency, backpressure, and partial-failure semantics; replacement and failover; host/client split Extensions; attribution across process boundaries; trust, Capability Grant, and Isolation Boundary enforcement at the bridge; lifecycle coordination when either side crashes or disconnects; and how developer ergonomics can remain consistent without hiding remote execution or failure. Map candidate frontend transports to supported Substrate ingress and observation boundaries, not private Rust internals or a second CLI-agent backend; examine request abort, subscription release, explicit cancellation, disconnect, retry/idempotency, and conformance-test fidelity separately.

**Current constraint:** A remote Capability MUST NOT masquerade as an infallible synchronous local call. Capability bridges do not transfer owner authority or replace trust classification and Isolation Boundaries. No transport, RPC framework, serialization schema, or bridge topology is selected.

### C-OQ-020 — UI Extension Point declaration and contribution policy

**Status:** Open\

**Decision:** Define the host-owned Extension Point declaration and UI Contribution composition policy without selecting concrete React APIs, a package manifest, or an extension runtime.

**Must evaluate:** navigator contribution identity and admission; host-default placement versus user-selected group, order, visibility, or ungrouped placement; preservation of hidden-item preferences across provider reload; highest-level default labels versus unrestricted descendant and user naming; coordination with C-OQ-025; the exact Extension Point Declaration schema; identity, namespacing, and compatibility; whether `Slot` is useful only as an implementation term; accepted contribution kinds; zero, one, list, keyed, or other multiplicity; ordering and priority; owner-defined fallback entries; election, replacement, and conflict resolution; UI Contribution Scope across Workspace, Lane, Workstream, Qualified Session, Surface, and provider boundaries; contribution permissions and their relationship to Capability Grants; whether authorized Extensions may declare child Extension Points; and how Extension Points relate to inline, Sticky, Tab, overlay, and Inspector presentation locations without becoming those placements. Evaluate both controlled presentation components and coherent framework-supplied interaction regions through owner-approved contributions; no existing AI Elements component API is a preservation constraint, and a library may implement an accepted frontend role rather than wrap a duplicate bespoke implementation.

**Current constraint:** `Extension Point` is the canonical public architecture term. Collider-owned layout declares valid points and retains composition authority while users control supported Navigator Layout organization; a contribution cannot override user preferences or obtain runtime permissions by being displayed. Surfaces remain the typed rendered abstraction, placements remain separate, and permission to contribute UI does not imply runtime, filesystem, process, network, credential, or semantic authority. No declaration schema, registry API, multiplicity model, election algorithm, or manifest format is selected.

### C-OQ-021 — UI contribution rendering, nesting, failure, and restoration boundaries

**Status:** Open\

**Decision:** Define how contributed UI crosses client and host boundaries, survives or fails safely, and is reconstructed without moving authoritative state into Collider presentation.

**Must evaluate:** navigator item disappearance, user-hidden state, provider return, stale shortcuts, and default-layout upgrades without erasing user organization; accessibility and recovery when groups or items are hidden; navigation-specific restoration under C-OQ-025; React and Tauri rendering boundaries; client-only versus host/client-paired contributions; dynamic registration and hot replacement; lifecycle coordination with Extension and provider replacement; nested Extension Point lifetime; parent Surface replacement and descendant contribution behavior; provider failure, error boundaries, fallback, and host recovery; accessibility obligations including keyboard, focus, announcements, and fallback semantics; themes, layout constraints, styling, and CSS isolation; narrow props, handles, and owner-approved data access; contribution permissions; serialization and restoration of contributed Surface identity and presentation state; provider unavailability during restoration; and whether an Extension may expose child Extension Points without creating unowned layout authority. Investigate functional and accessible fallback, retained display explanation, safe owner-action callbacks, and renderer-loss behavior; assess Tauri/WebView and CSS integration through actual imports and contracts rather than equating a web demo with a proven desktop integration.

**Current constraint:** A UI Contribution retains its Contribution Provider identity and owner-system bindings. Authoritative state remains with the nearest owner, and absence, failure, replacement, or restoration of contributed UI must not silently transfer that state to Collider. No concrete React API, Tauri bridge, styling mechanism, hot-replacement design, serialization format, or restoration guarantee is selected.

### C-OQ-022 — Projection definition, assembler, replay, and repair lifecycle

**Status:** Open\

**Decision:** Define how Projection Definitions are registered, versioned, executed, incrementally maintained, replayed, invalidated, and repaired without selecting the final Workspace-history or persistence topology.

**Must evaluate:** who may register Projection Definitions and under what authority or Capability Grant; definition identity, compatibility, versioning, and migration; where an Activity Assembler runs across Rust, Tauri, React, Substrate, Handbook, Collider, or provider processes; projection-specific fold state and Projection Index ownership; predecessor and projected-location indexes; initial reconstruction; incremental append; prepend and older-page loading; pagination; replay; snapshots and compaction; gap detection and repair; explicit full resynchronization; definition and renderer changes; projection invalidation; provider disappearance; unavailable source objects; partial reconstruction; privacy, redaction, and retention; deterministic replay and fault-injection validation; and how MCP App revisions or large shell and PTY streams become correlated Activity Items, summaries, or references. Compare complete client-controller adoption with presentation-only reuse; inspect per-run sequence checkpoints, pagination read boundaries, invalidation, duplicate/gap handling, and partial reconstruction, and require incremental-versus-reconstructed equivalence tests without replaying external effects.

**Current constraint:** Projection Definitions, Activity Assemblers, Projection Indexes, Activity Items, and Blocks do not become semantic, execution, or Source Record authority. No canonical Workspace event store, assembler process, database, language, persistence schema, Lane or Control Plane inclusion predicate, cross-projection Activity Item identity rule, or concrete Session Continuity mechanism is selected.

### C-OQ-023 — Workspace environments and World dependencies

**Status:** Open\
**Decision:** Define Collider's configuration and status Surfaces over Substrate's existing World dependency tooling without creating a competing environment/dependency model.

**Must evaluate:** current Substrate owner contracts and tooling; Workspace configuration sources and inheritance; dependency declaration versus resolved/effective requirements versus realized or observed readiness where supported; toolchains, binaries, libraries, services, variables, and secret references; Environment Dependencies versus Repository/package dependencies and Extension Dependency Declarations; Workspace defaults, Lane relationships, and permitted overrides; provisioning, validation, readiness, drift, failure, and recovery; offline or unavailable tooling; change previews and effects on active environments or future execution; authorized mutation, credentials, and audit; and incremental delivery of the Environments destination. Inspect existing World-tooling support for durable operation intent, attempt ownership, stale-attempt fencing, uncertain dispatch, idempotent retry, and persistent cleanup obligations; classify landed, planned, and unverified behavior instead of introducing a parallel Collider provisioning state machine; frontend controller, renderer, registry, or package boundaries as non-security boundaries; adoption or source ownership does not grant filesystem, credential, process, network, or semantic authority; framework-supplied capability descriptors versus owner-supported capability truth; unknown, unsupported, unavailable, degraded, and available states must remain distinguishable; a possible Tauri-native AgentTransport or equivalent bridge, including supported owner entrypoints, authentication, restart, cancellation, backpressure, replay, and deployment semantics; do not force HTTP or a Node sidecar solely to imitate a first-party backend; functional fallback when contributed or framework-owned renderers are unavailable; Tauri/WebView CSP, IPC permissions, URL/Markdown/upload/clipboard handling, accessibility, and renderer failure containment; owner cursor to projection sequence mapping, mapping version, snapshot cut, retention limits, gap repair, and replay after process or renderer restart; a filtered view must not preserve source sequence numbers in a way that creates artificial gaps.

**Current constraint:** Substrate remains authoritative for World preparation, dependency realization, policy, and execution. Collider exposes owner-supported requirements and status rather than asserting that configured means installed or ready. No new World schema, environment state machine, dependency resolver, isolation technology, or Lane cardinality is selected.

### C-OQ-024 — Harness, profile, and Workspace configuration

**Status:** Open\
**Decision:** Define how Collider exposes and edits Workspace configuration/policy, Agent Harnesses, Agent Profiles, and related workspace enablement through their existing owner boundaries.

**Must evaluate:** current Substrate and native harness configuration truth; harness identity and adaptation; native settings versus Substrate launch/integration/policy settings versus Collider presentation settings; declared, inherited, overridden, and effective values with source provenance; owner-approved edit and validation paths; reusable Agent Profiles versus native harness profiles and Handbook profiles; harness/provider/model/profile relationships; compatibility and supported controls; version and update status; environment requirements and C-OQ-023; credential bindings without general secret disclosure; Workspace-enabled skills, tools, apps, Extensions, providers, and external connections; availability/installation versus enablement versus authorization versus readiness; Capability Grant authority under C-OQ-014 and C-OQ-018; whether a change applies to future Invocations, active work, or after restart; Qualified Session compatibility and revalidation under C-OQ-008; and diagnostics and delivery scope. Compare requested UAA staging capabilities with Substrate's consumed dependency version, enabled features, and proven runtime-family paths; separate native harness support, negotiated behavior, authorization, readiness, and UI affordances rather than credit a candidate framework for replacing an existing CLI-agent layer; truthful separation among declared support, negotiated/runtime support, policy authorization, credential/environment readiness, and observed health when presenting harness/profile/capability configuration.

**Current constraint:** One configuration Surface does not create one configuration authority. Native harness, Substrate, Handbook, provider, and Collider state retain their existing owners. An Agent Profile is not an Agent Harness, Qualified Session, or permission grant. Navigator visibility is not capability enablement. No config schema, precedence order, provider fallback algorithm, native resume API, or execution topology is selected.

### C-OQ-025 — Navigator identity, customization, and persistence

**Status:** Open\
**Decision:** Define the supported Navigator Layout and persistence contract for user-arranged navigation over host-approved destinations and contributions.

**Must evaluate:** stable Navigator Item and Group identities independent of display labels and position; shortcuts and aliases; grouped and ungrouped items; group creation, naming, membership, and ordering; item visibility, hiding, recovery, and restore defaults; one-word highest-level shipped/default labels only, with multiword descendants and unrestricted user labels or user-promoted items; a root alias such as Home without renaming Control Plane; preferred Inbox naming; qualified Sessions as a real destination that may be user-prioritized; final default groups and nested versus root Harnesses, Environments, Profiles, and setup destinations; keyboard, focus, drag, narrow-window, and accessibility behavior; app/user/Workspace preference scope, cross-device synchronization, explicit shared defaults, and precedence; retention of personal choices without silently changing another user's layout or Substrate policy; contributed item admission, disappearance, reappearance, and migration under C-OQ-020 and C-OQ-021; unavailable targets; required execution awareness, cancellation, diagnostics, and recovery access; and delivery of basic customization within V1 separately from a general layout builder. Distinguish desired user layout from currently available contributions, user-hidden items from unavailable or failed providers, and functional host fallback from an error placeholder; evaluate framework navigation using stable host-owned destinations/actions without preserving provisional component code.

**Current constraint:** Items, groups, and layout are presentation, not semantic scope, permissions, or execution placement. Users must be able to rename groups, change displayed membership and order, and show items outside groups. Removing an item or group from navigation does not delete, disable, stop, rebind, or resume the underlying resources. The naming restriction does not extend to every tree level or user organization. No layout schema, persistence backend, synchronization mechanism, exact default tree, or mandatory immutable sidebar row is selected.

### C-OQ-026 — Inbox and notification semantics

**Status:** Open\
**Decision:** Define an Inbox destination for informational notifications and actionable owner-system items without creating a new authority for obligations or their resolution.

**Must evaluate:** source systems and existing Substrate inbox/notification semantics where present; questions, approvals, gates, blocked work, failures, integration and credential warnings; actionable versus informational classification; correlation and deduplication; links to Work Items, Lanes, execution, Qualified Sessions, and diagnostics; local read/dismiss state versus owner-authoritative acknowledgment, approval, or resolution; required policy checks before actions; suppression, snoozing, reminders, escalation, badges, grouping, and delivery channels as candidate behavior; unavailable source objects and stale items; permissions, private items, secrets, retention, deletion, and audit; interaction with Activity Projection without selecting Workspace history; recovery when the user hides Inbox; and first-slice versus later delivery. Separate notification read/dismiss state, request/approval lifecycle, and authoritative obligation resolution; test exact request correlation, stale responses, provider-loss fallback, and owner-approved action handling in any borrowed interaction UI; exact owner approval/obligation request identity, revision, actor, decision, stale/duplicate/conflicting/wrong-work handling, and explicit separation from local read/dismiss state.

**Current constraint:** Inbox is the preferred default label; Notifications may be a user label. Reading, dismissing, or hiding a notification does not implicitly approve a request, satisfy a Gate, unblock work, cancel execution, or change source authority. Any owner-system action uses the appropriate authorized boundary. No notification store, universal status model, delivery service, badge algorithm, or canonical Workspace history is selected.

### C-OQ-027 — Frontend foundation and reuse strategy

**Status:** Open\
**Decision:** Determine whether Collider should adopt, compose, adapt, selectively own, fork, or independently implement its frontend foundation, including presentation components and interaction controllers, over the existing Substrate/Handbook/UAA owner boundaries.

**Adjudication disposition:** **ADJUST — retain the shortlist; defer adoption.** The corrected report establishes a credible frontend-reuse opportunity, but documentation and source inspection do not establish integrated lifecycle truth, a supported Substrate GUI boundary, deployment compatibility, published-artifact provenance, or measured economics.

**Provisional proof shortlist:**

1. public AgentKit controller/protocol plus suitable AgentKit React regions, selective Toolkit, one owner-routed bridge, and Collider-defined Workstream/Control Plane/Lane behavior;
2. headless-first AgentKit when lifecycle mapping succeeds but the reference React composition does not;
3. substantial Toolkit reuse with another single controller when AgentKit lifecycle translation costs exceed demonstrated value;
4. a bounded coherent frontend fork when explicit controller/composition ownership is cheaper and clearer than a translation-heavy public adapter;
5. a broader owner-preserving framework/fork remains eligible but unselected pending retained-value, distribution, and maintenance evidence;
6. Collider-native implementation remains a fair comparator rather than the automatic fallback after any package mismatch.

**Must evaluate:** all previously listed adoption, composition, ejection, fork, native, and hybrid options against the same desired product; the actual public and private package seams; controller/protocol behavior; published artifacts and notices; Tauri/Vite/WebView deployment; and the retained value of one substantial agent-interaction region plus one non-chat region. In addition, prove the actual controller semantics for pending versus owner-accepted work, accepted/queued/running representation, lost acknowledgement, exact request/receipt identity, detach versus cancel, cancellation acknowledgement versus authoritative terminal effect, completion/cancel races, subscription interruption versus terminal completion, approval protocol segmentation and exact owner approval identity, automatic queue promotion and current authorization, repeated identical content with distinct identities, transcript versus reconstructed model context, capability-state honesty, source-cursor to projection-sequence mapping, coordinated Control Plane/Lane/inline/Sticky/Tab views under one behavioral owner, renderer fallback, and restart/replay behavior. Assess sidecars, private APIs, and fork size as deployment/economic tradeoffs unless they require forbidden authority or privilege changes.

**Current constraints:** AI Elements curation is disposable, with no preservation preference or sunk-cost penalty. Tauri + React + TypeScript + Vite is the target. Substrate and Handbook retain their existing authority; UAA/provider execution is not to be recreated in a parallel TypeScript backend. Framework code or owned source may implement Collider-defined roles; accepted semantics do not mandate bespoke implementations. AgentKit thread/run/controller identities do not become Workstream, Qualified Session, Invocation, Receipt, Source Record, Activity Item, or owner terminal truth by name. Default Agent-Native Core authority over database, auth, execution, persistence, or deployment is not selected. A sidecar or fork is not rejected merely for existing, but any authority-displacing design, false acceptance/terminal reporting, or UI-owned execution lifetime is a hard semantic failure. No Agent Native, AgentKit, Toolkit, BB, Cordis, controller, datastore, sidecar, fork, or dependency is adopted by this question.

**Required proof evidence:**

- **Contract gate:** exact supported owner entrypoint and production call path; versioned schemas; authority and policy checks; field-by-field command/receipt/work/cursor/approval/cancel/session mapping; unknown fields remain unknown.
- **Fixture/client proof:** the actual public AgentKit client/controller, not only a transport-shaped double; full relevant transport conformance plus Collider-specific lifecycle, identity, queue, approval, sequence, coordinated-view, and substantial non-chat Surface cases. Fixture success remains fixture evidence.
- **Owner-integrated proof:** the same scenario through the supported owner boundary and declared real execution path, preserving a traceable join from user intent through owner acceptance, source observations, projected events, and authoritative terminal/approval records; demonstrate view/window release and restart without implicit provider cancellation or duplicate dispatch.
- **Deployment/artifact/economics proof:** exact package artifacts, notices, provenance and applicable rights; Vite/Tauri/CSP/accessibility behavior; production dependency graph versus emitted assets; startup, memory, event-burst, replay, focus/keyboard, cleanup, maintenance, sidecar, source-divergence and upgrade costs.

**Decision boundary:** this adjudication selects a proof direction, not implementation. Only a separately authorized, passing exact-path proof plus applicable artifact/licensing evidence can support a later adoption or dependency decision. Research, adjudication, proof, and canonical amendment remain separate steps.

## 4. Research Clusters and Preliminary Dependencies

The following grouping is non-normative research-ordering metadata. It does not resolve an item, create implementation authority, or claim that every dependency is complete. Per-question `Depends on` and `Blocks` fields SHOULD be added only when a bounded research packet establishes them with sufficient confidence.

| Cluster | Questions | Preliminary dependency or gating relationship |
|---|---|---|
| **History and projection** | `C-OQ-001`–`C-OQ-007`, `C-OQ-022` | Establishes source, identity, ordering, assembly, replay, and reconstruction boundaries needed by later Workstream persistence and history-sensitive UX contracts. |
| **Qualified Session continuity** | `C-OQ-008` | Depends partly on owner-history and Activity Projection findings and coordinates destination behavior with `C-OQ-025`; it MUST NOT resolve Workspace history or make navigation imply continuation. |
| **Presentation placements** | `C-OQ-009–010` | Can proceed against stable Surface and Activity Item identities; persistence and restoration details may depend on history/projection findings. |
| **Execution topology and context lineage** | `C-OQ-011–012` | Requires current Substrate implementation truth and later gates the detailed Execution and Binding contract. |
| **Reference-architecture evaluation** | `C-OQ-013` | May inform experiments, but does not gate canonical Collider terminology or select a dependency. |
| **Extension trust, effects, and lifecycle** | `C-OQ-014–017` | Trust and isolation boundaries constrain every later dynamic-extension mechanism; recovery criteria gate truthful composability claims. |
| **Capability composition** | `C-OQ-018–019` | Depends on owner and trust boundaries from `C-OQ-014` and dependency-lifecycle findings from `C-OQ-017`. |
| **UI extension composition** | `C-OQ-020–021` | Depends on trust and capability constraints from `C-OQ-014`, `C-OQ-018`, and `C-OQ-019`; it does not replace Surface or placement semantics. |
| **Environment realization** | `C-OQ-023` | Starts from existing Substrate World dependency tooling; coordinates Lane/environment changes with the execution contract rather than creating a Collider resolver. |
| **Workspace execution setup** | `C-OQ-024` | Requires owner-grounded configuration truth and coordinates environment, capability, policy, and Session behavior with `C-OQ-023`, `C-OQ-014`, `C-OQ-018`, and `C-OQ-008`. |
| **Navigator customization** | `C-OQ-025` | Builds on Presentation Actions and UI-contribution boundaries; informs the Workspace Shell contract without requiring every destination editor first. |
| **Inbox and notifications** | `C-OQ-026` | Depends on the source owners' acknowledgment and resolution semantics; coordinates discoverability with `C-OQ-025` without selecting a universal history. |
| **Frontend foundation and reuse** | `C-OQ-027` | Research and adjudication are complete enough to shortlist proof candidates, but adoption remains Open. The next gate is the non-authorizing owner-boundary/proof specification and separately authorized staged proof; coordinates with history/projection, continuity, execution, capability, UI, setup, and Inbox questions without requiring them all to close first. |

## 5. Recommended First Research Packet

The first research packet should address **C-OQ-001 through C-OQ-007 together with C-OQ-022**, because they share one architectural boundary.

It should inspect current implementation truth for:

- Substrate invocation, JSONL, Receipt, process, tool, Lane, and execution records;
- Handbook Artifact history, Projection, Snapshot Memory, deltas, Evidence, and SDK boundaries;
- MCP App object and lifecycle semantics used or contemplated by Collider;
- shell and PTY capture already available in Substrate;
- Collider and Tauri persistence, restoration, and Surface-host constraints.

The packet should return:

1. a source-of-truth inventory;
2. required user-visible queries, Projection Definitions, and Activity Item identity requirements;
3. payload-copy versus reference rules;
4. candidate Activity Assembler and Projection Index placements with failure modes;
5. append, prepend, pagination, replay, resynchronization, gap-repair, snapshot, and compaction requirements;
6. candidate history architectures with failure modes;
7. a recommended option or a reason to preserve a bounded hybrid;
8. migration and incremental-delivery implications;
9. the exact glossary, foundations, workspace-shell, and execution-contract edits required by the decision.

### 5.1 Current frontend-evaluation sequence

The corrected Agent Native evaluation and bounded adjudication are complete as advisory evidence. `C-OQ-027` remains Open.

1. Preserve the corrected report, adjudication decision coverage, adjudication check record, exact pins, and missing-original-ledger limitation as research provenance. Do not reconstruct the missing ledger or promote the new check record to that status.
2. Use `agentkit-owner-boundary-proof-spec.md` to ground the exact supported Substrate/Handbook/UAA owner surface and field-by-field mapping. Contract readiness is not runtime acceptance evidence.
3. After separate authorization, run the staged proof: actual-client fixture proof, owner-integrated proof, and deployment/artifact/economics proof. Keep their results separate.
4. Draft stable implementation-neutral Workspace Shell behavior in parallel where sufficiently settled. Do not silently choose controller identity, history storage, or execution truth while doing so.
5. Use the owner inventory and exact-path evidence to complete supported Execution and Binding contracts. Where a real owner path is unavailable, record the capability as unavailable or unproved rather than creating a candidate-shaped workaround.
6. After the proof, explicitly decide dependency/composition/source-ownership/fork/native strategy. Amend glossary or Foundations only when an accepted product-semantic decision actually changes.

Deep Research and adjudication are evidence processes, not additional authority or permission grants. Fixture success, inspected tests, source declarations, and package metadata do not become production evidence through repetition.

## 6. Closure Rule

An open item is not closed by implementation convenience, a UI mockup, or an undocumented local choice.

Closure requires an explicit decision record and coordinated updates to every normative Collider V1 document whose definitions or invariants change.



# SOURCE: Collider/docs/agent-native/proof/agentkit-owner-boundary-proof-spec.md

Source SHA-256: c6bc06ff99d3d8aa3e397c77f856b2ad2ff89c37d77e6897b86099a16c0b278c

# Collider AgentKit Owner-Boundary and Proof Specification

**Status:** Proposed non-normative proof contract; no implementation authorization  
**Purpose:** Define the evidence boundary and staged proof required before any AgentKit/Toolkit dependency, source-ownership, fork, sidecar, or native-controller adoption decision  
**Canonical dependencies:** [`../canonical/00-collider-v1-glossary.md`](../canonical/00-collider-v1-glossary.md), [`../canonical/01-collider-v1-foundations.md`](../canonical/01-collider-v1-foundations.md), [`../canonical/02-collider-v1-open-architecture-questions.md`](../canonical/02-collider-v1-open-architecture-questions.md)  
**Research basis:** [`../research/corrected-agent-native-evaluation.md`](../research/corrected-agent-native-evaluation.md)  
**Adjudication basis:** [`../adjudication/adjudication-result.md`](../adjudication/adjudication-result.md), [`../adjudication/adjudication-decision-coverage.md`](../adjudication/adjudication-decision-coverage.md), [`../adjudication/adjudication-check-record.md`](../adjudication/adjudication-check-record.md)  
**Implementation authorization:** None. This document defines proof requirements only.

## 1. Purpose and decision boundary

The corrected Agent Native research and subsequent adjudication justify a **preferred proof candidate**, not an adoption decision.

The first candidate to prove is:

> **Public AgentKit controller/protocol plus suitable AgentKit React regions, selective Toolkit, one owner-routed bridge, and Collider-defined Workstream/Control Plane/Lane behavior.**

The proof MUST preserve the possibility of:

- a headless-first AgentKit path;
- Toolkit with another single behavioral controller;
- selective source ownership/ejection;
- a bounded coherent frontend fork;
- a broader owner-preserving framework/fork where retained product behavior justifies it;
- a Collider-native implementation.

A failed public-package mapping MUST NOT automatically choose bespoke implementation. A successful fixture MUST NOT automatically choose AgentKit.

Only a separately authorized, passing **owner-integrated exact-path proof** plus applicable deployment/artifact/licensing evidence may support a later dependency or architecture decision.

## 2. Controlling authority model

This proof MUST preserve the current owner boundaries.

| Concern | Authority / role during proof |
|---|---|
| Handbook semantic state, Scope, Resolution, Contracts, Evidence, verdicts and gates | **Handbook** |
| Workspace policy, Lanes, execution environments, accepted work, execution lifecycle, Receipts, supervision, obligations, runtime context delivery | **Substrate** |
| Provider-runtime integration consumed by Substrate | **Unified Agent API through the actual Substrate-owned path** |
| Workstream presentation, Activity Projection, Blocks, Surfaces, placements, Navigator and Presentation Actions | **Collider** |
| Candidate live interaction/controller state | **AgentKit only as a derived client/projection role if proven truthful** |
| Candidate presentation/editor/grid behavior | **Toolkit only as presentation/interaction implementation if proven suitable** |

No proof result may transfer owner authority merely because a library stores client state, exposes a thread/run identifier, renders an approval, or provides a controller.

## 3. Non-equivalences required by the proof

The proof MUST keep these identities distinct unless a later explicit decision establishes a narrower mapping:

```text
AgentKit thread
    ≠ Collider Workstream by definition
    ≠ Lane
    ≠ provider Qualified Session
    ≠ Substrate durable orchestration/session authority

AgentKit run
    ≠ accepted Invocation receipt by definition
    ≠ owner operation by definition
    ≠ authoritative terminal record

AgentKit message list
    ≠ Handbook Resolution
    ≠ Substrate Context Pack
    ≠ model-context authority

AgentKit task/activity/widget
    ≠ Work Item
    ≠ Obligation
    ≠ canonical Source Record
    ≠ admitted Evidence

AgentKit sequence/checkpoint
    ≠ universal Workspace order
    ≠ owner cursor unless explicitly mapped
```

A mapping MAY associate these identities. It MUST NOT collapse them merely for adapter convenience.

## 4. Proof stages and evidence classes

Evidence MUST be reported in four separate stages.

| Stage | Required activity | May establish | MUST NOT be represented as |
|---|---|---|---|
| **A — Contract gate** | Identify supported owner entrypoints, call chain, versioned schema, policy/authority checks, field-by-field mappings and readiness | A testable proposed owner contract | Runtime acceptance or production proof |
| **B — Fixture/client proof** | Exercise the actual public AgentKit client/controller against controlled fixtures, plus substantial Toolkit UI | Candidate expressiveness, controller behavior, protocol/client sequencing | Real Receipts, real provider survival, real owner authorization or production replay |
| **C — Owner-integrated proof** | Repeat the same product scenarios through the supported owner boundary and real declared execution/mutation path | Exact-path owner behavior for the tested features | Untested providers/platforms or general production readiness |
| **D — Deployment/artifact/economics proof** | Inspect exact artifacts/notices/provenance and measure target desktop behavior/cost | Distribution inputs, target deployment behavior and measured option tradeoffs | Canonical adoption without a separate decision |

A fixture served through a real endpoint remains **fixture evidence**.

## 5. Gate A — supported owner-surface inventory

Before any result may be called owner-integrated, produce an `owner-surface-inventory.md` that identifies the exact supported path for every exercised operation.

For each operation record:

1. public/supported entrypoint;
2. implementation owner;
3. production call path used by the proof;
4. version or commit/release identity;
5. request schema and response/event schema;
6. policy, authentication and authorization checks;
7. durable identity created, if any;
8. observation/replay interface;
9. terminal or closeout authority;
10. cancellation semantics;
11. retry/idempotency semantics;
12. capability/readiness conditions;
13. current implementation/proof status;
14. unsupported or unavailable behavior.

At minimum inventory these feature families where the chosen proof exercises them:

- command/request submission;
- accepted-work reconciliation / Receipt identity;
- durable observation and replay;
- cancellation request and authoritative terminal effect;
- approval/interactive request lifecycle;
- capability discovery/readiness;
- Qualified Session continuation where used;
- non-chat owner-backed read and mutation;
- stale-revision handling/readback;
- connection/restart behavior.

### 5.1 Forbidden shortcuts

The owner-integrated proof MUST NOT treat any of the following as a supported owner API merely because it exists:

- crate-private Substrate modules or stores;
- hidden development helpers;
- direct UI ownership of UAA run handles or event streams;
- architecture ownership tables without a callable path;
- serialized Rust types without a supported cross-process contract;
- fixture endpoints that mimic owner behavior.

If a required owner capability is unavailable, report that feature **unavailable or unproved**. Do not create a candidate-shaped authority workaround.

### 5.2 UAA correspondence requirement

The proof MUST identify whether the exercised path uses:

- the exact UAA release consumed by Substrate;
- staging source behavior;
- another pinned build.

Do not claim staging/release equivalence without evidence.

## 6. Field-by-field identity and lifecycle map

Before Gate C, produce a machine-readable mapping ledger and human-readable table for the exercised scenario.

The map MUST include, where applicable:

| Proof concept | Required mapping question |
|---|---|
| User command / intent identity | What stable identity exists before owner acceptance? |
| AgentKit thread ID | What client/projection scope does it identify? |
| AgentKit run ID / protocol segment | What exact owner work or continuation segment does it reference? |
| Substrate Invocation identity | When and by whom is it accepted? |
| Accepted-work Receipt | What proves owner acceptance and exact correlation? |
| Owner operation/execution identity | What actually runs or remains queued? |
| Qualified Session identity | Which provider/runtime continuity resource, if any, is explicitly resumed? |
| Approval/request identity | Which exact owner request, revision, actor and work item may be answered? |
| Source observation cursor | How is source completeness/replay represented? |
| AgentKit sequence/checkpoint | How does it relate to owner cursor and projection sequence? |
| Activity Item identity | How is stable Business Identity derived and versioned? |
| Surface identity / placement | Which presentation identity is shared across inline/Sticky/Tab? |
| Capability status | What is declared, owner-supported, authorized, ready and observed? |

Unknown fields MUST remain unknown. Text equality, list position, client-generated IDs, or local timestamps MUST NOT fill an owner identity gap.

## 7. Lifecycle truth requirements

### 7.1 Submission and acceptance

The proof MUST distinguish:

```text
pending local intent
    → owner request transmitted
    → owner accepted work / Receipt
    → accepted-but-queued or otherwise admitted
    → actually running where separately observable
    → terminal owner outcome
```

An optimistic AgentKit message or generated AgentKit run ID MUST NOT be presented as owner acceptance.

If `startRun()` can only return after owner acceptance, prove that exact relationship. If it cannot, the selected controller/composition MUST represent pre-acceptance state without lying.

Lost acknowledgement MUST NOT cause implicit resubmission. Repeated identical prompts with different request identities MUST remain distinguishable.

### 7.2 View detachment

Closing a Tab, unmounting React, changing Workstreams, releasing an AgentKit lease, or terminating the Tauri window MUST NOT implicitly cancel owner work.

Substrate MUST retain ownership of provider/UAA execution resources beyond every UI lifetime used in the proof.

### 7.3 Cancellation

The proof MUST distinguish:

```text
cancel requested
    ≠ cancel request accepted
    ≠ operation actually cancelled
    ≠ operation completed before cancellation took effect
```

Because the inspected AgentKit client may mark a run cancelled after `cancelRun()` resolves, the selected mapping MUST ensure the controller cannot report cancellation before the correct owner terminal outcome is known—or explicitly own a bounded controller change that fixes that behavior.

A completion-versus-cancel race is mandatory.

### 7.4 Completion

`AgentRunHandle.completed`, subscription end, local completion timestamps, stream teardown, approval interruption, or view teardown MUST NOT be treated as authoritative owner completion.

The proof MUST identify the exact owner terminal source and map it to the UI.

### 7.5 Approval and interactive continuation

Every approval/interactive decision MUST bind to:

- exact owner request identity;
- relevant revision/version;
- exact owner work;
- current actor/authorization context;
- explicit approve/deny/other supported decision;
- current policy and capability state.

Test stale, expired, duplicate, conflicting, wrong-actor and wrong-work responses.

If AgentKit v2 uses distinct replacement protocol runs after terminal interrupts, map protocol segments to owner operations explicitly. Protocol segmentation MUST NOT create false canonical Invocations, Receipts or Qualified Sessions.

Inbox dismissal/read state remains unrelated to owner authorization.

### 7.6 Queue behavior

If AgentKit queue machinery is enabled, queue promotion MUST remain an authorized owner operation under current bindings and policy.

The frontend queue MUST NOT become an independent durable execution scheduler.

Unsupported queue semantics MUST be disabled rather than approximated.

### 7.7 Context reconstruction

AgentKit message arrays, transcript snapshots, or client thread state MAY be UI/projection inputs. They MUST NOT become the model-context authority.

The owner path MUST reconstruct current context from the applicable Handbook/Substrate sources, selected references, current Scope, Lane, policy, Capability Grants and explicit Qualified Session continuation.

### 7.8 Capability honesty

The proof MUST preserve distinctions among:

- unknown;
- unsupported;
- unavailable;
- degraded;
- available;
- authorized;
- ready.

Library feature presence alone does not establish owner support or authorization.

## 8. Projection, replay and coordinated-view requirements

### 8.1 One behavioral owner for the same work

The same accepted work may appear in:

- Control Plane;
- one or more Lane-filtered views;
- inline Blocks;
- Sticky placements;
- Tabs.

Those placements MUST NOT create independent command, queue, approval or optimistic-state owners for the same work.

This does not require one global controller or one global datastore for unrelated work.

### 8.2 Control Plane and Lane proof

The fixture and owner-integrated proofs MUST show one accepted work item coherently in both:

- Lane-unfiltered Control Plane projection;
- appropriate Lane-filtered projection.

The proof MUST preserve the possibility that the views share Source Records while having different Activity Item identity, ordering, visibility or location.

### 8.3 Sequence and filtering

If AgentKit requires contiguous per-run sequences, view filtering MUST occur after complete source admission, or an explicit projection-sequence remap MUST be defined.

Removing Lane-ineligible source events while retaining original sequence numbers MUST NOT manufacture a false gap.

The mapping MUST identify:

- owner source cursor;
- completeness boundary;
- projection sequence;
- projection version;
- snapshot/read cut;
- retention limits;
- gap detection and repair behavior.

### 8.4 Reconnect and restart

The proof MUST distinguish:

```text
resubscribe / replay same work
    ≠ retry original request
    ≠ new owner dispatch
```

Test view reopen, frontend restart where supported, and process restart where resumability is claimed. No implicit provider redispatch is permitted.

## 9. Gate B — actual-client fixture proof

The fixture proof MUST use the **actual public AgentKit client/controller**, not only an object shaped like `AgentTransport`.

Where relevant, run the upstream conformance factory/profiles and add Collider-specific cases. Inspected upstream tests are not preexisting passing evidence for this project.

### 9.1 Mandatory fixture cases

| Area | Mandatory demonstration |
|---|---|
| Acceptance | pending vs accepted/queued/running; lost acknowledgement; no implicit retry; identical prompt content with different IDs |
| Lifecycle | detach vs cancel; cancellation acknowledgement vs terminal effect; completion/cancel race; interruption/teardown vs `completed` promise |
| Approval | approve/deny; stale/expired/duplicate/conflicting/wrong-actor/wrong-work; exact continuation identities |
| Queue | explicit current owner authorization for promotion; unsupported capability disabled |
| Replay | duplicate, gap, reorder and replay; cursor/sequence mapping; snapshot cut; retention limits |
| Capabilities | unknown vs unsupported vs unavailable vs degraded vs available; owner support and authorization separate |
| Shared product experience | same work in Control Plane and Lane; inline/Sticky/Tab; one behavioral owner; unified Composer with distinct routing |
| Missing UI/provider | stale reference, missing renderer, provider unavailable and fallback behavior |
| Non-chat Surface | substantial Toolkit grid/editor/configuration interaction producing validated owner-operation intents rather than direct authority |

### 9.2 Fixture evidence limit

A passing fixture MAY establish:

- candidate expressiveness;
- controller sequencing under controlled stimuli;
- UI composition feasibility;
- whether public APIs are sufficient for the tested scenario.

It MUST NOT be represented as proof of:

- Substrate Receipt semantics;
- production authorization;
- provider survival;
- durable restart;
- real approval continuation;
- production replay;
- shipping security.

## 10. Gate C — owner-integrated proof

Repeat the same core scenarios through the supported owner boundary and real declared execution/mutation path.

Capture a traceable, non-secret join:

```text
user intent
  → owner request
  → owner acceptance / Receipt
  → exact owner work
  → source observations
  → projected events / Activity Items
  → authoritative terminal or approval records
```

The owner-integrated proof MUST demonstrate:

- view/window release does not cancel provider work;
- reopening does not dispatch duplicate work;
- explicit cancellation targets the exact work and handles completion races;
- replay/reconnect uses owner truth and does not implicitly retry;
- approval continuation validates current owner state and authorization;
- current capability/readiness state is represented truthfully;
- a substantial non-chat owner-backed read and authorized edit, including stale-revision rejection and readback;
- any advertised Qualified Session continuation revalidates current Scope/Lane/policy/capability/model conditions.

If the selected owner path does not support a feature, record it as **unsupported/unavailable/unproved**. Do not silently simulate it in the owner-integrated result.

## 11. Gate D — deployment, artifact, licensing and economics proof

Before a shipping/adoption decision, inspect the exact artifacts and distribution mode actually proposed.

### 11.1 Artifact and rights evidence

For dependency, source-copy/ejection and fork modes separately, record:

- exact package/source revision;
- registry tarball or distributed artifact hash where applicable;
- included license/notice files;
- source-to-artifact correspondence where required;
- retained copyright notices;
- provenance/publish metadata;
- applicable copying/ejection/fork rights and unresolved legal questions.

Package manifest license fields alone are insufficient to close this gate.

### 11.2 Desktop/security validation

Test the actual target stack:

- Vite build;
- Tauri/WebView on supported platforms selected for the proof;
- deliberate CSP rather than relying on the current null policy;
- IPC permissions;
- navigation/external URLs;
- Markdown/rich content;
- uploads/files;
- clipboard;
- keyboard/focus;
- accessibility;
- renderer failure containment;
- subscription/resource cleanup.

### 11.3 Measurements

Establish a Collider baseline **before** comparing options. Measure at least:

- production dependency graph;
- emitted JavaScript/CSS separately from installed dependency graph;
- cold WebView startup;
- steady-state and burst memory for the tested region;
- event-burst update behavior;
- populated-history replay/reconnect behavior;
- cleanup/leak behavior;
- source divergence and local patch surface;
- expected upgrade/security-backport workflow.

Do not invent target numbers before baselines exist.

### 11.4 Cost attribution

Charge common work to every applicable strategy, including:

- Next-to-Vite transition;
- required supported Substrate frontend boundary;
- settled Collider shell semantics.

Charge candidate-specific work only where introduced, including:

- AgentKit protocol translation;
- library-only sidecar;
- private-API workaround;
- fork/source divergence;
- candidate-specific upgrade and security backport burden.

## 12. Stop, reversal and alternative-selection rules

### 12.1 Hard semantic stops for a selected mapping

Reject or stop the **selected mapping/design** when it requires:

- a second durable execution owner;
- UI/subscriber lifetime to own execution lifetime;
- false accepted-work or terminal reporting;
- a forced Session-centric Workstream semantic change;
- bypass of Substrate/Handbook authority, policy or authorization;
- forbidden privilege expansion to make the library work.

These findings do not automatically reject every other AgentKit/Toolkit/fork/native variant.

### 12.2 Public-dependency stop / fork assessment

If core scenarios require unsupported/private APIs, stop the unsupported public-dependency route and compare:

- bounded source ownership;
- headless-only reuse;
- presentation-only reuse;
- bounded coherent fork;
- native alternative.

### 12.3 Economic gates

A Node sidecar, sizable frontend fork, or pinned local source tree is an economic/deployment gate rather than an automatic architectural veto unless it also violates authority or security constraints.

Select only after comparing retained behavior against packaging, startup, memory, patch, backport, update and eventual replacement costs.

### 12.4 Distribution hold

If adequate rights/notices/provenance cannot be established for the selected adoption mode, hold distribution of that mode. Do not infer either permission or prohibition solely from repository/package metadata.

## 13. Required proof outputs

A separately authorized proof packet SHOULD return these artifacts:

1. `owner-surface-inventory.md` — supported owner entrypoints and readiness;
2. `identity-lifecycle-map.md` — field-by-field human-readable mapping;
3. `identity-lifecycle-map.json` — machine-readable mappings and unresolved fields;
4. `fixture-client-proof-results.md` — actual-client fixture results with exact versions;
5. `owner-integrated-proof-results.md` — exact owner path, traces and unsupported features;
6. `deployment-artifact-economics-results.md` — artifacts, notices, desktop/security/performance and option costs;
7. `proof-evidence-ledger.json` — hashes, pins, commands/tests actually executed, source refs and result classification;
8. `adoption-decision-input.md` — evidence-backed comparison of public composition, headless/presentation variants, source ownership/fork and native alternative.

Every result MUST state whether it is:

- inspected source/contract evidence;
- fixture evidence;
- owner-integrated evidence;
- deployment/artifact evidence;
- inference/recommendation.

## 14. Document implications

This proof specification does not amend the glossary or Foundations.

The updated open register keeps `C-OQ-027` Open and records the shortlist and evidence gates. Related open questions remain unresolved unless a later decision explicitly closes them.

Stable Workspace Shell behavior may continue to be drafted without selecting AgentKit or another framework. Execution and Binding must consume real owner contracts and the owner inventory rather than invent candidate-shaped semantics.

## 15. Final proof decision boundary

A passing proof does not itself adopt AgentKit, Toolkit, a fork, sidecar, or native alternative.

After the staged evidence exists, a separate decision must select one strategy or explicitly preserve more than one path. Only that decision may authorize dependency/fork adoption and any necessary coordinated canonical amendments.



# SOURCE: Collider/docs/agent-native/controller-correction/implementation-record.md

Source SHA-256: b5adc1a4a89120261010adee95516eedc65c13a2384ea151951cb932f0f78190

# Packet A implementation record — resumed correction

## Current result and authority

Packet A resumed on the **existing** `spike/agentkit-controller-correction` at
`de2882ee120b0e89d3d3b25b53b18ffea287144e`. No replacement branch was created.
The user's resumed cancellation contract closes the earlier three-part blocker;
the original stop is retained below as historical evidence, not the
current contract. In particular, accepted/refused/failed replace the old plan's
acknowledged/unknown labels for the controller-local request projection.

Authority remains the [source ownership plan](../controller-ownership-plan/controller-source-ownership-plan.md),
[lifecycle contract](../controller-ownership-plan/controller-owner-lifecycle-contract.md),
and [regression plan](../controller-ownership-plan/implementation-and-regression-plan.md),
as narrowed/clarified by the user's Packet A resumption. Nothing here authorizes B/C.

## Implemented surface and lifecycle

`UPSTREAM_SOURCE_OR_CONTRACT`: exact-version maintained pnpm patch, AgentKit
**0.2.3**, revision **collider-agentkit-0.2.3-a1**, profile
**collider-owner-lifecycle/1**. Toolkit **0.20.3** unchanged. All support lives in
[new isolated proof](../../../proofs/agentkit-controller-correction/README.md).
The original Gate B proof, its dependencies, tests and historical results are not
rewritten. [Provenance](patch-provenance.json) binds preimages, postimages, full
installed inventories, lock, patch, staged clean-install inputs and original stop.

Five emitted payloads remain necessary, not an artificial file-count target:

| Payload                       | Reason                                                                                                |
| ----------------------------- | ----------------------------------------------------------------------------------------------------- |
| `dist/client/client.js`       | Allocation is unobserved; cancel operation normalization/projection/generation; snapshot merge guards |
| `dist/client/state.js`        | Unobserved default, observed-running timestamp, duplicate/conflicting/postterminal admission          |
| `dist/protocol/validation.js` | Admit only the approved additional run statuses, including run-status events and snapshots            |
| `dist/protocol/index.d.ts`    | Extend run status and export controller-local cancellation request types                              |
| `dist/client/state.d.ts`      | Expose optional controller projection on client run state                                             |

The validator change is **not** for `cancellation_refused`: existing open-string
AgentError.code and parseAgentError suffice. No new refusal event, sequence slot,
owner receipt, durable owner field or protocol error taxonomy was added.
The validator is still required for `accepted`/`unobserved`; narrowing away this
file would leave declared status values rejected at runtime.

Cancellation Promise<void> success means accepted only. The existing operation
error boundary normalizes/validates structured AgentError before the controller
classifies exactly `cancellation_refused` with `retryable: false`; malformed refusal
shapes and all other operation failures project failed, not refusal. Free-text
matching, classes and HTTP statuses do not classify refusal. Plain normalized
AgentError messages/details are retained. Unsupported capability and abortion
remain their existing operation errors.

`cancelRequest` has generation/status/requestedAt/optional settledAt,
correlationId/error. Error is absent for requesting/accepted and present for
refused/failed. Settled requests have a local settlement time. The first actual
transport dispatch allocates generation 1; explicit failed/refused retries advance
one. An in-flight private promise map joins repeated calls, accepted requests do
not redispatch, and terminal-before-dispatch does nothing. Callback-time abort and
terminal checks occur before generation allocation. Settlement captures and checks
the generation and changes only this local projection. Old settlement cannot
replace a newer generation, observed status, terminal evidence or consumer.

Only admitted owner/protocol observations establish running/terminal evidence.
ACK, refusal, timeout/disconnect and cancellation operation settlement do not stop
observation. Accepted-to-cancelled/completed/failed and terminal-before-ACK races
are tested. No authoritative completion timestamp is manufactured by cancellation.
`completed` remains the observer/segment settlement promise, **not owner success**;
interrupt/detach versus work terminality remain distinct.

Snapshot merge preserves the controller-local request projection, including a
request already settled at the baseline. Old source cuts cannot regress a live
terminal; equal-cut contradictions fail closed. Exact event duplicates are
idempotent, conflicting duplicates and postterminal events reject. Local cancel
state is intentionally not restored into a new controller/process.

## Consumers, adapters and compatibility

This is a **bounded local status/protocol extension plus behavior correction**,
not ordinary unmodified v2 interchangeability. The proof constructor requires the
named profile before dispatch. Snapshot mapping version/read cut are validated.
Old valid run states and new states round-trip, invalid run/task/tool statuses
still reject. Public root/protocol/conformance/React headless/context entrypoints
resolve to the same patched package; no replacement controller or runtime shim.

The fixture transport emits explicit accepted evidence, not a run-ID inference.
Lost-ACK reconciliation captures exact original request/message identity before
awaiting acceptance. Approval continuation no longer fabricates run.started; the
new segment is unobserved until actual evidence. Enabled synthetic approvals and
queues stay separate from the selected disabled owner profile. Invocation remains
current intent plus explicit context, never implicit transcript replay.

The actual injected AgentKitProvider/useAgentRun path drives CP/Lane/tab views from
one same-work client. Views show cancel request separately from terminal status;
Sticky is view-local, close/reopen does not dispatch or cancel, and source sequence
admission precedes filtering. Toolkit DataGrid remains the original package.
Stock full UI and unselected package surfaces are not certified by these tests.

## Patch mechanics, source maps and maintenance

Used pnpm 10.11.1 `patch`/`patch-commit` with a temporary edit directory, never
manual edits to the shared store or a final installed node_modules. Exact-version
registration and frozen lock are isolated in the new proof. Type checking adds
only TypeScript 5.9.3 as a new resolution; direct React type pins already existed
transitively at exactly 19.3.0. No previous resolved package changed or disappeared.

The plan-approved fallback removes all five touched maps and their corresponding
sourceMappingURL comments. Other maps and notices are untouched. This sacrifices
source-level mapping for those emitted files; it does not invent owned TS sources
or advertise stale mappings. The verifier checks this on every validation run.

A clean directory exported all **40 staged proof files** through `git show :path`,
then ran frozen install, full artifact verification, 80 tests, two invariants,
strict types and production build. No installed bytes were carried over. Missing
patch and deliberately altered isolated installed bytes failed closed; restoring
them restored validation. Lock SHA-256 was unchanged. See [results](regression-results.md).

Reported publish-source metadata and selected source comparison remain distinct
from signature verification or source-to-package reproducibility. Historical
MIT/ISC/standalone-notice uncertainty remains unresolved; no distribution clearance
is claimed. Package upgrade/removal requires a fresh exact-artifact decision and
the same corrected-invariant/regression checks. No production adoption is implied.

## Historical pre-clarification stop — preserved history

<details>
<summary>Earlier mandatory stop (superseded only by the resumed Packet A contract)</summary>

# Packet A implementation record — stopped before patch

## Bound scope and live preflight

The current request authorizes Packet A implementation, not another strategy choice.
Its detailed authority is the committed [source ownership plan](../controller-ownership-plan/controller-source-ownership-plan.md),
[lifecycle contract](../controller-ownership-plan/controller-owner-lifecycle-contract.md),
and [regression plan](../controller-ownership-plan/implementation-and-regression-plan.md).
The selected route remains an exact-version pnpm patch; no vendoring alternative
was substituted. [Provenance](patch-provenance.json) records the initial checkout,
refs, artifact digests and all five approved preimages.

- Verified repository: `<HOME>/__Active_Code/atomize-hq/collider`.
- Initial branch: `docs/agentkit-bounded-controller-ownership-plan`.
- Recorded BASE_COMMIT: `de2882ee120b0e89d3d3b25b53b18ffea287144e`.
- Created `spike/agentkit-controller-correction` directly from BASE_COMMIT and
  verified initial HEAD equality. All four required commits are ancestors of base.
  There are no intervening planning-tip changes.
- Initial tracked tree/index clean; sole untracked file was the unrelated ZIP.
- pnpm 10.11.1 / Node v25.9.0. Root and isolated Gate B manifests/locks unchanged.
- Reused retained exact AgentKit 0.2.3 / Toolkit 0.20.3 archives; SHA-512 matches
  the committed lock. All 88 / 1,265 installed files respectively equal archives.
  Relevant planning-ledger A/F/X files were rehashed and matched. No acquisition,
  installation, package-store mutation, or dependency resolution refresh occurred.
- Public ESM root resolves to the original Gate B AgentKit instance. The red control
  uses that actual controller, not a copied reducer or replacement implementation.

The approved five-file payload remains `dist/client/client.js`,
`dist/client/state.js`, `dist/protocol/validation.js`, `dist/protocol/index.d.ts`,
and `dist/client/state.d.ts`. Map artifacts are listed separately in provenance.
If resumed, allowed support paths are the plan's new isolated
`proofs/agentkit-controller-correction/` proof/test/package/patch arrangement and
this evidence directory. Historical Gate B and production files remain untouched.

## Exact stop: missing cancellation classifier

`UPSTREAM_SOURCE_OR_CONTRACT`: live `dist/protocol/index.d.ts:853–864` exposes
`AgentTransport.cancelRun(...): Promise<void>`. `AgentError.code` at 394–400 is an
unconstrained string; `dist/protocol/errors.js:10–24` carries codes without defining
a cancellation-refusal taxonomy. The existing fixture throws generic errors for
refusal and unknown acknowledgement (`../gate-b` proof's `fixture-owner.mjs:141–150`).

The lifecycle contract at 85–93 and 101–103 requires **classified refusal → refused**
and **unclassified rejection/timeout/disconnect → unknown**. At 213–218 it requires
local operation generation for in-flight updates. The source ownership plan at
127–142 names optional `cancelRequest` but does not define its fields, refusal
discriminator, or trusted validation/classification boundary.

A textual error-message classifier would invent semantics. Treating every failure
as unknown would omit the required refusal behavior. A new named structured error
code could fit the existing generic error channel and five files, but selecting
its meaning/trust boundary is the missing contract decision, **not proof that a
broader upstream surface is inherently necessary**. No new semantics were selected.

Independent pre-edit reviewer `/root/packet_a_preflight_review`, Terra/high,
confirmed this P1 blocker. The earlier plan's CLEAN readiness verdict is retained
unchanged; this implementation preflight records the newly identified gap.

The reviewer also identified an authorized implementation requirement, not a
separate scope blocker: `fixture-transport.mjs:128–156` synthesizes `run.started`
from an approval-answer while creating a continuation. A resumed correction must
replace that inference and test unobserved-until-evidence. Profile gating can stay
in the proof integration constructor and must reject ordinary v2 before effects.

## Actual changes, compatibility and maintenance

No AgentKit runtime, declaration, validator, reducer, consumer, adapter, or Toolkit
implementation changed. New files are four evidence records and the two-test
[negative control](../../../proofs/agentkit-controller-correction/tests/unpatched-control.test.mjs).
There is no patch file, patch registration, corrected installation, or post-patch
hash. Results are [reported separately](regression-results.md).

The approved design is a **bounded local protocol extension plus behavior change**,
not behavior-only: `unobserved`/`accepted`, `collider-owner-lifecycle/1` validation,
and local cancellation state. None is implemented here. `completed` remains the
unmodified consumer-settlement promise. Existing snapshot/type compatibility and
patched React behavior have not been established. No second controller was added.

All source maps remain upstream-original. On resumption, use the approved
remove-touched-maps-and-references option unless reproducible regeneration is
established; never advertise stale mappings or nonexistent owned TS sources.

Retained SLSA metadata reports publish source `9a562b4255aee88e16bef3e6b3e481b8a308c0ae`;
it is not a Git blob, file SHA, signature verification or reproducible build.
Selected controller/DataGrid source correspondence does not establish whole-package
source equivalence with adjudication pin `c884b5cbe616fe8674586d23c7345d9661de81f5`.
MIT package metadata and absent standalone notices do not settle historical ISC/MIT
or distribution obligations. No notices were removed and nothing was published.

Future patch removal still requires an exact upstream release passing corrected
invariants and regression tests. No upgrade, broad fork, production adoption or
owner integration is implied by either this stop or a future passing fixture proof.

</details>



# SOURCE: Collider/docs/agent-native/controller-ownership-plan/substrate-accepted-work-boundary-plan.md

Source SHA-256: 6e75e058e965d1b4c613bd9ad326e412cd368f1063fa9c6b5746755156b7ca3f

# Minimal Substrate accepted-work boundary — planning, partially specified

Proposal, not an implemented facade or a newly exported API. [Packet 2 inventory](../packet-2/owner-surface-inventory.md)
remains historical evidence. IDs below resolve in [the new ledger](planning-evidence-ledger.json).
The narrow scenario is one exact Linux world ephemeral agent task, observed through
its existing host receipt/supervisor, plus Workspace Environment Dependencies
read/add/remove. Retained V2, human approval, queue promotion and Qualified Session
continuation are not selected. Choosing this scenario does not certify its installed
path or evade its current policy/gates. [P01–P05, S01–S12; proposal]

## 2026-09-22 target-mapping reconciliation

This is a documentation-only reconciliation against the proposed
[Substrate handoff](../../../../substrate/docs/contracts/draft/collider-b01-b05-handoff.md)
and [Lane/World binding](../../../../substrate/docs/contracts/draft/lane-world-binding.md),
not owner approval, a callable API, or implementation authorization. The authoring
basis is Substrate `f43173ff31ef2c037a7183c577d05d22b4eba94a` plus its working-tree
drafts, and Collider `75631d432099fa18f1a320fc93863207ac7bcd2b`. Links are live
navigation, not immutable source pins; recheck relevant semantics before admission.
Older ledger/source classifications below retain their inspection-time scope;
they are not a fresh whole-repository or Linux audit.

[Packet A](../controller-correction/controller-correction-decision.md) already
records `CONTROLLER_CORRECTION_VERIFIED` for the maintained AgentKit 0.2.3 local
extension. Its [patch identity](../controller-correction/patch-provenance.json)
and [evidence](../controller-correction/regression-results.md) remain the client
input, not unchanged upstream compatibility or new tests. Do not restart that
correction merely because owner mapping is next. Runtime, native, distribution,
adoption, and C-OQ-027 closure remain separate.

## Identity and refresh

This section preserves the original inspection basis; see the dated reconciliation above.

| Repository | Verified absolute checkout and origin                                                                                   | Branch / HEAD                                                                                          | Initial state and difference from Packet 2                  |
| ---------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------- |
| Substrate  | `<HOME>/__Active_Code/atomize-hq/substrate`; `https://github.com/atomize-hq/substrate.git`             | `feat/internal-host-orchestrator-world-dispatch-bootstrap`; `a8b1a44ed2f9c33f6963eb552f9da05c198bce6a` | Clean; same HEAD, no relevant dirty changes or commit delta |
| UAA        | `<HOME>/__Active_Code/atomize-hq/unified-agent-api`; `git@github.com:atomize-hq/unified-agent-api.git` | `staging`; `30ebf0b3b459fac56ad47f211b6df16c65e08d77`                                                  | Clean; same HEAD, no relevant dirty changes or commit delta |

Roots were verified with Git before inspection using Packet 2's absolute paths.
No remote freshness, installed runtime or branch promotion is inferred. No owner
branch switch, fetch/pull, file/config change or runtime execution occurred.
Decisive paths were refreshed; unchanged broad inventory is inherited explicitly.
One citation-location discrepancy: Packet 2 S09 cites shell manifest lines 74–76;
the exact UAA declarations at this same pin are lines **65–67**. Content/version
agrees; this is corrected here without rewriting Packet 2. [R00, S08]

## Current operations and supported call chains

“Current” in this original inventory means the recorded inspection, not the 2026-09-22 target update.

Classification retains Packet 2 meanings: SUPPORTED_PRODUCTION_BOUNDARY means
supported for the stated narrow CLI scope, not proof of a complete Collider API or
an executed integration. Internal means **not consumer-authorized**, even where
Rust visibility or raw serialized transport exists. [P01]

| Operation / Packet 2 ID       | Current class                 | Entrypoint, semantic owner, identities and exposure                                                                                                                                                                                | Authorization, durability, readiness and failure limit                                                                                                                                                                                                                                    |
| ----------------------------- | ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Submit O01                    | SUPPORTED_PRODUCTION_BOUNDARY | `substrate agent start --backend … --prompt … --json`, `agent turn --session … --backend … --prompt … --json` → private `handle_agent_command/run_start/run_turn` → HSA/owner/member path; session selectors and command envelopes | Current inventory, runtime, policy and HSA checks; supported narrow CLI, not B1 receipt ingress. No caller intent/idempotency input. Explicit failure/accepted command envelope not general accepted work. S01/S08; P01 S01/S02                                                           |
| Pending/acceptance O02/O03    | EXISTS_INTERNAL_ONLY          | `WorldWorkReceiptRegistry` proposal → runtime ACK → `WorldWorkAcceptanceRecordV1` → supervisor/foreground receipt; request/store/receipt revision/exact work/world                                                                 | Private B1 machinery; durable immutable acceptance, proposed ID not accepted; no public human request resource. S02–S05                                                                                                                                                                   |
| Reconcile O04                 | EXISTS_INTERNAL_ONLY          | HSA Start transaction and B1 exact acceptance lookup; `resolve_accepted_world_dispatch_before_current_authority`                                                                                                                   | Internal no-replay/idempotency rules exist, but public Start hash includes content/config and cannot distinguish two arbitrary identical client intents. New actor-scoped ingress cannot bypass current authorization just because an internal cached result exists. S01–S03; P01 S02/S03 |
| Accepted/running/queue O05    | SPECIFIED_OR_PARTIALLY_LANDED | `ActiveTaskStateV1` and exact receipt/supervisor projection; no queued/promotion enum in selected path                                                                                                                             | Accepted and Running can be exposed when observed; no universal queue requirement. Session active is not task running. S04                                                                                                                                                                |
| Observe/lifetime O06          | EXISTS_INTERNAL_ONLY          | Host `launch_task_supervision_and_handoff_receipt` → independent observation task; world `MemberRuntimeManager` owns UAA event/completion/cancel handles                                                                           | Source separates observer from UI, but no supported GUI subscription. Process death/survival is unproven for a Collider connection. S03/S05–S08                                                                                                                                           |
| Replay/read-cut O07           | EXISTS_INTERNAL_ONLY          | Supervisor journal exact NDJSON frame/event IDs; raw `AgentClient` execution replay → world-service runtime replay                                                                                                                 | Exact receipt authorization still required outside raw transport. Host durable prefix versus bounded memory tail, not general durable runtime history. Gaps/eviction/interruption cannot redispatch. S05/S06                                                                              |
| Terminal O08                  | EXISTS_INTERNAL_ONLY          | `WorldWorkTerminalObservationV1` and dispatch `project_world_work_terminal_evidence_v1`; terminal ref/class, exit/span/source identities                                                                                           | Supervisor terminal authority; no terminal wall-clock field. No public exact read-cut endpoint. S03–S05                                                                                                                                                                                   |
| Cancel and race O09/O10       | EXISTS_INTERNAL_ONLY          | `cancel_world_work` → exact target/current policy → B4 cancellation request/delivery → supervisor terminal                                                                                                                         | First cancellation identity/claim persistence and already-terminal handling exist; policy-denied, unreachable, invalid/wrong-world/ambiguous target, pending closeout stay distinct. B4 exact proof/CLEAN/closure not located in current index. S03–S05/S12                               |
| Capability/readiness O13      | SUPPORTED_PRODUCTION_BOUNDARY | `agent list/status/doctor --json` → inventory/validator; UAA runtime support records                                                                                                                                               | Useful narrow current diagnostics, not a complete per-actor work authorization/readiness API. Preserve unknown/degraded; refresh at effect. P01 S08, S01/S08/U01                                                                                                                          |
| Lane association O17          | MISSING_OR_UNRESOLVED         | Receipt world_id/world_generation/workspace/session/backend exist; no owner Lane resource/join in decisive paths                                                                                                                   | No substitution of world/session/cwd for Lane. Scoped search again found no LaneId/lane_id/LaneBinding in tracked crates. B03 needed before coordinated Lane integration. C00/S02–S04                                                                                                     |
| Human approvals/input O11/O12 | EXISTS_INTERNAL_ONLY          | Obligation ledger + typed retained continuation, default-deny approval response policy                                                                                                                                             | Not exposed with human actor/revision/expiry or native suspended-request equivalence. Deferred; fixture is not owner capability. P01 S07/S09                                                                                                                                              |
| Qualified continuation O14    | SUPPORTED_PRODUCTION_BOUNDARY | `agent turn`, `agent reattach` (resume alias) → exact HSA/native selector or member path                                                                                                                                           | Supported narrow CLI; reattach can start an owner episode, not subscriber reopen. Selected scenario omits continuation. E3-F only if retained V2 later selected. S01/S08/S12/U01; P01 S02/S09                                                                                             |
| Non-chat read O15             | SUPPORTED_PRODUCTION_BOUNDARY | Linux `world deps current list available/enabled/applied --json`, `current show ITEM --json --explain`, `workspace list enabled --json` → authenticated deps context → inventory/config/world probe                                | Accepted-home/workspace/effective policy context; applied may be unavailable. No macOS owner execution was established. S10–S11                                                                                                                                                           |
| Non-chat mutation O16         | MISSING_OR_UNRESOLVED         | Existing supported `world deps workspace add/remove ITEM --json` → `run_workspace_add/remove` → normalized config patch/write                                                                                                      | Base editing exists, but required expected-revision CAS does not. Atomic file replacement is not atomic stale-write rejection; JSON followed by stdout human note complicates framing. S10–S11                                                                                            |
| Typed context selection       | MISSING_OR_UNRESOLVED         | Public text/file prompt; Handbook dependencies, not a traced callable reference/Resolution pipeline                                                                                                                                | No decisive tracked-crate ContextPack/context_pack/Handbook resolver match. B04 names missing owner integration; no frontend resolver or transcript workaround. P01 S12, S01                                                                                                              |

### Selected underlying path and UAA correspondence

Proposed integration target: Linux `RunWorldTask` / `EphemeralTask`, rather than
inventing a session-first GUI backend. Source chain:
`dispatch_orchestrator_world_request_for_principal` →
`prepare_task_acceptance_submission` (policy/runtime/concurrency and B1 proposal) →
`run_world_task_with_started_task_run_id_tx` →
`execute_run_world_task_stream_with_policy` → host supervision/receipt handoff;
member dispatch materializes exact descriptor/binary/world binding → world member
runtime → `PromptFulfillmentBridge` → `GatewayAdapterRuntime::run_control` → UAA
provider wrapper. The authoritative accepted-work family and handle ownership are
located, but **a supported human caller binding into this chain is not**. This is
B01, not permission to impersonate an orchestrator/toolbox caller. [S03, S06–S08]

Substrate shell, world-service and gateway consume exact UAA `=0.3.7`, defaults
disabled with `codex` and `claude_code`; wrappers are also 0.3.7. Cached archives were
rehashed against Cargo.lock (U02–U04). All report source
`416c094953ce20d6e231b5047453273287248510`; six facade source files still equal
staging. No whole release/staging, wrapper tree, installed binary or enabled runtime
equivalence is inferred. UAA staging's other features do not become capabilities.
No UAA change is currently justified. Owner implementation must bind the actual
chosen descriptor/provider/platform and recheck this version/features at its base.
[S08, U01–U04]

Do not confuse the normal member bridge with shell durable Start/native resume,
which use pinned native wrapper APIs to retain typed continuity. Do not select the
unlanded retained V2 E3-F path by accident. Existing external-sandbox launch posture
belongs to Substrate; this packet neither widens it nor changes host/debugger
permissions. [S08, S12; P01 S09]

## Smallest independent boundary

**Recommendation/proposal:** a narrow Substrate shell/HSA-owned accepted-work
access boundary over existing registry/supervisor/runtime, plus reuse of CLI reads.
It should serve a terminal or another non-AgentKit client unchanged. Prefer an
owner command/client exposure with structured replies where sufficient; a local
service is justified only for subscription/lifetime semantics that command replies
cannot supply. No Node sidecar, HTTP, public Rust SDK or Tauri topology is selected.
A Tauri adapter later routes these calls; it cannot supply missing owner semantics.
[P01–P05, S01–S11; inference]

The following is **proposed operation vocabulary, not existing API names**:

| Proposed operation                                  | Necessary contract and current owner                                                                                                                                                                                          | Addition type                                                                                      |
| --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| SubmitIntent / ReconcileRequest                     | Actor-scoped caller request key + immutable normalized payload binding → pending/unknown/refused or exact accepted receipt/work ref. Reconcile is read-only. Reuse B1/HSA acceptance and no-replay rules; no second registry. | Missing supported caller/correlation semantics plus exposure: B01                                  |
| ReadAcceptedWork                                    | Exact receipt/store/revision/work → owner status/source refs/optional terminal; current caller authorization                                                                                                                  | Primarily exposure/projection over S02–S05, subject to exact path proof B02                        |
| ObserveAcceptedWork                                 | Exact receipt + owner cursor → authorized snapshot cut/replay/live events, completeness/retention and explicit unavailable                                                                                                    | Exposure plus missing stable read-cut/reconnect contract B02; not another supervisor               |
| RequestCancellation / ReadCancellation              | Exact receipt and client cancel key → original owner cancellation identity, accepted/refused/unknown/already-terminal, separately actual terminal evidence                                                                    | Existing B4 semantics plus public actor/correlation/exposure; B02                                  |
| DiscoverOperations                                  | Version + declared operation support + current authorization + availability/readiness/unknown for exact backend/resource                                                                                                      | Narrow projection of inventory/validator/policy; no grant from metadata                            |
| ReadWorkspaceRequirements / SetWorkspaceRequirement | Workspace inventory item + expected owner revision + normalized enabled boolean → new revision/readback or conflict/refusal                                                                                                   | Existing deps read/edit; missing CAS/all-writer coordination, caller authorization and framing B05 |

### Cross-cutting proposed facade contract

- **Identity/security:** authenticate actual local principal or an explicitly
  owner-approved delegated caller at ingress. Client actor/world/workspace fields
  are references, never authority. Initially prefer same authorized local owner
  principal; this does not grant a remote browser credentials. Current accepted-home,
  HSA/session/backend/world-generation, policy, capability and context checks still
  apply. Reconcile/read must authorize access even to already accepted records;
  no raw internal capability, private store, lease or credential reaches the UI.
- **Idempotency:** scope caller key to authenticated principal/workspace/operation;
  owner persists binding to normalized payload before uncertain effect and joins
  existing proposal/acceptance after ACK loss. Same key/same payload joins the same
  request; pending progress can advance, including acceptance known with incomplete
  receipt material, without changing the immutable accepted result. Equality covers
  destination/association revision, action/payload, supplied purpose/Scope/Resolution
  Posture and selected context identities/revisions/requiredness, execution constraints
  and delegation selector. Conflicting payload rejects; distinct keys with identical
  text are distinct intents. Retention window and tombstone/expired-key behavior are **B01
  unresolved facts**. Do not promise exactly-once execution or reuse expired keys.
- **Observation:** bind stream/frame cursor, event cursor, receipt revision and
  owner observation epoch separately. Owner returns a coherent exact-work cut or
  explicitly rejects/retries stale read, not a global Workspace snapshot. Adapter
  derives protocol checkpoint outside owner authority. B02 must specify retention,
  completeness and restart behavior; current fallback has 1,024 streams, 65,536
  frames/stream, 64 MiB/stream and eight subscriptions/stream. Durable member-turn
  replay is a distinct retained path, not free durability for ephemeral work.
- **Cancellation:** return request/delivery facts and original cancellation identity,
  plus terminal reference only when available. Timeout is unknown, not cancelled.
  Do not call session Stop as a surrogate for exact-work cancellation. [S04–S06]
- **Lifetime:** owner host/supervisor and world member manager retain UAA/provider
  handles independently of UI and transport connections. A command process may
  submit/read, but owner supervision must outlive it. Disconnect drops only observer;
  restart recovers owner-guaranteed same work or reports interruption/unavailability.
  No resubmission to fill a missing tail. A frontend-owned provider handle is a stop.
- **Versioning/credentials:** version the bounded request/response and reference
  schema independently of AgentKit; unknown versions fail closed. Redact payloads
  and return only allowed references/diagnostics. Reuse current owner credential
  bootstrap/carriers; no UI key store, copied provider credentials, host-program
  permission change or direct provider stream. IPC/remote authentication technology
  remains B02 exposure design, not an implicitly approved Tauri command.
- **Compatibility/tests:** preserve supported existing CLI outputs/callers. Add an
  explicitly versioned machine-output operation rather than silently changing legacy
  `--json` framing. Keep internal toolbox authority distinct. Extend existing public
  control, receipt/supervisor, runtime replay and World deps tests; no change to UAA
  needed unless a separately evidenced owner defect requires its own authorization.
- **Non-ownership:** not a receipt registry, scheduler, supervisor, credential store,
  Handbook resolver, semantic authority, Activity Index, AgentKit protocol server,
  generic filesystem editor or replacement UAA. It exposes/orchestrates current
  owners; missing durable behavior must land in those owners first.

## Non-chat contract: keep the proved Toolkit region

The Gate B controlled DataGrid already provides selection, editing, keyboard/focus,
available/enabled/applied/provenance columns and inline/tab/Sticky behavior. Retain
it; replace fixture callbacks only after a real owner contract exists. No new editor
domain or direct component file write is warranted. [F10, G02–G03]

**Proposed smallest mutation:** set one known workspace requirement's enabled state
through the existing add/remove owner semantics. A local yes/no editor normalizes
to a boolean proposal; owner independently validates the canonical item and current
inventory/authority. Distinguish available definition, workspace enabled override,
inherited enablement and observed applied state. Removing workspace enablement does
not disable an inherited requirement and does not uninstall. Restrict the proof to
explicitly owner-editable Workspace-local contributions. Inherited contributions
remain read-only, but a mixed local/inherited row can remove its editable local
declaration while effective enablement stays true. Inherited-only rows default to
read-only unless the owner permits adding a local declaration; unavailable/unknown
rows remain read-only. The effective boolean never determines editability. No
sync/install/provision, global configuration, credentials or policy mutation. [S10–S11; proposal]

**B05 required transaction:** owner read returns opaque revision covering the
workspace patch and relevant effective inventory/config preconditions. Under one
owner serialization boundary, revalidate caller/current policy and expected revision,
normalize add/remove, perform all-or-none mutation, and return resulting revision
plus authoritative readback. Concurrent stale request gets conflict and current
read reference, with no partial write. All supported owner writers to that patch
(including legacy add/remove/config paths) must participate in serialization or be
explicitly excluded by a reviewed contract; an endpoint-only mutex is insufficient.
Atomic rename alone does not close this requirement. External unsupported file
writes cannot be represented as serialized owner mutations; detect changed preimage
and refuse rather than claiming universal atomicity. Exact revision scope/locking
integration remains B05 design work, not established implementation. [S11; proposal]

Stable structured output must separate human diagnostics from reply framing.
Lost mutation ACK reconciles by operation identity/readback rather than blind
reapply; a readback differing because of a later authorized mutation is not proof
the original failed. Mutation result must identify its own committed revision.
Two placements share one pending operation. Toolkit callbacks never obtain write
paths, shell escape hatches or credentials. [F10/S10–S11; proposal]

The [B05 target](../../../../substrate/docs/contracts/draft/collider-b01-b05-handoff.md#71-planning-target-exact-mutation-and-writer-contract)
adds these consumer result rules: retain the mutation key, item, desired local
boolean and expected revision; an accepted retry resolves the original operation
before comparing its now-old expected revision. `Committed` displays its original
revision/readback; a later current value is separately labeled. `NoChange` still
requires authorization and valid expected revision, and retains an operation result
without forcing a declaration revision. `Conflict` refreshes for a new explicit
intent/key, not automatic overwrite. `Refused`/`KeyConflict` preserve their reasons;
`PendingOrUnknown` keeps the original key for read-only reconciliation. Absent,
expired or unavailable history is not proof of failure. Pages must share an owner
read revision. None of these results implies install/provision or active-World change.

## Concrete owner prerequisites and bounded next design work

| ID           | Missing fact/owner obligation                                                                                                                                                                                           | Closure and permitted independence                                                                                                                                                                      |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| B01          | Human caller-to-B1/HSA request namespace/authorization, durable intent idempotency and exact reconcile ingress; key retention/expiry                                                                                    | Substrate shell/HSA owners approve schema/call chain and tests for unknown ACK, retry, distinct identical prompts and current authorization. Not exposure-only; no toolbox impersonation.               |
| B02          | Supported authorized exact-work read/cancel/observation, consistent read cut, transport principal binding, nonstandard terminal-class mapping and concrete ephemeral replay/restart limits; current B4 exact-path proof | Substrate registry/supervisor/runtime owner publishes versioned boundary and installed-path proof. Missing runtime tail yields unavailable, not fabricated replay. Does not require all runtime slices. |
| B03          | Owner-recognized Lane identity/association with immutable actual world execution binding                                                                                                                                | Narrow Substrate Lane/binding contract; do not build a whole Lane scheduler or alias world ID. Needed for real CP/Lane proof, not controller fixture.                                                   |
| B04          | Typed selected references/Scope/Resolution → Handbook semantics → Substrate delivered context and provenance on chosen path                                                                                             | Owner-reviewed callable contract plus current-binding/policy tests. No assumption from Cargo dependencies; no broad Handbook/Session redesign.                                                          |
| B05          | Deps revision scope, serialized compare/mutate across relevant owner writers, actor enforcement, stable output and result reconciliation                                                                                | Separately implement/test in World deps/config ownership; can proceed independently of agent B01/B02 and controller A.                                                                                  |
| B06 deferred | Supported human approval request/revision/actor/expiry and continuation identity                                                                                                                                        | Keep disabled; not a prerequisite for selected minimal path.                                                                                                                                            |
| B07 deferred | Qualified continuation on selected installed path with revalidation                                                                                                                                                     | Not selected. Existing narrow CLI continuity is not removed; E3-F gates only a later retained V2 selection.                                                                                             |

Because B01–B05 have concrete but not yet owner-approved contract choices, the
owner plan is **partial**, not a complete implementation authorization. Request
focused owner contract design for these rows, not an open-ended facade build.
Read-only CLI diagnostics/deps proof may be separately useful now, but cannot be
misreported as complete Gate C. Controller correction and B05 can be specified,
reviewed and tested independently; no need to finish Substrate as a whole first.

## Current target mapping and remaining bindings — 2026-09-22

The historical prerequisites above are not a claim that later Substrate foundations
remain absent. The current draft handoff reconciles later owner work while keeping
consumer exposure and proof separate. This table is a mapping into those targets,
not a replacement owner specification or a new readiness registry.

| Target | Collider mapping now specified | Exact binding / proof still required |
|---|---|---|
| B01 | Original principal/Workspace/operation key and protected semantic inputs; request progress can advance. Acceptance reported with incomplete material is distinct from an actionable receipt and from Running. Complete reconciliation can reveal already-terminal work. | Approved ingress/delegation, correlation/equality/retention, runtime profile and uncertain-ACK/recovery evidence. |
| B02 | Exact-work snapshot/cursor and separate contiguous projection sequence; gaps/expiry visible. [Lifecycle mapping](controller-owner-lifecycle-contract.md#cancellation-races-and-observation) preserves cancellation categories and owner-unknown versus local failure. Owner terminal remains separate from `completed`. | Supported read/subscribe/cancel exposure and limits; lossless segment-ending binding for `NeedsRetainedFollowup`/`Invalidated`; restart/replay/cancel-race evidence. |
| B03 | One shared request/work model for CP and Lane. Preserve immutable accepted Lane/World association and revision separately from current association; new submit validates current destination. Opening a view never redispatches. | Genuine owner association, authorized selected execution/custody joins; no Lane=World/session aliases or general multi-World authority inferred from a matching-session path. |
| B04 | Persist explicit purpose/action/Scope/posture and versioned required/optional selections; show typed refusal/omissions and resolved-versus-injected lineage. No implicit transcript or native continuation. | [Selected positive-path binding](../../../../substrate/docs/contracts/draft/collider-b01-b05-handoff.md#outstanding-positive-path-binding-for-the-selected-b04-case) remains OPEN: actual reference kind, supported SDK/advanced-owner operation, validation/currentness, structured success/refusal, and Substrate delivery evidence. Not merely an unchosen wire encoding. |
| B05 | Controlled DataGrid edits local declarations only; owner editability, coherent read revision, mutation identity and original-result/current-readback distinction drive both placements. | Covered-writer inventory/transaction coordination, actor enforcement, supported structured API and concurrency/crash/ACK-loss proof. |

**Gate separation:** the target-based consumer plan can advance with named bindings
open; it is not a claim that those bindings are complete or callable. Actual B04
and the full selected integrated Gate C require real supported Handbook resolution
and Substrate delivery, plus the other applicable B01–B05/runtime prerequisites.
A separately authorized B05 or partial B01–B03 proof establishes only its named
scope. A new successful task dispatch must satisfy mandatory context on its actual
authorized route; an empty selection or public prompt argument is no bypass.
Do not replace the selected experiment with a smaller one, restart verified Packet A,
resume E3-F, or implement any owner boundary under this documentation update.



# SOURCE: Collider/proofs/agentkit-controller-correction/README.md

Source SHA-256: dfea2074ef469c8565d56726ea0950a41fc98c5470ee0c000aee1f1ba5e51122

# Packet A — isolated patched-client proof

Fixture-only; not Substrate/UAA, native Tauri, production adoption or distribution
clearance. Original `../agentkit-gate-b/` remains an unmodified negative control.

Use **pnpm 10.11.1** from this directory:

```sh
pnpm install --frozen-lockfile --ignore-scripts
pnpm verify:artifact
pnpm test
pnpm test:invariants
pnpm typecheck
pnpm build
pnpm test:ui
```

`test` contains corrected actual-client and preserved fixture tests, plus unchanged
upstream full-profile assertions (14 checks; approval/queue disabled) and a separate
expected rejection of accepted-first streams by its first-run.started assumption.
No upstream conformance code is modified. The two shared invariant tests pass in
`test:invariants`; against exact Gate B they must fail at the named assertions:

```sh
GATE_B_ROOT="$(cd ../agentkit-gate-b && pwd)" \
  node --experimental-import-meta-resolve --test tests/unpatched-control.test.mjs
```

Do not count a missing package/import error as a successful negative control.
Artifact verification hashes the entire installed AgentKit and Toolkit payloads,
patch and touched files; checks public entrypoint resolution and source-map loss.
Missing registration/application cannot masquerade as corrected bytes. Patch
registration is exact-version in `pnpm-workspace.yaml`, frozen by `pnpm-lock.yaml`.
No manually modified installation is a deliverable.

The proof constructor requires `collider-owner-lifecycle/1`; ordinary v2 negotiation
is insufficient. `artifact-identity.json` binds revision `collider-agentkit-0.2.3-a1`.
`accepted`/`unobserved` are local protocol extensions. Stock full UI is not selected;
actual injected Provider/useAgentRun consumers and Toolkit DataGrid are tested.
`completed` still means observer/segment settlement, not owner success. Local
cancelRequest times/generation are never owner terminal facts or durable tokens.

See the four records under `../../docs/agent-native/controller-correction/` for
contract history, exact tested hashes/results, limitations and review. Future
updates require a reviewed exact-version patch and full invariant regression.



# SOURCE: Collider/src/app/page.tsx

Source SHA-256: d5bf39f34ee5251acb31397966b85617996502ddb980f12d92e7ec4183fe9081

export default function Home() {
  return (
    <main className="min-h-screen" style={{ background: 'var(--color-background-base)' }}>
      <p style={{ color: 'var(--color-text-secondary)', padding: '2rem' }}>Collider</p>
    </main>
  );
}



# SOURCE: Substrate reconciliation — selected sections 13.4 and 15

Uncommitted investigation/proposal, not adopted owner contracts. IMPORTANT AUTHOR CORRECTION SINCE WRITING: fixture-first staged proof was already specified by canonical C-OQ-027 and the proof spec. Do not treat section 15 as a newly required strategic choice or a replacement plan. Its minimal subset cannot waive existing mandatory proof coverage. Review that correction critically.

### 13.4 Exact change responsibilities and closure tests

Names identify responsibilities, not selected DTO spellings or another acceptance registry. H1–H3 are the concrete Handbook changes required for this recommended general B04 composition; H4 is required for the recommended continuing-local-project scope, not for every possible initial-only demonstration.

| ID / responsible boundary | Required change, reusable base and explicit limit | Closure evidence |
|---|---|---|
| H1 — Handbook SDK supported composition | Add or explicitly revise a typed materialization request binding selected owner refs plus expected content revisions, purpose/action, Scope/Resolution and requiredness to a trusted repository/authority context. Publish supported kinds/representations and effective limits. Use SDK composition and existing Flow/artifact/native owners; do not encode missing fields inside packet IDs or simulate requests by changing project config. Candidate boundary: `crates/sdk/src/flow_api.rs`, `contract_membrane/flow_operation.rs`, `artifact.rs`, or one narrowly owned new SDK module, chosen during owner design. | A normal external SDK dependency, without test-support/private entry points, requests one required exact reference and receives expected content plus typed stale/unsupported/denied/optional outcomes. Supported subset is explicit and satisfies every requirement of the selected experiment. |
| H2 — Handbook Engine admission and exact material source | Join current semantic/disclosure authority with the actual complete read/render closure. Reuse retained roots, safe bounded reads, artifact owner locks/recovery and Context Resolution/projection checks. Extend owner entry points where expected content revisions, coherent dependency cuts and protected consumption are not expressible. Flow must consume admitted retained inputs or an equivalent guarded owner result, not independently select/reread a different baseline. Candidate owners: `context_resolution.rs`, `projection.rs`, `artifact_repository.rs`, `canonical_artifacts.rs`, Flow `resolver.rs`. | Wrong repository, unapproved source closure, path substitution, stale required ref and concurrent authority/source changes refuse with disclosure-safe results. Exact source and rendered byte correspondence survives multi-input races. A consumer mutex or a post-read digest comparison alone cannot pass this test. |
| H3 — Handbook result/current-use contract | Project the owner-validated input/dependency and authority basis, exact selected dispositions, retained material identities, representation/fidelity and complete typed failures. Expose a supported current-use revalidation/admission binding needed by S1; distinguish historical resolution from permission now. Reuse ordinary structured outcomes and per-file hashes; extend missing B04 distinctions rather than parsing log strings. Persistence can reuse owner facilities or authorized Substrate retention; a new Handbook content database is not established as necessary. | Result completeness/strict decoding, required full-content budget refusal, explicitly permitted omission, source-vs-render-vs-envelope identity, reopened history and revoked new-use tests. No currentness token is accepted outside its owner-defined scope/lifetime. |
| H4 — Handbook continuing-project local authority | Adopt and implement the bounded successor-aware native root-envelope route recommended by October 8 research, using current local lineage and positively resolved supported D/G states. Separate binding management from root use; initial human establishment, delegated replacement only under prior exact permission, independent revocation/history and original-request recovery. Preserve initial refusal and credential-required legacy behavior. Candidate owners: `local_authority.rs` and its local-check mechanisms, `context_resolution.rs`, typed SDK local authority surface. Full child/escalation/memory grant administration is not required. | Reopen a continuing local-check project, establish/use an explicitly authorized root binding, refuse unauthorized replacement and revoked/stale use, recover original outcomes without fresh effects. Existing evaluation, correction or work-registration permissions cannot authorize this operation. |
| S1 — Substrate acceptance and delivery adapter | Implement trusted repository/caller binding, original-request material retention, live admission protocol with H3, exact B01/B03 owner joins and runtime delivery evidence. Keep existing Receipt Registry/Supervisor/HSA responsibilities. A Handbook receipt is not execution acceptance and content preparation is not injection. | Positive installed Linux task consumes the exact material; lost preparation/submission/delivery acknowledgements, revocation and destination races preserve truthful accepted/pending/failed/uncertain outcomes and immutable original bindings. |
| C1 — Collider workflow construction and consumer mapping | Build the bounded interaction/projection surfaces; they are not an existing product UI (§15). Use the supported selections/capabilities and show semantic refusal, preparation, acceptance and actual delivery separately. Carry owner-qualified references, not inferred filenames or regenerated context. | Both consumer views identify the same accepted work/material; no display-only `Injected`, silent optional downgrade, latest substitution or automatic provider continuation. |

**H4 scope choice remains proposed:** recommend continuing local projects, not a fresh-setup-only workaround. An explicitly agreed initial-only prototype could use the existing initial admission and omit H4, but would still need H1–H3/S1/C1 and must declare that limitation. Existing SDK-first approval does not itself select the new Context Resolution management rule or grant a runtime principal permission. Preserve legacy credential paths; no automatic fallback or mandatory credential acquisition for the proposed local route. The research's reusable binding lifecycle is the recommendation, not a universal B04 prerequisite or a pretext to finish all Handbook architecture.

**Real-owner order (fixture-first Collider work is separately scoped in §15):** select that bounded authority scope; design H1–H3 together with S1's exact admission/effect protocol; implement H4 where the selected scope requires it; then run SDK and installed integration proofs. H1–H4 can share existing owners without a separate general authority subsystem. B01/B03 and B02 remain necessary execution/observation contracts; B05 remains a separate complete-Gate-C requirement. File-native storage and complete Work Progression are not prerequisites established by this investigation.



## 15. Corrected Collider readiness and smallest next experiment — 2026-10-18

**Disposition:** the user corrected the readiness estimate: Collider's application UI is a shell; its canonical documents describe intended construction, not a completed UI awaiting a few adapters. This section corrects that estimate and scopes a recommended next experiment. It does not approve implementation, select a production frontend, close open architecture questions, or weaken the SDK-first/full-B04 condition for the eventual real-owner prototype. Sections 13–14 retain their owner boundaries; their consumer-mapping rows were not a complete Collider implementation estimate.

### 15.1 Readiness correction and evidence

The full [glossary](../../../collider/docs/agent-native/canonical/00-collider-v1-glossary.md), [foundations](../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md), [open architecture questions](../../../collider/docs/agent-native/canonical/02-collider-v1-open-architecture-questions.md), and [owner-boundary proof specification](../../../collider/docs/agent-native/proof/agentkit-owner-boundary-proof-spec.md) were read, not merely their summaries. Repository pins remain those in §2.2. The application and isolated proof must be assessed separately:

| Surface | Verified state | Planning consequence |
|---|---|---|
| Main Collider application | [Home](../../../collider/src/app/page.tsx) renders a background and “Collider”; [layout](../../../collider/src/app/layout.tsx) is a basic wrapper. [Root package scripts](../../../collider/package.json) still use Next. | Workstream/Composer interaction, owner-backed views, context states, projection/reconstruction and execution awareness must be built for the chosen slice. Component/design-system assets do not establish those workflows. |
| Isolated corrected-client proof | [Packet A README](../../../collider/proofs/agentkit-controller-correction/README.md) explicitly limits it to fixtures. Actual injected AgentKit Provider/useAgentRun consumers and Toolkit DataGrid were exercised; stock full UI was not selected. Exact-version patch and local lifecycle corrections exist. | Reuse these candidate/proof assets and preserve their regression evidence. Do not restart the controller correction, treat it as production UI, or infer durable owner cancellation from its local state. |
| Product and frontend direction | Canonical foundations specify a substantially larger coherent V1. Open architecture questions retain unresolved history, placement, execution and frontend-adoption choices; the target stack includes Vite. | A bounded proof need not build all V1 navigation/editors or settle all 27 questions. Main-app Next-to-Vite migration, native-host qualification and final frontend adoption are not already done. |
| Downstream specifications | Foundations §19 anticipates `03-collider-v1-workspace-shell.md` and `04-collider-v1-execution-and-binding.md`; neither was found in the inspected agent-native documentation tree. | Their intended role is not evidence those detailed specifications exist. A small experiment-specific interaction/mapping packet can precede completing the broader Shell and Execution specifications. |
| Real owner integration | The consumer plans and Substrate drafts define target behavior; §13 identifies actual Handbook gaps. | There is no demonstrated complete Handbook → Substrate → Collider delivery path. Passing client fixtures cannot close it. |

### 15.2 Recommended first step: a bounded Collider workflow with owner fixtures

**Recommendation, pending scope approval:** extend the existing isolated corrected-client proof into one inspectable Collider workflow. Prefer its existing experimental host over migrating the main application or creating a new orchestration layer. Determine exact files during the implementation packet; preserve the original Gate B negative control and Packet A baseline evidence. This is a fixture-backed UI/integration experiment—not a working three-owner agent execution and not full Gate C. No real provider, Handbook mutation, Substrate execution, permission grant or credential is required for this step.

**Representative interaction:** in one fixture Workspace, select one exact Handbook reference and a Lane/World destination, enter a current natural-language intent, inspect the resolved-context presentation, submit, follow the same work in Control Plane and Lane views, inspect the result/provenance, and close/reopen a placement without creating another request. A separate dependency-declaration grid exercises B05-style read/add/remove and stale-write handling. Names and identities are fixture data; Lane, World, work, reference and view identities remain distinct.

**Minimum constructed surface:**

- One shared behavioral owner for the selected work, with CP/Lane presentations—not independent dispatch controllers per view and not a new global scheduler. One minimal Composer sends current intent plus explicit selections, never the accumulated transcript.
- A small reference selector/read-only semantic preview and context-status display: selected → resolved material → delivery evidence, with source revisions and omissions visible. No full Handbook browser, semantic editor, Work Item engine or generic resolver is needed.
- One built-in Activity projection with stable identities, pending/uncertain/acceptance-reported-but-incomplete states, running and authoritative terminal presentation. Preserve nonstandard/unsupported outcomes rather than silently coercing them into success.
- A non-chat grid Surface, inline and opened as a tab using an explicit keyboard-accessible action; closing/reopening preserves work identity. Basic unavailable-content fallback and persistent execution awareness/cancel affordance remain visible when the work's placement closes. No new Sticky region or full Navigator customization is required for this slice.
- Fixture-owned replay/checkpoints and deterministic failure/race controls, separate from projection-local state. Simulated reconnect or reconstructed controller state proves fixture behavior only, not a production persistence choice or native process survival.

The natural-language interaction may receive a deterministic scripted result; label it **simulated execution**. Adding a direct model call to make it look live would bypass the selected owners and change the experiment. Raw-shell/PTY operation, approvals, queued promotion, native-session continuation, multi-worker scheduling, merges, remote Worlds, complete Workspace Shell, full Inspector and file-native storage stay outside this bounded slice. Existing negative fixture coverage for deferred capabilities is preserved; it does not enable them.

### 15.3 Fixture contracts and acceptance boundaries

Use one versioned scenario/mapping ledger with a source pointer and explicit status for each exercised behavior: **implemented owner behavior**, **documented target**, or **unselected proposal**. Bind fixtures to the actual candidate client/profile, not just a transport-shaped mock. Keep fixture identifiers and evidence visibly namespaced as synthetic; they must never be usable as credentials, real receipts or permission tokens. Owner refusal is an input to the fixture, not authorization logic implemented by Collider.

| Boundary | Minimum fixture coverage | What passing it does not prove |
|---|---|---|
| B01 submission | Pending versus accepted; lost acknowledgement; repeated same request versus distinct request; acceptance reported before complete material; recover original identity with zero duplicate dispatch. | Durable caller authorization, acceptance persistence, crash recovery or installed owner admission. |
| B02 observation/control | Same work in CP/Lane; detach/reopen/reconnect; gap/expired observation; cancel acknowledgement versus terminal; cancellation/completion race; unsupported outcome visible. | Real retention/replay horizon, background lifetime, owner cancellation idempotency or restart safety. |
| B03 binding | Explicit current destination selection; immutable historical accepted binding; changed association rejects a new stale request rather than rewriting accepted history. | Lane authority, actual World placement, filesystem isolation or association fencing. |
| B04 context | One required exact reference plus an optional reference; resolved bytes/revision/provenance; denied/missing/stale required input prevents new submission; optional omission is explicit; delivery failure after acceptance preserves acceptance; Injected appears only upon a synthetic delivery-evidence event. | Trusted source binding, owner-backed permission/currentness, supported SDK materialization or actual agent-visible delivery. |
| B05 non-chat operation | Requirement read/add/remove, expected revision, competing write rejection and authoritative fixture readback; declaration status distinct from realized/install status. | All-writer CAS, genuine configuration persistence, package installation or environment realization. |

This is the **new delta's minimum acceptance set**, not a replacement for mandatory cases in the existing proof specification or Packet A regressions. Map reused tests versus missing tests before implementation. In particular, preserve capability/unsupported-queue handling, shared-controller behavior, malformed mapping rejection and source-cursor versus projection-order distinctions. Requiredness, revision and source fields in fixture DTOs are provisional consumer mappings, not silently selected public SDK serialization.

**Fixture exit:** a reviewer can run the bounded interaction and deterministic failures through the actual candidate client, inspect the work/context/placement joins and assertions, and identify every synthetic or unresolved owner guarantee. Produce the changed-file scope, version/profile pins, reproducible commands and pass/fail evidence. Do not label this Gate C, native Tauri readiness, frontend adoption, full B04, or coherent product V1.

### 15.4 Implementation work by repository and sequence

| Stage / repository | Smallest work package | Dependency and exit |
|---|---|---|
| Fixture design — all three responsibilities, recorded in Collider's existing proof planning | Pin the interaction, supported/proposed reference kind, semantic fields, capability/refusal table and scenario ledger to these sources. Identify reused Packet A tests and new gaps. Review Handbook meanings and Substrate lifecycle meanings without inventing callable APIs. | Can begin before real owners implement the targets. Unknown guarantees stay unknown or explicitly proposed; a fixture cannot decide them. |
| Fixture construction — **Collider** | Build the small Composer/reference/context view, shared CP/Lane projection, lifecycle awareness, inline/tab Surface and declaration grid; wire versioned fixture adapters and deterministic tests into the isolated candidate proof. | Requires approved bounded scope and implementation authorization. Reuse existing assets where they actually satisfy requirements; this remains real frontend work, not merely wiring two SDK calls. |
| Fixture support — **Handbook / Substrate** | Qualify the fixture ledger against source and accepted target semantics; record unsupported/undecided cases. | **No Handbook or Substrate runtime code change is required just to run the fixture experiment.** Source-backed review is still required; synthetic success must not be advertised as their capability. |
| Real semantic path — **Handbook** | H1–H3 supported bounded composition, exact admitted read/render source closure, evidence/current-use validation; H4 only for selected continuing-project authority. Test the normal external SDK path on a supported build. | Scope and trusted principal/binding must be selected; no raw Flow workaround or test-support bypass. SDK remains changeable. File-native engine remains separate. |
| Real execution path — **Substrate** | Finish concrete B01/B03 authority/acceptance/recovery bindings together, then B02 observation/control; implement the required ingress/owner extensions. Bind S1 to Handbook's retained exact material and actual runtime delivery. Implement/expose B05 revision-aware operations and all-writer coordination for the complete experiment. | Owner contract/recovery review and applicable Linux installed/security gates remain required. Existing owners are reused, not replaced by a facade/controller. This pass does not resume E3-F. |
| Real consumer join — **Collider**, with both owners | Replace fixture adapters through explicit supported mappings; expose supported capabilities; retain identical presentation semantics where the actual contracts agree. Revise provisional mappings when owners require it. | Run the real positive context/execution path and §14.5 negative/race cases. Include real B05 read/mutate/stale-conflict/readback for full Gate C; a task-only partial proof is labeled partial. |
| Deployment and product decision — **Collider** | Complete applicable Gate D native-host/artifact/security/licensing/economics evidence and explicitly decide candidate adoption. Then plan/build the coherent Workspace Shell/product slice. | Web fixture tests do not prove Tauri IPC/CSP, packaging or background lifetime. Broader implementation-neutral Shell design may proceed earlier in parallel; full product construction is not a prerequisite for the bounded experiment. |

**Revised order:** approve the bounded fixture experience → build/test it while Handbook/Substrate contract design proceeds in parallel → implement and qualify the selected real owner paths → join and run Gate C → complete applicable Gate D/adoption decisions → advance the coherent product UI. Do not wait for all architecture/storage work before learning about Collider interaction; equally, do not treat fixture UI success as permission to skip owner work. Actual execution and authenticated context bytes are only available at the real-owner stage.

For the real-owner path, the smallest target remains one supported semantic selection and one Linux ephemeral World task with CP/Lane views, plus the separate B05 declaration operation. It does not require a general scheduler, every Handbook domain, every reference representation, or a complete Lane topology. Exact endpoints, retained-material protocol, SDK source binding and runtime delivery hook still require concrete owner design; this document does not invent those missing interfaces.

### 15.5 Decisions versus implementation work

| Decision | Recommendation / status | When needed |
|---|---|---|
| Next proof milestone: fixture UI or real-owner execution first? | **Recommend fixture-first** as the smallest useful next Collider work; real-owner integration remains the eventual acceptance target. This request authorizes documenting the choice, not silently approving implementation. | Before starting the next implementation packet. If the immediate milestone must execute real work, the Handbook/Substrate rows above become prerequisites instead. |
| Scope of the fixture experience and candidate reuse | Recommend the bounded interaction in §15.2, reusing the isolated corrected AgentKit/Toolkit candidate rather than selecting it for production or building full V1 now. | Approve with the milestone; this is not a request to reopen all frontend/product architecture. |
| Real Handbook authority population | Continuing-project support remains recommended; initial-only is a deliberate narrower demonstration and cannot be substituted without agreement. H4's successor binding is still an owner-direction proposal. | Before claiming the real scenario's authority route, not before constructing neutral fixture UI. Fixtures for unselected authority semantics must be labeled proposals. |

The actual reference kind/recipe mapping, typed DTOs, trusted SDK source/principal binding, transport placement, exact durable recovery/custody operations, test reuse and file-level packet are **engineering/owner-contract work**, not product questions to make the user answer. Existing full-B04, ownership, fresh-context and file-native-separation decisions remain controlling. No new authority, API, production framework or repository-wide migration is adopted here.