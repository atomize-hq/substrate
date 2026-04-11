# FSE Pre-Planning Stage Output Contract

## Status

Proposed replacement contract for the current pre-planning lane under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/fse`.

## Grounding

This proposal is anchored to the current FSE pre-planning standards and prompts:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/fse/standards/planning/PLANNING_PRE_PLANNING_RESEARCH_WRAPPER.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/fse/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/fse/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

It is also grounded in the v2.5 downstream skill contracts that consume or advance a seam pack:

- `/Users/spensermcconnell/.agents/skills/feature-seam-extractor-v2-5/SKILL.md`
- `/Users/spensermcconnell/.agents/skills/threaded-seam-decomposer-v2-5/SKILL.md`
- `/Users/spensermcconnell/.agents/skills/seam-promotion-v2-5/SKILL.md`
- `/Users/spensermcconnell/.agents/skills/seam-execution-v2-5/SKILL.md`

## Intent

The revised pre-planning lane keeps the current narrow-agent, overlap-safe operating model, but the canonical tracked output is no longer a six-doc pre-planning pack. The canonical tracked output is a real v2.5 seam pack that downstream v2.5 tooling can consume without a translation pass.

The control-plane target for one feature is:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/README.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/scope_brief.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/spec_manifest.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/seam_map.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/threading.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/review_surfaces.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/seam-<n>-<slug>.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/governance/remediation-log.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/governance/seam-<n>-closeout.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/governance/pack-closeout.md`

The pre-planning lane must not create:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/threaded-seams/`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/threaded-seams/**/review.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/threaded-seams/**/slice-*.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/threaded-seams/**/subslice-*.md`
- legacy canonical pre-planning outputs under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/pre-planning/*.md`

## Canonical and Research-Only Lanes

Canonical truth:

- the v2.5 seam-pack files listed above
- only the owning stage may write its canonical outputs
- downstream stages must re-read canonical outputs before final promotion

Research-only evidence:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/<stage>/scratch.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/<stage>/handoff.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/<stage>/staged/**`
- optional compatibility renderings of the old six artifacts, but only under `logs/pre-planning-v2_5/compat/`

Research-only outputs may overlap aggressively. Canonical writes must remain single-owner and gate-checked.

## Hyper-Focused Agent Rule

The pipeline keeps canonical ownership narrow and singular, but it may use helper research stages when that keeps each agent tightly scoped.

Rules:

- helper stages may write only research-only outputs under their own `logs/pre-planning-v2_5/<stage>/` subtree
- helper stages may not write canonical seam-pack files
- the owning canonical stage must re-read and reconcile helper outputs before promotion
- helper stages do not create a second control plane

Recommended helper stages:

- `pp1a-scope-intake`: restates user, goal, success, constraints, and risk posture
- `pp1b-surface-authority`: derives exact durable contract homes, doc ownership, and explicit surface inventory
- `pp3a-threading`: derives contract registry, thread registry, dependency graph, and critical path
- `pp3b-verification-cadence`: derives checkpoint intent, platform proof grouping, and any conformance-heavy workstream notes

These helpers exist to preserve narrow-agent focus. They do not change the canonical file ownership listed below.

## Stage Contract

### PP0 - Bootstrap and Input Freeze

Purpose:

- resolve the ADR set
- create or confirm the feature pack root
- establish the exact run input set for downstream stages

Owned canonical outputs:

- none in the v2.5 seam pack
- existing metadata such as `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/fse_pre_planning.json` may remain as wrapper metadata only

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp0-bootstrap/input_digest.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp0-bootstrap/handoff.md`

Non-owned outputs:

- every v2.5 seam-pack file

Gate to complete:

- ADR inputs are resolved to exact paths
- the feature directory exists
- the run input digest names repo evidence, ADRs, and adjacent planning packs to scan

Notes:

- PP0 exists so later stages do not infer a moving input set.

### PP1 - Scope Brief

Purpose:

- convert ADR intent into the canonical feature brief expected by the extractor contract
- merge the outputs of the optional `pp1a-scope-intake` and `pp1b-surface-authority` helper stages into one canonical pack brief

Owned canonical outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/scope_brief.md`

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp1-scope/scratch.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp1-scope/handoff.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp1-scope/staged/scope_brief.md`

Non-owned outputs:

- `README.md`
- `seam_map.md`
- `threading.md`
- `review_surfaces.md`
- every `seam-<n>-<slug>.md`
- every governance file

Required `scope_brief.md` sections:

- goal and why now
- primary users or operators
- in-scope and out-of-scope
- success criteria
- hard constraints
- external systems and stakeholders
- known unknowns and risks
- explicit note that `spec_manifest.md` is the authoritative authored-doc-class inventory and surface-ownership register for the feature

Gate to promote canonical output:

- feature scope, goals, constraints, and risks are concrete enough to support authored-doc selection
- no task graph, kickoff prompt, or execution ownership surface is introduced

Downstream handoff:

- PP1.5 may begin research when `pp1-scope/handoff.md` exists
- PP2 may begin research when `pp1-scope/handoff.md` exists
- PP2 may not write canonical seam files until both `scope_brief.md` and `spec_manifest.md` are present

Optional helper-stage split:

- `pp1a-scope-intake` may draft user/job/success/constraint sections early
- `pp1b-surface-authority` may run in parallel to derive exact contract homes and surface inventory
- PP1 owns the canonical merge and resolves any mismatch between those helper outputs

### PP1.5 - Spec Manifest and Surface Authority

Purpose:

- keep the current `spec_manifest.md` responsibility as a first-class canonical output
- define the authored spec, contract, schema, policy, parity, compatibility, and validation document classes the feature requires before seam extraction locks the downstream planning shape
- give seam extraction an explicit domain-completeness input instead of forcing seam briefs or `threading.md` to infer missing document classes later

Owned canonical outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/spec_manifest.md`

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp1_5-spec-manifest/scratch.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp1_5-spec-manifest/handoff.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp1_5-spec-manifest/staged/spec_manifest.md`

Non-owned outputs:

- `scope_brief.md`
- `seam_map.md`
- `threading.md`
- `review_surfaces.md`
- every `seam-<n>-<slug>.md`
- every governance file

Required `spec_manifest.md` content:

- exact list of required authored docs for the feature
- explicit doc classes required by the feature, such as:
  - contract
  - protocol
  - schema
  - policy
  - telemetry
  - filesystem semantics
  - platform parity
  - compatibility
  - validation playbook
- one-owner-per-surface mapping
- distinction between:
  - canonical descriptive docs under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/`
  - pack-local planning docs
  - deferred downstream docs that must exist later
- absence semantics and determinism obligations for each selected doc class

Gate to promote canonical output:

- no required spec class remains implicit
- every durable contract surface has one intended canonical home
- seam extraction can proceed without inventing missing spec families later
- no task graph, kickoff prompt, or execution ownership surface is introduced

Downstream handoff:

- PP2 may begin research when `pp1_5-spec-manifest/handoff.md` exists
- PP2 may not write canonical seam files until `spec_manifest.md` is present

### PP2 - Seam Map and Seam Briefs

Purpose:

- replace the current `minimal_spec_draft.md` seam skeleton with real v2.5 seam briefs
- define the seam pack backbone that downstream decomposition expects

Owned canonical outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/seam_map.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/seam-<n>-<slug>.md`

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp2-seams/scratch.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp2-seams/handoff.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp2-seams/staged/seam_map.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp2-seams/staged/seam-<n>-<slug>.md`

Non-owned outputs:

- `scope_brief.md`
- `spec_manifest.md`
- `threading.md`
- `review_surfaces.md`
- `README.md`
- every governance file

Required `seam_map.md` content:

- seam list with stable `SEAM-<n>` identifiers
- seam type and touch surface summary
- boundary rationale
- pack-wide active, next, future horizon statement
- critical-path assumptions that PP3 must either confirm or reject

Required seam-brief posture:

- `status: proposed`
- `execution_horizon: active | next | future`
- exactly one `active` seam and one `next` seam by default
- `basis.currentness: provisional` unless repo evidence is strong enough to mark `current`
- `seam_exit_gate.required: true`
- `seam_exit_gate.planned_location: S99`
- `seam_exit_gate.status: pending`
- empty `open_remediations` by default

Gate to promote canonical outputs:

- seam IDs and slugs are stable
- every seam has value, touch surface, and verification intent
- horizon policy is explicit and matches the v2.5 extractor contract
- seam boundaries are consistent with the authored-doc classes and ownership rules declared in `spec_manifest.md`
- no slices or subslices are created

Downstream handoff:

- PP3 may begin research on `pp2-seams/handoff.md`
- PP4 may begin research on `pp2-seams/handoff.md`
- PP5 may read the seam handoff, but may not write governance canonically until PP3 completes

### PP3 - Threading, Contract Registry, and Dependency Control Plane

Purpose:

- replace `workstream_triage.md` with the v2.5 canonical thread and dependency control plane
- absorb the durable parts of `impact_map.md`, `ci_checkpoint_plan.md`, and the dependency-heavy parts of the old triage surface
- merge the outputs of the optional `pp3a-threading` and `pp3b-verification-cadence` helper stages into one canonical dependency surface

Owned canonical outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/threading.md`

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp3-threading/scratch.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp3-threading/handoff.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp3-threading/staged/threading.md`

Non-owned outputs:

- `scope_brief.md`
- `spec_manifest.md`
- `seam_map.md`
- every seam brief
- `review_surfaces.md`
- `README.md`
- every governance file

Required `threading.md` content:

- execution horizon summary
- contract registry with single producer ownership
- thread registry using `identified | defined | published | revalidated | closed`
- canonical contract refs under `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/<contract-slug>.md` when a durable contract doc is warranted
- dependency graph
- critical path
- workstreams
- revalidation triggers
- `satisfied_by` posture for each thread
- verification cadence and checkpoint intent when platform scope warrants it
- references back to `spec_manifest.md` when a thread carries a contract or doc obligation that depends on a specific authored spec class

Gate to promote canonical output:

- every cross-seam contract has one producer seam
- dependency direction is explicit
- horizon summary agrees with seam briefs
- thread states use the v2.5 vocabulary
- no legacy workstream registry semantics remain

Downstream handoff:

- PP4 may refresh its draft from `pp3-threading/handoff.md`
- PP5 may begin or refresh research from `pp3-threading/handoff.md`
- PP6 may not write `README.md` until `threading.md` is canonical

Optional helper-stage split:

- `pp3a-threading` may focus only on `C-*`, `THR-*`, dependency edges, revalidation triggers, and critical-path structure
- `pp3b-verification-cadence` may focus only on checkpoint intent, platform proof grouping, and conformance-heavy workstream notes
- PP3 owns the canonical `threading.md` merge and resolves any mismatch between those helper outputs

### PP4 - Review Surfaces

Purpose:

- create the pack-level orientation artifact required by the v2.5 extractor
- replace the diagram and impact-heavy parts of `impact_map.md`

Owned canonical outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/review_surfaces.md`

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp4-review/scratch.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp4-review/handoff.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp4-review/staged/review_surfaces.md`

Non-owned outputs:

- `scope_brief.md`
- `seam_map.md`
- every seam brief
- `threading.md`
- `README.md`
- every governance file

Required `review_surfaces.md` posture:

- product-facing or operator-facing Mermaid diagrams
- actual service, API, state, component, or workflow flows
- orientation only, not seam-local pre-exec review
- visible mismatch hotspots and high-risk boundaries

Gate to promote canonical output:

- diagrams and narrative reflect the current canonical seam map and threading
- the document explicitly states that downstream seam-local `review.md` remains a later artifact owned by the threaded seam decomposer

Downstream handoff:

- PP6 may begin assembling its summary from `pp4-review/handoff.md`

### PP5 - Governance Scaffold

Purpose:

- replace `alignment_report.md` with structured v2.5 governance scaffolding
- seed the control plane that seam promotion will later consume

Owned canonical outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/governance/remediation-log.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/governance/seam-<n>-closeout.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/governance/pack-closeout.md`

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp5-governance/scratch.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp5-governance/handoff.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp5-governance/staged/governance/**`

Non-owned outputs:

- every non-governance seam-pack file

Required governance posture:

- seam-only remediation ownership
- remediation entries with `origin_phase`, `source_gate`, `owner_seam`, `blocked_targets`, `summary`, `required_fix`, and `resolution_evidence`
- one closeout stub per seam
- each seam closeout stub seeds the realized `seam_exit_gate` record location for later post-exec use
- `pack-closeout.md` summarizes unresolved remediations, open threads, and stale-trigger expectations

Gate to promote canonical outputs:

- PP2 seam IDs are final
- PP3 thread IDs and contract ownership are final
- every blocker names an owning seam
- no invented post-exec truth is recorded

Downstream handoff:

- PP6 may read `pp5-governance/handoff.md`

### PP6 - README and Pack-Level Validation

Purpose:

- give the pack one stable landing page for downstream operators and v2.5 promotion tools
- close the lane only when the pack already satisfies the downstream input contract

Owned canonical outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/README.md`

Owned research-only outputs:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp6-readme/validation.md`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/<bucket>/<feature>/logs/pre-planning-v2_5/pp6-readme/handoff.md`

Non-owned outputs:

- every other seam-pack file

Required `README.md` content:

- pack summary
- source ADR list
- active seam and next seam
- pack file inventory
- downstream entry points:
  - threaded seam decomposition consumes `scope_brief.md`, `spec_manifest.md`, `seam_map.md`, `threading.md`, `review_surfaces.md`, seam briefs, and governance docs
  - seam promotion consumes `README.md`, `scope_brief.md`, `spec_manifest.md`, `threading.md`, `review_surfaces.md`, seam briefs, and governance docs
- explicit note that this pack ends before `threaded-seams/` planning and execution artifacts

Gate to promote canonical output:

- all required v2.5 seam-pack files exist
- no legacy pre-planning doc under `pre-planning/` is required for truth
- no `threaded-seams/` directory exists yet
- no slices or subslices exist
- horizon policy, thread vocabulary, and governance scaffolds satisfy the v2.5 contracts

## Mapping the Current Six Pre-Planning Artifacts

### 1. `spec_manifest.md`

Current role:

- surface inventory
- canonical doc ownership
- deferred-doc inventory

New canonical home:

- remain canonical as `spec_manifest.md`
- feed seam extraction rather than being replaced by it
- provide the domain-completeness and authored-doc-class inventory that seam briefs and `threading.md` must respect

Why it stays canonical:

- it answers a different question than `scope_brief.md`, `seam_map.md`, or `threading.md`
- it defines which spec, contract, schema, policy, parity, compatibility, and validation artifacts must exist at all
- without it, seam extraction can succeed while still omitting a required document family

### 2. `impact_map.md`

Current role:

- touch set
- cascading implications
- contradiction risk
- cross-queue alignment

New canonical home:

- `seam_map.md` owns touch-surface-by-seam boundaries
- `threading.md` owns the dependency graph, contradiction-prone contract edges, and critical path
- `review_surfaces.md` owns the operator-facing diagrams and mismatch hotspots

Research-only carry-forward:

- any exact path inventory or broad cross-queue scan dump stays in stage logs

### 3. `minimal_spec_draft.md`

Current role:

- cross-cutting defaults
- invariants
- draft seam skeleton

New canonical home:

- `scope_brief.md` owns pack-wide defaults and invariants
- `seam_map.md` and `seam-<n>-<slug>.md` own the real seam skeleton
- `README.md` owns the short pack posture summary

Research-only carry-forward:

- tentative split or merge analysis stays in `pp2-seams` logs

### 4. `ci_checkpoint_plan.md`

Current role:

- advisory checkpoint grouping
- verification cadence

New canonical home:

- `threading.md` owns checkpoint intent, verification cadence, and critical path grouping
- seam closeout stubs and `pack-closeout.md` own the later place where those checkpoints become realized evidence

Research-only carry-forward:

- optional checkpoint what-if analysis stays in `pp3-threading` logs

### 5. `workstream_triage.md`

Current role:

- downstream workstream proposal
- ordering
- split or merge recommendations

New canonical home:

- `threading.md` owns workstreams, dependency direction, and recommended downstream order
- `seam_map.md` owns the accepted seam set after split or merge decisions

Research-only carry-forward:

- draft restructuring analysis stays in `pp2-seams` or `pp3-threading` logs

### 6. `alignment_report.md`

Current role:

- consolidated follow-ups
- hard gates
- unresolved conflicts

New canonical home:

- `governance/remediation-log.md` owns structured open issues and blockers
- `governance/pack-closeout.md` owns pack-level unresolved thread and stale-trigger posture
- `README.md` owns only the summary, not the blocker truth

Research-only carry-forward:

- a human-readable wrapper summary may still be written under `logs/pre-planning-v2_5/compat/alignment_report.md`

## Exit Criteria for the Revised Lane

The revised pre-planning lane is complete only when all are true:

- the pack matches the v2.5 extractor output contract
- the pack is directly consumable by `/Users/spensermcconnell/.agents/skills/threaded-seam-decomposer-v2-5/SKILL.md`
- the pack contains no execution-ready slices, subslices, or seam-local `review.md`
- every canonical file has a single owning stage
- all overlapping work is confined to research-only outputs
- `spec_manifest.md` remains canonical
- the remaining five legacy artifacts are either retired or rendered as research-only compatibility views, not canonical truth
