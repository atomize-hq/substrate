Status: candidate-direction
Scope: program-wide
Authority: non-canonical
Artifact-boundary impact: proposed

# Handbook Context Integration

This note records the current working model for how handbook-derived truth
should relate to the Substrate code-intelligence program.

## Durable takeaways

1. Handbook is best understood as a canonical authored-truth namespace and
   engine, not as the code-intelligence program's contract or gate system.
2. `context` remains the owner of provider composition, trust and freshness
   filtering, and `ContextPacketV1` assembly.
3. The contract/evidence/verdict/gate layer remains a separate cross-program
   overlay. Handbook-derived truth may inform it, but does not replace it.
4. The likely migration order is extract first, migrate second: split the live
   handbook compiler into engine core, reusable pipeline core, and handbook
   product shell before moving a reusable engine into the Substrate workspace.
5. The public handbook CLI should remain a separate consumer even if a
   handbook-derived engine lands inside the Substrate workspace.

## Current best-fit ownership split

The strongest current model is:

- handbook owns canonical authored truth
- `context` ingests that truth and assembles read context packets
- `effort` consumes context packets while preserving planning ownership
- `exec` consumes context packets and related evidence while preserving runtime
  ownership
- the contracts-and-gates layer evaluates workflow claims and evidence rather
  than replacing handbook as the authoring namespace

## Live compiler seam buckets

The current handbook compiler appears to mix three layers.

### Engine core candidates

- canonical artifact inventory and loading
- artifact manifests
- structured authoring and validation for charter, project context, and
  environment inventory
- deterministic freshness logic
- doctor and truth-quality reporting

### Reusable pipeline core candidates

- stage sequencing and declarative pipeline definitions
- route evaluation and route-basis persistence
- compile versus capture separation
- handoff bundle emission and validation
- configurable setup and resolver flow

### Handbook product-shell candidates

- public CLI command behavior
- repo-local `.handbook/**` assumptions
- operator-facing refusal or recovery wording
- handbook-specific packet vocabulary and adoption flow

## Storage and runtime implication

The likely future does not remove handbook state and generated-document flows.
It changes which layer owns the root and shell assumptions.

The reusable requirement is:

- support repo-local `.handbook/**` when handbook runs as its own product
- support Substrate-managed roots such as `~/.substrate/<handbook>/*`
- preserve deterministic route/state/compile/capture behavior across both

That suggests reusing a pipeline core only after root layout and shell behavior
become configurable.

## Workspace landing direction

The strongest current workspace direction is:

- keep `context` as the owner of packet assembly
- land a handbook-derived engine as an adjacent provider crate rather than
  turning handbook into `context`
- keep downstream crates consuming handbook truth through `context` packets or
  other stable provider boundaries instead of direct handbook-internal imports

The current likely crate shape is something like `crates/handbook-engine` with
a package name such as `substrate-handbook-engine`, but that remains
provisional until the handbook split work is complete.

## Boundary caution

This note argues that handbook should be planned into the code-intelligence
program as a likely future provider and workspace landing.
It does not by itself freeze the final crate name, provider trait shape,
pipeline split, or migration order beyond the current extract-then-migrate
bias.
