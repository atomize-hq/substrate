# Seam Map - Azure Kimi Claude Gateway

This seam map extracts executable feature seams from the gateway architecture rather than mirroring ADR document boundaries.

Constraint posture:

- `IMPORTANT_SUBSTRATE_ALIGNMENT.md` and ADR 0005 through ADR 0007 constrain every seam
- ADR 0001 through ADR 0004 contribute executable feature pressure, but only where they imply bounded value, touch surface, and verification
- OpenAI Responses remains a deferred adapter concern and is not elevated into this pack's default execution horizon

## Horizon summary

- **Active seam**: none remaining in this pack
- **Next seam**: none remaining in this pack
- **Future seams**: none remaining in this pack

## Seam roster

| Seam | Horizon / state | Type | Why this is a seam | Likely value | Touch surface | Verification path |
| --- | --- | --- | --- | --- | --- | --- |
| `SEAM-1` `mux-foundation-baseline` | `landed` | `platform` | Establishes the actual runtime foundation and verification boundary for the whole feature | baseline-stable adopted gateway codebase, repo-local identity renames, extension points, and `5a372fb` validation note | adopted archived `claude-code-mux` codebase, build/runtime wiring, identity-renaming surfaces, foundation verification notes | baseline build/run first, then identity-rename verification, plus explicit note on what upstream Kimi behavior remains unresolved |
| `SEAM-2` `azure-kimi-normalization` | `landed` | `integration` | Owns the critical provider normalization contract instead of spreading Azure quirks through all layers | stable normalized internal tool/action/final events from Azure Kimi responses | provider adapter, reasoning parser, fixtures, normalization model | fixtures and probes proving explicit and hidden tool intent normalize into one contract |
| `SEAM-3` `anthropic-messages-gateway-surface` | `landed` | `capability` | Delivers the first user-facing client contract without making Anthropic the core data model | Claude Code-compatible Anthropic Messages gateway surface | HTTP routes, streaming adapters, session/tool loop surface | end-to-end Claude Code path over normalized core |
| `SEAM-4` `planner-executor-orchestration` | `landed` | `integration` | Keeps dual-model routing as internal policy rather than provider or public identity logic | internal planner/executor routing that can use `Kimi-K2-Thinking` and `Kimi-K2.5` safely | routing policy, session state handoff, policy config, diagnostics | end-to-end proof that planning can feed execution through normalized events |
| `SEAM-5` `substrate-compatible-boundary` | `landed` | `conformance` | Locks the feature into one logical backend identity, structured events, and replaceable deployment transport | boundary-conformance path without architectural inversion | public capability naming, transport/auth factoring, downstream structured event boundary, drift guards | docs/design/config evidence that external identity stays singular and events stay normalized |

## Ordering rationale

1. `SEAM-1` had to land first because the repo initially had no verified local foundation. That foundation is now published through `THR-01`.
2. `SEAM-2` had to land before the first public surface because every later seam consumes the normalized event contract it owns.
3. `SEAM-3` is now landed because the first external value seam already froze the public surface on closeout-backed `C-03` truth.
4. `SEAM-4` is now landed because internal planner/executor routing closed out on `THR-04` rather than remaining a forward blocker.
5. `SEAM-5` has now landed, published `THR-05`, and closed the boundary seam on closeout-backed external-boundary truth.

## Non-seams and pruned candidates

- An ADR-per-seam map was rejected because the ADRs are architectural decisions, not individually verifiable execution seams.
- A standalone OpenAI Responses seam is deferred because the first public target is Anthropic Messages and the user asked to keep Responses easy later, not primary now.
- A catch-all "validation seam" was rejected because verification belongs inside each seam, with `SEAM-5` reserved only for genuine cross-seam conformance and drift-guard work.
