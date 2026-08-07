# Lifecycle/convergence discovery review — AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN

## Review boundary

Fresh read-only discovery review of the full initial eight-path planning subject. The reviewer ran
no native, installation, Keychain, XPC, Lima, source, or Git action. Its independently constructed
fingerprint differed from the supplied initial fingerprint; this is preserved as a discovery finding,
not a clean-subject attestation.

## Findings classified against the selected outcome

| ID | Priority | Disposition | Evidence and required recovery-plan correction |
|---|---|---|---|
| `P1-PLN-001` | P1 | duplicate reported in authority report; remediated once | Eight-file task definition conflicted with earlier six-file shorthand. The recovery contract must make the eight literal paths and one deterministic hash algorithm authoritative. |
| `P1-LCR-002` | P1 | remediated in the consolidated planning pass | R5 carried `LimaStageOneAuthorizationV1` before every action even though Stage-1 is a one-time pre-PM create exception. Split create-only and post-PM branches; prohibit Stage-1 in forwarding/post-PM and require final-PM publication before projection. |
| `P2-LCR-003` | P2 | remediated in the consolidated planning pass | Dual-session retry/transcript rules were generic. Add durable phases for pre-intent, durable hello, durable transcript, and consumed ticket; fix exact nonce/key/transcript reuse and kill/reconnect tests. |
| `P2-LCR-004` | P2 | remediated in the consolidated planning pass | No packet explicitly owned known-host/SSH-UDS descriptor preservation across unlink, timeout, Drop, and retry. Assign it to R5 with no-follow/receipt-identity regressions. |

## Clear areas observed

- The direction of distinct PM-bound data and operator-TTY sessions was sound.
- The plan separated non-native source proof from official native evidence.
- The donor's one-stdio premise and raw helper action were excluded.

This discovery report is intentionally not a closure verdict. A different fresh reviewer must review
the remediated full subject.

## Fresh full-subject closure

A different fresh read-only reviewer, given no author reasoning or prior review conclusion, reviewed
the complete remediated eight-path subject fingerprint
`sha256:71c0b6794dcc4dc3b4e0df9269c9add66fdc03eb66957d0c002942ee6eee1234` and returned **CLEAN**:
no actionable findings. The closure reviewer performed no mutation and no native, installation,
Keychain, XPC, or Lima action.
