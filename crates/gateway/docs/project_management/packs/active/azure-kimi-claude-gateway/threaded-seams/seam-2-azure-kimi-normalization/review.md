---
seam_id: SEAM-2
review_phase: pre_exec
execution_horizon: active
basis_ref: seam.md#basis
---
# Review Bundle - SEAM-2 Azure Kimi Provider Normalization

This artifact feeds `gates.pre_exec.review`.
`../../review_surfaces.md` is pack orientation only.

## Falsification questions

- Can downstream code still infer tool intent from raw Azure `reasoning_content` or sentinel markers because `C-02` leaves provenance or normalized event boundaries ambiguous?
- Is parser work still coupled to planner/executor policy, model-role choice, or Anthropic surface semantics in a way that should belong to `SEAM-3` or `SEAM-4` instead?
- Could the seam claim normalized coverage from upstream commit `5a372fb` without reproducing the Azure Foundry hidden-tool cases captured in ADR 0002 and the two handoff artifacts named in `../../README.md`?

## R1 - Azure provider normalization flow that should land

```mermaid
flowchart LR
  AZ["Azure chat-completions response"] --> RAW["Provider adapter boundary"]
  RAW --> EXPL["Explicit `tool_calls` path"]
  RAW --> HIDDEN["Hidden `reasoning_content` marker path"]
  EXPL --> NORM["`C-02` normalized event model"]
  HIDDEN --> NORM
  NORM --> EVT["Tool / action / final events"]
  EVT --> DOWN["SEAM-3 / SEAM-4 / SEAM-5 consumers"]
```

## R2 - Evidence and verification flow this seam must preserve

```mermaid
flowchart LR
  HANDOFF["ADR 0002 + handoff evidence"] --> PROBE["Azure probe capture"]
  PROBE --> FIX["Fixture corpus"]
  FIX --> EXPECT["Expected normalized outputs"]
  EXPECT --> TEST["Regression coverage"]
  TEST --> PUB["`THR-02` publication decision"]
```

## Likely mismatch hotspots

- The adopted upstream provider transform may still blur explicit tool-call repair and Azure hidden-tool parsing, leaving the reuse-versus-bypass line under-specified.
- The normalized event contract may collapse tool intent, action progress, and final completion into ambiguous event shapes that later seams reinterpret differently.
- Fixture coverage may overfit to `Kimi-K2-Thinking` and leave `Kimi-K2.5` or mixed explicit-plus-hidden cases unclassified.

## Pre-exec findings

- No remediation is opened during decomposition. The current basis is usable because `THR-01` is already published and the pack already names the Azure hidden-tool gap as `SEAM-2` work.
- The main pre-exec review pressure is contract sharpness, not horizon posture: `C-02` must become explicit enough in `S1` that later seams can cite normalized event rules without reverse-engineering parser internals.
- Revalidation must check current Azure evidence against the `5a372fb` note and handoff chain before promotion to `exec-ready`; until then, the seam remains intentionally `decomposed`.

## Pre-exec gate disposition

- **Review gate**: `pending`
- **Contract gate concerns**: freeze the normalized event vocabulary, provenance/debug rules, malformed-marker behavior, and reuse-versus-bypass boundary before implementation begins.
- **Revalidation prerequisites**: confirm the `C-01` provider boundary from `docs/foundation/claude-code-mux-extension-boundary.md` still matches the intended attachment point and refresh Azure evidence against the handoff chain before execution starts.
- **Opened remediations**: none

## Planned seam-exit gate focus

- **What must be true before downstream promotion is legal**: `C-02` is concrete and landed, explicit and hidden Azure tool intent normalize into one event model with regression evidence, and no downstream seam needs raw Azure payload semantics to proceed.
- **Which outbound contracts/threads matter most**: `C-02` and `THR-02`
- **Which review-surface deltas would force downstream revalidation**: new hidden-tool variants, changed normalized event ordering/fields, or any parser change that pulls routing policy or public-surface behavior into the provider layer
