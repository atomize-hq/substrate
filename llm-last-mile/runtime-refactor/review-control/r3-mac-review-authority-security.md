# R3 MAC authority and security discovery review

Discovery subject fingerprint (frozen before remediation):
`sha256:1097874180b86a3331c7d14c06d2b803cfd545273576e691b8eb1f1ec5bb4bca`

The fingerprint is the SHA-256 of the sorted pre-edit manifest at
`/tmp/a1-1d-5r3-mac-preedit-subject.manifest`: expected base commit followed by every exact
non-review subject path, Git mode (or `NEW`), and no-filter blob ID (or `MISSING`). Review-control
metadata is excluded to avoid self-reference.

A fresh, read-only `gpt-5.6-terra` reviewer using Extra High reasoning reviewed the exact
A1.1d-5R3-MAC authority fence. The selected outcome is limited to the manifest-bound MAC
publisher/lifecycle provider, PM-bound SSH-UDS activation, the named MAC scripts/unit/test fence,
and the five permitted R3 documentation status appends. It explicitly excludes native evidence,
Lima provisioning, alternate transport/selector activation, and any common/control schema change.

## Discovery findings

- **P1-MAC-001 — unmanaged SSH-UDS teardown.** The existing forwarding path uses ambient defaults,
  pathname-only unlink, SSH `StreamLocalBindUnlink`, and implicit Drop cleanup. The remediation
  must bind socket, child, and known-host mutation to the IH/PM-selected identity and receipt; it
  must preserve unowned/replaced state.
- **P1-MAC-002 — direct Lima lifecycle before PM authority.** `lima-warm.sh` can create/start and
  `lima-stop.sh` hard-codes the instance. All mutation must flow through the mapped executor with
  an existing-instance identity or `LimaStageOneAuthorizationV1`.
- **P1-MAC-003 — unreceipted guest remediation.** Guest DNS, package, Rustup, Cargo, and build
  fallback branches can mutate without exact `ExecutorBuildEvidenceV1`. They must be tombstoned
  and fail closed before guest mutation.
- **P1-MAC-004 — forbidden socket propagation.** Remove exactly the socket unit's `PartOf=` line
  and add no substitute propagation edge.
- **P2-MAC-001 — required non-native proof surface is absent.** Add the named fixture and required
  embedded checks while keeping the two run-only fixture scripts byte-identical.
- **P2-MAC-002 — the mandatory review-control record and three MAC lenses are absent.** This
  packet supplies them and validates the closed record after each cycle.
- **P2-MAC-003 — native lifecycle execution is outside this implementation packet.** Every proof
  must use static or non-executing mock identity; native XPC/publisher/Lima actions are deferred
  exclusively to `EVIDENCE:R3-MAC-IMP-01`.

No actual P3 or P4 finding is admitted. If a later valid P3/P4 occurs, the conflict between the
frozen MAC documentation surface and `06-review-finding-inventory.md` is an authority stop; this
packet must not widen the file fence silently.

Protocol discovery verdict: `FINDINGS`.

## Closure-1 delta review and remediation

A different fresh read-only reviewer assessed subject
`sha256:f4db3ee786ba2c0e8ace7bda5a1064705b90afa488ce1d8eac089126c786dc65` and returned
`FINDINGS`: `MAC-R3-P1-01` through `MAC-R3-P1-04` and `MAC-R3-P2-01`.

The bounded remediation keeps `mapped_attempt` inside `ForwardingKind::SshUds`, leaving the VSock
and TCP constructors byte-equivalent to their compatibility behavior. It removes caller-selectable
executor/evidence environment switches; the lifecycle wrapper now canonically validates carrier,
mapping, prefix joins, and the full non-executing `ExecutorBuildEvidenceV1` shape before dispatch.
The fixed plist command and executor now bind `run-publisher` to the fixed Mach-service listener
operation rather than a one-shot launchd command.

The reviewer also identified that a helper response must not be treated as a substitute for an XPC
peer audit token. The supplemental review must independently verify the fixed listener/request
binding and reject any ambient or alternate transport claim. No native XPC action was run here.

Protocol closure-1 verdict: `FINDINGS`; all findings are remediated subject to the authorized
supplemental causal closure.

## Supplemental-causal-1 review

A fresh read-only reviewer assessed subject
`sha256:1cabe3430ef0a28a88ef560cca26cfc4d934c1107ca2512d66f22c126583fa31` and returned
`FINDINGS`: `P1-MAC-SUP-001`, `P1-MAC-SUP-002`, `P2-MAC-SUP-003`, and
`P2-MAC-SUP-004`.

The valid P1 findings are confined to the direct remediation surface: the helper's current stdio
relay is not an XPC request channel or peer attestation, and warm-stage VM selection still enters
via an ambient value before its mapped authority is joined. The valid P2 findings identify the
causal consequences in the failed-attempt identity capture and non-native protocol proof.

No P3/P4 finding was admitted. A second supplemental causal cycle is authorized only for the
bounded repair of these direct findings; no new principal, selector, transport, dependency, or
native evidence work is authorized.


## Supplemental-causal-2 final review — bounded stop

A different fresh read-only `gpt-5.6-terra` reviewer using Extra High reasoning assessed final
subject `sha256:e60963026aa39be67e334119ff7ae61f3f15f19dbfd31539d790f8735b493924` and returned
`FINDINGS`.

- **P1-MAC-FINAL-001 — XPC peer/audit attestation is not bound to the actual peer.** The
  listener handler accepts no connection audit token, and `mac_audit_token_ffi_v1` obtains the
  daemon's own PID rather than a peer identity. The asserted `audit_token` response is
  self-authored, while the designated requirement selects the publisher helper although normal
  requests originate in the shell client.
- **P1-MAC-FINAL-002 — privileged lifecycle dispatch remains raw and unjoined.**
  `execute_mac_managed_action_v1` takes raw JSON, checks only loose evidence fields, and can
  mutate protected state without a canonical carrier/mapping/action/Stage-1 join. A direct
  `retire-test-publisher` command also opens lifecycle state outside the XPC request handler.

These are P1 findings. Resolving them requires actual connection-audit-token API/binding and
canonical authority validation that are not currently proven by the permitted MAC surface. This
is the second supplemental causal cycle; the frozen review budget is exhausted. No further
remediation is authorized. No P3/P4 finding was admitted.

Protocol supplemental-causal-2 verdict: `FINDINGS`; terminal control disposition:
`bounded_stop` / `budget_exhausted`.
