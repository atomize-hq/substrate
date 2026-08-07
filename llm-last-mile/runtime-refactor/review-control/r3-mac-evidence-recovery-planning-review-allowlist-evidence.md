# Allowlist/evidence/recovery discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN

## Review boundary

Fresh read-only discovery review of the full initial eight-path planning subject. No source/native
state, installation, Keychain, XPC, Lima, or repository mutation was performed by the reviewer. The
reviewer reported a fingerprint mismatch, which remains a discovery finding rather than a clean
attestation.

## Findings classified against the selected outcome

| ID | Priority | Disposition | Evidence and required recovery-plan correction |
|---|---|---|---|
| `P1-PLN-001` | P1 | duplicate reported in other discovery reports; remediated once | Subject fingerprint definition was not authoritative across the task and control surface. Freeze the exact eight paths and algorithm, then review a changed subject. |
| `P1-AER-002` | P1 | remediated in the consolidated planning pass | R3 required a dispatch-supplied expected project ID but the authoritative validation command omitted it. Add `--expected-product-project-id <dispatch-bound-project-id>` to every recovery-era current command contract and require source/test coverage, retaining the historical Linux ID explicitly. |
| `P2-AER-003` | P2 | remediated in the consolidated planning pass | The hunk/symbol fence used conditional categories and shorthand. Enumerate exact paths, symbols/types/fields, exact Cargo/lock rule, and discard every remaining hunk in shared files. |
| `P2-AER-004` | P2 | remediated in the consolidated planning pass | Linux compatibility/zero-Windows-change proof was assertion-only. Give each shared packet explicit Linux differential/focused-test and Windows target-compile/forbidden-file-hash gates. |
| `P2-AER-005` | P2 | remediated in the consolidated planning pass | Native restoration mapping conflated unavailable capability with failed action/restoration. Reserve platform handoff for pre-action unavailability and use native-evidence block for available-run/restoration/validator failure. |
| `P2-AER-006` | P2 | remediated in the consolidated planning pass | Earlier present-tense MAC-to-evidence wording remained live. Append an explicit recovery-route supersession: historical attempt-4/donor are ineligible; only the later reviewed remote-equal R1–R6 receipt can precede evidence. |

## Clear areas observed

- The donor inventory and expected digest were captured and wholesale donor publication was banned.
- The initial planning diff was documentation-only and append-only in the five current-status files.
- Raw helper action, `lima-stdio-v1`, Windows behavior changes, ordinary Linux host pairing, and
  native-proof substitution were excluded.

This discovery report is intentionally not a closure verdict. A different fresh reviewer must review
the remediated full subject.

## Fresh full-subject closure

A different fresh read-only reviewer, given no author reasoning or prior review conclusion, reviewed
the complete remediated eight-path subject fingerprint
`sha256:71c0b6794dcc4dc3b4e0df9269c9add66fdc03eb66957d0c002942ee6eee1234` and returned **CLEAN**:
no actionable findings. The closure reviewer performed no mutation and no native, installation,
Keychain, XPC, or Lima action.
