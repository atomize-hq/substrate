# ADR Curation Ledger

This ledger records the curation rules and the current classification state for ADRs that may move
from `docs/project_management/adrs/**` into the stable `docs/adr/**` tree.

## Classification Axes

Every ADR under review is classified on two axes:

1. Curation disposition
   - `stable_keeper`: belongs in `docs/adr/**`
   - `historical_only`: keep only as history or audit trail
   - `superseded`: retained only as a replaced decision record
2. Implementation posture
   - `implemented`
   - `draft_but_implemented`
   - `still_draft`

`draft_but_implemented` is the key normalization bucket for this repo because multiple ADRs still
say `Status: Draft` while stable docs, code, and tests already rely on their decisions.

## Keep Criteria

Promote an ADR into `docs/adr/**` when one or more of these conditions hold:

- it defines a current operator-facing contract that stable docs already expose
- it defines a runtime boundary or data model that current code already implements
- stable docs, tests, or contract references still use the ADR number as an authority anchor
- later ADRs still depend on it as an active prerequisite rather than only historical context

Do not promote an ADR yet when it is primarily:

- a planning-pack wrapper around already-extracted stable docs
- a feature-local execution plan rather than a durable architectural decision
- a superseded proposal whose current value is only historical context

## Migration Policy

- Preferred strategy: `restate + supersede`
- Stable target tree:
  - `docs/adr/implemented/`
  - `docs/adr/draft/`
  - `docs/adr/historical/`
- Promotion order:
  1. curate stable keepers with current operator/runtime contract weight
  2. repoint stable references to `docs/adr/**`
  3. leave compatibility stubs in `docs/project_management/adrs/**` only where older planning or
     archive material still needs a breadcrumb

## First Cluster: Policy and Gateway Contract ADRs

This is the first high-value cluster called out by the retirement tracker.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0027 | `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md` | `stable_keeper` | `draft_but_implemented` | Stable policy contract/schema docs already exist under `docs/reference/policy/**`, broker and shell config/policy models implement the surface, and multiple later ADRs still treat ADR-0027 as the root contract. |
| ADR-0040 | `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md` | `stable_keeper` | `draft_but_implemented` | The gateway operator contract is already published under `docs/contracts/gateway/operator-contract.md`, and shell/world-service lifecycle code plus tests implement the named gateway command family and ownership boundary. |
| ADR-0041 | `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md` | `stable_keeper` | `draft_but_implemented` | Stable gateway backend-selection docs exist, `llm.routing.default_backend` is implemented in config models, and gateway runtime code already carries adapter-style backend bindings beyond a planning-only statement. |
| ADR-0042 | `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md` | `stable_keeper` | `draft_but_implemented` | The tuple and placement-posture surfaces are already reflected in gateway lifecycle/status code and tests, and later docs treat ADR-0042 as the semantic owner of tuple meaning. |
| ADR-0043 | `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md` | `stable_keeper` | `draft_but_implemented` | Stable tuple-constraint docs exist under `docs/reference/policy/tuple_constraints.md`, and broker/shell policy models already parse and validate `llm.constraints.*`. |
| ADR-0046 | `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md` | `stable_keeper` | `draft_but_implemented` | Although written as a thin implementation follow-on, the gateway runtime and shell lifecycle code already implement inventory-backed default-backend handling, integrated auth shaping, and multi-backend runtime wiring. |

## First-Cluster Decision

The first cluster belongs in `docs/adr/implemented/`, not in a long-lived draft bucket.

Rationale:

- Each ADR still defines active product/runtime truth.
- Each ADR already has stable downstream references, code touchpoints, or both.
- Leaving them under `docs/project_management/adrs/draft/**` preserves an inaccurate signal about
  implementation maturity and keeps stable references pointed at a namespace scheduled for
  retirement.

## Promoted In This Slice

The following curated ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0027-llm-and-agent-config-policy-surface.md`
- `docs/adr/implemented/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/adr/implemented/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/adr/implemented/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/adr/implemented/ADR-0046-gateway-backend-selection-runtime-integration.md`

## Explicit Non-Decisions In This Slice

- This ledger does not yet classify the full ADR registry.
- The legacy project-management ADR files remain in place with relocation notes; this slice does
  not yet archive or delete those historical source files.
- This ledger still leaves the broader `docs/project_management/**` dependency cleanup for later
  retirement slices after current ADR curation is complete.

## Second Cluster: Superseded Gateway and Agent-Hub Predecessors

This is the next bounded cluster after live-consumer prerequisite repointing because these ADRs
still carry legacy backlinks but are no longer the active architectural truth.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0023 | `docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md` | `superseded` | `still_draft` | Its gateway-capability intent is preserved historically, but the current runtime ownership split now lives in ADR-0040 and the adapter boundary in ADR-0041. |
| ADR-0024 | `docs/project_management/adrs/draft/ADR-0024-cli-backend-provider-engine.md` | `superseded` | `still_draft` | The stable backend-id and allowlisting goals survived, but the Substrate-local engine framing was replaced by the gateway-owned adapter contract in ADR-0041. |
| ADR-0025 | `docs/project_management/adrs/draft/ADR-0025-agent-hub-core-role-swappable.md` | `superseded` | `still_draft` | The core Agent Hub direction remains relevant, but the newer successor ADRs changed the identity and backend semantics enough that this draft is historical, not current truth. |

## Promoted In This Slice

The following curated historical ADRs now exist under `docs/adr/historical/`:

- `docs/adr/historical/ADR-0023-in-world-llm-gateway-front-door.md`
- `docs/adr/historical/ADR-0024-cli-backend-provider-engine.md`
- `docs/adr/historical/ADR-0025-agent-hub-core-role-swappable.md`

## Queued Draft: ADR-0026

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0026 | `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md` | `stable_keeper` | `still_draft` | The toolbox concept is still queued work rather than settled history. It has not been landed, and when implementation is ready it should be rewritten against the later orchestration, identity, and trace semantics rather than treated as an already-closed predecessor. |

Curated queued draft ADR:

- `docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md`

## Third Cluster: Orchestration and Workflow ADRs

This is the next current ADR cluster after the first contract-heavy promotion and predecessor
cleanup. It groups the accepted execution/trace contracts that are already implemented plus the
active queued orchestration/workflow follow-ons that still need restatement before landing.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0017 | `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` | `stable_keeper` | `implemented` | Accepted and materially implemented in REPL/output-routing behavior; still used as a live foundation for orchestration and trace-related work. |
| ADR-0028 | `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` | `stable_keeper` | `implemented` | Accepted and implemented through the canonical tracing stack and stable trace internals docs. |
| ADR-0047 | `docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md` | `stable_keeper` | `draft_but_implemented` | Its durable host-session and terminal-delivery posture is already treated as current runtime truth and backed by runtime/test anchors. |
| ADR-0021 | `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md` | `stable_keeper` | `still_draft` | Still queued architectural input for a future workflow runtime; not implemented and should remain draft. |
| ADR-0022 | `docs/project_management/adrs/draft/ADR-0022-forge-agent-loop-as-workflow-node.md` | `stable_keeper` | `still_draft` | Still queued as a workflow-node derivative of the broader workflow-engine direction. |
| ADR-0026 | `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md` | `stable_keeper` | `still_draft` | Queued toolbox work that still needs a rewrite before implementation; active draft, not history. |
| ADR-0029 | `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md` | `stable_keeper` | `still_draft` | Queued host-router/service direction that remains active architectural input but not landed behavior. |
| ADR-0044 | `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md` | `stable_keeper` | `still_draft` | Queued successor Agent Hub contract that should remain draft until the orchestration/session stack is restated and landed. |
| ADR-0045 | `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md` | `stable_keeper` | `still_draft` | Queued toolbox successor contract that depends on the surrounding orchestration and identity work. |

## Promoted In This Slice

The following curated implemented ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md`

The following curated draft ADRs now exist under `docs/adr/draft/`:

- `docs/adr/draft/ADR-0021-substrate-workflow-engine.md`
- `docs/adr/draft/ADR-0022-forge-agent-loop-as-workflow-node.md`
- `docs/adr/draft/ADR-0026-orchestration-toolbox-mcp.md`
- `docs/adr/draft/ADR-0029-host-event-bus-and-router-daemon.md`
- `docs/adr/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

## Fourth Cluster: Remaining Current ADR Tail

This tail slice captures the remaining current non-provisioning ADRs that still matter after the
contract-heavy, predecessor, and orchestration/workflow clusters were promoted.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0016 | `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md` | `stable_keeper` | `draft_but_implemented` | World-first REPL semantics are already implemented and still anchor later output-routing and tracing work, so this belongs in the implemented tree despite the legacy draft label. |
| ADR-0019 | `docs/project_management/adrs/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md` | `stable_keeper` | `still_draft` | Still queued UX work around config visibility and scope messaging; active input, not landed behavior. |
| ADR-0020 | `docs/project_management/adrs/draft/ADR-0020-profiles-config-policy-snapshots.md` | `stable_keeper` | `still_draft` | Still queued architecture for full profile snapshots and surface scoping; not implemented and should remain draft. |

## Promoted In This Slice

The following curated implemented ADR now exists under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0016-world-first-repl-persistent-pty.md`

The following curated draft ADRs now exist under `docs/adr/draft/`:

- `docs/adr/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`
- `docs/adr/draft/ADR-0020-profiles-config-policy-snapshots.md`

## Fifth Cluster: Provisioning-Time System-Package ADRs

This slice curates the remaining current provisioning ADR pair. Both decisions are already
implemented in the world-deps operator contract, runtime fail-early behavior, inventory schema,
and guest-world provisioning flow, so they belong in the implemented tree rather than in a
long-lived draft bucket.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0030 | `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md` | `stable_keeper` | `draft_but_implemented` | The explicit `substrate world enable --provision-deps` contract, runtime fail-early posture, and no-host-mutation guarantee are already implemented and documented in stable world-deps references. |
| ADR-0033 | `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md` | `stable_keeper` | `draft_but_implemented` | Manager-aware provisioning and `install.method=pacman` are already implemented in the inventory/runtime stack and stable world-deps docs, so this ADR is no longer only queued rationale. |

## Promoted In This Slice

The following curated implemented ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0030-provisioning-time-system-package-mutation-for-world-deps.md`
- `docs/adr/implemented/ADR-0033-manager-aware-system-package-provisioning-for-world-deps.md`

## Sixth Cluster: Config and Policy Foundation ADRs

This slice curates the already-implemented config and policy foundation ADRs that stable operator
and internals docs still use as current semantic anchors.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0003 | `docs/project_management/adrs/implemented/ADR-0003-policy-and-config-mental-model-simplification.md` | `stable_keeper` | `implemented` | Stable config docs still depend on its canonical file-name, workspace, and terminology model. |
| ADR-0005 | `docs/project_management/adrs/implemented/ADR-0005-workspace-config-precedence-over-env.md` | `stable_keeper` | `implemented` | The current config/env precedence contract is already exposed in stable config and env docs. |
| ADR-0006 | `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md` | `stable_keeper` | `implemented` | The supported env-variable contract and inventory still depend on its taxonomy and override split. |
| ADR-0008 | `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md` | `stable_keeper` | `implemented` | It defines the shared patch-file scope model that current config, policy, and world-deps docs still rely on. |
| ADR-0012 | `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md` | `stable_keeper` | `implemented` | Current config and world-deps docs rely on its merge-strategy and provenance contract. |
| ADR-0013 | `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` | `stable_keeper` | `implemented` | Stable policy and config docs depend on broker-canonical patch-only policy resolution semantics. |

## Promoted In This Slice

The following curated implemented ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0003-policy-and-config-mental-model-simplification.md`
- `docs/adr/implemented/ADR-0005-workspace-config-precedence-over-env.md`
- `docs/adr/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/adr/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- `docs/adr/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

## Seventh Cluster: World and Runtime Foundation ADRs

This slice curates the remaining implemented world/runtime foundation ADRs that stable docs or
runtime contracts still use as semantic anchors.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0004 | `docs/project_management/adrs/implemented/ADR-0004-world-overlayfs-directory-enumeration-reliability.md` | `stable_keeper` | `implemented` | Stable world and trace docs still depend on its filesystem-strategy fallback and observability semantics. |
| ADR-0007 | `docs/project_management/adrs/implemented/ADR-0007-host-and-world-doctor-scopes.md` | `stable_keeper` | `implemented` | Current installation, command, and platform docs depend on its host/world doctor split and readiness semantics. |
| ADR-0014 | `docs/project_management/adrs/implemented/ADR-0014-world-service-policy-resolution-and-concurrency.md` | `stable_keeper` | `implemented` | It defines the host-resolved policy snapshot authority that current world execution and trace semantics still rely on. |
| ADR-0015 | `docs/project_management/adrs/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md` | `stable_keeper` | `implemented` | Stable full-isolation docs still rely on its allowlisted-write correctness and overlay backing-dir semantics. |
| ADR-0018 | `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` | `stable_keeper` | `implemented` | Current world/config internals still depend on its granular filesystem policy and hardened deny posture. |

## Promoted In This Slice

The following curated implemented ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
- `docs/adr/implemented/ADR-0007-host-and-world-doctor-scopes.md`
- `docs/adr/implemented/ADR-0014-world-service-policy-resolution-and-concurrency.md`
- `docs/adr/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
- `docs/adr/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Eighth Cluster: World-Deps Predecessor and Current Contract ADRs

This slice resolves the remaining world-deps predecessor pair by separating stale historical
framing from the still-current inventory and enabled-set contract.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0002 | `docs/project_management/adrs/implemented/ADR-0002-world-deps-install-classes-and-world-provisioning.md` | `historical_only` | `implemented` | It established the early install-class and provisioning posture, but its command surface and selection-file model are no longer accurate. |
| ADR-0011 | `docs/project_management/adrs/implemented/ADR-0011-world-deps-packages-bundles-contract.md` | `stable_keeper` | `implemented` | It is still the active inventory-directory plus enabled-patch contract behind current world-deps docs and runtime behavior. |

## Promoted In This Slice

The following curated ADRs now exist under `docs/adr/**`:

- `docs/adr/historical/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- `docs/adr/implemented/ADR-0011-world-deps-packages-bundles-contract.md`

## Ninth Cluster: Remaining Installer, Diagnostics, and Backend ADR Tail

This slice classifies the remaining legacy-only ADR tail so the stable `docs/adr/**` registry
fully covers the current ADR set even where some entries remain active drafts.

| ADR | Current path | Curation disposition | Implementation posture | Why it stays or moves |
| --- | --- | --- | --- | --- |
| ADR-0009 | `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md` | `stable_keeper` | `still_draft` | It is still an active backend/provisioning direction, but the guest-rootfs backend contract is not yet the current landed runtime baseline. |
| ADR-0010 | `docs/project_management/adrs/draft/ADR-0010-world-backend-contract-and-capability-divergence.md` | `stable_keeper` | `still_draft` | It remains the queued cross-backend contract for surfacing capability divergence without claiming completed implementation. |
| ADR-0031 | `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md` | `stable_keeper` | `draft_but_implemented` | Linux installer distro/package-manager discovery and explicit override behavior are already implemented in install flows, tests, and installation docs. |
| ADR-0032 | `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` | `stable_keeper` | `draft_but_implemented` | Linux install-state persistence for detected distro/package-manager metadata is already implemented and verified by installer tests. |
| ADR-0034 | `docs/project_management/adrs/draft/ADR-0034-staging-beaver.md` | `stable_keeper` | `draft_but_implemented` | Dev-install helper staging under `SUBSTRATE_HOME` is already implemented in install scripts and world-enable runtime lookup paths. |
| ADR-0035 | `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md` | `stable_keeper` | `draft_but_implemented` | The “install with `--no-world`, enable later” dev workflow is already materially implemented in installer/runtime behavior and smoke coverage. |
| ADR-0036 | `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md` | `stable_keeper` | `draft_but_implemented` | Health and shim-doctor now treat `world.enabled: false` as a first-class disabled state rather than a failure. |
| ADR-0037 | `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` | `stable_keeper` | `draft_but_implemented` | Doctor/health diagnostics already attribute the highest-precedence reason that world isolation is disabled. |
| ADR-0038 | `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md` | `stable_keeper` | `draft_but_implemented` | Replay warnings already reuse world-disabled reason attribution instead of implying `--no-world` generically. |
| ADR-0039 | `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md` | `stable_keeper` | `still_draft` | It remains an active installer-metadata direction for macOS but is not yet a completed stable runtime contract. |
| ADR-2026-02-13 | `docs/project_management/adrs/draft/ADR-2026-02-13-macos-world-backend-virtualization-framework.md` | `stable_keeper` | `still_draft` | It is still a proposed backend direction rather than current macOS backend truth. |

## Promoted In This Slice

The following curated implemented ADRs now exist under `docs/adr/implemented/`:

- `docs/adr/implemented/ADR-0031-best-effort-linux-distro-package-manager-discovery-during-install.md`
- `docs/adr/implemented/ADR-0032-persist-linux-distro-package-manager-detection-in-install-state.md`
- `docs/adr/implemented/ADR-0034-stabilize-dev-install-helper-discovery-under-substrate-home.md`
- `docs/adr/implemented/ADR-0035-make-substrate-world-enable-work-after-dev-install-no-world.md`
- `docs/adr/implemented/ADR-0036-world-disabled-first-class-status-in-health-and-shim-doctor.md`
- `docs/adr/implemented/ADR-0037-doctor-health-attribute-why-world-is-disabled.md`
- `docs/adr/implemented/ADR-0038-replay-attribute-why-world-is-disabled-in-warnings.md`

The following curated draft ADRs now exist under `docs/adr/draft/`:

- `docs/adr/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
- `docs/adr/draft/ADR-0010-world-backend-contract-and-capability-divergence.md`
- `docs/adr/draft/ADR-0039-persist-macos-host-os-details-in-install-state.md`
- `docs/adr/draft/ADR-2026-02-13-macos-world-backend-virtualization-framework.md`

## Next Resume Slice

The ADR registry is now fully classified into `docs/adr/**`. Legacy copies remain intentionally
under `docs/project_management/adrs/**` as compatibility and historical breadcrumbs. Next,
continue with:

1. narrow any remaining `docs/project_management/**` dependency surface that still points stable
   readers at retiring namespaces
2. treat leftover legacy ADR files as retained history/stubs rather than unmigrated registry gaps
3. do not reopen already-curated ADR clusters unless new stable references are discovered
