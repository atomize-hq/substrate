# ADR-0013 — Policy Patch-Only Broker Canonical Effective Resolution

## Status

- Status: Implemented
- Original date (UTC): 2026-01-17
- Curated into `docs/adr/implemented/`: 2026-05-26
- Owner(s): Shell/Broker maintainers

## Curated From

- Historical planning ADR:
  - `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

This curated ADR is the stable decision record. The project-management ADR remains the
planning-rich historical source.

## Decision

Policy files are patch-only everywhere, and the broker is the canonical resolver for the effective
policy consumed by CLI and runtime execution surfaces.

The stable decision is:

- `policy.yaml` is a sparse patch format rather than a parallel full-document contract
- effective policy resolution follows defaults, global patch, then workspace patch
- broker-owned workspace discovery must honor `.substrate/workspace.disabled`
- runtime execution surfaces must not silently diverge from `policy current show`

## Stable Owned Surface

This ADR owns the stable policy-resolution contract documented in:

- `docs/reference/policy/contract.md`
- `docs/reference/policy/schema.md`
- `docs/CONFIGURATION.md`

## Current Implementation Anchors

The decision is materially implemented and verified through:

- `crates/broker/src/effective_policy.rs`
- `crates/broker/src/profile.rs`
- `crates/broker/src/policy.rs`
- `crates/shell/src/execution/policy_cmd.rs`
- `crates/broker/src/tests.rs`

## Related ADRs

- `docs/adr/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/adr/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`

## Historical Note

The original ADR captured the consolidation from conflicting policy loaders to one broker-owned
effective-policy resolver. The stable contract now lives here and in the policy reference docs.
