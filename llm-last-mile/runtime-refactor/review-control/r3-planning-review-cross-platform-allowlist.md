# R3 planning cross-platform and allowlist review

Terminal subject fingerprint:
`sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`

A fresh independent read-only gpt-5.4 Extra High reviewer using standard speed recomputed the
fingerprint and checked Linux, macOS/Lima, Windows/WSL, exact file/symbol/test fences, native
evidence and restoration, and the planning-only changed-file boundary. It authored no subject
byte.

The causal protocol ends `CLEAN` with zero unresolved P1/P2; P3/P4 do not start another causal
cycle. The separate A1.1d-5R3-PLAN publication gate requires zero unresolved P1-P4. The terminal
subject satisfies both, and this report preserves the distinction.

## Recorded causal lineage

At discovery fingerprint
`sha256:da97c6ca51b27a61fb1e0f64050030a46f84b38329136ffc337f263a0fe0576d`,
`R3-CPA-002` (P1) found Windows forwarder-log/release temp trees and Unix release temp roots outside
authorized role/action fences with no captured before-state or safe early-failure authority.
`R3-CPA-004` (P1) found that owner-helper teardown demanded durable launch identity that no packet
owned and an in-memory receipt could not supply. `R3-CPA-003` (P2) found a native receipt join to a
nonexistent evidence field rather than the validator's actual artifact digest field. The JSON
record preserves these with the other discovery P1/P2 IDs; `R3-AS-006-DISCOVERY` is explicitly an
occurrence alias for the raw reviewer's first duplicate `R3-AS-006`, not a rewritten finding.

At closure fingerprint
`sha256:e2b91149587e8d23dab2455de443e56383506d75bac041ca1402ecee71e43835`,
the discovery set was closed and exactly `R3-AS-006` and `R3-AS-007` remained P1. They alone are
the trigger IDs for the final changed-subject `supplemental_causal` cycle.

## Supplemental causal evidence

The Windows and Unix creator ranges now route through exact handle/descriptor-bound
`CurrentAttemptTempRollbackV1`; forwarder-log directories have one closed managed role and exact
before-state. Unsupported owner-helper kill authority is removed rather than invented:
`--kill-live-processes` and the legacy helper-kill path stop as `BLOCKED_SCOPE_EXPANSION` before
enumeration or signal. Native evidence artifacts and receipts are validated separately, and the
receipt joins exact field `evidence.artifact_sha256` using the orchestration validator's real
interface.

Linux, MAC, WIN, UNIX, three provider closeouts, three final native evidence gates, and final
closeout have disjoint exact mutation fences. All current world socket sources with the legacy
`PartOf=` edge have a singular packet-owned exact removal; the new WSL socket source omits every
propagation edge. Linux, Lima, and WSL packet/native tests prove coupled socket-state/endpoint
prepared records, endpoint non-requestability, mismatch preservation, kill/retry, and service-stop
nonmutation. Publisher host/guest endpoints are similarly confined to protected bootstrap and
retirement. WSL service installation separately binds template-source digest, PM inputs, rendered
service digest, socket-source digest, and unchanged installed-socket digest, with hostile ambient
and template/socket substitution negatives. MAC/WIN evidence alone may build and attest native
executors before UNIX distribution. Static review never substitutes for native proof, and absent
platforms retain `BLOCKED_PLATFORM_HANDOFF_REQUIRED`.

The terminal tracked subject diff contains only the six planning documents. No production, test,
fixture, script, dependency, schema, generated product artifact, or platform state was changed.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
