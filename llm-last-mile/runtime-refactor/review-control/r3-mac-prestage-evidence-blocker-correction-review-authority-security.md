# R3 MAC prestage evidence-blocker correction — authority/security review

Packet: `AUX-R3-MAC-PRESTAGE-EVIDENCE-BLOCKER-CORRECTION`
Authority: 0046 correction contract plus 0047 fixed-install-sequence continuation
Final non-review subject: `sha256:414537800707e8c90904ea38cb375540f74fa1a1f7aa08b6ae96ed84eb7638f9`

## Discovery and remediation lineage

The discovery three-lens review found no new authority family, selector, transport, binary target,
or generic scheduler. It found the missing preserved `lima-stop -> post_pm_action` branch and two
in-fence evidence issues (caller-controlled build root and insufficient retained-bundle faults).
The correction uses the existing signed Stage-1 successor only: the privileged executor derives a
literal forward-only installation plan, while the ordinary post-PM catalogue remains separately
available to the existing stop route.

Closure found that ordinary post-PM requests were issued from a stale pre-private-install anchor.
The corrected normal and resume paths refresh that ordinary catalogue only from the final protected
anchor. A first supplemental review then found the receipt/index/head crash window. The final code
admits only either the canonical current-anchor prefix or exactly one canonical prepared tail that
joins `previous_index_sha256`, `previous_head_sha256`, the signed prepared record, exact planned
receipt, and counter. It routes that tail through the existing idempotent policy executor and
re-reads state before considering the next fixed step.

The final supplemental reviewer initially observed an executable-bit regression on
`scripts/mac/lima-lifecycle.sh`. This was the contract-authorized small clerical in-fence error:
the owner restored mode `100755` and added an executable-bit regression to
`tests/mac/prestage_artifact_route_r3.sh`. The reviewer then reconfirmed CLEAN. No P1/P2 remains.

## Security conclusions

- Neither installer nor warm accepts a caller-selected role/action/plan; neither traverses
  `post_pm_requests_v1` as installation work.
- The private plan contains only fixed `Create` effects plus required socket `Enable` and `Start`.
  It contains no `Stop`, `Remove`, `Restore`, repair, or caller-selected action.
- The retained AArch64 bundle is exactly four existing binaries; the unsupported
  `mac.lima.guest-binary(world)` role and any world target/path are absent from product code.
- The external target build uses the fixed `/private/tmp` build root and no caller `TMPDIR` route.
  No credentials were added.

Reviewers were requested at gpt-5.4 Extra High; that model was unavailable in this runtime, so the
fresh reviewers recorded the default-runtime deviation. No reviewer authored the implementation it
reviewed.
