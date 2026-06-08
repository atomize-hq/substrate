# Fixture Manifest: Agent Drift Analyzer Session Archetype R4

This document names the canonical first-pass `R4` fixture matrix. It is a labeling authority for
tests and reviews, not an implementation artifact by itself.

Each fixture should record:

- expected public label
- expected confidence floor / ceiling where relevant
- decisive evidence
- counter-evidence
- why nearby competing labels lose

## Canonical Cases

### 1. Clear planning checkpoint

- Expected label: `planning`
- Confidence: `medium` or `high`
- Decisive evidence:
  - broad exploration-like inspection
  - directive synthesis / scope-shaping behavior
  - low verification density
- Counter-evidence:
  - any isolated edit or verification command
- Why other labels lose:
  - no stable source-edit plus verification cadence
  - no proof-oriented narrowing

### 2. Clear autonomous implementation checkpoint

- Expected label: `autonomous_implementation`
- Confidence: `medium` or `high`
- Decisive evidence:
  - concentrated source edits
  - stable objective
  - local verification on the active scope
- Counter-evidence:
  - residual exploration
- Why other labels lose:
  - verification is supporting implementation, not dominating as closeout
  - failure / diagnosis does not dominate the recent prefix

### 3. Clear troubleshooting checkpoint

- Expected label: `troubleshooting`
- Confidence: `medium` or `high`
- Decisive evidence:
  - repeated failing verification or diagnostics
  - meaningful exploration / narrowing on a constrained scope
  - insufficient stable implementation cadence
- Counter-evidence:
  - occasional edits or successful verification
- Why other labels lose:
  - proof gathering is not dominant enough for `verification_closeout`
  - implementation loops are not stable enough for `autonomous_implementation`

### 4. Clear verification-closeout checkpoint

- Expected label: `verification_closeout`
- Confidence: `medium` or `high`
- Decisive evidence:
  - successful proof-oriented verification
  - narrowing scope
  - little new source editing
- Counter-evidence:
  - residual implementation context from earlier checkpoints
- Why other labels lose:
  - verification dominates more than implementation
  - failures / diagnosis do not dominate the recent prefix

### 5. Ambiguous mixed checkpoint

- Expected label: implementation-defined winner with capped confidence
- Confidence: must stay `low` or `medium`
- Decisive evidence:
  - materially mixed implementation, verification, and exploration signals
- Counter-evidence:
  - strong competing evidence for at least one nearby label
- Why other labels lose:
  - no single mode dominates coherently enough for high confidence

### 6. Legitimate mode shift with hysteresis

- Expected label: transition across adjacent checkpoints without one-checkpoint flapping
- Confidence: degraded during the shift
- Decisive evidence:
  - sustained recent change in dominant behavior
- Counter-evidence:
  - earlier prefix still supports the prior mode
- Why other labels lose:
  - contradictory evidence is strong enough to shift, but not enough to erase transition ambiguity

### 7. PR-response loop

- Expected label: `autonomous_implementation`
- Confidence: `medium`
- Decisive evidence:
  - targeted code edits
  - local verification
  - concentrated working set
- Counter-evidence:
  - review / proof language in the objective
- Why other labels lose:
  - proof gathering has not yet overtaken active implementation

### 8. Proof-oriented closeout loop

- Expected label: `verification_closeout`
- Confidence: `medium`
- Decisive evidence:
  - successful verification dominates
  - scope narrows
  - new editing tapers
- Counter-evidence:
  - earlier implementation work in the same session
- Why other labels lose:
  - current mode is proof gathering, not active implementation

### 9. Failing verification loop

- Expected label: `troubleshooting`
- Confidence: `medium`
- Decisive evidence:
  - repeated failing verification
  - exploratory diagnosis around the failing scope
- Counter-evidence:
  - some proof-oriented commands
- Why other labels lose:
  - verification is failing and forcing renewed diagnosis, so this is not honest closeout

### 10. Delegating parent with opaque child work

- Expected label: visible-parent best fit, but confidence capped conservatively
- Confidence: `low` or `medium`
- Decisive evidence:
  - landed `DelegationContext` shows `delegating_parent`
  - `child_work_visibility = opaque`
- Counter-evidence:
  - any visible parent-side implementation or planning behavior
- Why other labels lose:
  - no archetype should claim high-confidence direct child-visible semantics from the parent prefix

### 11. Legacy schema compatibility

- Expected label: not applicable
- Confidence: not applicable
- Decisive evidence:
  - `v0.2` through `v0.4` artifacts still load without `session_archetype`
- Counter-evidence:
  - none
- Why other labels lose:
  - this is a compatibility contract case, not a semantic label case

### 12. `v0.5` requiredness

- Expected label: not applicable
- Confidence: not applicable
- Decisive evidence:
  - missing `session_archetype` fails closed for `v0.5`
- Counter-evidence:
  - none
- Why other labels lose:
  - this is a contract-validation case, not a semantic label case
