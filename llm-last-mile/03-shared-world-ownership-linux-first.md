# SOW: Shared-World Ownership Explicitness, Linux-First

Status: historical draft. This file records what changed after merge so readers do not treat the abandoned shell-authoritative design as current behavior.

Authoritative current references:

- [PLAN-03](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-03.md)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)

## Historical Drift Correction

This draft originally proposed a shell-owned binding store under `~/.substrate/run/agent-hub/world-bindings/` with Linux `session.json` acting as a mirrored copy.

That proposal is stale and was not the merged design.

The merged Linux-first implementation instead makes `crates/world` authoritative:

- persisted authority lives in `SessionWorldMetadata` in `crates/world/src/session.rs`
- `set_shared_binding_state()` is the only shared-binding mutation path
- `persist_metadata()` atomically writes owner-bearing metadata
- same-owner shared-world ensure and replace paths are serialized by a backend-local mutex in `crates/world/src/lib.rs`
- downstream surfaces may expose or accept only `binding_state=Active`

## Final Linux-First Behavior

### Ownership proof

Explicit shared-owner reuse is keyed by persisted owner metadata:

- `owner_mode=shared_orchestration`
- `orchestration_session_id`
- `world_generation`
- `binding_state`

Generic reusable worlds remain separate and do not carry shared-owner proof.

Shared-owner reuse requires:

- exact `orchestration_session_id` match
- compatible world inputs
- `binding_state=Active`

Ownerless legacy metadata, partial owner metadata, and inactive bindings are never reusable for shared-owner flows.

### Replace window

`replace_shared_owner_session()` now follows this ordering:

1. pre-commit the current world to `binding_state=Replacing`
2. commit a fresh replacement world with generation `expected_generation + 1` and `binding_state=Active`
3. roll the old world back to `Active` if replacement creation fails, then clean up the partial replacement root
4. finalize the old world to `binding_state=Replaced` after the replacement is committed

`Replacing` is an internal transition state only. It is not a valid downstream proof state.

### Recovery guarantees

Linux recovery now guarantees:

- a lone `Replacing` world for an owner is reconciled back to `Active`
- a newer `Active` world is preferred over an older `Replacing` world for the same owner
- ambiguous same-owner recovery state fails closed
- malformed or partial owner metadata is ignored without being adopted
- `Replaced` and `Abandoned` worlds are never reused for shared-owner flows

## What This File No Longer Claims

Do not use this historical draft as authority for any of the following:

- shell runtime persistence as the primary shared-world authority
- a co-authoritative or mirrored shell binding registry
- slice-03 ownership of runtime-manifest projection
- slice-03 ownership of participant invalidation or replacement registry behavior
- downstream acceptance of non-`Active` binding states
