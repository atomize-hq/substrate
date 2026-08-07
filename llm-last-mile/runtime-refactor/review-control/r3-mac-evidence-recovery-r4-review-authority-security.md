# R4 authority and security review

Packet: `AUX-R3-MAC-EVIDENCE-RECOVERY-R4`
Subject base: `b12cb6c6e2dc326b4150291ef30676171df34f58`

## Discovery and remediation lineage

The discovery subject (`sha256:5dfc6dffba611f8a4d445880aa4247f0e47eefaa1aa240709a5057781badf390`)
contained two blocking authority defects:

- `P1-R4-CONTROL-IMAGE-BINDING`: the fixed XPC peer requirement did not bind the one
  protected control image.
- `P1-R4-HOST-RECORD-EXPIRY`: the terminal host-record transition did not receive an
  explicit clock.

The implementation now validates a protected canonical authority whose requirement pins the fixed
publisher identifier and a lowercase 40-hex CodeDirectory hash. The XPC listener obtains its
requirement from that canonical System-Keychain record, cancels on a nonzero
`xpc_connection_set_peer_code_signing_requirement` result before activation, reads the actual peer
audit token, and before the first operation or request-byte decode uses
`SecCodeCreateWithXPCMessage` plus `SecCodeCheckValidity`. The macOS SDK documents that the first
API constructs `SecCode` from the sending XPC message's associated audit token. No PID-path lookup
or caller-supplied identity participates in admission.

Closure-1 (`sha256:dba6d99703255dfca2dbc366c4832ece91df85ea7c6dfca91c69285666855b39`) found
`P1-R4-AUDITED-PEER-PATH-TOCTOU`: the interim `proc_pidpath` artifact hash did not bind the
audited running image. The direct `SecCode` check above remediated it. The first supplemental
review then found the exact symbol-fence issue recorded separately in the allowlist review.

## Final independent conclusion

Final supplemental closure subject
`sha256:efd2f5b317b53cb79e4738f46f2633d0403e6d182f11339cc36ab708690ecc52` is **CLEAN for
P1/P2**. The Security/CoreFoundation objects created for requirement and code validation are
owned by the local reverse-release `OwnedCf` guard. The host private P-256 key remains
non-exportable: only `SecKeyCopyPublicKey` may be externally represented, and the resulting point
is parsed as canonical P-256 SPKI DER.

The reviewer recorded `P3-R4-CLOSURE-RESPONSE-METADATA`: rejected-response metadata still prints
the older generic requirement constant while actual admission enforces the stronger protected
identifier-and-CDHash requirement. It is not an admission or effect path. Per the review contract,
it is recorded here only and creates neither remediation nor another review cycle.

No Keychain, XPC service, Lima, install, code-signing, or native evidence action was performed
while reviewing.
