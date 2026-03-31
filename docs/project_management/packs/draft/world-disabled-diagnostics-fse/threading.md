# Threading - World Disabled Diagnostics

## Execution horizon summary

- **Active seam**: `SEAM-2`
  - first publishing seam for disabled/skipped runtime contracts, now revalidated against the published `THR-01` handoff
- **Next seam**: `SEAM-3`
  - queued consumer of `SEAM-2`'s shim status contracts; remains provisional until `THR-02` and `THR-03` publish
- **Future seams**: `SEAM-1`, `SEAM-4`
  - `SEAM-1` has left the forward window with a passed seam-exit gate
  - `SEAM-4` remains a seam brief until shim and health contracts land

## Source basis carried forward from the deep-researched pack

- **Contract basis**:
  - `contract.md`
  - `decision_register.md`
  - `world-disabled-diagnostics-json-schema-spec.md`
- **Execution basis**:
  - `slices/WDD0/WDD0-spec.md`
  - `slices/WDD1/WDD1-spec.md`
  - `slices/WDD2/WDD2-spec.md`
  - `tasks.json`
- **Validation basis**:
  - `manual_testing_playbook.md`
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
- **Cross-queue / risk basis**:
  - `pre-planning/impact_map.md`
  - `pre-planning/workstream_triage.md`
  - `pre-planning/ci_checkpoint_plan.md`

## Contract registry

- **Contract ID**: `C-01`
  - **Type**: config
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**: `SEAM-2`, `SEAM-3`
  - **Derived consumers**: `SEAM-4`; future attribution, json-envelope, and provisioning-related diagnostics work
  - **Thread IDs**: `THR-01`
  - **Definition**: Shared diagnostics-side resolution of effective `world.enabled` via the existing effective-config resolver, including CLI override mapping and fail-fast exit-2 posture on config-resolution failure.
  - **Versioning / compat**: no new config keys or environment variables; consumers must not infer enabled/disabled locally when the resolver fails.

- **Contract ID**: `C-02`
  - **Type**: schema
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`, `SEAM-4`
  - **Derived consumers**: JSON automation, `json-mode`, and future attribution work
  - **Thread IDs**: `THR-02`
  - **Definition**: Canonical world backend status enum at `.world.status` / `.shim.world.status` with values `healthy | needs_attention | disabled | unknown`.
    - Disabled mode publishes `.world.status = "disabled"` and short-circuits all world backend probes before any `substrate world doctor --json` subprocess or equivalent backend call.
    - Enabled healthy mode publishes `.world.status = "healthy"`.
    - Enabled failure mode publishes `.world.status = "needs_attention"` and preserves enabled-mode diagnostics.
    - `unknown` is reserved for genuinely unavailable classification surfaces, not for disabled-by-choice mode.
  - **Versioning / compat**: additive-only field; no renames/removals; downstream consumers must ignore unknown enum values.

- **Contract ID**: `C-03`
  - **Type**: schema
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-3`, `SEAM-4`
  - **Derived consumers**: provisioning-related diagnostics work and JSON automation
  - **Thread IDs**: `THR-03`
  - **Definition**: Canonical world-deps status enum at `.world_deps.status` / `.shim.world_deps.status` with values `ok | error | skipped_disabled | unknown`, plus disabled-mode omission of probe-derived legacy fields.
    - Disabled mode publishes `.world_deps.status = "skipped_disabled"` and must not compute applied world-deps state.
    - Disabled mode omits probe-derived fields that would imply a real probe occurred, specifically disabled-path `world.error`, `world.details`, `world_deps.error`, and `world_deps.report`.
    - Enabled mode may publish `.world_deps.status = "ok"` or `.world_deps.status = "error"` according to the real applied-state result.
  - **Versioning / compat**: additive-only field; disabled-mode omission is canonical and must not be backfilled by downstream consumers.

- **Contract ID**: `C-04`
  - **Type**: UX affordance
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**: `SEAM-4`
  - **Derived consumers**: docs/examples and future copy-attribution work
  - **Thread IDs**: `THR-04`
  - **Definition**: Exact disabled-mode `substrate shim doctor` lines, no `Error:` lines for disabled/skipped states, and a no-probe operator posture.
    - The disabled-mode text contract is:
      - `World backend:`
      - `  Status: disabled`
      - `  Next: run \`substrate world enable\` to provision`
      - `World deps:`
      - `  Status: skipped (world disabled)`
    - Disabled or skipped states must not print `Error:` lines.
    - Enabled-mode failure text remains fail-visible and must not collapse into disabled wording.
  - **Versioning / compat**: exact-line contract is intentionally small and explicit; enabled-mode copy remains flexible.

- **Contract ID**: `C-05`
  - **Type**: UX affordance
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**: `SEAM-4`
  - **Derived consumers**: `docs/USAGE.md`, future provisioning packs, and human operators who read `substrate health`
  - **Thread IDs**: `THR-05`
  - **Definition**: Disabled-mode `substrate health` copy and summary contract: `summary.world_ok = null`, omitted summary error fields, empty world-deps arrays, and suppression of enabled-world guidance when disabled.
  - **Versioning / compat**: additive-only summary posture; enabled-mode guidance and failure aggregation remain unchanged unless explicitly amended.

## Thread registry

- **Thread ID**: `THR-01`
  - **Producer seam**: `SEAM-1`
  - **Consumer seam(s)**: `SEAM-2`, `SEAM-3`
  - **Carried contract IDs**: `C-01`
  - **Purpose**: ensure both diagnostics commands branch from one authoritative effective-config decision and one config-error posture.
  - **State**: revalidated
  - **Revalidation trigger**: effective-config precedence changes, workspace override-ignore behavior changes, or diagnostics routing changes for exit-code `2`.
  - **Satisfied by**: `governance/seam-1-closeout.md` records the published resolver-backed helper plus invalid-config fail-fast evidence, and `threaded-seams/seam-2-shim-doctor-disabled-aware-reporting/review.md` revalidates shim-doctor planning against that closeout-backed handoff.
  - **Notes**: `SEAM-2` has now rebound to the published `C-01` truth as the active seam. `SEAM-3` still consumes the same handoff later alongside `THR-02` and `THR-03`.

- **Thread ID**: `THR-02`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`, `SEAM-4`
  - **Carried contract IDs**: `C-02`
  - **Purpose**: publish the canonical world backend status enum so downstream summary logic and external tooling can distinguish disabled from broken.
  - **State**: defined
  - **Revalidation trigger**: JSON envelope/field-shape changes, added attribution fields, or shim payload restructuring.
  - **Satisfied by**: landed shim-doctor JSON/text rendering plus tests asserting `.world.status` under disabled, healthy, and needs-attention cases.
  - **Notes**: future JSON envelope work must preserve this field path inside any wrapper.

- **Thread ID**: `THR-03`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-3`, `SEAM-4`
  - **Carried contract IDs**: `C-03`
  - **Purpose**: carry world-deps status and omission semantics into health aggregation and cross-platform conformance.
  - **State**: defined
  - **Revalidation trigger**: world-deps report shape changes, provisioning guidance changes, or any disabled-mode code path that reintroduces probe-backed error fields.
  - **Satisfied by**: landed `.world_deps.status` emission and omission tests for disabled mode.
  - **Notes**: this thread exists specifically to prevent health from misclassifying skipped-disabled as an error.

- **Thread ID**: `THR-04`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**: `SEAM-4`
  - **Carried contract IDs**: `C-04`
  - **Purpose**: lock the exact shim-doctor disabled-mode operator experience and prove the no-probe boundary with platform-parity evidence.
  - **State**: defined
  - **Revalidation trigger**: copy changes, probe path refactors, or any new world-backend call on the disabled path.
  - **Satisfied by**: landed text renderer, exact-line assertions, and evidence that disabled-mode paths do not spawn backend or world-deps probes.
  - **Notes**: this thread should not close until conformance evidence exists on Linux/macOS/Windows.

- **Thread ID**: `THR-05`
  - **Producer seam**: `SEAM-3`
  - **Consumer seam(s)**: `SEAM-4`
  - **Carried contract IDs**: `C-05`
  - **Purpose**: carry `substrate health` summary/copy semantics into smoke evidence, docs alignment, and pack closeout.
  - **State**: defined
  - **Revalidation trigger**: summary aggregation logic changes, `docs/USAGE.md` drift, or provisioning packs that modify enabled-mode guidance surfaces.
  - **Satisfied by**: landed health summary behavior, docs updates, and smoke assertions across all required platforms.
  - **Notes**: future packs touching `health.rs` must explicitly revalidate this thread.

## Dependency graph

```mermaid
flowchart LR
  S1[SEAM-1\nclassifier + config errors] -->|THR-01| S2[SEAM-2\nshim doctor contracts]
  S1 -->|THR-01| S3[SEAM-3\nhealth summary + docs]
  S2 -->|THR-02| S3
  S2 -->|THR-03| S3
  S2 -->|THR-04| S4[SEAM-4\ncross-platform conformance]
  S3 -->|THR-05| S4
```

## Critical path

1. `SEAM-1` has now published `C-01` and the `THR-01` handoff through a passed seam-exit gate.
2. `SEAM-2` is the active seam and must publish `C-02`, `C-03`, and `C-04` so the health command can branch on canonical status enums rather than legacy booleans/strings.
3. `SEAM-3` remains the queued next seam and must publish `C-05` so pack-level validation and operator docs can lock the final disabled-mode summary posture.
4. `SEAM-4` still closes the loop with Linux/macOS/Windows evidence and becomes the pack-level proof point that adjacent packs must consume.

## Workstreams

- **Foundation workstream**
  - `SEAM-1`
  - Closed foundation seam; its published handoff is now consumed by the active window.

- **Shim reporting workstream**
  - `SEAM-2`
  - Active execution seam; its basis is revalidated against the published `THR-01` handoff.

- **Health + docs workstream**
  - `SEAM-3`
  - Queued next seam; should wait for published shim status contracts so summary logic does not speculate about disabled-mode semantics.

- **Conformance workstream**
  - `SEAM-4`
  - Must consume landed reality from prior seams; it should not invent unpublished behavior.
