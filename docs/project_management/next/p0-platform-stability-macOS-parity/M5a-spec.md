# M5a Spec – World deps inventory & layering

## Goal
Make `substrate world deps` use a single, coherent “tool inventory” aligned with shim doctor/health, while keeping `world-deps.yaml` as an override layer (installed + user overlay).

This triad is about **what tools exist** and **how manifests layer**, not about changing host detection semantics or first-run UX wiring.

## Context / Problem
Users expect `substrate health` (manager parity) and `substrate world deps` (guest tool sync) to describe the same reality. Today these can diverge because inventories and detection semantics differ.

## Scope
### Required behavior
1. **Authoritative base inventory**
   - Define the canonical set of “syncable” tools/managers for world deps from the same manifest/inventory used by shim doctor/health.
2. **Overlay layering**
   - The installed/bundled `world-deps.yaml` is treated as an overlay on top of the base inventory:
     - Can add tools not present in the base inventory.
     - Can override guest detection/install recipes for base tools.
   - The user overlay remains highest-priority:
     - `~/.substrate/world-deps.local.yaml` (or `SUBSTRATE_HOME` equivalent).
3. **Observability**
   - `world deps status --json` surfaces the resolved manifests (base + overlays) used to compute the inventory.

### Out of scope
- Making host detection reflect interactive shell init (handled in M5b).
- Installer/provision/first-run UX changes (handled in M5c).
- Perfect version matching (only capability parity is in scope here).

## Acceptance criteria
- `substrate health` and `substrate world deps status` draw from a consistent underlying inventory so they do not disagree on “what tools exist”.
- `world-deps.yaml` clearly functions as an override layer rather than the canonical base inventory.
- No behavior depends on being inside a repo checkout (assumes M4 is complete).

