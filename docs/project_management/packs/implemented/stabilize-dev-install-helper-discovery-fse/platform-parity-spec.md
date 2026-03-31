# stabilize-dev-install-helper-discovery-fse - platform parity spec

This spec freezes the platform claim boundary for `SEAM-3`.
It is authoritative for what may be said about platform scope, not for new runtime behavior.
It defines the platform wording guardrails that `S1` applies to `REM-002`.

## Authority

This spec consumes the closeout-backed truth from:

- `governance/seam-1-closeout.md`
- `governance/seam-2-closeout.md`

It is bounded by the landed contract set:

- `C-01` helper discovery and CLI contract
- `C-02` fixed durable runtime bundle surface
- `C-03` managed-asset eligibility
- `C-04` protected-path refusal and preserved-path reporting

## Required platforms

- Behavior claim platforms: `linux`, `macos`
- Parity-only platform: `windows`

## Explicit guarantees

- Linux and macOS may be described only within the landed helper-discovery, validation, and managed-cleanup contract boundary.
- Windows may be described only as compile parity.
- The spec does not widen the staged bundle, the cleanup contract, or the helper-order contract.
- `REM-002` is handled here by refusing broader macOS or Windows claims until later proof surfaces land.
- Any future proof-surface work must preserve the same claim boundary rather than invent new platform support.

## Explicit non-claims

- macOS is not full provisioning parity.
- macOS is not release-root staging parity.
- Windows is not supported behavior for `substrate world enable`.
- Windows is not a behavior-validation platform for this seam.
- This spec does not authorize new smoke assertions or new operational scripts; those belong to later seam work.

## Platform rules

### Linux

- Linux may be used to describe the landed contract set in full.
- Any Linux-facing evidence wording must remain tied to `C-01`..`C-04` and the upstream closeouts.

### macOS

- macOS may be used to describe helper discovery, validation, and managed cleanup only.
- macOS wording must not imply that all release-root assets are staged.
- macOS wording must not imply that the feature equals a full provisioning workflow.

### Windows

- Windows remains compile parity only.
- Windows wording must not imply supported `substrate world enable` behavior.
- Windows wording must not imply behavior evidence beyond compilation and parity checks.

## Checkpoint wording rules

- Checkpoint-facing summaries must reference the landed closeouts and the contract set above.
- Checkpoint-facing summaries must not promote macOS scope beyond helper discovery, validation, and managed cleanup.
- Checkpoint-facing summaries must not promote Windows beyond compile parity.
- Checkpoint-facing summaries must remain non-promotional and falsifiable.
