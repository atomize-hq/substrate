# Fixture Manifest: Agent Drift Analyzer Session Archetype R4

This document names the canonical first-pass `R4` fixture matrix. It is a labeling authority for
tests and reviews, not an implementation artifact by itself.

Each fixture should record:

- expected public label
- expected confidence floor / ceiling where relevant
- decisive evidence
- counter-evidence
- why nearby competing labels lose

## Packet `R4-3` Locked Authority Matrix

This section is the reviewable authority for the first `R4` label matrix. The regression anchors named
here are the analyzer tests that must stay aligned with these expectations unless this document is
updated first.

## Canonical Cases

### 1. Clear planning checkpoint

- Regression anchor:
  - `checkpoints_classify_docs_heavy_scope_shaping_as_planning`
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

- Regression anchor:
  - `checkpoints_classify_source_edit_plus_local_verification_as_autonomous_implementation`
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

- Regression anchor:
  - `checkpoints_classify_repeated_failing_verification_as_troubleshooting`
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

- Regression anchor:
  - `checkpoints_shift_to_verification_closeout_when_proof_dominates_new_source_edits`
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

- Regression anchor:
  - `checkpoints_degrade_mixed_cases_instead_of_overclaiming_high_confidence`
  - `checkpoints_lock_ambiguous_mixed_fixture_as_medium_autonomous_implementation`
- Expected label: `autonomous_implementation`
- Confidence: `medium`
- Decisive evidence:
  - stable working set plus source edits still beat the competing modes
  - implementation-like evidence remains real even though it is contested
- Counter-evidence:
  - inspection widened the visible search space
  - verification remained active enough to prevent high confidence
  - docs/spec edits still counted against direct source-implementation certainty
- Why other labels lose:
  - `planning` loses because the source-edit cadence is still stronger than pure scope shaping
  - `verification_closeout` loses because verification is still mixed with active edits
  - no mode wins cleanly enough for `high` confidence

### 6. Legitimate mode shift with hysteresis

- Regression anchor:
  - `checkpoints_lock_transition_from_planning_to_implementation_without_flapping`
- Expected label:
  - checkpoint 1: `planning`
  - checkpoint 2: `autonomous_implementation`
- Confidence:
  - checkpoint 1: `medium`
  - checkpoint 2: `medium`
- Decisive evidence:
  - the first checkpoint is still dominated by planning-style inspection
  - the second checkpoint adds source editing plus local verification on the agreed scope
- Counter-evidence:
  - the earlier planning prefix still counts against an immediate high-confidence implementation claim
  - local verification keeps the shifted checkpoint from looking like a pure implementation loop
- Why other labels lose:
  - the label should shift once the behavior changes, but the transition must not jump straight to
    `high`
  - this case proves one honest mode change, not oscillation across adjacent checkpoints

### 7. PR-response loop

- Regression anchor:
  - `checkpoints_lock_pr_response_loop_as_medium_autonomous_implementation`
- Expected label: `autonomous_implementation`
- Confidence: `medium`
- Decisive evidence:
  - targeted code edits on the active review scope
  - local verification still supports implementation follow-through
  - concentrated working set keeps the patch loop narrow
- Counter-evidence:
  - residual inspection before editing
  - verification still prevents `high` confidence
- Why other labels lose:
  - proof gathering has not yet overtaken active implementation

### 8. Proof-oriented closeout loop

- Regression anchor:
  - `checkpoints_shift_to_verification_closeout_when_proof_dominates_new_source_edits`
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

- Regression anchor:
  - `checkpoints_classify_repeated_failing_verification_as_troubleshooting`
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

- Regression anchor:
  - `checkpoints_cap_confidence_when_parent_visible_behavior_is_child_opaque`
  - `checkpoints_lock_delegated_parent_opaque_fixture_as_low_confidence_planning`
- Expected label: `planning`
- Confidence: `low`
- Decisive evidence:
  - landed `DelegationContext` shows `delegating_parent`
  - visible parent behavior stays orchestration-heavy
  - `child_work_visibility = opaque`
- Counter-evidence:
  - opaque child work blocks high-confidence claims about the real execution mode underneath
- Why other labels lose:
  - no archetype should claim high-confidence child-visible semantics from the parent prefix
  - the visible parent evidence is still closer to planning/orchestration than to direct
    implementation

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
