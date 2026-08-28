# ChatGPT Pro advisory review: runtime-refactor D6 R3 macOS batched family landing

- Date: 2026-08-25
- Landing chat: none (local landing only)
- Initial review chat: https://chatgpt.com/c/6a8cd95b-6850-83ea-869a-2bdc5b57de83
- Remediation follow-up review chat: https://chatgpt.com/c/6a8cdfb5-c874-83ea-aca2-e03c7c11c3ed
- Visible model on the captured review surfaces: not surfaced in the DOM snapshots used for this landing
- Visible effort setting on both review surfaces: `Pro`
- Initial review delivery form: fresh conversation with nine pasted-Markdown delta attachments plus a bounded prompt; no single authoritative initial-patch file digest was retained because the first local intermediary patch had malformed headers and was not used for final truth claims
- Remediation-only follow-up patch digest: `sha256:89eb4a6e0551758f62863c88823e41d732d923223482c02366fb3b9f62245b11`
- Remediation follow-up prompt digest: `sha256:ecd8f8ac801be26139007cb788e8bce57aa50638a8941c6710e69a2aa0426f67`

> Advisory only; verify against local project truth and authoritative docs; do not widen scope without fresh user authorization.

## D6 families reviewed

This bounded Tuesday, August 25, 2026 landing applied only the explicitly user-batched adjacent R3 families inside the same live worktree that already contained the approved but uncommitted D6 macOS developer-parity landing:

1. `R3 macOS evidence-recovery planning records`; and
2. `R3 manifest/platform/lifecycle/closeout families`.

The landing stayed in `/Users/spensermcconnell/.codex/worktrees/bae3/substrate`, did not create or switch branches, and preserved the same detached `HEAD` at `0c47467db9917d16a2b8250574f57e4d4af6e16a`.

Within that scope, the landing:

1. created the canonical R3 implementation-family documentation bundle under `llm-last-mile/runtime-refactor/a1.1d-5r3/`;
2. created the canonical R3 macOS evidence-recovery projections under `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/`;
3. replaced the family-local root sections in `00-README.md`, `01-target-architecture.md`, `02-seam-crosswalk.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, and `05-debug-regression-ledger.md` with compatibility forwarders;
4. added the `A1.1d-5R3-family` and `r3-mac-evidence-recovery-family` packet-index rows; and
5. added the matching D6 extraction-ledger rows.

The live inventory kept the landing fenced away from `a1.1d-5r2-4/`, D5 review-control evidence, Linux implementation work, Windows, native proof execution, Lima mutation, Keychain work, protected lifecycle revival, and later D6 families.

## Accepted blocking findings from the initial independent review

The fresh initial review returned three accepted blocking findings:

1. `P1` — the `05-debug-regression-ledger.md` recovery bundle was not fully transferred because the `AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)` subsection remained uniquely inline in the root file, while the matching `migration/extraction-ledger.md` row claimed the full contiguous source span and therefore carried a false SHA-256 for that bundle.
2. `P2` — six `04-contracts-and-gates.md` compatibility forwarders targeted nonexistent anchors because the destination fragments omitted the double hyphen created when the em dash in each heading was stripped during GitHub-style slug generation.
3. `P2` — the extraction-ledger source anchor for the main R3 proof ledger omitted the archived-for-active-scheduling suffix and therefore did not resolve back to the preserved root compatibility anchor.

No other qualifying findings were reported in the initial review.

## Local remediation applied

The accepted-finding remediation remained bounded to four files only:

- `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`
- `llm-last-mile/runtime-refactor/r3-mac-evidence-recovery/evidence-regression.md`

That remediation:

1. added the omitted validator subsection to `r3-mac-evidence-recovery/evidence-regression.md`, including its exact source provenance and baseline SHA-256 `53c12c7d745bd67b8b15582f88beecdd4b8a4d6745c00cd2bf355c7744f1dba1`;
2. replaced the remaining root validator body in `05-debug-regression-ledger.md` with a canonical-content forwarder to the new destination anchor;
3. corrected the six `04-contracts-and-gates.md` forwarder fragments so each now matches the actual destination heading slug; and
4. corrected the `migration/extraction-ledger.md` R3 proof-ledger source heading/anchor plus the full recovery-bundle SHA-256 to `1200e5d34dbb452f8ffcb011fc7cd6cb271228e5deb9ecafdcd0e2d912852dd7`.

## Local validation evidence

- `git diff --check` passed after remediation.
- A targeted anchor-resolution script confirmed that:
  - the root `05` validator forwarder resolves into `r3-mac-evidence-recovery/evidence-regression.md`;
  - all six repaired `04` forwarder fragments resolve into `a1.1d-5r3/contracts-and-gates.md`; and
  - the repaired extraction-ledger source anchor resolves into the archived root `05` proof-ledger heading.
- Clean-baseline SHA-256 revalidation against `git show 0c47467db:llm-last-mile/runtime-refactor/05-debug-regression-ledger.md` matched:
  - `1200e5d34dbb452f8ffcb011fc7cd6cb271228e5deb9ecafdcd0e2d912852dd7` for lines `3619–3714`; and
  - `53c12c7d745bd67b8b15582f88beecdd4b8a4d6745c00cd2bf355c7744f1dba1` for lines `3653–3656`.
- The remediation follow-up delta was reconstructed exactly from the immediately pre-remediation reviewed local state and written to `/tmp/d6-r3-remediation.patch` with SHA-256 `89eb4a6e0551758f62863c88823e41d732d923223482c02366fb3b9f62245b11`.

## Remediation follow-up review verdict

The fresh remediation-follow-up review reported **no qualifying findings**.

This D6 batched R3 macOS family landing is therefore complete locally and approved. Remaining later D6 families stay blocked pending later explicit authorization.
