# TASKS-01: Supported Mode and Support Taxonomy

Source spec:
- [`SPEC-01-supported-mode-and-support-taxonomy.md`](./SPEC-01-supported-mode-and-support-taxonomy.md)

Source plan:
- [`PLAN-01.md`](./PLAN-01.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`  
Status: draft task set  
Execution model: three sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts the Slice `01` seam,
2. the support-contract scope stays limited to taxonomy and supported-mode
   language,
3. the implementation owner agrees that version-floor and external-semantic
   freeze work belongs to Slice `02` or later.

## Slice contract

This slice should land the first implementation-facing contract for:

1. supported same-user Lima mode
2. explicit Linux non-parity language
3. one stable taxonomy for:
   - `supported`
   - `degraded-but-supported`
   - `breakglass`

This slice must **not**:

1. freeze Lima version semantics
2. implement transport or policy changes
3. implement mount, listener, or unit changes
4. implement operator lifecycle commands
5. silently widen into top-level repo-wide macOS docs unless the scope is
   explicitly re-approved

Default execution boundary:

1. feature-local docs under `macos-hardening/macos-hardened-same-user-lima/`
2. Phase `0` docs inside the same feature directory

Treat edits to `docs/WORLD.md` or
`docs/reference/world/platforms/macos-lima-setup.md` as scope expansion unless
the orchestrator can point to a direct contradiction that the user has approved
to fix in this slice.

## Execution packets

### Packet 1: Supported-mode contract freeze

Session goal:

1. freeze the supported same-user Lima posture
2. freeze the explicit Linux non-parity language
3. confirm the normal operator path language

#### Tasks

- [ ] Task 1.1: Confirm the slice authority stack and freeze the posture language
  - Acceptance: the implementation pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, Phase `0`, milestone `0.1`, and
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and it lands one
    consistent description of the supported same-user Lima posture without
    widening into version-floor or transport-specific semantics.
  - Verify:
    - manual authority review
    - `sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/README.md`
    - `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md` only if the wording needs slice-local clarification
    - `macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md` only if bounded refinement is needed

- [ ] Task 1.2: Make the Linux non-parity claims impossible to miss
  - Acceptance: the touched docs clearly state that same-user Lima is not Linux
    host-side ownership parity, is not a privilege boundary against the owning
    host user, and does not treat direct guest administration as the normal
    path.
  - Verify:
    - `rg -n "host-side ownership|privilege boundary|direct guest|normal operator path" macos-hardening/macos-hardened-same-user-lima`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/README.md`
    - `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/README.md`
    - `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the supported same-user posture is explicit,
2. Linux non-parity claims are explicit,
3. no version-floor or transport-specific claims were frozen by accident,
4. the touched-doc set stayed within the default execution boundary unless the
   user approved an expansion.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Support taxonomy freeze and feature-local propagation

Session goal:

1. define the support taxonomy exactly
2. propagate the taxonomy through the most visible feature-local docs

#### Tasks

- [ ] Task 2.1: Freeze the support taxonomy terms and examples
  - Acceptance: the slice defines `supported`, `degraded-but-supported`, and
    `breakglass` exactly once in authoritative wording and uses those terms
    consistently in the touched feature-local docs.
  - Verify:
    - `rg -n "supported|degraded-but-supported|breakglass" macos-hardening/macos-hardened-same-user-lima`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md`
    - `macos-hardening/macos-hardened-same-user-lima/README.md`
    - `macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md` only if the canonical terminology block must be synchronized there

- [ ] Task 2.2: Align the roadmap and phase-local wording to the taxonomy
  - Acceptance: the roadmap and phase-local docs no longer imply conflicting
    support-boundary categories and future slice planning can rely on the same
    vocabulary without reinterpretation.
  - Verify:
    - manual diff review
    - `rg -n "supported|degraded-but-supported|breakglass" macos-hardening/macos-hardened-same-user-lima/ROADMAP.md macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/ROADMAP.md`
    - `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/README.md`
    - `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. the support-taxonomy terms are consistent,
2. feature-local execution docs use the same taxonomy,
3. the slice still remains feature-local and docs-first,
4. the next-slice handoff language still points cleanly to Slice `02`.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Validation, explicit deferrals, and next-slice readability

Session goal:

1. validate that Slice `01` stayed narrow
2. make Slice `02` ownership explicit
3. leave the short-prompt planning path clean

#### Tasks

- [ ] Task 3.1: Validate explicit deferrals to later slices
  - Acceptance: the touched docs explicitly leave version-floor work to Slice
    `02` and leave transport, policy, mount, unit, and lifecycle work to later
    slices rather than implying those seams are already resolved.
  - Verify:
    - `rg -n "Slice \`02\`|version floor|transport|policy|mount|unit|lifecycle" macos-hardening/macos-hardened-same-user-lima`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md`
    - `macos-hardening/macos-hardened-same-user-lima/ROADMAP.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md`

- [ ] Task 3.2: Final diff and coherence review
  - Acceptance: the final diff is narrow, terminology is consistent, and a
    future short prompt can identify the next honest slice from the landed
    materials without reopening the Slice `01` support posture.
  - Verify:
    - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima`
    - manual coherence review
  - Files:
    - all files touched by this slice only as required by final cleanup

### Packet 3 checkpoint

Packet `3` is complete only when:

1. Slice `01` stayed bounded,
2. Slice `02` remains legible as the next seam,
3. the feature-local planning stack is coherent enough for a short future
   prompt to continue the sequence,
4. the packet prompt artifact for Slice `01` can be generated without guessing
   packet scope or review/fix flow.

## Prompt artifact expectation

After Packet `3` planning is stable, the slice should have one ready-to-paste
prompt artifact for fresh orchestration sessions. That prompt artifact should:

1. provide one prompt per packet,
2. require a fresh GPT-5.4 high implementation subagent using
   `$incremental-implementation`,
3. require a fresh GPT-5.4 high review subagent using
   `$code-review-and-quality`,
4. require fix subagents when review finds issues,
5. require commits between implementation, review-driven fix rounds, and the
   next packet boundary.
