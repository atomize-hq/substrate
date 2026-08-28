# ChatGPT Pro advisory review: runtime-refactor D6 B1/B2.1 receipt-and-supervision family

- Date: 2026-08-25
- Landing chat: none (local landing only)
- Initial review chat: https://chatgpt.com/c/6a8d8e01-5278-83ea-a914-f04da69dbcbe
- Remediation follow-up review chat: https://chatgpt.com/c/6a8d95b9-a6f0-83ea-8a1e-14b388c62d1e
- Visible model on both review surfaces: `GPT-5.6 Sol`
- Visible effort setting (initial review): `Pro`
- Visible effort setting (remediation follow-up): `Pro`
- Initially reviewed patch digest: `sha256:6225c47eb90ec7d837b555699b23d2f34e6aa765a8c1ea6f59df9a90c9eef034`
- Remediation delta digest: `sha256:93cf8a462869f162f3c09ae5f9cab168e68811f0686dab3fa949b4c0fd8c4586`
- Family landing final local patch digest: `sha256:8dc6b4dbfb5be8bfd664de102021867971459296b8914984c748714eefb6e2da`
- Initial review prompt digest: `sha256:75bfc7a49ddd60e101899141dead8c6b712b73e743db8e87e1d297feee57d102`
- Remediation follow-up prompt digest: `sha256:4b1f16b2ec4bf74cb34332759489299b9985b56e7dc2d28d46f46186b0daad5f`

> Advisory only; verify against local project truth and authoritative docs; do not widen scope without fresh user authorization.

## D6 family landing reviewed

The bounded D6 landing applied only the B1/B2.1 receipt-and-supervision family on the local `df2cb3d430e333e045cdf85d2c6e19a14a55c71b` baseline in the same worktree. The live source-span inventory concluded that family-local canonical material remained in all six root compatibility documents and belonged together as one inseparable packet family.

The landing therefore:

1. replaced the family-local root sections in `00-README.md`, `01-target-architecture.md`, `02-seam-crosswalk.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, and `05-debug-regression-ledger.md` with compatibility forwarders;
2. created `b1-b2-1/README.md`;
3. created `b1-b2-1/current-state.md`;
4. created `b1-b2-1/architecture.md`;
5. created `b1-b2-1/crosswalk.md`;
6. created `b1-b2-1/slice-and-task.md`;
7. created `b1-b2-1/contracts-and-gates.md`;
8. created `b1-b2-1/evidence-regression.md`;
9. added the `B1-B2.1-family` row to `index/README.md`; and
10. added 27 D6 extraction-ledger rows to `migration/extraction-ledger.md`.

The landing preserved `A1.3-P1` as the sole active global implementation packet, preserved the exclusion of `B3.1`/`C1`, `A1.2`, and earlier histories, and did not reopen any out-of-scope implementation authority.

## Accepted blocking finding from the initial independent review

The fresh independent initial review reported one accepted blocking finding:

- `P2`: three extracted canonical owners retained source-position wording that became false after relocation:
  - `b1-b2-1/crosswalk.md#b32a-wa-bounded-seam-ownership-addendum` still referred to the broader seam rows as being "above" even though those broader rows remained root-canonical;
  - `b1-b2-1/contracts-and-gates.md#b1b21-bounded-read-only-dispatch-authority-adapter` still referred to the unnamed "startup rule above" without naming or linking the preserved root owner; and
  - `b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result` still referred to the completed-F ledger as being "below" even though the completed-F ledger remained outside this family.

No additional qualifying findings were identified.

## Local remediation applied

The remediation stayed inside the same worktree and kept the landing bounded:

1. changed only `b1-b2-1/crosswalk.md`, `b1-b2-1/contracts-and-gates.md`, and `b1-b2-1/evidence-regression.md`;
2. replaced the false positional references with explicit truthful rebased links to preserved canonical owners in `02-seam-crosswalk.md`, `04-contracts-and-gates.md`, and `a1.1d-5r2-2f/evidence-regression.md`; and
3. left the family fence, exclusions, chronology, and extracted bodies otherwise unchanged.

No other file changed during remediation.

## Local validation evidence

Initial landing validations:

- `git apply --check /tmp/d6-b1-b2-1-subagent.patch`
- `git diff --check`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b1-b2-1/README.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b1-b2-1/current-state.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b1-b2-1/architecture.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b1-b2-1/crosswalk.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b1-b2-1/slice-and-task.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b1-b2-1/contracts-and-gates.md`
- `git diff --no-index --check /dev/null llm-last-mile/runtime-refactor/b1-b2-1/evidence-regression.md`
- exact fifteen-path landing fence confirmed by `git status --short --untracked-files=all`
- destination-heading checks passed for all six canonical family docs
- `index/README.md` contained exactly one `B1-B2.1-family` row
- `migration/extraction-ledger.md` contained exactly 27 D6 rows pointing at `../b1-b2-1/`
- targeted root-forwarder checks passed in `00-README.md`, `01-target-architecture.md`, `02-seam-crosswalk.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, and `05-debug-regression-ledger.md`
- preserved B3.1/C1 sections and the `A1.3-P1` active-global-packet record remained present

Remediation validations:

- `git diff --check`
- `git apply --reverse --check /tmp/d6-b1-b2-1-reviewer-source-position-remediation.patch`
- targeted banned-phrase absence plus required-link presence checks for all three remediated files
- target-heading presence checks for:
  - `02-seam-crosswalk.md` `## A. Host authority and ingress`
  - `02-seam-crosswalk.md` `## B. Dispatch, policy, receipts, and retained runtime`
  - `04-contracts-and-gates.md` `### Lifecycle, retry, and fail-closed rules`
  - `a1.1d-5r2-2f/evidence-regression.md` `## A1.1d-5R2-2F final evidence and decision ledger`

## Remediation follow-up review verdict

The fresh remediation-follow-up review returned **`APPROVED`** with no qualifying findings.

D6's B1/B2.1 receipt-and-supervision family landing is therefore complete locally and approved. The remaining later D6 families are the logical `B3.1`/`C1` successor family and the later `A1.2`/earlier-histories family.
