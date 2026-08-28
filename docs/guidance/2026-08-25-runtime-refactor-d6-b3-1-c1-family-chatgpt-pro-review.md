# ChatGPT Pro advisory review: runtime-refactor D6 B3.1/C1 successor family

- Date: 2026-08-25
- Landing chat: none (local landing only)
- Initial review chat: https://chatgpt.com/c/6a8d9a4f-5ee0-83ea-a448-51836019aa73
- Remediation follow-up review chat: https://chatgpt.com/c/6a8d9e6a-fe3c-83ea-b8f6-049576b5b3b4
- Visible model on both review surfaces: `GPT-5.6 Sol`
- Visible effort setting (initial review): `Pro`
- Visible effort setting (remediation follow-up): `Pro`
- Initially reviewed patch digest: `sha256:770e5f72de34e8e77957b4a45f7b7b81cc5b0db220e514a15b80829e01464fe4`
- Remediation delta digest: `sha256:07c75ea7c2b2af80ffeaa087595bc5591d169e03387154460302f39b75f41dba`
- Initial review prompt digest: `sha256:00936b4e281a211a29af0091becb31dc69e9820c2bc8f7f8e428c3057a3d68f3`
- Remediation follow-up prompt digest: `sha256:1026a70314d7281d2061ba252d8999041bfa9ddac2af3a96e580ce12fba4d83b`

> Advisory only; verify against local project truth and authoritative docs; do not widen scope without fresh user authorization.

## D6 family landing reviewed

The bounded D6 landing applied only the B3.1/C1 successor family on the live post-B1/B2.1 local baseline in the same `df2cb3d430e333e045cdf85d2c6e19a14a55c71b` worktree. The live source-span inventory concluded that the remaining B3.1/C1-family-local canonical material was distributed across all six root compatibility documents plus the packet index and extraction ledger, and that B3.1/C1 had to land together as one successor family.

The landing therefore:

1. replaced the family-local root sections in `00-README.md`, `01-target-architecture.md`, `02-seam-crosswalk.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, and `05-debug-regression-ledger.md` with compatibility forwarders;
2. created `b3-1-c1/README.md`;
3. created `b3-1-c1/current-state.md`;
4. created `b3-1-c1/architecture.md`;
5. created `b3-1-c1/crosswalk.md`;
6. created `b3-1-c1/slice-and-task.md`;
7. created `b3-1-c1/contracts-and-gates.md`;
8. created `b3-1-c1/evidence-regression.md`;
9. added the `B3.1-C1-family` row to `index/README.md`; and
10. added 12 D6 extraction-ledger rows to `migration/extraction-ledger.md`.

The landing preserved `B1-B2.1-family` as the predecessor family, preserved `A1.3-P1` as the sole active global implementation packet, and kept `B2.2`, remaining `B3.2`/`B4`, `A1.2`, earlier histories, R3, grouped R2/F families, and D7+ out of scope.

## Accepted qualifying findings from the initial independent review

The fresh independent initial review reported two accepted qualifying findings:

1. `index/README.md` placed the new `B3.1-C1-family` row after `B1-B2.1-family`, which violated the required reverse active chronology for a successor family.
2. `b3-1-c1/evidence-regression.md` retained false destination-local references that A1.2b was “recorded below” even though the A1.2b record remained outside this extracted family under the preserved root `05-debug-regression-ledger.md` canonical owner.

No additional qualifying findings were identified.

## Local remediation applied

The remediation stayed inside the same worktree and kept the landing bounded:

1. changed only `index/README.md` and `b3-1-c1/evidence-regression.md`;
2. moved the `B3.1-C1-family` packet-index row ahead of `B1-B2.1-family` so reverse active chronology matches the declared predecessor/successor relation; and
3. replaced both false A1.2b positional references with explicit truthful preserved-root links to `05-debug-regression-ledger.md#a12b-recorded-result`.

No other file changed during remediation.

## Local validation evidence

Initial landing validations:

- `git apply --check /tmp/d6-b3-1-c1-family.patch`
- `git apply --reverse --check /tmp/d6-b3-1-c1-family.patch`
- `git diff --check`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b3-1-c1/README.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b3-1-c1/current-state.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b3-1-c1/architecture.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b3-1-c1/crosswalk.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b3-1-c1/slice-and-task.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b3-1-c1/contracts-and-gates.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b3-1-c1/evidence-regression.md`
- `python3 /tmp/validate_d6_b3_c1.py`
  - validated 12 source-span hashes
  - validated 12 destination bodies
  - validated 12 D6 extraction-ledger rows
  - validated the 15-path patch fence
  - validated forwarder links and anchors
- root pointer-table column-count checks
- byte-stability comparison for every `b1-b2-1/*.md` predecessor file
- exclusion checks for guidance artifacts, review-control evidence, `index/current.md`, and D5 paths

Remediation validations:

- `git diff --check`
- `git apply --reverse --check /tmp/d6-b3-1-c1-review-remediation.patch`
- `B3.1-C1-family` before `B1-B2.1-family` order check in `index/README.md`
- predecessor/successor wording preservation checks in `index/README.md`
- banned-phrase absence + exact preserved-root link-count checks in `b3-1-c1/evidence-regression.md`
- preserved root heading existence check for `05-debug-regression-ledger.md` `## A1.2b recorded result`

## Remediation follow-up review verdict

The fresh remediation-follow-up review returned **`APPROVED.`** with no qualifying findings.

D6's B3.1/C1 successor-family landing is therefore complete locally and approved. The only remaining later D6 family is the `A1.2`/earlier-histories grouping.
