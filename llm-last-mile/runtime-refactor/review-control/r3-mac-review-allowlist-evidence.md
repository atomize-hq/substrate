# A1.1d-5R3-MAC fresh MAC review epoch

This is the nonce-bound attempt-4 review epoch under
`substrate-a1-1d-5r3-mac-20260806-41f97e1c570d` authority amendment
`0003-fresh-mac-review-epoch.json`. The prior four-cycle bounded-stop record is immutable
predecessor evidence; its byte digest is `sha256:bc7258fb432375e50af899f14691c7691980d4f99a8174b8e2d6ee2e1a29e003`. This fresh record does not append a fifth
historical cycle. No native evidence, publisher installation, code-signing, Keychain action,
launchd action, Lima action, or successor dispatch is included.

## Discovery allowlist and evidence disposition

Discovery subject: `sha256:8da3cf1e8cb7d043d86ada4e63a08a5c5df6283e45dd904cf9b72c10810f60d2`. Its frozen non-review set contains the exact 16 production, test,
and R3-DOCS paths listed by the packet contract. The post-subject review set is exactly this
document, `r3-mac-review-authority-security.md`,
`r3-mac-review-lifecycle-convergence.md`, and `r3-mac-review-cycle-record.json`.

The independent review confirmed all subject blobs against the manifest and found no file-fence
escape, native evidence artifact, or successor implementation artifact. The non-native fixture
checks direct-XPC carrier hashing, peer-audit ordering, bootstrap response attestation, and
no-follow/preserve behavior; the forwarding tests exercise pre-spawn replacement, child-owned
early-exit cleanup, retry, timeout, Drop, and mapped-only selection. The frozen run-only
`prefix_mapping_r2_3.sh` retains its bytes but is incompatible with the new mandatory authority
arguments and stops at its historical no-authority invocation; this is recorded as a focused
regression incompatibility, not silently repaired. `lima_doctor_fixture.sh` passes.

No P3/P4 finding is recorded in `06-review-finding-inventory.md` because none was returned.


## Closure-1 review

Closure subject `sha256:c1f700f483ee59529c41b2f797815b903ab48cd72fffe22ee6f252ab70230765` stayed in the same
16-path subject and exact four-file review set. The sole returned P1 is causal to the discovery
Drop/descriptor remediation. No P3/P4 finding, evidence artifact, selector/transport addition,
or native action was returned.


## Supplemental-causal-1 review

The subject remained `sha256:5eed302e680a58332b210f2b4066ba8c8f3b13dba96cc20c22507dbc62e1e002`
and within the same exact 16-path/non-native fence. The sole P2 is causal to the just-added
descriptor capture. No P3/P4, native evidence, or selector/transport finding was returned.


## Supplemental-causal-2 final review

Final fresh review returned **CLEAN** for subject
`sha256:2f504d6b21503396af7b48276ec238de88f43f22dc4ac2c824131dce1252502c`. All 16 exact
non-review blobs and the exact MAC review-control set remained within fence; no evidence artifact
or native action exists. The review record is complete and no P3/P4 inventory item exists.
