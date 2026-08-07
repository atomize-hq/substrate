# Authority/security discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN

## Review boundary

Fresh read-only discovery review of the full initial eight-path planning subject. No source, native,
Keychain, XPC, Lima, installation, or repository mutation was performed by the reviewer. The
reviewer reported that its independently constructed manifest hash did not match the supplied
initial hash; this is recorded as a finding, not a clean-subject attestation.

## Findings classified against the selected outcome

| ID | Priority | Disposition | Evidence and required recovery-plan correction |
|---|---|---|---|
| `P1-PLN-001` | P1 | remediated in the consolidated planning pass | The initial TASKS subject list named eight paths while the controlling current-status shorthand remained six. Declare one exact eight-path subject and deterministic macOS `shasum` manifest algorithm, then rerun closure on the changed fingerprint. |
| `P1-ASR-001` | P1 | remediated in the consolidated planning pass | The mapped bridge allowed Stage-1 through protected bootstrap without `PublisherBootstrapAuthorizationV1`. Split Stage-1 absent-create, post-PM request, and direct-interactive bootstrap; prohibit Stage-1 reuse and test missing/replayed bootstrap authorization. |
| `P1-ASR-002` | P1 | remediated in the consolidated planning pass | Dual sessions omitted the retained host terminal as the sole full confirmation display source. Require host-terminal display, manual guest-TTY entry, protected confirmation commitment, and ticket/data/absent-terminal negatives. |
| `P2-ASR-003` | P2 | remediated in the consolidated planning pass | R5 privileged role/action selection was open-ended. Bind it to the existing closed MAC `ManagedArtifactRoleV1` table, enumerate wrapper branch mapping, and prove no new pair. |
| `P2-ASR-004` | P2 | remediated in the consolidated planning pass | R4's Cargo/lock fence was generic. Name only `libc = "0.2"`, `sha2 = { workspace = true }`, and the matching two lock dependency-list entries; all other manifest/lock changes are forbidden. |

## Clear areas observed

- The donor was confined to audited hunk recovery; wholesale publication was prohibited.
- Raw `lima-action` and helper bypasses were rejected directionally.
- Keychain/P-256/XPC requirements, peer admission before decode, and non-native planning posture were
  retained.

This discovery report is intentionally not a closure verdict. A different fresh reviewer must review
the remediated full subject.

## Fresh full-subject closure

A different fresh read-only reviewer, given no author reasoning or prior review conclusion, reviewed
the complete remediated eight-path subject fingerprint
`sha256:71c0b6794dcc4dc3b4e0df9269c9add66fdc03eb66957d0c002942ee6eee1234` and returned **CLEAN**:
no actionable findings. The closure reviewer performed no mutation and no native, installation,
Keychain, XPC, or Lima action.
