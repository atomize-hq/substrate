# Fixture Manifest: Agent Drift Analyzer Session Progress R5

Status: restored to the live docs tree on 2026-06-23 by Packet `R5.75-5.2`. Packet `R5-7` and
Packet `R5.5-4` still define the currently committed native/synthetic progress corpus, while
Packet `R5.75-5` adds the contract for a bounded adapted external **secondary** lane. This docs
slice records that authority boundary honestly before any adapted fixture directories land.

## Purpose

This is the live fixture-authority contract for R5 `session_progress` validation. It exists so
reviewers do not have to reconstruct the current wall from an archived manifest plus scattered
README fragments.

The key question this doc answers is: **which fixture families are primary semantic authority,
which are supporting coverage, and which are only bounded secondary robustness?**

## Authority Boundary

1. **Native annotated real-rollout cases are the primary semantic authority.**
   - They are the only cases that can satisfy the R5 requirement for bounded semantic acceptance on
     actual traces.
   - At least one native `annotated_real_rollout` must remain in the committed corpus.
2. **Synthetic bundle-shaped cases are supporting coverage only.**
   - They prove deterministic bundle assembly and guardrail behavior, but they do not replace the
     native acceptance anchor.
3. **Adapted external progress cases are secondary robustness only.**
   - They must stay explicitly separate from the native corpus in metadata, README wording, and
     review language.
   - They may strengthen regression coverage for already-adopted external repro classes, but they
     must never be described as equal authority with native real rollouts.
4. **Objective-shaped adapted evidence does not belong in this progress corpus.**
   - It may only enter
     `crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/stretch-external/`
     after a later packet proves it adds net-new signal beyond the locked native objective corpus.

## Current Committed Progress Corpus

### Native primary authority (`annotated_real_rollout`)

These are the live primary semantic acceptance cases under
`crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`:

1. `019e899c-453f-71f2-a99d-155848c7b081` — troubleshooting advancement
2. `019e940c-a91b-7fe0-a967-b0bdd595b581` — planning meander/stall
3. `019e8b42-42bd-7b10-baae-3265edb65f4b` — verification closeout narrowing
4. `019eb970-3543-7ab1-a5d6-2a62c00c7185` — delegated parent-visible positive proof that remains
   guardrail-limited under child opacity
5. `real-implementation-advancing-019e894a-ord6` — implementation verification-wall advancement
6. `real-closeout-conservative-019e767c-ord3` — closeout/review checkpoint that stays
   `insufficient_evidence`
7. `real-reopen-regressing-019e894a-ord7` — previously clean verifier that later regresses

### Synthetic support cases (`synthetic_bundle_shaped`)

These remain committed supporting coverage, not primary acceptance authority:

1. `synthetic-planning-advancing`
2. `synthetic-implementation-advancing`
3. `synthetic-parent-visible-opaque`
4. `synthetic-zero-verifier-anti-flap`

## Adapted External Secondary Lane

Packet `R5.75-5.1` routed the adopted adapted sessions into the following progress-owned secondary
lane:

1. `adapted-sparse-readable-f47b81f39f2495dd`
   - packet-owned expectation: preserve the `R5.75-2` sparse-readable conservative outcome
2. `adapted-zero-verifier-097d97e914ca220f`
   - packet-owned expectation: preserve the `R5.75-4` zero-verifier anti-flap conservative outcome
3. `adapted-parent-visible-da59436e63915185`
   - packet-owned expectation: preserve the combined `R5.75-3` / `R5.75-4` delegated
     parent-visible guardrail behavior

Current truth for this docs-only slice:

- this lane is **defined but not yet committed** under
  `crates/agent-drift-analyzer/tests/fixtures/progress_acceptance/**`
- no adapted external case counts toward the current primary semantic acceptance wall yet
- when these cases land, they must stay explicitly labeled secondary in fixture metadata, README
  language, and test/harness messaging

The adapted objective-shaped session `05a56cc51632982b` is intentionally **not** part of this
progress lane. Its only eligible home is the objective-side
`stretch-external/` placeholder lane, and only if a later packet proves net-new signal.

## Required Fixture Metadata

Every committed progress fixture should keep the design-arch contract explicit by recording:

- `Fixture id`
- `Fixture kind`
- `Expected dimension`
- `Expected status`
- `Expected confidence`
- `Decisive evidence`
- `Counter-evidence`
- `Comparable attempts`
- `Window/reset expectation`
- `Why competing statuses lose`
- `Implementation fixture location`

Additional requirements by lane:

- **Native primary cases**
  - must keep the existing `annotated_real_rollout` screening and provenance fields
- **Synthetic support cases**
  - must stay clearly labeled as supporting bundle-shaped coverage
- **Adapted secondary cases**
  - must cite their source adapted session id
  - must name the earlier packet-owned expectation they are preserving
  - must explain why they remain secondary robustness instead of primary semantic authority

## Objective Cross-Reference

The objective-side authority boundary must stay parallel to the progress-side one:

- `locked-acceptance/` remains the native primary objective corpus
- `stretch-external/` remains the adapted external secondary lane only
- Packet `R5.75-5.2` does **not** admit `05a56cc51632982b`; that root stays placeholder-only until
  a later packet proves the case adds signal beyond the locked native corpus

## Review Rules

When reviewing future fixture additions against this doc:

1. reject any wording that treats adapted external cases as equal authority with native real
   rollouts
2. reject any change that removes the native annotated real-rollout anchor without an explicit
   replacement
3. reject any progress-side admission of `05a56cc51632982b`
4. reject any objective-side `stretch-external/` admission that does not explain the net-new signal
   beyond the locked native objective corpus
