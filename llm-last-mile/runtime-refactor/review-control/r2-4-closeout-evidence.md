# R2-4 closeout evidence and authority record

**Status:** terminal docs/evidence closeout for `A1.1d-5R2-4`; no product behavior is added here.

## Live source and authority chain

The closeout preflight independently bound host `spenser-linux`, repository
`/home/spenser/__Active_code/substrate`, and
`refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap` at predecessor
`d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32` / tree
`b2d68905580d35d7d63aea8f36f274f723c26733` / parent
`316ee5c6cf12c060388c9d9376e0a79537f2094a`. The live origin and tracking ref were equal to that
commit, divergence was `0/0`, and the index/worktree/untracked set was clean. The predecessor
commit changed exactly:

- `scripts/substrate/install-substrate.sh`;
- `scripts/substrate/uninstall-substrate.sh`;
- `scripts/substrate/uninstall.sh`;
- `tests/installers/prefix_propagation_r2_2.sh`.

The earlier R2-4 dispatch identity is not discarded or shortened. Its canonical preflight at
`/home/spenser/.codex/visualizations/2026/08/01/r2-4-cc650df599d5f8db9a1f4502b1a9d75f068e147c502dc99d0fa7fe0bce8e8197/preflight/preflight-record.json`
(SHA-256 `8763af1455861b6dec2c4b1feab3a1ccd23026879ff6e72e07c22395d4e61e60`)
bound orchestration `substrate-a1-1d-5r2-4-20260731`, dispatch nonce
`cc650df599d5f8db9a1f4502b1a9d75f068e147c502dc99d0fa7fe0bce8e8197`, task
`019fbb7e-e2bb-79f3-b73d-24c66d2da9ee`, the same target ref, and exact base
`9e7b4b48e92864be6970ad373c35cb8bf18593b0` / tree
`71ab2e5675f251f5ac74eed98ea535d5c3aa10f2` / parent
`34084c963b14f086f207cd4984bff11cec199c30`. That commit is the historical
`docs: refresh GitNexus index counts` checkpoint. It remains an ancestor of the current ref.

The later resume-3 preflight at
`/home/spenser/.codex/visualizations/2026/08/01/r2-4-cc650df599d5f8db9a1f4502b1a9d75f068e147c502dc99d0fa7fe0bce8e8197/resume-3/preflight/resume-preflight-record.json`
(SHA-256 `24efb6577064e606bfaf46d3d2475b7b773bab016544bc5fe6e2dc3ccfc619ec`)
truthfully stopped when the clean local checkout had advanced one commit to
`316ee5c6cf12c060388c9d9376e0a79537f2094a` / tree
`1eae07018caef023b2f27ef22892825b140e9a4d` while origin still held
`9e7b4b48e92864be6970ad373c35cb8bf18593b0`. Its blocked
receipt is preserved at `resume-3/receipts/blocked-receipt.json` with SHA-256
`7cf4fd4af3aeaf2c7811812e242399078cf363ff45617daaa9bf504bc25d0dbd`. No reset, rebase, merge,
clean, or rewrite reconciled that stop. Instead,
`316ee5c6cf12c060388c9d9376e0a79537f2094a` was later accepted as the bounded Linux-evidence
source and became the published parent of correction
`d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32` in the normal fast-forward lineage. The exact
ancestry is therefore:

```text
9e7b4b48e92864be6970ad373c35cb8bf18593b0
  -> 316ee5c6cf12c060388c9d9376e0a79537f2094a
  -> d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32
```

## Immutable evidence bindings

| Evidence | Source binding | Immutable verification |
|---|---|---|
| Earlier bounded Linux product evidence, task `019fc04c-5400-7520-a7d9-407ae8e1783f` | `316ee5c6cf12c060388c9d9376e0a79537f2094a` / `1eae07018caef023b2f27ef22892825b140e9a4d` | `resume-5/MANIFEST.sha256` SHA-256 `bedf46519c368cc7eff4f4681560115f0f84b5bb9e6c4e190d4103f6a2842ebb`; all 131 entries verified; `resume-5/result.json` SHA-256 `35e77a683cfec6589776b974e6c2d511ef14d7a5760bb5ebf157bc71c41ebc28`; no separate resume-5 receipt file exists. |
| Landed installer correction and corrected product proof, task `019fc2e6-8ae1-7be2-bbbe-266c33b5225b` | proof bundle bound to `316ee5c6cf12c060388c9d9376e0a79537f2094a` plus explicit corrected-script hashes; landed child `d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32` / `b2d68905580d35d7d63aea8f36f274f723c26733` | `resume-6/MANIFEST.sha256` SHA-256 `e8aaf84786c2a8bf99615a463dccf1259b1a8e5540852636762274ff5b11bb38`; all 153 entries verified; `resume-6/result.json` SHA-256 `0a87fadd0cd22b406b7305c43400a10fe2f40541e4a5dfb29ce8d00b20932547`; legacy plain-text `resume-6/receipt.txt` SHA-256 `fe3d83727331ac3ae2dd0ee1491687f316534df47e33ff97c1a57a7fe625e620`. |

The resume-5 bundle source identity and preflight Git identity are themselves manifest-bound at
SHA-256 `1b38589854e158718115e47a042d61ff3e875c96155a64ad8c7a2aed016c399a` and
`a915f90e5a55a9bfd875700ca778beb651ecbcb110f381dd5b685890158f4811`. Resume-6's final bundle
source-identity record is manifest-bound at SHA-256
`97f168b5508e8ce9afc855e7409bf45a998418832d01466a7d5808be037d83f4`; its terminal result binds
the landed commit/tree/parent, exact four-file surface, live-origin equality, and clean checkout.

### Receipt-propagation deviation

The original `resume-6/receipt.txt` is immutable plain text, not a
`codex.top-level-task-receipt.v1` JSON document. No replacement or historical JSON receipt was
manufactured. The original predecessor turn also omitted the required terminal propagation to the
parent/meta task. A later receipt-only recovery turn sent that exact immutable receipt to parent
task `019fbeb0-c345-73b2-982a-e1fbe6dfc792` as its final tool action. Before authorizing this
closeout to resume, the parent independently reverified the target ref, HEAD/tree/parent, live-
origin parity, `0/0` divergence, clean checkout, exact four-path predecessor surface, and the
`receipt.txt` and `result.json` SHA-256 values above. This recovery closes propagation for the
bounded handoff; it is not represented as native JSON-protocol compliance.

## Adopted proof matrix

| R2-4 proof | Terminal disposition |
|---|---|
| Ordinary custom-A install under hostile ambient B | PASS. Corrected proof explicitly unset `SUBSTRATE_INSTALL_NO_PATH`; explicit `--prefix` selected custom A while ambient `HOME`, `SUBSTRATE_HOME`, and `SUBSTRATE_ROOT` selected hostile B. |
| Normal first/repeat commitment | PASS. Both commitments are exactly `3c57e0459af3e9e09bef46773e832e57bd3a0e5d14cac5ea51d81395c7bc8cbd`. |
| Shell-profile repeat stability | PASS. `.bashrc` and `.bash_profile` were byte-stable across repeat install and each contained exactly one selected-A managed block. |
| Public uninstall wrapper symmetry | PASS. The public wrapper completed with exit `0` and its child authenticated the same explicit custom-A context. |
| Hostile B | PASS. B remained non-authoritative and unchanged through first install, repeat install, and public uninstall. |
| Restoration | PASS. Default A, system root, profiles, service files, socket ACL, active/enabled units, unrelated state, and the bounded cache baseline were restored; custom A and B were absent; sudo credentials were invalidated. |
| PI-050 guardrail | Satisfied without reclassification. The common ambient resolver remains `OutOfScope`; explicit leaf context, not B, controlled the joined proof. |
| Bounded world/Codex reachability | Adopted only as a quick reachability observation. The earlier manual record reports a real world-backed `pwd` and in-world `codex-cli 0.125.0`; resume-5 independently retains an `ok` world doctor with socket `probe_ok=true` and custom-A commitment plus first/repeat `codex-runtime-version.txt` files containing `codex-cli 0.125.0`. The manifest binds those files at SHA-256 `9f90c311f5a8f0e1756430a9af646099eff0364941bfe3db9a415cae2d6e2f52`, `45b2ddd6dee2d47b2ab5cac855c6e6edc023d3b7ad5f4dea3c7758ad1e0dbdaa`, and `5090de9b4395d2d7dd6e259fefc7a70a02aedbdc5019614b84619bb05461e63b`. This is not authenticated Codex execution or complete direct-member architecture proof. |

Resume-5 remains immutable history. Its completed first/repeat installs used the PATH bypass, and
its public wrapper did not complete. Those two limitations are corrected and superseded only by
the landed resume-6 proof; they are not retroactively rewritten as resume-5 successes.

## Exact 0640/0650 cache decision

This decision applies only to
`/var/lib/substrate/world-deps/codex-runtime/downloads/codex-x86_64-unknown-linux-musl.tar.gz`.
The accepted earlier archive recorded mode `0640`. At resume-6 continuation, the exact live object
was accepted as a `0650` baseline because all of the following were simultaneously true:

- SHA-256 `4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001`;
- regular file, not a symlink, owned by `root:substrate`;
- ACL `user::rw-`, `user:spenser:r-x`, `group::r-x`, `mask::r-x`, `other::---`;
- no group or other write permission; and
- the install/sync proof left mode `0650` unchanged and terminal restoration reproduced exact
  `0650` continuation parity.

This is a bounded non-semantic local cache/archive-mode nuance. It is not a general permission
relaxation, a policy for other cache objects, or a claim about modes guaranteed by upstream
GitHub artifacts.

## Exclusions and deferred ownership

R2-4 does not prove or close authenticated Codex execution; retained world workers or tasks;
authoritative-session repair/refresh; orchestrator packet-3 lifecycle/routing; direct-member
world/Codex architecture as a whole; gateway adoption; passive health/world-deps diagnostic
failures; uninstall leftovers or complete cleanup; rollback, manifest, or convergence; R3; or the
whole runtime refactor.

Passive health/world-deps failures remain in their existing separate diagnostic lane. Uninstall
leftovers and all installer/uninstaller cleanup, rollback, managed-artifact manifest, and
convergence actions remain exclusively assigned to R3. Retained-worker/task,
authoritative-session, and orchestrator packet-3 lifecycle/routing ownership remains open and
unchanged. No product, test, dependency, generated artifact, adjacent packet, or finding-inventory
file is changed by this closeout.
