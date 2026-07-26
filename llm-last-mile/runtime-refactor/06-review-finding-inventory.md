# Review Finding Inventory

## Purpose

This is the single durable inventory for valid, non-blocking `P3` and `P4` findings discovered
while reviewing runtime-refactor packets. It is separate from
[`05-debug-regression-ledger.md`](05-debug-regression-ledger.md): `05` owns product/runtime defects,
proof gates, and regression obligations, while this file owns optional review and process debt.

This inventory is not an acceptance bypass. An entry cannot waive, downgrade, or conceal a `P1`
or `P2`, and no blocking finding may be placed here to complete the current packet.

## Priority and status

| Priority | Meaning | Completion effect |
|---|---|---|
| `P3` | Useful hardening or improvement without demonstrated failure of the selected integrated outcome | Non-blocking; retain here unless fixed safely inside the authorized packet |
| `P4` | Minor polish, naming, formatting, or consistency issue with no correctness effect | Non-blocking; retain here unless a deterministic local correction is already authorized |

Statuses are `open`, `accepted`, `scheduled`, `resolved`, or `superseded`. `accepted` means the
finding is consciously retained as non-blocking debt; it does not mean the underlying concern is
false or fixed.

## Admission and maintenance rules

1. The parent validates each finding against current authority, live repository truth, and the
   selected integrated outcome before admitting it.
2. Reviewer wording alone does not set priority. The raw review and its original verdict remain
   immutable evidence when the parent or human disposition assigns a different current priority.
3. `P1` and `P2` never enter this inventory as unresolved debt for a completed packet. They require
   remediation and closure review or a bounded non-completed stop.
4. A valid unfixed `P3` or `P4` uses one stable `RR-RF-####` ID. Rediscovery appends evidence to the
   existing row when scope, violated concern, and failure mode are the same.
5. New evidence may promote an inventory item to `P1` or `P2`. The active packet then follows the
   blocking workflow in `04`; this inventory supplies history, not a waiver.
6. A resolution records the fixing commit or artifact and the review/proof that closed it. Do not
   delete historical rows.
7. Inventory maintenance is mechanical only when it transcribes an already-dispositioned finding
   without changing priority, scope, acceptance, or contract meaning. Any such semantic change is
   material and requires review.

## Inventory

| ID | Priority | Status | Scope / packet | Summary | Evidence | Why non-blocking | Target |
|---|---|---|---|---|---|---|---|
| `RR-RF-0001` | `P3` | `accepted` | RP4 cache-only proof supervisor | Bind the reviewed controller bytes to execution rather than trusting a pathname plus earlier hash. | [`05` RP4 ledger](05-debug-regression-ledger.md#rp3rp4rp5-closeout-ledger); `/home/spenser/.cache/substrate-runtime-refactor/rp4-rp5-closeout-20260725T184536Z/rp4-attempt8-review-persistence.txt` | The affected file is bespoke cache-only orchestration, not Substrate product code or the canonical wall runner. No substitution was observed, and the focused, authenticated, canonical-wall, differential, authority, and restoration results remain independently inspectable. | Unassigned process-tooling hardening; no automatic remediation authority |
| `RR-RF-0002` | `P3` | `accepted` | RP4 cache-only proof supervisor | Fsync the parent directory after atomic manifest/result replacement before claiming crash durability. | [`05` RP4 ledger](05-debug-regression-ledger.md#rp3rp4rp5-closeout-ledger); `/home/spenser/.cache/substrate-runtime-refactor/rp4-rp5-closeout-20260725T184536Z/rp4-attempt8-review-persistence.txt` | The concern affects only crash durability of bespoke cache artifacts. The reviewed files survived and matched their recorded hashes; no product/runtime or canonical-wall result is invalidated. | Unassigned process-tooling hardening; no automatic remediation authority |
| `RR-RF-0003` | `P3` | `accepted` | RP4 cache-only proof controller | Persist the final post-persistence verdict in the controller packet rather than only in memory. | [`05` RP4 ledger](05-debug-regression-ledger.md#rp3rp4rp5-closeout-ledger); `/home/spenser/.cache/substrate-runtime-refactor/rp4-rp5-closeout-20260725T184536Z/rp4-attempt8-review-persistence.txt` | The persisted packet still contains the independently evaluable clean proof results; the omission weakens self-attestation/reporting only and does not change those results. | Unassigned process-tooling hardening; no automatic remediation authority |
| `RR-RF-0004` | `P3` | `accepted` | Review-control validator CLI | Add automated CLI-level tests for `--next-cycle supplemental_causal` with missing and supplied `--causal-evidence-ref`; current tests exercise `validate_next_cycle` directly. | Different-fresh closure review `/root/closure_review_process_calibration`; [`test_validate_review_cycle.py`](review-control/test_validate_review_cycle.py) | The closure reviewer manually verified that the CLI rejects missing causal evidence and accepts a supplied ref. Current behavior is correct; the gap affects future regression detection only. | Future review-control test hardening; no automatic remediation cycle |
