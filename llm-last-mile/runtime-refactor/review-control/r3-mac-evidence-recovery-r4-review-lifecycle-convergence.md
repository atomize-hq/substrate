# R4 lifecycle and convergence review

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R4`
Subject base: `b12cb6c6e2dc326b4150291ef30676171df34f58`

## Discovery disposition

The discovery review bound to
`sha256:5dfc6dffba611f8a4d445880aa4247f0e47eefaa1aa240709a5057781badf390` found that an
expired signed ticket could enter the pure terminal host-record transition. It is tracked as
`P1-R4-HOST-RECORD-EXPIRY`. The remedy introduced the explicit-time
`compare_and_swap_guest_publisher_pairing_host_record_at_v1` check and a focused expired-transition
test; an expired ticket now fails before the ticket-issued to ticket-consumed transition.

The discovery lens also considered two scope-boundary observations. They are not open P1/P2
findings in the final record:

1. The common host-record compare-and-swap is intentionally the signed canonical,
   predecessor-digest, generation, expiry, and one-use *core*. R4 does not open a pairing
   session, perform an external effect, or provide a durable consumer endpoint. A future durable
   effect gate must own its protected store and invoke the explicit-time transition; adding it to
   R4 would require a prohibited authority carrier or lifecycle surface.
2. Full ticket construction requires the separately excluded typed R5 issuer carrier. R4 instead
   opens the fixed Stage-1 record and protected state, verifies scope, PM/host/source/artifact
   joins, Stage-1 expiry, and System-Keychain SPKI binding, then fails closed before issuance.
   It introduces no generic request wrapper, channel/session protocol, mapped action, or guest
   command.

## Final convergence conclusion

The final supplemental closure subject
`sha256:efd2f5b317b53cb79e4738f46f2633d0403e6d182f11339cc36ab708690ecc52` is **CLEAN for
P1/P2**. It confirmed canonical fixed-service and scope accounts, non-exportable P-256 public
SPKI export, P1363 low-S validation, Stage-1-before-issue ordering, canonical protected-state
CAS, and fail-closed R5 boundary. The signed host-record core rejects replay and stale generation
and carries no transport or session material.

The following deterministic proof was observed without native state effects: four focused
`substrate-common` mac tests, the one inline lifecycle test, `tests/mac/lifecycle_r3.sh`, native
`aarch64-apple-darwin` compile/test proof, and `x86_64-apple-darwin` compile proof. No Linux or
Windows target command was run.
