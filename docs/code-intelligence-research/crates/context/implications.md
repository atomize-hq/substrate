Status: candidate-direction
Scope: context
Authority: non-canonical
Artifact-boundary impact: possible

# Context Implications

This note records the strongest `context`-specific implications from the
handbook/context integration review and the live handbook compiler seam check.

## Durable implications

1. `context` should own provider composition and packet assembly, not the
   authored-truth namespace itself.
2. A handbook-derived engine is the clearest current candidate for the first
   high-trust `context` provider that supplies canonical authored truth.
3. The likely long-term fit is handbook-engine -> `substrate-context` ->
   `ContextPacketV1` -> downstream consumers, rather than handbook replacing
   `context`.
4. Handbook-derived canonical artifacts such as `CHARTER`,
   `PROJECT_CONTEXT`, `ENVIRONMENT_INVENTORY`, and `FEATURE_SPEC` should enter
   planning and runtime through `context` packet assembly plus explicit
   provenance and freshness metadata.
5. Reusable pipeline mechanics may still matter for `context`, but only after
   storage-root and product-shell assumptions are peeled away from the current
   handbook implementation.

## Likely landing seam from the live handbook compiler

### Strong early engine candidates

These look closest to durable canonical-truth or truth-quality engine logic:

- `artifact_manifest`
- `canonical_artifacts`
- `doctor`
- `freshness`
- `author::{charter, project_context, environment_inventory}`

### Reusable pipeline-core candidates after refactor

These look reusable if their current storage and product assumptions are
parameterized:

- `pipeline`
- `pipeline_route`
- `pipeline_compile`
- `pipeline_capture`
- `pipeline_handoff`
- `route_state`
- `resolver`
- `rendering`
- `setup`

### Likely handbook-product shell pressure points

These should not be assumed to become stable `context` APIs without more
review:

- refusal and recovery wording
- packet-oriented public command behavior
- repo-local `.handbook/**` layout assumptions
- public CLI-specific budgeting or operator affordances

## Context-packet implications

The current strongest packet direction is that `ContextPacketV1` should be able
to carry handbook-derived truth as read context, including:

- canonical artifact refs and fingerprints
- trust class and provenance
- freshness status
- provider-root metadata for repo-local versus Substrate-managed roots
- explicit missing-input or low-trust markers

The packet should not collapse into handbook runtime state. It should expose
read context that `effort` and `exec` can consume without importing handbook
internals directly.

## Storage-root implication

The likely storage model is no longer only repo-local `.handbook/**`.

The reusable `context`-facing requirement is broader:

- repo-local handbook roots should remain possible
- Substrate-managed roots such as `~/.substrate/<handbook>/*` should also be
  possible
- tests and fixtures should be able to point at isolated roots

That means the reusable boundary should prefer explicit root configuration and
deterministic artifact refs over hardcoded repo-local paths.

## Promotion caution

This note supports planning for a handbook-derived provider seam and a future
workspace landing.
It does not by itself freeze final crate names, final provider trait shapes, or
which current handbook pipeline modules remain reusable after extraction.
