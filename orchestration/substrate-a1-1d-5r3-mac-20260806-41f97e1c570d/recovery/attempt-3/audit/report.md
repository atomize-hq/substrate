# A1.1d-5R3-MAC recovery attempt-3 independent audit

**Verdict: `VERIFIED_COMPLETE`**

## Exact identities and terminal hashes

- Transcript: `114a6f051814f46070e51af691ef4a6a87adf24d465f1ed500ff6a6edc331c98` (1241 lines).
- Exact base: `270f6e55e1a94b7e2f9b2667e605980d2e50579c` / `acef6844c15d17aba8cfd18d2fc7ef88d9c3420b`.
- Recovery manifest: `9edb61f5aab00299706efe3e73050886d81d80d1eedfdbf15c7d8662d595a908`.
- Recovered tree: `a023bc6e2f4929277468f5f418e954b0d7b96671d2d8c24f8790a9674763ff88`.
- Final subject fingerprint: `sha256:e60963026aa39be67e334119ff7ae61f3f15f19dbfd31539d790f8735b493924`.
- Review record: `bc7258fb432375e50af899f14691c7691980d4f99a8174b8e2d6ee2e1a29e003`.

## Result

All independent checks passed. The recovered tree contains exactly 19 regular files, with no extra, missing, symlink, or special file: 11 exact-base modifications and 8 new/untracked files. Every mode, byte size, SHA-256, and independently derived Git blob OID matches the recovery manifest and tree rows.

The historical 16-row subject-manifest algorithm was re-derived from raw transcript calls `call_Eb02M8CRfq8P03Ua4XHtYdpr` and `call_lmcD1PyIOR0nFyozkGxrlLpc`. It includes the unchanged installer fixture and excludes review-control metadata. The two terminal script blobs were independently read from the existing Git object database and match the recovered bytes.

The review record is semantically valid: exactly discovery, closure, supplemental-causal-1, and supplemental-causal-2; terminal `bounded_stop` / `budget_exhausted`; and exactly four final P1 plus two final P2 findings. No third causal cycle is authorized.

All 174 extracted exec records and all 64 chronological mutation events were verified against the raw transcript inputs and output inventories. The final non-review subject anchor is transcript line 1129; the final review-control anchor is line 1181. Static target analysis found no later product-path mutation that could make recovered bytes stale.

The replay plan uses a non-Git scratch/archive, limits materialization to the manifest-listed 19 paths, and forbids staging, commits, publication, protected-checkout edits, and remote actions. A detached exact-base worktree can be reconstructed by copying only those files with recorded modes; its base index remains untouched, producing zero staged changes.

See `report.json` for check-level/per-path coverage and `digests.json` for every rehashed bundle file.
