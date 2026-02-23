# Concrete Remediation Decisions

This file records any **decision-last** choices introduced while remediating concreteness gaps.

If a decision in this file conflicts with `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md`, the decision log wins.

## CRD-0001 — Work Lift strict pack mode forbids directory/prefix Touch Set entries

Date: 2026-02-22

### Context

Work Lift v1 allows directory/prefix entries in Impact Map Touch Sets (they degrade confidence and can be expanded deterministically for advisory lift estimation). However, the strict-mode onramp requires at least one concrete, pack-eligible invariant that is:

- deterministic,
- measurable from existing contracts, and
- safe to enforce without relying on non-deterministic repo state beyond `HEAD`.

### Decision

In **strict pack** checks (SEAM-5 S3), the pack MUST have **no** directory/prefix entries:

- `validate_impact_map.py --emit-json` MUST report `dir_prefixes == []`.

### Rationale

Directory/prefix entries inherently depend on `HEAD` for deterministic expansion and are explicitly treated as lower-confidence signals. Strict pack mode needs an invariant that reduces this uncertainty rather than masking it.

### Implications

- Packs that rely on directory/prefix entries can still compute advisory lift, but strict pack checks fail until the Touch Set is made explicit.
- This does not change the validator’s strict-vs-legacy gating; it only affects the opt-in strict lift wrapper.

## CRD-0002 — Promotion criteria for enabling strict checks by default

Date: 2026-02-22

### Context

The strict-mode onramp plan previously contained placeholders (“N calibration runs”, “acceptable false positive rate”). These values are required for a concrete rollout plan.

### Decision

Promotion to “enabled-by-default” strict checks requires:

- >= 20 calibration runs
- across >= 10 distinct eligible packs (`tasks.json.meta.slice_spec_version >= 2`)
- with strict failure false-positive rate <= 5%
- with any exceptions documented as explicit allowlist entries (path + rationale) in the strict-mode standard doc.

### Rationale

These thresholds are large enough to exercise variability across packs while remaining small enough to execute during a bounded rollout.

### Implications

Strict checks remain opt-in until these criteria are met and a separate PR explicitly flips defaults.
