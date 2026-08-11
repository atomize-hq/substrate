# AUX-R3 macOS launchd service-state correction: authority/security review

Packet: `AUX-R3-MAC-LAUNCHD-SERVICE-STATE-CORRECTION`

## Independent discovery review

- Reviewer: fresh read-only `gpt-5.4`, reasoning effort `xhigh`
- Subject fingerprint: `sha256:067debad07a4db9d7e52fc09a1447f38bc449129f0c6bc2b70149e973d16296c`
- Result: `FINDINGS` (`P1=0`, `P2=2`)

| Finding | Priority | Disposition |
| --- | --- | --- |
| `AUX-R3-MAC-LAUNCHD-001` generic coded `launchctl print` failures were treated as absence | P2 | Remediated. Only exact exit status 113 is absence; every other failure is indeterminate and preserve-stops. Both control and executor have a focused regression. |
| `AUX-R3-MAC-LAUNCHD-002` retirement ignored a failed `bootout` result | P2 | Remediated. Both effect-executing branches require successful fixed `bootout`; failure leaves the protected retirement precommit intact. |

The reviewer confirmed that the no-input FD3 route, fixed bootstrap/bootout operands, and absence of raw installer `launchctl` mutation were aligned with the authority boundary.

## Final authority shape

The caller can select only `publisher-service-state-install` or `publisher-service-state-retire` at the local control binary. The privileged executor receives one EOF-only AF_UNIX descriptor and accepts no serialized label, domain, path, program, action, command, or extra field. The executor re-attests the peer/session and joins the signed Stage-1 attempt, completed bootstrap intent, fixed provenance, exact installed file identities, code requirement, and protected Keychain state before changing launchd.

The only launchd effects are the compiled commands:

- `/bin/launchctl bootstrap system /Library/LaunchDaemons/com.substrate.lifecycle.publisher.v1.plist`
- `/bin/launchctl bootout system/com.substrate.lifecycle.publisher.v1`

`launchctl print` output is discarded. Only process exit supplies the bounded presence observation; no undocumented body field is parsed as structured identity. `kickstart`, `load`, `unload`, arbitrary shell launchctl, and caller-provided launchd operands are absent.

## Closure

The first fresh closure review bound to
`sha256:4324888c23f95cdb77b9b1a7461ea308e2a4c3075285deeb1a8327512e2abc34`
found `AUX-R3-MAC-LAUNCHD-006` (P2): service-state rereads did not yet prove the
unique predecessor chain or revalidate their retained bootstrap-attempt, Stage-1 capsule,
bootstrap-intent, and protected commitment joins. The remediation adds the fixed locator account
to the admission and receipt, derives every legal predecessor from the canonical install
precommit, and reconstructs the retained locator, issued capsule, and completed intent before an
existing receipt can authorize idempotent success or retirement.

A different fresh read-only `gpt-5.4` reviewer at reasoning effort `xhigh` reviewed the remediated
subject fingerprint
`sha256:b9dee2fe0beb155838a4a9267ed914090629c6aafa73d675f67cc863c182e5ce`
and returned terminal `CLEAN` with `P1=0`, `P2=0`, and no P3/P4 findings.
