# stabilize-dev-install-helper-discovery-fse - manual testing playbook

This playbook freezes the operator-facing claim boundary for `SEAM-3`.
It is documentation for evidence wording, not a smoke harness.
It records the wording guardrails that `S1` applies to `REM-002`.

## Scope

This playbook is authoritative for the wording that downstream evidence surfaces may use when describing the landed contracts from:

- `governance/seam-1-closeout.md`
- `governance/seam-2-closeout.md`

It references the closeout-backed contract set:

- `C-01` helper discovery and CLI contract
- `C-02` fixed durable runtime bundle surface
- `C-03` managed-asset eligibility
- `C-04` protected-path refusal and preserved-path reporting

## Claim boundary

- `REM-002` is addressed in this slice by freezing the wording boundary before any smoke or closeout evidence is added.
- macOS evidence may claim helper discovery, validation, and managed cleanup only.
- macOS evidence must not claim full provisioning parity.
- macOS evidence must not claim release-root staging parity.
- Windows evidence may claim compile parity only.
- Windows evidence must not claim supported `substrate world enable` behavior.
- Checkpoint-facing wording must stay aligned to the landed closeout truth above and must not broaden platform support beyond those contracts.

## Required wording checks

### Case 1 - macOS boundary check

Confirm that any operator-facing wording on macOS stays within the following boundary:

- helper discovery is correct
- validation reflects the landed contract surface
- managed cleanup is described only as the cleanup contract published by `SEAM-2`

Expected result:

- the wording does not imply that every release-root asset is staged
- the wording does not imply that macOS has parity with a full provisioning workflow

### Case 2 - Windows boundary check

Confirm that any operator-facing wording for Windows stays within compile-parity-only language.

Expected result:

- wording describes Windows as compile parity only
- wording does not imply supported behavior for `substrate world enable`

### Case 3 - checkpoint wording check

Confirm that checkpoint-facing summaries point at closeout-backed truth rather than provisional planning language.

Expected result:

- references to `C-01`..`C-04` remain explicit
- references to `governance/seam-1-closeout.md` and `governance/seam-2-closeout.md` remain the basis for the claim boundary
- no checkpoint summary promotes macOS or Windows scope beyond the contract set above

## Evidence capture guidance

- Capture only wording and boundary assertions in this slice.
- Defer smoke assertions, platform-specific command scripts, and checkpoint closeout material to later slices.
- Keep any future manual evidence notes narrow enough that they can be validated against the closeout-backed contract set without reinterpreting platform support.
