# AUX-R3 macOS launchd service-state correction: lifecycle/convergence review

Packet: `AUX-R3-MAC-LAUNCHD-SERVICE-STATE-CORRECTION`

## Independent discovery review

- Reviewer: fresh read-only `gpt-5.4`, reasoning effort `xhigh`
- Subject fingerprint: `sha256:3ed4e3c1ff6cd03af99d2e11230bd775bb52ebec594ce5b6d27f1445f12b7764`
- Result: `FINDINGS` (`P1=0`, `P2=1`)

`AUX-R3-MAC-LAUNCHD-003` (P2) found that sequential fixed-file unlink could strand a legitimate retirement after interruption. The implementation now stores `retirement_delete_cursor` in every deny-unknown-fields service-state receipt. Each fixed pathname is authorized by a protected Keychain CAS/readback before its unlink. A retry accepts absence only for the already-authorized prefix, exact-verifies every still-present receipted file, and resumes in the fixed order plist, helper, provenance.

## Exact protected chain

| Phase | Revision | Delete cursor | Previous digest |
| --- | ---: | ---: | --- |
| `install_precommitted` | 1 | 0 | absent |
| `installed` | 2 | 0 | required |
| `retirement_precommitted` | 3 | 0 | required |
| `retirement_precommitted` | 4 | 1 | required |
| `retirement_precommitted` | 5 | 2 | required |
| `retirement_precommitted` | 6 | 3 | required |
| `retired` | 7 | 3 | required |

All other phase/revision/cursor/predecessor combinations reject during canonical receipt validation. Registered service plus a nonzero deletion cursor preserve-stops. An absent path at or beyond the cursor preserve-stops. A present path at any cursor must exact-match the receipted no-follow root-owned identity, content, code identity/requirement where applicable, and provenance joins. `retired` commits only after cursor 3 and definite launchd/file absence.

## Retry and retirement convergence

- Absent service plus exact files and an install precommit resumes fixed bootstrap.
- Registered service plus exact installed receipt is idempotent success.
- Registered service plus install precommit remains an effect-visible/CAS gap and preserve-stops.
- Absent service plus installed receipt is unexpected drift and preserve-stops.
- Retirement writes its intent before successful fixed bootout.
- After definite service absence, each unlink has its own preceding durable CAS authorization.
- Foreign, mismatched, unreceipted, unexpectedly absent, or indeterminate state is never deleted.

## Closure

A different fresh read-only `gpt-5.4` reviewer at reasoning effort `xhigh` reviewed the final
non-review subject fingerprint
`sha256:b9dee2fe0beb155838a4a9267ed914090629c6aafa73d675f67cc863c182e5ce`
after the predecessor/retained-authority remediation and returned terminal `CLEAN` with `P1=0`,
`P2=0`, and no P3/P4 findings.
