# ChatGPT Pro advisory review: runtime-refactor D4 current-decision consolidation

- Date: 2026-08-24
- Initial review chat: https://chatgpt.com/c/6a8c5168-2d20-83ea-b97f-ca2ba107050a
- Remediation follow-up review chat: https://chatgpt.com/c/6a8c5346-5304-83ea-97b1-21ed0df7cd98
- Visible model on both ChatGPT surfaces: `GPT-5.6 Sol`
- Visible effort setting: `Extra High`
- Frozen baseline manifest digest: `sha256:b4589d5b7a76ddc93541d4568f67011c82b90886050b61690bdfde45c771730f`
- Original reviewed patch digest: `sha256:805c9a47782519b37f67a389f640afbce9fd7c5024fb03d3e27615f6c7deb6c5`
- Remediation delta digest: `sha256:fabdd064867a6a70ca6b3f4e19623956f7fa8f65c561df7e634f7bb6cf405c40`
- Final local D4 patch digest: `sha256:827003c41f98676077bc40b7963f660098e01b99df42ed255961e5b82f89512a`
- Initial review prompt digest: `sha256:1050bdd72c4634d7f285f25dcfc8a6314cea36920a3ed872950dd899bc2295b1`
- Initial review response digest: `sha256:159fb135b00399309a2a501e71286ace41d9d20a51b6db661bbd076ae89afe0c`
- Remediation follow-up prompt digest: `sha256:cb0fd934ab380b59406ea5dca364321013a9db7c8007dedd54df8a2139245aea`
- Remediation follow-up response digest: `sha256:e9fa0f97c682699cfc55c83d79611f3ab3cbf529ee5e1fe740b6933e3da06baf`
- Remediation validation record digest: `sha256:d8b60e7c2a3a3f27714f04d5b33702b1a77a84dec7035318fc2e4882f1680a4b`

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

## D4 landing reviewed

The atomic D4 landing:

1. created `index/README.md` as an explicitly non-authoritative decision and packet-owner index;
2. created `index/current.md` as an explicitly non-authoritative current-state projection;
3. classified current scheduling sections in `00-README.md`, `02-seam-crosswalk.md`, `03-phase-slice-map.md`, and `05-debug-regression-ledger.md` as projections rather than authority;
4. extracted the two current `AUTHORITY_REQUIRED` gate bodies from `04-contracts-and-gates.md` into canonical documents beneath `gates/`, while preserving the legacy headings and subordinate anchors as direct compatibility pointers;
5. added four exact control-pack-map rows and exactly two D4 extraction-ledger rows; and
6. left all five canonical scheduling decision and packet owners byte-stable.

## Accepted blocking finding from the initial independent review

- `P2`: each D4 ledger rollback cell reversed only its own gate extraction and omitted the shared index, map-row, and scheduling-projection changes. Either recipe could therefore leave the decision index or current-state projection pointing at removed gate documents rather than reversing the atomic D4 landing.

No additional qualifying finding was reported.

## Local remediation applied

The remediation changed only `llm-last-mile/runtime-refactor/migration/extraction-ledger.md`. Both D4 rollback cells now describe the same complete atomic reversal:

- restore both exact extracted source spans in `04-contracts-and-gates.md`;
- remove both exact canonical gate destinations;
- remove `index/README.md` and `index/current.md`;
- remove all four exact D4 control-pack-map rows;
- remove the D4 current-scheduling projection metadata and compatibility notes from `00-README.md`, `02-seam-crosswalk.md`, `03-phase-slice-map.md`, and `05-debug-regression-ledger.md`;
- remove both D4 extraction-ledger entries; and
- delete the ledger only if it would otherwise be empty.

## Local validation evidence

- All eight frozen source spans recovered their baseline SHA-256 values from the landed compatibility surfaces or canonical gate destinations.
- The five canonical scheduling owners remained byte-stable.
- The current-state projection covers the controlling decision, active packet, held packets, macOS lane-local gate, bounded Attempt 4 stop, Windows deferral, and undispatched successors.
- Both legacy gate headings and subordinate compatibility anchors resolve.
- Exactly four D4 control-pack-map rows and two D4 extraction-ledger rows are present.
- All local Markdown paths and anchors in the ten-path D4 source-document fence resolve.
- `06-review-finding-inventory.md` and `review-control/` remain unchanged.
- The final local D4 patch remains inside the exact ten-path source-document fence.
- `git diff --check` and reverse-apply checks for the final patch and remediation delta passed.

## Remediation follow-up review verdict

The fresh remediation-follow-up review reported **no qualifying findings**. It explicitly accepted that both ledger cells now reverse the complete atomic D4 landing and that the bounded remediation introduced no qualifying regression.

D4 is therefore complete locally and approved. D5 and later remain blocked pending explicit later authorization.
