# Spec: Slice 01 Supported Mode and Support Taxonomy

Source phase authority:
- [`../phase-0-security-contract-and-scope/README.md`](../phase-0-security-contract-and-scope/README.md)
- [`../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`](../phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design input:
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Related research note:
- [`../research/2026-04-28-macos-lima-parity-lockdown.md`](../research/2026-04-28-macos-lima-parity-lockdown.md)

Phase: `SPECIFY`  
Status: draft slice authority  
Slice focus: define the first execution-bearing seam for the hardened same-user
Lima program by freezing the supported-mode language and support taxonomy before
any version-floor or transport-specific slice opens.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `01` is the next honest slice because live repo truth shows the new
   feature-level roadmap, execution rubric, and design docs are present, while
   no prior `SPEC-*`, `PLAN-*`, or `TASKS-*` files exist yet under
   `macos-hardening/macos-hardened-same-user-lima/spec/`.
2. The first execution seam should stay at the support-contract layer rather
   than immediately jumping to Lima-version semantics, transport details, or
   backend policy behavior.
3. This slice is primarily a docs-and-contract slice. It should not widen into
   runtime code changes, provisioning changes, or operator command
   implementation.
4. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice uses
   `spec-driven-development` and only a light source-driven posture. It may use
   the existing repo-local research note as supporting context, but it should
   not block on new official Lima/systemd/Apple source collection unless the
   scope widens beyond taxonomy and support-contract language.
5. The goal of this slice is to make later slices safer and clearer, not to
   claim that the macOS hardening work is already implemented.

If any of these are wrong, correct them before implementation.

## Objective

Freeze one explicit supported-mode and support-taxonomy contract for
`macos-hardened-same-user-lima` so later slices can inherit consistent language
about:

1. what the supported same-user Lima mode is,
2. what Linux parity claims remain valid,
3. what Linux parity claims are explicitly not made,
4. what counts as `supported`,
5. what counts as `degraded-but-supported`,
6. what counts as `breakglass`.

This slice is complete only when a reviewer can read the relevant feature docs
and answer, without guessing:

1. what the normal same-user Lima operator path is,
2. what remains explicitly non-parity with Linux,
3. which workflows are normal, transitional, or breakglass,
4. which later slices still own the version-floor, transport, mount, unit, and
   lifecycle implementation work.

## Why this slice exists

The repo already has a credible phase structure and now also has a feature-level
roadmap, execution rubric, and design-doc scaffold. What it still lacks is the
first bounded execution slice that turns those materials into a reviewable,
implementation-facing contract.

That matters because the current macOS hardening materials still risk three
kinds of drift:

1. feature-level docs, phase docs, and later slice docs could use different
   support-language terms,
2. direct guest flows could remain insufficiently classified even after the
   design taxonomy exists,
3. later slices could reopen the same support-boundary argument instead of
   inheriting one frozen contract.

Slice `01` closes that gap before Slice `02` introduces version-sensitive Lima
semantics and before Phase `1` or Phase `2` slices start freezing transport,
mount, and unit behavior.

## Tech stack

- Rust workspace: `substrate` `0.2.8`
- Rust edition: `2021`
- MSRV: Rust `1.89`
- Primary slice output type: Markdown planning and contract docs under
  `macos-hardening/macos-hardened-same-user-lima/`

This slice is documentation- and planning-first. It should not require changes
to Rust code, shell scripts, or guest unit definitions.

## Commands

This is a docs-and-contract slice. The required commands are repo-inspection and
validation commands, not build or runtime commands.

```bash
# Inspect the feature planning stack
find macos-hardening/macos-hardened-same-user-lima -maxdepth 3 -type f | sort

# Review the phase-0 contract authorities
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md

# Review the execution rubric and design taxonomy authority
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md

# Scan for support-boundary wording that should stay aligned
rg -n "supported|degraded-but-supported|breakglass|SUBSTRATE_WORLD_SOCKET|limactl shell" \
  macos-hardening/macos-hardened-same-user-lima \
  docs/WORLD.md \
  docs/reference/world/platforms/macos-lima-setup.md
```

Optional validation command if implementation broadens beyond the planned docs
set:

```bash
git diff --stat -- macos-hardening/macos-hardened-same-user-lima docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md
```

## Project structure

This slice should stay grounded to these directories and file families:

```text
macos-hardening/macos-hardened-same-user-lima/
├── README.md                                   → feature-level contract
├── EXECUTION-RUBRIC.md                         → next-slice control surface
├── ROADMAP.md                                  → feature-level slice map
├── phase-0-security-contract-and-scope/
│   ├── README.md                               → phase-0 authority
│   └── milestone-0-1-target-mode-and-support-contract-sow.md
└── spec/
    ├── SPEC-01-supported-mode-and-support-taxonomy.md
    ├── PLAN-01.md
    ├── TASKS-01.md
    └── design/
        └── DESIGN-supported-mode-and-breakglass-taxonomy.md

docs/
├── WORLD.md                                    → top-level operator/runtime truth
└── reference/world/platforms/macos-lima-setup.md
                                               → current setup/troubleshooting truth
```

Expected implementation touch surface for Slice `01` should remain small and
primarily documentation-oriented.

## Code style

This slice is Markdown-authority work. The style contract should be:

1. assumptions at the top,
2. explicit supported vs non-goal language,
3. bullet- and matrix-friendly wording,
4. no vague “Linux parity except where different” phrasing,
5. support-taxonomy terms used exactly and consistently.

Example of acceptable slice output style:

```md
## Support taxonomy

- **Supported**: the normal operator path through Substrate-owned surfaces
- **Degraded-but-supported**: transitional or narrower supported paths
- **Breakglass**: exceptional recovery/debugging paths, not the happy path
```

## Testing strategy

This slice does not primarily rely on cargo tests. Its validation is document
coherence and authority alignment.

Validation levels:

1. **Authority review**
   - confirm the slice language matches Phase `0`, milestone `0.1`, and the
     design taxonomy
2. **Terminology scan**
   - verify `supported`, `degraded-but-supported`, and `breakglass` are used
     consistently
3. **Scope validation**
   - confirm the slice does not silently freeze Lima version-floor, transport,
     mount, unit, or launchd semantics that belong to later slices
4. **Diff review**
   - confirm the touched file set remains narrowly bounded to contract docs

## Boundaries

- Always:
  - keep this slice at the support-contract and terminology layer
  - make the same-user limitation explicit
  - preserve the distinction between already-landed operator surfaces and later
    implementation work
  - keep later slice ownership visible for version-floor, transport, mount,
    unit, and lifecycle details
- Ask first:
  - widening this slice into official-Lima-semantic freeze work that belongs to
    Slice `02`
  - widening this slice into transport, mount, unit, or CLI implementation
  - redefining the slice queue from the execution rubric without a repo-truth
    reason
- Never:
  - claim Linux-equivalent host authorization for same-user Lima
  - treat direct guest administration as the supported default
  - silently absorb multiple milestone-scale seams into Slice `01`
  - rely on ambiguous parity wording after this slice lands

## Success criteria

Slice `01` is successful when:

1. the feature-level contract documents define one supported same-user Lima mode
   clearly and consistently,
2. the support taxonomy is explicit and stable across the feature-level
   documents touched by the slice,
3. the docs distinguish `supported`, `degraded-but-supported`, and
   `breakglass` without overlap or fuzzy wording,
4. the slice explicitly states which semantics are still deferred to later
   slices:
   - Lima version floor
   - transport contract details
   - backend policy input parity
   - ingress/mount narrowing
   - guest unit unification
   - operator lifecycle cutover details
5. the resulting slice is narrow enough that Slice `02` can cleanly pick up the
   version-floor and breakglass-contract work without reopening Slice `01`
   language.

## Open questions

1. Should Slice `01` update only feature-local docs under
   `macos-hardening/macos-hardened-same-user-lima/`, or should it also touch
   top-level docs like `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` where the support
   taxonomy is currently implied rather than explicit?
2. Does the project want a compact support matrix table in the feature
   `README.md`, the milestone `0.1` SOW, or both?
3. Is a separate ADR expected later for the support-posture language, or should
   the milestone and slice docs remain the only authority for now?
