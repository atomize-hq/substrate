# Seam Map - make-doctor-health-output-explain-why

## Restated scope and assumptions

This feature is a messaging-and-contract change, not a behavior-change feature. It must explain **why** world isolation is disabled across doctor and health surfaces while preserving the actual `world.enabled` resolution model, existing exit semantics, and platform support boundaries.

The source pack already converged to two accepted execution units (`DHO0`, `DHO1`). In extractor posture, those become two governance-ready seams because they each have a clear value boundary, verification path, and touch surface. Cross-platform validation, smoke evidence, and queue-compatibility concerns stay attached as threads and closeout concerns rather than being promoted into artificial extra seams.

## Execution horizon policy

- **Active seam**: `SEAM-1` inferred from the source pack's first critical-path checkpoint (`DHO0` / CP1). It is the earliest landable seam that establishes the authoritative disable-attribution contract in doctor text output.
- **Next seam**: `SEAM-2` inferred from the source pack's second checkpoint (`DHO1` / CP2). It expands the same attribution truth into JSON and health surfaces after `SEAM-1` stabilizes.
- **Future seams**: none extracted. The source pack's own analysis says the behavior model converges cleanly to two execution seams, so additional seams would mostly duplicate verification or cleanup work already owned by these two seams.

Only the active seam is eligible for authoritative downstream deep planning by default. The next seam may later receive seam-local review and slices, but any deeper work remains provisional until the active seam lands and publishes its closeout-backed handoff.

## Candidate generation and pruning

### Kept

- **Capability seam** -> `SEAM-1` doctor text disable attribution
  - Why kept: it is a user-visible capability that can be verified independently and lands a coherent product truth: exact message bodies and correct winner attribution in doctor text.
- **Integration seam** -> `SEAM-2` JSON + health disable attribution
  - Why kept: it consumes the first seam's contract and publishes stable structured fields plus health parity across multiple surfaces and downstream consumers.

### Pruned

- **Standalone conformance seam** for smoke scripts / manual playbook / checkpoint prompts
  - Why pruned: these artifacts verify both seams but do not represent an independently landable user/system capability.
- **Standalone contract-authoring seam** for `contract.md`, `decision_register.md`, and schema inventory docs
  - Why pruned: those artifacts are pack-level governance and contract scaffolding that feed both seams; they are not a separate product boundary.

## Seam inventory

| Seam | Horizon | Type | Source-pack mapping | Purpose | Primary touch surface | Verification anchor |
|---|---|---|---|---|---|---|
| `SEAM-1` | `active` | `capability` | `DHO0` | Make doctor text tell the truth about the effective disable source, with exact wording and redaction rules. | platform doctor entrypoints + shared attribution helper + doctor text renderers | winner-to-message mapping tests, manual CLI/env/workspace/global scenarios, Linux/macOS/Windows parity checks |
| `SEAM-2` | `next` | `integration` | `DHO1` | Reuse the same attribution truth in top-level JSON fields and in health text/JSON, including nested doctor/shim paths. | health/shim reporting + doctor JSON + schema contract + parity validation | JSON emit/omit checks, health text parity checks, nested CLI-flag preservation, full CP2 cross-platform evidence |

## Seam boundaries

- `SEAM-1` owns the **human-facing doctor attribution truth** and the **shared disable-attribution model** that downstream surfaces must consume.
- `SEAM-2` owns the **structured JSON contract** and **health-surface propagation** of that truth.
- Queue overlap with disabled-status UX, JSON envelope work, provisioning packs, and replay-warning reuse is tracked as dependencies, threads, and stale triggers rather than hidden inside either seam.
